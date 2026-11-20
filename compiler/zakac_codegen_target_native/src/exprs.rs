//! The lowering of expressions, as the C target writes them. Each operand runs
//! in order, so no operand needs a temp to keep the order of its run.

use cranelift_codegen::ir::InstBuilder;
use cranelift_codegen::ir::Value;
use cranelift_codegen::ir::condcodes::FloatCC;
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::immediates::Ieee32;
use cranelift_codegen::ir::immediates::Ieee64;
use cranelift_codegen::ir::types;
use cranelift_module::Module;
use zakac_codegen_target_c::kinds::is_blob;
use zakac_ir_builder::fold::is_known;
use zakac_ir_builder::ir::CmpOp;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::FmtPiece;
use zakac_ir_builder::ir::HostFn;
use zakac_ir_builder::ir::LogicOp;
use zakac_ir_builder::ir::SliceFrom;
use zakac_ir_builder::ir::fn_shape;
use zakac_ir_builder::ir::fresh_text;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::foreign::is_c_text_result;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::value::Value as Const;

use crate::func::FnGen;
use crate::stmts::Callee;
use crate::stmts::node_name;

impl<M: Module> FnGen<'_, '_, M>
{
    /// Lower an expression: a scalar, or the address of a list, a struct or
    /// text.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    ///
    /// # Returns
    ///
    /// The value.
    pub fn expr(&mut self, e: &Expr) -> Value
    {
        let ptr = self.s.ptr;
        match &e.node
        {
            ExprNode::Const(v) => self.constant(*v, e.kind),
            ExprNode::Read(p) => self.read(*p),
            ExprNode::AddrOf(p) => self.addr_of(*p),
            ExprNode::Prefix { op, operand } =>
            {
                let a = self.expr(operand);
                self.prefix(*op, a, e.kind)
            }
            ExprNode::Infix { op, lhs, rhs } =>
            {
                let a = self.expr(lhs);
                let b = self.expr(rhs);
                self.infix(*op, a, b, e.kind, rhs.kind)
            }
            ExprNode::Compare { op, lhs, rhs } => self.compare(*op, lhs, rhs),
            ExprNode::Logic { op, lhs, rhs } => self.logic(*op, lhs, rhs),
            ExprNode::Convert { operand, .. } => self.convert(operand, e.kind),
            ExprNode::Call { func, args, .. } => self.call(Callee::Fn(*func), args),
            ExprNode::Spawn { func, args, .. } => self.spawn(*func, args),
            ExprNode::Wait(job) => self.wait(job, e.kind),
            ExprNode::MakePipe(room) =>
            {
                let ptr = self.s.ptr;
                let room = self.word(i64::from(room.unwrap_or(0)));
                self.rt("zk_pipe_new", &[ptr], &[ptr], &[room]).unwrap_or(room)
            }
            ExprNode::RingStat(n) =>
            {
                let n = self.b.ins().iconst(types::I32, i64::from(*n));
                let v = self
                    .rt("zk_ring_stat", &[types::I32], &[types::I64], &[n])
                    .unwrap_or(n);
                match e.kind == DataKind::Bool
                {
                    true => self.b.ins().icmp_imm_s(IntCC::NotEqual, v, 0),
                    false => v,
                }
            }
            ExprNode::Receive(pipe) => self.receive(pipe, e.kind),
            ExprNode::CallValue { callee, args } =>
            {
                self.call(Callee::Value(callee), args)
            }
            ExprNode::FnAddr { func, .. } =>
            {
                let id = self.s.records[*func as usize];
                self.data(id)
            }
            ExprNode::Closure { func, record, .. } =>
            {
                let kind = record.kind.pointee().expect("a record address");
                let k = self.descr(kind);
                let r = self.expr(record);
                let thunk = self.func_addr(*func);
                self.rt("zk_closure", &[ptr, ptr, ptr], &[ptr], &[r, thunk, k])
                    .unwrap_or(r)
            }
            ExprNode::Lend { operand } => self.expr(operand),
            ExprNode::FieldAddr { base, offset, .. } =>
            {
                let b = self.expr(base);
                self.b.ins().iadd_imm_s(b, i64::from(*offset))
            }
            ExprNode::SizeOf(k) =>
            {
                let t = self.s.clif_of(e.kind);
                self.b.ins().iconst(t, i64::from(self.s.size(*k)))
            }
            ExprNode::CField { base, offset } => self.c_field(base, *offset, e.kind),
            _ => self.data_expr(e),
        }
    }

