use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::VariantExpr;
use zakac_ast::name::Name;
use zakac_ast::stmt::SwitchStmt;
use zakac_ast::visit::Node;
use zakac_diagnostics::kinds as kd;
use zakac_diagnostics::resolve as rd;
use zakac_lexer::reserved::PLURAL;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use crate::enums::stem_of;
use crate::kind::DataKind;
use crate::list::KindUse;
use crate::ops::OpClass;
use crate::proof::result_var;
use crate::semcheck::Checker;
use crate::value::Value;
use crate::variants::Variants;

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
        let hit = match (case, subject.as_variants())
        {
            (Expr::Variant(v), _) => self.variant_case(v, subject),
            (Expr::Name(n), None) if subject.is_trait_value() =>
            {
                let name = Ident::builder().name(n.name).span(n.span).build();
                self.variant_at(subject, &name, n.span).map(|at| CaseHit::Member(at.at))
            }
            (_, Some(owner)) => self.data_case(case, owner),
            _ => match self.member_case(case, subject)
            {
                Some(member) => member,
                None => self.plain_case(case, (subject_expr, subject)),
            },
        };
        let Some(hit) = hit
        else
        {
            return;
        };
        ret_if!(seen.contains(&hit), self.report(kd::duplicate_case(case.span())));
        seen.push(hit);
    }

    /// Check a case written as the test of a variant,
    /// `[<name> ဟူသည့်] <variant> အဖြစ်သို့ ပြောင်းလဲနိုင်လျှင်`.
    ///
    /// # Arguments
    ///
    /// * `v` - The variant.
    /// * `subject` - The kind of the subject.
    ///
    /// # Returns
    ///
    /// The variant the case matches, or `None` after a report.
    fn variant_case(&mut self, v: &VariantExpr, subject: DataKind) -> Option<CaseHit>
    {
        let at = self.variant_at(subject, &v.variant, v.span)?;
        let bind = v.bind.as_ref().filter(|_| at.data.is_none());
        do_if!(let Some(b) = bind, {
            let written = at.written(&v.variant.name);
            self.report(rd::variant_has_no_data(b.span, &at.owner, &v.variant.name, &written));
        });
        self.kinds.insert(v.id, subject);
        Some(CaseHit::Member(at.at))
    }

    /// Check a case of a switch over an enumeration whose variants hold
    /// data: a variant, as a bare name or as `<stem>များထဲမှ <variant>`, with
    /// no data.
    ///
    /// # Arguments
    ///
    /// * `case` - The case.
    /// * `owner` - The enumeration.
    ///
    /// # Returns
    ///
    /// The variant the case matches, or `None` after a report.
    fn data_case(&mut self, case: &Expr, owner: Variants) -> Option<CaseHit>
    {
        let (variant, qualifier) = match case
        {
            Expr::Name(n) => (Ident::builder().name(n.name).span(n.span).build(), None),
            Expr::Member(m) if m.data.is_none() =>
            {
                (m.member.clone(), Some(&m.qualifier))
            }
            _ =>
            {
                self.report(rd::case_with_data(case.span(), owner.name()));
                return None;
            }
        };
        let stem = format!("{}{PLURAL}", owner.stem());
        // Over an instance of a generic enumeration, the subject gives the
        // kinds, so a case can name the generic with none.
        let generic = self
            .known_structs
            .get(&Name::new(owner.name()))
            .map(|(t, _)| format!("{}{PLURAL}", stem_of(t)));
        let named = |q: &&Ident| {
            q.name == stem.as_str() || generic.as_deref() == Some(q.name.as_str())
        };
        let wrong = qualifier.filter(|q| !named(q));
        ret_if!(let Some(q) = wrong, self.report(rd::unknown_enum(q.span, &q.name, Some(&stem))), None);
        let at = self.variant_at(owner.kind(), &variant, case.span())?;
        self.kinds.insert(case.id(), owner.kind());
        Some(CaseHit::Member(at.at))
    }

    /// Whether a kind is the bare name of the generic that a kind is an
    /// instance of.
    ///
    /// # Arguments
    ///
    /// * `written` - The kind as written.
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// `true` when it is.
    fn names_template_of(&self, written: &zakac_ast::kind::Kind, kind: DataKind)
    -> bool
    {
        let Some(name) = written.name.as_ref().filter(|_| written.args.is_empty())
        else
        {
            return false;
        };
        let instance = kind.as_struct().map(|s| Name::new(&s.name));
        let of = instance.and_then(|i| self.known_structs.get(&i)).map(|(t, _)| *t);
        of.is_some_and(|t| t == name.name || Name::from(stem_of(&t)) == name.name)
    }

    /// The name an arm gives the data of its variant, with the kind of the
    /// data.
    ///
    /// # Arguments
    ///
    /// * `subject` - The kind of the subject, when known.
    /// * `cases` - The cases of the arm.
    ///
    /// # Returns
    ///
    /// The name and the kind, or `None` when the arm names no data.
    pub(crate) fn arm_bind(
        &self, subject: Option<DataKind>, cases: &[Expr],
    ) -> Option<(Ident, DataKind)>
    {
        let [Expr::Variant(v)] = cases
        else
        {
            return None;
        };
        let bind = v.bind.clone()?;
        Some((bind, self.data_of(subject?, &v.variant.name)?))
    }

    /// The variant an arm with one case proves its subject variable holds.
    ///
    /// # Arguments
    ///
    /// * `subject` - The subject.
    /// * `kind` - Its kind, when known.
    /// * `cases` - The cases of the arm.
    ///
    /// # Returns
    ///
    /// The variable and the variant, or `None`.
    pub(crate) fn arm_fact(
        &self, subject: &Expr, kind: Option<DataKind>, cases: &[Expr],
    ) -> Option<(Name, Name)>
    {
        let var = result_var(subject)?;
        let owner = kind?.as_variants()?;
        let name = match cases
        {
            [Expr::Variant(v)] => v.variant.name,
            [Expr::Name(n)] => n.name,
            [Expr::Member(m)] => m.member.name,
            _ => return None,
        };
        owner.position(&name).map(|_| (var, name))
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
            let member = self.decls.member(&owner.name, &n.name);
            do_if!(let Some(d) = member, self.refer(&n.name, n.span, d));
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
            // The default arm of an instance of a generic enumeration can name
            // the generic alone.
            ret_if!(self.names_template_of(&d.kind, subject));
            let written = self.written_kind(&d.kind, KindUse::Other);
            ret_if!(written == subject);
            let (w, s) = (written.name(), subject.name());
            self.report(kd::default_kind(d.kind.span, &w, &s));
            return;
        }
        let listed = match (subject.as_enum(), subject.as_variants())
        {
            (Some(e), _) => (
                e.members().iter().map(|(m, _)| m.clone()).collect(),
                e.name.clone(),
                e.stem().to_string(),
            ),
            (_, Some(v)) => (
                v.names().iter().map(|m| m.to_string()).collect(),
                v.name().to_string(),
                v.stem().to_string(),
            ),
            (None, None) => return,
        };
        let (members, name, stem): (Vec<String>, String, String) = listed;
        let missing: Vec<String> = members
            .iter()
            .enumerate()
            .filter(|(at, _)| !seen.contains(&CaseHit::Member(*at)))
            .map(|(_, m)| format!("\"{m}\""))
            .collect();
        ret_if!(missing.is_empty(), _ = self.full_switches.insert(n.span));
        let span = n.subject.span();
        let word = self.classifiers.get(&name).map_or("ခု", String::as_str);
        let help = kd::switch_missing(span, &missing.join(", "), &stem, word);
        self.report(help);
    }
}
