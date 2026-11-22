---
title: Growing and shrinking
description: Change the size of a heap block while keeping its values, and fill, duplicate or move a run of cells in one statement.
---

# Growing and shrinking

A block from the heap can change size, as C's `realloc` does. When it grows, it keeps all
of its values, and when it shrinks, it keeps the values that still fit. This page also
shows three statements that write a whole run of cells at once, like C's `memset`,
`memcpy` and `memmove`.

## You will learn

- how to grow or shrink a heap block
- what a resize does to the address of the block
- how to fill a run of cells with one value
- how to duplicate or move a run of cells to another place

This program builds a list of numbers that grows as the numbers arrive. It starts with
room for 4, and doubles the room each time the list is full:

```zaka
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ဆံ့နိုင်ရေ သည် လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၄ ဖြစ်ပါစေ။ // room for
ကိန်းရှင် အရေအတွက် သည် လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။ // used so far
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ဆံ့နိုင်ရေ ခုစာ နေရာယူ၍ လိပ်စာကို ကိန်းများ တွင် သိမ်းထားမည်။

၁ မှ ၁၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ အရေအတွက် သည် ဆံ့နိုင်ရေ နှင့် ညီလျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဆံ့နိုင်ရေ ကို ဆံ့နိုင်ရေ * ၂ ဟု ထားမည်။
လိပ်စာ ကိန်းများ ရှိ မင်မိုရီကို ကိန်းပြည့် ဆံ့နိုင်ရေ ခုစာ ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ကိန်းများ + အရေအတွက် ၏ တန်ဖိုး ကို ကိန်း * ကိန်း ဟု ထားမည်။
အရေအတွက် ကို အရေအတွက် + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် နံပါတ် သည် လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ကာ နံပါတ် သည်
အရေအတွက် အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းများ + နံပါတ် ၏ တန်ဖိုး ကို ပြသမည်။
ပြီးလျှင် နံပါတ် ကို နံပါတ် + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လိပ်စာ ကိန်းများ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
၁
၄
၉
၁၆
၂၅
၃၆
၄၉
၆၄
၈၁
၁၀၀
```

The room grows from 4 to 8 to 16 while the ten squares arrive. To resize a block, we
name its pointer and give the new size in the same forms that we use to ask for a block:

```text
လိပ်စာ <pointer> ရှိ မင်မိုရီကို <size> ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။
```

| Zaka | Reads as |
| --- | --- |
| `လိပ်စာ ကိန်းများ ရှိ မင်မိုရီကို` | "the RAM at the address in numbers" |
| `ကိန်းပြည့် ဆံ့နိုင်ရေ ခုစာ ဖြစ်အောင်` | "so that it becomes room for *room* integers" |
| `နေရာအချဲ့အချုံ့လုပ်မည်။` | "will enlarge or shrink the room" |

The counters are `လက္ခဏာမဲ့ ကိန်းပြည့်`, because a size is never negative and the heap
statements need that data kind for a count.

## What resizing does

