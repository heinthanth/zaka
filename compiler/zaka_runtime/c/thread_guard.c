/* The guard of a thread for code that runs in the process of zakac, as the JIT's
   does: a build of a program compiles the same guard from the C of its own
   glue, while code the JIT writes finds this one in the runtime. ZK_NAME
   gives each name the prefix of the runtime, so it never meets the one of a
   build. */

#include <setjmp.h>
#include <stdint.h>

#define ZK_JOIN(a, b) a##b
#define ZK_PASTE(a, b) ZK_JOIN(a, b)
#define ZK_NAME(name) ZK_PASTE(ZK_PREFIX, name)

void *ZK_NAME(lib_jump)(void);
void ZK_NAME(lib_jump_set)(void *to);
void ZK_NAME(flush)(void);

/* A longjmp of 64-bit Windows unwinds each frame it leaves, and code the JIT
   writes gives no table to unwind it with; clearing the frame of the buffer
   makes the jump restore the registers alone, as on each other OS. */
#if defined(_WIN64)
#define ZK_THREAD_NO_UNWIND(buf) (((_JUMP_BUFFER *)(buf))->Frame = 0)
#else
#define ZK_THREAD_NO_UNWIND(buf) ((void)(buf))
#endif

/**
 * Jump back to the guard of the thread of this OS thread, on a trap that ends the
 * thread alone.
 *
 * @param trap The id of the trap, which the runtime keeps.
 */
_Noreturn void ZK_NAME(thread_leave)(int32_t trap)
{
    (void)trap;
    longjmp(*(jmp_buf *)ZK_NAME(lib_jump)(), 1);
}

/**
 * Run the step of a thread, keeping the place a trap of the thread jumps back to.
 *
 * @param step The step.
 * @param data The data block of the thread.
 * @return 0, or 1 once a trap ended the thread.
 */
int32_t ZK_NAME(thread_guard)(void (*step)(void *), void *data)
{
    jmp_buf zk_here;
    if (setjmp(zk_here) != 0) return 1;
    ZK_THREAD_NO_UNWIND(zk_here);
    ZK_NAME(lib_jump_set)(&zk_here);
    step(data);
    ZK_NAME(flush)();
    return 0;
}
