---
title: Lists
description: Keep values of one kind next to each other, and reach each one through its number.
---

# Lists

A *list* holds values of one kind, next to each other in RAM. Each value sits in a
*cell*, and a cell is reached through its number.

```zaka
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၀၊ ၀၊ ၀] ဖြစ်ပါစေ။ // three integers, each 0
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
| `ကိန်းပြည့် စာရင်း` | "an integer list" |
| `[၀၊ ၀၊ ၀]` | "the list of these three" |
| `အမှတ်များ ၏ အကွက် ၀` | "cell 0 of the scores" |
| `စာရင်း` | "list" |
| `အကွက်` | "cell" |

The value `[၀၊ ၀၊ ၀]` is a *list literal*: the cells, with `၊` between them, inside `[`
and `]`. It gives the list its cells, and so its count. The count of the list is
`အမှတ်များ ၏ အကွက်အရေအတွက်` ("the cell count of the scores"):

```zaka
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက်အရေအတွက် ကို ပြသပါ။
အမှတ်များ ၏ ၁ ခုမြောက်အကွက် ကို ပြသပါ။     // counts from 1: the first cell
အမှတ်များ ၏ အကွက် ၁ ကို ပြသပါ။             // counts from 0: the second cell
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
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၇၀၊ ၈၅၊ ၉၀] ဖြစ်ပါစေ။
အမှတ်များ ကို ပြသပါ။
[[၁၊ ၂]၊ [၃၊ ၄]] ကို ပြသပါ။
```

```text
[၇၀၊ ၈၅၊ ၉၀]
[[၁၊ ၂]၊ [၃၊ ၄]]
```

A list declared with no value has no cells. `[]` and `စာရင်းအလွတ်` ("blank list") are
lists with no cells too:

```zaka
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
က ၏ အကွက်အရေအတွက် ကို ပြသပါ။
က ကို [၇၊ ၈၊ ၉] ဟု ထားပါ။
က ကို ပြသပါ။
က ကို [] ဟု ထားပါ။
က ကို ပြသပါ။
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

`ကိန်းပြည့် စာရင်း` is a *plain list*, the list to reach for first. Two more forms of list
exist, and messages write each with a short name:

| Kind | Short name | Its cells |
| --- | --- | --- |
| `ကိန်းပြည့် စာရင်း` | `isize[]` | plain: their count comes with the value, and the cells sit in a heap block |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း` | `isize[3]` | fixed: 3 cells for good, kept in the list itself |
| `ကိန်းပြည့် စာရင်းသေ` | `isize[3]`, from its value | fixed, with the count its value gives |
| `ကိန်းပြည့် စာရင်းရှင်` | `isize[]~` | heap: as plain, and it owns its heap block from the start |

A *fixed list* keeps its count for good: the count is part of its kind, and it is known
before the program runs. The count comes first, then the kind of each cell, then
`စာရင်း`. The word `ခု` and a `၏` before `စာရင်း` are both optional, so these four kinds
are the same:

| Kind |
| --- |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း` |
| `အကွက် ၃ ရှိသော ကိန်းပြည့် စာရင်း` |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် ၏ စာရင်း` |
| `အကွက် ၃ ရှိသော ကိန်းပြည့် ၏ စာရင်း` |

The count is a whole number above zero, written as digits. A fixed list declared with no
value starts with each cell at its default: `၀` for a number, `မမှန်` for a true/false
value, `ဗလာ` for an address. `စာရင်းသေ` ("fixed list") in place of `စာရင်း` leaves the
count to the value:

```zaka
ကိန်းရှင် က သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် စာရင်းသေ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၇၊ ၈] ဖြစ်ပါစေ။
က ကို ပြသပါ။
ခ ကို ပြသပါ။
က ကို စာရင်းအလွတ် ဟု ထားပါ။
```

```text
[၀၊ ၀၊ ၀]
[၇၊ ၈]
```

On a fixed list, `စာရင်းအလွတ်` sets each cell back to its default, and the count remains.
`[]` is a list with no cells, so it does not fit a fixed list, save in its declaration,
where each cell starts at its default; see [List mistakes](mistakes.md).

A fixed list turns into a plain list on its own, as in `ခ` given to a plain list: its
cells are duplicated into a heap block. A plain list never turns into a fixed list on
its own, since nothing promises its count. A *heap list*, `စာရင်းရှင်` ("living list"),
works as a plain list does; plain and heap lists turn into each other.

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
ကိန်းရှင် က သည် အကွက် ၂ ခုရှိသော ဒဿမကိန်း စာရင်း တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အကွက် ၃ ရှိသော မှန်မှားတန်ဖိုး ၏ စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် စာရင်းအလွတ် ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် စာလုံး စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် 'က'၊ 'ခ' တို့ပါသော စာရင်း ဖြစ်ပါစေ။
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
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀၊ ၂၀၊ ၃၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက် ၀ + ၁ ကို ပြသပါ။          // the cell number is 0 + 1
(ကိန်းများ ၏ အကွက် ၀) + ၁ ကို ပြသပါ။            // the first cell, plus 1
```

```text
၂၀
၁၁
```

## Walking a list

An *each-cell loop* visits the cells in turn. It reads "taking each cell of the list as
*n*, do ...":

```zaka
ကိန်းရှင် စုစုပေါင်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၄၊ ၈၊ ၁၅၊ ၁၆ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက်တိုင်း ကို ကိန်း အနေဖြင့် ယူ၍ စုစုပေါင်း ကို စုစုပေါင်း + ကိန်း ဟု ထားပါ။
စုစုပေါင်း ကို ပြသပါ။
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
ကိန်းရှင် အမည်များ သည် စာသား စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ["မောင်"၊ "မြ"] ဖြစ်ပါစေ။
အမည်များ ၏ အကွက်တိုင်း ကို အမည် အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
အမည် ကို "{အမည်}!" ဟု ထားပါ။
အမည် ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
အမည်များ ကို ပြသပါ။
```

```text
မောင်!
မြ!
["မောင်"၊ "မြ"]
```

To change the cells themselves, or to know the number of each cell, walk the numbers with
a counted loop. The cell count, in parentheses, gives the last number:

```zaka
ကိန်းရှင် စုစုပေါင်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၄၊ ၈၊ ၁၅၊ ၁၆ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
၀ မှ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၁ အထိ န အနေဖြင့် ရေတွက်၍ စုစုပေါင်း ကို စုစုပေါင်း + (ကိန်းများ ၏ အကွက် န) ဟု ထားပါ။
စုစုပေါင်း ကို ပြသပါ။
```

```text
၄၃
```

## Hands on

- Declare a list of five decimal numbers, set the middle one, and print the list.
- Print the last cell of a list, once counting from 0 and once counting from 1.
- Find the largest value in a list with an each-cell loop.
- Double each cell of a list with a counted loop, and print the list.

Next: [Passing and comparing lists](passing.md).
