---
title: Static data and counted lists
description: Where the cells of text and lists live - static data for a literal or a constant, a heap block from the first edit - and the counted list, whose count is part of its kind and whose cells sit in the list itself.
---

# Static data and counted lists

The text and the lists of Part I are managed: `စာသား` and `ကိန်းပြည့် စာရင်း`. The compiler
picks where their cells live, and moves them when it must. This page shows where that is,
and the one list kind that keeps its cells in place: the *counted list*.

## Where the cells live

A literal whose cells are known before the program runs sits in the program itself, as
*static data*: text, numbers, lists of these and lists of lists. A variable set from such a
literal points at the static data and asks the heap for nothing. Its first edit, a write,
an add or a remove, copies the cells into a heap block of its own, so the literal never
changes:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရောင်များ သည် ["နီ"၊ "ဝါ"] ဖြစ်ပါစေ။
ကိန်းရှင် ရွေး သည် ရောင်များ ဖြစ်ပါစေ။
ရွေး ထဲသို့ "စိမ်း" ကို ထည့်မည်။          // the first edit: a heap block of its own
ရောင်များ ကို ပြသမည်။
ရွေး ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
["နီ"၊ "ဝါ"]
["နီ"၊ "ဝါ"၊ "စိမ်း"]
```

In the Explorer, the heap view shows the literal as a static list, and the first heap block
appears at the add. A list built from values known when the program runs, such as
`[က၊ ခ]` with two variables, takes a heap block from the start. The same holds for text:
see [Text](../text/README.md).

## Constants of text and lists

A constant that holds text or a plain list is static data in each build. Its value must be
known before the program runs, and the compiler then works out a cell read and the count
of it, so neither runs as the program runs:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ ရက်များ သည် [၃၁၊ ၂၈၊ ၃၁] ဖြစ်ပါစေ။
ကိန်းသေ ဖေဖော်ဝါရီ သည် ရက်များ ၏ အကွက် ၁ ဖြစ်ပါစေ။
ဖေဖော်ဝါရီ ကို ပြသမည်။
ရက်များ ၏ အကွက်အရေအတွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၈
၃
```

A value that needs the program to run, such as a variable in a cell, is an error. Declare
such a list with `ကိန်းရှင်`:

```zaka-error ZK0606
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် n သည် ၃ ဖြစ်ပါစေ။
ကိန်းသေ l သည် [n၊ ၂] ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0606]: a constant that holds text or a plain list must be known at compile time
```

## Counted lists

A *counted list* keeps its count for good: the count is part of its kind, and it is known
before the program runs. Its cells sit in the list itself, so it never needs a heap
block. A literal of known cells is copied in from static data in one step. Write the count first, then the kind of each cell, then `စာရင်း`:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၇၊ ၈] ဖြစ်ပါစေ။
က ကို ပြသမည်။
ခ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၀၊ ၀၊ ၀]
[၇၊ ၈]
```

| Kind | Short name | Its cells |
| --- | --- | --- |
| `ကိန်းပြည့် စာရင်း` | `isize[]` | plain: their count comes with the value |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း` | `isize[3]` | counted: 3 cells for good, kept in the list itself |

The word `ခု` and a `၏` before `စာရင်း` are both optional, so these four kinds are the
same:

| Kind |
| --- |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း` |
| `အကွက် ၃ ရှိသော ကိန်းပြည့် စာရင်း` |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် ၏ စာရင်း` |
| `အကွက် ၃ ရှိသော ကိန်းပြည့် ၏ စာရင်း` |

