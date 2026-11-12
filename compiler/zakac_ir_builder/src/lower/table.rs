//! The lowering of ဇယား values: literals, the rows a statement reads, writes
//! through, sets and removes, the count of rows, the loops over the rows, and
//! the fallback of the read of an optional.
//!
//! A ဇယား is a struct of its rows (a plain list, in add order), its slots and
//! its count of live rows; a row is a struct of the hash of its index, a live
//! mark, the index and the value. A read finds the number of the row first,
//! then reads the row through the plain list, as a cell read does; a write
//! through a row makes the list own its block first, as a cell write does.
//! Setting and removing a row, which move rows and slots, are statements of
//! their own.

use zakac_ast::stmt::EachPart;

use super::*;

/// The position of the rows in the struct of a ဇယား.
const ROWS: usize = 0;
/// The position of the count of live rows in the struct of a ဇယား.
const LIVE_COUNT: usize = 2;
/// The position of the live mark in the struct of a row.
const ROW_LIVE: usize = 1;
/// The position of the index in the struct of a row.
const ROW_INDEX: usize = 2;
/// The position of the value in the struct of a row.
const ROW_VALUE: usize = 3;

impl Lowerer<'_>
{
    /// The address of one field of a struct value.
    ///
    /// # Arguments
    ///
    /// * `base` - The struct.
    /// * `at` - The position of the field.
    /// * `span` - The source region.
    ///
    /// # Returns
    ///
    /// The address, a pointer to the field kind.
    fn part_addr(&self, base: Expr, at: usize, span: Span) -> Expr
    {
        let shape = base.kind.as_struct().expect("a ဇယား and its rows are structs");
        let kind = shape.fields()[at].1.repr();
        let offset = shape.offset(at, self.platform);
        let field = u32::try_from(at).expect("a struct has fewer than 2^32 fields");
        let node = ExprNode::FieldAddr { base: Box::new(base), field, offset };
        node_expr(kind.pointer_to(), node, span)
    }

    /// The read of one field of a struct value.
    ///
    /// # Arguments
    ///
    /// * `base` - The struct.
    /// * `at` - The position of the field.
    /// * `span` - The source region.
    ///
    /// # Returns
    ///
    /// The read, of the field kind.
    fn part_of(&self, base: Expr, at: usize, span: Span) -> Expr
    {
        let addr = self.part_addr(base, at, span);
        let kind = addr.kind.pointee().expect("a field address points at the field");
        node_expr(kind, ExprNode::Load { addr: Box::new(addr) }, span)
    }

    /// The read of the row of a ဇယား at a number.
    ///
    /// # Arguments
    ///
    /// * `table` - The ဇယား, read where it is.
    /// * `pos` - The number of the row, which a find gave.
    /// * `span` - The source region.
    ///
    /// # Returns
    ///
    /// The read, of the row kind.
    fn row_at(&self, table: Expr, pos: Expr, span: Span) -> Expr
    {
        let rows = self.part_of(table, ROWS, span);
        let row = rows.kind.cell_kind().expect("the rows of a ဇယား are a plain list");
        #[rustfmt::skip]
        let node = ExprNode::CellAddr { list: Box::new(rows), index: Box::new(pos), len: None, first: 0, checked: true };
        let addr = node_expr(row.pointer_to(), node, span);
        node_expr(row, ExprNode::Load { addr: Box::new(addr) }, span)
    }

    /// An `isize` constant.
    ///
    /// # Arguments
    ///
    /// * `n` - The value.
    /// * `span` - Its source region.
    ///
    /// # Returns
    ///
    /// The constant.
    fn isize_const(&self, n: i64, span: Span) -> Expr
    {
        let isize = DataKind::ISIZE.int_kind().expect("isize is an integer");
        let value = Value::int(isize, n as u128, self.platform);
        node_expr(DataKind::ISIZE, ExprNode::Const(value), span)
    }

    /// Find the number of the row of a ဇယား with an index, in a hidden slot
    /// that [`Self::pre`] sets.
    ///
    /// # Arguments
    ///
    /// * `table` - The ဇယား, read where it is.
    /// * `index` - The index.
    /// * `span` - The source region.
    ///
    /// # Returns
    ///
    /// The read of the slot: the number, or -1 when no row has the index.
    fn find_row(&mut self, table: Expr, index: &ast::Expr, span: Span) -> Expr
    {
        let (at, _) = table.kind.table_parts().expect("the check found a ဇယား");
        let index = Box::new(self.coerce(index, at));
        let node = ExprNode::RowFind { table: Box::new(table), index };
        let found = node_expr(DataKind::ISIZE, node, span);
        let pos = self.hidden(DataKind::ISIZE, span);
        self.pre.push(Stmt::Set { place: pos, value: found, span });
        read(pos, DataKind::ISIZE, span)
    }

