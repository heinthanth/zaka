//! Walks over a value that keep their steps on the heap, so a long chain of
//! objects takes no stack: the test of two values for sameness and the print of
//! one; the rows of a ဇယား; and the stops of results.

use std::collections::HashSet;
use std::ffi::c_char;

use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::heap::ZkFn;
use crate::heap::ld;
use crate::heap::move_octets;
use crate::heap::seq_grow;
use crate::heap::seq_kind;
use crate::heap::st;
use crate::heap::zero;
use crate::heap::zk_fn_code;
use crate::heap::zk_seq_new;
use crate::out::ZkBuf;
use crate::out::emit;
use crate::out::zk_put_bool;
use crate::out::zk_put_char;
use crate::out::zk_put_f32;
use crate::out::zk_put_f64;
use crate::out::zk_put_i;
use crate::out::zk_put_u;
use crate::out::zk_sink_close;
use crate::out::zk_sink_open;
use crate::program::ZkField;
use crate::program::ZkFnName;
use crate::program::ZkKind;
use crate::program::prog;
use crate::program::text;
use crate::seqs::at;
use crate::seqs::kind;
use crate::seqs::octets;
use crate::seqs::repr;
use crate::seqs::seq_room;
use crate::seqs::tag;
use crate::seqs::zk_drop;
use crate::seqs::zk_own;
use crate::seqs::zk_seq_base;
use crate::seqs::zk_seq_len;
use crate::seqs::zk_seq_put;
use crate::traps::stop_owned;
use crate::traps::trap;
use crate::traps::zk_plain_trap;

