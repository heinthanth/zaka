---
title: Changing a variable
description: Give a variable a new value with ... ကို ... ဟု ထားမည်။, and duplicate a value into one.
---

# Changing a variable

A variable can be given a new value later on. This is called *assignment*. In Zaka the
sentence reads "set x to v": `x ကို v ဟု ထားမည်။`

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။
အသက် ကို ပြသမည်။

အသက် ကို ၂၄ ဟု ထားမည်။                // set age to 24
အသက် ကို ပြသမည်။

အသက် ကို အသက် + ၁ ဟု ထားမည်။          // set age to age + 1
အသက် ကို ပြသမည်။

အသက် ကို အသက် နှင့် ၁၀ ကို ပေါင်းသည့်ရလဒ် ဟု ထားမည်။   // the word form works too
အသက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၈
၂၄
၂၅
၃၅
```

| Zaka | Reads as | Part |
| --- | --- | --- |
| `အသက်` | "age" | the variable to change |
| `ကို` | (object) | |
| `အသက် + ၁` | age + 1 | the new value |
| `ဟု` | "so", "as" | marks the new value |
| `ထားမည်` | "please put", "please set" | the verb: assign |

## How `အသက် ကို အသက် + ၁ ဟု ထားမည်။` works

To a maths student, "age = age + 1" looks impossible. To a computer it is two steps:

1. Work out the new value, `အသက် + ၁`, using the **current** value of `အသက်` (24).
2. Put the result (25) into `အသက်`, replacing the old value.

## Storing a true/false answer

A sentence that is true or false can be stored too:

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

One more verb puts a value into a variable: `ကူးထည့်မည်` ("will duplicate in"). After
`ကိန်းရှင်`, it makes a new variable of the value's kind:

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

A name in braces inside a string, as `{အမည်}`, is a *template slot*: the string holds the
value of the variable in its place. The [Text](../text/README.md#template-literals) page
shows slots in full.

| Zaka | Reads as |
| --- | --- |
| `မိတ္တူ ထဲသို့` | "into the duplicate" |
| `အမည် ကို` | "the name" (object) |
| `ကူးထည့်မည်` | "will duplicate in" |

On a number, text or a list, this does what `ဟု ထားမည်` and `ဖြစ်ပါစေ` do, since each of them
gives a duplicate. The verb matters for the [shared objects](../structs/objects.md#a-duplicate-of-its-own)
of a later page, where it makes a new object.

## The new value must fit

The new value must suit the variable's kind, just as the initial value did:

```zaka-error ZK0401
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။
အသက် ကို ၂.၅ ဟု ထားမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0401]: expected isize, found f64
```

## Hands on

- Start a variable at `၁` and double it four times, printing it after each change.
- Make a variable `ကျန်` (remaining) of `၁၀၀`, then subtract `၃၅` from it.

Next: [Names](names.md).
