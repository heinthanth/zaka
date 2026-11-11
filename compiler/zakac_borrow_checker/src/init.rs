//! The check that each value is set before it is read. A variable the
//! program declares with no value holds none until a write on each path to a
//! read gives it one; no kind gives it a hidden default. A struct is followed
//! one field at a time and a counted list one cell at a time. A global is
//! checked at each call of the start block and of the top level: the globals
//! the function reads, itself or through its calls, must be set there.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::ownership as od;
use zakac_ir_builder::ir::Dest;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::Func;
use zakac_ir_builder::ir::Module;
use zakac_ir_builder::ir::Place;
use zakac_ir_builder::ir::Slot;
use zakac_ir_builder::ir::Stmt;
use zakac_ir_builder::ir::each_stmt;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::const_op::InfixOp;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::kind::FALLIBLE;
use zakac_semcheck::value::Value;
use zakac_span::span::Span;

use crate::callgraph::start_block;
use crate::cfg::Cfg;
use crate::cfg::NodeKind;
use crate::cfg::build_cfg;
use crate::liveness::global_reads_of;
use crate::values::taken_fns;
use crate::walk::each_expr;
use crate::walk::stmt_exprs;
use crate::walk::sub_exprs;

/// The most cells of a counted list the check follows one at a time; a
/// larger list counts as one part.
const CELL_CAP: u32 = 256;

/// What the check knows of one followed slot at a point.
#[derive(Debug, Clone, PartialEq, Default)]
struct Unset
{
    /// The parts that can be unset: each field of a struct, each cell of a
    /// counted list, or part 0 for each other kind.
    parts: BTreeSet<u32>,
    /// Whether a cell was written at a position the check does not know, so
    /// a part left in `parts` can be set after all.
    doubt: bool,
}

/// The facts of each followed slot at a point.
type Facts = BTreeMap<Place, Unset>;

/// The part of a slot a read or a write reaches.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Part
{
    /// The whole slot.
    Whole,
    /// A field of a struct, as the range of leaf fields it holds: its first
    /// leaf and its count. A struct inside a struct is followed one leaf field
    /// at a time.
    Field(u32, u32),
    /// One cell of a counted list, its number from 0; `None` when the
    /// position is not known.
    Cell(Option<u64>),
}

/// Check the module.
///
/// # Arguments
///
/// * `module` - The module.
///
/// # Returns
///
/// The diagnostics.
pub(crate) fn check_init(module: &Module) -> Vec<Diagnostic>
{
    let reads = global_reads_of(module, |_| true);
    let taken = taken_fns(module);
    let through_values: BTreeSet<u32> = taken
        .iter()
        .flat_map(|&f| reads.get(f as usize).into_iter().flatten())
        .copied()
        .collect();
    let start = start_block(module);
    let ctx = Ctx { module, reads: &reads, through_values: &through_values, start };
    let mut diags = Vec::new();
    let others =
        module.funcs.iter().enumerate().filter(|(i, _)| Some(*i as u32) != start);
    for (_, f) in others
    {
        let (tracked, fills) = routine_slots(f);
        let mut run = Run::new(&ctx, &f.locals, tracked, false, None);
        run.fills = fills;
        diags.extend(run.check(&f.block, f.span));
    }
    let globals = followed(&module.globals, Place::Global, 0);
    let main = Run::new(&ctx, &[], globals.clone(), true, None);
    let (main_diags, at_start) = main.check_main(&module.main, module.span);
    diags.extend(main_diags);
    let Some(f) = start.and_then(|s| module.funcs.get(s as usize))
    else
    {
        return diags;
    };
    let (mut tracked, fills) = routine_slots(f);
    tracked.extend(globals);
    let mut run = Run::new(&ctx, &f.locals, tracked, true, at_start);
    run.fills = fills;
    diags.extend(run.check(&f.block, f.span));
    diags
}

/// What each run of the check shares.
struct Ctx<'m>
{
    /// The module.
    module: &'m Module,
    /// The globals each function reads, itself or through its calls.
    reads: &'m [BTreeSet<u32>],
    /// The globals a call through a function value can read: those of each
    /// function the program uses as a value.
    through_values: &'m BTreeSet<u32>,
    /// The start block.
    start: Option<u32>,
}

/// The slots of a list the check follows: those declared with no value, each
/// with its part count and its declaring statement, and each parameter marked
/// `ဒေတာ ထည့်ပေးရန်`, whose parts are those of the place it points at.
///
/// # Arguments
///
/// * `slots` - The slots.
/// * `place` - How a slot number becomes a place.
/// * `params` - The number of parameters among the slots.
///
/// # Returns
///
/// The followed slots.
fn followed(
    slots: &[Slot], place: fn(u32) -> Place, params: u32,
) -> BTreeMap<Place, (u32, Span)>
{
    let count = |i: usize, s: &Slot| {
        let target = s.kind.pointee().filter(|_| (i as u32) < params);
        part_count(target.unwrap_or(s.kind))
    };
    let each = slots.iter().enumerate();
    let kept =
        each.filter_map(|(i, s)| Some((place(i as u32), (count(i, s), s.unset?))));
    kept.collect()
}

