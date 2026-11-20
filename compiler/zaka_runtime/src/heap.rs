//! Storage: the heap blocks of the program under the cap of `--max-heap`,
//! the counted blocks of text, plain lists, boxes and closure records, raw
//! blocks of unsafe code, and function values.

use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::atomic::fence;

use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::program::NULL_BIT;
use crate::program::OVERLAP_BIT;
use crate::program::ZkKind;
use crate::program::check_on;
use crate::program::prog;
use crate::seqs::dispose;
use crate::seqs::note_candidate;
use crate::seqs::zk_drop;
use crate::traps::trap;
use crate::traps::zk_addr_trap;
use crate::traps::zk_null_deref;
use crate::traps::zk_out_of_mem;
use crate::traps::zk_plain_trap;

unsafe extern "C" {
    #[link_name = "malloc"]
    fn c_malloc(size: usize) -> *mut c_void;
    #[link_name = "calloc"]
    fn c_calloc(count: usize, size: usize) -> *mut c_void;
    pub fn realloc(p: *mut c_void, size: usize) -> *mut c_void;
    pub fn free(p: *mut c_void);
    fn atexit(f: extern "C" fn()) -> i32;
    #[cfg(target_os = "macos")]
    fn malloc_size(p: *const c_void) -> usize;
    #[cfg(any(target_os = "linux", target_os = "wasi"))]
    fn malloc_usable_size(p: *mut c_void) -> usize;
    #[cfg(windows)]
    fn _msize(p: *mut c_void) -> usize;
}

/// A block of the C allocator, or NULL when no room is left.
///
/// # Arguments
///
/// * `size` - The octets.
///
/// # Returns
///
/// The block.
pub fn malloc(size: usize) -> *mut c_void
{
    // SAFETY: malloc takes each size and gives NULL for one it cannot hold.
    unsafe { c_malloc(size) }
}

/// A block of the C allocator with each octet 0, or NULL when no room is left.
///
/// # Arguments
///
/// * `count` - The count of cells.
/// * `size` - The octets of a cell.
///
/// # Returns
///
/// The block.
pub fn calloc(count: usize, size: usize) -> *mut c_void
{
    // SAFETY: calloc checks the product and gives NULL for one it cannot hold.
    unsafe { c_calloc(count, size) }
}

/// The octets of a word: an address or a count.
pub const WORD: usize = size_of::<usize>();

/// Read a value of a kind from storage, at an address with no alignment.
///
/// # Arguments
///
/// * `at` - The address.
///
/// # Returns
///
/// The value.
pub fn ld<T: Copy>(at: *const u8) -> T
{
    // SAFETY: the generated code and the kind tables hand addresses of values.
    unsafe { at.cast::<T>().read_unaligned() }
}

/// Write a value of a kind into storage, at an address with no alignment.
///
/// # Arguments
///
/// * `at` - The address.
/// * `v` - The value.
pub fn st<T: Copy>(at: *mut u8, v: T)
{
    // SAFETY: the generated code and the kind tables hand addresses of values.
    unsafe { at.cast::<T>().write_unaligned(v) }
}

/// Move octets; the two runs can overlap.
///
/// # Arguments
///
/// * `to` - The first octet written.
/// * `from` - The first octet read.
/// * `n` - The count.
pub fn move_octets(to: *mut u8, from: *const u8, n: usize)
{
    ret_if!(n == 0);
    // SAFETY: the caller hands two runs of `n` octets.
    unsafe { std::ptr::copy(from, to, n) }
}

/// Write zero octets.
///
/// # Arguments
///
/// * `at` - The first octet.
/// * `n` - The count.
pub fn zero(at: *mut u8, n: usize)
{
    ret_if!(n == 0);
    // SAFETY: the caller hands a run of `n` octets.
    unsafe { std::ptr::write_bytes(at, 0, n) }
}

/// The heap octets the program holds now, which `--max-heap` caps.
static HEAP_LIVE: AtomicUsize = AtomicUsize::new(0);

