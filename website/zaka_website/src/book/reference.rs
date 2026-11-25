use std::collections::HashMap;
use std::sync::LazyLock;

use zakac_ast::stmt::FnDecl;
use zakac_ast::stmt::Stmt;
use zakac_ast::visit::Node;
use zakac_i18n::t;
use zakac_lexer::lexer::LexerOptions;
use zakac_parser::load::CORE_PARTS;
use zakac_parser::load::STD_MODULES;
use zakac_parser::load::parse_in_place;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;
use zakac_span::span::Span;

use super::render::FILE_FENCE;

/// The import name of the core module, whose parts sit in `CORE_PARTS`.
const CORE: &str = "စံကား/အခြေခံအုတ်မြစ်";

/// The reference of each stdlib module, as Markdown, on its import name.
static REFERENCES: LazyLock<HashMap<&'static str, String>> = LazyLock::new(|| {
    STD_MODULES
        .iter()
        .map(|(name, _, source)| {
            let core = || CORE_PARTS.iter().map(|(_, s)| *s).collect();
            let sources = tern!(*name == CORE, core(), vec![*source]);
            (*name, module_reference(name, &sources))
        })
        .collect()
});

/// The reference of a stdlib module: each declaration that it exports, with
/// its function signature or its whole form and its `///` lines, under one
/// `##` head for each group.
///
/// # Arguments
///
/// * `module` - The import name, such as `စံကား/စာသား`.
///
/// # Returns
///
/// The Markdown, or `None` for a name the stdlib does not have.
pub fn reference(module: &str) -> Option<&'static str>
{
    REFERENCES.get(module).map(String::as_str)
}

/// The group of an exported declaration, in the order the page shows them.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Group
{
    /// Structs, enumerations, aliases and built-in kinds.
    Kinds,
    /// Traits.
    Traits,
    /// Functions.
    Functions,
    /// Constants and variables.
    Constants,
}

impl Group
{
    /// The `##` head of the group.
    ///
    /// # Returns
    ///
    /// The text.
    fn head(self) -> String
    {
        match self
        {
            Group::Kinds => t!("website.stdlib.kinds"),
            Group::Traits => t!("website.stdlib.traits"),
            Group::Functions => t!("website.stdlib.functions"),
            Group::Constants => t!("website.stdlib.constants"),
        }
        .to_string()
    }
}

/// One exported declaration.
struct Item
{
    /// Its group.
    group: Group,
    /// Its name.
    name: String,
    /// The declaration as written: a function without its block.
    code: String,
    /// The `///` lines above it, without their marks.
    doc: String,
}

/// The reference of one module.
///
/// # Arguments
///
/// * `name` - The import name.
/// * `sources` - The source of each of its files.
///
/// # Returns
///
/// The Markdown.
fn module_reference(name: &str, sources: &[&'static str]) -> String
{
    let mut items: Vec<Item> =
        sources.iter().flat_map(|s| file_entries(name, s)).collect();
    items.sort_by_key(|e| e.group);
    let mut out = String::new();
    let mut last = None;
    for e in items
    {
        let head =
            (last != Some(e.group)).then(|| format!("\n## {}\n", e.group.head()));
        out.extend(head);
        last = Some(e.group);
        out.push_str(&item_markdown(&e));
    }
    out
}

/// The exported declarations of one file, in source order.
///
/// # Arguments
///
/// * `name` - The import name of its module.
/// * `source` - Its source.
///
/// # Returns
///
/// The items.
fn file_entries(name: &str, source: &'static str) -> Vec<Item>
{
    let mut sm = SourceMap::new();
    let file =
        sm.new_source_file(FileName::Custom(name.to_string()), source.to_string());
    let Ok(parsed) = parse_in_place(&file, &mut sm, &LexerOptions::default())
    else
    {
        return Vec::new();
    };
    let text = |span: Span| sm.span_to_snippet(&span).unwrap_or_default();
    let doc = |span: Span| {
        doc_above(&file.src, (span.lo.as_u32() - file.start_pos.as_u32()) as usize)
    };
    parsed
        .stmts
        .iter()
        .filter_map(|s| {
            let (group, ident, span) = exported(s)?;
            let code = text(span).trim().to_string();
            let doc = doc(s.span());
            Some(Item { group, name: ident, code, doc })
        })
        .collect()
}

/// The group, the name and the shown span of a statement that declares an
/// exported name.
///
/// # Arguments
///
/// * `s` - The statement.
///
/// # Returns
///
/// The three, or `None` for a statement that exports nothing.
fn exported(s: &Stmt) -> Option<(Group, String, Span)>
{
    let whole = |exported: bool, group, name: &str| {
        exported.then(|| (group, name.to_string(), s.span()))
    };
    match s
    {
        Stmt::Fn(f) if f.exported && !f.test && !f.start =>
        {
            Some((Group::Functions, f.name.name.to_string(), head_span(f)))
        }
        Stmt::Struct(n) => whole(n.exported, Group::Kinds, &n.name.name),
        Stmt::Enum(n) => whole(n.exported, Group::Kinds, &n.name.name),
        Stmt::Alias(n) => whole(n.exported, Group::Kinds, &n.name.name),
        Stmt::BuiltinKind(n) => whole(n.exported, Group::Kinds, &n.name.name),
        Stmt::Trait(n) => whole(n.exported, Group::Traits, &n.name.name),
        Stmt::Decl(n) => whole(n.exported, Group::Constants, &n.name.name),
        _ => None,
    }
}

/// The span of a function signature: the declaration up to its first
/// statement, or all of it when its block has no statements.
///
/// # Arguments
///
/// * `f` - The function.
///
/// # Returns
///
/// The span.
fn head_span(f: &FnDecl) -> Span
{
    let Some(first) = f.block.stmts.first()
    else
    {
        return f.span;
    };
    Span::new(f.span.lo, first.span().lo)
}

/// The `///` lines right above an offset of a source, without their marks.
///
/// # Arguments
///
/// * `source` - The source.
/// * `at` - The offset of the start of a declaration.
///
/// # Returns
///
/// The lines, joined; blank when none stands right above.
fn doc_above(source: &str, at: usize) -> String
{
    let before = source[..at].trim_end_matches([' ', '\t']);
    let lines = before.strip_suffix('\n').unwrap_or(before).lines().rev();
    let mut doc: Vec<&str> = lines
        .map_while(|l| l.trim_start().strip_prefix("///"))
        .map(|l| l.strip_prefix(' ').unwrap_or(l))
        .collect();
    doc.reverse();
    doc.join("\n")
}

/// One item as Markdown: a `###` head, the declaration painted, then its
/// doc, with heads such as `# Arguments` as bold lines and its examples
/// painted, never run.
///
/// # Arguments
///
/// * `e` - The item.
///
/// # Returns
///
/// The Markdown.
fn item_markdown(e: &Item) -> String
{
    let doc: Vec<String> = e.doc.lines().map(doc_line).collect();
    let doc = doc.join("\n");
    format!("\n### {}\n\n```{FILE_FENCE}\n{}\n```\n\n{doc}\n", e.name, e.code)
}

/// One line of a doc, as the page shows it.
///
/// # Arguments
///
/// * `line` - The line.
///
/// # Returns
///
/// The line, a `# Head` as bold text and a `zaka` fence as a painted one.
fn doc_line(line: &str) -> String
{
    ret_if!(let Some(head) = line.strip_prefix("# "), format!("**{head}**"));
    ret_if!(
        line.trim() == "```zaka",
        line.replace("```zaka", &format!("```{FILE_FENCE}"))
    );
    line.to_string()
}
