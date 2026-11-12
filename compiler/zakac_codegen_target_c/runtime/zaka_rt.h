/* The runtime of a Zaka program that the Zaka Compiler C target builds. */

#ifndef ZAKA_RT_H
#define ZAKA_RT_H

/* Round after each float operation, as the VM does: no fused mul-add. */
#pragma STDC FP_CONTRACT OFF

/* The check level and the call depth limit of the build, which the compiler
 * writes beside the runtime. */
#include <math.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include "zaka_checks.h"

/* The widest integer kinds: each Zaka integer kind converts to one of them
 * with no loss. */
typedef int64_t zk_imax;
typedef uint64_t zk_umax;

/* zaka_checks.h sets each ZK_CHECK_<FAMILY> to 1 or 0: the run-time stops the
 * program keeps, each check on its own. */

_Static_assert(sizeof(size_t) == sizeof(void *), "a word holds an address and a count");

/**
 * The traps a program can stop with; zaka_kinds.c holds the text of each,
 * from the locale of the compiler.
 */
enum zk_trap
{
    ZK_TRAP_ZERO_DIVISOR,
    ZK_TRAP_SHIFT_TOO_FAR,
    ZK_TRAP_STEP_NOT_POSITIVE,
    ZK_TRAP_OVERFLOW,
    ZK_TRAP_INDEX_OUT_OF_RANGE,
    ZK_TRAP_CHAR_OUT_OF_RANGE,
    ZK_TRAP_END_BEFORE_START,
    ZK_TRAP_NULL_CALL,
    ZK_TRAP_NULL_OBJECT,
    ZK_TRAP_CLOSED_HANDLE,
    ZK_TRAP_NULL_FROM_C,
    ZK_TRAP_CLOSURE_ESCAPE,
    ZK_TRAP_NO_CELL_TO_REMOVE,
    ZK_TRAP_NO_CHAR_TO_REMOVE,
    ZK_TRAP_NO_PLACE_TO_INSERT,
    ZK_TRAP_SLICE_OUT_OF_RANGE,
    ZK_TRAP_TEXT_CUT,
    ZK_TRAP_BAD_UTF8,
    ZK_TRAP_NOT_GOOD,
    ZK_TRAP_NOT_GOOD_BARE,
    ZK_TRAP_NOT_FAULT,
    ZK_TRAP_NO_DATA,
    ZK_TRAP_NO_ROW,
    ZK_TRAP_NULL_DEREF,
    ZK_TRAP_OUT_OF_MEM,
    ZK_TRAP_OVERLAP,
    ZK_TRAP_TOO_DEEP,
    ZK_TRAP_COUNT
};

/** The template of each trap, with %{name} slots. */
extern const char *const zk_trap_text[ZK_TRAP_COUNT];
/** The line around a trap text, with an %{e} slot. */
extern const char *const zk_stopped_text;
/** The text a print writes for true, false and the null address. */
extern const char zk_true_text[], zk_false_text[], zk_null_text[];

/* The functions of the runtime; zaka_rt.c documents each in full. */

/** Get the run going: raw output, a large output buffer. */
void zk_start(int argc, char **argv);
/** Write out what is buffered and give the exit status. */
int zk_finish(int status);

/** Stop the run: a zero divisor. */
_Noreturn void zk_zero_divisor(void);
/** Stop the run: a shift as far as the width of its value, or further. */
_Noreturn void zk_shift_too_far(zk_umax distance, unsigned bits);
/** Stop the run: a counted loop step that is not positive. */
_Noreturn void zk_step_not_positive(void);
/** Stop the run: a result that does not fit its kind. */
_Noreturn void zk_overflow(const char *op, const char *kind);
/** Stop the run: more nested calls than the depth limit. */
_Noreturn void zk_too_deep(void);

/** The count of Zaka calls in progress. */
extern uint32_t zk_depth;

/**
 * A place in the source: the index of its file in zk_site_files, its line
 * and its column, each counted from 1.
 */
