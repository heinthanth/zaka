---
title: The list module
description: Sort, search, reverse and remove the cells of a list.
---

# The list module

The list module, `"စံကား/စာရင်း"`, sorts, searches, reverses and removes the cells of a
list. Its functions are written in Zaka, and a list is a value, so each function returns a
new list and leaves its argument as it was.

Each function of the list module is [generic](../language/generics.md), so it accepts a list of each
data kind.

| Call | What it returns |
| --- | --- |
| `<list> ကို ငယ်စဉ်ကြီးလိုက်စီ` | the cells from the smallest to the largest; the cell kind must work with `<` (`နှိုင်းယှဉ်နိုင်သော`) |
| `<list> ကို (<function>) ကိုသုံး၍ စီ` | the cells in the order given by the comparison function, which returns true when its first argument comes before its second |
| `<list> မှ <number> အကွက်ကို ဖယ်` | the list without the cell at a number counted from 0; a number outside the list stops the program |
| `<list> ထဲတွင် <value> ၏ အကွက်နေရာကိုရှာ` | the first cell number, from 0, whose cell has the value, or blank data |
| `<value> ကို <list> ထဲတွင် ပါမပါစစ်` | whether a cell has the value |
| `<list> ကို ပြောင်းပြန်စီ` | the cells from the last to the first |

- Both sorts are a merge sort, which takes `n × log n` steps for `n` cells, and cells where
  neither comes before the other keep their order. Text does not work with `<`, so we sort a
  list of text through `စီ` with a function that compares two texts.
- `စီ` is a reserved word of the counted loop, so the module declares the function as
  `{စီ}`, an import names it `{စီ}`, and a call writes it bare, right before its call end.
- The function before `ကိုသုံး၍` goes in parentheses, because without them
  `f ဟူသော လုပ်ဆောင်ချက် ကို` would start a long call.
- `ပါမပါစစ်` of lists and `ပါမပါစစ်` of text can both be imported into one file, and each
  call uses the one that its arguments fit.
- Each function duplicates (copies) the list once.

```zaka
"စံကား/စာရင်း" မှ ငယ်စဉ်ကြီးလိုက်စီ၊ {စီ}၊ ဖယ်၊
အကွက်နေရာကိုရှာ နှင့် ပြောင်းပြန်စီ တို့ကို ထည့်သွင်းမည်။

ပိုကြီး ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
က သည် ခ ထက် ကြီးသည် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် l သည် [၅၊ ၃၊ ၈၊ ၁] ဖြစ်ပါစေ။
l ကို ငယ်စဉ်ကြီးလိုက်စီ သည့် ရလဒ် ကို ပြသမည်။
l ကို (ပိုကြီး ဟူသော လုပ်ဆောင်ချက်) ကိုသုံး၍ စီ သည့် ရလဒ် ကို ပြသမည်။
l မှ ၁ အကွက်ကို ဖယ် သည့် ရလဒ် ကို ပြသမည်။
l ထဲတွင် ၈ ၏ အကွက်နေရာကိုရှာ သည့် ရလဒ် ကို ပြသမည်။
l ကို ပြောင်းပြန်စီ သည့် ရလဒ် ကို ပြသမည်။
l ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၃၊ ၅၊ ၈]
[၈၊ ၅၊ ၃၊ ၁]
[၅၊ ၈၊ ၁]
ဒေတာ(၂)
[၁၊ ၈၊ ၃၊ ၅]
[၅၊ ၃၊ ၈၊ ၁]
```

## Open

- Removing a cell counted from 1, `<list> မှ <number> ကြိမ်မြောက်အကွက်ကို ဖယ်`, waits
  for generic functions that share a name, because a generic function at present has one
  declaration for each name.
