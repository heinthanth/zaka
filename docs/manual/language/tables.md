---
title: Tables
description: The table data kind, which finds each row through its index - the data kind, literals, reading, setting and removing a row, writing inside a row, the count, the loops over rows, index kinds, equal tables, duplicates, and how a table is stored.
---

# Tables

A table (map) keeps rows (entries), and each row has an index (key), which finds it, and a
value. We write the data kind with the word `ဇယား`. No two rows of one table have equal
indexes. A list reaches a value through the number of its cell, while a table reaches it
through the index, which can be text, a number, a struct or another data kind (type) that
can be hashed. The rows keep the order in which their indexes were first added. The Learn
book teaches tables in small steps in [Tables](../../learn/tables/README.md).

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် စာသား ဖြင့် ညွှန်းသော ကိန်းပြည့် ဇယား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [:] ဖြစ်ပါစေ။
အသက် ထဲမှ အညွှန်း "မမ" ၏ တန်ဖိုး ကို ၂၀ ဟု ထားမည်။
အသက် ထဲမှ အညွှန်း "ကို" ၏ တန်ဖိုး ကို ၃၀ ဟု ထားမည်။
အသက် ကို ပြသမည်။
အသက် ထဲမှ အညွှန်း "မမ" ၏ တန်ဖိုး ကို ပြသမည်။
အသက် ထဲမှ အညွှန်း "ဇော်" ၏ တန်ဖိုး ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
["မမ": ၂၀၊ "ကို": ၃၀]
ဒေတာ(၂၀)
အလွတ်
```

## The data kind

```text
<index kind> ဖြင့် ညွှန်းသော <value kind> ဇယား
```

To write the data kind of a table, we put the index kind first, then `ဖြင့် ညွှန်းသော`
("indexed with"), then the value kind, then `ဇယား`. We can also write `ဖြင့်` joined to
the index kind. Error messages show the data kind in a short form:

| Data kind | In messages |
| --- | --- |
| `စာသား ဖြင့် ညွှန်းသော ကိန်းပြည့် ဇယား` | `map<text, isize>` |
| `ကိန်းပြည့် ဖြင့် ညွှန်းသော စာသား စာရင်း ဇယား` | `map<isize, text[]>` |
| `စာသား ဖြင့် ညွှန်းသော ကိန်းပြည့် ဇယား စာရင်း` | `map<text, isize>[]` |

- The phrase is the [written-as hint](generics.md#written-as-hints) of `ဇယား` in the
  core module, which is a name and not a reserved word. So the long form
  `စာသား နှင့် ကိန်းပြည့် တို့ကို အစားထိုးအသုံးပြုသည့် ဇယား` and the form
  `ဇယား<စာသား၊ ကိန်းပြည့်>` name the same data kind.
- The value kind can be each data kind: a list, a struct, an optional, a shared object or
  another table. The index kind must be a data kind that can be hashed (see [Index
  kinds](#index-kinds)).
- A table is counted with the measure word (classifier) `ခု`, and another word is the
  warning ZK0509. The index kind and the value kind need no measure word, as with the
  cell kind of a list.
- Words after `ဇယား` wrap the table itself, so `… ဇယား စာရင်း` is a list of tables.
- The initial value (default value), `ကနဦးတန်ဖိုး`, is a table with no rows.

## Literals

```text
<index> အနေဖြင့် <value> နှင့် <index> အနေဖြင့် <value> တို့ပါသော ဇယား
[<index>: <value>၊ <index>: <value>]
ဇယားအလွတ်
[:]
```

We can write a table value in two forms that mean the same. In the word form, we write
each row as `<index> အနေဖြင့် <value>` ("with the index, the value"). In the notation
form, which is also how a table prints, we write each row as `<index>: <value>`.
`ဇယားအလွတ်` ("blank table") and `[:]` are a table with no rows.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် "နီ" အနေဖြင့် ၁ နှင့် "ပြာ" အနေဖြင့် ၂ တို့ပါသော ဇယား ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ["နီ": ၁၊ "ပြာ": ၂၊ "နီ": ၉] ဖြစ်ပါစေ။
က ကို ပြသမည်။
ခ ကို ပြသမည်။
ကိန်းရှင် ဂ သည် ကိန်းပြည့် ဖြင့် ညွှန်းသော စာသား ဇယား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဇယားအလွတ် ဖြစ်ပါစေ။
ဂ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
["နီ": ၁၊ "ပြာ": ၂]
["နီ": ၉၊ "ပြာ": ၂]
[:]
```

