---
title: Integers
description: Whole numbers - their sizes, signed and unsigned - and how to write them, with Burmese or Western digits, number bases and the data kind written after a number.
---

# Integers

An *integer* (`int`) is a whole number with no fraction, such as `၀`, `၇`, `၁၀၀၀` or `-၅`,
and its data kind (type) is `ကိန်းပြည့်` ("whole number"). The computer stores each
integer in a fixed number of *bits* (base-2 digits), so we can choose how big an integer
can get, and whether it can be negative.

## You will learn

- How to write an integer with Burmese or Western digits.
- How to choose the size and the sign of an integer kind.
- How to write a number in base 2, 8 or 16.
- What happens when a number does not fit its data kind.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၄၂ ကို ပြသမည်။                                     // a plain integer
-၅ ကို ပြသမည်။                                     // a negative integer
၁၂၇ ဟူသော ၈ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြသမည်။          // an 8-bit integer
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၂
-၅
၁၂၇
```

A plain number such as `၄၂` gets the data kind `ကိန်းပြည့်`. To choose another integer
kind for a number, we write `ဟူသော` and the data kind after the digits, as the third line
does:

```
<digits> ဟူသော <kind> တစ်ခု
```

| Zaka | Reads as |
| --- | --- |
| `၁၂၇` | 127 |
| `ဟူသော` | "that is", "called" |
| `၈ ဘစ် ကိန်းပြည့်` | "8-bit integer" |
| `တစ်ခု` | "one"; `ခု` is the measure word (classifier) for most data kinds |

## Building an integer kind

```
<sign> <size> ကိန်းပြည့်
```

An integer kind has up to three parts, in this order, and we can leave out the first two:

| Part | Words | Meaning |
| --- | --- | --- |
| sign (optional) | `လက္ခဏာရှိ` / `လက္ခဏာမဲ့` | "with sign" (can be negative) / "without sign" (zero or more) |
| size (optional) | `၈ ဘစ်`, `၁၆ ဘစ်`, `၃၂ ဘစ်`, `၆၄ ဘစ်` | 8, 16, 32 or 64 bits |
| data kind | `ကိန်းပြည့်` | integer |

When we leave out the sign, the integer is signed. When we leave out the size, the
integer has the *platform size*, which is 64 bits unless the program runs with a
different `--target-bits` setting. More bits mean bigger numbers:

| Integer kind | Smallest | Largest |
| --- | --- | --- |
| `၈ ဘစ် ကိန်းပြည့်` | -128 | 127 |
| `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်` | 0 | 255 |
| `၁၆ ဘစ် ကိန်းပြည့်` | -32,768 | 32,767 |
| `လက္ခဏာမဲ့ ၁၆ ဘစ် ကိန်းပြည့်` | 0 | 65,535 |
| `၃၂ ဘစ် ကိန်းပြည့်` | about -2.1 billion | about 2.1 billion |
| `၆၄ ဘစ် ကိန်းပြည့်` | about -9.2 × 10¹⁸ | about 9.2 × 10¹⁸ |
| `ကိန်းပြည့်` | (64 bits unless set otherwise) | |

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၂၅၅ ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြသမည်။        // 8 bits, no sign: 0 up to 255
၂၁၄၇၄၈၃၆၄၇ ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြသမည်။         // 32 bits
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၅၅
၂၁၄၇၄၈၃၆၄၇
```

When in doubt, we use plain `ကိန်းပြည့်`.

## A number that does not fit

When a number is too big for its data kind, the compiler stops, and the program does not
run:

```zaka-error ZK0220 repl=on
၂၅၆ ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြသမည်။
```

```text
error[ZK0220]: literal ၂၅၆ does not fit its kind
```

## Burmese and Western digits

