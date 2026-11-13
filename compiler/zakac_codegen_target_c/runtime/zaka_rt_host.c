/* Zaka runtime - host functions */

/* The POSIX file calls (stat, mkdir, opendir) stand behind this macro under
 * strict C11. */
#if !defined(_WIN32) && !defined(_POSIX_C_SOURCE)
#define _POSIX_C_SOURCE 200809L
#endif

#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "zaka_c_sugars.h"
#include "zaka_rt.h"

#ifdef _WIN32
#include <direct.h>
#include <windows.h>
#else
#include <dirent.h>
#include <sys/stat.h>
#include <sys/types.h>
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
    DO_IF(!s, zk_plain_trap(ZK_TRAP_NULL_FROM_C));
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

/**
 * Write a good result that holds a word: a member of an enumeration.
 *
 * @param slot The result.
 * @param ret Its kind.
 * @param word The word.
 */
static void zk_good_word(unsigned char *slot, const zk_kind *ret, intptr_t word)
{
    slot[zk_field_named(ret, "tag")->offset] = 1;
    const zk_field *ok = zk_field_named(ret, "ok");
    DO_IF(ok, memcpy(slot + ok->offset, &word, sizeof word));
}

/**
 * Write a good result that holds a plain list of text.
 *
 * @param slot The result.
 * @param ret Its kind.
 * @param count The number of texts.
 * @param items The texts, each zero-ended.
 */
static void zk_good_list(
    unsigned char *slot, const zk_kind *ret, int count, char **items
)
{
    slot[zk_field_named(ret, "tag")->offset] = 1;
    const zk_field *ok = zk_field_named(ret, "ok");
    DO_IF(ok, zk_text_list(slot + ok->offset, count, items));
}

#ifdef _WIN32
/**
 * The member of the I/O error for the last error of a Windows call.
 *
 * @return The member.
 */
static int zk_win_member(void)
{
    DWORD e = GetLastError();
    RET_IF(e == ERROR_FILE_NOT_FOUND || e == ERROR_PATH_NOT_FOUND, ZK_IO_NOT_FOUND);
    RET_IF(e == ERROR_ACCESS_DENIED, ZK_IO_NO_PERMISSION);
    return ZK_IO_OTHER;
}

/**
 * A UTF-8 path as the wide text of Windows.
 *
 * @param name The path.
 * @param wide Where the wide text goes: 4096 wide characters.
 * @return Whether the path fits.
 */
static int zk_wide(const char *name, wchar_t *wide)
{
    return MultiByteToWideChar(CP_UTF8, 0, name, -1, wide, 4096) > 0;
}
#endif

void zk_host_append_file(
    unsigned char *slot, const zk_kind *ret, const unsigned char *text,
    const unsigned char *path
)
{
    FILE *f = zk_open(path, "ab");
    DONE_IF(!f, zk_fault(slot, ret, zk_errno_member(errno)));
    size_t len = zk_seq_len(text);
    int failed = len && fwrite(zk_seq_base(text), 1, len, f) != len;
    failed |= fclose(f) != 0;
    DONE_IF(failed, zk_fault(slot, ret, ZK_IO_OTHER));
    zk_good(slot, ret, NULL, 0);
}

void zk_host_remove(unsigned char *slot, const zk_kind *ret, const unsigned char *path)
{
    char *name = zk_text_cstr(path);
#ifdef _WIN32
    wchar_t wide[4096];
    int ok = zk_wide(name, wide);
    int done = ok && (_wremove(wide) == 0 || RemoveDirectoryW(wide));
    int member = done ? 0 : zk_win_member();
#else
    int done = remove(name) == 0;
    int member = done ? 0 : zk_errno_member(errno);
#endif
    zk_text_cstr_free(name);
    DONE_IF(!done, zk_fault(slot, ret, member));
    zk_good(slot, ret, NULL, 0);
}

/**
 * Make one folder; a folder there before counts as made.
 *
 * @param name The path of the folder.
 * @return 0, or -1 with errno set.
 */
static int zk_mkdir_one(const char *name)
{
#ifdef _WIN32
    wchar_t wide[4096];
    RET_IF(!zk_wide(name, wide), -1);
    int made = _wmkdir(wide) == 0;
#else
    int made = mkdir(name, 0777) == 0;
#endif
    return made || errno == EEXIST ? 0 : -1;
}

void zk_host_make_folder(
    unsigned char *slot, const zk_kind *ret, const unsigned char *path
)
{
    char *name = zk_text_cstr(path);
    size_t len = strlen(name);
    int member = -1;
    for (size_t i = 1; i <= len && member < 0; i++)
    {
        NEXT_IF(i < len && name[i] != '/' && name[i] != '\\');
        char keep = name[i];
        name[i] = '\0';
        DO_IF(zk_mkdir_one(name) != 0, member = zk_errno_member(errno));
        name[i] = keep;
    }
    zk_text_cstr_free(name);
    DONE_IF(member >= 0, zk_fault(slot, ret, member));
    zk_good(slot, ret, NULL, 0);
}

/**
 * Order two names for qsort: on their u8s, as the VM sorts them.
 *
 * @param a The first name.
 * @param b The second name.
 * @return Less than, equal to or more than 0.
 */
static int zk_name_order(const void *a, const void *b)
{
    return strcmp(*(char *const *)a, *(char *const *)b);
}

