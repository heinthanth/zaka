---
title: Calling C
description: Declare a function of a shared lib, such as libc or libm, and call it as a function of the program.
---

# Calling C

Zaka can call functions such as `abs` from `libc` or the math functions from `libm`. To
use the foreign function interface (FFI), declare the function and name its lib
(library). We can then call it like a Zaka function.

## You will learn

- how to declare a C function and call it in an unsafe block
- which data kinds a call can pass to C, and how text and lists go to C
- how C can write into a list of the program
- how to declare different C functions on each OS

```zaka
// "Function abs, not to be trusted, coming from the lib libc, accepting a
//  32-bit integer n, will return a 32-bit integer result."
abs ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
n ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။

အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
abs ဟူသော လုပ်ဆောင်ချက်ကို -၄၂ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
၄၂
```

In the function signature, we write `"<lib>" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍` after `သည်` to
show that the function comes from a lib, and we leave out the statements and the block
closer, because the function is the C function of the same name:

```text
<C name> ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "<lib>" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
<parameters> လက်ခံပြီး <kind> တစ်ခု ကို ပြန်ပေးမည်။
```

| Zaka | Reads as |
| --- | --- |
| `abs` | the name in C |
| `စိတ်မချရသော` | "not to be trusted": the function is unsafe |
| `"libc" မှလာသော` | "coming from the lib libc" |
| `လုပ်ဆောင်ချက်ဖြစ်၍` | "being a function, and" |
| `n ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး` | "accepting a 32-bit integer called n" |
| `၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။` | "will return a 32-bit integer" |

We write the name the same as C does, in ASCII letters, digits and `_`, and each C
function names its lib, `libc` too. Zaka cannot see what C code does, because C can read
and write each part of the program's storage (memory) and Zaka checks none of it. So each C
function is unsafe, as [Unsafe Zaka](../pointers/README.md) explained: its function
signature starts with `စိတ်မချရသော`, and each call to it goes in an unsafe block. These
words mark the places where the program trusts C, and the rest of the program remains safe
code. This page needs no pointer, and [C functions and addresses](../pointers/c.md) adds
them later.

## Data kinds C knows

C has integers, floats and addresses. A call can pass integers of 8 to 64 bits, decimal
numbers, true and false, structs made of these, text (strings), lists of numbers and
function values, and a C function can return nothing, one of those numbers, or a struct of
them.

C reads a string up to a zero octet (byte), so the call hands C a duplicate of the text with
a zero octet at its end, and drops the duplicate once C is done. Here `strlen` counts the
octets of the text, and each Burmese letter has three of them:

```zaka
strlen ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။

အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
strlen ဟူသော လုပ်ဆောင်ချက်ကို "မောင်" ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
၁၅
```

Text cannot come back as a plain `စာသား`, because C can return NULL in its place, so a
plain `စာသား` result is an error:

```zaka-error ZK0465
getenv ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
n ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး စာသား တစ်ခု ကို ပြန်ပေးမည်။
```

```text
error[ZK0465]: a C function cannot take or give back a value of the kind "text"
  = help: C can give back NULL: write the result as an optional, "<kind> ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်"
```

The help line asks for an [optional](../optionals/README.md) (option type), which has the
text, or `အလွတ်` when C returns NULL. Zaka duplicates the text, so the program never has
the address, and we test the optional the same as each other optional:

```zaka
getenv ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
name ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။

အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ရှာတွေ့ချက် သည် getenv ဟူသော လုပ်ဆောင်ချက်ကို "ZAKA_NO_SUCH_NAME" ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
အကယ်၍ ရှာတွေ့ချက် ထဲတွင် ဒေတာ မရှိလျှင် "မရှိပါ" ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
မရှိပါ
```

