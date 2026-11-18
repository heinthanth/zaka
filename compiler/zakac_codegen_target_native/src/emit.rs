//! The lowering of a module: the functions, the globals, the records of the
//! functions, the kind descriptors and the table of the program.

use std::collections::HashMap;
use std::sync::Arc;

use cranelift_codegen::Context;
use cranelift_codegen::ir::AbiParam;
use cranelift_codegen::ir::Signature;
use cranelift_codegen::ir::SourceLoc;
use cranelift_codegen::ir::Type;
use cranelift_codegen::ir::UserFuncName;
use cranelift_codegen::ir::types;
use cranelift_frontend::FunctionBuilderContext;
use cranelift_module::DataId;
use cranelift_module::FuncId;
use cranelift_module::Linkage;
use cranelift_module::Module;
use zaka_host::Grant;
use zaka_host::Grants;
use zakac_codegen_target_c::descriptors::Descriptors;
use zakac_codegen_target_c::descriptors::KindRow;
use zakac_codegen_target_c::emit::PROGRAM_TABLE;
use zakac_codegen_target_c::emit::debug_path;
use zakac_codegen_target_c::emit::site_mark;
use zakac_codegen_target_c::emit::table_texts;
use zakac_codegen_target_c::exports::EXPORT_KINDS;
use zakac_codegen_target_c::exports::Export;
use zakac_codegen_target_c::exports::exported;
use zakac_codegen_target_c::kinds::is_blob;
use zakac_codegen_target_c::options::TargetCOptions;
use zakac_host_errors::Error;
use zakac_ir_builder::ir::Func;
use zakac_ir_builder::ir::Module as IrModule;
use zakac_ir_builder::ir::shown_name;
use zakac_ir_builder::spans::spell_repeats;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;
use zakac_span::source_file::SourceFile;
use zakac_span::span::Span;

use crate::blob::Blob;
use crate::blob::Target;
use crate::blob::define;
use crate::debug::FnLines;
use crate::debug::LineTable;
use crate::func::FnGen;
use crate::unwind::FnUnwind;
use crate::unwind::fn_unwind;

/// What a lowering gives: the main function, the data the runtime reads, and
/// the CLIF text when it was asked for.
#[derive(bon::Builder)]
pub struct Lowered
{
    /// The main function of the program: `main` for AOT, `zk_main` for JIT.
    pub main: FuncId,
    /// The CLIF text of each function, when asked for.
    pub clif: String,
    /// The table of the program, `zk_program`.
    pub table: DataId,
    /// Each kind descriptor, `zk_kind`, beside the row it was written from.
    pub descrs: Vec<(DataId, KindRow)>,
    /// In a C lib, the index of the bridge `zk_cbb_<n>` of each function that
    /// code outside can call, in the order of `exported`.
    #[builder(default)]
    pub export_shapes: Vec<usize>,
    /// The line table of the code, in a build with debug info.
    #[builder(default)]
    pub lines: LineTable,
    /// The unwind codes of each function, for 64-bit Windows.
    #[builder(default)]
    pub unwinds: Vec<FnUnwind>,
}

/// State shared across the functions of a program.
pub struct Shared<'m, M: Module>
{
    /// The module the code goes into.
    pub m: &'m mut M,
    /// The program.
    pub module: &'m IrModule,
    /// The choices of the build.
    pub options: TargetCOptions,
    /// The source files, for the sites of stops.
    files: Vec<Arc<SourceFile>>,
    /// The kind of an address.
    pub ptr: Type,
    /// The kind descriptors the program needs.
    pub descr: Descriptors,
    /// The data of each descriptor so far, under its number.
    descr_ids: Vec<DataId>,
    /// The data of each text, under its octets.
    strings: HashMap<Vec<u8>, DataId>,
    /// Each function of the runtime the code calls, under its name.
    runtime: HashMap<&'static str, FuncId>,
    /// Each function of the program.
    pub funcs: Vec<FuncId>,
    /// Each global of the program.
    pub globals: Vec<DataId>,
    /// The record of each function, the value its address gives.
    pub records: Vec<DataId>,
    /// Each site a stop can name; index 0 names none.
    sites: Vec<(u32, u32, u32)>,
    /// The index of each site in `sites`.
    site_index: HashMap<(u32, u32, u32), u32>,
    /// The first construct the target cannot build at present.
    pub missing: Option<String>,
    /// The number of the next piece of data with a name of its own.
    next_data: u32,
    /// The data of the C kind of each kind, under its text.
    pub ckinds: HashMap<String, DataId>,
    /// The data of each shape of a call of C, under its text.
    pub ffi_shapes: HashMap<String, DataId>,
    /// Each C function the code calls, under its name.
    pub c_fns: HashMap<String, FuncId>,
    /// Each function kind C calls back: its bridge and its native shape.
    pub cb_shapes: Vec<(DataKind, FuncId, DataId)>,
    /// The lines of each function compiled so far, in a build with debug
    /// info.
    pub lines: Vec<FnLines>,
    /// The unwind codes of each function compiled so far, for 64-bit
    /// Windows.
    pub unwinds: Vec<FnUnwind>,
}

