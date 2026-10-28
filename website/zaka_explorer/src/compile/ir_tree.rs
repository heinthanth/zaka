use bon::Builder;
use zakac_ir_builder::ir::Access;
use zakac_ir_builder::ir::Dest;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::Func;
use zakac_ir_builder::ir::Module;
use zakac_ir_builder::ir::Place;
use zakac_ir_builder::ir::PrintValue;
use zakac_ir_builder::ir::Slot;
use zakac_ir_builder::ir::Stmt;
use zakac_rust_sugars::tern;
use zakac_span::source_file::SourceFile;
use zakac_span::span::Span;

use crate::compile::spans::to_original;
use crate::compile::tree::TreeNode;

/// Build the tree view of an IR module.
///
/// # Arguments
///
/// * `file` - The source file of the module.
/// * `original` - The original text of the file.
/// * `module` - The module.
///
/// # Returns
///
/// The `module` node.
pub fn build_ir_tree(file: &SourceFile, original: &str, module: &Module) -> TreeNode
{
    let builder =
        IrTree::builder().file(file).original(original).module(module).build();
    builder.module()
}

/// Turns an IR module into [`TreeNode`]s.
#[derive(Clone, Copy, Builder)]
struct IrTree<'a>
{
    /// The source file.
    file: &'a SourceFile,
    /// The original text.
    original: &'a str,
    /// The module, for slot names.
    module: &'a Module,
    /// The function whose locals the places name, if any.
    func: Option<&'a Func>,
}

