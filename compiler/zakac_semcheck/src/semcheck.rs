use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;

use bon::Builder;
use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::NameExpr;
use zakac_ast::expr::NodeId;
use zakac_ast::expr::PrefixOp;
use zakac_ast::name::Name;
use zakac_ast::stmt::AddStmt;
use zakac_ast::stmt::AllocStmt;
use zakac_ast::stmt::AssignStmt;
use zakac_ast::stmt::BlitStmt;
use zakac_ast::stmt::BlockStmt;
use zakac_ast::stmt::BreakStmt;
use zakac_ast::stmt::CallStmt;
use zakac_ast::stmt::ContinueStmt;
use zakac_ast::stmt::CountedStmt;
use zakac_ast::stmt::DeclStmt;
use zakac_ast::stmt::EachStmt;
use zakac_ast::stmt::EnumDecl;
use zakac_ast::stmt::FillStmt;
use zakac_ast::stmt::FnDecl;
use zakac_ast::stmt::FreeStmt;
use zakac_ast::stmt::GenericStmt;
use zakac_ast::stmt::IfArm;
use zakac_ast::stmt::IfStmt;
use zakac_ast::stmt::ImportStmt;
use zakac_ast::stmt::Param;
use zakac_ast::stmt::PrintStmt;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::RemoveStmt;
use zakac_ast::stmt::ReplaceStmt;
use zakac_ast::stmt::ResizeStmt;
use zakac_ast::stmt::ReturnStmt;
use zakac_ast::stmt::SizeArg;
use zakac_ast::stmt::Stmt;
use zakac_ast::stmt::StructDecl;
use zakac_ast::stmt::SwitchStmt;
use zakac_ast::stmt::WhileStmt;
use zakac_ast::verb::VerbOp;
use zakac_ast::visit::Node;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::walk_stmt;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::kinds as kd;
use zakac_diagnostics::ownership as od;
use zakac_diagnostics::resolve as rd;
use zakac_lexer::token::Kw;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_false;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::ret_true;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::collect::collect_fns;
use crate::consts::ConstVal;
use crate::consts::CountTable;
use crate::consts::Folded;
use crate::consts::KnownConsts;
use crate::consts::open_structs;
use crate::consts::write_counts;
use crate::enum_check::KnownEnums;
use crate::enum_check::collect_enums;
use crate::foreign;
use crate::host;
use crate::host::IO_ERROR;
use crate::kind::DataKind;
use crate::list::KindUse;
use crate::ops::Want;
use crate::ops::kw_label;
use crate::overloads::Chosen;
use crate::overloads::write_overloads;
use crate::place::place_of;
use crate::place::var_name;
use crate::proof::Fact;
use crate::proof::changed_in;
use crate::proof::changed_in_stmt;
use crate::proof::facts_when;
use crate::proof::leaves_at_end;
use crate::scope::Access;
use crate::scope::Binding;
use crate::scope::FnSig;
use crate::scope::FnSigs;
use crate::scope::Scopes;
use crate::struct_check::collect_structs;
use crate::structs::StructTable;
use crate::units::StructHomes;
use crate::units::private_in_exports;
use crate::units::start_rules;
use crate::units::struct_homes;
use crate::units::unit_names;

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

/// Check names, scopes and data kinds of a program, and work out each list
/// count written as an expression. A count takes each expression that the
/// folder reduces to an integer; the check writes it back into the program as
/// digits, so each later stage sees a count of digits.
///
/// A count can wait for another count, through a size, or name a top-level
/// constant declared below it; so the check runs again while a run works out
/// a count or a constant it did not have, and the last run gives the output.
///
/// # Arguments
///
/// * `program` - The tree; its counts become digits.
/// * `options` - The options.
///
/// # Returns
///
/// The kind table and the diagnostics.
pub fn semcheck(program: &mut Program, options: &SemcheckOptions) -> SemcheckOutput
{
    let mut known = KnownConsts::new();
    let mut enums = KnownEnums::new();
    loop
    {
        let (mut out, counts, found, found_enums, chosen) =
            check_once(program, options, known.clone(), &enums);
        let written = write_counts(program, &counts);
        let settled = counts.is_empty() || (written == 0 && found == known);
        let enums_settled = found_enums == enums;
        enums = found_enums;
        if settled && enums_settled
        {
            out.diagnostics
                .extend(counts.into_values().filter_map(Result::err).flatten());
            out.diagnostics.sort_by_key(|d| d.span.lo);
            write_overloads(program, &chosen);
            return out;
        }
        known = found;
    }
}

/// Run the check once.
///
/// # Arguments
///
/// * `program` - The tree.
/// * `options` - The options.
/// * `known` - The top-level constants an earlier run found.
/// * `known_enums` - The enumeration values an earlier run worked out.
///
/// # Returns
///
/// The output, the list counts the run met, the top-level constants it
/// found, the enumeration values it worked out and the function each call of
/// an overloaded name chose.
fn check_once(
    program: &Program, options: &SemcheckOptions, known: KnownConsts,
    known_enums: &KnownEnums,
) -> (SemcheckOutput, CountTable, KnownConsts, KnownEnums, Chosen)
{
    let mut diags = Vec::new();
    let enums = collect_enums(program, known_enums, &mut diags);
    let mut structs = collect_structs(program, &enums, &mut diags);
    structs.extend(enums);
    let unit_fns = collect_fns(program, &structs, &mut diags);
    let homes = struct_homes(program);
    diags.extend(private_in_exports(program, &homes));
    diags.extend(start_rules(program));
    #[rustfmt::skip]
    let mut checker = Checker::builder().fns(unit_fns[0].clone()).unit_fns(unit_fns)
        .homes(homes).unit_names(unit_names(program)).structs(structs)
        .diags(diags).platform(options.platform_bits).no_raw(options.no_raw)
        .known(known).open_structs(open_structs(program)).build();
    checker.scopes.push();
    checker.visit_program(program);
    checker.diags.sort_by_key(|d| d.span.lo);
    #[rustfmt::skip]
    let out = SemcheckOutput::builder().kinds(checker.kinds).structs(checker.structs).diagnostics(checker.diags).build();
    (out, checker.counts, checker.found, checker.found_enums, checker.chosen)
}

