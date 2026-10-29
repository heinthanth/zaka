---
title: Decimal numbers
description: Numbers with a fraction part, written with a decimal point.
---

# Decimal numbers

A *decimal number* has a fraction part: `၃.၁၄`, `၀.၅`, `-၂.၅`. Its kind is
`ဒဿမကိန်း` ("decimal number"). Other languages call these *floating-point numbers*, or
*floats*.

```zaka
၃.၁၄ ကို ပြသပါ။                    // a decimal point makes a decimal number
-၂.၅ ကို ပြသပါ။
ဒဿမကိန်း ၁၀ ကို ပြသပါ။             // the kind prefix makes 10 a decimal number
1.5e3 ကို ပြသပါ။                   // scientific notation: 1.5 × 10³
1.5e-3 ကို ပြသပါ။                  // 1.5 × 10⁻³
၇.၀ / ၂ ကို ပြသပါ။                  // decimal division keeps the fraction
```

```text
၃.၁၄
-၂.၅
၁၀.၀
၁၅၀၀.၀
၀.၀၀၁၅
၃.၅
```

A decimal number prints with a decimal point each time, even when it is a whole number
(`၁၀.၀`).

## Sizes

Like integers, decimal numbers come in sizes. More bits mean more correct digits.

| Kind | Bits | Correct digits (about) |
| --- | --- | --- |
| `၁၆ ဘစ် ဒဿမကိန်း` | 16 | 3 |
| `၃၂ ဘစ် ဒဿမကိန်း` | 32 | 7 |
| `ဒဿမကိန်း`, `၆၄ ဘစ် ဒဿမကိန်း` | 64 | 15 (the default) |
| `၁၂၈ ဘစ် ဒဿမကိန်း` | 128 | 33 |

## Close, not exact

Decimal numbers are stored in base 2, so most fractions are stored as a close neighbour
and not as the true value. Most of the time this goes unnoticed, but now and then the
last digits show it:

```zaka
၀.၁ + ၀.၂ ကို ပြသပါ။
၁၆ ဘစ် ဒဿမကိန်း ၀.၁ ကို ပြသပါ။    // fewer bits, a less close neighbour
```

```text
၀.၃၀၀၀၀၀၀၀၀၀၀၀၀၀၀၀၄
၀.၀၉၉၉၇၆
```

This is true of decimal numbers in almost all programming languages, not just Zaka.

## Rules

- Decimal numbers are written in base 10 and no other base.
- Scientific notation (`e3`, `E-3`) needs Western digits: `1.5e3`, not `၁.၅e3`.

## Hands on

- Print `၁ / ၃` and then `၁.၀ / ၃`. What is different?
- Print `၃၂ ဘစ် ဒဿမကိန်း ၀.၁`. Is it closer to 0.1 than the 16-bit one?

Next: [Characters](characters.md).
