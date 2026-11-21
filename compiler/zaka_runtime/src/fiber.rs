//! Fibers: a stack of its own for a light task, and the switch between it and
//! the stack of the OS thread that runs it.
//!
//! On unix the switch keeps the registers a call must keep (the callee-saved
//! ones) on the stack it leaves, and takes them back from the stack it goes
//! to. It is a naked function of this crate, so its link name holds the
//! hash of the build: two builds of the runtime in one program never clash, as
//! the fixed names of an asm file would. On Windows a fiber of the OS stands
//! for a task, so the OS keeps the bounds of the stack it reads in each of
//! its calls.
//!
//! Each OS thread of the pool calls [`enter_thread`] first, and the pool calls
//! [`catch_overflow`] once: a stack of a task that runs full then stops the
//! program with the trap `stack full`.

#[cfg(unix)]
pub(crate) use unix::*;
#[cfg(windows)]
pub(crate) use windows::*;

/// The stacks and the switch on unix.
#[cfg(unix)]
mod unix
{
    use std::ffi::c_void;
    use std::ptr::null_mut;

    use zakac_rust_sugars::do_if;
    use zakac_rust_sugars::ret_if;

    /// The octets at the low end of a stack that no access can reach: more than
    /// the frame of each function a light task can call (the compiler gives a
    /// thread with a larger frame an OS thread), so a stack that runs full
    /// meets them in place of writing past them.
    const GUARD: usize = 64 * 1024;

    /// A stack of its own, with [`GUARD`] octets at its low end that no access
    /// can reach, so a stack that runs full stops the program in place of
    /// writing past it.
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
        /// Map a stack of `size` octets and a guard page; the OS gives the
        /// pages as each is first used.
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

    /// The octets the switch keeps on a stack it leaves, its return address
    /// among them.
    #[cfg(target_arch = "x86_64")]
    const SAVED: usize = 7 * 8;

    /// Set a stack up for its first switch: the switch takes back the saved
    /// registers of a call that never ran, and returns into [`entry`], which
    /// calls `main` with `arg`.
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
        // stack is 16-aligned once the `ret` pops the return address and
        // `entry` calls `main`.
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

    /// Write the start of a new fiber into its saved registers: `main` and
    /// `arg` in two callee-saved registers, and [`entry`] as the return
    /// address.
    ///
    /// # Arguments
    ///
    /// * `sp` - The saved registers.
    /// * `main` - The function.
    /// * `arg` - Its argument.
    ///
    /// The caller makes sure that `sp` holds [`SAVED`] octets.
    #[cfg(target_arch = "aarch64")]
    unsafe fn place_start(
        sp: *mut usize, main: extern "C" fn(*mut u8) -> !, arg: *mut u8,
    )
    {
        // x19 holds the argument, x20 the function, x30 (the link register) the
        // entry; x29 (the frame pointer) is 0, the end of a backtrace.
        unsafe {
            sp.write(arg as usize);
            sp.add(1).write(main as usize);
            sp.add(11).write(entry as *const () as usize);
        }
    }

    /// Write the start of a new fiber into its saved registers: `main` and
    /// `arg` in two callee-saved registers, and [`entry`] as the return
    /// address.
    ///
    /// # Arguments
    ///
    /// * `sp` - The saved registers.
    /// * `main` - The function.
    /// * `arg` - Its argument.
    ///
    /// The caller makes sure that `sp` holds [`SAVED`] octets.
    #[cfg(target_arch = "x86_64")]
    unsafe fn place_start(
        sp: *mut usize, main: extern "C" fn(*mut u8) -> !, arg: *mut u8,
    )
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
    /// The caller makes sure that `to` is a stack that a switch left and none
    /// went on at since, or a new one from [`prepare`].
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
    /// The caller makes sure that `to` is a stack that a switch left and none
    /// went on at since, or a new one from [`prepare`].
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

    /// The signals a fault on a guard can raise: SIGSEGV, and SIGBUS on macOS.
    const FAULTS: [libc::c_int; 2] = [libc::SIGSEGV, libc::SIGBUS];

