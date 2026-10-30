use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::stmt::AllocStmt;
use zakac_ast::stmt::FreeStmt;
use zakac_ast::stmt::ResizeStmt;
use zakac_ast::stmt::SizeArg;
use zakac_ast::stmt::Stmt;
use zakac_ast::visit::Node;
use zakac_diagnostics::Diagnostic;
use zakac_i18n::t;
use zakac_lexer::classifiers::COUNT_ENDING;
use zakac_lexer::token::Kw;
use zakac_rust_sugars::ret_if;
use zakac_span::span::Span;

use crate::describe::kw_text;
use crate::describe::quote;
use crate::parser::PResult;
use crate::parser::Parser;

impl Parser<'_>
{
    /// Parse `မင်မိုရီ <size> နေရာယူ၍ လိပ်စာကို <place> တွင် သိမ်းထားမည်။`;
    /// the current token is `မင်မိုရီ`.
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

    /// Parse `လိပ်စာရှိကို မင်မိုရီ <size> ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။`
    /// after the place; the current token is `လိပ်စာရှိကို`.
    ///
    /// # Arguments
    ///
    /// * `target` - The place that holds the address.
    ///
    /// # Returns
    ///
    /// The reallocation, or a diagnostic.
    pub(crate) fn parse_resize_tail(&mut self, target: Expr) -> PResult<Stmt>
    {
        self.bump();
        self.expect(Kw::Mem)?;
        let size = self.parse_size_arg()?;
        self.expect(Kw::SoThat)?;
        self.expect(Kw::WillResize)?;
        let stop = self.expect_stop()?;
        let span = target.span().to(&stop.span);
        let node = ResizeStmt::builder().target(target).size(size).span(span).build();
        Ok(Stmt::Resize(node))
    }

    /// Parse `လိပ်စာရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။` after the place;
    /// the current token is `လိပ်စာရှိ မင်မိုရီကို`.
    ///
    /// # Arguments
    ///
    /// * `target` - The address to release.
    ///
    /// # Returns
    ///
    /// The release, or a diagnostic.
    pub(crate) fn parse_free_tail(&mut self, target: Expr) -> PResult<Stmt>
    {
        self.bump();
        self.expect(Kw::WillRelease)?;
        let stop = self.expect_stop()?;
        let span = target.span().to(&stop.span);
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
