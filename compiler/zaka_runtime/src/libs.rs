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
use std::sync::atomic::AtomicPtr;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use zaka_host::Gate;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::tern;

use crate::program::ZkFrame;
use crate::program::ZkProgram;
use crate::program::make_frames;
use crate::program::zk_depth;
use crate::program::zk_prog;
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
    /// The depth of the calls where its call in progress began.
    base: Cell<u32>,
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
}

/// The function of the glue of a lib that runs a step as a call of the lib:
/// it takes the step and what the step takes, and gives the status of the
/// call.
pub type Guard = extern "C" fn(extern "C" fn(*mut c_void), *mut c_void) -> i32;

/// What the calls of a lib that one thread has in progress hold of the
/// runtime, parked while the thread waits in C and other threads run calls.
struct Parked
{
    /// How often the thread held the lock of the libs.
    holds: u32,
    /// The depth of its calls.
    depth: u32,
    /// The rows of the frame table its calls wrote.
    frames: Vec<ZkFrame>,
    /// The lib of its call, as an address.
    lib: usize,
    /// The table in use, as an address.
    prog: usize,
    /// Where the call of the lib began.
    base: u32,
    /// Where a trap of its call jumps, as an address.
    jump: usize,
    /// The marks of its calls.
    marks: Vec<Mark>,
    /// The closures it lent to C calls.
    lends: Vec<usize>,
    /// The sink of its output, as an address.
    sink: usize,
    /// The drop that frees the queue.
    drain: (usize, bool),
}

thread_local! {
    /// What the calls of this thread hold of the runtime, parked while it
    /// waits in C, the latest last.
    static PARKED: RefCell<Vec<Parked>> = const { RefCell::new(Vec::new()) };
}

/// Where a trap of the call in progress jumps: the `jmp_buf` of the wrapper
/// of the call, which its glue reads.
static JUMP: AtomicPtr<c_void> = AtomicPtr::new(null_mut());

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
}

/// A mark for each call of a lib in progress, the innermost last.
static MARKS: Mutex<Vec<Mark>> = Mutex::new(Vec::new());

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

/// The lib whose call is in progress, or NULL outside each lib.
static CURRENT: AtomicPtr<ZkLib> = AtomicPtr::new(null_mut());

/// The lib whose call is in progress.
///
/// # Returns
///
/// The lib, or `None` outside each lib.
pub(crate) fn current<'a>() -> Option<&'a ZkLib>
{
    // SAFETY: a lib lives as long as the program, and the lock of the libs
    // lets one thread in at a time.
    unsafe { CURRENT.load(Ordering::Acquire).as_ref() }
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
pub(crate) fn base() -> u32 { current().map_or(0, |l| l.base.get()) }

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

/// Make the record of a C lib, on its first call: the lib lives as long as
/// the program.
///
/// # Arguments
///
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
    program: *const ZkProgram, leave: extern "C" fn(i32), name: *const c_char,
    slot: *mut *const ZkTrapInfo, guard: Guard,
) -> *mut ZkLib
{
    // SAFETY: the glue of the lib hands its name as a NUL-ended text.
    let name = unsafe { CStr::from_ptr(name) }.to_owned();
    let mut slots = SLOTS.lock().unwrap_or_else(PoisonError::into_inner);
    do_if!(!slots.contains(&(slot as usize)), slots.push(slot as usize));
    drop(slots);
    #[rustfmt::skip]
    let lib = Box::new(ZkLib { prog: program, leave, name, base: Cell::new(0),
        gate: Mutex::new(None), last: Mutex::new(None), heap: AtomicUsize::new(0), guard });
    let lib = Box::into_raw(lib);
    let was = zk_lib_enter(lib);
    make_frames();
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
    // SAFETY: the lock of the libs lets one thread in at a time.
    let (prog, depth) = unsafe { (zk_prog, zk_depth) };
    let was = ZkLibPlace { lib: CURRENT.load(Ordering::Acquire), prog, base: base() };
    CURRENT.store(lib, Ordering::Release);
    MARKS.lock().unwrap_or_else(PoisonError::into_inner).push(mark());
    // SAFETY: the glue hands the lib `zk_lib_open` made.
    let lib = unsafe { &*lib };
    lib.base.set(depth);
    // SAFETY: as above.
    unsafe { zk_prog = lib.prog };
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
    MARKS.lock().unwrap_or_else(PoisonError::into_inner).pop();
    CURRENT.store(was.lib, Ordering::Release);
    // Outside each lib the table of the last lib remains, for `zk_free` from C,
    // which counts the heap against its cap.
    // SAFETY: the lock of the libs lets one thread in at a time.
    do_if!(!was.prog.is_null(), unsafe { zk_prog = was.prog });
    do_if!(let Some(lib) = current(), lib.base.set(was.base));
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
    Mark { lends, sink: crate::out::sink() as usize, drops: crate::seqs::drop_mark() }
}

