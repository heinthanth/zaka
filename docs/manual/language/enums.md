---
title: Enumerations
description: The enumeration declaration, member names and values, the initial member, comparing, converting and printing members, and the switch statement.
---

# Enumerations

An enumeration is a kind whose values are a fixed list of named *members*. Each member
stands for a constant value of one kind, an `isize` unless the first member gives
another. The Learn book starts in small steps with
[Enumerations and switches](../../learn/enums/README.md).

```zaka
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ၊ စိမ်း တို့ပါဝင်သည်။

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
<name> ဟူသော [အများသုံး] ဒေတာအမျိုးအစားကွဲတွင် <member>၊ … <member> တို့ပါဝင်သည်။
<name> ဟူသော [အများသုံး] ဒေတာအမျိုးအစားကွဲတွင် <member>၊ … <member> တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ <word> ဖြစ်သည်။
<name> ဟူသော [အများသုံး] ဒေတာအမျိုးအစားကွဲတွင် <member> ပါဝင်သည်။

member := [ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်] [တန်ဖိုးအနေဖြင့် <value> ရှိသည့်] <name>
```

- An enumeration opens with its name and `ဟူသော`, then `ဒေတာအမျိုးအစားကွဲ` or
  `အချက်အလက်အမျိုးအစားကွဲ` (one word, as `ဒေတာ` and `အချက်အလက်` are), where a struct has
  `ဒေတာအစုအဝေး` ([Structs](structs.md)). `တွင်` can stand joined to the word or apart.
- A declaration stands at the top level of a file. Inside a block, a loop, a conditional
  or a function it is ZK0235. An enumeration can be used above its declaration, since the
  declarations are read first.
- An enumeration has at least one member. The members are names with `၊` between them,
  and the list closes with `တို့ပါဝင်သည်`; a lone member closes with `ပါဝင်သည်`.
  `တို့` after one member, or none after several, is ZK0201.
