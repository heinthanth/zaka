//! Threads: a call that runs in parallel becomes a spawn of the function it
//! calls, and a call of the core's `စောင့်` on a thread becomes a wait for it.

use zakac_semcheck::semcheck::each_list;
use zakac_semcheck::task;
use zakac_semcheck::threads::ATOMIC_ADD;
use zakac_semcheck::threads::ATOMIC_CAS;
use zakac_semcheck::threads::ATOMIC_SUB;
use zakac_semcheck::threads::CLOSE;
use zakac_semcheck::threads::SEND;
use zakac_semcheck::threads::WAIT;

use super::*;
use crate::ir::Par;
use crate::ir::ParMap;

impl Lowerer<'_>
{
    /// The thread a call that runs in parallel starts. Its arguments are
    /// lowered as for a call; the thread takes a duplicate of each.
    ///
    /// # Arguments
    ///
    /// * `callee` - The name at the call.
    /// * `place` - The value a call on a value names, or `None`.
    /// * `args` - The arguments.
    ///
    /// # Returns
    ///
    /// The spawn.
    pub(super) fn spawn_node(
        &mut self, callee: &ast::Ident, place: Option<&ast::Expr>, args: &[ast::Expr],
    ) -> ExprNode
    {
        let (target, sig) = self.call_target(callee, place);
        let args = self.lower_args(args, &sig);
        // A thread gets no copies back: the check refuses a parameter the
        // function changes.
        self.post.clear();
        let Target::Fn { func, name } = target
        else
        {
            unreachable!("the check lets a thread call a declared function alone")
        };
        ExprNode::Spawn { func, name, args }
    }

