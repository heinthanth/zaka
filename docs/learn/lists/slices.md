---
title: Slices
description: A slice borrows a run of cells with their count, so one function takes lists of each length and parts of them.
---

# Slices

A list kind holds its count, so a function that takes `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း`
takes three cells and never four. A *slice* takes cells of each count. It borrows a run
of cells from a list or a heap block and keeps their count:

```zaka
လုပ်ဆောင်ချက် စုစုပေါင်း သည်
ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း ကိန်းများ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
ကိန်းရှင် န သည် လက္ခဏာမဲ့ ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ကာ န သည် ကိန်းများ ၏ အကွက်အရေအတွက် အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ရလဒ် ကို ရလဒ် + (ကိန်းများ ၏ အကွက် န) ဟု ထားပါ။
ပြီးလျှင် န ကို န + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် [၇၀၊ ၈၅၊ ၉၀၊ ၆၀] ဖြစ်ပါစေ။
ကိန်းရှင် နှစ်ခု သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် စုစုပေါင်း ကို အမှတ်များ ၏ တန်ဖိုးအငှား တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
လုပ်ဆောင်ချက် စုစုပေါင်း ကို နှစ်ခု ၏ တန်ဖိုးအငှား တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
```

```text
၃၀၅
၃
```

The parameter kind is `ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း`, "a part of an integer list": a
list kind with no count, then `တစ်စိတ်တစ်ပိုင်း`. Inside the function, the cells and the
count read as for a list. `အမှတ်များ ၏ တန်ဖိုးအငှား` lends the whole list, as
`ပ ၏ တန်ဖိုးအငှား` lends a heap block.

## Part of a list

`x ၏ အကွက် <start> မှစ၍ <length> ကွက်` is a slice of `length` cells, from cell `start`
on, counted from 0:

```zaka
ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် [၁၀၊ ၂၀၊ ၃၀၊ ၄၀၊ ၅၀] ဖြစ်ပါစေ။
l ၏ အကွက် ၁ မှစ၍ ၃ ကွက် ကို ပြသပါ။
l ၏ အကွက် ၀ မှစ၍ ၂ ကွက် ကို ပြသပါ။
l ၏ အကွက် ၅ မှစ၍ ၀ ကွက် ကို ပြသပါ။    // no cells
```

```text
[၂၀၊ ၃၀၊ ၄၀]
[၁၀၊ ၂၀]
[]
```

## Writing through a slice

A slice holds no cells of its own. A cell written through it is a cell of the list:

```zaka
ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃၊ ၄] ဖြစ်ပါစေ။
ကိန်းရှင် အလယ် သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း ဖြစ်၍ တန်ဖိုးသည် l ၏ အကွက် ၁ မှစ၍ ၂ ကွက် ဖြစ်ပါစေ။
အလယ် ၏ အကွက် ၀ ကို ၀ ဟု ထားပါ။
l ကို ပြသပါ။
```

```text
[၁၊ ၀၊ ၃၊ ၄]
```

## Slices of the heap

A heap block has no count in its kind. A range writes the count once, and the slice keeps
it from then on:

```zaka
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၃ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း ဖြစ်၍ တန်ဖိုးသည် ပ ၏ အကွက် ၀ မှစ၍ ၃ ကွက် ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက် ၂ ကို ၉ ဟု ထားပါ။
ကိန်းများ ကို ပြသပါ။
ကိန်းများ ၏ အကွက်အရေအတွက် ကို ပြသပါ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
[၀၊ ၀၊ ၉]
၃
```

## Slice mistakes

A list does not turn into a slice on its own. Lend it, so the reader sees the borrow:

```zaka-error ZK0401
လုပ်ဆောင်ချက် ပြ သည်
ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း s ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
s ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ပြ ကို l တို့ဖြင့် လုပ်ဆောင်ပါ။
```

A range past the end of its list stops the program, or the compiler when it can see the
numbers:

```zaka-error ZK0605
ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
l ၏ အကွက် ၂ မှစ၍ ၂ ကွက် ကို ပြသပါ။
```

A slice borrows, so its block must live until the last use of the slice:

```zaka-error ZK0704
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် s သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း ဖြစ်၍ တန်ဖိုးသည် ပ ၏ အကွက် ၀ မှစ၍ ၂ ကွက် ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
s ကို ပြသပါ။
```

[Slices](../../manual/language/slices.md) in the Docs gives each rule.

Next: [List mistakes](mistakes.md).
