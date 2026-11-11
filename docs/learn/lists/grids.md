---
title: Lists of lists
description: Build tables and grids from lists whose cells are lists.
---

# Lists of lists

A cell can hold a whole list. A list of rows is a table:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဇယား သည် ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [[၀၊ ၀၊ ၀]၊ [၀၊ ၀၊ ၀]] ဖြစ်ပါစေ။ // 2 rows of 3
ဇယား ၏ အကွက် ၁ ၏ အကွက် ၂ ကို ၉ ဟု ထားမည်။                                          // row 1, column 2
ဇယား ကို ပြသမည်။
ဇယား ၏ အကွက် ၁ ကို ပြသမည်။
ဇယား ၏ အကွက်အရေအတွက် ကို ပြသမည်။
ဇယား ၏ အကွက် ၀ ၏ အကွက်အရေအတွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[[၀၊ ၀၊ ၀]၊ [၀၊ ၀၊ ၉]]
[၀၊ ၀၊ ၉]
၂
၃
```

`ကိန်းပြည့် စာရင်း စာရင်း` is a list whose cells are integer lists: the first `စာရင်း`
makes a row of integers, and the second a list of such rows. Messages write this kind as
`isize[][]`. Lists nest as deep as needed.

## Nested literals

In the word form, each inner list literal goes in parentheses. Each row takes its cells
from the value, so rows can hold different counts.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဇယား သည် ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (၁၊ ၂ တို့ပါသော စာရင်း)၊ (၃၊ ၄ တို့ပါသော စာရင်း)၊ (၅၊ ၆ တို့ပါသော စာရင်း) တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ဇယား ကို ပြသမည်။
ဇယား ၏ ၃ ခုမြောက်အကွက် ၏ ၁ ခုမြောက်အကွက် ကို ပြသမည်။
ဇယား ၏ အကွက် ၀ ကို ဇယား ၏ အကွက် ၂ ဟု ထားမည်။          // a whole row is duplicated
ဇယား ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[[၁၊ ၂]၊ [၃၊ ၄]၊ [၅၊ ၆]]
၅
[[၅၊ ၆]၊ [၃၊ ၄]၊ [၅၊ ၆]]
```

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဇယား သည် ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [[၁၊ ၂၊ ၃]၊ [၄၊ ၅]] ဖြစ်ပါစေ။
ဇယား ကို ပြသမည်။
ဇယား ၏ အကွက် ၁ ၏ အကွက်အရေအတွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[[၁၊ ၂၊ ၃]၊ [၄၊ ၅]]
၂
```

## The size of a list

`အရွယ်အစား` gives the size of a kind, in octets. A list is three words, whatever its
count: where its cells are, the count, and the room it has for more. A list of lists is
three words too, since each row keeps its cells apart:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းပြည့် စာရင်း တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၄
၂၄
```

Part II adds counted lists, which keep their cells in place: see [Static data and counted
lists](../forms/counted-lists.md#the-size-of-a-counted-list).

## Hands on

- Fill a 3 x 3 table so that each cell holds its row number times its column number,
  and print it.
- Walk a row of a table with an each-cell loop and add up its cells.

Next: [List mistakes](mistakes.md).
