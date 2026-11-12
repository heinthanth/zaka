/* Float text as the VM writes it, worked out on the bits with natural numbers
 * of our own, so no print or read of floats from libc takes part: f32 and f64
 * print the fewest digits that read back, the digits Rust's {:?} writes:
 * Dragon4 in the form of Steele and White, as Rust's flt2dec runs it, with a
 * tie rounded up. */

#include <stdio.h>

#include "zaka_c_sugars.h"
#include "zaka_rt.h"

/** Limbs of a natural number: f32 and f64 need about 1100 bits. */
#define ZK_SMALL_LIMBS 48

/** A natural number in 32-bit limbs, the low limb first, no zero limb on top. */
typedef struct
{
    /** The limbs. */
    uint32_t d[ZK_SMALL_LIMBS];
    /** The count of limbs in use. */
    int n;
} zk_big;

/**
 * The bit pattern of a float, cut into its fields.
 */
typedef struct
{
    /** 1 when the sign bit is set. */
    int negative;
    /** The biased exponent field. */
    int field;
    /** The largest value of the exponent field: inf and NaN. */
    int top;
    /** The significand, the hidden bit set for a normal float. */
    uint64_t sig;
    /** The power of two of the low significand bit. */
    int exp;
    /** 1 when the fraction field is zero. */
    int frac_zero;
} zk_fparts;

/* ---- natural numbers ---- */

/**
 * Scale limbs up, times a small factor.
 *
 * @param d The limbs, room for one more.
 * @param n Their count.
 * @param m The factor.
 * @return The new count.
 */
static int zk_limbs_mul(uint32_t *d, int n, uint32_t m)
{
    uint64_t over = 0;
    DO_FOR(i, n)
    {
        uint64_t p = (uint64_t)d[i] * m + over;
        d[i] = (uint32_t)p;
        over = p >> 32;
    }
    DO_IF(over, d[n++] = (uint32_t)over);
    return n;
}

/**
 * Shift limbs left.
 *
 * @param d The limbs, room for the shift.
 * @param n Their count.
 * @param s The bit count of the shift.
 * @return The new count.
 */
static int zk_limbs_shl(uint32_t *d, int n, int s)
{
    RET_IF(n == 0, 0);
    int whole = s / 32, part = s % 32;
    uint32_t top = part ? d[n - 1] >> (32 - part) : 0;
    for (int i = n - 1; i > 0; i--)
        d[i + whole] = (d[i] << part) | (part ? d[i - 1] >> (32 - part) : 0);
    d[whole] = d[0] << part;
    DO_FOR(i, whole) d[i] = 0;
    n += whole;
    DO_IF(top, d[n++] = top);
    return n;
}

/**
 * Scale limbs up, times a power of ten, a chunk at a time.
 *
 * @param d The limbs, room for the product.
 * @param n Their count.
 * @param e The power.
 * @return The new count.
 */
static int zk_limbs_pow10(uint32_t *d, int n, int e)
{
    /* 10^9 is the largest power of ten below 2^32. */
    for (; e >= 9; e -= 9) n = zk_limbs_mul(d, n, 1000000000u);
    uint32_t rest = 1;
    DO_FOR(i, e) rest *= 10;
    return rest == 1 ? n : zk_limbs_mul(d, n, rest);
}

/**
 * Set a natural number to a 64-bit value.
 *
 * @param x The number.
 * @param v The value.
 */
static void zk_big_set(zk_big *x, uint64_t v)
{
    x->n = 0;
    for (; v; v >>= 32) x->d[x->n++] = (uint32_t)v;
}

/**
 * Compare two natural numbers.
 *
 * @param a The first.
 * @param b The second.
 * @return -1, 0 or 1 as a is below, equal to or above b.
 */
static int zk_big_cmp(const zk_big *a, const zk_big *b)
{
    RET_IF(a->n != b->n, a->n < b->n ? -1 : 1);
    for (int i = a->n - 1; i >= 0; i--)
        RET_IF(a->d[i] != b->d[i], a->d[i] < b->d[i] ? -1 : 1);
    return 0;
}

/**
 * The sum of two natural numbers.
 *
 * @param a The first.
 * @param b The second.
 * @return a + b.
 */
