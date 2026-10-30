use std::collections::BTreeMap;

use bon::Builder;
use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::NameExpr;
use zakac_ast::expr::NodeId;
use zakac_ast::expr::PrefixOp;
use zakac_ast::stmt::AllocStmt;
use zakac_ast::stmt::AssignStmt;
use zakac_ast::stmt::BlockStmt;
use zakac_ast::stmt::BreakStmt;
use zakac_ast::stmt::CallStmt;
use zakac_ast::stmt::ContinueStmt;
use zakac_ast::stmt::CountedStmt;
use zakac_ast::stmt::DeclStmt;
use zakac_ast::stmt::FnDecl;
use zakac_ast::stmt::FreeStmt;
use zakac_ast::stmt::GenericStmt;
use zakac_ast::stmt::IfArm;
use zakac_ast::stmt::Param;
use zakac_ast::stmt::PrintArg;
use zakac_ast::stmt::PrintStmt;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::ResizeStmt;
use zakac_ast::stmt::ReturnStmt;
use zakac_ast::stmt::SizeArg;
use zakac_ast::stmt::Stmt;
use zakac_ast::stmt::StructDecl;
use zakac_ast::stmt::WhileStmt;
use zakac_ast::visit::Node;
use zakac_ast::visit::Visitor;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::kinds as kd;
use zakac_diagnostics::ownership as od;
use zakac_diagnostics::resolve as rd;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::collect::collect_fns;
use crate::kind::DataKind;
use crate::list::KindUse;
use crate::place::place_of;
use crate::place::var_name;
use crate::scope::Access;
use crate::scope::Binding;
use crate::scope::FnSigs;
use crate::scope::Scopes;
use crate::struct_check::collect_structs;
use crate::structs::StructTable;

/// The implicit result variable of a function with a result.
pub const RESULT_NAME: &str = "ရလဒ်";

/// Options of the check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Builder)]
pub struct SemcheckOptions
{
    /// Bit count of the platform-sized integer kinds.
    #[builder(default = 64)]
    pub platform_bits: u16,
    /// Report particle idiom warnings as errors.
    #[builder(default)]
    pub strict_particles: bool,
    /// Refuse each kind with a raw address level.
    #[builder(default)]
    pub no_raw: bool,
}

impl Default for SemcheckOptions
{
    /// The options with a 64-bit platform.
    ///
    /// # Returns
    ///
    /// The options.
    fn default() -> Self { Self::builder().build() }
}

/// The kind of each expression, indexed on node number.
pub type KindTable = BTreeMap<NodeId, DataKind>;

/// Output of the check.
#[derive(Debug, Clone, PartialEq, Default, Builder)]
pub struct SemcheckOutput
{
    /// The kind of each expression whose kind is known.
    #[builder(default)]
    pub kinds: KindTable,
    /// The kind of each declared struct.
    #[builder(default)]
    pub structs: StructTable,
    /// The diagnostics, in source order.
    #[builder(default)]
    pub diagnostics: Vec<Diagnostic>,
}

/// Check names, scopes and data kinds of a program.
///
/// # Arguments
///
/// * `program` - The tree.
/// * `options` - The options.
///
/// # Returns
///
/// The kind table and the diagnostics.
pub fn semcheck(program: &Program, options: &SemcheckOptions) -> SemcheckOutput
{
    let mut diags = Vec::new();
    let structs = collect_structs(program, &mut diags);
    let fns = collect_fns(program, &structs, &mut diags);
    #[rustfmt::skip]
    let mut checker = Checker::builder().fns(fns).structs(structs).diags(diags).platform(options.platform_bits).no_raw(options.no_raw).build();
    checker.scopes.push();
    checker.visit_program(program);
    checker.diags.sort_by_key(|d| d.span.lo);
    #[rustfmt::skip]
    return SemcheckOutput::builder().kinds(checker.kinds).structs(checker.structs).diagnostics(checker.diags).build();
}