/// One step of a walk over a value.
enum Step
{
    /// Compare two values, or write one (`b` NULL); `nested` writes text in
    /// quotes.
    Value
    {
        a: *const u8,
        b: *const u8,
        k: *const ZkKind,
        nested: bool,
    },
    /// Write a piece of text.
    Piece(&'static [u8]),
    /// Leave an object the print is inside.
    Leave(*const u8),
}

/// The separator of cells and fields: U+104A and a space.
const SEP: &[u8] = "\u{104a} ".as_bytes();

/// A piece of text from a C string of the kind tables or the program table.
///
/// # Arguments
///
/// * `p` - The text.
///
/// # Returns
///
/// The step.
fn piece(p: *const c_char) -> Step { Step::Piece(text(p)) }

/// The field of a struct with a name.
///
/// # Arguments
///
/// * `k` - The struct kind.
/// * `name` - The name.
///
/// # Returns
///
/// The field, or NULL.
#[unsafe(export_name = stamped!("field_named"))]
pub extern "C" fn zk_field_named(
    k: *const ZkKind, name: *const c_char,
) -> *const ZkField
{
    let wanted = text(name);
    let found = kind(k).field_list().iter().find(|f| text(f.name) == wanted);
    found.map_or(std::ptr::null(), |f| f as *const ZkField)
}

// ---- sameness ----

/// Compare one pair of values: scalars and text at once, the parts of a list,
/// a plain list, an object or a struct as steps. Two objects compared before
/// count as equal, so a ring ends.
///
/// # Arguments
///
/// * `a` - The first value.
/// * `b` - The second value.
/// * `k` - The kind.
/// * `steps` - The steps, added to.
/// * `seen` - The pairs of objects met.
///
/// # Returns
///
/// `false` when the pair differs.
fn eq_step(
    a: *const u8, b: *const u8, k: &ZkKind, steps: &mut Vec<Step>,
    seen: &mut HashSet<(usize, usize)>,
) -> bool
{
    let k = repr(k);
    let pair = |a: *const u8, b: *const u8, k: *const ZkKind| Step::Value {
        a,
        b,
        k,
        nested: false,
    };
    match k.tag
    {
        tag::TEXT => octets(a) == octets(b),
        tag::SEQ if k.boxed != 0 =>
        {
            let (la, lb) = (zk_seq_base(a), zk_seq_base(b));
            ret_if!(la == lb, true);
            ret_if!(la.is_null() || lb.is_null(), false);
            ret_if!(!seen.insert((la as usize, lb as usize)), true);
            steps.push(pair(la, lb, k.cell));
            true
        }
        tag::SEQ =>
        {
            ret_if!(zk_seq_len(a) != zk_seq_len(b), false);
            push_pairs(steps, zk_seq_base(a), zk_seq_base(b), k.cell, zk_seq_len(a));
            true
        }
        tag::LIST =>
        {
            push_pairs(steps, a, b, k.cell, k.len);
            true
        }
        tag::STRUCT | tag::FALLIBLE if is_table(k) => table_eq_step(a, b, k, steps),
        tag::STRUCT | tag::FALLIBLE =>
        {
            do_for!(f in k.field_list(), steps.push(pair(at(a, f.offset), at(b, f.offset), f.kind)));
            true
        }
        tag::FLOAT if k.size == 4 => ld::<f32>(a) == ld::<f32>(b),
        tag::FLOAT => ld::<f64>(a) == ld::<f64>(b),
        // SAFETY: two values of one kind hold `size` octets each.
        _ =>
        unsafe {
            std::slice::from_raw_parts(a, k.size)
                == std::slice::from_raw_parts(b, k.size)
        },
    }
}

/// Add the steps that compare two runs of cells, each to the cell at the same
/// place.
///
/// # Arguments
///
/// * `steps` - The steps.
/// * `a` - The first cell of the first run.
/// * `b` - The first cell of the second run.
/// * `cell` - The kind of a cell.
/// * `count` - The cell count of each.
fn push_pairs(
    steps: &mut Vec<Step>, a: *const u8, b: *const u8, cell: *const ZkKind,
    count: usize,
)
{
    let size = kind(cell).size;
    do_for!(i in 0..count, steps.push(Step::Value { a: at(a, i * size), b: at(b, i * size), k: cell, nested: false }));
}

/// Whether two values are equal.
///
/// # Arguments
///
/// * `a` - The first value.
/// * `b` - The second value.
/// * `k` - The kind.
///
/// # Returns
///
/// 1 when the two are equal.
#[unsafe(export_name = stamped!("eq"))]
pub extern "C" fn zk_eq(a: *const u8, b: *const u8, k: *const ZkKind) -> i32
{
    let mut steps = vec![Step::Value { a, b, k, nested: false }];
    let mut seen = HashSet::new();
    while let Some(step) = steps.pop()
    {
        let Step::Value { a, b, k, .. } = step
        else
        {
            continue;
        };
        ret_if!(!eq_step(a, b, kind(k), &mut steps, &mut seen), 0);
    }
    1
}

// ---- print ----

/// Write text, in quotes when it is nested in a list or a struct: a backslash
/// and a quote escaped, each brace doubled.
///
/// # Arguments
///
/// * `p` - The text or view.
/// * `nested` - Whether to quote it.
fn show_text(p: *const u8, nested: bool)
{
    let s = octets(p);
    ret_if!(!nested, emit(s));
    let mut out = Vec::with_capacity(s.len() + 2);
    out.push(b'"');
    for &c in s
    {
        do_if!(c == b'\\' || c == b'"', out.push(b'\\'));
        do_if!(c == b'{' || c == b'}', out.push(c));
        out.push(c);
    }
    out.push(b'"');
    emit(&out)
}

/// The text the print of a value writes.
///
/// # Arguments
///
/// * `p` - The value.
/// * `k` - Its kind.
///
/// # Returns
///
/// The text.
fn shown(p: *const u8, k: *const ZkKind) -> Vec<u8>
{
    #[rustfmt::skip]
    let mut buf = ZkBuf { data: std::ptr::null_mut(), len: 0, cap: 0, prev: std::ptr::null_mut() };
    zk_sink_open(&mut buf);
    zk_show(p, k, 0);
    let mut held = [0usize; 3];
    let slot = held.as_mut_ptr().cast::<u8>();
    zk_sink_close(slot);
    let out = octets(slot).to_vec();
    crate::heap::zk_seq_free(zk_seq_base(slot));
    out
}

/// Write a member of an enumeration: the member whose value prints as the
/// value at the address does, or that value when no member has it.
///
/// # Arguments
///
/// * `p` - The value.
/// * `k` - The enumeration kind.
fn show_member(p: *const u8, k: &ZkKind)
{
    let value = shown(p, k.cell);
    // SAFETY: an enumeration kind holds `len` members.
    let members = unsafe { std::slice::from_raw_parts(k.members, k.len) };
    let found = members.iter().find(|m| text(m.value) == value.as_slice());
    emit(found.map_or(value.as_slice(), |m| text(m.shown)))
}

/// Write an address, or the words for the null address.
///
/// # Arguments
///
/// * `p` - The address.
#[unsafe(export_name = stamped!("put_ptr"))]
pub extern "C" fn zk_put_ptr(p: *const u8)
{
    ret_if!(p.is_null(), emit(text(prog().null_text)));
    emit(format!("{:#x}", p as usize).as_bytes())
}

/// Write a function value: the name of its function, or the words for the null
/// address.
///
/// # Arguments
///
/// * `f` - The value.
#[unsafe(export_name = stamped!("put_fn"))]
pub extern "C" fn zk_put_fn(f: ZkFn)
{
    let p = prog();
    ret_if!(f.is_null(), emit(text(p.null_text)));
    // SAFETY: the table holds `fn_count` rows.
    let names = unsafe { std::slice::from_raw_parts(p.fn_names, p.fn_count) };
    let code = zk_fn_code(f);
    let same = |n: &&ZkFnName| n.code.map(|c| c as usize) == code.map(|c| c as usize);
    let Some(row) = names.iter().find(same)
    else
    {
        return emit(b"0x?");
    };
    emit(&[text(p.fn_lead), text(row.name), text(p.fn_tail)].concat())
}

/// Write a scalar held in storage.
///
/// # Arguments
///
/// * `p` - The value.
/// * `k` - Its kind.
fn show_scalar(p: *const u8, k: &ZkKind)
{
    match (k.tag, k.size)
    {
        (tag::SINT, 1) => zk_put_i(i64::from(ld::<i8>(p))),
        (tag::SINT, 2) => zk_put_i(i64::from(ld::<i16>(p))),
        (tag::SINT, 4) => zk_put_i(i64::from(ld::<i32>(p))),
        (tag::SINT, 8) => zk_put_i(ld::<i64>(p)),
        (tag::UINT, 1) => zk_put_u(u64::from(ld::<u8>(p))),
        (tag::UINT, 2) => zk_put_u(u64::from(ld::<u16>(p))),
        (tag::UINT, 4) => zk_put_u(u64::from(ld::<u32>(p))),
        (tag::UINT, 8) => zk_put_u(ld::<u64>(p)),
        (tag::FLOAT, 4) => zk_put_f32(ld(p)),
        (tag::FLOAT, 8) => zk_put_f64(ld(p)),
        (tag::CHAR, _) => zk_put_char(ld(p)),
        (tag::BOOL, _) => zk_put_bool(ld(p)),
        (tag::PTR, _) => zk_put_ptr(ld(p)),
        (tag::FN, _) => zk_put_fn(ld(p)),
        _ =>
        {}
    }
}

/// Add the steps that write a run of cells between `[` and `]`, the last
/// first.
///
/// # Arguments
///
/// * `s` - The steps.
/// * `base` - The first cell.
/// * `cell` - The kind of a cell.
/// * `count` - The cell count.
fn push_cells(s: &mut Vec<Step>, base: *const u8, cell: *const ZkKind, count: usize)
{
    let size = kind(cell).size;
    s.push(Step::Piece(b"]"));
    for i in (0..count).rev()
    {
        s.push(Step::Value {
            a: at(base, i * size),
            b: std::ptr::null(),
            k: cell,
            nested: true,
        });
        do_if!(i > 0, s.push(Step::Piece(SEP)));
    }
    s.push(Step::Piece(b"["));
}

/// Add the steps that write a struct, the last first: its name, then each
/// field as `name: value`.
///
/// # Arguments
///
/// * `s` - The steps.
/// * `p` - The struct.
/// * `k` - Its kind.
fn push_struct(s: &mut Vec<Step>, p: *const u8, k: &ZkKind)
{
    s.push(Step::Piece(b")"));
    let fields = k.field_list();
    for (i, f) in fields.iter().enumerate().rev()
    {
        s.push(Step::Value {
            a: at(p, f.offset),
            b: std::ptr::null(),
            k: f.kind,
            nested: true,
        });
        s.push(Step::Piece(b": "));
        s.push(piece(f.name));
        do_if!(i > 0, s.push(Step::Piece(SEP)));
    }
    s.push(Step::Piece(b"("));
    s.push(piece(k.name));
}

/// Add the steps that write a result that can fail, the last first: its good
/// value or its error value, or the word of a side with no value. An optional
/// writes its data, or the word of an optional with no data.
///
/// # Arguments
///
/// * `s` - The steps.
/// * `p` - The result.
/// * `k` - Its kind.
fn push_fallible(s: &mut Vec<Step>, p: *const u8, k: &ZkKind)
{
    let pr = prog();
    let failed = k.field("tag").is_some_and(|t| ld::<u8>(at(p, t.offset)) == 0);
    let some = k.field("some");
    ret_if!(some.is_some() && failed, s.push(piece(pr.no_data_text)));
    let part = some.or_else(|| k.field(tern!(failed, "err", "ok")));
    let Some(part) = part
    else
    {
        return s.push(piece(tern!(failed, pr.fault_bare, pr.good_bare)));
    };
    let word = match (some.is_some(), failed)
    {
        (true, _) => pr.data_text,
        (false, true) => pr.fault_text,
        (false, false) => pr.good_text,
    };
    s.push(Step::Piece(b")"));
    s.push(Step::Value {
        a: at(p, part.offset),
        b: std::ptr::null(),
        k: part.kind,
        nested: true,
    });
    s.push(Step::Piece(b"("));
    s.push(piece(word));
}

/// Add the steps that write an object: its struct, or the words for the null
/// address for an object with no value. An object the print is still inside
/// is written as an ellipsis.
///
/// # Arguments
///
/// * `s` - The steps.
/// * `inside` - The objects the print is inside.
/// * `p` - The object: its box.
/// * `k` - The object kind.
fn push_object(s: &mut Vec<Step>, inside: &mut HashSet<usize>, p: *const u8, k: &ZkKind)
{
    let base = zk_seq_base(p);
    ret_if!(base.is_null(), s.push(piece(prog().null_text)));
    ret_if!(!inside.insert(base as usize), s.push(Step::Piece("\u{2026}".as_bytes())));
    s.push(Step::Leave(base));
    s.push(Step::Value { a: base, b: std::ptr::null(), k: k.cell, nested: false });
}

/// Run one step of a print: write a piece, leave an object, or write a value,
/// at once or as further steps.
///
/// # Arguments
///
/// * `step` - The step.
/// * `s` - The steps, added to.
/// * `inside` - The objects the print is inside.
fn show_step(step: Step, s: &mut Vec<Step>, inside: &mut HashSet<usize>)
{
    let (p, k, nested) = match step
    {
        Step::Piece(t) => return emit(t),
        Step::Leave(base) =>
        {
            return {
                inside.remove(&(base as usize));
            };
        }
        Step::Value { a, k, nested, .. } => (a, kind(k), nested),
    };
    match k.tag
    {
        tag::ENUM => show_member(p, k),
        tag::STRUCT if is_table(k) => push_table(s, p, k),
        tag::STRUCT => push_struct(s, p, k),
        tag::FALLIBLE => push_fallible(s, p, k),
        tag::TEXT => show_text(p, nested),
        tag::SEQ if k.boxed != 0 => push_object(s, inside, p, k),
        tag::SEQ => push_cells(s, zk_seq_base(p), k.cell, zk_seq_len(p)),
        tag::LIST => push_cells(s, p, k.cell, k.len),
        _ => show_scalar(p, k),
    }
}

/// Write a value, as the VM prints it.
///
/// # Arguments
///
/// * `p` - The value.
/// * `k` - Its kind.
/// * `nested` - 1 to write text in quotes.
#[unsafe(export_name = stamped!("show"))]
pub extern "C" fn zk_show(p: *const u8, k: *const ZkKind, nested: i32)
{
    let mut steps =
        vec![Step::Value { a: p, b: std::ptr::null(), k, nested: nested != 0 }];
    let mut inside = HashSet::new();
    while let Some(step) = steps.pop()
    {
        show_step(step, &mut steps, &mut inside);
    }
}

// ---- tables ----
//
// A ဇယား is a struct of its rows, a plain list in add order, its slots, a
// plain list of u32 whose count is a power of two, and its count of live
// rows. A slot holds 0 when no row ever used it, 1 when its row was removed,
// and 2 more than the number of its row otherwise. An index goes to the slot
// its hash names, or the first one after it that holds no other row of the
// same index; at least a third of the slots hold 0, so each search ends. A
// removed row is dropped and left as zero octets until a rebuild moves the
// live rows together.

/// The first state of the 64-bit FNV-1a hash.
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
/// The multiplier of the 64-bit FNV-1a hash.
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
/// The fewest slots a ဇယား with rows holds.
const MIN_SLOTS: usize = 8;

/// Where the parts of a ဇယား and of its rows sit, and their kinds.
struct Tab<'a>
{
    /// The rows, the slots and the count of live rows, in the ဇယား.
    rows: *mut u8,
    slots: *mut u8,
    live: *mut u8,
    /// The kinds of the rows, the slots, a row, an index and a value.
    rows_k: &'a ZkKind,
    slots_k: &'a ZkKind,
    row: &'a ZkKind,
    index_k: &'a ZkKind,
    value_k: &'a ZkKind,
    /// The size of a row, and the offsets of its parts.
    stride: usize,
    hash: usize,
    alive: usize,
    index: usize,
    value: usize,
}

/// Whether a kind is a ဇယား.
///
/// # Arguments
///
/// * `k` - The kind.
///
/// # Returns
///
/// `true` for a ဇယား.
fn is_table(k: &ZkKind) -> bool { k.tag == tag::STRUCT && text(k.name) == b"%map" }

/// Where the parts of a ဇယား sit.
///
/// # Arguments
///
/// * `p` - The ဇယား.
/// * `k` - Its kind.
///
/// # Returns
///
/// The parts.
fn tab_of<'a>(p: *const u8, k: &'a ZkKind) -> Tab<'a>
{
    let f = k.field_list();
    let rows_k = kind(f[0].kind);
    let row = kind(rows_k.cell);
    let r = row.field_list();
    #[rustfmt::skip]
    return Tab { rows: at(p, f[0].offset), slots: at(p, f[1].offset), live: at(p, f[2].offset),
        rows_k, slots_k: kind(f[1].kind), row, index_k: kind(r[2].kind), value_k: kind(r[3].kind),
        stride: row.size, hash: r[0].offset, alive: r[1].offset, index: r[2].offset,
        value: r[3].offset };
}

