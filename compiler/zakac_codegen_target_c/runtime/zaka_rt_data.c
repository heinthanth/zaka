/* The runtime of a Zaka program that the Zaka Compiler C target builds: the
 * heap, lists, text and the print of each kind. Each function follows the
 * VM op of the same name, so a program prints the same here as there. */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "zaka_c_sugars.h"
#include "zaka_rt.h"

/** The octets of a word: an address or a count. */
#define ZK_WORD sizeof(void *)
/** No limit on the cells of a slice source. */
#define ZK_NO_LIMIT SIZE_MAX

/* ---- storage ---- */

/**
 * A new heap block, or a stop of the run when no room is left.
 *
 * @param size The size in octets.
 * @return The block.
 */
static unsigned char *zk_block(size_t size)
{
    unsigned char *p = malloc(size ? size : 1);
    DO_IF(!p, zk_out_of_mem(size));
    return p;
}

/**
 * A block moved to a new size, or a stop of the run when no room is left.
 *
 * @param p The block, or NULL for a new one.
 * @param size The new size in octets.
 * @return The block.
 */
static unsigned char *zk_grow(unsigned char *p, size_t size)
{
    unsigned char *moved = realloc(p, size ? size : 1);
    DO_IF(!moved, zk_out_of_mem(size));
    return moved;
}

/**
 * Copy a value of a list or struct kind.
 *
 * @param to The address written.
 * @param from The address read; the two can overlap.
 * @param size The size in octets.
 */
void zk_put_block(unsigned char *to, const unsigned char *from, size_t size)
{
    RET_IF(size == 0 || to == from);
    ZK_NULL_CHECK(to);
    ZK_NULL_CHECK(from);
    memmove(to, from, size);
}

/**
 * An address moved count steps of stride octets. The VM wraps the sum
 * and checks a step from null (the mem checks).
 *
 * @param p The address.
 * @param count The steps, of each sign.
 * @param stride The octets of a step.
 * @return The address.
 */
unsigned char *zk_ptr_add(unsigned char *p, zk_imax count, size_t stride)
{
    DO_IF(ZK_MEM_CHECKS && !p && count != 0, zk_null_deref(p));
    uintptr_t moved = (uintptr_t)p + (uintptr_t)((zk_umax)count * (zk_umax)stride);
    return (unsigned char *)moved;
}

/**
 * The steps of stride octets from q to p.
 *
 * @param p The later address.
 * @param q The earlier address.
 * @param stride The octets of a step.
 * @return The steps.
 */
zk_imax zk_ptr_sub(const unsigned char *p, const unsigned char *q, size_t stride)
{
    zk_imax distance = (zk_imax)(intptr_t)((uintptr_t)p - (uintptr_t)q);
    return distance / (zk_imax)stride;
}

/**
 * A new block of count cells, zeroed as the VM zeroes it; a block of 0
 * octets is a real block.
 *
 * @param count The cell count.
 * @param stride The octets of a cell.
 * @return The block, or NULL when the size overflows or no room is left.
 */
unsigned char *zk_alloc(zk_umax count, size_t stride)
{
    RET_IF(stride && count > SIZE_MAX / stride, NULL);
    size_t size = (size_t)count * stride;
    return calloc(size ? size : 1, 1);
}

/**
 * A block moved to a new size of count cells.
 *
 * @param p The block, or NULL for a new one.
 * @param count The cell count.
 * @param stride The octets of a cell.
 * @return The block; the run stops when the size overflows or no room is
 *         left.
 */
unsigned char *zk_realloc(unsigned char *p, zk_umax count, size_t stride)
{
    DO_IF(stride && count > SIZE_MAX / stride, zk_out_of_mem((zk_umax)-1));
    return zk_grow(p, (size_t)count * stride);
}

/**
 * Copy count cells of stride octets.
 *
 * @param src The first cell read.
 * @param count The cell count.
 * @param dst The first cell written.
 * @param stride The octets of a cell.
 * @param overlap 1 when the ranges can overlap; with 0 the mem checks stop
 *        the run when the two do.
 */
void zk_blit(
    const unsigned char *src, zk_umax count, unsigned char *dst, size_t stride,
    int overlap
)
{
    size_t len = (size_t)count * stride;
    RET_IF(len == 0);
    uintptr_t s = (uintptr_t)src, d = (uintptr_t)dst;
    int apart = s + len <= d || d + len <= s;
    DO_IF(ZK_MEM_CHECKS && !overlap && !apart, zk_addr_trap(ZK_TRAP_OVERLAP, dst));
    ZK_NULL_CHECK(src);
    ZK_NULL_CHECK(dst);
    memmove(dst, src, len);
}

/**
 * Write one cell into each of count cells.
 *
 * @param dst The first cell.
 * @param count The cell count.
 * @param cell The value of a cell.
 * @param size The octets of a cell.
 */
