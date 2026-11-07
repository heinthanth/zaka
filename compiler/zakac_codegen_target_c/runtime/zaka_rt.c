/* The runtime of a Zaka program that the Zaka Compiler C target builds. */

#include "zaka_rt.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "zaka_c_sugars.h"

#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif

/**
 * The output buffer: the VM writes the output of a run in order, and so
 * does this buffer; a trap writes it out before its own line.
 */
static char zk_out_buf[1 << 16];

/**
 * The name of the executable, without its folder, which leads a trap line
 * as the name of a command leads its errors.
 */
static const char *zk_name = "zaka";

/**
 * The last part of a path, after each / and \.
 *
 * @param path The path.
 * @return The part after the last separator, or the whole path when it has none.
 */
static const char *zk_base_name(const char *path)
{
    const char *name = path;
    for (const char *p = path; *p; p++) DO_IF(*p == '/' || *p == '\\', name = p + 1);
    return name;
}

/**
 * Get the run going: keep the name of the executable for trap lines, write
 * the output as raw octets (Windows would turn each \n into \r\n), and give
 * it a large buffer.
 *
 * @param argc The argument count `main` was given.
 * @param argv The arguments `main` was given; the first is the executable.
 */
void zk_start(int argc, char **argv)
{
    DO_IF(argc > 0 && argv[0] && argv[0][0], zk_name = zk_base_name(argv[0]));
#ifdef _WIN32
    _setmode(_fileno(stdout), _O_BINARY);
    _setmode(_fileno(stderr), _O_BINARY);
#endif
    setvbuf(stdout, zk_out_buf, _IOFBF, sizeof zk_out_buf);
}

/**
 * End the run: write out what is buffered.
 *
 * @param status The exit status of the program.
 * @return The status, for `main` to return.
 */
int zk_finish(int status)
{
    fflush(stdout);
    return status;
}

/* ---- traps ---- */

/**
 * The value of the %{name} slot that a template holds from `p` to `end`.
 *
 * @param p The `%` that opens the slot.
 * @param end The `}` that closes it.
 * @param n The count of names.
 * @param names The name of each value.
 * @param values The values.
 * @return The value of the name in the slot, or NULL when no name matches.
 */
static const char *zk_slot_value(
    const char *p, const char *end, int n, const char *const *names,
    const char *const *values
)
{
    size_t len = (size_t)(end - p - 2);
    for (int i = 0; i < n; i++)
    {
        NEXT_IF(strlen(names[i]) != len || strncmp(p + 2, names[i], len));
        return values[i];
    }
    return NULL;
}

/**
 * Write a template to stderr, each %{name} slot filled with the value of
 * that name; a slot with no value is written as it stands.
 *
 * @param tmpl The template.
 * @param n The count of names.
 * @param names The name of each value.
 * @param values The values.
 */
static void zk_fill_template(
    const char *tmpl, int n, const char *const *names, const char *const *values
)
{
    const char *p = tmpl;
    while (*p)
    {
        const char *end = (p[0] == '%' && p[1] == '{') ? strchr(p, '}') : NULL;
        const char *value = end ? zk_slot_value(p, end, n, names, values) : NULL;
        if (value)
        {
            fputs(value, stderr);
            p = end + 1;
            continue;
        }
        fputc(*p++, stderr);
    }
}

/**
 * Stop the run, as the VM does: write what is buffered, then the trap line
 * `<executable>: the program stopped: <trap>`, then leave with status 1.
 *
 * @param trap The trap, an `enum zk_trap`.
 * @param n The count of names its template uses.
 * @param names The name of each value.
 * @param values The values, as text.
 */
static _Noreturn void zk_stop(
    int trap, int n, const char *const *names, const char *const *values
)
{
    fflush(stdout);
    fprintf(stderr, "%s: ", zk_name);
    const char *at = strstr(zk_stopped_text, "%{e}");
    DO_IF(at, fwrite(zk_stopped_text, 1, (size_t)(at - zk_stopped_text), stderr));
    zk_fill_template(zk_trap_text[trap], n, names, values);
    DO_IF(at, fputs(at + 4, stderr));
    fputc('\n', stderr);
    fflush(stderr);
    exit(1);
}

/**
 * An unsigned number in Western digits, as trap texts write it.
 *
 * @param v The number.
 * @param buf Room for the digits, which are written at its end.
 * @return The first digit, inside `buf`.
 */
static const char *zk_dec(zk_umax v, char buf[48])
{
    char *p = buf + 47;
    *p = 0;
    do *--p = (char)('0' + (int)(v % 10));
    while (v /= 10);
    return p;
}

