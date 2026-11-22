---
title: Adding and removing cells
description: Add a cell at the end of a list or in front of a cell, and remove the last cell.
---

# Adding and removing cells

A plain list (dynamic array) can grow and shrink while the program runs. We can add a new
cell (element) at the end or in front of another cell, and we can remove the last cell, so
the number of cells changes as we go.

## You will learn

- how to add a value at the end of a list, and how to remove the last cell
- how to add a value in front of a cell
- how to build a list with a loop
- how to let a function add to the caller's list

```zaka
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။
ကိန်းများ ထဲသို့ ၃ ကို ထည့်မည်။ // a new last cell
ကိန်းများ ကို ပြသမည်။
ကိန်းများ ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။ // the last cell goes
ကိန်းများ ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။
ကိန်းများ ကို ပြသမည်။
ကိန်းများ ၏ အကွက်အရေအတွက် ကို ပြသမည်။
```

```text
[၁၊ ၂၊ ၃]
[၁]
၁
```

We use `ထည့်မည်` ("please add") to put a value in a new last cell, and `ဖျက်မည်` ("please
remove") to take the last cell off:

```text
<list> ထဲသို့ <value> ကို ထည့်မည်။
<list> ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။
```

| Zaka | Reads as |
| --- | --- |
| `ကိန်းများ ထဲသို့` | "into the numbers" |
| `၃ ကို ထည့်မည်` | "please add 3" |
| `ကိန်းများ ၏ နောက်ဆုံးအကွက်ကို` | "the last cell of the numbers" |
| `ဖျက်မည်` | "please remove" |

We can use `နောက်ဆုံးအကွက်` ("the last cell") just with `ဖျက်မည်`, right after `၏`. To
read the last cell, we use its number instead, as in [Cell numbers](README.md#cell-numbers).

## Adding in the middle

```text
<list> ၏ အကွက် <number> ၏ အရှေ့တွင် <value> ကို ထည့်မည်။
```

To add a value in front of a cell, we write `၏ အရှေ့တွင်` ("in front of") after that
cell. The new value gets the number of that cell, and each cell from there on moves along
one place:

```zaka
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၀၊ ၃၀] ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက် ၁ ၏ အရှေ့တွင် ၂၀ ကို ထည့်မည်။ // in front of ၃၀
ကိန်းများ ၏ ၁ ခုမြောက်အကွက် ၏ အရှေ့တွင် ၀ ကို ထည့်မည်။ // at the start
ကိန်းများ ကို ပြသမည်။
```

```text
[၀၊ ၁၀၊ ၂၀၊ ၃၀]
```

| Zaka | Reads as |
| --- | --- |
| `ကိန်းများ ၏ အကွက် ၁` | "cell 1 of the numbers" |
| `၏ အရှေ့တွင်` | "in front of" |
| `၂၀ ကို ထည့်မည်` | "please add 20" |

The cell number can also be the cell count, which puts the value at the end, the same as
`ထဲသို့`, while a larger number stops the program.

## Building a list

A program often starts with a list with no cells, `[]`, and adds the cells in a loop:

```zaka
ကိန်းရှင် နှစ်ထပ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [] ဖြစ်ပါစေ။
၁ မှ ၅ အထိ န အနေဖြင့် ရေတွက်၍ နှစ်ထပ်များ ထဲသို့ န * န ကို ထည့်မည်။
နှစ်ထပ်များ ကို ပြသမည်။
```

```text
[၁၊ ၄၊ ၉၊ ၁၆၊ ၂၅]
```

The value we add must have the data kind (type) of a cell, and it goes into the list the
same as with `ထားမည်`: text and lists are duplicated (copied), so a later change to the
variable does not reach the list. `ကနဦးတန်ဖိုး` is also a list with no cells:

```zaka
ကိန်းရှင် အမည်များ သည် စာသား စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် အမည် သည် "မောင်" ဖြစ်ပါစေ။
အမည်များ ထဲသို့ အမည် ကို ထည့်မည်။ // a duplicate of the text
အမည်များ ထဲသို့ "မြ" ကို ထည့်မည်။
အမည် ကို "ဇော်" ဟု ထားမည်။
အမည်များ ကို ပြသမည်။
```

```text
["မောင်"၊ "မြ"]
```

## Growing a list inside a list

We can add to each list that we can set, which includes a variable, a cell of another list
and a field of a struct. Here the rows of a list of lists grow and shrink one at a time:

```zaka
ကိန်းရှင် ဂရစ် သည် ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [[၁]၊ [၂၊ ၃]] ဖြစ်ပါစေ။
ဂရစ် ၏ အကွက် ၀ ထဲသို့ ၅ ကို ထည့်မည်။ // row 0 grows
ဂရစ် ထဲသို့ [၆] ကို ထည့်မည်။ // a new row
ဂရစ် ၏ အကွက် ၁ ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။ // row 1 shrinks
ဂရစ် ကို ပြသမည်။
```

```text
[[၁၊ ၅]၊ [၂]၊ [၆]]
```

## Lists in functions

```text
<name> ဟူသော <kind> စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
```

A list parameter gets a duplicate of the list we pass in, so when a function adds to it,
the caller's list does not change. To let a function grow the caller's list, we write
`ဒေတာ ပြောင်းလဲရန်` in the parameter to make it a mutable parameter (`inout`), just as for
[the caller's variable](../functions/parameters.md#changing-the-callers-variable):

```zaka
// the parameter is a duplicate of the list passed in
ချဲ့ ဟူသော လုပ်ဆောင်ချက် သည်
ကိန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းများ ထဲသို့ ၉ ကို ထည့်မည်။
ကိန်းများ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// the function can change the caller's list
ထပ်ထည့် ဟူသော လုပ်ဆောင်ချက် သည်
ကိန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းများ ထဲသို့ ၉ ကို ထည့်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် မူရင်း သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။
ချဲ့ ဟူသော လုပ်ဆောင်ချက်ကို မူရင်း ဖြင့် လုပ်ဆောင်မည်။
မူရင်း ကို ပြသမည်။ // unchanged
ထပ်ထည့် ဟူသော လုပ်ဆောင်ချက်ကို မူရင်း ဖြင့် လုပ်ဆောင်မည်။
မူရင်း ကို ပြသမည်။ // the function changed it
```

```text
[၁၊ ၂၊ ၉]
[၁၊ ၂]
[၁၊ ၂၊ ၉]
```

## Details

- The value is worked out before the list changes, so a value that reads the list sees it
  as it was before the add: `ကိန်းများ ထဲသို့ ကိန်းများ ၏ အကွက်အရေအတွက် ကို ထည့်မည်။` adds
  the count from before the add ([Adding and removing
  cells](../../manual/language/lists.md#adding-and-removing-cells)).
- Removing the last cell of a list with no cells stops the program ([Removing from a list
  with no cells](mistakes.md#removing-from-a-list-with-no-cells)).
- Text can grow too, with text or a character at its end or in front of a character, and we
  remove its last character with `နောက်ဆုံးစာလုံး` ([Adding in the middle and removing the
  end](../text/changing.md#adding-in-the-middle-and-removing-the-end)). `နောက်ဆုံးအကွက်` on
  text is an error ([Removing from text](mistakes.md#removing-from-text)).
- The counted lists of Part II never grow or shrink ([Adding to a counted
  list](../forms/counted-lists.md#adding-to-a-counted-list)).

## Recap

- `<list> ထဲသို့ <value> ကို ထည့်မည်။` adds a value as the new last cell.
- `<list> ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။` removes the last cell.
- `<list> ၏ အကွက် <number> ၏ အရှေ့တွင် <value> ကို ထည့်မည်။` adds a value in front of a
  cell.
- A function adds to the caller's list when its parameter has `ဒေတာ ပြောင်းလဲရန်`.

## Hands on

- Build a list of the even numbers from 2 to 20 with a counted loop and an add.
- Remove the last cells of a list in a while loop until one cell is left.
- Keep a list in order: add each new number in front of the first cell that is larger.
- Write a function that can change a list of text, and adds a name to it.

Next: [Passing and comparing lists](passing.md).
