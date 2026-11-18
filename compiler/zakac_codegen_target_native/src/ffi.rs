//! Calls of C functions, and functions of the program C calls back. Each call
//! goes through `zk_ffi_call` of the runtime, which makes the libffi call
//! interface of its shape once: libffi places structs passed as values and
//! the values after a `...` as each ABI wants them. The shape and the C kind
//! of each value are data the code holds.

use cranelift_codegen::ir::InstBuilder;
use cranelift_codegen::ir::Value;
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::types;
use cranelift_module::DataId;
use cranelift_module::FuncId;
use cranelift_module::Linkage;
use cranelift_module::Module;
use zakac_codegen_target_c::callbacks::Crossing;
use zakac_codegen_target_c::callbacks::as_is;
use zakac_codegen_target_c::callbacks::box_list;
use zakac_codegen_target_c::callbacks::cells;
use zakac_codegen_target_c::callbacks::crossing;
use zakac_codegen_target_c::callbacks::fits;
use zakac_codegen_target_c::kinds::is_blob;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::fn_shape;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::foreign::is_c_handle_result;
use zakac_semcheck::foreign::is_c_text_result;
use zakac_semcheck::kind::DataKind;

use crate::blob::Blob;
use crate::blob::Target;
use crate::blob::define;
use crate::emit::Shared;
use crate::func::FnGen;

/// The tags of `ZkCKind` of the runtime.
mod tag
{
    pub const VOID: u32 = 0;
    pub const PTR: u32 = 11;
    pub const STRUCT: u32 = 12;
}

/// How C sees a kind: as an address, as a struct passed as a value, or as a
/// scalar.
#[derive(Clone, Copy, PartialEq, Eq)]
enum CSide
{
    /// An address: text, a handle, an address, a function.
    Ptr,
    /// A struct passed as a value.
    Struct,
    /// A number, a character or a boolean.
    Scalar,
}

/// How C sees a kind, as the C target's wrappers give it.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// The side.
fn c_side(kind: DataKind) -> CSide
{
    let address = kind.is_fn()
        || matches!(kind, DataKind::Ptr(_) | DataKind::Null)
        || kind.is_text()
        || is_c_text_result(kind)
        || kind.as_external().is_some()
        || is_c_handle_result(kind);
    ret_if!(address, CSide::Ptr);
    tern!(matches!(kind.repr(), DataKind::Struct(_)), CSide::Struct, CSide::Scalar)
}

impl<M: Module> Shared<'_, M>
{
    /// The data of the C kind of a kind, made once.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind; `None` for no result.
    ///
    /// # Returns
    ///
    /// The data.
    pub fn ckind(&mut self, kind: Option<DataKind>) -> DataId
    {
        let mark = kind.map_or_else(|| "void".to_string(), |k| format!("{k:?}"));
        ret_if!(let Some(id) = self.ckinds.get(&mark), *id);
        let id = self.anon(false);
        self.ckinds.insert(mark, id);
        let mut b = Blob::new();
        let (t, members) = match kind.map(|k| (k, c_side(k)))
        {
            None => (tag::VOID, None),
            Some((_, CSide::Ptr)) => (tag::PTR, None),
            Some((k, CSide::Struct)) => (tag::STRUCT, Some(self.struct_members(k))),
            Some((k, CSide::Scalar)) => (self.scalar_tag(k), None),
        };
        let count = members.as_ref().map_or(0, |(_, n)| *n);
        b.u32(t);
        b.u32(count as u32);
        b.ptr(members.map(|(m, _)| Target::Data(m, 0)));
        define(self.m, id, b.done()).expect("a C kind");
        id
    }

    /// The members of a struct as libffi places them: each field, and each
    /// cell of a counted list, in order.
    ///
    /// # Arguments
    ///
    /// * `kind` - The struct kind.
    ///
    /// # Returns
    ///
    /// The list of the C kind of each member, and its count.
    fn struct_members(&mut self, kind: DataKind) -> (DataId, usize)
    {
        let fields = kind.as_struct().map(|s| s.fields().to_vec()).unwrap_or_default();
        let each = fields.iter().map(|(_, k)| cells(*k));
        let flat: Vec<DataKind> =
            each.flat_map(|(cell, n)| std::iter::repeat_n(cell, n)).collect();
        let ids: Vec<DataId> = flat.iter().map(|k| self.ckind(Some(*k))).collect();
        (self.ptr_list(&ids), ids.len())
    }

