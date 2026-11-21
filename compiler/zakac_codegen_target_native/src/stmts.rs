//! The lowering of statements, as the C target writes them.

use cranelift_codegen::ir::InstBuilder;
use cranelift_codegen::ir::MemFlagsData;
use cranelift_codegen::ir::Value;
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::types;
use cranelift_module::Module;
use zakac_ast::stmt::CountMode;
use zakac_codegen_target_c::kinds::is_blob;
use zakac_codegen_target_c::options::RunCheck;
use zakac_ir_builder::ir::CmpOp;
use zakac_ir_builder::ir::Dest;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::HaltCause;
use zakac_ir_builder::ir::Place;
use zakac_ir_builder::ir::PrintValue;
use zakac_ir_builder::ir::RingsAct;
use zakac_ir_builder::ir::Stmt;
use zakac_ir_builder::ir::fresh_text;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::value::compare;

use crate::func::FnGen;

/// What a call runs: a function of the program, or a function value.
#[derive(Clone, Copy)]
pub enum Callee<'e>
{
    /// The index of a function.
    Fn(u32),
    /// A function value.
    Value(&'e Expr),
}

impl<M: Module> FnGen<'_, '_, M>
{
    /// Lower a list of statements; code after a return, a break or a stop is
    /// never lowered.
    ///
    /// # Arguments
    ///
    /// * `stmts` - The statements.
    pub fn block(&mut self, stmts: &[Stmt])
    {
        for s in stmts
        {
            ret_if!(!self.live);
            self.stmt(s);
        }
    }

    /// Lower a nested list of statements: a branch or a loop leaves the frame
    /// at a site of its own statements.
    ///
    /// # Arguments
    ///
    /// * `stmts` - The statements.
    fn braced(&mut self, stmts: &[Stmt])
    {
        self.last_site = None;
        self.block(stmts);
        self.last_site = None;
    }

    /// Lower one statement.
    ///
    /// # Arguments
    ///
    /// * `s` - The statement.
    fn stmt(&mut self, s: &Stmt)
    {
        self.mark_site(s.span());
        self.stmt_plain(s);
        let nested =
            matches!(s, Stmt::If { .. } | Stmt::Loop { .. } | Stmt::Counted { .. });
        do_if!(nested, self.last_site = None);
    }

