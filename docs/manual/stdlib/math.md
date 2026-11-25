---
title: The math module
description: Math constants and functions on 64-bit floats.
module: စံကား/သင်္ချာ
---

# The math module

The math module, `"စံကား/သင်္ချာ"`, holds math constants and functions on 64-bit floats.

Import the constants `ပိုင်` (π) and `အွိုင်လာကိန်းသေ` (e) from `"စံကား/သင်္ချာ"`.
Each function takes and gives a `ဒဿမကိန်း`, a 64-bit floating-point number.

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
