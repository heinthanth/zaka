---
title: Zaka sentences
description: Verb last, particles after each part, and two ways to write a sum.
---

# Zaka sentences

Zaka follows Burmese word order: first the things you talk about, **then the verb**.
In English you say "show 42"; in Zaka you say "42 show": `၄၂ ကို ပြသပါ။`

The verb comes last and decides what the sentence means. A small word called a
*particle* follows each part and tells which role that part plays.

```zaka
// "the sum of 3 and 4" — then show it
၃ နှင့် ၄ ၏ ပေါင်းလဒ် ကို ပြသပါ။

// "the product of 3 and 4"
၃ နှင့် ၄ ၏ မြှောက်လဒ် ကို ပြသပါ။

// "10 divided by 2: the quotient"
၁၀ ကို ၂ ဖြင့် စားသော စားလဒ် ကို ပြသပါ။

// "3 is greater than 2"
၃ သည် ၂ ထက် ကြီးသည် ဟု ပြသပါ။
```

```text
၇
၁၂
၅
မှန်
```

Read the first line word by word:

| Zaka | Reads as | Role |
| --- | --- | --- |
| `၃` | 3 | a number |
| `နှင့်` | "and", "with" | marks the first of two things |
| `၄` | 4 | a number |
| `၏` | "of" | marks the last of the things |
| `ပေါင်းလဒ်` | "sum" | the verb: adds the two numbers |
| `ကို` | (object) | marks what the next verb acts on |
| `ပြသပါ` | "please show" | the verb: print |
| `။` | full stop | ends the statement |

## Two kinds of verbs

- A verb that **makes a value** forms an *expression*. `ပေါင်းလဒ်` (sum) and `ကြီး`
  (is greater) are such verbs. An expression is something with a value, like `၃ + ၄`.
- A verb that **gives a command** forms a *statement*. `ပြသပါ` (show) is one. A statement
  does something and ends with `။`.

An expression can sit inside a statement. In `၃ နှင့် ၄ ၏ ပေါင်းလဒ် ကို ပြသပါ။`, the sum is
worked out first. Then its value, `၇`, takes the particle `ကို` and is shown.

## Words or symbols

Most operations can also be written with the symbols you know from school maths. Both
forms give the same result, and you can mix them:

```zaka
၃ + ၄ ကို ပြသပါ။                        // symbols
၃ နှင့် ၄ ၏ ပေါင်းလဒ် ကို ပြသပါ။           // words
၁ + ၂ နှင့် ၄ ၏ ပေါင်းလဒ် ကို ပြသပါ။       // both: the sum of (1 + 2) and 4
```

```text
၇
၇
၇
```

The [Operators](../operators/README.md) chapter lists every word and symbol.

## Try it

- Print the sum of 100 and 23 with words, then with `+`.
- Print the product of 6 and 7 with `မြှောက်လဒ်`.

Next: [Particles](particles.md).
