//! The start and the end of a run.

use std::ffi::CStr;
use std::ffi::c_char;
use std::sync::Condvar;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::thread::ThreadId;

use zakac_rust_sugars::do_if;
use zakac_rust_sugars::do_while;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::ctx::zk_ctx;
use crate::out::flush_out;
use crate::program::ZkProgram;
use crate::program::zk_prog;
use crate::traps::set_name;

/// Get the run going: keep the table of the program, make room for the calls
/// in progress, and keep the name of the executable for trap lines.
///
/// # Arguments
///
/// * `argc` - The argument count `main` was given.
/// * `argv` - The arguments `main` was given; the first is the executable.
/// * `program` - The table of the program, which the runtime reads from now on.
#[unsafe(export_name = stamped!("start"))]
pub extern "C" fn zk_start(argc: i32, argv: *mut *mut c_char, program: *const ZkProgram)
{
    // SAFETY: `main` calls this first, on the one OS thread of the program.
    unsafe { zk_prog = program };
    zk_ctx();
    crate::threads::mark_main();
    ret_if!(argc <= 0 || argv.is_null());
    // SAFETY: `argv` holds `argc` NUL-ended arguments.
    let first = unsafe { *argv };
    if !first.is_null()
    {
        let path = unsafe { CStr::from_ptr(first) }.to_bytes();
        do_if!(!path.is_empty(), set_name(path));
    }
}

/// The OS thread whose call of a C lib runs now, and how deep its calls go: C
/// that a call calls can call the lib again on the same OS thread.
static HELD: Mutex<(Option<ThreadId>, u32)> = Mutex::new((None, 0));

/// Wakes the OS threads that wait for the lib once no call of it runs.
static FREED: Condvar = Condvar::new();

/// Wait until no other OS thread runs a call of the C lib, then hold it: the
/// heap, the globals and the frames of the runtime serve one call at a time.
/// The OS thread that holds it can call again, as C that a call calls does.
#[unsafe(export_name = stamped!("lib_lock"))]
pub extern "C" fn zk_lib_lock()
{
    let me = std::thread::current().id();
    let mut held = HELD.lock().unwrap_or_else(PoisonError::into_inner);
    do_while!(
        held.0.is_some_and(|t| t != me),
        held = FREED.wait(held).unwrap_or_else(PoisonError::into_inner)
    );
    *held = (Some(me), held.1 + 1);
}

/// End the hold of [`zk_lib_lock`] for one call: once the calls of the
/// OS thread all end, the next OS thread that waits goes on.
#[unsafe(export_name = stamped!("lib_unlock"))]
pub extern "C" fn zk_lib_unlock()
{
    let mut held = HELD.lock().unwrap_or_else(PoisonError::into_inner);
    held.1 = held.1.saturating_sub(1);
    ret_if!(held.1 > 0);
    held.0 = None;
    FREED.notify_one();
}

/// Let go of the lock of the libs at each depth this OS thread holds it, for a
/// call to C that lets other OS threads into the libs while it runs.
///
/// # Returns
///
/// The depth it held.
pub(crate) fn let_go_lock() -> u32
{
    let me = std::thread::current().id();
    let mut held = HELD.lock().unwrap_or_else(PoisonError::into_inner);
    // An OS thread that holds no lock, such as a thread's, has none to let go
    // of.
    ret_if!(held.0 != Some(me), 0);
    let depth = held.1;
    *held = (None, 0);
    FREED.notify_all();
    depth
}

/// Take the lock of the libs back at the depth a call to C let go of, once
/// no other OS thread runs a call of a lib.
///
/// # Arguments
///
/// * `depth` - The depth, as [`let_go_lock`] gave it.
pub(crate) fn take_lock(depth: u32)
{
    ret_if!(depth == 0);
    let me = std::thread::current().id();
    let mut held = HELD.lock().unwrap_or_else(PoisonError::into_inner);
    do_while!(
        held.0.is_some_and(|t| t != me),
        held = FREED.wait(held).unwrap_or_else(PoisonError::into_inner)
    );
    *held = (Some(me), held.1 + depth);
}

/// Write out what is buffered, as a C lib does at the end of each call.
#[unsafe(export_name = stamped!("flush"))]
pub extern "C" fn zk_flush() { flush_out(); }

/// End the run: wait for each thread, then write out what is buffered. A thread
/// that stopped at a trap no wait saw makes the status 1.
///
/// # Arguments
///
/// * `status` - The exit status of the program.
///
/// # Returns
///
/// The status, for `main` to return.
#[unsafe(export_name = stamped!("finish"))]
pub extern "C" fn zk_finish(status: i32) -> i32
{
    let lost = crate::threads::end();
    flush_out();
    tern!(lost && status == 0, 1, status)
}