typedef struct zk_site
{
    uint32_t file;
    uint32_t line;
    uint32_t col;
} zk_site;

/**
 * One call in progress: the number of its function, 0 for main and n + 1 for
 * index n of zk_fn_names, and the site of the statement it runs, 0 for none.
 */
typedef struct zk_frame
{
    uint32_t func;
    uint32_t site;
} zk_frame;

/** The calls in progress, main at 0 and the last at zk_depth. */
extern zk_frame zk_frames[ZK_MAX_DEPTH + 1];

#if ZK_DEBUG_INFO
/** The site of each statement a stop can name; index 0 names none. */
extern const zk_site zk_sites[];
/** The name of each source file the sites name. */
extern const char *const zk_site_files[];
#endif

/** The texts of the place of a stop and of its backtrace, from the compiler. */
extern const char zk_where_text[], zk_in_text[], zk_backtrace_text[];
extern const char zk_call_at_text[], zk_call_text[], zk_call_again_text[];
extern const char zk_calls_more_text[], zk_full_hint_text[];

/** Whether the calls in progress are kept: for the depth check, or for the
 * place and the backtrace of a stop in a build with debug info. */
#define ZK_TRACK (ZK_CHECK_DEPTH || ZK_DEBUG_INFO)

/**
 * Note a call as it starts: count it, stop the run past the depth limit, and
 * keep its function; a call past the limit with no depth check is counted
 * but not kept.
 *
 * @param func The number of its function, as zk_frame counts them.
 */
static inline void zk_enter(uint32_t func)
{
    zk_depth++;
    if (ZK_CHECK_DEPTH && zk_depth > ZK_MAX_DEPTH) zk_too_deep();
    if (zk_depth > ZK_MAX_DEPTH) return;
    zk_frames[zk_depth].func = func;
    zk_frames[zk_depth].site = 0;
}

/**
 * Note a call as it starts, when the calls are kept.
 *
 * @param F The number of its function, as zk_frame counts them.
 */
/* clang-format off */
#define ZK_ENTER(F) do { if (ZK_TRACK) zk_enter(F); } while (0)
/* clang-format on */

/** Note a call as it returns. */
/* clang-format off */
#define ZK_LEAVE() do { if (ZK_TRACK) zk_depth--; } while (0)
/* clang-format on */

/**
 * Note the site of the statement that runs, in a build with debug info.
 *
 * @param S The index of the site in zk_sites.
 */
/* clang-format off */
#define ZK_AT(S) do { if (ZK_DEBUG_INFO && zk_depth <= ZK_MAX_DEPTH) zk_frames[zk_depth].site = (S); } while (0)
/* clang-format on */

/** Write octets unchanged. */
void zk_put_raw(const char *s, size_t n);
/** Write the line break that ends each print. */
void zk_put_line(void);
/** Write a signed integer, in Burmese digits. */
void zk_put_i(zk_imax v);
/** Write an unsigned integer, in Burmese digits. */
void zk_put_u(zk_umax v);
/** Write an f32 as Rust writes it, in Burmese digits. */
void zk_put_f32(float v);
/** Write an f64 as Rust writes it, in Burmese digits. */
void zk_put_f64(double v);
/** Write a boolean as the word for true or false. */
void zk_put_bool(uint8_t v);
/** Write a character as UTF-8. */
void zk_put_char(uint32_t c);

/** The character with this scalar value, or U+FFFD when it is none. */
uint32_t zk_char_of(int negative, zk_umax v);

/* ---- kinds ---- */

/** The code of a function of the program, cast to one kind. */
typedef void (*zk_code)(void);

/**
 * A function value: the address of a record whose first word is code. The
 * record of a function, zk_fnrec, holds NULL in its second word; a closure
 * record, on the heap, holds the code of its thunk, the kind of the record
 * in its second word, then its captures, and the thunk takes the record
 * last. A closure lent to a call has the lowest bit of its address set.
 */
typedef const void *zk_fn;

