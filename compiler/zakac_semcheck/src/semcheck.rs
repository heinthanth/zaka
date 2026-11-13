use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;

use bon::Builder;
use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::NameExpr;
use zakac_ast::expr::NodeId;
use zakac_ast::expr::PrefixOp;
use zakac_ast::kind::Kind;
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
use zakac_ast::stmt::EachPart;
use zakac_ast::stmt::EachStmt;
use zakac_ast::stmt::EnumDecl;
use zakac_ast::stmt::FillStmt;
use zakac_ast::stmt::FnDecl;
use zakac_ast::stmt::FreeStmt;
use zakac_ast::stmt::GenericStmt;
use zakac_ast::stmt::IfArm;
use zakac_ast::stmt::IfStmt;
use zakac_ast::stmt::ImportStmt;
use zakac_ast::stmt::LeaveStmt;
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
use zakac_ast::stmt::TraitBlock;
use zakac_ast::stmt::TraitDecl;
use zakac_ast::stmt::WhileStmt;
use zakac_ast::verb::VerbOp;
use zakac_ast::visit::Node;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::walk_stmt;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::kinds as kd;
use zakac_diagnostics::resolve as rd;
use zakac_lexer::token::Kw;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_false;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::ret_true;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::captures::Captured;
use crate::captures::write_captures;
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
use crate::generic;
use crate::generic::Templates;
use crate::generic::Uses;
use crate::generic::Wanted;
use crate::generic_struct::leftovers;
use crate::generic_struct::name_instances;
use crate::host;
use crate::host::IO_ERROR;
use crate::kind::DataKind;
use crate::list::KindUse;
use crate::op_calls::OpCalls;
use crate::op_calls::write_op_calls;
use crate::ops::Want;
use crate::ops::kw_label;
use crate::overloads::Chosen;
use crate::overloads::write_overloads;
use crate::place::place_of;
use crate::proof::Fact;
use crate::proof::changed_in;
use crate::proof::changed_in_stmt;
use crate::proof::facts_when;
use crate::proof::leaves_at_end;
use crate::proof::returns_on_each_path;
use crate::refs::KindDecls;
use crate::refs::kind_refs;
use crate::scope::Access;
use crate::scope::Binding;
use crate::scope::FnSig;
use crate::scope::FnSigs;
use crate::scope::Scopes;
use crate::struct_check::collect_structs;
use crate::structs::StructTable;
use crate::traits::Traits;
use crate::traits::traits;
use crate::units::StructHomes;
use crate::units::private_in_exports;
use crate::units::start_rules;
use crate::units::struct_homes;
use crate::units::unit_names;

/// The words of a heap statement in a report.
const HEAP_WORK: &str = "a heap statement";

/// The implicit result variable of a function with a result.
pub const RESULT_NAME: &str = "ရလဒ်";

/// What needs unsafe code, with a handle a function got as a
/// parameter: close it, since the caller, or C that called back, still
/// holds it.
const CLOSE_PARAM: &str = "closing a handle the function got as a parameter";

/// The classifier each struct and enumeration of a program declares.
///
/// # Arguments
///
/// * `program` - The tree.
///
/// # Returns
///
/// The classifier of each that declares one, under its name.
fn struct_classifiers(program: &Program) -> HashMap<String, String>
{
    let word = |name: &zakac_ast::expr::Ident, c: &Option<zakac_ast::expr::Ident>| {
        c.as_ref().map(|c| (name.name.to_string(), c.name.to_string()))
    };
    let declared = program.stmts.iter().filter_map(|s| match s
    {
        Stmt::Struct(d) => word(&d.name, &d.classifier),
        Stmt::Enum(d) => word(&d.name, &d.classifier),
        _ => None,
    });
    declared.collect()
}

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
    /// Refuse each unsafe function and each unsafe block.
    #[builder(default)]
    pub no_unsafe: bool,
    /// Check the file as a module that another file imports, as an editor
    /// does for an open module: it needs no start block.
    #[builder(default)]
    pub as_module: bool,
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

/// Each name the check resolved: the span of a use, or of a declared name,
/// beside the span of the declared name in the file that declares it.
pub type RefTable = Vec<(Span, Span)>;

