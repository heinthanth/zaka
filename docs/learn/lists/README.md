---
title: Lists
description: Keep several values of one data kind in order, and reach each one through its cell number.
---

# Lists

A *list* (dynamic array) stores values of the same data kind (type) in order. For
example, we can keep the scores for a class in one list. Each score occupies a *cell*
(element), and its *cell number* (index) lets us read or update it.

## You will learn

- how to create a list and read or change one of its cells
- how to count cells from 0 or from 1, and how to get the number of cells
- how to write a list with no cells
- how to visit each cell with a loop

```zaka
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၀၊ ၀၊ ၀] ဖြစ်ပါစေ။ // three integers, each 0
အမှတ်များ ၏ အကွက် ၀ ကို ၇၀ ဟု ထားမည်။                                   // the first cell
အမှတ်များ ၏ အကွက် ၂ ကို ၉၀ ဟု ထားမည်။                                   // the last cell
အမှတ်များ ကို ပြသမည်။
အမှတ်များ ၏ အကွက် ၂ ကို ပြသမည်။
```

```text
[၇၀၊ ၀၊ ၉၀]
၉၀
```

The first line creates a list of three integers named
`အမှတ်များ` (scores). We can use this pattern to create a list, and `<list> ၏ အကွက်
<number>` to reach one of its cells:

```text
ကိန်းရှင် <name> သည် <kind> စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [<value>၊ <value>၊ <value>] ဖြစ်ပါစေ။
```

| Zaka | Reads as |
| --- | --- |
| `<kind> စာရင်း` | "a `<kind>` list"; `စာရင်း` is the word for a list |
| `တစ်ခု` | "one" |
| `[၀၊ ၀၊ ၀]` | "these three values", with `၊` between them |
| `အမှတ်များ ၏ အကွက် ၀` | "cell 0 of the scores"; `အကွက်` is the word for a cell |

We can read a cell like a variable, and we can set it with `ထားမည်`. When we print the
whole list, the output shows its cells between `[` and `]`, with `၊` between them.

## List literals

```text
[<value>၊ <value>၊ <value>]
<value>၊ <value> နှင့် <value> တို့ပါသော စာရင်း
```

A *list literal* such as `[၀၊ ၀၊ ၀]` gives the list one cell for each value inside
the brackets. To write the same list in words, put `နှင့်` before the last value and
`တို့ပါသော စာရင်း` ("the list that contains these") after it. A list prints with brackets
in both cases. Use `အကွက်အရေအတွက်` ("cell count") to find its length.

```zaka
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅ နှင့် ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက်အရေအတွက် ကို ပြသမည်။
အမှတ်များ ကို [၁၊ ၂] ဟု ထားမည်။
အမှတ်များ ကို ပြသမည်။
[[၁၊ ၂]၊ [၃၊ ၄]] ကို ပြသမည်။
```

```text
၃
[၁၊ ၂]
[[၁၊ ၂]၊ [၃၊ ၄]]
```

Assigning `[၁၊ ၂]` replaces the original three-cell list with a two-cell list. The
last line shows how a list literal can contain other lists. See [Lists of
lists](grids.md) for more examples.

## A list with no cells

```text
[]
စာရင်းအလွတ်
```

To write a list with no cells, we can use `[]` or `စာရင်းအလွတ်` ("blank list"), and
`ကနဦးတန်ဖိုး`, the [initial value](../variables/README.md#the-initial-value) (default
value) of a list, is a list with no cells too. To test whether a list has no cells, we
compare its cell count with 0:

```zaka
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
က ၏ အကွက်အရေအတွက် ကို ပြသမည်။
(က ၏ အကွက်အရေအတွက်) == ၀ ကို ပြသမည်။
က ကို [၇၊ ၈၊ ၉] ဟု ထားမည်။
က ကို ပြသမည်။
က ကို [] ဟု ထားမည်။
က ကို ပြသမည်။
```

```text
၀
မှန်
[၇၊ ၈၊ ၉]
[]
```

