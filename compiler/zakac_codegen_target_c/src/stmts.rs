use zakac_ast::stmt::CountMode;
use zakac_ir_builder::ir::CmpOp;
use zakac_ir_builder::ir::Dest;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::HaltCause;
use zakac_ir_builder::ir::Place;
use zakac_ir_builder::ir::PrintValue;
use zakac_ir_builder::ir::Stmt;
use zakac_ir_builder::ir::fresh_text;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::value::Value;
use zakac_semcheck::value::compare;

use crate::emit::FnGen;
use crate::emit::c_string;
use crate::emit::func_name;
use crate::kinds::constant;
use crate::kinds::is_blob;
use crate::kinds::same_unsigned;

/// What a call runs: a function of the program, or a function value.
#[derive(Clone, Copy)]
pub(crate) enum Callee<'e>
{
    /// The index of a function.
    Fn(u32),
    /// A function value.
    Value(&'e Expr),
}

impl FnGen<'_, '_>
{
    /// Generate one statement.
    ///
    /// # Arguments
    ///
    /// * `s` - The statement.
    pub(crate) fn stmt(&mut self, s: &Stmt)
    {
        self.shared.span = s.span();
        match s
        {
            Stmt::Set { place, value, .. } => self.set(*place, value),
            Stmt::Print { value, .. } => self.print(value),
            Stmt::If { cond, then, els, .. } => self.if_stmt(cond, then, els),
            Stmt::Loop { init, cond, update, block, .. } =>
            {
                self.loop_stmt(init, cond, update, block)
            }
            Stmt::Counted { var, from, to, step, mode, block, .. } =>
            {
                self.counted(*var, from, to, step.as_ref(), *mode, block)
            }
            Stmt::Break { .. } => self.line("break;"),
            Stmt::Continue { .. } =>
            {
                let label = self.loops.last().copied().unwrap_or_default();
                self.line(&format!("goto zk_next{label};"));
            }
            Stmt::Return { value, .. } => self.return_stmt(value.as_ref()),
            Stmt::Call { func, args, .. } => self.call_stmt(Callee::Fn(*func), args),
            Stmt::Foreign { func, args, .. } =>
            {
                let ret = self.shared.module.foreign[*func as usize].ret;
                _ = self.foreign(*func, args, ret);
            }
            Stmt::CallValue { callee, args, .. } =>
            {
                self.call_stmt(Callee::Value(callee), args)
            }
            Stmt::Store { addr, value, .. } => self.store(addr, value),
            Stmt::Halt { cause, value, .. } => self.halt(*cause, value.as_ref()),
            Stmt::Drop { dest, kind, .. } =>
            {
                let at = self.dest_addr(dest);
                let k = self.shared.descr.of(*kind);
                self.line(&format!("zk_drop({at}, {k});"));
            }
            _ => self.heap_or_edit(s),
        }
    }

    /// Generate a statement on the heap, or an edit of a list or text.
    ///
    /// # Arguments
    ///
    /// * `s` - The statement.
    fn heap_or_edit(&mut self, s: &Stmt)
    {
        match s
        {
            Stmt::Alloc { dest, count, unit, .. } => self.alloc(dest, count, *unit),
            Stmt::Resize { dest, count, unit, .. } => self.resize(dest, count, *unit),
            Stmt::Free { dest, .. } => self.free(dest),
            Stmt::Blit { src, dst, count, unit, overlap, .. } =>
            {
                let stride = self.unit_size(*unit);
                let v = self.operands(&[src, count, dst]);
                let ov = u8::from(*overlap);
                self.line(&format!(
                    "zk_blit({}, (zk_umax)({}), {}, {stride}, {ov});",
                    v[0], v[1], v[2]
                ));
            }
            Stmt::Fill { dst, count, value, .. } => self.fill(dst, count, value),
            Stmt::AddCell { dest, kind, value, .. } =>
            {
                let cell = self.cell_temp(value);
                let at = self.dest_addr(dest);
                let k = self.shared.descr.of(*kind);
                self.line(&format!("zk_add_cell({at}, {k}, {cell});"));
            }
            Stmt::RemoveCell { dest, kind, .. } =>
            {
                let at = self.dest_addr(dest);
                let k = self.shared.descr.of(*kind);
                self.line(&format!("zk_remove_cell({at}, {k});"));
            }
            _ => self.text_edit(s),
        }
    }

