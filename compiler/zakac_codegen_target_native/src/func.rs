//! The generator of one function: where each local lives, the blocks of its
//! control flow, and the helpers the statements and expressions share.

use std::collections::HashMap;
use std::collections::HashSet;

use cranelift_codegen::ir::Block;
use cranelift_codegen::ir::FuncRef;
use cranelift_codegen::ir::Function;
use cranelift_codegen::ir::GlobalValue;
use cranelift_codegen::ir::InstBuilder;
use cranelift_codegen::ir::MemFlagsData;
use cranelift_codegen::ir::StackSlot;
use cranelift_codegen::ir::StackSlotData;
use cranelift_codegen::ir::StackSlotKind;
use cranelift_codegen::ir::TrapCode;
use cranelift_codegen::ir::Type;
use cranelift_codegen::ir::Value;
use cranelift_codegen::ir::types;
use cranelift_frontend::FunctionBuilder;
use cranelift_frontend::FunctionBuilderContext;
use cranelift_module::DataId;
use cranelift_module::FuncId;
use cranelift_module::Module;
use zakac_codegen_target_c::exports::init_stmts;
use zakac_codegen_target_c::kinds::is_blob;
use zakac_codegen_target_c::options::RunCheck;
use zakac_ir_builder::ir::Func;
use zakac_ir_builder::ir::Place;
use zakac_ir_builder::ir::Stmt;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_semcheck::kind::DataKind;
use zakac_span::span::Span;

/// The functions of the runtime that take the site of the expression that
/// calls them as their last argument, for a trap inside them; [`FnGen::rt`]
/// adds it.
const SITED: &[&str] = &[
    "zk_cell_of",
    "zk_slice_cell",
    "zk_object_cell",
    "zk_char_at",
    "zk_char_at_octet",
    "zk_check_utf8",
    "zk_make_slice",
    "zk_ptr_add",
    "zk_from_c",
    "zk_text_from_c",
    "zk_text_of_c",
    "zk_cb_handle",
    "zk_cb_opt_handle",
    "zk_fn_c",
    "zk_fn_check",
    "zk_pipe_receive",
    "zk_thread_wait",
    "zk_par_map",
];

/// The site of no source region, `NO_SITE` of the runtime: a check given it
/// stops at the site of the statement.
const NO_SITE: u32 = u32::MAX;

use crate::emit::Shared;

/// Where a local lives.
#[derive(Clone, Copy)]
pub enum Home
{
    /// A stack slot of its own.
    Slot(StackSlot),
    /// The storage of the caller, through an address: a list or struct
    /// parameter the function never writes.
    Ptr(Value),
}

/// The generator of one function, or of the main function.
pub struct FnGen<'s, 'm, M: Module>
{
    /// State shared across functions.
    pub s: &'s mut Shared<'m, M>,
    /// The builder of the function.
    pub b: FunctionBuilder<'s>,
    /// The function, or `None` for the top-level statements.
    pub func: Option<&'m Func>,
    /// Where each local lives.
    homes: Vec<Home>,
    /// The address a list or struct result goes to.
    pub ret_ptr: Option<Value>,
    /// The block of `continue` and the block after each loop around the
    /// statement being lowered, innermost last.
    pub loops: Vec<(Block, Block)>,
    /// The site the last statement marked, while no branch or loop came
    /// between.
    pub last_site: Option<u32>,
    /// The source region of the innermost expression being lowered, whose
    /// site a check in it names; `None` outside an expression.
    pub expr_span: Option<Span>,
    /// Whether the code being lowered can run: false after a return, a break
    /// or a stop, until a block that something jumps to.
    pub live: bool,
    /// The blocks something jumps to.
    reached: HashSet<Block>,
    /// The function of each runtime or program function the code calls.
    refs: HashMap<FuncId, FuncRef>,
    /// The global value of each piece of data the code reads.
    gvs: HashMap<DataId, GlobalValue>,
}

