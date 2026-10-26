---
title: Bitwise operators
description: Work on the individual bits of an integer.
---

# Bitwise operators

Inside the computer, an integer is a row of bits, each `0` or `1`. The number 202 in 8
bits is `11001010`. *Bitwise operators* work on these bits one position at a time.
Beginners rarely need them. They are useful for flags, colours, and low-level data.
Feel free to skip this page and come back later.

```zaka
ကိန်းရှင် က သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် အခြေ ၂ ဖြင့် ၁၁၀၀၁၀၁၀ ဖြစ်ပါစေ။   // 202
ကိန်းရှင် ခ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် အခြေ ၂ ဖြင့် ၁၀၁၀၁၀၁၀ ဖြစ်ပါစေ။   // 170

// Symbols
က & ခ ကို ပြသပါ။          // and:  10001010 = 138
က | ခ ကို ပြသပါ။          // or:   11101010 = 234
က ^ ခ ကို ပြသပါ။          // xor:  01100000 = 96
~က ကို ပြသပါ။             // not:  00110101 = 53
က << ၂ ကို ပြသပါ။         // shift left 2:  00101000 = 40 (bits pushed out are lost)
က >> ၄ ကို ပြသပါ။         // shift right 4: 00001100 = 12

// Words
က နှင့် ခ ၏ ဘစ်အလိုက် ဆုံလဒ် ကို ပြသပါ။
က နှင့် ခ ၏ ဘစ်အလိုက် ပေါင်းစုလဒ် ကို ပြသပါ။
က နှင့် ခ ၏ ဘစ်အလိုက် ခြားနားလဒ် ကို ပြသပါ။
က ၏ ဘစ်အလိုက် ပြောင်းပြန်ပြောင်းလဒ် ကို ပြသပါ။
က ကို ဘယ်သို့ ၂ ဘစ် ရွေ့သောရလဒ် ကို ပြသပါ။
က ကို ညာသို့ ၄ ဘစ် ရွေ့သောရလဒ် ကို ပြသပါ။
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

| Symbol | Words | Reads as | Each result bit is 1 when |
| --- | --- | --- | --- |
| `a & b` | `a နှင့် b ၏ ဘစ်အလိုက် ဆုံလဒ်` | "bitwise meeting" | both bits are 1 |
| `a \| b` | `a နှင့် b ၏ ဘစ်အလိုက် ပေါင်းစုလဒ်` | "bitwise union" | either bit is 1 |
| `a ^ b` | `a နှင့် b ၏ ဘစ်အလိုက် ခြားနားလဒ်` | "bitwise difference" | the bits differ |
| `~a` | `a ၏ ဘစ်အလိုက် ပြောင်းပြန်ပြောင်းလဒ်` | "bitwise flip" | the bit was 0 |
| `a << n` | `a ကို ဘယ်သို့ n ဘစ် ရွေ့သောရလဒ်` | "a moved left by n bits" | (bits move left) |
| `a >> n` | `a ကို ညာသို့ n ဘစ် ရွေ့သောရလဒ်` | "a moved right by n bits" | (bits move right) |

`ဘစ်အလိုက်` means "bit by bit". `ဘယ်သို့` is "to the left", and `ညာသို့` is "to the right".

## Rules

- Both sides must be integers.
- A shift must move fewer places than the integer has bits:

```zaka-error ZK0407
ကိန်းရှင် ခ သည် ၈ ဘစ် ကိန်းပြည့် ၁ ဖြစ်ပါစေ။
ခ ကို ဘယ်သို့ ၈ ဘစ် ရွေ့သောရလဒ် ကို ပြသပါ။
```

```text
error[ZK0407]: shift distance 8 is not below the 8-bit width
```

## A practical use

A *mask* keeps some bits and clears the rest. `& F0` (11110000) keeps the top four bits:

```zaka
ကိန်းရှင် က သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် အခြေ ၂ ဖြင့် ၁၁၀၀၁၀၁၀ ဖြစ်ပါစေ။
ကိန်းရှင် မျက်နှာဖုံး သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် အခြေ ၁၆ ဖြင့် F0 ဖြစ်ပါစေ။   // mask
က & မျက်နှာဖုံး ကို ပြသပါ။                     // 11000000 = 192
```

```text
၁၉၂
```

## Try it

- `၁ << n` is 2 to the power n. Print 2⁵ that way.
- Print whether bit 0 of 202 is set: `(က & ၁) != ၀`.

Next: [Converting between kinds](conversion.md).
