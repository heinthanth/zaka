---
title: The core
description: The stdlib module each file sees with no import, the compiler mark, the built-in traits, and operators on structs.
---

# The core

The core, `"စံကား/အခြေခံအုတ်မြစ်"`, is the stdlib module each file sees with no import
sentence. Its source is the `core` folder of the `stdlib` folder of the source tree:
`mod.zaka` [shares](modules.md#sharing-imported-names) each of the four parts, and a
program cannot import a part on its own. `kinds.zaka` holds the built-in kinds,
`traits.zaka` the built-in traits, `verbs.zaka` the verbs and counts the compiler gives,
and `heap.zaka` the [heap sentences](unsafe.md#heap-statements). A
sentence of a verb, such as `က နှင့် ခ ၏ ပေါင်းလဒ် ကို ပြသမည်`, is a call of the function its
head declares, written as a sentence, and its arguments are whole clauses. Their declarations hold the compiler mark: the compiler gives their
meaning. A trait of the core names the function a struct gives for an operator.

```zaka
နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
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

`ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော` ("included inside the compiler") marks a kind, a
trait or a function of the core. In the head of a kind or a trait, `အများသုံး` stands
right before the noun, after the mark; in the head of a function it stands after `ဟူသော`:

```text
<kind> သည် ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော [အများသုံး] ဒေတာအမျိုးအစား (ဖြစ်သည်။|ဖြစ်ပြီး ရေးသားရာတွင် <hint> ဟု ရေးသားနိုင်သည်။|ဖြစ်၍ <params> ၎င်းကို ရေးသားရာတွင် <hint> ဟု ရေးသားနိုင်သည်။)
<name>[<<params>>] ဟူသည့် ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော [အများသုံး] ဂုဏ်သတ္တိ …
<name> ဟူသော [အများသုံး] လုပ်ဆောင်ချက် သည် ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော လုပ်ဆောင်ချက်ဖြစ်၍ [<params> ၎င်းသည်] [<phrases> ကို သုံး၍] <parameters> လက်ခံမည်။
[အများသုံး] ကိန်းသေ <name> သည် ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော <kind> တစ်<word> ဖြစ်သည်။
```

`<params>` is the word form of the kind parameters, `ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍`.

- A marked kind names a built-in kind for its doc comment: `ကိန်းပြည့်`, `ဒဿမကိန်း`,
  `စာလုံး`, `စာသား`, `မှန်မှားတန်ဖိုး`, and, with their kind parameters, the list `စာရင်း`
  and the table `ဇယား`. Those two end in a [written-as hint](generics.md#written-as-hints),
  `A စာရင်း` and `K ဖြင့် ညွှန်းသော V ဇယား`, which tells how a program writes them.
- A marked trait: the built-in kinds have it through the compiler's rule, and each
  kind a program declares has it through its functions, with the rule of
  [Traits](generics.md#traits).
- A marked function has no block: a sentence of the language stands for each call of it,
  and its prepositions are the words of that sentence. `ပြသ` is `<value> ကို ပြသမည်။` (or `ဟု`);
  `ထည့်`, `ဖျက်` and `အစားထိုး` are the sentences that add, remove and replace; and
  `အကွက်အရေအတွက်`, `စာလုံးအရေအတွက်` and `အညွှန်းအရေအတွက်` are the counts read after `၏`.
  A function a file declares with one of these names joins the overloads, as below.
- A marked constant names no value: the compiler gives it the value of the target it
  builds for. The four of the core are the facts a
  [compile-time conditional](compile-time.md) reads, and a file that names one sees it
  with no import.
- The mark belongs to the modules of the stdlib, the core among them: in another
  file it is ZK0349.

## What the core declares

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
| `အလုပ်လုပ်မည့်ကွန်ပျူတာစနစ်`, `အလုပ်လုပ်မည့်စီပီယူဗိသုကာပုံစံ`, `အလုပ်လုပ်မည့်အပလီကေးရှင်းဘိုင်နရီကြားခံစနစ်`, `အလုပ်လုပ်မည့်ပလက်ဖောင်းဘစ်`, `ကွန်ပိုင်လာအထွက်ကုဒ်အမျိုးအစား` | the OS, the CPU, the ABI, the platform bit count and the code of the target ([Compile-time conditionals](compile-time.md)) |

A name a file declares hides the core's: a trait named `ပေါင်းနိုင်သော` in a file is
the file's own. The compiler reads the core for a program that names one of its
traits or one of its constants. The built-in kinds, verbs and counts are names, not reserved words
([Reserved words](../reference/reserved-words.md)).

## Verbs on structs

A file can declare a function named after a verb or a count of the core. When that
function takes the operands of a sentence, the sentence calls it; else the sentence keeps
the meaning the compiler gives. So `လ ကို ပြသမည်။` prints a struct through the file's own
`ပြသ`, and `၅ ကို ပြသမည်။` prints the number as before. A function with the same kinds
and prepositions as the core's own takes over each sentence of it in the file, which is the
warning ZK0350.

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု ပါဝင်သည်။

ပြသ ဟူသော လုပ်ဆောင်ချက် သည် လ ဟူသော လူ တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"လူ: {လ ၏ အမည်}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

အကွက်အရေအတွက် ဟူသော လုပ်ဆောင်ချက် သည် လ ဟူသော လူ တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
၁ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လ သည် လူ(အမည်: "မောင်") ဖြစ်ပါစေ။
လ ကို ပြသမည်။
၅ ကို ပြသမည်။
လ ၏ အကွက်အရေအတွက် ကို ပြသမည်။
[၁၊ ၂] ၏ အကွက်အရေအတွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
လူ: မောင်
၅
၁
၂
```

## Operators on structs

A struct takes an operator through a function of its own, the function the trait of the
operator names, with two parameters of the struct. The word operators, such as
`က နှင့် ခ ၏ ပေါင်းလဒ်`, are reserved words of the grammar, as the signs are: on numbers
`က နှင့် ခ ၏ ပေါင်းလဒ်` is `က + ခ`, and on a struct each one calls the function of the struct,
as the sign does. The function names are no reserved words: a struct declares `ပေါင်း` as it
declares each function.

| Operator, verb | Function |
| --- | --- |
| `+`, `… ၏ ပေါင်းလဒ်` | `ပေါင်း` |
| `-`, `… ၏ နုတ်လဒ်` | `နုတ်` |
| `*`, `… ၏ မြှောက်လဒ်` | `မြှောက်` |
| `/`, `… ကို … ဖြင့် စားသော စားလဒ်` | `စား` |
| `%`, `… ကို … ဖြင့် စားသော စားကြွင်း` | `အကြွင်းရှာ` |
| `&`, `… ၏ ဘစ်အလိုက် ဆုံလဒ်` | `ဘစ်ဆုံ` |
| `\|`, `… ၏ ဘစ်အလိုက် ပေါင်းစုလဒ်` | `ဘစ်ပေါင်းစု` |
| `^`, `… ၏ ဘစ်အလိုက် ခြားနားလဒ်` | `ဘစ်ခြားနား` |
| `~`, `… ၏ ဘစ်အလိုက် ပြောင်းပြန်ပြောင်းလဒ်` | `ဘစ်ပြောင်းပြန်`, one parameter |
| `<<`, `… ကို ဘယ်သို့ … ဘစ် ရွေ့သောရလဒ်` | `ဘစ်ဘယ်ရွေ့`, with the distance |
| `>>`, `… ကို ညာသို့ … ဘစ် ရွေ့သောရလဒ်` | `ဘစ်ညာရွေ့`, with the distance |
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

ပေါင်း ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ဗက်တာ တစ်ခု ကို လည်းကောင်း ခ ဟူသော ဗက်တာ တစ်ခု ကို လည်းကောင်း
လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ ဗက်တာ တစ်ခု ကို ပြန်ပေးမည်။
ဗက်တာ(x: (က ၏ x) + (ခ ၏ x)၊ y: (က ၏ y) + (ခ ၏ y)) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

စာသားအဖြစ်ပြောင်း ဟူသော လုပ်ဆောင်ချက် သည်
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
