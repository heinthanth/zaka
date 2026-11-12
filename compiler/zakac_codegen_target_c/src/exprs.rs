use zakac_ir_builder::fold::is_known;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::FmtPiece;
use zakac_ir_builder::ir::HostFn;
use zakac_ir_builder::ir::InfixOp;
use zakac_ir_builder::ir::LogicOp;
use zakac_ir_builder::ir::PrefixOp;
use zakac_ir_builder::ir::SliceFrom;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_false;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::foreign::is_c_text_result;
use zakac_semcheck::kind::DataKind;

use crate::emit::FnGen;
use crate::emit::c_string;
use crate::emit::func_name;
use crate::emit::record_name;
use crate::ffi::FfiCall;
use crate::ffi::wrapper_name;
use crate::kinds::constant;
use crate::kinds::is_blob;
use crate::stmts::Callee;
use crate::stmts::fresh;
use crate::stmts::node_name;

/// A piece of a format, read before the text is built.
enum Piece
{
    /// Text as written.
    Text(String),
    /// A value: its C text, its kind and whether it is fresh.
    Value(String, DataKind, bool),
}

impl FnGen<'_, '_>
{
    /// The C text of an expression. C leaves the order of operands and
    /// arguments open, so what has to run first goes into a local, on the
    /// lines before the statement that uses the text. A list, a struct or
    /// text is named through its address.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    ///
    /// # Returns
    ///
    /// The C text.
    pub(crate) fn expr(&mut self, e: &Expr) -> String
    {
        let platform = self.platform();
        match &e.node
        {
            ExprNode::Const(v) =>
            {
                // The kind is noted, so a float kind past C11 gets its #error.
                do_if!(e.kind.is_float(), _ = self.shared.kind(e.kind));
                constant(*v, e.kind, platform).unwrap_or_else(|| {
                    self.shared.missing(format!("a constant of {}", e.kind.name()))
                })
            }
            ExprNode::Read(p) => self.place(*p),
            ExprNode::AddrOf(p) => self.addr_of(*p),
            ExprNode::Prefix { op, operand } => self.prefix(*op, operand, e.kind),
            ExprNode::Infix { op, lhs, rhs } => self.infix(*op, lhs, rhs, e.kind),
            ExprNode::Compare { op, lhs, rhs } => self.compare(*op, lhs, rhs),
            ExprNode::Logic { op, lhs, rhs } => self.logic(*op, lhs, rhs),
            ExprNode::Convert { operand, .. } => self.convert(operand, e.kind),
            ExprNode::Call { func, args, .. } => self.call(Callee::Fn(*func), args),
            ExprNode::CallValue { callee, args } =>
            {
                self.call(Callee::Value(callee), args)
            }
            ExprNode::FnAddr { func, .. } =>
            {
                format!("((zk_fn)&{})", record_name(*func))
            }
            ExprNode::Closure { func, record, .. } =>
            {
                self.shared.made_closure = true;
                let kind = record.kind.pointee().expect("a record address");
                let k = self.shared.descr.of(kind);
                let record = self.expr(record);
                format!("zk_closure({record}, (zk_code){}, {k})", func_name(*func))
            }
            ExprNode::Lend { operand } => self.expr(operand),
            ExprNode::FieldAddr { base, offset, .. } =>
            {
                format!("({} + {offset})", self.expr(base))
            }
            ExprNode::SizeOf(k) =>
            {
                let t = self.shared.kind(e.kind);
                format!("(({t}){})", self.shared.size(*k))
            }
            ExprNode::CField { base, offset } => self.c_field(base, *offset, e.kind),
            _ => self.data(e),
        }
    }

    /// A field of a struct C keeps, copied out with `memcpy`, since its
    /// address lies outside the blocks the runtime knows: a number, a
    /// boolean or a counted list as it is, and for text the string the
    /// field's `char *` points at, or the optional with no data for NULL.
    ///
    /// # Arguments
    ///
    /// * `base` - The address of the C struct.
    /// * `offset` - The offset of the field.
    /// * `kind` - The kind of the field.
    ///
    /// # Returns
    ///
    /// The C text of the value.
    fn c_field(&mut self, base: &Expr, offset: u32, kind: DataKind) -> String
    {
        let b = self.expr(base);
        let at = format!("((const unsigned char *)({b}) + {offset})");
        if is_c_text_result(kind)
        {
            let size = self.shared.size(kind);
            let platform = self.shared.platform();
            let ok_at = kind.as_struct().map_or(0, |s| s.offset(1, platform));
            let t = self.blob_temp(kind);
            self.line(&format!(
                "{{ const char *zk_s; memcpy(&zk_s, {at}, sizeof zk_s); \
                 zk_text_from_c({t}, {size}, {ok_at}, zk_s); }}"
            ));
            return t;
        }
        if is_blob(kind)
        {
            let t = self.blob_temp(kind);
            self.line(&format!("memcpy({t}, {at}, {});", self.shared.size(kind)));
            return t;
        }
        let t = self.temp(kind);
        self.line(&format!("memcpy(&{t}, {at}, sizeof {t});"));
        t
    }