    /// Lower the work of one statement.
    ///
    /// # Arguments
    ///
    /// * `s` - The statement.
    fn stmt_plain(&mut self, s: &Stmt)
    {
        match s
        {
            Stmt::Set { place, value, .. } => self.set(*place, value),
            Stmt::Wait { thread, .. } =>
            {
                let ptr = self.s.ptr;
                let j = self.expr(thread);
                self.rt("zk_thread_drop", &[ptr], &[], &[j]);
            }
            Stmt::Send { pipe, value, .. } => self.send(pipe, value),
            Stmt::Close { pipe, .. } =>
            {
                let ptr = self.s.ptr;
                let p = self.expr(pipe);
                self.rt("zk_pipe_close", &[ptr], &[], &[p]);
            }
            Stmt::Sleep { ms, .. } =>
            {
                let ptr = self.s.ptr;
                let m = self.expr(ms);
                self.rt("zk_sleep", &[ptr], &[], &[m]);
            }
            Stmt::Select { pipes, slots, ms, idle, chosen, .. } =>
            {
                self.select(pipes, slots, ms.as_ref(), *idle, *chosen)
            }
            Stmt::Lock { obj, .. } =>
            {
                let ptr = self.s.ptr;
                let o = self.expr(obj);
                self.rt("zk_lock", &[ptr], &[], &[o]);
            }
            Stmt::AtomicAdd { addr, value, sub, .. } =>
            {
                let ptr = self.s.ptr;
                let a = self.expr(addr);
                let v = self.expr(value);
                let sub = self.b.ins().iconst(types::I32, i64::from(*sub));
                self.rt("zk_atomic_add", &[ptr, ptr, types::I32], &[], &[a, v, sub]);
            }
            Stmt::AtomicSet { addr, value, .. } =>
            {
                let ptr = self.s.ptr;
                let a = self.expr(addr);
                let v = self.expr(value);
                self.rt("zk_atomic_set", &[ptr, ptr], &[], &[a, v]);
            }
            Stmt::Unlock { obj, .. } =>
            {
                let ptr = self.s.ptr;
                let o = self.expr(obj);
                self.rt("zk_unlock", &[ptr], &[], &[o]);
            }
            Stmt::Print { value, .. } => self.print(value),
            Stmt::If { cond, then, els, .. } => self.if_stmt(cond, then, els),
            Stmt::Loop { par: Some(p), .. } => self.par(p),
            Stmt::Loop { init, cond, update, block, .. } =>
            {
                self.loop_stmt(init, cond, update, block)
            }
            Stmt::Counted { var, from, to, step, mode, block, .. } =>
            {
                self.counted(*var, from, to, step.as_ref(), *mode, block)
            }
            Stmt::Break { .. } =>
            {
                let exit = self.loops.last().map(|l| l.1);
                do_if!(let Some(b) = exit, self.jump(b));
            }
            Stmt::Continue { .. } =>
            {
                let next = self.loops.last().map(|l| l.0);
                do_if!(let Some(b) = next, self.jump(b));
            }
            Stmt::Return { value, .. } => self.return_stmt(value.as_ref()),
            Stmt::Call { func, args, .. } => self.call_stmt(Callee::Fn(*func), args),
            Stmt::Foreign { func, args, .. } =>
            {
                let ret = self.s.module.foreign[*func as usize].ret;
                _ = self.foreign(*func, args, ret);
            }
            Stmt::CallValue { callee, args, .. } =>
            {
                self.call_stmt(Callee::Value(callee), args)
            }
            Stmt::Store { addr, value, .. } => self.store_stmt(addr, value),
            Stmt::Halt { cause, value, .. } => self.halt(*cause, value.as_ref()),
            Stmt::Rings { act, octets, .. } => self.rings(*act, octets.as_ref()),
            Stmt::Drop { dest, kind, .. } =>
            {
                let at = self.dest_addr(dest);
                self.drop_at(at, *kind);
            }
            _ => self.heap_or_edit(s),
        }
    }

    /// Lower a statement on the heap, or an edit of a list or text.
    ///
    /// # Arguments
    ///
    /// * `s` - The statement.
    fn heap_or_edit(&mut self, s: &Stmt)
    {
        let ptr = self.s.ptr;
        match s
        {
            Stmt::Alloc { dest, count, unit, .. } => self.alloc(dest, count, *unit),
            Stmt::Resize { dest, count, unit, .. } => self.resize(dest, count, *unit),
            Stmt::Free { dest, .. } => self.free(dest),
            Stmt::Leave { value, grant, .. } => self.leave(value, *grant),
            Stmt::CStore { base, offset, value, .. } =>
            {
                self.c_store(base, *offset, value)
            }
            Stmt::Blit { src, dst, count, unit, overlap, .. } =>
            {
                let stride = self.unit_size(*unit);
                let (s_v, n_v, d_v) =
                    (self.expr(src), self.expr(count), self.expr(dst));
                let n = self.as_u64(n_v, count.kind);
                let stride = self.word(i64::from(stride));
                let ov = self.b.ins().iconst(types::I32, i64::from(*overlap));
                self.rt(
                    "zk_blit",
                    &[ptr, types::I64, ptr, ptr, types::I32],
                    &[],
                    &[s_v, n, d_v, stride, ov],
                );
            }
            Stmt::Fill { dst, count, value, .. } => self.fill(dst, count, value),
            Stmt::AddCell { dest, kind, value, .. } =>
            {
                let cell = self.cell_temp(value);
                let at = self.dest_addr(dest);
                let k = self.descr(*kind);
                self.rt("zk_add_cell", &[ptr, ptr, ptr], &[], &[at, k, cell]);
            }
            Stmt::RemoveCell { dest, kind, .. } =>
            {
                let at = self.dest_addr(dest);
                let k = self.descr(*kind);
                self.rt("zk_remove_cell", &[ptr, ptr], &[], &[at, k]);
            }
            Stmt::SetRow { dest, kind, index, value, .. } =>
            {
                let index = self.cell_temp(index);
                let cell = self.cell_temp(value);
                let at = self.dest_addr(dest);
                let k = self.descr(*kind);
                self.rt(
                    "zk_set_row",
                    &[ptr, ptr, ptr, ptr],
                    &[],
                    &[at, k, index, cell],
                );
            }
            Stmt::RemoveRow { dest, kind, index, .. } =>
            {
                let at = self.dest_addr(dest);
                let (held, v) = self.index_at(index);
                let k = self.descr(*kind);
                self.rt("zk_remove_row", &[ptr, ptr, ptr], &[], &[at, k, held]);
                self.drop_fresh(index, v);
            }
            _ => self.text_edit(s),
        }
    }

