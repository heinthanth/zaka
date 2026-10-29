---
title: The heap
description: Ask for heap room while the program runs, use it through a pointer, and give it back.
---

# The heap

A variable lasts as long as its block or function. Sometimes a program needs room that
remains until *the programmer* decides to let it go, or room whose size is not known
before the program runs. That room comes from the *heap*: ask for a block of it, use it
through a pointer, and give it back when done.

```zaka
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။    // room for one integer
ပ ၏ တန်ဖိုး ကို ၄၂ ဟု ထားပါ။
ပ ၏ တန်ဖိုး ကို ပြသပါ။
ပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။                      // give it back
ပ ကို ပြသပါ။                                                  // the pointer is now null
```

```text
၄၂
ဗလာ
```

Asking for heap room and giving it back are statements of their own:

| Zaka | Reads as |
| --- | --- |
| `မင်မိုရီ ... ခုစာ နေရာယူ၍` | "taking room worth ... items" |
| `လိပ်စာကို ပ တွင် သိမ်းထားမည်။` | "and will keep the address in ပ" |
| `ပ လိပ်စာရှိမင်မိုရီကို` | "the block at the address in ပ" |
| `ပြန်လည်စွန့်လွှတ်မည်။` | "will give back" |

`မင်မိုရီ` is borrowed from English and means RAM; the native Burmese word `မှတ်ဉာဏ်`
works the same.
Statements written with `…မည်` also accept the spoken `…မယ်`:

```zaka
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မှတ်ဉာဏ် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမယ်။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ။
ပ ၏ တန်ဖိုး ကို ပြသပါ။
ပ လိပ်စာရှိ မှတ်ဉာဏ်ကို ပြန်လည်စွန့်လွှတ်မယ်။
```

```text
၇
```

## How much to ask for

Give the size in one of two forms:

| Size | Means |
| --- | --- |
| `ကိန်းပြည့် ၁၀ ခုစာ` | room for 10 integers |
| `၈၀ ဘိုက်စာ` | room for 80 octets |

Most of the time the first form is the one to use. The second counts octets, the
smallest unit of storage. `<kind> ၏ အရွယ်အစား` ("the size of the kind") tells how much room,
in octets, one value of a kind takes:

```zaka
ကိန်းပြည့် ၏ အရွယ်အစား ကို ပြသပါ။
၃၂ ဘစ် ကိန်းပြည့် ၏ အရွယ်အစား ကို ပြသပါ။
မှန်မှားတန်ဖိုး ၏ အရွယ်အစား ကို ပြသပါ။
ကိန်းပြည့် လိပ်စာ ၏ အရွယ်အစား ကို ပြသပါ။
```

```text
၈
၄
၁
၈
```

So `ကိန်းပြည့် ၁၀ ခုစာ` and `ကိန်းပြည့် ၏ အရွယ်အစား * ၁၀ ဘိုက်စာ` ask for the same room.

The count is a `လက္ခဏာမဲ့ ကိန်းပြည့်` (an unsigned integer). A literal such as `၁၀` fits
on its own. A variable used as a count must have that kind.

## When there is no room

If the heap has no room left, the pointer gets `ဗလာ` instead of an address. A careful
program checks:

```zaka
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၁၀ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
အကယ်၍ ပ သည် ဗလာ နှင့် ညီလျှင် "နေရာ မရပါ" ဟု ပြသပါ။
သို့မဟုတ်ပါက "နေရာ ရပါပြီ" ဟု ပြသပါ။
ပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
နေရာ ရပါပြီ
```

The Zaka machine gives each program 256 MiB of heap; the Explorer gives 16 MiB. `zakac
run --max-heap <octets>` sets another limit.

## Giving a block back

Each block from the heap should be given back one time, no more and no less. The pointer
that receives the block *owns* it, and it alone gives the block back. Giving it back:

- sets the owner to `ဗလာ`, so using it again finds null instead of a freed block;
- does nothing when the owner is `ဗလာ` to begin with;
- is an error the compiler reports while a borrow of the block is still in use; see
  [owners and borrows](ownership.md) and [Pointer mistakes](mistakes.md).

The compiler reports a block that a function loses before it returns (ZK0708). For the
blocks that remain when the program ends, run it with `zakac run --leaks`; it lists each
forgotten block.

## Hands on

- Ask for room for three integers, store 1, 2 and 3 in the first one and nowhere else,
  and print it.
  (The next page shows how to reach the other two.)
- Ask for the same room with the octet form and `ကိန်းပြည့် ၏ အရွယ်အစား`.
- Run a program that never gives its block back with `zakac run --leaks`.

Next: [Pointer arithmetic](arithmetic.md).
