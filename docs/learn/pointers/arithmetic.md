---
title: Pointer arithmetic
description: Step a pointer through the cells of a block, measure the distance between two pointers, and compare addresses.
---

# Pointer arithmetic

A block from the heap keeps its values next to each other, like the cells (elements) of a
list, and adding a number to a pointer moves it along those cells. This is *pointer
arithmetic*, and it is how unsafe code reaches each value of a block.

## You will learn

- how to reach a cell of a block with `<pointer> + <number>`
- how `၏ တန်ဖိုး` and `*` group with the operators around them
- how to walk a pointer through a block and measure a distance
- how to read the same octets as another data kind

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။             // numbers
မင်မိုရီတွင် ကိန်းပြည့် ၃ ခုစာ နေရာယူ၍ လိပ်စာကို ကိန်းများ တွင် သိမ်းထားမည်။
ကိန်းများ ၏ တန်ဖိုး ကို ၁၀ ဟု ထားမည်။                                // cell 0
ကိန်းများ + ၁ ၏ တန်ဖိုး ကို ၂၀ ဟု ထားမည်။                             // cell 1
ကိန်းများ + ၂ ၏ တန်ဖိုး ကို ၃၀ ဟု ထားမည်။                             // cell 2
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

To reach a cell, we add its number to the pointer and read or write the value there:

```text
<pointer> + <number> ၏ တန်ဖိုး
```

| Zaka | Reads as |
| --- | --- |
| `ကိန်းများ + ၁` | "numbers plus 1": the address of the next cell |
| `၏ တန်ဖိုး` | "the value at" that address |

The cells are numbered from 0, so the first cell is `ကိန်းများ + ၀`, which is the same as
`ကိန်းများ`. Adding 1 moves one *integer* along, not one octet, because the data kind of
the pointer tells Zaka how large a cell is, and Zaka steps over that number of octets on
its own.

## Which value is read

`၏ တန်ဖိုး` reads the value at the whole expression before it, so
`ကိန်းများ + ၁ ၏ တန်ဖိုး` means "the value at (numbers + 1)". The short form `*` goes with
the name right after it, so `*ကိန်းများ + ၁` means "(the value at numbers) + 1":

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ကိန်းများ တွင် သိမ်းထားမည်။
ကိန်းများ ၏ တန်ဖိုး ကို ၁၀ ဟု ထားမည်။
ကိန်းများ + ၁ ၏ တန်ဖိုး ကို ၂၀ ဟု ထားမည်။
ကိန်းများ + ၁ ၏ တန်ဖိုး ကို ပြသမည်။      // the value at numbers + 1
*ကိန်းများ + ၁ ကို ပြသမည်။               // the value at numbers, plus 1
လိပ်စာ ကိန်းများ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၀
၁၁
```

The same rule matters when the pointer comes second. `စုစုပေါင်း + ညွှန်း ၏ တန်ဖိုး` means
"the value at (total + pointer)", not "total + the value at the pointer", so we put the
read in parentheses, `စုစုပေါင်း + (ညွှန်း ၏ တန်ဖိုး)`, or use the short form,
`စုစုပေါင်း + *ညွှန်း`.

## Walking with a pointer

```text
<pointer> - <pointer>
<pointer> သည် <pointer> အောက် ငယ်
```

We can compare two pointers into the same block, so a loop can walk a pointer from the
first cell to the end of the block. The end is `ကိန်းများ + ၃`, the place just past the
last cell, which the loop compares against but never reads:

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

(ကိန်းများ + ၃) - ကိန်းများ ကို ပြသမည်။       // the distance in cells
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

`ညွှန်း` starts as a duplicate (copy) of the address in `ကိန်းများ` and moves on, while
`ကိန်းများ` keeps the start of the block so that we can give the block back at the end.
When we subtract one pointer from another, we get the number of cells between them as a
`ကိန်းပြည့်`. These are all the forms that work on pointers:

| Form | Gives |
| --- | --- |
| `ပ + ၂`, `ပ နှင့် ၂ ၏ ပေါင်းလဒ်` | a pointer two cells on |
| `ပ - ၂`, `ပ နှင့် ၂ ၏ နုတ်လဒ်` | a pointer two cells back |
| `ပ - မ` | the number of cells from `မ` to `ပ` |
| `ပ သည် မ အောက် ငယ်` | whether `ပ` comes before `မ` |
| `ပ သည် မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ နှင့် ညီ` | whether `ပ` is null |

Other operators do not work on pointers, because a pointer times a number, for example,
means nothing:

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

## Reading the same octets as another data kind

```text
<pointer> ကို <kind> လိပ်စာ တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်
```

We can use a [conversion](../operators/conversion.md) to turn a pointer of one data kind
into a pointer of another. This program asks for 8 octets, stores a one-octet code in the
first one, and stores a 32-bit price in octets 4 to 7:

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

When we build a record from octets that we count out ourselves like this, it is simple to
make a mistake, so most programs use a [struct](../structs/README.md) instead, which does
the counting on its own, and [Lists and structs through addresses](structs.md) shows how to put
structs on the heap.

## Details

- An opaque `လိပ်စာ` converts to and from each pointer kind on its own, as long as the
  cells of that data kind contain no address. It has no cell size, so we cannot add to it
  or read through it until we convert it (ZK0414, [Conversions](../../manual/language/pointers.md#conversions)).
- Comparing or subtracting pointers of different data kinds is the error ZK0416, and
  subtracting pointers into different blocks is the mistake ZK0712 ([Pointer
  arithmetic](../../manual/language/pointers.md#pointer-arithmetic)).
- A pointer can point one cell past the end of its block, but a read or write there stops
  the program, as [Pointer mistakes](mistakes.md#reading-past-the-end) shows.

## Recap

- We write `<pointer> + <number>` to reach a later cell of a block, and Zaka steps a
  whole value of the pointer's data kind for each 1.
- `၏ တန်ဖိုး` reads at the whole expression before it, while `*` reads at the name right
  after it.
- We can compare two pointers into one block, and subtract them to get the number of
  cells between them.
- A conversion turns a pointer into a pointer of another data kind, so the same octets can
  be read in another form.

## Hands on

- Fill a block of 10 integers with the numbers 1 to 10, then print their sum.
- Walk a pointer backwards, from the last cell to the first.
- Read one cell past the end of the block. [Pointer mistakes](mistakes.md) explains what
  happens.

Next: [Growing and shrinking](resizing.md).
