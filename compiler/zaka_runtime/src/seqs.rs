//! Lists and text: the three words of a plain list or text (its block, its
//! count, the room of the block it owns), the walks that drop, share, lend
//! and own the blocks a value holds, the ring collector, and the edits of
//! cells and characters.

use std::cell::Cell;
use std::ffi::c_char;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use crate::heap::WORD;
use crate::heap::ZkFn;
use crate::heap::at_exit;
use crate::heap::block;
use crate::heap::drop_fn;
use crate::heap::free;
use crate::heap::head;
use crate::heap::holds_record;
use crate::heap::ld;
use crate::heap::malloc;
use crate::heap::move_octets;
use crate::heap::rc_add;
use crate::heap::rc_of;
use crate::heap::rc_sub;
use crate::heap::seq_grow;
use crate::heap::seq_kind;
use crate::heap::st;
use crate::heap::zero;
use crate::heap::zk_fn_dup;
use crate::heap::zk_fn_lend;
use crate::heap::zk_fn_rec;
use crate::heap::zk_ptr_add;
use crate::heap::zk_seq_free;
pub use crate::heap::zk_seq_new;
use crate::program::BOUNDS_BIT;
use crate::program::TEXT_BIT;
use crate::program::ZkKind;
use crate::program::check_on;
use crate::traps::named;
use crate::traps::stop_owned;
use crate::traps::trap;
use crate::traps::zk_addr_trap;
use crate::traps::zk_out_of_mem;
use crate::traps::zk_plain_trap;
use crate::traps::zk_range_trap;

/// The shape of a data kind, as the runtime walks it.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(missing_docs)]
pub enum ZkTag
{
    Sint,
    Uint,
    Float,
    Char,
    Bool,
    Ptr,
    Fn,
    List,
    Struct,
    Fallible,
    Seq,
    Text,
    Enum,
}

/// The number of each shape, as a descriptor holds it.
pub(crate) mod tag
{
    use super::ZkTag;

    pub(crate) const SINT: i32 = ZkTag::Sint as i32;
    pub(crate) const UINT: i32 = ZkTag::Uint as i32;
    pub(crate) const FLOAT: i32 = ZkTag::Float as i32;
    pub(crate) const CHAR: i32 = ZkTag::Char as i32;
    pub(crate) const BOOL: i32 = ZkTag::Bool as i32;
    pub(crate) const PTR: i32 = ZkTag::Ptr as i32;
    pub(crate) const FN: i32 = ZkTag::Fn as i32;
    pub(crate) const LIST: i32 = ZkTag::List as i32;
    pub(crate) const STRUCT: i32 = ZkTag::Struct as i32;
    pub(crate) const FALLIBLE: i32 = ZkTag::Fallible as i32;
    pub(crate) const SEQ: i32 = ZkTag::Seq as i32;
    pub(crate) const TEXT: i32 = ZkTag::Text as i32;
    pub(crate) const ENUM: i32 = ZkTag::Enum as i32;
}

/// The kind of a descriptor the generated code hands.
///
/// # Arguments
///
/// * `k` - The descriptor.
///
/// # Returns
///
/// The kind.
pub fn kind<'a>(k: *const ZkKind) -> &'a ZkKind
{
    // SAFETY: each descriptor lives in `zaka_kinds.c` for the whole run.
    unsafe { &*k }
}

/// The address `n` octets on.
///
/// # Arguments
///
/// * `p` - The address.
/// * `n` - The octets.
///
/// # Returns
///
/// The address.
pub fn at(p: *const u8, n: usize) -> *mut u8 { p.cast_mut().wrapping_add(n) }

// ---- the words ----

/// The first cell of a plain list or text.
///
/// # Arguments
///
/// * `at` - The value.
///
/// # Returns
///
/// The first cell.
#[unsafe(export_name = stamped!("seq_base"))]
pub extern "C" fn zk_seq_base(at: *const u8) -> *mut u8 { ld(at) }

/// The cell count of a plain list or text.
///
/// # Arguments
///
/// * `p` - The value.
///
/// # Returns
///
/// The count.
#[unsafe(export_name = stamped!("seq_len"))]
pub extern "C" fn zk_seq_len(p: *const u8) -> usize { ld(at(p, WORD)) }

/// The third word of a plain list or text: the size of its heap block, 0 when
/// it owns none.
///
/// # Arguments
///
/// * `p` - The value.
///
/// # Returns
///
/// The room in octets.
pub fn seq_room(p: *const u8) -> usize { ld(at(p, 2 * WORD)) }

/// Write the three words of a plain list or text.
///
/// # Arguments
///
/// * `p` - The slot.
/// * `base` - The first cell.
/// * `len` - The count.
/// * `room` - The size of the block it owns, 0 for none.
#[unsafe(export_name = stamped!("seq_put"))]
pub extern "C" fn zk_seq_put(p: *mut u8, base: *mut u8, len: usize, room: usize)
{
    st(p, base);
    st(at(p, WORD), len);
    st(at(p, 2 * WORD), room);
}

/// The octets of text or a plain list of u8s.
///
/// # Arguments
///
/// * `p` - The value.
///
/// # Returns
///
/// The octets.
pub fn octets<'a>(p: *const u8) -> &'a [u8]
{
    let (base, len) = (zk_seq_base(p), zk_seq_len(p));
    ret_if!(len == 0 || base.is_null(), &[]);
    // SAFETY: the value names `len` octets at its base.
    unsafe { std::slice::from_raw_parts(base, len) }
}

/// A duplicate of the octets of text that ends in a zero octet, as a C
/// function reads a string.
///
/// # Arguments
///
/// * `p` - The text.
///
/// # Returns
///
/// The duplicate on the heap, for `zk_text_cstr_free`.
#[unsafe(export_name = stamped!("text_cstr"))]
pub extern "C" fn zk_text_cstr(p: *const u8) -> *mut c_char
{
    let raw = octets(p);
    let dup = malloc(raw.len() + 1).cast::<u8>();
    do_if!(dup.is_null(), zk_out_of_mem(raw.len() as u64 + 1));
    move_octets(dup, raw.as_ptr(), raw.len());
    st(at(dup, raw.len()), 0u8);
    dup.cast()
}

/// Free a duplicate `zk_text_cstr` made.
///
/// # Arguments
///
/// * `dup` - The duplicate.
#[unsafe(export_name = stamped!("text_cstr_free"))]
pub extern "C" fn zk_text_cstr_free(dup: *mut c_char)
{
    // SAFETY: the duplicate came from malloc.
    unsafe { free(dup.cast()) }
}

/// Write static text into a slot.
///
/// # Arguments
///
/// * `slot` - The slot: text (three words) or a view (two).
/// * `s` - The octets.
/// * `len` - Their count.
/// * `size` - The size of the slot.
#[unsafe(export_name = stamped!("text_lit"))]
pub extern "C" fn zk_text_lit(slot: *mut u8, s: *const c_char, len: usize, size: usize)
{
    let base = if len > 0
    {
        s.cast_mut().cast::<u8>()
    }
    else
    {
        std::ptr::null_mut()
    };
    st(slot, base);
    st(at(slot, WORD), len);
    do_if!(size >= 3 * WORD, st(at(slot, 2 * WORD), 0usize));
}

