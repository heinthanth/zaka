//! Trait values, `<trait> အရာ`: a value of each kind that has the trait,
//! whose calls run the functions of the kind it holds. A trait value is a
//! function value (see [`crate::fn_kind::FnKind::trait_value`]); this module
//! checks the trait of each written trait value kind: the trait is declared,
//! it takes the kinds written after it, and each of its functions takes the
//! trait's kind as its first parameter and nowhere else but as its whole
//! result.

use std::collections::HashMap;
use std::collections::HashSet;

use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Kind;
use zakac_ast::name::Name;
use zakac_ast::stmt::FnDecl;
use zakac_ast::stmt::Param;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::TraitDecl;
use zakac_ast::threads::USABLE_IN_THREADS;
use zakac_ast::visit::Node;
use zakac_diagnostics::Code;
use zakac_diagnostics::resolve as rd;
use zakac_diagnostics::resolve::NotAKind;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::fn_kind::FnParam;
use crate::generic::kind_ast;
use crate::kind::DataKind;
use crate::scope::FnSig;
use crate::semcheck::Checker;
use crate::structs::StructTable;
use crate::traits::Wanted;
use crate::traits::builtin;
use crate::traits::traits;

impl Checker
{
    /// Check the trait of each trait value kind in a written kind phrase,
    /// its inner phrases included.
    ///
    /// # Arguments
    ///
    /// * `written` - The kind phrase.
    pub(crate) fn check_trait_values(&mut self, written: &Kind)
    {
        do_if!(written.base == BaseKind::TraitValue, self.check_trait_value(written));
        do_for!(inner in inner_kinds(written), self.check_trait_values(inner));
    }

    /// Check the trait of one trait value kind.
    ///
    /// # Arguments
    ///
    /// * `written` - The kind phrase, `<trait> အရာ`.
    fn check_trait_value(&mut self, written: &Kind)
    {
        let Some(ident) = &written.name
        else
        {
            return;
        };
        let found = self.traits.get(&ident.name).map(|(_, d)| d.clone());
        let Some(decl) = found
        else
        {
            let rule = ident.name.as_str() == USABLE_IN_THREADS
                || builtin(&ident.name, DataKind::Bool).is_some();
            let marked =
                rd::trait_not_a_kind(ident.span, &ident.name, NotAKind::Marked, "");
            let unknown = rd::unknown_trait(ident.span, &ident.name);
            return self.report(tern!(rule, marked, unknown));
        };
        self.refer(&ident.name, ident.span, decl.name.span);
        let want = decl.kind_params.len().saturating_sub(1);
        let got = written.args.len();
        let count = rd::kind_arg_count(ident.span, &ident.name, want, got);
        ret_if!(got != want, self.report(count));
        let marked =
            rd::trait_not_a_kind(ident.span, &ident.name, NotAKind::Marked, "");
        ret_if!(decl.builtin, self.report(marked));
        let Some((cause, func)) = not_a_kind(&decl)
        else
        {
            return;
        };
        let name = func.name.name;
        self.report(rd::trait_not_a_kind(ident.span, &ident.name, cause, &name));
    }
}

/// The first function of a trait that a trait value cannot run, with the
/// reason: each function takes the trait's kind first, takes it in no other
/// parameter, gives it back as its whole result or not at all, and has no
/// kind parameters of its own.
///
/// # Arguments
///
/// * `decl` - The trait.
///
/// # Returns
///
/// The reason and the function, or `None` when a trait value can run each
/// function.
pub(crate) fn not_a_kind(decl: &TraitDecl) -> Option<(NotAKind, &FnDecl)>
{
    let own = decl.kind_params.first()?.name.name;
    decl.fns.iter().find_map(|f| fn_not_a_kind(f, own).map(|cause| (cause, f)))
}

/// The reason a trait value cannot run one function of its trait.
///
/// # Arguments
///
/// * `f` - The function.
/// * `own` - The trait's kind parameter that stands for the held kind.
///
/// # Returns
///
/// The reason, or `None` when a trait value can run it.
fn fn_not_a_kind(f: &FnDecl, own: Name) -> Option<NotAKind>
{
    ret_if!(!f.kind_params.is_empty(), Some(NotAKind::Generic));
    let first = f.params.first().is_some_and(|p| is_bare(&p.kind, own));
    ret_if!(!first, Some(NotAKind::First));
    let elsewhere = f.params[1..].iter().any(|p| mentions(&p.kind, own));
    ret_if!(elsewhere, Some(NotAKind::Elsewhere));
    let inside = f.ret.as_ref().is_some_and(|r| mentions(r, own) && !is_bare(r, own));
    tern!(inside, Some(NotAKind::Result), None)
}

