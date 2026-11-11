---
title: Growing and shrinking
description: Change the size of a heap block while keeping what it holds.
---

# Growing and shrinking

A block from the heap can change size. Growing keeps all that is in it so far; shrinking
keeps what still fits. This program builds a list that grows as numbers arrive. It starts
with room for 4 and doubles the room each time the list is full:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ဆံ့နိုင်ရေ သည် လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၄ ဖြစ်ပါစေ။  // room for
ကိန်းရှင် အရေအတွက် သည် လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။  // used so far
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

ကိန်းရှင် နံပါတ် သည် လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ကာ နံပါတ် သည် အရေအတွက် အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းများ + နံပါတ် ၏ တန်ဖိုး ကို ပြသမည်။
ပြီးလျှင် နံပါတ် ကို နံပါတ် + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လိပ်စာ ကိန်းများ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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

The room grows from 4 to 8 to 16 while the ten squares arrive. The counters are
`လက္ခဏာမဲ့ ကိန်းပြည့်` because sizes are never negative and the heap statements take that
kind.

| Zaka | Reads as |
| --- | --- |
| `လိပ်စာ ကိန်းများ ရှိ မင်မိုရီကို` | "the block at the address in list" |
| `ကိန်းပြည့် ဆံ့နိုင်ရေ ခုစာ ဖြစ်အောင်` | "so that it becomes big enough for *room* integers" |
| `နေရာအချဲ့အချုံ့လုပ်မည်။` | "will enlarge or shrink the room" |

## What resizing does

- It can move the block to a new address. The variable named in the statement gets the
  new address, so keep using that pointer. A duplicate of the address made before the resize
  would point at the old spot: the compiler warns about a use of it, and the VM stops the
  program (see [Pointer mistakes](mistakes.md#using-a-block-after-giving-it-back)). Take
  the address again after the resize.
- Growing keeps each value and adds new room after the last one. Shrinking keeps the
  values that still fit.
- Resizing a null pointer is the same as asking for a new block.
- If the heap has no room for the new size, the program stops with "no heap room". It
  stops rather than set the pointer to null, because that would lose the block the
  pointer still holds.

## Writing and filling cells

Growing a block with a loop would write one cell at a time. Three statements write a whole
run of cells at once. Each starts like a resize, with `လိပ်စာ ပ ရှိ`:

| Zaka | Reads as |
| --- | --- |
| `လိပ်စာ ပ ရှိ မင်မိုရီကို ၀ ဖြင့် ကိန်းပြည့် ၅ ခုစာ ဖြည့်မည်။` | "fill the block at *p* with 0, for 5 integers" |
| `လိပ်စာ ပ ရှိ ကိန်းပြည့် ၃ ခုစာ မင်မိုရီကို လိပ်စာ မ ဆီသို့ ကူးယူမည်။` | "write 3 integers at *p* to the address *m*" |
| `လိပ်စာ ပ ရှိ ကိန်းပြည့် ၃ ခုစာ မင်မိုရီကို လိပ်စာ ပ + ၁ ဆီသို့ ပြောင်းထည့်မည်။` | "move 3 integers at *p* to the address *p + 1*" |

The last two differ in one thing: `ကူးယူမည်` needs a source and a destination that share no
cell, while `ပြောင်းထည့်မည်` allows them to share, as in a shift of cells one place on. This
program fills a block with zeros, sets three cells, and shifts them to make room in
front:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
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
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၅
၁၀
၂၀
၃၀
၀
```

Had the shift used `ကူးယူမည်`, the source `၀ မှ ၂` and the destination `၁ မှ ၃` would share
the cells 1 and 2. The compiler sees that here, since both addresses point into one block
at known cells, and warns before the program runs (ZK0721). In each case, the VM stops the
program before it writes a cell:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ကိန်းများ တွင် သိမ်းထားမည်။
လိပ်စာ ကိန်းများ ရှိ မင်မိုရီကို ၀ ဖြင့် ကိန်းပြည့် ၄ ခုစာ ဖြည့်မည်။
လိပ်စာ ကိန်းများ ရှိ ကိန်းပြည့် ၃ ခုစာ မင်မိုရီကို လိပ်စာ ကိန်းများ + ၁ ဆီသို့ ကူးယူမည်။
လိပ်စာ ကိန်းများ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: the cells at 0x101008 overlap their source; "ပြောင်းထည့်မည်" allows that
```

The cells must hold plain values: numbers, true/false, characters, pointers, and structs
of these. A block of text or of lists cannot be written with these statements, since two
cells would then share one block that neither counts (ZK0442).

## Hands on

- Start the list with room for 1 instead of 4. Does it still work?
- After the loop, shrink the block to fit `အရေအတွက်` integers and no more.
- Change the program to keep just the even squares.
- Fill a block of 8 octets with the size `၈ ဘိုက်စာ` and a value of `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်`.

Next: [Lists and structs through addresses](structs.md).
