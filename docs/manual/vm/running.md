---
title: Running a program
description: How zakac runs a program on the VM: loading, printing, exit codes and the limits of a run.
---

# Running a program

`zakac run` compiles a program and runs it on the VM in one step. This page tells
what the command does, what it prints, how it ends, and which limits it puts on a run.

## From source to a run

```text
 program.zaka ──▶ lex ──▶ parse ──▶ check ──▶ checker ──▶ code ──▶ VM
                                                             ▲
 program.zkc  ──────────────── decode ──────────────────────┘
```

1. **Load.** The command reads the input: a file, or standard input when the name is
   `-` or missing. If the octets begin as the ops of `zakac build` (raw `.zkc`, or
   the text form `.zkct` with its `# zakac-codegen-artifact` header), the command decodes the
   instructions from it. Otherwise it reads the input as Zaka source.
2. **Compile.** Source goes through each stage of the [compiler](../compiler/stages.md).
   A diagnostic of error level stops here: the messages go to standard error, nothing
   runs, and the exit code is 1.
3. **Run.** The VM starts at the first instruction of `main`, which sets the top-level
   variables and calls the start block. It takes one instruction at a time and writes
   what the program prints to standard output as it goes.

## Arguments and the environment

The [start block](../language/functions.md#the-start-block) of a program can take two
text lists: the name of the program (the input as written, or `-` for standard input) and the
words after `--` on the command line, and the environment variables, one `NAME=VALUE`
text for each. Before the first instruction, the VM puts each text in a heap
block of its own and each list in one more, and hands them to hidden top-level
variables. `main` sets the starting values of the top-level variables, calls the start
block with the two lists, and gives the u8s back at its end, as it does for each
top-level variable that holds text. A list with no room on the heap stops the run with
the `out of mem` trap before the first instruction.

```console
$ zakac run greet.zaka -- ကက ခခ
```

The Run buttons of these pages pass the name `main.zaka` and the Explorer `explorer.zaka`,
with no arguments and a blank environment.

To keep the instructions and run them later:

```console
$ zakac build program.zaka
$ zakac run program.zkc
```

The artifact holds the source and the diagnostics too, so a run from it reports the same
warnings as a run from source.

## Checking an artifact

An artifact can be cut or edited after `zakac build` wrote it. Before the first
instruction runs, the VM checks the instructions it decoded:

- Each jump lands on an instruction of its function, and no path runs past the last
  one.
- Where two paths meet, the stack holds the same count of values on both.
- No instruction takes more values than the stack holds, and `ret` leaves the result
  alone on the stack.
- An instruction that reads an address, a boolean or a value of a kind finds a value of
  that sort, where the instructions before it tell the sort.
- Each call, C call and `host` instruction finds the count of arguments its signature
  names.
- Each slot, function, text, static list, format and C call that an instruction names
  exists.
- No kind is larger than the larger of `--max-heap` and `--max-stack`, and the
  top-level variables together fit that size too.

An artifact that fails the check does not run. The command prints
`failed to decode artifact from F: malformed ops in N, op P: R` and exits with 1.
Instructions that `zakac run` compiles from source skip the check.

The check cannot tell whether a C function is declared as the lib defines it: a wrong
declaration can still crash the process, as it can in C. At run time, a value handed to
C must suit its parameter, an address for a pointer and a number for a number, or the
run stops with the `malformed` [trap](traps.md).

## Input, files and the standard streams

The external functions of a program (see
[External functions](../language/functions.md#external-functions)) reach the world the
run is given. `zakac run` gives the terminal: a line read comes from the standard input
of the process, a file is read and written on disk, and a write to `စံပြုအမှား` goes to
standard error. The books and the explorer give a world in storage instead, where the
files a program writes last as long as the run. On a page of the books, the input is set
before the run: the `Standard input` box under the program. In the explorer, the input
remains open: a read with no line makes the run wait (the status reads "Waiting for
input"), and the output panel asks for the line where the output ends, as a terminal
does. Enter gives the program the line, which the panel keeps in the output, and Ctrl+D
ends the input, so the read fails with the end of the input. A write to `စံပြုအထွက်` joins
what the program prints, in order, in each world.

## The options

`zakac run` takes the [compile options](../compiler/commands.md) (`--platform-bits`,
`--fold`, `--strict-particles`, `--mem-check`, `--mem-strict`, `--allow`, `--warn`, `--error`,
`--no-unsafe`, `--overflow`, `--results`, `--scratch`, `--release`) and these
options of the VM:

| Option | Default | Effect |
| --- | --- | --- |
| `--max-depth <CALLS>` | 4096 | the most calls waiting to return, `main` and the start block included |
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
| address | `0x` and lowercase hex digits; the null address as the phrase that writes it, `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` |
| text | itself |
| list | its cells between `[` and `]`, joined with `၊ `; text in a cell prints in quotes |
| member of an enumeration | `<stem>များထဲမှ <member>`, as in `အရောင်များထဲမှ နီ`, through its own instruction `print_enum`; see [Enumerations](../language/enums.md#values-of-an-enumeration) |

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၄၂ ကို ပြသမည်။
-၇ ကို ပြသမည်။
၃.၁၄ ကို ပြသမည်။
မှန် ကို ပြသမည်။
'ခ' ကို ပြသမည်။
"စာသား" ဟု ပြသမည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ပ ကို ပြသမည်။
ကိန်းရှင် စာရင်း၁ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
စာရင်း၁ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၂
-၇
၃.၁၄
မှန်
ခ
စာသား
မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ
[၁၊ ၂၊ ၃]
```

The text a program writes goes out after each instruction, not at the end. When a
program stops on a trap, all that was printed before the trap is written.

## How a run ends

| End | Exit code | What is printed |
| --- | --- | --- |
| `main` returns | 0, or the result of the start block | nothing more, except the `--leaks` list |
| a compile error | 1 | the diagnostics, on standard error |
| a trap | 1 | the reason, on standard error |
| an unreadable input | 1 | the host error, on standard error |

The result of a start block becomes the exit code with its lowest 8 bits, so `-၁`
gives 255. The Explorer shows a code that is not 0 in its status, as `Finished, exit
code 7`.

A trap reads like this:

```console
$ zakac run samples/mistakes/zero_divide.zaka
စားမည်
zaka_vm: the program stopped: integer division or remainder with zero as divisor (in ပရိုဂရမ်, at samples/mistakes/zero_divide.zaka:5:1)
```

The text after `in` names the function, and the place in the source of the instruction
that stopped the run: `at <file>:<line>:<column>`, with the column counted from 1. When
the trap names an address, the message ends with the block that holds it: its start, its
size, and where it came from (`heap`, or the name of a variable). The Run buttons of these
pages and the Explorer show the same reason without the place, as
`the program stopped: ...`. Each reason is explained in [When a program
stops](traps.md).

### Stack backtrace

A stop inside a function that was called tells where the calls came from. When more than
one call is in progress, the lines after the trap are a stack backtrace: each call, the most
recent first. The first line is the call that stopped, at the line that stopped it; each
line after it is a caller, at the line of its call:

```console
$ zakac run calls.zaka
zaka_vm: the program stopped: integer division or remainder with zero as divisor (in စား, at calls.zaka:4:1)
stack backtrace (most recent call first):
  စား at calls.zaka:4:1
  ခွဲ at calls.zaka:10:1
  ပရိုဂရမ် at calls.zaka:14:1
note: set ZAKA_BACKTRACE=full to list each call, or ZAKA_BACKTRACE=0 to leave the calls out
```

Three rules keep the list short:

- A run of the same call, as in a recursion, shows once, then `  ... the call above, N
  times more`.
- Past 16 lines, the rest show as `  ... N calls more`.
- The main function that the compiler adds around the start block is left out, unless the
  stop is in it, as with the start value of a global.

When a rule left a call out, a last line tells how to see it. The variable
`ZAKA_BACKTRACE` sets how much the backtrace lists, as `RUST_BACKTRACE` does for Rust:

| `ZAKA_BACKTRACE` | The backtrace |
| --- | --- |
| not set, or another value | short, as the three rules above keep it |
| `full` | each call, one line each, the compiler's main function too |
| `0` | none: the line of the stop alone |

`backtrace = "full"` in a [config file](../compiler/configuration.md) does the same
when `ZAKA_BACKTRACE` is not set.

```console
$ ZAKA_BACKTRACE=full zakac run calls.zaka
```

### Release builds

A build keeps the source line of each instruction as a rule. `--release` leaves the lines
out, as the release profile of Rust leaves out debug info: the artifact has no source
region for each instruction, no embedded source, no imported files and no diagnostics.
The trap then names the instruction with its index, and each call is a function and an
index:

```console
$ zakac run --release calls.zaka
zaka_vm: the program stopped: integer division or remainder with zero as divisor (in စား, op 3)
stack backtrace (most recent call first):
  စား, op 3
  ခွဲ, op 2
  ပရိုဂရမ်, op 1
note: set ZAKA_BACKTRACE=full to list each call, or ZAKA_BACKTRACE=0 to leave the calls out
```

`zakac run --release` compiles the source with no lines; `zakac build --release` writes
the artifact with none. An artifact built without `--release` keeps its lines, so
`zakac run program.zkct` shows them. See [the commands](../compiler/commands.md).

`--leaks` adds one line on standard error for each heap block still live when `main`
returns and the ring collector has run. It does not change the exit code. A block that
safe code made is counted and freed, so the blocks listed are those of the heap
statements of unsafe code:

```console
$ zakac run --leaks samples/mistakes/leak.zaka
warning[ZK0708]: the block that "ပ" holds is never given back or handed over on this path
၁၀
၂၀
၃၀
leak: block at 0x101000, 16 octets
leak: block at 0x101020, 16 octets
```

The compiler finds most lost blocks on its own ([ZK0708](../reference/errors.md)), and
warns about them. The list of `--leaks` holds the ones it cannot see, such as a block held
in a top-level variable at the end, or one a C function gave.

## Limits

Each limit ends a run that would otherwise go on, with a trap that names which one.

| Limit | Set with | When it is passed |
| --- | --- | --- |
| calls waiting to return | `--max-depth` | the next call traps: `more than 4096 nested calls` |
| instructions run | `--max-steps` | the next instruction traps: `stopped after N ops` |
| live heap octets | `--max-heap` | an allocate statement leaves the null pointer in its place; a resize traps |
| stack region | `--max-stack` | a call whose locals do not fit traps |
| address space | `--platform-bits` | a global or a block with no room traps or fails |

The stack region holds at most `--max-stack` octets, and at most half of the room that
the globals leave, so the heap keeps the other half.

### A heap that runs out

Running out of heap is a normal event of a program, so the allocate statement does not
trap. It leaves the null pointer, `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`, in its place, and a
program can test for that in unsafe code:

```text
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
အကယ်၍ ပ သည် မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ နှင့် ညီလျှင် "နေရာ မရပါ" ဟု ပြသမည်။
သို့မဟုတ်ပါက အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ရပါပြီ" ဟု ပြသမည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

The program asks for 16 octets. With the default cap it prints `ရပါပြီ`. With a heap of 8
octets the block does not fit:

```console
$ zakac run --max-heap 8 program.zaka
နေရာ မရပါ
```

A resize has no safe place to leave the null pointer, since that would lose the old block, so a
resize that finds no room traps instead.

### A platform with few addresses

An 8-bit platform has 256 addresses, and each variable takes at least 32 of them: its
block is aligned to 16, with a gap of 16 after it. Nine globals do not fit:

```console
$ zakac run --platform-bits 8 nine-globals.zaka
zaka_vm: the program stopped: no room in the address space for a global of 1 octets (in main, at nine-globals.zaka:1:1)
```

The code samples of this manual run with a budget of 20 million instructions, and
the Explorer lets a run go on as far as its own settings allow.
