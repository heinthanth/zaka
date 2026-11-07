use std::fmt::Write;
use std::sync::Arc;

use bon::Builder;
use zakac_borrow_checker::options::OverflowMode;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::target::kind_not_for_c;
use zakac_host_errors::Error;
use zakac_i18n::t;
use zakac_ir_builder::ir::Func;
use zakac_ir_builder::ir::Module;
use zakac_ir_builder::ir::Place;
use zakac_ir_builder::ir::Stmt;
use zakac_ir_builder::ir::fn_shape;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::fn_kind::FnKind;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::value::FALSE_TEXT;
use zakac_semcheck::value::NULL_TEXT;
use zakac_semcheck::value::TRUE_TEXT;
use zakac_span::source_file::SourceFile;
use zakac_span::span::Span;

use crate::descriptors::Descriptors;
use crate::descriptors::FAULT_BARE;
use crate::descriptors::FAULT_TEXT;
use crate::descriptors::FN_LEAD;
use crate::descriptors::FN_TAIL;
use crate::descriptors::GOOD_BARE;
use crate::descriptors::GOOD_TEXT;
use crate::ffi::FFI_SOURCE;
use crate::ffi::FfiCall;
use crate::ffi::ffi_source;
use crate::ffi::wrapper_proto;
use crate::kinds::c_kind;
use crate::kinds::is_blob;
use crate::kinds::mem_name;
use crate::kinds::ops_name;
use crate::options::TargetCOptions;

/// The header each generated file includes.
pub const KINDS_HEADER: &str = "zaka_kinds.h";
/// The file of generated descriptors and texts.
pub const KINDS_SOURCE: &str = "zaka_kinds.c";
/// The file that holds `main`.
pub const MAIN_SOURCE: &str = "main.c";

/// One generated C file.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct CFile
{
    /// The file name, with no folder.
    #[builder(into)]
    pub name: String,
    /// The C text.
    pub text: String,
}

/// The reason a program was not written as C.
#[derive(Debug)]
pub enum NotBuilt
{
    /// The program holds a kind the C target cannot build, at these
    /// places.
    Kinds(Vec<Diagnostic>),
    /// A construct the C target cannot build at present.
    Missing(Error),
}

/// Translate a checked program to C: a file for each module, the shared
/// header, the file of generated descriptors and texts, and `main.c`.
///
/// # Arguments
///
/// * `module` - The IR, after the ownership check.
/// * `files` - The source files of the program, in source map order: the module
///   of a function or a global is the file its span starts in.
/// * `options` - The choices of the C target.
///
/// # Returns
///
/// The files, or the reason the program was not written as C.
pub fn emit(
    module: &Module, files: &[Arc<SourceFile>], options: TargetCOptions,
) -> Result<Vec<CFile>, NotBuilt>
{
    let mut shared = Shared::builder().module(module).options(options).build();
    let mut units = vec![String::new(); files.len().max(1)];
    let unit_of = |lo| files.iter().rposition(|f| f.start_pos <= lo).unwrap_or(0);
    for (n, g) in module.globals.iter().enumerate()
    {
        shared.span = g.span;
        let decl = shared.decl(&global_name(n as u32), g.kind, &g.name);
        let _ = writeln!(units[unit_of(g.span.lo)], "{decl};");
    }
    for (n, f) in module.funcs.iter().enumerate()
    {
        let text = FnGen::new(&mut shared, Some(f)).func(n as u32);
        units[unit_of(f.span.lo)].push_str(&text);
    }
    let main = FnGen::new(&mut shared, None).main();
    let header = shared.header();
    let kinds = shared.kinds_source();
    let ffi = ffi_source(&module.foreign, &shared.ffi_calls, options.platform_bits);
    ret_if!(!shared.diags.is_empty(), Err(NotBuilt::Kinds(shared.diags)));
    let missing =
        shared.missing.map(|what| NotBuilt::Missing(Error::NotForTargetC { what }));
    ret_if!(let Some(reason) = missing, Err(reason));
    let mut out: Vec<CFile> = units
        .into_iter()
        .enumerate()
        .map(|(n, text)| {
            let name = files.get(n).map(|f| f.name.to_string()).unwrap_or_default();
            unit_file(n, &name, &text)
        })
        .collect();
    out.push(CFile::builder().name(KINDS_HEADER).text(header).build());
    out.push(CFile::builder().name(KINDS_SOURCE).text(kinds).build());
    out.push(CFile::builder().name(MAIN_SOURCE).text(main).build());
    do_if!(
        !module.foreign.is_empty(),
        out.push(CFile::builder().name(FFI_SOURCE).text(ffi).build())
    );
    Ok(out)
}