/// Count octets the program takes from the heap, under the cap of
/// `--max-heap`; with no cap each take fits.
///
/// # Arguments
///
/// * `size` - The octets.
///
/// # Returns
///
/// `true` when the octets fit and are counted.
pub fn heap_take(size: usize) -> bool { take_into(heap_owner(), size) }

/// The count a new block counts in: the count of the C lib whose call runs,
/// each lib under its own `--max-heap`, or else the count of the program.
///
/// # Returns
///
/// The count, as an address; 0 when no cap counts.
pub fn heap_owner() -> usize
{
    ret_if!(prog().max_heap == 0, 0);
    let count = crate::libs::current().map_or(&HEAP_LIVE, |l| &l.heap);
    (&raw const *count) as usize
}

/// Count octets in a count, under the cap of `--max-heap`.
///
/// # Arguments
///
/// * `owner` - The count, as [`heap_owner`] gives it; 0 counts nothing.
/// * `size` - The octets.
///
/// # Returns
///
/// `true` when the octets fit and are counted.
pub fn take_into(owner: usize, size: usize) -> bool
{
    ret_if!(owner == 0, true);
    let cap = prog().max_heap;
    // SAFETY: a count lives as long as the program.
    let count = unsafe { &*(owner as *const AtomicUsize) };
    let fits = |live: usize| {
        let fit = size as u64 <= cap && live as u64 <= cap - size as u64;
        fit.then(|| live + size)
    };
    count.try_update(Ordering::Relaxed, Ordering::Relaxed, fits).is_ok()
}

/// Give octets back to the count that held them.
///
/// # Arguments
///
/// * `owner` - The count, as [`heap_owner`] gave it; 0 counts nothing.
/// * `size` - The octets.
pub fn give_back(owner: usize, size: usize)
{
    ret_if!(owner == 0);
    // SAFETY: a count lives as long as the program.
    let count = unsafe { &*(owner as *const AtomicUsize) };
    let less = |live: usize| Some(live.saturating_sub(size));
    _ = count.try_update(Ordering::Relaxed, Ordering::Relaxed, less);
}

/// The raw blocks of unsafe code that count in the count of a C lib, each with
/// that count; a raw block of the program counts in the count of the program.
static RAW_OWNERS: Mutex<Vec<(usize, usize)>> = Mutex::new(Vec::new());

/// The count a raw block counts in, taken off the list of raw blocks.
///
/// # Arguments
///
/// * `p` - The block.
///
/// # Returns
///
/// The count, as an address; 0 when no cap counts.
fn raw_owner(p: *const u8) -> usize
{
    ret_if!(prog().max_heap == 0, 0);
    let mut owners = RAW_OWNERS.lock().unwrap_or_else(|e| e.into_inner());
    let at = owners.iter().position(|(b, _)| *b == p as usize);
    at.map_or((&raw const HEAP_LIVE) as usize, |i| owners.swap_remove(i).1)
}

/// Note the count a raw block counts in, when it is the count of a C lib.
///
/// # Arguments
///
/// * `p` - The block.
/// * `owner` - The count.
fn note_raw(p: *const u8, owner: usize)
{
    ret_if!(owner == 0 || owner == (&raw const HEAP_LIVE) as usize);
    RAW_OWNERS.lock().unwrap_or_else(|e| e.into_inner()).push((p as usize, owner));
}

/// The size the allocator gave a raw block, so `--max-heap` can give it back
/// when the block is freed; with no cap, or on a host with no such call, 0.
///
/// # Arguments
///
/// * `p` - The block.
///
/// # Returns
///
/// The size.
fn raw_size(p: *mut u8) -> usize
{
    ret_if!(prog().max_heap == 0 || p.is_null(), 0);
    #[cfg(target_os = "macos")]
    // SAFETY: `p` came from the C allocator.
    return unsafe { malloc_size(p.cast()) };
    #[cfg(any(target_os = "linux", target_os = "wasi"))]
    // SAFETY: `p` came from the C allocator.
    return unsafe { malloc_usable_size(p.cast()) };
    #[cfg(windows)]
    // SAFETY: `p` came from the C allocator.
    return unsafe { _msize(p.cast()) };
    #[allow(unreachable_code)]
    0
}

