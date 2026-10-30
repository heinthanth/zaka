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
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။                 // list
မင်မိုရီတွင် ကိန်းပြည့် ၃ ခုစာ နေရာယူ၍ လိပ်စာကို ကိန်းများ တွင် သိမ်းထားမည်။
ကိန်းများ ၏ တန်ဖိုး ကို ၁၀ ဟု ထားပါ။                                // box 0
ကိန်းများ + ၁ ၏ တန်ဖိုး ကို ၂၀ ဟု ထားပါ။                             // box 1
ကိန်းများ + ၂ ၏ တန်ဖိုး ကို ၃၀ ဟု ထားပါ။                             // box 2
၀ မှ ၂ အထိ နံပါတ် အနေဖြင့် ရေတွက်၍ ကိန်းများ + နံပါတ် ၏ တန်ဖိုး ကို ပြသပါ။
လိပ်စာ ကိန်းများ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
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
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ကိန်းများ တွင် သိမ်းထားမည်။
ကိန်းများ ၏ တန်ဖိုး ကို ၁၀ ဟု ထားပါ။
ကိန်းများ + ၁ ၏ တန်ဖိုး ကို ၂၀ ဟု ထားပါ။
ကိန်းများ + ၁ ၏ တန်ဖိုး ကို ပြသပါ။      // the value at list + 1
*ကိန်းများ + ၁ ကို ပြသပါ။               // the value at list, plus 1
လိပ်စာ ကိန်းများ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
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
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၃ ခုစာ နေရာယူ၍ လိပ်စာကို ကိန်းများ တွင် သိမ်းထားမည်။
၀ မှ ၂ အထိ နံပါတ် အနေဖြင့် ရေတွက်၍ ကိန်းများ + နံပါတ် ၏ တန်ဖိုး ကို နံပါတ် * နံပါတ် ဟု ထားပါ။

ကိန်းရှင် ညွှန်း သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ကိန်းများ ၏ တန်ဖိုးအငှား ဖြစ်ကာ ညွှန်း သည် ကိန်းများ + ၃ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ညွှန်း ၏ တန်ဖိုး ကို ပြသပါ။
ပြီးလျှင် ညွှန်း ကို ညွှန်း + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

(ကိန်းများ + ၃) - ကိန်းများ ကို ပြသပါ။       // the distance in boxes
လိပ်စာ ကိန်းများ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
၀
၁
၄
၃
```

Subtracting two pointers gives the number of boxes between them, as a `ကိန်းပြည့်`.

`ကိန်းများ ၏ တန်ဖိုးအငှား` lends the block: `ညွှန်း` borrows it and `ကိန်းများ` keeps it.
An address made with `+` or `-` is a borrow as well, so it never needs the word. [Ownership
and borrowing](ownership.md) has the rules.

| Form | Gives |
| --- | --- |
| `ပ ၏ တန်ဖိုးအငှား` | a borrow of the block `ပ` owns; `ပ` keeps it |
| `ပ + ၂`, `ပ နှင့် ၂ ၏ ပေါင်းလဒ်` | a pointer two boxes on |
| `ပ - ၂`, `ပ နှင့် ၂ ၏ နုတ်လဒ်` | a pointer two boxes back |
| `ပ - မ` | the number of boxes from `မ` to `ပ` |
| `ပ သည် မ အောက် ငယ်` | whether `ပ` comes before `မ` |
| `ပ သည် ဗလာ နှင့် ညီ` | whether `ပ` is null |

Nothing else works on pointers. A pointer times a number, for example, makes no sense:

```zaka-error ZK0415
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
ပ * ၂ ကို ပြသပါ။
```

```text
error[ZK0415]: the operator "*" cannot be used on pointers
```

## Reading the same octets as another kind

A [conversion](../operators/conversion.md) turns a pointer of one kind into another. This
program asks for 8 octets, stores a 1-octet code in the first one, and a 32-bit price in
octets 4 to 7:

```zaka
ကိန်းရှင် ပစ္စည်း သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။     // item: one octet each step
မင်မိုရီတွင် ၈ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို ပစ္စည်း တွင် သိမ်းထားမည်။
ပစ္စည်း ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ။                                       // code, at octet 0
ကိန်းရှင် ဈေးနှုန်း သည် ငှားထားသော ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။              // price
ဈေးနှုန်း ကို ပစ္စည်း + ၄ ကို ငှားထားသော ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ အဖြစ် ပြောင်းလဲသော ရလဒ် ဟု ထားပါ။
ဈေးနှုန်း ၏ တန်ဖိုး ကို ၁၅၀၀ ဟု ထားပါ။                                   // price, at octet 4
ဈေးနှုန်း ၏ တန်ဖိုး ကို ပြသပါ။
လိပ်စာ ပစ္စည်း ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
၁၅၀၀
```

Building a record from hand-counted octets like this invites mistakes. A later version
of Zaka adds records, which do the counting on their own.

An opaque `လိပ်စာ` converts to and from each pointer kind on its own, as long as the boxes
of that kind hold no address. It has no box size, so a program cannot add to it or read
through it until it is converted. A pointer to a pointer, such as
`ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ`, does not pass through `လိပ်စာ`, on its own or with one
conversion: that would read stored addresses as other data, which the checker cannot
follow. It goes through a raw address, `<kind> လိပ်စာအကြမ်း`, in two conversions.

## Hands on

- Fill a block of 10 integers with the numbers 1 to 10, then print their sum.
- Walk a pointer backwards, from the last box to the first.
- Read one box past the end of the block. [Pointer mistakes](mistakes.md) explains what
  happens.

Next: [Growing and shrinking](resizing.md).
