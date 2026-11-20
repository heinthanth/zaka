---
title: The C target
description: How zakac_codegen_target_c turns the IR into C, the Rust runtime it links, and the tests that hold it to the VM.
---

# The C target

`zakac_codegen_target_c` is the second backend. It reads the IR that the flow checker
leaves, the one `zakac_codegen_target_vm` reads, and writes C11. `zakac build --backend c`
runs it, then the C compiler. [Native programs](../compiler/native.md) tells what a user
sees; this page tells how it is built.

```text
ir::Module ──emit──▶ m0.c … mN.c, main.c, zaka_checks.h, zaka_kinds.{h,c}, zaka_ffi.c (with C functions), zaka_callback.c (with closures given to C)
                         + zaka_target_c_runtime.h, zaka_runtime.h, zaka_c_sugars.h, libzaka_runtime.a
                         │
                         └──cc -c, then link with the runtime lib──▶ executable
```

## The crate

| File | Job |
| --- | --- |
| `cli.rs` | the standalone command; `zakac build` calls its `command_main` for `--backend c` |
| `driver.rs` | the phases, then `emit`; the cache folder; the runtime headers, embedded with `include_str!`, and the runtime lib of each ABI it carries, embedded with `include_bytes!`: `runtime_lib(abi)` gives its octets, its file name and the OS libs it needs (`RuntimeLib::native_libs`); `write_cache_for` writes the one of the ABI of the build; `translate` takes a `TargetCOptions` (`checks`, `max_depth`, `debug_info`, `max_heap`) |
| `emit.rs` | the files of a program: a module file for each source file, the shared header, `main.c`, and the program table `zk_the_program` in `zaka_kinds.c` |
| `stmts.rs`, `exprs.rs` | a statement or an expression of the IR as C |
| `kinds.rs` | the C kind of each scalar kind, and constants as C text |
| `descriptors.rs` | the description of each kind the runtime walks: `zk_kind` in `zaka_kinds.c` |
| `ffi.rs` | `zaka_ffi.c`: the structs, declarations and wrappers of the C functions; the `-lX` flags of their libs |
| `callbacks.rs` | `zaka_callback.c`: its head (`CALLBACK_HEAD`: `zaka_runtime.h`), the scalar C kinds the bridges name (`zk_ck_<name>`, `scalar_kinds`), then a bridge and a `zk_cb_native_shape` (`zk_cbs_<n>`) for each function kind that goes to C |
| `exports.rs` | a C lib: the wrapper of each function C can call, after the glue of `runtime/zaka_lib_glue.h`, and the header of the lib with the part each lib shares (the status words, the ids of the traps, `zk_trap`, `zk_free`, `zk_last_trap`) |
| `lib_names.rs` | the names of a C lib (`LibNames`): its name, the prefix `zk_<name>_<hash>_` of each name it links under that C never calls, and `zaka_lib_names.h`, which maps each such name the C files use (`zk_f<n>`, `zk_r<n>`, `zk_k<n>`, `zk_g<n>`, `zk_cbs_<n>`, `zk_cbb_<n>`, `zk_the_program`, `zk_lib_init`) and the names of the glue (`zk_lib_reset`, `zk_lib_last_trap`, `ZK_LIB_NAME`) to those of the lib |
| `toolchain.rs` | finding the compiler (`find_cc_for`: for MSVC, `cl` or `clang-cl` on the PATH, else the `cl` of Visual Studio through `find-msvc-tools`, with the variables of its Developer Prompt in `Tool::env`), the flags of each `Dialect` (GNU or MSVC, as data in `Flags`), the compiles of the `.c` files (started at once) and the link, which takes the other inputs, the runtime lib among them, unchanged; `msvc_link` puts the options of the linker after `/link` |
| `build.rs` | builds `zaka_runtime` as a static lib for the target of the build, and on Windows for the other ABI too (see [The runtime](#the-runtime)) |
| `runtime/` | the hand-written C11 headers: `zaka_target_c_runtime.h`, `zaka_c_sugars.h` and `zaka_lib_glue.h`, the glue of a C lib (its record in the runtime, the start of its globals, the jump of a trap, its reset and its last trap) |

A function or a global goes into the module file of the source file its span starts in.

## Values in C

A scalar is a C scalar: `int32_t`, `double`, `uint32_t` for a character, `uint8_t` for a
boolean, `unsigned char *` for each address, `zk_fn` (`const void *`) for a function value.
A function value is the address of a record whose first word is code, `zk_code`. Each
function has a static record `zk_r<n>` = `{code, NULL}` (declared in `zaka_kinds.h`,
defined in `zaka_kinds.c`), and the address of a function is `&zk_r<n>`; a constant folds to
the index of its function plus one, as in the VM, and the C target writes it as that
address. A closure is a record in a counted heap block: `zk_closure(record, thunk, kind)`
moves the record slot of the IR to a new block, leaves zeros behind, and writes the code of
the thunk into its first word and the descriptor of the record kind into its second, which
is never NULL, so `zk_fn_bound` tests it. A lent closure has the lowest bit of its address
set, which `zk_fn_rec` clears. A call through a value is `zk_fn_bound(f) ? thunk(args…,
zk_fn_rec(f)) : code(args…)`, the code read through `zk_fn_code`. The descriptor walk
(`zk_each_block`) visits a `ZK_TAG_FN` value as it visits text: `zk_fn_dup` counts one more
holder of a record, a drop lets go of one and the last holder frees the record with what
its fields hold, and `zk_fn_lend` marks a loan, which a drop leaves alone. A C argument goes
through `zk_cb_code` when the function kind has a shape (see [C functions](#c-functions)),
and else through `zk_fn_c`, which stops a closure with `ZK_TRAP_CLOSURE_ESCAPE`, since
C calls a bare code address. These functions live in `heap.rs` of the runtime, and the table of names holds code
pointers, so a print finds the name of a closure through its thunk.

A list, a struct, a plain list and text are blocks of octets, laid out as the VM
sets them out: each field at the offset `StructKind::offset` gives, text and plain lists as
three words (the address of the cells, the count, the room). A slot of such a kind is an
aligned run of `unsigned char`, and its value in C is its address, as the VM pushes the
address of an aggregate. A read of a scalar through an address goes through `zk_ld_<N>` and
a write through `zk_st_<N>`, which move the octets with `memcpy`, so an address of each alignment
works. A function takes a list or struct parameter through its address and puts it into a
local of its own, unless the flow checker marks it as one the function never writes; a
list or struct result goes into the slot `zk_ret` the caller gives.

The runtime walks a value through its kind: a drop lets go of each counted block a value
holds, a duplicate counts one more holder of each, a print writes each cell and field. Each kind
it walks gets a `zk_kind` in `zaka_kinds.c`, with its size, its cell, its fields and their
offsets, or its members. The runtime follows the VM op of the same name, with the same
rules: a room of 0 owns no block, a full block grows to twice its room, a list of known
cells is static and never freed.

Each counted block (text, a plain list, a box, a closure record) starts with a head,
`zk_head`, padded so the cells keep the alignment `malloc` gives: the count `rc`, the size,
the kinds of its cells for the ring collector and for the drop, the next block in the
queue of blocks to free, the collector's color and buffer bits, and the pin bit. Each change
of a count goes through `rc_add`, `rc_sub` and `rc_of` (`heap.rs`): plain ops until
`zk_threads_begin` marks the program as one with threads, atomic ops from then on; the
trial counts of the ring collector remain plain, since a collection runs with the other
threads halted. `zk_seq_new`, `zk_seq_grow` and
`zk_seq_free` make, grow and free such blocks; `zk_own` and `zk_unshare` give a block that
another value shares a duplicate of its own before a write; `zk_pin` marks a block so each later
duplicate clones it. A drop queues the blocks it lets go (`zk_freeing`), so a long chain
frees in a loop. `zk_deep_seqs` makes a deep duplicate, duplicating each object once (the
head's next word maps an old block to its new one while the work runs), and
`zk_collect_rings` runs trial deletion when the waiting blocks hold the threshold (16,384
octets) and at the end of the run, as the VM does; a block waits when a block of its kind
can be part of a ring, worked out once for each descriptor. `zk_rings(act, octets)` serves
the statements of the collector: run it now (0), pause (1), resume (2), set the threshold
(3). `zk_ring_stat(n)` gives one field of the stats, as `ring_stat n` of the VM does: the
runs (0), the octets the collections freed (1), the octets waiting (2), the threshold (3), and 1 or 0
for a pause (4); the stats expression lowers to a struct of five `ExprNode::RingStat`
reads. `zk_obj_ptr` and `zk_ptr_obj` turn an object into a pointer and back,
`zk_check_utf8` checks a list of u8s that becomes text. The blocks of the heap statements
of unsafe code remain plain `zk_alloc` blocks with no head, and the two are never mixed. A
counted list moved to the heap (`to_heap`) remains uncounted.

## The order of a run

C leaves open the order of the operands of an operator and the arguments of a call; the
VM runs them left to right. `operands` keeps that order: an operand goes into a local of its
own, on a line before the statement, when a later operand could change it or stop the
program first. The C target writes no statement expressions. The right side of `&&` and
`||` runs just when needed: when it needs lines of its own, those go inside an `if`.

A fresh value that holds text, such as the text a format builds or a call result, is dropped
right after the statement reads it. `ir::fresh_text` holds that rule for both backends.

## The runtime

The runtime is the Rust crate `zaka_runtime`, a static lib (`crate-type = ["staticlib",
"rlib"]`) that both the C target and the [native target](native-backend.md) link. Each of
its functions is `extern "C"`, `#[unsafe(no_mangle)]`, with the name the C calls; its build
script scans `src/*.rs` for them and writes `exports()`, the table the JIT hands to
Cranelift.

| File | Holds |
| --- | --- |
| `program.rs` | `ZkProgram`, the mirror of `zk_program`: the checks, the limits, the rights, the texts, the function names and the sites; `zk_prog`, `zk_depth`, `zk_frames` |
| `start.rs` | `zk_start` and `zk_finish`: the start and the end of a run |
| `traps.rs` | the traps, the stop message and the backtrace |
| `out.rs` | the buffer of standard output, the sink of a format, the print of numbers |
| `heap.rs` | counted blocks, raw blocks, function values and closures, the heap count |
| `seqs.rs` | plain lists, text edits, drops, duplicates, loans, the ring collector |
| `show.rs` | `==`, the print of each kind, results and optionals, ဇယား rows |
| `host.rs` | the host functions of `zaka_host`: standard input, files and streams, the arguments and the environment |
| `rights.rs` | the rights of the run, and the asker on a terminal |
| `callbacks.rs` | the libffi closures of the closures that go to C |
| `ffi.rs`, `native.rs` | the parts the native target calls: `zk_ffi_call`, `zk_cb_shape_of`, `zk_enter_call`, `zk_at_site`, `zk_fmod` |

The C side reads the runtime through three headers:

| File | Holds |
| --- | --- |
| `zaka_runtime.h` | written from the Rust through cbindgen at build time: a declaration of each runtime function, the structs the generated code shares with the runtime (`zk_program`, `zk_kind`, `zk_field`, `zk_site`, `zk_cb_shape`, ...), the ids of the traps, the rights and the kinds (`ZK_TRAP_*`, `ZK_RIGHT_*`, `ZK_TAG_*`), `zk_context` and `zk_ctx()`, the calls in progress of each thread, and `zk_prog` |
| `zaka_target_c_runtime.h` | hand-written: the inline code the generated C runs for each operation, the checked integer operations of each kind (`zk_add_i32`, `zk_div_u64`, ...), the loads and stores and the marks of the calls in progress; it includes `zaka_runtime.h` |
| `zaka_c_sugars.h` | `RET_IF`, `DONE_IF`, `DO_IF`, `NEXT_IF`, `LEAVE_IF`, `DO_FOR` |

`zaka_target_c_runtime.h` includes two headers a build writes, and `zaka_lib_glue.h` a third,
`zaka_lib_names.h`, so an editor reads them with the headers of a build beside them:
`mise run editor:mock-header` writes a build of hello and of a lib into `target/c-headers`,
and `runtime/.clangd` points clangd there.

Each name the runtime exports links as `zk_rt_<version>_<hash>_<name>`: the version of the
crate, and a hash of its sources and those of `zaka_host` (`zaka_runtime/stamp.rs`, which the
build scripts of the runtime and of this crate both include). The Rust writes
`#[unsafe(export_name = stamped!("alloc"))]`; cbindgen reads a duplicate of the sources with a
plain `no_mangle` in its place, and the build script puts a `#define zk_alloc
zk_rt_..._alloc` for each name before the declarations, so the C keeps the plain names.
`RUNTIME_PREFIX` and `linked_name` give the prefix to Cranelift, and the table of exports
of the JIT carries the linked names. A lib `--runtime-lib` gives brings its own prefix:
`driver::prefix_in` reads it off the name of `zk_seq_new` in the lib, and
`runtime_prefix` serves it to `linked_name` and to the header written into the cache. Each
nested build of the runtime sets `RUSTFLAGS=-C metadata=<prefix>`, since Rust derives the
link names of each crate from its metadata, not its sources: two runtimes of other sources
in one program would clash on them. `ZAKA_RUNTIME_SALT` joins the hash, for a test of two
builds.

The runtime keeps a record of each C lib it serves (`zaka_runtime/src/libs.rs`): its table,
the jump back to its wrapper, its gate, its last trap and its heap count. Each wrapper
enters its lib (`zk_lib_enter`, which also pushes a mark of the lends, the sink of the
output and the drop queues) and returns to the lib around it; a trap puts the mark back
(`zk_lib_unwind`). An export that `runs_free` (borrow checker) marks runs with no lock
(`FREE_PARTS` in `exports.rs`): `zk_lib_free_in` counts it in `FREE` once the lib runs
and the counts are atomic, a trap fails the call alone (`zk_lib_fail`), and the ring
collector and the start of the globals wait for `FREE` to drain. The state of a lib
(`zk_lib_state_of`) and the status of a trap (`zk_lib_status`) live in the runtime, so a
call with no lock reads them with no race. Each lib's glue defines `zk_trap_slot_v1` weak (`selectany` on Windows),
so the linker keeps one for the program, and each runtime writes each trap to the slot of
each lib it serves. A counted block records the count it counts in (`Head::owner`), a raw
block of a lib with a cap goes in `RAW_OWNERS`, and a block the lib gives C counts in none.

`zaka_runtime/src/exchange.rs` writes and reads the C form of a value (`zk_to_c`,
`zk_from_c`): a plain list one cell at a time, each in its C form. A block it gives C starts
`C_HEAD` (16) octets after a head that holds the kind of its cells and their count, NULL
for text; `zk_free` frees from the head, and `zk_free_all` walks the cells first
(`free_inside`) and frees each text and list the cells hold. The header of a lib declares each
struct and each plain list in the order in which each holds the others (`CKinds::order`), and names a
list after its cells (`cell_part`). A struct of Zaka itself, whose name starts with `%`
(the one of a ဇယား), has no C form.

`zaka_runtime/cbindgen.toml` sets out `zaka_runtime.h`: the C names of the Rust structs
(`ZkProgram` is `zk_program`), the enum members with the name of their enum in front
(`ZkTrap::ZeroDivisor` is `ZK_TRAP_ZERO_DIVISOR`), and `ZK_NORETURN` for a function that
never returns, an attribute of GCC and Clang. The build script keeps the first paragraph of
each doc comment, so the header carries the gist of each item and the Rust carries the
rest. A function value is `zk_fn` (`ZkFn`, `const void *`) and its code `zk_code` (`ZkCode`,
`void (*)(void)`), so the generated C needs no cast between an object and a function
pointer. The C the compiler writes names each field of a shared struct
(`.tag = ZK_TAG_SINT`), and puts each text of a table of traps or rights under its id
(`[ZK_TRAP_OVERFLOW] = "..."`), so a field or a member that moves in the Rust moves in the C too.

The `build.rs` of the crate runs a cargo of its own: `cargo rustc -p zaka_runtime --lib
--crate-type staticlib --profile zaka-runtime --target <target>`, into a target folder in
`OUT_DIR`, with `--print native-static-libs` for the OS libs the lib needs. The
`zaka-runtime` profile builds for release with `panic = "abort"`. A GNU lib is stripped
(`strip -S`) and `include_bytes!` puts it into `zakac`; `driver::write_cache_for` writes it
into the cache folder of each build, and the link puts it after the objects. A build for
`wasm32` (the explorer) gets blank libs. Beside the lib of the host, `build_other` builds the
libs of other targets at once, each in a thread and a target folder of its own: on
Windows the lib of the other ABI (`other_windows`: `x86_64-pc-windows-gnu` and `-msvc`
swapped), named as that ABI names it (`zaka_runtime.lib` for MSVC, `libzaka_runtime.a`
for GNU), and on each host the libs of `wasm32-wasip1` and `wasm32-wasip2`, which are not
stripped, as the `strip` of the host reads no WebAssembly. When a lib cannot build (the
Rust target or the C tools of that ABI are missing) the build warns and carries a blank
lib, and `write_cache_for` refuses that ABI (`RuntimeNotCarried`).

### WASI

`--target-os wasi` keeps the C of each other host, with three changes in `wasi.rs` and
one in `main.c`:

- `wasi::find_cc` takes `--cc`, then `bin/<triple>-clang` of `WASI_SDK_PATH`, then
  `<triple>-clang` on the PATH; the Clang of wasi-sdk named for a triple builds for it
  with the sysroot of the SDK. `CC` is skipped, as it names a compiler of the host.
- `wasi::link_flags` adds a stack of 8 MiB (the linker gives 64 KiB), and for `wasip2`
  `--export=cabi_realloc`: the std of the runtime lib defines that function, which the
  component needs, and the linker leaves it unexported unless asked.
- `cli::other_host` lets a `wasi` target with the CPU `wasm32` and a WASI ABI through,
  and a build with C functions stops with `CFunctionsOnWasi`. The runtime leaves out
  its modules `ffi` and `callbacks` on WASI, with libffi.
- `main.c` starts with `#if defined(__wasi__)`: WASI starts a `main` of two
  parameters alone, so that `main` reads the environment from `environ`, where the other
  hosts take a third parameter, `envp`.

`zaka_checks.h` defines one `ZK_CHECK_<NAME>` as 1 or 0 for each run-time check
(`RunCheck` in `options.rs`: `DIVIDE`, `SHIFT`, `BOUNDS`, `TEXT`, `STEP`, `DEPTH`, `NULL`,
`OVERLAP`, `SANITIZERS`), `ZK_MAX_DEPTH` and `ZK_DEBUG_INFO` (1 in a debug build, 0 under
`--release`), for the inline code of `zaka_target_c_runtime.h` and the program. The runtime reads the
same values from the program table: `main` calls `zk_start(argc, argv, &zk_the_program)`,
and `zk_the_program` in `zaka_kinds.c` holds the check bits (`RunChecks::bits`), the
limits (`--max-depth`, `--max-heap`, 0 for no cap), the texts of the traps from the locale of
the compiler, the rights of the build, the names of the functions and the sites. So the
lib is the same for each program and each set of checks. `RunChecks` holds the set as bits;
`--checks` builds it from a level (`Checks`), then each `+name` or `-name`.
`ZK_CHECK_SANITIZERS` changes no C: the build adds the sanitizer flags when it is on.

A float prints from its bits (`zk_put_fbits(bits, raw)`, where `raw` is the 32- or 64-bit
pattern), through the number text of `zaka_host::numbers`, which the VM prints with, so
each host prints the same digits.

An optional and a result are the same struct as in the VM (a tag, then `some`, or `ok` and
`err`). `Stmt::Halt` becomes a call that stops the run: `zk_not_good(at, kind)`, which shows the error value, and
`zk_not_good_bare()` for `not_good`, `zk_not_fault()` for `not_fault`, `zk_closed_handle()` for a
closed handle, `zk_plain_trap(ZK_TRAP_NULL_FROM_C)` for `null_from_c` (C gave NULL for a plain handle result) and `zk_no_data()` for `no_data`, which stops with `ZK_TRAP_NO_DATA` and the text of the
trap `the optional holds no data` from the locale (`target_c.trap.no_data`); `stmts.rs` writes each.
A print of an optional goes through `zk_push_fallible`, which writes `ဒေတာ(<value>)` for tag 1 and
`အလွတ်` for tag 0, with the words `zk_data_text` and `zk_no_data_text` that `emit.rs` puts in
`zaka_kinds.c`; the null pointer prints as `zk_null_text`, the phrase `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`, as in the VM; a result writes `ရလဒ်ကောင်း(...)` or `အမှားအယွင်း(...)`, as in the VM.

Each thread keeps its calls in progress in a `zk_context` of its own (`depth`, `frames`,
`room`), which `zk_ctx()` gives and makes the frame table of on first use. `ZK_ENTER(f)` opens
each function and keeps the context in the local `zk_c`; `main` and `zk_lib_init` take it
through `ZK_HERE()`. `ZK_LEAVE()` closes each return: with `ZK_CHECK_DEPTH` on, the pair counts
`zk_c->depth` and calls `zk_too_deep` past `ZK_MAX_DEPTH` (`--max-depth`).
Frames are kept when the depth check is on or `ZK_DEBUG_INFO` is 1, so `--checks none` still
names the place in a debug build: the pair then pushes and pops a row of the frame table
`zk_c->frames` (one row for each call in progress: the function number and the site). `ZK_AT(site)`
comes before each statement whose site differs from the last one, and sets the site of the top
row. The sites (file, line, column of the start of a statement) are tables in
`zaka_kinds.c` that the program table points at; a `--release` build writes neither the
tables nor the marks. A stop writes the function and the place, then the backtrace from the frames of its thread, with the
rules of the VM (`ZAKA_BACKTRACE`: unset folds a run of one call to `... the call above, N
times more` and past 16 lines to `... N calls more`; `full` lists each call; `0` none; the C
`main` is left out). With the checks of depth and debug info both off, the pair counts nothing.

`--max-heap` counts in the runtime: a counted block adds the size of its cells when it is made,
and a raw block (`zk_alloc`, `zk_realloc`, `zk_raw_free`) adds the size the allocator gives
(`malloc_size` on macOS, `malloc_usable_size` on Linux, `_msize` on Windows; another host counts
no raw block). A counted block past the cap stops the run with the trap `no heap room for N
octets`; a raw allocation past it gives NULL; a resize past it stops the run. A block that C
made and the program frees was never counted, so the count stops at 0; a block given to C that
C frees is never given back. An else-if chain of the IR is written flat as
`else if`, not as nested blocks, so a chain of thousands of branches builds with each C
compiler.

The headers are strict C11 and pass `-pedantic-errors`. The GCC and Clang builtin for an
overflow test stands behind `__GNUC__`, with a C11 path beside it.

### ဇယား values

`zk_row_find`, `zk_set_row` and `zk_remove_row` in `show.rs` hold the same table the VM
holds: rows in add order, slots of `uint32_t` in a power of two, a rebuild when a third of the
slots would no longer be free, FNV-1a over the parts of an index. A ဇယား keeps the struct tag
in its kind, and `zk_is_table` knows it through its name, `%map`, so the walks that drop, duplicate
and own a struct need no change; `zk_eq` and `zk_show` call the ဇယား steps first. The trap of a
missing row is `ZK_TRAP_NO_ROW`.

## Jobs and channels

- `jobs.rs` writes `zaka_jobs.c` for a program that starts a job: `zk_job_guard`, which
  runs a step under `setjmp` and keeps the jump through `zk_lib_jump_set`, the thread's
  own; `zk_job_leave`, which jumps back; and `zk_job_step_<f>` for each function a job
  runs, which calls it with the arguments of the data block and writes the result at its
  start. The function drops its parameters as it returns, so the step drops none.
- A spawn takes a data block (`zk_job_data`), moves each argument into it at a 16-octet
  part (`part_offsets`; the arguments come owned, as the value of a set does, since calls alone
  lend), and calls `zk_job_start`. A wait copies the result out and shares what it holds
  (`zk_dup_seqs`); a receive writes the optional into a temp; a send moves its value
  through its address.
- The runtime (`zaka_runtime/src/jobs.rs`) keeps the jobs and channels under one lock and
  one condition. It counts the threads that run, the main one (`mark_main`, from
  `zk_start`) and each job: a wait that would leave none running is the trap `deadlock`,
  unless a thread out of the count can still act (`outside_acts`): each thread of a C
  lib, a call from a thread of C that runs now (`free_calls`), or a closure C holds that
  it can call from a thread of its own (`lent_to_threads`). A thread of C waits and
  changes no count. `zk_job_start` turns counts atomic (`zk_threads_begin`) and starts
  a thread of 64 MiB; under `--trap unwind`, `stop` writes `job <n> stopped: ...` and
  calls the leave of the thread's job. `zk_finish` waits for each job, drops each result
  and each value a channel holds, and makes the status 1 when a trap no wait saw ended a
  job. `collect_rings` waits while a job runs; the lists of the deep duplicate are each
  thread's own.
- `zk_lock` and `zk_unlock` find each lock through the block of the object (`zk_seq_base` of
  the variable) and name the holder through `owner`: the number of a job, or a number of
  its own for each other thread, from the high half (`NEXT_LOCKER`), so two threads of C
  never pass as one. A lock another thread holds waits through the same deadlock check, a
  thread takes again a lock it holds, and a job that stops at a trap leaves its locks
  broken (`LockBroken`).

## C functions

`emit` writes `zaka_ffi.c` when `Module::foreign` has rows. The file includes
`<stddef.h>` and `<stdint.h>` and no libc header, so a declaration of a C function in it,
with the kinds Zaka gives it (`_Bool`, `void *` for each address), cannot clash with the
one a header holds; pragmas turn off `-Wincompatible-library-redeclaration` (Clang) and
`-Wbuiltin-declaration-mismatch` (GCC). For each struct a call passes, it defines
`zk_ffi_s<n>` with a `_Static_assert` on its size, and for each C function a wrapper
`zk_ffi_<n>` (`wrapper_name`, `wrapper_proto`) whose declaration stands in `zaka_kinds.h`.
A handle of a kind from outside crosses as `void *`. The wrapper itself is no more than the
call: the IR guards each handle (`Stmt::Halt` with the cause `ClosedHandle`, which
`stmts.rs` writes as `zk_closed_handle()`, a runtime function that stops the run with
`ZK_TRAP_CLOSED_HANDLE`), wraps a result with the box of the handle after a test that stops
the run on NULL for a result written with no optional (`Halt` with the cause `NullFromC`),
and empties the box
after a take over. A field read, `ExprNode::CField`, is `exprs.rs` `c_field`: a `memcpy`
from the address of the C struct plus the offset, into a temp of the kind of the field (a
`char *` field goes through `zk_text_from_c`, which gives the optional with no data for NULL).
A wrapper stands for a shape of call, `FfiCall { func, extra }`: the C function and the
kinds of its other values. `exprs.rs` (`ffi_call`) numbers the shapes of the module in
`Shared::ffi_calls`, so a C function that takes other values (`ForeignFn::variadic`, declared
with `, ...`) has a wrapper for each shape of its calls, and the parameters of a wrapper
are the named ones and then the other values. A struct goes in and out of a wrapper as the
address of its octets, and a struct result goes to the address of the first parameter,
`zk_out`. A text goes in as the address of its octets and its count; the wrapper makes
a duplicate that ends in a zero octet with `zk_text_cstr` (`seqs.rs`, declared in
`zaka_target_c_runtime.h` and in the core of `zaka_ffi.c`), and frees it with `zk_text_cstr_free`
after the call. The module
files call the wrappers through `ExprNode::Foreign` and `Stmt::Foreign`. A list of numbers
comes as the address of its first cell, which the IR built before the call, so the wrapper
passes a pointer as for each other address.

A result of the kind `စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` (`is_c_text_result`)
is held as `const char *` and given to `zk_text_from_c(out, size, ok_at, s)`
(`host.rs`): NULL writes the optional with no data, else the string is copied into
new text of the program (`zk_text_of_c`), with `bad_utf8` for octets that are not UTF-8. The wrapper calls it before it
frees the duplicates of the text arguments, since `strstr` and `strchr` point into them.
`ffi_source` defines the structs of `FfiCall::extra` too, so a struct can be an other value.
A parameter of a function kind that `is_callback` takes is declared with the C pointer
kind of its signature (`void *` for each address), and the wrapper passes the code
`zk_cb_code` gives (below) cast to it; a function value holds a function at each point, so
C never gets NULL there. A trap in a callback is the trap of the run: the program stops
where it happens.

Function values go to C through `zaka_callback.c`, which `emit` writes when the program
gives function values to C or holds a leave statement (`Stmt::Leave`). The file
starts with `zaka_runtime.h` (the runtime keeps
`zk_cb_code`, `zk_cb_lent`, `zk_cb_end`, `zk_cb_leave` and `zk_cb_revoke` in `callbacks.rs`,
with `zk_fn_size` and `zk_fn_release` in `heap.rs`), and for each
function kind that goes to C a bridge and a `zk_cb_native_shape` named `zk_cbs_<n>`: the C
kinds of its result and parameters (`zk_c_kind`), as the native backend writes them. A
wrapper passes `zk_cb_code(value, zk_cb_shape_of(&zk_cbs_<n>))` for such a parameter
(`shape_ref`), and wraps the C call in `zk_cb_lent()` and `zk_cb_end()`, so the closure is
lent to the call; a leave statement is `zk_cb_leave(value, zk_cb_shape_of(&shape))` or
`zk_cb_revoke(value)`. `zk_cb_shape_of` makes the libffi kinds of a shape on its first use
and keeps them for the run. Each closure gets one libffi
closure, found again from the octets of its record, and its code is never freed, so a late
call from C stops the run with `ZK_TRAP_CLOSURE_ESCAPE` and never runs freed code; a call
from a thread of C's own runs when the table of names allows it: the row of the function,
or of the thunk of a closure, in `fn_names` holds `crosses`. It is set for a function
that, itself or through each function it calls, uses no mutable global, and for a thunk
each capture of its record is a duplicate of a value with no box, function value, pointer or
handle, save an object of a struct marked for jobs (`Module::jobs_structs`), whose fields
each thread reaches under its lock (`crossing_fns` in the borrow checker). A write to a
field of an object leaves the variable unwritten (`note_written` in the check), so such a
capture is a duplicate. The runtime finds the row through the
code word of the record. `run_on_c_thread` takes a count of a closure's record while the
lend or the leave holds (`LENDING`, a lock the lends, the leaves and their ends take
too), counts the call in `FREE`, so the ring collector waits, and the lend turned the
counts atomic before; a call after the lend with no leave stops with
`ZK_TRAP_CLOSURE_ESCAPE`. Each other call from another thread aborts a program
with a message. In a C lib, `run_in_lib` takes the call
instead: on a thread with calls parked in C it takes them back, runs the bridge and lets go
again; on another thread it runs the bridge through the guard of the lib (`zk_lib_guard`
of the glue, which `zk_lib_open` takes), whose `setjmp` catches a trap, and C gets zeros. Each
function C calls goes through the bridge, a plain function too, so no call skips the lock
of a lib or the thread rules of a program.

A call to C in a lib lets go of the lock (`ZK_LIB_LET_GO` in each wrapper of
`zaka_ffi.c`, which `ZK_LIB` of `zaka_lib_names.h` turns on, and `zk_ffi_call` for
Cranelift code): `zk_lib_let_go` writes out the output, keeps the count of its holds in a
thread-local list (`PARKED`) and lets go of the lock at each depth it held; the rest of
the state of a call (the lib, its base, the table, the marks, the lends, the jump target,
the sink and the drop queue) is each thread's own. `zk_lib_take_back` waits for the lock
and takes it back at that depth. The jump
target of a trap lives in the runtime (`zk_lib_jump`, `zk_lib_jump_set`), not in the
glue, since a thread that waits in C lets others into the same lib.

A function kind whose parameters each cross as written (`as_is`: numbers, booleans and
addresses) lets a plain function reach C as its own code. Each other kind a C declaration
takes (`callback_param`) goes through the bridge, for a plain function too (`zk_cb_of` keeps
one libffi closure for it, `kept` for the run): `crossing` picks how each parameter crosses.
A struct passed as a whole value gets a C kind with the tag 12 (`zk_cbst_<i>`, its members in
`zk_cbse_<i>`, a counted list as one element for each cell) and is copied into an aligned
block; text comes from `zk_text_of_c` (NULL is `ZK_TRAP_NULL_FROM_C`); an optional of text from
`zk_text_from_c`; a handle from `zk_cb_handle`, a new box around the address (NULL gives a box
holding 0); an optional of a handle from `zk_cb_opt_handle`. The function gets a lent
duplicate (`zk_lend_seqs`, room 0), and the bridge drops the original with `zk_drop` once it
returns. `zk_fn_c` remains the guard of a program with no `zaka_callback.c`. A leave of a kind the bridge has no shape for (a parameter `callback_param`
refuses, or a result that is no number, boolean or address) is a construct the target
cannot build. The runtime lib carries libffi itself (the `libffi` crate builds it), and the
bridges name no libffi kind: no build needs the host's libffi, its header or `-lffi`. A
bridge writes a narrow integer result as a whole word (`intptr_t` or `uintptr_t`, the width
of libffi's `ffi_arg`).

`link_libs` gives `-lX` for each lib `libX` the program names, but libc and libm, in
order of first use. `BuildPlan::libs` puts them after `-lm` and before the `LDFLAGS` and
`--ld-flag` flags of the user, which is where a `-L<folder>` flag still reaches them.
`lib_flags` gives a lib path (`is_lib_path`) to the linker as it is, and, on each host but
Windows, `-Wl,-rpath,<folder>` after it. The loader made a relative path absolute from the
folder of its file, so the flags hold an absolute path. On macOS the lib needs an install
name that is an absolute path or `@rpath/...`.

## Tests

| Test | Checks |
| --- | --- |
| `tests/samples.rs` | each repo sample prints, as an executable, the output the VM prints for it, at `--checks default` and `--checks none`, under `--overflow stop` and `wrap`; it passes with a note on a host with no C compiler, unless `ZAKA_REQUIRE_CC=1` makes that a failure, as on CI |
| `tests/ffi_lib.rs` | C functions, structs as values, other values, handles and closures given to C, against a C lib the test builds |
| `tests/text.rs` | the runtime writes thousands of floats and the ends of 64-bit integers as the VM writes them |
| `tests/traps.rs` | a zero divisor and an overflow stop with the VM's message and status 1 |
| `tests/forms.rs` | a C compiler sets out each struct of `zaka_runtime.h` as rustc does (each `sizeof` and `offsetof` against `size_of` and `offset_of!`), each struct and field of the header has a row in the test, and the trap texts, the tags and the rights the compiler lists follow the enums of the runtime |
| `zaka_website/tests/stops_in_c.rs` | each `zaka-stop` program of the books, built with this target, prints the output and the stop reason of the VM (see [Build, test and format](README.md#build-test-and-format)); a stop here names no source line and no calls in progress, which the VM names |

Each test builds and runs its program in a folder of its own, so a sample that writes a
file writes it there.
