use std::collections::HashMap;

use zakac_ast::expr::Ident;
use zakac_ast::kind::Kind;
use zakac_ast::name::Name;
use zakac_ast::stmt::ImportStmt;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::Stmt;
use zakac_ast::visit::Node;
use zakac_ast::visit::Visitor;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::resolve as rd;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use crate::scope::Binding;
use crate::scope::FnSig;
use crate::semcheck::Checker;

/// The file of each top-level struct and whether it is exported, indexed on
/// the struct name; the first declaration of a name wins.
pub(crate) type StructHomes = HashMap<Name, (u32, bool)>;

/// Where each top-level struct is declared.
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
        let Stmt::Struct(decl) = s
        else
        {
            continue;
        };
        let home = (program.unit_of(i), decl.exported);
        homes.entry(decl.name.name).or_insert(home);
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
        Stmt::Decl(d) if d.exported => Some((&d.name, vec![&d.kind])),
        Stmt::Struct(d) if d.exported =>
        {
            Some((&d.name, d.fields.iter().map(|f| &f.kind).collect()))
        }
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
        let runs = !matches!(
            stmt,
            Stmt::Fn(_) | Stmt::Struct(_) | Stmt::Decl(_) | Stmt::Import(_)
        );
        let name = || self.unit_names[unit as usize].clone();
        let imported = !program.is_root(unit);
        do_if!(
            runs && imported,
            self.report(rd::stmt_in_imported_file(stmt.span(), &name()))
        );
        self.visit_stmt(stmt);
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
        let sig =
            self.unit_fns[target as usize].get(wanted).filter(|s| s.exported).cloned();
        let saved = self.saved.get(target as usize).and_then(|s| s.get(wanted));
        let binding = saved.filter(|b| b.exported).copied();
        let home = self.homes.get(wanted).copied();
        let is_struct = home == Some((target, true));
        let found = sig.is_some() || binding.is_some() || is_struct;
        do_if!(let Some(sig) = sig, self.import_fn(name, sig));
        do_if!(let Some(binding) = binding, self.import_binding(name, binding));
        ret_if!(found);
        let private = self.declared_in(target, wanted);
        self.report(rd::import_not_found(name.span, file, wanted, private));
    }

    /// Bring an imported function into scope.
    ///
    /// # Arguments
    ///
    /// * `name` - The imported name.
    /// * `sig` - The function.
    fn import_fn(&mut self, name: &Ident, sig: FnSig)
    {
        let taken = self.fns.contains_key(&name.name);
        let diag = rd::duplicate_function(name.span, &name.name);
        ret_if!(taken, self.report(diag));
        let sig = FnSig { exported: false, ..sig };
        self.fns.insert(name.name, sig);
    }

    /// Bring an imported variable or constant into scope.
    ///
    /// # Arguments
    ///
    /// * `name` - The imported name.
    /// * `binding` - What it stands for in its file.
    fn import_binding(&mut self, name: &Ident, binding: Binding)
    {
        let binding = Binding { span: name.span, exported: false, ..binding };
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
