use zakac_ast::stmt::Stmt;
use zakac_burmese_segmenter::segmenter::SegmenterOptions;
use zakac_burmese_segmenter::segmenter::segment_at;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::parser as diag;
use zakac_lexer::reserved::lookup;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::ret_if;
use zakac_span::pos::Pos;

use crate::parser::Parser;

/// Split a name whose last vannas spell a particle, such as `အသက်ကို`.
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
        lookup(tail).filter(|kw| kw.is_particle()).map(|_| (&text[..at], tail))
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
        let err = self.hint_split(err);
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

    /// Skip to just past the full stop that ends the statement, or to a
    /// block closer or the end of the input, which are kept. Blocks inside the
    /// statement are skipped whole, and so are the else arms of a
    /// conditional.
    ///
    /// # Arguments
    ///
    /// * `start` - Index of the first token of the statement.
    /// * `is_if` - Whether the statement is a conditional.
    fn skip_stmt(&mut self, start: usize, is_if: bool)
    {
        let mut depth = 0u32;
        let stuck = self.cur.pos() == start && !self.at_eof();
        ret_if!(stuck && self.skip_token(&mut depth, is_if));
        while !self.at_eof() && !(depth == 0 && self.at(Kw::BlockClose))
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
        match kind
        {
            TokenKind::Kw(Kw::BlockOpen | Kw::FnOpen) => *depth += 1,
            TokenKind::Kw(Kw::BlockClose) => *depth = depth.saturating_sub(1),
            TokenKind::Punct(Punct::FullStop) if *depth == 0 =>
            {
                return !is_if || !(self.at(Kw::ElseIf) || self.at(Kw::Else));
            }
            _ =>
            {}
        }
        false
    }
}
