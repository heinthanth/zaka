---
title: Bitwise operators
description: Work on the single bits of an integer.
---

# Bitwise operators

Inside the computer, an integer is a row of bits, each `0` or `1`, so the number 202 in 8
bits is `11001010`. Bitwise operators work on these bits one position at a time. A
beginner seldom needs them, but these operators are useful for flags, colours and low-level data, so
feel free to skip this page and come back to it later.

## You will learn

- the six bitwise operators, in signs and in words
- how to shift the bits of an integer to the left or to the right
- how far a shift can go
- how to keep some bits and clear the rest with a mask

| Sign | Words | Reads as | Each result bit is 1 when |
| --- | --- | --- | --- |
| `a & b` | `a နှင့် b ၏ ဘစ်အလိုက် ဆုံလဒ်` | "bitwise meeting" | both bits are 1 |
| `a \| b` | `a နှင့် b ၏ ဘစ်အလိုက် ပေါင်းစုလဒ်` | "bitwise union" | either bit is 1 |
| `a ^ b` | `a နှင့် b ၏ ဘစ်အလိုက် ခြားနားလဒ်` | "bitwise difference" | the two bits differ |
| `~a` | `a ၏ ဘစ်အလိုက် ပြောင်းပြန်ပြောင်းလဒ်` | "bitwise flip" | the bit was 0 |
| `a << n` | `a ကို ဘယ်သို့ n ဘစ် ရွေ့သောရလဒ်` | "a moved n bits to the left" | (the bits move left) |
| `a >> n` | `a ကို ညာသို့ n ဘစ် ရွေ့သောရလဒ်` | "a moved n bits to the right" | (the bits move right) |

```zaka
ကိန်းရှင် က သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အခြေ ၂ ဖြင့် ၁၁၀၀၁၀၁၀ ဖြစ်ပါစေ။ // 202
ကိန်းရှင် ခ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အခြေ ၂ ဖြင့် ၁၀၁၀၁၀၁၀ ဖြစ်ပါစေ။ // 170

// Signs
က & ခ ကို ပြသမည်။          // and:  10001010 = 138
က | ခ ကို ပြသမည်။          // or:   11101010 = 234
က ^ ခ ကို ပြသမည်။          // xor:  01100000 = 96
~က ကို ပြသမည်။             // not:  00110101 = 53
က << ၂ ကို ပြသမည်။         // shift left 2:  00101000 = 40 (bits pushed out are lost)
က >> ၄ ကို ပြသမည်။         // shift right 4: 00001100 = 12

// Words
က နှင့် ခ ၏ ဘစ်အလိုက် ဆုံလဒ် ကို ပြသမည်။
က နှင့် ခ ၏ ဘစ်အလိုက် ပေါင်းစုလဒ် ကို ပြသမည်။
က နှင့် ခ ၏ ဘစ်အလိုက် ခြားနားလဒ် ကို ပြသမည်။
က ၏ ဘစ်အလိုက် ပြောင်းပြန်ပြောင်းလဒ် ကို ပြသမည်။
က ကို ဘယ်သို့ ၂ ဘစ် ရွေ့သောရလဒ် ကို ပြသမည်။
က ကို ညာသို့ ၄ ဘစ် ရွေ့သောရလဒ် ကို ပြသမည်။
```

```text
၁၃၈
၂၃၄
၉၆
၅၃
၄၀
၁၂
၁၃၈
၂၃၄
၉၆
၅၃
၄၀
၁၂
```

The first three word forms follow the same pattern as addition, with `ဘစ်အလိုက်` in front
of the result word:

```text
<a> နှင့် <b> ၏ ဘစ်အလိုက် ဆုံလဒ်
```

| Zaka | Reads as |
| --- | --- |
| `<a> နှင့် <b> ၏` | "of `<a>` and `<b>`" |
| `ဘစ်အလိုက်` | "bit for bit", one position at a time |
| `ဆုံလဒ်` | "where the bits meet" |