    /// The tag of a scalar kind.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// The tag.
    fn scalar_tag(&self, kind: DataKind) -> u32
    {
        match kind.repr()
        {
            DataKind::Int(k) =>
            {
                let bits = k.bits(self.platform());
                let unsigned = match bits
                {
                    8 => 1,
                    16 => 3,
                    32 => 5,
                    _ => 7,
                };
                unsigned + u32::from(k.signed)
            }
            DataKind::Float(32) => 9,
            DataKind::Float(_) => 10,
            DataKind::Char => 5,
            DataKind::Bool => 1,
            _ => tag::PTR,
        }
    }

    /// A list of the addresses of data.
    ///
    /// # Arguments
    ///
    /// * `ids` - The data.
    ///
    /// # Returns
    ///
    /// The list.
    fn ptr_list(&mut self, ids: &[DataId]) -> DataId
    {
        let id = self.anon(false);
        let mut b = Blob::new();
        ids.iter().for_each(|d| b.ptr(Some(Target::Data(*d, 0))));
        define(self.m, id, b.done()).expect("a list");
        id
    }

    /// The shape of a call of a C function, made once for each shape.
    ///
    /// # Arguments
    ///
    /// * `func` - Index into the C functions of the module.
    /// * `kinds` - The kind of each value, the other values last.
    ///
    /// # Returns
    ///
    /// The data.
    fn ffi_shape(&mut self, func: u32, kinds: &[DataKind]) -> DataId
    {
        let mark = format!("{func}:{kinds:?}");
        ret_if!(let Some(id) = self.ffi_shapes.get(&mark), *id);
        let def = &self.module.foreign[func as usize];
        let (ret, fixed, variadic) = (def.ret, def.params.len(), def.variadic);
        let ret_id = self.ckind(ret);
        let ids: Vec<DataId> = kinds.iter().map(|k| self.ckind(Some(*k))).collect();
        let args = self.ptr_list(&ids);
        let id = self.anon(false);
        let mut b = Blob::new();
        b.ptr(Some(Target::Data(ret_id, 0)));
        b.ptr(Some(Target::Data(args, 0)));
        b.u32(kinds.len() as u32);
        b.u32(fixed as u32);
        b.u32(u32::from(variadic));
        define(self.m, id, b.done()).expect("a call shape");
        self.ffi_shapes.insert(mark, id);
        id
    }

    /// The C function of a name, imported once.
    ///
    /// # Arguments
    ///
    /// * `name` - Its C name.
    ///
    /// # Returns
    ///
    /// The function.
    fn c_fn(&mut self, name: &str) -> FuncId
    {
        ret_if!(let Some(id) = self.c_fns.get(name), *id);
        let sig = self.m.make_signature();
        let id =
            self.m.declare_function(name, Linkage::Import, &sig).expect("a C name");
        self.c_fns.insert(name.to_string(), id);
        id
    }

    /// The linkage of a bridge: a C lib gives each one to the wrappers of its
    /// C file.
    ///
    /// # Returns
    ///
    /// The linkage.
    fn bridge_linkage(&self) -> Linkage
    {
        tern!(self.options.lib, Linkage::Export, Linkage::Local)
    }

    /// The native shape of a function kind C calls back, made on its first
    /// use with its bridge, which is defined once each function is lowered.
    ///
    /// # Arguments
    ///
    /// * `kind` - The function kind.
    ///
    /// # Returns
    ///
    /// The data, or `None` for a kind whose closure cannot go to C.
    pub fn cb_shape(&mut self, kind: DataKind) -> Option<DataId>
    {
        let f = kind.as_fn().filter(|f| fits(f, self.platform()))?;
        ret_if!(let Some((_, _, id)) = self.cb_shapes.iter().find(|(k, _, _)| *k == kind), Some(*id));
        let ptr = self.ptr;
        let mut sig = self.m.make_signature();
        sig.params.extend([ptr, ptr, ptr].map(cranelift_codegen::ir::AbiParam::new));
        let n = self.cb_shapes.len();
        let bridge = self
            .m
            .declare_function(&format!("zk_cbb_{n}"), self.bridge_linkage(), &sig)
            .expect("a bridge");
        let (params, ret) = fn_shape(f);
        let ret_id = self.ckind(ret);
        let param_ids: Vec<DataId> = params
            .iter()
            .map(|k| match crossing(*k, self.platform())
            {
                Some(Crossing::Struct) | Some(Crossing::AsIs) | None =>
                {
                    self.ckind(Some(*k))
                }
                Some(_) => self.ckind(Some(DataKind::OPAQUE)),
            })
            .collect();
        let args = self.ptr_list(&param_ids);
        let id = self.anon(false);
        let mut b = Blob::new();
        b.ptr(Some(Target::Func(bridge)));
        b.ptr(Some(Target::Data(ret_id, 0)));
        b.ptr(Some(Target::Data(args, 0)));
        b.u32(params.len() as u32);
        b.u32(u32::from(as_is(f, self.platform())));
        define(self.m, id, b.done()).expect("a callback shape");
        self.cb_shapes.push((kind, bridge, id));
        Some(id)
    }
}

