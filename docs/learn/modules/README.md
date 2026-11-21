---
title: Modules
description: Split a program into several files, share names with အများသုံး, and bring them into another file with ထည့်သွင်းမည်.
---

# Modules

We can split a growing program into several modules, with one `.zaka` source file per
module. To use a name from another module, mark it for export in its own file, then
import it where it is needed.

## You will learn

- How to share a function, a struct, a variable or a constant with other files.
- How to import names from another file, or each name that it shares.
- What a file that other files import can contain.
- How to pass imported names on to other files.

## Two files

The two files below sit in one folder. A Learn page cannot run a program of two files, so
save them and run the second one with `zakac run`:

```zaka-file
// math.zaka
နှစ်ထပ် ဟူသော အများသုံး လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```zaka-file
// main.zaka
"math.zaka" မှ နှစ်ထပ် ကို ထည့်သွင်းမည်။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက်ကို ၇ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```console
$ zakac run main.zaka
၄၉
```

In `math.zaka`, we write `အများသုံး` after `ဟူသော` to share the function, and in
`main.zaka`, we write an import sentence to bring it in:

```
<name> ဟူသော အများသုံး လုပ်ဆောင်ချက် သည် …
"<file>" မှ <name> ကို ထည့်သွင်းမည်။
```

| Zaka | Reads as |
| --- | --- |
| `နှစ်ထပ် ဟူသော အများသုံး လုပ်ဆောင်ချက်` | "the function `နှစ်ထပ်`, for common use" |
| `"math.zaka" မှ` | "from `math.zaka`" |
| `နှစ်ထပ် ကို ထည့်သွင်းမည်` | "bring in `နှစ်ထပ်`" |

## Sharing names

A function, a struct, a variable or a constant at the top level of a file belongs to that
file, and we use `အများသုံး` ("for common use") to share it with other files. On a
function, a struct or an enumeration, we write it after `ဟူသော`, and on a variable or a
constant, we write it at the start:

```
<name> ဟူသော အများသုံး လုပ်ဆောင်ချက် သည် …
<name> ဟူသော အများသုံး ဒေတာအစုအဝေးတွင် …
အများသုံး ကိန်းရှင် <name> သည် …
အများသုံး ကိန်းသေ <name> သည် …
```

A name without `အများသုံး` remains private, so a file can keep helper functions
that no other file sees:

```zaka-file
// math.zaka
အများသုံး ကိန်းသေ အများဆုံး သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀၀ ဖြစ်ပါစေ။

// shared: other files can import it
ကန့်သတ် ဟူသော အများသုံး လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ ကျော်လွန် ဟူသော လုပ်ဆောင်ချက်ကို က ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်လျှင် အများဆုံး ကို ပြန်ပေးမည်။
က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// private: just math.zaka can call it
ကျော်လွန် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
က သည် အများဆုံး ထက် ကြီးသည် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```zaka-file
// main.zaka
"math.zaka" မှ ကန့်သတ် နှင့် အများဆုံး တို့ကို ထည့်သွင်းမည်။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကန့်သတ် ဟူသော လုပ်ဆောင်ချက်ကို ၄၂ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ကန့်သတ် ဟူသော လုပ်ဆောင်ချက်ကို ၅၀၀ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
အများဆုံး ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```console
$ zakac run main.zaka
၄၂
၁၀၀
၁၀၀
```