static zk_big zk_big_sum(const zk_big *a, const zk_big *b)
{
    zk_big s;
    int n = a->n > b->n ? a->n : b->n;
    uint64_t over = 0;
    DO_FOR(i, n)
    {
        uint64_t v =
            over + ((int)i < a->n ? a->d[i] : 0) + ((int)i < b->n ? b->d[i] : 0);
        s.d[i] = (uint32_t)v;
        over = v >> 32;
    }
    DO_IF(over, s.d[n++] = (uint32_t)over);
    s.n = n;
    return s;
}

/**
 * Take a natural number from one no smaller.
 *
 * @param x The larger; changed to the difference.
 * @param s The smaller.
 */
static void zk_big_sub(zk_big *x, const zk_big *s)
{
    int64_t owe = 0;
    DO_FOR(i, x->n)
    {
        int64_t v = (int64_t)x->d[i] - ((int)i < s->n ? s->d[i] : 0) - owe;
        owe = v < 0;
        x->d[i] = (uint32_t)(v + (owe << 32));
    }
    while (x->n > 0 && x->d[x->n - 1] == 0) x->n--;
}

/**
 * Scale three natural numbers up, times ten each.
 *
 * @param a The first.
 * @param b The second.
 * @param c The third.
 */
static void zk_big_tens(zk_big *a, zk_big *b, zk_big *c)
{
    a->n = zk_limbs_mul(a->d, a->n, 10);
    b->n = zk_limbs_mul(b->d, b->n, 10);
    c->n = zk_limbs_mul(c->d, c->n, 10);
}

/* ---- the fields of a float ---- */

/**
 * Cut the bit pattern of an IEEE 754 float into its fields.
 *
 * @param bits The width of the float: 32 or 64.
 * @param raw The pattern, in the low `bits` bits.
 * @return The fields.
 */
static zk_fparts zk_fparts_of(int bits, uint64_t raw)
{
    zk_fparts f;
    int frac_bits = bits == 32 ? 23 : 52, exp_bits = bits - 1 - frac_bits;
    f.top = (1 << exp_bits) - 1;
    f.negative = (int)((raw >> (bits - 1)) & 1);
    f.field = (int)((raw >> frac_bits) & (uint64_t)f.top);
    f.sig = raw & ((UINT64_C(1) << frac_bits) - 1);
    f.frac_zero = f.sig == 0;
    int bias = (1 << (exp_bits - 1)) - 1;
    f.exp = (f.field == 0 ? 1 : f.field) - bias - frac_bits;
    DO_IF(f.field != 0, f.sig |= UINT64_C(1) << frac_bits);
    return f;
}

/**
 * The text of a float that needs no digits.
 *
 * @param f The fields of the float.
 * @return `NaN`, `inf`, `-inf`, `0.0` or `-0.0`, or NULL for each other float.
 */
static const char *zk_special(const zk_fparts *f)
{
    RET_IF(f->field == f->top && !f->frac_zero, "NaN");
    RET_IF(f->field == f->top, f->negative ? "-inf" : "inf");
    RET_IF(f->field == 0 && f->frac_zero, f->negative ? "-0.0" : "0.0");
    return NULL;
}

/* ---- the shortest digits ---- */

/**
 * A first guess at the power of ten above a value, as Rust's flt2dec makes
 * it: never above the true one, and at most one below.
 *
 * @param mant The value is mant * 2^exp.
 * @param exp The power of two.
 * @return The guess.
 */
static int zk_guess_k(uint64_t mant, int exp)
{
    int nbits = 0;
    for (uint64_t v = mant - 1; v; v >>= 1) nbits++;
    /* 1292913986 is floor(2^32 * log10(2)); the shift rounds toward minus inf. */
    int64_t p = (int64_t)(nbits + exp) * 1292913986;
    return (int)(p >= 0 ? p >> 32 : -((-p + (INT64_C(1) << 32) - 1) >> 32));
}

/**
 * Add one to the last of the digits; nines turn to zeros up the line.
 *
 * @param digits The digits.
 * @param n Their count.
 * @return 1 when each digit was a nine, so the digits became 1 and zeros and
 *         one more zero goes at their end.
 */
