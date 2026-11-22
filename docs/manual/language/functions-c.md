---
title: External and C functions
description: Functions declared without statements, which the VM serves or a C lib provides, the data kinds that cross to C, callbacks, variadic C functions, and handles to the structs of a C lib.
---

# External and C functions

Some functions come from outside the program, so their function signature has no
statements after it. An *external function* is one that the VM serves, such as the
functions of input and output in the stdlib, and a *C function* is a function of a shared
lib (library) that the program calls through the C calling convention (FFI). This page
covers both, the data kinds (types) that can cross to C, callbacks from C into the
program, variadic C functions, and handles to the structs that a C lib keeps. The Learn
pages [Calling C](../../learn/functions/c.md) and
[Callbacks and handles](../../learn/functions/c-callbacks.md) teach the same ideas with
examples, and functions that C calls in the other direction are on
[Functions C can call](functions-exports.md).

## External functions

To declare a function that comes from outside the program, we write `ပြင်ပမှလာသော`
("that comes from outside") after `သည်`. The function signature gives the parameters and
the result, and no statements follow it, since the VM does the work while the program runs.
The function signature is one sentence:

| Parameters | Result | Written |
| --- | --- | --- |
| none | none | `<name> ဟူသော လုပ်ဆောင်ချက် သည် ပြင်ပမှလာသော လုပ်ဆောင်ချက်ဖြစ်သည်။` |
| some | none | `... လုပ်ဆောင်ချက်ဖြစ်၍ <parameters> လက်ခံမည်။` |
| none | a result | `... လုပ်ဆောင်ချက်ဖြစ်၍ <kind> တစ်ခု ကို ပြန်ပေးမည်။` |
| some | a result | `... လုပ်ဆောင်ချက်ဖြစ်၍ <parameters> လက်ခံပြီး <kind> တစ်ခု ကို ပြန်ပေးမည်။` |

`လုပ်ဆောင်ချက်` can have `တစ်ခု` after it (`လုပ်ဆောင်ချက်တစ်ခုဖြစ်၍`), and we can also
write `ပြင်ပမှလာသော` apart (`ပြင်ပ မှ လာသော`). The parameters, prepositions included, and
the result kind follow the rules of a regular function signature. A call of an external
function is a call like others, in each call form.

The VM serves a fixed set of functions. An external declaration must match the name,
parameter kinds and result kind of one of them. A different one is ZK0334, and its help
lists the set. The stdlib declares each of these functions, so a program imports them from
there instead of writing the function signatures.

```zaka-error ZK0334
ဟိုဟို ဟူသော လုပ်ဆောင်ချက် သည် ပြင်ပမှလာသော လုပ်ဆောင်ချက်ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0334]: the VM serves no function "ဟိုဟို" with these parameter and result kinds
```

## C functions

