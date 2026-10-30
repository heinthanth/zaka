---
title: Growing and shrinking
description: Change the size of a heap block while keeping what it holds.
---

# Growing and shrinking

A block from the heap can change size. Growing keeps all that is in it so far; shrinking
keeps what still fits. This program builds a list that grows as numbers arrive. It starts
with room for 4 and doubles the room each time the list is full:

```zaka
ကိန်းရှင် ဆံ့နိုင်ရေ သည် လက္ခဏာမဲ့ ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၄ ဖြစ်ပါစေ။      // room for
ကိန်းရှင် အရေအတွက် သည် လက္ခဏာမဲ့ ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။      // used so far
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ဆံ့နိုင်ရေ ခုစာ နေရာယူ၍ လိပ်စာကို ကိန်းများ တွင် သိမ်းထားမည်။

၁ မှ ၁၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
အကယ်၍ အရေအတွက် သည် ဆံ့နိုင်ရေ နှင့် ညီလျှင် အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ဆံ့နိုင်ရေ ကို ဆံ့နိုင်ရေ * ၂ ဟု ထားပါ။
လိပ်စာ ကိန်းများ ရှိ မင်မိုရီကို ကိန်းပြည့် ဆံ့နိုင်ရေ ခုစာ ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
ကိန်းများ + အရေအတွက် ၏ တန်ဖိုး ကို ကိန်း * ကိန်း ဟု ထားပါ။
အရေအတွက် ကို အရေအတွက် + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် နံပါတ် သည် လက္ခဏာမဲ့ ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ကာ နံပါတ် သည် အရေအတွက် အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းများ + နံပါတ် ၏ တန်ဖိုး ကို ပြသပါ။
ပြီးလျှင် နံပါတ် ကို နံပါတ် + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

လိပ်စာ ကိန်းများ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
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

- It can move the block to a new address. The owner gets the new address, so keep using
  that pointer. A borrow made before the resize would point at the old spot, so the
  compiler refuses a resize while a borrow of the block is still in use (see [Pointer
  mistakes](mistakes.md#using-a-block-after-giving-it-back)). Lend the block again after
  the resize.
- Growing keeps each value and adds new room after the last one. Shrinking keeps the
  values that still fit.
- Resizing a null pointer (`ဗလာ`) is the same as asking for a new block.
- If the heap has no room for the new size, the program stops with "no heap room". It
  stops rather than set the pointer to null, because that would lose the block the
  pointer still holds.

## Hands on

- Start the list with room for 1 instead of 4. Does it still work?
- After the loop, shrink the block to fit `အရေအတွက်` integers and no more.
- Change the program to keep just the even squares.

Next: [Ownership and borrowing](ownership.md).