    /// The handlers of [`FAULTS`] before ours, which a fault off the guard of a
    /// task goes on to; written once, before ours takes their place.
    static mut BEFORE: std::mem::MaybeUninit<[libc::sigaction; 2]> =
        std::mem::MaybeUninit::uninit();

    /// The octets of the signal stack of an OS thread of the pool.
    const SIGNAL_STACK: usize = 128 * 1024;

    /// Put our handler on the faults of a guard, once, as the pool starts.
    pub(crate) fn catch_overflow()
    {
        // SAFETY: the pool starts once, under its lock, so no handler reads
        // `BEFORE` while it is written; `sigaction` reads and writes these.
        unsafe {
            let before = (&raw mut BEFORE).cast::<libc::sigaction>();
            let mut ours: libc::sigaction = std::mem::zeroed();
            ours.sa_sigaction = on_fault as *const () as libc::sighandler_t;
            ours.sa_flags = libc::SA_SIGINFO | libc::SA_ONSTACK;
            libc::sigemptyset(&raw mut ours.sa_mask);
            for (i, sig) in FAULTS.iter().enumerate()
            {
                libc::sigaction(*sig, &ours, before.add(i));
            }
        }
    }

    /// Give the calling OS thread a stack of its own for signals: a fault on
    /// the guard of a task comes while the stack of the task is full.
    pub(crate) fn enter_thread()
    {
        // SAFETY: a new mapping, which this OS thread alone uses, for good.
        unsafe {
            let prot = libc::PROT_READ | libc::PROT_WRITE;
            let flags = libc::MAP_PRIVATE | libc::MAP_ANON;
            let at = libc::mmap(null_mut(), SIGNAL_STACK, prot, flags, -1, 0);
            ret_if!(at == libc::MAP_FAILED);
            let stack = libc::stack_t { ss_sp: at, ss_size: SIGNAL_STACK, ss_flags: 0 };
            libc::sigaltstack(&stack, null_mut());
        }
    }

    /// The handler of a fault: one on the guard of the running task stops the
    /// program with the trap `stack full`; each other goes on to the handler
    /// before ours.
    ///
    /// # Arguments
    ///
    /// * `sig` - The signal.
    /// * `info` - What the OS tells of it.
    /// * `ctx` - The state of the OS thread.
    extern "C" fn on_fault(
        sig: libc::c_int, info: *mut libc::siginfo_t, ctx: *mut c_void,
    )
    {
        // SAFETY: the OS hands a valid `info`; the running task lives in its
        // box.
        unsafe {
            let at = (*info).si_addr() as usize;
            let room = crate::tasks::overflowed(Some(at));
            do_if!(let Some(room) = room, crate::traps::stack_full_from_signal(room));
            pass_on(sig, info, ctx);
        }
    }

    /// Hand a fault to the handler that was there before ours, or, for none,
    /// put the action of the OS back, so the fault comes again with it.
    ///
    /// # Arguments
    ///
    /// * `sig` - The signal.
    /// * `info` - What the OS tells of it.
    /// * `ctx` - The state of the OS thread.
    ///
    /// The caller makes sure that it runs in a handler of a signal of
    /// [`FAULTS`].
    unsafe fn pass_on(sig: libc::c_int, info: *mut libc::siginfo_t, ctx: *mut c_void)
    {
        let i = usize::from(sig == libc::SIGBUS);
        // SAFETY: written before ours took their place.
        let before = unsafe { &*(&raw const BEFORE).cast::<libc::sigaction>().add(i) };
        let handler = before.sa_sigaction;
        let plain = handler == libc::SIG_DFL || handler == libc::SIG_IGN;
        if plain
        {
            // SAFETY: the action of the OS comes back; the fault comes again.
            unsafe { libc::signal(sig, libc::SIG_DFL) };
            return;
        }
        type Full = extern "C" fn(libc::c_int, *mut libc::siginfo_t, *mut c_void);
        type Bare = extern "C" fn(libc::c_int);
        // SAFETY: the handler before ours takes the arguments its flags tell.
        unsafe {
            match before.sa_flags & libc::SA_SIGINFO != 0
            {
                true => std::mem::transmute::<libc::sighandler_t, Full>(handler)(
                    sig, info, ctx,
                ),
                false => std::mem::transmute::<libc::sighandler_t, Bare>(handler)(sig),
            }
        }
    }
}

