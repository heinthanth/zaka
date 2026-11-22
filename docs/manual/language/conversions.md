---
title: Conversions
description: Turn a value of one data kind into another - the conversion expression, which pairs convert, the conversion traits, what happens to the value, and the widening that needs no words.
---

# Conversions

A value of one data kind (type) can take the place of another data kind through a
conversion or a widening. A conversion (cast) is written out: we name the data kind we
want, and the value is changed to fit it. A widening happens with no words, but just
where the new data kind can contain each value of the old one. Each other change of data
kind is an error.

[Converting between data kinds](../../learn/operators/conversion.md) in the Learn book
shows conversions with gentle examples. This page states the rules.

## The conversion expression

```text
<value>  ကို  <kind> တစ်<measure word>  အဖြစ်  ပြောင်းလဲသော ရလဒ်
 operand       target data kind, counted         the verb
```

| Words | Reads as |
| --- | --- |
| `<value> ကို` | "the value" (the object) |
| `<kind> တစ်ခု အဖြစ်` | "as one `<kind>`" |
| `ပြောင်းလဲသော ရလဒ်` | "the result of changing" |

To convert a value, we write `<value> ကို <kind> တစ်<measure word> အဖြစ် ပြောင်းလဲသော ရလဒ်`.
The verb `ပြောင်းလဲသော ရလဒ်` closes a [clause](expressions.md#three-tiers), so the value
gets `ကို`, and the target data kind, with its [measure word](kinds.md#measure-words),
gets `အဖြစ်`. A data kind with no measure word there is ZK0234. The result has the target
data kind and can get its own particle, so a conversion can be the first part of a longer
phrase.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
က ကို ဒဿမကိန်း တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
က ကို ဒဿမကိန်း တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် နှင့် ၂.၅ ၏ ပေါင်းလဒ် ကို ပြသမည်။
(က ကို ဒဿမကိန်း တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်) / ၄ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂.၀
၄.၅
၀.၅
```

- Because a conversion is a clause, it needs parentheses inside a sum written with signs,
  as the third line shows.
- A conversion to the data kind that the value has changes nothing.
- The compiler works out a conversion of a constant before the program runs.
- Reading a part of a [whole result](reading-results.md#the-whole-result) of a function
  that can fail is not a conversion: `x ထဲမှ ဒေတာ` returns its good result, and
  `x ထဲမှ အမှားအယွင်း ဒေတာ` its error value, as [Errors and results](error-handling.md)
  states.

## Which pairs convert

A conversion works between the pairs in this table and between no others (ZK0408):

| From \ To | integer | decimal number | `စာလုံး` | `မှန်မှားတန်ဖိုး` |
| --- | --- | --- | --- | --- |
| integer | allowed | allowed | allowed | refused |
| decimal number | allowed | allowed | refused | refused |
| `စာလုံး` | allowed | refused | allowed | refused |
| `မှန်မှားတန်ဖိုး` | allowed | refused | refused | allowed |

An integer never becomes a true/false value, so to test an integer we compare it instead:
`n != ၀`. A few more pairs follow their own rules:

| From | To | Rule |
| --- | --- | --- |
| a plain list of `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်` | `စာသား` | allowed; the VM checks that the u8s are UTF-8 and stops the program when the u8s are not ([Text](text.md)) |
| each other list | a data kind other than `စာသား` | ZK0408 |
| each value but a handle of a data kind from outside | `စာသား` | the text that `ပြသမည်` prints ([Conversions to text](#conversions-to-text)) |
| a counted list | a plain list | needs no conversion: it happens where a value goes into a place ([Counted lists and storage](counted-lists.md#forms)) |
| a member of an enumeration | its value kind | allowed, and returns the member's value ([Enumerations](enums.md#values-of-an-enumeration)) |
| a member whose value is a number, a character or a true/false value | each data kind that its value converts to | allowed |
| a data kind | a member of an enumeration | refused |
| a value of an enumeration whose members hold data | a member, written with no measure word | the data of the member, or a trap for another member ([Reading the data](enums.md#reading-the-data)) |
| a pointer and an integer, two pointer kinds, a shared object and a pointer | | allowed in unsafe code alone, and ZK0468 outside it ([Unsafe Zaka](unsafe.md)) |

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [] ဖြစ်ပါစေ။
စ ထဲသို့ ၁၀၄ ကို ထည့်မည်။
စ ထဲသို့ ၁၀၅ ကို ထည့်မည်။
စ ကို စာသား တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
hi
```

```zaka-error ZK0408
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ ကို မှန်မှားတန်ဖိုး တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0408]: cannot convert isize to bool
```

## Conversions through traits

```text
<value> ကို <kind> တစ်<measure word> အဖြစ် ပြောင်းလဲသော ရလဒ်
<value> ကို <kind> တစ်<measure word> အဖြစ် ပြောင်းလဲသော ရလဒ်ကောင်း
```

When the table has no pair for two data kinds, a conversion goes through one of the two
conversion [traits](traits.md#built-in-traits) (`From` and `TryFrom`) of the [core
module](core.md):

| Trait | Its function | Use |
| --- | --- | --- |
| `ပြောင်းလဲနိုင်သော`, written `B အဖြစ် ပြောင်းလဲနိုင်သော` | `ပြောင်းလဲ`, which takes an `A` and returns a `B` | a conversion that cannot fail |
| `စမ်းသပ်ပြောင်းလဲနိုင်သော`, written `B အဖြစ် E အမှားဖြင့် စမ်းသပ်ပြောင်းလဲနိုင်သော` | `စမ်းသပ်ပြောင်းလဲ`, which takes an `A` and returns `B သို့မဟုတ် အမှားအယွင်းအနေဖြင့် E ရှိနိုင်သော ဒေတာဆဲလ်` | a conversion that can fail with an error of data kind `E` |

To let a data kind convert, we write the function of the trait for the pair, as a plain
function or in a [trait block](traits.md#trait-blocks). The compiler looks for it in the
file of the conversion and in the files of the two data kinds, so a conversion to a struct
of another file needs no import of the function. The conversion is then a call of that
function:

- `ပြောင်းလဲသော ရလဒ်` calls `ပြောင်းလဲ`, or `စမ်းသပ်ပြောင်းလဲ` when the pair has no
  `ပြောင်းလဲ`, and then its value is the whole result.
- `ပြောင်းလဲသော ရလဒ်ကောင်း` calls `စမ်းသပ်ပြောင်းလဲ` and passes a failure on, the same as
  `သည့် ရလဒ်ကောင်း` does for a call.
- Two functions of one name can share a parameter and differ in their results
  ([Overloads that differ in the result](functions-overloads.md#overloads-that-differ-in-the-result)),
  so one data kind can have a `ပြောင်းလဲ` for each data kind that it converts to.

```zaka
မီတာ ဟူသော ဒေတာအစုအဝေးတွင် အရှည် ဟူသော ဒဿမကိန်း တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ခု ဖြစ်သည်။

အရှည်အမှားအယွင်း ဟူသော ဒေတာအမျိုးအစားကွဲတွင်
ဗလာ နှင့်
သုညအောက် တို့ပါဝင်သည်။

ပြောင်းလဲ ဟူသော လုပ်ဆောင်ချက် သည်
မ ဟူသော မီတာ တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ ဒဿမကိန်း တစ်ခု ကို ပြန်ပေးမည်။
မ ၏ အရှည် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

စမ်းသပ်ပြောင်းလဲ ဟူသော လုပ်ဆောင်ချက် သည်
ဒ ဟူသော ဒဿမကိန်း တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ မီတာ သို့မဟုတ် အမှားအယွင်းအနေဖြင့် အရှည်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ ဒ < ၀.၀ လျှင် အရှည်အမှားအယွင်းများထဲမှ သုညအောက် ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
မီတာ(အရှည်: ဒ) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
မီတာ(အရှည်: ၂.၅) ကို ဒဿမကိန်း တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
၃.၀ ကို မီတာ တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
-၁.၀ ကို မီတာ တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂.၅
ရလဒ်ကောင်း(မီတာ(အရှည်: ၃.၀))
အမှားအယွင်း(အရှည်အမှားအယွင်းများထဲမှ သုညအောက်)
```

A function whose name ends in `အဖြစ်ပြောင်း` is a plain function: the conversion does
not look at names.

| Case | Code |
| --- | --- |
| `ပြောင်းလဲသော ရလဒ်ကောင်း` on a pair that cannot fail: a pair of the table, a pair with a `ပြောင်းလဲ` alone, or a conversion to text | ZK0444 |
| no pair in the table, no function of the two traits, and no conversion to text | ZK0408 |

## Conversions to text

Each value converts to `စာသား` with no function of its own: the result is the text that
`ပြသမည်` prints for the value, and a struct with `စာသားအဖြစ်ပြောင်း` gives the text of that
function. A `ပြောင်းလဲ` that returns `စာသား` for the data kind replaces this rule. A handle of
a data kind from outside has no text, so its conversion is ZK0408.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
(၄၂ ကို စာသား တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်) + "!" ဟု ပြသမည်။
'က' ကို စာသား တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
မှန် ကို စာသား တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၂!
က
မှန်
```

## Conversions in the stdlib

| Module | Function | Converts |
| --- | --- | --- |
| `"စံကား/စာသား"` | `စမ်းသပ်ပြောင်းလဲ` | text to an integer: an optional `-`, then the digits `0`-`9` or `၀`-`၉` |
| `"စံကား/လမ်းကြောင်း"` | `ပြောင်းလဲ` | text to a path |

A path converts to text through its `စာသားအဖြစ်ပြောင်း`, as each struct does. Text and the
integers are declared in no file, so to read an integer from text we import
`စမ်းသပ်ပြောင်းလဲ` from `"စံကား/စာသား"`. Its failure is a member of
`ကိန်းဂဏန်းဖတ်ခြင်းအမှားအယွင်း`: `ဗလာစာသား` (blank text), `ကိန်းဂဏန်းမဟုတ်သောစာလုံးပါခြင်း`
(a character that is not a digit) or `အကန့်အသတ်ကျော်ခြင်း` (a number past the largest
`ကိန်းပြည့်`). A kind alias converts as the data kind that it names.

```zaka
"စံကား/စာသား" မှ စမ်းသပ်ပြောင်းလဲ နှင့် ကိန်းဂဏန်းဖတ်ခြင်းအမှားအယွင်း တို့ကို ထည့်သွင်းမည်။

နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် သည် စ ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ကိန်းဂဏန်းဖတ်ခြင်းအမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် န သည် စ ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်ကောင်း ဖြစ်ပါစေ။
န * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"-၁၂" ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
"12a" ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
နှစ်ဆ("၂၁") ကို ပြသမည်။
နှစ်ဆ("") ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရလဒ်ကောင်း(-၁၂)
အမှားအယွင်း(ကိန်းဂဏန်းဖတ်ခြင်းအမှားအယွင်းများထဲမှ ကိန်းဂဏန်းမဟုတ်သောစာလုံးပါခြင်း)
ရလဒ်ကောင်း(၄၂)
အမှားအယွင်း(ကိန်းဂဏန်းဖတ်ခြင်းအမှားအယွင်းများထဲမှ ဗလာစာသား)
```

## What happens to the value

A conversion is the one place where Zaka changes a value without an error, even when the
new data kind cannot contain the old value. Each pair has one fixed rule:

| From | To | Rule |
| --- | --- | --- |
| integer | integer | the low bits are kept, and the sign is read again in the new data kind |
| integer | decimal number | the nearest value; big integers lose their low digits |
| decimal number | integer | the fraction is dropped toward zero; a value outside the range becomes the smallest or the largest of the data kind; `NaN` becomes `၀` |
| decimal number | decimal number | the nearest value; one that is too large becomes `inf` |
| `စာလုံး` | integer | the code point, cut to the width as an integer is |
| integer | `စာလုံး` | the character with that code point; a value that is not a Unicode scalar value (negative, a surrogate, above `10FFFF`) gives `U+FFFD` |
| `မှန်မှားတန်ဖိုး` | integer | `၁` for `မှန်` and `၀` for `မမှန်` |

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၃၀၀ ကို လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
-၁ ကို လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
၂၀၀ ကို လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ၈ ဘစ် ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
-၁ ကို ၈ ဘစ် ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို လက္ခဏာမဲ့ ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
၃.၉ ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
-၃.၉ ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
-၅.၅ ကို လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
1e30 ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၄
၂၅၅
-၅၆
၄၂၉၄၉၆၇၂၉၅
၃
-၃
၀
၉၂၂၃၃၇၂၀၃၆၈၅၄၇၇၅၈၀၇
```

Going from an integer to a wider data kind keeps the value: a signed integer is extended
with its sign, and an unsigned one with zeros. Going to a narrower data kind cuts the high
bits, which is the reason `၃၀၀` becomes `၄၄` (300 - 256) in a
`လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်`.

The next example converts characters, true/false values and decimal numbers of other
widths:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၆၅ ကို စာလုံး တစ်လုံး အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
'က' ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
'က' ကို ၈ ဘစ် ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
မမှန် ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
၁၆၇၇၇၂၁၇ ကို ၃၂ ဘစ် ဒဿမကိန်း တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
A
၄၀၉၆
၀
၀
၁၆၇၇၇၂၁၆.၀
```

## Widening: the change that needs no words

A value goes into a place of another data kind with no conversion when the place's data
kind can contain each value of the value's data kind. The places are the initial value of
a declaration, the new value of an assignment, an argument, a returned value, and the
operands of an operator ([the next section](#mixing-data-kinds-in-an-operation)).

| From | To | When |
| --- | --- | --- |
| integer | integer | the target is wider and no sign is lost: signed to wider signed, unsigned to wider unsigned, unsigned to wider signed |
| integer | decimal number | the integer's width is at most the bits of the float's significand: 24 for `f32`, 53 for `f64` |
| decimal number | decimal number | the target is wider |

So `u8` goes into `i16`, `i16` into `f32`, `i32` into `f64`, and `f32` into `f64`. These
pairs do not widen:

- signed to unsigned, and a narrower target;
- `isize` or `usize` (64 bits) into `f64`, and `i32` into `f32`;
- `isize` into `i64`, because the two data kinds have the same width but are different;
- a `စာလုံး` or a `မှန်မှားတန်ဖိုး` into another data kind;
- a pointer into a data kind other than the pointer kinds that [Unsafe Zaka](unsafe.md)
  describes.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀၀ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ၁၆ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် ၃၂ ဘစ် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ခ ဖြစ်ပါစေ။
ကိန်းရှင် ဃ သည် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဂ ဖြစ်ပါစေ။
ခ ကို ပြသမည်။
ဂ ကို ပြသမည်။
ဃ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၀၀
၂၀၀.၀
၂၀၀.၀
```

A widening that is missing is ZK0401, which names both data kinds and suggests the
conversion:

```zaka-error ZK0401
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စုစုပေါင်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀ ဖြစ်ပါစေ။
ကိန်းရှင် ပျမ်းမျှ သည် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် စုစုပေါင်း ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0401]: expected f64, found isize
 = help: convert the value with "ကို <kind> တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်"
```

## Mixing data kinds in an operation

An operator with two operands joins their data kinds as follows ([Expressions](expressions.md)):

| Operands | Result |
| --- | --- |
| both of one data kind | that data kind |
| one data kind widens to the other | the narrower operand is widened first, and the result has the wider data kind: `u8 + i16` is an `i16` |
| neither widens to the other, as with a signed and an unsigned integer of one width | no common data kind, and the program does not compile (ZK0404) |

A literal without a data kind after it gets the data kind of the other operand, so
`က + ၁` with a `u8` named `က` adds a `u8`. When both sides are literals, a fraction or an
exponent in either one makes both of them decimal numbers, so `၃ + ၀.၅` is `၃.၅`. A literal
that does not fit the data kind it must get is ZK0406.

```zaka-error ZK0404
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
က + ခ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0404]: cannot combine i8 and u8 without a conversion
```

```zaka-error ZK0406
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
က + ၃၀၀ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0406]: the literal 300 does not fit i8
```

To join two data kinds that do not widen, we convert one side:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀၀ ဖြစ်ပါစေ။
က နှင့် ခ ကို ၁၆ ဘစ် ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ၏ ပေါင်းလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၀၅
```

## Errors in one table

| Code | Raised when |
| --- | --- |
| ZK0401 | a value of the wrong data kind goes into a variable, a parameter or a result |
| ZK0403 | an operator gets an operand that it does not work on, such as `မှန် + ၁` |
| ZK0404 | the two operands of an operator have no common data kind |
| ZK0406 | a literal does not fit the data kind that its place gives it |
| ZK0408 | a conversion between two data kinds that cannot convert, with no pair in the table and no function of the conversion traits |
| ZK0444 | `ပြောင်းလဲသော ရလဒ်ကောင်း` on a conversion that cannot fail |
