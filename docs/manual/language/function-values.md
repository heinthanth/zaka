---
title: Function values
description: Function kinds, the address of a function, calls through a function value, ဗလာ, comparing and printing function values, nested functions and closures, and the errors of each.
---

# Function values

A function is a value too. A *function kind* names what a function takes and gives back,
the address of a function makes a value of that kind, and a call through the value runs
the function it holds. A function value can sit in a variable, a parameter, a field, a
list cell or a result, so a function can take another function as an argument, and a
list can hold functions to pick from. The Learn book starts with
[Functions as values](../../learn/pointers/functions.md).

```zaka
လုပ်ဆောင်ချက် နှစ်ဆ သည် ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က * ၂ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// each cell, changed with the function the caller gives
လုပ်ဆောင်ချက် ကွက်တိုင်း သည်
ကိန်းပြည့် စာရင်း စ၊
(ကိန်းပြည့် ကို လက်ခံပြီး ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) ပြောင်း
တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် စာရင်း ရလဒ် ကို ပြန်ပေးမည်။
စ ၏ အကွက်တိုင်း ကို တန် အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ရလဒ် ထဲသို့ ပြောင်း(တန်) ကို ထည့်ပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ကွက်တိုင်း ကို [၁၊ ၂၊ ၃]၊ လုပ်ဆောင်ချက် နှစ်ဆ ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
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
| some | a result | `<parameters> လက်ခံပြီး <kind> ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား` |
| some | none | `<parameters> လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား` |
| none | a result | `<kind> ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား` |
| none | none | `လုပ်ဆောင်ချက် အမျိုးအစား` |

- The parameters are written as in a [header](functions.md#declaring-a-function): kinds
  joined with `၊`, the last two with `နှင့်` when wanted, closed with `ကို` after one and
  `တို့ကို` after two or more.
- A name after a parameter kind is optional: `ကိန်းပြည့်၊ ကိန်းပြည့် တို့ကို` and
  `ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို` name one kind. A name documents the kind and takes
  no part in a match.
- A [mark](functions.md#marked-parameters) stands after a name, as in a header:
  `စာသား စာ(ကို)`. Each parameter has a mark, or none has (ZK0237).
- `လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သော` before a parameter lets the function change the
  argument, as in a [header](functions.md#edit-references).
- A result can be a [result kind](error-handling.md#result-kinds): it writes no `ရလဒ်` of
  its own, as in a header. A function kind can be the good side of a result kind, in
  parentheses: `(<function kind>) ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း`.
- A declaration counts the kind with `ခု`: `<function kind> တစ်ခု`.

Messages write a function kind as `fn(<parameters>) -> <result>`, each parameter as its
short name with `mut ` in front of an edited one and its mark after it:
`fn(isize, isize) -> bool`, `fn(text(ကို), text(သို့))`, `fn()`. Inside another short
name, such as a list of function values, it stands in parentheses: `(fn(isize))[]`. See
[Short names for kinds](../reference/errors.md#short-names-for-kinds).

### Inside a list of parameters or fields

A function kind with parameters inside a list of parameters or fields stands in
parentheses, since its parameters would run into the items before it:
`(ကိန်းပြည့်၊ ကိန်းပြည့် တို့ကို လက်ခံပြီး မှန်မှားတန်ဖိုး ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) ရှေ့`.
The same holds for a parameter or the result of another function kind. A function kind
with no parameters needs none. Without the parentheses the compiler reports ZK0239:

```zaka-error ZK0239
လုပ်ဆောင်ချက် ခေါ် သည်
ကိန်းပြည့် ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား ဖ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဖ(၁) ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0239]: a function kind with parameters in a list of parameters or fields stands in parentheses
```

## The address of a function

`လုပ်ဆောင်ချက် <name> ၏ လိပ်စာ` is a value of the function kind of the function it names,
a function of the file or an imported one. With one function of the name, the value
gives its kind on its own, so a declaration needs no written kind:

```zaka
လုပ်ဆောင်ချက် နှစ်ထပ် သည် ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က * က ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် လုပ်ဆောင်ချက် နှစ်ထပ် ၏ လိပ်စာ ဖြစ်ပါစေ။
ဖ(၇) ကို ပြသပါ။
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
လုပ်ဆောင်ချက် ပြ သည် ကိန်းပြည့် က ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ကိန်း {က}" ဟု ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပြ သည် စာသား က ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"စာ {က}" ဟု ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် စာသား ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် ပြ ၏ လိပ်စာ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ဖ ကို "မင်္ဂလာပါ" တို့ဖြင့် လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
စာ မင်္ဂလာပါ
```

```zaka-error ZK0463
လုပ်ဆောင်ချက် ပြ သည် ကိန်းပြည့် က ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပြ သည် စာသား က ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် လုပ်ဆောင်ချက် ပြ ၏ လိပ်စာ ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0463]: no single function "ပြ" fits here
```

A function that comes from outside, `ပြင်ပမှလာသော`, has no address (ZK0464): the VM
serves it, and it has no place among the functions of the program. A function of the
program that calls it can stand in for it.

```zaka-error ZK0464
"စံကား/အဝင်အထွက်" မှ စာကြောင်းတစ်ကြောင်းဖတ် ကို ထည့်သွင်းပါ။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် လုပ်ဆောင်ချက် စာကြောင်းတစ်ကြောင်းဖတ် ၏ လိပ်စာ ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0464]: "စာကြောင်းတစ်ကြောင်းဖတ်" comes from outside the program, so it has no address
```

## Calling a function value

Each [call form](functions.md#calls) calls a function value too. The name the call writes
finds a declared function first; with none, it finds a variable, a parameter or a
constant of a function kind, and the call runs the function the value holds:

| Form | Through a value `ဖ` |
| --- | --- |
| statement | `လုပ်ဆောင်ချက် ဖ ကို ၁ တို့ဖြင့် လုပ်ဆောင်ပါ။` |
| value | `လုပ်ဆောင်ချက် ဖ ကို ၁ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်` |
| sign form | `ဖ(၁)` |
| arguments first | `၁ ကို သုံး၍ ဖ သည့် ရလဒ်`, `၁ ကို သုံး၍ ဖ မည်။` |
| marked | `<argument> <mark> … ဖ မည်။`, when the kind has marks |

The arguments check against the function kind as against a header: their count
(ZK0314), the kind of each (ZK0401), and the marks and edit marks. With `ရလဒ်ကောင်း`, `?`
or the pass-on tail, a call through a value of a kind whose result can fail passes the
failure on, as a direct call does. The call works left to right as it is written: the
function value is read first, then the arguments run, so an argument that changes the
variable the function comes from does not change which function runs:

```zaka
လုပ်ဆောင်ချက် နှစ်ဆ သည် ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
"နှစ်ဆ ကို ခေါ်သည်" ဟု ပြသပါ။
က * ၂ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် နှစ်ထပ် သည် ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
"နှစ်ထပ် ကို ခေါ်သည်" ဟု ပြသပါ။
က * က ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ဖ သည် လုပ်ဆောင်ချက် နှစ်ဆ ၏ လိပ်စာ ဖြစ်ပါစေ။

