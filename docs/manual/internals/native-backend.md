---
title: The native target
description: How zakac_codegen_target_native lowers the IR to Cranelift IR, runs it at once in zakac or links it into an executable, and the tests that keep it in step with the VM.
---

# The native target

`zakac_codegen_target_native` is the third backend. It reads the IR that the flow checker
(borrow checker) leaves, as the C target does, and lowers it to Cranelift IR (CLIF). From
there, one path runs the code at once in the `zakac` process (`zakac run --jit`), and the
other writes an object file and links it into an executable (`zakac build --backend
native`). [Native programs](../compiler/native.md#cranelift) shows the commands from the
outside; this page explains how the crate works.

```text
ir::Module ──emit──▶ CLIF of each function, the descriptors, zk_program
                         │
                         ├──JITModule──▶ code in zakac, run at once (the runtime linked into zakac)
                         │
                         └──ObjectModule──▶ main.o ──link with libzaka_runtime.a──▶ executable
```

The C target remains the reference. The native target follows its C statement for
statement, calls the same runtime ([The runtime](c-backend.md#the-runtime)) with the same
names and arguments, and writes the same descriptors and program table. What the C target
leaves to its runtime header (`zaka_target_c_runtime.h`) and to the C compiler, the native
target writes as CLIF.

## An example

Here is the function `နှစ်ထပ်` of the [IR example](codegen.md#an-example), which returns
`က * က`, as `zakac build --backend native --emit clif` writes it on an aarch64 Mac:

```text
function u0:0(i64) -> i64 apple_aarch64 {
    ss0 = explicit_slot 8, align = 8
    ...
                                block0(v0: i64):
                                    v1 = stack_addr.i64 ss0
                                    v2 = iconst.i64 0
                                    store notrap aligned v2, v1  ; v2 = 0
                                    v3 = stack_addr.i64 ss0
                                    store v0, v3
                                    v4 = iconst.i32 1
                                    call fn0(v4)  ; v4 = 1
@0001                               v5 = iconst.i32 1
@0001                               call fn1(v5)  ; v5 = 1
@0001                               v6 = stack_addr.i64 ss0
@0001                               v7 = load.i64 v6
@0001                               v8 = stack_addr.i64 ss0
@0001                               v9 = load.i64 v8
@0001                               v10, v11 = smul_overflow v7, v9
@0001                               brif v11, block1, block2

                                block1:
@0001                               v12 = symbol_value.i64 gv0
@0001                               v13 = symbol_value.i64 gv1
@0001                               call fn2(v12, v13)
@0001                               trap user1

                                block2:
@0001                               call fn3()
@0001                               return v10
}
```

The parameter `က` lives in the stack slot `ss0`, so the function stores `v0` there and
loads it back for each read. `fn0` is `zk_enter_call` and `fn1` is `zk_at_site`, which
open the frame of the backtrace and set its site (`@0001` is that site). The product uses
`smul_overflow`, because `--overflow stop` is the default: on overflow, `block1` calls the
stop `zk_overflow` with the texts of the operator and the data kind, and `block2` leaves the frame through `zk_leave_call`
and returns the product.

## The crate

| File | What it does |
| --- | --- |
| `cli.rs` | `run_jit`, which `zakac run --jit` calls through `zaka_vm::cli::JIT_RUNNER`; `build_main`, which `zakac build --backend native` calls; `--emit bin`, `obj` and `clif` |
| `emit.rs` | `emit`: declares the globals, the records of the functions and the descriptors, defines each function, then the main function, the bridges and the program table; `Shared`, the state the functions share (the module, the caches of texts, descriptors, runtime functions, C kinds and call shapes) |
| `func.rs` | `FnGen`, the generator of one function: a stack slot for each local, the blocks, the helpers (`rt`, `rt_stop`, `load`, `store`, `null_check`), the frames of the depth check and the backtrace, and the main function |
| `stmts.rs`, `exprs.rs` | a statement or an expression of the IR as CLIF |
| `ops.rs` | checked and wrapping arithmetic, division, shifts, float operations and conversions |
| `ffi.rs` | calls of C functions, leave statements, and the bridges of closures that go to C |
| `threads.rs` | the data block of a spawn and the step of each function a thread runs (see [Threads](#threads)) |
| `blob.rs` | `Blob`, data set out as C sets out a struct, with the addresses of other data and functions as relocations |
| `jit.rs` | the JIT module: each runtime function from `zaka_runtime::exports()`, each C function from its lib, then the run |
| `aot.rs` | the object module for the triple of the target, and the CLIF text |
| `unwind.rs` | the unwind tables of 64-bit Windows (`.xdata` and `.pdata`), which `cranelift-object` does not write there, so a debugger and a `longjmp` can walk the frames; on ARM64, `arm_xdata` puts a header word (the length of the function, the words of codes, no epilog) before the codes of Cranelift and fills their last word with `end`, and a row of `.pdata` holds two image offsets |
| `debug.rs` | the DWARF line table of an object: a row for the code of each statement, and a subprogram with the Zaka name of each function |
| `disasm.rs` | with the feature `disasm`, the listing of the machine code of an object (x86-64 through iced-x86, aarch64 through yaxpeax-arm), each function under its Zaka name and each call to the runtime named |
| `main.rs` | the test program: `zakac run` with the JIT in place, so a stop ends a child process |

The crate has the feature `jit`, which `zakac` turns on. It pulls in `cranelift-jit`,
`cranelift-native`, `libloading` and the runtime, which a build for the browser cannot
use. With the feature off, the crate lowers and writes objects alone, and the explorer
uses it in this form to show the CLIF of a program. The feature `disasm`, which the explorer
turns on, adds the listing of the machine code of the object beside the CLIF.

## Values in CLIF

A scalar is a CLIF integer or float of its size (`i8` to `i64`, `f32`, `f64`). A
true/false value (boolean) is an `i8`, a character is an `i32`, and an address and a
function value are pointers. A list, a struct, a plain list and text are blocks of octets
(bytes) laid out the same as on the VM, and the value of each is the address of its
block, as in the C target.

Each local lives in its own stack slot, sized and aligned for its data kind (type), and a
scalar is loaded from its slot and stored back. Cranelift's register allocator keeps the
values of a block in registers, so the slots cost little, and the address of a local is
the address of its slot. A function receives a list or struct parameter through its
address and copies it into its slot, unless the flow checker marks the parameter as one
the function never writes. A list or struct result goes into the slot the caller passes,
as in C.

A function value is the address of a record whose first word is code. Each function has
a record in data, `{code, 0}`, and a call through a value tests the second word: for a
closure, the call runs its thunk with the record after the arguments, and for a plain
function, it runs the code.

## Checks and stops

The checks of the `default` level, and of `--checks`, are written inline:

- `+`, `-` and `*` use `sadd_overflow`, `uadd_overflow`, `smul_overflow` and the rest
  under `--overflow stop`, and the plain operation under `wrap`.
- A division tests for a zero divisor and for the one overflow of a signed division.
- A shift tests its width.
- A load or a store through an address tests for NULL when the `null` check is on.

A failed check calls a stop of the runtime (`zk_plain_trap` and the others like it),
which writes the message and the backtrace and ends the process with status 1, and the
block after the call is marked unreachable. When the depth check or the debug info is
on, `zk_enter_call` and `zk_leave_call` wrap each function, and `zk_at_site` sets the site
of the top frame before each statement whose site differs from the last one, as
`ZK_ENTER`, `ZK_LEAVE` and `ZK_AT` do in C.

`FnGen` tracks whether the current block can be reached (`live`, and the set `reached`).
After a return, a stop or a jump, the next statement writes no code until a label reaches
it, so Cranelift never sees an instruction after the end of a block.

## The program table

`define_table` writes `zk_program` as data, field for field as `ZkProgram` in
`zaka_runtime` sets it out: the check bits, the limits, the texts of the stops in the
locale of the compiler, the rights of the build, the names of the functions and the
tables of sites. The main function calls `zk_start(argc, argv, &zk_program)`, runs the
program start block (entry point) or the top of the program, flushes the output with
`zk_finish` and returns the status. Under the JIT, the rights in the table are blank,
and `zaka_runtime::set_gate` passes the runtime the rights of `zakac run` and the asker
of the VM instead.

## C functions

Each C call goes through `zk_ffi_call(code, shape, args, ret)` of the runtime. The shape
is data in the code: the C kind of the result and of each argument (a tag, or the
members of a struct), and the count of fixed parameters before a `...`. The runtime
builds the libffi call interface of a shape on its first call and keeps it, so libffi
places structs passed as values, and the values after a `...`, where the ABI of the host
wants them. Text goes as a duplicate (copy) that ends in a zero octet and is freed after
the call, as the C wrappers do.

A function of the program that C calls back gets a bridge, emitted as CLIF after the
main function, with the signature of its function kind. `zk_cb_shape_of` turns the shape
in the code into the shape that the closures of `callbacks.rs` read, so a closure that
goes to C is a libffi closure in the runtime, as in the C target.

The JIT finds a C function in its lib (library) through `libloading`, with the names the
VM uses for each lib (`ucrtbase.dll` for libc and libm on Windows), and passes its
address to Cranelift. The AOT path links the libs with the `-lX` flags of the C target.

## Threads

A spawn writes the data block the same as the C target does (`threads.rs`, with the
offsets of `zakac_codegen_target_c::threads`), and the step of each function a thread runs
is Cranelift code, `zk_thread_step_<f>`, defined after the bridges. A spawn calls
`zk_task_start` for a light task and `zk_thread_start` for an OS thread, with the rule of
the C target. The guard and the
leave are C: an AOT build compiles `native_threads_glue` beside the object and links it.

The JIT has no C compiler at run time, so the runtime carries its own guard. With the
feature `guard`, which the `jit` feature of this crate turns on, the build script of
`zaka_runtime` compiles `c/thread_guard.c` through `cc`, with the prefix of the runtime on
each name, and `zaka_runtime::thread_glue` passes the JIT its addresses under the plain
names `zk_thread_guard` and `zk_thread_leave`. On 64-bit Windows, that guard clears the
frame of its jump buffer, because the code of the JIT has no table to unwind.

## JIT and AOT

The JIT builds a `JITModule` for the host ISA (`cranelift-native`), with each function of
the runtime under its C name. The runtime is linked into `zakac`, so the code calls it in
place. The program runs on the OS thread of `zakac`, so a stop ends `zakac`.

The AOT path builds an `ObjectModule` for the triple of the target, as
position-independent code: `x86_64`, `aarch64` or `riscv64` on Windows
(`pc-windows-gnu`, or `pc-windows-msvc` for the ABI of MSVC), macOS or Linux. Windows on
ARM64 is the exception: `cranelift-object` writes no COFF relocation for the GOT or the
page of a name on ARM64, so its code is not position-independent, and `far_refs` marks
each name a function refers to as not colocated, so the code loads each address from a
pool through `IMAGE_REL_ARM64_ADDR64`, which the linker relocates.
`build_main` writes `main.o` (`main.obj` for MSVC) into the cache folder beside the
runtime lib of the ABI, and links both through the C toolchain of the C target
(`build_and_link`), with no C to compile. An executable is linked for the OS and the CPU
of the host and for an ABI whose runtime lib `zakac` carries, while `--emit obj` and
`--emit clif` accept another target.

Cranelift code calls each function of the runtime under its linked name
(`zk_rt_<version>_<hash>_<name>`, through `linked_name` of the C target). In a C lib,
`aot::object` renames, in the finished object, the names of the lib that C never calls
(`LibNames::held`, then `symbol_mut`), the same as `zaka_lib_names.h` maps them for the
glue.

## Tests

| Test | What it checks |
| --- | --- |
| `tests/samples.rs` | each repo sample prints the output the VM prints for it, through the JIT under `--overflow stop` and `wrap`, then as an AOT executable |
| `tests/traps.rs` | a zero divisor, an overflow, a cell past the end, a deep recursion, an optional with no data and a heap past its cap stop with the VM's message and status 1, and a stop names its line and the calls in progress |
| `tests/permissions.rs` | the rights of `zakac run --jit`, and the rights written into an executable |
| `tests/forms.rs` | each sample is lowered into the test process, and its program table and each descriptor are read back through the Rust structs of `zaka_runtime`, field for field against what the compiler meant to write |
| `tests/ffi_lib.rs` | the C lib of the C target's tests: structs as values, other values, text results, handles and closures given to C, through the JIT and as an executable; NULL from C where the program wants none stops the run |
| `tests/debug_info.rs` | the DWARF line table maps the code of each statement to its source line, unless the build is for release |
| `tests/disasm.rs` | the listing of the machine code of an object, each function under its Zaka name, for x86-64 and aarch64 |
| `tests/unwind.rs` | the unwind tables of 64-bit Windows in an object: `.xdata` with the unwind codes of each function, and `.pdata` with a row of three image offsets for each on x64 and two on ARM64 |

The tests run the program in a child process (the test program
`zakac_codegen_target_native` for the JIT), each in its own folder, so a stop ends that
process alone and a sample that writes a file writes it there.
