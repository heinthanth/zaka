use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::Stmt;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_span::span::Span;

/// The number of a node in its [`Cfg`].
pub(crate) type NodeId = u32;

/// What a node runs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum NodeKind<'m>
{
    /// The start of the routine.
    Start,
    /// A statement with no statement nested in it; a return goes to the
    /// exit.
    Leaf(&'m Stmt),
    /// The test of an `if`: the first edge when it holds, the second when
    /// it does not.
    Branch(&'m Expr),
    /// The test of a loop: the first edge into the loop, the second out.
    LoopHead(&'m Expr),
    /// The first and last values of a counted loop, run once.
    CountedInit(&'m Stmt),
    /// The step of a counted loop: the first edge into the loop with the
    /// next value, the second out.
    CountedHead(&'m Stmt),
    /// The end of the statements, where the routine falls off.
    End,
    /// The exit, after each return and the end.
    Exit,
}

/// One node of a [`Cfg`].
#[derive(Debug, Clone, PartialEq, bon::Builder)]
pub(crate) struct Node<'m>
{
    /// What it runs.
    pub(crate) kind: NodeKind<'m>,
    /// The nodes that run next, in the order [`NodeKind`] gives.
    pub(crate) succ: Vec<NodeId>,
    /// Source region.
    pub(crate) span: Span,
}

/// The control-flow graph of one routine.
#[derive(Debug, Clone, PartialEq, bon::Builder)]
pub(crate) struct Cfg<'m>
{
    /// The nodes, in number order.
    pub(crate) nodes: Vec<Node<'m>>,
    /// The start node.
    pub(crate) start: NodeId,
    /// The exit node.
    pub(crate) exit: NodeId,
    /// The nodes the start reaches, in reverse postorder.
    pub(crate) rpo: Vec<NodeId>,
}

/// An edge whose target is not known: the node and the number of its
/// successor.
type Edge = (NodeId, usize);

/// A successor not linked so far.
const OPEN: NodeId = NodeId::MAX;

/// The edges that leave a loop before its test fails, linked when the
/// loop is done.
#[derive(Debug, Default)]
struct LoopFrame
{
    /// The edges of each `break`, which go past the loop.
    breaks: Vec<Edge>,
    /// The edges of each `continue`, which go to the update or the test.
    continues: Vec<Edge>,
}

/// The graph while it is built.
struct Builder<'m>
{
    /// The nodes so far.
    nodes: Vec<Node<'m>>,
    /// The edges that fall through to the next node.
    pending: Vec<Edge>,
    /// The loops around the statement being built, inner last.
    frames: Vec<LoopFrame>,
    /// The edges of each return, which go to the exit.
    returns: Vec<Edge>,
}

/// Build the control-flow graph of a statement list.
///
/// # Arguments
///
/// * `block` - The statements of the routine.
/// * `end_span` - The span of the start, the end and the exit.
///
/// # Returns
///
/// The graph.
pub(crate) fn build_cfg(block: &[Stmt], end_span: Span) -> Cfg<'_>
{
    let start =
        Node::builder().kind(NodeKind::Start).succ(vec![OPEN]).span(end_span).build();
    let mut b = Builder {
        nodes: vec![start],
        pending: vec![(0, 0)],
        frames: Vec::new(),
        returns: Vec::new(),
    };
    b.stmts(block);
    let end = b.add(NodeKind::End, end_span, 1);
    b.pending = std::mem::take(&mut b.returns);
    b.pending.push((end, 0));
    let exit = b.add(NodeKind::Exit, end_span, 0);
    let mut cfg =
        Cfg::builder().nodes(b.nodes).start(0).exit(exit).rpo(Vec::new()).build();
    cfg.rpo = cfg.postorder().into_iter().rev().collect();
    cfg
}

impl<'m> Builder<'m>
{
    /// Add a node, link the pending edges to it, and leave nothing
    /// pending.
    ///
    /// # Arguments
    ///
    /// * `kind` - What it runs.
    /// * `span` - Source region.
    /// * `edges` - The number of its successors.
    ///
    /// # Returns
    ///
    /// Its number.
    fn add(&mut self, kind: NodeKind<'m>, span: Span, edges: usize) -> NodeId
    {
        let n = self.nodes.len() as NodeId;
        self.nodes.push(
            Node::builder().kind(kind).succ(vec![OPEN; edges]).span(span).build(),
        );
        let pending = std::mem::take(&mut self.pending);
        self.link(pending, n);
        n
    }

    /// Point edges at a node.
    ///
    /// # Arguments
    ///
    /// * `edges` - The edges.
    /// * `to` - The node.
    fn link(&mut self, edges: Vec<Edge>, to: NodeId)
    {
        do_for!((n, i) in edges, self.nodes[n as usize].succ[i] = to);
    }

    /// Add the nodes of a statement list.
    ///
    /// # Arguments
    ///
    /// * `stmts` - The statements.
    fn stmts(&mut self, stmts: &'m [Stmt]) { do_for!(s in stmts, self.stmt(s)) }

    /// Add the nodes of one statement.
    ///
    /// # Arguments
    ///
    /// * `s` - The statement.
    fn stmt(&mut self, s: &'m Stmt)
    {
        match s
        {
            Stmt::If { cond, then, els, .. } => self.branch(cond, then, els),
            Stmt::Loop { init, cond, update, block, .. } =>
            {
                self.stmts(init);
                let head = self.add(NodeKind::LoopHead(cond), cond.span, 2);
                self.repeat(head, block, update);
            }
            Stmt::Counted { block, span, .. } =>
            {
                let init = self.add(NodeKind::CountedInit(s), *span, 1);
                self.pending = vec![(init, 0)];
                let head = self.add(NodeKind::CountedHead(s), *span, 2);
                self.repeat(head, block, &[]);
            }
            Stmt::Break { .. } => self.leave(|frame| &mut frame.breaks),
            Stmt::Continue { .. } => self.leave(|frame| &mut frame.continues),
            Stmt::Return { span, .. } =>
            {
                let n = self.add(NodeKind::Leaf(s), *span, 1);
                self.returns.push((n, 0));
            }
            _ =>
            {
                let n = self.add(NodeKind::Leaf(s), s.span(), 1);
                self.pending = vec![(n, 0)];
            }
        }
    }

    /// Add the nodes of an `if`.
    ///
    /// # Arguments
    ///
    /// * `cond` - The test.
    /// * `then` - The statements when it holds.
    /// * `els` - The statements when it does not.
    fn branch(&mut self, cond: &'m Expr, then: &'m [Stmt], els: &'m [Stmt])
    {
        let test = self.add(NodeKind::Branch(cond), cond.span, 2);
        self.pending = vec![(test, 0)];
        self.stmts(then);
        let joined = std::mem::replace(&mut self.pending, vec![(test, 1)]);
        self.stmts(els);
        self.pending.extend(joined);
    }

    /// Add the loop statements after a test, the update after them, and
    /// the edges back to the test; leave the exits of the loop pending.
    ///
    /// # Arguments
    ///
    /// * `head` - The test node.
    /// * `block` - The loop statements.
    /// * `update` - The statements after each round.
    fn repeat(&mut self, head: NodeId, block: &'m [Stmt], update: &'m [Stmt])
    {
        self.pending = vec![(head, 0)];
        self.frames.push(LoopFrame::default());
        self.stmts(block);
        let frame = self.frames.pop().expect("the frame was pushed");
        self.pending.extend(frame.continues);
        self.stmts(update);
        let back = std::mem::take(&mut self.pending);
        self.link(back, head);
        self.pending = frame.breaks;
        self.pending.insert(0, (head, 1));
    }

    /// Move the pending edges to a list of the innermost loop, for a
    /// `break` or a `continue`; the statements after it are not reached.
    ///
    /// # Arguments
    ///
    /// * `list` - Picks the list of the loop.
    fn leave(&mut self, list: impl FnOnce(&mut LoopFrame) -> &mut Vec<Edge>)
    {
        let pending = std::mem::take(&mut self.pending);
        let frame = self.frames.last_mut().expect("a loop is around a break");
        list(frame).extend(pending);
    }
}

impl Cfg<'_>
{
    /// The nodes that run before each node.
    ///
    /// # Returns
    ///
    /// The predecessors of each node, in node order.
    #[cfg(test)]
    pub(crate) fn preds(&self) -> Vec<Vec<NodeId>>
    {
        let mut preds = vec![Vec::new(); self.nodes.len()];
        for (n, node) in self.nodes.iter().enumerate()
        {
            do_for!(&s in &node.succ, preds[s as usize].push(n as NodeId));
        }
        preds
    }

    /// The nodes the start reaches, each after its successors on the
    /// depth-first walk.
    ///
    /// # Returns
    ///
    /// The nodes in postorder.
    pub(crate) fn postorder(&self) -> Vec<NodeId>
    {
        let mut seen = vec![false; self.nodes.len()];
        let mut order = Vec::new();
        let mut stack = vec![(self.start, 0)];
        seen[self.start as usize] = true;
        while let Some((n, i)) = stack.pop()
        {
            let Some(&next) = self.nodes[n as usize].succ.get(i)
            else
            {
                order.push(n);
                continue;
            };
            stack.push((n, i + 1));
            do_if!(
                !seen[next as usize],
                seen[next as usize] = true,
                stack.push((next, 0))
            );
        }
        order
    }
}

#[cfg(test)]
mod tests
{
    use zakac_ir_builder::ir::ExprNode;
    use zakac_ir_builder::ir::Place;
    use zakac_ir_builder::ir::PrintValue;
    use zakac_ir_builder::value::Value;
    use zakac_semcheck::kind::DataKind;

    use super::*;

    /// A print of `text`, which names the statement in a test.
    fn print(text: &str) -> Stmt
    {
        let value = PrintValue::Text(text.to_string());
        Stmt::Print { value, span: Span::default() }
    }

    /// A boolean test.
    fn cond() -> Expr
    {
        let node = ExprNode::Read(Place::Global(0));
        Expr::builder().kind(DataKind::Bool).node(node).span(Span::default()).build()
    }

    /// An `if`.
    fn branch(then: Vec<Stmt>, els: Vec<Stmt>) -> Stmt
    {
        Stmt::If { cond: cond(), then, els, span: Span::default() }
    }

    /// A loop with its init and update lists.
    fn looped(init: Vec<Stmt>, block: Vec<Stmt>, update: Vec<Stmt>) -> Stmt
    {
        Stmt::Loop { init, cond: cond(), update, block, span: Span::default() }
    }

    /// A counted loop.
    fn counted(block: Vec<Stmt>) -> Stmt
    {
        let n = |v| {
            let value =
                Value::Int { kind: DataKind::ISIZE.int_kind().unwrap(), raw: v };
            Expr::builder()
                .kind(DataKind::ISIZE)
                .node(ExprNode::Const(value))
                .span(Span::default())
                .build()
        };
        #[rustfmt::skip]
        return Stmt::Counted {
            var: Place::Global(1), from: n(0), to: n(4), step: None,
            mode: zakac_ast::stmt::CountMode::Count, block, span: Span::default(),
        };
    }

    /// The node of the print of `text`.
    fn at(cfg: &Cfg, text: &str) -> NodeId
    {
        let named = |node: &Node| match node.kind
        {
            NodeKind::Leaf(Stmt::Print { value: PrintValue::Text(t), .. }) => t == text,
            _ => false,
        };
        cfg.nodes.iter().position(named).expect("the print is in the graph") as NodeId
    }

    /// The first node of a kind the filter picks.
    fn find(cfg: &Cfg, pick: impl Fn(&NodeKind) -> bool) -> NodeId
    {
        cfg.nodes.iter().position(|n| pick(&n.kind)).expect("the node is in the graph")
            as NodeId
    }

    /// The successors of a node.
    fn succ(cfg: &Cfg, n: NodeId) -> Vec<NodeId> { cfg.nodes[n as usize].succ.clone() }

    #[test]
    fn if_without_else_joins()
    {
        let block = vec![print("a"), branch(vec![print("b")], Vec::new()), print("c")];
        let cfg = build_cfg(&block, Span::default());
        let test = find(&cfg, |k| matches!(k, NodeKind::Branch(_)));
        let (a, b, c) = (at(&cfg, "a"), at(&cfg, "b"), at(&cfg, "c"));
        let end = find(&cfg, |k| matches!(k, NodeKind::End));
        assert_eq!(succ(&cfg, cfg.start), vec![a]);
        assert_eq!(succ(&cfg, a), vec![test]);
        assert_eq!(succ(&cfg, test), vec![b, c]);
        assert_eq!(succ(&cfg, b), vec![c]);
        assert_eq!(succ(&cfg, c), vec![end]);
        assert_eq!(succ(&cfg, end), vec![cfg.exit]);
        assert_eq!(cfg.preds()[c as usize], vec![test, b]);
        assert_eq!(cfg.rpo.len(), cfg.nodes.len());
    }

    #[test]
    fn else_side_keeps_the_second_edge()
    {
        let block = vec![branch(Vec::new(), vec![print("b")]), print("c")];
        let cfg = build_cfg(&block, Span::default());
        let test = find(&cfg, |k| matches!(k, NodeKind::Branch(_)));
        assert_eq!(succ(&cfg, test), vec![at(&cfg, "c"), at(&cfg, "b")]);
    }

    #[test]
    fn loop_edges_go_init_cond_block_update_cond()
    {
        let block = vec![
            looped(vec![print("init")], vec![print("block")], vec![print("update")]),
            print("after"),
        ];
        let cfg = build_cfg(&block, Span::default());
        let head = find(&cfg, |k| matches!(k, NodeKind::LoopHead(_)));
        let (init, inner) = (at(&cfg, "init"), at(&cfg, "block"));
        let (update, after) = (at(&cfg, "update"), at(&cfg, "after"));
        assert_eq!(succ(&cfg, cfg.start), vec![init]);
        assert_eq!(succ(&cfg, init), vec![head]);
        assert_eq!(succ(&cfg, head), vec![inner, after]);
        assert_eq!(succ(&cfg, inner), vec![update]);
        assert_eq!(succ(&cfg, update), vec![head]);
        assert_eq!(cfg.preds()[head as usize], vec![init, update]);
    }

    #[test]
    fn break_leaves_and_continue_updates()
    {
        let inner = vec![
            branch(vec![Stmt::Break { span: Span::default() }], Vec::new()),
            branch(vec![Stmt::Continue { span: Span::default() }], Vec::new()),
            print("block"),
        ];
        let block =
            vec![looped(Vec::new(), inner, vec![print("update")]), print("after")];
        let cfg = build_cfg(&block, Span::default());
        let head = find(&cfg, |k| matches!(k, NodeKind::LoopHead(_)));
        let tests: Vec<NodeId> = (0..cfg.nodes.len() as NodeId)
            .filter(|&n| matches!(cfg.nodes[n as usize].kind, NodeKind::Branch(_)))
            .collect();
        let (update, after) = (at(&cfg, "update"), at(&cfg, "after"));
        assert_eq!(succ(&cfg, head), vec![tests[0], after]);
        assert_eq!(succ(&cfg, tests[0]), vec![after, tests[1]]);
        assert_eq!(succ(&cfg, tests[1]), vec![update, at(&cfg, "block")]);
        assert_eq!(succ(&cfg, update), vec![head]);
    }

    #[test]
    fn continue_without_update_goes_to_the_test()
    {
        let inner = vec![Stmt::Continue { span: Span::default() }, print("dead")];
        let block = vec![looped(Vec::new(), inner, Vec::new())];
        let cfg = build_cfg(&block, Span::default());
        let head = find(&cfg, |k| matches!(k, NodeKind::LoopHead(_)));
        assert_eq!(succ(&cfg, head)[0], head);
        assert!(!cfg.rpo.contains(&at(&cfg, "dead")));
    }

    #[test]
    fn counted_edges_pass_through_init_and_head()
    {
        let block = vec![print("a"), counted(vec![print("b")]), print("c")];
        let cfg = build_cfg(&block, Span::default());
        let init = find(&cfg, |k| matches!(k, NodeKind::CountedInit(_)));
        let head = find(&cfg, |k| matches!(k, NodeKind::CountedHead(_)));
        let (a, b, c) = (at(&cfg, "a"), at(&cfg, "b"), at(&cfg, "c"));
        assert_eq!(succ(&cfg, a), vec![init]);
        assert_eq!(succ(&cfg, init), vec![head]);
        assert_eq!(succ(&cfg, head), vec![b, c]);
        assert_eq!(succ(&cfg, b), vec![head]);
    }

    #[test]
    fn return_leads_to_exit_not_end()
    {
        let ret = Stmt::Return { value: None, span: Span::default() };
        let block = vec![branch(vec![ret], Vec::new()), print("a")];
        let cfg = build_cfg(&block, Span::default());
        let leaf = find(&cfg, |k| matches!(k, NodeKind::Leaf(Stmt::Return { .. })));
        let end = find(&cfg, |k| matches!(k, NodeKind::End));
        assert_eq!(succ(&cfg, leaf), vec![cfg.exit]);
        assert_eq!(cfg.preds()[end as usize], vec![at(&cfg, "a")]);
        assert_eq!(cfg.preds()[cfg.exit as usize], vec![leaf, end]);
        assert_eq!(cfg.rpo.first(), Some(&cfg.start));
        assert_eq!(cfg.rpo.last(), Some(&cfg.exit));
    }
}
