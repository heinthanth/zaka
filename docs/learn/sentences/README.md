---
title: Zaka sentences
description: Verb last, particles after each part, and two forms for writing a sum.
---

# Zaka sentences

A Zaka sentence (statement) uses Burmese word order, with the verb at the end. For
example, "show 42" becomes `၄၂ ကို ပြသမည်။`. The word `ကို` follows `၄၂` to mark it as
the value to print. We call a word that marks a role like this a particle
(postposition).

## You will learn

- The order of a Zaka sentence: the parts first, the verb last.
- How particles show the role of each part.
- The difference between an expression, which has a value, and a statement, which does
  something.
- How to write a sum with words or with signs such as `+`.

```zaka
၃ နှင့် ၄ ၏ ပေါင်းလဒ် ကို ပြသမည်။
```

```text
၇
```

Word for word, the sentence means "3 and 4, their sum, show it". To add two numbers in
words, we write the two numbers with `နှင့်` and `၏` and then the word for the sum:

```
<number> နှင့် <number> ၏ ပေါင်းလဒ်
```

| Zaka | Reads as | Role |
| --- | --- | --- |
| `၃` | 3 | a number |
| `နှင့်` | "and", "with" | comes after the first of the two numbers |
| `၄` | 4 | a number |
| `၏` | "of" | comes after the last number |
| `ပေါင်းလဒ်` | "the sum" | adds the two numbers |
| `ကို` | (the object) | comes after the value that `ပြသမည်` prints |
| `ပြသမည်` | "please show" | the verb: print |
| `။` | full stop | ends the sentence |

## The verb comes last

Each of these sentences puts the numbers first and the verb last, and the particles
between them show what each number does:

```zaka
// "the product of 3 and 4"
၃ နှင့် ၄ ၏ မြှောက်လဒ် ကို ပြသမည်။

// "10 over 2: the quotient"
၁၀ ကို ၂ ဖြင့် စားသော စားလဒ် ကို ပြသမည်။

// "3 is greater than 2"
၃ သည် ၂ ထက် ကြီးသည် ဟု ပြသမည်။
```

```text
၁၂
၅
မှန်
```

The [Particles](particles.md) page goes through each of these particles.

## Values and commands

A sentence can have two sorts of verb, and the two sorts do different things:

- A verb that **makes a value** forms an *expression*. `ပေါင်းလဒ်` (the sum) and `ကြီး`
  (is greater) are verbs of this sort. An expression is something with a value, such as
  `၃ + ၄`.
- A verb that **gives a command** forms a *statement*. `ပြသမည်` (show) is one. A
  statement does something, and we end it with `။`.

We can put an expression inside a statement. In `၃ နှင့် ၄ ၏ ပေါင်းလဒ် ကို ပြသမည်။`, the
computer works out the sum first, and then it prints the value `၇`, which comes before
`ကို`.

## Words or signs

We can also write most operations with the signs from school maths. Both forms give the
same result, and we can mix the two in one sentence:

```
<number> + <number>
```

```zaka
၃ + ၄ ကို ပြသမည်။ // signs
၃ နှင့် ၄ ၏ ပေါင်းလဒ် ကို ပြသမည်။ // words
၁ + ၂ နှင့် ၄ ၏ ပေါင်းလဒ် ကို ပြသမည်။ // both: the sum of (1 + 2) and 4
```

```text
၇
၇
၇
```

The [Operators](../operators/README.md) chapter lists each word and sign.

## Details

- **A sentence can cover several lines.** A sentence ends at `။` and not at the end of a
  line, so one sentence can run over several lines, and several sentences can share a
  line. See [Sentences and the full
  stop](../../manual/language/words.md#sentences-and-the-full-stop).
- **Commas.** Between the items of a list we use `၊`, the Burmese comma, and a `,` is an
  error (ZK0101).
- **The verbs that end a sentence.** A sentence that does not end in a verb that Zaka
  knows, such as `ပြသမည်` or `ထားမည်`, is the error ZK0203. [The verb ends the
  sentence](../../manual/language/statements.md#the-verb-ends-the-sentence) lists these
  verbs.

## Recap

- A Zaka sentence puts its parts first and its verb last, and ends with `။`.
- A particle comes after each part and shows its role.
- An expression has a value, and a statement does something; an expression can go
  inside a statement.
- We can write most operations in words, such as `ပေါင်းလဒ်`, or with signs, such as `+`.

## Hands on

- Print the sum of 100 and 23 with words, then with `+`.
- Print the product of 6 and 7 with `မြှောက်လဒ်`.
- Split `၃ နှင့် ၄ ၏ ပေါင်းလဒ် ကို ပြသမည်။` over three lines. Does it still run?

Next: [Particles](particles.md).
