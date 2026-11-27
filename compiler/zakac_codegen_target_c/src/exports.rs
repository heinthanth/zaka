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

use zakac_borrow_checker::runs_free;
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
use crate::callbacks::in_c_form;
use crate::emit::comment;
use crate::emit::record_name;
use crate::kinds::c_kind;
use crate::lib_names::LibNames;
use crate::lib_names::trap_names;

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
    /// Whether its calls run on several threads at once with no lock: it
    /// reaches no mutable global and calls no C.
    pub free: bool,
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
            free: runs_free(module, n as u32),
        })
        .collect()
}

/// The names a shared lib gives to C: the name of each function that code
/// outside can call, the reset, the last trap and the free of the lib, and
/// `zk_free` and `zk_last_trap` of the runtime, under the names the runtime
/// links them with.
///
/// # Arguments
///
/// * `exports` - The functions.
/// * `names` - The names of the lib.
///
/// # Returns
///
/// The names.
pub fn api_names(exports: &[Export<'_>], names: &LibNames) -> Vec<String>
{
    let each = exports.iter().map(|e| e.c_name.to_string());
    let runtime = ["zk_free", "zk_free_all", "zk_last_trap"].map(crate::linked_name);
    let lib = [names.reset(), names.last_trap(), names.free(), names.free_all()];
    each.chain(lib).chain(runtime).collect()
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
/// kinds, and each struct a wrapper takes or gives, those of its fields first.
struct CKinds
{
    /// Bit count of the platform-sized kinds.
    platform: u16,
    /// The structs, each after the structs of its fields.
    structs: Vec<DataKind>,
    /// The cell kind of each plain list, which C takes as `{ ptr, count }`.
    lists: Vec<DataKind>,
    /// Each struct and each plain list, as its declaration comes: each after
    /// the structs and the lists it holds.
    order: Vec<DataKind>,
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
        let mut kinds = CKinds {
            platform,
            structs: Vec::new(),
            lists: Vec::new(),
            order: Vec::new(),
            c_names,
        };
        let each = exports
            .iter()
            .flat_map(|e| param_kinds(e.func).into_iter().chain(e.func.ret));
        let each: Vec<DataKind> = each.collect();
        each.into_iter().for_each(|k| kinds.note(k));
        kinds
    }

    /// Note the struct or the plain list a kind is, after the structs and the
    /// lists of its fields; another kind needs no declaration.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    fn note(&mut self, kind: DataKind)
    {
        let cell = cells(kind).0;
        ret_if!(let Some(s) = cell.as_seq(), self.note_list(cell, s.cell));
        let plain = cell.is_struct() && cell.fallible_parts().is_none();
        ret_if!(!plain || self.structs.contains(&cell));
        let fields = cell.as_struct().map(|s| s.fields().to_vec()).unwrap_or_default();
        fields.into_iter().for_each(|(_, k)| self.note(k));
        self.structs.push(cell);
        self.order.push(cell);
    }

    /// Note a plain list, after the struct or the list its cells are.
    ///
    /// # Arguments
    ///
    /// * `list` - The list kind.
    /// * `cell` - The kind of its cells.
    fn note_list(&mut self, list: DataKind, cell: DataKind)
    {
        ret_if!(self.lists.contains(&cell));
        self.note(cell);
        self.lists.push(cell);
        self.order.push(list);
    }

    /// The C kind of a value a wrapper takes: `const char *` for text,
    /// else as [`Self::c_given`] gives.
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
        tern!(text, "const char *".into(), self.c_given(kind))
    }

    /// The C kind of a value a wrapper gives, or of a field: `char *` for
    /// text the caller frees, `zk_list_<cell>` for a plain list, `void *` for
    /// an address or a handle, the name of a struct, else the C scalar.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// The C kind.
    fn c_given(&self, kind: DataKind) -> String
    {
        let crossed = crossing(kind, self.platform);
        ret_if!(kind.is_text() || crossed == Some(Crossing::OptText), "char *".into());
        ret_if!(let Some(s) = kind.as_seq(), self.list_name(s.cell));
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
        let decls: String = self.order.iter().map(|k| self.decl(*k)).collect();
        let structs = !self.structs.is_empty();
        tern!(structs, format!("{ASSERT_MACRO}{decls}"), decls)
    }

    /// The declaration of one struct, with a check of its size, or of one plain
    /// list.
    ///
    /// # Arguments
    ///
    /// * `kind` - The struct or the list kind.
    ///
    /// # Returns
    ///
    /// The C text.
    fn decl(&self, kind: DataKind) -> String
    {
        ret_if!(let Some(s) = kind.as_seq(), self.list_decl(s.cell));
        let Some((at, s)) =
            self.structs.iter().position(|k| *k == kind).zip(kind.as_struct())
        else
        {
            return String::new();
        };
        let mut out = format!("\n/* {} */\ntypedef struct\n{{\n", comment(&s.name));
        let fields = s.fields().iter().enumerate();
        fields.for_each(|(i, (name, k))| out.push_str(&self.field(i, name, *k)));
        let name = self.struct_name(at);
        let size = self.c_form(kind).0;
        let _ = writeln!(
            out,
            "}} {name};\nZK_LIB_ASSERT(sizeof({name}) == {size}, \"the C size of \
             {name} differs from its Zaka size\");"
        );
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
        format!("    {};\n", declared(&self.c_given(cell), &named))
    }

    /// The C name of the form of a plain list of a cell kind.
    ///
    /// # Arguments
    ///
    /// * `cell` - The cell kind.
    ///
    /// # Returns
    ///
    /// The name, such as `zk_list_int32_t`.
    fn list_name(&self, cell: DataKind) -> String
    {
        format!("zk_list_{}", self.cell_part(cell))
    }

    /// The part of the name of a list that names its cells: `text`, the name
    /// of a struct, `list_<cells>` for a list, else the C scalar.
    ///
    /// # Arguments
    ///
    /// * `cell` - The cell kind.
    ///
    /// # Returns
    ///
    /// The part.
    fn cell_part(&self, cell: DataKind) -> String
    {
        ret_if!(cell.is_text(), "text".into());
        ret_if!(let Some(s) = cell.as_seq(), format!("list_{}", self.cell_part(s.cell)));
        self.c_given(cell).replace(' ', "_")
    }
    /// The C form of each plain list: a pointer to its cells and their count,
    /// each declared once, as two libs can declare the same.
    ///
    /// # Returns
    ///
    /// The C text.
    fn list_decl(&self, cell: DataKind) -> String
    {
        let name = self.list_name(cell);
        let guard = name.to_uppercase();
        let c = self.c_given(cell);
        let ptr = declared(&c, "*ptr");
        format!(
            "\n#ifndef {guard}\n#define {guard}\n/* A plain list of {c}: its cells, \
             which the caller frees with zk_free(list.ptr), or with \
             zk_free_all(list.ptr) with each block the cells point at, and their \
             count. */\ntypedef struct\n{{\n    {ptr};\n    size_t count;\n}} \
             {name};\n#endif\n"
        )
    }

    /// The size of the C form of a kind, as the runtime writes it and C reads
    /// it.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// The size and the alignment, in octets.
    fn c_form(&self, kind: DataKind) -> (u32, u32)
    {
        let word = u32::from(self.platform / 8);
        ret_if!(kind.is_text(), (word, word));
        ret_if!(kind.as_seq().is_some(), (2 * word, word));
        if let Some(l) = kind.as_list()
        {
            let (size, align) = self.c_form(l.cell);
            return (size * l.len, align);
        }
        let Some(s) = kind.as_struct()
        else
        {
            return (kind.size(self.platform), kind.align(self.platform));
        };
        let (mut end, mut widest) = (0u32, 1u32);
        for (_, k) in s.fields()
        {
            let (size, align) = self.c_form(*k);
            (end, widest) = (end.next_multiple_of(align) + size, widest.max(align));
        }
        (end.next_multiple_of(widest), widest)
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
    let out = func.ret.map(|r| declared(&kinds.c_given(r), "*zk_out"));
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

/// The check of a size a C file and a C++ file both read.
const ASSERT_MACRO: &str = "
#ifndef ZK_LIB_ASSERT
#ifdef __cplusplus
#define ZK_LIB_ASSERT(c, m) static_assert(c, m)
#else
#define ZK_LIB_ASSERT(c, m) _Static_assert(c, m)
#endif
#endif
";

/// The glue of a lib, which the wrappers share: `runtime/zaka_lib_glue.h`.
const GLUE: &str = "\n#include \"zaka_lib_glue.h\"\n";

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
    let mut out = GLUE.to_string();
    out.push_str(&kinds.struct_decls());
    let records = exports.iter().map(|e| record_name(e.index));
    records.for_each(|r| _ = writeln!(out, "extern const zk_fnrec {r};"));
    let each = exports.iter().zip(shapes);
    each.for_each(|(e, shape)| out.push_str(&wrapper(e, *shape, &kinds)));
    out
}

/// The lines of a wrapper that holds the lock of its lib: the lock and the
/// entry, the check for a stopped lib, the end of a trap, the start of the
/// globals and the end.
const LOCKED_PARTS: [&str; 5] = [
    "    zk_lib_place zk_was = zk_lib_in();\n",
    "    RET_IF(\n        zk_lib_state_of(zk_lib_self) == ZK_LIB_STOPPED,\n        \
     zk_lib_end(zk_outer, zk_was, ZK_STOPPED)\n    );\n",
    "zk_lib_stop",
    "    zk_lib_begin();\n",
    "zk_lib_end",
];

/// The same lines of a wrapper whose calls run with no lock: a trap fails the
/// call alone.
const FREE_PARTS: [&str; 5] = [
    "    RET_IF(zk_lib_free_begin() == ZK_LIB_STOPPED, ZK_STOPPED);\n    zk_lib_place \
     zk_was = zk_lib_enter(zk_lib_self);\n",
    "",
    "zk_lib_fail",
    "",
    "zk_lib_free_end",
];

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
    // A struct result, or a result in its C form, comes whole into a local
    // of its C kind; each other result comes as a word of the union. The
    // runtime writes no C form for a caller with no out pointer.
    let c_form = e.func.ret.filter(|r| in_c_form(*r, kinds.platform));
    let whole = e.func.ret.filter(|r| kinds.structs.contains(r) || c_form.is_some());
    let held = whole.map_or_else(|| "zk_lib_result".into(), |r| kinds.c_given(r));
    let into =
        tern!(c_form.is_some(), "zk_out != NULL ? (void *)&zk_r : NULL", "&zk_r");
    let out = e.func.ret.map_or_else(String::new, |r| {
        let (c, m) = (kinds.c_side(r), result_member(r));
        let value = tern!(whole.is_some(), "zk_r".into(), format!("({c})zk_r.{m}"));
        format!("    DO_IF(zk_out != NULL, *zk_out = {value});\n")
    });
    let parts = tern!(e.free, &FREE_PARTS, &LOCKED_PARTS);
    let &[enter, stopped, stop, begin, end] = parts;
    format!(
        "\n/* {} */\nZK_LIB_EXPORT int32_t {}({})\n{{\n    void *zk_args[] = \
         {{{args}}};\n    {held} zk_r;\n    jmp_buf zk_here;\n{enter}    jmp_buf \
         *zk_outer = (jmp_buf *)zk_lib_jump();\n    uint32_t zk_depth_was = \
         zk_ctx()->depth;\n{stopped}    if (setjmp(zk_here) != 0)\n        return \
         {stop}(zk_outer, zk_was, zk_depth_was);\n    ZK_LIB_NO_UNWIND(zk_here);\n    \
         zk_lib_jump_set(&zk_here);\n{begin}    zk_cbb_{shape}({into}, zk_args, \
         (uint8_t *)&{});\n    zk_flush();\n{out}    return {end}(zk_outer, zk_was, \
         ZK_OK);\n}}\n",
        comment(&e.func.name),
        e.c_name,
        c_params(e.func, kinds),
        record_name(e.index),
    )
}

/// The header of a lib: the status words, the ids of the traps, the C form of
/// a trap, the functions of the runtime and of the lib for traps, frees and
/// resets, and the declaration of each wrapper.
///
/// # Arguments
///
/// * `exports` - The functions that code outside can call.
/// * `platform` - Bit count of the platform-sized kinds.
/// * `names` - The names of the lib.
/// * `links` - What a program that links the static lib links with it.
///
/// # Returns
///
/// The C text.
pub fn header(
    exports: &[Export<'_>], platform: u16, names: &LibNames, links: &[String],
) -> String
{
    let name = &names.name;
    let guard = name.to_ascii_uppercase();
    let links = tern!(links.is_empty(), String::new(), format!(" {}", links.join(" ")));
    let mut out = format!(
        "/* The functions of the Zaka lib {name} that C can call, as zakac wrote \
         them.\n * A program that links the static lib links with it:{links} \
         */\n#ifndef ZAKA_LIB_{guard}_H\n#define ZAKA_LIB_{guard}_H\n\n#include \
         <stddef.h>\n#include <stdint.h>\n\n#ifdef __cplusplus\nextern \"C\" \
         {{\n#endif\n{}\n/**\n * The trap the lib {name} stopped on last.\n *\n * \
         @return The trap, or NULL while it stopped on none.\n */\nconst zk_trap \
         *{}(void);\n\n/**\n * Start the lib {name} again after a trap stopped it: \
         the next call sets its globals again.\n */\nvoid {}(void);\n\n/**\n * Free \
         text or the cells of a list the lib {name} gave back, with the C runtime the \
         lib was built with; on Windows, where a lib of GNU and a lib of MSVC each \
         have their own, the one to use.\n *\n * @param p The `char *` or the `ptr` \
         of a list; NULL frees nothing.\n */\nvoid {}(void *p);\n\n/**\n * As the \
         free of the lib {name}, with each block the cells point at.\n *\n * @param p \
         The `char *` or the `ptr` of a list; NULL frees nothing.\n */\nvoid {}(void \
         *p);\n",
        shared_part(),
        names.last_trap(),
        names.reset(),
        names.free(),
        names.free_all()
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

/// The part of the header of a lib that each lib shares: the status words,
/// the ids of the traps, the C form of a trap, and `zk_free` and
/// `zk_last_trap` under the names the runtime links them with. The first
/// header of a lib that a C file includes writes it; each lib from the same
/// `zakac` links with the same runtime.
///
/// # Returns
///
/// The C text.
fn shared_part() -> String
{
    let ids: String = trap_names()
        .iter()
        .enumerate()
        .map(|(n, id)| format!("#define {id} {n}\n"))
        .collect();
    let (free, last) =
        (crate::linked_name("zk_free"), crate::linked_name("zk_last_trap"));
    let free_all = crate::linked_name("zk_free_all");
    format!(
        "\n#ifndef ZAKA_LIB_SHARED_H\n#define ZAKA_LIB_SHARED_H\n\n/* The runtime \
         links each of its names with its version and hash. */\n#define zk_free \
         {free}\n#define zk_free_all {free_all}\n#define zk_last_trap {last}\n\n/** \
         The status of a call that went through. A trap gives 1 + the id of the trap. \
         */\n#define ZK_OK 0\n/** The status of each call after a call stopped on a \
         trap. */\n#define ZK_STOPPED (-1)\n\n/* The id of each trap. */\n{ids}\n/** \
         One call in progress when a trap stopped a lib. */\ntypedef struct \
         zk_trap_frame\n{{\n    /** The Zaka name of the function, UTF-8, or NULL \
         when the build keeps no calls. */\n    const char *function;\n    /** The \
         source file, or NULL when the build keeps no source lines. */\n    const \
         char *file;\n    /** The line, from 1; 0 with no source line. */\n    \
         uint32_t line;\n    /** The column, from 1; 0 with no source line. */\n    \
         uint32_t column;\n}} zk_trap_frame;\n\n/** A trap a lib stopped on: valid \
         until the next trap of the same lib or its reset. */\ntypedef struct \
         zk_trap\n{{\n    /** The id of the trap, a ZK_TRAP_*; the status of the call \
         is the id + 1. */\n    int32_t code;\n    /** The short name of the trap, \
         such as \"overflow\". */\n    const char *kind;\n    /** The trap line, with \
         its place. */\n    const char *message;\n    /** The name of the lib. */\n    \
         const char *lib;\n    /** The call the trap stopped in. */\n    \
         zk_trap_frame where;\n    /** The calls in progress, the innermost first. \
         */\n    const zk_trap_frame *frames;\n    /** The count of frames: 0 when \
         the build keeps no calls. */\n    size_t frame_count;\n}} zk_trap;\n\n/**\n \
         * The trap a Zaka lib stopped on last: each lib linked with this runtime \
         counts.\n *\n * @return The trap, or NULL while no lib stopped on one.\n \
         */\nconst zk_trap *zk_last_trap(void);\n\n/**\n * Free text or the cells of \
         a list a lib gave back.\n *\n * @param p The `char *` or the `ptr` of a \
         list; NULL frees nothing.\n */\nvoid zk_free(void *p);\n\n/**\n * Free text \
         or the cells of a list a lib gave back, with each block the cells point at: \
         the text and the lists in them, at each depth.\n *\n * @param p The `char *` \
         or the `ptr` of a list; NULL frees nothing.\n */\nvoid zk_free_all(void \
         *p);\n\n#endif\n"
    )
}
