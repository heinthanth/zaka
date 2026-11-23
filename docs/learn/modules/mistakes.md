---
title: Module mistakes
description: The mistakes that imports and the export mark make possible, and how Zaka catches each one.
---

# Module mistakes

This page shows the common mistakes with modules (one source file each), the errors that
the compiler reports for them, and how to fix each one. Most of these mistakes need two
files, so the page shows the files and the error as text, while the mistakes that fit in
one file are checked on the page as well.

## You will learn

- Where `အများသုံးနိုင်သော` and an import sentence can go in a file.
- Which names a file can import from another file.
- What an imported file cannot contain.
- The reason a program without a file cannot import.

## The export mark in the wrong place

We can use `အများသုံးနိုင်သော` on a declaration at the top level of a file: a function, a struct,
an enumeration, a variable or a constant. Before a sentence, or before a declaration
inside a block, it has no meaning, so the compiler stops with the error ZK0229:

```zaka-error ZK0229
အများသုံးနိုင်သော ၁ ကို ပြသမည်။
```

```text
error[ZK0229]: "အများသုံးနိုင်သော" marks a declaration at the top level: it stands before a variable, a constant or a kind alias, and after "ဟူသော" in a function, a struct or an enumeration
```

## An import below a statement

The imports of a file come first. When we write an import below a declaration or a
sentence, or inside a block, the compiler stops with the error ZK0230:

```zaka-error ZK0230
၁ ကို ပြသမည်။
"math.zaka" မှ နှစ်ထပ် ကို ထည့်သွင်းမည်။
```

```text
error[ZK0230]: imports go at the top of the file
  = help: move it above the first declaration or statement
```

## Files that import each other

Here `a.zaka` imports `b.zaka`, and `b.zaka` imports `a.zaka`, so the compiler cannot
read either file first:

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
အများသုံးနိုင်သော ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
အများသုံးနိုင်သော ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
```

```text
error[ZK0231]: this import goes in a circle: a.zaka → b.zaka → a.zaka
```

The circle can also pass through more files. To fix it, we move what both files need to
a third file, and import that file from both.

## A name the file does not export

Here `math.zaka` shares `နှစ်ထပ်` and keeps `သုံးထပ်` private:

```zaka-file
// math.zaka
နှစ်ထပ် ဟူသော အများသုံးနိုင်သော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

သုံးထပ် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * က * က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```zaka-file
// main.zaka
"math.zaka" မှ နှစ်ထပ် နှင့် သုံးထပ် တို့ကို ထည့်သွင်းမည်။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
သုံးထပ် ဟူသော လုပ်ဆောင်ချက်ကို ၂ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0322]: "math.zaka" exports nothing named "သုံးထပ်"
  = help: "သုံးထပ်" is declared there without "အများသုံးနိုင်သော"; add the mark to share it
error[ZK0303]: function "သုံးထပ်" is not declared
  = help: "သုံးထပ်" is declared in "math.zaka"; mark it "အများသုံးနိုင်သော" there and import it here
```

To fix it, we mark `သုံးထပ်` with `အများသုံးနိုင်သော` in `math.zaka`. When `math.zaka` does not
declare the name at all, we get ZK0322 without the help line, and we check the spelling
of the name.

## A private struct in an export

Other files can use an exported function, variable, constant or struct, so each struct
that its data kinds (types) name must be exported too. Here `မူလ` is exported, but its
data kind, `နေရာ`, is private to the file:

```zaka-error ZK0324
နေရာ ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု နှင့် y ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။
အများသုံးနိုင်သော ကိန်းရှင် မူလ သည် နေရာ တစ်ခု ဖြစ်ပါစေ။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0324]: "မူလ" is exported, but its kind names the private struct "နေရာ"
  = help: mark "နေရာ" with "အများသုံးနိုင်သော" as well, or keep "မူလ" private
```

The rule is the same for the parameters and the result of a function and for the fields
of a struct, through each pointer and list around the struct. A struct that the file
imports from another file was exported there, so a file can name it without a problem.

## A statement in an imported file

An imported file contains declarations. When it contains a sentence, the compiler refuses
it, because the program would run that sentence at a moment that no one chose:

```zaka-file
// counter.zaka
အများသုံးနိုင်သော ကိန်းရှင် အကြိမ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
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

To fix it, we move the sentence to the file that we run.

## A starting value that needs a call

The variables and constants of an imported file start with a value that is known at
compile time, and a call of a plain function is not known then:

```zaka-file
// clock.zaka
စက္ကန့် ဟူသော လုပ်ဆောင်ချက် သည်
မိနစ် ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
မိနစ် * ၆၀ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အများသုံးနိုင်သော ကိန်းသေ တစ်နာရီ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် စက္ကန့် ဟူသော လုပ်ဆောင်ချက်ကို ၆၀ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
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
  = help: literals, constants, operators on them and calls of comptime functions are known; another call is not
```

To fix it, we write the value out, as `၆၀ * ၆၀`, or we mark `စက္ကန့်` as a [comptime
function](../functions/compile-time.md), whose call runs while the compiler runs.

## A variable of a module with no value

An imported file has no program start block, so nothing in it could set a variable later.
For this reason, each of its variables needs a value in its declaration:

```zaka-file
// counter.zaka
အများသုံးနိုင်သော ကိန်းရှင် အကြိမ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
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

To fix it, we write `ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး`, or the value that the
counter starts from.

## Imports outside a file

A program on this page, in the compiler explorer or on standard input has no folder, so
it cannot import, and the compiler stops with the error ZK0232:

```zaka-error ZK0232
"math.zaka" မှ နှစ်ထပ် ကို ထည့်သွင်းမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက်ကို ၇ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0232]: this program was not read from a file, so it cannot import "math.zaka"
```

To fix it, we save the program in a file next to `math.zaka` and run that file.

## Recap

- `အများသုံးနိုင်သော` and imports belong at the top level, and the imports come first.
- Two files cannot import each other, so we move the shared part to a third file.
- A file can import just the names that the other file marks with `အများသုံးနိုင်သော`, and an
  exported declaration can name just exported structs.
- An imported file contains declarations, and each of its variables gets a value that is
  known at compile time.
- A program without a file cannot import.

Next: [Threads](../threads/README.md).
