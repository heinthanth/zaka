---
title: The text module
description: Find, split, join and change text, and read text as an integer.
---

# The text module

The text module, `"စံကား/စာသား"`, finds, splits, joins and changes text, and reads text
as an integer. Its functions are written in Zaka, and text is a value, so each function
returns new text and leaves its argument as it was.

| Call | What it returns |
| --- | --- |
| `<piece> ကို <text> ထဲတွင် ရှာ` | the first u8 number where the piece appears, or blank data, as a `လက္ခဏာမဲ့ ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` |
| `<piece> ကို <text> ထဲတွင် ပါမပါစစ်` | whether the piece appears in the text |
| `<text> ကို <piece> ဖြင့် စမစစစ်` | whether the text starts with the piece |
| `<text> ကို <piece> ဖြင့် ဆုံးမဆုံးစစ်` | whether the text ends with the piece |
| `<text> ကို <separator> ဖြင့် ခွဲ` | the pieces between the separators, as a `စာသား စာရင်း`; a blank separator returns the text alone |
| `<text> ကို စာကြောင်းများအဖြစ်ခွဲ` | the lines, each without its `\n` or `\r\n`; a line break at the end starts no blank line |
| `<text> ကို စကားလုံးများအဖြစ်ခွဲ` | the words: the runs of characters between spaces, tabs and line breaks |
| `<list> ကို <separator> ဖြင့် ဆက်` | the texts of a list joined, with the separator between each two |
| `<text> ကို ဘေးနှစ်ဖက်တွင်နေရာလွတ်ဖယ်` | the text without the white space at its two ends |
| `<text> တွင် <old> ကို <new> ဖြင့် အကုန်အစားထိုး` | the text with each old piece replaced, from the start on |
| `<text> ကို <count> ကြိမ် ထပ်ခါထပ်ခါဆက်` | the text repeated exactly that many times, or empty text for a count of zero or less |
| `<text> ကို စာလုံးကြီးပြောင်း`, `<text> ကို စာလုံးသေးပြောင်း` | the text with each Latin letter in upper or in lower case |
| `<text> ကို <kind> တစ်<measure word> အဖြစ် ပြောင်းလဲသော ရလဒ်` | the text read as an integer, a decimal number or a character, or a failure ([Conversions in the stdlib](../language/conversions.md#conversions-in-the-stdlib)). Import `စမ်းသပ်ပြောင်းလဲ` to use these conversions |

A u8 number counts the u8s of the UTF-8 form of the text, as a [range of
text](../language/text.md#ranges) does, so `ကမ္ဘာ` starts at 22 in `မင်္ဂလာ ကမ္ဘာ`, because each
Burmese letter uses three u8s. A search walks the text once for each place it tries, so
finding a piece of `m` u8s in text of `n` u8s takes up to `n × m` steps.

```zaka
"စံကား/စာသား" မှ ရှာ၊ ခွဲ၊ ဆက်၊ ဘေးနှစ်ဖက်တွင်နေရာလွတ်ဖယ် နှင့် အကုန်အစားထိုး တို့ကို ထည့်သွင်းမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ကမ္ဘာ" ကို "မင်္ဂလာ ကမ္ဘာ" ထဲတွင် ရှာ သည့် ရလဒ် ကို ပြသမည်။
ကိန်းရှင် အပိုင်းများ သည် "a,b,,c" ကို "," ဖြင့် ခွဲ သည့် ရလဒ် ဖြစ်ပါစေ။
အပိုင်းများ ကို ပြသမည်။
အပိုင်းများ ကို " | " ဖြင့် ဆက် သည့် ရလဒ် ကို ပြသမည်။
ကိန်းရှင် ဖယ်ပြီး သည် "  hi  " ကို ဘေးနှစ်ဖက်တွင်နေရာလွတ်ဖယ် သည့် ရလဒ် ဖြစ်ပါစေ။
"[{ဖယ်ပြီး}]" ဟု ပြသမည်။
"a.b.c" တွင် "." ကို "::" ဖြင့် အကုန်အစားထိုး သည့် ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဒေတာ(၂၂)
["a"၊ "b"၊ ""၊ "c"]
a | b |  | c
[hi]
a::b::c
```
