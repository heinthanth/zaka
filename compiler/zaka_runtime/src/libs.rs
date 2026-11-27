//! The C libs this runtime serves. Each static lib of a program built with the
//! same `zakac` links to one runtime, so the runtime keeps a record of each
//! lib: its table, the jump back to its wrapper, its grants and its last
//! trap. A wrapper enters its lib before the call and returns to the lib
//! around it after, so a lib that calls C, which calls another lib, finds its
//! own record again.

use std::cell::Cell;
use std::cell::RefCell;
use std::ffi::CStr;
use std::ffi::CString;
use std::ffi::c_char;
use std::ffi::c_void;
use std::ptr::null;
use std::ptr::null_mut;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::AtomicPtr;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use zaka_host::Gate;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::do_while;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::ctx::depth;
use crate::ctx::zk_ctx;
use crate::lib_lock::Lock;
use crate::program::ZkProgram;
use crate::program::set_thread_prog;
use crate::program::thread_prog;
use crate::seqs::DropMark;

/// A C lib the runtime serves.
pub struct ZkLib
{
    /// The table of its program.
    prog: *const ZkProgram,
    /// The function that jumps back to the wrapper of its call in progress.
    leave: extern "C" fn(i32),
    /// Its name, as the build named it.
    name: CString,
    /// Where it stands, as [`zk_lib_state_of`] names it.
    state: AtomicI32,
    /// Its gate, made from its grants at the first check.
    pub(crate) gate: Mutex<Option<Gate>>,
    /// The trap it stopped on last.
    last: Mutex<Option<Box<TrapRecord>>>,
    /// The heap octets its calls hold, which its `--max-heap` caps.
    pub(crate) heap: AtomicUsize,
    /// The function of its glue that runs a step as a call of the lib, with a
    /// jump back for a trap, for C that calls a function of the program on a
    /// thread with no call of a lib in progress.
    guard: Guard,
    /// The lock its calls that write its globals hold.
    pub(crate) lock: Lock,
}

/// The function of the glue of a lib that runs a step as a call of the lib:
/// it takes the step and what the step takes, and gives the status of the
/// call.
pub type Guard = extern "C" fn(extern "C" fn(*mut c_void), *mut c_void) -> i32;

/// The state of a lib before its first call, and after a reset.
const FRESH: i32 = 0;
/// The state of a lib whose calls run.
const RUNS: i32 = 1;

thread_local! {
    /// The locks of the libs this thread held, each with its depth, for each
    /// call to C it waits in, the latest last. The rest of the state of its
    /// calls belongs to the thread and needs no parking.
    static PARKED: RefCell<Vec<Vec<(usize, u32)>>> = const { RefCell::new(Vec::new()) };
}

thread_local! {
    /// The lib whose call is in progress on this thread, or NULL outside each
    /// lib.
    static CURRENT: Cell<*mut ZkLib> = const { Cell::new(null_mut()) };
}

thread_local! {
    /// The depth of the calls where the call of the current lib began.
    static BASE: Cell<u32> = const { Cell::new(0) };
}

thread_local! {
    /// The status of the trap that ended the call in progress: 1 + its id.
    static STATUS: Cell<i32> = const { Cell::new(0) };
}

/// The calls in progress over each lib: the count of calls with no lock in
/// the low 32 bits, the count of OS threads that run a call under a lock from
/// [`LOCKED_ONE`] up, and [`COLLECTS`] while the ring collector runs.
static FREE: AtomicU64 = AtomicU64::new(0);

/// The part of [`FREE`] that counts the calls with no lock.
const FREE_PART: u64 = 0xFFFF_FFFF;

/// One OS thread that runs a call under the lock of a lib, in [`FREE`].
const LOCKED_ONE: u64 = 1 << 32;

/// The bit of [`FREE`] that the ring collector holds.
const COLLECTS: u64 = 1 << 63;

/// The count of the libs opened: from the second on, calls of two libs run at
/// the same time, so the counts of the heap turn atomic.
static OPENED: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    /// Where a trap of the call in progress jumps: the `jmp_buf` of the wrapper
    /// of the call, which its glue reads.
    static JUMP: Cell<*mut c_void> = const { Cell::new(null_mut()) };
}

/// What a trap inside a call of a lib can leave half done, as the call found
/// it.
struct Mark
{
    /// The count of closures lent to C calls.
    lends: usize,
    /// The sink of the output, as an address.
    sink: usize,
    /// The queues of the drops.
    drops: DropMark,
    /// The count of the locks of libs the OS thread held.
    holds: usize,
}

