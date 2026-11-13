---
title: Enumeration mistakes
description: The mistakes enumerations and switches make possible, and how Zaka catches each one.
---

# Enumeration mistakes

## An enumeration declared inside a block

An enumeration is declared at the top level and nowhere else: not in a block, a loop, a
conditional or a function.

```zaka-error ZK0235
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ နှင့် ဝါ တို့ပါဝင်သည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0235]: enumerations can be declared at the top level and nowhere else
```

## One member name twice

```zaka-error ZK0328
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ နှင့် နီ တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0328]: the enumeration "အရောင်" names the member "နီ" twice
```

## Two members with one value

Each member stands for a value of its own. A member with no value written takes the value
of the member before it plus one, which can land on a value written later:

```zaka-error ZK0457
အဆင့် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နိမ့်၊ အလယ် နှင့် တန်ဖိုးအနေဖြင့် ၁ ရှိသည့် မြင့် တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0457]: the member "မြင့်" has the value of "အလယ်"
```

## No member marked as the initial one

`ကနဦးတန်ဖိုး` stands for the member marked `ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်`. With no member marked, the
enumeration has no initial value:

```zaka-error ZK0479
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ နှင့် စိမ်း တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မီး သည် အရောင် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0479]: "အရောင်" has no initial value: no member of "အရောင်" is marked "ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်"
```

Write the member in full, as `အရောင်များထဲမှ နီ`, or mark one: `ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့် နီ၊ ဝါ၊ စိမ်း`.

## Two members marked as the initial one

One member alone can be the initial one:

```zaka-error ZK0480
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့် နီ၊ ဝါ နှင့် ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့် စိမ်း တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0480]: "အရောင်" marks a second member with "ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်"
```

## A member value that is not a constant

A member value is worked out when the program compiles, so it cannot read a variable,
whose value is known when the program runs:

```zaka-error ZK0456
ကိန်းရှင် စတင် သည် ၁၀ ဖြစ်ပါစေ။
အဆင့် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် တန်ဖိုးအနေဖြင့် စတင် ရှိသည့် နိမ့် နှင့် မြင့် တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0456]: the value of the member "နိမ့်" must work out when the program compiles
```

## A member with no value that cannot count on

Members count on from the member before them when their values are integers. Text and
each other kind need each value written:

```zaka-error ZK0461
အမိန့် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် တန်ဖိုးအနေဖြင့် "go" ရှိသည့် သွား နှင့် ရပ် တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0461]: the member "ရပ်" needs a value: no text value follows the member before it
```

## A member the enumeration does not have

```zaka-error ZK0329
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ နှင့် စိမ်း တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မီး သည် အရောင်များထဲမှ ပြာ ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0329]: the enumeration "အရောင်" has no member "ပြာ"
```

## A member written without the plural mark

A member takes the enumeration name with `များ` before `ထဲမှ`. The help line gives the
spelling to write.

```zaka-error ZK0330
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ နှင့် စိမ်း တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မီး သည် အရောင်ထဲမှ နီ ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0330]: "အရောင်" before "ထဲမှ" names no enumeration
  = help: write "အရောင်များထဲမှ"
```

## A switch that misses a member

A switch over an enumeration names each member in an arm, or ends with the default arm.

```zaka-error ZK0458
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ နှင့် စိမ်း တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မီး သည် အရောင်များထဲမှ ဝါ ဖြစ်ပါစေ။
မီး သည် နီ ဖြစ်လျှင် "ရပ်" ဟု ပြသမည်။
စိမ်း ဖြစ်လျှင် "သွား" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0458]: no arm of this switch names "ဝါ"
  = help: name each member in an arm, or add "တခြား အရောင် တစ်ခု ဖြစ်လျှင်" last
```

## A default arm of the wrong kind

The default arm names the kind of the subject, with its measure word.

```zaka-error ZK0459
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရက် သည် ၆ ဖြစ်ပါစေ။
ရက် သည် ၀ ဖြစ်လျှင် "တနင်္ဂနွေ" ဟု ပြသမည်။
တခြား စာသား တစ်ခု ဖြစ်လျှင် "ကြားရက်" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0459]: the default arm names "text", but the subject of the switch is "isize"
```

## An arm after the default arm

The default arm runs when no case before it equals the subject, so an arm after it could
never run.

```zaka-error ZK0236
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရက် သည် ၆ ဖြစ်ပါစေ။
ရက် သည် ၀ ဖြစ်လျှင် "တနင်္ဂနွေ" ဟု ပြသမည်။
တခြား ကိန်းပြည့် တစ်ခု ဖြစ်လျှင် "ကြားရက်" ဟု ပြသမည်။
၆ ဖြစ်လျှင် "စနေ" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0236]: the default arm "တခြား ... တစ်ခု ဖြစ်လျှင်" comes last in a switch
```

## One case twice

A switch runs the first arm whose case equals the subject, so a second arm with an equal
case could never run.

```zaka-error ZK0460
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရက် သည် ၆ ဖြစ်ပါစေ။
ရက် သည် ၆ ဖြစ်လျှင် "စနေ" ဟု ပြသမည်။
၃ + ၃ ဖြစ်လျှင် "စနေ" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0460]: an arm before this one has this case
```

Next: [Functions as values](../function-values/README.md).
