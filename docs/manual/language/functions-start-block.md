---
title: The program start block
description: Declare the program start block, receive arguments and environment variables, and set an exit code.
---

# The program start block

The program start block (entry point, `main`) begins with `ပရိုဂရမ်` ("program"). Its
function signature can declare parameters for command-line arguments and environment
variables, and a return value for the exit code. For a complete example, see
[Where a program starts](../../learn/programs/start.md) in Learn.

## Writing the program start block

```text
ပရိုဂရမ်သည်
<parameters> လက်ခံပြီး                                    (left out when there are none)
အောက်ပါတို့ကို လုပ်ဆောင်မည်။                                (exit code 0)
  or
အောက်ပါတို့ကို လုပ်ဆောင်ကာ <integer kind> တစ်ခု ကို ပြန်ပေးမည်။   (with an exit code)
<statements>
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

Place executable statements inside the program start block or a function. The function
signature can take up to two `စာသား စာရင်း` parameters and return an exit code with
`<integer kind> တစ်ခု ကို ပြန်ပေးမည်`.

| Part | Rule |
| --- | --- |
| end | Start the block closer (end of a block) with `ပရိုဂရမ်`, then write `ပြီးဆုံးပါပြီး`, `ပြီးဆုံးပါပြီ`, `ပြီးပါပြီး` or `ပြီးပါပြီ`. The `လုပ်ဆောင်စရာများ ...` block closer raises ZK0201 here. |
| parameters | none, one or two, each one a `စာသား စာရင်း`, with names that the program chooses |
| the first parameter | the program name, then one text value per command-line argument |
| the second parameter | one `NAME=VALUE` text value per environment variable |
| result | if declared, an integer return value that sets the exit code |

```zaka
ကိန်းရှင် နှုတ်ခွန်း သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မင်္ဂလာပါ" ဖြစ်ပါစေ။

ပရိုဂရမ်သည်
ညွှန်ကြားချက်များ ဟူသော စာသား စာရင်း တစ်ခု ကို လည်းကောင်း
ပတ်ဝန်းကျင် ဟူသော စာသား စာရင်း တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
နှုတ်ခွန်း ကို ပြသမည်။
ညွှန်ကြားချက်များ ၏ အကွက်အရေအတွက် ကို ပြသမည်။
၀ ကို ပြန်ပေးမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ
၁
```

The argument list has at least one cell because its first cell holds the program name.
On the command line, that name is the input path as written, or `-` for standard input.
Arguments come after `--`, and the program inherits the environment of `zakac`. The
Run buttons in these pages use the name `main.zaka` and the Explorer uses
`explorer.zaka`. Both pass no arguments and a blank list of environment variables.

```console
$ zakac run greet.zaka -- ကက ခခ
မင်္ဂလာပါ
၃
$ echo $?
0
```

## How a program runs

1. The top-level variables and constants get their starting values, in order, those of
   imported files first.
2. The program start block runs, with the arguments and the environment.
3. The text that the top-level variables contain is freed, as at the end of each program.

The result of the program start block is the exit code. The host keeps its lowest 8 bits,
so `-၁` ends the run with 255. A program start block with no result, or one that ends with
no return, ends the run with 0. A run that stops on a trap ends with 1, whatever the
program start block returns.

Inside, the program start block follows the rules of a function: it has its own variables,
and a return statement ends it. It can change its two lists as a function can change a list
parameter. The program start block itself cannot be unsafe, so pointer work goes in an
unsafe block (`စိတ်မချရသော်လည်း`) inside it, or in an unsafe function
([Unsafe Zaka](unsafe.md)).

## Where the program start block goes

| Rule | Code |
| --- | --- |
| a program has one program start block, in the file given to `zakac run`; a second one is an error | ZK0325 |
| a program file with no program start block is an error | ZK0327 |
| an imported file has no program start block | ZK0323 |
| the top level of a file has declarations alone: functions, structs, variables, constants and imports; a statement that runs goes in the program start block or a function | ZK0326 |
| each parameter is a `စာသား စာရင်း`, it is not a mutable or writable parameter, and there are two at most | ZK0440 |
| A return value must have an integer data kind (type). A result kind (result type) raises the same error as other non-integer kinds. | ZK0441 |
| the program start block cannot fail, so it has no pass-on ([Errors and results](error-handling.md#the-program-start-block)) | ZK0445 |
| it is declared at the top level, not inside a block | ZK0214 |
| `အများသုံး` does not export it | ZK0229 |

No call can name the program start block.

```zaka-error ZK0326
"ဟိုင်း" ဟု ပြသမည်။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0326]: a statement that runs goes inside the start block, not at the top level
```

```zaka-error ZK0325
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0325]: the program has a start block before this one
```

```zaka-error ZK0327 repl=off
ကိန်းရှင် အမည် သည် "ဇကား" ဖြစ်ပါစေ။
```

```text
error[ZK0327]: the program has no start block, "ပရိုဂရမ်သည်"
```

```zaka-error ZK0440
ပရိုဂရမ်သည်
အရေအတွက် ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0440]: a parameter of the start block is a "စာသား စာရင်း", and there are two of them at most
```

```zaka-error ZK0441
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0441]: the result of the start block is "text", but it must be an integer
```

## REPL mode

REPL mode lets us leave out the program start block. The compiler creates one from the
top-level statements in the file we run. This mode is built into
[`zakac repl`](../compiler/commands.md#repl). For a file, enable it with `--repl` on
`run`, `build`, `irgen` or `borrowck`.

- A statement that runs, at the top level, moves into the program start block, in order.
- A variable or constant moves in with them, as a local of the block, so the values are
  set in source order. A declaration remains at the top level if it has `အများသုံး` or
  if a function or another declaration that remains at the top level uses it.
- Functions, structs, enumerations, traits, kind aliases and imports remain in their places.
- A file with a program start block compiles as it does without the mode, so a statement
  outside the block is still ZK0326. An imported file keeps the rules above, and a file
  with nothing to run gets a blank program start block.

```console
$ zakac run --repl age.zaka
```

With `--repl`, the file `age.zaka` can hold the two lines below and print `၂၀`:

```text
ကိန်းရှင် အသက် သည် ၂၀ ဖြစ်ပါစေ။
အသက် ကို ပြသမည်။
```