impl<M: Module> FnGen<'_, '_, M>
{
    /// The address C calls for a function value given to a C call: through the
    /// runtime of closures for a kind C can call back, else the code of the
    /// function, which stops the run for a closure; NULL for a null value.
    ///
    /// # Arguments
    ///
    /// * `f` - The function value.
    /// * `kind` - Its kind.
    ///
    /// # Returns
    ///
    /// The address.
    fn c_code(&mut self, f: Value, kind: DataKind) -> Value
    {
        let ptr = self.s.ptr;
        let held = self.raw_temp(8);
        let shape = self.s.cb_shape(kind);
        let set = self.b.ins().icmp_imm_s(IntCC::NotEqual, f, 0);
        self.when(set, |g| {
            let code = match shape
            {
                Some(id) =>
                {
                    let native = g.data(id);
                    let s = g
                        .rt("zk_cb_shape_of", &[ptr], &[ptr], &[native])
                        .unwrap_or(native);
                    g.rt("zk_cb_code", &[ptr, ptr], &[ptr], &[f, s])
                }
                None =>
                {
                    let c = g.rt("zk_fn_c", &[ptr], &[ptr], &[f]).unwrap_or(f);
                    g.rt("zk_fn_code", &[ptr], &[ptr], &[c])
                }
            };
            do_if!(let Some(code) = code, g.store(code, held));
        });
        self.load(ptr, held)
    }

    /// Call a C function, as the C target's wrapper does: the right to call C
    /// is checked; text goes as a duplicate that ends in a zero octet, a
    /// function as its code, a handle as its address, a struct as its octets;
    /// the result comes back as a value of Zaka.
    ///
    /// # Arguments
    ///
    /// * `func` - Index into the C functions of the module.
    /// * `args` - The value of each parameter, then the other values.
    /// * `ret` - The result kind; `None` for no result.
    ///
    /// # Returns
    ///
    /// The result, when there is one.
    pub fn foreign(
        &mut self, func: u32, args: &[Expr], ret: Option<DataKind>,
    ) -> Option<Value>
    {
        let ptr = self.s.ptr;
        let def = self.s.module.foreign[func as usize].clone();
        let name = self.text_ptr(def.name.as_bytes());
        self.rt("zk_need_ffi", &[ptr], &[], &[name]);
        let kinds: Vec<DataKind> = args.iter().map(|a| a.kind).collect();
        let lends = kinds.iter().any(|k| k.is_fn());
        let lent = if lends
        {
            self.rt("zk_cb_lent", &[], &[ptr], &[])
        }
        else
        {
            None
        };
        let vals: Vec<Value> = args.iter().map(|a| self.expr(a)).collect();
        let list = self.raw_temp(8 * kinds.len().max(1) as u32);
        let mut texts = Vec::new();
        for (i, (v, k)) in vals.iter().zip(&kinds).enumerate()
        {
            let at = self.c_value(*v, *k, &mut texts);
            self.b.ins().store(
                cranelift_codegen::ir::MemFlagsData::new(),
                at,
                list,
                (8 * i) as i32,
            );
        }
        let size = ret.map_or(0, |k| self.s.size(k)).max(16);
        let out = self.raw_temp(size);
        let shape = self.s.ffi_shape(func, &kinds);
        let shape = self.data(shape);
        let code_id = self.s.c_fn(&def.name);
        let code = self.func_ref(code_id);
        let code = self.b.ins().func_addr(ptr, code);
        self.rt("zk_ffi_call", &[ptr, ptr, ptr, ptr], &[], &[code, shape, list, out]);
        do_if!(let Some(n) = lent, _ = self.rt("zk_cb_end", &[ptr], &[], &[n]));
        let result = ret.map(|k| self.c_result(out, k, def.owned_ret));
        for t in texts
        {
            self.rt("zk_text_cstr_free", &[ptr], &[], &[t]);
        }
        result
    }