impl<M: Module> Shared<'_, M>
{
    /// The bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The bit count.
    pub fn platform(&self) -> u16 { self.options.platform_bits }

    /// Note a construct the target cannot build at present; the first one is
    /// reported.
    ///
    /// # Arguments
    ///
    /// * `what` - The construct.
    pub fn missing(&mut self, what: impl Into<String>)
    {
        do_if!(self.missing.is_none(), self.missing = Some(what.into()));
    }

    /// The CLIF kind of a value: a scalar, or an address for a list, a
    /// struct or text.
    ///
    /// # Arguments
    ///
    /// * `kind` - The data kind.
    ///
    /// # Returns
    ///
    /// The CLIF kind.
    pub fn clif_of(&self, kind: DataKind) -> Type
    {
        ret_if!(is_blob(kind), self.ptr);
        match kind.repr()
        {
            DataKind::Int(k) => int_clif(k.bits(self.platform())),
            DataKind::Float(32) => types::F32,
            DataKind::Float(_) => types::F64,
            DataKind::Char => types::I32,
            DataKind::Bool => types::I8,
            _ => self.ptr,
        }
    }

    /// The size of a kind in octets.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// The size.
    pub fn size(&self, kind: DataKind) -> u32 { kind.size(self.platform()) }

    /// A parameter or result of a kind, extended as C extends it.
    ///
    /// # Arguments
    ///
    /// * `kind` - The data kind.
    ///
    /// # Returns
    ///
    /// The parameter.
    pub fn abi(&self, kind: DataKind) -> AbiParam
    {
        let p = AbiParam::new(self.clif_of(kind));
        ret_if!(is_blob(kind) || p.value_type.bits() > 16, p);
        let signed = kind.repr().int_kind().is_some_and(|k| k.signed);
        if signed { p.sext() } else { p.uext() }
    }

    /// The signature of a function of the program, as the C target writes it:
    /// a list or struct result goes out through an address the caller gives
    /// first, and a list or struct parameter comes in through its address.
    ///
    /// # Arguments
    ///
    /// * `params` - The kind of each parameter.
    /// * `ret` - The result kind.
    /// * `record` - Whether the address of a closure record comes last.
    ///
    /// # Returns
    ///
    /// The signature.
    pub fn sig(
        &self, params: &[DataKind], ret: Option<DataKind>, record: bool,
    ) -> Signature
    {
        let mut sig = self.m.make_signature();
        let blob_ret = ret.is_some_and(is_blob);
        do_if!(blob_ret, sig.params.push(AbiParam::new(self.ptr)));
        sig.params.extend(params.iter().map(|k| self.abi(*k)));
        do_if!(record, sig.params.push(AbiParam::new(self.ptr)));
        do_if!(let Some(k) = ret.filter(|_| !blob_ret), sig.returns.push(self.abi(k)));
        sig
    }

    /// The signature of a function of the program.
    ///
    /// # Arguments
    ///
    /// * `f` - The function.
    ///
    /// # Returns
    ///
    /// The signature.
    fn func_sig(&self, f: &Func) -> Signature
    {
        let params: Vec<DataKind> =
            f.locals[..f.params as usize].iter().map(|s| s.kind).collect();
        self.sig(&params, f.ret, false)
    }

