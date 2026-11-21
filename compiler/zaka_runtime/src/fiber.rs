//! Fibers: a stack of its own for a light task, and the switch between it and
//! the stack of the OS thread that runs it. The switch keeps the registers a
//! call must keep (the callee-saved ones) on the stack it leaves, and takes
//! them back from the stack it goes to.
//!
//! The switch is a naked function of this crate, so its link name carries the
//! hash of the build: two builds of the runtime in one program never clash, as
//! the fixed names of an asm file would.

use std::ptr::null_mut;

use zakac_rust_sugars::ret_if;

/// The octets at the low end of a stack that no access can reach: more than
/// the frame of each function a light task can call (the compiler gives a
/// thread with a larger frame an OS thread), so a stack that runs full meets
/// them in place of writing past them.
const GUARD: usize = 64 * 1024;

/// A stack of its own, with [`GUARD`] octets at its low end that no access can
/// reach, so a stack that runs full stops the program in place of writing
/// past it.
pub(crate) struct Stack
{
    /// The low end of the mapping, the guard first.
    base: *mut u8,
    /// The octets of the mapping.
    size: usize,
    /// The octets of the guard.
    guard: usize,
}

impl Stack
{
    /// Map a stack of `size` octets and a guard page; the OS gives the pages
    /// as each is first used.
    ///
    /// # Arguments
    ///
    /// * `size` - The octets of the stack, a count of pages.
    ///
    /// # Returns
    ///
    /// The stack, or `None` when the OS has no room.
    pub(crate) fn new(size: usize) -> Option<Stack>
    {
        // SAFETY: `sysconf` reads a setting of the OS.
        let page =
            usize::try_from(unsafe { libc::sysconf(libc::_SC_PAGESIZE) }).ok()?;
        let guard = GUARD.next_multiple_of(page);
        let size = size.next_multiple_of(page) + guard;
        let prot = libc::PROT_READ | libc::PROT_WRITE;
        let flags = libc::MAP_PRIVATE | libc::MAP_ANON;
        // SAFETY: a new private mapping, which no other code reaches.
        let base = unsafe { libc::mmap(null_mut(), size, prot, flags, -1, 0) };
        ret_if!(base == libc::MAP_FAILED, None);
        // SAFETY: the low end of the mapping above.
        let guarded = unsafe { libc::mprotect(base, guard, libc::PROT_NONE) };
        let stack = Stack { base: base.cast(), size, guard };
        (guarded == 0).then_some(stack)
    }

    /// Whether an address lies in the guard of the stack.
    ///
    /// # Arguments
    ///
    /// * `at` - The address.
    ///
    /// # Returns
    ///
    /// `true` when it does.
    pub(crate) fn guards(&self, at: usize) -> bool
    {
        let base = self.base as usize;
        (base..base + self.guard).contains(&at)
    }

    /// The octets a task can use, the guard left out.
    ///
    /// # Returns
    ///
    /// The count.
    pub(crate) fn room(&self) -> usize { self.size - self.guard }

    /// The high end of the stack, where it starts: a stack grows down.
    ///
    /// # Returns
    ///
    /// The address, 16-aligned.
    fn top(&self) -> *mut u8
    {
        // SAFETY: the end of the mapping; the size is a count of pages.
        unsafe { self.base.add(self.size) }
    }
}

impl Drop for Stack
{
    fn drop(&mut self)
    {
        // SAFETY: the mapping `new` made, which no fiber runs on now.
        unsafe { libc::munmap(self.base.cast(), self.size) };
    }
}

/// The octets the switch keeps on a stack it leaves.
#[cfg(target_arch = "aarch64")]
const SAVED: usize = 0xa0;

/// The octets the switch keeps on a stack it leaves, its return address among
/// them.
#[cfg(target_arch = "x86_64")]
const SAVED: usize = 7 * 8;

