---
title: Lists
description: The fixed-size list kinds, literals, cell numbers, lists of lists, passing and comparing, how a list is stored, and the mistakes with cell numbers.
---

# Lists

A list holds a fixed number of values of one kind, next to each other in RAM. Each value sits in
a *cell*, and a program reaches a cell through its number. The count of cells is part of
the kind and is known before the program runs; a list never grows or shrinks. For a list
that changes size, use a block of the [heap](pointers.md#heap-statements). The Learn book
starts in small steps with [Lists](../../learn/lists/README.md).

```zaka
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက် ၀ ကို ၇၀ ဟု ထားပါ။
အမှတ်များ ၏ အကွက် ၂ ကို ၉၀ ဟု ထားပါ။
အမှတ်များ ကို ပြသပါ။
```

```text
[၇၀၊ ၀၊ ၉၀]
```

## List kinds

```text
အကွက် <count> ခုရှိသော <cell kind> စာရင်း
```

The count comes first, then the kind of each cell, then `စာရင်း` ("list"). The word `ခု`
and a `၏` in front of `စာရင်း` are optional, so these four kinds are the same:

| Kind |
| --- |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း` |
| `အကွက် ၃ ရှိသော ကိန်းပြည့် စာရင်း` |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် ၏ စာရင်း` |
| `အကွက် ၃ ရှိသော ကိန်းပြည့် ၏ စာရင်း` |

- The count is a whole number above zero written with digits (ZK0227). It cannot come from
  a variable, because the size of a list is fixed before the program runs.
- The cell kind is a kind of each sort: a number, a true/false value, a character, a pointer, or a
  list. Messages write the kind as `[3]isize`.
- A declaration that gives the list a value can leave the count out, and the value tells
  it. In each other place a missing count is ZK0421.
- A list takes its default from its cells: `၀` for a number, `မမှန်` for a true/false
  value, `ဗလာ` for an address. A list declared with no value starts with all cells at the
  default.

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

## Literals

A list value is written with its cells between `၊` and a closing `တို့ပါသော စာရင်း` ("the
list that holds these"). `စာရင်းအလွတ်` ("blank list") is a list of the expected kind with
each cell at its default.

```text
၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း
စာရင်းအလွတ်
```

- The number of cells in a literal must equal the count of the kind it fills (ZK0419).
- `စာရင်းအလွတ်` needs a list kind from the place it stands in. Standing alone it has none
  (ZK0420).
- A literal that is a cell of another literal goes in parentheses.

```zaka-error ZK0419
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
```

```text
error[ZK0419]: the kind holds 3 cells, but the list has 2
```

```zaka-error ZK0420
စာရင်းအလွတ် ကို ပြသပါ။
```

```text
error[ZK0420]: a blank list needs a list kind from its place
```

```zaka-error ZK0421
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း ဖြစ်ပါစေ။
```

```text
error[ZK0421]: this list kind has no cell count, and nothing here gives one
```

## Cells

| Written | Cell |
| --- | --- |
| `အမှတ်များ ၏ အကွက် ၀` | the first (counted from 0, as programs count) |
| `အမှတ်များ ၏ ၁ ခုမြောက်အကွက်` | the first (counted from 1, as people count) |
| `အမှတ်များ ၏ အကွက် ၂` | the third |
| `အမှတ်များ ၏ ၃ ခုမြောက်အကွက်` | the third |
| `အမှတ်များ ၏ အကွက်အရေအတွက်` | the number of cells, a `လက္ခဏာမဲ့ ကိန်းပြည့်` and a constant |

A cell is a place: it can be read, printed, assigned to, and its address taken. The
cell number is an integer (ZK0423), and the value in front of `၏` has to be a list or a
pointer to one (ZK0422).

```zaka-error ZK0422
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၇၀ ဖြစ်ပါစေ။
အမှတ် ၏ အကွက် ၀ ကို ပြသပါ။
```

```text
error[ZK0422]: isize is not a list, so it has no cells
```

### The cell number takes the whole expression

The number after `အကွက်` is read as far as the operator expression goes, just as
`ပ + ၁ ၏ တန်ဖိုး` reads through `ပ + ၁`. To use a cell inside a sum, put the cell in
parentheses.

```zaka
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၁၀၊ ၂၀၊ ၃၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက် ၀ + ၁ ကို ပြသပါ။          // the cell number is 0 + 1
(ကိန်းများ ၏ အကွက် ၀) + ၁ ကို ပြသပါ။            // the first cell, plus 1
```

```text
၂၀
၁၁
```

The word `အကွက်အရေအတွက်` is a single word. Written with a space, `အကွက် အရေအတွက်` is the
cell numbered with a variable named `အရေအတွက်`.

A counted loop visits each cell; the cell count, in parentheses, gives the last number:

```zaka
ကိန်းရှင် စုစုပေါင်း သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၄၊ ၈၊ ၁၅၊ ၁၆ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
၀ မှ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၁ အထိ န အနေဖြင့် ရေတွက်၍ စုစုပေါင်း ကို စုစုပေါင်း + (ကိန်းများ ၏ အကွက် န) ဟု ထားပါ။
စုစုပေါင်း ကို ပြသပါ။
```

```text
၄၃
```

## Lists of lists

A cell can hold a whole list, and a list of rows is a table. Each count prefix pairs with
one `စာရင်း`, the inner ones first: in `အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့်
စာရင်း စာရင်း`, the `၃` belongs to the first `စာရင်း` (a row of 3 integers) and the `၂`
to the second (2 such rows). Messages write this kind as `[2][3]isize`. Lists nest as deep
as needed.

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

Nested literals put each inner literal in parentheses. A kind with no counts takes all of
them from the value:

```zaka
ကိန်းရှင် ဇယား သည် ကိန်းပြည့် စာရင်း စာရင်း ဖြစ်၍ တန်ဖိုးသည် (၁၊ ၂ တို့ပါသော စာရင်း)၊ (၃၊ ၄ တို့ပါသော စာရင်း)၊ (၅၊ ၆ တို့ပါသော စာရင်း) တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ဇယား ကို ပြသပါ။
ဇယား ၏ ၃ ခုမြောက်အကွက် ၏ ၁ ခုမြောက်အကွက် ကို ပြသပါ။
```

```text
[[၁၊ ၂]၊ [၃၊ ၄]၊ [၅၊ ၆]]
၅
```

## How a list is stored

The cells of a list stand next to each other in one stretch of RAM, in the order of their
numbers, with no gaps. The size of a list is its count times the size of one cell. A list
of lists stores its rows one after another, so a table is one long stretch with the rows
end to end.

```text
  table: [2][3]isize, 6 cells of 8 octets = 48 octets

  octet    0      8      16     24     32     40     48
           +------+------+------+------+------+------+
           |  1   |  2   |  3   |  4   |  5   |  6   |
           +------+------+------+------+------+------+
  cell     [0][0] [0][1] [0][2] [1][0] [1][1] [1][2]
           '------- row 0 ------''------- row 1 ------'

  one past the end: octet 48, not a cell
```

```zaka
အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း ၏ အရွယ်အစား ကို ပြသပါ။
အကွက် ၃ ခုရှိသော ၃၂ ဘစ် ကိန်းပြည့် စာရင်း ၏ အရွယ်အစား ကို ပြသပါ။
အကွက် ၅ ခုရှိသော မှန်မှားတန်ဖိုး စာရင်း ၏ အရွယ်အစား ကို ပြသပါ။
```

```text
၄၈
၁၂
၅
```

The address of a cell is a pointer to the cell, so the cell after it can be reached with
pointer arithmetic, and the cell after the last of a row is the first of the next row:

```zaka
ကိန်းရှင် ဇယား သည် ကိန်းပြည့် စာရင်း စာရင်း ဖြစ်၍ တန်ဖိုးသည် (၁၊ ၂၊ ၃ တို့ပါသော စာရင်း)၊ (၄၊ ၅၊ ၆ တို့ပါသော စာရင်း) တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် အစ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ဇယား ၏ အကွက် ၀ ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် အတန်း သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ဇယား ၏ အကွက် ၁ ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။
အတန်း - အစ ကို ပြသပါ။                      // the second row starts 3 cells on
အစ + ၄ ၏ တန်ဖိုး ကို ပြသပါ။                  // row 1, column 1
```

```text
၃
၅
```

## Passing and comparing

A list is a value, like an integer. Setting one list to another duplicates each cell, and
the two lists stand apart afterwards. The same holds for a list passed as an argument,
since a parameter is a new variable:

```zaka
ကိန်းရှင် မူရင်း သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် ပွား သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း ဖြစ်ပါစေ။
ပွား ကို မူရင်း ဟု ထားပါ။              // each cell is duplicated
ပွား ၏ အကွက် ၀ ကို ၁၀၀ ဟု ထားပါ။
မူရင်း ကို ပြသပါ။
ပွား ကို ပြသပါ။
```

```text
[၁၊ ၂၊ ၃]
[၁၀၀၊ ၂၊ ၃]
```

- A change to a list parameter remains in the function. To let a function change the
  caller's list, or to spare it a duplicate of a large list, pass the address of the list.
  The parameter kind is the list kind with `လိပ်စာ` after it and `ငှားထားသော` in front.
- A function cannot give back a list as its result (ZK0424). Pass the address of a list
  to fill instead.
- A list literal as an argument takes each value back to the last `၊` that starts the
  argument list. When another argument comes before it, put the literal in parentheses.
- A list whose cells hold owners moves when it is assigned or passed; see [Ownership and
  borrowing](ownership.md#one-owner).

```zaka
// the parameter is the address of the caller's list
လုပ်ဆောင်ချက် နှစ်ဆတိုး သည်
ငှားထားသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ ကိန်းများ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၀ မှ ၂ အထိ န အနေဖြင့် ရေတွက်၍ ကိန်းများ ၏ အကွက် န ကို (ကိန်းများ ၏ အကွက် န) * ၂ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် မူရင်း သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် နှစ်ဆတိုး ကို မူရင်း ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်ပါ။
မူရင်း ကို ပြသပါ။     // changed through the address
```

```text
[၂၊ ၄၊ ၆]
```

```zaka
// a list literal after another argument goes in parentheses
လုပ်ဆောင်ချက် ပေါင်း သည်
ကိန်းပြည့် အစ နှင့် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း ကိန်းများ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
ရလဒ် ကို အစ ဟု ထားပါ။
၀ မှ ၂ အထိ န အနေဖြင့် ရေတွက်၍ ရလဒ် ကို ရလဒ် + (ကိန်းများ ၏ အကွက် န) ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

လုပ်ဆောင်ချက် ပေါင်း ကို ၁၀၀၊ (၁၊ ၂၊ ၃ တို့ပါသော စာရင်း) တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
```

```text
၁၀၆
```

### Comparing

`==` and `!=`, and the verbs `ညီ` and `မညီ`, compare two lists of the same kind, one cell
after another. Lists of different kinds, such as `[3]isize` and `[2]isize`, do not compare
(ZK0404). Lists have no order, so `<` and `>` do not take them (ZK0403).

```zaka
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။
က သည် ခ နှင့် ညီသည် ဟု ပြသပါ။
ခ ၏ အကွက် ၂ ကို ၄ ဟု ထားပါ။
က == ခ ကို ပြသပါ။
က != စာရင်းအလွတ် ကို ပြသပါ။
```

```text
မှန်
မမှန်
မှန်
```

```zaka-error ZK0404
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
က == ခ ကို ပြသပါ။
```

```text
error[ZK0404]: cannot combine [3]isize and [2]isize without a conversion
```

## Lists and addresses

The address of a list points at the whole list, so its kind holds the count:
`ငှားထားသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ`, or `&*[3]isize` in messages. Adding 1
to it moves one whole list, which for a list of rows is one row:

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

The address of one cell is a plain pointer to the kind of the cell. The `အကွက်` forms
work on a list and on a pointer to one (read through one level).

## Mistakes with cell numbers

A cell number outside the list is a mistake that Zaka catches at two times.

**At compile time.** A constant cell number outside the list is ZK0603, in each mode. When
the number is in a variable that the compiler can follow, as in a counted loop, or a
constant held in a variable, the same check reports ZK0710 with the size of the list:

```zaka-error ZK0603
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
အမှတ်များ ၏ ၀ ခုမြောက်အကွက် ကို ပြသပါ။     // counting from 1 starts at 1
```

```text
error[ZK0603]: cell 0 is outside the list, whose cells run from 1 to 3
```

```zaka-error ZK0710
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် န သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက် န ကို ပြသပါ။            // cells 0, 1 and 2 hold values; 3 does not exist
```

```text
error[ZK0710]: offset 3 is past the end of a block of 3 elements
```

A common cause is the number taking a whole sum. `ကိန်းများ ၏ အကွက် ၁ + ၂` is cell 3,
not cell 1 plus 2:

```zaka-error ZK0603
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၁၀၊ ၂၀၊ ၃၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက် ၁ + ၂ ကို ပြသပါ။         // cell 3, not cell 1 plus 2
```

```text
error[ZK0603]: cell 3 is outside the list, whose cells run from 0 to 2
```

**At run time.** When the number comes from input, or from a value the compiler cannot
follow, the VM checks each access and stops the program with the range the list
allows. This is what `--mem-check warn` shows for the program above:

```zaka-stop mem-check=warn
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် န သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက် န ကို ပြသပါ။            // cells 0, 1 and 2 hold values; 3 does not exist
```

```text
the program stopped: cell 3 is outside the list, whose cells run from 0 to 2
```

When the compiler proves that a cell number is inside the list, the VM does not
repeat the check for that access. A proof is given up whenever something the compiler
cannot follow could change the number, for example a counter whose address is taken or a
program that uses a raw kind; then the VM checks as before.

The other mistakes with lists, in one table:

| Code | Mistake |
| --- | --- |
| ZK0227 | a count that is not a whole number above zero |
| ZK0403 | `<`, `>` or another order used on lists |
| ZK0404 | lists of different kinds compared or combined |
| ZK0419 | a literal whose cell count differs from the kind |
| ZK0420 | `စာရင်းအလွတ်` with no list kind from its place |
| ZK0421 | a list kind with no cell count and no value to give one |
| ZK0422 | cells of a value that is not a list |
| ZK0423 | a cell number that is not an integer |
| ZK0424 | a list as the result of a function |
| ZK0603 | a constant cell number outside the list |
| ZK0710 | a cell number proven past the end |

The text of each message is in [Error messages](../reference/errors.md).
