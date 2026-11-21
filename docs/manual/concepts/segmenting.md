---
title: Finding words in Burmese
description: How the compiler finds the edges of vannas and words in Burmese text that has no spaces, and where Zaka needs a space.
---

# Finding words in Burmese

English puts a space between words, so a program can split a line into words with one
rule. Burmese does not, and a person who reads Burmese finds where each word ends because
that person knows the words. A compiler does not know them, and Zaka gives it no word list. This
page explains what the compiler does instead, in three steps, and what that asks of us
when we write a program.

## Three steps, from letters to words

```text
 text                    အသက်ကို ပြသမည်။
   │  step 1: vannas  (zakac segment)
   ▼
 အ │ သက် │ ကို │ ␣ │ ပြ │ သ │ မည် │ ။
   │  step 2: words      (zakac lex)
   ▼
 အသက်ကို │ ပြသမည် │ ။          one word for each run with no space and no sign inside
   │  step 3: look each word up
   ▼
 name       print    full stop
```

1. **Vannas.** The *segmenter* cuts the text at the edges of vannas. For this it needs the
   shape of the Burmese script alone, which Unicode defines, and not the meaning of a word.
2. **Words.** The lexer joins vannas that touch into *words*, where a word is a run with no
   space, no sign and no change of script inside it.
3. **Lookup.** The lexer looks each word up in the list of reserved words. It splits a
   word made of reserved words alone into those words, and treats each other word as a
   name.

## Step 1: vannas

A Burmese *vanna* (ဝဏ္ဏ) is the unit of the script: a consonant with the marks that
belong to it, such as a medial, a vowel sign, a tone mark, and a final consonant silenced
with an asat (်). The segmenter reads one character at a time and starts a new vanna when
a character can begin one:

- A **consonant** or an independent vowel begins a new vanna, *unless* the character
  before it is a virama (the consonant is stacked), or the consonant itself has an asat or
  a virama after it (it is a final consonant, as `က်` is in `သက်`). The segmenter looks a
  short distance ahead to check this, because a dot below (့) can sit between a consonant
  and its asat.
- A **medial, vowel sign or tone mark** joins the vanna before it.
- A **digit**, and each mark that can stand alone, such as `၊ ။ ၌ ၍ ၏`, is a token
  on its own.
- A character that is **not Burmese**, such as a space, a Latin letter or a sign, is a
  token on its own too. With `--coalesce-non-burmese`, a run of these characters is one
  token.

We can see the cut with `zakac segment`:

```console
$ printf 'အသက် ကို' | zakac segment
# zakac-segment-artifact v1
name: Stdin
source: အသက် ကို
tokens:
- kind: burmese_vanna
  lexeme: အ
  span:
    lo: 0
    hi: 3
- kind: burmese_vanna
  lexeme: သက်
  span:
    lo: 3
    hi: 12
- kind: non_burmese_char
  lexeme: " "
  span:
    lo: 12
    hi: 13
- kind: burmese_vanna
  lexeme: ကို
  span:
    lo: 13
    hi: 22
```

A span counts octets (bytes) of UTF-8, and a Burmese letter uses three of them. The token
kind `burmese_vanna` is a vanna, `burmese_digit` is a digit, and `non_burmese_char` is
the rest. Here are some words and their cuts:

| Text | Vannas |
| --- | --- |
| `အသက်ကို` | `အ` `သက်` `ကို` |
| `ထက်ကြီးသည်` | `ထက်` `ကြီး` `သည်` |
| `တန်ဖိုးကို` | `တန်` `ဖိုး` `ကို` |
| `မြန်မာ` | `မြန်` `မာ` |
| `ကျောင်းသား၊` | `ကျောင်း` `သား` `၊` |
| `ပြန်လည်စွန့်လွှတ်မည်` | `ပြန်` `လည်` `စွန့်` `လွှတ်` `မည်` |
| `သည်ကျော်မည်` | `သည်` `ကျော်` `မည်` |

