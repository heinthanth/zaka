---
title: Bitwise operators
description: Work on the individual bits of an integer.
---

# Bitwise operators

Inside the computer, an integer is a row of bits, each `0` or `1`. The number 202 in 8
bits is `11001010`. *Bitwise operators* work on these bits one position at a time.
Beginners seldom need them. These are useful for flags, colours, and low-level data.
Feel free to skip this page and come back later.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
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
က နှင့် ခ ကို ဘစ်ဆုံသည့်ရလဒ် ကို ပြသမည်။
က နှင့် ခ ကို ဘစ်ပေါင်းစုသည့်ရလဒ် ကို ပြသမည်။
က နှင့် ခ ကို ဘစ်ခြားနားသည့်ရလဒ် ကို ပြသမည်။
က ကို ဘစ်ပြောင်းပြန်သည့်ရလဒ် ကို ပြသမည်။
က ကို ၂ ဖြင့် ဘစ်ဘယ်ရွေ့သည့်ရလဒ် ကို ပြသမည်။
က ကို ၄ ဖြင့် ဘစ်ညာရွေ့သည့်ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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

| Sign | Words | Reads as | Each result bit is 1 when |
| --- | --- | --- | --- |
| `a & b` | `a နှင့် b ကို ဘစ်ဆုံသည့်ရလဒ်` | "bitwise meeting" | both bits are 1 |
| `a \| b` | `a နှင့် b ကို ဘစ်ပေါင်းစုသည့်ရလဒ်` | "bitwise union" | either bit is 1 |
| `a ^ b` | `a နှင့် b ကို ဘစ်ခြားနားသည့်ရလဒ်` | "bitwise difference" | the bits differ |
| `~a` | `a ကို ဘစ်ပြောင်းပြန်သည့်ရလဒ်` | "bitwise flip" | the bit was 0 |
| `a << n` | `a ကို n ဖြင့် ဘစ်ဘယ်ရွေ့သည့်ရလဒ်` | "a moved n bits to the left" | (bits move left) |
| `a >> n` | `a ကို n ဖြင့် ဘစ်ညာရွေ့သည့်ရလဒ်` | "a moved n bits to the right" | (bits move right) |

Each word is a function of the prelude with `သည့်ရလဒ်` glued to it: `ဘစ်ဆုံ` is "bits
meet", `ဘစ်ဘယ်ရွေ့` is "bits move to the left", and `ဘစ်ညာရွေ့` is "bits move to the right".

## Rules

- Both sides must be integers.
- A shift must move fewer places than the integer has bits:

```zaka-error ZK0407
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
ခ ကို ၈ ဖြင့် ဘစ်ဘယ်ရွေ့သည့်ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0407]: shift distance 8 is not below the 8-bit width
```

## A practical use

A *mask* keeps some bits and clears the rest. `& F0` (11110000) keeps the top four bits:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အခြေ ၂ ဖြင့် ၁၁၀၀၁၀၁၀ ဖြစ်ပါစေ။
ကိန်းရှင် မျက်နှာဖုံး သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အခြေ ၁၆ ဖြင့် F0 ဖြစ်ပါစေ။ // mask
က & မျက်နှာဖုံး ကို ပြသမည်။                     // 11000000 = 192
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၉၂
```

## Hands on

- `၁ << n` is 2 to the power n. Print 2⁵ with a shift.
- Print whether bit 0 of 202 is set: `(က & ၁) != ၀`.

Next: [Converting between kinds](conversion.md).
