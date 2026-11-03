---
title: Slices
description: A slice borrows a run of cells with their count, so one function takes lists of each length and parts of them.
---

# Slices

A function that takes `ကိန်းပြည့် စာရင်း` takes plain lists of each count, but each time a
whole list, and a fixed list given to it is duplicated into a heap block first. A *slice*
borrows a run of cells and keeps their count: a whole list of each form, a part of one,
or a heap block:

```zaka
လုပ်ဆောင်ချက် စုစုပေါင်း သည်
ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း ကိန်းများ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
ကိန်းရှင် န သည် လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ကာ န သည် ကိန်းများ ၏ အကွက်အရေအတွက် အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ရလဒ် ကို ရလဒ် + (ကိန်းများ ၏ အကွက် န) ဟု ထားပါ။
ပြီးလျှင် န ကို န + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၇၀၊ ၈၅၊ ၉၀၊ ၆၀] ဖြစ်ပါစေ။
ကိန်းရှင် နှစ်ခု သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။
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
ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၀၊ ၂၀၊ ၃၀၊ ၄၀၊ ၅၀] ဖြစ်ပါစေ။
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
ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃၊ ၄] ဖြစ်ပါစေ။
ကိန်းရှင် အလယ် သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် l ၏ အကွက် ၁ မှစ၍ ၂ ကွက် ဖြစ်ပါစေ။
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
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၃ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ပ ၏ အကွက် ၀ မှစ၍ ၃ ကွက် ဖြစ်ပါစေ။
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

ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ပြ ကို l တို့ဖြင့် လုပ်ဆောင်ပါ။
```

A range past the end of its list stops the program. On a fixed list, whose count the
compiler knows, a range of constant numbers is an error:

```zaka-error ZK0605
ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်းသေ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
l ၏ အကွက် ၂ မှစ၍ ၂ ကွက် ကို ပြသပါ။
```

A slice borrows, so its block must live until the last use of the slice:

```zaka-error ZK0704
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် s သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ပ ၏ အကွက် ၀ မှစ၍ ၂ ကွက် ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
s ကို ပြသပါ။
```

A write into a plain list can move its cells to a block of its own, so it ends each
slice of the list. A slice used after such a write is an error; a slice of a heap list
or a fixed list stands:

```zaka-error ZK0704
ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် s သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် l ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
l ၏ အကွက် ၀ ကို ၉ ဟု ထားပါ။
s ကို ပြသပါ။
```

Write through the slice, or take the slice after the write.

An [add or a remove](growing.md) ends each slice of the list too, a heap list included:
an add can move the cells to a larger block, and a remove takes the last cell off.

```zaka-error ZK0704
ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်းရှင် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် s သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် l ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
l ထဲသို့ ၄ ကို ထည့်ပါ။
s ကို ပြသပါ။
```

[Slices](../../manual/language/slices.md) in the Docs gives each rule.

Next: [List mistakes](mistakes.md).