thread_local! {
    /// A mark for each call of a lib in progress on this thread, the
    /// innermost last.
    static MARKS: RefCell<Vec<Mark>> = const { RefCell::new(Vec::new()) };
}

/// The slot of each lib this runtime serves, each once.
static SLOTS: Mutex<Vec<usize>> = Mutex::new(Vec::new());

/// The lib a wrapper found when it entered its own, for the return.
#[repr(C)]
pub struct ZkLibPlace
{
    /// The lib of the call around it, or NULL outside each lib.
    pub lib: *mut ZkLib,
    /// The table in use around it.
    pub prog: *const ZkProgram,
    /// Where the call of that lib began.
    pub base: u32,
}

/// One call in progress when a trap stopped a lib.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ZkTrapFrame
{
    /// The Zaka name of the function, UTF-8, or NULL when the build keeps
    /// no calls.
    pub function: *const c_char,
    /// The source file, or NULL when the build keeps no source lines.
    pub file: *const c_char,
    /// The line, from 1; 0 with no source line.
    pub line: u32,
    /// The column, from 1; 0 with no source line.
    pub column: u32,
}

/// A trap a lib stopped on, as C reads it. It remains valid until the next
/// trap of the same lib or its reset.
#[repr(C)]
pub struct ZkTrapInfo
{
    /// The id of the trap; the status of the call is the id + 1.
    pub code: i32,
    /// The short name of the trap, such as `overflow`.
    pub kind: *const c_char,
    /// The trap line, with its place.
    pub message: *const c_char,
    /// The name of the lib.
    pub lib: *const c_char,
    /// The call the trap stopped in.
    pub r#where: ZkTrapFrame,
    /// The calls in progress, the innermost first.
    pub frames: *const ZkTrapFrame,
    /// The count of `frames`: 0 when the build keeps no calls.
    pub frame_count: usize,
}

/// A trap with the texts and the frames its C form points into.
pub(crate) struct TrapRecord
{
    /// The C form.
    info: ZkTrapInfo,
    /// The texts it points at.
    _texts: Vec<CString>,
    /// The frames it points at.
    _frames: Vec<ZkTrapFrame>,
}

impl TrapRecord
{
    /// A record of a trap.
    ///
    /// # Arguments
    ///
    /// * `code` - The id of the trap.
    /// * `kind` - Its short name.
    /// * `message` - The trap line.
    /// * `lib` - The name of the lib.
    /// * `calls` - Each call in progress, the innermost first: its function,
    ///   and its file, line and column when the build keeps them.
    ///
    /// # Returns
    ///
    /// The record.
    pub(crate) fn new(
        code: i32, kind: &str, message: Vec<u8>, lib: *const c_char,
        calls: Vec<(Vec<u8>, Option<(Vec<u8>, u32, u32)>)>,
    ) -> Box<TrapRecord>
    {
        let mut texts = vec![c_text(kind.as_bytes().to_vec()), c_text(message)];
        let mut frames = Vec::with_capacity(calls.len());
        for (function, site) in calls
        {
            texts.push(c_text(function));
            let function = texts.last().map_or(null(), |t| t.as_ptr());
            let Some((file, line, column)) = site
            else
            {
                frames.push(ZkTrapFrame { function, file: null(), line: 0, column: 0 });
                continue;
            };
            texts.push(c_text(file));
            let file = texts.last().map_or(null(), |t| t.as_ptr());
            frames.push(ZkTrapFrame { function, file, line, column });
        }
        let none = ZkTrapFrame { function: null(), file: null(), line: 0, column: 0 };
        #[rustfmt::skip]
        let info = ZkTrapInfo { code, kind: texts[0].as_ptr(), message: texts[1].as_ptr(),
            lib, r#where: frames.first().copied().unwrap_or(none),
            frames: tern!(frames.is_empty(), null(), frames.as_ptr()), frame_count: frames.len() };
        Box::new(TrapRecord { info, _texts: texts, _frames: frames })
    }
}

/// Text as a C string, with each zero octet in it dropped.
///
/// # Arguments
///
/// * `raw` - The octets.
///
/// # Returns
///
/// The C string.
fn c_text(mut raw: Vec<u8>) -> CString
{
    raw.retain(|c| *c != 0);
    CString::new(raw).unwrap_or_default()
}