    /// The C text of an expression that reads or makes data: a load, a
    /// move, a list, a slice or text.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    ///
    /// # Returns
    ///
    /// The C text.
    fn data(&mut self, e: &Expr) -> String
    {
        match &e.node
        {
            ExprNode::Load { addr } =>
            {
                let a = self.expr(addr);
                ret_if!(is_blob(e.kind), a);
                format!("zk_ld_{}({a})", self.shared.mem(e.kind))
            }
            ExprNode::Take { addr } => self.take(addr, e.kind),
            ExprNode::Own { value, pin } =>
            {
                let v = self.expr(value);
                let owned = format!("zk_own({v}, {})", self.shared.descr.of(e.kind));
                tern!(*pin, format!("zk_pin({owned})"), owned)
            }
            ExprNode::Dup { value, lent, deep } =>
            {
                self.dup(value, e.kind, (*lent, *deep))
            }
            ExprNode::PtrAdd { base, offset } =>
            {
                let stride = self.stride(base.kind);
                let (b, o) = self.pair(base, offset);
                format!("zk_ptr_add({b}, (zk_imax)({o}), {stride})")
            }
            ExprNode::PtrSub { lhs, rhs } =>
            {
                let stride = self.stride(lhs.kind);
                let t = self.shared.kind(e.kind);
                let (a, b) = self.pair(lhs, rhs);
                format!("(({t})zk_ptr_sub({a}, {b}, {stride}))")
            }
            ExprNode::List(cells) => self.list(cells, e.kind),
            ExprNode::BlankList => self.temp(e.kind),
            ExprNode::CellAddr { list, .. } if list.kind.as_object().is_some() =>
            {
                let l = self.expr(list);
                format!("zk_object_cell({l})")
            }
            ExprNode::CellAddr { list, index, len, first, checked } =>
            {
                let stride = self.stride(e.kind);
                let (l, i) = self.pair(list, index);
                cell_addr(&l, &i, *len, (*first, *checked), stride)
            }
            ExprNode::MakeSlice { source, start, len, from, checked } =>
            {
                self.make_slice(e.kind, [source, start, len], *from, *checked)
            }
            ExprNode::Text(text) =>
            {
                let t = self.temp(e.kind);
                let size = self.shared.size(e.kind);
                let lit = c_string(text);
                self.line(&format!("zk_text_lit({t}, {lit}, {}, {size});", text.len()));
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
                let k = self.shared.descr.of(table.kind);
                let found =
                    self.spill(e.kind, format!("zk_row_find({t}, {k}, {held})"));
                self.drop_fresh(index, &v);
                found
            }
            ExprNode::Host { func, args } =>
            {
                let t = self.temp(e.kind);
                self.host_into(&t, *func, args, e.kind);
                t
            }
            ExprNode::Foreign { func, args } => self.foreign(*func, args, Some(e.kind)),
            node => self.shared.missing(format!("the expression {}", node_name(node))),
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
        ptr.pointee().map_or(1, |k| self.shared.size(k))
    }

    /// The C text of operands, which run in order: an operand that a later
    /// one could change, or stop the program before, goes into a local
    /// first.
    ///
    /// # Arguments
    ///
    /// * `exprs` - The operands, in the order of their run.
    ///
    /// # Returns
    ///
    /// The C text of each.
    pub(crate) fn operands(&mut self, exprs: &[&Expr]) -> Vec<String>
    {
        let stop = self.stop();
        let mut texts = Vec::with_capacity(exprs.len());
        for (i, e) in exprs.iter().enumerate()
        {
            let text = self.expr(e);
            let first = runs_first(e, &exprs[i + 1..], stop);
            texts.push(tern!(first, self.spill(e.kind, text), text));
        }
        texts
    }

    /// Call the wrapper of a C function. A struct result goes to a temp
    /// whose address the wrapper takes first; another result goes to a temp.
    ///
    /// # Arguments
    ///
    /// * `func` - Index into the C functions of the module.
    /// * `args` - The value of each parameter.
    /// * `ret` - The result kind; `None` for a C function that gives back
    ///   nothing.
    ///
    /// # Returns
    ///
    /// The C text of the result; blank with no result.
    pub(crate) fn foreign(
        &mut self, func: u32, args: &[Expr], ret: Option<DataKind>,
    ) -> String
    {
        let refs: Vec<&Expr> = args.iter().collect();
        let mut v = self.operands(&refs);
        let name = wrapper_name(self.ffi_call(func, args));
        let Some(kind) = ret
        else
        {
            self.line(&format!("{name}({});", v.join(", ")));
            return String::new();
        };
        let t = self.temp(kind);
        let blob = is_blob(kind);
        do_if!(blob, v.insert(0, t.clone()));
        let call = format!("{name}({})", v.join(", "));
        self.line(&tern!(blob, format!("{call};"), format!("{t} = {call};")));
        t
    }

    /// The index of the call shape of a C call, added when it is new.
    ///
    /// # Arguments
    ///
    /// * `func` - Index into the C functions of the module.
    /// * `args` - The value of each parameter and other value.
    ///
    /// # Returns
    ///
    /// The index, which names the wrapper.
    fn ffi_call(&mut self, func: u32, args: &[Expr]) -> u32
    {
        let fixed = self.shared.module.foreign[func as usize].params.len();
        let extra = args.iter().skip(fixed).map(|a| a.kind).collect();
        let call = FfiCall::builder().func(func).extra(extra).build();
        let calls = &mut self.shared.ffi_calls;
        let known = calls.iter().position(|c| *c == call);
        let index = known.unwrap_or(calls.len());
        do_if!(known.is_none(), calls.push(call));
        index as u32
    }

    /// The C text of two operands, which run in order.
    ///
    /// # Arguments
    ///
    /// * `lhs` - The left operand, which runs first.
    /// * `rhs` - The right operand.
    ///
    /// # Returns
    ///
    /// The C text of each.
    pub(crate) fn pair(&mut self, lhs: &Expr, rhs: &Expr) -> (String, String)
    {
        let mut texts = self.operands(&[lhs, rhs]).into_iter();
        (texts.next().unwrap_or_default(), texts.next().unwrap_or_default())
    }

    /// Put a value into a local of its own, so it runs before what comes
    /// after it. A list or struct is named through its address, as the VM
    /// pushes its address.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of the value.
    /// * `text` - The C text of the value.
    ///
    /// # Returns
    ///
    /// The name of the local.
    pub(crate) fn spill(&mut self, kind: DataKind, text: String) -> String
    {
        let t = self.shared.kind(kind);
        let name = format!("zk_t{}", self.fresh());
        self.line(&format!("{t} {name} = {text};"));
        name
    }

    /// The C text of `and` or `or`. The right operand runs just when it is
    /// needed: when it needs lines of its own, those go in an `if` on the
    /// left operand.
    ///
    /// # Arguments
    ///
    /// * `op` - The operator.
    /// * `lhs` - The left operand.
    /// * `rhs` - The right operand.
    ///
    /// # Returns
    ///
    /// The C text.
    fn logic(&mut self, op: LogicOp, lhs: &Expr, rhs: &Expr) -> String
    {
        let a = self.expr(lhs);
        let mark = self.out.len();
        let b = self.expr(rhs);
        let lines = self.out.split_off(mark);
        let and = op == LogicOp::And;
        ret_if!(lines.is_empty(), format!("({a} {} {b})", tern!(and, "&&", "||")));
        let held = format!("zk_t{}", self.fresh());
        self.line(&format!("uint8_t {held} = {a};"));
        self.line(&format!("if ({}{held})", tern!(and, "", "!")));
        self.line("{");
        do_for!(l in lines.lines(), self.out.push_str(&format!("    {l}\n")));
        self.depth += 1;
        self.line(&format!("{held} = {b};"));
        self.depth -= 1;
        self.line("}");
        held
    }

    /// The C text of a comparison: text through its octets, a list or struct
    /// cell after cell, a scalar with the C operator.
    ///
    /// # Arguments
    ///
    /// * `op` - The comparison.
    /// * `lhs` - The left operand.
    /// * `rhs` - The right operand.
    ///
    /// # Returns
    ///
    /// The C text.
    fn compare(
        &mut self, op: zakac_ir_builder::ir::CmpOp, lhs: &Expr, rhs: &Expr,
    ) -> String
    {
        let (a, b) = self.pair(lhs, rhs);
        let not = tern!(op == zakac_ir_builder::ir::CmpOp::Ne, "!", "");
        ret_if!(lhs.kind.is_text(), format!("({not}zk_text_eq({a}, {b}))"));
        if is_blob(lhs.kind)
        {
            let k = self.shared.descr.of(lhs.kind);
            return format!("({not}zk_eq({a}, {b}, {k}))");
        }
        let mark: &str = op.into();
        ret_if!(lhs.kind.is_ptr(), format!("((uintptr_t){a} {mark} (uintptr_t){b})"));
        format!("({a} {mark} {b})")
    }

    /// The C text of a prefix operator.
    ///
    /// # Arguments
    ///
    /// * `op` - The operator.
    /// * `operand` - The operand.
    /// * `kind` - The result kind.
    ///
    /// # Returns
    ///
    /// The C text.
    fn prefix(&mut self, op: PrefixOp, operand: &Expr, kind: DataKind) -> String
    {
        let a = self.expr(operand);
        let t = self.shared.kind(kind);
        let stop = self.stop();
        match op
        {
            PrefixOp::Neg if kind.is_float() => format!("(-{a})"),
            PrefixOp::Neg if stop =>
            {
                let n = self.shared.ops(kind);
                format!("zk_neg_{n}({a}, \"{}\")", kind.name())
            }
            PrefixOp::Neg => format!("zk_wneg_{}({a})", self.shared.ops(kind)),
            PrefixOp::BitNot => format!("(({t})~{a})"),
            PrefixOp::Not => format!("(!{a})"),
        }
    }

    /// The C text of an arithmetic, bitwise or shift operator: an integer
    /// operation is a runtime function of its kind, such as `zk_add_i32`.
    ///
    /// # Arguments
    ///
    /// * `op` - The operator.
    /// * `lhs` - The left operand.
    /// * `rhs` - The right operand.
    /// * `kind` - The result kind.
    ///
    /// # Returns
    ///
    /// The C text.
    fn infix(&mut self, op: InfixOp, lhs: &Expr, rhs: &Expr, kind: DataKind) -> String
    {
        let t = self.shared.kind(kind);
        let (a, b) = self.pair(lhs, rhs);
        ret_if!(kind.is_float(), float_op(op, &t, &a, &b));
        let n = self.shared.ops(kind);
        let (k, stop, name) = (kind.name(), self.stop(), op.mnemonic());
        match op
        {
            InfixOp::Add | InfixOp::Sub | InfixOp::Mul if stop =>
            {
                format!("zk_{name}_{n}({a}, {b}, \"{k}\")")
            }
            InfixOp::Add | InfixOp::Sub | InfixOp::Mul =>
            {
                format!("zk_w{name}_{n}({a}, {b})")
            }
            InfixOp::Div | InfixOp::Rem =>
            {
                format!("zk_{name}_{n}({a}, {b}, {}, \"{k}\")", u8::from(stop))
            }
            InfixOp::Shl | InfixOp::Shr => format!("zk_{name}_{n}({a}, {b})"),
            InfixOp::BitAnd | InfixOp::BitOr | InfixOp::BitXor =>
            {
                let mark: &str = op.into();
                format!("(({t})({a} {mark} {b}))")
            }
        }
    }

    /// The C text of a conversion.
    ///
    /// # Arguments
    ///
    /// * `operand` - The value.
    /// * `to` - The kind it converts to.
    ///
    /// # Returns
    ///
    /// The C text.
    fn convert(&mut self, operand: &Expr, to: DataKind) -> String
    {
        let a = self.expr(operand);
        let from = operand.kind.repr();
        let target = to.repr();
        ret_if!(from == target, a);
        if to.is_seq() && from.is_list()
        {
            let t = self.temp(to);
            let k = self.shared.descr.of(from);
            // A list of known cells is static in the VM: a block with a
            // room of 0, which no drop frees, so places can share it.
            // A box is counted and shared, so it never points at static cells.
            let fixed = u8::from(!to.is_box() && is_known(&self.whole(operand, to)));
            self.line(&format!("zk_to_heap({t}, {a}, {k}, {fixed});"));
            return t;
        }
        // An object becomes a pointer that holds a count of it, and back.
        ret_if!(from.as_object().is_some() && to.is_ptr(), format!("zk_obj_ptr({a})"));
        if from.is_ptr() && to.as_object().is_some()
        {
            let t = self.temp(to);
            self.line(&format!("zk_ptr_obj({t}, {a});"));
            return t;
        }
        // A plain list of u8s becomes text once its UTF-8 is checked.
        if to.is_text() && from.is_seq()
        {
            let at = self.spill(from, a);
            self.line(&format!("zk_check_utf8({at});"));
            return at;
        }
        ret_if!(is_blob(to), a);
        self.scalar_convert(&a, from, target)
    }

    /// The conversion of a list to a plain list, as the VM tests it for
    /// known cells.
    ///
    /// # Arguments
    ///
    /// * `operand` - The list.
    /// * `to` - The plain list kind.
    ///
    /// # Returns
    ///
    /// The conversion.
    fn whole(&self, operand: &Expr, to: DataKind) -> Expr
    {
        let node =
            ExprNode::Convert { operand: Box::new(operand.clone()), implicit: true };
        Expr::builder().kind(to).node(node).span(operand.span).build()
    }

    /// The C text of a conversion of a scalar.
    ///
    /// # Arguments
    ///
    /// * `a` - The C text of the value.
    /// * `from` - Its kind.
    /// * `to` - The kind it converts to.
    ///
    /// # Returns
    ///
    /// The C text.
    fn scalar_convert(&mut self, a: &str, from: DataKind, to: DataKind) -> String
    {
        let t = self.shared.kind(to);
        let address = |k: DataKind| k.is_ptr() || k == DataKind::Null;
        match (from, to)
        {
            (DataKind::Float(128), DataKind::Int(_)) =>
            {
                format!("zk_q2i_{}({a})", self.shared.ops(to))
            }
            (DataKind::Float(_), DataKind::Int(_)) =>
            {
                format!("zk_f2i_{}({a})", self.shared.ops(to))
            }
            (DataKind::Int(_), DataKind::Char) =>
            {
                format!("zk_char_{}({a})", self.shared.ops(from))
            }
            (
                DataKind::Int(_) | DataKind::Char | DataKind::Bool | DataKind::Float(_),
                DataKind::Int(_) | DataKind::Float(_),
            ) => format!("(({t}){a})"),
            (f, DataKind::Int(_)) if address(f) => format!("(({t})(uintptr_t){a})"),
            (DataKind::Int(_), k) if address(k) => format!("(({t})(uintptr_t){a})"),
            (f, k) if address(f) && address(k) => a.to_string(),
            (DataKind::Fn(_), DataKind::Fn(_)) => a.to_string(),
            (DataKind::Null, DataKind::Fn(_)) => "((zk_fn)0)".into(),
            _ => self.shared.missing(format!(
                "a conversion from {} to {}",
                from.name(),
                to.name()
            )),
        }
    }

    /// The C text of a call. A list or struct result goes into a slot of
    /// its own.
    ///
    /// # Arguments
    ///
    /// * `callee` - What the call runs.
    /// * `args` - The arguments.
    ///
    /// # Returns
    ///
    /// The C text.
    fn call(&mut self, callee: Callee<'_>, args: &[Expr]) -> String
    {
        let ret = self.ret_of(callee);
        let Some(kind) = ret.filter(|k| is_blob(*k))
        else
        {
            return self.call_text(callee, args, None);
        };
        let slot = self.temp(kind);
        let call = self.call_text(callee, args, Some(&slot));
        self.line(&format!("{call};"));
        slot
    }

    /// The C text of a value moved out of an address: the place is left at
    /// zero, the default of each owner.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address of the place.
    /// * `kind` - The kind of the value.
    ///
    /// # Returns
    ///
    /// The C text.
    fn take(&mut self, addr: &Expr, kind: DataKind) -> String
    {
        let a = self.expr(addr);
        let at = self.spill(addr.kind, a);
        let size = self.shared.size(kind);
        let held = if is_blob(kind)
        {
            let t = self.temp(kind);
            self.line(&format!("zk_put_block({t}, {at}, {size});"));
            t
        }
        else
        {
            let n = self.shared.mem(kind);
            self.spill(kind, format!("zk_ld_{n}({at})"))
        };
        self.line(&format!("zk_zero({at}, {size});"));
        held
    }

    /// The C text of a duplicate of a value that holds text or a plain
    /// list: a duplicate whose text gets blocks of its own, or, lent, keeps the
    /// blocks with a room of 0.
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
    /// The C text.
    fn dup(
        &mut self, value: &Expr, kind: DataKind, (lent, deep): (bool, bool),
    ) -> String
    {
        let v = self.expr(value);
        let call = tern!(lent, "zk_fn_lend", "zk_fn_dup");
        ret_if!(kind.is_fn(), format!("{call}({v})"));
        ret_if!(!is_blob(kind), v);
        let t = self.temp(kind);
        let size = self.shared.size(kind);
        self.line(&format!("zk_put_block({t}, {v}, {size});"));
        let k = self.shared.descr.of(kind);
        let share = tern!(lent, "zk_lend_seqs", "zk_dup_seqs");
        let call = tern!(deep, "zk_deep_seqs", share);
        do_if_holds(self, kind, &format!("{call}({t}, {k});"));
        t
    }

    /// The C text of a list or struct written out a cell at a time.
    ///
    /// # Arguments
    ///
    /// * `cells` - The cells or fields, in order.
    /// * `kind` - The list or struct kind.
    ///
    /// # Returns
    ///
    /// The C text: the address of a local of its own.
    fn list(&mut self, cells: &[Expr], kind: DataKind) -> String
    {
        let t = self.temp(kind);
        let platform = self.platform();
        let stride = kind.as_list().map_or(0, |l| l.cell.size(platform));
        for (i, cell) in cells.iter().enumerate()
        {
            let offset =
                kind.as_struct().map_or(i as u32 * stride, |s| s.offset(i, platform));
            let v = self.expr(cell);
            self.put_at(&format!("({t} + {offset})"), &v, cell.kind);
        }
        t
    }

    /// The C text of a slice of a list, of a slice or of a block.
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
    /// The C text: the address of a local of its own.
    fn make_slice(
        &mut self, kind: DataKind, parts: [&Expr; 3], from: SliceFrom, checked: bool,
    ) -> String
    {
        let stride = kind.cell_kind().map_or(1, |c| self.shared.size(c));
        let v = self.operands(&parts);
        let source = tern!(
            from == SliceFrom::Slice,
            self.spill(parts[0].kind, v[0].clone()),
            v[0].clone()
        );
        let (base, limit) = match from
        {
            SliceFrom::List(n) if !checked => (source, n.to_string()),
            SliceFrom::Slice =>
            {
                (format!("zk_seq_base({source})"), format!("zk_seq_len({source})"))
            }
            _ => (source, "SIZE_MAX".into()),
        };
        let text = u8::from(kind.is_text() && !checked);
        let t = self.temp(kind);
        self.line(&format!(
            "zk_make_slice({t}, {base}, (zk_umax)({}), (zk_umax)({}), {limit}, \
             {stride}, {text});",
            v[1], v[2]
        ));
        // A view of a list or text lends its cells: a room of 0, which `own`
        // copies.
        let lent = format!("zk_seq_put({t}, zk_seq_base({t}), zk_seq_len({t}), 0);");
        do_if!(kind.is_seq() || kind.is_text(), self.line(&lent));
        t
    }

    /// The C text of text built from pieces when the program runs. Each
    /// value runs first, into a local; then the pieces are written into a
    /// buffer, which becomes the text.
    ///
    /// # Arguments
    ///
    /// * `pieces` - The pieces.
    /// * `kind` - The text kind.
    ///
    /// # Returns
    ///
    /// The C text: the address of a local of its own.
    fn format(&mut self, pieces: &[FmtPiece], kind: DataKind) -> String
    {
        let mut read = Vec::with_capacity(pieces.len());
        for piece in pieces
        {
            read.push(match piece
            {
                FmtPiece::Text(text) => Piece::Text(text.clone()),
                FmtPiece::Value(v) =>
                {
                    let text = self.expr(v);
                    let kept = matches!(v.node, ExprNode::Const(_) | ExprNode::Read(_));
                    let text = tern!(kept, text, self.spill(v.kind, text));
                    Piece::Value(text, v.kind, fresh(v))
                }
            });
        }
        let t = self.temp(kind);
        let buf = format!("zk_b{}", self.fresh());
        self.line(&format!("zk_buf {buf};"));
        self.line(&format!("zk_sink_open(&{buf});"));
        for piece in &read
        {
            match piece
            {
                Piece::Text(text) =>
                {
                    let lit = c_string(text);
                    self.line(&format!("zk_put_raw({lit}, {});", text.len()));
                }
                Piece::Value(v, k, _) => self.put_value(v, *k),
            }
        }
        self.line(&format!("zk_sink_close({t});"));
        for piece in &read
        {
            let Piece::Value(v, k, true) = piece
            else
            {
                continue;
            };
            let d = self.shared.descr.of(*k);
            self.line(&format!("zk_drop({v}, {d});"));
        }
        t
    }

    /// The C text of the character count of text, or of one of its
    /// characters. A fresh text is dropped once read.
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
    /// The C text.
    fn chars(
        &mut self, text: &Expr, index: Option<&Expr>, first: u8, octet: bool,
        kind: DataKind,
    ) -> String
    {
        let t = self.shared.kind(kind);
        let mut exprs = vec![text];
        exprs.extend(index);
        let v = self.operands(&exprs);
        let call = match (index, octet)
        {
            (None, _) => format!("(({t})zk_char_count({}))", v[0]),
            (Some(_), true) =>
            {
                format!("zk_char_at_octet({}, (zk_umax)({}))", v[0], v[1])
            }
            (Some(_), false) =>
            {
                format!("zk_char_at({}, (zk_imax)({}), {first})", v[0], v[1])
            }
        };
        ret_if!(!fresh(text), call);
        let held = self.spill(kind, call);
        self.drop_fresh(text, &v[0]);
        held
    }

    /// Run a host function into a result slot. Each argument is the address
    /// of a parameter.
    ///
    /// # Arguments
    ///
    /// * `slot` - The C text of the address of the result.
    /// * `func` - The host function.
    /// * `args` - The arguments.
    /// * `ret` - The result kind.
    pub(crate) fn host_into(
        &mut self, slot: &str, func: HostFn, args: &[Expr], ret: DataKind,
    )
    {
        let refs: Vec<&Expr> = args.iter().collect();
        let v = self.operands(&refs);
        let r = self.shared.descr.of(ret);
        let mut stream = |i: usize| {
            let k = args[i].kind.pointee().unwrap_or(args[i].kind);
            self.shared.descr.of(k)
        };
        let call = match func
        {
            HostFn::ReadLine =>
            {
                let s = stream(0);
                format!("zk_host_read_line({slot}, {r}, {}, {s})", v[0])
            }
            HostFn::ReadFile => format!("zk_host_read_file({slot}, {r}, {})", v[0]),
            HostFn::WriteFile =>
            {
                format!("zk_host_write_file({slot}, {r}, {}, {})", v[0], v[1])
            }
            HostFn::WriteStream =>
            {
                let s = stream(1);
                format!("zk_host_write_stream({slot}, {r}, {}, {}, {s})", v[0], v[1])
            }
        };
        self.line(&format!("{call};"));
    }
}

/// Write a line when a kind holds text or a plain list.
///
/// # Arguments
///
/// * `g` - The generator.
/// * `kind` - The kind.
/// * `line` - The line.
fn do_if_holds(g: &mut FnGen<'_, '_>, kind: DataKind, line: &str)
{
    ret_if!(!kind.holds_text());
    g.line(line);
}

/// The C text of the address of a cell of a list or a slice: a number out
/// of range stops the run, unless it is known to be in range.
///
/// # Arguments
///
/// * `l` - The list, or the slice.
/// * `i` - The number of the cell.
/// * `len` - The cell count of a list, or `None` for a slice.
/// * `flags` - The number of the first cell, and whether the number is known to
///   be in range.
/// * `stride` - The octets of a cell.
///
/// # Returns
///
/// The C text.
fn cell_addr(
    l: &str, i: &str, len: Option<u32>, flags: (u8, bool), stride: u32,
) -> String
{
    let (first, checked) = flags;
    let direct = checked && first == 0;
    match len
    {
        Some(_) if direct => format!("({l} + (size_t)({i}) * {stride})"),
        Some(n) =>
        {
            format!("({l} + zk_cell_of((zk_imax)({i}), {n}, {first}) * {stride})")
        }
        None if direct => format!("(zk_seq_base({l}) + (size_t)({i}) * {stride})"),
        None => format!("zk_slice_cell({l}, (zk_imax)({i}), {first}, {stride})"),
    }
}

/// The C text of a float operator.
///
/// # Arguments
///
/// * `op` - The operator.
/// * `t` - The C kind: `zk_half`, `float`, `double` or `zk_quad`.
/// * `a` - The left operand.
/// * `b` - The right operand.
///
/// # Returns
///
/// The C text.
fn float_op(op: InfixOp, t: &str, a: &str, b: &str) -> String
{
    let fmod = match t
    {
        "float" => "fmodf".into(),
        "double" => "fmod".into(),
        _ => format!("zk_fmod_{}", t.trim_start_matches("zk_")),
    };
    ret_if!(op == InfixOp::Rem, format!("{fmod}({a}, {b})"));
    let mark: &str = op.into();
    // A cast drops the wider range C can compute f16 in, so each operation
    // rounds to f16 as the VM's does.
    tern!(
        t == "zk_half",
        format!("(({t})({a} {mark} {b}))"),
        format!("({a} {mark} {b})")
    )
}

/// Whether an operand must go into a local before the operands after it
/// run: it is no constant, a later one is no constant either, and one of
/// them can change a value or stop the program. A read beside a call can
/// see the call change the slot, so the two must run in order.
///
/// # Arguments
///
/// * `e` - The operand.
/// * `later` - The operands after it.
/// * `stop` - Whether an overflow stops the program.
///
/// # Returns
///
/// `true` when it must run first.
fn runs_first(e: &Expr, later: &[&Expr], stop: bool) -> bool
{
    let constant = |x: &Expr| matches!(x.node, ExprNode::Const(_));
    ret_false!(constant(e) || later.iter().all(|l| constant(l)));
    acts(e, stop) || later.iter().any(|l| acts(l, stop))
}

/// Whether an expression can change a value or stop the program: a call,
/// a move, or an operation with a check, such as a division. A node the C
/// target does not list counts as one that can.
///
/// # Arguments
///
/// * `e` - The expression.
/// * `stop` - Whether an overflow stops the program.
///
/// # Returns
///
/// `true` when it can.
fn acts(e: &Expr, stop: bool) -> bool
{
    match &e.node
    {
        ExprNode::Const(_)
        | ExprNode::Read(_)
        | ExprNode::SizeOf(_)
        | ExprNode::AddrOf(_)
        | ExprNode::FnAddr { .. }
        | ExprNode::Text(_) => false,
        ExprNode::Load { addr } => acts(addr, stop),
        ExprNode::Lend { operand } => acts(operand, stop),
        ExprNode::FieldAddr { base, .. } => acts(base, stop),
        ExprNode::Prefix { op, operand } =>
        {
            (*op == PrefixOp::Neg && stop && e.kind.is_int()) || acts(operand, stop)
        }
        ExprNode::Infix { op, lhs, rhs } =>
        {
            checked(*op, e.kind, stop) || acts(lhs, stop) || acts(rhs, stop)
        }
        ExprNode::Compare { lhs, rhs, .. }
        | ExprNode::Logic { lhs, rhs, .. }
        | ExprNode::PtrSub { lhs, rhs } => acts(lhs, stop) || acts(rhs, stop),
        ExprNode::PtrAdd { base, offset } => acts(base, stop) || acts(offset, stop),
        ExprNode::Convert { operand, .. } =>
        {
            let checks = e.kind.is_text() && operand.kind.is_seq();
            checks || acts(operand, stop)
        }
        ExprNode::CellAddr { list, index, first, checked, .. } =>
        {
            !*checked || *first != 0 || acts(list, stop) || acts(index, stop)
        }
        _ => true,
    }
}

/// Whether an operator checks its operands and can stop the program.
///
/// # Arguments
///
/// * `op` - The operator.
/// * `kind` - The result kind.
/// * `stop` - Whether an overflow stops the program.
///
/// # Returns
///
/// `true` for an integer division, remainder or shift, and for integer
/// `+`, `-` and `*` under `--overflow stop`.
fn checked(op: InfixOp, kind: DataKind, stop: bool) -> bool
{
    let checks =
        matches!(op, InfixOp::Div | InfixOp::Rem | InfixOp::Shl | InfixOp::Shr);
    let overflow = matches!(op, InfixOp::Add | InfixOp::Sub | InfixOp::Mul);
    kind.repr().is_int() && (checks || (overflow && stop))
}
