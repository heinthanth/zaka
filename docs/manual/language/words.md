---
title: Source text and words
description: How the compiler reads a file - encoding, sentences, spaces, comments, names, literals and reserved words.
---

# Source text and words

Before the compiler asks what a program means, it cuts the file into words (tokens).
Burmese is written without spaces between words, and the compiler has no word list, so a
few plain rules decide where one word ends and the next one begins. This page states those
rules, from the file itself up to the literals. [Zaka sentences](../../learn/sentences/README.md)
and [Spaces between words](../../learn/sentences/spacing.md) in the Learn book introduce
them with gentle examples.

## Source files

A Zaka program is a text file in UTF-8, saved with the extension `.zaka`. The compiler
first puts the text into one standard form:

1. It drops a leading `U+FEFF` mark.
2. It counts `CR LF` and a lone `CR` each as one line break.
3. It normalizes the text to Unicode NFC and puts the signs of a Burmese vanna (letter
   cluster) in one fixed order, with the asat `်` before the dot below `့`, as in `နှင့်`
   and `သည့်`.

Because of the last step, two spellings that look the same give the same words, so a
name or a reserved word written with its signs in another order still matches.

## Sentences and the full stop

```text
<statement>။ <statement>။
```

A program is a list of statements (sentences), and each one ends with the Burmese full
stop `။`. A line break does not end a statement, so one statement can cover several lines,
and several statements can share a line.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"တစ်" ဟု ပြသမည်။ "နှစ်" ဟု ပြသမည်။
၁ +
၂ ကို
ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
တစ်
နှစ်
၃
```

A missing `။` is ZK0201 ("expected `။`"). To separate the arguments, the parameters and
the cells of a list literal, we use the Burmese comma `၊`, because a Western comma is not
a Zaka character.

```zaka-error ZK0101
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁, ၂ ပါသော စာရင်း ဖြစ်ပါစေ။
```

```text
error[ZK0101]: invalid character in ","
  = help: lists are separated with "၊", not ","
```

## Words and spaces

The lexer works in three steps: it cuts the text into vannas, digits and single other
characters, then it joins the pieces that touch into words, and then it classifies each
word. These rules decide the cuts:

| Rule | Example |
| --- | --- |
| White space separates words. | `အသက် ကို` is two words |
| Some characters are words on their own, even with no space beside them. | `၁+၂`, `ပြသမည်။` |
| A name runs to the next white space or the next character that is a word on its own. | `အမှတ်_၁`, `age` |
| A word made of reserved words alone is split into those reserved words. | `ထက်ကြီးသည်` is `ထက်` `ကြီး` `သည်` |
| A word with a vanna that is not part of a reserved word is a name. | `အသက်ကို` is one name |
| The words of a reserved phrase can have white space between them or none. | `စားသော စားလဒ်`, `စားသောစားလဒ်` |

- The seven white space characters are TAB, LINE FEED, VERTICAL TAB, FORM FEED, CARRIAGE
  RETURN, SPACE and ZERO WIDTH SPACE (`U+200B`, which the macOS Burmese input source
  enters for `Shift` + `Space`). One of them or a run of them counts the same, and none of
  them is a token.
- The characters that are words on their own are `၊`, `။`, `၏` and each ASCII punctuation
  character except `_`, such as `( ) + - * / % ; , .` and the quote marks. A pair that
  forms one sign (`<=  >=  ==  !=  <<  >>  &&  ||`) is joined when its two characters
  touch.
- A name can mix scripts, as long as nothing separates the pieces.
- The split of reserved words picks the fewest pieces: `တန်ဖိုးကို` is `တန်ဖိုး` `ကို`. A word
  that contains just particles, such as `ကိုကို`, is not split.

```text
အသက် ကို ပြသမည်။     အသက်   |  ကို   |  ပြသမည်  |  ။       a name, two reserved words, a stop
အသက်ကို ပြသမည်။     အသက်ကို  |  ပြသမည်  |  ။                one name: no reserved split
ထက်ကြီးသည်          ထက်  |  ကြီး  |  သည်                  reserved words alone: split
၁+၂                 ၁  |  +  |  ၂                         signs are cut out
```

These rules lead to the one habit that a Zaka writer needs: we put a space between a name
and the particle (postposition) after it. When the joined word is not a known name, the
compiler guesses that a space is missing and points it out:

```zaka-error ZK0203
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
အသက်ကို ပြသမည်။
```

```text
error[ZK0203]: this statement does not end in a statement verb, found "ပြသ"
help: write "အသက် ကို" with a space: a name joined to a particle is read as one word
```

### Polite endings

A polite ending (`သည်`, `ပါ`, `ပါသည်`) written attached to a comparison verb, or to `မှန်`,
`မမှန်` or `မဟုတ်`, is part of that word, and the lexer keeps it as a note on the token.
With a space before it, `သည်` is the subject particle instead. [Natural
wording](naturalness.md) states which ending fits where. Once a program allows spoken
Burmese, the spoken endings `တယ်` and `ပါတယ်` mean the same as `သည်` and `ပါသည်` ([Spoken
Burmese](spoken.md)).

## Comments

```text
// <comment>
/// <comment>
မှတ်ချက် <comment>
```

A comment starts at `//`, at `///`, or at the word `မှတ်ချက်` ("note"), and it runs to the
end of the line. A comment is not a token, and Zaka has no comment that spans lines. A
comment marker inside a string or character literal is plain text, and `မှတ်ချက်` starts a
comment just when it is a word on its own.