    /// The thread a wait waits for: the one argument of a call of the core's
    /// `စောင့်`, when it is a thread.
    ///
    /// # Arguments
    ///
    /// * `callee` - The name at the call.
    /// * `place` - The value a call on a value names, or `None`.
    /// * `args` - The arguments.
    ///
    /// # Returns
    ///
    /// The thread, or `None` for a call that is no wait.
    pub(super) fn waited<'a>(
        &self, callee: &ast::Ident, place: Option<&ast::Expr>, args: &'a [ast::Expr],
    ) -> Option<&'a ast::Expr>
    {
        let [thread] = args
        else
        {
            return None;
        };
        ret_if!(place.is_some() || callee.name.as_str() != WAIT, None);
        self.kind_of(thread).as_task().filter(|t| t.is_thread()).map(|_| thread)
    }

    /// Lower a wait for a thread whose value is read.
    ///
    /// # Arguments
    ///
    /// * `n` - The call.
    ///
    /// # Returns
    ///
    /// The wait, or `None` for a call that is no wait.
    pub(super) fn wait_node(&mut self, n: &ast::CallExpr) -> Option<ExprNode>
    {
        let thread = self.waited(&n.callee, n.place.as_deref(), &n.args)?;
        Some(ExprNode::Wait(Box::new(self.expr(thread))))
    }

    /// Lower a call that runs in parallel as a statement: the thread starts,
    /// and its number goes to a hidden slot no one reads.
    ///
    /// # Arguments
    ///
    /// * `n` - The call.
    pub(super) fn spawn_stmt(&mut self, n: &st::CallStmt)
    {
        let node = self.spawn_node(&n.callee, n.place.as_deref(), &n.args);
        let kind = DataKind::Task(task::make(false, None));
        self.spill(node_expr(kind, node, n.span));
    }

    /// Lower a send or a close of a channel: a call of the core's `ပို့` or
    /// `ပိတ်` on a channel.
    ///
    /// # Arguments
    ///
    /// * `n` - The call.
    ///
    /// # Returns
    ///
    /// The statement, or `None` for a call that is neither.
    pub(super) fn pipe_stmt(&mut self, n: &st::CallStmt) -> Option<Stmt>
    {
        ret_if!(n.place.is_some(), None);
        let name = n.callee.name.as_str();
        let pipe = n.args.first().filter(|a| self.is_pipe(a))?;
        match n.args.as_slice()
        {
            [_, value] if name == SEND =>
            {
                let carried = self.kind_of(pipe).as_task().and_then(|t| t.of)?;
                let pipe = self.expr(pipe);
                let value = self.owned(value, carried);
                Some(Stmt::Send { pipe, value, span: n.span })
            }
            [_] if name == CLOSE =>
            {
                Some(Stmt::Close { pipe: self.expr(pipe), span: n.span })
            }
            _ => None,
        }
    }

    /// Lower an add to an atomic number, or a subtraction: a call of the core's
    /// `ပေါင်းထည့်` or `နုတ်ယူ` whose first argument is an atomic number.
    ///
    /// # Arguments
    ///
    /// * `n` - The call.
    ///
    /// # Returns
    ///
    /// The statement, or `None` for a call that is neither.
    pub(super) fn atomic_stmt(&mut self, n: &st::CallStmt) -> Option<Stmt>
    {
        let name = n.callee.name.as_str();
        let sub = name == ATOMIC_SUB;
        ret_if!(n.place.is_some() || (name != ATOMIC_ADD && !sub), None);
        let [target, value] = n.args.as_slice()
        else
        {
            return None;
        };
        let int = self.kind_of(target);
        ret_if!(int.int_kind().is_none() || !self.holds_atomic(target), None);
        let addr = node_expr(DataKind::OPAQUE, self.addr_of(target), target.span());
        let value = self.owned(value, int);
        Some(Stmt::AtomicAdd { addr, value, sub, span: n.span })
    }

    /// Lower a compare-and-set of an atomic number: the set gives back what
    /// the number held, and the result is good, holding it, when that was the
    /// expected value, and an error holding it otherwise.
    ///
    /// # Arguments
    ///
    /// * `n` - The call.
    /// * `kind` - The result kind.
    ///
    /// # Returns
    ///
    /// A read of the result, or `None` for a call that is no compare-and-set.
    pub(super) fn cas_node(
        &mut self, n: &ast::CallExpr, kind: DataKind,
    ) -> Option<ExprNode>
    {
        ret_if!(n.place.is_some() || n.callee.name.as_str() != ATOMIC_CAS, None);
        let [target, expected, new] = n.args.as_slice()
        else
        {
            return None;
        };
        ret_if!(!self.holds_atomic(target), None);
        let fault = Fault::of_kind(kind)?;
        let int = fault.ok?;
        let span = n.span;
        let addr = node_expr(DataKind::OPAQUE, self.addr_of(target), target.span());
        let expected = self.owned(expected, int);
        let expected = self.spill(expected);
        let new = self.owned(new, int);
        #[rustfmt::skip]
        let cas = ExprNode::AtomicCas { addr: Box::new(addr),
            expected: Box::new(expected.clone()), new: Box::new(new) };
        let held = self.spill(node_expr(int, cas, span));
        let set = node_expr(
            DataKind::Bool,
            compare_node(CmpOp::Eq, held.clone(), expected),
            span,
        );
        let result = self.hidden(kind, span);
        let good = fault.value(false, Some(held.clone()), None, self.platform, span);
        let bad = fault.value(true, None, Some(held), self.platform, span);
        let then = vec![Stmt::Set { place: result, value: good, span }];
        let els = vec![Stmt::Set { place: result, value: bad, span }];
        self.pre.push(Stmt::If { cond: set, then, els, span });
        Some(ExprNode::Read(result))
    }

    /// Whether a place holds an atomic number: a variable, or a field.
    ///
    /// # Arguments
    ///
    /// * `e` - The place.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(super) fn holds_atomic(&mut self, e: &ast::Expr) -> bool
    {
        // The address of a field forgets that it holds an atomic number, so a
        // field is found through the struct of its base.
        let held = match e
        {
            ast::Expr::Group(g) => return self.holds_atomic(&g.inner),
            ast::Expr::Field(f) =>
            {
                let shape = self.kind_of(&f.base).object_struct();
                shape.and_then(|s| s.field(&f.field.name)).map(|(_, k)| k)
            }
            ast::Expr::Name(n) =>
            {
                let place = self.lookup(&n.name);
                return self.reaches_atomic(place);
            }
            _ => None,
        };
        held.and_then(DataKind::as_task).is_some_and(|t| t.atomic)
    }

    /// Whether a slot holds an atomic number, or the address of one from
    /// outside a round of a parallel loop.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(super) fn reaches_atomic(&mut self, place: Place) -> bool
    {
        let held = self.reached_kind(place).as_task().is_some_and(|t| t.atomic);
        let Place::Local(n) = place
        else
        {
            return held;
        };
        held || self.frame.as_ref().is_some_and(|f| f.atomics.contains(&n))
    }

    /// Whether a value is a channel.
    ///
    /// # Arguments
    ///
    /// * `e` - The value.
    ///
    /// # Returns
    ///
    /// `true` for a channel.
    fn is_pipe(&self, e: &ast::Expr) -> bool
    {
        self.kind_of(e).as_task().is_some_and(|t| t.pipe)
    }

    /// Receive the next value of a channel into a hidden optional, before the
    /// statement being lowered.
    ///
    /// # Arguments
    ///
    /// * `pipe` - The channel.
    /// * `span` - The receive.
    ///
    /// # Returns
    ///
    /// The hidden optional.
    fn receive_into(&mut self, pipe: Expr, span: Span) -> Place
    {
        let carried = pipe.kind.as_task().and_then(|t| t.of);
        let carried = carried.expect("the check found a channel");
        let optional = DataKind::optional(carried);
        let got =
            self.spill(node_expr(optional, ExprNode::Receive(Box::new(pipe)), span));
        let ExprNode::Read(place) = got.node
        else
        {
            unreachable!("a receive goes to a hidden slot")
        };
        place
    }

    /// Lower an arm of `အကယ်၍` whose whole condition tests a channel for data:
    /// the next value goes to a hidden optional, the arm runs when it holds
    /// one, and `<pipe> ထဲမှ ဒေတာ` in the arm reads it.
    ///
    /// # Arguments
    ///
    /// * `arm` - The arm.
    ///
    /// # Returns
    ///
    /// The test, the block and the span of the arm, or `None` for an arm
    /// whose condition tests no channel.
    pub(super) fn pipe_arm(
        &mut self, arm: &st::IfArm,
    ) -> Option<(Expr, Vec<Stmt>, Span)>
    {
        let ast::Expr::Verb(v) = &arm.cond
        else
        {
            return None;
        };
        let [ast::Expr::Name(pipe)] = v.args.as_slice()
        else
        {
            return None;
        };
        ret_if!(v.op != VerbOp::HasData || !self.is_pipe(&v.args[0]), None);
        let lowered = self.expr(&v.args[0]);
        let got = self.receive_into(lowered, v.span);
        let fault =
            Fault::of_kind(self.slot_kind(got)).expect("an optional is a fault kind");
        let failed = fault.failed(got, self.platform, v.span);
        let cond =
            node_expr(DataKind::Bool, prefix_node(PrefixOp::Not, failed), v.span);
        self.pipe_reads.push((pipe.name, got));
        let block = self.block(&arm.block.stmts);
        self.pipe_reads.pop();
        Some((cond, block, arm.span))
    }

    /// Lower `<pipe> ထဲမှ ဒေတာ` inside the arm that tested the channel: the
    /// data of the value that came.
    ///
    /// # Arguments
    ///
    /// * `value` - The channel.
    /// * `span` - The read.
    ///
    /// # Returns
    ///
    /// The read, or `None` for a value that is no channel.
    pub(super) fn pipe_data(
        &mut self, value: &ast::Expr, span: Span,
    ) -> Option<ExprNode>
    {
        ret_if!(!self.is_pipe(value), None);
        let ast::Expr::Name(n) = value
        else
        {
            unreachable!("the check lets a channel read be a name alone")
        };
        let found = self.pipe_reads.iter().rev().find(|(name, _)| *name == n.name);
        let (_, got) = *found.expect("the check found the test of the channel");
        let fault =
            Fault::of_kind(self.slot_kind(got)).expect("an optional is a fault kind");
        Some(fault.field(got, "ok", self.platform, span).node)
    }

    /// Lower a loop over each value of a channel: each round receives the
    /// next value, and the loop ends once the channel is closed and drained.
    ///
    /// # Arguments
    ///
    /// * `n` - The loop.
    /// * `out` - The list the statements go to.
    pub(super) fn each_received(&mut self, n: &st::EachStmt, out: &mut Vec<Stmt>)
    {
        let span = n.span;
        let pipe = self.expr(&n.source);
        let pipe = self.spill(pipe);
        let carried = pipe.kind.as_task().and_then(|t| t.of);
        let carried = carried.expect("the check found a channel");
        self.loops.push(self.scopes.len());
        let mut round = Vec::new();
        self.scoped(|l| {
            let (pre, got) = l.with_pre(|l| l.receive_into(pipe, span));
            round.extend(pre);
            let fault =
                Fault::of_kind(l.slot_kind(got)).expect("an optional is a fault kind");
            let mut leave = l.drops_from(l.drops.len() - 1, span);
            leave.push(Stmt::Break { span });
            let failed = fault.failed(got, l.platform, span);
            round.push(Stmt::If { cond: failed, then: leave, els: Vec::new(), span });
            let data = fault.field(got, "ok", l.platform, span);
            let data = tern!(carried.holds_text(), dup(data), data);
            let var = l.declare(&n.var, carried, Access::Var);
            round.push(Stmt::Set { place: var, value: data, span });
            l.scoped_stmts(&n.block.stmts, &mut round);
            let leaves = round.last().is_some_and(leaves_scope);
            do_if!(!leaves, round.extend(l.drops_from(l.drops.len() - 1, span)));
        });
        self.loops.pop();
        let cond = true_expr(span);
        out.push(Stmt::Loop {
            init: Vec::new(),
            cond,
            update: Vec::new(),
            block: round,
            span,
            par: None,
        });
    }

    /// Lower a parallel map: a plain list in a hidden slot, which a loop fills
    /// with what the function gives back for each cell, in the order of the
    /// cells. The VM runs the loop; a native build runs the rounds at once,
    /// through the mark on the loop.
    ///
    /// # Arguments
    ///
    /// * `n` - The map.
    /// * `kind` - The kind of the plain list it gives.
    ///
    /// # Returns
    ///
    /// The read of the plain list.
    pub(super) fn par_map(&mut self, n: &ast::VerbExpr, kind: DataKind) -> ExprNode
    {
        let span = n.span;
        let (list, f, threads) = match n.args.as_slice()
        {
            [list, ast::Expr::Name(f)] => (list, f, None),
            [list, ast::Expr::Name(f), threads] => (list, f, Some(threads)),
            _ => unreachable!("the parser gives a map its list and its function"),
        };
        let threads = threads.map(|t| {
            let mut out = Vec::new();
            let slot = self.thread_count(t, &mut out);
            self.pre.extend(out);
            slot
        });
        let given = self.kind_of(list);
        let list_kind = each_list(given, false);
        let source = tern!(
            list_kind == given,
            self.owned(list, list_kind),
            self.through(list, list_kind)
        );
        // A counted list holds its cells in place, and the rounds of a native
        // build read a plain list, so the map reads a plain list made of it.
        let cell = list_kind.cell_kind().filter(|_| list_kind.is_list());
        let list_kind = cell.map_or(list_kind, DataKind::seq_of);
        let src = self.spill(coerce_lowered(source, list_kind, self.platform));
        let res = self.spill(node_expr(kind, ExprNode::BlankList, span));
        let (ExprNode::Read(each), ExprNode::Read(out)) = (src.node, res.node)
        else
        {
            unreachable!("a spill gives a read of a hidden slot")
        };
        let sig =
            self.sigs.get(&f.name).cloned().expect("the check found the function");
        let param = sig.params[0];
        // Cells that widen to the parameter go to a list of its kind first,
        // so the rounds of a native build take each cell as it is.
        let widens = list_kind.cell_kind() != Some(param) && !param.holds_text();
        let (each, list_kind) = tern!(
            widens,
            self.widened(each, list_kind, param, span),
            (each, list_kind)
        );
        let ret = kind.cell_kind().expect("a map gives a plain list");
        let name = f.name.to_string();
        let platform = self.platform;
        let rounds = self.each_loop(each, list_kind, span, |value| {
            let arg = coerce_lowered(value, param, platform);
            let call = ExprNode::Call { func: sig.index, name, args: vec![arg] };
            let value = node_expr(ret, call, span);
            Stmt::AddCell { dest: Dest::Slot(out), kind, value, span }
        });
        let Stmt::Loop { par: slot, .. } = rounds
        else
        {
            unreachable!("each_loop gives a loop")
        };
        // A native build runs the rounds at once when the cells go to the
        // function as each is.
        let as_is = list_kind.cell_kind() == Some(param);
        let map = ParMap {
            src: each,
            out,
            func: sig.index,
            src_kind: list_kind,
            out_kind: kind,
            threads,
        };
        *slot = as_is.then(|| Box::new(Par::Map(map)));
        ExprNode::Read(out)
    }

    /// Widen each cell of a list into a hidden plain list of the kind of a
    /// parameter, in order.
    ///
    /// # Arguments
    ///
    /// * `each` - The hidden slot of the list.
    /// * `list_kind` - Its kind.
    /// * `param` - The kind of the parameter.
    /// * `span` - The map.
    ///
    /// # Returns
    ///
    /// The hidden slot of the widened list, and its kind.
    fn widened(
        &mut self, each: Place, list_kind: DataKind, param: DataKind, span: Span,
    ) -> (Place, DataKind)
    {
        let kind = DataKind::seq_of(param);
        let wide = self.spill(node_expr(kind, ExprNode::BlankList, span));
        let ExprNode::Read(wide) = wide.node
        else
        {
            unreachable!("a spill gives a read of a hidden slot")
        };
        let platform = self.platform;
        self.each_loop(each, list_kind, span, |value| {
            let value = coerce_lowered(value, param, platform);
            Stmt::AddCell { dest: Dest::Slot(wide), kind, value, span }
        });
        (wide, kind)
    }

    /// A loop over each cell of a list, in order, whose block is the one
    /// statement `round` makes of the value of the cell; it goes to the
    /// statements before the one being lowered.
    ///
    /// # Arguments
    ///
    /// * `each` - The hidden slot of the list.
    /// * `list_kind` - Its kind.
    /// * `span` - The source.
    /// * `round` - Makes the statement of a round from the value of its cell.
    ///
    /// # Returns
    ///
    /// The loop, in place among the statements before.
    fn each_loop(
        &mut self, each: Place, list_kind: DataKind, span: Span,
        round: impl FnOnce(Expr) -> Stmt,
    ) -> &mut Stmt
    {
        let at = self.hidden(DataKind::USIZE, span);
        self.pre.push(Stmt::Set { place: at, value: self.usize_const(0, span), span });
        let count = self.each_count(each, list_kind, span);
        let at_read = read(at, DataKind::USIZE, span);
        let cond = node_expr(
            DataKind::Bool,
            compare_node(CmpOp::Lt, at_read.clone(), count),
            span,
        );
        let (value, _) = each_cell(each, list_kind, at, span);
        let one = self.usize_const(1, span);
        let next =
            node_expr(DataKind::USIZE, infix_node(InfixOp::Add, at_read, one), span);
        let update = vec![Stmt::Set { place: at, value: next, span }];
        let block = vec![round(value)];
        #[rustfmt::skip]
        self.pre.push(Stmt::Loop { init: Vec::new(), cond, update, block, span, par: None });
        self.pre.last_mut().expect("the loop was pushed")
    }

    /// Lower a lock block: the lock is taken, the block runs, and each exit
    /// of it gives the lock back, after the drops of the block.
    ///
    /// # Arguments
    ///
    /// * `n` - The block.
    /// * `out` - The list the statements go to.
    pub(super) fn lock_stmt(&mut self, n: &st::LockStmt, out: &mut Vec<Stmt>)
    {
        let obj = self.read_value(&n.obj);
        out.push(Stmt::Lock { obj: obj.clone(), span: n.span });
        self.scoped(|l| {
            let depth = l.drops.len() - 1;
            l.locks.push((depth, obj));
            l.scoped_stmts(&n.block.stmts, out);
            let leaves = out.last().is_some_and(leaves_scope);
            do_if!(!leaves, out.extend(l.drops_from(depth, n.span)));
            l.locks.pop();
        });
    }

    /// Lower a wait for a thread as a statement, its result dropped.
    ///
    /// # Arguments
    ///
    /// * `n` - The call.
    ///
    /// # Returns
    ///
    /// The wait, or `None` for a call that is no wait.
    pub(super) fn wait_stmt(&mut self, n: &st::CallStmt) -> Option<Stmt>
    {
        let thread = self.waited(&n.callee, n.place.as_deref(), &n.args)?;
        let kind = self.kind_of(thread).as_task().and_then(|t| t.of);
        let thread = self.expr(thread);
        Some(Stmt::Wait { thread, kind, span: n.span })
    }
}
