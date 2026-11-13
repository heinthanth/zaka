//! A call end glued to the name of a visible function, such as `ပေါင်းသည့်ရလဒ်`
//! for `ပေါင်း သည့် ရလဒ်`, and the warnings for the names that can clash with
//! a glued call or with the words of a preposition.
//!
//! The split runs against the functions the file sees, not against a list of
//! suffixes, so `အမည်` ("name") remains a name unless the file sees a function
//! `အ`. A name in braces never splits.

use std::collections::HashSet;

use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::parser as diag;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex_at;
use zakac_lexer::token::Kw;
use zakac_lexer::token::LexOutput;
use zakac_lexer::token::Punct;
use zakac_lexer::token::Token;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::ret_if;
use zakac_span::pos::Pos;
use zakac_span::span::Span;

use crate::using::Functions;

/// The call ends that can be glued to a function name, longest first, so
/// the longest end that fits wins.
const ENDS: &[&str] = &[
    "မည့်ရလဒ်ကောင်း",
    "သည့်ရလဒ်ကောင်း",
    "သောရလဒ်ကောင်း",
    "မည့်ရလဒ်",
    "သည့်ရလဒ်",
    "သောရလဒ်",
    "မည်",
    "သည်",
    "ပါ",
];

/// The tokens of a file with each glued call end split off the function
/// name it follows, where the name is not being declared.
///
/// # Arguments
///
/// * `text` - The normalized source text.
/// * `base` - Absolute position of `text`.
/// * `lexed` - The lexer output for `text`.
/// * `functions` - The functions the file sees.
///
/// # Returns
///
/// The lexer output with the split tokens.
pub(crate) fn split_glued(
    text: &str, base: Pos, lexed: &LexOutput, functions: &Functions,
) -> LexOutput
{
    let mut out = lexed.clone();
    let tokens = &lexed.tokens;
    let split = |at: usize| -> Vec<Token> {
        let tok = &tokens[at];
        let parts = (!declared_at(tokens, at))
            .then(|| glued_parts(text, base, tok, functions))
            .flatten();
        parts.map_or_else(|| vec![*tok], |(stem, end)| split_token(tok, stem, end))
    };
    out.tokens = (0..tokens.len()).flat_map(split).collect();
    out
}

/// The function name and the call end a name token is glued from.
///
/// # Arguments
///
/// * `text` - The normalized source text.
/// * `base` - Absolute position of `text`.
/// * `tok` - The token.
/// * `functions` - The functions the file sees.
///
/// # Returns
///
/// The name and the end, or `None` for a token that is no glued call.
fn glued_parts(
    text: &str, base: Pos, tok: &Token, functions: &Functions,
) -> Option<(usize, &'static str)>
{
    ret_if!(tok.kind != TokenKind::Ident, None);
    let word = &text[(tok.span.lo - base) as usize..(tok.span.hi - base) as usize];
    let fits = |end: &&&str| {
        let stem = word.strip_suffix(**end);
        stem.is_some_and(|s| !s.is_empty() && functions.contains_key(s))
    };
    let end = ENDS.iter().find(fits)?;
    Some((word.len() - end.len(), end))
}

/// The name token and the end tokens of a glued call.
///
/// # Arguments
///
/// * `tok` - The glued token.
/// * `stem` - The length of the name, in octets.
/// * `end` - The text of the end.
///
/// # Returns
///
/// The tokens.
fn split_token(tok: &Token, stem: usize, end: &str) -> Vec<Token>
{
    let mut name = *tok;
    name.span = Span::new(tok.span.lo, tok.span.lo + stem);
    let lexed = lex_at(end, tok.span.lo + stem, &LexerOptions::default());
    let ends = lexed.tokens.into_iter().filter(|t| t.kind != TokenKind::Eof);
    let ends = ends.enumerate().map(|(n, mut t)| {
        t.joined = n == 0 || t.joined;
        t
    });
    std::iter::once(name).chain(ends).collect()
}

/// Whether the name token at `at` is being declared: after `ကိန်းရှင်`,
/// `ကိန်းသေ` or `လုပ်ဆောင်ချက်`, or before `ဟူသော`.
///
/// # Arguments
///
/// * `tokens` - The tokens of the file.
/// * `at` - The index of the token.
///
/// # Returns
///
/// `true` for a declared name.
fn declared_at(tokens: &[Token], at: usize) -> bool
{
    let kw = |t: Option<&Token>| match t.map(|t| t.kind)
    {
        Some(TokenKind::Kw(kw)) => Some(kw),
        _ => None,
    };
    let before = kw(at.checked_sub(1).and_then(|b| tokens.get(b)));
    let after = kw(tokens.get(at + 1));
    matches!(before, Some(Kw::Var | Kw::Const | Kw::Function))
        || after == Some(Kw::Called)
}

/// The warnings for each declared name that can clash: a word of a visible
/// preposition (outside a struct, whose fields never stand where a preposition
/// can), or a glued call of a visible function.
///
/// # Arguments
///
/// * `text` - The normalized source text.
/// * `base` - Absolute position of `text`.
/// * `lexed` - The lexer output for `text`.
/// * `functions` - The functions the file sees.
///
/// # Returns
///
/// The warnings.
pub(crate) fn clash_warnings(
    text: &str, base: Pos, lexed: &LexOutput, functions: &Functions,
) -> Vec<Diagnostic>
{
    let prepositions: Vec<(&str, &str)> = functions
        .iter()
        .flat_map(|(f, lists)| {
            lists.iter().flatten().map(move |m| (f.as_str(), m.as_str()))
        })
        .collect();
    let words: HashSet<&str> =
        prepositions.iter().flat_map(|(_, m)| m.split(' ')).collect();
    let tokens = &lexed.tokens;
    let mut found = Vec::new();
    let mut in_struct = false;
    for (at, tok) in tokens.iter().enumerate()
    {
        in_struct |= tok.kind == TokenKind::Kw(Kw::StructWord);
        in_struct &= tok.kind != TokenKind::Punct(Punct::FullStop);
        let named = tok.kind == TokenKind::Ident && declared_at(tokens, at);
        if !named
        {
            continue;
        }
        let word = &text[(tok.span.lo - base) as usize..(tok.span.hi - base) as usize];
        if let Some((stem, end)) = glued_parts(text, base, tok, functions)
        {
            found.push(diag::name_reads_as_call(tok.span, word, &word[..stem], end));
        }
        let preposition =
            prepositions.iter().find(|(_, m)| m.split(' ').any(|w| w == word));
        let clash =
            (!in_struct && words.contains(word)).then_some(preposition).flatten();
        if let Some((func, preposition)) = clash
        {
            found.push(diag::name_is_preposition_word(
                tok.span,
                word,
                preposition,
                func,
            ));
        }
    }
    found
}