    /// Generate an edit of text, or an insert into a plain list.
    ///
    /// # Arguments
    ///
    /// * `s` - The statement.
    fn text_edit(&mut self, s: &Stmt)
    {
        match s
        {
            Stmt::SetChar { dest, index, first, value, .. } =>
            {
                let (at, v) = self.dest_then(dest, &[index, value]);
                self.line(&format!(
                    "zk_set_char({at}, (zk_imax)({}), {first}, {});",
                    v[0], v[1]
                ));
            }
            Stmt::Insert { dest, kind, index, first, value, .. } if !kind.is_text() =>
            {
                let cell = self.cell_temp(value);
                let (at, v) = self.dest_then(dest, &[index]);
                let k = self.shared.descr.of(*kind);
                self.line(&format!(
                    "zk_insert_cell({at}, {k}, (zk_imax)({}), {first}, {cell});",
                    v[0]
                ));
            }
            Stmt::Insert { dest, index, first, value, .. } =>
            {
                let (at, v) = self.dest_then(dest, &[index, value]);
                let call = tern!(
                    value.kind == DataKind::Char,
                    "zk_insert_char",
                    "zk_insert_text"
                );
                self.line(&format!(
                    "{call}({at}, (zk_imax)({}), {first}, {});",
                    v[0], v[1]
                ));
                self.drop_fresh(value, &v[1]);
            }
            Stmt::Replace { dest, start, end, chars, value, .. } =>
            {
                let (at, v) = self.dest_then(dest, &[start, end, value]);
                let call = tern!(
                    value.kind == DataKind::Char,
                    "zk_replace_char",
                    "zk_replace_text"
                );
                let chars = u8::from(*chars);
                self.line(&format!(
                    "{call}({at}, (zk_imax)({}), (zk_imax)({}), {chars}, {});",
                    v[0], v[1], v[2]
                ));
                self.drop_fresh(value, &v[2]);
            }
            Stmt::AddText { dest, value, .. } =>
            {
                let (at, v) = self.dest_then(dest, &[value]);
                let call =
                    tern!(value.kind == DataKind::Char, "zk_add_char", "zk_add_text");
                self.line(&format!("{call}({at}, {});", v[0]));
                self.drop_fresh(value, &v[0]);
            }
            other =>
            {
                self.shared.missing(format!("the statement {}", node_name(other)));
            }
        }
    }

    /// Generate a set of a slot. A list or struct is copied into the slot;
    /// the result of a host function is written there.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    /// * `value` - The value.
    fn set(&mut self, place: Place, value: &Expr)
    {
        let target = self.place(place);
        if let ExprNode::Host { func, args } = &value.node
        {
            return self.host_into(&target, *func, args, value.kind);
        }
        let v = self.expr(value);
        ret_if!(!is_blob(value.kind), self.line(&format!("{target} = {v};")));
        let size = self.shared.size(value.kind);
        self.line(&format!("zk_put_block({target}, {v}, {size});"));
    }

    /// Generate a store through an address.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address.
    /// * `value` - The value.
    fn store(&mut self, addr: &Expr, value: &Expr)
    {
        let (a, v) = self.pair(addr, value);
        if is_blob(value.kind)
        {
            let size = self.shared.size(value.kind);
            return self.line(&format!("zk_put_block({a}, {v}, {size});"));
        }
        let n = self.shared.mem(value.kind);
        self.line(&format!("zk_st_{n}({a}, {v});"));
    }

    /// Generate a return: the value goes back, then the function ends.
    ///
    /// # Arguments
    ///
    /// * `value` - The value, for a function with a result.
    fn return_stmt(&mut self, value: Option<&Expr>)
    {
        let Some(e) = value
        else
        {
            return self.line("return;");
        };
        let v = self.expr(e);
        let ret = self.func.and_then(|f| f.ret);
        self.give_back(&v, ret);
    }

