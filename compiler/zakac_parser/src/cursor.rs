use bon::Builder;
use zakac_lexer::token::Token;
use zakac_lexer::token::TokenKind;

/// Position in a token list that ends with the end-of-input token. The
/// cursor never moves past that last token.
#[derive(Builder)]
pub(crate) struct Cursor<'t>
{
    /// The tokens; never blank.
    tokens: &'t [Token],
    /// Index of the current token.
    #[builder(default)]
    pos: usize,
}

impl Cursor<'_>
{
    /// The token `n` places ahead of the current one, or the end-of-input
    /// token past the end.
    ///
    /// # Arguments
    ///
    /// * `n` - How far to look ahead.
    ///
    /// # Returns
    ///
    /// The token.
    pub(crate) fn peek_n(&self, n: usize) -> Token
    {
        let last = self.tokens.len() - 1;
        self.tokens[(self.pos + n).min(last)]
    }

    /// The token `n` places behind the current one.
    ///
    /// # Arguments
    ///
    /// * `n` - How far to look back; 1 is the previous token.
    ///
    /// # Returns
    ///
    /// The token, or `None` before the start.
    pub(crate) fn back(&self, n: usize) -> Option<Token>
    {
        self.pos.checked_sub(n).map(|idx| self.tokens[idx])
    }

    /// Move to the next token, unless the current one ends the input.
    ///
    /// # Returns
    ///
    /// The token moved past.
    pub(crate) fn bump(&mut self) -> Token
    {
        let tok = self.peek_n(0);
        do_bump(&mut self.pos, tok);
        tok
    }

    /// Index of the current token.
    ///
    /// # Returns
    ///
    /// The index.
    pub(crate) fn pos(&self) -> usize { self.pos }
}

/// Advance `pos` past `tok` unless it is the end-of-input token.
///
/// # Arguments
///
/// * `pos` - The position to advance.
/// * `tok` - The current token.
fn do_bump(pos: &mut usize, tok: Token)
{
    *pos += usize::from(tok.kind != TokenKind::Eof);
}
