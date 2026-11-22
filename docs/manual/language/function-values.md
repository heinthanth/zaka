---
title: Function values
description: Function kinds, a function as a value, calls through a function value, a function that can be missing, comparing and printing function values, and the errors of each.
---

# Function values

In Zaka a function is also a value. A *function kind* (function type) describes what a
function accepts and returns, we write `<name> ဟူသော လုပ်ဆောင်ချက်` ("the function called
`<name>`") to make a *function value* (first-class function, closure) of that function kind, and a
call through the value runs the function in it. We can keep a function value in a
variable, a parameter, a field, a list cell or a result, so a function can accept another
function as an argument, and a list can contain functions to pick from. The Learn book
starts with [Functions as values](../../learn/function-values/README.md).

This page covers function kinds and the values of functions declared at the top level.
Nested functions, and the closures that capture their variables, are on
[Closures](closures.md).

```zaka
နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// each cell, changed with the function the caller gives
ကွက်တိုင်း ဟူသော လုပ်ဆောင်ချက် သည်
စ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လည်းကောင်း
ပြောင်း ဟူသော (ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် စာရင်း တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [] ဖြစ်ပါစေ။
စ ၏ အကွက်တိုင်း ကို တန် အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အဖြေ ထဲသို့ (ပြောင်း ဟူသော လုပ်ဆောင်ချက်ကို တန် ဖြင့် လုပ်ဆောင်သော ရလဒ်) ကို ထည့်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကွက်တိုင်း ဟူသော လုပ်ဆောင်ချက်ကို [၁၊ ၂၊
၃] နှင့် နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၂၊ ၄၊ ၆]
```

## Function kinds

```text
<parameters> လက်ခံပြီး <kind> တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား
```

A function kind (function type) repeats the function signature of a function: we write the
clauses that follow `သည်` in a function signature, in the same order, with no block
opener, and we end the last clause with `သော` ("that") in front of
`လုပ်ဆောင်ချက် အမျိုးအစား` ("function kind"). `သည့်` is the same word as `သော` here, so
`လက်ခံသည့်`, `ပြန်ပေးသည့်` and `သုံးသည့်` mean the same as `လက်ခံသော`, `ပြန်ပေးသော` and
`သုံးသော`. A function kind has one of four forms:

| Parameters | Result | Written |
| --- | --- | --- |
| some | a result | `<parameters> လက်ခံပြီး <kind> တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား` |
| some | none | `<parameters> လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား` |
| none | a result | `<kind> တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား` |
| none | none | `လုပ်ဆောင်ချက် အမျိုးအစား` |

Where we use a function kind as a whole, we count it with `ခု`, as in
`<function kind> တစ်ခု`, and `လုပ်ဆောင်ချက် အမျိုးအစား တစ်ခု` for the function kind with
no parameters and no result.

### Parameters and the result

```text
<kind> တစ်<word> ကို [ဒေတာ ပြောင်းလဲရန် | ဒေတာ ထည့်ပေးရန်] [အပိုင်အနေဖြင့်] [ဝိဘတ် "<preposition>" ကို သုံး၍] [လည်းကောင်း]
```

We write each parameter as in a [function signature](functions-parameters.md#writing-a-parameter), but with no
name, and when there are two or more parameters, each one ends with `လည်းကောင်း`. The
rules of a function signature hold for the parts of the parameter:

- **Measure words.** We count the data kind (type) of each parameter and of the result with its
  measure word (classifier), as in a function signature: `ကိန်းပြည့် တစ်ခု`,
  `စာလုံး တစ်လုံး`, or a struct with the measure word that it declares. A missing word is
  ZK0234, and a different word is the warning ZK0509, wherever the function kind is: in a
  variable, a parameter, a field, the result of another function kind, or inside an
  optional.
- **Prepositions.** A [preposition](functions-sentences.md#prepositions) comes after `ကို`, as in
  `(စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား)`, and each
  parameter has its own preposition or none.
- **Mutable parameters.** `ဒေတာ ပြောင်းလဲရန်` after `ကို` lets the function change the
  argument, as a [mutable parameter](functions-parameters.md#mutable-parameters) (`inout`) does in a
  function signature, and the argument must be set before a call through the value.
- **Writable parameters.** `ဒေတာ ထည့်ပေးရန်` makes a [writable
  parameter](functions-parameters.md#writable-parameters) (`out`), as in a function signature: a call
  through the value needs no value for the argument, and the argument is set after the
  call. A function with a writable parameter fits a function kind with either marker in
  that place. A function with a mutable parameter does not fit a function kind with
  `ဒေတာ ထည့်ပေးရန်` there, because it can read the argument before it sets it (ZK0401).
- **Result kinds.** The result can be a [result kind](result-kinds.md), as in a function
  signature. A function kind can also be the good side of a result kind, in parentheses:
  `(<function kind>) သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု`.
- **A function kind as a result.** When a function returns a function kind, we put the
  function kind in parentheses and count it with `တစ်ခု`:
  `(<function kind>) တစ်ခု ကို ပြန်ပေးမည်`.

### Phrases

A function kind can also have the [starting and ending
phrases](functions-sentences.md#starting-and-ending-phrases) of a function signature. We write them
first, the same as a function signature writes them after `သည်`: the list of phrases,
then `သုံး၍` before the parameters or the result, or `သုံးသော` right before
`လုပ်ဆောင်ချက် အမျိုးအစား` when the function kind has neither parameters nor a result:

```text
(အစ စကားစု "ဖိုင်" ကို သုံး၍ စာသား တစ်ခု ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား)
(အစ စကားစု "ဖန်သားပြင်ကို" ကို သုံးသော လုပ်ဆောင်ချက် အမျိုးအစား)
```

### Inside a list of parameters or fields

When a function kind with parameters is inside a list of parameters or fields, we put it in
parentheses, because its parameters would otherwise run into the items before it:
`ရှေ့ ဟူသော (ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု`.
This rule also applies to a parameter or the result of another function kind. A function
kind with no parameters needs no parentheses. Without the parentheses, the compiler
reports ZK0239:

```zaka-error ZK0239
ခေါ် ဟူသော လုပ်ဆောင်ချက် သည်
ဖ ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဖ(၁) ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0239]: a function kind with parameters in a list of parameters or fields stands in parentheses
```

### In messages

Messages write a function kind as `fn(<parameters>) -> <result>`. Each parameter appears as
the short name of its data kind, with `mut ` in front of a mutable parameter, `out ` in
front of a writable parameter, and its preposition after it. The phrases go in braces after
`fn`, with `…` for the arguments:

```text
fn(isize, isize) -> bool
fn(text(ကို), text(သို့))
fn()
fn(out isize)
fn{ဖိုင် … စာကို}(text(ထဲမှ)) -> text
```

Inside another short name, such as a list of function values, the function kind goes in
parentheses: `(fn(isize))[]`. See [Short names for
kinds](../reference/errors.md#short-names-for-data-kinds).

## A function as a value

```text
<name> ဟူသော လုပ်ဆောင်ချက်
```

We write `<name> ဟူသော လုပ်ဆောင်ချက်` to get a value of the function kind of the function
`<name>`, which can be a function of the file or an imported one. When there is one
function of that name, the value has a known function kind, so a declaration needs no
written data kind:

```zaka
နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။
ဖ(၇) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၉
```

When [several functions share one name](functions-overloads.md), the
place where the value goes picks one through its function kind: a declaration with a
written data kind, a parameter, a field, a cell, or the target of an assignment. When there
is no such place, no function that fits it, or two functions that fit it, the compiler
reports ZK0463:

```zaka
ပြ ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ကိန်း {က}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပြ ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"စာ {က}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် စာသား တစ်ခု ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား တစ်ခု ဖြစ်၍ တန်ဖိုးသည်
ပြ ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။
ဖ ဟူသော လုပ်ဆောင်ချက်ကို "မင်္ဂလာပါ" ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
စာ မင်္ဂလာပါ
```

```zaka-error ZK0463
ပြ ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပြ ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် ပြ ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0463]: no single function "ပြ" fits here
```

### Functions that cannot be values

A function that comes from outside the program, declared with `ပြင်ပမှလာသော`, cannot be a
value (ZK0464), because the VM provides it and it is not one of the functions of the
program. We can use a function of the program that calls it in its place.

```zaka-error ZK0464
"စံကား/အဝင်အထွက်" မှ စာကြောင်းတစ်ကြောင်းဖတ် ကို ထည့်သွင်းမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် စာကြောင်းတစ်ကြောင်းဖတ် ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0464]: "စာကြောင်းတစ်ကြောင်းဖတ်" comes from outside the program, so it is no function value
```

An unsafe function (`စိတ်မချရသော`) cannot be a value either (ZK0469). A call through a
value is a plain call, which safe code can make, while each call of an unsafe function
needs [unsafe code](unsafe.md). We can use a safe function that calls it inside an unsafe
block in its place.

```zaka-error ZK0469
ငါး ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
၅ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် ငါး ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0469]: "ငါး" is an unsafe function, declared with "စိတ်မချရသော", so it cannot be a function value
```

## Calling a function value

We can call a function value with each of the [call forms](functions-calls.md). The name in
the call first looks for a declared function, and when there is none, it looks for a
variable, a parameter or a constant of a function kind, and the call runs the function in
that value:

| Form | Through a value `ဖ` |
| --- | --- |
| statement | `ဖ ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်မည်။` |
| value | `ဖ ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်သော ရလဒ်` |
| sign form | `ဖ(၁)` |
| arguments first | `၁ ကို သုံး၍ ဖ သည့် ရလဒ်`, `၁ ကို သုံး၍ ဖ မည်။` |
| with prepositions | `<argument> <preposition> … ဖ မည်။`, when the function kind has prepositions |

The compiler checks the arguments against the function kind, as it checks them against a
function signature: their count (ZK0314), the data kind of each (ZK0401), their
prepositions and their `ဒေတာ ပြောင်းလဲရန်` markers. When the result of the function kind
can fail, a call through the value with `ရလဒ်ကောင်း`, `?` or the pass-on call statement
[passes the failure on](error-handling.md#passing-a-failure-on), as a direct call does.

The call runs from left to right as it is written: the program reads the function value
first and then works out the arguments, so an argument that changes the variable of the
function does not change which function runs:

```zaka
နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
"နှစ်ဆ ကို ခေါ်သည်" ဟု ပြသမည်။
က * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
"နှစ်ထပ် ကို ခေါ်သည်" ဟု ပြသမည်။
က * က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ဖ သည် နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။

// sets ဖ to another function, then returns ၅
ပြောင်း ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
"ပြောင်း ကို ခေါ်သည်" ဟု ပြသမည်။
ဖ ကို နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက် ဟု ထားမည်။
၅ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဖ(ပြောင်း()) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ပြောင်း ကို ခေါ်သည်
နှစ်ဆ ကို ခေါ်သည်
၁၀
```

When a name finds both a declared function and a value of a function kind in scope, the
compiler reports ZK0335, so that no one has to guess which one runs. When a name finds a
value of a data kind that is not a function kind, it reports ZK0462:

```zaka-error ZK0462
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဂ သည် ၅ ဖြစ်ပါစေ။
ဂ ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0462]: "ဂ" is isize, not a function value, so it cannot be called
```

### A field or a cell

```text
(<place>) ဟူသော လုပ်ဆောင်ချက်ကို <arguments> တို့ဖြင့် လုပ်ဆောင်မည်။
<call>(<arguments>)
```

The long forms, which write `ဟူသော လုပ်ဆောင်ချက်ကို` after the function, also accept a
place in parentheses in place of a name: a field, a cell, or, in unsafe code, a read
through a pointer, as in `(ဂရစ် ၏ အကွက် ခု) ဟူသော လုပ်ဆောင်ချက်ကို …`. So we can call a
field or a cell where it is. The parentheses mark the end of the place: without them,
`ဂရစ် ၏ အကွက် ခု ဟူသော လုပ်ဆောင်ချက်` means the cell whose number is the function `ခု`.
The short forms accept a name alone.

The sign form chains instead: a `(` right after a call calls the function value that the
call returns. So `ရွေး()(၅)` calls the function that `ရွေး` returns with `၅`, and
`f(a)(b)(c)` goes on as far as it is written.

```zaka
နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
"နှစ်ဆ ကို ခေါ်သည်" ဟု ပြသမည်။
က * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// ခေါ်ရမည့် လုပ်ဆောင်ချက်ကို ပြန်ပေးသည်
ရွေး ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
(ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို ပြန်ပေးမည်။
"ရွေး ကို ခေါ်သည်" ဟု ပြသမည်။
နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ငါး ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
"ငါး ကို ခေါ်သည်" ဟု ပြသမည်။
၅ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ရွေး()(ငါး()) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရွေး ကို ခေါ်သည်
ငါး ကို ခေါ်သည်
နှစ်ဆ ကို ခေါ်သည်
၁၀
```

A `?` between the calls passes a failure on from each one. Here `ရွေး` can fail before it
returns the function, and the function it returns can fail too:

```zaka
ခွဲ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ က သည် ၀ နှင့် ညီလျှင် "သုည" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
၁၀၀ / က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ရွေး ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
(ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ကိန်းပြည့်
သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးသော
လုပ်ဆောင်ချက် အမျိုးအစား)
သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
ခွဲ ဟူသော လုပ်ဆောင်ချက် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

သုံး ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
ရွေး()?(န)? ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
သုံး ဟူသော လုပ်ဆောင်ချက်ကို ၄ ဖြင့် လုပ်ဆောင်သော ရလဒ် ထဲမှ ဒေတာ ကို ပြသမည်။
ကိန်းရှင် မ သည် သုံး ဟူသော လုပ်ဆောင်ချက်ကို ၀ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
မ ထဲမှ အမှားအယွင်း ဒေတာ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၅
သုည
```

This program calls a function value from a cell of a list and from a field of a struct:

```zaka
လုပ်ငန်း ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ပေါင်း နှင့် မြှောက် တို့ပါဝင်သည်။

ပေါင်းပေး ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က + ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

မြှောက်ပေး ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ခလုတ် ဟူသော ဒေတာအစုအဝေးတွင်
အမည် ဟူသော စာသား တစ်ခု နှင့်
လုပ် ဟူသော (ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု
တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
// one cell for each member of the enumeration
ကိန်းရှင် ဂရစ် သည် [ပေါင်းပေး ဟူသော လုပ်ဆောင်ချက်၊ မြှောက်ပေး ဟူသော လုပ်ဆောင်ချက်] ဖြစ်ပါစေ။
ကိန်းရှင် ခု သည် လုပ်ငန်းများထဲမှ မြှောက် ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
(ဂရစ် ၏ အကွက် ခု) ဟူသော လုပ်ဆောင်ချက်ကို ၆ နှင့် ၇ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ကိန်းရှင် ခ သည် ခလုတ်(အမည်: "ပေါင်း"၊ လုပ်: ပေါင်းပေး ဟူသော လုပ်ဆောင်ချက်) ဖြစ်ပါစေ။
(ခ ၏ လုပ်) ဟူသော လုပ်ဆောင်ချက်ကို ၆ နှင့် ၇ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၂
၁၃
```

## A function that can be missing

A function value has a function at each point: no NULL can replace the function, so a call
through a function value has a function to run each time. A variable of a function kind
declared with no value is unset, and a call through it before each path gives it a function
is ZK0722, at each level of the `mem` lint ([Set before
read](declarations.md#set-before-read)). A function kind has no [initial
value](kinds.md#initial-values) (default value), because no function is a fair pick for the
program, so `ကနဦးတန်ဖိုး` for a function kind, or for a struct or a list that contains one,
is ZK0479.

```zaka-error ZK0722
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် ကိန်းပြည့် တစ်ခု ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား တစ်ခု ဖြစ်ပါစေ။
ဖ ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0722]: "ဖ" can be unset here: a path reaches this read with no value given to it
```

For a function that is there at times and missing at others, we use an
[optional](optionals.md) (option type) of the function kind, with the function kind in
parentheses, and `အလွတ်` when no function is there. The program tests the optional, then
reads the function out and calls it:

```zaka
နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် (ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍
တန်ဖိုးသည် အလွတ် ဖြစ်ပါစေ။
ဖ ကို ပြသမည်။
ဖ ကို နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် ဟု ထားမည်။
အကယ်၍ ဖ ထဲတွင် ဒေတာ ရှိလျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဂ သည် ဖ ထဲမှ ဒေတာ ဖြစ်ပါစေ။
ဂ(၂၁) ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ဖ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အလွတ်
၄၂
ဒေတာ(နှစ်ဆ ဟူသော လုပ်ဆောင်ချက်)
```

As a backstop for a case that the check might miss, the VM still stops a call through a
value that contains no function, with the trap `the function value holds no function to
call`. A program that passes the check does not reach it.

## Comparing and printing

`==` and `!=` compare two function values of the same function kind. Two values are equal
when both contain the same function, or the same closure for a
[closure](closures.md#closures-as-values), and the test
`X နှင့် Y သည် တစ်ခုတည်း ဖြစ်လျှင်` asks the same question. No other operator works on a
function value, and no conversion turns one into an integer or an address.

A function value prints in the form that we write it in, `<name> ဟူသော လုပ်ဆောင်ချက်`,
through `ပြသမည်`, in a template slot, and inside a list or a struct:

```zaka
တိုး ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က + ၁ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် တိုး ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။
ဖ ကို ပြသမည်။
"ဖ: {ဖ}" ဟု ပြသမည်။
ဖ == တိုး ဟူသော လုပ်ဆောင်ချက် ကို ပြသမည်။
ကိန်းရှင် ဂ သည် (ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍
တန်ဖိုးသည် အလွတ် ဖြစ်ပါစေ။
[ဖ] ကို ပြသမည်။
ဂ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
တိုး ဟူသော လုပ်ဆောင်ချက်
ဖ: တိုး ဟူသော လုပ်ဆောင်ချက်
မှန်
[တိုး ဟူသော လုပ်ဆောင်ချက်]
အလွတ်
```

## Prepositions, phrases and mutable parameters

The compiler compares the parameter data kinds, the `ဒေတာ ပြောင်းလဲရန်` markers and the
result of a function kind as written, with no widening. Prepositions and phrases
are words of the call sentence, not part of what the function does, so a function kind with
none of them accepts a function that has them:

| The function kind has | It accepts |
| --- | --- |
| no prepositions and no phrases | a function with or without prepositions and phrases, when the rest matches |
| prepositions or phrases | a function with the same prepositions and phrases, in the same order, and no other function |

A call through a value uses the prepositions and the phrases of its function kind, just as a
call of a function uses those of its function signature. So when the function kind has
prepositions, we call the value with a sentence. A variable or a constant gets prepositions
in two cases:

| The declaration | Its prepositions |
| --- | --- |
| writes a function kind | those of the function kind; a function kind with none gives none, whatever the starting value is |
| writes no data kind, and starts as a function with prepositions or phrases | those of the function, imported or not |

A parameter, a field and a result have the prepositions of their function kinds and no
others. Below, we call the parameter `ပို့သူ` and the variable `ပို့ရန်` with a sentence,
while `ဖ`, whose function kind has no prepositions, gets its arguments in a list:

```zaka
ရေး ဟူသော လုပ်ဆောင်ချက် သည်
စာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လည်းကောင်း
နေရာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "သို့" ကို သုံး၍ လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"{နေရာ}: {စာ}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပို့ ဟူသော လုပ်ဆောင်ချက် သည်
ပို့သူ ဟူသော (စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍
လည်းကောင်း စာသား တစ်ခု ကို ဝိဘတ် "သို့" ကို သုံး၍
လည်းကောင်း လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ" ကို "မှတ်စု" သို့ ပို့သူ မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပို့ ဟူသော လုပ်ဆောင်ချက်ကို ရေး ဟူသော လုပ်ဆောင်ချက် ဖြင့် လုပ်ဆောင်မည်။
ကိန်းရှင် ပို့ရန် သည် ရေး ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။
"နောက်တစ်စောင်" ကို "စာပုံး" သို့ ပို့ရန် မည်။
ကိန်းရှင် ဖ သည်
စာသား တစ်ခု ကို လည်းကောင်း စာသား တစ်ခု ကို လည်းကောင်း လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား တစ်ခု ဖြစ်၍
တန်ဖိုးသည် ရေး ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။
ဖ ဟူသော လုပ်ဆောင်ချက်ကို "ပြန်စာ" နှင့် "ဖိုင်" တို့ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှတ်စု: မင်္ဂလာပါ
စာပုံး: နောက်တစ်စောင်
ဖိုင်: ပြန်စာ
```

A sentence through `ဖ` is an error, even though `ဖ` starts as `ရေး`: the function kind in
the declaration has no prepositions, and the variable can later contain a function with
none, as it does here. The compiler does not treat the sentence as a call, so its operands
wait for a verb (ZK0208), as with a parameter of such a function kind:

```zaka-error ZK0208
ရေး ဟူသော လုပ်ဆောင်ချက် သည်
စာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လည်းကောင်း နေရာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "သို့" ကို သုံး၍ လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"{နေရာ}: {စာ}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ရေးခြင်း ဟူသော လုပ်ဆောင်ချက် သည်
စာ ဟူသော စာသား တစ်ခု ကို လည်းကောင်း နေရာ ဟူသော စာသား တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"{နေရာ}- {စာ}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် စာသား တစ်ခု ကို လည်းကောင်း စာသား တစ်ခု ကို လည်းကောင်း လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ရေး ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။
ဖ ကို ရေးခြင်း ဟူသော လုပ်ဆောင်ချက် ဟု ထားမည်။
"ပြန်စာ" ကို "ဖိုင်" သို့ ဖ မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0208]: operands wait for a verb: "ပြန်စာ" (ကို), "ဖိုင်" (no particle)
```

When the compiler still treats the sentence as a call through a value whose function kind
has no prepositions, such as through a name in an inner block that shadows a name with
prepositions, it reports ZK0332: no function of that name accepts the arguments.

A function kind with phrases lets a call through a value use the words of a command. Below,
the parameter `လုပ်` accepts a function with no parameters, and we call it with its
starting phrase:

```zaka
ရှင်း ဟူသော လုပ်ဆောင်ချက် သည် အစ စကားစု "ဖန်သားပြင်ကို" ကို သုံး၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ရှင်းပြီ" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ခေါ် ဟူသော လုပ်ဆောင်ချက် သည်
လုပ် ဟူသော (အစ စကားစု "ဖန်သားပြင်ကို" ကို သုံးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဖန်သားပြင်ကို လုပ် မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ခေါ် ဟူသော လုပ်ဆောင်ချက်ကို ရှင်း ဟူသော လုပ်ဆောင်ချက် ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရှင်းပြီ
```

## How a function value is stored

A function value is one word. For a function of the file, it is the number of the function
plus one in the VM, so no function value is 0, and in native code it is the address of a
record whose first word is the code address. The compiler knows such a value when it
compiles the program, so the value can be the value of a constant, a cell of a constant
list, and the starting value of a variable or a constant in an imported file. So a module
can export a table of functions.

A [closure](closures.md) is one word too: the address of a counted record on the heap that
contains its captures. Function values are shared, as [shared objects](objects.md)
(reference types) are, and never duplicated:

- `ဖြစ်ပါစေ`, `ဟု ထားမည်`, an argument, a field, a cell and a result all refer to the same
  function, so when a closure keeps a counter, calls through either of two names add
  to the same counter.
- `ကူးထည့်မည်`, `အကုန်ကူးထည့်မည်` and the duplicates `X ၏ မိတ္တူ` and
  `X တစ်ခုလုံး ၏ မိတ္တူ` share a function value too, because duplicating the captures would
  split state that the program shares on purpose.
- The record is freed when its last holder goes. A closure that captures the variable that
  contains it makes a ring, which the ring collector frees.

A call through a function value names no function, but each such value comes from a
function named with `ဟူသော`, closures included. So the [flow
checker](../internals/borrow-checker.md) (borrow checker) treats the call as a call of each
function that the program uses as a value and whose function kind fits, and joins what
each of those functions does.

## Errors at a glance

| Code | Problem |
| --- | --- |
| ZK0208 | a sentence call through a value whose function kind has no prepositions |
| ZK0234 | a parameter or result of a function kind with no measure word |
| ZK0239 | a function kind with parameters inside a list of parameters or fields, not in parentheses |
| ZK0314 | a call through a value with the wrong count of arguments |
| ZK0332 | a sentence call through a shadowing value whose function kind has no prepositions |
| ZK0335 | a call that names both a declared function and a value of a function kind |
| ZK0401 | a function value of another function kind for a variable or a parameter |
| ZK0462 | a call through a name or a place whose data kind is no function kind; a leave for C of a value that is no function value ([Closures and C](closures.md#closures-and-c)) |
| ZK0463 | a function value of an overloaded name where no single function fits |
| ZK0464 | a function value of a function that comes from outside |
| ZK0469 | a function value of an unsafe function (`စိတ်မချရသော`) |
| ZK0479 | `ကနဦးတန်ဖိုး` for a function kind, or for a data kind that contains one |
| ZK0509 | a different measure word on a parameter or result of a function kind (a warning) |
| ZK0722 | a call through a variable that a path reaches with no function given |

The errors of nested functions and closures are on [Closures](closures.md#errors).
