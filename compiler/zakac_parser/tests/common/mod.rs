#![allow(dead_code)]

use zakac_ast::expr::Expr;
use zakac_ast::s_expr::ToSexpr;
use zakac_ast::stmt::Stmt;
use zakac_diagnostics::Code;
use zakac_diagnostics::Diagnostic;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex;
use zakac_parser::parser::ParseOutput;
use zakac_parser::parser::parse;
use zakac_parser::parser::parse_lone_expr;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

/// The tree of an expression that must parse.
pub fn expr(text: &str) -> String
{
    let parsed = parse_expr(text).unwrap_or_else(|d| panic!("{text}: {}", d.message));
    parsed.to_sexpr()
}

/// The diagnostic code of an expression that must fail.
pub fn expr_err(text: &str) -> Code { parse_expr(text).expect_err(text).code }

/// The statements of a program that must parse without diagnostics.
pub fn stmts(text: &str) -> Vec<Stmt>
{
    let out = parse_text(text);
    let msgs: Vec<&str> = out.diagnostics.iter().map(|d| d.message.as_str()).collect();
    assert!(out.diagnostics.is_empty(), "{text}: {msgs:?}");
    out.program.stmts
}

/// The one statement of a program, as a tree.
pub fn stmt(text: &str) -> String
{
    let list = stmts(text);
    assert_eq!(list.len(), 1, "{text}");
    list[0].to_sexpr()
}

/// The parser diagnostic codes of a program.
pub fn codes(text: &str) -> Vec<Code>
{
    parse_text(text).diagnostics.iter().map(|d| d.code).collect()
}

/// Lex and parse `text`; panics on lexer errors.
pub fn parse_text(text: &str) -> ParseOutput
{
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("t.zaka".into()), text.to_string());
    let lexed = lex(&file, &LexerOptions::default());
    let msg = format!("lexer errors in {text}: {:?}", lexed.diagnostics);
    assert!(lexed.diagnostics.is_empty(), "{msg}");
    parse(&file, &lexed)
}

/// Parse `text` as one expression.
pub fn parse_expr(text: &str) -> Result<Expr, Diagnostic>
{
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("t.zaka".into()), text.to_string());
    let lexed = lex(&file, &LexerOptions::default());
    let msg = format!("lexer errors in {text}: {:?}", lexed.diagnostics);
    assert!(lexed.diagnostics.is_empty(), "{msg}");
    parse_lone_expr(&file.src, file.start_pos, &lexed)
}