    /// Lower an edit of text, or an insert into a plain list.
    ///
    /// # Arguments
    ///
    /// * `s` - The statement.
    fn text_edit(&mut self, s: &Stmt)
    {
        let ptr = self.s.ptr;
        let (i64t, i32t) = (types::I64, types::I32);
        match s
        {
            Stmt::SetChar { dest, index, first, value, .. } =>
            {
                let at = self.dest_addr(dest);
                let i = self.expr(index);
                let c = self.expr(value);
                let i = self.as_i64(i, index.kind);
                let f = self.b.ins().iconst(i32t, i64::from(*first));
                self.rt("zk_set_char", &[ptr, i64t, i32t, i32t], &[], &[at, i, f, c]);
            }
            Stmt::Insert { dest, kind, index, first, value, .. } if !kind.is_text() =>
            {
                let cell = self.cell_temp(value);
                let at = self.dest_addr(dest);
                let i = self.expr(index);
                let i = self.as_i64(i, index.kind);
                let k = self.descr(*kind);
                let f = self.b.ins().iconst(i32t, i64::from(*first));
                self.rt(
                    "zk_insert_cell",
                    &[ptr, ptr, i64t, i32t, ptr],
                    &[],
                    &[at, k, i, f, cell],
                );
            }
            Stmt::Insert { dest, index, first, value, .. } =>
            {
                let at = self.dest_addr(dest);
                let i = self.expr(index);
                let v = self.expr(value);
                let i = self.as_i64(i, index.kind);
                let f = self.b.ins().iconst(i32t, i64::from(*first));
                let char = value.kind == DataKind::Char;
                let (call, vt) =
                    tern!(char, ("zk_insert_char", i32t), ("zk_insert_text", ptr));
                self.rt(call, &[ptr, i64t, i32t, vt], &[], &[at, i, f, v]);
                self.drop_fresh(value, v);
            }
            Stmt::Replace { dest, start, end, chars, value, .. } =>
            {
                let at = self.dest_addr(dest);
                let a = self.expr(start);
                let e = self.expr(end);
                let v = self.expr(value);
                let (a, e) = (self.as_i64(a, start.kind), self.as_i64(e, end.kind));
                let c = self.b.ins().iconst(i32t, i64::from(*chars));
                let char = value.kind == DataKind::Char;
                let (call, vt) =
                    tern!(char, ("zk_replace_char", i32t), ("zk_replace_text", ptr));
                self.rt(call, &[ptr, i64t, i64t, i32t, vt], &[], &[at, a, e, c, v]);
                self.drop_fresh(value, v);
            }
            Stmt::AddText { dest, value, .. } =>
            {
                let at = self.dest_addr(dest);
                let v = self.expr(value);
                let char = value.kind == DataKind::Char;
                let (call, vt) =
                    tern!(char, ("zk_add_char", i32t), ("zk_add_text", ptr));
                self.rt(call, &[ptr, vt], &[], &[at, v]);
                self.drop_fresh(value, v);
            }
            other => self.s.missing(format!("the statement {}", node_name(other))),
        }
    }

