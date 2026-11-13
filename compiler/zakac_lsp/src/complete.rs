use std::collections::BTreeMap;
use std::collections::HashMap;

use gen_lsp_types::CompletionItem;
use gen_lsp_types::CompletionItemKind;
use gen_lsp_types::Position;
use zakac_ast::expr::Ident;
use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Kind;
use zakac_ast::kind::Wrap;
use zakac_ast::stmt::BlockStmt;
use zakac_ast::stmt::CountedStmt;
use zakac_ast::stmt::DeclStmt;
use zakac_ast::stmt::EachStmt;
use zakac_ast::stmt::EnumMember;
use zakac_ast::stmt::FnDecl;
use zakac_ast::stmt::GenericStmt;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::Stmt;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::walk_block;
use zakac_ast::visit::walk_counted;
use zakac_ast::visit::walk_each;
use zakac_ast::visit::walk_fn;
use zakac_ast::visit::walk_generic;
use zakac_lexer::phrases::PHRASES;
use zakac_lexer::reserved::RESERVED;
use zakac_lexer::role::TokenRole;
use zakac_lexer::role::kw_role;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::Token;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::enums::stem_of;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::semcheck::KindTable;
use zakac_semcheck::structs::StructTable;
use zakac_span::span::Span;

use crate::check::FileCheck;
use crate::outline::root_stmts;
use crate::outline::shown_name;

/// What the editor offers while a word is written: the reserved words and
/// phrases, and the names in scope at the cursor. The editor filters them on
/// the written text.
///
/// # Arguments
///
/// * `file` - The check of the open file, when there is one.
/// * `pos` - The cursor.
///
/// # Returns
///
/// The items.
pub fn completions(file: Option<&FileCheck>, pos: Position) -> Vec<CompletionItem>
{
    // After `၏` or `ထဲမှ` the cursor wants a field or a member, nothing else.
    let linked = file.and_then(|f| after_link(f, pos));
    ret_if!(let Some(items) = linked, items);
    let words = RESERVED.iter().filter(|(_, kw)| kw_role(*kw) != TokenRole::Particle);
    let words = words.map(|(word, kw)| reserved_item(word.to_string(), *kw));
    let phrases = PHRASES
        .iter()
        .filter_map(|(pieces, kw)| Some(reserved_item(spell(pieces)?, *kw)));
    let names = file.map(|f| names_in_scope(f, pos)).unwrap_or_default();
    words.chain(phrases).chain(names).collect()
}

/// A reserved word or phrase as an item, with its role as the detail.
///
/// # Arguments
///
/// * `label` - The spelling.
/// * `kw` - The reserved word.
///
/// # Returns
///
/// The item.
fn reserved_item(label: String, kw: Kw) -> CompletionItem
{
    let role: &'static str = kw_role(kw).into();
    item(label, CompletionItemKind::Keyword, role)
}

/// The spelling of a phrase: the first spelling of each piece, a particle
/// joined to the word before it and the other pieces apart.
///
/// # Arguments
///
/// * `pieces` - The reserved words of the phrase.
///
/// # Returns
///
/// The spelling, or `None` when a piece has no spelling of its own.
fn spell(pieces: &[Kw]) -> Option<String>
{
    let word = |kw: &Kw| RESERVED.iter().find(|(_, k)| k == kw).map(|(w, _)| *w);
    pieces.iter().try_fold(String::new(), |mut text, kw| {
        text.push_str(tern!(text.is_empty() || kw.is_particle(), "", " "));
        text.push_str(word(kw)?);
        Some(text)
    })
}

/// The fields after `<name> ၏` and the members after `<enumeration>ထဲမှ`. A
/// chain of fields, `က ၏ နေရာ ၏`, follows the kind of each field.
///
/// # Arguments
///
/// * `file` - The check of the open file.
/// * `pos` - The cursor.
///
/// # Returns
///
/// The items, or `None` when the cursor follows neither word.
fn after_link(file: &FileCheck, pos: Position) -> Option<Vec<CompletionItem>>
{
    let at = file.lines.offset(pos);
    let ends = |t: &&Token| file.root_range(&t.span).hi;
    // The chain holds to the cursor's line: an unfinished line above must not
    // join it, as `က ၏` on one line and `က ၏ အိမ် ၏` on the next.
    let line_start = file.lines.starts().get(pos.line as usize).copied().unwrap_or(0);
    let starts = |t: &&Token| file.root_range(&t.span).lo;
    let live = file.lexed.tokens.iter().filter(|t| t.kind != TokenKind::Eof);
    let on_line = live.filter(|t| starts(t) >= line_start);
    let before: Vec<&Token> = on_line.filter(|t| ends(t) <= at).collect();
    // A name that ends at the cursor is the part the editor filters on.
    let writing =
        before.last().is_some_and(|t| t.kind == TokenKind::Ident && ends(t) == at);
    let before = &before[..before.len() - usize::from(writing)];
    let (link, head) = before.split_last()?;
    let program = &file.checked.as_ref()?.lowered.checked.program;
    match link.kind
    {
        TokenKind::Punct(Punct::Ei) => fields_after(file, head, pos),
        TokenKind::Kw(Kw::Among) => members_after(file, program, head.last()?),
        _ => None,
    }
}

