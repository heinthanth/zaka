---
title: Static text, fixed lists and the heap
description: Text and lists come in three forms - plain, fixed and heap. Fixed text is static text, known at compile time; a fixed list keeps its count; heap text and heap lists own a heap block.
---

# Static text, fixed lists and the heap

The text and the lists of Part I are the *plain* form: `စာသား` and `ကိန်းပြည့် စာရင်း`.
The compiler picks where their cells live, and moves them when it must. Two more forms
make a promise each:

| Form | Text | List | Promise |
| --- | --- | --- | --- |
| plain | `စာသား` | `ကိန်းပြည့် စာရင်း` | none: the form to reach for first |
| fixed | `စာသားသေ` | `ကိန်းပြည့် စာရင်းသေ`, or a written count | the count never changes, and the cells never move |
| heap | `စာသားရှင်` | `ကိန်းပြည့် စာရင်းရှင်` | the cells sit in a heap block of their own, from the start |

`သေ` reads as "fixed", as in `ကိန်းသေ`, and `ရှင်` as "living", as in `ကိန်းရှင်`.

## Fixed and heap text

`စာသားသေ` ("fixed text") is text known before the program runs, which never changes:
Rust's `&'static str`. `စာသားရှင်` ("living text") sits on the heap, in a block of its
own, from the start. Each of the three converts to the
others, so one can go where another is wanted:

```zaka
ကိန်းရှင် က သည် စာသားသေ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မင်္ဂလာပါ" ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် စာသားရှင် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။     // a heap block of its own
ခ ကို ပြသပါ။
```

```text
မင်္ဂလာပါ
```

A literal such as `"မင်္ဂလာပါ"` sits in the program itself, and plain `စာသား` points at
it. Heap text takes a duplicate on the heap instead. The
[Docs](../../manual/language/text.md#forms) state the rules in full.

Plain and heap text change in place: a [character
write](../values/strings.md#changing-a-character) and an
[add](../values/strings.md#adding-to-text) each take them. Fixed text keeps its promise,
and the compiler refuses both:

```zaka
ကိန်းရှင် ခ သည် စာသားရှင် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မ" ဖြစ်ပါစေ။
ခ ထဲသို့ "ြ" ကို ထည့်ပါ။
ခ ၏ စာလုံး ၀ ကို 'က' ဟု ထားပါ။
ခ ကို ပြသပါ။
```

```text
ကြ
```

```zaka-error ZK0438
ကိန်းရှင် က သည် စာသားသေ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မင်္ဂလာပါ" ဖြစ်ပါစေ။
က ထဲသို့ "!" ကို ထည့်ပါ။
```

```text
error[ZK0438]: textf cannot change: a character write or an add takes text (စာသား) or heap text (စာသားရှင်)
```

A set still gives fixed text a whole new value, of another length too:
`က ကို "ဟယ်လို" ဟု ထားပါ။` is fine. But each value of fixed text must be known before
the program runs: a literal, a constant, or a template literal whose slots hold constants.
Text built from a variable is an error:

```zaka-error ZK0608
ကိန်းရှင် i သည် ကိန်းပြည့် ၃ ဖြစ်ပါစေ။
ကိန်းရှင် က သည် စာသားသေ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "အဆင့် {i}" ဖြစ်ပါစေ။
```

```text
error[ZK0608]: fixed text ("စာသားသေ") must be known at compile time
```

Give such text the kind `စာသား`.

## Fixed lists

A *fixed list* keeps its count for good: the count is part of its kind, and it is known
before the program runs. Its cells sit in the list itself, so it never needs a heap
block. Write the count first, then the kind of each cell, then `စာရင်း`; or write
`စာရင်းသေ` ("fixed list") in place of `စာရင်း`, and the value gives the count:

```zaka
ကိန်းရှင် က သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် စာရင်းသေ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၇၊ ၈] ဖြစ်ပါစေ။
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
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း` | `isize[3]` | fixed: 3 cells for good, kept in the list itself |
| `ကိန်းပြည့် စာရင်းသေ` | `isize[3]`, from its value | fixed, with the count its value gives |
| `ကိန်းပြည့် စာရင်းရှင်` | `isize[]~` | heap: as plain, and it owns its heap block from the start |

The word `ခု` and a `၏` before `စာရင်း` are both optional, so these four kinds are the
same:

| Kind |
| --- |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း` |
| `အကွက် ၃ ရှိသော ကိန်းပြည့် စာရင်း` |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် ၏ စာရင်း` |
| `အကွက် ၃ ရှိသော ကိန်းပြည့် ၏ စာရင်း` |

The count is a whole number above zero, written as digits. A fixed list declared with no
value starts with each cell at its default: `၀` for a number, `မမှန်` for a true/false
value, `ဗလာ` for an address. On a fixed list, `စာရင်းအလွတ်` sets each cell back to its
default, and the count remains. `[]` is a list with no cells, so it does not fit a fixed
list, save in its declaration, where each cell starts at its default; see [A list with no
cells in a fixed list](#a-list-with-no-cells-in-a-fixed-list).

## Tables of fixed lists

In a table of fixed lists, each count pairs with one `စာရင်း`, the inner ones first: in
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

`ကိန်းပြည့် စာရင်းသေ စာရင်းသေ` takes both counts from the value. The rows of a table of
fixed lists sit one after another, with no gap: Part III walks them with an address, in
[Rows and cells through addresses](../pointers/structs.md#rows-and-cells-through-addresses).

## Heap lists

A *heap list*, `စာရင်းရှင်` ("living list"), works as a plain list does: it grows and
shrinks a cell at a time, and its count comes with its value. It owns a heap block from
its first cell on, even for a literal, so a write never has to move its cells first:

```zaka
ကိန်းရှင် အမည်များ သည် စာသား စာရင်းရှင် တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် အမည် သည် "မောင်" ဖြစ်ပါစေ။
အမည်များ ထဲသို့ အမည် ကို ထည့်ပါ။           // a duplicate of the text
အမည်များ ထဲသို့ "မြ" ကို ထည့်ပါ။
အမည် ကို "ဇော်" ဟု ထားပါ။
အမည်များ ကို ပြသပါ။
```

```text
["မောင်"၊ "မြ"]
```

## From one form to another

A fixed list turns into a plain or a heap list on its own: its cells are duplicated into a
heap block. Plain and heap lists turn into each other. A plain or a heap list never turns
into a fixed list on its own, since nothing promises its count:

```zaka
ကိန်းရှင် က သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် ကိန်းပြည့် စာရင်းရှင် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ခ ဖြစ်ပါစေ။
ခ ကို [၄၊ ၅] ဟု ထားပါ။
က ကို ပြသပါ။
ခ ကို ပြသပါ။
ဂ ကို ပြသပါ။
```

```text
[၁၊ ၂၊ ၃]
[၄၊ ၅]
[၁၊ ၂၊ ၃]
```

A plain and a heap list of one cell kind compare with `==` and `!=`. A fixed and a plain
list do not, and neither do two fixed lists of different counts, such as `isize[3]` and
`isize[2]`.

## Fixed lists in functions

A fixed list parameter or result takes the fixed kind, `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း`.
Its count is part of the kind, so the function takes lists of that count alone. A plain
list parameter takes a fixed list too, which is duplicated into a heap block first.

A fixed result starts with each cell at its default, here `၀`:

```zaka
// a function that builds a fixed list and gives it back
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
the cells of the fixed one.

