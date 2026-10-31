---
title: Struct mistakes
description: The mistakes structs make possible, and how Zaka catches each one.
---

# Struct mistakes

## A struct declared inside a block

A struct is declared at the top level and nowhere else: not in a block, a loop, a
conditional or a function.

```zaka-error ZK0228
အကယ်၍ မှန် ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက် တို့ ပါဝင်သည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
```

```text
error[ZK0228]: structs can be declared at the top level and nowhere else
```

## One name for two structs

```zaka-error ZK0316
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက် တို့ ပါဝင်သည်။
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အရပ် တို့ ပါဝင်သည်။
```

```text
error[ZK0316]: "လူ" names a struct declared before
```

## One name for two fields

A field is found through its name, so two fields of one struct need two names:

```zaka-error ZK0317
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ဒဿမကိန်း အသက် တို့ ပါဝင်သည်။
```

```text
error[ZK0317]: "လူ" names the field "အသက်" twice
```

## A name that is no struct

A kind that names a struct no declaration gives is refused:

```zaka-error ZK0318
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက် တို့ ပါဝင်သည်။
ကိန်းရှင် က သည် လုပ် ဖြစ်ပါစေ။
```

```text
error[ZK0318]: no struct named "လုပ်" is declared
```

Check the spelling of the name against the declaration.

## A struct name that is also a value name

A struct name starts a kind wherever a value can stand, as in `လူ ၏ အရွယ်အစား`, so the
same name cannot name a variable, a constant, a parameter, a field or a function too:

```zaka-error ZK0319
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက် တို့ ပါဝင်သည်။
ကိန်းရှင် လူ သည် ကိန်းပြည့် ဖြစ်ပါစေ။
```

```text
error[ZK0319]: "လူ" names a struct, so it cannot name a value or a function too
```

## A field the struct does not have

```zaka-error ZK0320
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက် တို့ ပါဝင်သည်။
ကိန်းရှင် ကျော် သည် လူ ဖြစ်ပါစေ။
ကျော် ၏ အရပ် ကို ပြသပါ။
```

```text
error[ZK0320]: "လူ" has no field named "အရပ်"
```

## One field set twice

A struct literal gives each field one value:

```zaka-error ZK0321
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက် တို့ ပါဝင်သည်။
ကိန်းရှင် ကျော် သည် လူ ဖြစ်၍ တန်ဖိုးသည် အသက် ၂၀၊ အသက် ၃၀ ရှိသော လူတစ်ခု ဖြစ်ပါစေ။
```

```text
error[ZK0321]: the "လူ" literal sets the field "အသက်" twice
```

A struct literal that names a field the struct does not have is ZK0320, as above.

## A field of a value that is no struct

```zaka-error ZK0426
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်ပါစေ။
က ၏ အသက် ကို ပြသပါ။
```

```text
error[ZK0426]: "အသက်" is read as a field, but the value before "၏" is isize, not a struct
```

For a struct that a pointer points at, read through the pointer first:
`p ၏ တန်ဖိုး ၏ အသက်`, not `p ၏ အသက်`.

## A field read in a sum

A field inside an operator expression goes in parentheses. Without them, the sentence
does not read:

```zaka-error ZK0209
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက် တို့ ပါဝင်သည်။
ကိန်းရှင် ကျော် သည် လူ ဖြစ်ပါစေ။
ကျော် ၏ အသက် + ၁ ကို ပြသပါ။
```

```text
error[ZK0209]: operands left over before "ပြသပါ": ကျော် (၏)
```

Write `(ကျော် ၏ အသက်) + ၁`.

## A struct that holds itself

A field sits inside its struct, so a struct that holds itself would be larger than
itself:

```zaka-error ZK0427
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ လူ သူ တို့ ပါဝင်သည်။
```

```text
error[ZK0427]: "လူ" holds itself, so it would have no end
```

Hold its address instead, as `လူ လိပ်စာ သူ`: see [A linked list](README.md#a-linked-list).

## A struct as a function result

```zaka-error ZK0428
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက် တို့ ပါဝင်သည်။
လုပ်ဆောင်ချက် ဖန်တီး သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ လူ ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
```

```text
error[ZK0428]: a function cannot give back a struct as its result
```

Take a borrowed address as a parameter, and fill the fields through it. See [Passing to
a function](README.md#passing-to-a-function).

## A borrow in a field

A borrow lasts as long as the owner it borrows from, and the checker follows that through
variables and parameters. A struct can sit in the heap, out of the checker's sight, so a
field holds an owner (`လူ လိပ်စာ`) or a raw address (`လူ လိပ်စာအကြမ်း`):

```zaka-error ZK0718
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ငှားထားသော ကိန်းပြည့် လိပ်စာ ရှင် တို့ ပါဝင်သည်။
```

```text
error[ZK0718]: the field "ရှင်" holds a borrow, and borrows live in variables and parameters alone
```

## Using a struct after it moved

A struct that holds an owner moves when it is set to another, so the old one cannot be
used after:

```zaka-error ZK0701
ဒေတာအမျိုးအစား အဆစ် တွင် ကိန်းပြည့် တန်ဖိုးရင်း၊ အဆစ် လိပ်စာ နောက် တို့ ပါဝင်သည်။
ကိန်းရှင် က သည် အဆစ် ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အဆစ် ဖြစ်ပါစေ။
မင်မိုရီတွင် အဆစ် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို က ၏ နောက် တွင် သိမ်းထားမည်။
ခ ကို က ဟု ထားပါ။
က ၏ နောက် ကို ပြသပါ။
```

```text
error[ZK0701]: "က" was moved to another owner and now holds ဗလာ
```

## Comparing structs

`==` and `!=` do not take structs. Compare the fields in turn:

```zaka-error ZK0403
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက် တို့ ပါဝင်သည်။
ကိန်းရှင် က သည် လူ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် လူ ဖြစ်ပါစေ။
က == ခ ကို ပြသပါ။
```

```text
error[ZK0403]: == needs a number, found လူ
```

## The wrong classifier

Burmese counts things with a classifier. A struct is counted with the word its
declaration names, `ခု` when it names none, and each other kind is counted with `ခု`.
Another word gives a warning, and the program still runs, since the size is the same:

```zaka-warning ZK0509
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက် တို့ ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။
ကိန်းရှင် p သည် လူ လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် လူ ၃ ခုစာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။
လိပ်စာ p ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
warning[ZK0509]: လူ is counted with "ယောက်", not "ခု"
```

The same warning comes for `ယောက်` after a kind such as `ကိန်းပြည့်`, once a struct of the
program has named `ယောက်` as its classifier:

```zaka-warning ZK0509
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက် တို့ ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။
ကိန်းရှင် p သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၃ ယောက်စာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။
လိပ်စာ p ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
warning[ZK0509]: isize is counted with "ခု", not "ယောက်"
```

A struct literal counts its struct with the same word, after `တစ်`:

```zaka-warning ZK0509
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက် တို့ ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။
ကိန်းရှင် ကျော် သည် လူ ဖြစ်၍ တန်ဖိုးသည် အသက် ၂၀ ရှိသော လူတစ်ခု ဖြစ်ပါစေ။
```

```text
warning[ZK0509]: လူ is counted with "ယောက်", not "ခု"
```

## Hands on

- Declare a struct with a field `အသက်`, then read a field `အရပ်` and read the message.
- Give a struct the classifier `ယောက်`, size a heap block for it with `ခု`,
  and compare the warning with the one above.

Next: [Natural Zaka](../natural-zaka/README.md).