/** Stop the run: an integer division or remainder had zero as divisor. */
_Noreturn void zk_zero_divisor(void) { zk_stop(ZK_TRAP_ZERO_DIVISOR, 0, NULL, NULL); }

/**
 * Stop the run: a shift went as far as the width of its value, or further.
 *
 * @param distance The shift distance.
 * @param bits The bit width of the shifted value.
 */
_Noreturn void zk_shift_too_far(zk_umax distance, unsigned bits)
{
    char a[48], b[48];
    const char *names[] = {"distance", "bits"};
    const char *values[] = {zk_dec(distance, a), zk_dec(bits, b)};
    zk_stop(ZK_TRAP_SHIFT_TOO_FAR, 2, names, values);
}

/** Stop the run: the step of a counted loop was not positive. */
_Noreturn void zk_step_not_positive(void)
{
    zk_stop(ZK_TRAP_STEP_NOT_POSITIVE, 0, NULL, NULL);
}

/**
 * Stop the run: the result of an operation does not fit its kind.
 *
 * @param op The operator, such as `+` or `neg`.
 * @param kind The Zaka name of the kind, such as `isize`.
 */
_Noreturn void zk_overflow(const char *op, const char *kind)
{
    const char *names[] = {"op", "kind"};
    const char *values[] = {op, kind};
    zk_stop(ZK_TRAP_OVERFLOW, 2, names, values);
}

/**
 * A signed number in Western digits, as trap texts write it.
 *
 * @param v The number.
 * @param buf Room for the digits, which are written at its end.
 * @return The first character, inside `buf`.
 */
static const char *zk_dec_i(zk_imax v, char buf[48])
{
    zk_umax mag = v < 0 ? (zk_umax)0 - (zk_umax)v : (zk_umax)v;
    char *p = (char *)zk_dec(mag, buf);
    DO_IF(v < 0, *--p = '-');
    return p;
}

/**
 * Stop the run with a trap that names a number and its range.
 *
 * @param trap The trap.
 * @param index The number as written.
 * @param lo The first number of the range.
 * @param hi The last number of the range.
 */
_Noreturn void zk_range_trap(int trap, zk_imax index, zk_imax lo, zk_imax hi)
{
    char a[48], b[48], c[48];
    const char *names[] = {"index", "lo", "hi"};
    const char *values[] = {zk_dec_i(index, a), zk_dec_i(lo, b), zk_dec_i(hi, c)};
    zk_stop(trap, 3, names, values);
}

/**
 * Stop the run with a trap that names nothing.
 *
 * @param trap The trap.
 */
_Noreturn void zk_plain_trap(int trap) { zk_stop(trap, 0, NULL, NULL); }

/**
 * Stop the run with a trap that names two numbers: a range of text
 * (start, end) or of cells (len, start, limit).
 *
 * @param trap The trap.
 * @param n The count of names.
 * @param names The names.
 * @param nums The numbers.
 */
_Noreturn void zk_nums_trap(
    int trap, int n, const char *const *names, const zk_imax *nums
)
{
    char bufs[3][48];
    const char *values[3];
    DO_FOR(i, n) values[i] = zk_dec_i(nums[i], bufs[i]);
    zk_stop(trap, n, names, values);
}

/**
 * Stop the run with a trap that names an address.
 *
 * @param trap The trap.
 * @param addr The address.
 */
_Noreturn void zk_addr_trap(int trap, const void *addr)
{
    char buf[40];
    snprintf(buf, sizeof buf, "0x%llx", (unsigned long long)(uintptr_t)addr);
    const char *names[] = {"addr"};
    const char *values[] = {buf};
    zk_stop(trap, 1, names, values);
}

/**
 * Stop the run with a trap that names a text.
 *
 * @param trap The trap.
 * @param e The text of the `%{e}` slot.
 */
_Noreturn void zk_text_trap(int trap, const char *e)
{
    const char *names[] = {"e"};
    const char *values[] = {e};
    zk_stop(trap, 1, names, values);
}

_Noreturn void zk_null_deref(const void *addr)
{
    zk_addr_trap(ZK_TRAP_NULL_DEREF, addr);
}

_Noreturn void zk_out_of_mem(zk_umax size)
{
    char buf[48];
    const char *names[] = {"size"};
    const char *values[] = {zk_dec(size, buf)};
    zk_stop(ZK_TRAP_OUT_OF_MEM, 1, names, values);
}

/* ---- print ---- */

/** The buffer an interpolation writes into; NULL while text goes to
 * stdout. */
static zk_buf *zk_into = NULL;

/**
 * Write octets to the sink: stdout, or the buffer of an interpolation.
 *
 * @param s The octets.
 * @param n Their count.
 */
