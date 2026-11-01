use std::collections::BTreeSet;

use bon::Builder;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::ownership::Invalidation;
use zakac_diagnostics::ownership::overflow_proven;
use zakac_ir_builder::ir::Dest;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::InfixOp;
use zakac_ir_builder::ir::LogicOp;
use zakac_ir_builder::ir::Module;
use zakac_ir_builder::ir::Place;
use zakac_ir_builder::ir::PrefixOp;
use zakac_ir_builder::ir::PrintValue;
use zakac_ir_builder::ir::SliceFrom;
use zakac_ir_builder::ir::Stmt;
use zakac_ir_builder::value::Value;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_false;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;
use zakac_span::span::Span;

use crate::cfg::Cfg;
use crate::cfg::NodeId;
use crate::cfg::NodeKind;
use crate::cfg::build_cfg;
use crate::checks::bounds::check_cell;
use crate::checks::bounds::check_offset;
use crate::checks::bounds::check_ptr_sub;
use crate::checks::calls::all_given_back;
use crate::checks::calls::check_call;
use crate::checks::calls::given_back;
use crate::checks::calls::lift_reads;
use crate::checks::calls::note_read;
use crate::checks::calls::note_unfilled;
use crate::checks::escape::check_result_set;
use crate::checks::escape::check_return;
use crate::checks::leaks::check_exit;
use crate::checks::leaks::check_overwrite;
use crate::checks::leaks::return_span;
use crate::checks::lists::store_into_cell;
use crate::checks::loans::check_heap;
use crate::checks::loans::check_losses;
use crate::checks::loans::holds_blank;
use crate::checks::moves::check_loop_moves;
use crate::checks::moves::check_read;
use crate::checks::moves::run_take;
use crate::checks::nullness::check_deref;
use crate::checks::report;
use crate::checks::static_rules::check_free_dest;
use crate::checks::static_rules::check_global_set;
use crate::checks::static_rules::check_stored_borrow;
use crate::digest::FnDigest;
use crate::digest::Root;
use crate::digest::Seg;
use crate::interval::addressed_slots;
use crate::interval::arith;
use crate::interval::counted_range;
use crate::interval::elements;
use crate::interval::int_of;
use crate::liveness::LiveMap;
use crate::liveness::global_reads;
use crate::liveness::live_out;
use crate::narrow::int_slot;
use crate::narrow::narrow;
use crate::narrow::null_compared;
use crate::narrow::null_compared_cell;
use crate::options::BorrowCheckOptions;
use crate::options::OverflowMode;
use crate::roots::RootCtx;
use crate::roots::Roots;
use crate::roots::filled_path;
use crate::roots::freed_paths;
use crate::roots::known_cells;
use crate::roots::past_cells;
use crate::roots::roots_of;
use crate::roots::value_roots;
use crate::routine::Routine;
use crate::state::Interval;
use crate::state::OwnerState;
use crate::state::PtrInfo;
use crate::state::SlotState;
use crate::state::State;
use crate::walk::each_expr;
use crate::walk::sub_exprs;

/// A block that loses its owner in the node that runs.
#[derive(Debug, Clone, Builder)]
pub(crate) struct Loss
{
    /// The paths that lose their block.
    pub(crate) paths: Roots,
    /// What happens to the block.
    pub(crate) what: Invalidation,
    /// The operation.
    pub(crate) span: Span,
    /// The paths the result of a call points into: the callee hands them
    /// back, so the slot the node sets from that result holds no loan on
    /// them.
    #[builder(default)]
    pub(crate) returned: Roots,
    /// A loan the callee itself holds on a block the call frees, through
    /// a parameter or a global it reads, the call result read on the spot,
    /// or a borrow the rest of the node holds across the call: reported
    /// when no loan of the routine is: one report per call is enough.
    pub(crate) clash: Option<Diagnostic>,
}

/// What the flow of one routine found.
#[derive(Debug, Clone, PartialEq, Builder)]
pub(crate) struct FlowResult
{
    /// The digest of the routine.
    pub(crate) digest: FnDigest,
    /// The diagnostics, in the order the report pass met them.
    pub(crate) diags: Vec<Diagnostic>,
    /// The cell addresses proven in range.
    pub(crate) proofs: Vec<Span>,
    /// The cell addresses not proven in range.
    pub(crate) doubts: Vec<Span>,
}

/// The forward flow of one routine: the facts at each node, then one pass
/// that reports.
pub(crate) struct Flow<'m>
{
    /// The module.
    pub(crate) module: &'m Module,
    /// The routine.
    pub(crate) routine: Routine<'m>,
    /// Its graph.
    pub(crate) cfg: Cfg<'m>,
    /// The borrow slots live after each node.
    pub(crate) live: Vec<LiveMap>,
    /// The node the transfer runs.
    pub(crate) node: NodeId,
    /// The blocks the node that runs takes from their owners, through
    /// moves out and calls, in the report pass: each is checked against
    /// the loans once the node ends, when a borrow it sets holds its new
    /// roots.
    pub(crate) losses: Vec<Loss>,
    /// What [`roots_of`] reads besides the state.
    pub(crate) ctx: RootCtx<'m>,
    /// How to report.
    pub(crate) options: &'m BorrowCheckOptions,
    /// The bit count of the platform-sized integer kinds.
    pub(crate) platform: u16,
    /// The facts before each node, once a path reaches it.
    pub(crate) ins: Vec<Option<State>>,
    /// Whether the transfer runs in the report pass.
    pub(crate) reporting: bool,
    /// Whether the report pass reports reads of moved-out owners alone,
    /// as it does when it runs again over the loops. That second run
    /// starts the loops from facts that leave out the edge into them, so
    /// nothing it finds holds for the routine, save a read of a moved-out
    /// owner. Each write other than such a diagnostic (a proof, what the
    /// digest gets, each other finding) must sit behind
    /// [`Flow::recording`].
    pub(crate) moves_alone: bool,
    /// The diagnostics found.
    pub(crate) diags: Vec<Diagnostic>,
    /// The cell addresses proven in range each time the report pass met
    /// them.
    pub(crate) proofs: Vec<Span>,
    /// The cell addresses the report pass met without a proof.
    pub(crate) doubts: Vec<Span>,
    /// What the routine frees through its parameters and the globals.
    pub(crate) frees: BTreeSet<Root>,
    /// The paths from the parameters and the globals a free left without
    /// a block on some path to an exit of the routine.
    pub(crate) unfilled: BTreeSet<Root>,
    /// The paths from the parameters and the globals the routine reads or
    /// writes through, itself or in a call, in whatever order against its
    /// frees.
    pub(crate) reads: BTreeSet<Root>,
    /// The globals a function can name, which a call can write; see
    /// [`named_globals`].
    pub(crate) named: Vec<Place>,
    /// The slots a write the flow does not follow can change, see
    /// [`addressed_slots`]: an integer among them proves nothing.
    pub(crate) addressed: BTreeSet<Place>,
    /// The global borrows each function reads, itself or through its
    /// callees, in function order.
    pub(crate) global_uses: Vec<BTreeSet<u32>>,
    /// The parameters the result can point into.
    pub(crate) result_from: BTreeSet<u32>,
    /// What the result can point into, as paths from the parameters it
    /// borrows through and from the globals.
    pub(crate) result_roots: BTreeSet<Root>,
    /// Whether the result points into nothing that dies with the call.
    pub(crate) result_roots_ok: bool,
}

/// Run the flow of one routine.
///
/// # Arguments
///
/// * `module` - The module.
/// * `routine` - The routine.
/// * `digests` - The digest of each function so far.
/// * `global_roots` - The roots top-level code gives each global.
/// * `options` - How to report.
/// * `platform` - The bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// The digest of the routine, its diagnostics and its proofs.
pub(crate) fn run_flow<'m>(
    module: &'m Module, routine: Routine<'m>, digests: &'m [FnDigest],
    global_roots: &'m [Roots], options: &'m BorrowCheckOptions, platform: u16,
) -> FlowResult
{
    let mut flow = Flow::new(module, routine, digests, global_roots, options, platform);
    flow.solve();
    flow.report_pass();
    check_loop_moves(&mut flow);
    flow.finish()
}

