---
title: Enumerations and switches
description: Name a short list of values as a new data kind, and pick one block for each value with a switch.
---

# Enumerations and switches

An *enumeration* (enum) is a data kind (type) whose values are a short list of names,
which we call its *members* (variants). A traffic light is red, amber or green and
nothing else, so an enumeration fits it well. To run different code for each member, we
use a *switch*.

## You will learn

- how to declare an enumeration and use one of its members
- how to give each member a value, and how to pick an initial member
- how to choose a block for each value with a switch
- how to catch the values that no case names with a default arm

```zaka
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ နှင့် စိမ်း တို့ပါဝင်သည်။

ကိန်းရှင် မီး သည် အရောင်များထဲမှ ဝါ ဖြစ်ပါစေ။
မီး ကို ပြသမည်။
```

```text
အရောင်များထဲမှ ဝါ
```

The first line declares an enumeration named `အရောင်` (color) with three members: `နီ`
(red), `ဝါ` (amber) and `စိမ်း` (green). We can use this pattern to declare an
enumeration, in the same form as a struct but with `ဒေတာအမျိုးအစားကွဲ` in place of
`ဒေတာအစုအဝေး`:

```text
<name> ဟူသော ဒေတာအမျိုးအစားကွဲတွင် <member>၊ <member> နှင့် <member> တို့ပါဝင်သည်။
```

| Zaka | Reads as |
| --- | --- |
| `အရောင် ဟူသော` | "called `အရောင်`" |
| `ဒေတာအမျိုးအစားကွဲ` | "a data kind of variants", the word for an enumeration |
| `တွင် … တို့ပါဝင်သည်` | "in it, these are included" |
| `အရောင်များထဲမှ ဝါ` | "amber, from among the colors" |

After the declaration, `အရောင်` is a data kind like `ကိန်းပြည့်`, so
`ကိန်းရှင် မီး သည် အရောင် တစ်ခု ဖြစ်ပါစေ။` declares a variable of it.

## Naming a member

```text
<name>များထဲမှ <member>
```

To use a member, we write the enumeration name, then `များ` (the plural mark), then `ထဲမှ`
("from among"), then the member. A member prints in this same form, and so does a template
slot such as `{မီး}` inside text, so the output is valid Zaka. We can compare two members
of the same enumeration with `==` and `!=`:

```zaka
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ နှင့် စိမ်း တို့ပါဝင်သည်။

ကိန်းရှင် မီး သည် အရောင်များထဲမှ စိမ်း ဖြစ်ပါစေ။
"မီး {မီး}" ဟု ပြသမည်။
မီး == အရောင်များထဲမှ စိမ်း ကို ပြသမည်။
```

```text
မီး အရောင်များထဲမှ စိမ်း
မှန်
```

Names in Latin letters work in the same form, and so do names that mix the two scripts,
because Zaka splits `Deliveryများထဲမှ` at `များ`:

```zaka
Delivery ဟူသော ဒေတာအမျိုးအစားကွဲတွင် Waiting၊ Moving နှင့် Arrived တို့ပါဝင်သည်။

ကိန်းရှင် state သည် Deliveryများထဲမှ Moving ဖြစ်ပါစေ။
state ကို ပြသမည်။
```

```text
Deliveryများထဲမှ Moving
```

## Member values

```text
တန်ဖိုးအနေဖြင့် <value> ရှိသည့် <member>
```

Each member has a value. When we write no values, the first member is 0 and each member
after it is one more than the member before it. To set the value of a member, we write
`တန်ဖိုးအနေဖြင့် <value> ရှိသည့်` ("that has `<value>` as its value") in front of it, and
the members after it count on from there. The value must be a constant that the compiler
can work out, such as a literal, a constant or an operation on them, and a conversion
returns the value of a member:

```zaka
ကိန်းသေ အခြေခံ သည် ၁၀၀ ဖြစ်ပါစေ။
အမှားများ ဟူသော ဒေတာအမျိုးအစားကွဲတွင် တန်ဖိုးအနေဖြင့် အခြေခံ + ၄ ရှိသည့် မတွေ့၊ ပိတ်ထား နှင့် တန်ဖိုးအနေဖြင့် အခြေခံ * ၂ ရှိသည့် ပြည့်နေ တို့ပါဝင်သည်။

ကိန်းရှင် မ သည် အမှားများထဲမှ ပိတ်ထား ဖြစ်ပါစေ။
မ ကို ပြသမည်။
မ ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
```

