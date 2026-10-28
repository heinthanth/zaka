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
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
// the smallest case: 1! = 1 (and 0! = 1)
အကယ်၍ န သည် ၁ ထက် မကြီးလျှင် ၁ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
// all other cases: n × (n - 1)!
န * လုပ်ဆောင်ချက် ဖက်တိုရီယယ် ကို န - ၁ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

၁ မှ ၆ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ လုပ်ဆောင်ချက် ဖက်တိုရီယယ် ကို ကိန်း တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
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
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၂ အောက် ငယ်လျှင် န ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
ရလဒ် ကို လုပ်ဆောင်ချက် ဖီဘိုနာချီ ကို န - ၁ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် + လုပ်ဆောင်ချက် ဖီဘိုနာချီ ကို န - ၂ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

၀ မှ ၁၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ လုပ်ဆောင်ချက် ဖီဘိုနာချီ ကို ကိန်း တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
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
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် အဆုံးမဲ့ ကို န + ၁ တို့ဖြင့် လုပ်ဆောင်ပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

လုပ်ဆောင်ချက် အဆုံးမဲ့ ကို ၁ တို့ဖြင့် လုပ်ဆောင်ပါ။
```

```text
the program stopped: more than 4096 nested calls
```

## Hands on

- Write a recursive function for the sum 1 + 2 + ... + n.
- Write a recursive function that counts the digits of a number (hint: n / 10 has one
  digit fewer).

Next: [Pointers and the heap](../pointers/README.md).