// sets ဖ to another function, then gives back ၅
လုပ်ဆောင်ချက် ပြောင်း သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
"ပြောင်း ကို ခေါ်သည်" ဟု ပြသပါ။
ဖ ကို လုပ်ဆောင်ချက် နှစ်ထပ် ၏ လိပ်စာ ဟု ထားပါ။
၅ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဖ(ပြောင်း()) ကို ပြသပါ။
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
လုပ်ဆောင်ချက် ဂ ကို ၁ တို့ဖြင့် လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0462]: "ဂ" is isize, not a function value, so it cannot be called
```

### A field or a cell

The long forms, those that write `လုပ်ဆောင်ချက်` before the function, also take a place
in its stead: a field, a cell or a read through a pointer, up to the `ကို`. So a field
or a cell is called where it stands. The short forms take a name alone. The sign form
chains instead: a `(` right after a call calls the function value that call gives back,
so `ရွေး()(၅)` calls the function `ရွေး` gives back with `၅`, and `f(a)(b)(c)` goes on as
far as it is written.

```zaka
လုပ်ဆောင်ချက် နှစ်ဆ သည် ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
"နှစ်ဆ ကို ခေါ်သည်" ဟု ပြသပါ။
က * ၂ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// ခေါ်ရမည့် လုပ်ဆောင်ချက်ကို ပြန်ပေးသည်
လုပ်ဆောင်ချက် ရွေး သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ (ကိန်းပြည့် ကို လက်ခံပြီး ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) ရလဒ် ကို ပြန်ပေးမည်။
"ရွေး ကို ခေါ်သည်" ဟု ပြသပါ။
လုပ်ဆောင်ချက် နှစ်ဆ ၏ လိပ်စာ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ငါး သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
"ငါး ကို ခေါ်သည်" ဟု ပြသပါ။
၅ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ရွေး()(ငါး()) ကို ပြသပါ။
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
လုပ်ဆောင်ချက် ခွဲ သည် ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
အကယ်၍ က သည် ၀ နှင့် ညီလျှင် "သုည" ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
၁၀၀ / က ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ရွေး သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ (ကိန်းပြည့် ကို လက်ခံပြီး ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် ခွဲ ၏ လိပ်စာ ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် သုံး သည် ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
ရွေး()?(န)? ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် သုံး ကို ၄ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ရလဒ်ကောင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
ကိန်းရှင် မ သည် လုပ်ဆောင်ချက် သုံး ကို ၀ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
မ ကို အမှားအယွင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၅
သုည
```

```zaka
ဒေတာအမျိုးအစား လုပ်ငန်း ဆိုသည်မှာ ပေါင်း၊ မြှောက် တို့ ဖြစ်သည်။

လုပ်ဆောင်ချက် ပေါင်းပေး သည် ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က + ခ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် မြှောက်ပေး သည် ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က * ခ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ဒေတာအမျိုးအစား ခလုတ် တွင် စာသား အမည်၊ (ကိန်းပြည့်၊ ကိန်းပြည့် တို့ကို လက်ခံပြီး ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) လုပ် တို့ ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
// one cell for each member of the enumeration
ကိန်းရှင် ဇယား သည် [လုပ်ဆောင်ချက် ပေါင်းပေး ၏ လိပ်စာ၊ လုပ်ဆောင်ချက် မြှောက်ပေး ၏ လိပ်စာ] ဖြစ်ပါစေ။
ကိန်းရှင် ခု သည် လုပ်ငန်းများထဲမှ မြှောက် ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ဇယား ၏ အကွက် ခု ကို ၆၊ ၇ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ကိန်းရှင် ခ သည် ခလုတ်(အမည်: "ပေါင်း"၊ လုပ်: လုပ်ဆောင်ချက် ပေါင်းပေး ၏ လိပ်စာ) ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ခ ၏ လုပ် ကို ၆၊ ၇ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၂
၁၃
```

## No function

A function value can hold no function: `ဗလာ`, as a pointer can. `ဗလာ` converts to each
function kind, and it is the default of one: a variable with no value, a cleared field
or a cleared cell. A call through `ဗလာ` has no function to run, so the compiler follows
function values as it follows [pointers](ownership.md#null-and-bounds):

- **ZK0709.** A call through a value that is `ဗလာ` on each path to the call.
- **ZK0711**, with `--mem-strict` alone. A call through a variable that a `ဗလာ` the
  function itself wrote can reach on some path: a variable set in one branch of a
  conditional alone, for example. As for a pointer, a value that comes from outside,
  such as a parameter or a cell, is not reported; the VM stops a call through `ဗလာ`
  there.

```zaka-error ZK0709
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် ကိန်းပြည့် ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား တစ်ခု ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ဖ ကို ၁ တို့ဖြင့် လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0709]: "ဖ" is ဗလာ here, so it holds no function to call
```

```zaka-warning ZK0711 mem-strict=on
လုပ်ဆောင်ချက် နှစ်ဆ သည် ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က * ၂ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် ကိန်းပြည့် ကို လက်ခံပြီး ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ရွေး သည် မှန် ဖြစ်ပါစေ။
အကယ်၍ ရွေး ဖြစ်လျှင် ဖ ကို လုပ်ဆောင်ချက် နှစ်ဆ ၏ လိပ်စာ ဟု ထားပါ။
ဖ(၁) ကို ပြသပါ။
အကယ်၍ ဖ သည် ဗလာ နှင့် မညီလျှင် ဖ(၂) ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0711]: "ဖ" can be ဗလာ here; compare it with ဗလာ before the call
```

A comparison with `ဗလာ` narrows each branch, as for a pointer, so the second call above
gives no warning. A field or a cell is not followed: a call through one that holds
`ဗလာ` stops the program in the VM:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဇယား သည် အကွက် ၂ ခုရှိသော (ကိန်းပြည့် ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား) စာရင်း တစ်ခု ဖြစ်ပါစေ။
"အစ" ဟု ပြသပါ။
လုပ်ဆောင်ချက် ဇယား ၏ အကွက် ၀ ကို ၁ တို့ဖြင့် လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အစ
the program stopped: the function value is ဗလာ, so it holds no function to call
```

