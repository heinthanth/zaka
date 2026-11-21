---
title: If and else
description: Run sentences when a condition is true, and others when it is false.
---

# If and else

An `if` statement runs a block of code when its condition is true. We can add an `else`
block to run when the condition is false. In Burmese, the `if` form reads "if
*condition*, then *do this*".

## You will learn

- how to run a sentence just when a condition is true
- how to add an "otherwise" part (`else`)
- how to choose between more than two cases (`else if`)
- how to put several sentences, or another `if`, inside a branch

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၅ ဖြစ်ပါစေ။

// if age is less than 18, then ...
အကယ်၍ အသက် သည် ၁၈ အောက် ငယ်လျှင် "ကလေး" ဟု ပြသမည်။

// nothing happens when the condition is false
အကယ်၍ အသက် သည် ၆၀ ထက် ကြီးလျှင် "အငြိမ်းစား" ဟု ပြသမည်။
```

```text
ကလေး
```

The age is 15, so the first sentence prints `ကလေး` ("child"), while the second one prints
nothing, because 15 is not greater than 60. To run a sentence when a condition is true,
we use this pattern:

```text
အကယ်၍ <a> သည် <b> အောက် ငယ်လျှင် <inner part>
```

| Zaka | Reads as |
| --- | --- |
| `အကယ်၍` | "if" |
| `<a> သည် <b> အောက် ငယ်` | "`<a>` is less than `<b>`", a comparison in words |
| `လျှင်` | "then"; it joins the last verb of the condition, as in `ငယ်လျှင်` |
| `<inner part>` | one sentence, or a block |

The condition is a comparison in words, the same as on the [Comparison](../operators/comparison.md)
page, except that its verb ends in `လျှင်` in place of `သည်`.

## A true/false value as the condition

```text
အကယ်၍ <true/false value> ဖြစ်လျှင် <inner part>
```

The condition can also be a true/false value (boolean), such as a variable or a
comparison in signs. A value like this has no verb for `လျှင်` to join, so we write
`ဖြစ်လျှင်` ("if it is so, then") as a separate word after it:

```zaka
ကိန်းရှင် အောင် သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် မှန် ဖြစ်ပါစေ။ // passed?
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၃ ဖြစ်ပါစေ။
အကယ်၍ အောင် ဖြစ်လျှင် "အောင်ပါသည်" ဟု ပြသမည်။
အကယ်၍ အမှတ် > ၇၀ ဖြစ်လျှင် "ကောင်းပါသည်" ဟု ပြသမည်။
```

```text
အောင်ပါသည်
ကောင်းပါသည်
```

## Otherwise

```text
အကယ်၍ <condition>လျှင် <inner part>
သို့မဟုတ်ပါက <inner part>
```

To run something when the condition is false, we add a line that starts with
`သို့မဟုတ်ပါက` ("otherwise"), which works like `else` in other languages:

```zaka
ကိန်းရှင် အောင် သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် မမှန် ဖြစ်ပါစေ။ // passed?
အကယ်၍ အောင် ဖြစ်လျှင် "အောင်ပါသည်" ဟု ပြသမည်။
သို့မဟုတ်ပါက "ကျပါသည်" ဟု ပြသမည်။
```

```text
ကျပါသည်
```

## More than two choices

```text
အကယ်၍ <condition>လျှင် <inner part>
ထိုသို့မဟုတ်ဘဲ <condition>လျှင် <inner part>
သို့မဟုတ်ပါက <inner part>
```

To check another condition when the first one is false, we add a line that starts with
`ထိုသို့မဟုတ်ဘဲ` ("if not so, but"), which works like `else if`. Zaka tests the
conditions from top to bottom and runs the **first** branch whose condition is true:

```zaka
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၃ ဖြစ်ပါစေ။
အကယ်၍ အမှတ် သည် ၈၀ အောက် မငယ်လျှင် "အဆင့် က" ဟု ပြသမည်။
ထိုသို့မဟုတ်ဘဲ အမှတ် သည် ၆၀ အောက် မငယ်လျှင် "အဆင့် ခ" ဟု ပြသမည်။
ထိုသို့မဟုတ်ဘဲ အမှတ် သည် ၄၀ အောက် မငယ်လျှင် "အဆင့် ဂ" ဟု ပြသမည်။
သို့မဟုတ်ပါက "အဆင့် ဃ" ဟု ပြသမည်။
```

```text
အဆင့် ခ
```

73 is at least 40 too, but Zaka stops at the first branch that matches, so it prints
`အဆင့် ခ` alone. We can add more `ထိုသို့မဟုတ်ဘဲ` lines when we need more choices.

## Branches with several sentences

To run several sentences in a branch, we make the inner part a block:

```zaka
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၃ ဖြစ်ပါစေ။
အကယ်၍ အမှတ် % ၂ သည် ၀ နှင့် ညီလျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"စုံကိန်း" ဟု ပြသမည်။
အမှတ် / ၂ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
သို့မဟုတ်ပါက အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မကိန်း" ဟု ပြသမည်။
(အမှတ် + ၁) / ၂ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
မကိန်း
၃၇
```

`စုံကိန်း` is "even number" and `မကိန်း` is "odd number".

## An if inside an if

A one-sentence branch cannot be another `if`, because it would then be unclear which `if`
an "otherwise" belongs to, so the compiler stops with an error:

```zaka-error ZK0215
အကယ်၍ မှန် ဖြစ်လျှင် အကယ်၍ မမှန် ဖြစ်လျှင် ၁ ကို ပြသမည်။
```

```text
error[ZK0215]: a single-statement block cannot be another conditional
```

To put an `if` inside an `if`, we write the inner one in a block:

```zaka
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၉၅ ဖြစ်ပါစေ။
အကယ်၍ အမှတ် သည် ၄၀ အောက် မငယ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"အောင်" ဟု ပြသမည်။
အကယ်၍ အမှတ် သည် ၉၀ ထက် ကြီးလျှင် "ဂုဏ်ထူး" ဟု ပြသမည်။     // distinction
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
အောင်
ဂုဏ်ထူး
```

## Details

- **The condition must be true or false.** A number or other value as the condition is
  the error ZK0402 ([If](../../manual/language/control-flow.md#if)).
- **A whole sentence as the condition.** After a full comparison that ends in `သည်`, such
  as `က သည် ၀ ထက် ကြီးသည်`, we also write `ဖြစ်လျှင်` as a separate word
  ([If](../../manual/language/control-flow.md#if)).
- **Otherwise with no if.** A `ထိုသို့မဟုတ်ဘဲ` or `သို့မဟုတ်ပါက` line with no `အကယ်၍` before
  it is the error ZK0211 ([If](../../manual/language/control-flow.md#if)).
- **Logic in a condition.** We can join comparisons in a condition with `ဖြစ်၍` and
  `သို့မဟုတ်`, as on the [Logic](../operators/logic.md) page.

## Recap

- We write `အကယ်၍ <condition>လျှင် <inner part>` to run something when a condition is
  true.
- After a true/false variable or a comparison in signs, we write `ဖြစ်လျှင်` as a
  separate word.
- `သို့မဟုတ်ပါက` ("otherwise") runs when no condition was true, and `ထိုသို့မဟုတ်ဘဲ`
  adds another condition.
- Zaka runs the first branch whose condition is true and skips the rest.
- An `if` inside a one-sentence branch must go in a block.

## Hands on

- Print whether a number is positive, negative, or zero.
- Print `"FizzBuzz"` when 15 divides a number, `"Fizz"` when 3 divides it, `"Buzz"` when 5
  does, and the number itself otherwise. Check 9, 10, 15 and 7. Does the order of the
  tests matter?

Next: [While loops](while.md).
