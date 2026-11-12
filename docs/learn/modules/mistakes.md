---
title: Module mistakes
description: The mistakes imports and the export mark make possible, and how Zaka catches each one.
---

# Module mistakes

Most of these mistakes need two files, so this page shows the files and the error as
text. The ones that fit in one file the page checks as well.

## The export mark in the wrong place

`အများသုံး` marks a declaration at the top level of a file: a function, a struct, a
variable or a constant. Before a statement, or before a declaration inside a block, it
means nothing:

```zaka-error ZK0229
အများသုံး ၁ ကို ပြသမည်။
```

```text
error[ZK0229]: "အများသုံး" marks a top-level function, struct, enumeration, variable or constant, and nothing else
```

## An import below a statement

The imports of a file come first. An import below a declaration or a statement, or
inside a block, is refused:

```zaka-error ZK0230
၁ ကို ပြသမည်။
"math.zaka" မှ နှစ်ထပ် ကို ထည့်သွင်းမည်။
```

```text
error[ZK0230]: imports go at the top of the file
  = help: move it above the first declaration or statement
```

## Files that import each other

`a.zaka` imports `b.zaka`, and `b.zaka` imports `a.zaka`. Neither file can be read
first:

```zaka-file
// a.zaka
"b.zaka" မှ ခ ကို ထည့်သွင်းမည်။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ခ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```zaka-file
// b.zaka
"a.zaka" မှ က ကို ထည့်သွင်းမည်။
အများသုံး ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
အများသုံး ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
```

```text
error[ZK0231]: this import goes in a circle: a.zaka → b.zaka → a.zaka
```

The circle can pass through more files. Move what both files need to a third file, and
import it from both.

## A name the file does not export

`math.zaka` shares `နှစ်ထပ်` and keeps `သုံးထပ်` private:

```zaka-file
// math.zaka
အများသုံး လုပ်ဆောင်ချက် နှစ်ထပ် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် သုံးထပ် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * က * က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```zaka-file
// main.zaka
"math.zaka" မှ နှစ်ထပ်၊ သုံးထပ် တို့ကို ထည့်သွင်းမည်။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် သုံးထပ် ကို ၂ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0322]: "math.zaka" exports nothing named "သုံးထပ်"
  = help: "သုံးထပ်" is declared there without "အများသုံး"; add the mark to share it
error[ZK0303]: function "သုံးထပ်" is not declared
  = help: "သုံးထပ်" is declared in "math.zaka"; mark it "အများသုံး" there and import it here
```

Mark `သုံးထပ်` with `အများသုံး` in `math.zaka`. A name that `math.zaka` does not
declare at all gets ZK0322 with no help: check its spelling.

## A private struct in an export

An exported function, variable, constant or struct can be used from other files, so
each struct its kinds name must be exported too. Here `မူလ` is exported, but its kind,
`နေရာ`, is private to the file:

```zaka-error ZK0324
နေရာ ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု၊ y ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။
အများသုံး ကိန်းရှင် မူလ သည် နေရာ တစ်ခု ဖြစ်ပါစေ။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0324]: "မူလ" is exported, but its kind names the private struct "နေရာ"
  = help: mark "နေရာ" with "အများသုံး" as well, or keep "မူလ" private
```

The same holds for the parameters and the result of a function and for the fields of a
struct, with each pointer and list level around the struct. A struct that the file
imports from another file was exported there, so naming it is fine.

## A statement in an imported file

An imported file holds declarations. A statement in it is refused, since the program
would run it at a moment no one chose:

```zaka-file
// counter.zaka
အများသုံး ကိန်းရှင် အကြိမ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
"စတင်ပြီ" ဟု ပြသမည်။
```

```zaka-file
// main.zaka
"counter.zaka" မှ အကြိမ် ကို ထည့်သွင်းမည်။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကြိမ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0323]: "counter.zaka" is imported, so it holds declarations and no statements
```

Move the statement to the file that is run.

## A starting value that needs a call

The variables and constants of an imported file start with a value known at compile
time. A function call is not known then:

```zaka-file
// clock.zaka
လုပ်ဆောင်ချက် စက္ကန့် သည်
မိနစ် ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
မိနစ် * ၆၀ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အများသုံး ကိန်းသေ တစ်နာရီ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် စက္ကန့် ကို ၆၀ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
```

```zaka-file
// main.zaka
"clock.zaka" မှ တစ်နာရီ ကို ထည့်သွင်းမည်။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
တစ်နာရီ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0604]: the starting value of "တစ်နာရီ" must be known at compile time, since its file is imported
  = help: literals, constants and operators on them are known; a function call is not, for now
```

Write the value out, as `၆၀ * ၆၀`.

## A variable of a module with no value

A file run as a module has no start block, so nothing in it could set a variable later.
Each of its variables gets a value in its declaration:

```zaka-file
// counter.zaka
အများသုံး ကိန်းရှင် အကြိမ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
```

```zaka-file
// main.zaka
"counter.zaka" မှ အကြိမ် ကို ထည့်သွင်းမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကြိမ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0483]: "အကြိမ်" is a global of a module, so it needs its value where it is declared
  = help: give it a value, such as "ကနဦးတန်ဖိုး"
```

Write `ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး`, or the value the counter starts from.

## Imports outside a file

A program on this page, in the compiler explorer or on standard input has no folder, so
it cannot import:

```zaka-error ZK0232
"math.zaka" မှ နှစ်ထပ် ကို ထည့်သွင်းမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် နှစ်ထပ် ကို ၇ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0232]: this program was not read from a file, so it cannot import "math.zaka"
```
