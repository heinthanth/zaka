---
title: Function values
description: Function kinds, a function as a value, calls through a function value, a function that can be missing, comparing and printing function values, nested functions that capture, closures, and the errors of each.
---

# Function values

A function is a value too. A *function kind* names what a function takes and gives back,
`<name> ဟူသော လုပ်ဆောင်ချက်` ("the function called …") makes a value of that kind, and a
call through the value runs the function it holds. A function value can sit in a variable, a parameter, a field, a
list cell or a result, so a function can take another function as an argument, and a
list can hold functions to pick from. The Learn book starts with
[Functions as values](../../learn/function-values/README.md).

```zaka
လုပ်ဆောင်ချက် နှစ်ဆ သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// each cell, changed with the function the caller gives
လုပ်ဆောင်ချက် ကွက်တိုင်း သည်
စ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လည်းကောင်း
ပြောင်း ဟူသော (ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို လည်းကောင်း
လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် စာရင်း တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [] ဖြစ်ပါစေ။
စ ၏ အကွက်တိုင်း ကို တန် အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အဖြေ ထဲသို့ ပြောင်း(တန်) ကို ထည့်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကွက်တိုင်း ဟူသော လုပ်ဆောင်ချက်ကို [၁၊ ၂၊ ၃] နှင့် နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၂၊ ၄၊ ၆]
```

## The kind

A function kind repeats the header of a function, with `သော` ("that") in front of
`လုပ်ဆောင်ချက် အမျိုးအစား` ("function kind"), which closes each form:

| Parameters | Result | Written |
| --- | --- | --- |
| some | a result | `<parameters> လက်ခံပြီး <kind> တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား` |
| some | none | `<parameters> လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား` |
| none | a result | `<kind> တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား` |
| none | none | `လုပ်ဆောင်ချက် အမျိုးအစား` |

