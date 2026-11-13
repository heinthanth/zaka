use zakac_diagnostics::parser as diag;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::Token;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::describe::kw_text;
use crate::parser::PResult;
use crate::parser::Parser;

/// Whether a scan for the end of a list in words stops at a token: a
/// particle, `၏` or the full stop, which no item holds. `နှင့်` joins the
/// last item and `အနေဖြင့်` stands inside a row or a field value, so neither
/// stops it.
///
/// # Arguments
///
/// * `kind` - The token kind.
///
/// # Returns
///
/// `true` where the scan stops.
pub(crate) fn stops_item_scan(kind: TokenKind) -> bool
{
    let TokenKind::Kw(kw) = kind
    else
    {
        return matches!(
            kind,
            TokenKind::Punct(Punct::FullStop | Punct::Ei) | TokenKind::Eof
        );
    };
    let inside = matches!(kw, Kw::Hnint | Kw::CountAs);
    !inside && kw.is_particle()
}

impl Parser<'_>
{
    /// Parse a list in words: one item, or items with `၊` between them and
    /// `နှင့်` before the last one, as `က`, `က နှင့် ခ` and `က၊ ခ နှင့် ဂ`.
    ///
    /// # Arguments
    ///
    /// * `item` - Parses one item.
    ///
    /// # Returns
    ///
    /// The items, or a diagnostic.
    pub(crate) fn parse_word_list<T>(
        &mut self, mut item: impl FnMut(&mut Self) -> PResult<T>,
    ) -> PResult<Vec<T>>
    {
        let outer = std::mem::replace(&mut self.in_word_list, true);
        let first = item(self);
        let items = first.and_then(|first| self.word_list_rest(first, &mut item));
        self.in_word_list = outer;
        items
    }

    /// Parse the rest of a list in words after its first item, parsed before.
    ///
    /// # Arguments
    ///
    /// * `first` - The first item.
    /// * `item` - Parses each later item.
    ///
    /// # Returns
    ///
    /// The items, or a diagnostic.
    pub(crate) fn parse_word_list_from<T>(
        &mut self, first: T, mut item: impl FnMut(&mut Self) -> PResult<T>,
    ) -> PResult<Vec<T>>
    {
        let outer = std::mem::replace(&mut self.in_word_list, true);
        let items = self.word_list_rest(first, &mut item);
        self.in_word_list = outer;
        items
    }

    /// The items after the first: each after a `၊`, and the last after
    /// `နှင့်`, which a list of two or more needs and no `၊` follows.
    ///
    /// # Arguments
    ///
    /// * `first` - The first item.
    /// * `item` - Parses each later item.
    ///
    /// # Returns
    ///
    /// The items, or a diagnostic.
    fn word_list_rest<T>(
        &mut self, first: T, item: &mut impl FnMut(&mut Self) -> PResult<T>,
    ) -> PResult<Vec<T>>
    {
        let mut items = vec![first];
        while self.eat_punct(Punct::ListSep)
        {
            items.push(item(self)?);
        }
        let joined = self.at(Kw::Hnint);
        ret_if!(
            !joined && items.len() > 1,
            Err(diag::list_needs_and(self.peek().span))
        );
        ret_if!(!joined, Ok(items));
        self.advance();
        items.push(item(self)?);
        ret_if!(
            self.at_punct(Punct::ListSep),
            Err(diag::list_needs_and(self.peek().span))
        );
        Ok(items)
    }

    /// Move past the word that closes a list: `one` after one item, `many`
    /// after two or more.
    ///
    /// # Arguments
    ///
    /// * `count` - The number of items.
    /// * `one` - The word after one item, such as `ကို`.
    /// * `many` - The word after two or more, such as `တို့ကို`.
    ///
    /// # Returns
    ///
    /// The word, or a diagnostic: `ZK0246` for the word of the other count.
    pub(crate) fn expect_list_word(
        &mut self, count: usize, one: Kw, many: Kw,
    ) -> PResult<Token>
    {
        let (want, other) = tern!(count > 1, (many, one), (one, many));
        ret_if!(self.at(want), Ok(self.bump()));
        ret_if!(!self.at(other), self.expect(want));
        let tok = self.peek();
        let found = self.text_of(tok.span).to_string();
        Err(diag::list_word_count(tok.span, count, &kw_text(want), &found))
    }

    /// Whether `close` holds for a token from `from` on, outside parentheses
    /// and brackets, before a token that no item of a list in words holds.
    ///
    /// # Arguments
    ///
    /// * `from` - How far ahead the scan starts.
    /// * `close` - The test, given the parser and how far ahead.
    ///
    /// # Returns
    ///
    /// `true` if it holds.
    pub(crate) fn word_list_ahead(
        &self, from: usize, close: impl Fn(&Self, usize) -> bool,
    ) -> bool
    {
        self.word_list_end(from, close).is_some()
    }

    /// How far ahead `close` first holds, outside parentheses and brackets,
    /// before a token that no item of a list in words holds.
    ///
    /// # Arguments
    ///
    /// * `from` - How far ahead the scan starts.
    /// * `close` - The test, given the parser and how far ahead.
    ///
    /// # Returns
    ///
    /// The distance, or `None` when the scan stops first.
    pub(crate) fn word_list_end(
        &self, from: usize, close: impl Fn(&Self, usize) -> bool,
    ) -> Option<usize>
    {
        let mut depth = 0usize;
        for n in from..
        {
            let kind = self.peek_n(n).kind;
            ret_if!(kind == TokenKind::Eof, None);
            ret_if!(depth == 0 && close(self, n), Some(n));
            ret_if!(depth == 0 && stops_item_scan(kind), None);
            match kind
            {
                TokenKind::Punct(Punct::LParen | Punct::LBracket) => depth += 1,
                TokenKind::Punct(Punct::RParen | Punct::RBracket) if depth == 0 =>
                {
                    return None;
                }
                TokenKind::Punct(Punct::RParen | Punct::RBracket) => depth -= 1,
                _ => (),
            }
        }
        None
    }

    /// Run `f` outside a list in words: inside parentheses a `နှင့်`
    /// belongs to the clause again.
    ///
    /// # Arguments
    ///
    /// * `f` - The parse to run.
    ///
    /// # Returns
    ///
    /// What `f` gives.
    pub(crate) fn outside_word_list<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T
    {
        self.with_word_list(false, f)
    }

    /// Run `f` as the first item of a list in words: a `နှင့်` outside
    /// parentheses ends it.
    ///
    /// # Arguments
    ///
    /// * `f` - The parse to run.
    ///
    /// # Returns
    ///
    /// What `f` gives.
    pub(crate) fn inside_word_list<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T
    {
        self.with_word_list(true, f)
    }

    /// Run `f` with the flag of a list item set to `on`, then restore it.
    ///
    /// # Arguments
    ///
    /// * `on` - Whether an item of a list in words is being parsed.
    /// * `f` - The parse to run.
    ///
    /// # Returns
    ///
    /// What `f` gives.
    fn with_word_list<T>(&mut self, on: bool, f: impl FnOnce(&mut Self) -> T) -> T
    {
        let outer = std::mem::replace(&mut self.in_word_list, on);
        let out = f(self);
        self.in_word_list = outer;
        out
    }

    /// Whether a later item of a list in words comes next: `၊` or `နှင့်`.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn at_list_join(&self) -> bool
    {
        self.at_punct(Punct::ListSep) || self.at(Kw::Hnint)
    }
}
