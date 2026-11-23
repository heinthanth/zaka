---
title: Native programs
description: How zakac turns a program into native code, through C or through Cranelift, how zakac run runs it, the checks the executable keeps, and where it differs from the VM.
---

# Native programs

`zakac build --backend c` writes a program as C, one file for each module, and compiles
that C into an executable. The executable prints what the program prints on the VM
(virtual machine), at the speed of C.

```console
$ zakac build --backend c hello.zaka
$ ./hello
မင်္ဂလာပါ ကမ္ဘာကြီး
```

`zakac build --backend native` makes the same executable without C: [Cranelift](#cranelift)
lowers the program to machine code. `zakac run` uses the same code generator to run a
source file as native code at once, whenever it can, and falls back to the VM otherwise
([The VM or Cranelift](commands.md#the-vm-or-cranelift)). This page shows what a native
program keeps from the VM and what it does not. [Building programs](build.md) lists each
flag of the two targets, and [Hello, world](../../learn/hello/README.md#details) shows the
first native build.

## What the build needs

The build needs a C compiler on the PATH, which the native target uses for the link
alone. It uses the compiler that `--cc` names, else `CC`, else the first of `cc`, `gcc`,
`clang` and `zig cc`. The C it writes is plain C11, so GCC, Clang and `zig cc` each build
it. Windows supports two ABIs, MinGW and MSVC: see [Windows](#windows).

The runtime, `zaka_runtime`, is a Rust lib (library) that `zakac` carries inside it,
built for the host. The build writes it into the cache and links it, so no Rust toolchain
is needed. Neither target needs a libffi on the host, because the runtime carries its
own, closures to C included.

## Windows

We build a Windows executable for one of two ABIs, which `--target-abi` picks:

| ABI | Toolchain | Flags |
| --- | --- | --- |
| `gnu` | MinGW: the gcc or Clang of MSYS2 UCRT64, llvm-mingw, w64devkit, or `zig cc`; on ARM64, the Clang of MSYS2 CLANGARM64 or llvm-mingw, since no MinGW gcc builds for ARM64 (Rust names this target `aarch64-pc-windows-gnullvm`) | as on the other hosts |
| `msvc` | Visual Studio, or its Build Tools, with "Desktop development with C++": `cl` or `clang-cl` | `/std:c11 /nologo /O2 /w /fp:precise /utf-8 /MD`; a lib is `X.lib`, and the options of the linker follow `/link` |

When we leave out `--target-abi`, the ABI is the one `zakac` itself was built for. `zakac`
on Windows carries the runtime lib of both ABIs (`libzaka_runtime.a` and
`zaka_runtime.lib`) when its own build could make both, and it refuses an ABI it carries
no lib for, naming the target to add.

For `msvc`, `--cc` and `CC` win, then `cl` and `clang-cl` on the PATH (a Developer
Prompt), then the `cl` of the newest Visual Studio with the C++ tools, which the build
finds and runs with the variables of its Developer Prompt. A `--linker` of `link` or
`lld-link` receives `/OUT:` and each option itself, and other linkers link the same as
`cl` does.

- MSVC links a DLL through its import lib: a lib that the program names as a path to a
  `.dll` links the `.lib` of the same name beside it, and the program opens the `.dll`
  when it runs. MinGW writes one with `-Wl,--out-implib,<name>.lib`.
- `--checks +sanitizers` under `msvc` adds the address sanitizer alone
  (`/fsanitize=address`).
- `zakac run` runs inside `zakac`, on the VM or as native code, so a run uses the ABI of
  `zakac`.

## WASI

`--target-os wasi` builds a WebAssembly program that a WASI host, such as
[wasmtime](https://wasmtime.dev), runs. Its CPU is `wasm32`, with 32-bit platform-sized
integers, and `--target-abi` picks the version of WASI. The two ABIs do not mix, because
each has a separate libc.

| ABI | Makes | Runs in |
| --- | --- | --- |
| `wasip1`, the default | a core module that imports `wasi_snapshot_preview1` | wasmtime, wasmer, Node, and the WASI shims of the browsers |
| `wasip2` | a component of the WASI interfaces | wasmtime and other hosts of components |

```console
$ zakac build --backend c --target-os wasi hello.zaka
$ wasmtime hello.wasm
မင်္ဂလာပါ ကမ္ဘာကြီး
```

The build needs [wasi-sdk](https://github.com/WebAssembly/wasi-sdk). It uses the compiler
that `--cc` names, else `wasm32-wasip1-clang` (or `wasm32-wasip2-clang`) in the `bin`
folder of `WASI_SDK_PATH`, else that name on the PATH. `CC` and the `cc` of a config file
name a compiler for the host, so the build skips them. `zakac` carries the runtime lib of
both ABIs. With no extension on `-o`, the program ends in `.wasm`.

- A WASI host lets a program open no file unless told to: `wasmtime run --dir .
  program.wasm` gives it the working folder. The rights of the build (`--allow-read`,
  ...) hold inside it as well.
- WASI loads no shared libs, so a program that declares C functions does not build for
  it.
- The stack is 8 MiB, as a native thread has.
- The `sanitizers` check has no effect.
- `zakac run` runs no WASI program, so we run it in a WASI host.

Floats print the same as on the VM, digit for digit, on each host: the runtime works out
the digits from the bits of the float, with no print from libc.

## Checks

The executable keeps the checks that `--checks` names. It has three levels:

| `--checks` | What the executable checks | Cost |
| --- | --- | --- |
| `none` | nothing that C itself lacks: a zero divisor, a cell outside its list or a shift too far does what C does with it | the fastest |
| `default` | the checks of the language, each with the VM's message: a zero divisor, a shift too far, a counted loop whose step is not positive, a cell or a range outside its list, a range of text that cuts a character, octets that are not UTF-8, a call through a function value that contains no function (a backstop, since a function value is never null), and the wrong part of a result | a compare or two for each check |
| `mem` | those, plus a check for the null address on each read and write through an address, and the address and undefined-behavior sanitizers of the C compiler | far slower; for finding a mistake |

After the level, `--checks` accepts checks to add or drop one at a time: `divide`,
`shift`, `bounds`, `text`, `step`, `depth`, `null`, `overlap` and `sanitizers`, as in
`--checks mem,-sanitizers` or `--checks none,+bounds`. [Run-time
checks](build.md#run-time-checks) lists what each one stops.

With the `depth` check on, as under `default` and `mem`, the executable counts nested
calls and stops when more than `--max-depth` (default 4096) wait to return, with `the
program stopped: more than N nested calls`, as the VM does. Without it, as under `--checks
none`, nothing is counted, so a recursion with no base case overflows the native stack, as
it does in C. A [light task](../language/threads.md#light-tasks) calls no function again,
and when its calls still take more than its stack of 1 MiB, it stops at the trap `stack
full` (`the stack of a light task, N octets, ran full`): the thread ends alone under
`--trap unwind`, and the program stops under `--trap abort`. A debug build, the default, still keeps the list of calls in progress, so
`--checks none` names the place of a stop, while `--release` drops that list.

`--max-heap <OCTETS>` caps the heap, the same as the VM's flag of that name (default 0, no
cap). The executable counts the heap octets it has:

- A counted block (text, a plain list, a box, a closure record) counts at the size of its
  cells.
- A raw block of unsafe code (`zk_alloc`, `zk_realloc`, and `zk_raw_free` for the free
  statement) counts at the size the allocator gives (`malloc_size` on macOS,
  `malloc_usable_size` on Linux, `_msize` on Windows). A raw block on another host is not
  counted.
- A counted block past the cap stops the run with the VM's message, `the program stopped:
  no heap room for N octets`. A raw allocation past the cap gets the null address, as on
  the VM, and a resize past it stops the run.
- A block that C made and the program frees was never counted, so the count stops at 0
  there. A block the program gave to C, which C frees, is never subtracted.

`--overflow` works the same as for the VM: `stop`, the default, stops the program when a
result does not fit its data kind (type), and `wrap` wraps it around. A check that the
compiler proves can never fail costs nothing, at each level.

The checks of storage remain in the compiler. What it proves before the run holds in native
code too, and safe code counts its blocks there as on the VM, with the same ring
collector. A mistake that the VM alone would catch, such as a read past the end of a block
through a pointer in unsafe code, does what it does in C, unless `--checks mem` is on.
This also holds for a plain `zakac run` of a source file, which runs as native code with
the `default` checks, so to have the VM catch such a mistake we run with `--vm`. [Checks
in the VM and in native code](../concepts/checked.md) sorts each mistake.

A program that stops writes what it printed so far, then one line on standard error, and
its exit status is 1, as on the VM. The line starts with the name of the executable:

```console
$ ./divide
စားမည်
divide: the program stopped: integer division or remainder with zero as divisor (in ပရိုဂရမ်, at divide.zaka:5:1)
```

A debug build, the default, names the function and the place of the statement that
stopped, and lists the calls in progress as the VM does: when more than one call waits,
the lines after the trap are a stack backtrace, the most recent call first. The place is
the one the VM names: the start of the expression whose check failed, such as a division
in the middle of a line, or of the statement for a stop of the statement itself. A line of
the backtrace names the statement of its call. `ZAKA_BACKTRACE` works the same as for the VM (see [How a
run ends](../vm/running.md#how-a-run-ends)): unset gives a short backtrace, `full` lists
each call, and `0` none. The C of `main` is left out of the list.

```console
$ zakac build --backend c --release divide.zaka
$ ./divide
စားမည်
divide: the program stopped: integer division or remainder with zero as divisor (in ပရိုဂရမ်)
```

A `--release` build keeps no table of places, so a stop names the function alone, and
each backtrace line is a function name with no place.

## Cross builds

`zakac` carries the runtime lib of its own OS and CPU, so a build is for the host unless
we tell it otherwise. A build for another OS or CPU needs a runtime lib built for that
target and a C compiler for it. `mise run runtime:cross-build <triple>` builds the lib
with the Rust target of that triple (`rustup target add <triple>`), and writes
`native-libs.txt` beside it with the OS libs the lib needs. The build reads the names that
the lib links its functions under from the lib itself, so a lib built from other sources
than those of `zakac` works too:

```console
$ mise run runtime:cross-build x86_64-unknown-linux-gnu
$ zakac build --backend native --target-os linux --target-arch x86_64 \
    --runtime-lib target/runtime/x86_64-unknown-linux-gnu/libzaka_runtime.a \
    --cc x86_64-linux-gnu-gcc age.zaka
```

The C target accepts the same flags:

- `--target-os` and `--target-arch` pick the code and the compile-time conditionals.
- `--runtime-lib` lifts the check that the target is the host's.
- `--cc` (or `--linker`) builds and links for the target: a cross GCC,
  `zig cc -target <triple>`, or Clang with `--target`.

An executable for Windows gets `.exe` on each host. The VM and the native runs of
`zakac run` run on the host alone.

## Debuggers

A build with no `--release` carries debug info, so lldb and gdb stop on a line of Zaka
and show the lines of the calls in progress. The C target writes a `#line` before each
statement, which names its line of the source, and compiles with `-g`. The native target
writes a DWARF line table, from the code of each statement to its line, and a subprogram
with the Zaka name of each function. Both name the source with its absolute path.

```console
$ zakac build --backend native divide.zaka
$ lldb ./divide
(lldb) breakpoint set --file divide.zaka --line 4
(lldb) run
```

The debug info has lines alone, with no variables and no data kinds, so a debugger shows
the registers and the raw storage of the program, not its values under their names. A
backtrace through the C target names each function in C (`zk_f0`), while the native
target names it in Zaka. On macOS the debug info remains in the objects in the cache, which
the executable names, and `dsymutil` copies it into a `.dSYM` beside the executable, to
keep once the cache is gone.

## Permissions

An executable has the rights its build granted, with the same flags that `zakac run`
uses: `--allow-read[=<paths>]`, `--allow-write[=<paths>]`, `--allow-env[=<names>]`,
`--allow-ffi` and `-A` ([Permissions](../vm/running.md#permissions)). The build writes them
into the executable, which checks each file effect, the environment of the program start
block, each variable it reads and each C call against them, as the VM does:

- On a terminal it asks for a right it lacks, unless the build gave
  `--no-permission-prompt`.
- A denied file or variable fails with `ခွင့်ပြုချက်မရှိခြင်း`.
- A denied C call or environment stops the program with `permission_denied`.

Paths start from the folder the executable runs in.

```console
$ zakac build --backend c --allow-read=data report.zaka -o report
```

## Where the output differs from the VM

- **Addresses.** An address prints as the address in the native program, not the one on
  the VM.
- **Argument 0.** The first text of the program start block's arguments is the path of
  the executable, where the VM gives the path of the source file.
- **The trap line** names the statement where the VM names the operation, so a column can
  differ. A `--release` build names the function alone, and native code has no operation
  numbers.
- **Deep recursion.** Both stop at `--max-depth` calls under the language checks, while
  native code with `--checks none` counts no calls and runs out of the native stack.

All other values print the same: integers in Burmese digits, floats with the fewest digits
that read back the same, text, lists, structs, members of an enumeration and results.
Floats round after each operation, as on the VM, and the build turns off the fused
mul-add of the C compiler for that.

## C functions

A program that declares [C functions](../language/functions-c.md#c-functions) calls them
from native code with no VM between. The build writes one more file, `zaka_ffi.c`:

- It includes `<stddef.h>` and `<stdint.h>` and no libc header file, so a declaration in
  it cannot clash with the one in a header file. It turns off the warnings for a
  redeclaration that differs from the compiler's own
  (`-Wincompatible-library-redeclaration` in Clang), since `void *` for an `int *` is the
  same to the ABI.
- It declares each C function with the data kinds Zaka gives it (`_Bool` for true and
  false, `void *` for an address), and writes a struct that a call passes whole as a C
  struct, `zk_ffi_s<n>`, with a `_Static_assert` on its size.
- It defines a wrapper `zk_ffi_<n>` for each form of call (a C function, and the data
  kinds of its other values when it accepts some). The module files call the wrappers,
  whose declarations are in `zaka_kinds.h`. A struct goes in and out of a wrapper as the
  address of its octets.
- A C function that accepts other values (`အခြားတန်ဖိုးများနှင့်အတူ`) is declared with
  `, ...`, and each wrapper passes the other values of its call as C promotes them.
- A text parameter or other value goes to C as `const char *`: the wrapper makes a
  duplicate of the u8s with a zero octet at the end (`zk_text_cstr` of the runtime), calls
  the C function, and frees the duplicate (`zk_text_cstr_free`).
- A result of `စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` arrives as `const char *`: the wrapper
  passes it to `zk_text_from_c`, which writes the optional with no data for NULL, and
  otherwise copies the string into new text of the program. The wrapper reads the text
  before it frees the duplicates of the text arguments, since a result such as the one
  `strstr` returns points into them. C keeps what it returned, unless the result clause
  has `အပိုင်အနေဖြင့်`: then the wrapper frees the string with libc's `free` once the text
  has its octets.
- A struct goes in as an other value the same as it does as a parameter, so `zaka_ffi.c`
  defines the structs of the other values of each call too.
- A handle, a value of a [data kind from outside the
  program](../language/functions-c.md#data-kinds-from-outside-the-program), is a box of the
  program that contains C's address, and C gets that address as `void *`. The wrapper of
  the IR halts through `zk_closed_handle` when the box contains 0, which stops the run with
  the closed handle trap, and a C function that takes the handle over leaves 0 in the box.
  A handle result written with no optional promises that C returns no NULL: a NULL there
  stops the run with the VM's `null from C` message, and an optional handle result is
  blank for NULL. A read of a field is a `memcpy` from the address of the C struct plus the
  offset of the field, since that address lies in no block of the runtime, and a text
  field goes through `zk_text_from_c`, the same as a text result.
- A parameter of a function kind that crosses (see
  [Callbacks](../language/functions-c.md#callbacks)) is declared with the C signature of
  the function, with `void *` for each address. The wrapper passes `zk_fn_code(value)`
  cast to that pointer to a function. A function value contains a function at each
  point, so C never gets NULL there. The functions of the program have the C signature of
  the function kind that crosses, so C calls them as it calls a C function. A closure
  crosses through `zaka_callback.c` (see [Function values and
  closures](#function-values-and-closures)), and a trap in a callback is kept until C
  returns (see [Stops in unsafe code](../vm/traps-unsafe.md#c-functions)).

The link gets `-lX` for each lib `libX` that the program names, except libc and libm,
after `-lm` and before `LDFLAGS` and each `--ld-flag`. We give the folder of a lib with
`--ld-flag`:

```console
$ zakac build --backend c --ld-flag -L/opt/gears/lib gears.zaka
```

A lib given as a path (a text that contains `/` or `\`, ends in `.so`, `.dylib` or `.dll`,
or contains `.so.`) goes to the linker as it is, in place of `-lX`. On each host except
Windows, the link adds `-Wl,-rpath,<folder>` with the folder of the path, so the program
finds the lib when it runs. A relative path starts at the folder of the file that declares
the C function, and the compiler makes it absolute. On macOS the lib must name itself (its
install name) with an absolute path or `@rpath/...`, since a bare file name there is
looked up in the folder the program starts from.

With `--emit c`, we add the `-lX` flags to the `cc` line ourselves.

An address in a native program is a real address, so it reaches C as it is, with no block
of the VM to map. The checks of the program do not look inside C: a C function that writes
past a block does what it does in a C program.

## Function values and closures

A [function value](../language/function-values.md) is one word in the C, of the data kind
`zk_fn` (`const void *`): the address of a record whose first word is the code address.

- Each function has a static record `zk_r<n>`, `{code, NULL}`, declared in `zaka_kinds.h`
  and defined in `zaka_kinds.c`. The address of function `n` is `&zk_r<n>`. A function
  value is never NULL. A word of 0 in a heap block of unsafe code contains no function,
  and a call through it stops with the VM's message.
- A closure, the function value of a [nested
  function](../language/closures.md#nested-functions) with captures, is a counted record
  on the heap. Where the function is declared, `zk_closure(record, thunk, kind)` moves the
  record from a hidden local to a new counted block, leaves zeros behind, and writes the
  code of the thunk in the first word and the descriptor of the record's data kind in the
  second. That descriptor is never NULL, so `zk_fn_bound` tests it. The record counts as a
  holder of the box of each captured variable, so the closure and the block it was made
  in share them. The thunk accepts the declared parameters and the record, and calls the
  nested function with the boxes in the record.
- `zk_fn_dup` counts one more holder of a closure's record, so each duplicate runs the
  same closure. A drop subtracts one holder and frees the record, with the counts it has,
  when the last holder goes. A record that a drop leaves held waits for the ring
  collector, since a closure can capture the variable that contains it. `zk_fn_lend` sets
  the lowest bit of the address of a value lent to a call, so the callee runs the caller's
  closure and frees nothing, and a duplicate of a lent value counts as a separate holder.
  `zk_fn_rec` returns the address without that bit.
- A call through a value reads `zk_fn_bound(f)`: for a closure it calls the thunk with the
  arguments and the record, and for a plain function it calls the code with the
  arguments.
- A program that makes a closure and gives function values to C, or has a leave
  statement, gets one more generated file, `zaka_callback.c`: a head that includes the
  runtime, then a bridge and a `zk_cb_native_shape` (`zk_cbs_<n>`, the C data kinds of its
  values) for each function kind that goes to C. The closures themselves, and libffi, live
  in the runtime. The wrapper of a C function calls `zk_cb_code` for a parameter of a
  function kind, which returns the code address of a plain function or the libffi closure
  of a closure, and wraps the C call in `zk_cb_lent()` and `zk_cb_end()`, so the closure
  is lent to the call. `zk_cb_leave` and `zk_cb_revoke` run the two leave statements, and a
  late call from C stops the run with `ZK_TRAP_CLOSURE_ESCAPE`, with the message of the
  [closure escape trap](../vm/traps.md). The code of a closure is never freed, so a late
  call traps and never runs freed code. A call from C on another thread ends the run with
  a message.
- A C declaration accepts function values of numbers, booleans and addresses alone
  (ZK0465), so each closure that reaches C has a shape. A leave of a function value whose
  function kind accepts or returns a struct, text or a function value, which no C function
  can get, is a construct the C target cannot build.
- Printing a function value looks its code up in a table of names, so a closure prints as
  the function its thunk calls.

## Cranelift

[Cranelift](https://cranelift.dev) is a code generator written in Rust, which `zakac`
carries inside it. It lowers the IR of the program to machine code in two modes, from one
lowering:

| | `zakac build --backend native` | `zakac run` (`--jit`, or a source file with neither `--jit` nor `--vm`) |
| --- | --- | --- |
| makes | an object file, linked with the runtime lib into an executable | machine code in the process of `zakac`, run at once |
| needs | a C compiler or linker, for the link | nothing more |
| rights | the flags of the build, written into the executable | the flags of the run, as the VM uses them |
| argument 0 | the path of the executable | the input as written, as on the VM |
| limits | `--checks`, `--max-depth`, `--max-heap` | the `default` checks, `--max-depth`, `--max-heap` |
| `--trap` | `unwind` or `abort` | `unwind` or `abort`, as on the VM |

Both give the output fact `"native"` to a [compile-time
conditional](../language/compile-time.md), and both link the runtime that the C target
links, so a program prints the same, stops with the same messages and calls C the same as
the C target. A C function comes from the lib its declaration names: the native run opens
the lib as the VM opens it, and the executable links it.

- The CPU must be `x86_64`, `aarch64` or `riscv64`, because Cranelift has no code for
  `x86`, `arm` or `wasm32`.
- The C compiler builds no part of the program, so `--cc-flag` and `CFLAGS` change
  nothing, and the `sanitizers` check has no effect.
- `--emit obj` and `--emit clif` accept another OS or CPU, and `--emit clif` writes the
  Cranelift IR of each function, for reading.
- A stop in a native run ends `zakac` with the message and the exit status 1, the same as
  an executable ends.
- A native run checks what the `default` level checks, so a pointer mistake in unsafe
  code that the VM would stop on, such as a read through the null address, crashes the
  run or reads octets past the block. `zakac run --vm` runs the program on the VM instead.

```console
$ zakac run --jit hello.zaka
မင်္ဂလာပါ ကမ္ဘာကြီး
$ zakac build --backend native hello.zaka
$ ./hello
မင်္ဂလာပါ ကမ္ဘာကြီး
```

## Where the C goes

The build writes the C into a cache folder. With no `--cache`, it is
`zakac/<folder>/<name>-<hash>` in the cache folder of the user (`XDG_CACHE_HOME`, or
`~/.cache`; `%LOCALAPPDATA%` on Windows), or `.zakac_c_cache` beside the source when that
folder cannot be made. The build writes the runtime lib, `libzaka_runtime.a`, beside the
C, then compiles the C files at once and links them with it. `--emit c` stops after the
files are written, so that a C compiler can build them later, on the host the runtime lib
was built for:

```console
$ zakac build --backend c --emit c --cache gen/ app.zaka
$ cc -std=c11 -O2 gen/*.c gen/libzaka_runtime.a -lm -liconv -o app
```

The OS libs after the runtime lib depend on the host, such as `-liconv` on macOS. A program
with a `zaka_callback.c` in `gen/` links `-lffi` as well.

| File | What it contains |
| --- | --- |
| `m0.c`, `m1.c`, ... | the globals and functions of each module |
| `main.c` | `main`: the arguments, the program start block, the exit status |
| `zaka_checks.h` | the run-time checks, one `ZK_CHECK_<NAME>` set to 1 or 0 for each (`ZK_CHECK_DIVIDE`, `ZK_CHECK_NULL`, ...), `ZK_MAX_DEPTH` and `ZK_DEBUG_INFO`; each file of the program includes it |
| `zaka_kinds.h` | the declaration of each global, function and data kind |
| `zaka_kinds.c` | the description of each data kind, and `zk_the_program`: the table the runtime reads at the start, with the checks, the limits, the rights, the texts of the messages and the places of the stops |
| `zaka_ffi.c` | the wrappers of the C functions of the program, when it has some |
| `zaka_callback.c` | a bridge and a shape for each function kind of a closure that goes to C, when the program makes a closure and gives function values to C, or has a leave statement |
| `zaka_runtime.h` | the functions of the runtime and the structs the program shares with it, written from the Rust of the runtime |
| `zaka_target_c_runtime.h`, `zaka_c_sugars.h` | the inline code of each operation: checked arithmetic, loads and stores |
| `libzaka_runtime.a` | the runtime: text, lists, tables, prints, the host, the rights, the stops and the closures that go to C |
