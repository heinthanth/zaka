---
title: Functions
description: How to declare a function, what its parameters and result mean, how a call passes values, and the errors a header or a call can cause.
---

# Functions

A function is a named group of statements. This page states the whole rules: the shape of
a header, the kinds of parameters and results, the two forms of a call, how values
travel into a call, and the errors the compiler reports. For a gentle start, see
[Functions](../../learn/functions/README.md) in the Learn book.

```zaka
// square: one parameter, one result
လုပ်ဆောင်ချက် နှစ်ထပ် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် နှစ်ထပ် ကို ၇ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၉
```

## Declaring a function

```text
လုပ်ဆောင်ချက် <name> သည်
<parameters> လက်ခံပြီး                          (left out when there are none)
အောက်ပါတို့ကို လုပ်ဆောင်မည်။                       (no result)
  or
အောက်ပါတို့ကို လုပ်ဆောင်ကာ <kind> တစ်ခု ကို ပြန်ပေးမည်။   (with a result)
<statements>
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

| Part | Rule |
| --- | --- |
| name | a name like each other name |
| parameters | each `<name> ဟူသော <kind> တစ်<word> ကို`, then its markers; see [Parameters](#parameters) |
| closer of each parameter | `လည်းကောင်း` after each one when there are two or more; after a lone parameter it can stand or be left out |
| result | a single kind, then `တစ်` with the measure word of the kind (`တစ်ခု` for most kinds, `တစ်လုံး` for `စာလုံး`, the classifier of a struct, an enumeration or an alias), then `ကို ပြန်ပေးမည်` |
| end | `ပြီးပါပြီ`, `ပြီးပါပြီး`, `ပြီးဆုံးပါပြီ` or `ပြီးဆုံးပါပြီး`, all with `လုပ်ဆောင်စရာများ` before them |

The line breaks of the header are plain white space. The whole header can stand on one line.

Declaring a function runs nothing. The statements inside run each time the function is
called.

### Where a function stands

- A function with statements is declared at the top level of the program, or inside the
  statements of a function, the start block, a block, a loop or a conditional. A function
  declared inside another one is a *nested function*: it sees from its declaration to the
  end of its block, and it uses the locals around it through their names, sharing them. See
  [Nested functions](function-values.md#nested-functions-and-closures).
- External functions, C functions and the start block are declared at the top level and
  nowhere else (ZK0214).
- A call can come before the declaration it names at the top level. The order of
  declarations does not matter there, and two functions can call each other.
- A file can declare several functions of one name when their parameters differ (see
  [One name, several functions](#one-name-several-functions)); a second function with
  the same name, parameter kinds and marks is ZK0304. Nested functions of one scope have
  no overloads: a second one of a name is ZK0304.
- A call to a name that no function has is ZK0303.

```zaka-error ZK0214
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် abs သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
n ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0214]: external functions, C functions and the start block can be declared at the top level and nowhere else
```

## Parameters

A parameter is a variable that lives inside the function. A call fills it with the value
of the matching argument, so it starts with that value and nothing else. Parameters follow
the rules of other local variables, with one more rule: the names of one header are
distinct (ZK0305).

A parameter can have each kind a variable can have: a number, a true/false value, a
character, text, a list, a struct, a shared object, a function value, or a pointer. What
the call does with the argument depends on that kind; see
[How values are passed](#how-values-are-passed) below.

The argument must fit the kind of its parameter, under the rules of
[assignment](conversions.md): a value that widens without loss converts on its own, and
each other difference needs an explicit conversion.

Each parameter reads `<name> ဟူသော <kind> တစ်<word> ကို` ("the `<kind>` called `<name>`"),
its kind with its [measure word](kinds.md#measure-words), then the markers it takes, in
this order:

| Marker | Meaning |
| --- | --- |
| `ဒေတာ ပြောင်းလဲရန်` | the function reads and changes the caller's place, which the caller sets first; see [Changed parameters](#changed-parameters) |
| `ဒေတာ ထည့်ပေးရန်` | the function writes the caller's place, which the caller need not set; see [Filled parameters](#filled-parameters) |
| `အပိုင်အနေဖြင့်` | the function takes over the block a pointer names: pointers alone, see [Unsafe Zaka](unsafe.md) |
| `ဝိဘတ် "<mark>" ကို သုံး၍` | the word that follows the argument in a call; see [Marked parameters](#marked-parameters) |

```text
param := <name> ဟူသော <kind> တစ်<word> ကို [ဒေတာ ပြောင်းလဲရန် | ဒေတာ ထည့်ပေးရန်] [အပိုင်အနေဖြင့်] [ဝိဘတ် "<mark>" ကို သုံး၍]