/// The function whose block is being checked.
#[derive(Debug, Clone, Builder)]
pub(crate) struct FnCtx
{
    /// The function name.
    pub name: String,
    /// The result kind; `None` for a function without a result.
    pub ret: Option<DataKind>,
    /// The result kind it gives back when it can fail.
    pub fault: Option<DataKind>,
}

/// State of the walk.
#[derive(Builder)]
pub(crate) struct Checker
{
    /// Names in scope.
    #[builder(default)]
    pub scopes: Scopes,
    /// The functions the current file declares or imports.
    pub fns: FnSigs,
    /// The function each call of an overloaded name chose, under the span
    /// of its name at the call.
    #[builder(default)]
    pub chosen: Chosen,
    /// The functions each file declares.
    pub unit_fns: Vec<FnSigs>,
    /// The top-level names of each file checked so far.
    #[builder(default)]
    pub saved: Vec<HashMap<Name, Binding>>,
    /// The file of each struct.
    #[builder(default)]
    pub homes: StructHomes,
    /// The name of each file, for messages.
    #[builder(default)]
    pub unit_names: Vec<String>,
    /// The unit of the file being checked.
    #[builder(default)]
    pub unit: u32,
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
    /// The part each result variable is proven to hold here: `true` for a
    /// good result.
    #[builder(default)]
    pub proven: HashMap<Name, bool>,
    /// The variables that statements changed since the statement around
    /// them started, so what was proven about them before ends.
    #[builder(default)]
    pub touched: HashSet<Name>,
    /// Bit count of the platform-sized integer kinds.
    pub platform: u16,
    /// The parameter and result kinds of each C function name declared so
    /// far, in each file.
    #[builder(default)]
    pub c_names: HashMap<String, (Vec<DataKind>, Option<DataKind>)>,
    /// Whether kinds with a raw address level are refused.
    #[builder(default)]
    pub no_raw: bool,
    /// Whether the value checked now is the value of a declaration, where a
    /// counted list takes `[]`.
    #[builder(default)]
    pub decl_value: bool,
    /// Each list count written as an expression that this run met.
    #[builder(default)]
    pub counts: CountTable,
    /// The top-level constants with a value that an earlier run found.
    #[builder(default)]
    pub known: KnownConsts,
    /// The top-level constants with a value that this run finds.
    #[builder(default)]
    pub found: KnownConsts,
    /// The structs whose size waits for a count still open.
    #[builder(default)]
    pub open_structs: HashSet<Name>,
    /// The value of each constant list or struct this run found.
    #[builder(default)]
    pub aggs: Vec<Folded>,
    /// The enumeration values this run works out.
    #[builder(default)]
    pub found_enums: KnownEnums,
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
        self.scopes.declare(ident.name, binding).is_ok()
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

    /// Forget what was proven about variables that can change.
    ///
    /// # Arguments
    ///
    /// * `names` - The variables.
    fn forget(&mut self, names: HashSet<Name>)
    {
        do_for!(name in &names, _ = self.proven.remove(name));
        self.touched.extend(names);
    }

    /// Visit the parts of a loop: what the loop can change is forgotten
    /// before it, since a later round sees it; what its condition proves
    /// holds inside it, and is dropped after it.
    ///
    /// # Arguments
    ///
    /// * `changed` - The variables the loop can change.
    /// * `cond` - The condition, for a loop with one.
    /// * `f` - The work.
    fn proven_loop(
        &mut self, changed: HashSet<Name>, cond: Option<&Expr>,
        f: impl FnOnce(&mut Self),
    )
    {
        let facts = cond.map_or_else(Vec::new, |c| facts_when(c, true));
        let facts: Vec<Fact> =
            facts.into_iter().filter(|(n, _)| !changed.contains(n)).collect();
        self.forget(changed);
        let before = self.proven.clone();
        self.proven.extend(facts);
        f(self);
        self.proven = before;
    }

    /// Check a call against the function it names.
    ///
    /// # Arguments
    ///
    /// * `callee` - The function name, or the first name of the place.
    /// * `place` - The place that holds the function value, when written.
    /// * `args` - The arguments.
    /// * `edits` - Whether the edit mark follows each argument.
    /// * `span` - The call.
    ///
    /// # Returns
    ///
    /// The result kind of the function (`Some(None)` when it has none), or
    /// `None` when the call names no function it can run.
    pub fn check_call(
        &mut self, callee: &Ident, place: Option<&Expr>, args: &[Expr], edits: &[bool],
        marks: &[Ident], span: Span,
    ) -> Option<Option<DataKind>>
    {
        let Some(sigs) = self.call_target(callee, place)
        else
        {
            self.infer_all(args);
            return None;
        };
        let Some(sig) = self.choose(callee, args, marks, &sigs, span)
        else
        {
            self.infer_all(args);
            return None;
        };
        let (want, got) = (sig.params.len(), args.len());
        let bad_count = rd::arg_count_mismatch(span, &callee.name, want, got);
        let ret = Some(sig.ret);
        let wrong = tern!(sig.variadic, got < want, got != want);
        ret_if!(wrong, self.report(bad_count), self.infer_all(args), ret);
        do_for!(arg in &args[want..], self.check_rest_arg(arg));
        let marked = |n: usize| edits.get(n).copied().unwrap_or(false);
        for (n, (arg, kind)) in args.iter().zip(&sig.params).enumerate()
        {
            let edit = sig.edits.get(n).copied().unwrap_or(false);
            self.check_arg(arg, *kind, edit, marked(n));
        }
        self.check_edits_apart(args, edits);
        self.check_marks(callee, marks, sig.marks.as_deref());
        ret
    }

