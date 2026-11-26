//! Values a C lib gives to C or takes from it in their C form: text as a
//! `char *` the lib allocates, a plain list as `{ ptr, count }` with the C
//! forms of its cells in a block the lib allocates, a counted list as the
//! cells in a row, and a struct as a C struct of the C forms of its fields.
//! Numbers, characters, booleans and addresses are the same in both forms.
//!
//! Each block the lib gives C starts after a head that names the kind of its
//! cells and their count. C frees one block with `zk_free`, or a block with
//! each block its cells point at with `zk_free_all`.
//!
//! The C form of a struct places its fields as C does: each at the next
//! offset its alignment allows, the struct as wide as its widest alignment.
//! The header of a lib declares the same places and checks its size.

use std::ffi::c_void;

use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::heap::WORD;
use crate::heap::ld;
use crate::heap::move_octets;
use crate::heap::st;
use crate::heap::zk_seq_new;
use crate::host::zk_text_of_c;
use crate::native::NO_SITE;
use crate::native::at_site;
use crate::program::ZkKind;
use crate::seqs::at;
use crate::seqs::octets;
use crate::seqs::tag;
use crate::seqs::zk_seq_base;
use crate::seqs::zk_seq_len;
use crate::seqs::zk_seq_put;
use crate::traps::zk_out_of_mem;

/// The kind a descriptor names.
///
/// # Arguments
///
/// * `k` - The descriptor.
///
/// # Returns
///
/// The kind.
fn kind<'a>(k: *const ZkKind) -> &'a ZkKind
{
    // SAFETY: the compiler hands a descriptor that lives for the whole run.
    unsafe { &*k }
}

/// The size and the alignment of the C form of a kind.
///
/// # Arguments
///
/// * `k` - The kind.
///
/// # Returns
///
/// The size and the alignment, in octets.
pub fn c_form(k: &ZkKind) -> (usize, usize)
{
    match k.tag
    {
        tag::TEXT => (WORD, WORD),
        tag::SEQ => (2 * WORD, WORD),
        tag::LIST =>
        {
            let (size, align) = c_form(kind(k.cell));
            (size * k.len, align)
        }
        tag::STRUCT =>
        {
            let (end, align) = c_offsets(k).last().map_or((0, 1), |o| (o.1, o.2));
            (end.next_multiple_of(align), align)
        }
        _ => (k.size, k.size.max(1)),
    }
}

/// The offset of each field of the C form of a struct, with the end of the
/// field and the widest alignment so far.
///
/// # Arguments
///
/// * `k` - The struct kind.
///
/// # Returns
///
/// For each field: its offset, its end, and the widest alignment up to it.
fn c_offsets(k: &ZkKind) -> Vec<(usize, usize, usize)>
{
    let mut out = Vec::with_capacity(k.len);
    let (mut end, mut widest): (usize, usize) = (0, 1);
    for f in k.field_list()
    {
        let (size, align) = c_form(kind(f.kind));
        let start = end.next_multiple_of(align);
        (end, widest) = (start + size, widest.max(align));
        out.push((start, end, widest));
    }
    out
}

/// Write the C form of a value, for C to take.
///
/// # Arguments
///
/// * `from` - The value.
/// * `k` - Its kind.
/// * `to` - Where the C form goes: room for it, as [`c_form`] gives; NULL, when
///   C has no use for the value, writes nothing.
#[unsafe(export_name = stamped!("to_c"))]
pub extern "C" fn zk_to_c(from: *const u8, k: *const ZkKind, to: *mut u8)
{
    ret_if!(to.is_null());
    let k = kind(k);
    match k.tag
    {
        tag::TEXT => st(to, c_text(from)),
        tag::SEQ => seq_to_c(from, k, to),
        tag::LIST =>
        {
            each_cell(k, from.cast_mut(), to, &|f, cell, t| zk_to_c(f, cell, t))
        }
        tag::STRUCT =>
        {
            each_field(k, from.cast_mut(), to, &|f, field, t| zk_to_c(f, field, t))
        }
        _ => move_octets(to, from, k.size),
    }
}

