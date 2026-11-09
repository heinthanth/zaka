---
title: Native programs
description: How zakac build --target c turns a program into an executable through C, the checks the executable keeps, and where it differs from the VM.
---

# Native programs

`zakac build --target c` writes a program as C, a file for each module, and compiles that
C into an executable. The executable prints what the program prints in the VM, at the speed
of C.

```console
$ zakac build --target c hello.zaka
$ ./hello
မင်္ဂလာပါ ကမ္ဘာကြီး
```

[Commands](commands.md#build) lists each option of `build`. This page tells what a native
program keeps from the VM, and what it does not.

## What the build needs

A C compiler on the PATH. The build takes the one `--cc` names, else `CC`, else the first
of `cc`, `gcc`, `clang` and `zig cc`. The C it writes is plain C11, so GCC, Clang and `zig cc`
each build it. On Windows, a MinGW toolchain such as llvm-mingw or w64devkit, or Zig, builds
it with no Visual Studio.

`i128` and `u128` need a C compiler with 128-bit integers, as GCC and Clang have; another
compiler stops at an `#error`. `f16` and `f128` have no C kind that each compiler holds, so
the build refuses them with ZK0801.

## Checks

The executable keeps the checks `--checks` names:

| `--checks` | What the executable checks | Cost |
| --- | --- | --- |
| `none` | nothing that C itself lacks: a zero divisor, a cell outside its list or a shift too far does what C does with it | the fastest |
| `default` | the checks of the language, each with the VM's message: a zero divisor, a shift too far, a counted loop whose step is not positive, a cell or a range outside its list, a range of text that cuts a character, octets that are not UTF-8, a call through `ဗလာ`, and the wrong part of a result | a compare or two for each check |
| `mem` | those, a check for the null address on each read and write through an address, and the address and undefined-behavior sanitizers of the C compiler | far slower; for finding a mistake |

`--overflow` works as it does for the VM: `stop`, the default, stops the program when a
result does not fit its kind, and `wrap` wraps it around. A check the compiler proves never
fails costs nothing, at each level.

The checks of storage remain in the compiler. What it proves before the run, such as no use
after free through an owner, holds in native code too. A mistake the VM alone would catch,
such as a read past the end of a block through a raw address, does what it does in C,
unless `--checks mem` is on. [Checks in the VM and in native code](../concepts/checked.md)
sorts each mistake.

A program that stops writes what it printed so far, then one line on standard error, and
its exit status is 1, as in the VM. The line starts with the name of the executable:

```console
$ ./divide
စားမည်
divide: the program stopped: integer division or remainder with zero as divisor
```

## Where the output differs from the VM

- **Addresses.** An address prints as the address in the native program, not the one in
  the VM.
- **Argument 0.** The first text of the start block's arguments is the path of the
  executable, where the VM gives the path of the source file.
- **The trap line** names no function and no op: native code has no op numbers.

Each other value prints the same: integers in Burmese digits, floats with the fewest digits
that read back, text, lists, structs, members of an enumeration and results. Floats round
after each operation, as in the VM; the build turns off the fused mul-add of the C
compiler for that.

## C functions

A program that declares [C functions](../language/functions.md#c-functions) calls them
from native code with no VM between. The build writes one more file, `zaka_ffi.c`:

- It includes `<stddef.h>` and `<stdint.h>` and no libc header, so a declaration in it
  cannot clash with the one a header holds. It turns off the warnings for a redeclaration
  that differs from the compiler's own (`-Wincompatible-library-redeclaration` in Clang),
  since `void *` for an `int *` is the same to the ABI.
- It declares each C function with the kinds Zaka gives it (`_Bool` for true and false,
  `void *` for an address), and writes a struct that a call passes whole as a C struct,
  `zk_ffi_s<n>`, with a `_Static_assert` on its size.
- It defines a wrapper `zk_ffi_<n>` for each shape of call (a C function, and the kinds of
  its other values when it takes some); the module files call the wrappers, whose
  declarations stand in `zaka_kinds.h`. A struct goes in and out of a wrapper as the
  address of its octets.
- A C function that takes other values (`နှင့် အခြားတန်ဖိုးများ`) is declared with
  `, ...`, and each wrapper passes the other values of its call as C promotes them.
- A text parameter or other value goes to C as `const char *`: the wrapper makes a
  duplicate of the u8s with a zero octet at the end (`zk_text_cstr` of the runtime), calls
  the C function, and frees the duplicate (`zk_text_cstr_free`).

- A result of `စာသား ရလဒ် သို့မဟုတ် မအောင်မြင်သည့်သတင်း` is held as `const char *`: the
  wrapper gives it to `zk_text_from_c`, which writes the failure for NULL and otherwise
  copies the string into new text of the program. The wrapper takes the text before it
  frees the duplicates of the text arguments, since a result such as `strstr` gives points
  into them. C keeps what it gave, unless the result clause holds `အပိုင်အနေဖြင့်`: then the
  wrapper frees the string with libc's `free` once the text holds its octets.
- A struct goes in as an other value as it does as a parameter, so `zaka_ffi.c` defines the
  structs of the other values of each call too.
- A parameter of a function kind that crosses (see
  [Callbacks](../language/functions.md#c-functions)) is declared with the C signature of the
  function, with `void *` for each address. The wrapper passes `zk_fn_code(value)` cast to
  that pointer to a function, and NULL for `ဗလာ`. The functions of the program have the C
  signature of the kind that crosses, so C calls them as it calls a C function.
  A trap in a callback is kept until C returns (see
  [When a program stops](../vm/traps.md#c-functions)).

The link gets `-lX` for each lib `libX` the program names, but libc and libm, after `-lm`
and before `LDFLAGS` and each `--ld-flag`. Give the folder of a lib with `--ld-flag`:

```console
$ zakac build --target c --ld-flag -L/opt/gears/lib gears.zaka
```

A lib given as a path (a text that holds `/` or `\`, ends in `.so`, `.dylib` or `.dll`, or
holds `.so.`) goes to the linker as it is, in place of `-lX`, and on each host but Windows
the link adds `-Wl,-rpath,<folder>` with the folder of the path, so the program finds the
lib when it runs. A relative path starts at the folder of the file that declares the C
function, and the compiler makes it absolute. On macOS the lib must name itself (its
install name) with an absolute path or `@rpath/...`; a bare file name there is looked up
in the folder the program starts from.

With `--emit c`, the `-lX` flags are for the reader to add to the `cc` line.

An address in a native program is a real address, so it reaches C as it is; there is no
block of the VM to map. The checks of the program do not look inside C: a C function that
writes past a block does what it does in a C program.

## Function values and closures

A [function value](../language/function-values.md) is one word in the C, of the kind `zk_fn`
(`const void *`): the address of a record whose first word is the code address.

- Each function has a static record `zk_r<n>`, `{code, NULL}`, declared in
  `zaka_kinds.h` and defined in `zaka_kinds.c`. The address of function `n` is `&zk_r<n>`.
  `ဗလာ` is NULL.
- A closure, the function value of a [nested function](../language/function-values.md#nested-functions-and-closures)
  with captures, is a record on the heap. Where the function is declared,
  `zk_closure(record, thunk, kind)` moves the record from a hidden local to a new block,
  leaves zeros behind, and writes the code of the thunk in the first word and the
  descriptor of the record kind in the second, which is never NULL, so `zk_fn_bound` tests
  it. The thunk takes the declared parameters and the record, and calls the nested function
  with the address of each field, or the address a borrowed field holds.
- `zk_drop`, `zk_dup_seqs` and `zk_lend_seqs` walk a function value as each walks text:
  `zk_fn_dup` copies the record and duplicates its fields (a record that owns a block
  stops the run with `ZK_TRAP_CLOSURE_OWNER`), a drop frees the record with what its fields
  own, owner blocks included (`zk_free_owners`, through the `owns` flag of the
  descriptors), and `zk_fn_lend` sets the lowest bit of the address, so the callee
  runs the caller's closure and frees nothing. `zk_fn_rec` gives the address without
  that bit.
- A call through a value reads `zk_fn_bound(f)`: for a closure it calls the thunk with the
  arguments and the record, for a plain function it calls the code with the arguments.
- `zk_fn_plain(f)` is run on a value headed where a closure that borrows cannot go. It
  stops the run with `ZK_TRAP_CLOSURE_ESCAPE` when the descriptor of the record tells that it
  holds a borrow, with the message of the [closure escape trap](../vm/traps.md#all-the-traps).
  `zk_fn_c(f)` stops a closure of each sort that goes to C.
- Printing a value looks its code up in a table of names, so a closure prints as the
  function its thunk calls.

## Where the C goes

The build writes the C into a cache folder. With no `--cache`, it is
`zakac/<folder>/<name>-<hash>` in the cache folder of the user (`XDG_CACHE_HOME`, or
`~/.cache`; `%LOCALAPPDATA%` on Windows), or `.zakac_c_cache` beside the source when that
folder cannot be made. `--emit c` stops there, so a C compiler can build the files at a later time:

```console
$ zakac build --target c --emit c --cache gen/ app.zaka
$ cc -std=c11 -O2 gen/*.c -lm -o app
```

| File | Holds |
| --- | --- |
| `m0.c`, `m1.c`, ... | the globals and functions of each module |
| `main.c` | `main`: the arguments, the start block, the exit status |
| `zaka_kinds.h` | the declaration of each global, function and kind |
| `zaka_kinds.c` | the description of each kind, and the texts of the messages |
| `zaka_ffi.c` | the wrappers of the C functions of the program; written when it has some |
| `zaka_rt.h`, `zaka_rt*.c` | the runtime: checked arithmetic, text, lists, prints and the host |
