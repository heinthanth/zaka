//! The calls in progress that each thread keeps for itself. Generated C
//! reaches them through [`zk_ctx`], once at the start of each function;
//! Cranelift code and the runtime reach them through the same function.

use std::cell::UnsafeCell;
use std::ptr::null_mut;

use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use crate::program::ZkFrame;
use crate::program::prog;
use crate::program::prog_ptr;
use crate::program::tracks;

/// The calls in progress on one thread.
#[repr(C)]
pub struct ZkCtx
{
    /// The count of Zaka calls in progress on the thread, which the inline
    /// `ZK_ENTER` and `ZK_LEAVE` keep.
    pub depth: u32,
    /// The calls in progress, main at 0 and the last at `depth`; NULL when
    /// the build keeps none.
    pub frames: *mut ZkFrame,
    /// The rows `frames` has room for.
    pub room: usize,
}

thread_local! {
    /// The calls in progress on this thread. No destructor runs on it, so the
    /// collection at the end of the program still reaches it.
    static CTX: UnsafeCell<ZkCtx> =
        const { UnsafeCell::new(ZkCtx { depth: 0, frames: null_mut(), room: 0 }) };
}

/// The calls in progress on the calling thread, with room in the frame table
/// for the depth limit plus main when the build keeps the calls.
///
/// # Returns
///
/// The context of the thread; it lives as long as the thread.
#[unsafe(export_name = stamped!("ctx"))]
pub extern "C" fn zk_ctx() -> *mut ZkCtx
{
    let at = CTX.with(UnsafeCell::get);
    // SAFETY: each thread reaches its own context, and no reference to it
    // outlives this call.
    let ctx = unsafe { &mut *at };
    let want = wanted_rows();
    do_if!(ctx.room < want, grow(ctx, want));
    at
}

/// The rows the frame table of a thread needs: the depth limit plus main,
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

/// Give the frame table of a thread more rows, keeping the calls in progress:
/// a C lib whose limit is deeper than the one before it needs them.
///
/// # Arguments
///
/// * `ctx` - The context of the thread.
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
    // table of a thread of C's own that ends; a job frees its own as it ends.
    ctx.frames = Box::leak(frames).as_mut_ptr();
    ctx.room = rows;
}

/// Free the frame table of the calling thread, which ends: a job's, once its
/// step returned.
pub(crate) fn release_frames()
{
    let at = CTX.with(UnsafeCell::get);
    // SAFETY: each thread reaches its own context, and no reference to it
    // outlives this call.
    let ctx = unsafe { &mut *at };
    ret_if!(ctx.room == 0);
    // SAFETY: the table is the boxed slice of `room` rows `grow` leaked.
    let table = unsafe {
        Box::from_raw(std::ptr::slice_from_raw_parts_mut(ctx.frames, ctx.room))
    };
    drop(table);
    (ctx.frames, ctx.room) = (null_mut(), 0);
}

/// The count of calls in progress on the calling thread.
///
/// # Returns
///
/// The depth.
pub fn depth() -> u32
{
    // SAFETY: the context of the thread lives as long as the thread.
    unsafe { (*zk_ctx()).depth }
}

/// Set the count of calls in progress on the calling thread.
///
/// # Arguments
///
/// * `depth` - The depth.
pub fn set_depth(depth: u32)
{
    // SAFETY: the context of the thread lives as long as the thread.
    unsafe { (*zk_ctx()).depth = depth }
}