/// A new heap block, or a stop of the run when no room is left.
///
/// # Arguments
///
/// * `size` - The size in octets.
///
/// # Returns
///
/// The block.
pub fn block(size: usize) -> *mut u8
{
    let p = malloc(size.max(1)).cast::<u8>();
    do_if!(p.is_null(), zk_out_of_mem(size as u64));
    p
}

/// A block moved to a new size, or a stop of the run when no room is left.
///
/// # Arguments
///
/// * `p` - The block, or NULL for a new one.
/// * `size` - The new size in octets.
///
/// # Returns
///
/// The block.
fn grow(p: *mut u8, size: usize) -> *mut u8
{
    // SAFETY: `p` is NULL or a block of the C allocator.
    let moved = unsafe { realloc(p.cast(), size.max(1)) }.cast::<u8>();
    do_if!(moved.is_null(), zk_out_of_mem(size as u64));
    moved
}

/// The head in front of the cells of a counted block: text, a plain list, a
/// box or the record of a closure.
#[repr(C)]
pub struct Head
{
    /// The count of values that hold the block; the last to let go frees it.
    pub rc: usize,
    /// The octets of its cells.
    pub size: usize,
    /// The kind of each cell, so the ring collector can walk what the block
    /// holds; NULL for a block with nothing to walk.
    pub cell: *const ZkKind,
    /// The kind of each cell to drop as the block is freed, or NULL.
    pub drop_cell: *const ZkKind,
    /// The cells to drop as the block is freed.
    pub len: usize,
    /// The next block in the queue of blocks to free; the deep duplicate
    /// borrows it for the new block of a live one.
    pub next: *mut u8,
    /// Where the ring collector stands with the block: 0 black, 1 pending,
    /// 2 white.
    pub color: u8,
    /// Whether the block waits in the buffer of the ring collector.
    pub buffered: u8,
    /// Whether an address of one of its cells was taken: the block is never
    /// shared again, so a duplicate copies it.
    pub pinned: u8,
    /// The count of heap octets the block counts in, as an address: the count
    /// of the C lib whose call made it, or of the program; 0 when no
    /// `--max-heap` caps it.
    pub owner: usize,
}

/// The octets in front of the cells of a counted block: the head, padded so
/// the cells keep the alignment malloc gives.
const SEQ_HEAD: usize = size_of::<Head>().div_ceil(16) * 16;

/// The head of a counted block.
///
/// # Arguments
///
/// * `base` - The first cell of the block.
///
/// # Returns
///
/// The head, in front of the cells.
pub fn head<'a>(base: *const u8) -> &'a mut Head
{
    // SAFETY: each counted block starts `SEQ_HEAD` octets before its cells.
    unsafe { &mut *base.cast_mut().sub(SEQ_HEAD).cast::<Head>() }
}

/// Whether the program has started a second thread: from then on each count
/// changes with atomic ops. The start of a thread orders the set before each
/// op of the new thread, so the switch needs no more than that.
static THREADED: AtomicBool = AtomicBool::new(false);

/// Make each later count change atomic: the start of the first thread of the
/// program calls this before the thread runs.
#[unsafe(export_name = stamped!("threads_begin"))]
pub extern "C" fn zk_threads_begin() { THREADED.store(true, Ordering::SeqCst) }

/// Whether the counts of the heap change through atomic ops.
///
/// # Returns
///
/// `true` once [`zk_threads_begin`] ran.
pub(crate) fn threaded() -> bool { THREADED.load(Ordering::SeqCst) }

