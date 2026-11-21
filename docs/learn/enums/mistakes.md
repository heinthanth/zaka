---
title: Enumeration mistakes
description: The common mistakes with enumerations and switches, the message Zaka shows for each one, and the fix.
---

# Enumeration mistakes

The compiler catches these common mistakes with enumerations (enums) and switches before
the program runs. Each example shows the code, the message from the compiler, and how to
fix the error.

## You will learn

- the mistakes in declaring an enumeration and in giving its members (variants) values
- the mistakes in naming a member
- how the compiler checks that a switch covers each member
- the mistakes with members that hold data

## An enumeration declared inside a block

```zaka-error ZK0235
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ နှင့် ဝါ တို့ပါဝင်သည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0235]: enumerations can be declared at the top level and nowhere else
```

We can declare an enumeration at the top level of a file alone, and not inside a block, a
loop, a conditional or a function. The fix is to move the declaration above the program
start block.

## One member name twice

```zaka-error ZK0328
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ နှင့် နီ တို့ပါဝင်သည်။
```

```text
error[ZK0328]: the enumeration "အရောင်" names the member "နီ" twice
```

Each member of an enumeration needs a different name, so the fix is to rename or remove
the second `နီ`.

## Two members with one value

```zaka-error ZK0457
အဆင့် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နိမ့်၊ အလယ် နှင့် တန်ဖိုးအနေဖြင့် ၁ ရှိသည့် မြင့် တို့ပါဝင်သည်။
```

```text
error[ZK0457]: the member "မြင့်" has the value of "အလယ်"
```

Each member needs a different value. A member with no value written gets the value of the
member before it plus one, so here `အလယ်` is 1, which is the value written for `မြင့်`.
The fix is to give `မြင့်` a value that no other member has, such as 2.

## No member marked as the initial one

```zaka-error ZK0479
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ နှင့် စိမ်း တို့ပါဝင်သည်။

ကိန်းရှင် မီး သည် အရောင် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
```

```text
error[ZK0479]: "အရောင်" has no initial value: no member of "အရောင်" is marked "ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်"
```

`ကနဦးတန်ဖိုး` means the member that we mark with `ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်`, and here
no member has the mark, so the enumeration has no initial value. The fix is to write the
member in full, as `အရောင်များထဲမှ နီ`, or to mark one member:
`ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့် နီ၊ ဝါ နှင့် စိမ်း`.

## Two members marked as the initial one

```zaka-error ZK0480
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့် နီ၊ ဝါ နှင့် ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့် စိမ်း တို့ပါဝင်သည်။
```

```text
error[ZK0480]: "အရောင်" marks a second member with "ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်"
```

An enumeration can have just one initial member, so the fix is to remove one of the two
marks.

## A member value that is not a constant

```zaka-error ZK0456
ကိန်းရှင် စတင် သည် ၁၀ ဖြစ်ပါစေ။
အဆင့် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် တန်ဖိုးအနေဖြင့် စတင် ရှိသည့် နိမ့် နှင့် မြင့် တို့ပါဝင်သည်။
```

```text
error[ZK0456]: the value of the member "နိမ့်" must work out when the program compiles
```

The compiler works out each member value when the program compiles, but the value of a
variable is known when the program runs. The fix is to declare `စတင်` as a constant with
`ကိန်းသေ`.

## A member with no value that cannot count on

```zaka-error ZK0461
အမိန့် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် တန်ဖိုးအနေဖြင့် "go" ရှိသည့် သွား နှင့် ရပ် တို့ပါဝင်သည်။
```

```text
error[ZK0461]: the member "ရပ်" needs a value: no text value follows the member before it
```

Members count on from the member before them when their values are integers, but text and
the other data kinds (types) have no next value. The fix is to write the value of each
member, such as `တန်ဖိုးအနေဖြင့် "stop" ရှိသည့် ရပ်`.

## A member the enumeration does not have

```zaka-error ZK0329
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ နှင့် စိမ်း တို့ပါဝင်သည်။

ကိန်းရှင် မီး သည် အရောင်များထဲမှ ပြာ ဖြစ်ပါစေ။
```

```text
error[ZK0329]: the enumeration "အရောင်" has no member "ပြာ"
```

`အရောင်` has no member `ပြာ` (blue). The fix is to use one of its members, or to add `ပြာ`
to the declaration.

## A member written without the plural mark

```zaka-error ZK0330
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ နှင့် စိမ်း တို့ပါဝင်သည်။

ကိန်းရှင် မီး သည် အရောင်ထဲမှ နီ ဖြစ်ပါစေ။
```

```text
error[ZK0330]: "အရောင်" before "ထဲမှ" names no enumeration
  = help: write "အရောင်များထဲမှ"
```

To name a member, we write the enumeration name with `များ` before `ထဲမှ`. The help line
shows the fix: `အရောင်များထဲမှ နီ`.

## A switch that misses a member

