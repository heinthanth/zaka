---
title: Grouping and order
description: Which operator goes first, how word phrases nest, and when to use parentheses.
---

# Grouping and order

In `၂ + ၃ * ၄`, do you add first or multiply first? As in school maths, Zaka multiplies
first, so the answer is 14. Parentheses change the order: `(၂ + ၃) * ၄` is 20.

```zaka
၂ + ၃ * ၄ ကို ပြသပါ။         // * before +
(၂ + ၃) * ၄ ကို ပြသပါ။       // parentheses first
```

```text
၁၄
၂၀
```

## Order of symbols

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

## Word phrases take whole symbol sums

A part of a word phrase can be a whole symbol sum. In `က + ခ နှင့် ဂ ၏ မြှောက်လဒ်`, the
first part is `က + ခ`. It reads "the product of (k + kh) and g":

```zaka
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၄ ဖြစ်ပါစေ။
က + ခ နှင့် ဂ ၏ မြှောက်လဒ် ကို ပြသပါ။         // (2 + 3) × 4
```

```text
၂၀
```

## Word phrases inside word phrases

Word phrases can nest. Read from left to right. Each verb takes the parts just before it
that carry the particles it needs, and its result joins the sentence:

```zaka
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၄ ဖြစ်ပါစေ။

// k and (the product of kh and g): the sum      → 2 + (3 × 4)
က နှင့် ခ နှင့် ဂ ၏ မြှောက်လဒ် ၏ ပေါင်းလဒ် ကို ပြသပါ။

// (the sum of k and kh) and g: the product      → (2 + 3) × 4
က နှင့် ခ ၏ ပေါင်းလဒ် နှင့် ဂ ၏ မြှောက်လဒ် ကို ပြသပါ။
```

```text
၁၄
၂၀
```

In the first line, `မြှောက်လဒ်` takes `ခ နှင့်` and `ဂ ၏`, the two parts just before it. Its
result is followed by `၏`, so `ပေါင်းလဒ်` then takes `က နှင့်` and that result.

## Parentheses move a phrase into a sum

A word phrase cannot sit directly inside a symbol sum. To use one there, put it in
parentheses:

```zaka-error ZK0203
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
က နှင့် ခ ၏ ပေါင်းလဒ် * ၂ ကို ပြသပါ။
```

```text
error[ZK0203]: this statement does not end in a statement verb, found "*"
```

```zaka
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
(က နှင့် ခ ၏ ပေါင်းလဒ်) * ၂ ကို ပြသပါ။
```

```text
၁၀
```

Parentheses also wrap a [logical expression](logic.md) used as a value, as in
`(က ဖြစ်၍ ခ) ကို ပြသပါ။`

## Parentheses you do not need

Parentheses around a whole printed or stored value change nothing. Zaka warns you about
them, the way an editor flags a double space:

```zaka-warning ZK0507
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
(က + ၁) ကို ပြသပါ။
```

```text
warning[ZK0507]: these parentheses are not needed
```

## Try it

- Write `(၂ + ၃) * ၄` using only words.
- Print `၁၀ - ၃ - ၂` and `၁၀ - (၃ - ၂)`. Are they the same?

Next: [Control flow](../control-flow/README.md).
