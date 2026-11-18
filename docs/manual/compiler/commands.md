---
title: Commands
description: Each zakac command, what it reads, what it writes, and each of its options.
---

# Commands

`zakac` has one command for each stage of the compiler, one to build a program for a
target, and one to run a program. These share one shape:

```console
$ zakac <command> [options] [INPUT]
```

`INPUT` is a source file, or `-` for standard input (the default when it is left out).
A stage command reads the source, runs the stages up to its own, and writes what that
stage made, the *artifact*. The `run` command goes to the end and runs the program.
[Stages](stages.md) tells what each artifact holds.

| Command | Runs up to | Writes | Raw / text file |
| --- | --- | --- | --- |
| `segment` | the word segmenter | the pieces of the text | `.zkseg` / `.zksegt` |
| `lex` | the lexer | the tokens | `.zktok` / `.zktokt` |
| `parse` | the parser | the tree | `.zkast` / `.zkastt` |
| `semcheck` | names, kinds and natural wording | the tree and the kind of each expression | `.zksem` / `.zksemt` |
| `irgen` | the IR builder and constant folding | the IR | `.zkir` / `.zkirt` |
| `borrowck` | the checker: the flow checks and the pointer lint | the IR and a digest of each function | `.zkbck` / `.zkbckt` |
| `build` | the code generator of a target | the ops of the VM (`--backend vm`), an executable through C (`--backend c`), or one through Cranelift (`--backend native`) | `.zkc` / `.zkct` |
| `run` | the VM | nothing: the program runs | |
| `lsp` | the checker, for each file an editor holds open | errors, colors and hovers, to the editor | |

Each stage also builds as a program of its own (`zakac_lexer`, `zakac_parser`,
`zakac_semcheck`, `zakac_ir_builder`, `zakac_borrow_checker`, `zakac_burmese_segmenter`,
`zaka_vm`) with the same options as its command. Each target of `build` does too:
`zakac_codegen_target_vm` and `zakac_codegen_target_c`; and so does the language server,
`zakac_lsp`.

## A small session

Here is a program that adds up two numbers and prints the sum:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ + ၂ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၃
```

Saved as `sum.zaka`, it runs with one command. The second command stops after the code
generator and prints a listing of the ops, not the ops:

```console
$ zakac run sum.zaka
၃
$ zakac build --emit asm sum.zaka
; zakac asm listing v1, platform 64 bits

fn f0 ပရိုဂရမ် (params 0)
     0  push 3:isize
     1  print
     2  ret

fn f1 main (params 0)
     0  call f0              ; ပရိုဂရမ်
     1  ret