<param> လည်းကောင်း <param> လည်းကောင်း … လက်ခံပြီး      (two or more)
<param> [လည်းကောင်း] လက်ခံပြီး                          (one)
```

- With two or more parameters, each ends in `လည်းကောင်း`; one that does not, before
  another parameter or `လက်ခံပြီး`, is ZK0201.
- A parameter holds `ဒေတာ ပြောင်းလဲရန်` or `ဒေတာ ထည့်ပေးရန်`, not both. Each is a phrase of two
  words: `ပြောင်းလဲရန်` or `ထည့်ပေးရန်` alone is ZK0201.
- A marker out of its order is ZK0201.
- The spaces inside a parameter are optional: `ကိုလည်းကောင်း` reads as `ကို လည်းကောင်း`.
- A function kind with parameters stands in parentheses in a parameter:
  `ဖ ဟူသော (ကိန်းပြည့် တစ်ခု ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို` (ZK0239).
- The start block and a C function take the same form. A
  [function kind](function-values.md#the-kind) writes its parameters with no names.
- `အပိုင်အနေဖြင့်` on a parameter whose kind holds no pointer is ZK0470.

```zaka
လုပ်ဆောင်ချက် ပြ သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် သေး သည် ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ပြ ကို သေး တို့ဖြင့် လုပ်ဆောင်မည်။       // 32 bits widens to the full width
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၇
```

## Results and return

A function without a result returns nothing. A function with a result gives its value back
with a return statement:

- `<value> ကို ပြန်ပေးမည်။` gives back the value at once, and the call is that value.
- `ပြန်ပေးမည်။` alone ends a function that has no result.

The result kind is written in the header as `<kind> တစ်<count word> ကို ပြန်ပေးမည်`, for a result kind and an optional kind (`... ရှိနိုင်သော ဒေတာဆဲလ်`, `... ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`)
too; a header with no count word is ZK0234. A count word that does not fit the kind is a warning, ZK0509, and its help
line names the right one. The function has no variable named `ရလဒ်`: a name that is
never declared is ZK0301, and for `ရလဒ်` its help line names the return
`<value> ကို ပြန်ပေးမည်။`.

A function with a result must give back on each path. The block must end in a return, in
a block that gives back, in an `if` with an `မဟုတ်ပါက` arm where each arm gives back, or in a
`switch` with a default where each arm gives back, or in a loop whose test is `မှန်` and
that no break leaves, which never ends. Each other loop is no such end, as its test can be
false at the start. A block that can reach its end with no return is ZK0339:

```zaka-error ZK0339
လုပ်ဆောင်ချက် မေ့ သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0339]: function "မေ့" gives back a result, but its block can reach its end without giving one back
```

The start block is no such function: it can end without a return, and its exit code is
then 0 ([The start block](#the-start-block)).

A result clause can also be a whole result kind, `<kind> သို့မဟုတ် အမှားအယွင်းအနေဖြင့် <kind> ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်`,
or `အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် <kind> ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်` for a good side with no value. Such a function can fail:
`<value> ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။` gives back an error value instead of a result. The
rules are in [Errors and results](error-handling.md).

What the compiler does check:

| Mistake | Code |
| --- | --- |
| a value returned from a function without a result | ZK0312 |
| `ပြန်ပေးမည်။` with no value in a function with a result | ZK0313 |
| a return statement outside each function | ZK0311 |
| a function with a result whose block can reach its end with no return | ZK0339 |
| a value that does not fit the result kind | ZK0401 |

```zaka-error ZK0312
လုပ်ဆောင်ချက် ပြ သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0312]: function "ပြ" returns nothing, but a value is returned
```

```zaka-error ZK0313
လုပ်ဆောင်ချက် တစ် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0313]: function "တစ်" returns isize, but no value is returned
```

## Calls

A call has three forms here, and [Arguments first](#arguments-first) gives two more. The two
word forms name the function after `လုပ်ဆောင်ချက်`, and both close the argument list with
`တို့ဖြင့်` ("with these"), one argument or several. A function with no parameters has no
argument list. The sign form writes the arguments in parentheses after the name.

| Form | Written | Is |
| --- | --- | --- |
| statement | `လုပ်ဆောင်ချက် <name> ကို <arguments> တို့ဖြင့် လုပ်ဆောင်မည်။` (or `လုပ်ဆောင်မည်`) | a statement; a result, if there is one, is dropped |
| expression | `လုပ်ဆောင်ချက် <name> ကို <arguments> တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်` | a value |
| sign form | `<name>(<arguments>)` | a value, the same call as the expression form |

The name can also be a variable, a parameter or a constant that holds a function value,
and the word forms can name a field or a cell that holds one; see
[Function values](function-values.md#calling-a-function-value).

A function whose result clause names a result kind can fail. A call to it
passes the failure on, with `… လုပ်ဆောင်သော ရလဒ်ကောင်း` in place of `… လုပ်ဆောင်သော ရလဒ်`, a `?` after the `)`, or
`၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်` at the end of a call statement; or, with none
of these, the expression form gives back the whole result, which a test and a conversion
read. A call statement with no pass-on is ZK0443. See [Errors and results](error-handling.md).

The expression form is an operand like a number: print it, store it, pass it as an
argument, use it in a sum, or give it back from another function. Using the expression
form of a function that has no result is ZK0315. The number of arguments must match the
number of parameters (ZK0314), and each argument must fit its parameter (ZK0401).

An argument is a full expression, and `၊` ends it, so an argument needs no parentheses:

```zaka
လုပ်ဆောင်ချက် အစီအစဉ် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသမည်။
ခ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် အစီအစဉ် ကို ၁ + ၁၊ ၂ * ၅ တို့ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂
၁၀
```

The sign form is the same call as the expression form, as `+` is the same sum as the
word form: the same order of work, the same checks and the same codes. Its rules:

- The `(` touches the name. With a space, `f (a)`, the name and the group remain apart.
- `၊` comes between the arguments, and `f()` calls a function with no parameters.
- A struct name and `(` is a [struct literal](structs.md#struct-literals); another name and
  `(<field>:` is one too, and the check reports the name.
- A sign-form call is a value, never a statement: a call whose result is dropped takes
  the statement form.

```zaka
လုပ်ဆောင်ချက် နှစ်ထပ် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
နှစ်ထပ်(၇) ကို ပြသမည်။
နှစ်ထပ်(၃) + နှစ်ထပ်(၄) ကို ပြသမည်။
နှစ်ထပ်(နှစ်ထပ်(၂)) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၉
၂၅
၁၆
```

A statement that begins with `လုပ်ဆောင်ချက်` is read as a call statement. A sum or other
expression that begins with a call is written with that call in parentheses; see
[Expressions](expressions.md).

```zaka-error ZK0314
လုပ်ဆောင်ချက် ပြ သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ပြ ကို ၁၊ ၂ တို့ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0314]: function "ပြ" takes 1 arguments, 2 given
```

```zaka-error ZK0401
လုပ်ဆောင်ချက် ပြ သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ပြ ကို မှန် တို့ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0401]: expected isize, found bool
```

## Arguments first

A call can also give its arguments first and name its function last, the order a Burmese
sentence puts an object and its verb in. `သုံး၍` ("using") closes the arguments, with
`ကို` after one and `တို့ကို` after several. The function then comes in a long form or a
short form, and each ends as a statement or as a value:

| Form | Statement | Value |
| --- | --- | --- |
| long | `<arguments> ကို သုံး၍ လုပ်ဆောင်ချက် <name> ကို လုပ်ဆောင်မည်။` | `<arguments> ကို သုံး၍ လုပ်ဆောင်ချက် <name> ကို လုပ်ဆောင်သော ရလဒ်` |
| short | `<arguments> ကို သုံး၍ <name> မည်။` | `<arguments> ကို သုံး၍ <name> သည့် ရလဒ်` |

For a function with no parameters, leave out the arguments and `ကို သုံး၍`: `<name> မည်။`,
`<name> သည့် ရလဒ်`, and the long forms of [Calls](#calls). Each form is the same call as
the forms above, with the same order of work, the same checks and the same codes. A
failure passes on as it does there: `ရလဒ်ကောင်း` in place of `ရလဒ်` after the call, or
`၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်` (or `ပြန်ပေးမည်`) after `မည်` or `လုပ်ဆောင်မည်`.

```zaka
လုပ်ဆောင်ချက် ပေါင်း သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က + ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပြ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ခုနစ် သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
၇ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၃၊ ၄ တို့ကို သုံး၍ ပေါင်း သည့် ရလဒ် ကို ပြသမည်။
၅ ကို သုံး၍ ပြ မည်။
၆ ကို သုံး၍ လုပ်ဆောင်ချက် ပြ ကို လုပ်ဆောင်မည်။
ခုနစ် သည့် ရလဒ်၊ ၁ တို့ကို သုံး၍ ပေါင်း သည့် ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၇
၅
၆
၈
```

The rules of these forms:

- An argument is a notation expression: a name, a literal, a sum, a sign-form call, or a
  short call with no arguments. A sentence that gives a value, another arguments-first
  call included, goes in parentheses.
- `မည်` and `သည့်` are words of their own, with a space between each and the name:
  `ပြမည်` is one name, not a call of `ပြ`.
- `ကို` closes one argument and `တို့ကို` two or more; the other particle is the warning
  ZK0510 ([Natural wording](naturalness.md#the-rules)).
- The long form ends in `လုပ်ဆောင်မည်`; `လုပ်ဆောင်မည်` after `သုံး၍` is ZK0201.
- A statement end, `မည်` or `လုပ်ဆောင်မည်`, where a value is needed is ZK0201.

```zaka-error ZK0201
လုပ်ဆောင်ချက် ပြ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ၅ ကို သုံး၍ ပြ မည် ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0201]: expected "သည့် ရလဒ်" or "သည့် ရလဒ်ကောင်း", found "မည်"
```

## Marked parameters

A parameter can name the word that follows its argument in a call, its *mark*, with
`ဝိဘတ် "<mark>" ကို သုံး၍` ("using the particle …") after its `ကို`:
`လမ်းကြောင်း ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "၌ရှိသော" ကို သုံး၍`. A call then reads as a Burmese
sentence, each argument with its mark after it and the function last:

```text
လုပ်ဆောင်ချက် <name> သည် <param> ဟူသော <kind> တစ်<word> ကို ဝိဘတ် "<mark>" ကို သုံး၍ လည်းကောင်း ... လက်ခံပြီး ...
<argument> <mark> <argument> <mark> <name> မည်။
```

The call ends as the
[arguments-first](#arguments-first) forms do: `<name> မည်` or
`လုပ်ဆောင်ချက် <name> ကို လုပ်ဆောင်မည်` for a statement, `<name> သည့် ရလဒ်` or
`လုပ်ဆောင်ချက် <name> ကို လုပ်ဆောင်သော ရလဒ်` for a value, with `ရလဒ်ကောင်း` and the pass-on
tail as there. A function with marks takes the other call forms too.

```zaka
လုပ်ဆောင်ချက် ရေး သည်
စာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လည်းကောင်း နေရာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "သို့" ကို သုံး၍ လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"{နေရာ}: {စာ}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ဖတ် သည်
လမ်း ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "၌ရှိသော" ကို သုံး၍ လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"{လမ်း} ၏ စာ" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ" ကို "စာအုပ်" သို့ ရေး မည်။
"ဇာတ်လမ်း" ၌ရှိသော ဖတ် သည့် ရလဒ် ကို ပြသမည်။
ကိန်းရှင် စာ သည် "ကဗျာ" ၌ရှိသော လုပ်ဆောင်ချက် ဖတ် ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
စာ ကို ပြသမည်။
လုပ်ဆောင်ချက် ရေး ကို "ဟယ်လို"၊ "မှတ်စု" တို့ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
စာအုပ်: မင်္ဂလာပါ
ဇာတ်လမ်း ၏ စာ
ကဗျာ ၏ စာ
မှတ်စု: ဟယ်လို
```

The rules of marks:

- A mark is one word, written in the string: a name, or a particle such as `ကို`, `မှ`,
  `ဖြင့်`, `နှင့်`, `ထဲသို့`, `တွင်`. `သည်`, `ပါ`, `ဖြစ်ကာ`, `ပြီးလျှင်` and `လက်ခံပြီး` cannot be
  marks, and neither can `၏`. A mark of two words needs them glued: `၌ရှိသော`, not
  `၌ ရှိသော`. A string that is not one such word is ZK0241.
- A header marks each parameter or none of them; marks on some alone are ZK0237.
- A call gives the arguments in the order of the parameters, each with the mark of its
  parameter. Another mark is ZK0331.
- An argument is a notation expression, as in the arguments-first call; a sentence that
  gives a value, another marked call included, goes in parentheses.
- A marked call ends at a call of a function that declares marks, when the number of
  marks fits the function, and nowhere else. So a sentence that holds no such call keeps its
  meaning: `က ကို ခုနစ် သည့် ရလဒ် ဟု ထားမည်။` sets `က` when `ခုနစ်` declares no marks.

```zaka-error ZK0331
လုပ်ဆောင်ချက် ရေး သည်
စာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လည်းကောင်း နေရာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "သို့" ကို သုံး၍ လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စာ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ" ကို "စာအုပ်" ထဲသို့ ရေး မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0331]: argument 2 of "ရေး" is marked "ထဲသို့", not "သို့"
```

```zaka-error ZK0237
လုပ်ဆောင်ချက် ရေး သည်
စာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လည်းကောင်း နေရာ ဟူသော စာသား တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စာ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0237]: some parameters have a mark and some do not
```

## One name, several functions

A file can declare several functions of one name when their parameter kinds or their
marks differ. A call runs the one its arguments fit: an argument for each parameter, the
marks of the parameters when the call writes marks, and each argument fit for the kind of
its parameter, under the rules a call to a single function follows. The marks can tell two
functions apart on their own, so a marked call chooses with the words written too.

```zaka
လုပ်ဆောင်ချက် ပြ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ကိန်း {န}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပြ သည်
စ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"စာ {စ}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပြ သည်
စ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ထဲသို့" ကို သုံး၍ လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"{စ} ထဲသို့" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၅ ကို ပြ မည်။
"ဟယ်လို" ကို ပြ မည်။
"အိမ်" ထဲသို့ ပြ မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ကိန်း ၅
စာ ဟယ်လို
အိမ် ထဲသို့
```

The rules of functions that share a name:

- Two functions of one name with the same parameter kinds and the same marks in one file
  are ZK0304.
- A call that no function of the name fits is ZK0332; the help lists the parameters of
  each.
- A call that two or more fit is ZK0333. A number with no kind fits each integer kind,
  and a call with no marks fits functions that differ in their marks alone: write a kind
  on the number, or the marks of the function meant.
- An import of the name brings each exported function of it. A file cannot declare a
  function of a name it imports (ZK0304).
- Inside the compiler, the second function of a name is `<name>#2`, the third
  `<name>#3`; the IR, the flow checker and the VM listing show these names.