impl IrTree<'_>
{
    /// The same builder, naming the locals of `f`.
    ///
    /// # Arguments
    ///
    /// * `f` - The function.
    ///
    /// # Returns
    ///
    /// The builder.
    fn in_func<'f>(&self, f: &'f Func) -> IrTree<'f>
    where
        Self: 'f,
    {
        let mut inner: IrTree<'f> = *self;
        inner.func = Some(f);
        inner
    }

    /// A node with no kind.
    ///
    /// # Arguments
    ///
    /// * `label` - The node form.
    /// * `detail` - The text next to the label.
    /// * `span` - The source region.
    /// * `children` - The child nodes.
    ///
    /// # Returns
    ///
    /// The node.
    fn node(
        &self, label: &'static str, detail: String, span: Span, children: Vec<TreeNode>,
    ) -> TreeNode
    {
        let range = to_original(self.file, self.original, &span);
        #[rustfmt::skip]
        return TreeNode::builder().label(label).detail(detail).range(range)
            .children(children).build();
    }

    /// The `module` node: globals, functions, then the main statements.
    ///
    /// # Returns
    ///
    /// The node.
    fn module(&self) -> TreeNode
    {
        let m = self.module;
        let globals = self.slots("globals", &m.globals, Place::Global, 0, m.span);
        let funcs = m.funcs.iter().map(|f| self.in_func(f).func(f));
        let main = self.stmts("main", &m.main, m.span);
        let children = std::iter::once(globals).chain(funcs).chain([main]).collect();
        self.node("module", String::new(), m.span, children)
    }

    /// A list of slots.
    ///
    /// # Arguments
    ///
    /// * `label` - The list label.
    /// * `slots` - The slots.
    /// * `place` - Makes a place from a slot index.
    /// * `first` - The index of the first slot.
    /// * `span` - The source region of the list.
    ///
    /// # Returns
    ///
    /// The node.
    fn slots(
        &self, label: &'static str, slots: &[Slot], place: fn(u32) -> Place,
        first: usize, span: Span,
    ) -> TreeNode
    {
        let slot = |(i, s): (usize, &Slot)| {
            let access: &'static str = s.access.into();
            let detail = format!("{} {}", place((first + i) as u32), s.name);
            let detail =
                tern!(s.access == Access::Var, detail, format!("{detail} {access}"));
            let mut node = self.node("slot", detail, s.span, Vec::new());
            node.kind = Some(s.kind.name());
            node
        };
        let children = slots.iter().enumerate().map(slot).collect();
        self.node(label, String::new(), span, children)
    }

    /// The node of a function: parameters, other locals and statements.
    ///
    /// # Arguments
    ///
    /// * `f` - The function.
    ///
    /// # Returns
    ///
    /// The node.
    fn func(&self, f: &Func) -> TreeNode
    {
        let params = f.params as usize;
        let children = vec![
            self.slots("params", &f.locals[..params], Place::Local, 0, f.span),
            self.slots("locals", &f.locals[params..], Place::Local, params, f.span),
            self.stmts("block", &f.block, f.span),
        ];
        let mut node = self.node("fn", f.name.clone(), f.span, children);
        node.kind = f.ret.map(|k| k.name());
        node
    }

    /// A statement list.
    ///
    /// # Arguments
    ///
    /// * `label` - The list label.
    /// * `stmts` - The statements.
    /// * `span` - Where the list is when it holds no statement.
    ///
    /// # Returns
    ///
    /// The node.
    fn stmts(&self, label: &'static str, stmts: &[Stmt], span: Span) -> TreeNode
    {
        let lo = stmts.first().map_or(span, Stmt::span);
        let hi = stmts.last().map_or(span, Stmt::span);
        let children = stmts.iter().map(|s| self.stmt(s)).collect();
        self.node(label, String::new(), lo.to(&hi), children)
    }

    /// The text that names a slot: its place and name.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    ///
    /// # Returns
    ///
    /// The text, such as `g0 က`.
    fn place_text(&self, place: Place) -> String
    {
        let slot = match place
        {
            Place::Global(n) => self.module.globals.get(n as usize),
            Place::Local(n) => self.func.and_then(|f| f.locals.get(n as usize)),
        };
        slot.map_or(place.to_string(), |s| format!("{place} {}", s.name))
    }

    /// The node of a statement.
    ///
    /// # Arguments
    ///
    /// * `s` - The statement.
    ///
    /// # Returns
    ///
    /// The node.
    fn stmt(&self, s: &Stmt) -> TreeNode
    {
        let span = s.span();
        let blank = String::new();
        match s
        {
            Stmt::Set { place, value, .. } =>
            {
                self.node("set", self.place_text(*place), span, vec![self.expr(value)])
            }
            Stmt::Print { value, .. } =>
            {
                self.node("print", blank, span, vec![self.print(value, span)])
            }
            Stmt::If { cond, then, els, .. } =>
            {
                let mut children =
                    vec![self.expr(cond), self.stmts("then", then, span)];
                children
                    .extend((!els.is_empty()).then(|| self.stmts("else", els, span)));
                self.node("if", blank, span, children)
            }
            Stmt::Loop { init, cond, update, block, .. } =>
            {
                let init = (!init.is_empty()).then(|| self.stmts("init", init, span));
                let update =
                    (!update.is_empty()).then(|| self.stmts("update", update, span));
                let children = init
                    .into_iter()
                    .chain([self.expr(cond)])
                    .chain(update)
                    .chain([self.stmts("block", block, span)])
                    .collect();
                self.node("loop", blank, span, children)
            }
            Stmt::Counted { var, from, to, step, mode, block, .. } =>
            {
                let detail = format!("{} {}", self.place_text(*var), mode.as_ref());
                let bounds = [
                    Some(self.expr(from)),
                    Some(self.expr(to)),
                    step.as_ref().map(|e| self.expr(e)),
                ];
                let children = bounds
                    .into_iter()
                    .flatten()
                    .chain([self.stmts("block", block, span)])
                    .collect();
                self.node("count", detail, span, children)
            }
            Stmt::Break { .. } => self.node("break", blank, span, Vec::new()),
            Stmt::Continue { .. } => self.node("continue", blank, span, Vec::new()),
            Stmt::Return { value, .. } =>
            {
                let children = value.iter().map(|v| self.expr(v)).collect();
                self.node("return", blank, span, children)
            }
            Stmt::Call { name, args, .. } =>
            {
                let children = args.iter().map(|a| self.expr(a)).collect();
                self.node("call", name.clone(), span, children)
            }
            Stmt::Store { addr, value, .. } =>
            {
                self.node("store", blank, span, self.pair(addr, value))
            }
            Stmt::Alloc { dest, count, unit, .. } =>
            {
                let unit = unit.map(|k| k.name());
                self.heap("alloc", dest, Some(count), unit, span)
            }
            Stmt::Resize { dest, count, unit, .. } =>
            {
                let unit = unit.map(|k| k.name());
                self.heap("resize", dest, Some(count), unit, span)
            }
            Stmt::Free { dest, .. } => self.heap("free", dest, None, None, span),
        }
    }

    /// The node of a heap statement: the slot text and the unit kind as
    /// detail, then the address and the count as children.
    ///
    /// # Arguments
    ///
    /// * `label` - The statement form.
    /// * `dest` - Where the address lives.
    /// * `count` - The element count, if the statement has one.
    /// * `unit` - The name of the element kind, when written.
    /// * `span` - The statement.
    ///
    /// # Returns
    ///
    /// The node.
    fn heap(
        &self, label: &'static str, dest: &Dest, count: Option<&Expr>,
        unit: Option<String>, span: Span,
    ) -> TreeNode
    {
        let (slot, addr) = match dest
        {
            Dest::Slot(place) => (Some(self.place_text(*place)), None),
            Dest::Mem(e) => (None, Some(self.expr(e))),
        };
        let words: Vec<String> = slot.into_iter().chain(unit).collect();
        let children = addr.into_iter().chain(count.map(|c| self.expr(c))).collect();
        self.node(label, words.join(" "), span, children)
    }

    /// The node of what a print statement writes.
    ///
    /// # Arguments
    ///
    /// * `value` - The value.
    /// * `span` - The statement.
    ///
    /// # Returns
    ///
    /// The node.
    fn print(&self, value: &PrintValue, span: Span) -> TreeNode
    {
        match value
        {
            PrintValue::Expr(e) => self.expr(e),
            PrintValue::Text(text) =>
            {
                self.node("str", format!("\"{text}\""), span, Vec::new())
            }
        }
    }

    /// The node of an expression, with its kind.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    ///
    /// # Returns
    ///
    /// The node.
    fn expr(&self, e: &Expr) -> TreeNode
    {
        let (label, detail, children): (&'static str, String, Vec<TreeNode>) = match &e
            .node
        {
            ExprNode::Const(v) => ("const", v.to_string(), Vec::new()),
            ExprNode::Read(place) => ("read", self.place_text(*place), Vec::new()),
            ExprNode::Prefix { op, operand } =>
            {
                (op.mnemonic(), String::new(), vec![self.expr(operand)])
            }
            ExprNode::Infix { op, lhs, rhs } =>
            {
                (op.mnemonic(), String::new(), self.pair(lhs, rhs))
            }
            ExprNode::Compare { op, lhs, rhs } =>
            {
                (op.mnemonic(), String::new(), self.pair(lhs, rhs))
            }
            ExprNode::Logic { op, lhs, rhs } =>
            {
                (op.into(), String::new(), self.pair(lhs, rhs))
            }
            ExprNode::Convert { operand, implicit } => (
                tern!(*implicit, "widen", "cast"),
                String::new(),
                vec![self.expr(operand)],
            ),
            ExprNode::Call { name, args, .. } =>
            {
                ("call", name.clone(), args.iter().map(|a| self.expr(a)).collect())
            }
            ExprNode::AddrOf(place) => ("addr", self.place_text(*place), Vec::new()),
            ExprNode::Load { addr } => ("load", String::new(), vec![self.expr(addr)]),
            ExprNode::PtrAdd { base, offset } =>
            {
                ("ptr+", String::new(), self.pair(base, offset))
            }
            ExprNode::PtrSub { lhs, rhs } =>
            {
                ("ptr-", String::new(), self.pair(lhs, rhs))
            }
            ExprNode::SizeOf(k) => ("sizeof", k.name(), Vec::new()),
            ExprNode::List(cells) =>
            {
                ("list", String::new(), cells.iter().map(|c| self.expr(c)).collect())
            }
            ExprNode::BlankList => ("blank_list", String::new(), Vec::new()),
            ExprNode::CellAddr { list, index, len, first, .. } =>
            {
                let label = tern!(*first == 0, "cell@", "cell1@");
                (label, len.to_string(), self.pair(list, index))
            }
        };
        let mut node = self.node(label, detail, e.span, children);
        node.kind = Some(e.kind.name());
        node
    }

    /// The nodes of two operands.
    ///
    /// # Arguments
    ///
    /// * `lhs` - The left operand.
    /// * `rhs` - The right operand.
    ///
    /// # Returns
    ///
    /// The two nodes.
    fn pair(&self, lhs: &Expr, rhs: &Expr) -> Vec<TreeNode>
    {
        vec![self.expr(lhs), self.expr(rhs)]
    }
}
