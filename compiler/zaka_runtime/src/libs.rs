//! The C libs this runtime serves. Each static lib of a program built with the
//! same `zakac` links to one runtime, so the runtime keeps a record of each
//! lib: its table, the jump back to its wrapper, its grants and its last
//! trap. A wrapper enters its lib before the call and returns to the lib
//! around it after, so a lib that calls C, which calls another lib, finds its
//! own record again.

use std::cell::Cell;
use std::ffi::CStr;
use std::ffi::CString;
use std::ffi::c_char;
use std::ffi::c_void;
use std::ptr::null;
use std::ptr::null_mut;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::sync::atomic::AtomicPtr;
use std::sync::atomic::Ordering;

use zaka_host::Gate;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::heap::zk_raw_free;
use crate::program::ZkProgram;
use crate::program::make_frames;
use crate::program::zk_depth;
use crate::program::zk_prog;

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
}

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

/// The trap a lib of this runtime stopped on last, or NULL.
static LATEST: AtomicPtr<ZkTrapInfo> = AtomicPtr::new(null_mut());

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
    let info = (&raw const record.info).cast_mut();
    let mut last = lib.last.lock().unwrap_or_else(PoisonError::into_inner);
    let old = last.replace(record);
    forget_latest(old.as_deref());
    LATEST.store(info, Ordering::Release);
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
    let gone = (&raw const old.info).cast_mut();
    _ = LATEST.compare_exchange(gone, null_mut(), Ordering::AcqRel, Ordering::Acquire);
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
///
/// # Returns
///
/// The lib.
#[unsafe(export_name = stamped!("lib_open"))]
pub extern "C" fn zk_lib_open(
    program: *const ZkProgram, leave: extern "C" fn(i32), name: *const c_char,
) -> *mut ZkLib
{
    // SAFETY: the glue of the lib hands its name as a NUL-ended text.
    let name = unsafe { CStr::from_ptr(name) }.to_owned();
    #[rustfmt::skip]
    let lib = Box::new(ZkLib { prog: program, leave, name, base: Cell::new(0),
        gate: Mutex::new(None), last: Mutex::new(None) });
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

/// The trap a lib of this runtime stopped on last: each static lib of a
/// program built with the same `zakac` shares the runtime.
///
/// # Returns
///
/// The trap, or NULL while no lib stopped on one.
#[unsafe(export_name = stamped!("last_trap"))]
pub extern "C" fn zk_last_trap() -> *const ZkTrapInfo { LATEST.load(Ordering::Acquire) }

/// Forget the trap of a lib, for its reset; with no call in progress, clear
/// too what the trap left half done: the closures lent to C calls, the
/// buffer the output goes to and the queues of the drops.
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
    // The reset holds the lock once; a deeper hold is a call in progress.
    ret_if!(crate::start::holds() > 1);
    #[cfg(not(target_os = "wasi"))]
    crate::callbacks::end_lends();
    crate::out::reset_sink();
    crate::seqs::reset_drops();
}

/// Free text or the cells of a list a lib gave back.
///
/// # Arguments
///
/// * `p` - The `char *` or the `ptr` of a list; NULL frees nothing.
#[unsafe(export_name = stamped!("free"))]
pub extern "C" fn zk_free(p: *mut c_void) { zk_raw_free(p) }