- `တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ <word> ဖြစ်သည်` at the end, or `ပါဝင်ပြီး ...` after a lone
  member, names the classifier the enumeration counts with, as a struct does; with none it
  counts with `ခု` ([Measure words](kinds.md#measure-words)).
- The member names of one enumeration differ from each other (ZK0328).
- An enumeration and a struct share one set of names: a name declared twice, as either, is
  ZK0316. An enumeration name cannot also name a variable, a constant, a parameter, a
  field or a function (ZK0319), and a kind that names neither is ZK0318.
- `အများသုံး` after `ဟူသော` exports it, as it does a struct; see [Modules](modules.md).

### Member values

```zaka
ကိန်းသေ အခြေခံ သည် ၁၀၀ ဖြစ်ပါစေ။
အမှားများ ဟူသော ဒေတာအမျိုးအစားကွဲတွင် တန်ဖိုးအနေဖြင့် အခြေခံ + ၄ ရှိသည့် မတွေ့၊ ပိတ်ထား၊ တန်ဖိုးအနေဖြင့် အခြေခံ * ၂ ရှိသည့် ပြည့်နေ တို့ပါဝင်သည်။
အမိန့် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် တန်ဖိုးအနေဖြင့် "go" ရှိသည့် သွား၊ တန်ဖိုးအနေဖြင့် "stop" ရှိသည့် ရပ် တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အမှားများထဲမှ ပိတ်ထား ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
အမိန့်များထဲမှ ရပ် ကို စာသား တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၀၅
stop
```

- Each member stands for a constant value, written `တန်ဖိုးအနေဖြင့် <value> ရှိသည့်` in front
  of its name ("that has `<value>` as its value"). A value is each
  expression the folder works out: a literal, a constant, even one declared below the
  enumeration, or an operation on them. A value that does not work out, such as a
  variable, is ZK0456.
- The value of the first member gives the *value kind* of the enumeration, the kind
  of each member value: a number, a character, a true/false value, text, a list or a
  struct. With no value on the first member, the kind is `isize`. Each other value
  converts to that kind, as the value of a declaration does (ZK0401).
- A member with no value takes the value of the member before it plus one, the first 0,
  when the value kind is an integer. Another kind needs each value written, and so
  does a member past the largest value of its integer kind (ZK0461).
- Two members of one enumeration have different values (ZK0457), so a print or a switch
  can tell each member apart.
- Text works out as each constant text does: a literal, a text constant, a template whose
  slots work out, written as a print writes them, or text joined with `+` or
  `ဆက်ထားသည့် စာသား`. A list or a struct can hold such text.

## The initial member

No member of an enumeration is a default. One member can take the mark
`ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်` ("to be used as the initial value") in front of its name and
its value, which makes it the [initial value](kinds.md#initial-values) of the
kind: the member that `ကနဦးတန်ဖိုး` gives, and the one that a struct or a counted list
holding the enumeration takes when it is filled with `ကနဦးတန်ဖိုး`.

```text
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် တန်ဖိုးအနေဖြင့် ၁ ရှိသည့် နီ၊ ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့် တန်ဖိုးအနေဖြင့် ၀ ရှိသည့် စိမ်း၊ တန်ဖိုးအနေဖြင့် ၂ ရှိသည့် ပြာ တို့ပါဝင်သည်။
```

- The mark goes on one member at most: a second mark is ZK0480. With a value, the mark
  comes first and the value after it.
- The mark changes no value and no order: a member with no value still counts on from the
  member before it, and the first member still gives the value kind.
- An enumeration with no marked member has no initial value: `ကနဦးတန်ဖိုး` for it, or
  for a struct or a list that holds it, is ZK0479.
- `ကနဦးတန်ဖိုး` is a reserved word, so no member takes it as a name.

```zaka
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့် ဝါ၊ စိမ်း တို့ပါဝင်သည်။
မီးပွိုင့် ဟူသော ဒေတာအစုအဝေးတွင် အမှတ် ဟူသော ကိန်းပြည့် တစ်ခု၊ ရောင် ဟူသော အရောင် တစ်ခု တို့ပါဝင်သည်။

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
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့် နီ၊ ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့် စိမ်း တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0480]: "အရောင်" marks a second member with "ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်"
```

```zaka-error ZK0479
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ စိမ်း တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မီး သည် အရောင် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0479]: "အရောင်" has no initial value: no member of "အရောင်" is marked "ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်"
```

## The name and its stem

The *stem* of an enumeration name is the name with a closing `များ`, the plural mark,
taken off: `အရောင်` for `အရောင်`, and `အမှား` for `အမှားများ`. The stem names the
enumeration too, so `ကိန်းရှင် မ သည် အမှား တစ်ခု ဖြစ်ပါစေ။` declares a variable of
`အမှားများ`. The stem joins the names that ZK0316 and ZK0319 guard.

## Members

```text
<stem>များထဲမှ <member>
```

reads "`<member>` from among the `<stem>`s": `အရောင်များထဲမှ နီ`, `အမှားများထဲမှ မတွေ့`.

- The name before `ထဲမှ` is the stem with `များ` after it, and nothing else. A name that
  is no enumeration, or an enumeration written without its `များ`, is ZK0330; the help
  line of the message gives the spelling to write.
- A member the enumeration does not have is ZK0329.
- `ထဲမှ` can stand apart from the name or join it: `အရောင်များ ထဲမှ နီ` and
  `အရောင်များထဲမှ နီ` are one form. The lexer splits a name written joined to `ထဲမှ`, so
  no name of a program ends in `ထဲမှ`.
- A member is a constant: it can give a `ကိန်းသေ` its value, and the folder works it out.

A name in Latin letters, or in mixed scripts, splits from `ထဲမှ` just as a
Burmese name does:

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

## Values of an enumeration

- A value of an enumeration holds the value of its member, as a value of the value
  kind is held: it has the size and the arrangement of that kind. A text member holds static
  text.
- `==` and `!=` compare two members of one enumeration, as `==` compares their values. A
  member and a value of another kind, or members of two enumerations, do not compare
  (ZK0404); `<`, `>` and the other order operators do not take members (ZK0403).
- The conversion `<value> ကို <kind> တစ်<word> အဖြစ် ပြောင်းလဲသော ရလဒ်` turns a member into its
  value: to the value kind, and when the value is a number, a character or a
  true/false value, to each kind the value converts to; see [Conversions](conversions.md). No conversion goes
  from a value to a member.
- A variable of an enumeration declared with no value holds no member until the program
  sets it ([Set before read](statements.md#set-before-read)). No member is a default:
  `ကနဦးတန်ဖိုး` gives the member the declaration marks ([The initial
  member](#the-initial-member)). A heap block starts at zero octets, as with each kind,
  and those octets can be no member at all.
- A member is copied, never moved, and holds no address; a field, a cell, a parameter and a
  function result can each hold one.
- Printing a member, with `ပြသမည်` or in a template slot, writes `<stem>များထဲမှ <member>`,
  the form that reads back. Inside a list or a struct it is written the same. A value that
  no member stands for, from a heap block that starts at zero octets, prints as its
  value.

```zaka
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ၊ စိမ်း တို့ပါဝင်သည်။
မီးပွိုင့် ဟူသော ဒေတာအစုအဝေးတွင် အမှတ် ဟူသော ကိန်းပြည့် တစ်ခု၊ ရောင် ဟူသော အရောင် တစ်ခု တို့ပါဝင်သည်။

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
<case>၊ <case> တို့ ဖြစ်လျှင် <block>
တခြား <kind> တစ်<word> ဖြစ်လျှင် <block>
```

A switch compares its subject with the case of each arm, top to bottom, with `==`, and runs
the block of the first arm whose case equals it. When none does, it runs the block of the
default arm `တခြား <kind> တစ်<word> ဖြစ်လျှင်`, or nothing when there is none. The subject is worked
out once.

- An arm takes one case, or several with `၊` between them and `တို့` after the last; it
  runs when the subject equals one of them. A list literal is one case in each form:
  `[a၊ b]`, `a၊ b တို့ပါသော စာရင်း`, or either in parentheses.
- The first arm names the subject with `သည်`; each arm after it is a case alone. The block
  of an arm is one statement or a block, as in a conditional. A one-statement arm cannot be a
  conditional or another switch (ZK0215), so each arm after it belongs to one switch.
- The switch ends after its default arm, or at the first sentence that is not an arm. An
  arm is a sentence that reaches `ဖြစ်လျှင်` before its full stop, with no `သည်` before
  it and no statement word at its start; a sentence with a `သည်` starts a new switch.
- The subject is a value of each kind that `==` takes: a number, a character, a true/false
  value, text, a list, a struct, a pointer or an enumeration. Each case
  is a value that `==` takes with the subject; another kind is the error `==` gives, such
  as ZK0404.
- Over an enumeration, a case written as a bare name, such as `နီ`, is that member of the
  subject's enumeration, before a variable of that name. A name that is neither a member
  nor a variable is ZK0329. `<stem>များထဲမှ <member>` works as a case too.
- Two cases that fold to one value are ZK0460: the second arm could never run.
- The default arm names the kind of the subject with its measure word (ZK0459); for an
  enumeration, its name or its stem: `တခြား အမှား တစ်ခု ဖြစ်လျှင်`. It comes last (ZK0236).
- Over an enumeration, the cases name each member, or the switch has a default arm
  (ZK0458). A case that does not fold, such as a variable, names no member here. Over
  other kinds, the default arm can be left out.

```zaka
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ၊ စိမ်း တို့ပါဝင်သည်။

လုပ်ဆောင်ချက် လုပ်ရန် သည်
ယခု ဟူသော အရောင် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
ယခု သည် နီ ဖြစ်လျှင် "ရပ်" ကို ပြန်ပေးမည်။
ဝါ ဖြစ်လျှင် "ဖြည်းဖြည်း" ကို ပြန်ပေးမည်။
တခြား အရောင် တစ်ခု ဖြစ်လျှင် "သွား" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ရန် ဟူသော လုပ်ဆောင်ချက်ကို အရောင်များထဲမှ စိမ်း တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
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

The switch lowers to the chain of tests a conditional lowers to, one `==` per arm, so it
costs what the conditional costs; see [IR and code](../internals/codegen.md).