## Slices of fixed and heap lists

A write into a plain list can move its cells, so it ends each [slice](../lists/slices.md)
of the list. A fixed list never moves its cells, and a heap list moves them on an add or a
remove alone, so a slice of either stands through a write of a cell:

```zaka
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်းသေ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် စာရင်းရှင် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် s သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
ကိန်းရှင် t သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ခ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
က ၏ အကွက် ၀ ကို ၉ ဟု ထားပါ။
ခ ၏ အကွက် ၀ ကို ၈ ဟု ထားပါ။
s ကို ပြသပါ။
t ကို ပြသပါ။
```

```text
[၉၊ ၂၊ ၃]
[၈၊ ၂၊ ၃]
```

An add or a remove ends each slice of a heap list, as it does for a plain list:

```zaka-error ZK0704
ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်းရှင် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် s သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် l ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
l ထဲသို့ ၄ ကို ထည့်ပါ။
s ကို ပြသပါ။
```

On a fixed list, whose count the compiler knows, a range of constant numbers past the end
is an error before the program runs:

```zaka-error ZK0605
ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်းသေ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
l ၏ အကွက် ၂ မှစ၍ ၂ ကွက် ကို ပြသပါ။
```

## The size of a fixed list

A fixed list takes its count times the size of one cell, with no gaps. A plain list is
three words, whatever its count:

```zaka
အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း ၏ အရွယ်အစား ကို ပြသပါ။
ကိန်းပြည့် စာရင်း ၏ အရွယ်အစား ကို ပြသပါ။
```

```text
၄၈
၂၄
```

## A cell number past a fixed list

The count of a plain list comes when the program runs, so the VM checks each cell number.
A fixed list has its count in its kind, and the compiler checks the cell numbers it can
follow. Here the number sits in a variable, and the compiler still works out that it is 3:

```zaka-error ZK0710
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်းသေ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် န သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက် န ကို ပြသပါ။            // cells 0, 1 and 2 hold values; 3 does not exist
```

```text
error[ZK0710]: offset 3 is past the end of a block of 3 elements
```

With `--mem-check warn`, or **Warn** in the Explorer, the program runs and the VM
stops it, with the range the list allows:

```zaka-stop mem-check=warn
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်းသေ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
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
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်းသေ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
အမှတ်များ ၏ ၀ ခုမြောက်အကွက် ကို ပြသပါ။     // counting from 1 starts at 1
```

```text
error[ZK0603]: cell 0 is outside the list, whose cells run from 1 to 3
```

The cell number takes the whole operator expression after `အကွက်`, so this reads cell
`၁ + ၂`:

```zaka-error ZK0603
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်းသေ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀၊ ၂၀၊ ၃၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
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

## A list with no cells in a fixed list

`[]` is a list with no cells. A fixed list keeps its count, so `[]` fits it in its
declaration alone, where each cell starts at its default. Elsewhere, `စာရင်းအလွတ်` sets
each cell back to its default:

```zaka-error ZK0436
ကိန်းရှင် က သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၇၊ ၈၊ ၉] ဖြစ်ပါစေ။
က ကို [] ဟု ထားပါ။
```

```text
error[ZK0436]: [] is a list with no cells, and a fixed list of isize[3] cannot take it here
```

## A fixed list from a plain list

A plain list does not turn into a fixed list on its own: nothing promises its count.

```zaka-error ZK0401
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။
```

```text
error[ZK0401]: expected isize[3], found isize[]
```

## A list kind with no count

`စာရင်းသေ` makes a fixed list whose count its value gives, so it needs a value:

```zaka-error ZK0421
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်းသေ တစ်ခု ဖြစ်ပါစေ။
```

```text
error[ZK0421]: this list kind has no cell count, and nothing here gives one
```

## Adding to a fixed list

A fixed list keeps its count for good, so `ထည့်ပါ` and `ဖျက်ပါ` do not take it:

```zaka-error ZK0437
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
အမှတ်များ ထဲသို့ ၇၀ ကို ထည့်ပါ။
```

```text
error[ZK0437]: isize[3] cannot grow or shrink here: "ထည့်ပါ" takes a plain or a heap list or text, and "ဖျက်ပါ" a plain or a heap list
```

Declare the list with no count, `ကိန်းပြည့် စာရင်း`, to add cells to it.

## Hands on

- Declare a fixed list of five decimal numbers with a written count, set the middle one,
  and print the list.
- Read cell 5 of a fixed list of 5 cells through a variable, then with a constant number,
  and compare the two messages with the ones a plain list gives.
- Build a heap list of names with a loop, give it to a plain list variable, and print both.
- Take a slice of a fixed list, write a cell of the list, and print the slice.

Next: [What a duplicate costs](costs.md).