    /// The address of the code of a function of the program.
    ///
    /// # Arguments
    ///
    /// * `n` - The index of the function.
    ///
    /// # Returns
    ///
    /// The address.
    fn func_addr(&mut self, n: u32) -> Value
    {
        let id = self.s.funcs[n as usize];
        let r = self.func_ref(id);
        let ptr = self.s.ptr;
        self.b.ins().func_addr(ptr, r)
    }

    /// A constant.
    ///
    /// # Arguments
    ///
    /// * `v` - The constant.
    /// * `kind` - The kind of the expression that holds it.
    ///
    /// # Returns
    ///
    /// The value.
    fn constant(&mut self, v: Const, kind: DataKind) -> Value
    {
        let t = self.s.clif_of(kind);
        match v
        {
            Const::Int { raw, .. } =>
            {
                let mask = u64::MAX >> (64 - t.bits().min(64));
                self.b.ins().iconst(t, (raw as u64 & mask) as i64)
            }
            Const::Float { bits: 32, raw } =>
            {
                self.b.ins().f32const(Ieee32::with_bits(raw as u32))
            }
            Const::Float { raw, .. } => self.b.ins().f64const(Ieee64::with_bits(raw)),
            Const::Char(c) => self.b.ins().iconst(t, i64::from(c as u32)),
            Const::Bool(b) => self.b.ins().iconst(t, i64::from(b)),
            Const::Ptr(0) => self.b.ins().iconst(t, 0),
            // A function value folds to the index of its function plus one, as
            // the VM holds it; native code holds the address of its record.
            Const::Ptr(a) if kind.is_fn() =>
            {
                let id = self.s.records[a as usize - 1];
                self.data(id)
            }
            Const::Ptr(a) => self.b.ins().iconst(t, a as i64),
        }
    }

    /// A field of a struct C keeps, copied out: a number, a boolean or a
    /// counted list as it is, and for text the string the field's `char *`
    /// points at, or the optional with no data for NULL.
    ///
    /// # Arguments
    ///
    /// * `base` - The address of the C struct.
    /// * `offset` - The offset of the field.
    /// * `kind` - The kind of the field.
    ///
    /// # Returns
    ///
    /// The value.
    fn c_field(&mut self, base: &Expr, offset: u32, kind: DataKind) -> Value
    {
        let ptr = self.s.ptr;
        let b = self.expr(base);
        let at = self.b.ins().iadd_imm_s(b, i64::from(offset));
        if is_c_text_result(kind)
        {
            let platform = self.s.platform();
            let ok_at = kind.as_struct().map_or(0, |s| s.offset(1, platform));
            let t = self.temp(kind);
            let s = self.load(ptr, at);
            let (size, ok) =
                (self.word(i64::from(self.s.size(kind))), self.word(i64::from(ok_at)));
            self.rt("zk_text_from_c", &[ptr, ptr, ptr, ptr], &[], &[t, size, ok, s]);
            return t;
        }
        if is_blob(kind)
        {
            let t = self.temp(kind);
            let size = self.word(i64::from(self.s.size(kind)));
            self.rt("zk_put_block", &[ptr, ptr, ptr], &[], &[t, at, size]);
            return t;
        }
        let t = self.s.clif_of(kind);
        self.load(t, at)
    }

