---
title: Functions
description: Name a group of statements and run it whenever you need it.
---

# Functions

A *function* is a named group of statements. You write it once, then run it by name as
often as you like. Running a function is called *calling* it. Functions keep programs
short (no copying the same lines) and readable (the name says what the lines do).

```zaka
// Declare a function named နှုတ်ဆက် (greet)
လုပ်ဆောင်ချက် နှုတ်ဆက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ" ဟု ပြသပါ။
"နေကောင်းလား" ဟု ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

// Call it twice
လုပ်ဆောင်ချက် နှုတ်ဆက် ကို လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် နှုတ်ဆက် ကို လုပ်ဆောင်ပါ။
```

```text
မင်္ဂလာပါ
နေကောင်းလား
မင်္ဂလာပါ
နေကောင်းလား
```

Declaring a function does not run it. The statements inside run only when the function
is called.

## Declaring

```
လုပ်ဆောင်ချက် <name> သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
<statements>
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
```

| Zaka | Reads as |
| --- | --- |
| `လုပ်ဆောင်ချက်` | "function" |
| `နှုတ်ဆက်` | the name you choose |
| `သည်` | (subject) |
| `အောက်ပါတို့ကို လုပ်ဆောင်မည်။` | "will do the following" |
| `လုပ်ဆောင်စရာများ ပြီးပါပြီး။` | "the tasks are done" |

The line breaks in the header are only for readability. The header can also sit on one
line.

## Calling

```
လုပ်ဆောင်ချက် <name> ကို လုပ်ဆောင်ပါ။
```

"Please run the function *name*." `လုပ်ဆောင်ပါ` means "please carry out".

## Where functions go

- Declare functions at the top level of the program, not inside a block or another
  function.
- A function can be called before its declaration appears. Many programs put the main
  work at the top and the functions below it.

```zaka
လုပ်ဆောင်ချက် မျဉ်းဆွဲ ကို လုပ်ဆောင်ပါ။        // called before it is declared: fine
"ခေါင်းစဉ်" ဟု ပြသပါ။
လုပ်ဆောင်ချက် မျဉ်းဆွဲ ကို လုပ်ဆောင်ပါ။

// draw a line
လုပ်ဆောင်ချက် မျဉ်းဆွဲ သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"--------" ဟု ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
```

```text
--------
ခေါင်းစဉ်
--------
```

```zaka-error ZK0214
အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် တစ် သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
```

```text
error[ZK0214]: functions can be declared at the top level and nowhere else
```

## Leaving early

`ပြန်ပေးပါ။` ("please return") leaves the function at once. The program goes back to
where the function was called:

```zaka
လုပ်ဆောင်ချက် စမ်း သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ရောက်သည်" ဟု ပြသပါ။
ပြန်ပေးပါ။
"မရောက်ပါ" ဟု ပြသပါ။            // never runs
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

လုပ်ဆောင်ချက် စမ်း ကို လုပ်ဆောင်ပါ။
"ပြီး" ဟု ပြသပါ။
```

```text
ရောက်သည်
ပြီး
```

On its own this is not very useful. Together with [if](../control-flow/if-else.md) and
[parameters](parameters.md), it lets a function stop when there is nothing to do.

## Try it

- Write a function that prints a box made of three lines of `#`, and call it twice.
- Call a function whose name was never declared. Read the message.

Next: [Parameters](parameters.md).
