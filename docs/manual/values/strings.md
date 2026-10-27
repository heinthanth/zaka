---
title: Text in quotes
description: Text in double quotes, which for now is just for printing.
---

# Text in quotes

Text in double quotes, such as `"မင်္ဂလာပါ"`, is a *string*: a row of characters. In this
version of Zaka, a string can be **printed** and nothing more. It cannot be stored in a
variable, and two strings cannot be joined, at least for now.

```zaka
"မင်္ဂလာပါ" ဟု ပြသပါ။
"ဆို \"ဟယ်လို\"" ဟု ပြသပါ။               // \" puts a double quote inside
"ပထမကြောင်း{နောက်တစ်ကြောင်း}ဒုတိယကြောင်း" ဟု ပြသပါ။
```

```text
မင်္ဂလာပါ
ဆို "ဟယ်လို"
ပထမကြောင်း
ဒုတိယကြောင်း
```

Strings use the same [escape sequences](characters.md#escape-sequences) as characters.
Print a string with `ဟု`, which quotes it, rather than `ကို`.

## Just for printing

Using a string elsewhere is an error:

```zaka-error ZK0223
ကိန်းရှင် နာမည် သည် စာလုံး ဖြစ်၍ တန်ဖိုးသည် "မောင်မောင်" ဖြစ်ပါစေ။
```

```text
error[ZK0223]: a string literal is allowed as the operand of "ပြသပါ" and nowhere else
```

A string kind, with operations to join and compare text, is planned for a later
version of Zaka.

## Hands on

- Print a short poem, one line per `ပြသပါ`.
- Print the same poem with a single `ပြသပါ`, using `{နောက်တစ်ကြောင်း}`.

Next: [Variables](../variables/README.md).
