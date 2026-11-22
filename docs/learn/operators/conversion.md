---
title: Converting between data kinds
description: Turn a value of one data kind into another with ပြောင်းလဲသော ရလဒ်.
---

# Converting between data kinds

Sometimes a value comes in one data kind (type) while the program needs another, for
example an integer count that we want to divide as a decimal number, or a character whose
Unicode number we want. A conversion (cast) returns the same value in the new data kind,
and Zaka writes it as a Burmese phrase, with no sign form.

## You will learn

- how to convert a value to another data kind
- which conversions can lose part of the value
- which conversions Zaka refuses
- when Zaka converts a value on its own, and when we must write the conversion
- how to turn a value into text

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃၇ ဖြစ်ပါစေ။
ကိန်းရှင် အမှတ် သည် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၉.၇၅ ဖြစ်ပါစေ။
ကိန်းရှင် အက္ခရာ သည် စာလုံး တစ်လုံး ဖြစ်၍ တန်ဖိုးသည် 'A' ဖြစ်ပါစေ။

အသက် ကို ဒဿမကိန်း တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။ // 37 as a decimal
အမှတ် ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။ // 9.75 as an integer
အက္ခရာ ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။ // 'A' as its number
၆၆ ကို စာလုံး တစ်လုံး အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။ // 66 as a character
မှန် ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။ // true as a number
```

```text
၃၇.၀
၉
၆၅
B
၁
```

To convert a value, we use this pattern:

```text
<value> ကို <kind> တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်
```

| Zaka | Reads as |
| --- | --- |
| `<value> ကို` | "`<value>`", the value to convert |
| `<kind> တစ်ခု` | "one `<kind>`", the data kind we want |
| `အဖြစ်` | "as", "into" |
| `ပြောင်းလဲသော ရလဒ်` | "the result of changing" |

So the whole phrase means "the result of changing the value into the data kind". We write
the data kind with its measure word (classifier), the same as in a declaration, so a
for a character we write `တစ်လုံး`: `၆၆ ကို စာလုံး တစ်လုံး အဖြစ် ပြောင်းလဲသော ရလဒ်`.

| From | To | Example | Result |
| --- | --- | --- | --- |
| integer | decimal number | `၃၇` | `၃၇.၀` |
| decimal number | integer | `၉.၇၅` | `၉` |
| character | integer | `'A'` | `၆၅` |
| integer | character | `၆၆` | `B` |
| true/false value | integer | `မှန်` | `၁` |

## Converting can lose data

Zaka asks us to write each conversion out, because a conversion can change the value:

- A decimal number becomes an integer with the fraction dropped toward zero, so `၉.၇၅`
  becomes `၉` and `-၂.၇` becomes `-၂`.
- A number too big for a smaller integer data kind wraps around, so 300 as a
  `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်` is 44.
- A negative number in an unsigned data kind wraps too, so `-၁` becomes 255.

```zaka
-၂.၇ ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
၃၀၀ ကို လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
-၁ ကို လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
```

```text
-၂
၄၄
၂၅၅
```

## Conversions that Zaka refuses

Some conversions make no sense, so Zaka refuses them. A number cannot become a true/false
value, for example, and the compiler stops with an error:

```zaka-error ZK0408
၁ ကို မှန်မှားတန်ဖိုး တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
```

```text
error[ZK0408]: cannot convert isize to bool
```

To turn a number into a true/false value, we compare it instead: `န != ၀`.

## Converting on its own

When nothing can be lost, Zaka converts the value on its own, with no phrase. This
happens when a smaller data kind goes into a bigger one, such as an `၈ ဘစ် ကိန်းပြည့်`
into a `ကိန်းပြည့်` or into a decimal number, and it is called *widening*.

```zaka
ကိန်းရှင် ဘိုက် သည် ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀ ဖြစ်ပါစေ။ // octet
ကိန်းရှင် ကျယ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဘိုက် ဖြစ်ပါစေ။ // 8-bit into 64-bit: automatic
ကိန်းရှင် ဒဿမ သည် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဘိုက် ဖြစ်ပါစေ။ // 8-bit into a decimal: automatic
ကျယ် ကို ပြသမည်။
ဒဿမ ကို ပြသမည်။
```

```text
၁၀
၁၀.၀
```

A plain `ကိန်းပြည့်` does **not** go into a decimal number on its own. It has 64 bits,
and a decimal number cannot keep all 64-bit integers without loss, so the compiler stops
with an error:

```zaka-error ZK0401
ကိန်းရှင် စုစုပေါင်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀ ဖြစ်ပါစေ။
ကိန်းရှင် ဒဿမ သည် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် စုစုပေါင်း ဖြစ်ပါစေ။
```

```text
error[ZK0401]: expected f64, found isize
```

## A conversion inside a sum

```text
(<value> ကို <kind> တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်) / <value>
```

To work out an average that keeps its fraction, we convert both integers before we divide
them. A conversion is a word phrase, so inside a sum with signs we put it in parentheses:

```zaka
ကိန်းရှင် စုစုပေါင်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀ ဖြစ်ပါစေ။ // total
ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၄ ဖြစ်ပါစေ။ // count

