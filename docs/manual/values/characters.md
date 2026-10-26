---
title: Characters
description: Single characters in single quotes, and escape sequences.
---

# Characters

A *character* is one letter, digit, sign or symbol, written in single quotes: `'က'`,
`'A'`, `'?'`. Its kind is `စာလုံး` ("letter"). Any Unicode character works, emoji
included.

```zaka
'က' ကို ပြသပါ။
'A' ကို ပြသပါ။
'春' ကို ပြသပါ။
'🔥' ကို ပြသပါ။
```

```text
က
A
春
🔥
```

A character literal holds exactly one character. `'ab'` is an error.

## Escape sequences

Some characters are hard to type, or have a special meaning inside quotes. You write
these with an *escape sequence*. Escape sequences work in characters and in
[text](strings.md).

```zaka
'\'' ကို ပြသပါ။                       // a single quote
'{ယူနီကုဒ် 1000}' ကို ပြသပါ။            // the character with Unicode number 1000 (hex): က
"တစ်{နောက်တစ်ကြောင်း}နှစ်" ဟု ပြသပါ။    // a line break inside text
"{{ }}" ဟု ပြသပါ။                     // literal braces
```

```text
'
က
တစ်
နှစ်
{ }
```

| Escape | Means |
| --- | --- |
| `{နောက်တစ်ကြောင်း}` or `\n` | line break ("next line") |
| `\t` | tab |
| `\r` | carriage return |
| `\0` | the null character |
| `\\` | a backslash `\` |
| `\'` | a single quote `'` |
| `\"` | a double quote `"` |
| `{{` and `}}` | a brace `{` or `}` |
| `{ယူနီကုဒ် HHHH}` or `\uHHHH` | the character with Unicode number `HHHH` (4 or 8 hex digits) |

A brace starts a Burmese escape name, so a single `{` or `}` must be doubled. A name
that Zaka does not know is an error:

```zaka-error ZK0106
"{မသိ}" ဟု ပြသပါ။
```

```text
error[ZK0106]: unknown escape sequence "{မသိ}"
```

## Try it

- Print the character with Unicode number `1015`.
- Print one piece of text that comes out on three lines.

Next: [True and false](booleans.md).