    /// A ဇယား read for a find and a row read after it: a place is read where
    /// it is, each other value waits in a hidden slot.
    ///
    /// # Arguments
    ///
    /// * `table` - The ဇယား.
    ///
    /// # Returns
    ///
    /// The value.
    fn table_value(&mut self, table: &ast::Expr) -> Expr
    {
        let value = self.expr(table);
        let place = matches!(value.node, ExprNode::Read(_) | ExprNode::Load { .. });
        tern!(place, value, self.spill(value))
    }

    /// Lower `<ဇယား> ထဲမှ အညွှန်း <index> ၏ တန်ဖိုး`: as a read, an optional of a
    /// duplicate of the value; as the place a statement writes through, the
    /// value itself, read where it is.
    ///
    /// # Arguments
    ///
    /// * `n` - The read.
    /// * `kind` - Its recorded kind: the optional for a read, the value kind
    ///   for a place.
    ///
    /// # Returns
    ///
    /// The operation.
    pub(super) fn row_value(&mut self, n: &ast::VerbExpr, kind: DataKind) -> ExprNode
    {
        let [table, index] = n.args.as_slice()
        else
        {
            panic!("the read of a row has a ဇယား and an index");
        };
        let (_, data) =
            self.kind_of(table).table_parts().expect("the check found a ဇယား");
        ret_if!(
            kind == data.repr(),
            ExprNode::Load { addr: Box::new(self.row_addr(n)) }
        );
        let span = n.span;
        let table = self.table_value(table);
        let pos = self.find_row(table.clone(), index, span);
        let fault = Fault::of_kind(kind).expect("the read of a row is an optional");
        let held = self.hidden(kind, span);
        if kind.holds_text()
        {
            self.pre_init.push(Stmt::Set {
                place: held,
                value: default_constant(kind, span),
                span,
            });
            self.drops.last_mut().expect("a scope is open").push((held, kind));
        }
        let value =
            self.part_of(self.row_at(table, pos.clone(), span), ROW_VALUE, span);
        let value = tern!(value.kind.holds_text(), dup(value), value);
        let some = fault.value(false, Some(value), None, self.platform, span);
        let none = fault.value(true, None, None, self.platform, span);
        let found = compare_node(CmpOp::Ge, pos, self.isize_const(0, span));
        self.pre.push(Stmt::If {
            cond: node_expr(DataKind::Bool, found, span),
            then: vec![Stmt::Set { place: held, value: some, span }],
            els: vec![Stmt::Set { place: held, value: none, span }],
            span,
        });
        ExprNode::Read(held)
    }

    /// The address of the value of a row that a statement writes through.
    /// A ဇယား with no row of the index stops the run.
    ///
    /// # Arguments
    ///
    /// * `n` - `<ဇယား> ထဲမှ အညွှန်း <index> ၏ တန်ဖိုး`.
    ///
    /// # Returns
    ///
    /// The address, a pointer to the value kind.
    pub(super) fn row_addr(&mut self, n: &ast::VerbExpr) -> Expr
    {
        let [table, index] = n.args.as_slice()
        else
        {
            panic!("the read of a row has a ဇယား and an index");
        };
        let span = n.span;
        let table = self.expr(table);
        let pos = self.find_row(table.clone(), index, span);
        let missing = compare_node(CmpOp::Lt, pos.clone(), self.isize_const(0, span));
        let halt = Stmt::Halt { cause: HaltCause::NoRow, value: None, span };
        self.pre.push(Stmt::If {
            cond: node_expr(DataKind::Bool, missing, span),
            then: vec![halt],
            els: Vec::new(),
            span,
        });
        self.part_addr(self.row_at(table, pos, span), ROW_VALUE, span)
    }

    /// Lower `<ဇယား> ၏ အညွှန်းအရေအတွက်`: its count of live rows.
    ///
    /// # Arguments
    ///
    /// * `table` - The ဇယား.
    ///
    /// # Returns
    ///
    /// The read of the count, a `usize`.
    pub(super) fn row_count(&mut self, table: &ast::Expr) -> ExprNode
    {
        let table = self.expr(table);
        let span = table.span;
        self.part_of(table, LIVE_COUNT, span).node
    }