    /// Lower an expression that reads or makes data: a load, a move, a list,
    /// a slice or text.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    ///
    /// # Returns
    ///
    /// The value.
    fn data_expr(&mut self, e: &Expr) -> Value
    {
        let ptr = self.s.ptr;
        match &e.node
        {
            ExprNode::Load { addr } =>
            {
                let a = self.expr(addr);
                ret_if!(is_blob(e.kind), a);
                self.null_check(a);
                let t = self.s.clif_of(e.kind);
                self.load(t, a)
            }
            ExprNode::Take { addr } => self.take(addr, e.kind),
            ExprNode::Own { value, pin } =>
            {
                let v = self.expr(value);
                let k = self.descr(e.kind);
                let owned =
                    self.rt("zk_own", &[ptr, ptr], &[ptr], &[v, k]).unwrap_or(v);
                ret_if!(!*pin, owned);
                self.rt("zk_pin", &[ptr], &[ptr], &[owned]).unwrap_or(owned)
            }
            ExprNode::Dup { value, lent, deep } =>
            {
                self.dup(value, e.kind, (*lent, *deep))
            }
            ExprNode::PtrAdd { base, offset } =>
            {
                let stride = self.stride(base.kind);
                let b = self.expr(base);
                let o = self.expr(offset);
                let o = self.as_i64(o, offset.kind);
                let s = self.word(i64::from(stride));
                self.rt("zk_ptr_add", &[ptr, types::I64, ptr], &[ptr], &[b, o, s])
                    .unwrap_or(b)
            }
            ExprNode::PtrSub { lhs, rhs } =>
            {
                let stride = self.stride(lhs.kind);
                let a = self.expr(lhs);
                let b = self.expr(rhs);
                let s = self.word(i64::from(stride));
                let d =
                    self.rt("zk_ptr_sub", &[ptr, ptr, ptr], &[types::I64], &[a, b, s]);
                let d = d.unwrap_or(a);
                let t = self.s.clif_of(e.kind);
                self.int_cast(d, true, t)
            }
            ExprNode::List(cells) => self.list(cells, e.kind),
            ExprNode::BlankList => self.temp(e.kind),
            ExprNode::CellAddr { list, .. } if list.kind.as_object().is_some() =>
            {
                let l = self.expr(list);
                self.rt("zk_object_cell", &[ptr], &[ptr], &[l]).unwrap_or(l)
            }
            ExprNode::CellAddr { list, index, len, first, checked } =>
            {
                let stride = self.stride(e.kind);
                let l = self.expr(list);
                let i = self.expr(index);
                self.cell_addr(l, (i, index.kind), *len, (*first, *checked), stride)
            }
            ExprNode::MakeSlice { source, start, len, from, checked } =>
            {
                self.make_slice(e.kind, [source, start, len], *from, *checked)
            }
            ExprNode::Text(text) =>
            {
                let t = self.temp(e.kind);
                let p = self.text_ptr(text.as_bytes());
                let (n, size) = (
                    self.word(text.len() as i64),
                    self.word(i64::from(self.s.size(e.kind))),
                );
                self.rt("zk_text_lit", &[ptr, ptr, ptr, ptr], &[], &[t, p, n, size]);
                t
            }
            ExprNode::Format(pieces) => self.format(pieces, e.kind),
            ExprNode::Chars { text, index, first, octet } =>
            {
                self.chars(text, index.as_deref(), *first, *octet, e.kind)
            }
            ExprNode::RowFind { table, index } =>
            {
                let t = self.expr(table);
                let (held, v) = self.index_at(index);
                let k = self.descr(table.kind);
                let found = self.rt(
                    "zk_row_find",
                    &[ptr, ptr, ptr],
                    &[types::I64],
                    &[t, k, held],
                );
                self.drop_fresh(index, v);
                let clif = self.s.clif_of(e.kind);
                let found = found.unwrap_or(v);
                self.int_cast(found, true, clif)
            }
            ExprNode::Host { func, args } =>
            {
                let t = self.temp(e.kind);
                self.host_into(t, *func, args, e.kind);
                t
            }
            ExprNode::Foreign { func, args } =>
            {
                let v = self.foreign(*func, args, Some(e.kind));
                v.unwrap_or_else(|| self.word(0))
            }
            node =>
            {
                self.s.missing(format!("the expression {}", node_name(node)));
                self.word(0)
            }
        }
    }

    /// The octets of a step of a pointer: the size of what it points at.
    ///
    /// # Arguments
    ///
    /// * `ptr` - The pointer kind.
    ///
    /// # Returns
    ///
    /// The size.
    fn stride(&self, ptr: DataKind) -> u32
    {
        ptr.pointee().map_or(1, |k| self.s.size(k))
    }