```zaka-error ZK0333
လုပ်ဆောင်ချက် ပြ သည်
စ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပြ သည်
စ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ထဲသို့" ကို သုံး၍ လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ပြ ကို "ဟယ်လို" တို့ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0333]: more than one function "ပြ" takes these arguments
```

```zaka-error ZK0332
လုပ်ဆောင်ချက် ပြ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပြ သည်
စ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
မှန် ကို ပြ မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0332]: no function "ပြ" takes these arguments
```

## External functions

A function can come from outside the program, `ပြင်ပမှလာသော`: its header names its
parameters and its result, and no statements follow it, since the VM does its work while
the program runs. The header is one sentence:

| Parameters | Result | Written |
| --- | --- | --- |
| none | none | `လုပ်ဆောင်ချက် <name> သည် ပြင်ပမှလာသော လုပ်ဆောင်ချက်ဖြစ်သည်။` |
| some | none | `... လုပ်ဆောင်ချက်ဖြစ်၍ <parameters> လက်ခံမည်။` |
| none | a result | `... လုပ်ဆောင်ချက်ဖြစ်၍ <kind> တစ်ခု ကို ပြန်ပေးမည်။` |
| some | a result | `... လုပ်ဆောင်ချက်ဖြစ်၍ <parameters> လက်ခံပြီး <kind> တစ်ခု ကို ပြန်ပေးမည်။` |

`လုပ်ဆောင်ချက်` can take `တစ်ခု` after it (`လုပ်ဆောင်ချက်တစ်ခုဖြစ်၍`), and `ပြင်ပမှလာသော`
can be written apart (`ပြင်ပ မှ လာသော`). The parameters are those of a regular header,
marks included, and so is the result kind. A call of an external function is a call
like each other, in each call form.

The VM serves a fixed set of functions, so an external function must be one of them: the
same name, the same parameter kinds, and a result that can fail with the error
enumeration of input and output. Another one is ZK0334, whose help lists the set. The
stdlib declares each of them, so a program imports them from there rather than
writing the headers.

```zaka-error ZK0334
လုပ်ဆောင်ချက် ဟိုဟို သည် ပြင်ပမှလာသော လုပ်ဆောင်ချက်ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0334]: the VM serves no function "ဟိုဟို" with these parameter and result kinds
```

## C functions

A text with the name of a shared lib in place of `ပြင်ပ` makes a C function:
`"libm" မှလာသော`. The function is the C function of its name in that lib, so its name is a
C name: ASCII letters, digits and `_`, not starting with a digit (ZK0337). Each C function
names its lib, libc too (`"libc"`). The header has the forms of the table above, the
marks of its parameters included, and a call is a call like each other.

```zaka
လုပ်ဆောင်ချက် abs သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
n ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် abs ကို -၄၂ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၂
```

C takes each parameter and gives its result as a whole value, laid out as in C, so the
kinds that cross are those C has:

| Kind | In C | The mark |
| --- | --- | --- |
| integers of 8 to 64 bits, `ကိန်းပြည့်` | `int8_t` to `int64_t`, `uint8_t` to `uint64_t` | not needed |
| `၃၂ ဘစ် ဒဿမကိန်း`, `ဒဿမကိန်း` | `float`, `double` | not needed |
| `မှန်မှားတန်ဖိုး` | `_Bool` | not needed |
| a struct whose fields are these kinds, structs or counted lists of them | a struct, passed whole | not needed |
| `စာသား`, as a parameter | `const char *` | not needed |
| a plain or counted list of numbers, as a parameter | the address of its first cell | not needed |
| a function kind whose parameters and result cross, as a parameter | a pointer to a function | not needed |
| a kind from outside the program, as a parameter or a result (see [Kinds from outside the program](#c-functions)) | a pointer to a struct C keeps | not needed |
| `<kind from outside> ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`, as a result | a pointer to a struct C keeps, NULL for the optional with no data | not needed |
| a pointer, `<kind> လိပ်စာ`, or a struct that holds one | a pointer | needed |
| `စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`, as a result | `const char *`, NULL for the optional with no data | not needed |

### The mark on a C function

A C function that takes or gives a pointer can reach
storage that Zaka does not check, so its declaration carries the mark `စိတ်မချရသော`, as a
function of the program can ([Unsafe Zaka](unsafe.md)). Without the mark it is ZK0472.
Each call of a marked C function then needs marked code: a marked function, or a block
marked `စိတ်မချရသော်လည်း` (ZK0468 elsewhere).

A C function with none of these kinds needs no mark, and safe code calls it: its
parameters are numbers, true or false, text, lists of numbers, plain structs, function
values or handles of a kind from outside the program, and its result is nothing, a
number, a plain struct, such a handle, or an optional of text or of such a handle (C gives
NULL for the optional with no data, and Zaka holds no address).

```zaka-error ZK0472
လုပ်ဆောင်ချက် frexp သည် "libm" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
x ဟူသော ဒဿမကိန်း တစ်ခု ကို လည်းကောင်း e ဟူသော ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း လက်ခံပြီး ဒဿမကိန်း တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0472]: "frexp" passes a pointer, so its declaration needs the mark "စိတ်မချရသော"
```

```zaka-error ZK0468
စိတ်မချရသော လုပ်ဆောင်ချက် frexp သည် "libm" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
x ဟူသော ဒဿမကိန်း တစ်ခု ကို လည်းကောင်း e ဟူသော ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း လက်ခံပြီး ဒဿမကိန်း တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ထပ်ကိန်း သည် ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် frexp ကို ၈.၀၊ ထပ်ကိန်း ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0468]: a call of "frexp", a function marked "စိတ်မချရသော", needs marked code
```

### Text

C reads a string up to a zero octet, which Zaka text does not hold, so a text parameter
reaches C as the address of a duplicate of its u8s with a zero octet after them. The duplicate
lives until the call ends: C reads it but cannot change the text, and a zero octet
inside the text ends the string there for C.

```zaka
လုပ်ဆောင်ချက် strlen သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် strlen ကို "မောင်" တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၅
```

Each other kind is ZK0465: plain lists of other cells, which C has no form for; shared
objects; function values that do not cross (see [Callbacks](#c-functions)); text as a bare
result, since C can give back NULL for it, and its handle likewise (see
[Text as a result](#c-functions)); and another result that can fail.

```zaka-error ZK0465
လုပ်ဆောင်ချက် getenv သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
n ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး စာသား တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0465]: a C function cannot take or give back a value of the kind "text"
```

Each declaration of one C name, in each file and among overloads, gives it the same
parameter and result kinds (ZK0338). Like each function that comes from outside, a C
function cannot be a function value (ZK0464): a function of the program that calls it
stands in for it.

### Lists of numbers

A plain or counted list whose cells are integers of up to 64 bits, `၃၂ ဘစ် ဒဿမကိန်း` or
`ဒဿမကိန်း` reaches C as the address of its first cell, as C takes a run of cells: `const T *`.
The cells remain in place for the call, so C reads them there.

A list of numbers marked `ဒေတာ ပြောင်းလဲရန်` or `ဒေတာ ထည့်ပေးရန်` reaches C so too, and C
writes its cells in place: `T *`. A plain list first takes its cells for its own, so no duplicate of the list
sees what C writes. This is how C gives back more than one value. A struct that crosses
to C, marked `ဒေတာ ပြောင်းလဲရန်`, reaches C as its address, `T *`, and C writes its fields in
place, as a C function that fills a struct its caller gives it does. `ဒေတာ ပြောင်းလဲရန်` or
`ဒေတာ ထည့်ပေးရန်` on a parameter of another kind is ZK0466: C takes each other value whole, so
it has no place to write back to.

Mark a place that C reads, or reads and writes, `ဒေတာ ပြောင်းလဲရန်`, as `memset` takes its
list: the caller sets it first. Mark a place that C writes and never reads
[`ဒေတာ ထည့်ပေးရန်`](#filled-parameters), as `frexp` takes its exponent: the caller declares
it with no value, and it counts as set after the call. The compiler cannot see inside C,
so the mark is the author's promise that C writes the whole place.

```zaka
လုပ်ဆောင်ချက် strlen သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ကို လက်ခံပြီး လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် memset သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
b ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လည်းကောင်း c ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း n ဟူသော လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံမည်။
လုပ်ဆောင်ချက် frexp သည် "libm" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
x ဟူသော ဒဿမကိန်း တစ်ခု ကို လည်းကောင်း e ဟူသော အကွက် ၁ ခုရှိသော ၃၂ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ထည့်ပေးရန် လည်းကောင်း လက်ခံပြီး ဒဿမကိန်း တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၀၄၊ ၁၀၅၊ ၀] ဖြစ်ပါစေ။
ကိန်းရှင် ပွား သည် စာ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် strlen ကို စာ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်ချက် memset ကို စာ၊ ၇၊ ၂ တို့ဖြင့် လုပ်ဆောင်မည်။
// ပွား keeps the cells it had
"{စာ} {ပွား}" ဟု ပြသမည်။
ကိန်းရှင် ထပ်ကိန်း သည် အကွက် ၁ ခုရှိသော ၃၂ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် frexp ကို ၈.၀၊ ထပ်ကိန်း တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ထပ်ကိန်း ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂
[၇၊ ၇၊ ၀] [၁၀၄၊ ၁၀၅၊ ၀]
၀.၅
[၄]
```

```zaka-error ZK0466
လုပ်ဆောင်ချက် puts သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော စာသား တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0466]: the C function cannot change the parameter "s" in place
```

C knows no count of cells: it writes as far as it is told, so a C function that writes
past the last cell harms the run, as it would harm a C program.

### Text as a result

A C function that gives back a string writes its result as `စာသား ရှိကောင်းရှိနိုင်သော
ဒေတာဆဲလ် တစ်ခု` (an [optional](optionals.md)), and its declaration needs no mark: the program
gets a duplicate and never the address. NULL is the optional with no data, `အလွတ်`; each other
address is the text, as its data. Zaka copies the string up to its zero octet into new text of
its own. The old form, `စာသား သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ်`, is ZK0465, and
its help line names the optional. Octets that are no UTF-8 stop the program with the
[not UTF-8 trap](../vm/traps.md).

Who frees the string of C depends on the result clause:

| Result clause | C's string | For |
| --- | --- | --- |
| `... ကို ပြန်ပေးမည်` | C keeps it; Zaka never frees it | `getenv`, `strerror`, `strstr`, static buffers |
| `... ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်` | C hands it over; Zaka frees it with libc's `free` once its text holds the octets | `strdup` and each function that allocates the string it gives |

`အပိုင်အနေဖြင့်` stands on a C function whose result is an optional of text; on each other
C function it is ZK0467. Without it, a string that C allocates is never freed.

Zaka takes the text before it frees the duplicates of the text arguments, so a result
that points into an argument, as `strstr` gives, is copied while it is still there.

```zaka
လုပ်ဆောင်ချက် strstr သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော စာသား တစ်ခု ကို လည်းကောင်း part ဟူသော စာသား တစ်ခု ကို လည်းကောင်း လက်ခံပြီး စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
// strdup hands its string over
လုပ်ဆောင်ချက် strdup သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် strstr ကို "မင်္ဂလာပါ ကမ္ဘာ"၊ "ကမ္ဘာ" တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်ချက် strstr ကို "abc"၊ "x" တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်ချက် strdup ကို "စံကား" တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဒေတာ("ကမ္ဘာ")
အလွတ်
ဒေတာ("စံကား")
```

A program can also take the string as a pointer and free it itself, as Rust and Go
programs do: read its length with `strlen`, bring its octets into a list of its own with
`memcpy`, free it with `free`, and read the cells as text. The VM cannot read through an
address C gives, so `memcpy`, a C function, copies the octets. A range of the list gives
a plain list of u8s, which converts to text after a check of its UTF-8
([Text](text.md)).

```zaka
စိတ်မချရသော လုပ်ဆောင်ချက် strdup သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို ပြန်ပေးမည်။
စိတ်မချရသော လုပ်ဆောင်ချက် strlen သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လက်ခံပြီး လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
စိတ်မချရသော လုပ်ဆောင်ချက် memcpy သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
dst ဟူသော အကွက် ၆၄ ခုရှိသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လည်းကောင်း src ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း
n ဟူသော လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို ပြန်ပေးမည်။
စိတ်မချရသော လုပ်ဆောင်ချက် free သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
p ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လက်ခံမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် လုပ်ဆောင်ချက် strdup ကို "zaka" တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် ရေ သည် လုပ်ဆောင်ချက် strlen ကို ပ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် ဘောင် သည် အကွက် ၆၄ ခုရှိသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် ပြန် သည် လုပ်ဆောင်ချက် memcpy ကို ဘောင်၊ ပ၊ ရေ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် free ကို ပ တို့ဖြင့် လုပ်ဆောင်မည်။
(ဘောင် ၏ အကွက် ၀ မှစ၍ ရေ ကွက်) ကို စာသား တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
zaka
```

The list here holds 64 octets; a longer string needs a larger list, or a block from the
heap.

### Callbacks

A parameter of a function kind crosses to C as a pointer to a function. The result of the
kind is an integer up to 64 bits, a `၃၂ ဘစ် ဒဿမကိန်း`, a `ဒဿမကိန်း`, a `မှန်မှားတန်ဖိုး` or a
pointer, or the kind gives back nothing. Each parameter of the kind is one of these, or:

| Parameter | What C passes | What the function gets |
| --- | --- | --- |
| a plain struct | the struct, as a whole value | a struct with the same fields |
| `စာသား` | a `char *` | text copied from the string; NULL stops the program with the trap `null from C` |
| `စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` | a `char *` | an optional: `အလွတ်` for NULL, else the copied text |
| a kind from outside | the address of a struct C keeps | a new handle around the address; NULL stops the program with the trap `null from C`, since the parameter promised a handle |
| an optional of a kind from outside | the address, or NULL | an optional: `အလွတ်` for NULL, else a new handle |

The kind has no `ဒေတာ ပြောင်းလဲရန်` marker. Each other function kind is ZK0465. The function gets
each of these values lent, as a call of the program lends its arguments: C keeps the
string and the struct it passed, and the copies go once the function returns. A handle
that C passed belongs to C, so the function cannot close it in safe code: giving a handle
parameter to a C function that takes it over with `အပိုင်အနေဖြင့်` is ZK0468 outside marked
code.

```zaka-file
// callbacks.zaka
ဘူး ဟူသော ပြင်ပမှ လာသော ဒေတာအစုအဝေးတွင် n ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ပါဝင်သည်။
လုပ်ဆောင်ချက် each_name သည် "libnames" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
f ဟူသော (စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို လက်ခံမည်။
လုပ်ဆောင်ချက် each_box သည် "libnames" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
f ဟူသော (ဘူး တစ်ခု ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို လက်ခံမည်။

လုပ်ဆောင်ချက် အမည်ပြ သည် အမည် ဟူသော စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ အမည် ထဲတွင် ဒေတာ ရှိလျှင် အမည် ထဲမှ ဒေတာ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
``` The argument is a
function of the program named with `ဟူသော`, or a variable that holds one; C never gets
NULL for it, since a function value holds a function at each point. A closure can go too: it is lent to the call, so C can call it while the
call runs. A C function that keeps the function and calls it later needs the closure to
have leave, which the statement `f ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်` gives and
`f ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြန်လည်ရုပ်သိမ်းမည်` takes back; a call from C with no call running
and no leave stops the program with the trap `closure_escape`. See
[Closures and C](function-values.md#nested-functions-and-closures).

```zaka
စိတ်မချရသော လုပ်ဆောင်ချက် qsort သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
base ဟူသော ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း count ဟူသော လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း size ဟူသော လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
compare ဟူသော (၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို လည်းကောင်း
လက်ခံမည်။

// C calls it with the addresses of two cells
လုပ်ဆောင်ချက် ငယ်စဉ် သည်
က ဟူသော ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း ခ ဟူသော ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
(က ၏ တန်ဖိုး) - (ခ ၏ တန်ဖိုး) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် အကွက် ၅ ခုရှိသော ၃၂ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၅၊ ၂၊ ၉၊ ၁၊ ၇] ဖြစ်ပါစေ။
ကိန်းရှင် အရွယ် သည် ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ၏ အရွယ်အစား ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် qsort ကို ကိန်းများ ၏ အကွက် ၀ ၏ လိပ်စာ၊ ၅၊ အရွယ်၊ ငယ်စဉ် ဟူသော လုပ်ဆောင်ချက် တို့ဖြင့် လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ကိန်းများ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၂၊ ၅၊ ၇၊ ၉]
```

Each build gives C a pointer it can call:

- `zakac run` makes one native closure for each function a call gives C, on the first
  such call, and keeps it for the run. The callback runs the function in the VM until it
  returns.
- A native build passes the code address of a function with no captures. A closure goes
  through the native closure of libffi, which the build links in with `-lffi` (see
  [Native builds](../compiler/native.md)); a program with no closure never needs it.

A trap inside a callback waits until C returns, then stops the program; each later
callback of that C call gives 0. Text that a callback prints in the VM comes out after
the output of the C call itself, where a native build keeps the order. A callback must
not resize or free a block that C holds an address into, since the native address goes
stale.

### Other values

A C function whose parameters end in `...`, as `printf` does, takes more arguments than
it names. Its header ends the parameters with `အခြားတန်ဖိုးများနှင့်အတူ` ("together with
other values") before `လက်ခံပြီး`, after the last parameter it names; each parameter before
it ends in `လည်းကောင်း`, even a lone one, and a space can stand before `နှင့်အတူ`:

```text
... fmt ဟူသော စာသား တစ်ခု ကို လည်းကောင်း အခြားတန်ဖိုးများနှင့်အတူ လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
```

A call gives an argument for each parameter, then each other value it wants (too few is
ZK0314). An other value has the kind it has on its own: a number literal is a
`ကိန်းပြည့်` or a `ဒဿမကိန်း`. C reads it as C promotes it: an integer narrower than 32
bits and a boolean as an `int`, a `၃၂ ဘစ် ဒဿမကိန်း` as a `double`. Text goes as a duplicate
that ends in a zero octet, a list of numbers as the address of its first cell, and a
struct goes whole, so C reads it as it reads a struct parameter. The format must
match the kinds, as in C: `%ld` for a `ကိန်းပြည့်`, which has 64 bits, and `%d` for a
`၃၂ ဘစ် ကိန်းပြည့်` or a boolean. Functions the host serves take no other values (ZK0334).

A C function that takes other values can mark a list of numbers `ဒေတာ ပြောင်းလဲရန်`, so `snprintf` writes its
buffer from safe code:

```zaka
လုပ်ဆောင်ချက် snprintf သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
buf ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လည်းကောင်း size ဟူသော လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
fmt ဟူသော စာသား တစ်ခု ကို လည်းကောင်း အခြားတန်ဖိုးများနှင့်အတူ လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဘောင် သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [] ဖြစ်ပါစေ။
၁ မှ ၃၂ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဘောင် ထဲသို့ ၀ ကို ထည့်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ကိန်းရှင် ရေ သည် ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် snprintf ကို ဘောင်၊ ၃၂၊ "%s=%ld"၊ "x"၊ ၄၂ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
(ဘောင် ၏ အကွက် ၀ မှစ၍ ရေ ကွက်) ကို စာသား တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
x=42
```

### Kinds from outside the program

Some C libs keep a struct of their own and give the program a pointer to it: a database
connection, a counter, a window. Zaka names such a struct with a *kind from outside*,
declared with the phrase `ပြင်ပမှ လာသော ဒေတာအစုအဝေး` ("a struct that comes from
outside"; `ပြင်ပမှလာသော ဒေတာအစုအဝေး` is the same) after its name and `ဟူသော`. A value of it is a *handle*. The
declaration has two forms. The first names no field, so the program passes the handle
back to C and nothing more; the second lists the fields of the C struct that the program
reads:

```text
ဒေတာဘေ့စ် ဟူသော [အများသုံး] ပြင်ပမှ လာသော ဒေတာအစုအဝေး ဖြစ်သည်။
ရက်စွဲ ဟူသော [အများသုံး] ပြင်ပမှ လာသော ဒေတာအစုအဝေးတွင် <field> ဟူသော <kind> တစ်<word>၊ … တို့ပါဝင်သည်။
```

A lone field closes with `ပါဝင်သည်`, with no `တို့`, as in a struct.

A handle is a shared box, as an object of a `မျှဝေနိုင်သော` struct is
([Values and objects](objects.md#shared-objects)), and the one field the box holds is the
address C gave. Copies made with `ဖြစ်ပါစေ` and `ဟု ထားမည်` share that one box. A variable of
the kind declared with no value is unset until the program gives it a handle, as each
variable is ([Set before read](statements.md#set-before-read)), and a handle has no
[initial value](kinds.md#initial-values): a handle comes from C alone, so `ကနဦးတန်ဖိုး` for
one is ZK0479.

**What C functions take and give.** A C function can take a handle as a parameter and give
one back, and its declaration needs no mark `စိတ်မချရသော`: the program never holds the
address, so it cannot reach through it. A result written `<kind> တစ်ခု ကို ပြန်ပေးမည်`
wraps the address C gives. Such a plain result promises that C gives no NULL: when C gives
NULL all the same, the run stops with the trap [null from C](../vm/traps.md#c-functions),
since no variable holds a NULL handle.
A result written `<kind> ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်` gives the
optional with no data for NULL and the handle, as data, for each other address. A handle can also be one of the
other values of a C function that takes them, such as `printf`: C gets the address it
holds, as for a parameter.

**Closing is manual.** Zaka never closes a handle on its own. A C function closes one
through a parameter marked `အပိုင်အနေဖြင့်` ("as its own"): the C function takes the
handle over. After the call the box holds 0, so each duplicate of the handle reads as closed,
and the variable the caller named shares that box, so it reads as closed too. Use of a
closed handle, as an argument of a C function or through a field read, stops the program
with the trap [closed handle](../vm/traps.md#c-functions).

**Fields.** A declaration of the second form lists the fields of the C struct from its
start; a prefix is enough, since the program reads just the fields it names. Each field has
one of these kinds, and each other kind is ZK0465:

| Field kind | In C |
| --- | --- |
| an integer of 8 to 64 bits | `int8_t` to `int64_t`, `uint8_t` to `uint64_t` |
| `၃၂ ဘစ် ဒဿမကိန်း`, `ဒဿမကိန်း` | `float`, `double` |
| `မှန်မှားတန်ဖိုး` | `_Bool` |
| a counted list of numbers | a run of cells inside the struct, as `int32_t marks[2]` |
| `စာသား` | `char *` |
| a kind from outside | a pointer to the struct C keeps for that kind |

C places the fields: each at the next multiple of its alignment, and a text field or a
handle field is a pointer. The names are the program's own; the order and the kinds must match the C struct.

`h ၏ <field>` reads the field from C's storage each time, so Zaka keeps no duplicate: a
read after C changed the struct sees the change. A counted list is read as a list of
its own with the cells of that moment. A text field reads as an optional of text, `စာသား
ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`: Zaka duplicates the string up to its zero octet, and
a NULL `char *` is the optional with no data. C can give `""` for no text, so the two differ. A field of a
kind from outside reads as an optional of a new handle, `<kind>
ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`, with NULL for the optional with no data; the struct it points at is C's, so a
program closes it, when C asks for that, through the C function that closes it.

Safe code writes no field: C owns the struct, so a program changes it through a C
function, and a write is ZK0474. [Marked code](unsafe.md) can write a field of a number, a
boolean or a counted list with `… ၏ <field> ကို … ဟု ထားမည်။`; the value goes into C's
storage in place, after the same check that the handle is open. A text field or a handle
field is a pointer C owns, so no code writes it (ZK0474).

The program below declares a counter that a lib keeps. `counter_open` gives a handle, or the
optional with no data for a step of 0; `counter_next` adds one step; `counter_close` takes the handle over.
The fields `count`, `step` and `label` are the first three of the C struct. The program
cannot run in the book, which has no such lib, so it is shown as a file:

```zaka-file
// counter.zaka
ရေတွက်စက် ဟူသော ပြင်ပမှ လာသော ဒေတာအစုအဝေးတွင်
count ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု၊ step ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု၊ label ဟူသော စာသား တစ်ခု တို့ပါဝင်သည်။

လုပ်ဆောင်ချက် counter_open သည် "libcounter" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
step ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ရေတွက်စက် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် counter_next သည် "libcounter" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
c ဟူသော ရေတွက်စက် တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် counter_close သည် "libcounter" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
c ဟူသော ရေတွက်စက် တစ်ခု ကို အပိုင်အနေဖြင့် လက်ခံမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စက် သည် လုပ်ဆောင်ချက် counter_open ကို ၅ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ထဲမှ ဒေတာ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် counter_next ကို စက် တို့ဖြင့် လုပ်ဆောင်မည်။
"{စက် ၏ count} {စက် ၏ step}" ဟု ပြသမည်။
စက် ၏ label ကို ပြသမည်။
လုပ်ဆောင်ချက် counter_close ကို စက် တို့ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

A program cannot do three things with a handle, and each is ZK0473: make one
(`ရေတွက်စက်()`), duplicate the struct behind it (`၏ မိတ္တူ`, `ကူးထည့်မည်`, a deep duplicate), and
print it, a handle inside an optional too. Assigning a handle is fine, since the
variables share the box; a duplicate of the struct is what Zaka cannot make, as the
handle holds just the address.

```zaka-error ZK0473
ပြတင်းပေါက် ဟူသော ပြင်ပမှ လာသော ဒေတာအစုအဝေး ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ပြတင်းပေါက်() ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0473]: "ပြတင်းပေါက်" comes from outside the program, so Zaka cannot make a value of it
```

```zaka-error ZK0474
ရက်စွဲ ဟူသော ပြင်ပမှ လာသော ဒေတာအစုအဝေးတွင် နှစ် ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

လုပ်ဆောင်ချက် ပြင် သည် ရ ဟူသော ရက်စွဲ တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ရ ၏ နှစ် ကို ၂၀၂၅ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0474]: a field of "ရက်စွဲ" belongs to the struct C keeps, so Zaka reads it but cannot write it
```

Neither build declares the C struct: a read takes the address of the struct and adds the
offset of the field. The VM does it with the op [`c_load`](../vm/instructions.md), and a
native build with `memcpy` (see [Native builds](../compiler/native.md)).

### The lib

The text before `မှလာသော` is a lib name or a path to the file of a lib. A name, such as
`"libsqlite3"`, is looked up where the host keeps its libs. A path holds a `/` or a `\`,
or ends in `.so`, `.dylib` or `.dll`, such as `"./libpair.so"` or `"/opt/x/libfoo.dylib"`.
A relative path starts at the folder of the file that declares the function, as an import
does, so the program finds its lib wherever it runs from.

### How each build calls C

- `zakac run` opens each lib as the program first calls a C function: `libX` is the file
  `libX.dylib`, `libX.so` or `X.dll`, a path opens as it is, and a C name the lib does not
  hold is looked up in the process, which holds libc. An address that lies in a block of
  the VM reaches C as the native address of that octet, so C reads and writes the block in
  place. After each call the VM writes out what C holds back for its streams, so the text
  a C function prints comes in order with the text the program prints. A lib that does
  not open, or a C name no lib holds, stops the program with `ForeignLink`.
- The VM in the browser opens no lib: a call of a C function stops the program with
  `ForeignCall`.
- `zakac build --target c` declares each C function in a C file of its own and links
  `-lX` for each lib `libX` other than libc and libm, and a lib given as a path as it is,
  with its folder as a run path. Give the folder of a named lib with
  `--ld-flag -L<folder>`. On macOS a lib given as a path must name itself with an
  absolute path or `@rpath/...` (its install name), since a bare file name there is
  looked up in the folder the program starts from.

C does what C does: a C function that writes past a block, or keeps an address after
the call, harms the run as it would harm a C program. A native address that C gives
back, and that lies in no block, passes back to C as it is, but the VM cannot read
through it. C can call back into the program through a function value of a kind that
crosses (see [Callbacks](#c-functions)).

## Recursion

A function can call itself, at once or through other functions. Nothing marks a function
as recursive; the call is a plain call. Each call has its own parameters and its own
locals.

```zaka
// even and odd, each defined with the other
လုပ်ဆောင်ချက် စုံသလား သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် မှန် ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် မကိန်းလား ကို န - ၁ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် မကိန်းလား သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် မမှန် ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် စုံသလား ကို န - ၁ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် စုံသလား ကို ၁၀ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်ချက် စုံသလား ကို ၇ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
မမှန်
```

The VM allows 4096 calls that wait to return at one time; the next call stops the
program. `zakac run --max-depth <calls>` sets another limit. A native program, built with
`zakac build --target c`, counts calls just so under the default checks and takes
`--max-depth` too ([Native programs](../compiler/native.md#checks)). A recursion without a base
case meets this limit:

```zaka-stop
လုပ်ဆောင်ချက် အဆုံးမဲ့ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် အဆုံးမဲ့ ကို န + ၁ တို့ဖြင့် လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် အဆုံးမဲ့ ကို ၁ တို့ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: more than 4096 nested calls
```

See [When a program stops](../vm/traps.md) for the other stops.

## Lists and structs as results

A function can give back a list or a struct. The result follows the rules of an
assignment: the caller gets the whole value, as a duplicate of its own. See
[Values and objects](objects.md).

```zaka
နေရာ ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု၊ y ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။
လုပ်ဆောင်ချက် နေရာသစ် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ နေရာ တစ်ခု ကို ပြန်ပေးမည်။
x က၊ y ခ ရှိသော နေရာတစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် အစဉ် သည်
စ ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
၀ မှ ၂ အထိ န အနေဖြင့် ရေတွက်၍ အဖြေ ၏ အကွက် န ကို စ + န ဟု ထားမည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် နေရာသစ် ကို ၃၊ ၄ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်ချက် အစဉ် ကို ၅ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
နေရာ(x: ၃၊ y: ၄)
[၅၊ ၆၊ ၇]
```

- A plain list result takes its count from the value given back. A counted list
  result has a written count.
- A function can give back a [range](lists.md) of a local list: a range is a list of its
  own.
- The caller keeps a slot for the result of each call. With the default,
  `--results in-place`, the function builds a returned literal, or the local that each
  return gives back, straight in that slot. With `--results duplicate`, the function keeps
  the value in its own frame and the return duplicates it into the slot. The program prints the same in
  both modes; see [Commands](../compiler/commands.md#options-of-the-compile-stages).
- A pointer result can hold `အပိုင်အနေဖြင့်`, `<kind> တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်`: the
  caller then owns the block. See [Unsafe Zaka](unsafe.md).

## Changed parameters

A parameter marked `ဒေတာ ပြောင်းလဲရန်` ("to change the data") changes the caller's
place: the function starts from the value in the place, and what it leaves in the
parameter goes back to the variable, field or cell the caller gave. So the caller sets the
place before the call, and a place that can be unset there is ZK0722. A function that
writes the place and never reads it takes a [filled parameter](#filled-parameters)
instead. The call writes the argument as it writes each other one, with no mark,
in the word form and the sign form alike. The language server shows such an argument with
a hint.

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။
လုပ်ဆောင်ချက် တိုး သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို န + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ၅ ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် လူ(အသက်: ၃၀) ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် တိုး ကို က တို့ဖြင့် လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် တိုး ကို ပ ၏ အသက် တို့ဖြင့် လုပ်ဆောင်မည်။
က ကို ပြသမည်။
ပ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၆
လူ(အသက်: ၃၁)
```

Two changed parameters swap two variables, and a changed list grows in the caller:

```zaka
လုပ်ဆောင်ချက် လဲလှယ် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ယာယီ သည် က ဖြစ်ပါစေ။
က ကို ခ ဟု ထားမည်။
ခ ကို ယာယီ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် နှစ်ဆတိုး သည်
ကိန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ (ကိန်းများ ၏ အကွက်အရေအတွက်) == ၀ ဖြစ်လျှင် ပြန်ပေးမည်။
၀ မှ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၁ အထိ န အနေဖြင့် ရေတွက်၍ ကိန်းများ ၏ အကွက် န ကို (ကိန်းများ ၏ အကွက် န) * ၂ ဟု ထားမည်။
ကိန်းများ ထဲသို့ ၀ ကို ထည့်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပထမ သည် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် ဒုတိယ သည် ၂ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် လဲလှယ် ကို ပထမ၊ ဒုတိယ တို့ဖြင့် လုပ်ဆောင်မည်။
"{ပထမ} {ဒုတိယ}" ဟု ပြသမည်။
ကိန်းရှင် မူရင်း သည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် နှစ်ဆတိုး ကို မူရင်း တို့ဖြင့် လုပ်ဆောင်မည်။
မူရင်း ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂ ၁
[၂၊ ၄၊ ၆၊ ၀]
```

The parameter works on a value of its own, which goes back to the caller's place when the
call returns, as Swift's `inout` does: copied in, copied out. So the function never holds the
place itself, and a place the function also reaches through another route remains sound:
here the function adds to the list whose cell it changes, and the new value lands in the
cell after the call.

```zaka
ကိန်းရှင် ဂရစ် သည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။

လုပ်ဆောင်ချက် ချဲ့ သည်
ပ ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဂရစ် ထဲသို့ ၄ ကို ထည့်မည်။
ပ ကို ၉၉ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ချဲ့ ကို ဂရစ် ၏ အကွက် ၀ တို့ဖြင့် လုပ်ဆောင်မည်။
ဂရစ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၉၉၊ ၂၊ ၃၊ ၄]
```

The compiler passes the place's address straight when nothing else can reach it, a local
that no closure shares, so the common case costs nothing more. A place under a global or
a captured variable is copied into a hidden slot, which goes back to the place right after
the call; the place is found again then, so a cell lands where the list holds it after
the call.

The rules:

- The argument is a place: a variable, a field or a cell (ZK0410 for another expression).
  A constant cannot be changed (ZK0306).
- The place is set before the call (ZK0722), since the function can read it.
- The place has the kind of the parameter, with no widening (ZK0401).
- The place appears in no other argument of the same call, as a whole or through a field
  or a cell (ZK0433): the order in which the values go back would decide the result.
- A closure inside the function that uses the parameter shares the function's value; what
  the parameter holds when the function returns goes back to the caller.
- A call through a function value whose kind has `ဒေတာ ပြောင်းလဲရန်` changes its argument as a
  direct call does.
- With `အပိုင်အနေဖြင့်` too, on a pointer, the function takes over the old block and the
  caller owns whatever is written back ([Unsafe Zaka](unsafe.md)).

Returning the new value, `က ကို နောက်တစ်(က) ဟု ထားမည်။`, does the same work for one
value. A changed parameter suits a function that changes several places at once.

```zaka-error ZK0410
လုပ်ဆောင်ချက် တိုး သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို န + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် တိုး ကို ၅ တို့ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0410]: this cannot be assigned to; a target must be a variable or a dereference
```

```zaka-error ZK0433
လုပ်ဆောင်ချက် ထား သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လည်းကောင်း မ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို မ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ၅ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ထား ကို က၊ က + ၁ တို့ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0433]: "က" is changed through another argument of this call, so it cannot appear here
```

## Filled parameters

A parameter whose item holds `ဒေတာ ထည့်ပေးရန်` ("to put the data in") hands the function a
place of the caller to fill, as C#'s `out` does. The caller need not set the place: it can
declare the variable with no value, and the place counts as set after the call. The
function gives back more than one value so, each in a place of its own:

```zaka
လုပ်ဆောင်ချက် ခွဲ သည်
တန် ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လဒ် ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ထည့်ပေးရန် လည်းကောင်း ကြွင်း ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ထည့်ပေးရန် လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လဒ် ကို တန် / ၃ ဟု ထားမည်။
ကြွင်း ကို တန် % ၃ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။          // no value: the call sets it
ကိန်းရှင် ကျ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ခွဲ ကို ၁၀၊ လ၊ ကျ တို့ဖြင့် လုပ်ဆောင်မည်။
"{လ} {ကျ}" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၃ ၁
```

Inside the function the parameter starts unset, as a variable declared with no value
does. The function sets the whole of it on each path before it returns, a struct one
field at a time or whole, and reads it after it sets it, never before:

- A read before the parameter is set is ZK0722.
- A return, or the end of the function, that a path reaches with the parameter unset is
  ZK0725.
- The argument follows the rules of a [changed parameter](#changed-parameters): a place
  (ZK0410), of the kind of the parameter (ZK0401), in no other argument of the call
  (ZK0433). The value goes back to the place when the call returns.
- A [function kind](function-values.md#the-kind) takes `ဒေတာ ပြောင်းလဲရန်` and not
  `ဒေတာ ထည့်ပေးရန်` (ZK0201).
- For a [C function](#c-functions), the mark is the author's promise that C writes
  the place: the compiler cannot see inside C, so it trusts the mark. It takes the kinds
  that `ဒေတာ ပြောင်းလဲရန်` takes, and each other kind is ZK0466.

```zaka-error ZK0725
လုပ်ဆောင်ချက် တစ်ဝက် သည်
တန် ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လဒ် ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ထည့်ပေးရန် လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ တန် သည် ၀ ထက် ကြီးလျှင် လဒ် ကို တန် / ၂ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0725]: "လဒ်" is marked "ဒေတာ ထည့်ပေးရန်", so the function must set it on each path, but a path reaches this return without setting it
```

## How values are passed

Each parameter is a new variable, and the call stores the argument in it, as a variable
set from another stores its value. What that store does depends on the kind:

| Kind of the parameter | What the call does | Can the function change the caller's value? |
| --- | --- | --- |
| number, true/false value, character, enumeration | copies the value | no |
| text, plain list, counted list, struct, result | copies the value; text and plain lists share their cells until the first write | no |
| shared object | shares the object | it can: a change to its fields is seen through each name |
| function value | shares the function | no; a closure's captures are shared with each holder |
| pointer, in marked code | copies the address | through the address; with `အပိုင်အနေဖြင့်`, the caller's place holds the null address after the call |
| a parameter marked `ဒေတာ ပြောင်းလဲရန်` | copies in, and copies the value out at the return | it can: the value goes back to the caller's place |
| a parameter marked `ဒေတာ ထည့်ပေးရန်` | starts unset, and copies the value out at the return | it does: the function sets the caller's place on each path |

A duplicate is what the call means; the compiler skips the work where nothing can tell. Text
and a plain list keep their cells in a counted block, so a duplicate shares the block and adds
to its count. The first write to a shared block, to a cell or to the whole value, copies
the cells into a block of its own (the VM op `own`), so the caller's value never changes.
Text or a plain list that the caller reads again after the call is lent to the parameter
for the length of the call, with no count at all, and a value the caller reads no more
moves in. See [Values and objects](objects.md).

```zaka
အကောင့် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု၊ လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

// a change to the list remains in the function
လုပ်ဆောင်ချက် နှစ်ဆ သည် ကိန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းများ ၏ အကွက် ၀ ကို ၉၉ ဟု ထားမည်။
ကိန်းများ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// a change to the object is seen through each name
လုပ်ဆောင်ချက် အပ်ငွေ သည် အ ဟူသော အကောင့် တစ်ခု ကို လည်းကောင်း ပမာဏ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အ ၏ လက်ကျန် ကို (အ ၏ လက်ကျန်) + ပမာဏ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စ သည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် နှစ်ဆ ကို စ တို့ဖြင့် လုပ်ဆောင်မည်။
စ ကို ပြသမည်။
ကိန်းရှင် က သည် အကောင့်(အမည်: "မမ"၊ လက်ကျန်: ၁၀၀) ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် အပ်ငွေ ကို က၊ ၅၀ တို့ဖြင့် လုပ်ဆောင်မည်။
က ၏ လက်ကျန် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၉၉၊ ၂၊ ၃]
[၁၊ ၂၊ ၃]
၁၅၀
```

A function that must change the caller's value takes it as a
[changed parameter](#changed-parameters), or takes a shared object. Pointers, the heap,
and the owned marker `အပိုင်အနေဖြင့်` on parameters and results belong to marked code; their
rules are in [Unsafe Zaka](unsafe.md).

## The start block

Each program starts in its start block, a block of its own after the word `ပရိုဂရမ်`
("program"). Its header has the parts of a function header, with no name. The statements
that run go there or into functions: the top level of a file holds declarations alone.

```text
ပရိုဂရမ်သည်
<parameters> လက်ခံပြီး                                    (left out when there are none)
အောက်ပါတို့ကို လုပ်ဆောင်မည်။                                (exit code 0)
  or
အောက်ပါတို့ကို လုပ်ဆောင်ကာ <integer kind> တစ်ခု ကို ပြန်ပေးမည်။   (with an exit code)
<statements>
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

| Part | Rule |
| --- | --- |
| end | `ပရိုဂရမ်` with one of the four spellings of the done word: `ပြီးဆုံးပါပြီး`, `ပြီးဆုံးပါပြီ`, `ပြီးပါပြီး` or `ပြီးပါပြီ`; the closer of other blocks, `လုပ်ဆောင်စရာများ ...`, does not close it (ZK0201) |
| parameters | none, one or two, each a `စာသား စာရင်း`, with names the program chooses |
| the first | the name of the program, then the arguments of the run, one text for each |
| the second | the environment variables, one `NAME=VALUE` text for each |
| result | none, or an integer kind: the exit code of the run; the clauses of a [failing function](error-handling.md) can be used too |

```zaka
ကိန်းရှင် နှုတ်ခွန်း သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မင်္ဂလာပါ" ဖြစ်ပါစေ။

ပရိုဂရမ်သည်
ညွှန်ကြားချက်များ ဟူသော စာသား စာရင်း တစ်ခု ကို လည်းကောင်း ပတ်ဝန်းကျင် ဟူသော စာသား စာရင်း တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
နှုတ်ခွန်း ကို ပြသမည်။
ညွှန်ကြားချက်များ ၏ အကွက်အရေအတွက် ကို ပြသမည်။
၀ ကို ပြန်ပေးမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ
၁
```

The first cell is the name of the program, as in C, Rust and Python, so the count is at
least 1. On the command line it is the input as written, or `-` for standard input; the
arguments come after `--`, and the environment is the one `zakac` runs in. The Run
buttons of these pages give the name `main.zaka` and the Explorer `explorer.zaka`, with
no arguments and a blank environment:

```console
$ zakac run greet.zaka -- ကက ခခ
မင်္ဂလာပါ
၃
$ echo $?
0
```

### How a program runs

1. The top-level variables and constants get their starting values, in order, those of
   imported files first.
2. The start block runs, with the arguments and the environment.
3. The text that the top-level variables hold is given back, as at the end of each
   program.

The result of the start block is the exit code. The host keeps its lowest 8 bits, so
`-၁` ends the run with 255. A start block with no result, or one that ends with no
return, ends the run with 0. A run that stops on a trap ends with 1 whatever the
start block holds.

Inside, the start block follows the rules of a function: it has its own variables, and a
return statement ends it. It can change its two lists as it can each list
parameter. The start block itself cannot hold the mark `စိတ်မချရသော`: pointer work goes in
a block marked `စိတ်မချရသော်လည်း` inside it, or in a marked function
([Unsafe Zaka](unsafe.md)).

### Where a start block stands

- A program has one start block and no more, in the file given to `zakac run`: a second one
  is ZK0325, and a file with none is ZK0327. An imported file holds none (ZK0323).
- The top level of a file holds declarations alone: functions, structs, variables,
  constants and imports. A statement that runs there is ZK0326; it goes inside the start
  block or a function.
- A parameter that is no `စာသား စာရင်း`, a parameter the function can change, and a
  third parameter are ZK0440. A result that is no integer is ZK0441, a result kind among
  them: the start block cannot fail, so it holds no pass-on (ZK0445); see
  [Errors and results](error-handling.md#the-start-block).
- No call names the start block, it is not declared inside a block (ZK0214), and
  `အများသုံး` does not export it (ZK0229).

```zaka-error ZK0326
"ဟိုင်း" ဟု ပြသမည်။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0326]: a statement that runs goes inside the start block, not at the top level
```

```zaka-error ZK0325
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0325]: the program has a start block before this one
```

```zaka-error ZK0327
ကိန်းရှင် အမည် သည် "ဇကား" ဖြစ်ပါစေ။
```

```text
error[ZK0327]: the program has no start block, "ပရိုဂရမ်သည်"
```

```zaka-error ZK0440
ပရိုဂရမ်သည်
အရေအတွက် ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0440]: a parameter of the start block is a "စာသား စာရင်း", and there are two of them at most
```

```zaka-error ZK0441
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0441]: the result of the start block is "text", but it must be an integer
```

## Errors at a glance

| Code | Mistake |
| --- | --- |
| ZK0214 | an external function, a C function or the start block declared inside a block or a function |
| ZK0237 | a header that marks some parameters and not the others |
| ZK0241 | a mark string that is not one word a mark can be |
| ZK0303 | a call to a function that is not declared |
| ZK0304 | two functions with one name |
| ZK0305 | one parameter name listed twice |
| ZK0311 | a return statement outside a function |
| ZK0312 | a value returned from a function without a result |
| ZK0313 | no value returned from a function with a result |
| ZK0325 | a second start block |
| ZK0326 | a statement that runs, at the top level of a file |
| ZK0327 | a program with no start block |
| ZK0314 | a wrong number of arguments |
| ZK0315 | the result of a function that has none, used as a value |
| ZK0401 | an argument or a returned value that does not fit its kind |
| ZK0306 | a constant as the argument of a changed parameter |
| ZK0410 | an argument of a changed parameter that is no place |
| ZK0433 | a place a call changes, in another argument of the same call |
| ZK0440 | a start block parameter that is no `စာသား စာရင်း`, or a third one |
| ZK0441 | a start block result that is no integer |
| ZK0337 | a C function whose name is no C name |
| ZK0338 | two declarations of one C function with other kinds |
| ZK0465 | a parameter or a result of a C function with a kind C cannot take |
| ZK0466 | `ဒေတာ ပြောင်းလဲရန်` or `ဒေတာ ထည့်ပေးရန်` on a parameter of a C function that is no list of numbers and no struct that crosses |
| ZK0467 | `အပိုင်အနေဖြင့်` on a result that is no text from a C function |
| ZK0468 | a call of a marked function, a C one included, outside marked code |
| ZK0470 | `အပိုင်အနေဖြင့်` on a parameter or a result that holds no pointer |
| ZK0472 | a C function with a pointer in its kinds, with no mark `စိတ်မချရသော` |
| ZK0473 | a value of a kind from outside the program that the program makes, duplicates or prints |
| ZK0474 | a write to a field of a kind from outside the program, outside marked code or of a text or a handle field |
| ZK0722 | an argument of a changed parameter that can be unset, or a filled parameter read before the function sets it |
| ZK0725 | a return that a path reaches with a filled parameter unset |
| ZK0443 to ZK0449 | the failing-function mistakes of [Errors and results](error-handling.md#errors) |

The text of each message is in [Error messages](../reference/errors.md).
