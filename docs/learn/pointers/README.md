---
title: Unsafe Zaka
description: Part III of Learn - mark code as unsafe with စိတ်မချရသော, then take the address of a variable and read and change the variable through it.
---

# Unsafe Zaka

This is Part III of Learn. It introduces addresses (pointers), which let us read or
change a value at a location in RAM (memory). It also covers heap blocks (allocations)
that the program manages itself. We use these tools in unsafe code, for example when
calling C.

## You will learn

- how to mark a block of code as unsafe code with `စိတ်မချရသော်လည်း`
- how to take the address of a variable and read or change the variable through it
- how to write the data kind of a pointer, and what the null address (null pointer) is
- which values have an address and which do not

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။          // age = 18
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ညွှန်း သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အသက် ၏ လိပ်စာ ဖြစ်ပါစေ။ // a pointer to age
ညွှန်း ၏ တန်ဖိုး ကို ပြသမည်။                                 // read through it
ညွှန်း ၏ တန်ဖိုး ကို ၂၀ ဟု ထားမည်။                              // write through it
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အသက် ကို ပြသမည်။                                            // age has changed
```

```text
၁၈
၂၀
```

After its first line, the program never assigns to `အသက်` through its name, because it
changes the variable through the pointer `ညွှန်း` ("the pointer") instead. The compiler
cannot prove that such code is safe, so we put it in an unsafe block, and inside it we
use `၏ လိပ်စာ` to take an address and `၏ တန်ဖိုး` to reach the value at an address:

```text
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
<sentences>
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

<variable> ၏ လိပ်စာ
<pointer> ၏ တန်ဖိုး
```

| Zaka | Reads as |
| --- | --- |
| `အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။` | "will do the following, though it is not to be trusted" |
| `ကိန်းပြည့် လိပ်စာ` | "integer address": the data kind of a pointer to an integer |
| `အသက် ၏ လိပ်စာ` | "the address of age" |
| `ညွှန်း ၏ တန်ဖိုး` | "the value at the pointer" |

The word for unsafe code is `စိတ်မချရသော` ("not to be trusted"). Besides a block, a
whole function can be unsafe when its function signature starts with that word, which
the page on [owned pointers](ownership.md#unsafe-functions) shows.

## What unsafe means

Unsafe means that *the compiler* no longer proves each pointer right, but the rules still
hold. The VM checks each read and write through a pointer and stops the program at the
first bad one, as [Pointer mistakes](mistakes.md) shows, and inside unsafe code the
compiler reports the pointer mistakes it can see as warnings and lets the program run.

`zakac run` runs a program as native code when it can, and native code does not make
these checks, so a pointer mistake there does what it would do in C. To see the checks,
we run the programs of Part III in the Explorer or with `zakac run --vm`.

A search for `စိတ်မချရ` finds each unsafe place in a program. The program start block
itself cannot be unsafe, so we put an unsafe block inside it, as above. Outside unsafe
code, pointer work is the error ZK0468:

```zaka-error ZK0468
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။
ကိန်းရှင် ညွှန်း သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အသက် ၏ လိပ်စာ ဖြစ်ပါစေ။
```

```text
error[ZK0468]: "၏ လိပ်စာ" needs unsafe code
  = help: put it in a block "အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။", or in an unsafe function, declared with "စိတ်မချရသော"
