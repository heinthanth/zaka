---
title: C functions and addresses
description: Pass an address to a C function, read the text that C returns, and lend C a closure for one call.
---

# C functions and addresses

[Calling C](../functions/c.md) declares C functions that use numbers, structs of numbers,
text, lists of numbers and function values. This page adds the C functions that accept or
return an address (pointer), which points into the storage of the program, and the text
that C returns.

## You will learn

- how to pass the address of a variable to a C function, so that C can write a second
  result into it
- how text from C arrives as an optional value, and when Zaka frees it
- how to lend a closure to a C function for the length of one call
- what C can still break, since the VM cannot see inside it

A C function that returns more than one value often accepts the address of a place to
write the rest. `frexp` splits a number into a fraction and a power of two: it returns the
fraction and writes the power through its `int *` parameter.

```zaka
// frexp(x, &e): x is the fraction times two to the power e
frexp ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libm" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
x ဟူသော ဒဿမကိန်း တစ်ခု ကို လည်းကောင်း
e ဟူသော ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း လက်ခံပြီး ဒဿမကိန်း တစ်ခု ကို ပြန်ပေးမည်။

ကိန်းရှင် ထပ်ကိန်း သည် ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
frexp ဟူသော လုပ်ဆောင်ချက်ကို ၈.၀ နှင့် ထပ်ကိန်း ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ထပ်ကိန်း ကို ပြသမည်။
```

```text
၀.၅
၄
```

8.0 is 0.5 times 2 to the power 4, so the call returns `၀.၅`, and `ထပ်ကိန်း` (the power)
is `၄` after it. To declare a C parameter that accepts an address, we give it a pointer
kind, and in the call we pass the address of a variable:

```text
<name> ဟူသော <kind> လိပ်စာ တစ်ခု ကို
<variable> ၏ လိပ်စာ
```

| Zaka | Reads as |
| --- | --- |
| `e ဟူသော ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို` | "a 32-bit integer address called `e`": C's `int *e` |
| `ထပ်ကိန်း ၏ လိပ်စာ` | "the address of the power" |

Each C function must be declared with `စိတ်မချရသော`, and its calls go in unsafe code,
because Zaka cannot check what C does. A C declaration without the word is an error:

```zaka-error ZK0472
frexp ဟူသော လုပ်ဆောင်ချက် သည် "libm" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
x ဟူသော ဒဿမကိန်း တစ်ခု ကို လည်းကောင်း e ဟူသော ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း လက်ခံပြီး ဒဿမကိန်း တစ်ခု ကို ပြန်ပေးမည်။
```

```text
error[ZK0472]: "frexp" is a C function, so its declaration must be unsafe, with "စိတ်မချရသော"
```

## Text that C returns

```text
စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်
```

C returns NULL when it has no text, so we declare a text result as an
[optional](../optionals/README.md) (option type), and NULL arrives as the blank optional
`အလွတ်`. Zaka duplicates the text into new text of the program, while C keeps what it
returned, so the program never sees the address. `strstr` finds a part in a text and
returns the rest of the text from there, or NULL when the part is not in it:

```zaka
strstr ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော စာသား တစ်ခု ကို လည်းကောင်း
part ဟူသော စာသား တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။

အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
strstr ဟူသော လုပ်ဆောင်ချက်ကို "မင်္ဂလာပါ ကမ္ဘာ" နှင့် "ကမ္ဘာ" တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို
ပြသမည်။
strstr ဟူသော လုပ်ဆောင်ချက်ကို "abc" နှင့် "x" တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
ဒေတာ("ကမ္ဘာ")
အလွတ်
```

## Text that C hands over

```text
စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်
```

