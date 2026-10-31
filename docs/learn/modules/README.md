---
title: Modules
description: Split a program into files, share names with အများသုံး, and bring them into another file with ထည့်သွင်းပါ.
---

# Modules

A program can grow past one file. Each `.zaka` file keeps its names to itself, except the
ones it marks `အများသုံး` ("for common use"). Another file brings those names in with an
*import*.

The two files below sit in one folder. A page cannot run a program of two files, so save
them and run the second one with `zakac run`.

```zaka-file
// math.zaka
အများသုံး လုပ်ဆောင်ချက် နှစ်ထပ် သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က * က ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
```

```zaka-file
// main.zaka
"math.zaka" မှ နှစ်ထပ် ကို ထည့်သွင်းပါ။
လုပ်ဆောင်ချက် နှစ်ထပ် ကို ၇ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
```

```console
$ zakac run main.zaka
၄၉
```

| Zaka | Reads as |
| --- | --- |
| `အများသုံး လုပ်ဆောင်ချက် နှစ်ထပ်` | "the function `နှစ်ထပ်`, for common use" |
| `"math.zaka" မှ` | "from `math.zaka`" |
| `နှစ်ထပ် ကို ထည့်သွင်းပါ` | "please bring in `နှစ်ထပ်`" |

## Sharing names

A function, a struct, a variable or a constant at the top level of a file belongs to that
file. `အများသုံး` before it shares it: other files can then import it. Each name without
the mark remains private, so a file can keep helpers that no other file sees.

```zaka-file
// math.zaka
အများသုံး ကိန်းသေ အများဆုံး သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၀၀ ဖြစ်ပါစေ။

// shared: other files can import it
အများသုံး လုပ်ဆောင်ချက် ကန့်သတ် သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
အကယ်၍ လုပ်ဆောင်ချက် ကျော်လွန် ကို က တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်လျှင် အများဆုံး ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
က ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။

// private: math.zaka alone can call it
လုပ်ဆောင်ချက် ကျော်လွန် သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး ရလဒ် ကို ပြန်ပေးမည်။
က သည် အများဆုံး ထက် ကြီးသည် ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
```

```zaka-file
// main.zaka
"math.zaka" မှ ကန့်သတ်၊ အများဆုံး တို့ကို ထည့်သွင်းပါ။
လုပ်ဆောင်ချက် ကန့်သတ် ကို ၄၂ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
လုပ်ဆောင်ချက် ကန့်သတ် ကို ၅၀၀ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
အများဆုံး ကို ပြသပါ။
```

```console
$ zakac run main.zaka
၄၂
၁၀၀
၁၀၀
```

`main.zaka` cannot import `ကျော်လွန်`: the import of a private name is an error, as
[Module mistakes](mistakes.md#a-name-the-file-does-not-export) shows.

## Importing names

An import names a file and the names it takes from it:

```zaka-file
"math.zaka" မှ ကန့်သတ်၊ အများဆုံး တို့ကို ထည့်သွင်းပါ။
"math.zaka" မှ ကန့်သတ် ကို ထည့်သွင်းပါ။
```

- Several names take `၊` between them and `တို့ကို` after the last; one name takes `ကို`.
- The imports of a file come first, above each declaration and statement.
- The file name is relative to the folder of the file that imports it, so
  `"lib/point.zaka"` is the file `point.zaka` in the folder `lib` beside it.
- An imported name works as if the file had declared it: call the function, read or set
  the variable, use the struct as a kind.
- An import brings the names it lists and nothing else. The names that `math.zaka`
  itself imports remain in `math.zaka`; a file that needs them imports them from their
  own file.

A file is read once, however often other files import it. A variable it declares exists once,
so each file that imports it sees the same value: when one file changes it, the others
see the change.

## What an imported file holds

A file that another file imports holds declarations: functions, structs, variables and
constants, and the imports it needs. The statements that print, loop, call or set
values belong in the file that is run, the one given to `zakac run`.

The variables and constants of an imported file get their starting value before the
program runs, so the value must be known at compile time: a literal, a constant, or
operators on them, as in `၁၀ * ၃`. A value from a function call is not allowed there for
now. A variable with no value starts at its default, as elsewhere.

```zaka-file
// counter.zaka
အများသုံး ကိန်းသေ အများဆုံး သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၀ * ၃ ဖြစ်ပါစေ။
အများသုံး ကိန်းရှင် အကြိမ် သည် ကိန်းပြည့် ဖြစ်ပါစေ။

အများသုံး လုပ်ဆောင်ချက် တိုး သည်
ကိန်းပြည့် အရေ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ အကြိမ် + အရေ သည် အများဆုံး ထက် ကြီးလျှင် ပြန်ပေးပါ။
အကြိမ် ကို အကြိမ် + အရေ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
```

```zaka-file
// main.zaka
"counter.zaka" မှ အကြိမ်၊ တိုး တို့ကို ထည့်သွင်းပါ။
လုပ်ဆောင်ချက် တိုး ကို ၁၀ တို့ဖြင့် လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် တိုး ကို ၁၅ တို့ဖြင့် လုပ်ဆောင်ပါ။
အကြိမ် ကို ပြသပါ။
လုပ်ဆောင်ချက် တိုး ကို ၁၀ တို့ဖြင့် လုပ်ဆောင်ပါ။
အကြိမ် ကို ပြသပါ။
```

```console
$ zakac run main.zaka
၂၅
၂၅
```

The third call would take `အကြိမ်` past `အများဆုံး`, so `တိုး` leaves it alone.

## One name in two files

Two files can each keep a private function or variable of one name; each file uses its
own. Struct names are different: a struct name is declared once in the whole program, so
two files cannot each declare a struct `လူ`, even a private one.

## Where imports do not work

A program written on a Learn page, in the compiler explorer, or given to `zakac run` on
standard input has no folder, so it cannot import. Save it in a file first.
