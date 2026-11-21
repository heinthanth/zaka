---
title: What a duplicate costs
description: Where text and lists are duplicated, when the compiler shares or moves them instead, what an edit of text costs, and how a program steers the ring collector.
---

# What a duplicate costs

Each assignment, each argument and each result makes a separate value, a *duplicate*
(copy). For a number, a duplicate costs next to nothing, but text and lists that are built
while the program runs keep their cells in a block on the heap (dynamic memory), and a
full duplicate of one needs a new block and a copy of each cell. The program means the
same thing whatever the compiler does about it, and this page shows when the work
happens.

## You will learn

- How a duplicate shares its cells until one side writes (copy on write).
- Where a duplicate is made, and where the compiler skips it.
- What an edit of text costs.
- How to pause, run and read the ring collector.

## A duplicate shares until a write

A duplicate of text or of a list does not copy the cells at once. The two values
share one block, which keeps a count of the values that use it, and the first write to
either value copies the cells, so that the other value does not change:

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

In the Explorer, with debug mode on, the heap view shows one block for the two lists until
the write, and a new block at the write. So a program that duplicates a long list and
never writes the duplicate spends nothing on it.

## Where a duplicate is made

A duplicate is made wherever a value goes into another place:

- a declaration or an assignment from another variable: `ခ ကို က ဟု ထားမည်။`;
- an argument, which goes to a parameter;
- a result, which goes back to the caller;
- a value that we add to a list, or put into a cell or a field;
- a loop over each cell or character that changes its list or text, which walks a
  duplicate of it.

The last case shows in this program: the loop writes cell 0 of the list in each round,
but the loop still reads the cells that the list had when the loop began:

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

A [shared object](../structs/objects.md) (reference type) is never duplicated in these
places: each of them shares the one object.

## Where nothing is copied

The compiler skips the work where the program cannot tell the difference:

- **A duplicate that is never written** shares its cells for good, as above.
- **The last use moves.** When the place that we read is not read again, the new place
  takes its cells, and the count of the block does not even change.
