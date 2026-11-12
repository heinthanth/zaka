---
title: Calling C
description: Declare a function of a shared lib, such as libc or libm, and call it as a function of the program.
---

# Calling C

Each lib on a computer holds functions written in C: `libc` has `abs`, `libm` has the
math functions. Zaka can call them. The header of the function names the lib in place of
`ပြင်ပ` ("outside"), and the rest is a header like the ones before.

```zaka
// "Function abs, coming from the lib libc, accepting a 32-bit integer n,
//  will return a 32-bit integer result."
လုပ်ဆောင်ချက် abs သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
n ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် abs ကို -၄၂ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၂
```

The declaration has no statements: the function is the C function of the same name, so its name
is written as C writes it, in ASCII letters, digits and `_`. The text before `မှလာသော` is
the lib. Each C function names its lib, `libc` too. A call is a call like each other.

| Part | Reads as |
| --- | --- |
| `abs` | the name in C |
| `"libc" မှလာသော` | "coming from the lib libc" |
| `n ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး` | "accepting a 32-bit integer called n" |
| `၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။` | "will return a 32-bit integer result" |

## Kinds C knows

C has integers, floats and addresses. Safe code can pass integers of 8 to 64 bits,
decimal numbers, true and false, structs made of these, text, lists of numbers and
function values. A result is nothing, one of those numbers, or a struct of them.

Text goes in as C wants it. C reads a string up to a zero octet, so the call hands C a
duplicate of the text with a zero octet at its end, and drops the duplicate once C is done.
`strlen` counts the u8s of the text, and a Burmese letter takes three of them:

```zaka
လုပ်ဆောင်ချက် strlen သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် strlen ကို "မောင်" တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၅
```

Text cannot come back with this form, since C can give NULL for it. A bare `စာသား` result
is refused:

```zaka-error ZK0465
လုပ်ဆောင်ချက် getenv သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
n ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး စာသား တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0465]: a C function cannot take or give back a value of the kind "text"
  = help: C can give back NULL: write the result as an optional, "<kind> ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်"
```

The form the help line names is an [optional](../optionals/README.md): the text, or `အလွတ်`
when C gives NULL. Come back to this part after [Optional values](../optionals/README.md). It needs no mark, since Zaka copies the text and the program never
holds the address. A test reads it as for each optional:

```zaka
လုပ်ဆောင်ချက် getenv သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
name ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရှာတွေ့ချက် သည် လုပ်ဆောင်ချက် getenv ကို "ZAKA_NO_SUCH_NAME" တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
အကယ်၍ ရှာတွေ့ချက် ထဲတွင် ဒေတာ မရှိလျှင် "မရှိပါ" ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မရှိပါ
```

