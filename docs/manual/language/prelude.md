---
title: The prelude
description: The stdlib module each file sees with no import, the compiler mark, the built-in traits, and operators on structs.
---

# The prelude

The prelude, `"စံကား/အခြေခံအုတ်မြစ်"`, is the stdlib module each file sees with no import
sentence. It declares the built-in traits, so a reader can open `stdlib/prelude.zaka`
in the source tree and see them. Its declarations hold the
compiler mark: the compiler gives their meaning. The verbs and the operators are
grammar; a trait of the prelude names the function a struct gives for an operator.

```zaka
လုပ်ဆောင်ချက် နှစ်ဆ သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
က ဟူသော T တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
ဤနေရာတွင် သတိပြုရန်မှာ
T တွင် ပေါင်းနိုင်သော ဂုဏ်သတ္တိ ရှိသည်။
သတိပြုစရာများ ပြီးဆုံးပါပြီး။
က + က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
နှစ်ဆ(၂) ကို ပြသမည်။
နှစ်ဆ("ဟယ်") ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄
ဟယ်ဟယ်
```

## The compiler mark

`ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော` ("included inside the compiler") stands after
`ဟူသည့်` in the head of a trait:

```text
<name>[<<params>>] ဟူသည့် [အများသုံး] ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော ဂုဏ်သတ္တိ …
```

- A marked trait: the built-in kinds have it through the compiler's rule, and each
  kind a program declares has it through its functions, with the rule of
  [Traits](generics.md#traits).
- The mark heads a trait alone; a function or a kind with it is a malformed declaration.
- The mark belongs to the modules of the stdlib, the prelude among them: in another
  file it is ZK0349.

## What the prelude declares

| Declaration | What it is |
| --- | --- |
| `ပေါင်းနိုင်သော`, `နုတ်နိုင်သော`, `မြှောက်နိုင်သော`, `စားနိုင်သော` | the kinds that take `+`, `-`, `*` and `/`: the numbers, text for `+`, and each kind with the function |
| `နံပါတ်နှင့်တူသော` | the kinds with the four traits above |
| `အကြွင်းရှာနိုင်သော` | the kinds that take `%`: the numbers, and each kind with `အကြွင်းရှာ` |
| `ဘစ်ဆုံနိုင်သော`, `ဘစ်ပေါင်းစုနိုင်သော`, `ဘစ်ခြားနားနိုင်သော`, `ဘစ်ပြောင်းပြန်ပြောင်းလဲနိုင်သော` | the kinds that take `&`, `\|`, `^` and `~`: the integers, and each kind with the function |
| `ဘစ်ဘယ်ရွေ့နိုင်သော`, `ဘစ်ညာရွေ့နိုင်သော` | the kinds that take `<<` and `>>`: the integers, and each kind with the function |
| `အနုတ်ပြောင်းနိုင်သော` | the kinds that take the `-` of one operand: the signed numbers, and each kind with `အနုတ်ပြောင်း` |
| `နှိုင်းယှဉ်နိုင်သော` | the kinds that take `< > <= >=`: the numbers, the characters, and each kind with `ငယ်မငယ်စစ်` and `ကြီးမကြီးစစ်` |
| `တူမတူစစ်ဆေးနိုင်သော` | the kinds that take `==` and `!=`: each kind |
| `စာသားအဖြစ်ပြောင်းလဲပြသနိုင်သော` | the kinds `ပြသမည်` prints: each kind but a handle of a kind from outside |

A name a file declares hides the prelude's: a trait named `ပေါင်းနိုင်သော` in a file is
the file's own. The compiler reads the prelude for a program that names one of its
traits. The built-in kinds are names the compiler knows, not declarations of the
prelude ([Reserved words](../reference/reserved-words.md)).

## Operators on structs

A struct takes an operator through a function of its own, the function the trait of the
operator names, with two parameters of the struct:

| Operator, verb | Function |
| --- | --- |
| `+`, `… ၏ ပေါင်းလဒ်` | `ပေါင်း` |
| `-`, `… ၏ နုတ်လဒ်` | `နုတ်` |
| `*`, `… ၏ မြှောက်လဒ်` | `မြှောက်` |
| `/`, `… စားသော စားလဒ်` | `စား` |
| `%`, `… စားသော စားကြွင်း` | `အကြွင်းရှာ` |
| `&`, `… ၏ ဘစ်အလိုက် ဆုံလဒ်` | `ဘစ်ဆုံ` |
| `\|`, `… ၏ ဘစ်အလိုက် ပေါင်းစုလဒ်` | `ဘစ်ပေါင်းစု` |
| `^`, `… ၏ ဘစ်အလိုက် ခြားနားလဒ်` | `ဘစ်ခြားနား` |
| `~`, `… ၏ ဘစ်အလိုက် ပြောင်းပြန်ပြောင်းလဒ်` | `ဘစ်ပြောင်းပြန်`, one parameter |
| `<<`, `… ဘယ်သို့ … ဘစ် ရွေ့သောရလဒ်` | `ဘစ်ဘယ်ရွေ့`, with the distance |
| `>>`, `… ညာသို့ … ဘစ် ရွေ့သောရလဒ်` | `ဘစ်ညာရွေ့`, with the distance |
| the `-` of one operand, `အနုတ်` | `အနုတ်ပြောင်း`, one parameter |
| `<`, `ငယ်`; `>=`, `မငယ်` | `ငယ်မငယ်စစ်`; its negation |
| `>`, `ကြီး`; `<=`, `မကြီး` | `ကြီးမကြီးစစ်`; its negation |
| `==`, `ညီ`; `!=`, `မညီ` | `ညီမညီစစ်`; its negation |
| `ပြသမည်`, a template slot | `စာသားအဖြစ်ပြောင်း` |

- The function is the one of the file of the use or of the file of the struct whose
  parameter kinds are the two kinds of the struct; a test gives back `မှန်မှားတန်ဖိုး`.
  `~` and the `-` of one operand take the struct alone; a shift takes the struct and
  the distance, `လက္ခဏာမဲ့ ကိန်းပြည့်`, as the distance of a shift is unsigned.
- A struct with no `ညီမညီစစ်` compares one field at a time, and one with no
  `စာသားအဖြစ်ပြောင်း` prints its fields, as before.
- A struct with no function for an operator keeps the rule of the operator: `&` on
  such a struct is an error, as before.

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
