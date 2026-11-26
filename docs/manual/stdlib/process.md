---
title: The process module
description: Running another program - its arguments, its exit code and what it writes, under the right to run.
module: စံကား/လုပ်ငန်းစဉ်
---

# The process module

The process module, `"စံကား/လုပ်ငန်းစဉ်"`, runs another program and waits for it to end. We
give the name of the program with the preposition `ပရိုဂရမ်ကို` and its arguments, a list of
text, with `ဖြင့်`, then call `မောင်းနှင်`. Each argument goes to the program as it is, with
no shell between them, so spaces and quotes in an argument need no escapes.

```zaka-file
// main.zaka
"စံကား/လုပ်ငန်းစဉ်" မှ မောင်းနှင် ကို ထည့်သွင်းမည်။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"git" ပရိုဂရမ်ကို ["--version"] ဖြင့် မောင်းနှင် သည့် ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```console
$ zakac run --allow-run=git main.zaka
ရလဒ်ကောင်း(လုပ်ငန်းစဉ်ရလဒ်(ပြီးဆုံးမှုကုဒ်: ၀၊ စံပြုအထွက်: "git version 2.56.0
"၊ စံပြုအမှား: ""))
```

## The result

`မောင်းနှင်` gives a result (a `Result`), as each function of the [io module](io.md) that
reaches outside the program does. Its good side is a `လုပ်ငန်းစဉ်ရလဒ်` struct:

| Field | Data kind | Holds |
| --- | --- | --- |
| `ပြီးဆုံးမှုကုဒ်` | `ကိန်းပြည့်` | the exit code: 0 when the program went well, -1 when a signal ended it |
| `စံပြုအထွက်` | `စာသား` | what the program wrote to standard output |
| `စံပြုအမှား` | `စာသား` | what the program wrote to standard error |

A program that fails still gives a good result: the exit code and the error output tell
what went wrong. The result fails with an error of the io module when the program does not
start: `မတွေ့ခြင်း` when no program of that name is found, `ခွင့်ပြုချက်မရှိခြင်း` when the
run lacks the right, and another member for other reasons.

While the program runs, it reads the standard input of our run, so a tool that asks a
question reads the answer from the same terminal or pipe. What it writes comes back in the
struct and does not reach the screen; we print it when we want it shown.

## The right to run

A run needs the right to run, `--allow-run`, to start a program, and `zakac run`,
`zakac test` and a [native build](../compiler/native.md#permissions) take it
([Permissions](../vm/running.md#permissions)). With a list, as in `--allow-run=git,cc`,
the run can start those programs alone, matched against the name a call gives. On a terminal,
a run without the right asks first.

A [compile-time](../language/compile-time.md) call never runs a program, whatever flags
the command gets: compile time reads the host and never changes it, so the call stops
the build with ZK0611.

The books and the explorer run programs in storage, with no programs of a host to start,
so there the call fails with `တခြားအကြောင်းအရင်း`:

```zaka
"စံကား/လုပ်ငန်းစဉ်" မှ မောင်းနှင် ကို ထည့်သွင်းမည်။
"git" ပရိုဂရမ်ကို ["--version"] ဖြင့် မောင်းနှင် သည့် ရလဒ် ကို ပြသမည်။
```

```text
အမှားအယွင်း(အချက်အလက်အဝင်အထွက်အမှားအယွင်းများထဲမှ တခြားအကြောင်းအရင်း)
```