/// Set a stack up for its first switch: the switch takes back the saved
/// registers of a call that never ran, and returns into [`entry`], which calls
/// `main` with `arg`.
///
/// # Arguments
///
/// * `stack` - The stack.
/// * `main` - The function the fiber runs; it never returns.
/// * `arg` - Its argument.
///
/// # Returns
///
/// The stack address a switch to the fiber goes to.
pub(crate) fn prepare(
    stack: &Stack, main: extern "C" fn(*mut u8) -> !, arg: *mut u8,
) -> *mut u8
{
    let top = stack.top() as usize;
    // aarch64: the stack is 16-aligned after the restore; x86_64: the
    // stack is 16-aligned once the `ret` pops the return address and `entry`
    // calls `main`.
    let sp = start_sp(top) as *mut usize;
    let slots = SAVED / 8;
    // SAFETY: the slots lie inside the stack, below its top.
    unsafe {
        (0..slots).for_each(|i| sp.add(i).write(0));
        place_start(sp, main, arg);
    }
    sp.cast()
}

/// Where the saved registers of a new fiber start.
///
/// # Arguments
///
/// * `top` - The top of the stack.
///
/// # Returns
///
/// The address.
#[cfg(target_arch = "aarch64")]
fn start_sp(top: usize) -> usize { top - SAVED }

/// Where the saved registers of a new fiber start.
///
/// # Arguments
///
/// * `top` - The top of the stack.
///
/// # Returns
///
/// The address.
#[cfg(target_arch = "x86_64")]
fn start_sp(top: usize) -> usize { top - 16 - SAVED }

/// Write the start of a new fiber into its saved registers: `main` and `arg`
/// in two callee-saved registers, and [`entry`] as the return address.
///
/// # Arguments
///
/// * `sp` - The saved registers.
/// * `main` - The function.
/// * `arg` - Its argument.
///
/// The caller makes sure that `sp` holds [`SAVED`] octets.
#[cfg(target_arch = "aarch64")]
unsafe fn place_start(sp: *mut usize, main: extern "C" fn(*mut u8) -> !, arg: *mut u8)
{
    // x19 holds the argument, x20 the function, x30 (the link register) the
    // entry; x29 (the frame pointer) is 0, the end of a backtrace.
    unsafe {
        sp.write(arg as usize);
        sp.add(1).write(main as usize);
        sp.add(11).write(entry as *const () as usize);
    }
}

/// Write the start of a new fiber into its saved registers: `main` and `arg`
/// in two callee-saved registers, and [`entry`] as the return address.
///
/// # Arguments
///
/// * `sp` - The saved registers.
/// * `main` - The function.
/// * `arg` - Its argument.
///
/// The caller makes sure that `sp` holds [`SAVED`] octets.
#[cfg(target_arch = "x86_64")]
unsafe fn place_start(sp: *mut usize, main: extern "C" fn(*mut u8) -> !, arg: *mut u8)
{
    // The pops go r15, r14, r13, r12, rbx, rbp: r12 holds the function, rbx
    // the argument; rbp is 0, the end of a backtrace; the return address
    // is the entry.
    unsafe {
        sp.add(3).write(main as usize);
        sp.add(4).write(arg as usize);
        sp.add(6).write(entry as *const () as usize);
    }
}

/// The first code of a new fiber: call the function with the argument, both
/// from callee-saved registers.
#[cfg(target_arch = "aarch64")]
#[unsafe(naked)]
extern "C" fn entry() { std::arch::naked_asm!("mov x0, x19", "blr x20", "brk #1") }

/// The first code of a new fiber: call the function with the argument, both
/// from callee-saved registers.
#[cfg(target_arch = "x86_64")]
#[unsafe(naked)]
extern "C" fn entry() { std::arch::naked_asm!("mov rdi, rbx", "call r12", "ud2") }

