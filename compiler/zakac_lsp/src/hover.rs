use bon::Builder;
use gen_lsp_types::Contents;
use gen_lsp_types::Hover;
use gen_lsp_types::MarkupContent;
use gen_lsp_types::MarkupKind;
use gen_lsp_types::Position;
use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::NodeId;
use zakac_ast::kind::Kind;
use zakac_ast::stmt::AliasDecl;
use zakac_ast::stmt::CountedStmt;
use zakac_ast::stmt::DeclStmt;
use zakac_ast::stmt::EachStmt;
use zakac_ast::stmt::EnumDecl;
use zakac_ast::stmt::FnDecl;
use zakac_ast::stmt::GenericStmt;
use zakac_ast::stmt::StructDecl;
use zakac_ast::visit::Node;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::walk_alias;
use zakac_ast::visit::walk_counted;
use zakac_ast::visit::walk_decl;
use zakac_ast::visit::walk_each;
use zakac_ast::visit::walk_enum;
use zakac_ast::visit::walk_expr;
use zakac_ast::visit::walk_fn;
use zakac_ast::visit::walk_generic;
use zakac_ast::visit::walk_struct;
use zakac_i18n::t;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex;
use zakac_lexer::token::Comment;
use zakac_lexer::token::CommentKind;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::artifact::Checked;
use zakac_semcheck::enums::MemberValue;
use zakac_semcheck::enums::stem_of;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::semcheck::KindTable;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::pos::Pos;
use zakac_span::span::Span;
use zakac_span::text_range::Range;

use crate::check::FileCheck;
use crate::outline::shown_name;

/// The words a declaration starts with, as Zaka writes them.
const VAR: &str = "ကိန်းရှင်";
/// The word of a constant.
const CONST: &str = "ကိန်းသေ";
/// The word of a function.
const FUNCTION: &str = "လုပ်ဆောင်ချက်";
/// The words of a struct or an enumeration.
const DATA_KIND: &str = "ဒေတာအမျိုးအစား";
/// The word between a value and its field.
const EI: &str = "၏";
/// The words between an enumeration and its member.
const AMONG: &str = "များထဲမှ";

/// What a hover shows: a line of Zaka in a code block, as the declaration
/// reads, then notes under a rule.
#[derive(Debug, Clone, Builder)]
struct Card
{
    /// The range the hover is about, in the text the editor holds.
    range: Range,
    /// The Zaka line.
    code: String,
    /// The `///` lines above the declaration, without their marks.
    doc: Option<String>,
    /// The notes; none for a hover with the line alone.
    #[builder(default)]
    notes: Vec<String>,
}

impl Card
{
    /// The hover as Markdown.
    ///
    /// # Returns
    ///
    /// The Markdown text.
    fn markdown(&self) -> String
    {
        let code = format!("```zaka\n{}\n```", self.code);
        let notes = Some(self.notes.join("\n\n")).filter(|n| !n.is_empty());
        let parts = [Some(code), self.doc.clone(), notes];
        parts.into_iter().flatten().collect::<Vec<_>>().join("\n\n---\n\n")
    }
}

/// What the cursor is on: a name, with its declaration as Zaka writes it,
/// what it is and its size; else the kind of the innermost expression.
///
/// # Arguments
///
/// * `file` - The check of the open file.
/// * `pos` - The cursor.
///
/// # Returns
///
/// The hover, or `None` when the check did not run or nothing with a known
/// kind holds the cursor.
pub fn hover(file: &FileCheck, pos: Position) -> Option<Hover>
{
    let checked = &file.checked.as_ref()?.lowered.checked;
    let at = file.lines.offset(pos);
    let card = name_card(file, checked, at).or_else(|| expr_card(file, checked, at))?;
    let value = card.markdown();
    let contents = MarkupContent { kind: MarkupKind::Markdown, value };
    let range = Some(file.lines.range(card.range));
    Some(Hover { contents: Contents::MarkupContent(contents), range })
}

/// The card of the name under the cursor, from its declaration.
///
/// # Arguments
///
/// * `file` - The check of the open file.
/// * `checked` - The check.
/// * `at` - The cursor offset.
///
/// # Returns
///
/// The card, or `None` when the cursor is on no resolved name.
fn name_card(file: &FileCheck, checked: &Checked, at: u32) -> Option<Card>
{
    let holds = |s: &Span| {
        let r = file.root_range(s);
        file.in_root(s) && !s.is_placeholder() && r.lo <= at && at <= r.hi
    };
    let (used, decl) = checked.refs.iter().find(|(used, _)| holds(used))?;
    let mut maker = CardMaker { target: *decl, checked, found: None };
    maker.visit_program(&checked.program);
    let (code, notes, stmt) = maker.found?;
    let doc = stmt.and_then(|stmt| doc_of(file, stmt));
    let card = Card::builder().range(file.root_range(used)).code(code).maybe_doc(doc);
    Some(card.notes(notes).build())
}

