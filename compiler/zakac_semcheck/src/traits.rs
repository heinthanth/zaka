//! Traits. A kind has a trait when the functions of the program give it each
//! function of the trait: a function of the name whose parameter and result
//! kinds are the ones the trait names, with the kind in place of the first
//! kind parameter. A trait block checks that it gives them; a kind can have a
//! trait with no block. A generic checks each constraint of its kind
//! parameters where it is used, before the compiler makes the instance. The
//! built-in traits are the ones the compiler knows, from the operators each
//! kind takes.

use std::collections::HashMap;
use std::collections::HashSet;

use zakac_ast::expr::Ident;
use zakac_ast::kind::Kind;
use zakac_ast::name::Name;
use zakac_ast::stmt::FnDecl;
use zakac_ast::stmt::KindParam;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::Stmt;
use zakac_ast::stmt::TraitBlock;
use zakac_ast::stmt::TraitDecl;
use zakac_diagnostics::resolve as rd;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_false;
use zakac_rust_sugars::ret_if;
use zakac_span::span::Span;

use crate::generic::in_prelude;
use crate::generic::kind_ast;
use crate::kind::DataKind;
use crate::ops::Want;
use crate::scope::FnSig;
use crate::semcheck::Checker;

/// The built-in trait of the kinds that take `+ - * /`: the numbers.
pub const NUMBER_LIKE: &str = "နံပါတ်နှင့်တူသော";

/// The built-in trait of the kinds that take `+`: the numbers and text.
pub const ADDABLE: &str = "ပေါင်းနိုင်သော";

/// The built-in traits of the kinds that take `-`, `*` and `/`: the numbers.
pub const NUMBER_OPS: [&str; 3] = ["နုတ်နိုင်သော", "မြှောက်နိုင်သော", "စားနိုင်သော"];

/// The built-in trait of the kinds that take `%`: the numbers.
pub const REMAINDER: &str = "အကြွင်းရှာနိုင်သော";

/// The built-in traits of the kinds that take `&`, `|`, `^`, `~`, `<<` and
/// `>>`: the integers.
pub const INT_OPS: [&str; 6] = [
    "ဘစ်ဆုံနိုင်သော",
    "ဘစ်ပေါင်းစုနိုင်သော",
    "ဘစ်ခြားနားနိုင်သော",
    "ဘစ်ပြောင်းပြန်ပြောင်းလဲနိုင်သော",
    "ဘစ်ဘယ်ရွေ့နိုင်သော",
    "ဘစ်ညာရွေ့နိုင်သော",
];

/// The built-in trait of the kinds that take the `-` of one operand: the
/// signed numbers.
pub const NEGATABLE: &str = "အနုတ်ပြောင်းနိုင်သော";

/// The built-in trait of the kinds that take `< > <= >=`: the numbers and
/// the characters.
pub const ORDERED: &str = "နှိုင်းယှဉ်နိုင်သော";

/// The built-in trait of the kinds that take `==` and `!=`: each kind.
pub const EQUAL: &str = "တူမတူစစ်ဆေးနိုင်သော";

/// The built-in trait of the kinds `ပြသမည်` prints: each kind but a handle of
/// a kind from outside.
pub const PRINTABLE: &str = "စာသားအဖြစ်ပြောင်းလဲပြသနိုင်သော";

/// The traits of a program, under their names, each with the unit that
/// declares it; the first of a name wins.
pub(crate) type Traits = HashMap<Name, (u32, TraitDecl)>;

/// The traits a program declares.
///
/// # Arguments
///
/// * `program` - The program.
///
/// # Returns
///
/// The traits.
pub(crate) fn traits(program: &Program) -> Traits
{
    let mut found = Traits::new();
    for (i, stmt) in program.stmts.iter().enumerate()
    {
        let Stmt::Trait(decl) = stmt
        else
        {
            continue;
        };
        let unit = program.unit_of(i);
        // The first of a name wins, but a trait of a file hides one of the
        // prelude.
        let held = found.get(&decl.name.name).map(|(u, _)| *u);
        let taken =
            held.is_some_and(|u| !in_prelude(program, u) || in_prelude(program, unit));
        do_if!(!taken, _ = found.insert(decl.name.name, (unit, decl.clone())));
    }
    found
}

