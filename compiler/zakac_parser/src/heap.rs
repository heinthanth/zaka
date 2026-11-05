use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::stmt::AllocStmt;
use zakac_ast::stmt::BlitStmt;
use zakac_ast::stmt::FillStmt;
use zakac_ast::stmt::FreeStmt;
use zakac_ast::stmt::ResizeStmt;
use zakac_ast::stmt::SizeArg;
use zakac_ast::stmt::Stmt;
use zakac_diagnostics::Diagnostic;
use zakac_i18n::t;
use zakac_lexer::classifiers::COUNT_ENDING;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::ret_if;
use zakac_span::span::Span;

use crate::describe::kw_text;
use crate::describe::quote;
use crate::parser::PResult;
use crate::parser::Parser;

impl Parser<'_>
{
    /// Parse `မင်မိုရီတွင် <size> နေရာယူ၍ လိပ်စာကို <place> တွင် သိမ်းထားမည်။`;
    /// the current token is `မင်မိုရီတွင်`.
    ///
    /// # Returns
    ///
    /// The allocation, or a diagnostic.
    pub(crate) fn parse_alloc(&mut self) -> PResult<Stmt>
    {
        let start = self.bump().span;
        let size = self.parse_size_arg()?;
        self.expect(Kw::AllocTarget)?;
        let target = self.parse_clause_expr()?;
        self.expect(Kw::At)?;
        self.expect(Kw::WillKeep)?;
        let stop = self.expect_stop()?;
        let span = start.to(&stop.span);
        let node = AllocStmt::builder().size(size).target(target).span(span).build();
        Ok(Stmt::Alloc(node))
    }

    /// Parse a heap statement that starts with `လိပ်စာ <place> ရှိ`; the
    /// current token is `လိပ်စာ`. A size before `မင်မိုရီကို` makes a blit.
    /// After `မင်မိုရီကို`, `ပြန်လည်စွန့်လွှတ်မည်` makes a release; the verb
    /// that ends the statement tells a fill from a resize, so a fill value
    /// that starts with a kind word reads as a value, not as a size.
    ///
    /// # Returns
    ///
    /// The statement, or a diagnostic.
    pub(crate) fn parse_heap(&mut self) -> PResult<Stmt>
    {
        let start = self.bump().span;
        let target = self.parse_clause_expr()?;
        ret_if!(self.at(Kw::Located), self.parse_blit_end(start, target));
        self.expect(Kw::HeapHead)?;
        ret_if!(self.at(Kw::WillRelease), self.parse_free_end(start, target));
        ret_if!(self.ends_in_fill(), self.parse_fill_end(start, target));
        self.parse_resize_end(start, target)
    }

    /// Whether the statement ends in `ဖြည့်မည်`: the verb of a fill comes
    /// before the next `။`. No heap statement holds another statement, so
    /// the first `။` ends this one.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn ends_in_fill(&self) -> bool
    {
        let stop = TokenKind::Punct(Punct::FullStop);
        let kinds = (0..).map(|n| self.peek_n(n).kind);
        let mut rest = kinds.take_while(|k| *k != stop && *k != TokenKind::Eof);
        rest.any(|k| k == TokenKind::Kw(Kw::WillFill))
    }

    /// Parse `<size> ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။` after the head.
    ///
    /// # Arguments
    ///
    /// * `start` - The span of `လိပ်စာ`.
    /// * `target` - The place that holds the address.
    ///
    /// # Returns
    ///
    /// The reallocation, or a diagnostic.
    fn parse_resize_end(&mut self, start: Span, target: Expr) -> PResult<Stmt>
    {
        let size = self.parse_size_arg()?;
        self.expect(Kw::SoThat)?;
        self.expect(Kw::WillResize)?;
        let span = start.to(&self.expect_stop()?.span);
        let node = ResizeStmt::builder().target(target).size(size).span(span).build();
        Ok(Stmt::Resize(node))
    }

