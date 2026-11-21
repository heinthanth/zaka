---
title: Members that hold data
description: Let each member of an enumeration hold a value of its own data kind, then test the member, read its data, and name the data in a switch arm.
---

# Members that hold data

Each member (variant) of an enumeration (enum) can define what data (payload) it holds.
For a shape, a circle can hold a radius, a rectangle can hold a width and a length, and
a point can hold no data. The program supplies these values when it creates a shape.
This form is also called a tagged union.

## You will learn

- how to declare a member that holds data, and how to make a value of it
- how to test which member a value is
- how to read the data of a member
- how to give the data a name in a conditional or a switch arm
- how to leave the data kind open with a generic enumeration

```zaka
ပုံ ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ဒေတာအနေဖြင့် ကိန်းပြည့် တစ်ခု ရှိသည့် စက်ဝိုင်း နှင့် အမှတ် တို့ပါဝင်သည်။

ကိန်းရှင် ပ သည် ပုံများထဲမှ စက်ဝိုင်း(၅) ဖြစ်ပါစေ။
ပ ကို ပြသမည်။
```

```text
ပုံများထဲမှ စက်ဝိုင်း(၅)
```

The enumeration `ပုံ` (shape) has two members: `စက်ဝိုင်း` (circle), which holds an
integer, and `အမှတ်` (point), which holds nothing. We put the data kind of a member in
front of its name with this pattern:

```text
ဒေတာအနေဖြင့် <kind> တစ်<word> ရှိသည့် <member>
```

| Zaka | Reads as |
| --- | --- |
| `ဒေတာအနေဖြင့်` | "as its data" |
| `ကိန်းပြည့် တစ်ခု ရှိသည့်` | "that has an integer" |
| `ပုံများထဲမှ စက်ဝိုင်း(၅)` | "circle from among the shapes, with 5" |

A member with data is written with the data in parentheses after it, and it prints in the
same form, so the output is valid Zaka.

## Making a value

```text
<name>များထဲမှ <member>(<data>)
ဒေတာအနေဖြင့် <data> ရှိသော <name>များထဲမှ <member>
```

We can give the data in parentheses after the member, or in words before it, in the same
shape as a struct literal with one field named `ဒေတာ`. A member that holds a struct takes
a struct value as its data, so a member can hold several values at once. A member that
holds no data is written as in a plain enumeration:

```zaka
အရွယ် ဟူသော ဒေတာအစုအဝေးတွင် အနံ ဟူသော ကိန်းပြည့် တစ်ခု နှင့် အလျား ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။
ပုံ ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ဒေတာအနေဖြင့် ကိန်းပြည့် တစ်ခု ရှိသည့် စက်ဝိုင်း၊ ဒေတာအနေဖြင့် အရွယ် တစ်ခု ရှိသည့် စတုဂံ နှင့် အမှတ် တို့ပါဝင်သည်။

ကိန်းရှင် က သည် ဒေတာအနေဖြင့် ၃ ရှိသော ပုံများထဲမှ စက်ဝိုင်း ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ပုံများထဲမှ စတုဂံ(အရွယ်(အနံ: ၂၊ အလျား: ၄)) ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် ပုံများထဲမှ အမှတ် ဖြစ်ပါစေ။
က ကို ပြသမည်။
ခ ကို ပြသမည်။
ဂ ကို ပြသမည်။
က == ပုံများထဲမှ စက်ဝိုင်း(၃) ကို ပြသမည်။
```

```text
ပုံများထဲမှ စက်ဝိုင်း(၃)
ပုံများထဲမှ စတုဂံ(အရွယ်(အနံ: ၂၊ အလျား: ၄))
ပုံများထဲမှ အမှတ်
မှန်
```

Two values are equal with `==` when both are the same member and hold equal data.

## Testing a member

```text
<value> သည် <member> အဖြစ်သို့ ပြောင်းလဲနိုင်သည်
```