    /// Check an argument past the parameters of a C function that takes
    /// other values: its kind is the one it has on its own, and must cross
    /// to C as an argument of a C `...` does (ZK0465).
    ///
    /// # Arguments
    ///
    /// * `arg` - The argument.
    fn check_rest_arg(&mut self, arg: &Expr)
    {
        let kind = self.infer(arg, None);
        let platform = self.platform;
        let stuck = kind.filter(|k| !foreign::crosses_rest(*k, platform));
        do_if!(let Some(k) = stuck, self.report(kd::kind_not_across_c(arg.span(), &k.name())));
    }

    /// Check that an external function is one of the functions the host
    /// serves, or, with a lib, a C function.
    ///
    /// # Arguments
    ///
    /// * `n` - The declaration.
    /// * `ret` - Its result kind.
    fn check_external(&mut self, n: &FnDecl, ret: Option<DataKind>)
    {
        let params: Vec<DataKind> = n
            .params
            .iter()
            .map(|p| DataKind::from_ast(&p.kind, &self.structs))
            .collect();
        ret_if!(n.lib.is_some(), self.check_c_fn(n, params, ret));
        // The host serves no function that takes other values.
        ret_if!(!n.variadic && host::find(&n.name.name, &params, ret).is_some());
        let diag =
            rd::unknown_host_fn(n.name.span, &n.name.name, &host::listing(), IO_ERROR);
        self.report(diag);
    }

    /// Check a C function: its name is a C name (ZK0337), each parameter and
    /// the result cross to C (ZK0465), no parameter carries the edit mark
    /// (ZK0466), and each declaration of the name gives the same kinds
    /// (ZK0338).
    ///
    /// # Arguments
    ///
    /// * `n` - The declaration.
    /// * `params` - Its parameter kinds.
    /// * `ret` - Its result kind.
    fn check_c_fn(&mut self, n: &FnDecl, params: Vec<DataKind>, ret: Option<DataKind>)
    {
        let name = n.name.name.to_string();
        do_if!(
            !foreign::is_c_name(&name),
            self.report(rd::not_a_c_name(n.name.span, &name))
        );
        let platform = self.platform;
        let pairs = n.params.iter().zip(&params);
        let stuck = pairs.filter(|(_, k)| !foreign::crosses_param(**k, platform));
        let stuck: Vec<_> =
            stuck.map(|(p, k)| kd::kind_not_across_c(p.span, &k.name())).collect();
        do_for!(d in stuck, self.report(d));
        let edits = n.params.iter().filter(|p| p.edit);
        let edits: Vec<_> =
            edits.map(|p| kd::edit_across_c(p.span, &p.name.name)).collect();
        do_for!(d in edits, self.report(d));
        let bad_ret = ret.filter(|k| !foreign::crosses_back(*k, platform));
        do_if!(let Some(k) = bad_ret, self.report(kd::kind_not_across_c(n.name.span, &k.name())));
        let shape = (params, ret);
        let first = self.c_names.entry(name.clone()).or_insert_with(|| shape.clone());
        do_if!(*first != shape, self.report(rd::c_name_twice(n.name.span, &name)));
    }

    /// Choose the function a call runs among the functions of its name: the
    /// one there is, or the one overload whose parameter count, marks (when
    /// the call writes marks) and parameter kinds the arguments fit, as the
    /// check of an argument against its parameter accepts it. No fit is
    /// ZK0332 and more than one is ZK0333.
    ///
    /// # Arguments
    ///
    /// * `callee` - The function name at the call.
    /// * `args` - The arguments.
    /// * `marks` - The marks written; blank for the other call forms.
    /// * `sigs` - The functions of the name.
    /// * `span` - The call.
    ///
    /// # Returns
    ///
    /// The function, or `None` after a report.
    fn choose(
        &mut self, callee: &Ident, args: &[Expr], marks: &[Ident], sigs: &[FnSig],
        span: Span,
    ) -> Option<FnSig>
    {
        ret_if!(sigs.len() == 1, sigs.first().cloned());
        let fits: Vec<&FnSig> =
            sigs.iter().filter(|s| self.fits_call(s, args, marks)).collect();
        let list = || sigs.iter().map(params_text).collect::<Vec<_>>().join(" / ");
        match fits.as_slice()
        {
            [one] =>
            {
                self.chosen.insert(callee.span, one.inner);
                Some((*one).clone())
            }
            [] =>
            {
                self.report(rd::no_overload_fits(span, &callee.name, &list()));
                None
            }
            _ =>
            {
                self.report(rd::overload_unclear(span, &callee.name));
                None
            }
        }
    }

