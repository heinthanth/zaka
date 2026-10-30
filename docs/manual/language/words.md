---
title: Source text and words
description: How the compiler reads a file - encoding, sentences, spaces, comments, names, literals and reserved words.
---

# Source text and words

Before the compiler asks what a program means, it cuts the file into *words*. Burmese
is written without spaces, and the compiler has no word list, so a few plain rules
decide where one word ends and the next begins. This page states those rules, from the
the file itself up to the literals.

## Source files

A Zaka program is a text file in UTF-8, saved with the extension `.zaka`. The compiler
first puts the text into one standard form:

- a leading `U+FEFF` mark is dropped;
- `CR LF` and a lone `CR` each count as one line break;
- the text is normalized to Unicode NFC, and the signs of a Burmese vanna are put in
  one fixed order. The asat `်` comes before the dot below `့`, as in `နှင့်` and
  `သည့်`.

The last step is the reason two spellings that look the same lex the same. A name or a
reserved word written with its signs in another order still matches.

## Sentences and the full stop

A program is a list of statements. Each statement is a sentence that ends with the
Burmese full stop `။`. A line break does **not** end a statement, so one statement can
cover several lines and several statements can share a line.

```zaka
"တစ်" ဟု ပြသပါ။ "နှစ်" ဟု ပြသပါ။
၁ +
၂ ကို
ပြသပါ။
```

```text
တစ်
နှစ်
၃
```

A missing `။` is reported as ZK0201 ("expected `။`"). The list separator is `၊`, which
is used between arguments, parameters and the cells of a list literal; a comma is not a
Zaka character.

```zaka-error ZK0101
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၁, ၂ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
```

```text
error[ZK0101]: invalid character in ","
  = help: lists are separated with "၊", not ","
```

## Words and spaces

The lexer works in three steps: it cuts the text into vannas (Burmese letter
clusters), digits and single other characters; it joins the pieces that touch into
*words*; and it classifies each word. These rules decide the cuts.

- **White space separates words.** The seven white space characters are TAB, LINE FEED,
  VERTICAL TAB, FORM FEED, CARRIAGE RETURN, SPACE and ZERO WIDTH SPACE (`U+200B`, which
  the macOS Burmese input source enters for `Shift` + `Space`). One of them or a run of
  them count the same, and none is a token.
- **Some characters are words on their own.** `၊`, `။`, `၏` and each ASCII punctuation
  character except `_` are cut out even with no space beside them, such as `( ) + - * /
  % ; , .` and the quote marks. That is the reason `၁+၂` and `ပြသပါ။` need no spaces. A pair
  that forms one sign (`<=  >=  ==  !=  <<  >>  &&  ||`) is joined when its two
  characters touch.
- **A name runs to the next white space or lone character.** The name can mix scripts
  (`အမှတ်_၁`, `age`) as long as nothing separates the pieces.
- **A word made of reserved words alone is split.** `ထက်ကြီးသည်` is `ထက်` `ကြီး`
  `သည်`, and `တန်ဖိုးကို` is `တန်ဖိုး` `ကို`. The split picks the fewest pieces, and a
  word that holds just particles (such as `ကိုကို`) is not split.
- **A word with a vanna that is not reserved is a name.** `အသက်ကို` is one name, not
  `အသက်` then `ကို`.
- **The words of a reserved phrase** such as `စားသော စားလဒ်` or `အောက်ပါတို့ကို လုပ်ဆောင်ပါ`
  can have white space between the words or none; both read the same.

```text
အသက် ကို ပြသပါ။     အသက်   |  ကို   |  ပြသပါ  |  ။       a name, two reserved words, a stop
အသက်ကို ပြသပါ။     အသက်ကို  |  ပြသပါ  |  ။                one name: no reserved split
ထက်ကြီးသည်          ထက်  |  ကြီး  |  သည်                  reserved words alone: split
၁+၂                 ၁  |  +  |  ၂                         signs are cut out
```

The fourth rule gives the one habit a Zaka writer needs: **put a space between a name
and the particle after it.** When the joined word is not a known name, the compiler
guesses that a space is missing and reports it:

```zaka-error ZK0206
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
အသက်ကို ပြသပါ။
```

```text
error[ZK0206]: "ပြသပါ" needs "ကို|ဟု" after operand 1, found no particle
help: write "အသက် ကို" with a space: a name joined to a particle is read as one word
```

A polite ending (`သည်`, `ပါ`, `ပါသည်`) written attached to a comparison verb, to
`မှန်`, `မမှန်` or `မဟုတ်` is part of that word; the lexer keeps it as a note on the
token. With a space, `သည်` is the subject marker instead. [Natural wording](naturalness.md)
tells which ending reads well where.

## Comments

A comment starts at `//`, at `///`, or at the word `မှတ်ချက်` ("note"), and runs to the
end of the line. A comment is not a token. There is no comment that spans lines. A marker
inside a string or character literal is just text, and `မှတ်ချက်` starts a comment just
when it is a word on its own.

```zaka
// two slashes
/// three slashes
မှတ်ချက် the word for "note"
"http://zaka.example // not a comment" ဟု ပြသပါ။   // a comment after a statement
၁ ကို ပြသပါ။ မှတ်ချက် also after a statement
```

```text
http://zaka.example // not a comment
၁
```

## Names

