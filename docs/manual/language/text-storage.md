---
title: How text is stored
description: The three words of text, how an edit grows its heap block, folding and constant text, duplicates and moves, the drops the compiler adds, and static text.
---

# How text is stored

Text (string) is *managed*: the compiler decides where its octets (bytes) live, shares
them between duplicates (copies) until one of them changes, and frees them when no
place needs them. A program never allocates or frees text on its own. This page covers
how that works: the three words of text, edits, folding, duplicates, drops and static
text. [Text](text.md) covers how we write, read and change text, and the Learn book
shows where text lives in [Static data and counted
lists](../../learn/forms/counted-lists.md).

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မမ" ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အမည် ဖြစ်ပါစေ။
ခ ကို ပြသမည်။
အမည် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မမ
မမ
```

## The three words

A `စာသား` value is three words: the address of its octets, their count, and its *room*,
the size of the heap block that contains them. On a 64-bit platform it needs 24 octets.
A plain list has the same three words ([How a list is
stored](counted-lists.md#how-a-list-is-stored)).

A room of 0 means that the octets are in no block that this text owns: the octets sit in
static storage, or a parameter reads the octets of the caller. A heap block counts the
texts that share it, and the block is freed when the count reaches 0.

## Edits and the heap block

When it runs, an [edit](text.md#changing-text) of text with a room of 0 (static text, or
a parameter that reads the octets of the caller), or of text whose block other text
shares, first gets its own heap block with the same octets. So a literal is never
written, a duplicate shares nothing once it changes, and a function that edits its
parameter leaves the text of the caller as it was.

The new octets go into the block when it has room. Otherwise the text moves to a block
twice the size, with room for 16 octets at the least. So a run of adds duplicates the text
a few times, not once for each add.

## Folding

With expression folding on (the default, where `--fold` is set to `exprs`), a literal
whose template slots all fold to a constant becomes static text when the program is
compiled, and it needs no heap. A template slot folds when it contains a literal, a
constant, or constant arithmetic on them. A `+` of two pieces of constant text folds in
the same manner.

A template slot with another value, such as a variable or a list, makes the literal a
*format*: the program builds its text on the heap when it runs. Under `--fold none`, each
template literal is built when the program runs. The output is the same in both cases.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ အကျယ် သည် ၈ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
"အကျယ် {အကျယ်}၊ ဧရိယာ {အကျယ် * အကျယ်}" ဟု ပြသမည်။ // static text
ကိန်းရှင် i သည် ၃ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
"အဆင့် {i}" ဟု ပြသမည်။ // built when it runs
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အကျယ် ၈၊ ဧရိယာ ၆၄
အဆင့် ၃
```

### Constant text