// ---- kinds ----

/// A kind as storage holds it: an enumeration as its value kind.
///
/// # Arguments
///
/// * `k` - The kind.
///
/// # Returns
///
/// The kind.
pub fn repr(mut k: &ZkKind) -> &ZkKind
{
    while k.tag == tag::ENUM
    {
        k = kind(k.cell);
    }
    k
}

/// The octets of a cell of a plain list or text: 1 for text.
///
/// # Arguments
///
/// * `k` - The kind of the plain list or text.
///
/// # Returns
///
/// The stride.
fn stride(k: &ZkKind) -> usize
{
    if k.cell.is_null()
    {
        1
    }
    else
    {
        kind(k.cell).size
    }
}

/// Visit each text, plain list and function value a value holds, through its
/// fields and cells.
///
/// # Arguments
///
/// * `p` - The value.
/// * `k` - Its kind.
/// * `f` - The visit.
pub fn each_block(p: *mut u8, k: &ZkKind, f: &mut dyn FnMut(*mut u8, &ZkKind))
{
    let k = repr(k);
    ret_if!(k.holds_text == 0);
    match k.tag
    {
        tag::STRUCT | tag::FALLIBLE =>
        {
            for field in k.field_list()
            {
                each_block(at(p, field.offset), kind(field.kind), f);
            }
        }
        tag::LIST =>
        {
            let cell = kind(k.cell);
            (0..k.len).for_each(|i| each_block(at(p, i * cell.size), cell, f));
        }
        tag::TEXT | tag::SEQ | tag::FN => f(p, k),
        _ =>
        {}
    }
}

/// The cell kind of a plain list when its cells hold text of their own.
///
/// # Arguments
///
/// * `k` - The kind of the plain list or text.
///
/// # Returns
///
/// The cell kind, or NULL.
fn text_cells(k: &ZkKind) -> *const ZkKind
{
    ret_if!(k.tag != tag::SEQ || kind(k.cell).holds_text == 0, std::ptr::null());
    k.cell
}

// ---- the queue of blocks to free, and the ring collector ----
//
// The trial counts of the collector change the counts with plain ops: a
// collection runs with each other thread of the program halted.

/// A list of blocks that a static holds: the runtime runs on one thread, and
/// a static, unlike a thread local, still stands while the C lib runs
/// the collection at exit.
struct Blocks(Mutex<Vec<usize>>);

impl Blocks
{
    /// A list with no blocks.
    const fn new() -> Self { Blocks(Mutex::new(Vec::new())) }

    /// Add a block.
    ///
    /// # Arguments
    ///
    /// * `b` - The block.
    ///
    /// # Returns
    ///
    /// The count of blocks now.
    fn push(&self, b: *mut u8) -> usize
    {
        let mut list = self.0.lock().unwrap_or_else(|e| e.into_inner());
        list.push(b as usize);
        list.len()
    }

    /// Take the last block off.
    ///
    /// # Returns
    ///
    /// The block, or `None`.
    fn pop(&self) -> Option<*mut u8>
    {
        let mut list = self.0.lock().unwrap_or_else(|e| e.into_inner());
        list.pop().map(|b| b as *mut u8)
    }

    /// The count of blocks.
    ///
    /// # Returns
    ///
    /// The count.
    fn len(&self) -> usize { self.0.lock().unwrap_or_else(|e| e.into_inner()).len() }

    /// Drop the blocks past the first `n`.
    ///
    /// # Arguments
    ///
    /// * `n` - The count of blocks that remain.
    fn truncate(&self, n: usize)
    {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).truncate(n);
    }

    /// Take each block off.
    ///
    /// # Returns
    ///
    /// The blocks.
    fn take(&self) -> Vec<*mut u8>
    {
        let mut list = self.0.lock().unwrap_or_else(|e| e.into_inner());
        std::mem::take(&mut *list).into_iter().map(|b| b as *mut u8).collect()
    }
}

thread_local! {
    /// The blocks let go and still to free; the outermost drop frees them in a
    /// loop, so a long chain needs no deep call stack.
    static FREEING: Cell<*mut u8> = const { Cell::new(std::ptr::null_mut()) };
}
thread_local! {
    /// Whether a drop is freeing the queue, so an inner drop just queues.
    static DRAINING: Cell<bool> = const { Cell::new(false) };
}
/// The blocks a release left held that can be part of a ring.
static CANDIDATES: Blocks = Blocks::new();
thread_local! {
    /// The new blocks whose cells the deep duplicate of this thread still
    /// visits.
    static DEEP_TODO: Blocks = const { Blocks::new() };
}
thread_local! {
    /// The old objects the deep duplicate gave a new block; while it runs, the
    /// next word of each one's head points at its new block.
    static DEEP_SEEN: Blocks = const { Blocks::new() };
}

/// The octets of the blocks in the buffer of the ring collector.
static RING_WAITING: AtomicUsize = AtomicUsize::new(0);
/// The octets in the buffer that run a collection, unless the program sets
/// its own threshold.
static RING_THRESHOLD: AtomicUsize = AtomicUsize::new(16 * 1024);
/// Whether the program paused the collections a full buffer runs.
static RING_PAUSED: AtomicBool = AtomicBool::new(false);
/// The collections that ran.
static RING_RUNS: AtomicUsize = AtomicUsize::new(0);
/// The octets the collections freed.
static RING_FREED: AtomicUsize = AtomicUsize::new(0);
/// The act of [`zk_rings`] that runs the collector now.
const RINGS_COLLECT: u8 = 0;
/// The act of [`zk_rings`] that pauses the collections.
const RINGS_PAUSE: u8 = 1;
/// The act of [`zk_rings`] that resumes them.
const RINGS_RESUME: u8 = 2;

/// Queue a block that nothing holds now.
///
/// # Arguments
///
/// * `base` - The first cell of the block.
/// * `cell` - The kind of each cell to drop first, or NULL.
/// * `len` - The cells to drop.
fn queue_free(base: *mut u8, cell: *const ZkKind, len: usize)
{
    let h = head(base);
    h.drop_cell = cell;
    h.len = len;
    h.next = FREEING.get();
    FREEING.set(base);
}

/// Free a block that nothing holds; one in the buffer of the ring collector
/// remains, with a count of 0, until the collector frees it.
///
/// # Arguments
///
/// * `base` - The first cell of the block.
pub fn dispose(base: *mut u8)
{
    let h = head(base);
    if h.buffered != 0
    {
        h.rc = 0;
        return;
    }
    zk_seq_free(base);
}

/// Free each queued block, dropping its cells first; a cell let go joins the
/// queue.
fn drain_frees()
{
    loop
    {
        let base = FREEING.get();
        ret_if!(base.is_null());
        let h = head(base);
        FREEING.set(h.next);
        let (cell, len) = (h.drop_cell, h.len);
        if !cell.is_null()
        {
            let c = kind(cell);
            (0..len).for_each(|i| zk_drop(at(base, i * c.size), cell));
        }
        dispose(base);
    }
}