impl<'s, 'm, M: Module> FnGen<'s, 'm, M>
{
    /// A generator, with its first block.
    ///
    /// # Arguments
    ///
    /// * `s` - State shared across functions.
    /// * `func` - The CLIF function.
    /// * `fctx` - The builder state.
    ///
    /// # Returns
    ///
    /// The generator.
    pub fn new(
        s: &'s mut Shared<'m, M>, func: &'s mut Function,
        fctx: &'s mut FunctionBuilderContext,
    ) -> Self
    {
        let mut b = FunctionBuilder::new(func, fctx);
        let first = b.create_block();
        b.append_block_params_for_function_params(first);
        b.switch_to_block(first);
        #[rustfmt::skip]
        return FnGen { s, b, func: None, homes: Vec::new(), ret_ptr: None, loops: Vec::new(),
            last_site: None, expr_span: None, live: true, reached: HashSet::from([first]), refs: HashMap::new(),
            gvs: HashMap::new() };
    }

    /// Seal each block and end the function.
    pub fn finish(self)
    {
        let mut b = self.b;
        b.seal_all_blocks();
        b.finalize(self.s.m.target_config());
    }

    // ---- control flow ----

    /// Jump to a block, when the code can run.
    ///
    /// # Arguments
    ///
    /// * `target` - The block.
    pub fn jump(&mut self, target: Block)
    {
        ret_if!(!self.live);
        self.b.ins().jump(target, &[]);
        self.reached.insert(target);
        self.live = false;
    }

    /// Branch on a truth value, when the code can run.
    ///
    /// # Arguments
    ///
    /// * `cond` - The truth value.
    /// * `then` - The block when it holds.
    /// * `els` - The block when it does not.
    pub fn branch(&mut self, cond: Value, then: Block, els: Block)
    {
        ret_if!(!self.live);
        self.b.ins().brif(cond, then, &[], els, &[]);
        self.reached.extend([then, els]);
        self.live = false;
    }

    /// Go on in a block: it can run when something jumps to it.
    ///
    /// # Arguments
    ///
    /// * `block` - The block.
    pub fn enter_block(&mut self, block: Block)
    {
        self.b.switch_to_block(block);
        self.live = self.reached.contains(&block);
    }

    /// Run `f` in a branch that runs when `cond` holds, then go on after it.
    ///
    /// # Arguments
    ///
    /// * `cond` - The truth value.
    /// * `f` - The lowering of the branch.
    pub fn when(&mut self, cond: Value, f: impl FnOnce(&mut Self))
    {
        ret_if!(!self.live);
        let (then, after) = (self.b.create_block(), self.b.create_block());
        self.branch(cond, then, after);
        self.enter_block(then);
        f(self);
        self.jump(after);
        self.enter_block(after);
    }

    /// End the code that runs: a stop of the runtime never returns.
    pub fn unreachable(&mut self)
    {
        ret_if!(!self.live);
        self.b.ins().trap(TrapCode::unwrap_user(1));
        self.live = false;
    }

    // ---- calls ----

    /// The function reference of a declared function in this function.
    ///
    /// # Arguments
    ///
    /// * `id` - The function.
    ///
    /// # Returns
    ///
    /// The reference.
    pub fn func_ref(&mut self, id: FuncId) -> FuncRef
    {
        ret_if!(let Some(r) = self.refs.get(&id), *r);
        let r = self.s.m.declare_func_in_func(id, self.b.func);
        self.refs.insert(id, r);
        r
    }

    /// Call a function of the runtime; one of [`SITED`] gets the site of the
    /// expression as its last argument.
    ///
    /// # Arguments
    ///
    /// * `name` - Its C name.
    /// * `params` - The CLIF kind of each parameter, the site left out.
    /// * `rets` - The CLIF kind of its result.
    /// * `args` - The arguments, the site left out.
    ///
    /// # Returns
    ///
    /// The result, when it has one.
    pub fn rt(
        &mut self, name: &'static str, params: &[Type], rets: &[Type], args: &[Value],
    ) -> Option<Value>
    {
        ret_if!(!self.live, rets.first().map(|t| self.zero_of(*t)));
        ret_if!(SITED.contains(&name), self.rt_sited(name, params, rets, args));
        self.rt_call(name, params, rets, args)
    }

    /// Call a function of [`SITED`], with the site of the expression last.
    ///
    /// # Arguments
    ///
    /// * `name` - Its C name.
    /// * `params` - The CLIF kind of each parameter, the site left out.
    /// * `rets` - The CLIF kind of its result.
    /// * `args` - The arguments, the site left out.
    ///
    /// # Returns
    ///
    /// The result, when it has one.
    fn rt_sited(
        &mut self, name: &'static str, params: &[Type], rets: &[Type], args: &[Value],
    ) -> Option<Value>
    {
        let site = self.check_site();
        let site = self.b.ins().iconst(types::I32, i64::from(site));
        let params: Vec<Type> = params.iter().copied().chain([types::I32]).collect();
        let args: Vec<Value> = args.iter().copied().chain([site]).collect();
        self.rt_call(name, &params, rets, &args)
    }

