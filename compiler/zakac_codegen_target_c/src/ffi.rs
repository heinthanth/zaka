//! The C functions a program declares, `"<lib>" မှလာသော`. Each call shape (a C
//! function, and the kinds of the other values a call gives one that takes
//! them) gets a wrapper, `zk_ffi_<n>`, in a C file of its own, `zaka_ffi.c`,
//! which includes no libc header: it declares each C function with the kinds
//! Zaka gives it, so a declaration there cannot clash with the one a libc
//! header holds. The modules call the wrappers, in the kinds the generated C
//! holds: a struct goes in and out as the address of its octets, and text
//! goes in as the address of its words, which the wrapper turns into a
//! duplicate that ends in a zero octet.

use std::fmt::Write;
use std::path::Path;

use bon::Builder;
use zakac_ir_builder::ir::ForeignFn;
use zakac_ir_builder::ir::fn_shape;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::fn_kind::FnKind;
use zakac_semcheck::foreign::is_c_text_result;
use zakac_semcheck::foreign::is_lib_path;
use zakac_semcheck::kind::DataKind;

use crate::kinds::c_kind;
use crate::kinds::is_blob;

/// The C file of the wrappers.
pub const FFI_SOURCE: &str = "zaka_ffi.c";

/// The libs each link takes, which a program names with no flag.
const LINKED: &[&str] = &["libc", "libm"];

/// The start of `zaka_ffi.c`. A declaration that differs from the one the
/// compiler knows for a libc function, such as `void *` for `int *`, is the
/// same to the ABI, so the warning about it is turned off.
const PRELUDE: &str = "\
/* The C functions of the program. This file includes no libc header, so a
   declaration below cannot clash with the one a header holds. */
#include <stddef.h>
#include <stdint.h>

#if defined(__clang__)
#pragma clang diagnostic ignored \"-Wincompatible-library-redeclaration\"
#elif defined(__GNUC__)
#pragma GCC diagnostic ignored \"-Wbuiltin-declaration-mismatch\"
#endif

/** A duplicate of text that ends in a zero octet, from the runtime. */
char *zk_text_cstr(const unsigned char *at);
/** Free a duplicate zk_text_cstr made, from the runtime. */
void zk_text_cstr_free(char *dup);
/** The text result of a C function, or the failure for NULL, from the runtime. */
void zk_text_from_c(unsigned char *out, size_t size, size_t ok_at, const char *s);
/** libc's free, for a string C hands over. */
void free(void *p);
/** The code of a function, cast to one kind, from the runtime. */
typedef void (*zk_code)(void);
/** A function value, the address of its record, from the runtime. */
typedef const void *zk_fn;
/** The code a function value runs, from the runtime. */
zk_code zk_fn_code(zk_fn f);
/** The run stops when a function value is a closure, from the runtime. */
zk_fn zk_fn_c(zk_fn f);

/** Write `size` octets from `from` to `to`; the two do not overlap. */
static void zk_ffi_put(void *to, const void *from, size_t size)
{
    unsigned char *t = to;
    const unsigned char *f = from;
    for (size_t i = 0; i < size; i++)
    {
        t[i] = f[i];
    }
}
";

/// A call shape of a C function: which one, and the kind of each other value
/// a call gives, for a C function that takes them.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct FfiCall
{
    /// Index into the C functions of the module.
    pub func: u32,
    /// The kind of each other value, in order, as C promotes it.
    #[builder(default)]
    pub extra: Vec<DataKind>,
}

/// The name of the wrapper of a call shape.
///
/// # Arguments
///
/// * `n` - Index into the call shapes of the module.
///
/// # Returns
///
/// The name, `zk_ffi_<n>`.
pub fn wrapper_name(n: u32) -> String { format!("zk_ffi_{n}") }

/// The declaration of the wrapper of a call shape, in the kinds the generated
/// C holds: a struct or text parameter is the address of its octets, and a
/// struct result goes to the address in the first parameter, `zk_out`.
///
/// # Arguments
///
/// * `def` - The C function.
/// * `call` - The call shape.
/// * `n` - Its index.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The declaration, with no semicolon.
pub fn wrapper_proto(def: &ForeignFn, call: &FfiCall, n: u32, platform: u16) -> String
{
    let held =
        |k: DataKind| c_kind(k, platform).unwrap_or_else(|| "unsigned char *".into());
    let out =
        def.ret.filter(|k| is_blob(*k)).map(|_| "unsigned char *zk_out".to_string());
    let each = def.params.iter().chain(&call.extra).enumerate();
    let params: Vec<String> = out
        .into_iter()
        .chain(each.map(|(i, k)| format!("{} zk_a{i}", held(*k))))
        .collect();
    let params = tern!(params.is_empty(), "void".to_string(), params.join(", "));
    let ret = def.ret.filter(|k| !is_blob(*k)).map_or_else(|| "void".into(), held);
    format!("{ret} {}({params})", wrapper_name(n))
}