void zk_emit(const void *s, size_t n)
{
    DONE_IF(!zk_into, fwrite(s, 1, n, stdout));
    size_t need = zk_into->len + n;
    if (need > zk_into->cap)
    {
        size_t cap = need > 2 * zk_into->cap ? need : 2 * zk_into->cap;
        unsigned char *grown = realloc(zk_into->data, cap ? cap : 1);
        DO_IF(!grown, zk_out_of_mem(cap));
        zk_into->data = grown;
        zk_into->cap = cap;
    }
    DO_IF(n, memcpy(zk_into->data + zk_into->len, s, n));
    zk_into->len = need;
}

/**
 * Send what the zk_put functions write into a buffer.
 *
 * @param buf The buffer, made blank here.
 */
void zk_sink_open(zk_buf *buf)
{
    buf->data = NULL;
    buf->len = 0;
    buf->cap = 0;
    buf->prev = zk_into;
    zk_into = buf;
}

/**
 * Send what the zk_put functions write back to stdout, and keep the text of
 * the buffer at a slot, in a heap block of its exact size, as the VM does.
 *
 * @param slot The slot of the text: three words.
 */
void zk_sink_close(unsigned char *slot)
{
    zk_buf *buf = zk_into;
    zk_into = buf->prev;
    unsigned char *data = buf->data;
    DO_IF(buf->len < buf->cap && buf->len > 0, data = realloc(data, buf->len));
    DO_IF(!data && buf->len > 0, zk_out_of_mem(buf->len));
    zk_seq_put(slot, buf->len ? data : NULL, buf->len, buf->len);
    DO_IF(!buf->len, free(buf->data));
}

/**
 * Write octets unchanged.
 *
 * @param s The octets.
 * @param n Their count; a NUL among them is written too.
 */
void zk_put_raw(const char *s, size_t n) { zk_emit(s, n); }

/** Write the line break that ends each print. */
void zk_put_line(void) { zk_emit("\n", 1); }

/**
 * Write number text, each Western digit swapped for the Burmese digit of the
 * same value (U+1040 to U+1049); signs, points and letters stand
 * unchanged.
 *
 * @param s The text, NUL at its end.
 */
static void zk_put_burmese(const char *s)
{
    for (; *s; s++)
    {
        char digit[3] = {'\xe1', '\x81', (char)(0x80 + (*s - '0'))};
        if (*s >= '0' && *s <= '9')
            zk_emit(digit, 3);
        else
            zk_emit(s, 1);
    }
}

/**
 * Write an unsigned integer as wide as `zk_umax`: 128 bits where the
 * compiler has them, 64 elsewhere.
 *
 * @param v The value.
 */
void zk_put_u(zk_umax v)
{
    char buf[48];
    zk_put_burmese(zk_dec(v, buf));
}

/**
 * Write a signed integer as wide as `zk_imax`: 128 bits where the compiler
 * has them, 64 elsewhere.
 *
 * @param v The value.
 */
void zk_put_i(zk_imax v)
{
    DO_IF(v < 0, zk_emit("-", 1));
    zk_put_u(v < 0 ? (zk_umax)0 - (zk_umax)v : (zk_umax)v);
}

/**
 * Write a boolean as the word for true or false.
 *
 * @param v The value, 0 or 1.
 */
void zk_put_bool(uint8_t v)
{
    const char *text = v ? zk_true_text : zk_false_text;
    zk_emit(text, strlen(text));
}

/**
 * Write a character as its UTF-8 octets: each octet after the lead holds
 * six bits, from the low end.
 *
 * @param c The scalar value.
 */
void zk_put_char(uint32_t c)
{
    static const unsigned char lead[] = {0, 0, 0xc0, 0xe0, 0xf0};
    char octets[4];
    size_t n = c < 0x80 ? 1 : c < 0x800 ? 2 : c < 0x10000 ? 3 : 4;
    for (size_t i = n - 1; i > 0; i--, c >>= 6) octets[i] = (char)(0x80 | (c & 0x3f));
    octets[0] = (char)(lead[n] | c);
    zk_emit(octets, n);
}

/**
 * The text of a float that needs no digits.
 *
 * @param v The float.
 * @return `NaN`, `inf`, `-inf`, `0.0` or `-0.0`, or NULL for each other float.
 */
static const char *zk_special(double v)
{
    RET_IF(v != v, "NaN");
    RET_IF(isinf(v), v < 0 ? "-inf" : "inf");
    RET_IF(v == 0, signbit(v) ? "-0.0" : "0.0");
    return NULL;
}

