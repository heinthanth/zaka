---
title: Errors and results
description: Functions that return a good result or an error value, and how a caller handles each one.
---

# Errors and results

Some work has no answer in certain cases, such as a division with a zero divisor, so a
function that does such work can *fail*. When it fails, it returns an *error value*
(error) instead of its *good result* (`Ok` value).

## You will learn

- what a function that can fail returns
- how to write the function signature of a function that can fail, with a result kind (result type)
- how the function returns an error value
- which pages cover the caller's side

```zaka
စားခွဲ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ ခ သည် ၀ နှင့် ညီလျှင် "သုညဖြင့် စား၍ မရပါ" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
က / ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စားခွဲ(၁၀၀၊ ၂၅) ကို ပြသမည်။
စားခွဲ(၁၀၀၊ ၀) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရလဒ်ကောင်း(၄)
အမှားအယွင်း("သုညဖြင့် စား၍ မရပါ")
```

The first call succeeds and returns `ရလဒ်ကောင်း` ("good result") with `၄` inside, while
the second call has a zero divisor, so it returns `အမှားအယွင်း` ("error value") with the
text inside. Here the program start block just prints each whole result so that we can see
both outcomes, but most callers either pass the failure on or test the result, which the
next two pages explain.

## The function signature

In the function signature, we use the result clause, the third line of `စားခွဲ`, to define
what the function returns. To let a function fail, we write a result kind (result type)
there instead of a single data kind (type), so that it gives both the data kind of the good
result and the data kind of the error value:

```text
အောက်ပါတို့ကို လုပ်ဆောင်ကာ <good kind> သို့မဟုတ် အမှားအယွင်းအနေဖြင့် <error kind> ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
```

| Zaka | Reads as |
| --- | --- |
| `အောက်ပါတို့ကို လုပ်ဆောင်ကာ` | "will do the following, and" |
| `<good kind>` | the data kind of the good result, `ကိန်းပြည့်` above |
| `သို့မဟုတ်` | "or" |
| `အမှားအယွင်းအနေဖြင့် <error kind>` | "as an error value, a `<error kind>`": `စာသား` above |
| `ရှိနိုင်သော` | "that can hold" |
| `ဒေတာဆဲလ် တစ်ခု` | "one data cell" |
| `ကို ပြန်ပေးမည်` | "will give back" |

A function can fail when we write a result kind in its result clause, and it cannot fail
otherwise.

## Giving back an error value

```text
<value> ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
```

This line means "will return the error value `<value>`", so we write it when the function
should fail, and it ends the function at once. The good result is returned as in other
functions, with `<value> ကို ပြန်ပေးမည်။`. The error value can be of whatever data kind the
program needs, such as text, an integer or a struct, and in the next example it is the
wrong number itself:

```zaka
// a count of things: below zero is an error, and the error value is the number itself
အရေအတွက်စစ် ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ကိန်းပြည့် ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ အောက် ငယ်လျှင် န ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အရေအတွက်စစ်(၃) ကို ပြသမည်။
အရေအတွက်စစ်(၀ - ၂) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရလဒ်ကောင်း(၃)
အမှားအယွင်း(-၂)
```

## What comes next

Each caller of such a function must decide what to do when it fails. [Passing a failure
on](passing.md) shows how we send the error value up to the caller's caller, and [Reading
a whole result](results.md) shows how we keep the result and check which part it
contains. After those, [Writing result kinds](kinds.md) shows how we write the data kind of
a result for a variable, a parameter, a field or a result clause.

## Details

- When a function that can fail has no good result to return, we write
  `အောင်မြင်သည့်သတင်း` ("news of success") in place of the good data kind, and when its
  failure carries no value, we write `မအောင်မြင်သည့်သတင်း` in place of the error data kind
  ([Writing result kinds](kinds.md)).
- The program start block cannot fail, so we cannot give it a result kind, and it cannot
  pass a failure on ([The program start block cannot pass a failure
  on](passing.md#the-program-start-block-cannot-pass-a-failure-on)).
- The word after `တစ်` in the result clause is a measure word, and `ခု` is the right one
  for a result kind ([The result
  clause](../../manual/language/error-handling.md#the-result-clause)).
- The full rules on returning each part of a result are in [Giving back an error
  value](../../manual/language/error-handling.md#giving-back-an-error-value).

## Recap

- A function that can fail returns a whole result, which is either a good result or an
  error value.
- In its function signature, we write both data kinds in the result clause:
  `<good kind> သို့မဟုတ် အမှားအယွင်းအနေဖြင့် <error kind> ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။`
- We write `<value> ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။` to fail and `<value> ကို ပြန်ပေးမည်။`
  to return a good result.
- When we print a result, it looks like `ရလဒ်ကောင်း(…)` or `အမှားအယွင်း(…)`.

## Hands on

- Write a function that returns twice a number and fails with text when the number is
  below zero, then print its result for `၄` and for `၀ - ၁`.
- Change the error value of `စားခွဲ` to an integer, and print both results again.

Next: [Passing a failure on](passing.md).
