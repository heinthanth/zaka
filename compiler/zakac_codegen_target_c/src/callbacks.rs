//! The closures C calls back in a native build: `zaka_cb.c`, the runtime of
//! `zaka_rt_cb.c` with a bridge and the libffi kinds of each function kind
//! the program can hand C as a closure, or as a function whose parameters do
//! not cross as C passes them: text, a struct C passes as a value, a handle, or
//! an optional of text or of a handle. A program with no such kind gets no
//! file, and its link needs no libffi.

use std::fmt::Write;
use std::process::Command;
use std::sync::OnceLock;

use zakac_ir_builder::ir::fn_shape;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::fn_kind::FnKind;
use zakac_semcheck::foreign::is_c_handle_result;
use zakac_semcheck::foreign::is_c_text_result;
use zakac_semcheck::kind::DataKind;

use crate::descriptors::Descriptors;
use crate::ffi::collect_structs;
use crate::kinds::c_kind;
use crate::kinds::is_blob;

/// The name of the file of the closures C calls back.
pub const CB_SOURCE: &str = "zaka_cb.c";

/// The link flag of libffi, which `zaka_cb.c` calls.
pub const FFI_LIB: &str = "-lffi";

/// The flags of libffi, asked of `pkg-config` once: the compile flags of
/// `zaka_cb.c` and the link flags. macOS keeps libffi in the SDK, where the
/// compiler finds it, so `-lffi` alone links there; so it does where
/// `pkg-config` or its entry for libffi is missing.
///
/// # Returns
///
/// The compile flags and the link flags.
pub fn ffi_flags() -> &'static (Vec<String>, Vec<String>)
{
    static FLAGS: OnceLock<(Vec<String>, Vec<String>)> = OnceLock::new();
    FLAGS.get_or_init(|| {
        let ask = |what: &str| -> Option<Vec<String>> {
            ret_if!(cfg!(target_os = "macos"), None);
            let out =
                Command::new("pkg-config").args([what, "libffi"]).output().ok()?;
            let text = String::from_utf8_lossy(&out.stdout);
            out.status
                .success()
                .then(|| text.split_whitespace().map(str::to_string).collect())
        };
        let cflags = ask("--cflags").unwrap_or_default();
        let libs = ask("--libs").filter(|l| !l.is_empty());
        (cflags, libs.unwrap_or_else(|| vec![FFI_LIB.to_string()]))
    })
}

/// The runtime part of `zaka_cb.c`.
const RUNTIME_CB: &str = include_str!("../runtime/zaka_rt_cb.c");

/// The name of the shape of a function kind C calls back.
///
/// # Arguments
///
/// * `n` - Its index among the shapes of the program.
///
/// # Returns
///
/// The name, `zk_cbs_<n>`.
pub fn shape_name(n: usize) -> String { format!("zk_cbs_{n}") }

/// How a value C passes to a function of the program becomes a value of
/// Zaka in the bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Crossing
{
    /// A number, a character, a boolean or an address, as it is.
    AsIs,
    /// A plain struct C passes as a value, whose octets go into a block.
    Struct,
    /// A `char *`, whose octets go into new text; NULL stops the run.
    Text,
    /// A `char *`, as an optional of text; NULL holds no data.
    OptText,
    /// The address of a struct C keeps, in a new handle; NULL gives a handle
    /// that reads as closed.
    Handle,
    /// The address of a struct C keeps, as an optional of a new handle;
    /// NULL holds no data.
    OptHandle,
}

/// How a parameter kind crosses from C into a function of the program.
///
/// # Arguments
///
/// * `kind` - The parameter kind.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The crossing, or `None` for a kind the bridge cannot make from what C
/// passes.
fn crossing(kind: DataKind, platform: u16) -> Option<Crossing>
{
    ret_if!(kind.is_text(), Some(Crossing::Text));
    ret_if!(is_c_text_result(kind), Some(Crossing::OptText));
    ret_if!(kind.as_external().is_some(), Some(Crossing::Handle));
    ret_if!(is_c_handle_result(kind), Some(Crossing::OptHandle));
    let plain = is_plain_struct(kind);
    ret_if!(plain, struct_fits(kind, platform).then_some(Crossing::Struct));
    ffi_kind(kind, platform).map(|_| Crossing::AsIs)
}