`strstr` returns a part of a string that C keeps, but `strdup` makes a new string and
hands it over to the caller. To declare that, we write `အပိုင်အနေဖြင့်` ("as its own")
before `ပြန်ပေးမည်`, the same as for an [owned result](ownership.md#owned-results-and-parameters),
and Zaka then frees the string with C's `free` once it has the text. Windows names the
function `_strdup`, so this program picks the name at compile time:

```zaka
// Windows names it _strdup
အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် အလုပ်လုပ်မည့်ကွန်ပျူတာစနစ် == "windows" ဖြစ်လျှင်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
_strdup ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။
strdup ဟူသော လုပ်ဆောင်ချက် သည်
s ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
_strdup ဟူသော လုပ်ဆောင်ချက်ကို s ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
သို့မဟုတ်ပါက အောက်ပါတို့ကို လုပ်ဆောင်မည်။
strdup ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
strdup ဟူသော လုပ်ဆောင်ချက်ကို "စံကား" ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
ဒေတာ("စံကား")
```

When we leave the word out for a string that C hands over, the string is never freed,
and when we write it for a string that C keeps, the program frees something that is not
its own. On Windows, `strdup` here is a safe wrapper, a plain function with the unsafe
call inside, which gives the rest of the program the result in safe code, as on [Owned
pointers](ownership.md#a-safe-wrapper).

## A closure for a call

A C function that accepts a function, as `qsort` accepts the function that compares two
cells, calls it while the call runs. We can pass a
[closure](../function-values/closures.md) there, and the call needs nothing more, because
the closure is lent to C for that one call. This closure counts its calls in a variable
around it:

```zaka
qsort ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
base ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လည်းကောင်း
count ဟူသော လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
size ဟူသော လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
compare ဟူသော (ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း
လက်ခံပြီး
၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို လည်းကောင်း လက်ခံမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် နှိုင်းကြိမ် သည် ၀ ဖြစ်ပါစေ။
နှိုင်း ဟူသော လုပ်ဆောင်ချက် သည်
a ဟူသော ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း
b ဟူသော ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
နှိုင်းကြိမ် ကို နှိုင်းကြိမ် + ၁ ဟု ထားမည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
အကယ်၍ (a ၏ တန်ဖိုး) < (b ၏ တန်ဖိုး) ဖြစ်လျှင် -၁ ကို ပြန်ပေးမည်။
အကယ်၍ (a ၏ တန်ဖိုး) > (b ၏ တန်ဖိုး) ဖြစ်လျှင် ၁ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
၀ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ကိန်းရှင် ကိန်းများ သည် [၅၊ ၃၊ ၉၊ ၁] ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
qsort ဟူသော လုပ်ဆောင်ချက်ကို ကိန်းများ၊ ၄၊
၈ နှင့် နှိုင်း ဟူသော လုပ်ဆောင်ချက် တို့ဖြင့် လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ကိန်းများ ကို ပြသမည်။
နှိုင်းကြိမ် > ၀ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၃၊ ၅၊ ၉]
မှန်
```

The program prints whether `qsort` called the function (`မှန်`) and not the count of
its calls, because that count depends on the C lib (library).

## Details

- A run that calls C needs the FFI right, so from a terminal we run it with `zakac run --allow-ffi`
  ([Permissions](../../manual/vm/running.md#permissions)).
- C does what C does. A C function that writes past the place it was given harms the run
  as it would harm a C program, and the VM cannot see inside it to stop it.
- A C function that keeps a function for a later call needs our leave to call it, which
  [Callbacks and handles](../functions/c-callbacks.md#a-function-c-keeps) shows.
- [Calling C](../functions/c.md#c-writes-into-a-list) gets a second result from C with no
  address in the program, through a `ဒေတာ ထည့်ပေးရန်` list of one cell.
- The full rules for C parameters and results, addresses included, are in [C
  functions](../../manual/language/functions-c.md#c-functions).

## Recap

- We declare a C parameter with a pointer kind to accept an address, and pass
  `<variable> ၏ လိပ်စာ` in the call.
- Each C function is declared `စိတ်မချရသော`, and its calls go in unsafe code.
- Text from C arrives as an optional value, with `အလွတ်` for NULL, and `အပိုင်အနေဖြင့်` on
  the result tells Zaka to free a string that C hands over.
- A closure can go to a C function for the length of one call with nothing more.

## Hands on

- Declare `modf` from `"libm"`, which splits a number into its whole and fractional parts
  and writes the whole part through a `double *`. Print both parts of 3.25.
- Call `strstr` and print the text it finds, or `""` when C returns the blank optional.

Next: [Pointer mistakes](mistakes.md).
