---
title: The native target
description: How zakac_codegen_target_native lowers the IR to Cranelift IR, runs it at once in zakac or links it into an executable, and the tests that hold it to the VM.
---

# The native target

`zakac_codegen_target_native` is the third backend. It reads the IR that the flow checker
leaves, as the C target does, and lowers it to Cranelift IR (CLIF). From there one path
runs the code at once in the `zakac` process (`zakac run --jit`), and the other writes an
object file and links it into an executable (`zakac build --backend native`). [Native
programs](../compiler/native.md#cranelift) tells what a user sees; this page tells how it
is built.

```text
ir::Module ──emit──▶ CLIF of each function, the descriptors, zk_program
                         │
                         ├──JITModule──▶ code in zakac, run at once (the runtime linked into zakac)
                         │
                         └──ObjectModule──▶ main.o ──link with libzaka_runtime.a──▶ executable
```

The C target remains the reference: the native target follows its C statement for
statement, calls the same runtime ([The runtime](c-backend.md#the-runtime)) with the same
names and arguments, and reads the same descriptors and program table. What the C target
leaves to its own runtime header (`zaka_target_c_runtime.h`) and the C compiler, the native target writes as CLIF.

## The crate

| File | Job |
| --- | --- |
| `cli.rs` | `run_jit`, which `zakac run --jit` calls through `zaka_vm::cli::JIT_RUNNER`; `build_main`, which `zakac build --backend native` calls; `--emit bin`, `obj` and `clif` |
| `emit.rs` | `emit`: declares the globals, the records of the functions and the descriptors, defines each function, then the main function, the bridges and the program table; `Shared`, the state the functions share (the module, the caches of texts, descriptors, runtime functions, C kinds and call shapes) |
| `func.rs` | `FnGen`, the generator of one function: a stack slot for each local, the blocks, the helpers (`rt`, `rt_stop`, `load`, `store`, `null_check`), the frames of the depth check and the backtrace, and the main function |
| `stmts.rs`, `exprs.rs` | a statement or an expression of the IR as CLIF |
| `ops.rs` | checked and wrapping arithmetic, division, shifts, float operations and conversions |
| `ffi.rs` | calls of C functions, leave statements, and the bridges of closures that go to C |
| `blob.rs` | `Blob`, data set out as C sets out a struct, with the addresses of other data and functions as relocations |
| `jit.rs` | the JIT module: each runtime function from `zaka_runtime::exports()`, each C function from its lib, then the run |
| `aot.rs` | the object module for the triple of the target, and the CLIF text |
| `main.rs` | the test program: `zakac run` with the JIT in place, so a stop ends a child process |

The crate has the feature `jit`, which `zakac` turns on: it pulls in `cranelift-jit`,
`cranelift-native`, `libloading` and the runtime, which a build for the browser cannot
take. With the feature off, the crate lowers and writes objects alone, and the explorer
uses it so to show the CLIF of a program.

## Values in CLIF

A scalar is a CLIF integer or float of its size (`i8` to `i64`, `f32`, `f64`); a boolean
is an `i8`, a character an `i32`, and an address and a function value a pointer. A list,
a struct, a plain list and text are blocks of octets set out as the VM sets them out,
and their value is the address of the block, as in the C target.

Each local lives in a stack slot of its own, sized and aligned for its kind; a scalar is
loaded from its slot and stored back. Cranelift's register allocator keeps the values of
a block in registers, so the slots cost little, and an address of a local is the address
of its slot. A function takes a list or struct parameter through its address and puts it
into its slot, unless the flow checker marks it as one the function never writes, and a
list or struct result goes into the slot the caller gives, as in C.

A function value is the address of a record whose first word is code. Each function has a
record in data, `{code, 0}`, and a call through a value tests the second word: a closure
calls its thunk with the record after the arguments, a plain function calls its code.

## Checks and stops

The checks of the `default` level, and of `--checks`, are written inline:

- `+`, `-` and `*` use `sadd_overflow`, `uadd_overflow`, `smul_overflow` and the rest
  under `--overflow stop`, and the plain operation under `wrap`;
- a division tests a zero divisor and the one overflow of a signed division;
- a shift tests its width;
- a load or a store through an address tests NULL when the `null` check is on.

A failed check calls a stop of the runtime (`zk_plain_trap` and its kin), which writes the
message and the backtrace and ends the process with status 1; the block after the call is
marked unreachable. `zk_enter_call` and `zk_leave_call` wrap each function when the depth
check or the debug info is on, and `zk_at_site` sets the site of the top frame before each
statement whose site differs from the last, as `ZK_ENTER`, `ZK_LEAVE` and `ZK_AT` do in C.

`FnGen` tracks whether the current block can be reached (`live`, and the set `reached`):
after a return, a stop or a jump, the next statement opens no code until a label
reaches it, so Cranelift never sees an instruction after the end of a block.

## The program table

`define_table` writes `zk_program` as data, field for field as `ZkProgram` in
`zaka_runtime` sets it out:
the check bits, the limits, the texts of the stops in the locale of the compiler, the
rights of the build, the names of the functions and the tables of sites. The main function
calls `zk_start(argc, argv, &zk_program)`, runs the start block or the top of the
program, flushes the output with `zk_finish` and returns the status. Under the JIT the
rights in the table are blank: `zaka_runtime::set_gate` hands the runtime the rights of
`zakac run` and the asker of the VM instead.

## C functions

Each C call goes through `zk_ffi_call(code, shape, args, ret)` of the runtime. The shape is
data the code holds: the C kind of the result and of each argument (a tag, or the members
of a struct), and the count of fixed parameters before a `...`. The runtime builds the
libffi call interface of a shape on its first call and keeps it, so libffi places structs
passed as values, and the values after a `...`, as the ABI of the host wants them. Text
goes as a duplicate that ends in a zero octet, freed after the call, as the C wrappers do.

A function of the program that C calls back gets a bridge, emitted as CLIF after the main
function, with the signature of its kind. `zk_cb_shape_of` turns the shape the code holds
into the shape the closures of `callbacks.rs` read, so a closure that goes to C is a
libffi closure in the runtime, as in the C target.

The JIT finds a C function in its lib through `libloading`, with the names the VM uses
for each lib (`ucrtbase.dll` for libc and libm on Windows), and hands its address to
Cranelift. The AOT path links the libs with the `-lX` flags of the C target.

## JIT and AOT

The JIT builds a `JITModule` for the host ISA (`cranelift-native`), with each function of
the runtime under its C name: the runtime is linked into `zakac`, so the code calls it
in place. The program runs on the thread of `zakac`; a stop ends `zakac`.

The AOT path builds an `ObjectModule` for the triple of the target: `x86_64`, `aarch64` or
`riscv64` on Windows (`pc-windows-gnu`, or `pc-windows-msvc` for the ABI of MSVC), macOS
or Linux, as position-independent code. `build_main` writes `main.o` (`main.obj` for
MSVC) into the cache folder, beside the runtime lib of the ABI, and links both through
the C toolchain of the C target (`build_and_link`), with no C to compile. An executable is
linked for the OS and the CPU of the host, and an ABI whose runtime lib `zakac` carries;
`--emit obj` and `--emit clif` take another target.

## Tests

| Test | Checks |
| --- | --- |
| `tests/samples.rs` | each repo sample prints, through the JIT under `--overflow stop` and `wrap`, then as an AOT executable, the output the VM prints for it |
| `tests/traps.rs` | a zero divisor, an overflow, a cell past the end, a deep recursion, an optional with no data and a heap past its cap stop with the VM's message and status 1; a stop names its line and the calls in progress |
| `tests/permissions.rs` | the rights of `zakac run --jit`, and the rights written into an executable |
| `tests/forms.rs` | each sample is lowered into the test process, and its program table and each descriptor read back through the Rust structs of `zaka_runtime`, field for field against what the compiler meant to write |
| `tests/ffi_lib.rs` | the C lib of the C target's tests: structs as values, other values, text results, handles and closures given to C, through the JIT and as an executable; NULL from C where the program wants none stops the run |

The tests run the program in a child process (the test program `zakac_codegen_target_native`
for the JIT), each in a folder of its own, so a stop ends that process alone and a sample
that writes a file writes it there.
