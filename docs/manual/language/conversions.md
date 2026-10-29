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
<value>  ကို  <kind>  အဖြစ်  ပြောင်းလဲသော ရလဒ်
 operand      target kind        the verb
```

It reads "the result of changing the value into the kind". The verb `ပြောင်းလဲသော ရလဒ်`
closes a [clause](expressions.md): the value takes `ကို` and the kind takes `အဖြစ်`. The
result has the target kind, and it can take a particle of its own, so a conversion can
be the first part of a longer phrase.

```zaka
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
က ကို ဒဿမကိန်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
က ကို ဒဿမကိန်း အဖြစ် ပြောင်းလဲသော ရလဒ် နှင့် ၂.၅ ၏ ပေါင်းလဒ် ကို ပြသပါ။
(က ကို ဒဿမကိန်း အဖြစ် ပြောင်းလဲသော ရလဒ်) / ၄ ကို ပြသပါ။
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
`n != ၀`. Conversions between addresses and integers, and between pointer kinds, have
rules of their own, in [Pointers](pointers.md). A list never converts.

```zaka-error ZK0408
၁ ကို မှန်မှားတန်ဖိုး အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
```

```text
error[ZK0408]: cannot convert isize to bool
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
၃၀၀ ကို လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
-၁ ကို လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
၂၀၀ ကို လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ၈ ဘစ် ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
-၁ ကို ၈ ဘစ် ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို လက္ခဏာမဲ့ ၃၂ ဘစ် ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
၃.၉ ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
-၃.၉ ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
-၅.၅ ကို လက္ခဏာမဲ့ ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
1e30 ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
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
၆၅ ကို စာလုံး အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
'က' ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
'က' ကို ၈ ဘစ် ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
မမှန် ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
၁၆၇၇၇၂၁၇ ကို ၃၂ ဘစ် ဒဿမကိန်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
၁.၁ ကို ၁၆ ဘစ် ဒဿမကိန်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
```

```text
A
၄၀၉၆
၀
၀
၁၆၇၇၇၂၁၆.၀
၁.၀၉၉၆
```

## Widening: the change that needs no words

A value goes into a place of another kind with no conversion just when the place's
kind holds each value of the value's kind. The places are the initial value of a
declaration, the new value of an assignment, an argument, a returned value, and the
operands of an operator (the next section). The cases:

| From | To | When |
| --- | --- | --- |
| integer | integer | the target is wider and no sign is lost: signed to wider signed, unsigned to wider unsigned, unsigned to wider signed |
| integer | decimal number | the integer's width is at most the bits of the float's significand: 11 for `f16`, 24 for `f32`, 53 for `f64`, 113 for `f128` |
| decimal number | decimal number | the target is wider |

So `u8` goes into `i16`, `i16` into `f32`, `i32` into `f64`, and `f32` into `f64`.
These do not widen: signed to unsigned, a narrower target, `isize` or `usize` (64 bits)
into `f64`, `i32` into `f32`, and `isize` into `i64` (the two kinds are the same width
but differ). A `စာလုံး` and a `မှန်မှားတန်ဖိုး` never widen. An address widens to the
address kinds described in [Pointers](pointers.md) and to nothing else.

```zaka
ကိန်းရှင် က သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀၀ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ၁၆ ဘစ် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် ၃၂ ဘစ် ဒဿမကိန်း ဖြစ်၍ တန်ဖိုးသည် ခ ဖြစ်ပါစေ။
ကိန်းရှင် ဃ သည် ဒဿမကိန်း ဖြစ်၍ တန်ဖိုးသည် ဂ ဖြစ်ပါစေ။
ခ ကို ပြသပါ။
ဂ ကို ပြသပါ။
ဃ ကို ပြသပါ။
```

```text
၂၀၀
၂၀၀.၀
၂၀၀.၀
```

A widening that is missing is ZK0401, which names both kinds and suggests the
conversion:

```zaka-error ZK0401
ကိန်းရှင် စုစုပေါင်း သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၀ ဖြစ်ပါစေ။
ကိန်းရှင် ပျမ်းမျှ သည် ဒဿမကိန်း ဖြစ်၍ တန်ဖိုးသည် စုစုပေါင်း ဖြစ်ပါစေ။
```

```text
error[ZK0401]: expected f64, found isize
 = help: convert the value with "ကို <kind> အဖြစ် ပြောင်းလဲသော ရလဒ်"
```

## Mixing kinds in an operation

An operator with two operands joins them as follows (see [Expressions](expressions.md)).

- If both have one kind, the result has that kind.
- If one kind widens to the other, the narrower operand is widened first and the result
  has the wider kind: `u8 + i16` is an `i16`.
- If neither widens to the other, there is no common kind and the program does not
  compile (ZK0404). A signed and an unsigned integer of one width are such a pair.

A literal without a kind prefix takes the kind of the other operand, so `က + ၁` with a
`u8` named `က` adds a `u8`. When both sides are literals, a fraction or an exponent
in either makes both of them decimal numbers: `၃ + ၀.၅` is `၃.၅`. A literal that does not
fit the kind it must take is ZK0406.

```zaka-error ZK0404
ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
က + ခ ကို ပြသပါ။
```

```text
error[ZK0404]: cannot combine i8 and u8 without a conversion
```

```zaka-error ZK0406
ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
က + ၃၀၀ ကို ပြသပါ။
```

```text
error[ZK0406]: the literal 300 does not fit i8
```

Convert one side to join two kinds that do not widen:

```zaka
ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀၀ ဖြစ်ပါစေ။
က နှင့် ခ ကို ၁၆ ဘစ် ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ၏ ပေါင်းလဒ် ကို ပြသပါ။
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