/// Whether a value of a kind holds a box, an object or a closure, at once or
/// inside a list or a struct: what a deep duplicate walks into.
///
/// # Arguments
///
/// * `k` - The kind.
///
/// # Returns
///
/// `true` for such a kind.
fn holds_shared(k: &ZkKind) -> bool
{
    let k = repr(k);
    match k.tag
    {
        _ if k.boxed != 0 => true,
        tag::FN => true,
        tag::LIST | tag::SEQ => holds_shared(kind(k.cell)),
        tag::STRUCT | tag::FALLIBLE =>
        {
            k.field_list().iter().any(|f| holds_shared(kind(f.kind)))
        }
        _ => false,
    }
}

/// Whether each kind of block can be part of a ring, worked out once: the
/// address of the descriptor, then the answer. Each kind has one descriptor.
static RING_KINDS: Mutex<Vec<(usize, bool)>> = Mutex::new(Vec::new());

/// Whether a block whose cells are of a kind can be part of a ring: whether
/// the blocks its cells hold a count of lead back to a block of the kind, or
/// hold a closure, which can hold each thing. Worked out once for each kind.
///
/// # Arguments
///
/// * `k` - The kind of the cells of the block.
///
/// # Returns
///
/// `true` for such a kind.
fn can_ring(k: *const ZkKind) -> bool
{
    let mut known = RING_KINDS.lock().unwrap_or_else(|e| e.into_inner());
    let found = known.iter().find(|(at, _)| *at == k as usize).map(|(_, r)| *r);
    ret_if!(let Some(r) = found, r);
    let ring = reaches_itself(k);
    known.push((k as usize, ring));
    ring
}

/// [`can_ring`], worked out with a walk over the kinds.
///
/// # Arguments
///
/// * `start` - The kind of the cells of the block.
///
/// # Returns
///
/// `true` when the walk meets the kind again, or a closure.
fn reaches_itself(start: *const ZkKind) -> bool
{
    let (mut seen, mut todo) = (vec![start], Vec::new());
    ret_if!(!held_kinds(kind(start), &mut todo), true);
    while let Some(k) = todo.pop()
    {
        ret_if!(std::ptr::eq(k, start), true);
        do_if!(seen.iter().any(|s| std::ptr::eq(*s, k)), continue);
        seen.push(k);
        ret_if!(!held_kinds(kind(k), &mut todo), true);
    }
    false
}

/// Gather the kinds of the cells of the blocks a value holds a count of,
/// through its fields and cells: a box or a plain list; text holds
/// characters alone.
///
/// # Arguments
///
/// * `k` - The kind of the value.
/// * `out` - The cell kinds found.
///
/// # Returns
///
/// `false` when the value holds a function value, whose closure can hold
/// each thing; else `true`.
fn held_kinds(k: &ZkKind, out: &mut Vec<*const ZkKind>) -> bool
{
    let k = repr(k);
    match k.tag
    {
        tag::FN => false,
        tag::SEQ =>
        {
            do_if!(!k.cell.is_null(), out.push(k.cell));
            true
        }
        tag::LIST => held_kinds(kind(k.cell), out),
        tag::STRUCT | tag::FALLIBLE =>
        {
            k.field_list().iter().all(|f| held_kinds(kind(f.kind), out))
        }
        _ => true,
    }
}

/// The blocks a counted block holds a count of: the record of each closure
/// that is not lent, and the block each text or plain list that owns one
/// holds.
///
/// # Arguments
///
/// * `base` - The first cell of the block.
/// * `out` - The list the blocks join.
fn children(base: *mut u8, out: &mut Vec<*mut u8>)
{
    let h = head(base);
    ret_if!(h.cell.is_null());
    let cell = kind(h.cell);
    let stride = cell.size.max(1);
    let mut note = |p: *mut u8, k: &ZkKind| {
        if k.tag == tag::FN
        {
            let f: ZkFn = ld(p);
            do_if!(holds_record(f), out.push(zk_fn_rec(f)));
            return;
        }
        let b = zk_seq_base(p);
        do_if!(!b.is_null() && seq_room(p) > 0, out.push(b));
    };
    (0..h.size / stride)
        .for_each(|i| each_block(at(base, i * stride), cell, &mut note));
}

/// Take off, for a trial, each count a block reached from a root gives
/// another: paint each such block pending.
///
/// # Arguments
///
/// * `root` - A block of the buffer.
fn mark_pending(root: *mut u8)
{
    let (mut todo, mut kids) = (vec![root], Vec::new());
    while let Some(b) = todo.pop()
    {
        let h = head(b);
        do_if!(h.color == 1, continue);
        h.color = 1;
        kids.clear();
        children(b, &mut kids);
        for &c in &kids
        {
            let ch = head(c);
            ch.rc = ch.rc.saturating_sub(1);
            todo.push(c);
        }
    }
}

/// Give a block held from outside, and each block it reaches, the counts the
/// trial took off, and paint them black.
///
/// # Arguments
///
/// * `root` - The held block.
fn scan_black(root: *mut u8)
{
    head(root).color = 0;
    let (mut todo, mut kids) = (vec![root], Vec::new());
    while let Some(b) = todo.pop()
    {
        kids.clear();
        children(b, &mut kids);
        for &c in &kids
        {
            let ch = head(c);
            ch.rc += 1;
            if ch.color != 0
            {
                ch.color = 0;
                todo.push(c);
            }
        }
    }
}

/// Paint white each pending block that nothing outside holds, and give a held
/// one and what it reaches their counts back.
///
/// # Arguments
///
/// * `root` - A block of the buffer.
fn scan(root: *mut u8)
{
    let mut todo = vec![root];
    while let Some(b) = todo.pop()
    {
        let h = head(b);
        do_if!(h.color != 1, continue);
        if h.rc > 0
        {
            scan_black(b);
            continue;
        }
        h.color = 2;
        children(b, &mut todo);
    }
}

/// Gather the white blocks a root reaches, painting them black so each is
/// gathered once.
///
/// # Arguments
///
/// * `root` - A block of the buffer.
/// * `out` - The blocks to free.
fn collect_white(root: *mut u8, out: &mut Vec<*mut u8>)
{
    let mut todo = vec![root];
    while let Some(b) = todo.pop()
    {
        let h = head(b);
        do_if!(h.color != 2 || h.buffered != 0, continue);
        h.color = 0;
        out.push(b);
        children(b, &mut todo);
    }
}

/// Free each group of blocks in the buffer, or reached from it, that holds
/// itself alone: trial deletion that halts the program while it runs.
fn collect_rings()
{
    // shortcut: a collection waits while a job or a call of a C lib with no
    // lock runs, since its trial counts change counts with plain ops; halt
    // the threads at safe points instead if programs with long jobs need
    // rings freed sooner.
    ret_if!(crate::jobs::jobs_run() || !crate::libs::begin_collect());
    collect_held();
    crate::libs::end_collect();
}

