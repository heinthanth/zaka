/*
 * Closures C calls back, in a native build. C calls a bare code address, so
 * each closure a C call gets runs behind a libffi closure, which holds the
 * record. A closure is live while a C call that got it runs, and after that
 * while C has leave (`ခေါ်ဆိုခွင့် ပြုမည်`). The code remains in place after its
 * last use, so a late call from C stops the run instead of running freed
 * code. The compiler adds this file, and the link adds libffi, to a program
 * that can hand a closure to C alone.
 */

#if defined(__APPLE__)
#include <ffi/ffi.h>
#else
#include <ffi.h>
#endif
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "zaka_c_sugars.h"
#include "zaka_rt.h"

/** A closure C can call: its libffi closure and what it reads. */
typedef struct zk_cb
{
    /** The next closure C got. */
    struct zk_cb *next;
    /** The octets of the record when it was first seen: two values with the
     * same function and the same captures are one closure. */
    unsigned char *seen;
    /** The size of those octets. */
    size_t seen_size;
    /** The record the thunk takes last. */
    unsigned char *record;
    /** The C calls that run now and got the closure. */
    unsigned calls;
    /** Whether C has leave to call the closure after those calls. */
    int kept;
    /** The bridge and the libffi kinds of the function kind. */
    const zk_cb_shape *shape;
    /** The call interface libffi reads. */
    ffi_cif cif;
    /** The libffi closure. */
    ffi_closure *closure;
    /** The native address C calls. */
    void *code;
} zk_cb;

/** Each closure C got, kept for the whole run. */
static zk_cb *zk_cbs;
/** The closures lent to the C calls that run now, the latest last. */
static zk_cb **zk_cb_lends;
/** The count of lends. */
static size_t zk_cb_lent_count;
/** The room for lends. */
static size_t zk_cb_lent_room;
/** Set on the thread that runs the program, at its first lend or leave. */
static _Thread_local int zk_cb_home;

/**
 * Stop the run: the runtime has no room for a closure C can call.
 */
static _Noreturn void zk_cb_no_room(void) { zk_plain_trap(ZK_TRAP_OUT_OF_MEM); }

/**
 * The code libffi runs when C calls a closure: a call from another thread
 * ends the run, since the program runs on one thread; a call with no lend
 * and no leave stops it; else the bridge runs the thunk.
 *
 * @param cif The call interface.
 * @param ret Where the result goes.
 * @param args The address of each argument.
 * @param data The closure.
 */
static void zk_cb_run(ffi_cif *cif, void *ret, void **args, void *data)
{
    (void)cif;
    zk_cb *cb = data;
    if (!zk_cb_home)
    {
        fputs(
            "C called a function of the program from a thread other than the "
            "one that runs it; the run ends here\n",
            stderr
        );
        abort();
    }
    DO_IF(!cb->kept && cb->calls == 0, zk_plain_trap(ZK_TRAP_CLOSURE_ESCAPE));
    cb->shape->bridge(ret, args, cb->record);
}

/**
 * The closure C got with the octets of a record.
 *
 * @param rec The record.
 * @param size Its size.
 * @return The closure, or NULL.
 */
static zk_cb *zk_cb_find(const unsigned char *rec, size_t size)
{
    for (zk_cb *cb = zk_cbs; cb; cb = cb->next)
    {
        NEXT_IF(cb->seen_size != size || memcmp(cb->seen, rec, size) != 0);
        return cb;
    }
    return NULL;
}

/**
 * The closure C calls for a function value, made on its first use: a
 * closure, or a function whose parameters cross through the bridge. A
 * function lives for the whole run, so C has leave to call it at each time.
 *
 * @param f The function value.
 * @param shape The bridge and the libffi kinds of its kind.
 * @return The closure.
 */
static zk_cb *zk_cb_of(zk_fn f, const zk_cb_shape *shape)
{
    zk_cb_home = 1;
    unsigned char *rec = zk_fn_rec(f);
    int bound = zk_fn_bound(f);
    size_t size = bound ? zk_fn_size(f) : sizeof(zk_fnrec);
    zk_cb *found = zk_cb_find(rec, size);
    RET_IF(found, found);
    zk_cb *cb = calloc(1, sizeof *cb);
    unsigned char *seen = malloc(size);
    DO_IF(!cb || !seen, zk_cb_no_room());
    memcpy(seen, rec, size);
    cb->seen = seen;
    cb->seen_size = size;
    cb->record = rec;
    cb->kept = !bound;
    cb->shape = shape;
    ffi_type **args = (ffi_type **)shape->args;
    ffi_status prep = ffi_prep_cif(
        &cb->cif, FFI_DEFAULT_ABI, shape->nargs, (ffi_type *)shape->ret, args
    );
    cb->closure = ffi_closure_alloc(sizeof(ffi_closure), &cb->code);
    DO_IF(prep != FFI_OK || !cb->closure, zk_cb_no_room());
    prep = ffi_prep_closure_loc(cb->closure, &cb->cif, zk_cb_run, cb, cb->code);
    DO_IF(prep != FFI_OK, zk_cb_no_room());
    cb->next = zk_cbs;
    zk_cbs = cb;
    return cb;
}

