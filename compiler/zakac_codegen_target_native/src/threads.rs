//! Threads and channels in Cranelift code: a spawn fills the data block of the
//! thread, which each argument moves into, and hands it to the runtime with the
//! step this file writes for the function. The guard the step runs under is
//! C, which the build compiles beside the object.

use cranelift_codegen::ir::AbiParam;
use cranelift_codegen::ir::InstBuilder;
use cranelift_codegen::ir::Value;
use cranelift_codegen::ir::types;
use cranelift_module::FuncId;
use cranelift_module::Linkage;
use cranelift_module::Module;
use zakac_codegen_target_c::kinds::is_blob;
use zakac_codegen_target_c::threads::part_offsets;
use zakac_codegen_target_c::threads::parts;
use zakac_codegen_target_c::threads::round_parts;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::Par;
use zakac_ir_builder::ir::ParMap;
use zakac_ir_builder::ir::ParRounds;
use zakac_ir_builder::ir::Place;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;

use crate::emit::Shared;
use crate::func::FnGen;

impl<M: Module> Shared<'_, M>
{
    /// The step of a thread that runs a function, declared on its first use.
    ///
    /// # Arguments
    ///
    /// * `f` - The function.
    ///
    /// # Returns
    ///
    /// The step.
    pub fn step_of(&mut self, f: u32) -> FuncId
    {
        let known = self.steps.iter().find(|(g, _)| *g == f).map(|(_, id)| *id);
        ret_if!(let Some(id) = known, id);
        let mut sig = self.m.make_signature();
        sig.params.push(AbiParam::new(self.ptr));
        let name = format!("zk_thread_step_{f}");
        let id = self.m.declare_function(&name, Linkage::Local, &sig).expect("a step");
        self.steps.push((f, id));
        id
    }

    /// The step of a parallel map through a function, declared on its first
    /// use.
    ///
    /// # Arguments
    ///
    /// * `f` - The function.
    ///
    /// # Returns
    ///
    /// The step.
    pub fn map_step_of(&mut self, f: u32) -> FuncId
    {
        let known = self.map_steps.iter().find(|(g, _)| *g == f).map(|(_, id)| *id);
        ret_if!(let Some(id) = known, id);
        let mut sig = self.m.make_signature();
        sig.params.extend([AbiParam::new(self.ptr), AbiParam::new(self.ptr)]);
        let name = format!("zk_map_step_{f}");
        let id = self.m.declare_function(&name, Linkage::Local, &sig).expect("a step");
        self.map_steps.push((f, id));
        id
    }

    /// The step of the rounds of a parallel loop through their function,
    /// declared on its first use.
    ///
    /// # Arguments
    ///
    /// * `f` - The function of a round.
    ///
    /// # Returns
    ///
    /// The step.
    pub fn round_step_of(&mut self, f: u32) -> FuncId
    {
        let known = self.round_steps.iter().find(|(g, _)| *g == f).map(|(_, id)| *id);
        ret_if!(let Some(id) = known, id);
        let mut sig = self.m.make_signature();
        sig.params.extend([AbiParam::new(self.ptr), AbiParam::new(self.ptr)]);
        let name = format!("zk_round_step_{f}");
        let id = self.m.declare_function(&name, Linkage::Local, &sig).expect("a step");
        self.round_steps.push((f, id));
        id
    }

    /// A function of the C glue of threads, under its plain name, declared on
    /// its first use.
    ///
    /// # Arguments
    ///
    /// * `name` - Its name.
    /// * `params` - The CLIF kind of each parameter.
    /// * `rets` - The CLIF kind of its result.
    ///
    /// # Returns
    ///
    /// The function.
    fn glue_fn(
        &mut self, name: &str, params: &[types::Type], rets: &[types::Type],
    ) -> FuncId
    {
        let mut sig = self.m.make_signature();
        sig.params.extend(params.iter().map(|t| AbiParam::new(*t)));
        sig.returns.extend(rets.iter().map(|t| AbiParam::new(*t)));
        self.m.declare_function(name, Linkage::Import, &sig).expect("a glue name")
    }
}

impl<M: Module> FnGen<'_, '_, M>
{
    /// The address of a function, from its id.
    ///
    /// # Arguments
    ///
    /// * `id` - The function.
    ///
    /// # Returns
    ///
    /// The address.
    fn addr_of_fn(&mut self, id: FuncId) -> Value
    {
        let r = self.func_ref(id);
        let ptr = self.s.ptr;
        self.b.ins().func_addr(ptr, r)
    }