    /// Generate a print: the value, then a line break; a fresh value that
    /// holds text is dropped after.
    ///
    /// # Arguments
    ///
    /// * `value` - What to write.
    fn print(&mut self, value: &PrintValue)
    {
        let e = match value
        {
            PrintValue::Text(text) =>
            {
                let lit = c_string(text);
                self.line(&format!("zk_put_raw({lit}, {});", text.len()));
                return self.line("zk_put_line();");
            }
            PrintValue::Expr(e) => e,
        };
        let v = self.expr(e);
        self.put_value(&v, e.kind);
        self.line("zk_put_line();");
        self.drop_fresh(e, &v);
    }

    /// Write a value as a print writes it, with no line break.
    ///
    /// # Arguments
    ///
    /// * `v` - The C text of the value.
    /// * `kind` - Its kind; a member of an enumeration writes its name.
    pub(crate) fn put_value(&mut self, v: &str, kind: DataKind)
    {
        ret_if!(is_blob(kind) || kind.as_enum().is_some(), self.show(v, kind));
        let put = match kind
        {
            DataKind::Int(k) if k.signed => "zk_put_i",
            DataKind::Int(_) => "zk_put_u",
            DataKind::Float(32) => "zk_put_f32",
            DataKind::Float(64) => "zk_put_f64",
            DataKind::Bool => "zk_put_bool",
            DataKind::Char => "zk_put_char",
            DataKind::Ptr(_) | DataKind::Null => "zk_put_ptr",
            DataKind::Fn(_) => "zk_put_fn",
            k =>
            {
                self.shared.kind(k);
                self.shared.missing(format!("a print of {}", k.name()));
                return;
            }
        };
        self.line(&format!("{put}({v});"));
    }

    /// Write a value through its kind descriptor: a scalar goes into a
    /// local of its own first, so it has an address.
    ///
    /// # Arguments
    ///
    /// * `v` - The C text of the value.
    /// * `kind` - Its kind.
    fn show(&mut self, v: &str, kind: DataKind)
    {
        let at = self.addressed(v, kind);
        let k = self.shared.descr.of(kind);
        self.line(&format!("zk_show({at}, {k}, 0);"));
    }

    /// The address of a value: a block names its own; a scalar goes into a
    /// local of its own.
    ///
    /// # Arguments
    ///
    /// * `v` - The C text of the value.
    /// * `kind` - Its kind.
    ///
    /// # Returns
    ///
    /// The C text of the address.
    pub(crate) fn addressed(&mut self, v: &str, kind: DataKind) -> String
    {
        ret_if!(is_blob(kind), v.to_string());
        let t = self.temp(kind);
        self.line(&format!("{t} = {v};"));
        format!("((unsigned char *)&{t})")
    }

    /// Generate a stop of the run: a conversion read the part of a result
    /// it does not hold.
    ///
    /// # Arguments
    ///
    /// * `cause` - Which part was read.
    /// * `value` - The error value, when the failure holds one.
    fn halt(&mut self, cause: HaltCause, value: Option<&Expr>)
    {
        ret_if!(cause == HaltCause::NotFault, self.line("zk_not_fault();"));
        let Some(e) = value
        else
        {
            return self.line("zk_not_good_bare();");
        };
        let v = self.expr(e);
        let at = self.addressed(&v, e.kind);
        let k = self.shared.descr.of(e.kind);
        self.line(&format!("zk_not_good({at}, {k});"));
    }

    /// Drop a fresh value that holds text once it is read: the text a
    /// format builds, a call result or a list literal, as the VM drops it.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    /// * `v` - The C text of its value.
    pub(crate) fn drop_fresh(&mut self, e: &Expr, v: &str)
    {
        ret_if!(!fresh(e));
        let k = self.shared.descr.of(e.kind);
        self.line(&format!("zk_drop({v}, {k});"));
    }

