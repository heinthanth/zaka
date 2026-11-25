---
title: The text module
description: Find, split, join and change text, and read text as an integer.
module: စံကား/စာသား
---

# The text module

The text module, `"စံကား/စာသား"`, finds, splits, joins and changes text, and reads text
as an integer. Its functions are written in Zaka, and text is a value, so each function
returns new text and leaves its argument as it was. Text reads as an integer, a decimal
number or a character through a conversion that can fail ([Conversions in the
stdlib](../language/conversions.md#conversions-in-the-stdlib)); import `စမ်းသပ်ပြောင်းလဲ` to
use it.

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
