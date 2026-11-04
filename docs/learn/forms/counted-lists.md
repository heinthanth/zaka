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
ကိန်းရှင် ရောင်များ သည် ["နီ"၊ "ဝါ"] ဖြစ်ပါစေ။
ကိန်းရှင် ရွေး သည် ရောင်များ ဖြစ်ပါစေ။
ရွေး ထဲသို့ "စိမ်း" ကို ထည့်ပါ။          // the first edit: a heap block of its own
ရောင်များ ကို ပြသပါ။
ရွေး ကို ပြသပါ။
```

```text
["နီ"၊ "ဝါ"]
["နီ"၊ "ဝါ"၊ "စိမ်း"]
```

In the Explorer, the heap view shows the literal as a static list, and the first heap block
appears at the add. A list built from values known when the program runs, such as
`[က၊ ခ]` with two variables, takes a heap block from the start. The same holds for text:
see [Text](../values/strings.md).

## Constants of text and lists

A constant that holds text or a plain list is static data in each build. Its value must be
known before the program runs, and the compiler then works out a cell read and the count
of it, so neither runs as the program runs:

```zaka
ကိန်းသေ ရက်များ သည် [၃၁၊ ၂၈၊ ၃၁] ဖြစ်ပါစေ။
ကိန်းသေ ဖေဖော်ဝါရီ သည် ရက်များ ၏ အကွက် ၁ ဖြစ်ပါစေ။
ဖေဖော်ဝါရီ ကို ပြသပါ။
ရက်များ ၏ အကွက်အရေအတွက် ကို ပြသပါ။
```

```text
၂၈
၃
```

A value that needs the program to run, such as a variable in a cell, is an error. Declare
such a list with `ကိန်းရှင်`:

```zaka-error ZK0606
ကိန်းရှင် n သည် ၃ ဖြစ်ပါစေ။
ကိန်းသေ l သည် [n၊ ၂] ဖြစ်ပါစေ။
```

```text
error[ZK0606]: a constant that holds text or a plain list must be known at compile time
```

## Counted lists

A *counted list* keeps its count for good: the count is part of its kind, and it is known
before the program runs. Its cells sit in the list itself, so it never needs a heap
block. A literal of known cells is copied in from static data in one step. Write the count first, then the kind of each cell, then `စာရင်း`:

```zaka
ကိန်းရှင် က သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၇၊ ၈] ဖြစ်ပါစေ။
က ကို ပြသပါ။
ခ ကို ပြသပါ။
က ကို စာရင်းအလွတ် ဟု ထားပါ။
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

