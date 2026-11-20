/*
 * The inline code a Zaka program that the Zaka Compiler C target builds runs
 * for each operation: checked arithmetic, loads and stores, and the marks of
 * the calls in progress. The runtime itself is the Rust crate zaka_runtime;
 * zaka_runtime.h, which the build writes from it, declares its functions and
 * the structs the generated code shares with it.
 */

#ifndef ZAKA_TARGET_C_RUNTIME_H
#define ZAKA_TARGET_C_RUNTIME_H

/* Round after each float operation, as the VM does: no fused mul-add. */
#pragma STDC FP_CONTRACT OFF

#include <math.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

/* The check level and the call depth limit of the build, which the compiler
 * writes beside the runtime: each ZK_CHECK_<FAMILY> is 1 or 0. */
#include "zaka_checks.h"
#include "zaka_runtime.h"

/* The widest integer kinds: each Zaka integer kind converts to one of them
 * with no loss. */
typedef int64_t zk_imax;
typedef uint64_t zk_umax;

_Static_assert(sizeof(size_t) == sizeof(void *), "a word holds an address and a count");

/** Whether the calls in progress are kept: for the depth check, or for the
 * place and the backtrace of a stop in a build with debug info. */
#define ZK_TRACK (ZK_CHECK_DEPTH || ZK_DEBUG_INFO)

/**
 * Note a call as it starts on the calls in progress of its thread: count it,
 * stop the run past the depth limit, and keep its function; a call past the
 * limit with no depth check is counted but not kept.
 *
 * @param func The number of its function, as zk_frame counts them.
 * @return The calls in progress of the thread.
 */
static inline zk_context *zk_enter(uint32_t func)
{
    zk_context *c = zk_ctx();
    c->depth++;
    if (ZK_CHECK_DEPTH && c->depth > ZK_MAX_DEPTH) zk_too_deep();
    if (c->depth > ZK_MAX_DEPTH) return c;
    c->frames[c->depth].func = func;
    c->frames[c->depth].site = 0;
    return c;
}

/**
 * Note a call as it starts, when the calls are kept, and keep the calls in
 * progress of the thread in the local zk_c, which ZK_LEAVE and ZK_AT read.
 *
 * @param F The number of its function, as zk_frame counts them.
 */
/* clang-format off */
#define ZK_ENTER(F) zk_context *const zk_c = ZK_TRACK ? zk_enter(F) : NULL; (void)zk_c
/* clang-format on */

/**
 * Keep the calls in progress of the thread in the local zk_c, for the code of
 * main and of the start of a lib, which no ZK_ENTER opens.
 */
/* clang-format off */
#define ZK_HERE() zk_context *const zk_c = ZK_TRACK ? zk_ctx() : NULL; (void)zk_c
/* clang-format on */

/** Note a call as it returns. */
/* clang-format off */
#define ZK_LEAVE() do { if (ZK_TRACK) zk_c->depth--; } while (0)
/* clang-format on */

/**
 * Note the site of the statement that runs, in a build with debug info.
 *
 * @param S The index of the site in the sites of the program table.
 */
/* clang-format off */
#define ZK_AT(S) do { if (ZK_DEBUG_INFO && zk_c->depth <= ZK_MAX_DEPTH) zk_c->frames[zk_c->depth].site = (S); } while (0)
/* clang-format on */

/**
 * A safe point of the ring collector at each round of a loop of a program
 * with jobs: a thread a collection asks to halt waits in it until the
 * collection ends. It reads one flag of the runtime.
 */
#ifdef __GNUC__
/* clang-format off */
#define ZK_SAFE_POINT() do { if (__atomic_load_n(&zk_halt, __ATOMIC_ACQUIRE)) zk_safe_point(); } while (0)
/* clang-format on */
#else
/* clang-format off */
#define ZK_SAFE_POINT() do { if (*(volatile uint8_t *)&zk_halt) zk_safe_point(); } while (0)
/* clang-format on */
#endif

#if ZK_CHECK_NULL
#define ZK_NULL_CHECK(p) ((p) == NULL ? zk_null_deref(p) : (void)0)
#else
#define ZK_NULL_CHECK(p) ((void)0)
#endif

