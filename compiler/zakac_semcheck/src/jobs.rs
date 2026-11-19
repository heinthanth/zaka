//! A call that runs as a job: its value is the job, and each value that goes
//! to the job, or comes back from it, must be one jobs on other threads can
//! use (ZK0494).

use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::visit::Node;
use zakac_diagnostics::kinds as kd;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_span::span::Span;

use crate::kind::DataKind;
use crate::semcheck::Checker;
use crate::task;

/// The verb of the core that waits for a job.
pub const WAIT: &str = "စောင့်";

impl Checker
{
    /// The job a call that runs in parallel gives, after a report of each
    /// argument, and of the result, that jobs cannot use.
    ///
    /// # Arguments
    ///
    /// * `args` - The arguments of the call.
    /// * `ret` - The kind its function gives back, or `None`.
    /// * `span` - The call.
    ///
    /// # Returns
    ///
    /// The job kind.
    pub(crate) fn start_job(
        &mut self, args: &[Expr], ret: Option<DataKind>, span: Span,
    ) -> DataKind
    {
        for arg in args
        {
            let kind = self.kinds.get(&arg.id()).copied();
            do_if!(let Some(kind) = kind, self.goes_to_job(kind, arg.span()));
        }
        do_if!(let Some(kind) = ret, self.goes_to_job(kind, span));
        DataKind::Task(task::make(false, ret))
    }

    /// A wait for a job, `<job> ကို စောင့်မည်` or `<job> ကို စောင့်သော ရလဒ်`: a call of
    /// the core's `စောင့်` on one value of a job kind.
    ///
    /// # Arguments
    ///
    /// * `callee` - The name at the call.
    /// * `place` - The value a call on a value names, if any.
    /// * `args` - The arguments.
    ///
    /// # Returns
    ///
    /// `Some` with the kind the function of the job gives back, or `None` for
    /// a call that is no wait.
    pub(crate) fn wait_call(
        &mut self, callee: &Ident, place: Option<&Expr>, args: &[Expr],
    ) -> Option<Option<DataKind>>
    {
        let [job] = args
        else
        {
            return None;
        };
        ret_if!(place.is_some() || callee.name.as_str() != WAIT, None);
        let task = self.infer(job, None)?.as_task().filter(|t| !t.pipe)?;
        Some(task.of)
    }

    /// Report a kind that jobs cannot use, where a value of it goes to a job
    /// or comes back from one.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    /// * `span` - The value.
    pub(crate) fn goes_to_job(&mut self, kind: DataKind, span: Span)
    {
        do_if!(
            !self.usable_in_jobs(kind),
            self.report(kd::not_for_jobs(span, &kind.name()))
        );
    }
}