/// The link flags of each lib the C functions come from, once each: `-lX`
/// for `libX`, or `-l<name>` for a name with no `lib` in front; libc and libm
/// need none. A path goes to the linker as it is, with its folder as a run
/// path on each host but Windows, so the program finds the lib when it runs.
///
/// # Arguments
///
/// * `fns` - The C functions of the module.
///
/// # Returns
///
/// The flags, in the order the libs first appear.
pub fn link_libs(fns: &[ForeignFn]) -> Vec<String>
{
    let mut flags: Vec<String> = Vec::new();
    let names = fns.iter().map(|f| f.lib.as_str()).filter(|l| !LINKED.contains(l));
    let each = names.flat_map(lib_flags);
    each.for_each(|f| tern!(flags.contains(&f), (), flags.push(f)));
    flags
}

/// The link flags of one lib.
///
/// # Arguments
///
/// * `lib` - The lib, as the program writes it: a name or a path.
///
/// # Returns
///
/// The flags.
fn lib_flags(lib: &str) -> Vec<String>
{
    ret_if!(
        !is_lib_path(lib),
        vec![format!("-l{}", lib.strip_prefix("lib").unwrap_or(lib))]
    );
    let folder = Path::new(lib).parent().map(|f| f.to_string_lossy().into_owned());
    let run_path = folder.filter(|f| !f.is_empty() && !cfg!(windows));
    let run_path = run_path.map(|f| format!("-Wl,-rpath,{f}"));
    std::iter::once(lib.to_string()).chain(run_path).collect()
}

/// The text of `zaka_ffi.c`: the structs the C functions pass, each C
/// function as Zaka declares it, and the wrapper of each call shape.
///
/// # Arguments
///
/// * `fns` - The C functions of the module.
/// * `calls` - The call shapes.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The C text.
pub fn ffi_source(fns: &[ForeignFn], calls: &[FfiCall], platform: u16) -> String
{
    let mut structs = Vec::new();
    // A text result is a string to C, no struct.
    let ret = |f: &ForeignFn| f.ret.into_iter().filter(|k| !is_c_text_result(*k));
    let kinds = fns.iter().flat_map(|f| f.params.iter().copied().chain(ret(f)));
    // A struct given as an other value alone needs its definition too.
    let extra = calls.iter().flat_map(|c| c.extra.iter().copied());
    kinds.chain(extra).for_each(|k| collect_structs(k, &mut structs));
    let names = Names { structs: &structs, platform };
    let mut out = PRELUDE.to_string();
    structs
        .iter()
        .enumerate()
        .for_each(|(i, s)| out.push_str(&names.struct_def(*s, i)));
    fns.iter().for_each(|f| out.push_str(&names.declaration(f)));
    let each = calls.iter().enumerate();
    each.for_each(|(n, c)| {
        out.push_str(&names.wrapper(&fns[c.func as usize], c, n as u32))
    });
    out
}

/// Add each struct a kind holds to a list, the structs of its fields first,
/// so each C struct is defined before a field names it.
///
/// # Arguments
///
/// * `kind` - The kind.
/// * `out` - The structs so far.
fn collect_structs(kind: DataKind, out: &mut Vec<DataKind>)
{
    let kind = cell_of(kind);
    let Some(s) = kind.as_struct()
    else
    {
        return;
    };
    ret_if!(out.contains(&kind));
    s.fields().iter().for_each(|(_, k)| collect_structs(*k, out));
    out.push(kind);
}

/// The kind of the cells of a counted list, through each level of lists; a
/// kind that is no list comes back as it is.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// The cell kind.
fn cell_of(kind: DataKind) -> DataKind
{
    kind.as_list().map_or(kind, |l| cell_of(l.cell))
}

/// The names C gives the kinds of the C functions.
struct Names<'a>
{
    /// The structs the C functions pass, in the order of their definitions.
    structs: &'a [DataKind],
    /// Bit count of the platform-sized kinds.
    platform: u16,
}