/// Mix octets into a 64-bit FNV-1a hash.
///
/// # Arguments
///
/// * `h` - The hash so far.
/// * `s` - The octets.
///
/// # Returns
///
/// The hash.
fn mix(h: u64, s: &[u8]) -> u64
{
    s.iter().fold(h, |h, &c| (h ^ u64::from(c)).wrapping_mul(FNV_PRIME))
}

/// The octets of a value of a kind.
///
/// # Arguments
///
/// * `p` - The value.
/// * `size` - Its size.
///
/// # Returns
///
/// The octets.
fn raw<'a>(p: *const u8, size: usize) -> &'a [u8]
{
    ret_if!(size == 0, &[]);
    // SAFETY: a value of a kind holds `size` octets.
    unsafe { std::slice::from_raw_parts(p, size) }
}

/// How an index compares a decimal number: each NaN is one NaN, and -0.0 is
/// 0.0; each other value counts through its octets.
///
/// # Arguments
///
/// * `p` - The number.
/// * `size` - Its octets.
///
/// # Returns
///
/// 0 for a zero, 1 for a NaN, 2 for each other value.
fn float_class(p: *const u8, size: usize) -> u8
{
    let v = tern!(size == 4, f64::from(ld::<f32>(p)), ld::<f64>(p));
    tern!(v.is_nan(), 1, tern!(v == 0.0, 0, 2))
}

