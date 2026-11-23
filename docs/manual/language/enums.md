---
title: Enumerations
description: The enumeration declaration, member names and values, the initial member, comparing, converting and printing members, and the switch statement.
---

# Enumerations

An enumeration (enum, tagged union) is a data kind (type) whose values are a fixed list of
named members (variants). Each member has a constant value, which is an `isize` unless the
first member gives a value of another data kind. An enumeration can instead let its
members hold data of their own, a value that the program sets as it runs, and then it is a
tagged union ([Members that hold data](#members-that-hold-data)). The Learn book teaches
enumerations in small steps in [Enumerations and switches](../../learn/enums/README.md) and
[Members that hold data](../../learn/enums/data.md).

```zaka
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ နှင့် စိမ်း တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မီး သည် အရောင်များထဲမှ ဝါ ဖြစ်ပါစေ။
မီး ကို ပြသမည်။
မီး ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အရောင်များထဲမှ ဝါ
၁
```

## Declaring an enumeration

```text
<name> ဟူသော [အများသုံးနိုင်သော] ဒေတာအမျိုးအစားကွဲတွင် <member>၊ … နှင့် <member> တို့ပါဝင်သည်။
<name> ဟူသော [အများသုံးနိုင်သော] ဒေတာအမျိုးအစားကွဲတွင် <member> နှင့် … <member> တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ <word> ဖြစ်သည်။
<name> ဟူသော [အများသုံးနိုင်သော] ဒေတာအမျိုးအစားကွဲတွင် <member> ပါဝင်သည်။

member := [ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်] [တန်ဖိုးအနေဖြင့် <value> ရှိသည့်] <name>
```

To declare an enumeration, we write its name, `ဟူသော`, the word `ဒေတာအမျိုးအစားကွဲတွင်` and
its members, in the same shape as a [struct](structs.md#declaring-a-struct) with
`ဒေတာအမျိုးအစားကွဲ` in place of `ဒေတာအစုအဝေး`. The members form a [list in
words](words.md#lists-in-words), so we put `၊` between the names and `နှင့်` before the
last one, and we end the list with `တို့ပါဝင်သည်`, or with `ပါဝင်သည်` after a single
member. To give the enumeration a measure word (classifier), as a struct can, we end with
`တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ <word> ဖြစ်သည်`, or with `ပါဝင်ပြီး …` after a single member.
Without one, the enumeration is counted with `ခု` ([Measure words](kinds.md#measure-words)).

An enumeration needs at least one member. We declare it at the top level of a file, and the
rest of the program can use it even above the declaration, because the compiler reads all
declarations first. `အချက်အလက်အမျိုးအစားကွဲ` is the same word as `ဒေတာအမျိုးအစားကွဲ`, as
`အချက်အလက်` and `ဒေတာ` are, and we can join `တွင်` to the word or write it apart. To export
an enumeration, we put `အများသုံးနိုင်သော` after `ဟူသော`, as for a struct ([Modules](modules.md)).

| Mistake | Code |
| --- | --- |
| `တို့` after a single member, or no `တို့` after several | ZK0201 |
| a list of members with no `နှင့်` before the last one | ZK0245 |
| a declaration inside a block, a loop, a conditional or a function | ZK0235 |
| two members of one enumeration with the same name | ZK0328 |
| a name declared twice, as an enumeration or a struct, which share one set of names | ZK0316 |
| an enumeration name that also names a variable, a constant, a parameter, a field or a function | ZK0319 |
| a name in the place of a data kind that names no enumeration and no struct | ZK0318 |

### Member values

```text
တန်ဖိုးအနေဖြင့် <value> ရှိသည့် <member>
```

```zaka
ကိန်းသေ အခြေခံ သည် ၁၀၀ ဖြစ်ပါစေ။
အမှားများ ဟူသော ဒေတာအမျိုးအစားကွဲတွင်
တန်ဖိုးအနေဖြင့် အခြေခံ + ၄ ရှိသည့် မတွေ့၊
ပိတ်ထား နှင့်
တန်ဖိုးအနေဖြင့် အခြေခံ * ၂ ရှိသည့် ပြည့်နေ
တို့ပါဝင်သည်။
အမိန့် ဟူသော ဒေတာအမျိုးအစားကွဲတွင်
တန်ဖိုးအနေဖြင့် "go" ရှိသည့် သွား နှင့်
တန်ဖိုးအနေဖြင့် "stop" ရှိသည့် ရပ်
တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အမှားများထဲမှ ပိတ်ထား ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
အမိန့်များထဲမှ ရပ် ကို စာသား တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၀၅
stop
```

To give a member a value, we write `တန်ဖိုးအနေဖြင့် <value> ရှိသည့်` ("that has `<value>`
as its value") before its name. The value can be each expression that the folder can work
out at compile time: a literal, a constant, even one declared below the enumeration, or an
operation on them. A value that cannot be worked out, such as a variable, is ZK0456.

The value of the first member sets the *value kind* of the enumeration, which is the data
kind of all member values: a number, a character, a true/false value, text, a list or a
struct. When the first member has no value, the value kind is `isize`. Each other value
converts to the value kind, as the value of a declaration does (ZK0401).

- When the value kind is an integer, a member with no value gets the value of the member
  before it plus one, and the first member gets 0. A member with no value that cannot count
  on is ZK0461: its value kind is not an integer, or the next value would go past the
  largest value of the integer kind. In both cases, we write the value.
- Two members of one enumeration must have different values (ZK0457), so that printing
  and a switch can tell the members apart.
- Text values work out as all constant text does: a literal, a text constant, a template
  whose slots work out, written as a print writes them, or text joined with `+` or
  `ဆက်ထားသည့် စာသား`. A list or a struct can contain such text.

## The initial member

```text
ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့် [တန်ဖိုးအနေဖြင့် <value> ရှိသည့်] <member>
```

No member of an enumeration is a default on its own. To choose one, we put the mark
`ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်` ("to be used as the initial value") before its name and its
value, and that member becomes the [initial value](kinds.md#initial-values) (default
value) of the data kind. It is the member that `ကနဦးတန်ဖိုး` gives, and the one that a
struct or a counted list of the enumeration gets when it is filled with `ကနဦးတန်ဖိုး`.

```text
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် တန်ဖိုးအနေဖြင့် ၁ ရှိသည့် နီ၊ ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့် တန်ဖိုးအနေဖြင့် ၀ ရှိသည့် စိမ်း နှင့် တန်ဖိုးအနေဖြင့် ၂ ရှိသည့် ပြာ တို့ပါဝင်သည်။
```

- We can mark a single member, and a second mark is ZK0480. When the member also has a
  value, the mark comes first and the value after it.
- The mark changes no value and no order, so a member with no value still counts on from
  the member before it, and the first member still sets the value kind.
- An enumeration with no marked member has no initial value, so `ကနဦးတန်ဖိုး` for it, or
  for a struct or a list that contains it, is ZK0479.
- `ကနဦးတန်ဖိုး` is a reserved word, so no member can use it as a name.

```zaka
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့် ဝါ နှင့် စိမ်း တို့ပါဝင်သည်။
မီးပွိုင့် ဟူသော ဒေတာအစုအဝေးတွင်
အမှတ် ဟူသော ကိန်းပြည့် တစ်ခု နှင့်
ရောင် ဟူသော အရောင် တစ်ခု
တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မီး သည် အရောင် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
မီး ကို ပြသမည်။
မီး ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
ကိန်းရှင် ပ သည် မီးပွိုင့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ပ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အရောင်များထဲမှ ဝါ
၁
မီးပွိုင့်(အမှတ်: ၀၊ ရောင်: အရောင်များထဲမှ ဝါ)
```

```zaka-error ZK0480
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့် နီ နှင့် ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့် စိမ်း တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0480]: "အရောင်" marks a second member with "ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်"
```

```zaka-error ZK0479
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ နှင့် စိမ်း တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မီး သည် အရောင် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0479]: "အရောင်" has no initial value: no member of "အရောင်" is marked "ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်"
```

## The name and its stem

The *stem* of an enumeration name is the name with a final `များ`, the plural ending, taken
off: `အရောင်` for `အရောင်`, and `အမှား` for `အမှားများ`. The stem names the enumeration
too, so `ကိန်းရှင် မ သည် အမှား တစ်ခု ဖြစ်ပါစေ။` declares a variable of `အမှားများ`. The stem
also counts as a name for ZK0316 and ZK0319, so no other struct, enumeration or value can
use it.

## Members

```text
<stem>များထဲမှ <member>
```

To refer to a member, we write the stem, `များထဲမှ` and the member name, which means
"`<member>` from among the `<stem>`s", as in `အရောင်များထဲမှ နီ` and `အမှားများထဲမှ မတွေ့`.

- Before `ထဲမှ` we write the stem with `များ` after it, and nothing else. A name that is not
  an enumeration, or an enumeration written without its `များ`, is ZK0330, and the help
  line of the message gives the right spelling.
- A member that the enumeration does not have is ZK0329.
- We can write `ထဲမှ` apart from the name or joined to it, so `အရောင်များ ထဲမှ နီ` and
  `အရောင်များထဲမှ နီ` are the same. The lexer splits a name written joined to `ထဲမှ`, so no
  name in a program can end in `ထဲမှ`.
- A member is a constant, so it can be the value of a `ကိန်းသေ`, and the folder works it
  out at compile time.

A name in Latin letters, or in mixed scripts, splits from `ထဲမှ` the same as a Burmese
name:

```zaka
Delivery ဟူသော ဒေတာအမျိုးအစားကွဲတွင် Waiting၊ Moving နှင့် Arrived တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် state သည် Deliveryများထဲမှ Moving ဖြစ်ပါစေ။
state ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
Deliveryများထဲမှ Moving
```

## Values of an enumeration

A value of an enumeration keeps the value of its member in the same form as a value of the
value kind, with the same size and arrangement, and a text member keeps static text. A
member is duplicated, never moved, and has no address, and a field, a cell, a parameter
and a function result can each contain one.

- We can use `==` and `!=` on two members of one enumeration, which compares their values.
  A member and a value of another data kind, or members of two different enumerations,
  cannot be compared (ZK0404), and `<`, `>` and the other ordering operators do not work
  on members (ZK0403).
- The conversion `<value> ကို <kind> တစ်<word> အဖြစ် ပြောင်းလဲသော ရလဒ်` turns a member into
  its value. It converts to the value kind, and when the value is a number, a character or
  a true/false value, also to each data kind that the value converts to
  ([Conversions](conversions.md)). There is no conversion from a value back to a member.
- A variable of an enumeration declared with no value has no member until the program sets
  it ([Set before read](declarations.md#set-before-read)). No member is a default, and
  `ကနဦးတန်ဖိုး` gives the member that the declaration marks ([The initial
  member](#the-initial-member)). A heap block starts at zero octets (bytes), as for all
  data kinds, and those octets might match no member at all.
- When we print a member, with `ပြသမည်` or in a template slot, the output is
  `<stem>များထဲမှ <member>`, which is also the form that we write in code. Inside a list or
  a struct, a member prints the same. A value that matches no member, from a heap block
  that starts at zero octets, prints as its value.

```zaka
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ နှင့် စိမ်း တို့ပါဝင်သည်။
မီးပွိုင့် ဟူသော ဒေတာအစုအဝေးတွင်
အမှတ် ဟူသော ကိန်းပြည့် တစ်ခု နှင့်
ရောင် ဟူသော အရောင် တစ်ခု
တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပွိုင့် သည် မီးပွိုင့်(အမှတ်: ၃၊ ရောင်: အရောင်များထဲမှ စိမ်း) ဖြစ်ပါစေ။
ပွိုင့် ကို ပြသမည်။
[အရောင်များထဲမှ နီ၊ အရောင်များထဲမှ စိမ်း] ကို ပြသမည်။
"ယခု {ပွိုင့် ၏ ရောင်}" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မီးပွိုင့်(အမှတ်: ၃၊ ရောင်: အရောင်များထဲမှ စိမ်း)
[အရောင်များထဲမှ နီ၊ အရောင်များထဲမှ စိမ်း]
ယခု အရောင်များထဲမှ စိမ်း
```

## Members that hold data

```text
member := [ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်] [ဒေတာအနေဖြင့် <kind> တစ်<word> ရှိသည့်] <name>
```

A member can hold data (payload): a value of its own data kind, which the program sets when it makes
a value of the member. We write `ဒေတာအနေဖြင့် <kind> တစ်<word> ရှိသည့်` ("that has a
`<kind>` as its data") before the name of each member that holds data, and leave it out for
a member that holds none. Each member holds one value at the most; a member that needs
several holds a struct.

```zaka
အရွယ် ဟူသော ဒေတာအစုအဝေးတွင်
အနံ ဟူသော ကိန်းပြည့် တစ်ခု နှင့်
အလျား ဟူသော ကိန်းပြည့် တစ်ခု
တို့ပါဝင်သည်။
ပုံ ဟူသော ဒေတာအမျိုးအစားကွဲတွင်
ဒေတာအနေဖြင့် ကိန်းပြည့် တစ်ခု ရှိသည့် စက်ဝိုင်း၊
ဒေတာအနေဖြင့် အရွယ် တစ်ခု ရှိသည့် စတုဂံ နှင့်
အမှတ်
တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ပုံများထဲမှ စက်ဝိုင်း(၃) ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ဒေတာအနေဖြင့် အရွယ်(အနံ: ၂၊ အလျား: ၄) ရှိသော ပုံများထဲမှ စတုဂံ ဖြစ်ပါစေ။
[က၊ ခ၊ ပုံများထဲမှ အမှတ်] ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[ပုံများထဲမှ စက်ဝိုင်း(၃)၊ ပုံများထဲမှ စတုဂံ(အရွယ်(အနံ: ၂၊ အလျား: ၄))၊ ပုံများထဲမှ အမှတ်]
```

- An enumeration gives its members constant values or lets them hold data, not both
  (ZK0353). The data kind can be each data kind a struct field can have.
- We make a value of a member that holds data with its data in parentheses,
  `<stem>များထဲမှ <member>(<data>)`, or in words before it,
  `ဒေတာအနေဖြင့် <data> ရှိသော <stem>များထဲမှ <member>`, in the shape of a struct literal
  with one field. The value owns a duplicate of text it reads from a place, as a field does.
  A member that holds data written with none is ZK0354, and data for a member that holds
  none is ZK0355.
- A value prints as `<stem>များထဲမှ <member>(<data>)`, and as `<stem>များထဲမှ <member>` for a
  member with no data, so the output reads back.
- `==` and `!=` compare the member and the data, which needs `==` on each data kind.
- A value of such an enumeration is stored as a struct the compiler makes: a tag, the
  position of its member, then one field for the data of each member that holds some, next
  to each other, as a result keeps its two parts. The fields of the other members hold zero.
  A member that holds the enumeration again, at once or through a struct, makes a kind
  with no end (ZK0427); a pointer or a [shared struct](objects.md#shared-objects) breaks
  the loop. A kind is built when one of its members can be built.
- The marked initial member gives the initial value, with the initial value of its data
  (ZK0479 when the data kind has none).
- A value can be a constant, and the folder works out its data.
- C has no form of such a value, so it does not go to a C function or come back from one
  (ZK0465).

## Testing a member

```text
<value> သည် <member> အဖြစ်သို့ ပြောင်းလဲနိုင်သည်
```

The test is a true/false value (boolean) that is true when the value is the member,
whatever its data. Its verb `ပြောင်းလဲနိုင်` takes the endings of a comparison verb:
`ပြောင်းလဲနိုင်လျှင်` before a block, `ပြောင်းလဲနိုင်သည် မဟုတ်` for the opposite, and the
other forms of [Natural wording](naturalness.md). The test works on each enumeration, on
members with values too. A value of another data kind is ZK0356, and a member the
enumeration does not have is ZK0329.

## Reading the data

```text
<value> ကို <member> အဖြစ် ပြောင်းလဲသော ရလဒ်
```

The read gives the data of the member, in the form of a conversion with the member name in
place of a data kind and no measure word. When the value is another member, the program
stops with a trap ([When a program stops](../vm/traps.md#a-read-of-the-data-of-another-member)).
A member that holds no data is ZK0355. Inside a block that a test of a variable guards,
with `ပြောင်းလဲနိုင်လျှင်` or as a switch arm, the variable is that member while nothing changes
it, so a read of another member there is ZK0358.

```zaka
ပုံ ဟူသော ဒေတာအမျိုးအစားကွဲတွင်
ဒေတာအနေဖြင့် ကိန်းပြည့် တစ်ခု ရှိသည့် စက်ဝိုင်း နှင့်
အမှတ်
တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ပုံများထဲမှ စက်ဝိုင်း(၅) ဖြစ်ပါစေ။
ပ သည် စက်ဝိုင်း အဖြစ်သို့ ပြောင်းလဲနိုင်သည် ဟု ပြသမည်။
ပ ကို စက်ဝိုင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
၅
```

```zaka-error ZK0358
ပုံ ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ဒေတာအနေဖြင့် ကိန်းပြည့် တစ်ခု ရှိသည့် စက်ဝိုင်း နှင့် အမှတ် တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ပုံများထဲမှ အမှတ် ဖြစ်ပါစေ။
အကယ်၍ ပ သည် အမှတ် အဖြစ်သို့ ပြောင်းလဲနိုင်လျှင် ပ ကို စက်ဝိုင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0358]: "ပ" is "အမှတ်" here, so reading it as "စက်ဝိုင်း" stops the run each time
```

## Naming the data

```text
<value> သည် <name> ဟူသည့် <member> အဖြစ်သို့ ပြောင်းလဲနိုင်လျှင် <block>
<name> ဟူသည့် <member> အဖြစ်သို့ ပြောင်းလဲနိုင်လျှင် <block>
```

A test can give the data of the member a name with `<name> ဟူသည့်`. The name holds a duplicate of
the data in the block that the test guards, in a scope of its own, and a write to it
changes the duplicate alone. The test with a name is the whole condition of `အကယ်၍` or an arm of
a [switch](#switch); in another place, such as a value or a condition joined with
`ဖြစ်၍`, it is ZK0357. A name for a member that holds no data is ZK0355.

The value of a test with a name runs once, even when it is a call, and so does the subject
of a switch.

## Generic enumerations

```text
<name> ဟူသော ဒေတာအမျိုးအစားကွဲသည် ယေဘုယျဒေတာအမျိုးအစား <param> ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင် …
<name><<param>၊ …> ဟူသော ဒေတာအမျိုးအစားကွဲတွင် …                     (sign form)
<kinds> (ကို|တို့ကို) အစားထိုးအသုံးပြုသည့် <stem>များထဲမှ <member>        (kinds of a member, words)
<stem><<kind>၊ …>များထဲမှ <member>                                   (kinds of a member, signs)
```

An enumeration whose members hold data can have kind parameters (type parameters), in the
forms of a [generic struct](generics.md#kind-parameters): the word form, with `သည်` after
`ဒေတာအမျိုးအစားကွဲ` and `၎င်းတွင်` before the members, or the sign form after the name. A
data kind of a member can then name a kind parameter. At a use, the data kinds go in the
forms of a generic struct, `ဖြစ်နိုင်<စာသား> တစ်ခု`, or in the words of its written-as hint.

```zaka
ဖြစ်နိုင် ဟူသော ဒေတာအမျိုးအစားကွဲသည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင်
ဒေတာအနေဖြင့် T တစ်ခု ရှိသည့် တွေ့ နှင့်
မတွေ့
တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ဖြစ်နိုင်များထဲမှ တွေ့(၅) ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ဖြစ်နိုင်<စာသား> တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဖြစ်နိုင်များထဲမှ မတွေ့ ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် စာသား ကို အစားထိုးအသုံးပြုသည့် ဖြစ်နိုင်များထဲမှ မတွေ့ ဖြစ်ပါစေ။
က ကို ပြသမည်။
ခ ကို ပြသမည်။
ဂ == ခ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဖြစ်နိုင်<isize>များထဲမှ တွေ့(၅)
ဖြစ်နိုင်<text>များထဲမှ မတွေ့
မှန်
```

- A generic enumeration lets its members hold data, so its members have no constant
  values (ZK0353).
- A member value takes the data kinds of the place it goes into, or works them out from its
  data. When neither gives them, as for a member with no data printed on its own, it
  writes them, in words before the member or in signs after the enumeration name; with
  none written, it is ZK0341, and another count of data kinds than the declaration has is
  ZK0340.
- Each set of data kinds makes an instance, a separate enumeration named with those data
  kinds, such as `ဖြစ်နိုင်<isize>`, and a value prints with that name before `များထဲမှ`, as
  a generic struct prints with its instance name.
- The test, the read and the arms of a switch work on an instance as on each enumeration.
  The default arm can name the generic alone: `တခြား ဖြစ်နိုင် တစ်ခု ဖြစ်လျှင်`.
- A member can hold a generic shared struct that holds the enumeration again, which builds
  a tree of each data kind. An instance that asks for deeper instances with no end stops
  at ZK0342, as for structs.

## Switch

```text
<subject> သည် <case> ဖြစ်လျှင် <block>
<case> နှင့် <case> တို့ ဖြစ်လျှင် <block>
တခြား <kind> တစ်<word> ဖြစ်လျှင် <block>
```

A switch (`switch`, `match`) compares its subject with the case of each arm, from top to
bottom, using `==`, and runs the block of the first arm whose case equals the subject.
When none does, it runs the block of the default arm `တခြား <kind> တစ်<word> ဖြစ်လျှင်`, or
nothing when there is no default arm. The switch works out the subject once.

- An arm has one case, or several with `၊` between them and `တို့` after the last, and it
  runs when the subject equals one of them. A list literal is one case in each of its
  forms: `[a၊ b]`, `a နှင့် b တို့ပါသော စာရင်း`, or either one in parentheses.
- The first arm names the subject with `သည်`, and each arm after it has a case alone. The
  block of an arm is one statement or a block, as in a conditional.
- The switch ends after its default arm, or at the first sentence that is not an arm. An
  arm is a sentence that reaches `ဖြစ်လျှင်` before its full stop, with no `သည်` before it
  and no statement word at its start, so a sentence with a `သည်` starts a new switch.
- The subject can be a value of each data kind that `==` works on: a number, a character,
  a true/false value, text, a list, a struct, a pointer or an enumeration. Each case is a
  value that `==` accepts together with the subject.
- In a switch over an enumeration, a case written as a bare name, such as `နီ`, is that
  member of the subject's enumeration, even when a variable has the same name. We can
  also write the full form `<stem>များထဲမှ <member>` as a case.
- In the default arm, we write the data kind of the subject with its measure word, and
  for an enumeration we can use its name or its stem: `တခြား အမှား တစ်ခု ဖြစ်လျှင်`.
- A switch over an enumeration must name each member in its cases or end with a default
  arm.
- An arm can test a member with `[<name> ဟူသည့်] <member> အဖြစ်သို့ ပြောင်းလဲနိုင်လျှင်`, the
  first arm with the subject and `သည်` in front, as [Naming the
  data](#naming-the-data) shows. Such an arm names one member.
- Over an enumeration whose members hold data, an arm compares the member alone, whatever
  its data. A case names a member, as a bare name or as `<stem>များထဲမှ <member>` with no
  data; a case with data is ZK0359. A case that the folder cannot work out, such as a variable, does not count as
  naming a member. Over other data kinds, we can leave out the default arm.

| Mistake | Code |
| --- | --- |
| a one-statement arm that is a conditional or another switch, which would make it unclear which switch the next arms belong to | ZK0215 |
| a case of a data kind that `==` does not accept with the subject | the error of `==`, such as ZK0404 |
| a bare name that is neither a member nor a variable | ZK0329 |
| two cases that fold to one value, so the second arm could never run | ZK0460 |
| a default arm whose data kind is not the data kind of the subject | ZK0459 |
| a default arm that is not the last arm | ZK0236 |
| a switch over an enumeration that misses a member and has no default arm | ZK0458 |

```zaka
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ နှင့် စိမ်း တို့ပါဝင်သည်။

လုပ်ရန် ဟူသော လုပ်ဆောင်ချက် သည်
ယခု ဟူသော အရောင် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
ယခု သည် နီ ဖြစ်လျှင် "ရပ်" ကို ပြန်ပေးမည်။
ဝါ ဖြစ်လျှင် "ဖြည်းဖြည်း" ကို ပြန်ပေးမည်။
တခြား အရောင် တစ်ခု ဖြစ်လျှင် "သွား" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ရန် ဟူသော လုပ်ဆောင်ချက်ကို အရောင်များထဲမှ စိမ်း ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ကိန်းရှင် အက္ခရာ သည် 'ခ' ဖြစ်ပါစေ။
အက္ခရာ သည် 'က' ဖြစ်လျှင် "ပထမ" ဟု ပြသမည်။
'ခ' ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ဒုတိယ" ဟု ပြသမည်။
အက္ခရာ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
တခြား စာလုံး တစ်လုံး ဖြစ်လျှင် "အခြား" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
သွား
ဒုတိယ
ခ
```

A switch lowers to the same chain of tests as a conditional, with one `==` for each arm, so
it costs the same as the conditional ([The IR](../internals/codegen.md)).

## Errors at a glance

| Code | Problem |
| --- | --- |
| ZK0201 | `တို့` after a single member, or no `တို့` after several |
| ZK0215 | a one-statement arm that is a conditional or another switch |
| ZK0235 | an enumeration declared inside a block |
| ZK0236 | a default arm that is not the last arm |
| ZK0245 | a list of members with no `နှင့်` before the last one |
| ZK0316 | a name declared twice, as an enumeration or a struct |
| ZK0318 | a name in the place of a data kind that names no enumeration and no struct |
| ZK0319 | an enumeration name that also names a value or a function |
| ZK0328 | two members of one enumeration with the same name |
| ZK0329 | a member that the enumeration does not have |
| ZK0330 | a name before `ထဲမှ` that is not an enumeration, or one without its `များ` |
| ZK0353 | an enumeration that gives some members values and others data |
| ZK0354 | a member that holds data written with none |
| ZK0355 | data given to, read from or named for a member that holds none |
| ZK0356 | a test or a read of a member on a value that is no enumeration |
| ZK0357 | a name for the data of a member outside the condition of `အကယ်၍` or a switch arm |
| ZK0358 | a read of a member that a test ruled out |
| ZK0359 | a case of a switch over members with data that gives data |
| ZK0403 | an ordering operator on members |
| ZK0427 | a member that holds its own enumeration, at once or through a struct |
| ZK0404 | members of two enumerations, or a member and another value, compared |
| ZK0456 | a member value that the folder cannot work out |
| ZK0457 | two members with the same value |
| ZK0458 | a switch over an enumeration that misses a member and has no default arm |
| ZK0459 | a default arm whose data kind is not the data kind of the subject |
| ZK0460 | two cases that fold to one value |
| ZK0461 | a member with no value that cannot count on from the member before it |
| ZK0479 | `ကနဦးတန်ဖိုး` for an enumeration with no initial member |
| ZK0480 | a second member marked as the initial member |