To ask which member a value is, we write the test `သည် <member> အဖြစ်သို့
ပြောင်းလဲနိုင်သည်` ("can become `<member>`"). It is a true/false value (boolean), and
its verb takes the same endings as a comparison: `ပြောင်းလဲနိုင်လျှင်` in a conditional,
and `ပြောင်းလဲနိုင်သည် မဟုတ်` for the opposite:

```zaka
ပုံ ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ဒေတာအနေဖြင့် ကိန်းပြည့် တစ်ခု ရှိသည့် စက်ဝိုင်း နှင့် အမှတ် တို့ပါဝင်သည်။

ကိန်းရှင် ပ သည် ပုံများထဲမှ စက်ဝိုင်း(၅) ဖြစ်ပါစေ။
ပ သည် စက်ဝိုင်း အဖြစ်သို့ ပြောင်းလဲနိုင်သည် ဟု ပြသမည်။
အကယ်၍ ပ သည် အမှတ် အဖြစ်သို့ ပြောင်းလဲနိုင်သည် မဟုတ်လျှင် "အမှတ် မဟုတ်" ဟု ပြသမည်။
```

```text
မှန်
အမှတ် မဟုတ်
```

## Reading the data

```text
<value> ကို <member> အဖြစ် ပြောင်းလဲသော ရလဒ်
```

To read the data of a member, we convert the value to the member, as we convert a value to
a data kind, but with the member name and no measure word. The result is a duplicate of
the data. When the value is another member, the program stops:

```zaka
ပုံ ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ဒေတာအနေဖြင့် ကိန်းပြည့် တစ်ခု ရှိသည့် စက်ဝိုင်း နှင့် အမှတ် တို့ပါဝင်သည်။

ကိန်းရှင် ပ သည် ပုံများထဲမှ စက်ဝိုင်း(၅) ဖြစ်ပါစေ။
ကိန်းရှင် ရ သည် ပ ကို စက်ဝိုင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
ရ * ၂ ကို ပြသမည်။
```

```text
၁၀
```

```zaka-stop
ပုံ ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ဒေတာအနေဖြင့် ကိန်းပြည့် တစ်ခု ရှိသည့် စက်ဝိုင်း နှင့် အမှတ် တို့ပါဝင်သည်။

ကိန်းရှင် ပ သည် ပုံများထဲမှ အမှတ် ဖြစ်ပါစေ။
ပ ကို စက်ဝိုင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
```

```text
the program stopped: the value is another member than the one read
```

## Naming the data

```text
<value> သည် <name> ဟူသည့် <member> အဖြစ်သို့ ပြောင်းလဲနိုင်လျှင် <block>
```

A test can give the data a name, with `<name> ဟူသည့်` in front of the member. The name
holds a duplicate of the data in the block the test guards, so we read the data and test the
member in one step. This works as the condition of `အကယ်၍`, and as an arm of a switch,
where the later arms leave out the value, as switch arms do. An arm can also name a member
with `ဖြစ်လျှင်` alone, when it needs no data:

```zaka
အရွယ် ဟူသော ဒေတာအစုအဝေးတွင် အနံ ဟူသော ကိန်းပြည့် တစ်ခု နှင့် အလျား ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။
ပုံ ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ဒေတာအနေဖြင့် ကိန်းပြည့် တစ်ခု ရှိသည့် စက်ဝိုင်း၊ ဒေတာအနေဖြင့် အရွယ် တစ်ခု ရှိသည့် စတုဂံ နှင့် အမှတ် တို့ပါဝင်သည်။

ဧရိယာ ဟူသော လုပ်ဆောင်ချက် သည်
ပ ဟူသော ပုံ တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ပ သည် ရ ဟူသည့် စက်ဝိုင်း အဖြစ်သို့ ပြောင်းလဲနိုင်လျှင် ရ * ရ * ၃ ကို ပြန်ပေးမည်။
စ ဟူသည့် စတုဂံ အဖြစ်သို့ ပြောင်းလဲနိုင်လျှင် (စ ၏ အနံ) * (စ ၏ အလျား) ကို ပြန်ပေးမည်။
အမှတ် ဖြစ်လျှင် ၀ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဧရိယာ(ပုံများထဲမှ စက်ဝိုင်း(၂)) ကို ပြသမည်။
ဧရိယာ(ပုံများထဲမှ စတုဂံ(အရွယ်(အနံ: ၂၊ အလျား: ၄))) ကို ပြသမည်။
ကိန်းရှင် ပ သည် ပုံများထဲမှ စက်ဝိုင်း(၅) ဖြစ်ပါစေ။
အကယ်၍ ပ သည် ရ ဟူသည့် စက်ဝိုင်း အဖြစ်သို့ ပြောင်းလဲနိုင်လျှင် ရ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၂
၈
၅
```

