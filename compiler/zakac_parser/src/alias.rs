//! Kind aliases. An alias is a second name for a kind, so the loader puts the
//! kind in place of each use of the name: in a kind, as the name of a struct
//! literal, and before `ထဲမှ` in a member of an enumeration. The stages after
//! the loader see no alias.

use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;

use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Cells;
use zakac_ast::kind::Kind;
use zakac_ast::kind::Wrap;
use zakac_ast::name::Name;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::Stmt;
use zakac_ast::visit::VisitorMut;
use zakac_ast::visit::walk_expr_mut;
use zakac_ast::visit::walk_sig_mut;
use zakac_ast::visit::walk_table_mut;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::resolve as rd;
use zakac_lexer::reserved::PLURAL;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

/// Each alias under its name, and under its stem when the name ends in
/// `များ`, with the kind it names as written, the name in its declaration and
/// its kind parameters.
type Aliases = HashMap<Name, (Kind, Ident, Vec<Name>)>;

/// Each alias name the expansion met, where it stands, beside the name in
/// its declaration; each declared alias name stands beside itself too. The
/// editors read it, as the check never sees an alias name.
pub type AliasRefs = Vec<(Span, Span)>;

/// Put the kind of each alias in place of each use of its name.
///
/// # Arguments
///
/// * `program` - The program, with the statements of each file.
///
/// # Returns
///
/// A `ZK0316` diagnostic for each alias whose name a struct, an enumeration
/// or another alias has, and a `ZK0336` diagnostic for each alias that leads
/// back to itself; and each alias name met, beside its declaration.
pub fn expand_aliases(program: &mut Program) -> (Vec<Diagnostic>, AliasRefs)
{
    let mut diags = Vec::new();
    let aliases = collect(program, &mut diags);
    ret_if!(aliases.is_empty(), (diags, AliasRefs::new()));
    let own = aliases.values().map(|(_, decl, _)| (decl.span, decl.span)).collect();
    #[rustfmt::skip]
    let mut swap = Swap { aliases: &aliases, trail: Vec::new(), looped: BTreeSet::new(),
        refs: own, diags: Vec::new() };
    swap.visit_program_mut(program);
    let refs = std::mem::take(&mut swap.refs);
    diags.append(&mut swap.diags);
    let looped = swap.looped.into_iter().filter_map(|name| aliases.get(&name));
    diags.extend(
        looped.map(|(_, ident, _)| rd::alias_loop(ident.span, ident.name.as_str())),
    );
    diags.sort_by_key(|d| d.span.lo);
    (diags, refs)
}

/// The aliases of a program; the first declaration of a name wins.
///
/// # Arguments
///
/// * `program` - The program.
/// * `diags` - Where to report a name that is taken.
///
/// # Returns
///
/// The aliases.
fn collect(program: &Program, diags: &mut Vec<Diagnostic>) -> Aliases
{
    let mut taken: HashSet<Name> = program.stmts.iter().flat_map(kind_names).collect();
    let mut aliases = Aliases::new();
    for stmt in &program.stmts
    {
        let Stmt::Alias(decl) = stmt
        else
        {
            continue;
        };
        let name = decl.name.name;
        if taken.contains(&name)
        {
            diags.push(rd::duplicate_struct(decl.name.span, name.as_str()));
            continue;
        }
        let params = decl.kind_params.iter().map(|p| p.name.name).collect();
        let pair = (decl.kind.clone(), decl.name.clone(), params);
        do_for!(n in with_stem(name), {
            taken.insert(n);
            aliases.insert(n, pair.clone());
        });
    }
    aliases
}

/// The kind names a struct or an enumeration declaration claims.
///
/// # Arguments
///
/// * `stmt` - The statement.
///
/// # Returns
///
/// The names; none for another statement.
fn kind_names(stmt: &Stmt) -> Vec<Name>
{
    match stmt
    {
        Stmt::Struct(d) => vec![d.name.name],
        Stmt::Enum(d) => with_stem(d.name.name),
        _ => Vec::new(),
    }
}

/// A name, and its stem when it ends in `များ`.
///
/// # Arguments
///
/// * `name` - The name.
///
/// # Returns
///
/// The name, then the stem.
fn with_stem(name: Name) -> Vec<Name>
{
    let stem = name.as_str().strip_suffix(PLURAL).filter(|s| !s.is_empty());
    std::iter::once(name).chain(stem.map(Name::new)).collect()
}

/// Walks the program and puts each alias kind in place.
struct Swap<'a>
{
    /// The aliases.
    aliases: &'a Aliases,
    /// The aliases being expanded, outermost first, to find a loop.
    trail: Vec<Name>,
    /// The aliases that lead back to themselves.
    looped: BTreeSet<Name>,
    /// Each alias name met, beside its declaration.
    refs: AliasRefs,
    /// The uses of a generic alias with another count of kinds, ZK0340.
    diags: Vec<Diagnostic>,
}

impl Swap<'_>
{
    /// Record an alias name where it stands, beside its declaration.
    ///
    /// # Arguments
    ///
    /// * `used` - The name, when it is an alias.
    fn note(&mut self, used: &Ident)
    {
        let decl = self.aliases.get(&used.name).map(|(_, d, _)| d.span);
        do_if!(let Some(decl) = decl, self.refs.push((used.span, decl)));
    }

