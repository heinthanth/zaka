---
title: The Collatz sequence
description: A short rule, an unsolved puzzle, and two functions that share a helper.
---

# The Collatz sequence

We pick a whole number, and if it is even, we halve it, while if it is odd, we triple it
and add 1. Then we repeat. Each number that people have tried reaches 1 in the end, but
no one has proved that this is true for all numbers, which makes it one of the most famous
unsolved problems in mathematics.

Starting from 27, the sequence takes 111 steps and climbs as high as 9,232 before it
falls to 1. This program works out both numbers.

```zaka
/// The next number: half of an even number, or 3n + 1 for an odd one.
နောက်တစ်ဆင့် ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န % ၂ သည် ၀ နှင့် ညီလျှင် န / ၂ ကို ပြန်ပေးမည်။
န * ၃ + ၁ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

/// The number of steps from n down to 1.
အဆင့်အရေအတွက် ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ၀ ဖြစ်ပါစေ။
န သည် ၁ နှင့် မညီနေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို နောက်တစ်ဆင့် ဟူသော လုပ်ဆောင်ချက်ကို န ဖြင့် လုပ်ဆောင်သော ရလဒ် ဟု ထားမည်။
အဖြေ ကို အဖြေ + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

/// The highest number reached on the path down to 1.
အမြင့်ဆုံး ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ၀ ဖြစ်ပါစေ။
အဖြေ ကို န ဟု ထားမည်။
န သည် ၁ နှင့် မညီနေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို နောက်တစ်ဆင့် ဟူသော လုပ်ဆောင်ချက်ကို န ဖြင့် လုပ်ဆောင်သော ရလဒ် ဟု ထားမည်။
အကယ်၍ န သည် အဖြေ ထက် ကြီးလျှင် အဖြေ ကို န ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// The program starts here
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ စမ်းကိန်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၇ ဖြစ်ပါစေ။
"အဆင့် အရေအတွက်" ဟု ပြသမည်။
အဆင့်အရေအတွက် ဟူသော လုပ်ဆောင်ချက်ကို စမ်းကိန်း ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
"အမြင့်ဆုံး" ဟု ပြသမည်။
အမြင့်ဆုံး ဟူသော လုပ်ဆောင်ချက်ကို စမ်းကိန်း ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အဆင့် အရေအတွက်
၁၁၁
အမြင့်ဆုံး
၉၂၃၂
```

## How it works

- **`နောက်တစ်ဆင့်`** ("next step") has the rule, in one place. For an even number, it
  [returns at once](../functions/results.md#returning-the-result), so the last sentence
  runs for odd numbers alone.
- **`အဆင့်အရေအတွက်`** ("number of steps") and **`အမြင့်ဆုံး`** ("highest") both loop
  until `န` reaches 1, and both call `နောက်တစ်ဆင့်`. Because the two share the helper,
  we write the rule just once.
- **`အဆင့်အရေအတွက်`** counts in `အဖြေ`, which starts at 0 and is returned when the loop
  ends. **`အမြင့်ဆုံး`** starts `အဖြေ` at `န` and raises it each time a bigger number
  comes up.
- **Each function changes its parameter `န`**, which is fine, because the parameter is a
  local duplicate (copy) and the caller's `စမ်းကိန်း` does not change.

## Hands on

- Find the number below 100 that takes the most steps. (Use a counted loop, and keep the
  best number so far and its steps in two variables.)

Next: [Scores](scores.md).