void zk_fill(unsigned char *dst, zk_umax count, const unsigned char *cell, size_t size)
{
    DO_IF(count, ZK_NULL_CHECK(dst));
    DO_FOR(i, count) memcpy(dst + i * size, cell, size);
}

/**
 * The function a function value holds.
 *
 * @param f The value.
 * @return The function; the run stops when the value is null.
 */
zk_fn zk_fn_check(zk_fn f)
{
    DO_IF(!f, zk_plain_trap(ZK_TRAP_NULL_CALL));
    return f;
}

/* ---- the words of lists and text ---- */

unsigned char *zk_seq_base(const unsigned char *at)
{
    unsigned char *p;
    memcpy(&p, at, ZK_WORD);
    return p;
}

size_t zk_seq_len(const unsigned char *at)
{
    size_t n;
    memcpy(&n, at + ZK_WORD, ZK_WORD);
    return n;
}

/**
 * The third word of a plain list or text: the size of its heap block, 0
 * when it owns none.
 *
 * @param at The value.
 * @return The room in octets.
 */
static size_t zk_seq_room(const unsigned char *at)
{
    size_t n;
    memcpy(&n, at + 2 * ZK_WORD, ZK_WORD);
    return n;
}

void zk_seq_put(unsigned char *at, unsigned char *base, size_t len, size_t room)
{
    memcpy(at, &base, ZK_WORD);
    memcpy(at + ZK_WORD, &len, ZK_WORD);
    memcpy(at + 2 * ZK_WORD, &room, ZK_WORD);
}

/**
 * Write static text into a slot.
 *
 * @param slot The slot: text (three words) or a view (two).
 * @param s The octets.
 * @param len Their count.
 * @param size The size of the slot.
 */
void zk_text_lit(unsigned char *slot, const char *s, size_t len, size_t size)
{
    unsigned char *base = len ? (unsigned char *)s : NULL;
    memcpy(slot, &base, ZK_WORD);
    memcpy(slot + ZK_WORD, &len, ZK_WORD);
    RET_IF(size < 3 * ZK_WORD);
    size_t room = 0;
    memcpy(slot + 2 * ZK_WORD, &room, ZK_WORD);
}

/* ---- kinds ---- */

/**
 * A kind as storage holds it: an enumeration as its value kind.
 *
 * @param k The kind.
 * @return The kind.
 */
static const zk_kind *zk_repr(const zk_kind *k)
{
    while (k->tag == ZK_K_ENUM) k = k->cell;
    return k;
}

/**
 * The octets of a cell of a plain list or text: 1 for text.
 *
 * @param k The kind of the plain list or text.
 * @return The stride.
 */
static size_t zk_stride(const zk_kind *k) { return k->cell ? k->cell->size : 1; }

/** A visit of a text or a plain list inside a value. */
typedef void (*zk_visit)(unsigned char *at, const zk_kind *k);

/**
 * Visit each text and plain list a value holds, through its fields and
 * cells, as the VM's blocks_in walks them.
 *
 * @param at The value.
 * @param k Its kind.
 * @param f The visit.
 */
static void zk_each_block(unsigned char *at, const zk_kind *k, zk_visit f)
{
    k = zk_repr(k);
    RET_IF(!k->holds_text);
    if (k->tag == ZK_K_STRUCT || k->tag == ZK_K_FALLIBLE)
    {
        DO_FOR(i, k->len) zk_each_block(at + k->fields[i].offset, k->fields[i].kind, f);
        return;
    }
    if (k->tag == ZK_K_LIST)
    {
        DO_FOR(i, k->len) zk_each_block(at + i * k->cell->size, k->cell, f);
        return;
    }
    DO_IF(k->tag == ZK_K_TEXT || k->tag == ZK_K_SEQ, f(at, k));
}

/**
 * The cell kind of a plain list when its cells hold text of their own.
 *
 * @param k The kind of the plain list or text.
 * @return The cell kind, or NULL.
 */
static const zk_kind *zk_text_cells(const zk_kind *k)
{
    RET_IF(k->tag != ZK_K_SEQ || !k->cell->holds_text, NULL);
    return k->cell;
}

/**
 * Free a block a text or plain list owns, with the text of its cells.
 *
 * @param at The text or plain list.
 * @param k Its kind.
 */
static void zk_drop_block(unsigned char *at, const zk_kind *k)
{
    unsigned char *base = zk_seq_base(at);
    size_t count = zk_seq_len(at);
    const zk_kind *cell = zk_text_cells(k);
    if (zk_seq_room(at) > 0)
    {
        DO_IF(cell, DO_FOR(i, count) zk_drop(base + i * cell->size, cell));
        free(base);
    }
    memset(at, 0, 3 * ZK_WORD);
}

void zk_drop(unsigned char *at, const zk_kind *k)
{
    zk_each_block(at, k, zk_drop_block);
}

/**
 * Give a text or plain list a heap block of its own, with copies of the
 * text of its cells.
 *
 * @param at The text or plain list.
 * @param k Its kind.
 */
