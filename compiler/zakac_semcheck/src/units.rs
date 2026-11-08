use std::collections::HashMap;

use zakac_ast::expr::Ident;
use zakac_ast::kind::Cells;
use zakac_ast::kind::Kind;
use zakac_ast::kind::Wrap;
use zakac_ast::name::Name;
use zakac_ast::stmt::FnDecl;
use zakac_ast::stmt::ImportStmt;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::Stmt;
use zakac_ast::visit::Node;
use zakac_ast::visit::Visitor;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::kinds as kd;
use zakac_diagnostics::resolve as rd;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::enums::stem_of;
use crate::kind::DataKind;
use crate::scope::Binding;
use crate::scope::FnSig;
use crate::semcheck::Checker;

/// The file of each top-level struct, enumeration or kind alias and whether it
/// is exported, indexed on the name, and on the stem of an enumeration or alias
/// name too; the first declaration of a name wins.
pub(crate) type StructHomes = HashMap<Name, (u32, bool)>;

/// Where each top-level struct, enumeration and kind alias is declared.
///
/// # Arguments
///
/// * `program` - The tree.
///
/// # Returns
///
/// The unit and the export mark of each struct.
pub(crate) fn struct_homes(program: &Program) -> StructHomes
{
    let mut homes = StructHomes::new();
    for (i, s) in program.stmts.iter().enumerate()
    {
        let (name, exported) = match s
        {
            Stmt::Struct(d) => (d.name.name, d.exported),
            Stmt::Enum(d) => (d.name.name, d.exported),
            Stmt::Alias(d) => (d.name.name, d.exported),
            _ => continue,
        };
        let home = (program.unit_of(i), exported);
        homes.entry(name).or_insert(home);
        let stemmed = matches!(s, Stmt::Enum(_) | Stmt::Alias(_));
        let stem = tern!(stemmed, Some(stem_of(name.as_str())), None);
        let stem = stem.filter(|st| *st != name.as_str());
        do_if!(let Some(st) = stem, _ = homes.entry(Name::new(st)).or_insert(home));
    }
    homes
}

/// Report each kind of an exported declaration that names a struct its file
/// keeps private: a parameter or result of a function, the kind of a
/// variable or constant, or a field of a struct. A file that imports the
/// declaration could not name that struct.
///
/// # Arguments
///
/// * `program` - The tree.
/// * `homes` - The file of each struct and its export mark.
///
/// # Returns
///
/// One `ZK0324` diagnostic per such kind.
pub(crate) fn private_in_exports(
    program: &Program, homes: &StructHomes,
) -> Vec<Diagnostic>
{
    let decls = program.stmts.iter().enumerate();
    let kinds =
        decls.filter_map(|(i, s)| Some((program.unit_of(i), exported_kinds(s)?)));
    let flat = kinds.flat_map(|(unit, (name, kinds))| {
        kinds.into_iter().map(move |kind| (unit, name, kind))
    });
    let private = flat.filter_map(|(unit, name, kind)| {
        let hidden = private_struct(kind, unit, homes)?;
        Some(rd::private_in_export(kind.span, &name.name, hidden))
    });
    private.collect()
}

/// The name and the written kinds of an exported declaration.
///
/// # Arguments
///
/// * `stmt` - The statement.
///
/// # Returns
///
/// The name and the kinds, or `None` for a statement that exports nothing.
fn exported_kinds(stmt: &Stmt) -> Option<(&Ident, Vec<&Kind>)>
{
    match stmt
    {
        Stmt::Fn(f) if f.exported =>
        {
            let params = f.params.iter().map(|p| &p.kind);
            Some((&f.name, params.chain(f.ret.as_ref()).collect()))
        }
        Stmt::Decl(d) if d.exported => Some((&d.name, d.kind.iter().collect())),
        Stmt::Struct(d) if d.exported =>
        {
            Some((&d.name, d.fields.iter().map(|f| &f.kind).collect()))
        }
        Stmt::Alias(d) if d.exported => Some((&d.name, vec![&d.kind])),
        _ => None,
    }
}

/// The struct a kind names, when its file declares it without the export
/// mark.
///
/// # Arguments
///
/// * `kind` - The kind, with each pointer and list level around its base.
/// * `unit` - The file of the declaration that writes the kind.
/// * `homes` - The file of each struct and its export mark.
///
/// # Returns
///
/// The struct name, or `None` for a kind that names no private struct.
fn private_struct<'k>(kind: &'k Kind, unit: u32, homes: &StructHomes)
-> Option<&'k str>
{
    let name = kind.name.as_ref()?;
    let (home, exported) = homes.get(&name.name)?;
    (*home == unit && !exported).then_some(name.name.as_str())
}