A name starts with a letter of a script, an emoji, `_`, or a Burmese character that is
not a digit. After the first character it can also hold digits (Burmese or Western).
Upper and lower case differ: `Age` and `age` are two names. A name cannot be just a
reserved word or a row of reserved words.

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် Age သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
ကိန်းရှင် age သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
ကိန်းရှင် အမှတ်_၁ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၄ ဖြစ်ပါစေ။
ကိန်းရှင် 🍎 သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
အသက် + Age + age + အမှတ်_၁ + 🍎 ကို ပြသပါ။
```

```text
၁၅
```

### Raw names

A name written between backticks is a *raw name*. It can hold reserved words, spaces
and signs. It cannot hold a backtick or a line break, and it cannot be blank.

```zaka
ကိန်းရှင် `ကို ပြသပါ` သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၇ ဖြစ်ပါစေ။
ကိန်းရှင် `the total` သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၉ ဖြစ်ပါစေ။
`ကို ပြသပါ` + `the total` ကို ပြသပါ။
```

```text
၁၆
```

A raw name with no closing backtick on its line is ZK0108, and two backticks in a row
are ZK0109. Writing a plain reserved word as a name gives ZK0201 ("expected a name").

```zaka-error ZK0109
ကိန်းရှင် `` သည် ကိန်းပြည့် ဖြစ်ပါစေ။
```

## Numbers

A number literal is a run of digits. The digits are Burmese (`၀` to `၉`) or Western
(`0` to `9`), but one number holds one script alone (ZK0103). A literal has up to three
parts in a fixed order:

| Part | Written | Notes |
| --- | --- | --- |
| base | `အခြေ ၁၆ ဖြင့်` | optional; a base from 2 to 36, itself written in base 10 (ZK0218) |
| kind | `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်` | optional; see [Kinds and values](kinds.md) (ZK0216, ZK0217) |
| digits | `၂၅၅`, `FF`, `၃.၁၄`, `1.5e3` | in base 10, a fraction `.` and an exponent are allowed |

- Digits above 9 are the letters `A` to `Z`, in either case. A digit not below the base is
  ZK0219.
- A fraction needs digits on both sides of the point: `၁.၅`, not `၁.` or `.၅`.
- An exponent (`e` or `E`, an optional sign, digits) needs Western digits: `1.5e3` works,
  `၂.၅e3` is ZK0102. A fraction or an exponent makes a decimal number, and a base prefix
  cannot go with one.
- There is no negative literal. `-၅` is the minus sign applied to `၅`, and so is
  `အနုတ် ၅`. The compiler works it out before the program runs.
- A literal with no kind prefix is *flexible*: it takes the kind that its place wants
  (see [Expressions](expressions.md)). A literal that does not fit its kind is an error:
  ZK0220 when the kind is written in front of it, and ZK0406 when the place gave the
  kind.

```zaka
၁၂၃ ကို ပြသပါ။                                    // Burmese digits
123 ကို ပြသပါ။                                     // Western digits: the same number
အခြေ ၂ ဖြင့် ၁၀၁၀ ကို ပြသပါ။                        // 1010 in base 2
အခြေ ၁၆ ဖြင့် ff ကို ပြသပါ။                          // letters, either case
အခြေ ၁၆ ဖြင့် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် FF ကို ပြသပါ။   // base, then kind, then digits
၃.၁၄ ကို ပြသပါ။
1.5e3 ကို ပြသပါ။
အနုတ် ၅ ကို ပြသပါ။
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
၁၂.34 ကို ပြသပါ။
```

```zaka-error ZK0219
အခြေ ၂ ဖြင့် ၁၂ ကို ပြသပါ။
```

```text
error[ZK0219]: "၂" is not a digit in base 2
```

## Characters and strings

A *character literal* holds one Unicode scalar value in single quotes: `'က'`, `'A'`,
`'🔥'`. A Burmese vanna with signs, such as `ကို`, is several scalar values, so
`'ကို'` is ZK0107. A *string literal* is text in double quotes. It has no kind of its
own: it can be printed and nothing else (ZK0223). A literal must close on the line where
it opens (ZK0104 for a string, ZK0105 for a character).

Escapes work in both. The backslash forms are for readers who know C, and the brace
forms are the Burmese spelling. A brace is reserved inside a literal, so a plain `{` or
`}` is doubled.

| Escape | Means |
| --- | --- |
| `{နောက်တစ်ကြောင်း}` or `\n` | line feed |
| `\r`, `\t`, `\0` | carriage return, tab, the null character |
| `\\`, `\'`, `\"` | a backslash, a single quote, a double quote |
| `{{`, `}}` | a brace `{`, a brace `}` |
| `\uHHHH` or `\uHHHHHHHH` | the code point with 4 or 8 hex digits |
| `{ယူနီကုဒ် HHHH}` or `{ယူနီကုဒ် HHHHHHHH}` | the same, in Burmese |

A code point must be a scalar value: a surrogate, a value above `10FFFF` and a digit
count other than 4 or 8 are errors. Another `{...}` is ZK0106; braces are kept free so
that a later version can give them a use.

```zaka
"တစ်{နောက်တစ်ကြောင်း}နှစ်" ဟု ပြသပါ။
"a\\b" ဟု ပြသပါ။
"{{ }}" ဟု ပြသပါ။
'က' ကို ပြသပါ။
'{ယူနီကုဒ် 1000}' ကို ပြသပါ။
"x\u00000041y" ဟု ပြသပါ။
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
"{မသိ}" ဟု ပြသပါ။
```

```zaka-error ZK0107
'ကို' ကို ပြသပါ။
```

```text
error[ZK0107]: a character literal must hold one character, found 2
```

```zaka-error ZK0104
"မပိတ်ထားသော စာ ဟု ပြသပါ။
```

## Reserved words

The words that Zaka gives a meaning to cannot be used as names. [Reserved
words](../reference/reserved-words.md) lists each of them. To use a reserved word as a
name, put it between backticks.

Each diagnostic of this page has a code from `ZK0101` to `ZK0109`. [Error
messages](../reference/errors.md) lists them with the others.
