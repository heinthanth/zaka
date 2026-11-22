---
title: Tables
description: Find a value through an index instead of a number, the same as a phone book finds a number through a name, and set, remove and walk through the rows.
---

# Tables

A *table* (map) lets us look up a value using an *index* (key), much as we look up a
phone number using a name. In a Zaka table, written `ဇယား`, each row pairs one index
(`အညွှန်း`) with one value (`တန်ဖိုး`). Each index identifies a single row.

## You will learn

- how to create a table and write one out as a literal
- how to read the row of an index, which might be missing
- how to set and remove rows, and how to count them
- how to visit each row with a loop

```zaka
ကိန်းရှင် ဖုန်း သည် ["မမ": ၉၅၁၁၁၊ "ကို": ၉၅၂၂၂] ဖြစ်ပါစေ။
ကိန်းရှင် အသစ် သည် စာသား ဖြင့် ညွှန်းသော ကိန်းပြည့် ဇယား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [:] ဖြစ်ပါစေ။
ဖုန်း ကို ပြသမည်။
အသစ် ကို ပြသမည်။
```

```text
["မမ": ၉၅၁၁၁၊ "ကို": ၉၅၂၂၂]
[:]
```

The first table, `ဖုန်း` (phone), has two rows, and each row has a name as its index and a
phone number as its value. We can use these patterns to write the data kind (type) of a
table and a table literal:

```text
<index kind> ဖြင့် ညွှန်းသော <value kind> ဇယား
[<index>: <value>၊ <index>: <value>]
```

| Zaka | Reads as |
| --- | --- |
| `စာသား ဖြင့် ညွှန်းသော` | "indexed with text" |
| `ကိန်းပြည့် ဇယား` | "an integer table", so each value is an integer |
| `"မမ": ၉၅၁၁၁` | one row: the index `"မမ"` and the value `၉၅၁၁၁` |
| `[:]` | a table with no rows |

When the table literal has rows, the variable gets its data kind from the first row, so
`ဖုန်း` needs no data kind. For a table with no rows, `[:]`, we write the data kind in
the declaration, as `အသစ်` (new) shows.

## Table literals in words

```text
<index> အနေဖြင့် <value> နှင့် <index> အနေဖြင့် <value> တို့ပါသော ဇယား
```

