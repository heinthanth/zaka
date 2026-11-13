---
title: Recursion
description: A function that calls itself.
---

# Recursion

A function can call itself. This is called *recursion*. It suits problems that contain
a smaller version of themselves. The *factorial* of n, written n!, is
n × (n − 1) × ... × 1. So 5! is 5 × 4!, and 4! is 4 × 3!, and so on down to 1! = 1.

```zaka
// factorial
လုပ်ဆောင်ချက် ဖက်တိုရီယယ် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
// the smallest case: 1! = 1 (and 0! = 1)
အကယ်၍ န သည် ၁ ထက် မကြီးလျှင် ၁ ကို ပြန်ပေးမည်။
// all other cases: n × (n - 1)!
န * ဖက်တိုရီယယ် ဟူသော လုပ်ဆောင်ချက်ကို န - ၁ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ မှ ၆ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ဖက်တိုရီယယ် ဟူသော လုပ်ဆောင်ချက်ကို ကိန်း ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁
၂
၆
၂၄
၁၂၀
၇၂၀
```

Each recursive function needs two parts:

1. **A base case** that gives the answer without calling itself: `n ≤ 1` gives 1.
2. **A recursive case** that calls itself on a *smaller* problem: n − 1.

Each call works on a smaller n, so the calls reach the base case in the end and stop.

## Two calls at once

The Fibonacci numbers start 0, 1, and each next number is the sum of the two before it:
0, 1, 1, 2, 3, 5, 8, ... A recursive function can follow that rule word for word:

```zaka
// fibonacci
လုပ်ဆောင်ချက် ဖီဘိုနာချီ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၂ အောက် ငယ်လျှင် န ကို ပြန်ပေးမည်။
(ဖီဘိုနာချီ ဟူသော လုပ်ဆောင်ချက်ကို န - ၁ ဖြင့် လုပ်ဆောင်သော ရလဒ်) + ဖီဘိုနာချီ ဟူသော လုပ်ဆောင်ချက်ကို န - ၂ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၀ မှ ၁၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ဖီဘိုနာချီ ဟူသော လုပ်ဆောင်ချက်ကို ကိန်း ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
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
၅၅
```

This version is simple to read but slow for large n, because it works out the same
numbers again and again. The [Fibonacci numbers](../programs/fibonacci.md) program uses
a loop instead.

## Recursion without an end

If the base case is missing or never reached, the function calls itself forever. Zaka
stops a program after 4096 calls that are still waiting to return:

```zaka-stop
လုပ်ဆောင်ချက် အဆုံးမဲ့ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အဆုံးမဲ့ ဟူသော လုပ်ဆောင်ချက်ကို န + ၁ ဖြင့် လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အဆုံးမဲ့ ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: more than 4096 nested calls
```

## Hands on

- Write a recursive function for the sum 1 + 2 + ... + n.
- Write a recursive function that counts the digits of a number (hint: n / 10 has one
  digit fewer).

Next: [Calling C](c.md).
