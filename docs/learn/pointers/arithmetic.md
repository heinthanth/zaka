---
title: Pointer arithmetic
description: Step a pointer through a block of values, measure distances, and compare addresses.
---

# Pointer arithmetic

A block from the heap holds its values next to each other, like a row of boxes. Adding a
number to a pointer moves it along the row: `ကိန်းများ + ၁` is the next box, `ကိန်းများ + ၂` the one
after. The boxes are numbered from 0, so the first box is `ကိန်းများ + ၀`, which is just
`ကိန်းများ`.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။             // list
မင်မိုရီတွင် ကိန်းပြည့် ၃ ခုစာ နေရာယူ၍ လိပ်စာကို ကိန်းများ တွင် သိမ်းထားမည်။
ကိန်းများ ၏ တန်ဖိုး ကို ၁၀ ဟု ထားမည်။                                // box 0
ကိန်းများ + ၁ ၏ တန်ဖိုး ကို ၂၀ ဟု ထားမည်။                             // box 1
ကိန်းများ + ၂ ၏ တန်ဖိုး ကို ၃၀ ဟု ထားမည်။                             // box 2
၀ မှ ၂ အထိ နံပါတ် အနေဖြင့် ရေတွက်၍ ကိန်းများ + နံပါတ် ၏ တန်ဖိုး ကို ပြသမည်။
လိပ်စာ ကိန်းများ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၀
၂၀
၃၀
```

`+ ၁` moves one *integer* along, not one octet. The pointer's kind tells how big a box
is, and Zaka steps over that number of octets on its own.

## Which value does `၏ တန်ဖိုး` read?

`၏ တန်ဖိုး` reads the value at the whole expression written before it, so `ကိန်းများ + ၁ ၏ တန်ဖိုး`
is "the value at (list + 1)". The short form `*` binds to the name right after it, so
`*ကိန်းများ + ၁` is "(the value at list) + 1":

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ကိန်းများ တွင် သိမ်းထားမည်။
ကိန်းများ ၏ တန်ဖိုး ကို ၁၀ ဟု ထားမည်။
ကိန်းများ + ၁ ၏ တန်ဖိုး ကို ၂၀ ဟု ထားမည်။
ကိန်းများ + ၁ ၏ တန်ဖိုး ကို ပြသမည်။      // the value at list + 1
*ကိန်းများ + ၁ ကို ပြသမည်။               // the value at list, plus 1
လိပ်စာ ကိန်းများ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၀
၁၁
```

The same rule bites when a pointer comes second. `စုစုပေါင်း + ညွှန်း ၏ တန်ဖိုး` is "the value at
(total + pointer)", not "total + the value at the pointer". Put the read in parentheses,
`စုစုပေါင်း + (ညွှန်း ၏ တန်ဖိုး)`, or use the short form, `စုစုပေါင်း + *ညွှန်း`.

## Walking with a pointer

Pointers into the same block can be compared, so a loop can walk a pointer from the
first box to the end. The end is `ကိန်းများ + ၃`, the place just past the last box: the
loop compares against it but never reads it.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၃ ခုစာ နေရာယူ၍ လိပ်စာကို ကိန်းများ တွင် သိမ်းထားမည်။
၀ မှ ၂ အထိ နံပါတ် အနေဖြင့် ရေတွက်၍ ကိန်းများ + နံပါတ် ၏ တန်ဖိုး ကို နံပါတ် * နံပါတ် ဟု ထားမည်။

ကိန်းရှင် ညွှန်း သည် ကိန်းများ ဖြစ်ကာ ညွှန်း သည် ကိန်းများ + ၃ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ညွှန်း ၏ တန်ဖိုး ကို ပြသမည်။
ပြီးလျှင် ညွှန်း ကို ညွှန်း + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

(ကိန်းများ + ၃) - ကိန်းများ ကို ပြသမည်။       // the distance in boxes
လိပ်စာ ကိန်းများ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၀
၁
၄
၃
```

Subtracting two pointers gives the number of boxes between them, as a `ကိန်းပြည့်`.

`ညွှန်း` starts as a duplicate of the address in `ကိန်းများ`, and moves on while `ကိန်းများ` remains
at the start, to give the block back at the end.

| Form | Gives |
| --- | --- |
| `ပ + ၂`, `ပ နှင့် ၂ ကို ပေါင်းသည့်ရလဒ်` | a pointer two boxes on |
| `ပ - ၂`, `ပ နှင့် ၂ ကို နုတ်သည့်ရလဒ်` | a pointer two boxes back |
| `ပ - မ` | the number of boxes from `မ` to `ပ` |
| `ပ သည် မ အောက် ငယ်` | whether `ပ` comes before `မ` |
| `ပ သည် မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ နှင့် ညီ` | whether `ပ` is null |

Nothing else works on pointers. A pointer times a number, for example, makes no sense:

```zaka-error ZK0415
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
ပ * ၂ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0415]: the operator "*" cannot be used on pointers
```

## Reading the same octets as another kind

A [conversion](../operators/conversion.md) turns a pointer of one kind into another. This
program asks for 8 octets, stores a 1-octet code in the first one, and a 32-bit price in
octets 4 to 7:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပစ္စည်း သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။ // item: one octet each step
မင်မိုရီတွင် ၈ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို ပစ္စည်း တွင် သိမ်းထားမည်။
ပစ္စည်း ၏ တန်ဖိုး ကို ၇ ဟု ထားမည်။                                       // code, at octet 0
ကိန်းရှင် ဈေးနှုန်း သည် ပစ္စည်း + ၄ ကို ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။ // price
ဈေးနှုန်း ၏ တန်ဖိုး ကို ၁၅၀၀ ဟု ထားမည်။                                   // price, at octet 4
ဈေးနှုန်း ၏ တန်ဖိုး ကို ပြသမည်။
လိပ်စာ ပစ္စည်း ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၅၀၀
```

Building a record from hand-counted octets like this invites mistakes. A
[struct](../structs/README.md) does the counting on its own, and [Lists and structs
through addresses](structs.md) puts structs on the heap.

An opaque `လိပ်စာ` converts to and from each pointer kind on its own, as long as the boxes
of that kind hold no address. It has no box size, so a program cannot add to it or read
through it until it is converted.

## Hands on

- Fill a block of 10 integers with the numbers 1 to 10, then print their sum.
- Walk a pointer backwards, from the last box to the first.
- Read one box past the end of the block. [Pointer mistakes](mistakes.md) explains what
  happens.

Next: [Growing and shrinking](resizing.md).
