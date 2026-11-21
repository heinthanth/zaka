---
title: Decimal numbers
description: Numbers with a fraction part, written with a decimal point.
---

# Decimal numbers

A *decimal number* (float, floating-point number) is a number with a fraction part, such
as `၃.၁၄`, `၀.၅` or `-၂.၅`, and its data kind (type) is `ဒဿမကိန်း` ("decimal number").

## You will learn

- How to write a decimal number, with a point or in scientific notation.
- How to turn a whole number into a decimal number.
- The two sizes of decimal number, 32 and 64 bits.
- The reason `၀.၁ + ၀.၂` does not print `၀.၃`.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၃.၁၄ ကို ပြသမည်။
၇.၀ / ၂ ကို ပြသမည်။        // decimal division keeps the fraction
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၃.၁၄
၃.၅
```

To write a decimal number, we put a point between the whole part and the fraction, with
digits on both sides of the point:

```
<digits>.<digits>
```

| Zaka | Reads as |
| --- | --- |
| `၃` | the whole part, 3 |
| `.` | the decimal point |
| `၁၄` | the fraction, .14 |

A decimal number prints with a decimal point each time, even when it is a whole number,
as in `၁၀.၀`.

## Other forms of a decimal number

We can also turn a whole number into a decimal number with `ဟူသော ဒဿမကိန်း တစ်ခု`, and
we can write a big or small number in scientific notation, with `e` and a power of 10:

```
<digits> ဟူသော ဒဿမကိန်း တစ်ခု
<digits>.<digits>e<power>
```

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
-၂.၅ ကို ပြသမည်။
၁၀ ဟူသော ဒဿမကိန်း တစ်ခု ကို ပြသမည်။             // the data kind after it makes 10 a decimal number
1.5e3 ကို ပြသမည်။                   // scientific notation: 1.5 × 10³
1.5e-3 ကို ပြသမည်။                  // 1.5 × 10⁻³
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
-၂.၅
၁၀.၀
၁၅၀၀.၀
၀.၀၀၁၅
```

Scientific notation needs Western digits, so we write `1.5e3` and not `၁.၅e3`.

## Sizes

Like integers, decimal numbers come in sizes, and more bits mean more correct digits:

| Data kind | Bits | Correct digits (about) |
| --- | --- | --- |
| `၃၂ ဘစ် ဒဿမကိန်း` | 32 | 7 |
| `ဒဿမကိန်း`, `၆၄ ဘစ် ဒဿမကိန်း` | 64 | 15 (the default) |

## Close, not exact

The computer stores a decimal number in base 2, so most fractions are stored as a close
neighbour and not as the true value. Most of the time we do not notice, but now and then
the last digits show it:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၀.၁ + ၀.၂ ကို ပြသမည်။
၁ ဟူသော ၃၂ ဘစ် ဒဿမကိန်း တစ်ခု / ၃ ကို ပြသမည်။    // fewer bits, a less close neighbour
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၀.၃၀၀၀၀၀၀၀၀၀၀၀၀၀၀၀၄
၀.၃၃၃၃၃၃၃၄
```

This is true of decimal numbers in almost all programming languages, not just Zaka.

## Details

- **Base 10 alone.** We write decimal numbers in base 10, so a base prefix such as
  `အခြေ ၂ ဖြင့်` does not work with a fraction or an exponent.
- **Digits on both sides.** A fraction needs digits on both sides of the point: `၁.၅`,
  not `၁.` or `.၅`.
- **`inf` and `NaN`.** A division with a zero divisor gives `inf`, `-inf` or `NaN`, and
  the program does not stop. A literal that is too big for its data kind becomes `inf`.
  See [Decimal number kinds](../../manual/language/kinds.md#decimal-number-kinds).

## Recap

- A decimal number has a point and a fraction, and its data kind is `ဒဿမကိန်း`.
- It prints with a decimal point, even when it is whole.
- We can write `1.5e3` for 1.5 × 10³, with Western digits.
- `ဒဿမကိန်း` has 64 bits, and `၃၂ ဘစ် ဒဿမကိန်း` has fewer correct digits.
- A decimal number is a close neighbour of the true value, not the value itself.

## Hands on

- Print `၁ / ၃` and then `၁.၀ / ၃`. What is different?
- Print `၁.၀ / ၃` and `၁ ဟူသော ၃၂ ဘစ် ဒဿမကိန်း တစ်ခု / ၃`. Which digits of the two results match?
- Print `၁ / ၀.၀`. Does the program stop?

Next: [Characters](characters.md).
