---
title: Heap memory
description: Ask for memory while the program runs, use it through a pointer, and give it back.
---

# Heap memory

A variable lasts as long as its block or function. Sometimes you need memory that stays
until *you* decide it is no longer needed, or memory whose size you only know while the
program runs. That memory comes from the *heap*: you ask for a block of it, use it
through a pointer, and give it back when you are done.

```zaka
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။    // room for one integer
ပ ၏ တန်ဖိုး ကို ၄၂ ဟု ထားပါ။
ပ ၏ တန်ဖိုး ကို ပြသပါ။
ပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။                      // give it back
ပ ကို ပြသပါ။                                                  // the pointer is now empty
```

```text
၄၂
ဗလာ
```

Asking for memory and giving it back are statements of their own:

| Zaka | Reads as |
| --- | --- |
| `မင်မိုရီ ... ခုစာ နေရာယူ၍` | "taking memory room worth ... items" |
| `လိပ်စာကို ပ တွင် သိမ်းထားမည်။` | "and will keep the address in ပ" |
| `ပ လိပ်စာရှိမင်မိုရီကို` | "the memory at the address in ပ" |
| `ပြန်လည်စွန့်လွှတ်မည်။` | "will give back" |

`မင်မိုရီ` is the loanword for memory; the Burmese word `မှတ်ဉာဏ်` works the same.
Statements written with `…မည်` also accept the spoken `…မယ်`:

```zaka
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မှတ်ဉာဏ် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမယ်။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ။
ပ ၏ တန်ဖိုး ကို ပြသပါ။
ပ လိပ်စာရှိ မှတ်ဉာဏ်ကို ပြန်လည်စွန့်လွှတ်မယ်။
```

```text
၇
```

## How much to ask for

Give the size in one of two ways:

| Size | Means |
| --- | --- |
| `ကိန်းပြည့် ၁၀ ခုစာ` | room for 10 integers |
| `၈၀ ဘိုက်စာ` | room for 80 bytes |

The first form is usually what you want. The second counts bytes, the smallest unit of
memory. `<kind> ၏ အရွယ်အစား` ("the size of the kind") tells you how many bytes one value
of a kind takes:

```zaka
ကိန်းပြည့် ၏ အရွယ်အစား ကို ပြသပါ။
၃၂ ဘစ် ကိန်းပြည့် ၏ အရွယ်အစား ကို ပြသပါ။
မှန်မှားတန်ဖိုး ၏ အရွယ်အစား ကို ပြသပါ။
ကိန်းပြည့် လိပ်စာ ၏ အရွယ်အစား ကို ပြသပါ။
```

```text
၈
၄
၁
၈
```

So `ကိန်းပြည့် ၁၀ ခုစာ` and `ကိန်းပြည့် ၏ အရွယ်အစား * ၁၀ ဘိုက်စာ` ask for the same room.

The count is a `လက္ခဏာမဲ့ ကိန်းပြည့်` (an unsigned integer). A literal such as `၁၀` fits
on its own. A variable used as a count must have that kind.

## When there is no room

If the heap has no room left, the pointer gets `ဗလာ` instead of an address. A careful
program checks:

```zaka
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၁၀ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
အကယ်၍ ပ သည် ဗလာ နှင့် ညီလျှင် "နေရာ မရပါ" ဟု ပြသပါ။
သို့မဟုတ်ပါက "နေရာ ရပါပြီ" ဟု ပြသပါ။
ပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
နေရာ ရပါပြီ
```

The Zaka machine gives each program 256 MiB of heap; the Explorer gives 16 MiB. `zakac
run --max-heap <bytes>` sets another limit.

## Giving memory back

Each block you ask for should be given back exactly once. Giving it back:

- empties the pointer you name, so using it again finds `ဗလာ` instead of old memory;
- does nothing when the pointer is already `ဗလာ`;
- does not empty *other* pointers to the same block. Using one of those is a mistake the
  machine catches: see [Memory mistakes](mistakes.md).

To find blocks that were never given back, run the program with `zakac run --leaks`. It
lists each forgotten block when the program ends.

## Try it

- Ask for room for three integers, store 1, 2 and 3 in the first one only, and print it.
  (The next page shows how to reach the other two.)
- Ask for the same room with the byte form and `ကိန်းပြည့် ၏ အရွယ်အစား`.
- Run a program that never gives its block back with `zakac run --leaks`.

Next: [Pointer arithmetic](arithmetic.md).
