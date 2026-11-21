---
title: Building programs
description: The targets of zakac build in full, the VM, C and Cranelift, with their flags, the run-time checks of an executable, debug info, and C libs.
---

# Building programs

`zakac build` compiles a program for a selected target. It can emit operations
(instructions) for the VM (virtual machine), build an executable through C or Cranelift,
or build a C lib (library). For flags at a glance, see [Commands](commands.md#build).
For executable behavior and a worked lib example, see [Native programs](native.md) and
[A lib for C](../../learn/functions/c-lib.md).

```console
$ zakac build age.zaka
$ zakac run age.zkc
၂၀
$ zakac build --backend c age.zaka
$ ./age
၂၀
```

## The VM target

```text
zakac build [--emit zkc|zkct|asm] [flags] [-o FILE] [INPUT]
```

To build for the VM, we leave out `--backend` or write `--backend vm`. `--emit zkc` writes
the raw operations, `zkct` writes the text form (a first line that names the artifact,
then YAML), and `asm` writes a listing of the operations. The listing is plain text for
reading, and it goes to standard output, or to the file we give with `-o`. Here is the
listing of a program with a variable (`age.zaka`), for a 32-bit platform:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
အသက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၀
```

```console
$ zakac build --target-bits 32 --emit asm age.zaka
; zakac asm listing v1, platform 32 bits

fn f0 ပရိုဂရမ် (params 0)
locals
  l0 အသက် isize
     0  local_addr l0        ; အသက်
     1  push 20:isize
     2  store isize
     3  local_addr l0        ; အသက်
     4  load isize
     5  print
     6  ret

fn f1 main (params 0)
     0  call f0              ; ပရိုဂရမ်
     1  ret
```

[Instructions](../vm/instructions.md) lists each operation. The VM target also uses
`--results` and `--scratch` ([Commands](commands.md#options-of-the-compile-stages)).

## The C target

```text
zakac build --backend c [--emit bin|c|lib] [flags] [-o FILE] [INPUT]
```

`--backend c` writes a C file for each module of the program, then compiles and links the
files with a C compiler, and `--emit c` stops once the C files are written. The C is plain
C11, so GCC, Clang, MinGW and `zig cc` each build it, and so does MSVC on Windows
(`--target-abi msvc`, see [Windows](native.md#windows)). `--target-os wasi` builds a
WebAssembly program with the Clang of wasi-sdk ([WASI](native.md#wasi)). [Native
programs](native.md) shows what the executable checks and where its output differs from
the VM.

```console
$ zakac build --backend c age.zaka
$ ./age
၂၀
$ CFLAGS=-O3 zakac build --backend c --linker 'clang -fuse-ld=lld' age.zaka
```

## Flags of the C and native targets

Both targets use these flags, on top of the [compile
flags](commands.md#options-of-the-compile-stages) and `--release`:

| Flag | Value | Default | What it does |
| --- | --- | --- | --- |
| `--checks` | a level, then `+name` or `-name` items | `default` | the run-time checks of the executable, as a comma list or the flag given again; see [Run-time checks](#run-time-checks) |
| `--max-depth` | `<N>` | `4096` | the most nested calls before the executable stops with `more than N nested calls`, as `--max-depth` of the VM does; counted while the `depth` check is on |
| `--max-heap` | `<OCTETS>` | `0` | the most heap octets the executable holds at once, with no cap for 0; see below |
| `--trap` | `unwind` or `abort` | `unwind` | what a trap in a [thread](../language/threads.md#a-trap-in-a-thread) of the executable does: `unwind` ends that thread alone, and a wait for it traps in turn, while `abort` stops the whole program |
| `--cache` | `<DIR>` | the cache of the user | the folder for the C files, the objects and the runtime lib; see [Where the C goes](native.md#where-the-c-goes) |
| `--cc` | `<CMD>` | see below | the C compiler, with the words before its flags |
| `--linker` | `<CMD>` | the C compiler | the program that links the objects, such as `clang -fuse-ld=lld` |
| `--cc-flag` | `<FLAG>` | | one more flag for the C compiler; give it again for more |
| `--ld-flag` | `<FLAG>` | | one more flag for the linker; give it again for more |
| `--no-default-flags` | | off | leave out the default C flags `-O2 -w -ffp-contract=off` (`/nologo /O2 /w /fp:precise /utf-8 /MD` for MSVC); `-std=c11` remains |
| `--lib-name` | `<NAME>` | the name of the files | the name of a C lib (`--emit lib`) in `zk_reset_<NAME>`, `zk_last_trap_from_<NAME>` and the names it links under: ASCII letters, digits and `_`, starting with a letter or `_`; a file name that C cannot use needs it |
| `--runtime-lib` | `<FILE>` | the runtime lib that `zakac` carries | a runtime lib built for another target, with the OS libs it needs listed in `native-libs.txt` beside it; see [Cross builds](native.md#cross-builds) |
| `--allow-read`, `--allow-write`, `--allow-env` | `[=<PATHS>]` or `[=<NAMES>]` | none | the rights the executable has, written into it, as `zakac run` grants them to a run |
| `--allow-ffi`, `-A` (`--allow-all`) | | off | the right to call C functions, and all four rights |
| `--no-permission-prompt` | | off | the executable asks nothing on a terminal for a right it lacks |

**The cache.** The runtime lib is `libzaka_runtime.a`, or `zaka_runtime.lib` for MSVC,
which `zakac` carries inside it and writes into the cache. With no `--cache`, the folder
is `zakac/<folder>/<name>-<hash>` in `XDG_CACHE_HOME` or `~/.cache` (`%LOCALAPPDATA%` on
Windows), or `.zakac_c_cache` beside the source when that folder cannot be made.

**The C compiler.** With no `--cc`, the build uses `CC`, then the `cc` of a config file,
then the first of `cc`, `gcc`, `clang` and `zig cc` on the PATH. For `--target-abi msvc`
it uses `cl`, `clang-cl`, then the `cl` of Visual Studio. For WASI it uses `--cc` alone,
then `wasm32-wasip1-clang` or `wasm32-wasip2-clang` in the `bin` folder of
`WASI_SDK_PATH`, then on the PATH.

**The heap cap.** Under `--max-heap`, a counted block (text, a plain list, a box, a
closure record) past the cap stops the run with `no heap room for N octets`, as `--max-heap`
of the VM does, while a raw allocation of unsafe code past it gets the null address. See
[Checks](native.md#checks).

**The runtime lib of another target.** With a C compiler for that target in `--cc`, the
C and native targets build for its OS and CPU. The build reads the names that the lib
links its functions under from the lib itself, so a lib from other sources or from
another build of `zakac` works too.

**Rights.** The executable asks on a terminal for a right it lacks, as the VM does. See
[Permissions](../vm/running.md#permissions) and [Native
programs](native.md#permissions).

## Run-time checks

Each run-time check stops the program at one kind of mistake. A level sets all of them:
`none` turns them all off (the fastest, as C is), `default` turns on the checks of the
language, and `mem` turns on all of them.

| Check | Stops the program at | In |
| --- | --- | --- |
| `divide` | a zero divisor | `default`, `mem` |
| `shift` | a shift as wide as its data kind or wider | `default`, `mem` |
| `bounds` | a cell, a place, a slice or a range outside its list or its text | `default`, `mem` |
| `text` | a range of text that cuts a character, and u8s that are not UTF-8 | `default`, `mem` |
| `step` | a loop step that is not positive, in a counted loop and a parallel one alike; with it off, a parallel loop given a step of 0 stops on the `divide` check as it counts its rounds | `default`, `mem` |
| `depth` | more nested calls than `--max-depth` | `default`, `mem` |
| `null` | an access through the null address | `mem` |
| `overlap` | a block copied onto a block it overlaps | `mem` |
| `sanitizers` | the address and undefined-behavior sanitizers of the C compiler | `mem` |

The list starts from `default` and then applies each item in turn, so a level inside the
list starts over:

```console
$ zakac build --backend c --checks mem,-sanitizers,-depth age.zaka
$ zakac build --backend c --checks none,+bounds age.zaka
$ zakac build --backend c --checks=+null age.zaka
```

Integer overflow is not one of these checks: `--overflow` picks what it does.

## Flags for the C compiler and the linker

The flags reach the C compiler in this order:

1. `-std=c11`;
2. the default flags;
3. `-g`, unless we give `--release`;
4. `CFLAGS`;
5. each `--cc-flag`.

The linker gets, in order:

1. the objects and the runtime lib;
2. `-lm`;
3. the OS libs that the runtime lib needs (`-liconv` on macOS, for one);
4. the `-lX` of each lib that the [C functions](../language/functions-c.md#how-each-build-calls-c) of
   the program name, except libc and libm; a lib given as a path goes as it is, with a
   run path;
5. `LDFLAGS`;
6. each `--ld-flag`.

MSVC uses the same order in its own form: `/std:c11` and its default flags, then `X.lib`
for each lib, and the options of the linker after `/link`. A later flag wins where the
compiler keeps the last one, so `--cc-flag -O3` replaces `-O2`. `-w` silences warnings
wherever it appears, so to see warnings we need `--no-default-flags`.

## Debug info and release

A build with no `--release` carries debug info, so lldb and gdb stop on a line of Zaka
(see [Debuggers](native.md#debuggers)):

- The C target writes a `#line` before each statement and compiles with `-g` (`/Z7`, and
  `/DEBUG` for the link, under MSVC).
