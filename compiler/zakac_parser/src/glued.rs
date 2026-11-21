//! A call end glued to the name of a visible function, such as `ပေါင်းသည့်ရလဒ်`
//! for `ပေါင်း သည့် ရလဒ်`, and the warnings for the names that can clash with
//! a glued call or with the words of a preposition.
//!
//! The split runs against the functions the file sees, not against a list of
//! suffixes, so `အမည်` ("name") remains a name unless the file sees a function
//! `အ`. A name in braces never splits.

use std::collections::HashSet;

use zakac_diagnostics::Code;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::parser as diag;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex_at;
use zakac_lexer::phrases::join_phrases;
use zakac_lexer::reserved::lookup;
use zakac_lexer::token::Kw;
use zakac_lexer::token::LexOutput;
use zakac_lexer::token::Punct;
use zakac_lexer::token::Token;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::ret_if;
use zakac_span::pos::Pos;
use zakac_span::span::Span;

use crate::using::CallWords;
use crate::using::Functions;

/// The result ends that can be glued to a function name, longest first.
const RESULT_ENDS: &[&str] = &[
    "မည့်ရလဒ်ကောင်း",
    "မယ့်ရလဒ်ကောင်း",
    "သည့်ရလဒ်ကောင်း",
    "တဲ့ရလဒ်ကောင်း",
    "သောရလဒ်ကောင်း",
    "မည့်ရလဒ်",
    "မယ့်ရလဒ်",
    "သည့်ရလဒ်",
    "တဲ့ရလဒ်",
    "သောရလဒ်",
];

/// The statement ends that can be glued to a function name.
const STMT_ENDS: &[&str] = &["မယ်", "မည်", "တယ်", "သည်", "ပါ"];

/// The ends that can be glued to a function name when `ရလဒ်` or
/// `ရလဒ်ကောင်း` stands apart after them, as in `စောင့်သော ရလဒ်`.
const ENDS_BEFORE_RESULT: &[&str] = &["မည့်", "မယ့်", "သည့်", "တဲ့", "သော"];

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
        let next = tokens.get(at + 1).map(|t| t.kind);
        let result = matches!(next, Some(TokenKind::Kw(Kw::Result | Kw::GoodResult)));
        let extra_ends = if result { ENDS_BEFORE_RESULT } else { &[] };
        let parts = (!declared_at(tokens, at))
            .then(|| glued_parts(text, base, tok, functions, extra_ends))
            .flatten();
        parts.map_or_else(|| vec![*tok], |parts| split_token(tok, parts))
    };
    let split: Vec<Token> = (0..tokens.len()).flat_map(split).collect();
    // An end split off before `ရလဒ်` joins it as the one word they make.
    out.tokens = join_phrases(split);
    out
}

/// The parts of a glued call: the reserved word glued in front, with its
/// length, as `အကုန်` in `အကုန်ကူးထည့်မည်`; the length of the function name;
/// and the call end, which can include an attached `ဖြစ်ပါစေ`.
type Glued<'s> = (Option<(Kw, usize)>, usize, &'s str);

