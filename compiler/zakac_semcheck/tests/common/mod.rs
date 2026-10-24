#![allow(dead_code)]

use zakac_ast::expr::Expr;
use zakac_ast::stmt::Program;
use zakac_ast::visit::Node;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::walk_expr;
use zakac_diagnostics::Code;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex;
use zakac_parser::parser::parse;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_semcheck::semcheck::SemcheckOutput;
use zakac_semcheck::semcheck::semcheck;
use zakac_span::filename::FileName;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;

/// A checked program with its source.
pub struct Checked
{
    /// The source file.
    pub file: std::sync::Arc<SourceFile>,
    /// The tree.
    pub program: Program,
    /// The check output.
    pub out: SemcheckOutput,
}

/// Lex, parse and check `text`; panics on lexer or parser diagnostics.
pub fn check_with(text: &str, options: &SemcheckOptions) -> Checked
{
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("t.zaka".into()), text.to_string());
    let lexed = lex(&file, &LexerOptions::default());
    let parsed = parse(&file, &lexed);
    let mut front = lexed.diagnostics;
    front.extend(parsed.diagnostics);
    let msgs: Vec<&str> = front.iter().map(|d| d.message.as_str()).collect();
    assert!(front.is_empty(), "front-end errors in {text}: {msgs:?}");
    let out = semcheck(&parsed.program, options);
    Checked { file, program: parsed.program, out }
}

/// Check `text` with the default options.
pub fn check(text: &str) -> Checked { check_with(text, &SemcheckOptions::default()) }

/// The check diagnostic codes of `text`.
pub fn codes(text: &str) -> Vec<Code>
{
    check(text).out.diagnostics.iter().map(|d| d.code).collect()
}

/// Assert that `text` checks without diagnostics.
pub fn ok(text: &str)
{
    let out = check(text).out;
    let msgs: Vec<&str> = out.diagnostics.iter().map(|d| d.message.as_str()).collect();
    assert!(out.diagnostics.is_empty(), "{text}: {msgs:?}");
}

/// Collects expressions with their source text, outermost first.
struct Finder<'a>
{
    /// The source file.
    file: &'a SourceFile,
    /// Text and expression, in visit order.
    found: Vec<(String, Expr)>,
}

impl Visitor for Finder<'_>
{
    fn visit_expr(&mut self, n: &Expr)
    {
        let base = self.file.start_pos.as_u32();
        let span = n.span();
        let (lo, hi) = (span.lo.as_u32() - base, span.hi.as_u32() - base);
        let text = self.file.src[lo as usize..hi as usize].to_string();
        self.found.push((text, n.clone()));
        walk_expr(self, n);
    }
}

/// The kind name recorded for the first expression whose source text is
/// `needle`, or `"-"` when none is recorded.
pub fn kind_at(checked: &Checked, needle: &str) -> String
{
    let mut finder = Finder { file: &checked.file, found: Vec::new() };
    finder.visit_program(&checked.program);
    let hit = finder.found.iter().find(|(text, _)| text == needle);
    let (_, expr) = hit.unwrap_or_else(|| panic!("no expression \"{needle}\""));
    checked.out.kinds.get(&expr.id()).map_or("-".to_string(), |k| k.name())
}
