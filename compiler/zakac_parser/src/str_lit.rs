use zakac_ast::expr::Expr;
use zakac_ast::expr::StrExpr;
use zakac_ast::expr::StrPiece;
use zakac_diagnostics::parser as diag;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex_at;
use zakac_lexer::token::StrPart;
use zakac_rust_sugars::ret_if;
use zakac_span::span::Span;

use crate::parser::PResult;
use crate::parser::Parser;

impl Parser<'_>
{
    /// Parse a string literal; the text of each template slot is lexed and
    /// parsed as an expression, so its errors point inside the literal.
    ///
    /// # Arguments
    ///
    /// * `idx` - The index of the literal in the lexer output.
    ///
    /// # Returns
    ///
    /// The literal, or the first diagnostic of a template slot.
    pub(crate) fn parse_str(&mut self, idx: u32) -> PResult<Expr>
    {
        let tok = self.bump();
        let parts = &self.strings[idx as usize].parts;
        let mut pieces: Vec<StrPiece> = Vec::with_capacity(parts.len());
        for part in parts
        {
            let piece = match part
            {
                StrPart::Text(text) | StrPart::Bad(text) =>
                {
                    StrPiece::Text(text.clone())
                }
                StrPart::Escape(ch) => StrPiece::Text(ch.to_string()),
                StrPart::TemplateSlot { span, .. } =>
                {
                    StrPiece::TemplateSlot(self.parse_template_slot(*span)?)
                }
            };
            push_piece(&mut pieces, piece);
        }
        let id = self.new_id();
        Ok(Expr::Str(StrExpr::builder().id(id).pieces(pieces).span(tok.span).build()))
    }

    /// Parse the text of a template slot as one expression.
    ///
    /// # Arguments
    ///
    /// * `span` - The region of the text between the braces.
    ///
    /// # Returns
    ///
    /// The expression, or the first lexer or parser diagnostic.
    fn parse_template_slot(&mut self, span: Span) -> PResult<Expr>
    {
        let text = self.text_of(span);
        let lexed = lex_at(text, span.lo, &LexerOptions::default());
        ret_if!(let Some(d) = lexed.diagnostics.first(), Err(d.clone()));
        ret_if!(text.trim().is_empty(), Err(diag::expected_operand(span, "}")));
        let mut inner = Parser::new(text, span.lo, &lexed);
        inner.next_id = self.next_id;
        inner.structs.extend(self.structs.iter());
        let expr = inner.parse_expr();
        let rest = (!inner.at_eof()).then(|| inner.err_expected("}"));
        self.next_id = inner.next_id;
        let expr = expr?;
        ret_if!(let Some(d) = rest, Err(d));
        Ok(expr)
    }
}

/// Add a piece, joining text to the text before it.
///
/// # Arguments
///
/// * `pieces` - The pieces so far.
/// * `piece` - The next piece.
fn push_piece(pieces: &mut Vec<StrPiece>, piece: StrPiece)
{
    let (Some(StrPiece::Text(last)), StrPiece::Text(text)) =
        (pieces.last_mut(), &piece)
    else
    {
        pieces.push(piece);
        return;
    };
    last.push_str(text);
}
