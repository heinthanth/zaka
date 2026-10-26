---
title: Writing integers
description: Burmese and Western digits, number bases, and kind prefixes.
---

# Writing integers

A number written straight into a program, like `၄၂`, is called a *literal*. Zaka gives
you a few ways to write integer literals.

```zaka
၁၂၃ ကို ပြသပါ။                                 // Burmese digits
123 ကို ပြသပါ။                                  // Western digits: the same number
အခြေ ၂ ဖြင့် ၁၀၁၀ ကို ပြသပါ။                     // base 2 (binary): 1010 = ten
အခြေ ၈ ဖြင့် ၇၇ ကို ပြသပါ။                       // base 8: 77 = sixty-three
အခြေ ၁၆ ဖြင့် FF ကို ပြသပါ။                       // base 16 (hexadecimal): FF = 255
အခြေ ၁၆ ဖြင့် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် FF ကို ပြသပါ။  // base and kind together
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

`အခြေ ... ဖြင့်` reads "by base ...". It writes a number in a base from 2 to 36. Digits
above 9 are the letters `A` to `Z` (upper or lower case): in base 16, `A` is ten and `F`
is fifteen. The base itself is always written in ordinary base 10.

| Zaka | Reads as |
| --- | --- |
| `အခြေ` | "base" |
| `ဖြင့်` | "by" |

## The order of the parts

A full integer literal has up to three parts, always in this order:

1. the base, `အခြေ <n> ဖြင့်` (optional),
2. the kind, such as `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်` (optional),
3. the digits.

## Negative numbers

There is no negative literal. `-၅` is the minus operator applied to `၅`. The word
`အနုတ်` ("minus") does the same job. The compiler works out the result before the
program runs, so you never notice the difference.

```zaka
-၅ ကို ပြသပါ။
အနုတ် ၅ ကို ပြသပါ။
```

```text
-၅
-၅
```

## Try it

- Print `အခြေ ၂ ဖြင့် ၁၁၁၁၁၁၁၁`. Which number in base 10 is it?
- What does `အခြေ ၂ ဖြင့် ၁၂` do? Read the message.

Next: [Decimal numbers](floats.md).