```text
အမှားများထဲမှ ပိတ်ထား
၁၀၅
```

Here `မတွေ့` (not found) is 104, so `ပိတ်ထား` (closed) counts on to 105. The name
`အမှားများ` (errors) ends in `များ`, so a member is `အမှားများထဲမှ ပိတ်ထား`, without a
second `များ`.

The value of the first member sets the data kind of all member values, which can be an
integer, a decimal number, a character, text, a list or a struct. Just integers count on,
so for the other data kinds, we write the value of each member:

```zaka
အမိန့် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် တန်ဖိုးအနေဖြင့် "go" ရှိသည့် သွား နှင့် တန်ဖိုးအနေဖြင့် "stop" ရှိသည့် ရပ် တို့ပါဝင်သည်။

ကိန်းရှင် ညွှန်ကြား သည် အမိန့်များထဲမှ ရပ် ဖြစ်ပါစေ။
ညွှန်ကြား ကို ပြသမည်။
ညွှန်ကြား ကို စာသား တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
```

```text
အမိန့်များထဲမှ ရပ်
stop
```

## The initial member

```text
ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့် <member>
```

Zaka does not pick a member for a variable on its own, neither the first one nor the one
with the value 0. To give an enumeration an [initial value](../variables/README.md#the-initial-value)
(default value), we write `ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်` ("to be used as the initial
value") in front of one member, and before its value when it has one. `ကနဦးတန်ဖိုး` for the
enumeration is then that member, and so is each field or cell of that enumeration when a
struct or a list gets its initial value:

```zaka
အမိန့် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် တန်ဖိုးအနေဖြင့် "go" ရှိသည့် သွား နှင့် ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့် တန်ဖိုးအနေဖြင့် "stop" ရှိသည့် ရပ် တို့ပါဝင်သည်။
ခရီး ဟူသော ဒေတာအစုအဝေးတွင် အကွာ ဟူသော ကိန်းပြည့် တစ်ခု နှင့် ညွှန် ဟူသော အမိန့် တစ်ခု တို့ပါဝင်သည်။

ကိန်းရှင် ညွှန်ကြား သည် အမိန့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ညွှန်ကြား ကို ပြသမည်။
ကိန်းရှင် ခရီးစဉ် သည် ခရီး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ခရီးစဉ် ကို ပြသမည်။
```

```text
အမိန့်များထဲမှ ရပ်
ခရီး(အကွာ: ၀၊ ညွှန်: အမိန့်များထဲမှ ရပ်)
```

## Choosing with a switch

```text
<subject> သည် <case> ဖြစ်လျှင် <sentence>
<case> ဖြစ်လျှင် <sentence>
<case> နှင့် <case> တို့ ဖြစ်လျှင် <sentence>
```

A switch compares one value, its *subject*, with a list of cases, and runs the sentence of
the first case that equals it. Each line of a switch is an *arm*: we write the subject
with `သည်` in the first arm, and just the case in each arm after it:

```zaka
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ နှင့် စိမ်း တို့ပါဝင်သည်။

ကိန်းရှင် မီး သည် အရောင်များထဲမှ ဝါ ဖြစ်ပါစေ။
မီး သည် နီ ဖြစ်လျှင် "ရပ်" ဟု ပြသမည်။
ဝါ ဖြစ်လျှင် "ဖြည်းဖြည်း" ဟု ပြသမည်။
စိမ်း ဖြစ်လျှင် "သွား" ဟု ပြသမည်။
```

```text
ဖြည်းဖြည်း
```

| Zaka | Reads as |
| --- | --- |
| `မီး သည် နီ ဖြစ်လျှင်` | "if the light is red" |
| `ဝါ ဖြစ်လျှင်` | "if amber" |

The subject shows which enumeration the cases come from, so we can write a bare member
name such as `နီ` as a case, and `အရောင်များထဲမှ နီ ဖြစ်လျှင်` works too. One arm can
cover several cases, with `နှင့်` before the last case and `တို့` after it, and a block
can follow `ဖြစ်လျှင်` in place of a single sentence, the same as after `အကယ်၍`:

```zaka
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ နှင့် စိမ်း တို့ပါဝင်သည်။

ကိန်းရှင် မီး သည် အရောင်များထဲမှ စိမ်း ဖြစ်ပါစေ။
မီး သည် နီ ဖြစ်လျှင် "ရပ်" ဟု ပြသမည်။
ဝါ နှင့် စိမ်း တို့ ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မရပ်ပါ" ဟု ပြသမည်။
မီး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
မရပ်ပါ
အရောင်များထဲမှ စိမ်း
```

## The default arm

```text
တခြား <kind> တစ်ခု ဖြစ်လျှင် <sentence>
```

The *default arm*, `တခြား <kind> တစ်ခု ဖြစ်လျှင်` ("if another `<kind>`"), comes last and
runs when no case equals the subject. A switch over an enumeration must either name each
member in an arm or end with the default arm, so when we add a member to the enumeration
later, the compiler points at each switch that misses it.

The subject of a switch can also be a number, a character, text, a list or a struct, so
each data kind that works with `==`. Over a number or text, we can leave out the default
arm, and then a value that no case equals runs nothing:

```zaka
ကိန်းရှင် ရက် သည် ၆ ဖြစ်ပါစေ။
ရက် သည် ၀ ဖြစ်လျှင် "တနင်္ဂနွေ" ဟု ပြသမည်။
၆ ဖြစ်လျှင် "စနေ" ဟု ပြသမည်။
တခြား ကိန်းပြည့် တစ်ခု ဖြစ်လျှင် "ကြားရက်" ဟု ပြသမည်။

ကိန်းရှင် ညွှန်ကြား သည် "ရပ်" ဖြစ်ပါစေ။
ညွှန်ကြား သည် "သွား" ဖြစ်လျှင် "go" ဟု ပြသမည်။
"ရပ်" ဖြစ်လျှင် "stop" ဟု ပြသမည်။
```

```text
စနေ
stop
```

## Details

- An enumeration is declared at the top level of a file, the same as a struct, and the
  rest of the program can use it even above the declaration ([An enumeration declared
  inside a block](mistakes.md#an-enumeration-declared-inside-a-block)).
- `<` and `>` do not work on members, because colors have no order.
- A variable of an enumeration keeps the value of its member, so `ညွှန်ကြား` above keeps
  the text `"stop"`. No conversion goes back from a value to a member, and a member never
  equals a value of its data kind without a conversion first.
- An enumeration with no initial member has no initial value, so `ကနဦးတန်ဖိုး` for it is
  a mistake, and so is a second initial member ([No member marked as the initial
  one](mistakes.md#no-member-marked-as-the-initial-one)).
- When the enumeration name ends in `များ`, the default arm writes the name without it, as
  in `တခြား အမှား တစ်ခု ဖြစ်လျှင်`. The data kind in the default arm uses its measure word
  (classifier), which is `ခု` unless the enumeration gives another.
- The full rules are in [Enumerations](../../manual/language/enums.md), and the mistakes
  that the compiler catches are in [Enumeration mistakes](mistakes.md).

## Recap

- `<name> ဟူသော ဒေတာအမျိုးအစားကွဲတွင် … တို့ပါဝင်သည်။` declares an enumeration, and its
  name is a data kind from then on.
- `<name>များထဲမှ <member>` names a member, and a member prints in the same form.
- Members count from 0 unless we give them values with `တန်ဖိုးအနေဖြင့် … ရှိသည့်`, and
  `ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်` picks the initial member.
- A switch runs the first arm whose case equals the subject, and `တခြား … ဖြစ်လျှင်` catches
  the rest.

## Hands on

- Declare an enumeration that names the seven parts of a week, from `တနင်္ဂနွေ` to `စနေ`,
  and print the one that a number from 0 to 6 names, with a switch.
- Give the members of an enumeration text values, and print each value with a conversion.
- Write a switch with one arm for two members, and a default arm for the rest.
- Declare a struct with an enumeration field, and print it with no value set.
- Set the numbers of an enumeration from a constant, such as `(အခြေခံ + ၁)`.

Next: [Members that hold data](data.md).
