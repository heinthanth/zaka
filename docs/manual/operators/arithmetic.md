---
title: Arithmetic
description: Add, subtract, multiply, divide, take the remainder, and negate.
---

# Arithmetic

```zaka
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၇ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။

// Symbols
က + ခ ကို ပြသပါ။          // add
က - ခ ကို ပြသပါ။          // subtract
က * ခ ကို ပြသပါ။          // multiply
က / ခ ကို ပြသပါ။          // divide
က % ခ ကို ပြသပါ။          // remainder
-က ကို ပြသပါ။             // negate

// Words
က နှင့် ခ ၏ ပေါင်းလဒ် ကို ပြသပါ။
က နှင့် ခ ၏ နုတ်လဒ် ကို ပြသပါ။
က နှင့် ခ ၏ မြှောက်လဒ် ကို ပြသပါ။
က ကို ခ ဖြင့် စားသော စားလဒ် ကို ပြသပါ။
က ကို ခ ဖြင့် စားသော စားကြွင်း ကို ပြသပါ။
အနုတ် က ကို ပြသပါ။
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

| Symbol | Words | Reads as | Does |
| --- | --- | --- | --- |
| `a + b` | `a နှင့် b ၏ ပေါင်းလဒ်` | "the sum of a and b" | add |
| `a - b` | `a နှင့် b ၏ နုတ်လဒ်` | "the difference of a and b" | subtract |
| `a * b` | `a နှင့် b ၏ မြှောက်လဒ်` | "the product of a and b" | multiply |
| `a / b` | `a ကို b ဖြင့် စားသော စားလဒ်` | "a divided by b: the quotient" | divide |
| `a % b` | `a ကို b ဖြင့် စားသော စားကြွင်း` | "a divided by b: the remainder" | remainder |
| `-a` | `အနုတ် a` | "minus a" | negate |

## Dividing integers

Dividing two integers gives an integer. The fraction is dropped, so `၁၇ / ၅` is `၃`, not
`၃.၄`. The remainder `%` gives what is left over: 17 = 3 × 5 + **2**.

To keep the fraction, divide decimal numbers: `၁၇.၀ / ၅` is `၃.၄`.

```zaka
၁၇ / ၅ ကို ပြသပါ။
၁၇.၀ / ၅ ကို ပြသပါ။
-၇ / ၂ ကို ပြသပါ။        // the fraction is dropped toward zero
-၇ % ၂ ကို ပြသပါ။
```

```text
၃
၃.၄
-၃
-၁
```

Dividing by zero has no answer. If the compiler can see the zero, the program does not
run:

```zaka-error ZK0602
၁ / ၀ ကို ပြသပါ။
```

```text
error[ZK0602]: / with the constant zero as divisor
```

If the zero only appears while the program runs, the program stops at that point:

```zaka-stop
ကိန်းရှင် ခ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
"အစ" ဟု ပြသပါ။
၁၀ / ခ ကို ပြသပါ။
"ဤနေရာ မရောက်ပါ" ဟု ပြသပါ။
```

```text
အစ
the program stopped: integer division or remainder with zero as divisor
```

## Mixing kinds

Two numbers of the same family combine freely. The smaller kind is widened to the
larger one. An `၈ ဘစ် ကိန်းပြည့်` plus a `ကိန်းပြည့်` gives a `ကိန်းပြည့်`. But a plain
integer and a decimal number need an explicit [conversion](conversion.md), because a
large integer could lose digits as a decimal:

```zaka-error ZK0404
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
ကိန်းရှင် ဈေး သည် ဒဿမကိန်း ဖြစ်၍ တန်ဖိုးသည် ၁.၅ ဖြစ်ပါစေ။
အသက် * ဈေး ကို ပြသပါ။
```

```text
error[ZK0404]: cannot combine isize and f64 without a conversion
```

Numbers written straight into the program bend to fit: `၃ + ၀.၅` is `၃.၅`.

## Too big for the kind

Each integer kind has a largest and a smallest value (see [Integers](../values/integers.md)).
If a result goes past the edge while the program runs, it *wraps around* to the other
end, like a car's odometer rolling over:

```zaka
ကိန်းရှင် ခ သည် ၈ ဘစ် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၂၇ ဖြစ်ပါစေ။               // the largest 8-bit integer
ခ + ၁ ကို ပြသပါ။                                        // wraps to the smallest
ကိန်းရှင် လ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
လ - ၁ ကို ပြသပါ။                                        // wraps to 255
```

```text
-၁၂၈
၂၅၅
```

Pick a kind big enough for your numbers. Plain `ကိန်းပြည့်` is a safe choice.

## Try it

- Work out the average of 4, 7 and 10 as a decimal number.
- Use `%` to print whether 2026 divided by 4 leaves a remainder.

Next: [Comparison](comparison.md).