    /// The address of a cell of a list or a slice: a number out of range stops
    /// the run, unless it is known to be in range.
    ///
    /// # Arguments
    ///
    /// * `l` - The list, or the slice.
    /// * `index` - The number of the cell, and its kind.
    /// * `len` - The cell count of a list, or `None` for a slice.
    /// * `flags` - The number of the first cell, and whether the number is
    ///   known to be in range.
    /// * `stride` - The octets of a cell.
    ///
    /// # Returns
    ///
    /// The address.
    fn cell_addr(
        &mut self, l: Value, index: (Value, DataKind), len: Option<u32>,
        flags: (u8, bool), stride: u32,
    ) -> Value
    {
        let ptr = self.s.ptr;
        let (first, checked) = flags;
        let direct = checked && first == 0;
        let i = self.as_i64(index.0, index.1);
        let f = self.b.ins().iconst(types::I32, i64::from(first));
        let base = match len
        {
            Some(_) => l,
            None => self.rt("zk_seq_base", &[ptr], &[ptr], &[l]).unwrap_or(l),
        };
        let cell = match (len, direct)
        {
            (_, true) => i,
            (Some(n), false) =>
            {
                let n = self.word(i64::from(n));
                self.rt(
                    "zk_cell_of",
                    &[types::I64, ptr, types::I32],
                    &[ptr],
                    &[i, n, f],
                )
                .unwrap_or(i)
            }
            (None, false) =>
            {
                let s = self.word(i64::from(stride));
                return self
                    .rt(
                        "zk_slice_cell",
                        &[ptr, types::I64, types::I32, ptr],
                        &[ptr],
                        &[l, i, f, s],
                    )
                    .unwrap_or(l);
            }
        };
        let off = self.b.ins().imul_imm_s(cell, i64::from(stride));
        self.b.ins().iadd(base, off)
    }

    /// Lower `and` or `or`: the right operand runs just when it is needed.
    ///
    /// # Arguments
    ///
    /// * `op` - The operator.
    /// * `lhs` - The left operand.
    /// * `rhs` - The right operand.
    ///
    /// # Returns
    ///
    /// The truth value.
    fn logic(&mut self, op: LogicOp, lhs: &Expr, rhs: &Expr) -> Value
    {
        let a = self.expr(lhs);
        let held = self.temp(DataKind::Bool);
        self.store(a, held);
        let go =
            tern!(op == LogicOp::And, a, self.b.ins().icmp_imm_s(IntCC::Equal, a, 0));
        self.when(go, |g| {
            let b = g.expr(rhs);
            g.store(b, held);
        });
        self.load(types::I8, held)
    }

    /// Lower a comparison: text through its octets, a list or struct cell after
    /// cell, a scalar with the operator.
    ///
    /// # Arguments
    ///
    /// * `op` - The comparison.
    /// * `lhs` - The left operand.
    /// * `rhs` - The right operand.
    ///
    /// # Returns
    ///
    /// The truth value.
    fn compare(&mut self, op: CmpOp, lhs: &Expr, rhs: &Expr) -> Value
    {
        let ptr = self.s.ptr;
        let a = self.expr(lhs);
        let b = self.expr(rhs);
        let kind = lhs.kind.repr();
        if is_blob(lhs.kind)
        {
            let same = if lhs.kind.is_text()
            {
                self.rt("zk_text_eq", &[ptr, ptr], &[types::I32], &[a, b])
            }
            else
            {
                let k = self.descr(lhs.kind);
                self.rt("zk_eq", &[ptr, ptr, ptr], &[types::I32], &[a, b, k])
            };
            let same = same.unwrap_or(a);
            let cc = tern!(op == CmpOp::Ne, IntCC::Equal, IntCC::NotEqual);
            return self.b.ins().icmp_imm_s(cc, same, 0);
        }
        if kind.is_float()
        {
            let cc = match op
            {
                CmpOp::Eq => FloatCC::Equal,
                CmpOp::Ne => FloatCC::NotEqual,
                CmpOp::Lt => FloatCC::LessThan,
                CmpOp::Le => FloatCC::LessThanOrEqual,
                CmpOp::Gt => FloatCC::GreaterThan,
                CmpOp::Ge => FloatCC::GreaterThanOrEqual,
            };
            return self.b.ins().fcmp(cc, a, b);
        }
        let signed = kind.int_kind().is_some_and(|k| k.signed);
        let cc = match (op, signed)
        {
            (CmpOp::Eq, _) => IntCC::Equal,
            (CmpOp::Ne, _) => IntCC::NotEqual,
            (CmpOp::Lt, true) => IntCC::SignedLessThan,
            (CmpOp::Le, true) => IntCC::SignedLessThanOrEqual,
            (CmpOp::Gt, true) => IntCC::SignedGreaterThan,
            (CmpOp::Ge, true) => IntCC::SignedGreaterThanOrEqual,
            (CmpOp::Lt, false) => IntCC::UnsignedLessThan,
            (CmpOp::Le, false) => IntCC::UnsignedLessThanOrEqual,
            (CmpOp::Gt, false) => IntCC::UnsignedGreaterThan,
            (CmpOp::Ge, false) => IntCC::UnsignedGreaterThanOrEqual,
        };
        self.b.ins().icmp(cc, a, b)
    }

