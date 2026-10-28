---
title: Control flow
description: Make decisions and repeat work.
---

# Control flow

So far, each program has run straight from the first line to the last. Real programs
need to **choose** ("if the score is 40 or more, print *pass*") and to **repeat**
("print the numbers 1 to 100"). The order in which statements run is called *control flow*.

```zaka
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၇၃ ဖြစ်ပါစေ။

// choose: if ... then ... otherwise ...
အကယ်၍ အမှတ် သည် ၄၀ အောက် မငယ်လျှင် "အောင်" ဟု ပြသပါ။
သို့မဟုတ်ပါက "ကျ" ဟု ပြသပါ။

// repeat: count from 1 to 3
၁ မှ ၃ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသပါ။
```

```text
အောင်
၁
၂
၃
```

## In this chapter

| Page | Does |
| --- | --- |
| [If and else](if-else.md) | run statements when a condition is true |
| [While loops](while.md) | repeat while a condition holds |
| [Counted loops](counted-loops.md) | repeat once for each number in a range |
| [Generic loops](generic-loops.md) | a loop with its own counter, test and update |
| [Break and continue](break-continue.md) | leave a loop at once, or skip to its next round |

Each of these runs an *inner part*: either one statement or
a [block](../variables/scope.md) (`အောက်ပါတို့ကို လုပ်ဆောင်ပါ။ ... လုပ်ဆောင်စရာများ ပြီးပါပြီး။`).

Next: [If and else](if-else.md).