    /// The site a check of the expression being lowered names: the site of
    /// the innermost expression, or `NO_SITE` outside an expression and in a
    /// build with no debug info, so the check stops at the site of the
    /// statement, as the VM does.
    ///
    /// # Returns
    ///
    /// The site.
    fn check_site(&mut self) -> u32
    {
        let span = self.expr_span.filter(|_| self.s.options.debug_info);
        span.and_then(|s| self.s.site_of(s)).unwrap_or(NO_SITE)
    }

    /// Call a function of the runtime with these arguments alone.
    ///
    /// # Arguments
    ///
    /// * `name` - Its C name.
    /// * `params` - The CLIF kind of each parameter.
    /// * `rets` - The CLIF kind of its result.
    /// * `args` - The arguments.
    ///
    /// # Returns
    ///
    /// The result, when it has one.
    fn rt_call(
        &mut self, name: &'static str, params: &[Type], rets: &[Type], args: &[Value],
    ) -> Option<Value>
    {
        let id = self.s.runtime_fn(name, params, rets);
        let r = self.func_ref(id);
        let inst = self.b.ins().call(r, args);
        self.b.inst_results(inst).first().copied()
    }

    /// Call a function of the runtime that never returns: a stop.
    ///
    /// # Arguments
    ///
    /// * `name` - Its C name.
    /// * `params` - The CLIF kind of each parameter.
    /// * `args` - The arguments.
    pub fn rt_stop(&mut self, name: &'static str, params: &[Type], args: &[Value])
    {
        // The stop names the expression whose check failed, noted on the
        // failing branch alone.
        let site = self.check_site();
        do_if!(site != NO_SITE, {
            let v = self.b.ins().iconst(types::I32, i64::from(site));
            self.rt("zk_at_site", &[types::I32], &[], &[v]);
        });
        self.rt(name, params, &[], args);
        self.unreachable();
    }

    /// A value of zero of a CLIF kind.
    ///
    /// # Arguments
    ///
    /// * `t` - The kind.
    ///
    /// # Returns
    ///
    /// The value.
    pub fn zero_of(&mut self, t: Type) -> Value
    {
        match t
        {
            types::F32 => self.b.ins().f32const(0.0),
            types::F64 => self.b.ins().f64const(0.0),
            _ => self.b.ins().iconst(t, 0),
        }
    }

    // ---- data ----

    /// The address of data.
    ///
    /// # Arguments
    ///
    /// * `id` - The data.
    ///
    /// # Returns
    ///
    /// The address.
    pub fn data(&mut self, id: DataId) -> Value
    {
        let gv = match self.gvs.get(&id)
        {
            Some(gv) => *gv,
            None =>
            {
                let gv = self.s.m.declare_data_in_func(id, self.b.func);
                self.gvs.insert(id, gv);
                gv
            }
        };
        let ptr = self.s.ptr;
        self.b.ins().symbol_value(ptr, gv)
    }

    /// The address of text with a zero octet after it.
    ///
    /// # Arguments
    ///
    /// * `octets` - The text.
    ///
    /// # Returns
    ///
    /// The address.
    pub fn text_ptr(&mut self, octets: &[u8]) -> Value
    {
        let id = self.s.string(octets);
        self.data(id)
    }

    /// The address of the descriptor of a kind.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// The address.
    pub fn descr(&mut self, kind: DataKind) -> Value
    {
        let id = self.s.descr(kind);
        self.data(id)
    }

    /// A word constant.
    ///
    /// # Arguments
    ///
    /// * `v` - The value.
    ///
    /// # Returns
    ///
    /// The value.
    pub fn word(&mut self, v: i64) -> Value
    {
        let ptr = self.s.ptr;
        self.b.ins().iconst(ptr, v)
    }

    // ---- storage ----

    /// A stack slot of its own, zeroed.
    ///
    /// # Arguments
    ///
    /// * `size` - Its size.
    /// * `align` - Its alignment.
    ///
    /// # Returns
    ///
    /// The slot.
    pub(crate) fn slot(&mut self, size: u32, align: u32) -> StackSlot
    {
        let shift = align.max(1).trailing_zeros().min(4) as u8;
        let data = StackSlotData::new(StackSlotKind::ExplicitSlot, size.max(1), shift);
        let slot = self.b.create_sized_stack_slot(data);
        let ptr = self.s.ptr;
        let at = self.b.ins().stack_addr(ptr, slot, 0);
        self.zero(at, size);
        slot
    }

    /// A temp of its own for a value of a kind, zeroed.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// Its address.
    pub fn temp(&mut self, kind: DataKind) -> Value
    {
        let (size, align) = (self.s.size(kind), kind.align(self.s.platform()));
        let slot = self.slot(size, align);
        let ptr = self.s.ptr;
        self.b.ins().stack_addr(ptr, slot, 0)
    }

    /// A temp of raw octets, zeroed.
    ///
    /// # Arguments
    ///
    /// * `size` - The octets.
    ///
    /// # Returns
    ///
    /// Its address.
    pub fn raw_temp(&mut self, size: u32) -> Value
    {
        let slot = self.slot(size, 8);
        let ptr = self.s.ptr;
        self.b.ins().stack_addr(ptr, slot, 0)
    }

    /// Write zero octets at an address the code owns.
    ///
    /// # Arguments
    ///
    /// * `at` - The address.
    /// * `size` - The octets.
    pub fn zero(&mut self, at: Value, size: u32)
    {
        let flags = MemFlagsData::trusted();
        let mut n = 0;
        while n + 8 <= size
        {
            let z = self.b.ins().iconst(types::I64, 0);
            self.b.ins().store(flags, z, at, n as i32);
            n += 8;
        }
        while n < size
        {
            let z = self.b.ins().iconst(types::I8, 0);
            self.b.ins().store(flags, z, at, n as i32);
            n += 1;
        }
    }

    /// Load a scalar.
    ///
    /// # Arguments
    ///
    /// * `t` - Its CLIF kind.
    /// * `at` - The address.
    ///
    /// # Returns
    ///
    /// The value.
    pub fn load(&mut self, t: Type, at: Value) -> Value
    {
        self.b.ins().load(t, MemFlagsData::new(), at, 0)
    }

    /// Store a scalar.
    ///
    /// # Arguments
    ///
    /// * `v` - The value.
    /// * `at` - The address.
    pub fn store(&mut self, v: Value, at: Value)
    {
        self.b.ins().store(MemFlagsData::new(), v, at, 0);
    }

    /// Stop the run at an access through the null address, under the mem
    /// checks.
    ///
    /// # Arguments
    ///
    /// * `p` - The address.
    pub fn null_check(&mut self, p: Value)
    {
        ret_if!(!self.s.options.checks.has(RunCheck::Null));
        let null = self.b.ins().icmp_imm_s(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            p,
            0,
        );
        let ptr = self.s.ptr;
        self.when(null, |g| g.rt_stop("zk_null_deref", &[ptr], &[p]));
    }

    /// Write a value at an address: a store of a scalar, a move of a block.
    ///
    /// # Arguments
    ///
    /// * `at` - The address.
    /// * `v` - The value.
    /// * `kind` - Its kind.
    pub fn put_at(&mut self, at: Value, v: Value, kind: DataKind)
    {
        if is_blob(kind)
        {
            let ptr = self.s.ptr;
            let size = self.word(i64::from(self.s.size(kind)));
            self.rt("zk_put_block", &[ptr, ptr, ptr], &[], &[at, v, size]);
            return;
        }
        self.store(v, at);
    }

    /// The address of a value: a block names its own; a scalar goes into a
    /// temp of its own.
    ///
    /// # Arguments
    ///
    /// * `v` - The value.
    /// * `kind` - Its kind.
    ///
    /// # Returns
    ///
    /// The address.
    pub fn addressed(&mut self, v: Value, kind: DataKind) -> Value
    {
        ret_if!(is_blob(kind), v);
        let t = self.temp(kind);
        self.store(v, t);
        t
    }

    // ---- places ----

    /// The kind of a slot.
    ///
    /// # Arguments
    ///
    /// * `p` - The slot.
    ///
    /// # Returns
    ///
    /// The kind.
    pub fn place_kind(&self, p: Place) -> DataKind
    {
        match p
        {
            Place::Global(n) => self.s.module.globals[n as usize].kind,
            Place::Local(n) =>
            {
                self.func.map_or(DataKind::Bool, |f| f.locals[n as usize].kind)
            }
        }
    }

    /// The address of a slot.
    ///
    /// # Arguments
    ///
    /// * `p` - The slot.
    ///
    /// # Returns
    ///
    /// The address.
    pub fn addr_of(&mut self, p: Place) -> Value
    {
        let ptr = self.s.ptr;
        match p
        {
            Place::Global(n) =>
            {
                let id = self.s.globals[n as usize];
                self.data(id)
            }
            Place::Local(n) => match self.homes[n as usize]
            {
                Home::Slot(slot) => self.b.ins().stack_addr(ptr, slot, 0),
                Home::Ptr(v) => v,
            },
        }
    }

    /// The value of a slot: a scalar, or the address of a block.
    ///
    /// # Arguments
    ///
    /// * `p` - The slot.
    ///
    /// # Returns
    ///
    /// The value.
    pub fn read(&mut self, p: Place) -> Value
    {
        let kind = self.place_kind(p);
        let at = self.addr_of(p);
        ret_if!(is_blob(kind), at);
        let t = self.s.clif_of(kind);
        self.load(t, at)
    }

    // ---- frames and sites ----

    /// Whether the build keeps the calls in progress.
    ///
    /// # Returns
    ///
    /// `true` for the depth check, or a build with debug info.
    fn tracks(&self) -> bool
    {
        self.s.options.debug_info || self.s.options.checks.has(RunCheck::Depth)
    }

    /// Note a call as it returns, in a function.
    pub fn leave_call(&mut self)
    {
        ret_if!(self.func.is_none() || !self.tracks());
        self.rt("zk_leave_call", &[], &[], &[]);
    }

    /// Note the site of a statement in the frame of the call, in a build with
    /// debug info, unless the statement before it noted the same site.
    ///
    /// # Arguments
    ///
    /// * `span` - The source region of the statement.
    pub fn mark_site(&mut self, span: Span)
    {
        ret_if!(!self.s.options.debug_info || !self.live);
        let site = self.s.site_of(span);
        ret_if!(site.is_none() || site == self.last_site);
        self.last_site = site;
        // The code of the statement maps to its site in the line table.
        self.b.set_srcloc(cranelift_codegen::ir::SourceLoc::new(site.unwrap_or(0)));
        let v = self.b.ins().iconst(types::I32, i64::from(site.unwrap_or(0)));
        self.rt("zk_at_site", &[types::I32], &[], &[v]);
    }

    // ---- the bodies ----

    /// Lower a function: its locals, its parameters copied in as the C target
    /// copies them, its statements, and its result.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of the function.
    /// * `f` - The function.
    pub fn func(&mut self, n: u32, f: &'m Func)
    {
        self.func = Some(f);
        let first = self.b.current_block().expect("the first block");
        let mut params = self.b.block_params(first).to_vec().into_iter();
        let blob_ret = f.ret.is_some_and(is_blob);
        do_if!(blob_ret, self.ret_ptr = params.next());
        let platform = self.s.platform();
        for (i, slot) in f.locals.iter().enumerate()
        {
            let param = i < f.params as usize;
            let shared =
                param && is_blob(slot.kind) && f.shared_params.contains(&(i as u32));
            let arg = if param { params.next() } else { None };
            if let (true, Some(v)) = (shared, arg)
            {
                self.homes.push(Home::Ptr(v));
                continue;
            }
            let s = self.slot(slot.kind.size(platform), slot.kind.align(platform));
            self.homes.push(Home::Slot(s));
            let Some(v) = arg
            else
            {
                continue;
            };
            let at = self.addr_of(Place::Local(i as u32));
            self.put_at(at, v, slot.kind);
        }
        do_if!(self.tracks(), {
            let v = self.b.ins().iconst(types::I32, i64::from(n + 1));
            self.rt("zk_enter_call", &[types::I32], &[], &[v]);
        });
        self.block(&f.block);
        ret_if!(!self.live);
        let returns = matches!(f.block.last(), Some(Stmt::Return { .. }));
        let result = f.result.filter(|_| !returns && f.ret.is_some());
        match result
        {
            Some(r) =>
            {
                let v = self.read(Place::Local(r));
                self.give_back(v, f.ret);
            }
            None =>
            {
                self.leave_call();
                self.b.ins().return_(&[]);
                self.live = false;
            }
        }
    }

    /// Hand the result of the function back: a scalar as the result, a list
    /// or struct into the place the caller gave.
    ///
    /// # Arguments
    ///
    /// * `v` - The value.
    /// * `ret` - The result kind.
    pub fn give_back(&mut self, v: Value, ret: Option<DataKind>)
    {
        ret_if!(!self.live);
        if self.func.is_none()
        {
            return self.finish_main(Some(v), ret);
        }
        if let (Some(kind), Some(at)) = (ret.filter(|k| is_blob(*k)), self.ret_ptr)
        {
            self.put_at(at, v, kind);
            self.leave_call();
            self.b.ins().return_(&[]);
            self.live = false;
            return;
        }
        self.leave_call();
        let vals: Vec<Value> = ret.map(|_| v).into_iter().collect();
        self.b.ins().return_(&vals);
        self.live = false;
    }

    /// Lower the main function: the start of the run, the arguments and the
    /// environment of the start block, the top-level statements, and the exit
    /// status.
    ///
    /// # Arguments
    ///
    /// * `table` - The data of the table of the program.
    pub fn main(&mut self, table: DataId)
    {
        let module = self.s.module;
        let first = self.b.current_block().expect("the first block");
        let [argc, argv, envp] = self.b.block_params(first)[..]
        else
        {
            return;
        };
        let ptr = self.s.ptr;
        let t = self.data(table);
        self.rt("zk_start", &[types::I32, ptr, ptr], &[], &[argc, argv, t]);
        if let Some(g) = module.host.args
        {
            let at = self.addr_of(Place::Global(g));
            self.rt("zk_text_list", &[ptr, types::I32, ptr], &[], &[at, argc, argv]);
        }
        if let Some(g) = module.host.env
        {
            let count = self.raw_temp(4);
            let env = self
                .rt("zk_allowed_env", &[ptr, ptr], &[ptr], &[envp, count])
                .unwrap_or(envp);
            let n = self.load(types::I32, count);
            let at = self.addr_of(Place::Global(g));
            self.rt("zk_text_list", &[ptr, types::I32, ptr], &[], &[at, n, env]);
        }
        self.block(&module.main);
        self.finish_main(None, None);
    }

    /// Lower `zk_lib_init` of a C lib: the top-level statements, which set the
    /// globals, with no drop of them at the end, since the globals live as
    /// long as the lib.
    pub fn lib_init(&mut self)
    {
        self.block(init_stmts(self.s.module));
        ret_if!(!self.live);
        self.b.ins().return_(&[]);
    }

    /// End the main function: write out what is buffered, and give the exit
    /// status.
    ///
    /// # Arguments
    ///
    /// * `value` - The value a return gave, when it gave one.
    /// * `kind` - Its kind.
    fn finish_main(&mut self, value: Option<Value>, kind: Option<DataKind>)
    {
        ret_if!(!self.live);
        let status = match (value, kind, self.s.module.host.exit)
        {
            (Some(v), Some(k), _) if !is_blob(k) => self.as_i32(v),
            (_, _, Some(g)) =>
            {
                let at = self.addr_of(Place::Global(g));
                let low = self.load(types::I8, at);
                self.b.ins().uextend(types::I32, low)
            }
            _ => self.b.ins().iconst(types::I32, 0),
        };
        let out = self
            .rt("zk_finish", &[types::I32], &[types::I32], &[status])
            .unwrap_or(status);
        self.b.ins().return_(&[out]);
        self.live = false;
    }

    /// A scalar as an `i32`, cut or widened.
    ///
    /// # Arguments
    ///
    /// * `v` - The value.
    ///
    /// # Returns
    ///
    /// The value.
    fn as_i32(&mut self, v: Value) -> Value
    {
        let t = self.b.func.dfg.value_type(v);
        match t.bits()
        {
            32 => v,
            b if b > 32 => self.b.ins().ireduce(types::I32, v),
            _ => self.b.ins().uextend(types::I32, v),
        }
    }
}