static void zk_clone_block(unsigned char *at, const zk_kind *k)
{
    size_t count = zk_seq_len(at), stride = zk_stride(k), size = count * stride;
    DONE_IF(size == 0, zk_seq_put(at, NULL, 0, 0));
    unsigned char *block = zk_block(size);
    memcpy(block, zk_seq_base(at), size);
    zk_seq_put(at, block, count, size);
    const zk_kind *cell = zk_text_cells(k);
    DO_IF(cell, DO_FOR(i, count) zk_dup_seqs(block + i * stride, cell));
}

/**
 * Give a text or plain list with cells a block of its own.
 *
 * @param at The text or plain list.
 * @param k Its kind.
 */
static void zk_dup_block(unsigned char *at, const zk_kind *k)
{
    DO_IF(zk_seq_base(at), zk_clone_block(at, k));
}

void zk_dup_seqs(unsigned char *at, const zk_kind *k)
{
    zk_each_block(at, k, zk_dup_block);
}

/**
 * Lend a text or plain list: its room becomes 0, so a drop frees nothing.
 *
 * @param at The text or plain list.
 * @param k Its kind.
 */
static void zk_lend_block(unsigned char *at, const zk_kind *k)
{
    (void)k;
    size_t room = 0;
    memcpy(at + 2 * ZK_WORD, &room, ZK_WORD);
}

void zk_lend_seqs(unsigned char *at, const zk_kind *k)
{
    zk_each_block(at, k, zk_lend_block);
}

/**
 * Give a text or plain list with a room of 0 a block of its own.
 *
 * @param at The text or plain list.
 * @param k Its kind.
 */
static void zk_own_block(unsigned char *at, const zk_kind *k)
{
    DO_IF(zk_seq_room(at) == 0, zk_clone_block(at, k));
}

unsigned char *zk_own(unsigned char *at, const zk_kind *k)
{
    zk_each_block(at, k, zk_own_block);
    return at;
}

void zk_zero(unsigned char *at, size_t size) { DO_IF(size, memset(at, 0, size)); }

void zk_to_heap(
    unsigned char *dst, const unsigned char *src, const zk_kind *list, int fixed
)
{
    size_t count = list->len, size = count * list->cell->size;
    DONE_IF(size == 0, zk_seq_put(dst, NULL, 0, 0));
    unsigned char *block = zk_block(size);
    memcpy(block, src, size);
    zk_seq_put(dst, block, count, fixed ? 0 : size);
}

/* ---- cells and slices ---- */

size_t zk_cell_of(zk_imax index, size_t len, int first)
{
    int outside = index < first || index >= (zk_imax)len + first;
    DO_IF(
        ZK_LANG_CHECKS && outside,
        zk_range_trap(
            ZK_TRAP_INDEX_OUT_OF_RANGE, index, first, (zk_imax)len + first - 1
        )
    );
    return (size_t)(index - first);
}

/**
 * The character a number names, counted from first.
 *
 * @param index The number.
 * @param len The character count.
 * @param first The number of the first character: 0 or 1.
 * @return The character, counted from 0.
 */
static size_t zk_char_index(zk_imax index, size_t len, int first)
{
    int outside = index < first || index >= (zk_imax)len + first;
    DO_IF(
        ZK_LANG_CHECKS && outside,
        zk_range_trap(ZK_TRAP_CHAR_OUT_OF_RANGE, index, first, (zk_imax)len + first - 1)
    );
    return (size_t)(index - first);
}

/**
 * The place a number names, in front of a cell or at the end.
 *
 * @param index The number.
 * @param count The cell count.
 * @param first The number of the first cell: 0 or 1.
 * @return The place, counted from 0.
 */
static size_t zk_place_of(zk_imax index, size_t count, int first)
{
    int outside = index < first || index > (zk_imax)count + first;
    DO_IF(
        ZK_LANG_CHECKS && outside,
        zk_range_trap(ZK_TRAP_NO_PLACE_TO_INSERT, index, first, (zk_imax)count + first)
    );
    return (size_t)(index - first);
}

unsigned char *zk_slice_cell(
    const unsigned char *at, zk_imax index, int first, size_t stride
)
{
    size_t cell = zk_cell_of(index, zk_seq_len(at), first);
    return zk_seq_base(at) + cell * stride;
}

/**
 * Stop the run when an octet of text is inside a character.
 *
 * @param base The first octet of the text.
 * @param at The octet, counted from base.
 */
static void zk_edge_check(const unsigned char *base, size_t at)
{
    DO_IF((base[at] & 0xC0) == 0x80, zk_addr_trap(ZK_TRAP_TEXT_CUT, base + at));
}

