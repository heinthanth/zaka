---
title: The Collatz sequence
description: A simple rule, an unsolved puzzle, and two functions that share a helper.
---

# The Collatz sequence

Pick any whole number. If it is even, halve it. If it is odd, triple it and add 1.
Repeat. Every number anyone has tried eventually reaches 1, but no one has proved this
always happens. It is one of the most famous unsolved problems in mathematics.

Starting from 27, the sequence takes 111 steps and climbs as high as 9,232 before it
falls to 1.

```zaka
/// The next number: half of an even number, or 3n + 1 for an odd one.
လုပ်ဆောင်ချက် နောက်တစ်ဆင့် သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
အကယ်၍ န % ၂ သည် ၀ နှင့် ညီလျှင် န / ၂ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
န * ၃ + ၁ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

/// How many steps from n down to 1.
လုပ်ဆောင်ချက် အဆင့်အရေအတွက် သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
န သည် ၁ နှင့် မညီနေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
န ကို လုပ်ဆောင်ချက် နောက်တစ်ဆင့် ကို န တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဟု ထားပါ။
ရလဒ် ကို ရလဒ် + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

/// The highest number reached on the way down to 1.
လုပ်ဆောင်ချက် အမြင့်ဆုံး သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
ရလဒ် ကို န ဟု ထားပါ။
န သည် ၁ နှင့် မညီနေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
န ကို လုပ်ဆောင်ချက် နောက်တစ်ဆင့် ကို န တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဟု ထားပါ။
အကယ်၍ န သည် ရလဒ် ထက် ကြီးလျှင် ရလဒ် ကို န ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

// The main program
ကိန်းသေ စမ်းကိန်း သည် ကိန်းပြည့် ၂၇ ဖြစ်ပါစေ။
"အဆင့် အရေအတွက်" ဟု ပြသပါ။
လုပ်ဆောင်ချက် အဆင့်အရေအတွက် ကို စမ်းကိန်း တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
"အမြင့်ဆုံး" ဟု ပြသပါ။
လုပ်ဆောင်ချက် အမြင့်ဆုံး ကို စမ်းကိန်း တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
```

```text
အဆင့် အရေအတွက်
၁၁၁
အမြင့်ဆုံး
၉၂၃၂
```

## How it works

- **`နောက်တစ်ဆင့်`** ("next step") holds the rule in one place. It
  [returns early](../functions/results.md#returning-the-result) for even numbers.
- **`အဆင့်အရေအတွက်`** ("number of steps") and **`အမြင့်ဆုံး`** ("highest") both loop
  until `န` reaches 1, and both call `နောက်တစ်ဆင့်`. Sharing the helper means the rule is
  written once.
- `အဆင့်အရေအတွက်` counts in `ရလဒ်`, which starts at 0. `အမြင့်ဆုံး` starts `ရလဒ်` at `န`
  and raises it whenever a bigger number shows up.

## Try it

- Find the number below 100 that takes the most steps. (Use a counted loop and keep the
  best so far in two variables.)
