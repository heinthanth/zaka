//! The structs the generated code shares with the runtime: `zaka_runtime.h`,
//! which cbindgen writes from the Rust, must set each one out as rustc does,
//! and each list of ids the compiler writes in order must follow the enums of
//! the runtime.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::mem::offset_of;

use zaka_runtime::callbacks::ZkCbShape;
use zaka_runtime::ffi::ZkCKind;
use zaka_runtime::ffi::ZkCbNativeShape;
use zaka_runtime::ffi::ZkFfiShape;
use zaka_runtime::heap::ZkFnRec;
use zaka_runtime::libs::ZkLibPlace;
use zaka_runtime::libs::ZkTrapFrame;
use zaka_runtime::libs::ZkTrapInfo;
use zaka_runtime::out::ZkBuf;
use zaka_runtime::program::ZkField;
use zaka_runtime::program::ZkFnName;
use zaka_runtime::program::ZkFrame;
use zaka_runtime::program::ZkGrant;
use zaka_runtime::program::ZkKind;
use zaka_runtime::program::ZkMember;
use zaka_runtime::program::ZkProgram;
use zaka_runtime::program::ZkSite;
use zakac_codegen_target_c::descriptors::TAGS;
use zakac_codegen_target_c::driver::RUNTIME;
use zakac_codegen_target_c::emit::RIGHTS;
use zakac_codegen_target_c::emit::trap_texts;
use zakac_codegen_target_c::toolchain::Tool;

use crate::common::cc;

