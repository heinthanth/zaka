//! Jobs: a call that runs in parallel becomes a spawn of the function it
//! calls, and a call of the core's `စောင့်` on a job becomes a wait for it.

use zakac_semcheck::jobs::CLOSE;
use zakac_semcheck::jobs::SEND;
use zakac_semcheck::jobs::WAIT;
use zakac_semcheck::task;

use super::*;

impl Lowerer<'_>
{
    /// The job a call that runs in parallel starts. Its arguments are
    /// lowered as for a call; the job takes a duplicate of each.
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
        // A job gets no copies back: the check refuses a parameter the
        // function changes.
        self.post.clear();
        let Target::Fn { func, name } = target
        else
        {
            unreachable!("the check lets a job call a declared function alone")
        };
        ExprNode::Spawn { func, name, args }
    }

    /// The job a wait waits for: the one argument of a call of the core's
    /// `စောင့်`, when it is a job.
    ///
    /// # Arguments
    ///
    /// * `callee` - The name at the call.
    /// * `place` - The value a call on a value names, or `None`.
    /// * `args` - The arguments.
    ///
    /// # Returns
    ///
    /// The job, or `None` for a call that is no wait.
    pub(super) fn waited<'a>(
        &self, callee: &ast::Ident, place: Option<&ast::Expr>, args: &'a [ast::Expr],
    ) -> Option<&'a ast::Expr>
    {
        let [job] = args
        else
        {
            return None;
        };
        ret_if!(place.is_some() || callee.name.as_str() != WAIT, None);
        self.kind_of(job).as_task().filter(|t| !t.pipe).map(|_| job)
    }

    /// Lower a wait for a job whose value is read.
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
        let job = self.waited(&n.callee, n.place.as_deref(), &n.args)?;
        Some(ExprNode::Wait(Box::new(self.expr(job))))
    }

    /// Lower a call that runs in parallel as a statement: the job starts,
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
        });
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

    /// Lower a wait for a job as a statement, its result dropped.
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
        let job = self.waited(&n.callee, n.place.as_deref(), &n.args)?;
        let kind = self.kind_of(job).as_task().and_then(|t| t.of);
        let job = self.expr(job);
        Some(Stmt::Wait { job, kind, span: n.span })
    }
}
