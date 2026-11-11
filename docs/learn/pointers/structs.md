---
title: Lists and structs through addresses
description: Reach fields and cells through addresses, walk rows with pointers, put structs on the heap, and link them into a list.
---

# Lists and structs through addresses

Safe Zaka changes a caller's list or struct with a [`ဒေတာ ပြောင်းလဲရန်`
parameter](../functions/parameters.md#changing-the-callers-variable), and links values to
each other with [shared objects](../structs/objects.md). Addresses do the same work in the
open, as C does: a pointer to a field or a cell, and structs in heap blocks that the
program asks for.

## Fields through addresses

A field is a place, so it has an address too: a pointer to a value of the field's kind.

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု၊ အမြင့် ဟူသော ဒဿမကိန်း တစ်ခု တို့ပါဝင်သည်။

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

## A struct through its address

A struct name goes in front of `လိပ်စာ` to make a pointer kind. Through a pointer, a field
is reached with `တန်ဖိုး` first: `သူ ၏ တန်ဖိုး ၏ အသက်`. A function that takes the address
reads and writes the caller's struct itself:

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု၊ အတန်း ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

စိတ်မချရသော လုပ်ဆောင်ချက် အသက်ကြီး သည်
သူ ဟူသော လူ လိပ်စာ တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
သူ ၏ တန်ဖိုး ၏ အသက် ကို (သူ ၏ တန်ဖိုး ၏ အသက်) + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ(အသက်: ၂၀၊ အတန်း: ၇) ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် အသက်ကြီး ကို ကျော် ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ကျော် ၏ အသက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၁
```

`အသက်ကြီး` is marked, so its call needs marked code too. A field is not reached through
the pointer itself. `သူ ၏ အသက်`, with no `တန်ဖိုး`, reads a field of the address, and an
address has none:

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

The address of a counted list points at the whole list, so its kind holds the count:
`အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ`, or `isize[3]&` in messages. The rows of a table
of counted lists sit one after another, so adding 1 to the address of a row moves one
whole row:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ဇယား သည် အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (၁၊ ၂၊ ၃ တို့ပါသော စာရင်း)၊ (၄၊ ၅၊ ၆ တို့ပါသော စာရင်း) တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် အတန်း သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဇယား ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။
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

A plain list keeps its cells in a heap block of its own, so the step from one row to the
next works on counted rows alone. The address of one cell is a plain pointer, and [pointer
arithmetic](arithmetic.md) walks the cells after it, in a list of each form.

## The address of a cell pins the list

A duplicate of a list shares its cells until one side writes. A pointer could write a cell with
no warning to the duplicate, so taking the address of a cell first gives the list cells of its
own, and *pins* them: a duplicate made later copies the cells at once. A write through the
pointer then reaches this list alone:

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

The same holds for the cells of text. An add can still move the cells to a larger block,
so a cell address must not be used after the list grows. The compiler warns (ZK0704),
and the VM stops the read:

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

A heap block has no count in its kind. A [range](../forms/ranges.md) writes the count
once, and gives a list of its own, with copies of the cells:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၃ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ + ၂ ၏ တန်ဖိုး ကို ၉ ဟု ထားမည်။
ကိန်းရှင် ကိန်းများ သည် ပ ၏ အကွက် ၀ မှစ၍ ၃ ကွက် ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ကိန်းများ ကို ပြသမည်။                        // the list stands after the block is gone
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၀၊ ၀၊ ၉]
```

## Structs on the heap

A struct name goes in front of a count to ask the heap for room, as a kind does. The size
counts with the [count word](../structs/README.md#count-words) of the struct:

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု၊ အတန်း ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

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

Adding 1 to the address of a struct moves one whole struct, here 16 octets. A heap size
with another count word gives the warning of [The wrong
classifier](../structs/mistakes.md#the-wrong-classifier):

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

A struct cannot hold itself, but it can hold the address of its own kind, so a chain of
nodes forms a linked list. The `နောက်` ("next") field holds the null address at the last node, and
the program gives back each node itself:

```zaka
အဆစ် ဟူသော ဒေတာအစုအဝေးတွင် တန်ဖိုးရင်း ဟူသော ကိန်းပြည့် တစ်ခု၊ နောက် ဟူသော အဆစ် လိပ်စာ တစ်ခု တို့ပါဝင်သည်။

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

A duplicate of a struct that holds a pointer copies the address, not the block it points at,
so two structs can then reach one block. The same list built from a [shared
object](../structs/objects.md) needs no marked code and frees itself.

## Hands on

- Write a marked function that takes the address of a `လူ` and sets both of its fields,
  then compare it with a `ဒေတာ ပြောင်းလဲရန်` parameter.
- Ask the heap for room for two structs, set a field of each, and print both.
- Add a function to the linked list that sums the `တန်ဖိုးရင်း` of each node.

Next: [Owned pointers](ownership.md).