/// The lib whose call is in progress.
///
/// # Returns
///
/// The lib, or `None` outside each lib.
pub(crate) fn current<'a>() -> Option<&'a ZkLib>
{
    // SAFETY: a lib lives as long as the program.
    unsafe { CURRENT.get().as_ref() }
}

/// The function that jumps back to the wrapper of the call in progress.
///
/// # Returns
///
/// The function, or `None` outside each lib, where a trap ends the run.
pub(crate) fn leave() -> Option<extern "C" fn(i32)> { current().map(|l| l.leave) }

/// Where the call of the current lib began, among the calls in progress.
///
/// # Returns
///
/// The depth, 0 outside each lib.
pub(crate) fn base() -> u32 { BASE.get() }

/// The name of the current lib.
///
/// # Returns
///
/// The name, or NULL outside each lib.
pub(crate) fn name() -> *const c_char { current().map_or(null(), |l| l.name.as_ptr()) }

/// Keep a trap as the last one of the current lib and of the runtime.
///
/// # Arguments
///
/// * `record` - The trap.
pub(crate) fn keep_trap(record: Box<TrapRecord>)
{
    let Some(lib) = current()
    else
    {
        return;
    };
    let info = &raw const record.info;
    let mut last = lib.last.lock().unwrap_or_else(PoisonError::into_inner);
    let old = last.replace(record);
    forget_latest(old.as_deref());
    // SAFETY: each slot is a global of the glue of a lib.
    each_slot(|slot| unsafe { *slot = info });
}

/// Run a step on each slot of the last trap: one when the linker kept one for
/// each lib, one for each lib when its compiler cannot share it.
///
/// # Arguments
///
/// * `step` - The step.
fn each_slot(step: impl Fn(*mut *const ZkTrapInfo))
{
    let slots = SLOTS.lock().unwrap_or_else(PoisonError::into_inner);
    slots.iter().for_each(|s| step(*s as *mut *const ZkTrapInfo));
}

/// Clear the latest trap of the runtime when it is `old`, which is freed.
///
/// # Arguments
///
/// * `old` - The trap that is freed.
fn forget_latest(old: Option<&TrapRecord>)
{
    let Some(old) = old
    else
    {
        return;
    };
    let gone = &raw const old.info;
    // SAFETY: each slot is a global of the glue of a lib.
    each_slot(|slot| unsafe { do_if!(*slot == gone, *slot = null()) });
}

/// Make the record of a C lib, on its first call, unless another thread made
/// it first: the lib lives as long as the program.
///
/// # Arguments
///
/// * `home` - Where the glue keeps the record; the runtime writes it once.
/// * `program` - The table of its program.
/// * `leave` - The function that jumps back to the wrapper of its call in
///   progress; it takes the trap and does not return.
/// * `name` - Its name.
/// * `slot` - The slot of the last trap of each lib, `zk_trap_slot_v1`.
/// * `guard` - The function of its glue that runs a step as a call of the lib.
///
/// # Returns
///
/// The lib.
#[unsafe(export_name = stamped!("lib_open"))]
pub extern "C" fn zk_lib_open(
    home: *mut *mut ZkLib, program: *const ZkProgram, leave: extern "C" fn(i32),
    name: *const c_char, slot: *mut *const ZkTrapInfo, guard: Guard,
) -> *mut ZkLib
{
    // SAFETY: the glue hands a static of its own, which the runtime alone
    // writes.
    let home = unsafe { AtomicPtr::from_ptr(home) };
    let mut slots = SLOTS.lock().unwrap_or_else(PoisonError::into_inner);
    let known = home.load(Ordering::Acquire);
    ret_if!(!known.is_null(), known);
    do_if!(!slots.contains(&(slot as usize)), slots.push(slot as usize));
    // SAFETY: the glue of the lib hands its name as a NUL-ended text.
    let name = unsafe { CStr::from_ptr(name) }.to_owned();
    #[rustfmt::skip]
    let lib = Box::new(ZkLib { prog: program, leave, name, state: AtomicI32::new(FRESH),
        gate: Mutex::new(None), last: Mutex::new(None), heap: AtomicUsize::new(0), guard,
        lock: Lock::new() });
    let lib = Box::into_raw(lib);
    crate::program::keep_last(program);
    home.store(lib, Ordering::Release);
    drop(slots);
    do_if!(OPENED.fetch_add(1, Ordering::SeqCst) > 0, crate::heap::zk_threads_begin());
    let was = zk_lib_enter(lib);
    zk_ctx();
    zk_lib_return(was);
    lib
}