/// Switch stacks: keep the callee-saved registers on this stack, write its
/// address to `from`, and go on at `to`, a stack an earlier switch left or
/// [`prepare`] made.
///
/// # Arguments
///
/// * `from` - Where the address of this stack goes.
/// * `to` - The stack to go on at.
///
/// The caller makes sure that `to` is a stack that a switch left and none went
/// on at since, or a new one from [`prepare`].
#[cfg(target_arch = "aarch64")]
#[unsafe(naked)]
pub(crate) unsafe extern "C" fn switch(from: *mut *mut u8, to: *mut u8)
{
    std::arch::naked_asm!(
        "sub sp, sp, #0xa0",
        "stp x19, x20, [sp, #0x00]",
        "stp x21, x22, [sp, #0x10]",
        "stp x23, x24, [sp, #0x20]",
        "stp x25, x26, [sp, #0x30]",
        "stp x27, x28, [sp, #0x40]",
        "stp x29, x30, [sp, #0x50]",
        "stp d8, d9, [sp, #0x60]",
        "stp d10, d11, [sp, #0x70]",
        "stp d12, d13, [sp, #0x80]",
        "stp d14, d15, [sp, #0x90]",
        "mov x2, sp",
        "str x2, [x0]",
        "mov sp, x1",
        "ldp x19, x20, [sp, #0x00]",
        "ldp x21, x22, [sp, #0x10]",
        "ldp x23, x24, [sp, #0x20]",
        "ldp x25, x26, [sp, #0x30]",
        "ldp x27, x28, [sp, #0x40]",
        "ldp x29, x30, [sp, #0x50]",
        "ldp d8, d9, [sp, #0x60]",
        "ldp d10, d11, [sp, #0x70]",
        "ldp d12, d13, [sp, #0x80]",
        "ldp d14, d15, [sp, #0x90]",
        "add sp, sp, #0xa0",
        "ret",
    )
}

/// Switch stacks: keep the callee-saved registers on this stack, write its
/// address to `from`, and go on at `to`, a stack an earlier switch left or
/// [`prepare`] made.
///
/// # Arguments
///
/// * `from` - Where the address of this stack goes.
/// * `to` - The stack to go on at.
///
/// The caller makes sure that `to` is a stack that a switch left and none went
/// on at since, or a new one from [`prepare`].
#[cfg(target_arch = "x86_64")]
#[unsafe(naked)]
pub(crate) unsafe extern "C" fn switch(from: *mut *mut u8, to: *mut u8)
{
    std::arch::naked_asm!(
        "push rbp",
        "push rbx",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "mov [rdi], rsp",
        "mov rsp, rsi",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbx",
        "pop rbp",
        "ret",
    )
}

#[cfg(test)]
mod tests
{
    use std::ptr::null_mut;

    use super::*;

    /// What the test fiber and its OS thread pass each other.
    struct Pair
    {
        /// The stack of the OS thread, while the fiber runs.
        back: *mut u8,
        /// The stack of the fiber, while the OS thread runs.
        fiber: *mut u8,
        /// The rounds the fiber counted.
        count: u32,
    }

    /// The fiber: count three rounds, switching back after each, then wait
    /// there.
    extern "C" fn count(arg: *mut u8) -> !
    {
        let pair = arg.cast::<Pair>();
        loop
        {
            // SAFETY: the pair outlives the fiber, and the OS thread waits
            // while the fiber runs.
            unsafe {
                (*pair).count += 1;
                switch(&raw mut (*pair).fiber, (*pair).back);
            }
        }
    }

    #[test]
    fn a_fiber_and_its_os_thread_take_turns()
    {
        let stack = Stack::new(64 * 1024).expect("a stack");
        let mut pair = Pair { back: null_mut(), fiber: null_mut(), count: 0 };
        let arg = (&raw mut pair).cast();
        pair.fiber = prepare(&stack, count, arg);
        let floats = [1.5f64, 2.5, 3.5];
        for round in 1..=3
        {
            // SAFETY: the fiber switches back after each round.
            unsafe { switch(&raw mut pair.back, pair.fiber) };
            assert_eq!(pair.count, round);
        }
        // The registers a call keeps came back whole.
        assert_eq!(floats.iter().sum::<f64>(), 7.5);
    }
}
