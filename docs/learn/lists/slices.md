---
title: Slices
description: A slice borrows a run of cells with their count, so one function reads and writes whole lists and parts of them with no duplicate.
---

# Slices

A function that takes `ကိန်းပြည့် စာရင်း` takes lists of each count, but each time a
whole list, and a function that writes its parameter writes a duplicate. A *slice* borrows
a run of cells and keeps their count: a whole list or a part of one. A [view of
text](../forms/views.md) is a slice of the cells of text.

```zaka
လုပ်ဆောင်ချက် စုစုပေါင်း သည်
ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း ကိန်းများ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
ကိန်းရှင် န သည် လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ကာ န သည် ကိန်းများ ၏ အကွက်အရေအတွက် အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ရလဒ် ကို ရလဒ် + (ကိန်းများ ၏ အကွက် န) ဟု ထားပါ။
ပြီးလျှင် န ကို န + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၇၀၊ ၈၅၊ ၉၀၊ ၆၀] ဖြစ်ပါစေ။
ကိန်းရှင် နှစ်ခု သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် စုစုပေါင်း ကို အမှတ်များ ၏ တန်ဖိုးအငှား တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
လုပ်ဆောင်ချက် စုစုပေါင်း ကို နှစ်ခု ၏ တန်ဖိုးအငှား တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၃၀၅
၃
```

The parameter kind is `ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း`, "a part of an integer list": a
list kind with no count, then `တစ်စိတ်တစ်ပိုင်း`. Inside the function, the cells and the
count read as for a list. `အမှတ်များ ၏ တန်ဖိုးအငှား` lends the whole list, as it lends
text. Messages write a slice kind with `[..]`, as `isize[..]`.

## Part of a list

`x ၏ အကွက် <start> မှစ၍ <length> ကွက်` is a slice of `length` cells, from cell `start`
on, counted from 0:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၀၊ ၂၀၊ ၃၀၊ ၄၀၊ ၅၀] ဖြစ်ပါစေ။
l ၏ အကွက် ၁ မှစ၍ ၃ ကွက် ကို ပြသပါ။
l ၏ အကွက် ၀ မှစ၍ ၂ ကွက် ကို ပြသပါ။
l ၏ အကွက် ၅ မှစ၍ ၀ ကွက် ကို ပြသပါ။    // no cells
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၂၀၊ ၃၀၊ ၄၀]
[၁၀၊ ၂၀]
[]
```

## Writing through a slice

A slice holds no cells of its own. A cell written through it is a cell of the list:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃၊ ၄] ဖြစ်ပါစေ။
ကိန်းရှင် အလယ် သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် l ၏ အကွက် ၁ မှစ၍ ၂ ကွက် ဖြစ်ပါစေ။
အလယ် ၏ အကွက် ၀ ကို ၀ ဟု ထားပါ။
l ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၀၊ ၃၊ ၄]
```

## Slice mistakes

A list does not turn into a slice on its own. Lend it, so the reader sees the borrow:

```zaka-error ZK0401
လုပ်ဆောင်ချက် ပြ သည်
ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း s ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
s ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ပြ ကို l တို့ဖြင့် လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0401]: expected isize[][..], found isize[]
  = help: hold the value in a variable and lend its cells with "<variable> ၏ တန်ဖိုးအငှား"
```

The fix lends the list: `လုပ်ဆောင်ချက် ပြ ကို l ၏ တန်ဖိုးအငှား တို့ဖြင့် လုပ်ဆောင်ပါ။`.

A range past the end of its list stops the program:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
l ၏ အကွက် ၂ မှစ၍ ၂ ကွက် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: 2 cells from cell 2 pass the end of 3 cells
```

A slice borrows its list, so the cells must stand still while the slice is in use. A write
into a list can move its cells to a block of its own, so it ends each slice of the list,
and a slice used after such a write is an error:

```zaka-error ZK0704
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် s သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် l ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
l ၏ အကွက် ၀ ကို ၉ ဟု ထားပါ။
s ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

Write through the slice, or take the slice after the write.

An [add or a remove](growing.md) ends each slice of the list too: an add can move the
cells to a larger block, and a remove takes the last cell off.

```zaka-error ZK0704
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် s သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် l ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
l ထဲသို့ ၄ ကို ထည့်ပါ။
s ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

The counted lists of the next page keep their cells in place through a write, so a slice
of one stands: see [Slices of counted
lists](../forms/counted-lists.md#slices-of-counted-lists). Part III takes slices
of heap blocks: see [Slices of the heap](../pointers/structs.md#slices-of-the-heap).
[Slices](../../manual/language/slices.md) in the Docs gives each rule.

## Hands on

- Write a function that takes a slice and prints its largest cell, and call it with a
  whole list and with the last two cells of it.
- Set each cell of a part of a list to 0 through a slice, and print the list.
- Take a slice of a list, add a cell to the list, then print the slice, and read the
  message.

Next: [Static data and counted lists](../forms/counted-lists.md).
