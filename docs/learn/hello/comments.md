---
title: Comments
description: Notes for people in a program, which the computer skips.
---

# Comments

A *comment* is a note that we write for the people who read a program, and the computer
skips it when the program runs. Zaka has three markers that start a comment, and each
comment runs to the end of its line.

## You will learn

- How to start a comment with `//`, `///` or `မှတ်ချက်`.
- How to put a comment after a sentence (statement), on the same line.
- How to switch a line off with a comment, without deleting it.
- The one rule of `မှတ်ချက်`: it needs a space after it.

```zaka
// The program starts here.
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ" ဟု ပြသမည်။ // a comment after a sentence
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ
```

The program prints the greeting and nothing else, because the computer skips both
comments. To write a comment, we start it with `//` and write the note after it:

```
// <note>
```

| Zaka | What it does |
| --- | --- |
| `//` | starts a comment that runs to the end of the line |
| `<note>` | the note itself, in a language of our choice |

## Three markers

We can start a comment in three forms, and all three do the same thing:

```zaka
// A comment can start with two slashes.
/// Or with three slashes, often used to describe what comes next.
မှတ်ချက် Or with the Burmese word for "note".

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၃ ကို ပြသမည်။ // after a sentence
၄ ကို ပြသမည်။ မှတ်ချက် also after a sentence
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၃
၄
```

| Zaka | Reads as | Use it for |
| --- | --- | --- |
| `//` | | a comment to the end of the line |
| `///` | | the same; as a habit, a note about the code below it |
| `မှတ်ချက်` | "note" | the same, in Burmese |

Zaka has no comment that spans several lines, so we start each line of a longer note
with its own marker.

## Switching a line off

When we want to skip a sentence for a while, we can put `//` in front of it. The
sentence remains in the file, and we can bring it back later when we remove the `//`:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"တစ်" ဟု ပြသမည်။
// "နှစ်" ဟု ပြသမည်။
"သုံး" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
တစ်
သုံး
```

## `မှတ်ချက်` needs a space

`မှတ်ချက်` starts a comment when it is a word on its own, with a space after it. When
we join it to the next word, the compiler sees one long word of program text, and it
stops with an error:

```zaka-error ZK0202
မှတ်ချက်ကို မမေ့ပါနှင့်
```

```text
error[ZK0202]: expected an operand, found "မှတ်ချက်"
```

## Details

- **Markers inside quotes.** A marker inside text in quotes is part of the text, so
  `"http://zaka.example // not a comment" ဟု ပြသမည်။` prints the whole line. See
  [Comments](../../manual/language/words.md#comments).
- **Comments are not words of the program.** The compiler drops each comment before it
  reads the sentences, so a comment can sit between two words of a long sentence that
  covers several lines.

## Recap

- A comment is a note for people, and the computer skips it.
- We start a comment with `//`, `///` or `မှတ်ချက်`, and it runs to the end of the line.
- We can put a comment on its own line or after a sentence.
- We need a space after `မှတ်ချက်`, or the compiler reads it as program text.

## Hands on

- Add a comment above each line of the [Hello, world](README.md) program that explains
  what the line does.
- Switch off the middle line of the last example with `မှတ်ချက်` in place of `//`. What
  prints now?

Next: [Printing](printing.md).
