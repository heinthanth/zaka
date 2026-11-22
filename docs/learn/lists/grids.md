---
title: Lists of lists
description: Build grids from lists whose cells are lists.
---

# Lists of lists

A cell (element) can hold another list, so we can store a grid as a list of rows. To
access a value, we use two cell numbers. The first selects the row and the second
selects the column within that row.

## You will learn

- how to write the data kind of a list of lists
- how to read and set a cell inside a row
- how to write a list of lists as a literal
- how much storage a list uses

```zaka
ကိန်းရှင် ဂရစ် သည် ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [[၀၊ ၀၊ ၀]၊ [၀၊ ၀၊ ၀]] ဖြစ်ပါစေ။ // 2 rows of 3
ဂရစ် ၏ အကွက် ၁ ၏ အကွက် ၂ ကို ၉ ဟု ထားမည်။ // row 1, column 2
ဂရစ် ကို ပြသမည်။
ဂရစ် ၏ အကွက် ၁ ကို ပြသမည်။
ဂရစ် ၏ အကွက်အရေအတွက် ကို ပြသမည်။
ဂရစ် ၏ အကွက် ၀ ၏ အကွက်အရေအတွက် ကို ပြသမည်။
```

```text
[[၀၊ ၀၊ ၀]၊ [၀၊ ၀၊ ၉]]
[၀၊ ၀၊ ၉]
၂
၃
```

We write `စာရင်း` twice to get a list of lists, and we add one more `၏ အကွက်` to reach a
cell inside a row:

```text
<kind> စာရင်း စာရင်း
<list> ၏ အကွက် <row> ၏ အကွက် <column>
```

| Zaka | Reads as |
| --- | --- |
| `ကိန်းပြည့် စာရင်း` | "an integer list", which is one row |
| `ကိန်းပြည့် စာရင်း စာရင်း` | "a list of integer lists", the rows together |
| `ဂရစ် ၏ အကွက် ၁` | "cell 1 of the grid", which is a whole row |
| `ဂရစ် ၏ အကွက် ၁ ၏ အကွက် ၂` | "cell 2 of cell 1 of the grid" |

Error messages write this data kind (type) as `isize[][]`. Lists can nest as deep as we
need, so `ကိန်းပြည့် စာရင်း စာရင်း စာရင်း` is a list of grids.

## Nested literals

```text
(<value> နှင့် <value> တို့ပါသော စာရင်း)၊ (…) နှင့် (…) တို့ပါသော စာရင်း
```

When we write a list of lists in words, we put each inner list literal in parentheses.
Each row gets its cells from the literal, and when we set one row to another, the whole row
is duplicated (copied):

```zaka
ကိန်းရှင် ဂရစ် သည် ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည်
(၁ နှင့် ၂ တို့ပါသော စာရင်း)၊
(၃ နှင့် ၄ တို့ပါသော စာရင်း) နှင့် (၅ နှင့် ၆ တို့ပါသော စာရင်း) တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ဂရစ် ကို ပြသမည်။
ဂရစ် ၏ ၃ ခုမြောက်အကွက် ၏ ၁ ခုမြောက်အကွက် ကို ပြသမည်။
ဂရစ် ၏ အကွက် ၀ ကို ဂရစ် ၏ အကွက် ၂ ဟု ထားမည်။ // a whole row is duplicated
ဂရစ် ကို ပြသမည်။
```

```text
[[၁၊ ၂]၊ [၃၊ ၄]၊ [၅၊ ၆]]
၅
[[၅၊ ၆]၊ [၃၊ ၄]၊ [၅၊ ၆]]
```

## Rows of different lengths

Each row is a separate list, so the rows do not need to have the same number of cells:

```zaka
ကိန်းရှင် ဂရစ် သည် ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [[၁၊ ၂၊ ၃]၊ [၄၊ ၅]] ဖြစ်ပါစေ။
ဂရစ် ကို ပြသမည်။
ဂရစ် ၏ အကွက် ၁ ၏ အကွက်အရေအတွက် ကို ပြသမည်။
```

```text
[[၁၊ ၂၊ ၃]၊ [၄၊ ၅]]
၂
```

## The size of a list

```text
<kind> တစ်ခု ၏ အရွယ်အစား
```

We can use `အရွယ်အစား` to get the size of a data kind in octets (bytes). A list uses three
words of storage, whatever its count: one for where its cells are, one for the count, and
one for the room it has for more cells. A list of lists also uses three words, because the
cells of each row are kept elsewhere:

```zaka
ကိန်းပြည့် စာရင်း တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
```

```text
၂၄
၂၄
```

## Details

- A row can grow and shrink like each other list, with `ထည့်မည်` and `ဖျက်မည်` ([Growing a
  list inside a list](growing.md#growing-a-list-inside-a-list)).
- In the bracket form, inner lists need no parentheses: `[[၁၊ ၂]၊ [၃၊ ၄]]`.
- Part II adds counted lists, which keep their cells in place, so the size of a counted
  list is its count times the size of one cell ([The size of a counted
  list](../forms/counted-lists.md#the-size-of-a-counted-list)).
- The full rules are in [Lists of lists](../../manual/language/lists.md#lists-of-lists).

## Recap

- `<kind> စာရင်း စာရင်း` is a list whose cells are lists, and each row can have its own
  number of cells.
- `<list> ၏ အကွက် <row> ၏ အကွက် <column>` reaches one value of a grid.
- In the word form, each inner list literal goes in parentheses.
- A plain list uses three words of storage, whatever its count.

## Hands on

- Fill a 3 x 3 grid so that each cell has its row number times its column number, and
  print it.
- Walk a row of a grid with an each-cell loop and add up its cells.

Next: [List mistakes](mistakes.md).