/// Mix an index into a hash: its parts in order, decimals folded, text and
/// plain lists through their content.
///
/// # Arguments
///
/// * `h` - The hash so far.
/// * `p` - The index.
/// * `k` - Its kind.
///
/// # Returns
///
/// The hash.
fn index_hash(h: u64, p: *const u8, k: &ZkKind) -> u64
{
    let k = repr(k);
    match k.tag
    {
        tag::TEXT => mix(mix(h, octets(p)), &[0xff]),
        tag::SEQ =>
        {
            let (n, base, cell) = (zk_seq_len(p), zk_seq_base(p), kind(k.cell));
            let h = mix(h, &n.to_ne_bytes());
            (0..n).fold(h, |h, i| index_hash(h, at(base, i * cell.size), cell))
        }
        tag::LIST =>
        {
            let cell = kind(k.cell);
            (0..k.len).fold(h, |h, i| index_hash(h, at(p, i * cell.size), cell))
        }
        tag::STRUCT | tag::FALLIBLE => k
            .field_list()
            .iter()
            .fold(h, |h, f| index_hash(h, at(p, f.offset), kind(f.kind))),
        tag::FLOAT =>
        {
            let c = float_class(p, k.size);
            tern!(c == 2, mix(h, raw(p, k.size)), mix(h, &[c]))
        }
        _ => mix(h, raw(p, k.size)),
    }
}