We can also write a table literal in words, with `<index> အနေဖြင့် <value>` ("with the
index, the value") for each row, the same as a list literal in words. A table
prints in the bracket form, and `ဇယားအလွတ်` ("blank table") is the word form of `[:]`:

```zaka
ကိန်းရှင် ဖုန်း သည် "မမ" အနေဖြင့် ၉၅၁၁၁ နှင့် "ကို" အနေဖြင့် ၉၅၂၂၂ တို့ပါသော ဇယား ဖြစ်ပါစေ။
ဖုန်း ကို ပြသမည်။
ကိန်းရှင် အသစ် သည် ကိန်းပြည့် ဖြင့် ညွှန်းသော စာသား ဇယား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဇယားအလွတ် ဖြစ်ပါစေ။
အသစ် ကို ပြသမည်။
```

```text
["မမ": ၉၅၁၁၁၊ "ကို": ၉၅၂၂၂]
[:]
```

## Reading a row

```text
<table> ထဲမှ အညွှန်း <index> ၏ တန်ဖိုး
<table> ထဲမှ အညွှန်း <index> ၏ တန်ဖိုး ထဲမှ ဒေတာ၊ မရှိလျှင် <value>
```

To read the row of an index, we write `<table> ထဲမှ အညွှန်း <index> ၏ တန်ဖိုး` ("from the
table, the value of the index"). A name that is not in the phone book has no row, so the
read returns an [optional value](../optionals/README.md): the value in a data cell, or
`အလွတ်` (blank) when the row is missing. To get a plain value instead, we add `ထဲမှ ဒေတာ၊
မရှိလျှင် <value>` ("the data, or if none, `<value>`"):

```zaka
ကိန်းရှင် ဖုန်း သည် ["မမ": ၉၅၁၁၁၊ "ကို": ၉၅၂၂၂] ဖြစ်ပါစေ။
ဖုန်း ထဲမှ အညွှန်း "မမ" ၏ တန်ဖိုး ကို ပြသမည်။
ဖုန်း ထဲမှ အညွှန်း "ဇော်" ၏ တန်ဖိုး ကို ပြသမည်။
ဖုန်း ထဲမှ အညွှန်း "ဇော်" ၏ တန်ဖိုး ထဲမှ ဒေတာ၊ မရှိလျှင် ၀ ကို ပြသမည်။
အကယ်၍ ဖုန်း ထဲမှ အညွှန်း "ကို" ၏ တန်ဖိုး ထဲတွင် ဒေတာ ရှိလျှင် "ကို ၏ ဖုန်း ရှိသည်" ဟု ပြသမည်။
```

```text
ဒေတာ(၉၅၁၁၁)
အလွတ်
၀
ကို ၏ ဖုန်း ရှိသည်
```

| Zaka | Reads as |
| --- | --- |
| `ဖုန်း ထဲမှ` | "from the phone book" |
| `အညွှန်း "မမ" ၏ တန်ဖိုး` | "the value of the index `"မမ"`" |
| `ထဲမှ ဒေတာ၊ မရှိလျှင် ၀` | "the data, or if none, 0" |
| `ထဲတွင် ဒေတာ ရှိလျှင်` | "if there is data in it" |

A missing row never stops the program, because the read returns `အလွတ်` for it. The last
line tests whether the row exists, the same as for each other optional value.

## Setting and removing rows

```text
<table> ထဲမှ အညွှန်း <index> ၏ တန်ဖိုး ကို <value> ဟု ထားမည်။
<table> ထဲမှ အညွှန်း <index> ကို ဖျက်မည်။
<table> ၏ အညွှန်းအရေအတွက်
```

We use `ထားမည်` to set the value of a row, and when no row has that index, the table gets
a new row at the end. We use `ဖျက်မည်` to remove the row of an index, and
`အညွှန်းအရေအတွက်` ("index count") gives the number of rows:

```zaka
ကိန်းရှင် ဖုန်း သည် ["မမ": ၉၅၁၁၁၊ "ကို": ၉၅၂၂၂] ဖြစ်ပါစေ။
ဖုန်း ထဲမှ အညွှန်း "ဇော်" ၏ တန်ဖိုး ကို ၉၅၃၃၃ ဟု ထားမည်။ // a new row, at the end
ဖုန်း ထဲမှ အညွှန်း "မမ" ၏ တန်ဖိုး ကို ၉၅၁၀၀ ဟု ထားမည်။ // a new value, same place
ဖုန်း ထဲမှ အညွှန်း "ကို" ကို ဖျက်မည်။
ဖုန်း ကို ပြသမည်။
ဖုန်း ၏ အညွှန်းအရေအတွက် ကို ပြသမည်။
```

```text
["မမ": ၉၅၁၀၀၊ "ဇော်": ၉၅၃၃၃]
၂
```

A row that gets a new value keeps its place, so the rows remain in the order in which we
first added them. Removing an index that has no row does nothing.

## Walking the rows

```text
<table> ၏ အညွှန်းတိုင်းကို <name> အနေဖြင့် ယူ၍ <sentence>
<table> ၏ အတန်းတိုင်းကို <name> အနေဖြင့် ယူ၍ <sentence>
<table> ၏ အညွှန်းတိုင်းကို <index name> နှင့် <value name> အနေဖြင့် ယူ၍ <sentence>
```

To visit each row, we can use `အညွှန်းတိုင်း` ("each index") to get the indexes, or
`အတန်းတိုင်း` ("each row") to get the values. With two names, the loop gets the index and
then the value of each row. The rows come in the order in which we added them:

```zaka
ကိန်းရှင် ဖုန်း သည် ["မမ": ၉၅၁၁၁၊ "ကို": ၉၅၂၂၂] ဖြစ်ပါစေ။
ဖုန်း ၏ အညွှန်းတိုင်းကို အမည် အနေဖြင့် ယူ၍ အမည် ကို ပြသမည်။
ဖုန်း ၏ အတန်းတိုင်းကို နံပါတ် အနေဖြင့် ယူ၍ နံပါတ် ကို ပြသမည်။
ဖုန်း ၏ အညွှန်းတိုင်းကို အမည် နှင့် နံပါတ် အနေဖြင့် ယူ၍ "{အမည်}: {နံပါတ်}" ဟု ပြသမည်။
```

```text
မမ
ကို
၉၅၁၁၁
၉၅၂၂၂
မမ: ၉၅၁၁၁
ကို: ၉၅၂၂၂
```

## Counting words

A table is good at counting things. In this example, for each word, we read its count so
far, with 0 for a word we have not seen, and set the count one higher:

```zaka
ကိန်းရှင် စကားလုံးများ သည် ["နေ"၊ "လ"၊ "နေ"၊ "ကြယ်"၊ "နေ"] ဖြစ်ပါစေ။
ကိန်းရှင် ရေ သည် စာသား ဖြင့် ညွှန်းသော ကိန်းပြည့် ဇယား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [:] ဖြစ်ပါစေ။
စကားလုံးများ ၏ အကွက်တိုင်း ကို စ အနေဖြင့် ယူ၍
ရေ ထဲမှ အညွှန်း စ ၏ တန်ဖိုး ကို ရေ ထဲမှ အညွှန်း စ ၏ တန်ဖိုး ထဲမှ ဒေတာ၊ မရှိလျှင် ၀ + ၁ ဟု ထားမည်။
ရေ ကို ပြသမည်။
```

```text
["နေ": ၃၊ "လ": ၁၊ "ကြယ်": ၁]
```

In `ထဲမှ ဒေတာ၊ မရှိလျှင် ၀ + ၁`, the `+ ၁` applies to the result of the whole read, so we
get the count plus 1, or 0 plus 1 for a new word.

## Details

- A table is a value, the same as a list, so assigning it or passing it to a function
  makes a duplicate (copy) ([Duplicates](../../manual/language/tables.md#duplicates)).
- Two tables are equal with `==` when both have the same indexes with the same values, in
  whatever order ([Equal tables](../../manual/language/tables.md#equal-tables)).
- An index can be text, a number, a character, a true/false value, an enumeration, or a
  list or a struct made of these, but not a shared object, an optional value or another
  table. A decimal number as the index gives the warning ZK0487, because rounding can turn
  two equal-looking sums into two rows ([Index
  kinds](../../manual/language/tables.md#index-kinds)).
- We can change the value inside a row in place, such as adding to a list in a row, but
  the row must exist, or the program stops ([Writing inside a
  row](../../manual/language/tables.md#writing-inside-a-row)).
- A loop over the rows walks a duplicate of the table, so a set or a remove inside the loop
  does not change the rows that the loop visits ([Counting and walking the
  rows](../../manual/language/tables.md#counting-and-walking-the-rows)).
- `[:]` alone, where no data kind of a table is known, is the error ZK0486, and error
  messages write a table kind as `map<text, isize>`
  ([Literals](../../manual/language/tables.md#literals)).

## Recap

- `<index kind> ဖြင့် ညွှန်းသော <value kind> ဇယား` is a table kind, and `[<index>: <value>]`
  is a table literal, with `[:]` for a table with no rows.
- `<table> ထဲမှ အညွှန်း <index> ၏ တန်ဖိုး` reads a row as an optional value, so a missing
  row gives `အလွတ်`.
- `ထားမည်` sets a row or adds it at the end, and `ဖျက်မည်` removes it.
- `အညွှန်းတိုင်း` and `အတန်းတိုင်း` visit the rows in the order in which the rows were added.

## Hands on

- Make a table of three friends and their ages, then print the age of one of them.
- Read the age of a name that is not in the table, once as an optional value and once
  with `မရှိလျှင်`.
- Count the characters of a text with a table indexed with `စာလုံး`.
- Remove a row, then print the number of rows.

Next: [Structs](../structs/README.md).
