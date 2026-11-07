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
use zakac_ast::visit::Node;
use zakac_diagnostics::kinds as kd;
use zakac_diagnostics::resolve as rd;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_span::span::Span;

use crate::fn_kind::FnKind;
use crate::fn_kind::FnParam;
use crate::kind::DataKind;
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
        let fns = self.fns.get(&callee.name).cloned();
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
                self.value_target(callee.name, callee.span, binding.kind?)
            }
            (None, None) =>
            {
                let diag = rd::undeclared_function(callee.span, &callee.name);
                self.report(self.with_hidden(diag, &callee.name));
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
        let sig = FnSig::builder()
            .params(f.param_kinds())
            .edits(f.edits())
            .maybe_ret(f.ret)
            .maybe_marks(f.marks())
            .inner(name);
        Some(vec![sig.build()])
    }

    /// Infer `လုပ်ဆောင်ချက် <name> ၏ လိပ်စာ`: the function kind of the function
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
        let Some(sigs) = self.fns.get(&name.name).cloned()
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
        do_if!(sigs.len() > 1, _ = self.chosen.insert(name.span, one.inner));
        Some(DataKind::Fn(sig_kind(one)))
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
/// The kind, its parameters with their edit marks and marks.
pub(crate) fn sig_kind(sig: &FnSig) -> &'static FnKind
{
    let mark = |n: usize| sig.marks.as_ref().and_then(|m| m.get(n).copied());
    let edit = |n: usize| sig.edits.get(n).copied().unwrap_or(false);
    let param = |(n, kind): (usize, &DataKind)| FnParam {
        kind: *kind,
        edit: edit(n),
        mark: mark(n),
    };
    FnKind::make(sig.params.iter().enumerate().map(param).collect(), sig.ret)
}