The cell count is a word phrase, so we put it in parentheses before an operator such as
`==`, as [Grouping](../operators/grouping.md) explains.

## Cell numbers

```text
<list> ၏ အကွက် <number>
<list> ၏ <number> ခုမြောက်အကွက်
```

The cell number starts at 0 after `အကွက်` and at 1 before `ခုမြောက်အကွက်`.
Both `အကွက် ၀` and `၁ ခုမြောက်အကွက်` select the first cell.

```zaka
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၇၀၊ ၈၅၊ ၉၀] ဖြစ်ပါစေ။
အမှတ်များ ၏ ၁ ခုမြောက်အကွက် ကို ပြသမည်။     // counts from 1: the first cell
အမှတ်များ ၏ အကွက် ၁ ကို ပြသမည်။             // counts from 0: the second cell
```

```text
၇၀
၈၅
```

| Written | Cell |
| --- | --- |
| `အမှတ်များ ၏ အကွက် ၀` | the first |
| `အမှတ်များ ၏ ၁ ခုမြောက်အကွက်` | the first |
| `အမှတ်များ ၏ အကွက် ၂` | the third |
| `အမှတ်များ ၏ ၃ ခုမြောက်အကွက်` | the third |

A list of three cells has the cell numbers 0, 1 and 2, or 1, 2 and 3 when we count from 1.
A cell number outside the list stops the program, as [List mistakes](mistakes.md) shows.

## List kinds

`ကိန်းပြည့် စာရင်း` is a *plain list*, which is the list we use most of the time: its count
comes with its value, and it can grow and shrink. Error messages write a list kind in a
short form, with the data kind of a cell followed with `[]`:

| Zaka | Short form |
| --- | --- |
| `ကိန်းပြည့် စာရင်း` | `isize[]` |
| `ကိန်းပြည့် စာရင်း စာရင်း` | `isize[][]` |

Part II adds the *counted list* (fixed-size array), whose count never changes and whose cells keep their place, in [Static data and counted lists](../forms/counted-lists.md).

## Lists of other data kinds

A list can contain values of each data kind, such as decimal numbers, true/false values or
characters. We can also put `၏` before `စာရင်း`, as in `မှန်မှားတန်ဖိုး ၏ စာရင်း`, which
means the same as `မှန်မှားတန်ဖိုး စာရင်း`:

```zaka
ကိန်းရှင် က သည် ဒဿမကိန်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁.၅၊ ၂.၀] ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် မှန်မှားတန်ဖိုး ၏ စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [မှန်၊ မမှန်၊ မှန်] ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် စာလုံး စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် 'က' နှင့် 'ခ' တို့ပါသော စာရင်း ဖြစ်ပါစေ။
က ကို ပြသမည်။
ခ ကို ပြသမည်။
ဂ ကို ပြသမည်။
```

```text
[၁.၅၊ ၂.၀]
[မှန်၊ မမှန်၊ မှန်]
[က၊ ခ]
```

## Cells in operator expressions

The cell number after `အကွက်` includes the whole operator expression that follows it, so
`ကိန်းများ ၏ အကွက် ၀ + ၁` means cell `၀ + ၁`, which is cell 1. To use the value of a cell in
a sum, we put the cell in parentheses:

```zaka
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀၊ ၂၀ နှင့် ၃၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက် ၀ + ၁ ကို ပြသမည်။          // the cell number is 0 + 1
(ကိန်းများ ၏ အကွက် ၀) + ၁ ကို ပြသမည်။            // the first cell, plus 1
```

```text
၂၀
၁၁
```

## Walking a list

```text
<list> ၏ အကွက်တိုင်း ကို <name> အနေဖြင့် ယူ၍ <sentence>
```

To visit each cell in turn, we can use an *each-cell loop* (for-each loop), which means
"taking each cell of the list as `<name>`, do this". Here the loop adds up the numbers:

```zaka
ကိန်းရှင် စုစုပေါင်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၄၊ ၈၊ ၁၅ နှင့် ၁၆ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက်တိုင်း ကို ကိန်း အနေဖြင့် ယူ၍ စုစုပေါင်း ကို စုစုပေါင်း + ကိန်း ဟု ထားမည်။
စုစုပေါင်း ကို ပြသမည်။
```

```text
၄၃
```

| Zaka | Reads as | Part |
| --- | --- | --- |
| `ကိန်းများ ၏` | "of the numbers" | the list |
| `အကွက်တိုင်း ကို` | "each cell" | what the loop visits |
| `ကိန်း အနေဖြင့်` | "as ကိန်း" | the name of the loop variable |
| `ယူ၍` | "taking" | the loop verb |

In each round, the loop variable has a duplicate (copy) of the next cell, so when the loop
changes the variable, the list does not change. A block can follow `ယူ၍` in place of a
single sentence:

```zaka
ကိန်းရှင် အမည်များ သည် စာသား စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ["မောင်"၊ "မြ"] ဖြစ်ပါစေ။
အမည်များ ၏ အကွက်တိုင်း ကို အမည် အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အမည် ကို "{အမည်}!" ဟု ထားမည်။
အမည် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အမည်များ ကို ပြသမည်။
```

```text
မောင်!
မြ!
["မောင်"၊ "မြ"]
```

To change the cells themselves, or to know the number of each cell, we use a [counted
loop](../control-flow/counted-loops.md) over the cell numbers instead. The last cell number
is the cell count minus 1:

```zaka
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၄၊ ၈၊ ၁၅ နှင့် ၁၆ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
၀ မှ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၁ အထိ န အနေဖြင့် ရေတွက်၍ ကိန်းများ ၏ အကွက် န ကို (ကိန်းများ ၏ အကွက် န) * ၂ ဟု ထားမည်။
ကိန်းများ ကို ပြသမည်။
```

```text
[၈၊ ၁၆၊ ၃၀၊ ၃၂]
```

## Details

- The cell count is a `လက္ခဏာမဲ့ ကိန်းပြည့်` (unsigned integer), so for a list with no
  cells, `အကွက်အရေအတွက် - ၁` stops the program ([A list with no cells and the last
  number](mistakes.md#a-list-with-no-cells-and-the-last-number)).
- `အကွက်အရေအတွက်` is one word, and with a space between its parts it means something else
  ([The cell count, written as two words](mistakes.md#the-cell-count-written-as-two-words)).
- A list declared with no value has nothing to read until the program sets it, the same as
  each other variable ([Set before read](../variables/README.md#set-before-read)).
- `စာရင်းအလွတ်` gets its data kind from the place where we write it, so it cannot stand
  alone ([A blank list with no data kind](mistakes.md#a-blank-list-with-no-data-kind)).
- The list module of the stdlib, `"áá¶áá¬á¸/áá¬áááºá¸"`, has functions to sort a list, to find a
  value in it and to remove a cell from the middle ([The stdlib](../stdlib/README.md#lists)).
- The full rules for literals, cells and list kinds are in
  [Lists](../../manual/language/lists.md).

## Recap

- We write `<kind> စာရင်း` for a list kind, and a list literal such as `[၁၊ ၂၊ ၃]` or
  `၁၊ ၂ နှင့် ၃ တို့ပါသော စာရင်း` for its cells.
- `<list> ၏ အကွက် <number>` counts from 0, and `<list> ၏ <number> ခုမြောက်အကွက်` counts
  from 1.
- `<list> ၏ အကွက်အရေအတွက်` gives the number of cells, and `[]` is a list with no cells.
- An each-cell loop visits a duplicate of each cell, and a counted loop over the cell
  numbers can change the cells.

## Hands on

- Declare a list of five decimal numbers, set the middle one, and print the list.
- Print the last cell of a list, once counting from 0 and once counting from 1.
- Find the largest value in a list with an each-cell loop.
- Double each cell of a list with a counted loop, and print the list.

Next: [Adding and removing cells](growing.md).
