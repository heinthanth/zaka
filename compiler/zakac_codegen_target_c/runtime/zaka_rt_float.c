/* Float text as the VM writes it, worked out on the bits with natural numbers
 * of our own, so no print or read of floats from libc takes part.
 *
 * - f32 and f64 print the fewest digits that read back, the digits Rust's
 *   {:?} writes: Dragon4 in the form of Steele and White, as Rust's flt2dec
 *   runs it, with a tie rounded up.
 * - f16 and f128 print as rustc_apfloat writes them: the exact decimal
 *   value, cut to 2 + P * 59 / 196 digits (P the bits of the significand)
 *   with a half rounded up. */

#include <stdio.h>

#include "zaka_c_sugars.h"
#include "zaka_rt.h"

/** Limbs of a small natural number: f32 and f64 need about 1100 bits. */
#define ZK_SMALL_LIMBS 48
/** Limbs of a large natural number: the least f128 needs 113 + 16494 * log2(5)
 * bits, 38410 in all. */
#define ZK_LARGE_LIMBS 1216
/** Decimal digits of a large natural number, at most. */
#define ZK_LARGE_DIGITS 11600

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
    uint64_t sig_hi, sig_lo;
    /** The power of two of the low significand bit. */
    int exp;
    /** The bits of the significand, the hidden bit counted. */
    int precision;
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
 * Scale limbs up, times a power of a small base, a chunk at a time.
 *
 * @param d The limbs, room for the product.
 * @param n Their count.
 * @param base The base, 5 or 10.
 * @param e The power.
 * @return The new count.
 */
static int zk_limbs_pow(uint32_t *d, int n, uint32_t base, int e)
{
    /* 5^13 and 10^9 are the largest powers below 2^32. */
    int step = base == 5 ? 13 : 9;
    uint32_t chunk = base == 5 ? 1220703125u : 1000000000u;
    for (; e >= step; e -= step) n = zk_limbs_mul(d, n, chunk);
    uint32_t rest = 1;
    DO_FOR(i, e) rest *= base;
    return rest == 1 ? n : zk_limbs_mul(d, n, rest);
}

/**
 * Divide limbs in place, with a small divisor.
 *
 * @param d The limbs.
 * @param n Their count; changed to the count of the quotient.
 * @param m The divisor.
 * @return The remainder.
 */
static uint32_t zk_limbs_div(uint32_t *d, int *n, uint32_t m)
{
    uint64_t rem = 0;
    for (int i = *n - 1; i >= 0; i--)
    {
        uint64_t cur = (rem << 32) | d[i];
        d[i] = (uint32_t)(cur / m);
        rem = cur % m;
    }
    while (*n > 0 && d[*n - 1] == 0) (*n)--;
    return (uint32_t)rem;
}

/**
 * The count of bits up to the highest set one.
 *
 * @param d The limbs.
 * @param n Their count.
 * @return The bit count, 0 for zero.
 */
