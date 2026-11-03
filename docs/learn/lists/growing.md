---
title: Adding and removing cells
description: Add a cell at the end of a plain or heap list, and remove its last cell.
---

# Adding and removing cells

A plain list and a heap list can grow and shrink a cell at a time. `ထည့်ပါ` ("please
add") puts a value in a new last cell, and `ဖျက်ပါ` ("please remove") takes the last
cell off:

```zaka
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။
ကိန်းများ ထဲသို့ ၃ ကို ထည့်ပါ။             // a new last cell
ကိန်းများ ကို ပြသပါ။
ကိန်းများ ၏ နောက်ဆုံးအကွက်ကို ဖျက်ပါ။      // the last cell goes
ကိန်းများ ၏ နောက်ဆုံးအကွက်ကို ဖျက်ပါ။
ကိန်းများ ကို ပြသပါ။
ကိန်းများ ၏ အကွက်အရေအတွက် ကို ပြသပါ။
```

```text
[၁၊ ၂၊ ၃]
[၁]
၁
```

| Zaka | Reads as |
| --- | --- |
| `ကိန်းများ ထဲသို့ ၃ ကို ထည့်ပါ` | "please add 3 into the numbers" |
| `ကိန်းများ ၏ နောက်ဆုံးအကွက်ကို ဖျက်ပါ` | "please remove the last cell of the numbers" |
| `ထဲသို့` | "into" |
| `နောက်ဆုံးအကွက်` | "the last cell" |

`နောက်ဆုံးအကွက်` names the last cell for a remove alone: it comes right after `၏`, and
`ကို ဖျက်ပါ` follows it. To read the last cell, use its number, as in [Cell
numbers](README.md#cell-numbers).

## Building a list

A list declared with no value has no cells, and a loop can fill it:

```zaka
ကိန်းရှင် နှစ်ထပ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
၁ မှ ၅ အထိ န အနေဖြင့် ရေတွက်၍ နှစ်ထပ်များ ထဲသို့ န * န ကို ထည့်ပါ။
နှစ်ထပ်များ ကို ပြသပါ။
```

```text
[၁၊ ၄၊ ၉၊ ၁၆၊ ၂၅]
```

The value has the kind of a cell, and it goes in as an assigned value does: text and lists
are duplicated, so the list holds a value of its own. A heap list grows likewise:

```zaka
ကိန်းရှင် အမည်များ သည် စာသား စာရင်းရှင် တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် အမည် သည် "မောင်" ဖြစ်ပါစေ။
အမည်များ ထဲသို့ အမည် ကို ထည့်ပါ။           // a duplicate of the text
အမည်များ ထဲသို့ "မြ" ကို ထည့်ပါ။
အမည် ကို "ဇော်" ဟု ထားပါ။
အမည်များ ကို ပြသပါ။
```

```text
["မောင်"၊ "မြ"]
```

The value is worked out before the list changes, so a value that reads the list reads it
as it was:

```zaka
ကိန်းရှင် ကိန်းများ သည် လက္ခဏာမဲ့ ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၇] ဖြစ်ပါစေ။
ကိန်းများ ထဲသို့ ကိန်းများ ၏ အကွက်အရေအတွက် ကို ထည့်ပါ။     // the count before the add: 1
ကိန်းများ ထဲသို့ ကိန်းများ ၏ အကွက်အရေအတွက် ကို ထည့်ပါ။     // 2
ကိန်းများ ကို ပြသပါ။
```

```text
[၇၊ ၁၊ ၂]
```

The list can be each place a value can be set in: a variable, a cell of another list,
a field of a struct, or a list reached through its address:

```zaka
ကိန်းရှင် ဇယား သည် ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [[၁]၊ [၂၊ ၃]] ဖြစ်ပါစေ။
ဇယား ၏ အကွက် ၀ ထဲသို့ ၅ ကို ထည့်ပါ။        // row 0 grows
ဇယား ထဲသို့ [၆] ကို ထည့်ပါ။                 // a new row
ဇယား ၏ အကွက် ၁ ၏ နောက်ဆုံးအကွက်ကို ဖျက်ပါ။   // row 1 shrinks
ဇယား ကို ပြသပါ။
```

```text
[[၁၊ ၅]၊ [၂]၊ [၆]]
```

## Lists in functions

A list parameter is a duplicate of the list passed in, so a function that adds to it
leaves the caller's list as it was. To grow the caller's list, pass its address, and add
to `ကိန်းများ ၏ တန်ဖိုး`, the list the address points at:

```zaka
// the parameter is a duplicate of the list passed in
လုပ်ဆောင်ချက် ချဲ့ သည်
ကိန်းပြည့် စာရင်း ကိန်းများ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းများ ထဲသို့ ၉ ကို ထည့်ပါ။
ကိန်းများ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

// the parameter is the address of the caller's list
လုပ်ဆောင်ချက် ထပ်ထည့် သည်
ငှားထားသော ကိန်းပြည့် စာရင်း လိပ်စာ ကိန်းများ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းများ ၏ တန်ဖိုး ထဲသို့ ၉ ကို ထည့်ပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် မူရင်း သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ချဲ့ ကို မူရင်း တို့ဖြင့် လုပ်ဆောင်ပါ။
မူရင်း ကို ပြသပါ။     // unchanged
လုပ်ဆောင်ချက် ထပ်ထည့် ကို မူရင်း ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်ပါ။
မူရင်း ကို ပြသပါ။     // changed through the address
```

```text
[၁၊ ၂၊ ၉]
[၁၊ ၂]
[၁၊ ၂၊ ၉]
```

## Which lists grow

A fixed list keeps its count for good, so it neither grows nor shrinks; neither does
text, nor a slice. The compiler stops each of these: see [List
mistakes](mistakes.md#adding-to-a-fixed-list). A remove from a list with no cells left
stops the program when it runs: see [List mistakes](mistakes.md#removing-from-a-list-with-no-cells).

A slice or a cell address taken from a list cannot be used after the list grows or
shrinks, since an add can move the cells to a new block. See [Slices](slices.md#slice-mistakes).

## Hands on

- Build a list of the even numbers from 2 to 20 with a counted loop and an add.
- Remove the last cells of a list in a while loop until one cell is left.
- Write a function that takes the address of a list of text and adds a name to it.

Next: [Lists of lists](grids.md).
