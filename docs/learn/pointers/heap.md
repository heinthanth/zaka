---
title: The heap
description: Ask for heap room in unsafe code, use it through a pointer, and give it back.
---

# The heap

A variable lasts as long as its block or function. Sometimes a program needs room that
remains until *the programmer* decides to let it go, as C does with `malloc` and `free`.
That room comes from the *heap*: ask for a block of it, use it through a pointer, and
give it back when done. Each heap statement needs unsafe code. Text, lists and shared
objects need none of this: the compiler frees them.

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

Asking for heap room and giving it back are statements of their own:

| Zaka | Reads as |
| --- | --- |
| `မင်မိုရီတွင် ... ခုစာ နေရာယူ၍` | "in RAM, taking room worth ... items" |
| `လိပ်စာကို ပ တွင် သိမ်းထားမည်။` | "and will keep the address in ပ" |
| `လိပ်စာ ပ ရှိ မင်မိုရီကို` | "the block at the address in ပ" |
| `ပြန်လည်စွန့်လွှတ်မည်။` | "will give back" |

`မင်မိုရီ` is borrowed from English and means RAM; the native Burmese word `မှတ်ဉာဏ်`
works the same:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မှတ်ဉာဏ်တွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားမည်။
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လိပ်စာ ပ ရှိ မှတ်ဉာဏ်ကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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
smallest unit of storage. `<kind> တစ်ခု ၏ အရွယ်အစား` ("the size of one of the kind") tells how much room,
in octets, one value of a kind takes:

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

So `ကိန်းပြည့် ၁၀ ခုစာ` and `ကိန်းပြည့် တစ်ခု ၏ အရွယ်အစား * ၁၀ ဘိုက်စာ` ask for the same room.
The word before `စာ` is the measure word of the kind: `စာလုံး ၅ လုံးစာ` asks for room for five
characters.

The count is a `လက္ခဏာမဲ့ ကိန်းပြည့်` (an unsigned integer). A literal such as `၁၀` fits
on its own. A variable used as a count must have that kind.

## When there is no room

If the heap has no room left, the pointer gets the null address, `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`, instead of an
address. A careful
program checks:

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

The Zaka VM gives each program 256 MiB of heap; the Explorer gives 16 MiB. `zakac
run --max-heap <octets>` sets another limit.

## Giving a block back

Each block from the heap should be given back one time, no more and no less. Giving it
back through a pointer variable:

- sets that variable to the null address, so using it again finds null instead of a
  freed block;
- does nothing when the variable holds the null address to begin with.

A duplicate of the pointer in another variable still holds the old address, and a use of it
after the give-back stops the program. The compiler warns about a block that a function
loses before it returns (ZK0708). For the blocks that remain when the program ends, run
it with `zakac run --leaks`; it lists each forgotten block. [Owned
pointers](ownership.md) tells which function gives a block back.

## Hands on

- Ask for room for three integers, store 1, 2 and 3 in the first one and nowhere else,
  and print it.
  (The next page shows how to reach the other two.)
- Ask for the same room with the octet form and `ကိန်းပြည့် တစ်ခု ၏ အရွယ်အစား`.
- Run a program that never gives its block back with `zakac run --leaks`.

Next: [Pointer arithmetic](arithmetic.md).