    /// Whether a call fits a function: an argument for each parameter, the
    /// marks of the parameters when the call writes marks, and each argument
    /// fit for its parameter with no report, which the check gives back.
    ///
    /// # Arguments
    ///
    /// * `sig` - The function.
    /// * `args` - The arguments.
    /// * `marks` - The marks written.
    ///
    /// # Returns
    ///
    /// `true` when it fits.
    fn fits_call(&mut self, sig: &FnSig, args: &[Expr], marks: &[Ident]) -> bool
    {
        let short = args.len() < sig.params.len();
        ret_false!(tern!(sig.variadic, short, sig.params.len() != args.len()));
        let named =
            |want: &Vec<Name>| marks.iter().map(|m| m.name).eq(want.iter().copied());
        ret_false!(!marks.is_empty() && !sig.marks.as_ref().is_some_and(named));
        let before = self.diags.len();
        args.iter()
            .zip(&sig.params)
            .for_each(|(arg, kind)| self.expect_kind(arg, Some(*kind)));
        let clean = self.diags.len() == before;
        self.diags.truncate(before);
        clean
    }

    /// Check that each argument of a marked call carries the mark of its
    /// parameter.
    ///
    /// # Arguments
    ///
    /// * `callee` - The function name.
    /// * `marks` - The marks written; blank for the other call forms.
    /// * `want` - The marks of the parameters, when the parameters have marks.
    fn check_marks(&mut self, callee: &Ident, marks: &[Ident], want: Option<&[Name]>)
    {
        let Some(want) = want
        else
        {
            return;
        };
        let wrong =
            marks.iter().zip(want).enumerate().filter(|(_, (m, w))| m.name != **w);
        let found: Vec<Diagnostic> = wrong
            .map(|(n, (m, w))| rd::wrong_mark(m.span, &callee.name, n + 1, &m.name, w))
            .collect();
        do_for!(diag in found, self.report(diag));
    }

    /// Check how a call meets a failure of the function: a call statement
    /// to a function that can fail passes the failure on, a pass-on stands
    /// inside a function that fails with the same error kind, and a call to
    /// a function that cannot fail passes nothing on. A call expression
    /// without a pass-on gives the whole result.
    ///
    /// # Arguments
    ///
    /// * `callee` - The function name.
    /// * `ret` - The result kind of the function the call runs.
    /// * `pass` - Whether the call passes a failure on.
    /// * `dropped` - Whether the result is dropped: a call statement.
    /// * `span` - The call.
    pub fn check_pass(
        &mut self, callee: &Ident, ret: Option<DataKind>, pass: bool, dropped: bool,
        span: Span,
    )
    {
        let name = &callee.name;
        let Some((_, found)) = ret.and_then(DataKind::fallible_parts)
        else
        {
            return do_if!(pass, self.report(kd::pass_without_failure(span, name)));
        };
        ret_if!(
            !pass,
            do_if!(dropped, self.report(kd::failure_not_passed(span, name)))
        );
        let own = self.func.as_ref().and_then(|f| f.fault);
        let Some((_, want)) = own.and_then(DataKind::fallible_parts)
        else
        {
            return self.report(kd::pass_outside_failing(span));
        };
        let differs = kd::fault_kind_differs(span, &err_name(found), &err_name(want));
        do_if!(found != want, self.report(differs));
    }

    /// Check `... ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။` against the error kind of
    /// the function.
    ///
    /// # Arguments
    ///
    /// * `n` - The return statement.
    /// * `func` - The function around it.
    fn check_fault_return(&mut self, n: &ReturnStmt, func: &FnCtx)
    {
        let Some((_, err)) = func.fault.and_then(DataKind::fallible_parts)
        else
        {
            self.report(kd::fault_from_sure(n.span, &func.name));
            do_if!(let Some(v) = &n.value, _ = self.infer(v, None));
            return;
        };
        match (&n.value, err)
        {
            (Some(v), Some(err)) => self.expect_kind(v, Some(err)),
            (Some(v), None) =>
            {
                self.report(kd::fault_value_extra(n.span, &func.name));
                self.infer(v, None);
            }
            (None, Some(err)) =>
            {
                self.report(kd::fault_value_missing(n.span, &err.name()))
            }
            (None, None) => (),
        }
    }

    /// Check one argument against its parameter: a value that converts to
    /// the parameter kind, or for a parameter the function can change, a
    /// writable place of that kind alone, with the mark on both ends.
    ///
    /// # Arguments
    ///
    /// * `arg` - The argument.
    /// * `kind` - The parameter kind.
    /// * `edit` - Whether the function can change the parameter.
    /// * `mark` - Whether the edit mark follows the argument.
    fn check_arg(&mut self, arg: &Expr, kind: DataKind, edit: bool, mark: bool)
    {
        do_if!(edit && !mark, self.report(kd::edit_mark_missing(arg.span())));
        do_if!(!edit && mark, self.report(kd::edit_mark_extra(arg.span())));
        if !(edit && mark)
        {
            return self.expect_kind(arg, Some(kind));
        }
        let Some(found) = self.place_kind(arg)
        else
        {
            return;
        };
        do_if!(found != kind, self.report(mismatch_report(arg.span(), kind, found)));
    }

