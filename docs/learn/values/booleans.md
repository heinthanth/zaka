---
title: True and false
description: The two values မှန် and မမှန်, and where each comes from.
---

# True and false

Some questions have just two possible answers. "Is 5 bigger than 3?" "Is it raining?"
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
၅ < ၃ ကို ပြသပါ။                  // so does the operator form
```

```text
မှန်
မမှန်
မှန်
မမှန်
```

Programs seldom write `မှန်` or `မမှန်` out. Most booleans come from
[comparisons](../operators/comparison.md), and these drive decisions such as
[if and else](../control-flow/if-else.md).

## Endings

Like a verb, `မှန်` and `မမှန်` can take a polite ending. `မှန်`, `မှန်သည်`, `မှန်ပါသည်` all mean
"true". When an ending makes it a sentence, print it with `ဟု`:

```zaka
မှန်သည် ဟု ပြသပါ။
မမှန်ပါ ဟု ပြသပါ။
```

```text
မှန်
မမှန်
```

## Hands on

- Print whether 10 equals 10, using `==`.
- Add `၁` to `မှန်`. Read the message: booleans are not numbers.

Next: [Text](strings.md).
