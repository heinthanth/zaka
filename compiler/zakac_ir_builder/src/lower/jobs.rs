//! Jobs: a call that runs in parallel becomes a spawn of the function it
//! calls, and a call of the core's `စောင့်` on a job becomes a wait for it.

use zakac_semcheck::jobs::WAIT;
use zakac_semcheck::task;

use super::*;

impl Lowerer<'_>
{
    /// The job a call that runs in parallel starts. Its arguments are
    /// lowered as for a call; the job takes its own copy of each.
    ///
    /// # Arguments
    ///
    /// * `callee` - The name at the call.
    /// * `place` - The value a call on a value names, if any.
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
    /// * `place` - The value a call on a value names, if any.
    /// * `args` - The arguments.
    ///
    /// # Returns
    ///
    /// The job, or `None` for a call that is no wait.
    fn waited<'a>(
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