    /// A function of the runtime, declared on its first use.
    ///
    /// # Arguments
    ///
    /// * `name` - Its C name.
    /// * `params` - The CLIF kind of each parameter.
    /// * `rets` - The CLIF kind of its result, when it has one.
    ///
    /// # Returns
    ///
    /// The function.
    pub fn runtime_fn(
        &mut self, name: &'static str, params: &[Type], rets: &[Type],
    ) -> FuncId
    {
        ret_if!(let Some(id) = self.runtime.get(name), *id);
        let mut sig = self.m.make_signature();
        let ext = |t: &Type| tern_abi(*t);
        sig.params.extend(params.iter().map(ext));
        sig.returns.extend(rets.iter().map(ext));
        let id = self
            .m
            .declare_function(name, Linkage::Import, &sig)
            .expect("a runtime name");
        self.runtime.insert(name, id);
        id
    }

    /// The data of text, with a zero octet after it, made once.
    ///
    /// # Arguments
    ///
    /// * `octets` - The text.
    ///
    /// # Returns
    ///
    /// The data.
    pub fn string(&mut self, octets: &[u8]) -> DataId
    {
        ret_if!(let Some(id) = self.strings.get(octets), *id);
        let id = self.anon(false);
        let mut blob = Blob::new();
        blob.octets = [octets, &[0]].concat();
        blob.align = 1;
        define(self.m, id, blob).expect("text data");
        self.strings.insert(octets.to_vec(), id);
        id
    }

    /// Declare data with a name of its own.
    ///
    /// # Arguments
    ///
    /// * `writable` - Whether the program writes it.
    ///
    /// # Returns
    ///
    /// The data.
    pub fn anon(&mut self, writable: bool) -> DataId
    {
        self.next_data += 1;
        let name = format!("zk_d{}", self.next_data);
        self.m
            .declare_data(&name, Linkage::Local, writable, false)
            .expect("a data name")
    }

    /// The data of the descriptor of a kind, made on its first use with the
    /// descriptors of the kinds inside it.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// The data.
    pub fn descr(&mut self, kind: DataKind) -> DataId
    {
        let n = self.descr.index_of(kind);
        self.declare_descrs();
        self.descr_ids[n]
    }

    /// Declare the data of each descriptor made so far.
    fn declare_descrs(&mut self)
    {
        while self.descr_ids.len() < self.descr.count()
        {
            let name = format!("zk_k{}", self.descr_ids.len());
            let id = self
                .m
                .declare_data(&name, Linkage::Local, false, false)
                .expect("a descriptor");
            self.descr_ids.push(id);
        }
    }

    /// The index of the site of a source region in the site table, noted on
    /// its first use.
    ///
    /// # Arguments
    ///
    /// * `span` - The region.
    ///
    /// # Returns
    ///
    /// The index, or `None` for a placeholder or a region outside each file.
    pub fn site_of(&mut self, span: Span) -> Option<u32>
    {
        let mark = site_mark(&self.files, span)?;
        ret_if!(let Some(i) = self.site_index.get(&mark), Some(*i));
        let i = self.sites.len() as u32;
        self.sites.push(mark);
        self.site_index.insert(mark, i);
        Some(i)
    }
}

/// A parameter of a runtime function: a narrow integer is extended as C
/// extends an unsigned one.
///
/// # Arguments
///
/// * `t` - The CLIF kind.
///
/// # Returns
///
/// The parameter.
fn tern_abi(t: Type) -> AbiParam
{
    let p = AbiParam::new(t);
    if t.is_int() && t.bits() < 32
    {
        p.uext()
    }
    else
    {
        p
    }
}

/// The CLIF integer kind of a bit count.
///
/// # Arguments
///
/// * `bits` - The bit count.
///
/// # Returns
///
/// The kind.
pub fn int_clif(bits: u16) -> Type
{
    match bits
    {
        8 => types::I8,
        16 => types::I16,
        32 => types::I32,
        _ => types::I64,
    }
}