/// Output of the check.
#[derive(Debug, Clone, PartialEq, Default, Builder)]
pub struct SemcheckOutput
{
    /// Each resolved name with its declaration, for the editors.
    #[builder(default)]
    pub refs: RefTable,
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
    // A run writes each count it works out back as digits, so the names in a
    // count are met in the earlier runs alone: their records are kept.
    let mut refs = RefTable::new();
    let templates = generic::templates(program);
    let mut depth = 0;
    // The instances of generic functions and structs, from each run so far.
    let mut instances = Instances::default();
    loop
    {
        let (mut out, counts, found, found_enums, chosen, captured, asked, op_calls) =
            check_once(program, options, known.clone(), &enums, &templates, &instances);
        refs.extend(std::mem::take(&mut out.refs));
        let written = write_counts(program, &counts);
        let settled = counts.is_empty() || (written == 0 && found == known);
        let enums_settled = found_enums == enums;
        enums = found_enums;
        // The next run checks each new instance, and the instances it calls.
        instances.structs.extend(asked.structs);
        instances.fns.extend(asked.fns.clone());
        for (name, at) in asked.uses
        {
            instances.uses.entry(name).or_insert(at);
        }
        let deep = depth >= generic::INSTANCE_DEPTH;
        let structs = &out.structs;
        let named = name_instances(
            program,
            structs,
            &templates,
            &mut instances.structs,
            &mut instances.uses,
            deep,
        );
        let added = !deep
            && generic::add_instances(
                program,
                &templates,
                &asked.fns,
                &instances.structs,
            );
        depth += u32::from(added);
        if settled && enums_settled && !added && !named
        {
            out.diagnostics.extend(leftovers(program, &templates, &out.structs));
            out.diagnostics
                .extend(counts.into_values().filter_map(Result::err).flatten());
            out.diagnostics.sort_by_key(|d| d.span.lo);
            // Instances of one generic function share its spans, and each
            // reports the faults common to them.
            let mut seen = Vec::new();
            out.diagnostics.retain(|d| {
                let key = (d.code, d.span, d.message.clone());
                !seen.contains(&key) && {
                    seen.push(key);
                    true
                }
            });
            write_overloads(program, &chosen);
            let next = generic::next_id(program);
            write_op_calls(program, &op_calls, &mut out.kinds, next);
            write_captures(program, &captured);
            refs.sort_by_key(|(at, decl)| (at.lo, at.hi, decl.lo, decl.hi));
            refs.dedup();
            return SemcheckOutput { refs, ..out };
        }
        known = found;
    }
}

/// Instances of generic functions and structs: the ones a run of the check
/// asks for, or the ones known so far.
#[derive(Default)]
struct Instances
{
    /// The instances of generic functions.
    fns: Wanted,
    /// The instances of generic structs.
    structs: Wanted,
    /// The first use of each instance.
    uses: Uses,
}

/// Run the check once.
///
/// # Arguments
///
/// * `program` - The tree.
/// * `options` - The options.
/// * `known` - The top-level constants an earlier run found.
/// * `known_enums` - The enumeration values an earlier run worked out.
/// * `templates` - The generic functions and structs.
/// * `instances` - The instances of generic functions and structs so far.
///
/// # Returns
///
/// The output, the list counts the run met, the top-level constants it
/// found, the enumeration values it worked out, the function each call of
/// an overloaded or generic name chose, the captures, and the instances its
/// calls and struct values ask for.
fn check_once(
    program: &Program, options: &SemcheckOptions, known: KnownConsts,
    known_enums: &KnownEnums, templates: &Templates, instances: &Instances,
) -> (
    SemcheckOutput,
    CountTable,
    KnownConsts,
    KnownEnums,
    Chosen,
    Captured,
    Instances,
    OpCalls,
)
{
    let mut diags = Vec::new();
    let enums = collect_enums(program, known_enums, &mut diags);
    let mut structs = collect_structs(program, &enums, &mut diags);
    structs.extend(enums);
    let unit_fns = collect_fns(program, &structs, &mut diags);
    let homes = struct_homes(program);
    let decls = KindDecls::of(program);
    diags.extend(private_in_exports(program, &homes));
    diags.extend(start_rules(program, !options.as_module));
    #[rustfmt::skip]
    let mut checker = Checker::builder().fns(unit_fns[0].clone()).unit_fns(unit_fns)
        .homes(homes).unit_names(unit_names(program)).structs(structs)
        .diags(diags).platform(options.platform_bits).no_unsafe(options.no_unsafe)
        .as_module(options.as_module)
        .known(known).open_structs(open_structs(program)).decls(decls)
        .classifiers(struct_classifiers(program)).templates(templates.clone()).known_fns(instances.fns.clone()).known_structs(instances.structs.clone())
        .uses(instances.uses.clone())
        .maybe_prelude_unit((0..program.units.len() as u32).find(|u| generic::in_prelude(program, *u)))
        .traits(traits(program)).build();
    checker.scopes.push();
    checker.check_struct_bounds();
    checker.visit_program(program);
    let own = checker.decls.own();
    checker.refs.extend(own);
    checker.refs.extend(kind_refs(program, &checker.decls));
    checker.diags.sort_by_key(|d| d.span.lo);
    checker.refs.sort_by_key(|(at, decl)| (at.lo, at.hi, decl.lo, decl.hi));
    checker.refs.dedup();
    #[rustfmt::skip]
    let out = SemcheckOutput::builder().kinds(checker.kinds).structs(checker.structs).diagnostics(checker.diags).refs(checker.refs).build();
    let captured = std::mem::take(&mut checker.captured);
    #[rustfmt::skip]
    #[rustfmt::skip]
    let asked = Instances { fns: checker.wanted, structs: checker.struct_wanted, uses: checker.uses };
    #[rustfmt::skip]
    return (out, checker.counts, checker.found, checker.found_enums, checker.chosen, captured, asked, checker.op_calls);
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
    /// The parameters that hold a handle, or an optional of one: the
    /// function did not make them, so safe code cannot close them.
    #[builder(default)]
    pub handles: Vec<Name>,
}

