---
title: Lists
description: Keep values of one kind next to each other, and reach each one through its number.
---

# Lists

A *list* holds values of one kind, next to each other in RAM. Each value sits in a
*cell*, and a cell is reached through its number.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၀၊ ၀၊ ၀] ဖြစ်ပါစေ။ // three integers, each 0
အမှတ်များ ၏ အကွက် ၀ ကို ၇၀ ဟု ထားပါ။                                   // the first cell
အမှတ်များ ၏ အကွက် ၂ ကို ၉၀ ဟု ထားပါ။                                   // the last cell
အမှတ်များ ကို ပြသပါ။
အမှတ်များ ၏ အကွက် ၂ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၇၀၊ ၀၊ ၉၀]
၉၀
```

| Zaka | Reads as |
| --- | --- |
| `ကိန်းပြည့် စာရင်း` | "an integer list" |
| `[၀၊ ၀၊ ၀]` | "the list of these three" |
| `အမှတ်များ ၏ အကွက် ၀` | "cell 0 of the scores" |
| `စာရင်း` | "list" |
| `အကွက်` | "cell" |

The value `[၀၊ ၀၊ ၀]` is a *list literal*: the cells, with `၊` between them, inside `[`
and `]`. It gives the list its cells, and so its count. The count of the list is
`အမှတ်များ ၏ အကွက်အရေအတွက်` ("the cell count of the scores"):

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက်အရေအတွက် ကို ပြသပါ။
အမှတ်များ ၏ ၁ ခုမြောက်အကွက် ကို ပြသပါ။     // counts from 1: the first cell
အမှတ်များ ၏ အကွက် ၁ ကို ပြသပါ။             // counts from 0: the second cell
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၃
၇၀
၈၅
```

The value `၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း` is the same literal in words: the cells, then
`တို့ပါသော စာရင်း` ("the list that holds these"). The two forms mean the same, and the
bracket form is the one a list prints in. A bracket list can hold other bracket lists as
written:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၇၀၊ ၈၅၊ ၉၀] ဖြစ်ပါစေ။
အမှတ်များ ကို ပြသပါ။
[[၁၊ ၂]၊ [၃၊ ၄]] ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၇၀၊ ၈၅၊ ၉၀]
[[၁၊ ၂]၊ [၃၊ ၄]]
```

A list declared with no value has no cells. `[]` and `စာရင်းအလွတ်` ("blank list") are
lists with no cells too:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
က ၏ အကွက်အရေအတွက် ကို ပြသပါ။
က ကို [၇၊ ၈၊ ၉] ဟု ထားပါ။
က ကို ပြသပါ။
က ကို [] ဟု ထားပါ။
က ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၀
[၇၊ ၈၊ ၉]
[]
```

Setting a list to another value gives it the cells of that value, and their count. The
count is known when the program runs. A plain list can also grow and shrink a cell at a
time: see [Adding and removing cells](growing.md).

## List kinds

`ကိန်းပြည့် စာရင်း` is a *plain list*, the list to reach for first: its count comes with
its value, and it can change. Messages write a list kind with a short name, the kind of a
cell and then `[]`:

| Kind | Short name |
| --- | --- |
| `ကိန်းပြည့် စာရင်း` | `isize[]` |
| `ကိန်းပြည့် စာရင်း စာရင်း` | `isize[][]` |

Part II shows where the cells of a list live, and adds the counted list, which keeps its
count for good and its cells in place. See [Static data and counted
lists](../forms/counted-lists.md).

## Cell numbers

`ကိန်းများ ၏ အကွက် ၀` counts from 0, as programs count. `ကိန်းများ ၏ ၁ ခုမြောက်အကွက်` counts from 1, as people count: "the first cell".

| Written | Cell |
| --- | --- |
| `အမှတ်များ ၏ အကွက် ၀` | the first |
| `အမှတ်များ ၏ ၁ ခုမြောက်အကွက်` | the first |
| `အမှတ်များ ၏ အကွက် ၂` | the third |
| `အမှတ်များ ၏ ၃ ခုမြောက်အကွက်` | the third |

`အမှတ်များ ၏ အကွက်အရေအတွက်` ("the cell count of the scores") gives the number of cells,
a `လက္ခဏာမဲ့ ကိန်းပြည့်`. A cell number outside the list stops the program: see [List
mistakes](mistakes.md).

## Lists of other kinds

Cells can hold each kind, and a list prints its cells between `[` and `]`. A `၏` before
`စာရင်း` is optional:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ဒဿမကိန်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁.၅၊ ၂.၀] ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် မှန်မှားတန်ဖိုး ၏ စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [မှန်၊ မမှန်၊ မှန်] ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် စာလုံး စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် 'က'၊ 'ခ' တို့ပါသော စာရင်း ဖြစ်ပါစေ။
က ကို ပြသပါ။
ခ ကို ပြသပါ။
ဂ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁.၅၊ ၂.၀]
[မှန်၊ မမှန်၊ မှန်]
[က၊ ခ]
```

## Cells in operator expressions

The number after `အကွက်` takes the whole operator expression that follows it, just as
`ပ + ၁ ၏ တန်ဖိုး` reads through `ပ + ၁`. To use a cell inside a sum, put the cell in
parentheses:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀၊ ၂၀၊ ၃၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက် ၀ + ၁ ကို ပြသပါ။          // the cell number is 0 + 1
(ကိန်းများ ၏ အကွက် ၀) + ၁ ကို ပြသပါ။            // the first cell, plus 1
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၀
၁၁
```

## Walking a list

An *each-cell loop* visits the cells in turn. It reads "taking each cell of the list as
*n*, do ...":

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စုစုပေါင်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၄၊ ၈၊ ၁၅၊ ၁၆ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက်တိုင်း ကို ကိန်း အနေဖြင့် ယူ၍ စုစုပေါင်း ကို စုစုပေါင်း + ကိန်း ဟု ထားပါ။
စုစုပေါင်း ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၃
```

| Zaka | Reads as | Part |
| --- | --- | --- |
| `ကိန်းများ ၏` | "of the numbers" | the list |
| `အကွက်တိုင်း ကို` | "each cell" | what the loop takes |
| `ကိန်း အနေဖြင့်` | "as ကိန်း" | the name of the loop variable |
| `ယူ၍` | "taking" | the loop verb |

Each round, the loop variable holds a duplicate of the next cell. The inner part can set
it, and the list keeps its cells:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည်များ သည် စာသား စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ["မောင်"၊ "မြ"] ဖြစ်ပါစေ။
အမည်များ ၏ အကွက်တိုင်း ကို အမည် အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
အမည် ကို "{အမည်}!" ဟု ထားပါ။
အမည် ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အမည်များ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မောင်!
မြ!
["မောင်"၊ "မြ"]
```

To change the cells themselves, or to know the number of each cell, walk the numbers with
a counted loop. The cell count, in parentheses, gives the last number:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စုစုပေါင်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၄၊ ၈၊ ၁၅၊ ၁၆ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
၀ မှ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၁ အထိ န အနေဖြင့် ရေတွက်၍ စုစုပေါင်း ကို စုစုပေါင်း + (ကိန်းများ ၏ အကွက် န) ဟု ထားပါ။
စုစုပေါင်း ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၃
```

## Hands on

- Declare a list of five decimal numbers, set the middle one, and print the list.
- Print the last cell of a list, once counting from 0 and once counting from 1.
- Find the largest value in a list with an each-cell loop.
- Double each cell of a list with a counted loop, and print the list.

Next: [Adding and removing cells](growing.md).