## Comparing and printing

`==` and `!=` compare two function values that one kind takes, or a function value with
`ဗလာ`; two values are equal when both hold the same function. No other operator takes a
function value, and no conversion turns one into an integer or an address.

A function value prints in the form that reads back, `လုပ်ဆောင်ချက် <name> ၏ လိပ်စာ`, or
`ဗလာ`, through `ပြသပါ`, a template slot, and inside a list or a struct:

```zaka
လုပ်ဆောင်ချက် တိုး သည် ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က + ၁ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် လုပ်ဆောင်ချက် တိုး ၏ လိပ်စာ ဖြစ်ပါစေ။
ဖ ကို ပြသပါ။
"ဖ: {ဖ}" ဟု ပြသပါ။
ဖ == လုပ်ဆောင်ချက် တိုး ၏ လိပ်စာ ကို ပြသပါ။
ဖ == ဗလာ ကို ပြသပါ။
ကိန်းရှင် မရှိ သည် ကိန်းပြည့် ကို လက်ခံပြီး ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား တစ်ခု ဖြစ်ပါစေ။
[ဖ၊ မရှိ] ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
လုပ်ဆောင်ချက် တိုး ၏ လိပ်စာ
ဖ: လုပ်ဆောင်ချက် တိုး ၏ လိပ်စာ
မှန်
မမှန်
[လုပ်ဆောင်ချက် တိုး ၏ လိပ်စာ၊ ဗလာ]
```