/// The followed slots of a function and its parameters marked
/// `ဒေတာ ထည့်ပေးရန်`. The function of a C function trusts C to fill them, so
/// it follows none of them.
///
/// # Arguments
///
/// * `f` - The function.
///
/// # Returns
///
/// The followed slots, and the places of the filled parameters.
fn routine_slots(f: &Func) -> (BTreeMap<Place, (u32, Span)>, BTreeSet<Place>)
{
    let mut tracked = followed(&f.locals, Place::Local, f.params);
    let fills: BTreeSet<Place> =
        (0..f.params).map(Place::Local).filter(|p| tracked.contains_key(p)).collect();
    ret_if!(!calls_c(&f.block), (tracked, fills));
    tracked.retain(|p, _| !fills.contains(p));
    (tracked, BTreeSet::new())
}

/// Whether statements call C: the one statement of the function of a C
/// function.
///
/// # Arguments
///
/// * `block` - The statements.
///
/// # Returns
///
/// `true` when one does.
fn calls_c(block: &[Stmt]) -> bool
{
    let mut found = false;
    each_stmt(block, &mut |s| {
        found |= matches!(s, Stmt::Foreign { .. });
        let mut note = |e: &Expr| found |= matches!(e.node, ExprNode::Foreign { .. });
        do_for!(e in stmt_exprs(s), each_expr(e, &mut note));
    });
    found
}

/// The parts the check follows in a value of a kind.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// The count: the leaf fields of a plain struct, the cells of a counted list
/// up to [`CELL_CAP`], else 1.
fn part_count(kind: DataKind) -> u32
{
    ret_if!(plain_struct(kind).is_some(), leaves(kind).max(1));
    let cells = kind.as_list().map(|l| l.len).filter(|&n| n > 0 && n <= CELL_CAP);
    cells.unwrap_or(1)
}

/// The struct of a plain struct kind: no result, no optional and no object.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// The struct, or `None` for each other kind.
fn plain_struct(kind: DataKind)
-> Option<&'static zakac_semcheck::structs::StructKind>
{
    kind.as_struct().filter(|_| kind.fallible_parts().is_none())
}

/// The leaf fields of a kind: each field of a plain struct, with a plain
/// struct inside counted by its own leaves; 1 for each other kind.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// The count.
fn leaves(kind: DataKind) -> u32
{
    let Some(s) = plain_struct(kind)
    else
    {
        return 1;
    };
    s.fields().iter().map(|(_, k)| leaves(*k)).sum()
}

/// The leaf range of one field of a plain struct kind.
///
/// # Arguments
///
/// * `kind` - The struct kind.
/// * `field` - The position of the field.
///
/// # Returns
///
/// The first leaf and the count, or `None` for a kind that is no plain struct.
fn field_leaves(kind: DataKind, field: u32) -> Option<(u32, u32)>
{
    let fields = plain_struct(kind)?.fields();
    let before = fields.iter().take(field as usize).map(|(_, k)| leaves(*k)).sum();
    Some((before, leaves(fields.get(field as usize)?.1)))
}

/// One run of the check over a routine.
struct Run<'c, 'm>
{
    /// What the runs share.
    ctx: &'c Ctx<'m>,
    /// The locals of the routine.
    locals: &'m [Slot],
    /// The followed slots, each with its part count and declaring statement.
    tracked: BTreeMap<Place, (u32, Span)>,
    /// Whether each call is checked for the globals its callee reads.
    calls: bool,
    /// The facts of the globals when the routine starts, for the start block.
    seeded: Option<Facts>,
    /// Whether the transfer reports.
    reporting: bool,
    /// The diagnostics so far.
    diags: Vec<Diagnostic>,
    /// The facts at the call of the start block, kept by the run of the top
    /// level.
    at_start: Option<Facts>,
    /// Each read of an earlier cell inside a counted loop that fills its list,
    /// such as `a[i-1]` in a loop that writes `a[i]`, with its slot and the
    /// cells that must be set before the loop.
    earlier: Vec<(*const Expr, Place, std::ops::Range<u32>)>,
    /// The parameters marked `ဒေတာ ထည့်ပေးရန်`: the place each points at
    /// starts unset, and each return must leave it set.
    fills: BTreeSet<Place>,
}

impl<'c, 'm> Run<'c, 'm>
{
    /// A run with nothing found.
    ///
    /// # Arguments
    ///
    /// * `ctx` - What the runs share.
    /// * `locals` - The locals of the routine.
    /// * `tracked` - The followed slots.
    /// * `calls` - Whether each call is checked for the globals its callee
    ///   reads.
    /// * `seeded` - The facts of the globals when the routine starts.
    ///
    /// # Returns
    ///
    /// The run.
    fn new(
        ctx: &'c Ctx<'m>, locals: &'m [Slot], tracked: BTreeMap<Place, (u32, Span)>,
        calls: bool, seeded: Option<Facts>,
    ) -> Self
    {
        #[rustfmt::skip]
        return Self { ctx, locals, tracked, calls, seeded, reporting: false, diags: Vec::new(), at_start: None, earlier: Vec::new(), fills: BTreeSet::new() };
    }

