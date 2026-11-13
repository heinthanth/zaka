//! Generic structs. A struct with kind parameters is a template: the check
//! makes no kind of it. Each kind that names it with its kinds, such as
//! `အတွဲ<စာသား၊ ကိန်းပြည့်>`, asks for an instance, a struct in which those
//! kinds stand for its kind parameters, named `အတွဲ<text၊ isize>`. Between
//! two runs of the check, each such kind whose kinds are all known takes the
//! name of its instance, so the next run reads a plain struct name. A struct
//! value with no kinds written takes them from the kind its place expects, or
//! else from the kinds of its field values.

use std::collections::HashMap;

use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::StructLit;
use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Kind;
use zakac_ast::name::Name;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::StructDecl;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::VisitorMut;
use zakac_ast::visit::walk_expr;
use zakac_ast::visit::walk_expr_mut;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::resolve as rd;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use crate::generic::Templates;
use crate::generic::Uses;
use crate::generic::Wanted;
use crate::generic::infer_kinds;
use crate::generic::instance_name;
use crate::generic::is_template;
use crate::generic::kind_ast;
use crate::kind::DataKind;
use crate::semcheck::Checker;
use crate::structs::StructTable;

/// Give each kind and struct value that names a generic struct with known
/// kinds the name of its instance, and record the instance.
///
/// # Arguments
///
/// * `program` - The program.
/// * `structs` - The struct kinds of the last run.
/// * `templates` - The templates.
/// * `known` - The struct instances so far; each new one is added.
/// * `deep` - Whether instances have reached their depth limit, so a kind whose
///   instance does not exist keeps its kinds.
///
/// # Returns
///
/// `true` when a kind took a name, so the check runs again.
pub(crate) fn name_instances(
    program: &mut Program, structs: &StructTable, templates: &Templates,
    known: &mut Wanted, uses: &mut Uses, deep: bool,
) -> bool
{
    let templates = &templates.structs;
    let mut namer = Namer { structs, templates, known, uses, deep, changed: false };
    let plain = program.stmts.iter_mut().filter(|s| !is_template(s));
    do_for!(stmt in plain, namer.visit_stmt_mut(stmt));
    namer.changed
}

/// The walker that names instances.
struct Namer<'a>
{
    /// The struct kinds of the last run.
    structs: &'a StructTable,
    /// The generic structs.
    templates: &'a HashMap<Name, StructDecl>,
    /// The struct instances so far.
    known: &'a mut Wanted,
    /// The first use of each instance.
    uses: &'a mut Uses,
    /// Whether instances have reached their depth limit.
    deep: bool,
    /// Whether a kind took a name.
    changed: bool,
}

impl Namer<'_>
{
    /// Name the instance a kind stands for, the kinds inside it first.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    fn name_kind(&mut self, kind: &mut Kind)
    {
        do_for!(inner in inner_kinds(kind), self.name_kind(inner));
        let Some(name) = self.instance_of(kind.name.as_ref(), &kind.args)
        else
        {
            return;
        };
        do_if!(let Some(ident) = &mut kind.name, ident.name = name);
        kind.args.clear();
        self.changed = true;
    }

    /// The instance a generic struct name and its kinds stand for, recorded
    /// among the known instances.
    ///
    /// # Arguments
    ///
    /// * `ident` - The struct name.
    /// * `args` - Its kinds as written.
    ///
    /// # Returns
    ///
    /// The instance name, or `None` while a kind is not known, for another
    /// count of kinds, or for an instance past the depth limit.
    fn instance_of(&mut self, ident: Option<&Ident>, args: &[Kind]) -> Option<Name>
    {
        ret_if!(args.is_empty(), None);
        let ident = ident?;
        let template = self.templates.get(&ident.name)?;
        ret_if!(template.kind_params.len() != args.len(), None);
        ret_if!(!args.iter().all(|a| resolved(a, self.structs)), None);
        let kinds: Vec<DataKind> =
            args.iter().map(|a| DataKind::from_ast(a, self.structs)).collect();
        ret_if!(kinds.iter().any(|k| kind_ast(*k, ident.span).is_none()), None);
        let name = instance_name(ident.name, &kinds);
        ret_if!(self.deep && !self.structs.contains_key(&name), None);
        self.known.insert(name, (ident.name, kinds));
        self.uses.entry(name).or_insert(ident.span);
        Some(name)
    }
}

impl VisitorMut for Namer<'_>
{
    fn visit_kind_mut(&mut self, n: &mut Kind) { self.name_kind(n) }