## Marks and edit marks

A function kind compares its parameter kinds, its edit marks and its result as written,
with no widening. Marks are words of a call sentence, not part of what the function
does, so a kind with none takes a function with marks:

| The kind | Takes |
| --- | --- |
| no marks | a function with marks or without, when the rest matches |
| marks | a function with the same marks, in the same order, alone |

A call through a value uses the marks of its kind. Through a kind with marks, the call
reads as a sentence. A variable reads so when its declaration writes a kind with marks,
or when it starts as the address of a function with marks:

```zaka
လုပ်ဆောင်ချက် ရေး သည်
စာသား စာ(ကို)၊ စာသား နေရာ(သို့) လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"{နေရာ}: {စာ}" ဟု ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပို့ သည်
(စာသား စာ(ကို)၊ စာသား နေရာ(သို့) လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား) ပို့သူ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ" ကို "မှတ်စု" သို့ ပို့သူ မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ပို့ ကို လုပ်ဆောင်ချက် ရေး ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်ပါ။
ကိန်းရှင် ပို့ရန် သည် လုပ်ဆောင်ချက် ရေး ၏ လိပ်စာ ဖြစ်ပါစေ။
"နောက်တစ်စောင်" ကို "စာပုံး" သို့ ပို့ရန် မည်။
ကိန်းရှင် ဖ သည် စာသား၊ စာသား တို့ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် ရေး ၏ လိပ်စာ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ဖ ကို "ပြန်စာ"၊ "ဖိုင်" တို့ဖြင့် လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှတ်စု: မင်္ဂလာပါ
စာပုံး: နောက်တစ်စောင်
ဖိုင်: ပြန်စာ
```

