use std::path::PathBuf;

use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex;
use zakac_lexer::token::Kw;
use zakac_lexer::token::TokenKind;
use zakac_parser::parser::parse;
use zakac_parser::parser::parse_lone_expr;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

/// Fenced blocks without a language tag.
fn untagged_blocks(text: &str) -> Vec<String>
{
    let mut blocks = Vec::new();
    let mut open: Option<String> = None;
    for line in text.lines()
    {
        let fence = line.trim_start().starts_with("```");
        match (&mut open, fence)
        {
            (None, true) if line.trim() == "```" => open = Some(String::new()),
            (None, true) => open = Some("\u{0}".into()),
            (Some(block), true) =>
            {
                if !block.starts_with('\u{0}')
                {
                    blocks.push(std::mem::take(block));
                }
                open = None;
            }
            (Some(block), false) => block.push_str(&format!("{line}\n")),
            (None, false) =>
            {}
        }
    }
    blocks
}

/// Whether a block is grammar notation, a template or a trace, not source
/// code.
fn is_template(block: &str) -> bool
{
    let trace = block.contains('→');
    let placeholder = block.contains('<')
        && block.split('<').skip(1).any(|rest| {
            rest.chars().next().is_some_and(|c| c.is_ascii_lowercase())
                && rest.contains('>')
        });
    block.contains(":=")
        || block.contains("...")
        || block.contains("(ဖြစ်)?")
        || placeholder
        || trace
}

/// Problems found in one block. A block with full stops is a program; if it
/// fails as a whole, its lines without a full stop are tried as expressions
/// and the rest as a program, for blocks that show a value and a statement.
fn check_block(sm: &mut SourceMap, block: &str) -> Vec<String>
{
    if !block.contains('။')
    {
        return block.lines().flat_map(|line| check_line(sm, line)).collect();
    }
    let whole = check_program(sm, block);
    if whole.is_empty()
    {
        return whole;
    }
    let (stmts, exprs): (Vec<&str>, Vec<&str>) =
        block.lines().partition(|line| code_of(line).ends_with('။'));
    let mut split: Vec<String> =
        exprs.iter().flat_map(|line| check_line(sm, line)).collect();
    split.extend(check_program(sm, &stmts.join("\n")));
    if split.is_empty() { split } else { whole }
}

/// The code part of a line, without a trailing `//` comment.
fn code_of(line: &str) -> &str { line.split("//").next().unwrap_or("").trim() }

/// Problems found when `text` is parsed as a program.
fn check_program(sm: &mut SourceMap, text: &str) -> Vec<String>
{
    let file = sm.new_source_file(FileName::Custom("spec".into()), text.to_string());
    let lexed = lex(&file, &LexerOptions::default());
    if let Some(first) = lexed.diagnostics.first()
    {
        return vec![format!("lexer: {}", first.message)];
    }
    parse(&file, &lexed).diagnostics.iter().map(|d| d.message.clone()).collect()
}

/// Problems in an expression line. Items set apart with two or more spaces
/// are separate examples; comments, header fragments such as
/// `အကယ်၍ ... လျှင်` are left out.
fn check_line(sm: &mut SourceMap, line: &str) -> Vec<String>
{
    let code = code_of(line);
    code.split("  ")
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .filter_map(|item| check_item(sm, item))
        .collect()
}

/// A problem in one expression, or `None` when it parses or is no
/// expression.
fn check_item(sm: &mut SourceMap, item: &str) -> Option<String>
{
    let file = sm.new_source_file(FileName::Custom("item".into()), item.to_string());
    let lexed = lex(&file, &LexerOptions::default());
    let kinds: Vec<TokenKind> = lexed.tokens.iter().map(|t| t.kind).collect();
    let header_words = [Kw::Then, Kw::ThenIs, Kw::While, Kw::WhileIs];
    let is_header = kinds.first() == Some(&TokenKind::Kw(Kw::If))
        || kinds
            .iter()
            .rev()
            .nth(1)
            .is_some_and(|k| header_words.iter().any(|w| *k == TokenKind::Kw(*w)));
    if kinds.len() <= 1 || is_header
    {
        return None;
    }
    if let Some(first) = lexed.diagnostics.first()
    {
        return Some(format!("{item}: lexer: {}", first.message));
    }
    let parsed = parse_lone_expr(&file.src, file.start_pos, &lexed);
    parsed.err().map(|d| format!("{item}: {}", d.message))
}

#[test]
fn spec_examples_parse()
{
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/LANGUAGE.internal.md");
    let Ok(text) = std::fs::read_to_string(&path)
    else
    {
        eprintln!("spec not found at {}, skipped", path.display());
        return;
    };
    let mut sm = SourceMap::new();
    let mut failures = Vec::new();
    let blocks: Vec<String> =
        untagged_blocks(&text).into_iter().filter(|b| !is_template(b)).collect();
    for block in &blocks
    {
        let problems = check_block(&mut sm, block);
        if !problems.is_empty()
        {
            failures.push(format!("--- block:\n{block}problems: {problems:#?}"));
        }
    }
    assert!(blocks.len() > 20, "found {} example blocks", blocks.len());
    assert!(
        failures.is_empty(),
        "{} of {} blocks fail:\n{}",
        failures.len(),
        blocks.len(),
        failures.join("\n")
    );
}
