---
title: The C target
description: How zakac_codegen_target_c turns the IR into C, the Rust runtime it links, and the tests that keep it in step with the VM.
---

# The C target

`zakac_codegen_target_c` is the second backend. It reads the IR that the flow checker
(borrow checker) leaves, the same IR that `zakac_codegen_target_vm` reads, and writes C11.
`zakac build --backend c` runs it and then the C compiler. [Native
programs](../compiler/native.md) shows the commands from the outside; this page explains
how the crate works.

```text
ir::Module ──emit──▶ m0.c … mN.c, main.c, zaka_checks.h, zaka_kinds.{h,c},
                     zaka_ffi.c (with C functions), zaka_callback.c (with closures given to C),
                     zaka_threads.c (with threads)
                         + zaka_target_c_runtime.h, zaka_runtime.h, zaka_c_sugars.h, libzaka_runtime.a
                         │
                         └──cc -c, then link with the runtime lib──▶ executable
```

## An example

Here is the program of the [IR example](codegen.md#an-example), which prints the square
of the constant `အလျား`. `zakac build --backend c --emit c --cache out sq.zaka` writes the
C into the folder `out` and stops before the C compiler, and its module file `m0.c` is:

```text
/* sq.zaka */
#include "zaka_kinds.h"

int64_t zk_g0 /* အလျား */ = {0};
/* နှစ်ထပ် */
int64_t zk_f0(int64_t l0)
{
    #line 3 "/work/sq.zaka"
    ZK_ENTER(1);
    #line 6 "/work/sq.zaka"
    ZK_AT(1);
    int64_t zk_t1 /*  */ = {0};
    zk_t1 = zk_mul_i64(l0, l0, "isize");
    ZK_LEAVE();
    return zk_t1;
}

/* ပရိုဂရမ် */
void zk_f1(void)
{
    #line 9 "/work/sq.zaka"
    ZK_ENTER(2);
    #line 10 "/work/sq.zaka"
    ZK_AT(2);
    zk_put_i(zk_f0(((int64_t)12)));
    zk_put_line();
    ZK_LEAVE();
}
```

The global is `zk_g0` and each function is `zk_f<n>`, with the Zaka name in a comment.
`zk_mul_i64` is the checked `*` of `zaka_target_c_runtime.h`, which stops the run on
overflow and names the data kind (type) `isize` in the message. `ZK_ENTER`, `ZK_AT` and
`ZK_LEAVE` keep the frames of the backtrace, and each `#line` names the source line of
the statement after it, unless the build is for release. The folder has `main.c`, the
runtime headers and `libzaka_runtime.a` beside it.

## The crate

| File | What it does |
| --- | --- |
| `cli.rs` | the standalone command; `zakac build` calls its `command_main` for `--backend c` |
| `driver.rs` | the phases, then `emit`; the cache folder; the runtime headers, embedded with `include_str!`, and the runtime lib of each ABI it carries, embedded with `include_bytes!`: `runtime_lib(abi)` returns its octets (bytes), its file name and the OS libs it needs (`RuntimeLib::native_libs`); `write_cache_for` writes the one of the ABI of the build; `translate` accepts a `TargetCOptions` (`checks`, `max_depth`, `debug_info`, `max_heap`) |
| `emit.rs` | the files of a program: a module file for each source file, the shared header, `main.c`, and the program table `zk_the_program` in `zaka_kinds.c` |
| `stmts.rs`, `exprs.rs` | a statement or an expression of the IR as C |
| `kinds.rs` | the C kind of each scalar data kind, and constants as C text |
| `descriptors.rs` | the description of each data kind the runtime walks: `zk_kind` in `zaka_kinds.c` |
| `ffi.rs` | `zaka_ffi.c`: the structs, declarations and wrappers of the C functions; the `-lX` flags of their libs |
| `callbacks.rs` | `zaka_callback.c`: its first lines (`CALLBACK_HEAD`: `zaka_runtime.h`), the scalar C kinds the bridges name (`zk_ck_<name>`, `scalar_kinds`), then a bridge and a `zk_cb_native_shape` (`zk_cbs_<n>`) for each function kind that goes to C |
| `threads.rs` | `zaka_threads.c`: the guard, the leave and the step of each function a thread runs (see [Threads and channels](#threads-and-channels)), and `native_threads_glue`, the same C for the native target |
| `exports.rs` | a C lib (library): the wrapper of each function C can call, after the glue of `runtime/zaka_lib_glue.h`, and the header file of the lib with the part each lib shares (the status words, the ids of the traps, `zk_trap`, `zk_free`, `zk_last_trap`) |
| `lib_names.rs` | the names of a C lib (`LibNames`): its name, the prefix `zk_<name>_<hash>_` of each name it links under that C never calls, and `zaka_lib_names.h`, which maps each such name the C files use (`zk_f<n>`, `zk_r<n>`, `zk_k<n>`, `zk_g<n>`, `zk_cbs_<n>`, `zk_cbb_<n>`, `zk_the_program`, `zk_lib_init`) and the names of the glue (`zk_lib_reset`, `zk_lib_last_trap`, `ZK_LIB_NAME`) to those of the lib |
| `toolchain.rs` | finding the compiler (`find_cc_for`: for MSVC, `cl` or `clang-cl` on the PATH, else the `cl` of Visual Studio through `find-msvc-tools`, with the variables of its Developer Prompt in `Tool::env`), the flags of each `Dialect` (GNU or MSVC, as data in `Flags`), the compiles of the `.c` files (started at once) and the link, which passes the other inputs, the runtime lib among them, unchanged; `msvc_link` puts the options of the linker after `/link` |
| `wasi.rs` | the changes for WASI (see [WASI](#wasi)) |
| `build.rs` | builds `zaka_runtime` as a static lib for the target of the build, and on Windows for the other ABI too (see [The runtime](#the-runtime)) |
| `runtime/` | the hand-written C11 headers: `zaka_target_c_runtime.h`, `zaka_c_sugars.h` and `zaka_lib_glue.h`, the glue of a C lib (its record in the runtime, the start of its globals, the jump of a trap, its reset and its last trap) |

A function or a global goes into the module file of the source file where its span
starts.

## Values in C

A scalar is a C scalar: `int32_t`, `double`, `uint32_t` for a character, `uint8_t` for a
true/false value (boolean), `unsigned char *` for each address, and `zk_fn` (`const void
*`) for a function value.

### Function values

A function value is the address of a record whose first word is code, `zk_code`. Each
function has a static record `zk_r<n>` = `{code, NULL}` (declared in `zaka_kinds.h`,
defined in `zaka_kinds.c`), and the address of a function is `&zk_r<n>`. A constant folds
to the index of its function plus one, as in the VM, and the C target writes it as that
address.

A closure is a record in a counted heap block. `zk_closure(record, thunk, kind)` moves the
record slot of the IR to a new block, leaves zeros behind, and writes the code of the
thunk into its first word and the descriptor of the record kind into its second word.
That second word is never NULL, so `zk_fn_bound` tests it. A lent closure has the lowest
bit of its address set, which `zk_fn_rec` clears. A call through a value is
`zk_fn_bound(f) ? thunk(args…, zk_fn_rec(f)) : code(args…)`, with the code read through
`zk_fn_code`.

The descriptor walk (`each_block` in `seqs.rs`) visits a `ZK_TAG_FN` value the same as
it visits text: `zk_fn_dup` counts one more holder of a record, a drop lets go of one and
the last holder frees the record with what its fields contain, and `zk_fn_lend` marks a
loan, which a drop leaves alone. A C argument goes through `zk_cb_code` when the function
kind has a shape (see [C functions](#c-functions)), and otherwise through `zk_fn_c`, which
stops a closure with `ZK_TRAP_CLOSURE_ESCAPE`, since C calls a bare code address. These
functions live in `heap.rs` of the runtime, and the table of names has code pointers, so
a print finds the name of a closure through its thunk.

### Lists, structs and text

A list, a struct, a plain list and text are blocks of octets, laid out the same as on the
VM: each field at the offset that `StructKind::offset` gives, and text and plain lists as
three words (the address of the cells, the count, the room). A slot of such a data kind
is an aligned run of `unsigned char`, and its value in C is its address, the same as the
VM pushes the address of an aggregate. A read of a scalar through an address goes through
`zk_ld_<N>` and a write through `zk_st_<N>`, which move the octets with `memcpy`, so an
address of each alignment works. A function receives a list or struct parameter through
its address and copies it into a local, unless the flow checker marks it as one the
function never writes. A list or struct result goes into the slot `zk_ret` that the
caller passes.

The runtime walks a value through its data kind: a drop lets go of each counted block a
value contains, a duplicate (copy) counts one more holder of each, and a print writes
each cell and field. Each data kind it walks gets a `zk_kind` in `zaka_kinds.c`, with its
size, its cell, its fields and their offsets, or its members. The runtime follows the VM
operation of the same name, with the same rules: a room of 0 owns no block, a full block
grows to twice its room, and a list of known cells is static and never freed.

### Counted blocks

Each counted block (text, a plain list, a box, a closure record) starts with a head,
`Head` (`heap.rs`), padded so the cells keep the alignment `malloc` gives. The head has
the count `rc`, the size, the data kinds of its cells for the ring collector and for the
drop, the next block in the queue of blocks to free, the collector's color and buffer
bits, the pin bit, and the count the block counts in (`Head::owner`). Each change of a
count goes through `rc_add`, `rc_sub` and `rc_of` (`heap.rs`): plain operations until
`zk_threads_begin` marks the program as one with threads, and atomic operations from
then on. The trial counts of the ring collector remain plain, since a collection runs with
the other threads halted.

These runtime functions work on counted blocks:

| Function | What it does |
| --- | --- |
| `zk_seq_new`, `seq_grow`, `zk_seq_free` | make, grow and free a counted block |
| `zk_own`, `unshare` | give a block that another value shares its own duplicate before a write |
| `zk_pin` | mark a block so each later duplicate clones it |
| `zk_drop` | let go of the blocks a value contains; it queues them (`FREEING` in `seqs.rs`), so a long chain frees in a loop |
| `zk_deep_seqs` | make a deep duplicate (deep copy), duplicating each object once; the head's next word maps an old block to its new one while the work runs |
| `zk_rings(act, octets)` | serve the statements of the ring collector: run it now (0), pause (1), resume (2), set the threshold (3) |
| `zk_ring_stat(n)` | return one field of the stats, as `ring_stat n` of the VM does: the runs (0), the octets the collections freed (1), the octets waiting (2), the threshold (3), and 1 or 0 for a pause (4); the stats expression lowers to a struct of five `ExprNode::RingStat` reads |
| `zk_obj_ptr`, `zk_ptr_obj` | turn an object into a pointer and back |
| `zk_check_utf8` | check a list of octets that becomes text |

`collect_rings` (`seqs.rs`) runs trial deletion when the waiting blocks reach the
threshold (16,384 octets) and at the end of the run, as the VM does. A block waits when a
block of its data kind can be part of a ring, which is worked out once for each
descriptor.

The blocks of the heap statements of unsafe code are plain `zk_alloc` blocks with no head,
and the two are never mixed. A counted list moved to the heap (`to_heap`) remains
uncounted.

## The order of a run

C leaves open the order of the operands of an operator and the arguments of a call, while
the VM runs them left to right. `operands` keeps that order: an operand goes into its own
local, on a line before the statement, when a later operand could change it or stop the
program first. The C target writes no statement expressions. The right side of `&&` and
`||` runs just when needed, so when it needs its own lines, those go inside an `if`.

A fresh value that contains text, such as the text a format builds or a call result, is
dropped right after the statement reads it. `ir::fresh_text` defines that rule for both
backends.

An else-if chain of the IR is written flat as `else if`, not as nested blocks, so a chain
of thousands of branches builds with each C compiler.

## Checks, sites and stops

`zaka_checks.h` defines one `ZK_CHECK_<NAME>` as 1 or 0 for each run-time check
(`RunCheck` in `options.rs`: `DIVIDE`, `SHIFT`, `BOUNDS`, `TEXT`, `STEP`, `DEPTH`, `NULL`,
`OVERLAP`, `SANITIZERS`), plus `ZK_MAX_DEPTH` and `ZK_DEBUG_INFO` (1 in a debug build, 0
under `--release`), for the inline code of `zaka_target_c_runtime.h` and the program.
`ZK_CHECK_SANITIZERS` changes no C: the build adds the sanitizer flags when it is on.

The runtime reads the same values from the program table. `main` calls `zk_start(argc,
argv, &zk_the_program)`, and `zk_the_program` in `zaka_kinds.c` has the check bits
(`RunChecks::bits`), the limits (`--max-depth`, `--max-heap`, 0 for no cap), the texts of
the traps from the locale of the compiler, the rights of the build, the names of the
functions and the sites. So the lib is the same for each program and each set of
checks. `RunChecks` stores the set as bits, and `--checks` builds it from a level
(`Checks`), then each `+name` or `-name`.

### Frames and sites

Each OS thread keeps its calls in progress in its own `zk_context` (`ZkCtx` in `ctx.rs`:
`depth`, `frames`, `room`), which `zk_ctx()` returns, making the frame table on first use.
A thread of the program has a context of its own, which `zk_ctx()` returns while an OS
thread runs the thread (`Local` in `local.rs`), so the `zk_c` that a function keeps belongs
to the thread and not to the OS thread.
`ZK_ENTER(f)` opens each function and keeps the context in the local `zk_c`, while `main`
and `zk_lib_init` get it through `ZK_HERE()`. `ZK_LEAVE()` closes each return. With
`ZK_CHECK_DEPTH` on, the pair counts `zk_c->depth` and calls `zk_too_deep` past
`ZK_MAX_DEPTH` (`--max-depth`).

Frames are kept when the depth check is on or `ZK_DEBUG_INFO` is 1, so `--checks none`
still names the place in a debug build. The pair then pushes and pops a row of the frame
table `zk_c->frames`, one row for each call in progress, with the function number and the
site. `ZK_AT(site)` comes before each statement whose site differs from the last one, and
sets the site of the top row. The sites (file, line, column of the start of a statement)
are tables in `zaka_kinds.c` that the program table points at, and a `--release` build
writes neither the tables nor the marks. A check inside an expression names the site of
that expression, as the VM does: each checked operation of the header (`zk_div_<N>`,
`zk_ld_<N>` and the rest) and each call of the runtime that checks a value
(`zk_cell_of`, `zk_char_at` and the others) takes the site last, or `ZK_NO_SITE` outside
an expression. The header notes it on the failing branch alone (`ZK_STOP_AT`), so the
order C gives the arguments of a call never mixes two sites; a call of the runtime keeps
it in `call_site` of the context while it runs, and the stop notes it. With the checks of depth and debug info both off,
the pair counts nothing.

A stop writes the function and the place, then the backtrace from the frames of its
thread, with the rules of the VM. `ZAKA_BACKTRACE` unset folds a run of one call to `...
the call above, N times more` and cuts past 16 lines to `... N calls more`, `full` lists
each call, and `0` lists none. The C `main` is left out.

### Heap limit

`--max-heap` counts in the runtime. A counted block adds the size of its cells when it is
made, and a raw block (`zk_alloc`, `zk_realloc`, `zk_raw_free`) adds the size the allocator
gives (`malloc_size` on macOS, `malloc_usable_size` on Linux, `_msize` on Windows, while
another host counts no raw block). A counted block past the cap stops the run with the
trap `no heap room for N octets`, a raw allocation past it returns NULL, and a resize
past it stops the run. A block that C made and the program frees was never counted, so
the count stops at 0, and a block given to C that C frees is never given back.

### Halts and prints of results

An optional and a result are the same struct as in the VM: a tag, then `some`, or `ok`
and `err`. `Stmt::Halt` becomes a call that stops the run, and `stmts.rs` writes each
one:

| Cause | Call |
| --- | --- |
| `not_good` with an error value | `zk_not_good(at, kind)`, which shows the error value |
| `not_good` with none | `zk_not_good_bare()` |
| `not_fault` | `zk_not_fault()` |
| `closed_handle` | `zk_closed_handle()` |
| `null_from_c` (C returned NULL for a plain handle result) | `zk_plain_trap(ZK_TRAP_NULL_FROM_C)` |
| `no_data` | `zk_no_data()`, which stops with `ZK_TRAP_NO_DATA` and the text of the trap `the optional holds no data` from the locale (`target_c.trap.no_data`) |

A print of an optional goes through `push_fallible` (`show.rs`), which writes
`ဒေတာ(<value>)` for tag 1 and `အလွတ်` for tag 0, with the words `data_text` and
`no_data_text` that `emit.rs` puts in the program table. The null address prints as
`null_text`, the phrase `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`, and a result writes
`ရလဒ်ကောင်း(...)` or `အမှားအယွင်း(...)`, all as in the VM.

A float prints from its bits (`zk_put_fbits(bits, raw)`, where `raw` is the 32- or 64-bit
pattern), through the number text of `zaka_host::numbers`, which the VM prints with, so
each host prints the same digits.

The headers are strict C11 and pass `-pedantic-errors`. The GCC and Clang builtin for an
overflow test sits behind `__GNUC__`, with a C11 path beside it.

## The runtime

The runtime is the Rust crate `zaka_runtime`, a static lib (`crate-type = ["staticlib",
"rlib"]`) that both the C target and the [native target](native-backend.md) link. Each of
its functions is `extern "C"`, with the name that the C calls, and its build script scans
`src/*.rs` for them and writes `exports()`, the table that the JIT passes to Cranelift.

| File | What it contains |
| --- | --- |
| `program.rs` | `ZkProgram`, the mirror of `zk_program`: the checks, the limits, the rights, the texts, the function names and the sites; `zk_prog`, the table of the running program |
| `ctx.rs` | `ZkCtx` (`zk_context`) and `zk_ctx`, the calls in progress of each OS thread, or of the thread it runs |
| `local.rs` | `Local`, what a thread keeps for itself (its context, its trap jump, its number, its output sink and line), swapped with the cells of the OS thread that runs it |
| `tasks.rs` | the pool of light tasks: `zk_task_start`, one OS thread for each core, the queue and the sleeping tasks, a task that parks in a wait, sleeps or ends its turn at `zk_safe_point` (a second bit of `zk_halt`) |
| `fiber.rs` | the stack of a light task and the switch between stacks: on unix a mapped stack with a guard of 64 KiB and naked functions for aarch64 and x86_64 whose link names hold the hash of the build; on Windows a fiber of the OS (`CreateFiberEx`, `SwitchToFiber`); the catch of an overflow (a signal handler on unix, a vectored exception handler on Windows) |
| `start.rs` | `zk_start` and `zk_finish`: the start and the end of a run |
| `traps.rs` | the traps, the stop message and the backtrace |
| `out.rs` | the buffer of standard output, the sink of a format, the print of numbers |
| `heap.rs` | counted blocks, raw blocks, function values and closures, the heap count |
| `seqs.rs` | plain lists, text edits, drops, duplicates, loans, the ring collector |
| `show.rs` | `==`, the print of each data kind, results and optionals, the rows of a ဇယား (map) |
| `host.rs` | host calls for streams, files, arguments, the environment, math and time |
| `rights.rs` | the rights of the run, and the asker on a terminal |
| `threads.rs` | threads and channels, each thread as a light task or on its own OS thread (see [Threads and channels](#threads-and-channels)) |
| `waiters.rs` | the light tasks that wait, filed under what each waits on (a thread, a lock, the bit of a channel, a time limit, each change), so a change looks at the tasks it concerns alone |
| `libs.rs` | the record of each C lib the runtime serves |
| `exchange.rs` | the C form of a value that a C lib passes to C or receives from it |
| `callbacks.rs` | the libffi closures of the closures that go to C |
| `ffi.rs`, `native.rs` | the parts the native target calls: `zk_ffi_call`, `zk_cb_shape_of`, `zk_enter_call`, `zk_at_site`, `zk_fmod` |

The C side reads the runtime through three headers:

| File | What it contains |
| --- | --- |
| `zaka_runtime.h` | written from the Rust through cbindgen at build time: a declaration of each runtime function, the structs the generated code shares with the runtime (`zk_program`, `zk_kind`, `zk_field`, `zk_site`, `zk_cb_shape`, ...), the ids of the traps, the rights and the data kinds (`ZK_TRAP_*`, `ZK_RIGHT_*`, `ZK_TAG_*`), `zk_context` and `zk_ctx()`, and `zk_prog` |
| `zaka_target_c_runtime.h` | hand-written: the inline code the generated C runs for each operation, the checked integer operations of each data kind (`zk_add_i32`, `zk_div_u64`, ...), the loads and stores and the marks of the calls in progress; it includes `zaka_runtime.h` |
| `zaka_c_sugars.h` | `RET_IF`, `DONE_IF`, `DO_IF`, `NEXT_IF`, `LEAVE_IF`, `DO_FOR` |

`zaka_target_c_runtime.h` includes two headers that a build writes, and
`zaka_lib_glue.h` includes a third, `zaka_lib_names.h`, so an editor needs the headers of
a build beside them to read them. `mise run editor:mock-header` writes a build of hello
and of a lib into `target/c-headers`, and `runtime/.clangd` points clangd there.

### Link names

Each name the runtime exports links as `zk_rt_<version>_<hash>_<name>`: the version of the
crate, and a hash of its sources and those of `zaka_host` (`zaka_runtime/stamp.rs`, which
the build scripts of the runtime and of this crate both include). The Rust writes
`#[unsafe(export_name = stamped!("alloc"))]`. cbindgen reads a duplicate of the sources
with a plain `no_mangle` in its place, and the build script puts a `#define zk_alloc
zk_rt_..._alloc` for each name before the declarations, so the C keeps the plain names.
`RUNTIME_PREFIX` and `linked_name` give the prefix to Cranelift, and the table of exports
of the JIT carries the linked names.

A lib that `--runtime-lib` names brings its own prefix: `driver::prefix_in` reads it off
the name of `zk_seq_new` in the lib, and `runtime_prefix` serves it to `linked_name` and to
the header written into the cache. Each nested build of the runtime sets
`RUSTFLAGS=-C metadata=<prefix>`, because Rust derives the link names of each crate from
its metadata, not its sources, so two runtimes of other sources in one program would
clash on them. `ZAKA_RUNTIME_SALT` joins the hash, for a test of two builds.

### C libs

The runtime keeps a record of each C lib it serves (`zaka_runtime/src/libs.rs`): its
table, the jump back to its wrapper, its gate, its last trap and its heap count. Each
wrapper enters its lib (`zk_lib_enter`, which also pushes a mark of the lends, the sink of
the output and the drop queues) and returns to the lib around it, and a trap puts the
mark back (`zk_lib_unwind`).

An export that `runs_free` (in the flow checker) marks runs with no lock (`FREE_PARTS` in
`exports.rs`). `zk_lib_free_in` counts it in `FREE` once the lib runs and the counts are
atomic, a trap fails the call alone (`zk_lib_fail`), and the ring collector and the start
of the globals wait for `FREE` to drain. The state of a lib (`zk_lib_state_of`) and the
status of a trap (`zk_lib_status`) live in the runtime, so a call with no lock reads them
with no race. The glue of each lib defines `zk_trap_slot_v1` weak (`selectany` on
Windows), so the linker keeps one for the program, and each runtime writes each trap to
the slot of each lib it serves. A counted block records the count it counts in
(`Head::owner`), a raw block of a lib with a cap goes in `RAW_OWNERS`, and a block the lib
gives C counts in none.

`zaka_runtime/src/exchange.rs` writes and reads the C form of a value (`zk_to_c`,
`zk_from_c`): a plain list one cell at a time, each in its C form. A block it gives C
starts `C_HEAD` (16) octets after a head with the data kind of its cells and their count,
NULL for text. `zk_free` frees from the head, and `zk_free_all` walks the cells first
(`free_inside`) and frees each text and list the cells contain. The header file of a lib
declares each struct and each plain list in the order in which each contains the others
(`CKinds::order`), and names a list after its cells (`cell_part`). A struct of Zaka
itself, whose name starts with `%` (such as the struct of a ဇယား), has no C form.

### cbindgen

`zaka_runtime/cbindgen.toml` sets out `zaka_runtime.h`: the C names of the Rust structs
(`ZkProgram` is `zk_program`), the enum members with the name of their enum in front
(`ZkTrap::ZeroDivisor` is `ZK_TRAP_ZERO_DIVISOR`), and `ZK_NORETURN` for a function that
never returns, an attribute of GCC and Clang. The build script keeps the first paragraph
of each doc comment, so the header carries the gist of each item and the Rust carries the
rest. A function value is `zk_fn` (`ZkFn`, `const void *`) and its code `zk_code`
(`ZkCode`, `void (*)(void)`), so the generated C needs no cast between an object pointer
and a function pointer. The C the compiler writes names each field of a shared struct
(`.tag = ZK_TAG_SINT`) and puts each text of a table of traps or rights under its id
(`[ZK_TRAP_OVERFLOW] = "..."`), so a field or a member that moves in the Rust moves in the
C too.

### Building the runtime lib

The `build.rs` of the crate runs its own cargo: `cargo rustc -p zaka_runtime --lib
--crate-type staticlib --profile zaka-runtime --target <target>`, into a target folder in
`OUT_DIR`, with `--print native-static-libs` for the OS libs the lib needs. The
`zaka-runtime` profile builds for release with `panic = "abort"`. A GNU lib is stripped
(`strip -S`), and `include_bytes!` puts it into `zakac`. `driver::write_cache_for` writes
it into the cache folder of each build, and the link puts it after the objects. A build
for `wasm32` (the explorer) gets blank libs.

Beside the lib of the host, `build_other` builds the libs of other targets at once, each
in its own thread and target folder. On Windows, it builds the lib of the other ABI
(`other_windows`: `x86_64-pc-windows-gnu` and `-msvc` swapped, and on ARM64
`aarch64-pc-windows-gnullvm` and `-msvc`), named as that ABI names it
(`zaka_runtime.lib` for MSVC, `libzaka_runtime.a` for GNU). On each host, it builds the
libs of `wasm32-wasip1` and `wasm32-wasip2`, which are not stripped, because the `strip`
of the host reads no WebAssembly. When a lib cannot build (the Rust target or the C tools
of that ABI are missing), the build warns and carries a blank lib, and `write_cache_for`
refuses that ABI (`RuntimeNotCarried`).

## WASI

`--target-os wasi` keeps the C of each other host, with three changes in `wasi.rs` and
one in `main.c`:

- `wasi::find_cc` uses `--cc`, then `bin/<triple>-clang` of `WASI_SDK_PATH`, then
  `<triple>-clang` on the PATH. The Clang of wasi-sdk named for a triple builds for it with
  the headers and libs of the SDK. `CC` is skipped, because it names a compiler of the host.
- `wasi::link_flags` adds a stack of 8 MiB (the linker gives 64 KiB), and for `wasip2`
  `--export=cabi_realloc`: the std of the runtime lib defines that function, which the
  component needs, and the linker leaves it unexported unless asked.
- `cli::other_host` lets a `wasi` target with the CPU `wasm32` and a WASI ABI through,
  and a build with C functions stops with `CFunctionsOnWasi`. On WASI, the runtime leaves
  out its modules `ffi` and `callbacks`, and libffi with them.
- `main.c` starts with `#if defined(__wasi__)`: WASI starts a `main` of two parameters
  alone, so that `main` reads the environment from `environ`, where the other hosts pass
  a third parameter, `envp`.

## ဇယား values

`zk_row_find`, `zk_set_row` and `zk_remove_row` in `show.rs` keep the same table (hash
table) that the VM keeps: rows in add order, slots of `uint32_t` in a power of two, a
rebuild when a third of the slots would no longer be free, and FNV-1a over the parts of an
index (key). A ဇယား keeps the struct tag in its data kind, and `is_table` (`show.rs`)
knows it through its name, `%map`, so the walks that drop, duplicate and own a struct
need no change, and `zk_eq` and `zk_show` call the ဇယား steps first. The trap of a missing
row is `ZK_TRAP_NO_ROW`.

## Threads and channels

The C target runs each thread (`အပြိုင်လုပ်ငန်း`) as a light task, on a stack of its own that
a pool of OS threads takes turns on, or on an OS thread of its own. A spawn calls
`zk_task_start` when `light_fns` of the borrow checker holds its function and the call is
not written `သီးသန့် အပြိုင်`, and `zk_thread_start` otherwise; both take the same arguments,
and the rest of this section holds for both.

The number a start gives names a row of the thread table of the runtime: the row plus one
in the low half, and in the high half a count of the threads that held the row before. A row
goes back once a wait took the result (`zk_thread_taken`), and a spawn written as a statement
wraps its start in `zk_thread_detach`, so the row of a thread no one can wait for goes back
as the thread ends; its trap still ends the program with code 1. A number whose row went to a
later thread names no thread, so a second wait for it traps `thread waited`.

### The C side

`threads.rs` writes `zaka_threads.c` for a program that starts a thread. The file has:

- `zk_thread_guard`, which runs a step under `setjmp` and keeps the jump through
  `zk_lib_jump_set`, which belongs to the OS thread;
- `zk_thread_leave`, which jumps back;
- `zk_thread_step_<f>` for each function a thread runs, which calls it with the arguments
  of the data block and writes the result at its start. The function drops its
  parameters as it returns, so the step drops none.

A spawn gets a data block (`zk_thread_data`), moves each argument into it at a 16-octet
part (`part_offsets`), and calls `zk_thread_start`. The arguments come owned, the same as
the value of a set does, since calls alone lend. A wait moves the result out of the block,
and `zk_thread_taken` frees the block (the word in front of it has its size). A wait
whose value nothing reads calls `zk_thread_drop`, which drops the result and frees the
block. A second wait for a thread is the trap `thread waited`. A receive writes the
optional into a scratch slot, and a send moves its value through its address.

### The runtime side

A light task (`tasks.rs`) runs on a stack of 1 MiB from `fiber.rs`, which the switch
between stacks reaches through naked functions. Below the stack lie 64 KiB that no access
can reach; a fault there, which a handler on a signal stack of each OS thread of the pool
catches, writes the trap `stack full` with no allocation, after the output that waits
for stdout (`flush_raw`, with no lock that waits). Under `--trap unwind` the handler jumps
back to the guard of the task, so the thread alone ends (the signal is open again:
`SA_NODEFER`); under `--trap abort` it ends the program. Each other
fault goes on to the handler that was there before. On Windows a task is a fiber of the OS,
which keeps 32 KiB of its stack for the handler (`SetThreadStackGuarantee`), and a vectored
exception handler turns the stack overflow of a task into the same trap. Under `--trap
unwind` the handler does not jump to the guard, since a jump out of the handler unwinds
through the dispatch of the OS: `end_running` marks the task ended at the trap and switches
back to the pool for good, as a task whose guard returned does. A task takes its
`Local`, its call context and its other cells, from one OS thread of the pool to the next.
It stops in `wait_until`,
which drops the lock of the state first (a wait on several channels, `zk_select`, waits
there too: a light task in it parks on `Key::Pipes`, a mask of its channels, so a change of
one of them wakes it, and an OS thread of the runtime marks the end of each time limit with
`Key::Timer`, and a wait is no deadlock while such a limit runs), in `zk_sleep`, in a call on a file or a read of standard input (`blocking`
in `host.rs`: an OS thread for I/O, up to 64 of them, runs the call while the task waits,
so a slow disk or a named pipe holds no OS thread of the pool), and in `zk_safe_point` once its
turn of 10 ms is over while other tasks wait. Each loop of each OS thread reads
`zk_halt`, so the OS thread that watches the turns sets its turn bit just while a task is
past its turn and others wait, and a task that gives its turn on clears it: the loops of
the program start block take the call for that short while alone. A parked task counts as not running, as a
waiting OS thread does, so the deadlock rule below holds for both. A parked task waits on
a key (a thread, a channel, a lock, or each change), and a change wakes the tasks of its
key alone; a woken task counts as halted for the ring collector until it runs again.

The runtime (`zaka_runtime/src/threads.rs`) keeps the threads under one lock and one
condition. Each channel has a lock of its own and a count of its waits (`PipeCell`): a send,
a receive or a close takes the lock of the channel alone, and takes the lock of the state to
wake the waits just when its count is above zero. A wait raises the count before it looks,
so a change after the look sees it, and then waits through `wait_until`, as the other waits
do. It counts the OS threads that run: the main one (`mark_main`, from
`zk_start`) and each thread of the program. A wait that would leave none running is the
trap `deadlock`, unless an OS thread outside the count can still act (`outside_acts`):

- each OS thread of a C lib;
- a call from an OS thread of C that runs now (`free_calls`);
- a closure with captures that C has and can call from its own OS thread
  (`lent_to_c_threads`); a function with no captures reaches no value a thread waits on.

An OS thread of C waits and changes no count, and once it leaves its call,
`recheck_waits` wakes each wait to check again. `zk_thread_start` turns the counts atomic
(`zk_threads_begin`) and starts an OS thread with a stack of 64 MiB. Under `--trap
unwind`, `stop` writes `thread <n> stopped: ...` and calls the leave of the thread.
`zk_finish` waits for each thread, drops each result and each value a channel contains,
and makes the status 1 when a trap that no wait saw ended a thread.

While threads run, `collect_rings` asks the other running OS threads to halt at a safe
point and collects once each one halted (`halt_others`). The safe points
(`seqs::safe_point`) are each allocation in `take_into`, each candidate in
`note_candidate`, each wake in `wait_until`, and, in a program with threads
(`starts_threads`), each round of a loop: `ZK_SAFE_POINT()` in C, which reads the flag
`zk_halt`, and in Cranelift code an atomic load of the address that `zk_halt_flag`
returns. When one OS thread reaches no safe point in 20 ms, the collection is dropped, and
the next one waits until the buffer doubled (`RETRY_AT`). The lists of the deep duplicate
belong to each OS thread.

### Locks

`zk_lock` and `zk_unlock` find each lock through the block of the object (`zk_seq_base` of
the variable) and name the holder through `owner`: the number of a thread of the program,
or, for each other OS thread, a number from the high half (`NEXT_LOCKER`), so two OS
threads of C never pass as one. The locks sit apart from the state of the threads, in 64
shards on the address of the object (`LOCKS`). A lock that no other OS thread holds is
taken, and one that no OS thread waits for is given back, under the lock of its shard
alone (`take_free`). A lock that another OS thread holds waits through the same deadlock
check, counted in `LOCK_WAITERS`, so a give-back wakes waiters when there are some. An OS
thread can take again a lock it holds, and a thread that stops at a trap leaves its locks
broken (`LockBroken`). Eight threads that each lock their own object 200,000 times ran in
0.21 s with the locks in the state, and in 0.01 s with the locks apart.

## C functions

### zaka_ffi.c

`emit` writes `zaka_ffi.c` when `Module::foreign` has rows. The file includes
`<stddef.h>` and `<stdint.h>` and no libc header, so a declaration of a C function in it,
with the C kinds Zaka gives it (`_Bool`, `void *` for each address), cannot clash with the
one in a header. Pragmas turn off `-Wincompatible-library-redeclaration` (Clang) and
`-Wbuiltin-declaration-mismatch` (GCC). For each struct a call passes, the file defines
`zk_ffi_s<n>` with a `_Static_assert` on its size, and for each C function a wrapper
`zk_ffi_<n>` (`wrapper_name`, `wrapper_proto`), declared in `zaka_kinds.h`.

A handle of a data kind from outside the program crosses as `void *`. The wrapper itself
is little more than the call, because the IR does the rest: it guards each handle
(`Stmt::Halt` with the cause `ClosedHandle`, which `stmts.rs` writes as
`zk_closed_handle()`, a runtime function that stops the run with
`ZK_TRAP_CLOSED_HANDLE`), wraps a result with the box of the handle after a test that
stops the run on NULL for a result written with no optional (`Halt` with the cause
`NullFromC`), and empties the box after a take-over. A field read, `ExprNode::CField`, is
`c_field` in `exprs.rs`: a `memcpy` from the address of the C struct plus the offset, into
a scratch slot of the data kind of the field. A `char *` field goes through
`zk_text_from_c`, which returns the optional with no data for NULL.

### Call shapes

A wrapper serves one shape of call, `FfiCall { func, extra }`: the C function and the data
kinds of its other values. `ffi_call` in `exprs.rs` numbers the shapes of the module in
`Shared::ffi_calls`, so a C function that accepts other values (`ForeignFn::variadic`,
declared with `, ...`) has a wrapper for each shape of its calls, and the parameters of a
wrapper are the named ones, then the other values. `ffi_source` defines the structs of
`FfiCall::extra` too, so a struct can be an other value.

A struct goes in and out of a wrapper as the address of its octets, and a struct result
goes to the address in the first parameter, `zk_out`. A text goes in as the address of its
octets and its count. The wrapper makes a duplicate that ends in a zero octet with
`zk_text_cstr` (`seqs.rs`, declared in `zaka_target_c_runtime.h` and in the first part of
`zaka_ffi.c`), and frees it with `zk_text_cstr_free` after the call. The module files call
the wrappers through `ExprNode::Foreign` and `Stmt::Foreign`. A list of numbers comes as
the address of its first cell, which the IR built before the call, so the wrapper passes
a pointer, the same as for each other address.

A result of the data kind `စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` (`is_c_text_result`) is
kept as `const char *` and passed to `zk_text_from_c(out, size, ok_at, s)` (`host.rs`).
NULL writes the optional with no data, and otherwise the string is copied into new text of
the program (`zk_text_of_c`), with `bad_utf8` for octets that are not UTF-8. The wrapper
calls it before it frees the duplicates of the text arguments, since `strstr` and
`strchr` return pointers into them.

### Function values that go to C

A parameter of a function kind that `is_callback` accepts is declared with the C pointer
kind of its signature (`void *` for each address), and the wrapper passes the code that
`zk_cb_code` returns (below), cast to it. A function value contains a function at each
point, so C never gets NULL there. A trap in a callback is the trap of the run: the
program stops where it happens.

Function values go to C through `zaka_callback.c`, which `emit` writes when the program
gives function values to C or has a leave statement (`Stmt::Leave`). The file starts with
`zaka_runtime.h`; the runtime keeps `zk_cb_code`, `zk_cb_lent`, `zk_cb_end`, `zk_cb_leave`
and `zk_cb_revoke` in `callbacks.rs`, with `zk_fn_size` and `zk_fn_release` in `heap.rs`.
Then, for each function kind that goes to C, the file has a bridge and a
`zk_cb_native_shape` named `zk_cbs_<n>`: the C kinds of its result and parameters
(`zk_c_kind`), the same as the native backend writes them.

A wrapper passes `zk_cb_code(value, zk_cb_shape_of(&zk_cbs_<n>))` for such a parameter
(`shape_ref`), and wraps the C call in `zk_cb_lent()` and `zk_cb_end()`, so the closure is
lent to the call. A leave statement is `zk_cb_leave(value, zk_cb_shape_of(&shape))` or
`zk_cb_revoke(value)`. `zk_cb_shape_of` makes the libffi kinds of a shape on its first use
and keeps them for the run. Each closure gets one libffi closure, found again from the
octets of its record, and its code is never freed, so a late call from C stops the run
with `ZK_TRAP_CLOSURE_ESCAPE` and never runs freed code.

### Calls from OS threads of C

A call from an OS thread that C started runs when the table of names allows it: the row of
the function, or of the thunk of a closure, in `fn_names` has `crosses` set. It is set:

- for a function that, itself or through each function it calls, uses no mutable global;
- for a thunk when each capture of its record is a duplicate of a value with no box,
  function value, pointer or handle, except an object of a struct marked for threads
  (`Module::threads_structs`), whose fields each thread reaches under its lock
  (`crossing_fns` in the flow checker).

A write to a field of an object leaves the variable unwritten (`note_written` in the flow
checker), so such a capture is a duplicate. The runtime finds the row through the code
word of the record. `run_on_c_thread` keeps a count of a closure's record while the lend
or the leave holds (`LENDING`, a lock that the lends, the leaves and their ends take too),
counts the call in `FREE`, so the ring collector waits, and the lend turned the counts
atomic before. A call after the lend with no leave stops with `ZK_TRAP_CLOSURE_ESCAPE`.
Each other call from another OS thread ends the program with a message and status 1.

In a C lib, `run_in_lib` handles the call instead. On an OS thread with calls parked in C,
it takes them back, runs the bridge and lets go again. On another OS thread, it runs the
bridge through the guard of the lib (`zk_lib_guard` of the glue, which `zk_lib_open`
receives), whose `setjmp` catches a trap, and C gets zeros. Each function C calls goes
through the bridge, a plain function too, so no call skips the lock of a lib or the
thread rules of a program.

A call to C in a lib lets go of the lock (`ZK_LIB_LET_GO` in each wrapper of
`zaka_ffi.c`, which `ZK_LIB` of `zaka_lib_names.h` turns on, and `zk_ffi_call` for
Cranelift code). `zk_lib_let_go` writes out the output, keeps the count of its holds in a
thread-local list (`PARKED`) and lets go of the lock at each depth it held. The rest of
the state of a call (the lib, its base, the table, the marks, the lends, the jump target,
the sink and the drop queue) belongs to each OS thread. `zk_lib_take_back` waits for the
lock and takes it back at that depth. The jump target of a trap lives in the runtime
(`zk_lib_jump`, `zk_lib_jump_set`), not in the glue, since an OS thread that waits in C
lets others into the same lib.

### How a parameter crosses

A function kind whose parameters each cross as written (`Crossing::AsIs`: numbers,
characters, booleans and addresses) lets a plain function reach C as its own code. Each
other function kind that a C declaration accepts (`callback_param`) goes through the
bridge, a plain function too (`cb_of` keeps one libffi closure for it, `kept` for the
run), and `crossing` picks how each parameter crosses:

| Crossing | What the bridge does |
| --- | --- |
| `AsIs` | passes the number, character, boolean or address as it is |
| `Struct` | a struct passed as a whole value gets a C kind with the tag 12 (`zk_cbst_<i>`, its members in `zk_cbse_<i>`, a counted list as one element for each cell) and is copied into an aligned block |
| `Text` | makes text with `zk_text_of_c`; NULL is `ZK_TRAP_NULL_FROM_C` |
| `OptText` | makes an optional of text with `zk_text_from_c` |
| `Handle` | makes a new box around the address with `zk_cb_handle`; NULL gives a box that contains 0 |
| `OptHandle` | makes an optional of a handle with `zk_cb_opt_handle` |
| `Rich` | reads a struct or a plain list in its C form, with text and lists in it, into a new value, for a function a lib gives C |

The function gets a lent duplicate (`zk_lend_seqs`, room 0), and the bridge drops the
original with `zk_drop` once the function returns. `zk_fn_c` remains the guard of a
program with no `zaka_callback.c`. A leave of a function kind that the bridge has no shape
for (a parameter that `callback_param` refuses, or a result that is not a number,
boolean or address) is a construct the target cannot build. The runtime lib carries
libffi itself (the `libffi` crate builds it), and the bridges name no libffi kind, so no
build needs the host's libffi, its header or `-lffi`. A bridge writes a narrow integer
result as a whole word (`intptr_t` or `uintptr_t`, the width of libffi's `ffi_arg`).

### Link flags

`link_libs` gives `-lX` for each lib `libX` that the program names, except libc and libm,
in order of first use. `BuildPlan::libs` puts them after `-lm` and before the `LDFLAGS` and
`--ld-flag` flags of the user, where a `-L<folder>` flag still reaches them. `lib_flags`
passes a lib path (`is_lib_path`) to the linker as it is, and, on each host except
Windows, `-Wl,-rpath,<folder>` after it. The loader made a relative path absolute from the
folder of its file, so the flags contain an absolute path. On macOS, the lib needs an
install name that is an absolute path or `@rpath/...`.

## Tests

| Test | What it checks |
| --- | --- |
| `tests/samples.rs` | each repo sample prints, as an executable, the output the VM prints for it, at `--checks default` and `--checks none`, under `--overflow stop` and `wrap`; it passes with a note on a host with no C compiler, unless `ZAKA_REQUIRE_CC=1` makes that a failure, as on CI |
| `tests/ffi_lib.rs` | C functions, structs as values, other values, handles and closures given to C, against a C lib the test builds |
| `tests/libs.rs` | a C lib built from Zaka (`--emit lib`): a C program links the static lib and the shared lib, calls the functions that code outside can call, gets a trap back as a status, and finds the lib going on after it |
| `tests/text.rs` | the runtime writes thousands of floats and the ends of 64-bit integers the same as the VM writes them |
| `tests/floats.rs` | float bit patterns, random and hard ones, print the same as the VM prints them |
| `tests/traps.rs` | a zero divisor and an overflow stop with the VM's message and status 1 |
| `tests/permissions.rs` | the rights an executable has are the ones its build granted: a denied file or variable read fails with no permission, and a denied C call stops the program and names the flag |
| `tests/debug_info.rs` | a `#line` before each statement names its source line, unless the build is for release |
| `tests/forms.rs` | a C compiler sets out each struct of `zaka_runtime.h` the same as rustc does (each `sizeof` and `offsetof` against `size_of` and `offset_of!`), each struct and field of the header has a row in the test, and the trap texts, the tags and the rights the compiler lists follow the enums of the runtime |
| `zaka_website/tests/stops_in_c.rs` | each `zaka-stop` program of the books, built with this target, prints the output and the stop reason of the VM (see [Build, test and format](README.md#build-test-and-format)); a stop here names no source line and no calls in progress, which the VM names |

Each test builds and runs its program in its own folder, so a sample that writes a file
writes it there.