/// Whether a kind is a struct of the program: no result and no optional,
/// which are structs of Zaka alone.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// `true` for such a struct.
fn is_plain_struct(kind: DataKind) -> bool
{
    kind.is_struct() && kind.fallible_parts().is_none()
}

/// Whether libffi can pass a plain struct as a value: each field, and each
/// cell of a counted list, is a kind libffi passes as it is, or such a
/// struct.
///
/// # Arguments
///
/// * `kind` - The kind.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// `true` when it can.
fn struct_fits(kind: DataKind, platform: u16) -> bool
{
    let Some(s) = kind.as_struct().filter(|_| is_plain_struct(kind))
    else
    {
        return false;
    };
    s.fields().iter().all(|(_, k)| {
        let cell = cells(*k).0;
        ffi_kind(cell, platform).is_some() || struct_fits(cell, platform)
    })
}

/// The cell kind of a counted list, through each level of lists, and the
/// count of such cells; a kind that is no list is one cell of itself.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// The cell kind and the count.
fn cells(kind: DataKind) -> (DataKind, usize)
{
    let Some(l) = kind.as_list()
    else
    {
        return (kind, 1);
    };
    let (cell, count) = cells(l.cell);
    (cell, count * l.len as usize)
}

/// The kind of the cells of the box of a handle, as a list of one cell: for
/// a handle, or for an optional of a handle.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// The list kind, or `None` for a kind that holds no handle.
fn box_list(kind: DataKind) -> Option<DataKind>
{
    let data = kind.fallible_parts().and_then(|(some, _)| some);
    let handle = kind.as_external().or_else(|| data?.as_external())?;
    Some(DataKind::list_of(DataKind::Struct(handle), 1))
}

/// Whether a closure of a function kind can go to C: the result is a kind
/// libffi passes as it is, and each parameter a kind the bridge can make
/// from what C passes.
///
/// # Arguments
///
/// * `f` - The function kind.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// `true` when it can.
pub fn fits(f: &FnKind, platform: u16) -> bool
{
    let (params, ret) = fn_shape(f);
    let ret_fits = ret.is_none_or(|k| ffi_kind(k, platform).is_some());
    ret_fits && params.iter().all(|k| crossing(*k, platform).is_some())
}

/// Whether C can call the code of a function of a function kind with no
/// bridge: each parameter crosses as it is. A closure takes the bridge in
/// each case, which hands the thunk its record.
///
/// # Arguments
///
/// * `f` - The function kind.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// `true` when it can.
pub fn as_is(f: &FnKind, platform: u16) -> bool
{
    let (params, _) = fn_shape(f);
    params.iter().all(|k| crossing(*k, platform) == Some(Crossing::AsIs))
}

/// The kinds whose descriptors the bridge of a function kind names: each
/// parameter the bridge makes, to lend it and drop it, and the cells of the
/// box of each handle, to make it.
///
/// # Arguments
///
/// * `f` - The function kind.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The kinds.
pub(crate) fn bridge_kinds(f: &FnKind, platform: u16) -> Vec<DataKind>
{
    let (params, _) = fn_shape(f);
    let made = |k: &DataKind| {
        let c = crossing(*k, platform);
        !matches!(c, None | Some(Crossing::AsIs | Crossing::Struct))
    };
    let each = params.into_iter().filter(made);
    each.flat_map(|k| std::iter::once(k).chain(box_list(k))).collect()
}

/// The libffi kind of a kind a closure takes or gives.
///
/// # Arguments
///
/// * `kind` - The kind.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The name of the `ffi_type`, or `None` for a kind libffi cannot pass as
/// it is.
fn ffi_kind(kind: DataKind, platform: u16) -> Option<String>
{
    let repr = kind.repr();
    match repr
    {
        DataKind::Int(k) =>
        {
            let bits = repr.bits(platform);
            ret_if!(bits > 64, None);
            Some(format!("ffi_type_{}int{bits}", tern!(k.signed, "s", "u")))
        }
        DataKind::Float(32) => Some("ffi_type_float".into()),
        DataKind::Float(64) => Some("ffi_type_double".into()),
        DataKind::Char => Some("ffi_type_uint32".into()),
        DataKind::Bool => Some("ffi_type_uint8".into()),
        DataKind::Ptr(_) | DataKind::Null => Some("ffi_type_pointer".into()),
        _ => None,
    }
}

