/* The glue of a C lib that zakac builds, which the wrappers of its functions
   share: the record the runtime keeps of the lib, the start of its globals on
   the first call, the jump back to the wrapper on a trap, and the reset and
   the last trap of the lib. The lib's callback file includes it once;
   zaka_lib_names.h maps zk_the_program, zk_lib_init, zk_lib_reset,
   zk_lib_last_trap and ZK_LIB_NAME to the names of the lib. */

#ifndef ZAKA_LIB_GLUE_H
#define ZAKA_LIB_GLUE_H

#include <setjmp.h>

#include "zaka_c_sugars.h"
#include "zaka_lib_names.h"
#include "zaka_runtime.h"

/* A DLL gives C the wrappers alone. */
#if defined(_WIN32)
#define ZK_LIB_EXPORT __declspec(dllexport)
#else
#define ZK_LIB_EXPORT
#endif

/* The status of a call that went through. */
#define ZK_OK 0
/* The status of each call after a call stopped on a trap. */
#define ZK_STOPPED (-1)

/* A longjmp of 64-bit Windows unwinds each frame it leaves, through the unwind
   tables the code of C, of Cranelift and of the runtime gives. ZK_LIB_PLAIN_JUMP
   clears the frame in the buffer, so the jump restores the registers alone, as
   it does on each other OS. */
#if defined(_WIN64) && defined(ZK_LIB_PLAIN_JUMP)
#define ZK_LIB_NO_UNWIND(buf) (((_JUMP_BUFFER *)(buf))->Frame = 0)
#else
#define ZK_LIB_NO_UNWIND(buf) ((void)(buf))
#endif

/* A result as a bridge gives it back: a word of the kind of the result. */
typedef union
{
    intptr_t i;
    uintptr_t u;
    float f;
    double d;
    void *p;
} zk_lib_result;

extern const zk_program zk_the_program;
void zk_lib_init(void);

/* The last trap of each Zaka lib of the program. Each lib defines it, the
   linker keeps one, and the runtime of each build of zakac writes its traps
   there, so zk_last_trap sees each lib. A compiler with no weak definitions
   gives each lib its own. */
#if defined(_WIN32)
__declspec(selectany) const zk_trap_info *zk_trap_slot_v1 = NULL;
#elif defined(__GNUC__)
__attribute__((weak)) const zk_trap_info *zk_trap_slot_v1 = NULL;
#else
static const zk_trap_info *zk_trap_slot_v1 = NULL;
#endif

/* The state of the lib, which a call reads and writes once zk_lib_lock lets it
   in, one thread at a time. */
/* The record the runtime keeps of the lib, from its first call. Where a trap
   leaves to, the wrapper of the call in progress, the runtime keeps for each
   thread (zk_lib_jump), since a thread that waits in C lets others in. */
static zk_lib *zk_lib_self;
/* 0 before the first call, 1 while the lib runs, 2 once a trap stopped it. */
static int zk_lib_state;
/* The status of the trap that stopped the lib: 1 + the id of the trap. */
static int32_t zk_lib_status;

/**
 * Leave the call in progress on a trap: jump back to its wrapper.
 *
 * @param trap The id of the trap.
 */
_Noreturn static void zk_lib_leave(int32_t trap)
{
    zk_lib_status = trap + 1;
    longjmp(*(jmp_buf *)zk_lib_jump(), 1);
}

static int32_t zk_lib_guard(void (*step)(void *), void *data);

/**
 * Enter the lib for a call, after the runtime makes its record on the first
 * call.
 *
 * @return The lib around the call, for zk_lib_end.
 */
static zk_lib_place zk_lib_in(void)
{
    DO_IF(
        zk_lib_self == NULL,
        zk_lib_self = zk_lib_open(
            &zk_the_program, zk_lib_leave, ZK_LIB_NAME, &zk_trap_slot_v1, zk_lib_guard
        )
    );
    return zk_lib_enter(zk_lib_self);
}