/// The fields of the struct a chain ends in: `က ၏ နေရာ ၏`, or
/// `l ၏ အကွက် ၁ ၏`. The kind of each step comes from the check, so the
/// number of a cell does not matter: each cell of a list has its kind.
///
/// # Arguments
///
/// * `file` - The check of the open file.
/// * `head` - The tokens before the last `၏`.
/// * `pos` - The cursor.
///
/// # Returns
///
/// The fields, or `None` when the chain ends in no struct.
fn fields_after(
    file: &FileCheck, head: &[&Token], pos: Position,
) -> Option<Vec<CompletionItem>>
{
    let (root, steps) = chain(file, head)?;
    let scope = scope_at(file, pos)?;
    let start = scope.kinds.get(&root).copied()?;
    let end = steps.iter().try_fold(start, |kind, step| step.follow(kind))?;
    let fields = end.as_struct()?.fields();
    let field = |(name, kind): &(String, DataKind)| {
        item(name.clone(), CompletionItemKind::Field, &kind.name())
    };
    Some(fields.iter().map(field).collect())
}

/// One step of a chain after its first name.
enum Step
{
    /// A field, named.
    Field(String),
    /// A cell of a list: `အကွက် <number>` or `နောက်ဆုံးအကွက်`.
    Cell,
}

impl Step
{
    /// The kind the step leads to, under the rules of the check: a field of a
    /// struct, and a cell of a list, a slice, or the list a pointer holds.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind before the step.
    ///
    /// # Returns
    ///
    /// The kind after it, or `None` when the step does not fit the kind.
    fn follow(&self, kind: DataKind) -> Option<DataKind>
    {
        match self
        {
            Step::Field(name) => kind.as_struct()?.field(name).map(|(_, k)| k),
            Step::Cell =>
            {
                let held = kind.cell_kind();
                held.or_else(|| kind.pointee().and_then(DataKind::cell_kind))
            }
        }
    }
}

/// The first name and the steps of a chain `a ၏ အကွက် ၁ ၏ b`, read back
/// from its end.
///
/// # Arguments
///
/// * `file` - The check of the open file.
/// * `head` - The tokens that end in the chain.
///
/// # Returns
///
/// The first name and the steps in order, or `None` when the tokens do not
/// end in a chain.
fn chain(file: &FileCheck, head: &[&Token]) -> Option<(String, Vec<Step>)>
{
    let is_ei = |t: &&Token| t.kind == TokenKind::Punct(Punct::Ei);
    let mut steps = Vec::new();
    let mut rest = head;
    loop
    {
        // A cell runs from the `၏` before it to the end, whatever its number.
        let link = rest.iter().rposition(is_ei);
        let tail = &rest[link.map_or(0, |at| at + 1)..];
        let cell = tail.first().is_some_and(|t| is_cell_word(t.kind));
        if let (true, Some(at)) = (cell, link)
        {
            steps.push(Step::Cell);
            rest = &rest[..at];
            continue;
        }
        let (last, before) = rest.split_last()?;
        ret_if!(last.kind != TokenKind::Ident, None);
        let name = file.sm.span_to_snippet(&last.span).ok()?;
        let Some(before) =
            before.split_last().filter(|(t, _)| is_ei(t)).map(|(_, b)| b)
        else
        {
            steps.reverse();
            return Some((name, steps));
        };
        steps.push(Step::Field(name));
        rest = before;
    }
}

/// Whether a reserved word starts a cell of a list.
///
/// # Arguments
///
/// * `kind` - The token kind.
///
/// # Returns
///
/// `true` for `အကွက်` and `နောက်ဆုံးအကွက်`.
fn is_cell_word(kind: TokenKind) -> bool
{
    matches!(kind, TokenKind::Kw(Kw::Cell | Kw::LastCell))
}

