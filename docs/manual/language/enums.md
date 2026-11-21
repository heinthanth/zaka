---
title: Enumerations
description: The enumeration declaration, member names and values, the initial member, comparing, converting and printing members, and the switch statement.
---

# Enumerations

An enumeration (enum, tagged union) is a data kind (type) whose values are a fixed list of
named members (variants). Each member has a constant value, which is an `isize` unless the
first member gives a value of another data kind. Members have no fields, so a Zaka
enumeration is more like an `enum` of C than a tagged union. The Learn book teaches
enumerations in small steps in [Enumerations and switches](../../learn/enums/README.md).

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
<name> ဟူသော [အများသုံး] ဒေတာအမျိုးအစားကွဲတွင် <member>၊ … နှင့် <member> တို့ပါဝင်သည်။
<name> ဟူသော [အများသုံး] ဒေတာအမျိုးအစားကွဲတွင် <member> နှင့် … <member> တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ <word> ဖြစ်သည်။
<name> ဟူသော [အများသုံး] ဒေတာအမျိုးအစားကွဲတွင် <member> ပါဝင်သည်။

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
an enumeration, we put `အများသုံး` after `ဟူသော`, as for a struct ([Modules](modules.md)).

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
အမှားများ ဟူသော ဒေတာအမျိုးအစားကွဲတွင် တန်ဖိုးအနေဖြင့် အခြေခံ + ၄ ရှိသည့် မတွေ့၊ ပိတ်ထား နှင့် တန်ဖိုးအနေဖြင့် အခြေခံ * ၂ ရှိသည့် ပြည့်နေ တို့ပါဝင်သည်။
အမိန့် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် တန်ဖိုးအနေဖြင့် "go" ရှိသည့် သွား နှင့် တန်ဖိုးအနေဖြင့် "stop" ရှိသည့် ရပ် တို့ပါဝင်သည်။

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
မီးပွိုင့် ဟူသော ဒေတာအစုအဝေးတွင် အမှတ် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် ရောင် ဟူသော အရောင် တစ်ခု တို့ပါဝင်သည်။

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
မီးပွိုင့် ဟူသော ဒေတာအစုအဝေးတွင် အမှတ် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် ရောင် ဟူသော အရောင် တစ်ခု တို့ပါဝင်သည်။

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
  arm. A case that the folder cannot work out, such as a variable, does not count as
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
| ZK0403 | an ordering operator on members |
| ZK0404 | members of two enumerations, or a member and another value, compared |
| ZK0456 | a member value that the folder cannot work out |
| ZK0457 | two members with the same value |
| ZK0458 | a switch over an enumeration that misses a member and has no default arm |
| ZK0459 | a default arm whose data kind is not the data kind of the subject |
| ZK0460 | two cases that fold to one value |
| ZK0461 | a member with no value that cannot count on from the member before it |
| ZK0479 | `ကနဦးတန်ဖိုး` for an enumeration with no initial member |
| ZK0480 | a second member marked as the initial member |