/**
 * Whether digits in exponent form read back to a float.
 *
 * @param sci The digits, as printf writes them with %e.
 * @param v The float.
 * @param single 1 when it reads back as an f32.
 * @return@return 1 when the digits read back to v.
 */
static int zk_reads_back(const char *sci, double v, int single)
{
    return single ? strtof(sci, NULL) == (float)v : strtod(sci, NULL) == v;
}

/**
 * Round a tie up, as Rust does. When the float lies at the midpoint of
 * the digits printf picked and the next ones up, printf keeps the even digit;
 * Rust takes the ones up when those read back too.
 *
 * @param v The float, above zero and finite.
 * @param single 1 when it reads back as an f32.
 * @param sci The digits printf picked, in exponent form; edited in place.
 * @param p Their count.
 */
static void zk_tie_up(double v, int single, char *sci, int p)
{
    char *last = strchr(sci, 'e') - 1;
    /* A tie rounded to even leaves an even digit, so the digit up never carries. */
    RET_IF(((*last - '0') & 1) != 0);
    /* A double has at most 767 significant digits, so this print is exact. */
    char exact[800];
    snprintf(exact, sizeof exact, "%.770e", v);
    /* d.ddd: digit 0 sits at index 0, and digit k > 0 at index k + 1 */
    char *tail = exact + p + 1;
    RET_IF(*tail != '5');
    /* A kept digit that differs from the exact one was rounded up before. */
    RET_IF(exact[p == 1 ? 0 : p] != *last);
    for (tail++; *tail == '0'; tail++);
    RET_IF(*tail != 'e');
    (*last)++;
    DO_IF(!zk_reads_back(sci, v, single), (*last)--);
}

/**
 * The fewest digits of a positive float that read back to it, the nearest
 * ones of that count, with a tie rounded up.
 * shortcut: at a power of two a farther digit string of the shortest count
 * can also read back, which Rust would pick when nearer. Move to a Ryu port
 * if a sample ever shows the difference.
 *
 * @param v The float, above zero and finite.
 * @param single 1 when it is an f32 widened to double, so it reads back as
 *        an f32.
 * @param digits Where the digits go, NUL at their end.
 * @param exp Where the power of ten of the first digit goes.
 * @return The count of digits.
 */
static int zk_shortest(double v, int single, char digits[24], int *exp)
{
    char sci[48];
    int p = 1;
    for (; p <= 17; p++)
    {
        snprintf(sci, sizeof sci, "%.*e", p - 1, v);
        LEAVE_IF(zk_reads_back(sci, v, single));
    }
    zk_tie_up(v, single, sci, p);
    /* sci is d[.ddd]e(+|-)xx */
    int n = 0;
    const char *s = sci;
    for (; *s != 'e'; s++) DO_IF(*s != '.', digits[n++] = *s);
    digits[n] = 0;
    *exp = atoi(s + 1);
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
 * Write a float as Rust writes it with {:?}: the fewest digits that read
 * back, in plain form from 1e-4 up to 1e16 and with an exponent outside
 * that. The bounds are compared in the width of the float.
 *
 * @param v The float.
 * @param single 1 when it is an f32 widened to double.
 */
static void zk_put_float(double v, int single)
{
    const char *special = zk_special(v);
    DONE_IF(special, zk_put_burmese(special));
    double mag = fabs(v);
    char digits[24], text[64];
    int exp;
    int n = zk_shortest(mag, single, digits, &exp);
    int far = single ? ((float)mag < 1e-4f || (float)mag >= 1e16f)
                     : (mag < 1e-4 || mag >= 1e16);
    char *t = text;
    DO_IF(v < 0, *t++ = '-');
    t = far ? zk_exp_form(t, digits, n, exp) : zk_plain_form(t, digits, n, exp);
    *t = 0;
    zk_put_burmese(text);
}

/**
 * Write an f32.
 *
 * @param v The value.
 */
void zk_put_f32(float v) { zk_put_float((double)v, 1); }

/**
 * Write an f64.
 *
 * @param v The value.
 */
void zk_put_f64(double v) { zk_put_float(v, 0); }

/* ---- conversions ---- */

/**
 * The character an integer names as its scalar value.
 *
 * @param negative 1 when the integer is below zero.
 * @param v The integer, as its unsigned bits.
 * @return The scalar value, or U+FFFD when the integer is no scalar value: below
 *         zero, past U+10FFFF, or a surrogate.
 */
uint32_t zk_char_of(int negative, zk_umax v)
{
    RET_IF(negative || v > 0x10ffff, 0xfffd);
    RET_IF(v >= 0xd800 && v <= 0xdfff, 0xfffd);
    return (uint32_t)v;
}