/// Enter a lib for a call: its table and its traps serve the call.
///
/// # Arguments
///
/// * `lib` - The lib.
///
/// # Returns
///
/// The lib around it, for [`zk_lib_return`].
#[unsafe(export_name = stamped!("lib_enter"))]
pub extern "C" fn zk_lib_enter(lib: *mut ZkLib) -> ZkLibPlace
{
    let was = ZkLibPlace { lib: CURRENT.get(), prog: thread_prog(), base: base() };
    BASE.set(depth());
    CURRENT.set(lib);
    MARKS.with(|m| m.borrow_mut().push(mark()));
    // SAFETY: the glue hands the lib `zk_lib_open` made.
    set_thread_prog(unsafe { &*lib }.prog);
    was
}

/// Return to the lib around a call, after it.
///
/// # Arguments
///
/// * `was` - What [`zk_lib_enter`] gave.
#[unsafe(export_name = stamped!("lib_return"))]
pub extern "C" fn zk_lib_return(was: ZkLibPlace)
{
    MARKS.with(|m| m.borrow_mut().pop());
    CURRENT.set(was.lib);
    BASE.set(was.base);
    // Outside each lib the table of the last lib of the thread remains, for
    // `zk_free` from C, which counts the heap against its cap.
    do_if!(!was.prog.is_null(), set_thread_prog(was.prog));
}

/// The trap a lib stopped on last.
///
/// # Arguments
///
/// * `lib` - The lib, or NULL before its first call.
///
/// # Returns
///
/// The trap, or NULL while it stopped on none.
#[unsafe(export_name = stamped!("lib_trap"))]
pub extern "C" fn zk_lib_trap(lib: *const ZkLib) -> *const ZkTrapInfo
{
    // SAFETY: a lib lives as long as the program.
    let Some(lib) = (unsafe { lib.as_ref() })
    else
    {
        return null();
    };
    let last = lib.last.lock().unwrap_or_else(PoisonError::into_inner);
    last.as_ref().map_or(null(), |r| &raw const r.info)
}

/// The trap a lib of the program stopped on last: each lib writes its traps
/// to the slot it shares with the others, whichever build of `zakac` made it.
///
/// # Returns
///
/// The trap, or NULL while no lib stopped on one.
#[unsafe(export_name = stamped!("last_trap"))]
pub extern "C" fn zk_last_trap() -> *const ZkTrapInfo
{
    let slots = SLOTS.lock().unwrap_or_else(PoisonError::into_inner);
    // SAFETY: each slot is a global of the glue of a lib.
    slots.first().map_or(null(), |s| unsafe { *(*s as *const *const ZkTrapInfo) })
}

/// Forget the trap of a lib, for its reset.
///
/// # Arguments
///
/// * `lib` - The lib, or NULL before its first call.
#[unsafe(export_name = stamped!("lib_forget"))]
pub extern "C" fn zk_lib_forget(lib: *const ZkLib)
{
    // SAFETY: a lib lives as long as the program.
    let Some(lib) = (unsafe { lib.as_ref() })
    else
    {
        return;
    };
    let old = lib.last.lock().unwrap_or_else(PoisonError::into_inner).take();
    forget_latest(old.as_deref());
}

/// What a trap inside a call can leave half done, as it stands now.
///
/// # Returns
///
/// The mark.
fn mark() -> Mark
{
    #[cfg(not(target_os = "wasi"))]
    let lends = crate::callbacks::lends();
    #[cfg(target_os = "wasi")]
    let lends = 0;
    #[rustfmt::skip]
    let mark = Mark { lends, sink: crate::out::sink() as usize,
        drops: crate::seqs::drop_mark(), holds: crate::lib_lock::held_count() };
    mark
}

/// Put back what the call of a lib that a trap ended found as it began: the
/// closures it lent to C calls, the sink of the output, the queues of the
/// drops and the locks of libs, such as the lock of the lib of a closure C
/// called inside it. The calls around it keep theirs.
#[unsafe(export_name = stamped!("lib_unwind"))]
pub extern "C" fn zk_lib_unwind()
{
    let Some((lends, sink, drops, holds)) =
        MARKS.with(|m| m.borrow().last().map(|k| (k.lends, k.sink, k.drops, k.holds)))
    else
    {
        return;
    };
    #[cfg(not(target_os = "wasi"))]
    crate::callbacks::end_lends_from(lends);
    #[cfg(target_os = "wasi")]
    let _ = lends;
    crate::out::set_sink(sink as *mut crate::out::ZkBuf);
    crate::seqs::back_to(drops);
    crate::lib_lock::let_go_from(holds);
}

