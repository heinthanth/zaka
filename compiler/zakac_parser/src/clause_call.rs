//! Calls written as a sentence whose arguments are whole clauses, as
//! `က နှင့် ခ ၏ ပေါင်းလဒ် ကို ပြ မည်`. The parser cuts the words before the
//! function at the prepositions of the function, and keeps each cut whose
//! pieces each read as one clause: one such cut is the call, and two are
//! ZK0260, as the sentence then has two readings.

use zakac_ast::expr::Phrases;
use zakac_diagnostics::parser as diag;
use zakac_lexer::token::Punct;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::clause::Mode;
use crate::parser::PResult;
use crate::parser::Parser;
use crate::using::CallWords;
use crate::using::Head;

/// One cut of the words before the function of a call into its parts,
/// each counted in tokens from the start of the call.
#[derive(Debug, Clone, PartialEq)]
struct Cut
{
    /// The tokens of the starting phrase; 0 with none.
    start: usize,
    /// Each argument, from its first token to the token after it.
    args: Vec<(usize, usize)>,
    /// The tokens after each argument: its preposition, a `၊`, or none.
    after: Vec<usize>,
    /// Whether each argument has a preposition after it.
    preps: Vec<bool>,
    /// The tokens of the ending phrase; 0 with none.
    end: usize,
}

impl Parser<'_>
{
    /// Read a call written as a sentence from the current token, its
    /// arguments as whole clauses, when one cut of its words reads; the
    /// function nearest the end of the sentence is tried first.
    ///
    /// # Arguments
    ///
    /// * `mode` - Whether the call can be a statement.
    ///
    /// # Returns
    ///
    /// The call, ZK0260 when the words cut into two readings, or `None` with no
    /// cut.
    pub(crate) fn attempt_clause_call(&mut self, mode: Mode) -> Option<PResult<Head>>
    {
        let callees = self.callees_ahead();
        callees.into_iter().rev().find_map(|at| self.clause_call_at(at, mode))
    }

    /// The places of the functions ahead, outside parentheses and before the
    /// end of the sentence, that end a call written as a sentence.
    ///
    /// # Returns
    ///
    /// The distances to the function names, nearest first.
    fn callees_ahead(&self) -> Vec<usize>
    {
        let (mut depth, mut found) = (0usize, Vec::new());
        for n in 0..
        {
            let tok = self.peek_n(n);
            let closes =
                matches!(tok.kind, TokenKind::Punct(Punct::RParen | Punct::RBracket));
            let ends =
                matches!(tok.kind, TokenKind::Eof | TokenKind::Punct(Punct::FullStop));
            do_if!(ends || (closes && depth == 0), break);
            let opens =
                matches!(tok.kind, TokenKind::Punct(Punct::LParen | Punct::LBracket));
            depth = tern!(opens, depth + 1, tern!(closes, depth - 1, depth));
            let callee = depth == 0 && n > 0 && self.call_end_at(n).is_some();
            do_if!(callee, found.push(n));
        }
        found
    }

    /// Read the call of the function `at` tokens ahead, when its words cut
    /// just once.
    ///
    /// # Arguments
    ///
    /// * `at` - The distance to the function name.
    /// * `mode` - Whether the call can be a statement.
    ///
    /// # Returns
    ///
    /// The call, ZK0260 for two cuts, or `None` with none.
    fn clause_call_at(&mut self, at: usize, mode: Mode) -> Option<PResult<Head>>
    {
        let name = self.callee_text(&self.peek_n(at));
        let lists: Vec<CallWords> = self.fn_words(&name).cloned().collect();
        let found: Vec<Cut> = lists.iter().flat_map(|w| self.cuts_of(w, at)).collect();
        let mut cuts: Vec<Cut> = Vec::new();
        do_for!(cut in found, do_if!(!cuts.contains(&cut), cuts.push(cut)));
        ret_if!(cuts.is_empty(), None);
        let span = self.peek().span.to(&self.peek_n(at).span);
        ret_if!(cuts.len() > 1, Some(Err(diag::ambiguous_call(span, &name))));
        Some(self.read_cut(&cuts[0], mode))
    }

    /// The cuts of the words before the function `at` tokens ahead that fit
    /// one head of it: its phrases where it has them, and an argument for
    /// each parameter with its preposition after it.
    ///
    /// # Arguments
    ///
    /// * `words` - The words of the head.
    /// * `at` - The distance to the function name.
    ///
    /// # Returns
    ///
    /// The cuts.
    fn cuts_of(&mut self, words: &CallWords, at: usize) -> Vec<Cut>
    {
        let start = match &words.start
        {
            Some(phrase) => self.spells_at(phrase, 0),
            None => Some(0),
        };
        let Some(start) = start
        else
        {
            return Vec::new();
        };
        let end = match &words.end
        {
            Some(phrase) =>
            {
                (start..at).find(|k| self.spells_at(phrase, *k) == Some(at - k))
            }
            None => Some(at),
        };
        let Some(stop) = end
        else
        {
            return Vec::new();
        };
        #[rustfmt::skip]
        let cut = Cut { start, args: Vec::new(), after: Vec::new(), preps: Vec::new(), end: at - stop };
        self.cut_args(words, cut, start, stop)
    }

