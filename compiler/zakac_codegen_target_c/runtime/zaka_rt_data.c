/* Zaka runtime - data functions */

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
 * The head in front of the cells of a counted block: text, a plain list, a
 * box or the record of a closure.
 */
typedef struct zk_head
{
    /** The count of values that hold the block; the last to let go frees it. */
    size_t rc;
    /** The octets of its cells. */
    size_t size;
    /** The kind of each cell, so the ring collector can walk what the block
     * holds; NULL for a block with nothing to walk. */
    const zk_kind *cell;
    /** The kind of each cell to drop as the block is freed, or NULL. */
    const zk_kind *drop_cell;
    /** The cells to drop as the block is freed. */
    size_t len;
    /** The next block in the queue of blocks to free. */
    unsigned char *next;
    /** Where the ring collector stands with the block: 0 black, 1 pending,
     * 2 white. */
    unsigned char color;
    /** Whether the block waits in the buffer of the ring collector. */
    unsigned char buffered;
    /** Whether an address of one of its cells was taken: the block is never
     * shared again, so a duplicate copies it. */
    unsigned char pinned;
} zk_head;

/**
 * The octets in front of the cells of a counted block: the head, padded so
 * the cells keep the alignment malloc gives.
 */
#define ZK_SEQ_HEAD ((sizeof(zk_head) + 15) / 16 * 16)

/**
 * The head of a counted block.
 *
 * @param base The first cell of the block.
 * @return The head, in front of the cells.
 */
static zk_head *zk_head_of(unsigned char *base)
{
    return (zk_head *)(void *)(base - ZK_SEQ_HEAD);
}

/**
 * The count of the values that hold a counted block.
 *
 * @param base The first cell of the block.
 * @return The count, in its head.
 */
static size_t *zk_seq_count(unsigned char *base) { return &zk_head_of(base)->rc; }

/**
 * A new counted block of text or a plain list, with one holder.
 *
 * @param size The octets of its cells.
 * @return The first cell; the run stops when no room is left.
 */
unsigned char *zk_seq_new(size_t size)
{
    unsigned char *p = malloc(ZK_SEQ_HEAD + size);
    DO_IF(!p, zk_out_of_mem(size));
    zk_head *head = (zk_head *)(void *)p;
    memset(head, 0, sizeof *head);
    head->rc = 1;
    head->size = size;
    return p + ZK_SEQ_HEAD;
}

/**
 * A counted block moved to a new size; its count and kind remain.
 *
 * @param base The first cell, or NULL for a new block.
 * @param size The new octets of its cells.
 * @return The first cell; the run stops when no room is left.
 */
static unsigned char *zk_seq_grow(unsigned char *base, size_t size)
{
    RET_IF(!base, zk_seq_new(size));
    unsigned char *p = realloc(base - ZK_SEQ_HEAD, ZK_SEQ_HEAD + size);
    DO_IF(!p, zk_out_of_mem(size));
    ((zk_head *)(void *)p)->size = size;
    return p + ZK_SEQ_HEAD;
}

/**
 * Record the kind of each cell of a counted block, so the ring collector
 * can walk what it holds.
 *
 * @param base The first cell of the block.
 * @param cell The kind of each cell.
 */
static void zk_seq_kind(unsigned char *base, const zk_kind *cell)
{
    zk_head_of(base)->cell = cell;
}

void zk_seq_free(unsigned char *base) { DO_IF(base, free(base - ZK_SEQ_HEAD)); }
/**
 * Write a value of a list or struct kind.
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
 * Move count cells of stride octets.
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

_Static_assert(
    sizeof(zk_code) == sizeof(void *),
    "a closure record holds code in a word of an address"
);

/**
 * The code a function value runs: the first word of its record.
 *
 * @param f The value, not null.
 * @return The code.
 */
zk_code zk_fn_code(zk_fn f)
{
    zk_code code;
    memcpy(&code, zk_fn_rec(f), sizeof code);
    return code;
}

/**
 * The record of a function value: its address without the mark of a loan.
 *
 * @param f The value.
 * @return The record.
 */
unsigned char *zk_fn_rec(zk_fn f)
{
    return (unsigned char *)((uintptr_t)f & ~(uintptr_t)1);
}

/**
 * The kind of the record of a closure: the second word of the record.
 *
 * @param f The value, not null.
 * @return The kind, or NULL for a function.
 */
static const zk_kind *zk_fn_kind(zk_fn f)
{
    const zk_kind *k;
    memcpy(&k, zk_fn_rec(f) + sizeof(zk_code), sizeof k);
    return k;
}

/**
 * Whether a function value is a closure: the second word of its record is
 * the kind of the record, where the record of a function holds NULL.
 *
 * @param f The value, not null.
 * @return 1 for a closure, else 0.
 */
int zk_fn_bound(zk_fn f) { return zk_fn_kind(f) != NULL; }

/**
 * Make a closure: move its record from a slot of the caller to the heap,
 * leaving zeros behind, with the code of its thunk in the first word and the
 * kind of the record in the second.
 *
 * @param record The record, in a slot of the caller.
 * @param thunk The code of the thunk.
 * @param k The kind of the record.
 * @return The closure.
 */