    /// Check a routine.
    ///
    /// # Arguments
    ///
    /// * `block` - Its statements.
    /// * `span` - Its end.
    ///
    /// # Returns
    ///
    /// The diagnostics.
    fn check(mut self, block: &'m [Stmt], span: Span) -> Vec<Diagnostic>
    {
        self.earlier = earlier_reads(block);
        let cfg = build_cfg(block, span);
        self.report(&cfg);
        self.diags
    }

    /// Check the top level, and keep the facts at the call of the start block.
    ///
    /// # Arguments
    ///
    /// * `block` - Its statements.
    /// * `span` - Its end.
    ///
    /// # Returns
    ///
    /// The diagnostics, and the facts at the call of the start block.
    fn check_main(
        mut self, block: &'m [Stmt], span: Span,
    ) -> (Vec<Diagnostic>, Option<Facts>)
    {
        self.earlier = earlier_reads(block);
        let cfg = build_cfg(block, span);
        self.report(&cfg);
        (self.diags, self.at_start)
    }

    /// Solve the facts of a routine, then run each node a path reaches once
    /// more to report.
    ///
    /// # Arguments
    ///
    /// * `cfg` - The graph of the routine.
    fn report(&mut self, cfg: &Cfg<'m>)
    {
        let ins = self.solve(cfg);
        self.reporting = true;
        for &n in &cfg.rpo
        {
            let Some(facts) = &ins[n as usize]
            else
            {
                continue;
            };
            let node = &cfg.nodes[n as usize];
            do_if!(node.kind == NodeKind::End, self.check_fills(node.span, facts));
            let _ = self.transfer(&node.kind, facts.clone(), 0);
        }
    }

    /// The facts before each node: passes in reverse postorder until no fact
    /// grows.
    ///
    /// # Arguments
    ///
    /// * `cfg` - The graph of the routine.
    ///
    /// # Returns
    ///
    /// The facts before each node a path reaches.
    fn solve(&mut self, cfg: &Cfg<'m>) -> Vec<Option<Facts>>
    {
        let mut ins: Vec<Option<Facts>> = vec![None; cfg.nodes.len()];
        ins[cfg.start as usize] = Some(self.start_facts());
        let mut changed = true;
        while changed
        {
            changed = false;
            for &n in &cfg.rpo
            {
                let Some(facts) = ins[n as usize].clone()
                else
                {
                    continue;
                };
                let node = &cfg.nodes[n as usize];
                for (edge, &succ) in node.succ.iter().enumerate()
                {
                    let out = self.transfer(&node.kind, facts.clone(), edge);
                    changed |= merge(&mut ins[succ as usize], out);
                }
            }
        }
        ins
    }

    /// The facts when the routine starts: each followed slot unset in full,
    /// and the globals of the start block as the top level left them.
    ///
    /// # Returns
    ///
    /// The facts.
    fn start_facts(&self) -> Facts
    {
        let all = |n: u32| Unset { parts: (0..n).collect(), doubt: false };
        let mut facts: Facts =
            self.tracked.iter().map(|(p, (n, _))| (*p, all(*n))).collect();
        do_if!(let Some(globals) = &self.seeded, facts.extend(globals.clone()));
        facts
    }

    /// The facts after a node, on one of its edges.
    ///
    /// # Arguments
    ///
    /// * `kind` - What the node runs.
    /// * `facts` - The facts before it.
    /// * `edge` - The number of the edge out.
    ///
    /// # Returns
    ///
    /// The facts after it.
    fn transfer(&mut self, kind: &NodeKind<'m>, mut facts: Facts, edge: usize)
    -> Facts
    {
        match kind
        {
            NodeKind::Leaf(s) => self.stmt(s, &mut facts),
            NodeKind::Branch(e) | NodeKind::LoopHead(e) => self.read_expr(e, &facts),
            NodeKind::CountedInit(s) =>
            {
                do_for!(e in stmt_exprs(s), self.read_expr(e, &facts))
            }
            NodeKind::CountedHead(s) if edge == 1 => fill(s, &self.tracked, &mut facts),
            _ => (),
        }
        facts
    }

    /// Run one statement.
    ///
    /// # Arguments
    ///
    /// * `s` - The statement.
    /// * `facts` - The facts, changed in place.
    fn stmt(&mut self, s: &'m Stmt, facts: &mut Facts)
    {
        match s
        {
            Stmt::Set { place, value, span } =>
            {
                self.read_expr(value, facts);
                self.fill_args(value, facts);
                self.start_call(value, facts);
                self.set(*place, *span, facts);
            }
            Stmt::Store { addr, value, .. } =>
            {
                self.read_expr(value, facts);
                self.fill_args(value, facts);
                let Some((place, part, index)) = part_of(addr, &self.fills)
                else
                {
                    return self.read_expr(addr, facts);
                };
                do_if!(let Some(i) = index, self.read_expr(i, facts));
                write(facts, place, part);
            }
            Stmt::Return { value, span } =>
            {
                do_if!(let Some(v) = value, self.read_expr(v, facts));
                do_if!(let Some(v) = value, self.fill_args(v, facts));
                self.check_fills(*span, facts);
            }
            Stmt::Alloc { dest, count, .. } =>
            {
                self.read_expr(count, facts);
                self.alloc_into(dest, facts);
            }
            // A drop the compiler adds reads no value the program wrote: an
            // unset place holds zero octets, which drop as nothing.
            Stmt::Drop { .. } => (),
            Stmt::Call { func, args, span, .. } =>
            {
                self.call_args(*func, args, facts);
                self.write_fills(*func, args, facts);
                do_if!(Some(*func) == self.ctx.start, self.keep_start(facts));
                do_if!(
                    Some(*func) != self.ctx.start,
                    self.call(Some(*func), *span, facts)
                );
            }
            _ =>
            {
                do_for!(e in stmt_exprs(s), self.read_expr(e, facts));
                do_for!(e in stmt_exprs(s), self.fill_args(e, facts));
                do_if!(let Some((place, span)) = dest_slot(s), self.read(place, Part::Whole, span, facts));
                do_if!(let Stmt::CallValue { span, .. } = s, self.call(None, *span, facts));
            }
        }
    }

    /// An allocation into a place: a slot, or a field or a cell of one, which
    /// the allocation sets as a write does.
    ///
    /// # Arguments
    ///
    /// * `dest` - The place.
    /// * `facts` - The facts, changed in place.
    fn alloc_into(&mut self, dest: &'m Dest, facts: &mut Facts)
    {
        let addr = match dest
        {
            Dest::Slot(place) => return write(facts, *place, Part::Whole),
            Dest::Mem(addr) => addr,
        };
        let Some((place, part, index)) = part_of(addr, &self.fills)
        else
        {
            return self.read_expr(addr, facts);
        };
        do_if!(let Some(i) = index, self.read_expr(i, facts));
        write(facts, place, part);
    }

    /// Whether a parameter of a function is marked `ဒေတာ ထည့်ပေးရန်`.
    ///
    /// # Arguments
    ///
    /// * `func` - The function.
    /// * `at` - The position of the parameter.
    ///
    /// # Returns
    ///
    /// `true` for such a parameter.
    fn fills_param(&self, func: u32, at: usize) -> bool
    {
        let f = self.ctx.module.funcs.get(func as usize);
        let param = f.filter(|f| (at as u32) < f.params).and_then(|f| f.locals.get(at));
        param.is_some_and(|s| s.unset.is_some())
    }

    /// Read the arguments of a call. An argument for a parameter marked
    /// `ဒေတာ ထည့်ပေးရန်` is the place the callee fills: it reads nothing but
    /// the cell number of its address.
    ///
    /// # Arguments
    ///
    /// * `func` - The function.
    /// * `args` - The arguments.
    /// * `facts` - The facts.
    fn call_args(&mut self, func: u32, args: &'m [Expr], facts: &Facts)
    {
        for (at, a) in args.iter().enumerate()
        {
            let filled =
                arg_place(a, &self.fills).filter(|_| self.fills_param(func, at));
            match filled
            {
                Some((_, _, index)) =>
                {
                    do_if!(let Some(i) = index, self.read_expr(i, facts))
                }
                None => self.read_expr(a, facts),
            }
        }
    }

    /// Set the places a call fills through its parameters marked
    /// `ဒေတာ ထည့်ပေးရန်`.
    ///
    /// # Arguments
    ///
    /// * `func` - The function.
    /// * `args` - The arguments.
    /// * `facts` - The facts, changed in place.
    fn write_fills(&self, func: u32, args: &[Expr], facts: &mut Facts)
    {
        let filled =
            args.iter().enumerate().filter(|(at, _)| self.fills_param(func, *at));
        let places = filled.filter_map(|(_, a)| arg_place(a, &self.fills));
        places.for_each(|(place, part, _)| write(facts, place, part));
    }

    /// Set the places each call inside an expression fills.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    /// * `facts` - The facts, changed in place.
    fn fill_args(&self, e: &Expr, facts: &mut Facts)
    {
        let mut calls = Vec::new();
        each_expr(e, &mut |sub| {
            do_if!(let ExprNode::Call { func, args, .. } = &sub.node, calls.push((*func, args)));
        });
        do_for!((func, args) in calls, self.write_fills(func, args, facts));
    }

    /// Report each parameter marked `ဒေတာ ထည့်ပေးရန်` that a return can leave
    /// unset.
    ///
    /// # Arguments
    ///
    /// * `span` - The return, or the end of the routine.
    /// * `facts` - The facts.
    fn check_fills(&mut self, span: Span, facts: &Facts)
    {
        ret_if!(!self.reporting);
        let open = |p: &&Place| facts.get(*p).is_some_and(|u| !u.parts.is_empty());
        let left: Vec<Place> = self.fills.iter().filter(open).copied().collect();
        for place in left
        {
            let name = self.name(place);
            self.diags.push(od::fill_unset_at_return(span, &name));
        }
    }

    /// A set of a whole slot. The set of the statement that declares the slot
    /// with no value leaves it unset.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    /// * `span` - The statement.
    /// * `facts` - The facts, changed in place.
    fn set(&self, place: Place, span: Span, facts: &mut Facts)
    {
        let Some((count, decl)) = self.tracked.get(&place)
        else
        {
            return;
        };
        let parts = tern!(*decl == span, (0..*count).collect(), BTreeSet::new());
        facts.insert(place, Unset { parts, doubt: false });
    }

    /// Keep the facts at the call of the start block from a set of its exit
    /// global.
    ///
    /// # Arguments
    ///
    /// * `value` - The value set.
    /// * `facts` - The facts before the set.
    fn start_call(&mut self, value: &Expr, facts: &Facts)
    {
        let ExprNode::Call { func, .. } = &value.node
        else
        {
            return;
        };
        do_if!(Some(*func) == self.ctx.start, self.keep_start(facts));
    }

    /// Keep the facts at the call of the start block, in the report pass.
    ///
    /// # Arguments
    ///
    /// * `facts` - The facts.
    fn keep_start(&mut self, facts: &Facts)
    {
        do_if!(self.reporting, self.at_start = Some(facts.clone()));
    }

    /// Report the reads of an expression.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    /// * `facts` - The facts.
    fn read_expr(&mut self, e: &'m Expr, facts: &Facts)
    {
        match &e.node
        {
            ExprNode::Read(place) | ExprNode::AddrOf(place) =>
            {
                return self.read(*place, Part::Whole, e.span, facts);
            }
            ExprNode::Load { addr } =>
            {
                if let Some(set) = self.earlier_set(e, facts)
                {
                    return self.earlier_read(addr, set, e.span, facts);
                }
                if let Some((place, part, index)) = part_of(addr, &self.fills)
                {
                    do_if!(let Some(i) = index, self.read_expr(i, facts));
                    return self.read(place, part, e.span, facts);
                }
            }
            ExprNode::Call { func, args, .. } =>
            {
                do_if!(
                    Some(*func) != self.ctx.start,
                    self.call(Some(*func), e.span, facts)
                );
                return self.call_args(*func, args, facts);
            }
            ExprNode::CallValue { .. } => self.call(None, e.span, facts),
            _ => (),
        }
        do_for!(sub in sub_exprs(e), self.read_expr(sub, facts));
    }

    /// Report a read of a part of a slot that can be unset.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    /// * `part` - The part read.
    /// * `span` - The read.
    /// * `facts` - The facts.
    fn read(&mut self, place: Place, part: Part, span: Span, facts: &Facts)
    {
        let Some(unset) = facts.get(&place).filter(|u| !u.parts.is_empty())
        else
        {
            return;
        };
        let count = self.tracked.get(&place).map_or(1, |(n, _)| *n);
        let hit = match self.plain_part(place, part)
        {
            Part::Field(at, len) =>
            {
                (at..at + len).any(|f| unset.parts.contains(&f)).then_some(false)
            }
            Part::Cell(Some(c)) if count > 1 =>
            {
                let c = u32::try_from(c).unwrap_or(u32::MAX);
                unset.parts.contains(&c).then_some(unset.doubt)
            }
            // A cell at a position the check does not know is unset for sure
            // when no cell is set.
            Part::Cell(_) => Some(unset.doubt || unset.parts.len() < count as usize),
            Part::Whole => Some(unset.doubt),
        };
        let Some(doubt) = hit.filter(|_| self.reporting)
        else
        {
            return;
        };
        let name = self.name(place);
        let diag = tern!(
            doubt,
            od::read_unset_doubt(span, &name),
            od::read_unset(span, &name)
        );
        self.diags.push(diag);
    }

    /// Whether a read of an earlier cell inside the loop that fills its list
    /// is set: each round reads a cell set before the loop or by an earlier
    /// round, so it is when each cell before the start of the loop is.
    ///
    /// # Arguments
    ///
    /// * `e` - The read.
    /// * `facts` - The facts.
    ///
    /// # Returns
    ///
    /// Whether it is set, or `None` for a read of another form.
    fn earlier_set(&self, e: &Expr, facts: &Facts) -> Option<bool>
    {
        let found = self.earlier.iter().find(|(read, _, _)| std::ptr::eq(*read, e));
        let (_, place, cells) = found?;
        let unset = facts.get(place).map(|u| &u.parts);
        Some(!cells.clone().any(|c| unset.is_some_and(|p| p.contains(&c))))
    }

    /// Run a read of an earlier cell: its cell number is read, and a cell that
    /// can be unset before the loop is reported, since a round reads it before
    /// each write.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address of the cell.
    /// * `set` - Whether the cells it needs are set.
    /// * `span` - The read.
    /// * `facts` - The facts.
    fn earlier_read(&mut self, addr: &'m Expr, set: bool, span: Span, facts: &Facts)
    {
        let ExprNode::CellAddr { list, index, .. } = &addr.node
        else
        {
            return;
        };
        self.read_expr(index, facts);
        ret_if!(set || !self.reporting);
        let ExprNode::Read(place) = &list.node
        else
        {
            return;
        };
        let name = self.name(*place);
        self.diags.push(od::read_unset(span, &name));
    }

    /// The part a read reaches, as the check follows it: a field of a slot
    /// that is no plain struct, such as the count of a plain list, reads the
    /// whole slot.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    /// * `part` - The part of the address.
    ///
    /// # Returns
    ///
    /// The part.
    fn plain_part(&self, place: Place, part: Part) -> Part
    {
        let Part::Field(..) = part
        else
        {
            return part;
        };
        let slot = match place
        {
            Place::Global(g) => self.ctx.module.globals.get(g as usize),
            Place::Local(l) => self.locals.get(l as usize),
        };
        let plain =
            slot.and_then(|s| s.kind.as_struct()).filter(|s| s.name != FALLIBLE);
        tern!(plain.is_some(), part, Part::Whole)
    }

    /// Report the globals a call reads that can be unset at it.
    ///
    /// # Arguments
    ///
    /// * `func` - The function, or `None` for a call through a function value.
    /// * `span` - The call.
    /// * `facts` - The facts.
    fn call(&mut self, func: Option<u32>, span: Span, facts: &Facts)
    {
        ret_if!(!self.calls || !self.reporting);
        let reads = match func
        {
            Some(f) => self.ctx.reads.get(f as usize),
            None => Some(self.ctx.through_values),
        };
        let unset = |g: &&u32| {
            facts.get(&Place::Global(**g)).is_some_and(|u| !u.parts.is_empty())
        };
        let missing: Vec<u32> =
            reads.into_iter().flatten().filter(unset).copied().collect();
        let callee = func.and_then(|f| self.ctx.module.funcs.get(f as usize));
        let callee = callee.map_or("", |f| f.name.as_str()).to_string();
        for g in missing
        {
            let global = self.name(Place::Global(g));
            self.diags.push(od::global_unset_at_call(span, &callee, &global));
        }
    }

    /// The name of a slot, for a message.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    ///
    /// # Returns
    ///
    /// The name.
    fn name(&self, place: Place) -> String
    {
        let slot = match place
        {
            Place::Global(g) => self.ctx.module.globals.get(g as usize),
            Place::Local(l) => self.locals.get(l as usize),
        };
        slot.map_or_else(String::new, |s| s.name.clone())
    }
}

/// Join the facts that reach a node along one more edge.
///
/// # Arguments
///
/// * `into` - The facts before the node so far.
/// * `more` - The facts of the edge.
///
/// # Returns
///
/// `true` when the facts grew.
fn merge(into: &mut Option<Facts>, more: Facts) -> bool
{
    let Some(old) = into
    else
    {
        *into = Some(more);
        return true;
    };
    let before = old.clone();
    for (place, unset) in more
    {
        let slot = old.entry(place).or_default();
        slot.parts.extend(unset.parts);
        slot.doubt |= unset.doubt;
    }
    *old != before
}

/// A write of a part of a slot.
///
/// # Arguments
///
/// * `facts` - The facts, changed in place.
/// * `place` - The slot.
/// * `part` - The part written.
fn write(facts: &mut Facts, place: Place, part: Part)
{
    let Some(unset) = facts.get_mut(&place)
    else
    {
        return;
    };
    match part
    {
        Part::Whole => *unset = Unset::default(),
        Part::Field(at, len) => (at..at + len).for_each(|f| _ = unset.parts.remove(&f)),
        Part::Cell(Some(c)) =>
        {
            _ = unset.parts.remove(&u32::try_from(c).unwrap_or(u32::MAX))
        }
        Part::Cell(None) => unset.doubt = true,
    }
}

/// The slot, the part and the cell position of an address: a field or a cell
/// of a variable, or a field of such a field, to each depth.
///
/// # Arguments
///
/// * `addr` - The address.
///
/// # Returns
///
/// The slot, the part and the expression of the position, or `None` for an
/// address of another form.
fn part_of<'e>(
    addr: &'e Expr, fills: &BTreeSet<Place>,
) -> Option<(Place, Part, Option<&'e Expr>)>
{
    match &addr.node
    {
        // The address a parameter marked `ဒေတာ ထည့်ပေးရန်` holds is the whole
        // place it fills.
        ExprNode::Read(place) if fills.contains(place) =>
        {
            Some((*place, Part::Whole, None))
        }
        ExprNode::FieldAddr { base, field, .. } =>
        {
            let (at, len) = field_leaves(base.kind, *field)?;
            let outer = match &base.node
            {
                ExprNode::Read(place) =>
                {
                    return Some((*place, Part::Field(at, len), None));
                }
                ExprNode::Load { addr } => part_of(addr, fills)?,
                _ => return None,
            };
            let (place, start, index) = match outer
            {
                (place, Part::Field(start, _), index) => (place, start, index),
                (place, Part::Whole, index) => (place, 0, index),
                _ => return None,
            };
            Some((place, Part::Field(start + at, len), index))
        }
        ExprNode::CellAddr { list, index, first, .. } =>
        {
            let place = list_slot(list, fills)?;
            let at = constant(index).and_then(|i| i.checked_sub(u64::from(*first)));
            Some((place, Part::Cell(at), Some(index)))
        }
        _ => None,
    }
}

/// The slot of a list a cell address names: a variable, the list a parameter
/// marked `ဒေတာ ထည့်ပေးရန်` fills, under the pin a changed argument adds.
///
/// # Arguments
///
/// * `list` - The list operand.
/// * `fills` - The parameters marked `ဒေတာ ထည့်ပေးရန်`.
///
/// # Returns
///
/// The slot, or `None` for a list of another form.
fn list_slot(list: &Expr, fills: &BTreeSet<Place>) -> Option<Place>
{
    match &list.node
    {
        ExprNode::Read(place) => Some(*place),
        ExprNode::Own { value, .. } => list_slot(value, fills),
        ExprNode::Load { addr } => match &addr.node
        {
            ExprNode::Read(place) if fills.contains(place) => Some(*place),
            _ => None,
        },
        _ => None,
    }
}

/// The place an argument for a changed parameter hands over: the address of
/// a variable, of a field or of a cell, or the place a parameter marked
/// `ဒေတာ ထည့်ပေးရန်` fills.
///
/// # Arguments
///
/// * `arg` - The argument.
/// * `fills` - The parameters marked `ဒေတာ ထည့်ပေးရန်`.
///
/// # Returns
///
/// The slot, the part and the expression of a cell position, or `None`.
fn arg_place<'e>(
    arg: &'e Expr, fills: &BTreeSet<Place>,
) -> Option<(Place, Part, Option<&'e Expr>)>
{
    match &arg.node
    {
        ExprNode::AddrOf(place) => Some((*place, Part::Whole, None)),
        ExprNode::Lend { operand } | ExprNode::Convert { operand, .. } =>
        {
            arg_place(operand, fills)
        }
        _ => part_of(arg, fills),
    }
}

/// The value of a constant whole number, under conversions.
///
/// # Arguments
///
/// * `e` - The expression.
///
/// # Returns
///
/// The number, or `None` for an expression that is no constant.
fn constant(e: &Expr) -> Option<u64>
{
    match &e.node
    {
        ExprNode::Convert { operand, .. } => constant(operand),
        ExprNode::Const(Value::Int { kind, raw })
            if !kind.signed || (*raw as i128) >= 0 =>
        {
            u64::try_from(*raw).ok()
        }
        _ => None,
    }
}

/// The slot a statement changes in place, such as the list an add changes.
///
/// # Arguments
///
/// * `s` - The statement.
///
/// # Returns
///
/// The slot and the statement.
fn dest_slot(s: &Stmt) -> Option<(Place, Span)>
{
    match s
    {
        Stmt::Free { dest: Dest::Slot(place), span }
        | Stmt::Resize { dest: Dest::Slot(place), span, .. }
        | Stmt::AddCell { dest: Dest::Slot(place), span, .. }
        | Stmt::RemoveCell { dest: Dest::Slot(place), span, .. }
        | Stmt::SetChar { dest: Dest::Slot(place), span, .. }
        | Stmt::Insert { dest: Dest::Slot(place), span, .. }
        | Stmt::Replace { dest: Dest::Slot(place), span, .. }
        | Stmt::AddText { dest: Dest::Slot(place), span, .. } => Some((*place, *span)),
        _ => None,
    }
}

/// Set the cells a counted loop fills, on the edge out of the loop: a loop of
/// fixed bounds, with no `break`, whose block writes the cell of its counter
/// at its top level, so each round writes one, sets each cell from its start
/// to its end.
///
/// # Arguments
///
/// * `s` - The counted loop.
/// * `tracked` - The followed slots.
/// * `facts` - The facts, changed in place.
fn fill(s: &Stmt, tracked: &BTreeMap<Place, (u32, Span)>, facts: &mut Facts)
{
    let Stmt::Counted { var, from, to, step: None, block, .. } = s
    else
    {
        return;
    };
    let (Some(lo), Some(hi)) = (constant(from), constant(to))
    else
    {
        return;
    };
    let mut broken = false;
    each_stmt(block, &mut |s| broken |= matches!(s, Stmt::Break { .. }));
    ret_if!(broken);
    ret_if!(lo > hi);
    for (place, first, len) in block.iter().filter_map(|s| written_cell(s, *var))
    {
        let Some((count, _)) = tracked.get(&place)
        else
        {
            continue;
        };
        let (Some(a), Some(b)) = (lo.checked_sub(first), hi.checked_sub(first))
        else
        {
            continue;
        };
        if *count > 1
        {
            let cells = (a..=b).filter_map(|c| u32::try_from(c).ok());
            cells.for_each(|c| write(facts, place, Part::Cell(Some(u64::from(c)))));
            continue;
        }
        let whole = a == 0 && len.is_some_and(|n| b + 1 >= u64::from(n));
        do_if!(whole, write(facts, place, Part::Whole));
    }
}

/// The reads of earlier cells inside each counted loop that writes the cell of
/// its counter on each round: a loop of fixed start, with no `break`, whose
/// block writes `a[i]` at its top level. A read of `a[i-k]` there, for a
/// constant `k` of at least 1, needs cells `s-k` to `s-1` set before the loop,
/// where `s` is the start.
///
/// # Arguments
///
/// * `block` - The statements of a routine.
///
/// # Returns
///
/// Each such read, with its list and the cells it needs.
fn earlier_reads(block: &[Stmt]) -> Vec<(*const Expr, Place, std::ops::Range<u32>)>
{
    let mut found = Vec::new();
    each_stmt(block, &mut |s| found.extend(earlier_in(s)));
    found
}

/// The reads of earlier cells inside one counted loop, as
/// [`earlier_reads`] gives them.
///
/// # Arguments
///
/// * `s` - The statement.
///
/// # Returns
///
/// The reads, none when the statement is no such loop.
fn earlier_in(s: &Stmt) -> Vec<(*const Expr, Place, std::ops::Range<u32>)>
{
    let Stmt::Counted { var, from, step: None, block, .. } = s
    else
    {
        return Vec::new();
    };
    let Some(lo) = constant(from)
    else
    {
        return Vec::new();
    };
    let mut broken = false;
    each_stmt(block, &mut |s| broken |= matches!(s, Stmt::Break { .. }));
    let filled = block.iter().find_map(|s| written_cell(s, *var)).filter(|_| !broken);
    let Some((list, first, _)) = filled
    else
    {
        return Vec::new();
    };
    let mut found = Vec::new();
    let mut note = |e: &Expr| found.extend(earlier_cell(e, list, *var, lo, first));
    each_stmt(block, &mut |s| do_for!(e in stmt_exprs(s), each_expr(e, &mut note)));
    found
}

/// The list, the number of its first cell and its cell count, of a statement
/// that writes the cell of a counter: `a[i] = ...`.
///
/// # Arguments
///
/// * `s` - The statement.
/// * `var` - The counter.
///
/// # Returns
///
/// The list, the number of its first cell and its cell count when known, or
/// `None`.
fn written_cell(s: &Stmt, var: Place) -> Option<(Place, u64, Option<u32>)>
{
    let Stmt::Store { addr, .. } = s
    else
    {
        return None;
    };
    let ExprNode::CellAddr { list, index, first, len, .. } = &addr.node
    else
    {
        return None;
    };
    let ExprNode::Read(place) = &list.node
    else
    {
        return None;
    };
    is_read_of(index, var).then_some((*place, u64::from(*first), *len))
}

/// A read of an earlier cell of a list: `a[i-k]` for a constant `k` of at
/// least 1, with the cells it needs set before the loop.
///
/// # Arguments
///
/// * `e` - The expression.
/// * `list` - The list the loop fills.
/// * `var` - The counter.
/// * `lo` - The start of the counter.
/// * `first` - The number of the first cell.
///
/// # Returns
///
/// The read, its list and the cells, or `None`.
fn earlier_cell(
    e: &Expr, list: Place, var: Place, lo: u64, first: u64,
) -> Option<(*const Expr, Place, std::ops::Range<u32>)>
{
    let ExprNode::Load { addr } = &e.node
    else
    {
        return None;
    };
    let ExprNode::CellAddr { list: of, index, .. } = &addr.node
    else
    {
        return None;
    };
    ret_if!(!matches!(of.node, ExprNode::Read(p) if p == list), None);
    let k = back_step(index, var)?;
    // The first round reads cell `lo - k`; one before the first cell is out
    // of range, which the run stops on, so no proof is made.
    let start = lo.checked_sub(k)?.checked_sub(first)?;
    let end = lo.checked_sub(first)?;
    let cells = u32::try_from(start).ok()?..u32::try_from(end).ok()?;
    Some((e as *const Expr, list, cells))
}

/// The step back of a cell number `i - k` from a counter, under conversions.
///
/// # Arguments
///
/// * `e` - The cell number.
/// * `var` - The counter.
///
/// # Returns
///
/// `k` when it is a constant of at least 1, else `None`.
fn back_step(e: &Expr, var: Place) -> Option<u64>
{
    match &e.node
    {
        ExprNode::Convert { operand, .. } => back_step(operand, var),
        ExprNode::Infix { op: InfixOp::Sub, lhs, rhs } if is_read_of(lhs, var) =>
        {
            constant(rhs).filter(|&k| k >= 1)
        }
        _ => None,
    }
}

/// Whether an expression reads a slot, under conversions.
///
/// # Arguments
///
/// * `e` - The expression.
/// * `var` - The slot.
///
/// # Returns
///
/// `true` when it does.
fn is_read_of(e: &Expr, var: Place) -> bool
{
    match &e.node
    {
        ExprNode::Convert { operand, .. } => is_read_of(operand, var),
        ExprNode::Read(place) => *place == var,
        _ => false,
    }
}