/// The `///` lines right above a declaration, each on the line before the
/// next, without their marks.
///
/// # Arguments
///
/// * `file` - The check of the open file.
/// * `stmt` - The declaration, in the open file or a file it imports.
///
/// # Returns
///
/// The lines, or `None` when no `///` line stands right above it.
fn doc_of(file: &FileCheck, stmt: Span) -> Option<String>
{
    let source = file.sm.lookup_source_file(&stmt.lo);
    let lexed = lex(&source, &LexerOptions::default());
    let line = |pos: &Pos| file.sm.lookup_char_pos(pos).line;
    let mut want = line(&stmt.lo);
    let above = lexed.comments.iter().rev().filter(|c| c.span.hi <= stmt.lo);
    let doc = above.take_while(|c| {
        want = want.saturating_sub(1);
        c.kind == CommentKind::DocSlash && line(&c.span.lo) == want
    });
    let text = |c: &Comment| file.sm.span_to_snippet(&c.span).ok();
    let mut lines: Vec<String> = doc.filter_map(text).map(|t| unmarked(&t)).collect();
    lines.reverse();
    Some(lines.join("\n")).filter(|t| !t.is_empty())
}

/// A `///` line without its mark and the one space after it.
///
/// # Arguments
///
/// * `line` - The comment.
///
/// # Returns
///
/// The text.
fn unmarked(line: &str) -> String
{
    let text = line.strip_prefix("///").unwrap_or(line);
    text.strip_prefix(' ').unwrap_or(text).to_string()
}

/// The card of the innermost expression under the cursor: its kind.
///
/// # Arguments
///
/// * `file` - The check of the open file.
/// * `checked` - The check.
/// * `at` - The cursor offset.
///
/// # Returns
///
/// The card, or `None` when no expression with a known kind holds the
/// cursor.
fn expr_card(file: &FileCheck, checked: &Checked, at: u32) -> Option<Card>
{
    let mut finder = Finder { file, kinds: &checked.kinds, at, best: None };
    finder.visit_program(&checked.program);
    let (range, id) = finder.best?;
    let kind = checked.kinds.get(&id)?;
    Some(Card::builder().range(range).code(kind.name()).build())
}

/// The bit count of the platform-sized kinds the editors check with.
fn platform() -> u16 { SemcheckOptions::default().platform_bits }

/// The size and alignment of a kind, as a note.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// The note, or `None` for a kind with no fixed size.
fn layout(kind: DataKind) -> Option<String>
{
    let size = kind.checked_size(platform())?;
    let align = kind.align(platform());
    Some(t!("lsp.hover_layout", size = size, align = align).to_string())
}

/// Builds the card of the declaration with one name span.
struct CardMaker<'a>
{
    /// The declared name to find.
    target: Span,
    /// The check.
    checked: &'a Checked,
    /// The line, the notes and the declaration statement, once found.
    found: Option<(String, Vec<String>, Option<Span>)>,
}

impl CardMaker<'_>
{
    /// Keep the first card found.
    ///
    /// # Arguments
    ///
    /// * `code` - The Zaka line.
    /// * `notes` - The notes.
    /// * `stmt` - The declaration statement, whose `///` lines the hover shows.
    fn keep(&mut self, code: String, notes: Vec<Option<String>>, stmt: Option<Span>)
    {
        let notes = notes.into_iter().flatten().collect();
        do_if!(self.found.is_none(), self.found = Some((code, notes, stmt)));
    }

    /// Whether a name is the one to find.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    ///
    /// # Returns
    ///
    /// `true` when it is.
    fn is_target(&self, name: &Ident) -> bool { name.span == self.target }

    /// The kind a written kind stands for.
    ///
    /// # Arguments
    ///
    /// * `kind` - The written kind.
    ///
    /// # Returns
    ///
    /// The kind.
    fn kind_of(&self, kind: &Kind) -> DataKind
    {
        DataKind::from_ast(kind, &self.checked.structs)
    }

