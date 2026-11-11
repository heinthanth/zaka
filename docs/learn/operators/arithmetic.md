---
title: Arithmetic
description: Addition, subtraction, multiplication, division, remainder and negation.
---

# Arithmetic

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

| Sign | Words | Reads as | Operation |
| --- | --- | --- | --- |
| `a + b` | `a နှင့် b ၏ ပေါင်းလဒ်` | "the sum of a and b" | addition |
| `a - b` | `a နှင့် b ၏ နုတ်လဒ်` | "the difference of a and b" | subtraction |
| `a * b` | `a နှင့် b ၏ မြှောက်လဒ်` | "the product of a and b" | multiplication |
| `a / b` | `a ကို b ဖြင့် စားသော စားလဒ်` | "a over b: the quotient" | division |
| `a % b` | `a ကို b ဖြင့် စားသော စားကြွင်း` | "a over b: the remainder" | remainder |
| `-a` | `အနုတ် a` | "minus a" | negation |

## Dividing integers

Dividing two integers gives an integer. The fraction is dropped, so `၁၇ / ၅` is `၃`, not
`၃.၄`. The remainder `%` gives what is left over: 17 = 3 × 5 + **2**.

To keep the fraction, divide decimal numbers: `၁၇.၀ / ၅` is `၃.၄`.

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

Division with zero as the divisor has no answer. If the compiler can see the zero, the
program does not run:

```zaka-error ZK0602
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ / ၀ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0602]: / with the constant zero as divisor
```

If the zero first appears while the program runs, the program stops at that point:

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

## Mixing kinds

Two integers, or two decimal numbers, combine with no extra step. The smaller kind
is widened to the larger one. An `၈ ဘစ် ကိန်းပြည့်` plus a `ကိန်းပြည့်` gives a `ကိန်းပြည့်`. But a plain
integer and a decimal number need an explicit [conversion](conversion.md), because a
large integer could lose digits as a decimal:

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

Numbers written straight into the program bend to fit: `၃ + ၀.၅` is `၃.၅`.

## Too big for the kind

Each integer kind has a largest and a smallest value (see [Integers](../values/integers.md)).
If a result goes past the edge while the program runs, the program stops there and names
the operation and the kind:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၂၇ ဖြစ်ပါစေ။           // the largest 8-bit integer
ခ + ၁ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: the result of + does not fit i8
```

Pick a kind big enough for the numbers at hand. Plain `ကိန်းပြည့်` is a safe choice.

Some programs want the result to *wrap around* to the other end instead, like a car's
odometer rolling over. Compile them with `--overflow wrap`, or turn off **Stop on
overflow** in the explorer:

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

See [Overflow](../../manual/language/expressions.md#arithmetic) in the Docs.

## Hands on

- Work out the average of 4, 7 and 10 as a decimal number.
- Use `%` to print whether 2026 over 4 leaves a remainder.

Next: [Comparison](comparison.md).