    /// Report each argument of a call that names a place another argument
    /// lets the function change: the function would read the place while it
    /// changes it.
    ///
    /// # Arguments
    ///
    /// * `args` - The arguments.
    /// * `edits` - Whether the edit mark follows each argument.
    fn check_edits_apart(&mut self, args: &[Expr], edits: &[bool])
    {
        let marked = args.iter().zip(edits).filter(|(_, m)| **m).map(|(a, _)| a);
        let roots: Vec<(Name, Span)> = marked
            .filter_map(|a| place_of(a).and_then(|p| self.owner_name(p)))
            .map(|n| (n.name, n.span))
            .collect();
        for (root, at) in roots
        {
            let others = args.iter().filter(|a| !a.span().contains(&at));
            let clash = others.filter(|a| names_in(a).contains(&root));
            let found: Vec<Span> = clash.map(Node::span).collect();
            do_for!(span in found, self.report(kd::edit_shared(span, &root)));
        }
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

    /// The kind of the variable of an each loop: a character for text, a
    /// cell for a list or a slice whose cells hold no owner pointer, or for
    /// the list a pointer points at, as a cell read takes it.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of the value the loop walks.
    /// * `n` - The loop.
    ///
    /// # Returns
    ///
    /// The kind, or `None` after a report.
    fn each_var_kind(&mut self, kind: DataKind, n: &EachStmt) -> Option<DataKind>
    {
        let cell = each_list(kind, n.chars).cell_kind();
        let walked = tern!(
            n.chars,
            kind.is_textual().then_some(DataKind::Char),
            cell.filter(|c| !kind.is_text_view() && !c.holds_owner_ptr())
        );
        ret_if!(walked.is_some(), walked);
        self.report(kd::each_source_wrong(n.source.span(), &kind.name()));
        None
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
        let text =
            tern!(let Expr::Index(cell) = place, self.kinds.get(&cell.list.id()), None);
        let text = text.copied().filter(|k| k.is_textual());
        ret_if!(
            let Some(k) = text,
            self.report(kd::text_cell_write(target.span(), &k.name())),
            None
        );
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

    /// Check the list an add or a remove statement changes: a place that
    /// holds a plain list.
    ///
    /// # Arguments
    ///
    /// * `list` - The list.
    /// * `kind` - Its kind.
    ///
    /// # Returns
    ///
    /// The kind of its cells, or `None` after an error.
    fn growing_cells(&mut self, list: &Expr, kind: DataKind) -> Option<DataKind>
    {
        let cell = kind.as_seq().map(|s| s.cell);
        do_if!(
            cell.is_none(),
            self.report(kd::list_cannot_grow(list.span(), &kind.name()))
        );
        cell
    }

    /// Check an add to text: the text can change, and the value is text, a
    /// view of text or a character.
    ///
    /// # Arguments
    ///
    /// * `text` - The text that grows, a place.
    /// * `kind` - Its kind.
    /// * `value` - What is added.
    fn text_add(&mut self, text: &Expr, kind: DataKind, value: &Expr)
    {
        self.changeable_text(text, kind);
        let Some(found) = self.infer(value, None)
        else
        {
            return;
        };
        ret_if!(found.is_textual() || found == DataKind::Char);
        self.report(mismatch_report(value.span(), DataKind::TEXT, found));
    }

    /// Check an add that puts its value in front of a cell of a plain list or
    /// a character of text: the list or the text is a place that can grow,
    /// the number an integer, and the value fits a cell or is text, a view of
    /// text or a character.
    ///
    /// # Arguments
    ///
    /// * `at` - The cell or the character.
    /// * `value` - The value added.
    fn insert(&mut self, at: &Expr, value: &Expr)
    {
        if let Some((text, index)) = char_target(at)
        {
            let kind = self.place_kind(text);
            self.cell_number(index);
            let Some(kind) = kind.filter(|k| self.is_text_place(*k, text))
            else
            {
                _ = self.infer(value, None);
                return;
            };
            return self.text_add(text, kind, value);
        }
        let Expr::Index(cell) = at
        else
        {
            self.report(kd::insert_not_at_cell(at.span()));
            _ = self.infer(value, None);
            return;
        };
        let kind = self.place_kind(&cell.list);
        self.cell_number(&cell.index);
        let cells = kind.and_then(|k| self.growing_cells(&cell.list, k));
        self.expect_kind(value, cells);
    }

    /// Check the text a remove of the last character changes.
    ///
    /// # Arguments
    ///
    /// * `text` - The text.
    /// * `kind` - Its kind.
    fn shrinking_text(&mut self, text: &Expr, kind: DataKind)
    {
        do_if!(self.is_text_place(kind, text), self.changeable_text(text, kind));
    }

    /// Whether a place a character statement names is text or a view of it,
    /// with a report for each other kind.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of the place.
    /// * `text` - The place.
    ///
    /// # Returns
    ///
    /// `true` for text or a view of text.
    fn is_text_place(&mut self, kind: DataKind, text: &Expr) -> bool
    {
        ret_true!(kind.is_textual());
        let label = kw_label(Kw::CharKind);
        _ = self.need(kind, Want::Text, &label, text);
        false
    }

    /// Check the number of a cell or a character: an integer.
    ///
    /// # Arguments
    ///
    /// * `index` - The number.
    fn cell_number(&mut self, index: &Expr)
    {
        let at = self.infer(index, Some(DataKind::USIZE));
        let bad = at.filter(|k| !k.is_int());
        do_if!(let Some(k) = bad, self.report(kd::cell_number_not_int(index.span(), &k.name())));
    }

    /// Check a write of one character of text: the text is a place that can
    /// change, the number an integer and the value a character.
    ///
    /// # Arguments
    ///
    /// * `text` - The text.
    /// * `index` - The number of the character.
    /// * `value` - The new character.
    fn char_write(&mut self, text: &Expr, index: &Expr, value: &Expr)
    {
        let kind = self.place_kind(text);
        self.cell_number(index);
        self.expect_kind(value, Some(DataKind::Char));
        let Some(kind) = kind
        else
        {
            return;
        };
        do_if!(self.is_text_place(kind, text), self.changeable_text(text, kind));
    }

    /// Report a view of text that a statement would change.
    ///
    /// # Arguments
    ///
    /// * `text` - The text.
    /// * `kind` - Its kind, textual.
    fn changeable_text(&mut self, text: &Expr, kind: DataKind)
    {
        let view = kind.is_text_view();
        do_if!(view, self.report(kd::text_cannot_change(text.span(), &kind.name())));
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
            self.report(
                self.with_hidden(rd::undeclared_name(n.span, &n.name), &n.name),
            );
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

    /// Check an address that a blit or a fill reads or writes through: a
    /// value of a pointer kind, which need not be a place.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address.
    ///
    /// # Returns
    ///
    /// The kind of the address, or `None` after a report.
    fn heap_address(&mut self, addr: &Expr) -> Option<DataKind>
    {
        let kind = self.infer(addr, None)?;
        ret_if!(kind.is_ptr(), Some(kind));
        self.report(kd::heap_needs_pointer(addr.span(), &kind.name()));
        None
    }

    /// Check the size operand of a heap statement.
    ///
    /// # Arguments
    ///
    /// * `size` - The size.
    ///
    /// # Returns
    ///
    /// The kind of each unit, when the size counts a kind.
    fn heap_size(&mut self, size: &SizeArg) -> Option<DataKind>
    {
        self.expect_kind(&size.count, Some(DataKind::USIZE));
        size.unit.as_ref().map(|unit| self.written_kind(unit, KindUse::Other))
    }

    /// Report the cells of a blit or a fill that own a block or borrow one: the
    /// unit kind of the size, or each kind the addresses point at for a
    /// size in octets.
    ///
    /// # Arguments
    ///
    /// * `size` - The size.
    /// * `unit` - The unit kind of the size.
    /// * `addrs` - The kinds of the addresses.
    fn owned_cells(
        &mut self, size: &SizeArg, unit: Option<DataKind>, addrs: &[Option<DataKind>],
    )
    {
        let pointees = || addrs.iter().flatten().filter_map(|a| a.pointee()).collect();
        let cells: Vec<DataKind> = tern!(let Some(u) = unit, vec![u], pointees());
        let held =
            |k: &DataKind| k.holds_owner_ptr() || k.holds_text() || k.holds_borrow();
        let owner = cells.into_iter().find(held);
        do_if!(let Some(k) = owner, self.report(kd::blit_of_held_cells(size.span, &k.name())));
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
/// wanted kind, and gets no help when it cannot, as for a list. A list or
/// text where a slice or a view of the same cells is wanted gets help that
/// names the lend.
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
    ret_if!(lends_to(found, want), kd::lend_wanted(span, &want_name, &found_name));
    // Two lists of different forms differ in their count, not in the
    // addresses their blocks hold.
    let lists = [want, found]
        .iter()
        .all(|k| k.pointee().and_then(DataKind::cell_kind).is_some());
    ret_if!(
        found.reads_addresses_as(want) && !lists,
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

/// Whether a lend of a value of one kind gives a value of another: a list
/// lends a slice of its cells, and text a view.
///
/// # Arguments
///
/// * `found` - The kind of the value.
/// * `want` - The kind wanted.
///
/// # Returns
///
/// `true` when the lend gives `want`.
fn lends_to(found: DataKind, want: DataKind) -> bool
{
    ret_if!(found.is_text(), want.is_text_view());
    let cell = want.as_slice().filter(|s| !s.text).map(|s| s.cell);
    cell.is_some() && found.cell_kind() == cell && !found.is_slice()
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

/// The names an expression reads.
///
/// # Arguments
///
/// * `e` - The expression.
///
/// # Returns
///
/// The names, in source order.
pub(crate) fn names_in(e: &Expr) -> Vec<Name>
{
    /// The walker that collects the names.
    struct Names(Vec<Name>);
    impl Visitor for Names
    {
        fn visit_name(&mut self, n: &NameExpr) { self.0.push(n.name) }
    }
    let mut found = Names(Vec::new());
    found.visit_expr(e);
    found.0
}

impl Visitor for Checker
{
    fn visit_decl(&mut self, n: &DeclStmt)
    {
        let kind = self.decl_kind(n.kind.as_ref(), n.init.as_ref(), &n.name);
        let missing = n.is_const && n.init.is_none();
        do_if!(missing, self.report(rd::const_without_init(n.name.span, &n.name.name)));
        let access = tern!(n.is_const, Access::Const, Access::Var);
        // A new variable of this name hides what was proven about the old.
        self.proven.remove(&n.name.name);
        self.declare(&n.name, kind, access);
        do_if!(n.exported, self.scopes.mark_exported(&n.name.name));
        let init = n.init.as_ref().filter(|_| n.is_const);
        let value = init.map_or(ConstVal::Unknown, |e| self.const_value(e, kind));
        self.scopes.set_value(&n.name.name, value);
        let top = self.scopes.depth() == 1 && n.is_const;
        do_if!(
            top,
            _ = self.found.insert((self.unit, n.name.name), (kind, self.seen(value)))
        );
    }

    fn visit_program(&mut self, n: &Program)
    {
        // A member value can name a constant declared below its
        // enumeration, so the values of a file are worked out once its
        // top-level names are all in scope: before the walk leaves the file.
        let mut pending: Vec<&EnumDecl> = Vec::new();
        for (i, s) in n.stmts.iter().enumerate()
        {
            let leaving = n.unit_of(i) != self.unit;
            do_if!(leaving, do_for!(e in pending.drain(..), self.check_enum(e)));
            self.visit_top(n, i, s);
            do_if!(let Stmt::Enum(e) = s, pending.push(e));
        }
        do_for!(e in pending.drain(..), self.check_enum(e));
    }

    fn visit_import(&mut self, n: &ImportStmt) { self.import(n) }

    fn visit_assign(&mut self, n: &AssignStmt)
    {
        ret_if!(let Some((text, index)) = char_target(&n.target), self.char_write(text, index, &n.value));
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

    fn visit_blit(&mut self, n: &BlitStmt)
    {
        let src = self.heap_address(&n.src);
        let unit = self.heap_size(&n.size);
        let dst = self.heap_address(&n.dst);
        self.owned_cells(&n.size, unit, &[src, dst]);
    }

    fn visit_fill(&mut self, n: &FillStmt)
    {
        let dst = self.heap_address(&n.dst);
        let unit = self.heap_size(&n.size);
        self.expect_kind(&n.value, Some(unit.unwrap_or(DataKind::U8)));
        self.owned_cells(&n.size, unit, &[dst]);
    }

    fn visit_add(&mut self, n: &AddStmt)
    {
        ret_if!(n.before, self.insert(&n.list, &n.value));
        let kind = self.place_kind(&n.list);
        ret_if!(let Some(k) = kind.filter(|k| k.is_textual()), self.text_add(&n.list, k, &n.value));
        let cell = kind.and_then(|k| self.growing_cells(&n.list, k));
        self.expect_kind(&n.value, cell);
    }

    fn visit_remove(&mut self, n: &RemoveStmt)
    {
        let kind = self.place_kind(&n.list);
        ret_if!(n.chars, do_if!(let Some(k) = kind, self.shrinking_text(&n.list, k)));
        do_if!(let Some(k) = kind, { _ = self.growing_cells(&n.list, k); });
    }

    fn visit_replace(&mut self, n: &ReplaceStmt)
    {
        let kind = self.place_kind(&n.text);
        self.cell_number(&n.start);
        self.cell_number(&n.end);
        let Some(kind) = kind.filter(|k| self.is_text_place(*k, &n.text))
        else
        {
            _ = self.infer(&n.value, None);
            return;
        };
        self.text_add(&n.text, kind, &n.value);
    }

    fn visit_print(&mut self, n: &PrintStmt) { _ = self.infer(&n.value, None); }

    fn visit_block(&mut self, n: &BlockStmt)
    {
        self.scopes.push();
        self.visit_stmts(&n.stmts);
        self.scopes.pop();
    }

    fn visit_stmt(&mut self, n: &Stmt)
    {
        walk_stmt(self, n);
        self.forget(changed_in_stmt(n));
    }

    fn visit_if(&mut self, n: &IfStmt)
    {
        let before = self.proven.clone();
        let outer = std::mem::take(&mut self.touched);
        let mut ruled_out: Vec<Fact> = Vec::new();
        for arm in &n.arms
        {
            self.proven = before.clone();
            self.proven.extend(ruled_out.iter().copied());
            self.expect_bool(&arm.cond);
            self.proven.extend(facts_when(&arm.cond, true));
            self.visit_block(&arm.block);
            ruled_out.extend(facts_when(&arm.cond, false));
        }
        self.proven = before.clone();
        self.proven.extend(ruled_out.iter().copied());
        do_if!(let Some(b) = &n.else_block, self.visit_block(b));
        let inside = std::mem::replace(&mut self.touched, outer);
        let mut after = before;
        after.retain(|name, _| !inside.contains(name));
        // A lone arm that leaves at its end proves the opposite of its test
        // after the statement.
        let guard = n.arms.len() == 1 && n.else_block.is_none();
        let guard = guard && n.arms.first().is_some_and(|a| leaves_at_end(&a.block));
        do_if!(guard, after.extend(facts_when(&n.arms[0].cond, false)));
        self.proven = after;
        self.touched.extend(inside);
    }

    fn visit_switch(&mut self, n: &SwitchStmt)
    {
        let subject = self.infer(&n.subject, None);
        let before = self.proven.clone();
        let outer = std::mem::take(&mut self.touched);
        let mut seen = Vec::new();
        for arm in &n.arms
        {
            self.proven = before.clone();
            do_for!(case in &arm.cases, self.check_case(case, (&n.subject, subject), &mut seen));
            self.visit_block(&arm.block);
        }
        self.proven = before.clone();
        self.check_default(n, subject, &seen);
        do_if!(let Some(d) = &n.default, self.visit_block(&d.block));
        let inside = std::mem::replace(&mut self.touched, outer);
        let mut after = before;
        after.retain(|name, _| !inside.contains(name));
        self.proven = after;
        self.touched.extend(inside);
    }

    /// The values are worked out once the file has its top-level names, see
    /// `visit_program`.
    fn visit_enum(&mut self, _n: &EnumDecl) {}

    fn visit_if_arm(&mut self, n: &IfArm)
    {
        self.expect_bool(&n.cond);
        self.visit_block(&n.block);
    }

    fn visit_while(&mut self, n: &WhileStmt)
    {
        let changed = changed_in(&n.block.stmts);
        self.forget(changed.clone());
        self.expect_bool(&n.cond);
        self.proven_loop(changed, Some(&n.cond), |c| {
            c.in_loop(|c| c.visit_block(&n.block))
        });
    }

    fn visit_counted(&mut self, n: &CountedStmt)
    {
        let kind = self.counted_kind(n);
        self.scopes.push();
        self.declare(&n.var, kind, Access::LoopVar);
        let changed = changed_in(&n.block.stmts);
        self.proven_loop(changed, None, |c| {
            c.in_loop(|c| c.visit_stmts(&n.block.stmts))
        });
        self.scopes.pop();
    }

    fn visit_each(&mut self, n: &EachStmt)
    {
        let kind = self.infer(&n.source, None).and_then(|k| self.each_var_kind(k, n));
        self.scopes.push();
        self.declare(&n.var, kind, Access::Var);
        let changed = changed_in(&n.block.stmts);
        self.proven_loop(changed, None, |c| {
            c.in_loop(|c| c.visit_stmts(&n.block.stmts))
        });
        self.scopes.pop();
    }

    fn visit_generic(&mut self, n: &GenericStmt)
    {
        let kind = self.decl_kind(n.kind.as_ref(), n.init.as_ref(), &n.var);
        self.scopes.push();
        self.declare(&n.var, kind, Access::Var);
        let mut changed = changed_in(&n.block.stmts);
        changed.extend(n.update.as_deref().map(changed_in_stmt).unwrap_or_default());
        changed.insert(n.var.name);
        self.forget(changed.clone());
        self.expect_bool(&n.cond);
        self.proven_loop(changed, Some(&n.cond), |c| {
            c.in_loop(|c| {
                c.visit_stmts(&n.block.stmts);
                do_if!(let Some(update) = &n.update, c.visit_stmt(update));
            })
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
        self.proven.clear();
        self.touched.clear();
        let given = n.ret.as_ref().map(|k| self.written_kind(k, KindUse::Result));
        // A function that can fail keeps its good result in `ရလဒ်`. The start
        // block gives back an exit code, never a result: ZK0441 reports a
        // result kind on it, and its statements check as ones that cannot fail.
        let parts = given.and_then(DataKind::fallible_parts).filter(|_| !n.start);
        let (ret, fault) = parts.map_or((given, None), |(ok, _)| (ok, given));
        #[rustfmt::skip]
        let ctx = FnCtx::builder().name(n.name.name.to_string()).maybe_ret(ret).maybe_fault(fault)
            .build();
        self.func = Some(ctx);
        self.scopes.push();
        do_for!(p in &n.params, self.declare_param(p));
        do_if!(n.start, self.check_start_header(n, ret));
        do_if!(n.external, self.check_external(n, given));
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
        ret_if!(n.fault, self.check_fault_return(n, &func));
        // A function that can fail gives back its good result with
        // `ရလဒ်ကောင်းအဖြစ်`, and a plain one with `ရလဒ်အဖြစ်`.
        let fails = func.fault.is_some();
        let wrong = n.value.is_some() && n.good != fails;
        do_if!(wrong, self.report(kd::good_return_word(n.span, &func.name, fails)));
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
        let place = n.place.as_deref();
        let ret =
            self.check_call(&n.callee, place, &n.args, &n.edits, &n.marks, n.span);
        do_if!(let Some(ret) = ret, self.check_pass(&n.callee, ret, n.pass, true, n.span));
    }

    fn visit_struct(&mut self, n: &StructDecl)
    {
        do_for!(f in &n.fields, self.work_out_counts(&f.kind));
    }
}

/// The text and the number of the character an assignment writes, when its
/// target is a character of text: `<text> ၏ <n> ခုမြောက်စာလုံး` or
/// `<text> ၏ စာလုံး <n>`.
///
/// # Arguments
///
/// * `target` - The target of the assignment.
///
/// # Returns
///
/// The text and the number, or `None` for each other target.
pub(crate) fn char_target(target: &Expr) -> Option<(&Expr, &Expr)>
{
    match target
    {
        Expr::Group(n) => char_target(&n.inner),
        Expr::Verb(n) if matches!(n.op, VerbOp::CharAt | VerbOp::CharNth) =>
        {
            let [text, index] = n.args.as_slice()
            else
            {
                return None;
            };
            Some((text, index))
        }
        _ => None,
    }
}

/// The value an each loop walks: the list a pointer points at for an
/// each-cell loop, else the value itself.
///
/// # Arguments
///
/// * `kind` - The kind of the source.
/// * `chars` - Whether the loop walks characters.
///
/// # Returns
///
/// The kind walked.
pub fn each_list(kind: DataKind, chars: bool) -> DataKind
{
    let pointee = kind.pointee().filter(|k| !chars && (k.is_list() || k.is_seq()));
    pointee.unwrap_or(kind)
}

/// The name of the error side of a result, for a message.
///
/// # Arguments
///
/// * `err` - The error kind, `None` for a side with no value.
///
/// # Returns
///
/// The kind name, or `မအောင်မြင်သည့်သတင်း`.
fn err_name(err: Option<DataKind>) -> String
{
    err.map_or_else(|| "မအောင်မြင်သည့်သတင်း".to_string(), DataKind::name)
}

/// The parameters of a function as a message shows them: `(text(ကို), text)`.
///
/// # Arguments
///
/// * `sig` - The function.
///
/// # Returns
///
/// The text.
fn params_text(sig: &FnSig) -> String
{
    let mark = |n: usize| {
        sig.marks.as_ref().map(|m| format!("({})", m[n])).unwrap_or_default()
    };
    let each =
        sig.params.iter().enumerate().map(|(n, k)| format!("{}{}", k.name(), mark(n)));
    format!("({})", each.collect::<Vec<_>>().join(", "))
}