/** The record of a function of the program. */
typedef struct
{
    zk_code code;
    const void *none;
} zk_fnrec;

/** The shape of a data kind, as the runtime walks it. */
enum zk_tag
{
    ZK_K_SINT,
    ZK_K_UINT,
    ZK_K_FLOAT,
    ZK_K_CHAR,
    ZK_K_BOOL,
    ZK_K_PTR,
    ZK_K_FN,
    ZK_K_LIST,
    ZK_K_STRUCT,
    ZK_K_FALLIBLE,
    ZK_K_SEQ,
    ZK_K_TEXT,
    ZK_K_ENUM
};

typedef struct zk_kind zk_kind;

/** A field of a struct: its name, its offset in octets and its kind. */
typedef struct
{
    const char *name;
    size_t offset;
    const zk_kind *kind;
} zk_field;

/** A member of an enumeration: its value as the value kind prints it, and
 * the text a print writes for it. */
typedef struct
{
    const char *value;
    const char *shown;
} zk_member;

/** A data kind: zaka_kinds.c holds one for each kind the program walks. */
struct zk_kind
{
    /** The shape, an enum zk_tag. */
    int tag;
    /** The size in octets. */
    size_t size;
    /** The cell of a list, a slice or a plain list; the value kind of an
     * enumeration. */
    const zk_kind *cell;
    /** The cell count of a list, the field count of a struct, the member
     * count of an enumeration. */
    size_t len;
    /** The fields of a struct. */
    const zk_field *fields;
    /** The members of an enumeration. */
    const zk_member *members;
    /** The name of a struct. */
    const char *name;
    /** Whether a value holds text, a plain list or a closure on the heap. */
    int holds_text;
    /** Whether a value holds a borrow. */
    int holds_borrow;
    /** Whether the kind is an owner pointer, or holds one. */
    int owns;
    /** Whether a plain list is a box: an object, or a variable a closure
     * shares. A box is shared among its duplicates. */
    int boxed;
};

/** The words of a print that zaka_kinds.c holds, from the compiler. */
extern const char zk_good_text[], zk_fault_text[], zk_good_bare[], zk_fault_bare[];
extern const char zk_data_text[], zk_no_data_text[];
extern const char zk_fn_lead[], zk_fn_tail[];

/** A function of the program and its name, for the print of a value. */
typedef struct
{
    zk_code code;
    const char *name;
} zk_fn_name;

/** Each function of the program; the count is in zk_fn_count. */
extern const zk_fn_name zk_fn_names[];
extern const size_t zk_fn_count;

/* ---- text out ---- */

/** A growable run of octets: the text an interpolation builds. */
typedef struct
{
    unsigned char *data;
    size_t len;
    size_t cap;
    /** The sink before this one, which comes back when it closes. */
    void *prev;
} zk_buf;

/** Send the text the zk_put functions write into a buffer, not stdout. */
void zk_sink_open(zk_buf *buf);
/** Send the text back to stdout and make the buffer text at a slot. */
void zk_sink_close(unsigned char *slot);
/** Write octets to the sink. */
void zk_emit(const void *s, size_t n);

/* ---- storage ---- */

/** Stop the run with a trap that names a number and the range it is out
 * of. */
_Noreturn void zk_range_trap(int trap, zk_imax index, zk_imax lo, zk_imax hi);
/** Stop the run with a trap that names nothing. */
_Noreturn void zk_plain_trap(int trap);
/** Stop the run with a trap that names numbers. */
_Noreturn void zk_nums_trap(
    int trap, int n, const char *const *names, const zk_imax *nums
);
/** Stop the run with a trap that names an address. */
_Noreturn void zk_addr_trap(int trap, const void *addr);
/** Stop the run with a trap that names a text. */
_Noreturn void zk_text_trap(int trap, const char *e);
/** Stop the run: an access through a null address (the null check). */
_Noreturn void zk_null_deref(const void *addr);
/** Stop the run: no heap room is left for a block. */
_Noreturn void zk_out_of_mem(zk_umax size);

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

