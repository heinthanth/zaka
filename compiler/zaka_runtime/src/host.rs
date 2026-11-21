//! The host functions of the stdlib, on the world of the terminal that the VM
//! uses too, so a native run reads and writes as a VM run does.

use std::ffi::CStr;
use std::ffi::c_char;
use std::io;
use std::io::Write;

use zaka_host::HostFn;
use zaka_host::NO_PERMISSION;
use zaka_host::OTHER;
use zaka_host::Right;
use zaka_host::STDERR;
use zaka_host::STDIN;
use zaka_host::STDOUT;
use zaka_host::TIME_RANGE;
use zaka_host::TermWorld;
use zaka_host::World;
use zaka_host::io_member;
use zaka_host::time;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use crate::out::emit;
use crate::out::flush_out;
use crate::program::ZkKind;
use crate::rights::allowed;
use crate::seqs::zk_seq_base;
use crate::seqs::zk_seq_len;
use crate::seqs::zk_seq_new;
use crate::seqs::zk_seq_put;
use crate::tasks::blocking;
use crate::traps::trap;
use crate::traps::zk_addr_trap;
use crate::traps::zk_plain_trap;

/// The octets of text at a slot.
///
/// # Arguments
///
/// * `at` - The slot: text or a view.
///
/// # Returns
///
/// The octets.
pub fn text_at<'a>(at: *const u8) -> &'a [u8]
{
    let (base, len) = (zk_seq_base(at), zk_seq_len(at));
    ret_if!(len == 0 || base.is_null(), &[]);
    // SAFETY: a text slot names `len` octets at its base.
    unsafe { std::slice::from_raw_parts(base, len) }
}

/// The text at a slot as a Rust string, for a path or a name.
///
/// # Arguments
///
/// * `at` - The slot.
///
/// # Returns
///
/// The text; a zero octet ends it, as C reads it.
fn string_at(at: *const u8) -> String
{
    let raw = text_at(at);
    let cut = raw.split(|c| *c == 0).next().unwrap_or(raw);
    String::from_utf8_lossy(cut).into_owned()
}

/// Text in a heap block of its exact size, at a slot; text with no octets owns
/// no block.
///
/// # Arguments
///
/// * `at` - The slot: three words.
/// * `s` - The octets.
pub fn text_put(at: *mut u8, s: &[u8])
{
    ret_if!(s.is_empty(), zk_seq_put(at, std::ptr::null_mut(), 0, 0));
    let block = zk_seq_new(s.len());
    // SAFETY: the block holds `s.len()` octets.
    unsafe { std::ptr::copy_nonoverlapping(s.as_ptr(), block, s.len()) };
    zk_seq_put(at, block, s.len(), s.len());
}

/// Write a good result: its tag, and its text when the kind has one.
///
/// # Arguments
///
/// * `slot` - The result.
/// * `ret` - Its kind.
/// * `s` - The octets of its text, or `None` for a result with no value.
fn good(slot: *mut u8, ret: &ZkKind, s: Option<&[u8]>)
{
    set_tag(slot, ret, 1);
    let ok = ret.field("ok").map(|f| f.offset);
    if let (Some(at), Some(s)) = (ok, s)
    {
        // SAFETY: the field lies in the result.
        text_put(unsafe { slot.add(at) }, s);
    }
}

/// Write a good result that holds a word: a member of an enumeration.
///
/// # Arguments
///
/// * `slot` - The result.
/// * `ret` - Its kind.
/// * `word` - The word.
fn good_word(slot: *mut u8, ret: &ZkKind, word: isize)
{
    set_tag(slot, ret, 1);
    if let Some(f) = ret.field("ok")
    {
        // SAFETY: the field lies in the result and holds a word.
        unsafe { slot.add(f.offset).cast::<isize>().write_unaligned(word) };
    }
}

/// Write the tag of a result.
///
/// # Arguments
///
/// * `slot` - The result.
/// * `ret` - Its kind.
/// * `tag` - 1 for good, 0 for a fault.
fn set_tag(slot: *mut u8, ret: &ZkKind, tag: u8)
{
    let at = ret.field("tag").map_or(0, |f| f.offset);
    // SAFETY: the tag lies in the result.
    unsafe { *slot.add(at) = tag };
}

/// Write a failed result: the member of the I/O error, as a word.
///
/// # Arguments
///
/// * `slot` - The result.
/// * `ret` - Its kind.
/// * `member` - The member.
fn fault(slot: *mut u8, ret: &ZkKind, member: u32)
{
    set_tag(slot, ret, 0);
    if let Some(f) = ret.field("err")
    {
        // SAFETY: the field lies in the result and holds a word.
        unsafe { slot.add(f.offset).cast::<isize>().write_unaligned(member as isize) };
    }
}