/// The work of [`collect_rings`], while no other thread changes a count.
fn collect_held()
{
    RING_WAITING.store(0, Ordering::Relaxed);
    let roots = CANDIDATES.take();
    let mut live = Vec::new();
    for b in roots
    {
        let h = head(b);
        h.buffered = 0;
        // A block let go while it waited is freed now.
        if h.rc == 0
        {
            zk_seq_free(b);
            continue;
        }
        live.push(b);
    }
    live.iter().for_each(|&b| mark_pending(b));
    live.iter().for_each(|&b| scan(b));
    let mut white = Vec::new();
    live.iter().for_each(|&b| collect_white(b, &mut white));
    RING_RUNS.fetch_add(1, Ordering::Relaxed);
    let freed: usize = white.iter().map(|&b| head(b).size).sum();
    RING_FREED.fetch_add(freed, Ordering::Relaxed);
    white.into_iter().for_each(|b| zk_seq_free(b));
}

/// One stat of the ring collector, for its field in the struct of the core: 0
/// the collections that ran, 1 the octets the collections freed, 2 the octets
/// waiting in the buffer, 3 the octets in the buffer that run a collection, 4
/// whether the program paused the collections (1 or 0).
///
/// # Arguments
///
/// * `n` - The field.
///
/// # Returns
///
/// The stat; 0 past the last field.
#[unsafe(export_name = stamped!("ring_stat"))]
pub extern "C" fn zk_ring_stat(n: u32) -> i64
{
    let stat = match n
    {
        0 => RING_RUNS.load(Ordering::Relaxed),
        1 => RING_FREED.load(Ordering::Relaxed),
        2 => RING_WAITING.load(Ordering::Relaxed),
        3 => RING_THRESHOLD.load(Ordering::Relaxed),
        4 => usize::from(RING_PAUSED.load(Ordering::Relaxed)),
        _ => 0,
    };
    i64::try_from(stat).unwrap_or(i64::MAX)
}

/// The collection the C lib runs as the program ends.
extern "C" fn collect_at_exit() { collect_rings() }

/// Note a block that a release left held: it can be part of a ring that
/// nothing outside holds. A full buffer runs a collection, unless the
/// program paused them.
///
/// # Arguments
///
/// * `base` - The first cell of the block.
pub fn note_candidate(base: *mut u8)
{
    let h = head(base);
    ret_if!(h.buffered != 0 || h.cell.is_null() || !can_ring(h.cell));
    // The program frees each ring left as it ends, as the VM does.
    static AT_EXIT: std::sync::Once = std::sync::Once::new();
    AT_EXIT.call_once(|| at_exit(collect_at_exit));
    h.buffered = 1;
    CANDIDATES.push(base);
    let waiting = RING_WAITING.fetch_add(h.size, Ordering::Relaxed) + h.size;
    do_if!(!RING_PAUSED.load(Ordering::Relaxed) && full(waiting), collect_rings());
}

/// Whether the octets in the buffer run a collection.
///
/// # Arguments
///
/// * `waiting` - The octets in the buffer.
///
/// # Returns
///
/// `true` at the threshold or past it.
fn full(waiting: usize) -> bool { waiting >= RING_THRESHOLD.load(Ordering::Relaxed) }

/// The work of a statement of the ring collector: run it now (`0`), pause the
/// collections a full buffer runs (`1`), resume them (`2`), or make `octets`
/// the octets in the buffer that run a collection (`3`). A resume, or a
/// threshold that the buffer already holds, collects at once unless paused.
///
/// # Arguments
///
/// * `act` - What it does, numbered as the compiler numbers it.
/// * `octets` - The threshold; unused by the other acts.
#[unsafe(export_name = stamped!("rings"))]
pub extern "C" fn zk_rings(act: u8, octets: u64)
{
    match act
    {
        RINGS_COLLECT => return collect_rings(),
        RINGS_PAUSE => return RING_PAUSED.store(true, Ordering::Relaxed),
        RINGS_RESUME => RING_PAUSED.store(false, Ordering::Relaxed),
        _ => RING_THRESHOLD
            .store(usize::try_from(octets).unwrap_or(usize::MAX), Ordering::Relaxed),
    }
    let now = full(RING_WAITING.load(Ordering::Relaxed));
    do_if!(now && !RING_PAUSED.load(Ordering::Relaxed), collect_rings());
}

/// Let go of a text or plain list: the last holder queues its block, and one
/// still held is noted for the ring collector.
///
/// # Arguments
///
/// * `p` - The text or plain list.
/// * `k` - Its kind.
fn drop_block(p: *mut u8, k: &ZkKind)
{
    ret_if!(k.tag == tag::FN, drop_fn(p));
    let (base, len, owned) = (zk_seq_base(p), zk_seq_len(p), seq_room(p) > 0);
    zero(p, 3 * WORD);
    ret_if!(!owned);
    ret_if!(rc_sub(base) > 0, note_candidate(base));
    queue_free(base, text_cells(k), len);
}

/// Let go of each block a value holds.
///
/// # Arguments
///
/// * `p` - The value.
/// * `k` - Its kind.
#[unsafe(export_name = stamped!("drop"))]
pub extern "C" fn zk_drop(p: *mut u8, k: *const ZkKind)
{
    each_block(p, kind(k), &mut drop_block);
    drain_queue();
}

/// Free the queue of blocks to free, unless an outer drop of this thread is
/// freeing it now.
fn drain_queue()
{
    ret_if!(DRAINING.get());
    DRAINING.set(true);
    drain_frees();
    DRAINING.set(false);
}

// ---- sharing ----

/// Give a text or plain list a heap block of its own, with copies of the
/// text of its cells.
///
/// # Arguments
///
/// * `p` - The text or plain list.
/// * `k` - Its kind.
fn clone_block(p: *mut u8, k: &ZkKind)
{
    let (count, stride) = (zk_seq_len(p), stride(k));
    let size = count * stride;
    ret_if!(size == 0, zk_seq_put(p, std::ptr::null_mut(), 0, 0));
    let block = zk_seq_new(size);
    seq_kind(block, k.cell);
    move_octets(block, zk_seq_base(p), size);
    zk_seq_put(p, block, count, size);
    let cell = text_cells(k);
    do_if!(
        !cell.is_null(),
        (0..count).for_each(|i| zk_dup_seqs(at(block, i * stride), cell))
    );
}

/// Give a text or plain list with cells a block of its own: an owned block is
/// shared and counted; a lent or static one, or a pinned one, is copied.
///
/// # Arguments
///
/// * `p` - The text or plain list.
/// * `k` - Its kind.
fn dup_block(p: *mut u8, k: &ZkKind)
{
    if k.tag == tag::FN
    {
        let f: ZkFn = ld(p);
        return st(p, zk_fn_dup(f));
    }
    let base = zk_seq_base(p);
    ret_if!(base.is_null());
    if seq_room(p) > 0 && head(base).pinned == 0
    {
        rc_add(base);
        return;
    }
    clone_block(p, k);
}

/// Give a text or plain list a block of its own when other values share its
/// block, so a write reaches this value alone.
///
/// # Arguments
///
/// * `p` - The text or plain list, which owns its block.
/// * `k` - Its kind.
fn unshare(p: *mut u8, k: &ZkKind)
{
    let base = zk_seq_base(p);
    ret_if!(base.is_null() || rc_of(base) <= 1);
    let len = zk_seq_len(p);
    clone_block(p, k);
    // Another thread can let go of the old block meanwhile, leaving this the
    // last holder; then this frees it.
    ret_if!(rc_sub(base) > 0);
    queue_free(base, text_cells(k), len);
    drain_queue();
}