    /// The kind an alias names, with each alias in it expanded too.
    ///
    /// # Arguments
    ///
    /// * `name` - The alias.
    ///
    /// # Returns
    ///
    /// The kind, or `None` for no alias or an alias that leads back to
    /// itself.
    fn expanded(&mut self, name: Name) -> Option<Kind>
    {
        let (kind, ident, _) = self.aliases.get(&name)?;
        let first = ident.name;
        if let Some(at) = self.trail.iter().position(|n| *n == first)
        {
            // each alias of the loop leads back to itself
            self.looped.extend(self.trail[at..].iter().copied());
            return None;
        }
        self.trail.push(first);
        let mut kind = kind.clone();
        self.visit_kind_mut(&mut kind);
        self.trail.pop();
        // a loop leaves the alias name in place, and semcheck must not see it
        ret_if!(self.looped.contains(&first), None);
        Some(kind)
    }

    /// The struct or enumeration name an alias stands for, when it names one
    /// with no levels around it.
    ///
    /// # Arguments
    ///
    /// * `name` - The alias.
    ///
    /// # Returns
    ///
    /// The name, or `None`.
    fn bare_target(&mut self, name: Name) -> Option<Name>
    {
        let kind = self.expanded(name)?;
        let bare = kind.base == BaseKind::Named && kind.wraps.is_empty();
        bare.then_some(kind.name?.name)
    }

    /// The kind of a generic alias with the kinds of its use in place of its
    /// kind parameters. Another count of kinds than the alias declares is
    /// ZK0340.
    ///
    /// # Arguments
    ///
    /// * `target` - The kind the alias names.
    /// * `used` - The kind as written at the use, with its kinds.
    ///
    /// # Returns
    ///
    /// The kind, or `None` after a report.
    fn with_args(&mut self, mut target: Kind, used: &Kind) -> Option<Kind>
    {
        let ident = used.name.as_ref()?;
        let (_, _, params) = self.aliases.get(&ident.name)?;
        let (want, got) = (params.len(), used.args.len());
        let count = rd::kind_arg_count(ident.span, &ident.name, want, got);
        ret_if!(want != got, self.diags.push(count), None);
        let bound: HashMap<Name, Kind> =
            params.iter().copied().zip(used.args.iter().cloned()).collect();
        target.substitute(&bound);
        Some(target)
    }

    /// Put the enumeration an alias stands for before `ထဲမှ`:
    /// `<alias>များထဲမှ <member>`.
    ///
    /// # Arguments
    ///
    /// * `qualifier` - The name before `ထဲမှ`.
    fn swap_qualifier(&mut self, qualifier: &mut Ident)
    {
        let text = qualifier.name.as_str();
        let stem = text.strip_suffix(PLURAL).map(Name::new);
        let alias = [Some(qualifier.name), stem]
            .into_iter()
            .flatten()
            .find(|n| self.aliases.contains_key(n));
        let Some(target) = alias.and_then(|n| self.bare_target(n))
        else
        {
            return;
        };
        let decl = alias.and_then(|n| self.aliases.get(&n)).map(|(_, d, _)| d.span);
        do_if!(let Some(decl) = decl, self.refs.push((qualifier.span, decl)));
        // before `ထဲမှ` an enumeration name ends in `များ`
        let text = target.as_str();
        let plural = text.ends_with(PLURAL);
        qualifier.name = tern!(plural, target, Name::new(&format!("{text}{PLURAL}")));
    }
}

/// The kind of an alias in place of a use of its name: the levels written at
/// the use go around the levels of the alias, and the use keeps its measure
/// word. A use that adds no levels names the alias, whose word it counts with.
///
/// # Arguments
///
/// * `target` - The kind the alias names.
/// * `used` - The kind as written at the use.
///
/// # Returns
///
/// The kind.
fn spliced(mut target: Kind, used: &Kind) -> Kind
{
    target.alias = used.name.clone().filter(|_| used.wraps.is_empty());
    target.wraps.extend(used.wraps.iter().cloned());
    target.span = used.span;
    target.classifier = used.classifier.clone();
    target
}

impl VisitorMut for Swap<'_>
{
    fn visit_kind_mut(&mut self, n: &mut Kind)
    {
        do_for!(arg in &mut n.args, self.visit_kind_mut(arg));
        let alias = (n.base == BaseKind::Named).then_some(n.name.as_ref()).flatten();
        do_if!(let Some(used) = alias, self.note(used));
        let target = alias.and_then(|ident| self.expanded(ident.name));
        let target = target.and_then(|t| self.with_args(t, n));
        do_if!(let Some(target) = target, *n = spliced(target, n));
        do_if!(let Some(sig) = &mut n.sig, walk_sig_mut(self, sig));
        do_if!(let Some(t) = &mut n.table, walk_table_mut(self, t));
        let parts = n.parts.as_deref_mut();
        let sides = parts.into_iter().flat_map(|p| [p.ok.as_mut(), p.err.as_mut()]);
        do_for!(side in sides.flatten(), self.visit_kind_mut(side));
        let counts = n.wraps.iter_mut().filter_map(count_expr);
        do_for!(count in counts, self.visit_expr_mut(count));
    }

    fn visit_expr_mut(&mut self, n: &mut Expr)
    {
        match n
        {
            Expr::Struct(lit) =>
            {
                let target = self.bare_target(lit.name.name);
                do_if!(target.is_some(), self.note(&lit.name));
                do_if!(let Some(target) = target, lit.name.name = target);
            }
            Expr::Member(member) => self.swap_qualifier(&mut member.qualifier),
            _ => (),
        }
        walk_expr_mut(self, n);
    }
}

/// The count expression of a list level, when it is written as one.
///
/// # Arguments
///
/// * `wrap` - The level.
///
/// # Returns
///
/// The expression, or `None`.
fn count_expr(wrap: &mut Wrap) -> Option<&mut Expr>
{
    let Wrap::List(Cells::Expr(e)) = wrap
    else
    {
        return None;
    };
    Some(e)
}
