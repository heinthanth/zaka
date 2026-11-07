---
title: The C target
description: How zakac_codegen_target_c turns the IR into C, the runtime it writes beside the C, and the tests that hold it to the VM.
---

# The C target

`zakac_codegen_target_c` is the second backend. It reads the IR that the borrow checker
leaves, the one `zakac_codegen_target_vm` reads, and writes C11. `zakac build --target c`
runs it, then the C compiler. [Native programs](../compiler/native.md) tells what a user
sees; this page tells how it is built.

```text
ir::Module ──emit──▶ m0.c … mN.c, main.c, zaka_kinds.{h,c}
                         + zaka_rt.h, zaka_rt*.c, zaka_c_sugars.h
                         │
                         └──cc -c, then link──▶ executable
```

## The crate

| File | Job |
| --- | --- |
| `cli.rs` | the standalone command; `zakac build` calls its `command_main` for `--target c` |
| `driver.rs` | the phases, then `emit`; the cache folder; the runtime files, embedded with `include_str!` |
| `emit.rs` | the files of a program: a module file for each source file, the shared header, `main.c` |
| `stmts.rs`, `exprs.rs` | a statement or an expression of the IR as C |
| `kinds.rs` | the C kind of each scalar kind, and constants as C text |
| `descriptors.rs` | the description of each kind the runtime walks: `zk_kind` in `zaka_kinds.c` |
| `toolchain.rs` | finding the compiler, the flags, the compile and the link |
| `runtime/` | the runtime: hand-written C11 |

A function or a global goes into the module file of the source file its span starts in.

## Values in C

A scalar is a C scalar: `int32_t`, `double`, `uint32_t` for a character, `uint8_t` for a
boolean, `unsigned char *` for each address, `zk_fn` for a function value. A function value
folds to the index of its function plus one, as in the VM; the C target writes it as the
address of that function.

A list, a struct, a slice, a plain list and text are blocks of octets, laid out as the VM
sets them out: each field at the offset `StructKind::offset` gives, text and plain lists as
three words (the address of the cells, the count, the room). A slot of such a kind is an
aligned run of `unsigned char`, and its value in C is its address, as the VM pushes the
address of an aggregate. A read of a scalar through an address goes through `zk_ld_<N>` and
a write through `zk_st_<N>`, which move the octets with `memcpy`, so an address of each alignment
works. A function takes a list or struct parameter through its address and puts it into a
local of its own, unless the borrow checker marks it as one the function never writes; a
list or struct result goes into the slot `zk_ret` the caller gives.

The runtime walks a value through its kind: a drop frees each text and plain list a value owns,
a duplicate gives each one a block of its own, a print writes each cell and field. Each kind
it walks gets a `zk_kind` in `zaka_kinds.c`, with its size, its cell, its fields and their
offsets, or its members. The runtime follows the VM op of the same name, with the same
rules: a room of 0 owns no block, a full block grows to twice its room, a list of known
cells is static and never freed.

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
| `zaka_rt.c` | the start and the end of a run, the traps, the print of numbers |
| `zaka_rt_data.c` | the heap, plain lists, text edits, slices, `==`, the print of each kind |
| `zaka_rt_host.c` | standard input, files and output streams, and the arguments and the environment |
| `zaka_c_sugars.h` | `RET_IF`, `DONE_IF`, `DO_IF`, `NEXT_IF`, `LEAVE_IF`, `DO_FOR` |

`zaka_kinds.h` defines `ZK_CHECKS_NONE`, `ZK_CHECKS_DEFAULT` or `ZK_CHECKS_MEM` before it
includes `zaka_rt.h`, so the C of a program is the same at each level and a level is a define
and a few flags. The texts of the traps come from the locale of the compiler, into
`zaka_kinds.c`. A float prints with the fewest digits that read back, found with `%.*e` and
`strtod`. When the float lies at the midpoint of two such digit strings, printf keeps the
even one and Rust the one above, so the runtime turns a tie that printf rounded down into
the one above.

The runtime is strict C11 and passes `-pedantic-errors`. 128-bit integers stand behind
`__SIZEOF_INT128__`, and the GCC and Clang builtin for an overflow test behind `__GNUC__`,
each with a C11 path beside it.

## Tests

| Test | Checks |
| --- | --- |
| `tests/samples.rs` | each repo sample prints, as an executable, the output the VM prints for it, at `--checks default` and `--checks none`, under `--overflow stop` and `wrap`; it passes with a note on a host with no C compiler |
| `tests/text.rs` | the runtime writes thousands of floats and the ends of 128-bit integers as the VM writes them |
| `tests/traps.rs` | a zero divisor and an overflow stop with the VM's message and status 1 |

Each test builds and runs its program in a folder of its own, so a sample that writes a
file writes it there.