```

These need unsafe code:

- taking an address with `၏ လိပ်စာ`, reading or writing through a pointer with
  `၏ တန်ဖိုး`, and pointer arithmetic
- the heap statements of the next pages, and conversions to and from pointers
- a call of an unsafe function, which includes each C function

A pointer value itself can pass through safe code: we can keep it in a variable or a
field, duplicate (copy) it, print it, and compare it with the null address.

## Pointer kinds

```text
<kind> လိပ်စာ
<kind> လိပ်စာ ၏ လိပ်စာ
```

To write the data kind (type) of a pointer, we put `လိပ်စာ` after the data kind of the
value it points at. For a pointer to a pointer, we add one `၏ လိပ်စာ` ("address of") for
each further level:

| Data kind | Has the address of |
| --- | --- |
| `ကိန်းပြည့် လိပ်စာ` | an integer |
| `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ` | an unsigned 8-bit integer |
| `ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ` | a pointer to an integer |
| `လိပ်စာ` | a value of some data kind that the pointer does not tell (an *opaque* address) |

A pointer accepts addresses of the data kind in its declaration alone, so the address of a `ကိန်းပြည့်`
does not fit a pointer to an 8-bit integer:

```zaka-error ZK0401
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ၈ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
error[ZK0401]: expected i8*, found isize*
```

A bare `လိပ်စာ` can point at a value of each data kind, but we cannot read through it
until we convert it to a pointer of the data kind that is there, which the page on
[pointer arithmetic](arithmetic.md#reading-the-same-octets-as-another-data-kind) shows.

## The null address

```text
မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ
```

A pointer that points at nothing has the null address (null pointer), which we write as
`မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` ("an address that points at nothing"). It prints as that
phrase, and it is the [initial value](../variables/README.md#the-initial-value) (default
value) of each pointer kind. A pointer is not a true/false value, so to test it we
compare it with the null address, and neither step needs unsafe code:

```zaka
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။ // no address so far
ပ ကို ပြသမည်။
အကယ်၍ ပ သည် မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ နှင့် ညီလျှင် "ဘာမှ မညွှန်ပါ" ဟု ပြသမည်။
```

```text
မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ
ဘာမှ မညွှန်ပါ
```

The null address is the one NULL in Zaka, because text, objects and function values
never have one. Reading or writing through it stops the program, as [Pointer
mistakes](mistakes.md#reading-through-a-null-pointer) shows.

## Short forms

```text
&<variable>
*<pointer>
```

Programmers who know C can write `&` for "the address of" and `*` for "the value at":

```zaka
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် &က ဖြစ်ပါစေ။ // the same as က ၏ လိပ်စာ
*ပ ကို *ပ + ၁ ဟု ထားမည်။                              // the same as ပ ၏ တန်ဖိုး
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
က ကို ပြသမည်။
```

```text
၆
```

Both spellings mean the same thing, but the word forms and the short forms group in a
different order when an operator comes next to them, and the page on [pointer
arithmetic](arithmetic.md#which-value-is-read) explains the difference.

## What has an address

Variables, parameters, fields, cells and each value that we reach through a pointer have
addresses, while a number such as `၅` and the result of a sum do not. A constant has a
place in RAM, but Zaka does not give out its address, because a write through it would
change the constant, and the variable of a counted loop is treated the same:

```zaka-error ZK0412
ကိန်းသေ ကြီးဆုံး သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀၀ ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကြီးဆုံး ၏ လိပ်စာ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
error[ZK0412]: "ကြီးဆုံး" cannot be changed, so its address cannot be taken
```

To change the caller's variable from a function, safe code needs no address, because a
[mutable parameter](../functions/parameters.md#changing-the-callers-variable) (`inout`)
does that.

## Details

- A pointer declared with no value at all has nothing to read until the program sets it,
  the same as each other variable ([The null
  address](../../manual/language/pointers.md#the-null-address)).
- Each function declared inside an unsafe function is unsafe too, and a call of an unsafe
  function needs unsafe code ([Unsafe
  code](../../manual/language/unsafe.md#unsafe-code)).
- The full list of what needs unsafe code, and what can remain in safe code, is in [What
  needs unsafe code](../../manual/language/unsafe.md#what-needs-unsafe-code).
- The rules on which values have an address, with the codes ZK0411 and ZK0412, are in
  [Taking an address](../../manual/language/pointers.md#taking-an-address).
- To see addresses, we can run a program in the Explorer with debug mode on. Its heap
  pane shows a block for each global and for each local of each call in progress, and the
  Block column next to each variable gives the address of its block ([Addresses and
  blocks](../../manual/vm/blocks.md)).
- A lesson or a project can refuse unsafe code altogether, which the page on [owned
  pointers](ownership.md#keeping-to-safe-zaka) shows.

## Recap

- We put pointer work inside `အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။`, an unsafe
  block, and a search for `စိတ်မချရ` finds each one.
- We write `<variable> ၏ လိပ်စာ` (or `&`) to take an address and `<pointer> ၏ တန်ဖိုး`
  (or `*`) to read or write the value there.
- The data kind of a pointer is the data kind of its value with `လိပ်စာ` after it.
- The null address, `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`, is the initial value of each pointer,
  and we compare a pointer with it to test it.
- The VM checks each access through a pointer, so we run these programs in the Explorer or
  with `zakac run --vm`.

## Hands on

- Declare two integers and one pointer. Point at the first and change it through the
  pointer, then point at the second and change that one.
- Move the line `ညွှန်း ၏ တန်ဖိုး ကို ပြသမည်။` out of the unsafe block, and read the
  message.
- Declare a pointer to a pointer, `ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ`, that points at a
  `ကိန်းပြည့် လိပ်စာ` variable, and print `၏ တန်ဖိုး ၏ တန်ဖိုး` through it.

Next: [Calling C](../functions/c.md).
