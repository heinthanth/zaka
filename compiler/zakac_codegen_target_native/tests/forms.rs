//! The data the native target writes for the runtime: the table of the
//! program and the kind descriptors, set out field for field as Cranelift data.
//! Each sample is lowered into this process, and its data is read back as
//! the runtime reads it, through the Rust structs of `zaka_runtime`.

use std::collections::BTreeMap;
use std::ffi::c_char;
use std::ptr::null;

use cranelift_jit::JITModule;
use zaka_host::Grant;
use zaka_host::Grants;
use zaka_runtime::program::ZkFnName;
use zaka_runtime::program::ZkGrant;
use zaka_runtime::program::ZkKind;
use zaka_runtime::program::ZkMember;
use zaka_runtime::program::ZkProgram;
use zaka_runtime::program::ZkSite;
use zaka_runtime::program::text;
use zakac_borrow_checker::options::OverflowMode;
use zakac_codegen_target_c::descriptors::KindRow;
use zakac_codegen_target_c::emit::table_texts;
use zakac_codegen_target_c::options::TargetCOptions;
use zakac_codegen_target_native::emit::Lowered;
use zakac_codegen_target_native::jit::lower;
use zakac_ir_builder::ir::Module as IrModule;

use crate::common::checked;
use crate::samples::sample_files;

/// A slice the data holds, or a blank one for NULL.
///
/// # Arguments
///
/// * `at` - The first item.
/// * `n` - The count.
///
/// # Returns
///
/// The items.
fn items<'a, T>(at: *const T, n: usize) -> &'a [T]
{
    let blank = at.is_null() || n == 0;
    // SAFETY: the data holds `n` items at a non-NULL address.
    if blank
    {
        &[]
    }
    else
    {
        unsafe { std::slice::from_raw_parts(at, n) }
    }
}

/// The texts of a list of C strings.
///
/// # Arguments
///
/// * `at` - The list.
/// * `n` - The count.
///
/// # Returns
///
/// The texts.
fn texts<'a>(at: *const *const c_char, n: usize) -> Vec<&'a [u8]>
{
    items(at, n).iter().map(|t| text(*t)).collect()
}

/// The rights the tables of the test hold: one of each form of a grant.
///
/// # Returns
///
/// The grants.
fn grants() -> Grants
{
    let read = Grant::Some(vec!["data".into()]);
    Grants::builder().read(read).write(Grant::All).ffi(true).prompt(true).build()
}

/// The choices of the build, each set apart from its default.
///
/// # Returns
///
/// The options.
fn options() -> TargetCOptions
{
    TargetCOptions::builder().max_depth(777).max_heap(4321).debug_info(true).build()
}

/// The table holds the choices, the rights, each text and the functions, at
/// the fields where the runtime reads them.
///
/// # Arguments
///
/// * `m` - The module, final.
/// * `lowered` - What the lowering gave.
/// * `program` - The IR.
fn check_table(m: &JITModule, lowered: &Lowered, program: &IrModule)
{
    let (at, size) = m.get_finalized_data(lowered.table);
    assert_eq!(size, size_of::<ZkProgram>(), "the size of the table");
    // SAFETY: the data is a table of the size of `ZkProgram`.
    let p = unsafe { &*at.cast::<ZkProgram>() };
    let o = options();
    let config = (p.max_depth, p.debug_info, p.max_heap, p.checks);
    assert_eq!(config, (o.max_depth, 1, o.max_heap, u32::from(o.checks.bits())));
    check_grants(p);
    check_texts(p);
    let names = items(p.fn_names, p.fn_count + 1);
    assert_eq!(p.fn_count, program.funcs.len(), "the count of functions");
    let named = names[..p.fn_count]
        .iter()
        .all(|n| n.code.is_some() && !text(n.name).is_empty());
    assert!(named, "a row of the function names is blank");
    let last: &ZkFnName = &names[p.fn_count];
    assert!(last.code.is_none() && last.name.is_null(), "the names end in a NULL row");
    let first: &ZkSite = &items(p.sites, 1)[0];
    assert_eq!((first.file, first.line, first.col), (0, 0, 0), "site 0 names no place");
    assert!(!texts(p.site_files, 1)[0].is_empty(), "the first site file has a name");
}

/// The rights of the table are those the build gave, one row for each right.
///
/// # Arguments
///
/// * `p` - The table.
fn check_grants(p: &ZkProgram)
{
    let rows: &[ZkGrant] = items(p.grants, 4);
    let each: Vec<(i32, i32)> = rows.iter().map(|g| (g.all, g.count)).collect();
    assert_eq!(each, [(0, 1), (1, 0), (0, 0), (1, 0)], "the grants of each right");
    assert_eq!(
        texts(rows[0].items, 1),
        [b"data".as_slice()],
        "the paths of the read right"
    );
    assert_eq!(p.grant_prompt, 1, "the prompt flag");
}

