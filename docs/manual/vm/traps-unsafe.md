---
title: Stops in unsafe code
description: The reasons the VM stops a program that works with addresses or calls C: each message, its common cause and how to fix it.
---

# Stops in unsafe code

This page continues [When a program stops](traps.md) with the stops (traps) that come
from [unsafe code](../language/unsafe.md): work with addresses (pointers) and blocks
(allocations), and calls of C functions. Each section gives the message, the cause and
the fix, and the table of all the reasons is on [When a program
stops](traps.md#all-the-traps). [Addresses and blocks](blocks.md) describes the checks
behind these stops, and Learn walks through the common mistakes in [Pointer
mistakes](../../learn/pointers/mistakes.md).

The programs on this page run under the default `--lint-warn mem`, so the checker warns
about several of them and the VM (virtual machine) then shows its own check. With
`--lint-error mem`, the compiler refuses those programs before a run.

## Addresses

All the stops of this section come from unsafe code, where the VM checks each address
(pointer) as [Addresses and blocks](blocks.md) describes.

### Null address

An access below the first block is a null access. That covers the null address,
`မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`, and each small address near it. A step from the null
address is the same stop, since no offset turns null into the address of a block. The
checker warns about the read below (ZK0709), and the VM stops it:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။ // the null address
"အစ" ဟု ပြသမည်။
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အစ
the program stopped: access through a null address at 0x0
```

To fix it, we test the address against `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` before the read,
or set it first.

### Out of bounds

An access must lie inside one live block. An address in the gap after a block, or in no
block at all, such as a number turned into an address, is out of bounds:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် ၁၂၃၄၅၆ ကို ကိန်းပြည့် လိပ်စာ တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
ရ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: access of 8 octets at 0x1e240 lands outside each live block
```

An access that starts inside a block and runs past its end is the same stop. Reading cell
4 of a block of 4 integers starts at the end of the block, in the gap:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ + ၃ ၏ တန်ဖိုး ကို ပြသမည်။ // the last cell: fine
ပ + ၄ ၏ တန်ဖိုး ကို ပြသမည်။ // one past the end
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၀
the program stopped: access of 8 octets at 0x101020 lands outside each live block
```

### Step out of bounds

Forming an address reads nothing, so it can reach one past the end of a block. A step that
lands farther, or before the start, stops the program at once. The step reads nothing, so
its message names no octet count:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် နေရာ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀၀၀ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် ဝေး သည် ပ + နေရာ ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: a pointer step to 0x102f40 leaves the block it started in
```

### Misaligned

An integer of 8 octets must sit at a multiple of 8, and a character at a multiple of 4. An
address made from a block of octets plus one is neither:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ဘိုက် သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ၁၆ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို ဘိုက် တွင် သိမ်းထားမည်။
ကိန်းရှင် ရ သည် ဘိုက် + ၁ ကို ကိန်းပြည့် လိပ်စာ တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
ရ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: address 0x101001 is not a multiple of 8
```

### Bad value

Not each pattern of octets is a value. A true/false octet must be `0` or `1`, and a
character must be a Unicode scalar value, so reading other octets as one of these data
kinds stops the program:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ဘိုက် သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ၁၆ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို ဘိုက် တွင် သိမ်းထားမည်။
ဘိုက် ၏ တန်ဖိုး ကို ၇ ဟု ထားမည်။
ကိန်းရှင် အမှန် သည် ဘိုက် ကို မှန်မှားတန်ဖိုး လိပ်စာ တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
အမှန် ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: the octets at 0x101000 do not form a bool
```

### Use after free

A freed block remains in the table, so an access through a stale address finds it. The heap
block behind an address that was freed, and the block of a local whose call has returned,
both stop the program in this manner:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားမည်။
ကိန်းရှင် ဒုတိယ သည် ပ ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ဒုတိယ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: access at 0x101000 inside a freed block
```

A function that returns the address of one of its locals leaves the caller with the same
fault. The checker warns about such a function (ZK0706), the VM shows the fault, and the
long form of the message names the variable:

```zaka-stop
ဒေသ ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် လိပ်စာ တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် ပမာဏ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
ပမာဏ ၏ လိပ်စာ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ဒေသ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: access at 0x1020 inside a freed block
```

```console
$ zakac run --vm program.zaka
zaka_vm: the program stopped: access at 0x1020 inside a freed block (in ပရိုဂရမ်, at program.zaka:10:1); block at 0x1020, 8 octets, ပမာဏ of ဒေသ
```

Without `--vm`, native code runs this program with no check and prints whatever the
octets contain, which is the reason to keep such programs on the VM while we look for the
mistake.

### Double free

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် ဒုတိယ သည် ပ ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လိပ်စာ ဒုတိယ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: the block at 0x101000 was freed before
```

Freeing the null address does nothing and does not stop the program. That is the reason
the free statement leaves the null address in the variable it frees, so a second free of
the same variable is harmless. The fault above comes from a second variable, `ဒုတိယ`,
that still has the old address.

### Bad free

Just the start of a live heap block can be freed. The block of a variable, an address
inside a block and a wild address cannot:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: 0x1000 is not the start of a live heap block
```

### Mixed blocks

The distance between two addresses means something just inside one block, so the VM
checks that one live block reaches both addresses, its end included:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို က တွင် သိမ်းထားမည်။
မင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ခ တွင် သိမ်းထားမည်။
က နှင့် ခ ၏ နုတ်လဒ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: pointers into different blocks cannot be subtracted
```

### Overlap

A `ကူးယူမည်` writes cells as `memcpy` does in C, so its source and destination ranges
must not share an octet. When the two share one, the VM stops the program before it writes and
names the block of the destination. When both addresses point into one block at offsets
the compiler knows, and the count is known, the checker reports the overlap first (ZK0721,
a warning in unsafe code under the default `--lint-warn mem`). To fix it, we use
`ပြောင်းထည့်မည်`, which allows the overlap because it reads the whole source before it
writes:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို က တွင် သိမ်းထားမည်။
လိပ်စာ က ရှိ မင်မိုရီကို ၀ ဖြင့် ကိန်းပြည့် ၄ ခုစာ ဖြည့်မည်။
လိပ်စာ က ရှိ ကိန်းပြည့် ၃ ခုစာ မင်မိုရီကို လိပ်စာ က + ၁ ဆီသို့ ကူးယူမည်။
လိပ်စာ က ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: the cells at 0x101008 overlap their source; "ပြောင်းထည့်မည်" allows that
```

The other checks of these statements stop the program as an access does (out of bounds,
use after free, a null address, a write into static text). A count of 0 checks nothing,
and a count whose product with the size of a cell does not fit is out of bounds.

### Frozen write

Each static text sits in a frozen block, which each literal of the same text shares
([The address space](blocks.md#the-address-space)). The compiler refuses a write to a
cell of text (ZK0429), and a character write or an add first copies static text into a
heap block, so an address in unsafe code is the one route to such a write. The long form
names the block as `static text t0`, the index in the `texts` table of the listing:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် t သည် "abc" ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် p သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် t ၏ လိပ်စာ ကို လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် q သည် p ၏ တန်ဖိုး ဖြစ်ပါစေ။
q ၏ တန်ဖိုး ကို ၁၂၀ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: write at 0x1000 inside static data, which cannot change
```

```console
$ zakac run --vm program.zaka
zaka_vm: the program stopped: write at 0x1000 inside static data, which cannot change (in ပရိုဂရမ်, at program.zaka:6:1); block at 0x1000, 3 octets, static text t0
```

### Null call and null object

A function value is never null in safe code: a variable of a function kind gets a function
before each read, as the check of [unset values](../language/declarations.md#set-before-read)
proves. The `null call` stop remains as a backstop for the cases the check cannot see. One
is a heap block in unsafe code, which starts as zero octets, and zero octets are no
function:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် (ကိန်းပြည့် တစ်ခု ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား) လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် (ကိန်းပြည့် တစ်ခု ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား) ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် လ သည် ပ ၏ တန်ဖိုး ဖြစ်ပါစေ။ // a new block contains zero octets
"အစ" ဟု ပြသမည်။
လ ဟူသော လုပ်ဆောင်ချက်ကို ၅ ဖြင့် လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အစ
the program stopped: the function value holds no function to call
```

`null object` is the same backstop for a field read or written through a shared object
(reference type) variable that has no object.

## C functions

Some stops belong to a [C function](../language/functions-c.md#c-functions). Those of a
call stop the program before C is entered. C itself does what C does, and the VM has no
check for what C does with a block.

### Null from C

C returns NULL where its declaration promised a plain value, and Zaka has no NULL outside
addresses to keep it in. Three places stop the program:

- a C function whose result is a plain handle, written with no optional, returns NULL;
- a function of the program that C calls back gets NULL as a `char *` for a parameter of
  plain text, `စာသား`;
- a function of the program that C calls back gets NULL for a parameter of a plain handle
  data kind.

To fix it, we declare the result or the parameter as an optional when C can return NULL:
`စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` gets `အလွတ်` for NULL, and so does an optional of a
handle (see [Data kinds from outside the
program](../language/functions-c.md#data-kinds-from-outside-the-program)).

```text
the program stopped: C gave NULL where its declaration promised a value; declare the result or the parameter as an optional, "T ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်", when C can give NULL
```

### C function not run

The VM of the Explorer is built with no `ffi`, so it opens no lib: the first call of a C
function stops the program, and what the program printed before the call remains printed.
`zakac run` and a native build run the call.

```text
the program stopped: the C function frexp runs in "zakac run" and in native builds, not in this VM
```

### C link

The first call in a run opens each lib that the program names and looks up each C name in
its lib, then in the process, which has libc. The program stops when a lib does not open,
when no lib has a C name, or when the program is built for addresses of another size than
the host's (`--target-bits`). The reason follows the colon:

| Reason | Message |
| --- | --- |
| a lib that does not open, such as a path that names no file | `the lib "L" does not open (E)` |
| a C name that no lib has | `the lib "L" holds no C function N` |
| a platform size that is not the host's | `the program has B-bit addresses, but this host has H-bit ones` |

```zaka-stop
nosuchfn ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libm" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ n ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
nosuchfn ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: the C functions of the program cannot be reached: the lib "libm" holds no C function nosuchfn
```

### Closed handle

A [handle](../language/functions-c.md#data-kinds-from-outside-the-program) is a data kind
from outside the program. A C function that takes one over, with a parameter marked
`အပိုင်အနေဖြင့်`, leaves the box of the handle at 0, so each variable that shares it sees
it closed. A call of a C function with a closed handle, and a read of a field through
one, stop the program before C is entered or read. A handle is never NULL in a variable:
a plain handle result that C returns as NULL stops the program with `null from C`
(above). The stop comes from the operation `halt`, with the cause `closed_handle`, and a
native build stops with the same message. The C lib that the program needs cannot run in
the Explorer, so the case below is shown as a file:

```zaka-file
// closed.zaka
ရေတွက်စက် ဟူသော ဒေတာအစုအဝေးသည် ပြင်ပမှ လာသော ဒေတာအစုအဝေး ဖြစ်၍ ၎င်းတွင် count ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

counter_open ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libcounter" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
step ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ရေတွက်စက် တစ်ခု ကို ပြန်ပေးမည်။
counter_close ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libcounter" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
c ဟူသော ရေတွက်စက် တစ်ခု ကို အပိုင်အနေဖြင့် လက်ခံမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် စက် သည် counter_open ဟူသော လုပ်ဆောင်ချက်ကို ၃ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် တူ သည် စက် ဖြစ်ပါစေ။
counter_close ဟူသော လုပ်ဆောင်ချက်ကို စက် ဖြင့် လုပ်ဆောင်မည်။
// တူ shares the box that counter_close emptied
တူ ၏ count ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: a C function that took the handle over closed it, so it cannot be used again
```

### Closure escape

A closure passed to a [C function](../language/functions-c.md#callbacks) is lent to that
call, so C can call it while the call runs. When C calls it at another time, with no call
that was given it running and no leave from the program, the VM stops the program. A
[leave](../language/closures.md#closures-and-c) makes the call valid, and a revoke ends
it. The native code of a closure remains in place after its last use, so a late call stops
the program and never runs freed code.

The program below revokes the leave and then raises the signal. The Explorer runs no C, so
this stop shows under `zakac run`, and a native build stops with the same message:

```zaka-file
// late.zaka
signal ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
sig ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း handler ဟူသော (၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို လည်းကောင်း လက်ခံမည်။
raise ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ sig ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖမ်းကြိမ် သည် ၀ ဖြစ်ပါစေ။
ဖမ်း ဟူသော လုပ်ဆောင်ချက် သည် s ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဖမ်းကြိမ် ကို ဖမ်းကြိမ် + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ဖမ်း ဟူသော လုပ်ဆောင်ချက်ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
signal ဟူသော လုပ်ဆောင်ချက်ကို ၁၅ နှင့် ဖမ်း ဟူသော လုပ်ဆောင်ချက် တို့ဖြင့် လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ဖမ်း ဟူသော လုပ်ဆောင်ချက်ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြန်လည်ရုပ်သိမ်းမည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
raise ဟူသော လုပ်ဆောင်ချက်ကို ၁၅ ဖြင့် လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: C called a closure after the call that gave it ended, and C has no leave to call it; give one with "ခေါ်ဆိုခွင့် ပြုမည်"
```

The VM runs on one OS thread, so a call from C on another OS thread ends the run with a
message for that case:
`C called a function of the program from an OS thread other than the one that runs it; the VM runs on one OS thread, so the run ends here`.

### A stop inside a callback

C calls a [callback](../language/functions-c.md#callbacks) while its C call is running,
and the VM cannot unwind C. So the VM keeps a stop in the callback: C gets 0 from that
callback, each later callback of that call returns 0 without running, and the program
stops with the kept reason once C returns. What C did in the meantime is not undone.