/// Whether two indexes are one: equal parts, each NaN equal to each other one,
/// and -0.0 equal to 0.0.
///
/// # Arguments
///
/// * `a` - The first index.
/// * `b` - The second index.
/// * `k` - The index kind.
///
/// # Returns
///
/// `true` when the two are one.
fn index_eq(a: *const u8, b: *const u8, k: &ZkKind) -> bool
{
    let k = repr(k);
    match k.tag
    {
        tag::TEXT => octets(a) == octets(b),
        tag::SEQ =>
        {
            ret_if!(zk_seq_len(a) != zk_seq_len(b), false);
            let (la, lb, cell) = (zk_seq_base(a), zk_seq_base(b), kind(k.cell));
            let s = cell.size;
            (0..zk_seq_len(a)).all(|i| index_eq(at(la, i * s), at(lb, i * s), cell))
        }
        tag::LIST =>
        {
            let cell = kind(k.cell);
            (0..k.len)
                .all(|i| index_eq(at(a, i * cell.size), at(b, i * cell.size), cell))
        }
        tag::STRUCT | tag::FALLIBLE => k
            .field_list()
            .iter()
            .all(|f| index_eq(at(a, f.offset), at(b, f.offset), kind(f.kind))),
        tag::FLOAT =>
        {
            let (ca, cb) = (float_class(a, k.size), float_class(b, k.size));
            ca == cb && (ca != 2 || raw(a, k.size) == raw(b, k.size))
        }
        _ => raw(a, k.size) == raw(b, k.size),
    }
}