/// State of the walk.
#[derive(Builder)]
pub(crate) struct Checker
{
    /// Names in scope.
    #[builder(default)]
    pub scopes: Scopes,
    /// The classifier each struct declares; a struct with none counts with
    /// `ခု`.
    #[builder(default)]
    pub classifiers: HashMap<String, String>,
    /// The functions the current file declares or imports.
    pub fns: FnSigs,
    /// The function each call of an overloaded or generic name chose.
    #[builder(default)]
    pub chosen: Chosen,
    /// The generic functions, which the walk checks through their instances.
    #[builder(default)]
    pub templates: Templates,
    /// The instances the calls of generic functions ask for.
    #[builder(default)]
    pub wanted: Wanted,
    /// The traits of the program.
    #[builder(default)]
    pub traits: Traits,
    /// The instances of generic functions from the runs before.
    #[builder(default)]
    pub known_fns: Wanted,
    /// The instances of generic structs from the runs before.
    #[builder(default)]
    pub known_structs: Wanted,
    /// The first use of each instance, from the runs before and this one.
    #[builder(default)]
    pub uses: Uses,
    /// The instances the struct values of generic structs ask for.
    #[builder(default)]
    pub struct_wanted: Wanted,
    /// The kind the place of the call expression being checked expects, for
    /// the kinds of a generic function its arguments leave open; the check
    /// of the call takes it.
    pub call_expect: Option<DataKind>,
    /// The unit of the prelude, when the program reads it.
    pub prelude_unit: Option<u32>,
    /// Whether the walk is in an instance of a generic function, which sees
    /// the functions of the files of its generic, its kinds and its traits.
    #[builder(default)]
    pub in_instance: bool,
    /// The call each operator, print and template slot on a struct takes.
    #[builder(default)]
    pub op_calls: OpCalls,
    /// The operator the next two-operand inference stands for, with its
    /// node, when a struct can take it through a function.
    pub pending_op: Option<(NodeId, (&'static str, bool))>,
    /// The top-level function the walk is in; a blank name outside one.
    #[builder(default = Name::new(""))]
    pub top: Name,
    /// The locals and closures each nested function uses from around it.
    #[builder(default)]
    pub captured: Captured,
    /// Each resolved name with its declaration.
    #[builder(default)]
    pub refs: RefTable,
    /// The declared names of the structs, fields, enumerations and members.
    #[builder(default)]
    pub decls: KindDecls,
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
    /// The switches over an enumeration that name each member, so no path
    /// runs past them with no arm.
    #[builder(default)]
    pub full_switches: HashSet<Span>,
    /// Bit count of the platform-sized integer kinds.
    pub platform: u16,
    /// The parameter and result kinds of each C function name declared so
    /// far, in each file.
    #[builder(default)]
    pub c_names: HashMap<String, (Vec<DataKind>, Option<DataKind>)>,
    /// Whether unsafe code is refused.
    #[builder(default)]
    pub no_unsafe: bool,
    /// Whether the statements checked now are unsafe code: an unsafe function,
    /// an unsafe block, or a function inside either.
    #[builder(default)]
    pub unchecked: bool,
    /// Whether the program is checked as a module, with no start block.
    #[builder(default)]
    pub as_module: bool,
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
    /// The reads of a row that stand at the base of a place a statement
    /// writes through, which have the value kind and not an optional.
    #[builder(default)]
    pub row_writes: HashSet<NodeId>,
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
        let fresh = self.scopes.declare(ident.name, binding).is_ok();
        do_if!(fresh, self.refer(&ident.name, ident.span, ident.span));
        fresh
    }

    /// Record a resolved name with its declaration. The implicit result
    /// `ရလဒ်` is left out: it has no declaration of its own to go to.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    /// * `at` - The span of the name where it stands.
    /// * `decl` - The declared name in the file that declares it.
    pub(crate) fn refer(&mut self, name: &str, at: Span, decl: Span)
    {
        let real = name != RESULT_NAME && !decl.is_placeholder();
        do_if!(real, self.refs.push((at, decl)));
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
        self.report(mismatch_report(e.span(), want, found));
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
    /// * `kind_args` - The kinds written for a generic function.
    /// * `place` - The place that holds the function value, when written.
    /// * `args` - The arguments.
    /// * `marks` - The word after each argument of a marked call.
    /// * `span` - The call.
    ///
    /// # Returns
    ///
    /// The result kind of the function (`Some(None)` when it has none), or
    /// `None` when the call names no function it can run.
    pub fn check_call(
        &mut self, callee: &Ident, kind_args: &[Kind], place: Option<&Expr>,
        args: &[Expr], marks: &[Ident], span: Span,
    ) -> Option<Option<DataKind>>
    {
        let expect = self.call_expect.take();
        let template = self.template_named(&callee.name).filter(|_| place.is_none());
        let target = match template
        {
            Some(t) =>
            {
                let sig = self.generic_callee(&t, callee, kind_args, args, expect);
                sig.map(|s| vec![s])
            }
            None => self.plain_callee(callee, kind_args, place),
        };
        let Some(sigs) = target
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
        self.name_foreign(&sig, callee.span);
        self.refer(&callee.name, callee.span, sig.decl);
        let call = format!("a call of \"{}\", an unsafe function,", callee.name);
        do_if!(sig.unchecked, self.need_unchecked(span, &call));
        let (want, got) = (sig.params.len(), args.len());
        let bad_count = rd::arg_count_mismatch(span, &callee.name, want, got);
        let ret = Some(sig.ret);
        let wrong = tern!(sig.variadic, got < want, got != want);
        ret_if!(wrong, self.report(bad_count), self.infer_all(args), ret);
        do_for!(arg in &args[want..], self.check_rest_arg(arg));
        for (n, (arg, kind)) in args.iter().zip(&sig.params).enumerate()
        {
            let edit = sig.edits.get(n).copied().unwrap_or(false);
            self.check_arg(arg, *kind, edit);
            let closes = sig.closes.get(n).copied().unwrap_or(false);
            do_if!(
                closes && self.is_handle_param(arg),
                self.need_unchecked(arg.span(), CLOSE_PARAM)
            );
        }
        self.check_edits_apart(args, &sig.edits);
        self.check_marks(callee, marks, sig.marks.as_deref());
        ret
    }

    /// Check an argument past the parameters of a C function that takes
    /// other values: its kind is the one it has on its own, and must cross
    /// to C as a parameter does (ZK0465); C passes a struct through `...` as
    /// it passes a struct parameter.
    ///
    /// # Arguments
    ///
    /// * `arg` - The argument.
    fn check_rest_arg(&mut self, arg: &Expr)
    {
        let kind = self.infer(arg, None);
        let platform = self.platform;
        let stuck = kind.filter(|k| !foreign::crosses_param(*k, platform));
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
    /// the result cross to C (ZK0465), no parameter but a list of numbers is
    /// marked `ဒေတာ ပြောင်းလဲရန်` (ZK0466), a function that passes a pointer or
    /// gives back text that can be NULL is declared `စိတ်မချရသော` (ZK0472), and
    /// each declaration of the name gives the same kinds (ZK0338).
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
        let written = |k: &DataKind| foreign::edit_crosses(*k, platform);
        let edits = n.params.iter().zip(&params).filter(|(p, k)| p.edit && !written(k));
        let edits = edits.map(|(p, _)| p);
        let edits: Vec<_> =
            edits.map(|p| kd::edit_across_c(p.span, &p.name.name)).collect();
        do_for!(d in edits, self.report(d));
        let bad_ret = ret.filter(|k| !foreign::crosses_back(*k, platform));
        let bad_ret =
            bad_ret.map(|k| match k.is_text() || foreign::is_failing_pointer(k)
            {
                true => kd::text_result_across_c(n.name.span, &k.name()),
                false => kd::kind_not_across_c(n.name.span, &k.name()),
            });
        do_if!(let Some(d) = bad_ret, self.report(d));
        let pointer = params.iter().chain(&ret).any(|k| foreign::holds_ptr(*k));
        // NULL from C is an optional the kind declares, so it needs no mark.
        let mark = kd::foreign_needs_mark(n.name.span, &name);
        do_if!(pointer && !n.unchecked, self.report(mark));
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
                self.chosen.insert((self.top, callee.span), one.inner);
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
        // An optional holds no data or some; it is no failure to pass on.
        let fails = ret.filter(|k| !k.is_optional());
        let Some((_, found)) = fails.and_then(DataKind::fallible_parts)
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

    /// Check `... ကို အမှားအယွင်းအဖြစ် ပြန်ပေးမည်။` against the error kind of
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
                self.report(kd::fault_needs_result(v.span()));
                self.infer(v, None);
            }
            (None, Some(err)) =>
            {
                self.report(kd::fault_value_missing(n.span, &err.name()))
            }
            (None, None) => (),
        }
    }