/** Write a value of a list or struct kind to an address; the two can
 * overlap. */
void zk_put_block(unsigned char *to, const unsigned char *from, size_t size);
/** An address moved count steps of stride octets; wraps as the VM does. */
unsigned char *zk_ptr_add(unsigned char *p, zk_imax count, size_t stride);
/** The steps of stride octets from q to p. */
zk_imax zk_ptr_sub(const unsigned char *p, const unsigned char *q, size_t stride);
/** A new block of count cells of stride octets, zeroed; NULL when the size
 * overflows or no room is left. */
unsigned char *zk_alloc(zk_umax count, size_t stride);
/** Free a raw block that unsafe code made and give back its octets. */
void zk_free(void *p);
/** A block moved to a new size; the run stops when no room is left. */
unsigned char *zk_realloc(unsigned char *p, zk_umax count, size_t stride);
/** Move count cells of stride octets; with overlap 0, the overlap check stops
 * the run when the two ranges overlap. */
void zk_blit(
    const unsigned char *src, zk_umax count, unsigned char *dst, size_t stride,
    int overlap
);
/** Write one cell into each of count cells. */
void zk_fill(unsigned char *dst, zk_umax count, const unsigned char *cell, size_t size);
/** The function of a function value; the run stops when it is null. */
zk_fn zk_fn_check(zk_fn f);
/** The code a function value runs. */
zk_code zk_fn_code(zk_fn f);
/** Whether a function value is a closure. */
int zk_fn_bound(zk_fn f);
/** The record of a function value, without the mark of a loan. */
unsigned char *zk_fn_rec(zk_fn f);
/** Make a closure: move its record to the heap, with its code and its kind. */
zk_fn zk_closure(unsigned char *record, zk_code thunk, const zk_kind *k);
/** Stop the run when a function value is a closure, which C cannot call. */
zk_fn zk_fn_c(zk_fn f);
/** The size of the record of a closure. */
size_t zk_fn_size(zk_fn f);
/** Let go of one count of the record of a closure. */
void zk_fn_release(zk_fn f);

/**
 * The bridge of a function kind C calls back, with its libffi kinds, which
 * zaka_cb.c defines for each such kind of the program.
 */
typedef struct zk_cb_shape
{
    /** Read the arguments libffi gives, run the thunk with the record, and
     * write the result where libffi wants it. */
    void (*bridge)(void *ret, void **args, unsigned char *record);
    /** The libffi kind of the result, an ffi_type. */
    const void *ret;
    /** The libffi kind of each parameter, each an ffi_type. */
    void *const *args;
    /** The count of parameters. */
    unsigned nargs;
    /** Whether C can call the code of a function of the kind with no
     * bridge: each parameter crosses as it is. */
    int as_is;
} zk_cb_shape;

/** The address C calls for a function value given to a C call. */
zk_code zk_cb_code(zk_fn f, const zk_cb_shape *shape);
/** The count of lends before a C call. */
size_t zk_cb_lent(void);
/** End the lends of the closures a C call got. */
void zk_cb_end(size_t from);
/** Write a new handle around an address C passed to a function of the
 * program. */
void zk_cb_handle(unsigned char *out, void *const *addr, const zk_kind *list);
/** Write an optional of a new handle around an address C passed to a
 * function of the program, with no data for NULL. */
void zk_cb_opt_handle(
    unsigned char *out, size_t size, size_t ok_at, void *const *addr,
    const zk_kind *list
);
/** Give C leave to call a closure after the call that gave it. */
void zk_cb_leave(zk_fn f, const zk_cb_shape *shape);
/** End the leave zk_cb_leave gave a closure. */
void zk_cb_revoke(zk_fn f);
/** A duplicate of a function value: a closure gets a record of its own. */
zk_fn zk_fn_dup(zk_fn f);
/** A function value lent to a call: a closure gets the mark of a loan. */
zk_fn zk_fn_lend(zk_fn f);

/* ---- lists and text ---- */