/**
 * A load and a store of a scalar of each C kind through an address:
 * `zk_ld_<N>(p)` and `zk_st_<N>(p, v)`, through memcpy, so each address works
 * whatever its alignment and its kind.
 *
 * @param N The name of the kind in function names, such as `i32`.
 * @param T The C kind.
 */
#define ZK_MEM_OPS(N, T)                                                               \
    static inline T zk_ld_##N(const unsigned char *p)                                  \
    {                                                                                  \
        T v;                                                                           \
        ZK_NULL_CHECK(p);                                                              \
        memcpy(&v, p, sizeof v);                                                       \
        return v;                                                                      \
    }                                                                                  \
    static inline void zk_st_##N(unsigned char *p, T v)                                \
    {                                                                                  \
        ZK_NULL_CHECK(p);                                                              \
        memcpy(p, &v, sizeof v);                                                       \
    }

ZK_MEM_OPS(i8, int8_t)
ZK_MEM_OPS(i16, int16_t)
ZK_MEM_OPS(i32, int32_t)
ZK_MEM_OPS(i64, int64_t)
ZK_MEM_OPS(u8, uint8_t)
ZK_MEM_OPS(u16, uint16_t)
ZK_MEM_OPS(u32, uint32_t)
ZK_MEM_OPS(u64, uint64_t)
ZK_MEM_OPS(f32, float)
ZK_MEM_OPS(f64, double)
ZK_MEM_OPS(p, unsigned char *)
ZK_MEM_OPS(fn, zk_fn)

/**
 * The test whether a product overflows, one function for each kind:
 * `zk_mul_ovf_<N>(a, b)`. GCC and Clang have a builtin for it; the rest
 * divide, as CERT INT32-C does.
 *
 * @param N The name of the kind in function names, such as `i32`.
 * @param T The C kind.
 * @param MIN The least value of `T`.
 * @param MAX The greatest value of `T`.
 */
#if defined(__GNUC__)
#define ZK_SMUL_TEST(N, T, MIN, MAX)                                                   \
    static inline int zk_mul_ovf_##N(T a, T b)                                         \
    {                                                                                  \
        T r;                                                                           \
        return __builtin_mul_overflow(a, b, &r);                                       \
    }
#define ZK_UMUL_TEST(N, T, MAX) ZK_SMUL_TEST(N, T, 0, MAX)
#else
#define ZK_SMUL_TEST(N, T, MIN, MAX)                                                   \
    static inline int zk_mul_ovf_##N(T a, T b)                                         \
    {                                                                                  \
        if (a > 0) return b > 0 ? a > (MAX) / b : b < (MIN) / a;                       \
        return b > 0 ? a < (MIN) / b : (a != 0 && b < (MAX) / a);                      \
    }
#define ZK_UMUL_TEST(N, T, MAX)                                                        \
    static inline int zk_mul_ovf_##N(T a, T b) { return a != 0 && b > (MAX) / a; }
#endif

/**
 * The operations of a signed integer kind, each a function named for the
 * kind, so the generated C reads its operands as function arguments:
 *
 * - `zk_add_<N>`, `zk_sub_<N>`, `zk_mul_<N>`, `zk_neg_<N>`: the result, or
 *   a stop with the overflow trap when it does not fit.
 * - `zk_wadd_<N>`, `zk_wsub_<N>`, `zk_wmul_<N>`, `zk_wneg_<N>`: the result
 *   cut to the width of the kind, as two's complement gives it.
 * - `zk_div_<N>`, `zk_rem_<N>`: each rounds toward zero, as Rust does. A zero
 *   divisor stops the program (the language checks alone); MIN / -1 stops
 *   it when `stop` is 1 and gives MIN (MIN % -1 gives 0) when it wraps.
 * - `zk_shl_<N>`, `zk_shr_<N>`: bits shifted out are dropped, and a shift
 *   right keeps the sign; a distance as large as the width stops the
 *   program (the language checks alone).
 * - `zk_f2i_<N>`: a float to the kind, the fraction dropped, values past
 *   the range at the nearest end, NaN at 0, as Rust converts.
 * - `zk_char_<N>`: the character the value names, or U+FFFD.
 *
 * @param N The name of the kind in function names, such as `i32`.
 * @param T The C kind.
 * @param W An unsigned kind at least as wide as `T` and as `unsigned int`,
 *          so wrapping arithmetic never meets a signed overflow.
 * @param MIN The least value of `T`.
 * @param MAX The greatest value of `T`.
 * @param BITS The width of `T`.
 */
