//! A call that runs as a job: its value is the job, and each value that goes
//! to the job, or comes back from it, must be one jobs on other threads can
//! use (ZK0494).

use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::name::Name;
use zakac_ast::stmt::EachStmt;
use zakac_ast::verb::VerbOp;
use zakac_ast::visit::Node;
use zakac_diagnostics::kinds as kd;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::kind::DataKind;
use crate::proof::result_var;
use crate::semcheck::Checker;
use crate::task;

/// The verb of the core that waits for a job.
pub const WAIT: &str = "စောင့်";

/// The verb of the core that sends a value to a channel.
pub const SEND: &str = "ပို့";

/// The verb of the core that closes a channel.
pub const CLOSE: &str = "ပိတ်";

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
    /// * `place` - The value a call on a value names, or `None`.
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

    /// A send, `<pipe> ထဲသို့ <value> ကို ပို့မည်`, or a close, `<pipe> ကို
    /// ပိတ်မည်`: a call of the core's `ပို့` or `ပိတ်` on a channel.
    ///
    /// # Arguments
    ///
    /// * `callee` - The name at the call.
    /// * `place` - The value a call on a value names, or `None`.
    /// * `args` - The arguments.
    ///
    /// # Returns
    ///
    /// `Some(None)`, for neither gives a value, or `None` for a call that is
    /// neither.
    pub(crate) fn pipe_call(
        &mut self, callee: &Ident, place: Option<&Expr>, args: &[Expr],
    ) -> Option<Option<DataKind>>
    {
        ret_if!(place.is_some(), None);
        let name = callee.name.as_str();
        match args
        {
            [pipe, value] if name == SEND && self.names_pipe(pipe) =>
            {
                let carried = self.pipe_of(pipe);
                self.expect_kind(value, carried);
                do_if!(let Some(kind) = carried, self.goes_to_job(kind, value.span()));
                Some(None)
            }
            [pipe] if name == CLOSE && self.names_pipe(pipe) => Some(None),
            _ => None,
        }
    }

    /// Whether a value is a channel, which a call of the core's `ပို့` or
    /// `ပိတ်` takes in place of the overloads a program declares.
    ///
    /// # Arguments
    ///
    /// * `e` - The value.
    ///
    /// # Returns
    ///
    /// `true` for a channel.
    fn names_pipe(&mut self, e: &Expr) -> bool
    {
        self.infer(e, None).and_then(DataKind::as_task).is_some_and(|t| t.pipe)
    }

    /// The kind a channel carries, after a report of a value that is no
    /// channel.
    ///
    /// # Arguments
    ///
    /// * `e` - The value.
    ///
    /// # Returns
    ///
    /// The kind, or `None` after a report.
    fn pipe_of(&mut self, e: &Expr) -> Option<DataKind>
    {
        let kind = self.infer(e, None)?;
        let task = kind.as_task().filter(|t| t.pipe);
        ret_if!(
            task.is_none(),
            self.report(kd::not_a_pipe(e.span(), &kind.name())),
            None
        );
        task.and_then(|t| t.of)
    }

    /// Infer a receive, `<pipe> မှ ရရှိသော ဒေတာ`: an optional of the kind the
    /// channel carries.
    ///
    /// # Arguments
    ///
    /// * `pipe` - The channel.
    ///
    /// # Returns
    ///
    /// The kind, or `None` after an error.
    pub(crate) fn infer_receive(&mut self, pipe: &Expr) -> Option<DataKind>
    {
        self.pipe_of(pipe).map(DataKind::optional)
    }

    /// The kind of the variable of a loop over each value of a channel.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of the value the loop walks.
    /// * `n` - The loop.
    ///
    /// # Returns
    ///
    /// The kinds of the two names of the loop, or `None` after a report.
    pub(crate) fn received_var_kind(
        &mut self, kind: DataKind, n: &EachStmt,
    ) -> Option<(Option<DataKind>, DataKind)>
    {
        let carried = kind.as_task().filter(|t| t.pipe).and_then(|t| t.of);
        ret_if!(let Some(carried) = carried, Some((None, carried)));
        self.report(kd::not_a_pipe(n.source.span(), &kind.name()));
        None
    }

    /// Check the condition of an arm of `အကယ်၍`, where a test of a channel for
    /// data can stand alone.
    ///
    /// # Arguments
    ///
    /// * `cond` - The condition.
    ///
    /// # Returns
    ///
    /// The channel the condition tests, whose data its branch can read.
    pub(crate) fn check_arm_cond(&mut self, cond: &Expr) -> Option<Name>
    {
        let tested = match cond
        {
            Expr::Verb(v) if v.op == VerbOp::HasData && v.args.len() == 1 =>
            {
                Some(&v.args[0])
            }
            _ => None,
        };
        let outer = std::mem::replace(&mut self.pipe_test, tested.map(Expr::id));
        self.expect_bool(cond);
        self.pipe_test = outer;
        let pipe = tested.filter(|t| {
            let kind = self.kinds.get(&t.id()).copied();
            kind.and_then(DataKind::as_task).is_some_and(|k| k.pipe)
        });
        pipe.and_then(result_var)
    }

    /// Infer a test of a channel for data, or a read of its data: the test
    /// stands alone as the condition of `အကယ်၍` (ZK0496 elsewhere), and the read
    /// inside the branch of that test.
    ///
    /// # Arguments
    ///
    /// * `op` - The verb.
    /// * `value` - The channel.
    /// * `kind` - Its kind.
    ///
    /// # Returns
    ///
    /// The kind of the test or the read.
    pub(crate) fn pipe_part(
        &mut self, op: VerbOp, value: &Expr, kind: DataKind,
    ) -> Option<DataKind>
    {
        let name = result_var(value);
        let ok = match op
        {
            VerbOp::HasData => self.pipe_test == Some(value.id()) && name.is_some(),
            VerbOp::Unwrap => name.is_some_and(|n| self.pipe_reads.contains(&n)),
            _ => false,
        };
        do_if!(!ok, self.report(kd::pipe_data_outside(value.span())));
        let carried = kind.as_task().and_then(|t| t.of);
        tern!(op == VerbOp::Unwrap, carried, Some(DataKind::Bool))
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