/// Report each start block after the first, a file that is run with no start
/// block when it needs one, and each statement that runs in place at the top
/// level of the file that is run. A start block or a statement in an imported
/// file is reported as a statement there.
///
/// # Arguments
///
/// * `program` - The tree.
/// * `needs_start` - Whether the file that is run needs a start block.
///
/// # Returns
///
/// The diagnostics.
pub(crate) fn start_rules(program: &Program, needs_start: bool) -> Vec<Diagnostic>
{
    let stmts = program.stmts.iter().enumerate();
    let root: Vec<&Stmt> = stmts
        .filter(|(i, _)| program.is_root(program.unit_of(*i)))
        .map(|(_, s)| s)
        .collect();
    let starts: Vec<Span> = root.iter().filter_map(|s| start_of(s)).collect();
    let mut diags: Vec<Diagnostic> =
        starts.iter().skip(1).map(|&span| rd::second_start(span)).collect();
    let top = root.iter().filter(|s| runs_in_place(s));
    diags.extend(top.map(|s| rd::top_level_stmt(s.span())));
    let file_start = root.first().map_or(program.span, |s| s.span()).shrink_to_lo();
    do_if!(needs_start && starts.is_empty(), diags.push(rd::no_start(file_start)));
    diags
}

/// Whether a written kind is a plain list of text, `စာသား စာရင်း`.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// `true` for a text list with no count and no borrow marker.
fn is_text_list(kind: &Kind) -> bool
{
    kind.base.is_text() && kind.wraps == [Wrap::List(Cells::Inferred)] && !kind.borrowed
}

/// Whether a top-level statement runs where it stands, rather than
/// declaring something.
///
/// # Arguments
///
/// * `stmt` - The statement.
///
/// # Returns
///
/// `false` for a function, a struct, an enumeration, a kind alias, a variable,
/// a constant or an import.
fn runs_in_place(stmt: &Stmt) -> bool
{
    !matches!(
        stmt,
        Stmt::Fn(_)
            | Stmt::Struct(_)
            | Stmt::Enum(_)
            | Stmt::Alias(_)
            | Stmt::Decl(_)
            | Stmt::Import(_)
    )
}

/// The header word of a start block.
///
/// # Arguments
///
/// * `stmt` - The statement.
///
/// # Returns
///
/// The span of `ပရိုဂရမ်`, or `None` when the statement is no start block.
fn start_of(stmt: &Stmt) -> Option<Span>
{
    let decl = tern!(let Stmt::Fn(decl) = stmt, Some(decl), None)?;
    tern!(decl.start, Some(decl.name.span), None)
}

/// The names of the units of a program, as messages show them.
///
/// # Arguments
///
/// * `program` - The tree.
///
/// # Returns
///
/// One name per unit; blank for a program with no units.
pub(crate) fn unit_names(program: &Program) -> Vec<String>
{
    program.units.iter().map(|u| u.name.clone()).collect()
}

impl Checker
{
    /// Check one top-level statement, after moving to the file that holds
    /// it.
    ///
    /// # Arguments
    ///
    /// * `program` - The tree.
    /// * `index` - The index of the statement.
    /// * `stmt` - The statement.
    pub(crate) fn visit_top(&mut self, program: &Program, index: usize, stmt: &Stmt)
    {
        let unit = program.unit_of(index);
        self.enter_unit(unit);
        let runs = runs_in_place(stmt) || start_of(stmt).is_some();
        let name = || self.unit_names[unit as usize].clone();
        let imported = !program.is_root(unit);
        do_if!(
            runs && imported,
            self.report(rd::stmt_in_imported_file(stmt.span(), &name()))
        );
        self.visit_stmt(stmt);
    }

    /// Check the header of the start block: no parameter, one or two, each a
    /// plain text list, and no result or an integer one.
    ///
    /// # Arguments
    ///
    /// * `n` - The start block.
    /// * `ret` - The kind of its result, when it has one.
    pub(crate) fn check_start_header(&mut self, n: &FnDecl, ret: Option<DataKind>)
    {
        let bad = n.params.iter().enumerate();
        let bad = bad.filter(|(i, p)| *i >= 2 || p.edit || !is_text_list(&p.kind));
        do_for!((_, p) in bad, self.report(kd::start_param_kind(p.span)));
        let not_int = ret.filter(|k| !k.is_int()).zip(n.ret.as_ref());
        do_if!(
            let Some((kind, written)) = not_int,
            self.report(kd::start_result_kind(written.span, &kind.name()))
        );
    }

