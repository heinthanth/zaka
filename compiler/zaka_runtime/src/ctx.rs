//! The calls in progress that each OS thread keeps for itself. Generated C
//! reaches them through [`zk_ctx`], once at the start of each function;
//! Cranelift code and the runtime reach them through the same function.

use std::cell::Cell;
use std::cell::UnsafeCell;
use std::ptr::null_mut;

use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::program::ZkFrame;
use crate::program::prog;
use crate::program::prog_ptr;
use crate::program::tracks;

/// The calls in progress on one OS thread.
#[repr(C)]
pub struct ZkCtx
{
    /// The count of Zaka calls in progress on the OS thread, which the inline
    /// `ZK_ENTER` and `ZK_LEAVE` keep.
    pub depth: u32,
    /// The calls in progress, main at 0 and the last at `depth`; NULL when
    /// the build keeps none.
    pub frames: *mut ZkFrame,
    /// The rows `frames` has room for.
    pub room: usize,
}

thread_local! {
    /// The calls in progress on this OS thread. No destructor runs on it, so the
    /// collection at the end of the program still reaches it.
    static CTX: UnsafeCell<ZkCtx> =
        const { UnsafeCell::new(ZkCtx { depth: 0, frames: null_mut(), room: 0 }) };
    /// The context of the thread this OS thread runs now, when it is not
    /// [`CTX`]: a thread holds its own, so the context that the code of a call
    /// keeps is still the thread's when the thread goes on on another OS
    /// thread.
    static ACTIVE: Cell<*mut ZkCtx> = const { Cell::new(null_mut()) };
}

/// The context the calling OS thread runs with: the one of its thread, or its
/// own.
///
/// # Returns
///
/// The context.
fn active() -> *mut ZkCtx
{
    let thread = ACTIVE.get();
    tern!(thread.is_null(), CTX.with(UnsafeCell::get), thread)
}

/// Put the context of a thread in place of the one the calling OS thread runs
/// with, and give back the one before; NULL stands for the OS thread's own.
///
/// # Arguments
///
/// * `ctx` - The context to run with, which gets the one before.
pub(crate) fn swap_active(ctx: &mut *mut ZkCtx) { *ctx = ACTIVE.replace(*ctx); }

/// A new context for a thread, which [`free_ctx`] frees.
///
/// # Returns
///
/// The context.
pub(crate) fn new_ctx() -> *mut ZkCtx
{
    Box::into_raw(Box::new(ZkCtx { depth: 0, frames: null_mut(), room: 0 }))
}

/// Free the context of a thread and its frame table, once the thread ended and
/// no OS thread runs with it.
///
/// # Arguments
///
/// * `ctx` - The context, from [`new_ctx`].
pub(crate) fn free_ctx(ctx: *mut ZkCtx)
{
    ret_if!(ctx.is_null());
    // SAFETY: `new_ctx` boxed it, and no OS thread runs with it now.
    let ctx = unsafe { Box::from_raw(ctx) };
    ret_if!(ctx.room == 0);
    // SAFETY: the table is the boxed slice of `room` rows `grow` leaked.
    drop(unsafe {
        Box::from_raw(std::ptr::slice_from_raw_parts_mut(ctx.frames, ctx.room))
    });
}

/// The calls in progress on the calling OS thread, with room in the frame table
/// for the depth limit plus main when the build keeps the calls.
///
/// # Returns
///
/// The context of the OS thread; it lives as long as the OS thread.
#[unsafe(export_name = stamped!("ctx"))]
pub extern "C" fn zk_ctx() -> *mut ZkCtx
{
    let at = active();
    // SAFETY: each OS thread reaches the context it runs with, and no
    // reference to it outlives this call.
    let ctx = unsafe { &mut *at };
    let want = wanted_rows();
    do_if!(ctx.room < want, grow(ctx, want));
    at
}

/// The rows the frame table of an OS thread needs: the depth limit plus main,
/// when a program is running and its build keeps the calls.
///
/// # Returns
///
/// The count, 0 when no frame is kept.
fn wanted_rows() -> usize
{
    // `zk_start` and the start of a lib set the table before the program
    // runs; a NULL table means no program runs yet.
    let started = !prog_ptr().is_null();
    ret_if!(!started || !tracks(), 0);
    prog().max_depth as usize + 1
}

/// Give the frame table of an OS thread more rows, keeping the calls in
/// progress: a C lib whose limit is deeper than the one before it needs them.
///
/// # Arguments
///
/// * `ctx` - The context of the OS thread.
/// * `rows` - The rows it needs.
fn grow(ctx: &mut ZkCtx, rows: usize)
{
    let mut frames = vec![ZkFrame::default(); rows].into_boxed_slice();
    // SAFETY: the table before holds `room` rows, which live to the end of
    // the program.
    do_if!(
        ctx.room > 0,
        frames[..ctx.room].copy_from_slice(unsafe {
            std::slice::from_raw_parts(ctx.frames, ctx.room)
        })
    );
    // shortcut: the table before a growth is never freed, and neither is the
    // table of an OS thread of C's own that ends; a thread frees its own as it
    // ends.
    ctx.frames = Box::leak(frames).as_mut_ptr();
    ctx.room = rows;
}

/// The count of calls in progress on the calling OS thread.
///
/// # Returns
///
/// The depth.
pub fn depth() -> u32
{
    // SAFETY: the context of the OS thread lives as long as the OS thread.
    unsafe { (*zk_ctx()).depth }
}

/// Set the count of calls in progress on the calling thread.
///
/// # Arguments
///
/// * `depth` - The depth.
pub fn set_depth(depth: u32)
{
    // SAFETY: the context of the OS thread lives as long as the OS thread.
    unsafe { (*zk_ctx()).depth = depth }
}
