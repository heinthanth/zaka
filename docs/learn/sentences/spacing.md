---
title: Spaces between words
description: Where Zaka needs a space, and where it does not.
---

# Spaces between words

Written Burmese puts no spaces between words, and people who read it know where each word
ends, but the compiler has no word list, so it cannot tell. In Zaka, a space (white space)
separates two words, and the one space that we must not forget is the space between a
name that we make up and the particle after it.

## You will learn

- Where Zaka needs a space: between a name and the particle after it.
- What the compiler reports when that space is missing.
- Where a space is optional: between Zaka's own words, and around signs.
- That one space, several spaces, a tab and a line break all count the same.

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။ // make a number named အသက် (age)
အသက် ကို ပြသမည်။                          // a space between အသက် and ကို
```

```text
၂၀
```

The first line makes a number with the name `အသက်` (age), which the
[Variables](../variables/README.md) chapter explains, and the second line prints it. When we
use a name that we made up, we put a space after it, before its particle:

```
<name> ကို
```

| Zaka | The compiler sees |
| --- | --- |
| `အသက် ကို` | two words: the name `အသက်`, then the particle `ကို` |
| `အသက်ကို` | one word: a name `အသက်ကို`, which does not exist |

## A missing space

Without the space, the compiler sees `အသက်ကို` as one long name. No name of that spelling
exists, so the compiler reports an error, and it suggests the fix:

```zaka-error ZK0203
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
အသက်ကို ပြသမည်။
```

```text
error[ZK0203]: this statement does not end in a statement verb, found "ပြသ"
help: write "အသက် ကို" with a space: a name joined to a particle is read as one word
```

## Where spaces are optional

We can leave the spaces out in three places:

- **Between Zaka's own words.** The compiler can split a run of words that are all
  reserved words, so `ထက်ကြီးသည်` is `ထက်` + `ကြီး` + `သည်`.
- **Around signs.** `။`, `၊`, `၏`, `(`, `)`, quotes, and maths signs such as `+` and `<`
  are words on their own in all cases, so `၁+၂`, `ပြသမည်။` and `၏ပေါင်းလဒ်` need no space.
- **How much space.** One space, several spaces, a tab or a line break all count the
  same.

```zaka
၅ သည် ၃ ထက်ကြီးသည် ဟု ပြသမည်။      // reserved words joined
၁+၂ ကို ပြသမည်။                     // no spaces around +
၁ နှင့် ၂ ၏ ပေါင်းလဒ် ကို ပြသမည်။      // ၏ needs no space
```

```text
မှန်
၃
၃
```

## Details

- **The zero width space.** With the macOS Burmese input source, `Shift` + `Space` enters
  an invisible "zero width space", and Zaka counts it as a space too.
- **How a run of reserved words is split.** The compiler splits a run into the fewest
  pieces, and it leaves a run of particles alone, such as `ကိုကို`. The words of a
  reserved phrase, such as `စားသော စားလဒ်`, work with a space between them or with none.
- **Digits next to letters.** A digit joined to a Burmese letter forms one word too, so
  `၃နှင့်၄` is the error ZK0102 (an invalid number). See [Words and
  spaces](../../manual/language/words.md#words-and-spaces) for the full rules.

## Recap

- A space separates two words in Zaka.
- We put a space between a name that we made up and the particle after it, as in
  `အသက် ကို`.
- Zaka's own words, and signs such as `+` and `။`, need no space around them.
- A tab, a line break or several spaces count the same as one space.

## Hands on

- Remove the space between `အသက်` and `ကို` in the first example and read the message.
- Write `၃ နှင့် ၄ ၏ ပေါင်းလဒ်` with as few spaces as possible while it still runs.

Next: [Scalar values](../values/README.md).