A `ကိန်းသေ` of the data kind `စာသား` (or a list or struct with text in it) is static
text in each build, as a constant plain list is static data ([Static
data](counted-lists.md#static-data)), so its value must fold at compile time whatever
`--fold` is set to. Its value can use literals, other constants, template slots of
these, `+` and calls of [comptime functions](compile-time.md#comptime-functions). A value
that needs the program to run, such as a variable in a template slot, a call of another
function, a cell or a field, is ZK0606, reported at the first part that does not fold.
`--fold` changes how the IR shows the reads of the constant, never whether it is valid.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မမ" ဖြစ်ပါစေ။
ကိန်းသေ နှုတ်ဆက် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "ဟေး {အမည်}" + "!" ဖြစ်ပါစေ။
နှုတ်ဆက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဟေး မမ!
```

```zaka-error ZK0606
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် n သည် ၃ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
ကိန်းသေ အဆင့် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "အဆင့် {n}" ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0606]: a constant that holds text or a plain list must be known at compile time
```

## Duplicates

`စာသား` is duplicated as a number is. When we set one text place from another
(`ခ ကို က ဟု ထားမည်။`), declare a variable from one, pass one to a `စာသား` parameter,
return one, or put one into a list or a struct field, the new place gets its own text,
and the old place keeps its text. A list or a struct with text in it is duplicated in the
same manner.

A duplicate costs nothing at first: the two places share one heap block, and the block
counts the texts that use it. The first [edit](text.md#changing-text) of either one gives
it its own block with the same octets, and the other keeps the old block. Static text is
shared in the same manner, with no count: nothing writes it, so a duplicate of static
text needs no block, and its first edit gives it a new heap block. The example at the top
of the page duplicates a constant in this manner.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "abc" ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် စာသား တစ်ခု ဖြစ်ပါစေ။
ခ ကို က ဟု ထားမည်။
ခ ကို ခ + "!" ဟု ထားမည်။
က ကို ပြသမည်။
ခ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
abc
abc!
```

### Moves

When the place that is read is not read again, the compiler moves the text instead, and
the count does not change: the new place gets the block, and the old one is left blank.
Nothing reads the old place, so the move never shows. The compiler does not make a move:

- from a parameter, whose text can belong to the caller;
- from a global read inside a function;
- from a global that a call in the same statement sets;
- from a place whose address unsafe code takes;
- from a place that the same statement reads twice.

### Lending to a parameter

When text passed to a function is still read after the call, the compiler lends it to
the parameter with a room of 0: the parameter reads the octets of the caller for the
length of the call, and the count does not change. The function still sees a separate
value. Setting the parameter gives it new text, an edit of it first duplicates the octets
of the caller into a new block, and returning it, storing it or setting another place
from it makes a duplicate there. The compiler shares the text through the count at the
call instead when the function, or a call inside the same arguments, sets the global
passed, or when unsafe code takes the address of the place.

## Drops

The compiler frees text, and a program has no statement for it. A place whose data kind
contains text (a `စာသား` variable or parameter, a struct with a `စာသား` field, a list of
`စာသား`) drops its text:

- at the end of its block;
- at the end of each round of a loop;
- before `ကွင်းဆက် ကို ရပ်မည်`, before `နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်` and before a return;
- before a new value replaces the old one;
- for a top-level variable, at the end of the program.

A short-lived text, such as a join, a range or a format that is printed or used in a
template slot, is dropped right after its use. A drop takes one off the count of the heap
block, and frees the block when the count reaches 0. It leaves alone static text (room 0)
and text that was moved out. `zakac run --leaks` lists none of these blocks. A pointer
next to the text in a struct or a list is not dropped: the block behind it, if it came
from a heap statement, is for unsafe code to free ([Unsafe Zaka](unsafe.md)).

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် i သည် ၁ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
i သည် ၁၀ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လိုင်း သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "အဆင့် {i}: {i * i}" ဖြစ်ပါစေ။
အကယ်၍ i သည် ၄ နှင့် ညီလျှင် ကွင်းဆက် ကို ရပ်မည်။
လိုင်း ကို ပြသမည်။
i ကို i + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အဆင့် ၁: ၁
အဆင့် ၂: ၄
အဆင့် ၃: ၉
```

## Static text

The compiled module has one table of static texts, and each distinct text appears in it
once: string literals, folded formats and the texts that print instructions write all
share it. Before `main` starts, the VM puts each one in a separate static block, and the
block is frozen. Text made from a literal points into that block with a room of 0, so
nothing is duplicated.

A write inside a frozen block stops the program. The compiler forbids a cell write
(ZK0429), and a character write or an add first moves text with a room of 0 to its own
block, so a pointer that unsafe code takes around the check is the sole route to a
frozen block:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် t သည် "abc" ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် p သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် t ၏ လိပ်စာ ကို လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် q သည် p ၏ တန်ဖိုး ဖြစ်ပါစေ။
q ၏ တန်ဖိုး ကို ၁၂၀ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: write at 0x1000 inside static data, which cannot change
```

[Instructions](../vm/instructions.md) lists the operations that build and drop text, and
[When a program stops](../vm/traps.md) lists the trap.