/// The function whose block is being checked.
#[derive(Debug, Clone, Builder)]
pub(crate) struct FnCtx
{
    /// The function name.
    pub name: String,
    /// The result kind; `None` for a function without a result.
    pub ret: Option<DataKind>,
}

/// State of the walk.
#[derive(Builder)]
pub(crate) struct Checker
{
    /// Names in scope.
    #[builder(default)]
    pub scopes: Scopes,
    /// The top-level functions.
    pub fns: FnSigs,
    /// The kind of each declared struct.
    #[builder(default)]
    pub structs: StructTable,
    /// Kinds found so far.
    #[builder(default)]
    pub kinds: KindTable,
    /// Diagnostics found so far.
    #[builder(default)]
    pub diags: Vec<Diagnostic>,
    /// Number of loops around the current statement, inside the function.
    #[builder(default)]
    pub loop_depth: u32,
    /// The function around the current statement.
    pub func: Option<FnCtx>,
    /// Bit count of the platform-sized integer kinds.
    pub platform: u16,
    /// Whether kinds with a raw address level are refused.
    #[builder(default)]
    pub no_raw: bool,
}

impl Checker
{
    /// Record a diagnostic.
    ///
    /// # Arguments
    ///
    /// * `diag` - The diagnostic.
    pub fn report(&mut self, diag: Diagnostic) { self.diags.push(diag) }

    /// Declare a name in the innermost scope.
    ///
    /// # Arguments
    ///
    /// * `ident` - The name.
    /// * `kind` - Its kind, when known.
    /// * `access` - What it allows.
    ///
    /// # Returns
    ///
    /// `false` when the scope holds the name; nothing is reported.
    fn declare_quiet(
        &mut self, ident: &Ident, kind: Option<DataKind>, access: Access,
    ) -> bool
    {
        #[rustfmt::skip]
        let binding = Binding::builder().maybe_kind(kind).access(access).span(ident.span).build();
        self.scopes.declare(&ident.name, binding).is_ok()
    }

    /// Declare a name, reporting a second declaration in one scope.
    ///
    /// # Arguments
    ///
    /// * `ident` - The name.
    /// * `kind` - Its kind, when known.
    /// * `access` - What it allows.
    fn declare(&mut self, ident: &Ident, kind: Option<DataKind>, access: Access)
    {
        self.report_struct_name(ident);
        let fresh = self.declare_quiet(ident, kind, access);
        do_if!(!fresh, self.report(rd::redeclaration(ident.span, &ident.name)));
    }

    /// Report a value name that a struct has.
    ///
    /// # Arguments
    ///
    /// * `ident` - The name.
    fn report_struct_name(&mut self, ident: &Ident)
    {
        ret_if!(!self.structs.contains_key(&ident.name));
        self.report(rd::struct_name_clash(ident.span, &ident.name));
    }

    /// Declare a function parameter.
    ///
    /// # Arguments
    ///
    /// * `param` - The parameter.
    fn declare_param(&mut self, param: &Param)
    {
        let kind = self.written_kind(&param.kind, KindUse::Other);
        self.report_struct_name(&param.name);
        let fresh = self.declare_quiet(&param.name, Some(kind), Access::Var);
        let name = &param.name;
        do_if!(!fresh, self.report(rd::duplicate_param(name.span, &name.name)));
    }

    /// Infer `e` against an expected kind and report a value that does not
    /// convert to it.
    ///
    /// # Arguments
    ///
    /// * `e` - The value.
    /// * `want` - The kind its place expects, when known.
    pub fn expect_kind(&mut self, e: &Expr, want: Option<DataKind>)
    {
        let found = self.infer(e, want);
        let (Some(found), Some(want)) = (found, want)
        else
        {
            return;
        };
        self.expect_fit(e, found, want);
    }

