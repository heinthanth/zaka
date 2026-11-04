---
title: Where a program starts
description: The start block, the arguments a program gets on the command line, its environment and its exit code.
---

# Where a program starts

Each program so far starts in its *start block*, `ပရိုဂရမ်သည်`. This page shows the
rest of what a start block can do: it gets the command line it was run with, the name of
the program first, and its result tells the shell how the run went.

```zaka
/// Greet one person.
လုပ်ဆောင်ချက် နှုတ်ဆက် သည်
စာသား အမည် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ {အမည်}" ဟု ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည်
စာသား စာရင်း အမည်များ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
အကယ်၍ အမည်များ ၏ အကွက်အရေအတွက် သည် ၁ နှင့် ညီလျှင် အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
"အသုံးပြုပုံ: {အမည်များ ၏ ၁ ခုမြောက်အကွက်} <အမည်>..." ဟု ပြသပါ။
၁ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
၂ မှ အမည်များ ၏ အကွက်အရေအတွက် အထိ နံပါတ် အနေဖြင့် ရေတွက်၍ လုပ်ဆောင်ချက် နှုတ်ဆက် ကို အမည်များ ၏ နံပါတ် ခုမြောက်အကွက် တို့ဖြင့် လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အသုံးပြုပုံ: main.zaka <အမည်>...
```

The Run button gives the program its name, `main.zaka`, and nothing more, so it tells
how to use it. On the command line, the arguments come after `--`:

```console
$ zakac run greet.zaka -- မောင်မောင် အေးအေး
မင်္ဂလာပါ မောင်မောင်
မင်္ဂလာပါ အေးအေး
```

## The parts of a start block

The start block reads like a function with no name. In place of
`လုပ်ဆောင်ချက် <name> သည်`, it opens with `ပရိုဂရမ်သည်`, "the program":

- `စာသား စာရင်း အမည်များ ကို လက်ခံပြီး` names the list of arguments, one text for each
  word. The name is `အမည်များ` here, but each name works.
- The first cell holds the name of the program, as in C, Rust and Python, and the
  arguments start at the second. The counted loop starts at `၂` for that reason.
- `ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်` gives the start block an integer result: the *exit
  code*.
- The block ends with `ပရိုဂရမ် ပြီးဆုံးပါပြီး။`, "the program is done", where a function ends with `လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။`.

The count of the list, `အမည်များ ၏ အကွက်အရေအတွက်`, is at least 1: a run with no
arguments still has the name of the program.

## The exit code

A shell reads the exit code of each program it runs: `0` means that all went well, and
another number means that something failed. Here the program returns `၁` when it gets
no name:

```console
$ zakac run greet.zaka
အသုံးပြုပုံ: greet.zaka <အမည်>...
$ echo $?
1
```

A start block with no result ends the run with `0`.

## The environment

A second parameter gets the environment variables of the run, one `NAME=VALUE` text for
each:

```text
ပရိုဂရမ်သည်
စာသား စာရင်း အမည်များ နှင့် စာသား စာရင်း ပတ်ဝန်းကျင် တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပတ်ဝန်းကျင် ၏ အကွက်အရေအတွက် ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

The Run buttons of these pages give a blank environment.

## The top level beside a start block

The top level of a file declares things and runs nothing: functions, structs, variables
and constants. The starting values of the variables are set
first, and then the start block runs. A statement that runs at the top level belongs
inside the start block:

```zaka-error ZK0326
"ဟိုင်း" ဟု ပြသပါ။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0326]: a statement that runs goes inside the start block, not at the top level
```

The full rules, and the errors of a wrong header, are in [The start
block](../../manual/language/functions.md#the-start-block).
