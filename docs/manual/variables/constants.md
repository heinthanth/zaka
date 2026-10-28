---
title: Constants
description: Values that never change, declared with ကိန်းသေ.
---

# Constants

Some values never change while a program runs: the number π, the length of a week, the
pass mark of a test. Declare these as *constants* with `ကိန်းသေ` ("fixed
number") instead of `ကိန်းရှင်` ("living number", a variable).

```zaka
ကိန်းသေ ပိုင် သည် ဒဿမကိန်း ဖြစ်၍ တန်ဖိုးသည် ၃.၁၄၁၅၉ ဖြစ်ပါစေ။         // pi
ကိန်းသေ တစ်ပတ် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၇ ဖြစ်ပါစေ။              // the length of a week
ကိန်းသေ နှစ်ပတ် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် တစ်ပတ် * ၂ ဖြစ်ပါစေ။     // built from another constant

ပိုင် ကို ပြသပါ။
နှစ်ပတ် ကို ပြသပါ။
```

```text
၃.၁၄၁၅၉
၁၄
```

| Zaka | Reads as | Makes |
| --- | --- | --- |
| `ကိန်းရှင်` | "living number" | a variable: its value can change |
| `ကိန်းသေ` | "fixed number" | a constant: its value never changes |

Constants make a program simpler to read (`တစ်ပတ်` is clearer than `၇`) and simpler to
change: fix the value in one place and each use follows.

## Rules for constants

A constant must be given its value when it is declared:

```zaka-error ZK0308
ကိန်းသေ တစ်ပတ် သည် ကိန်းပြည့် ဖြစ်ပါစေ။
```

```text
error[ZK0308]: constant "တစ်ပတ်" needs an initial value
```

After that, it can never be given a new value. The next page shows how to change a
variable; attempting the same on a constant fails:

```zaka-error ZK0306
ကိန်းသေ တစ်ပတ် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၇ ဖြစ်ပါစေ။
တစ်ပတ် ကို ၈ ဟု ထားပါ။
```

```text
error[ZK0306]: "တစ်ပတ်" is a constant and cannot be assigned
```

## Hands on

- Declare a constant for the number of minutes in an hour, and use it to print the
  number of minutes in 24 hours.

Next: [Changing a variable](assignment.md).
