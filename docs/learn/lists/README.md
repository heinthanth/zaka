---
title: Lists
description: Keep a fixed number of values of one kind next to each other, and reach each one through its number.
---

# Lists

A *list* holds a fixed number of values of one kind, next to each other in RAM. Each value
sits in a *cell*, and a cell is reached through its number.

```zaka
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း ဖြစ်ပါစေ။     // three integers, each 0
အမှတ်များ ၏ အကွက် ၀ ကို ၇၀ ဟု ထားပါ။                                   // the first cell
အမှတ်များ ၏ အကွက် ၂ ကို ၉၀ ဟု ထားပါ။                                   // the last cell
အမှတ်များ ကို ပြသပါ။
အမှတ်များ ၏ အကွက် ၂ ကို ပြသပါ။
```

```text
[၇၀၊ ၀၊ ၉၀]
၉၀
```

| Zaka | Reads as |
| --- | --- |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း` | "an integer list that has three cells" |
| `အမှတ်များ ၏ အကွက် ၀` | "cell 0 of the scores" |
| `စာရင်း` | "list" |
| `အကွက်` | "cell" |

A list declared with no value starts with each cell at its default: `၀` for a number,
`မမှန်` for a true/false value, `ဗလာ` for an address. `စာရင်းအလွတ်` ("blank list"), or
`[]`, writes the same start out in full.

## List kinds

The count comes first, then the kind of each cell, then `စာရင်း`. The word `ခု` and a
`၏` before `စာရင်း` are both optional, so these four kinds are the same:

| Kind |
| --- |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း` |
| `အကွက် ၃ ရှိသော ကိန်းပြည့် စာရင်း` |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် ၏ စာရင်း` |
| `အကွက် ၃ ရှိသော ကိန်းပြည့် ၏ စာရင်း` |

Messages write this kind as `[3]isize`. The count is a whole number above zero, written
as digits: the size of a list is fixed before the program runs.

When a declaration gives the list a value, the count can be left out. The value then
tells it:

```zaka
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက်အရေအတွက် ကို ပြသပါ။
အမှတ်များ ၏ ၁ ခုမြောက်အကွက် ကို ပြသပါ။     // counts from 1: the first cell
အမှတ်များ ၏ အကွက် ၁ ကို ပြသပါ။             // counts from 0: the second cell
```

```text
၃
၇၀
၈၅
```

The value `၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း` is a *list literal*: the cells, with `၊` between them,
then `တို့ပါသော စာရင်း` ("the list that holds these").

The same list can also be written in brackets, `[၇၀၊ ၈၅၊ ၉၀]`, the form a list prints in.
The two forms mean the same. A bracket list can hold other bracket lists as written:

```zaka
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် [၇၀၊ ၈၅၊ ၉၀] ဖြစ်ပါစေ။
အမှတ်များ ကို ပြသပါ။
[[၁၊ ၂]၊ [၃၊ ၄]] ကို ပြသပါ။
```

```text
[၇၀၊ ၈၅၊ ၉၀]
[[၁၊ ၂]၊ [၃၊ ၄]]
```

`[]` is not a list with no cells. A list kind fixes its count, so `[]`, like
`စာရင်းအလွတ်`, fills each cell of that count with its default:

```zaka
ကိန်းရှင် က သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် [၇၊ ၈၊ ၉] ဖြစ်ပါစေ။
က ကို [] ဟု ထားပါ။
က ကို ပြသပါ။
က ၏ အကွက်အရေအတွက် ကို ပြသပါ။
```

```text
[၀၊ ၀၊ ၀]
၃
```

Setting `က` to `[]` resets each cell to `၀`; the count is still 3. With no list kind from
its place, as in `[] ကို ပြသပါ။`, `[]` has no count to fill and is an error.

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

Cells can hold each kind, and a list prints its cells between `[` and `]`:

```zaka
ကိန်းရှင် က သည် အကွက် ၂ ခုရှိသော ဒဿမကိန်း စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အကွက် ၃ ရှိသော မှန်မှားတန်ဖိုး ၏ စာရင်း ဖြစ်၍ တန်ဖိုးသည် စာရင်းအလွတ် ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် စာလုံး စာရင်း ဖြစ်၍ တန်ဖိုးသည် 'က'၊ 'ခ' တို့ပါသော စာရင်း ဖြစ်ပါစေ။
က ကို ပြသပါ။
ခ ကို ပြသပါ။
ဂ ကို ပြသပါ။
```

```text
[၀.၀၊ ၀.၀]
[မမှန်၊ မမှန်၊ မမှန်]
[က၊ ခ]
```

## Cells in operator expressions

The number after `အကွက်` takes the whole operator expression that follows it, just as
`ပ + ၁ ၏ တန်ဖိုး` reads through `ပ + ၁`. To use a cell inside a sum, put the cell in
parentheses:

```zaka
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၁၀၊ ၂၀၊ ၃၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက် ၀ + ၁ ကို ပြသပါ။          // the cell number is 0 + 1
(ကိန်းများ ၏ အကွက် ၀) + ၁ ကို ပြသပါ။            // the first cell, plus 1
```

```text
၂၀
၁၁
```

## Walking a list

A counted loop visits each cell. The cell count, in parentheses, gives the last number:

```zaka
ကိန်းရှင် စုစုပေါင်း သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၄၊ ၈၊ ၁၅၊ ၁၆ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
၀ မှ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၁ အထိ န အနေဖြင့် ရေတွက်၍ စုစုပေါင်း ကို စုစုပေါင်း + (ကိန်းများ ၏ အကွက် န) ဟု ထားပါ။
စုစုပေါင်း ကို ပြသပါ။
```

```text
၄၃
```

## Hands on

- Declare a list of five decimal numbers, set the middle one, and print the list.
- Print the last cell of a list, once counting from 0 and once counting from 1.
- Find the largest value in a list with a counted loop.

Next: [Passing and comparing lists](passing.md).