/// Whether the runtime serves a C lib: then the lock of each lib lets one
/// thread at a time into the calls of the lib that write its globals.
///
/// # Returns
///
/// `true` once a lib opened.
pub(crate) fn serves_libs() -> bool
{
    !SLOTS.lock().unwrap_or_else(PoisonError::into_inner).is_empty()
}

/// The function of the glue of the lib whose call is in progress that runs a
/// step as a call of the lib.
///
/// # Returns
///
/// The lib, as an address, or 0 outside each lib.
pub(crate) fn current_lib() -> usize { CURRENT.get() as usize }

/// Run a step as a call of a lib, on a thread with no call of a lib in
/// progress, through the guard of its glue.
///
/// # Arguments
///
/// * `lib` - The lib, as an address.
/// * `step` - The step.
/// * `data` - What the step takes.
///
/// # Returns
///
/// The status of the call: 0, or the status of the trap it stopped on.
pub(crate) fn guarded(
    lib: usize, step: extern "C" fn(*mut c_void), data: *mut c_void,
) -> i32
{
    // SAFETY: a lib lives as long as the program.
    let lib = unsafe { &*(lib as *const ZkLib) };
    (lib.guard)(step, data)
}

/// Whether this thread waits in C with calls of a lib parked.
///
/// # Returns
///
/// `true` when it does.
pub(crate) fn parked_here() -> bool { PARKED.with(|p| !p.borrow().is_empty()) }

/// Where a trap of the call in progress jumps.
///
/// # Returns
///
/// The `jmp_buf`, or NULL.
#[unsafe(export_name = stamped!("lib_jump"))]
pub extern "C" fn zk_lib_jump() -> *mut c_void { JUMP.get() }

/// Set where a trap of the call in progress jumps.
///
/// # Arguments
///
/// * `to` - The `jmp_buf` of the wrapper of the call, or the one of the call
///   around it at its end.
#[unsafe(export_name = stamped!("lib_jump_set"))]
pub extern "C" fn zk_lib_jump_set(to: *mut c_void) { JUMP.set(to) }

/// Put the jump of a thread in place of the one of the calling OS thread, and
/// give back the one before.
///
/// # Arguments
///
/// * `to` - The jump to run with, which gets the one before.
pub(crate) fn swap_jump(to: &mut *mut c_void) { *to = JUMP.replace(*to); }

/// Let other threads into the libs while this one waits in C: write out what
/// is buffered and let go of the lock of each lib it holds. Each call of a lib
/// to C does it, and
/// [`zk_lib_take_back`] once C returns.
#[unsafe(export_name = stamped!("lib_let_go"))]
pub extern "C" fn zk_lib_let_go()
{
    crate::out::flush_out();
    let held = crate::lib_lock::let_go_locks();
    PARKED.with(|p| p.borrow_mut().push(held));
}

/// Take the locks of the libs back once C returns, as [`zk_lib_let_go`] let go
/// of them.
#[unsafe(export_name = stamped!("lib_take_back"))]
pub extern "C" fn zk_lib_take_back()
{
    let Some(held) = PARKED.with(|p| p.borrow_mut().pop())
    else
    {
        return;
    };
    crate::lib_lock::take_locks(&held);
}

/// The state of a lib: 0 before its first call and after a reset, 3 while
/// its globals are being set, 1 while its calls run, 2 once a trap of a call
/// under its lock stopped it.
///
/// # Arguments
///
/// * `lib` - The lib, or NULL before its first call.
///
/// # Returns
///
/// The state.
#[unsafe(export_name = stamped!("lib_state_of"))]
pub extern "C" fn zk_lib_state_of(lib: *const ZkLib) -> i32
{
    // SAFETY: a lib lives as long as the program.
    unsafe { lib.as_ref() }.map_or(FRESH, |l| l.state.load(Ordering::Acquire))
}

/// Set the state of a lib, as [`zk_lib_state_of`] names it.
///
/// # Arguments
///
/// * `lib` - The lib; NULL is left alone.
/// * `state` - The state.
#[unsafe(export_name = stamped!("lib_state_set"))]
pub extern "C" fn zk_lib_state_set(lib: *const ZkLib, state: i32)
{
    // SAFETY: a lib lives as long as the program.
    do_if!(let Some(l) = unsafe { lib.as_ref() }, l.state.store(state, Ordering::Release));
}