zk_fn zk_closure(unsigned char *record, zk_code thunk, const zk_kind *k)
{
    unsigned char *block = zk_seq_new(k->size);
    zk_seq_kind(block, k);
    memcpy(block, record, k->size);
    memset(record, 0, k->size);
    memcpy(block, &thunk, sizeof thunk);
    memcpy(block + sizeof thunk, &k, sizeof k);
    return block;
}

/**
 * Check a function value headed to C in a program with no runtime of
 * closures for C (zaka_cb.c), or of a kind libffi cannot pass: C calls a bare
 * code address, so a closure stops the run.
 *
 * @param f The value.
 * @return The value.
 */
zk_fn zk_fn_c(zk_fn f)
{
    DO_IF(f && zk_fn_bound(f), zk_plain_trap(ZK_TRAP_CLOSURE_ESCAPE));
    return f;
}

/**
 * A duplicate of a function value: a closure counts one more holder of its
 * record, so each duplicate runs the one closure and its captures remain
 * shared. A duplicate of a lent closure holds a count, so it is no longer
 * lent.
 *
 * @param f The value.
 * @return The duplicate.
 */
zk_fn zk_fn_dup(zk_fn f)
{
    RET_IF(!f || !zk_fn_bound(f), f);
    unsigned char *rec = zk_fn_rec(f);
    ++*zk_seq_count(rec);
    return rec;
}

/**
 * A function value lent to a call: a closure gets the mark of a loan, so a
 * drop lets go of nothing, and a duplicate counts one more holder of the
 * record and is no longer lent.
 *
 * @param f The value.
 * @return The lent value.
 */
zk_fn zk_fn_lend(zk_fn f)
{
    RET_IF(!f || !zk_fn_bound(f), f);
    return (zk_fn)((uintptr_t)f | 1);
}

static const zk_kind *zk_repr(const zk_kind *k);
static void zk_note_candidate(unsigned char *base);
static void zk_dispose(unsigned char *base);

/**
 * Let go of the record of the closure a slot holds, and leave NULL: the last
 * holder drops its fields, the boxes of its captures, and frees it. A
 * function, or a lent
 * closure, holds no count of a record.
 *
 * @param at The slot.
 */
static void zk_drop_fn(unsigned char *at)
{
    zk_fn f;
    memcpy(&f, at, sizeof f);
    int owned = f && !((uintptr_t)f & 1) && zk_fn_bound(f);
    memset(at, 0, sizeof f);
    RET_IF(!owned);
    unsigned char *rec = zk_fn_rec(f);
    DONE_IF(--*zk_seq_count(rec) > 0, zk_note_candidate(rec));
    zk_drop(rec, zk_fn_kind(f));
    zk_dispose(rec);
}

/**
 * The size of the record of a closure, from the kind in its second word.
 *
 * @param f The value, a closure.
 * @return The size in octets.
 */
size_t zk_fn_size(zk_fn f) { return zk_fn_kind(f)->size; }

/**
 * Let go of one count of the record of a closure: the last holder drops its
 * fields and frees it.
 *
 * @param f The value.
 */
