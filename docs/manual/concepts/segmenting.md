---
title: Finding words in Burmese
description: How the compiler finds the edges of vannas and words in Burmese text that has no spaces, and where Zaka needs a space.
---

# Finding words in Burmese

English puts a space between words, so a program can split a line with a single rule.
Burmese does not. A reader knows where each word ends because the reader knows the words.
A compiler does not, and Zaka gives it no word list. This page tells what the compiler
does instead, in two steps, and what that asks of the person who writes the program.

## Two steps, from letters to words

```text
 text                    အသက်ကို ပြသပါ။
   │  step 1: vannas  (zakac segment)
   ▼
 အ │ သက် │ ကို │ ␣ │ ပြ │ သ │ ပါ │ ။
   │  step 2: words      (zakac lex)
   ▼
 အသက်ကို │ ပြသပါ │ ။          one word per run without a space or a sign
   │  step 3: look each word up
   ▼
 name       print    full stop
```

1. **Vannas.** The *segmenter* cuts the text at the edges of vannas. It needs the
   shape of the Burmese script alone, which Unicode fixes, and not the meaning of a word.
2. **Words.** The lexer joins vannas that touch into *words*: a word is a run with no
   space, no sign and no change of script inside it.
3. **Lookup.** Each word is looked up in the table of reserved words. A word made of
   reserved words alone is split into them; each other word is a name.

## Step 1: vannas

A Burmese *vanna* (ဝဏ္ဏ) is the unit of the script: a consonant with the marks that belong to it, such as a
medial, a vowel sign, a tone mark, and a final consonant killed with an asat (်). The segmenter reads one
character at a time and starts a new vanna when a character can begin one alone:

- A **consonant** or an independent vowel begins a new vanna, *unless* the one before
  is a virama (it is a stacked consonant), or the consonant itself carries an asat or a
  virama (it is a final consonant, as `က်` is in `သက်`). The segmenter looks a short distance
  ahead for that; a dot below (့) alone can sit between a consonant and its asat.
- A **medial, vowel sign or tone mark** joins the vanna it follows.
- A **digit**, and each mark that can stand alone, such as `၊ ။ ၌ ၍ ၏`, is a token alone.
- A character that is **not Burmese** stands alone too: a space, a letter, a sign. With
  `--coalesce-non-burmese`, a run of them is one token.

The command shows the cut:

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

A span counts octets of UTF-8, and a Burmese letter takes three. The kind
`burmese_vanna` is a vanna, `burmese_digit` a digit, and `non_burmese_char` the rest.
Some words cut like this:

| Text | Vannas |
| --- | --- |
| `အသက်ကို` | `အ` `သက်` `ကို` |
| `ထက်ကြီးသည်` | `ထက်` `ကြီး` `သည်` |
| `တန်ဖိုးကို` | `တန်` `ဖိုး` `ကို` |
| `မြန်မာ` | `မြန်` `မာ` |
| `ကျောင်းသား၊` | `ကျောင်း` `သား` `၊` |
| `ပြန်လည်စွန့်လွှတ်မည်` | `ပြန်` `လည်` `စွန့်` `လွှတ်` `မည်` |
| `သည်ကျော်ပါ` | `သည်` `ကျော်` `ပါ` |

The last row shows a rule that needed care. In `ကျော်` the asat belongs to the
vowel, not to a killed consonant, so `ကျော်` is one vanna and `သည်ကျော်ပါ` can still be
split between `သည်` and `ကျော်`.

## Step 2: words

The lexer joins neighbouring vannas into a *word*. A word ends at:

- **a space.** One space, several, a tab, a line break, and the zero width space
  (U+200B) all count the same. The macOS Burmese input method writes that last one for
  `Shift` + `Space`.
- **a sign that stands alone.** `၊ ။ ၏`, quotes, brackets and the maths signs are each
  a word of their own, so `ပြသပါ။` needs no space and nor does `၁+၂`.
- **a change of script.** `abcက` is two words, and so is `၁၂34`.

So `အသက်ကို` is one word, whatever its meaning to a reader. The lexer cannot know that it
is a name with a particle after it.

## Step 3: reserved words

The lexer looks each word up in the table of reserved words, in this order, and takes the
first match:

| Order | Test | Result |
| --- | --- | --- |
| 1 | the word is a comment marker, or a quote opens | a comment, a string or a character |
| 2 | the word is a sign | punctuation |
| 3 | the word is a reserved word whole | that word |
| 4 | the word is made of reserved words alone | those words, in order |
| 5 | the word starts with a digit | a number |
| 6 | the word has the shape of a name | a name |

Step 4 is the one that matters here. It tries to cut the word at vanna edges into
reserved words, and takes the cut with the fewest pieces. It never cuts inside a vanna,
so `မှာ` cannot lose its `မှ`. It refuses a cut where each piece is a particle, so
`ကိုကို` remains a name. And one vanna that is not reserved defeats the whole cut:

| Written | Lexed as |
| --- | --- |
| `တန်ဖိုးကို` | `တန်ဖိုး` `ကို` |
| `အောက်ပါတို့ကို` | `အောက်ပါ` `တို့ကို` |
| `ထက်ကြီးသည်` | `ထက်` `ကြီးသည်` |
| `အသက်ကို` | one name, since `အသက်` is not reserved |
| `အသက်သည်` | one name, for the same reason |
| `ကိုကို` | one name, since each piece is a particle |

## What it asks of the writer

**A space between a made-up name and the particle after it.** The lexer sees one word
where the writer meant two, and no cut can fix that:

```zaka-error ZK0206
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
အသက်ကို ပြသပါ။
```

```text
error[ZK0206]: "ပြသပါ" needs "ကို|ဟု" after operand 1, found no particle
help: write "အသက် ကို" with a space: a name joined to a particle is read as one word
```

The message names the missing particle and gives the fix.

**No space is needed** between reserved words, or around a sign:

```zaka
၅ သည် ၃ ထက်ကြီးသည် ဟု ပြသပါ။      // ထက်ကြီးသည်: three reserved words, joined
၁+၂ ကို ပြသပါ။                     // no space around the sign
၁ နှင့် ၂ ၏ပေါင်းလဒ် ကို ပြသပါ။      // ၏ stands alone, so no space is needed
```

```text
မှန်
၃
၃
```

The simple rule is that a space separates two words, and a made-up name has a space
on each side. Learn has the same advice in [Spaces between
words](../../learn/sentences/spacing.md).

## No word list

A word list of Burmese would let the compiler cut `အသက်ကို` on its own. It would
also make the meaning of a line depend on a word list, which grows and which speakers do
not share. Zaka asks for less on purpose: the compiler needs the rules of the
script, a fixed table of reserved words, and one space from the writer where a name meets
a particle. The same line is cut in the same manner each time.

## Running it

- `zakac segment file.zaka` writes the vannas of a file; `-o` picks a file and
  `--output-format text|raw` the form.
- `zakac lex file.zaka` writes the tokens that the words became.

See [The lexer](../internals/lexer.md) for the code behind both.
