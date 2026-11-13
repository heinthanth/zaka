use zakac_ast::stmt::Stmt;
use zakac_burmese_segmenter::segmenter::SegmenterOptions;
use zakac_burmese_segmenter::segmenter::segment_at;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::parser as diag;
use zakac_lexer::reserved::lookup;
use zakac_lexer::role::builtin_count;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::ret_false;
use zakac_rust_sugars::ret_if;
use zakac_span::pos::Pos;

use crate::clause::as_verb;
use crate::notation::infix_of;
use crate::parser::Parser;

/// Split a name whose last vannas spell a particle, such as `အသက်ကို`, or
/// `ထဲမှ`, which a member read takes after the enumeration.
///
/// # Arguments
///
/// * `text` - The identifier text.
///
/// # Returns
///
/// The name part and the particle, with the longest particle first, or
/// `None` when the name does not end in a particle.
pub(crate) fn split_particle(text: &str) -> Option<(&str, &str)>
{
    let pieces = segment_at(text, Pos::ZERO, &SegmenterOptions::default());
    pieces.iter().skip(1).find_map(|piece| {
        let at = piece.span.lo.as_u32() as usize;
        let tail = &text[at..];
        let split = |kw: &Kw| kw.is_particle() || *kw == Kw::Among;
        lookup(tail).filter(split).map(|_| (&text[..at], tail))
    })
}

impl Parser<'_>
{
    /// Parse a statement; on an error, record it and skip the statement.
    ///
    /// # Returns
    ///
    /// The statement, or `None` after an error.
    pub(crate) fn parse_stmt_recover(&mut self) -> Option<Stmt>
    {
        let start = self.cur.pos();
        let is_if = self.at(Kw::If);
        let err = match self.parse_stmt()
        {
            Ok(stmt) => return Some(stmt),
            Err(err) => err,
        };
        let err = self.hint_group(self.hint_split(err));
        self.diags.push(err);
        self.skip_stmt(start, is_if);
        None
    }

    /// Add a hint when one of the two tokens before the error is a name that
    /// ends in a particle: the name and the particle must be two words.
    ///
    /// # Arguments
    ///
    /// * `err` - The diagnostic.
    ///
    /// # Returns
    ///
    /// The diagnostic, with the hint when it applies.
    fn hint_split(&self, err: Diagnostic) -> Diagnostic
    {
        ret_if!(err.help.is_some(), err);
        let names = (1..=2).filter_map(|n| self.cur.back(n));
        let mut names = names.filter(|tok| tok.kind == TokenKind::Ident);
        let split = names.find_map(|tok| split_particle(self.text_of(tok.span)));
        let Some((stem, particle)) = split
        else
        {
            return err;
        };
        diag::with_split_hint(err, stem, particle)
    }

    /// Add a hint when an operator follows a verb that reads a value, as
    /// in `xs ၏ အကွက်အရေအတွက် == ၀`: the clause before the operator takes
    /// parentheses, `(xs ၏ အကွက်အရေအတွက်) == ၀`.
    ///
    /// # Arguments
    ///
    /// * `err` - The diagnostic.
    ///
    /// # Returns
    ///
    /// The diagnostic, with the hint when it applies.
    fn hint_group(&self, err: Diagnostic) -> Diagnostic
    {
        ret_if!(err.help.is_some(), err);
        let op = self.peek();
        let before = |n: usize| self.cur.back(n);
        let count = before(1)
            .is_some_and(|t| builtin_count(self.text_of(t.span)).is_some())
            && before(2).is_some_and(|t| t.kind == TokenKind::Punct(Punct::Ei));
        let after_verb = before(1).is_some_and(|t| as_verb(&t).is_some()) || count;
        ret_if!(infix_of(&op).is_none() || !after_verb, err);
        diag::with_group_hint(err, self.text_of(op.span))
    }

    /// Skip to just past the full stop that ends the statement, or to a
    /// block closer or the end of the input, which are kept. Blocks inside the
    /// statement are skipped whole, also one whose opener came before the
    /// error, and so are the else arms of a conditional.
    ///
    /// # Arguments
    ///
    /// * `start` - Index of the first token of the statement.
    /// * `is_if` - Whether the statement is a conditional.
    fn skip_stmt(&mut self, start: usize, is_if: bool)
    {
        let mut depth = self.open_blocks_since(start);
        let stuck = self.cur.pos() == start && !self.at_eof();
        ret_if!(stuck && self.skip_token(&mut depth, is_if));
        while !self.at_eof() && !(depth == 0 && self.at_close())
        {
            ret_if!(self.skip_token(&mut depth, is_if));
        }
    }

    /// Skip the current token and track block depth.
    ///
    /// # Arguments
    ///
    /// * `depth` - Open blocks inside the skipped statement.
    /// * `is_if` - Whether else arms after a full stop belong to the statement.
    ///
    /// # Returns
    ///
    /// `true` when the token ends the statement: a full stop outside the
    /// skipped blocks, with no else arm after it when `is_if` is set.
    fn skip_token(&mut self, depth: &mut u32, is_if: bool) -> bool
    {
        let kind = self.bump().kind;
        *depth = track_depth(*depth, kind);
        ret_false!(*depth > 0 || kind != TokenKind::Punct(Punct::FullStop));
        !is_if || !(self.at(Kw::ElseIf) || self.at(Kw::Else))
    }

    /// Blocks still open among the tokens of the statement that were moved
    /// past before the error.
    ///
    /// # Arguments
    ///
    /// * `start` - Index of the first token of the statement.
    ///
    /// # Returns
    ///
    /// The open block count.
    fn open_blocks_since(&self, start: usize) -> u32
    {
        let seen = (1..=self.cur.pos() - start).rev().filter_map(|n| self.cur.back(n));
        seen.fold(0, |depth, tok| track_depth(depth, tok.kind))
    }
}

/// The block depth after a token.
///
/// # Arguments
///
/// * `depth` - The depth before the token.
/// * `kind` - The token kind.
///
/// # Returns
///
/// One more after an opener, one less after a closer, else the same.
fn track_depth(depth: u32, kind: TokenKind) -> u32
{
    match kind
    {
        TokenKind::Kw(Kw::BlockOpen | Kw::UnsafeBlockOpen | Kw::FnOpenRet) => depth + 1,
        TokenKind::Kw(Kw::BlockClose | Kw::ProgramClose) => depth.saturating_sub(1),
        _ => depth,
    }
}