/// Lower a checked program into a module.
///
/// # Arguments
///
/// * `m` - The module the code goes into.
/// * `module` - The IR, after the ownership check.
/// * `files` - The source files, in source map order.
/// * `options` - The choices of the build.
/// * `grants` - The rights the build bakes in.
/// * `main_name` - The name of the main function: `main` for AOT, `zk_main` for
///   JIT.
/// * `want_clif` - Whether to keep the CLIF text of each function.
///
/// # Returns
///
/// The main function and the CLIF text, or the construct the target cannot
/// build.
pub fn emit<M: Module>(
    m: &mut M, module: &IrModule, files: &[Arc<SourceFile>], options: TargetCOptions,
    grants: &Grants, main_name: &str, want_clif: bool,
) -> Result<Lowered, Error>
{
    // The backends write each cell, so a repeated cell is spelled out first.
    let spelled = spell_repeats(module);
    let module = &spelled;
    let ptr = m.target_config().pointer_type();
    #[rustfmt::skip]
    let mut s = Shared { m, module, options, files: files.to_vec(), ptr,
        descr: Descriptors::default(), descr_ids: Vec::new(), strings: HashMap::new(),
        runtime: HashMap::new(), funcs: Vec::new(), globals: Vec::new(), records: Vec::new(),
        sites: vec![(0, 0, 0)], site_index: HashMap::new(), missing: None, next_data: 0,
        ckinds: HashMap::new(), ffi_shapes: HashMap::new(), c_fns: HashMap::new(),
        cb_shapes: Vec::new(), lines: Vec::new(), unwinds: Vec::new() };
    declare_program(&mut s);
    let mut fctx = FunctionBuilderContext::new();
    let mut clif = String::new();
    for (n, f) in module.funcs.iter().enumerate()
    {
        let sig = s.func_sig(f);
        let id = s.funcs[n];
        let text =
            define_fn(&mut s, &mut fctx, id, sig, want_clif, |g| g.func(n as u32, f));
        clif.push_str(&text);
    }
    // A C lib has no `main`: `zk_lib_init` sets the globals on its first call,
    // and the wrappers find the table by its name.
    let lib = options.lib;
    let mut main_sig = s.m.make_signature();
    if !lib
    {
        let words = [AbiParam::new(types::I32), AbiParam::new(ptr), AbiParam::new(ptr)];
        main_sig.params.extend(words);
        main_sig.returns.push(AbiParam::new(types::I32));
    }
    let main_name = tern!(lib, "zk_lib_init", main_name);
    let main =
        s.m.declare_function(main_name, Linkage::Export, &main_sig)
            .map_err(module_error)?;
    let named = |s: &mut Shared<'_, M>| {
        s.m.declare_data(PROGRAM_TABLE, Linkage::Export, false, false)
    };
    let table = tern!(lib, named(&mut s).map_err(module_error)?, s.anon(false));
    let text = define_fn(&mut s, &mut fctx, main, main_sig, want_clif, |g| {
        tern!(lib, g.lib_init(), g.main(table))
    });
    clif.push_str(&text);
    let exports = tern!(lib, exported(module), Vec::new());
    let made = exports.iter().map(|e| s.cb_shape(e.kind));
    do_if!(made.collect::<Vec<_>>().contains(&None), s.missing(EXPORT_KINDS));
    // A bridge can make the shape of another function kind as it is lowered.
    let mut done = 0;
    while done < s.cb_shapes.len()
    {
        let (kind, bridge, _) = s.cb_shapes[done];
        let mut sig = s.m.make_signature();
        sig.params.extend([AbiParam::new(ptr), AbiParam::new(ptr), AbiParam::new(ptr)]);
        let text =
            define_fn(&mut s, &mut fctx, bridge, sig, want_clif, |g| g.bridge(kind));
        clif.push_str(&text);
        done += 1;
    }
    ret_if!(let Some(what) = s.missing.take(), Err(Error::NotForNative { what }));
    define_descrs(&mut s).map_err(module_error)?;
    define_table(&mut s, table, grants).map_err(module_error)?;
    let rows = s.descr.rows(s.platform());
    let descrs = s.descr_ids.iter().copied().zip(rows).collect();
    let shape_of =
        |e: &Export<'_>| s.cb_shapes.iter().position(|(k, _, _)| *k == e.kind);
    let export_shapes = exports.iter().filter_map(shape_of).collect();
    let lines = line_table(&mut s, main, main_name);
    #[rustfmt::skip]
    return Ok(Lowered::builder().main(main).clif(clif).table(table).descrs(descrs)
        .export_shapes(export_shapes).lines(lines)
        .unwinds(std::mem::take(&mut s.unwinds)).build());
}