/// Each text field of the table holds the text of its name, and each list
/// its texts in order.
///
/// # Arguments
///
/// * `p` - The table.
fn check_texts(p: &ZkProgram)
{
    let all = table_texts();
    #[rustfmt::skip]
    let fields = [
        ("ask_text", p.ask_text), ("stopped_text", p.stopped_text),
        ("true_text", p.true_text), ("false_text", p.false_text), ("null_text", p.null_text),
        ("where_text", p.where_text), ("in_text", p.in_text),
        ("backtrace_text", p.backtrace_text), ("call_at_text", p.call_at_text),
        ("call_text", p.call_text), ("call_again_text", p.call_again_text),
        ("calls_more_text", p.calls_more_text), ("full_hint_text", p.full_hint_text),
        ("good_text", p.good_text), ("fault_text", p.fault_text),
        ("good_bare", p.good_bare), ("fault_bare", p.fault_bare),
        ("data_text", p.data_text), ("no_data_text", p.no_data_text),
        ("fn_lead", p.fn_lead), ("fn_tail", p.fn_tail),
    ];
    let got: BTreeMap<&str, &[u8]> =
        fields.into_iter().map(|(n, t)| (n, text(t))).collect();
    let want: BTreeMap<&str, &[u8]> =
        all.single.iter().map(|(n, t)| (*n, t.as_bytes())).collect();
    assert_eq!(got, want, "the text fields of the table");
    let lists = [
        (texts(p.trap_text, all.trap.len()), &all.trap),
        (texts(p.permission_text, all.permission.len()), &all.permission),
        (texts(p.flag_text, all.flags.len()), &all.flags),
    ];
    let same = |(got, want): &(Vec<&[u8]>, &Vec<String>)| {
        got.iter().copied().eq(want.iter().map(String::as_bytes))
    };
    assert!(lists.iter().all(same), "a list of texts of the table differs");
}

/// Each descriptor holds its row, at the fields where the runtime reads it.
///
/// # Arguments
///
/// * `m` - The module, final.
/// * `lowered` - What the lowering gave.
fn check_descrs(m: &JITModule, lowered: &Lowered)
{
    let data = lowered.descrs.iter().map(|(id, _)| m.get_finalized_data(*id));
    let data: Vec<(*const u8, usize)> = data.collect();
    let sized = data.iter().all(|(_, size)| *size == size_of::<ZkKind>());
    assert!(sized, "a descriptor is not the size of zk_kind");
    let all: Vec<*const ZkKind> = data.iter().map(|(at, _)| at.cast()).collect();
    let rows = lowered.descrs.iter().map(|(_, row)| row);
    rows.zip(&all).for_each(|(row, k)| check_kind(row, *k, &all));
}

/// One descriptor holds its row.
///
/// # Arguments
///
/// * `row` - The row it was written from.
/// * `k` - The descriptor.
/// * `all` - Each descriptor, for the kinds it points at.
fn check_kind(row: &KindRow, k: *const ZkKind, all: &[*const ZkKind])
{
    // SAFETY: the data is a descriptor of the size of `ZkKind`.
    let k = unsafe { &*k };
    let at = |i: Option<usize>| i.map_or(null(), |i| all[i]);
    let head = (k.tag, k.size, k.len, k.cell, text(k.name));
    let name = row.name.as_deref().unwrap_or_default().as_bytes();
    assert_eq!(head, (row.tag, row.size as usize, row.len, at(row.cell), name));
    let flags = (k.holds_text, k.holds_borrow, k.owns, k.boxed);
    let want = [row.holds_text, row.holds_borrow, row.owns, row.boxed].map(i32::from);
    assert_eq!(
        flags,
        (want[0], want[1], want[2], want[3]),
        "the flags of a descriptor"
    );
    let fields = items(k.fields, row.fields.len()).iter();
    let fields: Vec<_> = fields.map(|f| (text(f.name), f.offset, f.kind)).collect();
    let rows = row.fields.iter();
    let want: Vec<_> =
        rows.map(|(n, o, i)| (n.as_bytes(), *o as usize, all[*i])).collect();
    assert_eq!(fields, want, "the fields of {}", String::from_utf8_lossy(name));
    let members: &[ZkMember] = items(k.members, row.members.len());
    let members: Vec<_> =
        members.iter().map(|m| (text(m.value), text(m.shown))).collect();
    let want: Vec<_> =
        row.members.iter().map(|(v, s)| (v.as_bytes(), s.as_bytes())).collect();
    assert_eq!(members, want, "the members of a descriptor");
}

/// The table and the descriptors of each sample read back as the runtime
/// reads them.
#[test]
fn the_data_of_each_sample_reads_back_as_the_runtime_reads_it()
{
    for (_, file) in sample_files()
    {
        let (sm, program) = checked(&file, OverflowMode::Stop);
        let lowered = lower(&program, sm.files(), options(), &grants());
        let (m, lowered) =
            lowered.unwrap_or_else(|e| panic!("{}: {e}", file.display()));
        check_table(&m, &lowered, &program);
        check_descrs(&m, &lowered);
    }
}