void zk_make_slice(
    unsigned char *slot, unsigned char *base, zk_umax start, zk_umax len, size_t limit,
    size_t stride, int text
)
{
    zk_umax end = start + len;
    int past = limit != ZK_NO_LIMIT && (end < start || end > limit);
    if (ZK_LANG_CHECKS && past)
    {
        const char *names[] = {"len", "start", "limit"};
        zk_imax nums[] = {(zk_imax)len, (zk_imax)start, (zk_imax)limit};
        zk_nums_trap(ZK_TRAP_SLICE_OUT_OF_RANGE, 3, names, nums);
    }
    int edges = ZK_LANG_CHECKS && text && limit != ZK_NO_LIMIT && base;
    DO_IF(edges && start < limit, zk_edge_check(base, (size_t)start));
    DO_IF(edges && end < limit, zk_edge_check(base, (size_t)end));
    unsigned char *at = zk_ptr_add(base, (zk_imax)start, stride);
    size_t count = (size_t)len;
    memcpy(slot, &at, ZK_WORD);
    memcpy(slot + ZK_WORD, &count, ZK_WORD);
}

/**
 * The octet count of a valid UTF-8 sequence that starts at s, or 0 when
 * none does.
 *
 * @param s The octets.
 * @param n Their count.
 * @return The count of the sequence.
 */
static size_t zk_utf8_len(const unsigned char *s, size_t n)
{
    unsigned char b = s[0];
    size_t need = b < 0x80         ? 1
                  : (b >> 5) == 6  ? 2
                  : (b >> 4) == 14 ? 3
                  : (b >> 3) == 30 ? 4
                                   : 0;
    RET_IF(need == 0 || need > n, 0);
    DO_FOR(i, need - 1) RET_IF((s[i + 1] & 0xC0) != 0x80, 0);
    uint32_t c = need == 1 ? b : (uint32_t)(b & (0xFF >> (need + 1)));
    DO_FOR(i, need - 1) c = (c << 6) | (s[i + 1] & 0x3F);
    static const uint32_t least[] = {0, 0, 0x80, 0x800, 0x10000};
    RET_IF(c < least[need] || c > 0x10FFFF || (c >= 0xD800 && c <= 0xDFFF), 0);
    return need;
}

int zk_is_utf8(const unsigned char *s, size_t n)
{
    size_t i = 0;
    while (i < n)
    {
        size_t step = zk_utf8_len(s + i, n - i);
        RET_IF(step == 0, 0);
        i += step;
    }
    return 1;
}

void zk_check_utf8(const unsigned char *at)
{
    RET_IF(!ZK_LANG_CHECKS);
    const unsigned char *base = zk_seq_base(at);
    size_t count = zk_seq_len(at), i = 0;
    while (i < count)
    {
        size_t n = zk_utf8_len(base + i, count - i);
        DO_IF(n == 0, zk_addr_trap(ZK_TRAP_BAD_UTF8, base + i));
        i += n;
    }
}

/* ---- plain lists ---- */

/**
 * Put a cell in front of place n of a plain list, as the VM's put_cell
 * does: the list takes a block of its own first, and a full block moves to
 * a larger one.
 *
 * @param at The plain list.
 * @param k Its kind.
 * @param src The new cell, moved out and zeroed.
 * @param n The place.
 */
static void zk_put_cell(
    unsigned char *at, const zk_kind *k, unsigned char *src, size_t n
)
{
    size_t stride = k->cell->size;
    zk_own(at, k);
    unsigned char *block = zk_seq_base(at);
    size_t count = zk_seq_len(at), held = zk_seq_room(at), room = held;
    size_t need = (count + 1) * stride;
    if (need > held)
    {
        room = need > 2 * held ? need : 2 * held;
        room = room > 4 * stride ? room : 4 * stride;
        block = zk_grow(held == 0 ? NULL : block, room);
    }
    DO_IF(
        count > n,
        memmove(block + (n + 1) * stride, block + n * stride, (count - n) * stride)
    );
    DO_IF(stride, memcpy(block + n * stride, src, stride));
    zk_zero(src, stride);
    zk_seq_put(at, block, count + 1, room);
}

void zk_add_cell(unsigned char *at, const zk_kind *k, unsigned char *src)
{
    zk_put_cell(at, k, src, zk_seq_len(at));
}

void zk_insert_cell(
    unsigned char *at, const zk_kind *k, zk_imax index, int first, unsigned char *src
)
{
    zk_put_cell(at, k, src, zk_place_of(index, zk_seq_len(at), first));
}

/* ---- text ---- */

/**
 * The octet where character n of text starts, or the octet count when the
 * text has n characters or fewer.
 *
 * @param s The octets.
 * @param len Their count.
 * @param n The character, counted from 0.
 * @return The octet.
 */
static size_t zk_char_start(const unsigned char *s, size_t len, size_t n)
{
    size_t seen = 0;
    DO_FOR(i, len)
    {
        NEXT_IF((s[i] & 0xC0) == 0x80);
        RET_IF(seen == n, i);
        seen++;
    }
    return len;
}

/**
 * The character count of octets of text.
 *
 * @param s The octets.
 * @param len Their count.
 * @return The count.
 */
static size_t zk_chars_in(const unsigned char *s, size_t len)
{
    size_t count = 0;
    DO_FOR(i, len) count += (s[i] & 0xC0) != 0x80;
    return count;
}