    fn visit_expr_mut(&mut self, n: &mut Expr)
    {
        if let Expr::Struct(lit) = n
        {
            do_for!(arg in &mut lit.kind_args, self.name_kind(arg));
            let name = self.instance_of(Some(&lit.name), &lit.kind_args);
            if let Some(name) = name
            {
                lit.name.name = name;
                lit.kind_args.clear();
                self.changed = true;
            }
        }
        walk_expr_mut(self, n);
    }
}

/// The kinds one level inside a kind: the sides of a result, the index and
/// value of a ဇယား, the parameters and result of a function kind, and the
/// kinds of a generic.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// The kinds.
fn inner_kinds(kind: &mut Kind) -> Vec<&mut Kind>
{
    let mut inner: Vec<&mut Kind> = Vec::new();
    if let Some(parts) = kind.parts.as_deref_mut()
    {
        inner.extend(parts.ok.iter_mut().chain(parts.err.iter_mut()));
    }
    if let Some(table) = kind.table.as_deref_mut()
    {
        inner.extend([&mut table.index, &mut table.value]);
    }
    if let Some(sig) = kind.sig.as_deref_mut()
    {
        inner.extend(
            sig.params.iter_mut().map(|p| &mut p.kind).chain(sig.ret.iter_mut()),
        );
    }
    inner.extend(kind.args.iter_mut());
    inner
}

/// The kinds one level inside a kind, to read.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// The kinds.
fn inner_of(kind: &Kind) -> Vec<&Kind>
{
    let parts = kind.parts.iter().flat_map(|p| p.ok.iter().chain(p.err.iter()));
    let table = kind.table.iter().flat_map(|t| [&t.index, &t.value]);
    let sig = kind
        .sig
        .iter()
        .flat_map(|s| s.params.iter().map(|p| &p.kind).chain(s.ret.iter()));
    parts.chain(table).chain(sig).chain(kind.args.iter()).collect()
}

/// Whether the check knows each struct a kind names: no generic in it waits
/// for its instance.
///
/// # Arguments
///
/// * `kind` - The kind.
/// * `structs` - The struct kinds.
///
/// # Returns
///
/// `true` when each name is a known struct.
fn resolved(kind: &Kind, structs: &StructTable) -> bool
{
    let named = kind.base != BaseKind::Named
        || kind.args.is_empty()
            && kind.name.as_ref().is_some_and(|n| structs.contains_key(&n.name));
    named && inner_of(kind).into_iter().all(|k| resolved(k, structs))
}

/// The reports for each generic struct that kept its kinds once the check
/// settled: ZK0340 for another count of kinds, ZK0342 for an instance past
/// the depth limit, and an unknown kind name for kinds after a name that is
/// no generic struct. A kind that is not known has a report of its own.
///
/// # Arguments
///
/// * `program` - The program.
/// * `templates` - The templates.
/// * `structs` - The struct kinds.
///
/// # Returns
///
/// The diagnostics.
pub(crate) fn leftovers(
    program: &Program, templates: &Templates, structs: &StructTable,
) -> Vec<Diagnostic>
{
    let mut left =
        Leftovers { templates: &templates.structs, structs, diags: Vec::new() };
    let plain = program.stmts.iter().filter(|s| !is_template(s));
    do_for!(stmt in plain, left.visit_stmt(stmt));
    left.diags
}

/// The walker that reports generic structs that kept their kinds.
struct Leftovers<'a>
{
    /// The generic structs.
    templates: &'a HashMap<Name, StructDecl>,
    /// The struct kinds.
    structs: &'a StructTable,
    /// The reports.
    diags: Vec<Diagnostic>,
}

impl Leftovers<'_>
{
    /// Report a kind, and each kind inside it.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    fn kind(&mut self, kind: &Kind)
    {
        do_for!(inner in inner_of(kind), self.kind(inner));
        do_if!(let Some(ident) = &kind.name, self.report(ident, &kind.args));
    }

    /// Report a name that kept its kinds.
    ///
    /// # Arguments
    ///
    /// * `ident` - The name.
    /// * `args` - Its kinds.
    fn report(&mut self, ident: &Ident, args: &[Kind])
    {
        ret_if!(args.is_empty());
        let Some(template) = self.templates.get(&ident.name)
        else
        {
            return self.diags.push(rd::unknown_kind_name(ident.span, &ident.name));
        };
        let (want, got) = (template.kind_params.len(), args.len());
        let count = rd::kind_arg_count(ident.span, &ident.name, want, got);
        ret_if!(want != got, self.diags.push(count));
        let known = args.iter().all(|a| resolved(a, self.structs));
        do_if!(known, self.diags.push(rd::instances_endless(ident.span, &ident.name)));
    }
}