`ပေါင်းစုလဒ်` is "the union" and `ခြားနားလဒ်` is "the difference", while `~` has one value
alone, so its word form is `<a> ၏ ဘစ်အလိုက် ပြောင်းပြန်ပြောင်းလဒ်`, "the flip of `<a>`".

## Shifting bits

```text
<a> ကို ဘယ်သို့ <n> ဘစ် ရွေ့သောရလဒ်
<a> ကို ညာသို့ <n> ဘစ် ရွေ့သောရလဒ်
```

A shift moves all the bits of a number to the left or to the right. In the word form, we
write the direction, `ဘယ်သို့` ("to the left") or `ညာသို့` ("to the right"), then the
number of places and `ဘစ်`. Each place to the left doubles the number, so `၁ << ၅` is 32,
and the bits that move past the edge are lost.

| Zaka | Reads as |
| --- | --- |
| `<a> ကို` | "`<a>`", the number we shift |
| `ဘယ်သို့`, `ညာသို့` | "to the left", "to the right" |
| `<n> ဘစ်` | "`<n>` bits" |
| `ရွေ့သောရလဒ်` | "the result of moving" |

## How far a shift can go

A shift must move fewer places than the integer has bits, so an 8-bit integer can move
up to 7 places. When the compiler can see a longer shift, it stops with an error:

```zaka-error ZK0407
ကိန်းရှင် ခ သည် ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
ခ ကို ဘယ်သို့ ၈ ဘစ် ရွေ့သောရလဒ် ကို ပြသမည်။
```

```text
error[ZK0407]: shift distance 8 is not below the 8-bit width
```

## Keeping some bits with a mask

A mask is a number that we use with `&` to keep some bits and clear the rest. For
example, `F0` in base 16 is `11110000`, so `& F0` keeps the top four bits:

```zaka
ကိန်းရှင် က သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အခြေ ၂ ဖြင့် ၁၁၀၀၁၀၁၀ ဖြစ်ပါစေ။
ကိန်းရှင် မျက်နှာဖုံး သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အခြေ ၁၆ ဖြင့် F0 ဖြစ်ပါစေ။ // mask
က & မျက်နှာဖုံး ကို ပြသမည်။                     // 11000000 = 192
```

```text
၁၉၂
```

## Details

- **Integers alone.** Both sides of `&`, `|` and `^` must be integers of one data kind
  (type), and a shift keeps the data kind of the number on its left
  ([Bitwise operators](../../manual/language/expressions.md#bitwise-operators)).
- **A shift found too long at run time.** When the number of places comes from a variable
  and is too large, the program stops while it runs
  ([Bitwise operators](../../manual/language/expressions.md#bitwise-operators)).
- **Negative numbers.** A right shift of a signed integer repeats the sign bit, so
  `-၈ >> ၁` is `-၄` ([Bitwise operators](../../manual/language/expressions.md#bitwise-operators)).
- **Order.** Unlike C, the bitwise operators bind tighter than comparisons, so
  `၆ & ၃ == ၂` means `(၆ & ၃) == ၂` ([Precedence](../../manual/language/expressions.md#precedence)).
- **Writing bits.** `အခြေ ၂ ဖြင့်` and `အခြေ ၁၆ ဖြင့်` write a number in base 2 or base 16
  ([Integers](../values/integers.md)).

## Recap

- Bitwise operators work on the bits of integers, one position at a time.
- `&`, `|` and `^` have word forms that end in `ဆုံလဒ်`, `ပေါင်းစုလဒ်` and `ခြားနားလဒ်`.
- In a shift, we write the direction, `ဘယ်သို့` or `ညာသို့`, and the shift must move fewer
  places than the integer has bits.
- A mask with `&` keeps some bits and clears the rest.

## Hands on

- `၁ << n` is 2 to the power n. Print 2⁵ with a shift.
- Print whether bit 0 of 202 is set: `(က & ၁) != ၀`.

Next: [Converting between data kinds](conversion.md).