/**
 * Set the globals, on the first call alone and on the first call after a
 * reset.
 */
static void zk_lib_begin(void)
{
    RET_IF(zk_lib_state != 0);
    zk_lib_state = 1;
    zk_lib_init();
}

/**
 * End a call: the jump target and the lib go back to the call around it, and
 * the lib goes to the next thread that waits once the calls of this one end.
 *
 * @param outer The jump target of the call around this one, or NULL.
 * @param was The lib around the call.
 * @param status The status of the call.
 * @return The status.
 */
static int32_t zk_lib_end(jmp_buf *outer, zk_lib_place was, int32_t status)
{
    zk_lib_jump_set(outer);
    zk_lib_return(was);
    zk_lib_unlock();
    return status;
}

/**
 * End a call that stopped on a trap: the depth of the calls, the closures lent
 * to C calls, the sink of the output and the queues of the drops go back to
 * where the call found them, and the lib remains stopped.
 *
 * @param outer The jump target of the call around this one, or NULL.
 * @param was The lib around the call.
 * @param depth The depth of the calls where the call began.
 * @return The status of the trap.
 */
static int32_t zk_lib_stop(jmp_buf *outer, zk_lib_place was, uint32_t depth)
{
    zk_lib_state = 2;
    zk_depth = depth;
    zk_lib_unwind();
    zk_flush();
    return zk_lib_end(outer, was, zk_lib_status);
}

/**
 * Run a step as a call of the lib, for C that calls a function of the program
 * on a thread with no call of a lib in progress: the lock, the start of the
 * globals, and a jump back here for a trap, which stops the lib.
 *
 * @param step The step.
 * @param data What the step takes.
 * @return The status of the call.
 */
static int32_t zk_lib_guard(void (*step)(void *), void *data)
{
    jmp_buf zk_here;
    zk_lib_lock();
    zk_lib_place zk_was = zk_lib_in();
    jmp_buf *zk_outer = (jmp_buf *)zk_lib_jump();
    uint32_t zk_depth_was = zk_depth;
    RET_IF(zk_lib_state == 2, zk_lib_end(zk_outer, zk_was, ZK_STOPPED));
    if (setjmp(zk_here) != 0) return zk_lib_stop(zk_outer, zk_was, zk_depth_was);
    ZK_LIB_NO_UNWIND(zk_here);
    zk_lib_jump_set(&zk_here);
    zk_lib_begin();
    step(data);
    zk_flush();
    return zk_lib_end(zk_outer, zk_was, ZK_OK);
}

/**
 * Start the lib again after a trap stopped it: the next call sets the globals
 * again. The blocks of the heap from before remain, out of reach.
 */
ZK_LIB_EXPORT void zk_lib_reset(void)
{
    zk_lib_lock();
    zk_lib_forget(zk_lib_self);
    zk_lib_state = 0;
    zk_lib_unlock();
}

/**
 * The trap the lib stopped on last.
 *
 * @return The trap, or NULL while it stopped on none.
 */
ZK_LIB_EXPORT const zk_trap_info *zk_lib_last_trap(void)
{
    return zk_lib_trap(zk_lib_self);
}

/**
 * Free text or the cells of a list the lib gave back, with the C runtime the
 * lib was built with: on Windows a lib of GNU and a lib of MSVC each have
 * their own.
 *
 * @param p The `char *` or the `ptr` of a list; NULL frees nothing.
 */
ZK_LIB_EXPORT void zk_lib_free(void *p) { zk_free(p); }

/**
 * Free text or the cells of a list the lib gave back with each block the
 * cells point at, with the C runtime the lib was built with.
 *
 * @param p The `char *` or the `ptr` of a list; NULL frees nothing.
 */
ZK_LIB_EXPORT void zk_lib_free_all(void *p) { zk_free_all(p); }

#endif