/// The count of a block as an atomic, for the ops of a program with threads.
///
/// # Arguments
///
/// * `base` - The first cell of the block.
///
/// # Returns
///
/// The count.
fn atomic_rc<'a>(base: *const u8) -> &'a AtomicUsize
{
    // SAFETY: each counted block starts `SEQ_HEAD` octets before its cells,
    // and its count is a word that each thread changes with atomic ops alone
    // once threads start.
    unsafe {
        let h = base.cast_mut().sub(SEQ_HEAD).cast::<Head>();
        AtomicUsize::from_ptr(std::ptr::addr_of_mut!((*h).rc))
    }
}

/// Add a holder to a counted block.
///
/// # Arguments
///
/// * `base` - The first cell of the block.
pub fn rc_add(base: *const u8)
{
    ret_if!(
        THREADED.load(Ordering::Relaxed),
        _ = atomic_rc(base).fetch_add(1, Ordering::Relaxed)
    );
    head(base).rc += 1;
}

/// Take a holder off a counted block.
///
/// # Arguments
///
/// * `base` - The first cell of the block.
///
/// # Returns
///
/// The holders left; at 0 the caller frees the block, and each write of the
/// other holders is seen.
pub fn rc_sub(base: *const u8) -> usize
{
    if THREADED.load(Ordering::Relaxed)
    {
        let left = atomic_rc(base).fetch_sub(1, Ordering::Release) - 1;
        do_if!(left == 0, fence(Ordering::Acquire));
        return left;
    }
    let h = head(base);
    h.rc -= 1;
    h.rc
}

/// The holders of a counted block now.
///
/// # Arguments
///
/// * `base` - The first cell of the block.
///
/// # Returns
///
/// The count.
pub fn rc_of(base: *const u8) -> usize
{
    ret_if!(THREADED.load(Ordering::Relaxed), atomic_rc(base).load(Ordering::Acquire));
    head(base).rc
}

/// A new counted block of text or a plain list, with one holder.
///
/// # Arguments
///
/// * `size` - The octets of its cells.
///
/// # Returns
///
/// The first cell; the run stops when no room is left.
#[unsafe(export_name = stamped!("seq_new"))]
pub extern "C" fn zk_seq_new(size: usize) -> *mut u8
{
    let owner = heap_owner();
    do_if!(!take_into(owner, size), zk_out_of_mem(size as u64));
    let p = malloc(SEQ_HEAD + size).cast::<u8>();
    do_if!(p.is_null(), zk_out_of_mem(size as u64));
    // SAFETY: the block holds a head, then the cells.
    let base = unsafe { p.add(SEQ_HEAD) };
    #[rustfmt::skip]
    let fresh = Head { rc: 1, size, cell: std::ptr::null(), drop_cell: std::ptr::null(), len: 0,
        next: std::ptr::null_mut(), color: 0, buffered: 0, pinned: 0, owner };
    // SAFETY: the head lies in front of the cells.
    unsafe { p.cast::<Head>().write(fresh) };
    base
}

/// A counted block moved to a new size; its count and kind remain.
///
/// # Arguments
///
/// * `base` - The first cell, or NULL for a new block.
/// * `size` - The new octets of its cells.
///
/// # Returns
///
/// The first cell; the run stops when no room is left.
pub fn seq_grow(base: *mut u8, size: usize) -> *mut u8
{
    ret_if!(base.is_null(), zk_seq_new(size));
    let (old, owner) = (head(base).size, head(base).owner);
    do_if!(size > old && !take_into(owner, size - old), zk_out_of_mem(size as u64));
    do_if!(size < old, give_back(owner, old - size));
    // SAFETY: the block came from `zk_seq_new`.
    let p = unsafe { realloc(base.sub(SEQ_HEAD).cast(), SEQ_HEAD + size) }.cast::<u8>();
    do_if!(p.is_null(), zk_out_of_mem(size as u64));
    // SAFETY: the moved block holds its head, then the cells.
    let moved = unsafe { p.add(SEQ_HEAD) };
    head(moved).size = size;
    moved
}