/// The status of the trap that ended the call in progress on this thread.
///
/// # Returns
///
/// 1 + the id of the trap.
#[unsafe(export_name = stamped!("lib_status"))]
pub extern "C" fn zk_lib_status() -> i32 { STATUS.get() }

/// Keep the status of the trap that ends the call in progress on this thread.
///
/// # Arguments
///
/// * `status` - 1 + the id of the trap.
#[unsafe(export_name = stamped!("lib_status_set"))]
pub extern "C" fn zk_lib_status_set(status: i32) { STATUS.set(status) }

/// Count a call with no lock in, once the lib runs and the counts of the
/// heap are atomic: such a call reaches no mutable global and calls no C, so
/// calls of it run on several threads at once.
///
/// # Arguments
///
/// * `home` - Where the glue keeps the record of the lib.
///
/// # Returns
///
/// 1 when the call goes on, and is counted until [`zk_lib_free_out`]; 2 when
/// a trap stopped the lib; else 0 or 3, and the glue first opens the lib and
/// sets its globals under the lock, and asks again.
#[unsafe(export_name = stamped!("lib_free_in"))]
pub extern "C" fn zk_lib_free_in(home: *mut *mut ZkLib) -> i32
{
    // SAFETY: the glue hands a static of its own, which the runtime alone
    // writes.
    let lib = unsafe { AtomicPtr::from_ptr(home) }.load(Ordering::Acquire);
    ret_if!(lib.is_null(), FRESH);
    free_enter();
    let state = zk_lib_state_of(lib);
    ret_if!(state == RUNS && crate::heap::threaded(), RUNS);
    FREE.fetch_sub(1, Ordering::SeqCst);
    tern!(state == RUNS, FRESH, state)
}

/// Count a call with no lock in: a call of a lib, or a call C makes from a
/// thread of its own. It waits while the ring collector runs, which changes
/// counts with plain ops.
pub(crate) fn free_enter()
{
    do_while!(FREE.fetch_add(1, Ordering::SeqCst) & COLLECTS != 0, {
        FREE.fetch_sub(1, Ordering::SeqCst);
        std::thread::yield_now();
    });
}

/// The count of calls with no lock that run now: calls of a lib, and calls C
/// makes from threads of its own.
///
/// # Returns
///
/// The count.
pub(crate) fn free_calls() -> usize
{
    (FREE.load(Ordering::SeqCst) & FREE_PART) as usize
}

/// Count this OS thread among those that run a call under a lock, once it
/// holds its first lock. It waits while the ring collector runs.
pub(crate) fn locked_in()
{
    do_while!(FREE.fetch_add(LOCKED_ONE, Ordering::SeqCst) & COLLECTS != 0, {
        FREE.fetch_sub(LOCKED_ONE, Ordering::SeqCst);
        std::thread::yield_now();
    });
}

/// Count this OS thread out of those that run a call under a lock, once it
/// holds no lock.
pub(crate) fn locked_out() { FREE.fetch_sub(LOCKED_ONE, Ordering::SeqCst); }

/// Count a call with no lock out, at its end.
#[unsafe(export_name = stamped!("lib_free_out"))]
pub extern "C" fn zk_lib_free_out()
{
    crate::out::flush_line();
    FREE.fetch_sub(1, Ordering::SeqCst);
}

/// Wait until no call with no lock runs, before the globals of a lib are set
/// again: such calls read the constants. The caller holds the lock of its
/// lib, and a lib whose globals are being set lets no new call with no
/// lock in.
#[unsafe(export_name = stamped!("lib_drain"))]
pub extern "C" fn zk_lib_drain()
{
    do_while!(FREE.load(Ordering::SeqCst) & FREE_PART != 0, std::thread::yield_now());
}

/// Hold off the other calls of each lib while the ring collector runs: a
/// call under the lock of another lib changes counts as well.
///
/// # Returns
///
/// `true` when no call runs but the one of this OS thread, and the collector
/// goes on; it ends the hold with [`end_collect`].
pub(crate) fn begin_collect() -> bool
{
    let own = tern!(crate::lib_lock::holds_lock(), LOCKED_ONE, 0);
    let ord = Ordering::SeqCst;
    FREE.compare_exchange(own, own | COLLECTS, ord, ord).is_ok()
}

/// End the hold of [`begin_collect`].
pub(crate) fn end_collect() { FREE.fetch_and(!COLLECTS, Ordering::SeqCst); }
