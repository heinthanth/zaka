---
title: Variables
description: Give a value a name with ကိန်းရှင် ... ဖြစ်ပါစေ။
---

# Variables

A *variable* is a named box that holds a value. Create it once, then use its name
wherever the value is needed. Creating a variable is called *declaring* it.

```zaka
// "Let the variable age be one integer, and its value be 18."
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။

// "Let the variable price be one decimal number, and its value be 2.5."
ကိန်းရှင် ဈေး သည် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂.၅ ဖြစ်ပါစေ။

// The initial value can be worked out from other variables.
ကိန်းရှင် နှစ်ဆ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အသက် * ၂ ဖြစ်ပါစေ။

အသက် ကို ပြသပါ။
ဈေး ကို ပြသပါ။
နှစ်ဆ ကို ပြသပါ။
```

```text
၁၈
၂.၅
၃၆
```

A declaration reads like a Burmese sentence:

| Zaka | Reads as | Part |
| --- | --- | --- |
| `ကိန်းရှင်` | "variable" | announces a new variable |
| `အသက်` | "age" | the chosen name |
| `သည်` | (subject) | |
| `ကိန်းပြည့်` | "integer" | the kind of value it holds |
| `တစ်ခု` | "one" | counts the kind, as Burmese counts a thing |
| `ဖြစ်၍` | "is, and" | joins the kind and the value |
| `တန်ဖိုးသည်` | "its value" | |
| `၁၈` | 18 | the initial value (optional) |
| `ဖြစ်ပါစေ` | "let it be" | the verb that declares |
| `။` | full stop | |

Burmese counts a thing with `တစ်` ("one") and a *classifier*, a word that fits the thing
counted. Most kinds take `ခု`; `စာလုံး` (a character) takes `လုံး`. Leave the count word out
and the compiler stops with ZK0234.

Each variable has a kind, and it keeps that kind for its whole life. A `ကိန်းပြည့်`
variable holds an integer, and nothing else.

## Letting the value give the kind

When the value is written, the kind can be left out: the variable takes the kind of its
value. A whole number gives `ကိန်းပြည့်`, a number with a point gives `ဒဿမကိန်း`, and text
in quotes gives `စာသား`.

```zaka
ကိန်းရှင် အသက် သည် ၁၈ ဖြစ်ပါစေ။
ကိန်းရှင် ဈေး သည် ၂.၅ ဖြစ်ပါစေ။
ကိန်းရှင် အမည် သည် "မမ" ဖြစ်ပါစေ။
ကိန်းရှင် နှစ်ဆ သည် အသက် * ၂ ဖြစ်ပါစေ။
အမည် ကို ပြသပါ။
နှစ်ဆ ကို ပြသပါ။
```

```text
မမ
၃၆
```

Write the kind when the value alone would pick another one, such as a small integer kind:
`ကိန်းရှင် ဘိုက် သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀၀ ဖြစ်ပါစေ။`

## Starting values

Leave out `ဖြစ်၍ တန်ဖိုးသည်` and the initial value, and the variable starts at its
kind's *default*:

| Kind | Default |
| --- | --- |
| integers and decimal numbers | `၀` (zero) |
| `မှန်မှားတန်ဖိုး` | `မမှန်` (false) |
| `စာလုံး` | the null character `'\0'` |

```zaka
ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။      // count, starts at 0
ကိန်းရှင် ပြီးပြီ သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်ပါစေ။      // done, starts as false
ကိန်းရှင် ပျမ်းမျှ သည် ဒဿမကိန်း တစ်ခု ဖြစ်ပါစေ။         // average, starts at 0.0
အရေအတွက် ကို ပြသပါ။
ပြီးပြီ ကို ပြသပါ။
ပျမ်းမျှ ကို ပြသပါ။
```

```text
၀
မမှန်
၀.၀
```

## The short form

The value can also come straight after the kind, without `ဖြစ်၍ တန်ဖိုးသည်`. These two
lines declare the same kind of variable with the same value:

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။
ကိန်းရှင် အရွယ် သည် ကိန်းပြည့် ၁၈ ဖြစ်ပါစေ။
အသက် ကို ပြသပါ။
အရွယ် ကို ပြသပါ။
```

```text
၁၈
၁၈
```

The short form is quicker to write. The long form keeps the kind and the value apart,
which reads better when the value is long or starts with a kind word of its own. These
pages use the long form.

## The value must fit the kind

The initial value must suit the variable's kind. A decimal number cannot go into an
integer variable:

```zaka-error ZK0401
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈.၅ ဖြစ်ပါစေ။
```

```text
error[ZK0401]: expected isize, found f64
```

`isize` is the compiler's short name for plain `ကိန်းပြည့်`, and `f64` for `ဒဿမကိန်း`.
The [Error messages](../../manual/reference/errors.md) page lists all these short names.

## Using a name before declaring it

A variable exists from its declaration onward, and not before. Using a name that was
never declared is an error:

```zaka-error ZK0301
အသက် ကို ပြသပါ။
```

```text
error[ZK0301]: "အသက်" is not declared
```

## Hands on

- Declare a variable for a height in centimetres and print it.
- Declare a `မှန်မှားတန်ဖိုး` variable with the value `၅ သည် ၃ ထက် ကြီးသည်` and print it.

Next: [Constants](constants.md).
