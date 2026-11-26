---
title: Printing
description: Show values with ပြသမည်, and choose between ကို and ဟု.
---

# Printing

`ပြသမည်` ("will show") prints a value and then starts a new line. Put a particle
(postposition) between the value and `ပြသမည်`. Use `ကို` for a value such as a number
or a variable, and `ဟု` for text (string) in quotes or a full clause such as
`၃ သည် ၂ ထက် ကြီးသည်` ("3 is greater than 2").

## You will learn

- How to print a number, a character or a sum with `ကို`.
- How to print text in quotes, or a whole sentence, with `ဟု`.
- How numbers, true/false values and text look when printed.
- What the compiler reports when we pick the other particle.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၄၂ ကို ပြသမည်။
"မင်္ဂလာပါ" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၂
မင်္ဂလာပါ
```

The two sentences print two lines, and use the two forms of a print sentence:

```
<value> ကို ပြသမည်။
"<text>" ဟု ပြသမည်။
```

| Zaka | Reads as | Use it for |
| --- | --- | --- |
| `ကို` | (the object) | after a plain value: a number, a character, a sum |
| `ဟု` | "so" (quoting) | after text in quotes, or a sentence that ends in `သည်` or `ပါ` |
| `ပြသမည်` | "will show" | print, then go to a new line |
| `။` | full stop | ends the sentence |

## Plain values with `ကို`

Use `ကို` after a whole number, a decimal number, a character in single quotes, or a
sum. The last print statement below adds two numbers and prints the result.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၄၂ ကို ပြသမည်။
၃.၁၄ ကို ပြသမည်။
'က' ကို ပြသမည်။
၁၀ + ၅ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၂
၃.၁၄
က
၁၅
```

## Text and sentences with `ဟု`

Use `ဟု` after text in double quotes or a clause such as `၃ သည် ၂ ထက် ကြီးသည်`
("3 is greater than 2"). Use `""` with `ဟု` to print a blank line.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ" ဟု ပြသမည်။
၃ သည် ၂ ထက် ကြီးသည် ဟု ပြသမည်။
"" ဟု ပြသမည်။
"ပြီးပါပြီ" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ
မှန်

ပြီးပါပြီ
```

Since 3 is greater than 2, the second print statement prints `မှန်` (true).
See [True and false](../values/booleans.md) for more examples.

## How values look when printed

- Numbers print with Burmese digits, even when the program writes `123`.
- Decimal numbers keep their decimal point, even when the number is whole: `၁၀.၀`.
- True and false print as `မှန်` and `မမှန်`.
- Characters and text print unchanged, without quotes.

## Picking the other particle

When we use the other particle, the program still runs, but the compiler gives a
warning because the sentence sounds less natural in Burmese. A warning starts with
`warning`, and this program still prints `၅`:

```zaka-warning ZK0506
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၅ ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0506]: use "ကို" to print this value
```

Text in quotes with `ကို` gets the other warning:

```zaka-warning ZK0505
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ" ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0505]: a sentence or a string is quoted with "ဟု", not marked with "ကို"
```

## Details

- **Text in a variable.** Once text is in a [variable](../variables/README.md), we print
  the variable with `ကို`, because the variable is a plain value and not text in quotes.
  See [Printing](../../manual/language/statements.md#printing).
- **Lists and structs.** Later chapters print lists between `[` and `]` and structs with
  their field names, and text inside a list or a struct prints in quotes.
- **Other wording warnings.** The [Natural Zaka](../natural-zaka/README.md) chapter
  lists each warning about wording, and [Natural
  wording](../../manual/language/naturalness.md) states the rules in full.

## Recap

- We write `<value> ကို ပြသမည်။` to print a plain value, such as a number or a sum.
- We write `"<text>" ဟု ပြသမည်။` to print text, and we use `ဟု` after a whole sentence
  too.
- Each print sentence prints one value and then starts a new line.
- The other particle still works, but the compiler gives a warning.

## Hands on

- Print an age with `ကို`, then print a sentence about that age in quotes with `ဟု`.
- Print `၇ / ၂` and `၇.၀ / ၂`. What makes the results differ? (The answer is in
  [Arithmetic](../operators/arithmetic.md).)
- Print two blank lines between two greetings.

Next: [Zaka sentences](../sentences/README.md).