/// Record the kind of each cell of a counted block, so the ring collector can
/// walk what it holds.
///
/// # Arguments
///
/// * `base` - The first cell of the block.
/// * `cell` - The kind of each cell.
pub fn seq_kind(base: *mut u8, cell: *const ZkKind) { head(base).cell = cell }

/// Free a counted block.
///
/// # Arguments
///
/// * `base` - The first cell, or NULL.
#[unsafe(export_name = stamped!("seq_free"))]
pub extern "C" fn zk_seq_free(base: *mut u8)
{
    ret_if!(base.is_null());
    give_back(head(base).owner, head(base).size);
    // SAFETY: the block came from `zk_seq_new`.
    unsafe { free(base.sub(SEQ_HEAD).cast()) }
}

/// Stop the run at an access through the null address, when the mem checks
/// are on.
///
/// # Arguments
///
/// * `p` - The address.
pub fn null_check(p: *const u8)
{
    do_if!(p.is_null() && check_on(NULL_BIT), zk_null_deref(p.cast()));
}

/// Write a value of a list or struct kind.
///
/// # Arguments
///
/// * `to` - The address written.
/// * `from` - The address read; the two can overlap.
/// * `size` - The size in octets.
#[unsafe(export_name = stamped!("put_block"))]
pub extern "C" fn zk_put_block(to: *mut u8, from: *const u8, size: usize)
{
    ret_if!(size == 0 || std::ptr::eq(to, from));
    null_check(to);
    null_check(from);
    move_octets(to, from, size)
}

/// An address moved `count` steps of `stride` octets; the sum wraps, and a
/// step from null stops the run under the mem checks.
///
/// # Arguments
///
/// * `p` - The address.
/// * `count` - The steps, of each sign.
/// * `stride` - The octets of a step.
///
/// # Returns
///
/// The address.
#[unsafe(export_name = stamped!("ptr_add"))]
pub extern "C" fn zk_ptr_add(p: *mut u8, count: i64, stride: usize) -> *mut u8
{
    do_if!(check_on(NULL_BIT) && p.is_null() && count != 0, zk_null_deref(p.cast()));
    let step = (count as u64).wrapping_mul(stride as u64) as usize;
    (p as usize).wrapping_add(step) as *mut u8
}

/// The steps of `stride` octets from `q` to `p`.
///
/// # Arguments
///
/// * `p` - The later address.
/// * `q` - The earlier address.
/// * `stride` - The octets of a step.
///
/// # Returns
///
/// The steps.
#[unsafe(export_name = stamped!("ptr_sub"))]
pub extern "C" fn zk_ptr_sub(p: *const u8, q: *const u8, stride: usize) -> i64
{
    let distance = (p as usize).wrapping_sub(q as usize) as isize as i64;
    distance / stride as i64
}

/// A new block of `count` cells, zeroed as the VM zeroes it; a block of 0
/// octets is a real block.
///
/// # Arguments
///
/// * `count` - The cell count.
/// * `stride` - The octets of a cell.
///
/// # Returns
///
/// The block, or NULL when the size overflows or no room is left.
#[unsafe(export_name = stamped!("alloc"))]
pub extern "C" fn zk_alloc(count: u64, stride: usize) -> *mut u8
{
    let Some(size) = usize::try_from(count).ok().and_then(|c| c.checked_mul(stride))
    else
    {
        return std::ptr::null_mut();
    };
    let p = calloc(size.max(1), 1).cast::<u8>();
    ret_if!(p.is_null(), p);
    let owner = heap_owner();
    let fits = take_into(owner, raw_size(p));
    do_if!(fits, note_raw(p, owner));
    ret_if!(fits, p);
    // Past the cap of --max-heap the program gets no block, as from malloc.
    // SAFETY: `p` came from calloc.
    unsafe { free(p.cast()) };
    std::ptr::null_mut()
}

