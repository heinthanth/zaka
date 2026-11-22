---
title: True and false
description: The two values မှန် and မမှန်, and where each comes from.
---

# True and false

Some questions have just two answers, such as "Is 5 bigger than 3?" or "Is it raining?".
A value that is either true or false is a *true/false value* (boolean, `bool`), and its
data kind (type) is `မှန်မှားတန်ဖိုး` ("true/false value").

## You will learn

- The two true/false values, `မှန်` and `မမှန်`.
- How a comparison gives a true/false value.
- The polite endings after `မှန်` and `မမှန်`.

```zaka
မှန် ကို ပြသမည်။
မမှန် ကို ပြသမည်။
```

```text
မှန်
မမှန်
```

The data kind has two values, and we write them as two Burmese words:

```
မှန်
မမှန်
```

| Zaka | Reads as | Value |
| --- | --- | --- |
| `မှန်` | "right", "correct" | true |
| `မမှန်` | "not right" | false |

## Answers from comparisons

Programs seldom write `မှန်` or `မမှန်` out. Most true/false values come from a
[comparison](../operators/comparison.md), in words or with a sign such as `<`:

```zaka
၅ သည် ၃ ထက် ကြီးသည် ဟု ပြသမည်။ // a comparison gives a true/false value
၅ < ၃ ကို ပြသမည်။ // and so does the sign form
```

```text
မှန်
မမှန်
```

Later, these answers decide which part of a program runs, as in [If and
else](../control-flow/if-else.md).

## Endings

```
မှန်သည်
မမှန်ပါ
```

Like a verb, `မှန်` and `မမှန်` can have a polite ending, so `မှန်`, `မှန်သည်` and
`မှန်ပါသည်` all mean "true". An ending makes the word a sentence, so we print it with
`ဟု`:

```zaka
မှန်သည် ဟု ပြသမည်။
မမှန်ပါ ဟု ပြသမည်။
```

```text
မှန်
မမှန်
```

## Details

- **A second name.** The loanword `ဘူလီယန်` is a second name for `မှန်မှားတန်ဖိုး`.
- **Not a number.** A true/false value is not a number, so `မှန် + ၁` is the error
  ZK0403, and we cannot use an integer as a condition in place of true or false
  (ZK0402). See [True/false values](../../manual/language/kinds.md#truefalse-values).

## Recap

- A true/false value is `မှန်` (true) or `မမှန်` (false), and its data kind is
  `မှန်မှားတန်ဖိုး`.
- Most true/false values come from comparisons.
- With a polite ending, such as `မှန်သည်`, we print the value with `ဟု`.

## Hands on

- Print whether 10 equals 10, using `==`.
- Add `၁` to `မှန်`, and read the message: true/false values are not numbers.

Next: [Variables](../variables/README.md).
