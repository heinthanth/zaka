use zakac_ast::expr::Expr;
use zakac_ast::stmt::SwitchStmt;
use zakac_ast::visit::Node;
use zakac_diagnostics::kinds as kd;
use zakac_diagnostics::resolve as rd;
use zakac_rust_sugars::ret_if;

use crate::kind::DataKind;
use crate::list::KindUse;
use crate::ops::OpClass;
use crate::semcheck::Checker;
use crate::value::Value;

/// What a switch case is known to match, for the duplicate and coverage
/// checks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum CaseHit
{
    /// The member at a position of the enumeration of the subject.
    Member(usize),
    /// A folded value of another kind.
    Value(Value),
}

impl Checker
{
    /// Check the case of a switch arm against the subject, as `==` checks
    /// its two sides, and report a case whose value an arm before it has.
    /// With no subject kind, the case is not checked.
    ///
    /// # Arguments
    ///
    /// * `case` - The case.
    /// * `subject` - The subject and its kind, when known.
    /// * `seen` - What the cases before it match.
    pub(crate) fn check_case(
        &mut self, case: &Expr, subject: (&Expr, Option<DataKind>),
        seen: &mut Vec<CaseHit>,
    )
    {
        // with no subject kind, a bare member name would read as an
        // undeclared variable, so the cases wait for the subject's fix
        let (subject_expr, Some(subject)) = subject
        else
        {
            return;
        };
        let hit = match self.member_case(case, subject)
        {
            Some(member) => member,
            None => self.plain_case(case, (subject_expr, subject)),
        };
        let Some(hit) = hit
        else
        {
            return;
        };
        ret_if!(seen.contains(&hit), self.report(kd::duplicate_case(case.span())));
        seen.push(hit);
    }

    /// Check a case that is no bare member name: an expression that `==`
    /// takes with the subject.
    ///
    /// # Arguments
    ///
    /// * `case` - The case.
    /// * `subject` - The subject and its kind.
    ///
    /// # Returns
    ///
    /// What the case matches, when it is a member or its value folds.
    fn plain_case(&mut self, case: &Expr, subject: (&Expr, DataKind))
    -> Option<CaseHit>
    {
        let kind = self.infer(case, Some(subject.1))?;
        let (lhs, rhs) = ((subject.1, subject.0), (kind, case));
        self.pair_result(OpClass::Equal, "==", lhs, rhs, case.span())?;
        let Some(owner) = subject.1.as_enum()
        else
        {
            return self.work_out(case).ok().map(CaseHit::Value);
        };
        let Expr::Member(m) = case
        else
        {
            let v = self.work_out(case).ok()?;
            return owner.find(|mv| mv.scalar() == Some(v)).map(CaseHit::Member);
        };
        owner.position(&m.member.name).map(CaseHit::Member)
    }

    /// A case written as a bare member name, over a subject of an
    /// enumeration: the member comes before a variable of that name.
    ///
    /// # Arguments
    ///
    /// * `case` - The case.
    /// * `subject` - The kind of the subject.
    ///
    /// # Returns
    ///
    /// `None` for a case that is no bare member name; else the member,
    /// `None` inside after the report of a name that is neither a member nor
    /// a variable.
    fn member_case(&mut self, case: &Expr, subject: DataKind)
    -> Option<Option<CaseHit>>
    {
        let (Expr::Name(n), Some(owner)) = (case, subject.as_enum())
        else
        {
            return None;
        };
        if let Some(at) = owner.position(&n.name)
        {
            self.kinds.insert(case.id(), subject);
            return Some(Some(CaseHit::Member(at)));
        }
        ret_if!(self.scopes.lookup(&n.name).is_some(), None);
        self.report(rd::unknown_member(n.span, &owner.name, &n.name));
        Some(None)
    }

    /// Check the default arm of a switch against its subject, and that a
    /// switch over an enumeration with no default arm names each member.
    ///
    /// # Arguments
    ///
    /// * `n` - The switch.
    /// * `subject` - The kind of the subject, when known.
    /// * `seen` - What the cases match.
    pub(crate) fn check_default(
        &mut self, n: &SwitchStmt, subject: Option<DataKind>, seen: &[CaseHit],
    )
    {
        let Some(subject) = subject
        else
        {
            return;
        };
        if let Some(d) = &n.default
        {
            let written = self.written_kind(&d.kind, KindUse::Other);
            ret_if!(written == subject);
            let (w, s) = (written.name(), subject.name());
            self.report(kd::default_kind(d.kind.span, &w, &s));
            return;
        }
        let Some(owner) = subject.as_enum()
        else
        {
            return;
        };
        let missing: Vec<String> = owner
            .members()
            .iter()
            .enumerate()
            .filter(|(at, _)| !seen.contains(&CaseHit::Member(*at)))
            .map(|(_, (m, _))| format!("\"{m}\""))
            .collect();
        ret_if!(missing.is_empty());
        let span = n.subject.span();
        self.report(kd::switch_missing(span, &missing.join(", "), owner.stem()));
    }
}
