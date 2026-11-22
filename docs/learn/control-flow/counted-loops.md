---
title: Counted loops
description: Run sentences once for each number in a range.
---

# Counted loops

A common task is to repeat something once for each number from one value to another, as
in "for each number from 1 to 5, print it". A counted loop (a `for` loop over a range)
does this, and in Zaka we write it as "from 1 to 5, counting as *n*, do the following".

## You will learn

- how to repeat sentences once for each number in a range
- how to count down, and how to count in bigger steps
- how to build a total with a loop
- how to put one loop inside another

```zaka
၁ မှ ၅ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသမည်။
```

```text
၁
၂
၃
၄
၅
```

To count through a range, we use this pattern:

```text
<first> မှ <last> အထိ <name> အနေဖြင့် ရေတွက်၍ <inner part>
```

| Zaka | Reads as | Part |
| --- | --- | --- |
| `၁ မှ` | "from 1" | the first value |
| `၅ အထိ` | "up to 5" | the last value, which the loop includes |
| `ကိန်း အနေဖြင့်` | "as `ကိန်း`" | the name of the counter variable |
| `ရေတွက်၍` | "counting" | the loop verb |
| `ကိန်း ကို ပြသမည်။` | | the inner part |

The loop creates the counter variable `ကိန်း` on its own, and in each round the counter
has the next number. We need a space before `အနေဖြင့်`, because `ကိန်းအနေဖြင့်` with no
space is one word to Zaka.

## Counting down, and steps

```text
<first> မှ <last> အထိ <step> စီ <name> အနေဖြင့် ရေတွက်၍ <inner part>
```

`ရေတွက်၍` counts up or down, whichever direction reaches the last value. To take bigger
steps, we add `<step> စီ` ("`<step>` at a time") before the counter's name:

```zaka
၅ မှ ၁ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသမည်။ // counts down
"---" ဟု ပြသမည်။
၀ မှ ၁၀၀ အထိ ၂၅ စီ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသမည်။ // 25 at a time
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
| `တိုး၍` | "increasing" | up, never down |
| `လျှော့၍` | "decreasing" | down, never up |

With `တိုး၍` or `လျှော့၍`, when the first value is past the last one, the loop runs zero
times:

```zaka
၁၀ မှ ၀ အထိ ၂ စီ ကိန်း အနေဖြင့် လျှော့၍ ကိန်း ကို ပြသမည်။ // 10, 8, 6, 4, 2, 0
၅ မှ ၁ အထိ ကိန်း အနေဖြင့် တိုး၍ ကိန်း ကို ပြသမည်။ // never runs
"ပြီး" ဟု ပြသမည်။
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

A loop often builds a total one round at a time, so we set a variable to zero before the
loop and add the counter to it in each round:

```zaka
ကိန်းရှင် စုစုပေါင်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။ // total
၁ မှ ၁၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ စုစုပေါင်း ကို စုစုပေါင်း + ကိန်း ဟု ထားမည်။
စုစုပေါင်း ကို ပြသမည်။
```

```text
၅၅
```

## Loops inside loops

A loop can contain another loop. This one prints a small times table, where the outer loop
runs once for each row and the inner loop runs once for each column of that row:

```zaka
၁ မှ ၃ အထိ အတန်း အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။ // row
၁ မှ ၃ အထိ အတိုင် အနေဖြင့် ရေတွက်၍ အတန်း * အတိုင် ကို ပြသမည်။ // column
"---" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
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

## The counter is fixed

The inner part can read the counter but cannot change it, because the loop sets it in
each round, so the compiler stops with an error:

```zaka-error ZK0307
၁ မှ ၁၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ၀ ဟု ထားမည်။
```

```text
error[ZK0307]: "ကိန်း" is a counted-loop variable and cannot be assigned
```

The counter also belongs to the loop, so it is gone after the loop ends.

## Details

- **Integers alone.** The first value, the last value and the step must be integers, or
  the compiler reports ZK0405. These can be variables or sums, as in
  `အစ မှ အဆုံး - ၁ အထိ ...` ([Counted loops](../../manual/language/control-flow.md#counted-loops)).
- **Worked out once.** Zaka works out the first value, the last value and the step once,
  before the first round, so changing a variable of the last value inside the loop does
  not make the loop longer ([Counted loops](../../manual/language/control-flow.md#counted-loops)).
- **The step.** The step must be greater than zero: a step of `၀` written in the program
  is the error ZK0409, and a zero step that comes from a variable stops the program when
  the loop starts ([Counted loops](../../manual/language/control-flow.md#counted-loops)).
- **The counter's name.** The inner part cannot declare a variable with the same name as
  the counter (ZK0302) ([Counted loops](../../manual/language/control-flow.md#counted-loops)).
- **The counter's data kind.** The counter has the data kind (type) of the two values, and
  it never goes past the last value, so it does not overflow
  ([Overflow](../../manual/language/expressions.md#arithmetic)).
- **Each cell of a list.** To run a loop once for each cell (element) of a list, Zaka has
  a separate loop, shown in [Lists](../lists/README.md).

## Recap

- We write `<first> မှ <last> အထိ <name> အနေဖြင့် ရေတွက်၍ <inner part>` to count through
  a range, and the loop includes both ends.
- We add `<step> စီ` to take bigger steps, and use `တိုး၍` or `လျှော့၍` to fix the
  direction.
- The inner part can read the counter but cannot change it.
- A loop can contain another loop.

## Hands on

- Print the even numbers from 2 to 20.
- Work out 1 × 2 × 3 × ... × 10 with a counted loop.

Next: [Generic loops](generic-loops.md).