    /// The address of the C value of one argument: a temp that holds it, or
    /// the octets of a struct.
    ///
    /// # Arguments
    ///
    /// * `v` - The value of Zaka.
    /// * `kind` - Its kind.
    /// * `texts` - The duplicates of text to free after the call, added to.
    ///
    /// # Returns
    ///
    /// The address.
    fn c_value(&mut self, v: Value, kind: DataKind, texts: &mut Vec<Value>) -> Value
    {
        let ptr = self.s.ptr;
        let side = c_side(kind);
        ret_if!(side == CSide::Struct, v);
        let c = match kind
        {
            k if k.is_text() =>
            {
                let s = self.rt("zk_text_cstr", &[ptr], &[ptr], &[v]).unwrap_or(v);
                texts.push(s);
                s
            }
            k if k.is_fn() => self.c_code(v, k),
            k if is_blob(k) => self.load(ptr, v),
            _ => v,
        };
        let held = self.raw_temp(16);
        self.store(c, held);
        held
    }

    /// The value of Zaka of the result of a C function: text copied from the
    /// string C gave (freed when C handed it over), a struct copied from its
    /// octets, a handle around its address, or a scalar.
    ///
    /// # Arguments
    ///
    /// * `out` - Where libffi wrote the result.
    /// * `kind` - The result kind.
    /// * `owned` - Whether C hands over a string result.
    ///
    /// # Returns
    ///
    /// The value.
    fn c_result(&mut self, out: Value, kind: DataKind, owned: bool) -> Value
    {
        let ptr = self.s.ptr;
        if is_c_text_result(kind)
        {
            let t = self.temp(kind);
            let r = self.load(ptr, out);
            let platform = self.s.platform();
            let ok_at = kind.as_struct().map_or(0, |s| s.offset(1, platform));
            let (size, ok) =
                (self.word(i64::from(self.s.size(kind))), self.word(i64::from(ok_at)));
            self.rt("zk_text_from_c", &[ptr, ptr, ptr, ptr], &[], &[t, size, ok, r]);
            do_if!(owned, _ = self.rt("zk_c_free", &[ptr], &[], &[r]));
            return t;
        }
        if is_blob(kind)
        {
            let t = self.temp(kind);
            let n = tern!(c_side(kind) == CSide::Ptr, 8, self.s.size(kind));
            let size = self.word(i64::from(n));
            self.rt("zk_put_block", &[ptr, ptr, ptr], &[], &[t, out, size]);
            return t;
        }
        let t = self.s.clif_of(kind);
        self.load(t, out)
    }

    /// Give C leave to call a function value after the call that gave it, or
    /// take that leave back.
    ///
    /// # Arguments
    ///
    /// * `value` - The function value.
    /// * `grant` - `true` to give the leave, `false` to take it back.
    pub fn leave(&mut self, value: &Expr, grant: bool)
    {
        let ptr = self.s.ptr;
        let v = self.expr(value);
        let Some(id) = self.s.cb_shape(value.kind)
        else
        {
            let what = "a leave for C of a function that takes or gives a value C \
                        cannot pass to a callback";
            return self.s.missing(what);
        };
        if !grant
        {
            self.rt("zk_cb_revoke", &[ptr], &[], &[v]);
            return;
        }
        let native = self.data(id);
        let s = self.rt("zk_cb_shape_of", &[ptr], &[ptr], &[native]).unwrap_or(native);
        self.rt("zk_cb_leave", &[ptr, ptr], &[], &[v, s]);
    }