A number that we write straight into a program, such as `၄၂`, is called a *literal*. We
can write its digits as Burmese digits `၀ ၁ ၂ ၃ ၄ ၅ ၆ ၇ ၈ ၉` or as Western digits `0 1 2 3
4 5 6 7 8 9`, and both give the same number. The output uses Burmese digits in both cases:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁၂၃ ကို ပြသမည်။                                 // Burmese digits
123 ကို ပြသမည်။                                  // Western digits: the same number
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၂၃
၁၂၃
```

We cannot mix the two sets of digits in one number:

```zaka-error ZK0103 repl=on
၁၂.34 ကို ပြသမည်။
```

```text
error[ZK0103]: Burmese and Western digits cannot be mixed in one number
```

## Bases

```
အခြေ <base> ဖြင့် <digits>
```

To write a number in another base, from 2 to 36, we put `အခြေ <base> ဖြင့်` ("with base
...") before the digits. Digits above 9 are the letters `A` to `Z`, in upper or lower
case, so in base 16 `A` is ten and `F` is fifteen. We write the base itself in plain base
10.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အခြေ ၂ ဖြင့် ၁၀၁၀ ကို ပြသမည်။                     // base 2: 1010 = ten
အခြေ ၈ ဖြင့် ၇၇ ကို ပြသမည်။                       // base 8: 77 = 63
အခြေ ၁၆ ဖြင့် FF ကို ပြသမည်။                       // base 16 (hexadecimal): FF = 255
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၀
၆၃
၂၅၅
```

| Zaka | Reads as |
| --- | --- |
| `အခြေ` | "base" |
| `ဖြင့်` | "with" |

## The order of the parts

An integer literal has up to three parts, in this fixed order:

1. the base, `အခြေ <base> ဖြင့်` (optional),
2. the digits,
3. the data kind, as `ဟူသော` and an integer kind with its measure word, such as
   `ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု` (optional).

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အခြေ ၁၆ ဖြင့် FF ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြသမည်။  // base, digits, data kind
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၅၅
```

## Negative numbers

Zaka has no negative literal: `-၅` is the minus operator in front of `၅`, and we can also
write it with the word `အနုတ်` ("minus"). The compiler works out the result before the
program runs, so we never see the difference:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
-၅ ကို ပြသမည်။
အနုတ် ၅ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
-၅
-၅
```

## Details

- **Two data kinds of 64 bits.** `ကိန်းပြည့်` and `၆၄ ဘစ် ကိန်းပြည့်` are two data kinds
  even when the platform has 64 bits, so adding one to the other needs a conversion
  (ZK0404). See [Integer kinds](../../manual/language/kinds.md#integer-kinds).
- **A minus sign with a data kind.** The data kind after the digits covers the minus sign
  too, so `-၁၂၈ ဟူသော ၈ ဘစ် ကိန်းပြည့် တစ်ခု` is the smallest 8-bit integer.
- **No minus for an unsigned integer.** `-၁ ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု` is the
  error ZK0403, because a `လက္ခဏာမဲ့` integer cannot be negative.
- **Other mistakes.** A size other than 8, 16, 32 or 64 is ZK0216, a digit that does not
  belong to the base is ZK0219, and a number that does not fit the data kind of its place
  is ZK0406. [Numbers](../../manual/language/words.md#numbers) and [Integer
  kinds](../../manual/language/kinds.md#integer-kinds) state the full rules.

## Recap

- An integer is a whole number, and plain `ကိန်းပြည့်` is the data kind to use when the
  size does not matter.
- We build other integer kinds from a sign, a size and `ကိန်းပြည့်`, such as
  `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်`.
- We write `<digits> ဟူသော <kind> တစ်ခု` to give a number a data kind.
- We can write Burmese or Western digits, and a base with `အခြေ <base> ဖြင့်`.

## Hands on

- Print the largest `လက္ခဏာမဲ့ ၁၆ ဘစ် ကိန်းပြည့်`, then test one more than it.
- Can a `လက္ခဏာမဲ့` integer be `-၁`? Test it.
- Print `အခြေ ၂ ဖြင့် ၁၁၁၁၁၁၁၁`. Which number in base 10 is it?
- What does `အခြေ ၂ ဖြင့် ၁၂` do? Read the message.

Next: [Decimal numbers](floats.md).