```zaka
// two slashes
/// three slashes
မှတ်ချက် the word for "note"
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"http://zaka.example // not a comment" ဟု ပြသမည်။   // a comment after a statement
၁ ကို ပြသမည်။ မှတ်ချက် also after a statement
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
http://zaka.example // not a comment
၁
```

## Names

A name (identifier) starts with a letter of a script, an emoji, `_`, or a Burmese
character that is not a digit. After the first character, it can also contain digits,
Burmese or Western. Upper and lower case differ, so `Age` and `age` are two names. A name
cannot be just a reserved word or a row of reserved words.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် Age သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
ကိန်းရှင် age သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
ကိန်းရှင် အမှတ်_၁ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၄ ဖြစ်ပါစေ။
ကိန်းရှင် 🍎 သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
အသက် + Age + age + အမှတ်_၁ + 🍎 ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၅
```

### Raw names

```text
{<name>}
```

To use reserved words, spaces or signs in a name, we write the name between braces, and
we call it a *raw name*. A raw name cannot contain a `}` or a line break, and it cannot be
blank. It is a name in each place: it is never taken as a reserved word, as a word of a
preposition, or as a function with a call ending glued to it, and after `၏` it refers to a
field alone ([What the flexible grammar
costs](functions-sentences.md#what-the-flexible-grammar-costs)). Inside a string, `{…}` is a
template slot, and a slot contains an expression, so `"{ {ကို ပြသမည်} }"` prints the value
of the raw name. A backtick outside a string is ZK0101.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် {ကို ပြသမည်} သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇ ဖြစ်ပါစေ။
ကိန်းရှင် {the total} သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၉ ဖြစ်ပါစေ။
{ကို ပြသမည်} + {the total} ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၆
```

| Mistake | Code |
| --- | --- |
| a raw name with no closing `}` on its line | ZK0108 |
| `{}` with nothing between the braces | ZK0109 |
| a plain reserved word written as a name | ZK0201 ("expected a name") |

```zaka-error ZK0109
ကိန်းရှင် {} သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
```

### Two spellings of one word

`ဒေတာ` and `အချက်အလက်` ("data") are one word in each name and each reserved word:

| Written | Same as |
| --- | --- |
| `ဒေတာအစုအဝေး` | `အချက်အလက်အစုအဝေး` |
| `ဒေတာအမျိုးအစားကွဲ` | `အချက်အလက်အမျိုးအစားကွဲ` |
| `ဒေတာအမျိုးအစား` | `အချက်အလက်အမျိုးအစား` |
| `ဒေတာဆဲလ်` | `အချက်အလက်ဆဲလ်` |
| `ဒေတာ` alone: a reserved word after `ထဲမှ`, `ထဲတွင်` and `အမှားအယွင်း`, and a name elsewhere | `အချက်အလက်` |
| a variable `ဒေတာအရေအတွက်` | the variable `အချက်အလက်အရေအတွက်` |