    /// Lower the bridge of a function kind C calls back: read each argument
    /// libffi gives, make a value of Zaka of each one that does not cross as
    /// it is, call the thunk of a closure with its record or a function with
    /// none, then write the result where libffi wants it.
    ///
    /// # Arguments
    ///
    /// * `kind` - The function kind.
    pub fn bridge(&mut self, kind: DataKind)
    {
        let ptr = self.s.ptr;
        let first = self.b.current_block().expect("the first block");
        let &[ret_at, list, rec] = self.b.block_params(first)
        else
        {
            return;
        };
        let Some(f) = kind.as_fn()
        else
        {
            return;
        };
        let (params, ret) = fn_shape(f);
        let mut drops = Vec::new();
        let mut vals = Vec::new();
        for (i, k) in params.iter().enumerate()
        {
            let at = self.b.ins().load(
                ptr,
                cranelift_codegen::ir::MemFlagsData::new(),
                list,
                (8 * i) as i32,
            );
            vals.push(self.bridge_arg(at, *k, &mut drops));
        }
        let code = self.rt("zk_fn_code", &[ptr], &[ptr], &[rec]).unwrap_or(rec);
        let bound =
            self.rt("zk_fn_bound", &[ptr], &[types::I32], &[rec]).unwrap_or(rec);
        let (plain, with_rec) =
            (self.s.sig(&params, ret, false), self.s.sig(&params, ret, true));
        let (plain, with_rec) =
            (self.b.import_signature(plain), self.b.import_signature(with_rec));
        let held = self.raw_temp(16);
        let (on, off, after) =
            (self.b.create_block(), self.b.create_block(), self.b.create_block());
        self.branch(bound, on, off);
        self.enter_block(on);
        let args: Vec<Value> = vals.iter().copied().chain([rec]).collect();
        let inst = self.b.ins().call_indirect(with_rec, code, &args);
        let r = self.b.inst_results(inst).first().copied();
        do_if!(let Some(r) = r, self.store(r, held));
        self.jump(after);
        self.enter_block(off);
        let inst = self.b.ins().call_indirect(plain, code, &vals);
        let r = self.b.inst_results(inst).first().copied();
        do_if!(let Some(r) = r, self.store(r, held));
        self.jump(after);
        self.enter_block(after);
        let result = ret.map(|k| (k, self.load(self.s.clif_of(k), held)));
        for (at, k) in drops
        {
            self.drop_at(at, k);
        }
        do_if!(let Some((k, r)) = result, self.bridge_result(ret_at, r, k));
        self.b.ins().return_(&[]);
        self.live = false;
    }

    /// The value of one argument a bridge hands the thunk.
    ///
    /// # Arguments
    ///
    /// * `at` - The address of the value C passed.
    /// * `kind` - The parameter kind.
    /// * `drops` - The values to drop after the call, added to.
    ///
    /// # Returns
    ///
    /// The value.
    fn bridge_arg(
        &mut self, at: Value, kind: DataKind, drops: &mut Vec<(Value, DataKind)>,
    ) -> Value
    {
        let ptr = self.s.ptr;
        let platform = self.s.platform();
        let cross = crossing(kind, platform).unwrap_or(Crossing::AsIs);
        if cross == Crossing::AsIs
        {
            let t = self.s.clif_of(kind);
            return self.load(t, at);
        }
        let v = self.temp(kind);
        let size = self.word(i64::from(self.s.size(kind)));
        let ok_at =
            self.word(i64::from(kind.as_struct().map_or(0, |s| s.offset(1, platform))));
        let chars = self.load(ptr, at);
        match cross
        {
            Crossing::Struct =>
            {
                self.rt("zk_put_block", &[ptr, ptr, ptr], &[], &[v, at, size]);
                return v;
            }
            Crossing::Text =>
            {
                _ = self.rt("zk_text_of_c", &[ptr, ptr], &[], &[v, chars])
            }
            Crossing::OptText =>
            {
                _ = self.rt(
                    "zk_text_from_c",
                    &[ptr, ptr, ptr, ptr],
                    &[],
                    &[v, size, ok_at, chars],
                )
            }
            Crossing::Handle =>
            {
                let cells = box_list(kind).map(|k| self.descr(k)).unwrap_or(at);
                self.rt("zk_cb_handle", &[ptr, ptr, ptr], &[], &[v, at, cells]);
            }
            _ =>
            {
                let cells = box_list(kind).map(|k| self.descr(k)).unwrap_or(at);
                self.rt(
                    "zk_cb_opt_handle",
                    &[ptr, ptr, ptr, ptr, ptr],
                    &[],
                    &[v, size, ok_at, at, cells],
                );
            }
        }
        let lent = self.temp(kind);
        self.rt("zk_put_block", &[ptr, ptr, ptr], &[], &[lent, v, size]);
        let d = self.descr(kind);
        self.rt("zk_lend_seqs", &[ptr, ptr], &[], &[lent, d]);
        drops.push((v, kind));
        lent
    }

    /// Write the result of a thunk where libffi wants it: a whole word for an
    /// integer narrower than one, as libffi asks.
    ///
    /// # Arguments
    ///
    /// * `at` - Where the result goes.
    /// * `r` - The result.
    /// * `kind` - Its kind.
    fn bridge_result(&mut self, at: Value, r: Value, kind: DataKind)
    {
        let v = match kind.repr()
        {
            DataKind::Int(k) if k.signed => self.int_cast(r, true, types::I64),
            DataKind::Int(_) | DataKind::Char | DataKind::Bool =>
            {
                self.int_cast(r, false, types::I64)
            }
            _ => r,
        };
        self.store(v, at);
    }
}
