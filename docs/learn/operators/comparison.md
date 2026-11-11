---
title: Comparison
description: Ask whether values are equal, greater or less.
---

# Comparison

A *comparison* asks a true-or-false question about two values: are the two equal? Is
one bigger? The answer is a [boolean](../values/booleans.md), `မှန်` or `မမှန်`.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၅ ဖြစ်ပါစေ။

// Words: the subject takes သည်, then ထက်, အောက် or နှင့်, then the verb
အသက် သည် ၁၈ ထက် ကြီးသည် ဟု ပြသမည်။        // age is greater than 18
အသက် သည် ၁၈ အောက် ငယ်သည် ဟု ပြသမည်။       // age is less than 18
အသက် သည် ၂၅ နှင့် ညီသည် ဟု ပြသမည်။        // age equals 25

// Signs
အသက် > ၁၈ ကို ပြသမည်။
အသက် < ၁၈ ကို ပြသမည်။
အသက် == ၂၅ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
မမှန်
မှန်
မှန်
မမှန်
မှန်
```

## All six comparisons

| Sign | Words | Reads as |
| --- | --- | --- |
| `a == b` | `a သည် b နှင့် ညီသည်` | a is equal to b |
| `a != b` | `a သည် b နှင့် မညီပါ` | a is not equal to b |
| `a > b` | `a သည် b ထက် ကြီးသည်` | a is greater than b |
| `a <= b` | `a သည် b ထက် မကြီးပါ` | a is not greater than b |
| `a < b` | `a သည် b အောက် ငယ်သည်` | a is less than b |
| `a >= b` | `a သည် b အောက် မငယ်ပါ` | a is not less than b |

The `မ` in front of a verb means "not": `မညီ` is "not equal", `မကြီး` is "not greater".
Equals uses `=` twice: `==`. A single `=` is not an operator in Zaka.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၅ ဖြစ်ပါစေ။
အသက် သည် ၃၀ နှင့် မညီပါ ဟု ပြသမည်။
အသက် သည် ၆၀ ထက် မကြီးပါ ဟု ပြသမည်။
အသက် သည် ၁၈ အောက် မငယ်ပါ ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
မှန်
မှန်
```

Characters can be compared too. Their order follows the Unicode number, so `'က'`
comes before `'ခ'`, and `'A'` (65) comes before `'a'` (97):

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
'က' < 'ခ' ကို ပြသမည်။
'A' < 'a' ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
မှန်
```

## Keeping the answer

A comparison is a value, so it can be stored in a boolean variable:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၃ ဖြစ်ပါစေ။
ကိန်းရှင် အောင် သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အမှတ် သည် ၄၀ အောက် မငယ်သည် ဖြစ်ပါစေ။ // passed
အောင် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
```

## One comparison at a time

Maths allows 1 < 2 < 3. Zaka does not allow a comparison inside another
comparison:

```zaka-error ZK0210
၁ < ၂ < ၃ ကို ပြသမည်။
```

```text
error[ZK0210]: a comparison cannot be the operand of another comparison
```

Ask two questions and join them with "and" instead: `၁ < ၂ ဖြစ်၍ ၂ < ၃`. The next page,
[Logic](logic.md), shows how.

## Hands on

- Print whether 7 is not equal to 7, in words and in signs.
- Store whether an age is at least 18 in a variable, then print it.

Next: [Logic](logic.md).
