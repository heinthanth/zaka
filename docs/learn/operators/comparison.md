---
title: Comparison
description: Check whether two values are equal, or which one is greater or less.
---

# Comparison

A comparison checks two values, for example whether the two are equal or which one is
greater, and returns a true/false value (boolean): `မှန်` (true) or `မမှန်` (false). In
words, a comparison is a short Burmese sentence, such as "age is greater than 18".

## You will learn

- the six comparisons, in signs and in words
- how `မ` turns a comparison word into its opposite
- how to keep the answer of a comparison in a variable
- that one comparison cannot go straight inside another

| Sign | Words | Reads as |
| --- | --- | --- |
| `a == b` | `a သည် b နှင့် ညီသည်` | a is equal to b |
| `a != b` | `a သည် b နှင့် မညီပါ` | a is not equal to b |
| `a > b` | `a သည် b ထက် ကြီးသည်` | a is greater than b |
| `a <= b` | `a သည် b ထက် မကြီးပါ` | a is not greater than b |
| `a < b` | `a သည် b အောက် ငယ်သည်` | a is less than b |
| `a >= b` | `a သည် b အောက် မငယ်ပါ` | a is not less than b |

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၅ ဖြစ်ပါစေ။

// Words: the first value, then သည်, the second value with ထက်, အောက် or နှင့်, then the verb
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

To compare two values in words, we use this pattern:

```text
<a> သည် <b> ထက် ကြီးသည်
```

| Zaka | Reads as |
| --- | --- |
| `<a> သည်` | "`<a>`", the subject of the sentence |
| `<b> ထက်` | "than `<b>`" |
| `ကြီးသည်` | "is big" |

The other comparisons change two words: `<b> အောက် ငယ်သည်` ("is small below `<b>`") for
less than, and `<b> နှင့် ညီသည်` ("is equal with `<b>`") for equal. A comparison in words
is a whole sentence, so we print it with `ဟု`, while the sign form is a plain value that
we print with `ကို`.

## Writing "not"

```text
<a> သည် <b> နှင့် မညီပါ
```

In Burmese, `မ` in front of a verb means "not", so `မညီ` is "not equal" and `မကြီး` is
"not greater". With `မ`, the verb can end in `ပါ` or in `သည်`, so `မညီပါ` and `မညီသည်`
mean the same. "Not greater" means less than or equal, so `<=` is `ထက် မကြီးပါ` and `>=` is
`အောက် မငယ်ပါ`.

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

In the sign form, "equal" is two `=` signs, `==`, because a single `=` is not an operator
in Zaka.

## Comparing characters

We can compare characters too, and their order follows their Unicode numbers, so `'က'`
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

```text
ကိန်းရှင် <name> သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် <comparison> ဖြစ်ပါစေ။
```

A comparison is a value, so we can keep its answer in a true/false variable and use it
later:

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

In maths we can write 1 < 2 < 3, but Zaka does not let one comparison go straight inside
another, so the compiler stops with an error:

```zaka-error ZK0210 repl=on
၁ < ၂ < ၃ ကို ပြသမည်။
```

```text
error[ZK0210]: a comparison cannot be the operand of another comparison
```

To check both parts, we ask two questions and join them with "and":
`၁ < ၂ ဖြစ်၍ ၂ < ၃`. The next page, [Logic](logic.md), shows how.

## Details

- **In a condition.** After `အကယ်၍` ("if"), the verb ends in `လျှင်` in place of `သည်`,
  as in `ကြီးလျှင်` ([If and else](../control-flow/if-else.md)).
- **Which values compare.** `==` and `!=` work on values of each data kind (type),
  including text, lists and structs, which compare their contents. `<`, `>`, `<=` and
  `>=` work on numbers and characters alone, so ordering two true/false values is the
  error ZK0403 ([Comparison](../../manual/language/expressions.md#comparison)).
- **Two integer data kinds.** When the two numbers have different integer data kinds,
  Zaka widens the narrower one first, the same as in
  [arithmetic](arithmetic.md#mixing-data-kinds).
- **`NaN`.** The decimal value `NaN` is equal to nothing, not even to itself, and each
  ordering with it is false ([Comparison](../../manual/language/expressions.md#comparison)).
- **One and the same.** To ask whether two shared objects (reference types) are one and
  the same object, Zaka has `X နှင့် Y သည် တစ်ခုတည်း`
  ([Comparison](../../manual/language/expressions.md#comparison)).

## Recap

- Each comparison has a sign form (`က > ခ`) and a word form (`က သည် ခ ထက် ကြီးသည်`), and
  it returns `မှန်` or `မမှန်`.
- `ထက်` goes with "greater", `အောက်` with "less" and `နှင့်` with "equal".
- `မ` in front of the verb turns the comparison into its opposite.
- One comparison cannot go straight inside another, so we join two comparisons with
  "and".

## Hands on

- Print whether 7 is not equal to 7, in words and in signs.
- Keep whether an age is at least 18 in a variable, then print it.
- Print whether `'z'` comes before `'Z'`.

Next: [Logic](logic.md).
