//! Jobs and channels in Cranelift code: a spawn fills the data block of the
//! job, which each argument moves into, and hands it to the runtime with the
//! step this file writes for the function. The guard the step runs under is
//! C, which the build compiles beside the object.

use cranelift_codegen::ir::AbiParam;
use cranelift_codegen::ir::InstBuilder;
use cranelift_codegen::ir::Value;
use cranelift_codegen::ir::types;
use cranelift_module::FuncId;
use cranelift_module::Linkage;
use cranelift_module::Module;
use zakac_codegen_target_c::jobs::part_offsets;
use zakac_codegen_target_c::jobs::parts;
use zakac_codegen_target_c::kinds::is_blob;
use zakac_ir_builder::ir::Expr;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_semcheck::kind::DataKind;

use crate::emit::Shared;
use crate::func::FnGen;

impl<M: Module> Shared<'_, M>
{
    /// The step of a job that runs a function, declared on its first use.
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
        // The JIT gives a program with a job to the VM.
        do_if!(self.jit, self.missing("a job"));
        let known = self.steps.iter().find(|(g, _)| *g == f).map(|(_, id)| *id);
        ret_if!(let Some(id) = known, id);
        let mut sig = self.m.make_signature();
        sig.params.push(AbiParam::new(self.ptr));
        let name = format!("zk_job_step_{f}");
        let id = self.m.declare_function(&name, Linkage::Local, &sig).expect("a step");
        self.steps.push((f, id));
        id
    }

    /// A function of the C glue of jobs, under its plain name, declared on its
    /// first use.
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
    ///
    /// # Returns
    ///
    /// The number of the job.
    pub fn spawn(&mut self, f: u32, args: &[Expr]) -> Value
    {
        let ptr = self.s.ptr;
        let kinds = parts(self.s.module, f);
        let (offsets, size) = part_offsets(&kinds, self.s.platform());
        let vals: Vec<Value> = args.iter().map(|a| self.expr(a)).collect();
        let size = self.word(i64::from(size));
        let data = self.rt("zk_job_data", &[ptr], &[ptr], &[size]).unwrap_or(size);
        for ((v, arg), at) in vals.into_iter().zip(args).zip(offsets.iter().skip(1))
        {
            let place = self.b.ins().iadd_imm_u(data, i64::from(*at));
            self.put_at(place, v, arg.kind);
        }
        let step = self.s.step_of(f);
        let step = self.addr_of_fn(step);
        let guard = self.s.glue_fn("zk_job_guard", &[ptr, ptr], &[types::I32]);
        let guard = self.addr_of_fn(guard);
        let leave = self.s.glue_fn("zk_job_leave", &[types::I32], &[]);
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
        self.rt("zk_job_start", &params, &[ptr], &args).unwrap_or(data)
    }

    /// A wait that takes the result of a job: a duplicate of it, which the job
    /// keeps to the end of the program.
    ///
    /// # Arguments
    ///
    /// * `job` - The job.
    /// * `kind` - The result kind.
    ///
    /// # Returns
    ///
    /// The value.
    pub fn wait(&mut self, job: &Expr, kind: DataKind) -> Value
    {
        let ptr = self.s.ptr;
        let j = self.expr(job);
        let at = self.rt("zk_job_wait", &[ptr], &[ptr], &[j]).unwrap_or(j);
        ret_if!(!is_blob(kind), self.load(self.s.clif_of(kind), at));
        let t = self.temp(kind);
        self.put_at(t, at, kind);
        if kind.holds_text()
        {
            let d = self.descr(kind);
            self.rt("zk_dup_seqs", &[ptr, ptr], &[], &[t, d]);
        }
        t
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

    /// The code of the step of a job that runs a function: it calls the
    /// function with the arguments of the data block, which the function
    /// drops as it returns, and writes the result at its start.
    ///
    /// # Arguments
    ///
    /// * `f` - The function.
    pub fn job_step(&mut self, f: u32)
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
