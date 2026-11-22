---
title: The heap and owned pointers
description: The heap statements that ask for, resize and free a block, the statements that write and fill cells, and owned pointers on parameters and results.
---

# The heap and owned pointers

The heap is the part of the RAM (memory) where a program can ask for a block (allocation)
while it runs. Safe Zaka uses the heap too, but there the VM counts each block and frees it
on its own. This page covers the heap statements of unsafe code, which ask for a block,
resize it and free it, the statements that write and fill a run of cells, and owned
pointers (owning pointers), which tell the compiler which function must free a block.
The pointers themselves are in [Pointers and addresses](pointers.md), and the rules of
unsafe code are in [Unsafe Zaka](unsafe.md). The Learn book teaches these in [The
heap](../../learn/pointers/heap.md), [Growing and
shrinking](../../learn/pointers/resizing.md) and [Owned
pointers](../../learn/pointers/ownership.md).

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၁၀ ဟု ထားမည်။
ပ + ၁ ၏ တန်ဖိုး ကို ၂၀ ဟု ထားမည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ကိန်းပြည့် ၃ ခုစာ ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။
ပ + ၂ ၏ တန်ဖိုး ကို ၃၀ ဟု ထားမည်။
၀ မှ ၂ အထိ န အနေဖြင့် ရေတွက်၍ ပ + န ၏ တန်ဖိုး ကို ပြသမည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ပ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၀
၂၀
၃၀
မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ
```

This program asks for a block of two integers, grows it to three, frees it twice (the
second time does nothing) and prints the pointer, which is now the null address.

## Heap statements

```text
မင်မိုရီတွင် <size> နေရာယူ၍ လိပ်စာကို <place> တွင် သိမ်းထားမည်။
လိပ်စာ <place> ရှိ မင်မိုရီကို <size> ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။
လိပ်စာ <place> ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

