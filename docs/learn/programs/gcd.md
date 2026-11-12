---
title: Greatest common divisor
description: Euclid's method, with a while loop and functions that call functions.
---

# Greatest common divisor

The *greatest common divisor* (GCD) of two numbers is the largest number that divides
both. The GCD of 48 and 18 is 6. Long ago, in ancient Greece, Euclid found a fast
method to compute it: replace the pair (a, b) with (b, a mod b) until b is zero, and a
is the answer.

The *least common multiple* (LCM), the smallest number both divide into, follows from
it: LCM = a / GCD × b.

```zaka
/// The greatest common divisor of k and kh, with Euclid's method.
လုပ်ဆောင်ချက် ဘုံဆခွဲကိန်း သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ခ သည် ၀ နှင့် မညီနေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကြွင်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ကို ခ ဖြင့် စားသော စားကြွင်း ဖြစ်ပါစေ။
က ကို ခ ဟု ထားမည်။
ခ ကို ကြွင်း ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

/// The least common multiple, using the greatest common divisor.
လုပ်ဆောင်ချက် ဘုံဆတိုးကိန်း သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က / (ဘုံဆခွဲကိန်း ဟူသော လုပ်ဆောင်ချက်ကို က၊ ခ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်) * ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

/// Print both, with headings.
လုပ်ဆောင်ချက် နှစ်ခုပြ သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ဘုံဆခွဲကိန်း" ဟု ပြသမည်။
ဘုံဆခွဲကိန်း ဟူသော လုပ်ဆောင်ချက်ကို က၊ ခ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
"ဘုံဆတိုးကိန်း" ဟု ပြသမည်။
ဘုံဆတိုးကိန်း ဟူသော လုပ်ဆောင်ချက်ကို က၊ ခ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// The program starts here
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
နှစ်ခုပြ ဟူသော လုပ်ဆောင်ချက်ကို ၄၈၊ ၁၈ တို့ဖြင့် လုပ်ဆောင်မည်။
နှစ်ခုပြ ဟူသော လုပ်ဆောင်ချက်ကို ၁၇၊ ၅ တို့ဖြင့် လုပ်ဆောင်မည်။
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
  not zero. Each round it computes the remainder, then shifts the pair along. The
  parameters `က` and `ခ` are changed in place; that is fine, because
  [parameters are local copies](../functions/parameters.md#parameters-are-local-copies).
- **`ဘုံဆတိုးကိန်း`** (LCM) calls `ဘုံဆခွဲကိန်း` inside a sum. The call is in parentheses
  so that the division happens before the multiplication. Dividing first keeps the
  numbers small.
- **`နှစ်ခုပြ`** ("show both") has no result. It prints headings and calls the other two.

Tracing GCD(48, 18):

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
