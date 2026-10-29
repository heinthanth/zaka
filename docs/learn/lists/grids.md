---
title: Lists of lists
description: Build tables and grids from lists whose cells are lists, and walk them with addresses.
---

# Lists of lists

A cell can hold a whole list. A list of rows is a table:

```zaka
ကိန်းရှင် ဇယား သည် အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း ဖြစ်ပါစေ။   // 2 rows of 3
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
write this kind as `[2][3]isize`. Lists nest as deep as needed.

## Nested literals

Each inner list literal goes in parentheses. A kind with no counts takes all of them
from the value:

```zaka
ကိန်းရှင် ဇယား သည် ကိန်းပြည့် စာရင်း စာရင်း ဖြစ်၍ တန်ဖိုးသည် (၁၊ ၂ တို့ပါသော စာရင်း)၊ (၃၊ ၄ တို့ပါသော စာရင်း)၊ (၅၊ ၆ တို့ပါသော စာရင်း) တို့ပါသော စာရင်း ဖြစ်ပါစေ။
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

## Rows and cells through addresses

The address of a list points at the whole list, so its kind holds the count:
`ငှားထားသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ`, or `&*[3]isize` in messages. Adding 1 to it
moves one whole row:

```zaka
ကိန်းရှင် ဇယား သည် ကိန်းပြည့် စာရင်း စာရင်း ဖြစ်၍ တန်ဖိုးသည် (၁၊ ၂၊ ၃ တို့ပါသော စာရင်း)၊ (၄၊ ၅၊ ၆ တို့ပါသော စာရင်း) တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် အတန်း သည် ငှားထားသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ဇယား ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။
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

The address of one cell is a plain pointer, and [pointer
arithmetic](../pointers/arithmetic.md) walks the cells after it:

```zaka
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၂၊ ၃၊ ၅၊ ၇ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ကိန်းများ ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။   // the first cell
ပ ၏ တန်ဖိုး ကို ပြသပါ။
ပ + ၃ ၏ တန်ဖိုး ကို ပြသပါ။                 // the cell 3 steps further
```

```text
၂
၇
```

## The size of a list

A list takes its count times the size of one cell, with no gaps:

```zaka
အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း ၏ အရွယ်အစား ကို ပြသပါ။
```

```text
၄၈
```

## Hands on

- Fill a 3 x 3 table so that each cell holds its row number times its column number,
  and print it.
- Walk a row of a table with a cell pointer and add up its cells.

Next: [List mistakes](mistakes.md).
