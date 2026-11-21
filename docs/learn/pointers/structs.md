---
title: Lists and structs through addresses
description: Reach fields and cells through addresses, walk rows with pointers, put structs on the heap, and link them into a list.
---

# Lists and structs through addresses

Safe Zaka changes a caller's list or struct with a [mutable
parameter](../functions/parameters.md#changing-the-callers-variable) (`inout`), and links
values to each other with [shared objects](../structs/objects.md) (reference types).
Addresses (pointers) do the same work in the open, as C does, with a pointer to a field or
a cell, and with structs in heap blocks that the program asks for.

## You will learn

- how to take the address of a field or a cell, and how to reach a field through the
  address of a struct
- how a pointer steps through the rows of a counted list
- how to put structs on the heap
- how to build a linked list from structs that point at each other

A field is a place in RAM, so it has an address too, and the data kind of that address is
a pointer to the data kind of the field:

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် အမြင့် ဟူသော ဒဿမကိန်း တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကျော် ၏ အသက် ၏ လိပ်စာ ဖြစ်ပါစေ။
ပ ၏ တန်ဖိုး ကို ၂၅ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ကျော် ၏ အသက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၅
```

To take the address of a field, we write the field as before and add `၏ လိပ်စာ`:

```text
<variable> ၏ <field> ၏ လိပ်စာ
```

| Zaka | Reads as |
| --- | --- |
| `ကျော် ၏ အသက်` | "the age of `ကျော်`" |
| `၏ လိပ်စာ` | "the address of" it |

## A struct through its address

```text
<struct> လိပ်စာ
<pointer> ၏ တန်ဖိုး ၏ <field>
```

To write a pointer to a struct, we put `လိပ်စာ` after the struct name, the same as for
other data kinds. Through such a pointer, we reach a field with `တန်ဖိုး` first, as in
`သူ ၏ တန်ဖိုး ၏ အသက်` ("the age of the value at him"), and a function that accepts the
address can then read and write the caller's struct itself:

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် အတန်း ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

အသက်ကြီး ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည်
သူ ဟူသော လူ လိပ်စာ တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
သူ ၏ တန်ဖိုး ၏ အသက် ကို (သူ ၏ တန်ဖိုး ၏ အသက်) + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ(အသက်: ၂၀၊ အတန်း: ၇) ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
အသက်ကြီး ဟူသော လုပ်ဆောင်ချက်ကို ကျော် ၏ လိပ်စာ ဖြင့် လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ကျော် ၏ အသက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၁
```

`အသက်ကြီး` is an unsafe function, so its call needs unsafe code too. We cannot reach a
field through the pointer itself, because `သူ ၏ အသက်`, with no `တန်ဖိုး`, asks for a field
of the address, and an address has no fields:

```zaka-error ZK0426
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် p သည် လူ လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကျော် ၏ လိပ်စာ ဖြစ်ပါစေ။
p ၏ အသက် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0426]: "အသက်" is read as a field, but the value before "၏" is လူ*, not a struct
```

## Rows and cells through addresses

```text
အကွက် <count> ခုရှိသော <kind> စာရင်း လိပ်စာ
```

The address of a counted list (fixed-size array) points at the whole list, so its data
kind includes the count: `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ`, which messages
write as `isize[3]*`. The rows of a grid of counted lists sit one after another, so when
we add 1 to the address of a row, the pointer moves one whole row:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ဂရစ် သည် အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (၁၊ ၂ နှင့် ၃ တို့ပါသော စာရင်း) နှင့် (၄၊ ၅ နှင့် ၆ တို့ပါသော စာရင်း) တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် အတန်း သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဂရစ် ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။
အတန်း ၏ တန်ဖိုး ကို ပြသမည်။
အတန်း ကို အတန်း + ၁ ဟု ထားမည်။                 // one whole row further
အတန်း ၏ တန်ဖိုး ကို ပြသမည်။
အတန်း ၏ အကွက် ၂ ကို ပြသမည်။                     // a cell, through the row address
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၂၊ ၃]
[၄၊ ၅၊ ၆]
၆
```

A plain list keeps its cells in a separate heap block, so this step from one row to the
next works on counted rows alone. The address of one cell is a plain pointer, and in a
list of either form, [pointer arithmetic](arithmetic.md) walks the cells after it.

## The address of a cell pins the list

A duplicate (copy) of a list shares its cells with the original until one side writes. A
write through a pointer would give the duplicate no chance to notice, so when we take the
address of a cell, Zaka first gives the list its own cells and *pins* them, which means
that a duplicate made later duplicates the cells at once. A write through the pointer then
reaches this one list:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် [၂၊ ၃၊ ၅၊ ၇] ဖြစ်ပါစေ။
ကိန်းရှင် အရင် သည် ကိန်းများ ဖြစ်ပါစေ။               // a duplicate from before
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကိန်းများ ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။
ပ + ၃ ၏ တန်ဖိုး ကို ပြသမည်။                       // the cell 3 steps further
ကိန်းရှင် နောက် သည် ကိန်းများ ဖြစ်ပါစေ။               // a duplicate from after
ပ ၏ တန်ဖိုး ကို ၉ ဟု ထားမည်။
ကိန်းများ ကို ပြသမည်။
အရင် ကို ပြသမည်။
နောက် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၇
[၉၊ ၃၊ ၅၊ ၇]
[၂၊ ၃၊ ၅၊ ၇]
[၂၊ ၃၊ ၅၊ ၇]
```

