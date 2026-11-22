---
title: Lists
description: The data kinds of lists, list literals, cells and cell numbers, adding and removing cells, lists of lists, passing and comparing, ranges, and the mistakes with cell numbers.
---

# Lists

A list holds values of one data kind (type) in order. Each value occupies a *cell*
(element), which we access through its *cell number* (index). A plain list (dynamic
array) can grow or shrink while the program runs. A counted list (fixed-size array) has
a length fixed in its data kind. For rules specific to counted lists and for storage
details, see [Counted lists and storage](counted-lists.md). For a first example, see
[Lists](../../learn/lists/README.md) in Learn.

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

To write the data kind of a plain list, we put the data kind of its cells before
`စာရင်း` ("list"). To make a counted list, we add `အကွက် <count> ခုရှိသော` ("that has
`<count>` cells") in front of it. Error messages show these data kinds in a short form:

| Data kind | In messages | Form |
| --- | --- | --- |
| `ကိန်းပြည့် စာရင်း` | `isize[]` | plain |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း` | `isize[3]` | counted |

The word `ခု` and a `၏` before `စာရင်း` are optional, so these four spellings name the
same data kind:

| Data kind |
| --- |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း` |
| `အကွက် ၃ ရှိသော ကိန်းပြည့် စာရင်း` |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် ၏ စာရင်း` |
| `အကွက် ၃ ရှိသော ကိန်းပြည့် ၏ စာရင်း` |

- The cell kind can be each data kind there is: a number, a true/false value, a
  character, text, a pointer, a struct, or another list ([Lists of
  lists](#lists-of-lists)).
- `<cell kind> စာရင်း` is the [written-as hint](generics.md#written-as-hints) of
  `စာရင်း` in the core module, which is a name and not a reserved word. So the long form
  `ကိန်းပြည့် ကို အစားထိုးအသုံးပြုသည့် စာရင်း` and the form `စာရင်း<ကိန်းပြည့်>` also name a
  plain list. A count in front works with the hint form alone.
- The count is a whole number above zero that the compiler knows (ZK0227). We can write
  digits, a constant, or an expression that the compiler can fold; [The
  count](counted-lists.md#the-count) lists each form it accepts.

A list declared with no value has no cells and no count, so it is unset until the
program sets it ([Set before read](declarations.md#set-before-read)). The [initial
value](kinds.md#initial-values) (default value), `ကနဦးတန်ဖိုး`, depends on the form:

| Form | `ကနဦးတန်ဖိုး` gives |
| --- | --- |
| plain | a list with no cells, the same as `[]` |
| counted | each cell set to the initial value of its cell kind: `၀` for a number, `မမှန်` for a true/false value, the null address for a pointer |

A counted list declared with no value counts as set once each of its cells is set;
[Setting each cell](counted-lists.md#setting-each-cell) states the rule and its warning,
ZK0723.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အကွက် ၂ ခုရှိသော ဒဿမကိန်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အကွက် ၃ ရှိသော မှန်မှားတန်ဖိုး ၏ စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် စာလုံး စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် 'က' နှင့် 'ခ' တို့ပါသော စာရင်း ဖြစ်ပါစေ။
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

```text
<value>၊ <value> နှင့် <value> တို့ပါသော စာရင်း
<value> ပါသော စာရင်း
[<value>၊ <value>၊ <value>]
စာရင်းအလွတ်
[]
```

We can write a list value in two forms that mean the same. The word form is a [list in
words](words.md#lists-in-words), with `၊` between the cells and `နှင့်` before the last,
closed with `တို့ပါသော စာရင်း` ("the list that has these"); a list of one cell closes
with `ပါသော စာရင်း`. The bracket form puts the cells between `[` and `]` with `၊` between
them, which is also how a list prints. So `၇၀၊ ၈၅ နှင့် ၉၀ တို့ပါသော စာရင်း` and
`[၇၀၊ ၈၅၊ ၉၀]` are the same list. `စာရင်းအလွတ်` ("blank list") and `[]` are a list with no
cells.

To give a literal its data kind, we write `ဟူသော <kind> တစ်<word>` after it, as in
`[၁၊ ၂] ဟူသော ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု`. The literal then has that list kind, plain or
counted, and each cell gets the cell kind. Without a written data kind, a literal gets
the list kind of its place.

| Case | What happens | Code |
| --- | --- | --- |
| a data kind after the literal that is not a list | an error | ZK0222 |
| a literal that fills a counted list | its number of cells must equal the count | ZK0419 |
| a literal that fills a plain list | each number of cells fits | |
| `စာရင်းအလွတ်` or `[]` with no list kind from its place and none written, as in `စာရင်းအလွတ် ကို ပြသမည်။` | an error; write one, as in `[] ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု` | ZK0420 |
| `စာရင်းအလွတ်` or `[]` for a plain list | a list with no cells | |
| `စာရင်းအလွတ်` or `[]` for a counted list, in each place, the declaration included | an error, because a counted list has all of its cells at all times; `ကနဦးတန်ဖိုး` resets each cell instead | ZK0436 |

Each cell of a literal is a notation expression. A bracket list and a struct literal in
the short form are notation expressions too, so these nest as written, as in
`[[၁၊ ၂]၊ [၃၊ ၄]]`. A clause, or a literal in the word form, goes in parentheses when it
is a cell of another literal.

```zaka-error ZK0419
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀ နှင့် ၈၅ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
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

Setting a plain list to `[]` removes all of its cells, while setting a counted list to
`ကနဦးတန်ဖိုး` resets each cell:

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

```text
<list> ၏ အကွက် <number>
<list> ၏ <number> ခုမြောက်အကွက်
<list> ၏ အကွက်အရေအတွက်
```

We can reach a cell in two forms: `အကွက် <number>` counts from 0, as programs count, and
`<number> ခုမြောက်အကွက်` counts from 1, as people count. `အကွက်အရေအတွက်` ("cell count")
is the number of cells.

| Written | Means |
| --- | --- |
| `အမှတ်များ ၏ အကွက် ၀` | the first cell |
| `အမှတ်များ ၏ ၁ ခုမြောက်အကွက်` | the first cell |
| `အမှတ်များ ၏ အကွက် ၂` | the third cell |
| `အမှတ်များ ၏ ၃ ခုမြောက်အကွက်` | the third cell |
| `အမှတ်များ ၏ အကွက်အရေအတွက်` | the number of cells, a `လက္ခဏာမဲ့ ကိန်းပြည့်`: a constant for a counted list, and known when the program runs for a plain list |

A cell is a place, so we can read it, print it and assign to it, and unsafe code can take
its address ([Unsafe Zaka](unsafe.md)). The cell number must be an integer (ZK0423), and
the value before `၏` must be a list or a pointer to one (ZK0422).

```zaka-error ZK0422
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀ ဖြစ်ပါစေ။
အမှတ် ၏ အကွက် ၀ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0422]: isize is not a list, so it has no cells
```

To visit each cell in turn, we can use an [each-cell loop](control-flow.md#each-cell-loops):
`အမှတ်များ ၏ အကွက်တိုင်း ကို အမှတ် အနေဖြင့် ယူ၍ ...` puts each cell into the variable
`အမှတ်`, one round at a time. A list of pointers works as well, with one address in each
round. An each-cell loop over a value that is not a list is ZK0434.

### The cell number uses the whole expression

The number after `အကွက်` runs as far as the operator expression goes, in the same manner
that `ပ + ၁ ၏ တန်ဖိုး` covers all of `ပ + ၁`. To use a cell inside a sum, we put the cell
in parentheses.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀၊ ၂၀ နှင့် ၃၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက် ၀ + ၁ ကို ပြသမည်။ // the cell number is 0 + 1
(ကိန်းများ ၏ အကွက် ၀) + ၁ ကို ပြသမည်။ // the first cell, plus 1
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၀
၁၁
```

`အကွက်အရေအတွက်` is one word. Written with a space, `အကွက် အရေအတွက်` is the cell whose
number is in a variable named `အရေအတွက်`.

To visit each cell with a counted loop, we use the cell count in parentheses to get the
last number:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၄၊ ၈၊ ၁၅ နှင့် ၁၆ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
၀ မှ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၁ အထိ န အနေဖြင့် ရေတွက်၍ ကိန်းများ ၏ အကွက် န ကို (ကိန်းများ ၏ အကွက် န) * ၂ ဟု ထားမည်။
ကိန်းများ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၈၊ ၁၆၊ ၃၀၊ ၃၂]
```

On a list with no cells, `အကွက်အရေအတွက် - ၁` stops the program, since the count cannot go
below 0: see [A list with no cells and the last
number](../../learn/lists/mistakes.md#a-list-with-no-cells-and-the-last-number).

## Adding and removing cells

```text
<list> ထဲသို့ <value> ကို ထည့်မည်။
<list> ၏ အကွက် <number> ၏ အရှေ့တွင် <value> ကို ထည့်မည်။
<list> ၏ <number> ခုမြောက်အကွက် ၏ အရှေ့တွင် <value> ကို ထည့်မည်။
<list> ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။
```

We use `ထည့်မည်` ("add") to put a value in a new last cell. With `၏ အရှေ့တွင်` ("in front
of"), we put the value in front of the cell at that number instead: the new value gets
that number, and each cell from there on moves up one place. We use
`နောက်ဆုံးအကွက်ကို ဖျက်မည်` ("remove the last cell") to take the last cell off. These are
statements, so none of them returns a value.

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

The rules of an add and a remove:

- The list must be a plain list in a place we can set: a variable, a field, a cell of
  another list, or, in unsafe code, `ပ ၏ တန်ဖိုး` through a pointer `ပ`. A pointer to a
  list is not a list, so we add to `ပ ၏ တန်ဖိုး`, not to `ပ`.
- The number before `၏ အရှေ့တွင်` counts from 0 for `အကွက် n` and from 1 for
  `n ခုမြောက်အကွက်`, as a cell read does. It can also equal the count, which puts the
  value at the end.
- The value goes into the list as an assigned value goes into a place: text, lists and
  structs are duplicated (copied), and a shared object is shared ([Values and
  objects](objects.md)).
- The value is worked out before the list changes, so
  `က ထဲသို့ က ၏ အကွက်အရေအတွက် ကို ထည့်မည်။` adds the count from before the add.
- To add to text or to remove its last character, see [Changing
  text](text.md#changing-text).

| Case | What happens | Code |
| --- | --- | --- |
| the list is a constant | an error, as for an assignment | ZK0306 |
| the list is a counted list, whose count is fixed | an error | ZK0437 |
| `နောက်ဆုံးအကွက်` on text, which loses its last character through `နောက်ဆုံးစာလုံး` | an error | ZK0437 |
| `၏ အရှေ့တွင်` after a value that is not a cell of a list or a character of text | an error | ZK0439 |
| the number before `၏ အရှေ့တွင်` is not an integer | an error | ZK0423 |
| the value does not have the cell kind | an error | ZK0401 |
| `နောက်ဆုံးအကွက်` ("last cell") with no `ကို ဖျက်မည်` after it: it is not an expression, so we read the last cell through its number | an error | ZK0208 |
| the number before `၏ အရှေ့တွင်` is larger than the count | the program stops: "no value can go in front of 3: the places run from 0 to 2, and 2 is the end" | |
| a remove on a list with no cells | the program stops: "the list holds no cells, so it has no last cell to remove" ([When a program stops](../vm/traps.md#all-the-traps)) | |

### What an add and a remove cost

- A list that shares its block with a duplicate or with a caller ([How a list is
  stored](counted-lists.md#how-a-list-is-stored)) first gets its own block on its first
  add, as on its first write, so the other list does not change. A remove on a list with
  a room of 0 lowers the count and leaves the shared cells alone, so it duplicates
  nothing.
- When the block is full, the cells move to a block twice the size, with room for four
  cells at the least.
- When the list is the sole owner of its block, a remove drops the value in the last
  cell, so text in that cell is freed.
- An add can move the cells to a larger block, so a pointer that unsafe code took into
  the cells can be left pointing at freed cells; see [Edits and pinned
  cells](counted-lists.md#edits-and-pinned-cells).

The add and the remove work on a list inside a list, and in front of a cell:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဂရစ် သည် ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [[၁]၊ [၂၊ ၃]] ဖြစ်ပါစေ။
ဂရစ် ၏ အကွက် ၀ ထဲသို့ ၅ ကို ထည့်မည်။
ဂရစ် ထဲသို့ [၆] ကို ထည့်မည်။
ဂရစ် ၏ အကွက် ၁ ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။
ဂရစ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[[၁၊ ၅]၊ [၂]၊ [၆]]
```

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၀၊ ၃၀] ဖြစ်ပါစေ။
က ၏ အကွက် ၁ ၏ အရှေ့တွင် ၂၀ ကို ထည့်မည်။ // in front of ၃၀
က ၏ ၁ ခုမြောက်အကွက် ၏ အရှေ့တွင် ၀ ကို ထည့်မည်။ // at the start
က ၏ အကွက် ၄ ၏ အရှေ့တွင် ၄၀ ကို ထည့်မည်။ // ၄ is the count: at the end
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

```text
<cell kind> စာရင်း စာရင်း
အကွက် <rows> ခုရှိသော အကွက် <columns> ခုရှိသော <cell kind> စာရင်း စာရင်း
```

A cell can contain a whole list, so we can build a grid as a list of rows. Each count in
front pairs with one `စာရင်း`, the inner ones first: in `အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော
ကိန်းပြည့် စာရင်း စာရင်း`, the `၃` belongs to the first `စာရင်း` (a row of 3 integers) and
the `၂` to the second (2 such rows). Messages write this data kind as `isize[3][2]`. The
two forms nest as deep as we need: `isize[3][]` is a plain list of counted rows, and
`isize[][]` is a plain list of plain rows, where each row has its own count.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဂရစ် သည် အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။ // 2 rows of 3
ဂရစ် ၏ အကွက် ၁ ၏ အကွက် ၂ ကို ၉ ဟု ထားမည်။ // row 1, column 2
ဂရစ် ကို ပြသမည်။
ဂရစ် ၏ အကွက် ၁ ကို ပြသမည်။
ဂရစ် ၏ အကွက်အရေအတွက် ကို ပြသမည်။
ဂရစ် ၏ အကွက် ၀ ၏ အကွက်အရေအတွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[[၀၊ ၀၊ ၀]၊ [၀၊ ၀၊ ၉]]
[၀၊ ၀၊ ၉]
၂
၃
```

In the word form, we put each inner literal in parentheses. In a data kind with no
counts, each list is plain and gets its count from the value:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဂရစ် သည် ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (၁ နှင့် ၂ တို့ပါသော စာရင်း)၊ (၃ နှင့် ၄ တို့ပါသော စာရင်း) နှင့် (၅ နှင့် ၆ တို့ပါသော စာရင်း) တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ဂရစ် ကို ပြသမည်။
ဂရစ် ၏ ၃ ခုမြောက်အကွက် ၏ ၁ ခုမြောက်အကွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[[၁၊ ၂]၊ [၃၊ ၄]၊ [၅၊ ၆]]
၅
```

## Passing and comparing

A list is a value, like an integer. When we set one list from another, we get a separate
list, and a change to one does not reach the other. A list passed as an argument works
the same, since a parameter is a new variable:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မူရင်း သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂ နှင့် ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် ပွား သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
ပွား ကို မူရင်း ဟု ထားမည်။ // the two share one block until a write
ပွား ၏ အကွက် ၀ ကို ၁၀၀ ဟု ထားမည်။ // this write first duplicates the cells
မူရင်း ကို ပြသမည်။
ပွား ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၂၊ ၃]
[၁၀၀၊ ၂၊ ၃]
```

- A duplicate (copy) of a list costs nothing at first: the two lists share one heap
  block, and the block counts the lists that use it. The first write into either list (a
  cell write, an add or a remove) gives that list its own block with the same cells,
  through the VM operation `own`. A list that is the sole user of its block is written in
  place.
- Each text and each list inside the cells is a value too, and is duplicated in the same
  manner on its first write. A [shared object](objects.md) (reference type) in a cell is
  shared, not duplicated.
- A change to a list parameter remains inside the function. A plain list parameter reads
  the caller's cells with a room of 0 ([How a list is
  stored](counted-lists.md#how-a-list-is-stored)) until its first write, and that write
  first duplicates the cells into a new block, so the caller's list does not change. A
  counted list parameter gets a duplicate, or reads the caller's list in place when the
  function does not write to it.
- To let a function change the caller's list, we mark the parameter with
  `ဒေတာ ပြောင်းလဲရန်`, which makes it a mutable parameter (`inout`)
  ([Mutable parameters](functions-parameters.md#mutable-parameters)). The argument needs no marker and must be a place, and the
  writes, adds and removes of the function reach the caller's list when the call returns.
- A plain list parameter accepts lists of each count, and a counted list passed to it is
  duplicated into a heap block. To pass a part of a list, we pass a [range](#ranges).
- A function can return a list of either form; a counted result kind has the cell count
  written in it. See [Lists and structs as
  results](functions-results.md#lists-and-structs-as-results).
- A list literal in the word form, used as an argument, collects each value back to the
  last `၊` that starts the argument list. So when another argument comes before it, we
  put the literal in parentheses.

```zaka
// the parameter is the caller's list, to change
နှစ်ဆတိုး ဟူသော လုပ်ဆောင်ချက် သည်
ကိန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ (ကိန်းများ ၏ အကွက်အရေအတွက်) == ၀ ဖြစ်လျှင် ပြန်ပေးမည်။
၀ မှ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၁ အထိ န အနေဖြင့် ရေတွက်၍ ကိန်းများ ၏ အကွက် န ကို (ကိန်းများ ၏ အကွက် န) * ၂ ဟု ထားမည်။
ကိန်းများ ထဲသို့ ၀ ကို ထည့်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မူရင်း သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂ နှင့် ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် ပွား သည် မူရင်း ဖြစ်ပါစေ။
နှစ်ဆတိုး ဟူသော လုပ်ဆောင်ချက်ကို မူရင်း ဖြင့် လုပ်ဆောင်မည်။
မူရင်း ကို ပြသမည်။ // changed through the parameter
ပွား ကို ပြသမည်။ // a separate duplicate: unchanged
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၂၊ ၄၊ ၆၊ ၀]
[၁၊ ၂၊ ၃]
```

```zaka
// a list literal after another argument goes in parentheses
ပေါင်း ဟူသော လုပ်ဆောင်ချက် သည်
အစ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ကိန်းများ ဟူသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
အဖြေ ကို အစ ဟု ထားမည်။
၀ မှ ၂ အထိ န အနေဖြင့် ရေတွက်၍ အဖြေ ကို အဖြေ + (ကိန်းများ ၏ အကွက် န) ဟု ထားမည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို ၁၀၀ နှင့် (၁၊ ၂ နှင့် ၃ တို့ပါသော စာရင်း) တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၀၆
```

### Comparing

```text
<list> == <list>
<list> != <list>
<list> သည် <list> နှင့် ညီသည်
<list> သည် <list> နှင့် မညီ
```

We can compare two lists of the same data kind with `==` and `!=`, or with the verbs
`ညီ` and `မညီ`. The comparison goes one cell after another: two lists are equal when
their counts match and each pair of cells is equal (the VM operation `seq_eq`). Lists of
different data kinds cannot be compared (ZK0404), such as counted lists of different
counts (`isize[3]` and `isize[2]`) or a counted and a plain list. Lists have no order, so
`<`, `>` and the other orderings do not work on them (ZK0403).

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂ နှင့် ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
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
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂ နှင့် ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂ နှင့် ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
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

A range (slice), "`<length>` cells from cell `<start>` on", is a new plain list with
those cells of the list, duplicated. It is separate from the list it came from, so a
later write or add on either one leaves the other unchanged. A range of text is text; see
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

- The cells are counted from 0. The start and the length must be integers (ZK0423).
  Each of them runs up to the word after it, so `x ၏ အကွက် i + ၁ မှစ၍ န - ၁ ကွက်` needs no
  parentheses. A length of 0 gives a list with no cells.
- The source can be a list of either form: a variable, a constant, a field, a cell, a
  parameter, a literal, a call result or another range. In unsafe code it can also be a
  pointer to a list or into a heap block. Another value is ZK0422.
- A range is a plain list whatever the form of its source: a range of an `isize[3]` is
  an `isize[]`.
- The range must lie inside its source. When the source is a counted list and the start
  and the length are constants, a range past the end is ZK0605. The program checks each
  other range when it runs: past the end of a list, it stops with the range it asked
  for, and past the end of a heap block, it stops as a pointer step out of the block
  does.
- A range is a value like each other list, so we can print it, compare it, pass it,
  return it and store it. A function can return a range of a local list.
- The cells are duplicated when the range is made, so a range costs time and space in
  its length.

```zaka
// the result is a separate list
အလယ်ပိုင်း ဟူသော လုပ်ဆောင်ချက် သည်
ကိန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် စာရင်း တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းများ ၏ အကွက် ၁ မှစ၍ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၂ ကွက် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် l သည် [၁၊ ၂၊ ၃၊ ၄] ဖြစ်ပါစေ။
အလယ်ပိုင်း ဟူသော လုပ်ဆောင်ချက်ကို l ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
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

## Mistakes with cell numbers

A cell number outside the list is a mistake that Zaka catches at two times: at compile
time when the compiler can work the number out, and when the program runs otherwise.

### At compile time

The compiler knows the count of a counted list, so it checks the cell numbers it can work
out. A constant cell number outside the list is ZK0603, in each mode. When the number is
in a variable that the compiler can follow, as in a counted loop or a constant kept in a
variable, the same check reports ZK0710 with the size of the list:

```zaka-error ZK0603
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅ နှင့် ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
အမှတ်များ ၏ ၀ ခုမြောက်အကွက် ကို ပြသမည်။     // counting from 1 starts at 1
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0603]: cell 0 is outside the list, whose cells run from 1 to 3
```

```zaka-error ZK0710
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅ နှင့် ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် န သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက် န ကို ပြသမည်။            // cells 0, 1 and 2 have values; 3 does not exist
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0710]: offset 3 is past the end of a block of 3 elements
```

A common cause is a cell number that runs over a whole sum. `ကိန်းများ ၏ အကွက် ၁ + ၂` is
cell 3, not cell 1 plus 2:

```zaka-error ZK0603
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀၊ ၂၀ နှင့် ၃၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက် ၁ + ၂ ကို ပြသမည်။         // cell 3, not cell 1 plus 2
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0603]: cell 3 is outside the list, whose cells run from 0 to 2
```

### When the program runs

The count of a plain list comes when the program runs, so the VM checks each access to
its cells, constant numbers too. For a counted list, the VM checks each access whose
number comes from input or from a value the compiler cannot follow. A failed check stops
the program with the range the list allows. This is what a plain list does with the
program above, and what `--lint-warn mem` shows for the counted one:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅ နှင့် ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် န သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက် န ကို ပြသမည်။ // cells 0, 1 and 2 have values; 3 does not exist
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: cell 3 is outside the list, whose cells run from 0 to 2
```

When the compiler proves that a cell number is inside the list, the VM does not repeat
the check for that access. The compiler gives a proof up when something it cannot follow
could change the number, for example a counter whose address unsafe code takes, and then
the VM checks as before.

## Errors at a glance

| Code | Mistake |
| --- | --- |
| ZK0208 | `နောက်ဆုံးအကွက်` with no `ကို ဖျက်မည်` after it |
| ZK0227 | a count that is not a whole number above zero, or that names no constant that works out ([The count](counted-lists.md#the-count)) |
| ZK0306 | an add or a remove on a constant list |
| ZK0401 | a plain list given where a counted list is wanted, or an added value without the cell kind |
| ZK0403 | `<`, `>` or another ordering used on lists |
| ZK0404 | lists of different data kinds compared or combined, such as a counted and a plain list |
| ZK0408 | a counted list of u8s, or another list, converted to text: a plain list of u8s alone converts |
| ZK0419 | a literal whose cell count differs from the counted list kind |
| ZK0420 | `စာရင်းအလွတ်` or `[]` with no list kind from its place |
| ZK0422 | cells or a range of a value that is not a list |
| ZK0423 | a cell number, a start or a length that is not an integer |
| ZK0434 | an each-cell loop over a value that is not a list |
| ZK0436 | `[]` or `စာရင်းအလွတ်` given to a counted list |
| ZK0437 | an add or a remove on a counted list, or `နောက်ဆုံးအကွက်` on text |
| ZK0439 | `၏ အရှေ့တွင်` after a value that is not a cell of a list or a character of text |
| ZK0603 | a constant cell number outside the list |
| ZK0605 | a range with a constant start and length past the end of its counted list |
| ZK0606 | a constant plain list whose value needs the program to run ([Static data](counted-lists.md#static-data)) |
| ZK0704 | in unsafe code, a pointer into a plain list used after an add or a write that can move its cells ([Edits and pinned cells](counted-lists.md#edits-and-pinned-cells)) |
| ZK0710 | a cell number proven past the end |
| ZK0723 | a warning: a read of a counted list whose cells the check cannot prove set ([Setting each cell](counted-lists.md#setting-each-cell)) |

The text of each message is in [Error messages](../reference/errors.md).