/// The count of live rows of a ဇယား.
///
/// # Arguments
///
/// * `t` - The parts.
///
/// # Returns
///
/// The count.
fn tab_live(t: &Tab<'_>) -> usize { ld(t.live) }

/// Find the row of a ဇယား that holds an index.
///
/// # Arguments
///
/// * `t` - The parts.
/// * `index` - The index.
///
/// # Returns
///
/// The number of the row and of its slot, or `None` when no row holds it.
fn tab_find(t: &Tab<'_>, index: *const u8) -> Option<(usize, usize)>
{
    let count = zk_seq_len(t.slots);
    ret_if!(count == 0, None);
    let (slots, rows) = (zk_seq_base(t.slots), zk_seq_base(t.rows));
    let h = index_hash(FNV_OFFSET, index, t.index_k);
    let mask = count - 1;
    let mut slot = h as usize & mask;
    for _ in 0..count
    {
        let s: u32 = ld(at(slots, 4 * slot));
        ret_if!(s == 0, None);
        // A slot of 1 held a removed row: the search goes on past it.
        let n = (s as usize).saturating_sub(2);
        let row = at(rows, n * t.stride);
        let same = s > 1
            && ld::<u64>(at(row, t.hash)) == h
            && index_eq(at(row, t.index), index, t.index_k);
        ret_if!(same, Some((n, slot)));
        slot = (slot + 1) & mask;
    }
    None
}

/// Find the row of a ဇယား that holds an index.
///
/// # Arguments
///
/// * `p` - The ဇယား.
/// * `k` - Its kind.
/// * `index` - The index.
///
/// # Returns
///
/// The number of the row, or -1 when no row holds the index.
#[unsafe(export_name = stamped!("row_find"))]
pub extern "C" fn zk_row_find(p: *const u8, k: *const ZkKind, index: *const u8) -> i64
{
    tab_find(&tab_of(p, kind(k)), index).map_or(-1, |(row, _)| row as i64)
}

/// Point a free slot at a row: the slot its hash names, or the first free one
/// after it.
///
/// # Arguments
///
/// * `t` - The parts.
/// * `h` - The hash of the index of the row.
/// * `row` - The number of the row.
fn tab_place(t: &Tab<'_>, h: u64, row: usize)
{
    let (count, slots) = (zk_seq_len(t.slots), zk_seq_base(t.slots));
    let mask = count - 1;
    let mut slot = h as usize & mask;
    while ld::<u32>(at(slots, 4 * slot)) > 1
    {
        slot = (slot + 1) & mask;
    }
    st(at(slots, 4 * slot), row as u32 + 2);
}

/// Move the live rows of a ဇယား together, in their order, and give it a new
/// block of slots, each pointing at its row again.
///
/// # Arguments
///
/// * `t` - The parts; the ဇယား owns its blocks.
/// * `count` - The new count of slots, a power of two.
fn tab_rebuild(t: &Tab<'_>, count: usize)
{
    let rows = zk_seq_base(t.rows);
    let mut kept = 0;
    for n in 0..zk_seq_len(t.rows)
    {
        let row = at(rows, n * t.stride);
        do_if!(ld::<u8>(at(row, t.alive)) == 0, continue);
        if kept != n
        {
            move_octets(at(rows, kept * t.stride), row, t.stride);
            zero(row, t.stride);
        }
        kept += 1;
    }
    zk_seq_put(t.rows, rows, kept, seq_room(t.rows));
    zk_drop(t.slots, t.slots_k);
    let block = zk_seq_new(4 * count);
    seq_kind(block, t.slots_k.cell);
    zero(block, 4 * count);
    zk_seq_put(t.slots, block, count, 4 * count);
    do_for!(n in 0..kept, tab_place(t, ld(at(rows, n * t.stride + t.hash)), n));
}

/// Add a new last row to a ဇယား, moving its index and its value in.
///
/// # Arguments
///
/// * `t` - The parts; the ဇယား owns its blocks.
/// * `h` - The hash of the index.
/// * `index` - The index, moved out and zeroed.
/// * `src` - The value, moved out and zeroed.
///
/// # Returns
///
/// The number of the new row.
fn tab_push(t: &Tab<'_>, h: u64, index: *mut u8, src: *mut u8) -> usize
{
    let (mut block, count, held) =
        (zk_seq_base(t.rows), zk_seq_len(t.rows), seq_room(t.rows));
    let mut room = held;
    let need = (count + 1) * t.stride;
    if need > held
    {
        room = need.max(2 * held).max(4 * t.stride);
        block = seq_grow(tern!(held == 0, std::ptr::null_mut(), block), room);
        seq_kind(block, t.row);
    }
    let row = at(block, count * t.stride);
    zero(row, t.stride);
    st(at(row, t.hash), h);
    st(at(row, t.alive), 1u8);
    move_octets(at(row, t.index), index, t.index_k.size);
    zero(index, t.index_k.size);
    move_octets(at(row, t.value), src, t.value_k.size);
    zero(src, t.value_k.size);
    zk_seq_put(t.rows, block, count + 1, room);
    count
}

/// Set the value of an index of a ဇယား: the row that holds the index takes
/// the value, else a new row does.
///
/// # Arguments
///
/// * `p` - The ဇယား.
/// * `k` - Its kind.
/// * `index` - The index, moved out and zeroed.
/// * `src` - The value, moved out and zeroed.
#[unsafe(export_name = stamped!("set_row"))]
pub extern "C" fn zk_set_row(p: *mut u8, k: *const ZkKind, index: *mut u8, src: *mut u8)
{
    let t = tab_of(p, kind(k));
    zk_own(t.rows, t.rows_k);
    zk_own(t.slots, t.slots_k);
    if let Some((found, _)) = tab_find(&t, index)
    {
        let value = at(zk_seq_base(t.rows), found * t.stride + t.value);
        zk_drop(value, t.value_k);
        move_octets(value, src, t.value_k.size);
        zero(src, t.value_k.size);
        return zk_drop(index, t.index_k);
    }
    let h = index_hash(FNV_OFFSET, index, t.index_k);
    let live = tab_live(&t);
    let mut want = MIN_SLOTS;
    while want < 2 * (live + 1)
    {
        want *= 2;
    }
    // Each row, live or removed, holds a slot until a rebuild, and a third of
    // the slots remain free.
    do_if!(
        (zk_seq_len(t.rows) + 1) * 3 > zk_seq_len(t.slots) * 2,
        tab_rebuild(&t, want)
    );
    tab_place(&t, h, tab_push(&t, h, index, src));
    st(t.live, live + 1);
}

/// Remove the row of an index from a ဇယား.
///
/// # Arguments
///
/// * `p` - The ဇယား.
/// * `k` - Its kind.
/// * `index` - The index.
#[unsafe(export_name = stamped!("remove_row"))]
pub extern "C" fn zk_remove_row(p: *mut u8, k: *const ZkKind, index: *const u8)
{
    let t = tab_of(p, kind(k));
    let Some((found, slot)) = tab_find(&t, index)
    else
    {
        return;
    };
    zk_own(t.rows, t.rows_k);
    zk_own(t.slots, t.slots_k);
    let row = at(zk_seq_base(t.rows), found * t.stride);
    zk_drop(row, t.row);
    zero(row, t.stride);
    st(at(zk_seq_base(t.slots), 4 * slot), 1u32);
    let live = tab_live(&t) - 1;
    // A ဇယား left with no live row gives back its blocks.
    ret_if!(live == 0, zk_drop(p, k));
    st(t.live, live);
}

/// Compare two ဇယား values: equal counts of live rows, and for each live row
/// of the first, a row of the second with one index, whose values the steps
/// compare.
///
/// # Arguments
///
/// * `a` - The first.
/// * `b` - The second.
/// * `k` - The ဇယား kind.
/// * `steps` - The steps, added to.
///
/// # Returns
///
/// `false` when the two differ so far.
fn table_eq_step(a: *const u8, b: *const u8, k: &ZkKind, steps: &mut Vec<Step>)
-> bool
{
    let (ta, tb) = (tab_of(a, k), tab_of(b, k));
    ret_if!(tab_live(&ta) != tab_live(&tb), false);
    let (rows, other) = (zk_seq_base(ta.rows), zk_seq_base(tb.rows));
    for n in 0..zk_seq_len(ta.rows)
    {
        let row = at(rows, n * ta.stride);
        do_if!(ld::<u8>(at(row, ta.alive)) == 0, continue);
        let Some((found, _)) = tab_find(&tb, at(row, ta.index))
        else
        {
            return false;
        };
        let there = at(other, found * tb.stride + tb.value);
        steps.push(Step::Value {
            a: at(row, ta.value),
            b: there,
            k: ta.value_k,
            nested: false,
        });
    }
    true
}

/// Add the steps that write a ဇယား, the last first: each live row as
/// `index: value`, in add order, joined with the separator between `[` and
/// `]`; `[:]` for one with no rows.
///
/// # Arguments
///
/// * `s` - The steps.
/// * `p` - The ဇယား.
/// * `k` - Its kind.
fn push_table(s: &mut Vec<Step>, p: *const u8, k: &ZkKind)
{
    let t = tab_of(p, k);
    ret_if!(tab_live(&t) == 0, s.push(Step::Piece(b"[:]")));
    let rows = zk_seq_base(t.rows);
    let mut later = false;
    s.push(Step::Piece(b"]"));
    for n in (0..zk_seq_len(t.rows)).rev()
    {
        let row = at(rows, n * t.stride);
        do_if!(ld::<u8>(at(row, t.alive)) == 0, continue);
        do_if!(later, s.push(Step::Piece(SEP)));
        s.push(Step::Value {
            a: at(row, t.value),
            b: std::ptr::null(),
            k: t.value_k,
            nested: true,
        });
        s.push(Step::Piece(b": "));
        s.push(Step::Value {
            a: at(row, t.index),
            b: std::ptr::null(),
            k: t.index_k,
            nested: true,
        });
        later = true;
    }
    s.push(Step::Piece(b"["));
}

// ---- results ----

/// Stop the run: a result that failed was read as good; the stop names its
/// error value.
///
/// # Arguments
///
/// * `p` - The result.
/// * `k` - Its kind.
#[unsafe(export_name = stamped!("not_good"))]
pub extern "C" fn zk_not_good(p: *const u8, k: *const ZkKind) -> !
{
    let value = shown(p, repr(kind(k)) as *const ZkKind);
    stop_owned(trap::NOT_GOOD, vec![("e".to_string(), value)])
}

/// Stop the run: a result with no value that failed was read as good.
#[unsafe(export_name = stamped!("not_good_bare"))]
pub extern "C" fn zk_not_good_bare() -> ! { zk_plain_trap(trap::NOT_GOOD_BARE) }

/// Stop the run: a good result was read as failed.
#[unsafe(export_name = stamped!("not_fault"))]
pub extern "C" fn zk_not_fault() -> ! { zk_plain_trap(trap::NOT_FAULT) }

/// Stop the run: an optional that holds no data was read.
#[unsafe(export_name = stamped!("no_data"))]
pub extern "C" fn zk_no_data() -> ! { zk_plain_trap(trap::NO_DATA) }

/// Stop the run: a handle a C function closed went to C again.
#[unsafe(export_name = stamped!("closed_handle"))]
pub extern "C" fn zk_closed_handle() -> ! { zk_plain_trap(trap::CLOSED_HANDLE) }