    /// Report a value whose inferred kind does not convert to the kind its
    /// place expects.
    ///
    /// # Arguments
    ///
    /// * `e` - The value.
    /// * `found` - Its kind.
    /// * `want` - The kind its place expects.
    pub fn expect_fit(&mut self, e: &Expr, found: DataKind, want: DataKind)
    {
        ret_if!(found.widens_to(want, self.platform));
        ret_if!(self.report_flavour(e, found, want));
        self.report(mismatch_report(e.span(), want, found));
    }

    /// Report a value whose kind differs from the wanted one in its flavour
    /// alone: a borrow given for an owner, or an owner given for a borrow.
    /// A raw address never matches a checked one here; that mismatch is left
    /// to the caller. The shapes are compared as [`DataKind::meets_pointer`]
    /// does, so a borrow of a block of addresses given for an owner `ptr` is
    /// named as a flavour mistake, not as a block read as other data.
    ///
    /// # Arguments
    ///
    /// * `e` - The value.
    /// * `found` - Its kind.
    /// * `want` - The kind its place expects.
    ///
    /// # Returns
    ///
    /// `true` when a flavour diagnostic was reported.
    pub fn report_flavour(&mut self, e: &Expr, found: DataKind, want: DataKind)
    -> bool
    {
        let fits = |k: DataKind| k.meets_pointer(want);
        let borrow_for_owner = found.is_borrow() && want.is_owner_ptr();
        ret_if!(
            borrow_for_owner && fits(found.as_owner()),
            self.report(od::borrow_for_owner(e.span())),
            true
        );
        let owner_for_borrow = found.is_owner_ptr() && want.is_borrow();
        ret_if!(!owner_for_borrow || !fits(found.as_borrowed()), false);
        self.report(od::owner_for_borrow(e.span(), var_name(e)));
        true
    }

    /// Infer a condition and report one that is not a boolean.
    ///
    /// # Arguments
    ///
    /// * `e` - The condition.
    fn expect_bool(&mut self, e: &Expr)
    {
        let found = self.infer(e, Some(DataKind::Bool));
        let bad = found.filter(|k| *k != DataKind::Bool);
        do_if!(let Some(k) = bad, self.report(kd::condition_not_bool(e.span(), &k.name())));
    }

    /// Infer each expression with no expected kind.
    ///
    /// # Arguments
    ///
    /// * `list` - The expressions.
    ///
    /// # Returns
    ///
    /// `None`, for callers that give up on the result.
    pub fn infer_all(&mut self, list: &[Expr]) -> Option<DataKind>
    {
        do_for!(e in list, self.infer(e, None));
        None
    }

    /// Visit statements in the current scope.
    ///
    /// # Arguments
    ///
    /// * `stmts` - The statements.
    fn visit_stmts(&mut self, stmts: &[Stmt])
    {
        do_for!(s in stmts, self.visit_stmt(s))
    }

    /// Check a call against the function it names.
    ///
    /// # Arguments
    ///
    /// * `callee` - The function name.
    /// * `args` - The arguments.
    /// * `span` - The call.
    ///
    /// # Returns
    ///
    /// The result kind of the function (`Some(None)` when it has none), or
    /// `None` when the function is not declared.
    pub fn check_call(
        &mut self, callee: &Ident, args: &[Expr], span: Span,
    ) -> Option<Option<DataKind>>
    {
        let Some(sig) = self.fns.get(&callee.name).cloned()
        else
        {
            self.report(rd::undeclared_function(callee.span, &callee.name));
            self.infer_all(args);
            return None;
        };
        let (want, got) = (sig.params.len(), args.len());
        let bad_count = rd::arg_count_mismatch(span, &callee.name, want, got);
        let ret = Some(sig.ret);
        ret_if!(want != got, self.report(bad_count), self.infer_all(args), ret);
        // A parameter whose list count is open was reported at its function.
        let known = |kind: &DataKind| Some(*kind).filter(|k| !k.has_open_count());
        do_for!((arg, kind) in args.iter().zip(&sig.params), self.expect_kind(arg, known(kind)));
        ret
    }

