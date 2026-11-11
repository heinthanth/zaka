/* Zaka runtime - host functions */

#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "zaka_c_sugars.h"
#include "zaka_rt.h"

#ifdef _WIN32
#include <windows.h>
#endif

/** The members of the I/O error, in the order of their numbers. */
enum
{
    ZK_IO_NOT_FOUND,
    ZK_IO_NO_PERMISSION,
    ZK_IO_END_OF_INPUT,
    ZK_IO_BAD_TEXT,
    ZK_IO_OTHER
};

/** The handles of the streams. */
enum
{
    ZK_STDIN,
    ZK_STDOUT,
    ZK_STDERR
};

/**
 * Text in a heap block of its exact size, at a slot; text with no octets
 * owns no block.
 *
 * @param at The slot: three words.
 * @param s The octets.
 * @param len Their count.
 */
static void zk_text_put(unsigned char *at, const void *s, size_t len)
{
    DONE_IF(len == 0, zk_seq_put(at, NULL, 0, 0));
    unsigned char *block = zk_seq_new(len);
    memcpy(block, s, len);
    zk_seq_put(at, block, len, len);
}

/**
 * The result of a C function that gives back text, as Zaka holds an
 * optional of text: NULL is the optional with no data; else the zero-ended
 * octets are copied into text of its own, and C keeps what it gave. The tag
 * sits at offset 0.
 *
 * @param out The result: `size` octets.
 * @param size The size of the result.
 * @param ok_at The offset of the text in it.
 * @param s What C gave back.
 */
void zk_text_from_c(unsigned char *out, size_t size, size_t ok_at, const char *s)
{
    memset(out, 0, size);
    RET_IF(!s);
    zk_text_of_c(out + ok_at, s);
    out[0] = 1;
}

/**
 * Text C passed to a function of the program: the zero-ended octets are
 * copied into text of its own, and C keeps what it passed. NULL stops the
 * run, as do octets that are not UTF-8.
 *
 * @param out Where the text goes.
 * @param s What C passed.
 */
void zk_text_of_c(unsigned char *out, const char *s)
{
    DO_IF(!s, zk_plain_trap(ZK_TRAP_NULL_TEXT));
    size_t len = strlen(s);
    const unsigned char *raw = (const unsigned char *)s;
    DO_IF(!zk_is_utf8(raw, len), zk_addr_trap(ZK_TRAP_BAD_UTF8, s));
    zk_text_put(out, s, len);
}

/**
 * Write a good result.
 *
 * @param slot The result.
 * @param ret Its kind.
 * @param s The octets of its text, or NULL for a result with no value.
 * @param len Their count.
 */
static void zk_good(unsigned char *slot, const zk_kind *ret, const void *s, size_t len)
{
    slot[zk_field_named(ret, "tag")->offset] = 1;
    const zk_field *ok = zk_field_named(ret, "ok");
    DO_IF(ok && s, zk_text_put(slot + ok->offset, s, len));
}

/**
 * Write a failed result: the member of the I/O error, as a word.
 *
 * @param slot The result.
 * @param ret Its kind.
 * @param member The member.
 */
static void zk_fault(unsigned char *slot, const zk_kind *ret, int member)
{
    slot[zk_field_named(ret, "tag")->offset] = 0;
    const zk_field *err = zk_field_named(ret, "err");
    RET_IF(!err);
    intptr_t word = member;
    memcpy(slot + err->offset, &word, sizeof word);
}

/**
 * The member of the I/O error for the errno of a failed call.
 *
 * @param e The errno.
 * @return The member.
 */
static int zk_errno_member(int e)
{
    RET_IF(e == ENOENT, ZK_IO_NOT_FOUND);
    RET_IF(e == EACCES || e == EPERM, ZK_IO_NO_PERMISSION);
    return ZK_IO_OTHER;
}

/**
 * The handle of a stream: the first field of its struct.
 *
 * @param at The stream.
 * @param k Its kind.
 * @return The handle, cut to 32 bits as the VM cuts it.
 */
static uint32_t zk_handle(const unsigned char *at, const zk_kind *k)
{
    uint64_t raw = 0;
    size_t size = k->fields[0].kind->size;
    memcpy(&raw, at, size < sizeof raw ? size : sizeof raw);
    return (uint32_t)raw;
}

/**
 * Open a file whose path is text. Windows takes a UTF-8 path through its
 * wide functions, so a Burmese file name works there too.
 *
 * @param path The path, as text.
 * @param mode The mode, such as "rb".
 * @return The file, or NULL with errno set.
 */