    /// Move to a file: keep the top-level names of the files that ended,
    /// and start with a blank scope and the functions of the new file.
    ///
    /// # Arguments
    ///
    /// * `unit` - The unit of the new file.
    fn enter_unit(&mut self, unit: u32)
    {
        ret_if!(unit == self.unit);
        while self.saved.len() < unit as usize
        {
            let top = self.scopes.restart();
            self.saved.push(top);
        }
        self.fns = self.unit_fns[unit as usize].clone();
        self.unit = unit;
    }

    /// Bring the names of an import into the scope of its file.
    ///
    /// # Arguments
    ///
    /// * `n` - The import; one the loader did not read brings nothing.
    pub(crate) fn import(&mut self, n: &ImportStmt)
    {
        let Some(target) = n.unit
        else
        {
            return;
        };
        let file = &n.path.lit.value;
        do_for!(name in &n.names, self.import_name(target, file, name));
    }

    /// Bring each exported item of one name into scope: a function, a
    /// variable or constant, or a struct.
    ///
    /// # Arguments
    ///
    /// * `target` - The unit of the imported file.
    /// * `file` - The imported file, as written.
    /// * `name` - The name.
    fn import_name(&mut self, target: u32, file: &str, name: &Ident)
    {
        let wanted = &name.name;
        let sigs: Vec<FnSig> =
            self.unit_fns[target as usize].get(wanted).map_or_else(Vec::new, |list| {
                list.iter().filter(|s| s.exported).cloned().collect()
            });
        let saved = self.saved.get(target as usize).and_then(|s| s.get(wanted));
        let binding = saved.filter(|b| b.exported).copied();
        let home = self.homes.get(wanted).copied();
        let is_struct = home == Some((target, true));
        let found = !sigs.is_empty() || binding.is_some() || is_struct;
        do_if!(!sigs.is_empty(), self.import_fn(name, sigs));
        do_if!(let Some(binding) = binding, self.import_binding(name, binding));
        ret_if!(found);
        let private = self.declared_in(target, wanted);
        self.report(rd::import_not_found(name.span, file, wanted, private));
    }

    /// Bring the exported functions of an imported name into scope, each
    /// overload of it. A name the file declares or imports before is ZK0304.
    ///
    /// # Arguments
    ///
    /// * `name` - The imported name.
    /// * `sigs` - The functions of that name.
    fn import_fn(&mut self, name: &Ident, sigs: Vec<FnSig>)
    {
        let taken = self.fns.contains_key(&name.name);
        let diag = rd::duplicate_function(name.span, &name.name);
        ret_if!(taken, self.report(diag));
        do_if!(let Some(first) = sigs.first(), self.refer(&name.name, name.span, first.decl));
        let sigs = sigs.into_iter().map(|sig| FnSig { exported: false, ..sig });
        self.fns.insert(name.name, sigs.collect());
    }

    /// Bring an imported variable or constant into scope.
    ///
    /// # Arguments
    ///
    /// * `name` - The imported name.
    /// * `binding` - What it stands for in its file.
    fn import_binding(&mut self, name: &Ident, binding: Binding)
    {
        self.refer(&name.name, name.span, binding.home());
        let from = Some(binding.home());
        let binding = Binding { span: name.span, from, exported: false, ..binding };
        let fresh = self.scopes.declare(name.name, binding).is_ok();
        do_if!(!fresh, self.report(rd::redeclaration(name.span, &name.name)));
    }

    /// Whether a file declares a top-level name, exported or not.
    ///
    /// # Arguments
    ///
    /// * `unit` - The unit of the file.
    /// * `name` - The name.
    ///
    /// # Returns
    ///
    /// `true` when it does.
    fn declared_in(&self, unit: u32, name: &str) -> bool
    {
        let index = unit as usize;
        let as_fn = self.unit_fns[index].contains_key(name);
        let as_value = self.saved.get(index).is_some_and(|s| s.contains_key(name));
        let as_struct = self.homes.get(name).is_some_and(|(u, _)| *u == unit);
        as_fn || as_value || as_struct
    }

    /// Add help to an unknown-name report when another file declares the
    /// name.
    ///
    /// # Arguments
    ///
    /// * `diag` - The report.
    /// * `name` - The name.
    ///
    /// # Returns
    ///
    /// The report, with the help when it applies.
    pub(crate) fn with_hidden(&self, diag: Diagnostic, name: &str) -> Diagnostic
    {
        let units = (0..self.unit_names.len() as u32).filter(|u| *u != self.unit);
        let mut homes = units.filter(|u| self.declared_in(*u, name));
        let Some(home) = homes.next()
        else
        {
            return diag;
        };
        ret_if!(diag.help.is_some(), diag);
        let file = &self.unit_names[home as usize];
        diag.with_help(rd::hidden_help(name, file))
    }
}