/// Free a raw block that unsafe code made and give back its octets.
///
/// # Arguments
///
/// * `p` - The block, or NULL.
#[unsafe(export_name = stamped!("raw_free"))]
pub extern "C" fn zk_raw_free(p: *mut c_void)
{
    ret_if!(p.is_null());
    give_back(raw_owner(p.cast()), raw_size(p.cast()));
    // SAFETY: `p` came from the C allocator.
    unsafe { free(p) }
}

/// A block moved to a new size of `count` cells.
///
/// # Arguments
///
/// * `p` - The block, or NULL for a new one.
/// * `count` - The cell count.
/// * `stride` - The octets of a cell.
///
/// # Returns
///
/// The block; the run stops when the size overflows or no room is left.
#[unsafe(export_name = stamped!("realloc"))]
pub extern "C" fn zk_realloc(p: *mut u8, count: u64, stride: usize) -> *mut u8
{
    let Some(size) = usize::try_from(count).ok().and_then(|c| c.checked_mul(stride))
    else
    {
        zk_out_of_mem(u64::MAX)
    };
    let old = raw_size(p);
    let owner = tern!(p.is_null(), heap_owner(), raw_owner(p));
    let moved = grow(p, size);
    let now = raw_size(moved);
    do_if!(now > old && !take_into(owner, now - old), zk_out_of_mem(now as u64));
    do_if!(now < old, give_back(owner, old - now));
    note_raw(moved, owner);
    moved
}

/// Move `count` cells of `stride` octets.
///
/// # Arguments
///
/// * `src` - The first cell read.
/// * `count` - The cell count.
/// * `dst` - The first cell written.
/// * `stride` - The octets of a cell.
/// * `overlap` - 1 when the runs can overlap; with 0 the mem checks stop the
///   run when the two do.
#[unsafe(export_name = stamped!("blit"))]
pub extern "C" fn zk_blit(
    src: *const u8, count: u64, dst: *mut u8, stride: usize, overlap: i32,
)
{
    let len = (count as usize).wrapping_mul(stride);
    ret_if!(len == 0);
    let (s, d) = (src as usize, dst as usize);
    let apart = s.wrapping_add(len) <= d || d.wrapping_add(len) <= s;
    do_if!(
        check_on(OVERLAP_BIT) && overlap == 0 && !apart,
        zk_addr_trap(trap::OVERLAP, dst.cast())
    );
    null_check(src);
    null_check(dst);
    move_octets(dst, src, len)
}

/// Write one cell into each of `count` cells.
///
/// # Arguments
///
/// * `dst` - The first cell.
/// * `count` - The cell count.
/// * `cell` - The value of a cell.
/// * `size` - The octets of a cell.
#[unsafe(export_name = stamped!("fill"))]
pub extern "C" fn zk_fill(dst: *mut u8, count: u64, cell: *const u8, size: usize)
{
    do_if!(count > 0, null_check(dst));
    for i in 0..count as usize
    {
        // SAFETY: the run holds `count` cells.
        move_octets(unsafe { dst.add(i * size) }, cell, size);
    }
}

/// Run each collection that waits, as the program ends; the C lib calls
/// it.
pub fn at_exit(f: extern "C" fn()) { unsafe { atexit(f) }; }

// ---- function values ----

/// The code of a function of the program, as C calls it: the first word of a
/// record.
pub type ZkCode = Option<unsafe extern "C" fn()>;

/// A function value: the address of a record whose first word is code. A
/// closure lent to a call has the lowest bit of its address set.
pub type ZkFn = *const c_void;

/// The record of a function of the program: its code, then NULL where a
/// closure record holds its kind.
#[repr(C)]
pub struct ZkFnRec
{
    /// The code.
    pub code: ZkCode,
    /// NULL, which tells a function from a closure.
    pub none: *const c_void,
}

/// The function a function value holds.
///
/// # Arguments
///
/// * `f` - The value.
///
/// # Returns
///
/// The value; the run stops when it is null.
#[unsafe(export_name = stamped!("fn_check"))]
pub extern "C" fn zk_fn_check(f: ZkFn) -> ZkFn
{
    do_if!(f.is_null(), zk_plain_trap(trap::NULL_CALL));
    f
}

