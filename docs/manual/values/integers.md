---
title: Integers
description: Whole numbers, their sizes, and signed versus unsigned.
---

# Integers

An *integer* is a whole number with no fraction: `၀`, `၇`, `၁၀၀၀`, `-၅`. Its kind is
`ကိန်းပြည့်` ("whole number").

The computer stores each integer in a fixed number of *bits* (binary digits). More bits
hold bigger numbers. You can choose the size, and you can choose whether negative
numbers are allowed.

```zaka
၄၂ ကို ပြသပါ။                                // plain integer: the default kind
၈ ဘစ် ကိန်းပြည့် ၁၂၇ ကို ပြသပါ။                // 8 bits: -128 up to 127
လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် ၂၅၅ ကို ပြသပါ။        // 8 bits, no sign: 0 up to 255
၃၂ ဘစ် ကိန်းပြည့် ၂၁၄၇၄၈၃၆၄၇ ကို ပြသပါ။         // 32 bits
-၅ ကို ပြသပါ။                                // a negative number
```

```text
၄၂
၁၂၇
၂၅၅
၂၁၄၇၄၈၃၆၄၇
-၅
```

Writing a kind before the digits, as in `၈ ဘစ် ကိန်းပြည့် ၁၂၇`, fixes the kind of that
number. Without one, an integer has the default kind `ကိန်းပြည့်`.

## Building an integer kind

An integer kind is made of up to three parts, in this order:

| Part | Words | Meaning |
| --- | --- | --- |
| sign (optional) | `လက္ခဏာရှိ` / `လက္ခဏာမဲ့` | "with sign" (can be negative) / "without sign" (zero or more) |
| size (optional) | `၈ ဘစ်`, `၁၆ ဘစ်`, `၃၂ ဘစ်`, `၆၄ ဘစ်`, `၁၂၈ ဘစ်` | 8, 16, 32, 64 or 128 bits |
| kind | `ကိန်းပြည့်` | integer |

Leave out the sign and the integer is signed. Leave out the size and it uses the
*platform size*, which is 64 bits unless you run the program with a different
`--platform-bits` setting.

| Kind | Smallest | Largest |
| --- | --- | --- |
| `၈ ဘစ် ကိန်းပြည့်` | -128 | 127 |
| `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်` | 0 | 255 |
| `၁၆ ဘစ် ကိန်းပြည့်` | -32,768 | 32,767 |
| `လက္ခဏာမဲ့ ၁၆ ဘစ် ကိန်းပြည့်` | 0 | 65,535 |
| `၃၂ ဘစ် ကိန်းပြည့်` | about -2.1 billion | about 2.1 billion |
| `၆၄ ဘစ် ကိန်းပြည့်` | about -9.2 × 10¹⁸ | about 9.2 × 10¹⁸ |
| `ကိန်းပြည့်` | (same as 64 bits by default) | |

When in doubt, use plain `ကိန်းပြည့်`.

## A number that does not fit

If you write a number too big for its kind, the program does not run:

```zaka-error ZK0220
လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် ၂၅၆ ကို ပြသပါ။
```

```text
error[ZK0220]: literal ၂၅၆ does not fit its kind
```

## Try it

- Print the largest `လက္ခဏာမဲ့ ၁၆ ဘစ် ကိန်းပြည့်`, then try one more than it.
- Can a `လက္ခဏာမဲ့` integer hold `-၁`? Try it.

Next: [Writing integers](literals.md).
