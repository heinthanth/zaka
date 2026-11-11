---
title: Counted loops
description: Run statements once for each number in a range.
---

# Counted loops

A common task is to repeat something for each number from one value to another:
"for each number from 1 to 5, print it". A *counted loop* does just that. It reads
"from 1 to 5, counting as *n*, do ...":

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ မှ ၅ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁
၂
၃
၄
၅
```

| Zaka | Reads as | Part |
| --- | --- | --- |
| `၁ မှ` | "from 1" | the first value |
| `၅ အထိ` | "up to 5" | the last value; it is included |
| `ကိန်း အနေဖြင့်` | "as ကိန်း" | the name of the counter variable |
| `ရေတွက်၍` | "counting" | the loop verb |
| `ကိန်း ကို ပြသမည်။` | | the inner part |

The loop creates the counter variable `ကိန်း` itself. Each round, it holds the next
number. The inner part can read it but cannot change it.

## Counting down, and steps

`ရေတွက်၍` counts in whichever direction reaches the last value. Add `<n> စီ` ("n at a
time") to take bigger steps:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၅ မှ ၁ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသမည်။          // counts down
"---" ဟု ပြသမည်။
၀ မှ ၁၀၀ အထိ ၂၅ စီ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသမည်။    // 25 at a time
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၅
၄
၃
၂
၁
---
၀
၂၅
၅၀
၇၅
၁၀၀
```

Two other verbs fix the direction:

| Verb | Reads as | Direction |
| --- | --- | --- |
| `ရေတွက်၍` | "counting" | toward the last value, up or down |
| `တိုး၍` | "increasing" | never down |
| `လျှော့၍` | "decreasing" | never up |

With `တိုး၍` or `လျှော့၍`, the loop runs zero times if the first value starts past the
last one:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁၀ မှ ၀ အထိ ၂ စီ ကိန်း အနေဖြင့် လျှော့၍ ကိန်း ကို ပြသမည်။   // 10, 8, 6, 4, 2, 0
၅ မှ ၁ အထိ ကိန်း အနေဖြင့် တိုး၍ ကိန်း ကို ပြသမည်။           // never runs
"ပြီး" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၀
၈
၆
၄
၂
၀
ပြီး
```

## Adding up

A loop often builds a total, one round at a time:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စုစုပေါင်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။       // total
၁ မှ ၁၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ စုစုပေါင်း ကို စုစုပေါင်း + ကိန်း ဟု ထားမည်။
စုစုပေါင်း ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၅၅
```

## Loops inside loops

A loop can hold another loop. This one prints a small times table. For each row, the
inner loop runs once for each column:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ မှ ၃ အထိ အတန်း အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။     // row
၁ မှ ၃ အထိ အတိုင် အနေဖြင့် ရေတွက်၍ အတန်း * အတိုင် ကို ပြသမည်။      // column
"---" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁
၂
၃
---
၂
၄
၆
---
၃
၆
၉
---
```

## Rules

- The first value, last value and step must be integers. These can be variables or sums:
  `အစ မှ အဆုံး - ၁ အထိ ...`.
- The step must be greater than zero. A zero step written as a variable stops the
  program when the loop starts.
- Put a space before `အနေဖြင့်`. `ကိန်းအနေဖြင့်` is read as one name.
- The counter cannot be changed in the inner part, and it does not exist after the loop:

```zaka-error ZK0307
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ မှ ၁၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ၀ ဟု ထားမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0307]: "ကိန်း" is a counted-loop variable and cannot be assigned
```

## Hands on

- Print the even numbers from 2 to 20.
- Work out 1 × 2 × 3 × ... × 10 with a counted loop.

Next: [Generic loops](generic-loops.md).
