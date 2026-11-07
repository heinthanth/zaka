---
title: Lists and structs through addresses
description: Change a caller's list or struct through its address, walk rows and cells with pointers, put structs on the heap, and link them into a list.
---

# Lists and structs through addresses

Part I changed a caller's list or struct with a marked parameter, and kept each struct in
a variable. An address does the same work in the open: the function holds a pointer to
the caller's value, and a struct can sit in a heap block that the program asks for.

## Changing the caller's list

The parameter kind is the list kind with `လိပ်စာ` after it and `ငှားထားသော` in front,
since the function borrows the caller's list, and the call gives `မူရင်း ၏ လိပ်စာ`. The
cells are reached through it with `အကွက်` as before:

```zaka
// the parameter is the address of the caller's list
လုပ်ဆောင်ချက် နှစ်ဆတိုး သည်
ငှားထားသော ကိန်းပြည့် စာရင်း လိပ်စာ ကိန်းများ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၀ မှ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၁ အထိ န အနေဖြင့် ရေတွက်၍ ကိန်းများ ၏ အကွက် န ကို (ကိန်းများ ၏ အကွက် န) * ၂ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မူရင်း သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် နှစ်ဆတိုး ကို မူရင်း ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်ပါ။
မူရင်း ကို ပြသပါ။     // changed through the address
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၂၊ ၄၊ ၆]
```

To grow the caller's list, add to `ကိန်းများ ၏ တန်ဖိုး`, the list the address points at:

```zaka
// the parameter is a duplicate of the list passed in
လုပ်ဆောင်ချက် ချဲ့ သည်
ကိန်းပြည့် စာရင်း ကိန်းများ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းများ ထဲသို့ ၉ ကို ထည့်ပါ။
ကိန်းများ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// the parameter is the address of the caller's list
လုပ်ဆောင်ချက် ထပ်ထည့် သည်
ငှားထားသော ကိန်းပြည့် စာရင်း လိပ်စာ ကိန်းများ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းများ ၏ တန်ဖိုး ထဲသို့ ၉ ကို ထည့်ပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မူရင်း သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ချဲ့ ကို မူရင်း တို့ဖြင့် လုပ်ဆောင်ပါ။
မူရင်း ကို ပြသပါ။     // unchanged
လုပ်ဆောင်ချက် ထပ်ထည့် ကို မူရင်း ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်ပါ။
မူရင်း ကို ပြသပါ။     // changed through the address
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၂၊ ၉]
[၁၊ ၂]
[၁၊ ၂၊ ၉]
```

A counted list is passed in the same manner, through the address of
`အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း`; its count is part of the kind, so the function
takes lists of that count alone.

## Fields through addresses

A field is a place, so it has an address too: a pointer to a value of the field's kind,
which can be lent or written through:

```zaka
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ဒဿမကိန်း အမြင့် တို့ ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကျော် ၏ အသက် ၏ လိပ်စာ ဖြစ်ပါစေ။
ပ ၏ တန်ဖိုး ကို ၂၅ ဟု ထားပါ။
ကျော် ၏ အသက် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၅
```

## Changing the caller's struct

A struct name goes in front of `လိပ်စာ` to make a pointer kind. Through a pointer, a field
is reached with `တန်ဖိုး` first: `သူ ၏ တန်ဖိုး ၏ အသက်`. To let a function change the
caller's struct, pass a borrowed address: the parameter kind is `ငှားထားသော လူ လိပ်စာ`,
and the call gives `ကျော် ၏ လိပ်စာ`.

```zaka
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ကိန်းပြည့် အတန်း တို့ ပါဝင်သည်။
လုပ်ဆောင်ချက် ပွားကြည့် သည်
လူ သူ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
သူ ၏ အသက် ကို ၉၉ ဟု ထားပါ။
သူ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် အသက်ကြီး သည်
ငှားထားသော လူ လိပ်စာ သူ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
သူ ၏ တန်ဖိုး ၏ အသက် ကို (သူ ၏ တန်ဖိုး ၏ အသက်) + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်ပါစေ။
ကျော် ၏ အသက် ကို ၂၀ ဟု ထားပါ။
လုပ်ဆောင်ချက် ပွားကြည့် ကို ကျော် တို့ဖြင့် လုပ်ဆောင်ပါ။
ကျော် ၏ အသက် ကို ပြသပါ။
လုပ်ဆောင်ချက် အသက်ကြီး ကို ကျော် ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်ပါ။
ကျော် ၏ အသက် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
လူ(အသက်: ၉၉၊ အတန်း: ၀)
၂၀
၂၁
```