    /// Lower a conversion.
    ///
    /// # Arguments
    ///
    /// * `operand` - The value.
    /// * `to` - The kind it converts to.
    ///
    /// # Returns
    ///
    /// The value.
    fn convert(&mut self, operand: &Expr, to: DataKind) -> Value
    {
        let ptr = self.s.ptr;
        let a = self.expr(operand);
        let from = operand.kind.repr();
        let target = to.repr();
        ret_if!(from == target, a);
        if to.is_seq() && from.is_list()
        {
            let t = self.temp(to);
            let k = self.descr(from);
            // A list of known cells is static in the VM: a block with a room
            // of 0, which no drop frees, so places can share it. A box is
            // counted and shared, so it never points at static cells.
            let whole = Expr::builder()
                .kind(to)
                .node(ExprNode::Convert {
                    operand: Box::new(operand.clone()),
                    implicit: true,
                })
                .span(operand.span)
                .build();
            let fixed = i64::from(!to.is_box() && is_known(&whole));
            let fixed = self.b.ins().iconst(types::I32, fixed);
            self.rt("zk_to_heap", &[ptr, ptr, ptr, types::I32], &[], &[t, a, k, fixed]);
            return t;
        }
        // An object becomes a pointer that holds a count of it, and back.
        if from.as_object().is_some() && to.is_ptr()
        {
            return self.rt("zk_obj_ptr", &[ptr], &[ptr], &[a]).unwrap_or(a);
        }
        if from.is_ptr() && to.as_object().is_some()
        {
            let t = self.temp(to);
            self.rt("zk_ptr_obj", &[ptr, ptr], &[], &[t, a]);
            return t;
        }
        // A plain list of u8s becomes text once its UTF-8 is checked.
        if to.is_text() && from.is_seq()
        {
            self.rt("zk_check_utf8", &[ptr], &[], &[a]);
            return a;
        }
        ret_if!(is_blob(to), a);
        self.scalar_convert(a, from, target)
    }

