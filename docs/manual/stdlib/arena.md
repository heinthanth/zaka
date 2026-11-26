---
title: The arena module
description: An arena on the heap - one block that hands out room piece after piece and frees the whole batch at once.
module: စံကား/နေရာကွက်
---

# The arena module

The arena module, `"စံကား/နေရာကွက်"`, gives an arena (an arena allocator): a
`မင်မိုရီနေရာကွက်` takes one block of the [heap](../language/heap.md) and hands out room
from its start, one piece after another. A program that builds a large set of values with one
lifetime, such as the nodes of a tree for one request, takes each node from the arena and
frees them all in one step, with no free for each node.

Each function of the module is [unsafe](../language/unsafe.md), as the heap statements
are, so we call it inside `စိတ်မချရသော်လည်း` or from an unsafe function, and the arena
goes to each call as a [mutable
parameter](../language/functions-parameters.md#mutable-parameters). A
[cleanup](../language/cleanup.md) gives the block back when the block of code ends:

```zaka
"စံကား/နေရာကွက်" မှ နေရာကွက်ပြုလုပ်၊ နေရာထုတ်၊ အစမှပြန်စ နှင့် စွန့်လွှတ် တို့ကို ထည့်သွင်းမည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ဧရိယာ သည် ၆၄ ဘိုက်စာ ဖြင့် နေရာကွက်ပြုလုပ် သည့် ရလဒ် ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများပြီးသည့် အခါတွင် ဧရိယာ ကို စွန့်လွှတ် မည်။
ကိန်းရှင် ပ သည် ဧရိယာ မှ ကိန်းပြည့် ၂ ခုစာ နေရာထုတ် သည့် ရလဒ်
ကို ကိန်းပြည့် လိပ်စာ တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားမည်။
(ပ + ၁) ၏ တန်ဖိုး ကို ၈ ဟု ထားမည်။
(ပ ၏ တန်ဖိုး) + ((ပ + ၁) ၏ တန်ဖိုး) ကို ပြသမည်။
ဧရိယာ ၏ သုံးပြီးသောပမာဏ ကို ပြသမည်။
ဧရိယာ ကို အစမှပြန်စ မည်။
ဧရိယာ ၏ သုံးပြီးသောပမာဏ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
၁၅
၁၆
၀
```

| Function | Does |
| --- | --- |
| `<size> ဖြင့် နေရာကွက်ပြုလုပ်` | takes a block of that size from the heap and gives an arena over it |
| `<arena> မှ <size> နေရာထုတ်` | gives the address of a piece of that size, as a `လိပ်စာ`, and moves past it |
| `<arena> ကို အစမှပြန်စ` | makes the whole block free again: the next piece starts at the start |
| `<arena> ကို စွန့်လွှတ်` | gives the block back to the heap; a second call does nothing |

A size is a [size form](../language/heap.md#heap-statements) such as `ကိန်းပြည့် ၃ ခုစာ` or
`၁၀၂၄ ဘိုက်စာ`, or a `လက္ခဏာမဲ့ ကိန်းပြည့်` count of octets.

## Pieces

`နေရာထုတ်` gives a plain `လိပ်စာ`, and we [convert](../language/pointers.md#conversions)
it to the pointer kind of the values the piece holds. Each piece starts at a multiple of 16
octets from the start of the block, as `malloc` of C gives, so a value of each data kind
can sit there, and a gap fills the room up to it. The struct shows the state of the arena:

| Field | Data kind | Holds |
| --- | --- | --- |
| `အစလိပ်စာ` | `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ` | the start of the block |
| `ပမာဏ` | `လက္ခဏာမဲ့ ကိန်းပြည့်` | the size of the block, in octets |
| `သုံးပြီးသောပမာဏ` | `လက္ခဏာမဲ့ ကိန်းပြည့်` | the octets handed out, gaps included |

When the room left is too small, `နေရာထုတ်` gives the null address, as a heap statement
does with no room, and the arena does not move, so a careful program compares the address
with `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`. An arena whose block the heap could not give has a
size of zero, and each piece it is asked for is the null address.

## What the VM checks

The arena owns its block, and each piece lies inside it, so the [VM](../vm/README.md)
checks a piece as part of that block: a read or a write past the end of the block stops the
program. A write past the end of one piece but inside the block reaches the next piece, as
it would in C, and so does a read through an address handed out before `အစမှပြန်စ`. After
`စွန့်လွှတ်`, each address of the arena is dangling, and the VM stops a use of one.

```zaka-stop
"စံကား/နေရာကွက်" မှ နေရာကွက်ပြုလုပ်၊ နေရာထုတ် နှင့် စွန့်လွှတ် တို့ကို ထည့်သွင်းမည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ဧရိယာ သည် ကိန်းပြည့် ၂ ခုစာ ဖြင့် နေရာကွက်ပြုလုပ် သည့် ရလဒ် ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများပြီးသည့် အခါတွင် ဧရိယာ ကို စွန့်လွှတ် မည်။
ကိန်းရှင် ပ သည် ဧရိယာ မှ ကိန်းပြည့် ၂ ခုစာ နေရာထုတ် သည့် ရလဒ်
ကို ကိန်းပြည့် လိပ်စာ တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
(ပ + ၁) ၏ တန်ဖိုး ကို ၇ ဟု ထားမည်။
"ဤနေရာ ရောက်သည်" ဟု ပြသမည်။
(ပ + ၂) ၏ တန်ဖိုး ကို ၈ ဟု ထားမည်။
"ဤနေရာ မရောက်ပါ" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
ဤနေရာ ရောက်သည်
the program stopped: access of 8 octets at 0x101090 lands outside each live block
```
