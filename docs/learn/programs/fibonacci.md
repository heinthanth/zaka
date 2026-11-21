---
title: Fibonacci numbers
description: A fast Fibonacci with a generic loop, and the golden ratio with decimal numbers.
---

# Fibonacci numbers

Each Fibonacci number is the sum of the two before it: 0, 1, 1, 2, 3, 5, 8, and so on. The
[Recursion](../functions/recursion.md) page worked them out with a function that calls
itself twice, which soon gets slow, so this version walks forward with a loop and keeps
just the last two numbers.

When we divide a Fibonacci number with the one before it, the answer comes nearer
and nearer to the *golden ratio*, about 1.618, and the program prints that too.

```zaka
/// The n-th Fibonacci number, worked out with a loop.
ဖီဘိုနာချီ ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် နောက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် အလှည့် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ကာ အလှည့် သည် န အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လာမည့် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အဖြေ + နောက် ဖြစ်ပါစေ။
အဖြေ ကို နောက် ဟု ထားမည်။
နောက် ကို လာမည့် ဟု ထားမည်။
ပြီးလျှင် အလှည့် ကို အလှည့် + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

/// Two neighbours divided: close to the golden ratio for large n.
အချိုး ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ဒဿမကိန်း တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် ပိုင်းဝေ သည် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဖီဘိုနာချီ ဟူသော လုပ်ဆောင်ချက်ကို န + ၁ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ဒဿမကိန်း တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် ပိုင်းခြေ သည် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဖီဘိုနာချီ ဟူသော လုပ်ဆောင်ချက်ကို န ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ဒဿမကိန်း တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
ပိုင်းဝေ / ပိုင်းခြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// The program starts here
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၀ မှ ၉ အထိ န အနေဖြင့် ရေတွက်၍ ဖီဘိုနာချီ ဟူသော လုပ်ဆောင်ချက်ကို န ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
"၅၀ ကြိမ်မြောက်" ဟု ပြသမည်။
ဖီဘိုနာချီ ဟူသော လုပ်ဆောင်ချက်ကို ၅၀ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
"ရွှေအချိုး" ဟု ပြသမည်။
အချိုး ဟူသော လုပ်ဆောင်ချက်ကို ၂၀ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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

- **`ဖီဘိုနာချီ`** keeps two numbers: `အဖြေ` (the current one, starting at 0) and
  `နောက်` (the next one, starting at 1). Each round of the [generic
  loop](../control-flow/generic-loops.md) moves both one step forward, so after n rounds,
  `အဖြေ` has the n-th number, and the function [returns
  it](../functions/results.md#returning-the-result).
- **`လာမည့်`** ("coming") is declared inside the block of the loop, so it is a new
  variable in each round.
- **`အချိုး`** ("ratio") [converts](../operators/conversion.md) both numbers to decimal
  numbers (floats) before it divides them, because integer division would drop the
  fraction and print `၁`.
- **The 50th number**, 12,586,269,025, is far too big for a 32-bit integer, but it fits
  with room to spare in a plain `ကိန်းပြည့်`, which has 64 bits.

## Hands on

- Print the 90th Fibonacci number, then the 93rd. What happens, and what is the reason?
  (See [Too big for the data kind](../operators/arithmetic.md#too-big-for-the-data-kind).)

Next: [The Collatz sequence](collatz.md).