/// The members of the enumeration before `ထဲမှ`, written with `များ`, or of
/// the alias that names it.
///
/// # Arguments
///
/// * `file` - The check of the open file.
/// * `program` - The program.
/// * `qualifier` - The name before `ထဲမှ`.
///
/// # Returns
///
/// The members, or `None` for a name that is no enumeration.
fn members_after(
    file: &FileCheck, program: &Program, qualifier: &Token,
) -> Option<Vec<CompletionItem>>
{
    ret_if!(qualifier.kind != TokenKind::Ident, None);
    let written = file.sm.span_to_snippet(&qualifier.span).ok()?;
    let stem = stem_of(&written).to_string();
    let named = |name: &str| name == written || stem_of(name) == stem;
    let alias = program.stmts.iter().find_map(|s| match s
    {
        Stmt::Alias(a) if named(a.name.name.as_str()) => struct_of(&a.kind),
        _ => None,
    });
    let target = alias.map_or(stem.clone(), |a| stem_of(&a).to_string());
    let decl = program.stmts.iter().find_map(|s| match s
    {
        Stmt::Enum(e) if stem_of(e.name.name.as_str()) == target => Some(e),
        _ => None,
    })?;
    let member =
        |m: &EnumMember| item(shown_name(&m.name), CompletionItemKind::EnumMember, "");
    Some(decl.members.iter().map(member).collect())
}

/// The struct or enumeration a written kind names, at once or through
/// pointers.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// The name, or `None` for another kind.
fn struct_of(kind: &Kind) -> Option<String>
{
    let through = kind.wraps.iter().all(|w| matches!(w, Wrap::Ptr));
    let named = kind.base == BaseKind::Named && through;
    named.then(|| kind.name.as_ref().map(|n| n.name.to_string())).flatten()
}

/// The names in scope at the cursor: the top-level names of the open file
/// and the names it imports, then, inside a function, its parameters and
/// result, and the variables and loop variables of each block around the
/// cursor that are declared before it. A private name of another file is never
/// in scope.
///
/// # Arguments
///
/// * `file` - The check of the open file.
/// * `pos` - The cursor.
///
/// # Returns
///
/// The items, each name once.
fn names_in_scope(file: &FileCheck, pos: Position) -> Vec<CompletionItem>
{
    let Some(scope) = scope_at(file, pos)
    else
    {
        return Vec::new();
    };
    scope.names.into_iter().map(|(name, kind)| item(name, kind, "")).collect()
}

/// The names in scope at the cursor, with the struct each holds.
///
/// # Arguments
///
/// * `file` - The check of the open file.
/// * `pos` - The cursor.
///
/// # Returns
///
/// The scope, or `None` when the file could not be read.
fn scope_at(file: &FileCheck, pos: Position) -> Option<Scope<'_>>
{
    let checked = &file.checked.as_ref()?.lowered.checked;
    let program = &checked.program;
    #[rustfmt::skip]
    let mut scope = Scope { file, at: file.lines.offset(pos), names: BTreeMap::new(),
        table: &checked.kinds, structs: &checked.structs, kinds: HashMap::new() };
    root_stmts(program).for_each(|s| scope.top(s, program));
    Some(scope)
}

/// The item kind of a name a statement declares.
///
/// # Arguments
///
/// * `stmt` - The statement.
///
/// # Returns
///
/// The declared name and its item kind, or `None` for a statement that
/// declares no name that other statements use.
fn declared(stmt: &Stmt) -> Option<(&Ident, CompletionItemKind)>
{
    match stmt
    {
        Stmt::Fn(f) if !f.start && !f.name.name.contains('<') =>
        {
            Some((&f.name, CompletionItemKind::Function))
        }
        Stmt::Trait(t) => Some((&t.name, CompletionItemKind::Interface)),
        Stmt::Struct(s) => Some((&s.name, CompletionItemKind::Struct)),
        Stmt::Enum(e) => Some((&e.name, CompletionItemKind::Enum)),
        Stmt::Alias(a) => Some((&a.name, CompletionItemKind::TypeParameter)),
        Stmt::Decl(d) => Some((&d.name, decl_kind(d))),
        _ => None,
    }
}

/// The item kind of a variable or a constant.
///
/// # Arguments
///
/// * `d` - The declaration.
///
/// # Returns
///
/// The kind.
fn decl_kind(d: &DeclStmt) -> CompletionItemKind
{
    tern!(d.is_const, CompletionItemKind::Constant, VARIABLE)
}

/// The item kind of a variable.
const VARIABLE: CompletionItemKind = CompletionItemKind::Variable;

/// An item with a label, a kind and a detail.
///
/// # Arguments
///
/// * `label` - The text to insert.
/// * `kind` - What it is.
/// * `detail` - A note beside it; blank for none.
///
/// # Returns
///
/// The item.
fn item(label: String, kind: CompletionItemKind, detail: &str) -> CompletionItem
{
    CompletionItem {
        label,
        kind: Some(kind),
        detail: (!detail.is_empty()).then(|| detail.to_string()),
        ..Default::default()
    }
}