- The native target writes a DWARF line table and the Zaka name of each function into its
  object.

`zakac build --backend c --release` writes no table of places and no marks for them, so a
stop names the function alone (`(in addfee)`) and each backtrace line is a function name.
It also leaves out the debug info. The native target and the Cranelift runs of `zakac run`
leave out the same table under `--release`.

## The native target

```text
zakac build --backend native [--emit bin|obj|clif|lib] [flags] [-o FILE] [INPUT]
```

`--backend native` lowers the program with [Cranelift](https://cranelift.dev) to an object
file for the target, then links it with the runtime lib through the C compiler, as the C
target links. It runs the same checks as the C target, uses the same flags and gives the
same output. `--cc-flag`, `--no-default-flags` and `CFLAGS` change nothing, because there
is no C to compile. [Cranelift](native.md#cranelift) shows where it differs.

- `--emit obj` writes the object file alone, and `--emit clif` writes the Cranelift IR of
  each function, for reading.
- The CPU must be `x86_64`, `aarch64` or `riscv64`; the others stop the build with an
  error.
- An executable is linked for the OS and the CPU of the host alone, with either ABI on
  Windows. `--emit obj` and `--emit clif` accept another `--target-os`, `--target-arch`
  or `--target-abi`, so that we can link the object there.
- The `sanitizers` check has no effect, because Cranelift code has no marks for them.

```console
$ zakac build --backend native age.zaka
$ ./age
၂၀
$ zakac build --backend native --emit obj --target-os linux --target-arch x86_64 age.zaka
```

## A C lib

```text
zakac build --backend c|native --emit lib [--lib-name NAME] [flags] [-o FILE] [INPUT]
```

`--emit lib` builds a C lib from the functions marked with
`"<C name>" အမည်ဖြင့် ပြင်ပမှ ခေါ်ဆိုနိုင်သော` ([Functions C can
call](../language/functions-exports.md)), for C, Rust and each language that
calls C. `--backend native --emit lib` builds the same lib from Cranelift code, with the
wrappers in C beside it. The program needs no program start block, and a program with
none of those functions is an error.

```console
$ zakac build --backend c --emit lib -o out/mathlib mathlib.zaka
$ cc -Iout main.c out/libmathlib.a -liconv -lm -o main
```

### The files

The build writes three files in one folder, named after the input or the last part of
`-o`:

| File | What it contains |
| --- | --- |
| `lib<name>.a` (`<name>.lib` for MSVC) | the static lib: the objects of the program, with the runtime lib inside it |
| `lib<name>.so`, `lib<name>.dylib` or `<name>.dll` | the shared lib, which gives C these functions and nothing else: the functions of the program, `zk_last_trap_from_<name>`, `zk_reset_<name>`, `zk_free_from_<name>`, `zk_free_all_from_<name>`, `zk_last_trap`, `zk_free` and `zk_free_all`; MSVC writes its import lib as `<name>.dll.lib` |
| `<name>.h` | the C declarations: each function under its C name; each struct those functions accept or return, under its C name, with a check of its size; the id of each trap (`ZK_TRAP_OVERFLOW`, ...); the `zk_trap` struct; and the seven functions of the shared lib above |

### Calls and their status

Each function returns an `int32_t` status and writes its result through a last parameter,
an address it writes unless it is NULL. The status is one of these:

| Status | Meaning |
| --- | --- |
| `ZK_OK` (0) | the call succeeded |
| `1 +` the id of a trap | the call stopped on that trap |
| `ZK_STOPPED` (-1) | a trap stopped the lib earlier, so the lib refuses each call |

A trap stops the whole lib when its function, itself or through a function it calls,
reaches a mutable global, calls C or a function value, or starts a thread, because the
trap can leave the heap and the globals half written. A trap in each other function fails
that call alone.

`zk_last_trap_from_<name>()` returns that trap as a `zk_trap`: its id (`code`), its short
name (`kind`), its line (`message`), the lib, the call it stopped in (`where`: the
function, the file, the line and the column), and each call in progress (`frames`, the
innermost first, none in a `--release` build). It remains valid until the next trap of the
lib or its reset. `zk_last_trap()` returns the last trap of all the libs of the program:
each lib writes its traps to one slot that the linker keeps for all of them,
`zk_trap_slot_v1`, whichever build of `zakac` made it. A shared lib keeps a separate slot
on Windows and macOS, so there `zk_last_trap_from_<name>()` names the lib.

A trap puts back what its call found: the depth of the calls, the closures it lent to C,
the sink of the output and the queues of the drops, so the calls around it go on.
`zk_reset_<name>()` starts the lib again. The first call starts the runtime and sets the
globals.

### Data between Zaka and C

| Zaka value | In C |
| --- | --- |
| text, as an argument | `const char *` |
| text, as a result | `char *` |
| an address or a handle | `void *` |
| a struct | the struct whole, as the `.h` file declares it |
| a plain list | `zk_list_<cells>`, named after its cells (`zk_list_int32_t`, `zk_list_text`, `zk_list_person`, `zk_list_list_text`), with the C form of each cell |

Each block the lib gives C has the data kind and the count of its cells in front of it.
C frees one block with `zk_free`, or a block together with each block its cells point at,
at each depth, with `zk_free_all`. `zk_free_all_from_<name>` does the same with the C
runtime of the lib.

### Threads

Threads of C can call the lib:

- The functions whose trap fails the call alone run on several threads at once, with no
  lock. Such a function reads constants alone, and the counts of the heap turn atomic at
  its first call.
- The rest run one call at a time, under one lock for each lib of the runtime. The first
  call of the lib sets its globals under that lock, and a reset waits for the calls with
  no lock to end before the next call sets them again.
- A call that waits in a C function lets go of the lock, so other threads can go into the
  libs while it waits, and it takes the lock back once C returns, the same as Python lets
  go of its lock around C.
- While it waits, another thread can change a global. So, as Rust does, the checker
  counts a call to C as a change of each global: a function that reaches C takes globals
  as duplicates, never as loans, and no value a call has is freed under it. A global that
  the call reads again after C can have a new value.
- C can call a function of the program on each thread. On a thread that waits in C, the
  call goes on as part of that call. On another thread it runs as a call of the lib, and a
  trap there stops the lib and gives C a result of zeros.

### Several libs in one program

Several Zaka libs link into one program, static or shared. `<name>` is the name of the
files, or `--lib-name`: ASCII letters, digits and `_`.

- Each name that a lib links under and C never calls has the form
  `zk_<name>_<hash>_...`, with a hash of the lib name, its program and the runtime, so two
  libs never clash.
- The runtime links its names as `zk_rt_<version>_<hash>_...`, and the `.h` file maps
  `zk_free` and `zk_last_trap` to them. The static libs from one `zakac` share one
  runtime, and libs from two builds of `zakac` each keep their own. The build of each
  runtime joins its hash to the metadata of each Rust crate in it, so no name of one
  clashes with the other.
- A trap stops, or fails a call of, the lib it came from alone.
- `--max-heap` caps each lib apart: a block counts toward the lib whose call made it, and
  text and lists that a lib gives to C count toward none, since C frees them.
- `zk_free_from_<name>()` frees with the C runtime the lib was built with. On Windows a
  lib of GNU and a lib of MSVC each have their own, so a program that loads both frees
  what each returned with that lib's function.
- Each lib carries the calls of the libs it calls through C on top of its own, and a trap
  lists the calls of its own lib alone.

### Linking the lib

A program that links the static lib also links the OS libs of the runtime, which the
comment at the top of the `.h` file lists. The shared lib of macOS names itself
`@rpath/lib<name>.dylib`, so a program finds it through its run path
(`-Wl,-rpath,<folder>`). When the shared lib is beside the static one, the linker of macOS
picks the shared lib for `-l<name>`: give the static lib as a path, or keep it in a
separate folder, as a Rust build does with `-L <folder> -l static=<name>`.