/**
 * The character that starts at an octet, or U+FFFD when none does.
 *
 * @param s The octets from there.
 * @param n Their count.
 * @return The character.
 */
static uint32_t zk_decode(const unsigned char *s, size_t n)
{
    size_t need = zk_utf8_len(s, n);
    RET_IF(need == 0, 0xFFFD);
    uint32_t c = need == 1 ? s[0] : (uint32_t)(s[0] & (0xFF >> (need + 1)));
    DO_FOR(i, need - 1) c = (c << 6) | (s[i + 1] & 0x3F);
    return c;
}

/**
 * The UTF-8 octets of a character.
 *
 * @param c The character.
 * @param out Room for four octets.
 * @return Their count.
 */
static size_t zk_encode(uint32_t c, unsigned char out[4])
{
    static const unsigned char lead[] = {0, 0, 0xc0, 0xe0, 0xf0};
    size_t n = c < 0x80 ? 1 : c < 0x800 ? 2 : c < 0x10000 ? 3 : 4;
    for (size_t i = n - 1; i > 0; i--, c >>= 6)
        out[i] = (unsigned char)(0x80 | (c & 0x3f));
    out[0] = (unsigned char)(lead[n] | c);
    return n;
}

/**
 * Put octets as the new text at a slot, as the VM's put_text does: a full
 * block moves to a larger one, of at least 16 octets.
 *
 * @param at The text, which owns its block.
 * @param s The octets, apart from the block.
 * @param len Their count.
 */
static void zk_put_text(unsigned char *at, const unsigned char *s, size_t len)
{
    unsigned char *block = zk_seq_base(at);
    size_t held = zk_seq_room(at), room = held;
    if (len > held)
    {
        room = len > 2 * held ? len : 2 * held;
        room = room > 16 ? room : 16;
        block = zk_grow(held == 0 ? NULL : block, room);
    }
    DO_IF(len, memcpy(block, s, len));
    zk_seq_put(at, block, len, room);
}

/**
 * Splice octets into text: the octets from `from` to `to` give place to the
 * added ones, then the text takes a block of its own and holds the result.
 *
 * @param at The text.
 * @param from The first octet replaced.
 * @param to The octet after the last one replaced.
 * @param add The added octets.
 * @param n Their count.
 */
static void zk_splice(
    unsigned char *at, size_t from, size_t to, const unsigned char *add, size_t n
)
{
    const unsigned char *base = zk_seq_base(at);
    size_t len = zk_seq_len(at), total = len - (to - from) + n;
    unsigned char *edited = zk_block(total);
    DO_IF(from, memcpy(edited, base, from));
    DO_IF(n, memcpy(edited + from, add, n));
    DO_IF(len > to, memcpy(edited + from + n, base + to, len - to));
    static const zk_kind text = {
        ZK_K_TEXT, 3 * sizeof(void *), NULL, 0, NULL, NULL, NULL, 1
    };
    zk_own(at, &text);
    zk_put_text(at, edited, total);
    free(edited);
}

void zk_add_text(unsigned char *at, const unsigned char *text)
{
    size_t len = zk_seq_len(at);
    zk_splice(at, len, len, zk_seq_base(text), zk_seq_len(text));
}

void zk_add_char(unsigned char *at, uint32_t c)
{
    unsigned char octets[4];
    size_t n = zk_encode(c, octets), len = zk_seq_len(at);
    zk_splice(at, len, len, octets, n);
}

/**
 * The octet of the place in front of character index of text.
 *
 * @param at The text.
 * @param index The number.
 * @param first The number of the first character.
 * @return The octet.
 */
static size_t zk_insert_at(const unsigned char *at, zk_imax index, int first)
{
    const unsigned char *base = zk_seq_base(at);
    size_t len = zk_seq_len(at);
    size_t n = zk_place_of(index, zk_chars_in(base, len), first);
    return zk_char_start(base, len, n);
}

void zk_insert_text(
    unsigned char *at, zk_imax index, int first, const unsigned char *text
)
{
    size_t split = zk_insert_at(at, index, first);
    zk_splice(at, split, split, zk_seq_base(text), zk_seq_len(text));
}

void zk_insert_char(unsigned char *at, zk_imax index, int first, uint32_t c)
{
    unsigned char octets[4];
    size_t n = zk_encode(c, octets), split = zk_insert_at(at, index, first);
    zk_splice(at, split, split, octets, n);
}

void zk_set_char(unsigned char *at, zk_imax index, int first, uint32_t c)
{
    const unsigned char *base = zk_seq_base(at);
    size_t len = zk_seq_len(at);
    size_t n = zk_char_index(index, zk_chars_in(base, len), first);
    size_t from = zk_char_start(base, len, n), to = zk_char_start(base, len, n + 1);
    unsigned char octets[4];
    size_t count = zk_encode(c, octets);
    zk_splice(at, from, to, octets, count);
}

