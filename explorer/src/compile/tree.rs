use bon::Builder;
use zakac_ast::expr::Expr;
use zakac_ast::expr::NodeId;
use zakac_ast::stmt::Program;
use zakac_ast::visit::Node;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::walk_expr;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::semcheck::KindTable;
use zakac_span::source_file::SourceFile;
use zakac_span::span::Span;

use crate::compile::spans::Range;
use crate::compile::spans::to_original;

/// Most characters of source text shown next to a node label.
const DETAIL_CHARS: usize = 40;

/// One node of the tree view.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct TreeNode
{
    /// The node form, such as `decl` or `verb`.
    pub label: &'static str,
    /// The start of the source text of the node, on one line.
    #[builder(default)]
    pub detail: String,
    /// The source region in the original text.
    pub range: Range,
    /// The node number of an expression.
    pub id: Option<u32>,
    /// The data kind the check gave an expression.
    pub kind: Option<String>,
    /// The child nodes, in source order.
    #[builder(default)]
    pub children: Vec<TreeNode>,
}

impl Default for TreeNode
{
    /// A blank `program` node.
    ///
    /// # Returns
    ///
    /// The node.
    fn default() -> Self
    {
        TreeNode::builder().label("program").range(Range::default()).build()
    }
}

impl TreeNode
{
    /// Whether the node is a literal, whose source text is its value.
    ///
    /// # Returns
    ///
    /// `true` for `int`, `float`, `char`, `bool` and `str` nodes.
    pub fn is_literal(&self) -> bool
    {
        matches!(self.label, "int" | "float" | "char" | "bool" | "str")
    }
}

/// Build the tree view of a program.
///
/// # Arguments
///
/// * `file` - The source file of the program.
/// * `original` - The original text of the file.
/// * `program` - The program.
/// * `kinds` - The kinds the check found; blank when the check did not run.
///
/// # Returns
///
/// The `program` node.
pub fn build_tree(
    file: &SourceFile, original: &str, program: &Program, kinds: &KindTable,
) -> TreeNode
{
    let mut builder =
        TreeBuilder { file, original, kinds, stack: Vec::new(), pending_id: None };
    builder.visit_program(program);
    builder.stack.pop().unwrap_or_default()
}

/// Visitor that turns the AST into [`TreeNode`]s.
struct TreeBuilder<'a>
{
    /// The source file.
    file: &'a SourceFile,
    /// The original text.
    original: &'a str,
    /// Kinds of the expressions.
    kinds: &'a KindTable,
    /// Open nodes; the last one gets the next finished child.
    stack: Vec<TreeNode>,
    /// Node number of the expression whose node opens next.
    pending_id: Option<NodeId>,
}

impl TreeBuilder<'_>
{
    /// The one-line source text of a span, cut to [`DETAIL_CHARS`].
    ///
    /// # Arguments
    ///
    /// * `span` - The span.
    ///
    /// # Returns
    ///
    /// The text.
    fn detail(&self, span: &Span) -> String
    {
        ret_if!(span.is_placeholder(), String::new());
        let lo = self.file.relative_position(&span.lo);
        let hi = self.file.relative_position(&span.hi);
        let text = self.file.snippet(&lo, &hi).unwrap_or("");
        let line = text.lines().next().unwrap_or("").trim();
        let cut: String = line.chars().take(DETAIL_CHARS).collect();
        let more = text.chars().count() > cut.chars().count();
        tern!(more, format!("{cut} …"), cut)
    }
}

impl Visitor for TreeBuilder<'_>
{
    fn enter(&mut self, n: &dyn Node)
    {
        let id = self.pending_id.take();
        let kind = id.and_then(|i| self.kinds.get(&i)).map(|k| k.name());
        let span = n.span();
        let node = TreeNode::builder()
            .label(n.label())
            .detail(self.detail(&span))
            .range(to_original(self.file, self.original, &span))
            .maybe_id(id.map(|i| i.0))
            .maybe_kind(kind)
            .build();
        self.stack.push(node);
    }

    fn leave(&mut self, _: &dyn Node)
    {
        ret_if!(self.stack.len() < 2);
        let done = self.stack.pop().unwrap_or_default();
        do_if!(let Some(parent) = self.stack.last_mut(), parent.children.push(done));
    }

    fn visit_expr(&mut self, n: &Expr)
    {
        self.pending_id = Some(n.id());
        walk_expr(self, n);
    }
}
