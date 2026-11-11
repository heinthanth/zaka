use std::collections::BTreeSet;
use std::collections::HashMap;

use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::name::Name;
use zakac_ast::stmt::EnumDecl;
use zakac_ast::stmt::EnumMember;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::Stmt;
use zakac_ast::visit::Node;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::kinds as kd;
use zakac_diagnostics::resolve as rd;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::const_op::InfixOp;
use crate::consts::Folded;
use crate::consts::Stop;
use crate::enums::EnumKind;
use crate::enums::MemberValue;
use crate::enums::make;
use crate::enums::stem_of;
use crate::kind::DataKind;
use crate::semcheck::Checker;
use crate::structs::StructTable;
use crate::value;
use crate::value::Value;

/// The value kind and the member values of each enumeration that a run
/// of the check worked out, on the enumeration name.
pub(crate) type KnownEnums = HashMap<Name, (DataKind, Vec<MemberValue>)>;

/// Make the kinds of the top-level enumerations, reporting a name that a
/// struct or another enumeration has and a member name used twice.
///
/// A member value can name a constant, so the walk of the check works the
/// values out; until a run has, an enumeration with a value written stands
/// as `isize` members numbered from 0, and the check runs again.
///
/// # Arguments
///
/// * `program` - The tree.
/// * `known` - The values an earlier run worked out.
/// * `diags` - Where to report.
///
/// # Returns
///
/// The kind of each enumeration, on its name and on its stem; the first
/// declaration of a name wins.
pub(crate) fn collect_enums(
    program: &Program, known: &KnownEnums, diags: &mut Vec<Diagnostic>,
) -> StructTable
{
    let structs: BTreeSet<&str> = program
        .stmts
        .iter()
        .filter_map(
            |s| tern!(let Stmt::Struct(d) = s, Some(d.name.name.as_str()), None),
        )
        .collect();
    let mut table = StructTable::new();
    for decl in program.stmts.iter().filter_map(as_enum)
    {
        let kind = DataKind::Enum(enum_kind(decl, known, diags));
        let name = decl.name.name.as_str();
        let stem = Some(stem_of(name)).filter(|s| *s != name);
        for word in std::iter::once(name).chain(stem)
        {
            let taken = structs.contains(word) || table.contains_key(&Name::new(word));
            do_if!(taken, diags.push(rd::duplicate_struct(decl.name.span, word)));
            do_if!(!taken, _ = table.insert(Name::new(word), kind));
        }
    }
    table
}

/// The declaration of a top-level enumeration statement.
///
/// # Arguments
///
/// * `stmt` - The statement.
///
/// # Returns
///
/// The declaration, or `None` for another statement.
fn as_enum(stmt: &Stmt) -> Option<&EnumDecl>
{
    tern!(let Stmt::Enum(decl) = stmt, Some(decl), None)
}

