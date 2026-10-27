---
title: Converting between kinds
description: Turn a value of one kind into another with ပြောင်းလဲသော ရလဒ်.
---

# Converting between kinds

Sometimes a value comes in one kind but the program needs another: an integer count to
divide as a decimal, a decimal score to round down to a whole number, or a character's
Unicode number. *Converting* gives the value in the new kind.

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၃၇ ဖြစ်ပါစေ။
ကိန်းရှင် အမှတ် သည် ဒဿမကိန်း ဖြစ်၍ တန်ဖိုးသည် ၉.၇၅ ဖြစ်ပါစေ။
ကိန်းရှင် အက္ခရာ သည် စာလုံး ဖြစ်၍ တန်ဖိုးသည် 'A' ဖြစ်ပါစေ။

အသက် ကို ဒဿမကိန်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။   // 37 as a decimal
အမှတ် ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။   // 9.75 as an integer
အက္ခရာ ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။   // 'A' as its number
၆၆ ကို စာလုံး အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။         // 66 as a character
မှန် ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။     // true as a number
```

```text
၃၇.၀
၉
၆၅
B
၁
```

The phrase reads "the result of changing *x* into *kind*":

| Zaka | Reads as |
| --- | --- |
| `အသက်` | "age" — the value to convert |
| `ကို` | (object) |
| `ဒဿမကိန်း` | the target kind |
| `အဖြစ်` | "as", "into" |
| `ပြောင်းလဲသော ရလဒ်` | "the changed result" |

## Converting can lose data

Each conversion has to be written out, because it can change the value:

- Decimal to integer drops the fraction, toward zero: `၉.၇၅` becomes `၉`, `-၂.၇` becomes `-၂`.
- A number too big for a smaller kind wraps around: 300 as `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်` is 44.
- A negative number as an unsigned kind wraps too: `-၁` becomes 255.

```zaka
-၂.၇ ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
၃၀၀ ကို လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
-၁ ကို လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
```

```text
-၂
၄၄
၂၅၅
```

Some conversions make no sense and are refused. A number cannot become true or false:

```zaka-error ZK0408
၁ ကို မှန်မှားတန်ဖိုး အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
```

```text
error[ZK0408]: cannot convert isize to bool
```

## Automatic conversion

When nothing can be lost, Zaka converts on its own. A smaller kind goes into a bigger
one: an `၈ ဘစ် ကိန်းပြည့်` into a `ကိန်းပြည့်`, or into a decimal
number. These are called *widening* conversions.

```zaka
ကိန်းရှင် ဘိုက် သည် ၈ ဘစ် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၀ ဖြစ်ပါစေ။        // octet
ကိန်းရှင် ကျယ် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ဘိုက် ဖြစ်ပါစေ။           // 8-bit into 64-bit: automatic
ကိန်းရှင် ဒဿမ သည် ဒဿမကိန်း ဖြစ်၍ တန်ဖိုးသည် ဘိုက် ဖြစ်ပါစေ။          // 8-bit into a decimal: automatic
ကျယ် ကို ပြသပါ။
ဒဿမ ကို ပြသပါ။
```

```text
၁၀
၁၀.၀
```

A plain `ကိန်းပြည့်` does **not** go into a decimal on its own. It has 64 bits, and a
decimal cannot hold all 64-bit integers without loss:

```zaka-error ZK0401
ကိန်းရှင် စုစုပေါင်း သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၀ ဖြစ်ပါစေ။
ကိန်းရှင် ဒဿမ သည် ဒဿမကိန်း ဖြစ်၍ တန်ဖိုးသည် စုစုပေါင်း ဖြစ်ပါစေ။
```

```text
error[ZK0401]: expected f64, found isize
```

Write the conversion out. Here is an average that keeps its fraction:

```zaka
ကိန်းရှင် စုစုပေါင်း သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၀ ဖြစ်ပါစေ။     // total
ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၄ ဖြစ်ပါစေ။      // count

// 10 / 4 with integers drops the fraction
စုစုပေါင်း / အရေအတွက် ကို ပြသပါ။

// convert both to decimals first
(စုစုပေါင်း ကို ဒဿမကိန်း အဖြစ် ပြောင်းလဲသော ရလဒ်) / (အရေအတွက် ကို ဒဿမကိန်း အဖြစ် ပြောင်းလဲသော ရလဒ်) ကို ပြသပါ။
```

```text
၂
၂.၅
```

The parentheses let a conversion phrase sit inside a `/` sum. The next page explains
the reason.

## Hands on

- Print the Unicode number of `'က'`.
- Print the character whose number is 4160.

Next: [Grouping and order](grouping.md).
