---
title: The heap
description: Ask the heap for a block in unsafe code, use it through a pointer, and give it back.
---

# The heap

A variable lasts as long as its block of code or its function, but sometimes a program
needs room that lasts until the program itself decides to let it go, as C does with
`malloc` and `free`. That room comes from *the heap* (dynamic memory): we ask it for a
block (allocation), use the block through a pointer, and give it back when we are done.

## You will learn

- how to ask the heap for a block and how to give the block back
- how to tell the heap how much room we need
- what happens when the heap has no room left
- what giving a block back does to the pointer

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။  // room for one integer
ပ ၏ တန်ဖိုး ကို ၄၂ ဟု ထားမည်။
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။                    // give it back
ပ ကို ပြသမည်။                                                  // the pointer is now null
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၂
မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ
```

We use one statement to ask for a block, which also puts its address in a pointer
variable, and another to give the block back through that variable. Both are heap
statements, so both go in unsafe code:

```text
မင်မိုရီတွင် <size> နေရာယူ၍ လိပ်စာကို <pointer> တွင် သိမ်းထားမည်။
လိပ်စာ <pointer> ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

| Zaka | Reads as |
| --- | --- |
| `မင်မိုရီတွင်` | "in RAM" |
| `ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍` | "taking room worth one integer, and" |
| `လိပ်စာကို ပ တွင် သိမ်းထားမည်။` | "will keep the address in ပ" |
| `လိပ်စာ ပ ရှိ မင်မိုရီကို` | "the RAM at the address in ပ" |
| `ပြန်လည်စွန့်လွှတ်မည်။` | "will give back" |

Text, lists and shared objects need none of this, because the compiler frees them on its
own. The heap statements are for the blocks that unsafe code manages itself.

## How much to ask for

```text
<kind> <count> <measure word>စာ
<count> ဘိုက်စာ
```

We can give the size of a block in two forms:

| Size | Means |
| --- | --- |
| `ကိန်းပြည့် ၁၀ ခုစာ` | room for 10 integers |
| `၈၀ ဘိုက်စာ` | room for 80 octets (bytes) |

Most of the time we use the first form. The second counts octets, the smallest unit of
storage, and to find the size in octets of one value of a data kind, we write
`<kind> တစ်ခု ၏ အရွယ်အစား` ("the size of one of the data kind"):

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းပြည့် တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
မှန်မှားတန်ဖိုး တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
ကိန်းပြည့် လိပ်စာ တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၈
၄
၁
၈
```

So `ကိန်းပြည့် ၁၀ ခုစာ` and `ကိန်းပြည့် တစ်ခု ၏ အရွယ်အစား * ၁၀ ဘိုက်စာ` ask for the same
room. The word before `စာ` is the measure word (classifier) of the data kind, so
`စာလုံး ၅ လုံးစာ` asks for room for five characters.

Both forms are values too, so a program can work out a size before it asks for it:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အရွယ် သည် ကိန်းပြည့် ၁၀ ခုစာ ဖြစ်ပါစေ။
အရွယ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၈၀
```

The count is a `လက္ခဏာမဲ့ ကိန်းပြည့်` (an unsigned integer), and a number such as `၁၀`
fits on its own, but a variable that we use as a count must have that data kind.

## When there is no room

When the heap has no room left, the pointer gets the null address instead of an address,
so a careful program compares the pointer with `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` before it
uses the block:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁၀ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
အကယ်၍ ပ သည် မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ နှင့် ညီလျှင် "နေရာ မရပါ" ဟု ပြသမည်။
သို့မဟုတ်ပါက "နေရာ ရပါပြီ" ဟု ပြသမည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
နေရာ ရပါပြီ
```

The VM gives each program 256 MiB of heap, and we can set another limit with
`zakac run --max-heap <octets>`. In the Explorer, the **Heap limit** setting does the
same, and its default is the RAM that the browser reports.

## Giving a block back

We give each block from the heap back one time, never twice and never zero times. When we
give it back through a pointer variable:

- that variable gets the null address, so a later use of it finds null instead of a freed
  block
- nothing happens when the variable has the null address to begin with

A duplicate (copy) of the pointer in another variable still has the old address, and a
use of it after the block is given back stops the program. The compiler warns about a
block that a function loses before it returns (ZK0708), and to find the blocks that remain
when the program ends, we run it with `zakac run --leaks`, which lists each forgotten
block. The page on [owned pointers](ownership.md) shows how a function signature tells
which function gives a block back.

## Details

- `မင်မိုရီ` is a loan word from English, and the Burmese word `မှတ်ဉာဏ်` works the same
  in both statements: `မှတ်ဉာဏ်တွင် ... နေရာယူ၍` and `လိပ်စာ ပ ရှိ မှတ်ဉာဏ်ကို
  ပြန်လည်စွန့်လွှတ်မည်။` ([Heap statements](../../manual/language/heap.md#heap-statements)).
- On the VM, a new block starts out filled with zeros, as the page on [pointer
  mistakes](mistakes.md#reading-past-the-end) shows.
- The size of each data kind, and how `အရွယ်အစား` works on structs and lists, is in [The size
  of a data kind](../../manual/language/pointers.md#the-size-of-a-data-kind).
- Giving back the address of a variable, or an address inside a block, stops the program
  ([Pointer mistakes](mistakes.md#giving-back-a-block-that-was-not-asked-for)).
- `--leaks` and `--max-heap` are VM flags, and [Running a
  program](../../manual/vm/running.md#the-options) lists them with the other flags of
  `zakac run`.

## Recap

- We use `မင်မိုရီတွင် <size> နေရာယူ၍ လိပ်စာကို <pointer> တွင် သိမ်းထားမည်။` to ask the
  heap for a block, and `လိပ်စာ <pointer> ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။` to give
  it back.
- We give the size as a count of a data kind, `ကိန်းပြည့် ၁၀ ခုစာ`, or as octets,
  `၈၀ ဘိုက်စာ`.
- When the heap has no room, the pointer gets the null address.
- Giving a block back sets the pointer to the null address, but a duplicate of the pointer
  still has the old address.

## Hands on

- Ask for room for three integers, store 1, 2 and 3 in the first one, and print it. (The
  next page shows how to reach the other two.)
- Ask for the same room with the octet form and `ကိန်းပြည့် တစ်ခု ၏ အရွယ်အစား`.
- Run a program that never gives its block back with `zakac run --leaks`.

Next: [Pointer arithmetic](arithmetic.md).