A field is not reached through the pointer itself. `သူ ၏ အသက်`, with no `တန်ဖိုး`, reads
a field of the address, and an address has none:

```zaka-error ZK0426
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက် တို့ ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် p သည် ငှားထားသော လူ လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကျော် ၏ လိပ်စာ ဖြစ်ပါစေ။
p ၏ အသက် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0426]: "အသက်" is read as a field, but the value before "၏" is လူ&, not a struct
```

## Rows and cells through addresses

The address of a counted list points at the whole list, so its kind holds the count:
`ငှားထားသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ`, or `isize[3]&` in messages. The rows
of a table of counted lists sit one after another, so adding 1 to the address of a row
moves one whole row:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဇယား သည် အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (၁၊ ၂၊ ၃ တို့ပါသော စာရင်း)၊ (၄၊ ၅၊ ၆ တို့ပါသော စာရင်း) တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် အတန်း သည် ငှားထားသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဇယား ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။
အတန်း ၏ တန်ဖိုး ကို ပြသပါ။
အတန်း ကို အတန်း + ၁ ဟု ထားပါ။                 // one whole row further
အတန်း ၏ တန်ဖိုး ကို ပြသပါ။
အတန်း ၏ အကွက် ၂ ကို ပြသပါ။                     // a cell, through the row address
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၂၊ ၃]
[၄၊ ၅၊ ၆]
၆
```

A plain list keeps each row in a heap block of its own, so the step from one row to the
next works on counted rows alone. The address of one cell is a plain pointer, and [pointer
arithmetic](arithmetic.md) walks the cells after it, in a list of each form:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၊ ၃၊ ၅၊ ၇ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကိန်းများ ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။ // the first cell
ပ ၏ တန်ဖိုး ကို ပြသပါ။
ပ + ၃ ၏ တန်ဖိုး ကို ပြသပါ။                 // the cell 3 steps further
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂
၇
```

A cell address borrows the list, as a [slice](../lists/slices.md) does. An add can move
the cells to a new block, so the address cannot be used after the list grows or shrinks:

```zaka-error ZK0704
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် l ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။
l ထဲသို့ ၃ ကို ထည့်ပါ။
ပ ၏ တန်ဖိုး ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0704]: the block of "l" is edited, so its cells can move, here, but "ပ" still borrows it and is used later
```

## Slices of the heap

A heap block has no count in its kind. A range writes the count once, and the
[slice](../lists/slices.md) keeps it from then on. `ပ ၏ တန်ဖိုးအငှား` lends a whole block
in the same manner:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၃ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ပ ၏ အကွက် ၀ မှစ၍ ၃ ကွက် ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက် ၂ ကို ၉ ဟု ထားပါ။
ကိန်းများ ကို ပြသပါ။
ကိန်းများ ၏ အကွက်အရေအတွက် ကို ပြသပါ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၀၊ ၀၊ ၉]
၃
```

A slice borrows, so its block must live until the last use of the slice:

```zaka-error ZK0704
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် s သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ပ ၏ အကွက် ၀ မှစ၍ ၂ ကွက် ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
s ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## Structs on the heap