impl<'m> Flow<'m>
{
    /// A flow with no fact found.
    ///
    /// # Arguments
    ///
    /// * `module` - The module.
    /// * `routine` - The routine.
    /// * `digests` - The digest of each function so far.
    /// * `global_roots` - The roots top-level code gives each global.
    /// * `options` - How to report.
    /// * `platform` - The bit count of the platform-sized integer kinds.
    ///
    /// # Returns
    ///
    /// The flow.
    pub(crate) fn new(
        module: &'m Module, routine: Routine<'m>, digests: &'m [FnDigest],
        global_roots: &'m [Roots], options: &'m BorrowCheckOptions, platform: u16,
    ) -> Self
    {
        let cfg = build_cfg(routine.block, routine.span);
        let live = live_out(&cfg, module, &routine);
        let ctx = RootCtx::builder()
            .module(module)
            .locals(routine.locals)
            .global_roots(global_roots)
            .digests(digests)
            .build();
        let ins = vec![None; cfg.nodes.len()];
        let node = cfg.start;
        Self {
            module,
            routine,
            cfg,
            live,
            node,
            losses: Vec::new(),
            ctx,
            options,
            platform,
            ins,
            reporting: false,
            moves_alone: false,
            diags: Vec::new(),
            proofs: Vec::new(),
            doubts: Vec::new(),
            frees: BTreeSet::new(),
            unfilled: BTreeSet::new(),
            reads: BTreeSet::new(),
            named: named_globals(module),
            addressed: addressed_slots(module, &routine),
            global_uses: global_reads(module),
            result_from: BTreeSet::new(),
            result_roots: BTreeSet::new(),
            result_roots_ok: true,
        }
    }

    /// Whether the transfer runs in the report pass over the facts the
    /// routine reaches, so it can keep proofs, digest material and
    /// diagnostics of each code.
    ///
    /// # Returns
    ///
    /// `true` in the report pass, `false` in the solve and in the second
    /// run over the loops.
    pub(crate) fn recording(&self) -> bool { self.reporting && !self.moves_alone }

    /// Find the facts before each node.
    pub(crate) fn solve(&mut self)
    {
        let blank = vec![None; self.cfg.nodes.len()];
        self.ins = self.fixpoint(&blank);
    }

    /// The facts before each node: a worklist in reverse postorder from
    /// the start, one out-state per successor edge, widened at loop tests.
    /// A node with pinned facts keeps them, whatever reaches it.
    ///
    /// # Arguments
    ///
    /// * `pinned` - The facts fixed at some nodes, one for each node.
    ///
    /// # Returns
    ///
    /// The facts before each node a path reaches.
    pub(crate) fn fixpoint(&mut self, pinned: &[Option<State>]) -> Vec<Option<State>>
    {
        let rank = self.ranks();
        let mut ins = pinned.to_vec();
        let mut start =
            State::at_start(self.module, &self.routine, self.ctx.global_roots);
        for &place in &self.addressed
        {
            let int = matches!(start.get(place), SlotState::Int(_));
            do_if!(int, start.set(place, SlotState::Int(Interval::TOP)));
        }
        ins[self.cfg.start as usize] = Some(start);
        let queued =
            (0..ins.len()).filter(|&n| ins[n].is_some() && rank[n] != usize::MAX);
        let mut work: BTreeSet<usize> = queued.map(|n| rank[n]).collect();
        while let Some(i) = work.pop_first()
        {
            let n = self.cfg.rpo[i];
            let state = ins[n as usize].clone().expect("a queued node is reached");
            let outs = self.transfer(n, state);
            let succ = self.cfg.nodes[n as usize].succ.clone();
            for (s, out) in succ.into_iter().zip(outs)
            {
                let open = pinned[s as usize].is_none();
                do_if!(open && self.merge_into(&mut ins, s, out), {
                    work.insert(rank[s as usize]);
                });
            }
        }
        ins
    }

    /// The place of each node in reverse postorder.
    ///
    /// # Returns
    ///
    /// The rank of each node, [`usize::MAX`] for a node the start does
    /// not reach.
    pub(crate) fn ranks(&self) -> Vec<usize>
    {
        let mut rank = vec![usize::MAX; self.cfg.nodes.len()];
        do_for!((i, &n) in self.cfg.rpo.iter().enumerate(), rank[n as usize] = i);
        rank
    }

    /// Join an out-state into the facts before a node, widened at a loop
    /// test.
    ///
    /// # Arguments
    ///
    /// * `ins` - The facts before each node, changed in place.
    /// * `n` - The node.
    /// * `out` - The out-state of an edge into it.
    ///
    /// # Returns
    ///
    /// `true` when the facts before the node changed.
    fn merge_into(&self, ins: &mut [Option<State>], n: NodeId, out: State) -> bool
    {
        let head = is_loop_test(&self.cfg.nodes[n as usize].kind);
        let slot = &mut ins[n as usize];
        let Some(old) = slot
        else
        {
            *slot = Some(out);
            return true;
        };
        let joined = old.join(&out);
        let next = tern!(head, joined.widen(old), joined);
        ret_if!(next == *old, false);
        *slot = Some(next);
        true
    }

    /// Run the transfer of each reached node once more with its final
    /// facts, so the checks report.
    pub(crate) fn report_pass(&mut self)
    {
        self.reporting = true;
        for n in self.cfg.rpo.clone()
        {
            let Some(state) = self.ins[n as usize].clone()
            else
            {
                continue;
            };
            self.transfer(n, state);
        }
        self.reporting = false;
    }

    /// The facts after a node, one state per successor edge.
    ///
    /// # Arguments
    ///
    /// * `n` - The node.
    /// * `state` - The facts before it.
    ///
    /// # Returns
    ///
    /// The out-states, in the order of the successors.
    pub(crate) fn transfer(&mut self, n: NodeId, state: State) -> Vec<State>
    {
        self.node = n;
        let outs = self.node_outs(n, state);
        let losses = std::mem::take(&mut self.losses);
        do_if!(let Some(out) = outs.first(), check_losses(self, &losses, out));
        outs
    }

    /// The facts after a node, before the moves it makes are checked.
    ///
    /// # Arguments
    ///
    /// * `n` - The node.
    /// * `state` - The facts before it.
    ///
    /// # Returns
    ///
    /// The out-states, in the order of the successors.
    fn node_outs(&mut self, n: NodeId, mut state: State) -> Vec<State>
    {
        match self.cfg.nodes[n as usize].kind
        {
            NodeKind::Start => vec![state],
            NodeKind::End =>
            {
                check_exit(self, self.routine.span.shrink_to_hi(), &state);
                self.leave(&state);
                vec![state]
            }
            NodeKind::Exit => Vec::new(),
            NodeKind::Leaf(s) =>
            {
                self.stmt(s, &mut state);
                vec![state]
            }
            NodeKind::Branch(cond) | NodeKind::LoopHead(cond) =>
            {
                self.split(cond, state)
            }
            NodeKind::CountedInit(s) =>
            {
                self.counted_init(s, &mut state);
                vec![state]
            }
            NodeKind::CountedHead(s) => self.counted_head(s, state),
        }
    }

    /// The two out-states of a test: narrowed where it holds, then where
    /// it does not.
    ///
    /// # Arguments
    ///
    /// * `cond` - The test.
    /// * `state` - The facts before it.
    ///
    /// # Returns
    ///
    /// The two states.
    fn split(&mut self, cond: &Expr, mut state: State) -> Vec<State>
    {
        self.value(cond, &mut state);
        let ints = (!has_effects(cond)).then_some(self.platform);
        let mut taken = state.clone();
        narrow(cond, true, &mut taken, ints);
        narrow(cond, false, &mut state, ints);
        vec![taken, state]
    }

    /// Run the bounds of a counted loop once and set its variable to the
    /// first value.
    ///
    /// # Arguments
    ///
    /// * `s` - The counted loop.
    /// * `state` - The facts, changed in place.
    fn counted_init(&mut self, s: &Stmt, state: &mut State)
    {
        let Stmt::Counted { var, from, to, step, .. } = s
        else
        {
            return;
        };
        let first = self.value(from, state);
        self.value(to, state);
        do_if!(let Some(step) = step, { self.value(step, state); });
        state.set(*var, self.trusted(*var, first));
    }