// 10 / 4 with integers drops the fraction
စုစုပေါင်း / အရေအတွက် ကို ပြသမည်။

// convert both to decimals first
(စုစုပေါင်း ကို ဒဿမကိန်း တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်) / (အရေအတွက် ကို ဒဿမကိန်း တစ်ခု အဖြစ်
ပြောင်းလဲသော ရလဒ်) ကို ပြသမည်။
```

```text
၂
၂.၅
```

The next page, [Grouping and order](grouping.md), explains the reason for the parentheses.

## Converting to text

```text
<value> ကို စာသား တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်
```

Each value converts to text, and the text is what `ပြသမည်` would print for it. This helps
when we want to join a number to other text with `+`:

```zaka
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၉၅ ဖြစ်ပါစေ။ // score
"ရမှတ်: " + (အမှတ် ကို စာသား တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်) ဟု ပြသမည်။
```

```text
ရမှတ်: ၉၅
```

## Details

- **Which pairs convert.** Integers, decimal numbers, characters and true/false values
  convert between some pairs and not others, and the full table is on [Which pairs
  convert](../../manual/language/conversions.md#which-pairs-convert).
- **The exact result.** Each pair has a fixed rule for values that do not fit, such as
  a decimal number too large for an integer ([What happens to the
  value](../../manual/language/conversions.md#what-happens-to-the-value)).
- **Which data kinds widen.** The full list of conversions that Zaka makes on its own is
  on [Widening](../../manual/language/conversions.md#widening-the-change-that-needs-no-words).
- **Text to a number.** Text such as `"၄၂"` converts to a number or a character through a
  function of the stdlib, and the conversion can fail ([Conversions in the
  stdlib](../../manual/language/conversions.md#conversions-in-the-stdlib)).
- **Our own data kinds.** A struct converts to another data kind, and back, through the
  functions of the conversion traits, `ပြောင်းလဲ` and `စမ်းသပ်ပြောင်းလဲ` ([Conversions
  through traits](../../manual/language/conversions.md#conversions-through-traits)).
- **Integers that must fit.** `ပြောင်းလဲသော ရလဒ်ကောင်း` between two integers fails instead of
  wrapping when the value does not fit ([Integers that must
  fit](../../manual/language/conversions.md#integers-that-must-fit)).
- **Same data kind.** A conversion to the data kind that the value has changes nothing.

## Recap

- We write `<value> ကို <kind> တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်` to convert a value.
- A conversion can drop a fraction or wrap a number around.
- Zaka refuses conversions that make no sense, such as a number to a true/false value.
- Each value converts to the text that `ပြသမည်` prints for it.
- A smaller data kind widens into a bigger one on its own, but a plain `ကိန်းပြည့်` needs
  a conversion to become a decimal number.

## Hands on

- Print the Unicode number of `'က'`.
- Print the character whose number is 4160.
- Work out the average of 7 and 8 as a decimal number.

Next: [Grouping and order](grouping.md).