    /// The kind of a counted-loop variable: the common kind of the bounds.
    ///
    /// # Arguments
    ///
    /// * `n` - The loop.
    ///
    /// # Returns
    ///
    /// The kind, or `None` after an error.
    fn counted_kind(&mut self, n: &CountedStmt) -> Option<DataKind>
    {
        let (from, to) = self.infer_sides(&n.from, &n.to, Some(DataKind::ISIZE));
        let from = from.and_then(|k| self.need_bound(k, &n.from));
        let to = to.and_then(|k| self.need_bound(k, &n.to));
        let span = n.from.span().to(&n.to.span());
        let kind = from.zip(to).and_then(|(a, b)| self.common_or_report(a, b, span));
        do_if!(let Some(step) = &n.step, self.check_step(step, kind));
        kind
    }

    /// Report a counted-loop bound or step that is not an integer.
    ///
    /// # Arguments
    ///
    /// * `kind` - Its kind.
    /// * `e` - The bound or step.
    ///
    /// # Returns
    ///
    /// The kind when it is an integer.
    fn need_bound(&mut self, kind: DataKind, e: &Expr) -> Option<DataKind>
    {
        ret_if!(kind.is_int(), Some(kind));
        self.report(kd::loop_bound_not_int(e.span(), &kind.name()));
        None
    }

    /// Check the step of a counted loop.
    ///
    /// # Arguments
    ///
    /// * `step` - The step.
    /// * `kind` - The kind of the loop variable, when known.
    fn check_step(&mut self, step: &Expr, kind: Option<DataKind>)
    {
        let found = self.infer(step, kind).and_then(|k| self.need_bound(k, step));
        let non_positive = non_positive_literal(step);
        do_if!(non_positive, self.report(kd::step_not_positive(step.span())));
        let (Some(found), Some(kind)) = (found, kind)
        else
        {
            return;
        };
        ret_if!(found.widens_to(kind, self.platform));
        self.report(kd::kind_mismatch(step.span(), &kind.name(), &found.name()));
    }

    /// Resolve the target of an assignment or a heap statement.
    ///
    /// # Arguments
    ///
    /// * `target` - The target.
    ///
    /// # Returns
    ///
    /// The kind of the place, or `None` after an error or for an unknown
    /// kind.
    fn place_kind(&mut self, target: &Expr) -> Option<DataKind>
    {
        let Some(place) = place_of(target)
        else
        {
            self.report(kd::not_a_place(target.span()));
            return None;
        };
        if let Expr::Name(n) = place
        {
            ret_if!(!self.check_writable(n), None);
            return self.infer(target, None);
        }
        let kind = self.infer(target, None);
        ret_if!(
            self.in_temp_list(place),
            self.report(kd::not_a_place(target.span())),
            None
        );
        let owner = self.owner_name(place);
        let access = owner.and_then(|n| Some((self.scopes.lookup(&n.name)?.access, n)));
        let diag = access.and_then(|(access, n)| access_error(access, n));
        do_if!(let Some(d) = diag, self.report(d));
        kind
    }

    /// Report a name that cannot be assigned: undeclared, a constant or a
    /// counted-loop variable.
    ///
    /// # Arguments
    ///
    /// * `n` - The name.
    ///
    /// # Returns
    ///
    /// `false` when the name is not declared.
    fn check_writable(&mut self, n: &NameExpr) -> bool
    {
        let Some(binding) = self.scopes.lookup(&n.name)
        else
        {
            self.report(rd::undeclared_name(n.span, &n.name));
            return false;
        };
        do_if!(let Some(diag) = access_error(binding.access, n), self.report(diag));
        true
    }

    /// Check the target of a heap statement: a place of a pointer kind.
    ///
    /// # Arguments
    ///
    /// * `target` - The target.
    fn heap_target(&mut self, target: &Expr)
    {
        let Some(kind) = self.place_kind(target)
        else
        {
            return;
        };
        ret_if!(kind.is_ptr());
        self.report(kd::heap_needs_pointer(target.span(), &kind.name()));
    }

