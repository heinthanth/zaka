---
title: Pointer arithmetic
description: Step a pointer through a block of values, measure distances, and compare addresses.
---

# Pointer arithmetic

A block from the heap holds its values side by side, like a row of boxes. Adding a number
to a pointer moves it along the row: `စာရင်း + ၁` is the next box, `စာရင်း + ၂` the one
after. The boxes are numbered from 0, so the first box is `စာရင်း + ၀`, which is just
`စာရင်း`.

```zaka
ကိန်းရှင် စာရင်း သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။                 // list
မင်မိုရီ ကိန်းပြည့် ၃ ခုစာ နေရာယူ၍ လိပ်စာကို စာရင်း တွင် သိမ်းထားမည်။
စာရင်း ၏ တန်ဖိုး ကို ၁၀ ဟု ထားပါ။                                // box 0
စာရင်း + ၁ ၏ တန်ဖိုး ကို ၂၀ ဟု ထားပါ။                             // box 1
စာရင်း + ၂ ၏ တန်ဖိုး ကို ၃၀ ဟု ထားပါ။                             // box 2
၀ မှ ၂ အထိ နံပါတ် အနေဖြင့် ရေတွက်၍ စာရင်း + နံပါတ် ၏ တန်ဖိုး ကို ပြသပါ။
စာရင်း လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
၁၀
၂၀
၃၀
```

`+ ၁` moves by one *integer*, not by one byte. The pointer's kind says how big a box is,
and Zaka steps over that many bytes for you.

## Which value does `၏ တန်ဖိုး` read?

`၏ တန်ဖိုး` reads the value at everything written before it, so `စာရင်း + ၁ ၏ တန်ဖိုး`
is "the value at (list + 1)". The short form `*` binds to the name right after it, so
`*စာရင်း + ၁` is "(the value at list) + 1":

```zaka
ကိန်းရှင် စာရင်း သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို စာရင်း တွင် သိမ်းထားမည်။
စာရင်း ၏ တန်ဖိုး ကို ၁၀ ဟု ထားပါ။
စာရင်း + ၁ ၏ တန်ဖိုး ကို ၂၀ ဟု ထားပါ။
စာရင်း + ၁ ၏ တန်ဖိုး ကို ပြသပါ။      // the value at list + 1
*စာရင်း + ၁ ကို ပြသပါ။               // the value at list, plus 1
စာရင်း လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
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
first box to the end. The end is `စာရင်း + ၃`, the place just past the last box: the
loop compares against it but never reads it.

```zaka
ကိန်းရှင် စာရင်း သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၃ ခုစာ နေရာယူ၍ လိပ်စာကို စာရင်း တွင် သိမ်းထားမည်။
၀ မှ ၂ အထိ နံပါတ် အနေဖြင့် ရေတွက်၍ စာရင်း + နံပါတ် ၏ တန်ဖိုး ကို နံပါတ် * နံပါတ် ဟု ထားပါ။

ကိန်းရှင် ညွှန်း သည် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် စာရင်း ဖြစ်ကာ ညွှန်း သည် စာရင်း + ၃ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ညွှန်း ၏ တန်ဖိုး ကို ပြသပါ။
ပြီးလျှင် ညွှန်း ကို ညွှန်း + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

(စာရင်း + ၃) - စာရင်း ကို ပြသပါ။       // how many boxes apart
စာရင်း လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
၀
၁
၄
၃
```

Subtracting two pointers gives the number of boxes between them, as a `ကိန်းပြည့်`.

| Form | Gives |
| --- | --- |
| `ပ + ၂`, `ပ နှင့် ၂ ၏ ပေါင်းလဒ်` | a pointer two boxes on |
| `ပ - ၂`, `ပ နှင့် ၂ ၏ နုတ်လဒ်` | a pointer two boxes back |
| `ပ - မ` | the number of boxes from `မ` to `ပ` |
| `ပ သည် မ အောက် ငယ်` | whether `ပ` comes before `မ` |
| `ပ သည် ဗလာ နှင့် ညီ` | whether `ပ` is empty |

Nothing else works on pointers. Multiplying a pointer, for example, makes no sense:

```zaka-error ZK0415
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
ပ * ၂ ကို ပြသပါ။
```

```text
error[ZK0415]: the operator "*" cannot be used on pointers
```

## Looking at the same bytes another way

A [conversion](../operators/conversion.md) turns a pointer of one kind into another. This
program asks for 8 bytes, stores a 1-byte code in the first one, and a 32-bit price in
bytes 4 to 7:

```zaka
ကိန်းရှင် ပစ္စည်း သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။     // item: one byte each step
မင်မိုရီ ၈ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို ပစ္စည်း တွင် သိမ်းထားမည်။
ပစ္စည်း ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ။                                       // code, at byte 0
ကိန်းရှင် ဈေးနှုန်း သည် ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။              // price
ဈေးနှုန်း ကို ပစ္စည်း + ၄ ကို ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ အဖြစ် ပြောင်းလဲသော ရလဒ် ဟု ထားပါ။
ဈေးနှုန်း ၏ တန်ဖိုး ကို ၁၅၀၀ ဟု ထားပါ။                                   // price, at byte 4
ဈေးနှုန်း ၏ တန်ဖိုး ကို ပြသပါ။
ပစ္စည်း လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
၁၅၀၀
```

Laying out a record by counting bytes like this is easy to get wrong. A later version of
Zaka adds records that do the counting for you.

An opaque `လိပ်စာ` converts to and from any pointer kind on its own, but it has no box
size, so you cannot add to it or read through it until you convert it.

## Try it

- Fill a block of 10 integers with the numbers 1 to 10, then print their sum.
- Walk a pointer backwards, from the last box to the first.
- Read one box past the end of the block. [Memory mistakes](mistakes.md) explains what
  happens.

Next: [Growing and shrinking](resizing.md).
