//! What a thread of the program keeps for itself while it runs: its calls in
//! progress, where a trap of it jumps, its number, and the sink and the line of
//! its output. The OS thread that runs a thread holds these in its own cells; a
//! thread holds them in a [`Local`] while no OS thread runs it, and a swap
//! moves them between the two, so a thread can go on on another OS thread.

use std::ffi::c_void;
use std::ptr::null_mut;

use crate::ctx::ZkCtx;
use crate::ctx::free_ctx;
use crate::ctx::new_ctx;
use crate::out::ZkBuf;
use crate::threads::Own;

/// What a thread keeps while no OS thread runs it, or, while one does, what
/// that OS thread kept before.
pub(crate) struct Local
{
    /// The calls in progress, a context of the thread's own; NULL for the
    /// context of the OS thread itself.
    ctx: *mut ZkCtx,
    /// Where a trap jumps.
    jump: *mut c_void,
    /// The leave, the trap and the number of the thread.
    own: Own,
    /// The sink of an interpolation.
    sink: *mut ZkBuf,
    /// The line it prints, with no end so far.
    line: Vec<u8>,
}

// SAFETY: a thread that no OS thread runs reaches none of what these point at,
// so the OS thread that runs it next takes them over whole.
unsafe impl Send for Local {}

impl Local
{
    /// What a new thread starts with: a context of its own and no output
    /// held.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of the thread.
    /// * `leave` - The jump back to its guard and its number, `None` when a
    ///   trap of it stops the whole program.
    ///
    /// # Returns
    ///
    /// The cells.
    pub(crate) fn of_thread(
        n: usize, leave: Option<(extern "C" fn(i32), usize)>,
    ) -> Local
    {
        #[rustfmt::skip]
        let local = Local { ctx: new_ctx(), jump: null_mut(), own: (leave, 0, n),
            sink: null_mut(), line: Vec::new() };
        local
    }

    /// Swap these cells with those of the calling OS thread: a thread that
    /// starts or goes on puts its own in place, and the swap back as it stops
    /// gives the OS thread its own again.
    pub(crate) fn swap(&mut self)
    {
        crate::ctx::swap_active(&mut self.ctx);
        crate::libs::swap_jump(&mut self.jump);
        crate::threads::swap_own(&mut self.own);
        crate::out::swap_sink(&mut self.sink, &mut self.line);
    }

    /// Free what the thread held, once it ended and swapped back.
    pub(crate) fn free(self) { free_ctx(self.ctx); }
}
