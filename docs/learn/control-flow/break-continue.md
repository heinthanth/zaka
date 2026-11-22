---
title: Break and continue
description: Leave a loop at once, or skip the rest of one round.
---

# Break and continue

Use `break` to exit a loop at once. Use `continue` to skip the rest of the current round
and start the next one. Zaka writes both as short Burmese sentences.

## You will learn

- how to leave a loop at once with `break`
- how to skip the rest of a round with `continue`
- how to search with a loop that runs until it finds an answer
- which loop `break` and `continue` act on when one loop is inside another

| Zaka | Reads as | What it does |
| --- | --- | --- |
| `ကွင်းဆက် ကို ရပ်မည်။` | "will stop the loop" | leaves the loop at once (`break`) |
| `နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်။` | "will skip to the next round" | skips the rest of this round (`continue`) |

```zaka
// print the numbers from 1, skipping multiples of 3, stopping after 10
၁ မှ ၁၀၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ ကိန်း % ၃ သည် ၀ နှင့် ညီလျှင် နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်။
အကယ်၍ ကိန်း သည် ၁၀ ထက် ကြီးလျှင် ကွင်းဆက် ကို ရပ်မည်။
ကိန်း ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
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

The loop counts up to 100, but `continue` skips 3, 6 and 9 before the loop prints them, and
`break` leaves the loop at 11. We most often write the two sentences as the inner part of
an `if`:

```text
အကယ်၍ <condition>လျှင် ကွင်းဆက် ကို ရပ်မည်။
အကယ်၍ <condition>လျှင် နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်။
```

| Zaka | Reads as |
| --- | --- |
| `ကွင်းဆက် ကို` | "the loop", the object of the verb |
| `ရပ်မည်` | "will stop" |
| `နောက်တစ်ကြိမ်ဆီသို့` | "to the next round" |
| `ကျော်မည်` | "will skip" |

## Searching with break

```text
မှန် ဖြစ်နေသရွေ့ <inner part>
```

A loop with `မှန်` (true) as its condition would run forever, so we add a `break` that
leaves the loop when it finds the answer. This is a common pattern for a search:

```zaka
// find the first power of 2 above 100
ကိန်းရှင် ဂဏန်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
မှန် ဖြစ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဂဏန်း ကို ဂဏန်း * ၂ ဟု ထားမည်။
အကယ်၍ ဂဏန်း သည် ၁၀၀ ထက် ကြီးလျှင် ကွင်းဆက် ကို ရပ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ဂဏန်း ကို ပြသမည်။
```

```text
၁၂၈
```

## Continue in a generic loop

In a [generic loop](generic-loops.md), `continue` still runs the `ပြီးလျှင်` update, so
the variable keeps moving:

```zaka
// print the even numbers below 10
ကိန်းရှင် ကိန်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ကာ ကိန်း သည် ၁၀ အောက် ငယ်နေသရွေ့
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ ကိန်း % ၂ သည် ၁ နှင့် ညီလျှင် နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်။
ကိန်း ကို ပြသမည်။
ပြီးလျှင် ကိန်း ကို ကိန်း + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
၀
၂
၄
၆
၈
```

In a while loop with no update, the same `continue` would skip the line that adds 1, and
the loop would never end.

## Loops inside loops

When one loop is inside another, `break` and `continue` act on the **innermost** loop
around them. Here the `break` leaves the inner loop when the column passes the row, while
the outer loop goes on with its next row:

```zaka
၁ မှ ၃ အထိ အတန်း အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ မှ ၃ အထိ အတိုင် အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ အတိုင် သည် အတန်း ထက် ကြီးလျှင် ကွင်းဆက် ကို ရပ်မည်။
အတန်း * အတိုင် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
"---" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
၁
---
၂
၄
---
၃
၆
၉
---
```

## Outside a loop

Both sentences must be inside a loop, so the compiler stops with an error when one of them
is not:

```zaka-error ZK0309
ကွင်းဆက် ကို ရပ်မည်။
```

```text
error[ZK0309]: "ကွင်းဆက် ကို ရပ်မည်" stands outside a loop
```

The same mistake with `နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်။` is the error ZK0310.

## Recap

- `ကွင်းဆက် ကို ရပ်မည်။` (`break`) leaves the loop at once.
- `နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်။` (`continue`) skips the rest of the round, and in a
  generic loop the update still runs.
- A loop with `မှန်` as its condition and a `break` inside it is a common search pattern.
- Both sentences act on the innermost loop around them.

## Hands on

- Print the first number above 1 that divides 91 with no remainder.
- In the times table from [Counted loops](counted-loops.md), add a `continue` that skips
  the cells where the row and the column are equal.

Next: [Functions](../functions/README.md).