The count is a whole number above zero, written as digits. A counted list declared with no
value starts with each cell at its default: `၀` for a number, `မမှန်` for a true/false
value, `ဗလာ` for an address. On a counted list, `စာရင်းအလွတ်` sets each cell back to its
default, and the count remains. `[]` is a list with no cells, so it does not fit a
counted list, save in its declaration, where each cell starts at its default; see [A list
with no cells in a counted list](#a-list-with-no-cells-in-a-counted-list).

## Tables of counted lists

In a table of counted lists, each count pairs with one `စာရင်း`, the inner ones first: in
`အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း`, the `၃` belongs to the
first `စာရင်း` (a row of 3 integers) and the `၂` to the second (2 such rows). Messages
write this kind as `isize[3][2]`.

```zaka
ကိန်းရှင် ဇယား သည် အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်ပါစေ။ // 2 rows of 3
ဇယား ၏ အကွက် ၁ ၏ အကွက် ၂ ကို ၉ ဟု ထားပါ။                                          // row 1, column 2
ဇယား ကို ပြသပါ။
ဇယား ၏ အကွက် ၁ ကို ပြသပါ။
```

```text
[[၀၊ ၀၊ ၀]၊ [၀၊ ၀၊ ၉]]
[၀၊ ၀၊ ၉]
```

The rows of a table of
counted lists sit one after another, with no gap: Part III walks them with an address, in
[Rows and cells through addresses](../pointers/structs.md#rows-and-cells-through-addresses).

## From a counted list to a plain list

A counted list turns into a plain list on its own: its cells are duplicated into a heap
block. A plain list never turns into a counted list on its own, since nothing promises its
count:

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

A counted and a plain list do not compare with `==` and `!=`, and neither do two counted
lists of different counts, such as `isize[3]` and `isize[2]`.

## Counted lists in functions

A counted list parameter or result takes the counted kind, `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း`.
Its count is part of the kind, so the function takes lists of that count alone. A plain
list parameter takes a counted list too, which is duplicated into a heap block first.

A counted result starts with each cell at its default, here `၀`:

```zaka
// a function that builds a counted list and gives it back
လုပ်ဆောင်ချက် နှစ်ထပ်များ သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အကွက် ၄ ခုရှိသော ကိန်းပြည့် စာရင်း ရလဒ် ကို ပြန်ပေးမည်။
၀ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ ရလဒ် ၏ အကွက် န ကို န * န ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် နှစ်ထပ်များ ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းများ ကို ပြသပါ။
```

```text
[၀၊ ၁၊ ၄၊ ၉]
```

The caller gets a list of its own, as with an assignment: here a plain list, which takes
the cells of the counted one.

## Slices of counted lists

A write into a plain list can move its cells, so it ends each [slice](../lists/slices.md)
of the list. A counted list never moves its cells, so a slice of it stands through a write
of a cell:

```zaka
ကိန်းရှင် က သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် s သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
က ၏ အကွက် ၀ ကို ၉ ဟု ထားပါ။
s ကို ပြသပါ။
```

```text
[၉၊ ၂၊ ၃]
```

On a counted list, whose count the compiler knows, a range of constant numbers past the end
is an error before the program runs:

```zaka-error ZK0605
ကိန်းရှင် l သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
l ၏ အကွက် ၂ မှစ၍ ၂ ကွက် ကို ပြသပါ။
```

## The size of a counted list

A counted list takes its count times the size of one cell, with no gaps. A plain list is
three words, whatever its count:

```zaka
အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း ၏ အရွယ်အစား ကို ပြသပါ။
ကိန်းပြည့် စာရင်း ၏ အရွယ်အစား ကို ပြသပါ။
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
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် န သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက် န ကို ပြသပါ။            // cells 0, 1 and 2 hold values; 3 does not exist
```

```text
error[ZK0710]: offset 3 is past the end of a block of 3 elements
```

With `--mem-check warn`, or **Warn** in the Explorer, the program runs and the VM
stops it, with the range the list allows:

```zaka-stop mem-check=warn
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် န သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက် န ကို ပြသပါ။            // cells 0, 1 and 2 hold values; 3 does not exist
```

```text
the program stopped: cell 3 is outside the list, whose cells run from 0 to 2
```

When the number comes from input, the VM's check is the one that catches it. When the
number is written in place, as a constant, Zaka reports it with another code, in each
mode. Counting from 1 starts at 1, not 0:

```zaka-error ZK0603
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
အမှတ်များ ၏ ၀ ခုမြောက်အကွက် ကို ပြသပါ။     // counting from 1 starts at 1
```

```text
error[ZK0603]: cell 0 is outside the list, whose cells run from 1 to 3
```

The cell number takes the whole operator expression after `အကွက်`, so this reads cell
`၁ + ၂`:

```zaka-error ZK0603
ကိန်းရှင် ကိန်းများ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀၊ ၂၀၊ ၃၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက် ၁ + ၂ ကို ပြသပါ။         // cell 3, not cell 1 plus 2
```

```text
error[ZK0603]: cell 3 is outside the list, whose cells run from 0 to 2
```

## A literal with the wrong count

```zaka-error ZK0419
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
```

```text
error[ZK0419]: the kind holds 3 cells, but the list has 2
```

## A list with no cells in a counted list

`[]` is a list with no cells. A counted list keeps its count, so `[]` fits it in its
declaration alone, where each cell starts at its default. Elsewhere, `စာရင်းအလွတ်` sets
each cell back to its default:

```zaka-error ZK0436
ကိန်းရှင် က သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၇၊ ၈၊ ၉] ဖြစ်ပါစေ။
က ကို [] ဟု ထားပါ။
```

```text
error[ZK0436]: [] is a list with no cells, and a counted list of isize[3] cannot take it here
```

## A counted list from a plain list

A plain list does not turn into a counted list on its own: nothing promises its count.

```zaka-error ZK0401
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။
```

```text
error[ZK0401]: expected isize[3], found isize[]
```

## Adding to a counted list

A counted list keeps its count for good, so `ထည့်ပါ` and `ဖျက်ပါ` do not take it:

```zaka-error ZK0437
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
အမှတ်များ ထဲသို့ ၇၀ ကို ထည့်ပါ။
```

```text
error[ZK0437]: isize[3] cannot grow or shrink here: "ထည့်ပါ" takes a plain list or text, "နောက်ဆုံးအကွက်ကို ဖျက်ပါ" a plain list, and "နောက်ဆုံးစာလုံးကို ဖျက်ပါ" text
```

Declare the list with no count, `ကိန်းပြည့် စာရင်း`, to add cells to it.

## Hands on

- Declare a counted list of five decimal numbers with a written count, set the middle one,
  and print the list.
- Read cell 5 of a counted list of 5 cells through a variable, then with a constant number,
  and compare the two messages with the ones a plain list gives.
- Declare a constant list of three numbers, read its last cell into another constant, and
  print both; then set one of its cells from a variable and read the error.
- Take a slice of a counted list, write a cell of the list, and print the slice.

Next: [What a duplicate costs](costs.md).
