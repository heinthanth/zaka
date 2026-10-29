---
title: Running a program
description: How zakac runs a program on the VM: loading, printing, exit codes and the limits of a run.
---

# Running a program

`zakac run` compiles a program and runs it on the VM in one step. This page tells
what the command does, what it prints, how it ends, and which limits it puts on a run.

## From source to a run

```text
 program.zaka ──▶ lex ──▶ parse ──▶ check ──▶ borrow check ──▶ code ──▶ VM
                                                                  ▲
 program.zkbc  ──────────────── decode ───────────────────────────┘
```

1. **Load.** The command reads the input: a file, or standard input when the name is
   `-` or missing. If the octets begin as a codegen artifact (raw `.zkbc`, or the text
   form with its `# zakac-codegen-artifact` header), the command decodes the
   instructions from it. Otherwise it reads the input as Zaka source.
2. **Compile.** Source goes through each stage of the [compiler](../compiler/stages.md).
   A diagnostic of error level stops here: the messages go to standard error, nothing
   runs, and the exit code is 1.
3. **Run.** The VM starts at the first instruction of `main`, takes one instruction
   at a time, and writes what the program prints to standard output as it goes.

To keep the instructions and run them later:

```console
$ zakac codegen -o program.zkbc program.zaka
$ zakac run program.zkbc
```

The artifact holds the source and the diagnostics too, so a run from it reports the same
warnings as a run from source.

## The options

`zakac run` takes the [compile options](../compiler/commands.md) (`--platform-bits`,
`--fold`, `--strict-particles`, `--mem-check`, `--mem-strict`, `--no-raw`) and these
options of the VM:

| Option | Default | Effect |
| --- | --- | --- |
| `--max-depth <CALLS>` | 4096 | the most calls waiting to return, `main` included |
| `--max-steps <OPS>` | none | the most instructions to run |
| `--max-heap <OCTETS>` | 268435456 (256 MiB) | the most live heap octets |
| `--max-stack <OCTETS>` | 1048576 (1 MiB) | the most octets of the stack region |
| `--frames <MODE>` | `checked` | `checked` or `packed`; see [blocks](blocks.md#checked-and-packed-variables) |
| `--leaks` | off | list the heap blocks never given back when the run ends |

The same command is built as a program of its own, `zaka_vm`, with the same options.

## Printing

A print instruction pops a value and writes it with a line break. The text depends on
the kind of the value:

| Value | Printed as |
| --- | --- |
| integer | the digits `၀၁၂၃၄၅၆၇၈၉`, with `-` for a negative one |
| decimal number | the shortest text that reads back, in Burmese digits: `၃.၁၄`, `inf`, `NaN` |
| character | itself |
| boolean | `မှန်` or `မမှန်` |
| address | `0x` and lowercase hex digits; the null address as `ဗလာ` |
| text in quotes | itself |
| list | its cells between `[` and `]`, joined with `၊ ` |

```zaka
၄၂ ကို ပြသပါ။
-၇ ကို ပြသပါ။
၃.၁၄ ကို ပြသပါ။
မှန် ကို ပြသပါ။
'ခ' ကို ပြသပါ။
"စာသား" ဟု ပြသပါ။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
ပ ကို ပြသပါ။
ကိန်းရှင် စာရင်း၁ သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
စာရင်း၁ ကို ပြသပါ။
```

```text
၄၂
-၇
၃.၁၄
မှန်
ခ
စာသား
ဗလာ
[၁၊ ၂၊ ၃]
```

The text a program writes goes out after each instruction, not at the end. When a
program stops on a trap, all that was printed before the trap is written.

## How a run ends

| End | Exit code | What is printed |
| --- | --- | --- |
| `main` returns | 0 | nothing more, except the `--leaks` list |
| a compile error | 1 | the diagnostics, on standard error |
| a trap | 1 | the reason, on standard error |
| an unreadable input | 1 | the host error, on standard error |

A trap reads like this:

```console
$ zakac run samples/mistakes/zero_divide.zaka
စားမည်
zaka_vm: the program stopped: integer division or remainder with zero as divisor (in main, op 7)
```

The text after `in` names the function and the index of the instruction that stopped the
run. When the trap names an address, the message ends with the block that holds it: its
start, its size, and where it came from (`heap`, or the name of a variable). The Run
buttons of these pages and the Explorer show the same reason without the place, as
`the program stopped: ...`. Each reason is explained in [When a program
stops](traps.md).

`--leaks` adds one line on standard error for each heap block still live when `main`
returns. It does not change the exit code. The warning of the compiler for this program
(ZK0708) is left out below:

```console
$ zakac run --mem-check warn --leaks samples/mistakes/leak.zaka
၁၀
၂၀
၃၀
leak: block at 0x101040, 16 octets
leak: block at 0x101060, 16 octets
```

The compiler finds most lost blocks on its own ([ZK0708](../reference/errors.md)). The
list of `--leaks` is for the blocks that a top-level variable still holds at the end.

## Limits

Each limit ends a run that would otherwise go on, with a trap that names which one.

| Limit | Set with | When it is passed |
| --- | --- | --- |
| calls waiting to return | `--max-depth` | the next call traps: `more than 4096 nested calls` |
| instructions run | `--max-steps` | the next instruction traps: `stopped after N ops` |
| live heap octets | `--max-heap` | an allocate statement leaves `ဗလာ` in its place; a resize traps |
| stack region | `--max-stack` | a call whose locals do not fit traps |
| address space | `--platform-bits` | a global or a block with no room traps or fails |

The stack region holds at most `--max-stack` octets, and at most half of the room that
the globals leave, so the heap keeps the other half.

### A heap that runs out

Running out of heap is a normal event of a program, so the allocate statement does not
trap. It leaves `ဗလာ` in its place, and a program can test for that:

```text
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
အကယ်၍ ပ သည် ဗလာ နှင့် ညီလျှင် "နေရာ မရပါ" ဟု ပြသပါ။
သို့မဟုတ်ပါက အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
"ရပါပြီ" ဟု ပြသပါ။
ပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
```

The program asks for 16 octets. With the default cap it prints `ရပါပြီ`. With a heap of 8
octets the block does not fit:

```console
$ zakac run --max-heap 8 program.zaka
နေရာ မရပါ
```

A resize has no safe place to leave `ဗလာ`, since that would lose the old block, so a
resize that finds no room traps instead.

### A platform with few addresses

An 8-bit platform has 256 addresses, and each variable takes at least 32 of them: its
block is aligned to 16, with a gap of 16 after it. Nine globals do not fit:

```console
$ zakac run --platform-bits 8 nine-globals.zaka
zaka_vm: the program stopped: no room in the address space for a global of 1 octets (in main, op 0)
```

The code samples of this manual run with a budget of 20 million instructions, and
the Explorer lets a run go on as far as its own settings allow.