/// The reserved word, the function name and the call end a name token is
/// glued from; the reserved word in front is optional.
///
/// # Arguments
///
/// * `text` - The normalized source text.
/// * `base` - Absolute position of `text`.
/// * `tok` - The token.
/// * `functions` - The functions the file sees.
/// * `extra_ends` - Additional call ends when a result word follows.
///
/// # Returns
///
/// The parts, or `None` for a token that is no glued call.
fn glued_parts<'s>(
    text: &'s str, base: Pos, tok: &Token, functions: &Functions, extra_ends: &[&str],
) -> Option<Glued<'s>>
{
    ret_if!(tok.kind != TokenKind::Ident, None);
    let word = &text[(tok.span.lo - base) as usize..(tok.span.hi - base) as usize];
    let lead = |stem: &str| -> Option<Option<(Kw, usize)>> {
        ret_if!(functions.contains_key(stem), Some(None));
        let mut cuts = (1..stem.len()).filter(|i| stem.is_char_boundary(*i));
        let cut = cuts.find(|i| {
            lookup(&stem[..*i]).is_some() && functions.contains_key(&stem[*i..])
        })?;
        lookup(&stem[..cut]).map(|kw| Some((kw, cut)))
    };
    let find_end = |call: &str, ends: &[&str]| {
        ends.iter().find_map(|end| {
            let stem = call.strip_suffix(end).filter(|s| !s.is_empty())?;
            Some((lead(stem)?, stem.len()))
        })
    };
    let (front, stem) = find_end(word, RESULT_ENDS)
        .or_else(|| find_end(word, STMT_ENDS))
        .or_else(|| find_end(word, extra_ends))
        .or_else(|| find_end(word.strip_suffix("ဖြစ်ပါစေ")?, RESULT_ENDS))?;
    let lead_len = front.map_or(0, |(_, len)| len);
    Some((front, stem - lead_len, &word[stem..]))
}

/// The tokens of a glued call: the reserved word in front, when there is one,
/// the name and the end.
///
/// # Arguments
///
/// * `tok` - The glued token.
/// * `parts` - The parts of the glued call.
///
/// # Returns
///
/// The tokens.
fn split_token(tok: &Token, parts: Glued<'_>) -> Vec<Token>
{
    let (front, stem, end) = parts;
    let lead = front.map_or(0, |(_, len)| len);
    let lead_tok = front.map(|(kw, len)| {
        let span = Span::new(tok.span.lo, tok.span.lo + len);
        Token { kind: TokenKind::Kw(kw), span, ..*tok }
    });
    let mut name = *tok;
    name.span = Span::new(tok.span.lo + lead, tok.span.lo + lead + stem);
    name.joined = name.joined || lead > 0;
    let lexed = lex_at(end, tok.span.lo + lead + stem, &LexerOptions::default());
    let ends = lexed.tokens.into_iter().filter(|t| t.kind != TokenKind::Eof);
    let ends = ends.enumerate().map(|(n, mut t)| {
        t.joined = n == 0 || t.joined;
        t
    });
    lead_tok.into_iter().chain(std::iter::once(name)).chain(ends).collect()
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
    matches!(before, Some(Kw::Var | Kw::Const)) || after == Some(Kw::Called)
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
            lists
                .iter()
                .flat_map(CallWords::texts)
                .map(move |m| (f.as_str(), m.as_str()))
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
        if let Some((front, stem, end)) = glued_parts(text, base, tok, functions, &[])
        {
            let lead = front.map_or(0, |(_, len)| len);
            let name = &word[lead..lead + stem];
            found.push(diag::name_reads_as_call(tok.span, word, name, end));
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

/// The ZK0510 of each spoken call end glued to the name of a visible
/// function, such as the `မယ်` of `ပြသမယ်`, which the lexer read as a part of
/// the name.
///
/// shortcut: the ZK0511 count of the lexer leaves these ends out; count them
/// when files that glue spoken ends report a wrong mix.
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
/// The diagnostics.
pub(crate) fn glued_spoken(
    text: &str, base: Pos, lexed: &LexOutput, functions: &Functions,
) -> Vec<Diagnostic>
{
    let tokens = &lexed.tokens;
    let end_diags = |at: usize| {
        let tok = &tokens[at];
        let (front, stem, end) = glued_parts(text, base, tok, functions, &[])?;
        let at = tok.span.lo + front.map_or(0, |(_, len)| len) + stem;
        Some(lex_at(end, at, &LexerOptions::default()).diagnostics)
    };
    let open = (0..tokens.len()).filter(|&at| !declared_at(tokens, at));
    let found = open.filter_map(end_diags).flatten();
    found.filter(|d| d.code == Code::SpokenForm).collect()
}