The last row shows a rule that needed care. In `ကျော်` the asat belongs to the vowel, not
to a silenced consonant, so `ကျော်` is one vanna, and the segmenter can still split
`သည်ကျော်မည်` between `သည်` and `ကျော်`.

## Step 2: words

The lexer joins neighbouring vannas into a *word*, and a word ends at:

- **a space.** One space, several spaces, a tab, a line break and the zero width space
  (U+200B) all count the same. The macOS Burmese input method writes that last one for
  `Shift` + `Space`.
- **a sign that stands alone.** `၊ ။ ၏`, quotes, brackets and the arithmetic signs are
  each a word, so `ပြသမည်။` needs no space, and neither does `၁+၂`.
- **a change of script.** `abcက` is two words, and so is `၁၂34`.

So `အသက်ကို` is one word, whatever it means to a person. The lexer cannot know that it is
a name followed with a particle.

## Step 3: reserved words

The lexer looks each word up in this order, and uses the first test that matches:

| Order | Test | Result |
| --- | --- | --- |
| 1 | the word is a comment marker, or a quote opens | a comment, a text literal or a character |
| 2 | the word is a sign | punctuation |
| 3 | the word is a whole reserved word | that reserved word |
| 4 | the word is made of reserved words alone | those reserved words, in order |
| 5 | the word starts with a digit | a number |
| 6 | the word has the shape of a name | a name |

Step 4 is the one that matters here. The lexer tries to cut the word at vanna edges into
reserved words, and picks the cut with the fewest pieces. It never cuts inside a vanna,
so `မှာ` cannot lose its `မှ`. It refuses a cut where each piece is a particle, so
`ကိုကို` remains a name. And one vanna that is not part of a reserved word is enough to
reject the whole cut:

| Written | Lexed as |
| --- | --- |
| `တန်ဖိုးကို` | `တန်ဖိုး` `ကို` |
| `အောက်ပါတို့ကို` | `အောက်ပါ` `တို့ကို` |
| `ထက်ကြီးသည်` | `ထက်` `ကြီးသည်` |
| `အသက်ကို` | one name, because `အသက်` is not reserved |
| `အသက်သည်` | one name, for the same reason |
| `ကိုကို` | one name, because each piece is a particle |

## What it asks of us

**A space between a name we made up and the particle after it.** Without the space, the
lexer sees one word where we meant two, and no cut can fix that:

```zaka-error ZK0203 repl=on
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
အသက်ကို ပြသမည်။
```

```text
error[ZK0203]: this statement does not end in a statement verb, found "ပြသ"
help: write "အသက် ကို" with a space: a name joined to a particle is read as one word
```

The help line of the message points at the joined word and shows the fix.

**No space is needed** between reserved words, or around a sign:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၅ သည် ၃ ထက်ကြီးသည် ဟု ပြသမည်။      // ထက်ကြီးသည်: three reserved words, joined
၁+၂ ကို ပြသမည်။                     // no space around the sign
၁ နှင့် ၂ ၏ ပေါင်းလဒ် ကို ပြသမည်။      // ၏ is a word on its own, so it needs no space
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
၃
၃
```

The simple rule is that a space separates two words, and a name that we made up has a
space on each side. Learn gives the same advice in [Spaces between
words](../../learn/sentences/spacing.md).

## The reason for no word list

With a word list of Burmese, the compiler could cut `အသက်ကို` without help. But then the
meaning of a line would depend on the word list, which grows over time and which no two
speakers agree on. Zaka asks for less on purpose: the compiler needs the rules of the
script, a fixed list of reserved words, and one space from us where a name meets a
particle, so the compiler gives a line the same cut each time.

## Running it

- `zakac segment file.zaka` writes the vannas of a file, `-o` picks the output file, and
  `--output-format text|raw` picks its form.
- `zakac lex file.zaka` writes the tokens that the words became.

[The lexer](../internals/lexer.md) describes the code behind both commands.
