---
title: Compile-time conditionals
description: The conditional the compiler settles for the target a program is built for - the four facts of the target, the arm it keeps, the code it never checks - and the error ZK0255.
---

# Compile-time conditionals

A C lib is not named and laid out the same on each OS: the UCRT of Windows calls
`strdup` `_strdup`, and has no `snprintf`. The program names the lib to read and the
name to call, and a *compile-time conditional* lets it choose them for each OS. The
compiler keeps the arm whose condition holds for the target it builds for, and drops the
others before it checks names and kinds.

## The conditional

```text
အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် <condition> (လျှင်|ဖြစ်လျှင်) <branch>
{ ထိုသို့မဟုတ်ဘဲ <condition> (လျှင်|ဖြစ်လျှင်) <branch> }
[ သို့မဟုတ်ပါက <branch> ]
```

- `ကွန်ပိုင်းလုပ်ချိန်တွင်` ("at compile time") after `အကယ်၍` makes the whole chain one
  the compiler settles; the other arms read as in a plain [conditional](statements.md).
- The compiler keeps the statements of the first arm whose condition holds, or of the
  else branch, and puts them in place of the conditional: the arm opens no scope, so a
  name it declares is seen after the conditional. When no arm holds and no else branch
  is written, nothing is left.
- A conditional stands at the top level, around declarations, C functions, structs,
  traits and imports, or among the statements of a block. At the top level, its arms
  hold top-level statements.
- The arms the target drops must parse, but their names and kinds are never checked:
  an arm for Windows can name a lib or a C function that this host lacks.

## The facts of the target

A condition reads the four facts of the target, which the [prelude](prelude.md)
declares as constants:

| Constant | Values |
| --- | --- |
| `အလုပ်လုပ်မည့်ကွန်ပျူတာစနစ်` | the OS: `"windows"`, `"macos"` or `"linux"` |
| `အလုပ်လုပ်မည့်စီပီယူဗိသုကာပုံစံ` | the CPU: `"x86_64"`, `"aarch64"`, `"x86"`, `"arm"`, `"riscv64"` or `"wasm32"` |
| `အလုပ်လုပ်မည့်ပလက်ဖောင်းဘစ်` | the bit count of `--platform-bits`: 8, 16, 32 or 64 |
| `ကွန်ပိုင်လာအထွက်ကုဒ်အမျိုးအစား` | the code the compiler gives out: `"vm"` or `"c"` |

A condition holds the facts, text and digits, `==` and `!=` (and `<`, `<=`, `>`, `>=` on
the bit count), `ဖြစ်၍` (`&&`), `သို့မဟုတ်` (`||`), `မဟုတ်` and parentheses. Each
condition of a chain is settled, and each other part of one is ZK0255, as is a value a
fact never holds: `"window"` would be a branch that never runs.

The facts are constants all through the program as well: `အလုပ်လုပ်မည့်ကွန်ပျူတာစနစ် ကို
ပြသမည်။` prints the OS, and a plain conditional can test one when the program runs.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
// an arm that holds on each target, and one that holds on none
အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် အလုပ်လုပ်မည့်ပလက်ဖောင်းဘစ် >= ၈ ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည် သည် "built" ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် အလုပ်လုပ်မည့်ပလက်ဖောင်းဘစ် > ၆၄ ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
နာမည်မရှိ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အမည် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
built
```

```zaka-error ZK0255
အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် အလုပ်လုပ်မည့်ကွန်ပျူတာစနစ် == "window" ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ ပြတင်း သည် ၁ ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## C functions on each OS

The arm of each OS declares the C function its lib holds. A small function of the
program can give the two the same name, so the calls remain one:

```zaka-file
// dup.zaka
အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် အလုပ်လုပ်မည့်ကွန်ပျူတာစနစ် == "windows" ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် _strdup သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။
လုပ်ဆောင်ချက် strdup သည်
s ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
_strdup ဟူသော လုပ်ဆောင်ချက်ကို s ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
သို့မဟုတ်ပါက အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် strdup သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

A C function that takes other values cannot be wrapped so, as a function of the
program takes no other values: the call itself stands in a conditional of each arm, as
`snprintf` does on [C functions](functions.md#c-functions). The VM opens `ucrtbase.dll`
for `"libc"` on Windows, where `snprintf` is missing, so the program reads `_snprintf`
from `"msvcrt"` there, when `ကွန်ပိုင်လာအထွက်ကုဒ်အမျိုးအစား` is `"vm"`; the C target
links its own `snprintf`.

## The target of a build

The target is the host the compiler runs on: its OS, its CPU, the bit count of
`--platform-bits`, and the code of the command (`"c"` for `zakac build --target c`,
`"vm"` for the rest). `--target-os` and `--target-arch` name another
([Commands](../compiler/commands.md#options-of-the-compile-stages)).

- A program that holds a compile-time conditional or names a fact is settled for its
  target, and its ops for the VM record it: the VM refuses to run them on another OS
  or CPU, with a message that names both.
- The C target builds an executable for the host alone: for another target it writes
  the C files (`--emit c`), to build there.
- The explorer settles a program for Linux on x86-64.

## Errors

| Code | When |
| --- | --- |
| ZK0255 | a compile-time condition that holds what the compiler cannot settle, or compares a fact with a value it never holds |
