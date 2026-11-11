//! Give each overloaded function the name the later stages use for it. One
//! file can declare several functions of one name; the first keeps its name
//! and the n-th becomes `<name>#<n>`. Once the check has chosen the function
//! of each call, the program is written over: each declaration and each call
//! takes the inner name, and an import of a name lists the inner names of the
//! other functions of that name, so the IR builder, the checker and
//! codegen see one function for each name.

use std::collections::HashMap;

use zakac_ast::expr::Ident;
use zakac_ast::name::Name;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::Stmt;
use zakac_ast::visit::VisitorMut;
use zakac_rust_sugars::do_if;
use zakac_span::span::Span;

use crate::scope::overload_name;

/// The inner name of the function each call of an overloaded name chose,
/// under the span of the name at the call.
pub(crate) type Chosen = HashMap<Span, Name>;

/// Write the inner names of overloaded functions into the program.
///
/// # Arguments
///
/// * `program` - The program, checked with no error.
/// * `chosen` - The function each call of an overloaded name chose.
pub(crate) fn write_overloads(program: &mut Program, chosen: &Chosen)
{
    let (decls, extra) = overloaded_decls(program);
    let mut names = chosen.clone();
    names.extend(decls);
    do_if!(!names.is_empty(), Renamer(&names).visit_program_mut(program));
    add_to_imports(program, &extra);
}

/// The inner name of each declaration after the first of a name in its file,
/// under the span of its name, and for each file the inner names each written
/// name gained.
///
/// # Arguments
///
/// * `program` - The program.
///
/// # Returns
///
/// The renames, and the gained names of each unit.
fn overloaded_decls(program: &Program) -> (Chosen, Vec<HashMap<Name, Vec<Name>>>)
{
    let mut renames = Chosen::new();
    let mut gained =
        vec![HashMap::<Name, Vec<Name>>::new(); program.units.len().max(1)];
    let mut seen: HashMap<(u32, Name), usize> = HashMap::new();
    for (i, s) in program.stmts.iter().enumerate()
    {
        let Stmt::Fn(decl) = s
        else
        {
            continue;
        };
        let unit = program.unit_of(i);
        let index = seen.entry((unit, decl.name.name)).or_default();
        let inner = overload_name(decl.name.name, *index);
        *index += 1;
        if inner != decl.name.name
        {
            renames.insert(decl.name.span, inner);
            gained[unit as usize].entry(decl.name.name).or_default().push(inner);
        }
    }
    (renames, gained)
}

/// Add to each import of an overloaded name the inner names of its other
/// functions, so the importing file reaches each of them.
///
/// # Arguments
///
/// * `program` - The program.
/// * `gained` - The inner names each written name gained, for each unit.
fn add_to_imports(program: &mut Program, gained: &[HashMap<Name, Vec<Name>>])
{
    for s in &mut program.stmts
    {
        let Stmt::Import(n) = s
        else
        {
            continue;
        };
        let Some(names) = n.unit.and_then(|u| gained.get(u as usize))
        else
        {
            continue;
        };
        let more = n.names.iter().filter_map(|i| Some((i.span, names.get(&i.name)?)));
        let more: Vec<Ident> = more
            .flat_map(|(span, inner)| {
                inner
                    .iter()
                    .map(move |name| Ident::builder().name(*name).span(span).build())
            })
            .collect();
        n.names.extend(more);
    }
}

/// The walker that gives a name its inner name, found from its span.
struct Renamer<'a>(&'a Chosen);

impl VisitorMut for Renamer<'_>
{
    fn visit_ident_mut(&mut self, n: &mut Ident)
    {
        do_if!(let Some(inner) = self.0.get(&n.span), n.name = *inner);
    }
}
