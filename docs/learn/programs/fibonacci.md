---
title: Fibonacci numbers
description: A fast Fibonacci with a generic loop, and the golden ratio with decimals.
---

# Fibonacci numbers

Each Fibonacci number is the sum of the two before it: 0, 1, 1, 2, 3, 5, 8, ... The
[Recursion](../functions/recursion.md) page computed them with a function that calls
itself twice, which soon gets slow. This version walks forward with a loop and keeps
just the last two numbers.

Take the ratio of a Fibonacci number to the one before it, and the answer gets closer
and closer to the *golden ratio*, about 1.618.

```zaka
/// The n-th Fibonacci number, worked out with a loop.
လုပ်ဆောင်ချက် ဖီဘိုနာချီ သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
ကိန်းရှင် နောက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် အလှည့် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ကာ အလှည့် သည် န အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် လာမည့် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ရလဒ် + နောက် ဖြစ်ပါစေ။
ရလဒ် ကို နောက် ဟု ထားပါ။
နောက် ကို လာမည့် ဟု ထားပါ။
ပြီးလျှင် အလှည့် ကို အလှည့် + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

/// Two neighbours divided: close to the golden ratio for large n.
လုပ်ဆောင်ချက် အချိုး သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ဒဿမကိန်း ရလဒ် ကို ပြန်ပေးမည်။
ကိန်းရှင် ပိုင်းဝေ သည် ဒဿမကိန်း ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် ဖီဘိုနာချီ ကို န + ၁ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ဒဿမကိန်း အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် ပိုင်းခြေ သည် ဒဿမကိန်း ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် ဖီဘိုနာချီ ကို န တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ဒဿမကိန်း အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
ပိုင်းဝေ / ပိုင်းခြေ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

// The main program
၀ မှ ၉ အထိ န အနေဖြင့် ရေတွက်၍ လုပ်ဆောင်ချက် ဖီဘိုနာချီ ကို န တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
"၅၀ ကြိမ်မြောက်" ဟု ပြသပါ။
လုပ်ဆောင်ချက် ဖီဘိုနာချီ ကို ၅၀ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
"ရွှေအချိုး" ဟု ပြသပါ။
လုပ်ဆောင်ချက် အချိုး ကို ၂၀ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
```

```text
၀
၁
၁
၂
၃
၅
၈
၁၃
၂၁
၃၄
၅၀ ကြိမ်မြောက်
၁၂၅၈၆၂၆၉၀၂၅
ရွှေအချိုး
၁.၆၁၈၀၃၃၉၉၈၅၂၁၈၀၃၃
```

## How it works

- **`ဖီဘိုနာချီ`** keeps two numbers: `ရလဒ်` (the current one, starting at the default 0)
  and `နောက်` (the next one, starting at 1). Each round of the
  [generic loop](../control-flow/generic-loops.md) moves both one step forward. After n
  rounds, `ရလဒ်` holds the n-th number and becomes the result
  [with no return statement](../functions/results.md#returning-the-result).
- `လာမည့်` ("coming") is declared inside the loop's block, so it is a fresh variable
  each round.
- **`အချိုး`** ("ratio") [converts](../operators/conversion.md) both numbers to decimals
  before dividing. Integer division would drop the fraction and print `၁`.
- The 50th number, 12,586,269,025, is far too big for a 32-bit integer, but fits with
  room to spare in a plain `ကိန်းပြည့်`.

## Hands on

- Print the 90th Fibonacci number. Then print the 93rd. What happens, and what is the
  reason? (See [Too big for the kind](../operators/arithmetic.md#too-big-for-the-kind).)
