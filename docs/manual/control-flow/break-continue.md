---
title: Break and continue
description: Leave a loop at once, or skip the rest of one round.
---

# Break and continue

Two statements change how a loop runs from inside the loop itself:

| Zaka | Reads as | Does |
| --- | --- | --- |
| `ကွင်းဆက် ကို ရပ်ပါ။` | "please stop the loop" | leaves the loop at once (*break*) |
| `နောက်တစ်ကြိမ်ဆီသို့ ကျော်ပါ။` | "please skip to the next round" | skips the rest of this round (*continue*) |

```zaka
// print the numbers from 1, skipping multiples of 3, stopping after 10
၁ မှ ၁၀၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
အကယ်၍ ကိန်း % ၃ သည် ၀ နှင့် ညီလျှင် နောက်တစ်ကြိမ်ဆီသို့ ကျော်ပါ။
အကယ်၍ ကိန်း သည် ၁၀ ထက် ကြီးလျှင် ကွင်းဆက် ကို ရပ်ပါ။
ကိန်း ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
```

```text
၁
၂
၄
၅
၇
၈
၁၀
```

## Searching with break

A loop that runs "forever" with `မှန်` (true) as its condition, plus a `break` when the
answer is found, is a common search pattern:

```zaka
// find the first power of 2 above 100
ကိန်းရှင် ဂဏန်း သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
မှန် ဖြစ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ဂဏန်း ကို ဂဏန်း * ၂ ဟု ထားပါ။
အကယ်၍ ဂဏန်း သည် ၁၀၀ ထက် ကြီးလျှင် ကွင်းဆက် ကို ရပ်ပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
ဂဏန်း ကို ပြသပါ။
```

```text
၁၂၈
```

## Continue in a generic loop

In a [generic loop](generic-loops.md), `continue` still runs the `ပြီးလျှင်` update, so
the counter keeps moving:

```zaka
// print the even numbers below 10
ကိန်းရှင် ကိန်း သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ကာ ကိန်း သည် ၁၀ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
အကယ်၍ ကိန်း % ၂ သည် ၁ နှင့် ညီလျှင် နောက်တစ်ကြိမ်ဆီသို့ ကျော်ပါ။
ကိန်း ကို ပြသပါ။
ပြီးလျှင် ကိန်း ကို ကိန်း + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
```

```text
၀
၂
၄
၆
၈
```

## Rules

- Both statements act on the **innermost** loop around them.
- Both must be inside a loop:

```zaka-error ZK0309
ကွင်းဆက် ကို ရပ်ပါ။
```

```text
error[ZK0309]: "ကွင်းဆက် ကို ရပ်ပါ" stands outside a loop
```

## Hands on

- Print the first number above 1 that divides 91 with no remainder.
- In the times table from [Counted loops](counted-loops.md), add a `break` to the inner
  loop. Which loop does it stop?

Next: [Functions](../functions/README.md).