    /// The address of the place a heap or edit statement works on.
    ///
    /// # Arguments
    ///
    /// * `dest` - The place.
    ///
    /// # Returns
    ///
    /// The C text.
    pub(crate) fn dest_addr(&mut self, dest: &Dest) -> String
    {
        match dest
        {
            Dest::Slot(p) => self.addr_of(*p),
            Dest::Mem(at) => self.expr(at),
        }
    }

    /// The address of a place, then the operands after it, in order.
    ///
    /// # Arguments
    ///
    /// * `dest` - The place.
    /// * `later` - The operands.
    ///
    /// # Returns
    ///
    /// The address and the C text of each operand.
    fn dest_then(&mut self, dest: &Dest, later: &[&Expr]) -> (String, Vec<String>)
    {
        let Dest::Mem(at) = dest
        else
        {
            let addr = self.dest_addr(dest);
            return (addr, self.operands(later));
        };
        let mut all = vec![at];
        all.extend_from_slice(later);
        let mut texts = self.operands(&all);
        let addr = texts.remove(0);
        (addr, texts)
    }

    /// A cell of its own that holds a value, for an add or an insert, which
    /// moves the cell into the list.
    ///
    /// # Arguments
    ///
    /// * `value` - The value.
    ///
    /// # Returns
    ///
    /// The C text of the address of the cell.
    fn cell_temp(&mut self, value: &Expr) -> String
    {
        let v = self.expr(value);
        let cell = self.blob_temp(value.kind);
        self.put_at(&cell, &v, value.kind);
        cell
    }

    /// A block of its own the size of a kind, zeroed, even for a scalar
    /// kind.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// Its C name.
    pub(crate) fn blob_temp(&mut self, kind: DataKind) -> String
    {
        let name = format!("zk_t{}", self.fresh());
        let size = self.shared.size(kind).max(1);
        let align = kind.align(self.platform());
        self.line(&format!("_Alignas({align}) unsigned char {name}[{size}] = {{0}};"));
        name
    }

    /// Write a value at an address: a store of a scalar, a blit of a block.
    ///
    /// # Arguments
    ///
    /// * `at` - The C text of the address.
    /// * `v` - The C text of the value.
    /// * `kind` - Its kind.
    pub(crate) fn put_at(&mut self, at: &str, v: &str, kind: DataKind)
    {
        if is_blob(kind)
        {
            let size = self.shared.size(kind);
            return self.line(&format!("zk_put_block({at}, {v}, {size});"));
        }
        let n = self.shared.mem(kind);
        self.line(&format!("zk_st_{n}({at}, {v});"));
    }

    /// The size of a unit of a heap statement: the kind, or one octet.
    ///
    /// # Arguments
    ///
    /// * `unit` - The kind, when written.
    ///
    /// # Returns
    ///
    /// The size.
    fn unit_size(&self, unit: Option<DataKind>) -> u32
    {
        unit.map_or(1, |k| self.shared.size(k))
    }