    /// The two out-states of the step of a counted loop: into the loop
    /// with the variable at the values it takes, a `break` aside, and out
    /// of it with the variable not known.
    ///
    /// # Arguments
    ///
    /// * `s` - The counted loop.
    /// * `state` - The facts before the step.
    ///
    /// # Returns
    ///
    /// The two states.
    fn counted_head(&mut self, s: &Stmt, mut state: State) -> Vec<State>
    {
        let Stmt::Counted { var, from, to, step, mode, .. } = s
        else
        {
            return vec![state.clone(), state];
        };
        let step = step.as_ref();
        let range = counted_range(from, to, step, *mode, &state, self.platform);
        let mut inside = state.clone();
        inside.set(*var, self.trusted(*var, SlotState::Int(range)));
        state.set(*var, SlotState::Int(Interval::TOP));
        vec![inside, state]
    }

    /// Run one statement with no statement nested in it.
    ///
    /// # Arguments
    ///
    /// * `s` - The statement.
    /// * `state` - The facts, changed in place.
    fn stmt(&mut self, s: &Stmt, state: &mut State)
    {
        match s
        {
            Stmt::Set { place, value, span } =>
            {
                check_global_set(self, *place, value, *span);
                check_result_set(self, *place, value, *span, state);
                let v = self.moved(value, state);
                check_overwrite(self, *place, *span, state);
                self.assign(*place, v, state);
            }
            Stmt::Print { value: PrintValue::Expr(e), .. } =>
            {
                self.value(e, state);
            }
            Stmt::Call { func, args, span, .. } => self.call(*func, args, *span, state),
            Stmt::Store { addr, value, span } =>
            {
                check_stored_borrow(self, addr, value, *span);
                let at = self.value(addr, state);
                self.deref(addr, state);
                check_offset(self, &at, addr.span, true);
                let stored = self.moved(value, state);
                self.havoc_through(addr, state);
                store_into_cell(self, addr, &stored, state);
            }
            Stmt::Return { value, span } => self.ret(value.as_ref(), *span, state),
            Stmt::Alloc { dest, count, unit, span } =>
            {
                check_free_dest(self, dest, *span);
                let block = self.new_block(dest, count, *unit, state);
                do_if!(let Dest::Slot(place) = dest, {
                    check_overwrite(self, *place, *span, state);
                });
                self.fill(dest, state);
                self.write_dest(dest, block, state);
            }
            Stmt::Resize { dest, count, unit, span } =>
            {
                self.give_back(dest, Invalidation::Resized, *span, state);
                self.forget_freed(dest, state);
                let block = self.new_block(dest, count, *unit, state);
                self.fill(dest, state);
                self.write_dest(dest, block, state);
            }
            Stmt::Free { dest, span } =>
            {
                let counted = self.give_back(dest, Invalidation::Freed, *span, state);
                do_if!(counted, self.leave_unfilled(dest, state));
                self.forget_freed(dest, state);
                self.write_dest(dest, SlotState::Ptr(PtrInfo::null()), state)
            }
            Stmt::Drop { dest, kind, span } =>
            {
                self.give_back(dest, Invalidation::Freed, *span, state);
                self.forget_freed(dest, state);
                self.write_dest(dest, SlotState::blank(*kind), state)
            }
            _ => (),
        }
    }

    /// Leave the routine: the value given back moves out, then each owner
    /// that still holds a block is lost.
    ///
    /// # Arguments
    ///
    /// * `value` - The value given back, if the function has a result.
    /// * `span` - The return.
    /// * `state` - The facts, changed in place.
    fn ret(&mut self, value: Option<&Expr>, span: Span, state: &mut State)
    {
        do_if!(let Some(v) = value, {
            check_return(self, v, span, state);
            self.moved(v, state);
        });
        check_exit(self, return_span(self, value, span), state);
        self.leave(state);
    }

    /// Keep, at an exit of the routine, the paths a free left without a
    /// block: a result path among them is no renewal, and a caller finds
    /// the owner there `ဗလာ`.
    ///
    /// # Arguments
    ///
    /// * `state` - The facts at the exit.
    fn leave(&mut self, state: &State)
    {
        ret_if!(!self.recording());
        self.unfilled.extend(state.unfilled.iter().cloned());
    }

    /// Check a free or a resize: a borrow cannot give a block back, and a
    /// block an owner gives back ends each loan on it. A rejected one
    /// gives nothing back, so the digest does not count it.
    ///
    /// # Arguments
    ///
    /// * `dest` - The destination.
    /// * `what` - What happens to the block.
    /// * `span` - The statement.
    /// * `state` - The facts before it runs.
    ///
    /// # Returns
    ///
    /// `true` when the block is given back, `false` for a rejected one.
    fn give_back(
        &mut self, dest: &Dest, what: Invalidation, span: Span, state: &State,
    ) -> bool
    {
        ret_false!(check_free_dest(self, dest, span));
        check_heap(self, dest, what, span, state);
        true
    }

    /// Note the paths from a parameter or a global a free leaves without
    /// a block, until an allocation fills them again.
    ///
    /// # Arguments
    ///
    /// * `dest` - The destination of the free.
    /// * `state` - The facts, changed in place.
    fn leave_unfilled(&self, dest: &Dest, state: &mut State)
    {
        ret_if!(self.routine.is_main);
        let params = self.routine.params;
        let paths = freed_paths(dest, state, &self.ctx);
        let outer = paths.into_iter().filter(|r| r.is_outer(params));
        state.unfilled.extend(outer);
    }

    /// Drop the paths an allocation fills again: those at or inside the
    /// one block it writes, when the destination names one for certain.
    ///
    /// # Arguments
    ///
    /// * `dest` - The destination of the allocation.
    /// * `state` - The facts, changed in place.
    fn fill(&self, dest: &Dest, state: &mut State)
    {
        ret_if!(self.routine.is_main || state.unfilled.is_empty());
        let Some(filled) = filled_path(dest, state, &self.ctx)
        else
        {
            return;
        };
        state.unfilled.retain(|r| !(known_cells(r) && filled.reaches(r)));
    }

    /// Check a read or a write through an address, and note each path
    /// from a parameter or a global it reaches.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address.
    /// * `state` - The facts when it runs.
    pub(crate) fn deref(&mut self, addr: &Expr, state: &State)
    {
        check_deref(self, addr, state);
        note_read(self, addr, state);
    }

    /// The new block of an allocation or a resize, once its count runs.
    /// Its element count is in elements of the kind the owner points at.
    ///
    /// # Arguments
    ///
    /// * `dest` - Where the owner of the block lives.
    /// * `count` - The element count.
    /// * `unit` - The element kind; `None` for a count of octets.
    /// * `state` - The facts, changed in place.
    ///
    /// # Returns
    ///
    /// The fact of the owner of the new block.
    fn new_block(
        &mut self, dest: &Dest, count: &Expr, unit: Option<DataKind>, state: &mut State,
    ) -> SlotState
    {
        let n = int_of(&self.value(count, state));
        let owner = match dest
        {
            Dest::Slot(place) => self.ctx.kind_of(*place),
            Dest::Mem(addr) => addr.kind.pointee(),
        };
        let len = elements(n, unit, owner.and_then(DataKind::pointee), self.platform);
        let info = PtrInfo::builder()
            .state(OwnerState::FRESH)
            .offset(Interval::point(0))
            .maybe_len(len)
            .build();
        SlotState::Ptr(info)
    }

    /// Store a new owner fact at a heap destination: in its slot, or, for
    /// a cell in mem, make each slot the address can name not known.
    ///
    /// # Arguments
    ///
    /// * `dest` - The destination.
    /// * `fact` - The fact of the owner there now.
    /// * `state` - The facts, changed in place.
    fn write_dest(&mut self, dest: &Dest, fact: SlotState, state: &mut State)
    {
        match dest
        {
            Dest::Slot(place) => state.set(*place, fact),
            Dest::Mem(addr) =>
            {
                let at = self.value(addr, state);
                self.deref(addr, state);
                check_offset(self, &at, addr.span, true);
                self.havoc_through(addr, state);
            }
        }
    }

    /// After a write through an address, make the slots it names
    /// themselves, not a block one of them holds, not known. An address
    /// through a parameter can name each global integer whose address the
    /// program takes, and one with no root known each such integer; the
    /// facts of a pointer follow the digests and the loans instead.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address.
    /// * `state` - The facts, changed in place.
    pub(crate) fn havoc_through(&self, addr: &Expr, state: &mut State)
    {
        let roots = roots_of(addr, state, &self.ctx);
        let slots = roots.iter().filter(|r| !r.path.contains(&Seg::Block));
        let mut places: BTreeSet<Place> = slots.map(|r| r.place).collect();
        let params = self.routine.params;
        let outer = roots.is_empty() || roots.iter().any(|r| r.is_param(params));
        let global = |p: &&Place| matches!(p, Place::Global(_));
        let reachable = self.addressed.iter().filter(|p| roots.is_empty() || global(p));
        let reachable = reachable.filter(|p| holds_int(**p, state));
        do_if!(outer, places.extend(reachable));
        do_for!(place in places, self.reset(place, state));
    }

