---
title: Enumerations and switches
description: Name a short list of values as a kind of its own, and pick one block for each value with a switch.
---

# Enumerations and switches

An *enumeration* is a kind whose values are a short list of names, its *members*. A
traffic light is red, amber or green, and nothing else:

```zaka
အမျိုးအစား အရောင် ဆိုသည်မှာ နီ၊ ဝါ၊ စိမ်း တို့ ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မီး သည် အရောင်များထဲမှ ဝါ ဖြစ်ပါစေ။
မီး ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အရောင်များထဲမှ ဝါ
```

| Zaka | Reads as |
| --- | --- |
| `အမျိုးအစား အရောင်` | "the kind `အရောင်` (color)" |
| `ဆိုသည်မှာ ... တို့ ဖြစ်သည်` | "means these" |
| `အရောင်များထဲမှ ဝါ` | "amber, from among the colors" |

An enumeration is declared at the top level of a file, as a struct is, and the rest of
the program can use it from the first line. Its name is a kind from then on:
`ကိန်းရှင် မီး သည် အရောင် တစ်ခု ဖြစ်ပါစေ။` declares a variable of it.

## Naming a member

A member is written `<name>များထဲမှ <member>`: the enumeration name, then `များ`, the
plural mark, then `ထဲမှ`, "from among". A name that ends in `များ` itself, as `အမှားများ`
does, does not take a second one: `အမှားများထဲမှ ပြည့်နေ`.

Printing a member writes it in that same form, so a printed member reads back as Zaka.
Inside text, a template slot such as `{မီး}` writes it so too:

```zaka
အမျိုးအစား အရောင် ဆိုသည်မှာ နီ၊ ဝါ၊ စိမ်း တို့ ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မီး သည် အရောင်များထဲမှ စိမ်း ဖြစ်ပါစေ။
"မီး {မီး}" ဟု ပြသပါ။
မီး == အရောင်များထဲမှ စိမ်း ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မီး အရောင်များထဲမှ စိမ်း
မှန်
```

Two members of one enumeration compare with `==` and `!=`. `<` and `>` do not take
members: colors have no order.

## Member values

Each member stands for a value. With none written, the first member is 0 and each member
after it is one more than the member before. A value in parentheses after a member sets
it, and the members after it count on from there. A value can be each constant the
compiler works out: a literal, a constant, or an operation on them. A conversion gives
the value of a member:

```zaka
ကိန်းသေ အခြေခံ သည် ၁၀၀ ဖြစ်ပါစေ။
အမျိုးအစား အမှားများ ဆိုသည်မှာ မတွေ့ (အခြေခံ + ၄)၊ ပိတ်ထား၊ ပြည့်နေ (အခြေခံ * ၂) တို့ ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မ သည် အမှားများထဲမှ ပိတ်ထား ဖြစ်ပါစေ။
မ ကို ပြသပါ။
မ ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
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
အမျိုးအစား အမိန့် ဆိုသည်မှာ သွား ("go")၊ ရပ် ("stop") တို့ ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ညွှန်ကြား သည် အမိန့်များထဲမှ ရပ် ဖြစ်ပါစေ။
ညွှန်ကြား ကို ပြသပါ။
ညွှန်ကြား ကို စာသား အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အမိန့်များထဲမှ ရပ်
stop
```

A variable of an enumeration holds the value of its member, so `ညွှန်ကြား` above holds the
text `"stop"`. No conversion goes back from a value to a member, and a member never equals
a value of its kind without a conversion first.

## Starting values

A variable declared with no value starts at the first member. A struct or a list with an
enumeration inside starts each field and each cell at its first member too, since a
struct starts each field at the starting value of its kind:

```zaka
အမျိုးအစား အမိန့် ဆိုသည်မှာ သွား ("go")၊ ရပ် ("stop") တို့ ဖြစ်သည်။
ဒေတာအမျိုးအစား ခရီး တွင် ကိန်းပြည့် အကွာ၊ အမိန့် ညွှန် တို့ ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ညွှန်ကြား သည် အမိန့် တစ်ခု ဖြစ်ပါစေ။
ညွှန်ကြား ကို ပြသပါ။
ကိန်းရှင် ခရီးစဉ် သည် ခရီး တစ်ခု ဖြစ်ပါစေ။
ခရီးစဉ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အမိန့်များထဲမှ သွား
ခရီး(အကွာ: ၀၊ ညွှန်: အမိန့်များထဲမှ သွား)
```

## Choosing with a switch

A *switch* compares one value, its *subject*, with a list of cases, and runs the block of
the first case that equals it:

```zaka
အမျိုးအစား အရောင် ဆိုသည်မှာ နီ၊ ဝါ၊ စိမ်း တို့ ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မီး သည် အရောင်များထဲမှ ဝါ ဖြစ်ပါစေ။
မီး သည် နီ ဖြစ်လျှင် "ရပ်" ဟု ပြသပါ။
ဝါ ဖြစ်လျှင် "ဖြည်းဖြည်း" ဟု ပြသပါ။
စိမ်း ဖြစ်လျှင် "သွား" ဟု ပြသပါ။
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

`တခြား <kind> ဖြစ်လျှင်`, "if another `<kind>`", comes last and runs when no case equals
the subject. Over an enumeration, a switch either names each member in an arm or ends with
the default arm, so a member added to the enumeration later is caught at each switch that
forgets it. For an enumeration whose name ends in `များ`, the default arm writes the name
without it: `တခြား အမှား ဖြစ်လျှင်`.

A switch takes a subject of each kind that `==` takes: a number, a character, text, a
list, a struct or an enumeration. Over a number or text, the default arm can be left out;
then a value that no case equals runs nothing.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရက် သည် ၆ ဖြစ်ပါစေ။
ရက် သည် ၀ ဖြစ်လျှင် "တနင်္ဂနွေ" ဟု ပြသပါ။
၆ ဖြစ်လျှင် "စနေ" ဟု ပြသပါ။
တခြား ကိန်းပြည့် ဖြစ်လျှင် "ကြားရက်" ဟု ပြသပါ။

ကိန်းရှင် ညွှန်ကြား သည် "ရပ်" ဖြစ်ပါစေ။
ညွှန်ကြား သည် "သွား" ဖြစ်လျှင် "go" ဟု ပြသပါ။
"ရပ်" ဖြစ်လျှင် "stop" ဟု ပြသပါ။
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