    /// The variables statements can change, at each depth.
    ///
    /// # Arguments
    ///
    /// * `stmts` - The statements.
    ///
    /// # Returns
    ///
    /// The names.
    fn changed(&self, stmts: &[Stmt]) -> HashSet<Name>
    {
        changed_in(stmts, &|name| self.scopes.lookup_fn(name).map(|s| s.edits))
    }

    /// The variables a statement can change on its own.
    ///
    /// # Arguments
    ///
    /// * `s` - The statement.
    ///
    /// # Returns
    ///
    /// The names.
    fn changed_stmt(&self, s: &Stmt) -> HashSet<Name>
    {
        changed_in_stmt(s, &|name| self.scopes.lookup_fn(name).map(|s| s.edits))
    }

    /// Check the marks `အပိုင်အနေဖြင့်` of a function: each on a parameter or
    /// a result that is a pointer (ZK0470), and on a C result, a pointer or
    /// text the program takes over (ZK0467).
    ///
    /// # Arguments
    ///
    /// * `n` - The declaration.
    /// * `given` - Its result kind.
    fn check_owned(&mut self, n: &FnDecl, given: Option<DataKind>)
    {
        let kind = |p: &Param| DataKind::from_ast(&p.kind, &self.structs);
        // A C function can take a handle over, which closes it.
        let held =
            |k: DataKind| k.is_ptr() || (n.lib.is_some() && k.as_external().is_some());
        let bad: Vec<_> =
            n.params.iter().filter(|p| p.owned && !held(kind(p))).collect();
        let bad: Vec<_> =
            bad.iter().map(|p| kd::owned_not_ptr(p.span, &kind(p).name())).collect();
        do_for!(d in bad, self.report(d));
        ret_if!(!n.owned_ret);
        let ptr = given.is_some_and(DataKind::is_ptr);
        let text = given.is_some_and(foreign::is_c_text_result);
        ret_if!(ptr || (n.lib.is_some() && text));
        let name = given.map_or_else(String::new, |k| k.name());
        let diag = tern!(
            n.lib.is_some(),
            kd::owned_not_c_text(n.name.span, &n.name.name),
            kd::owned_not_ptr(n.name.span, &name)
        );
        self.report(diag);
    }

