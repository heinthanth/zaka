---
title: Lists
description: The list kinds, literals, cell numbers, adding and removing cells, lists of lists, passing and comparing, ranges, then the plain and counted forms, static data, how a list is stored, addresses in marked code, and the mistakes with cell numbers.
---

# Lists

A list holds values of one kind, next to each other in RAM. Each value sits in a *cell*,
and a program reaches a cell through its number. A list comes in two [forms](#forms): a
plain list takes its count from its value when the program runs, and a counted list holds
its count in its kind, known before the program runs. A set gives a plain list a new
count, and a plain list [grows and shrinks](#adding-and-removing-cells) a cell at a time. The Learn book starts in small steps with
[Lists](../../learn/lists/README.md).

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက် ၀ ကို ၇၀ ဟု ထားမည်။
အမှတ်များ ၏ အကွက် ၂ ကို ၉၀ ဟု ထားမည်။
အမှတ်များ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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

- The count is a whole number above zero (ZK0227): digits, or each expression the
  compiler [folds](../compiler/stages.md#one-sentence-through-each-stage) to an integer. That takes literals, constants
  (`ကိန်းသေ`), the arithmetic, bitwise and shift operators in each spelling (`N << ၂`,
  `N နှင့် ၂ ၏ ပေါင်းလဒ်`), comparisons, conversions (`… ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော
  ရလဒ်`), sizes (`ဇယား တစ်ခု ၏ အရွယ်အစား`), cells and fields of constant lists and structs
  (`L ၏ အကွက် ၁`, `P ၏ x`), the cell count of a constant plain list, and the cell count of
  each counted list (`l ၏ အကွက်အရေအတွက်`, from its kind), each in the kinds the program
  gives them, with the folder's own math: a constant `B` of `၈ ဘစ် ကိန်းပြည့်` makes `B * ၂`
  overflow (ZK0601) as it would in the program, a division with zero is ZK0602, a shift
  too far ZK0407 and a cell outside a constant list ZK0603. A constant list is read when
  each of its cells is known. A constant counts when it is declared before the count in
  its block, at the top of its file (above or below the count), or imported. A count
  cannot come from a variable's value, a call, or the count of text, because the size of a
  counted list is known before the program runs; a variable or a parameter of the same
  name hides a constant. A count that waits on itself, such as a struct whose count is its
  own size, is ZK0227.
- The cell kind is a kind of each sort: a number, a true/false value, a character, text, a
  pointer, a struct, or a list.
- A list declared with no value holds no cells and no count: it is unset until the
  program sets it ([Set before read](statements.md#set-before-read)). A plain list starts
  with no cells when its value is `[]` or `ကနဦးတန်ဖိုး`.
- A counted list takes its [initial value](kinds.md#initial-values) from its cells:
  `ကနဦးတန်ဖိုး` sets each cell to the initial value of its kind, `၀` for a number, `မမှန်`
  for a true/false value, the null address for a pointer.
- A counted list declared with no value counts as set once each cell is set: cells at
  constant positions, or a counted loop that writes the cell of its counter, which sets
  each cell from its start to its end. Inside such a loop, a read of a cell before the
  counter is set when the cells before the start of the loop are. A read the check cannot
  prove set, as after a write at a computed position, is the warning ZK0723
  ([Set before read](statements.md#set-before-read)).

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အကွက် ၂ ခုရှိသော ဒဿမကိန်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အကွက် ၃ ရှိသော မှန်မှားတန်ဖိုး ၏ စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် စာလုံး စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် 'က'၊ 'ခ' တို့ပါသော စာရင်း ဖြစ်ပါစေ။
က ကို ပြသမည်။
ခ ကို ပြသမည်။
ဂ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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
and `[]` are lists with no cells, which a counted list cannot hold.

```text
၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း
[၇၀၊ ၈၅၊ ၉၀]
စာရင်းအလွတ်
[]
```

- A literal of either form can take its kind after it, `ဟူသော <kind> တစ်<word>`, as in
  `[၁၊ ၂] ဟူသော ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု`: the literal has that list kind, plain or
  counted, and each cell takes its cell kind. A kind that is not a list is ZK0222.
- The number of cells in a literal must equal the count of the counted list it fills
  (ZK0419). A plain list takes a literal of each count.
- `စာရင်းအလွတ်` and `[]` can take a kind after them too, as in
  `[] ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု`. Without one, each needs a list kind from the place
  it stands in; standing alone, neither has one (ZK0420).
- On a plain list, `စာရင်းအလွတ်` and `[]` are a list with no cells.
- On a counted list, `စာရင်းအလွတ်` and `[]` are ZK0436 in each place, the declaration
  included: a counted list holds each of its cells. `ကနဦးတန်ဖိုး` sets each cell to the
  initial value of its kind.
- A cell is a notation expression. A bracket list and a struct literal in the notation
  form are notation expressions, so each nests as written, as in `[[၁၊ ၂]၊ [၃၊ ၄]]`. A
  clause, or a literal in the `တို့ပါသော စာရင်း` form, goes in parentheses when it is a
  cell of another literal.

```zaka-error ZK0419
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0419]: the kind holds 3 cells, but the list has 2
```

```zaka-error ZK0420
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စာရင်းအလွတ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0420]: a blank list needs a list kind from its place
```

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၇၊ ၈၊ ၉] ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၄၊ ၅] ဖြစ်ပါစေ။
က ကို [] ဟု ထားမည်။
ခ ကို ကနဦးတန်ဖိုး ဟု ထားမည်။
က ကို ပြသမည်။
ခ ကို ပြသမည်။
က ၏ အကွက်အရေအတွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[]
[၀၊ ၀]
၀
```

```zaka-error ZK0436
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၇၊ ၈၊ ၉] ဖြစ်ပါစေ။
က ကို [] ဟု ထားမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0436]: a list with no cells does not fit the counted list isize[3]
```

## Cells

| Written | Cell |
| --- | --- |
| `အမှတ်များ ၏ အကွက် ၀` | the first (counted from 0, as programs count) |
| `အမှတ်များ ၏ ၁ ခုမြောက်အကွက်` | the first (counted from 1, as people count) |
| `အမှတ်များ ၏ အကွက် ၂` | the third |
| `အမှတ်များ ၏ ၃ ခုမြောက်အကွက်` | the third |
| `အမှတ်များ ၏ အကွက်အရေအတွက်` | the number of cells, a `လက္ခဏာမဲ့ ကိန်းပြည့်`: a constant for a counted list, known when the program runs for a plain list |

A cell is a place: it can be read, printed and assigned to, and marked code can take its
address ([Unsafe Zaka](unsafe.md)). The cell number is an integer (ZK0423), and the value
in front of `၏` has to be a list or a pointer to one (ZK0422).

```zaka-error ZK0422
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀ ဖြစ်ပါစေ။
အမှတ် ၏ အကွက် ၀ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0422]: isize is not a list, so it has no cells
```

An [each-cell loop](statements.md#loops), `အမှတ်များ ၏ အကွက်တိုင်း ကို အမှတ် အနေဖြင့်
ယူ၍ ...`, reads each cell in turn into a variable of its own. A list of pointers is walked
as well: each round reads one address. A value that is no list gives ZK0434.

### The cell number takes the whole expression

The number after `အကွက်` is read as far as the operator expression goes, just as
`ပ + ၁ ၏ တန်ဖိုး` reads through `ပ + ၁`. To use a cell inside a sum, put the cell in
parentheses.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀၊ ၂၀၊ ၃၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက် ၀ + ၁ ကို ပြသမည်။          // the cell number is 0 + 1
(ကိန်းများ ၏ အကွက် ၀) + ၁ ကို ပြသမည်။            // the first cell, plus 1
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၀
၁၁
```

The word `အကွက်အရေအတွက်` is a single word. Written with a space, `အကွက် အရေအတွက်` is the
cell numbered with a variable named `အရေအတွက်`.

A counted loop visits each cell; the cell count, in parentheses, gives the last number:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၄၊ ၈၊ ၁၅၊ ၁၆ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
၀ မှ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၁ အထိ န အနေဖြင့် ရေတွက်၍ ကိန်းများ ၏ အကွက် န ကို (ကိန်းများ ၏ အကွက် န) * ၂ ဟု ထားမည်။
ကိန်းများ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၈၊ ၁၆၊ ၃၀၊ ၃၂]
```

A list with no cells makes `အကွက်အရေအတွက် - ၁` stop the program, since the count cannot go
below 0: see [A list with no cells and the last number](../../learn/lists/mistakes.md#a-list-with-no-cells-and-the-last-number).

## Adding and removing cells

```text
<list> ထဲသို့ <value> ကို ထည့်မည်။
<list> ၏ အကွက် <number> ၏ အရှေ့တွင် <value> ကို ထည့်မည်။
<list> ၏ <number> ခုမြောက်အကွက် ၏ အရှေ့တွင် <value> ကို ထည့်မည်။
<list> ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။
```

`ထည့်မည်` ("please add") puts the value in a new last cell of the list, or, with
`၏ အရှေ့တွင်` ("in front of"), in front of the cell at the number: the value takes that
number, and the cells from there on move one cell on. `ဖျက်မည်` ("please remove") takes the
last cell off. No statement gives a value back.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [] ဖြစ်ပါစေ။
၁ မှ ၄ အထိ န အနေဖြင့် ရေတွက်၍ က ထဲသို့ န * ၁၀ ကို ထည့်မည်။
က ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။
က ကို ပြသမည်။
က ၏ အကွက်အရေအတွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၀၊ ၂၀၊ ၃၀]
၃
```

- The list is a place that can be set: a variable, a field, a cell of another list, or,
  in marked code, `ပ ၏ တန်ဖိုး` through a pointer `ပ`. A constant is ZK0306, as for a set.
  A pointer to a list is no list: add to `ပ ၏ တန်ဖိုး`, not to `ပ`.
- The list is a plain list. A counted list keeps its count: an add or a remove on it is
  ZK0437. An add to text is the [add to
  text](text.md#changing-text), with rules of its own, and text loses its last character
  through `နောက်ဆုံးစာလုံး`; `နောက်ဆုံးအကွက်` on text is ZK0437.
- The number in front of `၏ အရှေ့တွင်` counts from 0 for `အကွက် n` and from 1 for
  `n ခုမြောက်အကွက်`, as a cell read does, and is an integer (ZK0423). It can also be the
  count, which puts the value at the end; a larger number stops the program: "no value
  can go in front of 3: the places run from 0 to 2, and 2 is the end". `၏ အရှေ့တွင်`
  after a value that is no cell, and no character of text, is ZK0439.
- The value has the cell kind (ZK0401), and it goes into the list as an assigned value
  goes into a place: text, lists and structs are copied, and a shared object is shared
  ([Values and objects](objects.md)).
- The value is worked out before the list changes: `က ထဲသို့ က ၏ အကွက်အရေအတွက် ကို
  ထည့်မည်။` adds the count from before the add.
- `နောက်ဆုံးအကွက်` ("last cell") is read right after `၏` alone, and `ကို ဖျက်မည်` must
  follow it (ZK0201): it is no expression, so the last cell is read through its number.
- A list whose block is shared, with a duplicate or with a caller (see [How a list is
  stored](#how-a-list-is-stored)), first takes a block of its own on its first add, as on
  its first write, so the other list never changes. A remove on a list with a room of 0
  lowers the count and leaves the shared cells unchanged: it duplicates nothing.
- A full block moves to a block twice its size, four cells at the least.
- A remove on a list with no cells stops the program: "the list holds no cells, so it has
  no last cell to remove" (see [When a program stops](../vm/traps.md#all-the-traps)).
- A remove drops the value in the last cell when the list holds its block alone: text
  in it is let go.
- An add can move the cells to a larger block, so a pointer that marked code took into
  the cells can be left pointing at freed cells; see [Edits and pinned
  cells](#edits-and-pinned-cells).

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဇယား သည် ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [[၁]၊ [၂၊ ၃]] ဖြစ်ပါစေ။
ဇယား ၏ အကွက် ၀ ထဲသို့ ၅ ကို ထည့်မည်။
ဇယား ထဲသို့ [၆] ကို ထည့်မည်။
ဇယား ၏ အကွက် ၁ ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။
ဇယား ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[[၁၊ ၅]၊ [၂]၊ [၆]]
```

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၀၊ ၃၀] ဖြစ်ပါစေ။
က ၏ အကွက် ၁ ၏ အရှေ့တွင် ၂၀ ကို ထည့်မည်။      // in front of ၃၀
က ၏ ၁ ခုမြောက်အကွက် ၏ အရှေ့တွင် ၀ ကို ထည့်မည်။   // at the start
က ၏ အကွက် ၄ ၏ အရှေ့တွင် ၄၀ ကို ထည့်မည်။      // ၄ is the count: at the end
က ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၀၊ ၁၀၊ ၂၀၊ ၃၀၊ ၄၀]
```

```zaka-error ZK0439
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၀၊ ၃၀] ဖြစ်ပါစေ။
က ၏ အရှေ့တွင် ၅ ကို ထည့်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0439]: "အရှေ့တွင်" follows a cell of a list or a character of text
```

```zaka-error ZK0437
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။
က ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0437]: isize[2] cannot grow or shrink here: "ထည့်မည်" takes a plain list or text, "နောက်ဆုံးအကွက်ကို ဖျက်မည်" a plain list, and "နောက်ဆုံးစာလုံးကို ဖျက်မည်" text
```

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် စာသား စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [] ဖြစ်ပါစေ။
က ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဇယား သည် အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။ // 2 rows of 3
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

Nested literals put each inner literal in parentheses. In a kind with no counts, each list
is plain and takes its count from the value:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဇယား သည် ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (၁၊ ၂ တို့ပါသော စာရင်း)၊ (၃၊ ၄ တို့ပါသော စာရင်း)၊ (၅၊ ၆ တို့ပါသော စာရင်း) တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ဇယား ကို ပြသမည်။
ဇယား ၏ ၃ ခုမြောက်အကွက် ၏ ၁ ခုမြောက်အကွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[[၁၊ ၂]၊ [၃၊ ၄]၊ [၅၊ ၆]]
၅
```

## Passing and comparing

A list is a value, like an integer. Setting one list from another gives a list of its
own, and the two lists stand apart afterwards. The same holds for a list passed as an
argument, since a parameter is a new variable:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မူရင်း သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် ပွား သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
ပွား ကို မူရင်း ဟု ထားမည်။              // the two share one block until a write
ပွား ၏ အကွက် ၀ ကို ၁၀၀ ဟု ထားမည်။        // this write first copies the cells
မူရင်း ကို ပြသမည်။
ပွား ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၂၊ ၃]
[၁၀၀၊ ၂၊ ၃]
```

- The copied list costs nothing at first: the two lists share one heap block, and the
  block counts the lists that hold it. The first write into either list (a cell write, an
  add, a remove) gives that list a block of its own with the same cells, the VM op `own`.
  A list that holds its block alone is written in place.
- Each text and each list in the cells is a value too, copied alike on its own
  first write. A [shared object](objects.md) in a cell is shared, not copied.
- A change to a list parameter remains in the function. A plain list parameter reads the
  caller's cells with a room of 0 ([How a list is stored](#how-a-list-is-stored)) until its
  first write; that write first copies the cells into a block of its own, so the caller's
  list never changes. A counted list parameter takes a copied list, or reads the caller's list in
  place when the function never writes it.
- To let a function change the caller's list, mark the parameter `ဒေတာ ပြောင်းလဲရန်`
  ([Functions](functions.md)). The argument takes no mark and must be a place; the
  function's writes, adds and removes reach the caller's list when the call returns.
- A plain list parameter takes lists of each count, and a counted list given to it is
  copied into a heap block. To pass a part of a list, pass a [range](#ranges).
- A function can give back a list of each form; a counted result kind gives the cell count
  as a written count. See [Lists and structs as
  results](functions.md#lists-and-structs-as-results).
- A list literal as an argument takes each value back to the last `၊` that starts the
  argument list. When another argument comes before it, put the literal in parentheses.

```zaka
// the parameter is the caller's list, to change
လုပ်ဆောင်ချက် နှစ်ဆတိုး သည်
ကိန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ (ကိန်းများ ၏ အကွက်အရေအတွက်) == ၀ ဖြစ်လျှင် ပြန်ပေးမည်။
၀ မှ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၁ အထိ န အနေဖြင့် ရေတွက်၍ ကိန်းများ ၏ အကွက် န ကို (ကိန်းများ ၏ အကွက် န) * ၂ ဟု ထားမည်။
ကိန်းများ ထဲသို့ ၀ ကို ထည့်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မူရင်း သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် ပွား သည် မူရင်း ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် နှစ်ဆတိုး ကို မူရင်း တို့ဖြင့် လုပ်ဆောင်မည်။
မူရင်း ကို ပြသမည်။     // changed through the parameter
ပွား ကို ပြသမည်။       // a copied list of its own: unchanged
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၂၊ ၄၊ ၆၊ ၀]
[၁၊ ၂၊ ၃]
```

```zaka
// a list literal after another argument goes in parentheses
လုပ်ဆောင်ချက် ပေါင်း သည်
အစ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ကိန်းများ ဟူသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
အဖြေ ကို အစ ဟု ထားမည်။
၀ မှ ၂ အထိ န အနေဖြင့် ရေတွက်၍ အဖြေ ကို အဖြေ + (ကိန်းများ ၏ အကွက် န) ဟု ထားမည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ပေါင်း ကို ၁၀၀၊ (၁၊ ၂၊ ၃ တို့ပါသော စာရင်း) တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။
က သည် ခ နှင့် ညီသည် ဟု ပြသမည်။
ခ ၏ အကွက် ၂ ကို ၄ ဟု ထားမည်။
က == ခ ကို ပြသမည်။
က != စာရင်းအလွတ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
မမှန်
မှန်
```

```zaka-error ZK0404
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
က == ခ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0404]: cannot combine isize[] and isize[3] without a conversion
```

## Ranges

```text
<list> ၏ အကွက် <start> မှစ၍ <length> ကွက်
```

A range ("`length` cells from cell `start` on") is a new plain list that holds those cells
of the list, copied. It stands apart from the list it came from: a later write or add on
either one leaves the other as it was. A range of text is text; see
[Text](text.md#ranges).

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် [၁၊ ၂၊ ၃၊ ၄] ဖြစ်ပါစေ။
ကိန်းရှင် အလယ် သည် က ၏ အကွက် ၁ မှစ၍ ၂ ကွက် ဖြစ်ပါစေ။
က ထဲသို့ ၅ ကို ထည့်မည်။
က ၏ အကွက် ၁ ကို ၉ ဟု ထားမည်။
အလယ် ထဲသို့ ၇ ကို ထည့်မည်။
အလယ် ကို ပြသမည်။
က ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၂၊ ၃၊ ၇]
[၁၊ ၉၊ ၃၊ ၄၊ ၅]
```

- The cells are counted from 0. The source is a list of either form, or, in marked code, a
  pointer to a list or into a heap block; another value is ZK0422. A range of a plain list
  of `isize` cells is an `isize[]`, whatever the form of its source.
- The source must be a place: a variable, a field, a cell or a parameter. A literal, a
  call result or another range is ZK0411, and a constant is ZK0412. Put such a value in a
  variable first.
- The start and the length are integers (ZK0423). Each one reads up to the word after it,
  so `x ၏ အကွက် i + ၁ မှစ၍ န - ၁ ကွက်` needs no parentheses. A length of 0 gives a list
  with no cells.
- The range must lie in its source. With a counted list as the source and the start and
  the length as constants, a range past the end is ZK0605. Each other range is checked
  when the program runs: past a list, the program stops with the range it asked for;
  past a heap block, it stops as a pointer step out of the block does.
- A range is a value like each other list: it can be printed, compared, passed, returned
  and stored. A function can give back a range of its own local list.
- The cells are copied when the range is made, so a range costs time and space in its
  length.

```zaka
// the result is a list of its own
လုပ်ဆောင်ချက် အလယ်ပိုင်း သည်
ကိန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် စာရင်း တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းများ ၏ အကွက် ၁ မှစ၍ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၂ ကွက် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် l သည် [၁၊ ၂၊ ၃၊ ၄] ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် အလယ်ပိုင်း ကို l တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
(l ၏ အကွက် ၀ မှစ၍ ၂ ကွက်) == [၁၊ ၂] ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၂၊ ၃]
မှန်
```

```zaka-error ZK0605
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် l သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
l ၏ အကွက် ၂ မှစ၍ ၂ ကွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0605]: 2 cells from cell 2 pass the end of a list of 3 cells
```

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် n သည် လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၄ ဖြစ်ပါစေ။
l ၏ အကွက် ၀ မှစ၍ n ကွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: 4 cells from cell 0 pass the end of 3 cells
```

## Forms

| Form | Burmese | Its cells |
| --- | --- | --- |
| plain | `စာရင်း` | in static data, or in a heap block that it holds alone or shares; the count comes when the program runs |
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
| counted | plain | on its own: the cells are copied into a heap block |
| plain | counted | never on its own (ZK0401): the count of a counted list is part of its kind |

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

```zaka-error ZK0401
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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
- The first edit (a cell write, an add, a remove) first copies the cells into a heap block
  of its own, so the static data never changes. A remove lowers the count alone and copies
  nothing.
- A copied list shares the static block, as copied static text does.
- A literal with a value known when the program runs and not before, such as a variable, is built in
  a heap block.
- With `--fold none` a literal of operations on constants is built when the program runs;
  the output is the same.
- A counted list keeps its cells in place, so it never points at static data. A counted
  literal of known cells is still a static list, and the list copies its cells from it in
  one step, in place of a write for each cell.

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

A `ကိန်းသေ` that holds a plain list (or text, or a list or struct with either inside) is
static data in each build: its value must fold at compile time, whatever `--fold` holds,
else ZK0606, reported at the first cell that does not fold. With constant propagation on,
a cell read of such a constant at a constant number, a field of a struct in it, and its
count become constants, and a constant number outside it is ZK0603.

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

- The room is the size of the heap block in octets. A room of 0 marks cells the list
  does not hold: static cells, the caller's cells read through a parameter, or no cells at all.
- Each heap block counts the lists that share it. A copied list adds one to the count,
  and a list let go takes one off; the block is freed when the count reaches 0.
- A write into a list of room 0, or into a block whose count is above 1, first copies
  the cells into a block of its own, so a plain list never writes cells that another
  list reads.
- An add into a full block moves the cells to a block twice its size, four cells at the
  least; a remove keeps the block and its room.
- Each list in the cells of a plain list is a value of its own: a plain list of plain
  lists holds three words per row, and each row has a block of its own.

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

## Lists in marked code

The rules of pointers are in [Unsafe Zaka](unsafe.md); this section gives the ones that
touch lists. Each example runs inside a marked block.

The address of a cell is a pointer to the cell, so the cell after it can be reached with
pointer arithmetic. In a table of counted lists, the cell after the last of a row is the
first of the next row; the rows of a plain list each sit in a block of their own, and a
step from one into the next stops the program:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဇယား သည် အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (၁၊ ၂၊ ၃ တို့ပါသော စာရင်း)၊ (၄၊ ၅၊ ၆ တို့ပါသော စာရင်း) တို့ပါသော စာရင်း ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် အစ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဇယား ၏ အကွက် ၀ ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် အတန်း သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဇယား ၏ အကွက် ၁ ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။
အတန်း - အစ ကို ပြသမည်။                      // the second row starts 3 cells on
အစ + ၄ ၏ တန်ဖိုး ကို ပြသမည်။                  // row 1, column 1
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၃
၅
```

The address of a list points at the whole list. For a counted list its kind holds the
count: `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ`, or `isize[3]*` in messages. Adding 1
to it moves one whole list, which for a counted list of counted rows is one row. The
address of a plain list, `isize[]*`, points at its three words, and an `isize[]*` does not
take an `isize[3]*` (ZK0401).

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဇယား သည် အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (၁၊ ၂၊ ၃ တို့ပါသော စာရင်း)၊ (၄၊ ၅၊ ၆ တို့ပါသော စာရင်း) တို့ပါသော စာရင်း ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် အတန်း သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဇယား ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။
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

The address of one cell is a plain pointer to the kind of the cell. The `အကွက်` forms
work on a list and on a pointer to one (read through one level).

## Edits and pinned cells

Taking the address of a cell of a plain list *pins* its block: the list first takes a
block of its own (the op `own`), and the block is marked (the op `pin`) so that it is never
shared again. A list copied from a pinned list gets cells of its own at once, so a write
through the pointer never reaches the copied list.

A pin does not hold the cells in place. An add can move them to a larger block, and a
write into the whole list replaces them, so a pointer taken before the edit is left
pointing at freed cells. The checker reports such a use as ZK0704, a warning in marked code
unless `--mem-check` sets another level ([Compiler checks](../compiler/checks.md)), and the VM stops the program at the
access. The cells of a counted list never move.

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။
ပ ၏ တန်ဖိုး ကို ၉ ဟု ထားမည်။
က ကို ပြသမည်။
ခ ကို ပြသမည်။                      // a copied list of its own: unchanged
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

```zaka-error ZK0704 mem-check=error
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။
က ထဲသို့ ၄ ကို ထည့်မည်။
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## Mistakes with cell numbers

A cell number outside the list is a mistake that Zaka catches at two times.

**At compile time,** for a counted list, whose count the compiler knows. A constant cell
number outside the list is ZK0603, in each mode. When the number is in a variable that the
compiler can follow, as in a counted loop, or a constant held in a variable, the same check
reports ZK0710 with the size of the list:

```zaka-error ZK0603
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
အမှတ်များ ၏ ၀ ခုမြောက်အကွက် ကို ပြသမည်။     // counting from 1 starts at 1
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0603]: cell 0 is outside the list, whose cells run from 1 to 3
```

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

A common cause is the number taking a whole sum. `ကိန်းများ ၏ အကွက် ၁ + ၂` is cell 3,
not cell 1 plus 2:

```zaka-error ZK0603
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀၊ ၂၀၊ ၃၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက် ၁ + ၂ ကို ပြသမည်။         // cell 3, not cell 1 plus 2
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် န သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက် န ကို ပြသမည်။            // cells 0, 1 and 2 hold values; 3 does not exist
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: cell 3 is outside the list, whose cells run from 0 to 2
```

When the compiler proves that a cell number is inside the list, the VM does not
repeat the check for that access. A proof is given up whenever something the compiler
cannot follow could change the number, for example a counter whose address marked code
takes; then the VM checks as before.

The other mistakes with lists, in one table:

| Code | Mistake |
| --- | --- |
| ZK0227 | a count that is not a whole number above zero, or names no constant that works out |
| ZK0401 | a plain list given where a counted list is wanted |
| ZK0403 | `<`, `>` or another order used on lists |
| ZK0404 | lists of different kinds compared or combined, such as a counted and a plain list |
| ZK0419 | a literal whose cell count differs from the counted kind |
| ZK0420 | `စာရင်းအလွတ်` with no list kind from its place |
| ZK0408 | a counted list of u8s, or another list, converted to text: a plain list of u8s alone converts |
| ZK0411 | a range of a literal, a call result or another range |
| ZK0412 | a range of a constant |
| ZK0422 | cells or a range of a value that is not a list |
| ZK0423 | a cell number that is not an integer |
| ZK0436 | `[]` or `စာရင်းအလွတ်` given to a counted list |
| ZK0723 | a warning: a read of a counted list whose cells the check cannot prove set |
| ZK0434 | an each-cell loop over a value that is no list |
| ZK0437 | an add or a remove on a counted list, or `နောက်ဆုံးအကွက်` on text |
| ZK0603 | a constant cell number outside the list |
| ZK0605 | a range with a constant start and length past the end of its counted list |
| ZK0606 | a constant plain list whose value needs the program to run |
| ZK0704 | in marked code, a pointer into a plain list used after an add or a write that can move its cells |
| ZK0710 | a cell number proven past the end |

The text of each message is in [Error messages](../reference/errors.md).
