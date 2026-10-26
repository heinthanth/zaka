---
title: Changing a variable
description: Give a variable a new value with ... ကို ... ဟု ထားပါ။
---

# Changing a variable

A variable can be given a new value at any time. This is called *assignment*. In Zaka
you say "set x to v": `x ကို v ဟု ထားပါ။`

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ၁၈ ဖြစ်ပါစေ။
အသက် ကို ပြသပါ။

အသက် ကို ၂၄ ဟု ထားပါ။                // set age to 24
အသက် ကို ပြသပါ။

အသက် ကို အသက် + ၁ ဟု ထားပါ။          // set age to age + 1
အသက် ကို ပြသပါ။

အသက် ကို အသက် နှင့် ၁၀ ၏ ပေါင်းလဒ် ဟု ထားပါ။   // the word form works too
အသက် ကို ပြသပါ။
```

```text
၁၈
၂၄
၂၅
၃၅
```

| Zaka | Reads as | Part |
| --- | --- | --- |
| `အသက်` | "age" | the variable to change |
| `ကို` | (object) | |
| `အသက် + ၁` | age + 1 | the new value |
| `ဟု` | "so", "as" | marks the new value |
| `ထားပါ` | "please put", "please set" | the verb: assign |

## How `အသက် ကို အသက် + ၁ ဟု ထားပါ။` works

To a maths student, "age = age + 1" looks impossible. To a computer it is two steps:

1. Work out the new value, `အသက် + ၁`, using the **current** value of `အသက်` (24).
2. Put the result (25) into `အသက်`, replacing the old value.

## Storing a true/false answer

A sentence that is true or false can be stored too:

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ၂၀ ဖြစ်ပါစေ။
ကိန်းရှင် အရွယ်ရောက် သည် မှန်မှားတန်ဖိုး ဖြစ်ပါစေ။       // adult, starts as false
အရွယ်ရောက် ကို အသက် သည် ၁၈ ထက် ကြီးသည် ဟု ထားပါ။
အရွယ်ရောက် ကို ပြသပါ။
```

```text
မှန်
```

## The new value must fit

The new value must suit the variable's kind, just as the initial value did:

```zaka-error ZK0401
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ၁၈ ဖြစ်ပါစေ။
အသက် ကို ၂.၅ ဟု ထားပါ။
```

```text
error[ZK0401]: expected isize, found f64
```

## Try it

- Start a variable at `၁` and double it four times, printing it after each change.
- Make a variable `ကျန်` (remaining) of `၁၀၀`, then take away `၃၅` from it.

Next: [Names](names.md).
