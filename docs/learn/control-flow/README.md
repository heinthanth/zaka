---
title: Control flow
description: Make decisions and repeat work.
---

# Control flow

So far, each program has run straight from its first line to its last. Real programs also
need to **choose**, as in "when the score is 40 or more, print *pass*", and to **repeat**,
as in "print the numbers 1 to 100". The order in which a program runs its sentences
(statements) is called control flow, and this chapter shows the sentences that change it.

## You will learn

- how a program chooses between sentences with `if` and `else`
- how a program repeats sentences with a loop
- how to give a choice or a loop one sentence or a whole block

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၃ ဖြစ်ပါစေ။

// choose: if ... then ... otherwise ...
အကယ်၍ အမှတ် သည် ၄၀ အောက် မငယ်လျှင် "အောင်" ဟု ပြသမည်။
သို့မဟုတ်ပါက "ကျ" ဟု ပြသမည်။

// repeat: count from 1 to 3
၁ မှ ၃ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အောင်
၁
၂
၃
```

The first two sentences print `အောင်` ("pass") because 73 is not less than 40, and the
last sentence counts from 1 to 3 and prints each number.

## The inner part

Each sentence in this chapter runs an *inner part*: the sentences that it chooses or
repeats. The inner part can be one sentence, written right after the condition, or a
[block](../variables/scope.md) of several sentences:

```text
<condition or loop> <one sentence>။

<condition or loop> အောက်ပါတို့ကို လုပ်ဆောင်မည်။
<sentences>
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

| Zaka | Reads as |
| --- | --- |
| `အောက်ပါတို့ကို လုပ်ဆောင်မည်။` | "will do the following", which opens the block |
| `လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။` | "the tasks are done", the block closer (end of a block) |

## In this chapter

| Page | What it does |
| --- | --- |
| [If and else](if-else.md) | runs sentences when a condition is true |
| [While loops](while.md) | repeats sentences while a condition is true |
| [Counted loops](counted-loops.md) | repeats sentences once for each number in a range |
| [Generic loops](generic-loops.md) | a loop with its own variable, test and update |
| [Break and continue](break-continue.md) | leaves a loop at once, or skips to its next round |

## Recap

- Control flow is the order in which a program runs its sentences.
- `အကယ်၍` chooses, and loops repeat.
- The inner part of a choice or a loop is one sentence or a block.

Next: [If and else](if-else.md).