`main.zaka` cannot import `ကျော်လွန်`, because an import of a private name is an error, as
[Module mistakes](mistakes.md#a-name-the-file-does-not-export) shows.

## Importing names

An import sentence names a file and the names that it brings in from that file:

```
"<file>" မှ <name> ကို ထည့်သွင်းမည်။
"<file>" မှ <name> နှင့် <name> တို့ကို ထည့်သွင်းမည်။
```

```zaka-file
"math.zaka" မှ ကန့်သတ် ကို ထည့်သွင်းမည်။
"math.zaka" မှ ကန့်သတ် နှင့် အများဆုံး တို့ကို ထည့်သွင်းမည်။
```

- For one name, we use `ကို`. For two or more, we put `နှင့်` before the last name, `၊`
  between the others, and `တို့ကို` after the last one.
- The imports of a file come first, above each declaration and sentence.
- The file name is relative to the folder of the file that imports it, so
  `"lib/point.zaka"` is the file `point.zaka` in the folder `lib` next to it.
- An imported name works as if the file had declared it: we can call the function, read
  or set the variable, and use the struct as a data kind (type).
- An import brings in the names that it lists and nothing else. The names that
  `math.zaka` imports for itself remain in `math.zaka`, and a file that needs them imports
  them from their own file.

To bring in each name that a file shares, we leave the names out:

```
"<file>" ကို ထည့်သွင်းမည်။
```

```zaka-file
"math.zaka" ကို ထည့်သွင်းမည်။
```

When two such whole imports bring in the same name from two files, the program cannot
tell which one we mean, and a use of that name is the error ZK0262. A named import of it
from one of the two files settles it.

## Passing names on

A file can pass the names that it imports on to the files that import it, which suits a
file that gathers several files into one. We use `ပြန်လည် မျှဝေမည်` ("share again"):

```
"<file>" ကို ပြန်လည် မျှဝေမည်။
<name> ကို အများသုံးနိုင်ရန် ပြန်လည် မျှဝေမည်။
```

```zaka-file
// shapes.zaka
"point.zaka" ကို ပြန်လည် မျှဝေမည်။
"circle.zaka" မှ စက်ဝိုင်း ကို ထည့်သွင်းမည်။
စက်ဝိုင်း ကို အများသုံးနိုင်ရန် ပြန်လည် မျှဝေမည်။
```

The first line shares each name that `point.zaka` shares, without bringing those names
into `shapes.zaka`. The last line shares `စက်ဝိုင်း`, which `shapes.zaka` imports. A file
that imports `shapes.zaka` can then use both.

## What an imported file contains

A file that another file imports contains declarations: functions, structs, variables,
constants, and the imports that it needs. The sentences that print, loop, call or set
values belong in the file that we run, which is the one that we give to `zakac run`.

The variables and constants of an imported file get their starting value before the
program runs, so the value must be known at compile time: a literal, a constant, or
operators on them, as in `၁၀ * ၃`. Such a file has no program start block that could set
its variables later, so each variable gets its value in its declaration:

```zaka-file
// counter.zaka
အများသုံး ကိန်းသေ အများဆုံး သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀ * ၃ ဖြစ်ပါစေ။
အများသုံး ကိန်းရှင် အကြိမ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။

တိုး ဟူသော အများသုံး လုပ်ဆောင်ချက် သည်
အရေ ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ အကြိမ် + အရေ သည် အများဆုံး ထက် ကြီးလျှင် ပြန်ပေးမည်။
အကြိမ် ကို အကြိမ် + အရေ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```zaka-file
// main.zaka
"counter.zaka" မှ အကြိမ် နှင့် တိုး တို့ကို ထည့်သွင်းမည်။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
တိုး ဟူသော လုပ်ဆောင်ချက်ကို ၁၀ ဖြင့် လုပ်ဆောင်မည်။
တိုး ဟူသော လုပ်ဆောင်ချက်ကို ၁၅ ဖြင့် လုပ်ဆောင်မည်။
အကြိမ် ကို ပြသမည်။
တိုး ဟူသော လုပ်ဆောင်ချက်ကို ၁၀ ဖြင့် လုပ်ဆောင်မည်။
အကြိမ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```console
$ zakac run main.zaka
၂၅
၂၅
```

The third call would take `အကြိမ်` past `အများဆုံး`, so `တိုး` leaves it as it is.

The compiler reads each file once, however often other files import it. So a variable
that a file declares exists once, and each file that imports it sees the same value: when
one file changes it, the others see the change.

## Details

- **Starting values.** A starting value from a call of a plain function is not known at
  compile time, so it is the error ZK0604, while a call of a [comptime
  function](../functions/compile-time.md) is fine ([A starting value that needs a
  call](mistakes.md#a-starting-value-that-needs-a-call)). A variable of an imported file
  without a value is the error ZK0483 ([Globals of a
  module](../../manual/language/modules.md#globals-of-a-module)).
- **One name in two files.** Two files can each have a private function or variable of the
  same name, and each file uses its own. Struct names are different: we declare a struct
  name once in the whole program, so two files cannot each declare a struct `လူ`, even a
  private one.
- **Programs without a file.** A program on a Learn page, in the compiler explorer, or given
  to `zakac run` on standard input has no folder, so it cannot import
  ([Imports outside a file](mistakes.md#imports-outside-a-file)).
- **The stdlib.** We import a module of the stdlib (standard library) in the same form, with
  a path that starts with `စံကား/`, as in `"စံကား/စာရင်း" မှ ဖယ် ကို ထည့်သွင်းမည်။`
  ([The stdlib](../stdlib/README.md)).
- **The full rules.** [Modules](../../manual/language/modules.md) has the full rules,
  including [passing imported names
  on](../../manual/language/modules.md#sharing-imported-names).

## Recap

- Each `.zaka` file is a module, and its names are private unless we mark them with
  `အများသုံး`.
- `"<file>" မှ <name> ကို ထည့်သွင်းမည်။` imports a name, and `"<file>" ကို ထည့်သွင်းမည်။`
  imports each name that the file shares.
- Imports come first in a file, and the file name is relative to the folder of the file.
- An imported file contains declarations, and its variables get values that are known at
  compile time.
- `ပြန်လည် မျှဝေမည်` passes imported names on to other files.

## Hands on

- Move the function `ကျော်လွန်` into a third file, share it from there, and import it into
  `math.zaka`.
- Change the import of `main.zaka` to a whole import, then add a constant to
  `counter.zaka` and print it from `main.zaka`.
- Import a private name and read the error.

Next: [Module mistakes](mistakes.md).
