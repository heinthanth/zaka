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

/** The most lines of calls a short backtrace shows, as in the VM. */
#define ZK_SHOWN_CALLS 16

/**
 * The last call in progress that the frame table keeps.
 *
 * @return Its index in zk_frames.
 */
static uint32_t zk_top(void)
{
    return zk_depth > ZK_MAX_DEPTH ? ZK_MAX_DEPTH : zk_depth;
}

/**
 * The name of the function of a call.
 *
 * @param func Its number, as zk_frame counts them.
 * @return The name; `main` for the main function.
 */
static const char *zk_frame_name(uint32_t func)
{
    RET_IF(func == 0 || func > zk_fn_count, "main");
    return zk_fn_names[func - 1].name;
}

/**
 * Write a site as `<file>:<line>:<column>`.
 *
 * @param site The index of the site, 0 for none.
 * @param buf Where the text goes.
 * @param size The room in buf.
 * @return 1 when the build keeps the site, else 0.
 */
static int zk_place(uint32_t site, char *buf, size_t size)
{
#if ZK_DEBUG_INFO
    RET_IF(site == 0, 0);
    zk_site s = zk_sites[site];
    unsigned long line = s.line, col = s.col;
    snprintf(buf, size, "%s:%lu:%lu", zk_site_files[s.file], line, col);
    return 1;
#else
    (void)site;
    (void)buf;
    (void)size;
    return 0;
#endif
}

/**
 * Write one line of a call: its function, and its site when the build keeps
 * it.
 *
 * @param f The call.
 */
static void zk_call_line(zk_frame f)
{
    char place[512];
    const char *names[] = {"func", "place"};
    const char *values[] = {zk_frame_name(f.func), place};
    int known = zk_place(f.site, place, sizeof place);
    zk_fill_template(
        known ? zk_call_at_text : zk_call_text, known ? 2 : 1, names, values
    );
    fputc('\n', stderr);
}

/**
 * Write a line of a template with one count in it.
 *
 * @param tmpl The template, which names the count `n`.
 * @param n The count.
 */
static void zk_count_line(const char *tmpl, zk_umax n);

/**
 * Write the place of a stop after its reason: ` (in <func>, at <place>)`, or
 * ` (in <func>)` when the build keeps no sites; nothing when the calls are
 * not kept.
 */
static void zk_where(void)
{
    RET_IF(!ZK_TRACK);
    zk_frame f = zk_frames[zk_top()];
    char place[512];
    const char *names[] = {"func", "place"};
    const char *values[] = {zk_frame_name(f.func), place};
    int known = zk_place(f.site, place, sizeof place);
    fputc(' ', stderr);
    zk_fill_template(known ? zk_where_text : zk_in_text, known ? 2 : 1, names, values);
}

/**
 * Whether a short backtrace leaves a call out: the main function, unless the
 * stop is in it.
 *
 * @param i The index of the call.
 * @param top The index of the last call.
 * @return 1 when it is left out.
 */
static int zk_hidden_call(uint32_t i, uint32_t top)
{
    return i != top && zk_frames[i].func == 0;
}

/**
 * Write the backtrace of a stop, as much of it as `ZAKA_BACKTRACE` asks:
 * `full` each call, `0` none; else a run of one call once, the first
 * ZK_SHOWN_CALLS lines, and a last line that tells how to see the rest. A
 * backtrace of one call is left out.
 */
static void zk_backtrace(void)
{
    RET_IF(!ZK_TRACK);
    const char *mode = getenv("ZAKA_BACKTRACE");
    RET_IF(mode && strcmp(mode, "0") == 0);
    int full = mode && strcmp(mode, "full") == 0;
    uint32_t top = zk_top();
    uint32_t shown = 0;
    DO_FOR(k, top + 1) shown += full || !zk_hidden_call((uint32_t)k, top);
    RET_IF(shown < 2);
    fputs(zk_backtrace_text, stderr);
    fputc('\n', stderr);
    zk_umax runs = 0, hidden = 0;
    int left_out = 0;
    for (uint32_t i = top + 1; i-- > 0;)
    {
        zk_frame f = zk_frames[i];
        int skip = !full && zk_hidden_call(i, top);
        left_out |= skip;
        NEXT_IF(skip);
        zk_umax n = 1;
        while (!full && i > 0 && zk_frames[i - 1].func == f.func &&
               zk_frames[i - 1].site == f.site)
        {
            i--;
            n++;
        }
        int past = !full && runs >= ZK_SHOWN_CALLS;
        DO_IF(past, hidden += n);
        NEXT_IF(past);
        runs++;
        zk_call_line(f);
        DO_IF(n > 1, zk_count_line(zk_call_again_text, n - 1));
        left_out |= n > 1;
    }
    DO_IF(hidden, zk_count_line(zk_calls_more_text, hidden));
    left_out |= hidden > 0;
    RET_IF(full || !left_out);
    const char *names[] = {"var"};
    const char *values[] = {"ZAKA_BACKTRACE"};
    zk_fill_template(zk_full_hint_text, 1, names, values);
    fputc('\n', stderr);
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
    zk_where();
    fputc('\n', stderr);
    zk_backtrace();
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

static void zk_count_line(const char *tmpl, zk_umax n)
{
    char buf[48];
    const char *names[] = {"n"};
    const char *values[] = {zk_dec(n, buf)};
    zk_fill_template(tmpl, 1, names, values);
    fputc('\n', stderr);
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

/** The count of Zaka calls in progress, which ZK_ENTER and ZK_LEAVE keep. */
uint32_t zk_depth = 0;

/** The calls in progress, main at 0; ZK_ENTER and ZK_AT keep it. */
zk_frame zk_frames[ZK_MAX_DEPTH + 1];

/** Stop the run: more nested calls than the depth limit of the build. */
_Noreturn void zk_too_deep(void)
{
    char a[48];
    const char *names[] = {"depth"};
    const char *values[] = {zk_dec(ZK_MAX_DEPTH, a)};
    zk_stop(ZK_TRAP_TOO_DEEP, 1, names, values);
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
    unsigned char *block = buf->len ? zk_seq_new(buf->len) : NULL;
    DO_IF(block, memcpy(block, buf->data, buf->len));
    free(buf->data);
    zk_seq_put(slot, block, buf->len, buf->len);
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
void zk_put_burmese(const char *s)
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
 * Write an unsigned integer as wide as `zk_umax`.
 *
 * @param v The value.
 */
void zk_put_u(zk_umax v)
{
    char buf[48];
    zk_put_burmese(zk_dec(v, buf));
}

/**
 * Write a signed integer as wide as `zk_imax`.
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