/// Whether a kind has a built-in trait.
///
/// # Arguments
///
/// * `name` - The trait.
/// * `kind` - The kind.
///
/// # Returns
///
/// `Some` with the answer for a built-in trait, `None` for another name.
fn builtin(name: &str, kind: DataKind) -> Option<bool>
{
    match name
    {
        NUMBER_LIKE => Some(Want::Number.accepts(kind)),
        ADDABLE => Some(Want::Number.accepts(kind) || kind.is_text()),
        _ if NUMBER_OPS.contains(&name) => Some(Want::Number.accepts(kind)),
        REMAINDER => Some(Want::Number.accepts(kind)),
        _ if INT_OPS.contains(&name) => Some(Want::Int.accepts(kind)),
        NEGATABLE => Some(Want::Signed.accepts(kind)),
        ORDERED => Some(Want::Ordered.accepts(kind)),
        EQUAL => Some(true),
        PRINTABLE => Some(kind.as_external().is_none()),
        _ => None,
    }
}

/// A function a trait names, for one kind: its name, parameter kinds and
/// result kind.
struct Wanted
{
    /// The function name.
    name: Ident,
    /// The parameter kinds.
    params: Vec<DataKind>,
    /// The result kind.
    ret: Option<DataKind>,
}

impl Checker
{
    /// Whether each kind meets the constraints of its kind parameter: each
    /// trait a kind parameter needs, with the kinds of the other kind
    /// parameters in place in it.
    ///
    /// # Arguments
    ///
    /// * `params` - The kind parameters.
    /// * `kinds` - The kinds.
    /// * `span` - The use, for reports.
    ///
    /// # Returns
    ///
    /// `true` when each constraint is met; each one that is not is reported.
    pub(crate) fn meets_bounds(
        &mut self, params: &[KindParam], kinds: &[DataKind], span: Span,
    ) -> bool
    {
        let phrases = kinds.iter().filter_map(|k| kind_ast(*k, span));
        let names = params.iter().map(|p| p.name.name);
        let bound: HashMap<Name, Kind> = names.zip(phrases).collect();
        let mut met = true;
        for (param, kind) in params.iter().zip(kinds)
        {
            for written in &param.bounds
            {
                let mut need = written.clone();
                need.substitute(&bound);
                met &= self.has_trait(*kind, &need, span, &mut HashSet::new());
            }
        }
        met
    }

    /// Whether a kind has a trait, with each trait it needs; a trait that is
    /// not met is reported at the use.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    /// * `need` - The trait with the kinds after the first.
    /// * `span` - The use, for reports.
    /// * `seen` - The traits checked on the path here, so a trait that needs
    ///   itself ends.
    ///
    /// # Returns
    ///
    /// `true` when the kind has the trait.
    fn has_trait(
        &mut self, kind: DataKind, need: &Kind, span: Span, seen: &mut HashSet<Name>,
    ) -> bool
    {
        let Some(ident) = need.name.clone()
        else
        {
            return true;
        };
        let decl = self.traits.get(&ident.name).map(|(_, d)| d.clone());
        let Some(decl) = decl
        else
        {
            let known = builtin(&ident.name, kind);
            do_if!(
                known.is_none(),
                self.report(rd::unknown_trait(ident.span, &ident.name))
            );
            let met = known.unwrap_or(true);
            let report = rd::bound_not_met(span, &kind.name(), &ident.name, "");
            do_if!(!met, self.report(report));
            return met;
        };
        self.refer(&ident.name, ident.span, decl.name.span);
        // A marked trait: the built-in kinds have it through the compiler's
        // rule; a kind a file declares, through its functions.
        let rule = builtin(&ident.name, kind).filter(|_| decl.builtin);
        ret_if!(rule == Some(true), true);
        let declared = self.kind_home(kind).is_some();
        let lacks = rule == Some(false) && !declared;
        let report = rd::bound_not_met(span, &kind.name(), &ident.name, "");
        ret_if!(lacks, self.report(report), false);
        ret_if!(!seen.insert(ident.name), true);
        let Some(bound) = self.trait_kinds(&decl, kind, &ident, &need.args)
        else
        {
            return false;
        };
        let needs: Vec<Kind> = decl.kind_params[0].bounds.clone();
        let mut met = needs.iter().all(|n| {
            let mut n = n.clone();
            n.substitute(&bound);
            self.has_trait(kind, &n, span, seen)
        });
        ret_false!(!met);
        for want in self.trait_fns(&decl, &bound)
        {
            met &= self.gives(kind, &ident.name, &want, span);
        }
        met
    }