[C functions and addresses](../pointers/c.md#text-that-c-gives-back) shows `strstr` and
`strdup`, with the C functions that take addresses.

## Lists of numbers

Come back to this section after [Lists](../lists/README.md).

A list whose cells are integers of up to 64 bits or decimal numbers goes to C as the
address of its first cell, as C passes the cells of a list. A list of other cells, such as a list of
text, is refused (ZK0465):

```zaka
// strlen reads u8s up to a zero, so a list of u8s that ends in 0 will do
လုပ်ဆောင်ချက် strlen သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ကို လက်ခံပြီး လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၀၄၊ ၁၀၅၊ ၀] ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် strlen ကို စာ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂
```

## C writes into a list

Come back to this section after [Lists](../lists/README.md).

C writes a list parameter marked
[`ဒေတာ ပြောင်းလဲရန်`](parameters.md#changing-the-callers-variable) in place, as a C function writes
the cells it was given. The list gets cells
of its own first, so a duplicate of it never sees the writes. `memset` sets a run of u8s, and
its header has no result, so it ends in `လက်ခံမည်` ("will accept"):

```zaka
လုပ်ဆောင်ချက် memset သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
b ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လည်းကောင်း c ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း n ဟူသော လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် မိတ္တူ သည် စာ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် memset ကို စာ၊ ၇၊ ၂ တို့ဖြင့် လုပ်ဆောင်မည်။
စာ ကို ပြသမည်။
မိတ္တူ ကို ပြသမည်။                 // stands as it was
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၇၊ ၇၊ ၃]
[၁၊ ၂၊ ၃]
```

A C function that writes one value through a pointer takes a counted list of one cell in
the same manner. `frexp` splits a number into a fraction and a power of two: it gives the
fraction back and writes the power into the list. It never reads the list first, so the
list is marked [`ဒေတာ ထည့်ပေးရန်`](parameters.md#filling-the-callers-variable), and the
caller declares it with no value; the mark promises that C writes it.

```zaka
လုပ်ဆောင်ချက် frexp သည် "libm" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
x ဟူသော ဒဿမကိန်း တစ်ခု ကို လည်းကောင်း e ဟူသော အကွက် ၁ ခုရှိသော ၃၂ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ထည့်ပေးရန် လည်းကောင်း လက်ခံပြီး ဒဿမကိန်း တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ထပ်ကိန်း သည် အကွက် ၁ ခုရှိသော ၃၂ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် frexp ကို ၈.၀၊ ထပ်ကိန်း တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ထပ်ကိန်း ၏ အကွက် ၀ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၀.၅
၄
```

8.0 is 0.5 times 2 to the power 4. A struct marked `ဒေတာ ပြောင်းလဲရန်` or `ဒေတာ ထည့်ပေးရန်`
goes to C as its address too, so C can fill its fields. Either mark on a parameter of
another kind is refused (ZK0466).

## Other values

`printf` takes a format, then the values the format asks for. Its header ends the
parameters with `အခြားတန်ဖိုးများနှင့်အတူ` ("together with other values") before
`လက်ခံပြီး`, each parameter before it closed with `လည်းကောင်း`, and a call gives the format,
then the values:

```zaka-file
// hello.zaka
လုပ်ဆောင်ချက် printf သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
fmt ဟူသော စာသား တစ်ခု ကို လည်းကောင်း အခြားတန်ဖိုးများနှင့်အတူ လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် printf ကို "%s is %ld\n"၊ "x"၊ ၄၂ တို့ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
x is 42
```

The format must fit the values, as in C: `%ld` for a `ကိန်းပြည့်`, which has 64 bits, `%d`
for a `၃၂ ဘစ် ကိန်းပြည့်`, `%f` for a decimal number and `%s` for text.

A C function that takes other values can still mark a list `ဒေတာ ပြောင်းလဲရန်`:

```zaka-file
// buffer.zaka
လုပ်ဆောင်ချက် snprintf သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
buf ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လည်းကောင်း size ဟူသော လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
fmt ဟူသော စာသား တစ်ခု ကို လည်းကောင်း အခြားတန်ဖိုးများနှင့်အတူ လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
```

## A function C keeps

Come back to this section after [Closures](../function-values/closures.md).

A function of the program that has captures, a [closure](../function-values/closures.md),
goes to a C function as a function value too. Some C functions keep the function and call
it later: `signal` keeps a handler, and `raise` makes the signal come. A closure that C
calls after the call that gave it ended needs *leave* from the program. The first
statement below reads "allow C to call it", and the last reads "take back the leave for
C":

```zaka
လုပ်ဆောင်ချက် signal သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
sig ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း handler ဟူသော (၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို လည်းကောင်း လက်ခံမည်။
လုပ်ဆောင်ချက် raise သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ sig ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖမ်းကြိမ် သည် ၀ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ဖမ်း သည် s ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဖမ်းကြိမ် ကို ဖမ်းကြိမ် + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ဖမ်း ဟူသော လုပ်ဆောင်ချက်ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်။
လုပ်ဆောင်ချက် signal ကို ၁၅၊ ဖမ်း ဟူသော လုပ်ဆောင်ချက် တို့ဖြင့် လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် raise ကို ၁၅ တို့ဖြင့် လုပ်ဆောင်မည်။
ဖမ်းကြိမ် ကို ပြသမည်။
ဖမ်း ဟူသော လုပ်ဆောင်ချက်ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြန်လည်ရုပ်သိမ်းမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁
```

A closure that C calls with no call running and no leave stops the program, with the
[closure escape trap](../../manual/vm/traps.md). A function with no captures needs no
leave. [Function values](../../manual/language/function-values.md#nested-functions-and-closures)
has the full rules, and [C functions and addresses](../pointers/c.md#a-closure-for-a-call)
shows a closure that C calls while a call runs.

C can call back with more than numbers. A parameter of the function can be a plain struct,
which C passes as a whole value; text, copied from the `char *` C passes; or a handle around a
struct C keeps. When C can pass NULL, write the parameter as an optional, such as
`စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`, and test it before reading it: a NULL for plain text stops
the program. A handle that C passed is C's, so the function cannot close it outside marked
code. The [Docs](../../manual/language/functions.md#c-functions) list each kind.

## A struct C keeps

Come back to this section after [Structs](../structs/README.md).

Some libs keep a struct of their own and give the program a pointer to it: a counter, a
database connection. Zaka names such a struct with a kind that comes from outside, and a
value of it is a *handle*. The declaration reads `<name> ဟူသော ပြင်ပမှ လာသော ဒေတာအစုအဝေးတွင်`, then lists
the C fields the program wants to read, from the start of the C struct:

```zaka-file
// counter.zaka
ရေတွက်စက် ဟူသော ပြင်ပမှ လာသော ဒေတာအစုအဝေးတွင်
count ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု၊ step ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု၊ label ဟူသော စာသား တစ်ခု တို့ပါဝင်သည်။

လုပ်ဆောင်ချက် counter_open သည် "libcounter" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
step ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ရေတွက်စက် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် counter_next သည် "libcounter" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
c ဟူသော ရေတွက်စက် တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် counter_close သည် "libcounter" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
c ဟူသော ရေတွက်စက် တစ်ခု ကို အပိုင်အနေဖြင့် လက်ခံမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စက် သည် လုပ်ဆောင်ချက် counter_open ကို ၅ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ထဲမှ ဒေတာ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် counter_next ကို စက် တို့ဖြင့် လုပ်ဆောင်မည်။
"{စက် ၏ count} {စက် ၏ step}" ဟု ပြသမည်။
စက် ၏ label ကို ပြသမည်။
လုပ်ဆောင်ချက် counter_close ကို စက် တို့ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

The program opens a counter, steps it once, reads two fields and its label, and closes it:

- `counter_open` gives a handle, or `အလွတ်` when C gives NULL: its result is an
  optional of the handle. A plain `ရေတွက်စက်` result would promise that C never gives
  NULL, so a NULL would stop the run. No mark is needed on these C functions, since the
  program never holds the address.
- `စက် ၏ count` reads the field from C's storage each time, so it shows what C holds now.
- `စက် ၏ label` is an optional of text, since a `char *` can be NULL.
- A field cannot be written. A write such as `စက် ၏ count ကို ၀ ဟု ထားမည်` is refused (ZK0474);
  a C function changes the struct.
- A program cannot make a handle, duplicate the struct behind it or print it (ZK0473).

Closing is the program's work. `counter_close` marks its parameter `အပိုင်အနေဖြင့်` ("as
its own"): the C function takes the handle over. After the call, each variable that shares
the handle reads as closed, and a call or a field read through one stops the program with
the [closed handle trap](../../manual/vm/traps.md#c-functions). Close a handle once, and
leave it alone after that.

[Kinds from outside the program](../../manual/language/functions.md#c-functions) has the
full rules.

## The lib

The text before `မှလာသော` names the lib, as `"libc"` does, or gives the path to its file,
as `"./libpair.so"` does. A path that does not start at `/` starts at the folder of the
program file.

## Addresses

A C function whose parameters or result hold an address needs a mark, and its calls go in
marked code. [C functions and addresses](../pointers/c.md) teaches them in the last part
of Learn.

## Where the call runs

- `zakac run` opens the lib when the program first calls a function from it.
- `zakac build --target c` links the lib into the executable.
- The explorer in the browser cannot open a lib, so a program that calls C stops there
  with a message. Run it with `zakac run`.

## Hands on

- Declare `labs` from `"libc"` over 64-bit integers and call it with a negative number.
- Declare `sqrt` from `"libm"` over decimal numbers and print the square root of 2.0.

Next: [Text](../text/README.md).