impl Visitor for Leftovers<'_>
{
    fn visit_kind(&mut self, n: &Kind) { self.kind(n) }

    fn visit_expr(&mut self, n: &Expr)
    {
        if let Expr::Struct(lit) = n
        {
            do_for!(arg in &lit.kind_args, self.kind(arg));
            self.report(&lit.name, &lit.kind_args);
        }
        walk_expr(self, n);
    }
}

impl Checker
{
    /// Check the constraints of each instance of a generic struct, at its
    /// first use.
    pub(crate) fn check_struct_bounds(&mut self)
    {
        let known: Vec<(Name, Name, Vec<DataKind>)> =
            self.known_structs.iter().map(|(n, (t, k))| (*n, *t, k.clone())).collect();
        for (name, template, kinds) in known
        {
            let Some(decl) = self.templates.structs.get(&template).cloned()
            else
            {
                continue;
            };
            let at = self.uses.get(&name).copied().unwrap_or(decl.name.span);
            self.meets_bounds(&decl.kind_params, &kinds, at);
        }
    }

    /// The kind of a struct value that names a generic struct with no kinds:
    /// the instance its place expects, or else the instance the kinds of its
    /// field values give (ZK0341 when one remains unknown). The value asks
    /// for the instance, and a run before the instance exists has no kind.
    ///
    /// # Arguments
    ///
    /// * `template` - The generic struct.
    /// * `n` - The struct value.
    /// * `expect` - The kind its place expects, when known.
    ///
    /// # Returns
    ///
    /// The instance kind, or `None` while it does not exist or after a
    /// report.
    pub(crate) fn generic_struct_lit(
        &mut self, template: &StructDecl, n: &StructLit, expect: Option<DataKind>,
    ) -> Option<DataKind>
    {
        let name = &n.name;
        let kinds = self
            .expected_kinds(template, expect)
            .or_else(|| self.field_kinds(template, n));
        let unknown = rd::kinds_unknown(name.span, &name.name);
        let named =
            |ks: &Vec<DataKind>| ks.iter().all(|k| kind_ast(*k, name.span).is_some());
        let Some(kinds) = kinds.filter(named)
        else
        {
            self.report(unknown);
            return None;
        };
        let instance = instance_name(name.name, &kinds);
        self.struct_wanted.insert(instance, (name.name, kinds));
        self.uses.entry(instance).or_insert(name.span);
        self.chosen.insert((self.top, name.span), instance);
        let kind = self.structs.get(&instance).copied();
        do_if!(
            kind.is_none(),
            self.report(rd::instances_endless(name.span, &name.name))
        );
        kind
    }

    /// The kinds of the instance of a generic struct that a place expects.
    ///
    /// # Arguments
    ///
    /// * `template` - The generic struct.
    /// * `expect` - The kind the place expects.
    ///
    /// # Returns
    ///
    /// The kinds, or `None` when the place expects no instance of it.
    fn expected_kinds(
        &self, template: &StructDecl, expect: Option<DataKind>,
    ) -> Option<Vec<DataKind>>
    {
        let expect = expect?;
        let expect = expect.optional_data().unwrap_or(expect);
        let instance = Name::new(&expect.as_struct()?.name);
        let (of, kinds) = self.known_structs.get(&instance)?;
        (*of == template.name.name).then(|| kinds.clone())
    }

    /// The kinds of the instance of a generic struct that the field values
    /// of a struct value give.
    ///
    /// # Arguments
    ///
    /// * `template` - The generic struct.
    /// * `n` - The struct value.
    ///
    /// # Returns
    ///
    /// The kinds, or `None` when one remains unknown.
    fn field_kinds(
        &mut self, template: &StructDecl, n: &StructLit,
    ) -> Option<Vec<DataKind>>
    {
        let names: Vec<Name> =
            template.kind_params.iter().map(|p| p.name.name).collect();
        let field = |f: &Ident| template.fields.iter().find(|p| p.name.name == f.name);
        let set: Vec<(&Kind, &Expr)> = n
            .fields
            .iter()
            .filter_map(|init| Some((&field(&init.field)?.kind, &init.value)))
            .collect();
        let found = self.quiet_kinds(set.iter().map(|(_, v)| *v));
        let pairs = set.iter().map(|(k, _)| *k).zip(found);
        infer_kinds(&names, pairs, &self.known_structs)
    }
}
