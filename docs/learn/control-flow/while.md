---
title: While loops
description: Repeat sentences as long as a condition is true.
---

# While loops

A loop runs its inner part again and again. A `while` loop keeps going as long as a
condition is true, and in Zaka we write it as "as long as *condition*, do the following".

## You will learn

- how to repeat sentences while a condition is true
- how a while loop runs, one round at a time
- how to use a true/false value as the condition
- how to stop a program whose loop never ends

```zaka
ကိန်းရှင် ကိန်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။

// as long as the number is less than 100, ...
ကိန်း သည် ၁၀၀ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်း ကို ပြသမည်။
ကိန်း ကို ကိန်း * ၂ ဟု ထားမည်။        // double it
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

"ပြီးပြီ" ဟု ပြသမည်။
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

To repeat sentences while a condition is true, we use this pattern:

```text
<a> သည် <b> အောက် ငယ်နေသရွေ့ <inner part>
```

| Zaka | Reads as |
| --- | --- |
| `<a> သည် <b> အောက် ငယ်` | "`<a>` is less than `<b>`", a comparison in words |
| `နေသရွေ့` | "as long as"; it joins the last verb of the condition, as in `ငယ်နေသရွေ့` |
| `<inner part>` | one sentence, or a block |

## How a loop runs

Each round of the loop goes like this:

1. Zaka tests the condition. When it is false, the loop ends, and the program goes on
   with the sentence after the loop.
2. Zaka runs the inner part.
3. Zaka goes back to step 1.

In the first example, after the loop prints 64, it doubles the number to 128. The test
"128 is less than 100" is then false, so the loop ends and never prints 128. Because Zaka
tests the condition before each round, a loop whose condition is false at the start runs
zero times.

## One-sentence loops

The inner part can be a single sentence, written right after `နေသရွေ့`:

```zaka
ကိန်းရှင် ကျန် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။      // remaining
ကျန် သည် ၀ ထက် ကြီးနေသရွေ့ ကျန် ကို ကျန် - ၁ ဟု ထားမည်။
ကျန် ကို ပြသမည်။
```

```text
၀
```

## A true/false value as the condition

```text
<true/false value> ဖြစ်နေသရွေ့ <inner part>
```

When the condition is a true/false variable or a comparison in signs, it has no verb for
`နေသရွေ့` to join, so we write `ဖြစ်နေသရွေ့` ("as long as it is so") as a separate word
after it:

```zaka
ကိန်းရှင် ဆက်ရှာ သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် မှန် ဖြစ်ပါစေ။ // keep searching
ကိန်းရှင် အကြိမ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။        // round
ဆက်ရှာ ဖြစ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကြိမ် ကို အကြိမ် + ၁ ဟု ထားမည်။
// keep going while round × round is below 50
ဆက်ရှာ ကို အကြိမ် * အကြိမ် သည် ၅၀ အောက် ငယ်သည် ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အကြိမ် ကို ပြသမည်။
```

```text
၈
```

The loop stops in the round where `အကြိမ် * အကြိမ်` first reaches 50 or more, which is
8 × 8 = 64.

## Loops that never end

When the condition never becomes false, the loop runs forever, so the inner part must
change something that the condition depends on. To end a program that does not stop, we
press **Stop** in the Explorer, or `Ctrl`+`C` in a terminal.

## Details

- **A whole sentence as the condition.** After a full comparison that ends in `သည်`, such
  as `က သည် ၀ ထက် ကြီးသည်`, we also write `ဖြစ်နေသရွေ့` as a separate word
  ([While loops](../../manual/language/control-flow.md#while-loops)).
- **Leaving in the middle.** To leave a loop in the middle of a round, see [Break and
  continue](break-continue.md).
- **Counting.** For a loop over a range of numbers, a [counted loop](counted-loops.md) is
  shorter.
- **The condition must be true or false.** A number as the condition is the error ZK0402,
  the same as in an [if](if-else.md).

## Recap

- We write `<condition>နေသရွေ့ <inner part>` to repeat sentences while a condition is
  true.
- Zaka tests the condition before each round, so a loop can run zero times.
- After a true/false variable or a comparison in signs, we write `ဖြစ်နေသရွေ့` as a
  separate word.
- The inner part must change what the condition tests, or the loop never ends.

## Hands on

- Print the numbers 10 down to 1 with a while loop.
- Start from 1000 and keep halving the number while it is greater than 1. How often does
  it get halved?

Next: [Counted loops](counted-loops.md).