/// The members of a declaration, each name once: a member name used twice is
/// reported, and its second member left out.
///
/// # Arguments
///
/// * `decl` - The declaration.
/// * `diags` - Where to report, when given.
///
/// # Returns
///
/// The members kept, each with its position in the declaration.
fn distinct_members<'d>(
    decl: &'d EnumDecl, mut diags: Option<&mut Vec<Diagnostic>>,
) -> Vec<(usize, &'d Ident)>
{
    let mut kept: Vec<(usize, &Ident)> = Vec::new();
    for (at, m) in decl.members.iter().enumerate()
    {
        let twice = kept.iter().any(|(_, n)| n.name == m.name.name);
        let owner = decl.name.name.as_str();
        let report = |d: &mut Vec<Diagnostic>| {
            d.push(rd::duplicate_member(m.name.span, owner, &m.name.name));
        };
        do_if!(twice, diags.as_deref_mut().map(report).unwrap_or_default());
        do_if!(!twice, kept.push((at, &m.name)));
    }
    kept
}

/// The kind of one enumeration: the values an earlier run worked out, or,
/// with none, `isize` members numbered from 0.
///
/// # Arguments
///
/// * `decl` - The declaration.
/// * `known` - The values an earlier run worked out.
/// * `diags` - Where to report.
///
/// # Returns
///
/// The kind.
fn enum_kind(
    decl: &EnumDecl, known: &KnownEnums, diags: &mut Vec<Diagnostic>,
) -> &'static EnumKind
{
    let kept = distinct_members(decl, Some(diags));
    let initial = initial_member(decl, &kept, diags);
    let names: Vec<String> = kept.iter().map(|(_, n)| n.name.to_string()).collect();
    let found = known.get(&decl.name.name).filter(|(_, v)| v.len() == names.len());
    if let Some((under, values)) = found
    {
        let members = names.into_iter().zip(values.clone()).collect();
        return make(&decl.name.name, *under, members, initial);
    }
    let numbered =
        (0..names.len()).map(|i| MemberValue::Scalar(isize_value(i as i128)));
    let members = names.into_iter().zip(numbered).collect();
    make(&decl.name.name, DataKind::ISIZE, members, initial)
}

/// The member marked `ကနဦးတန်ဖိုး`, the initial value of the enumeration. A
/// second mark is reported.
///
/// # Arguments
///
/// * `decl` - The declaration.
/// * `kept` - The members kept, each with its position in the declaration.
/// * `diags` - Where to report.
///
/// # Returns
///
/// The position of the marked member among those kept, or `None`.
fn initial_member(
    decl: &EnumDecl, kept: &[(usize, &Ident)], diags: &mut Vec<Diagnostic>,
) -> Option<usize>
{
    let mut marked = decl.members.iter().filter(|m| m.initial);
    let first = marked.next()?;
    let owner = decl.name.name.as_str();
    let twice =
        |m: &EnumMember| kd::two_initial_members(m.span, owner, &first.name.name);
    diags.extend(marked.map(twice));
    kept.iter().position(|(at, _)| decl.members[*at].initial)
}

/// An `isize` value.
///
/// # Arguments
///
/// * `v` - The number.
///
/// # Returns
///
/// The value.
fn isize_value(v: i128) -> Value
{
    let kind = DataKind::ISIZE.int_kind().expect("isize is an integer");
    Value::Int { kind, raw: v as u128 }
}

impl Checker
{
    /// Work out the value kind and the value of each member of an
    /// enumeration with a value written, and keep them for the next run.
    ///
    /// The value of the first member gives the value kind, `isize` when
    /// it has none written; each other value converts to that kind. A member
    /// with no value takes the value of the member before it plus one, the
    /// first 0, for an integer kind; another kind needs each value written.
    ///
    /// # Arguments
    ///
    /// * `n` - The declaration.
    pub(crate) fn check_enum(&mut self, n: &EnumDecl)
    {
        ret_if!(n.members.iter().all(|m| m.value.is_none()));
        let first = n.members[0].value.as_ref();
        let Some(under) = first.map_or(Some(DataKind::ISIZE), |e| self.infer(e, None))
        else
        {
            return;
        };
        let mut values: Vec<(&Ident, MemberValue)> = Vec::new();
        let mut complete = true;
        for (at, name) in distinct_members(n, None)
        {
            let written = n.members[at].value.as_ref();
            let value = match written
            {
                Some(e) => self.member_value(e, under, name, at == 0),
                None => self.count_on(values.last().map(|(_, v)| v), under, name),
            };
            let Some(value) = value
            else
            {
                complete = false;
                continue;
            };
            let first = values.iter().find(|(_, v)| *v == value).map(|(m, _)| *m);
            do_if!(let Some(first) = first, {
                let span = n.members[at].span;
                self.report(kd::duplicate_member_value(span, &name.name, &first.name));
            });
            values.push((name, value));
        }
        ret_if!(!complete);
        let values = values.into_iter().map(|(_, v)| v).collect();
        _ = self.found_enums.insert(n.name.name, (under, values));
    }

    /// The value of a member with a value written: a constant of the
    /// value kind.
    ///
    /// # Arguments
    ///
    /// * `e` - The value.
    /// * `under` - The value kind.
    /// * `name` - The member name.
    /// * `first` - Whether it is the first member, whose value gave the kind.
    ///
    /// # Returns
    ///
    /// The value, or `None` when it waits for a list count or does not work
    /// out; the second is reported.
    fn member_value(
        &mut self, e: &Expr, under: DataKind, name: &Ident, first: bool,
    ) -> Option<MemberValue>
    {
        let before = self.diags.len();
        do_if!(!first, self.expect_kind(e, Some(under)));
        // a value of the wrong kind is reported once, as a kind mismatch
        ret_if!(self.diags.len() > before, None);
        match self.constant_of(e, under)
        {
            Ok(v) => Some(v),
            Err(Stop::Pending) => None,
            Err(Stop::Not(_)) =>
            {
                self.report(kd::member_value(e.span(), &name.name));
                None
            }
        }
    }

    /// The constant value of an expression in a kind: a scalar folded and
    /// converted to the kind, and text, a member, or the cells or fields of a
    /// list or a struct as the folder works them out.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// The value, or the reason it has none.
    fn constant_of(&self, e: &Expr, kind: DataKind) -> Result<MemberValue, Stop>
    {
        let scalar = !kind.is_aggregate() && kind.as_enum().is_none();
        ret_if!(scalar, self.coerced(e, kind).map(MemberValue::Scalar));
        items_of(self.folded(e)?)
    }

    /// The value of a member with no value written: the value of the member
    /// before it plus one, or 0 for the first.
    ///
    /// # Arguments
    ///
    /// * `prev` - The value of the member before it.
    /// * `under` - The value kind.
    /// * `name` - The member name.
    ///
    /// # Returns
    ///
    /// The value, or `None` after a report: the kind is no integer, or the
    /// value passes the largest of the kind.
    fn count_on(
        &mut self, prev: Option<&MemberValue>, under: DataKind, name: &Ident,
    ) -> Option<MemberValue>
    {
        let int = under.repr().int_kind();
        let next = match (int, prev.and_then(MemberValue::scalar))
        {
            (Some(k), None) => Some(Value::Int { kind: k, raw: 0 }),
            (Some(k), Some(p)) =>
            {
                let one = Value::Int { kind: k, raw: 1 };
                value::infix(InfixOp::Add, p, one, self.platform).ok()
            }
            (None, _) => None,
        };
        do_if!(
            next.is_none(),
            self.report(kd::member_needs_value(name.span, &name.name, &under.name()))
        );
        next.map(MemberValue::Scalar)
    }
}

/// The member value of the cells or fields of a folded list or struct.
///
/// # Arguments
///
/// * `folded` - The folded value.
///
/// # Returns
///
/// The value, or none for a value the folder knows as a whole alone.
fn items_of(folded: Folded) -> Result<MemberValue, Stop>
{
    match folded
    {
        Folded::Value(v) => Ok(MemberValue::Scalar(v)),
        Folded::Items(items) =>
        {
            let items: Result<Vec<_>, _> = items.into_iter().map(items_of).collect();
            items.map(MemberValue::Items)
        }
        Folded::Text(t) => Ok(MemberValue::Text(t)),
        Folded::Whole => Err(Stop::Not(None)),
    }
}
