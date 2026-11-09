---
title: Functions
description: Name a group of statements and run it whenever it is needed.
---

# Functions

A *function* is a named group of statements. Write it once, then run it through its
name as often as needed. Running a function is called *calling* it. Functions keep
programs short (no copies of the same lines) and readable (the name tells what the
lines do).

```zaka
// Declare a function named နှုတ်ဆက် (greet)
လုပ်ဆောင်ချက် နှုတ်ဆက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ" ဟု ပြသပါ။
"နေကောင်းလား" ဟု ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// Call it twice
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် နှုတ်ဆက် ကို လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် နှုတ်ဆက် ကို လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ
နေကောင်းလား
မင်္ဂလာပါ
နေကောင်းလား
```

Declaring a function does not run it. The statements inside run when the function is
called, and not before.

## Declaring

```
လုပ်ဆောင်ချက် <name> သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
<statements>
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

| Zaka | Reads as |
| --- | --- |
| `လုပ်ဆောင်ချက်` | "function" |
| `နှုတ်ဆက်` | the chosen name |
| `သည်` | (subject) |
| `အောက်ပါတို့ကို လုပ်ဆောင်မည်။` | "will do the following" |
| `လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။` | "the tasks are done" |

The line breaks in the header are there to help the reader. The header can also sit on
one line.

## Calling

```
လုပ်ဆောင်ချက် <name> ကို လုပ်ဆောင်ပါ။
```

"Please run the function *name*." `လုပ်ဆောင်ပါ` means "please perform".

A shorter call names the function and ends with `မည်` ("will"): `နှုတ်ဆက် မည်။` runs the
same call. Keep a space between the name and `မည်`: `နှုတ်ဆက်မည်` reads as one name.

## Where functions go

- Declare functions at the top level of the program. A function can also stand inside the
  start block or another function (see below).
- A function can be called before its declaration appears. A lot of programs put the
  start block at the top and the functions below it.

```zaka

// draw a line
လုပ်ဆောင်ချက် မျဉ်းဆွဲ သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"--------" ဟု ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် မျဉ်းဆွဲ ကို လုပ်ဆောင်ပါ။        // called before it is declared: fine
"ခေါင်းစဉ်" ဟု ပြသပါ။
လုပ်ဆောင်ချက် မျဉ်းဆွဲ ကို လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
--------
ခေါင်းစဉ်
--------
```

## A function inside a function

A function can be declared inside the start block or inside another function. It can use
the variables declared above it in the block around it, when it lists them before
`လက်ခံပြီး`. With `ငှား၍` ("borrowing") it reads and changes those variables themselves,
not copies of them; then each item of the list ends in `ကိုလည်းကောင်း` ("and also"):

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စုစုပေါင်း သည် ၀ ဖြစ်ပါစေ။
// a function inside the start block: it borrows စုစုပေါင်း, to read and change it
လုပ်ဆောင်ချက် ပေါင်းထည့် သည် ကိန်းပြည့် က ကိုလည်းကောင်း၊ စုစုပေါင်း ကို ငှား၍လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စုစုပေါင်း ကို စုစုပေါင်း + က ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် ပေါင်းထည့် ကို ၅ တို့ဖြင့် လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် ပေါင်းထည့် ကို ၇ တို့ဖြင့် လုပ်ဆောင်ပါ။
စုစုပေါင်း ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၂
```

`ပေါင်းထည့်` can be called from its declaration to the end of the block that holds it. A
function that keeps copies of its own, and so can be given back, is in
[Functions as values](../pointers/functions.md#a-function-that-keeps-values). The start
block itself, and a function that comes from outside, remain at the top level. The rules in
full are in
[Nested functions and closures](../../manual/language/function-values.md#nested-functions-and-closures).

## Leaving before the end

`ပြန်ပေးပါ။` ("please return") leaves the function at once. The program goes back to
where the function was called:

```zaka
လုပ်ဆောင်ချက် စမ်း သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ရောက်သည်" ဟု ပြသပါ။
ပြန်ပေးပါ။
"မရောက်ပါ" ဟု ပြသပါ။            // never runs
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် စမ်း ကို လုပ်ဆောင်ပါ။
"ပြီး" ဟု ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရောက်သည်
ပြီး
```

On its own this does little. Together with [if](../control-flow/if-else.md) and
[parameters](parameters.md), it lets a function stop when there is nothing left to do.

## Hands on

- Write a function that prints a box made of three lines of `#`, and call it twice.
- Call a function whose name was never declared. Read the message.

Next: [Parameters](parameters.md).