/**
 * The octets a range of text covers: characters counted from 1, or octets
 * counted from 0, both ends in the range.
 *
 * @param at The text.
 * @param start The first character or octet.
 * @param end The last one.
 * @param chars 1 for characters, 0 for octets.
 * @param range Where the first octet and the octet after the last go.
 */
static void zk_span(
    const unsigned char *at, zk_imax start, zk_imax end, int chars, size_t range[2]
)
{
    const unsigned char *base = zk_seq_base(at);
    size_t len = zk_seq_len(at), count = chars ? zk_chars_in(base, len) : len;
    size_t lo = chars ? zk_char_index(start, count, 1) : zk_cell_of(start, count, 0);
    size_t hi = chars ? zk_char_index(end, count, 1) : zk_cell_of(end, count, 0);
    if (ZK_LANG_CHECKS && hi < lo)
    {
        const char *names[] = {"start", "end"};
        zk_imax nums[] = {(zk_imax)lo + chars, (zk_imax)hi + chars};
        zk_nums_trap(ZK_TRAP_END_BEFORE_START, 2, names, nums);
    }
    range[0] = chars ? zk_char_start(base, len, lo) : lo;
    range[1] = chars ? zk_char_start(base, len, hi + 1) : hi + 1;
    RET_IF(chars || !ZK_LANG_CHECKS);
    DO_IF(range[0] < len, zk_edge_check(base, range[0]));
    DO_IF(range[1] < len, zk_edge_check(base, range[1]));
}

void zk_replace_text(
    unsigned char *at, zk_imax start, zk_imax end, int chars, const unsigned char *text
)
{
    size_t range[2];
    zk_span(at, start, end, chars, range);
    zk_splice(at, range[0], range[1], zk_seq_base(text), zk_seq_len(text));
}

void zk_replace_char(
    unsigned char *at, zk_imax start, zk_imax end, int chars, uint32_t c
)
{
    size_t range[2];
    zk_span(at, start, end, chars, range);
    unsigned char octets[4];
    size_t n = zk_encode(c, octets);
    zk_splice(at, range[0], range[1], octets, n);
}

/**
 * Remove the last character of text.
 *
 * @param at The text.
 */
static void zk_remove_char(unsigned char *at)
{
    const unsigned char *base = zk_seq_base(at);
    size_t len = zk_seq_len(at);
    DO_IF(len == 0, zk_plain_trap(ZK_TRAP_NO_CHAR_TO_REMOVE));
    size_t last = len - 1;
    while (last > 0 && (base[last] & 0xC0) == 0x80) last--;
    zk_seq_put(at, (unsigned char *)base, last, zk_seq_room(at));
}

void zk_remove_cell(unsigned char *at, const zk_kind *k)
{
    DONE_IF(k->tag == ZK_K_TEXT, zk_remove_char(at));
    size_t count = zk_seq_len(at), room = zk_seq_room(at);
    DO_IF(count == 0, zk_plain_trap(ZK_TRAP_NO_CELL_TO_REMOVE));
    unsigned char *base = zk_seq_base(at);
    DO_IF(room > 0, zk_drop(base + (count - 1) * k->cell->size, k->cell));
    zk_seq_put(at, base, count - 1, room);
}

size_t zk_char_count(const unsigned char *at)
{
    return zk_chars_in(zk_seq_base(at), zk_seq_len(at));
}

uint32_t zk_char_at(const unsigned char *at, zk_imax index, int first)
{
    const unsigned char *base = zk_seq_base(at);
    size_t len = zk_seq_len(at);
    size_t n = zk_char_index(index, zk_chars_in(base, len), first);
    size_t from = zk_char_start(base, len, n);
    return zk_decode(base + from, len - from);
}

uint32_t zk_char_at_octet(const unsigned char *at, zk_umax index)
{
    const unsigned char *base = zk_seq_base(at);
    size_t len = zk_seq_len(at);
    int outside = index >= len;
    uint32_t c = outside ? 0xFFFD : zk_decode(base + index, len - (size_t)index);
    DO_IF(
        ZK_LANG_CHECKS && (outside || c == 0xFFFD),
        zk_range_trap(ZK_TRAP_CHAR_OUT_OF_RANGE, (zk_imax)index, 0, (zk_imax)len - 1)
    );
    return c;
}

/* ---- comparison ---- */

int zk_text_eq(const unsigned char *a, const unsigned char *b)
{
    size_t n = zk_seq_len(a);
    RET_IF(n != zk_seq_len(b), 0);
    return n == 0 || memcmp(zk_seq_base(a), zk_seq_base(b), n) == 0;
}

/**
 * Whether two runs of cells are equal.
 *
 * @param a The first cell of one.
 * @param b The first cell of the other.
 * @param cell The kind of a cell.
 * @param count The cell count.
 * @return 1 when each cell is equal.
 */
static int zk_cells_eq(
    const unsigned char *a, const unsigned char *b, const zk_kind *cell, size_t count
)
{
    DO_FOR(i, count) RET_IF(!zk_eq(a + i * cell->size, b + i * cell->size, cell), 0);
    return 1;
}

