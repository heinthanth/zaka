---
title: Logic
description: Combine true/false answers with and, or, and not.
---

# Logic

A question often has more than one part, such as "is it raining **and** do I have no
umbrella?" Logical operators join two true/false values (booleans) into one answer, or
turn a single answer into its opposite, and like the other operators, each one has a
sign form and a word form.

## You will learn

- how to write and, or and not, in words and in signs
- how to join two comparisons into one question
- when a logical expression needs parentheses
- which part of an "and" or an "or" Zaka skips

| Sign | Words | Reads as | True when |
| --- | --- | --- | --- |
| `a && b` | `a ဖြစ်၍ b` | "a, and b" | both are true |
| `a \|\| b` | `a သို့မဟုတ် b` | "a or b" | at least one is true |
| `!a` | `a မဟုတ်` | "not a" | a is false |

```zaka
ကိန်းရှင် မိုးရွာ သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် မှန် ဖြစ်ပါစေ။ // it is raining
ကိန်းရှင် ထီးပါ သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် မမှန် ဖြစ်ပါစေ။ // I have an umbrella

(မိုးရွာ ဖြစ်၍ ထီးပါ) ကို ပြသမည်။ // and
(မိုးရွာ သို့မဟုတ် ထီးပါ) ကို ပြသမည်။ // or
(ထီးပါ မဟုတ်) ကို ပြသမည်။ // not

(မိုးရွာ && ထီးပါ) ကို ပြသမည်။ // the same with signs
(မိုးရွာ || ထီးပါ) ကို ပြသမည်။
!ထီးပါ ကို ပြသမည်။
```

```text
မမှန်
မှန်
မှန်
မမှန်
မှန်
မှန်
```

We can use these three patterns to write and, or and not in words:

```text
<a> ဖြစ်၍ <b>
<a> သို့မဟုတ် <b>
<a> မဟုတ်
```

| Zaka | Reads as |
| --- | --- |
| `ဖြစ်၍` | "it is so, and"; both sides must be true |
| `သို့မဟုတ်` | "or" |
| `မဟုတ်` | "is not" |

`မဟုတ်` comes **after** the value it turns around, as it does in Burmese, while the sign
`!` comes before it.

## Parentheses when printing

```text
(<logical expression>) ကို ပြသမည်။
```

To print a logical expression, we put it in parentheses before `ကို` or `ဟု`. Without
them, Zaka cannot tell where the expression ends and the print sentence begins, so the
compiler stops with an error:

```zaka-error ZK0204
ကိန်းရှင် မိုးရွာ သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် မှန် ဖြစ်ပါစေ။
မိုးရွာ ဖြစ်၍ မှန် ကို ပြသမည်။
```

```text
error[ZK0204]: "ပြသမည်" ends a statement and does not produce a value
```

## Joining comparisons

The most common use of logic is to join two comparisons, for example to check whether an
age is between 18 and 60. We can write the comparisons in words or in signs:

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃၀ ဖြစ်ပါစေ။
(အသက် သည် ၁၈ အောက် မငယ်သည် ဖြစ်၍ အသက် သည် ၆၀ အောက် ငယ်သည်) ဟု ပြသမည်။
(အသက် >= ၁၈ && အသက် < ၆၀) ကို ပြသမည်။

// not (age is 30)
(အသက် သည် ၃၀ နှင့် ညီသည် မဟုတ်) ကို ပြသမည်။

// a logical expression can also be kept in a variable
ကိန်းရှင် ထွက်သင့် သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (အသက် > ၁၈ ဖြစ်၍ အသက် < ၆၀) ဖြစ်ပါစေ။
ထွက်သင့် ကို ပြသမည်။
```

```text
မှန်
မှန်
မမှန်
မှန်
```

When `မဟုတ်` comes after a comparison, it turns the whole comparison around, so
`အသက် သည် ၃၀ နှင့် ညီသည် မဟုတ်` means "age is not 30".

## Which goes first

"Not" binds tightest, then "and", then "or", the same as × goes before + in maths. So
`က သို့မဟုတ် ခ ဖြစ်၍ ဂ` means `က သို့မဟုတ် (ခ ဖြစ်၍ ဂ)`. When we want a different grouping,
or when it makes the line clearer, we add parentheses.

## Skipping the right side

`ဖြစ်၍` and `သို့မဟုတ်` stop as soon as the answer is known. When the left side of an
"and" is false, the whole answer is false, so Zaka never works out the right side, and
when the left side of an "or" is true, it skips the right side too. This is called
short-circuit evaluation, and it matters once the right side does some work, such as
calling a [function](../functions/README.md).

## Details

- **Operands.** Both sides of "and" and "or", and the value after `!` or before `မဟုတ်`,
  must be true/false values, or the compiler reports ZK0403
  ([Logic](../../manual/language/expressions.md#logic)).
- **`!` and `မဟုတ်` are not the same in a comparison.** `က == ခ မဟုတ်` means `!(က == ခ)`,
  while `!က == ခ` means `(!က) == ခ`, because `!` binds to the one value right after it
  ([Logic](../../manual/language/expressions.md#logic)).
- **Short-circuit with a function.** The [Logic](../../manual/language/expressions.md#logic)
  section in Docs has an example where the right side calls a function, and the call does
  not happen.

## Recap

- `ဖြစ်၍` (`&&`) is "and", `သို့မဟုတ်` (`||`) is "or" and `မဟုတ်` (`!`) is "not".
- `မဟုတ်` comes after the value, while `!` comes before it.
- We put a logical expression in parentheses to print it.
- "And" and "or" skip the right side when the left side settles the answer.

## Hands on

- Print whether both 3 and 5 divide 15 (`%` returns `၀` when the division leaves nothing
  over).
- Print whether the character `'ခ'` is between `'က'` and `'င'`.

Next: [Bitwise operators](bitwise.md).
