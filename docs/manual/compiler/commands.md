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
| `build` | the code generator of a target | the ops of the VM (`--target vm`), or an executable through C (`--target c`) | `.zkc` / `.zkct` |
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
| `--platform-bits <BITS>` | `64` | the bit count of the platform-sized integer kinds, `ကိန်းပြည့်` and `လက္ခဏာမဲ့ ကိန်းပြည့်`: 8, 16, 32 or 64 |
| `--strict-particles` | off | report the [natural wording](../language/naturalness.md) warnings (ZK05) as errors |

These belong to `irgen` and the stages after it, and to `run`:

| Option | Default | Meaning |
| --- | --- | --- |
| `--fold <LIST>` | `all` | the foldings to run: `all`, `none`, or a comma list of `exprs`, `consts` and `branches` |
| `--no-unsafe` | off | refuse each use of `စိတ်မချရသော`, on a function or a block, with ZK0471 |

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
| `--mem-check <MODE>` | `warn` | how the pointer lint of unsafe code is reported: `error`, `warn` or `off` |
| `--mem-strict` | off | also report the accesses that the checker cannot prove safe (ZK0711) |
| `--overflow <MODE>` | `stop` | what integer arithmetic does with a result outside its kind: `stop` stops the program and warns about an operation that overflows for each value it can see (ZK0719), `wrap` wraps it around; see [Overflow](../language/expressions.md#arithmetic) |
| `--allow <NAME>` | | leave out each finding of a lint group or code; give it again, or a comma list, for more |
| `--warn <NAME>` | | report each finding of a lint group or code as a warning |
| `--error <NAME>` | | report each finding of a lint group or code as an error |

A lint name is a group or a code:

| Group | Codes |
| --- | --- |
| `mem` | the pointer lint that `--mem-check` sets: ZK0702 to ZK0704, ZK0706 to ZK0712, ZK0721 |
| `leaks` | ZK0708 |
| `frees` | ZK0702, ZK0707 |
| `borrows` | ZK0703, ZK0704, ZK0706 |
| `nulls` | ZK0709, ZK0711 |
| `pointers` | ZK0710, ZK0712, ZK0721 |
| `doubts` | ZK0711, ZK0723 |
| `overflow` | ZK0719 |

`--mem-check` sets the level of the whole pointer lint. A group named in `--allow`,
`--warn` or `--error` wins over it, and a code wins over a group. Between two names of one
tier, the later flag wins, and a flag on the command line wins over a [config
file](configuration.md). ZK0709, ZK0710 and ZK0711 take the level inside unsafe code alone;
outside it, the three remain errors. The doubtful findings (ZK0711, and the leaks the
checker cannot prove) show under `--mem-strict` alone: a name sets their level, but does
not turn them on. A code outside the table, such as ZK0701 or ZK0722, keeps its level:
those findings keep safe code safe.

```console
$ zakac run --error leaks leak.zaka
error[ZK0708]: the block that "ပ" holds is never given back or handed over on this path
$ zakac run --error mem --allow ZK0708 leak.zaka
၇
```

These belong to `build --target vm` and to `run`:

| Option | Default | Meaning |
| --- | --- | --- |
| `--results <MODE>` | `in-place` | how a list or struct result reaches the caller: `in-place` builds it in the caller's slot, `duplicate` keeps it in the function's frame and duplicates it on return |
| `--scratch <MODE>` | `apart` | where a list or struct that a statement builds and uses up lives, such as a printed literal or a range passed to a function: `apart` gives each one a slot of its own, `shared` lets a later statement take the slot of one before it |

The program prints the same in each mode. `duplicate` costs one more pass over each
list or struct result. `shared` leaves fewer slots in a frame, and `apart` shows each
scratch value as a block of its own in the explorer's RAM view. See [Lists and structs
as results](../language/functions.md#lists-and-structs-as-results).

`--release` also belongs to `build --target vm`, to `build --target c` and to `run`:

| Option | Default | Meaning |
| --- | --- | --- |
| `--release` | off | build for release: leave out the source region of each op, the embedded source, the imported files and the diagnostics of the artifact, so a stop names the function and the index of the op, not the line; see [Release builds](../vm/running.md#how-a-run-ends) |

`zakac build --target c --release` writes no table of places and no marks for them, so a stop
names the function alone (`(in addfee)`) and each backtrace line is a function name; it does not
change the flags of the C compiler (`-O2`, no `-g`, with or with no `--release`). A `.zkc` or `.zkct` built with no `--release` keeps its
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
$ zakac build [--target vm|c] [--emit WHAT] [options] [INPUT]
```

`build` runs each stage, then the code generator of a target:

| Target | Makes | `--emit` | Output with no `-o` |
| --- | --- | --- | --- |
| `vm` (the default) | the ops of the VM, which `run` runs | `zkc` (default), `zkct`, `asm` | `<name>.zkc` or `<name>.zkct` beside the source; the listing goes to standard output |
| `c` | an executable, through C and a C compiler | `bin` (default), `c` | `<name>` beside the source (`<name>.exe` on Windows) |

A source read from standard input writes the ops to standard output. Each target
takes all the compile options above, and the options of its own; an option of the other
target, or an `--emit` word of the other target, stops the build with an error.

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
$ zakac build --platform-bits 32 --emit asm age.zaka
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

`--target c` writes a C file for each module of the program, then compiles and links
them with a C compiler; `--emit c` stops after the C files. The C is plain C11, so GCC,
Clang, MinGW and `zig cc` each build it, on Windows too, with no Visual Studio.
[Native programs](native.md) tells what the executable checks, and where its output
differs from the VM.

| Option | Default | Meaning |
| --- | --- | --- |
| `--checks <LIST>` | `default` | the run-time checks of the executable: a level, then checks to add (`+name` or `name`) or drop (`-name`), in order, as a comma list or the flag given again; see below |
| `--max-depth <CALLS>` | `4096` | the most nested calls before the executable stops with `more than N nested calls`, as `--max-depth` of the VM does; counted while the `depth` check is on |
| `--max-heap <OCTETS>` | `0` | the most heap octets the executable holds, no cap when 0: a counted block (text, a plain list, a box, a closure record) past it stops the run with `no heap room for N octets`, as `--max-heap` of the VM does; a raw allocation of unsafe code past it gives the null address; see [Native programs](native.md#checks) |
| `--cache <DIR>` | the cache of the user | the folder for the C files and the objects; when left out, `zakac/<folder>/<name>-<hash>` in `XDG_CACHE_HOME` or `~/.cache` (`%LOCALAPPDATA%` on Windows), or `.zakac_c_cache` beside the source when that folder cannot be made |
| `--cc <CMD>` | `CC`, then `cc` of a config file, then the first of `cc`, `gcc`, `clang` and `zig cc` | the C compiler, with the words before its flags |
| `--linker <CMD>` | the C compiler | the program that links the objects, such as `clang -fuse-ld=lld` |
| `--cc-flag <FLAG>` | | one more flag for the C compiler; give it again for more |
| `--ld-flag <FLAG>` | | one more flag for the linker; give it again for more |
| `--no-default-flags` | off | leave out the default C flags `-O2 -w -ffp-contract=off` |

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
$ zakac build --target c --checks mem,-sanitizers,-depth age.zaka
$ zakac build --target c --checks none,+bounds age.zaka
$ zakac build --target c --checks=+null age.zaka
```

Integer overflow is not one of these: `--overflow` picks it.

The flags reach the C compiler in this order: `-std=c11`, the default flags, `CFLAGS`,
then each `--cc-flag`. The linker gets `-lm`, the `-lX` of each lib that the [C functions](../language/functions.md#c-functions) of the program name (but libc and libm; a lib given as a path goes as it is, with a run path), `-lffi` when the build writes `zaka_cb.c` (a program that gives closures to C), `LDFLAGS`, then each `--ld-flag`. A later
flag wins where the compiler keeps the last one, so `--cc-flag -O3` replaces `-O2`. `-w`
silences warnings wherever it stands, so warnings need `--no-default-flags`.

```console
$ zakac build --target c age.zaka
$ ./age
၂၀
$ CFLAGS=-O3 zakac build --target c --linker 'clang -fuse-ld=lld' age.zaka
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

## Config file

Each option can also be set in a TOML config file: `zakac.toml` beside the sources, a file
of the user, `/etc/zakac/config.toml`, or the file `--config <FILE>` names. An option on the
command line wins over each file. [Configuration](configuration.md) tells where the files
are, which one wins, and how each option is named in them.

## Environment

| Name | Meaning |
| --- | --- |
| `ZAKAC_CARETS` | the rows under a marked span on a color terminal: `off` leaves them out, `codepoint` draws them with one cell per code point, `cluster` draws them with a consonant and its `ေ`, `ျ` or `ြ` as two cells, `flow` places them where a terminal that draws Burmese text at its natural width puts the text; when unset, the compiler picks one for the terminal it knows, else `off` |
| `NO_COLOR` | when it is set, color is off |
| `ZAKA_BACKTRACE` | how much of the stack a stop lists, in `run` and in a native executable: `full` each call, `0` none; when unset or another value, a short backtrace (see [How a run ends](../vm/running.md#how-a-run-ends)) |
| `CC` | the C compiler of `build --target c` when `--cc` is left out |
| `CFLAGS`, `LDFLAGS` | flags for the C compiler and the linker of `build --target c`, split on white space; each `--cc-flag` and `--ld-flag` comes after them |
| `XDG_CACHE_HOME` | the cache folder of the user, where `build --target c` keeps its C files when `--cache` is left out |
| `XDG_CONFIG_HOME` | the config folder of the user, where the commands look for `zakac/config.toml` |

A [config file](configuration.md#settings-that-variables-hold) can set `ZAKAC_CARETS`, `ZAKA_BACKTRACE`, `NO_COLOR`,
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
| 2 | the command line holds a mistake, or a config file gives an option a wrong value |
| 3 | a file could not be read or written, or a config file holds a mistake |
| other | for `run`, the result of the start block, its lowest 8 bits |

Messages go to standard error and the artifact or the program output to standard output,
so `zakac build age.zaka -o -` can be piped while the messages still show.