static int zk_round_up(char *digits, int n)
{
    int i = n - 1;
    for (; i >= 0 && digits[i] == '9'; i--) digits[i] = '0';
    DO_IF(i >= 0, digits[i]++);
    RET_IF(i >= 0, 0);
    digits[0] = '1';
    return 1;
}

/**
 * The next digit: the count of times `scale` goes into `mant`, below ten,
 * with `mant` left as the remainder.
 *
 * @param mant The number; changed to the remainder.
 * @param scale The scale, and its double, four times and eight times.
 * @return The digit.
 */
static int zk_next_digit(zk_big *mant, const zk_big scale[4])
{
    int digit = 0;
    for (int i = 3; i >= 0; i--)
    {
        NEXT_IF(zk_big_cmp(mant, &scale[i]) < 0);
        zk_big_sub(mant, &scale[i]);
        digit += 1 << i;
    }
    return digit;
}

/**
 * The fewest digits that read back to a float, as Rust's Dragon4 finds them:
 * the value is mant * 2^exp, and each number between (mant - minus) * 2^exp
 * and (mant + plus) * 2^exp reads back to it, the ends counted when
 * `inclusive` is 1. A tie between two last digits rounds up.
 *
 * @param mant The scaled significand.
 * @param minus The gap down to the float below, in the same units.
 * @param plus The gap up to the float above.
 * @param exp The power of two of the units.
 * @param inclusive 1 when the ends of the gap read back too.
 * @param digits Where the digits go, room for 24.
 * @param k Where the power of ten goes: the value is 0.d1d2... * 10^k.
 * @return The count of digits.
 */
static int zk_dragon(
    uint64_t mant, uint64_t minus, uint64_t plus, int exp, int inclusive, char *digits,
    int *k
)
{
    /* A comparison c of two numbers counts as "below" when c < edge. */
    int edge = inclusive ? 1 : 0;
    *k = zk_guess_k(mant + plus, exp);
    zk_big m, lo, hi, s[4];
    zk_big_set(&m, mant);
    zk_big_set(&lo, minus);
    zk_big_set(&hi, plus);
    zk_big_set(&s[0], 1);
    if (exp < 0)
        s[0].n = zk_limbs_shl(s[0].d, s[0].n, -exp);
    else
    {
        m.n = zk_limbs_shl(m.d, m.n, exp);
        lo.n = zk_limbs_shl(lo.d, lo.n, exp);
        hi.n = zk_limbs_shl(hi.d, hi.n, exp);
    }
    if (*k >= 0)
        s[0].n = zk_limbs_pow10(s[0].d, s[0].n, *k);
    else
    {
        m.n = zk_limbs_pow10(m.d, m.n, -*k);
        lo.n = zk_limbs_pow10(lo.d, lo.n, -*k);
        hi.n = zk_limbs_pow10(hi.d, hi.n, -*k);
    }
    zk_big top = zk_big_sum(&m, &hi);
    if (zk_big_cmp(&s[0], &top) < edge)
        (*k)++;
    else
        zk_big_tens(&m, &lo, &hi);
    for (int i = 1; i < 4; i++)
    {
        s[i] = s[0];
        s[i].n = zk_limbs_shl(s[i].d, s[i].n, i);
    }
    int n = 0, down, up;
    for (;;)
    {
        digits[n++] = (char)('0' + zk_next_digit(&m, s));
        top = zk_big_sum(&m, &hi);
        down = zk_big_cmp(&m, &lo) < edge;
        up = zk_big_cmp(&s[0], &top) < edge;
        LEAVE_IF(down || up);
        zk_big_tens(&m, &lo, &hi);
    }
    zk_big twice = m;
    twice.n = zk_limbs_shl(twice.d, twice.n, 1);
    RET_IF(!up || (down && zk_big_cmp(&twice, &s[0]) < 0), n);
    DO_IF(zk_round_up(digits, n), (digits[n++] = '0', (*k)++));
    return n;
}

/**
 * The fewest digits that read back to a finite float above zero, f32 or f64,
 * with the gaps to its neighbours as Rust's decoder gives them: at a power of
 * two the gap below is half the gap above.
 *
 * @param f The fields of the float.
 * @param digits Where the digits go, room for 24.
 * @param exp Where the power of ten of the first digit goes.
 * @return The count of digits.
 */