A resize can move the block to a new address. The pointer that we name in the statement
gets the new address, so we keep using that pointer, but a duplicate (copy) of the
address made before the resize would still point at the old place. The compiler warns
about a use of such a duplicate, and the VM stops the program at it (see [Pointer
mistakes](mistakes.md#using-a-block-after-giving-it-back)), so we take the address again
after each resize.

When the block grows, each value remains and the new room comes after the last one, and
when the block shrinks, the values that still fit remain.

## Writing and filling cells

```text
လိပ်စာ <pointer> ရှိ မင်မိုရီကို <value> ဖြင့် <size> ဖြည့်မည်။
လိပ်စာ <pointer> ရှိ <size> မင်မိုရီကို လိပ်စာ <pointer> ဆီသို့ ကူးယူမည်။
လိပ်စာ <pointer> ရှိ <size> မင်မိုရီကို လိပ်စာ <pointer> ဆီသို့ ပြောင်းထည့်မည်။
```

To write a whole run of cells, we can use one statement in place of a loop that writes one
cell at a time. Each of the three statements starts like a resize, with `လိပ်စာ ပ ရှိ`:

| Zaka | Reads as |
| --- | --- |
| `လိပ်စာ ပ ရှိ မင်မိုရီကို ၀ ဖြင့် ကိန်းပြည့် ၅ ခုစာ ဖြည့်မည်။` | "fill the RAM at *p* with 0, for 5 integers" |
| `လိပ်စာ ပ ရှိ ကိန်းပြည့် ၃ ခုစာ မင်မိုရီကို လိပ်စာ မ ဆီသို့ ကူးယူမည်။` | "duplicate 3 integers at *p* to the address *m*" |
| `လိပ်စာ ပ ရှိ ကိန်းပြည့် ၃ ခုစာ မင်မိုရီကို လိပ်စာ ပ + ၁ ဆီသို့ ပြောင်းထည့်မည်။` | "move 3 integers at *p* to the address *p + 1*" |

The last two differ in one thing: `ကူးယူမည်` needs a source and a destination that share
no cell, while `ပြောင်းထည့်မည်` lets them share cells, as in a shift of cells one place
on. This program fills a block with zeros, sets three cells, and shifts them on to make
room at the front:

```zaka
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၅ ခုစာ နေရာယူ၍ လိပ်စာကို ကိန်းများ တွင် သိမ်းထားမည်။

လိပ်စာ ကိန်းများ ရှိ မင်မိုရီကို ၀ ဖြင့် ကိန်းပြည့် ၅ ခုစာ ဖြည့်မည်။
ကိန်းများ ၏ တန်ဖိုး ကို ၁၀ ဟု ထားမည်။
ကိန်းများ + ၁ ၏ တန်ဖိုး ကို ၂၀ ဟု ထားမည်။
ကိန်းများ + ၂ ၏ တန်ဖိုး ကို ၃၀ ဟု ထားမည်။

// move the first 3 cells one place on
လိပ်စာ ကိန်းများ ရှိ ကိန်းပြည့် ၃ ခုစာ မင်မိုရီကို လိပ်စာ ကိန်းများ + ၁ ဆီသို့ ပြောင်းထည့်မည်။
ကိန်းများ ၏ တန်ဖိုး ကို ၅ ဟု ထားမည်။

၀ မှ ၄ အထိ န အနေဖြင့် ရေတွက်၍ ကိန်းများ + န ၏ တန်ဖိုး ကို ပြသမည်။
လိပ်စာ ကိန်းများ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
၅
၁၀
၂၀
၃၀
၀
```

## When the cells overlap

If the shift above used `ကူးယူမည်`, the source, cells 0 to 2, and the destination, cells 1
to 3, would share cells 1 and 2. The compiler can see that here, because both addresses
point into one block at known cells, so it warns before the program runs (ZK0721), and
in each case the VM stops the program before it writes a cell:

```zaka-stop
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ကိန်းများ တွင် သိမ်းထားမည်။
လိပ်စာ ကိန်းများ ရှိ မင်မိုရီကို ၀ ဖြင့် ကိန်းပြည့် ၄ ခုစာ ဖြည့်မည်။
လိပ်စာ ကိန်းများ ရှိ ကိန်းပြည့် ၃ ခုစာ မင်မိုရီကို လိပ်စာ ကိန်းများ + ၁ ဆီသို့ ကူးယူမည်။
လိပ်စာ ကိန်းများ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
the program stopped: the cells at 0x101008 overlap their source; "ပြောင်းထည့်မည်" allows that
```

When we are not sure whether two runs of cells overlap, we use `ပြောင်းထည့်မည်`.

## Details

- Resizing a pointer that has the null address is the same as asking for a new block
  ([Heap statements](../../manual/language/heap.md#heap-statements)).
- When the heap has no room for the new size, the program stops with `no heap room`. It
  stops instead of setting the pointer to the null address, because that would lose the
  block the pointer still has.
- The cells of these three statements must contain plain values: numbers, true/false
  values, characters, pointers, and structs of these. A run of text or of lists cannot be
  written with them, because two cells would then share one block that neither counts
  (ZK0442, [Writing and filling
  cells](../../manual/language/heap.md#writing-and-filling-cells)).

## Recap

- We use `လိပ်စာ <pointer> ရှိ မင်မိုရီကို <size> ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။` to
  resize a block, and the block keeps the values that fit.
- A resize can move the block, so after it we use the pointer named in the statement and
  no older duplicate of the address.
- `ဖြည့်မည်` fills a run of cells with one value, `ကူးယူမည်` duplicates a run to a place
  that does not overlap it, and `ပြောင်းထည့်မည်` moves a run to a place that can overlap.

## Hands on

- Start the list with room for 1 instead of 4. Does it still work?
- After the loop, shrink the block to fit `အရေအတွက်` integers and nothing more.
- Change the program to keep just the even squares.
- Fill a block of 8 octets with the size `၈ ဘိုက်စာ` and a value of
  `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်`.

Next: [Lists and structs through addresses](structs.md).
