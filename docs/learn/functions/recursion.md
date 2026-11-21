---
title: Recursion
description: A function that calls itself, its base case, and what happens when it never stops.
---

# Recursion

A function can call itself, which is called *recursion*. It suits problems that contain a
smaller version of themselves, such as the *factorial* of n, written n!, which is
n × (n − 1) × … × 1. So 5! is 5 × 4!, and 4! is 4 × 3!, and so on down to 1! = 1.

## You will learn

- how a function calls itself
- the two parts that each recursive function needs
- what happens when a recursion does not stop

```zaka
// factorial
ဖက်တိုရီယယ် ဟူသော လုပ်ဆောင်ချက် သည်
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

A recursive call is a plain call, written the same as each other call, and nothing in the
function signature marks the function as recursive. Each recursive function needs two parts:

```text
အကယ်၍ <base case>လျှင် <answer> ကို ပြန်ပေးမည်။
… <name> ဟူသော လုပ်ဆောင်ချက်ကို <smaller argument> ဖြင့် လုပ်ဆောင်သော ရလဒ် … ကို ပြန်ပေးမည်။
```

1. A *base case*, which returns the answer without a call to itself: for n ≤ 1, the
   answer is 1.
2. A *recursive case*, which calls the function itself on a *smaller* problem: n − 1.

Each call works on a smaller n, so the calls reach the base case in the end and stop. Each
call also has its own parameters and its own variables, so the `န` of one call does not
change the `န` of another.

## Two calls at once

The Fibonacci numbers start with 0 and 1, and each next number is the sum of the two before
it: 0, 1, 1, 2, 3, 5, 8, … A recursive function can follow that rule word for word:

```zaka
// fibonacci
ဖီဘိုနာချီ ဟူသော လုပ်ဆောင်ချက် သည်
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

This version is simple to follow, but it is slow for a large n, because it works out the
same numbers again and again. The [Fibonacci numbers](../programs/fibonacci.md) program uses
a loop instead.

## Recursion without an end

When the base case is missing or the calls never reach it, the function keeps calling
itself. Zaka stops the program once more than 4096 calls are waiting to return at the same
time:

```zaka-stop
အဆုံးမဲ့ ဟူသော လုပ်ဆောင်ချက် သည်
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

## Details

- `zakac run --max-depth <calls>` sets a different limit in place of 4096, and a native
  program, built with `--backend c` or `--backend native` or run with `--jit`, counts
  calls in the same manner ([Recursion](../../manual/language/functions-calls.md#recursion)).
- Two functions can also call each other, such as one that tests for an even number and
  one that tests for an odd number, each through the other
  ([Recursion](../../manual/language/functions-calls.md#recursion)).

## Recap

- A function can call itself with a plain call.
- Each recursive function needs a base case, which returns without a call, and a recursive
  case, which calls the function on a smaller problem.
- Each call has its own parameters and variables.
- More than 4096 calls waiting at the same time stop the program.

## Hands on

- Write a recursive function for the sum 1 + 2 + … + n.
- Write a recursive function that counts the digits of a number (hint: n / 10 has one
  digit fewer).
- Remove the base case from `ဖက်တိုရီယယ်` and run it.

Next: [Compile time](compile-time.md).
