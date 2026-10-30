use std::collections::HashSet;

use zakac_span::pos::Pos;
use zakac_span::span::Span;

use crate::token::Kw;
use crate::token::Token;
use crate::token::TokenKind;

/// The ending that turns a classifier into a count word: `ယောက်` → `ယောက်စာ`.
pub const COUNT_ENDING: &str = "စာ";

/// Mark the count words of the classifiers a program declares.
///
/// A classifier is the name after `မျိုးပြပစ္စည်းမှာ`. The lexer has no word list,
/// so `ယောက်စာ` lexes as a name; each name spelled `<classifier>စာ` becomes
/// [`Kw::Items`] here, with its span kept, so the parser reads the classifier
/// from the text before `စာ`.
///
/// # Arguments
///
/// * `tokens` - The tokens, phrases joined.
/// * `text` - The source text.
/// * `base` - Absolute position of `text`.
pub fn mark_classifiers(tokens: &mut [Token], text: &str, base: Pos)
{
    let text_of =
        |span: Span| &text[(span.lo - base) as usize..(span.hi - base) as usize];
    let declared = |pair: &[Token]| {
        let named = pair[0].kind == TokenKind::Kw(Kw::ClassifierIs)
            && pair[1].kind == TokenKind::Ident;
        named.then(|| format!("{}{COUNT_ENDING}", text_of(pair[1].span)))
    };
    let words: HashSet<String> = tokens.windows(2).filter_map(declared).collect();
    let is_count =
        |tok: &Token| tok.kind == TokenKind::Ident && words.contains(text_of(tok.span));
    tokens
        .iter_mut()
        .filter(|tok| is_count(tok))
        .for_each(|tok| tok.kind = TokenKind::Kw(Kw::Items));
}