/// Write the outcome of an effect with no value.
///
/// # Arguments
///
/// * `slot` - The result.
/// * `ret` - Its kind.
/// * `done` - The outcome.
fn outcome(slot: *mut u8, ret: &ZkKind, done: io::Result<()>)
{
    match done
    {
        Ok(()) => good(slot, ret, None),
        Err(e) => fault(slot, ret, io_member(&e)),
    }
}

/// The handle of a stream: the first field of its struct, cut to 32 bits as
/// the VM cuts it.
///
/// # Arguments
///
/// * `at` - The stream.
/// * `k` - Its kind.
///
/// # Returns
///
/// The handle.
fn handle(at: *const u8, k: &ZkKind) -> u32
{
    // SAFETY: a stream kind has a first field, of an integer kind.
    let size = unsafe { (*k.field_list()[0].kind).size }.min(8);
    let mut raw = [0u8; 8];
    // SAFETY: the stream holds `size` octets of its handle.
    unsafe { std::ptr::copy_nonoverlapping(at, raw.as_mut_ptr(), size) };
    u64::from_le_bytes(raw) as u32
}

/// The kind of a result, as the generated code hands it.
///
/// # Arguments
///
/// * `ret` - The kind.
///
/// # Returns
///
/// The kind.
fn kind<'a>(ret: *const ZkKind) -> &'a ZkKind
{
    // SAFETY: the generated code hands a descriptor of `zaka_kinds.c`.
    unsafe { &*ret }
}

/// Run an effect on a path the program holds a right for; a path it holds
/// none for fails with the member for no permission.
///
/// # Arguments
///
/// * `slot` - The result.
/// * `ret` - Its kind.
/// * `right` - The right the effect needs.
/// * `path` - The path, as text.
/// * `f` - The effect, which writes the result.
fn with_right(
    slot: *mut u8, ret: *const ZkKind, right: Right, path: *const u8,
    f: impl FnOnce(&str, &ZkKind),
)
{
    let name = string_at(path);
    let ret = kind(ret);
    ret_if!(!allowed(right, &name), fault(slot, ret, NO_PERMISSION));
    f(&name, ret)
}

/// Read a line of standard input.
#[unsafe(export_name = stamped!("host_read_line"))]
pub extern "C" fn zk_host_read_line(
    slot: *mut u8, ret: *const ZkKind, stream: *const u8, stream_kind: *const ZkKind,
)
{
    let ret = kind(ret);
    ret_if!(handle(stream, kind(stream_kind)) != STDIN, fault(slot, ret, OTHER));
    flush_out();
    match TermWorld.read_line()
    {
        Ok(Some(line)) => good(slot, ret, Some(line.as_bytes())),
        Ok(None) => fault(slot, ret, zaka_host::END_OF_INPUT),
        Err(e) => fault(slot, ret, io_member(&e)),
    }
}

/// Read a whole file as text.
#[unsafe(export_name = stamped!("host_read_file"))]
pub extern "C" fn zk_host_read_file(slot: *mut u8, ret: *const ZkKind, path: *const u8)
{
    with_right(slot, ret, Right::Read, path, |name, ret| {
        match blocking(|| TermWorld.read_file(name))
        {
            Ok(text) => good(slot, ret, Some(text.as_bytes())),
            Err(e) => fault(slot, ret, io_member(&e)),
        }
    })
}

/// Write text as the whole of a file.
#[unsafe(export_name = stamped!("host_write_file"))]
pub extern "C" fn zk_host_write_file(
    slot: *mut u8, ret: *const ZkKind, text: *const u8, path: *const u8,
)
{
    let data = String::from_utf8_lossy(text_at(text)).into_owned();
    with_right(slot, ret, Right::Write, path, |name, ret| {
        outcome(slot, ret, blocking(|| TermWorld.write_file(name, &data)))
    })
}

/// Add text at the end of a file.
#[unsafe(export_name = stamped!("host_append_file"))]
pub extern "C" fn zk_host_append_file(
    slot: *mut u8, ret: *const ZkKind, text: *const u8, path: *const u8,
)
{
    let data = String::from_utf8_lossy(text_at(text)).into_owned();
    with_right(slot, ret, Right::Write, path, |name, ret| {
        outcome(slot, ret, blocking(|| TermWorld.append_file(name, &data)))
    })
}