    /// A spawn: a data block that each argument moves into, handed to the
    /// runtime with the step of the function.
    ///
    /// # Arguments
    ///
    /// * `f` - The function.
    /// * `args` - The arguments, owned.
    /// * `apart` - Whether the call asks for an OS thread of its own.
    ///
    /// # Returns
    ///
    /// The number of the thread.
    pub fn spawn(&mut self, f: u32, args: &[Expr], apart: bool) -> Value
    {
        let ptr = self.s.ptr;
        let kinds = parts(self.s.module, f);
        let (offsets, size) = part_offsets(&kinds, self.s.platform());
        let vals: Vec<Value> = args.iter().map(|a| self.expr(a)).collect();
        let size = self.word(i64::from(size));
        let data = self.rt("zk_thread_data", &[ptr], &[ptr], &[size]).unwrap_or(size);
        for ((v, arg), at) in vals.into_iter().zip(args).zip(offsets.iter().skip(1))
        {
            let place = self.b.ins().iadd_imm_u(data, i64::from(*at));
            self.put_at(place, v, arg.kind);
        }
        let step = self.s.step_of(f);
        let step = self.addr_of_fn(step);
        let guard = self.s.glue_fn("zk_thread_guard", &[ptr, ptr], &[types::I32]);
        let guard = self.addr_of_fn(guard);
        let leave = self.s.glue_fn("zk_thread_leave", &[types::I32], &[]);
        let leave = self.addr_of_fn(leave);
        let ret = match kinds[0]
        {
            Some(k) => self.descr(k),
            None => self.word(0),
        };
        let abort =
            self.b.ins().iconst(types::I32, i64::from(self.s.options.trap_abort));
        let params = [ptr, ptr, ptr, ptr, ptr, types::I32];
        let args = [guard, leave, step, data, ret, abort];
        let light = !apart && self.s.light.contains(&f);
        let start = tern!(light, "zk_task_start", "zk_thread_start");
        self.rt(start, &params, &[ptr], &args).unwrap_or(data)
    }

    /// A wait that takes the result of a thread: the result moves out of the
    /// data block of the thread, which the runtime then frees.
    ///
    /// # Arguments
    ///
    /// * `thread` - The thread.
    /// * `kind` - The result kind.
    ///
    /// # Returns
    ///
    /// The value.
    pub fn wait(&mut self, thread: &Expr, kind: DataKind) -> Value
    {
        let ptr = self.s.ptr;
        let j = self.expr(thread);
        let at = self.rt("zk_thread_wait", &[ptr], &[ptr], &[j]).unwrap_or(j);
        // The result moves out, and the runtime frees the block after.
        let value = if is_blob(kind)
        {
            let t = self.temp(kind);
            self.put_at(t, at, kind);
            t
        }
        else
        {
            self.load(self.s.clif_of(kind), at)
        };
        self.rt("zk_thread_taken", &[ptr], &[], &[j]);
        value
    }

    /// A receive: the optional, in a temp of its own.
    ///
    /// # Arguments
    ///
    /// * `pipe` - The channel.
    /// * `kind` - The optional kind.
    ///
    /// # Returns
    ///
    /// The address of the optional.
    pub fn receive(&mut self, pipe: &Expr, kind: DataKind) -> Value
    {
        let ptr = self.s.ptr;
        let p = self.expr(pipe);
        let t = self.temp(kind);
        let d = self.descr(kind);
        self.rt("zk_pipe_receive", &[ptr, ptr, ptr], &[], &[p, t, d]);
        t
    }

