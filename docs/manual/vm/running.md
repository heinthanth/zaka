---
title: Running a program
description: When a program runs on the VM, and how the VM runs it: loading, arguments, rights, printing, exit codes, stop reports and limits.
---

# Running a program

`zakac run` compiles a program and runs it in one step. This page covers when that run
uses the VM (virtual machine), what the VM does with the program from the first operation
(instruction) to the last, what it prints, how a run ends, and the limits it puts on a
run. The first run in Learn is in [Hello, world](../../learn/hello/README.md#running-it).

## When a program runs on the VM

A plain `zakac run` of a source file runs native code when it can and the VM otherwise:
[Cranelift](../compiler/native.md#cranelift) turns the program into machine code of the
host, and `zakac` runs that code in its own process, as a native executable runs. The
other routes to a run use the VM:

| How we run the program | What runs it |
| --- | --- |
| `zakac run program.zaka` | native code from Cranelift when it can; the VM otherwise (the list below) |
| `zakac run --vm program.zaka` | the VM |
| `zakac run --jit program.zaka` | native code alone: a program Cranelift cannot build is an error |
| `zakac run program.zkc`, `zakac run program.zkct` | the VM, since an artifact contains VM operations |
| `zakac run -`, or `zakac run` with no input | the VM, which reads the source from standard input |
| `zaka_vm program.zaka` | the VM: this program has the same options and carries no Cranelift |
| the [Explorer](/explorer/) and the Run buttons of these pages | the VM, built to WebAssembly and running in the browser |
| a call of a comptime function | the VM, inside the compiler ([Runs at compile time](#runs-at-compile-time)) |

A plain `zakac run` of a source file goes to the VM in these cases too:

- the file name does not end in `.zaka`;
- the command has a flag of the VM alone: `--max-steps`, `--max-stack`, `--frames`,
  `--leaks`, `--shuffle-thread-order`, `--results` or `--scratch`;
- the CPU of the host is not one Cranelift builds for (`x86_64`, `aarch64`, `riscv64`),
  `--target-bits` is not the address width of the host, or `--target-os`,
  `--target-arch` or `--target-abi` names another host;
- the program has a construct the native target does not build at present.

The native run keeps `--max-depth`, `--max-heap`, `--trap`, the rights and the
arguments after `--`. [Commands](../compiler/commands.md#run) lists each flag,
and [Checks in the VM and in native code](../concepts/checked.md) tells which checks
native code keeps. We can force the VM with `--vm`:

```console
$ zakac run --vm hello.zaka
```

The rest of this page describes a run on the VM.

## From source to a run

```text
 program.zaka ──▶ lex ──▶ parse ──▶ check ──▶ checker ──▶ code ──▶ VM
                                                             ▲
 program.zkc  ──────────────── decode ──────────────────────┘
```

1. **Load.** The command reads the input: a file, or standard input when the name is
   `-` or missing. When the first octets (bytes) are those of an artifact from
   `zakac build`, the raw `.zkc` form or the text `.zkct` form with its
   `# zakac-codegen-artifact` first line, the command decodes the operations from it.
   Otherwise it reads the input as Zaka source.
2. **Compile.** Source goes through each stage of the [compiler](../compiler/stages.md).
   A diagnostic of error level stops the command here: the messages go to standard
   error, nothing runs, and the exit code is 1.
3. **Run.** The VM starts at the first operation of `main`, which sets the top-level
   variables and calls the program start block (entry point, `main`). It runs one
   operation at a time and writes what the program prints to standard output as it goes.

To keep the operations and run them later, we build an artifact first:

```console
$ zakac build program.zaka
$ zakac run program.zkc
```

The artifact contains the source and the diagnostics too, so a run from it reports the
same warnings as a run from source. A program settled for a target, through a
[compile-time conditional](../language/compile-time.md) or a fact of the target it
names, records that target, and the VM refuses to run it on another OS or CPU.

## Arguments and the environment

The [program start block](../language/functions-start-block.md) can have two text
(string) lists as parameters. The first has the name of the program (the input as
written, or `-` for standard input) and then the words after `--` on the command line.
The second has the environment variables, one `NAME=VALUE` text for each.

```console
$ zakac run greet.zaka -- ကက ခခ
```

Before the first operation, the VM puts each text in a heap block (allocation) and each
list in one more, and passes them to hidden top-level variables. `main` sets the starting
values of the top-level variables, calls the program start block with the two lists, and
frees their u8s at its end, as it does for each top-level variable that has text. When the
heap has no room for a list, the run stops with the `out of mem` trap before the first
operation.

The Run buttons of these pages pass the name `main.zaka`, and the Explorer passes
`explorer.zaka`, with no arguments and a blank environment. With `zakac run`, a program
start block that reads the environment needs the env right ([Permissions](#permissions)):
`--allow-env` passes each variable, and `--allow-env=HOME,LANG` passes those two alone.

## Checking an artifact

An artifact can be cut or edited after `zakac build` wrote it, so before the first
operation runs, the VM checks the operations it decoded:

- Each jump lands on an operation of its function, and no path runs past the last one.
- Where two paths meet, the operand stack has the same count of values on both.
- No operation pops more values than the stack has, and `ret` leaves the result alone on
  the stack.
- An operation that reads an address (pointer), a true/false value or a value of a data
  kind (type) finds a value of that sort, where the operations before it tell the sort.
- Each call, C call and `host` operation finds the count of arguments its signature
  names.
- Each slot, function, text, static list, format and C call that an operation names
  exists.
- No data kind is larger than the larger of `--max-heap` and `--max-stack`, and the
  top-level variables together fit that size too.

An artifact that fails the check does not run: the command prints
`failed to decode artifact from F: malformed ops in N, op P: R` and exits with 3, as for
another input it cannot read. Operations that `zakac run` compiles from source skip the
check.

The check cannot tell whether a C function is declared as its lib defines it, so a wrong
declaration can still crash the process, as it can in C. At run time, a value passed to C
must suit its parameter, an address for a pointer and a number for a number, or the
program stops with the `malformed` [trap](traps.md).

## Input, files and the standard streams

The [external functions](../language/functions-c.md#external-functions) of a program are
host functions: the VM serves them from the world the run is given. `zakac run` gives the
terminal, so a line read comes from the standard input of the process, a file is read and
written on disk, and a write to `စံပြုအမှား` goes to standard error.

The books and the Explorer give a world in storage instead, where the files a program
writes last as long as the run. On a page of the books, the input is set before the run,
in the `Standard input` box under the program. In the Explorer, the input remains open: a
read with no line makes the run wait (the status shows "Waiting for input"), and the
output panel asks for the line where the output ends, as a terminal does. Enter passes the
line to the program, and the panel keeps it in the output, while Ctrl+D ends the input, so
the read fails with the end of the input. In each world, a write to `စံပြုအထွက်` joins
what the program prints, in order.

## Permissions

A run has no right unless its command grants it, as in Deno, and each effect that reaches
the host needs one:

| Right | Effects | Flag |
| --- | --- | --- |
| read | read a file, list a folder, tell what kind of thing a path names | `--allow-read[=<paths>]` |
| write | write or add to a file, remove, move, make a folder | `--allow-write[=<paths>]` |
| env | read a variable, the environment of the program start block | `--allow-env[=<names>]` |
| ffi | call a C function | `--allow-ffi` |

```console
$ zakac run --allow-read=data --allow-write=out report.zaka
```

A flag with no list grants the right on each path or variable. With a list, with commas
between its items, it grants the right on those alone: a path covers the files and
folders under it, and we write it relative to the folder the command runs in, with `.`
and `..` worked out. `-A` (`--allow-all`) grants each right. Reading a line of standard
input and writing to the standard streams need no right.

On a terminal, an effect with no right asks first, as Deno does:

```text
reading "data.txt": allow it? [Y = from now on, O = once, N = no, A = each of this kind]
```

The first letter of the answer counts, in either case:

| Answer | Effect |
| --- | --- |
| `Y` | allows this path, variable or C function for the rest of the run |
| `O` | allows this one effect; the next one asks again |
| `N` | refuses this path, variable or C function for the rest of the run, with no second question |
| `A` | allows the right on each path or variable for the rest of the run |

Each other answer counts as `N`. `--no-permission-prompt` asks nothing, and a run with no
terminal never asks.

A refused read or write of a file, and a refused variable, fail with the error
`ခွင့်ပြုချက်မရှိခြင်း`, which the program can handle as it handles other failures of the
[io module](../language/io.md). A refused C call, and a program start block that reads the
environment with no env right, have no result to fail with, so the program stops with the
`permission denied` [trap](traps.md), which names the flag. The books and the Explorer
run in storage, with no file or variable of the host to reach, so their runs have each
right.

## The options

`zakac run` accepts the [compile options](../compiler/commands.md) (`--target-bits`,
`--fold`, `--lint-allow`, `--lint-warn`, `--lint-error`, `--no-unsafe`, `--debug-asserts`,
`--overflow`, `--results`, `--scratch`, `--release`) and these options of the VM:

| Option | Default | Effect |
| --- | --- | --- |
| `--max-depth <CALLS>` | 4096 | the most calls waiting to return, `main` and the program start block included |
| `--max-steps <OPS>` | none | the most operations to run |
| `--max-heap <OCTETS>` | 268435456 (256 MiB) | the most live heap octets |
| `--max-stack <OCTETS>` | 1048576 (1 MiB) | the most octets of the stack region |
| `--frames <MODE>` | `checked` | `checked` or `packed`; see [Checked and packed variables](blocks.md#checked-and-packed-variables) |
| `--trap <MODE>` | `unwind` | what a trap in a [thread](../language/threads.md#a-trap-in-a-thread) does: `unwind` ends that thread alone, and `abort` stops the whole program |
| `--shuffle-thread-order[=<SEED>]` | off | shuffle the [turns of the threads](../language/threads.md#turns-on-the-vm), from `SEED`; with no `SEED` the run draws one and writes it to standard error |
| `--leaks` | off | list the heap blocks never freed when the run ends |
| `--vm` | off | run on the VM, not as native code |

The `zaka_vm` program has the same options and runs each program on the VM.

## Printing

A print operation pops a value and writes it with a line break. The text depends on the
data kind of the value:

| Value | Printed as |
| --- | --- |
| integer | the digits `၀၁၂၃၄၅၆၇၈၉`, with `-` for a negative one |
| decimal number | the shortest text that reads back as the same number, in Burmese digits: `၃.၁၄`, `inf`, `NaN` |
| character | itself |
| true/false value | `မှန်` or `မမှန်` |
| address | `0x` and lowercase hex digits; the null address as the phrase we write for it, `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` |
| text | itself |
| list | its cells between `[` and `]`, joined with `၊ `; text in a cell prints in quotes |
| member of an enumeration | `<stem>များထဲမှ <member>`, as in `အရောင်များထဲမှ နီ`, through the operation `print_enum`; see [Enumerations](../language/enums.md#values-of-an-enumeration) |

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
ကိန်းရှင် စာရင်း၁ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည်
၁၊ ၂ နှင့် ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
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

The VM writes the text out after each operation, not at the end, so when the program
stops, all that it printed before the stop is there.

## How a run ends

| End | Exit code | What is printed |
| --- | --- | --- |
| `main` returns | 0, or the result of the program start block | nothing more, except the `--leaks` list |
| a compile error | 1 | the diagnostics, on standard error |
| the program stops (trap) | 1 | the reason, on standard error |
| an input that cannot be read | 3 | the host error, on standard error |

The result of a program start block becomes the exit code with its lowest 8 bits, so `-၁`
ends the run with 255. The Explorer shows a code that is not 0 in its status, as
`Finished, exit code 7`.

When the program stops, the VM writes one line with the reason:

```console
$ zakac run --vm samples/mistakes/zero_divide.zaka
စားမည်
zaka_vm: the program stopped: integer division or remainder with zero as divisor (in ပရိုဂရမ်, at samples/mistakes/zero_divide.zaka:5:1)
```

The text after `in` names the function and the place in the source of the operation that
stopped the run, as `at <file>:<line>:<column>`, with the column counted from 1. When the
reason names an address, the line ends with the block that contains it: its start, its
size and where it came from (`heap`, or the name of a variable). The Run buttons of these
pages and the Explorer show the same reason without the place, as
`the program stopped: ...`. [When a program stops](traps.md) explains each reason.

Native code writes the same line with the name of the program in place of `zaka_vm`:
`zero_divide.zaka: the program stopped: ...` from a plain `zakac run`, or the name of
the executable from a native build.

### Stack backtrace

When the program stops inside a function that was called, the report also tells where
the calls came from. With more than one call in progress, the lines after the reason are
a stack backtrace, most recent call first. The first line is the call that stopped, at the
line that stopped it, and each line after it is a caller, at the line of its call:

```console
$ zakac run --vm calls.zaka
zaka_vm: the program stopped: integer division or remainder with zero as divisor (in စား, at calls.zaka:4:1)
stack backtrace (most recent call first):
  စား at calls.zaka:4:1
  ခွဲ at calls.zaka:10:1
  ပရိုဂရမ် at calls.zaka:14:1
note: set ZAKA_BACKTRACE=full to list each call, or ZAKA_BACKTRACE=0 to leave the calls out
```

Three rules keep the list short:

- A run of the same call, as in a recursion, shows once, then
  `  ... the call above, N times more`.
- Past 16 lines, the rest show as `  ... N calls more`.
- The `main` function that the compiler adds around the program start block is left out,
  unless the stop is in it, as with the starting value of a global.

When a rule left a call out, a last line tells how to see it. The variable
`ZAKA_BACKTRACE` sets how much the backtrace lists, as `RUST_BACKTRACE` does for Rust:

| `ZAKA_BACKTRACE` | The backtrace |
| --- | --- |
| not set, or another value | short, as the three rules above keep it |
| `full` | each call, one line each, the compiler's `main` function too |
| `0` | none: the line of the stop alone |

`backtrace = "full"` in a [config file](../compiler/configuration.md) does the same when
`ZAKA_BACKTRACE` is not set.

```console
$ ZAKA_BACKTRACE=full zakac run --vm calls.zaka
```

### Release builds

A build keeps the source line of each operation unless we ask otherwise. `--release`
leaves the lines out, as the release profile of Rust leaves out debug info: the artifact
has no source region for each operation, no embedded source, no imported files and no
diagnostics. The report then names each operation with its index, and each call as a
function and an index:

```console
$ zakac run --vm --release calls.zaka
zaka_vm: the program stopped: integer division or remainder with zero as divisor (in စား, op 4)
stack backtrace (most recent call first):
  စား, op 4
  ခွဲ, op 3
  ပရိုဂရမ်, op 1
note: set ZAKA_BACKTRACE=full to list each call, or ZAKA_BACKTRACE=0 to leave the calls out
```

`zakac run --release` compiles the source with no lines, and `zakac build --release`
writes the artifact with none. An artifact built without `--release` keeps its lines, so
`zakac run program.zkct` shows them. See [Commands](../compiler/commands.md).

### Leaks

`--leaks` adds one line on standard error for each heap block that is still live when
`main` returns and the ring collector has run. It does not change the exit code. Safe
code counts and frees each block it makes, so the blocks listed are those of the heap
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

The compiler finds most lost blocks itself and warns about them
([ZK0708](../reference/errors.md)). The `--leaks` list adds the ones it cannot see, such
as a block that a top-level variable still points at the end, or one that a C function
returned.

## Runs at compile time

The compiler runs each call of a comptime function on this VM when the call sits in a
constant, a count or another spot that needs a value before the program runs. The runner
is `zaka_vm::comptime`, which each command, the language server and the Explorer install.
The compiler builds the calls into a small module with no program start block, and the VM
runs it from its `main` function with:

- a cap of 50 000 000 operations and 64 MiB of heap;
- the rights of the `--comptime-allow-*` flags (none unless granted), with the paths of
  the program read from its folder, no question asked, and each refusal a stop (ZK0611);
- in the language server and the Explorer, a cap of 1 000 000 operations, reads under the
  folder of the program at most, no write and no C call;
- what the calls print kept, as a ZK0612 note at the call.

The runner keeps the results of each small program, along with its text, its arguments
and its rights, so a check that meets the same calls again (the language server, on each
edit) runs nothing. Once the run ends, the runner reads the global of each call back from
its block, following its data kind (`read_tree`), and the globals of the calls get no
drop at the end of `main`. When the program stops, the compiler reports ZK0611 at the
call whose global was being set.

## Limits

Each limit ends a run that would otherwise go on, with a trap that names the limit:

| Limit | Set with | When it is passed |
| --- | --- | --- |
| calls waiting to return | `--max-depth` | the next call stops the program: `more than 4096 nested calls` |
| operations run | `--max-steps` | the next operation stops the program: `stopped after N ops` |
| live heap octets | `--max-heap` | an allocate statement leaves the null address in its place; a resize stops the program |
| stack region | `--max-stack` | a call whose locals do not fit stops the program |
| address space | `--target-bits` | a global or a block with no room stops the program or fails |

The stack region has at most `--max-stack` octets, and at most half of the room that the
globals leave, so the heap keeps the other half. The code samples of this manual run with
a budget of 20 million operations, and the Explorer lets a run go on as far as its own
settings allow.

### A heap that runs out

Running out of heap is a normal event for a program, so the allocate statement does not
stop the program. It leaves the null address, `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`, in its
place, and the program can test for that in unsafe code:

```text
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
အကယ်၍ ပ သည် မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ နှင့် ညီလျှင် "နေရာ မရပါ" ဟု ပြသမည်။
သို့မဟုတ်ပါက အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ရပါပြီ" ဟု ပြသမည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

The program asks for 16 octets. With the default cap it prints `ရပါပြီ`, and with a heap
of 8 octets the block does not fit:

```console
$ zakac run --max-heap 8 program.zaka
နေရာ မရပါ
```

A resize has no safe place to leave the null address, since that would lose the old
block, so a resize that finds no room stops the program instead.

### A platform with few addresses

An 8-bit platform has 256 addresses, and each variable needs at least 32 of them, because
its block is aligned to 16 and has a gap of 16 after it. Nine globals do not fit:

```console
$ zakac run --target-bits 8 nine-globals.zaka
zaka_vm: the program stopped: no room in the address space for a global of 1 octets (in main, at nine-globals.zaka:1:1)
```