int zk_eq(const unsigned char *a, const unsigned char *b, const zk_kind *k)
{
    k = zk_repr(k);
    switch (k->tag)
    {
    case ZK_K_TEXT:
    case ZK_K_VIEW:
        return zk_text_eq(a, b);
    case ZK_K_SEQ:
    case ZK_K_SLICE:
        RET_IF(zk_seq_len(a) != zk_seq_len(b), 0);
        return zk_cells_eq(zk_seq_base(a), zk_seq_base(b), k->cell, zk_seq_len(a));
    case ZK_K_LIST:
        return zk_cells_eq(a, b, k->cell, k->len);
    case ZK_K_STRUCT:
    case ZK_K_FALLIBLE:
        DO_FOR(i, k->len)
        {
            size_t at = k->fields[i].offset;
            RET_IF(!zk_eq(a + at, b + at, k->fields[i].kind), 0);
        }
        return 1;
    case ZK_K_FLOAT:
        RET_IF(k->size == 4, zk_ld_f32(a) == zk_ld_f32(b));
        return zk_ld_f64(a) == zk_ld_f64(b);
    default:
        return memcmp(a, b, k->size) == 0;
    }
}

/* ---- print ---- */

/** The separator of cells and fields: U+104A and a space. */
static const char zk_sep[] = "\xe1\x81\x8a ";

/**
 * Write text, in quotes when it is nested in a list or a struct: a
 * backslash and a quote escaped, each brace doubled.
 *
 * @param at The text or view.
 * @param nested 1 for quotes.
 */
static void zk_show_text(const unsigned char *at, int nested)
{
    const unsigned char *s = zk_seq_base(at);
    size_t len = zk_seq_len(at);
    DONE_IF(!nested, zk_emit(s, len));
    zk_emit("\"", 1);
    DO_FOR(i, len)
    {
        char c = (char)s[i];
        DO_IF(c == '\\' || c == '"', zk_emit("\\", 1));
        DO_IF(c == '{' || c == '}', zk_emit(&c, 1));
        zk_emit(&c, 1);
    }
    zk_emit("\"", 1);
}

/**
 * Write a run of cells between `[` and `]`.
 *
 * @param base The first cell.
 * @param cell The kind of a cell.
 * @param count The cell count.
 */
static void zk_show_cells(const unsigned char *base, const zk_kind *cell, size_t count)
{
    zk_emit("[", 1);
    DO_FOR(i, count)
    {
        DO_IF(i, zk_emit(zk_sep, sizeof zk_sep - 1));
        zk_show(base + i * cell->size, cell, 1);
    }
    zk_emit("]", 1);
}

/**
 * Write a struct: its name, then each field as `name: value`.
 *
 * @param at The struct.
 * @param k Its kind.
 */
static void zk_show_struct(const unsigned char *at, const zk_kind *k)
{
    zk_emit(k->name, strlen(k->name));
    zk_emit("(", 1);
    DO_FOR(i, k->len)
    {
        DO_IF(i, zk_emit(zk_sep, sizeof zk_sep - 1));
        zk_emit(k->fields[i].name, strlen(k->fields[i].name));
        zk_emit(": ", 2);
        zk_show(at + k->fields[i].offset, k->fields[i].kind, 1);
    }
    zk_emit(")", 1);
}

/**
 * The field of a struct with a name.
 *
 * @param k The struct kind.
 * @param name The name.
 * @return The field, or NULL.
 */
const zk_field *zk_field_named(const zk_kind *k, const char *name)
{
    DO_FOR(i, k->len) RET_IF(!strcmp(k->fields[i].name, name), &k->fields[i]);
    return NULL;
}

/**
 * Write a result that can fail: its good value or its error value, or the
 * word of a side with no value.
 *
 * @param at The result.
 * @param k Its kind.
 */
static void zk_show_fallible(const unsigned char *at, const zk_kind *k)
{
    const zk_field *tag = zk_field_named(k, "tag");
    int failed = tag && at[tag->offset] == 0;
    const zk_field *part = zk_field_named(k, failed ? "err" : "ok");
    const char *bare = failed ? zk_fault_bare : zk_good_bare;
    DONE_IF(!part, zk_emit(bare, strlen(bare)));
    const char *word = failed ? zk_fault_text : zk_good_text;
    zk_emit(word, strlen(word));
    zk_emit("(", 1);
    zk_show(at + part->offset, part->kind, 1);
    zk_emit(")", 1);
}

/**
 * Write a member of an enumeration: the member whose value prints as the
 * value at the address does, or that value when no member has it.
 *
 * @param at The value.
 * @param k The enumeration kind.
 */