A struct name goes in front of a count to ask the heap for room, as a kind does. The size
counts with the [classifier](../structs/README.md#count-words) of the struct:

```zaka
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ကိန်းပြည့် အတန်း တို့ ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် p သည် လူ လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် လူ ၃ ယောက်စာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။
p ၏ တန်ဖိုး ၏ အသက် ကို ၁၀ ဟု ထားပါ။
(p + ၁) ၏ တန်ဖိုး ၏ အသက် ကို ၁၁ ဟု ထားပါ။
p ၏ တန်ဖိုး ကို ပြသပါ။
(p + ၁) ၏ တန်ဖိုး ကို ပြသပါ။
လူ ၏ အရွယ်အစား ကို ပြသပါ။
လိပ်စာ p ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
လူ(အသက်: ၁၀၊ အတန်း: ၀)
လူ(အသက်: ၁၁၊ အတန်း: ၀)
၁၆
```

Adding 1 to the address of a struct moves one whole struct, here 16 octets. A list count
(`အကွက် ၃ ခုရှိသော`) and an ordinal (`၁ ခုမြောက်အကွက်`) count cells, not structs, so
these use `ခု` each time. A heap size with another word gives the warning of [The wrong
classifier](../structs/mistakes.md#the-wrong-classifier):

```zaka-warning ZK0509
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက် တို့ ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် p သည် လူ လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် လူ ၃ ခုစာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။
လိပ်စာ p ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0509]: လူ is counted with "ယောက်", not "ခု"
```

The same warning comes for `ယောက်` in the heap size of a kind such as `ကိန်းပြည့်`, once a
struct of the program has named `ယောက်` as its classifier:

```zaka-warning ZK0509
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက် တို့ ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် p သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၃ ယောက်စာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။
လိပ်စာ p ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0509]: isize is counted with "ခု", not "ယောက်"
```

## A linked list

A struct cannot hold itself, but it can hold the address of its own kind. Such an address
is an owner, so each node owns the next one, and a chain of nodes forms a linked list.
The `နောက်` ("next") field ends in `ဗလာ` at the last node.

```zaka
ဒေတာအမျိုးအစား အဆစ် တွင် ကိန်းပြည့် တန်ဖိုးရင်း၊ အဆစ် လိပ်စာ နောက် တို့ ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ထိပ် သည် အဆစ် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
၁ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် အသစ် သည် အဆစ် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် အဆစ် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို အသစ် တွင် သိမ်းထားမည်။
အသစ် ၏ တန်ဖိုး ၏ တန်ဖိုးရင်း ကို န * န ဟု ထားပါ။
အသစ် ၏ တန်ဖိုး ၏ နောက် ကို ထိပ် ဟု ထားပါ။    // the old head moves into the new node
ထိပ် ကို အသစ် ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// walk the list with a borrowed address
ကိန်းရှင် လက်ရှိ သည် ငှားထားသော အဆစ် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ထိပ် ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
လက်ရှိ သည် ဗလာ နှင့် မညီနေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
လက်ရှိ ၏ တန်ဖိုး ၏ တန်ဖိုးရင်း ကို ပြသပါ။
လက်ရှိ ကို လက်ရှိ ၏ တန်ဖိုး ၏ နောက် ၏ တန်ဖိုးအငှား ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// give back each node
ထိပ် သည် ဗလာ နှင့် မညီနေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် ကျန် သည် အဆစ် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ထိပ် ၏ တန်ဖိုး ၏ နောက် ဖြစ်ပါစေ။
လိပ်စာ ထိပ် ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ထိပ် ကို ကျန် ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၉
၄
၁
```

A struct that holds an owner moves instead of being duplicated, as an owner does: the new
struct takes the address, and the old one is left with `ဗလာ` in that field and cannot be
used again. The next page, [Ownership and borrowing](ownership.md#one-owner-each), gives
the rules of owners and borrows.

## Using a struct after it moved

A struct that holds an owner moves when it is set to another, so the old one cannot be
used after:

```zaka-error ZK0701
ဒေတာအမျိုးအစား အဆစ် တွင် ကိန်းပြည့် တန်ဖိုးရင်း၊ အဆစ် လိပ်စာ နောက် တို့ ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အဆစ် တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အဆစ် တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် အဆစ် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို က ၏ နောက် တွင် သိမ်းထားမည်။
ခ ကို က ဟု ထားပါ။
က ၏ နောက် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0701]: "က" was moved to another owner and now holds ဗလာ
```

## A borrow in a field

A borrow lasts as long as the owner it borrows from, and the checker follows that through
variables and parameters. A struct can sit in the heap, out of the checker's sight, so a
field holds an owner (`လူ လိပ်စာ`) or a raw address (`လူ လိပ်စာအကြမ်း`), never a borrow:

```zaka-error ZK0718
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ငှားထားသော ကိန်းပြည့် လိပ်စာ ရှင် တို့ ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0718]: the field "ရှင်" holds a borrow, and borrows live in variables and parameters alone
```

## Hands on

- Write a function that takes the address of a list of integers and adds 1 to each cell,
  then compare it with the marked parameter of [Passing and comparing
  lists](../lists/passing.md).
- Write a function that takes the address of a list of text and adds a name to it.
- Ask the heap for room for two structs, set a field of each, and print both.
- Add a function to the linked list that sums the `တန်ဖိုးရင်း` of each node.

Next: [Functions as values](functions.md).
