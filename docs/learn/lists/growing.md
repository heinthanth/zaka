---
title: Adding and removing cells
description: Add a cell at the end of a list or in front of a cell, and remove its last cell.
---

# Adding and removing cells

A plain list can grow and shrink a cell at a time. `ထည့်မည်` ("please
add") puts a value in a new last cell, and `ဖျက်မည်` ("please remove") takes the last
cell off:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။
ကိန်းများ ထဲသို့ ၃ ကို ထည့်မည်။             // a new last cell
ကိန်းများ ကို ပြသမည်။
ကိန်းများ ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။      // the last cell goes
ကိန်းများ ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။
ကိန်းများ ကို ပြသမည်။
ကိန်းများ ၏ အကွက်အရေအတွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၂၊ ၃]
[၁]
၁
```

| Zaka | Reads as |
| --- | --- |
| `ကိန်းများ ထဲသို့ ၃ ကို ထည့်မည်` | "please add 3 into the numbers" |
| `ကိန်းများ ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်` | "please remove the last cell of the numbers" |
| `ထဲသို့` | "into" |
| `နောက်ဆုံးအကွက်` | "the last cell" |

`နောက်ဆုံးအကွက်` names the last cell for a remove alone: it comes right after `၏`, and
`ကို ဖျက်မည်` follows it. To read the last cell, use its number, as in [Cell
numbers](README.md#cell-numbers).

## Adding in the middle

`၏ အရှေ့တွင်` ("in front of") after a cell puts the new value in front of it. The value
takes the number of that cell, and the cells from there on move one cell on:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၀၊ ၃၀] ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက် ၁ ၏ အရှေ့တွင် ၂၀ ကို ထည့်မည်။        // in front of ၃၀
ကိန်းများ ၏ ၁ ခုမြောက်အကွက် ၏ အရှေ့တွင် ၀ ကို ထည့်မည်။   // at the start
ကိန်းများ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၀၊ ၁၀၊ ၂၀၊ ၃၀]
```

| Zaka | Reads as |
| --- | --- |
| `ကိန်းများ ၏ အကွက် ၁ ၏ အရှေ့တွင် ၂၀ ကို ထည့်မည်` | "please add 20 in front of cell 1 of the numbers" |
| `အရှေ့တွင်` | "in front of" |

The number can also be the count, which puts the value at the end, as `ထဲသို့` does. A
larger number stops the program.

## Building a list

A loop can fill a list that starts with no cells, `[]`. `ကနဦးတန်ဖိုး` gives the
same list with no cells:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် နှစ်ထပ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [] ဖြစ်ပါစေ။
၁ မှ ၅ အထိ န အနေဖြင့် ရေတွက်၍ နှစ်ထပ်များ ထဲသို့ န * န ကို ထည့်မည်။
နှစ်ထပ်များ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၄၊ ၉၊ ၁၆၊ ၂၅]
```

The value has the kind of a cell, and it goes in as an assigned value does: text and lists
are duplicated, so the list holds a value of its own:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည်များ သည် စာသား စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် အမည် သည် "မောင်" ဖြစ်ပါစေ။
အမည်များ ထဲသို့ အမည် ကို ထည့်မည်။           // a duplicate of the text
အမည်များ ထဲသို့ "မြ" ကို ထည့်မည်။
အမည် ကို "ဇော်" ဟု ထားမည်။
အမည်များ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
["မောင်"၊ "မြ"]
```

The value is worked out before the list changes, so a value that reads the list reads it
as it was:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် လက္ခဏာမဲ့ ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၇] ဖြစ်ပါစေ။
ကိန်းများ ထဲသို့ ကိန်းများ ၏ အကွက်အရေအတွက် ကို ထည့်မည်။     // the count before the add: 1
ကိန်းများ ထဲသို့ ကိန်းများ ၏ အကွက်အရေအတွက် ကို ထည့်မည်။     // 2
ကိန်းများ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၇၊ ၁၊ ၂]
```

The list can be each place a value can be set in: a variable, a cell of another list, or
a field of a struct:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဂရစ် သည် ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [[၁]၊ [၂၊ ၃]] ဖြစ်ပါစေ။
ဂရစ် ၏ အကွက် ၀ ထဲသို့ ၅ ကို ထည့်မည်။        // row 0 grows
ဂရစ် ထဲသို့ [၆] ကို ထည့်မည်။                 // a new row
ဂရစ် ၏ အကွက် ၁ ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။   // row 1 shrinks
ဂရစ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[[၁၊ ၅]၊ [၂]၊ [၆]]
```

## Lists in functions

A list parameter is a duplicate of the list passed in, so a function that adds to it
leaves the caller's list as it was. To grow the caller's list, mark the parameter and the
argument as [the caller's variable](../functions/parameters.md#changing-the-callers-variable)
is marked:

```zaka
// the parameter is a duplicate of the list passed in
လုပ်ဆောင်ချက် ချဲ့ သည်
ကိန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းများ ထဲသို့ ၉ ကို ထည့်မည်။
ကိန်းများ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// the function can change the caller's list
လုပ်ဆောင်ချက် ထပ်ထည့် သည်
ကိန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းများ ထဲသို့ ၉ ကို ထည့်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မူရင်း သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။
ချဲ့ ဟူသော လုပ်ဆောင်ချက်ကို မူရင်း ဖြင့် လုပ်ဆောင်မည်။
မူရင်း ကို ပြသမည်။     // unchanged
ထပ်ထည့် ဟူသော လုပ်ဆောင်ချက်ကို မူရင်း ဖြင့် လုပ်ဆောင်မည်။
မူရင်း ကို ပြသမည်။     // the function changed it
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၂၊ ၉]
[၁၊ ၂]
[၁၊ ၂၊ ၉]
```

## Which lists grow

Text takes an add too, of text or a character, at its end or in front of a character, and
it loses its last character with `နောက်ဆုံးစာလုံး` (see [Adding in the middle and removing
the end](../text/changing.md#adding-in-the-middle-and-removing-the-end));
`နောက်ဆုံးအကွက်` on text is an error (see [List mistakes](mistakes.md#removing-from-text)). A remove from a
list with no cells left stops the program when it runs: see [List
mistakes](mistakes.md#removing-from-a-list-with-no-cells).

## Hands on

- Build a list of the even numbers from 2 to 20 with a counted loop and an add.
- Remove the last cells of a list in a while loop until one cell is left.
- Keep a list in order: add each new number in front of the first cell that is larger.
- Write a function that can change a list of text, and adds a name to it.

Next: [Passing and comparing lists](passing.md).