    /// Check a write of a field of the struct C keeps behind a handle:
    /// unsafe code can write a number, a boolean or a counted list; text and
    /// a handle are pointers C owns, and safe code writes no field.
    ///
    /// # Arguments
    ///
    /// * `f` - The field.
    /// * `value` - The value written.
    ///
    /// # Returns
    ///
    /// `true` when the base is a handle, so the write is checked here.
    fn c_field_write(&mut self, f: &zakac_ast::expr::FieldExpr, value: &Expr) -> bool
    {
        let base = self.infer(&f.base, None);
        let Some(shape) = base.and_then(DataKind::as_external)
        else
        {
            return false;
        };
        let field = shape.c_field(&f.field.name, self.platform).map(|(k, _)| k);
        let Some(kind) = field
        else
        {
            self.report(rd::unknown_field(f.field.span, &shape.name, &f.field.name));
            return true;
        };
        let pointer = kind.is_text() || kind.as_external().is_some();
        ret_if!(
            !self.unchecked || pointer,
            self.report(kd::c_field_write(f.span, &shape.name)),
            true
        );
        self.expect_kind(value, Some(kind));
        true
    }

    /// Whether a parameter holds a handle, or an optional of one.
    ///
    /// # Arguments
    ///
    /// * `p` - The parameter.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn holds_handle(&self, p: &Param) -> bool
    {
        let kind = DataKind::from_ast(&p.kind, &self.structs);
        kind.as_external().is_some() || foreign::is_c_handle_result(kind)
    }

    /// Whether an argument is a handle parameter of the function around it,
    /// as it is or read out of its optional with `ထဲမှ ဒေတာ`.
    ///
    /// # Arguments
    ///
    /// * `arg` - The argument.
    ///
    /// # Returns
    ///
    /// `true` for such a parameter.
    fn is_handle_param(&self, arg: &Expr) -> bool
    {
        let name = match arg
        {
            Expr::Name(n) => Some(n.name),
            Expr::Group(g) => return self.is_handle_param(&g.inner),
            Expr::Verb(v) if v.op == VerbOp::Unwrap => match v.args.as_slice()
            {
                [Expr::Name(n)] => Some(n.name),
                _ => None,
            },
            _ => None,
        };
        let handles =
            self.func.as_ref().map(|f| f.handles.as_slice()).unwrap_or_default();
        name.is_some_and(|n| handles.contains(&n))
    }

    /// Report an operation that works with pointers outside unsafe code.
    ///
    /// # Arguments
    ///
    /// * `span` - The operation.
    /// * `what` - The words of the operation, as text.
    pub(crate) fn need_unchecked(&mut self, span: Span, what: &str)
    {
        do_if!(!self.unchecked, self.report(kd::unsafe_outside(span, what)));
    }

