//! A C lib of a program: the wrapper of each function that code outside can
//! call, the glue that starts the runtime on the first call and turns a trap
//! into the status of the call, and the header of the lib.
//!
//! Each wrapper returns an `int32_t` status: `ZK_OK`, `1 +` the id of the
//! trap the call stopped on, or `ZK_STOPPED` for each call after such a trap,
//! since the trap can leave the heap and the globals half written. The result
//! goes through a last parameter, an address the wrapper writes unless it is
//! NULL. The values cross as into a function value C calls back: the
//! wrapper hands the address of each parameter to the bridge of the function
//! kind in `zaka_callback.c`.

use std::collections::BTreeMap;
use std::fmt::Write;

use zakac_ir_builder::ir::Dest;
use zakac_ir_builder::ir::Func;
use zakac_ir_builder::ir::Module;
use zakac_ir_builder::ir::Place;
use zakac_ir_builder::ir::Stmt;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::fn_kind::FnKind;
use zakac_semcheck::fn_kind::FnParam;
use zakac_semcheck::foreign::is_c_name;
use zakac_semcheck::kind::DataKind;

use crate::callbacks::Crossing;
use crate::callbacks::cells;
use crate::callbacks::crossing;
use crate::emit::PROGRAM_TABLE;
use crate::emit::comment;
use crate::emit::record_name;
use crate::kinds::c_kind;

/// The construct a build cannot make when a function that code outside can
/// call has kinds no bridge passes.
pub const EXPORT_KINDS: &str =
    "a function that code outside can call whose kinds no bridge passes";

/// One function that code outside can call.
pub struct Export<'a>
{
    /// Its index among the functions of the module.
    pub index: u32,
    /// The function.
    pub func: &'a Func,
    /// The name C calls it with.
    pub c_name: &'a str,
    /// Its function kind, whose bridge the wrapper calls.
    pub kind: DataKind,
    /// The C name of each struct the lib gives to C, under the struct name.
    pub c_structs: &'a BTreeMap<String, String>,
}

/// The functions of a module that code outside can call, in source order.
///
/// # Arguments
///
/// * `module` - The module.
///
/// # Returns
///
/// The functions.
pub fn exported(module: &Module) -> Vec<Export<'_>>
{
    let each = module.funcs.iter().enumerate();
    let named = each.filter_map(|(n, f)| f.c_name.as_deref().map(|c| (n, f, c)));
    named
        .map(|(n, func, c_name)| Export {
            index: n as u32,
            func,
            c_name,
            kind: kind_of(func),
            c_structs: &module.c_structs,
        })
        .collect()
}

