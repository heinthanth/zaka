---
title: Errors and results
description: Functions that give back a good result or an error value, and how a caller deals with each.
---

# Errors and results

Some work cannot be done each time it is asked for. A division with a zero divisor has
no answer, and a number below zero is no count of things. A function for such work can
*fail*. In Zaka a failure is a value: the function gives back either its good result or an *error value*,
and each call tells what happens when the function fails.

## A function that can fail

The header names the whole result kind: the kind of the good result, then `သို့မဟုတ်`, then the kind of the error value with `အမှားအယွင်း` after it:

```zaka-file
// divide.zaka
// Gives back a good integer, or text that tells what went wrong.
လုပ်ဆောင်ချက် စားခွဲ သည်
ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
အကယ်၍ ခ သည် ၀ နှင့် ညီလျှင် "သုညဖြင့် စား၍ မရပါ" ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
က / ခ ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

Two parts are new:

| Where | Written | Reads as |
| --- | --- | --- |
| header | `<kind> ရလဒ် သို့မဟုတ် <kind> အမှားအယွင်း ကို ပြန်ပေးမည်။` | "will return a result, or an error value of this kind" |
| inside | `<value> ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။` | "give back the value as an error value" |

The good result is given back with `ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ` (in a function that can fail,
`ရလဒ်အဖြစ်` is ZK0455) or the variable `ရလဒ်`. The error value can be of whatever kind is needed: an integer, text, a struct. A function can
also fail and have no result; it writes `အောင်မြင်သည့်သတင်း` for the good side.
[Writing result kinds](kinds.md) shows the written form of the whole kind.

## What comes next

A caller of such a function has to tell what happens when it fails. [Passing a
failure on](passing.md) hands the error value up to the caller of the caller, and [Reading a
whole result](results.md) keeps the result and looks at it. [Writing result kinds](kinds.md)
writes the kind of such a result, for a variable, a parameter, a field or a clause.

Next: [Passing a failure on](passing.md).