    /// Parse `ရှိ <size> မင်မိုရီကို လိပ်စာ <dst> ဆီသို့ ကူးယူမည်။`, or the
    /// same with `ပြောင်းထည့်မည်`, after the source address.
    ///
    /// # Arguments
    ///
    /// * `start` - The span of `လိပ်စာ`.
    /// * `src` - The address the cells come from.
    ///
    /// # Returns
    ///
    /// The blit, or a diagnostic.
    fn parse_blit_end(&mut self, start: Span, src: Expr) -> PResult<Stmt>
    {
        self.bump();
        let size = self.parse_size_arg()?;
        self.expect(Kw::Mem)?;
        self.expect(Kw::Ko)?;
        self.expect(Kw::Address)?;
        let dst = self.parse_clause_expr()?;
        self.expect(Kw::Toward)?;
        let verb = self.expect_one_of(&[Kw::WillBlit, Kw::WillShift])?;
        let overlap = verb.kind == TokenKind::Kw(Kw::WillShift);
        let span = start.to(&self.expect_stop()?.span);
        #[rustfmt::skip]
        let node = BlitStmt::builder().src(src).size(size).dst(dst).overlap(overlap).span(span).build();
        Ok(Stmt::Blit(node))
    }

    /// Parse `<value> ဖြင့် <size> ဖြည့်မည်။` after the head. A `ဖြင့်`
    /// right after a lone operand ends the value.
    ///
    /// # Arguments
    ///
    /// * `start` - The span of `လိပ်စာ`.
    /// * `dst` - The address of the first cell.
    ///
    /// # Returns
    ///
    /// The fill, or a diagnostic.
    fn parse_fill_end(&mut self, start: Span, dst: Expr) -> PResult<Stmt>
    {
        let outer = std::mem::replace(&mut self.in_fill, true);
        let value = self.parse_clause_expr();
        self.in_fill = outer;
        let value = value?;
        self.expect(Kw::Hpjint)?;
        let size = self.parse_size_arg()?;
        self.expect(Kw::WillFill)?;
        let span = start.to(&self.expect_stop()?.span);
        let node =
            FillStmt::builder().dst(dst).value(value).size(size).span(span).build();
        Ok(Stmt::Fill(node))
    }

    /// Parse `ပြန်လည်စွန့်လွှတ်မည်။` after the head.
    ///
    /// # Arguments
    ///
    /// * `start` - The span of `လိပ်စာ`.
    /// * `target` - The address to release.
    ///
    /// # Returns
    ///
    /// The release, or a diagnostic.
    fn parse_free_end(&mut self, start: Span, target: Expr) -> PResult<Stmt>
    {
        self.bump();
        let span = start.to(&self.expect_stop()?.span);
        Ok(Stmt::Free(FreeStmt::builder().target(target).span(span).build()))
    }

    /// Parse the size of a heap statement: `<count> ဘိုက်စာ`, or
    /// `<kind> <count> ခုစာ`. A kind starts the second form unless it is the
    /// start of `<kind> ၏ အရွယ်အစား`.
    ///
    /// # Returns
    ///
    /// The size, or a diagnostic; a missing `ခုစာ` suggests parentheses
    /// around a kind-prefixed literal.
    pub(crate) fn parse_size_arg(&mut self) -> PResult<SizeArg>
    {
        let start = self.peek().span;
        ret_if!(!self.starts_kind() || self.at_size_of(), self.parse_octets(start));
        let unit = self.parse_kind()?;
        let count = self.parse_clause_expr()?;
        ret_if!(!self.at(Kw::Items), Err(self.err_items()));
        let end = self.bump().span;
        let word = self.text_of(end);
        let classifier = word.strip_suffix(COUNT_ENDING).unwrap_or(word);
        let classifier = Ident::builder().name(classifier).span(end).build();
        let span = start.to(&end);
        #[rustfmt::skip]
        return Ok(SizeArg::builder().count(count).unit(unit).classifier(classifier).span(span).build());
    }

    /// Parse `<count> ဘိုက်စာ`.
    ///
    /// # Arguments
    ///
    /// * `start` - The span of the first token of the count.
    ///
    /// # Returns
    ///
    /// The size in octets, or a diagnostic.
    fn parse_octets(&mut self, start: Span) -> PResult<SizeArg>
    {
        let count = self.parse_clause_expr()?;
        let end = self.expect(Kw::Octets)?.span;
        let span = start.to(&end);
        Ok(SizeArg::builder().count(count).span(span).build())
    }

    /// An `expected-token` diagnostic for a missing `ခုစာ`, with a hint to
    /// put a kind-prefixed literal in parentheses.
    ///
    /// # Returns
    ///
    /// The diagnostic.
    fn err_items(&self) -> Diagnostic
    {
        let diag = self.err_expected(&quote(&kw_text(Kw::Items)));
        diag.with_help(t!("parser.size_parens"))
    }
}