impl Names<'_>
{
    /// The C kind of a kind as the C function sees it: `_Bool`, `void *`,
    /// a struct of this file, or the C kind the generated C holds.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind, no list.
    ///
    /// # Returns
    ///
    /// The C kind.
    fn real(&self, kind: DataKind) -> String
    {
        ret_if!(let Some(f) = kind.as_fn(), self.declarator(f, ""));
        match kind
        {
            DataKind::Bool => "_Bool".into(),
            DataKind::Ptr(_) => "void *".into(),
            _ if kind.is_textual() || is_c_text_result(kind) => "const char *".into(),
            DataKind::Struct(_) =>
            {
                let at = self.structs.iter().position(|s| *s == kind).unwrap_or(0);
                format!("zk_ffi_s{at}")
            }
            _ => c_kind(kind, self.platform).unwrap_or_else(|| "void".into()),
        }
    }

    /// The C kind of a function C calls back, with a name in it for a
    /// declaration: `int32_t (*name)(void *, void *)`.
    ///
    /// # Arguments
    ///
    /// * `f` - The function kind.
    /// * `name` - The name; blank for the kind alone.
    ///
    /// # Returns
    ///
    /// The C text.
    fn declarator(&self, f: &FnKind, name: &str) -> String
    {
        let (params, ret) = fn_shape(f);
        let ret = ret.map_or_else(|| "void".into(), |k| self.real(k));
        let params: Vec<String> = params.iter().map(|k| self.real(*k)).collect();
        let params = tern!(params.is_empty(), "void".to_string(), params.join(", "));
        format!("{ret} (*{name})({params})")
    }

    /// The declaration of a field: a counted list becomes a run of C cells,
    /// with a count for each level.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of the field.
    /// * `name` - Its C name.
    ///
    /// # Returns
    ///
    /// The declaration, with no semicolon.
    fn field(&self, kind: DataKind, name: &str) -> String
    {
        let mut counts = String::new();
        let mut cell = kind;
        while let Some(l) = cell.as_list()
        {
            let _ = write!(counts, "[{}]", l.len);
            cell = l.cell;
        }
        format!("{} {name}{counts}", self.real(cell))
    }

    /// The definition of a struct, with a check that C gives it the size
    /// Zaka gives it.
    ///
    /// # Arguments
    ///
    /// * `kind` - The struct kind.
    /// * `at` - Its index among the structs of this file.
    ///
    /// # Returns
    ///
    /// The C text.
    fn struct_def(&self, kind: DataKind, at: usize) -> String
    {
        let s = kind.as_struct().expect("the list holds structs");
        let each = s.fields().iter().enumerate();
        let fields: String = each
            .map(|(i, (_, k))| format!("    {};\n", self.field(*k, &format!("f{i}"))))
            .collect();
        let size = kind.size(self.platform);
        format!(
            "\n/* {} */\ntypedef struct\n{{\n{fields}}} \
             zk_ffi_s{at};\n_Static_assert(sizeof(zk_ffi_s{at}) == {size}, \"the C \
             size of a struct differs from its Zaka size\");\n",
            s.name.replace("*/", "* /")
        )
    }

    /// The declaration of a C function, with the kinds Zaka gives it, and
    /// `...` when it takes other values.
    ///
    /// # Arguments
    ///
    /// * `def` - The C function.
    ///
    /// # Returns
    ///
    /// The C text.
    fn declaration(&self, def: &ForeignFn) -> String
    {
        let ret = def.ret.map_or_else(|| "void".into(), |k| self.real(k));
        let mut kinds: Vec<String> = def.params.iter().map(|k| self.real(*k)).collect();
        do_if!(def.variadic, kinds.push("...".into()));
        let kinds = tern!(kinds.is_empty(), "void".to_string(), kinds.join(", "));
        format!("\n/* {} */\nextern {ret} {}({kinds});\n", def.lib, def.name)
    }