    /// Lower a call; a list or struct result goes into a temp of its own.
    ///
    /// # Arguments
    ///
    /// * `callee` - What the call runs.
    /// * `args` - The arguments.
    ///
    /// # Returns
    ///
    /// The result: a scalar, the address of a block, or zero for none.
    pub fn call(&mut self, callee: Callee<'_>, args: &[Expr]) -> Value
    {
        let ptr = self.s.ptr;
        let f = match callee
        {
            Callee::Value(c) =>
            {
                let v = self.expr(c);
                Some((c, self.rt("zk_fn_check", &[ptr], &[ptr], &[v]).unwrap_or(v)))
            }
            Callee::Fn(_) => None,
        };
        let ret = self.ret_of(callee);
        let mut vals: Vec<Value> = args.iter().map(|a| self.expr(a)).collect();
        let slot = ret.filter(|k| is_blob(*k)).map(|k| self.temp(k));
        do_if!(let Some(s) = slot, vals.insert(0, s));
        let result = match (callee, f)
        {
            (Callee::Fn(n), _) =>
            {
                let id = self.s.funcs[n as usize];
                let r = self.func_ref(id);
                let inst = self.b.ins().call(r, &vals);
                self.b.inst_results(inst).first().copied()
            }
            (Callee::Value(_), Some((c, fv))) => self.call_value(c, fv, &vals),
            _ => None,
        };
        slot.or(result).unwrap_or_else(|| self.word(0))
    }

    /// Call the function a function value holds: a closure runs its thunk,
    /// which takes the record last.
    ///
    /// # Arguments
    ///
    /// * `c` - The function value, for its kind.
    /// * `f` - The value, checked.
    /// * `vals` - The arguments, the result slot first.
    ///
    /// # Returns
    ///
    /// The scalar result, when there is one.
    fn call_value(&mut self, c: &Expr, f: Value, vals: &[Value]) -> Option<Value>
    {
        let ptr = self.s.ptr;
        let Some(kind) = c.kind.as_fn()
        else
        {
            self.s.missing("a call of a value");
            return None;
        };
        let (params, ret) = fn_shape(kind);
        let plain = self.s.sig(&params, ret, false);
        let bound = self.s.sig(&params, ret, true);
        let (plain, bound) =
            (self.b.import_signature(plain), self.b.import_signature(bound));
        let code = self.rt("zk_fn_code", &[ptr], &[ptr], &[f])?;
        let is_bound = self.rt("zk_fn_bound", &[ptr], &[types::I32], &[f])?;
        let scalar = ret.filter(|k| !is_blob(*k)).map(|k| self.s.clif_of(k));
        let held = scalar.map(|t| (t, self.raw_temp(16)));
        let (on, off, after) =
            (self.b.create_block(), self.b.create_block(), self.b.create_block());
        self.branch(is_bound, on, off);
        self.enter_block(on);
        let rec = self.rt("zk_fn_rec", &[ptr], &[ptr], &[f])?;
        let with_rec: Vec<Value> = vals.iter().copied().chain([rec]).collect();
        let inst = self.b.ins().call_indirect(bound, code, &with_rec);
        let r = self.b.inst_results(inst).first().copied();
        do_if!(let (Some((_, at)), Some(r)) = (held, r), self.store(r, at));
        self.jump(after);
        self.enter_block(off);
        let inst = self.b.ins().call_indirect(plain, code, vals);
        let r = self.b.inst_results(inst).first().copied();
        do_if!(let (Some((_, at)), Some(r)) = (held, r), self.store(r, at));
        self.jump(after);
        self.enter_block(after);
        held.map(|(t, at)| self.load(t, at))
    }

    /// The result kind of what a call runs.
    ///
    /// # Arguments
    ///
    /// * `callee` - What the call runs.
    ///
    /// # Returns
    ///
    /// The kind, or `None` for a function with no result.
    pub fn ret_of(&self, callee: Callee<'_>) -> Option<DataKind>
    {
        match callee
        {
            Callee::Fn(n) => self.s.module.funcs[n as usize].ret,
            Callee::Value(c) => c.kind.as_fn().and_then(|f| fn_shape(f).1),
        }
    }

    /// A value moved out of an address: the place is left at zero, the
    /// default of each owner.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address of the place.
    /// * `kind` - The kind of the value.
    ///
    /// # Returns
    ///
    /// The value.
    fn take(&mut self, addr: &Expr, kind: DataKind) -> Value
    {
        let ptr = self.s.ptr;
        let at = self.expr(addr);
        let size = self.word(i64::from(self.s.size(kind)));
        let held = if is_blob(kind)
        {
            let t = self.temp(kind);
            self.rt("zk_put_block", &[ptr, ptr, ptr], &[], &[t, at, size]);
            t
        }
        else
        {
            self.null_check(at);
            let t = self.s.clif_of(kind);
            self.load(t, at)
        };
        self.rt("zk_zero", &[ptr, ptr], &[], &[at, size]);
        held
    }

    /// A duplicate of a value that holds text or a plain list: blocks of its
    /// own, or, lent, the blocks kept with a room of 0.
    ///
    /// # Arguments
    ///
    /// * `value` - The value.
    /// * `kind` - Its kind.
    /// * `(lent, deep)` - Whether it is lent to a parameter for one call, and
    ///   whether each object it reaches gets a duplicate of its own.
    ///
    /// # Returns
    ///
    /// The value.
    fn dup(&mut self, value: &Expr, kind: DataKind, (lent, deep): (bool, bool))
    -> Value
    {
        let ptr = self.s.ptr;
        let v = self.expr(value);
        if kind.is_fn()
        {
            let name = tern!(lent, "zk_fn_lend", "zk_fn_dup");
            return self.rt(name, &[ptr], &[ptr], &[v]).unwrap_or(v);
        }
        ret_if!(!is_blob(kind), v);
        let t = self.temp(kind);
        self.put_at(t, v, kind);
        ret_if!(!kind.holds_text(), t);
        let share = tern!(lent, "zk_lend_seqs", "zk_dup_seqs");
        let name = tern!(deep, "zk_deep_seqs", share);
        let k = self.descr(kind);
        self.rt(name, &[ptr, ptr], &[], &[t, k]);
        t
    }

    /// A list or struct written out a cell at a time.
    ///
    /// # Arguments
    ///
    /// * `cells` - The cells or fields, in order.
    /// * `kind` - The list or struct kind.
    ///
    /// # Returns
    ///
    /// The address of a temp of its own.
    fn list(&mut self, cells: &[Expr], kind: DataKind) -> Value
    {
        let t = self.temp(kind);
        let platform = self.s.platform();
        let stride = kind.as_list().map_or(0, |l| l.cell.size(platform));
        for (i, cell) in cells.iter().enumerate()
        {
            let offset =
                kind.as_struct().map_or(i as u32 * stride, |s| s.offset(i, platform));
            let v = self.expr(cell);
            let at = self.b.ins().iadd_imm_s(t, i64::from(offset));
            self.put_at(at, v, cell.kind);
        }
        t
    }

    /// A slice of a list, of a slice or of a block.
    ///
    /// # Arguments
    ///
    /// * `kind` - The slice kind.
    /// * `parts` - The source, the first cell and the cell count.
    /// * `from` - What the source is.
    /// * `checked` - Whether the range is known to lie in its source.
    ///
    /// # Returns
    ///
    /// The address of a temp of its own.
    fn make_slice(
        &mut self, kind: DataKind, parts: [&Expr; 3], from: SliceFrom, checked: bool,
    ) -> Value
    {
        let ptr = self.s.ptr;
        let stride = kind.cell_kind().map_or(1, |c| self.s.size(c));
        let source = self.expr(parts[0]);
        let start = self.expr(parts[1]);
        let len = self.expr(parts[2]);
        let (start, len) =
            (self.as_u64(start, parts[1].kind), self.as_u64(len, parts[2].kind));
        let (base, limit) = match from
        {
            SliceFrom::List(n) if !checked => (source, self.word(i64::from(n))),
            SliceFrom::Slice =>
            {
                let b =
                    self.rt("zk_seq_base", &[ptr], &[ptr], &[source]).unwrap_or(source);
                let l =
                    self.rt("zk_seq_len", &[ptr], &[ptr], &[source]).unwrap_or(source);
                (b, l)
            }
            _ => (source, self.word(-1)),
        };
        let text =
            self.b.ins().iconst(types::I32, i64::from(kind.is_text() && !checked));
        let t = self.temp(kind);
        let s = self.word(i64::from(stride));
        let params = [ptr, ptr, types::I64, types::I64, ptr, ptr, types::I32];
        self.rt("zk_make_slice", &params, &[], &[t, base, start, len, limit, s, text]);
        // A view of a list or text lends its cells: a room of 0, which `own`
        // copies.
        if kind.is_seq() || kind.is_text()
        {
            let b = self.rt("zk_seq_base", &[ptr], &[ptr], &[t]).unwrap_or(t);
            let l = self.rt("zk_seq_len", &[ptr], &[ptr], &[t]).unwrap_or(t);
            let zero = self.word(0);
            self.rt("zk_seq_put", &[ptr, ptr, ptr, ptr], &[], &[t, b, l, zero]);
        }
        t
    }

    /// Text built from pieces when the program runs: each value runs first,
    /// then the pieces are written into a buffer, which becomes the text.
    ///
    /// # Arguments
    ///
    /// * `pieces` - The pieces.
    /// * `kind` - The text kind.
    ///
    /// # Returns
    ///
    /// The address of a temp of its own.
    fn format(&mut self, pieces: &[FmtPiece], kind: DataKind) -> Value
    {
        let ptr = self.s.ptr;
        let mut read: Vec<(&FmtPiece, Option<Value>)> =
            Vec::with_capacity(pieces.len());
        for piece in pieces
        {
            let v = match piece
            {
                FmtPiece::Text(_) => None,
                FmtPiece::Value(e) => Some(self.expr(e)),
            };
            read.push((piece, v));
        }
        let t = self.temp(kind);
        let buf = self.raw_temp(32);
        self.rt("zk_sink_open", &[ptr], &[], &[buf]);
        for (piece, v) in &read
        {
            match (piece, v)
            {
                (FmtPiece::Text(text), _) => self.put_raw(text.as_bytes()),
                (FmtPiece::Value(e), Some(v)) => self.put_value(*v, e.kind),
                _ =>
                {}
            }
        }
        self.rt("zk_sink_close", &[ptr], &[], &[t]);
        for (piece, v) in read
        {
            if let (FmtPiece::Value(e), Some(v)) = (piece, v)
            {
                self.drop_fresh(e, v);
            }
        }
        t
    }

    /// The character count of text, or one of its characters; a fresh text is
    /// dropped once read.
    ///
    /// # Arguments
    ///
    /// * `text` - The text.
    /// * `index` - The number of the character; `None` for the count.
    /// * `first` - The number of the first character: 0 or 1.
    /// * `octet` - Whether the number counts octets.
    /// * `kind` - The result kind.
    ///
    /// # Returns
    ///
    /// The value.
    fn chars(
        &mut self, text: &Expr, index: Option<&Expr>, first: u8, octet: bool,
        kind: DataKind,
    ) -> Value
    {
        let ptr = self.s.ptr;
        let t = self.expr(text);
        let i = index.map(|e| (self.expr(e), e.kind));
        let result = match i
        {
            None =>
            {
                let n = self.rt("zk_char_count", &[ptr], &[ptr], &[t]).unwrap_or(t);
                let clif = self.s.clif_of(kind);
                self.int_cast(n, false, clif)
            }
            Some((v, k)) if octet =>
            {
                let v = self.as_u64(v, k);
                self.rt("zk_char_at_octet", &[ptr, types::I64], &[types::I32], &[t, v])
                    .unwrap_or(v)
            }
            Some((v, k)) =>
            {
                let v = self.as_i64(v, k);
                let f = self.b.ins().iconst(types::I32, i64::from(first));
                let params = [ptr, types::I64, types::I32];
                self.rt("zk_char_at", &params, &[types::I32], &[t, v, f]).unwrap_or(v)
            }
        };
        do_if!(fresh_text(text), self.drop_at(t, text.kind));
        result
    }

    /// Run a host function into a result slot; each argument is the address of
    /// a parameter.
    ///
    /// # Arguments
    ///
    /// * `slot` - The address of the result.
    /// * `func` - The host function.
    /// * `args` - The arguments.
    /// * `ret` - The result kind.
    pub fn host_into(&mut self, slot: Value, func: HostFn, args: &[Expr], ret: DataKind)
    {
        let ptr = self.s.ptr;
        let v: Vec<Value> = args.iter().map(|a| self.expr(a)).collect();
        let r = self.descr(ret);
        let stream = |g: &mut Self, i: usize| {
            let k = args[i].kind.pointee().unwrap_or(args[i].kind);
            g.descr(k)
        };
        let (name, mut call): (&'static str, Vec<Value>) = match func
        {
            HostFn::ReadLine => ("zk_host_read_line", vec![v[0], stream(self, 0)]),
            HostFn::ReadFile => ("zk_host_read_file", vec![v[0]]),
            HostFn::WriteFile => ("zk_host_write_file", vec![v[0], v[1]]),
            HostFn::WriteStream =>
            {
                ("zk_host_write_stream", vec![v[0], v[1], stream(self, 1)])
            }
            HostFn::AppendFile => ("zk_host_append_file", vec![v[0], v[1]]),
            HostFn::Remove => ("zk_host_remove", vec![v[0]]),
            HostFn::MakeFolder => ("zk_host_make_folder", vec![v[0]]),
            HostFn::ListFolder => ("zk_host_list_folder", vec![v[0]]),
            HostFn::PathKind => ("zk_host_path_kind", vec![v[0]]),
            HostFn::Rename => ("zk_host_rename", vec![v[0], v[1]]),
            HostFn::ReadEnv => ("zk_host_read_env", vec![v[0]]),
        };
        call.splice(0..0, [slot, r]);
        let params = vec![ptr; call.len()];
        self.rt(name, &params, &[], &call);
    }
}