    /// The kind the check found for an expression.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    ///
    /// # Returns
    ///
    /// The kind, or `None` when it is not known.
    fn kind_at(&self, e: &Expr) -> Option<DataKind>
    {
        self.checked.kinds.get(&e.id()).copied()
    }

    /// The head of a function as Zaka reads it in a hover:
    /// `လုပ်ဆောင်ချက် ရွှေ့(က: နေရာ၊ ခ: နေရာ): နေရာ`.
    ///
    /// # Arguments
    ///
    /// * `n` - The function.
    ///
    /// # Returns
    ///
    /// The head.
    fn head(&self, n: &FnDecl) -> String
    {
        let param = |p: &zakac_ast::stmt::Param| {
            format!("{}: {}", shown_name(&p.name), self.kind_of(&p.kind).name())
        };
        let params: Vec<String> = n.params.iter().map(param).collect();
        let ret = n.ret.as_ref().map(|k| format!(": {}", self.kind_of(k).name()));
        let name = shown_name(&n.name);
        format!("{FUNCTION} {name}({}){}", params.join("၊ "), ret.unwrap_or_default())
    }

    /// The card of a struct and its fields, or of one of its fields.
    ///
    /// # Arguments
    ///
    /// * `n` - The struct.
    fn struct_card(&mut self, n: &StructDecl)
    {
        let kind = self.checked.structs.get(&n.name.name).copied();
        let fields = kind.and_then(DataKind::as_struct).map(|s| s.fields().to_vec());
        let fields = fields.unwrap_or_default();
        let name = shown_name(&n.name);
        if self.is_target(&n.name)
        {
            let lines = fields.iter().map(|(f, k)| format!("\n    {f}: {}", k.name()));
            let code = format!("{DATA_KIND} {name}{}", lines.collect::<String>());
            self.keep(code, vec![kind.and_then(layout)], Some(n.span));
        }
        let Some(at) = n.fields.iter().position(|p| self.is_target(&p.name))
        else
        {
            return;
        };
        let Some((field, field_kind)) = fields.get(at).cloned()
        else
        {
            return;
        };
        let offset =
            kind.and_then(DataKind::as_struct).map(|s| s.offset(at, platform()));
        let place = t!("lsp.hover_field", name = name, offset = offset.unwrap_or(0));
        let code = format!("{name} {EI} {field}: {}", field_kind.name());
        self.keep(code, vec![Some(place.to_string()), layout(field_kind)], None);
    }

    /// The card of an enumeration and its members, or of one of its members.
    ///
    /// # Arguments
    ///
    /// * `n` - The enumeration.
    fn enum_card(&mut self, n: &EnumDecl)
    {
        let kind = self.checked.structs.get(&n.name.name).copied();
        let name = shown_name(&n.name);
        if self.is_target(&n.name)
        {
            let lines =
                n.members.iter().map(|m| format!("\n    {}", shown_name(&m.name)));
            let code = format!("{DATA_KIND} {name}{}", lines.collect::<String>());
            self.keep(code, vec![kind.and_then(layout)], Some(n.span));
        }
        let Some(member) = n.members.iter().find(|m| self.is_target(&m.name))
        else
        {
            return;
        };
        let member_name = shown_name(&member.name);
        let value =
            kind.and_then(DataKind::as_enum).and_then(|e| e.member(&member_name));
        let value = value.and_then(value_text);
        let value = value.map(|v| t!("lsp.hover_value", value = v).to_string());
        let place = Some(t!("lsp.hover_member", name = name).to_string());
        let code = format!("{}{AMONG} {member_name}", stem_of(&name));
        self.keep(code, vec![place, value], None);
    }
}

/// The text of a member value: a number, a character, a truth value, an
/// address or a text.
///
/// # Arguments
///
/// * `value` - The value.
///
/// # Returns
///
/// The text, or `None` for a list or struct value.
fn value_text(value: &MemberValue) -> Option<String>
{
    match value
    {
        // The text form of a value ends in its kind, `1:isize`; the hover
        // shows the value alone.
        MemberValue::Scalar(v) =>
        {
            let text = v.to_string();
            Some(text.rsplit_once(':').map_or(text.clone(), |(value, _)| value.into()))
        }
        MemberValue::Text(text) => Some(format!("\"{text}\"")),
        MemberValue::Items(_) => None,
    }
}