The compiler keeps the spelling `အချက်အလက်`, so messages and printed struct names use it.
Text in a string literal and in a comment keeps the spelling that we wrote.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဒေတာအရေအတွက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
အချက်အလက်အရေအတွက် + ၁ ကို ပြသမည်။
"ဒေတာ" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၆
ဒေတာ
```

Since the two spellings make one name, declaring `ဒေတာအရေအတွက်` and then
`အချက်အလက်အရေအတွက်` in one scope is ZK0302.

## Numbers

```text
[အခြေ <base> ဖြင့်] <digits> [ဟူသော <kind> တစ်<measure word>]
```

A number literal is a run of digits, Burmese (`၀` to `၉`) or Western (`0` to `9`), but one
number uses one script (ZK0103). A literal has up to three parts, in this order:

| Part | Written | Rules |
| --- | --- | --- |
| base | `အခြေ ၁၆ ဖြင့်` | optional; a base from 2 to 36, itself written in base 10 (ZK0218) |
| digits | `၂၅၅`, `FF`, `၃.၁၄`, `1.5e3` | in base 10, a fraction `.` and an exponent are allowed |
| data kind | `ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု` | optional; a plain number kind with its [measure word](kinds.md#measure-words) (ZK0216, ZK0217, ZK0234); another data kind is ZK0222 |

- Digits above 9 are the letters `A` to `Z`, in either case, and a digit that is not below
  the base is ZK0219.
- A fraction needs digits on both sides of the point: `၁.၅`, not `၁.` or `.၅`.
- An exponent (`e` or `E`, an optional sign, then digits) needs Western digits, so
  `1.5e3` works and `၂.၅e3` is ZK0102. A fraction or an exponent makes a decimal number,
  and a base prefix cannot go with one.
- Zaka has no negative literal: `-၅` is the minus sign applied to `၅`, and so is
  `အနုတ် ၅`, and the compiler works it out before the program runs. A data kind after the
  digits belongs to the whole literal, the minus sign included:
  `-၁၂၈ ဟူသော ၈ ဘစ် ကိန်းပြည့် တစ်ခု`.
- A literal with no data kind is *flexible*: it gets the data kind that its place wants
  ([Expressions](expressions.md#atoms)). A literal that does not fit its data kind is
  ZK0220 when the data kind is written after it, and ZK0406 when the place gave the data
  kind.
- Characters, text and true/false values have one data kind each, and we write no data
  kind after them.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁၂၃ ကို ပြသမည်။                                    // Burmese digits
123 ကို ပြသမည်။                                     // Western digits: the same number
အခြေ ၂ ဖြင့် ၁၀၁၀ ကို ပြသမည်။                        // 1010 in base 2
အခြေ ၁၆ ဖြင့် ff ကို ပြသမည်။                          // letters, either case
အခြေ ၁၆ ဖြင့် FF ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြသမည်။   // base, then digits, then data kind
၃.၁၄ ကို ပြသမည်။
1.5e3 ကို ပြသမည်။
အနုတ် ၅ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၂၃
၁၂၃
၁၀
၂၅၅
၂၅၅
၃.၁၄
၁၅၀၀.၀
-၅
```

Numbers print with Burmese digits, whatever digits the program used.

```zaka-error ZK0103
၁၂.34 ကို ပြသမည်။
```

```zaka-error ZK0219
အခြေ ၂ ဖြင့် ၁၂ ကို ပြသမည်။
```

```text
error[ZK0219]: "၂" is not a digit in base 2
```

## Characters and strings

```text
'<character>'
"<text>"
```

A character literal contains one Unicode scalar value in single quotes: `'က'`, `'A'`,
`'🔥'`. A Burmese vanna with signs, such as `ကို`, is several scalar values, so `'ကို'` is
ZK0107. A string literal is text in double quotes, of the data kind `စာသား` (string)
([Text](text.md)). A literal must close on the line where it opens: ZK0104 for a string
and ZK0105 for a character.

Escapes work in both. The backslash forms are for developers who know C, and the brace
forms are the Burmese spelling. A brace is reserved inside a literal, so we double a plain
`{` or `}`.

| Escape | Means |
| --- | --- |
| `{နောက်တစ်ကြောင်း}` or `\n` | line feed |
| `\r`, `\t`, `\0` | carriage return, tab, the null character |
| `\\`, `\'`, `\"` | a backslash, a single quote, a double quote |
| `{{`, `}}` | a brace `{`, a brace `}` |
| `\uHHHH` or `\uHHHHHHHH` | the code point with 4 or 8 hex digits |
| `{ယူနီကုဒ် HHHH}` or `{ယူနီကုဒ် HHHHHHHH}` | the same, in Burmese |