    /// Lower `<optional> ထဲမှ ဒေတာ၊ မရှိလျှင် <value>`: the optional waits in a
    /// hidden slot, and a test moves its data, or puts the value, in a second
    /// one. The value runs when the optional holds no data, and at no other
    /// time.
    ///
    /// # Arguments
    ///
    /// * `n` - The read with its fallback.
    /// * `kind` - The data kind.
    ///
    /// # Returns
    ///
    /// The read of the second slot.
    pub(super) fn or_else(&mut self, n: &ast::VerbExpr, kind: DataKind) -> ExprNode
    {
        let [optional, fallback] = n.args.as_slice()
        else
        {
            panic!("a fallback has an optional and a value");
        };
        let span = n.span;
        let result = self.expr(optional);
        let fault = Fault::of_kind(result.kind).expect("the check found an optional");
        let held = self.hidden(fault.wrapper, span);
        if fault.wrapper.holds_text()
        {
            let blank = fault.value(true, None, None, self.platform, span);
            self.pre_init.push(Stmt::Set { place: held, value: blank, span });
            self.drops.last_mut().expect("a scope is open").push((held, fault.wrapper));
        }
        let read_place =
            matches!(result.node, ExprNode::Read(_) | ExprNode::Load { .. });
        let result = tern!(read_place && result.kind.holds_text(), dup(result), result);
        self.pre.push(Stmt::Set { place: held, value: result, span });
        let out = self.hidden(kind, span);
        if kind.holds_text()
        {
            self.pre_init.push(Stmt::Set {
                place: out,
                value: default_constant(kind, span),
                span,
            });
            self.drops.last_mut().expect("a scope is open").push((out, kind));
        }
        let (mut then, value) = self.with_pre(|l| l.owned(fallback, kind));
        then.push(Stmt::Set { place: out, value, span });
        let data = take(fault.field(held, "ok", self.platform, span));
        self.pre.push(Stmt::If {
            cond: fault.failed(held, self.platform, span),
            then,
            els: vec![Stmt::Set { place: out, value: data, span }],
            span,
        });
        ExprNode::Read(out)
    }

    /// Lower a ဇယား literal: a hidden ဇယား starts blank, and each row is set
    /// in it in order, so a later row with an index of an earlier one gives
    /// it its value.
    ///
    /// # Arguments
    ///
    /// * `n` - The literal.
    /// * `kind` - The ဇယား kind.
    ///
    /// # Returns
    ///
    /// The read of the hidden ဇယား.
    pub(super) fn table_lit(&mut self, n: &ast::TableLit, kind: DataKind) -> ExprNode
    {
        let span = n.span;
        let (at, data) = kind.table_parts().expect("the check found a ဇယား");
        let held = self.hidden(kind, span);
        self.pre_init.push(Stmt::Set {
            place: held,
            value: default_constant(kind, span),
            span,
        });
        self.drops.last_mut().expect("a scope is open").push((held, kind));
        for row in &n.rows
        {
            let index = self.owned(&row.index, at);
            let value = self.owned(&row.value, data);
            self.pre.push(Stmt::SetRow {
                dest: Dest::Slot(held),
                kind,
                index,
                value,
                span,
            });
        }
        ExprNode::Read(held)
    }

    /// Lower `<ဇယား> ထဲမှ အညွှန်း <index> ၏ တန်ဖိုး ကို <value> ဟု ထားမည်။`.
    ///
    /// # Arguments
    ///
    /// * `row` - The row read.
    /// * `value` - The value.
    /// * `span` - The statement.
    ///
    /// # Returns
    ///
    /// The statement.
    pub(super) fn set_row(
        &mut self, row: &ast::VerbExpr, value: &ast::Expr, span: Span,
    ) -> Stmt
    {
        let [table, index] = row.args.as_slice()
        else
        {
            panic!("the read of a row has a ဇယား and an index");
        };
        let kind = self.kind_of(table);
        let (at, data) = kind.table_parts().expect("the check found a ဇယား");
        let index = self.owned(index, at);
        let value = self.owned(value, data);
        let dest = self.dest(table);
        Stmt::SetRow { dest, kind, index, value, span }
    }

    /// Lower `<ဇယား> ထဲမှ အညွှန်း <index> ကို ဖျက်မည်။`.
    ///
    /// # Arguments
    ///
    /// * `n` - The statement.
    ///
    /// # Returns
    ///
    /// The statement.
    pub(super) fn remove_row(&mut self, n: &st::RemoveStmt) -> Stmt
    {
        extract!(
            ast::Expr::Verb(row) = ungrouped(&n.list),
            "a remove of a row names a row"
        );
        let [table, index] = row.args.as_slice()
        else
        {
            panic!("a row has a ဇယား and an index");
        };
        let kind = self.kind_of(table);
        let (at, _) = kind.table_parts().expect("the check found a ဇယား");
        let index = self.coerce(index, at);
        let dest = self.dest(table);
        Stmt::RemoveRow { dest, kind, index, span: n.span }
    }