    /// Check the size operand of a heap statement.
    ///
    /// # Arguments
    ///
    /// * `size` - The size.
    fn heap_size(&mut self, size: &SizeArg)
    {
        self.expect_kind(&size.count, Some(DataKind::USIZE));
        do_if!(let Some(unit) = &size.unit, let _ = self.written_kind(unit, KindUse::Other));
    }

    /// Run `f` with one more loop around it.
    ///
    /// # Arguments
    ///
    /// * `f` - The work inside the loop.
    fn in_loop(&mut self, f: impl FnOnce(&mut Self))
    {
        self.loop_depth += 1;
        f(self);
        self.loop_depth -= 1;
    }
}

/// Whether `e` is a literal step of zero or below.
///
/// # Arguments
///
/// * `e` - The step.
///
/// # Returns
///
/// `true` for `၀`, or `-` before an integer literal.
fn non_positive_literal(e: &Expr) -> bool
{
    match e
    {
        Expr::Int(n) => n.value == 0,
        Expr::Prefix(n) =>
        {
            n.op == PrefixOp::Neg && matches!(n.rhs.as_ref(), Expr::Int(_))
        }
        Expr::Group(n) => non_positive_literal(&n.inner),
        _ => false,
    }
}

/// The report of a value that does not fit its place. No conversion
/// turns a borrow into an owner or back, so a mismatch of flavours gets
/// no help. When the two kinds are owners or borrows whose blocks hold
/// addresses in different places, the conversion verb cannot bridge them
/// either, so the help names the raw address to go through. Each other
/// mismatch names the conversion verb when it can turn the value into the
/// wanted kind, and gets no help when it cannot, as for a list.
///
/// # Arguments
///
/// * `span` - The value.
/// * `want` - The kind its place expects.
/// * `found` - Its kind.
///
/// # Returns
///
/// The `ZK0401` diagnostic.
pub(crate) fn mismatch_report(span: Span, want: DataKind, found: DataKind)
-> Diagnostic
{
    let (want_name, found_name) = (want.name(), found.name());
    let flavours = want.is_checked_ptr() && found.is_checked_ptr();
    let flavours = flavours && want.is_borrow() != found.is_borrow();
    let unbridged = kd::kind_mismatch_unbridged(span, &want_name, &found_name);
    ret_if!(flavours, unbridged);
    ret_if!(
        found.reads_addresses_as(want),
        kd::address_block_mismatch(
            span,
            &want_name,
            &found_name,
            &want.with_outer_raw().name()
        )
    );
    tern!(
        found.cast_allowed(want),
        kd::kind_mismatch(span, &want_name, &found_name),
        unbridged
    )
}

/// The diagnostic for assigning to a name that does not allow it.
///
/// # Arguments
///
/// * `access` - What the name allows.
/// * `target` - The assigned name.
///
/// # Returns
///
/// The diagnostic, or `None` for a variable.
fn access_error(access: Access, target: &NameExpr) -> Option<Diagnostic>
{
    match access
    {
        Access::Var => None,
        Access::Const => Some(rd::assign_to_const(target.span, &target.name)),
        Access::LoopVar => Some(rd::assign_to_loop_var(target.span, &target.name)),
    }
}

impl Visitor for Checker
{
    fn visit_decl(&mut self, n: &DeclStmt)
    {
        let kind = self.decl_kind(&n.kind, n.init.as_ref());
        let missing = n.is_const && n.init.is_none();
        do_if!(missing, self.report(rd::const_without_init(n.name.span, &n.name.name)));
        let access = tern!(n.is_const, Access::Const, Access::Var);
        self.declare(&n.name, kind, access);
    }

    fn visit_assign(&mut self, n: &AssignStmt)
    {
        let want = self.place_kind(&n.target);
        self.expect_kind(&n.value, want);
    }

