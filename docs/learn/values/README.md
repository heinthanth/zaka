---
title: Scalar values
description: Each value has a kind; a scalar value holds one thing with no parts - an integer, a decimal number, a character or true/false.
---

# Scalar values

A *value* is a piece of data: the number `၄၂`, the letter `'က'`, the answer "true".
Each value has a **kind**. The kind tells what sort of data the value is and what a
program can do with it. Two numbers can be added, but two answers of "true" cannot.

This chapter is about *scalar* values: each holds one thing, with no parts inside it.
Text, lists and structs are built from parts, and have chapters of their own later.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၄၂ ကို ပြသမည်။          // an integer (a whole number)
၃.၁၄ ကို ပြသမည်။        // a decimal number
'က' ကို ပြသမည်။         // a character
မှန် ကို ပြသမည်။         // a true/false value: true
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၂
၃.၁၄
က
မှန်
```

## The kinds

| Kind | Reads as | Holds | Page |
| --- | --- | --- | --- |
| `ကိန်းပြည့်` | "whole number" | integers: `၀`, `၁၈`, `-၅` | [Integers](integers.md) |
| `ဒဿမကိန်း` | "decimal number" | numbers with a fraction: `၃.၁၄` | [Decimal numbers](floats.md) |
| `စာလုံး` | "letter" | one character: `'က'`, `'A'` | [Characters](characters.md) |
| `မှန်မှားတန်ဖိုး` | "true/false value" | `မှန်` (true) or `မမှန်` (false) | [True and false](booleans.md) |

Text in double quotes, `"စာ"`, prints too, as [Printing](../hello/printing.md) showed. Its
kind, `စာသား`, holds a row of characters, and [Text](../text/README.md) comes after
[Functions](../functions/README.md).

The kind matters as soon as a program stores values. The next chapter,
[Variables](../variables/README.md), shows how. Each variable is declared with its kind.

## Hands on

- Print one value of each scalar kind.
- Print `၃ + ၀.၅`. Then print `'က' + ၁` and read the error message.

Next: [Integers](integers.md).
