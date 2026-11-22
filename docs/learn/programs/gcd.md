---
title: Greatest common divisor
description: Euclid's method, with a while loop and functions that call functions.
---

# Greatest common divisor

The *greatest common divisor* (GCD) of two numbers is the largest number that divides
both, so the GCD of 48 and 18 is 6. Long ago, in ancient Greece, Euclid found a fast
method to work it out: we replace the pair (a, b) with (b, a mod b) until b is zero, and
then a is the answer.

The *least common multiple* (LCM), the smallest number that both numbers divide, follows
from the GCD: LCM = a / GCD × b. This program prints both for two pairs of numbers.

```zaka
/// The greatest common divisor of k and kh, with Euclid's method.
ဘုံဆခွဲကိန်း ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ခ သည် ၀ နှင့် မညီနေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကြွင်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ကို ခ ဖြင့် စားသော စားကြွင်း ဖြစ်ပါစေ။
က ကို ခ ဟု ထားမည်။
ခ ကို ကြွင်း ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

/// The least common multiple, using the greatest common divisor.
ဘုံဆတိုးကိန်း ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က / (ဘုံဆခွဲကိန်း ဟူသော လုပ်ဆောင်ချက်ကို က နှင့် ခ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်) * ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

/// Print both, with headings.
နှစ်ခုပြ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ဘုံဆခွဲကိန်း" ဟု ပြသမည်။
ဘုံဆခွဲကိန်း ဟူသော လုပ်ဆောင်ချက်ကို က နှင့် ခ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
"ဘုံဆတိုးကိန်း" ဟု ပြသမည်။
ဘုံဆတိုးကိန်း ဟူသော လုပ်ဆောင်ချက်ကို က နှင့် ခ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// The program starts here
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
နှစ်ခုပြ ဟူသော လုပ်ဆောင်ချက်ကို ၄၈ နှင့် ၁၈ တို့ဖြင့် လုပ်ဆောင်မည်။
နှစ်ခုပြ ဟူသော လုပ်ဆောင်ချက်ကို ၁၇ နှင့် ၅ တို့ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဘုံဆခွဲကိန်း
၆
ဘုံဆတိုးကိန်း
၁၄၄
ဘုံဆခွဲကိန်း
၁
ဘုံဆတိုးကိန်း
၈၅
```

## How it works

- **`ဘုံဆခွဲကိန်း`** (GCD) runs a [while loop](../control-flow/while.md) as long as `ခ` is
  not zero. In each round, it works out the remainder, then moves the pair along one step.
  The loop changes the parameters `က` and `ခ`, which is fine, because each parameter is a
  local duplicate (copy) of the argument ([Parameters are local
  copies](../functions/parameters.md#parameters-are-local-copies)).
- **`ဘုံဆတိုးကိန်း`** (LCM) calls `ဘုံဆခွဲကိန်း` inside a sum. We put the call in
  parentheses so that the division happens before the multiplication, and dividing first
  keeps the numbers small.
- **`နှစ်ခုပြ`** ("show both") has no result: it prints the headings and calls the other
  two functions.

This table traces GCD(48, 18), one round of the loop on each row:

| Round | က | ခ | remainder |
| --- | --- | --- | --- |
| 1 | 48 | 18 | 12 |
| 2 | 18 | 12 | 6 |
| 3 | 12 | 6 | 0 |
| end | 6 | 0 | |

## Hands on

- Print the GCD of 1071 and 462.
- Rewrite `ဘုံဆခွဲကိန်း` with [recursion](../functions/recursion.md): GCD(a, 0) = a, and
  GCD(a, b) = GCD(b, a mod b).

Next: [Fibonacci numbers](fibonacci.md).
