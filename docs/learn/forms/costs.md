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
- an each loop that changes its list or text, which walks a duplicate of it.

The last one shows in a run: the loop below writes cell 0 of the list each round, and the
loop still reads the cells the list held when the loop began.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက်တိုင်း ကို ကိန်း အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းများ ၏ အကွက် ၀ ကို ကိန်း ဟု ထားပါ။     // the list changes, the walk does not
ကိန်း ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ကိန်းများ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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
- **Literals stand still.** A text literal, a list literal of known values and a constant
  of either sit in the program itself, and nothing writes them, so a duplicate of one takes
  no block until its first edit (see [The cost of an edit](#the-cost-of-an-edit) and
  [Static data and counted lists](counted-lists.md#where-the-cells-live)).
- **A loop that reads alone lends.** An each loop that writes, moves or takes the
  address of nothing of its list or text, and calls no function that sets it, reads the
  cells in place.
- **A parameter reads in place.** Text given to a function and read again after the call
  is lent to the parameter for the length of the call, until its first character write or
  add, which gives it cells of its own. A list or a struct parameter that
  the function never writes reads the caller's cells; a plain list parameter reads them
  until its first write, which then gives it cells of its own.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် i သည် ကိန်းပြည့် ၇ ဖြစ်ပါစေ။
ကိန်းရှင် ပထမ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "Zaka {i}" ဖြစ်ပါစေ။   // built as the program runs
ကိန်းရှင် ဒုတိယ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ပထမ ဖြစ်ပါစေ။        // a duplicate: ပထမ is read again
ကိန်းရှင် တတိယ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ပထမ ဖြစ်ပါစေ။        // a move: the last read of ပထမ
ဒုတိယ ကို ပြသပါ။
တတိယ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
Zaka ၇
Zaka ၇
```

The move is never seen, since nothing reads the old place again. It is not made for a
place that a view, a slice or an address reads, nor for a place that one statement
reads twice.

## The cost of an edit

A [character write](../text/characters.md#changing-a-character) and an
[add](../text/changing.md#adding-to-text) change text in place, and each has a cost of
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
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မမ" ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် အော် ကို အမည် တို့ဖြင့် လုပ်ဆောင်ပါ။
အမည် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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
warning[ZK0720]: "ပထမ" is duplicated here; the duplicate can take a new heap block for its cells
 --> costs.zaka:3:50
  |
3 | ကိန်းရှင် ဒုတိယ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ပထမ ဖြစ်ပါစေ။        // a duplicate: ပထမ is read again
  |                                    ^^^
  = help: lend a view or a slice to read it with no duplicate, or read it no more after this statement, so it moves
Zaka ၇
Zaka ၇
```

Line 3 is named; line 4, the move, is not. The flag belongs to the borrow checker, so
`zakac borrow-check --warn-duplicates` takes it too.

## Removing a duplicate

The help line of the warning names the two routes:

- **Lend a view or a slice** where the cells are read and not kept: a parameter of
  `စာသား တစ်စိတ်တစ်ပိုင်း` or `ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း`, given
  `<value> ၏ တန်ဖိုးအငှား`. See [Borrowing text](views.md) and
  [Slices](../lists/slices.md).
- **Read the place no more** after the statement, so that the compiler moves it.

A function that must change the caller's list or struct needs no duplicate either: mark
the parameter and the argument, as in [Changing the caller's
variable](../functions/parameters.md#changing-the-callers-variable).

Part III gives full control of storage: [Pointers and the heap](../pointers/README.md)
asks the heap for blocks of its own, and the owner of each block frees it.

## The scores program with no duplicate

In the [Scores](../programs/scores.md) program of Part I, each call lends the list to
its function, which never writes it, and each function walks the list with an each-cell
loop that reads alone, so the loop reads the cells in place too. Run with
`--warn-duplicates`, it names nothing. That rests on what each function does inside:
a function that later writes its list takes a duplicate at each call. Below is the same
program with the two routes above, which put the promise in the signatures: each
function takes a slice or a view, each call lends `<list> ၏ တန်ဖိုးအငှား`, and the passing
scores go into a list the function builds. It names nothing either, and it prints what
the first one prints.

```zaka
ကိန်းသေ အောင်မှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၄၀ ဖြစ်ပါစေ။

/// The total of the scores.
လုပ်ဆောင်ချက် စုစုပေါင်း သည်
ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း အမှတ်များ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
အမှတ်များ ၏ အကွက်တိုင်း ကို အမှတ် အနေဖြင့် ယူ၍ ရလဒ် ကို ရလဒ် + အမှတ် ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

/// The cell number of the highest score, counting from 0.
လုပ်ဆောင်ချက် ထိပ်ဆုံး သည်
ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း အမှတ်များ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
ကိန်းရှင် နေရာ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက်တိုင်း ကို အမှတ် အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
အကယ်၍ အမှတ် သည် အမှတ်များ ၏ အကွက် ရလဒ် ထက် ကြီးလျှင် ရလဒ် ကို နေရာ ဟု ထားပါ။
နေရာ ကို နေရာ + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

/// A list of the scores at or above the pass mark.
လုပ်ဆောင်ချက် အောင်သောအမှတ်များ သည်
ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း အမှတ်များ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် စာရင်း ရလဒ် ကို ပြန်ပေးမည်။
အမှတ်များ ၏ အကွက်တိုင်း ကို အမှတ် အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
အကယ်၍ အမှတ် သည် အောင်မှတ် အောက် မငယ်လျှင် ရလဒ် ထဲသို့ အမှတ် ကို ထည့်ပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

/// Print each score above the average.
လုပ်ဆောင်ချက် ပျမ်းမျှအထက် သည်
ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း အမှတ်များ နှင့် ကိန်းပြည့် ပျမ်းမျှ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အမှတ်များ ၏ အကွက်တိုင်း ကို အမှတ် အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
အကယ်၍ အမှတ် သည် ပျမ်းမျှ ထက် ကြီးလျှင် အမှတ် ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

/// Print a name one character at a time.
လုပ်ဆောင်ချက် စာလုံးချင်း သည်
စာသား တစ်စိတ်တစ်ပိုင်း အမည် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အမည် ၏ စာလုံးတိုင်း ကို စ အနေဖြင့် ယူ၍ စ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၇၂၊ ၃၅၊ ၉၁၊ ၅၈၊ ၂၄] ဖြစ်ပါစေ။

// The program starts here
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည်များ သည် စာသား စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ["အောင်အောင်"၊ "သီတာ"၊ "မြမြ"၊ "ဇော်ဇော်"၊ "နန္ဒာ"] ဖြစ်ပါစေ။

ကိန်းရှင် အောင်သူ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အောင်သောအမှတ်များ(အမှတ်များ ၏ တန်ဖိုးအငှား) ဖြစ်ပါစေ။
အောင်သူ ကို ပြသပါ။
အောင်သူ ၏ နောက်ဆုံးအကွက်ကို ဖျက်ပါ။
အောင်သူ ကို ပြသပါ။

ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (အမှတ်များ ၏ အကွက်အရေအတွက်) ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် ပေါင်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် စုစုပေါင်း(အမှတ်များ ၏ တန်ဖိုးအငှား) ဖြစ်ပါစေ။
ကိန်းရှင် ထိပ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ထိပ်ဆုံး(အမှတ်များ ၏ တန်ဖိုးအငှား) ဖြစ်ပါစေ။
"အရေအတွက် {အရေအတွက်}" ဟု ပြသပါ။
"စုစုပေါင်း {ပေါင်း}" ဟု ပြသပါ။
"အများဆုံး {အမှတ်များ ၏ အကွက် ထိပ်}" ဟု ပြသပါ။
"ပျမ်းမျှ {ပေါင်း / အရေအတွက်} ထက် ကြီးသော အမှတ်များ" ဟု ပြသပါ။
လုပ်ဆောင်ချက် ပျမ်းမျှအထက် ကို အမှတ်များ ၏ တန်ဖိုးအငှား၊ ပေါင်း / အရေအတွက် တို့ဖြင့် လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် စာလုံးချင်း ကို အမည်များ ၏ အကွက် ထိပ် ၏ တန်ဖိုးအငှား တို့ဖြင့် လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၇၂၊ ၉၁၊ ၅၈]
[၇၂၊ ၉၁]
အရေအတွက် ၅
စုစုပေါင်း ၂၈၀
အများဆုံး ၉၁
ပျမ်းမျှ ၅၆ ထက် ကြီးသော အမှတ်များ
၇၂
၉၁
၅၈
မ
ြ
မ
ြ
```

What changed, and what each change saves:

- **Slice parameters** (`ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း`) read the caller's cells, and an
  each-cell loop over a slice walks those cells in place.
- **A text view** (`စာသား တစ်စိတ်တစ်ပိုင်း`) lends the top name, a cell of `အမည်များ`,
  to `စာလုံးချင်း`, and none of its text is copied.
- **A list built in the function** holds the passing scores. It is returned to `အောင်သူ`
  as a move, so its block is never copied, and it grows and shrinks in that block.

## Hands on

- Run the program of `ပထမ` with `--warn-duplicates`, then print `ပထမ` at the end and run it
  again: which lines does the warning name now?
- Write a function that prints a list, call it with a list the program reads again, and
  run it with `--warn-duplicates`. The function never writes its parameter, so it reads the
  caller's cells, and the warning names nothing.
- Change the loop above so that it walks the cell numbers with a counted loop, and compare
  the output.

Next: [Pointers and the heap](../pointers/README.md).