impl Visitor for CardMaker<'_>
{
    fn visit_fn(&mut self, n: &FnDecl)
    {
        let head = self.head(n);
        if self.is_target(&n.name) && !n.start
        {
            let lib = n.lib.as_ref().map(|l| l.lit.value.clone());
            let c_fn = lib.map(|lib| t!("lsp.hover_c_fn", lib = lib).to_string());
            self.keep(head.clone(), vec![c_fn], Some(n.span));
        }
        let param = n.params.iter().find(|p| self.is_target(&p.name));
        if let Some(p) = param
        {
            let kind = self.kind_of(&p.kind);
            let code = format!("{}: {}", shown_name(&p.name), kind.name());
            let of = t!("lsp.hover_param", name = shown_name(&n.name)).to_string();
            self.keep(code, vec![Some(of), layout(kind)], None);
        }
        walk_fn(self, n);
    }

    fn visit_decl(&mut self, n: &DeclStmt)
    {
        if self.is_target(&n.name)
        {
            let written = n.kind.as_ref().map(|k| self.kind_of(k));
            let kind =
                written.or_else(|| n.init.as_ref().and_then(|e| self.kind_at(e)));
            let word = tern!(n.is_const, CONST, VAR);
            let kind_text = kind.map(DataKind::name).unwrap_or_else(|| "?".into());
            let code = format!("{word} {}: {kind_text}", shown_name(&n.name));
            self.keep(code, vec![kind.and_then(layout)], Some(n.span));
        }
        walk_decl(self, n);
    }

    fn visit_struct(&mut self, n: &StructDecl)
    {
        self.struct_card(n);
        walk_struct(self, n);
    }

    fn visit_enum(&mut self, n: &EnumDecl)
    {
        self.enum_card(n);
        walk_enum(self, n);
    }

    fn visit_alias(&mut self, n: &AliasDecl)
    {
        if self.is_target(&n.name)
        {
            let kind = self.kind_of(&n.kind);
            let code = format!("{DATA_KIND} {} = {}", shown_name(&n.name), kind.name());
            let alias = Some(t!("lsp.hover_alias").to_string());
            self.keep(code, vec![alias, layout(kind)], Some(n.span));
        }
        walk_alias(self, n);
    }

    fn visit_each(&mut self, n: &EachStmt)
    {
        let source = self.kind_at(&n.source);
        let cell =
            source.and_then(|k| k.cell_kind().or_else(|| k.pointee()?.cell_kind()));
        self.loop_card(&n.var, cell);
        walk_each(self, n);
    }

    fn visit_counted(&mut self, n: &CountedStmt)
    {
        let kind = self.kind_at(&n.from);
        self.loop_card(&n.var, kind);
        walk_counted(self, n);
    }

    fn visit_generic(&mut self, n: &GenericStmt)
    {
        let written = n.kind.as_ref().map(|k| self.kind_of(k));
        let kind = written.or_else(|| n.init.as_ref().and_then(|e| self.kind_at(e)));
        self.loop_card(&n.var, kind);
        walk_generic(self, n);
    }
}

impl CardMaker<'_>
{
    /// The card of a loop variable.
    ///
    /// # Arguments
    ///
    /// * `var` - The variable.
    /// * `kind` - Its kind, when known.
    fn loop_card(&mut self, var: &Ident, kind: Option<DataKind>)
    {
        do_if!(!self.is_target(var), return);
        let kind_text = kind.map(DataKind::name).unwrap_or_else(|| "?".into());
        let code = format!("{}: {kind_text}", shown_name(var));
        let note = Some(t!("lsp.hover_loop").to_string());
        self.keep(code, vec![note, kind.and_then(layout)], None);
    }
}

/// Finds the shortest expression of the open file with a known kind that
/// holds an offset.
struct Finder<'a>
{
    /// The check of the open file.
    file: &'a FileCheck,
    /// The kind of each expression.
    kinds: &'a KindTable,
    /// The offset, in the text the editor holds.
    at: u32,
    /// The best expression so far: its range and node number.
    best: Option<(Range, NodeId)>,
}

impl Visitor for Finder<'_>
{
    fn visit_expr(&mut self, n: &Expr)
    {
        let span = n.span();
        let range = self.file.root_range(&span);
        let inside = range.lo <= self.at && self.at <= range.hi;
        let known = self.kinds.contains_key(&n.id());
        let holds = self.file.in_root(&span) && inside && known;
        let shorter =
            |(best, _): (Range, NodeId)| range.hi - range.lo < best.hi - best.lo;
        let better = holds && self.best.is_none_or(shorter);
        do_if!(better, self.best = Some((range, n.id())));
        walk_expr(self, n);
    }
}
