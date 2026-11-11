//! The check of kinds no program can build: with no NULL, a struct whose
//! fields lead back to it needs a value of itself before its first value, so
//! the first one never comes.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;

use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Cells;
use zakac_ast::kind::Kind;
use zakac_ast::kind::Wrap;
use zakac_ast::stmt::StructDecl;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::kinds as kd;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

/// The struct names each struct needs a value of, one list per struct.
type Needs<'a> = BTreeMap<&'a str, Vec<&'a str>>;

/// Report each loop of structs that no program can build. A loop of plain
/// structs alone holds itself inline, which `struct_holds_itself` reports,
/// so a loop is reported here when a shared struct is on it. A struct that
/// holds a struct of such a loop gets no report of its own: fixing the loop
/// fixes it.
///
/// # Arguments
///
/// * `decls` - The struct declarations, each name once.
/// * `diags` - Where to report.
pub(crate) fn report_unbuildable(
    decls: &BTreeMap<&str, &StructDecl>, diags: &mut Vec<Diagnostic>,
)
{
    let needs: Needs = decls.iter().map(|(n, d)| (*n, needs_of(d, decls))).collect();
    let built = buildable(&needs);
    let mut left: Vec<&str> =
        decls.keys().copied().filter(|n| !built.contains(n)).collect();
    left.sort_by_key(|n| decls[n].span.lo);
    let mut reported = BTreeSet::new();
    for name in left
    {
        let found =
            loop_from(name, &needs, &built).filter(|_| !reported.contains(name));
        let Some(chain) = found.filter(|c| c.iter().any(|n| decls[n].shared))
        else
        {
            continue;
        };
        let text = format!("{} → {name}", chain.join(" → "));
        diags.push(kd::kind_never_built(decls[name].name.span, name, &text));
        reported.extend(chain);
    }
}

/// The structs one struct needs a value of: the struct of each field that
/// is required. C builds a kind from outside, so it needs none.
///
/// # Arguments
///
/// * `decl` - The declaration.
/// * `decls` - The struct declarations.
///
/// # Returns
///
/// The struct names.
fn needs_of<'a>(
    decl: &'a StructDecl, decls: &BTreeMap<&str, &StructDecl>,
) -> Vec<&'a str>
{
    ret_if!(decl.external, Vec::new());
    decl.fields.iter().filter_map(|f| required(&f.kind, decls)).collect()
}

/// The struct a field needs a value of before its own struct can be built.
/// An optional, a result, a function value, a pointer and a plain list each
/// have a value with no struct in it, so none of them needs one.
///
/// # Arguments
///
/// * `kind` - The field kind phrase.
/// * `decls` - The struct declarations.
///
/// # Returns
///
/// The struct name, or `None`.
fn required<'a>(kind: &'a Kind, decls: &BTreeMap<&str, &StructDecl>)
-> Option<&'a str>
{
    ret_if!(kind.parts.is_some() || kind.sig.is_some(), None);
    let ends = |w: &Wrap| matches!(w, Wrap::Ptr | Wrap::List(Cells::Inferred));
    ret_if!(kind.base != BaseKind::Named || kind.wraps.iter().any(ends), None);
    let name = kind.name.as_ref()?.name.as_str();
    decls.contains_key(name).then_some(name)
}

/// The structs a program can build: the least set that holds each struct
/// whose needed structs it holds.
///
/// # Arguments
///
/// * `needs` - The structs each struct needs.
///
/// # Returns
///
/// The names.
fn buildable<'a>(needs: &Needs<'a>) -> BTreeSet<&'a str>
{
    let mut built = BTreeSet::new();
    loop
    {
        let fits = |(n, ns): &(&&'a str, &Vec<&'a str>)| {
            !built.contains(**n) && ns.iter().all(|x| built.contains(x))
        };
        let fresh: Vec<&str> = needs.iter().filter(fits).map(|(n, _)| *n).collect();
        ret_if!(fresh.is_empty(), built);
        built.extend(fresh);
    }
}

/// The shortest loop from a struct back to itself through structs that
/// cannot be built.
///
/// # Arguments
///
/// * `start` - The struct.
/// * `needs` - The structs each struct needs.
/// * `built` - The structs a program can build.
///
/// # Returns
///
/// The structs of the loop in order, `start` first, or `None` when no loop
/// comes back to it.
fn loop_from<'a>(
    start: &'a str, needs: &Needs<'a>, built: &BTreeSet<&'a str>,
) -> Option<Vec<&'a str>>
{
    let mut before: BTreeMap<&str, &str> = BTreeMap::new();
    let mut todo = VecDeque::from([start]);
    while let Some(at) = todo.pop_front()
    {
        let next: Vec<&str> =
            needs[at].iter().copied().filter(|n| !built.contains(n)).collect();
        ret_if!(next.contains(&start), Some(path_to(at, start, &before)));
        for n in next
        {
            let fresh = before.insert(n, at).is_none();
            do_if!(fresh, todo.push_back(n));
        }
    }
    None
}

/// The path the search took from `start` to `end`.
///
/// # Arguments
///
/// * `end` - The last struct.
/// * `start` - The first struct.
/// * `before` - The struct each struct was reached from.
///
/// # Returns
///
/// The structs in order, `start` first.
fn path_to<'a>(
    end: &'a str, start: &'a str, before: &BTreeMap<&str, &'a str>,
) -> Vec<&'a str>
{
    let mut path = vec![end];
    let mut at = end;
    while at != start
    {
        at = before[at];
        path.push(at);
    }
    path.reverse();
    path
}
