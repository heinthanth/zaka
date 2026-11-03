---
title: Lists
description: The list kinds, literals, cell numbers, adding and removing cells, lists of lists, passing and comparing, then the plain and counted forms, static data, how a list is stored, addresses and loans, and the mistakes with cell numbers.
---

# Lists

A list holds values of one kind, next to each other in RAM. Each value sits in a *cell*,
and a program reaches a cell through its number. A list comes in two [forms](#forms): a
plain list takes its count from its value when the program runs, and a counted list holds
its count in its kind, known before the program runs. A set gives a plain list a new
count, and a plain list [grows and shrinks](#adding-and-removing-cells) a cell at a time. The Learn book starts in small steps with
[Lists](../../learn/lists/README.md).

```zaka
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက် ၀ ကို ၇၀ ဟု ထားပါ။
အမှတ်များ ၏ အကွက် ၂ ကို ၉၀ ဟု ထားပါ။
အမှတ်များ ကို ပြသပါ။
```

```text
[၇၀၊ ၀၊ ၉၀]
```

## List kinds

```text
<cell kind> စာရင်း
အကွက် <count> ခုရှိသော <cell kind> စာရင်း
```

| Kind | In messages | Form |
| --- | --- | --- |
| `ကိန်းပြည့် စာရင်း` | `isize[]` | plain |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း` | `isize[3]` | counted |

A written count comes first, then the kind of each cell, then `စာရင်း` ("list"). The word
`ခု` and a `၏` in front of `စာရင်း` are optional, so these four kinds are the same:

| Kind |
| --- |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း` |
| `အကွက် ၃ ရှိသော ကိန်းပြည့် စာရင်း` |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် ၏ စာရင်း` |
| `အကွက် ၃ ရှိသော ကိန်းပြည့် ၏ စာရင်း` |

- The count is a whole number above zero written with digits (ZK0227). It cannot come from
  a variable, because the size of a counted list is known before the program runs.
- The cell kind is a kind of each sort: a number, a true/false value, a character, text, a
  pointer, a struct, or a list.
- A counted list takes its default from its cells: `၀` for a number, `မမှန်` for a true/false
  value, `ဗလာ` for an address. A counted list declared with no value starts with all cells at
  the default. A plain list declared with no value holds no cells.

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

## Literals

A list value is written in one of two forms: its cells between `၊` and a closing
`တို့ပါသော စာရင်း` ("the list that holds these"), or its cells between `၊` inside `[` and
`]`, the form a list prints in. The two forms mean the same. `စာရင်းအလွတ်` ("blank list")
and `[]` are lists with no cells, save on a counted list, below.

```text
၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း
[၇၀၊ ၈၅၊ ၉၀]
စာရင်းအလွတ်
[]
```

- The number of cells in a literal must equal the count of the counted list it fills
  (ZK0419). A plain list takes a literal of each count.
- `စာရင်းအလွတ်` and `[]` each need a list kind from the place it stands in. Standing
  alone, neither has one (ZK0420).
- On a plain list, `စာရင်းအလွတ်` and `[]` are a list with no cells.
- On a counted list, `စာရင်းအလွတ်` sets each cell to its default, in each place. `[]` is a
  list with no cells, which a counted list cannot hold: it is allowed in the declaration of
  the list alone, where each cell starts at its default, and elsewhere it is ZK0436.
- A cell is a notation expression. A bracket list and a struct literal in the notation
  form are notation expressions, so each nests as written, as in `[[၁၊ ၂]၊ [၃၊ ၄]]`. A
  clause, or a literal in the `တို့ပါသော စာရင်း` form, goes in parentheses when it is a
  cell of another literal.

```zaka-error ZK0419
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
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

```zaka
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၇၊ ၈၊ ၉] ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [] ဖြစ်ပါစေ။
က ကို [] ဟု ထားပါ။
ခ ကို စာရင်းအလွတ် ဟု ထားပါ။
က ကို ပြသပါ။
ခ ကို ပြသပါ။
က ၏ အကွက်အရေအတွက် ကို ပြသပါ။
```

```text
[]
[၀၊ ၀]
၀
```

```zaka-error ZK0436
ကိန်းရှင် က သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၇၊ ၈၊ ၉] ဖြစ်ပါစေ။
က ကို [] ဟု ထားပါ။
```

```text
error[ZK0436]: [] is a list with no cells, and a counted list of isize[3] cannot take it here
```

## Cells

| Written | Cell |
| --- | --- |
| `အမှတ်များ ၏ အကွက် ၀` | the first (counted from 0, as programs count) |
| `အမှတ်များ ၏ ၁ ခုမြောက်အကွက်` | the first (counted from 1, as people count) |
| `အမှတ်များ ၏ အကွက် ၂` | the third |
| `အမှတ်များ ၏ ၃ ခုမြောက်အကွက်` | the third |
| `အမှတ်များ ၏ အကွက်အရေအတွက်` | the number of cells, a `လက္ခဏာမဲ့ ကိန်းပြည့်`: a constant for a counted list, known when the program runs for a plain list |

A cell is a place: it can be read, printed, assigned to, and its address taken. The
cell number is an integer (ZK0423), and the value in front of `၏` has to be a list, a
pointer to one, or a [slice](slices.md) (ZK0422).

```zaka-error ZK0422
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀ ဖြစ်ပါစေ။
အမှတ် ၏ အကွက် ၀ ကို ပြသပါ။
```

```text
error[ZK0422]: isize is not a list, so it has no cells
```

An [each-cell loop](statements.md#loops), `အမှတ်များ ၏ အကွက်တိုင်း ကို အမှတ် အနေဖြင့်
ယူ၍ ...`, reads each cell in turn into a variable of its own; a list whose cells hold an
owner pointer cannot be walked so (ZK0434).

### The cell number takes the whole expression

The number after `အကွက်` is read as far as the operator expression goes, just as
`ပ + ၁ ၏ တန်ဖိုး` reads through `ပ + ၁`. To use a cell inside a sum, put the cell in
parentheses.

```zaka
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀၊ ၂၀၊ ၃၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
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
ကိန်းရှင် စုစုပေါင်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၄၊ ၈၊ ၁၅၊ ၁၆ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
၀ မှ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၁ အထိ န အနေဖြင့် ရေတွက်၍ စုစုပေါင်း ကို စုစုပေါင်း + (ကိန်းများ ၏ အကွက် န) ဟု ထားပါ။
စုစုပေါင်း ကို ပြသပါ။
```

```text
၄၃
```

## Adding and removing cells

```text
<list> ထဲသို့ <value> ကို ထည့်ပါ။
<list> ၏ နောက်ဆုံးအကွက်ကို ဖျက်ပါ။
```

`ထည့်ပါ` ("please add") puts the value in a new last cell of the list, and `ဖျက်ပါ`
("please remove") takes the last cell off. Neither statement gives a value back.

```zaka
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
၁ မှ ၄ အထိ န အနေဖြင့် ရေတွက်၍ က ထဲသို့ န * ၁၀ ကို ထည့်ပါ။
က ၏ နောက်ဆုံးအကွက်ကို ဖျက်ပါ။
က ကို ပြသပါ။
က ၏ အကွက်အရေအတွက် ကို ပြသပါ။
```

```text
[၁၀၊ ၂၀၊ ၃၀]
၃
```

- The list is a place that can be set: a variable, a field, a cell of another list, or
  `ပ ၏ တန်ဖိုး` through a pointer `ပ`. A constant is ZK0306, as for a set. A pointer to a
  list is no list: add to `ပ ၏ တန်ဖိုး`, not to `ပ`.
- The list is a plain list. A counted list keeps its count, and a slice holds no
  cells of its own to add to: each is ZK0437. An add to text is the [add to
  text](text.md#changing-text), with rules of its own; a remove from text is ZK0437.
- The value has the cell kind (ZK0401), and it goes into the list as an assigned value
  goes into a place: text and lists are duplicated, and an owner pointer moves into the
  list, so the variable it came from holds `ဗလာ` (ZK0701 on a later read).
- The value is worked out before the list changes: `က ထဲသို့ က ၏ အကွက်အရေအတွက် ကို
  ထည့်ပါ။` adds the count from before the add.
- `နောက်ဆုံးအကွက်` ("last cell") is read right after `၏` alone, and `ကို ဖျက်ပါ` must
  follow it (ZK0201): it is no expression, so the last cell is read through its number.
- A plain list parameter is lent with a room of 0 (see [How a list is
  stored](#how-a-list-is-stored)). Its first add, as its first write, gives
  it a block of its own, so the caller's list never changes. A remove on a list with a
  room of 0 lowers the count and leaves the shared cells unchanged: it duplicates nothing.
- A full block moves to a block twice its size, four cells at the least.
- A remove on a list with no cells stops the program: "the list holds no cells, so it has
  no last cell to remove" (see [When a program stops](../vm/traps.md#all-the-traps)).
- A remove drops the value in the last cell when the list owns its block: text in it is
  freed.
- Each add or remove ends each loan into the list; see [Edits and
  loans](#edits-and-loans).

```zaka
ကိန်းရှင် ဇယား သည် ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [[၁]၊ [၂၊ ၃]] ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ငှားထားသော ကိန်းပြည့် စာရင်း စာရင်း လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဇယား ၏ လိပ်စာ ဖြစ်ပါစေ။
ဇယား ၏ အကွက် ၀ ထဲသို့ ၅ ကို ထည့်ပါ။
ပ ၏ တန်ဖိုး ထဲသို့ [၆] ကို ထည့်ပါ။
ဇယား ၏ အကွက် ၁ ၏ နောက်ဆုံးအကွက်ကို ဖျက်ပါ။
ဇယား ကို ပြသပါ။
```

```text
[[၁၊ ၅]၊ [၂]၊ [၆]]
```

```zaka-error ZK0437
ကိန်းရှင် က သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။
က ၏ နောက်ဆုံးအကွက်ကို ဖျက်ပါ။
```

```text
error[ZK0437]: isize[2] cannot grow or shrink here: "ထည့်ပါ" takes a plain list or text, and "ဖျက်ပါ" a plain list
```

```zaka-stop
ကိန်းရှင် က သည် စာသား စာရင်း တစ်ခု ဖြစ်ပါစေ။
က ၏ နောက်ဆုံးအကွက်ကို ဖျက်ပါ။
```

```text
the program stopped: the list holds no cells, so it has no last cell to remove
```

## Lists of lists

A cell can hold a whole list, and a list of rows is a table. Each count prefix pairs with
one `စာရင်း`, the inner ones first: in `အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့်
စာရင်း စာရင်း`, the `၃` belongs to the first `စာရင်း` (a row of 3 integers) and the `၂`
to the second (2 such rows). Messages write this kind as `isize[3][2]`. Lists of each form
nest as deep as needed: `isize[3][]` is a plain list of counted rows, and `isize[][]` a
plain list of plain rows, each row with a count of its own.

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

Nested literals put each inner literal in parentheses. In a kind with no counts, each list
is plain and takes its count from the value:

```zaka
ကိန်းရှင် ဇယား သည် ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (၁၊ ၂ တို့ပါသော စာရင်း)၊ (၃၊ ၄ တို့ပါသော စာရင်း)၊ (၅၊ ၆ တို့ပါသော စာရင်း) တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ဇယား ကို ပြသပါ။
ဇယား ၏ ၃ ခုမြောက်အကွက် ၏ ၁ ခုမြောက်အကွက် ကို ပြသပါ။
```

```text
[[၁၊ ၂]၊ [၃၊ ၄]၊ [၅၊ ၆]]
၅
```

## Passing and comparing

A list is a value, like an integer. Setting one list to another duplicates each cell, and
the two lists stand apart afterwards. The same holds for a list passed as an argument,
since a parameter is a new variable:

```zaka
ကိန်းရှင် မူရင်း သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် ပွား သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
ပွား ကို မူရင်း ဟု ထားပါ။              // each cell is duplicated
ပွား ၏ အကွက် ၀ ကို ၁၀၀ ဟု ထားပါ။
မူရင်း ကို ပြသပါ။
ပွား ကို ပြသပါ။
```

```text
[၁၊ ၂၊ ၃]
[၁၀၀၊ ၂၊ ၃]
```

- Each text and each list in the cells is duplicated as well, with a block of its own.
- A change to a list parameter remains in the function. A plain list parameter is lent
  the caller's cells, with a room of 0 ([How a list is stored](#how-a-list-is-stored)),
  until its first write; that write first copies the
  cells into a block of its own (the VM op `own`), so the caller's list never changes. A
  counted list parameter takes a
  duplicate, or reads the caller's list in place when the function never writes it.
- To let a function change the caller's list, pass the address of the list. The parameter
  kind is the list kind with `လိပ်စာ` after it and `ငှားထားသော` in front.
- A plain list parameter takes lists of each count, and a counted list given to it is
  duplicated into a heap block. To take a part of a list, or a counted list with no
  duplicate, take a [slice](slices.md): the argument `x ၏ တန်ဖိုးအငှား` lends the whole
  list.
- A function can give back a list of each form; a counted result kind gives the cell count
  as a written count. Its cells
  cannot be borrows (ZK0424); see [Lists and structs as
  results](functions.md#lists-and-structs-as-results).
- A list literal as an argument takes each value back to the last `၊` that starts the
  argument list. When another argument comes before it, put the literal in parentheses.
- A list whose cells hold owners, plain or counted, moves when it is assigned or passed; see
  [Ownership and borrowing](ownership.md#one-owner).

```zaka
// the parameter is the address of the caller's list
လုပ်ဆောင်ချက် နှစ်ဆတိုး သည်
ငှားထားသော ကိန်းပြည့် စာရင်း လိပ်စာ ကိန်းများ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၀ မှ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၁ အထိ န အနေဖြင့် ရေတွက်၍ ကိန်းများ ၏ အကွက် န ကို (ကိန်းများ ၏ အကွက် န) * ၂ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် မူရင်း သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
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
after another. Two plain lists are equal when their counts match and each cell is equal (the VM
op `seq_eq`). Lists of different
kinds do not compare (ZK0404): counted lists of different counts, such as `isize[3]` and
`isize[2]`, or a counted and a plain list. Lists have no order, so `<` and `>` do not take
them (ZK0403).

```zaka
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။
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
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
က == ခ ကို ပြသပါ။
```

```text
error[ZK0404]: cannot combine isize[] and isize[3] without a conversion
```

## Forms

| Form | Burmese | Its cells |
| --- | --- | --- |
| plain | `စာရင်း` | in static data, in a heap block it owns, or lent from another list; the count comes when the program runs |
| counted | a written count, `အကွက် <n> ခုရှိသော` | inline: in the frame, the static block or the struct that holds the list; it never allocates |

- **A plain list** is three words, laid out as text is: the address of its first cell, its
  count, and the room of its heap block in octets. A literal of known values points at
  [static data](#static-data); each other literal is built, and its cells go to a heap
  block of their own. A set replaces the whole value, so the count can change.
- **A counted list** holds its cells in place, and its count is part of its kind:
  `isize[3]` and `isize[2]` are different kinds.
- A list literal with no counted list around it is a plain list:
  `ကိန်းရှင် l သည် [၁၊ ၂] ဖြစ်ပါစေ။` declares a plain list of two cells.
- A constant keeps the kind its declaration gives: a plain list remains plain, and a
  written count makes a counted list.

Conversions happen on their own where a value goes into a place: a declaration, a set, an
argument, a result, a struct field or a list cell.

| From | To | |
| --- | --- | --- |
| counted | plain | on its own: the cells are duplicated into a heap block |
| plain | counted | never on its own (ZK0401): the count of a counted list is part of its kind |

```zaka
ကိန်းရှင် က သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။
ခ ကို [၄၊ ၅] ဟု ထားပါ။
က ကို ပြသပါ။
ခ ကို ပြသပါ။
```

```text
[၁၊ ၂၊ ၃]
[၄၊ ၅]
```

```zaka-error ZK0401
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။
```

```text
error[ZK0401]: expected isize[3], found isize[]
```

## Static data

A plain list literal whose cells are each known at compile time is *static data*: its cells
sit in a frozen static block of the program, as the u8s of a string literal do, and the list
points there with a room of 0. Known cells are constants, sizes, static text, blank lists,
and counted lists, structs and plain lists of known cells.

- Reads (a cell, the count, a print, a compare, an each-cell loop) work as on each other
  plain list, and take no heap block.
- The first edit (a cell write, an add, a remove) or a slice of the list first copies the
  cells into a heap block of its own, so the static data never changes. A remove lowers the
  count alone and copies nothing.
- A duplicate shares the static block, as a duplicate of static text does.
- A literal with a value known when the program runs and not before, such as a variable, is built in
  a heap block.
- With `--fold none` a literal of operations on constants is built when the program runs;
  the output is the same.
- A counted list keeps its cells in place, so it never points at static data. A counted
  literal of known cells is still a static list, and the list copies its cells from it in
  one step, in place of a write for each cell.

```zaka
ကိန်းရှင် l သည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် m သည် l ဖြစ်ပါစေ။
m ထဲသို့ ၄ ကို ထည့်ပါ။
l ကို ပြသပါ။
m ကို ပြသပါ။
```

```text
[၁၊ ၂၊ ၃]
[၁၊ ၂၊ ၃၊ ၄]
```

A `ကိန်းသေ` that holds a plain list (or text, or a list or struct with either inside) is
static data in each build: its value must fold at compile time, whatever `--fold` holds,
else ZK0606, reported at the first cell that does not fold. With constant propagation on,
a cell read of such a constant at a constant number, a field of a struct in it, and its
count become constants, and a constant number outside it is ZK0603.

```zaka
ကိန်းသေ l သည် [၁၀၊ ၂၀၊ ၃၀] ဖြစ်ပါစေ။
ကိန်းသေ x သည် l ၏ အကွက် ၁ ဖြစ်ပါစေ။
x ကို ပြသပါ။
l ၏ အကွက်အရေအတွက် ကို ပြသပါ။
```

```text
၂၀
၃
```

```zaka-error ZK0606
ကိန်းရှင် n သည် ၃ ဖြစ်ပါစေ။
ကိန်းသေ l သည် [n၊ ၂] ဖြစ်ပါစေ။
```

```zaka-error ZK0603
ကိန်းသေ l သည် [၁၀၊ ၂၀၊ ၃၀] ဖြစ်ပါစေ။
l ၏ အကွက် ၅ ကို ပြသပါ။
```

## How a list is stored

The cells of a list stand next to each other in one stretch of RAM, in the order of their
numbers, with no gaps. A counted list is that stretch, inline: its size is its count times
the size of one cell. A counted list of counted lists stores its rows one after another, so a
table is one long stretch with the rows end to end.

```text
  table: isize[3][2], 6 cells of 8 octets = 48 octets

  octet    0      8      16     24     32     40     48
           +------+------+------+------+------+------+
           |  1   |  2   |  3   |  4   |  5   |  6   |
           +------+------+------+------+------+------+
  cell     [0][0] [0][1] [0][2] [1][0] [1][1] [1][2]
           '------- row 0 ------''------- row 1 ------'

  one past the end: octet 48, not a cell
```

A plain list is three words, laid out as text is, and its cells sit in static data or in a
heap block:

```text
  list: isize[], holding [1, 2, 3]

  the list, 24 octets                       its heap block, 24 octets
  +-----------+-----------+-----------+     +------+------+------+
  |  address  |  count 3  |  room 24  |     |  1   |  2   |  3   |
  +-----+-----+-----------+-----------+     +------+------+------+
        |                                   ^
        '-----------------------------------'
```

- The room is the size of the heap block in octets. A room of 0 marks cells the list does
  not own: cells lent to a parameter, or no cells at all.
- A write into a list of room 0 first copies the cells into a block of its own, so a
  plain list never writes cells it was lent.
- An add into a full block moves the cells to a block twice its size, four cells at the
  least; a remove keeps the block and its room.
- Each list in the cells of a plain list is a value of its own: a plain list of plain
  lists holds three words per row, and each row has a block of its own.

```zaka
အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း ၏ အရွယ်အစား ကို ပြသပါ။
အကွက် ၃ ခုရှိသော ၃၂ ဘစ် ကိန်းပြည့် စာရင်း ၏ အရွယ်အစား ကို ပြသပါ။
အကွက် ၅ ခုရှိသော မှန်မှားတန်ဖိုး စာရင်း ၏ အရွယ်အစား ကို ပြသပါ။
ကိန်းပြည့် စာရင်း ၏ အရွယ်အစား ကို ပြသပါ။
ကိန်းပြည့် စာရင်း စာရင်း ၏ အရွယ်အစား ကို ပြသပါ။
```

```text
၄၈
၁၂
၅
၂၄
၂၄
```

The address of a cell is a pointer to the cell, so the cell after it can be reached with
pointer arithmetic. In a table of counted lists, the cell after the last of a row is the
first of the next row; the rows of a plain list each sit in a block of their own, and a
step from one into the next stops the program:

```zaka
ကိန်းရှင် ဇယား သည် အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (၁၊ ၂၊ ၃ တို့ပါသော စာရင်း)၊ (၄၊ ၅၊ ၆ တို့ပါသော စာရင်း) တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် အစ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဇယား ၏ အကွက် ၀ ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် အတန်း သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဇယား ၏ အကွက် ၁ ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။
အတန်း - အစ ကို ပြသပါ။                      // the second row starts 3 cells on
အစ + ၄ ၏ တန်ဖိုး ကို ပြသပါ။                  // row 1, column 1
```

```text
၃
၅
```

## Lists and addresses

The address of a list points at the whole list. For a counted list its kind holds the
count: `ငှားထားသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ`, or `isize[3]&` in messages.
Adding 1 to it moves one whole list, which for a counted list of counted rows is one row. The
address of a plain list, `isize[]&`, points at its three words, and an `isize[]&` does not
take an `isize[3]&` (ZK0401).

```zaka
ကိန်းရှင် ဇယား သည် အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (၁၊ ၂၊ ၃ တို့ပါသော စာရင်း)၊ (၄၊ ၅၊ ၆ တို့ပါသော စာရင်း) တို့ပါသော စာရင်း ဖြစ်ပါစေ။
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

The address of one cell is a plain pointer to the kind of the cell. The `အကွက်` forms
work on a list and on a pointer to one (read through one level).

## Edits and loans

A write into a plain list can first move its cells to a block of its own, when its room is
0. So each write into a plain list, to a cell, a field of a cell, or the whole list, ends
each loan into it: a slice of the list, or a pointer to one of its cells, used after the
write is ZK0704. The cells of a counted list never move.

An [add or a remove](#adding-and-removing-cells) ends each loan into the list too: an add can
move the cells to a larger block, and a remove drops the last cell. Each also ends the loans
into a plain list that holds the edited one, as a write does, since that list first takes a
block of its own.

```zaka-error ZK0704
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။
က ထဲသို့ ၄ ကို ထည့်ပါ။
ပ ၏ တန်ဖိုး ကို ပြသပါ။
```

```text
error[ZK0704]: the block of "က" is edited, so its cells can move, here, but "ပ" still borrows it and is used later
```

```zaka-error ZK0704
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် s သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
က ၏ အကွက် ၀ ကို ၉ ဟု ထားပါ။
s ကို ပြသပါ။
```

```text
error[ZK0704]: the block of "က" is edited, so its cells can move, here, but "s" still borrows it and is used later
```

```zaka
ကိန်းရှင် က သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် s သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
က ၏ အကွက် ၀ ကို ၉ ဟု ထားပါ။
s ကို ပြသပါ။
```

```text
[၉၊ ၂၊ ၃]
```

## Mistakes with cell numbers

A cell number outside the list is a mistake that Zaka catches at two times.

**At compile time,** for a counted list, whose count the compiler knows. A constant cell
number outside the list is ZK0603, in each mode. When the number is in a variable that the
compiler can follow, as in a counted loop, or a constant held in a variable, the same check
reports ZK0710 with the size of the list:

```zaka-error ZK0603
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
အမှတ်များ ၏ ၀ ခုမြောက်အကွက် ကို ပြသပါ။     // counting from 1 starts at 1
```

```text
error[ZK0603]: cell 0 is outside the list, whose cells run from 1 to 3
```

```zaka-error ZK0710
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် န သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက် န ကို ပြသပါ။            // cells 0, 1 and 2 hold values; 3 does not exist
```

```text
error[ZK0710]: offset 3 is past the end of a block of 3 elements
```

A common cause is the number taking a whole sum. `ကိန်းများ ၏ အကွက် ၁ + ၂` is cell 3,
not cell 1 plus 2:

```zaka-error ZK0603
ကိန်းရှင် ကိန်းများ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀၊ ၂၀၊ ၃၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက် ၁ + ၂ ကို ပြသပါ။         // cell 3, not cell 1 plus 2
```

```text
error[ZK0603]: cell 3 is outside the list, whose cells run from 0 to 2
```

**At run time.** The count of a plain list comes when the program runs, so the VM
checks each of its cells, constant numbers too. For a counted list, when the number comes
from input, or from a value the compiler cannot follow, the VM checks each access alike.
It stops the program with the range the list allows. This is what a plain list does with
the program above, and what `--mem-check warn` shows for the counted one:

```zaka-stop
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် န သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
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
| ZK0401 | a plain list given where a counted list is wanted |
| ZK0403 | `<`, `>` or another order used on lists |
| ZK0404 | lists of different kinds compared or combined, such as a counted and a plain list |
| ZK0419 | a literal whose cell count differs from the counted kind |
| ZK0420 | `စာရင်းအလွတ်` with no list kind from its place |
| ZK0422 | cells of a value that is not a list or a slice |
| ZK0423 | a cell number that is not an integer |
| ZK0424 | a list of borrows as the result of a function |
| ZK0436 | `[]` given to a counted list outside its declaration |
| ZK0437 | an add or a remove on a counted list or a slice, or a remove from text |
| ZK0603 | a constant cell number outside the list |
| ZK0606 | a constant plain list whose value needs the program to run |
| ZK0704 | a slice of a plain list used after a write into the list, or after an add or a remove |
| ZK0710 | a cell number proven past the end |

The text of each message is in [Error messages](../reference/errors.md).