/** The first word of a slice, a plain list or text: the address of its
 * first cell. */
unsigned char *zk_seq_base(const unsigned char *at);
/** The second word: the cell count, or the octet count of text. */
size_t zk_seq_len(const unsigned char *at);
/** A duplicate of text that ends in a zero octet, as C reads a string. */
char *zk_text_cstr(const unsigned char *at);
/** Free a duplicate zk_text_cstr made. */
void zk_text_cstr_free(char *dup);
/** The text result of a C function, an optional with no data for NULL. */
void zk_text_from_c(unsigned char *out, size_t size, size_t ok_at, const char *s);
/** Text C passed to a function of the program; NULL stops the run. */
void zk_text_of_c(unsigned char *out, const char *s);
/** Write the three words of a plain list or text. */
void zk_seq_put(unsigned char *at, unsigned char *base, size_t len, size_t room);
/** A new counted block of text or a plain list, with one holder. */
unsigned char *zk_seq_new(size_t size);
/** Free a counted block of text or a plain list; NULL does nothing. */
void zk_seq_free(unsigned char *base);
/** Write static text into a slot: its octets and a room of 0. */
void zk_text_lit(unsigned char *slot, const char *s, size_t len, size_t size);

/** Free each text and plain list a value owns on the heap, and blank them. */
void zk_drop(unsigned char *at, const zk_kind *k);
/** Give each text and plain list of a value a heap block of its own. */
void zk_dup_seqs(unsigned char *at, const zk_kind *k);
/** Give a value a duplicate of its own of each object it reaches, once each. */
void zk_deep_seqs(unsigned char *at, const zk_kind *k);
/** Lend each text and plain list of a value: its room becomes 0. */
void zk_lend_seqs(unsigned char *at, const zk_kind *k);
/** Give each text and plain list of a value with a room of 0 a block. */
unsigned char *zk_own(unsigned char *at, const zk_kind *k);
/** The block of an object, holding one more count of it. */
unsigned char *zk_obj_ptr(const unsigned char *at);
/** Write the object a pointer to its block names; it takes the count. */
void zk_ptr_obj(unsigned char *at, unsigned char *base);
/** Pin the block of text or a plain list, so a duplicate copies it. */
unsigned char *zk_pin(unsigned char *at);
/** Zero a value an owner moved out of. */
void zk_zero(unsigned char *at, size_t size);
/** Write the cells of a list to the heap as a plain list; a fixed one, of
 * known cells, gets a room of 0, as the VM's static lists do. */
void zk_to_heap(
    unsigned char *dst, const unsigned char *src, const zk_kind *list, int fixed
);

/** The cell a number names, counted from first; the run stops when it is
 * outside the count. */
size_t zk_cell_of(zk_imax index, size_t len, int first);
/** The address of the fields of an object; the run stops when it is the zero word. */
unsigned char *zk_object_cell(const unsigned char *at);
/** The address of a cell of a slice or a plain list. */
unsigned char *zk_slice_cell(
    const unsigned char *at, zk_imax index, int first, size_t stride
);
/** Write a slice of len cells from cell start of base into a slot; limit is
 * the cell count of the source, SIZE_MAX for none, and text keeps whole
 * characters. */
/** Stop the run when a plain list of u8s holds no UTF-8. */
void zk_check_utf8(const unsigned char *at);
void zk_make_slice(
    unsigned char *slot, unsigned char *base, zk_umax start, zk_umax len, size_t limit,
    size_t stride, int text
);