    fn visit_alloc(&mut self, n: &AllocStmt)
    {
        self.heap_size(&n.size);
        self.heap_target(&n.target);
    }

    fn visit_resize(&mut self, n: &ResizeStmt)
    {
        self.heap_target(&n.target);
        self.heap_size(&n.size);
    }

    fn visit_free(&mut self, n: &FreeStmt) { self.heap_target(&n.target) }

    fn visit_print(&mut self, n: &PrintStmt)
    {
        do_if!(let PrintArg::Expr(e) = &n.value, let _ = self.infer(e, None));
    }

    fn visit_block(&mut self, n: &BlockStmt)
    {
        self.scopes.push();
        self.visit_stmts(&n.stmts);
        self.scopes.pop();
    }

    fn visit_if_arm(&mut self, n: &IfArm)
    {
        self.expect_bool(&n.cond);
        self.visit_block(&n.block);
    }

    fn visit_while(&mut self, n: &WhileStmt)
    {
        self.expect_bool(&n.cond);
        self.in_loop(|c| c.visit_block(&n.block));
    }

    fn visit_counted(&mut self, n: &CountedStmt)
    {
        let kind = self.counted_kind(n);
        self.scopes.push();
        self.declare(&n.var, kind, Access::LoopVar);
        self.in_loop(|c| c.visit_stmts(&n.block.stmts));
        self.scopes.pop();
    }

    fn visit_generic(&mut self, n: &GenericStmt)
    {
        let kind = self.decl_kind(&n.kind, n.init.as_ref());
        self.scopes.push();
        self.declare(&n.var, kind, Access::Var);
        self.expect_bool(&n.cond);
        self.in_loop(|c| {
            c.visit_stmts(&n.block.stmts);
            do_if!(let Some(update) = &n.update, c.visit_stmt(update));
        });
        self.scopes.pop();
    }

    fn visit_break(&mut self, n: &BreakStmt)
    {
        do_if!(self.loop_depth == 0, self.report(rd::break_outside_loop(n.span)));
    }

    fn visit_continue(&mut self, n: &ContinueStmt)
    {
        do_if!(self.loop_depth == 0, self.report(rd::continue_outside_loop(n.span)));
    }

    fn visit_fn(&mut self, n: &FnDecl)
    {
        let depth = std::mem::take(&mut self.loop_depth);
        let ret = n.ret.as_ref().map(|k| self.written_kind(k, KindUse::Result));
        let ctx = FnCtx::builder().name(n.name.name.clone()).maybe_ret(ret).build();
        self.func = Some(ctx);
        self.scopes.push();
        do_for!(p in &n.params, self.declare_param(p));
        let result = Ident::builder().name(RESULT_NAME).span(n.name.span).build();
        do_if!(ret.is_some(), let _ = self.declare_quiet(&result, ret, Access::Var));
        self.visit_stmts(&n.block.stmts);
        self.scopes.pop();
        self.func = None;
        self.loop_depth = depth;
    }

    fn visit_return(&mut self, n: &ReturnStmt)
    {
        let Some(func) = self.func.clone()
        else
        {
            self.report(rd::return_outside_function(n.span));
            do_if!(let Some(v) = &n.value, let _ = self.infer(v, None));
            return;
        };
        match (func.ret, &n.value)
        {
            (None, Some(v)) =>
            {
                self.report(rd::value_returned_from_void(n.span, &func.name));
                self.infer(v, None);
            }
            (Some(k), None) =>
            {
                self.report(rd::missing_return_value(n.span, &func.name, &k.name()))
            }
            (Some(k), Some(v)) => self.expect_kind(v, Some(k)),
            (None, None) => (),
        }
    }

    fn visit_call_stmt(&mut self, n: &CallStmt)
    {
        self.check_call(&n.callee, &n.args, n.span);
    }

    fn visit_struct(&mut self, _: &StructDecl) {}
}