## Ownership

A function value is one word: in the VM the number of the function plus one, so 0 is
`ဗလာ`; in native code the address of a record whose first word is the code address. The
address of a function is known when the program is compiled, so it can be the value of a
constant, a cell of a constant list, and the starting value of a variable or a constant in
an imported file: a module can export a table of functions.

A [closure](#nested-functions-and-closures) is one word too: the address of a record on the
heap that holds its captures. A function value is a value as text is:

- `ဖြစ်ပါစေ`, `ဟု ထားပါ`, `ကူးထည့်မည်`, a field, a cell and a result make a duplicate, and
  the duplicate of a closure gets a record of its own, its captures duplicated;
- the last use of a variable moves the value instead, and `ပြောင်းထည့်မည်` moves it;
- an argument *lends* the value for the call: the callee runs the caller's closure, so a
  counter passed to a helper keeps counting, and the callee frees nothing of it;
- a variable frees the record of the closure it holds as its block ends.

A call through a function value names no function, but such a value comes from the
address of a function alone, a closure included. So the [borrow checker](../internals/borrow-checker.md)
treats the call as a call of each function whose address the program takes and whose
kind fits, and joins what each of them frees and keeps.

## Nested functions and closures

A function with statements can be declared inside the statements of a function, the start
block, a block, a loop or a conditional. Such a *nested function* is visible from its
declaration to the end of the block that holds it, and inside itself, so it can call
itself. Two nested functions of one name in one scope are ZK0304, and nested functions
have no overloads. A nested function hides a top-level function of its name. External
functions, C functions and the start block remain at the top level (ZK0214).

### The capture list

A nested function lists each local variable and each parameter of the functions around it
that its statements use, its *captures*, in the list of its parameters. With a capture in
the list, each item of the list ends in `ကိုလည်းကောင်း` ("and also"), the parameters too, and
`၊` can stand between the items. A parameter has a kind; a capture is a bare name, with the
kind of the outer local. A word before `လည်းကောင်း` gives the mode of a capture:

| Item | Mode | The function gets | The outer local after |
| --- | --- | --- | --- |
| `<name> ကိုလည်းကောင်း` | duplicate | a duplicate of its own | unchanged |
| `<name> ကို အပိုင်အနေဖြင့်လည်းကောင်း` | move | the value itself | blank: `""`, `[]` or `ဗလာ`; a number is copied |
| `<name> ကို ငှား၍လည်းကောင်း` | borrow | the outer local itself | shared with the function |

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည် သည် "စံ" ဖြစ်ပါစေ။
ကိန်းရှင် စုစုပေါင်း သည် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် မှတ်တမ်း သည် "" ဖြစ်ပါစေ။
// a parameter, then a duplicate, a borrow and a move
လုပ်ဆောင်ချက် နှုတ်ဆက် သည် စာသား နောက် ကိုလည်းကောင်း၊ အမည် ကိုလည်းကောင်း၊
စုစုပေါင်း ကို ငှား၍လည်းကောင်း၊ မှတ်တမ်း ကို အပိုင်အနေဖြင့်လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အမည် ထဲသို့ နောက် ကို ထည့်ပါ။
မှတ်တမ်း ထဲသို့ "+" ကို ထည့်ပါ။
စုစုပေါင်း ကို စုစုပေါင်း + ၁ ဟု ထားပါ။
"{အမည်} {မှတ်တမ်း} {စုစုပေါင်း}" ဟု ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် နှုတ်ဆက် ကို "က" တို့ဖြင့် လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် နှုတ်ဆက် ကို "ခ" တို့ဖြင့် လုပ်ဆောင်ပါ။
"အပြင်: {အမည်} [{မှတ်တမ်း}] {စုစုပေါင်း}" ဟု ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
စံက + ၁
စံကခ ++ ၂
အပြင်: စံ [] ၂
```

The duplicates and the moves happen once, where the function is declared, and what the
function writes to them remains for its next call: `အမည်` grows from one call to the next,
while the outer `အမည်` keeps `စံ`.

- Each local the statements use is listed: one the list leaves out is ZK0339, and a name
  that is no local of a function around it, such as a global, is ZK0340. Globals and
  top-level functions need no listing. A listed capture the statements never use is the
  warning ZK0341.
- A function nested in a nested function lists what it uses, and the function around it
  lists those in turn, since it makes the inner one.
- A nested function that calls a nested function with captures, or takes its address,
  lists it too, under its name: the closure is captured as each other value is, and so is
  the function it sits inside. Its own name inside itself needs no listing.
- A nested function with no captures is a plain function, with no list.

```zaka-error ZK0339
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် န သည် ၁ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ပြ သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် ပြ ကို လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0339]: "ပြ" uses "န" of the function around it, which its list leaves out
```

A borrow reaches the local itself, as in a Pascal nested function or a Rust closure that
borrows. A function nested two levels deep borrows through the one around it:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မှတ်တမ်း သည် "" ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် မှတ် သည် စာသား စာ ကိုလည်းကောင်း၊ မှတ်တမ်း ကို ငှား၍လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
// nested in မှတ်: it borrows မှတ်တမ်း through မှတ်
လုပ်ဆောင်ချက် ခြား သည် မှတ်တမ်း ကို ငှား၍လည်းကောင်း လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ (မှတ်တမ်း ၏ အကွက်အရေအတွက်) > ၀ ဖြစ်လျှင် မှတ်တမ်း ထဲသို့ "၊ " ကို ထည့်ပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် ခြား ကို လုပ်ဆောင်ပါ။
မှတ်တမ်း ထဲသို့ စာ ကို ထည့်ပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် မှတ် ကို "က" တို့ဖြင့် လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် မှတ် ကို "ခ" တို့ဖြင့် လုပ်ဆောင်ပါ။
မှတ်တမ်း ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
က၊ ခ
```

