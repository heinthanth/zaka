//! Function values: the address of a function, and a call through a value
//! of a function kind.
//!
//! A call names its function. The name finds a declared function first,
//! then a variable, a parameter or a constant of a function kind; a name
//! that finds both is ZK0335, so a reader never has to guess. The long call
//! forms can name a place instead (a field, a cell, a read through a
//! pointer), which must hold a function kind. Each later stage decides the
//! same rule: a declared function of the name is a direct call, else the call
//! goes through the value.

use zakac_ast::expr::Expr;
use zakac_ast::expr::FnAddrExpr;
use zakac_ast::expr::Ident;
use zakac_ast::name::Name;
use zakac_ast::stmt::FnDecl;
use zakac_ast::visit::Node;
use zakac_diagnostics::kinds as kd;
use zakac_diagnostics::resolve as rd;
use zakac_diagnostics::resolve::TraitValueUse;
use zakac_parser::load::core_builtin_fns;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_span::span::Span;

use crate::collect::sig_of;
use crate::fn_kind::FnKind;
use crate::fn_kind::FnParam;
use crate::kind::DataKind;
use crate::scope::Capture;
use crate::scope::FnSig;
use crate::semcheck::Checker;

impl Checker
{
    /// The functions a call can run: the declared functions of its name, or
    /// the one function kind of the value it goes through.
    ///
    /// # Arguments
    ///
    /// * `callee` - The name at the call; with a place, its first name.
    /// * `place` - The place that holds the function value, when written.
    ///
    /// # Returns
    ///
    /// The functions, or `None` after a report.
    pub(crate) fn call_target(
        &mut self, callee: &Ident, place: Option<&Expr>,
    ) -> Option<Vec<FnSig>>
    {
        if let Some(place) = place
        {
            let kind = self.infer(place, None)?;
            return self.value_target(callee.name, place.span(), kind);
        }
        let value = self.scopes.lookup(&callee.name);
        let holds_fn = value.and_then(|b| b.kind).is_some_and(DataKind::is_fn);
        let fns = self.fns_named(&callee.name);
        match (fns, value)
        {
            (Some(_), Some(_)) if holds_fn =>
            {
                self.report(rd::callee_twice(callee.span, &callee.name));
                None
            }
            (Some(sigs), _) => Some(sigs),
            (None, Some(binding)) =>
            {
                self.refer(&callee.name, callee.span, binding.home());
                self.value_target(callee.name, callee.span, binding.kind?)
            }
            (None, None) =>
            {
                // A function the compiler gives reads as its sentences alone;
                // a call with other words fits none of its heads.
                let heads = builtin_heads(&callee.name);
                let fits_none =
                    heads.map(|h| rd::no_overload_fits(callee.span, &callee.name, &h));
                let missing = rd::undeclared_function(callee.span, &callee.name);
                let diag = fits_none
                    .unwrap_or_else(|| self.with_hidden(missing, &callee.name));
                self.report(diag);
                None
            }
        }
    }

    /// The one function a value of `kind` can run.
    ///
    /// # Arguments
    ///
    /// * `name` - The name at the call, for messages.
    /// * `span` - The name or the place.
    /// * `kind` - The kind of the value.
    ///
    /// # Returns
    ///
    /// The function, or `None` after a ZK0462 report.
    fn value_target(
        &mut self, name: Name, span: Span, kind: DataKind,
    ) -> Option<Vec<FnSig>>
    {
        let Some(f) = kind.as_fn()
        else
        {
            self.report(kd::not_a_function(span, &name, &kind.name()));
            return None;
        };
        let call = rd::trait_value_use(span, TraitValueUse::Call);
        ret_if!(f.of_trait.is_some(), self.report(call), None);
        let sig = FnSig::builder()
            .params(f.param_kinds())
            .edits(f.edits())
            .fills(f.fills())
            .maybe_ret(f.ret)
            .prepositions(f.prepositions())
            .maybe_start(f.start)
            .maybe_end(f.end)
            .inner(name);
        Some(vec![sig.build()])
    }

    /// Infer `<name> ဟူသော လုပ်ဆောင်ချက်`: the function kind of the function
    /// it names. With several functions of the name, the kind the value goes
    /// to picks one (ZK0463 when none or several fit). An external function
    /// has no address (ZK0464).
    ///
    /// # Arguments
    ///
    /// * `n` - The address.
    /// * `expect` - The kind its place expects, when known.
    ///
    /// # Returns
    ///
    /// The function kind, or `None` after a report.
    pub(crate) fn infer_fn_addr(
        &mut self, n: &FnAddrExpr, expect: Option<DataKind>,
    ) -> Option<DataKind>
    {
        let name = &n.name;
        let templates = self.templates_named(&name.name);
        let count = rd::kind_arg_count(name.span, &name.name, 0, n.kind_args.len());
        ret_if!(
            templates.is_empty() && !n.kind_args.is_empty(),
            self.report(count),
            None
        );
        let generic = (!templates.is_empty()).then(|| {
            let t =
                self.choose_value_template(templates, name, &n.kind_args, expect)?;
            self.generic_value(&t, name, &n.kind_args, expect)
        });
        let sigs = match generic
        {
            Some(sig) => Some(vec![sig?]),
            None => self.fns_named(&name.name),
        };
        let Some(sigs) = sigs
        else
        {
            let diag = rd::undeclared_function(name.span, &name.name);
            self.report(self.with_hidden(diag, &name.name));
            return None;
        };
        let want = expect.and_then(DataKind::as_fn);
        let fits =
            |s: &&FnSig| sigs.len() == 1 || want.is_some_and(|w| sig_kind(s).fits(w));
        let found: Vec<&FnSig> = sigs.iter().filter(fits).collect();
        let [one] = found.as_slice()
        else
        {
            self.report(kd::fn_addr_unclear(n.span, &name.name));
            return None;
        };
        ret_if!(
            one.external,
            self.report(kd::external_fn_addr(n.span, &name.name)),
            None
        );
        ret_if!(
            one.unchecked,
            self.report(kd::unsafe_fn_value(n.span, &name.name)),
            None
        );
        do_if!(
            sigs.len() > 1,
            _ = self.chosen.insert((self.top, name.span), one.inner)
        );
        self.name_foreign(one, name.span, &sigs);
        self.refer(&name.name, name.span, one.decl);
        Some(DataKind::Fn(sig_kind(one)))
    }
}