    /// Make a slot not known, keeping the roots of a borrow, which no
    /// write can change.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    /// * `state` - The facts, changed in place.
    fn reset(&self, place: Place, state: &mut State)
    {
        let Some(kind) = self.ctx.kind_of(place)
        else
        {
            return;
        };
        let roots = state.ptr(place).filter(|_| kind.holds_borrow());
        let roots = roots.map(|info| info.roots.clone()).unwrap_or_default();
        state.set(place, SlotState::unknown(kind, roots));
    }

    /// Give a slot the value of an assignment.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    /// * `v` - The value.
    /// * `state` - The facts, changed in place.
    fn assign(&self, place: Place, v: SlotState, state: &mut State)
    {
        let Some(kind) = self.ctx.kind_of(place)
        else
        {
            return;
        };
        let owner = kind.is_owner_ptr();
        let fact = match v
        {
            SlotState::Ptr(info) if owner =>
            {
                let offset = Interval::point(0);
                SlotState::Ptr(PtrInfo { roots: Roots::new(), offset, ..info })
            }
            SlotState::None => SlotState::unknown(kind, Roots::new()),
            other => other,
        };
        state.set(place, self.trusted(place, fact));
    }

    /// The fact a slot can keep: an integer whose slot a write the flow
    /// does not follow can change is not known.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    /// * `fact` - The fact found.
    ///
    /// # Returns
    ///
    /// The fact, or [`Interval::TOP`] for such an integer.
    fn trusted(&self, place: Place, fact: SlotState) -> SlotState
    {
        let int = matches!(fact, SlotState::Int(_));
        tern!(
            int && self.addressed.contains(&place),
            SlotState::Int(Interval::TOP),
            fact
        )
    }

    /// The value of an expression headed for a place that takes
    /// ownership: a slot read of a move kind there moves out of the slot.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    /// * `state` - The facts, changed in place.
    ///
    /// # Returns
    ///
    /// The value.
    fn moved(&mut self, e: &Expr, state: &mut State) -> SlotState
    {
        let ExprNode::Read(place) = e.node
        else
        {
            return self.value(e, state);
        };
        ret_if!(
            !e.kind.is_move_kind() || self.ctx.is_const(place),
            self.value(e, state)
        );
        check_read(self, place, e.span, state, false);
        let value = held_value(place, e.kind, state);
        leave_moved(place, e.kind, state);
        value
    }

    /// The value of an expression, after the moves and the calls in it.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    /// * `state` - The facts, changed in place.
    ///
    /// # Returns
    ///
    /// The value; a pointer value carries its roots.
    fn value(&mut self, e: &Expr, state: &mut State) -> SlotState
    {
        let v = self.node_value(e, state);
        let roots = || roots_of(e, state, &self.ctx);
        match v
        {
            SlotState::Ptr(info) => SlotState::Ptr(PtrInfo { roots: roots(), ..info }),
            SlotState::List(info) =>
            {
                SlotState::List(PtrInfo { roots: roots(), ..info })
            }
            other => other,
        }
    }

    /// The value of an expression, without its roots.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    /// * `state` - The facts, changed in place.
    ///
    /// # Returns
    ///
    /// The value.
    fn node_value(&mut self, e: &Expr, state: &mut State) -> SlotState
    {
        match &e.node
        {
            ExprNode::Read(place) =>
            {
                check_read(self, *place, e.span, state, false);
                read_value(*place, e.kind, state)
            }
            ExprNode::Compare { lhs, rhs, .. } => self.compare(e, lhs, rhs, state),
            ExprNode::Logic { op, lhs, rhs } => self.logic(e, *op, lhs, rhs, state),
            ExprNode::Load { addr } =>
            {
                let at = self.value(addr, state);
                self.load(e, addr, &at, state)
            }
            ExprNode::Take { addr } => self.take(e, addr, state),
            ExprNode::Call { func, args, .. } =>
            {
                self.call(*func, args, e.span, state);
                SlotState::unknown(e.kind, Roots::new())
            }
            ExprNode::List(cells) =>
            {
                let values: Vec<SlotState> =
                    cells.iter().map(|c| self.moved(c, state)).collect();
                list_value(e.kind, &values)
            }
            ExprNode::PtrAdd { base, offset } =>
            {
                let b = self.value(base, state);
                let o = int_of(&self.value(offset, state));
                let moved = shifted(b, o);
                check_offset(self, &moved, e.span, false);
                moved
            }
            ExprNode::PtrSub { lhs, rhs } =>
            {
                check_ptr_sub(self, lhs, rhs, e.span, state);
                self.operands_value(e, state)
            }
            ExprNode::CellAddr { list, .. } =>
            {
                let l = self.value(list, state);
                self.cell_addr(e, &l, state)
            }
            ExprNode::FieldAddr { base, .. } =>
            {
                let b = self.value(base, state);
                cell_value(base, &b, Interval::point(0), 1)
            }
            ExprNode::MakeSlice { source, start, len, from, .. } =>
            {
                let s = self.value(source, state);
                self.value(start, state);
                let count = int_of(&self.value(len, state));
                do_if!(*from == SliceFrom::Block, self.deref(source, state));
                let made = tern!(
                    matches!(from, SliceFrom::List(_)),
                    held_cells(),
                    unbounded(&s)
                );
                counted(made, count)
            }
            ExprNode::Lend { operand } | ExprNode::Convert { operand, .. } =>
            {
                let v = self.value(operand, state);
                converted(e, operand.kind, v, self.platform)
            }
            _ => self.operands_value(e, state),
        }
    }

    /// The value of an expression whose operands run first.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    /// * `state` - The facts, changed in place.
    ///
    /// # Returns
    ///
    /// The value.
    fn operands_value(&mut self, e: &Expr, state: &mut State) -> SlotState
    {
        let subs: Vec<SlotState> =
            sub_exprs(e).into_iter().map(|sub| self.value(sub, state)).collect();
        self.check_overflow(e, &subs);
        self.plain_value(e, &subs)
    }

    /// With `--overflow stop`, report an integer operation whose result lies
    /// outside its kind for each value its operands can hold: the program
    /// stops there each time it runs.
    ///
    /// # Arguments
    ///
    /// * `e` - The operation.
    /// * `subs` - The values of its operands.
    fn check_overflow(&mut self, e: &Expr, subs: &[SlotState])
    {
        ret_if!(self.options.overflow != OverflowMode::Stop);
        let Some(kind) = e.kind.int_kind()
        else
        {
            return;
        };
        let ints = |n: usize| subs.get(n).map_or(Interval::TOP, int_of);
        let (result, op) = match &e.node
        {
            ExprNode::Infix { op: InfixOp::Add, .. } => (ints(0).plus(ints(1)), "+"),
            ExprNode::Infix { op: InfixOp::Sub, .. } => (ints(0).minus(ints(1)), "-"),
            ExprNode::Infix { op: InfixOp::Mul, .. } => (ints(0).times(ints(1)), "*"),
            ExprNode::Prefix { op: PrefixOp::Neg, .. } => (ints(0).negate(), "-"),
            _ => return,
        };
        let range = Interval::of_int_kind(kind, self.platform);
        let outside =
            result.is_bounded() && (result.hi < range.lo || result.lo > range.hi);
        do_if!(outside, report(self, overflow_proven(e.span, op, &e.kind.name())));
    }

    /// The value of a load, once its address runs.
    ///
    /// # Arguments
    ///
    /// * `e` - The load.
    /// * `addr` - Its address.
    /// * `at` - The value of the address.
    /// * `state` - The facts, changed in place.
    ///
    /// # Returns
    ///
    /// The value.
    fn load(
        &mut self, e: &Expr, addr: &Expr, at: &SlotState, state: &State,
    ) -> SlotState
    {
        self.deref(addr, state);
        check_offset(self, at, addr.span, true);
        SlotState::unknown(e.kind, Roots::new())
    }