/// Write text to standard output or standard error.
#[unsafe(export_name = stamped!("host_write_stream"))]
pub extern "C" fn zk_host_write_stream(
    slot: *mut u8, ret: *const ZkKind, text: *const u8, stream: *const u8,
    stream_kind: *const ZkKind,
)
{
    let ret = kind(ret);
    let octets = text_at(text);
    match handle(stream, kind(stream_kind))
    {
        STDOUT => emit(octets),
        STDERR =>
        {
            flush_out();
            let mut err = io::stderr().lock();
            let _ = err.write_all(octets);
            let _ = err.flush();
        }
        _ => return fault(slot, ret, OTHER),
    }
    good(slot, ret, None)
}

/// Remove a file, or a folder that holds nothing.
#[unsafe(export_name = stamped!("host_remove"))]
pub extern "C" fn zk_host_remove(slot: *mut u8, ret: *const ZkKind, path: *const u8)
{
    with_right(slot, ret, Right::Write, path, |name, ret| {
        outcome(slot, ret, blocking(|| TermWorld.remove(name)))
    })
}

/// Make a folder and each folder above it.
#[unsafe(export_name = stamped!("host_make_folder"))]
pub extern "C" fn zk_host_make_folder(
    slot: *mut u8, ret: *const ZkKind, path: *const u8,
)
{
    with_right(slot, ret, Right::Write, path, |name, ret| {
        outcome(slot, ret, blocking(|| TermWorld.make_folder(name)))
    })
}

/// The names a folder holds, sorted.
#[unsafe(export_name = stamped!("host_list_folder"))]
pub extern "C" fn zk_host_list_folder(
    slot: *mut u8, ret: *const ZkKind, path: *const u8,
)
{
    with_right(slot, ret, Right::Read, path, |name, ret| {
        match blocking(|| TermWorld.list_folder(name))
        {
            Err(e) => fault(slot, ret, io_member(&e)),
            Ok(names) => good_names(slot, ret, &names),
        }
    })
}

/// Write a good result that holds a list of names.
///
/// # Arguments
///
/// * `slot` - The result.
/// * `ret` - Its kind.
/// * `names` - The names.
fn good_names(slot: *mut u8, ret: &ZkKind, names: &[String])
{
    set_tag(slot, ret, 1);
    let Some(f) = ret.field("ok")
    else
    {
        return;
    };
    let rows: Vec<&[u8]> = names.iter().map(|n| n.as_bytes()).collect();
    // SAFETY: the field lies in the result.
    put_text_list(unsafe { slot.add(f.offset) }, &rows);
}

/// What a path holds: a file, a folder or something else.
#[unsafe(export_name = stamped!("host_path_kind"))]
pub extern "C" fn zk_host_path_kind(slot: *mut u8, ret: *const ZkKind, path: *const u8)
{
    with_right(slot, ret, Right::Read, path, |name, ret| {
        match blocking(|| TermWorld.path_kind(name))
        {
            Ok(k) => good_word(slot, ret, k as isize),
            Err(e) => fault(slot, ret, io_member(&e)),
        }
    })
}

/// Move a file or a folder, replacing what the target holds.
#[unsafe(export_name = stamped!("host_rename"))]
pub extern "C" fn zk_host_rename(
    slot: *mut u8, ret: *const ZkKind, from: *const u8, to: *const u8,
)
{
    let (old, new) = (string_at(from), string_at(to));
    let ret = kind(ret);
    ret_if!(
        !allowed(Right::Write, &old) || !allowed(Right::Write, &new),
        fault(slot, ret, NO_PERMISSION)
    );
    outcome(slot, ret, blocking(|| TermWorld.rename(&old, &new)))
}

/// Read a variable of the environment.
#[unsafe(export_name = stamped!("host_read_env"))]
pub extern "C" fn zk_host_read_env(slot: *mut u8, ret: *const ZkKind, name: *const u8)
{
    with_right(slot, ret, Right::Env, name, |var, ret| match TermWorld.read_env(var)
    {
        Ok(Some(value)) => good(slot, ret, Some(value.as_bytes())),
        _ => fault(slot, ret, zaka_host::NOT_FOUND),
    })
}

/// Run a math operation on two f64 slots. A call with one input passes the
/// same slot twice.
#[unsafe(export_name = stamped!("host_math"))]
pub extern "C" fn zk_host_math(
    slot: *mut u8, op: u32, first: *const u8, second: *const u8,
)
{
    let calc = HostFn::math_at(op).expect("a math host operation");
    // SAFETY: the generated call passes two f64 inputs and an f64 result slot.
    unsafe {
        let first = first.cast::<f64>().read_unaligned();
        let second = second.cast::<f64>().read_unaligned();
        slot.cast::<f64>().write_unaligned(calc(first, second));
    }
}

