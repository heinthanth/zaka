---
title: The prelude
description: The stdlib module each file sees with no import, the compiler mark, the built-in traits and functions, and operators on structs.
---

# The prelude

The prelude, `"စံကား/အခြေခံအုတ်မြစ်"`, is the stdlib module each file sees with no import
sentence. It declares the built-in kinds, the built-in traits and the functions the
operators and verbs stand for, so a reader can open `stdlib/prelude.zaka` in the
source tree and see them. Its declarations hold the compiler mark: the compiler gives
their meaning and their code.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပေါင်း(၁၊ ၂) ကို ပြသမည်။
ငယ်မငယ်စစ်(၁၊ ၂) ကို ပြသမည်။
စာသားအဖြစ်ပြောင်း('က') ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၃
မှန်
က
```

## The compiler mark

`ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော` ("included inside the compiler") stands where
`ပြင်ပမှလာသော` stands on an external declaration:

```text
[အများသုံး] လုပ်ဆောင်ချက် <name>[<<params>>] သည် ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော လုပ်ဆောင်ချက် ဖြစ်၍ <params> လက်ခံပြီး <kind> ကို ပြန်ပေးမည်။
<name>[<<params>>] ဟူသည့် [အများသုံး] ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော ဂုဏ်သတ္တိ …
[အများသုံး] <kind> သည် ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော ဒေတာအမျိုးအစား ဖြစ်သည်။
```

- A marked function has a head and no block, as an external function has; the
  compiler gives its code. The head takes the forms of an external one: `ဖြစ်သည်` for no
  parameters and no result, or `ဖြစ်၍` and the parameters or the result.
- A marked trait: the built-in kinds have it through the compiler's rule, and each
  kind a program declares has it through its functions, with the rule of
  [Traits](generics.md#traits).
- A marked kind names a built-in kind for its doc comment. Its name is a name, not a
  reserved word: a struct, an enumeration, an alias or a kind parameter of that name
  in a file hides the built-in kind there, and a variable can take the name. The sign
  words and `N ဘစ်` before it remain grammar, and a built-in kind name where a value
  must stand is ZK0202.
- The mark belongs to the modules of the stdlib, the prelude among them: in another
  file it is ZK0349.

## What the prelude declares

| Declaration | What it is |
| --- | --- |
| `ကိန်းပြည့်`, `ဒဿမကိန်း`, `စာလုံး`, `စာသား`, `မှန်မှားတန်ဖိုး` | the built-in kinds |
| `ပေါင်းနိုင်သော`, `နုတ်နိုင်သော`, `မြှောက်နိုင်သော`, `စားနိုင်သော` | the kinds that take `+`, `-`, `*` and `/`: the numbers, text for `+`, and each kind with the function |
| `နံပါတ်နှင့်တူသော` | the kinds with the four traits above |
| `နှိုင်းယှဉ်နိုင်သော` | the kinds that take `< > <= >=`: the numbers, the characters, and each kind with `ငယ်မငယ်စစ်` and `ကြီးမကြီးစစ်` |
| `တူမတူစစ်ဆေးနိုင်သော` | the kinds that take `==` and `!=`: each kind |
| `စာသားအဖြစ်ပြောင်းလဲပြသနိုင်သော` | the kinds `ပြသမည်` prints: each kind but a handle of a kind from outside |
| `ပေါင်း`, `နုတ်`, `မြှောက်`, `စား` | `က + ခ`, `က - ခ`, `က * ခ`, `က / ခ` as functions |
| `ငယ်မငယ်စစ်`, `ကြီးမကြီးစစ်`, `ညီမညီစစ်` | `က < ခ`, `က > ခ`, `က == ခ` as functions |
| `စာသားအဖြစ်ပြောင်း` | a value as the text `ပြသမည်` prints |
| `ပြသ` | `ပြသမည်` as a function |

A name a file declares hides the prelude's: a file with a function `ပေါင်း` of its own
calls its own. The compiler reads the prelude for a program that names one of its
declarations.

## Marked functions of other modules

The functions of one kind of value live in a module of their own, imported as each
module is, with the compiler mark and marked parameters, so a call reads as the
statement it stands for:

| Module | Function | Call |
| --- | --- | --- |
| `"စံကား/စာရင်း"` | `ထည့်` | `က ကို ကွက်များ ထဲသို့ ထည့် မည်။`, as `ကွက်များ ထဲသို့ က ကို ထည့်မည်။` |
| `"စံကား/စာရင်း"` | `ဖျက်` | `ဖျက်(ကွက်များ)`, as `ကွက်များ ၏ နောက်ဆုံးအကွက် ကို ဖျက်မည်။` |
| `"စံကား/စာသား"` | `အစားထိုး` | `စာ တွင် စ မှ ဆုံး အထိ အသစ် ဖြင့် အစားထိုး မည်။`, as `စာ ၏ အကွက် စ မှ ဆုံး ထိ ကို အသစ် ဖြင့် အစားထိုးမည်။` |

```zaka
"စံကား/စာရင်း" မှ ထည့် ကို ထည့်သွင်းမည်။
"စံကား/စာသား" မှ အစားထိုး ကို ထည့်သွင်းမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းစု သည် [၁၊ ၂] ဖြစ်ပါစေ။
၃ ကို ကိန်းစု ထဲသို့ ထည့် မည်။
ကိန်းစု ကို ပြသမည်။
ကိန်းရှင် စာ သည် "မင်္ဂလာပါ" ဖြစ်ပါစေ။
စာ တွင် ၀ မှ ၂ အထိ "ဟယ်" ဖြင့် အစားထိုး မည်။
စာ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၂၊ ၃]
ဟယ်င်္ဂလာပါ
```

A marked call keeps the name and `မည်` apart (`ထည့် မည်`); joined, `ထည့်မည်` is the
statement verb.

## Operators on structs

A struct takes an operator through a function of its own, the function the trait of the
operator names, with two parameters of the struct:

| Operator, verb | Function |
| --- | --- |
| `+`, `… ၏ ပေါင်းလဒ်` | `ပေါင်း` |
| `-`, `… ၏ နုတ်လဒ်` | `နုတ်` |
| `*`, `… ၏ မြှောက်လဒ်` | `မြှောက်` |
| `/`, `… စားသော စားလဒ်` | `စား` |
| `<`, `ငယ်`; `>=`, `မငယ်` | `ငယ်မငယ်စစ်`; its negation |
| `>`, `ကြီး`; `<=`, `မကြီး` | `ကြီးမကြီးစစ်`; its negation |
| `==`, `ညီ`; `!=`, `မညီ` | `ညီမညီစစ်`; its negation |
| `ပြသမည်`, a template slot | `စာသားအဖြစ်ပြောင်း` |

- The function is the one of the file of the use or of the file of the struct whose
  parameter kinds are the two kinds of the struct; a test gives back `မှန်မှားတန်ဖိုး`.
- A struct with no `ညီမညီစစ်` compares one field at a time, and one with no
  `စာသားအဖြစ်ပြောင်း` prints its fields, as before.
- The other operators keep their rules: a bit operator or a shift on a struct is an
  error.

```zaka
ဗက်တာ ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု နှင့် y ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

လုပ်ဆောင်ချက် ပေါင်း သည်
က ဟူသော ဗက်တာ တစ်ခု ကို လည်းကောင်း ခ ဟူသော ဗက်တာ တစ်ခု ကို လည်းကောင်း
လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ ဗက်တာ တစ်ခု ကို ပြန်ပေးမည်။
ဗက်တာ(x: (က ၏ x) + (ခ ၏ x)၊ y: (က ၏ y) + (ခ ၏ y)) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် စာသားအဖြစ်ပြောင်း သည်
က ဟူသော ဗက်တာ တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"<{က ၏ x}၊ {က ၏ y}>" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ဗက်တာ(x: ၁၊ y: ၂) ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ဗက်တာ(x: ၃၊ y: ၄) ဖြစ်ပါစေ။
က + ခ ကို ပြသမည်။
"ပေါင်းလဒ်: {က + ခ}" ဟု ပြသမည်။
က == ခ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
<၄၊ ၆>
ပေါင်းလဒ်: <၄၊ ၆>
မမှန်
```

## Errors

| Code | When |
| --- | --- |
| ZK0349 | the compiler mark outside the modules of the stdlib |
