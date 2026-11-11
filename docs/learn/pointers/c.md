---
title: C functions and addresses
description: Mark a C function that takes or gives an address, call it from marked code, and read the text that C gives back.
---

# C functions and addresses

[Calling C](../functions/c.md) declares C functions that safe code can call: their
parameters are numbers, structs of numbers, text, lists of numbers and function
values, and their results are numbers or plain structs. A C function outside that is
marked `စိတ်မချရသော`, and its calls go in marked code: a parameter or a result that holds a
pointer. Text that C gives back needs no mark, since Zaka copies it
([below](#text-that-c-gives-back)).

## Writing through an address

A C function that gives back more than one value takes the address of a place to write
the rest. `frexp` splits a number into a fraction and a power of two: it gives the
fraction back and writes the power through its `int *`.

```zaka
// frexp(x, &e): x is the fraction times two to the power e
စိတ်မချရသော လုပ်ဆောင်ချက် frexp သည် "libm" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
x ဟူသော ဒဿမကိန်း တစ်ခု ကို လည်းကောင်း e ဟူသော ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း လက်ခံပြီး ဒဿမကိန်း တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ထပ်ကိန်း သည် ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် frexp ကို ၈.၀၊ ထပ်ကိန်း ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ထပ်ကိန်း ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၀.၅
၄
```

8.0 is 0.5 times 2 to the power 4, so the call gives `၀.၅` and `ထပ်ကိန်း` holds `၄` after
it. [Calling C](../functions/c.md#c-writes-into-a-list) does the same with no mark, through
a `ဒေတာ ထည့်ပေးရန်` list of one cell. A declaration with a pointer and no mark is an error:

```zaka-error ZK0472
လုပ်ဆောင်ချက် frexp သည် "libm" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
x ဟူသော ဒဿမကိန်း တစ်ခု ကို လည်းကောင်း e ဟူသော ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း လက်ခံပြီး ဒဿမကိန်း တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0472]: "frexp" passes a pointer, so its declaration needs the mark "စိတ်မချရသော"
```

## Text that C gives back

Text comes back as an [optional](../optionals/README.md): C gives NULL when it has no
text, and that is the blank optional `အလွတ်`. Zaka copies the text into a new one of its
own, and C keeps what it gave. The declaration needs no mark, and neither does its call,
since the program never holds the address. `strstr` finds a part in a text and gives the
rest, or NULL when the part is absent:

```zaka
လုပ်ဆောင်ချက် strstr သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော စာသား တစ်ခု ကို လည်းကောင်း part ဟူသော စာသား တစ်ခု ကို လည်းကောင်း လက်ခံပြီး စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် strstr ကို "မင်္ဂလာပါ ကမ္ဘာ"၊ "ကမ္ဘာ" တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်ချက် strstr ကို "abc"၊ "x" တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဒေတာ("ကမ္ဘာ")
အလွတ်
```

`strstr` gives a part of a string that C keeps. `strdup` makes a new string and hands it
over, so its header writes `အပိုင်အနေဖြင့်` ("as its own") before `ပြန်ပေးမည်`, and Zaka frees
the string with C's `free` once it holds the text:

```zaka
လုပ်ဆောင်ချက် strdup သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် strdup ကို "စံကား" တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဒေတာ("စံကား")
```

Leave the word out for a string C hands over, and the string is never freed; write it for
a string C keeps, and the program frees what is not its own. A function that also takes a
pointer is marked, and a safe wrapper, a plain function with the marked call inside, gives
the rest of the program the result with no mark, as on
[Owned pointers](ownership.md#a-safe-wrapper).

## A closure for a call

A C function that takes a function, as `qsort` takes the function that compares two cells,
calls it while the call runs. A [closure](../function-values/closures.md) can stand there,
and the call needs no more: the closure is lent to it. This one counts its calls in a
variable around it:

```zaka
လုပ်ဆောင်ချက် qsort သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
base ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လည်းကောင်း count ဟူသော လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
size ဟူသော လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
compare ဟူသော (ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို လည်းကောင်း လက်ခံမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် နှိုင်းကြိမ် သည် ၀ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် နှိုင်း သည် a ဟူသော ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း b ဟူသော ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
နှိုင်းကြိမ် ကို နှိုင်းကြိမ် + ၁ ဟု ထားမည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
အကယ်၍ (a ၏ တန်ဖိုး) < (b ၏ တန်ဖိုး) ဖြစ်လျှင် -၁ ကို ပြန်ပေးမည်။
အကယ်၍ (a ၏ တန်ဖိုး) > (b ၏ တန်ဖိုး) ဖြစ်လျှင် ၁ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
၀ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ကိန်းရှင် ကိန်းများ သည် [၅၊ ၃၊ ၉၊ ၁] ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် qsort ကို ကိန်းများ၊ ၄၊ ၈၊ နှိုင်း ဟူသော လုပ်ဆောင်ချက် တို့ဖြင့် လုပ်ဆောင်မည်။
ကိန်းများ ကို ပြသမည်။
နှိုင်းကြိမ် > ၀ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၃၊ ၅၊ ၉]
မှန်
```

The program prints whether `qsort` called the function (`မှန်`), not the count of its
calls, since that count belongs to the C lib. A C function that keeps the function for a
later call needs a leave, which [Calling C](../functions/c.md#a-function-c-keeps) shows.

## What C does

C does what C does. A C function that writes past the place it was given harms the run as
it would harm a C program, and the VM cannot see inside it. A closure crosses to C as a
function value does, and a C function that keeps it needs leave to call it later (see
[a closure for a call](#a-closure-for-a-call)).

## Hands on

- Declare `modf` from `"libm"`, which splits a number into its whole and fractional parts
  and writes the whole part through a `double *`. Print both parts of 3.25.
- Call `strstr` and print the text it finds, or `""` when C gives the blank optional.

Next: [Pointer mistakes](mistakes.md).
