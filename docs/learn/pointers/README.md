---
title: Unsafe Zaka
description: Part III of Learn - mark code as unchecked with စိတ်မချရသော, then hold the address of a variable and read and change the variable through it.
---

# Unsafe Zaka

This page starts Part III of Learn, an advanced part. Parts I and II need no pointer and
free nothing in code, and most programs never leave them. Part III adds *pointers*: values
that hold the address of a place in RAM, and blocks of the heap that the program asks for
and gives back itself. That is the work of C, and of code that calls C.

The compiler cannot prove such code safe, so the code tells the reader: it sits in *marked code*.
The mark is `စိတ်မချရသော` ("not to be trusted"). A block that starts with
`အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။` ("will do the following, though it is not
to be trusted") is marked, and so is a function whose header starts with the word.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။          // age = 18
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ညွှန်း သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အသက် ၏ လိပ်စာ ဖြစ်ပါစေ။ // a pointer to age
ညွှန်း ၏ တန်ဖိုး ကို ပြသမည်။                                 // read through it
ညွှန်း ၏ တန်ဖိုး ကို ၂၀ ဟု ထားမည်။                              // write through it
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အသက် ကို ပြသမည်။                                            // age has changed
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၈
၂၀
```

After the first line, the program never assigns to `အသက်` with its name. It changes the
variable through the pointer `ညွှန်း` ("the pointer").

| Zaka | Reads as |
| --- | --- |
| `အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။` | "will do the following, though it is not to be trusted" |
| `ကိန်းပြည့် လိပ်စာ` | "integer address": the kind of a pointer to an integer |
| `အသက် ၏ လိပ်စာ` | "the address of age" |
| `ညွှန်း ၏ တန်ဖိုး` | "the value at the pointer" |

## What unsafe means

Unsafe means that *the compiler* stops proving each pointer right; it never means that
the rules are off. The Zaka VM still checks each read and write through a pointer, and stops
the program at the first bad one, as [Pointer mistakes](mistakes.md) shows. Inside marked
code, the compiler reports the pointer mistakes it can see as warnings, and the program
runs.

A search for `စိတ်မချရ` finds each marked place of a program. The start block itself cannot
be marked, so a marked block goes inside it, as above. Outside marked code, the work of
pointers is an error (ZK0468):

```zaka-error ZK0468
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။
ကိန်းရှင် ညွှန်း သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အသက် ၏ လိပ်စာ ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0468]: "၏ လိပ်စာ" needs marked code
  = help: put it in a block "အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။", or in a function marked "စိတ်မချရသော"
```

These need marked code:

- `x ၏ လိပ်စာ`, reading or writing through a pointer with `၏ တန်ဖိုး`, and pointer
  arithmetic;
- the heap statements of the next pages, and conversions to and from pointers;
- a call of a marked function, or of a C function that takes or gives addresses.

A pointer value itself can pass through safe code: held in a variable or a field, copied,
printed, and compared with the null address.

## Pointer kinds

A pointer kind is the kind of the value it points to, with `လိပ်စာ` after it. A pointer
to a pointer adds `၏ လိပ်စာ` ("address of") for each further level:

| Kind | Holds the address of |
| --- | --- |
| `ကိန်းပြည့် လိပ်စာ` | an integer |
| `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ` | an unsigned 8-bit integer |
| `ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ` | a pointer to an integer |
| `လိပ်စာ` | a value of whatever kind (an *opaque* address) |

A pointer holds the addresses of its own kind alone. The address of a `ကိန်းပြည့်` does not
fit a pointer to an 8-bit integer:

```zaka-error ZK0401
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ၈ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0401]: expected i8*, found isize*
```

A bare `လိပ်စာ` holds the address of a value of whatever kind, but the program cannot read
through it until it states what kind of value is there. The page on [pointer
arithmetic](arithmetic.md) shows how.

## The null address

A pointer that points nowhere holds the null address, `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` ("an address that points at
nothing"). It prints as that phrase, and it is the [initial
value](../variables/README.md#the-initial-value) of each pointer kind. It is the one NULL
in Zaka: text, objects and function values never hold one. A pointer is not true or false,
so compare it with the null address to test it. Neither needs marked code:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။ // no address so far
ပ ကို ပြသမည်။
အကယ်၍ ပ သည် မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ နှင့် ညီလျှင် "ဘာမှ မညွှန်ပါ" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ
ဘာမှ မညွှန်ပါ
```

A pointer declared with no value at all holds nothing to read until the program sets it,
as each variable does.

Reading or writing through a null pointer stops the program. See [Pointer
mistakes](mistakes.md).

## Short forms

Programmers who know C can write `&` for "the address of" and `*` for "the value at":

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် &က ဖြစ်ပါစေ။ // the same as က ၏ လိပ်စာ
*ပ ကို *ပ + ၁ ဟု ထားမည်။                              // the same as ပ ၏ တန်ဖိုး
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
က ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၆
```

Both spellings mean the same thing, but the word forms and the short forms group in a
different order. The page on [pointer arithmetic](arithmetic.md) explains the
difference.

## What has an address

Variables, parameters, fields, cells and each value reached through a pointer have
addresses. A number such as `၅` does not, and neither does the result of a sum. A constant
has an address in RAM, but Zaka will not give it out, because writing through it would
change the constant. The same holds for the variable of a counted loop:

```zaka-error ZK0412
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ ကြီးဆုံး သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀၀ ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကြီးဆုံး ၏ လိပ်စာ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0412]: "ကြီးဆုံး" cannot be changed, so its address cannot be taken
```

To change the caller's variable from a function, safe code needs no address: a
[`ဒေတာ ပြောင်းလဲရန်` parameter](../functions/parameters.md#changing-the-callers-variable) does
it. To see the addresses, run a program in the Explorer with debug mode on. Its heap pane
shows a block for each global and for each local of each call in progress, and the Block
column next to each variable gives the address of its block.

## Hands on

- Declare two integers and one pointer. Point at the first, change it through the
  pointer, then point at the second and change that one.
- Move the line `ညွှန်း ၏ တန်ဖိုး ကို ပြသမည်။` out of the marked block, and read the message.
- Declare a pointer to a pointer, `ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ`, that points at a
  `ကိန်းပြည့် လိပ်စာ` variable, and print `၏ တန်ဖိုး ၏ တန်ဖိုး` through it.

Next: [The heap](heap.md).