A switch over such an enumeration names each member in its arms, or ends with a default
arm, as for each enumeration.

## A generic enumeration

```text
<name> ဟူသော ဒေတာအမျိုးအစားကွဲသည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင် …
```

An enumeration can leave the data kind of its members open with a kind parameter (type
parameter), as a [generic struct](../generics/README.md) does. A value works out the data
kind from its data, or from the place it goes into, and prints with the data kind in the
name of the enumeration:

```zaka
ဖြစ်နိုင် ဟူသော ဒေတာအမျိုးအစားကွဲသည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင် ဒေတာအနေဖြင့် T တစ်ခု ရှိသည့် တွေ့ နှင့် မတွေ့ တို့ပါဝင်သည်။

ကိန်းရှင် က သည် ဖြစ်နိုင်များထဲမှ တွေ့(၅) ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ဖြစ်နိုင်<စာသား> တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဖြစ်နိုင်များထဲမှ မတွေ့ ဖြစ်ပါစေ။
က ကို ပြသမည်။
ခ ကို ပြသမည်။
```

```text
ဖြစ်နိုင်<isize>များထဲမှ တွေ့(၅)
ဖြစ်နိုင်<text>များထဲမှ မတွေ့
```

When neither the data nor the place gives the data kind, we write it, as
`ဖြစ်နိုင်<စာသား>များထဲမှ မတွေ့` or `စာသား ကို အစားထိုးအသုံးပြုသည့် ဖြစ်နိုင်များထဲမှ
မတွေ့` ([Generic enumerations](../../manual/language/enums.md#generic-enumerations)).

## Details

- An enumeration gives its members constant values or lets them hold data, not both
  (ZK0353). A member that holds data is written with its data (ZK0354), and a member that
  holds none takes no data, read or name (ZK0355).
- A name with `ဟူသည့်` stands in the whole condition of `အကယ်၍` or in a switch arm alone
  (ZK0357).
- Inside a block that tested the member of a variable, a read of another member is an
  error (ZK0358), since it would stop each time.
- A member can hold a [shared struct](../structs/objects.md) that holds the enumeration
  again, which builds a tree, such as the parts of an expression.
- The marked [initial member](README.md#the-initial-member) gives the initial value, with
  the initial value of its data.
- [Enumerations](../../manual/language/enums.md#members-that-hold-data) in the Docs states
  the rules in full.

## Recap

- `ဒေတာအနေဖြင့် <kind> တစ်ခု ရှိသည့် <member>` lets a member hold data.
- `<name>များထဲမှ <member>(<data>)` makes a value, and prints in the same form.
- `<value> သည် <member> အဖြစ်သို့ ပြောင်းလဲနိုင်သည်` tests the member.
- `<value> ကို <member> အဖြစ် ပြောင်းလဲသော ရလဒ်` reads the data.
- `<name> ဟူသည့် <member>` names the data in a conditional or a switch arm.
- A generic enumeration leaves the data kind open, and each value works it out.

## Hands on

1. Add a member `တြိဂံ` (triangle) that holds an `အရွယ်` to `ပုံ`, and give it an arm in
   `ဧရိယာ` that returns half of the width times the length.
2. Write an enumeration `တိုကင်` (token) whose members hold an integer, text or nothing,
   and print each value of a list of tokens.

Next: [Enumeration mistakes](mistakes.md).
