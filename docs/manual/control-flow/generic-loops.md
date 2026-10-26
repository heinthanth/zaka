---
title: Generic loops
description: A loop with its own counter, a test, and an update step.
---

# Generic loops

A *generic loop* puts the three parts of many loops in one place: a variable that
belongs to the loop, the test, and the update to run after each round. Other languages
call this a `for` loop.

```zaka
// "variable i being integer 0, as long as i < 5, do the following"
ကိန်းရှင် ကိန်း သည် ကိန်းပြည့် ၀ ဖြစ်ပြီး ကိန်း သည် ၅ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်း * ၁၀ ကို ပြသပါ။
ပြီးလျှင် ကိန်း ကို ကိန်း + ၁ ဟု ထားပါ။      // when done: add 1
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
```

```text
၀
၁၀
၂၀
၃၀
၄၀
```

The first line starts like a [declaration](../variables/README.md). `ဖြစ်ပြီး` ("being
... and") takes the place of `ဖြစ်ပါစေ` and turns it into a loop.

| Part | Zaka | Runs |
| --- | --- | --- |
| setup | `ကိန်းရှင် ကိန်း သည် ကိန်းပြည့် ၀ ဖြစ်ပြီး` | once, before the loop |
| test | `ကိန်း သည် ၅ အောက် ငယ်နေသရွေ့` | before every round |
| body | the statements in the block | every round |
| update | `ပြီးလျှင် ...` ("when done, ...") | after every round |

The update must be the **last** statement in the block. It is optional; without it, the
loop is a [while loop](while.md) with its own variable.

## Any update, any kind

The update can be any statement, not only "add one". Here the counter doubles, and it
is an 8-bit unsigned integer:

```zaka
ကိန်းရှင် နှစ်ထပ် သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် ၁ ဖြစ်ပြီး နှစ်ထပ် သည် ၁၂၈ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
နှစ်ထပ် ကို ပြသပါ။
ပြီးလျှင် နှစ်ထပ် ကို နှစ်ထပ် * ၂ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
```

```text
၁
၂
၄
၈
၁၆
၃၂
၆၄
```

## The variable belongs to the loop

The loop's variable exists only inside the loop:

```zaka-error ZK0301
ကိန်းရှင် ကိန်း သည် ကိန်းပြည့် ၀ ဖြစ်ပြီး ကိန်း သည် ၃ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ပြီးလျှင် ကိန်း ကို ကိန်း + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
ကိန်း ကို ပြသပါ။
```

```text
error[ZK0301]: "ကိန်း" is not declared
```

## The update goes last

```zaka-error ZK0213
ကိန်းရှင် ကိန်း သည် ကိန်းပြည့် ၀ ဖြစ်ပြီး ကိန်း သည် ၃ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ပြီးလျှင် ကိန်း ကို ကိန်း + ၁ ဟု ထားပါ။
ကိန်း ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
```

```text
error[ZK0213]: the "ပြီးလျှင်" update must be the last statement of the loop
```

## Which loop should I use?

| Situation | Loop |
| --- | --- |
| each number in a range | [counted loop](counted-loops.md) |
| until something happens | [while loop](while.md) |
| a counter that changes in an unusual way | generic loop |

## Try it

- Print 100, 90, 80, ... 0 with a generic loop.
- Print the powers of 3 below 1000.

Next: [Break and continue](break-continue.md).
