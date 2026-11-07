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
၃၂ ဘစ် ကိန်းပြည့် n ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် abs ကို -၄၂ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
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
| `၃၂ ဘစ် ကိန်းပြည့် n ကို လက်ခံပြီး` | "accepting a 32-bit integer n" |
| `၃၂ ဘစ် ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။` | "will return a 32-bit integer result" |

## Kinds C knows

C has integers, floats and addresses, so those are what can cross: integers of 8 to 64
bits, decimal numbers, true and false, addresses, and structs made of these.

Text can go in too. C reads a string up to a zero octet, so the call hands C a duplicate
of the text with a zero octet at its end, and drops the duplicate once C is done.
`strlen` counts the u8s of the text, and a Burmese letter takes three of them:

```zaka
လုပ်ဆောင်ချက် strlen သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
စာသား s ကို လက်ခံပြီး လက္ခဏာမဲ့ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် strlen ကို "မောင်" တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၅
```

Text cannot come back as a result: Zaka cannot tell who frees what C gives. Lists,
owners and function values have no form in C either, so the compiler refuses them with
[ZK0465](../../manual/reference/errors.md).

```zaka-error ZK0465
လုပ်ဆောင်ချက် getenv သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
စာသား n ကို လက်ခံပြီး စာသား ရလဒ် ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0465]: a C function cannot take or give back a value of the kind "text"
```

## Other values

`printf` takes a format, then as many values as the format asks for. Its header ends the
parameters with `နှင့် အခြားတန်ဖိုးများ` ("and other values"), and a call gives the
format, then the values:

```zaka-file
// hello.zaka
လုပ်ဆောင်ချက် printf သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
စာသား fmt နှင့် အခြားတန်ဖိုးများ ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် printf ကို "%s is %ld\n"၊ "x"၊ ၄၂ တို့ဖြင့် လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
x is 42
```

The format must fit the values, as in C: `%ld` for a `ကိန်းပြည့်`, which has 64 bits, `%d`
for a `၃၂ ဘစ် ကိန်းပြည့်`, `%f` for a decimal number and `%s` for text.

## The lib

The text before `မှလာသော` names the lib, as `"libc"` does, or gives the path to its file,
as `"./libpair.so"` does. A path that does not start at `/` starts at the folder of the
program file.

## Addresses

A C function that writes a value takes the address of it. That needs borrowed addresses,
which [C functions and addresses](../pointers/c.md) teaches once pointers are known.

## Where the call runs

- `zakac run` opens the lib when the program first calls a function from it.
- `zakac build --target c` links the lib into the executable.
- The explorer in the browser cannot open a lib, so a program that calls C stops there
  with a message. Run it with `zakac run`.

Callbacks are not supported: a function of the program cannot go to C. The full rules, with
the kinds that cross and what each build does, are in
[C functions](../../manual/language/functions.md#c-functions).

## Hands on

- Declare `labs` from `"libc"` over 64-bit integers and call it with a negative number.
- Declare `sqrt` from `"libm"` over decimal numbers and print the square root of 2.0.

Next: [Text](../text/README.md).