The count is a whole number above zero, written as digits or worked out from constants
(see [A count from constants](#a-count-from-constants)). `ကနဦးတန်ဖိုး` gives a counted
list with each cell at the [initial value](../variables/README.md#the-initial-value) of
its kind: `၀` for a number, `မမှန်` for a true/false value. `[]` and `စာရင်းအလွတ်` are
lists with no cells, so neither fits a counted list; see [A list with no cells in a
counted list](#a-list-with-no-cells-in-a-counted-list).

## Filling the cells one at a time

A counted list declared with no value holds no cells to read until the program sets them.
The compiler follows each cell: the list counts as set once each cell is, through cell
numbers written in place or through a counted loop over each cell that writes the cell of its counter:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
က ၏ အကွက် ၀ ကို ၅ ဟု ထားမည်။
က ၏ အကွက် ၁ ကို ၆ ဟု ထားမည်။
က ကို ပြသမည်။
ကိန်းရှင် ခ သည် အကွက် ၄ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
၀ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ ခ ၏ အကွက် န ကို န * ၁၀ ဟု ထားမည်။    // each cell, 0 to 3
ခ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၅၊ ၆]
[၀၊ ၁၀၊ ၂၀၊ ၃၀]
```

A read of a cell that no write has reached is the error ZK0722. When the compiler cannot
tell which cells a write reaches, as with a cell number in a variable or a loop over some
of the cells, the read is the warning ZK0723, and the program still runs:

```zaka-warning ZK0723
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
၀ မှ ၁ အထိ န အနေဖြင့် ရေတွက်၍ က ၏ အကွက် န ကို န ဟု ထားမည်။     // cell 2 is never set
က ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0723]: "က" can have a cell with no value here; the check cannot tell which cells a write or a read reaches
```

Fill such a list with `ကနဦးတန်ဖိုး` or a literal first, or loop over each of its cells.

## A count from constants

A count can name constants and work with them, so one constant sets the size of several
lists. The compiler works the count out before the program runs, as it folds
constants: each operator and conversion it can work out, the size of a kind, a cell or a
field of a constant list or struct, and the cell count of a counted list are fine.

```zaka
ကိန်းသေ အရွယ် သည် ၃ ဖြစ်ပါစေ။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အကွက် အရွယ် ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အကွက် အရွယ် * ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
က ကို ပြသမည်။
ခ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၀၊ ၀၊ ၀]
[၀၊ ၀၊ ၀၊ ၀၊ ၀၊ ၀]
```

A variable does not work: its value is known when the program runs, after the size of the
list must be fixed.

```zaka-error ZK0227
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အရွယ် သည် ၃ ဖြစ်ပါစေ။
ကိန်းရှင် က သည် အကွက် အရွယ် ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0227]: "အရွယ်" is not a cell count; write a whole number above zero, in digits or worked out from constants
```

## Tables of counted lists

In a table of counted lists, each count pairs with one `စာရင်း`, the inner ones first: in
`အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း`, the `၃` belongs to the
first `စာရင်း` (a row of 3 integers) and the `၂` to the second (2 such rows). Messages
write this kind as `isize[3][2]`.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဂရစ် သည် အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။ // 2 rows of 3
ဂရစ် ၏ အကွက် ၁ ၏ အကွက် ၂ ကို ၉ ဟု ထားမည်။                                          // row 1, column 2
ဂရစ် ကို ပြသမည်။
ဂရစ် ၏ အကွက် ၁ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[[၀၊ ၀၊ ၀]၊ [၀၊ ၀၊ ၉]]
[၀၊ ၀၊ ၉]
```

The rows of a table of
counted lists sit one after another, with no gap: Part III walks them with an address, in
[Rows and cells through addresses](../pointers/structs.md#rows-and-cells-through-addresses).

## From a counted list to a plain list

A counted list turns into a plain list on its own: its cells are copied into a heap
block. A plain list never turns into a counted list on its own, since nothing promises its
count:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။
ခ ကို [၄၊ ၅] ဟု ထားမည်။
က ကို ပြသမည်။
ခ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၂၊ ၃]
[၄၊ ၅]
```

A counted and a plain list do not compare with `==` and `!=`, and neither do two counted
lists of different counts, such as `isize[3]` and `isize[2]`.

## Counted lists in functions

A counted list parameter or result takes the counted kind, `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း`.
Its count is part of the kind, so the function takes lists of that count alone. A plain
list parameter takes a counted list too, which is copied into a heap block first.

Here the loop sets each cell of the counted result before the return:

```zaka
// a function that builds a counted list and gives it back
လုပ်ဆောင်ချက် နှစ်ထပ်များ သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အကွက် ၄ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် အကွက် ၄ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
၀ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ အဖြေ ၏ အကွက် န ကို န * န ဟု ထားမည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် နှစ်ထပ်များ ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းများ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၀၊ ၁၊ ၄၊ ၉]
```

The caller gets a list of its own, as with an assignment: here a plain list, which takes
the cells of the counted one.

## The size of a counted list

A counted list takes its count times the size of one cell, with no gaps. A plain list is
three words, whatever its count:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
ကိန်းပြည့် စာရင်း တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၈
၂၄
```