/// Whether a kind phrase is a kind parameter alone, with no levels.
///
/// # Arguments
///
/// * `kind` - The phrase.
/// * `name` - The kind parameter.
///
/// # Returns
///
/// `true` if it is.
pub(crate) fn is_bare(kind: &Kind, name: Name) -> bool
{
    let named = kind.base == BaseKind::Named
        && kind.name.as_ref().is_some_and(|n| n.name == name);
    named && kind.wraps.is_empty() && kind.args.is_empty() && kind.parts.is_none()
}

/// Whether a kind phrase names a kind parameter, at its base or inside.
///
/// # Arguments
///
/// * `kind` - The phrase.
/// * `name` - The kind parameter.
///
/// # Returns
///
/// `true` if it does.
fn mentions(kind: &Kind, name: Name) -> bool
{
    let named = kind.base != BaseKind::TraitValue
        && kind.name.as_ref().is_some_and(|n| n.name == name);
    named || inner_kinds(kind).into_iter().any(|k| mentions(k, name))
}

/// The kind phrases inside a phrase: its kinds, the parameters and the result
/// of a function kind, the sides of a result kind and the two kinds of a
/// table.
///
/// # Arguments
///
/// * `kind` - The phrase.
///
/// # Returns
///
/// The inner phrases.
fn inner_kinds(kind: &Kind) -> Vec<&Kind>
{
    let mut out: Vec<&Kind> = kind.args.iter().collect();
    if let Some(sig) = kind.sig.as_deref()
    {
        out.extend(sig.params.iter().map(|p| &p.kind));
        out.extend(&sig.ret);
    }
    if let Some(parts) = kind.parts.as_deref()
    {
        out.extend(parts.ok.iter().chain(&parts.err));
    }
    if let Some(t) = kind.table.as_deref()
    {
        out.extend([&t.index, &t.value]);
    }
    out
}

/// The name of the call the check puts around a value that goes into a trait
/// value; the IR builder makes the trait value from it.
pub const TO_TRAIT_FN: &str = "%trait-value";

/// The start of the name a call through a trait value takes, then the
/// number of the function among the functions of the trait.
pub const TRAIT_CALL: &str = "%trait-call:";

impl Checker
{
    /// Whether a value of kind `found` goes into a place of a trait value kind:
    /// its kind has the trait (ZK0343 when not). Outside trial checks, the
    /// conversion is recorded, and a plain value whose trait has a function
    /// that changes its first parameter is ZK0362.
    ///
    /// # Arguments
    ///
    /// * `e` - The value.
    /// * `found` - Its kind.
    /// * `want` - The kind of the place.
    ///
    /// # Returns
    ///
    /// `true` when the place is a trait value kind and another kind goes in:
    /// the value fits, or ZK0343 was reported.
    pub(crate) fn trait_fit(
        &mut self, e: &Expr, found: DataKind, want: DataKind,
    ) -> bool
    {
        let Some((name, args)) = want.as_trait_value()
        else
        {
            return false;
        };
        ret_if!(found.is_trait_value(), false);
        let need = trait_phrase(name, &args, e.span());
        let before = self.diags.len();
        let met = self.has_trait(found, &need, e.span(), &mut HashSet::new());
        let message = rd::bound_not_met_value(&found.name(), &name);
        let lacks =
            self.diags[before..].iter_mut().filter(|d| d.code == Code::BoundNotMet);
        lacks.for_each(|d| d.message = message.clone());
        ret_if!(!met || self.trials > 0, true);
        let at = Span::new(e.span().lo, e.span().lo);
        let Some(fns) = self.impl_names(found, name, &args, at)
        else
        {
            return true;
        };
        self.op_calls.to_traits.insert(e.id(), (want, fns));
        self.duplicate_lint(e, found, name);
        true
    }