/// Duplicate each block a value holds.
///
/// # Arguments
///
/// * `p` - The value.
/// * `k` - Its kind.
#[unsafe(export_name = stamped!("dup_seqs"))]
pub extern "C" fn zk_dup_seqs(p: *mut u8, k: *const ZkKind)
{
    each_block(p, kind(k), &mut dup_block);
}

/// Duplicate one text, plain list or function value inside a deep duplicate:
/// an object gets its new block, made once; a list that can reach an object
/// gets a new block whose cells are visited in turn; the rest are shared.
///
/// # Arguments
///
/// * `p` - The slot.
/// * `k` - Its kind.
fn deep_block(p: *mut u8, k: &ZkKind)
{
    ret_if!(k.tag == tag::FN || !holds_shared(k), dup_block(p, k));
    let base = zk_seq_base(p);
    ret_if!(base.is_null());
    let h = head(base);
    if !h.next.is_null()
    {
        rc_add(h.next);
        return st(p, h.next);
    }
    let count = zk_seq_len(p);
    let size = count * stride(k);
    ret_if!(size == 0, zk_seq_put(p, std::ptr::null_mut(), 0, 0));
    let block = zk_seq_new(size);
    seq_kind(block, k.cell);
    move_octets(block, base, size);
    zk_seq_put(p, block, count, size);
    DEEP_TODO.with(|b| b.push(block));
    ret_if!(k.boxed == 0);
    h.next = block;
    DEEP_SEEN.with(|b| b.push(base));
}

/// Duplicate a value through its objects: each object it reaches gets a block
/// of its own, once, so a ring remains a ring.
///
/// # Arguments
///
/// * `p` - The value.
/// * `k` - Its kind.
#[unsafe(export_name = stamped!("deep_seqs"))]
pub extern "C" fn zk_deep_seqs(p: *mut u8, k: *const ZkKind)
{
    each_block(p, kind(k), &mut deep_block);
    while let Some(b) = DEEP_TODO.with(|b| b.pop())
    {
        let h = head(b);
        let cell = kind(h.cell);
        let stride = cell.size.max(1);
        (0..h.size / stride)
            .for_each(|i| each_block(at(b, i * stride), cell, &mut deep_block));
    }
    while let Some(b) = DEEP_SEEN.with(|b| b.pop())
    {
        head(b).next = std::ptr::null_mut();
    }
}

/// Lend each block a value holds: a room of 0, so a drop frees nothing.
///
/// # Arguments
///
/// * `p` - The value.
/// * `k` - Its kind.
#[unsafe(export_name = stamped!("lend_seqs"))]
pub extern "C" fn zk_lend_seqs(p: *mut u8, k: *const ZkKind)
{
    each_block(p, kind(k), &mut |p, k| {
        if k.tag == tag::FN
        {
            let f: ZkFn = ld(p);
            return st(p, zk_fn_lend(f));
        }
        st(at(p, 2 * WORD), 0usize);
    });
}

/// Give each text or plain list a value holds a block of its own: one with a
/// room of 0 is copied, a shared one unshared.
///
/// # Arguments
///
/// * `p` - The value.
/// * `k` - Its kind.
///
/// # Returns
///
/// The value.
#[unsafe(export_name = stamped!("own"))]
pub extern "C" fn zk_own(p: *mut u8, k: *const ZkKind) -> *mut u8
{
    own(p, kind(k));
    p
}

/// Give each text or plain list a value holds a block of its own.
///
/// # Arguments
///
/// * `p` - The value.
/// * `k` - Its kind.
fn own(p: *mut u8, k: &ZkKind)
{
    each_block(p, k, &mut |p, k| {
        ret_if!(k.tag == tag::FN);
        ret_if!(seq_room(p) == 0, clone_block(p, k));
        unshare(p, k);
    });
}

/// The address of the block of an object, which holds one more count of it.
///
/// # Arguments
///
/// * `p` - The object.
///
/// # Returns
///
/// The block, or NULL for an object that is the zero word.
#[unsafe(export_name = stamped!("obj_ptr"))]
pub extern "C" fn zk_obj_ptr(p: *const u8) -> *mut u8
{
    let base = zk_seq_base(p);
    do_if!(!base.is_null(), rc_add(base));
    base
}

/// Write the object a pointer to its block names; the object takes the count
/// the pointer held.
///
/// # Arguments
///
/// * `p` - The slot of the object.
/// * `base` - The block, or NULL for the zero word.
#[unsafe(export_name = stamped!("ptr_obj"))]
pub extern "C" fn zk_ptr_obj(p: *mut u8, base: *mut u8)
{
    let size = if base.is_null() { 0 } else { head(base).size };
    zk_seq_put(p, base, usize::from(!base.is_null()), size);
}

/// Pin the block of text or a plain list: an address of one of its cells is
/// taken, so it is never shared again.
///
/// # Arguments
///
/// * `p` - The text or plain list, which owns its block.
///
/// # Returns
///
/// The value.
#[unsafe(export_name = stamped!("pin"))]
pub extern "C" fn zk_pin(p: *mut u8) -> *mut u8
{
    let base = zk_seq_base(p);
    do_if!(!base.is_null(), head(base).pinned = 1);
    p
}

/// Write zero octets.
///
/// # Arguments
///
/// * `p` - The first octet.
/// * `size` - The count.
#[unsafe(export_name = stamped!("zero"))]
pub extern "C" fn zk_zero(p: *mut u8, size: usize) { zero(p, size) }

/// Move a list into a heap block of a plain list.
///
/// # Arguments
///
/// * `dst` - The plain list.
/// * `src` - The list.
/// * `list` - The kind of the list.
/// * `fixed` - 1 for a static list, which holds no count and no drop frees.
#[unsafe(export_name = stamped!("to_heap"))]
pub extern "C" fn zk_to_heap(
    dst: *mut u8, src: *const u8, list: *const ZkKind, fixed: i32,
)
{
    let list = kind(list);
    let count = list.len;
    let size = count * kind(list.cell).size;
    ret_if!(size == 0, zk_seq_put(dst, std::ptr::null_mut(), 0, 0));
    let b = if fixed != 0
    {
        block(size)
    }
    else
    {
        zk_seq_new(size)
    };
    do_if!(fixed == 0, seq_kind(b, list.cell));
    move_octets(b, src, size);
    zk_seq_put(dst, b, count, if fixed != 0 { 0 } else { size });
}

// ---- cells and slices ----

/// The cell a number names, counted from `first`.
///
/// # Arguments
///
/// * `index` - The number.
/// * `len` - The cell count.
/// * `first` - The number of the first cell: 0 or 1.
///
/// # Returns
///
/// The cell, counted from 0.
#[unsafe(export_name = stamped!("cell_of"))]
pub extern "C" fn zk_cell_of(index: i64, len: usize, first: i32) -> usize
{
    in_range(trap::INDEX_OUT_OF_RANGE, index, len as i64, first, 1)
}