    /// The value of a cell address, once its list runs.
    ///
    /// # Arguments
    ///
    /// * `e` - The cell address.
    /// * `l` - The value of its list.
    /// * `state` - The facts, changed in place.
    ///
    /// # Returns
    ///
    /// The value.
    fn cell_addr(&mut self, e: &Expr, l: &SlotState, state: &mut State) -> SlotState
    {
        let ExprNode::CellAddr { list, index, len, first, .. } = &e.node
        else
        {
            return self.value(e, state);
        };
        do_if!(!list.kind.is_list(), self.deref(list, state));
        let i = int_of(&self.value(index, state));
        let Some(len) = *len
        else
        {
            self.slice_cell(list, index, i, *first, (l, state), e.span);
            return unbounded(l);
        };
        check_cell(self, i, len, *first, e.span);
        let cell = i.minus(Interval::point(i128::from(*first)));
        cell_value(list, l, cell, len)
    }

    /// Check a cell of a slice. A slice of a known count checks as a list of
    /// that count does, and a cell outside it is reported. Else the cell is
    /// proved in range when its number is a local at 0 or above that a test
    /// left below the count of the slice local, and is checked when the
    /// program runs otherwise.
    ///
    /// # Arguments
    ///
    /// * `list` - The slice.
    /// * `index` - The cell number.
    /// * `i` - The values the cell number can take.
    /// * `first` - The number of the first cell: 0 or 1.
    /// * `facts` - The value of the slice, and the facts.
    /// * `span` - The cell address.
    fn slice_cell(
        &mut self, list: &Expr, index: &Expr, i: Interval, first: u8,
        facts: (&SlotState, &State), span: Span,
    )
    {
        let (l, state) = facts;
        let slice = tern!(let ExprNode::Read(place) = list.node, Some(place), None);
        let n = int_slot(index, self.platform);
        let pair = n.zip(slice).filter(|pair| state.below.contains(pair));
        let kind = n.and_then(|p| self.ctx.kind_of(p));
        let unsigned = kind.is_some_and(DataKind::is_unsigned);
        let below = first == 0 && (i.lo >= 0 || unsigned) && pair.is_some();
        ret_if!(below, do_if!(self.recording(), self.proofs.push(span)));
        let known = ptr_info(l).filter(|p| p.offset == Interval::point(0));
        let count = known.and_then(|p| p.len).and_then(|n| u32::try_from(n).ok());
        ret_if!(let Some(n) = count, check_cell(self, i, n, first, span));
        do_if!(self.recording(), self.doubts.push(span));
    }

    /// Run a load of a list cell or a struct field tested against `ဗလာ`.
    /// The slot is read without the moved-out check, as the test does not
    /// use the value: the cells of a list moved out read as `ဗလာ`, as do the
    /// fields of a struct.
    ///
    /// # Arguments
    ///
    /// * `e` - The load.
    /// * `state` - The facts, changed in place.
    fn tested_cell(&mut self, e: &Expr, state: &mut State)
    {
        let ExprNode::Load { addr } = &e.node
        else
        {
            return;
        };
        let list = match &addr.node
        {
            ExprNode::CellAddr { list, .. }
            | ExprNode::FieldAddr { base: list, .. } => list,
            _ => return,
        };
        let ExprNode::Read(place) = list.node
        else
        {
            return;
        };
        let l = read_value(place, list.kind, state);
        let at = tern!(
            matches!(addr.node, ExprNode::CellAddr { .. }),
            self.cell_addr(addr, &l, state),
            cell_value(list, &l, Interval::point(0), 1)
        );
        self.load(e, addr, &at, state);
    }

    /// The value of a comparison. A test of a slot, or of a cell of a list
    /// slot, against `ဗလာ` reads the slot without using its value, so it
    /// is no use of a moved-out owner.
    ///
    /// # Arguments
    ///
    /// * `e` - The comparison.
    /// * `lhs` - The left operand.
    /// * `rhs` - The right operand.
    /// * `state` - The facts, changed in place.
    ///
    /// # Returns
    ///
    /// The value.
    fn compare(
        &mut self, e: &Expr, lhs: &Expr, rhs: &Expr, state: &mut State,
    ) -> SlotState
    {
        let unknown = SlotState::unknown(e.kind, Roots::new());
        if let Some(place) = null_compared(lhs, rhs)
        {
            check_read(self, place, e.span, state, true);
            return unknown;
        }
        let Some(cell) = null_compared_cell(lhs, rhs)
        else
        {
            return self.operands_value(e, state);
        };
        self.tested_cell(cell, state);
        unknown
    }

    /// The value of `and` or `or`. The right operand runs when the left one
    /// does not decide, so it runs with the facts narrowed to that edge of
    /// the left one; the facts after it join those of the edge that skips
    /// it.
    ///
    /// # Arguments
    ///
    /// * `e` - The operation.
    /// * `op` - The operator.
    /// * `lhs` - The left operand.
    /// * `rhs` - The right operand.
    /// * `state` - The facts, changed in place.
    ///
    /// # Returns
    ///
    /// The value.
    fn logic(
        &mut self, e: &Expr, op: LogicOp, lhs: &Expr, rhs: &Expr, state: &mut State,
    ) -> SlotState
    {
        self.value(lhs, state);
        let skipped = state.clone();
        let ints = (!has_effects(lhs)).then_some(self.platform);
        narrow(lhs, op == LogicOp::And, state, ints);
        self.value(rhs, state);
        *state = skipped.join(state);
        SlotState::unknown(e.kind, Roots::new())
    }

    /// The value of an expression whose operands ran.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    /// * `subs` - The values of its operands.
    ///
    /// # Returns
    ///
    /// The value.
    fn plain_value(&self, e: &Expr, subs: &[SlotState]) -> SlotState
    {
        let ints = |n: usize| subs.get(n).map_or(Interval::TOP, int_of);
        match &e.node
        {
            ExprNode::Const(v) => const_value(e.kind, *v, self.platform),
            ExprNode::AddrOf(_) =>
            {
                let info = PtrInfo::builder()
                    .state(OwnerState::HELD)
                    .offset(Interval::point(0))
                    .len(1)
                    .build();
                SlotState::Ptr(info)
            }
            ExprNode::SizeOf(kind) =>
            {
                let size = Interval::point(i128::from(kind.size(self.platform)));
                int_fact(e.kind, size, self.platform)
            }
            ExprNode::Infix { op, .. } =>
            {
                int_fact(e.kind, arith(*op, ints(0), ints(1)), self.platform)
            }
            ExprNode::Prefix { op: PrefixOp::Neg, .. } =>
            {
                int_fact(e.kind, ints(0).negate(), self.platform)
            }
            ExprNode::BlankList => SlotState::blank(e.kind),
            _ => SlotState::unknown(e.kind, Roots::new()),
        }
    }

    /// Move a value out of the place an address names.
    ///
    /// # Arguments
    ///
    /// * `e` - The [`ExprNode::Take`] expression.
    /// * `addr` - The address.
    /// * `state` - The facts, changed in place.
    ///
    /// # Returns
    ///
    /// The value moved out.
    fn take(&mut self, e: &Expr, addr: &Expr, state: &mut State) -> SlotState
    {
        let value = match addr.node
        {
            ExprNode::AddrOf(place) => held_value(place, e.kind, state),
            _ =>
            {
                let at = self.value(addr, state);
                check_offset(self, &at, addr.span, true);
                SlotState::unknown(e.kind, Roots::new())
            }
        };
        let blank = matches!(addr.node, ExprNode::AddrOf(p) if holds_blank(p, state));
        let paths = run_take(self, e, state);
        let lost = Loss::builder().paths(paths).what(Invalidation::Moved).span(e.span);
        do_if!(!blank && self.recording(), self.losses.push(lost.build()));
        value
    }