### Closures

The address of a nested function, `လုပ်ဆောင်ချက် <name> ၏ လိပ်စာ`, is a plain function value
when the function has no captures. With captures it is a *closure*: a value of the same
function kind, so each function kind takes closures, and no new spelling is needed. The
captures never show in the kind. A call of the function through its name, and its address, go
through that one closure, made where the function is declared. It prints as
`လုပ်ဆောင်ချက် <name> ၏ လိပ်စာ`, as a function value does.

A closure of duplicates and moves owns its captures, so it goes where each value goes: it
can be given back, stored in a field, a cell or a global, and kept after the block that
declares it ends. Inside its own statements, `လုပ်ဆောင်ချက် <name> ၏ လိပ်စာ` is that same
closure, with the rules of each other place: an argument lends it, so the callee runs it
and its captures change, while `=`, a store or a result makes a duplicate.

```zaka
လုပ်ဆောင်ချက် ရေတွက်စက် သည် ကိန်းပြည့် စ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ (လုပ်ဆောင်ချက် အမျိုးအစား) ရလဒ် ကို ပြန်ပေးမည်။
ကိန်းရှင် ကြိမ် သည် စ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် တိုး သည် ကြိမ် ကိုလည်းကောင်း လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကြိမ် ကို ကြိမ် + ၁ ဟု ထားပါ။
ကြိမ် ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် တိုး ၏ လိပ်စာ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် နှစ်ကြိမ် သည် (လုပ်ဆောင်ချက် အမျိုးအစား) ဖ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ဖ ကို လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် ဖ ကို လုပ်ဆောင်ပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရေတွက် သည် လုပ်ဆောင်ချက် ရေတွက်စက် ကို ၁၀ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ရေတွက် ကို လုပ်ဆောင်ပါ။
// the helper runs this closure: its count goes on
လုပ်ဆောင်ချက် နှစ်ကြိမ် ကို ရေတွက် တို့ဖြင့် လုပ်ဆောင်ပါ။
// a duplicate counts on its own
ကိန်းရှင် မိတ္တူ ထဲသို့ ရေတွက် ကို ကူးထည့်မည်။
လုပ်ဆောင်ချက် မိတ္တူ ကို လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် ရေတွက် ကို လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၁
၁၂
၁၃
၁၄
၁၄
```