- **Literals do not move.** A text literal, a list literal of known values and a constant of
  either one sit in the program itself, and nothing writes them, so a duplicate of one
  needs no block until its first edit (see [Static data and counted
  lists](counted-lists.md#where-the-cells-live)).
- **A loop that just reads** walks the cells in place.
- **A parameter that the function never writes** reads the caller's cells. A function
  that writes its parameter copies the cells at the first write.

So a program that passes its lists to functions that just read them, as the
[Scores](../programs/scores.md) program does, copies no cell at all.

## The cost of an edit

A [character write](../text/characters.md#changing-a-character) and an
[add](../text/changing.md#adding-to-text) change text in place, and each one has its own
cost:

- **The first edit copies shared text.** A literal, a duplicate of text, and text passed
  to a parameter share their cells with another place, so their first edit copies the
  cells into a new heap block, and the literal and the caller's text do not change.
- **An add fills spare room first.** The new cells go into the block when it has room.
  When it has none, the text moves to a block twice its size, at least 16 cells, so a loop
  of adds copies the text a few times, not once in each round.
- **A character of another size shifts the rest.** `'a'` takes one cell and `'က'` takes
  three, so writing one in place of the other moves each cell after it.
- **A character write walks from the start.** A write finds its character the same as a
  read does, from the first cell, so a counted loop that writes each character in turn
  takes time in the square of the count.

In this program, the function adds to its parameter, so the first edit copies the
caller's cells, and the caller's text does not change:

```zaka
အော် ဟူသော လုပ်ဆောင်ချက် သည် စ ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
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

## Rings on demand

Most blocks are freed as soon as their count reaches zero, while the ring collector
frees the objects that point at each other in a ring ([Objects that point at
each other](../structs/objects.md#objects-that-point-at-each-other)). The collector runs
on its own, but we can also steer it: we can pause it where the program must not halt for
it, and run it where the program has time. Here a loop leaves 100 rings behind while the
collector rests, and one sentence frees them all:

```zaka
အဆစ် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် တန် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် နောက် ဟူသော အဆစ် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု တို့ပါဝင်သည်။

ကွင်းဆောက် ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အဆစ်(တန်: ၁၊ နောက်: အလွတ်) ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အဆစ်(တန်: ၂၊ နောက်: က) ဖြစ်ပါစေ။
က ၏ နောက် ကို ခ ဟု ထားမည်။                      // a ring of two nodes, left behind
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အလိုအလျောက် မင်မိုရီ ရှင်းလင်းမှုကို ခေတ္တရပ်မည်။      // pause
၁ မှ ၁၀၀ အထိ န အနေဖြင့် ရေတွက်၍ ကွင်းဆောက် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
မင်မိုရီရှိ မလိုအပ်သော အရာများကို ရှင်းလင်းမည်။        // free the 100 rings now
အလိုအလျောက် မင်မိုရီ ရှင်းလင်းမှုကို ပြန်လည်စတင်မည်။    // resume
"ပြီး" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ပြီး
```

These four sentences steer the collector:

| Zaka | Reads as | What it does |
| --- | --- | --- |
| `အလိုအလျောက် မင်မိုရီ ရှင်းလင်းမှုကို ခေတ္တရပ်မည်။` | "will pause the automatic cleanup of memory" | pauses the collector |
| `မင်မိုရီရှိ မလိုအပ်သော အရာများကို ရှင်းလင်းမည်။` | "will clean the unneeded things in memory" | runs the collector now |
| `အလိုအလျောက် မင်မိုရီ ရှင်းလင်းမှုကို ပြန်လည်စတင်မည်။` | "will restart the automatic cleanup of memory" | resumes the collector |
| `အလိုအလျောက် မင်မိုရီ ရှင်းလင်းမှုကို <n> ဘိုက် နှင့်အထက်မှ လုပ်ဆောင်မည်။` | "will do the automatic cleanup from `n` octets and up" | sets how full the list of waiting blocks gets, in octets (bytes), before the collector runs on its own |

## Reading the collector

```
အလိုအလျောက် မင်မိုရီ ရှင်းလင်းသည့် အခြေအနေ
```

This expression returns the stats of the collector as a struct of the core module,
`အလိုအလျောက်မင်မိုရီရှင်းလင်းမှုအခြေအနေပြကိန်းဂဏန်းများ`. Its fields tell the number of
collections that ran, the octets that the collections freed, the octets waiting in the
list, the octets that start a run, and whether the runs are paused. In this program, each
ring leaves one node of 40 octets waiting, and the collection frees both nodes of each
ring:

```zaka
အဆစ် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် တန် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် နောက် ဟူသော အဆစ် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု တို့ပါဝင်သည်။

ကွင်းဆောက် ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အဆစ်(တန်: ၁၊ နောက်: အလွတ်) ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အဆစ်(တန်: ၂၊ နောက်: က) ဖြစ်ပါစေ။
က ၏ နောက် ကို ခ ဟု ထားမည်။                      // a ring of two nodes, left behind
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အလိုအလျောက် မင်မိုရီ ရှင်းလင်းမှုကို ခေတ္တရပ်မည်။
၁ မှ ၁၀၀ အထိ န အနေဖြင့် ရေတွက်၍ ကွင်းဆောက် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စံချိန် သည် အလိုအလျောက် မင်မိုရီ ရှင်းလင်းသည့် အခြေအနေ ဖြစ်ပါစေ။
စံချိန် ၏ ရှင်းလင်းရန်စီစဥ်နေသောဘိုက်အရေအတွက် ကို ပြသမည်။       // waiting in the buffer
မင်မိုရီရှိ မလိုအပ်သော အရာများကို ရှင်းလင်းမည်။
စံချိန် ကို အလိုအလျောက် မင်မိုရီ ရှင်းလင်းသည့် အခြေအနေ ဟု ထားမည်။
စံချိန် ၏ အကြိမ်အရေအတွက် ကို ပြသမည်။                        // one collection so far
စံချိန် ၏ ရှင်းလင်းခဲ့သောဘိုက်အရေအတွက် ကို ပြသမည်။              // the octets it freed
စံချိန် ၏ ခေတ္တရပ်ထား ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၀၀၀
၁
၈၀၀၀
မှန်
```

## Details

- **Which blocks wait.** Text, lists of numbers and structs of numbers never wait for the
  collector, because these cannot be part of a ring ([When a block is
  freed](../../manual/language/objects.md#when-a-block-is-freed)).
- **The threshold.** At first, the collector runs when 16,384 octets of blocks are waiting,
  and again when the program ends. A small threshold keeps the heap small and runs the
  collector often, and a large one runs it less often and lets more rings wait.
- **During a pause.** Blocks still wait, and the end of the program still collects. When
  the list filled up during the pause, the resume collects at once ([When a block is
  freed](../../manual/language/objects.md#when-a-block-is-freed)).
- **The field names.** Docs lists each field of the stats struct with its data kind, and
  the rules for when a block is freed: see [When a block is
  freed](../../manual/language/objects.md#when-a-block-is-freed).
- **Two options for the compiler.** `--results` and `--scratch` change where the compiler
  builds list and struct results and scratch values, without changing what the program
  prints ([Options of the compile
  stages](../../manual/compiler/commands.md#options-of-the-compile-stages)).

## Recap

- A duplicate of text or a list shares its cells until one side writes, and the write
  copies the cells.
- A duplicate is made at each declaration, assignment, argument, result and add, but the
  compiler moves the value at its last use and reads in place where nothing writes.
- The first edit of shared text copies it, and an add grows the block to twice its size
  when it is full.
- Four sentences pause, run, resume and tune the ring collector, and one expression reads
  its stats.

## Hands on

- Duplicate a list into a second variable, write a cell of the duplicate, and watch the
  heap view of the Explorer in debug mode.
- Write a function that prints a list, and another that sets cell 0 of its list and
  prints it. Which one copies the cells?
- Change the loop that writes cell 0 so that it walks the cell numbers with a counted
  loop, and compare the output.

Next: [Unsafe Zaka](../pointers/README.md).
