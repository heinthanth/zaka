---
title: Grouping and order
description: Which operator goes first, how word phrases nest, and when to use parentheses.
---

# Grouping and order

When a line has more than one operator, Zaka needs to know which one to work out first,
which is called the order of operations (operator precedence). As in school maths, Zaka
does multiplication before addition, and parentheses change the order.

## You will learn

- which operators Zaka works out first
- how to change the order with parentheses
- how word phrases nest inside each other
- when a word phrase needs parentheses, and when parentheses are not needed

| Signs | Words | Result |
| --- | --- | --- |
| `၂ + ၃ * ၄` | `၂ နှင့် ၃ နှင့် ၄ ၏ မြှောက်လဒ် ၏ ပေါင်းလဒ်` | 14 |
| `(၂ + ၃) * ၄` | `၂ နှင့် ၃ ၏ ပေါင်းလဒ် နှင့် ၄ ၏ မြှောက်လဒ်` | 20 |

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၂ + ၃ * ၄ ကို ပြသမည်။         // * before +
(၂ + ၃) * ၄ ကို ပြသမည်။       // parentheses first
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၄
၂၀
```

In `၂ + ၃ * ၄`, Zaka multiplies first, so the answer is 14. To add first, we put the
addition in parentheses:

```text
(<expression>)
```

Zaka works out what is inside the parentheses before the rest, so `(၂ + ၃) * ၄` is 20.

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

Operators on the same line go from left to right, so `၁၀ - ၃ - ၂` is `(၁၀ - ၃) - ၂`. The
[Operator precedence](../../manual/reference/precedence.md) page has the full table.

## A whole sum inside a word phrase

```text
<a> + <b> နှင့် <c> ၏ မြှောက်လဒ်
```

Each part of a word phrase can be a whole sum with signs, because signs bind tighter than
words. In `က + ခ နှင့် ဂ ၏ မြှောက်လဒ်`, the first part is `က + ခ`, so the phrase means
"the product of (k + kh) and g":

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၄ ဖြစ်ပါစေ။
က + ခ နှင့် ဂ ၏ မြှောက်လဒ် ကို ပြသမည်။         // (2 + 3) × 4
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၀
```

## Word phrases inside word phrases

Word phrases can nest, and we read them from left to right. Each result word, such as
`မြှောက်လဒ်`, uses the parts right before it that end in the particles it needs, and its
result then joins the rest of the line:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၄ ဖြစ်ပါစေ။

// k and (the product of kh and g): the sum      → 2 + (3 × 4)
က နှင့် ခ နှင့် ဂ ၏ မြှောက်လဒ် ၏ ပေါင်းလဒ် ကို ပြသမည်။

// (the sum of k and kh) and g: the product      → (2 + 3) × 4
က နှင့် ခ ၏ ပေါင်းလဒ် နှင့် ဂ ၏ မြှောက်လဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၄
၂၀
```

In the first line, `မြှောက်လဒ်` uses `ခ နှင့်` and `ဂ ၏`, the two parts right before it.
A `၏` follows its result, so `ပေါင်းလဒ်` then uses `က နှင့်` and that result. In the
second line, `ပေါင်းလဒ်` comes first and uses `က နှင့်` and `ခ ၏`, and its result, with
`နှင့်` after it, becomes the first part of `မြှောက်လဒ်`.

## Parentheses move a phrase into a sum

```text
(<word phrase>) * <value>
```

A word phrase cannot go straight inside a sum with signs, so to use one there, we put it
in parentheses. Without them, the compiler stops with an error:

```zaka-error ZK0203
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
က နှင့် ခ ၏ ပေါင်းလဒ် * ၂ ကို ပြသမည်။
```

```text
error[ZK0203]: this statement does not end in a statement verb, found "*"
```

With the parentheses, the line works:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
(က နှင့် ခ ၏ ပေါင်းလဒ်) * ၂ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၀
```

The same goes for a [conversion](conversion.md) inside a sum, and for a
[logical expression](logic.md) that we print, as in `(က ဖြစ်၍ ခ) ကို ပြသမည်။`

## Needless parentheses

Parentheses around a whole value that we print or keep change nothing, so Zaka warns
about them, much as an editor points out a double space:

```zaka-warning ZK0507
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
(က + ၁) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0507]: these parentheses are not needed
```

A logical expression that we print is the exception, because it needs its parentheses.

## Details

- **Values read with a verb.** A value that ends in a verb, such as the number of cells
  (elements) in a list, `xs ၏ အကွက်အရေအတွက်`, can have no operator after it unless it is in parentheses, so
  we write `(xs ၏ အကွက်အရေအတွက်) == ၀`. In a condition or a declaration the compiler
  reports ZK0201, and its help tells us to add the parentheses
  ([Grouping](../../manual/language/expressions.md#grouping)).
- **Calls.** A function call needs no parentheses inside a sum
  ([Grouping](../../manual/language/expressions.md#grouping)).
- **Bitwise before comparison.** Unlike C, `&`, `^` and `|` bind tighter than `==`, so
  `၆ & ၃ == ၂` means `(၆ & ၃) == ၂` ([Precedence](../../manual/language/expressions.md#precedence)).
- **How deep.** Parentheses and other nested parts can go 128 levels deep, and a program
  that goes further gets ZK0244 ([Grouping](../../manual/language/expressions.md#grouping)).

## Recap

- Zaka works out `*`, `/` and `%` before `+` and `-`, and operators of one level go from
  left to right.
- Parentheses change the order: `(၂ + ၃) * ၄` is 20.
- Each part of a word phrase can be a whole sum with signs.
- A word phrase inside a sum with signs needs parentheses.
- Parentheses around a whole printed value are not needed, and Zaka warns about them.

## Hands on

- Write `(၂ + ၃) * ၄` with words alone.
- Print `၁၀ - ၃ - ၂` and `၁၀ - (၃ - ၂)`. Do the two match?

Next: [Control flow](../control-flow/README.md).
