---
title: Operators
description: Work with values through Burmese words or signs.
---

# Operators

An operator works on one or two values and returns a new value, so `+` adds two numbers
and `<` compares them. Zaka has two forms for most operators: a sign form, the same as in
school maths, and a word form, which is a short Burmese phrase. Both forms mean the same
thing, and we can use whichever one reads better.

## You will learn

- how to write an operator as a sign and as Burmese words
- that we can mix the two forms in one program
- which groups of operators Zaka has, and where this chapter covers each group

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၇ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။

က + ခ ကို ပြသမည်။                       // sign
က နှင့် ခ ၏ ပေါင်းလဒ် ကို ပြသမည်။          // words: the same sum

က > ခ ကို ပြသမည်။                       // sign
က သည် ခ ထက် ကြီးသည် ဟု ပြသမည်။            // words: k is greater than kh
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၂
၂၂
မှန်
မှန်
```

The two `ပြသမည်` lines of each pair print the same value. To add two numbers, we can
write either form:

```text
<a> + <b>
<a> နှင့် <b> ၏ ပေါင်းလဒ်
```

| Zaka | Reads as |
| --- | --- |
| `<a> နှင့်` | "`<a>` and" |
| `<b> ၏` | "`<b>`'s"; `၏` joins the two numbers to the word after it |
| `ပေါင်းလဒ်` | "sum" |

So the word form means "the sum of `<a>` and `<b>`". A word form ends with a word for its
result, such as `ပေါင်းလဒ်` (sum), and Zaka uses that word to know which operation we want.

## Signs or words

Signs are short and look like maths, while words follow the order of a Burmese sentence.
We can pick the form that makes each line clearer, and we can mix the two forms in one
program, or even in one expression, as [Grouping and order](grouping.md) shows.

A comparison in words, such as `က သည် ခ ထက် ကြီးသည်` ("k is greater than kh"), is a whole
Burmese sentence, so we print it with `ဟု`, the same as quoted text. The sign form `က > ခ`
is a plain value, so we print it with `ကို`.

## In this chapter

| Page | Operators |
| --- | --- |
| [Arithmetic](arithmetic.md) | `+ - * / %` and negation |
| [Comparison](comparison.md) | `== != < > <= >=` |
| [Logic](logic.md) | and, or, not |
| [Bitwise operators](bitwise.md) | `& \| ^ ~ << >>` |
| [Converting between data kinds](conversion.md) | turn a value of one data kind (type) into another |
| [Grouping and order](grouping.md) | parentheses, and which operator goes first |

## Recap

- Most operators have a sign form (`က + ခ`) and a word form (`က နှင့် ခ ၏ ပေါင်းလဒ်`),
  and the two forms mean the same thing.
- We can mix signs and words in one program.
- We print a comparison in words with `ဟု`, because it is a whole sentence.

Next: [Arithmetic](arithmetic.md).
