---
title: Constants
description: Values that never change, declared with ကိန်းသေ.
---

# Constants

Some values never change while a program runs, such as the number π, the length of a week
or the pass mark of a test. We declare these as *constants* (`const`) with `ကိန်းသေ`
("fixed number") in place of `ကိန်းရှင်` ("living number", a variable).

## You will learn

- How to declare a constant with `ကိန်းသေ`.
- How to build a constant from another constant.
- The two rules of a constant: it needs a value, and the value never changes.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ တစ်ပတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇ ဖြစ်ပါစေ။   // the length of a week
တစ်ပတ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၇
```

A constant declaration is the same as a variable declaration, with `ကိန်းသေ` as its first
word:

```
ကိန်းသေ <name> သည် <kind> တစ်ခု ဖြစ်၍ တန်ဖိုးသည် <value> ဖြစ်ပါစေ။
```

| Zaka | Reads as | Makes |
| --- | --- | --- |
| `ကိန်းရှင်` | "living number" | a variable: its value can change |
| `ကိန်းသေ` | "fixed number" | a constant: its value never changes |

Constants make a program simpler to read, because `တစ်ပတ်` is clearer than `၇`, and
simpler to change, because we fix the value in one place and each use follows.

## Constants from constants

The value of a constant can use other constants, and we use a constant in the same form
as a variable:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ ပိုင် သည် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃.၁၄၁၅၉ ဖြစ်ပါစေ။     // pi
ကိန်းသေ တစ်ပတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇ ဖြစ်ပါစေ။          // the length of a week
ကိန်းသေ နှစ်ပတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် တစ်ပတ် * ၂ ဖြစ်ပါစေ။ // built from another constant

ပိုင် ကို ပြသမည်။
နှစ်ပတ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၃.၁၄၁၅၉
၁၄
```

## A constant needs a value

We must give a constant its value when we declare it, because we cannot set it later:

```zaka-error ZK0308
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ တစ်ပတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0308]: constant "တစ်ပတ်" needs an initial value
```

## A constant never changes

After the declaration, we can never give a constant a new value. The next page shows how
to change a variable, and the same sentence on a constant is an error:

```zaka-error ZK0306
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ တစ်ပတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇ ဖြစ်ပါစေ။
တစ်ပတ် ကို ၈ ဟု ထားမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0306]: "တစ်ပတ်" is a constant and cannot be assigned
```

## Details

- **Shorter forms.** A constant can also let its value choose the data kind, as in
  `ကိန်းသေ တစ်ပတ် သည် ၇ ဖြစ်ပါစေ။`, and `ကနဦးတန်ဖိုး` counts as a value. See
  [Declarations](../../manual/language/declarations.md#declarations).
- **Constants in other places.** We cannot pass a constant to a function that changes its
  argument (ZK0306), and an inner block can declare a variable with the name of a
  constant, which [Blocks and scope](scope.md) explains. See [Constants and
  counters](../../manual/language/scope.md#constants-and-counters).

## Recap

- We declare a constant with `ကိန်းသေ`, in the same form as a variable.
- A constant needs its value in the declaration.
- We can never give a constant a new value.
- A good constant name, such as `တစ်ပတ်`, is clearer than the bare number.

## Hands on

- Declare a constant for the number of minutes in an hour, and use it to print the
  number of minutes in 24 hours.
- Declare a constant with no value and read the message.

Next: [Changing a variable](assignment.md).