    /// The pointer a heap statement reads and the C text that writes a new
    /// one there.
    ///
    /// # Arguments
    ///
    /// * `dest` - Where the pointer lives.
    ///
    /// # Returns
    ///
    /// The C text that reads it, and the start of the C text that writes
    /// it, which the new value and `);` or `;` close.
    fn pointer_at(&mut self, dest: &Dest) -> (String, String, &'static str)
    {
        match dest
        {
            Dest::Slot(p) =>
            {
                let name = self.place(*p);
                (name.clone(), format!("{name} = "), ";")
            }
            Dest::Mem(at) =>
            {
                let a = self.expr(at);
                let a = self.spill(at.kind, a);
                (format!("zk_ld_p({a})"), format!("zk_st_p({a}, "), ");")
            }
        }
    }

    /// Generate an allocate: a new block, or null when there is no room.
    ///
    /// # Arguments
    ///
    /// * `dest` - Where the address goes.
    /// * `count` - The cell count, or the size in octets.
    /// * `unit` - The cell kind, when written.
    fn alloc(&mut self, dest: &Dest, count: &Expr, unit: Option<DataKind>)
    {
        let stride = self.unit_size(unit);
        let (_, write, close) = self.pointer_at(dest);
        let n = self.expr(count);
        self.line(&format!("{write}zk_alloc((zk_umax)({n}), {stride}){close}"));
    }

    /// Generate a resize: the block moves to a new size.
    ///
    /// # Arguments
    ///
    /// * `dest` - Where the address lives.
    /// * `count` - The cell count, or the size in octets.
    /// * `unit` - The cell kind, when written.
    fn resize(&mut self, dest: &Dest, count: &Expr, unit: Option<DataKind>)
    {
        let stride = self.unit_size(unit);
        let (read, write, close) = self.pointer_at(dest);
        let old = self.spill(DataKind::OPAQUE, read);
        let n = self.expr(count);
        self.line(&format!(
            "{write}zk_realloc({old}, (zk_umax)({n}), {stride}){close}"
        ));
    }

    /// Generate a free: the block goes back, and the address becomes null.
    ///
    /// # Arguments
    ///
    /// * `dest` - Where the address lives.
    fn free(&mut self, dest: &Dest)
    {
        let (read, write, close) = self.pointer_at(dest);
        self.line(&format!("free({read});"));
        self.line(&format!("{write}NULL{close}"));
    }

    /// Generate a fill: one value in each cell.
    ///
    /// # Arguments
    ///
    /// * `dst` - The first cell.
    /// * `count` - The cell count.
    /// * `value` - The value.
    fn fill(&mut self, dst: &Expr, count: &Expr, value: &Expr)
    {
        let v = self.operands(&[dst, value, count]);
        let cell = self.blob_temp(value.kind);
        self.put_at(&cell, &v[1], value.kind);
        let size = self.shared.size(value.kind);
        self.line(&format!("zk_fill({}, (zk_umax)({}), {cell}, {size});", v[0], v[2]));
    }

    /// The C text of a call: the arguments run in order, after the function
    /// value when there is one. A list or struct result goes into `into`.
    ///
    /// # Arguments
    ///
    /// * `callee` - What the call runs.
    /// * `args` - The arguments.
    /// * `into` - The slot of a list or struct result.
    ///
    /// # Returns
    ///
    /// The C text of the call.
    pub(crate) fn call_text(
        &mut self, callee: Callee<'_>, args: &[Expr], into: Option<&str>,
    ) -> String
    {
        let mut exprs: Vec<&Expr> = Vec::new();
        do_if!(let Callee::Value(c) = callee, exprs.push(c));
        exprs.extend(args.iter());
        let mut texts = self.operands(&exprs);
        let value = match callee
        {
            Callee::Fn(_) => None,
            Callee::Value(c) => Some((c, texts.remove(0))),
        };
        do_if!(let Some(slot) = into, texts.insert(0, slot.to_string()));
        let Some((c, f)) = value
        else
        {
            let Callee::Fn(n) = callee
            else
            {
                return self.shared.missing("a call");
            };
            return format!("{}({})", func_name(n), texts.join(", "));
        };
        // The value is read once; a closure runs its thunk, which takes the
        // record last.
        let f = self.spill(c.kind, format!("zk_fn_check({f})"));
        let kind = c.kind.as_fn();
        let shapes = kind.map(|k| {
            (self.shared.fn_pointer(k, false), self.shared.fn_pointer(k, true))
        });
        let Some((plain, bound)) = shapes
        else
        {
            return self.shared.missing("a call of a value");
        };
        let args = texts.join(", ");
        let with_record = texts.into_iter().chain([format!("zk_fn_rec({f})")]);
        let with_record: Vec<String> = with_record.collect();
        format!(
            "(zk_fn_bound({f}) ? (({bound})zk_fn_code({f}))({}) : \
             (({plain})zk_fn_code({f}))({args}))",
            with_record.join(", ")
        )
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
    pub(crate) fn ret_of(&self, callee: Callee<'_>) -> Option<DataKind>
    {
        match callee
        {
            Callee::Fn(n) => self.shared.module.funcs[n as usize].ret,
            Callee::Value(c) =>
            {
                c.kind.as_fn().and_then(|f| zakac_ir_builder::ir::fn_shape(f).1)
            }
        }
    }

    /// Generate a call whose result is dropped; a result that holds text is
    /// dropped after the call, as the VM drops it.
    ///
    /// # Arguments
    ///
    /// * `callee` - What the call runs.
    /// * `args` - The arguments.
    fn call_stmt(&mut self, callee: Callee<'_>, args: &[Expr])
    {
        let ret = self.ret_of(callee);
        // A function value given back and left unused is dropped, so a
        // closure frees its record.
        if let Some(kind) = ret.filter(|k| k.is_fn())
        {
            let call = self.call_text(callee, args, None);
            let slot = self.temp(kind);
            let k = self.shared.descr.of(kind);
            self.line(&format!("{slot} = {call};"));
            return self.line(&format!("zk_drop((unsigned char *)&{slot}, {k});"));
        }
        let Some(kind) = ret.filter(|k| is_blob(*k))
        else
        {
            let call = self.call_text(callee, args, None);
            return self.line(&format!("(void){call};"));
        };
        let slot = self.temp(kind);
        let call = self.call_text(callee, args, Some(&slot));
        self.line(&format!("{call};"));
        do_if!(kind.holds_text(), {
            let k = self.shared.descr.of(kind);
            self.line(&format!("zk_drop({slot}, {k});"));
        });
    }

    /// Generate an `if`; an `else if` chain nests in the `else`.
    ///
    /// # Arguments
    ///
    /// * `cond` - The condition.
    /// * `then` - Statements when it holds.
    /// * `els` - Statements when it does not.
    fn if_stmt(&mut self, cond: &Expr, then: &[Stmt], els: &[Stmt])
    {
        let cond = self.expr(cond);
        self.line(&format!("if ({cond})"));
        self.line("{");
        self.braced(then);
        self.line("}");
        ret_if!(els.is_empty());
        self.line("else");
        self.line("{");
        self.braced(els);
        self.line("}");
    }

    /// Generate a loop: `init` once, then the test, the statements and the
    /// update in turn; `continue` jumps to the update.
    ///
    /// # Arguments
    ///
    /// * `init` - Statements before the first test.
    /// * `cond` - The test.
    /// * `update` - Statements after each round.
    /// * `block` - The loop statements.
    fn loop_stmt(&mut self, init: &[Stmt], cond: &Expr, update: &[Stmt], block: &[Stmt])
    {
        self.block(init);
        let label = self.fresh();
        self.line("for (;;)");
        self.line("{");
        self.depth += 1;
        let cond = self.expr(cond);
        self.line(&format!("if (!({cond})) break;"));
        self.in_loop(label, block);
        self.line(&format!("zk_next{label}:;"));
        self.block(update);
        self.depth -= 1;
        self.line("}");
    }

    /// Generate loop statements, with `continue` bound to a label.
    ///
    /// # Arguments
    ///
    /// * `label` - The label number.
    /// * `block` - The statements.
    fn in_loop(&mut self, label: u32, block: &[Stmt])
    {
        self.loops.push(label);
        self.block(block);
        self.loops.pop();
    }

    /// Generate a counted loop. The variable runs from `from` to `to`, both
    /// included. The distance left is worked out in the unsigned kind of
    /// the same width, which gives the true distance even for bounds of each
    /// sign, so a step never moves the variable past the last value.
    ///
    /// # Arguments
    ///
    /// * `var` - The loop variable.
    /// * `from` - The first value.
    /// * `to` - The last value.
    /// * `step` - The step; one when absent.
    /// * `mode` - The direction.
    /// * `block` - The loop statements.
    fn counted(
        &mut self, var: Place, from: &Expr, to: &Expr, step: Option<&Expr>,
        mode: CountMode, block: &[Stmt],
    )
    {
        let kind = from.kind;
        let platform = self.platform();
        let (Some(int), Some(u)) =
            (kind.repr().int_kind(), same_unsigned(kind, platform))
        else
        {
            self.shared.missing(format!("a counted loop over {}", kind.name()));
            return;
        };
        let t = self.shared.kind(kind);
        let v = self.place(var);
        let first = self.expr(from);
        self.line(&format!("{v} = {first};"));
        let label = self.fresh();
        self.line("{");
        self.depth += 1;
        let last = self.bound(&format!("zk_to{label}"), &t, to);
        let one = Value::int(int, 1, platform);
        let step_text = match step
        {
            Some(s) => self.bound(&format!("zk_step{label}"), &t, s),
            None => constant(one, kind, platform).unwrap_or_default(),
        };
        let fixed = step.is_none_or(|s| s.as_const().is_some());
        let check = format!(
            "if (ZK_LANG_CHECKS && !({step_text} > 0)) zk_step_not_positive();"
        );
        do_if!(!fixed, self.line(&check));
        let up = match mode
        {
            CountMode::Increase => Some(true),
            CountMode::Decrease => Some(false),
            CountMode::Count => static_direction(from, to),
        };
        let dir = format!("zk_dir{label}");
        let up_text = up.map_or_else(|| dir.clone(), |u| u8::from(u).to_string());
        do_if!(up.is_none(), self.line(&format!("uint8_t {dir} = {v} <= {last};")));
        let past = format!("({up_text} ? {v} > {last} : {v} < {last})");
        let gap = format!(
            "({up_text} ? ({u})(({u}){last} - ({u}){v}) : ({u})(({u}){v} - \
             ({u}){last}))"
        );
        let next = format!(
            "({t})({up_text} ? ({u})(({u}){v} + ({u}){step_text}) : ({u})(({u}){v} - \
             ({u}){step_text}))"
        );
        self.line(&format!("if (!{past})"));
        self.line("for (;;)");
        self.line("{");
        self.depth += 1;
        self.in_loop(label, block);
        self.line(&format!("zk_next{label}:;"));
        self.line(&format!("if ({gap} < ({u}){step_text}) break;"));
        self.line(&format!("{v} = {next};"));
        self.depth -= 1;
        self.line("}");
        self.depth -= 1;
        self.line("}");
    }

    /// A counted-loop bound, read once: a constant as it is, another value
    /// into a local.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the local.
    /// * `t` - The C kind.
    /// * `e` - The bound.
    ///
    /// # Returns
    ///
    /// The C text that reads it.
    fn bound(&mut self, name: &str, t: &str, e: &Expr) -> String
    {
        let value = self.expr(e);
        ret_if!(e.as_const().is_some(), value);
        self.line(&format!("{t} {name} = {value};"));
        name.into()
    }
}

/// Whether an expression is a fresh value that holds text, which is dropped
/// once read; the rule the VM target follows too.
///
/// # Arguments
///
/// * `e` - The expression.
///
/// # Returns
///
/// `true` for such a value.
pub(crate) fn fresh(e: &Expr) -> bool { fresh_text(e) }

/// The direction of a counting loop whose bounds are both constants.
///
/// # Arguments
///
/// * `from` - The first value.
/// * `to` - The last value.
///
/// # Returns
///
/// `Some(true)` to count up, `Some(false)` to count down, `None` when the
/// direction is known at run time alone.
fn static_direction(from: &Expr, to: &Expr) -> Option<bool>
{
    let (from, to) = (from.as_const()?, to.as_const()?);
    compare(CmpOp::Le, from, to).ok()
}

/// The name of an IR node, for the message of a construct the C target
/// cannot build: the variant name, as its debug text starts.
///
/// # Arguments
///
/// * `node` - The node.
///
/// # Returns
///
/// The name.
pub(crate) fn node_name(node: &impl std::fmt::Debug) -> String
{
    let text = format!("{node:?}");
    text.split(|c: char| !c.is_alphanumeric()).next().unwrap_or_default().to_string()
}
