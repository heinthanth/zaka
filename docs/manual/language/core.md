---
title: The core module
description: The stdlib module that each file sees with no import - its parts, the compiler mark, what it declares, the verbs a program can take over, and operators on structs.
---

# The core module

The core module (prelude), `"စံကား/အခြေခံအုတ်မြစ်"`, is the stdlib module that each file
sees without an import sentence. It declares the built-in data kinds (types), the built-in
traits, the verbs of the language and the facts of the target, and a trait of the core
module names the function that a struct needs for an operator. The Learn book meets it in
[Generics and traits](../../learn/generics/traits.md).

```zaka
နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
က ဟူသော T တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
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

## The parts of the core module

The source of the core module is the `core` folder inside the `stdlib` folder of the
source tree. Its `mod.zaka` [shares](modules.md#sharing-imported-names) four parts, and a
program cannot import one part on its own:

| File | What it declares |
| --- | --- |
| `kinds.zaka` | the built-in data kinds, and the facts of the target |
| `traits.zaka` | the built-in traits |
| `verbs.zaka` | the verbs and the counts that the compiler provides |
| `heap.zaka` | the [heap statements](heap.md#heap-statements) |

A verb is a function that we call as a sentence, such as `က နှင့် ခ ၏ ပေါင်းလဒ် ကို
ပြသမည်`. Its arguments are whole clauses, and its function signature in the core module
gives the prepositions that the sentence uses. These declarations have the compiler mark,
because the compiler provides their meaning.

## The compiler mark

```text
<kind> သည် ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော [အများသုံး] ဒေတာအမျိုးအစား (ဖြစ်သည်။|ဖြစ်ပြီး ရေးသားရာတွင် <hint> ဟု ရေးသားနိုင်သည်။|ဖြစ်၍ <params> ၎င်းကို ရေးသားရာတွင် <hint> ဟု ရေးသားနိုင်သည်။)
<name>[<<params>>] ဟူသည့် ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော [အများသုံး] ဂုဏ်သတ္တိ …
<name> ဟူသော [အများသုံး] လုပ်ဆောင်ချက် သည် ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော လုပ်ဆောင်ချက်ဖြစ်၍ [<params> ၎င်းသည်] [<phrases> ကို သုံး၍] <parameters> လက်ခံမည်။
[အများသုံး] ကိန်းသေ <name> သည် ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော <kind> တစ်<word> ဖြစ်သည်။
```

We write `ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော` ("included inside the compiler") to mark a
data kind, a trait, a function or a constant of the core module. In the declaration of a
data kind or a trait, `အများသုံး` goes right before the noun, after the mark, and in a
function signature it goes after `ဟူသော`. `<params>` is the word form of the kind
parameters, `ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍`.

| What is marked | What the mark means |
| --- | --- |
| a data kind | names a built-in data kind, so that it can have a doc comment: `ကိန်းပြည့်`, `ဒဿမကိန်း`, `စာလုံး`, `စာသား`, `မှန်မှားတန်ဖိုး`, and, with their kind parameters, the list `စာရင်း`, the table `ဇယား`, the thread `အပြိုင်လုပ်ငန်း` and the channel `ဒေတာလမ်းကြောင်း`. Those four end in a [written-as hint](generics.md#written-as-hints), such as `A စာရင်း` or `K ဖြင့် ညွှန်းသော V ဇယား`, which shows how a program writes them. |
| a trait | the built-in data kinds have the trait through a rule of the compiler, and each data kind that a program declares has it through its functions, under the rules of [Traits](traits.md) |
| a function | the function has no block, because each call of it is written as a sentence of the language, and its prepositions are the words of that sentence |
| a constant | the constant has no value in the source, and the compiler gives it the value of the target that it builds for |

The marked functions are the verbs and the counts:

- `ပြသ` is the sentence `<value> ကို ပြသမည်။` (or `ဟု`).
- `ထည့်`, `ဖျက်` and `အစားထိုး` are the sentences that add, remove and replace, and
  `ကူးထည့်` is the sentence that puts a duplicate (copy) of a value into a place, or a
  duplicate of all with `အကုန်ကူးထည့်` ([Values and objects](objects.md)).
- `စောင့်`, `ပို့`, `ပိတ်`, `ပေါင်းထည့်` and `နုတ်ယူ`, with a second `အစားထိုး`, are the
  sentences of [threads](threads.md): wait, send, close, and the steps on an atomic
  number.
- `အကွက်အရေအတွက်`, `စာလုံးအရေအတွက်` and `အညွှန်းအရေအတွက်` are the counts that we read
  after `၏`.

A function that a file declares with one of these names joins the overloads of the verb,
as [Verbs on structs](#verbs-on-structs) shows. The five marked constants are the facts
that a [compile-time conditional](compile-time.md#the-facts-of-the-target) reads, and a
file that names one sees it without an import.

The mark belongs to the modules of the stdlib, which include the core module, and in
another file it is ZK0349.

## What the core module declares

| Declaration | What it is |
| --- | --- |
| `ပေါင်းနိုင်သော`, `နုတ်နိုင်သော`, `မြှောက်နိုင်သော`, `စားနိုင်သော` | the data kinds that work with `+`, `-`, `*` and `/`: the numbers, text for `+`, and each data kind with the function |
| `နံပါတ်နှင့်တူသော` | the data kinds with all four traits above |
| `အကြွင်းရှာနိုင်သော` | the data kinds that work with `%`: the numbers, and each data kind with `အကြွင်းရှာ` |
| `ဘစ်ဆုံနိုင်သော`, `ဘစ်ပေါင်းစုနိုင်သော`, `ဘစ်ခြားနားနိုင်သော`, `ဘစ်ပြောင်းပြန်ပြောင်းလဲနိုင်သော` | the data kinds that work with `&`, `\|`, `^` and `~`: the integers, and each data kind with the function |
| `ဘစ်ဘယ်ရွေ့နိုင်သော`, `ဘစ်ညာရွေ့နိုင်သော` | the data kinds that work with `<<` and `>>`: the integers, and each data kind with the function |
| `အနုတ်ပြောင်းနိုင်သော` | the data kinds that work with the `-` of one operand: the signed numbers, and each data kind with `အနုတ်ပြောင်း` |
| `နှိုင်းယှဉ်နိုင်သော` | the data kinds that work with `< > <= >=`: the numbers, the characters, and each data kind with `ငယ်မငယ်စစ်` and `ကြီးမကြီးစစ်` |
| `တူမတူစစ်ဆေးနိုင်သော` | the data kinds that work with `==` and `!=`: all data kinds |
| `စာသားအဖြစ်ပြောင်းလဲပြသနိုင်သော` | the data kinds that `ပြသမည်` prints: all data kinds except a handle of a data kind from outside |
| `ပြောင်းလဲနိုင်သော`, `စမ်းသပ်ပြောင်းလဲနိုင်သော` | the data kinds that convert to another with `အဖြစ် ပြောင်းလဲသော ရလဒ်`, through the compiler's table or a `ပြောင်းလဲ`, and those that convert with a failure, through a `စမ်းသပ်ပြောင်းလဲ` ([Conversions](conversions.md#conversions-through-traits)) |
| `ကိန်းဂဏန်းဖတ်ခြင်းအမှားအယွင်း` | the failure of a number read from text, or of an integer that does not fit its kind ([Conversions](conversions.md#integers-that-must-fit)) |
| `စမ်းသပ်ပြောင်းလဲ`, generic over two integer kinds | the conversion of one integer to another that fails when the value does not fit, which `ပြောင်းလဲသော ရလဒ်ကောင်း` calls |
| `အလုပ်လုပ်မည့်ကွန်ပျူတာစနစ်`, `အလုပ်လုပ်မည့်စီပီယူဗိသုကာပုံစံ`, `အလုပ်လုပ်မည့်အပလီကေးရှင်းဘိုင်နရီကြားခံစနစ်`, `အလုပ်လုပ်မည့်ပလက်ဖောင်းဘစ်`, `ကွန်ပိုင်လာအထွက်ကုဒ်အမျိုးအစား` | the OS, the CPU, the ABI, the platform bit count and the code of the target ([Compile time](compile-time.md#the-facts-of-the-target)) |

- A name that a file declares hides the name of the core module, so a trait named
  `ပေါင်းနိုင်သော` in a file is the file's own trait.
- The compiler reads the core module for a program that names one of its traits, its
  enumerations or its constants, or writes `ပြောင်းလဲသော ရလဒ်ကောင်း`. A file sees the data
  kinds of the core module with no import.
- A generic function of a file with the name of a generic function of the core module
  hides that one when the two take the same parameters and give the same result, and
  stands beside it otherwise.
- The built-in data kinds, verbs and counts are names, not reserved words ([Reserved
  words](../reference/reserved-words.md)).

## Verbs on structs

A file can declare a function named after a verb or a count of the core module. When that
function fits the operands of a sentence, the sentence calls it, and otherwise the sentence
keeps the meaning that the compiler gives. So `လ ကို ပြသမည်။` prints a struct through the
file's own `ပြသ`, while `၅ ကို ပြသမည်။` prints the number as before. A function with the
same data kinds and prepositions as the verb of the core module takes over each sentence of
that verb in the file, which is the warning ZK0350.

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု ပါဝင်သည်။

ပြသ ဟူသော လုပ်ဆောင်ချက် သည် လ ဟူသော လူ တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"လူ: {လ ၏ အမည်}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

အကွက်အရေအတွက် ဟူသော လုပ်ဆောင်ချက် သည်
လ ဟူသော လူ တစ်ခု ကို လက်ခံပြီး
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

To let a struct work with an operator, we declare the function that the trait of the
operator names, with two parameters of the struct. The word operators, such as
`က နှင့် ခ ၏ ပေါင်းလဒ်`, are reserved words of the grammar, just as the signs are. On
numbers, `က နှင့် ခ ၏ ပေါင်းလဒ်` means `က + ခ`, and on a struct each word operator calls
the function of the struct, as the sign does. The function names are not reserved words,
so a struct declares `ပေါင်း` as it declares other functions.

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

- The compiler uses the function, from the file of the use or the file of the struct,
  whose parameter kinds are the two data kinds of the operands. A test returns
  `မှန်မှားတန်ဖိုး`.
- `~` and the `-` of one operand pass the struct alone. A shift passes the struct and the
  distance, which is a `လက္ခဏာမဲ့ ကိန်းပြည့်`, because the distance of a shift is
  unsigned.
- A struct without `ညီမညီစစ်` is compared one field at a time, and a struct without
  `စာသားအဖြစ်ပြောင်း` prints its fields, as before.
- A struct without a function for an operator keeps the rule of the operator, so `&` on
  such a struct is an error, as before.

```zaka
ဗက်တာ ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု နှင့် y ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

ပေါင်း ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ဗက်တာ တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ဗက်တာ တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ဗက်တာ တစ်ခု ကို ပြန်ပေးမည်။
ဗက်တာ(x: (က ၏ x) + (ခ ၏ x)၊ y: (က ၏ y) + (ခ ၏ y)) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

စာသားအဖြစ်ပြောင်း ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ဗက်တာ တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
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
| ZK0350 | a warning: a function with the name, the data kinds and the prepositions of a verb of the core module, which then takes over each sentence of that verb |