    /// Lower a set of a slot: a list or struct is copied into the slot; the
    /// result of a host function is written there.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    /// * `value` - The value.
    fn set(&mut self, place: Place, value: &Expr)
    {
        if let ExprNode::Host { func, args } = &value.node
        {
            let at = self.addr_of(place);
            return self.host_into(at, *func, args, value.kind);
        }
        let v = self.expr(value);
        let at = self.addr_of(place);
        self.put_at(at, v, value.kind);
    }

    /// Lower a store through an address.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address.
    /// * `value` - The value.
    fn store_stmt(&mut self, addr: &Expr, value: &Expr)
    {
        let a = self.expr(addr);
        let v = self.expr(value);
        do_if!(!is_blob(value.kind), self.null_check(a));
        self.put_at(a, v, value.kind);
    }

    /// Lower a return: the value goes back, then the function ends.
    ///
    /// # Arguments
    ///
    /// * `value` - The value, for a function with a result.
    fn return_stmt(&mut self, value: Option<&Expr>)
    {
        let ret = self.func.and_then(|f| f.ret);
        let Some(e) = value
        else
        {
            if self.func.is_none()
            {
                let zero = self.b.ins().iconst(types::I32, 0);
                return self.give_back(zero, None);
            }
            self.leave_call();
            self.b.ins().return_(&[]);
            self.live = false;
            return;
        };
        let v = self.expr(e);
        self.give_back(v, tern!(self.func.is_some(), ret, Some(e.kind)));
    }

