---
title: The prelude
description: The stdlib module each file sees with no import, the compiler mark, the built-in traits, and operators on structs.
---

# The prelude

The prelude, `"စံကား/အခြေခံအုတ်မြစ်"`, is the stdlib module each file sees with no import
sentence. Its source is three files of the `stdlib` folder of the source tree, which the
compiler joins into one module: `core_kinds.zaka` holds the built-in kinds,
`core_traits.zaka` the built-in traits, and `core_verbs.zaka` the verbs and counts the
compiler gives. Their declarations hold the compiler mark: the compiler gives their
meaning. A trait of the prelude names the function a struct gives for an operator.

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

`ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော` ("included inside the compiler") marks a kind, a
trait or a function of the prelude. In the head of a kind or a trait, `အများသုံး` stands
right before the noun, after the mark:

```text
<kind> သည် [<params> ၎င်းသည်] ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော [အများသုံး] ဒေတာအမျိုးအစား ဖြစ်သည်။
<name>[<<params>>] ဟူသည့် ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော [အများသုံး] ဂုဏ်သတ္တိ …
[အများသုံး] လုပ်ဆောင်ချက် <name> သည် [<params> ၎င်းသည်] ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော လုပ်ဆောင်ချက်ဖြစ်၍ <parameters> လက်ခံမည်။
```

`<params>` is the word form of the kind parameters, `ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍`.

- A marked kind names a built-in kind for its doc comment: `ကိန်းပြည့်`, `ဒဿမကိန်း`,
  `စာလုံး`, `စာသား`, `မှန်မှားတန်ဖိုး`, and, with their kind parameters, the list `စာရင်း`
  and the table `ဇယား`. How a program writes each kind remains grammar: `A စာရင်း`,
  `K ဖြင့် ညွှန်းသော V ဇယား`.
- A marked trait: the built-in kinds have it through the compiler's rule, and each
  kind a program declares has it through its functions, with the rule of
  [Traits](generics.md#traits).
- A marked function has no block: a sentence of the language stands for each call of it,
  and its prepositions are the words of that sentence. `ပြသ` is `<value> ကို ပြသမည်။` (or `ဟု`);
  `ထည့်`, `ဖျက်` and `အစားထိုး` are the sentences that add, remove and replace; and
  `အကွက်အရေအတွက်`, `စာလုံးအရေအတွက်` and `အညွှန်းအရေအတွက်` are the counts read after `၏`.
  A function a file declares with one of these names joins the overloads, as below.
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
traits. The built-in kinds, verbs and counts are names, not reserved words
([Reserved words](../reference/reserved-words.md)).

## Verbs on structs

A file can declare a function named after a verb or a count of the prelude. When that
function takes the operands of a sentence, the sentence calls it; else the sentence keeps
the meaning the compiler gives. So `လ ကို ပြသမည်။` prints a struct through the file's own
`ပြသ`, and `၅ ကို ပြသမည်။` prints the number as before. A function with the same kinds
and prepositions as the prelude's own takes over each sentence of it in the file, which is the
warning ZK0350.

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု ပါဝင်သည်။

လုပ်ဆောင်ချက် ပြသ သည် လ ဟူသော လူ တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"လူ: {လ ၏ အမည်}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် အကွက်အရေအတွက် သည် လ ဟူသော လူ တစ်ခု ကို လက်ခံပြီး
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
operator names, with two parameters of the struct. The word operators are calls of these
functions: the prelude gives each trait function its prepositions, `နှင့်` and `ကို` for the sum,
the difference, the product and the bit operators of two values, `ကို` and `ဖြင့်` for the
quotient, the remainder and the shifts, and `ကို` for `ဘစ်ပြောင်းပြန်`. A file sees each of
them, so `က နှင့် ခ ကို ပေါင်းသည့်ရလဒ်` is `က + ခ` on numbers and the call of the struct's
`ပေါင်း` on a struct. A struct function without these prepositions still takes the sign
operator; the word form reaches it as the operator does.

| Operator, verb | Function |
| --- | --- |
| `+`, `… ကို ပေါင်းသည့်ရလဒ်` | `ပေါင်း` |
| `-`, `… ကို နုတ်သည့်ရလဒ်` | `နုတ်` |
| `*`, `… ကို မြှောက်သည့်ရလဒ်` | `မြှောက်` |
| `/`, `… စားသည့်ရလဒ်` | `စား` |
| `%`, `… အကြွင်းရှာသည့်ရလဒ်` | `အကြွင်းရှာ` |
| `&`, `… ကို ဘစ်ဆုံသည့်ရလဒ်` | `ဘစ်ဆုံ` |
| `\|`, `… ကို ဘစ်ပေါင်းစုသည့်ရလဒ်` | `ဘစ်ပေါင်းစု` |
| `^`, `… ကို ဘစ်ခြားနားသည့်ရလဒ်` | `ဘစ်ခြားနား` |
| `~`, `… ကို ဘစ်ပြောင်းပြန်သည့်ရလဒ်` | `ဘစ်ပြောင်းပြန်`, one parameter |
| `<<`, `… ကို … ဖြင့် ဘစ်ဘယ်ရွေ့သည့်ရလဒ်` | `ဘစ်ဘယ်ရွေ့`, with the distance |
| `>>`, `… ကို … ဖြင့် ဘစ်ညာရွေ့သည့်ရလဒ်` | `ဘစ်ညာရွေ့`, with the distance |
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
