---
title: Functions
description: Give a group of statements a name, and run it through that name whenever the program needs it.
---

# Functions

A *function* gives a group of statements a name. We write the statements once and
*call* the function wherever we need to run them. For example, a function named
`နှုတ်ဆက်` (greet) can print a greeting each time we call it.

## You will learn

- how to declare a function and how to call it
- where functions go in a file, and how to put one function inside another
- how to leave a function before its end

```zaka
// Declare a function named နှုတ်ဆက် (greet)
နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ" ဟု ပြသမည်။
"နေကောင်းလား" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// Call it twice
နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
```

```text
မင်္ဂလာပါ
နေကောင်းလား
မင်္ဂလာပါ
နေကောင်းလား
```

The first part of the file declares the function, and the two lines after it call it twice;
those lines form the program start block (entry point, `main`), as in [REPL
mode](../../manual/language/functions-start-block.md#repl-mode). Declaring a function
does not run it: the statements inside run each time the function is called, and not
before. We can use this pattern to declare a
function:

```text
<name> ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
<statements>
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

| Zaka | Reads as |
| --- | --- |
| `နှုတ်ဆက်` | the name we choose |
| `ဟူသော` | "called" |
| `လုပ်ဆောင်ချက်` | "function" |
| `သည်` | (the subject particle) |
| `အောက်ပါတို့ကို လုပ်ဆောင်မည်။` | "will do the following" |
| `လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။` | "the tasks are done" |

The first two lines are the *function signature*, and the last line is
the block closer (end of a block). The name comes first, then `ဟူသော`, the same as in the
declaration of a [struct](../structs/README.md). The next pages add parameters and a result
to the function signature.

## Calling a function

```text
<name> ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
<name> မည်။
```

To call a function, we write its name, then `ဟူသော လုပ်ဆောင်ချက်ကို` ("the function
called …") and `လုပ်ဆောင်မည်` ("will perform"). The second line is a short form of the same
call, where we write the name and then `မည်` ("will"):

```zaka
နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။ // the long form
နှုတ်ဆက် မည်။ // the short form
```

```text
မင်္ဂလာပါ
မင်္ဂလာပါ
```

| Zaka | Reads as |
| --- | --- |
| `နှုတ်ဆက်` | the name |
| `ဟူသော လုပ်ဆောင်ချက်ကို` | "the function called …" (the object) |
| `လုပ်ဆောင်မည်။` | "will perform" |
| `နှုတ်ဆက် မည်။` | "will greet" |

## Where functions go

We declare most functions at the top level of the file, outside the program start block.
A call to a top-level function can appear before its declaration. In this example,
both calls use the function declared at the end.

```zaka
မျဉ်းဆွဲ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။ // called before its declaration: fine
"ခေါင်းစဉ်" ဟု ပြသမည်။
မျဉ်းဆွဲ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။

// draw a line
မျဉ်းဆွဲ ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"--------" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
--------
ခေါင်းစဉ်
--------
```

## A function inside a function

We can declare a *nested function* (local function) inside the program start block or
another function. A nested function can read and change variables declared before it
in the enclosing block. In this example, each call passes a number in the parameter
`က` and adds it to the same variable, `စုစုပေါင်း`. The [next page](parameters.md)
explains parameters in more detail.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စုစုပေါင်း သည် ၀ ဖြစ်ပါစေ။
// a function inside the program start block: it changes စုစုပေါင်း
ပေါင်းထည့် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စုစုပေါင်း ကို စုစုပေါင်း + က ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပေါင်းထည့် ဟူသော လုပ်ဆောင်ချက်ကို ၅ ဖြင့် လုပ်ဆောင်မည်။
ပေါင်းထည့် ဟူသော လုပ်ဆောင်ချက်ကို ၇ ဖြင့် လုပ်ဆောင်မည်။
စုစုပေါင်း ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၂
```

We can call `ပေါင်းထည့်` from its declaration to the end of the block around it. A function
that uses the variables around it is a *closure*, and [Closures](../function-values/closures.md)
shows how one can even leave its block.

## Leaving before the end

```text
ပြန်ပေးမည်။
```

Write `ပြန်ပေးမည်။` ("will return") to leave a function at once. The program resumes
after the call.

```zaka
စမ်း ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ရောက်သည်" ဟု ပြသမည်။
ပြန်ပေးမည်။
"မရောက်ပါ" ဟု ပြသမည်။ // never runs
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

စမ်း ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
"ပြီး" ဟု ပြသမည်။
```

```text
ရောက်သည်
ပြီး
```

A return at a fixed place does little on its own, but together with
[if](../control-flow/if-else.md) and [parameters](parameters.md) it lets a function stop
when there is nothing left to do.

## Details

- The line breaks in a function signature are plain spaces, so we can also write the whole
  function signature on one line ([Declaring a
  function](../../manual/language/functions.md#declaring-a-function)).
- In a call, we can write `မည့်` in place of `ဟူသော`:
  `နှုတ်ဆက် မည့် လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။` is the same call
  ([Calls](../../manual/language/functions-calls.md#the-word-forms)).
- When the file has a function of that name, the short call also works with no space:
  `နှုတ်ဆက်မည်။` calls `နှုတ်ဆက်`. In a file with no such function, `နှုတ်ဆက်မည်` is just one
  name ([What the flexible grammar
  costs](../../manual/language/functions-sentences.md#what-the-flexible-grammar-costs)).
- The block closer has other spellings too, such as `လုပ်ဆောင်စရာများ ပြီးပါပြီ။` ([Four
  closing forms](../variables/scope.md#four-closing-forms)).
- A call to a name that no function has is the error ZK0303, and two functions with the
  same name and the same parameters are ZK0304 ([Declaring a
  function](../../manual/language/functions.md#declaring-a-function)).
- `ပြန်ပေးမည်။` outside of a function is the error ZK0311.

## Recap

- We write `<name> ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။` to start a
  function and `လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။` to end it, and its statements run when
  we call it.
- We call a function with `<name> ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။` or the short
  form `<name> မည်။`.
- At the top level, a call can come before the declaration of its function.
- A nested function can use and change the variables around it.
- `ပြန်ပေးမည်။` leaves the function at once.

## Hands on

- Write a function that prints a box made of three lines of `#`, and call it twice.
- Call a function whose name was never declared, and read the message.
- Call the box function with the short form, with and without the space before `မည်`.

Next: [Parameters](parameters.md).