/// The line that writes the result of a thunk where libffi wants it: a whole
/// `ffi_arg` for an integer narrower than one, as libffi asks.
///
/// # Arguments
///
/// * `kind` - The result kind.
///
/// # Returns
///
/// The C line.
fn result_line(kind: DataKind) -> String
{
    match kind.repr()
    {
        DataKind::Int(k) if k.signed => "*(ffi_sarg *)zk_ret = (ffi_sarg)zk_r;".into(),
        DataKind::Int(_) | DataKind::Char | DataKind::Bool =>
        {
            "*(ffi_arg *)zk_ret = (ffi_arg)zk_r;".into()
        }
        DataKind::Float(32) => "*(float *)zk_ret = zk_r;".into(),
        DataKind::Float(_) => "*(double *)zk_ret = zk_r;".into(),
        _ => "*(void **)zk_ret = zk_r;".into(),
    }
}

/// The text of `zaka_cb.c`: the runtime, the descriptors the bridges name,
/// the libffi kind of each struct C passes as a value, then the bridge and
/// the shape of each function kind.
///
/// # Arguments
///
/// * `kinds` - The function kinds, in the order of their shapes.
/// * `platform` - Bit count of the platform-sized kinds.
/// * `descr` - The descriptors, which hold those the bridges name since each
///   shape was noted.
///
/// # Returns
///
/// The C text.
pub(crate) fn cb_source(
    kinds: &[DataKind], platform: u16, descr: &mut Descriptors,
) -> String
{
    let each = kinds.iter().enumerate();
    let fns: Vec<(usize, DataKind, &FnKind)> =
        each.filter_map(|(n, k)| k.as_fn().map(|f| (n, *k, f))).collect();
    let params = fns.iter().flat_map(|(_, _, f)| fn_shape(f).0);
    let mut structs = Vec::new();
    let whole = |k: &DataKind| crossing(*k, platform) == Some(Crossing::Struct);
    params.filter(whole).for_each(|k| collect_structs(k, &mut structs));
    let named = fns.iter().flat_map(|(_, _, f)| bridge_kinds(f, platform));
    let mut names: Vec<String> = named.map(|k| descr.of(k)).collect();
    names.sort();
    names.dedup();
    let mut out = RUNTIME_CB.to_string();
    out.push_str("\n/* ---- the function kinds of the program ---- */\n");
    names.iter().for_each(|d| {
        _ = writeln!(out, "extern const zk_kind {};", d.trim_start_matches('&'))
    });
    let bridge = Bridge { platform, structs: &structs };
    let each = structs.iter().enumerate();
    each.for_each(|(i, s)| out.push_str(&bridge.struct_kind(*s, i)));
    fns.iter().for_each(|(n, k, f)| out.push_str(&bridge.shape(*n, *k, f, descr)));
    out
}

/// What the bridges of `zaka_cb.c` share.
struct Bridge<'a>
{
    /// Bit count of the platform-sized kinds.
    platform: u16,
    /// The structs C passes as a value, the structs of their fields first,
    /// in the order of their libffi kinds.
    structs: &'a [DataKind],
}

impl Bridge<'_>
{
    /// The name of the libffi kind of a struct C passes as a value.
    ///
    /// # Arguments
    ///
    /// * `kind` - The struct kind.
    ///
    /// # Returns
    ///
    /// The C text of its address, `&zk_cbst_<n>`.
    fn struct_ffi(&self, kind: DataKind) -> String
    {
        let at = self.structs.iter().position(|s| *s == kind).unwrap_or(0);
        format!("&zk_cbst_{at}")
    }