/** Add a cell at the end of a plain list; the cell moves out of src. */
void zk_add_cell(unsigned char *at, const zk_kind *k, unsigned char *src);
/** Put a cell in front of cell index of a plain list. */
void zk_insert_cell(
    unsigned char *at, const zk_kind *k, zk_imax index, int first, unsigned char *src
);
/** Remove the last cell of a plain list, or the last character of text. */
void zk_remove_cell(unsigned char *at, const zk_kind *k);
/** Add text at the end of text. */
void zk_add_text(unsigned char *at, const unsigned char *text);
/** Add a character at the end of text. */
void zk_add_char(unsigned char *at, uint32_t c);
/** Put text in front of character index of text. */
void zk_insert_text(
    unsigned char *at, zk_imax index, int first, const unsigned char *text
);
/** Put a character in front of character index of text. */
void zk_insert_char(unsigned char *at, zk_imax index, int first, uint32_t c);
/** Put a character in place of character index of text. */
void zk_set_char(unsigned char *at, zk_imax index, int first, uint32_t c);
/** Put text in place of a range of text: characters from 1, or octets. */
void zk_replace_text(
    unsigned char *at, zk_imax start, zk_imax end, int chars, const unsigned char *text
);
/** Put a character in place of a range of text. */
void zk_replace_char(
    unsigned char *at, zk_imax start, zk_imax end, int chars, uint32_t c
);
/** The character count of text. */
size_t zk_char_count(const unsigned char *at);
/** The character a number names, counted from first. */
uint32_t zk_char_at(const unsigned char *at, zk_imax index, int first);
/** The character that starts at an octet. */
uint32_t zk_char_at_octet(const unsigned char *at, zk_umax index);

/** Whether two texts hold the same octets. */
int zk_text_eq(const unsigned char *a, const unsigned char *b);
/** Whether two values of a kind are equal, cell after cell. */
int zk_eq(const unsigned char *a, const unsigned char *b, const zk_kind *k);
/** The field of a struct with a name, or NULL. */
const zk_field *zk_field_named(const zk_kind *k, const char *name);
/** Whether octets are UTF-8 text. */
int zk_is_utf8(const unsigned char *s, size_t n);
/** Write a value as a print writes it; nested text is quoted. */
void zk_show(const unsigned char *at, const zk_kind *k, int nested);
/** Write a function value. */
void zk_put_fn(zk_fn f);
/** Write an address, or the word for null. */
void zk_put_ptr(const unsigned char *p);

/* ---- results ---- */

/** Stop the run: the good part of a result that failed was read. */
_Noreturn void zk_not_good(const unsigned char *at, const zk_kind *k);
/** Stop the run: the good part of a result that failed with no value. */
_Noreturn void zk_not_good_bare(void);
/** Stop the run: the error part of a good result was read. */
_Noreturn void zk_not_fault(void);
/** Stop the run: an optional that holds no data was read. */
_Noreturn void zk_no_data(void);

/* ---- tables ---- */

/** The number of the row of a ဇယား that holds an index, or -1. */
zk_imax zk_row_find(
    const unsigned char *at, const zk_kind *k, const unsigned char *index
);
/** Give the row of an index its value, or add a new last row; both move in. */
void zk_set_row(
    unsigned char *at, const zk_kind *k, unsigned char *index, unsigned char *src
);
/** Remove the row of a ဇယား that holds an index, when one does. */
void zk_remove_row(unsigned char *at, const zk_kind *k, const unsigned char *index);
/** Stop the run: a handle a C function closed went to C again. */
_Noreturn void zk_closed_handle(void);

/* ---- host ---- */

/** Read a line of an input stream into a result. */
void zk_host_read_line(
    unsigned char *slot, const zk_kind *ret, const unsigned char *stream,
    const zk_kind *stream_kind
);
/** Read a file as text into a result. */
void zk_host_read_file(
    unsigned char *slot, const zk_kind *ret, const unsigned char *path
);
/** Write text to a file. */
void zk_host_write_file(
    unsigned char *slot, const zk_kind *ret, const unsigned char *text,
    const unsigned char *path
);
/** Write text to an output stream. */
void zk_host_write_stream(
    unsigned char *slot, const zk_kind *ret, const unsigned char *text,
    const unsigned char *stream, const zk_kind *stream_kind
);
/** Fill a list of text with C strings: the arguments or the environment. */
void zk_text_list(unsigned char *at, int count, char **items);

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

#include "zaka_rt_float.h"

#endif