A code point must be a scalar value, so a surrogate, a value above `10FFFF` and a digit
count other than 4 or 8 are errors. In a string, another `{…}` is a template slot that
contains an expression ([Template literals](text.md#template-literals)), and it ends at
the `}` that matches its `{` on the same line. A character literal has no template slots,
but its braces follow the same rule: `'{{'` and `'}}'` are the brace characters, and
another `{…}` is ZK0106, as are a lone `}` and a `{` with no `}` after it on the line.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"တစ်{နောက်တစ်ကြောင်း}နှစ်" ဟု ပြသမည်။
"a\\b" ဟု ပြသမည်။
"{{ }}" ဟု ပြသမည်။
'က' ကို ပြသမည်။
'{ယူနီကုဒ် 1000}' ကို ပြသမည်။
"x\u00000041y" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
တစ်
နှစ်
a\b
{ }
က
က
xAy
```

```zaka-error ZK0106
'{မသိ}' ကို ပြသမည်။
```

```zaka-error ZK0107
'ကို' ကို ပြသမည်။
```

```text
error[ZK0107]: a character literal must hold one character, found 3
```

```zaka-error ZK0104
"မပိတ်ထားသော စာ ဟု ပြသမည်။
```

## Lists in words

```text
<a> ကို
<a> နှင့် <b> တို့ကို
<a>၊ <b> နှင့် <c> တို့ကို
```

To write a list in words, we put `၊` between its items and `နှင့်` before the last one, as
Burmese prose does. The particle that closes the list matches the count: one item gets the
particle alone, and two or more get it with `တို့` before it.

| Items | Form |
| --- | --- |
| one | `က ကို` |
| two | `က နှင့် ခ တို့ကို` |
| three or more | `က၊ ခ နှင့် ဂ တို့ကို` |

The rule applies to each list in words:

- the cells of a word list (`ပါသော စာရင်း`, `တို့ပါသော စာရင်း`) and the rows of a table
  (map);
- the pieces of a joined text;
- the arguments of a call (`ဖြင့်` and `တို့ဖြင့်`, or `ကို` and `တို့ကို` before `သုံး၍`);
- the names of an import;
- the fields of a struct and their values, and the members of an enumeration (`ပါဝင်သည်`
  and `တို့ပါဝင်သည်`);
- the cases of a switch arm.

A list without its `နှင့်` is ZK0245, and a closing particle that does not match the count
is ZK0246.

An item that contains its own `နှင့်`, such as the sum `က နှင့် ခ ၏ ပေါင်းလဒ်`, goes in
parentheses, so the `နှင့်` of the list is never taken as part of it. Lists in notation,
`[က၊ ခ]` and `f(က၊ ခ)`, keep a plain `၊`, and so do the parameters that end in
`လည်းကောင်း`.

```zaka
ပေါင်း ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က + ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ၁၊ ၂ နှင့် ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းများ ကို ပြသမည်။
ပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို ၃ နှင့် (၁ နှင့် ၂ ၏ ပေါင်းလဒ်) တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ကိန်းရှင် စာ သည် "က"၊ "ခ" နှင့် "ဂ" တို့ကို ဆက်ထားသည့် စာသား ဖြစ်ပါစေ။
စာ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၂၊ ၃]
၆
ကခဂ
```

## Second spellings

Some parts of the reserved words have a second or a third spelling that means the same.
Each spelling is plain Zaka and each one is reserved, so a reserved word written with one
of them is the same word as one written with another.

| Parts | Where | Example |
| --- | --- | --- |
| `၍`, `ပြီး`, `ကာ` | the end of a verb that the sentence goes on from; not `အကယ်၍`, and not the block closer `ပြီးဆုံးပါပြီး` | `လက်ခံပြီး`, `လက်ခံ၍`, `လက်ခံကာ`; `ဖြစ်၍`, `ဖြစ်ပြီး`, `ဖြစ်ကာ` |
| `သော`, `သည့်` | the end of `သုံးသော`, `လက်ခံသော` and `ပြန်ပေးသော` | `သုံးသည့်` |
| `ပြီးဆုံးပါပြီး`, `ပြီးဆုံးပါပြီ`, `ပြီးပါပြီး`, `ပြီးပါပြီ` | the block closer | |
| `မင်မိုရီ`, `မှတ်ဉာဏ်` | "storage" | |
| `တွင်`, `၌` | "at", "in"; `၌` is the written form | `ထဲ၌`, `ဤနေရာ၌` |
| `သည်`, `၏` | the end of a verb that ends a sentence; `၏` is the written form. The word `သည်` on its own keeps one spelling, since `၏` on its own is "of" | `ဖြစ်၏`, `ပါဝင်၏`, `တို့ပါဝင်၏` |

The spelling that a word uses changes nothing, but its place does. `ဖြစ်၍` (or `ဖြစ်ပြီး`,
`ဖြစ်ကာ`) after the data kind of a declaration starts its value when `တန်ဖိုးသည်` follows,
and it starts the condition of a [generic loop](control-flow.md#generic-loops) when another
word follows, while between two conditions it means "and".

`ဒေတာ` and `အချက်အလက်` are one word in names too ([Names](#names)). The spoken spellings
of the parts, such as `မယ်` for `မည်`, are on [Spoken Burmese](spoken.md).

## Reserved words

The words that Zaka gives a meaning to cannot be names, and [Reserved
words](../reference/reserved-words.md) lists each of them. To use a reserved word as a
name, we put it between braces ([Raw names](#names)).

Each diagnostic on this page has a code from `ZK0101` to `ZK0109`, and [Error
messages](../reference/errors.md) lists them with the others.
