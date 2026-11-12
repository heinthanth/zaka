---
title: Enumerations and switches
description: Name a short list of values as a kind of its own, and pick one block for each value with a switch.
---

# Enumerations and switches

An *enumeration* is a kind whose values are a short list of names, its *members*. A
traffic light is red, amber or green, and nothing else:

```zaka
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ၊ စိမ်း တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မီး သည် အရောင်များထဲမှ ဝါ ဖြစ်ပါစေ။
မီး ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အရောင်များထဲမှ ဝါ
```

| Zaka | Reads as |
| --- | --- |
| `အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲ` | "the data kind of variants called `အရောင်` (color)" |
| `တွင် ... တို့ပါဝင်သည်` | "in it, these are included" |
| `အရောင်များထဲမှ ဝါ` | "amber, from among the colors" |

An enumeration reads as a struct does, with `ဒေတာအမျိုးအစားကွဲ` in place of
`ဒေတာအစုအဝေး`. It is declared at the top
level of a file, as a struct is, and the rest of the program can use it from the first
line. Its name is a kind from then on:
`ကိန်းရှင် မီး သည် အရောင် တစ်ခု ဖြစ်ပါစေ။` declares a variable of it.

## Naming a member

A member is written `<name>များထဲမှ <member>`: the enumeration name, then `များ`, the
plural mark, then `ထဲမှ`, "from among". A name that ends in `များ` itself, as `အမှားများ`
does, does not take a second one: `အမှားများထဲမှ ပြည့်နေ`.

Printing a member writes it in that same form, so a printed member reads back as Zaka.
Inside text, a template slot such as `{မီး}` writes it so too:

```zaka
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ၊ စိမ်း တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မီး သည် အရောင်များထဲမှ စိမ်း ဖြစ်ပါစေ။
"မီး {မီး}" ဟု ပြသမည်။
မီး == အရောင်များထဲမှ စိမ်း ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မီး အရောင်များထဲမှ စိမ်း
မှန်
```

A name in Latin letters is written in the same form, as is a name that mixes scripts; the
lexer splits `Deliveryများထဲမှ` at the `များ`:

```zaka
Delivery ဟူသော ဒေတာအမျိုးအစားကွဲတွင် Waiting၊ Moving၊ Arrived တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် state သည် Deliveryများထဲမှ Moving ဖြစ်ပါစေ။
state ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
Deliveryများထဲမှ Moving
```

Two members of one enumeration compare with `==` and `!=`. `<` and `>` do not take
members: colors have no order.

## Member values

