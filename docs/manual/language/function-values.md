---
title: Function values
description: Function kinds, the address of a function, calls through a function value, ဗလာ, comparing and printing function values, and the errors of each.
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
အမျိုးအစား လုပ်ငန်း ဆိုသည်မှာ ပေါင်း၊ မြှောက် တို့ ဖြစ်သည်။

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

A function value holds no borrow and no block, so it is duplicated on assignment and on
a call, never moved and never freed, and it can sit in a global and cross a module edge.
The address of a function is known when the program is compiled, so it can be the value
of a constant, a cell of a constant list, and the starting value of a variable or a
constant in an imported file: a module can export a table of functions.
A function value is one word: in the VM the number of the function plus one, so 0 is
`ဗလာ`; in native code the code address.

A call through a function value names no function, but such a value comes from the
address of a function alone. So the [borrow checker](../internals/borrow-checker.md)
treats the call as a call of each function whose address the program takes and whose
kind fits, and joins what each of them frees and keeps.

## Errors at a glance

| Code | Problem |
| --- | --- |
| ZK0239 | a function kind with parameters inside a list of parameters or fields, not in parentheses |
| ZK0335 | a call that names both a declared function and a value of a function kind |
| ZK0401 | a function value of another kind for a variable or a parameter |
| ZK0462 | a call through a name or a place whose kind is no function kind |
| ZK0463 | the address of an overloaded name where no single function fits |
| ZK0464 | the address of a function that comes from outside |
| ZK0709 | a call through a value that is `ဗလာ` on each path |
| ZK0711 | a warning, with `--mem-strict` alone: a call through a variable that a `ဗလာ` the function wrote can reach |