    /// The wrapper of a call shape: it reads each struct from its octets and
    /// makes a duplicate of each text that ends in a zero octet, calls the C
    /// function, frees the duplicates, and gives back the result.
    ///
    /// # Arguments
    ///
    /// * `def` - The C function.
    /// * `call` - The call shape.
    /// * `n` - Its index.
    ///
    /// # Returns
    ///
    /// The C text.
    fn wrapper(&self, def: &ForeignFn, call: &FfiCall, n: u32) -> String
    {
        let mut out = format!("\n{}\n{{\n", wrapper_proto(def, call, n, self.platform));
        let kinds: Vec<DataKind> =
            def.params.iter().chain(&call.extra).copied().collect();
        let each = kinds.iter().enumerate();
        each.clone().for_each(|(i, k)| out.push_str(&self.unpack(i, *k)));
        let arg = |(i, k): (usize, &DataKind)| match k
        {
            k if k.is_textual() => format!("zk_s{i}"),
            k if is_blob(*k) || k.is_fn() => format!("zk_v{i}"),
            _ => format!("zk_a{i}"),
        };
        let args: Vec<String> = each.clone().map(arg).collect();
        let call_text = format!("{}({})", def.name, args.join(", "));
        let ret = def.ret.map(|k| self.real(k));
        let _ = match &ret
        {
            Some(r) => writeln!(out, "    {r} zk_r = {call_text};"),
            None => writeln!(out, "    {call_text};"),
        };
        // A string C gives back can point into a duplicate of a text
        // argument, so the text of the result is taken before those are freed.
        let taken = def.ret.filter(|k| is_c_text_result(*k)).map(|k| {
            let ok_at = k.as_struct().map_or(0, |s| s.offset(1, self.platform));
            let size = k.size(self.platform);
            let line = format!("    zk_text_from_c(zk_out, {size}, {ok_at}, zk_r);\n");
            // A string C hands over is freed once its octets are copied.
            let free = tern!(def.owned_ret, "    free((void *)zk_r);\n", "");
            format!("{line}{free}")
        });
        out.push_str(taken.as_deref().unwrap_or_default());
        let texts = each.filter(|(_, k)| k.is_textual());
        texts.for_each(|(i, _)| _ = writeln!(out, "    zk_text_cstr_free(zk_s{i});"));
        let tail = match def.ret
        {
            None => "",
            Some(_) if taken.is_some() => "",
            Some(k) if is_blob(k) => "    zk_ffi_put(zk_out, &zk_r, sizeof zk_r);\n",
            Some(_) => "    return zk_r;\n",
        };
        out.push_str(tail);
        out.push_str("}\n");
        out
    }

    /// The lines that turn a parameter of a wrapper into what C takes: a
    /// copy of text that ends in a zero octet, or a struct read from its
    /// octets; nothing for each other kind.
    ///
    /// # Arguments
    ///
    /// * `i` - The index of the parameter.
    /// * `kind` - Its kind.
    ///
    /// # Returns
    ///
    /// The C lines.
    fn unpack(&self, i: usize, kind: DataKind) -> String
    {
        ret_if!(
            kind.is_textual(),
            format!("    char *zk_s{i} = zk_text_cstr(zk_a{i});\n")
        );
        if let Some(f) = kind.as_fn()
        {
            // C calls the code of the function; a closure stops the run.
            let decl = self.declarator(f, &format!("zk_v{i}"));
            let cast = self.declarator(f, "");
            return format!(
                "    {decl} = zk_a{i} ? ({cast})zk_fn_code(zk_fn_c(zk_a{i})) : \
                 ({cast})0;\n"
            );
        }
        ret_if!(!is_blob(kind), String::new());
        let real = self.real(kind);
        format!(
            "    {real} zk_v{i};\n    zk_ffi_put(&zk_v{i}, zk_a{i}, sizeof zk_v{i});\n"
        )
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// A C function of a lib with these kinds.
    fn def(
        lib: &str, name: &str, params: Vec<DataKind>, ret: Option<DataKind>,
    ) -> ForeignFn
    {
        ForeignFn::builder().lib(lib).name(name).params(params).maybe_ret(ret).build()
    }

    #[test]
    fn the_ffi_file_includes_no_libc_header()
    {
        let abs = def("libc", "abs", vec![DataKind::ISIZE], Some(DataKind::ISIZE));
        let call = FfiCall::builder().func(0).build();
        let text = ffi_source(&[abs], &[call], 64);
        let includes: Vec<&str> =
            text.lines().filter(|l| l.starts_with("#include")).collect();
        assert_eq!(includes, ["#include <stddef.h>", "#include <stdint.h>"]);
        assert!(text.contains("extern int64_t abs(int64_t);"));
        assert!(text.contains("int64_t zk_r = abs(zk_a0);"));
    }

    #[test]
    fn libs_link_once_and_libc_needs_no_flag()
    {
        let fns = [
            def("libc", "abs", vec![], None),
            def("libsqlite3", "a", vec![], None),
            def("libsqlite3", "b", vec![], None),
            def("user32", "c", vec![], None),
        ];
        assert_eq!(link_libs(&fns), ["-lsqlite3", "-luser32"]);
        let path = [def("/opt/x/libfoo.so", "f", vec![], None)];
        let run = tern!(cfg!(windows), vec![], vec!["-Wl,-rpath,/opt/x".to_string()]);
        let want: Vec<String> =
            std::iter::once("/opt/x/libfoo.so".into()).chain(run).collect();
        assert_eq!(link_libs(&path), want);
    }
}