[C functions and addresses](../pointers/c.md#text-that-c-returns) shows `strstr` and
`strdup`, together with the C functions that use addresses.

## Lists of numbers

A list whose cells are integers of up to 64 bits or decimal numbers goes to C as the
address of its first cell, which is how C passes an array. A list of other cells, such as a
list of text, is the error ZK0465:

```zaka
// strlen reads u8s up to a zero, so a list of u8s that ends in 0 will do
strlen ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ကို လက်ခံပြီး လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို
ပြန်ပေးမည်။

ကိန်းရှင် စာ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၀၄၊ ၁၀၅၊ ၀] ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
strlen ဟူသော လုပ်ဆောင်ချက်ကို စာ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
၂
```

## C writes into a list

```text
<parameter> ဟူသော <kind> စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန်
<C name> ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "<lib>" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ <parameters> လက်ခံမည်။
```

When a list parameter is a [mutable parameter](parameters.md#changing-the-callers-variable)
(`inout`), marked with `ဒေတာ ပြောင်းလဲရန်`, C writes into the list in place, as a C
function writes into the cells it gets. Before the call, the list gets cells that it does
not share, so a duplicate of the list never sees the writes. `memset` sets a run of octets,
and because it has no result, its function signature ends in `လက်ခံမည်` ("will accept"):

```zaka
memset ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
b ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လည်းကောင်း
c ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
n ဟူသော လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံမည်။

ကိန်းရှင် စာ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် မိတ္တူ သည် စာ ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
memset ဟူသော လုပ်ဆောင်ချက်ကို စာ၊ ၇ နှင့် ၂ တို့ဖြင့် လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
စာ ကို ပြသမည်။
မိတ္တူ ကို ပြသမည်။ // unchanged
```

```text
[၇၊ ၇၊ ၃]
[၁၊ ၂၊ ၃]
```

A C function that writes one value through a pointer can get a counted list of one cell in
the same manner. `frexp` splits a number into a fraction and a power of two: it returns the
fraction and writes the power into the list. It never reads the list first, so we make the
list a [writable parameter](parameters.md#filling-the-callers-variable) (`out`), marked
with `ဒေတာ ထည့်ပေးရန်`, and the caller declares it with no value. For a C function, this
mark is our promise that C writes the list, because the compiler cannot check it.

```zaka
frexp ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libm" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
x ဟူသော ဒဿမကိန်း တစ်ခု ကို လည်းကောင်း
e ဟူသော အကွက် ၁ ခုရှိသော ၃၂ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ထည့်ပေးရန် လည်းကောင်း လက်ခံပြီး
ဒဿမကိန်း တစ်ခု ကို ပြန်ပေးမည်။

ကိန်းရှင် ထပ်ကိန်း သည် အကွက် ၁ ခုရှိသော ၃၂ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
frexp ဟူသော လုပ်ဆောင်ချက်ကို ၈.၀ နှင့် ထပ်ကိန်း တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ထပ်ကိန်း ၏ အကွက် ၀ ကို ပြသမည်။
```

```text
၀.၅
၄
```

8.0 is 0.5 times 2 to the power 4. A struct marked with `ဒေတာ ပြောင်းလဲရန်` or
`ဒေတာ ထည့်ပေးရန်` goes to C as its address too, so that C can fill its fields.

## Other values

```text
<parameter> ဟူသော <kind> တစ်ခု ကို လည်းကောင်း အခြားတန်ဖိုးများနှင့်အတူ လက်ခံပြီး
```

`printf` accepts a format and then the values that the format asks for, which C writes as
`...`. To declare such a function, we end the parameters with `အခြားတန်ဖိုးများနှင့်အတူ`
("together with other values") before `လက်ခံပြီး`, with `လည်းကောင်း` after each parameter
before it, and a call passes the format and then the values:

```zaka-file
// hello.zaka
printf ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
fmt ဟူသော စာသား တစ်ခု ကို လည်းကောင်း အခြားတန်ဖိုးများနှင့်အတူ လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
printf ဟူသော လုပ်ဆောင်ချက်ကို "%s is %lld\n"၊ "x" နှင့် ၄၂ တို့ဖြင့် လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
x is 42
```

The format must fit the values, as in C: `%lld` for a `ကိန်းပြည့်`, which has 64 bits (a C
`long`, which `%ld` reads, has 32 bits on Windows), `%d` for a `၃၂ ဘစ် ကိန်းပြည့်`, `%f` for
a decimal number and `%s` for text. A function that accepts other values can still have a
mutable list parameter:

```zaka-file
// buffer.zaka
snprintf ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
buf ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လည်းကောင်း size ဟူသော လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
fmt ဟူသော စာသား တစ်ခု ကို လည်းကောင်း အခြားတန်ဖိုးများနှင့်အတူ လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
```

## The lib

The text before `မှလာသော` is the name of the lib, such as `"libc"`, or the path to its
file, such as `"./libpair.so"`. A path that does not start at `/` starts at the folder of
the program file.

## Each OS its own lib

```text
အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် အလုပ်လုပ်မည့်ကွန်ပျူတာစနစ် == "<OS>" ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
```

A lib does not have the same name and the same functions on each OS: the UCRT of Windows,
for example, calls `strdup` `_strdup`. To handle that, we use `အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင်`,
a [compile-time conditional](compile-time.md#conditions-that-call-comptime-functions) that
the compiler settles before the program runs. It keeps the arm whose condition is true for
the target, so each OS can declare its own C functions. The condition can test the facts of
the target, which are constants of the core module (prelude), such as
`အလုပ်လုပ်မည့်ကွန်ပျူတာစနစ်`:

```zaka
အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် အလုပ်လုပ်မည့်ကွန်ပျူတာစနစ် == "windows" ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
_strdup ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို အပိုင်အနေဖြင့်
ပြန်ပေးမည်။
ကူး ဟူသော လုပ်ဆောင်ချက် သည်
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
ကူး ဟူသော လုပ်ဆောင်ချက် သည်
s ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
strdup ဟူသော လုပ်ဆောင်ချက်ကို s ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကူး ဟူသော လုပ်ဆောင်ချက်ကို "စံကား" ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
```

```text
ဒေတာ("စံကား")
```

`ကူး` is a *safe wrapper*: a plain function with the C call inside its unsafe block. The
program calls `ကူး` from safe code, so the unsafe block is in one place, however
often the program duplicates text. The compiler never checks the arm that it drops for the
target, so that arm can name a C function that this computer does not have.

## Where the call runs

- `zakac run` opens the lib when the program first calls a function from it, and
  `zakac run --jit` opens it before the program starts.
- `zakac build --backend c` and `zakac build --backend native` link the lib into the
  executable.
- The explorer in the browser cannot open a lib, so a program that calls C stops there with
  a message, and we run it with `zakac run` instead.

## Details

- A C function is declared at the top level of a file and nowhere else (ZK0214), and
  without `စိတ်မချရသော` its declaration is an error ([C
  functions](../../manual/language/functions-c.md#c-functions)).
- A parameter or result of a data kind that C does not know is ZK0465, and
  `ဒေတာ ပြောင်းလဲရန်` or `ဒေတာ ထည့်ပေးရန်` on a parameter that is neither a list of numbers
  nor a struct is ZK0466 ([C functions](../../manual/language/functions-c.md#c-functions)).
- `အပိုင်အနေဖြင့်` on the result of `strdup` means that the program takes over the text that
  C hands it and frees it with the `free` of libc ([C functions and
  addresses](../pointers/c.md#text-that-c-hands-over)).
- A C function can also accept or return an address, a pointer into the program's storage,
  which [C functions and addresses](../pointers/c.md) teaches.
- The facts of the target that a compile-time condition can test are listed in [The facts
  of the target](../../manual/language/compile-time.md#the-facts-of-the-target).

## Recap

- We declare a C function with `<C name> ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "<lib>" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍`,
  its parameters and its result, and with no statements.
- Each call to a C function goes in an unsafe block.
- Text goes to C with a zero octet at its end, text from C comes back as an optional, and a
  list of numbers goes to C as the address of its first cell.
- A mutable or writable list parameter lets C write into the list.
- A compile-time conditional can declare different C functions on each OS.

## Hands on

- Declare `labs` from `"libc"` over 64-bit integers and call it with a negative number.
- Declare `sqrt` from `"libm"` over decimal numbers and print the square root of 2.0.
- Write a safe wrapper around `abs`, and call it from safe code.

Next: [Callbacks and handles](c-callbacks.md).
