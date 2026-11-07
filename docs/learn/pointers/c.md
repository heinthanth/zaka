---
title: C functions and addresses
description: Let a C function write a value back through a borrowed address, as frexp does.
---

# C functions and addresses

[Calling C](../functions/c.md) passes whole values. A C function that gives back more
than one value takes the address of a place to write the rest. Declare that parameter as a
borrowed address, and pass `၏ လိပ်စာ` of a variable. `frexp` splits a number into a
fraction and a power of two: it gives the fraction back and writes the power through its
`int *`.

```zaka
// frexp(x, &e): x is the fraction times two to the power e
လုပ်ဆောင်ချက် frexp သည် "libm" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
ဒဿမကိန်း x၊ ငှားထားသော ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ e တို့ကို လက်ခံပြီး ဒဿမကိန်း ရလဒ် ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ထပ်ကိန်း သည် ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် frexp ကို ၈.၀၊ ထပ်ကိန်း ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ထပ်ကိန်း ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၀.၅
၄
```

8.0 is 0.5 times 2 to the power 4, so the call gives `၀.၅` and `ထပ်ကိန်း` holds `၄`
after it. C reads and writes the block of `ထပ်ကိန်း` in place, and the borrow ends with
the call, as for each borrow.

## What the compiler refuses

- A parameter with the edit mark is [ZK0466](../../manual/reference/errors.md): lend an
  address, as above.
- A result cannot be a borrow, since C has no place to lend from. A C function that gives
  an address back gives a raw address.
- An owner is refused (ZK0465): the call would free the block as it ends.

C does what C does. A C function that writes past the block it was lent harms the run as
it would harm a C program. The full rules are in
[C functions](../../manual/language/functions.md#c-functions).

## Hands on

- Declare `modf` from `"libm"`, which splits a number into its whole and fractional parts
  and writes the whole part through a `double *`. Print both parts of 3.25.

Next: [Pointer mistakes](mistakes.md).
