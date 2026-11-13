---
title: The stdlib
description: Text, paths, lists and files with the modules the compiler carries - finding and splitting text, the parts of a path, sorting a list, and folders.
---

# The stdlib

Zaka carries a few modules of functions, the *stdlib*. A program imports the names
it needs from each, as from a file of its own ([Modules](../modules/README.md)). The
functions of text, paths and lists are written in Zaka; each call reads as a sentence.

## Text

`"စံကား/စာသား"` finds, splits, joins and changes text:

```zaka
"စံကား/စာသား" မှ ပါမပါစစ်၊ ခွဲ၊ ဆက် နှင့် စာလုံးကြီးပြောင်း တို့ကို ထည့်သွင်းမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ကမ္ဘာ" ကို "မင်္ဂလာပါ ကမ္ဘာ" ထဲတွင် ပါမပါစစ် သည့် ရလဒ် ကို ပြသမည်။
ကိန်းရှင် သီးများ သည် "ပန်း၊ငှက်ပျော၊သရက်" ကို "၊" ဖြင့် ခွဲ သည့် ရလဒ် ဖြစ်ပါစေ။
သီးများ ကို ပြသမည်။
သီးများ ကို " / " ဖြင့် ဆက် သည့် ရလဒ် ကို ပြသမည်။
"zaka" ကို စာလုံးကြီးပြောင်း သည့် ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
["ပန်း"၊ "ငှက်ပျော"၊ "သရက်"]
ပန်း / ငှက်ပျော / သရက်
ZAKA
```

The module also reads text as an integer. Import `ကိန်းပြည့်အဖြစ်ပြောင်း`, and the
conversion `ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်` works on text: it gives a result,
good or failed, as [Errors and results](../error-handling/README.md) shows.

```zaka
"စံကား/စာသား" မှ ကိန်းပြည့်အဖြစ်ပြောင်း ကို ထည့်သွင်းမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"၄၂" ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
"လေးဆယ်" ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရလဒ်ကောင်း(၄၂)
အမှားအယွင်း(ကိန်းဂဏန်းဖတ်ခြင်းအမှားအယွင်းများထဲမှ ကိန်းဂဏန်းမဟုတ်သောစာလုံးပါခြင်း)
```

## Paths

`"စံကား/လမ်းကြောင်း"` reads the parts of a path. After `၏`, a function reads like a field
of the path:

```zaka
"စံကား/လမ်းကြောင်း" မှ ဖိုင်အမည်၊ ဖိုင်အမည်နောက်တွဲ နှင့် ဖိုင်လမ်းကြောင်းဆက် တို့ကို ထည့်သွင်းမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လမ်း သည် "photos" ကို "ပန်း.jpg" နှင့် ဖိုင်လမ်းကြောင်းဆက် သည့် ရလဒ် ဖြစ်ပါစေ။
လမ်း ကို ပြသမည်။
လမ်း ၏ ဖိုင်အမည် ကို ပြသမည်။
လမ်း ၏ ဖိုင်အမည်နောက်တွဲ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
photos/ပန်း.jpg
ပန်း.jpg
ဒေတာ("jpg")
```

## Lists

`"စံကား/စာရင်း"` sorts, removes, finds and reverses. Each function gives back a new list:
the list it takes stands as it was.

```zaka
"စံကား/စာရင်း" မှ ငယ်စဉ်ကြီးလိုက်စီ၊ ဖယ် နှင့် ပြောင်းပြန်စီ တို့ကို ထည့်သွင်းမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ်များ သည် [၇၀၊ ၉၅၊ ၆၁၊ ၈၈] ဖြစ်ပါစေ။
အမှတ်များ ကို ငယ်စဉ်ကြီးလိုက်စီ သည့် ရလဒ် ကို ပြသမည်။
အမှတ်များ မှ ၀ အကွက်ကို ဖယ် သည့် ရလဒ် ကို ပြသမည်။
အမှတ်များ ကို ပြောင်းပြန်စီ သည့် ရလဒ် ကို ပြသမည်။
အမှတ်များ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၆၁၊ ၇၀၊ ၈၈၊ ၉၅]
[၉၅၊ ၆၁၊ ၈၈]
[၈၈၊ ၆၁၊ ၉၅၊ ၇၀]
[၇၀၊ ၉၅၊ ၆၁၊ ၈၈]
```

## Files and folders

`"စံကား/အဝင်အထွက်"`, the module of [Input and output](../io/README.md), makes folders, adds
to files and lists what a folder holds. Each of these can fail, so the program prints the
whole result:

```zaka
"စံကား/အဝင်အထွက်" မှ ဖိုင်တွဲတစ်ခုပြုလုပ်၊ နောက်ဆုံးမှဆက်ရေး နှင့် စု တို့ကို ထည့်သွင်းမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"notes" နေရာတွင် ဖိုင်တွဲတစ်ခုပြုလုပ် သည့် ရလဒ် ကို ပြသမည်။
"ဒီနေ့" ကို "notes/day.txt" ၏ နောက်ဆုံးမှဆက်ရေး သည့် ရလဒ် ကို ပြသမည်။
"notes" ရှိ ဖိုင်တွဲအတွင်းရှိ အမည်များကို စု သည့် ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အောင်မြင်သည့်သတင်း
အောင်မြင်သည့်သတင်း
ရလဒ်ကောင်း(["day.txt"])
```

[The stdlib](../../manual/language/stdlib.md) in Docs lists each function with its rules.
