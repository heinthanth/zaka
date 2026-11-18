use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::SizeExpr;
use zakac_ast::kind::Kind;
use zakac_ast::stmt::AllocStmt;
use zakac_ast::stmt::BlitStmt;
use zakac_ast::stmt::FillStmt;
use zakac_ast::stmt::FreeStmt;
use zakac_ast::stmt::ResizeStmt;
use zakac_ast::stmt::SizeArg;
use zakac_ast::stmt::Stmt;
use zakac_ast::visit::Node;
use zakac_diagnostics::Diagnostic;
use zakac_lexer::classifiers::COUNT_ENDING;
use zakac_lexer::token::Kw;
use zakac_rust_sugars::ret_if;
use zakac_span::span::Span;

use crate::describe::kw_text;
use crate::describe::quote;
use crate::parser::PResult;
use crate::parser::Parser;

/// `သိမ်းထား`: take room on the heap.
const ALLOC: &str = "သိမ်းထား";
/// `ပြန်လည်စွန့်လွှတ်`: give a block back.
const FREE: &str = "ပြန်လည်စွန့်လွှတ်";
/// `နေရာအချဲ့အချုံ့လုပ်`: give a block a new size.
const RESIZE: &str = "နေရာအချဲ့အချုံ့လုပ်";
/// `ဖြည့်`: set each cell of a block to one value.
const FILL: &str = "ဖြည့်";
/// `ကူးယူ`: write the cells of one block to another, with no overlap.
const BLIT: &str = "ကူးယူ";
/// `ပြောင်းထည့်`: write the cells of one block to another, overlap allowed.
const SHIFT: &str = "ပြောင်းထည့်";

/// The heap functions of the core, each with the number of its
/// arguments: the parser turns a call statement of one into its statement
/// node.
pub(crate) const HEAP_FNS: [(&str, usize); 6] =
    [(ALLOC, 2), (FREE, 1), (RESIZE, 2), (FILL, 3), (BLIT, 3), (SHIFT, 3)];

/// The size argument of a heap call as the node holds it: a size operand
/// gives its count and its kind; another value counts octets.
///
/// # Arguments
///
/// * `arg` - The argument.
///
/// # Returns
///
/// The size.
fn size_arg(arg: Expr) -> SizeArg
{
    let span = arg.span();
    match arg
    {
        Expr::Size(n) =>
        {
            SizeArg::builder().count(*n.count).maybe_unit(n.unit).span(n.span).build()
        }
        arg => SizeArg::builder().count(arg).span(span).build(),
    }
}

impl Parser<'_>
{
    /// Turn a call statement of a heap function of the core into the
    /// statement node the later stages take: `သိမ်းထား` an allocation,
    /// `ပြန်လည်စွန့်လွှတ်` a release, `နေရာအချဲ့အချုံ့လုပ်` a resize, `ဖြည့်` a fill,
    /// `ကူးယူ` and `ပြောင်းထည့်` a blit. A function of the file of such a name
    /// hides the one of the core, and its calls remain calls; so does a
    /// call with another number of arguments, which the check reports.
    ///
    /// # Arguments
    ///
    /// * `stmt` - The statement.
    ///
    /// # Returns
    ///
    /// The heap statement, or the statement as it was.
    pub(crate) fn heap_node(&self, stmt: Stmt) -> Stmt
    {
        let Stmt::Call(call) = stmt
        else
        {
            return stmt;
        };
        let name = call.callee.name.as_str();
        let heap = self.core_heap.contains(name) && call.place.is_none();
        let fits =
            HEAP_FNS.iter().any(|(n, count)| *n == name && *count == call.args.len());
        ret_if!(!heap || !fits || call.pass, Stmt::Call(call));
        let span = call.span;
        let mut args = call.args.into_iter();
        let mut next = || args.next().expect("the count of the arguments is checked");
        match name
        {
            ALLOC =>
            {
                let size = size_arg(next());
                Stmt::Alloc(
                    AllocStmt::builder().size(size).target(next()).span(span).build(),
                )
            }
            FREE => Stmt::Free(FreeStmt::builder().target(next()).span(span).build()),
            RESIZE =>
            {
                let target = next();
                let size = size_arg(next());
                Stmt::Resize(
                    ResizeStmt::builder().target(target).size(size).span(span).build(),
                )
            }
            FILL =>
            {
                let (dst, value) = (next(), next());
                let size = size_arg(next());
                #[rustfmt::skip]
                let node = FillStmt::builder().dst(dst).value(value).size(size).span(span).build();
                Stmt::Fill(node)
            }
            _ =>
            {
                let (src, size, dst) = (next(), size_arg(next()), next());
                #[rustfmt::skip]
                let node = BlitStmt::builder().src(src).size(size).dst(dst).overlap(name == SHIFT).span(span).build();
                Stmt::Blit(node)
            }
        }
    }