- The rows are set in order. A later row with the index of an earlier one gives that row
  its value and keeps its place, so `"နီ"` remains first in `ခ` above, with the value 9.
- Without a written data kind, a literal gets the table kind of its place, or else the
  data kinds of its first row.
- To give a literal its data kind, we write `ဟူသော <kind> တစ်ခု` after it, as in
  `[:] ဟူသော စာသား ဖြင့် ညွှန်းသော ကိန်းပြည့် ဇယား တစ်ခု`. A data kind that is not a table
  is ZK0222.
- `ဇယားအလွတ်` and `[:]` with no data kind from their place and none written are ZK0486.
- An index and a value are notation expressions, as a list cell is, so a clause goes in
  parentheses.

```zaka-error ZK0486
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
[:] ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0486]: a blank ဇယား needs a ဇယား kind from its place
```

## Reading a row

```text
<table> ထဲမှ အညွှန်း <index> ၏ တန်ဖိုး
<optional> ထဲမှ ဒေတာ၊ မရှိလျှင် <value>
```

To read the row of an index, we write `<table> ထဲမှ အညွှန်း <index> ၏ တန်ဖိုး` ("from the
table, the value of the index"). The result is an [optional](optionals.md) of the value
kind: the value of the row, or `အလွတ်` when no row has the index. So a read never stops
the program.

We test and read this optional as we do each other optional. To get the data, or a
fallback value when there is none, we write `၊ မရှိလျှင် <value>` ("if none, the value")
after `ထဲမှ ဒေတာ`. This form works on each optional, not on a row alone.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဈေး သည် ["ပန်းသီး": ၅၀၀၊ "ငှက်ပျောသီး": ၃၀၀] ဖြစ်ပါစေ။
အကယ်၍ ဈေး ထဲမှ အညွှန်း "ပန်းသီး" ၏ တန်ဖိုး ထဲတွင် ဒေတာ ရှိလျှင် "ပန်းသီး ရှိသည်" ဟု ပြသမည်။
ဈေး ထဲမှ အညွှန်း "ငှက်ပျောသီး" ၏ တန်ဖိုး ထဲမှ ဒေတာ ကို ပြသမည်။
ဈေး ထဲမှ အညွှန်း "သရက်သီး" ၏ တန်ဖိုး ထဲမှ ဒေတာ၊ မရှိလျှင် ၀ ကို ပြသမည်။
ဈေး ထဲမှ အညွှန်း "ပန်းသီး" ၏ တန်ဖိုး ထဲမှ ဒေတာ၊ မရှိလျှင် ၀ + ၁ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ပန်းသီး ရှိသည်
၃၀၀
၀
၅၀၁
```

- The index after `အညွှန်း` is a notation expression, so it ends before the `၏`, and a
  clause goes in parentheses. It converts to the index kind (ZK0401).
- The value after `မရှိလျှင်` is one operand, together with a prefix operator on it,
  because a read binds tighter than an operator: `x ထဲမှ ဒေတာ၊ မရှိလျှင် ၀ + ၁` adds 1 to
  the data or to 0. The fallback runs when the optional has no data and at no other time,
  and it converts to the data kind of the data.
- `ထဲမှ အညွှန်း` after a value that is not a table is ZK0484.

```zaka-error ZK0484
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် [၁၀၊ ၂၀] ဖြစ်ပါစေ။
က ထဲမှ အညွှန်း ၀ ၏ တန်ဖိုး ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0484]: isize[] is no ဇယား, so it has no rows
```

## Setting and removing rows

```text
<table> ထဲမှ အညွှန်း <index> ၏ တန်ဖိုး ကို <value> ဟု ထားမည်။
<table> ထဲမှ အညွှန်း <index> ကို ဖျက်မည်။
```

We use `ထားမည်` to give the row of an index its value. When a row has the index, it keeps
its place and gets the new value; when no row has it, a new last row is added with the
index and the value. We use `ဖျက်မည်` to remove the row of an index; when no row has the
index, nothing happens.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဇ သည် ["က": ၁၊ "ခ": ၂၊ "ဂ": ၃] ဖြစ်ပါစေ။
ဇ ထဲမှ အညွှန်း "ခ" ၏ တန်ဖိုး ကို ၂၀ ဟု ထားမည်။
ဇ ထဲမှ အညွှန်း "က" ကို ဖျက်မည်။
ဇ ထဲမှ အညွှန်း "မရှိ" ကို ဖျက်မည်။
ဇ ထဲမှ အညွှန်း "က" ၏ တန်ဖိုး ကို ၁၀ ဟု ထားမည်။
ဇ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
["ခ": ၂၀၊ "ဂ": ၃၊ "က": ၁၀]
```

