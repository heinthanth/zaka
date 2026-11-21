---
title: Scalar values
description: Each value has a data kind, and a scalar value is one thing with no parts - an integer, a decimal number, a character or a true/false value.
---

# Scalar values

A *value* is one piece of data, such as the number `၄၂`, the letter `'က'` or the answer
"true". Each value has a data kind (type), which shows what sort of data the value is and
what a program can do with it: we can add two numbers, for example, but we
cannot add two answers of "true".

## You will learn

- What a data kind is, and the reason the compiler checks it.
- The four scalar data kinds: integers, decimal numbers, characters and true/false
  values.
- How to write a value of each of them.
- Where text (string) fits in.

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

Each line prints a value of a different data kind. These four are *scalar* data kinds,
which means that each value is one thing, with no parts inside it. Text, lists and
structs are built from parts, and each has its own chapter later. Each data kind has
a Burmese name, and we use that name later when we make a [variable](../variables/README.md):

| Data kind | Reads as | Values | Page |
| --- | --- | --- | --- |
| `ကိန်းပြည့်` | "whole number" | integers (`int`): `၀`, `၁၈`, `-၅` | [Integers](integers.md) |
| `ဒဿမကိန်း` | "decimal number" | numbers with a fraction (float): `၃.၁၄` | [Decimal numbers](floats.md) |
| `စာလုံး` | "letter" | one character (`char`): `'က'`, `'A'` | [Characters](characters.md) |
| `မှန်မှားတန်ဖိုး` | "true/false value" | `မှန်` (true) or `မမှန်` (false), a boolean | [True and false](booleans.md) |

## Data kinds do not mix

The compiler checks the data kind of each value before the program runs. When an
operation does not fit a data kind, such as adding a number to a character, the program
does not run, and the compiler reports the mistake:

```zaka-error ZK0403
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
'က' + ၁ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0403]: + needs a number, found char
```

Compiler messages use short English names for the data kinds, such as `char` for
`စာလုံး`, and the [error messages](../../manual/reference/errors.md) page lists them.

## Text

Text (string) in double quotes, such as `"စာ"`, is a value too, and we printed some in
[Printing](../hello/printing.md). Its data kind is `စာသား`, and a text value is a row of
characters, so it is not a scalar. The [Text](../text/README.md) chapter comes after
[Functions](../functions/README.md).

## Details

- **Numbers with no data kind written.** A number such as `၃` takes the data kind that
  its place wants, so `၃ + ၀.၅` works and prints `၃.၅`. See
  [Numbers](../../manual/language/words.md#numbers).
- **Converting.** To turn a value into another data kind, we use a conversion (cast),
  which [Converting between data kinds](../operators/conversion.md) shows.
- **The full rules.** [Data kinds and values](../../manual/language/kinds.md) states the
  range, the size and the initial value of each data kind.

## Recap

- Each value has a data kind (type), which decides what we can do with it.
- The four scalar data kinds are `ကိန်းပြည့်`, `ဒဿမကိန်း`, `စာလုံး` and
  `မှန်မှားတန်ဖိုး`.
- The compiler checks the data kinds before the program runs, and it stops a program that
  mixes them.
- Text, `စာသား`, is a row of characters and has its own chapter.

## Hands on

- Print one value of each scalar data kind.
- Print `မှန် + ၁` and read the error message. Which short English name does it use for
  `မှန်မှားတန်ဖိုး`?

Next: [Integers](integers.md).