Each member stands for a value. With none written, the first member is 0 and each member
after it is one more than the member before. `တန်ဖိုးအနေဖြင့် <value> ရှိသည့်` ("that has
`<value>` as its value") in front of a member sets it, and the members after it count on
from there. A value can be each constant the
compiler works out: a literal, a constant, or an operation on them. A conversion gives
the value of a member:

```zaka
ကိန်းသေ အခြေခံ သည် ၁၀၀ ဖြစ်ပါစေ။
အမှားများ ဟူသော ဒေတာအမျိုးအစားကွဲတွင် တန်ဖိုးအနေဖြင့် အခြေခံ + ၄ ရှိသည့် မတွေ့၊ ပိတ်ထား၊ တန်ဖိုးအနေဖြင့် အခြေခံ * ၂ ရှိသည့် ပြည့်နေ တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မ သည် အမှားများထဲမှ ပိတ်ထား ဖြစ်ပါစေ။
မ ကို ပြသမည်။
မ ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အမှားများထဲမှ ပိတ်ထား
၁၀၅
```

The value of the first member sets the kind of each member value: an integer, a decimal
number, a character, text, a list or a struct. Members of text or of another kind that is
no integer do not count on, so each needs its value written:

```zaka
အမိန့် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် တန်ဖိုးအနေဖြင့် "go" ရှိသည့် သွား၊ တန်ဖိုးအနေဖြင့် "stop" ရှိသည့် ရပ် တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ညွှန်ကြား သည် အမိန့်များထဲမှ ရပ် ဖြစ်ပါစေ။
ညွှန်ကြား ကို ပြသမည်။
ညွှန်ကြား ကို စာသား တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အမိန့်များထဲမှ ရပ်
stop
```

A variable of an enumeration holds the value of its member, so `ညွှန်ကြား` above holds the
text `"stop"`. No conversion goes back from a value to a member, and a member never equals
a value of its kind without a conversion first.

## The initial member

No member is picked for a variable on its own: not the first, not the one of value 0.
To give an enumeration an [initial value](../variables/README.md#the-initial-value), mark
one member with `ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်` ("to be used as the initial value") in front of
it, before its value when it has one. `ကနဦးတန်ဖိုး` for an `အမိန့်` is then that member, and
so is each enumeration field or cell of a struct or a list at its initial value:

```zaka
အမိန့် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် တန်ဖိုးအနေဖြင့် "go" ရှိသည့် သွား၊ ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့် တန်ဖိုးအနေဖြင့် "stop" ရှိသည့် ရပ် တို့ပါဝင်သည်။
ခရီး ဟူသော ဒေတာအစုအဝေးတွင် အကွာ ဟူသော ကိန်းပြည့် တစ်ခု၊ ညွှန် ဟူသော အမိန့် တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ညွှန်ကြား သည် အမိန့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ညွှန်ကြား ကို ပြသမည်။
ကိန်းရှင် ခရီးစဉ် သည် ခရီး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ခရီးစဉ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အမိန့်များထဲမှ ရပ်
ခရီး(အကွာ: ၀၊ ညွှန်: အမိန့်များထဲမှ ရပ်)
```

An enumeration with no marked member has no initial value, so `ကနဦးတန်ဖိုး` for it is a
mistake, and so are two marks: see [Enumeration
mistakes](mistakes.md#no-member-marked-as-the-initial-one).

## Choosing with a switch

A *switch* compares one value, its *subject*, with a list of cases, and runs the block of
the first case that equals it:

```zaka
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ၊ စိမ်း တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မီး သည် အရောင်များထဲမှ ဝါ ဖြစ်ပါစေ။
မီး သည် နီ ဖြစ်လျှင် "ရပ်" ဟု ပြသမည်။
ဝါ ဖြစ်လျှင် "ဖြည်းဖြည်း" ဟု ပြသမည်။
စိမ်း ဖြစ်လျှင် "သွား" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဖြည်းဖြည်း
```

| Zaka | Reads as |
| --- | --- |
| `မီး သည် နီ ဖြစ်လျှင်` | "if the light is red" |
| `ဝါ ဖြစ်လျှင်` | "if amber" |

The first arm names the subject with `သည်`; each arm after it gives a case alone. Over
an enumeration, a case can be a bare member name, as above, since the subject tells which
enumeration it comes from. `အရောင်များထဲမှ နီ ဖြစ်လျှင်` works as well. A block can follow
`ဖြစ်လျှင်` in place of one statement, as it can after `အကယ်၍`.

One arm can take several cases, with `၊` between them and `တို့` after the last:
`ဝါ၊ စိမ်း တို့ ဖြစ်လျှင်` runs for amber and for green.

## The default arm

`တခြား <kind> တစ်ခု ဖြစ်လျှင်`, "if another `<kind>`", comes last and runs when no case equals
the subject. Over an enumeration, a switch either names each member in an arm or ends with
the default arm, so a member added to the enumeration later is caught at each switch that
forgets it. For an enumeration whose name ends in `များ`, the default arm writes the name
without it: `တခြား အမှား တစ်ခု ဖြစ်လျှင်`. The kind takes its measure word, `ခု` unless the
enumeration names another.

A switch takes a subject of each kind that `==` takes: a number, a character, text, a
list, a struct or an enumeration. Over a number or text, the default arm can be left out;
then a value that no case equals runs nothing.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရက် သည် ၆ ဖြစ်ပါစေ။
ရက် သည် ၀ ဖြစ်လျှင် "တနင်္ဂနွေ" ဟု ပြသမည်။
၆ ဖြစ်လျှင် "စနေ" ဟု ပြသမည်။
တခြား ကိန်းပြည့် တစ်ခု ဖြစ်လျှင် "ကြားရက်" ဟု ပြသမည်။

ကိန်းရှင် ညွှန်ကြား သည် "ရပ်" ဖြစ်ပါစေ။
ညွှန်ကြား သည် "သွား" ဖြစ်လျှင် "go" ဟု ပြသမည်။
"ရပ်" ဖြစ်လျှင် "stop" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
စနေ
stop
```

The rules in full are in [Enumerations](../../manual/language/enums.md), and the mistakes
the compiler catches are in [Enumeration mistakes](mistakes.md).

## Hands on

- Declare an enumeration that names the seven parts of a week, from `တနင်္ဂနွေ` to `စနေ`,
  and print the one that a number from 0 to 6 names, with a switch.
- Give the members of an enumeration text values, and print each value with a conversion.
- Write a switch with one arm for two members, and a default arm for the rest.
- Declare a struct with an enumeration field, and print it with no value set.
- Set the numbers of an enumeration from a constant, such as `(အခြေခံ + ၁)`.

Next: [Enumeration mistakes](mistakes.md).
