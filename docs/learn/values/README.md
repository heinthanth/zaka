---
title: Values and kinds
description: Each value has a kind - integer, decimal number, character or true/false.
---

# Values and kinds

A *value* is a piece of data: the number `၄၂`, the letter `'က'`, the answer "true".
Each value has a **kind**. The kind tells what sort of data the value is and what a
program can do with it. Two numbers can be added, but two answers of "true" cannot.

```zaka
၄၂ ကို ပြသပါ။          // an integer (a whole number)
၃.၁၄ ကို ပြသပါ။        // a decimal number
'က' ကို ပြသပါ။         // a character
မှန် ကို ပြသပါ။         // a true/false value: true
"စာ" ဟု ပြသပါ။         // text in quotes, just for printing
```

```text
၄၂
၃.၁၄
က
မှန်
စာ
```

## The kinds

| Kind | Reads as | Holds | Page |
| --- | --- | --- | --- |
| `ကိန်းပြည့်` | "whole number" | integers: `၀`, `၁၈`, `-၅` | [Integers](integers.md) |
| `ဒဿမကိန်း` | "decimal number" | numbers with a fraction: `၃.၁၄` | [Decimal numbers](floats.md) |
| `စာလုံး` | "letter" | one character: `'က'`, `'A'` | [Characters](characters.md) |
| `မှန်မှားတန်ဖိုး` | "true/false value" | `မှန်` (true) or `မမှန်` (false) | [True and false](booleans.md) |

Text in quotes, such as `"စာ"`, has no kind of its own so far. A program can do nothing
with it but print it (see [Text in quotes](strings.md)).

The kind matters as soon as a program stores values. The next chapter,
[Variables](../variables/README.md), shows how. Each variable is declared with its kind.

## Hands on

- Print one value of each kind.
- Print `၃ + ၀.၅`. Then print `'က' + ၁` and read the error message.

Next: [Integers](integers.md).