The cells of text work the same. When we add to a list, Zaka can still move its cells to
a larger block, so we must not use a cell address after the list grows. The compiler warns
about it (ZK0704), and the VM stops the read:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် l သည် [၁၊ ၂] ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် l ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။
l ထဲသို့ ၃ ကို ထည့်မည်။
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: access at 0x101020 inside a freed block
```

## A range of a heap block

```text
<pointer> ၏ အကွက် <number> မှစ၍ <count> ကွက်
```

A heap block has no count in its data kind, so to turn some of its cells into a list, we
write the count once in a [range](../forms/ranges.md). The range gives a separate list
with duplicates of the cells, so the list remains after the block is gone:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၃ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ + ၂ ၏ တန်ဖိုး ကို ၉ ဟု ထားမည်။
ကိန်းရှင် ကိန်းများ သည် ပ ၏ အကွက် ၀ မှစ၍ ၃ ကွက် ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ကိန်းများ ကို ပြသမည်။                        // the list remains after the block is gone
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၀၊ ၀၊ ၉]
```

## Structs on the heap

To ask the heap for room for structs, we put the struct name in front of the count, the
same as for other data kinds, and we count with the [measure
word](../structs/README.md#measure-words) (classifier) of the struct:

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် အတန်း ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် p သည် လူ လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် လူ ၃ ယောက်စာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။
p ၏ တန်ဖိုး ၏ အသက် ကို ၁၀ ဟု ထားမည်။
(p + ၁) ၏ တန်ဖိုး ၏ အသက် ကို ၁၁ ဟု ထားမည်။
p ၏ တန်ဖိုး ကို ပြသမည်။
(p + ၁) ၏ တန်ဖိုး ကို ပြသမည်။
လူ တစ်ယောက် ၏ အရွယ်အစား ကို ပြသမည်။
လိပ်စာ p ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
လူ(အသက်: ၁၀၊ အတန်း: ၀)
လူ(အသက်: ၁၁၊ အတန်း: ၀)
၁၆
```

When we add 1 to the address of a struct, the pointer moves one whole struct, which here
is 16 octets. A heap size with another measure word gives the warning of [The wrong
measure word](../structs/mistakes.md#the-wrong-measure-word):

```zaka-warning ZK0509
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် p သည် လူ လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် လူ ၃ ခုစာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။
လိပ်စာ p ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0509]: လူ is counted with "ယောက်", not "ခု"
```

## A linked list

A struct cannot contain itself, but it can contain the address of a struct of the same
data kind, so a chain of nodes that point at each other forms a linked list. The `နောက်`
("next") field of the last node has the null address, and the program frees each node
itself:

```zaka
အဆစ် ဟူသော ဒေတာအစုအဝေးတွင် တန်ဖိုးရင်း ဟူသော ကိန်းပြည့် တစ်ခု နှင့် နောက် ဟူသော အဆစ် လိပ်စာ တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ထိပ် သည် အဆစ် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ ဖြစ်ပါစေ။
၁ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသစ် သည် အဆစ် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် အဆစ် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို အသစ် တွင် သိမ်းထားမည်။
အသစ် ၏ တန်ဖိုး ၏ တန်ဖိုးရင်း ကို န * န ဟု ထားမည်။
အသစ် ၏ တန်ဖိုး ၏ နောက် ကို ထိပ် ဟု ထားမည်။    // the old head goes after the new node
ထိပ် ကို အသစ် ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// walk the list with a second pointer
ကိန်းရှင် လက်ရှိ သည် ထိပ် ဖြစ်ပါစေ။
လက်ရှိ သည် မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ နှင့် မညီနေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လက်ရှိ ၏ တန်ဖိုး ၏ တန်ဖိုးရင်း ကို ပြသမည်။
လက်ရှိ ကို လက်ရှိ ၏ တန်ဖိုး ၏ နောက် ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// give back each node
ထိပ် သည် မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ နှင့် မညီနေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျန် သည် ထိပ် ၏ တန်ဖိုး ၏ နောက် ဖြစ်ပါစေ။
လိပ်စာ ထိပ် ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ထိပ် ကို ကျန် ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၉
၄
၁
```

Each new node goes in front of the old head, so the list prints the squares from the
last one to the first. The same list built from [shared
objects](../structs/objects.md#objects-that-point-at-each-other) needs no unsafe code and
frees itself.

## Details

- A duplicate of a struct that contains a pointer duplicates the address, not the block
  that it points at, so two structs can then reach one block ([Taking an
  address](../../manual/language/pointers.md#taking-an-address)).
- Keeping the address of a variable in a heap block, behind a pointer, or in a top-level
  variable from a function is the mistake ZK0703, because the variable can end before the
  block does ([Errors at a
  glance](../../manual/language/unsafe.md#errors-at-a-glance)).
- How fields are laid out in RAM, and so how far `+ ၁` moves on a struct, is in [How a
  struct is stored](../../manual/language/structs.md#how-a-struct-is-stored).

## Recap

- We write `<variable> ၏ <field> ၏ လိပ်စာ` or `<list> ၏ အကွက် <number> ၏ လိပ်စာ` to take
  the address of a field or a cell.
- Through the address of a struct, we reach a field with
  `<pointer> ၏ တန်ဖိုး ၏ <field>`.
- Taking the address of a cell pins the list, and a cell address must not be used after
  the list grows.
- We ask the heap for structs with the struct name and its measure word, and a struct
  can point at another struct of the same data kind, which gives a linked list.

## Hands on

- Write an unsafe function that accepts the address of a `လူ` and sets both of its fields,
  then compare it with a mutable parameter, `ဒေတာ ပြောင်းလဲရန်`.
- Ask the heap for room for two structs, set a field of each, and print both.
- Add a function to the linked list that sums the `တန်ဖိုးရင်း` of each node.

Next: [Owned pointers](ownership.md).