    /// The function that gives a kind each function of a trait, as the later
    /// stages know it: a function of the name, parameter kinds and result
    /// kind in the file of the use, of the kind or of the trait, else the
    /// instance for the kind of the default of the trait function.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind that has the trait.
    /// * `name` - The trait.
    /// * `args` - Its kinds after the first.
    /// * `at` - A span no name at a call takes, for the instance of a default.
    ///
    /// # Returns
    ///
    /// The names in the trait's order, or `None` while an instance does not
    /// exist.
    fn impl_names(
        &mut self, kind: DataKind, name: Name, args: &[DataKind], at: Span,
    ) -> Option<Vec<Name>>
    {
        let decl = self.traits.get(&name).map(|(_, d)| d.clone())?;
        let ident = Ident::builder().name(name).span(at).build();
        let phrases: Vec<Kind> = args.iter().filter_map(|k| kind_ast(*k, at)).collect();
        let bound = self.trait_kinds(&decl, kind, &ident, &phrases)?;
        let wants = self.trait_fns(&decl, &bound);
        wants.iter().map(|w| self.impl_of(kind, name, w, at)).collect()
    }

    /// The function that gives a kind one function of a trait, as the later
    /// stages know it.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    /// * `name` - The trait.
    /// * `want` - The trait function, with the kind in place.
    /// * `at` - A span no name at a call takes.
    ///
    /// # Returns
    ///
    /// The name, or `None` while the instance of a default does not exist.
    fn impl_of(
        &mut self, kind: DataKind, name: Name, want: &Wanted, at: Span,
    ) -> Option<Name>
    {
        let units = self.places(kind, &name);
        let fits = |s: &&FnSig| s.params == want.params && s.ret == want.ret;
        let found = units
            .iter()
            .filter_map(|u| self.unit_fns.get(*u as usize))
            .filter_map(|fns| fns.get(&want.name.name))
            .flatten()
            .find(fits)
            .cloned();
        ret_if!(let Some(sig) = found, Some(self.known_name(&sig)));
        // The default: the generic function whose kind needs the trait.
        let needs = |t: &FnDecl| {
            let first = t.kind_params.first();
            first.is_some_and(|p| {
                p.bounds.iter().any(|b| b.name.as_ref().is_some_and(|n| n.name == name))
            })
        };
        let template =
            self.templates.fns_of(&want.name.name).into_iter().find(needs)?;
        let ident = Ident::builder().name(want.name.name).span(at).build();
        let sig = self.instance_sig(&template, &ident, Some(vec![kind]))?;
        Some(self.known_name(&sig))
    }

    /// Report ZK0362 for a plain value that goes into a trait value whose
    /// trait has a function that changes its first parameter.
    ///
    /// # Arguments
    ///
    /// * `e` - The value.
    /// * `found` - Its kind.
    /// * `name` - The trait.
    fn duplicate_lint(&mut self, e: &Expr, found: DataKind, name: Name)
    {
        ret_if!(found.as_object().is_some());
        let decl = self.traits.get(&name).map(|(_, d)| d.clone());
        let changing = decl.and_then(|d| {
            d.fns.iter().find(|f| f.params.first().is_some_and(|p| p.edit)).cloned()
        });
        let Some(f) = changing
        else
        {
            return;
        };
        self.report(rd::trait_value_duplicates(e.span(), &found.name(), &f.name.name));
    }

    /// Whether a call of this name on a value of this kind runs through a trait
    /// value: the kind is a trait value kind whose trait has a function of the
    /// name.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of the first argument.
    /// * `name` - The function name.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn runs_through(&self, kind: DataKind, name: &str) -> bool
    {
        let Some((trait_name, _)) = kind.as_trait_value()
        else
        {
            return false;
        };
        let decl = self.traits.get(&trait_name);
        decl.is_some_and(|(_, d)| d.fns.iter().any(|f| f.name.name == name))
    }