- The table must be a place we can set: a variable, a field, a cell of a list, or the
  value of a row of another table. A constant is ZK0306.
- The index and the value go in as an assigned value goes into a place: text, lists,
  structs and tables are duplicated (copied), and a shared object is shared ([Values and
  objects](objects.md)).
- The index and the value are worked out before the table changes.
- `ကို ဖျက်မည်` after a value that is not a row, `<table> ထဲမှ အညွှန်း <index>`, fits no
  sentence of `ဖျက်` (ZK0332), and the help lists the sentences that fit. A list loses its
  last cell through `နောက်ဆုံးအကွက်ကို ဖျက်မည်` ([Lists](lists.md#adding-and-removing-cells)).

```zaka-error ZK0306
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ ဇ သည် ["က": ၁] ဖြစ်ပါစေ။
ဇ ထဲမှ အညွှန်း "ခ" ၏ တန်ဖိုး ကို ၂ ဟု ထားမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0306]: "ဇ" is a constant and cannot be assigned
```

```zaka-error ZK0332
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ၅ ဖြစ်ပါစေ။
က ကို ဖျက်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0332]: no function "ဖျက်" takes these arguments
```

## Writing inside a row

We can write inside the value of a row in place, as we write inside a variable: set a
field or a cell of it, add to it, or remove from it. The row must exist: when no row has
the index, the program stops with `no row of the ဇယား has this index`, in the VM and in a
native program alike.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အုပ် သည် စာသား ဖြင့် ညွှန်းသော ကိန်းပြည့် စာရင်း ဇယား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [:] ဖြစ်ပါစေ။
အုပ် ထဲမှ အညွှန်း "စုံ" ၏ တန်ဖိုး ကို [] ဟု ထားမည်။
အုပ် ထဲမှ အညွှန်း "မ" ၏ တန်ဖိုး ကို [] ဟု ထားမည်။
၁ မှ ၆ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ န % ၂ သည် ၀ နှင့် ညီလျှင် အုပ် ထဲမှ အညွှန်း "စုံ" ၏ တန်ဖိုး ထဲသို့ န ကို ထည့်မည်။
သို့မဟုတ်ပါက အုပ် ထဲမှ အညွှန်း "မ" ၏ တန်ဖိုး ထဲသို့ န ကို ထည့်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အုပ် ထဲမှ အညွှန်း "မ" ၏ တန်ဖိုး ၏ အကွက် ၀ ကို ၁၀၀ ဟု ထားမည်။
အုပ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
["စုံ": [၂၊ ၄၊ ၆]၊ "မ": [၁၀၀၊ ၃၊ ၅]]
```

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အုပ် သည် စာသား ဖြင့် ညွှန်းသော ကိန်းပြည့် စာရင်း ဇယား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [:] ဖြစ်ပါစေ။
"မစမီ" ဟု ပြသမည်။
အုပ် ထဲမှ အညွှန်း "စုံ" ၏ တန်ဖိုး ထဲသို့ ၂ ကို ထည့်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မစမီ
the program stopped: no row of the ဇယား has this index
```

- A write through a row writes the value inside the table, not a duplicate of it, as a
  write through a cell does.
- A table inside a row is written in the same manner:
  `ဇ ထဲမှ အညွှန်း "က" ၏ တန်ဖိုး ထဲမှ အညွှန်း "ခ" ၏ တန်ဖိုး ကို ၁ ဟု ထားမည်။` sets a row of
  the table in the row `"က"`.
- Unsafe code cannot take the address of a row, so a row is never pinned.

## Counting and walking the rows

```text
<table> ၏ အညွှန်းအရေအတွက်
<table> ၏ အညွှန်းတိုင်းကို <name> အနေဖြင့် ယူ၍ BODY
<table> ၏ အတန်းတိုင်းကို <name> အနေဖြင့် ယူ၍ BODY
<table> ၏ အညွှန်းတိုင်းကို <index name> နှင့် <value name> အနေဖြင့် ယူ၍ BODY
```

`အညွှန်းအရေအတွက်` ("index count") is the number of rows, a `usize`. To walk the indexes,
we loop over `အညွှန်းတိုင်း` ("each index"); to walk the values, we loop over
`အတန်းတိုင်း` ("each row"). With two names, either loop gets the index and then the value.
The rows come in their order, the first added first.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဇ သည် ["နီ": ၁၊ "ဝါ": ၂၊ "ပြာ": ၃] ဖြစ်ပါစေ။
ဇ ၏ အညွှန်းအရေအတွက် ကို ပြသမည်။
ဇ ၏ အညွှန်းတိုင်းကို အရောင် အနေဖြင့် ယူ၍ အရောင် ကို ပြသမည်။
ဇ ၏ အတန်းတိုင်းကို ဖ အနေဖြင့် ယူ၍ ဖ ကို ပြသမည်။
ဇ ၏ အညွှန်းတိုင်းကို အရောင် နှင့် န အနေဖြင့် ယူ၍ "{အရောင်} = {န}" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၃
နီ
ဝါ
ပြာ
၁
၂
၃
နီ = ၁
ဝါ = ၂
ပြာ = ၃
```

- A loop walks the table as it was when the loop began, because the loop works on a
  duplicate. So a set or a remove in its BODY changes the table and not the rows the loop
  walks.
- Each name gets a duplicate of the index or the value, as the variable of an each-cell
  loop gets a duplicate of a cell. The [loop rule](control-flow.md#each-cell-loops) is the same for
  lists, text and tables.
- `အညွှန်းတိုင်း` or `အတန်းတိုင်း` on a value that is not a table is ZK0484, and
  `အကွက်တိုင်း` or `စာလုံးတိုင်း` on a table is ZK0434.

Since the loop walks a duplicate, we can remove each row inside it:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဇ သည် ["က": ၁၊ "ခ": ၂၊ "ဂ": ၃] ဖြစ်ပါစေ။
ဇ ၏ အညွှန်းတိုင်းကို စ အနေဖြင့် ယူ၍ ဇ ထဲမှ အညွှန်း စ ကို ဖျက်မည်။
ဇ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[:]
```

## Index kinds

An index kind is a data kind whose values can be hashed and compared as whole values:
integers, characters, true/false values, text, enumerations, decimal numbers, and lists
and structs whose parts are all index kinds. Two indexes are the same index when their
parts are equal, text through its characters and a list through its cells.

```zaka
အမှတ် ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု နှင့် y ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည် သည် အမှတ် ဖြင့် ညွှန်းသော စာသား ဇယား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [:] ဖြစ်ပါစေ။
အမည် ထဲမှ အညွှန်း အမှတ်(x: ၀၊ y: ၀) ၏ တန်ဖိုး ကို "မူလ" ဟု ထားမည်။
အမည် ထဲမှ အညွှန်း အမှတ်(x: ၀၊ y: ၀) ၏ တန်ဖိုး ထဲမှ ဒေတာ ကို ပြသမည်။
ကိန်းရှင် လမ်း သည် ကိန်းပြည့် စာရင်း ဖြင့် ညွှန်းသော စာသား ဇယား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [:] ဖြစ်ပါစေ။
လမ်း ထဲမှ အညွှန်း [၁၊ ၂] ၏ တန်ဖိုး ကို "ညာ၊ အောက်" ဟု ထားမည်။
လမ်း ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မူလ
[[၁၊ ၂]: "ညာ၊ အောက်"]
```

Shared objects, function values, pointers, optionals, results and tables are not index
kinds, because each of them can change or has no single value to compare. An index kind
that contains one of them is ZK0485.

```zaka-error ZK0485
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဇ သည် ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် ဖြင့် ညွှန်းသော စာသား ဇယား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [:] ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0485]: optional isize cannot index the rows of a ဇယား: optional isize cannot be hashed and compared as a whole value
```

### Decimal indexes

A decimal number can index rows, with the rule of the JS `Map`: all NaN values are one
index, `-0.0` and `0.0` are one index, and the other values are compared as `==` compares
them. The row keeps the index it was first added with. `==` on decimal numbers does not
change.

Each index kind that contains a decimal number gets the warning ZK0487, because
`၀.၁ + ၀.၂` and `၀.၃` are two rows when rounding sets them apart. An index that counts in
whole steps, such as cents for a price, keeps the rows apart as written.

```zaka-warning ZK0487
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဇ သည် ဒဿမကိန်း ဖြင့် ညွှန်းသော စာသား ဇယား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [:] ဖြစ်ပါစေ။
ဇ ထဲမှ အညွှန်း ၀.၁ + ၀.၂ ၏ တန်ဖိုး ကို "က" ဟု ထားမည်။
ဇ ထဲမှ အညွှန်း ၀.၃ ၏ တန်ဖိုး ကို "ခ" ဟု ထားမည်။
ဇ ထဲမှ အညွှန်း -၀.၀ ၏ တန်ဖိုး ကို "ဂ" ဟု ထားမည်။
ဇ ထဲမှ အညွှန်း ၀.၀ ၏ တန်ဖိုး ကို "ဃ" ဟု ထားမည်။
ဇ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0487]: f64 holds a decimal number, so two values that round apart are two rows
```

The table it prints has three rows: `[၀.၃၀၀၀၀၀၀၀၀၀၀၀၀၀၀၀၀၄: "က"၊ ၀.၃: "ခ"၊ -၀.၀: "ဃ"]`.
The sum and `၀.၃` are two rows, and `၀.၀` gives its value to the row of `-၀.၀`, which
keeps its index.

## Equal tables

Two tables are `==` when the two have equal indexes with equal values, whatever the order of
their rows. The order of the rows shows in a print and in a loop alone.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ["တစ်": ၁၊ "နှစ်": ၂] ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ["နှစ်": ၂၊ "တစ်": ၁] ဖြစ်ပါစေ။
က သည် ခ နှင့် ညီ ကို ပြသမည်။
ခ ထဲမှ အညွှန်း "နှစ်" ၏ တန်ဖိုး ကို ၃ ဟု ထားမည်။
က သည် ခ နှင့် ညီ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
မမှန်
```

## Duplicates

A table is a value, as a list is. When we assign a table, pass it or return it, the other
side gets a duplicate (copy). The two share their rows until one side writes, and the
write then duplicates the rows first. To see one table through several names, we put it
in a field of a shared object ([Values and objects](objects.md)).

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ["တစ်": ၁] ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။
ခ ထဲမှ အညွှန်း "နှစ်" ၏ တန်ဖိုး ကို ၂ ဟု ထားမည်။
က ကို ပြသမည်။
ခ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
["တစ်": ၁]
["တစ်": ၁၊ "နှစ်": ၂]
```

## How a table is stored

A table is a struct of three parts: its rows, a plain list in their order; its slots, a
plain list of `u32`; and its count of live rows. Each row has the hash of its index, a
live mark, the index and the value.

```text
  table: map<text, isize>, with the rows ["နီ": 1, "ပြာ": 2]

  rows   (plain list)    +------+------+-------+-------+   +------+------+--------+-------+
                         | hash | live | "နီ"  |   1   |   | hash | live | "ပြာ"  |   2   |
                         +------+------+-------+-------+   +------+------+--------+-------+
  slots  (u32[8])        [ 0 | 3 | 0 | 0 | 2 | 0 | 0 | 0 ]   2 + the row number, where its hash lands
  live                   2
```

- A slot is 0 when no row has used it, 1 when its row was removed, and otherwise 2 more
  than the number of its row. The count of slots is a power of two, at least 8.
- An index goes to the slot that its hash names, or to the next slot after it that has
  no other row. A search walks the same slots, so it ends at a slot that is 0.
- At least a third of the slots are 0. An add that would fill more moves the live rows
  together, in their order, and builds the slots again, with twice the count that the
  live rows need.
- A remove drops the index and the value of the row, leaves the row as zero octets, and
  marks its slot 1. A table left with no live row frees its blocks.
- The hash is FNV-1a over 64 bits on the parts of the index, so a print and a loop do not
  depend on it: the order of the rows is the order of their adds.

## Errors at a glance

| Code | What went wrong |
| --- | --- |
| ZK0222 | a data kind after a literal that is not a table |
| ZK0306 | a set or a remove of a row of a constant |
| ZK0332 | `ကို ဖျက်မည်` after a value that is not a row |
| ZK0401 | an index that does not convert to the index kind |
| ZK0434 | `အကွက်တိုင်း` or `စာလုံးတိုင်း` on a table |
| ZK0484 | a row read, a count or a loop of rows on a value that is not a table |
| ZK0485 | an index kind that cannot be hashed and compared as a whole value |
| ZK0486 | a blank table with no data kind from its place |
| ZK0487 | a warning: an index kind that contains a decimal number |
| ZK0509 | a warning: a measure word other than `ခု` after a table kind |
