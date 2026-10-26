---
title: Printing
description: Show values with ပြသပါ, and choose between ကို and ဟု.
---

# Printing

`ပြသပါ` ("please show") prints one value and then starts a new line. The value comes
first, then a small word that marks it, then `ပြသပါ`. That small word is either `ကို` or
`ဟု`.

```zaka
// ကို: "show this value"
၄၂ ကို ပြသပါ။
၃.၁၄ ကို ပြသပါ။
'က' ကို ပြသပါ။
၁၀ + ၅ ကို ပြသပါ။

// ဟု: "say these words", for text in quotes
"မင်္ဂလာပါ" ဟု ပြသပါ။

// ဟု also quotes a whole sentence, such as "3 is greater than 2"
၃ သည် ၂ ထက် ကြီးသည် ဟု ပြသပါ။

// An empty pair of quotes prints an empty line
"" ဟု ပြသပါ။
"ပြီးပါပြီ" ဟု ပြသပါ။
```

```text
၄၂
၃.၁၄
က
၁၅
မင်္ဂလာပါ
မှန်

ပြီးပါပြီ
```

The sentence `၃ သည် ၂ ထက် ကြီးသည်` is either true or false, so it prints `မှန်` (true).
[True and false](../values/booleans.md) covers that in detail.

| Zaka | Reads as | Use it for |
| --- | --- | --- |
| `ကို` | (marks the object) | a plain value: a number, a character, a sum |
| `ဟု` | "so", quoting | text in quotes, or a sentence that ends in `သည်` or `ပါ` |
| `ပြသပါ` | "please show" | print, then go to a new line |

## How values look when printed

- Numbers print with Burmese digits, even if you wrote `123`.
- Decimal numbers always show a decimal point: `၁၀.၀`.
- True and false print as `မှန်` and `မမှန်`.
- Characters and text print as they are, without quotes.

## Picking the wrong word

Both words work, so the program still runs. But the compiler warns you when the other
word reads more naturally. A warning starts with `warning`; the program still runs and
prints `၅`:

```zaka-warning ZK0506
၅ ဟု ပြသပါ။
```

```text
warning[ZK0506]: a plain value is marked with "ကို", not quoted with "ဟု"
```

```zaka-warning ZK0505
"မင်္ဂလာပါ" ကို ပြသပါ။
```

```text
warning[ZK0505]: a sentence or a string is quoted with "ဟု", not marked with "ကို"
```

The [Natural Zaka](../natural-zaka/README.md) chapter lists every warning of this kind.

## Try it

- Print your age with `ကို`, then print a sentence about it in quotes with `ဟု`.
- Print `၇ / ၂` and `၇.၀ / ၂`. Why do you think the results differ? (The answer is in
  [Arithmetic](../operators/arithmetic.md).)

Next: [Zaka sentences](../sentences/README.md).