To declare a C function, we write the name of a shared lib as text in place of `ပြင်ပ`, as
in `"libm" မှလာသော`. The function is the C function with the same name in that lib, so its
name must be a C name: ASCII letters, digits and `_`, not starting with a digit (ZK0337).
Each C function names its lib, libc too (`"libc"`). The function signature has the forms
of the table above, with the prepositions of its parameters, and starts with
`စိတ်မချရသော`, because each C function is unsafe ([below](#unsafe-c-functions)). So a call
of it, which is otherwise a call like others, goes in unsafe code.

```zaka
abs ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
n ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
abs ဟူသော လုပ်ဆောင်ချက်ကို -၄၂ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၂
```

C receives each parameter and returns its result as a whole value, laid out as in C, so
the data kinds that cross are those that C has:

| Data kind | In C |
| --- | --- |
| integers of 8 to 64 bits, `ကိန်းပြည့်` | `int8_t` to `int64_t`, `uint8_t` to `uint64_t` |
| `၃၂ ဘစ် ဒဿမကိန်း`, `ဒဿမကိန်း` | `float`, `double` |
| `မှန်မှားတန်ဖိုး` | `_Bool` |
| a struct whose fields are these data kinds, structs or counted lists of them | a struct, passed whole |
| `စာသား`, as a parameter | `const char *` ([Text](#text)) |
| a plain or counted list of numbers, as a parameter | the address of its first cell ([Lists of numbers](#lists-of-numbers)) |
| a function kind whose parameters and result cross, as a parameter | a pointer to a function ([Callbacks](#callbacks)) |
| a data kind from outside the program, as a parameter or a result | a pointer to a struct that C keeps ([Data kinds from outside the program](#data-kinds-from-outside-the-program)) |
| `<kind from outside> ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`, as a result | a pointer to a struct that C keeps, NULL for the optional with no data |
| a pointer, `<kind> လိပ်စာ`, or a struct with a pointer in it | a pointer |
| `စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`, as a result | `const char *`, NULL for the optional with no data ([Text as a result](#text-as-a-result)) |

Each other data kind is ZK0465. That covers plain lists of other cells, which C has no form
for; shared objects; function values that do not cross ([Callbacks](#callbacks)); text as a
bare result, since C can return NULL for it, and a handle in the same case
([Text as a result](#text-as-a-result)); and other results that can fail.

```zaka-error ZK0465
getenv ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
n ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး စာသား တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0465]: a C function cannot take or give back a value of the kind "text"
```

Each declaration of one C name, in each file and among overloads, gives it the same
parameter and result kinds (ZK0338). Like each function that comes from outside, a C
function cannot be a function value (ZK0464), so we write a function of the program that
calls it and use that function in its place.

## Unsafe C functions

C code runs outside the checks of Zaka, and it can read and write storage that Zaka does
not see. So each C function is declared unsafe with `စိတ်မချရသော`, as a function of the
program can be ([Unsafe Zaka](unsafe.md)), whatever its data kinds, even when it works on
numbers alone. Without the word, the declaration is ZK0472. Each call of a C function then
needs unsafe code: an unsafe function, or an unsafe block opened with `စိတ်မချရသော်လည်း`
(ZK0468 elsewhere). A safe function can wrap the call in an unsafe block, so the rest of
the program calls the wrapper from safe code.

```zaka-error ZK0472
frexp ဟူသော လုပ်ဆောင်ချက် သည် "libm" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
x ဟူသော ဒဿမကိန်း တစ်ခု ကို လည်းကောင်း e ဟူသော ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း လက်ခံပြီး ဒဿမကိန်း တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0472]: "frexp" is a C function, so its declaration must be unsafe, with "စိတ်မချရသော"
```

```zaka-error ZK0468
frexp ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libm" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
x ဟူသော ဒဿမကိန်း တစ်ခု ကို လည်းကောင်း e ဟူသော ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း လက်ခံပြီး ဒဿမကိန်း တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ထပ်ကိန်း သည် ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
frexp ဟူသော လုပ်ဆောင်ချက်ကို ၈.၀ နှင့် ထပ်ကိန်း ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0468]: a call of "frexp", an unsafe function, needs unsafe code
```

## Text

C reads a string up to a zero octet, which Zaka text does not have, so a text parameter
reaches C as the address of a duplicate (copy) of its u8s with a zero octet after them. The
duplicate lives until the call ends. C reads it but cannot change the text, and a zero
octet inside the text ends the string there for C.

```zaka
strlen ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
strlen ဟူသော လုပ်ဆောင်ချက်ကို "မောင်" ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၅
```

## Lists of numbers

A plain or counted list whose cells are integers of up to 64 bits, `၃၂ ဘစ် ဒဿမကိန်း` or
`ဒဿမကိန်း` reaches C as the address of its first cell, which is how C receives a run of
cells: `const T *`. The cells remain in place for the call, so C reads them there.

A list of numbers marked `ဒေတာ ပြောင်းလဲရန်` or `ဒေတာ ထည့်ပေးရန်` reaches C in the same
form, and C writes its cells in place: `T *`. A plain list first makes its cells its own,
so no duplicate of the list sees what C writes. This is how C returns more than one value.
A struct that crosses to C, marked `ဒေတာ ပြောင်းလဲရန်`, reaches C as its address, `T *`,
and C writes its fields in place, as a C function that fills a struct from its caller does.
`ဒေတာ ပြောင်းလဲရန်` or `ဒေတာ ထည့်ပေးရန်` on a parameter of another data kind is ZK0466,
because C receives each other value whole, so it has no place to write back to.

We mark a place that C reads, or reads and writes, as a
[mutable parameter](functions-parameters.md#mutable-parameters) with `ဒေတာ ပြောင်းလဲရန်`,
as `memset` does with its list, and the caller sets it first. We mark a place that C
writes and never reads as a [writable parameter](functions-parameters.md#writable-parameters)
with `ဒေတာ ထည့်ပေးရန်`, as `frexp` does with its exponent: the caller declares it with no
value, and it counts as set after the call. The compiler cannot see inside C, so the
marker is the author's promise that C writes the whole place.

```zaka
strlen ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ကို လက်ခံပြီး လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို
ပြန်ပေးမည်။
memset ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
b ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လည်းကောင်း
c ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
n ဟူသော လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံမည်။
frexp ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libm" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
x ဟူသော ဒဿမကိန်း တစ်ခု ကို လည်းကောင်း
e ဟူသော အကွက် ၁ ခုရှိသော ၃၂ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ထည့်ပေးရန် လည်းကောင်း လက်ခံပြီး
ဒဿမကိန်း တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၀၄၊ ၁၀၅၊ ၀] ဖြစ်ပါစေ။
ကိန်းရှင် ပွား သည် စာ ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
strlen ဟူသော လုပ်ဆောင်ချက်ကို စာ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
memset ဟူသော လုပ်ဆောင်ချက်ကို စာ၊ ၇ နှင့် ၂ တို့ဖြင့် လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
// ပွား keeps the cells it had
"{စာ} {ပွား}" ဟု ပြသမည်။
ကိန်းရှင် ထပ်ကိန်း သည် အကွက် ၁ ခုရှိသော ၃၂ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
frexp ဟူသော လုပ်ဆောင်ချက်ကို ၈.၀ နှင့် ထပ်ကိန်း တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ထပ်ကိန်း ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂
[၇၊ ၇၊ ၀] [၁၀၄၊ ၁၀၅၊ ၀]
၀.၅
[၄]
```

```zaka-error ZK0466
puts ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော စာသား တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0466]: the C function cannot change the parameter "s" in place
```

C knows no count of cells and writes as far as it is told, so a C function that writes
past the last cell harms the run, as it would harm a C program.

## Text as a result

When a C function returns a string, we write its result as
`စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု` (an [optional](optionals.md)), and its
declaration needs no `စိတ်မချရသော`, because the program gets a duplicate and never the
address. NULL is the optional with no data, `အလွတ်`, and each other address is the text, as
its data. Zaka copies the string up to its zero octet into new text. The old form,
`စာသား သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ်`, is ZK0465, and the help line of
that error names the optional. Octets that are not UTF-8 stop the program with the
[not UTF-8 trap](../vm/traps.md).

The result clause decides who frees the string of C:

| Result clause | C's string | For |
| --- | --- | --- |
| `... ကို ပြန်ပေးမည်` | C keeps it, and Zaka never frees it | `getenv`, `strerror`, `strstr`, static buffers |
| `... ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်` | C hands it over, and Zaka frees it with the `free` of libc once its text has the octets | `strdup` and each function that allocates the string it returns |

`အပိုင်အနေဖြင့်` is allowed on a C function whose result is an optional of text, and on
each other C function it is ZK0467. Without it, a string that C allocates is never freed.

Zaka copies the text before it frees the duplicates of the text arguments, so a result that
points into an argument, as the result of `strstr` does, is copied while it is still there.

```zaka
strstr ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော စာသား တစ်ခု ကို လည်းကောင်း
part ဟူသော စာသား တစ်ခု ကို လည်းကောင်း လက်ခံပြီး စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို
ပြန်ပေးမည်။
// strdup hands its string over
// Windows names it _strdup
အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် အလုပ်လုပ်မည့်ကွန်ပျူတာစနစ် == "windows" ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
_strdup ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို အပိုင်အနေဖြင့်
ပြန်ပေးမည်။
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
s ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို အပိုင်အနေဖြင့်
ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
strstr ဟူသော လုပ်ဆောင်ချက်ကို "မင်္ဂလာပါ ကမ္ဘာ" နှင့် "ကမ္ဘာ" တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
strstr ဟူသော လုပ်ဆောင်ချက်ကို "abc" နှင့် "x" တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
strdup ဟူသော လုပ်ဆောင်ချက်ကို "စံကား" ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဒေတာ("ကမ္ဘာ")
အလွတ်
ဒေတာ("စံကား")
```

A program can also receive the string as a pointer and free it itself, as Rust and Go
programs do: read its length with `strlen`, bring its octets into a list of the program
with `memcpy`, free it with `free`, and read the cells as text. The VM cannot read through
an address that C gives, so `memcpy`, a C function, copies the octets. A range of the list
gives a plain list of u8s, which converts to text after a check of its UTF-8
([Text](text.md)).

```zaka
// Windows names it _strdup
အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် အလုပ်လုပ်မည့်ကွန်ပျူတာစနစ် == "windows" ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
_strdup ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို ပြန်ပေးမည်။
strdup ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည်
s ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို ပြန်ပေးမည်။
_strdup ဟူသော လုပ်ဆောင်ချက်ကို s ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
သို့မဟုတ်ပါက အောက်ပါတို့ကို လုပ်ဆောင်မည်။
strdup ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
strlen ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လက်ခံပြီး လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို
ပြန်ပေးမည်။
memcpy ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
dst ဟူသော အကွက် ၆၄ ခုရှိသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လည်းကောင်း
src ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း
n ဟူသော လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု
ကို ပြန်ပေးမည်။
free ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
p ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လက်ခံမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် strdup ဟူသော လုပ်ဆောင်ချက်ကို "zaka" ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် ရေ သည် strlen ဟူသော လုပ်ဆောင်ချက်ကို ပ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် ဘောင် သည် အကွက် ၆၄ ခုရှိသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် ပြန် သည် memcpy ဟူသော လုပ်ဆောင်ချက်ကို ဘောင်၊ ပ နှင့် ရေ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
free ဟူသော လုပ်ဆောင်ချက်ကို ပ ဖြင့် လုပ်ဆောင်မည်။
(ဘောင် ၏ အကွက် ၀ မှစ၍ ရေ ကွက်) ကို စာသား တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
zaka
```

The list here has 64 octets, so a longer string needs a larger list, or a block from the
heap.

## Callbacks

A parameter of a function kind crosses to C as a pointer to a function. The result of the
function kind is an integer of up to 64 bits, a `၃၂ ဘစ် ဒဿမကိန်း`, a `ဒဿမကိန်း`, a
`မှန်မှားတန်ဖိုး`, a pointer, or a plain struct of these, which C gets as a whole value; or
the function kind returns nothing. Each parameter of the function kind is one of these, or
one of the following:

| Parameter | What C passes | What the function gets |
| --- | --- | --- |
| a plain struct | the struct, as a whole value | a struct with the same fields |
| `စာသား` | a `char *` | text copied from the string; NULL stops the program with the trap `null from C` |
| `စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` | a `char *` | an optional: `အလွတ်` for NULL, else the copied text |
| a data kind from outside | the address of a struct that C keeps | a new handle around the address; NULL stops the program with the trap `null from C`, since the parameter promised a handle |
| an optional of a data kind from outside | the address, or NULL | an optional: `အလွတ်` for NULL, else a new handle |

The function kind has no `ဒေတာ ပြောင်းလဲရန်` marker, and each other function kind is
ZK0465. The function gets each of these values lent, as a call of the program lends its
arguments: C keeps the string and the struct that it passed, and the duplicates go once the
function returns. A handle that C passed belongs to C, so the function cannot close it in
safe code: passing a handle parameter to a C function that takes it over with
`အပိုင်အနေဖြင့်` is ZK0468 outside unsafe code.

```zaka-file
// callbacks.zaka
ဘူး ဟူသော ဒေတာအစုအဝေးသည် ပြင်ပမှ လာသော ဒေတာအစုအဝေး ဖြစ်၍ ၎င်းတွင် n ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ပါဝင်သည်။
each_name ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libnames" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
f ဟူသော (စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို လက်ခံမည်။
each_box ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libnames" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
f ဟူသော (ဘူး တစ်ခု ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို လက်ခံမည်။

အမည်ပြ ဟူသော လုပ်ဆောင်ချက် သည် အမည် ဟူသော စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ အမည် ထဲတွင် ဒေတာ ရှိလျှင် အမည် ထဲမှ ဒေတာ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

The argument is a function of the program named with `ဟူသော`, or a variable that has one.
C never gets NULL for it, since a function value refers to a function at each point. A closure can
go too: it is lent to the call, so C can call it while the call runs. A C function that
keeps the function and calls it later needs the closure to have leave, which the statement
`f ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်` gives and
`f ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြန်လည်ရုပ်သိမ်းမည်` takes back. A call from C with no call
running and no leave stops the program with the trap `closure_escape`. See
[Closures and C](closures.md#closures-and-c).

```zaka
qsort ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
base ဟူသော ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း
count ဟူသော လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
size ဟူသော လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
compare ဟူသော (၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို
လည်းကောင်း လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို
လည်းကောင်း လက်ခံမည်။

// C calls it with the addresses of two cells
ငယ်စဉ် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
(က ၏ တန်ဖိုး) - (ခ ၏ တန်ဖိုး) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် အကွက် ၅ ခုရှိသော ၃၂ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၅၊ ၂၊ ၉၊ ၁၊ ၇] ဖြစ်ပါစေ။
ကိန်းရှင် အရွယ် သည် ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ၏ အရွယ်အစား ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
qsort ဟူသော လုပ်ဆောင်ချက်ကို ကိန်းများ ၏ အကွက် ၀ ၏ လိပ်စာ၊ ၅၊ အရွယ် နှင့် ငယ်စဉ် ဟူသော လုပ်ဆောင်ချက် တို့ဖြင့် လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ကိန်းများ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၂၊ ၅၊ ၇၊ ၉]
```

Each build gives C a pointer that it can call:

- `zakac run` makes one native closure for each function that a call gives C, on the
  first such call, and keeps it for the run. The callback runs the function in the VM until
  it returns.
- A native build passes the code address of a function with no captures. A closure goes
  through a native closure of libffi, which the runtime lib carries (see
  [Native builds](../compiler/native.md)), so the build needs no libffi on the host.

A trap inside a callback waits until C returns, then stops the program, and each later
callback of that C call returns 0. Text that a callback prints in the VM comes out after
the output of the C call itself, while a native build keeps the order. A callback must not
resize or free a block that C has an address into, since the native address goes stale.

## Other values

```text
... fmt ဟူသော စာသား တစ်ခု ကို လည်းကောင်း အခြားတန်ဖိုးများနှင့်အတူ လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
```

A C function whose parameters end in `...`, as `printf` does, is variadic: it accepts more
arguments than it names. To declare one, we write `အခြားတန်ဖိုးများနှင့်အတူ` ("together
with other values") before `လက်ခံပြီး`, after the last parameter it names. Each parameter
before it ends in `လည်းကောင်း`, even a lone one, and a space can go before `နှင့်အတူ`.

A call gives an argument for each parameter, then each other value it wants, and too few
arguments is ZK0314. An other value has its own data kind, so a number literal is a
`ကိန်းပြည့်` or a `ဒဿမကိန်း`. C reads it with the C promotions: an integer narrower than 32
bits and a boolean as an `int`, and a `၃၂ ဘစ် ဒဿမကိန်း` as a `double`. Text goes as a
duplicate that ends in a zero octet, a list of numbers as the address of its first cell,
and a struct goes whole, so C reads it as it reads a struct parameter. The format must
match the data kinds, as in C: `%lld` for a `ကိန်းပြည့်`, which has 64 bits (a C `long`,
which `%ld` reads, has 32 bits on Windows), and `%d` for a `၃၂ ဘစ် ကိန်းပြည့်` or a boolean.
Functions that the host serves accept no other values (ZK0334).

A C function that accepts other values can mark a list of numbers `ဒေတာ ပြောင်းလဲရန်`, so
`snprintf` writes its buffer from safe code:

```zaka
// The UCRT that the VM opens on Windows has no snprintf; msvcrt has _snprintf
အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် အလုပ်လုပ်မည့်ကွန်ပျူတာစနစ် == "windows" ဖြစ်၍ ကွန်ပိုင်လာအထွက်ကုဒ်အမျိုးအစား == "vm" ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
_snprintf ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "msvcrt" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
buf ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လည်းကောင်း
size ဟူသော လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
fmt ဟူသော စာသား တစ်ခု ကို လည်းကောင်း အခြားတန်ဖိုးများနှင့်အတူ လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို
ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
သို့မဟုတ်ပါက အောက်ပါတို့ကို လုပ်ဆောင်မည်။
snprintf ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
buf ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လည်းကောင်း
size ဟူသော လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
fmt ဟူသော စာသား တစ်ခု ကို လည်းကောင်း အခြားတန်ဖိုးများနှင့်အတူ လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို
ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဘောင် သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [] ဖြစ်ပါစေ။
၁ မှ ၃၂ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဘောင် ထဲသို့ ၀ ကို ထည့်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် အလုပ်လုပ်မည့်ကွန်ပျူတာစနစ် == "windows" ဖြစ်၍ ကွန်ပိုင်လာအထွက်ကုဒ်အမျိုးအစား == "vm" ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရေ သည် ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် _snprintf ဟူသော လုပ်ဆောင်ချက်ကို ဘောင်၊ ၃၂၊ "%s=%lld"၊ "x" နှင့် ၄၂ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
သို့မဟုတ်ပါက အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရေ သည် ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် snprintf ဟူသော လုပ်ဆောင်ချက်ကို ဘောင်၊ ၃၂၊ "%s=%lld"၊ "x" နှင့် ၄၂ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
(ဘောင် ၏ အကွက် ၀ မှစ၍ ရေ ကွက်) ကို စာသား တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
x=42
```

## Data kinds from outside the program

```text
ဒေတာဘေ့စ် ဟူသော ပြင်ပမှ လာသော [အများသုံး] ဒေတာအစုအဝေး ဖြစ်သည်။
ရက်စွဲ ဟူသော ပြင်ပမှ လာသော [အများသုံး] ဒေတာအစုအဝေးတွင် <field> ဟူသော <kind> တစ်<word>၊ … နှင့် … တို့ပါဝင်သည်။
```

Some C libs keep a struct and give the program a pointer to it: a database connection, a
counter, a window. Zaka names such a struct with a *data kind from outside* (opaque type),
which we declare with `ပြင်ပမှ လာသော ဒေတာအစုအဝေး` ("a struct that comes from outside";
`ပြင်ပမှလာသော` is the same) after `<name> ဟူသော ဒေတာအစုအဝေးသည်` (see
[Marks after the head](structs.md#marks-after-the-head)). A value of it is a *handle*. The
declaration has two forms. The first names no field, so the program can pass the handle
back to C and do nothing else with it, and the second lists the fields of the C struct
that the program reads. A lone field ends with `ပါဝင်သည်`, without `တို့`, as in a struct.

A handle is a shared box, as an object of a `မျှဝေနိုင်သော` struct is
([Values and objects](objects.md#shared-objects)), and the one field of the box is the
address that C gave. Copies made with `ဖြစ်ပါစေ` and `ဟု ထားမည်` share that box. A variable
of the data kind declared with no value is unset until the program gives it a handle, as
for each variable ([Set before read](declarations.md#set-before-read)), and a handle has no
[initial value](kinds.md#initial-values): a handle comes from C alone, so `ကနဦးတန်ဖိုး` for
one is ZK0479.

### What C functions accept and return

A C function can accept a handle as a parameter and return one, and its declaration needs
no `စိတ်မချရသော`, because the program never has the address and so cannot reach through
it. A result written `<kind> တစ်ခု ကို ပြန်ပေးမည်` wraps the address that C returns. Such a
plain result promises that C returns no NULL: when C returns NULL all the same, the run
stops with the trap [null from C](../vm/traps-unsafe.md#c-functions), since no variable can have a
NULL handle. A result written `<kind> ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်`
returns the optional with no data for NULL, and the handle, as data, for each other
address. A handle can also be one of the other values of a C function that accepts them,
such as `printf`, and C then gets the address of the handle, as for a parameter.

### Closing a handle

Zaka never closes a handle on its own. A C function closes one through a parameter marked
`အပိုင်အနေဖြင့်` ("as its own"), which means that the C function takes the handle over.
After the call the box has 0 in it, so each duplicate of the handle is seen as closed, and
the variable that the caller named shares that box, so it is closed too. A closed handle
used as an argument of a C function or through a field read stops the program with the
trap [closed handle](../vm/traps-unsafe.md#c-functions).

### Fields

A declaration of the second form lists the fields of the C struct from its start. A prefix
is enough, since the program reads just the fields it names. Each field has one of these
data kinds, and each other data kind is ZK0465:

| Data kind of the field | In C |
| --- | --- |
| an integer of 8 to 64 bits | `int8_t` to `int64_t`, `uint8_t` to `uint64_t` |
| `၃၂ ဘစ် ဒဿမကိန်း`, `ဒဿမကိန်း` | `float`, `double` |
| `မှန်မှားတန်ဖိုး` | `_Bool` |
| a counted list of numbers | a run of cells inside the struct, as `int32_t marks[2]` |
| `စာသား` | `char *` |
| a data kind from outside | a pointer to the struct that C keeps for that data kind |

C places the fields, each one at the next multiple of its alignment, and a text field or a
handle field is a pointer. The names are those of the program, while the order and the data
kinds must match the C struct.

`h ၏ <field>` reads the field from the storage of C each time, so Zaka keeps no duplicate,
and a read after C changed the struct sees the change. A counted list is read as a new list
with the cells of that moment. A text field is read as an optional of text,
`စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`: Zaka duplicates the string up to its zero octet,
and a NULL `char *` is the optional with no data. C can give `""` for no text, so the two
differ. A field of a data kind from outside is read as an optional of a new handle,
`<kind> ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`, with NULL for the optional with no data. The struct
it points at belongs to C, so a program closes it, when C asks for that, through the C
function that closes it.

Safe code writes no field: C owns the struct, so a program changes it through a C
function, and a write is ZK0474. [Unsafe code](unsafe.md) can write a field of a number, a
boolean or a counted list with `… ၏ <field> ကို … ဟု ထားမည်။`, and the value goes into the
storage of C in place, after the same check that the handle is open. A text field or a
handle field is a pointer that C owns, so no code writes it (ZK0474).

### An example

The program below declares a counter that a lib keeps. `counter_open` returns a handle, or
the optional with no data for a step of 0, `counter_next` adds one step, and
`counter_close` takes the handle over. The fields `count`, `step` and `label` are the first
three of the C struct. The program cannot run in the book, which has no such lib, so it is
shown as a file:

```zaka-file
// counter.zaka
ရေတွက်စက် ဟူသော ဒေတာအစုအဝေးသည် ပြင်ပမှ လာသော ဒေတာအစုအဝေး ဖြစ်၍ ၎င်းတွင်
count ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု၊ step ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု နှင့် label ဟူသော စာသား တစ်ခု တို့ပါဝင်သည်။

counter_open ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libcounter" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
step ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ရေတွက်စက် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
counter_next ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libcounter" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
c ဟူသော ရေတွက်စက် တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
counter_close ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libcounter" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
c ဟူသော ရေတွက်စက် တစ်ခု ကို အပိုင်အနေဖြင့် လက်ခံမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် စက် သည် counter_open ဟူသော လုပ်ဆောင်ချက်ကို ၅ ဖြင့် လုပ်ဆောင်သော ရလဒ် ထဲမှ ဒေတာ ဖြစ်ပါစေ။
counter_next ဟူသော လုပ်ဆောင်ချက်ကို စက် ဖြင့် လုပ်ဆောင်မည်။
"{စက် ၏ count} {စက် ၏ step}" ဟု ပြသမည်။
စက် ၏ label ကို ပြသမည်။
counter_close ဟူသော လုပ်ဆောင်ချက်ကို စက် ဖြင့် လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

### What a program cannot do with a handle

A program cannot do three things with a handle, and each one is ZK0473: make one
(`ရေတွက်စက်()`), duplicate the struct behind it (`၏ မိတ္တူ`, `ကူးထည့်မည်`, a deep
duplicate), and print it, even inside an optional. Assigning a handle is fine, since the
variables share the box. A duplicate of the struct is what Zaka cannot make, as the handle
has just the address.

```zaka-error ZK0473
ပြတင်းပေါက် ဟူသော ဒေတာအစုအဝေးသည် ပြင်ပမှ လာသော ဒေတာအစုအဝေး ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ပြတင်းပေါက်() ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0473]: "ပြတင်းပေါက်" comes from outside the program, so Zaka cannot make a value of it
```

```zaka-error ZK0474
ရက်စွဲ ဟူသော ဒေတာအစုအဝေးသည် ပြင်ပမှ လာသော ဒေတာအစုအဝေး ဖြစ်၍ ၎င်းတွင် နှစ် ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ပြင် ဟူသော လုပ်ဆောင်ချက် သည် ရ ဟူသော ရက်စွဲ တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ရ ၏ နှစ် ကို ၂၀၂၅ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0474]: a field of "ရက်စွဲ" belongs to the struct C keeps, so Zaka reads it but cannot write it
```

Neither build declares the C struct: a read uses the address of the struct and adds the
offset of the field. The VM does it with the operation [`c_load`](../vm/instructions.md),
and a native build with `memcpy` (see [Native builds](../compiler/native.md)).

## The lib

The text before `မှလာသော` is a lib name or a path to the file of a lib. A name, such as
`"libsqlite3"`, is looked up where the host keeps its libs. A path has a `/` or a `\` in
it, or ends in `.so`, `.dylib` or `.dll`, such as `"./libpair.so"` or
`"/opt/x/libfoo.dylib"`. A relative path starts at the folder of the file that declares the
function, as an import does, so the program finds its lib wherever it runs from.

## How each build calls C

A lib has a different name and file form on each OS, so we use a
[compile-time conditional](compile-time.md) to declare the C functions of each one.

- `zakac run` opens each lib when the program first calls a C function. `libX` is the
  file `libX.dylib`, `libX.so` or `X.dll` (libc and libm on Windows are `ucrtbase.dll`), a
  path opens as it is, and a C name that the lib does not have is looked up in the
  process, which has libc. An address that lies in a block of the VM reaches C as the
  native address of that octet, so C reads and writes the block in place. After each call
  the VM writes out what C keeps back for its streams, so the text that a C function prints
  comes in order with the text that the program prints. A lib that does not open, or a C
  name that no lib has, stops the program with `ForeignLink`.
- The VM in the browser opens no lib, and a call of a C function stops the program with
  `ForeignCall`.
- `zakac run --jit` opens the lib of each C function as the VM opens it, before the
  program starts, and calls through libffi, as the VM does.
- `zakac build --backend native` links each lib as the C target does, below, and calls
  through libffi.
- `zakac build --backend c` declares each C function in a separate C file and links `-lX`
  for each lib `libX` other than libc and libm, and a lib given as a path as it is, with
  its folder as a run path. We give the folder of a named lib with
  `--ld-flag -L<folder>`. On macOS a lib given as a path must name itself with an absolute
  path or `@rpath/...` (its install name), since a bare file name there is looked up in the
  folder that the program starts from.

C does what C does: a C function that writes past a block, or keeps an address after the
call, harms the run as it would harm a C program. A native address that C returns, and that
lies in no block, passes back to C as it is, but the VM cannot read through it. C can call
back into the program through a function value of a function kind that crosses
([Callbacks](#callbacks)).
