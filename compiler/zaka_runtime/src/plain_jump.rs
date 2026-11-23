//! The guard of a thread and its jump on Windows on ARM64, for the code the JIT
//! writes. A `longjmp` of Windows on ARM64 unwinds each frame it leaves, and
//! the code of the JIT gives no table to unwind it with, so the guard keeps the
//! registers a call keeps, the stack pointer and the place to go on at, and the
//! jump restores them alone, as `longjmp` does on each other OS.
//!
//! The frame of the guard holds, from its stack pointer: `x29` and `x30`, then
//! `x19` to `x28`, then `d8` to `d15`, then the stack pointer and the address
//! the jump goes on at, which `lib_jump_set` keeps for the thread.

use std::arch::naked_asm;
use std::ffi::c_void;

use crate::libs::zk_lib_jump;
use crate::libs::zk_lib_jump_set;
use crate::start::zk_flush;
use crate::threads::ZkStep;

/// Run the step of a thread, keeping the place a trap of the thread jumps back
/// to.
///
/// # Arguments
///
/// * `step` - The step.
/// * `data` - The data block of the thread.
///
/// # Returns
///
/// 0, or 1 once a trap ended the thread.
#[unsafe(naked)]
pub(crate) unsafe extern "C" fn thread_guard(step: ZkStep, data: *mut c_void) -> i32
{
    naked_asm!(
        "stp x29, x30, [sp, #-176]!",
        "mov x29, sp",
        "stp x19, x20, [sp, #16]",
        "stp x21, x22, [sp, #32]",
        "stp x23, x24, [sp, #48]",
        "stp x25, x26, [sp, #64]",
        "stp x27, x28, [sp, #80]",
        "stp d8, d9, [sp, #96]",
        "stp d10, d11, [sp, #112]",
        "stp d12, d13, [sp, #128]",
        "stp d14, d15, [sp, #144]",
        "mov x19, x0",
        "mov x20, x1",
        // The place a jump goes on at: this stack pointer and label 2.
        "mov x9, sp",
        "adr x10, 2f",
        "stp x9, x10, [sp, #160]",
        "add x0, sp, #160",
        "bl {set}",
        "mov x0, x20",
        "blr x19",
        "bl {flush}",
        "mov w0, #0",
        "b 3f",
        "2:",
        "mov w0, #1",
        "3:",
        "ldp d14, d15, [sp, #144]",
        "ldp d12, d13, [sp, #128]",
        "ldp d10, d11, [sp, #112]",
        "ldp d8, d9, [sp, #96]",
        "ldp x27, x28, [sp, #80]",
        "ldp x25, x26, [sp, #64]",
        "ldp x23, x24, [sp, #48]",
        "ldp x21, x22, [sp, #32]",
        "ldp x19, x20, [sp, #16]",
        "ldp x29, x30, [sp], #176",
        "ret",
        set = sym zk_lib_jump_set,
        flush = sym zk_flush,
    )
}

/// Jump back to the guard of the thread of this OS thread, on a trap that ends
/// the thread alone: the stack pointer of the guard comes back, and its frame
/// restores the registers a call keeps.
///
/// # Arguments
///
/// * `trap` - The id of the trap, which the runtime keeps.
#[unsafe(naked)]
pub(crate) unsafe extern "C" fn thread_leave(trap: i32) -> !
{
    naked_asm!(
        "bl {jump}",
        "ldp x9, x10, [x0]",
        "mov sp, x9",
        "br x10",
        jump = sym zk_lib_jump,
    )
}