    /// The cuts of the arguments from the one `cut` reached so far, its
    /// next argument starting at `pos`; the arguments end at `stop`.
    ///
    /// # Arguments
    ///
    /// * `words` - The words of the head.
    /// * `cut` - The cut so far.
    /// * `pos` - Where the next argument starts.
    /// * `stop` - Where the arguments end.
    ///
    /// # Returns
    ///
    /// The cuts that read to the end.
    fn cut_args(
        &mut self, words: &CallWords, cut: Cut, pos: usize, stop: usize,
    ) -> Vec<Cut>
    {
        let n = cut.args.len();
        ret_if!(n == words.params, tern!(pos == stop, vec![cut], Vec::new()));
        let prep = words.prepositions.get(n).cloned().flatten();
        let has_prep = prep.is_some();
        let ends = self.arg_ends(prep, pos, stop, n + 1 == words.params);
        let next = |(end, after): (usize, usize)| {
            let mut next = cut.clone();
            next.args.push((pos, end));
            next.after.push(after);
            next.preps.push(has_prep);
            (next, end + after)
        };
        let nexts: Vec<(Cut, usize)> = ends.into_iter().map(next).collect();
        nexts
            .into_iter()
            .flat_map(|(c, at)| self.cut_args(words, c, at, stop))
            .collect()
    }

    /// The places an argument from `pos` can end: before each spelling of
    /// its preposition, or anywhere when it has none, a `၊` after it taken
    /// along; the last argument ends at `stop`. Each place leaves an argument
    /// that reads as one clause.
    ///
    /// # Arguments
    ///
    /// * `prep` - The preposition of the parameter, when it has one.
    /// * `pos` - Where the argument starts.
    /// * `stop` - Where the arguments end.
    /// * `last` - Whether this is the last argument.
    ///
    /// # Returns
    ///
    /// Each end, with the number of tokens after it before the next argument.
    fn arg_ends(
        &mut self, prep: Option<String>, pos: usize, stop: usize, last: bool,
    ) -> Vec<(usize, usize)>
    {
        let comma = |p: &Self, q: usize| {
            usize::from(p.peek_n(q).kind == TokenKind::Punct(Punct::ListSep))
        };
        let ends: Vec<(usize, usize)> = match &prep
        {
            Some(m) => (pos + 1..stop)
                .filter_map(|q| self.spells_at(m, q).map(|len| (q, len)))
                .filter(|(q, len)| !last || q + len == stop)
                .collect(),
            None if last => vec![(stop, 0)],
            None => (pos + 1..stop).map(|q| (q, comma(self, q))).collect(),
        };
        let top: Vec<(usize, usize)> =
            ends.into_iter().filter(|(q, _)| self.at_top(pos, *q)).collect();
        top.into_iter().filter(|(q, _)| self.arg_reads(pos, *q)).collect()
    }

    /// Whether the token `to` ahead stands outside the parentheses opened
    /// from `from` on.
    ///
    /// # Arguments
    ///
    /// * `from` - Where the argument starts.
    /// * `to` - The token.
    ///
    /// # Returns
    ///
    /// `true` when it does.
    fn at_top(&self, from: usize, to: usize) -> bool
    {
        let step = |depth: i32, n: usize| match self.peek_n(n).kind
        {
            TokenKind::Punct(Punct::LParen | Punct::LBracket) => depth + 1,
            TokenKind::Punct(Punct::RParen | Punct::RBracket) => depth - 1,
            _ => depth,
        };
        (from..to).fold(0, step) == 0
    }

    /// Whether the tokens from `from` to `to` ahead read as one clause and
    /// nothing more. Nothing is consumed and nothing is reported.
    ///
    /// # Arguments
    ///
    /// * `from` - Where the argument starts.
    /// * `to` - Where it ends.
    ///
    /// # Returns
    ///
    /// `true` when they do.
    fn arg_reads(&mut self, from: usize, to: usize) -> bool
    {
        let (cur, diags, next_id) = (self.cur, self.diags.len(), self.next_id);
        let base = cur.pos();
        self.cur = cur.at(base + from).limited(Some(base + to));
        let parsed = self.parse_clause_expr().is_ok();
        let read =
            parsed && self.peek().kind == TokenKind::Eof && self.diags.len() == diags;
        self.cur = cur;
        self.diags.truncate(diags);
        self.next_id = next_id;
        read
    }

    /// Read a call along a cut: its phrases, each argument as a clause with
    /// its preposition, and the function with its end.
    ///
    /// # Arguments
    ///
    /// * `cut` - The cut.
    /// * `mode` - Whether the call can be a statement.
    ///
    /// # Returns
    ///
    /// The call, or a diagnostic.
    fn read_cut(&mut self, cut: &Cut, mode: Mode) -> PResult<Head>
    {
        let (base, outer) = (self.cur.pos(), self.cur.limit());
        let span = self.peek().span;
        let start = (cut.start > 0).then(|| self.take_words(cut.start));
        let (mut args, mut prepositions) = (Vec::new(), Vec::new());
        for (n, (_, to)) in cut.args.iter().enumerate()
        {
            self.cur = self.cur.limited(Some(base + to));
            let arg = self.parse_clause_expr();
            self.cur = self.cur.limited(outer);
            args.push(arg?);
            let word = cut.preps[n].then(|| self.take_words(cut.after[n]));
            do_if!(!cut.preps[n] && cut.after[n] > 0, self.advance());
            prepositions.push(word);
        }
        let end = (cut.end > 0).then(|| self.take_words(cut.end));
        let phrases = Phrases::builder().maybe_start(start).maybe_end(end).build();
        self.finish_sentence_call(span, args, prepositions, phrases, mode)
    }
}
