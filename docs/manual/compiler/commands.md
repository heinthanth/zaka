---
title: Commands
description: Each zakac command, what it reads, what it writes, and each of its options.
---

# Commands

`zakac` has one command for each stage of the compiler, and one to run a program. These
share one shape:

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
| `semantic-check` | names, kinds and natural wording | the tree and the kind of each expression | `.zksem` / `.zksemt` |
| `irgen` | the IR builder and constant folding | the IR | `.zkir` / `.zkirt` |
| `borrow-check` | the ownership check | the IR and a digest of each function | `.zkbck` / `.zkbckt` |
| `codegen` | the code generator | the ops for the VM | `.zkbc` / `.zkbct` |
| `run` | the VM | nothing: the program runs | |

Each stage also builds as a program of its own (`zakac_lexer`, `zakac_parser`,
`zakac_semcheck`, `zakac_ir_builder`, `zakac_borrow_checker`, `zakac_codegen`,
`zakac_burmese_segmenter`, `zaka_vm`) with the same options as its command.

## A small session

Here is a program that adds up two numbers and prints the sum:

```zaka
၁ + ၂ ကို ပြသပါ။
```

```text
၃
```

Saved as `sum.zaka`, it runs with one command. The second command stops after the code
generator and prints a listing of the ops, not an artifact:

```console
$ zakac run sum.zaka
၃
$ zakac codegen --asm sum.zaka
; zakac asm listing v1, platform 64 bits

fn f0 main (params 0)
     0  push 3:isize
     1  print
     2  ret
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

```console
$ zakac codegen age.zaka -o out/age.zkbc
$ zakac codegen age.zaka -o out/age.zkbct
$ zakac codegen age.zaka -o out/
$ ls out
age.zkbc  age.zkbct
```

A stage writes its artifact even when it finds errors; the errors are in the artifact
too, and the exit code is 1. See [Diagnostics](diagnostics.md).

### What `run` reads

`run` takes a source file, or a `codegen` artifact in either form. It tells them apart
from the first octets of the input, not from the file name:

```console
$ zakac codegen age.zaka -o age.zkbc
$ zakac run age.zkbc
၂၀
$ zakac run age.zaka
၂၀
```

The other artifacts are not programs: `run` reads one of them as source and reports
errors at once. Running an artifact skips the compile, and the checks that were made when
it was written are the checks it carries.

## Options of the compile stages

These options belong to `semantic-check` and the stages after it, and to `run`.

| Option | Default | Meaning |
| --- | --- | --- |
| `--platform-bits <BITS>` | `64` | the bit count of the platform-sized integer kinds, `ကိန်းပြည့်` and `လက္ခဏာမဲ့ ကိန်းပြည့်`: 8, 16, 32, 64 or 128 |
| `--strict-particles` | off | report the [natural wording](../language/naturalness.md) warnings (ZK05) as errors |

These belong to `irgen` and the stages after it, and to `run`:

| Option | Default | Meaning |
| --- | --- | --- |
| `--fold <LIST>` | `all` | the foldings to run: `all`, `none`, or a comma list of `exprs`, `consts` and `branches` |
| `--no-raw` | off | refuse each raw address kind (`လိပ်စာအကြမ်း`) with ZK0717 |

`--fold` changes the IR, not what the program does. `exprs` replaces an operation on
constants with its value, `consts` replaces a read of a `ကိန်းသေ` that was set to a
constant, and `branches` keeps the branch of an `if` that is taken and drops a loop whose
condition is false. Errors in constant operations, such as a division with the constant
zero as divisor (ZK0602), are reported with `--fold none` too.

```console
$ zakac irgen --fold none sum.zaka
```

With `--fold none` the IR of `sum.zaka` keeps `add` with `1` and `2` as its two sides;
without it, the IR holds the constant `3`.

These belong to `borrow-check` and the stages after it, and to `run`. These are explained
in [Pointer checks](checks.md).

| Option | Default | Meaning |
| --- | --- | --- |
| `--mem-check <MODE>` | `error` | how the ownership check reports: `error`, `warn` or `off` |
| `--mem-strict` | off | also report the accesses that the check cannot prove safe |

These belong to `codegen` and to `run`:

| Option | Default | Meaning |
| --- | --- | --- |
| `--results <MODE>` | `in-place` | how a list or struct result reaches the caller: `in-place` builds it in the caller's slot, `duplicate` keeps it in the function's frame and duplicates it on return |
| `--scratch <MODE>` | `apart` | where a list, struct or slice that a statement builds and uses up lives, such as a printed literal or a slice passed to a function: `apart` gives each one a slot of its own, `shared` lets a later statement take the slot of one before it |

The program prints the same in each mode. `duplicate` costs one more pass over each
list or struct result. `shared` leaves fewer slots in a frame, and `apart` shows each
scratch value as a block of its own in the explorer's RAM view. See [Lists and structs
as results](../language/functions.md#lists-and-structs-as-results).

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

## lex, parse, semantic-check, irgen, borrow-check

These five take the input and output options and the compile options listed above for
their stage (`lex` and `parse` have none), and nothing else. `lex` segments the text
first, `parse` lexes first, and so on, so a command never needs the artifact of the stage
before it: it starts from the source.

From `parse` on, a command reads the files its input imports as well, and its artifact
holds them under `imports` ([Modules](../language/modules.md)). `segment` and
`lex` work on the one file given to them.

## codegen

```console
$ zakac codegen [--asm] [options] [INPUT]
```

Besides all the compile options, `codegen` takes `--asm`, which writes a listing of the
ops in place of the artifact. The listing is plain text, and it goes to the file in `-o`
or to standard output. Here is the listing of a program with a variable (`age.zaka`),
for a 32-bit platform:

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
အသက် ကို ပြသပါ။
```

```text
၂၀
```

```console
$ zakac codegen --platform-bits 32 --asm age.zaka
; zakac asm listing v1, platform 32 bits
globals
  g0 အသက် isize

fn f0 main (params 0)
     0  global_addr g0       ; အသက်
     1  push 20:isize
     2  store isize
     3  global_addr g0       ; အသက်
     4  load isize
     5  print
     6  ret
```

The ops are listed in [Instructions](../vm/instructions.md).

## run

```console
$ zakac run [options] [INPUT]
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

`--leaks` writes one line to standard error for each block that was never given back:

```console
$ zakac run --mem-check warn --leaks leak.zaka
၇
leak: block at 0x101020, 16 octets
```

## Exit codes

| Code | Meaning |
| --- | --- |
| 0 | the stage finished, with warnings at most; for `run`, the program ended |
| 1 | an error in the program, or the VM stopped the program |

Messages go to standard error and the artifact or the program output to standard output,
so `zakac codegen age.zaka -o -` can be piped while the messages still show.
