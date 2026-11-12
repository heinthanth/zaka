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
"မင်္ဂလာပါ" ဟု ပြသမည်။
"နေကောင်းလား" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// Call it twice
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
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
<name> ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
```

"The function called *name*: will perform it." The name comes first, then
`ဟူသော လုပ်ဆောင်ချက်ကို` ("the function called … "), then `လုပ်ဆောင်မည်` ("will perform").
`မည့်` can stand for `ဟူသော`: `<name> မည့် လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။` is the same call.

| Zaka | Reads as |
| --- | --- |
| `နှုတ်ဆက်` | the name |
| `ဟူသော လုပ်ဆောင်ချက်ကို` | "the function called …" (object) |
| `လုပ်ဆောင်မည်။` | "will perform" |

A shorter call names the function and ends with `မည်` ("will"): `နှုတ်ဆက် မည်။` runs the
same call. Keep a space between the name and `မည်`: `နှုတ်ဆက်မည်` reads as one name.

## Where functions go

- Declare functions at the top level of the program. A function can also stand inside the
  start block or another function (see below).
- A function can be called before its declaration appears. A lot of programs put the
  start block at the top and the functions below it.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
မျဉ်းဆွဲ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။        // called before it is declared: fine
"ခေါင်းစဉ်" ဟု ပြသမည်။
မျဉ်းဆွဲ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။

// draw a line
လုပ်ဆောင်ချက် မျဉ်းဆွဲ သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"--------" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
--------
ခေါင်းစဉ်
--------
```

## A function inside a function

A function can be declared inside the start block or inside another function. It can use
the variables declared above it in the block around it, as in JavaScript or Python: it
reads and changes those variables themselves, not copies of them.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စုစုပေါင်း သည် ၀ ဖြစ်ပါစေ။
// a function inside the start block: it changes စုစုပေါင်း
လုပ်ဆောင်ချက် ပေါင်းထည့် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စုစုပေါင်း ကို စုစုပေါင်း + က ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပေါင်းထည့် ဟူသော လုပ်ဆောင်ချက်ကို ၅ တို့ဖြင့် လုပ်ဆောင်မည်။
ပေါင်းထည့် ဟူသော လုပ်ဆောင်ချက်ကို ၇ တို့ဖြင့် လုပ်ဆောင်မည်။
စုစုပေါင်း ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၂
```

`ပေါင်းထည့်` can be called from its declaration to the end of the block that holds it.
The start block itself, and a function that comes from outside, remain at the top level.
A function that uses the variables around it is a *closure*, and it can even leave its
block: see [Closures](../function-values/closures.md).

## Leaving before the end

`ပြန်ပေးမည်။` ("will return") leaves the function at once. The program goes back to
where the function was called:

```zaka
လုပ်ဆောင်ချက် စမ်း သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ရောက်သည်" ဟု ပြသမည်။
ပြန်ပေးမည်။
"မရောက်ပါ" ဟု ပြသမည်။            // never runs
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စမ်း ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
"ပြီး" ဟု ပြသမည်။
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