/// The record of a function value: its address without the mark of a loan.
///
/// # Arguments
///
/// * `f` - The value.
///
/// # Returns
///
/// The record.
#[unsafe(export_name = stamped!("fn_rec"))]
pub extern "C" fn zk_fn_rec(f: ZkFn) -> *mut u8 { (f as usize & !1) as *mut u8 }

/// The code a function value runs: the first word of its record.
///
/// # Arguments
///
/// * `f` - The value, not null.
///
/// # Returns
///
/// The code.
#[unsafe(export_name = stamped!("fn_code"))]
pub extern "C" fn zk_fn_code(f: ZkFn) -> ZkCode { ld(zk_fn_rec(f)) }

/// The kind of the record of a closure: the second word of the record.
///
/// # Arguments
///
/// * `f` - The value, not null.
///
/// # Returns
///
/// The kind, or NULL for a function.
fn fn_kind(f: ZkFn) -> *const ZkKind
{
    // SAFETY: a record holds two words at least.
    ld(unsafe { zk_fn_rec(f).add(WORD) })
}

/// Whether a function value is a closure: the second word of its record is
/// the kind of the record, where the record of a function holds NULL.
///
/// # Arguments
///
/// * `f` - The value, not null.
///
/// # Returns
///
/// 1 for a closure, else 0.
#[unsafe(export_name = stamped!("fn_bound"))]
pub extern "C" fn zk_fn_bound(f: ZkFn) -> i32 { i32::from(!fn_kind(f).is_null()) }

/// Whether a function value holds a count of a closure record: a closure that
/// is not lent.
///
/// # Arguments
///
/// * `f` - The value.
///
/// # Returns
///
/// `true` when it does.
pub fn holds_record(f: ZkFn) -> bool
{
    !f.is_null() && f as usize & 1 == 0 && zk_fn_bound(f) != 0
}

/// Make a closure: move its record from a slot of the caller to the heap,
/// leaving zeros behind, with the code of its thunk in the first word and the
/// kind of the record in the second.
///
/// # Arguments
///
/// * `record` - The record, in a slot of the caller.
/// * `thunk` - The code of the thunk.
/// * `k` - The kind of the record.
///
/// # Returns
///
/// The closure.
#[unsafe(export_name = stamped!("closure"))]
pub extern "C" fn zk_closure(record: *mut u8, thunk: ZkCode, k: *const ZkKind) -> ZkFn
{
    // SAFETY: the generated code hands a descriptor.
    let size = unsafe { (*k).size };
    let block = zk_seq_new(size);
    seq_kind(block, k);
    move_octets(block, record, size);
    zero(record, size);
    st(block, thunk);
    // SAFETY: a record holds two words at least.
    st(unsafe { block.add(WORD) }, k);
    block.cast_const().cast()
}

/// Check a function value headed to C in a program with no runtime of
/// closures for C, or of a kind libffi cannot pass: C calls a bare code
/// address, so a closure stops the run.
///
/// # Arguments
///
/// * `f` - The value.
///
/// # Returns
///
/// The value.
#[unsafe(export_name = stamped!("fn_c"))]
pub extern "C" fn zk_fn_c(f: ZkFn) -> ZkFn
{
    do_if!(!f.is_null() && zk_fn_bound(f) != 0, zk_plain_trap(trap::CLOSURE_ESCAPE));
    f
}

/// A duplicate of a function value: a closure counts one more holder of its
/// record, and a duplicate of a lent closure is no longer lent.
///
/// # Arguments
///
/// * `f` - The value.
///
/// # Returns
///
/// The duplicate.
#[unsafe(export_name = stamped!("fn_dup"))]
pub extern "C" fn zk_fn_dup(f: ZkFn) -> ZkFn
{
    ret_if!(f.is_null() || zk_fn_bound(f) == 0, f);
    let rec = zk_fn_rec(f);
    rc_add(rec);
    rec.cast_const().cast()
}

