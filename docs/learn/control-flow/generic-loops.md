---
title: Generic loops
description: A loop with its own variable, a test, and an update step.
---

# Generic loops

A generic loop puts the three parts that most loops share in one place: a variable that
belongs to the loop, the test, and the update that runs after each round. Other languages
call this a `for` loop, as in C's `for (i = 0; i < 5; i++)`.

## You will learn

- how to write a loop with a variable, a test and an update
- that the update must be the last sentence of the loop
- that the loop's variable is gone after the loop
- which loop to pick for a task

```zaka
// "variable i, an integer whose value is 0; as long as i < 5, do the following"
ကိန်းရှင် ကိန်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ကာ ကိန်း သည် ၅ အောက် ငယ်နေသရွေ့
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်း * ၁၀ ကို ပြသမည်။
ပြီးလျှင် ကိန်း ကို ကိန်း + ၁ ဟု ထားမည်။ // when done: add 1
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
၀
၁၀
၂၀
၃၀
၄၀
```

The first line starts like a [declaration](../variables/README.md), but it ends with
`ဖြစ်ကာ` ("being so, and") in place of `ဖြစ်ပါစေ`, and then a
[while loop](while.md) follows. We can use this pattern:

```text
ကိန်းရှင် <name> သည် <kind> တစ်ခု ဖြစ်၍ တန်ဖိုးသည် <value> ဖြစ်ကာ <condition>နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
<sentences>
ပြီးလျှင် <update>။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

| Part | Zaka | Runs |
| --- | --- | --- |
| setup | `ကိန်းရှင် ကိန်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ကာ` | once, before the loop |
| test | `ကိန်း သည် ၅ အောက် ငယ်နေသရွေ့` | before each round |
| inner part | the sentences in the block | in each round |
| update | `ပြီးလျှင် ...` ("when done, ...") | after each round |

The update is optional, and without it the loop is a while loop with its own variable.

## Other updates, other data kinds

The update can be a sentence that fits the task, not just "add one", and the variable can
have a data kind (type) other than `ကိန်းပြည့်`. Here the variable doubles in each round,
and it is an 8-bit unsigned integer:

```zaka
ကိန်းရှင် နှစ်ထပ် သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ကာ နှစ်ထပ် သည်
၁၂၈ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
နှစ်ထပ် ကို ပြသမည်။
ပြီးလျှင် နှစ်ထပ် ကို နှစ်ထပ် * ၂ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
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

The loop's variable is gone after the block closer, so reading it after the loop is an
error:

```zaka-error ZK0301
ကိန်းရှင် ကိန်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ကာ ကိန်း သည် ၃ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပြီးလျှင် ကိန်း ကို ကိန်း + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ကိန်း ကို ပြသမည်။
```

```text
error[ZK0301]: "ကိန်း" is not declared
```

The update changes the variable, so we declare it with `ကိန်းရှင်` (variable), never with
`ကိန်းသေ` (constant), which is the error ZK0225.

## The update goes last

The `ပြီးလျှင်` update must be the last sentence of the block, so the compiler stops with
an error when another sentence comes after it:

```zaka-error ZK0213
ကိန်းရှင် ကိန်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ကာ ကိန်း သည် ၃ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပြီးလျှင် ကိန်း ကို ကိန်း + ၁ ဟု ထားမည်။
ကိန်း ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
error[ZK0213]: the "ပြီးလျှင်" update must be the last statement of the loop
```

A `ပြီးလျှင်` line outside a generic loop is the error ZK0212.

## Which loop to use

| Task | Loop |
| --- | --- |
| each number in a range | [counted loop](counted-loops.md) |
| until something happens | [while loop](while.md) |
| a variable that changes in an unusual pattern | generic loop |

## Details

- **Other joining words.** `ဖြစ်ပြီး` or `ဖြစ်၍` works in place of `ဖြစ်ကာ`, and the setup
  can write the [value with its data kind](../variables/README.md):
  `ကိန်းရှင် ကိန်း သည် ၀ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပြီး ...`
  ([Generic loops](../../manual/language/control-flow.md#generic-loops)).
- **A block alone.** A generic loop needs a block as its inner part, so a single sentence
  after `နေသရွေ့` is the error ZK0201 ([Generic loops](../../manual/language/control-flow.md#generic-loops)).
- **Continue still updates.** `နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်။` skips the rest of the round
  but still runs the update ([Break and continue](break-continue.md#continue-in-a-generic-loop)).

## Recap

- A generic loop has a setup, a test, an inner part and an update.
- The setup is a declaration that ends in `ဖြစ်ကာ`, and the test is a while condition.
- The `ပြီးလျှင်` update must be the last sentence of the block.
- The loop's variable is gone after the loop.

## Hands on

- Print 100, 90, 80, ... 0 with a generic loop.
- Print the powers of 3 below 1000.

Next: [Break and continue](break-continue.md).
