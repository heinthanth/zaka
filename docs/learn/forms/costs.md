---
title: What a duplicate costs
description: Where text and lists are duplicated, when the compiler moves or lends them instead, what an edit of text costs, and how --warn-duplicates names each duplicate left.
---

# What a duplicate costs

In Part I, each assignment, each argument and each result gives a value of its own: a
*duplicate*. For a number, a duplicate costs next to nothing. Text and lists built while
the program runs keep their cells in a heap block, so a duplicate of one asks the heap for
a new block and copies each cell into it. The program means the same thing whatever the
compiler does about it; this page tells when the work happens, and how to see it.

## Where a duplicate is made

A duplicate of text or of a list is made where the value goes into another place:

- a declaration or a set from another variable: `ခ ကို က ဟု ထားပါ။`;
- an argument, given to a parameter of a plain kind;
- a result, given back to the caller;
- a value added to a list, or put into a cell or a field;
- an each-cell loop, which walks a duplicate of its list.

The last one shows in a run: the loop below writes cell 0 of the list each round, and the
loop still reads the cells the list held when the loop began.

```zaka
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက်တိုင်း ကို ကိန်း အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းများ ၏ အကွက် ၀ ကို ကိန်း ဟု ထားပါ။     // the list changes, the walk does not
ကိန်း ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
ကိန်းများ ကို ပြသပါ။
```

```text
၁
၂
၃
[၃၊ ၂၊ ၃]
```

## Where nothing is copied

The compiler skips the work where the program cannot tell the difference:

- **The last use moves.** When the place read is not read again, the new place takes its
  cells, and nothing is copied.
- **Literals stand still.** A text literal or a text constant sits in the program itself,
  and nothing writes it, so a duplicate of one takes no block until its first edit (see
  [The cost of an edit](#the-cost-of-an-edit)).
- **A parameter reads in place.** Text given to a function and read again after the call
  is lent to the parameter for the length of the call, until its first character write or
  add, which gives it cells of its own. A list or a struct parameter that
  the function never writes reads the caller's cells; a plain list parameter reads them
  until its first write, which then gives it cells of its own.

```zaka
ကိန်းရှင် i သည် ကိန်းပြည့် ၇ ဖြစ်ပါစေ။
ကိန်းရှင် ပထမ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "Zaka {i}" ဖြစ်ပါစေ။   // built as the program runs
ကိန်းရှင် ဒုတိယ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ပထမ ဖြစ်ပါစေ။        // a duplicate: ပထမ is read again
ကိန်းရှင် တတိယ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ပထမ ဖြစ်ပါစေ။        // a move: the last read of ပထမ
ဒုတိယ ကို ပြသပါ။
တတိယ ကို ပြသပါ။
```

```text
Zaka ၇
Zaka ၇
```

The move is never seen, since nothing reads the old place again. It is not made for a
place that a view, a slice or an address reads, nor for a place that one statement
reads twice.

## The cost of an edit

A [character write](../values/strings.md#changing-a-character) and an
[add](../values/strings.md#adding-to-text) change text in place, and each has a cost of
its own:

- **The first edit copies text with no block of its own.** A literal, a duplicate of one,
  and text lent to a parameter read cells that the text does not own. Their first edit
  copies the cells into a new heap block, so the literal and the caller's text stand
  still.
- **An add fills spare room first.** The new cells go into the block when the block has room. When
  it has none, the text moves to a block twice its size, 16 cells at the least, so a loop
  of adds copies the text a few times, not once for each round.
- **A character of another size shifts the rest.** `'a'` takes one cell and `'က'` three, so
  a write of one in place of the other moves each cell after it.
- **A character write walks from the start.** A write finds its character as a read does,
  from the first cell, so a counted loop that writes each character in turn takes time in
  the square of the count.

```zaka
လုပ်ဆောင်ချက် အော် သည် စာသား စ ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ထဲသို့ "!" ကို ထည့်ပါ။     // the first edit copies the caller's cells
စ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။

ကိန်းရှင် အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မမ" ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် အော် ကို အမည် တို့ဖြင့် လုပ်ဆောင်ပါ။
အမည် ကို ပြသပါ။
```

```text
မမ!
မမ
```

## Seeing each duplicate

`--warn-duplicates` makes the compiler name each statement where a duplicate is left
after the moves (ZK0720). It changes nothing about the run. Save the program of `ပထမ`,
`ဒုတိယ` and `တတိယ` from [Where nothing is copied](#where-nothing-is-copied) as
`costs.zaka` and run it with the flag:

```sh
zakac run --warn-duplicates costs.zaka
```

```text
warning[ZK0720]: "ပထမ" is duplicated here; text on the heap takes a new block for the duplicate
 --> costs.zaka:3:50
  |
3 | ကိန်းရှင် ဒုတိယ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ပထမ ဖြစ်ပါစေ။        // a duplicate: ပထမ is read again
  |                                    ^^^
  = help: lend a view to read it with no duplicate, or read it no more after this statement, so it moves
Zaka ၇
Zaka ၇
```

Line 3 is named; line 4, the move, is not. The flag belongs to the borrow checker, so
`zakac borrow-check --warn-duplicates` takes it too.

## Removing a duplicate

The help line of the warning names the two routes:

- **Lend a view or a slice** where the cells are read and not kept: a parameter of
  `စာသား တစ်စိတ်တစ်ပိုင်း` or `ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း`, given
  `<value> ၏ တန်ဖိုးအငှား`. See [Views of text](views.md) and
  [Slices](../lists/slices.md).
- **Read the place no more** after the statement, so that the compiler moves it.

A function that must change the caller's list or struct needs no duplicate either: mark
the parameter and the argument, as in [Changing the caller's
variable](../functions/parameters.md#changing-the-callers-variable).

Part III gives full control of storage: [Pointers and the heap](../pointers/README.md)
asks the heap for blocks of its own, and the owner of each block frees it.

## Hands on

- Run the program of `ပထမ` with `--warn-duplicates`, then print `ပထမ` at the end and run it
  again: which lines does the warning name now?
- Write a function that prints a list, call it with a list the program reads again, and
  run it with `--warn-duplicates`. The function never writes its parameter, so it reads the
  caller's cells, and the warning names nothing.
- Change the loop above so that it walks the cell numbers with a counted loop, and compare
  the output.

Next: [Pointers and the heap](../pointers/README.md).