We use these three statements to ask for a block, to resize it and to free it, and
[three more](#writing-and-filling-cells) write the cells of a block. A block from these
statements has no count, so the VM never frees it on its own, and the program must free
it. In each statement we can write `မှတ်ဉာဏ်` in place of `မင်မိုရီ`, because both words
mean RAM. Resizing and freeing begin alike, with `လိပ်စာ <place> ရှိ မင်မိုရီကို` ("the RAM
at the address in the place"), and the words after that tell the two apart.

Each heap statement is a [call written as a
sentence](functions-sentences.md#starting-and-ending-phrases) of a function of [the core
module](core.md): `သိမ်းထား`, `ပြန်လည်စွန့်လွှတ်`, `နေရာအချဲ့အချုံ့လုပ်`, `ဖြည့်`, `ကူးယူ` and
`ပြောင်းထည့်`. These names are not reserved words, so a file that declares a function with
one of these names hides the function of the core module. As in each sentence call, an
argument is a whole clause: a name, a number, a size, `ပ ၏ တန်ဖိုး`, a field or a
conversion. A heap statement returns no value, so using one as a value is ZK0315. Each
heap statement needs unsafe code (ZK0468), and a heap statement whose place is not a
pointer is ZK0417:

```zaka-error ZK0417
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ၈ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို က တွင် သိမ်းထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0417]: the heap statement needs a pointer, found isize
```

### Sizes

```text
<kind> <count> <word>စာ
<count> ဘိုက်စာ
```

A heap statement needs a size, and we can write it in one of two forms:

| Form | Asks for |
| --- | --- |
| `<kind> <count> <word>စာ` | room for `<count>` values of the data kind, where `<word>` is its [measure word](kinds.md#measure-words) (classifier): `ကိန်းပြည့် ၃ ခုစာ`, `စာလုံး ၅ လုံးစာ`, `လူ ၂ ယောက်စာ` |
| `<count> ဘိုက်စာ` | `<count>` octets (bytes) |

The count is a `လက္ခဏာမဲ့ ကိန်းပြည့်`, so a literal fits as written, and a variable that
we use as a count must have that data kind. Another `လက္ခဏာမဲ့ ကိန်းပြည့်` value in place of
a size counts octets, as `n ဘိုက်စာ` does. Outside the heap statements, both forms are
values too: the [size](kinds.md#the-size-of-a-data-kind) in octets, as a
`လက္ခဏာမဲ့ ကိန်းပြည့်`. A form with a data kind multiplies the count with the [size of the
data kind](pointers.md#the-size-of-a-data-kind), and a product that does not fit counts as running
out of room.

### Asking for a block

```text
မင်မိုရီတွင် <size> နေရာယူ၍ လိပ်စာကို <place> တွင် သိမ်းထားမည်။
```

This statement reserves a new block and stores its address in the place, which must have
a pointer kind. The VM fills a new block with zeros, but the language promises nothing
about what a new block contains, so a program should write before it reads. A size of 0
returns a block that is not null and has no room. When the heap has no room left, the
place gets the null address, so a careful program compares the place with
`မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` before it uses the block. The VM starts with 256 MiB of
heap, and `zakac run --max-heap <octets>` sets another limit.

### Resizing a block

```text
လိပ်စာ <place> ရှိ မင်မိုရီကို <size> ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။
```

This statement grows or shrinks the block whose address is in the place, and stores the
address of the result back in the place, because the block can move. The values that fit
the new size are kept, and a larger block has new room after them. When the place
contains the null address, the statement asks for a new block. When the heap has no room
for the new size, the program stops with "no heap room" instead of setting the place to
the null address, because that would lose the block that the place still points at.

### Freeing a block

```text
လိပ်စာ <place> ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

This statement frees the block and sets the place to the null address. Freeing a place
that contains the null address does nothing, which is the reason the second free in the
first example is harmless. The statement clears this one place alone, so another pointer
to the block now points at a block that is gone. On the VM, a use of such a pointer stops
the program, and the compiler warns before the run when it can see the mistake (ZK0704).

## Writing and filling cells

```text
လိပ်စာ <src> ရှိ <size> မင်မိုရီကို လိပ်စာ <dst> ဆီသို့ ကူးယူမည်။
လိပ်စာ <src> ရှိ <size> မင်မိုရီကို လိပ်စာ <dst> ဆီသို့ ပြောင်းထည့်မည်။
လိပ်စာ <dst> ရှိ မင်မိုရီကို <value> ဖြင့် <size> ဖြည့်မည်။
```

We use these three statements to write a whole run of cells at once. Each begins with
`လိပ်စာ <pointer> ရှိ`, as resizing does, and each has a size of the cells it writes, in one
of the size forms of [the heap statements](#heap-statements): `<kind> N <word>စာ` for a count of cells with the measure word
of their data kind, or `N ဘိုက်စာ` for a count of octets.

| Statement | What it does | In C |
| --- | --- | --- |
| `ကူးယူမည်` | writes the cells at `<src>` to `<dst>`; the two ranges must not overlap | `memcpy` |
| `ပြောင်းထည့်မည်` | writes the cells at `<src>` to `<dst>`; it reads the whole source before it writes, so the ranges can overlap, as when a run of cells moves one place on | `memmove` |
| `ဖြည့်မည်` | sets each cell at `<dst>` to `<value>` | `memset`, for each data kind |

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို က တွင် သိမ်းထားမည်။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ခ တွင် သိမ်းထားမည်။

လိပ်စာ က ရှိ မင်မိုရီကို ၇ ဖြင့် ကိန်းပြည့် ၄ ခုစာ ဖြည့်မည်။ // fill
က ၏ တန်ဖိုး ကို ၁ ဟု ထားမည်။
က + ၁ ၏ တန်ဖိုး ကို ၂ ဟု ထားမည်။
က + ၂ ၏ တန်ဖိုး ကို ၃ ဟု ထားမည်။
လိပ်စာ က ရှိ ကိန်းပြည့် ၄ ခုစာ မင်မိုရီကို လိပ်စာ ခ ဆီသို့ ကူးယူမည်။ // two blocks
လိပ်စာ က ရှိ ကိန်းပြည့် ၃ ခုစာ မင်မိုရီကို လိပ်စာ က + ၁ ဆီသို့ ပြောင်းထည့်မည်။ // one block
၀ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ "{က + န ၏ တန်ဖိုး} {ခ + န ၏ တန်ဖိုး}" ဟု ပြသမည်။
လိပ်စာ က ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လိပ်စာ ခ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁ ၁
၁ ၂
၂ ၃
၃ ၇
```

### The operands

- `<src>` and `<dst>` can be values of each pointer kind or the opaque `လိပ်စာ`. Neither
  needs to be a variable (`က + ၁` works), because the statement writes nothing back to
  them. The data kinds of `<src>`, `<dst>` and the cells need not match, and a value that
  is not a pointer is ZK0417.
- The count is a `လက္ခဏာမဲ့ ကိန်းပြည့်`, as in the other heap statements, and another data
  kind is ZK0401.
- The value of a fill has the data kind of the cells when the size is `<kind> N <word>စာ`,
  and the data kind `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်` when the size is `N ဘိုက်စာ`. Another data
  kind is ZK0401. A struct value can fill cells as well:
  `အမှတ်(ဒေါင်: ၁၊ အလျား: ၂) ဖြင့် အမှတ် ၃ ခုစာ`.
- The value is a notation expression, as each argument is, so `က / ၂ ဖြင့် ၁ ဘိုက်စာ` fills
  with the quotient, and a clause goes in parentheses:
  `(က ကို ၂ ဖြင့် စားသော စားလဒ်) ဖြင့် ၁ ဘိုက်စာ`.

### Checks

A count of 0 does nothing, and then neither the compiler nor the VM checks the addresses.
With another count, the VM checks that the source range lies in one live block and that
the destination range lies in one live block that the program can write. Each failure
stops the program as a bad access does: out of bounds, use after free, a null address, or
a write into static text. A count whose product with the size of a cell does not fit is
out of bounds as well.

`ကူးယူမည်` with overlapping ranges gets the warning ZK0721 when both addresses point into
one block at offsets that the compiler knows and the count is known. Otherwise the VM
stops the program with the trap `overlap` (see [When a program stops](../vm/traps.md)).
We use `ပြောင်းထည့်မည်` when the ranges can overlap.

The compiler treats the destination as a write through its address, as it treats a store
through a pointer: a destination that is the null address for certain, or whose first or last
cell lies past the end of its block, gets a finding, and the compiler forgets what it knew
about the slots that the destination reaches. It treats the source as a read through its
address, with the same checks of its first and last cell. The last cell is known when the
count is known on each path.

### Cells with counted blocks

The cells must not contain a counted block (ZK0442): text, a plain list, a shared object,
a function value, or a struct that contains one of these. Each such cell keeps a count on
its block, and a duplicate made octet for octet would give the block a second holder that
the count does not know about. The compiler checks the data kind of the size, or, for a
size in octets, the data kind that each address points at. To move such cells, we assign
them one at a time. Cells that contain pointers are plain addresses, as in C, so a block
write or a fill can handle them.

```zaka-error ZK0442
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် စာသား လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် စာသား လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် စာသား ၂ ခုစာ နေရာယူ၍ လိပ်စာကို က တွင် သိမ်းထားမည်။
မင်မိုရီတွင် စာသား ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ခ တွင် သိမ်းထားမည်။
လိပ်စာ က ရှိ စာသား ၂ ခုစာ မင်မိုရီကို လိပ်စာ ခ ဆီသို့ ကူးယူမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0442]: the cells are "text", which hold counted blocks: a raw duplicate or fill would not keep their counts
```

## Owned pointers

```text
<name> ဟူသော <kind> လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့်
<kind> လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။
```

A pointer kind does not show who frees the block. To state it, we write
`အပိုင်အနေဖြင့်` ("as its own") on a pointer parameter or a pointer result, which makes it
an owned pointer (owning pointer), and the compiler checks the hand-over.

### Owned parameters

We put `အပိုင်အနေဖြင့်` after the `ကို` of a parameter (see
[Function parameters](functions-parameters.md#writing-a-parameter)) when the function takes over the block. After the
call, the caller's variable contains the null address, because the call stores it there,
and the function must now free the block, hand it on or store it. So an argument for an
owned parameter must be a place that the call can clear. When the program reads the
cleared place after the hand-over on each path, the compiler reports ZK0701, which is an
error in each mode:

```zaka-error ZK0701
ရှင်း ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည်
ပ ဟူသော ကိန်းပြည့် လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ရှင်း ဟူသော လုပ်ဆောင်ချက်ကို ပ ဖြင့် လုပ်ဆောင်မည်။
ပ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0701]: "ပ" handed its block to a parameter marked "အပိုင်အနေဖြင့်" and now points at nothing
```

A hand-over on one branch alone leaves the place null on that branch. The compiler does
not report it, and on the VM a read through the place stops the program. When another
argument of the same call also reaches the block, the compiler warns with ZK0707, because
the function could free the block while it still reads it through the other argument.
When the parameter is also a mutable parameter (`inout`), with `ဒေတာ ပြောင်းလဲရန်` beside
`အပိုင်အနေဖြင့်`, the function takes over the old block, and the caller owns whatever
block the function writes back.

### Owned results

We write `<kind> လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်` in the function signature when
the function returns a block to the caller, and the caller must then free the block, hand
it on or store it. When a function returns a block that it asked for and its result has
no `အပိုင်အနေဖြင့်`, the compiler warns about a leak with ZK0708, and its help line asks
for `အပိုင်အနေဖြင့်`.

### Where it is allowed

`အပိုင်အနေဖြင့်` hands over a block, so we can use it on pointer kinds alone. On another
data kind it is ZK0470, because other values are duplicated or shared and hand nothing
over. The one exception is a parameter of a C function whose data kind comes from outside
the program and which the C function closes ([External and C functions](functions-c.md#data-kinds-from-outside-the-program)).

```zaka-error ZK0470
ယူ ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို အပိုင်အနေဖြင့် လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ယူ ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0470]: "အပိုင်အနေဖြင့်" hands over a block, and "isize" holds no pointer to one
```

## Errors

| Code | Mistake |
| --- | --- |
| ZK0315 | a heap statement used as a value |
| ZK0401 | a count or a fill value of the wrong data kind |
| ZK0417 | a heap statement given something other than a pointer |
| ZK0442 | cells that contain a counted block, in a block write or a fill |
| ZK0468 | a heap statement outside unsafe code |
| ZK0470 | `အပိုင်အနေဖြင့်` on a data kind that is not a pointer |
| ZK0701 | a read of a place after it handed its block to an owned parameter |
| ZK0702 to ZK0708, ZK0721 | findings of the pointer lint, in [The checks on unsafe code](unsafe.md#the-checks-on-unsafe-code) |

The text of each message is in [Error messages](../reference/errors.md).