    /// The libffi kind of a struct C passes as a value: a struct of libffi
    /// with one member for each field, and one for each cell of a counted
    /// list, as libffi places a C list in a struct.
    ///
    /// # Arguments
    ///
    /// * `kind` - The struct kind.
    /// * `at` - Its index among the structs.
    ///
    /// # Returns
    ///
    /// The C text.
    fn struct_kind(&self, kind: DataKind, at: usize) -> String
    {
        let s = kind.as_struct().expect("the list holds structs");
        let member = |cell: DataKind| {
            let scalar = ffi_kind(cell, self.platform).map(|t| format!("&{t}"));
            scalar.unwrap_or_else(|| self.struct_ffi(cell))
        };
        let each = s.fields().iter().map(|(_, k)| cells(*k));
        let members = each.flat_map(|(cell, n)| std::iter::repeat_n(member(cell), n));
        let members: Vec<String> = members.chain(["NULL".to_string()]).collect();
        format!(
            "\n/* {} */\nstatic ffi_type *zk_cbse_{at}[] = {{{}}};\nstatic ffi_type \
             zk_cbst_{at} = {{0, 0, FFI_TYPE_STRUCT, zk_cbse_{at}}};\n",
            s.name.replace("*/", "* /"),
            members.join(", ")
        )
    }

    /// The libffi kind of a parameter: a struct C passes as a value, the
    /// kind of a value libffi passes as it is, or an address.
    ///
    /// # Arguments
    ///
    /// * `kind` - The parameter kind.
    ///
    /// # Returns
    ///
    /// The C text of its address.
    fn param_ffi(&self, kind: DataKind) -> String
    {
        match crossing(kind, self.platform)
        {
            Some(Crossing::Struct) => self.struct_ffi(kind),
            Some(Crossing::AsIs) | None =>
            {
                format!("&{}", ffi_kind(kind, self.platform).unwrap_or_default())
            }
            Some(_) => "&ffi_type_pointer".into(),
        }
    }

    /// The C kind the thunk takes or gives for a kind: the address of a
    /// block, or the C scalar.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// The C kind.
    fn held(&self, kind: DataKind) -> String
    {
        ret_if!(is_blob(kind), "unsigned char *".into());
        c_kind(kind, self.platform).unwrap_or_else(|| "int".into())
    }

    /// The bridge and the shape of one function kind. The bridge reads each
    /// argument libffi gives, makes a value of Zaka of each one that does
    /// not cross as it is, and calls the thunk of a closure with its record,
    /// or a function with none.
    ///
    /// # Arguments
    ///
    /// * `n` - The index of its shape.
    /// * `kind` - The kind, for a comment.
    /// * `f` - The function kind.
    /// * `descr` - The descriptors.
    ///
    /// # Returns
    ///
    /// The C text.
    fn shape(
        &self, n: usize, kind: DataKind, f: &FnKind, descr: &mut Descriptors,
    ) -> String
    {
        let (params, ret) = fn_shape(f);
        let r = ret.map_or_else(|| "void".into(), |k| self.held(k));
        let mut kinds: Vec<String> = params.iter().map(|k| self.held(*k)).collect();
        let plain = tern!(kinds.is_empty(), "void".to_string(), kinds.join(", "));
        kinds.push("unsigned char *".into());
        let (plain_cast, bound_cast) =
            (format!("{r} (*)({plain})"), format!("{r} (*)({})", kinds.join(", ")));
        let mut steps = String::new();
        let mut drops = String::new();
        do_unused(&mut steps, params.is_empty(), "zk_args");
        let each = params.iter().enumerate();
        let args: Vec<String> =
            each.map(|(i, k)| self.arg(i, *k, descr, &mut steps, &mut drops)).collect();
        let bound_args: Vec<String> =
            args.iter().cloned().chain(["zk_rec".to_string()]).collect();
        let call = format!(
            "zk_fn_bound(zk_rec) ? (({bound_cast})zk_c)({}) : (({plain_cast})zk_c)({})",
            bound_args.join(", "),
            args.join(", ")
        );
        steps.push_str("    zk_code zk_c = zk_fn_code(zk_rec);\n");
        let _ = match ret
        {
            None => write!(steps, "    (void)zk_ret;\n    {call};\n{drops}"),
            Some(k) =>
            {
                write!(steps, "    {r} zk_r = {call};\n{drops}    {}\n", result_line(k))
            }
        };
        let listed: Vec<String> = params.iter().map(|k| self.param_ffi(*k)).collect();
        let listed = tern!(listed.is_empty(), "NULL".to_string(), listed.join(", "));
        let ret_ffi = ret.map_or_else(
            || "ffi_type_void".into(),
            |k| ffi_kind(k, self.platform).unwrap_or_default(),
        );
        let name = kind.name().replace("*/", "* /");
        format!(
            "\n/* {name} */\nstatic void zk_cbb_{n}(void *zk_ret, void **zk_args, \
             unsigned char *zk_rec)\n{{\n{steps}}}\nstatic ffi_type *zk_cbt_{n}[] = \
             {{{listed}}};\nconst zk_cb_shape {} = {{zk_cbb_{n}, &{ret_ffi}, (void \
             *const *)zk_cbt_{n}, {}, {}}};\n",
            shape_name(n),
            params.len(),
            u8::from(as_is(f, self.platform))
        )
    }

