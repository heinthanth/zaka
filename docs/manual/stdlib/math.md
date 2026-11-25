---
title: The math module
description: Math constants and functions on 64-bit floats.
---

# The math module

The math module, `"စံကား/သင်္ချာ"`, holds math constants and functions on 64-bit floats.

Import the constants `ပိုင်` (π) and `အွိုင်လာကိန်းသေ` (e) from `"စံကား/သင်္ချာ"`.
Each function takes and gives a `ဒဿမကိန်း`, a 64-bit floating-point number.

| Call | What it gives |
| --- | --- |
| `ပကတိတန်ဖိုး(n)` | absolute value |
| `နှစ်ထပ်ကိန်းရင်း(n)` | square root |
| `a ကို b ထပ်တင်သည့်ရလဒ်`, `ထပ်တင်(a၊ b)` | `a` raised to the power `b` |
| `အခြေ၁၀ရှိသောလော့ဂရစ်သမ်(n)` | natural and base-ten logarithms |
| `ဆိုင်း(n)`, `ကိုဆိုင်း(n)`, `တန်းဂျင့်(n)` | sine, cosine and tangent of an angle in radians |
| `အောက်ဘက်အနီးဆုံးကိန်းပြည့်လျှော့ယူ(n)`, `အပေါ်ဘက်အနီးဆုံးကိန်းပြည့်တိုးယူ(n)` | round down or up to an integer value |
| `အနီးဆုံးကိန်းပြည့်ယူ(n)` | round to the nearest integer value, with ties going to the larger magnitude |
| `ဒဿမဖြုတ်(n)` | drop the fractional part, rounding towards zero |

The functions follow floating-point arithmetic. The square root of a negative number
gives `NaN`, and the logarithm of zero gives `-inf`. Transcendental functions
can differ in their last digits across platforms.

```zaka
"စံကား/သင်္ချာ" မှ နှစ်ထပ်ကိန်းရင်း၊ ထပ်တင် နှင့် အနီးဆုံးကိန်းပြည့်ယူ တို့ကို ထည့်သွင်းမည်။
၈၁.၀ ၏ နှစ်ထပ်ကိန်းရင်း ကို ပြသမည်။
၁၀ ကို ၂ ထပ်တင်သည့်ရလဒ် ကို ပြသမည်။
အနီးဆုံးကိန်းပြည့်ယူ(-၂.၅) ကို ပြသမည်။
```

```text
၉.၀
၁၀၀.၀
-၃.၀
```
