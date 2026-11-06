---
title: Integers
description: Whole numbers - their sizes, signed and unsigned - and how to write them, with Burmese or Western digits, number bases and kind prefixes.
---

# Integers

An *integer* is a whole number with no fraction: `၀`, `၇`, `၁၀၀၀`, `-၅`. Its kind is
`ကိန်းပြည့်` ("whole number").

The computer stores each integer in a fixed number of *bits* (base-2 digits). More bits
hold bigger numbers. A program can choose the size, and whether negative numbers are
allowed.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၄၂ ကို ပြသပါ။                                // plain integer: the default kind
၈ ဘစ် ကိန်းပြည့် ၁၂၇ ကို ပြသပါ။                // 8 bits: -128 up to 127
လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် ၂၅၅ ကို ပြသပါ။        // 8 bits, no sign: 0 up to 255
၃၂ ဘစ် ကိန်းပြည့် ၂၁၄၇၄၈၃၆၄၇ ကို ပြသပါ။         // 32 bits
-၅ ကို ပြသပါ။                                // a negative number
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၂
၁၂၇
၂၅၅
၂၁၄၇၄၈၃၆၄၇
-၅
```

Writing a kind before the digits, as in `၈ ဘစ် ကိန်းပြည့် ၁၂၇`, fixes the kind of that
number. Without one, an integer has the default kind `ကိန်းပြည့်`.

## Building an integer kind

An integer kind is made of up to three parts, in this order:

| Part | Words | Meaning |
| --- | --- | --- |
| sign (optional) | `လက္ခဏာရှိ` / `လက္ခဏာမဲ့` | "with sign" (can be negative) / "without sign" (zero or more) |
| size (optional) | `၈ ဘစ်`, `၁၆ ဘစ်`, `၃၂ ဘစ်`, `၆၄ ဘစ်`, `၁၂၈ ဘစ်` | 8, 16, 32, 64 or 128 bits |
| kind | `ကိန်းပြည့်` | integer |

Leave out the sign and the integer is signed. Leave out the size and it uses the
*platform size*, which is 64 bits unless the program runs with a different
`--platform-bits` setting.

| Kind | Smallest | Largest |
| --- | --- | --- |
| `၈ ဘစ် ကိန်းပြည့်` | -128 | 127 |
| `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်` | 0 | 255 |
| `၁၆ ဘစ် ကိန်းပြည့်` | -32,768 | 32,767 |
| `လက္ခဏာမဲ့ ၁၆ ဘစ် ကိန်းပြည့်` | 0 | 65,535 |
| `၃၂ ဘစ် ကိန်းပြည့်` | about -2.1 billion | about 2.1 billion |
| `၆၄ ဘစ် ကိန်းပြည့်` | about -9.2 × 10¹⁸ | about 9.2 × 10¹⁸ |
| `ကိန်းပြည့်` | (64 bits unless set otherwise) | |

When in doubt, use plain `ကိန်းပြည့်`.

## A number that does not fit

If a number is too big for its kind, the program does not run:

```zaka-error ZK0220
လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် ၂၅၆ ကို ပြသပါ။
```

```text
error[ZK0220]: literal ၂၅၆ does not fit its kind
```

## Writing integers

A number written straight into a program, like `၄၂`, is called a *literal*. Zaka accepts
several forms of integer literal.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁၂၃ ကို ပြသပါ။                                 // Burmese digits
123 ကို ပြသပါ။                                  // Western digits: the same number
အခြေ ၂ ဖြင့် ၁၀၁၀ ကို ပြသပါ။                     // base 2: 1010 = ten
အခြေ ၈ ဖြင့် ၇၇ ကို ပြသပါ။                       // base 8: 77 = 63
အခြေ ၁၆ ဖြင့် FF ကို ပြသပါ။                       // base 16 (hexadecimal): FF = 255
အခြေ ၁၆ ဖြင့် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် FF ကို ပြသပါ။  // base and kind together
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၂၃
၁၂၃
၁၀
၆၃
၂၅၅
၂၅၅
```

## Digits

Use Burmese digits `၀ ၁ ၂ ၃ ၄ ၅ ၆ ၇ ၈ ၉` or Western digits `0 1 2 3 4 5 6 7 8 9`, but not
both in one number:

```zaka-error ZK0103
၁၂.34 ကို ပြသပါ။
```

```text
error[ZK0103]: Burmese and Western digits cannot be mixed in one number
```

## Bases

`အခြေ ... ဖြင့်` reads "with base ...". It writes a number in a base from 2 to 36. Digits
above 9 are the letters `A` to `Z` (upper or lower case): in base 16, `A` is ten and `F`
is fifteen. The base itself is written in plain base 10.

| Zaka | Reads as |
| --- | --- |
| `အခြေ` | "base" |
| `ဖြင့်` | "with" |

## The order of the parts

A full integer literal has up to three parts, in this fixed order:

1. the base, `အခြေ <n> ဖြင့်` (optional),
2. the kind, such as `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်` (optional),
3. the digits.

## Negative numbers

There is no negative literal. `-၅` is the minus operator applied to `၅`. The word
`အနုတ်` ("minus") does the same job. The compiler works out the result before the
program runs, so the difference never shows.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
-၅ ကို ပြသပါ။
အနုတ် ၅ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
-၅
-၅
```

## Hands on

- Print the largest `လက္ခဏာမဲ့ ၁၆ ဘစ် ကိန်းပြည့်`, then test one more than it.
- Can a `လက္ခဏာမဲ့` integer hold `-၁`? Test it.
- Print `အခြေ ၂ ဖြင့် ၁၁၁၁၁၁၁၁`. Which number in base 10 is it?
- What does `အခြေ ၂ ဖြင့် ၁၂` do? Read the message.

Next: [Decimal numbers](floats.md).