/// A shared struct as rustc sets it out: its C name, its size, and the
/// offset of each field.
struct Form
{
    /// The C name.
    name: &'static str,
    /// The size in octets.
    size: usize,
    /// Each field and its offset.
    fields: Vec<(&'static str, usize)>,
}

/// The [`Form`] of each struct listed, from rustc.
macro_rules! forms {
    ($($rust:ident => $c:literal { $($field:ident),* $(,)? })*) => {
        vec![$(Form {
            name: $c,
            size: size_of::<$rust>(),
            // A field C names with a reserved word of Rust, `where`, is raw in Rust.
            fields: vec![$((stringify!($field).trim_start_matches("r#"), offset_of!($rust, $field))),*],
        }),*]
    };
}

/// The shared structs, as rustc sets them out.
///
/// # Returns
///
/// The forms.
fn forms() -> Vec<Form>
{
    forms! {
        ZkProgram => "zk_program" {
            max_depth, debug_info, max_heap, checks, grants, grant_prompt, permission_text,
            flag_text, ask_text, trap_text, stopped_text, true_text, false_text, null_text,
            where_text, in_text, backtrace_text, call_at_text, call_text, call_again_text,
            calls_more_text, full_hint_text, good_text, fault_text, good_bare, fault_bare,
            data_text, no_data_text, fn_lead, fn_tail, fn_names, fn_count, sites, site_files,
        }
        ZkGrant => "zk_grant" { all, count, items }
        ZkSite => "zk_site" { file, line, col }
        ZkFrame => "zk_frame" { func, site }
        ZkFnName => "zk_fn_name" { code, name }
        ZkFnRec => "zk_fnrec" { code, none }
        ZkField => "zk_field" { name, offset, kind }
        ZkMember => "zk_member" { value, shown }
        ZkKind => "zk_kind" {
            tag, size, cell, len, fields, members, name, holds_text, holds_borrow, owns, boxed,
        }
        ZkBuf => "zk_buf" { data, len, cap, prev }
        ZkCbShape => "zk_cb_shape" { bridge, ret, args, nargs, as_is }
        ZkCbNativeShape => "zk_cb_native_shape" { bridge, ret, args, nargs, as_is }
        ZkCKind => "zk_c_kind" { tag, count, members }
        ZkFfiShape => "zk_ffi_shape" { ret, args, nargs, nfixed, variadic }
        ZkLibPlace => "zk_lib_place" { lib, prog, base }
        ZkTrapFrame => "zk_trap_frame" { function, file, line, column }
        ZkTrapInfo => "zk_trap_info" { code, kind, message, lib, r#where, frames, frame_count }
    }
}

/// The rows of the forms: `struct` with its size, `struct.field` with its
/// offset.
///
/// # Arguments
///
/// * `all` - The forms.
///
/// # Returns
///
/// The rows.
fn rows(all: &[Form]) -> BTreeMap<String, usize>
{
    let sizes = all.iter().map(|l| (l.name.to_string(), l.size));
    let fields = all
        .iter()
        .flat_map(|l| l.fields.iter().map(|(f, at)| (format!("{}.{f}", l.name), *at)));
    sizes.chain(fields).collect()
}

/// The C lines that print the same rows from the header.
///
/// # Arguments
///
/// * `all` - The forms.
///
/// # Returns
///
/// The lines.
fn c_lines(all: &[Form]) -> String
{
    let size = |l: &Form| format!("    printf(\"{0} %zu\\n\", sizeof({0}));\n", l.name);
    let field = |name: &str, f: &str| {
        format!("    printf(\"{name}.{f} %zu\\n\", offsetof({name}, {f}));\n")
    };
    let each = |l: &Form| {
        let fields = l.fields.iter().map(|(f, _)| field(l.name, f));
        std::iter::once(size(l)).chain(fields).collect::<String>()
    };
    all.iter().map(each).collect()
}

/// The text of the generated header.
///
/// # Returns
///
/// The header.
fn header() -> &'static str
{
    let found = RUNTIME.iter().find(|(name, _)| *name == "zaka_runtime.h");
    found.expect("the runtime files hold zaka_runtime.h").1
}

/// The lines between `typedef <what> <name> {` and its `}` in the header,
/// with the doc comments left out.
///
/// # Arguments
///
/// * `what` - `struct` or `enum`.
/// * `name` - The C name.
///
/// # Returns
///
/// The lines, trimmed.
fn block(what: &str, name: &str) -> Vec<&'static str>
{
    let open = format!("typedef {what} {name} {{");
    let lines = header().lines().skip_while(|l| *l != open).skip(1);
    let inside = lines.take_while(|l| !l.starts_with('}')).map(str::trim);
    inside
        .filter(|l| !l.is_empty() && !l.starts_with("/*") && !l.starts_with('*'))
        .collect()
}

/// The name a field line of a struct declares: the word before the `;`, or
/// the name of a function pointer.
///
/// # Arguments
///
/// * `line` - The line.
///
/// # Returns
///
/// The name.
fn field_name(line: &str) -> String
{
    if let Some((_, rest)) = line.split_once("(*")
    {
        return rest.split(')').next().unwrap_or_default().to_string();
    }
    let decl = line.trim_end_matches(';');
    decl.rsplit([' ', '*']).next().unwrap_or_default().to_string()
}

/// The structs the header declares, each with its fields.
///
/// # Returns
///
/// The rows, named as [`rows`] names them.
fn declared() -> BTreeSet<String>
{
    let structs = header().lines().filter_map(|l| l.strip_prefix("typedef struct "));
    let names = structs.filter_map(|l| l.strip_suffix(" {"));
    let rows = names.flat_map(|name| {
        let fields = block("struct", name).into_iter().map(field_name);
        let fields = fields.map(|f| format!("{name}.{f}")).collect::<Vec<_>>();
        std::iter::once(name.to_string()).chain(fields)
    });
    rows.collect()
}

/// The members of an enum of the header, in their order.
///
/// # Arguments
///
/// * `name` - The C name of the enum.
///
/// # Returns
///
/// The members.
fn members(name: &str) -> Vec<String>
{
    block("enum", name)
        .into_iter()
        .map(|l| l.trim_end_matches(',').to_string())
        .collect()
}

/// Each struct and each field of the header has a row here, so a new one
/// cannot go unchecked.
#[test]
fn each_shared_struct_and_field_has_a_row()
{
    let listed: BTreeSet<String> = rows(&forms()).into_keys().collect();
    assert_eq!(declared(), listed, "the header and the rows of this test differ");
}

/// Build and run a C program that prints the rows of the forms from the
/// header.
///
/// # Arguments
///
/// * `cc` - The C compiler.
/// * `all` - The forms.
///
/// # Returns
///
/// The rows the C compiler gives.
fn c_rows(cc: &Tool, all: &[Form]) -> BTreeMap<String, usize>
{
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("zaka_runtime.h"), header()).unwrap();
    let program = format!(
        "#include <stddef.h>\n#include <stdio.h>\n#include \"zaka_runtime.h\"\n\nint \
         main(void)\n{{\n{}    return 0;\n}}\n",
        c_lines(all)
    );
    std::fs::write(dir.path().join("forms.c"), program).unwrap();
    let exe = dir.path().join(format!("forms{}", std::env::consts::EXE_SUFFIX));
    let mut cmd = cc.command();
    cmd.current_dir(dir.path()).args(["-std=c11", "-pedantic-errors", "forms.c", "-o"]);
    let built = cmd.arg(&exe).output().unwrap();
    let errors = String::from_utf8_lossy(&built.stderr);
    assert!(built.status.success(), "the forms program did not build:\n{errors}");
    let ran = std::process::Command::new(&exe).output().unwrap();
    let printed = String::from_utf8_lossy(&ran.stdout).into_owned();
    let row =
        |l: &str| l.split_once(' ').map(|(n, v)| (n.to_string(), v.parse().unwrap()));
    printed.lines().filter_map(row).collect()
}

/// A C compiler sets out each shared struct as rustc does.
#[test]
fn the_header_sets_out_each_struct_as_rust_does()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let all = forms();
    assert_eq!(
        c_rows(&cc, &all),
        rows(&all),
        "C and Rust set out a shared struct apart"
    );
}

/// The trap texts the compiler lists follow `enum zk_trap`: the native
/// target writes them in order.
#[test]
fn the_trap_texts_follow_the_trap_enum()
{
    let listed = trap_texts().into_iter().map(|(name, _)| name.to_string());
    let listed: Vec<String> = listed.chain(["ZK_TRAP_COUNT".to_string()]).collect();
    assert_eq!(members("zk_trap"), listed);
}

/// The tags of the descriptors follow `enum zk_tag`, whose numbers the
/// native target writes.
#[test]
fn the_tags_follow_the_tag_enum()
{
    assert_eq!(members("zk_tag"), TAGS.map(str::to_string).to_vec());
}

/// The rights follow `enum zk_right`: the native target writes the grants,
/// the permission texts and the flags in order.
#[test]
fn the_rights_follow_the_right_enum()
{
    let listed: Vec<String> = RIGHTS.iter().map(|r| r.to_string()).collect();
    let listed = [listed, vec!["ZK_RIGHT_COUNT".to_string()]].concat();
    assert_eq!(members("zk_right"), listed);
}
