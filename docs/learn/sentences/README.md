---
title: Zaka sentences
description: Verb last, particles after each part, and two forms for writing a sum.
---

# Zaka sentences

Zaka follows Burmese word order: first the things the sentence talks about, **then the
verb**. English puts it as "show 42"; Zaka puts it as "42 show": `၄၂ ကို ပြသမည်။`

The verb comes last and decides what the sentence means. A small word called a
*particle* follows each part and tells the role of that part.

```zaka
// "the sum of 3 and 4" — then show it
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၃ နှင့် ၄ ၏ ပေါင်းလဒ် ကို ပြသမည်။

// "the product of 3 and 4"
၃ နှင့် ၄ ၏ မြှောက်လဒ် ကို ပြသမည်။

// "10 over 2: the quotient"
၁၀ ကို ၂ ဖြင့် စားသော စားလဒ် ကို ပြသမည်။

// "3 is greater than 2"
၃ သည် ၂ ထက် ကြီးသည် ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၇
၁၂
၅
မှန်
```

Read the first line one word at a time:

| Zaka | Reads as | Role |
| --- | --- | --- |
| `၃` | 3 | a number |
| `နှင့်` | "and", "with" | marks the first of two things |
| `၄` | 4 | a number |
| `၏` | "of" | marks the last of the things |
| `ပေါင်းလဒ်` | "sum" | the verb: adds the two numbers |
| `ကို` | (object) | marks what the next verb acts on |
| `ပြသမည်` | "please show" | the verb: print |
| `။` | full stop | ends the statement |

## Two kinds of verbs

- A verb that **makes a value** forms an *expression*. `ပေါင်းလဒ်` (sum) and `ကြီး`
  (is greater) are such verbs. An expression is something with a value, like `၃ + ၄`.
- A verb that **gives a command** forms a *statement*. `ပြသမည်` (show) is one. A statement
  does something and ends with `။`.

An expression can sit inside a statement. In `၃ နှင့် ၄ ၏ ပေါင်းလဒ် ကို ပြသမည်။`, the sum is
worked out first. Then its value, `၇`, takes the particle `ကို` and is shown.

## Words or signs

Most operations can also be written with the signs from school maths. Both forms give
the same result, and the two can be mixed:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၃ + ၄ ကို ပြသမည်။                        // signs
၃ နှင့် ၄ ၏ ပေါင်းလဒ် ကို ပြသမည်။           // words
၁ + ၂ နှင့် ၄ ၏ ပေါင်းလဒ် ကို ပြသမည်။       // both: the sum of (1 + 2) and 4
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၇
၇
၇
```

The [Operators](../operators/README.md) chapter lists each word and sign.

## Hands on

- Print the sum of 100 and 23 with words, then with `+`.
- Print the product of 6 and 7 with `မြှောက်လဒ်`.

Next: [Particles](particles.md).
