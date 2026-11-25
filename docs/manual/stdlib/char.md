---
title: The character module
description: Tests on one character - digits, English and Burmese letters and white space.
module: စံကား/စာလုံး
---

# The character module

The character module, `"စံကား/စာလုံး"`, tests what kind of character a `စာလုံး` holds. A
lexer or a parser of text reads its input one character at a time and asks these
questions at each step.

Each test takes one `စာလုံး` with the preposition `ကို` and gives a `မှန်မှားတန်ဖိုး`.
Digits count in both scripts: `0` to `9` and `၀` to `၉`.

```zaka
"စံကား/စာလုံး" မှ ဂဏန်းဟုတ်မဟုတ်စစ် နှင့် ဗမာအက္ခရာဟုတ်မဟုတ်စစ် တို့ကို ထည့်သွင်းမည်။
'၇' ကို ဂဏန်းဟုတ်မဟုတ်စစ် သည့် ရလဒ် ကို ပြသမည်။
'7' ကို ဂဏန်းဟုတ်မဟုတ်စစ် သည့် ရလဒ် ကို ပြသမည်။
'က' ကို ဗမာအက္ခရာဟုတ်မဟုတ်စစ် သည့် ရလဒ် ကို ပြသမည်။
'q' ကို ဗမာအက္ခရာဟုတ်မဟုတ်စစ် သည့် ရလဒ် ကို ပြသမည်။
```

```text
မှန်
မှန်
မှန်
မမှန်
```

| Function | Gives `မှန်` for |
| --- | --- |
| `ဂဏန်းဟုတ်မဟုတ်စစ်` | `0` to `9` and `၀` to `၉` |
| `အင်္ဂလိပ်အက္ခရာဟုတ်မဟုတ်စစ်` | `A` to `Z` and `a` to `z` |
| `ဗမာအက္ခရာဟုတ်မဟုတ်စစ်` | `U+1000` to `U+102A`: the consonants `က` to `အ` and the vowels that stand alone, `ဣ` to `ဪ` |
| `နေရာလွတ်ဟုတ်မဟုတ်စစ်` | a space, a tab, `\n` and `\r` |

The code point of a character, and the character of a code point, come from the
built-in conversions between `စာလုံး` and an integer, with no import
([Conversions](../language/conversions.md)).