/// A number counted from `first`, inside `count` places, or the run stops.
///
/// # Arguments
///
/// * `which` - The trap.
/// * `index` - The number.
/// * `count` - The places.
/// * `first` - The number of the first place: 0 or 1.
/// * `short` - 1 when the last place is `count - 1`, 0 when it is `count`.
///
/// # Returns
///
/// The place, counted from 0.
fn in_range(which: i32, index: i64, count: i64, first: i32, short: i64) -> usize
{
    let first = i64::from(first);
    let last = count + first - short;
    do_if!(
        check_on(BOUNDS_BIT) && (index < first || index > last),
        zk_range_trap(which, index, first, last)
    );
    (index - first) as usize
}

/// The character a number names, counted from `first`.
///
/// # Arguments
///
/// * `index` - The number.
/// * `len` - The character count.
/// * `first` - The number of the first character.
///
/// # Returns
///
/// The character, counted from 0.
fn char_index(index: i64, len: usize, first: i32) -> usize
{
    in_range(trap::CHAR_OUT_OF_RANGE, index, len as i64, first, 1)
}

/// The place a number names, in front of a cell or at the end.
///
/// # Arguments
///
/// * `index` - The number.
/// * `count` - The cell count.
/// * `first` - The number of the first cell.
///
/// # Returns
///
/// The place, counted from 0.
fn place_of(index: i64, count: usize, first: i32) -> usize
{
    in_range(trap::NO_PLACE_TO_INSERT, index, count as i64, first, 0)
}

/// The address of the fields of an object: cell 0 of its box; an object that
/// holds the zero word stops the run.
///
/// # Arguments
///
/// * `p` - The box.
///
/// # Returns
///
/// The address of the fields.
#[unsafe(export_name = stamped!("object_cell"))]
pub extern "C" fn zk_object_cell(p: *const u8) -> *mut u8
{
    do_if!(zk_seq_len(p) == 0, zk_plain_trap(trap::NULL_OBJECT));
    zk_seq_base(p)
}

/// The address of a cell of a slice or a plain list.
///
/// # Arguments
///
/// * `p` - The slice.
/// * `index` - The number.
/// * `first` - The number of the first cell.
/// * `stride` - The octets of a cell.
///
/// # Returns
///
/// The address.
#[unsafe(export_name = stamped!("slice_cell"))]
pub extern "C" fn zk_slice_cell(
    p: *const u8, index: i64, first: i32, stride: usize,
) -> *mut u8
{
    at(zk_seq_base(p), zk_cell_of(index, zk_seq_len(p), first) * stride)
}

/// Stop the run when an octet of text is inside a character.
///
/// # Arguments
///
/// * `base` - The first octet of the text.
/// * `n` - The octet, counted from `base`.
fn edge_check(base: *const u8, n: usize)
{
    let octet: u8 = ld(at(base, n));
    do_if!(octet & 0xC0 == 0x80, zk_addr_trap(trap::TEXT_CUT, at(base, n).cast()));
}

/// Make a slice of `len` cells from cell `start`, inside `limit` cells.
///
/// # Arguments
///
/// * `slot` - The slice: two words.
/// * `base` - The first cell of the source.
/// * `start` - The first cell of the slice.
/// * `len` - Its cell count.
/// * `limit` - The cell count of the source, or the largest size for none.
/// * `stride` - The octets of a cell.
/// * `text` - 1 for text, whose ends must fall between characters.
#[unsafe(export_name = stamped!("make_slice"))]
pub extern "C" fn zk_make_slice(
    slot: *mut u8, base: *mut u8, start: u64, len: u64, limit: usize, stride: usize,
    text: i32,
)
{
    let end = start.wrapping_add(len);
    let bounded = limit != usize::MAX;
    if check_on(BOUNDS_BIT) && bounded && (end < start || end > limit as u64)
    {
        let nums = vec![
            named("len", len as i64),
            named("start", start as i64),
            named("limit", limit as i64),
        ];
        stop_owned(trap::SLICE_OUT_OF_RANGE, nums);
    }
    let edges = check_on(TEXT_BIT) && text != 0 && bounded && !base.is_null();
    do_if!(edges && start < limit as u64, edge_check(base, start as usize));
    do_if!(edges && end < limit as u64, edge_check(base, end as usize));
    st(slot, zk_ptr_add(base, start as i64, stride));
    st(at(slot, WORD), len as usize);
}

/// The octet count of a valid UTF-8 sequence at the start of `s`, or 0 when
/// none starts there.
///
/// # Arguments
///
/// * `s` - The octets.
///
/// # Returns
///
/// The count.
fn utf8_len(s: &[u8]) -> usize
{
    let need = match s.first().copied()
    {
        Some(b) if b < 0x80 => 1,
        Some(b) if b >> 5 == 6 => 2,
        Some(b) if b >> 4 == 14 => 3,
        Some(b) if b >> 3 == 30 => 4,
        _ => 0,
    };
    ret_if!(need == 0 || need > s.len(), 0);
    match std::str::from_utf8(&s[..need])
    {
        Ok(_) => need,
        Err(_) => 0,
    }
}

/// Whether octets are UTF-8.
///
/// # Arguments
///
/// * `s` - The octets.
/// * `n` - Their count.
///
/// # Returns
///
/// 1 when the octets are UTF-8.
#[unsafe(export_name = stamped!("is_utf8"))]
pub extern "C" fn zk_is_utf8(s: *const u8, n: usize) -> i32
{
    ret_if!(n == 0, 1);
    // SAFETY: the caller hands `n` octets.
    let raw = unsafe { std::slice::from_raw_parts(s, n) };
    i32::from(std::str::from_utf8(raw).is_ok())
}

/// Stop the run when a plain list of u8s holds no UTF-8.
///
/// # Arguments
///
/// * `p` - The plain list.
#[unsafe(export_name = stamped!("check_utf8"))]
pub extern "C" fn zk_check_utf8(p: *const u8)
{
    ret_if!(!check_on(TEXT_BIT));
    let raw = octets(p);
    let mut i = 0;
    while i < raw.len()
    {
        let n = utf8_len(&raw[i..]);
        do_if!(n == 0, zk_addr_trap(trap::BAD_UTF8, raw[i..].as_ptr().cast()));
        i += n;
    }
}

// ---- plain lists ----

/// Put a cell in front of place `n` of a plain list, as the VM's `put_cell`
/// does: the list takes a block of its own first, and a full block moves to a
/// larger one.
///
/// # Arguments
///
/// * `p` - The plain list.
/// * `k` - Its kind.
/// * `src` - The new cell, moved out and zeroed.
/// * `n` - The place.
fn put_cell(p: *mut u8, k: &ZkKind, src: *mut u8, n: usize)
{
    let stride = kind(k.cell).size;
    own(p, k);
    let (mut block, count, held) = (zk_seq_base(p), zk_seq_len(p), seq_room(p));
    let mut room = held;
    let need = (count + 1) * stride;
    if need > held
    {
        room = need.max(2 * held).max(4 * stride);
        block = seq_grow(
            if held == 0
            {
                std::ptr::null_mut()
            }
            else
            {
                block
            },
            room,
        );
        seq_kind(block, k.cell);
    }
    if count > n
    {
        move_octets(
            at(block, (n + 1) * stride),
            at(block, n * stride),
            (count - n) * stride,
        );
    }
    move_octets(at(block, n * stride), src, stride);
    zero(src, stride);
    zk_seq_put(p, block, count + 1, room);
}

