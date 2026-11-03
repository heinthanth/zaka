---
title: Lists of lists
description: Build tables and grids from lists whose cells are lists, and walk them with addresses.
---

# Lists of lists

A cell can hold a whole list. A list of rows is a table:

```zaka
ကိန်းရှင် ဇယား သည် အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်ပါစေ။ // 2 rows of 3
ဇယား ၏ အကွက် ၁ ၏ အကွက် ၂ ကို ၉ ဟု ထားပါ။                                          // row 1, column 2
ဇယား ကို ပြသပါ။
ဇယား ၏ အကွက် ၁ ကို ပြသပါ။
ဇယား ၏ အကွက်အရေအတွက် ကို ပြသပါ။
ဇယား ၏ အကွက် ၀ ၏ အကွက်အရေအတွက် ကို ပြသပါ။
```

```text
[[၀၊ ၀၊ ၀]၊ [၀၊ ၀၊ ၉]]
[၀၊ ၀၊ ၉]
၂
၃
```

Each count prefix pairs with one `စာရင်း`, the inner ones first: in
`အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း`, the `၃` belongs to the
first `စာရင်း` (a row of 3 integers) and the `၂` to the second (2 such rows). Messages
write this kind as `isize[3][2]`. Lists nest as deep as needed.

## Nested literals

Each inner list literal goes in parentheses. `ကိန်းပြည့် စာရင်း စာရင်း`, with no counts, is
a plain list of plain lists: each row takes its cells from the value, and rows can hold
different counts.

```zaka
ကိန်းရှင် ဇယား သည် ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (၁၊ ၂ တို့ပါသော စာရင်း)၊ (၃၊ ၄ တို့ပါသော စာရင်း)၊ (၅၊ ၆ တို့ပါသော စာရင်း) တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ဇယား ကို ပြသပါ။
ဇယား ၏ ၃ ခုမြောက်အကွက် ၏ ၁ ခုမြောက်အကွက် ကို ပြသပါ။
ဇယား ၏ အကွက် ၀ ကို ဇယား ၏ အကွက် ၂ ဟု ထားပါ။          // a whole row is duplicated
ဇယား ကို ပြသပါ။
```

```text
[[၁၊ ၂]၊ [၃၊ ၄]၊ [၅၊ ၆]]
၅
[[၅၊ ၆]၊ [၃၊ ၄]၊ [၅၊ ၆]]
```

```zaka
ကိန်းရှင် ဇယား သည် ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [[၁၊ ၂၊ ၃]၊ [၄၊ ၅]] ဖြစ်ပါစေ။
ဇယား ကို ပြသပါ။
ဇယား ၏ အကွက် ၁ ၏ အကွက်အရေအတွက် ကို ပြသပါ။
```

```text
[[၁၊ ၂၊ ၃]၊ [၄၊ ၅]]
၂
```

## Rows and cells through addresses

The address of a fixed list points at the whole list, so its kind holds the count:
`ငှားထားသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ`, or `isize[3]&` in messages. The rows
of a table of fixed lists sit one after another, so adding 1 to the address of a row
moves one whole row. `ကိန်းပြည့် စာရင်းသေ စာရင်းသေ` makes such a table and takes both
counts from the value:

```zaka
ကိန်းရှင် ဇယား သည် ကိန်းပြည့် စာရင်းသေ စာရင်းသေ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (၁၊ ၂၊ ၃ တို့ပါသော စာရင်း)၊ (၄၊ ၅၊ ၆ တို့ပါသော စာရင်း) တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် အတန်း သည် ငှားထားသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဇယား ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။
အတန်း ၏ တန်ဖိုး ကို ပြသပါ။
အတန်း ကို အတန်း + ၁ ဟု ထားပါ။                 // one whole row further
အတန်း ၏ တန်ဖိုး ကို ပြသပါ။
အတန်း ၏ အကွက် ၂ ကို ပြသပါ။                     // a cell, through the row address
```

```text
[၁၊ ၂၊ ၃]
[၄၊ ၅၊ ၆]
၆
```

A plain list keeps each row in a heap block of its own, so the step from one row to the
next works on fixed rows alone. The address of one cell is a plain pointer, and [pointer
arithmetic](../pointers/arithmetic.md) walks the cells after it, in a list of each form:

```zaka
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၊ ၃၊ ၅၊ ၇ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကိန်းများ ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။ // the first cell
ပ ၏ တန်ဖိုး ကို ပြသပါ။
ပ + ၃ ၏ တန်ဖိုး ကို ပြသပါ။                 // the cell 3 steps further
```

```text
၂
၇
```

## The size of a list

A fixed list takes its count times the size of one cell, with no gaps. A plain list is
three words wherever its cells are: the address of the first cell, the count, and the
room of its heap block:

```zaka
အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း ၏ အရွယ်အစား ကို ပြသပါ။
ကိန်းပြည့် စာရင်း ၏ အရွယ်အစား ကို ပြသပါ။
```

```text
၄၈
၂၄
```

## Hands on

- Fill a 3 x 3 table so that each cell holds its row number times its column number,
  and print it.
- Walk a row of a table with a cell pointer and add up its cells.

Next: [Slices](slices.md).
