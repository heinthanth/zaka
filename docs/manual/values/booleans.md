---
title: True and false
description: The two values မှန် and မမှန်, and where they come from.
---

# True and false

Some questions have only two answers: yes or no. "Is 5 bigger than 3?" "Is it raining?"
A value that is either **true** or **false** is a *boolean*. Its kind is `မှန်မှားတန်ဖိုး`
("true/false value"); the loanword `ဘူလီယန်` means the same.

| Zaka | Reads as | Value |
| --- | --- | --- |
| `မှန်` | "right", "correct" | true |
| `မမှန်` | "not right" | false |

```zaka
မှန် ကို ပြသပါ။
မမှန် ကို ပြသပါ။
၅ သည် ၃ ထက် ကြီးသည် ဟု ပြသပါ။     // a comparison gives a boolean
၅ < ၃ ကို ပြသပါ။                  // so does the symbol form
```

```text
မှန်
မမှန်
မှန်
မမှန်
```

You will rarely type `မှန်` or `မမှန်` yourself. Booleans mostly come from
[comparisons](../operators/comparison.md), and they drive decisions such as
[if and else](../control-flow/if-else.md).

## Endings

Like a verb, `မှန်` and `မမှန်` may carry a polite ending. `မှန်`, `မှန်သည်`, `မှန်ပါသည်` are all
"true". When an ending makes it a sentence, print it with `ဟု`:

```zaka
မှန်သည် ဟု ပြသပါ။
မမှန်ပါ ဟု ပြသပါ။
```

```text
မှန်
မမှန်
```

## Try it

- Print whether 10 equals 10, using `==`.
- Add `၁` to `မှန်`. Read the message: booleans are not numbers.

Next: [Text in quotes](strings.md).
