---
title: Commands
description: Each zakac command, what it reads and writes, and each of its flags with its default.
---

# Commands

`zakac` has one command for each stage (compiler pass), one to build a program for a
target and one to run it. This page lists each command with its flags and defaults.
[Building programs](build.md) covers the C and native targets of `build` in full, and
[Hello, world](../../learn/hello/README.md#running-it) shows the first run.

All the commands share one form:

```text
zakac <command> [flags] [INPUT]
```

`INPUT` is a source file, or `-` for standard input, which is also what we get when we
leave it out. A stage command runs each stage up to its own and writes what that stage
made, the *artifact*, while `run` goes to the end and runs the program.

| Command | Purpose | Writes | Raw / text file |
| --- | --- | --- | --- |
| `segment` | cut the text into pieces | the pieces | `.zkseg` / `.zksegt` |
| `lex` | turn the pieces into tokens | the tokens | `.zktok` / `.zktokt` |
| `parse` | build the tree of sentences | the tree | `.zkast` / `.zkastt` |
| `semcheck` | resolve names and check data kinds (types) and natural wording | the tree and the data kind of each expression | `.zksem` / `.zksemt` |
| `irgen` (or `mkir`) | lower the tree to the IR and fold its constants | the IR | `.zkir` / `.zkirt` |
| `borrowck` (or `mkbck`) | run the flow checks and the pointer lint | the IR and a digest of each function | `.zkbck` / `.zkbckt` |
| `build` | build the program for a target | the operations (instructions) of the VM, an executable or a lib (library) | `.zkc` / `.zkct` for the VM |
| `run` | run the program | nothing: the program runs | |
| `test` | run the [test blocks](../language/tests.md) (unit tests) of files and folders | a line for each test, and a count | |
| `repl` | run statements as we write them, with no program start block | nothing: each statement runs | |
| `lsp` | serve an editor | errors, colors and hovers, to the editor | |
| `fmt` | format source files | each file, in place, or the formatted text | |
| `fetch` | clone the git deps of a [package](../language/packages.md) and pin each one in `zakac.lock` | the lock file | |

[Stages](stages.md) shows what each artifact contains. Each stage also builds as a
program apart from `zakac`, with the same flags as its command: `zakac_burmese_segmenter`,
`zakac_lexer`, `zakac_parser`, `zakac_semcheck`, `zakac_ir_builder`,
`zakac_borrow_checker` and `zaka_vm` (for `run`). So do the targets of `build`
(`zakac_codegen_target_vm`, `zakac_codegen_target_c` and `zakac_codegen_target_native`),
the language server, `zakac_lsp`, and the formatter, `zakafmt`.

## A small session

Here is a program that adds two numbers and prints the sum:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ + ၂ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၃
```

We save it as `sum.zaka` and run it with one command. The second command stops after the
code generator and prints a listing of the VM's operations instead of the operations
themselves:

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

The compiler worked out the sum at compile time, so the VM just pushes `၃`.
[Stages](stages.md) follows this sentence through each stage.

## Flags of each command

| Flag | Value | Default | What it does |
| --- | --- | --- | --- |
| `-h`, `--help` | | | print the flags of the command |
| `-v`, `--version` | | | print the version of `zakac` |
| `--config` | `<FILE>` | | read defaults from this config file, on top of the files the command finds; see [Configuration](configuration.md) |

`zakac --help` lists the commands, and `zakac <command> --help` lists the flags of one.

## Input and output

The stage commands, from `segment` to `borrowck`, all use these:

| Flag | Value | Default | What it does |
| --- | --- | --- | --- |
| `[INPUT]` | a file, or `-` | standard input | the source file |
| `-o`, `--output` | `<FILE>` | standard output | where to write the artifact: a file, a folder, or `-` for standard output |
| `--output-format` | `text` or `raw` | from the output | the form of the artifact |

The command picks the form in this order:

1. `--output-format`, when we give it.
2. Standard output gets `text`, a YAML file we can read.
3. A file name that ends in the text extension of the stage (`.zkirt`) gets `text`, and one
   that ends in the raw extension (`.zkir`) gets `raw`.
4. A folder (one that exists, or a path that ends in `/`) gets a file named after the
   input, with the raw extension, unless we give `--output-format text`.
5. All other file names get `raw`.

Both forms contain the same data: the raw form is compact and quick to load, and the text
form is for reading and for diffs.

`build` follows the same rules for `-o`, with two changes: `--emit` picks what it writes
in place of `--output-format`, and with no `-o` it writes beside the source file (see
[build](#build)).

```console
$ zakac build age.zaka -o out/age.zkc
$ zakac build age.zaka -o out/age.zkct
$ zakac build age.zaka -o out/
$ ls out
age.zkc  age.zkct
```

A stage writes its artifact even when it finds errors. The errors go into the artifact
too, and the exit code is 1 (see [Diagnostics](diagnostics.md)).

### What `run` reads

`run` reads a source file, or the operations that `zakac build` wrote, in either form
(`.zkc` or `.zkct`). It tells them apart from the first octets of the input, not from the
file name:

```console
$ zakac build age.zaka
$ zakac run age.zkc
၂၀
$ zakac run age.zaka
၂၀
```

The other artifacts are not programs, so `run` reads one of them as source and reports
errors at once. Running an artifact skips the compile, and the program keeps the checks
that were made when the artifact was written.

## Options of the compile stages

The flags below steer the stages from `semcheck` on. Each group belongs to the stage named
in its heading, to the stages after it, and to `build`, `run` and `test`.

### From `semcheck` on: the platform, lints and comptime rights

| Flag | Value | Default | What it does |
| --- | --- | --- | --- |
| `--target-bits` | `8`, `16`, `32` or `64` | `32` for `wasm32`, else `64` | the bit count of the platform-sized integers, `ကိန်းပြည့်` and `လက္ခဏာမဲ့ ကိန်းပြည့်` |
| `--target-os` | `windows`, `macos`, `linux` or `wasi` | the host's | the OS that a [compile-time conditional](../language/compile-time.md) sees; `wasi` sets the CPU to `wasm32` when we leave out `--target-arch`, and `--backend c` then builds a WebAssembly program ([WASI](native.md#wasi)) |
| `--target-arch` | `x86_64`, `aarch64`, `x86`, `arm`, `riscv64` or `wasm32` | the host's | the CPU that a compile-time conditional sees |
| `--target-abi` | `gnu`, `msvc`, `apple`, `wasip1` or `wasip2` | the common one of the OS | the ABI that a compile-time conditional sees; see below |
| `--lint-allow` | a group or a code | | leave out each finding of a lint group or code |
| `--lint-warn` | a group or a code | | report each finding of a lint group or code as a warning |
| `--lint-error` | a group or a code | | report each finding of a lint group or code as an error |
| `--comptime-allow-read` | `[=<PATHS>]` | none | let the runs of [comptime calls](../language/compile-time.md#what-a-run-can-do) read files: all of them, or those under a comma list of paths; a path in the program starts from its folder |
| `--comptime-allow-write` | `[=<PATHS>]` | none | let them write, remove and move files and make folders |
| `--comptime-allow-env` | `[=<NAMES>]` | none | let them read environment variables: all of them, or those named |
| `--comptime-allow-ffi` | | off | let them call C functions |
| `--comptime-allow-all` | | off | grant all four |

When we leave out `--target-abi`, the ABI is `apple` on macOS, `gnu` on Linux, `wasip1` on
WASI, and on Windows the ABI that `zakac` itself was built for. The VM and the Cranelift
runs of `zakac run` use the ABI of `zakac` alone, while an executable for Windows can use
`gnu` or `msvc` ([Windows](native.md#windows)).

We give a lint flag again, or a comma list, to name more than one group or code. The
front end reads the groups `naturalness`, `spoken` and `duplicates`, and the checker reads
the rest; [Lint names](#lint-names) lists them.

### From `irgen` on: folding, unsafe code, REPL mode and debug asserts

| Flag | Value | Default | What it does |
| --- | --- | --- | --- |
| `--fold` | `all`, `none`, or a comma list of `exprs`, `consts` and `branches` | `all` | the foldings to run |
| `--no-unsafe` | | off | refuse each use of `စိတ်မချရသော`, on a function or a block, with ZK0471 |
| `--repl` | | off | REPL mode: the statements at the top level of the file that we run form its program start block, so the file can leave the block out ([REPL mode](../language/functions-start-block.md#repl-mode)) |
| `--debug-asserts` | `on` or `off` | `on` | keep the [debug asserts](../language/asserts.md#debug-asserts), `ပရိုဂရမ်ရေးသားချိန်တွင် … ဖြစ်ရမည်`, or drop them with `off`; the check of names and data kinds reads each assert with either value, and a plain assert remains in each build |

`--fold` changes the IR, not what the program does:

- `exprs` replaces an operation on constants with its value. It also turns a text
  (string) literal whose template slots each contain a constant into static text, and so
  for a `+` of constant text. With `--fold none` the program builds that text when it
  runs, with the same output.
- `consts` replaces a read of a `ကိန်းသေ` that was set to a constant.
- `branches` keeps the branch of an `if` that is taken and drops a loop whose condition is
  false.

The compiler reports errors in constant operations, such as a division with the constant
zero as divisor (ZK0602), under `--fold none` too.

```console
$ zakac irgen --fold none sum.zaka
```

With `--fold none`, the IR of `sum.zaka` keeps `add` with `1` and `2` as its two sides;
without it, the IR has the constant `3`.

### From `borrowck` on: overflow

| Flag | Value | Default | What it does |
| --- | --- | --- | --- |
| `--overflow` | `stop` or `wrap` | `stop` | what integer arithmetic does with a result outside its data kind: `stop` stops the program and warns (ZK0719) about an operation that overflows for each value the checker can see, and `wrap` wraps the result around; see [Arithmetic](../language/expressions.md#arithmetic) |

[Compiler checks](checks.md) shows what this flag and the lint flags change.

### For the VM: results and scratch values

These two belong to `build --backend vm`, `run` and `test`, and change how the VM sets out its
frames:

| Flag | Value | Default | What it does |
| --- | --- | --- | --- |
| `--results` | `in-place` or `duplicate` | `in-place` | how a list or struct result reaches the caller: `in-place` builds it in the caller's slot, and `duplicate` keeps it in the function's frame and duplicates it on return |
| `--scratch` | `apart` or `shared` | `apart` | where a list or struct that a statement builds and uses up lives, such as a printed literal or a range passed to a function: `apart` gives each one a slot apart, and `shared` lets a later statement reuse the slot of an earlier one |

The program prints the same in each mode. `duplicate` costs one more pass over each list
or struct result, `shared` leaves fewer slots in a frame, and `apart` shows each scratch
value as a separate block in the explorer's RAM view. See [Lists and structs as
results](../language/functions-results.md#lists-and-structs-as-results).

### Release builds

| Flag | Value | Default | What it does |
| --- | --- | --- | --- |
| `--release` | | off | build for release: leave out the source region of each operation, the embedded source, the imported files and the diagnostics of the artifact |

`--release` belongs to each target of `build`, to `run` and to `test`. A stop in a release build
names the function and the index of the operation instead of the line (see [How a run
ends](../vm/running.md#how-a-run-ends)), while a `.zkc` or `.zkct` built without
`--release` keeps its lines, so `zakac run age.zkct` shows them. For the C and native
targets, `--release` also leaves out the debug info: [Building
programs](build.md#debug-info-and-release) has the details. `--release` keeps the debug
asserts; `--debug-asserts off` drops them.

## Lint names

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
| `duplicates` | ZK0362, a plain value that goes into a [trait value](../language/trait-values.md#changes-through-a-trait-value) whose trait has a function that changes its first parameter |

The rules for two names that meet:

- A code wins over a group.
- Between two names of the same tier, the later flag wins, and a flag on the command line
  wins over a [config file](configuration.md).
- ZK0709, ZK0710 and ZK0711 use the level inside unsafe code alone. Outside it, the three
  keep their level: ZK0709 and ZK0710 are errors, and ZK0711 a warning.
- The doubtful findings (ZK0711, and the leaks the checker cannot prove) show once a name
  sets a level for ZK0711: `doubts`, `nulls`, `mem` or the code itself. So
  `--lint-warn mem` shows more than the default, which sets no name.
- A code that is not in the table, such as ZK0701 or ZK0722, keeps its level, because
  those findings keep safe code safe.

```console
$ zakac run --lint-error leaks leak.zaka
error[ZK0708]: the block that "ပ" holds is never given back or handed over on this path
$ zakac run --lint-error mem --lint-allow ZK0708 leak.zaka
၇
```

## segment

Cut the text into the pieces the lexer starts from: Burmese words, digits and the other
characters.

```text
zakac segment [--coalesce-non-burmese] [-o FILE] [--output-format FORMAT] [INPUT]
```

| Flag | Value | Default | What it does |
| --- | --- | --- | --- |
| `--coalesce-non-burmese` | | off | join a run of characters that are not Burmese into one piece |

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

The `span` of each piece is left out here. See [Finding words in
Burmese](../concepts/segmenting.md).

## lex, parse, semcheck, irgen, borrowck

Run the stages up to the one named, and write its artifact.

```text
zakac lex      [-o FILE] [--output-format FORMAT] [INPUT]
zakac parse    [-o FILE] [--output-format FORMAT] [INPUT]
zakac semcheck [compile flags] [-o FILE] [--output-format FORMAT] [INPUT]
zakac irgen    [compile flags] [-o FILE] [--output-format FORMAT] [INPUT]
zakac borrowck [compile flags] [-o FILE] [--output-format FORMAT] [INPUT]
```

These five use the [input and output](#input-and-output) flags, plus the [compile
flags](#options-of-the-compile-stages) of their stage (`lex` and `parse` have none), and
nothing else. `irgen` also answers to `mkir`, and `borrowck` to `mkbck`.

Each command starts from the source: `lex` segments the text first, `parse` lexes it
first, and so on, so we never need the artifact of the stage before. From `parse` on, a
command also reads the files that its input imports, and its artifact lists them under
`imports` ([Modules](../language/modules.md)). `segment` and `lex` work on the one file we
give them.

## build

Build the program for a target: the operations of the VM, or an executable or a C lib
through C or Cranelift.

```text
zakac build [--backend vm|c|native] [--emit WHAT] [flags] [-o FILE] [INPUT]
```

| Flag | Value | Default | What it does |
| --- | --- | --- | --- |
| `--backend` | `vm`, `c` or `native` | `vm` | the target: the VM, a native executable through C, or one through Cranelift |
| `--emit` | see the table below | `zkc` for `vm`, `bin` for `c` and `native` | what to write |
| `-o`, `--output` | `<FILE>`, a folder, or `-` | beside the input | where to write it; the `asm` listing goes to standard output |

Each target also uses all the [compile flags](#options-of-the-compile-stages) and
`--release`. The VM target adds `--results` and `--scratch`, and the C and native targets
add their own flags, which [Building programs](build.md#flags-of-the-c-and-native-targets)
lists: `--checks`, `--max-depth`, `--max-heap`, `--trap`, `--cache`, `--cc`, `--linker`,
`--cc-flag`, `--ld-flag`, `--no-default-flags`, `--lib-name`, `--runtime-lib` and the
rights (`--allow-read` and the rest). A flag of another target, or an `--emit` word of
another target, stops the build with an error and exit code 3:

```console
$ zakac build --backend c --results duplicate age.zaka
build: --results does not go with --backend c
```

| Target | Makes | `--emit` | Output with no `-o` |
| --- | --- | --- | --- |
| `vm` | the VM's operations, which `run` runs | `zkc` (default), `zkct`, `asm` | `<name>.zkc` or `<name>.zkct` beside the source; the listing goes to standard output |
| `c` | an executable through C and a C compiler, or a C lib | `bin` (default), `c`, `lib` | `<name>` beside the source (`<name>.exe` on Windows); for `lib`, `lib<name>.a`, the shared lib and `<name>.h` |
| `native` | an executable through Cranelift and the linker of a C compiler, or a C lib | `bin` (default), `obj`, `clif`, `lib` | `<name>` beside the source (`<name>.exe` on Windows), `<name>.o` or `<name>.clif`; for `lib`, the same files as the C target |

A source read from standard input writes the operations to standard output.

```console
$ zakac build age.zaka
$ zakac build --emit zkct age.zaka
$ zakac build --backend c age.zaka
$ ./age
၂၀
$ zakac build --backend native age.zaka
```

[Building programs](build.md) covers each target in full: the listing of the VM target,
the flags and checks of the C target, C libs, and the native target.

## run

Run a program: a source file, or the operations that `build` wrote.

```text
zakac run [flags] [INPUT] [-- ARGS...]
```

`run` uses all the [compile flags](#options-of-the-compile-stages), `--overflow`,
`--results`, `--scratch` and `--release`, and reads the files a source file imports, as
`parse` and the stages after it do. It adds these:

| Flag | Value | Default | What it does |
| --- | --- | --- | --- |
| `--max-depth` | `<CALLS>` | `4096` | the most nested calls before the program stops |
| `--max-steps` | `<OPS>` | no limit | the most operations to run before the program stops |
| `--max-heap` | `<OCTETS>` | `268435456` | the most live heap octets before an allocation fails |
| `--max-stack` | `<OCTETS>` | `1048576` | the most octets of the stack region, where the locals of each call live |
| `--frames` | `checked` or `packed` | `checked` | `checked` gives each variable a separate block, and `packed` puts the locals of a call next to each other, as C does |
| `--trap` | `unwind` or `abort` | `unwind` | what a trap in a [thread](../language/threads.md#a-trap-in-a-thread) does: `unwind` ends that thread alone, and a wait for it traps in turn, while `abort` stops the whole program, on the VM and in a Cranelift run alike |
| `--shuffle-thread-order` | `[=<SEED>]` | off | shuffle the [turns of the threads](../language/threads.md#turns-on-the-vm) on the VM; see below |
| `--leaks` | | off | list the heap blocks left unfreed when the program ends |
| `--jit` | | off | run the program as native code through Cranelift, and report a program it cannot build as an error; see [The VM or Cranelift](#the-vm-or-cranelift) |
| `--vm` | | off | run the program on the VM; `--jit` wins over it |
| `--allow-read` | `[=<PATHS>]` | none | let the program read files, list folders and see what a path is: all paths, or those under a comma list; see [Permissions](../vm/running.md#permissions) |
| `--allow-write` | `[=<PATHS>]` | none | let it write, append to, remove and move files and make folders |
| `--allow-env` | `[=<NAMES>]` | none | let it read environment variables, and give the program start block the environment: all variables, or those named |
| `--allow-ffi` | | off | let it call C functions |
| `--allow-run` | `[=<PROGRAMS>]` | none | let it run other programs: all, or those named |
| `-A`, `--allow-all` | | off | grant all five |
| `--no-permission-prompt` | | off | ask nothing on a terminal, so an effect with no right is denied at once |

A program that passes a limit stops with a message, listed in [When a program
stops](../vm/traps.md). [Addresses and blocks](../vm/blocks.md) explains `--frames`.

```console
$ zakac run age.zaka
၂၀
$ zakac run --vm --max-steps 1000 age.zaka
၂၀
```

### Arguments and input

The words after `--` are the arguments of the program. Its [program start
block](../language/functions-start-block.md) gets them as a text list after the name of
the program (the input as written, or `-`), as in C, together with the environment
variables of `zakac`. A program with no program start block ignores them.

```console
$ zakac run greet.zaka -- ကက ခခ
```

A program that uses the stdlib module [`"စံကား/အဝင်အထွက်"`](../stdlib/io.md) reads the
standard input of `zakac`, and reads and writes files on disk relative to the folder the
command runs in:

```console
$ printf 'က\nခ\n' | zakac run echo.zaka
> က
> ခ
စာကြောင်း ၂ ကြောင်း
```

### Shuffled turns of threads

`--shuffle-thread-order` shuffles the turns of the threads on the VM: each turn runs 1 to
200 operations and goes to a thread that can run, and both are drawn from `SEED`. With no
`SEED`, the run draws one and writes it to standard error, and the same seed gives the
same order again. The seed must follow `=`:

```console
$ zakac run --shuffle-thread-order age.zaka
the turns of the threads are shuffled with the seed 16788843531747065664; --shuffle-thread-order=16788843531747065664 runs the same order again
၂၀
$ zakac run --shuffle-thread-order=16788843531747065664 age.zaka
၂၀
```

### Leaks

`--leaks` writes one line to standard error for each block that was never given back.
The VM frees each counted block itself, text, lists and shared objects among them, and
runs the ring collector at the end, so the blocks that show here come from the [heap
statements](../language/heap.md#heap-statements) of unsafe code:

```console
$ zakac run --leaks leak.zaka
warning[ZK0708]: the block that "ပ" holds is never given back or handed over on this path
၇
leak: block at 0x101000, 8 octets
```

## The VM or Cranelift

With neither `--jit` nor `--vm`, `zakac run` runs a source file as native code when it
can, and on the VM otherwise. The native run lowers the program as `build --backend
native` does, with the output fact `"native"`, and runs it inside the `zakac` process with
the checks of the `default` level, as a native executable runs. `--max-depth` and
`--max-heap` set its limits and `--trap` what a trap in a thread does, as for the VM, the
rights come from the flags of [run](#run), and the arguments come after `--`.

The VM runs the program instead when one of these is true:

- we give `--vm`;
- the input is not a file that ends in `.zaka`: an artifact (`.zkc` or `.zkct`), or
  standard input;
- we give a flag that the VM alone has: `--max-steps`, `--max-stack`, `--frames`,
  `--leaks`, `--shuffle-thread-order`, `--results` or `--scratch`;
- the CPU of the host is not one that Cranelift builds for (`x86_64`, `aarch64`,
  `riscv64`), `--target-bits` is not the width of an address on the host, or the program
  has a construct that the native target cannot build at present.

The two runs print the same, but the VM and native code do not catch the same
mistakes. Native code checks what the `default` level of `--checks` checks, so a read through the null address or past
the end of a block in unsafe code does what it does in C, while the VM checks each access
and stops the program at the bad one. To see the VM's checks on such a program, we give
`--vm`. [Checks in the VM and in native code](../concepts/checked.md) sorts each mistake.

`--trap abort` stops the whole program at the first trap of a thread in both runs, with
the reason of that thread. Here `trap.zaka` starts a thread that divides with zero as
divisor, counts to 20000, then prints `ပြီး`:

```console
$ zakac run --vm --trap abort trap.zaka
zaka_vm: the program stopped: integer division or remainder with zero as divisor (in စား, at trap.zaka:3:1)
$ zakac run --trap abort trap.zaka
trap.zaka: the program stopped: integer division or remainder with zero as divisor (in စား, at trap.zaka:3:1)
```

`--jit` asks for Cranelift alone: a program it cannot build is an error, `--max-heap` caps
the heap when we give it and not otherwise, and `--shuffle-thread-order` cannot go with
it. The `zaka_vm` program carries no Cranelift and runs each program on the VM.

## test

Run the [test blocks](../language/tests.md) (unit tests) of files and folders, each in a VM of
its own.

```text
zakac test [flags] [PATH]...
```

Each `PATH` is a file or a folder, and a folder gives each `.zaka` file under it and its
subfolders; the files of all the paths are sorted together. `test` runs the tests of each
file and of each file it imports, each test once, in place of the program start block,
and checks each file as a module, so a file of tests needs no program start block.

`test` uses all the [compile flags](#options-of-the-compile-stages), `--overflow`,
`--results`, `--scratch` and `--release`, and the rights flags of [run](#run):
`--allow-read`, `--allow-write`, `--allow-env`, `--allow-ffi`, `--allow-run`, `-A` and
`--no-permission-prompt`. It adds these:

| Flag | Value | Default | What it does |
| --- | --- | --- | --- |
| `[PATH]...` | files and folders | the paths of `--paths` | where to find the tests |
| `--paths` | `<PATH>`, or a comma list | `tests` | the files and folders to read when the command line names none |
| `--filter` | `<WORD>` | | run the tests whose name, `<file>::<test>`, holds the word, and no other |
| `--jit` | | off | run each test as native code made with Cranelift, in a process of its own, as `run --jit` runs a program |
| `--max-steps` | `<OPS>` | `100000000` | the most operations one test runs on the VM; a test that runs more stops and fails |
| `--time-limit` | `<SECONDS>` | `60` | the most seconds one test runs with `--jit`, where no operations are counted; a test that runs longer is killed and fails |

```console
$ zakac test
test src/math.zaka::နှစ်ဆ ... ok
test tests/math_test.zaka::သုည ... ok
test tests/math_test.zaka::ဆယ် ... ok

test result: ok. 3 passed, 0 failed
$ zakac test --filter သုည tests
test tests/math_test.zaka::သုည ... ok

test result: ok. 1 passed, 0 failed
```

A test that fails shows `FAILED`, then what it printed and its stop, indented with two
spaces. The exit code is 0 when each test passed and 1 when a test failed or a file did
not compile; a path that does not exist gives 3 before a test runs. `--max-steps` does
not hold under `--jit`, and `--time-limit` does not hold on the VM.
[Tests](../language/tests.md#the-output) shows the whole output.

In a config file, the `[test]` section holds these flags, as `paths = ["tests", "src"]`,
`filter = "math"`, `jit = true`, `max-steps = 100000000` and `time-limit = 60`.

## repl

Run statements as we write them, as a REPL (read-eval-print loop) does.

```text
zakac repl [--allow-read[=PATHS]] [--allow-write[=PATHS]] [--allow-env[=NAMES]] [-A]
```

`zakac repl` compiles in [REPL mode](../language/functions-start-block.md#repl-mode), so
we write statements with no program start block. Each line runs once, on one VM that
lives through the whole session, so a line sees the values that the lines before it left,
and its output is printed as it comes. A line that is one expression, such as `က + ၁`,
prints its value, as `က + ၁ ကို ပြသမည်။` would. A line that leaves a block open, such as the
head of an `အကယ်၍` block, waits for the lines after it, under a `...` prompt; a blank
line ends the block as it stands. A statement that does not compile, or that stops the
program, prints its message and is dropped: the variables keep the values from
before it, and the statements after it run as before.

On a terminal, the arrows move through the line and through the lines of earlier
sessions, Ctrl-R searches them, and Ctrl-C drops the lines of an open block. `zakac`
keeps those lines in `zakac/repl_lines` in the data folder of the user, such as
`~/.local/share` (`XDG_DATA_HOME`) on Linux and macOS. With input from a pipe or a file,
the lines are read one after another, with no editing.

A line that starts with `:` is a command:

| Command | What it does |
| --- | --- |
| `:exit` | end the session, as Ctrl-D does |
| `:clear` | clear the screen; the statements so far remain |
| `:reset` | forget the statements so far, so the session starts again |

```console
$ zakac repl
Zaka REPL. Write statements and press Enter. :exit or Ctrl-D leaves, :clear clears the screen and :reset forgets the statements so far.
> ကိန်းရှင် က သည် ၁ ဖြစ်ပါစေ။
> က ကို ပြသမည်။
၁
> က ကို က + ၁ ဟု ထားမည်။
> က + ၁
၃
```

The variables of the session live in globals of the VM, which the VM keeps from one line
to the next. A line that changes the shape of the program so much that the VM cannot go
on runs the whole session again on a new VM and prints the part of the output that is new.
That happens when a line declares a function after the session made a closure or ran a
parallel loop, and when a line imports a module.

A line reads standard input and works with files and the environment as `zakac run` does,
with the same rights: `--allow-read`, `--allow-write`, `--allow-env` and `-A` give them,
and on a terminal a line that needs a right asks for it (see
[Permissions](../vm/running.md#permissions)). With input from a pipe, a line that reads
standard input takes the line after it. When the whole session runs again, the lines
before the new one run in storage: their writes do not reach the disk a second time, and
a read of standard input or of a file in them gives the end of the input or an error, so
a variable that took such a read holds that result from then on. A session cannot call C,
since a run again would call it twice.

## lsp

Serve the Language Server Protocol to an editor.

```text
zakac lsp [--config FILE]
```

`zakac lsp` reads no input file. It serves the protocol over standard input and output
until the editor shuts it down. [Editors](editors.md) shows what it does and how to set it
up.

## fmt

Format source files: the space between words, the line breaks and the indent.

```text
zakac fmt [flags] [PATH...]
```

`zakac fmt` rewrites each file that a path names in place, and walks a folder and its
subfolders for `.zaka` files. With no path, or with `-`, it reads standard input and
writes the formatted text to standard output, so an editor can pipe a file through it. The
same command builds as the program `zakafmt`, and the editors and the
[compiler explorer](/explorer/) run the same code (see [Editors](editors.md#formatting)).

The formatter prints each word of the file as it is written and changes the space
between words: it never splits or joins a word, since the end of a word changes how the
lexer reads it. It checks its output through a second lex, and when the words differ it
writes nothing and reports a fault of the formatter. A file that does not parse is left as
it is, with its errors shown, and the exit code is then 1. A file that imports other files
is read with them, as a build reads it, so a call written as a sentence parses as it
should.

These rules hold for each file:

- One space between two words that the file keeps apart, and none between joined words.
- A line break that the author put inside a statement is kept, except in a struct, an
  enumeration or a function signature, whose lines the formatter sets (below).
- No space at the end of a line, and one line break at the end of the file.
- A comment on a line of its own takes the depth of the statement after it, and a
  comment after code has one space before it.

| Flag | Default | What it does |
| --- | --- | --- |
| `--check` | off | write nothing: list the files that would change, and exit with 1 when one would |
| `--indent <SPACES>` | `0` | spaces for each level of block depth; `0` keeps each line flush left |
| `--continuation-indent <SPACES>` | `preserve` | the later lines of one statement: `preserve` keeps the spaces the author wrote, and a number sets that count past the depth of the statement |
| `--blank-lines-max <LINES>` | `1` | the longest run of blank lines that is kept |
| `--one-statement-per-line <BOOL>` | `true` | a line break after `။` when two statements share a line |
| `--mark-order <BOOL>` | `true` | the Burmese marks in their canonical order, the asat ahead of the dot below |
| `--speech-mode <MODE>` | `written` | each reserved word in its `written` spelling (`ထားမည်`) or its `spoken` one (`ထားမယ်`) |
| `--digits <SCRIPT>` | `preserve` | the digits of number literals: `preserve`, `burmese` (`၁၀`) or `latin` (`10`) |
| `--import-order <ORDER>` | `preserve` | `sorted` sorts each run of imports at the head of a file: the stdlib first, then the others, each group in the order of its path |
| `--max-width <COLUMNS>` | `100` | the widest line that `fit` allows, in characters, as an editor counts its columns: a Burmese mark counts as one, as a letter does |
| `--struct-lines <LINES>` | `fit` | the lines of a struct: `preserve`, `split` or `fit` |
| `--enum-lines <LINES>` | `fit` | the lines of an enumeration: `preserve`, `split` or `fit` |
| `--signature-lines <LINES>` | `fit` | the lines of a function signature: `preserve`, `split` or `fit` |
| `--inline-block-lines <LINES>` | `fit` | the line of a block written as one statement: `preserve`, `split` or `fit` |
| `--wrap-long-lines <BOOL>` | `true` | wrap each other line wider than `--max-width` |

`--speech-mode` changes reserved words and the ends of calls, never names, strings or
comments. A spoken spelling that stands for two written words, as `ရှိတဲ့` stands for
`ရှိသော` and `ရှိသည့်`, becomes the word the parser read in that place. The spoken
spellings are an error unless the lint allows them (see
[Spoken Burmese](../language/spoken.md)). `--digits` leaves a literal with letters, such
as one in base 16 or with an exponent, as it is. With `--import-order sorted`, a blank
line or a comment on a line of its own ends a run of imports, so the groups the author
made remain apart, and the `///` comments above an import move with it.

### The lines of a declaration

A struct, an enumeration and a function signature each have a setting of their lines. With `fit`, the
default, a declaration that fits in `--max-width` goes on one line, and the line breaks
written inside it go; one that does not fit is broken into its lines. With `split` it is
broken into its lines, short or long, and with `preserve` it keeps the lines as written.
A declaration with a comment inside keeps its lines, since a comment ends its line.

A broken struct puts its head on a line, each field on a line of its own and its end on
the last line; a separator, `၊` or `နှင့်`, ends its line. An enumeration
puts each member on a line the same way.

```text
ခရီး ဟူသော ဒေတာအစုအဝေးတွင်
အကွာ ဟူသော ကိန်းပြည့် တစ်ခု၊
ညွှန် ဟူသော အမိန့် တစ်ခု နှင့်
အဆင့်တန်း ဟူသော အဆင့် တစ်ခု
တို့ပါဝင်သည်။
```

A broken function signature puts its name on a line, each parameter on a line of its
own, with `လက်ခံပြီး` after the last one, and the opener with the result on the last line:

```text
ရှာ ဟူသော အများသုံးနိုင်သော လုပ်ဆောင်ချက် သည်
ပိုင်း ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လည်းကောင်း
စာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ထဲတွင်" ကို သုံး၍ လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ လက္ခဏာမဲ့ ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
```

A line that is still too wide breaks again: the line of the opener breaks after
`အောက်ပါတို့ကို လုပ်ဆောင်ကာ`, so the data kind of the result starts a line of its own; a
data kind still too wide breaks before its `သို့မဟုတ်`; and a line still too wide after
that breaks at the last space that fits.

```text
စမ်းသပ်ပြောင်းလဲ ဟူသော အများသုံးနိုင်သော လုပ်ဆောင်ချက် သည်
စ ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
စာလုံး
သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာလုံးဖတ်ခြင်းအမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
```

A block written as one statement after its condition or its loop head, as in
`အကယ်၍ <condition> လျှင် <statement>။`, has `--inline-block-lines`. With `fit`, the statement goes
on a line of its own when the line that holds it is too wide, and back after its condition
when the two fit on one line; the lines of the condition remain as written.

```text
အကယ်၍ အရေအတွက် == ၀ လျှင်
စာလုံးဖတ်ခြင်းအမှားအယွင်းများထဲမှ ဗလာစာသား ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
```

Each other line wider than `--max-width` wraps, at the place that reads best and keeps the
line within the width, best first: the end of a clause (after a reserved word that ends in
`၍`, `ကာ`, `ပြီး` or `သည်`, after `လျှင်`, or before the opener of a block), after a `၊`
between the items of a list, and last a space after a reserved word or a closing mark, so a
value keeps the particle after it. A wrap never leaves the last word of a statement alone
on its line, so a line with no good place, such as one long string and its verb, remains as
it is. The wrap adds line breaks and never removes the author's, and a comment after
code keeps its line and takes no part in the width.

```text
ကိန်းရှင် ဂရစ် သည် အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည်
(၁၊ ၂ နှင့် ၃ တို့ပါသော စာရင်း) နှင့် (၄၊ ၅ နှင့် ၆ တို့ပါသော စာရင်း) တို့ပါသော စာရင်း ဖြစ်ပါစေ။
```

A line the formatter starts takes the depth of its statement, and the count of
`--continuation-indent` when it gives one. The formatter breaks a line between two words
alone, never inside a word or before a word joined to the one before it.

Each flag is also a setting of the `[fmt]` section of `zakac.toml`, such as
`indent = 2`, and the editors read that section too.

```console
$ zakac fmt --check samples
samples/basics/rough.zaka
$ zakac fmt samples
```

## fetch

Clone each git dep of a [package](../language/packages.md) and of its deps into the cache
folder, and pin each one to a commit in `zakac.lock`, beside the `zakac.toml` of the
package.

```text
zakac fetch [PATH]
```

`PATH` is a folder of the package or its `zakac.toml`, and the working folder when we
leave it out; the nearest `zakac.toml` from there upward names the package. `fetch` walks
the path deps and the git deps of the package, and the deps of those, and runs `git` for
each git dep that the lock does not pin, or whose checkout left the cache. A checkout
that the lock pins and the cache holds remains as it is, so a second `fetch` reaches the
network for new revs alone.

```console
$ zakac fetch
fetched json: https://example.org/zaka-json.git at v1.2.0, commit 1a97de1ae314
git checkouts pinned in zakac.lock: 1
```

The checkouts go in `zakac/git/<repo>-<hash>/<commit>` under the cache folder of the user,
or under the folder that `ZAKAC_CACHE` names. `build`, `run`, `test` and `lsp` read the
lock and the cache and never run `git`. `git` must be on the `PATH`; a folder with no
`zakac.toml` above it, or a clone or a checkout that fails, gives exit code 3 with the
reason.

## Config file

We can also set each flag in a TOML config file: `zakac.toml` beside the sources, a file of
the user, `/etc/zakac/config.toml`, or the file that `--config <FILE>` names. A flag on the
command line wins over each file. [Configuration](configuration.md) shows where the files
are, which one wins and how each flag is named in them, and `zakac.example.toml` at the
root of the repo lists each setting with its default.

## Environment

| Variable | What it does |
| --- | --- |
| `ZAKAC_CARETS` | the rows under a marked span on a color terminal: `off`, `codepoint`, `cluster` or `flow`; when unset, the compiler picks one for the terminal, else `off` (see [Carets on a terminal](diagnostics.md#carets-on-a-terminal)) |
| `ZAKAC_LEARN` | `true`, `1` or `on` turns on learner mode: each message also explains its code, once a run, and links to the page that teaches it (see [Learner mode](diagnostics.md#learner-mode)) |
| `NO_COLOR` | when it is set, color is off |
| `ZAKA_BACKTRACE` | how much of the stack a stop lists, in `run` and in a native executable: `full` lists each call and `0` none; unset or another value gives a short backtrace (see [How a run ends](../vm/running.md#how-a-run-ends)) |
| `CC` | the C compiler of `build --backend c` and `--backend native` when we leave out `--cc` |
| `CFLAGS`, `LDFLAGS` | flags for the C compiler and the linker of `build --backend c`, split on white space; each `--cc-flag` and `--ld-flag` comes after them, and `--backend native` reads `LDFLAGS` |
| `XDG_CACHE_HOME` | the cache folder of the user, where `build --backend c` and `--backend native` keep their files when we leave out `--cache`, and where `fetch` keeps its checkouts |
| `ZAKAC_CACHE` | the folder where `fetch` keeps the checkouts of git deps, and where the other commands look for them, in place of the cache folder of the user |
| `XDG_CONFIG_HOME` | the config folder of the user, where the commands look for `zakac/config.toml` |
| `ZAKAC_NO_CONFIG` | when it is set, the commands read no config file except the one `--config` names (see [Where the files are](configuration.md#where-the-files-are)) |
| `LC_ALL`, `LC_MESSAGES`, `LANG`, `LANGUAGE` | the language of the messages (see [The language of messages](diagnostics.md#the-language-of-messages)) |

A [config file](configuration.md#settings-that-variables-hold) can set `ZAKAC_CARETS`,
`ZAKAC_LEARN`, `ZAKA_BACKTRACE`, `NO_COLOR`, `CFLAGS` and `LDFLAGS` too, and a variable
that is set wins over the file.

## Exit codes

| Code | Meaning |
| --- | --- |
| 0 | the stage finished, with warnings at most; for `run`, the program ended |
| 1 | an error in the program, or the program stopped; for `test`, a test failed |
| 2 | a mistake on the command line, such as an unknown flag or a value the flag does not accept |
| 3 | a file could not be read or written, a config file has a mistake (a wrong value among them), a flag does not go with the target of `build`, or `fetch` could not run `git` |
| other | for `run`, the result of the program start block, its lowest 8 bits |

Messages go to standard error, and the artifact or the program's output go to standard
output, so we can pipe `zakac build age.zaka -o -` while the messages still show.