/// Add a cell at the end of a plain list.
///
/// # Arguments
///
/// * `p` - The plain list.
/// * `k` - Its kind.
/// * `src` - The new cell, moved out and zeroed.
#[unsafe(export_name = stamped!("add_cell"))]
pub extern "C" fn zk_add_cell(p: *mut u8, k: *const ZkKind, src: *mut u8)
{
    put_cell(p, kind(k), src, zk_seq_len(p))
}

/// Put a cell in front of a numbered place of a plain list.
///
/// # Arguments
///
/// * `p` - The plain list.
/// * `k` - Its kind.
/// * `index` - The number of the place.
/// * `first` - The number of the first cell.
/// * `src` - The new cell, moved out and zeroed.
#[unsafe(export_name = stamped!("insert_cell"))]
pub extern "C" fn zk_insert_cell(
    p: *mut u8, k: *const ZkKind, index: i64, first: i32, src: *mut u8,
)
{
    put_cell(p, kind(k), src, place_of(index, zk_seq_len(p), first))
}

/// Remove the last cell of a plain list, or the last character of text.
///
/// # Arguments
///
/// * `p` - The plain list or text.
/// * `k` - Its kind.
#[unsafe(export_name = stamped!("remove_cell"))]
pub extern "C" fn zk_remove_cell(p: *mut u8, k: *const ZkKind)
{
    let k = kind(k);
    ret_if!(k.tag == tag::TEXT, remove_char(p));
    let count = zk_seq_len(p);
    do_if!(count == 0, zk_plain_trap(trap::NO_CELL_TO_REMOVE));
    // The drop writes into the block, so a shared one is copied first.
    do_if!(seq_room(p) > 0, unshare(p, k));
    let (room, base) = (seq_room(p), zk_seq_base(p));
    if room > 0
    {
        let cell = kind(k.cell);
        zk_drop(at(base, (count - 1) * cell.size), k.cell);
    }
    zk_seq_put(p, base, count - 1, room);
}

// ---- text ----

/// The octet where character `n` of text starts, or the octet count when the
/// text has `n` characters or fewer.
///
/// # Arguments
///
/// * `s` - The octets.
/// * `n` - The character, counted from 0.
///
/// # Returns
///
/// The octet.
fn char_start(s: &[u8], n: usize) -> usize
{
    let starts = s.iter().enumerate().filter(|(_, c)| **c & 0xC0 != 0x80);
    starts.map(|(i, _)| i).nth(n).unwrap_or(s.len())
}

/// The character count of octets of text.
///
/// # Arguments
///
/// * `s` - The octets.
///
/// # Returns
///
/// The count.
fn chars_in(s: &[u8]) -> usize { s.iter().filter(|c| **c & 0xC0 != 0x80).count() }

/// The character that starts at the first octet, or U+FFFD when none does.
///
/// # Arguments
///
/// * `s` - The octets from there.
///
/// # Returns
///
/// The character.
fn decode(s: &[u8]) -> u32
{
    let need = utf8_len(s);
    let ch = std::str::from_utf8(&s[..need]).ok().and_then(|t| t.chars().next());
    ch.map_or(0xFFFD, u32::from)
}

/// The UTF-8 octets of a character.
///
/// # Arguments
///
/// * `c` - The character.
///
/// # Returns
///
/// The octets.
fn encode(c: u32) -> Vec<u8>
{
    let ch = char::from_u32(c).unwrap_or(char::REPLACEMENT_CHARACTER);
    ch.to_string().into_bytes()
}

/// Put octets as the new text at a slot, as the VM's `put_text` does: a full
/// block moves to a larger one, of at least 16 octets.
///
/// # Arguments
///
/// * `p` - The text, which owns its block.
/// * `s` - The octets, apart from the block.
fn put_text(p: *mut u8, s: &[u8])
{
    let (mut block, held) = (zk_seq_base(p), seq_room(p));
    let mut room = held;
    if s.len() > held
    {
        room = s.len().max(2 * held).max(16);
        block = seq_grow(
            if held == 0
            {
                std::ptr::null_mut()
            }
            else
            {
                block
            },
            room,
        );
    }
    move_octets(block, s.as_ptr(), s.len());
    zk_seq_put(p, block, s.len(), room);
}

/// Splice octets into text: the octets from `from` to `to` give place to the
/// added ones, then the text takes a block of its own and holds the result.
///
/// # Arguments
///
/// * `p` - The text.
/// * `from` - The first octet replaced.
/// * `to` - The octet after the last one replaced.
/// * `add` - The added octets.
fn splice(p: *mut u8, from: usize, to: usize, add: &[u8])
{
    let old = octets(p);
    let edited = [&old[..from], add, &old[to..]].concat();
    #[rustfmt::skip]
    let text = ZkKind { tag: tag::TEXT, size: 3 * WORD, cell: std::ptr::null(), len: 0,
        fields: std::ptr::null(), members: std::ptr::null(), name: std::ptr::null(),
        holds_text: 1, holds_borrow: 0, owns: 0, boxed: 0 };
    own(p, &text);
    put_text(p, &edited);
}

/// Add text at the end of text.
///
/// # Arguments
///
/// * `p` - The text.
/// * `text` - The added text.
#[unsafe(export_name = stamped!("add_text"))]
pub extern "C" fn zk_add_text(p: *mut u8, text: *const u8)
{
    let len = zk_seq_len(p);
    splice(p, len, len, octets(text))
}

/// Add a character at the end of text.
///
/// # Arguments
///
/// * `p` - The text.
/// * `c` - The character.
#[unsafe(export_name = stamped!("add_char"))]
pub extern "C" fn zk_add_char(p: *mut u8, c: u32)
{
    let len = zk_seq_len(p);
    splice(p, len, len, &encode(c))
}

/// The octet of the place in front of a numbered character of text.
///
/// # Arguments
///
/// * `p` - The text.
/// * `index` - The number.
/// * `first` - The number of the first character.
///
/// # Returns
///
/// The octet.
fn insert_at(p: *const u8, index: i64, first: i32) -> usize
{
    let s = octets(p);
    char_start(s, place_of(index, chars_in(s), first))
}

/// Put text in front of a numbered character of text.
///
/// # Arguments
///
/// * `p` - The text.
/// * `index` - The number.
/// * `first` - The number of the first character.
/// * `text` - The added text.
#[unsafe(export_name = stamped!("insert_text"))]
pub extern "C" fn zk_insert_text(p: *mut u8, index: i64, first: i32, text: *const u8)
{
    let split = insert_at(p, index, first);
    splice(p, split, split, octets(text))
}

/// Put a character in front of a numbered character of text.
///
/// # Arguments
///
/// * `p` - The text.
/// * `index` - The number.
/// * `first` - The number of the first character.
/// * `c` - The character.
#[unsafe(export_name = stamped!("insert_char"))]
pub extern "C" fn zk_insert_char(p: *mut u8, index: i64, first: i32, c: u32)
{
    let split = insert_at(p, index, first);
    splice(p, split, split, &encode(c))
}

