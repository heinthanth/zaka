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
| `zaka_rt.h`, `zaka_rt*.c` | the runtime: checked arithmetic, text, lists, prints and the host |