/// Put back what the call of a lib that a trap ended found as it began: the
/// closures it lent to C calls, the sink of the output and the queues of the
/// drops. The calls around it keep theirs.
#[unsafe(export_name = stamped!("lib_unwind"))]
pub extern "C" fn zk_lib_unwind()
{
    let marks = MARKS.lock().unwrap_or_else(PoisonError::into_inner);
    let Some(mark) = marks.last()
    else
    {
        return;
    };
    #[cfg(not(target_os = "wasi"))]
    crate::callbacks::end_lends_from(mark.lends);
    crate::out::set_sink(mark.sink as *mut crate::out::ZkBuf);
    crate::seqs::back_to(mark.drops);
}

/// Whether the runtime serves a C lib: then a lock lets one thread at a time
/// into the code of the program.
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
pub(crate) fn current_lib() -> usize { CURRENT.load(Ordering::Acquire) as usize }

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
pub extern "C" fn zk_lib_jump() -> *mut c_void { JUMP.load(Ordering::Acquire) }

/// Set where a trap of the call in progress jumps.
///
/// # Arguments
///
/// * `to` - The `jmp_buf` of the wrapper of the call, or the one of the call
///   around it at its end.
#[unsafe(export_name = stamped!("lib_jump_set"))]
pub extern "C" fn zk_lib_jump_set(to: *mut c_void) { JUMP.store(to, Ordering::Release) }

/// Let other threads into the libs while this one waits in C: write out what
/// is buffered, park what its calls hold of the runtime, and let go of the
/// lock. Each call of a lib to C does it, and [`zk_lib_take_back`] once C
/// returns.
#[unsafe(export_name = stamped!("lib_let_go"))]
pub extern "C" fn zk_lib_let_go()
{
    crate::out::flush_out();
    // SAFETY: this thread holds the lock of the libs.
    let (depth, frames) = unsafe { (zk_depth, crate::program::zk_frames) };
    let rows = (depth as usize + 1).min(crate::program::frame_room());
    let kept = tern!(frames.is_null(), Vec::new(), unsafe {
        // SAFETY: the frame table holds `frame_room` rows.
        std::slice::from_raw_parts(frames, rows).to_vec()
    });
    #[cfg(not(target_os = "wasi"))]
    let lends = crate::callbacks::take_lends();
    #[cfg(target_os = "wasi")]
    let lends = Vec::new();
    #[rustfmt::skip]
    let parked = Parked { holds: 0, depth, frames: kept, lib: current_lib(),
        prog: unsafe { zk_prog } as usize, base: base(), jump: zk_lib_jump() as usize,
        marks: std::mem::take(&mut *MARKS.lock().unwrap_or_else(PoisonError::into_inner)),
        lends, sink: crate::out::sink() as usize, drain: crate::seqs::drain_state() };
    // The next thread starts with no call in progress.
    // SAFETY: as above.
    unsafe { zk_depth = 0 };
    CURRENT.store(null_mut(), Ordering::Release);
    zk_lib_jump_set(null_mut());
    crate::out::set_sink(null_mut());
    crate::seqs::set_drain_state((0, false));
    let holds = crate::start::let_go_lock();
    PARKED.with(|p| p.borrow_mut().push(Parked { holds, ..parked }));
}

/// Take the lock of the libs back once C returns, and what the calls of this
/// thread held of the runtime, as [`zk_lib_let_go`] parked it.
#[unsafe(export_name = stamped!("lib_take_back"))]
pub extern "C" fn zk_lib_take_back()
{
    let Some(parked) = PARKED.with(|p| p.borrow_mut().pop())
    else
    {
        return;
    };
    crate::start::take_lock(parked.holds);
    // SAFETY: this thread holds the lock of the libs, and the frame table
    // holds at least the rows it held at the time it parked them.
    unsafe {
        zk_depth = parked.depth;
        zk_prog = parked.prog as *const ZkProgram;
        let frames = crate::program::zk_frames;
        let rows = parked.frames.len();
        do_if!(
            !frames.is_null(),
            std::ptr::copy_nonoverlapping(parked.frames.as_ptr(), frames, rows)
        );
    }
    CURRENT.store(parked.lib as *mut ZkLib, Ordering::Release);
    do_if!(let Some(lib) = current(), lib.base.set(parked.base));
    zk_lib_jump_set(parked.jump as *mut c_void);
    *MARKS.lock().unwrap_or_else(PoisonError::into_inner) = parked.marks;
    #[cfg(not(target_os = "wasi"))]
    crate::callbacks::put_lends(parked.lends);
    crate::out::set_sink(parked.sink as *mut crate::out::ZkBuf);
    crate::seqs::set_drain_state(parked.drain);
}