/// Gathers the names in scope at the cursor.
struct Scope<'a>
{
    /// The check of the open file.
    file: &'a FileCheck,
    /// The cursor, in UTF-8 offsets of the text the editor holds.
    at: u32,
    /// The names found, with their item kinds.
    names: BTreeMap<String, CompletionItemKind>,
    /// The kind of each expression the check knows.
    table: &'a KindTable,
    /// The kind of each declared struct, to read written kinds.
    structs: &'a StructTable,
    /// The kind of each variable, parameter and loop variable found, on its
    /// name.
    kinds: HashMap<String, DataKind>,
}

impl Scope<'_>
{
    /// Add a name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    /// * `kind` - Its item kind.
    fn add(&mut self, name: &Ident, kind: CompletionItemKind)
    {
        self.names.insert(shown_name(name), kind);
    }

    /// Record the kind of a name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    /// * `kind` - Its kind.
    fn note(&mut self, name: &Ident, kind: DataKind)
    {
        self.kinds.insert(shown_name(name), kind);
    }

    /// Record the kind of a variable or constant: its written kind, or the
    /// kind the check found for its value.
    ///
    /// # Arguments
    ///
    /// * `d` - The declaration.
    fn note_decl(&mut self, d: &DeclStmt)
    {
        let written = d.kind.as_ref().map(|k| DataKind::from_ast(k, self.structs));
        let value = d.init.as_ref().and_then(|e| self.table.get(&e.id()).copied());
        do_if!(let Some(kind) = written.or(value), self.note(&d.name, kind));
    }

    /// Whether a span of the open file holds the cursor.
    ///
    /// # Arguments
    ///
    /// * `span` - The span.
    ///
    /// # Returns
    ///
    /// `true` when it does.
    fn holds(&self, span: &Span) -> bool
    {
        let range = self.file.root_range(span);
        range.lo <= self.at && self.at <= range.hi
    }

    /// Add the names a top-level statement brings into scope, and walk into
    /// the function that holds the cursor.
    ///
    /// # Arguments
    ///
    /// * `stmt` - The statement.
    /// * `program` - The program, for the kind of each imported name.
    fn top(&mut self, stmt: &Stmt, program: &Program)
    {
        do_if!(let Some((name, kind)) = declared(stmt), self.add(name, kind));
        do_if!(let Stmt::Decl(d) = stmt, self.note_decl(d));
        do_if!(let Stmt::Fn(f) = stmt, self.visit_fn(f));
        let Stmt::Import(import) = stmt
        else
        {
            return;
        };
        for name in &import.names
        {
            let same =
                |s: &&Stmt| declared(s).is_some_and(|(n, _)| n.name == name.name);
            let home = program.stmts.iter().find(same);
            do_if!(let Some(Stmt::Decl(d)) = home, self.note_decl(d));
            let kind = home.and_then(declared);
            self.add(name, kind.map_or(VARIABLE, |(_, k)| k));
        }
    }
}

impl Visitor for Scope<'_>
{
    fn visit_fn(&mut self, n: &FnDecl)
    {
        ret_if!(!self.holds(&n.span));
        for p in &n.params
        {
            self.add(&p.name, VARIABLE);
            self.note(&p.name, DataKind::from_ast(&p.kind, self.structs));
        }
        walk_fn(self, n);
    }

    fn visit_block(&mut self, n: &BlockStmt)
    {
        do_if!(self.holds(&n.span), walk_block(self, n));
    }

    fn visit_decl(&mut self, n: &DeclStmt)
    {
        let before = self.file.root_range(&n.span).hi <= self.at;
        do_if!(before, self.add(&n.name, decl_kind(n)));
        do_if!(before, self.note_decl(n));
    }

    fn visit_each(&mut self, n: &EachStmt)
    {
        do_if!(self.holds(&n.span), self.add(&n.var, VARIABLE));
        let cells = self.table.get(&n.source.id()).and_then(|k| Step::Cell.follow(*k));
        do_if!(let Some(cell) = cells.filter(|_| self.holds(&n.span)), self.note(&n.var, cell));
        walk_each(self, n);
    }

    fn visit_counted(&mut self, n: &CountedStmt)
    {
        do_if!(self.holds(&n.span), self.add(&n.var, VARIABLE));
        walk_counted(self, n);
    }

    fn visit_generic(&mut self, n: &GenericStmt)
    {
        do_if!(self.holds(&n.span), self.add(&n.var, VARIABLE));
        walk_generic(self, n);
    }
}
