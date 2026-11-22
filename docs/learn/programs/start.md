---
title: Where a program starts
description: The program start block, the arguments a program gets on the command line, its environment and its exit code.
---

# Where a program starts

Each program so far has started in its program start block (entry point, `main`), the
block from `ပရိုဂရမ်သည်` to `ပရိုဂရမ် ပြီးဆုံးပါပြီး။`. This page shows what else that
block can do: it can receive the command line that the program was run with, and it can
return an exit code that tells the shell how the run went.

## You will learn

- How the program start block receives the arguments of the command line.
- How to return an exit code.
- How to read the environment variables of the run.
- Where the sentences that run must go.

## The arguments of the command line

This program greets each name that we give it on the command line, and when we give it
no names, it explains how to use it:

```zaka
/// Greet one person.
နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည် အမည် ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ {အမည်}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည်
အမည်များ ဟူသော စာသား စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ အမည်များ ၏ အကွက်အရေအတွက် သည် ၁ နှင့် ညီလျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"အသုံးပြုပုံ: {အမည်များ ၏ ၁ ခုမြောက်အကွက်} <အမည်>..." ဟု ပြသမည်။
၁ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
၂ မှ အမည်များ ၏ အကွက်အရေအတွက် အထိ နံပါတ် အနေဖြင့် ရေတွက်၍
နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက်ကို အမည်များ ၏ နံပါတ် ခုမြောက်အကွက် ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အသုံးပြုပုံ: main.zaka <အမည်>...
```

The Run button gives the program its name, `main.zaka`, and no other arguments, so the
program prints how to use it (`အသုံးပြုပုံ`, "usage"). On the command line, we write the
arguments after `--`:

```console
$ zakac run greet.zaka -- မောင်မောင် အေးအေး
မင်္ဂလာပါ မောင်မောင်
မင်္ဂလာပါ အေးအေး
```

The program start block works like a function with no name. In place of
`<name> ဟူသော လုပ်ဆောင်ချက် သည်`, we start it with `ပရိုဂရမ်သည်`, and the rest of the
function signature follows:

```
ပရိုဂရမ်သည်
<name> ဟူသော စာသား စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
<sentences>
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

| Zaka | Reads as | What it does |
| --- | --- | --- |
| `ပရိုဂရမ်သည်` | "the program" | starts the program start block |
| `အမည်များ ဟူသော စာသား စာရင်း တစ်ခု ကို လက်ခံပြီး` | "receives a list of text called `အမည်များ`" | the arguments, one text for each word |
| `အောက်ပါတို့ကို လုပ်ဆောင်ကာ` | "does the following, and" | the sentences follow |
| `ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်` | "will return an integer" | the exit code |
| `ပရိုဂရမ် ပြီးဆုံးပါပြီး။` | "the program is done" | the block closer, in place of `လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။` |

We can pick a different name than `အမည်များ` for the list. The first cell has the name of
the program, as in C, Rust and Python, and the arguments start at the second cell, which
is the reason the counted loop starts at `၂`. So the list has at least one cell, even
when we give no arguments.

## The exit code

```
<number> ကို ပြန်ပေးမည်။
```

A shell reads the exit code of each program it runs: `0` means that all went well, and
other numbers mean that something failed. Our program returns `၁` when it gets no names:

```console
$ zakac run greet.zaka
အသုံးပြုပုံ: greet.zaka <အမည်>...
$ echo $?
1
```

The return also ends the program start block at once, the same as in a function. When
the block has no result, or reaches its end without a return, the program ends with `0`.

## The environment

```
ပရိုဂရမ်သည်
<name> ဟူသော စာသား စာရင်း တစ်ခု ကို လည်းကောင်း <name> ဟူသော စာသား စာရင်း တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
```

A second parameter receives the environment variables of the run, as one `NAME=VALUE`
text for each variable. This program prints their count:

```text
ပရိုဂရမ်သည်
အမည်များ ဟူသော စာသား စာရင်း တစ်ခု ကို လည်းကောင်း ပတ်ဝန်းကျင် ဟူသော စာသား စာရင်း တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပတ်ဝန်းကျင် ၏ အကွက်အရေအတွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

The Run buttons of these pages give a blank environment. `zakac run` passes the
environment just to a program that has the right, which we grant with `--allow-env`.
Without the flag, the program stops before its first sentence, and the message names the
flag.

## Sentences outside the block

The top level of a file, outside each block, is for declarations: functions, structs,
variables and constants. When the program starts, the top-level variables get their
first values, and then the program start block runs. A sentence that does something, such
as a print, must go inside the program start block or a function:

```zaka-error ZK0326
"ဟိုင်း" ဟု ပြသမည်။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0326]: a statement that runs goes inside the start block, not at the top level
```

## Details

- **One program start block.** A program has one program start block, in the file that we
  run: a second one is ZK0325, a file with none is ZK0327, and an imported file cannot
  have one ([The start
  block](../../manual/language/functions-start-block.md)).
- **REPL mode.** `zakac repl` runs statements as we write them, and `zakac run --repl`
  runs a file in the same mode: the statements at the top level form the program start
  block, so we can leave the block out ([REPL
  mode](../../manual/language/functions-start-block.md#repl-mode)).
- **The parameters.** The block can have none, one or two parameters, and each one must
  be a `စာသား စာရင်း`; other parameters are ZK0440.
- **The result.** The result must be an integer, or the compiler reports ZK0441. The
  OS keeps the lowest 8 bits of it, so `-၁` ends the run with 255, and a run that stops
  at run time ends with 1 ([The start
  block](../../manual/language/functions-start-block.md)).
- **No failure to pass on.** The program start block cannot fail, so it cannot pass a
  failure on, and it reads the whole result of a call that can fail instead ([Passing a
  failure on](../error-handling/passing.md)).
- **Other block closers.** We can also end the block with `ပရိုဂရမ် ပြီးဆုံးပါပြီ။`,
  `ပရိုဂရမ် ပြီးပါပြီး။` or `ပရိုဂရမ် ပြီးပါပြီ။` ([Four closing
  forms](../variables/scope.md#four-closing-forms)).
- **The name in the first cell.** With `zakac run`, the first cell has the path of the
  program as we wrote it, and the Explorer gives the name `explorer.zaka`.

## Recap

- The program start block works like a function with no name, from `ပရိုဂရမ်သည်` to
  `ပရိုဂရမ် ပြီးဆုံးပါပြီး။`.
- A `စာသား စာရင်း` parameter receives the command line: the name of the program first,
  then the arguments.
- An integer result is the exit code, and a block with no result ends the run with `0`.
- A second `စာသား စာရင်း` parameter receives the environment, with `--allow-env`.
- The top level has declarations alone, and sentences that run go inside a block.

## Hands on

- Run the greeting program with `zakac run` and three names, then with none, and print
  the exit code each time.
- Change the program so that it returns `၂` when it gets more than five names.
- Print each environment variable of a run with `--allow-env`, one on each line.

Next: [Copies and storage](../forms/README.md).
