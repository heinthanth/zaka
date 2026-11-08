use std::collections::HashMap;

use zakac_ast::kind::Kind;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::Stmt;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::walk_kind;
use zakac_rust_sugars::do_if;
use zakac_span::span::Span;

use crate::enums::stem_of;
use crate::semcheck::RefTable;

/// The declared names of the structs, their fields, the enumerations, their
/// members and the top-level constants, in all files of the program; the
/// first declaration of a name wins, as in the check.
#[derive(Debug, Clone, Default)]
pub(crate) struct KindDecls
{
    /// Each struct and enumeration name, and the stem of an enumeration name.
    names: HashMap<String, Span>,
    /// Each field, on its struct and field names.
    fields: HashMap<(String, String), Span>,
    /// Each member, on its enumeration and member names.
    members: HashMap<(String, String), Span>,
    /// Each top-level constant, on its file and name.
    consts: HashMap<(u32, String), Span>,
}

impl KindDecls
{
    /// Gather the declared names of a program.
    ///
    /// # Arguments
    ///
    /// * `program` - The program.
    ///
    /// # Returns
    ///
    /// The names.
    pub(crate) fn of(program: &Program) -> Self
    {
        let mut decls = KindDecls::default();
        program.stmts.iter().for_each(|s| decls.add(s));
        for (i, stmt) in program.stmts.iter().enumerate()
        {
            let Stmt::Decl(d) = stmt
            else
            {
                continue;
            };
            let at = (program.unit_of(i), d.name.name.to_string());
            do_if!(d.is_const, _ = decls.consts.entry(at).or_insert(d.name.span));
        }
        decls
    }

    /// Add the names a statement declares.
    ///
    /// # Arguments
    ///
    /// * `stmt` - The statement.
    fn add(&mut self, stmt: &Stmt)
    {
        match stmt
        {
            Stmt::Struct(s) =>
            {
                let name = s.name.name.to_string();
                self.names.entry(name.clone()).or_insert(s.name.span);
                for f in &s.fields
                {
                    let at = (name.clone(), f.name.name.to_string());
                    self.fields.entry(at).or_insert(f.name.span);
                }
            }
            Stmt::Enum(e) =>
            {
                let name = e.name.name.to_string();
                self.names.entry(stem_of(&name).to_string()).or_insert(e.name.span);
                self.names.entry(name.clone()).or_insert(e.name.span);
                for m in &e.members
                {
                    let at = (name.clone(), m.name.name.to_string());
                    self.members.entry(at).or_insert(m.name.span);
                }
            }
            _ => (),
        }
    }

    /// The declaration of a struct or enumeration name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    ///
    /// # Returns
    ///
    /// The declared name, or `None` for a name no file declares.
    pub(crate) fn name(&self, name: &str) -> Option<Span>
    {
        self.names.get(name).copied()
    }

    /// The declaration of a field.
    ///
    /// # Arguments
    ///
    /// * `owner` - The struct name.
    /// * `field` - The field name.
    ///
    /// # Returns
    ///
    /// The declared field name, or `None` when the struct has no such field.
    pub(crate) fn field(&self, owner: &str, field: &str) -> Option<Span>
    {
        self.fields.get(&(owner.to_string(), field.to_string())).copied()
    }

    /// The declaration of a member.
    ///
    /// # Arguments
    ///
    /// * `owner` - The enumeration name.
    /// * `member` - The member name.
    ///
    /// # Returns
    ///
    /// The declared member name, or `None` when the enumeration has no such
    /// member.
    pub(crate) fn member(&self, owner: &str, member: &str) -> Option<Span>
    {
        self.members.get(&(owner.to_string(), member.to_string())).copied()
    }

    /// The declaration of a top-level constant.
    ///
    /// # Arguments
    ///
    /// * `unit` - The file.
    /// * `name` - The constant name.
    ///
    /// # Returns
    ///
    /// The declared name, or `None` when the file declares no such constant.
    pub(crate) fn constant(&self, unit: u32, name: &str) -> Option<Span>
    {
        self.consts.get(&(unit, name.to_string())).copied()
    }

    /// Each declared name with itself, so a cursor on a declaration finds it.
    ///
    /// # Returns
    ///
    /// The pairs.
    pub(crate) fn own(&self) -> RefTable
    {
        let all = self
            .names
            .values()
            .chain(self.fields.values())
            .chain(self.members.values());
        all.map(|&span| (span, span)).collect()
    }
}

/// Each struct or enumeration name written in a kind, with its declaration.
///
/// # Arguments
///
/// * `program` - The program.
/// * `decls` - The declared names.
///
/// # Returns
///
/// The pairs.
pub(crate) fn kind_refs(program: &Program, decls: &KindDecls) -> RefTable
{
    let mut finder = KindNames { decls, refs: RefTable::new() };
    finder.visit_program(program);
    finder.refs
}

/// Finds the struct and enumeration names written in kinds.
struct KindNames<'a>
{
    /// The declared names.
    decls: &'a KindDecls,
    /// The pairs found so far.
    refs: RefTable,
}

impl Visitor for KindNames<'_>
{
    fn visit_kind(&mut self, n: &Kind)
    {
        let named =
            n.name.as_ref().and_then(|i| Some((i.span, self.decls.name(&i.name)?)));
        self.refs.extend(named);
        walk_kind(self, n);
    }
}
