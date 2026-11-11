---
title: Printing
description: Show values with ပြသမည်, and choose between ကို and ဟု.
---

# Printing

`ပြသမည်` ("please show") prints one value and then starts a new line. The value comes
first, then a small word that marks it, then `ပြသမည်`. That small word is either `ကို` or
`ဟု`.

```zaka
// ကို: "show this value"
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၄၂ ကို ပြသမည်။
၃.၁၄ ကို ပြသမည်။
'က' ကို ပြသမည်။
၁၀ + ၅ ကို ပြသမည်။

// ဟု: "quote these words", for text in quotes
"မင်္ဂလာပါ" ဟု ပြသမည်။

// ဟု also quotes a whole sentence, such as "3 is greater than 2"
၃ သည် ၂ ထက် ကြီးသည် ဟု ပြသမည်။

// A pair of quotes with nothing inside prints a blank line
"" ဟု ပြသမည်။
"ပြီးပါပြီ" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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
| `ပြသမည်` | "please show" | print, then go to a new line |

## How values look when printed

- Numbers print with Burmese digits, even when the program writes `123`.
- Decimal numbers keep their decimal point, even when whole: `၁၀.၀`.
- True and false print as `မှန်` and `မမှန်`.
- Characters and text print as is, without quotes.

## Picking the wrong word

Both words work, so the program still runs. But when the other word sounds more
natural, the compiler gives a warning. A warning starts with `warning`; the program
still runs and prints `၅`:

```zaka-warning ZK0506
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၅ ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0506]: a plain value is marked with "ကို", not quoted with "ဟု"
```

```zaka-warning ZK0505
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ" ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0505]: a sentence or a string is quoted with "ဟု", not marked with "ကို"
```

The [Natural Zaka](../natural-zaka/README.md) chapter lists each warning of this kind.

## Hands on

- Print an age with `ကို`, then print a sentence about that age in quotes with `ဟု`.
- Print `၇ / ၂` and `၇.၀ / ၂`. What makes the results differ? (The answer is in
  [Arithmetic](../operators/arithmetic.md).)

Next: [Zaka sentences](../sentences/README.md).