void zk_fn_release(zk_fn f)
{
    unsigned char slot[sizeof f];
    memcpy(slot, &f, sizeof f);
    zk_drop_fn(slot);
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
 * A duplicate of the octets of text or a text view that ends in a zero octet, as
 * a C function reads a string. A zero octet inside the text ends the string
 * there for C.
 *
 * @param at The address of the text.
 * @return The duplicate on the heap, for zk_text_cstr_free; the program stops
 *         when no heap room is left.
 */
char *zk_text_cstr(const unsigned char *at)
{
    size_t len = zk_seq_len(at);
    char *dup = malloc(len + 1);
    DO_IF(!dup, zk_out_of_mem(len + 1));
    DO_IF(len, memcpy(dup, zk_seq_base(at), len));
    dup[len] = 0;
    return dup;
}

/**
 * Free a duplicate zk_text_cstr made.
 *
 * @param dup The duplicate.
 */
void zk_text_cstr_free(char *dup) { free(dup); }

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
    DO_IF(k->tag == ZK_K_TEXT || k->tag == ZK_K_SEQ || k->tag == ZK_K_FN, f(at, k));
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

/* ---- the queue of blocks to free, and the ring collector ---- */

/** The blocks let go and still to free; the outermost drop frees them in a
 * loop, so a long chain needs no deep call stack. */
static unsigned char *zk_freeing = NULL;

/** Whether a drop is freeing the queue, so an inner drop just queues. */
static int zk_draining = 0;

/**
 * Queue a block that nothing holds now.
 *
 * @param base The first cell of the block.
 * @param cell The kind of each cell to drop first, or NULL.
 * @param len The cells to drop.
 */
static void zk_queue_free(unsigned char *base, const zk_kind *cell, size_t len)
{
    zk_head *h = zk_head_of(base);
    h->drop_cell = cell;
    h->len = len;
    h->next = zk_freeing;
    zk_freeing = base;
}

/**
 * Free a block that nothing holds; one in the buffer of the ring collector
 * remains, with a count of 0, until the collector frees it.
 *
 * @param base The first cell of the block.
 */
static void zk_dispose(unsigned char *base)
{
    zk_head *h = zk_head_of(base);
    DONE_IF(h->buffered, h->rc = 0);
    zk_seq_free(base);
}

/** Free each queued block, dropping its cells first; a cell let go joins the
 * queue. */
static void zk_drain_frees(void)
{
    while (zk_freeing)
    {
        unsigned char *base = zk_freeing;
        zk_head *h = zk_head_of(base);
        zk_freeing = h->next;
        const zk_kind *cell = h->drop_cell;
        DO_IF(cell, DO_FOR(i, h->len) zk_drop(base + i * cell->size, cell));
        zk_dispose(base);
    }
}

/** A list of blocks that grows. */
typedef struct
{
    unsigned char **at;
    size_t len, cap;
} zk_blocks;

/**
 * Add a block to a list.
 *
 * @param b The list.
 * @param x The block.
 */
static void zk_blocks_push(zk_blocks *b, unsigned char *x)
{
    if (b->len == b->cap)
    {
        size_t cap = b->cap ? 2 * b->cap : 64;
        unsigned char **more = realloc(b->at, cap * sizeof *more);
        DO_IF(!more, zk_out_of_mem(cap * sizeof *more));
        b->at = more;
        b->cap = cap;
    }
    b->at[b->len++] = x;
}

/**
 * Take the last block off a list.
 *
 * @param b The list.
 * @return The block, or NULL for a list with no blocks.
 */
static unsigned char *zk_blocks_pop(zk_blocks *b)
{
    return b->len ? b->at[--b->len] : NULL;
}

/** The count of blocks that wait in the buffer before a collection runs. */
#define ZK_BUFFER 1024

/** The blocks a release left held that can be part of a ring. */
static zk_blocks zk_candidates;

/**
 * Whether a value of a kind can hold a block that holds it back: a box, an
 * object or a closure, at once or inside a list or a struct.
 *
 * @param k The kind.
 * @return 1 for such a kind, else 0.
 */
static int zk_can_ring(const zk_kind *k)
{
    k = zk_repr(k);
    RET_IF(k->boxed || k->tag == ZK_K_FN, 1);
    RET_IF(k->tag == ZK_K_LIST || k->tag == ZK_K_SEQ, zk_can_ring(k->cell));
    RET_IF(k->tag != ZK_K_STRUCT && k->tag != ZK_K_FALLIBLE, 0);
    DO_FOR(i, k->len) RET_IF(zk_can_ring(k->fields[i].kind), 1);
    return 0;
}

/** Where zk_note_child writes the blocks it finds. */
static zk_blocks *zk_child_out;

/**
 * Note the block a slot holds a count of: the record of a closure that is
 * not lent, or the block of text or a plain list that owns it.
 *
 * @param at The slot.
 * @param k Its kind.
 */
static void zk_note_child(unsigned char *at, const zk_kind *k)
{
    if (k->tag == ZK_K_FN)
    {
        zk_fn f;
        memcpy(&f, at, sizeof f);
        int held = f && !((uintptr_t)f & 1) && zk_fn_bound(f);
        DO_IF(held, zk_blocks_push(zk_child_out, zk_fn_rec(f)));
        return;
    }
    unsigned char *base = zk_seq_base(at);
    DO_IF(base && zk_seq_room(at) > 0, zk_blocks_push(zk_child_out, base));
}

/**
 * The blocks a counted block holds a count of.
 *
 * @param base The first cell of the block.
 * @param out The list the blocks join.
 */
static void zk_children(unsigned char *base, zk_blocks *out)
{
    const zk_head *h = zk_head_of(base);
    RET_IF(!h->cell);
    size_t stride = h->cell->size ? h->cell->size : 1;
    zk_child_out = out;
    DO_FOR(i, h->size / stride)
        zk_each_block(base + i * stride, h->cell, zk_note_child);
}

/**
 * Take off, for a trial, each count a block reached from a root gives
 * another: paint each such block pending.
 *
 * @param root A block of the buffer.
 */
static void zk_mark_pending(unsigned char *root)
{
    zk_blocks todo = {NULL, 0, 0}, kids = {NULL, 0, 0};
    zk_blocks_push(&todo, root);
    for (unsigned char *b; (b = zk_blocks_pop(&todo));)
    {
        zk_head *h = zk_head_of(b);
        NEXT_IF(h->color == 1);
        h->color = 1;
        kids.len = 0;
        zk_children(b, &kids);
        DO_FOR(i, kids.len)
        {
            zk_head *c = zk_head_of(kids.at[i]);
            DO_IF(c->rc, c->rc--);
            zk_blocks_push(&todo, kids.at[i]);
        }
    }
    free(todo.at);
    free(kids.at);
}

/**
 * Give a block held from outside, and each block it reaches, the counts the
 * trial took off, and paint them black.
 *
 * @param root The held block.
 */
static void zk_scan_black(unsigned char *root)
{
    zk_blocks todo = {NULL, 0, 0}, kids = {NULL, 0, 0};
    zk_head_of(root)->color = 0;
    zk_blocks_push(&todo, root);
    for (unsigned char *b; (b = zk_blocks_pop(&todo));)
    {
        kids.len = 0;
        zk_children(b, &kids);
        DO_FOR(i, kids.len)
        {
            zk_head *c = zk_head_of(kids.at[i]);
            c->rc++;
            NEXT_IF(c->color == 0);
            c->color = 0;
            zk_blocks_push(&todo, kids.at[i]);
        }
    }
    free(todo.at);
    free(kids.at);
}

/**
 * Paint white each pending block that nothing outside holds, and give a held
 * one and what it reaches their counts back.
 *
 * @param root A block of the buffer.
 */
static void zk_scan(unsigned char *root)
{
    zk_blocks todo = {NULL, 0, 0};
    zk_blocks_push(&todo, root);
    for (unsigned char *b; (b = zk_blocks_pop(&todo));)
    {
        zk_head *h = zk_head_of(b);
        NEXT_IF(h->color != 1);
        if (h->rc > 0)
        {
            zk_scan_black(b);
            continue;
        }
        h->color = 2;
        zk_children(b, &todo);
    }
    free(todo.at);
}

/**
 * Gather the white blocks a root reaches, painting them black so each is
 * gathered once.
 *
 * @param root A block of the buffer.
 * @param out The blocks to free.
 */
static void zk_collect_white(unsigned char *root, zk_blocks *out)
{
    zk_blocks todo = {NULL, 0, 0};
    zk_blocks_push(&todo, root);
    for (unsigned char *b; (b = zk_blocks_pop(&todo));)
    {
        zk_head *h = zk_head_of(b);
        NEXT_IF(h->color != 2 || h->buffered);
        h->color = 0;
        zk_blocks_push(out, b);
        zk_children(b, &todo);
    }
    free(todo.at);
}

/** Free each group of blocks in the buffer, or reached from it, that holds
 * itself alone: trial deletion that halts the program while it runs. */
static void zk_collect_rings(void)
{
    zk_blocks roots = zk_candidates, live = {NULL, 0, 0}, white = {NULL, 0, 0};
    zk_candidates = (zk_blocks){NULL, 0, 0};
    DO_FOR(i, roots.len)
    {
        zk_head *h = zk_head_of(roots.at[i]);
        h->buffered = 0;
        /* A block let go while it waited is freed now. */
        if (h->rc == 0)
        {
            zk_seq_free(roots.at[i]);
            continue;
        }
        zk_blocks_push(&live, roots.at[i]);
    }
    DO_FOR(i, live.len) zk_mark_pending(live.at[i]);
    DO_FOR(i, live.len) zk_scan(live.at[i]);
    DO_FOR(i, live.len) zk_collect_white(live.at[i], &white);
    DO_FOR(i, white.len) zk_seq_free(white.at[i]);
    free(roots.at);
    free(live.at);
    free(white.at);
}

/**
 * Note a block that a release left held: it can be part of a ring that
 * nothing outside holds. A full buffer runs a collection.
 *
 * @param base The first cell of the block.
 */
static void zk_note_candidate(unsigned char *base)
{
    zk_head *h = zk_head_of(base);
    RET_IF(h->buffered || !h->cell || !zk_can_ring(h->cell));
    /* The program frees each ring left as it ends, as the VM does. */
    static int at_exit = 0;
    DO_IF(!at_exit, at_exit = atexit(zk_collect_rings) == 0);
    h->buffered = 1;
    zk_blocks_push(&zk_candidates, base);
    DO_IF(zk_candidates.len >= ZK_BUFFER, zk_collect_rings());
}

/**
 * Let go of a text or plain list: the last holder queues its block, and one
 * still held is noted for the ring collector.
 *
 * @param at The text or plain list.
 * @param k Its kind.
 */
static void zk_drop_block(unsigned char *at, const zk_kind *k)
{
    DONE_IF(k->tag == ZK_K_FN, zk_drop_fn(at));
    unsigned char *base = zk_seq_base(at);
    size_t len = zk_seq_len(at);
    int owned = zk_seq_room(at) > 0;
    memset(at, 0, 3 * ZK_WORD);
    RET_IF(!owned);
    DONE_IF(--*zk_seq_count(base) > 0, zk_note_candidate(base));
    zk_queue_free(base, zk_text_cells(k), len);
}

void zk_drop(unsigned char *at, const zk_kind *k)
{
    zk_each_block(at, k, zk_drop_block);
    RET_IF(zk_draining);
    zk_draining = 1;
    zk_drain_frees();
    zk_draining = 0;
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
    unsigned char *block = zk_seq_new(size);
    zk_seq_kind(block, k->cell);
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
    if (k->tag == ZK_K_FN)
    {
        zk_fn f;
        memcpy(&f, at, sizeof f);
        f = zk_fn_dup(f);
        memcpy(at, &f, sizeof f);
        return;
    }
    unsigned char *base = zk_seq_base(at);
    RET_IF(!base);
    /* An owned block is shared and counted; a lent or static one (room 0)
     * holds no count to share, and a pinned one is never shared, so each is
     * copied. */
    DONE_IF(zk_seq_room(at) > 0 && !zk_head_of(base)->pinned, ++*zk_seq_count(base));
    zk_clone_block(at, k);
}

/**
 * Give a text or plain list a block of its own when other values share its
 * block, so a write reaches this value alone.
 *
 * @param at The text or plain list, which owns its block.
 * @param k Its kind.
 */
static void zk_unshare(unsigned char *at, const zk_kind *k)
{
    unsigned char *base = zk_seq_base(at);
    RET_IF(!base || *zk_seq_count(base) <= 1);
    zk_clone_block(at, k);
    /* The other holders keep the old block, so this never frees it. */
    --*zk_seq_count(base);
}

void zk_dup_seqs(unsigned char *at, const zk_kind *k)
{
    zk_each_block(at, k, zk_dup_block);
}

/* ---- the deep duplicate ---- */

/** The new blocks whose cells the deep duplicate still visits. */
static zk_blocks zk_deep_todo;

/** The old objects the deep duplicate gave a new block. While it runs, the
 * next word of each one's head points at its new block: a live block is in no
 * queue to free, so the word is free to use, and it is cleared at the end. */
static zk_blocks zk_deep_seen;

/**
 * Duplicate one text, plain list or function value inside a deep duplicate:
 * an object gets its new block, made once; a list that can reach an object
 * gets a new block whose cells are visited in turn; the rest are shared.
 *
 * @param at The slot.
 * @param k Its kind.
 */
static void zk_deep_block(unsigned char *at, const zk_kind *k)
{
    DONE_IF(k->tag == ZK_K_FN || !zk_can_ring(k), zk_dup_block(at, k));
    unsigned char *base = zk_seq_base(at);
    RET_IF(!base);
    zk_head *h = zk_head_of(base);
    if (h->next)
    {
        ++*zk_seq_count(h->next);
        memcpy(at, &h->next, sizeof h->next);
        return;
    }
    size_t count = zk_seq_len(at), size = count * zk_stride(k);
    DONE_IF(size == 0, zk_seq_put(at, NULL, 0, 0));
    unsigned char *block = zk_seq_new(size);
    zk_seq_kind(block, k->cell);
    memcpy(block, base, size);
    zk_seq_put(at, block, count, size);
    zk_blocks_push(&zk_deep_todo, block);
    RET_IF(!k->boxed);
    h->next = block;
    zk_blocks_push(&zk_deep_seen, base);
}

void zk_deep_seqs(unsigned char *at, const zk_kind *k)
{
    zk_each_block(at, k, zk_deep_block);
    unsigned char *b;
    while ((b = zk_blocks_pop(&zk_deep_todo)))
    {
        const zk_head *h = zk_head_of(b);
        size_t stride = h->cell->size ? h->cell->size : 1;
        DO_FOR(i, h->size / stride)
            zk_each_block(b + i * stride, h->cell, zk_deep_block);
    }
    while ((b = zk_blocks_pop(&zk_deep_seen))) zk_head_of(b)->next = NULL;
}

/**
 * Lend a text or plain list: its room becomes 0, so a drop frees nothing.
 *
 * @param at The text or plain list.
 * @param k Its kind.
 */
static void zk_lend_block(unsigned char *at, const zk_kind *k)
{
    if (k->tag == ZK_K_FN)
    {
        zk_fn f;
        memcpy(&f, at, sizeof f);
        f = zk_fn_lend(f);
        memcpy(at, &f, sizeof f);
        return;
    }
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
    RET_IF(k->tag == ZK_K_FN);
    DONE_IF(zk_seq_room(at) == 0, zk_clone_block(at, k));
    zk_unshare(at, k);
}

unsigned char *zk_own(unsigned char *at, const zk_kind *k)
{
    zk_each_block(at, k, zk_own_block);
    return at;
}

/**
 * The address of the block of an object, which holds one more count of it.
 *
 * @param at The object.
 * @return The block, or NULL for an object that is the zero word.
 */
unsigned char *zk_obj_ptr(const unsigned char *at)
{
    unsigned char *base = zk_seq_base(at);
    DO_IF(base, ++*zk_seq_count(base));
    return base;
}

/**
 * Write the object a pointer to its block names; the object takes the count
 * the pointer held.
 *
 * @param at The slot of the object.
 * @param base The block, or NULL for the zero word.
 */
void zk_ptr_obj(unsigned char *at, unsigned char *base)
{
    size_t size = base ? zk_head_of(base)->size : 0;
    zk_seq_put(at, base, base ? 1 : 0, size);
}

/**
 * Pin the block of text or a plain list: an address of one of its cells is
 * taken, so it is never shared again and a duplicate copies it.
 *
 * @param at The text or plain list, which owns its block.
 * @return at.
 */
unsigned char *zk_pin(unsigned char *at)
{
    unsigned char *base = zk_seq_base(at);
    DO_IF(base, zk_head_of(base)->pinned = 1);
    return at;
}

void zk_zero(unsigned char *at, size_t size) { DO_IF(size, memset(at, 0, size)); }

void zk_to_heap(
    unsigned char *dst, const unsigned char *src, const zk_kind *list, int fixed
)
{
    size_t count = list->len, size = count * list->cell->size;
    DONE_IF(size == 0, zk_seq_put(dst, NULL, 0, 0));
    /* A fixed list is static, as in the VM: no drop frees it, so it holds
     * no count. */
    unsigned char *block = fixed ? zk_block(size) : zk_seq_new(size);
    DO_IF(!fixed, zk_seq_kind(block, list->cell));
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

/**
 * The address of the fields of an object: cell 0 of its box. An object
 * variable that holds the zero word has a box with no cell, and the run stops.
 *
 * @param at The box.
 * @return The address of the fields.
 */
unsigned char *zk_object_cell(const unsigned char *at)
{
    DO_IF(zk_seq_len(at) == 0, zk_plain_trap(ZK_TRAP_NULL_OBJECT));
    return zk_seq_base(at);
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

/**
 * Stop the run when a plain list of u8s holds no UTF-8.
 *
 * @param at The plain list.
 */
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
        block = zk_seq_grow(held == 0 ? NULL : block, room);
        zk_seq_kind(block, k->cell);
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
        block = zk_seq_grow(held == 0 ? NULL : block, room);
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
        ZK_K_TEXT, 3 * sizeof(void *), NULL, 0, NULL, NULL, NULL, 1, 0, 0
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
    size_t count = zk_seq_len(at);
    DO_IF(count == 0, zk_plain_trap(ZK_TRAP_NO_CELL_TO_REMOVE));
    /* The drop writes into the block, so a shared one is copied first. */
    DO_IF(zk_seq_room(at) > 0, zk_unshare(at, k));
    size_t room = zk_seq_room(at);
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

/* ---- walks that keep their steps on the heap ---- */

/** What a step of a walk does. */
enum
{
    /** Compare or write a value. */
    ZK_STEP_VALUE,
    /** Write a piece of text. */
    ZK_STEP_PIECE,
    /** Leave an object the print is inside. */
    ZK_STEP_LEAVE
};

/** One step of a walk over a value, kept on the heap so a long chain of
 * objects takes no C stack. */
typedef struct zk_step
{
    int what;
    const unsigned char *a, *b;
    const zk_kind *k;
    size_t len;
    int nested;
} zk_step;

/** The steps still to run, the last one first. */
typedef struct zk_steps
{
    zk_step *at;
    size_t len, cap;
} zk_steps;

/**
 * Add a step to the steps still to run.
 *
 * @param s The steps.
 * @param step The step.
 */
static void zk_steps_push(zk_steps *s, zk_step step)
{
    if (s->len == s->cap)
    {
        size_t cap = s->cap ? 2 * s->cap : 64;
        zk_step *more = realloc(s->at, cap * sizeof *more);
        DO_IF(!more, zk_out_of_mem(cap * sizeof *more));
        s->at = more;
        s->cap = cap;
    }
    s->at[s->len++] = step;
}

/** A set of pairs of addresses: open addressing, with a mark for a pair
 * taken out. */
typedef struct zk_set
{
    const unsigned char **slots;
    size_t cap, used;
} zk_set;

/** The mark of a slot whose pair was taken out. */
static const unsigned char zk_gone = 0;

/**
 * The slot of a pair: where it is, or the first free slot on its probe.
 *
 * @param set The set; its room is a power of two.
 * @param a The first address.
 * @param b The second address.
 * @return The slot number.
 */
static size_t zk_set_slot(
    const zk_set *set, const unsigned char *a, const unsigned char *b
)
{
    uintptr_t h = ((uintptr_t)a * 2654435761u) ^ ((uintptr_t)b * 40503u);
    size_t at = (size_t)(h >> 4) & (set->cap - 1);
    size_t free_at = set->cap;
    while (set->slots[2 * at])
    {
        RET_IF(set->slots[2 * at] == a && set->slots[2 * at + 1] == b, at);
        DO_IF(set->slots[2 * at] == &zk_gone && free_at == set->cap, free_at = at);
        at = (at + 1) & (set->cap - 1);
    }
    return free_at == set->cap ? at : free_at;
}

/**
 * Add a pair to a set.
 *
 * @param set The set.
 * @param a The first address, not NULL.
 * @param b The second address.
 * @return 1 when the pair is new, 0 when the set holds it.
 */
static int zk_set_add(zk_set *set, const unsigned char *a, const unsigned char *b)
{
    if (2 * (set->used + 1) > set->cap)
    {
        zk_set grown = {NULL, set->cap ? 2 * set->cap : 64, 0};
        grown.slots = calloc(2 * grown.cap, sizeof *grown.slots);
        DO_IF(!grown.slots, zk_out_of_mem(2 * grown.cap * sizeof *grown.slots));
        DO_FOR(i, set->cap)
        {
            const unsigned char *x = set->slots[2 * i];
            NEXT_IF(!x || x == &zk_gone);
            size_t at = zk_set_slot(&grown, x, set->slots[2 * i + 1]);
            grown.slots[2 * at] = x;
            grown.slots[2 * at + 1] = set->slots[2 * i + 1];
            grown.used++;
        }
        free(set->slots);
        *set = grown;
    }
    size_t at = zk_set_slot(set, a, b);
    RET_IF(set->slots[2 * at] == a && set->slots[2 * at + 1] == b, 0);
    DO_IF(!set->slots[2 * at], set->used++);
    set->slots[2 * at] = a;
    set->slots[2 * at + 1] = b;
    return 1;
}

/**
 * Take a pair out of a set.
 *
 * @param set The set.
 * @param a The first address.
 * @param b The second address.
 */
static void zk_set_drop(zk_set *set, const unsigned char *a, const unsigned char *b)
{
    RET_IF(!set->cap);
    size_t at = zk_set_slot(set, a, b);
    RET_IF(set->slots[2 * at] != a || set->slots[2 * at + 1] != b);
    set->slots[2 * at] = &zk_gone;
    set->slots[2 * at + 1] = NULL;
}

/**
 * Add a step that compares or writes a value.
 *
 * @param s The steps.
 * @param a The value, or the first of two.
 * @param b The second value, or NULL.
 * @param k The kind.
 * @param nested 1 when text is written in quotes.
 */
static void zk_push_value(
    zk_steps *s, const unsigned char *a, const unsigned char *b, const zk_kind *k,
    int nested
)
{
    zk_steps_push(s, (zk_step){ZK_STEP_VALUE, a, b, k, 0, nested});
}

/**
 * Add the steps that compare two runs of cells, each to the cell at the
 * same place.
 *
 * @param s The steps.
 * @param a The first cell of the first run.
 * @param b The first cell of the second run.
 * @param cell The kind of a cell.
 * @param count The cell count of each.
 */
static void zk_push_cell_pairs(
    zk_steps *s, const unsigned char *a, const unsigned char *b, const zk_kind *cell,
    size_t count
)
{
    DO_FOR(i, count) zk_push_value(s, a + i * cell->size, b + i * cell->size, cell, 0);
}

/**
 * Compare one pair of values: scalars and text at once, the parts of a list,
 * a plain list, an object or a struct as steps. Two objects compared before
 * count as equal, so a ring ends.
 *
 * @param a The first value.
 * @param b The second value.
 * @param k The kind.
 * @param s The steps, added to.
 * @param seen The pairs of objects met, added to.
 * @return 0 when the pair differs, else 1.
 */
static int zk_eq_step(
    const unsigned char *a, const unsigned char *b, const zk_kind *k, zk_steps *s,
    zk_set *seen
)
{
    k = zk_repr(k);
    switch (k->tag)
    {
    case ZK_K_TEXT:
        return zk_text_eq(a, b);
    case ZK_K_SEQ:
        if (k->boxed)
        {
            const unsigned char *la = zk_seq_base(a), *lb = zk_seq_base(b);
            RET_IF(la == lb, 1);
            RET_IF(!la || !lb, 0);
            RET_IF(!zk_set_add(seen, la, lb), 1);
            zk_push_value(s, la, lb, k->cell, 0);
            return 1;
        }
        RET_IF(zk_seq_len(a) != zk_seq_len(b), 0);
        zk_push_cell_pairs(s, zk_seq_base(a), zk_seq_base(b), k->cell, zk_seq_len(a));
        return 1;
    case ZK_K_LIST:
        zk_push_cell_pairs(s, a, b, k->cell, k->len);
        return 1;
    case ZK_K_STRUCT:
    case ZK_K_FALLIBLE:
        DO_FOR(i, k->len)
        {
            size_t at = k->fields[i].offset;
            zk_push_value(s, a + at, b + at, k->fields[i].kind, 0);
        }
        return 1;
    case ZK_K_FLOAT:
        RET_IF(k->size == 4, zk_ld_f32(a) == zk_ld_f32(b));
        return zk_ld_f64(a) == zk_ld_f64(b);
    default:
        return memcmp(a, b, k->size) == 0;
    }
}

int zk_eq(const unsigned char *a, const unsigned char *b, const zk_kind *k)
{
    zk_steps steps = {NULL, 0, 0};
    zk_set seen = {NULL, 0, 0};
    zk_push_value(&steps, a, b, k, 0);
    int same = 1;
    while (same && steps.len)
    {
        zk_step step = steps.at[--steps.len];
        same = zk_eq_step(step.a, step.b, step.k, &steps, &seen);
    }
    free(steps.at);
    free(seen.slots);
    return same;
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
    zk_seq_free(zk_seq_base(held));
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
        NEXT_IF(zk_fn_names[i].code != zk_fn_code(f));
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

/**
 * Add a step that writes a piece of text.
 *
 * @param s The steps.
 * @param text The text.
 * @param len Its length in octets.
 */
static void zk_push_piece(zk_steps *s, const char *text, size_t len)
{
    zk_steps_push(
        s, (zk_step){ZK_STEP_PIECE, (const unsigned char *)text, NULL, NULL, len, 0}
    );
}

/**
 * Add the steps that write a run of cells between `[` and `]`, the last
 * first.
 *
 * @param s The steps.
 * @param base The first cell.
 * @param cell The kind of a cell.
 * @param count The cell count.
 */
static void zk_push_cells(
    zk_steps *s, const unsigned char *base, const zk_kind *cell, size_t count
)
{
    zk_push_piece(s, "]", 1);
    for (size_t i = count; i-- > 0;)
    {
        zk_push_value(s, base + i * cell->size, NULL, cell, 1);
        DO_IF(i, zk_push_piece(s, zk_sep, sizeof zk_sep - 1));
    }
    zk_push_piece(s, "[", 1);
}

/**
 * Add the steps that write a struct, the last first: its name, then each
 * field as `name: value`.
 *
 * @param s The steps.
 * @param at The struct.
 * @param k Its kind.
 */
static void zk_push_struct(zk_steps *s, const unsigned char *at, const zk_kind *k)
{
    zk_push_piece(s, ")", 1);
    for (size_t i = k->len; i-- > 0;)
    {
        zk_push_value(s, at + k->fields[i].offset, NULL, k->fields[i].kind, 1);
        zk_push_piece(s, ": ", 2);
        zk_push_piece(s, k->fields[i].name, strlen(k->fields[i].name));
        DO_IF(i, zk_push_piece(s, zk_sep, sizeof zk_sep - 1));
    }
    zk_push_piece(s, "(", 1);
    zk_push_piece(s, k->name, strlen(k->name));
}

/**
 * Add the steps that write a result that can fail, the last first: its good
 * value or its error value, or the word of a side with no value. An optional
 * writes its data, or the word of an optional with no data.
 *
 * @param s The steps.
 * @param at The result.
 * @param k Its kind.
 */
static void zk_push_fallible(zk_steps *s, const unsigned char *at, const zk_kind *k)
{
    const zk_field *tag = zk_field_named(k, "tag");
    int failed = tag && at[tag->offset] == 0;
    const zk_field *some = zk_field_named(k, "some");
    DONE_IF(some && failed, zk_push_piece(s, zk_no_data_text, strlen(zk_no_data_text)));
    const zk_field *part = some ? some : zk_field_named(k, failed ? "err" : "ok");
    const char *bare = failed ? zk_fault_bare : zk_good_bare;
    DONE_IF(!part, zk_push_piece(s, bare, strlen(bare)));
    const char *word = some ? zk_data_text : failed ? zk_fault_text : zk_good_text;
    zk_push_piece(s, ")", 1);
    zk_push_value(s, at + part->offset, NULL, part->kind, 1);
    zk_push_piece(s, "(", 1);
    zk_push_piece(s, word, strlen(word));
}

/**
 * Add the steps that write an object: its struct, or the null text for an
 * object with no value. An object the print is still inside is written as
 * an ellipsis.
 *
 * @param s The steps.
 * @param inside The objects the print is inside, added to.
 * @param at The object: its box.
 * @param k The object kind.
 */
static void zk_push_object(
    zk_steps *s, zk_set *inside, const unsigned char *at, const zk_kind *k
)
{
    const unsigned char *base = zk_seq_base(at);
    DONE_IF(!base, zk_push_piece(s, zk_null_text, strlen(zk_null_text)));
    DONE_IF(!zk_set_add(inside, base, NULL), zk_push_piece(s, "\xe2\x80\xa6", 3));
    zk_steps_push(s, (zk_step){ZK_STEP_LEAVE, base, NULL, NULL, 0, 0});
    zk_push_value(s, base, NULL, k->cell, 0);
}

/**
 * Run one step of a print: write a piece, leave an object, or write a value,
 * at once or as further steps.
 *
 * @param step The step.
 * @param s The steps, added to.
 * @param inside The objects the print is inside.
 */
static void zk_show_step(const zk_step *step, zk_steps *s, zk_set *inside)
{
    DONE_IF(step->what == ZK_STEP_PIECE, zk_emit(step->a, step->len));
    DONE_IF(step->what == ZK_STEP_LEAVE, zk_set_drop(inside, step->a, NULL));
    const unsigned char *at = step->a;
    const zk_kind *k = step->k;
    switch (k->tag)
    {
    case ZK_K_ENUM:
        zk_show_member(at, k);
        return;
    case ZK_K_STRUCT:
        zk_push_struct(s, at, k);
        return;
    case ZK_K_FALLIBLE:
        zk_push_fallible(s, at, k);
        return;
    case ZK_K_TEXT:
        zk_show_text(at, step->nested);
        return;
    case ZK_K_SEQ:
        DONE_IF(k->boxed, zk_push_object(s, inside, at, k));
        zk_push_cells(s, zk_seq_base(at), k->cell, zk_seq_len(at));
        return;
    case ZK_K_LIST:
        zk_push_cells(s, at, k->cell, k->len);
        return;
    default:
        zk_show_scalar(at, k);
        return;
    }
}

void zk_show(const unsigned char *at, const zk_kind *k, int nested)
{
    zk_steps steps = {NULL, 0, 0};
    zk_set inside = {NULL, 0, 0};
    zk_push_value(&steps, at, NULL, k, nested);
    while (steps.len)
    {
        zk_step step = steps.at[--steps.len];
        zk_show_step(&step, &steps, &inside);
    }
    free(steps.at);
    free(inside.slots);
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

/**
 * Stop the run: an optional that holds no data was read.
 */
_Noreturn void zk_no_data(void) { zk_plain_trap(ZK_TRAP_NO_DATA); }

/**
 * Stop the run: a handle a C function closed went to C again.
 */
_Noreturn void zk_closed_handle(void) { zk_plain_trap(ZK_TRAP_CLOSED_HANDLE); }