/// The fibers of Windows.
#[cfg(windows)]
mod windows
{
    use std::cell::Cell;
    use std::ffi::c_void;
    use std::ptr::null;
    use std::ptr::null_mut;

    use windows_sys::Win32::Foundation::EXCEPTION_STACK_OVERFLOW;
    use windows_sys::Win32::System::Diagnostics::Debug::AddVectoredExceptionHandler;
    use windows_sys::Win32::System::Diagnostics::Debug::EXCEPTION_CONTINUE_SEARCH;
    use windows_sys::Win32::System::Diagnostics::Debug::EXCEPTION_POINTERS;
    use windows_sys::Win32::System::Threading::ConvertThreadToFiber;
    use windows_sys::Win32::System::Threading::CreateFiberEx;
    use windows_sys::Win32::System::Threading::DeleteFiber;
    use windows_sys::Win32::System::Threading::SetThreadStackGuarantee;
    use windows_sys::Win32::System::Threading::SwitchToFiber;
    use windows_sys::Win32::System::WindowsProgramming::FIBER_FLAG_FLOAT_SWITCH;
    use zakac_rust_sugars::do_if;

    /// The octets each task keeps for the handler of an overflow of its stack,
    /// which runs on that stack.
    const GUARANTEE: u32 = 32 * 1024;

    /// The stack of a task: a fiber of the OS, which [`prepare`] makes.
    pub(crate) struct Stack
    {
        /// The fiber, once [`prepare`] made it.
        fiber: Cell<*mut c_void>,
        /// The octets of its stack.
        size: usize,
    }

    impl Stack
    {
        /// A stack of `size` octets, which [`prepare`] makes.
        ///
        /// # Arguments
        ///
        /// * `size` - The octets of the stack.
        ///
        /// # Returns
        ///
        /// The stack.
        pub(crate) fn new(size: usize) -> Option<Stack>
        {
            Some(Stack { fiber: Cell::new(null_mut()), size })
        }

        /// Whether an address lies in the guard of the stack: Windows tells an
        /// overflow through its code, not its address.
        ///
        /// # Arguments
        ///
        /// * `_at` - The address.
        ///
        /// # Returns
        ///
        /// `false`.
        pub(crate) fn guards(&self, _at: usize) -> bool { false }

        /// The octets a task can use.
        ///
        /// # Returns
        ///
        /// The count.
        pub(crate) fn room(&self) -> usize { self.size }
    }

    impl Drop for Stack
    {
        fn drop(&mut self)
        {
            let fiber = self.fiber.get();
            // SAFETY: the fiber `prepare` made, which no OS thread runs now.
            do_if!(!fiber.is_null(), unsafe { DeleteFiber(fiber) });
        }
    }

    /// What the first code of a fiber calls.
    struct Start
    {
        /// The function.
        main: extern "C" fn(*mut u8) -> !,
        /// Its argument.
        arg: *mut u8,
    }

    /// Make the fiber of a stack, which calls `main` with `arg` at its first
    /// switch.
    ///
    /// # Arguments
    ///
    /// * `stack` - The stack.
    /// * `main` - The function the fiber runs; it never returns.
    /// * `arg` - Its argument.
    ///
    /// # Returns
    ///
    /// The fiber, which a switch goes to; NULL when the OS has no room.
    pub(crate) fn prepare(
        stack: &Stack, main: extern "C" fn(*mut u8) -> !, arg: *mut u8,
    ) -> *mut u8
    {
        let start = Box::into_raw(Box::new(Start { main, arg }));
        let flags = FIBER_FLAG_FLOAT_SWITCH;
        // SAFETY: `begin` takes the box back at the first switch.
        let fiber =
            unsafe { CreateFiberEx(0, stack.size, flags, Some(begin), start.cast()) };
        stack.fiber.set(fiber);
        fiber.cast()
    }

