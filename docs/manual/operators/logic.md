---
title: Logic
description: Combine true/false answers with and, or, and not.
---

# Logic

Real questions often have parts: "Is it raining **and** have I no umbrella?" The
*logical operators* join or flip booleans.

```zaka
ကိန်းရှင် မိုးရွာ သည် မှန်မှားတန်ဖိုး မှန် ဖြစ်ပါစေ။       // it is raining
ကိန်းရှင် ထီးပါ သည် မှန်မှားတန်ဖိုး မမှန် ဖြစ်ပါစေ။       // I have an umbrella

(မိုးရွာ ဖြစ်၍ ထီးပါ) ကို ပြသပါ။                     // and
(မိုးရွာ သို့မဟုတ် ထီးပါ) ကို ပြသပါ။                  // or
(ထီးပါ မဟုတ်) ကို ပြသပါ။                            // not

(မိုးရွာ && ထီးပါ) ကို ပြသပါ။                        // the same with symbols
(မိုးရွာ || ထီးပါ) ကို ပြသပါ။
!ထီးပါ ကို ပြသပါ။
```

```text
မမှန်
မှန်
မှန်
မမှန်
မှန်
မှန်
```

| Symbol | Words | Reads as | True when |
| --- | --- | --- | --- |
| `a && b` | `a ဖြစ်၍ b` | "a, and b" | both are true |
| `a \|\| b` | `a သို့မဟုတ် b` | "a or b" | at least one is true |
| `!a` | `a မဟုတ်` | "not a" | a is false |

Notice that `မဟုတ်` ("not") comes **after** what it negates, as it does in Burmese.

## Parentheses when printing

A logical expression must be wrapped in parentheses before `ကို` or `ဟု`. Without them,
Zaka cannot tell where the expression ends and the print statement begins:

```zaka-error ZK0204
ကိန်းရှင် မိုးရွာ သည် မှန်မှားတန်ဖိုး မှန် ဖြစ်ပါစေ။
မိုးရွာ ဖြစ်၍ မှန် ကို ပြသပါ။
```

```text
error[ZK0204]: statement verb "ပြသပါ" cannot stand inside an expression
```

## Joining comparisons

The most common use is to join comparisons. Is the age between 18 and 60?

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ၃၀ ဖြစ်ပါစေ။
(အသက် သည် ၁၈ အောက် မငယ်သည် ဖြစ်၍ အသက် သည် ၆၀ အောက် ငယ်သည်) ဟု ပြသပါ။
(အသက် >= ၁၈ && အသက် < ၆၀) ကို ပြသပါ။

// not (age is 30)
(အသက် သည် ၃၀ နှင့် ညီသည် မဟုတ်) ကို ပြသပါ။

// a logical expression can also be stored
ကိန်းရှင် ထွက်သင့် သည် မှန်မှားတန်ဖိုး (အသက် > ၁၈ ဖြစ်၍ အသက် < ၆၀) ဖြစ်ပါစေ။
ထွက်သင့် ကို ပြသပါ။
```

```text
မှန်
မှန်
မမှန်
မှန်
```

## Which goes first

"Not" binds tightest, then "and", then "or", just as × goes before + in maths.
`က သို့မဟုတ် ခ ဖြစ်၍ ဂ` means `က သို့မဟုတ် (ခ ဖြစ်၍ ဂ)`. Add parentheses when you mean
something else, or when it helps the reader.

## Stopping early

`ဖြစ်၍` and `သို့မဟုတ်` stop as soon as they know the answer. If the left side of
"and" is false, the whole thing is false, so the right side is never worked out. If the
left side of "or" is true, the right side is skipped. This is called *short-circuit*
evaluation. It matters once the right side does work, such as calling a
[function](../functions/README.md).

## Try it

- Print whether 15 is divisible by both 3 and 5 (`%` gives `၀` when it divides).
- Print whether a character `'ခ'` is between `'က'` and `'င'`.

Next: [Bitwise operators](bitwise.md).
