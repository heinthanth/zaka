---
title: Arithmetic
description: Addition, subtraction, multiplication, division, remainder and negation, in signs and in words.
---

# Arithmetic

Arithmetic operators do addition, subtraction, multiplication and division, the same as
a calculator does. Each one has a sign form, such as `+`, and a word form, such as
`<a> နှင့် <b> ၏ ပေါင်းလဒ်` ("the sum of a and b"), and both forms work on integers and on
decimal numbers (floats).

## You will learn

- how to write addition, subtraction, multiplication, division and negation, in signs
  and in words
- what happens to the fraction when we divide two integers, and how to get the remainder
- what happens when we divide with zero, or when a result does not fit its data kind (type)
- which numbers we can mix in one sum

| Sign | Words | Reads as | Operation |
| --- | --- | --- | --- |
| `a + b` | `a နှင့် b ၏ ပေါင်းလဒ်` | "the sum of a and b" | addition |
| `a - b` | `a နှင့် b ၏ နုတ်လဒ်` | "the difference of a and b" | subtraction |
| `a * b` | `a နှင့် b ၏ မြှောက်လဒ်` | "the product of a and b" | multiplication |
| `a / b` | `a ကို b ဖြင့် စားသော စားလဒ်` | "a divided with b: the quotient" | division |
| `a % b` | `a ကို b ဖြင့် စားသော စားကြွင်း` | "a divided with b: the remainder" | remainder |
| `-a` | `အနုတ် a` | "minus a" | negation |

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၇ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။

// Signs
က + ခ ကို ပြသမည်။          // add
က - ခ ကို ပြသမည်။          // subtract
က * ခ ကို ပြသမည်။          // times
က / ခ ကို ပြသမည်။          // divide
က % ခ ကို ပြသမည်။          // remainder
-က ကို ပြသမည်။             // negate

// Words
က နှင့် ခ ၏ ပေါင်းလဒ် ကို ပြသမည်။
က နှင့် ခ ၏ နုတ်လဒ် ကို ပြသမည်။
က နှင့် ခ ၏ မြှောက်လဒ် ကို ပြသမည်။
က ကို ခ ဖြင့် စားသော စားလဒ် ကို ပြသမည်။
က ကို ခ ဖြင့် စားသော စားကြွင်း ကို ပြသမည်။
အနုတ် က ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၂
၁၂
၈၅
၃
၂
-၁၇
၂၂
၁၂
၈၅
၃
၂
-၁၇
```

The word forms follow two patterns. Addition, subtraction and multiplication join the
two numbers with `နှင့်` and `၏`, while division and the remainder name the number we
divide and the number we divide with:

```text
<a> နှင့် <b> ၏ <result word>
<a> ကို <b> ဖြင့် စားသော <result word>
```

| Zaka | Reads as |
| --- | --- |
| `<a> နှင့် <b> ၏` | "of `<a>` and `<b>`" |
| `<a> ကို` | "`<a>`", the number we divide |
| `<b> ဖြင့်` | "with `<b>`", the number we divide with |
| `စားသော` | "that divides" |
| `ပေါင်းလဒ်`, `နုတ်လဒ်`, `မြှောက်လဒ်` | "sum", "difference", "product" |
| `စားလဒ်`, `စားကြွင်း` | "quotient", "remainder" |

## Dividing integers

```text
<integer> / <integer>
<integer> % <integer>
```

When we divide two integers, the result is an integer too, so Zaka drops the fraction:
`၁၇ / ၅` is `၃`, not `၃.၄`. To get what is left over, we use `%`, the remainder: 17 is
3 × 5 + **2**, so `၁၇ % ၅` is `၂`. To keep the fraction, we divide decimal numbers
instead, and `၁၇.၀ / ၅` is `၃.၄`.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁၇ / ၅ ကို ပြသမည်။
၁၇.၀ / ၅ ကို ပြသမည်။
-၇ / ၂ ကို ပြသမည်။        // the fraction is dropped toward zero
-၇ % ၂ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၃
၃.၄
-၃
-၁
```

With a negative number, the fraction is dropped toward zero, so `-၇ / ၂` is `-၃`, and the
remainder has the sign of the number we divide.

Division with zero as the divisor has no answer. When the compiler can see the zero, it
stops with an error and the program does not run:

```zaka-error ZK0602
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ / ၀ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0602]: / with the constant zero as divisor
```

When the zero shows up while the program runs, the program stops at that point:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
"အစ" ဟု ပြသမည်။
၁၀ / ခ ကို ပြသမည်။
"ဤနေရာ မရောက်ပါ" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အစ
the program stopped: integer division or remainder with zero as divisor
```

## Mixing data kinds

We can add two integers, or two decimal numbers, with no extra step, even when their data
kinds have different sizes: Zaka widens the smaller data kind to the larger one,
so an `၈ ဘစ် ကိန်းပြည့်` plus a `ကိန်းပြည့်` returns a `ကိန်းပြည့်`. An integer variable
and a decimal variable do not mix, though, because a large integer could lose digits as a
decimal number, so we need to write a [conversion](conversion.md) to combine them:

```zaka-error ZK0404
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
ကိန်းရှင် ဈေး သည် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁.၅ ဖြစ်ပါစေ။
အသက် * ဈေး ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0404]: cannot combine isize and f64 without a conversion
```

A number that we write straight into the program gets the data kind that fits, so
`၃ + ၀.၅` is `၃.၅` with no conversion.

## Too big for the data kind

Each integer data kind has a largest and a smallest value (see
[Integers](../values/integers.md)). When a result goes past that edge while the program
runs, the program stops there, and the message names the operation and the data kind:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၂၇ ဖြစ်ပါစေ။           // the largest 8-bit integer
ခ + ၁ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: the result of + does not fit i8
```

To avoid this, we pick a data kind that is big enough for the numbers at hand, and plain
`ကိန်းပြည့်` is a safe choice.

Some programs want the result to wrap around to the other end instead, like a car's
odometer that rolls over. To get that, we compile them with `--overflow wrap`, or turn off
**Stop on overflow** in the Explorer:

```zaka overflow=wrap
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၂၇ ဖြစ်ပါစေ။           // the largest 8-bit integer
ခ + ၁ ကို ပြသမည်။                                        // wraps to the smallest
ကိန်းရှင် လ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
လ - ၁ ကို ပြသမည်။                                        // wraps to 255
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
-၁၂၈
၂၅၅
```

## Details

- **Negation.** `-` and `အနုတ်` need a signed integer or a decimal number, so `-က` on an
  unsigned `က` is the error ZK0403 ([Arithmetic](../../manual/language/expressions.md#arithmetic)).
- **Other data kinds.** A true/false value or a character does not work with arithmetic
  (ZK0403), while `+` in either form joins two pieces of text into new text
  ([Text](../../manual/language/text.md#joining)).
- **Decimal numbers and zero.** Dividing a decimal number with zero does not stop the
  program: `၁.၀ / ၀.၀` is `inf`, and `၀.၀ / ၀.၀` is `NaN`. The remainder of decimal
  numbers works too, so `၁၇.၅ % ၅` is `၂.၅`
  ([Arithmetic](../../manual/language/expressions.md#arithmetic)).
- **Constants.** When all the numbers in a sum are constants, the compiler works the sum
  out before the program runs, so a result that does not fit is the error ZK0601 rather
  than a stop ([Constants are worked out
  first](../../manual/language/expressions.md#constants-are-worked-out-first)).
- **A warning ahead of time.** When the compiler can see that an operation overflows for
  each value its variables can have, it warns with ZK0719
  ([Arithmetic](../../manual/language/expressions.md#arithmetic)).
- **Which numbers mix.** The full rules for mixing data kinds are on [Mixing data kinds in an
  operation](../../manual/language/conversions.md#mixing-data-kinds-in-an-operation).

## Recap

- Each arithmetic operator has a sign form (`က + ခ`) and a word form
  (`က နှင့် ခ ၏ ပေါင်းလဒ်`).
- Dividing two integers drops the fraction, and `%` returns the remainder.
- Dividing with zero is an error when the compiler can see the zero, and it stops the
  program otherwise.
- An integer variable and a decimal variable need a conversion before we can combine them.
- A result that does not fit its data kind stops the program, unless we compile with
  `--overflow wrap`.

## Hands on

- Work out the average of 4, 7 and 10 as a decimal number.
- Use `%` to print whether 2026 divided with 4 leaves a remainder.
- Print the quotient and the remainder of 100 divided with 7, in words.

Next: [Comparison](comparison.md).
