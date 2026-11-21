---
title: Changing a variable
description: Give a variable a new value with ... ကို ... ဟု ထားမည်။, and duplicate a value into one.
---

# Changing a variable

After we declare a variable, we can give it a new value, and this is called
*assignment*. In Zaka we write it as "set x to v": `x ကို v ဟု ထားမည်။`.

## You will learn

- How to give a variable a new value with `ထားမည်`.
- How `အသက် ကို အသက် + ၁ ဟု ထားမည်။` works.
- How to store a true/false answer in a variable.
- How to duplicate (copy) a value into a new variable with `ကူးထည့်မည်`.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။
အသက် ကို ပြသမည်။
အသက် ကို ၂၄ ဟု ထားမည်။                // set age to 24
အသက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၈
၂၄
```

To give a variable a new value, we write the variable with `ကို`, the new value with
`ဟု`, and then `ထားမည်`:

```
<variable> ကို <value> ဟု ထားမည်။
```

| Zaka | Reads as | Part |
| --- | --- | --- |
| `အသက်` | "age" | the variable to change |
| `ကို` | (the object) | |
| `၂၄` | 24 | the new value |
| `ဟု` | "so", "as" | comes after the new value |
| `ထားမည်` | "please put", "please set" | the verb: assign |

## Using the old value

The new value can use the variable itself, as in `အသက် ကို အသက် + ၁ ဟု ထားမည်။`. To a
maths student, "age = age + 1" looks impossible, but the computer does it in two steps:

1. It works out the new value, `အသက် + ၁`, with the **current** value of `အသက်`.
2. It puts the result into `အသက်`, in place of the old value.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၄ ဖြစ်ပါစေ။
အသက် ကို အသက် + ၁ ဟု ထားမည်။          // set age to age + 1
အသက် ကို ပြသမည်။
အသက် ကို အသက် နှင့် ၁၀ ၏ ပေါင်းလဒ် ဟု ထားမည်။   // the word form works too
အသက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၅
၃၅
```

## Storing a true/false answer

We can also store a sentence that is true or false, such as `အသက် သည် ၁၈ ထက် ကြီးသည်`
("age is greater than 18"), in a `မှန်မှားတန်ဖိုး` variable:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
ကိန်းရှင် အရွယ်ရောက် သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်ပါစေ။   // adult, set on the next line
အရွယ်ရောက် ကို အသက် သည် ၁၈ ထက် ကြီးသည် ဟု ထားမည်။
အရွယ်ရောက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
```

## Duplicating in

```
ကိန်းရှင် <name> ထဲသို့ <value> ကို ကူးထည့်မည်။
```

Another verb puts a value into a variable: `ကူးထည့်မည်` ("will duplicate in"). After
`ကိန်းရှင်`, it makes a new variable with a duplicate (copy) of the value, and the new
variable gets the data kind of the value:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည် သည် "မောင်" ဖြစ်ပါစေ။
ကိန်းရှင် မိတ္တူ ထဲသို့ အမည် ကို ကူးထည့်မည်။      // a duplicate: both keep the text
"[{အမည်}] [{မိတ္တူ}]" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[မောင်] [မောင်]
```

| Zaka | Reads as |
| --- | --- |
| `မိတ္တူ ထဲသို့` | "into the duplicate" |
| `အမည် ကို` | "the name" (the object) |
| `ကူးထည့်မည်` | "will duplicate in" |

A name in braces inside text, such as `{အမည်}`, is a *template slot*, and the text shows
the value of the variable in its place. The [Text](../text/README.md#template-literals)
page shows slots in full.

On a number, text or a list, `ကူးထည့်မည်` does the same as `ဟု ထားမည်` and `ဖြစ်ပါစေ`,
because each of them makes a duplicate. The verb matters for the [shared
objects](../structs/objects.md#duplicating-an-object) of a later page, where it makes a
new object.

## The new value must fit

The new value must suit the data kind of the variable, just as the first value did:

```zaka-error ZK0401
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။
အသက် ကို ၂.၅ ဟု ထားမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0401]: expected isize, found f64
```

## Details

- **Places other than a variable.** Later chapters assign to a field of a struct and to
  a cell of a list in the same form. The counter of a counted loop cannot change
  (ZK0307), and neither can a constant (ZK0306). See
  [Assignment](../../manual/language/declarations.md#assignment).
- **The initial value.** `ကနဦးတန်ဖိုး` works as a new value too:
  `အသက် ကို ကနဦးတန်ဖိုး ဟု ထားမည်။` sets `အသက်` to `၀`.
- **A deep duplicate.** `အကုန်ကူးထည့်မည်` ("will duplicate all in") also duplicates each
  shared object that the value reaches. See [Copies into a
  variable](../../manual/language/declarations.md#copies-into-a-variable).

## Recap

- We write `<variable> ကို <value> ဟု ထားမည်။` to give a variable a new value.
- The computer works out the new value first, with the current value, and then stores it.
- `ကိန်းရှင် <name> ထဲသို့ <value> ကို ကူးထည့်မည်။` declares a new variable with a
  duplicate of the value.
- The new value must fit the data kind of the variable.

## Hands on

- Start a variable at `၁` and double it four times, printing it after each change.
- Make a variable `ကျန်` (remaining) of `၁၀၀`, then subtract `၃၅` from it.
- Set an integer variable to `ကနဦးတန်ဖိုး` and print it.

Next: [Names](names.md).