/// The names a shared lib gives to C: the name of each function that code
/// outside can call, and `zk_last_trap`.
///
/// # Arguments
///
/// * `exports` - The functions.
///
/// # Returns
///
/// The names.
pub fn api_names(exports: &[Export<'_>]) -> Vec<String>
{
    let names = exports.iter().map(|e| e.c_name.to_string());
    names.chain(["zk_last_trap".to_string()]).collect()
}

/// The statements `zk_lib_init` of a lib runs: the top-level statements,
/// which set the globals, with no drop of them at the end, since the globals
/// live as long as the lib.
///
/// # Arguments
///
/// * `module` - The module.
///
/// # Returns
///
/// The statements.
pub fn init_stmts(module: &Module) -> &[Stmt]
{
    let main = &module.main;
    let global_drop =
        |s: &&Stmt| matches!(s, Stmt::Drop { dest: Dest::Slot(Place::Global(_)), .. });
    let dropped = main.iter().rev().take_while(global_drop).count();
    &main[..main.len() - dropped]
}

/// The C file of a lib whose code Cranelift wrote: the glue and the
/// wrappers, which call the bridges and the records the object gives.
///
/// # Arguments
///
/// * `exports` - The functions that code outside can call.
/// * `shapes` - The index of the bridge of each one's kind.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The C text.
pub fn native_glue(exports: &[Export<'_>], shapes: &[usize], platform: u16) -> String
{
    let mut out = "/* The wrappers of a lib whose code Cranelift wrote. */\n#include \
                   <stdint.h>\n\n#include \"zaka_runtime.h\"\n\n"
        .to_string();
    let mut bridges = shapes.to_vec();
    bridges.sort_unstable();
    bridges.dedup();
    bridges.iter().for_each(|n| {
        _ = writeln!(
            out,
            "void zk_cbb_{n}(void *zk_ret, void **zk_args, uint8_t *zk_rec);"
        )
    });
    out + &lib_source(exports, shapes, platform)
}

/// The function kind of a function, from the kinds of its parameters and
/// its result.
///
/// # Arguments
///
/// * `func` - The function.
///
/// # Returns
///
/// The kind.
fn kind_of(func: &Func) -> DataKind
{
    let param =
        |k: DataKind| FnParam { kind: k, edit: false, fill: false, preposition: None };
    let params = param_kinds(func).into_iter().map(param).collect();
    DataKind::Fn(FnKind::make(params, func.ret))
}

/// The kinds of the parameters of a function.
///
/// # Arguments
///
/// * `func` - The function.
///
/// # Returns
///
/// The kinds, in order.
fn param_kinds(func: &Func) -> Vec<DataKind>
{
    func.locals.iter().take(func.params as usize).map(|s| s.kind).collect()
}

/// The C kinds of the values of the wrappers: the width of the platform-sized
/// kinds, and each struct a wrapper takes, those of its fields first.
struct CKinds
{
    /// Bit count of the platform-sized kinds.
    platform: u16,
    /// The structs, each after the structs of its fields.
    structs: Vec<DataKind>,
    /// The C name of each struct, under the struct name.
    c_names: BTreeMap<String, String>,
}

impl CKinds
{
    /// The C kinds of the wrappers of some functions.
    ///
    /// # Arguments
    ///
    /// * `exports` - The functions.
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The C kinds.
    fn of(exports: &[Export<'_>], platform: u16) -> CKinds
    {
        let c_names = exports.first().map(|e| e.c_structs.clone()).unwrap_or_default();
        let mut kinds = CKinds { platform, structs: Vec::new(), c_names };
        let params = exports.iter().flat_map(|e| param_kinds(e.func));
        let whole = |k: &DataKind| crossing(*k, platform) == Some(Crossing::Struct);
        let structs: Vec<DataKind> = params.filter(whole).collect();
        structs.into_iter().for_each(|k| kinds.note(k));
        kinds
    }

    /// Note a struct, after the structs of its fields.
    ///
    /// # Arguments
    ///
    /// * `kind` - The struct.
    fn note(&mut self, kind: DataKind)
    {
        ret_if!(self.structs.contains(&kind));
        let Some(s) = kind.as_struct()
        else
        {
            return;
        };
        let inner = s.fields().iter().map(|(_, k)| cells(*k).0);
        let inner: Vec<DataKind> = inner.filter(|k| k.as_struct().is_some()).collect();
        inner.into_iter().for_each(|k| self.note(k));
        self.structs.push(kind);
    }

    /// The C kind of a value a wrapper takes or gives: `const char *` for
    /// text, `void *` for an address or a handle, the name of a struct, else
    /// the C scalar.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// The C kind.
    fn c_side(&self, kind: DataKind) -> String
    {
        let crossed = crossing(kind, self.platform);
        let text = matches!(crossed, Some(Crossing::Text | Crossing::OptText));
        ret_if!(text, "const char *".into());
        let handle = matches!(crossed, Some(Crossing::Handle | Crossing::OptHandle));
        let address = matches!(kind.repr(), DataKind::Ptr(_) | DataKind::Null);
        ret_if!(handle || address, "void *".into());
        let at = self.structs.iter().position(|k| *k == kind);
        ret_if!(let Some(at) = at, self.struct_name(at));
        c_kind(kind, self.platform).unwrap_or_else(|| "int".into())
    }

    /// The C name of a struct: the one its declaration gives, else
    /// `zaka_struct_<n>`, which the checks leave to no struct C takes.
    ///
    /// # Arguments
    ///
    /// * `at` - Its place among the structs.
    ///
    /// # Returns
    ///
    /// The name.
    fn struct_name(&self, at: usize) -> String
    {
        let s = self.structs[at].as_struct();
        let named = s.and_then(|s| self.c_names.get(&s.name)).cloned();
        named.unwrap_or_else(|| format!("zaka_struct_{at}"))
    }

    /// The C declaration of each struct, as a `typedef`: the fields in order,
    /// each under its Zaka name when that is a C name, else `f<n>`, and a list
    /// of cells as a C array, as libffi and the bridges place it.
    ///
    /// # Returns
    ///
    /// The C text.
    fn struct_decls(&self) -> String
    {
        let mut out = String::new();
        for (at, kind) in self.structs.iter().enumerate()
        {
            let Some(s) = kind.as_struct()
            else
            {
                continue;
            };
            let _ = write!(out, "\n/* {} */\ntypedef struct\n{{\n", comment(&s.name));
            let fields = s.fields().iter().enumerate();
            fields.for_each(|(i, (name, k))| out.push_str(&self.field(i, name, *k)));
            let _ = writeln!(out, "}} {};", self.struct_name(at));
        }
        out
    }

    /// The C line of one field of a struct.
    ///
    /// # Arguments
    ///
    /// * `i` - Its place among the fields.
    /// * `name` - Its Zaka name.
    /// * `kind` - Its kind.
    ///
    /// # Returns
    ///
    /// The line.
    fn field(&self, i: usize, name: &str, kind: DataKind) -> String
    {
        let named = tern!(is_c_name(name), name.to_string(), format!("f{i}"));
        let (cell, count) = cells(kind);
        let named = tern!(kind.as_list().is_some(), format!("{named}[{count}]"), named);
        format!("    {};\n", declared(&self.c_side(cell), &named))
    }
}

/// The member of `zk_lib_result` that holds a result the bridge gives back,
/// as the bridge writes it.
///
/// # Arguments
///
/// * `kind` - The result kind.
///
/// # Returns
///
/// The member name.
fn result_member(kind: DataKind) -> &'static str
{
    match kind.repr()
    {
        DataKind::Int(k) if k.signed => "i",
        DataKind::Int(_) | DataKind::Char | DataKind::Bool => "u",
        DataKind::Float(32) => "f",
        DataKind::Float(_) => "d",
        _ => "p",
    }
}

/// The C parameters of a wrapper: one for each parameter, then the address
/// of the result, when the function has one.
///
/// # Arguments
///
/// * `func` - The function.
/// * `kinds` - The C kinds.
///
/// # Returns
///
/// The C text of the list, `void` for none.
fn c_params(func: &Func, kinds: &CKinds) -> String
{
    let each = param_kinds(func).into_iter().enumerate();
    let mut params: Vec<String> =
        each.map(|(i, k)| declared(&kinds.c_side(k), &format!("zk_p{i}"))).collect();
    let out = func.ret.map(|r| declared(&kinds.c_side(r), "*zk_out"));
    params.extend(out);
    tern!(params.is_empty(), "void".into(), params.join(", "))
}

/// A C declaration of a name of a C kind, with no space after a `*`.
///
/// # Arguments
///
/// * `kind` - The C kind.
/// * `name` - The name.
///
/// # Returns
///
/// The C text.
fn declared(kind: &str, name: &str) -> String
{
    tern!(kind.ends_with('*'), format!("{kind}{name}"), format!("{kind} {name}"))
}

/// The glue of a lib, which the wrappers share: the start of the runtime on
/// the first call, the jump back to the wrapper on a trap, and the text of
/// that trap.
const GLUE: &str = "
/* ---- the C lib: the start of the runtime, the traps and the wrappers ---- */
#include <setjmp.h>

#include \"zaka_c_sugars.h\"

/* A DLL gives C the wrappers alone. */
#if defined(_WIN32)
#define ZK_LIB_EXPORT __declspec(dllexport)
#else
#define ZK_LIB_EXPORT
#endif

/* The status of a call that went through. */
#define ZK_OK 0
/* The status of each call after a call stopped on a trap. */
#define ZK_STOPPED (-1)

/* A longjmp of 64-bit Windows unwinds each frame it leaves, through the unwind
   tables. With no frame in its buffer the jump restores the registers alone, as
   it does on each other OS, and skips no handler of the frames between, which
   hold none; ZK_LIB_UNWIND keeps the unwinding, through the tables the code of
   Cranelift and of C gives. */
#if defined(_WIN64) && !defined(ZK_LIB_UNWIND)
#define ZK_LIB_NO_UNWIND(buf) (((_JUMP_BUFFER *)(buf))->Frame = 0)
#else
#define ZK_LIB_NO_UNWIND(buf) ((void)(buf))
#endif

/* A result as a bridge gives it back: a word of the kind of the result. */
typedef union
{
    intptr_t i;
    uintptr_t u;
    float f;
    double d;
    void *p;
} zk_lib_result;

extern const zk_program %TABLE%;
void zk_lib_init(void);

/* The state of the lib, which a call reads and writes once zk_lib_lock lets it
   in, one thread at a time. */
/* Where a trap leaves to: the wrapper of the call in progress. */
static jmp_buf *zk_lib_at;
/* 0 before the first call, 1 while the lib runs, 2 once a trap stopped it. */
static int zk_lib_state;
/* The status of the trap that stopped the lib: 1 + the id of the trap. */
static int32_t zk_lib_status;

/**
 * Leave the call in progress on a trap: jump back to its wrapper.
 *
 * @param trap The id of the trap.
 */
_Noreturn static void zk_lib_leave(int32_t trap)
{
    zk_lib_status = trap + 1;
    longjmp(*zk_lib_at, 1);
}

/**
 * Start the runtime and set the globals, on the first call alone.
 */
static void zk_lib_begin(void)
{
    RET_IF(zk_lib_state != 0);
    zk_lib_state = 1;
    zk_start(0, NULL, &%TABLE%);
    zk_on_trap(zk_lib_leave);
    zk_lib_init();
}

/**
 * End a call: the jump target goes back to the call around it, and the lib
 * goes to the next thread that waits once the calls of this one end.
 *
 * @param outer The jump target of the call around this one, or NULL.
 * @param status The status of the call.
 * @return The status.
 */
static int32_t zk_lib_end(jmp_buf *outer, int32_t status)
{
    zk_lib_at = outer;
    zk_lib_unlock();
    return status;
}

/**
 * End a call that stopped on a trap: the lib remains stopped.
 *
 * @param outer The jump target of the call around this one, or NULL.
 * @return The status of the trap.
 */
static int32_t zk_lib_stop(jmp_buf *outer)
{
    zk_lib_state = 2;
    zk_flush();
    return zk_lib_end(outer, zk_lib_status);
}

/**
 * The text of the trap the lib stopped on.
 *
 * @return The text, or NULL while no call stopped on a trap.
 */
ZK_LIB_EXPORT const char *zk_last_trap(void) { return zk_trap_text(); }
";

/// The C text a lib adds to `zaka_callback.c`: the glue, then a wrapper for
/// each function that code outside can call.
///
/// # Arguments
///
/// * `exports` - The functions.
/// * `shapes` - The index of the shape of each one's kind.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The C text.
pub fn lib_source(exports: &[Export<'_>], shapes: &[usize], platform: u16) -> String
{
    let kinds = CKinds::of(exports, platform);
    let mut out = GLUE.replace("%TABLE%", PROGRAM_TABLE);
    out.push_str(&kinds.struct_decls());
    let records = exports.iter().map(|e| record_name(e.index));
    records.for_each(|r| _ = writeln!(out, "extern const zk_fnrec {r};"));
    let each = exports.iter().zip(shapes);
    each.for_each(|(e, shape)| out.push_str(&wrapper(e, *shape, &kinds)));
    out
}

/// The wrapper of one function: it starts the runtime on the first call,
/// calls the bridge of the function kind with the address of each
/// parameter, writes the result, and returns the status.
///
/// # Arguments
///
/// * `e` - The function.
/// * `shape` - The index of the shape of its kind.
/// * `kinds` - The C kinds.
///
/// # Returns
///
/// The C text.
fn wrapper(e: &Export<'_>, shape: usize, kinds: &CKinds) -> String
{
    let count = e.func.params as usize;
    let args: Vec<String> = (0..count).map(|i| format!("&zk_p{i}")).collect();
    let args = tern!(args.is_empty(), "NULL".into(), args.join(", "));
    let out = e.func.ret.map_or_else(String::new, |r| {
        let (c, m) = (kinds.c_side(r), result_member(r));
        format!("    DO_IF(zk_out != NULL, *zk_out = ({c})zk_r.{m});\n")
    });
    format!(
        "\n/* {} */\nZK_LIB_EXPORT int32_t {}({})\n{{\n    void *zk_args[] = \
         {{{args}}};\n    zk_lib_result zk_r;\n    jmp_buf zk_here;\n    zk_lib_lock();\n    \
         jmp_buf *zk_outer = zk_lib_at;\n    RET_IF(zk_lib_state == 2, \
         zk_lib_end(zk_outer, ZK_STOPPED));\n    if \
         (setjmp(zk_here) != 0)\n        return zk_lib_stop(zk_outer);\n    ZK_LIB_NO_UNWIND(zk_here);\n    zk_lib_at \
         = &zk_here;\n    zk_lib_begin();\n    zk_cbb_{shape}(&zk_r, zk_args, \
         (uint8_t *)&{});\n    zk_flush();\n{out}    return zk_lib_end(zk_outer, \
         ZK_OK);\n}}\n",
        comment(&e.func.name),
        e.c_name,
        c_params(e.func, kinds),
        record_name(e.index),
    )
}

/// The header of a lib: the status words, the text of the last trap, and the
/// declaration of each wrapper.
///
/// # Arguments
///
/// * `exports` - The functions that code outside can call.
/// * `platform` - Bit count of the platform-sized kinds.
/// * `name` - The name of the lib, for the guard and the comment.
/// * `links` - What a program that links the static lib links with it.
///
/// # Returns
///
/// The C text.
pub fn header(
    exports: &[Export<'_>], platform: u16, name: &str, links: &[String],
) -> String
{
    let guard: String = name
        .chars()
        .map(|c| tern!(c.is_ascii_alphanumeric(), c.to_ascii_uppercase(), '_'))
        .collect();
    let links = tern!(links.is_empty(), String::new(), format!(" {}", links.join(" ")));
    let mut out = format!(
        "/* The functions of the Zaka lib {name} that C can call, as zakac wrote \
         them.\n * A program that links the static lib links with it:{links} \
         */\n#ifndef ZAKA_LIB_{guard}_H\n#define ZAKA_LIB_{guard}_H\n\n#include \
         <stdint.h>\n\n#ifdef __cplusplus\nextern \"C\" {{\n#endif\n\n/** The status \
         of a call that went through. A trap gives 1 + the id of the trap. \
         */\n#define ZK_OK 0\n/** The status of each call after a call stopped on a \
         trap. */\n#define ZK_STOPPED (-1)\n\n/**\n * The text of the trap the lib \
         stopped on.\n *\n * @return The text, or NULL while no call stopped on a \
         trap.\n */\nconst char *zk_last_trap(void);\n"
    );
    let kinds = CKinds::of(exports, platform);
    out.push_str(&kinds.struct_decls());
    for e in exports
    {
        let _ = write!(
            out,
            "\n/**\n * {}\n *\n * @return The status of the call.\n */\nint32_t \
             {}({});\n",
            comment(&e.func.name),
            e.c_name,
            c_params(e.func, &kinds)
        );
    }
    out.push_str("\n#ifdef __cplusplus\n}\n#endif\n\n#endif\n");
    out
}