An owner moved into a closure, `ပ ကို အပိုင်အနေဖြင့်လည်းကောင်း`, belongs to the record, which
frees its block as the closure ends, unless the function freed it first. Two records cannot
free one block, so such a closure moves and is lent but is never duplicated, as a Rust
closure that is not `Clone`: a duplicate the compiler sees is ZK0724, and one it cannot
see, such as a parameter put into a variable, stops the program with the trap
`closure_owner`.

```zaka-error ZK0724
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၄၂ ဟု ထားပါ။
လုပ်ဆောင်ချက် ပြ သည် ပ ကို အပိုင်အနေဖြင့်လည်းကောင်း လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပ ၏ တန်ဖိုး ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ကိန်းရှင် ပွား ထဲသို့ လုပ်ဆောင်ချက် ပြ ၏ လိပ်စာ ကို ကူးထည့်မည်။
လုပ်ဆောင်ချက် ပြ ကို လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် ပွား ကို လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0724]: the closure of "ပြ" owns a block, so it cannot be duplicated
```

Write `ပြောင်းထည့်မည်` in place of `ကူးထည့်မည်`, or use `ပြ` no more after the line, and the
closure moves.

A closure with a borrowed capture, or with a capture that holds a borrow, *borrows*: it
cannot go where it would outlive the locals it borrows. Moving into such a closure is
allowed. The compiler reports ZK0722 when the address of a borrowing function, written in
the place itself, is:

- given back as a result;
- put in a field of a struct, in a literal or an assignment;
- put in a cell of a list, in a literal or with an add or an insert;
- put in a global, or in a place reached through an address;
- set into a variable of an outer block.

```zaka-error ZK0722
လုပ်ဆောင်ချက် ဖန်တီး သည် ကိန်းပြည့် တိုး ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ (ကိန်းပြည့် ကို လက်ခံပြီး ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် ထည့် သည် ကိန်းပြည့် က ကိုလည်းကောင်း၊ တိုး ကို ငှား၍လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က + တိုး ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် ထည့် ၏ လိပ်စာ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0722]: "ထည့်" borrows the locals around it, so its address cannot go where it outlives them
```

C calls a bare code address, so no closure goes to a C function, borrowing or not: the
address of a nested function with captures as the argument of a C function is ZK0723.