```zaka-error ZK0458
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ နှင့် စိမ်း တို့ပါဝင်သည်။

ကိန်းရှင် မီး သည် အရောင်များထဲမှ ဝါ ဖြစ်ပါစေ။
မီး သည် နီ ဖြစ်လျှင် "ရပ်" ဟု ပြသမည်။
စိမ်း ဖြစ်လျှင် "သွား" ဟု ပြသမည်။
```

```text
error[ZK0458]: no arm of this switch names "ဝါ"
  = help: name each member in an arm, or add "တခြား အရောင် တစ်ခု ဖြစ်လျှင်" last
```

A switch over an enumeration must name each member in an arm, or end with the default
arm. The fix is to add an arm for `ဝါ`, or the default arm, as the help line shows.

## A default arm of the wrong data kind

```zaka-error ZK0459
ကိန်းရှင် ရက် သည် ၆ ဖြစ်ပါစေ။
ရက် သည် ၀ ဖြစ်လျှင် "တနင်္ဂနွေ" ဟု ပြသမည်။
တခြား စာသား တစ်ခု ဖြစ်လျှင် "ကြားရက်" ဟု ပြသမည်။
```

```text
error[ZK0459]: the default arm names "text", but the subject of the switch is "isize"
```

The default arm names the data kind of the subject, with its measure word (classifier).
The subject `ရက်` is an integer, so the fix is `တခြား ကိန်းပြည့် တစ်ခု ဖြစ်လျှင်`.

## An arm after the default arm

```zaka-error ZK0236
ကိန်းရှင် ရက် သည် ၆ ဖြစ်ပါစေ။
ရက် သည် ၀ ဖြစ်လျှင် "တနင်္ဂနွေ" ဟု ပြသမည်။
တခြား ကိန်းပြည့် တစ်ခု ဖြစ်လျှင် "ကြားရက်" ဟု ပြသမည်။
၆ ဖြစ်လျှင် "စနေ" ဟု ပြသမည်။
```

```text
error[ZK0236]: the default arm "တခြား ... တစ်ခု ဖြစ်လျှင်" comes last in a switch
```

The default arm runs when no case before it equals the subject, so an arm after it could
never run. The fix is to move the default arm to the end.

## One case twice

```zaka-error ZK0460
ကိန်းရှင် ရက် သည် ၆ ဖြစ်ပါစေ။
ရက် သည် ၆ ဖြစ်လျှင် "စနေ" ဟု ပြသမည်။
၃ + ၃ ဖြစ်လျှင် "စနေ" ဟု ပြသမည်။
```

```text
error[ZK0460]: an arm before this one has this case
```

A switch runs the first arm whose case equals the subject, and `၃ + ၃` equals `၆`, so the
second arm could never run. The fix is to remove the second arm, or to change its case.

## A member that holds data written with none

```zaka-error ZK0354
ပုံ ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ဒေတာအနေဖြင့် ကိန်းပြည့် တစ်ခု ရှိသည့် စက်ဝိုင်း နှင့် အမှတ် တို့ပါဝင်သည်။

ကိန်းရှင် ပ သည် ပုံများထဲမှ စက်ဝိုင်း ဖြစ်ပါစေ။
```

```text
error[ZK0354]: the member "စက်ဝိုင်း" of "ပုံ" holds data of the kind "isize", and none is given
```

`စက်ဝိုင်း` holds an integer, so each value of it needs one. The fix is to give the data,
`ပုံများထဲမှ စက်ဝိုင်း(၅)`. The opposite mistake, data for a member that holds none, such as
`ပုံများထဲမှ အမှတ်(၁)`, is ZK0355.

## A read of a member the test ruled out

```zaka-error ZK0358
ပုံ ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ဒေတာအနေဖြင့် ကိန်းပြည့် တစ်ခု ရှိသည့် စက်ဝိုင်း နှင့် အမှတ် တို့ပါဝင်သည်။

ကိန်းရှင် ပ သည် ပုံများထဲမှ အမှတ် ဖြစ်ပါစေ။
ပ သည် အမှတ် ဖြစ်လျှင် ပ ကို စက်ဝိုင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
တခြား ပုံ တစ်ခု ဖြစ်လျှင် "အခြား" ဟု ပြသမည်။
```

```text
error[ZK0358]: "ပ" is "အမှတ်" here, so reading it as "စက်ဝိုင်း" stops the run each time
```

Inside the arm of `အမှတ်`, the variable `ပ` is `အမှတ်`, so a read of the data of
`စက်ဝိုင်း` there would stop the program each time. The fix is to read the data in the arm
of `စက်ဝိုင်း`, or to name it in that arm with `ရ ဟူသည့် စက်ဝိုင်း အဖြစ်သို့
ပြောင်းလဲနိုင်လျှင်`.

## Recap

- An enumeration is declared at the top level, and each member has a different name and
  a different value.
- Member values are constants, and members with text or other non-integer values each need
  a value written.
- A member is named with `<name>များထဲမှ <member>`, with `များ` before `ထဲမှ`.
- A switch over an enumeration names each member or ends with the default arm, which comes
  last and names the data kind of the subject.
- A member that holds data is written with its data, and its data is read in the arm or
  branch that tested it.

Next: [Functions as values](../function-values/README.md).