static void zk_show_member(const unsigned char *at, const zk_kind *k)
{
    zk_buf buf;
    zk_sink_open(&buf);
    zk_show(at, k->cell, 0);
    unsigned char held[3 * sizeof(void *)];
    zk_sink_close(held);
    const char *value = (const char *)zk_seq_base(held);
    size_t len = zk_seq_len(held);
    const zk_member *found = NULL;
    DO_FOR(i, k->len)
    {
        const char *m = k->members[i].value;
        NEXT_IF(strlen(m) != len || (len && memcmp(m, value, len)));
        found = &k->members[i];
        break;
    }
    DO_IF(found, zk_emit(found->shown, strlen(found->shown)));
    DO_IF(!found, zk_emit(value, len));
    free(zk_seq_base(held));
}

void zk_put_ptr(const unsigned char *p)
{
    DONE_IF(!p, zk_emit(zk_null_text, strlen(zk_null_text)));
    char buf[40];
    int n = snprintf(buf, sizeof buf, "0x%llx", (unsigned long long)(uintptr_t)p);
    zk_emit(buf, (size_t)n);
}

void zk_put_fn(zk_fn f)
{
    DONE_IF(!f, zk_emit(zk_null_text, strlen(zk_null_text)));
    DO_FOR(i, zk_fn_count)
    {
        NEXT_IF(zk_fn_names[i].fn != f);
        zk_emit(zk_fn_lead, strlen(zk_fn_lead));
        zk_emit(zk_fn_names[i].name, strlen(zk_fn_names[i].name));
        zk_emit(zk_fn_tail, strlen(zk_fn_tail));
        return;
    }
    zk_emit("0x?", 3);
}

/**
 * Write a scalar held in storage.
 *
 * @param at The value.
 * @param k Its kind.
 */
static void zk_show_scalar(const unsigned char *at, const zk_kind *k)
{
    switch (k->tag)
    {
    case ZK_K_SINT:
        DONE_IF(k->size == 1, zk_put_i(zk_ld_i8(at)));
        DONE_IF(k->size == 2, zk_put_i(zk_ld_i16(at)));
        DONE_IF(k->size == 4, zk_put_i(zk_ld_i32(at)));
        DONE_IF(k->size == 8, zk_put_i(zk_ld_i64(at)));
#ifdef __SIZEOF_INT128__
        DONE_IF(k->size == 16, zk_put_i(zk_ld_i128(at)));
#endif
        return;
    case ZK_K_UINT:
        DONE_IF(k->size == 1, zk_put_u(zk_ld_u8(at)));
        DONE_IF(k->size == 2, zk_put_u(zk_ld_u16(at)));
        DONE_IF(k->size == 4, zk_put_u(zk_ld_u32(at)));
        DONE_IF(k->size == 8, zk_put_u(zk_ld_u64(at)));
#ifdef __SIZEOF_INT128__
        DONE_IF(k->size == 16, zk_put_u(zk_ld_u128(at)));
#endif
        return;
    case ZK_K_FLOAT:
        DONE_IF(k->size == 4, zk_put_f32(zk_ld_f32(at)));
        DONE_IF(k->size == 8, zk_put_f64(zk_ld_f64(at)));
        return;
    case ZK_K_CHAR:
        zk_put_char(zk_ld_u32(at));
        return;
    case ZK_K_BOOL:
        zk_put_bool(zk_ld_u8(at));
        return;
    case ZK_K_PTR:
        zk_put_ptr(zk_ld_p(at));
        return;
    case ZK_K_FN:
        zk_put_fn(zk_ld_fn(at));
        return;
    default:
        return;
    }
}

void zk_show(const unsigned char *at, const zk_kind *k, int nested)
{
    switch (k->tag)
    {
    case ZK_K_ENUM:
        zk_show_member(at, k);
        return;
    case ZK_K_STRUCT:
        zk_show_struct(at, k);
        return;
    case ZK_K_FALLIBLE:
        zk_show_fallible(at, k);
        return;
    case ZK_K_TEXT:
    case ZK_K_VIEW:
        zk_show_text(at, nested);
        return;
    case ZK_K_SEQ:
    case ZK_K_SLICE:
        zk_show_cells(zk_seq_base(at), k->cell, zk_seq_len(at));
        return;
    case ZK_K_LIST:
        zk_show_cells(at, k->cell, k->len);
        return;
    default:
        zk_show_scalar(at, k);
        return;
    }
}

/* ---- results ---- */

_Noreturn void zk_not_good(const unsigned char *at, const zk_kind *k)
{
    zk_buf buf;
    zk_sink_open(&buf);
    const zk_kind *shown = zk_repr(k);
    zk_show(at, shown, 0);
    zk_emit("", 1);
    unsigned char held[3 * sizeof(void *)];
    zk_sink_close(held);
    zk_text_trap(ZK_TRAP_NOT_GOOD, (const char *)zk_seq_base(held));
}

_Noreturn void zk_not_good_bare(void) { zk_plain_trap(ZK_TRAP_NOT_GOOD_BARE); }

_Noreturn void zk_not_fault(void) { zk_plain_trap(ZK_TRAP_NOT_FAULT); }