```

The sum was worked out at compile time, so the VM just pushes `၃`.
[Stages](stages.md) follows this sentence through each stage.

## Input and output

All the stage commands take these three:

| Option | Meaning |
| --- | --- |
| `[INPUT]` | the source file, or `-` for standard input |
| `-o`, `--output <FILE>` | where to write the artifact: a file, a folder, or `-` for standard output (the default) |
| `--output-format <FORMAT>` | `text` or `raw`; when left out, the format follows the output |

The format is picked in this order:

1. `--output-format`, when it is given.
2. Standard output gets `text`, a readable YAML file.
3. A file name that ends in the text extension of the stage (`.zkirt`) gets `text`; one that
   ends in the raw extension (`.zkir`) gets `raw`.
4. A folder (an existing one, or a path ending in `/`) gets a file named after the input,
   with the raw extension, unless `--output-format text` is given.
5. Each other file name gets `raw`.

The raw form is compact and quick to load; the text form is for reading and for diffs.
Both hold the same data.

`build` follows these rules for `-o` as well, with two changes: `--emit` picks what it
writes in place of `--output-format`, and with no `-o` it writes beside the source file
(see [build](#build)).

```console
$ zakac build age.zaka -o out/age.zkc
$ zakac build age.zaka -o out/age.zkct
$ zakac build age.zaka -o out/
$ ls out
age.zkc  age.zkct
```

A stage writes its artifact even when it finds errors; the errors are in the artifact
too, and the exit code is 1. See [Diagnostics](diagnostics.md).

### What `run` reads

`run` takes a source file, or the ops of `zakac build` in either form (`.zkc` or
`.zkct`). It tells them apart from the first octets of the input, not from the file name:

```console
$ zakac build age.zaka
$ zakac run age.zkc
၂၀
$ zakac run age.zaka
၂၀
```

The other artifacts are not programs: `run` reads one of them as source and reports
errors at once. Running an artifact skips the compile, and the checks that were made when
it was written are the checks it carries.

## Options of the compile stages

These options belong to `semcheck` and the stages after it, and to `run`.

| Option | Default | Meaning |
| --- | --- | --- |
| `--target-bits <BITS>` | `32` for `wasm32`, else `64` | the bit count of the platform-sized integer kinds, `ကိန်းပြည့်` and `လက္ခဏာမဲ့ ကိန်းပြည့်`: 8, 16, 32 or 64 |
| `--target-os <OS>` | the host's | the OS a [compile-time conditional](../language/compile-time.md) reads: `windows`, `macos`, `linux` or `wasi`; `wasi` takes the CPU `wasm32` when `--target-arch` is not given, and `--backend c` builds a WebAssembly program for it ([Native programs](native.md#wasi)) |
| `--target-arch <ARCH>` | the host's | the CPU a compile-time conditional reads: `x86_64`, `aarch64`, `x86`, `arm`, `riscv64` or `wasm32` |
| `--target-abi <ABI>` | the usual one of the OS | the ABI a compile-time conditional reads: `gnu`, `msvc`, `apple`, `wasip1` or `wasip2`; left out, `apple` on macOS, `gnu` on Linux, `wasip1` on WASI, and on Windows the ABI `zakac` was built for. The VM and `--jit` take the ABI of `zakac` alone; an executable on Windows takes `gnu` or `msvc` ([Native programs](native.md#windows)) |
| `--lint-allow <NAME>`, `--lint-warn <NAME>`, `--lint-error <NAME>` | | set the level of a lint group or code, as below; the front end reads the groups `naturalness` and `spoken` |

These belong to `irgen` and the stages after it, and to `run`:

| Option | Default | Meaning |
| --- | --- | --- |
| `--fold <LIST>` | `all` | the foldings to run: `all`, `none`, or a comma list of `exprs`, `consts` and `branches` |
| `--no-unsafe` | off | refuse each use of `စိတ်မချရသော`, on a function or a block, with ZK0471 |
| `--comptime-allow-read[=<PATHS>]` | none | let the runs of [comptime calls](../language/compile-time.md#what-a-run-can-do) read files: each one, or those under the paths of a comma list; a path of the program starts from its folder |
| `--comptime-allow-write[=<PATHS>]` | none | let them write, remove, move files and make folders |
| `--comptime-allow-env[=<NAMES>]` | none | let them read variables of the environment: each one, or those named |
| `--comptime-allow-ffi` | off | let them call C functions |
| `--comptime-allow-all` | off | grant each of the four |

`--fold` changes the IR, not what the program does. `exprs` replaces an operation on
constants with its value, `consts` replaces a read of a `ကိန်းသေ` that was set to a
constant, and `branches` keeps the branch of an `if` that is taken and drops a loop whose
condition is false. Under `exprs`, a string literal whose template slots each hold a constant
becomes static text, and so does a `+` of constant text; with `--fold none` the program
builds that text when it runs, with the same output. Errors in constant operations, such as a division with the constant
zero as divisor (ZK0602), are reported with `--fold none` too.

```console
$ zakac irgen --fold none sum.zaka
```

With `--fold none` the IR of `sum.zaka` keeps `add` with `1` and `2` as its two sides;
without it, the IR holds the constant `3`.

These belong to `borrowck` and the stages after it, and to `run`. These are explained
in [Compiler checks](checks.md).

| Option | Default | Meaning |
| --- | --- | --- |
| `--overflow <MODE>` | `stop` | what integer arithmetic does with a result outside its kind: `stop` stops the program and warns about an operation that overflows for each value it can see (ZK0719), `wrap` wraps it around; see [Overflow](../language/expressions.md#arithmetic) |
| `--lint-allow <NAME>` | | leave out each finding of a lint group or code; give it again, or a comma list, for more |
| `--lint-warn <NAME>` | | report each finding of a lint group or code as a warning |
| `--lint-error <NAME>` | | report each finding of a lint group or code as an error |

A lint name is a group or a code:

| Group | Codes |
| --- | --- |
| `mem` | the pointer lint of unsafe code, warnings when no name sets it: ZK0702 to ZK0704, ZK0706 to ZK0712, ZK0721 |
| `leaks` | ZK0708 |
| `frees` | ZK0702, ZK0707 |
| `borrows` | ZK0703, ZK0704, ZK0706 |
| `nulls` | ZK0709, ZK0711 |
| `pointers` | ZK0710, ZK0712, ZK0721 |
| `doubts` | ZK0711, ZK0723: the findings the checker cannot prove, off until a name sets them |
| `overflow` | ZK0719 |
| `naturalness` | the [natural wording](../language/naturalness.md) rules, ZK0501 to ZK0511 |
| `spoken` | ZK0510, a word in its [spoken spelling](../language/spoken.md); `--lint-allow spoken` lets a program use them |

A code wins over a group. Between two names of one tier, the later flag wins, and a flag
on the command line wins over a [config file](configuration.md). ZK0709, ZK0710 and
ZK0711 take the level inside unsafe code alone; outside it, the three remain errors. The
doubtful findings (ZK0711, and the leaks the checker cannot prove) show once a name sets
the group `doubts`, as `--lint-warn doubts`. A code outside the table, such as ZK0701 or
ZK0722, keeps its level: those findings keep safe code safe.

```console
$ zakac run --lint-error leaks leak.zaka
error[ZK0708]: the block that "ပ" holds is never given back or handed over on this path
$ zakac run --lint-error mem --lint-allow ZK0708 leak.zaka
၇
```

These belong to `build --backend vm` and to `run`:

| Option | Default | Meaning |
| --- | --- | --- |
| `--results <MODE>` | `in-place` | how a list or struct result reaches the caller: `in-place` builds it in the caller's slot, `duplicate` keeps it in the function's frame and duplicates it on return |
| `--scratch <MODE>` | `apart` | where a list or struct that a statement builds and uses up lives, such as a printed literal or a range passed to a function: `apart` gives each one a slot of its own, `shared` lets a later statement take the slot of one before it |

The program prints the same in each mode. `duplicate` costs one more pass over each
list or struct result. `shared` leaves fewer slots in a frame, and `apart` shows each
scratch value as a block of its own in the explorer's RAM view. See [Lists and structs
as results](../language/functions.md#lists-and-structs-as-results).

`--release` also belongs to each target of `build` and to `run`:

| Option | Default | Meaning |
| --- | --- | --- |
| `--release` | off | build for release: leave out the source region of each op, the embedded source, the imported files and the diagnostics of the artifact, so a stop names the function and the index of the op, not the line; see [Release builds](../vm/running.md#how-a-run-ends) |

`zakac build --backend c --release` writes no table of places and no marks for them, so a stop
names the function alone (`(in addfee)`) and each backtrace line is a function name; it does not
change the flags of the C compiler (`-O2`, no `-g`, with or with no `--release`). The native
target and `zakac run --jit` leave out the same table under `--release`. A `.zkc` or `.zkct` built with no `--release` keeps its
lines, so `zakac run age.zkct` shows them.

## segment

```console
$ zakac segment [--coalesce-non-burmese] [INPUT]
```

Cuts the text into the pieces the lexer starts from: Burmese words, digits, and the other
characters. `--coalesce-non-burmese` joins a run of characters that are not Burmese into
one piece.

```console
$ echo 'abc ၁' | zakac segment --coalesce-non-burmese
```

```text
- kind: non_burmese_char
  lexeme: "abc "