    /// Lower a print: the value, then a line break; a fresh value that holds
    /// text is dropped after.
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
                self.put_raw(text.as_bytes());
                self.rt("zk_put_line", &[], &[], &[]);
                return;
            }
            PrintValue::Expr(e) => e,
        };
        let v = self.expr(e);
        self.put_value(v, e.kind);
        self.rt("zk_put_line", &[], &[], &[]);
        self.drop_fresh(e, v);
    }

    /// Write octets as written.
    ///
    /// # Arguments
    ///
    /// * `octets` - The octets.
    pub fn put_raw(&mut self, octets: &[u8])
    {
        let ptr = self.s.ptr;
        let p = self.text_ptr(octets);
        let n = self.word(octets.len() as i64);
        self.rt("zk_put_raw", &[ptr, ptr], &[], &[p, n]);
    }

    /// Write a value as a print writes it, with no line break.
    ///
    /// # Arguments
    ///
    /// * `v` - The value.
    /// * `kind` - Its kind; a member of an enumeration writes its name.
    pub fn put_value(&mut self, v: Value, kind: DataKind)
    {
        let ptr = self.s.ptr;
        if is_blob(kind) || kind.as_enum().is_some()
        {
            let at = self.addressed(v, kind);
            let k = self.descr(kind);
            let zero = self.b.ins().iconst(types::I32, 0);
            self.rt("zk_show", &[ptr, ptr, types::I32], &[], &[at, k, zero]);
            return;
        }
        let (name, t, arg) = match kind
        {
            DataKind::Int(k) if k.signed =>
            {
                ("zk_put_i", types::I64, self.as_i64(v, kind))
            }
            DataKind::Int(_) => ("zk_put_u", types::I64, self.as_u64(v, kind)),
            DataKind::Float(32) => ("zk_put_f32", types::F32, v),
            DataKind::Float(_) => ("zk_put_f64", types::F64, v),
            DataKind::Bool => ("zk_put_bool", types::I8, v),
            DataKind::Char => ("zk_put_char", types::I32, v),
            DataKind::Ptr(_) | DataKind::Null => ("zk_put_ptr", ptr, v),
            DataKind::Fn(_) => ("zk_put_fn", ptr, v),
            k =>
            {
                self.s.missing(format!("a print of {}", k.name()));
                return;
            }
        };
        self.rt(name, &[t], &[], &[arg]);
    }

    /// Lower a stop of the run: a conversion read the part of a result it does
    /// not hold.
    ///
    /// # Arguments
    ///
    /// * `cause` - Which part was read.
    /// * `value` - The error value, when the failure holds one.
    fn halt(&mut self, cause: HaltCause, value: Option<&Expr>)
    {
        // The step of a parallel loop follows the step check, as each counted
        // loop does.
        let unchecked = !self.s.options.checks.has(RunCheck::Step);
        ret_if!(cause == HaltCause::StepNotPositive && unchecked);
        let plain = |code: i64| code;
        let stop = match cause
        {
            HaltCause::NotFault => Some(("zk_not_fault", None)),
            HaltCause::NoData => Some(("zk_no_data", None)),
            HaltCause::NoRow => Some(("zk_plain_trap", Some(plain(22)))),
            HaltCause::WrongVariant => Some(("zk_plain_trap", Some(plain(35)))),
            HaltCause::StepNotPositive => Some(("zk_step_not_positive", None)),
            HaltCause::ThreadsNotPositive => Some(("zk_threads_not_positive", None)),
            HaltCause::ClosedHandle => Some(("zk_closed_handle", None)),
            HaltCause::NullFromC => Some(("zk_plain_trap", Some(plain(10)))),
            HaltCause::NotGood => None,
        };
        if let Some((name, code)) = stop
        {
            let Some(code) = code
            else
            {
                return self.rt_stop(name, &[], &[]);
            };
            let c = self.b.ins().iconst(types::I32, code);
            return self.rt_stop(name, &[types::I32], &[c]);
        }
        let Some(e) = value
        else
        {
            return self.rt_stop("zk_not_good_bare", &[], &[]);
        };
        let ptr = self.s.ptr;
        let v = self.expr(e);
        let at = self.addressed(v, e.kind);
        let k = self.descr(e.kind);
        self.rt_stop("zk_not_good", &[ptr, ptr], &[at, k]);
    }

    /// Let go of each block a value holds.
    ///
    /// # Arguments
    ///
    /// * `at` - The address of the value.
    /// * `kind` - Its kind.
    pub fn drop_at(&mut self, at: Value, kind: DataKind)
    {
        let ptr = self.s.ptr;
        let k = self.descr(kind);
        self.rt("zk_drop", &[ptr, ptr], &[], &[at, k]);
    }

    /// Drop a fresh value that holds text once it is read: the text a format
    /// builds, a call result or a list literal, as the VM drops it.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    /// * `v` - Its value: the address of a block.
    pub fn drop_fresh(&mut self, e: &Expr, v: Value)
    {
        ret_if!(!fresh_text(e));
        self.drop_at(v, e.kind);
    }

    /// The address of the place a heap or edit statement works on.
    ///
    /// # Arguments
    ///
    /// * `dest` - The place.
    ///
    /// # Returns
    ///
    /// The address.
    pub fn dest_addr(&mut self, dest: &Dest) -> Value
    {
        match dest
        {
            Dest::Slot(p) => self.addr_of(*p),
            Dest::Mem(at) => self.expr(at),
        }
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
    /// The address of the cell.
    fn cell_temp(&mut self, value: &Expr) -> Value
    {
        let v = self.expr(value);
        let cell = self.temp(value.kind);
        self.put_at(cell, v, value.kind);
        cell
    }

    /// The address of an index read where it is: a list, a struct or text as
    /// it is, each other value put in a temp of its own.
    ///
    /// # Arguments
    ///
    /// * `index` - The index.
    ///
    /// # Returns
    ///
    /// The address, and the value, which a fresh value is dropped through.
    pub fn index_at(&mut self, index: &Expr) -> (Value, Value)
    {
        let v = self.expr(index);
        ret_if!(is_blob(index.kind), (v, v));
        let held = self.temp(index.kind);
        self.put_at(held, v, index.kind);
        (held, v)
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
        unit.map_or(1, |k| self.s.size(k))
    }

    /// Where a heap statement reads and writes its pointer.
    ///
    /// # Arguments
    ///
    /// * `dest` - Where the pointer lives.
    ///
    /// # Returns
    ///
    /// The address of the pointer.
    fn pointer_at(&mut self, dest: &Dest) -> Value
    {
        match dest
        {
            Dest::Slot(p) => self.addr_of(*p),
            Dest::Mem(at) =>
            {
                let a = self.expr(at);
                self.null_check(a);
                a
            }
        }
    }

    /// Lower an allocate: a new block, or null when there is no room.
    ///
    /// # Arguments
    ///
    /// * `dest` - Where the address goes.
    /// * `count` - The cell count, or the size in octets.
    /// * `unit` - The cell kind, when written.
    fn alloc(&mut self, dest: &Dest, count: &Expr, unit: Option<DataKind>)
    {
        let ptr = self.s.ptr;
        let stride = self.unit_size(unit);
        let at = self.pointer_at(dest);
        let n = self.expr(count);
        let n = self.as_u64(n, count.kind);
        let stride = self.word(i64::from(stride));
        let p = self.rt("zk_alloc", &[types::I64, ptr], &[ptr], &[n, stride]);
        do_if!(let Some(p) = p, self.store(p, at));
    }

    /// Lower a resize: the block moves to a new size.
    ///
    /// # Arguments
    ///
    /// * `dest` - Where the address lives.
    /// * `count` - The cell count, or the size in octets.
    /// * `unit` - The cell kind, when written.
    fn resize(&mut self, dest: &Dest, count: &Expr, unit: Option<DataKind>)
    {
        let ptr = self.s.ptr;
        let stride = self.unit_size(unit);
        let at = self.pointer_at(dest);
        let old = self.load(ptr, at);
        let n = self.expr(count);
        let n = self.as_u64(n, count.kind);
        let stride = self.word(i64::from(stride));
        let p =
            self.rt("zk_realloc", &[ptr, types::I64, ptr], &[ptr], &[old, n, stride]);
        do_if!(let Some(p) = p, self.store(p, at));
    }

    /// Lower a free: the block goes back, and the address becomes null.
    ///
    /// # Arguments
    ///
    /// * `dest` - Where the address lives.
    fn free(&mut self, dest: &Dest)
    {
        let ptr = self.s.ptr;
        let at = self.pointer_at(dest);
        let p = self.load(ptr, at);
        self.rt("zk_raw_free", &[ptr], &[], &[p]);
        let null = self.word(0);
        self.store(null, at);
    }

    /// Lower a write of a field of a struct C keeps.
    ///
    /// # Arguments
    ///
    /// * `base` - The address of the C struct.
    /// * `offset` - The offset of the field.
    /// * `value` - The value.
    fn c_store(&mut self, base: &Expr, offset: u32, value: &Expr)
    {
        let ptr = self.s.ptr;
        let b = self.expr(base);
        let v = self.expr(value);
        let at = self.addressed(v, value.kind);
        let to = self.b.ins().iadd_imm_s(b, i64::from(offset));
        let size = self.word(i64::from(self.s.size(value.kind)));
        self.rt("zk_put_block", &[ptr, ptr, ptr], &[], &[to, at, size]);
    }

    /// Lower a statement of the ring collector: a call of the runtime, with
    /// the octets of a threshold.
    ///
    /// # Arguments
    ///
    /// * `act` - What it does.
    /// * `octets` - The octets of a threshold; `None` for the other acts.
    fn rings(&mut self, act: RingsAct, octets: Option<&Expr>)
    {
        let v = match octets
        {
            Some(o) =>
            {
                let v = self.expr(o);
                self.as_u64(v, o.kind)
            }
            None => self.b.ins().iconst(types::I64, 0),
        };
        let act = self.b.ins().iconst(types::I8, i64::from(act as u8));
        self.rt("zk_rings", &[types::I8, types::I64], &[], &[act, v]);
    }

    /// Lower a fill: one value in each cell.
    ///
    /// # Arguments
    ///
    /// * `dst` - The first cell.
    /// * `count` - The cell count.
    /// * `value` - The value.
    fn fill(&mut self, dst: &Expr, count: &Expr, value: &Expr)
    {
        let ptr = self.s.ptr;
        let d = self.expr(dst);
        let v = self.expr(value);
        let n = self.expr(count);
        let cell = self.temp(value.kind);
        self.put_at(cell, v, value.kind);
        let n = self.as_u64(n, count.kind);
        let size = self.word(i64::from(self.s.size(value.kind)));
        self.rt("zk_fill", &[ptr, types::I64, ptr, ptr], &[], &[d, n, cell, size]);
    }

    /// Lower a call whose result is dropped; a result that holds text, or a
    /// function value, is dropped after the call, as the VM drops it.
    ///
    /// # Arguments
    ///
    /// * `callee` - What the call runs.
    /// * `args` - The arguments.
    fn call_stmt(&mut self, callee: Callee<'_>, args: &[Expr])
    {
        let ret = self.ret_of(callee);
        let v = self.call(callee, args);
        let Some(kind) = ret
        else
        {
            return;
        };
        if kind.is_fn()
        {
            let at = self.addressed(v, kind);
            return self.drop_at(at, kind);
        }
        do_if!(is_blob(kind) && kind.holds_text(), self.drop_at(v, kind));
    }

    /// Lower an `if` and its `else` branch.
    ///
    /// # Arguments
    ///
    /// * `cond` - The condition.
    /// * `then` - Statements when it holds.
    /// * `els` - Statements when it does not.
    fn if_stmt(
        &mut self, cond: &zakac_ir_builder::ir::Expr, then: &[Stmt], els: &[Stmt],
    )
    {
        let c = self.expr(cond);
        let (t, e, after) =
            (self.b.create_block(), self.b.create_block(), self.b.create_block());
        self.branch(c, t, e);
        self.enter_block(t);
        self.braced(then);
        self.jump(after);
        self.enter_block(e);
        self.braced(els);
        self.jump(after);
        self.enter_block(after);
    }

    /// The address of the flag a collection sets to halt the running
    /// OS threads, read before a loop of a program with threads.
    ///
    /// # Returns
    ///
    /// The address, or `None` for a program with no threads.
    fn halt_flag(&mut self) -> Option<Value>
    {
        ret_if!(!self.s.safe_points || !self.live, None);
        let ptr = self.s.ptr;
        self.rt("zk_halt_flag", &[], &[ptr], &[])
    }

    /// A safe point of the ring collector at the head of a round: one atomic
    /// load of the flag, and a halt in the runtime once it is set.
    ///
    /// # Arguments
    ///
    /// * `flag` - The address of the flag, or `None` for a program with no
    ///   threads.
    fn safe_point(&mut self, flag: Option<Value>)
    {
        let Some(flag) = flag.filter(|_| self.live)
        else
        {
            return;
        };
        let set = self.b.ins().atomic_load(types::I8, MemFlagsData::trusted(), flag);
        self.when(set, |g| _ = g.rt("zk_safe_point", &[], &[], &[]));
    }

    /// Lower a loop: `init` once, then the test, the statements and the update
    /// in turn; `continue` jumps to the update.
    ///
    /// # Arguments
    ///
    /// * `init` - Statements before the first test.
    /// * `cond` - The test.
    /// * `update` - Statements after each round.
    /// * `block` - The loop statements.
    fn loop_stmt(&mut self, init: &[Stmt], cond: &Expr, update: &[Stmt], block: &[Stmt])
    {
        let span = cond.span;
        self.block(init);
        let flag = self.halt_flag();
        let head = self.b.create_block();
        let (inner, next, exit) =
            (self.b.create_block(), self.b.create_block(), self.b.create_block());
        self.jump(head);
        self.enter_block(head);
        self.safe_point(flag);
        self.last_site = None;
        self.mark_site(span);
        let c = self.expr(cond);
        self.branch(c, inner, exit);
        self.enter_block(inner);
        self.loops.push((next, exit));
        self.braced(block);
        self.loops.pop();
        self.jump(next);
        self.enter_block(next);
        self.block(update);
        self.jump(head);
        self.enter_block(exit);
    }

    /// Lower a counted loop. The variable runs from `from` to `to`, both
    /// included. The distance left is worked out in the unsigned kind of the
    /// same width, which gives the true distance even for bounds of each sign,
    /// so a step never moves the variable past the last value.
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
        let Some(int) = kind.repr().int_kind()
        else
        {
            return self.s.missing(format!("a counted loop over {}", kind.name()));
        };
        let t = self.s.clif_of(kind);
        let first = self.expr(from);
        let at = self.addr_of(var);
        self.store(first, at);
        let last = self.expr(to);
        let step_v = match step
        {
            Some(s) => self.expr(s),
            None => self.b.ins().iconst(t, 1),
        };
        let fixed = step.is_none_or(|s| s.as_const().is_some());
        if !fixed && self.s.options.checks.has(RunCheck::Step)
        {
            let cc = tern!(int.signed, IntCC::SignedLessThanOrEqual, IntCC::Equal);
            let bad = self.b.ins().icmp_imm_s(cc, step_v, 0);
            self.when(bad, |g| g.rt_stop("zk_step_not_positive", &[], &[]));
        }
        let (le, gt, lt) = tern!(
            int.signed,
            (
                IntCC::SignedLessThanOrEqual,
                IntCC::SignedGreaterThan,
                IntCC::SignedLessThan
            ),
            (
                IntCC::UnsignedLessThanOrEqual,
                IntCC::UnsignedGreaterThan,
                IntCC::UnsignedLessThan
            )
        );
        let fixed_dir = match mode
        {
            CountMode::Increase => Some(true),
            CountMode::Decrease => Some(false),
            CountMode::Count => static_direction(from, to),
        };
        let v0 = self.load(t, at);
        let up = match fixed_dir
        {
            Some(u) => self.b.ins().iconst(types::I8, i64::from(u)),
            None => self.b.ins().icmp(le, v0, last),
        };
        let (above, below) =
            (self.b.ins().icmp(gt, v0, last), self.b.ins().icmp(lt, v0, last));
        let past = self.b.ins().select(up, above, below);
        let flag = self.halt_flag();
        let (inner, next, step_b, exit) = (
            self.b.create_block(),
            self.b.create_block(),
            self.b.create_block(),
            self.b.create_block(),
        );
        self.branch(past, exit, inner);
        self.enter_block(inner);
        self.safe_point(flag);
        self.loops.push((next, exit));
        self.braced(block);
        self.loops.pop();
        self.jump(next);
        self.enter_block(next);
        if self.live
        {
            let v = self.load(t, at);
            let (up_gap, down_gap) =
                (self.b.ins().isub(last, v), self.b.ins().isub(v, last));
            let gap = self.b.ins().select(up, up_gap, down_gap);
            let short = self.b.ins().icmp(IntCC::UnsignedLessThan, gap, step_v);
            self.branch(short, exit, step_b);
            self.enter_block(step_b);
            let (plus, minus) =
                (self.b.ins().iadd(v, step_v), self.b.ins().isub(v, step_v));
            let moved = self.b.ins().select(up, plus, minus);
            self.store(moved, at);
            self.jump(inner);
        }
        self.enter_block(exit);
    }
}

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

/// The name of an IR node, for the message of a construct the target cannot
/// build: the variant name, as its debug text starts.
///
/// # Arguments
///
/// * `node` - The node.
///
/// # Returns
///
/// The name.
pub fn node_name(node: &impl std::fmt::Debug) -> String
{
    let text = format!("{node:?}");
    text.split(|c: char| !c.is_alphanumeric()).next().unwrap_or_default().to_string()
}
