#![allow(dead_code)]

use std::sync::Arc;

use zakac_diagnostics::Code;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex;
use zakac_lexer::token::Kw;
use zakac_lexer::token::LexOutput;
use zakac_lexer::token::TokenKind;
use zakac_span::filename::FileName;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;

/// Normalize `text` through a source map and lex it.
pub fn lex_text(text: &str) -> (LexOutput, Arc<SourceFile>)
{
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("t.zaka".into()), text.to_string());
    (lex(&file, &LexerOptions::default()), file)
}

/// Token kinds of `text`, without the end-of-input token.
pub fn kinds(text: &str) -> Vec<TokenKind>
{
    let (out, _) = lex_text(text);
    let mut kinds: Vec<TokenKind> = out.tokens.iter().map(|t| t.kind).collect();
    assert_eq!(kinds.pop(), Some(TokenKind::Eof));
    kinds
}

/// Reserved-word kinds of `text`; panics on other tokens.
pub fn kws(text: &str) -> Vec<Kw>
{
    kinds(text)
        .into_iter()
        .map(|k| match k
        {
            TokenKind::Kw(kw) => kw,
            other => panic!("not a reserved word: {other:?} in {text}"),
        })
        .collect()
}

/// Diagnostic codes of `text`.
pub fn codes(text: &str) -> Vec<Code>
{
    let (out, _) = lex_text(text);
    out.diagnostics.into_iter().map(|d| d.code).collect()
}

/// Source text of each token of `text`, without the end-of-input token.
pub fn lexemes(text: &str) -> Vec<String>
{
    let (out, file) = lex_text(text);
    let base = file.start_pos;
    let mut lexemes: Vec<String> = out
        .tokens
        .iter()
        .map(|t| {
            file.src[(t.span.lo - base) as usize..(t.span.hi - base) as usize]
                .to_string()
        })
        .collect();
    lexemes.pop();
    lexemes
}