Where the compiler cannot tell, as with a closure that came in as a parameter, the check
moves to run time. A function value that a program stores in a global, a field or a cell,
or gives back, is checked as it leaves: a borrowing closure stops the program with the
closure escape trap (see [When a program stops](../vm/traps.md#all-the-traps)). A function
value that goes to C stops it when it is a closure of each sort.

```zaka-stop
// a global that keeps a function value
ကိန်းရှင် သိမ်း သည် (ကိန်းပြည့် ကို လက်ခံပြီး ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ဖြစ်ပါစေ။

လုပ်ဆောင်ချက် မှတ်ထား သည်
(ကိန်းပြည့် ကို လက်ခံပြီး ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) ဖ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
သိမ်း ကို ဖ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် နှစ်ဆ သည် ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က * ၂ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် မှတ်ထား ကို လုပ်ဆောင်ချက် နှစ်ဆ ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်ပါ။
"ပုံမှန် လုပ်ဆောင်ချက် ရ" ဟု ပြသပါ။
ကိန်းရှင် ထပ် သည် ၁ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ထည့် သည် ကိန်းပြည့် က ကိုလည်းကောင်း၊ ထပ် ကို ငှား၍လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က + ထပ် ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် မှတ်ထား ကို လုပ်ဆောင်ချက် ထည့် ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်ပါ။
"မရောက်" ဟု ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ပုံမှန် လုပ်ဆောင်ချက် ရ
the program stopped: the function value borrows the locals around its function, so it cannot go where it outlives them
```

### How it runs

The function of a nested function takes its declared parameters, then its own closure,
lent, then the address of each capture. Where it is declared, the program fills a record of a code word, an info
word and one field for each capture (the value for a duplicate or a move, the address of
the local for a borrow), and moves the record to the heap. A hidden variable of the block
holds the closure, and frees the record as the block ends. A call through a closure runs a
*thunk*, which takes the record last and calls the function with the address of each
field, or the address a borrowed field holds. A call of the function inside itself passes
its own closure and captures on. A closure value that a call makes and an argument takes
at once waits in a hidden variable, which frees it as its block ends. See [call_value](../vm/instructions.md#the-instructions) and
[Function values in C](../compiler/native.md#function-values-and-closures).

### Limits

- A call that gets a closure, as an argument or through a variable that holds one, can run
  it or not, so what the closure does to the locals it borrows counts as what the call can
  do. After a call that gets a closure that frees `ပ`, a borrow into the block of `ပ` is
  ZK0704, and `ပ` can be `ဗလာ`: compare it with `ဗလာ` before a read, as after each call that
  can free it (`--mem-strict` reports a read that does not, ZK0711). Another argument of
  that call that reaches the block is ZK0707.
- The checker follows the closures a closure holds at each depth, each one once: a closure
  that calls a closure it holds, as a duplicate, a move or a borrow, counts as a call of that
  one too, and so on inward.
- Sharing one closure for good, as JS does, needs a counted box, which is not built:
  each duplicate counts on its own.

## Errors at a glance

| Code | Problem |
| --- | --- |
| ZK0239 | a function kind with parameters inside a list of parameters or fields, not in parentheses |
| ZK0335 | a call that names both a declared function and a value of a function kind |
| ZK0339 | a nested function that uses a local of a function around it, or a closure, that its list leaves out |
| ZK0340 | a capture list that names no local of a function around it |
| ZK0341 | a warning: a listed capture the statements never use |
| ZK0401 | a function value of another kind for a variable or a parameter |
| ZK0462 | a call through a name or a place whose kind is no function kind |
| ZK0463 | the address of an overloaded name where no single function fits |
| ZK0464 | the address of a function that comes from outside |
| ZK0709 | a call through a value that is `ဗလာ` on each path |
| ZK0711 | a warning, with `--mem-strict` alone: a call through a variable that a `ဗလာ` the function wrote can reach |
| ZK0722 | the address of a function that borrows locals, where it would outlive them |
| ZK0723 | the address of a nested function with captures, as the argument of a C function |
| ZK0724 | a duplicate of a closure that owns a block |