    /// A wait on several channels: three words for each channel on the stack
    /// (its number, the address of its optional and the kind of the
    /// optional), then the call that gives the place of the chosen one.
    ///
    /// # Arguments
    ///
    /// * `pipes` - The channels.
    /// * `slots` - The optional of each.
    /// * `ms` - The time limit in milliseconds, or `None`.
    /// * `idle` - Whether the wait does not wait.
    /// * `chosen` - The slot of the chosen place.
    pub fn select(
        &mut self, pipes: &[Expr], slots: &[Place], ms: Option<&Expr>, idle: bool,
        chosen: Place,
    )
    {
        let ptr = self.s.ptr;
        let word = ptr.bytes();
        let words = self.slot(3 * word * pipes.len() as u32, word);
        let base = self.b.ins().stack_addr(ptr, words, 0);
        for (i, (pipe, slot)) in pipes.iter().zip(slots).enumerate()
        {
            let kind = self.place_kind(*slot);
            let parts = [self.expr(pipe), self.addr_of(*slot), self.descr(kind)];
            parts.into_iter().enumerate().for_each(|(k, v)| {
                let at =
                    self.b.ins().iadd_imm_u(base, i64::from(word) * (3 * i + k) as i64);
                self.store(v, at);
            });
        }
        let m = ms.map(|m| self.expr(m));
        let m = m.unwrap_or_else(|| self.b.ins().iconst(ptr, 0));
        // The modes of `ZkSelect` in the runtime: waits 0, timed 1, idle 2.
        let mode = tern!(ms.is_some(), 1, tern!(idle, 2, 0));
        let mode = self.b.ins().iconst(types::I32, mode);
        let n = self.b.ins().iconst(ptr, pipes.len() as i64);
        let params = [ptr, ptr, ptr, types::I32];
        let got = self.rt("zk_select", &params, &[ptr], &[base, n, m, mode]);
        let got = got.expect("zk_select gives the chosen place");
        let at = self.addr_of(chosen);
        self.store(got, at);
    }

    /// A send: the value moves into the channel, and zeros take its place.
    ///
    /// # Arguments
    ///
    /// * `pipe` - The channel.
    /// * `value` - The value, owned.
    pub fn send(&mut self, pipe: &Expr, value: &Expr)
    {
        let ptr = self.s.ptr;
        let p = self.expr(pipe);
        let v = self.expr(value);
        let at = self.addressed(v, value.kind);
        let d = self.descr(value.kind);
        self.rt("zk_pipe_send", &[ptr, ptr, ptr], &[], &[p, at, d]);
    }

    /// What runs at once in place of a loop: a parallel map or the rounds of
    /// a parallel loop.
    ///
    /// # Arguments
    ///
    /// * `par` - The map or the rounds.
    pub fn par(&mut self, par: &Par)
    {
        match par
        {
            Par::Map(p) => self.par_map(p),
            Par::Rounds(p) => self.par_rounds(p),
        }
    }

    /// The guard of a thread, the function that jumps back to it, and whether a
    /// trap stops the whole program, as the runtime takes them.
    ///
    /// # Returns
    ///
    /// The three values.
    fn guard_args(&mut self) -> [Value; 3]
    {
        let ptr = self.s.ptr;
        let guard = self.s.glue_fn("zk_thread_guard", &[ptr, ptr], &[types::I32]);
        let guard = self.addr_of_fn(guard);
        let leave = self.s.glue_fn("zk_thread_leave", &[types::I32], &[]);
        let leave = self.addr_of_fn(leave);
        let abort =
            self.b.ins().iconst(types::I32, i64::from(self.s.options.trap_abort));
        [guard, leave, abort]
    }

    /// The count of threads a parallel loop or map was given, 0 for one
    /// thread for each core.
    ///
    /// # Arguments
    ///
    /// * `threads` - The slot of the count, or `None`.
    ///
    /// # Returns
    ///
    /// The count.
    fn threads_arg(&mut self, threads: Option<Place>) -> Value
    {
        match threads
        {
            Some(t) => self.read(t),
            None => self.word(0),
        }
    }

    /// A parallel map that runs its rounds at once, in place of its loop.
    ///
    /// # Arguments
    ///
    /// * `p` - The map.
    fn par_map(&mut self, p: &ParMap)
    {
        let ptr = self.s.ptr;
        let src = self.addr_of(p.src);
        let out = self.addr_of(p.out);
        let (sk, ok) = (self.descr(p.src_kind), self.descr(p.out_kind));
        let step = self.s.map_step_of(p.func);
        let step = self.addr_of_fn(step);
        let [guard, leave, abort] = self.guard_args();
        let threads = self.threads_arg(p.threads);
        let params = [ptr, ptr, ptr, ptr, ptr, ptr, ptr, types::I32, ptr];
        let args = [src, sk, out, ok, step, guard, leave, abort, threads];
        self.rt("zk_par_map", &params, &[], &args);
    }