static int zk_limbs_bits(const uint32_t *d, int n)
{
    RET_IF(n == 0, 0);
    int bits = (n - 1) * 32;
    for (uint32_t top = d[n - 1]; top; top >>= 1) bits++;
    return bits;
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
 * @param bits The width of the float: 16, 32, 64 or 128.
 * @param hi The high 64 bits of the pattern, for f128.
 * @param lo The low 64 bits of the pattern.
 * @return The fields.
 */
static zk_fparts zk_fparts_of(int bits, uint64_t hi, uint64_t lo)
{
    static const int precision_of[] = {11, 24, 53, 113};
    zk_fparts f;
    f.precision = precision_of[bits == 16 ? 0 : bits == 32 ? 1 : bits == 64 ? 2 : 3];
    int frac_bits = f.precision - 1, exp_bits = bits - f.precision;
    f.top = (1 << exp_bits) - 1;
    if (bits == 128)
    {
        f.negative = (int)(hi >> 63);
        f.field = (int)((hi >> 48) & 0x7fff);
        f.sig_hi = hi & ((UINT64_C(1) << 48) - 1);
        f.sig_lo = lo;
    }
    else
    {
        f.negative = (int)((lo >> (bits - 1)) & 1);
        f.field = (int)((lo >> frac_bits) & (uint64_t)f.top);
        f.sig_hi = 0;
        f.sig_lo = lo & ((UINT64_C(1) << frac_bits) - 1);
    }
    f.frac_zero = f.sig_hi == 0 && f.sig_lo == 0;
    int bias = (1 << (exp_bits - 1)) - 1;
    f.exp = (f.field == 0 ? 1 : f.field) - bias - frac_bits;
    RET_IF(f.field == 0, f);
    DO_IF(bits == 128, f.sig_hi |= UINT64_C(1) << 48);
    DO_IF(bits != 128, f.sig_lo |= UINT64_C(1) << frac_bits);
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

/* ---- the shortest digits: f32 and f64 ---- */

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
        s[0].n = zk_limbs_pow(s[0].d, s[0].n, 10, *k);
    else
    {
        m.n = zk_limbs_pow(m.d, m.n, 10, -*k);
        lo.n = zk_limbs_pow(lo.d, lo.n, 10, -*k);
        hi.n = zk_limbs_pow(hi.d, hi.n, 10, -*k);
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
    uint64_t mant = f->sig_lo;
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

/* ---- the fixed digits: f16 and f128 ---- */

/**
 * The decimal digits of a finite float other than zero, low digit first, as
 * rustc_apfloat finds them: each digit of the exact value, less a count of
 * low ones it drops unread, and less the zeros at the low end.
 *
 * @param f The fields of the float.
 * @param keep The count of digits the text keeps.
 * @param all Where the digits go, room for ZK_LARGE_DIGITS.
 * @param exp Where the power of ten of the low digit goes.
 * @param n Where the count of digits goes.
 * @return The first kept digit, within `all`.
 */
static char *zk_exact_digits(const zk_fparts *f, int keep, char *all, int *exp, int *n)
{
    uint32_t d[ZK_LARGE_LIMBS];
    uint64_t hi = f->sig_hi, lo = f->sig_lo;
    int e = f->exp;
    for (; (lo & 1) == 0; e++) lo = (lo >> 1) | (hi << 63), hi >>= 1;
    uint64_t words[2] = {lo, hi};
    int len = 0;
    DO_FOR(i, 4) d[i] = (uint32_t)(words[i / 2] >> (i % 2 * 32));
    DO_FOR(i, 4) DO_IF(d[i], len = (int)i + 1);
    /* N * 2^-e is N * 5^e * 10^-e, so the power of two turns into one of ten. */
    if (e > 0)
        len = zk_limbs_shl(d, len, e);
    else
        len = zk_limbs_pow(d, len, 5, -e);
    *exp = e > 0 ? 0 : e;
    /* 196 / 59 is a little above log2(10). */
    int required = (keep * 196 + 58) / 59, bits = zk_limbs_bits(d, len);
    int drop = bits > required ? (bits - required) * 59 / 196 : 0;
    int count = 0;
    while (len > 0)
    {
        uint32_t rem = zk_limbs_div(d, &len, 1000000000u);
        for (int i = 0; i < 9 && (len > 0 || rem); i++, rem /= 10)
            all[count++] = (char)('0' + rem % 10);
    }
    int at = drop < count ? drop : count;
    for (; at < count && all[at] == '0'; at++);
    *exp += at;
    *n = count - at;
    return all + at;
}

/**
 * Cut digits to a count, a half rounded up, as rustc_apfloat does.
 *
 * @param b The digits, low first; the result starts within them.
 * @param n Their count; changed to the count left.
 * @param keep The count to keep at most.
 * @param exp The power of ten of the low digit; changed.
 * @return The digits left, low first.
 */
static char *zk_cut_digits(char *b, int *n, int keep, int *exp)
{
    RET_IF(*n <= keep, b);
    int first = *n - keep;
    if (b[first - 1] < '5')
        for (; first < *n && b[first] == '0'; first++);
    else
    {
        /* Nines turn to zeros, which the cut drops with them. */
        for (; first < *n && b[first] == '9'; first++);
        DO_IF(first < *n, b[first]++);
    }
    *exp += first;
    *n -= first;
    b += first;
    RET_IF(*n > 0, b);
    /* Each digit was a nine, so one 1 is left. */
    b[-1] = '1';
    *n = 1;
    return b - 1;
}

/**
 * The text of a finite f16 or f128 other than zero, as the VM writes it from
 * rustc_apfloat's Display: plain while the low digit lies at most three
 * places above the point and the first at most three below, with an exponent
 * outside that, then a `.0` on a whole number.
 *
 * @param f The fields of the float.
 * @param t Where the text goes, after the sign.
 */
static void zk_fixed_text(const zk_fparts *f, char *t)
{
    char all[ZK_LARGE_DIGITS];
    int keep = 2 + f->precision * 59 / 196, exp, n;
    char *b = zk_exact_digits(f, keep, all, &exp, &n);
    b = zk_cut_digits(b, &n, keep, &exp);
    int lead = exp + n - 1;
    int far = exp >= 0 ? exp > 3 || n + exp > keep : lead < -3;
    if (far)
    {
        *t++ = b[n - 1];
        DO_IF(n > 1, *t++ = '.');
        for (int i = n - 2; i >= 0; i--) *t++ = b[i];
        sprintf(t, "e%d", lead);
        return;
    }
    if (exp >= 0)
    {
        for (int i = n - 1; i >= 0; i--) *t++ = b[i];
        DO_FOR(i, exp) *t++ = '0';
        strcpy(t, ".0");
        return;
    }
    int unit = -exp;
    if (unit >= n)
    {
        t += sprintf(t, "0.");
        DO_FOR(i, unit - n) *t++ = '0';
    }
    for (int i = n - 1; i >= 0; i--)
    {
        *t++ = b[i];
        DO_IF(i == unit && unit < n, *t++ = '.');
    }
    *t = 0;
}

/* ---- prints ---- */

/**
 * Write a float from its bit pattern, as the VM writes it.
 *
 * @param bits The width: 16, 32, 64 or 128.
 * @param hi The high 64 bits of the pattern, for f128.
 * @param lo The low 64 bits of the pattern.
 * @param far For f32 and f64, 1 when the float lies outside the plain range
 *        of {:?}, from 1e-4 up to 1e16.
 */
static void zk_put_float(int bits, uint64_t hi, uint64_t lo, int far)
{
    zk_fparts f = zk_fparts_of(bits, hi, lo);
    const char *special = zk_special(&f);
    DONE_IF(special, zk_put_burmese(special));
    char text[80];
    text[0] = '-';
    char *t = text + f.negative;
    if (bits == 32 || bits == 64)
        zk_shortest_text(&f, far, t);
    else
        zk_fixed_text(&f, t);
    zk_put_burmese(text);
}

/**
 * Write a float from its bit pattern, as the VM writes it.
 *
 * @param bits The width: 16, 32, 64 or 128.
 * @param hi The high 64 bits of the pattern, for f128.
 * @param lo The low 64 bits of the pattern.
 */
void zk_put_fbits(int bits, uint64_t hi, uint64_t lo)
{
    double mag = 0;
    DO_IF(bits == 64, (memcpy(&mag, &lo, 8), mag = fabs(mag)));
    float single = 0;
    uint32_t low = (uint32_t)lo;
    DO_IF(bits == 32, (memcpy(&single, &low, 4), single = fabsf(single)));
    int far =
        bits == 32 ? single < 1e-4f || single >= 1e16f : mag < 1e-4 || mag >= 1e16;
    zk_put_float(bits, hi, lo, far);
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
    zk_put_fbits(32, 0, b);
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
    zk_put_fbits(64, 0, b);
}

#ifdef ZK_HAS_F16
/**
 * Write an f16.
 *
 * @param v The value.
 */
void zk_put_f16(zk_half v)
{
    uint16_t b;
    memcpy(&b, &v, 2);
    zk_put_fbits(16, 0, b);
}
#endif

#ifdef ZK_HAS_F128
/**
 * Write an f128.
 *
 * @param v The value.
 */
void zk_put_f128(zk_quad v)
{
    uint64_t w[2];
    memcpy(w, &v, 16);
#if defined(__BYTE_ORDER__) && __BYTE_ORDER__ == __ORDER_BIG_ENDIAN__
    zk_put_fbits(128, w[0], w[1]);
#else
    zk_put_fbits(128, w[1], w[0]);
#endif
}
#endif