/**
 * The native address C calls for a closure.
 *
 * @param cb The closure.
 * @return The address.
 */
static zk_code zk_cb_addr(const zk_cb *cb)
{
    void *code = cb->code;
    zk_code out;
    memcpy(&out, &code, sizeof out);
    return out;
}

/**
 * The address C calls for a function value given to a C call: the code of a
 * function whose parameters cross as C passes them; else the libffi closure,
 * which for a closure of the program is lent until the call ends.
 *
 * @param f The function value, not null.
 * @param shape The bridge and the libffi kinds of its kind.
 * @return The address.
 */
zk_code zk_cb_code(zk_fn f, const zk_cb_shape *shape)
{
    int bound = zk_fn_bound(f);
    RET_IF(!bound && shape->as_is, zk_fn_code(f));
    zk_cb *cb = zk_cb_of(f, shape);
    RET_IF(!bound, zk_cb_addr(cb));
    if (zk_cb_lent_count == zk_cb_lent_room)
    {
        size_t room = zk_cb_lent_room ? zk_cb_lent_room * 2 : 8;
        zk_cb **grown = realloc(zk_cb_lends, room * sizeof *grown);
        DO_IF(!grown, zk_cb_no_room());
        zk_cb_lends = grown;
        zk_cb_lent_room = room;
    }
    zk_cb_lends[zk_cb_lent_count++] = cb;
    cb->calls++;
    DO_IF(!cb->kept, cb->record = zk_fn_rec(f));
    return zk_cb_addr(cb);
}

/**
 * The count of lends before a C call, for zk_cb_end once it returns.
 *
 * @return The count.
 */
size_t zk_cb_lent(void) { return zk_cb_lent_count; }

/**
 * End the lends of the closures a C call got, once it returns.
 *
 * @param from The count of lends before the call.
 */
void zk_cb_end(size_t from)
{
    while (zk_cb_lent_count > from)
    {
        zk_cb_lends[--zk_cb_lent_count]->calls--;
    }
}

/**
 * Give C leave to call a closure after the call that gave it: the leave
 * holds a count of the record. A function, NULL and a closure with leave are
 * left alone.
 *
 * @param f The function value.
 * @param shape The bridge and the libffi kinds of its kind.
 */
void zk_cb_leave(zk_fn f, const zk_cb_shape *shape)
{
    RET_IF(!f || !zk_fn_bound(f));
    zk_cb *cb = zk_cb_of(f, shape);
    RET_IF(cb->kept);
    cb->record = (unsigned char *)zk_fn_dup(f);
    cb->kept = 1;
}

/**
 * End the leave zk_cb_leave gave a closure, letting go of the count of its
 * record. A value with no leave is left alone.
 *
 * @param f The function value.
 */
void zk_cb_revoke(zk_fn f)
{
    RET_IF(!f || !zk_fn_bound(f));
    zk_cb *cb = zk_cb_find(zk_fn_rec(f), zk_fn_size(f));
    RET_IF(!cb || !cb->kept);
    cb->kept = 0;
    zk_fn_release(cb->record);
}

/**
 * Write a new handle around an address C passed to a function of the
 * program: a box whose one field holds the address. NULL stops the run: the
 * parameter promised a handle, and an optional takes NULL.
 *
 * @param out Where the handle goes.
 * @param addr Where the address C passed lies.
 * @param list The kind of the cells of the box, as a list of one cell.
 */
void zk_cb_handle(unsigned char *out, void *const *addr, const zk_kind *list)
{
    DO_IF(!*addr, zk_plain_trap(ZK_TRAP_NULL_TEXT));
    zk_to_heap(out, (const unsigned char *)addr, list, 0);
}

/**
 * Write an optional of a handle C passed to a function of the program: no
 * data for NULL, else a new handle around the address. The tag sits at
 * offset 0.
 *
 * @param out Where the optional goes: `size` octets.
 * @param size The size of the optional.
 * @param ok_at The offset of the handle in it.
 * @param addr Where the address C passed lies.
 * @param list The kind of the cells of the box, as a list of one cell.
 */
void zk_cb_opt_handle(
    unsigned char *out, size_t size, size_t ok_at, void *const *addr,
    const zk_kind *list
)
{
    memset(out, 0, size);
    RET_IF(!*addr);
    out[0] = 1;
    zk_cb_handle(out + ok_at, addr, list);
}