    /// Parse the `<classifier>စာ` that closes a size of a kind, and give the
    /// kind its classifier, the text before `စာ`.
    ///
    /// # Arguments
    ///
    /// * `unit` - The kind of each value.
    ///
    /// # Returns
    ///
    /// The span of the word, or a diagnostic.
    fn parse_items_word(&mut self, unit: &mut Kind) -> PResult<Span>
    {
        ret_if!(!self.at(Kw::Items), Err(self.err_items()));
        let end = self.bump().span;
        let word = self.text_of(end);
        let classifier = word.strip_suffix(COUNT_ENDING).unwrap_or(word);
        unit.classifier = Some(Ident::builder().name(classifier).span(end).build());
        Ok(end)
    }

    /// Read a size operand that starts with a kind, `<kind> <count>
    /// <classifier>စာ`, when one reads here; nothing is read otherwise.
    ///
    /// # Returns
    ///
    /// The size, or `None` with no such operand here.
    pub(crate) fn attempt_kind_size(&mut self) -> Option<Expr>
    {
        ret_if!(!self.starts_kind() || self.at_size_of(), None);
        let (cur, diags, next_id) = (self.cur, self.diags.len(), self.next_id);
        let read = self.parse_kind_size();
        ret_if!(let Ok(size) = read, Some(size));
        self.cur = cur;
        self.diags.truncate(diags);
        self.next_id = next_id;
        None
    }

    /// Parse `<kind> <count> <classifier>စာ`.
    ///
    /// # Returns
    ///
    /// The size, or a diagnostic.
    fn parse_kind_size(&mut self) -> PResult<Expr>
    {
        let start = self.peek().span;
        let mut unit = self.parse_kind()?;
        let count = self.parse_notation()?;
        let end = self.parse_items_word(&mut unit)?;
        let span = start.to(&end);
        let id = self.new_id();
        #[rustfmt::skip]
        let node = SizeExpr::builder().id(id).count(Box::new(count)).unit(unit).span(span).build();
        Ok(Expr::Size(node))
    }

    /// Close a notation expression with `ဘိုက်စာ` when the word follows it:
    /// the expression is then a count of octets.
    ///
    /// # Arguments
    ///
    /// * `count` - The expression.
    ///
    /// # Returns
    ///
    /// The size, or the expression as it was.
    pub(crate) fn octets_after(&mut self, count: Expr) -> Expr
    {
        ret_if!(!self.at(Kw::Octets), count);
        let end = self.bump().span;
        let span = count.span().to(&end);
        let id = self.new_id();
        #[rustfmt::skip]
        let node = SizeExpr::builder().id(id).count(Box::new(count)).span(span).build();
        Expr::Size(node)
    }

    /// An `expected-token` diagnostic for a missing `ခုစာ`.
    ///
    /// # Returns
    ///
    /// The diagnostic.
    fn err_items(&self) -> Diagnostic { self.err_expected(&quote(&kw_text(Kw::Items))) }
}