/**
 * Add a duplicate of a name to a growing list of names.
 *
 * @param names The list.
 * @param count The number of names in it.
 * @param room The room of the list.
 * @param name The name.
 */
static void zk_add_name(char ***names, int *count, int *room, const char *name)
{
    if (*count == *room)
    {
        *room = *room ? 2 * *room : 16;
        *names = realloc(*names, (size_t)*room * sizeof **names);
        DO_IF(!*names, zk_out_of_mem((size_t)*room * sizeof **names));
    }
    size_t len = strlen(name);
    char *dup = malloc(len + 1);
    DO_IF(!dup, zk_out_of_mem(len + 1));
    memcpy(dup, name, len + 1);
    (*names)[(*count)++] = dup;
}

/**
 * Read the names a folder holds, with no `.` and no `..`.
 *
 * @param name The path of the folder.
 * @param names Where the names go.
 * @param count Where their number goes.
 * @return -1, or the member of the I/O error for a folder that cannot be read.
 */
static int zk_read_folder(const char *name, char ***names, int *count)
{
    int room = 0;
#ifdef _WIN32
    wchar_t wide[4096];
    char pattern[4100];
    snprintf(pattern, sizeof pattern, "%s\\*", name);
    RET_IF(!zk_wide(pattern, wide), ZK_IO_OTHER);
    WIN32_FIND_DATAW found;
    HANDLE h = FindFirstFileW(wide, &found);
    RET_IF(h == INVALID_HANDLE_VALUE, zk_win_member());
    do
    {
        char utf8[1024];
        int ok = WideCharToMultiByte(
                     CP_UTF8, 0, found.cFileName, -1, utf8, 1024, NULL, NULL
                 ) > 0;
        NEXT_IF(!ok || strcmp(utf8, ".") == 0 || strcmp(utf8, "..") == 0);
        zk_add_name(names, count, &room, utf8);
    } while (FindNextFileW(h, &found));
    FindClose(h);
#else
    DIR *dir = opendir(name);
    RET_IF(!dir, zk_errno_member(errno));
    struct dirent *entry;
    while ((entry = readdir(dir)) != NULL)
    {
        const char *n = entry->d_name;
        NEXT_IF(strcmp(n, ".") == 0 || strcmp(n, "..") == 0);
        zk_add_name(names, count, &room, n);
    }
    closedir(dir);
#endif
    return -1;
}

void zk_host_list_folder(
    unsigned char *slot, const zk_kind *ret, const unsigned char *path
)
{
    char *name = zk_text_cstr(path);
    char **names = NULL;
    int count = 0;
    int member = zk_read_folder(name, &names, &count);
    zk_text_cstr_free(name);
    DO_FOR(i, count)
    {
        const unsigned char *raw = (const unsigned char *)names[i];
        DO_IF(
            member < 0 && !zk_is_utf8(raw, strlen(names[i])), member = ZK_IO_BAD_TEXT
        );
    }
    DO_IF(count > 1, qsort(names, (size_t)count, sizeof *names, zk_name_order));
    if (member >= 0)
    {
        zk_fault(slot, ret, member);
    }
    else
    {
        zk_good_list(slot, ret, count, names);
    }
    DO_FOR(i, count) free(names[i]);
    free(names);
}

void zk_host_path_kind(
    unsigned char *slot, const zk_kind *ret, const unsigned char *path
)
{
    enum
    {
        ZK_IS_FILE,
        ZK_IS_FOLDER,
        ZK_IS_OTHER
    };
    char *name = zk_text_cstr(path);
#ifdef _WIN32
    wchar_t wide[4096];
    DWORD at = zk_wide(name, wide) ? GetFileAttributesW(wide) : INVALID_FILE_ATTRIBUTES;
    int member = at == INVALID_FILE_ATTRIBUTES ? zk_win_member() : -1;
    int kind = (at & FILE_ATTRIBUTE_DIRECTORY) ? ZK_IS_FOLDER : ZK_IS_FILE;
#else
    struct stat st;
    int member = stat(name, &st) != 0 ? zk_errno_member(errno) : -1;
    int kind = member >= 0           ? ZK_IS_OTHER
               : S_ISDIR(st.st_mode) ? ZK_IS_FOLDER
               : S_ISREG(st.st_mode) ? ZK_IS_FILE
                                     : ZK_IS_OTHER;
#endif
    zk_text_cstr_free(name);
    DONE_IF(member >= 0, zk_fault(slot, ret, member));
    zk_good_word(slot, ret, kind);
}

void zk_host_rename(
    unsigned char *slot, const zk_kind *ret, const unsigned char *from,
    const unsigned char *to
)
{
    char *old_name = zk_text_cstr(from);
    char *new_name = zk_text_cstr(to);
#ifdef _WIN32
    wchar_t old_wide[4096], new_wide[4096];
    int ok = zk_wide(old_name, old_wide) && zk_wide(new_name, new_wide);
    int done = ok && MoveFileExW(old_wide, new_wide, MOVEFILE_REPLACE_EXISTING);
    int member = done ? 0 : zk_win_member();
#else
    int done = rename(old_name, new_name) == 0;
    int member = done ? 0 : zk_errno_member(errno);
#endif
    zk_text_cstr_free(old_name);
    zk_text_cstr_free(new_name);
    DONE_IF(!done, zk_fault(slot, ret, member));
    zk_good(slot, ret, NULL, 0);
}