    /// The kind phrase each kind parameter of a trait stands for: the kind
    /// that has it for the first, the kinds written for the others (ZK0340
    /// for another count).
    ///
    /// # Arguments
    ///
    /// * `decl` - The trait.
    /// * `kind` - The kind.
    /// * `ident` - The trait name where it is used.
    /// * `args` - The kinds written after the first.
    ///
    /// # Returns
    ///
    /// The phrases, or `None` after a report or for a kind with no phrase.
    fn trait_kinds(
        &mut self, decl: &TraitDecl, kind: DataKind, ident: &Ident, args: &[Kind],
    ) -> Option<HashMap<Name, Kind>>
    {
        let want = decl.kind_params.len() - 1;
        let count = rd::kind_arg_count(ident.span, &ident.name, want, args.len());
        ret_if!(args.len() != want, self.report(count), None);
        let first = kind_ast(kind, ident.span)?;
        let phrases = std::iter::once(first).chain(args.iter().cloned());
        let names = decl.kind_params.iter().map(|p| p.name.name);
        Some(names.zip(phrases).collect())
    }

    /// The functions a trait names, with the kinds of its kind parameters in
    /// place.
    ///
    /// # Arguments
    ///
    /// * `decl` - The trait.
    /// * `bound` - The kind phrase of each kind parameter.
    ///
    /// # Returns
    ///
    /// The functions.
    fn trait_fns(&self, decl: &TraitDecl, bound: &HashMap<Name, Kind>) -> Vec<Wanted>
    {
        let kind_of = |k: &Kind| {
            let mut k = k.clone();
            k.substitute(bound);
            DataKind::from_ast(&k, &self.structs)
        };
        let one = |f: &FnDecl| Wanted {
            name: f.name.clone(),
            params: f.params.iter().map(|p| kind_of(&p.kind)).collect(),
            ret: f.ret.as_ref().map(kind_of),
        };
        decl.fns.iter().map(one).collect()
    }

    /// Whether the functions of the program give a kind one function of a
    /// trait: none is ZK0343, two are ZK0347. The functions looked at are
    /// those of the file of the use, of the kind and of the trait.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    /// * `trait_name` - The trait.
    /// * `want` - The function.
    /// * `span` - The use, for reports.
    ///
    /// # Returns
    ///
    /// `true` when one function gives it.
    fn gives(
        &mut self, kind: DataKind, trait_name: &str, want: &Wanted, span: Span,
    ) -> bool
    {
        let found = self.fitting(want, self.places(kind, trait_name));
        match found
        {
            1 => true,
            0 =>
            {
                let name = &want.name.name;
                self.report(rd::bound_not_met(span, &kind.name(), trait_name, name));
                false
            }
            _ =>
            {
                self.report(rd::trait_fn_twice(span, &kind.name(), &want.name.name));
                false
            }
        }
    }

    /// The count of functions, each once, in some files that have the name,
    /// the parameter kinds and the result kind of a trait function.
    ///
    /// # Arguments
    ///
    /// * `want` - The function.
    /// * `units` - The files to look in.
    ///
    /// # Returns
    ///
    /// The count.
    fn fitting(&self, want: &Wanted, units: Vec<u32>) -> usize
    {
        let fits = |s: &&FnSig| s.params == want.params && s.ret == want.ret;
        let sigs = units.iter().filter_map(|u| self.unit_fns.get(*u as usize));
        let sigs = sigs.filter_map(|fns| fns.get(&want.name.name)).flatten();
        let decls: HashSet<Span> = sigs.filter(fits).map(|s| s.decl).collect();
        decls.len()
    }