/// Read the clock for elapsed time, in milliseconds.
#[unsafe(export_name = stamped!("host_monotonic_ms"))]
pub extern "C" fn zk_host_monotonic_ms(slot: *mut u8)
{
    // SAFETY: the generated call passes an i64 result slot.
    unsafe { slot.cast::<i64>().write_unaligned(time::elapsed_ms(&mut TermWorld)) };
}

/// Read the UTC clock, in Unix milliseconds.
#[unsafe(export_name = stamped!("host_unix_ms"))]
pub extern "C" fn zk_host_unix_ms(slot: *mut u8)
{
    // SAFETY: the generated call passes an i64 result slot.
    unsafe { slot.cast::<i64>().write_unaligned(TermWorld.unix_ms()) };
}

/// Format a Unix millisecond stamp in UTC.
#[unsafe(export_name = stamped!("host_format_utc"))]
pub extern "C" fn zk_host_format_utc(slot: *mut u8, ret: *const ZkKind, ms: *const u8)
{
    // SAFETY: the generated call passes an i64 input slot.
    let ms = unsafe { ms.cast::<i64>().read_unaligned() };
    let ret = kind(ret);
    match time::format_utc(ms)
    {
        Some(stamp) => good(slot, ret, Some(stamp.as_bytes())),
        None => fault(slot, ret, TIME_RANGE),
    }
}

/// A plain list of text at a slot, each text copied.
///
/// # Arguments
///
/// * `at` - The slot: three words.
/// * `rows` - The texts.
fn put_text_list(at: *mut u8, rows: &[&[u8]])
{
    ret_if!(rows.is_empty(), zk_seq_put(at, std::ptr::null_mut(), 0, 0));
    let cell = 3 * size_of::<usize>();
    let size = rows.len() * cell;
    let block = zk_seq_new(size);
    for (i, row) in rows.iter().enumerate()
    {
        // SAFETY: the block holds a cell for each row.
        text_put(unsafe { block.add(i * cell) }, row);
    }
    zk_seq_put(at, block, rows.len(), size);
}

/// A plain list of text from C strings: the arguments or the environment of
/// the start block.
///
/// # Arguments
///
/// * `at` - The slot: three words.
/// * `count` - The number of texts.
/// * `items` - The texts, each zero-ended.
#[unsafe(export_name = stamped!("text_list"))]
pub extern "C" fn zk_text_list(at: *mut u8, count: i32, items: *mut *mut c_char)
{
    let n = count.max(0) as usize;
    // SAFETY: the caller hands `count` NUL-ended texts.
    let rows: Vec<&[u8]> =
        (0..n).map(|i| unsafe { CStr::from_ptr(*items.add(i)) }.to_bytes()).collect();
    put_text_list(at, &rows)
}

/// Text C passed to a function of the program, copied into text of its own;
/// NULL stops the run, as do octets that are not UTF-8.
///
/// # Arguments
///
/// * `out` - Where the text goes.
/// * `s` - What C passed.
#[unsafe(export_name = stamped!("text_of_c"))]
pub extern "C" fn zk_text_of_c(out: *mut u8, s: *const c_char)
{
    do_if!(s.is_null(), zk_plain_trap(trap::NULL_FROM_C));
    // SAFETY: C hands a NUL-ended string.
    let raw = unsafe { CStr::from_ptr(s) }.to_bytes();
    do_if!(std::str::from_utf8(raw).is_err(), zk_addr_trap(trap::BAD_UTF8, s.cast()));
    text_put(out, raw)
}

/// The result of a C function that gives back text, as an optional of text:
/// NULL is the optional with no data; else the octets are copied. The tag sits
/// at offset 0.
///
/// # Arguments
///
/// * `out` - The result: `size` octets.
/// * `size` - The size of the result.
/// * `ok_at` - The offset of the text in it.
/// * `s` - What C gave back.
#[unsafe(export_name = stamped!("text_from_c"))]
pub extern "C" fn zk_text_from_c(
    out: *mut u8, size: usize, ok_at: usize, s: *const c_char,
)
{
    // SAFETY: the result holds `size` octets.
    unsafe { std::ptr::write_bytes(out, 0, size) };
    ret_if!(s.is_null());
    // SAFETY: the text lies in the result.
    zk_text_of_c(unsafe { out.add(ok_at) }, s);
    // SAFETY: the tag is the first octet of the result.
    unsafe { *out = 1 };
}
