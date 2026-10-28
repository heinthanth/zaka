---
title: Grouping and order
description: Which operator goes first, how word phrases nest, and when to use parentheses.
---

# Grouping and order

In `၂ + ၃ * ၄`, which comes first, the addition or the multiplication? As in school
maths, Zaka does the multiplication first, so the answer is 14. Parentheses change the
order: `(၂ + ၃) * ၄` is 20.

```zaka
၂ + ၃ * ၄ ကို ပြသပါ။         // * before +
(၂ + ၃) * ၄ ကို ပြသပါ။       // parentheses first
```

```text
၁၄
၂၀
```

## Order of operators

From first to last (tightest to loosest):

1. `-`, `အနုတ်`, `~`, `!` in front of a value
2. `*`, `/`, `%`
3. `+`, `-`
4. `<<`, `>>`
5. `&`
6. `^`
7. `|`
8. `<`, `>`, `<=`, `>=`
9. `==`, `!=`
10. word phrases (`... ၏ ပေါင်းလဒ်`, `... ထက် ကြီး`, and so on)
11. `မဟုတ်` (not)
12. `ဖြစ်၍`, `&&` (and)
13. `သို့မဟုတ်`, `||` (or)

Operators on the same line go left to right: `၁၀ - ၃ - ၂` is `(၁၀ - ၃) - ၂`. The
[Operator precedence](../reference/precedence.md) page has the full table.

## Word phrases take whole sums

A part of a word phrase can be a whole sum written with signs. In
`က + ခ နှင့် ဂ ၏ မြှောက်လဒ်`, the first part is `က + ခ`. It reads "the product of (k + kh) and g":

```zaka
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၄ ဖြစ်ပါစေ။
က + ခ နှင့် ဂ ၏ မြှောက်လဒ် ကို ပြသပါ။         // (2 + 3) × 4
```

```text
၂၀
```

## Word phrases inside word phrases

Word phrases can nest. Read from left to right. Each verb takes the parts just before it
that hold the particles it needs, and its result joins the sentence:

```zaka
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၄ ဖြစ်ပါစေ။

// k and (the product of kh and g): the sum      → 2 + (3 × 4)
က နှင့် ခ နှင့် ဂ ၏ မြှောက်လဒ် ၏ ပေါင်းလဒ် ကို ပြသပါ။

// (the sum of k and kh) and g: the product      → (2 + 3) × 4
က နှင့် ခ ၏ ပေါင်းလဒ် နှင့် ဂ ၏ မြှောက်လဒ် ကို ပြသပါ။
```

```text
၁၄
၂၀
```

In the first line, `မြှောက်လဒ်` takes `ခ နှင့်` and `ဂ ၏`, the two parts just before it.
A `၏` follows its result, so `ပေါင်းလဒ်` then takes `က နှင့်` and that result.

## Parentheses move a phrase into a sum

A word phrase cannot sit straight inside a sum written with signs. To use one there,
put it in parentheses:

```zaka-error ZK0203
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
က နှင့် ခ ၏ ပေါင်းလဒ် * ၂ ကို ပြသပါ။
```

```text
error[ZK0203]: this statement does not end in a statement verb, found "*"
```

```zaka
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
(က နှင့် ခ ၏ ပေါင်းလဒ်) * ၂ ကို ပြသပါ။
```

```text
၁၀
```

Parentheses also wrap a [logical expression](logic.md) used as a value, as in
`(က ဖြစ်၍ ခ) ကို ပြသပါ။`

## Needless parentheses

Parentheses around a whole printed or stored value change nothing. Zaka warns about
them, much as an editor flags a double space:

```zaka-warning ZK0507
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
(က + ၁) ကို ပြသပါ။
```

```text
warning[ZK0507]: these parentheses are not needed
```

## Hands on

- Write `(၂ + ၃) * ၄` using words alone.
- Print `၁၀ - ၃ - ၂` and `၁၀ - (၃ - ၂)`. Do the two match?

Next: [Control flow](../control-flow/README.md).
