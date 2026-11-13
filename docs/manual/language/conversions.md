---
title: Conversions
description: Turn a value of one kind into another - the conversion expression, which pairs convert, what happens to the value, and the widening that needs no words.
---

# Conversions

A value of one kind can take the place of another kind through two means. A *conversion*
is written out: the program names the kind it wants and the value is changed to fit. A
*widening* happens with no words, but just where the new kind holds each value of the old
one. Each other change of kind is an error.

The [Learn chapter](../../learn/operators/conversion.md) shows conversions with gentle
examples. This page gives the rules.

## The conversion expression

```text
<value>  ကို  <kind> တစ်<word>  အဖြစ်  ပြောင်းလဲသော ရလဒ်
 operand      target kind, counted     the verb
```

It reads "the result of changing the value into the kind". The verb `ပြောင်းလဲသော ရလဒ်`
closes a [clause](expressions.md): the value takes `ကို` and the kind, with its
[measure word](kinds.md#measure-words), takes `အဖြစ်`; a kind with no word there is ZK0234. The
result has the target kind, and it can take a particle of its own, so a conversion can
be the first part of a longer phrase.

A part of a [whole result](error-handling.md#reading-a-whole-result) of a function that
can fail is no conversion: `x ထဲမှ ဒေတာ` gives its good result, and `x ထဲမှ အမှားအယွင်း ဒေတာ` its
error value, as [Errors and results](error-handling.md) states.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
က ကို ဒဿမကိန်း တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
က ကို ဒဿမကိန်း တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် နှင့် ၂.၅ ကို ပေါင်းသည့်ရလဒ် ကို ပြသမည်။
(က ကို ဒဿမကိန်း တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်) / ၄ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂.၀
၄.၅
၀.၅
```

A conversion is a clause, so inside a sum written with signs it needs parentheses, as the
third line shows. A conversion to the kind that the value has changes nothing.
The compiler works out a conversion of a constant.

## Which pairs convert

A conversion is allowed between the pairs below, and in no other pair (ZK0408):

| From \ To | integer | decimal number | `စာလုံး` | `မှန်မှားတန်ဖိုး` |
| --- | --- | --- | --- | --- |
| integer | allowed | allowed | allowed | refused |
| decimal number | allowed | allowed | refused | refused |
| `စာလုံး` | allowed | refused | allowed | refused |
| `မှန်မှားတန်ဖိုး` | allowed | refused | refused | allowed |

An integer is never turned into a true/false value. Test it with a comparison instead:
`n != ၀`. Conversions between pointers and integers, between pointer kinds, and between
a shared object and a pointer, have rules of their own and need unsafe code (ZK0468
outside it), in [Unsafe Zaka](unsafe.md). The conversion verb takes one list: a plain
list of `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်` converts to text, and the VM checks that its u8s are
UTF-8, stopping the program when the u8s are not ([Text](text.md)). Each other list is
ZK0408; a counted list becomes a plain list on its own where a value goes into a place, as
[Lists](lists.md#forms) states. A member of an enumeration converts to its value
kind, which gives its value, and a member whose value is a number, a character or a
true/false value converts to each kind its value converts to, as
[Enumerations](enums.md#values-of-an-enumeration) states; nothing converts to a member.

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

## Conversions through functions

```text
<value> ကို <kind> တစ်<word> အဖြစ် ပြောင်းလဲသော ရလဒ်
<value> ကို <kind> တစ်<word> အဖြစ် ပြောင်းလဲသော ရလဒ်ကောင်း
```

When the table above has no pair for two kinds, a conversion goes through a function the
file sees: one whose name ends in `အဖြစ်ပြောင်း`, that takes one value of the source kind,
and gives the target kind, or a result whose good side is the target kind. The
stdlib names such a function `<K>အဖြစ်ပြောင်း`, with the word of the target kind
for `K`. The conversion is then the call of that function: its value is the result of the
call, and `ပြောင်းလဲသော ရလဒ်ကောင်း` passes a failure on, as `သည့် ရလဒ်ကောင်း` does for a
call. `ပြောင်းလဲသော ရလဒ်ကောင်း` on a pair the table converts is ZK0444, since that conversion
cannot fail. With no such function, the conversion is ZK0408, as before.

- `"စံကား/စာသား"` gives `ကိန်းပြည့်အဖြစ်ပြောင်း`, which reads text as an integer: an optional
  `-`, then the digits `0`-`9` or `၀`-`၉`. Its failure is a member of
  `ကိန်းဂဏန်းဖတ်ခြင်းအမှားအယွင်း`: `ဗလာစာသား` (blank text),
  `ကိန်းဂဏန်းမဟုတ်သောစာလုံးပါခြင်း` (a character that is no digit) or
  `အကန့်အသတ်ကျော်ခြင်း` (a number past the largest `ကိန်းပြည့်`).
- `"စံကား/လမ်းကြောင်း"` gives `ဖိုင်လမ်းကြောင်းအဖြစ်ပြောင်း`, text to a path, and
  `စာသားအဖြစ်ပြောင်း`, a path to text.
- A struct converts through a function of its own of the same shape, and a kind alias
  converts as the kind it names.

```zaka
"စံကား/စာသား" မှ ကိန်းပြည့်အဖြစ်ပြောင်း နှင့် ကိန်းဂဏန်းဖတ်ခြင်းအမှားအယွင်း တို့ကို ထည့်သွင်းမည်။

လုပ်ဆောင်ချက် နှစ်ဆ သည် စ ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
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

A conversion is the one place where Zaka changes a value without an error, even if the
new kind cannot hold the old value. Each pair has one fixed rule.

| From | To | Rule |
| --- | --- | --- |
| integer | integer | the low bits are kept; the sign is read again in the new kind |
| integer | decimal number | the nearest value; big integers lose their low digits |
| decimal number | integer | the fraction is dropped toward zero; a value outside the range becomes the smallest or the largest of the kind; `NaN` becomes `၀` |
| decimal number | decimal number | the nearest value; one too large becomes `inf` |
| `စာလုံး` | integer | the code point, cut to the width as an integer is |
| integer | `စာလုံး` | the character with that code point; a value that is no Unicode scalar value (negative, a surrogate, above `10FFFF`) gives `U+FFFD` |
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

Going from an integer to a wider kind keeps the value: a signed integer is extended with
its sign, an unsigned one with zeros. Going to a narrower kind cuts the high bits, which
gives `၃၀၀` the result `၄၄` (300 - 256) in a `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်`.

Characters, true/false values and decimal numbers of other widths:

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

A value goes into a place of another kind with no conversion just when the place's
kind holds each value of the value's kind. The places are the initial value of a
declaration, the new value of an assignment, an argument, a returned value, and the
operands of an operator (the next section). The cases:

| From | To | When |
| --- | --- | --- |
| integer | integer | the target is wider and no sign is lost: signed to wider signed, unsigned to wider unsigned, unsigned to wider signed |
| integer | decimal number | the integer's width is at most the bits of the float's significand: 24 for `f32`, 53 for `f64` |
| decimal number | decimal number | the target is wider |

So `u8` goes into `i16`, `i16` into `f32`, `i32` into `f64`, and `f32` into `f64`.
These do not widen: signed to unsigned, a narrower target, `isize` or `usize` (64 bits)
into `f64`, `i32` into `f32`, and `isize` into `i64` (the two kinds are the same width
but differ). A `စာလုံး` and a `မှန်မှားတန်ဖိုး` never widen. A pointer widens to the
pointer kinds described in [Unsafe Zaka](unsafe.md) and to nothing else.

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

A widening that is missing is ZK0401, which names both kinds and suggests the
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

## Mixing kinds in an operation

An operator with two operands joins them as follows (see [Expressions](expressions.md)).

- If both have one kind, the result has that kind.
- If one kind widens to the other, the narrower operand is widened first and the result
  has the wider kind: `u8 + i16` is an `i16`.
- If neither widens to the other, there is no common kind and the program does not
  compile (ZK0404). A signed and an unsigned integer of one width are such a pair.

A literal without a kind after it takes the kind of the other operand, so `က + ၁` with a
`u8` named `က` adds a `u8`. When both sides are literals, a fraction or an exponent
in either makes both of them decimal numbers: `၃ + ၀.၅` is `၃.၅`. A literal that does not
fit the kind it must take is ZK0406.

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

Convert one side to join two kinds that do not widen:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀၀ ဖြစ်ပါစေ။
က နှင့် ခ ကို ၁၆ ဘစ် ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပေါင်းသည့်ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၀၅
```

## Errors in one table

| Code | Raised when |
| --- | --- |
| ZK0401 | a value of the wrong kind goes into a variable, parameter or result |
| ZK0403 | an operator gets an operand it does not take, such as `မှန် + ၁` |
| ZK0404 | the two operands of an operator have no common kind |
| ZK0406 | a literal does not fit the kind that its place gives it |
| ZK0408 | a conversion between two kinds that cannot convert |