/// Read a value from its C form, as C gave it.
///
/// # Arguments
///
/// * `from` - The C form.
/// * `k` - The kind of the value.
/// * `to` - Where the value goes.
/// * `site` - The site of the expression that makes the call, for a trap inside
///   it; `NO_SITE` keeps the site the call runs under.
#[unsafe(export_name = stamped!("from_c"))]
pub extern "C" fn zk_from_c(from: *const u8, k: *const ZkKind, to: *mut u8, site: u32)
{
    let _at = at_site(site);
    let k = kind(k);
    match k.tag
    {
        tag::TEXT => zk_text_of_c(to, ld(from), NO_SITE),
        tag::SEQ => seq_from_c(from, k, to),
        tag::LIST => each_cell(k, to, from.cast_mut(), &|t, cell, f| {
            zk_from_c(f, cell, t, NO_SITE)
        }),
        tag::STRUCT => each_field(k, to, from.cast_mut(), &|t, field, f| {
            zk_from_c(f, field, t, NO_SITE)
        }),
        _ => move_octets(to, from, k.size),
    }
}

/// Run a step on each cell of a counted list in both forms.
///
/// # Arguments
///
/// * `k` - The list kind.
/// * `own` - The value, in the form of the program.
/// * `c` - The C form.
/// * `step` - The step: the cell in each form and its kind.
// The steps call the function that calls this one, so with this inlined, an
// optimized build inlines the step, then this again, without end: unwinding
// makes each round heavier, and LLVM's inliner runs for hours.
#[inline(never)]
fn each_cell(
    k: &ZkKind, own: *mut u8, c: *mut u8,
    step: &dyn Fn(*mut u8, *const ZkKind, *mut u8),
)
{
    let cell = kind(k.cell);
    let c_size = c_form(cell).0;
    do_for!(i in 0..k.len, step(at(own, i * cell.size), k.cell, at(c, i * c_size)));
}

/// Run a step on each field of a struct in both forms.
///
/// # Arguments
///
/// * `k` - The struct kind.
/// * `own` - The value, in the form of the program.
/// * `c` - The C form.
/// * `step` - The step: the field in each form and its kind.
// Kept out of line for the reason `each_cell` is.
#[inline(never)]
fn each_field(
    k: &ZkKind, own: *mut u8, c: *mut u8,
    step: &dyn Fn(*mut u8, *const ZkKind, *mut u8),
)
{
    let places = k.field_list().iter().zip(c_offsets(k));
    do_for!((f, (start, ..)) in places, step(at(own, f.offset), f.kind, at(c, start)));
}

/// Write the C form of a plain list: its cells in a block of the C
/// allocator, and their count; no block for no cell.
///
/// # Arguments
///
/// * `from` - The list.
/// * `k` - Its kind.
/// * `to` - Where the two words go.
fn seq_to_c(from: *const u8, k: &ZkKind, to: *mut u8)
{
    let count = zk_seq_len(from);
    let (cell, base) = (kind(k.cell), zk_seq_base(from));
    let c_size = c_form(cell).0;
    let cells = c_block(count * c_size, k.cell, count);
    let each =
        |i: usize| zk_to_c(at(base, i * cell.size), k.cell, at(cells, i * c_size));
    do_for!(i in 0..tern!(cells.is_null(), 0, count), each(i));
    st(to, cells);
    st(at(to, WORD), count);
}

/// The octets in front of a block the lib gives C: the kind of its cells and
/// their count, padded so the cells keep the alignment malloc gives.
const C_HEAD: usize = 16;