    /// Lower an each loop over the rows of a ဇယား into a walk over a
    /// duplicate of it: a hidden number runs over its rows, and a round skips
    /// a removed row.
    ///
    /// # Arguments
    ///
    /// * `n` - The loop.
    /// * `out` - The list the statements go to.
    pub(super) fn each_rows(&mut self, n: &st::EachStmt, out: &mut Vec<Stmt>)
    {
        let kind = self.kind_of(&n.source);
        let span = n.source.span();
        let source = self.owned(&n.source, kind);
        let mut parts = None;
        self.scoped(|l| {
            let name = ast::Ident::builder().name("%each").span(span).build();
            let each = l.declare(&name, kind, Access::Var);
            let at = l.hidden(DataKind::USIZE, span);
            let zero = l.usize_const(0, span);
            let init = vec![
                Stmt::Set { place: each, value: source, span },
                Stmt::Set { place: at, value: zero, span },
            ];
            let rows = l.part_of(read(each, kind, span), ROWS, span);
            let count = node_expr(DataKind::USIZE, l.count_word(rows, span), span);
            let at_read = read(at, DataKind::USIZE, span);
            let cond = node_expr(
                DataKind::Bool,
                compare_node(CmpOp::Lt, at_read.clone(), count),
                span,
            );
            let block = l.row_round(n, each, at);
            let one = l.usize_const(1, span);
            let next = node_expr(
                DataKind::USIZE,
                infix_node(InfixOp::Add, at_read, one),
                span,
            );
            let update = vec![Stmt::Set { place: at, value: next, span }];
            let after = l.drops_from(l.drops.len() - 1, n.span);
            parts = Some((init, cond, block, update, after));
        });
        let (init, cond, block, update, after) = parts.expect("the scope ran");
        out.push(Stmt::Loop { init, cond, update, block, span: n.span });
        out.extend(after);
    }

    /// Lower one round of an each loop over rows in its own scope: skip a
    /// removed row, set the names, then run the block.
    ///
    /// # Arguments
    ///
    /// * `n` - The loop.
    /// * `each` - The slot of the duplicate of the ဇယား.
    /// * `at` - The slot of the number of the row.
    ///
    /// # Returns
    ///
    /// The round.
    fn row_round(&mut self, n: &st::EachStmt, each: Place, at: Place) -> Vec<Stmt>
    {
        let kind = self.slot_kind(each);
        let span = n.var.span;
        let mut block = Vec::new();
        self.loops.push(self.scopes.len());
        self.scoped(|l| {
            let row = l.row_at(read(each, kind, span), read(at, DataKind::USIZE, span), span);
            let dead = prefix_node(PrefixOp::Not, l.part_of(row.clone(), ROW_LIVE, span));
            let skip = vec![Stmt::Continue { span }];
            #[rustfmt::skip]
            block.push(Stmt::If { cond: node_expr(DataKind::Bool, dead, span), then: skip, els: Vec::new(), span });
            let index = l.part_of(row.clone(), ROW_INDEX, span);
            let value = l.part_of(row, ROW_VALUE, span);
            let (first, last) = match (&n.place, n.part)
            {
                (Some(place), _) => (Some((place, index)), value),
                (None, EachPart::Index) => (None, index),
                (None, _) => (None, value),
            };
            do_if!(let Some((name, e)) = first, l.bind(name, e, &mut block));
            l.bind(&n.var, last, &mut block);
            do_for!(s in &n.block.stmts, l.stmt(s, &mut block));
            let leaves = block.last().is_some_and(leaves_scope);
            let end = l.drops_from(l.drops.len() - 1, n.block.span);
            do_if!(!leaves, block.extend(end));
        });
        self.loops.pop();
        block
    }

    /// Declare a name of an each loop and set it to a duplicate of a value.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    /// * `value` - The value, read where it is.
    /// * `block` - The round, which gets the statements.
    pub(super) fn bind(&mut self, name: &ast::Ident, value: Expr, block: &mut Vec<Stmt>)
    {
        let kind = value.kind;
        let value = tern!(kind.holds_text(), dup(value), value);
        let var = self.declare(name, kind, Access::Var);
        block.push(Stmt::Set { place: var, value, span: name.span });
        do_if!(self.is_boxed_name(&name.name), self.rebox(name, var, kind, block));
    }
}

/// The read of a row a statement writes through or sets, when the place is
/// one.
///
/// # Arguments
///
/// * `e` - The place.
///
/// # Returns
///
/// The read, or `None` for each other place.
pub(super) fn row_place(e: &ast::Expr) -> Option<&ast::VerbExpr>
{
    let ast::Expr::Verb(v) = ungrouped(e)
    else
    {
        return None;
    };
    (v.op == VerbOp::RowValue).then_some(v)
}