static FILE *zk_open(const unsigned char *path, const char *mode)
{
    char *name = zk_text_cstr(path);
#ifdef _WIN32
    wchar_t wide[4096], wmode[8];
    int ok = MultiByteToWideChar(CP_UTF8, 0, name, -1, wide, 4096) > 0;
    MultiByteToWideChar(CP_UTF8, 0, mode, -1, wmode, 8);
    FILE *f = ok ? _wfopen(wide, wmode) : NULL;
#else
    FILE *f = fopen(name, mode);
#endif
    zk_text_cstr_free(name);
    return f;
}

/**
 * Add an octet at the end of a buffer.
 *
 * @param b The buffer.
 * @param c The octet.
 */
static void zk_append(zk_buf *b, unsigned char c)
{
    if (b->len == b->cap)
    {
        b->cap = b->cap ? 2 * b->cap : 64;
        b->data = realloc(b->data, b->cap);
        DO_IF(!b->data, zk_out_of_mem(b->cap));
    }
    b->data[b->len++] = c;
}

void zk_host_read_line(
    unsigned char *slot, const zk_kind *ret, const unsigned char *stream,
    const zk_kind *stream_kind
)
{
    DONE_IF(
        zk_handle(stream, stream_kind) != ZK_STDIN, zk_fault(slot, ret, ZK_IO_OTHER)
    );
    fflush(stdout);
    zk_buf line = {NULL, 0, 0, NULL};
    int c, read = 0;
    while ((c = getchar()) != EOF)
    {
        read = 1;
        LEAVE_IF(c == '\n');
        zk_append(&line, (unsigned char)c);
    }
    DONE_IF(!read, zk_fault(slot, ret, ZK_IO_END_OF_INPUT));
    size_t len = line.len;
    DO_IF(len && line.data[len - 1] == '\r', len--);
    if (!zk_is_utf8(line.data, len))
    {
        free(line.data);
        zk_fault(slot, ret, ZK_IO_BAD_TEXT);
        return;
    }
    zk_good(slot, ret, line.data ? (const void *)line.data : "", len);
    free(line.data);
}

void zk_host_read_file(
    unsigned char *slot, const zk_kind *ret, const unsigned char *path
)
{
    FILE *f = zk_open(path, "rb");
    DONE_IF(!f, zk_fault(slot, ret, zk_errno_member(errno)));
    zk_buf all = {NULL, 0, 0, NULL};
    unsigned char chunk[4096];
    size_t n;
    while ((n = fread(chunk, 1, sizeof chunk, f)) > 0)
    {
        all.data = realloc(all.data, all.len + n);
        DO_IF(!all.data, zk_out_of_mem(all.len + n));
        memcpy(all.data + all.len, chunk, n);
        all.len += n;
    }
    int failed = ferror(f);
    fclose(f);
    int member = failed                           ? ZK_IO_OTHER
                 : !zk_is_utf8(all.data, all.len) ? ZK_IO_BAD_TEXT
                                                  : -1;
    DO_IF(member >= 0, zk_fault(slot, ret, member));
    DO_IF(
        member < 0, zk_good(slot, ret, all.data ? (const void *)all.data : "", all.len)
    );
    free(all.data);
}

void zk_host_write_file(
    unsigned char *slot, const zk_kind *ret, const unsigned char *text,
    const unsigned char *path
)
{
    FILE *f = zk_open(path, "wb");
    DONE_IF(!f, zk_fault(slot, ret, zk_errno_member(errno)));
    size_t len = zk_seq_len(text);
    int failed = len && fwrite(zk_seq_base(text), 1, len, f) != len;
    failed |= fclose(f) != 0;
    DONE_IF(failed, zk_fault(slot, ret, ZK_IO_OTHER));
    zk_good(slot, ret, NULL, 0);
}

void zk_host_write_stream(
    unsigned char *slot, const zk_kind *ret, const unsigned char *text,
    const unsigned char *stream, const zk_kind *stream_kind
)
{
    uint32_t handle = zk_handle(stream, stream_kind);
    DONE_IF(
        handle != ZK_STDOUT && handle != ZK_STDERR, zk_fault(slot, ret, ZK_IO_OTHER)
    );
    size_t len = zk_seq_len(text);
    DO_IF(handle == ZK_STDOUT && len, zk_emit(zk_seq_base(text), len));
    DO_IF(handle == ZK_STDERR, fflush(stdout));
    DO_IF(handle == ZK_STDERR && len, fwrite(zk_seq_base(text), 1, len, stderr));
    zk_good(slot, ret, NULL, 0);
}

void zk_text_list(unsigned char *at, int count, char **items)
{
    size_t word = sizeof(void *), cell = 3 * word, size = (size_t)count * cell;
    DONE_IF(count <= 0, zk_seq_put(at, NULL, 0, 0));
    unsigned char *block = zk_seq_new(size);
    DO_FOR(i, count) zk_text_put(block + i * cell, items[i], strlen(items[i]));
    zk_seq_put(at, block, (size_t)count, size);
}