/// A block the lib gives C, after its head, or NULL for no octets.
///
/// # Arguments
///
/// * `size` - The octets of its cells.
/// * `cell` - The kind of each cell, for `zk_free_all`; NULL for text.
/// * `count` - The count of cells.
///
/// # Returns
///
/// The block.
fn c_block(size: usize, cell: *const ZkKind, count: usize) -> *mut u8
{
    ret_if!(size == 0, std::ptr::null_mut());
    // C owns the block, so it counts against no cap of `--max-heap`.
    let start = crate::heap::calloc(C_HEAD + size, 1).cast::<u8>();
    do_if!(start.is_null(), zk_out_of_mem(size as u64));
    st(start, cell);
    st(at(start, WORD), count);
    at(start, C_HEAD)
}

/// Free a block the lib gave C, with each block its cells point at.
///
/// # Arguments
///
/// * `p` - The `char *` or the `ptr` of a list; NULL frees nothing.
#[unsafe(export_name = stamped!("free_all"))]
pub extern "C" fn zk_free_all(p: *mut c_void)
{
    ret_if!(p.is_null());
    let start = p.cast::<u8>().wrapping_sub(C_HEAD);
    let (cell, count): (*const ZkKind, usize) = (ld(start), ld(at(start, WORD)));
    do_if!(!cell.is_null(), {
        let step = c_form(kind(cell)).0;
        do_for!(i in 0..count, free_inside(at(p.cast(), i * step), cell));
    });
    zk_free(p)
}

/// Free each block a value in its C form points at, and none of its own.
///
/// # Arguments
///
/// * `c` - The value in its C form.
/// * `k` - Its kind.
fn free_inside(c: *mut u8, k: *const ZkKind)
{
    let k = kind(k);
    match k.tag
    {
        tag::TEXT | tag::SEQ => zk_free_all(ld(c)),
        tag::LIST => each_cell(k, c, c, &|_, cell, f| free_inside(f, cell)),
        tag::STRUCT => each_field(k, c, c, &|_, field, f| free_inside(f, field)),
        _ => (),
    }
}

/// Free one block the lib gave C.
///
/// # Arguments
///
/// * `p` - The `char *` or the `ptr` of a list; NULL frees nothing.
#[unsafe(export_name = stamped!("free"))]
pub extern "C" fn zk_free(p: *mut c_void)
{
    ret_if!(p.is_null());
    // SAFETY: the block came from `c_block`, after its head, outside each
    // count of the heap.
    unsafe { crate::heap::free(p.cast::<u8>().wrapping_sub(C_HEAD).cast()) }
}

/// A duplicate of text that ends in a zero octet, in a raw block C frees with
/// `zk_free`.
///
/// # Arguments
///
/// * `from` - The text.
///
/// # Returns
///
/// The `char *`.
fn c_text(from: *const u8) -> *mut u8
{
    let raw = octets(from);
    let dup = c_block(raw.len() + 1, std::ptr::null(), raw.len() + 1);
    move_octets(dup, raw.as_ptr(), raw.len());
    st(at(dup, raw.len()), 0u8);
    dup
}

/// Read a plain list from its C form: the cells C gives, copied into a new
/// list.
///
/// # Arguments
///
/// * `from` - The two words.
/// * `k` - The list kind.
/// * `to` - Where the list goes.
fn seq_from_c(from: *const u8, k: &ZkKind, to: *mut u8)
{
    let (cells, count): (*const u8, usize) = (ld(from), ld(at(from, WORD)));
    let cell = kind(k.cell);
    let size = count * cell.size;
    ret_if!(size == 0 || cells.is_null(), zk_seq_put(to, std::ptr::null_mut(), 0, 0));
    let (block, c_size) = (zk_seq_new(size), c_form(cell).0);
    let each = |i: usize| {
        zk_from_c(
            at(cells.cast_mut(), i * c_size),
            k.cell,
            at(block, i * cell.size),
            NO_SITE,
        )
    };
    do_for!(i in 0..count, each(i));
    zk_seq_put(to, block, count, size);
}
