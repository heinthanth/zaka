---
title: Struct mistakes
description: The mistakes structs make possible, and how Zaka catches each one.
---

# Struct mistakes

## A struct declared inside a block

A struct is declared at the top level and nowhere else: not in a block, a loop, a
conditional or a function.

```zaka-error ZK0228
အကယ်၍ မှန် ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
error[ZK0228]: structs can be declared at the top level and nowhere else
```

## One name for two structs

```zaka-error ZK0316
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။
လူ ဟူသော ဒေတာအစုအဝေးတွင် အရပ် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0316]: "လူ" names a struct, an enumeration or a kind alias declared before
```

## One name for two fields

A field is found through its name, so two fields of one struct need two names:

```zaka-error ZK0317
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု၊ အသက် ဟူသော ဒဿမကိန်း တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0317]: "လူ" names the field "အသက်" twice
```

## A name that is no struct

A kind that names a struct no declaration gives is refused:

```zaka-error ZK0318
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် လုပ် တစ်ခု ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0318]: no struct or enumeration named "လုပ်" is declared
```

Check the spelling of the name against the declaration.

## A struct name that is also a value name

A struct name starts a kind wherever a value can stand, as in `လူ တစ်ခု ၏ အရွယ်အစား`, so the
same name cannot name a variable, a constant, a parameter, a field or a function too:

```zaka-error ZK0319
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လူ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0319]: "လူ" names a struct or an enumeration, so it cannot name a value or a function too
```

## A field the struct does not have

```zaka-error ZK0320
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်ပါစေ။
ကျော် ၏ အရပ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0320]: "လူ" has no field named "အရပ်"
```

## One field set twice

A struct literal gives each field one value:

```zaka-error ZK0321
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အသက် ၂၀၊ အသက် ၃၀ ရှိသော လူတစ်ခု ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0321]: the "လူ" literal sets the field "အသက်" twice
```

A struct literal that names a field the struct does not have is ZK0320, as above.

## A field left out

A struct literal gives each field a value. Zaka fills in no field on its own:

```zaka-error ZK0482
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု၊ အမြင့် ဟူသော ဒဿမကိန်း တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ(အသက်: ၂၀) ဖြစ်ပါစေ။
ကျော် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0482]: "လူ" needs a value for "အမြင့်"
```

Give the field its value, or `ကနဦးတန်ဖိုး` for its initial value:
`လူ(အသက်: ၂၀၊ အမြင့်: ကနဦးတန်ဖိုး)`.

## A struct read before each field is set

A struct declared with no value counts as set once each field is. A read before then is
refused, since one field would hold no value:

```zaka-error ZK0722
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု၊ အမြင့် ဟူသော ဒဿမကိန်း တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်ပါစေ။
ကျော် ၏ အသက် ကို ၂၀ ဟု ထားမည်။
ကျော် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0722]: "ကျော်" can be unset here: a path reaches this read with no value given to it
```

Set `ကျော် ၏ အမြင့်` too before the read, or declare the struct with a value, such as
`ကနဦးတန်ဖိုး`. The rules in full are in [Set before
read](../../manual/language/statements.md#set-before-read).

## A field of a value that is no struct

```zaka-error ZK0426
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
က ၏ အသက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0426]: "အသက်" is read as a field, but the value before "၏" is isize, not a struct
```

## A field read in a sum

A field inside an operator expression goes in parentheses. Without them, the sentence
does not read:

```zaka-error ZK0209
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်ပါစေ။
ကျော် ၏ အသက် + ၁ ကို ပြသမည်။
```

```text
error[ZK0209]: operands left over before "ပြသမည်": ကျော် (၏)
```

Write `(ကျော် ၏ အသက်) + ၁`.

## A struct that holds itself

A field sits inside its struct, so a struct that holds itself would be larger than
itself:

```zaka-error ZK0427
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု၊ သူ ဟူသော လူ တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0427]: "လူ" holds itself, so it would have no end
```

Part III shows how a struct reaches another of its kind through an address: see [A linked
list](../pointers/structs.md#a-linked-list).

## Ordering structs

`==` and `!=` take two structs of one kind, but `<`, `>` and the like do not: a struct has
no order of its own. Compare the field that gives the order:

```zaka-error ZK0403
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် လူ(အသက်: ၁၀) ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် လူ(အသက်: ၁၂) ဖြစ်ပါစေ။
က < ခ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0403]: < needs a number or a character, found လူ
```

Write `က ၏ အသက် < ခ ၏ အသက်` instead.

## The wrong classifier

Burmese counts things with a classifier. A struct is counted with the word its
declaration names, `ခု` when it names none, `စာလုံး` with `လုံး`, and each other kind
with `ခု`.
Another word gives a warning, and the program still runs, since the meaning is the same:

```zaka-warning ZK0509
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0509]: လူ is counted with "ယောက်", not "ခု"
```

The same warning comes for `ယောက်` after a kind such as `ကိန်းပြည့်`, wherever the kind is
counted:

```zaka-warning ZK0509
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ယောက် ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0509]: isize is counted with "ခု", not "ယောက်"
```

A struct literal counts its struct with the same word, after `တစ်`:

```zaka-warning ZK0509
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ယောက် ဖြစ်၍ တန်ဖိုးသည် အသက် ၂၀ ရှိသော လူတစ်ခု ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0509]: လူ is counted with "ယောက်", not "ခု"
```

## Hands on

- Declare a struct with a field `အသက်`, then read a field `အရပ်` and read the message.
- Give a struct the classifier `ယောက်`, declare a variable of it with `တစ်ခု`, and
  compare the warning with the ones above.

Next: [Shared objects](objects.md).
