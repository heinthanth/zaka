---
title: FizzBuzz
description: A classic counting game, built from functions, true/false values and if/else.
---

# FizzBuzz

FizzBuzz is a counting game: we count up from 1 and call out **Fizz** for multiples of 3,
**Buzz** for multiples of 5, **FizzBuzz** for multiples of both, and the number itself
for all the rest. This program runs the game up to 15.

```zaka
/// Does d divide n with nothing left over?
ပြတ်သလား ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ဆခွဲ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
န % ဆခွဲ သည် ၀ နှင့် ညီသည် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

/// Print Fizz, Buzz, FizzBuzz, or the number itself.
FizzBuzz ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် သုံးပြတ် သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ပြတ်သလား ဟူသော လုပ်ဆောင်ချက်ကို န နှင့် ၃ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် ငါးပြတ် သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ပြတ်သလား ဟူသော လုပ်ဆောင်ချက်ကို န နှင့် ၅ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
အကယ်၍ သုံးပြတ် ဖြစ်၍ ငါးပြတ် ဖြစ်လျှင် "FizzBuzz" ဟု ပြသမည်။
ထိုသို့မဟုတ်ဘဲ သုံးပြတ် ဖြစ်လျှင် "Fizz" ဟု ပြသမည်။
ထိုသို့မဟုတ်ဘဲ ငါးပြတ် ဖြစ်လျှင် "Buzz" ဟု ပြသမည်။
သို့မဟုတ်ပါက န ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// The program starts here
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ အဆုံး သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၅ ဖြစ်ပါစေ။
၁ မှ အဆုံး အထိ ကိန်း အနေဖြင့် ရေတွက်၍ FizzBuzz ဟူသော လုပ်ဆောင်ချက်ကို ကိန်း ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁
၂
Fizz
၄
Buzz
Fizz
၇
၈
Fizz
Buzz
၁၁
Fizz
၁၃
၁၄
FizzBuzz
```

## How it works

- **`ပြတ်သလား`** ("does it divide?") returns a [true/false value](../values/booleans.md)
  (boolean): whether `န % ဆခွဲ`, the remainder, is zero. Its name is a question, so a
  call to it sounds like one.
- **`FizzBuzz`** keeps the two answers in the variables `သုံးပြတ်` ("3 divides it") and
  `ငါးပြတ်` ("5 divides it"), then picks one of four outputs with an [if / else-if /
  else](../control-flow/if-else.md) chain.
- **The order of the tests matters.** The test for both must come first: 3 divides 15, so
  if the Fizz test came first, 15 would print Fizz.
- **The program start block** has two sentences: a [constant](../variables/constants.md)
  for the last number, and a [counted loop](../control-flow/counted-loops.md) that calls
  `FizzBuzz` once for each number.

## Hands on

- Change `အဆုံး` to 100.
- Add a rule: print **Bazz** for multiples of 7.

Next: [Greatest common divisor](gcd.md).
