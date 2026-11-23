//! The calls in progress that each OS thread keeps for itself. Generated C
//! reaches them through [`zk_ctx`], once at the start of each function;
//! Cranelift code and the runtime reach them through the same function.

use std::cell::Cell;
use std::cell::UnsafeCell;
use std::ptr::null_mut;

use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::native::NO_SITE;
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
    /// The site of the expression whose call of the runtime is in progress,
    /// for a trap inside that call; `NO_SITE` outside such a call. It is kept
    /// here, not on the OS thread, since a light task that waits goes on on
    /// another OS thread.
    pub call_site: u32,
}

thread_local! {
    /// The calls in progress on this OS thread. No destructor runs on it, so the
    /// collection at the end of the program still reaches it.
    static CTX: UnsafeCell<ZkCtx> =
        const { UnsafeCell::new(ZkCtx { depth: 0, frames: null_mut(), room: 0, call_site: NO_SITE }) };
    /// The context of the thread this OS thread runs now, when it is not
    /// [`CTX`]: a thread holds its own, so the context that the code of a call
    /// keeps is still the thread's when the thread goes on on another OS
    /// thread.
    static ACTIVE: Cell<*mut ZkCtx> = const { Cell::new(null_mut()) };
    /// Frees the frame table of [`CTX`] as the OS thread ends; set up the first
    /// time the table is made, so an OS thread of C that ends frees its own.
    static OWN_TABLE: OwnTable = const { OwnTable };
}

/// The frame table of the OS thread's own context, which its drop frees.
struct OwnTable;

impl Drop for OwnTable
{
    fn drop(&mut self)
    {
        // SAFETY: the OS thread ends, and no call of it still runs.
        _ = CTX.try_with(|c| free_table(unsafe { &mut *c.get() }));
    }
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
/// # Arguments
///
/// * `rows` - The rows of its frame table, made now; 0 to make it at its first
///   use, with room for the depth limit.
///
/// # Returns
///
/// The context.
pub(crate) fn new_ctx(rows: usize) -> *mut ZkCtx
{
    let mut ctx =
        Box::new(ZkCtx { depth: 0, frames: null_mut(), room: 0, call_site: NO_SITE });
    do_if!(rows > 0, grow(&mut ctx, rows));
    Box::into_raw(ctx)
}

/// The rows the frame table of a light task needs: a task calls no function
/// again, so its calls go no deeper than the count of functions, under the
/// depth limit.
///
/// # Returns
///
/// The count, 0 when no frame is kept.
#[cfg(zk_tasks)]
pub(crate) fn task_rows() -> usize
{
    let want = wanted_rows();
    ret_if!(want == 0, 0);
    let count = prog().fn_count;
    tern!(count == 0, want, want.min(count + 2))
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
    let mut ctx = unsafe { Box::from_raw(ctx) };
    free_table(&mut ctx);
}

/// Free the frame table of a context, which then has none.
///
/// # Arguments
///
/// * `ctx` - The context.
fn free_table(ctx: &mut ZkCtx)
{
    ret_if!(ctx.room == 0);
    // SAFETY: the table is the boxed slice of `room` rows `grow` leaked, and
    // the code of each call reads `frames` again at each use, so none holds the
    // table past this.
    drop(unsafe {
        Box::from_raw(std::ptr::slice_from_raw_parts_mut(ctx.frames, ctx.room))
    });
    ctx.frames = null_mut();
    ctx.room = 0;
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
    // The table of a thread is made once, to the size it needs; the OS
    // thread's own grows for a C lib whose limit is deeper.
    let own = ACTIVE.get().is_null();
    ret_if!(ctx.room >= want || !(own || ctx.room == 0), at);
    // The table of the OS thread's own context goes when the OS thread ends.
    do_if!(own, _ = OWN_TABLE.try_with(|_| ()));
    grow(ctx, want);
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
    // SAFETY: the table before holds `room` rows.
    do_if!(
        ctx.room > 0,
        frames[..ctx.room].copy_from_slice(unsafe {
            std::slice::from_raw_parts(ctx.frames, ctx.room)
        })
    );
    free_table(ctx);
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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn a_growth_keeps_the_calls_and_frees_the_table_before()
    {
        let mut ctx =
            ZkCtx { depth: 2, frames: null_mut(), room: 0, call_site: NO_SITE };
        grow(&mut ctx, 3);
        // SAFETY: the table has 3 rows.
        unsafe { (*ctx.frames.add(2)).func = 7 };
        grow(&mut ctx, 8);
        assert_eq!(ctx.room, 8);
        // SAFETY: the table has 8 rows, the first 3 copied.
        assert_eq!(unsafe { (*ctx.frames.add(2)).func }, 7);
        free_table(&mut ctx);
        assert!(ctx.frames.is_null() && ctx.room == 0);
    }
}