/// The file of one module.
///
/// # Arguments
///
/// * `n` - The number of the module.
/// * `name` - The name of its source file.
/// * `text` - Its globals and functions.
///
/// # Returns
///
/// The file, `m<n>.c`.
fn unit_file(n: usize, name: &str, text: &str) -> CFile
{
    let text =
        format!("/* {} */\n#include \"{KINDS_HEADER}\"\n\n{text}", comment(name));
    CFile::builder().name(format!("m{n}.c")).text(text).build()
}

/// The C name of a global.
///
/// # Arguments
///
/// * `n` - The index of the global.
///
/// # Returns
///
/// The name.
pub fn global_name(n: u32) -> String { format!("zk_g{n}") }

/// The C name of a function.
///
/// # Arguments
///
/// * `n` - The index of the function.
///
/// # Returns
///
/// The name.
pub fn func_name(n: u32) -> String { format!("zk_f{n}") }

/// Text that can stand inside a C comment.
///
/// # Arguments
///
/// * `text` - The text.
///
/// # Returns
///
/// The text, with each `*/` broken up.
pub fn comment(text: &str) -> String { text.replace("*/", "* /") }

/// A C string literal of text: quotes, backslashes and control characters
/// escaped, each other character as its UTF-8 octets.
///
/// # Arguments
///
/// * `text` - The text.
///
/// # Returns
///
/// The literal, quotes included.
pub fn c_string(text: &str) -> String
{
    let mut out = String::from("\"");
    for c in text.chars()
    {
        match c
        {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            // C11 reads ??) and kin as trigraphs; \? is a ? in each spot.
            '?' => out.push_str("\\?"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            // An octal escape reads three digits at most, so the text after
            // it never joins it.
            c if (c as u32) < 0x20 || c == '\u{7f}' =>
            {
                let _ = write!(out, "\\{:03o}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The name of a function as a print of its value writes it: the name as
/// written, without the overload number and the file the compiler adds.
///
/// # Arguments
///
/// * `name` - The name of the function in the IR.
///
/// # Returns
///
/// The name.
fn shown_name(name: &str) -> &str { name.split(['#', '@']).next().unwrap_or(name) }

/// State shared across the functions of a program.
#[derive(Builder)]
pub(crate) struct Shared<'m>
{
    /// The program.
    pub(crate) module: &'m Module,
    /// The choices of the C target.
    pub(crate) options: TargetCOptions,
    /// The first construct the C target cannot build at present.
    pub(crate) missing: Option<String>,
    /// Whether the program holds a 128-bit integer, which C11 lacks.
    #[builder(default)]
    wide: bool,
    /// The kind descriptors the program needs.
    #[builder(default)]
    pub(crate) descr: Descriptors,
    /// The place in the source of the statement or slot being written.
    #[builder(default)]
    pub(crate) span: Span,
    /// A ZK0801 for each place that holds a kind the C target cannot build.
    #[builder(default)]
    diags: Vec<Diagnostic>,
    /// Each C call shape met so far; its index names its wrapper.
    #[builder(default)]
    pub(crate) ffi_calls: Vec<FfiCall>,
}

impl Shared<'_>
{
    /// The bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The bit count.
    pub(crate) fn platform(&self) -> u16 { self.options.platform_bits }

    /// Note a construct the C target cannot build at present; the first one
    /// is reported.
    ///
    /// # Arguments
    ///
    /// * `what` - The construct.
    ///
    /// # Returns
    ///
    /// C text that stands in for it.
    pub(crate) fn missing(&mut self, what: impl Into<String>) -> String
    {
        do_if!(self.missing.is_none(), self.missing = Some(what.into()));
        "0".into()
    }

    /// The C kind of a value of a data kind: the C scalar, or
    /// `unsigned char *` for a block, which a value names through its address.
    ///
    /// # Arguments
    ///
    /// * `kind` - The data kind.
    ///
    /// # Returns
    ///
    /// The C kind name.
    pub(crate) fn kind(&mut self, kind: DataKind) -> String
    {
        let platform = self.platform();
        let repr = kind.repr();
        do_if!(repr.is_int() && repr.bits(platform) > 64, self.wide = true);
        if matches!(repr, DataKind::Float(16 | 128))
        {
            let seen = self.diags.last().is_some_and(|d| d.span == self.span);
            do_if!(!seen, self.diags.push(kind_not_for_c(self.span, &repr.name())));
            return "double".into();
        }
        ret_if!(is_blob(kind), "unsigned char *".into());
        c_kind(kind, platform).unwrap_or_else(|| {
            self.missing(format!("the kind {}", kind.name()));
            "int".into()
        })
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
    pub(crate) fn size(&self, kind: DataKind) -> u32 { kind.size(self.platform()) }

    /// The name of an integer kind in the names of its runtime functions,
    /// noting a kind that is no integer.
    ///
    /// # Arguments
    ///
    /// * `kind` - The integer kind.
    ///
    /// # Returns
    ///
    /// The name, such as `i32`.
    pub(crate) fn ops(&mut self, kind: DataKind) -> String
    {
        let platform = self.platform();
        ops_name(kind, platform).unwrap_or_else(|| {
            self.missing(format!("integer operations on {}", kind.name()))
        })
    }

    /// The name of a scalar kind in `zk_ld_<N>` and `zk_st_<N>`.
    ///
    /// # Arguments
    ///
    /// * `kind` - The scalar kind.
    ///
    /// # Returns
    ///
    /// The name, such as `i32` or `p`.
    pub(crate) fn mem(&mut self, kind: DataKind) -> String
    {
        self.kind(kind);
        let platform = self.platform();
        mem_name(kind, platform).unwrap_or_else(|| {
            self.missing(format!("a load or store of {}", kind.name()))
        })
    }

    /// The declaration of storage for a value of a kind, zeroed, with its
    /// Zaka name in a comment: a C scalar, or an aligned block of octets.
    ///
    /// # Arguments
    ///
    /// * `c_name` - The C name.
    /// * `kind` - The data kind.
    /// * `name` - The Zaka name.
    ///
    /// # Returns
    ///
    /// The declaration, with no semicolon.
    pub(crate) fn decl(&mut self, c_name: &str, kind: DataKind, name: &str) -> String
    {
        format!("{} = {{0}}", self.bare_decl(c_name, kind, name))
    }

    /// The declaration of storage, with no initializer.
    ///
    /// # Arguments
    ///
    /// * `c_name` - The C name.
    /// * `kind` - The data kind.
    /// * `name` - The Zaka name.
    ///
    /// # Returns
    ///
    /// The declaration, with no semicolon.
    fn bare_decl(&mut self, c_name: &str, kind: DataKind, name: &str) -> String
    {
        let note = comment(name);
        ret_if!(!is_blob(kind), format!("{} {c_name} /* {note} */", self.kind(kind)));
        let (size, align) = (self.size(kind).max(1), kind.align(self.platform()));
        format!("_Alignas({align}) unsigned char {c_name}[{size}] /* {note} */")
    }

    /// The signature of a function. A list or struct parameter comes in through
    /// its address; one the function can change is copied into a local of
    /// its own, as the VM copies it into the callee's frame. A list or
    /// struct result goes out through the address `zk_ret` the caller gives.
    ///
    /// # Arguments
    ///
    /// * `n` - The index of the function.
    ///
    /// # Returns
    ///
    /// The signature, with no semicolon.
    pub(crate) fn signature(&mut self, n: u32) -> String
    {
        let f = &self.module.funcs[n as usize];
        let blob_ret = f.ret.is_some_and(is_blob);
        let ret =
            f.ret.filter(|_| !blob_ret).map_or_else(|| "void".into(), |k| self.kind(k));
        let mut params = Vec::new();
        do_if!(blob_ret, params.push("unsigned char *zk_ret".to_string()));
        for (i, s) in f.locals[..f.params as usize].iter().enumerate()
        {
            let name = tern!(copied_param(f, i), format!("zk_a{i}"), format!("l{i}"));
            params.push(format!("{} {name}", self.kind(s.kind)));
        }
        let params = tern!(params.is_empty(), "void".into(), params.join(", "));
        format!("{ret} {}({params})", func_name(n))
    }

    /// The C kind of a pointer to a function of a function kind, for a call
    /// through a function value.
    ///
    /// # Arguments
    ///
    /// * `f` - The function kind.
    ///
    /// # Returns
    ///
    /// The C kind, such as `int64_t (*)(int64_t, unsigned char *)`.
    pub(crate) fn fn_pointer(&mut self, f: &FnKind) -> String
    {
        let (params, ret) = fn_shape(f);
        let blob_ret = ret.is_some_and(is_blob);
        let r =
            ret.filter(|_| !blob_ret).map_or_else(|| "void".into(), |k| self.kind(k));
        let mut list: Vec<String> = Vec::new();
        do_if!(blob_ret, list.push("unsigned char *".into()));
        list.extend(params.iter().map(|k| self.kind(*k)));
        let list = tern!(list.is_empty(), "void".into(), list.join(", "));
        format!("{r} (*)({list})")
    }

    /// The shared header: the check level, the runtime, and the
    /// declaration of each global, function and descriptor.
    ///
    /// # Returns
    ///
    /// The C text.
    fn header(&mut self) -> String
    {
        let level = self.options.checks.macro_name();
        let bits = self.platform();
        let mut out = format!(
            "/* The kinds, globals and functions of the program. */\n#ifndef \
             ZAKA_KINDS_H\n#define ZAKA_KINDS_H\n\n#define {level} 1\n#include \
             \"zaka_rt.h\"\n\n_Static_assert(sizeof(void *) * 8 == {bits}, \
             \"--platform-bits must equal the width of an address\");\n\n"
        );
        let module = self.module;
        for (n, g) in module.globals.iter().enumerate()
        {
            let decl = self.bare_decl(&global_name(n as u32), g.kind, &g.name);
            let _ = writeln!(out, "extern {decl};");
        }
        for (n, f) in module.funcs.iter().enumerate()
        {
            let proto = self.signature(n as u32);
            let _ = writeln!(out, "{proto}; /* {} */", comment(&f.name));
        }
        for (n, call) in self.ffi_calls.iter().enumerate()
        {
            let f = &module.foreign[call.func as usize];
            let proto = wrapper_proto(f, call, n as u32, bits);
            let _ = writeln!(
                out,
                "{proto}; /* {}: {} */",
                comment(&f.lib),
                comment(&f.name)
            );
        }
        out.push_str(&self.descr.declarations());
        if self.wide
        {
            let error = c_string(&t!("target_c.no_int128"));
            let _ =
                writeln!(out, "\n#ifndef __SIZEOF_INT128__\n#error {error}\n#endif");
        }
        out.push_str("\n#endif\n");
        out
    }

    /// The file of generated data: the trap templates from the locale, the
    /// words a print writes, the name of each function and each descriptor.
    ///
    /// # Returns
    ///
    /// The C text.
    fn kinds_source(&self) -> String
    {
        let mut out = format!("#include \"{KINDS_HEADER}\"\n\n");
        out.push_str("const char *const zk_trap_text[ZK_TRAP_COUNT] = {\n");
        do_for!((name, text) in trap_texts(), {
            let _ = writeln!(out, "    [{name}] = {},", c_string(&text));
        });
        out.push_str("};\n");
        let words = [
            ("zk_true_text", TRUE_TEXT),
            ("zk_false_text", FALSE_TEXT),
            ("zk_null_text", NULL_TEXT),
            ("zk_good_text", GOOD_TEXT),
            ("zk_fault_text", FAULT_TEXT),
            ("zk_good_bare", GOOD_BARE),
            ("zk_fault_bare", FAULT_BARE),
            ("zk_fn_lead", FN_LEAD),
            ("zk_fn_tail", FN_TAIL),
        ];
        let stopped = c_string(&t!("target_c.stopped"));
        let _ = writeln!(out, "const char *const zk_stopped_text = {stopped};");
        do_for!((name, text) in words, {
            let _ = writeln!(out, "const char {name}[] = {};", c_string(text));
        });
        out.push_str("const zk_fn_name zk_fn_names[] = {\n");
        for (n, f) in self.module.funcs.iter().enumerate()
        {
            let name = c_string(shown_name(&f.name));
            let _ = writeln!(out, "    {{(zk_fn){}, {name}}},", func_name(n as u32));
        }
        out.push_str("    {NULL, NULL}\n};\n");
        let count = self.module.funcs.len();
        let _ = writeln!(out, "const size_t zk_fn_count = {count};\n");
        out.push_str(&self.descr.definitions(self.platform()));
        out
    }
}

/// Whether a parameter is copied into a local of its own: a list or struct
/// parameter the function can change.
///
/// # Arguments
///
/// * `f` - The function.
/// * `i` - The index of the parameter.
///
/// # Returns
///
/// `true` when it is copied.
fn copied_param(f: &Func, i: usize) -> bool
{
    is_blob(f.locals[i].kind) && !f.shared_params.contains(&(i as u32))
}

/// The trap templates from the locale, each with the name of its member
/// of `enum zk_trap`.
///
/// # Returns
///
/// The member names and the templates.
fn trap_texts() -> Vec<(&'static str, String)>
{
    let pairs = [
        ("ZK_TRAP_ZERO_DIVISOR", t!("target_c.trap.zero_divisor")),
        ("ZK_TRAP_SHIFT_TOO_FAR", t!("target_c.trap.shift_too_far")),
        ("ZK_TRAP_STEP_NOT_POSITIVE", t!("target_c.trap.step_not_positive")),
        ("ZK_TRAP_OVERFLOW", t!("target_c.trap.overflow")),
        ("ZK_TRAP_INDEX_OUT_OF_RANGE", t!("target_c.trap.index_out_of_range")),
        ("ZK_TRAP_CHAR_OUT_OF_RANGE", t!("target_c.trap.char_out_of_range")),
        ("ZK_TRAP_END_BEFORE_START", t!("target_c.trap.end_before_start")),
        ("ZK_TRAP_NULL_CALL", t!("target_c.trap.null_call")),
        ("ZK_TRAP_NO_CELL_TO_REMOVE", t!("target_c.trap.no_cell_to_remove")),
        ("ZK_TRAP_NO_CHAR_TO_REMOVE", t!("target_c.trap.no_char_to_remove")),
        ("ZK_TRAP_NO_PLACE_TO_INSERT", t!("target_c.trap.no_place_to_insert")),
        ("ZK_TRAP_SLICE_OUT_OF_RANGE", t!("target_c.trap.slice_out_of_range")),
        ("ZK_TRAP_TEXT_CUT", t!("target_c.trap.text_cut")),
        ("ZK_TRAP_BAD_UTF8", t!("target_c.trap.bad_utf8")),
        ("ZK_TRAP_NOT_GOOD", t!("target_c.trap.not_good")),
        ("ZK_TRAP_NOT_GOOD_BARE", t!("target_c.trap.not_good_bare")),
        ("ZK_TRAP_NOT_FAULT", t!("target_c.trap.not_fault")),
        ("ZK_TRAP_NULL_DEREF", t!("target_c.trap.null_deref")),
        ("ZK_TRAP_OUT_OF_MEM", t!("target_c.trap.out_of_mem")),
        ("ZK_TRAP_OVERLAP", t!("target_c.trap.overlap")),
    ];
    pairs.into_iter().map(|(n, t)| (n, t.to_string())).collect()
}

/// The generator of one function, or of `main`.
pub(crate) struct FnGen<'s, 'm>
{
    /// State shared across functions.
    pub(crate) shared: &'s mut Shared<'m>,
    /// The function, or `None` for the top-level statements.
    pub(crate) func: Option<&'m Func>,
    /// The C text so far.
    pub(crate) out: String,
    /// The nesting depth, for indentation.
    pub(crate) depth: usize,
    /// The label number of each loop around the statement being written,
    /// innermost last; `continue` jumps to the label.
    pub(crate) loops: Vec<u32>,
    /// The number of the next label or local of its own.
    next: u32,
}

impl<'s, 'm> FnGen<'s, 'm>
{
    /// A generator.
    ///
    /// # Arguments
    ///
    /// * `shared` - State shared across functions.
    /// * `func` - The function, or `None` for the top-level statements.
    ///
    /// # Returns
    ///
    /// The generator.
    fn new(shared: &'s mut Shared<'m>, func: Option<&'m Func>) -> Self
    {
        FnGen { shared, func, out: String::new(), depth: 1, loops: Vec::new(), next: 0 }
    }

    /// The bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The bit count.
    pub(crate) fn platform(&self) -> u16 { self.shared.platform() }

    /// Whether an overflow stops the program.
    ///
    /// # Returns
    ///
    /// `true` under `--overflow stop`.
    pub(crate) fn stop(&self) -> bool
    {
        self.shared.options.overflow == OverflowMode::Stop
    }

    /// A fresh number for a label or a local of its own.
    ///
    /// # Returns
    ///
    /// The number.
    pub(crate) fn fresh(&mut self) -> u32
    {
        self.next += 1;
        self.next
    }

    /// Write one line at the current depth.
    ///
    /// # Arguments
    ///
    /// * `line` - The line.
    pub(crate) fn line(&mut self, line: &str)
    {
        let _ = writeln!(self.out, "{:w$}{line}", "", w = self.depth * 4);
    }

    /// A local of its own for a value of a kind, zeroed: a C scalar or an
    /// aligned block of octets.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// Its C name, which names the address of a block.
    pub(crate) fn temp(&mut self, kind: DataKind) -> String
    {
        let name = format!("zk_t{}", self.fresh());
        let decl = self.shared.decl(&name, kind, "");
        self.line(&format!("{decl};"));
        name
    }

    /// The C text of a slot.
    ///
    /// # Arguments
    ///
    /// * `p` - The slot.
    ///
    /// # Returns
    ///
    /// The C name; for a block, its address.
    pub(crate) fn place(&self, p: Place) -> String
    {
        match p
        {
            Place::Global(n) => global_name(n),
            Place::Local(n) => format!("l{n}"),
        }
    }

    /// The kind of a slot.
    ///
    /// # Arguments
    ///
    /// * `p` - The slot.
    ///
    /// # Returns
    ///
    /// The kind.
    pub(crate) fn place_kind(&self, p: Place) -> DataKind
    {
        match p
        {
            Place::Global(n) => self.shared.module.globals[n as usize].kind,
            Place::Local(n) =>
            {
                self.func.map_or(DataKind::Bool, |f| f.locals[n as usize].kind)
            }
        }
    }

    /// The address of a slot, as `unsigned char *`.
    ///
    /// # Arguments
    ///
    /// * `p` - The slot.
    ///
    /// # Returns
    ///
    /// The C text.
    pub(crate) fn addr_of(&self, p: Place) -> String
    {
        let name = self.place(p);
        tern!(is_blob(self.place_kind(p)), name, format!("((unsigned char *)&{name})"))
    }

    /// Generate the function.
    ///
    /// # Arguments
    ///
    /// * `n` - The index of the function.
    ///
    /// # Returns
    ///
    /// The C text of the function.
    fn func(mut self, n: u32) -> String
    {
        let f = self.func.expect("a function");
        let proto = self.shared.signature(n);
        for (i, s) in f.locals.iter().enumerate()
        {
            let param = i < f.params as usize;
            let copied = param && copied_param(f, i);
            do_if!(param && !copied, continue);
            self.shared.span = s.span;
            let decl = self.shared.decl(&format!("l{i}"), s.kind, &s.name);
            self.line(&format!("{decl};"));
            let size = self.shared.size(s.kind);
            do_if!(copied, self.line(&format!("zk_put_block(l{i}, zk_a{i}, {size});")));
        }
        self.block(&f.block);
        let returns = matches!(f.block.last(), Some(Stmt::Return { .. }));
        let result = f.result.filter(|_| !returns && f.ret.is_some());
        do_if!(let Some(r) = result, self.give_back(&format!("l{r}"), f.ret));
        format!("/* {} */\n{proto}\n{{\n{}}}\n\n", comment(&f.name), self.out)
    }

    /// Hand the result of the function back: a scalar as the C result, a
    /// list or struct into the slot the caller gave.
    ///
    /// # Arguments
    ///
    /// * `value` - The C text of the result.
    /// * `ret` - The result kind.
    pub(crate) fn give_back(&mut self, value: &str, ret: Option<DataKind>)
    {
        let Some(kind) = ret.filter(|k| is_blob(*k))
        else
        {
            return self.line(&format!("return {value};"));
        };
        let size = self.shared.size(kind);
        self.line(&format!("zk_put_block(zk_ret, {value}, {size});"));
        self.line("return;");
    }

    /// Generate `main`: the arguments and the environment of the start
    /// block, the top-level statements, which set the globals and run the
    /// start block, then the exit status.
    ///
    /// # Returns
    ///
    /// The C text of `main.c`.
    fn main(mut self) -> String
    {
        let module = self.shared.module;
        self.line("zk_start(argc, argv);");
        do_if!(let Some(g) = module.host.args, {
            self.line(&format!("zk_text_list({}, argc, argv);", global_name(g)));
        });
        do_if!(let Some(g) = module.host.env, {
            self.line("int zk_envc = 0;");
            self.line("while (envp && envp[zk_envc]) zk_envc++;");
            self.line(&format!("zk_text_list({}, zk_envc, envp);", global_name(g)));
        });
        self.block(&module.main);
        let status = module.host.exit.map_or_else(
            || "0".into(),
            |g| format!("(int)(uint8_t){}", global_name(g)),
        );
        self.line(&format!("return zk_finish({status});"));
        format!(
            "#include \"{KINDS_HEADER}\"\n\nint main(int argc, char **argv, char \
             **envp)\n{{\n    (void)envp;\n{}}}\n",
            self.out
        )
    }

    /// Generate a list of statements.
    ///
    /// # Arguments
    ///
    /// * `stmts` - The statements.
    pub(crate) fn block(&mut self, stmts: &[Stmt])
    {
        do_for!(s in stmts, self.stmt(s));
    }

    /// Generate a nested list of statements in braces.
    ///
    /// # Arguments
    ///
    /// * `stmts` - The statements.
    pub(crate) fn braced(&mut self, stmts: &[Stmt])
    {
        self.depth += 1;
        self.block(stmts);
        self.depth -= 1;
    }
}
