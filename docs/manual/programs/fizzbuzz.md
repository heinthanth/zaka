---
title: FizzBuzz
description: A classic counting game, built from functions, booleans and if/else.
---

# FizzBuzz

FizzBuzz is a counting game. Count up from 1. Say **Fizz** for numbers that divide by
3, **Buzz** for numbers that divide by 5, **FizzBuzz** for numbers that divide by both,
and the number itself otherwise.

```zaka
/// Does n divide by d with nothing left over?
လုပ်ဆောင်ချက် ပြတ်သလား သည်
ကိန်းပြည့် န၊ ကိန်းပြည့် ဆခွဲ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး ရလဒ် ကို ပြန်ပေးမည်။
န % ဆခွဲ သည် ၀ နှင့် ညီသည် ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

/// Print Fizz, Buzz, FizzBuzz, or the number itself.
လုပ်ဆောင်ချက် FizzBuzz သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် သုံးပြတ် သည် မှန်မှားတန်ဖိုး လုပ်ဆောင်ချက် ပြတ်သလား ကို န၊ ၃ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် ငါးပြတ် သည် မှန်မှားတန်ဖိုး လုပ်ဆောင်ချက် ပြတ်သလား ကို န၊ ၅ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
အကယ်၍ သုံးပြတ် ဖြစ်၍ ငါးပြတ် ဖြစ်လျှင် "FizzBuzz" ဟု ပြသပါ။
ထိုသို့မဟုတ်ဘဲ သုံးပြတ် ဖြစ်လျှင် "Fizz" ဟု ပြသပါ။
ထိုသို့မဟုတ်ဘဲ ငါးပြတ် ဖြစ်လျှင် "Buzz" ဟု ပြသပါ။
သို့မဟုတ်ပါက န ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

// The main program
ကိန်းသေ အဆုံး သည် ကိန်းပြည့် ၁၅ ဖြစ်ပါစေ။
၁ မှ အဆုံး အထိ ကိန်း အနေဖြင့် ရေတွက်၍ လုပ်ဆောင်ချက် FizzBuzz ကို ကိန်း တို့ဖြင့် လုပ်ဆောင်ပါ။
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

- **`ပြတ်သလား`** ("does it divide?") returns a [boolean](../values/booleans.md): whether
  `န % ဆခွဲ` is zero. Its name is a question, which keeps the code readable.
- **`FizzBuzz`** stores the two answers in variables `သုံးပြတ်` ("divides by 3") and
  `ငါးပြတ်` ("divides by 5"), then picks one of four outputs with an
  [if / else-if / else](../control-flow/if-else.md) chain.
- The **test order matters**. "Both" must come first. 15 divides by 3, so if the Fizz
  test came first, 15 would print Fizz.
- The **main program** is two lines: a [constant](../variables/constants.md) for where to
  stop, and a [counted loop](../control-flow/counted-loops.md) that calls `FizzBuzz` once
  per number.

## Try it

- Change `အဆုံး` to 100.
- Add a rule: say **Bazz** for numbers that divide by 7.