/// The line table of a program: each function with code, named as a
/// backtrace shows it, the sites, and the path of each source file.
///
/// # Arguments
///
/// * `s` - The shared state, whose lines the table takes.
/// * `main` - The main function.
/// * `main_name` - Its name.
///
/// # Returns
///
/// The table.
fn line_table<M: Module>(
    s: &mut Shared<'_, M>, main: FuncId, main_name: &str,
) -> LineTable
{
    let named =
        s.funcs.iter().zip(&s.module.funcs).map(|(id, f)| (*id, shown_name(&f.name)));
    let mut names: HashMap<FuncId, String> =
        named.map(|(id, n)| (id, n.to_string())).collect();
    names.insert(main, main_name.to_string());
    let mut funcs = std::mem::take(&mut s.lines);
    funcs.retain(|f| names.contains_key(&f.id));
    funcs.iter_mut().for_each(|f| f.name = names[&f.id].clone());
    let files = s.files.iter().map(|f| debug_path(f)).collect();
    LineTable { funcs, sites: s.sites.clone(), files }
}

/// The error of a module, as the error of the build.
///
/// # Arguments
///
/// * `e` - The error.
///
/// # Returns
///
/// The error.
pub fn module_error(e: impl std::fmt::Display) -> Error
{
    Error::NotForNative { what: e.to_string() }
}

/// Declare each function, each global and the record of each function.
///
/// # Arguments
///
/// * `s` - The shared state.
fn declare_program<M: Module>(s: &mut Shared<'_, M>)
{
    for (n, f) in s.module.funcs.iter().enumerate()
    {
        let sig = s.func_sig(f);
        let id =
            s.m.declare_function(&format!("zk_f{n}"), Linkage::Local, &sig)
                .expect("a function");
        s.funcs.push(id);
    }
    let platform = s.platform();
    for (n, g) in s.module.globals.iter().enumerate()
    {
        let id =
            s.m.declare_data(&format!("zk_g{n}"), Linkage::Local, true, false)
                .expect("a global");
        let mut blob = Blob::new();
        blob.octets = vec![0; g.kind.size(platform).max(1) as usize];
        blob.align = u64::from(g.kind.align(platform).max(1));
        define(s.m, id, blob).expect("a global");
        s.globals.push(id);
    }
    for n in 0..s.module.funcs.len()
    {
        // The wrappers of a C lib call each function C can call through its
        // record.
        let called = s.options.lib && s.module.funcs[n].c_name.is_some();
        let linkage = tern!(called, Linkage::Export, Linkage::Local);
        let id =
            s.m.declare_data(&format!("zk_r{n}"), linkage, false, false)
                .expect("a record");
        let mut blob = Blob::new();
        blob.ptr(Some(Target::Func(s.funcs[n])));
        blob.ptr(None);
        define(s.m, id, blob.done()).expect("a record");
        s.records.push(id);
    }
}

/// Lower one function and define it.
///
/// # Arguments
///
/// * `s` - The shared state.
/// * `fctx` - The builder state, used again for each function.
/// * `id` - The function.
/// * `sig` - Its signature.
/// * `want_clif` - Whether to keep its CLIF text.
/// * `lower` - The lowering of its statements.
///
/// # Returns
///
/// Its CLIF text, or blank.
fn define_fn<'m, M: Module>(
    s: &mut Shared<'m, M>, fctx: &mut FunctionBuilderContext, id: FuncId,
    sig: Signature, want_clif: bool, lower: impl FnOnce(&mut FnGen<'_, 'm, M>),
) -> String
{
    let mut ctx = Context::new();
    ctx.func.signature = sig;
    ctx.func.name = UserFuncName::user(0, id.as_u32());
    {
        let mut g = FnGen::new(s, &mut ctx.func, fctx);
        lower(&mut g);
        g.finish();
    }
    let text = if want_clif
    {
        format!("{}\n", ctx.func.display())
    }
    else
    {
        String::new()
    };
    let defined = tern!(s.missing.is_none(), s.m.define_function(id, &mut ctx), Ok(()));
    do_if!(let Err(e) = defined, s.missing(format!("{e:?}")));
    let base = ctx.func.params.base_srcloc();
    let code = ctx.compiled_code().filter(|_| s.options.debug_info);
    do_if!(let Some(code) = code, s.lines.push(fn_lines(id, code, base)));
    let unwind = ctx.compiled_code().and_then(|c| fn_unwind(id, c, s.m.isa()));
    s.unwinds.extend(unwind);
    text
}