    /// Run a call: its arguments, the checks against the digest of the
    /// callee, then what the callee can change. Each global a function can
    /// name, each local whose bare address is an argument, each slot an
    /// argument points at itself, not into its block, and each integer a
    /// global borrow the callee can name points at, is not known after it,
    /// since the callee can write them; the roots of a borrow are kept.
    /// A slot the arguments of this call move out of remains moved out on
    /// each path where it was, so a read after a move into the call is
    /// still a read of a moved-out owner. An owner whose block the callee
    /// can give back gets [`OwnerState::GIVEN`], and one that had it keeps
    /// it.
    ///
    /// # Arguments
    ///
    /// * `func` - The function number.
    /// * `args` - The arguments.
    /// * `span` - The call.
    /// * `state` - The facts, changed in place.
    fn call(&mut self, func: u32, args: &[Expr], span: Span, state: &mut State)
    {
        let roots: Vec<Roots> =
            args.iter().map(|a| value_roots(a, state, &self.ctx)).collect();
        let mut taken = moved_slots(args);
        taken.retain(|p| !self.ctx.is_const(*p));
        do_for!(a in args, { self.moved(a, state); });
        check_call(self, func, &roots, span, state);
        note_unfilled(self, func, &roots, state);
        lift_reads(self, func, &roots);
        let given: BTreeSet<Place> =
            given_back(self, func, &roots).iter().filter_map(own_block).collect();
        let resized = all_given_back(self, func, &roots);
        let globals = self.named.iter().copied();
        let addressed = args.iter().filter_map(bare_address);
        let pointed = roots.iter().flatten().filter(|r| !r.path.contains(&Seg::Block));
        let pointed = pointed.map(|r| r.place);
        let through = self.named.iter().filter_map(|g| state.ptr(*g));
        let through = through.flat_map(|info| info.roots.iter());
        let through =
            through.filter(|r| !r.path.contains(&Seg::Block)).map(|r| r.place);
        let through: Vec<Place> = through.filter(|p| holds_int(*p, state)).collect();
        let moved = |p: &Place| state.ptr(*p).is_some_and(|info| info.state.is_moved());
        let kept = |p: &Place| taken.contains(p) && moved(p);
        let places = globals.chain(addressed).chain(pointed).chain(through);
        let places: BTreeSet<Place> = places.filter(|p| !kept(p)).collect();
        for place in places
        {
            let had = state.ptr(place).is_some_and(|info| info.state.shares_given());
            self.reset(place, state);
            do_if!(had || given.contains(&place), self.mark_given(place, state));
        }
        forget_extents(&resized, state);
    }

    /// Forget the extent of each pointer into a block a free or a resize
    /// gives back.
    ///
    /// # Arguments
    ///
    /// * `dest` - Where the owner of the block lives.
    /// * `state` - The facts before the statement, changed in place.
    fn forget_freed(&self, dest: &Dest, state: &mut State)
    {
        let paths = freed_paths(dest, state, &self.ctx);
        forget_extents(&paths, state);
    }

    /// Mark an owner whose block a call can have given back, or a move
    /// through an address can have taken.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    /// * `state` - The facts, changed in place.
    pub(crate) fn mark_given(&self, place: Place, state: &mut State)
    {
        let (Some(kind), Some(info)) = (self.ctx.kind_of(place), state.ptr(place))
        else
        {
            return;
        };
        let marked =
            PtrInfo { state: info.state.join(OwnerState::GIVEN), ..info.clone() };
        state.set(place, SlotState::of_kind(kind, marked, Interval::TOP));
    }

    /// The digest, the diagnostics and the proofs.
    ///
    /// # Returns
    ///
    /// The result.
    fn finish(self) -> FlowResult
    {
        let freed = |r: &&Root| self.frees.iter().any(|f| f.reaches(r));
        let renewed = self.result_roots.iter().filter(freed).cloned().collect();
        let digest = FnDigest::builder()
            .name(self.routine.name)
            .params(self.routine.params)
            .frees(self.frees.into_iter().collect())
            .result_from(self.result_from.into_iter().collect())
            .result_roots(self.result_roots.into_iter().collect())
            .renewed(renewed)
            .unfilled(self.unfilled.into_iter().collect())
            .reads(self.reads.into_iter().collect())
            .result_roots_ok(self.result_roots_ok)
            .build();
        FlowResult::builder()
            .digest(digest)
            .diags(self.diags)
            .proofs(self.proofs)
            .doubts(self.doubts)
            .build()
    }
}

/// The globals a function can name: those declared before the last
/// function in source order, since a function names a global declared
/// before it alone. A global declared after the last function changes in
/// a call through an address argument alone, and the call resets those
/// on their own.
///
/// # Arguments
///
/// * `module` - The module.
///
/// # Returns
///
/// The globals, in module order.
fn named_globals(module: &Module) -> Vec<Place>
{
    let last = module.funcs.iter().map(|f| f.span.lo).max();
    let named = |n: &usize| last.is_some_and(|lo| module.globals[*n].span.lo < lo);
    let count = module.globals.len();
    (0..count).filter(named).map(|n| Place::Global(n as u32)).collect()
}

/// The owner slot whose own block a path names: the slot itself, or a
/// list of owners through one of its cells, at whatever depth of nested
/// lists.
///
/// # Arguments
///
/// * `root` - The path.
///
/// # Returns
///
/// The slot, or `None` for a path into a deeper block.
fn own_block(root: &Root) -> Option<Place>
{
    (past_cells(&root.path) == [Seg::Block]).then_some(root.place)
}

/// The slot whose bare address an argument is.
///
/// # Arguments
///
/// * `arg` - The argument.
///
/// # Returns
///
/// The slot of an address of a slot, under conversions and lends too.
fn bare_address(arg: &Expr) -> Option<Place>
{
    match &arg.node
    {
        ExprNode::AddrOf(place) => Some(*place),
        ExprNode::Convert { operand, .. } | ExprNode::Lend { operand } =>
        {
            bare_address(operand)
        }
        _ => None,
    }
}

/// The slots the arguments of a call move out of: the slot of each move
/// out of a slot, at whatever depth, and each argument that is a bare
/// read of a move kind.
///
/// # Arguments
///
/// * `args` - The arguments.
///
/// # Returns
///
/// The slots.
fn moved_slots(args: &[Expr]) -> BTreeSet<Place>
{
    let mut found = BTreeSet::new();
    let read = |a: &Expr| tern!(let ExprNode::Read(p) = a.node, Some(p), None);
    let bare = args.iter().filter(|a| a.kind.is_move_kind()).filter_map(read);
    found.extend(bare);
    let mut take = |e: &Expr| {
        let ExprNode::Take { addr } = &e.node
        else
        {
            return;
        };
        do_if!(let ExprNode::AddrOf(p) = addr.node, { found.insert(p); });
    };
    do_for!(a in args, each_expr(a, &mut take));
    found
}

/// The value a slot holds, as a move out of it gives it on: a value moved
/// out before is `ဗလာ`.
///
/// # Arguments
///
/// * `place` - The slot.
/// * `kind` - The kind of the value.
/// * `state` - The facts.
///
/// # Returns
///
/// The value.
fn held_value(place: Place, kind: DataKind, state: &State) -> SlotState
{
    let info =
        state.ptr(place).cloned().unwrap_or_else(|| PtrInfo::unknown(Roots::new()));
    let value = PtrInfo { state: info.state.as_read(), ..info };
    SlotState::of_kind(kind, value, Interval::TOP)
}

/// Leave a slot moved out.
///
/// # Arguments
///
/// * `place` - The slot.
/// * `kind` - Its kind.
/// * `state` - The facts, changed in place.
pub(crate) fn leave_moved(place: Place, kind: DataKind, state: &mut State)
{
    let moved =
        PtrInfo::builder().state(OwnerState::MOVED).offset(Interval::point(0)).build();
    state.set(place, SlotState::of_kind(kind, moved, Interval::TOP));
}

/// Whether a node is the test of a loop.
///
/// # Arguments
///
/// * `kind` - What the node runs.
///
/// # Returns
///
/// `true` for the test of a loop and the step of a counted loop.
pub(crate) fn is_loop_test(kind: &NodeKind) -> bool
{
    matches!(kind, NodeKind::LoopHead(_) | NodeKind::CountedHead(_))
}

/// The value of a constant.
///
/// # Arguments
///
/// * `kind` - The kind of the expression.
/// * `v` - The constant.
/// * `platform` - The bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// `ဗလာ` for the null address, the point of an integer.
fn const_value(kind: DataKind, v: Value, platform: u16) -> SlotState
{
    ret_if!(v == Value::NULL, SlotState::blank(kind));
    let Value::Int { kind: int, raw } = v
    else
    {
        return SlotState::unknown(kind, Roots::new());
    };
    let n = tern!(int.signed, Some(raw as i128), i128::try_from(raw).ok());
    int_fact(kind, n.map_or(Interval::TOP, Interval::point), platform)
}

