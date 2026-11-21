---
title: Counted lists and storage
description: The two forms of list and the conversion between them, the count of a counted list, setting each cell, static data, how a list sits in RAM, lists in unsafe code, and pinned cells.
---

# Counted lists and storage

A list comes in two forms. A plain list (dynamic array) gets its count when the program
runs and keeps its cells in static data or in a heap block, while a counted list
(fixed-size array) has its count in its data kind (type) and keeps its cells in place.
This page states the rules that set the two forms apart: the count, setting each cell,
the conversion from one form to the other, static data, and how each form sits in RAM,
then the rules for lists in unsafe code. [Lists](lists.md) covers what the two forms
share: literals, cells, adding and removing cells, and ranges. The Learn book teaches
these ideas in [Static data and counted lists](../../learn/forms/counted-lists.md).

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

## Forms

```text
<cell kind> စာရင်း
အကွက် <count> ခုရှိသော <cell kind> စာရင်း
```

| Form | Burmese | Where its cells are |
| --- | --- | --- |
| plain | `စာရင်း` | in static data, or in a heap block that it owns or shares; the count comes when the program runs |
| counted | a written count, `အကွက် <n> ခုရှိသော` | inline: in the frame, the static block or the struct that contains the list; it never allocates |

- A plain list is three words, laid out as text is: the address of its first cell, its
  count, and the room of its heap block in octets (bytes). A literal of known values
  points at [static data](#static-data); each other literal is built, and its cells go to
  a new heap block. A set replaces the whole value, so the count can change.
- A counted list keeps its cells in place, and its count is part of its data kind, so
  `isize[3]` and `isize[2]` are different data kinds.
- A list literal with no counted list around it is a plain list:
  `ကိန်းရှင် l သည် [၁၊ ၂] ဖြစ်ပါစေ။` declares a plain list of two cells.
- A constant keeps the data kind its declaration gives: a plain list remains plain, and a
  written count makes a counted list.

A value converts from one form to the other on its own where it goes into a place: a
declaration, a set, an argument, a result, a struct field or a list cell. The example at
the top of the page sets a plain list from a counted one.

| From | To | What happens |
| --- | --- | --- |
| counted | plain | converts on its own: the cells are duplicated (copied) into a heap block |
| plain | counted | never on its own, because the count of a counted list is part of its data kind (ZK0401) |

```zaka-error ZK0401
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0401]: expected isize[3], found isize[]
```

## The count

```text
အကွက် <count> ခုရှိသော <cell kind> စာရင်း
```

The count must be a whole number above zero that the compiler knows (ZK0227). We can
write digits, or an expression that the compiler
[folds](../compiler/stages.md#one-sentence-through-each-stage) to an integer:

| Form | Example |
| --- | --- |
| a literal or a constant (`ကိန်းသေ`) | `၃`, `N` |
| the arithmetic, bitwise and shift operators, in each spelling | `N << ၂`, `N နှင့် ၂ ၏ ပေါင်းလဒ်` |
| a comparison or a conversion | `… ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်` |
| a size | `ဂရစ် တစ်ခု ၏ အရွယ်အစား` |
| a cell or a field of a constant list or struct | `L ၏ အကွက် ၁`, `P ၏ x` |
| the cell count of a constant plain list, or of each counted list (from its data kind) | `l ၏ အကွက်အရေအတွက်` |
| a call of a [comptime function](compile-time.md#comptime-functions), which runs while the compiler runs | |

The compiler works each value out in the data kinds the program gives it, with the same
math as the program, so a mistake shows up as it would when the program runs:

| Case | Code |
| --- | --- |
| a constant `B` of `၈ ဘစ် ကိန်းပြည့်`, then `B * ၂`, which overflows | ZK0601 |
| a division with zero | ZK0602 |
| a shift too far | ZK0407 |
| a cell outside a constant list | ZK0603 |

- A constant list can be read once each of its cells is known.
- A constant counts when it is declared before the count in its block, at the top of its
  file (above or below the count), or imported. A variable or a parameter of the same
  name hides the constant.
- A count cannot come from the value of a variable, a call of another function, or the
  count of text, because the size of a counted list is known before the program runs.
- A count that depends on itself, such as a struct whose count is its own size, is
  ZK0227.

## Setting each cell

A counted list gets its initial value (default value) from its cells: `ကနဦးတန်ဖိုး` sets
each cell to the initial value of its cell kind. A counted list declared with no value
counts as set once each of its cells is set ([Set before
read](declarations.md#set-before-read)). We can set the cells in two forms:

- with writes at constant cell numbers, one for each cell;
- with a counted loop that writes the cell of its counter, which sets each cell from the
  start of the loop to its end.

Inside such a loop, a read of a cell before the counter counts as set when the cells
before the start of the loop are set. A read that the check cannot prove set, as after a
write at a computed cell number, gets the warning ZK0723.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
၀ မှ ၂ အထိ န အနေဖြင့် ရေတွက်၍ က ၏ အကွက် န ကို န * ၁၀ ဟု ထားမည်။
က ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၀၊ ၁၀၊ ၂၀]
```

## Static data

A plain list literal whose cells are all known at compile time is *static data*: its
cells sit in a frozen static block of the program, as the u8s of a string literal do,
and the list points there with a room of 0. Known cells are constants, sizes, static
text, blank lists, and counted lists, structs and plain lists of known cells.

- Reads (a cell, the count, a print, a comparison, an each-cell loop) work as on each
  other plain list, and need no heap block.
- The first edit (a cell write, an add or a remove) first duplicates the cells into a new
  heap block, so the static data never changes. A remove lowers the count alone and
  duplicates nothing.
- A duplicate of the list shares the static block, as a duplicate of static text does.
- A literal with a value that is known when the program runs and not before, such as a
  variable, is built in a heap block.
- With `--fold none`, a literal of operations on constants is built when the program
  runs; the output is the same.
- A counted list keeps its cells in place, so it never points at static data. A counted
  literal of known cells is still a static list, and the counted list gets its cells
  from it in one step, in place of a write for each cell.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် l သည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် m သည် l ဖြစ်ပါစေ။
m ထဲသို့ ၄ ကို ထည့်မည်။
l ကို ပြသမည်။
m ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၂၊ ၃]
[၁၊ ၂၊ ၃၊ ၄]
```

### Constant lists

A `ကိန်းသေ` with a plain list in it (or text, or a list or struct with either inside) is
static data in each build, so its value must fold at compile time, whatever `--fold` is
set to. A value that does not fold is ZK0606, reported at the first cell that does not
fold. A call of a [comptime function](compile-time.md#comptime-functions) folds, since it
runs while the compiler runs. With constant propagation on, a cell read of such a
constant at a constant number, a field of a struct in it, and its count become
constants, and a constant number outside it is ZK0603.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ l သည် [၁၀၊ ၂၀၊ ၃၀] ဖြစ်ပါစေ။
ကိန်းသေ x သည် l ၏ အကွက် ၁ ဖြစ်ပါစေ။
x ကို ပြသမည်။
l ၏ အကွက်အရေအတွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၀
၃
```

```zaka-error ZK0606
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် n သည် ၃ ဖြစ်ပါစေ။
ကိန်းသေ l သည် [n၊ ၂] ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```zaka-error ZK0603
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ l သည် [၁၀၊ ၂၀၊ ၃၀] ဖြစ်ပါစေ။
l ၏ အကွက် ၅ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## How a list is stored

The cells of a list sit next to each other in one stretch of RAM, in the order of their
numbers, with no gaps. A counted list is that stretch, inline, so its size is its count
times the size of one cell. A counted list of counted lists stores its rows one after
another, so a grid is one long stretch with the rows end to end.

```text
  grid: isize[3][2], 6 cells of 8 octets = 48 octets

  octet    0      8      16     24     32     40     48
           +------+------+------+------+------+------+
           |  1   |  2   |  3   |  4   |  5   |  6   |
           +------+------+------+------+------+------+
  cell     [0][0] [0][1] [0][2] [1][0] [1][1] [1][2]
           '------- row 0 ------''------- row 1 ------'

  one past the end: octet 48, not a cell
```

A plain list is three words, laid out as text is, and its cells sit in static data or in
a heap block:

```text
  list: isize[], with the cells [1, 2, 3]

  the list, 24 octets                       its heap block, 24 octets
  +-----------+-----------+-----------+     +------+------+------+
  |  address  |  count 3  |  room 24  |     |  1   |  2   |  3   |
  +-----+-----+-----------+-----------+     +------+------+------+
        |                                   ^
        '-----------------------------------'
```

- The room is the size of the heap block in octets. A room of 0 marks cells that the list
  does not own: static cells, the cells of a caller read through a parameter, or no cells
  at all.
- Each heap block counts the lists that share it. A duplicate of the list adds one to the
  count, and a list that is let go takes one off; the block is freed when the count
  reaches 0.
- A write into a list with a room of 0, or into a block whose count is above 1, first
  duplicates the cells into a new block, so a plain list never writes cells that another
  list reads.
- An add into a full block moves the cells to a block twice the size, with room for four
  cells at the least; a remove keeps the block and its room.
- Each list in the cells of a plain list is a separate value: a plain list of plain lists
  has three words for each row, and each row has its own block.

The size of a list, through `အရွယ်အစား`, shows how each form sits in RAM:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
အကွက် ၃ ခုရှိသော ၃၂ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
အကွက် ၅ ခုရှိသော မှန်မှားတန်ဖိုး စာရင်း တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
ကိန်းပြည့် စာရင်း တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၈
၁၂
၅
၂၄
၂၄
```

## Lists in unsafe code

[Unsafe Zaka](unsafe.md) states the rules of pointers; this section gives the ones that
touch lists. Each example runs inside an unsafe block.

The address of a cell is a pointer to the cell, so we can reach the cell after it with
pointer arithmetic. In a grid of counted lists, the cell after the last cell of a row is
the first cell of the next row. The rows of a plain list each sit in a separate block,
and a step from one into the next stops the program.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဂရစ် သည် အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (၁၊ ၂ နှင့် ၃ တို့ပါသော စာရင်း) နှင့် (၄၊ ၅ နှင့် ၆ တို့ပါသော စာရင်း) တို့ပါသော စာရင်း ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် အစ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဂရစ် ၏ အကွက် ၀ ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် အတန်း သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဂရစ် ၏ အကွက် ၁ ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။
အတန်း - အစ ကို ပြသမည်။                      // the second row starts 3 cells on
အစ + ၄ ၏ တန်ဖိုး ကို ပြသမည်။                  // row 1, column 1
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၃
၅
```

The address of a list points at the whole list. For a counted list, the data kind of the
address includes the count: `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ`, or `isize[3]*` in
messages. Adding 1 to it moves on one whole list, which for a counted list of counted
rows is one row. The address of a plain list, `isize[]*`, points at its three words, and
an `isize[]*` does not accept an `isize[3]*` (ZK0401).

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဂရစ် သည် အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (၁၊ ၂ နှင့် ၃ တို့ပါသော စာရင်း) နှင့် (၄၊ ၅ နှင့် ၆ တို့ပါသော စာရင်း) တို့ပါသော စာရင်း ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် အတန်း သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဂရစ် ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။
အတန်း ၏ တန်ဖိုး ကို ပြသမည်။
အတန်း ကို အတန်း + ၁ ဟု ထားမည်။                 // one whole row further
အတန်း ၏ တန်ဖိုး ကို ပြသမည်။
အတန်း ၏ အကွက် ၂ ကို ပြသမည်။                     // a cell, through the row address
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၂၊ ၃]
[၄၊ ၅၊ ၆]
၆
```

The address of one cell is a plain pointer to the cell kind. The `အကွက်` forms work on a
list and on a pointer to a list, through one level of pointer.

## Edits and pinned cells

When we take the address of a cell of a plain list, the list *pins* its block: the list
first gets a block that it alone owns (the operation `own`), and the block is marked (the
operation `pin`) so that it is never shared again. A list duplicated from a pinned list
gets its own cells at once, so a write through the pointer never reaches the duplicate.

A pin does not keep the cells in place. An add can move them to a larger block, and a
write into the whole list replaces them, so a pointer taken before the edit is left
pointing at freed cells. The checker reports such a use as ZK0704, a warning in unsafe
code unless `--lint-allow mem` or `--lint-error mem` sets another level ([Compiler
checks](../compiler/checks.md)), and the VM stops the program at the access. The cells
of a counted list never move.

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။
ပ ၏ တန်ဖိုး ကို ၉ ဟု ထားမည်။
က ကို ပြသမည်။
ခ ကို ပြသမည်။                      // a separate duplicate: unchanged
က ထဲသို့ ၄ ကို ထည့်မည်။             // the cells move to a larger block
ပ ၏ တန်ဖိုး ကို ပြသမည်။              // ZK0704, and the VM stops here
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၉၊ ၂၊ ၃]
[၁၊ ၂၊ ၃]
the program stopped: access at 0x101030 inside a freed block
```

```zaka-error ZK0704 lint-error=mem
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။
က ထဲသို့ ၄ ကို ထည့်မည်။
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## Errors at a glance

| Code | Mistake |
| --- | --- |
| ZK0227 | a count that is not a whole number above zero, or that names no constant that works out |
| ZK0401 | a plain list given where a counted list is wanted, or an `isize[3]*` given where an `isize[]*` is wanted |
| ZK0601, ZK0602, ZK0407 | an overflow, a division with zero or a shift too far while the compiler works out a count |
| ZK0603 | a constant cell number outside a constant or counted list |
| ZK0606 | a constant plain list whose value needs the program to run |
| ZK0704 | in unsafe code, a pointer into a plain list used after an add or a write that can move its cells |
| ZK0723 | a warning: a read of a counted list whose cells the check cannot prove set |

[Lists](lists.md#errors-at-a-glance) lists the other mistakes with lists, and [Error
messages](../reference/errors.md) has the text of each message.