    /// The lines that make the value of one argument the thunk takes. A
    /// struct goes into a block of the bridge; text and a handle are made
    /// new, and the function gets a lent duplicate, as each call lends, so
    /// its drop frees nothing; the bridge drops the value once the function
    /// returns, and a value the function kept holds its own count.
    ///
    /// # Arguments
    ///
    /// * `i` - The index of the parameter.
    /// * `kind` - Its kind.
    /// * `descr` - The descriptors.
    /// * `steps` - The steps before the call, added to.
    /// * `drops` - The steps after the call, added to.
    ///
    /// # Returns
    ///
    /// The C text of the argument.
    fn arg(
        &self, i: usize, kind: DataKind, descr: &mut Descriptors, steps: &mut String,
        drops: &mut String,
    ) -> String
    {
        let crossing = crossing(kind, self.platform).unwrap_or(Crossing::AsIs);
        ret_if!(
            crossing == Crossing::AsIs,
            format!("*({} *)zk_args[{i}]", self.held(kind))
        );
        let (size, align) =
            (kind.size(self.platform).max(1), kind.align(self.platform));
        let slot = |name: &str| {
            format!("    _Alignas({align}) unsigned char {name}[{size}];\n")
        };
        steps.push_str(&slot(&format!("zk_v{i}")));
        let (at, chars) =
            (format!("zk_args[{i}]"), format!("*(const char **)zk_args[{i}]"));
        let ok_at = kind.as_struct().map_or(0, |s| s.offset(1, self.platform));
        let cells = box_list(kind).map(|k| descr.of(k)).unwrap_or_default();
        let fill = match crossing
        {
            Crossing::Struct => format!("memcpy(zk_v{i}, {at}, {size});"),
            Crossing::Text => format!("zk_text_of_c(zk_v{i}, {chars});"),
            Crossing::OptText =>
            {
                format!("zk_text_from_c(zk_v{i}, {size}, {ok_at}, {chars});")
            }
            Crossing::Handle => format!("zk_cb_handle(zk_v{i}, {at}, {cells});"),
            _ => format!("zk_cb_opt_handle(zk_v{i}, {size}, {ok_at}, {at}, {cells});"),
        };
        let _ = writeln!(steps, "    {fill}");
        ret_if!(crossing == Crossing::Struct, format!("zk_v{i}"));
        let d = descr.of(kind);
        steps.push_str(&slot(&format!("zk_l{i}")));
        let _ = writeln!(
            steps,
            "    memcpy(zk_l{i}, zk_v{i}, {size});\n    zk_lend_seqs(zk_l{i}, {d});"
        );
        let _ = writeln!(drops, "    zk_drop(zk_v{i}, {d});");
        format!("zk_l{i}")
    }
}

/// Add a line that marks a parameter used, when the bridge reads none of it.
///
/// # Arguments
///
/// * `steps` - The steps so far.
/// * `unused` - Whether the parameter is unused.
/// * `name` - Its name.
fn do_unused(steps: &mut String, unused: bool, name: &str)
{
    ret_if!(!unused);
    let _ = writeln!(steps, "    (void){name};");
}