/// The value a slot read gives: the fact of the slot, a moved-out owner
/// read as `ဗလာ`.
///
/// # Arguments
///
/// * `place` - The slot.
/// * `kind` - Its kind.
/// * `state` - The facts.
///
/// # Returns
///
/// The value.
fn read_value(place: Place, kind: DataKind, state: &State) -> SlotState
{
    let fact = state.get(place).clone();
    let unmoved = |info: PtrInfo| PtrInfo { state: info.state.as_read(), ..info };
    match fact
    {
        SlotState::Ptr(info) => SlotState::Ptr(unmoved(info)),
        SlotState::List(info) => SlotState::List(unmoved(info)),
        SlotState::None => SlotState::unknown(kind, Roots::new()),
        other => other,
    }
}

/// The value of a list written out: one fact for each cell.
///
/// # Arguments
///
/// * `kind` - The list kind.
/// * `cells` - The values of the cells.
///
/// # Returns
///
/// The join of the cell facts.
fn list_value(kind: DataKind, cells: &[SlotState]) -> SlotState
{
    let infos = cells.iter().filter_map(ptr_info);
    let none = PtrInfo::builder().state(OwnerState::NONE).offset(Interval::TOP).build();
    let info = infos.fold(none, |all, cell| all.join(cell));
    SlotState::of_kind(kind, info, Interval::TOP)
}

/// An address moved some elements on.
///
/// # Arguments
///
/// * `base` - The value of the address.
/// * `step` - The element count.
///
/// # Returns
///
/// The address with its offset moved.
fn shifted(base: SlotState, step: Interval) -> SlotState
{
    let SlotState::Ptr(info) = base
    else
    {
        return base;
    };
    SlotState::Ptr(PtrInfo { offset: info.offset.plus(step), ..info })
}

/// The value of a conversion or a lend of a value.
///
/// # Arguments
///
/// * `e` - The conversion.
/// * `from` - The kind of the operand.
/// * `v` - The value of the operand.
/// * `platform` - The bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// The value; a pointer from a raw address is not known, an integer
/// the compiler widened keeps its interval.
fn converted(e: &Expr, from: DataKind, v: SlotState, platform: u16) -> SlotState
{
    let implicit = matches!(e.node, ExprNode::Convert { implicit: true, .. });
    let keeps = matches!(e.node, ExprNode::Lend { .. }) || implicit;
    match v
    {
        SlotState::Ptr(info) if !from.is_raw_ptr() =>
        {
            let info = recounted(info, from, e.kind, platform);
            SlotState::of_kind(e.kind, info, Interval::TOP)
        }
        SlotState::Int(n) if keeps => int_fact(e.kind, n, platform),
        _ => SlotState::unknown(e.kind, Roots::new()),
    }
}

/// The fact of the address of a cell. A cell of a list slot lies in the
/// slot, so its block is the list; a cell reached through a pointer to a
/// list lies in the block of that pointer, which holds whole lists. A cell
/// of another list value has no extent known.
///
/// # Arguments
///
/// * `list` - The list, or a pointer to it.
/// * `l` - The value of `list`.
/// * `cell` - The cell number from 0.
/// * `len` - The number of cells of the list.
///
/// # Returns
///
/// The fact.
fn cell_value(list: &Expr, l: &SlotState, cell: Interval, len: u32) -> SlotState
{
    let n = Interval::point(i128::from(len));
    let slot = matches!(list.node, ExprNode::Read(_));
    let (state, offset, count) = match ptr_info(l)
    {
        _ if list.kind.is_aggregate() =>
        {
            (OwnerState::HELD, tern!(slot, cell, Interval::TOP), slot.then_some(1))
        }
        Some(info) => (info.state, info.offset.times(n).plus(cell), info.len),
        None => (OwnerState::UNKNOWN, Interval::TOP, None),
    };
    let len = count.and_then(|c| c.checked_mul(u64::from(len)));
    SlotState::Ptr(
        PtrInfo::builder().state(state).offset(offset).maybe_len(len).build(),
    )
}

/// The fact of a pointer to cells whose extent the check does not follow:
/// a slice, or a cell of one. It holds what the pointer it comes from holds.
///
/// # Arguments
///
/// * `from` - The value it comes from.
///
/// # Returns
///
/// The fact, with no offset and no extent known.
fn unbounded(from: &SlotState) -> SlotState
{
    let state = ptr_info(from).map_or(OwnerState::UNKNOWN, |info| info.state);
    SlotState::Ptr(PtrInfo::builder().state(state).offset(Interval::TOP).build())
}

/// The fact of a slice with the count it was made with, when that count is
/// one known value: its first cell is at offset 0, and its cells are that
/// count.
///
/// # Arguments
///
/// * `made` - The fact of the slice.
/// * `count` - The values its count can take.
///
/// # Returns
///
/// The fact.
fn counted(made: SlotState, count: Interval) -> SlotState
{
    let SlotState::Ptr(info) = made
    else
    {
        return made;
    };
    let exact = (count.lo == count.hi).then_some(count.lo);
    let Some(n) = exact.and_then(|n| u64::try_from(n).ok())
    else
    {
        return SlotState::Ptr(info);
    };
    SlotState::Ptr(PtrInfo { offset: Interval::point(0), len: Some(n), ..info })
}

/// The fact of a slice of the cells of a list, which lives as long as the
/// list.
///
/// # Returns
///
/// The fact, with no offset and no extent known.
fn held_cells() -> SlotState
{
    SlotState::Ptr(
        PtrInfo::builder().state(OwnerState::HELD).offset(Interval::TOP).build(),
    )
}

/// The fact of a pointer seen as a pointer to another kind: its offset and
/// its element count in elements of the new pointee, when the octets these
/// stand for hold whole elements of it.
///
/// # Arguments
///
/// * `info` - The fact.
/// * `from` - The kind of the pointer.
/// * `to` - The kind it becomes.
/// * `platform` - The bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// The fact, with no extent known when the octets do not divide.
fn recounted(info: PtrInfo, from: DataKind, to: DataKind, platform: u16) -> PtrInfo
{
    let size = |k: DataKind| k.pointee().map(|p| i128::from(p.size(platform)));
    let (Some(old), Some(new)) = (size(from), size(to))
    else
    {
        return PtrInfo { offset: Interval::TOP, len: None, ..info };
    };
    ret_if!(old == new, info);
    let whole = |octets: i128| (new > 0 && octets % new == 0).then(|| octets / new);
    let at = info.offset;
    let offset = (at.lo == at.hi).then(|| whole(at.lo.checked_mul(old)?)).flatten();
    let len = info.len.and_then(|n| whole(i128::from(n).checked_mul(old)?));
    let len = len.and_then(|n| u64::try_from(n).ok());
    PtrInfo { offset: offset.map_or(Interval::TOP, Interval::point), len, ..info }
}

/// Forget the element count and the offset of each pointer that can point
/// into a block one of the paths names, and of each owner a path starts
/// at: the block is gone or has another size.
///
/// # Arguments
///
/// * `paths` - The paths given back.
/// * `state` - The facts, changed in place.
fn forget_extents(paths: &Roots, state: &mut State)
{
    ret_if!(paths.is_empty());
    let globals = (0..state.globals.len()).map(|n| Place::Global(n as u32));
    let locals = (0..state.locals.len()).map(|n| Place::Local(n as u32));
    let places: Vec<Place> = globals.chain(locals).collect();
    for place in places
    {
        let Some(info) = state.ptr_mut(place)
        else
        {
            continue;
        };
        let into =
            |p: &Root| p.place == place || info.roots.iter().any(|r| p.reaches(r));
        let hit = paths.iter().any(into);
        do_if!(
            hit,
            *info = PtrInfo { offset: Interval::TOP, len: None, ..info.clone() }
        );
    }
}

/// Whether a slot holds an integer.
///
/// # Arguments
///
/// * `place` - The slot.
/// * `state` - The facts.
///
/// # Returns
///
/// `true` for an integer slot.
fn holds_int(place: Place, state: &State) -> bool
{
    matches!(state.get(place), SlotState::Int(_))
}

/// Whether an expression calls a function or moves a value out: the call
/// can change a slot the expression read before it.
///
/// # Arguments
///
/// * `e` - The expression.
///
/// # Returns
///
/// `true` when a call or a move sits in it.
fn has_effects(e: &Expr) -> bool
{
    let mut found = false;
    let mut look = |sub: &Expr| {
        found |= matches!(sub.node, ExprNode::Call { .. } | ExprNode::Take { .. });
    };
    each_expr(e, &mut look);
    found
}