/// The lines of a compiled function: the start of each run of code that
/// belongs to one site, the name to be set later.
///
/// # Arguments
///
/// * `id` - The function.
/// * `code` - Its code.
/// * `base` - The site the relative sites of its code count from.
///
/// # Returns
///
/// The lines.
fn fn_lines(
    id: FuncId, code: &cranelift_codegen::CompiledCode, base: SourceLoc,
) -> FnLines
{
    let abs = |l: &cranelift_codegen::MachSrcLoc| {
        tern!(l.loc.is_rel(), l.loc.relocate(base), l.loc.as_abs())
    };
    let located = code.buffer.get_srclocs_sorted().iter().map(|l| (l.start, abs(l)));
    let rows =
        located.filter(|(_, loc)| !loc.is_default()).map(|(at, loc)| (at, loc.bits()));
    let rows = rows.collect();
    #[rustfmt::skip]
    return FnLines::builder().id(id).name("").size(code.code_info().total_size).rows(rows).build();
}

/// Define each descriptor, with the tables of its fields and members.
///
/// # Arguments
///
/// * `s` - The shared state.
///
/// # Returns
///
/// `Ok`, or the error of the module.
fn define_descrs<M: Module>(s: &mut Shared<'_, M>)
-> cranelift_module::ModuleResult<()>
{
    s.declare_descrs();
    let rows = s.descr.rows(s.platform());
    for (n, row) in rows.iter().enumerate()
    {
        let fields = if row.fields.is_empty() && row.tag != 8 && row.tag != 9
        {
            None
        }
        else
        {
            let id = s.anon(false);
            let mut b = Blob::new();
            for (name, offset, k) in &row.fields
            {
                let text = s.string(name.as_bytes());
                b.ptr(Some(Target::Data(text, 0)));
                b.u64(u64::from(*offset));
                b.ptr(Some(Target::Data(s.descr_ids[*k], 0)));
            }
            define(s.m, id, b.done())?;
            Some(id)
        };
        let members = if row.tag == 12
        {
            let id = s.anon(false);
            let mut b = Blob::new();
            for (value, shown) in &row.members
            {
                let (v, t) = (s.string(value.as_bytes()), s.string(shown.as_bytes()));
                b.ptr(Some(Target::Data(v, 0)));
                b.ptr(Some(Target::Data(t, 0)));
            }
            define(s.m, id, b.done())?;
            Some(id)
        }
        else
        {
            None
        };
        let name = row.name.as_ref().map(|n| s.string(n.as_bytes()));
        let mut b = Blob::new();
        b.u32(row.tag as u32);
        b.u64(u64::from(row.size));
        b.ptr(row.cell.map(|c| Target::Data(s.descr_ids[c], 0)));
        b.u64(row.len as u64);
        b.ptr(fields.map(|f| Target::Data(f, 0)));
        b.ptr(members.map(|f| Target::Data(f, 0)));
        b.ptr(name.map(|f| Target::Data(f, 0)));
        let flags = [row.holds_text, row.holds_borrow, row.owns, row.boxed];
        flags.into_iter().for_each(|f| b.u32(u32::from(f)));
        define(s.m, s.descr_ids[n], b.done())?;
    }
    Ok(())
}

/// A list of texts as data: the address of each text.
///
/// # Arguments
///
/// * `s` - The shared state.
/// * `texts` - The texts.
/// * `null_end` - Whether a NULL row ends the list.
///
/// # Returns
///
/// The data, or the error of the module.
fn text_list<M: Module>(
    s: &mut Shared<'_, M>, texts: &[String], null_end: bool,
) -> cranelift_module::ModuleResult<DataId>
{
    let id = s.anon(false);
    let mut b = Blob::new();
    for t in texts
    {
        let d = s.string(t.as_bytes());
        b.ptr(Some(Target::Data(d, 0)));
    }
    do_if!(null_end, b.ptr(None));
    define(s.m, id, b.done())?;
    Ok(id)
}

/// Define the table of the program, in the order of the fields of
/// `struct zk_program`.
///
/// # Arguments
///
/// * `s` - The shared state.
/// * `table` - The data of the table, declared.
/// * `grants` - The rights the build bakes in.
///
/// # Returns
///
/// `Ok`, or the error of the module.
fn define_table<M: Module>(
    s: &mut Shared<'_, M>, table: DataId, grants: &Grants,
) -> cranelift_module::ModuleResult<()>
{
    let texts = table_texts();
    let grants_id = grants_data(s, grants)?;
    let permission = text_list(s, &texts.permission, false)?;
    let flags = text_list(s, &texts.flags, false)?;
    let trap = text_list(s, &texts.trap, false)?;
    let single: HashMap<&str, DataId> = texts
        .single
        .iter()
        .map(|(name, text)| (*name, s.string(text.as_bytes())))
        .collect();
    let fn_names = fn_names(s)?;
    let (sites, files) = site_tables(s)?;
    let at = |id: DataId| Some(Target::Data(id, 0));
    let mut b = Blob::new();
    let o = s.options;
    b.u32(o.max_depth);
    b.u32(u32::from(o.debug_info));
    b.u64(o.max_heap);
    b.u32(u32::from(o.checks.bits()));
    b.ptr(at(grants_id));
    b.u32(u32::from(grants.prompt));
    b.ptr(at(permission));
    b.ptr(at(flags));
    b.ptr(at(single["ask_text"]));
    b.ptr(at(trap));
    let names = [
        "stopped_text",
        "true_text",
        "false_text",
        "null_text",
        "where_text",
        "in_text",
        "backtrace_text",
        "call_at_text",
        "call_text",
        "call_again_text",
        "calls_more_text",
        "full_hint_text",
        "good_text",
        "fault_text",
        "good_bare",
        "fault_bare",
        "data_text",
        "no_data_text",
        "fn_lead",
        "fn_tail",
    ];
    names.iter().for_each(|n| b.ptr(at(single[n])));
    b.ptr(at(fn_names));
    b.u64(s.module.funcs.len() as u64);
    b.ptr(sites.map(|d| Target::Data(d, 0)));
    b.ptr(files.map(|d| Target::Data(d, 0)));
    define(s.m, table, b.done())
}

/// The grants of the build as data: for read, write and env, whether the right
/// covers each path or name, the count of the listed ones and the list; for
/// ffi, whether it is granted.
///
/// # Arguments
///
/// * `s` - The shared state.
/// * `grants` - The rights.
///
/// # Returns
///
/// The data, or the error of the module.
fn grants_data<M: Module>(
    s: &mut Shared<'_, M>, grants: &Grants,
) -> cranelift_module::ModuleResult<DataId>
{
    let id = s.anon(false);
    let mut b = Blob::new();
    for grant in [&grants.read, &grants.write, &grants.env]
    {
        let (all, items) = match grant
        {
            Grant::None => (false, Vec::new()),
            Grant::All => (true, Vec::new()),
            Grant::Some(list) => (false, list.clone()),
        };
        let list =
            (!items.is_empty()).then(|| text_list(s, &items, false)).transpose()?;
        b.u32(u32::from(all));
        b.u32(items.len() as u32);
        b.ptr(list.map(|d| Target::Data(d, 0)));
    }
    b.u32(u32::from(grants.ffi));
    b.u32(0);
    b.ptr(None);
    define(s.m, id, b.done())?;
    Ok(id)
}

/// The name of each function, with its code, then a NULL row.
///
/// # Arguments
///
/// * `s` - The shared state.
///
/// # Returns
///
/// The data, or the error of the module.
fn fn_names<M: Module>(s: &mut Shared<'_, M>)
-> cranelift_module::ModuleResult<DataId>
{
    let id = s.anon(false);
    let mut b = Blob::new();
    for (n, f) in s.module.funcs.iter().enumerate()
    {
        let name = s.string(shown_name(&f.name).as_bytes());
        b.ptr(Some(Target::Func(s.funcs[n])));
        b.ptr(Some(Target::Data(name, 0)));
    }
    b.ptr(None);
    b.ptr(None);
    define(s.m, id, b.done())?;
    Ok(id)
}

/// The site table and the name of each file it names, in a build with debug
/// info.
///
/// # Arguments
///
/// * `s` - The shared state.
///
/// # Returns
///
/// The two tables, or `None` for a build with no debug info.
fn site_tables<M: Module>(
    s: &mut Shared<'_, M>,
) -> cranelift_module::ModuleResult<(Option<DataId>, Option<DataId>)>
{
    ret_if!(!s.options.debug_info, Ok((None, None)));
    let names: Vec<String> =
        s.files.iter().map(|f| f.name.virtual_path_string()).collect();
    let files = text_list(s, &names, false)?;
    let id = s.anon(false);
    let mut b = Blob::new();
    for (file, line, col) in s.sites.clone()
    {
        b.u32(file);
        b.u32(line);
        b.u32(col);
    }
    b.align = 4;
    define(s.m, id, b)?;
    Ok((Some(id), Some(files)))
}