#define ZK_SIGNED_OPS(N, T, W, MIN, MAX, BITS)                                         \
    ZK_SMUL_TEST(N, T, MIN, MAX)                                                       \
    static inline T zk_add_##N(T a, T b, const char *k)                                \
    {                                                                                  \
        if ((b > 0 && a > (MAX) - b) || (b < 0 && a < (MIN) - b)) zk_overflow("+", k); \
        return (T)(a + b);                                                             \
    }                                                                                  \
    static inline T zk_sub_##N(T a, T b, const char *k)                                \
    {                                                                                  \
        if ((b < 0 && a > (MAX) + b) || (b > 0 && a < (MIN) + b)) zk_overflow("-", k); \
        return (T)(a - b);                                                             \
    }                                                                                  \
    static inline T zk_mul_##N(T a, T b, const char *k)                                \
    {                                                                                  \
        if (zk_mul_ovf_##N(a, b)) zk_overflow("*", k);                                 \
        return (T)((W)a * (W)b);                                                       \
    }                                                                                  \
    static inline T zk_neg_##N(T a, const char *k)                                     \
    {                                                                                  \
        if (a == (MIN)) zk_overflow("neg", k);                                         \
        return (T) - a;                                                                \
    }                                                                                  \
    static inline T zk_wadd_##N(T a, T b) { return (T)((W)a + (W)b); }                 \
    static inline T zk_wsub_##N(T a, T b) { return (T)((W)a - (W)b); }                 \
    static inline T zk_wmul_##N(T a, T b) { return (T)((W)a * (W)b); }                 \
    static inline T zk_wneg_##N(T a) { return (T)((W)0 - (W)a); }                      \
    static inline T zk_div_##N(T a, T b, int stop, const char *k)                      \
    {                                                                                  \
        if (ZK_CHECK_DIVIDE && b == 0) zk_zero_divisor();                              \
        if (b != -1 || a != (MIN)) return (T)(a / b);                                  \
        if (stop) zk_overflow("/", k);                                                 \
        return (MIN);                                                                  \
    }                                                                                  \
    static inline T zk_rem_##N(T a, T b, int stop, const char *k)                      \
    {                                                                                  \
        if (ZK_CHECK_DIVIDE && b == 0) zk_zero_divisor();                              \
        if (b != -1) return (T)(a % b);                                                \
        if (stop && a == (MIN)) zk_overflow("%", k);                                   \
        return 0;                                                                      \
    }                                                                                  \
    static inline T zk_shl_##N(T a, zk_umax d)                                         \
    {                                                                                  \
        if (ZK_CHECK_SHIFT && d >= (BITS)) zk_shift_too_far(d, (BITS));                \
        return (T)((W)a << d);                                                         \
    }                                                                                  \
    static inline T zk_shr_##N(T a, zk_umax d)                                         \
    {                                                                                  \
        if (ZK_CHECK_SHIFT && d >= (BITS)) zk_shift_too_far(d, (BITS));                \
        return a < 0 ? (T) ~(~a >> d) : (T)(a >> d);                                   \
    }                                                                                  \
    static inline T zk_f2i_##N(double x)                                               \
    {                                                                                  \
        if (x != x) return 0;                                                          \
        if (x <= (double)(MIN)) return (MIN);                                          \
        if (x >= (double)(MAX)) return (MAX);                                          \
        return (T)x;                                                                   \
    }                                                                                  \
    static inline uint32_t zk_char_##N(T x) { return zk_char_of(x < 0, (zk_umax)x); }

/**
 * The operations of an unsigned integer kind, as ZK_SIGNED_OPS gives them
 * for a signed one; `stop` and `k` of the division are there so each kind
 * takes the same arguments.
 *
 * @param N The name of the kind in function names, such as `u32`.
 * @param T The C kind.
 * @param W An unsigned kind at least as wide as `T` and as `unsigned int`.
 * @param MAX The greatest value of `T`.
 * @param BITS The width of `T`.
 */
#define ZK_UNSIGNED_OPS(N, T, W, MAX, BITS)                                            \
    ZK_UMUL_TEST(N, T, MAX)                                                            \
    static inline T zk_add_##N(T a, T b, const char *k)                                \
    {                                                                                  \
        if (a > (MAX) - b) zk_overflow("+", k);                                        \
        return (T)(a + b);                                                             \
    }                                                                                  \
    static inline T zk_sub_##N(T a, T b, const char *k)                                \
    {                                                                                  \
        if (a < b) zk_overflow("-", k);                                                \
        return (T)(a - b);                                                             \
    }                                                                                  \
    static inline T zk_mul_##N(T a, T b, const char *k)                                \
    {                                                                                  \
        if (zk_mul_ovf_##N(a, b)) zk_overflow("*", k);                                 \
        return (T)((W)a * (W)b);                                                       \
    }                                                                                  \
    static inline T zk_neg_##N(T a, const char *k)                                     \
    {                                                                                  \
        if (a != 0) zk_overflow("neg", k);                                             \
        return 0;                                                                      \
    }                                                                                  \
    static inline T zk_wadd_##N(T a, T b) { return (T)((W)a + (W)b); }                 \
    static inline T zk_wsub_##N(T a, T b) { return (T)((W)a - (W)b); }                 \
    static inline T zk_wmul_##N(T a, T b) { return (T)((W)a * (W)b); }                 \
    static inline T zk_wneg_##N(T a) { return (T)((W)0 - (W)a); }                      \
    static inline T zk_div_##N(T a, T b, int stop, const char *k)                      \
    {                                                                                  \
        (void)stop;                                                                    \
        (void)k;                                                                       \
        if (ZK_CHECK_DIVIDE && b == 0) zk_zero_divisor();                              \
        return (T)(a / b);                                                             \
    }                                                                                  \
    static inline T zk_rem_##N(T a, T b, int stop, const char *k)                      \
    {                                                                                  \
        (void)stop;                                                                    \
        (void)k;                                                                       \
        if (ZK_CHECK_DIVIDE && b == 0) zk_zero_divisor();                              \
        return (T)(a % b);                                                             \
    }                                                                                  \
    static inline T zk_shl_##N(T a, zk_umax d)                                         \
    {                                                                                  \
        if (ZK_CHECK_SHIFT && d >= (BITS)) zk_shift_too_far(d, (BITS));                \
        return (T)((W)a << d);                                                         \
    }                                                                                  \
    static inline T zk_shr_##N(T a, zk_umax d)                                         \
    {                                                                                  \
        if (ZK_CHECK_SHIFT && d >= (BITS)) zk_shift_too_far(d, (BITS));                \
        return (T)(a >> d);                                                            \
    }                                                                                  \
    static inline T zk_f2i_##N(double x)                                               \
    {                                                                                  \
        if (x != x || x <= 0) return 0;                                                \
        if (x >= (double)(MAX)) return (MAX);                                          \
        return (T)x;                                                                   \
    }                                                                                  \
    static inline uint32_t zk_char_##N(T x) { return zk_char_of(0, (zk_umax)x); }

ZK_SIGNED_OPS(i8, int8_t, uint64_t, INT8_MIN, INT8_MAX, 8)
ZK_SIGNED_OPS(i16, int16_t, uint64_t, INT16_MIN, INT16_MAX, 16)
ZK_SIGNED_OPS(i32, int32_t, uint64_t, INT32_MIN, INT32_MAX, 32)
ZK_SIGNED_OPS(i64, int64_t, uint64_t, INT64_MIN, INT64_MAX, 64)
ZK_UNSIGNED_OPS(u8, uint8_t, uint64_t, UINT8_MAX, 8)
ZK_UNSIGNED_OPS(u16, uint16_t, uint64_t, UINT16_MAX, 16)
ZK_UNSIGNED_OPS(u32, uint32_t, uint64_t, UINT32_MAX, 32)
ZK_UNSIGNED_OPS(u64, uint64_t, uint64_t, UINT64_MAX, 64)

#endif
