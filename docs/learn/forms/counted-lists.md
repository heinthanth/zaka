---
title: Static data and counted lists
description: Where the cells of text and lists live - static data for a literal or a constant, a heap block from the first edit - and the counted list, whose count is part of its data kind and whose cells sit in the list itself.
---

# Static data and counted lists

The text and lists of Part I, `စာသား` and `ကိန်းပြည့် စာရင်း`, are managed for us: the
compiler picks where in storage (memory) their cells live, and moves them when it needs
to. This page shows where that is, and the one list data kind that keeps its cells in
place, the *counted list* (fixed-size array).

## You will learn

- Where the cells of a literal live, and when these move to the heap.
- How to declare a counted list, with a count that is part of its data kind.
- How to fill a counted list one cell at a time, or with a count from constants.
- Which mistakes the compiler catches on a counted list before the program runs.

## Where the cells live

A literal whose cells are known before the program runs sits in the program itself, as
*static data*. This works for text, numbers, lists of these and lists of
lists. A variable that we set from such a literal points at the static data, and needs
nothing from the heap (dynamic memory). Its first edit, such as a write, an add or a
remove, duplicates (copies) the cells into a new heap block, so the literal itself never
changes:

```zaka
ကိန်းရှင် ရောင်များ သည် ["နီ"၊ "ဝါ"] ဖြစ်ပါစေ။
ကိန်းရှင် ရွေး သည် ရောင်များ ဖြစ်ပါစေ။
ရွေး ထဲသို့ "စိမ်း" ကို ထည့်မည်။          // the first edit: a new heap block
ရောင်များ ကို ပြသမည်။
ရွေး ကို ပြသမည်။
```

```text
["နီ"၊ "ဝါ"]
["နီ"၊ "ဝါ"၊ "စိမ်း"]
```

In the Explorer, the heap view shows the literal as a static list, and the first heap
block appears at the add. A list built from values that are known just when the program
runs, such as `[က၊ ခ]` with two variables, needs a heap block from the start. Text works
the same: see [Text](../text/README.md).

## Constants of text and lists

```
ကိန်းသေ <name> သည် <literal> ဖြစ်ပါစေ။
```

A constant of text or of a plain list is static data in each build. Its value must be
known before the program runs, so the compiler can work out a cell read and the count of
the constant at compile time, and neither runs while the program runs:

```zaka
ကိန်းသေ ရက်များ သည် [၃၁၊ ၂၈၊ ၃၁] ဖြစ်ပါစေ။
ကိန်းသေ ဖေဖော်ဝါရီ သည် ရက်များ ၏ အကွက် ၁ ဖြစ်ပါစေ။
ဖေဖော်ဝါရီ ကို ပြသမည်။
ရက်များ ၏ အကွက်အရေအတွက် ကို ပြသမည်။
```

```text
၂၈
၃
```

A value that needs the program to run, such as a variable in a cell, is an error, so we
declare such a list with `ကိန်းရှင်` instead:

```zaka-error ZK0606
ကိန်းရှင် n သည် ၃ ဖြစ်ပါစေ။
ကိန်းသေ l သည် [n၊ ၂] ဖြစ်ပါစေ။
```

```text
error[ZK0606]: a constant that holds text or a plain list must be known at compile time
```

## Counted lists

A counted list keeps the same count of cells for its whole life, because the count is
part of its data kind, and the compiler knows it before the program runs. Its cells sit
in the list itself, so it never needs a heap block, and a literal of known cells is
duplicated in from static data in one step:

```zaka
ကိန်းရှင် က သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၇၊ ၈] ဖြစ်ပါစေ။
က ကို ပြသမည်။
ခ ကို ပြသမည်။
```

```text
[၀၊ ၀၊ ၀]
[၇၊ ၈]
```

To write the data kind of a counted list, we put the count first, then the data kind of
each cell, then `စာရင်း`:

```
အကွက် <count> ခုရှိသော <kind> စာရင်း
```

| Zaka | Reads as |
| --- | --- |
| `အကွက် ၃` | "cells, 3" |
| `ခုရှိသော` | "that has (that number)"; `ခု` is the measure word (classifier) |
| `ကိန်းပြည့် စာရင်း` | "list of integers" |

The two list data kinds next to each other, with the short names that error messages use:

| Data kind | Short name | Its cells |
| --- | --- | --- |
| `ကိန်းပြည့် စာရင်း` | `isize[]` | plain: the count comes with the value |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း` | `isize[3]` | counted: 3 cells for good, kept in the list itself |

The count is a whole number above zero, written as digits or worked out from constants
(see [A count from constants](#a-count-from-constants)). With `ကနဦးတန်ဖိုး`, each cell
starts at the [initial value](../variables/README.md#the-initial-value) (default value)
of its data kind: `၀` for a number, and `မမှန်` for a true/false value.

## Filling the cells one at a time

When we declare a counted list with no value, it has no cells we can read until the
program sets them. The compiler follows each cell, and counts the list as set once each
cell is set, either through cell numbers written in place or through a counted loop over
each cell that writes the cell of its counter:

```zaka
ကိန်းရှင် က သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
က ၏ အကွက် ၀ ကို ၅ ဟု ထားမည်။
က ၏ အကွက် ၁ ကို ၆ ဟု ထားမည်။
က ကို ပြသမည်။
ကိန်းရှင် ခ သည် အကွက် ၄ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
၀ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ ခ ၏ အကွက် န ကို န * ၁၀ ဟု ထားမည်။    // each cell, 0 to 3
ခ ကို ပြသမည်။
```

```text
[၅၊ ၆]
[၀၊ ၁၀၊ ၂၀၊ ၃၀]
```

A read of a cell that no write has reached is the error ZK0722. When the compiler cannot
tell which cells a write reaches, as with a cell number in a variable or a loop over some
of the cells, the read gets the warning ZK0723, and the program still runs:

```zaka-warning ZK0723
ကိန်းရှင် က သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
၀ မှ ၁ အထိ န အနေဖြင့် ရေတွက်၍ က ၏ အကွက် န ကို န ဟု ထားမည်။     // cell 2 is never set
က ကို ပြသမည်။
```

```text
warning[ZK0723]: "က" can have a cell with no value here; the check cannot tell which cells a write or a read reaches
```

To avoid the warning, we fill such a list with `ကနဦးတန်ဖိုး` or a literal first, or loop
over each of its cells.

## A count from constants

```
အကွက် <constant expression> ခုရှိသော <kind> စာရင်း
```

A count can use constants and work with them, so one constant can set the size of
several lists. The compiler works out the count before the program runs, the same as it
folds other constants:

```zaka
ကိန်းသေ အရွယ် သည် ၃ ဖြစ်ပါစေ။

