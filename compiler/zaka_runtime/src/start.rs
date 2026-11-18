//! The start and the end of a run.

use std::ffi::CStr;
use std::ffi::c_char;

use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use crate::out::flush_out;
use crate::program::ZkProgram;
use crate::program::make_frames;
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
#[unsafe(no_mangle)]
pub extern "C" fn zk_start(argc: i32, argv: *mut *mut c_char, program: *const ZkProgram)
{
    // SAFETY: `main` calls this first, on the one thread of the program.
    unsafe { zk_prog = program };
    make_frames();
    ret_if!(argc <= 0 || argv.is_null());
    // SAFETY: `argv` holds `argc` NUL-ended arguments.
    let first = unsafe { *argv };
    if !first.is_null()
    {
        let path = unsafe { CStr::from_ptr(first) }.to_bytes();
        do_if!(!path.is_empty(), set_name(path));
    }
}

/// Write out what is buffered, as a C lib does at the end of each call.
#[unsafe(no_mangle)]
pub extern "C" fn zk_flush() { flush_out(); }

/// End the run: write out what is buffered.
///
/// # Arguments
///
/// * `status` - The exit status of the program.
///
/// # Returns
///
/// The status, for `main` to return.
#[unsafe(no_mangle)]
pub extern "C" fn zk_finish(status: i32) -> i32
{
    flush_out();
    status
}