/// A function value lent to a call: a closure gets the mark of a loan, so a
/// drop lets go of nothing.
///
/// # Arguments
///
/// * `f` - The value.
///
/// # Returns
///
/// The lent value.
#[unsafe(export_name = stamped!("fn_lend"))]
pub extern "C" fn zk_fn_lend(f: ZkFn) -> ZkFn
{
    ret_if!(f.is_null() || zk_fn_bound(f) == 0, f);
    (f as usize | 1) as ZkFn
}

/// Let go of the record of the closure a slot holds, and leave NULL: the last
/// holder drops its fields and frees it.
///
/// # Arguments
///
/// * `at` - The slot.
pub fn drop_fn(at: *mut u8)
{
    let f: ZkFn = ld(at);
    let owned = holds_record(f);
    st(at, std::ptr::null::<c_void>());
    ret_if!(!owned);
    let rec = zk_fn_rec(f);
    ret_if!(rc_sub(rec) > 0, note_candidate(rec));
    zk_drop(rec, fn_kind(f));
    dispose(rec);
}

/// The size of the record of a closure.
///
/// # Arguments
///
/// * `f` - The value, a closure.
///
/// # Returns
///
/// The size in octets.
#[unsafe(export_name = stamped!("fn_size"))]
pub extern "C" fn zk_fn_size(f: ZkFn) -> usize
{
    // SAFETY: a closure record holds its kind.
    unsafe { (*fn_kind(f)).size }
}

/// Let go of one count of the record of a closure.
///
/// # Arguments
///
/// * `f` - The value.
#[unsafe(export_name = stamped!("fn_release"))]
pub extern "C" fn zk_fn_release(f: ZkFn)
{
    let mut slot = f;
    drop_fn((&raw mut slot).cast());
}

/// The kind a closure record holds, for the walks of the collector.
///
/// # Arguments
///
/// * `f` - The value.
///
/// # Returns
///
/// The kind.
pub fn record_kind(f: ZkFn) -> *const ZkKind { fn_kind(f) }

#[cfg(test)]
mod tests
{
    use super::*;

    /// A counted block with one holder and 16 octets of cells, outside the
    /// heap of a program, as the count helpers see it.
    ///
    /// # Returns
    ///
    /// The storage, which must outlive the block, and the first cell.
    fn block_of_one() -> (Vec<u64>, usize)
    {
        let mut room = vec![0u64; (SEQ_HEAD + 16) / 8];
        #[rustfmt::skip]
        let h = Head { rc: 1, size: 16, cell: std::ptr::null(), drop_cell: std::ptr::null(),
            len: 0, next: std::ptr::null_mut(), color: 0, buffered: 0, pinned: 0, owner: 0 };
        let start = room.as_mut_ptr().cast::<u8>();
        // SAFETY: the storage holds a head and 16 octets, aligned for a word.
        unsafe { start.cast::<Head>().write(h) };
        let base = start as usize + SEQ_HEAD;
        (room, base)
    }

    #[test]
    fn a_holder_added_and_let_go_leaves_the_count_as_it_was()
    {
        let (_room, base) = block_of_one();
        let p = base as *const u8;
        rc_add(p);
        assert_eq!(rc_of(p), 2);
        assert_eq!(rc_sub(p), 1);
    }

    #[test]
    fn threads_change_one_count_with_no_update_lost()
    {
        let (_room, base) = block_of_one();
        zk_threads_begin();
        let workers: Vec<_> = (0..8)
            .map(|_| {
                std::thread::spawn(move || {
                    let p = base as *const u8;
                    (0..100_000).for_each(|_| rc_add(p));
                    (0..99_000).for_each(|_| _ = rc_sub(p));
                })
            })
            .collect();
        workers.into_iter().for_each(|w| w.join().unwrap());
        assert_eq!(rc_of(base as *const u8), 1 + 8 * 1000);
    }
}
