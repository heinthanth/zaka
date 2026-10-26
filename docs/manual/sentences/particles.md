---
title: Particles
description: The small words that mark what each part of a sentence does.
---

# Particles

A *particle* is a short word placed **after** a part of a sentence to show its role. In
English, word order and little words such as "of", "than" and "by" do this job. In Zaka
the particles do it.

```zaka
// သည် marks the subject, ထက် means "than"
၁၀ သည် ၃ ထက် ကြီးသည် ဟု ပြသပါ။            // 10 is greater than 3

// အောက် means "below", "less than"
၁၀ သည် ၃ အောက် ငယ်သည် ဟု ပြသပါ။           // 10 is less than 3

// နှင့် marks the first operand, ၏ ("of") the last
၁၀ နှင့် ၃ ၏ နုတ်လဒ် ကို ပြသပါ။            // the difference of 10 and 3

// ကို marks the object, ဖြင့် means "by"
၁၀ ကို ၃ ဖြင့် စားသော စားကြွင်း ကို ပြသပါ။  // the remainder of 10 divided by 3
```

```text
မှန်
မမှန်
၇
၁
```

## The common particles

| Particle | Reads as | Example | Meaning |
| --- | --- | --- | --- |
| `သည်` | (subject) | `က သည် ခ ထက် ကြီး` | k is greater than kh |
| `ကို` | (object) | `က ကို ပြသပါ` | show k |
| `နှင့်` | "and", "with" | `က နှင့် ခ ၏ ပေါင်းလဒ်` | the sum of k and kh |
| `၏` | "of" | `က ၏ ...` | closes a list of operands |
| `ဖြင့်` | "by", "using" | `က ကို ခ ဖြင့် စားသော စားလဒ်` | k divided by kh |
| `ထက်` | "than" | `က သည် ခ ထက် ကြီး` | k is greater than kh |
| `အောက်` | "below" | `က သည် ခ အောက် ငယ်` | k is less than kh |
| `ဟု` | "so" (quoting) | `"..." ဟု ပြသပါ` | say "..." |

Later chapters introduce a few more, each with the feature that uses it.

## Particles after a verb

A verb at the end of a sentence may carry a polite ending: `သည်`, `ပါ`, or `ပါသည်`. These
endings do not change the meaning. `ကြီး`, `ကြီးသည်` and `ကြီးပါသည်` all mean "is
greater":

```zaka
၅ သည် ၂ ထက် ကြီး ကို ပြသပါ။
၅ သည် ၂ ထက် ကြီးသည် ဟု ပြသပါ။
၅ သည် ၂ ထက် ကြီးပါသည် ဟု ပြသပါ။
```

```text
မှန်
မှန်
မှန်
```

Notice the print particle. A sentence with an ending (`ကြီးသည်`) is quoted with `ဟု`. A
bare verb (`ကြီး`) is shown with `ကို`. Which ending sounds natural depends on what comes
next. [Natural Zaka](../natural-zaka/README.md) gives the rules.

## Try it

- Print whether 7 is less than 9, using `အောက်` and `ငယ်သည်`.
- Print the remainder of 17 divided by 5 with `ဖြင့်` and `စားကြွင်း`.

Next: [Spaces between words](spacing.md).