/// Write a numbered character of text.
///
/// # Arguments
///
/// * `p` - The text.
/// * `index` - The number.
/// * `first` - The number of the first character.
/// * `c` - The character.
#[unsafe(export_name = stamped!("set_char"))]
pub extern "C" fn zk_set_char(p: *mut u8, index: i64, first: i32, c: u32)
{
    let s = octets(p);
    let n = char_index(index, chars_in(s), first);
    let (from, to) = (char_start(s, n), char_start(s, n + 1));
    splice(p, from, to, &encode(c))
}

/// The octets a range of text covers: characters counted from 1, or octets
/// counted from 0, both ends in the range.
///
/// # Arguments
///
/// * `p` - The text.
/// * `start` - The first character or octet.
/// * `end` - The last one.
/// * `chars` - 1 for characters, 0 for octets.
///
/// # Returns
///
/// The first octet and the octet after the last.
fn span(p: *const u8, start: i64, end: i64, chars: i32) -> (usize, usize)
{
    let s = octets(p);
    let in_chars = chars != 0;
    let count = if in_chars { chars_in(s) } else { s.len() };
    let pick = |n| {
        if in_chars
        {
            char_index(n, count, 1)
        }
        else
        {
            zk_cell_of(n, count, 0)
        }
    };
    let (lo, hi) = (pick(start), pick(end));
    if check_on(BOUNDS_BIT) && hi < lo
    {
        let start = named("start", lo as i64 + i64::from(chars));
        let end = named("end", hi as i64 + i64::from(chars));
        stop_owned(trap::END_BEFORE_START, vec![start, end]);
    }
    ret_if!(in_chars, (char_start(s, lo), char_start(s, hi + 1)));
    let base = zk_seq_base(p);
    do_if!(check_on(TEXT_BIT) && lo < s.len(), edge_check(base, lo));
    do_if!(check_on(TEXT_BIT) && hi + 1 < s.len(), edge_check(base, hi + 1));
    (lo, hi + 1)
}

/// Replace a range of text with text.
///
/// # Arguments
///
/// * `p` - The text.
/// * `start` - The first character or octet.
/// * `end` - The last one.
/// * `chars` - 1 for characters, 0 for octets.
/// * `text` - The new text.
#[unsafe(export_name = stamped!("replace_text"))]
pub extern "C" fn zk_replace_text(
    p: *mut u8, start: i64, end: i64, chars: i32, text: *const u8,
)
{
    let (from, to) = span(p, start, end, chars);
    splice(p, from, to, octets(text))
}

/// Replace a range of text with a character.
///
/// # Arguments
///
/// * `p` - The text.
/// * `start` - The first character or octet.
/// * `end` - The last one.
/// * `chars` - 1 for characters, 0 for octets.
/// * `c` - The character.
#[unsafe(export_name = stamped!("replace_char"))]
pub extern "C" fn zk_replace_char(p: *mut u8, start: i64, end: i64, chars: i32, c: u32)
{
    let (from, to) = span(p, start, end, chars);
    splice(p, from, to, &encode(c))
}

/// Remove the last character of text.
///
/// # Arguments
///
/// * `p` - The text.
fn remove_char(p: *mut u8)
{
    let s = octets(p);
    do_if!(s.is_empty(), zk_plain_trap(trap::NO_CHAR_TO_REMOVE));
    let mut last = s.len() - 1;
    while last > 0 && s[last] & 0xC0 == 0x80
    {
        last -= 1;
    }
    zk_seq_put(p, zk_seq_base(p), last, seq_room(p));
}

/// The character count of text.
///
/// # Arguments
///
/// * `p` - The text.
///
/// # Returns
///
/// The count.
#[unsafe(export_name = stamped!("char_count"))]
pub extern "C" fn zk_char_count(p: *const u8) -> usize { chars_in(octets(p)) }

/// A numbered character of text.
///
/// # Arguments
///
/// * `p` - The text.
/// * `index` - The number.
/// * `first` - The number of the first character.
///
/// # Returns
///
/// The character.
#[unsafe(export_name = stamped!("char_at"))]
pub extern "C" fn zk_char_at(p: *const u8, index: i64, first: i32) -> u32
{
    let s = octets(p);
    let from = char_start(s, char_index(index, chars_in(s), first));
    decode(&s[from..])
}

/// The character that starts at an octet of text.
///
/// # Arguments
///
/// * `p` - The text.
/// * `index` - The octet, counted from 0.
///
/// # Returns
///
/// The character.
#[unsafe(export_name = stamped!("char_at_octet"))]
pub extern "C" fn zk_char_at_octet(p: *const u8, index: u64) -> u32
{
    let s = octets(p);
    let inside = index < s.len() as u64;
    let c = if inside
    {
        decode(&s[index as usize..])
    }
    else
    {
        0xFFFD
    };
    do_if!(
        check_on(BOUNDS_BIT) && (!inside || c == 0xFFFD),
        zk_range_trap(trap::CHAR_OUT_OF_RANGE, index as i64, 0, s.len() as i64 - 1)
    );
    c
}

/// Whether two texts hold the same octets.
///
/// # Arguments
///
/// * `a` - The first text.
/// * `b` - The second text.
///
/// # Returns
///
/// 1 when the two hold the same octets.
#[unsafe(export_name = stamped!("text_eq"))]
pub extern "C" fn zk_text_eq(a: *const u8, b: *const u8) -> i32
{
    i32::from(octets(a) == octets(b))
}

/// Where the queues of the drops stood when a call of a C lib began.
#[derive(Clone, Copy)]
pub(crate) struct DropMark
{
    /// The head of the queue of blocks to free, as an address.
    freeing: usize,
    /// Whether a drop was freeing the queue.
    draining: bool,
    /// The count of blocks that could be part of a ring.
    candidates: usize,
    /// The count of new blocks the deep duplicate still visited.
    deep_todo: usize,
    /// The count of old objects the deep duplicate gave a new block.
    deep_seen: usize,
}

/// Where the queues of the drops stand now.
///
/// # Returns
///
/// The mark.
pub(crate) fn drop_mark() -> DropMark
{
    #[rustfmt::skip]
    let mark = DropMark { freeing: FREEING.get() as usize,
        draining: DRAINING.get(), candidates: CANDIDATES.len(),
        deep_todo: DEEP_TODO.with(|b| b.len()), deep_seen: DEEP_SEEN.with(|b| b.len()) };
    mark
}

/// Put the queues of the drops back where a call of a C lib found them: a
/// trap inside the call left them half done.
///
/// # Arguments
///
/// * `mark` - Where the queues stood.
pub(crate) fn back_to(mark: DropMark)
{
    FREEING.set(mark.freeing as *mut u8);
    DRAINING.set(mark.draining);
    CANDIDATES.truncate(mark.candidates);
    DEEP_TODO.with(|b| b.truncate(mark.deep_todo));
    DEEP_SEEN.with(|b| b.truncate(mark.deep_seen));
}
