---
title: The C target
description: How zakac_codegen_target_c turns the IR into C, the runtime it writes beside the C, and the tests that hold it to the VM.
---

# The C target

`zakac_codegen_target_c` is the second backend. It reads the IR that the flow checker
leaves, the one `zakac_codegen_target_vm` reads, and writes C11. `zakac build --target c`
runs it, then the C compiler. [Native programs](../compiler/native.md) tells what a user
sees; this page tells how it is built.

```text
ir::Module ──emit──▶ m0.c … mN.c, main.c, zaka_checks.h, zaka_kinds.{h,c}, zaka_ffi.c (with C functions), zaka_cb.c (with closures given to C)
                         + zaka_rt.h, zaka_rt*.c, zaka_c_sugars.h
                         │
                         └──cc -c, then link──▶ executable
```

## The crate

| File | Job |
| --- | --- |
| `cli.rs` | the standalone command; `zakac build` calls its `command_main` for `--target c` |
| `driver.rs` | the phases, then `emit`; the cache folder; the runtime files, embedded with `include_str!`; `translate` takes a `TargetCOptions` (`checks`, `max_depth`, `debug_info`, `max_heap`) |
| `emit.rs` | the files of a program: a module file for each source file, the shared header, `main.c` |
| `stmts.rs`, `exprs.rs` | a statement or an expression of the IR as C |
| `kinds.rs` | the C kind of each scalar kind, and constants as C text |
| `descriptors.rs` | the description of each kind the runtime walks: `zk_kind` in `zaka_kinds.c` |
| `ffi.rs` | `zaka_ffi.c`: the structs, declarations and wrappers of the C functions; the `-lX` flags of their libs |
| `callbacks.rs` | `zaka_cb.c`: the runtime `zaka_rt_cb.c`, then a bridge and a `zk_cb_shape` (`zk_cbs_<n>`) for each function kind that goes to C; the `-lffi` flag |
| `toolchain.rs` | finding the compiler, the flags, the compile and the link |
| `runtime/` | the runtime: hand-written C11 |

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
(`zk_each_block`) visits a `ZK_K_FN` value as it visits text: `zk_fn_dup` counts one more
holder of a record, a drop lets go of one and the last holder frees the record with what
its fields hold, and `zk_fn_lend` marks a loan, which a drop leaves alone. A C argument goes
through `zk_cb_code` when the function kind has a shape (see [C functions](#c-functions)),
and else through `zk_fn_c`, which stops a closure with `ZK_TRAP_CLOSURE_ESCAPE`, since
C calls a bare code address. These functions live in `zaka_rt_data.c`, and the table of names holds code
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
queue of blocks to free, the collector's color and buffer bits, and the pin bit. `zk_seq_new`, `zk_seq_grow` and
`zk_seq_free` make, grow and free such blocks; `zk_own` and `zk_unshare` give a block that
another value shares a duplicate of its own before a write; `zk_pin` marks a block so each later
duplicate clones it. A drop queues the blocks it lets go (`zk_freeing`), so a long chain
frees in a loop. `zk_deep_seqs` makes a deep duplicate, duplicating each object once (the
head's next word maps an old block to its new one while the work runs), and
`zk_collect_rings` runs trial deletion when 1,024 blocks wait and at the end of the run,
as the VM does. `zk_obj_ptr` and `zk_ptr_obj` turn an object into a pointer and back,
`zk_check_utf8` checks a list of u8s that becomes text. The blocks of the heap statements
of marked code remain plain `zk_alloc` blocks with no head, and the two are never mixed. A
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

| File | Holds |
| --- | --- |
| `zaka_rt.h` | the kinds, the descriptors, the trap ids, the checked integer operations of each kind (`zk_add_i32`, `zk_div_u64`, ...), the loads and stores |
| `zaka_rt.c` | the start and the end of a run, the traps, the depth count, the print of numbers |
| `zaka_rt_data.c` | the heap, counted blocks and the ring collector, plain lists, text edits, deep duplicates, `==`, the print of each kind |
| `zaka_rt_host.c` | standard input, files and output streams, and the arguments and the environment |
| `zaka_c_sugars.h` | `RET_IF`, `DONE_IF`, `DO_IF`, `NEXT_IF`, `LEAVE_IF`, `DO_FOR` |

`zaka_checks.h` defines one `ZK_CHECK_<NAME>` as 1 or 0 for each run-time check
(`RunCheck` in `options.rs`: `DIVIDE`, `SHIFT`, `BOUNDS`, `TEXT`, `STEP`, `DEPTH`, `NULL`,
`OVERLAP`, `SANITIZERS`), `ZK_MAX_DEPTH`, `ZK_MAX_HEAP` (`--max-heap`, as `<n>u`, 0 for no cap) and `ZK_DEBUG_INFO` (1 in a debug build, 0 under `--release`); `zaka_rt.h` includes it, so each file of the
runtime (`zaka_rt.c`, `zaka_rt_data.c`, `zaka_rt_host.c`) and of the program sees the same
checks, the C of a program is the same under each set of checks, and a set is a few
defines and a few flags. `RunChecks` holds the set as bits; `--checks` builds it from a
level (`Checks`), then each `+name` or `-name`. `ZK_CHECK_SANITIZERS` changes no C: the
build adds the sanitizer flags when it is on. The texts of the traps come from the locale of the compiler, into
`zaka_kinds.c`.

`zaka_rt_float.{h,c}` hold the float kinds past C11 and the print of each float kind.
`zk_half` is `_Float16` where the compiler defines `__FLT16_MANT_DIG__`; `zk_quad` is
`_Float128` (`__FLT128_MANT_DIG__`), a `long double` of 113 bits, or `__float128`
(`__SIZEOF_FLOAT128__`). Each sets `ZK_HAS_F16` or `ZK_HAS_F128`, and `zaka_kinds.h` stops
at an `#error` that names the kind when the program holds one the compiler lacks, as it does
for `i128`. A constant of either kind comes from its bits (`zk_f16_of`, `zk_f128_of`), since
C11 has no literal for it; an `f16` operation is cast back to `zk_half`, so it rounds as the
VM's does; `%` calls `zk_fmod_half` or `zk_fmod_quad` (`fmodl` or `fmodf128`), and an `f128`
converts to an integer through `zk_q2i_<N>`, not through `double`.

A float prints from its bits (`zk_put_fbits`), with natural numbers of the runtime's own and
no print from libc, so each host prints the same digits. `f32` and `f64` print the fewest
digits that read back, as Rust's `{:?}` does: Dragon4 as Rust's `flt2dec` runs it, with the
same gaps to the neighbours (half the gap below at a power of two) and a tie rounded up.
`f16` and `f128` print as `rustc_apfloat` writes them, as the VM does: the exact value cut to
`2 + P * 59 / 196` digits (5 for `f16`, 36 for `f128`), a half rounded up.

An optional and a result are the same struct as in the VM (a tag, then `some`, or `ok` and
`err`). `Stmt::Halt` becomes a call that stops the run: `zk_not_good(at, kind)`, which shows the error value, and
`zk_not_good_bare()` for `not_good`, `zk_not_fault()` for `not_fault`, `zk_closed_handle()` for a
closed handle, `zk_plain_trap(ZK_TRAP_NULL_FROM_C)` for `null_from_c` (C gave NULL for a plain handle result) and `zk_no_data()` for `no_data`, which stops with `ZK_TRAP_NO_DATA` and the text of the
trap `the optional holds no data` from the locale (`target_c.trap.no_data`); `stmts.rs` writes each.
A print of an optional goes through `zk_push_fallible`, which writes `ဒေတာ(<value>)` for tag 1 and
`အလွတ်` for tag 0, with the words `zk_data_text` and `zk_no_data_text` that `emit.rs` puts in
`zaka_kinds.c`; the null pointer prints as `zk_null_text`, the phrase `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`, as in the VM; a result writes `ရလဒ်ကောင်း(...)` or `အမှားအယွင်း(...)`, as in the VM.

`ZK_ENTER(f)` opens each function and `ZK_LEAVE()` closes each return: with `ZK_CHECK_DEPTH`
on, the pair counts `zk_depth` and calls `zk_too_deep` past `ZK_MAX_DEPTH` (`--max-depth`).
Frames are kept when the depth check is on or `ZK_DEBUG_INFO` is 1, so `--checks none` still
names the place in a debug build: the pair then pushes and pops a row of the frame table
`zk_frames` (one row for each call in progress: the function number and the site). `ZK_AT(site)`
comes before each statement whose site differs from the last one, and sets the site of the top
row. The sites (file, line, column of the start of a statement) are the tables `zk_sites` and
`zk_site_files` in `zaka_kinds.c`; a `--release` build writes neither the tables nor the
marks. A stop writes the function and the place, then the backtrace from `zk_frames`, with the
rules of the VM (`ZAKA_BACKTRACE`: unset folds a run of one call to `... the call above, N
times more` and past 16 lines to `... N calls more`; `full` lists each call; `0` none; the C
`main` is left out). With the checks of depth and debug info both off, the pair counts nothing.

`--max-heap` counts in the runtime: a counted block adds the size of its cells when it is made,
and a raw block (`zk_alloc`, `zk_realloc`, `zk_free`) adds the size the allocator gives
(`malloc_size` on macOS, `malloc_usable_size` on Linux, `_msize` on Windows; another host counts
no raw block). A counted block past the cap stops the run with the trap `no heap room for N
octets`; a raw allocation past it gives NULL; a resize past it stops the run. A block that C
made and the program frees was never counted, so the count stops at 0; a block given to C that
C frees is never given back. An else-if chain of the IR is written flat as
`else if`, not as nested blocks, so a chain of thousands of branches builds with each C
compiler.

The runtime is strict C11 and passes `-pedantic-errors`. 128-bit integers stand behind
`__SIZEOF_INT128__`, and the GCC and Clang builtin for an overflow test behind `__GNUC__`,
each with a C11 path beside it.

### ဇယား values

`zk_row_find`, `zk_set_row` and `zk_remove_row` in `zaka_rt_data.c` hold the same table the VM
holds: rows in add order, slots of `uint32_t` in a power of two, a rebuild when a third of the
slots would no longer be free, FNV-1a over the parts of an index. A ဇယား keeps the struct tag
in its kind, and `zk_is_table` knows it through its name, `%map`, so the walks that drop, duplicate
and own a struct need no change; `zk_eq` and `zk_show` call the ဇယား steps first. The trap of a
missing row is `ZK_TRAP_NO_ROW`.

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
a duplicate that ends in a zero octet with `zk_text_cstr` (`zaka_rt_data.c`, declared in
`zaka_rt.h` and in the prelude of `zaka_ffi.c`), and frees it with `zk_text_cstr_free`
after the call; `zk_open` of the host runtime uses the same function for a path. The module
files call the wrappers through `ExprNode::Foreign` and `Stmt::Foreign`. A list of numbers
comes as the address of its first cell, which the IR built before the call, so the wrapper
passes a pointer as for each other address.

A result of the kind `စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` (`is_c_text_result`)
is held as `const char *` and given to `zk_text_from_c(out, size, ok_at, s)`
(`zaka_rt_host.c`): NULL writes the optional with no data, else the string is copied into
new text of the program (`zk_text_of_c`), with `bad_utf8` for octets that are not UTF-8. The wrapper calls it before it
frees the duplicates of the text arguments, since `strstr` and `strchr` point into them.
`ffi_source` defines the structs of `FfiCall::extra` too, so a struct can be an other value.
A parameter of a function kind that `is_callback` takes is declared with the C pointer
kind of its signature (`void *` for each address), and the wrapper passes `zk_fn_code(value)`
cast to it; a function value holds a function at each point, so C never gets NULL there. The C signature of a Zaka function that such a kind can hold
matches the C one, so C can call it as it calls a C function. A trap in a callback is the trap of the run: the
program stops where it happens.

A closure goes to C through `zaka_cb.c`, which `emit` writes when the program makes a
closure and gives function values to C or holds a leave statement (`Stmt::Leave`). The file
holds the runtime `zaka_rt_cb.c` (`zk_cb_code`, `zk_cb_lent`, `zk_cb_end`, `zk_cb_leave`,
`zk_cb_revoke`, with `zk_fn_size` and `zk_fn_release` in `zaka_rt_data.c`), and for each
function kind that goes to C a bridge and a `zk_cb_shape` named `zk_cbs_<n>`. A wrapper
passes `zk_cb_code(value, &zk_cbs_<n>)` for such a parameter, and wraps the C call in
`zk_cb_lent()` and `zk_cb_end()`, so the closure is lent to the call; a leave statement is
`zk_cb_leave(value, &shape)` or `zk_cb_revoke(value)`. Each closure gets one libffi
closure, found again from the octets of its record, and its code is never freed, so a late
call from C stops the run with `ZK_TRAP_CLOSURE_ESCAPE` and never runs freed code; a call
on another thread aborts with a message.

A function kind whose parameters each cross as written (`as_is`: numbers, booleans and
addresses) lets a plain function reach C as its own code. Each other kind a C declaration
takes (`callback_param`) goes through the bridge, for a plain function too (`zk_cb_of` keeps
one libffi closure for it, `kept` for the run): `crossing` picks how each parameter crosses.
A struct passed as a whole value gets a libffi struct kind (`zk_cbst_<i>`, its elements in
`zk_cbse_<i>`, a counted list as one element for each cell) and is copied into an aligned
block; text comes from `zk_text_of_c` (NULL is `ZK_TRAP_NULL_FROM_C`); an optional of text from
`zk_text_from_c`; a handle from `zk_cb_handle`, a new box around the address (NULL gives a box
holding 0); an optional of a handle from `zk_cb_opt_handle`. The function gets a lent
duplicate (`zk_lend_seqs`, room 0), and the bridge drops the original with `zk_drop` once it
returns. `zk_fn_c` remains the guard of a program with no `zaka_cb.c`. A leave of a kind the bridge has no shape for (a parameter `callback_param`
refuses, or a result that is no number, boolean or address) is a construct the target
cannot build. `link_command` adds `-lffi` when
`zaka_cb.c` is written, so a program with no such closure never needs libffi.

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
| `tests/text.rs` | the runtime writes thousands of floats and the ends of 128-bit integers as the VM writes them |
| `tests/traps.rs` | a zero divisor and an overflow stop with the VM's message and status 1 |
| `zaka_website/tests/stops_in_c.rs` | each `zaka-stop` program of the books, built with this target, prints the output and the stop reason of the VM (see [Build, test and format](README.md#build-test-and-format)); a stop here names no source line and no calls in progress, which the VM names |

Each test builds and runs its program in a folder of its own, so a sample that writes a
file writes it there.
