---
title: What a duplicate costs
description: Where text and lists are copied, when the compiler shares or moves them instead, and what an edit of text costs.
---

# What a duplicate costs

Each assignment, each argument and each result gives a value of its own: a *duplicate*. For a
number, a duplicate costs next to nothing. Text and lists built while the program runs keep
their cells in a heap block, and a full duplicate of one asks the heap for a new block and
copies each cell into it. The program means the same thing whatever the compiler does
about it; this page tells when the work happens.

## A duplicate shares until a write

A duplicate of text or of a list does not duplicate the cells at once. The two share one block,
which keeps a count of the values that hold it, and the first write to either of them
copies the cells, so that the other stands still:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် i သည် ၇ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် က သည် [i၊ ၂၊ ၃] ဖြစ်ပါစေ။     // built as the program runs: a heap block
ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။           // shares the block of က
ခ ၏ အကွက် ၀ ကို ၉ ဟု ထားမည်။           // copies the cells, then writes
က ကို ပြသမည်။
ခ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၇၊ ၂၊ ၃]
[၉၊ ၂၊ ၃]
```

In the Explorer, with debug mode on, the heap view shows one block for the two lists
until the write, and a new block at it. A program that copies a long list and never writes the duplicate
spends nothing on it.

## Where a duplicate is made

A duplicate is made where a value goes into another place:

- a declaration or a set from another variable: `ခ ကို က ဟု ထားမည်။`;
- an argument, given to a parameter;
- a result, given back to the caller;
- a value added to a list, or put into a cell or a field;
- an each loop that changes its list or text, which walks a duplicate of it.

The last one shows in a run: the loop below writes cell 0 of the list each round, and the
loop still reads the cells the list held when the loop began.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက်တိုင်း ကို ကိန်း အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းများ ၏ အကွက် ၀ ကို ကိန်း ဟု ထားမည်။     // the list changes, the walk does not
ကိန်း ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ကိန်းများ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁
၂
၃
[၃၊ ၂၊ ၃]
```

A [shared object](../structs/objects.md) is never copied in these places: each of them
shares the one object.

## Where nothing is copied

The compiler skips the work where the program cannot tell the difference:

- **A duplicate that is never written** shares its cells for good, as above.
- **The last use moves.** When the place read is not read again, the new place takes its
  cells, and the count of the block does not even change.
- **Literals stand still.** A text literal, a list literal of known values and a constant
  of either sit in the program itself, and nothing writes them, so a duplicate of one takes
  no block until its first edit (see [Static data and counted lists](counted-lists.md#where-the-cells-live)).
- **A loop that reads alone** walks the cells in place.
- **A parameter the function never writes** reads the caller's cells. A function that
  writes its parameter copies the cells at the first write.

So a program that gives its lists to functions that read them, as the
[Scores](../programs/scores.md) program does, copies no cell at all.

## The cost of an edit

A [character write](../text/characters.md#changing-a-character) and an
[add](../text/changing.md#adding-to-text) change text in place, and each has a cost of
its own:

- **The first edit copies shared text.** A literal, a duplicate of text, and text given to a
  parameter share their cells with another place. Their first edit copies the cells into
  a new heap block, so the literal and the caller's text stand still.
- **An add fills spare room first.** The new cells go into the block when the block has room. When
  it has none, the text moves to a block twice its size, 16 cells at the least, so a loop
  of adds copies the text a few times, not once for each round.
- **A character of another size shifts the rest.** `'a'` takes one cell and `'က'` three, so
  a write of one in place of the other moves each cell after it.
- **A character write walks from the start.** A write finds its character as a read does,
  from the first cell, so a counted loop that writes each character in turn takes time in
  the square of the count.

```zaka
လုပ်ဆောင်ချက် အော် သည် စ ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ထဲသို့ "!" ကို ထည့်မည်။     // the first edit copies the caller's cells
စ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မမ" ဖြစ်ပါစေ။
အော် ဟူသော လုပ်ဆောင်ချက်ကို အမည် ဖြင့် လုပ်ဆောင်မည်။
အမည် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မမ!
မမ
```

## Hands on

- Duplicate a list into a second variable, write a cell of the duplicate, and watch the heap view
  of the Explorer in debug mode.
- Write a function that prints a list, and another that sets cell 0 of its list and
  prints it. Which one copies the cells?
- Change the loop above so that it walks the cell numbers with a counted loop, and compare
  the output.

Next: [Unsafe Zaka](../pointers/README.md).