    /// The first code of a fiber: keep room for the handler of an overflow,
    /// then call the function of the fiber.
    ///
    /// # Arguments
    ///
    /// * `param` - The box of [`Start`].
    unsafe extern "system" fn begin(param: *mut c_void)
    {
        // SAFETY: `prepare` boxed it for this fiber.
        let Start { main, arg } = *unsafe { Box::from_raw(param.cast::<Start>()) };
        let mut room = GUARANTEE;
        // SAFETY: the guarantee belongs to the running fiber.
        unsafe { SetThreadStackGuarantee(&mut room) };
        main(arg)
    }

    /// The fiber the calling OS thread runs.
    ///
    /// # Returns
    ///
    /// The fiber, from the block of the thread that the OS keeps.
    #[cfg(target_arch = "x86_64")]
    fn current_fiber() -> *mut u8
    {
        let fiber: *mut u8;
        // SAFETY: `gs` holds the block of the thread; its fiber is at 0x20.
        unsafe {
            std::arch::asm!("mov {}, gs:[0x20]", out(reg) fiber, options(nostack, readonly))
        };
        fiber
    }

    /// The fiber the calling OS thread runs.
    ///
    /// # Returns
    ///
    /// The fiber, from the block of the thread that the OS keeps.
    #[cfg(target_arch = "aarch64")]
    fn current_fiber() -> *mut u8
    {
        let fiber: *mut u8;
        // SAFETY: x18 holds the block of the thread; its fiber is at 0x20.
        unsafe {
            std::arch::asm!("ldr {}, [x18, #0x20]", out(reg) fiber, options(nostack, readonly))
        };
        fiber
    }

    /// Switch fibers: write the running fiber to `from`, and go on in `to`.
    ///
    /// # Arguments
    ///
    /// * `from` - Where the running fiber goes.
    /// * `to` - The fiber to go on in.
    ///
    /// The caller makes sure that `to` is a fiber of [`prepare`] or one that a
    /// switch left.
    pub(crate) unsafe extern "C" fn switch(from: *mut *mut u8, to: *mut u8)
    {
        // SAFETY: the caller hands a fiber; `from` belongs to it.
        unsafe {
            *from = current_fiber();
            SwitchToFiber(to.cast());
        }
    }

    /// Make the calling OS thread a fiber, so it can switch to the fibers of
    /// tasks.
    pub(crate) fn enter_thread()
    {
        // SAFETY: an OS thread of the pool, which no fiber runs so far.
        unsafe { ConvertThreadToFiber(null()) };
    }

    /// Put our handler on the exceptions of the process, once, as the pool
    /// starts.
    pub(crate) fn catch_overflow()
    {
        // SAFETY: the handler lives as long as the program.
        unsafe { AddVectoredExceptionHandler(1, Some(on_exception)) };
    }

    /// The handler of an exception: an overflow of the stack of the running
    /// task stops the program with the trap `stack full`; each other goes on
    /// to the next handler.
    ///
    /// # Arguments
    ///
    /// * `info` - What the OS tells of the exception.
    ///
    /// # Returns
    ///
    /// `EXCEPTION_CONTINUE_SEARCH` for each exception it leaves.
    unsafe extern "system" fn on_exception(info: *mut EXCEPTION_POINTERS) -> i32
    {
        // SAFETY: the OS hands a valid record.
        let code = unsafe { (*(*info).ExceptionRecord).ExceptionCode };
        let room =
            (code == EXCEPTION_STACK_OVERFLOW).then(|| crate::tasks::overflowed(None));
        do_if!(let Some(Some(room)) = room, crate::traps::stack_full_from_signal(room));
        EXCEPTION_CONTINUE_SEARCH
    }
}

#[cfg(all(test, unix))]
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