ကိန်းရှင် က သည် အကွက် အရွယ် ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အကွက် အရွယ် * ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
က ကို ပြသမည်။
ခ ကို ပြသမည်။
```

```text
[၀၊ ၀၊ ၀]
[၀၊ ၀၊ ၀၊ ၀၊ ၀၊ ၀]
```

A variable does not work, because its value is known when the program runs, and the size
of the list must be fixed before that:

```zaka-error ZK0227
ကိန်းရှင် အရွယ် သည် ၃ ဖြစ်ပါစေ။
ကိန်းရှင် က သည် အကွက် အရွယ် ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
```

```text
error[ZK0227]: "အရွယ်" is not a cell count; write a whole number above zero, in digits or worked out from constants
```

## Lists of counted lists

```
အကွက် <rows> ခုရှိသော အကွက် <columns> ခုရှိသော <kind> စာရင်း စာရင်း
```

In a counted list of counted lists, each count goes with one `စာရင်း`, the inner one
first. In `အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း`, the `၃` belongs to
the first `စာရင်း` (a row of 3 integers) and the `၂` to the second (2 such rows), and
messages write this data kind as `isize[3][2]`:

```zaka
ကိန်းရှင် ဂရစ် သည် အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။ // 2 rows of 3
ဂရစ် ၏ အကွက် ၁ ၏ အကွက် ၂ ကို ၉ ဟု ထားမည်။                                          // row 1, column 2
ဂရစ် ကို ပြသမည်။
ဂရစ် ၏ အကွက် ၁ ကို ပြသမည်။
```

```text
[[၀၊ ၀၊ ၀]၊ [၀၊ ၀၊ ၉]]
[၀၊ ၀၊ ၉]
```

The rows sit one after another in storage, with no gap, and Part III walks them with an
address in [Rows and cells through
addresses](../pointers/structs.md#rows-and-cells-through-addresses).

## From a counted list to a plain list

A counted list turns into a plain list when we assign it to one, and its cells are
duplicated into a heap block. The other direction does not work (see [A counted list
from a plain list](#a-counted-list-from-a-plain-list)):

```zaka
ကိန်းရှင် က သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။
ခ ကို [၄၊ ၅] ဟု ထားမည်။
က ကို ပြသမည်။
ခ ကို ပြသမည်။
```

```text
[၁၊ ၂၊ ၃]
[၄၊ ၅]
```

## Counted lists in functions

A parameter or a result can be a counted list, such as
`အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း`. The count is part of the data kind, so the
function accepts lists of that count alone. A plain list parameter accepts a counted list
too, and duplicates its cells into a heap block first.

Here the loop sets each cell of the counted result before the return:

```zaka
// a function that builds a counted list and returns it
နှစ်ထပ်များ ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အကွက် ၄ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် အကွက် ၄ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
၀ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ အဖြေ ၏ အကွက် န ကို န * န ဟု ထားမည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် နှစ်ထပ်များ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းများ ကို ပြသမည်။
```

```text
[၀၊ ၁၊ ၄၊ ၉]
```

The caller gets a separate list, the same as with an assignment. Here it is a plain list,
which gets the cells of the counted one.

## The size of a counted list

A counted list takes its count times the size of one cell, with no gaps, while a plain
list takes three words, whatever its count:

```zaka
အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
ကိန်းပြည့် စာရင်း တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
```

```text
၄၈
၂၄
```

The first is 6 integers of 8 octets (bytes) each. The second is the three words of a
plain list, of 8 octets each, which point at the cells on the heap and keep the count.

## A cell number past a counted list

The count of a plain list comes when the program runs, so the VM checks each cell number
then. A counted list has its count in its data kind, so the compiler checks each cell
number it can follow. Here the number is in a variable, and the compiler still works out
that it is 3:

```zaka-error ZK0710
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅ နှင့် ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် န သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက် န ကို ပြသမည်။            // cells 0, 1 and 2 have values; 3 does not exist
```

```text
error[ZK0710]: offset 3 is past the end of a block of 3 elements
```

This check is an error in each mode. When the number comes from input, the compiler
cannot follow it, so the check of the VM catches it and stops the program. When the
number is written in place, as a fixed number, the compiler reports it with another code.
Counting from 1 starts at 1, not 0:

```zaka-error ZK0603
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅ နှင့် ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
အမှတ်များ ၏ ၀ ခုမြောက်အကွက် ကို ပြသမည်။     // counting from 1 starts at 1
```

```text
error[ZK0603]: cell 0 is outside the list, whose cells run from 1 to 3
```

The cell number is the whole operator expression after `အကွက်`, so this program reads
cell `၁ + ၂`:

```zaka-error ZK0603
ကိန်းရှင် ကိန်းများ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀၊ ၂၀ နှင့် ၃၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက် ၁ + ၂ ကို ပြသမည်။         // cell 3, not cell 1 plus 2
```

```text
error[ZK0603]: cell 3 is outside the list, whose cells run from 0 to 2
```

## A literal with the wrong count

The literal must have the same number of cells as the count in the data kind:

```zaka-error ZK0419
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀ နှင့် ၈၅ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
```

```text
error[ZK0419]: the kind holds 3 cells, but the list has 2
```

## A list with no cells in a counted list

`[]` and `စာရင်းအလွတ်` are lists with no cells. A counted list keeps its count, so neither
one fits it, in a declaration or elsewhere. To set each cell to its initial value, we
use `ကနဦးတန်ဖိုး` instead:

```zaka-error ZK0436
ကိန်းရှင် က သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၇၊ ၈၊ ၉] ဖြစ်ပါစေ။
က ကို [] ဟု ထားမည်။
```

```text
error[ZK0436]: a list with no cells does not fit the counted list isize[3]
```

## A counted list from a plain list

A plain list does not turn into a counted list, because nothing promises its count:

```zaka-error ZK0401
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။
```

```text
error[ZK0401]: expected isize[3], found isize[]
```

## Adding to a counted list

A counted list keeps its count for good, so we cannot use `ထည့်မည်` or `ဖျက်မည်` on it:

```zaka-error ZK0437
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
အမှတ်များ ထဲသို့ ၇၀ ကို ထည့်မည်။
```

```text
error[ZK0437]: isize[3] cannot grow or shrink here: "ထည့်မည်" takes a plain list or text, "နောက်ဆုံးအကွက်ကို ဖျက်မည်" a plain list, and "နောက်ဆုံးစာလုံးကို ဖျက်မည်" text
```

To add cells to a list, we declare it with no count, as `ကိန်းပြည့် စာရင်း`.

## Details

- **Other spellings.** The word `ခု` and a `၏` before `စာရင်း` are both optional, so
  `အကွက် ၃ ရှိသော ကိန်းပြည့် စာရင်း` and `အကွက် ၃ ခုရှိသော ကိန်းပြည့် ၏ စာရင်း` are the
  same data kind as `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း` ([Lists](../../manual/language/lists.md#list-kinds)).
- **What a count can use.** In a count, the compiler accepts each operator and conversion
  it can work out, the size of a data kind, a cell or a field of a constant list or
  struct, and the cell count of a counted list.
- **Comparing.** We cannot compare a counted list with a plain list using `==` and `!=`,
  and neither two counted lists of different counts, such as `isize[3]` and `isize[2]`.
- **Static data in full.** [Static data](../../manual/language/counted-lists.md#static-data)
  in Docs lists which literals become static data, and [How a list is
  stored](../../manual/language/counted-lists.md#how-a-list-is-stored) shows the three words of a
  plain list.

## Recap

- A literal of known cells, or a constant of text or a plain list, sits in the program as
  static data, and its first edit duplicates it into a heap block.
- `အကွက် <count> ခုရှိသော <kind> စာရင်း` is a counted list, whose count is part of its data
  kind and whose cells sit in the list itself.
- A count can be digits or a constant expression, but not a variable.
- The compiler checks the cell numbers of a counted list that it can follow, and a counted
  list cannot grow or shrink.

## Hands on

- Declare a counted list of five decimal numbers with a written count and
  `ကနဦးတန်ဖိုး`, set the middle one, and print the list. Then drop `ကနဦးတန်ဖိုး` and set
  each cell in a counted loop.
- Read cell 5 of a counted list of 5 cells through a variable, then with a fixed number,
  and compare the two messages with the ones a plain list gives.
- Declare a constant list of three numbers, read its last cell into another constant, and
  print both; then set one of its cells from a variable and read the error.
- Take a range of a counted list, write a cell of the range, and print both.

Next: [What a duplicate costs](costs.md).
