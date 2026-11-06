---
title: Enumerations
description: The enumeration declaration, member names and values, starting values, comparing, converting and printing members, and the switch statement.
---

# Enumerations

An enumeration is a kind whose values are a fixed list of named *members*. Each member
stands for a constant value of one kind, an `isize` unless the first member gives
another. The Learn book starts in small steps with
[Enumerations and switches](../../learn/enums/README.md).

```zaka
အမျိုးအစား အရောင် ဆိုသည်မှာ နီ၊ ဝါ၊ စိမ်း တို့ ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မီး သည် အရောင်များထဲမှ ဝါ ဖြစ်ပါစေ။
မီး ကို ပြသပါ။
မီး ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အရောင်များထဲမှ ဝါ
၁
```

## Declaring an enumeration

```text
အမျိုးအစား <name> ဆိုသည်မှာ <member>၊ <member> (<value>)၊ <member> တို့ ဖြစ်သည်။
အမျိုးအစား <name> ဆိုသည်မှာ <member> ဖြစ်သည်။
```

- A declaration stands at the top level of a file. Inside a block, a loop, a conditional
  or a function it is ZK0235. An enumeration can be used above its declaration, since the
  declarations are read first.
- An enumeration has at least one member. The members are names with `၊` between them,
  and the list closes with `တို့ ဖြစ်သည်`; a lone member closes with `ဖြစ်သည်` alone.
- The member names of one enumeration differ from each other (ZK0328).
- An enumeration and a struct share one set of names: a name declared twice, as either, is
  ZK0316. An enumeration name cannot also name a variable, a constant, a parameter, a
  field or a function (ZK0319), and a kind that names neither is ZK0318.
- `အများသုံး` before the declaration exports it, as it does a struct; see
  [Modules](modules.md).

### Member values

```zaka
ကိန်းသေ အခြေခံ သည် ၁၀၀ ဖြစ်ပါစေ။
အမျိုးအစား အမှားများ ဆိုသည်မှာ မတွေ့ (အခြေခံ + ၄)၊ ပိတ်ထား၊ ပြည့်နေ (အခြေခံ * ၂) တို့ ဖြစ်သည်။
အမျိုးအစား အမိန့် ဆိုသည်မှာ သွား ("go")၊ ရပ် ("stop") တို့ ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အမှားများထဲမှ ပိတ်ထား ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
အမိန့်များထဲမှ ရပ် ကို စာသား အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၀၅
stop
```

- Each member stands for a constant value, its value in parentheses. A value is each
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

## Values of an enumeration

- A value of an enumeration holds the value of its member, as a value of the value
  kind is held: it has the size and the arrangement of that kind. A text member holds static
  text.
- `==` and `!=` compare two members of one enumeration, as `==` compares their values. A
  member and a value of another kind, or members of two enumerations, do not compare
  (ZK0404); `<`, `>` and the other order operators do not take members (ZK0403).
- The conversion `<value> ကို <kind> အဖြစ် ပြောင်းလဲသော ရလဒ်` turns a member into its
  value: to the value kind, and when the value is a number, a character or a
  true/false value, to each kind the value converts to; see [Conversions](conversions.md). No conversion goes
  from a value to a member.
- A variable of an enumeration declared with no value starts at the first member. A
  struct or a counted list starts each field and cell at the starting value of its kind,
  so an enumeration inside starts at its first member too. A heap block starts at zero
  octets, as with each kind.
- A member is copied, never moved, and holds no address; a field, a cell, a parameter and a
  function result can each hold one.
- Printing a member, with `ပြသပါ` or in a template slot, writes `<stem>များထဲမှ <member>`,
  the form that reads back. Inside a list or a struct it is written the same. A value that
  no member stands for, from a heap block that starts at zero octets, prints as its
  value.

```zaka
အမျိုးအစား အရောင် ဆိုသည်မှာ နီ၊ ဝါ၊ စိမ်း တို့ ဖြစ်သည်။
ဒေတာအမျိုးအစား မီးပွိုင့် တွင် ကိန်းပြည့် အမှတ်၊ အရောင် ရောင် တို့ ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပွိုင့် သည် မီးပွိုင့်(အမှတ်: ၃၊ ရောင်: အရောင်များထဲမှ စိမ်း) ဖြစ်ပါစေ။
ပွိုင့် ကို ပြသပါ။
[အရောင်များထဲမှ နီ၊ အရောင်များထဲမှ စိမ်း] ကို ပြသပါ။
"ယခု {ပွိုင့် ၏ ရောင်}" ဟု ပြသပါ။
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
တခြား <kind> ဖြစ်လျှင် <block>
```

A switch compares its subject with the case of each arm, top to bottom, with `==`, and runs
the block of the first arm whose case equals it. When none does, it runs the block of the
default arm `တခြား <kind> ဖြစ်လျှင်`, or nothing when there is none. The subject is worked
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
  value, text or a view of text, a list, a struct, an address or an enumeration. Each case
  is a value that `==` takes with the subject; another kind is the error `==` gives, such
  as ZK0404.
- Over an enumeration, a case written as a bare name, such as `နီ`, is that member of the
  subject's enumeration, before a variable of that name. A name that is neither a member
  nor a variable is ZK0329. `<stem>များထဲမှ <member>` works as a case too.
- Two cases that fold to one value are ZK0460: the second arm could never run.
- The default arm names the kind of the subject (ZK0459); for an enumeration, its name or
  its stem: `တခြား အမှား ဖြစ်လျှင်`. It comes last (ZK0236).
- Over an enumeration, the cases name each member, or the switch has a default arm
  (ZK0458). A case that does not fold, such as a variable, names no member here. Over
  other kinds, the default arm can be left out.

```zaka
အမျိုးအစား အရောင် ဆိုသည်မှာ နီ၊ ဝါ၊ စိမ်း တို့ ဖြစ်သည်။

လုပ်ဆောင်ချက် လုပ်ရန် သည်
အရောင် ယခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား ရလဒ် ကို ပြန်ပေးမည်။
ယခု သည် နီ ဖြစ်လျှင် "ရပ်" ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
ဝါ ဖြစ်လျှင် "ဖြည်းဖြည်း" ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
စိမ်း ဖြစ်လျှင် "သွား" ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် လုပ်ရန် ကို အရောင်များထဲမှ စိမ်း တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ကိန်းရှင် အက္ခရာ သည် 'ခ' ဖြစ်ပါစေ။
အက္ခရာ သည် 'က' ဖြစ်လျှင် "ပထမ" ဟု ပြသပါ။
'ခ' ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
"ဒုတိယ" ဟု ပြသပါ။
အက္ခရာ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
တခြား စာလုံး ဖြစ်လျှင် "အခြား" ဟု ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
သွား
ဒုတိယ
ခ
```

The switch lowers to the chain of tests a conditional lowers to, one `==` per arm, so it
costs what the conditional costs; see [IR and code](../internals/codegen.md).
