---
title: Particles
description: The small words that show the role of each part of a sentence.
---

# Particles

A particle (postposition) is a short word that we put **after** a part of a sentence to
show its role. English does this with word order and with small words such as "of",
"than" and "with", while Zaka uses particles.

## You will learn

- The common particles: `သည်`, `ကို`, `နှင့်`, `၏`, `ဖြင့်`, `ထက်`, `အောက်` and `ဟု`.
- How to compare two numbers with `ထက်` and `အောက်`.
- The polite endings `သည်`, `ပါ` and `ပါသည်` after a verb.
- How to turn on the spoken endings `တယ်` and `မယ်`.

```zaka
၁၀ သည် ၃ ထက် ကြီးသည် ဟု ပြသမည်။
```

```text
မှန်
```

The sentence means "10 is greater than 3", which is true, so it prints `မှန်`. To
compare two numbers, we use `သည်` after the first number and `ထက်` ("than") after the
second:

```
<number> သည် <number> ထက် ကြီးသည်
```

| Zaka | Reads as | Role |
| --- | --- | --- |
| `၁၀` | 10 | the first number |
| `သည်` | (the subject) | comes after the subject of the sentence |
| `၃` | 3 | the second number |
| `ထက်` | "than" | comes after the number we compare against |
| `ကြီးသည်` | "is greater" | the verb |
| `ဟု` | "so" (quoting) | quotes the whole sentence for `ပြသမည်` |

## The common particles

This program uses most of the common particles, and the comment above each line shows
what the line means:

```zaka
// "10 is greater than 3"
၁၀ သည် ၃ ထက် ကြီးသည် ဟု ပြသမည်။

// "10 is less than 3": အောက် means "below"
၁၀ သည် ၃ အောက် ငယ်သည် ဟု ပြသမည်။

// "the difference of 10 and 3"
၁၀ နှင့် ၃ ၏ နုတ်လဒ် ကို ပြသမည်။

// "the remainder of 10 ÷ 3"
၁၀ ကို ၃ ဖြင့် စားသော စားကြွင်း ကို ပြသမည်။
```

```text
မှန်
မမှန်
၇
၁
```

In the table, `က` and `ခ` are two numbers:

| Particle | Reads as | Example | Meaning |
| --- | --- | --- | --- |
| `သည်` | (the subject) | `က သည် ခ ထက် ကြီး` | k is greater than kh |
| `ကို` | (the object) | `က ကို ပြသမည်` | show k |
| `နှင့်` | "and", "with" | `က နှင့် ခ ၏ ပေါင်းလဒ်` | the sum of k and kh |
| `၏` | "of" | `က နှင့် ခ ၏ ...` | comes after the last of the numbers |
| `ဖြင့်` | "with", "using" | `က ကို ခ ဖြင့် စားသော စားလဒ်` | k ÷ kh |
| `ထက်` | "than" | `က သည် ခ ထက် ကြီး` | k is greater than kh |
| `အောက်` | "below" | `က သည် ခ အောက် ငယ်` | k is less than kh |
| `ဟု` | "so" (quoting) | `"..." ဟု ပြသမည်` | show "..." |

Later chapters bring in a few more particles, each with the feature that uses it.

## Endings after a verb

```
<verb>
<verb>သည်
<verb>ပါသည်
```

We can add a polite ending to a verb at the end of a sentence: `သည်`, `ပါ` or `ပါသည်`.
The ending does not change the meaning, so `ကြီး`, `ကြီးသည်` and `ကြီးပါသည်` all mean "is
greater":

```zaka
၅ သည် ၂ ထက် ကြီး ကို ပြသမည်။
၅ သည် ၂ ထက် ကြီးသည် ဟု ပြသမည်။
၅ သည် ၂ ထက် ကြီးပါသည် ဟု ပြသမည်။
```

```text
မှန်
မှန်
မှန်
```

Look at the particle before `ပြသမည်` in each line. With an ending, as in `ကြီးသည်`, the
words form a whole sentence, so we quote it with `ဟု`. With the bare verb `ကြီး`, we use
`ကို`. The ending that sounds natural depends on what comes next, and [Natural
Zaka](../natural-zaka/README.md) gives the rules.

## Spoken endings

```
<verb>တယ်
<verb>မယ်
```

When people speak Burmese, people end a sentence with `တယ်` where writing has `သည်`, and
with `မယ်` where writing has `မည်`. Zaka accepts the spoken words too, once we allow them
with `zakac run --lint-allow spoken`. We run this program with that flag:

```zaka lint-allow=spoken
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မယ်။
၅ သည် ၂ ထက် ကြီးတယ် လို့ ပြသမယ်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
```

Without `--lint-allow spoken`, each spoken word is an error, and the message shows the
written spelling to use in its place.

## Details

- **An ending joined to a verb.** We write a polite ending joined to its verb, as in
  `ကြီးသည်`. With a space before it, `သည်` is the subject particle instead. See [Words and
  spaces](../../manual/language/words.md#words-and-spaces).
- **Each spoken word.** [Spoken Burmese](../../manual/language/spoken.md) lists each
  spoken word, `လို့` for `ဟု` among them, and the levels of the `spoken` lint.

## Recap

- A particle comes after a part of a sentence and shows its role.
- We use `သည်` after the subject, `ကို` after the object, and `ထက်` or `အောက်` to compare.
- We use `နှင့်` and `၏` around the numbers of a sum, and `ဖြင့်` for "with".
- A polite ending such as `သည်` does not change the meaning, and a sentence with one is
  quoted with `ဟု`.

## Hands on

- Print whether 7 is less than 9, using `အောက်` and `ငယ်သည်`.
- Print what is left over when 5 goes into 17, with `ဖြင့်` and `စားသော စားကြွင်း`.
- Run the spoken example without `--lint-allow spoken`, and read the messages.

Next: [Spaces between words](spacing.md).