    /// The rounds of a parallel loop that run at once, in place of its loop: a
    /// data block of the arguments the rounds share, handed to the runtime
    /// with the step of the round.
    ///
    /// # Arguments
    ///
    /// * `p` - The rounds.
    fn par_rounds(&mut self, p: &ParRounds)
    {
        let ptr = self.s.ptr;
        let (each, size) = round_parts(self.s.module, p.func, self.s.platform());
        let vals: Vec<Value> = p.args.iter().map(|a| self.expr(a)).collect();
        let size = self.word(i64::from(size));
        let data = self.rt("zk_thread_data", &[ptr], &[ptr], &[size]).unwrap_or(size);
        for (v, (k, at)) in vals.into_iter().zip(&each)
        {
            let place = self.b.ins().iadd_imm_u(data, i64::from(*at));
            self.put_at(place, v, *k);
        }
        let count = self.read(p.count);
        let step = self.s.round_step_of(p.func);
        let step = self.addr_of_fn(step);
        let [guard, leave, abort] = self.guard_args();
        let threads = self.threads_arg(p.threads);
        let params = [ptr, ptr, ptr, ptr, ptr, types::I32, ptr];
        let args = [count, step, data, guard, leave, abort, threads];
        self.rt("zk_par_for", &params, &[], &args);
    }

    /// The code of the step of a round of a parallel loop: it calls the
    /// function of the round with the number of the round and the arguments
    /// of the data block.
    ///
    /// # Arguments
    ///
    /// * `f` - The function of a round.
    pub fn round_step(&mut self, f: u32)
    {
        let first = self.b.current_block().expect("the first block");
        let &[data, i] = self.b.block_params(first)
        else
        {
            return;
        };
        let (each, _) = round_parts(self.s.module, f, self.s.platform());
        let mut vals = vec![i];
        for (k, at) in each
        {
            let place = self.b.ins().iadd_imm_u(data, i64::from(at));
            vals.push(self.load(self.s.clif_of(k), place));
        }
        let id = self.s.funcs[f as usize];
        let r = self.func_ref(id);
        self.b.ins().call(r, &vals);
        self.b.ins().return_(&[]);
        self.live = false;
    }

    /// The code of the step of a parallel map through a function: it lends
    /// the cell to the function, as a call does, and writes the result.
    ///
    /// # Arguments
    ///
    /// * `f` - The function.
    pub fn map_step(&mut self, f: u32)
    {
        let first = self.b.current_block().expect("the first block");
        let &[cell, out] = self.b.block_params(first)
        else
        {
            return;
        };
        let ptr = self.s.ptr;
        let kinds = parts(self.s.module, f);
        let (ret, param) = (kinds[0], kinds[1].expect("a map takes one parameter"));
        let arg = match is_blob(param)
        {
            true =>
            {
                let lent = self.temp(param);
                self.put_at(lent, cell, param);
                if param.holds_text()
                {
                    let d = self.descr(param);
                    self.rt("zk_lend_seqs", &[ptr, ptr], &[], &[lent, d]);
                }
                lent
            }
            false => self.load(self.s.clif_of(param), cell),
        };
        let mut vals: Vec<Value> =
            ret.filter(|k| is_blob(*k)).map(|_| out).into_iter().collect();
        vals.push(arg);
        let id = self.s.funcs[f as usize];
        let r = self.func_ref(id);
        let inst = self.b.ins().call(r, &vals);
        let result = self.b.inst_results(inst).first().copied();
        do_if!(let Some(v) = result.filter(|_| ret.is_some_and(|k| !is_blob(k))), self.store(v, out));
        self.b.ins().return_(&[]);
        self.live = false;
    }

    /// The code of the step of a thread that runs a function: it calls the
    /// function with the arguments of the data block, which the function
    /// drops as it returns, and writes the result at its start.
    ///
    /// # Arguments
    ///
    /// * `f` - The function.
    pub fn thread_step(&mut self, f: u32)
    {
        let first = self.b.current_block().expect("the first block");
        let &[data] = self.b.block_params(first)
        else
        {
            return;
        };
        let kinds = parts(self.s.module, f);
        let (offsets, _) = part_offsets(&kinds, self.s.platform());
        let ret = kinds[0];
        let mut vals: Vec<Value> =
            ret.filter(|k| is_blob(*k)).map(|_| data).into_iter().collect();
        for (k, at) in kinds.iter().zip(&offsets).skip(1)
        {
            let k = k.expect("a parameter has a kind");
            let place = self.b.ins().iadd_imm_u(data, i64::from(*at));
            let v = if is_blob(k)
            {
                place
            }
            else
            {
                self.load(self.s.clif_of(k), place)
            };
            vals.push(v);
        }
        let id = self.s.funcs[f as usize];
        let r = self.func_ref(id);
        let inst = self.b.ins().call(r, &vals);
        let result = self.b.inst_results(inst).first().copied();
        do_if!(let Some(v) = result.filter(|_| ret.is_some_and(|k| !is_blob(k))), self.store(v, data));
        self.b.ins().return_(&[]);
        self.live = false;
    }
}
