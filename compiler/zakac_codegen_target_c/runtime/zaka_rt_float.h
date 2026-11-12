/* The float kinds C11 lacks, f16 and f128, where the C compiler has them, and
 * the print of each float kind. zaka_rt.h includes it at its end. */

#ifndef ZAKA_RT_FLOAT_H
#define ZAKA_RT_FLOAT_H

#include <float.h>
#include <math.h>
#include <stdint.h>
#include <string.h>

/* f16 is _Float16 where the compiler defines __FLT16_MANT_DIG__. Without it,
 * a program with f16 stops at an #error in zaka_kinds.h. */
#ifdef __FLT16_MANT_DIG__
#define ZK_HAS_F16 1
__extension__ typedef _Float16 zk_half;
#endif

/* f128 is _Float128 (GCC), a long double of 113 bits (Linux on arm64), or
 * __float128 (Clang on x86-64). Without one of them, a program with f128
 * stops at an #error in zaka_kinds.h. */
#if defined(__FLT128_MANT_DIG__) && __FLT128_MANT_DIG__ == 113
#define ZK_HAS_F128 1
__extension__ typedef _Float128 zk_quad;
#elif LDBL_MANT_DIG == 113
#define ZK_HAS_F128 1
typedef long double zk_quad;
#elif defined(__SIZEOF_FLOAT128__)
#define ZK_HAS_F128 1
__extension__ typedef __float128 zk_quad;
#endif

/** Write number text with Burmese digits; zaka_rt.c holds it. */
void zk_put_burmese(const char *s);
/** Write a float from its bit pattern, as the VM writes it. */
void zk_put_fbits(int bits, uint64_t hi, uint64_t lo);

#ifdef ZK_HAS_F16
_Static_assert(sizeof(zk_half) == 2, "f16 is 16 bits wide");

/**
 * The f16 of a bit pattern.
 *
 * @param b The bits.
 * @return The float.
 */
static inline zk_half zk_f16_of(uint16_t b)
{
    zk_half v;
    memcpy(&v, &b, sizeof v);
    return v;
}

/**
 * The remainder of an f16 division, the sign of the dividend kept. The
 * remainder is exact, so the f32 one rounds to itself.
 *
 * @param a The dividend.
 * @param b The divisor.
 * @return The remainder.
 */
static inline zk_half zk_fmod_half(zk_half a, zk_half b)
{
    return (zk_half)fmodf((float)a, (float)b);
}

/** Write an f16. */
void zk_put_f16(zk_half v);
ZK_MEM_OPS(f16, zk_half)
#endif

#ifdef ZK_HAS_F128
_Static_assert(sizeof(zk_quad) == 16, "f128 is 128 bits wide");

/**
 * The f128 of a bit pattern.
 *
 * @param hi The high 64 bits.
 * @param lo The low 64 bits.
 * @return The float.
 */
static inline zk_quad zk_f128_of(uint64_t hi, uint64_t lo)
{
#if defined(__BYTE_ORDER__) && __BYTE_ORDER__ == __ORDER_BIG_ENDIAN__
    uint64_t w[2] = {hi, lo};
#else
    uint64_t w[2] = {lo, hi};
#endif
    zk_quad v;
    memcpy(&v, w, sizeof v);
    return v;
}

/**
 * The remainder of an f128 division, the sign of the dividend kept.
 *
 * @param a The dividend.
 * @param b The divisor.
 * @return The remainder.
 */
static inline zk_quad zk_fmod_quad(zk_quad a, zk_quad b)
{
#if LDBL_MANT_DIG == 113
    return (zk_quad)fmodl((long double)a, (long double)b);
#else
    return fmodf128(a, b);
#endif
}

/**
 * The conversions of an f128 to an integer kind, as `zk_f2i_<N>` gives them
 * for a double: an f128 holds more bits than a double, so it converts with
 * no stop at double.
 *
 * @param N The name of the kind in function names, such as `i32`.
 * @param T The C kind.
 * @param MIN The least value of `T`.
 * @param MAX The greatest value of `T`.
 */
#define ZK_Q2I(N, T, MIN, MAX)                                                         \
    static inline T zk_q2i_##N(zk_quad x)                                              \
    {                                                                                  \
        if (x != x) return 0;                                                          \
        if (x <= (zk_quad)(MIN)) return (MIN);                                         \
        if (x >= (zk_quad)(MAX)) return (MAX);                                         \
        return (T)x;                                                                   \
    }

ZK_Q2I(i8, int8_t, INT8_MIN, INT8_MAX)
ZK_Q2I(i16, int16_t, INT16_MIN, INT16_MAX)
ZK_Q2I(i32, int32_t, INT32_MIN, INT32_MAX)
ZK_Q2I(i64, int64_t, INT64_MIN, INT64_MAX)
ZK_Q2I(u8, uint8_t, 0, UINT8_MAX)
ZK_Q2I(u16, uint16_t, 0, UINT16_MAX)
ZK_Q2I(u32, uint32_t, 0, UINT32_MAX)
ZK_Q2I(u64, uint64_t, 0, UINT64_MAX)
#ifdef __SIZEOF_INT128__
ZK_Q2I(i128, zk_i128, ZK_I128_MIN, ZK_I128_MAX)
ZK_Q2I(u128, zk_u128, 0, ZK_U128_MAX)
#endif

/** Write an f128. */
void zk_put_f128(zk_quad v);
ZK_MEM_OPS(f128, zk_quad)
#endif

#endif
