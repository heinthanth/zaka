---
title: While loops
description: Repeat a body as long as a condition stays true.
---

# While loops

A *loop* runs its body again and again. A *while loop* keeps going as long as a
condition is true. In Zaka: "as long as *condition*, do the following":
`<condition> နေသရွေ့ <body>`.

```zaka
ကိန်းရှင် ကိန်း သည် ကိန်းပြည့် ၁ ဖြစ်ပါစေ။

// as long as the number is less than 100, ...
ကိန်း သည် ၁၀၀ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်း ကို ပြသပါ။
ကိန်း ကို ကိန်း * ၂ ဟု ထားပါ။        // double it
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

"ပြီးပြီ" ဟု ပြသပါ။
```

```text
၁
၂
၄
၈
၁၆
၃၂
၆၄
ပြီးပြီ
```

Each round goes like this:

1. Test the condition. If it is false, the loop ends, and the program goes on after it.
2. Run the body.
3. Go back to step 1.

After 64 is printed, the number is doubled to 128. Then the test "128 is less than 100"
is false, so the loop ends and 128 is never printed.

| Zaka | Reads as |
| --- | --- |
| `နေသရွေ့` | "as long as"; joined to the condition's last verb (`ငယ်နေသရွေ့`) |
| `ဖြစ်နေသရွေ့` | "as long as it is so"; a separate word, used after a variable or a full sentence |

## One-statement bodies

The body can be a single statement:

```zaka
ကိန်းရှင် ကျန် သည် ကိန်းပြည့် ၅ ဖြစ်ပါစေ။          // remaining
ကျန် သည် ၀ ထက် ကြီးနေသရွေ့ ကျန် ကို ကျန် - ၁ ဟု ထားပါ။
ကျန် ကို ပြသပါ။
```

```text
၀
```

## A boolean as the condition

```zaka
ကိန်းရှင် ဆက်ရှာ သည် မှန်မှားတန်ဖိုး မှန် ဖြစ်ပါစေ။     // keep searching
ကိန်းရှင် အကြိမ် သည် ကိန်းပြည့် ၀ ဖြစ်ပါစေ။            // round
ဆက်ရှာ ဖြစ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
အကြိမ် ကို အကြိမ် + ၁ ဟု ထားပါ။
// keep going while round × round is below 50
ဆက်ရှာ ကို အကြိမ် * အကြိမ် သည် ၅၀ အောက် ငယ်သည် ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
အကြိမ် ကို ပြသပါ။
```

```text
၈
```

The loop stops in the round where `အကြိမ် * အကြိမ်` first reaches 50 or more: 8 × 8 = 64.

## Loops that never end

If the condition never becomes false, the loop runs forever. Make sure the body changes
something the condition depends on. In the Explorer, press **Stop** to end a runaway
program. In a terminal, press `Ctrl`+`C`.

## Try it

- Print the numbers 10 down to 1 with a while loop.
- Starting from 1000, keep halving the number while it is greater than 1. How many
  halvings does it take?

Next: [Counted loops](counted-loops.md).