    /// Check one argument against its parameter: a value that converts to
    /// the parameter kind, or for a parameter the function can change, a
    /// writable place of that kind alone.
    ///
    /// # Arguments
    ///
    /// * `arg` - The argument.
    /// * `kind` - The parameter kind.
    /// * `edit` - Whether the function can change the parameter.
    fn check_arg(&mut self, arg: &Expr, kind: DataKind, edit: bool)
    {
        ret_if!(!edit, self.expect_kind(arg, Some(kind)));
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
    /// * `edits` - Whether the function can change each parameter.
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
    /// cell for a list, or for the list a pointer points at, as a cell read
    /// takes it.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of the value the loop walks.
    /// * `n` - The loop.
    ///
    /// # Returns
    ///
    /// The kind, or `None` after a report.
    fn each_var_kind(
        &mut self, kind: DataKind, n: &EachStmt,
    ) -> Option<(Option<DataKind>, DataKind)>
    {
        ret_if!(n.part.is_row(), self.row_var_kinds(kind, n));
        let chars = n.part == EachPart::Char;
        let cell = each_list(kind, chars).cell_kind();
        let walked = tern!(chars, kind.is_text().then_some(DataKind::Char), cell);
        // The first of two names counts the cells or characters from 0.
        let place = n.place.as_ref().map(|_| DataKind::USIZE);
        ret_if!(let Some(walked) = walked, Some((place, walked)));
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
    pub(crate) fn place_kind(&mut self, target: &Expr) -> Option<DataKind>
    {
        self.mark_row_write(target);
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
        // C owns the fields of a struct a handle points at.
        let base =
            tern!(let Expr::Field(f) = place, self.kinds.get(&f.base.id()), None);
        let handle = base.copied().and_then(DataKind::as_external);
        ret_if!(
            let Some(s) = handle,
            self.report(kd::c_field_write(target.span(), &s.name)),
            None
        );
        let text =
            tern!(let Expr::Index(cell) = place, self.kinds.get(&cell.list.id()), None);
        let text = text.copied().filter(|k| k.is_text());
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

    /// Check what an add puts in text: text or a character.
    ///
    /// # Arguments
    ///
    /// * `value` - What is added.
    fn text_add(&mut self, value: &Expr)
    {
        let Some(found) = self.infer(value, None)
        else
        {
            return;
        };
        ret_if!(found.is_text() || found == DataKind::Char);
        self.report(mismatch_report(value.span(), DataKind::TEXT, found));
    }

    /// Check an add that puts its value in front of a cell of a plain list or
    /// a character of text: the list or the text is a place that can grow,
    /// the number an integer, and the value fits a cell or is text or a
    /// character.
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
            let text = kind.is_some_and(|k| self.is_text_place(k, text));
            ret_if!(!text, _ = self.infer(value, None));
            return self.text_add(value);
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
        _ = self.is_text_place(kind, text);
    }

    /// Whether a place a character statement names is text, with a report
    /// for each other kind.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of the place.
    /// * `text` - The place.
    ///
    /// # Returns
    ///
    /// `true` for text.
    fn is_text_place(&mut self, kind: DataKind, text: &Expr) -> bool
    {
        ret_true!(kind.is_text());
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
        do_if!(let Some(kind) = kind, _ = self.is_text_place(kind, text));
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
        self.refer(&n.name, n.span, binding.home());
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

    /// Report the cells of a blit or a fill that hold counted blocks (text,
    /// plain lists, objects, function values), whose counts a raw duplicate
    /// would not keep: the unit kind of the size, or each kind the addresses
    /// point at for a size in octets. Pointer cells are plain addresses.
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
        let owner = cells.into_iter().find(|k| k.holds_text());
        do_if!(let Some(k) = owner, self.report(kd::blit_of_held_cells(size.span, &k.name())));
    }

    /// Check the statements of a loop block, and the update of a generic
    /// loop after them: unsafe code when the block opens with
    /// `အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။`.
    ///
    /// # Arguments
    ///
    /// * `block` - The block of the loop.
    /// * `update` - The update of a generic loop.
    fn scan_loop_block(&mut self, block: &BlockStmt, update: Option<&Stmt>)
    {
        do_if!(
            block.unchecked && self.no_unsafe,
            self.report(kd::unsafe_refused(block.span))
        );
        let outer = self.unchecked;
        self.unchecked |= block.unchecked;
        self.visit_stmts(&block.stmts);
        do_if!(let Some(update) = update, self.visit_stmt(update));
        self.unchecked = outer;
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
/// turns a pointer the compiler lends into one that owns its block, or
/// back, so a mismatch of the two gets no help. Each other mismatch names
/// the conversion verb when it can turn the value into the wanted kind, as
/// for a pointer whose block holds addresses, which unsafe code converts
/// with the verb; it gets no help when the verb cannot, as for a list.
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
        ret_if!(let Expr::Field(f) = &n.target && self.c_field_write(f, &n.value));
        let want = self.place_kind(&n.target);
        self.expect_kind(&n.value, want);
    }

    fn visit_alloc(&mut self, n: &AllocStmt)
    {
        self.need_unchecked(n.span, HEAP_WORK);
        self.heap_size(&n.size);
        self.heap_target(&n.target);
    }

    fn visit_resize(&mut self, n: &ResizeStmt)
    {
        self.need_unchecked(n.span, HEAP_WORK);
        self.heap_target(&n.target);
        self.heap_size(&n.size);
    }

    fn visit_free(&mut self, n: &FreeStmt)
    {
        self.need_unchecked(n.span, HEAP_WORK);
        self.heap_target(&n.target);
    }

    fn visit_leave(&mut self, n: &LeaveStmt)
    {
        let kind = self.infer(&n.value, None);
        let Some(kind) = kind.filter(|k| !k.is_fn())
        else
        {
            return;
        };
        self.report(kd::leave_needs_function(n.value.span(), &kind.name()));
    }

    fn visit_blit(&mut self, n: &BlitStmt)
    {
        self.need_unchecked(n.span, HEAP_WORK);
        let src = self.heap_address(&n.src);
        let unit = self.heap_size(&n.size);
        let dst = self.heap_address(&n.dst);
        self.owned_cells(&n.size, unit, &[src, dst]);
    }

    fn visit_fill(&mut self, n: &FillStmt)
    {
        self.need_unchecked(n.span, HEAP_WORK);
        let dst = self.heap_address(&n.dst);
        let unit = self.heap_size(&n.size);
        self.expect_kind(&n.value, Some(unit.unwrap_or(DataKind::U8)));
        self.owned_cells(&n.size, unit, &[dst]);
    }

    fn visit_add(&mut self, n: &AddStmt)
    {
        ret_if!(n.before, self.insert(&n.list, &n.value));
        let kind = self.place_kind(&n.list);
        ret_if!(kind.is_some_and(|k| k.is_text()), self.text_add(&n.value));
        let cell = kind.and_then(|k| self.growing_cells(&n.list, k));
        self.expect_kind(&n.value, cell);
    }

    fn visit_remove(&mut self, n: &RemoveStmt)
    {
        ret_if!(n.row, self.remove_row(&n.list));
        let kind = self.place_kind(&n.list);
        ret_if!(n.chars, do_if!(let Some(k) = kind, self.shrinking_text(&n.list, k)));
        do_if!(let Some(k) = kind, { _ = self.growing_cells(&n.list, k); });
    }

    fn visit_replace(&mut self, n: &ReplaceStmt)
    {
        let kind = self.place_kind(&n.text);
        self.cell_number(&n.start);
        self.cell_number(&n.end);
        let text = kind.is_some_and(|k| self.is_text_place(k, &n.text));
        ret_if!(!text, _ = self.infer(&n.value, None));
        self.text_add(&n.value);
    }

    fn visit_print(&mut self, n: &PrintStmt)
    {
        let kind = self.infer(&n.value, None);
        self.struct_text(&n.value, kind);
        // An optional of a handle prints its handle when it holds one.
        let good = kind.and_then(|k| k.fallible_parts()).and_then(|(ok, _)| ok);
        let handle = kind.or(good).and_then(DataKind::as_external);
        let handle = handle.or_else(|| good.and_then(DataKind::as_external));
        do_if!(let Some(s) = handle, self.report(kd::handle_in_zaka(n.value.span(), &s.name, kd::HandleUse::Print)));
    }

    fn visit_block(&mut self, n: &BlockStmt)
    {
        do_if!(n.unchecked && self.no_unsafe, self.report(kd::unsafe_refused(n.span)));
        let outer = self.unchecked;
        self.unchecked |= n.unchecked;
        self.scopes.push();
        self.visit_stmts(&n.stmts);
        self.scopes.pop();
        self.unchecked = outer;
    }

    fn visit_stmt(&mut self, n: &Stmt)
    {
        walk_stmt(self, n);
        self.forget(self.changed_stmt(n));
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

    fn visit_trait(&mut self, n: &TraitDecl) { self.check_trait(n) }

    fn visit_trait_block(&mut self, n: &TraitBlock) { self.check_trait_block(n) }

    fn visit_if_arm(&mut self, n: &IfArm)
    {
        self.expect_bool(&n.cond);
        self.visit_block(&n.block);
    }

    fn visit_while(&mut self, n: &WhileStmt)
    {
        let changed = self.changed(&n.block.stmts);
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
        let changed = self.changed(&n.block.stmts);
        self.proven_loop(changed, None, |c| {
            c.in_loop(|c| c.scan_loop_block(&n.block, None))
        });
        self.scopes.pop();
    }

    fn visit_each(&mut self, n: &EachStmt)
    {
        let kinds = self.infer(&n.source, None).and_then(|k| self.each_var_kind(k, n));
        self.scopes.push();
        do_if!(let Some(place) = &n.place, self.declare(place, kinds.and_then(|(p, _)| p), Access::Var));
        self.declare(&n.var, kinds.map(|(_, k)| k), Access::Var);
        let changed = self.changed(&n.block.stmts);
        self.proven_loop(changed, None, |c| {
            c.in_loop(|c| c.scan_loop_block(&n.block, None))
        });
        self.scopes.pop();
    }

    fn visit_generic(&mut self, n: &GenericStmt)
    {
        let kind = self.decl_kind(n.kind.as_ref(), n.init.as_ref(), &n.var);
        self.scopes.push();
        self.declare(&n.var, kind, Access::Var);
        let mut changed = self.changed(&n.block.stmts);
        changed.extend(
            n.update.as_deref().map(|s| self.changed_stmt(s)).unwrap_or_default(),
        );
        changed.insert(n.var.name);
        self.forget(changed.clone());
        self.expect_bool(&n.cond);
        self.proven_loop(changed, Some(&n.cond), |c| {
            c.in_loop(|c| c.scan_loop_block(&n.block, n.update.as_deref()))
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
        do_if!(!n.start, self.refer(&n.name.name, n.name.span, n.name.span));
        // A function inside a function shares the locals around it that it
        // names, and the walk of the outer one goes on after it.
        let nested = self.func.is_some();
        do_if!(n.unchecked && self.no_unsafe, self.report(kd::unsafe_refused(n.span)));
        // A function inside unsafe code is unsafe code too.
        let outer_unchecked = self.unchecked;
        self.unchecked = n.unchecked || (nested && outer_unchecked);
        do_if!(nested, self.declare_nested(n));
        let outer = self.func.take();
        let depth = std::mem::take(&mut self.loop_depth);
        let proven = std::mem::take(&mut self.proven);
        let touched = std::mem::take(&mut self.touched);
        let given = n.ret.as_ref().map(|k| self.written_kind(k, KindUse::Result));
        // A function that can fail gives back its good result or its error
        // value; an optional is a plain result. The start block gives back an
        // exit code, never a result: ZK0441 reports a result kind on it, and
        // its statements check as ones that cannot fail.
        let fails = given.filter(|k| !k.is_optional() && !n.start);
        let parts = fails.and_then(DataKind::fallible_parts);
        let (ret, fault) = parts.map_or((given, None), |(ok, _)| (ok, given));
        #[rustfmt::skip]
        let handles = n.params.iter().filter(|p| self.holds_handle(p)).map(|p| p.name.name);
        let handles = handles.collect();
        let ctx = FnCtx::builder()
            .name(n.name.name.to_string())
            .maybe_ret(ret)
            .maybe_fault(fault)
            .handles(handles)
            .build();
        self.func = Some(ctx);
        self.scopes.enter_fn();
        do_for!(p in &n.params, self.declare_param(p));
        do_if!(n.start, self.check_start_header(n, ret));
        do_if!(n.external, self.check_external(n, given));
        self.check_owned(n, given);
        self.visit_stmts(&n.block.stmts);
        let open_end = ret.is_some() && !n.start && !n.external;
        do_if!(
            open_end && !returns_on_each_path(&n.block.stmts, &self.full_switches),
            self.report(rd::end_without_return(n.block.span, &n.name.name))
        );
        let used = self.scopes.leave_fn();
        do_if!(nested, self.note_captures(n, &used));
        self.func = outer;
        self.unchecked = outer_unchecked;
        self.loop_depth = depth;
        (self.proven, self.touched) =
            tern!(nested, (proven, touched), Default::default());
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
        // A function that can fail takes its good result as it is, or a
        // whole result such as `<e> ဟူသော အမှားအယွင်း`.
        let whole = func.fault;
        match (func.ret, &n.value)
        {
            (None, Some(v)) if whole.is_none() =>
            {
                self.report(rd::value_returned_from_void(n.span, &func.name));
                self.infer(v, None);
            }
            (Some(k), None) =>
            {
                self.report(rd::missing_return_value(n.span, &func.name, &k.name()))
            }
            (ret, Some(v)) =>
            {
                self.expect_kind(v, whole.or(ret));
            }
            (None, None) => (),
        }
    }

    fn visit_call_stmt(&mut self, n: &CallStmt)
    {
        let place = n.place.as_deref();
        let ret =
            self.check_call(&n.callee, &n.kind_args, place, &n.args, &n.marks, n.span);
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