static int zk_shortest(const zk_fparts *f, char *digits, int *exp)
{
    uint64_t mant = f->sig;
    int k, n, e = f->exp;
    /* Rust decodes a subnormal as mant * 2 at one power of two less, so it
     * counts as even and its gaps are 1 each. */
    if (f->field == 0)
        n = zk_dragon(mant << 1, 1, 1, e - 1, 1, digits, &k);
    else if (f->frac_zero)
        n = zk_dragon(mant << 2, 1, 2, e - 2, (mant & 1) == 0, digits, &k);
    else
        n = zk_dragon(mant << 1, 1, 1, e - 1, (mant & 1) == 0, digits, &k);
    digits[n] = 0;
    *exp = k - 1;
    return n;
}

/**
 * Write digits with an exponent, such as 1.5e-5.
 *
 * @param t Where the text goes.
 * @param digits The digits, NUL at their end.
 * @param n Their count.
 * @param exp The power of ten of the first digit.
 * @return The end of the text written.
 */
static char *zk_exp_form(char *t, const char *digits, int n, int exp)
{
    *t++ = digits[0];
    DO_IF(n > 1, t += sprintf(t, ".%s", digits + 1));
    return t + sprintf(t, "e%d", exp);
}

/**
 * Write digits as a plain number with a point, such as 0.001, 12.5 or
 * 100.0.
 *
 * @param t Where the text goes.
 * @param digits The digits, NUL at their end.
 * @param n Their count.
 * @param exp The power of ten of the first digit.
 * @return The end of the text written.
 */
static char *zk_plain_form(char *t, const char *digits, int n, int exp)
{
    int point = exp + 1;
    if (point <= 0)
    {
        t += sprintf(t, "0.");
        DO_FOR(i, -point) *t++ = '0';
        return t + sprintf(t, "%s", digits);
    }
    DO_FOR(i, point) *t++ = (int)i < n ? digits[i] : '0';
    *t++ = '.';
    RET_IF(n <= point, t + sprintf(t, "0"));
    return t + sprintf(t, "%s", digits + point);
}

/**
 * The text of a finite f32 or f64 other than zero, as Rust writes it with
 * {:?}: the fewest digits that read back, in plain form from 1e-4 up to 1e16
 * and with an exponent outside that.
 *
 * @param f The fields of the float.
 * @param far 1 when the float lies outside the plain range, in its own width.
 * @param t Where the text goes, after the sign.
 */
static void zk_shortest_text(const zk_fparts *f, int far, char *t)
{
    char digits[24];
    int exp;
    int n = zk_shortest(f, digits, &exp);
    t = far ? zk_exp_form(t, digits, n, exp) : zk_plain_form(t, digits, n, exp);
    *t = 0;
}

/* ---- prints ---- */

/**
 * Write a float from its bit pattern, as the VM writes it.
 *
 * @param bits The width: 32 or 64.
 * @param raw The pattern, in the low `bits` bits.
 */
void zk_put_fbits(int bits, uint64_t raw)
{
    zk_fparts f = zk_fparts_of(bits, raw);
    const char *special = zk_special(&f);
    DONE_IF(special, zk_put_burmese(special));
    double mag = 0;
    DO_IF(bits == 64, (memcpy(&mag, &raw, 8), mag = fabs(mag)));
    float single = 0;
    uint32_t low = (uint32_t)raw;
    DO_IF(bits == 32, (memcpy(&single, &low, 4), single = fabsf(single)));
    /* {:?} writes the plain form from 1e-4 up to 1e16, in the float's width. */
    int far =
        bits == 32 ? single < 1e-4f || single >= 1e16f : mag < 1e-4 || mag >= 1e16;
    char text[80];
    text[0] = '-';
    zk_shortest_text(&f, far, text + f.negative);
    zk_put_burmese(text);
}

/**
 * Write an f32.
 *
 * @param v The value.
 */
void zk_put_f32(float v)
{
    uint32_t b;
    memcpy(&b, &v, 4);
    zk_put_fbits(32, b);
}

/**
 * Write an f64.
 *
 * @param v The value.
 */
void zk_put_f64(double v)
{
    uint64_t b;
    memcpy(&b, &v, 8);
    zk_put_fbits(64, b);
}