/// The pointer fact of a value.
///
/// # Arguments
///
/// * `v` - The value.
///
/// # Returns
///
/// The fact of a pointer or a list of pointers.
fn ptr_info(v: &SlotState) -> Option<&PtrInfo>
{
    match v
    {
        SlotState::Ptr(info) | SlotState::List(info) => Some(info),
        _ => None,
    }
}

/// The fact of an integer of `kind`.
///
/// # Arguments
///
/// * `kind` - The kind.
/// * `n` - The interval.
/// * `platform` - The bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// The fact; nothing for a kind that is no integer, [`Interval::TOP`]
/// when `n` leaves the range of the kind, which wraps at run time.
fn int_fact(kind: DataKind, n: Interval, platform: u16) -> SlotState
{
    SlotState::of_kind(kind, PtrInfo::unknown(Roots::new()), n.fit(kind, platform))
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::roots::global_roots;
    use crate::testing::program;

    /// The facts before the first node `pick` takes, once the flow of
    /// `routine` is solved.
    fn state_at(
        module: &Module, routine: Routine, pick: impl Fn(&NodeKind) -> bool,
    ) -> State
    {
        let roots = global_roots(module, &[]);
        let options = BorrowCheckOptions::default();
        let mut flow = Flow::new(module, routine, &[], &roots, &options, 64);
        flow.solve();
        let n = flow
            .cfg
            .nodes
            .iter()
            .position(|n| pick(&n.kind))
            .expect("the node is in the graph");
        flow.ins[n].clone().expect("the node is reached")
    }

    /// The owner state of a slot.
    fn owner(state: &State, place: Place) -> OwnerState
    {
        state.ptr(place).expect("the slot is a pointer").state
    }

    #[test]
    fn a_move_leaves_the_source_moved_and_gives_the_block_on()
    {
        let prog = program(
            "ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၃ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် ဗ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ဖြစ်ပါစေ။
",
        );
        let end = state_at(&prog.module, Routine::main(&prog.module), |k| {
            matches!(k, NodeKind::End)
        });
        assert_eq!(owner(&end, Place::Global(0)), OwnerState::MOVED);
        let to = end.ptr(Place::Global(1)).unwrap();
        assert_eq!((to.state, to.len), (OwnerState::FRESH, Some(3)));
    }

    #[test]
    fn a_null_test_narrows_each_branch()
    {
        let prog = program(
            "ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
အကယ်၍ ပ သည် ဗလာ နှင့် ညီလျှင် \"ဗလာ\" ဟု ပြသပါ။
",
        );
        let routine = Routine::main(&prog.module);
        let inside = state_at(&prog.module, routine, |k| {
            matches!(k, NodeKind::Leaf(Stmt::Print { .. }))
        });
        assert_eq!(owner(&inside, Place::Global(0)), OwnerState::NULL);
        let end = state_at(&prog.module, routine, |k| matches!(k, NodeKind::End));
        assert_eq!(owner(&end, Place::Global(0)), OwnerState::UNKNOWN);
    }

    #[test]
    fn a_call_resets_the_named_globals_but_keeps_a_move_into_it()
    {
        let prog = program(
            "ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။

လုပ်ဆောင်ချက် ရှင်းပစ် သည်
ကိန်းပြည့် လိပ်စာ က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လိပ်စာ က ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် ဃ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ဂ တွင် သိမ်းထားမည်။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ဃ တွင် သိမ်းထားမည်။
လုပ်ဆောင်ချက် ရှင်းပစ် ကို ပ တို့ဖြင့် လုပ်ဆောင်ပါ။
",
        );
        let module = &prog.module;
        let end =
            state_at(module, Routine::main(module), |k| matches!(k, NodeKind::End));
        assert_eq!(owner(&end, Place::Global(0)), OwnerState::MOVED);
        assert_eq!(owner(&end, Place::Global(1)), OwnerState::UNKNOWN);
        assert_eq!(owner(&end, Place::Global(2)), OwnerState::FRESH);
        let inner = Routine::func(&module.funcs[0]);
        let end = state_at(module, inner, |k| matches!(k, NodeKind::End));
        assert_eq!(owner(&end, Place::Local(0)), OwnerState::NULL);
    }

    #[test]
    fn a_borrow_param_points_into_its_argument()
    {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../samples/heap/swap.zaka");
        let prog = program(&std::fs::read_to_string(path).unwrap());
        let module = &prog.module;
        let inner_end = state_at(module, Routine::func(&module.funcs[0]), |k| {
            matches!(k, NodeKind::End)
        });
        let first = inner_end.ptr(Place::Local(0)).unwrap();
        assert_eq!(first.roots, Roots::from([Root::slot(Place::Local(0)).block()]));
        assert_eq!(first.state, OwnerState::UNKNOWN);
        assert_eq!(*inner_end.get(Place::Local(2)), SlotState::Int(Interval::TOP));
        let end =
            state_at(module, Routine::main(module), |k| matches!(k, NodeKind::End));
        assert_eq!(*end.get(Place::Global(0)), SlotState::Int(Interval::TOP));
    }

    #[test]
    fn a_cell_of_a_borrow_list_param_points_through_its_cells()
    {
        let prog = program(
            "လုပ်ဆောင်ချက် ဖတ် သည်
ငှားထားသော အကွက် ၂ ခုရှိသော ကိန်းပြည့် လိပ်စာ စာရင်း တန်း ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဂ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် တန်း ၏ အကွက် ၁ ဖြစ်ပါစေ။
ဂ ၏ တန်ဖိုး ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
",
        );
        let module = &prog.module;
        let end = state_at(module, Routine::func(&module.funcs[0]), |k| {
            matches!(k, NodeKind::End)
        });
        let cells = Root::slot(Place::Local(0)).cell(None).block();
        assert_eq!(end.ptr(Place::Local(1)).unwrap().roots, Roots::from([cells]));
    }

    #[test]
    fn an_interval_that_wraps_is_not_known()
    {
        let prog = program(
            "ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၂၇ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ၈ ဘစ် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် က + ၁ ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် ဖြစ်ပါစေ။
ကိန်းရှင် ဃ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ဂ - ၁ ဖြစ်ပါစေ။
ကိန်းရှင် င သည် ၈ ဘစ် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် က - ၁ ဖြစ်ပါစေ။
၀ မှ က + ၁ အထိ န အနေဖြင့် ရေတွက်၍ န ကို ပြသပါ။
",
        );
        let module = &prog.module;
        let routine = Routine::main(module);
        let inside = state_at(module, routine, |k| {
            matches!(k, NodeKind::Leaf(Stmt::Print { .. }))
        });
        let top = SlotState::Int(Interval::TOP);
        assert_eq!(*inside.get(Place::Global(1)), top);
        assert_eq!(*inside.get(Place::Global(3)), top);
        assert_eq!(*inside.get(Place::Global(4)), SlotState::Int(Interval::point(126)));
        assert_eq!(*inside.get(Place::Global(5)), top);
    }

    #[test]
    fn solve_terminates_on_walking_borrow()
    {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../samples/heap/walk.zaka");
        let prog = program(&std::fs::read_to_string(path).unwrap());
        let module = &prog.module;
        let roots = global_roots(module, &[]);
        let options = BorrowCheckOptions::default();
        let mut flow =
            Flow::new(module, Routine::main(module), &[], &roots, &options, 64);
        flow.solve();
        let node = |pick: &dyn Fn(&NodeKind) -> bool| {
            let n = flow.cfg.nodes.iter().position(|n| pick(&n.kind)).unwrap();
            flow.ins[n].clone().expect("the node is reached")
        };
        let head = node(&|k| matches!(k, NodeKind::LoopHead(_)));
        let walker = head.ptr(Place::Global(3)).expect("the borrow is a pointer");
        assert_eq!(walker.offset, Interval::TOP);
        assert_eq!(walker.roots, Roots::from([Root::slot(Place::Global(0)).block()]));
        let store = node(&|k| matches!(k, NodeKind::Leaf(Stmt::Store { .. })));
        assert_eq!(
            *store.get(Place::Global(1)),
            SlotState::Int(Interval { lo: 0, hi: 4 })
        );
        let owner = store.ptr(Place::Global(0)).unwrap();
        assert_eq!((owner.state, owner.len), (OwnerState::FRESH, Some(5)));
        let end = node(&|k| matches!(k, NodeKind::End));
        assert_eq!(end.ptr(Place::Global(0)).unwrap().state, OwnerState::NULL);
        assert!(flow.ins.iter().all(Option::is_some));
    }
}