    /// The function a call runs when its first argument is a trait value whose
    /// trait has a function of the call's name: the trait function with the
    /// trait value kind in place of the trait's kind, named for the call
    /// through the trait value. A plain function that takes the trait value
    /// kind first wins.
    ///
    /// # Arguments
    ///
    /// * `callee` - The function name at the call.
    /// * `args` - The arguments.
    ///
    /// # Returns
    ///
    /// The function, or `None` when the call does not run through a trait
    /// value.
    pub(crate) fn dispatch_sig(
        &mut self, callee: &Ident, args: &[Expr],
    ) -> Option<FnSig>
    {
        let first = args.first()?;
        // A name that no trait has as a function runs through no trait value,
        // so its first argument needs no second inference.
        let named = |d: &TraitDecl| d.fns.iter().any(|f| f.name.name == callee.name);
        ret_if!(!self.traits.values().any(|(_, d)| named(d)), None);
        let before = self.diags.len();
        let kind = self.infer(first, None);
        self.diags.truncate(before);
        let kind = kind.filter(|k| self.runs_through(*k, &callee.name))?;
        let plain = self.fns.get(&callee.name).cloned().unwrap_or_default();
        ret_if!(plain.iter().any(|s| s.params.first() == Some(&kind)), None);
        let (name, _) = kind.as_trait_value()?;
        let decl = self.traits.get(&name).map(|(_, d)| d.clone())?;
        let index = decl.fns.iter().position(|f| f.name.name == callee.name)?;
        let (params, ret) = dispatch_shape(&decl, kind, index, &self.structs)?;
        let inner = Name::from(format!("{TRAIT_CALL}{index}"));
        #[rustfmt::skip]
        let sig = FnSig::builder().params(params.iter().map(|p| p.kind).collect())
            .edits(params.iter().map(|p| p.edit).collect())
            .fills(params.iter().map(|p| p.fill).collect()).maybe_ret(ret)
            .prepositions(params.iter().map(|p| p.preposition).collect())
            .decl(decl.fns[index].name.span).inner(inner).build();
        self.chosen.insert((self.top, callee.span), inner);
        self.refer(&callee.name, callee.span, sig.decl);
        Some(sig)
    }
}

/// The traits of a program, under their names: the first of a name, but a
/// trait of a file hides one of the core.
///
/// # Arguments
///
/// * `program` - The program.
///
/// # Returns
///
/// The traits.
pub fn trait_decls(program: &Program) -> HashMap<Name, TraitDecl>
{
    traits(program).into_iter().map(|(n, (_, d))| (n, d)).collect()
}

/// The parameters and the result of a function of a trait, called through a
/// trait value of the trait: the trait value kind stands for the trait's
/// kind, and the kinds it writes for the trait's other kinds.
///
/// # Arguments
///
/// * `decl` - The trait.
/// * `value` - The trait value kind.
/// * `index` - The number of the function among the functions of the trait.
/// * `structs` - The kind of each declared struct.
///
/// # Returns
///
/// The parameters and the result, or `None` for a number past the functions
/// or a kind with no phrase.
pub fn dispatch_shape(
    decl: &TraitDecl, value: DataKind, index: usize, structs: &StructTable,
) -> Option<(Vec<FnParam>, Option<DataKind>)>
{
    let (_, args) = value.as_trait_value()?;
    let f = decl.fns.get(index)?;
    let span = f.name.span;
    let mut phrases = vec![kind_ast(value, span)?];
    phrases.extend(args.iter().filter_map(|k| kind_ast(*k, span)));
    let names = decl.kind_params.iter().map(|p| p.name.name);
    let bound: HashMap<Name, Kind> = names.zip(phrases).collect();
    let kind_of = |k: &Kind| {
        let mut k = k.clone();
        k.substitute(&bound);
        DataKind::from_ast(&k, structs)
    };
    let param = |p: &Param| FnParam {
        kind: kind_of(&p.kind),
        edit: p.edit,
        fill: p.fill,
        preposition: p.preposition.as_ref().map(|m| m.name),
    };
    Some((f.params.iter().map(param).collect(), f.ret.as_ref().map(kind_of)))
}

/// The number of the function of a call through a trait value, from the
/// name the check gave the call.
///
/// # Arguments
///
/// * `name` - The name.
///
/// # Returns
///
/// The number, or `None` for each other call.
pub fn trait_call_index(name: &str) -> Option<usize>
{
    name.strip_prefix(TRAIT_CALL)?.parse().ok()
}

/// The trait of a trait value kind as the phrase a constraint writes: its
/// name with its kinds after the first.
///
/// # Arguments
///
/// * `name` - The trait.
/// * `args` - Its kinds after the first.
/// * `span` - The span the phrase takes.
///
/// # Returns
///
/// The phrase.
fn trait_phrase(name: Name, args: &[DataKind], span: Span) -> Kind
{
    let ident = Ident::builder().name(name).span(span).build();
    let args = args.iter().filter_map(|k| kind_ast(*k, span)).collect();
    Kind::builder().base(BaseKind::Named).name(ident).args(args).span(span).build()
}
