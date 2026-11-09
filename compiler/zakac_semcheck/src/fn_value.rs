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
use zakac_ast::stmt::CaptureItem;
use zakac_ast::stmt::CaptureMode;
use zakac_ast::stmt::FnDecl;
use zakac_ast::visit::Node;
use zakac_diagnostics::kinds as kd;
use zakac_diagnostics::ownership as od;
use zakac_diagnostics::resolve as rd;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::collect::sig_of;
use crate::fn_kind::FnKind;
use crate::fn_kind::FnParam;
use crate::kind::DataKind;
use crate::scope::Capture;
use crate::scope::FnSig;
use crate::semcheck::Checker;
use crate::semcheck::RESULT_NAME;

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
        let Some(sigs) = self.fns_named(&name.name)
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
    /// name in the scope is ZK0304; these functions have no overloads.
    ///
    /// # Arguments
    ///
    /// * `n` - The declaration.
    /// * `borrows` - Whether its closure borrows.
    pub(crate) fn declare_nested(&mut self, n: &FnDecl, borrows: bool)
    {
        let mut sig = sig_of(n, &self.structs, 0);
        (sig.closure, sig.borrows) = (!n.captures.is_empty(), borrows);
        let prior = self.scopes.declare_fn(n.name.name, sig);
        do_if!(
            prior.is_err(),
            self.report(rd::duplicate_function(n.name.span, &n.name.name))
        );
    }

    /// Check the capture list of a function declared inside a function,
    /// before its statements: each name is a local of a function around it
    /// (ZK0340 otherwise), or a nested function with captures.
    ///
    /// # Arguments
    ///
    /// * `n` - The declaration.
    ///
    /// # Returns
    ///
    /// Whether the closure borrows.
    pub(crate) fn check_capture_list(&mut self, n: &FnDecl) -> bool
    {
        let each: Vec<bool> =
            n.captures.iter().map(|c| self.check_capture(n, c)).collect();
        each.into_iter().any(|b| b)
    }

    /// Check one capture of a capture list.
    ///
    /// # Arguments
    ///
    /// * `n` - The declaration.
    /// * `c` - The capture.
    ///
    /// # Returns
    ///
    /// Whether the capture makes the closure borrow.
    fn check_capture(&mut self, n: &FnDecl, c: &CaptureItem) -> bool
    {
        let (name, lent) = (&c.name, c.mode == CaptureMode::Lend);
        let not_local = || rd::capture_not_local(name.span, &n.name.name, &name.name);
        if let Some(at) = self.scopes.index_of(&name.name)
        {
            let binding = self.scopes.lookup(&name.name).expect("the name is declared");
            self.refer(&name.name, name.span, binding.home());
            ret_if!(!self.scopes.is_local(at), self.report(not_local()), false);
            return lent || binding.kind.is_some_and(DataKind::holds_borrow);
        }
        let sig = self.scopes.lookup_fn(&name.name).filter(|s| s.closure);
        let Some(sig) = sig
        else
        {
            self.report(not_local());
            return false;
        };
        self.refer(&name.name, name.span, sig.decl);
        lent || sig.borrows
    }

    /// Check the uses of a function declared inside a function against its
    /// capture list, after its statements: a local or a closure from around
    /// it that the list leaves out is ZK0339, and a listed one never used is
    /// ZK0341. Its own name needs no listing.
    ///
    /// # Arguments
    ///
    /// * `n` - The declaration.
    /// * `used` - The names from around it that its statements use.
    pub(crate) fn check_captures_used(&mut self, n: &FnDecl, used: &[Capture])
    {
        let func = &n.name;
        let listed = |u: &&Capture| n.captures.iter().any(|c| c.name.name == u.name);
        let unlisted =
            used.iter().filter(|u| u.decl != func.span).filter(|u| !listed(u));
        let mut diags: Vec<_> = unlisted
            .map(|u| rd::capture_unlisted(func.span, &func.name, &u.name))
            .collect();
        let unused =
            n.captures.iter().filter(|c| !used.iter().any(|u| u.name == c.name.name));
        // A name that is no capture has its own report.
        let local = |c: &&CaptureItem| {
            let at = self.scopes.index_of(&c.name.name);
            at.map_or(self.scopes.fn_index_of(&c.name.name).is_some(), |at| {
                self.scopes.is_local(at)
            })
        };
        let unused = unused.filter(local);
        diags.extend(
            unused.map(|c| rd::capture_unused(c.name.span, &func.name, &c.name.name)),
        );
        do_for!(d in diags, self.report(d));
    }

    /// Report the address of a function that borrows locals where it would
    /// outlive them (ZK0722): a result, a field, a cell, a place reached
    /// through an address, or a C function.
    ///
    /// # Arguments
    ///
    /// * `e` - The value.
    pub(crate) fn check_no_closure(&mut self, e: &Expr)
    {
        let Some(name) = self.closure_of(e)
        else
        {
            return;
        };
        self.report(od::closure_escapes(e.span(), &name));
    }

    /// Report the address of a function that borrows locals set into a
    /// place it would outlive (ZK0722): each place but a variable of the
    /// block that declares the function, or of a block inside it. The
    /// result variable counts as the result.
    ///
    /// # Arguments
    ///
    /// * `target` - The place.
    /// * `value` - The value.
    pub(crate) fn check_closure_set(&mut self, target: &Expr, value: &Expr)
    {
        let Some(name) = self.closure_of(value)
        else
        {
            return;
        };
        let var = tern!(let Expr::Name(v) = target, Some(v.name), None);
        let var = var.filter(|v| v.as_str() != RESULT_NAME);
        let depth = var.and_then(|v| self.scopes.index_of(&v));
        let inside =
            depth.zip(self.scopes.fn_index_of(&name)).is_some_and(|(v, f)| v >= f);
        do_if!(!inside, self.report(od::closure_escapes(value.span(), &name)));
    }

    /// Report the address of a nested function with captures given to a C
    /// function (ZK0723): C calls a bare code address.
    ///
    /// # Arguments
    ///
    /// * `e` - The argument.
    pub(crate) fn check_no_closure_to_c(&mut self, e: &Expr)
    {
        let Some(name) = self.closure_named(e, false)
        else
        {
            return;
        };
        self.report(od::closure_to_c(e.span(), &name));
    }

    /// The name of the function an expression gives the address of, when it
    /// is declared inside a function and borrows locals.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    ///
    /// # Returns
    ///
    /// The name, or `None`.
    pub(crate) fn closure_of(&self, e: &Expr) -> Option<Name>
    {
        self.closure_named(e, true)
    }

    /// The name of the function an expression gives the address of, when it
    /// is declared inside a function with captures.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    /// * `borrows` - Whether to find a borrowing closure alone.
    ///
    /// # Returns
    ///
    /// The name, or `None`.
    fn closure_named(&self, e: &Expr, borrows: bool) -> Option<Name>
    {
        ret_if!(let Expr::Group(g) = e, self.closure_named(&g.inner, borrows));
        let Expr::FnAddr(n) = e
        else
        {
            return None;
        };
        let sig = self.scopes.lookup_fn(&n.name.name)?;
        let found = tern!(borrows, sig.borrows, sig.closure);
        found.then_some(n.name.name)
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