- The parameters are written as in a [header](functions.md#parameters), with no names:
  each is `<kind> တစ်<word> ကို`, then its markers, and with two or more each ends in
  `လည်းကောင်း`: `(<kind> တစ်<word> ကို [ဒေတာ ပြောင်းလဲရန်] [အပိုင်အနေဖြင့်] [ဝိဘတ် "<preposition>" ကို သုံး၍] [လည်းကောင်း] … လက်ခံပြီး <kind> တစ်<word> ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား)`.
- Each parameter and the result count their kind with its measure word, as in a header:
  `ကိန်းပြည့် တစ်ခု`, `စာလုံး တစ်လုံး`, a struct with the classifier it declares. A missing
  word is ZK0234; another word is the warning ZK0509, wherever the function kind stands:
  a variable, a parameter, a field, the result of another function kind, or inside an
  optional.
- A [preposition](functions.md#prepositions) stands after `ကို`:
  `(စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား)`; each
  parameter has a preposition, or none has (ZK0237).
- `ဒေတာ ပြောင်းလဲရန်` after `ကို` lets the function change the argument, as in a
  [header](functions.md#changed-parameters). A function kind takes that marker alone:
  `ဒေတာ ထည့်ပေးရန်` in one is ZK0243. A function whose parameter is
  [filled](functions.md#filled-parameters) fits a kind with `ဒေတာ ပြောင်းလဲရန်` there, and a
  call through the value sets its argument first, as for each changed parameter.
- A result can be a [result kind](error-handling.md#result-kinds), as in a header. A
  function kind can be the good side of a result kind, in parentheses:
  `(<function kind>) သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု`.
- A function kind that is itself the result of a header is in parentheses too, and the
  header counts it with `တစ်ခု`: `(<function kind>) တစ်ခု ကို ပြန်ပေးမည်`.
- Where the function kind stands as a whole, it counts with `ခု`: `<function kind> တစ်ခု`,
  and `လုပ်ဆောင်ချက် အမျိုးအစား တစ်ခု` for the kind with neither parameters nor result.

Messages write a function kind as `fn(<parameters>) -> <result>`, each parameter as its
short name with `mut ` in front of a changed one and its preposition after it:
`fn(isize, isize) -> bool`, `fn(text(ကို), text(သို့))`, `fn()`. Inside another short
name, such as a list of function values, it stands in parentheses: `(fn(isize))[]`. See
[Short names for kinds](../reference/errors.md#short-names-for-kinds).

### Inside a list of parameters or fields

A function kind with parameters inside a list of parameters or fields stands in
parentheses, since its parameters would run into the items before it:
`ရှေ့ ဟူသော (ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု`.
The same holds for a parameter or the result of another function kind. A function kind
with no parameters needs none. Without the parentheses the compiler reports ZK0239:

```zaka-error ZK0239
လုပ်ဆောင်ချက် ခေါ် သည်
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

## A function as a value

`<name> ဟူသော လုပ်ဆောင်ချက်` is a value of the function kind of the function it names,
a function of the file or an imported one. With one function of the name, the value
gives its kind on its own, so a declaration needs no written kind:

```zaka
လုပ်ဆောင်ချက် နှစ်ထပ် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
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

With [several functions of one name](functions.md#one-name-several-functions), the kind
the value goes to picks one: a declaration with a written kind, a parameter, a field, a
cell, the target of an assignment. No kind to go to, no function that fits it, or two
that fit it, is ZK0463:

```zaka
လုပ်ဆောင်ချက် ပြ သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ကိန်း {က}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပြ သည် က ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"စာ {က}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် စာသား တစ်ခု ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ပြ ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။
ဖ ဟူသော လုပ်ဆောင်ချက်ကို "မင်္ဂလာပါ" ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
စာ မင်္ဂလာပါ
```

```zaka-error ZK0463
လုပ်ဆောင်ချက် ပြ သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပြ သည် က ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် ပြ ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0463]: no single function "ပြ" fits here
```

A function that comes from outside, `ပြင်ပမှလာသော`, cannot be a value (ZK0464): the VM
serves it, and it has no place among the functions of the program. A function of the
program that calls it can stand in for it.

```zaka-error ZK0464
"စံကား/အဝင်အထွက်" မှ စာကြောင်းတစ်ကြောင်းဖတ် ကို ထည့်သွင်းမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် စာကြောင်းတစ်ကြောင်းဖတ် ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0464]: "စာကြောင်းတစ်ကြောင်းဖတ်" comes from outside the program, so it is no function value
```

An unsafe function (`စိတ်မချရသော`) cannot be a value either (ZK0469): a call through a value
is a plain call, which safe code can make, while each call of an unsafe function needs
[unsafe code](unsafe.md). A safe function that calls it inside an unsafe block can
stand in for it.

```zaka-error ZK0469
စိတ်မချရသော လုပ်ဆောင်ချက် ငါး သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
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

Each [call form](functions.md#calls) calls a function value too. The name the call writes
finds a declared function first; with none, it finds a variable, a parameter or a
constant of a function kind, and the call runs the function the value holds:

| Form | Through a value `ဖ` |
| --- | --- |
| statement | `ဖ ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်မည်။` |
| value | `ဖ ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်သော ရလဒ်` |
| sign form | `ဖ(၁)` |
| arguments first | `၁ ကို သုံး၍ ဖ သည့် ရလဒ်`, `၁ ကို သုံး၍ ဖ မည်။` |
| marked | `<argument> <preposition> … ဖ မည်။`, when the kind has prepositions |

The arguments check against the function kind as against a header: their count
(ZK0314), the kind of each (ZK0401), and the prepositions and `ဒေတာ ပြောင်းလဲရန်` markers. With `ရလဒ်ကောင်း`, `?`
or the pass-on tail, a call through a value of a kind whose result can fail passes the
failure on, as a direct call does. The call works left to right as it is written: the
function value is read first, then the arguments run, so an argument that changes the
variable the function comes from does not change which function runs:

```zaka
လုပ်ဆောင်ချက် နှစ်ဆ သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
"နှစ်ဆ ကို ခေါ်သည်" ဟု ပြသမည်။
က * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် နှစ်ထပ် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
"နှစ်ထပ် ကို ခေါ်သည်" ဟု ပြသမည်။
က * က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ဖ သည် နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။

// sets ဖ to another function, then gives back ၅
လုပ်ဆောင်ချက် ပြောင်း သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
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

A name that finds both a declared function and a value of a function kind in reach is
ZK0335, so a reader never has to guess which one runs. A name that finds a value of
another kind is ZK0462:

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

The long forms, those that write `ဟူသော လုပ်ဆောင်ချက်ကို` after the function, also take a
place in its stead: a field, a cell, or in unsafe code a read through a pointer, in
parentheses, as in `(ဂရစ် ၏ အကွက် ခု) ဟူသော လုပ်ဆောင်ချက်ကို …`. So a field or a cell is
called where it stands. The parentheses close the place: without them,
`ဂရစ် ၏ အကွက် ခု ဟူသော လုပ်ဆောင်ချက်` reads as the cell that the function `ခု` numbers. The short
forms take a name alone. The sign form
chains instead: a `(` right after a call calls the function value that call gives back,
so `ရွေး()(၅)` calls the function `ရွေး` gives back with `၅`, and `f(a)(b)(c)` goes on as
far as it is written.

```zaka
လုပ်ဆောင်ချက် နှစ်ဆ သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
"နှစ်ဆ ကို ခေါ်သည်" ဟု ပြသမည်။
က * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// ခေါ်ရမည့် လုပ်ဆောင်ချက်ကို ပြန်ပေးသည်
လုပ်ဆောင်ချက် ရွေး သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ (ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို ပြန်ပေးမည်။
"ရွေး ကို ခေါ်သည်" ဟု ပြသမည်။
နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ငါး သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
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

A `?` between the calls passes a failure on from each: here `ရွေး` can fail before it
gives the function, and the function it gives can fail too.

```zaka
လုပ်ဆောင်ချက် ခွဲ သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ က သည် ၀ နှင့် ညီလျှင် "သုည" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
၁၀၀ / က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ရွေး သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ (ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
ခွဲ ဟူသော လုပ်ဆောင်ချက် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် သုံး သည် န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
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

```zaka
လုပ်ငန်း ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ပေါင်း နှင့် မြှောက် တို့ပါဝင်သည်။

လုပ်ဆောင်ချက် ပေါင်းပေး သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က + ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် မြှောက်ပေး သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ခလုတ် ဟူသော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု နှင့် လုပ် ဟူသော (ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု တို့ပါဝင်သည်။

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

A function value holds a function at each point: no NULL stands in for one, and a call
through a function value has a function to run each time. A variable of a function kind declared with
no value is unset, and a call through it before each path gives it a function is ZK0722,
at each `--mem-check` level ([Set before read](statements.md#set-before-read)). A
function kind has no [initial value](kinds.md#initial-values): no function is a fair pick
for the program, so `ကနဦးတန်ဖိုး` for a function kind, or for a struct or a list that holds
one, is ZK0479.

```zaka-error ZK0722
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် ကိန်းပြည့် တစ်ခု ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား တစ်ခု ဖြစ်ပါစေ။
ဖ ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0722]: "ဖ" can be unset here: a path reaches this read with no value given to it
```

A function that is there at times and not at others is an [optional](optionals.md) of the
function kind, with the function kind in parentheses: `အလွတ်` when no function is there.
The program tests it, then reads the function out and calls it:

```zaka
လုပ်ဆောင်ချက် နှစ်ဆ သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် (ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အလွတ် ဖြစ်ပါစေ။
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

The VM still stops a call through a value that holds no function, with the trap
`the function value holds no function to call`, as a backstop for a case the check would
miss; a program the check passes does not reach it.

## Comparing and printing

`==` and `!=` compare two function values that one kind takes; two values are equal when both hold the same function, the same closure for a
[closure](#nested-functions-and-closures). The test `X နှင့် Y သည် တစ်ခုတည်း ဖြစ်လျှင်` asks the same. No other
operator takes a function value, and no conversion turns one into an integer or an
address.

A function value prints in the form that reads back, `<name> ဟူသော လုပ်ဆောင်ချက်`,
through `ပြသမည်`, a template slot, and inside a list or a struct:

```zaka
လုပ်ဆောင်ချက် တိုး သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က + ၁ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် တိုး ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။
ဖ ကို ပြသမည်။
"ဖ: {ဖ}" ဟု ပြသမည်။
ဖ == တိုး ဟူသော လုပ်ဆောင်ချက် ကို ပြသမည်။
ကိန်းရှင် ဂ သည် (ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အလွတ် ဖြစ်ပါစေ။
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

## Prepositions and changed parameters

A function kind compares its parameter kinds, its `ဒေတာ ပြောင်းလဲရန်` markers and its result as written,
with no widening. Prepositions are words of a call sentence, not part of what the function
does, so a kind with none takes a function with prepositions:

| The kind | Takes |
| --- | --- |
| no prepositions | a function with prepositions or without, when the rest matches |
| prepositions | a function with the same prepositions, in the same order, alone |

A call through a value uses the prepositions of its kind. Through a kind with prepositions, the call
reads as a sentence. A variable or a constant reads so in two cases:

| The declaration | Its prepositions |
| --- | --- |
| writes a kind | those of the kind; a kind with none gives none, whatever the starting value |
| writes no kind, and starts as a function with prepositions | those of the function, imported or not |

A parameter, a field and a result have the prepositions of their kinds alone. Below, the
parameter `ပို့သူ` and the variable `ပို့ရန်` take a sentence, and `ဖ`, whose kind has
none, takes its arguments in a list:

```zaka
လုပ်ဆောင်ချက် ရေး သည်
စာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လည်းကောင်း နေရာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "သို့" ကို သုံး၍ လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"{နေရာ}: {စာ}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပို့ သည်
ပို့သူ ဟူသော (စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လည်းကောင်း စာသား တစ်ခု ကို ဝိဘတ် "သို့" ကို သုံး၍ လည်းကောင်း လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ" ကို "မှတ်စု" သို့ ပို့သူ မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပို့ ဟူသော လုပ်ဆောင်ချက်ကို ရေး ဟူသော လုပ်ဆောင်ချက် ဖြင့် လုပ်ဆောင်မည်။
ကိန်းရှင် ပို့ရန် သည် ရေး ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။
"နောက်တစ်စောင်" ကို "စာပုံး" သို့ ပို့ရန် မည်။
ကိန်းရှင် ဖ သည် စာသား တစ်ခု ကို လည်းကောင်း စာသား တစ်ခု ကို လည်းကောင်း လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ရေး ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။
ဖ ဟူသော လုပ်ဆောင်ချက်ကို "ပြန်စာ" နှင့် "ဖိုင်" တို့ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှတ်စု: မင်္ဂလာပါ
စာပုံး: နောက်တစ်စောင်
ဖိုင်: ပြန်စာ
```

A sentence through `ဖ` is an error, though `ဖ` starts as `ရေး`: the kind written in the
declaration has no prepositions, and the variable can later hold a function with none,
as here. The sentence does not read as a call, so its operands wait for a verb (ZK0208),
as with a parameter of such a kind:

```zaka-error ZK0208
လုပ်ဆောင်ချက် ရေး သည်
စာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လည်းကောင်း နေရာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "သို့" ကို သုံး၍ လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"{နေရာ}: {စာ}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ရေးခြင်း သည်
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

Where the sentence still reads as a call through a value whose kind has no prepositions,
such as through a name in an inner block that shadows a name with prepositions, the
check reports it as ZK0332: no function of that name takes the arguments.


## How a function value is held

A function value is one word. For a function of the file it is, in the VM, the number of
the function plus one, so no function value is 0; in native code, the address of a record whose
first word is the code address. Such a value is known when the program is compiled, so it
can be the value of a constant, a cell of a constant list, and the starting value of a
variable or a constant in an imported file: a module can export a table of functions.

A [closure](#nested-functions-and-closures) is one word too: the address of a counted
record on the heap that holds its captures. Function values are shared, as
[shared objects](objects.md) are, and never copied:

- `ဖြစ်ပါစေ`, `ဟု ထားမည်`, an argument, a field, a cell and a result all hold the same
  function: a closure reached through two names counts on, whichever name calls it.
- `ကူးထည့်မည်`, `အကုန်ကူးထည့်မည်` and the copies `X ၏ မိတ္တူ` and `X တစ်ခုလုံး ၏ မိတ္တူ`
  share a function value too, since duplicating the captures would split state the program
  shares on purpose.
- The record is freed when its last holder goes. A closure that captures the variable
  that holds it makes a ring, which the ring collector frees.

A call through a function value names no function, but such a value comes from a function
named with `ဟူသော` alone, a closure included. So the [flow checker](../internals/borrow-checker.md)
treats the call as a call of each function the program takes as a value and whose kind
fits, and joins what each of them does.

## Nested functions and closures

A function with statements can be declared inside the statements of a function, the start
block, a block, a loop or a conditional. Such a *nested function* is visible from its
declaration to the end of the block that holds it, and inside itself, so it can call
itself. Two nested functions of one name in one scope are ZK0304, and nested functions
have no overloads. A nested function hides a top-level function of its name. External
functions, C functions and the start block remain at the top level (ZK0214).

### Captures

A nested function uses the local variables and the parameters of the functions around it
through their names, as a JavaScript or Python function does. Each one it uses is a *capture*. Nothing
lists the captures: the compiler finds them in the statements. A capture is the outer
variable itself, shared between the function and the block around it, so a write on
either side is seen on the other:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စုစုပေါင်း သည် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် မှတ်တမ်း သည် "" ဖြစ်ပါစေ။
// uses စုစုပေါင်း and မှတ်တမ်း of the start block
လုပ်ဆောင်ချက် ထည့် သည် န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စုစုပေါင်း ကို စုစုပေါင်း + န ဟု ထားမည်။
မှတ်တမ်း ထဲသို့ "+{န}" ကို ထည့်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ထည့် ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်မည်။
စုစုပေါင်း ကို ၁၀ ဟု ထားမည်။
ထည့် ဟူသော လုပ်ဆောင်ချက်ကို ၂ ဖြင့် လုပ်ဆောင်မည်။
"{စုစုပေါင်း} {မှတ်တမ်း}" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၂ +၁+၂
```

- Globals and top-level functions are no captures: each function reaches them.
- A function nested in a nested function reaches the locals of each function around it.
  The function between the two captures them too, since it makes the inner one.
- A nested function that calls another nested function with captures, or names it with
  `ဟူသော`, captures that closure as it captures a variable.
- A nested function with no captures is a plain function value.
- A capture that is a `ဒေတာ ပြောင်းလဲရန်` parameter of the function around it shares that
  parameter, and what the parameter holds when that function returns goes back to its
  caller, as in [Changed parameters](functions.md#changed-parameters).

### Closures

`<name> ဟူသော လုပ်ဆောင်ချက်` of a nested function with captures is a *closure*: a value of
the same function kind, so each function kind takes closures, and no new spelling is
needed. The captures never show in the kind. A call of the function through its name goes
through the one closure made where the function is declared. It prints as
`<name> ဟူသော လုပ်ဆောင်ချက်`, as each function value does.

A closure keeps its captures alive, so it goes where each value goes: it can be given
back, stored in a field, a cell or a global, and called after the block that declares it
ends. Each holder shares the one closure and its captures:

```zaka
လုပ်ဆောင်ချက် ရေတွက်စက် သည် စ ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ (လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် ကြိမ် သည် စ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် တိုး သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကြိမ် ကို ကြိမ် + ၁ ဟု ထားမည်။
ကြိမ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
// ကြိမ် outlives the call, inside the closure
တိုး ဟူသော လုပ်ဆောင်ချက် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် နှစ်ကြိမ် သည် ဖ ဟူသော (လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဖ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
ဖ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရေတွက် သည် ရေတွက်စက် ဟူသော လုပ်ဆောင်ချက်ကို ၁၀ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် အခြား သည် ရေတွက် ဖြစ်ပါစေ။
ရေတွက် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
// the same counter, through another name and a parameter
နှစ်ကြိမ် ဟူသော လုပ်ဆောင်ချက်ကို အခြား ဖြင့် လုပ်ဆောင်မည်။
ကိန်းရှင် နောက် သည် ရေတွက်စက် ဟူသော လုပ်ဆောင်ချက်ကို ၁၀ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
// a second call makes a second counter
နောက် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
ရေတွက် == အခြား ကို ပြသမည်။
ရေတွက် == နောက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၁
၁၂
၁၃
၁၁
မှန်
မမှန်
```

A closure in a field or a global, and a closure nested two levels deep:

```zaka
ခလုတ် ဟူသော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု နှင့် နှိပ် ဟူသော (လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု တို့ပါဝင်သည်။

လုပ်ဆောင်ချက် ဘာမျှမလုပ် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// a global that keeps a function value, with a function from the start
ကိန်းရှင် သိမ်း သည် (လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဘာမျှမလုပ် ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။

လုပ်ဆောင်ချက် ခလုတ်သစ် သည် အမည် ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ခလုတ် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် ကြိမ် သည် ၀ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် နှိပ် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
// nested in နှိပ်: it reaches ကြိမ် and အမည် through နှိပ်
လုပ်ဆောင်ချက် ပြ သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"{အမည်}: {ကြိမ်}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ကြိမ် ကို ကြိမ် + ၁ ဟု ထားမည်။
ပြ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
သိမ်း ကို နှိပ် ဟူသော လုပ်ဆောင်ချက် ဟု ထားမည်။
ခလုတ်(အမည်: အမည်၊ နှိပ်: နှိပ် ဟူသော လုပ်ဆောင်ချက်) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ခလုတ်သစ် ဟူသော လုပ်ဆောင်ချက်ကို "အိုကေ" ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
(ခ ၏ နှိပ်) ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
သိမ်း ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
(ခ ၏ နှိပ်) ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
ခ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အိုကေ: ၁
အိုကေ: ၂
အိုကေ: ၃
ခလုတ်(အမည်: "အိုကေ"၊ နှိပ်: နှိပ် ဟူသော လုပ်ဆောင်ချက်)
```

`==` and the test `X နှင့် Y သည် တစ်ခုတည်း ဖြစ်လျှင်` both ask whether two function values
hold one function: the same function of the file, or the same closure. Two calls of
`ရေတွက်စက်` above make two closures, so the two are not equal, though their code is one.

### A fresh variable each round

A local declared inside the block of a loop is a new variable in each round, as a `let` of
JavaScript is, so closures made in a loop each keep their own round. The counter of a
counted loop and the cell variable of an each-loop are new in each round too. The
variable of a generic loop is one variable for the whole loop, since its update carries
it from round to round, and so is each variable declared before the loop:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပြများ သည် (လုပ်ဆောင်ချက် အမျိုးအစား) စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [] ဖြစ်ပါစေ။
// a counted loop: န is new in each round
၁ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ပြ သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ရေတွက် {န}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပြများ ထဲသို့ ပြ ဟူသော လုပ်ဆောင်ချက် ကို ထည့်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
// a generic loop: မ is one variable for each round
ကိန်းရှင် မ သည် ၁ ဖြစ်ကာ မ <= ၂ နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဆ သည် မ * ၁၀ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ပြ၂ သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မ {မ}၊ ဆ {ဆ}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပြများ ထဲသို့ ပြ၂ ဟူသော လုပ်ဆောင်ချက် ကို ထည့်မည်။
ပြီးလျှင် မ ကို မ + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပြများ ၏ အကွက်တိုင်း ကို ဖ အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဖ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရေတွက် ၁
ရေတွက် ၂
ရေတွက် ၃
မ ၃၊ ဆ ၁၀
မ ၃၊ ဆ ၂၀
```

### Closures and C

A closure, a nested function with captures or a variable that holds one, goes to the
parameter of a function kind of a [C function](functions.md#c-functions) as a function of
the program does. C gets a pointer it can call, and a call from C runs the closure.

**Lent for the call.** A closure given to a C call is lent to that call: C can call it
while the call runs, and the call needs nothing more.

```text
C call runs  ->  C calls the closure  ->  fine
C call ends  ->  C calls the closure  ->  the trap `closure escape`
```

**A leave.** A C function that keeps the function, as `signal` keeps a handler, calls it
after the call that gave it ended. The closure needs *leave* for that, which two
statements give and end:

```text
<function value> ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်။
<function value> ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြန်လည်ရုပ်သိမ်းမည်။
```

The first gives C leave to call the closure after the call that gave it, and the leave
holds a count of the record of the closure until it ends. The second takes the leave back.
A leave still open when the program ends ends with it, as C can call nothing more, so
`--leaks` lists nothing for it.
In the program below, `signal` keeps a handler and `raise` calls it. (The sample
`samples/ffi/closures.zaka` adds a `qsort` call, which needs no leave.)

```zaka
လုပ်ဆောင်ချက် signal သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
sig ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း handler ဟူသော (၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို လည်းကောင်း လက်ခံမည်။
လုပ်ဆောင်ချက် raise သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ sig ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖမ်းကြိမ် သည် ၀ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ဖမ်း သည် s ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဖမ်းကြိမ် ကို ဖမ်းကြိမ် + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ဖမ်း ဟူသော လုပ်ဆောင်ချက်ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်။
signal ဟူသော လုပ်ဆောင်ချက်ကို ၁၅ နှင့် ဖမ်း ဟူသော လုပ်ဆောင်ချက် တို့ဖြင့် လုပ်ဆောင်မည်။
raise ဟူသော လုပ်ဆောင်ချက်ကို ၁၅ ဖြင့် လုပ်ဆောင်မည်။
ဖမ်းကြိမ် ကို ပြသမည်။
ဖမ်း ဟူသော လုပ်ဆောင်ချက်ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြန်လည်ရုပ်သိမ်းမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁
```

The rules of the two statements:

- A function value of each form stands before `ကို`: the name form `X ဟူသော လုပ်ဆောင်ချက်`, a
  variable, a field or a cell. A value of another kind is ZK0462.
- `ပြင်ပမှ` names code outside the program, as `ပြင်ပမှလာသော` does. The words
  `ခေါ်ဆိုခွင့်`, `ပြုမည်` and `ပြန်လည်ရုပ်သိမ်းမည်` are reserved, and the fused spelling
  `ခေါ်ဆိုခွင့်ပြုမည်` is the same as the spaced one. Each statement stands as the inline
  block of an `if` too.
- A second leave of one closure, and a revoke of a closure with no leave, do nothing. A
  function with no captures needs no leave, so the statement does nothing for it.
- Two values with the same function and the same captures are one closure, so one leave
  covers both. The closures of different rounds of a loop are different closures: each
  round gives its own leave.

```zaka-error ZK0462
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဂ သည် ၅ ဖြစ်ပါစေ။
ဂ ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0462]: this value is isize, not a function value, so C cannot get leave to call it
```

**A late call.** A call from C to a closure with no C call running and no leave stops the
program with the trap `closure_escape` (see [When a program stops](../vm/traps.md)),
and its message tells how to give leave. The native code of a closure remains in place after its
last use, so a late call traps and never runs freed code. A call from C on another thread
ends the run with a message, since the program runs on one thread.

**Where it runs.** `zakac run` makes one native closure for each closure given to C,
found again from the octets of its record. A native build writes an extra file, `zaka_cb.c`,
for a program that makes a closure and gives function values to C or holds a leave
statement, and links `-lffi`; see [Native builds](../compiler/native.md). A C declaration
takes function values of numbers, booleans and addresses alone (ZK0465), so each closure
that reaches C has a native closure. A leave of a function value whose kind takes or gives
a struct, text or a function value, which no C function can get, is a construct a native
build cannot write.

### How it runs

A captured local lives in a counted *box*, a block of one cell on the heap, and each read
and write of it, inside the nested function and in the block around it, goes through the
box. A local that no closure captures remains in its frame. A local of a loop block gets a
new box in each round, and a captured counter of a counted loop, or the cell variable of
an each-loop, is copied into a new box in each round.

The function of a nested function takes its declared parameters, then its own closure,
then each capture, passed with a count: a box, or the closure of another nested
function. Where the function is declared, the
program fills a counted record of a code word, an info word and one field for each box,
and a hidden variable of the block holds it. A call through a closure runs a *thunk*,
which takes the record last and calls the function with the boxes of its fields. A call of
the function inside itself passes its own closure and boxes on. See
[call_value](../vm/instructions.md) and
[Function values in C](../compiler/native.md).

## Errors at a glance

| Code | Problem |
| --- | --- |
| ZK0239 | a function kind with parameters inside a list of parameters or fields, not in parentheses |
| ZK0335 | a call that names both a declared function and a value of a function kind |
| ZK0401 | a function value of another kind for a variable or a parameter |
| ZK0462 | a call through a name or a place whose kind is no function kind; a leave for C of a value that is no function value |
| ZK0463 | a function value of an overloaded name where no single function fits |
| ZK0464 | a function value of a function that comes from outside |
| ZK0469 | a function value of an unsafe function (`စိတ်မချရသော`) |
| ZK0479 | `ကနဦးတန်ဖိုး` for a function kind, or for a kind that holds one |
| ZK0722 | a call through a variable that a path reaches with no function given |