    /// The files whose functions can give a kind a trait: the file of the
    /// use, the file of the kind and the file of the trait.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    /// * `trait_name` - The trait.
    ///
    /// # Returns
    ///
    /// The units.
    fn places(&self, kind: DataKind, trait_name: &str) -> Vec<u32>
    {
        let home = self.kind_home(kind);
        let of_trait = self.traits.get(trait_name).map(|(u, _)| *u);
        let mut units = vec![self.unit];
        units.extend(home.into_iter().chain(of_trait).filter(|u| *u != self.unit));
        units
    }

    /// The file that declares the struct or the enumeration of a kind.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// The unit, or `None` for a kind no file declares.
    pub(crate) fn kind_home(&self, kind: DataKind) -> Option<u32>
    {
        let name = kind.as_struct().map(|s| s.name.clone());
        let name = name.or_else(|| kind.as_enum().map(|e| e.name.clone()))?;
        self.homes.get(name.as_str()).map(|(u, _)| *u)
    }

    /// Check a trait: each trait it needs is one, and its kind parameters
    /// and functions name no other trait.
    ///
    /// # Arguments
    ///
    /// * `n` - The trait.
    pub(crate) fn check_trait(&mut self, n: &TraitDecl)
    {
        let bounds: Vec<Kind> =
            n.kind_params.iter().flat_map(|p| p.bounds.clone()).collect();
        for bound in &bounds
        {
            let Some(ident) = &bound.name
            else
            {
                continue;
            };
            let decl = self.traits.get(&ident.name).map(|(_, d)| d.name.span);
            do_if!(let Some(decl) = decl, self.refer(&ident.name, ident.span, decl));
            let known =
                decl.is_some() || builtin(&ident.name, DataKind::Bool).is_some();
            do_if!(!known, self.report(rd::unknown_trait(ident.span, &ident.name)));
        }
    }

    /// Check a trait block: its trait is one the program declares, each
    /// function of the trait is in the block with the kinds the trait gives
    /// it, each function of the block is one of the trait, and the block
    /// stands in the file of the kind or of the trait.
    ///
    /// # Arguments
    ///
    /// * `n` - The block.
    pub(crate) fn check_trait_block(&mut self, n: &TraitBlock)
    {
        let kind = self.written_kind(&n.kind, crate::list::KindUse::Other);
        let Some(ident) = n.bound.name.clone()
        else
        {
            return;
        };
        let found = self.traits.get(&ident.name).cloned();
        let Some((home, decl)) = found
        else
        {
            return self.report(rd::unknown_trait(ident.span, &ident.name));
        };
        self.refer(&ident.name, ident.span, decl.name.span);
        let at_home = self.unit == home || self.kind_home(kind) == Some(self.unit);
        let place = rd::trait_block_place(ident.span, &kind.name(), &ident.name);
        do_if!(!at_home, self.report(place));
        let Some(bound) = self.trait_kinds(&decl, kind, &ident, &n.bound.args)
        else
        {
            return;
        };
        let wanted = self.trait_fns(&decl, &bound);
        let given: Vec<(Ident, Option<FnSig>)> =
            n.fn_names.iter().map(|f| (f.clone(), self.sig_at(f))).collect();
        let fits = |w: &Wanted, (f, s): &(Ident, Option<FnSig>)| {
            f.name == w.name.name
                && s.as_ref().is_some_and(|s| s.params == w.params && s.ret == w.ret)
        };
        let missing = wanted.iter().filter(|w| !given.iter().any(|g| fits(w, g)));
        let missing: Vec<String> = missing.map(|w| w.name.name.to_string()).collect();
        do_for!(f in missing, self.report(rd::trait_fn_missing(ident.span, &ident.name, &f)));
        let foreign = given.iter().filter(|g| !wanted.iter().any(|w| fits(w, g)));
        let foreign: Vec<Ident> = foreign.map(|(f, _)| f.clone()).collect();
        do_for!(f in foreign, self.report(rd::trait_fn_foreign(f.span, &ident.name, &f.name)));
    }

    /// The signature of the function declared at a name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name in the declaration.
    ///
    /// # Returns
    ///
    /// The signature, or `None` when no function of the file is declared
    /// there.
    fn sig_at(&self, name: &Ident) -> Option<FnSig>
    {
        let sigs = self.fns.get(&name.name)?;
        sigs.iter().find(|s| s.decl == name.span).cloned()
    }
}
