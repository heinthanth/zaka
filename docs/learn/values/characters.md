---
title: Characters
description: Single characters in single quotes, and escape sequences.
---

# Characters

A *character* (`char`) is one letter, digit, sign or mark, which we write in single
quotes, such as `'က'`, `'A'` or `'?'`. Its data kind (type) is `စာလုံး` ("letter"), and
each Unicode character works, emoji included.

## You will learn

- How to write a character in single quotes.
- The reason some Burmese letters with marks are more than one character.
- How to write a character that is hard to enter, with an escape sequence.
- How braces work inside quotes.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
'က' ကို ပြသမည်။
'A' ကို ပြသမည်။
'春' ကို ပြသမည်။
'🔥' ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
က
A
春
🔥
```

To write a character, we put it between two single quotes:

```
'<character>'
```

| Zaka | Reads as |
| --- | --- |
| `'` | opens the character |
| `က` | the character itself |
| `'` | closes the character |

## One character at a time

A character literal must contain one character, not two and not zero, so `'ab'` is an
error. A Burmese letter with marks, such as `ကို`, is also more than one character to the
computer: `က`, then the vowel mark `ိ`, then `ု`. So `'ကို'` is the same error:

```zaka-error ZK0107
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
'ကို' ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0107]: a character literal must hold one character, found 3
```

For a letter with marks, we use text (string) in double quotes, `"ကို"`, which
[Text](../text/README.md) covers.

## Escape sequences

Some characters are hard to enter, and some have a special meaning inside quotes. To
write these, we use an *escape sequence*, which works in characters and in
[text](../text/README.md) alike:

```
\<letter>
{<Burmese name>}
```

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
'\'' ကို ပြသမည်။                       // a single quote
'{ယူနီကုဒ် 1000}' ကို ပြသမည်။            // the character with Unicode number 1000 (hex): က
"တစ်{နောက်တစ်ကြောင်း}နှစ်" ဟု ပြသမည်။    // a line break inside text
"{{ }}" ဟု ပြသမည်။                     // braces inside text
'{{' ကို ပြသမည်။                       // a brace character
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
'
က
တစ်
နှစ်
{ }
{
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

## Braces inside quotes

Inside quotes, a brace starts a Burmese escape name, so we double a brace to write it as
a character: `'{{'` is the character `{`, in a character and in text alike. In a
character, a name that Zaka does not know is an error:

```zaka-error ZK0106
'{မသိ}' ကို ပြသမည်။
```

```text
error[ZK0106]: unknown escape sequence "{မသိ}"
```

In text, the same braces make a *template slot*, which puts a value into the text (see
[Text](../text/README.md#template-literals)).

## Details

- **The measure word.** We count `စာလုံး` with `လုံး` in place of `ခု`, so a variable of
  this data kind is `စာလုံး တစ်လုံး`, as [Variables](../variables/README.md) shows.
- **Comparing and converting.** We can compare characters with `==`, `!=`, `<` and `>`,
  and convert a character to an integer and back, but we cannot do arithmetic on them.
  See [Characters](../../manual/language/kinds.md#characters).
- **The rules of escapes.** A character must close on the line where it opens (ZK0105),
  and a Unicode number must name a real character. [Characters and
  strings](../../manual/language/words.md#characters-and-strings) states the full rules.

## Recap

- A character is one Unicode character in single quotes, and its data kind is `စာလုံး`.
- A Burmese letter with marks, such as `ကို`, is several characters, so it goes in text.
- We write hard characters with an escape sequence, such as `\n` or `{နောက်တစ်ကြောင်း}`.
- A brace inside quotes is doubled: `'{{'`.

## Hands on

- Print the character with Unicode number `1015`.
- Print one piece of text that comes out on three lines.
- Print `'ကိ'` and read the message. Which count of characters does the message show?

Next: [True and false](booleans.md).