impl Checker
{
    /// The functions a name calls: the function declared inside a function
    /// that is in scope, which hides each top-level one, else the top-level
    /// functions of the name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    ///
    /// # Returns
    ///
    /// The functions, or `None` when none has the name.
    pub(crate) fn fns_named(&self, name: &str) -> Option<Vec<FnSig>>
    {
        let nested = self.scopes.lookup_fn(name).map(|s| vec![s]);
        nested.or_else(|| self.fns.get(name).cloned())
    }

    /// Declare a function inside a function in the scope around it, so the
    /// statements after it, and its own, can call it. A second one of the
    /// name in the scope is ZK0304; these functions have no overloads. It is
    /// a closure once its statements turn out to name locals around it.
    ///
    /// # Arguments
    ///
    /// * `n` - The declaration.
    pub(crate) fn declare_nested(&mut self, n: &FnDecl)
    {
        let mut sig = sig_of(n, &self.structs, 0, None);
        (sig.closure, sig.borrows) = (false, false);
        let fresh = self.scopes.declare_fn(n.name.name, sig);
        do_if!(!fresh, self.report(rd::duplicate_function(n.name.span, &n.name.name)));
    }

    /// Note the locals and closures a function declared inside a function
    /// names from around it, after its statements: with one or more, the
    /// function is a closure that shares them. Its own name is no capture.
    ///
    /// # Arguments
    ///
    /// * `n` - The declaration.
    /// * `used` - The names from around it that its statements use.
    pub(crate) fn note_captures(&mut self, n: &FnDecl, used: &[Capture])
    {
        let own = n.name.span;
        let used: Vec<Capture> =
            used.iter().filter(|u| u.decl != own).copied().collect();
        ret_if!(used.is_empty());
        do_if!(used.iter().any(|u| !u.func), self.scopes.mark_closure(&n.name.name));
        self.captured.of_fn.insert(own, used);
    }
}
/// The function kind of a declared function.
///
/// # Arguments
///
/// * `sig` - The function.
///
/// # Returns
///
/// The kind, its parameters with their `ဒေတာ ပြောင်းလဲရန်` flags and marks.
pub(crate) fn sig_kind(sig: &FnSig) -> &'static FnKind
{
    let edit = |n: usize| sig.edits.get(n).copied().unwrap_or(false);
    let fill = |n: usize| sig.fills.get(n).copied().unwrap_or(false);
    let param = |(n, kind): (usize, &DataKind)| FnParam {
        kind: *kind,
        edit: edit(n),
        fill: fill(n),
        preposition: sig.preposition(n),
    };
    let params = sig.params.iter().enumerate().map(param).collect();
    FnKind::make_with(params, sig.ret, sig.start, sig.end)
}

/// The heads of the functions of a name that the compiler gives in the
/// core, each as the sentence its call writes, each argument as `…`:
/// `… ထဲသို့ … ကို ထည့်`.
///
/// # Arguments
///
/// * `name` - The function name.
///
/// # Returns
///
/// The heads, `;` between them, or `None` when the core gives no such
/// function.
fn builtin_heads(name: &str) -> Option<String>
{
    let heads: Vec<String> = core_builtin_fns()
        .iter()
        .filter(|d| d.name.name == name)
        .map(head_text)
        .collect();
    (!heads.is_empty()).then(|| heads.join("; "))
}

/// The sentence a call of a head writes, each argument as `…`.
///
/// # Arguments
///
/// * `decl` - The head.
///
/// # Returns
///
/// The text.
fn head_text(decl: &FnDecl) -> String
{
    let word = |i: &Option<Ident>| i.as_ref().map(|i| i.name.to_string());
    let args = decl
        .params
        .iter()
        .flat_map(|p| std::iter::once("…".to_string()).chain(word(&p.preposition)));
    let start = word(&decl.phrases.start).into_iter();
    let end = word(&decl.phrases.end).into_iter();
    let words: Vec<String> =
        start.chain(args).chain(end).chain([decl.name.name.to_string()]).collect();
    words.join(" ")
}