## A cell number past a counted list

The count of a plain list comes when the program runs, so the VM checks each cell number.
A counted list has its count in its kind, and the compiler checks the cell numbers it can
follow. Here the number sits in a variable, and the compiler still works out that it is 3:

```zaka-error ZK0710
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် န သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက် န ကို ပြသမည်။            // cells 0, 1 and 2 hold values; 3 does not exist
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0710]: offset 3 is past the end of a block of 3 elements
```

This check is an error in each mode. When the number comes from input, the VM's check is
the one that catches it, and it stops the program. When the
number is written in place, as a constant, Zaka reports it with another code, in each
mode. Counting from 1 starts at 1, not 0:

```zaka-error ZK0603
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
အမှတ်များ ၏ ၀ ခုမြောက်အကွက် ကို ပြသမည်။     // counting from 1 starts at 1
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0603]: cell 0 is outside the list, whose cells run from 1 to 3
```

The cell number takes the whole operator expression after `အကွက်`, so this reads cell
`၁ + ၂`:

```zaka-error ZK0603
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀၊ ၂၀၊ ၃၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက် ၁ + ၂ ကို ပြသမည်။         // cell 3, not cell 1 plus 2
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0603]: cell 3 is outside the list, whose cells run from 0 to 2
```

## A literal with the wrong count

```zaka-error ZK0419
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0419]: the kind holds 3 cells, but the list has 2
```

## A list with no cells in a counted list

`[]` and `စာရင်းအလွတ်` are lists with no cells. A counted list keeps its count, so
neither fits it, in a declaration or elsewhere. `ကနဦးတန်ဖိုး` sets each cell to its
initial value:

```zaka-error ZK0436
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၇၊ ၈၊ ၉] ဖြစ်ပါစေ။
က ကို [] ဟု ထားမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0436]: a list with no cells does not fit the counted list isize[3]
```

## A counted list from a plain list

A plain list does not turn into a counted list on its own: nothing promises its count.

```zaka-error ZK0401
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0401]: expected isize[3], found isize[]
```

## Adding to a counted list

A counted list keeps its count for good, so `ထည့်မည်` and `ဖျက်မည်` do not take it:

```zaka-error ZK0437
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
အမှတ်များ ထဲသို့ ၇၀ ကို ထည့်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0437]: isize[3] cannot grow or shrink here: "ထည့်မည်" takes a plain list or text, "နောက်ဆုံးအကွက်ကို ဖျက်မည်" a plain list, and "နောက်ဆုံးစာလုံးကို ဖျက်မည်" text
```

Declare the list with no count, `ကိန်းပြည့် စာရင်း`, to add cells to it.

## Hands on

- Declare a counted list of five decimal numbers with a written count and
  `ကနဦးတန်ဖိုး`, set the middle one, and print the list. Then drop `ကနဦးတန်ဖိုး` and set
  each cell in a counted loop.
- Read cell 5 of a counted list of 5 cells through a variable, then with a constant number,
  and compare the two messages with the ones a plain list gives.
- Declare a constant list of three numbers, read its last cell into another constant, and
  print both; then set one of its cells from a variable and read the error.
- Take a range of a counted list, write a cell of the range, and print both.

Next: [What a duplicate costs](costs.md).