- kind: burmese_digit
  lexeme: ၁
- kind: non_burmese_char
  lexeme: "\n"
```

(The `span` of each piece is left out here.) See [Finding words in
Burmese](../concepts/segmenting.md).

## lex, parse, semcheck, irgen, borrowck

These five take the input and output options and the compile options listed above for
their stage (`lex` and `parse` have none), and nothing else. `lex` segments the text
first, `parse` lexes first, and so on, so a command never needs the artifact of the stage
before it: it starts from the source.

From `parse` on, a command reads the files its input imports as well, and its artifact
holds them under `imports` ([Modules](../language/modules.md)). `segment` and
`lex` work on the one file given to them.

## build

```console
$ zakac build [--backend vm|c|native] [--emit WHAT] [options] [INPUT]
```

`build` runs each stage, then the code generator of a target:

| Target | Makes | `--emit` | Output with no `-o` |
| --- | --- | --- | --- |
| `vm` (the default) | the ops of the VM, which `run` runs | `zkc` (default), `zkct`, `asm` | `<name>.zkc` or `<name>.zkct` beside the source; the listing goes to standard output |
| `c` | an executable, through C and a C compiler; or a C lib | `bin` (default), `c`, `lib` | `<name>` beside the source (`<name>.exe` on Windows); for `lib`, `lib<name>.a`, the shared lib and `<name>.h` |
| `native` | an executable, through Cranelift and the linker of a C compiler | `bin` (default), `obj`, `clif` | `<name>` beside the source (`<name>.exe` on Windows), `<name>.o` or `<name>.clif` |

A source read from standard input writes the ops to standard output. Each target
takes all the compile options above, and the options of its own; an option of another
target, or an `--emit` word of another target, stops the build with an error.

### The VM target

`--emit zkc` writes the raw ops, `zkct` the text form (a header line, then YAML),
and `asm` a listing of the ops. The listing is plain text, for reading; it goes to
standard output, or to the file in `-o`. Here is the listing of a program with a variable
(`age.zaka`), for a 32-bit platform:

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

The ops are listed in [Instructions](../vm/instructions.md).

### The C target

`--backend c` writes a C file for each module of the program, then compiles and links
them with a C compiler; `--emit c` stops after the C files. The C is plain C11, so GCC,
Clang, MinGW and `zig cc` each build it, and so does MSVC on Windows (`--target-abi msvc`,
see [Native programs](native.md#windows)). `--target-os wasi` builds a WebAssembly program
with the Clang of wasi-sdk ([WASI](native.md#wasi)).
[Native programs](native.md) tells what the executable checks, and where its output
differs from the VM.

| Option | Default | Meaning |
| --- | --- | --- |
| `--checks <LIST>` | `default` | the run-time checks of the executable: a level, then checks to add (`+name` or `name`) or drop (`-name`), in order, as a comma list or the flag given again; see below |
| `--max-depth <CALLS>` | `4096` | the most nested calls before the executable stops with `more than N nested calls`, as `--max-depth` of the VM does; counted while the `depth` check is on |
| `--max-heap <OCTETS>` | `0` | the most heap octets the executable holds, no cap when 0: a counted block (text, a plain list, a box, a closure record) past it stops the run with `no heap room for N octets`, as `--max-heap` of the VM does; a raw allocation of unsafe code past it gives the null address; see [Native programs](native.md#checks) |
| `--cache <DIR>` | the cache of the user | the folder for the C files, the objects and the runtime lib (`libzaka_runtime.a`, or `zaka_runtime.lib` for MSVC, which `zakac` carries inside it and writes there); when left out, `zakac/<folder>/<name>-<hash>` in `XDG_CACHE_HOME` or `~/.cache` (`%LOCALAPPDATA%` on Windows), or `.zakac_c_cache` beside the source when that folder cannot be made |
| `--cc <CMD>` | `CC`, then `cc` of a config file, then the first of `cc`, `gcc`, `clang` and `zig cc`; for `--target-abi msvc`, `cl`, `clang-cl`, then the `cl` of Visual Studio; for WASI, `--cc` alone, then `wasm32-wasip1-clang` or `wasm32-wasip2-clang` in the `bin` folder of `WASI_SDK_PATH`, then on the PATH | the C compiler, with the words before its flags |
| `--linker <CMD>` | the C compiler | the program that links the objects, such as `clang -fuse-ld=lld` |
| `--cc-flag <FLAG>` | | one more flag for the C compiler; give it again for more |
| `--ld-flag <FLAG>` | | one more flag for the linker; give it again for more |
| `--no-default-flags` | off | leave out the default C flags `-O2 -w -ffp-contract=off` (`/nologo /O2 /w /fp:precise /utf-8 /MD` for MSVC) |
| `--allow-read[=<PATHS>]`, `--allow-write[=<PATHS>]`, `--allow-env[=<NAMES>]`, `--allow-ffi`, `-A`, `--no-permission-prompt` | none | the rights the executable holds, written into it, as `zakac run` grants them to a run; the executable asks for a right it lacks on a terminal, as the VM does; see [Permissions](../vm/running.md#permissions) |

Each run-time check is one kind of stop. A level sets each check: `none` turns all of
them off (as C, the fastest), `default` turns on the checks of the language, and `mem`
turns on all of them.

| Check | Stops the program at | In |
| --- | --- | --- |
| `divide` | a zero divisor | `default`, `mem` |
| `shift` | a shift as wide as its kind or wider | `default`, `mem` |
| `bounds` | a cell, a place, a slice or a range outside its list or its text | `default`, `mem` |
| `text` | a range of text that cuts a character, and u8s that hold no UTF-8 | `default`, `mem` |
| `step` | a loop step that is not positive | `default`, `mem` |
| `depth` | more nested calls than `--max-depth` | `default`, `mem` |
| `null` | an access through the null address | `mem` |
| `overlap` | a block copied onto a block it overlaps | `mem` |
| `sanitizers` | the address and undefined-behavior sanitizers of the C compiler | `mem` |

The list starts from `default` and takes each item in turn, so a level in it starts over:

```console
$ zakac build --backend c --checks mem,-sanitizers,-depth age.zaka
$ zakac build --backend c --checks none,+bounds age.zaka
$ zakac build --backend c --checks=+null age.zaka
```

Integer overflow is not one of these: `--overflow` picks it.

The flags reach the C compiler in this order: `-std=c11`, the default flags, `CFLAGS`,
then each `--cc-flag`. The linker gets the objects and the runtime lib, `-lm`, the OS libs the runtime lib needs (`-liconv` on macOS, for one), the `-lX` of each lib that the [C functions](../language/functions.md#c-functions) of the program name (but libc and libm; a lib given as a path goes as it is, with a run path), `LDFLAGS`, then each `--ld-flag`. MSVC takes the same order in its own form: `/std:c11` and
its default flags, then `X.lib` for each lib, and the options of the linker after `/link`. A later
flag wins where the compiler keeps the last one, so `--cc-flag -O3` replaces `-O2`. `-w`
silences warnings wherever it stands, so warnings need `--no-default-flags`.

```console
$ zakac build --backend c age.zaka
$ ./age
၂၀
$ CFLAGS=-O3 zakac build --backend c --linker 'clang -fuse-ld=lld' age.zaka
```

#### A C lib

`--emit lib` builds a C lib of the functions marked `"<C name>" အမည်ဖြင့် ပြင်ပမှ ခေါ်ဆိုနိုင်သော`,
for C, Rust and each language that calls C. The program needs no start block, and a
program with none of those functions is an error. The build writes three files in one
folder, named after the input or the last part of `-o`:

| File | What it holds |
| --- | --- |
| `lib<name>.a` (`<name>.lib` for MSVC) | the static lib: the objects of the program with the runtime lib inside it |
| `lib<name>.so`, `lib<name>.dylib` or `<name>.dll` | the shared lib; MSVC writes its import lib as `<name>.dll.lib` |
| `<name>.h` | the declaration of each function, under its C name, and `zk_last_trap` |

Each function returns an `int32_t` status and gives its result through a last
parameter, an address it writes unless it is NULL. The status is `ZK_OK` (0), `1 +` the id
of the trap the call stopped on, or `ZK_STOPPED` (-1) for each call after a trap, since a
trap can leave the heap and the globals half written; `zk_last_trap()` gives the text of
that trap. The first call starts the runtime and sets the globals. Text comes in as a
`const char *`; an address or a handle as a `void *`. A program that links the static lib
links the OS libs of the runtime with it, which the comment at the top of the header
lists.

```console
$ zakac build --backend c --emit lib -o out/mathlib mathlib.zaka
$ cc -Iout main.c out/libmathlib.a -liconv -lm -o main
```

### The native target

`--backend native` lowers the program with [Cranelift](https://cranelift.dev) to an object
file for the target, then links it with the runtime lib through the C compiler, as the C
target links. It runs the same checks as the C target and takes the same options, and
it gives the same output; `--cc-flag`, `--no-default-flags` and `CFLAGS` change nothing, as
there is no C to compile.
[Native programs](native.md#cranelift) tells where it differs.

- `--emit obj` writes the object file alone, and `--emit clif` the Cranelift IR of each
  function, for reading.
- The CPU is `x86_64`, `aarch64` or `riscv64`; the others stop the build with an error.
- An executable is linked for the OS and the CPU of the host alone, with either ABI on
  Windows. `--emit obj` and `--emit clif` take another `--target-os`, `--target-arch` or
  `--target-abi`, to link the object there.
- The `sanitizers` check has no effect: Cranelift code holds no marks for them.

```console
$ zakac build --backend native age.zaka
$ ./age
၂၀
$ zakac build --backend native --emit obj --target-os linux --target-arch x86_64 age.zaka
```

## run

```console
$ zakac run [options] [INPUT] [-- ARGS...]
```

`run` takes the compile options and the two checks above, and it reads the files a
source file imports, as `parse` and the stages after it do. It adds limits on the VM,
which stop a program that goes too far, and two options to look at the use of the heap and the stack:

| Option | Default | Meaning |
| --- | --- | --- |
| `--max-depth <CALLS>` | `4096` | the most nested calls before the program stops |
| `--max-steps <OPS>` | no limit | the most ops to run before the program stops |
| `--max-heap <OCTETS>` | `268435456` | the most live heap octets before an allocation fails |
| `--max-stack <OCTETS>` | `1048576` | the most octets of the stack region, where the locals of each call live |
| `--frames <MODE>` | `checked` | `checked` gives each variable a block of its own; `packed` puts the locals of a call next to each other, as C does |
| `--leaks` | off | list the heap blocks left unfreed when the program ends |
| `--jit` | off | run the program through Cranelift, as code of the host, in place of the VM; see below |
| `--allow-read[=<PATHS>]` | none | let the program read files, list folders and tell what a path holds: each path, or those under the paths of a comma list; see [Permissions](../vm/running.md#permissions) |
| `--allow-write[=<PATHS>]` | none | let it write, add to, remove and move files and make folders |
| `--allow-env[=<NAMES>]` | none | let it read variables of the environment, and give the start block the environment: each variable, or those named |
| `--allow-ffi` | off | let it call C functions |
| `-A`, `--allow-all` | off | grant each of the four |
| `--no-permission-prompt` | off | ask nothing on a terminal: an effect with no right is denied at once |

A program that passes a limit stops with a message from the VM, listed in [When a
program stops](../vm/traps.md). `--frames` is explained in [Checked and packed
blocks](../vm/blocks.md).

The words after `--` are the arguments of the program. Its [start
block](../language/functions.md#the-start-block) gets them as a text list after the name
of the program (the input as written, or `-`), as in C, together with the environment
variables of `zakac`. A program with no start block ignores them.

```console
$ zakac run greet.zaka -- ကက ခခ
```

A program that uses the stdlib module [`"စံကား/အဝင်အထွက်"`](../language/io.md) reads the
standard input of `zakac`, and reads and writes files on disk, relative to the folder the
command runs in:

```console
$ printf 'က\nခ\n' | zakac run echo.zaka
> က
> ခ
စာကြောင်း ၂ ကြောင်း
```

`--leaks` writes one line to standard error for each block that was never given back.
The VM frees each counted block on its own, text, lists and shared objects among them,
and runs the ring collector at the end, so the blocks that show here are those of the
[heap statements](../language/unsafe.md#heap-statements) that unsafe code never gave back:

```console
$ zakac run --leaks leak.zaka
warning[ZK0708]: the block that "ပ" holds is never given back or handed over on this path
၇
leak: block at 0x101000, 8 octets
```

`--jit` lowers the program as `build --backend native` does, with the output fact
`"native"`, and runs it in the `zakac` process. It runs as a native executable runs, with
the checks of the `default` level: `--max-depth` and `--max-heap` set its limits (no heap
cap when `--max-heap` is left out), the rights come from the flags above, and the
arguments from `--`. `--max-steps`, `--max-stack`, `--frames`, `--leaks`, `--results` and
`--scratch` belong to the VM and change nothing under `--jit`. The CPU must be `x86_64`,
`aarch64` or `riscv64`.

```console
$ zakac run --jit age.zaka
၂၀
```

## Config file

Each option can also be set in a TOML config file: `zakac.toml` beside the sources, a file
of the user, `/etc/zakac/config.toml`, or the file `--config <FILE>` names. An option on the
command line wins over each file. [Configuration](configuration.md) tells where the files
are, which one wins, and how each option is named in them; `zakac.example.toml` at the
root of the repo lists each setting with its default.

## Environment

| Name | Meaning |
| --- | --- |
| `ZAKAC_CARETS` | the rows under a marked span on a color terminal: `off` leaves them out, `codepoint` draws them with one cell per code point, `cluster` draws them with a consonant and its `ေ`, `ျ` or `ြ` as two cells, `flow` places them where a terminal that draws Burmese text at its natural width puts the text; when unset, the compiler picks one for the terminal it knows, else `off` |
| `ZAKAC_LEARN` | `true`, `1` or `on` turns on learner mode: each message also explains its code, once a run, and links to the page that teaches it (see [Learner mode](diagnostics.md#learner-mode)) |
| `NO_COLOR` | when it is set, color is off |
| `ZAKA_BACKTRACE` | how much of the stack a stop lists, in `run` and in a native executable: `full` each call, `0` none; when unset or another value, a short backtrace (see [How a run ends](../vm/running.md#how-a-run-ends)) |
| `CC` | the C compiler of `build --backend c` and `--backend native` when `--cc` is left out |
| `CFLAGS`, `LDFLAGS` | flags for the C compiler and the linker of `build --backend c`, split on white space; each `--cc-flag` and `--ld-flag` comes after them; `--backend native` reads `LDFLAGS` |
| `XDG_CACHE_HOME` | the cache folder of the user, where `build --backend c` and `--backend native` keep their files when `--cache` is left out |
| `XDG_CONFIG_HOME` | the config folder of the user, where the commands look for `zakac/config.toml` |
| `ZAKAC_NO_CONFIG` | when it is set, the commands read no config file but the one `--config` names (see [Where the files are](configuration.md#where-the-files-are)) |

A [config file](configuration.md#settings-that-variables-hold) can set `ZAKAC_CARETS`, `ZAKAC_LEARN`, `ZAKA_BACKTRACE`, `NO_COLOR`,
`CFLAGS` and `LDFLAGS` too; a variable that is set wins over the file.

Terminals do not agree on the cells a Burmese sign takes, so a `^^^^` row can drift from
the text it points at. On a color terminal the span is marked inside the source line. The
compiler draws the rows there when it knows how the terminal draws Burmese text, and
leaves them out otherwise; `ZAKAC_CARETS` overrides the choice. Pick `codepoint` for the
VS Code terminal, iTerm or Terminal.app, and `cluster` for Ghostty, kitty or foot.

So far the compiler knows Ghostty. When `TERM_PROGRAM` is `ghostty`, it runs
`ghostty +show-config` and picks the rows from what that lists:

- [saturngod's Ghostty fork](https://ghostty.myanmars.app) lists the setting
  `myanmar-prompt-width`. It draws Burmese text at the natural width of its font and
  pushes the rest of the line right, so the compiler uses `flow`: it shapes the line with
  the Burmese font the fork draws with (the first installed of Noto Sans Myanmar, Myanmar
  MN, Myanmar Text and Padauk) and places the rows under the text. Where none of these
  fonts is installed, the rows are left out.
- Ghostty itself puts Burmese text on its cell grid and gives a grapheme cluster its
  cells, so the compiler uses `cluster`; with `grapheme-width-method = legacy` it uses
  `codepoint`.

Other terminals are measured. The compiler asks the terminal where the cursor is, prints
`ကျော` hidden, asks again, and erases the word. A terminal that gives each code point its
cells moves the cursor four columns and gets `codepoint`; one that gives a grapheme
cluster its cells moves it three and gets `cluster`. The VS Code terminal, iTerm,
Terminal.app, kitty and tmux all go through this measure. A terminal that does not
answer within a tenth of a second, or moves the cursor another distance, gets no rows.

Without color, as in a pipe, the rows are drawn, one cell per code point. See [How a
message is set out](diagnostics.md#how-a-message-is-set-out).

## lsp

`zakac lsp` takes no input and no options: it serves the Language Server Protocol over
standard input and output until the editor shuts it down. [Editors](editors.md) tells
what it shows and how to set it up.

## Exit codes

| Code | Meaning |
| --- | --- |
| 0 | the stage finished, with warnings at most; for `run`, the program ended |
| 1 | an error in the program, or the VM stopped the program |
| 2 | the command line holds a mistake |
| 3 | a file could not be read or written, or a config file holds a mistake, a wrong value among them |
| other | for `run`, the result of the start block, its lowest 8 bits |

Messages go to standard error and the artifact or the program output to standard output,
so `zakac build age.zaka -o -` can be piped while the messages still show.
