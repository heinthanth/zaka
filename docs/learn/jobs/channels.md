---
title: Channels
description: A channel carries values from one job to another: making one, sending, receiving, a loop over each value, closing, and a channel with a room.
---

# Channels

A channel (`ဒေတာလမ်းကြောင်း`) is a queue between jobs, as a moving belt takes
dishes from the kitchen to the counter: one job puts values on it, another takes them off
in the same order.

## Sending and receiving

`ကနဦးတန်ဖိုး` makes a new channel. `<pipe> ထဲသို့ <value> ကို ပို့မည်။` sends a value, and
`<pipe> ကို ပိတ်မည်။` closes the channel once no value is left to send.
`<pipe> မှ ရရှိသော ဒေတာတိုင်းကို <name> အနေဖြင့် ယူ၍` takes each value that comes, and ends
once the channel is closed and drained:

```zaka
ထုတ် ဟူသော လုပ်ဆောင်ချက် သည် လမ်း ဟူသော စာသား ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လမ်း ထဲသို့ "ထမင်း" ကို ပို့မည်။
လမ်း ထဲသို့ "ဟင်း" ကို ပို့မည်။
လမ်း ကို ပိတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခါးပတ် သည် စာသား ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ထုတ် ဟူသော လုပ်ဆောင်ချက်ကို ခါးပတ် ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
ခါးပတ် မှ ရရှိသော ဒေတာတိုင်းကို ပန်းကန် အနေဖြင့် ယူ၍ "ရ: {ပန်းကန်}" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရ: ထမင်း
ရ: ဟင်း
```

## One value at a time

`<pipe> မှ ရရှိသော ဒေတာ` takes the next value as an [optional](../optionals/README.md): it
waits while the channel is open and holds none, and gives `အလွတ်` once it is closed and
drained. `အကယ်၍ <pipe> ထဲတွင် ဒေတာ ရှိလျှင်` takes the next value as well, and inside that
branch `<pipe> ထဲမှ ဒေတာ` names it:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လမ်း သည် ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
လမ်း ထဲသို့ ၁၀ ကို ပို့မည်။
လမ်း ထဲသို့ ၂၀ ကို ပို့မည်။
လမ်း ကို ပိတ်မည်။
လမ်း မှ ရရှိသော ဒေတာ ကို ပြသမည်။
အကယ်၍ လမ်း ထဲတွင် ဒေတာ ရှိလျှင် (လမ်း ထဲမှ ဒေတာ) * ၂ ကို ပြသမည်။
လမ်း မှ ရရှိသော ဒေတာ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဒေတာ(၁၀)
၄၀
အလွတ်
```

## A channel with a room

`<kind> <n> ခု ပို့နိုင်သော ဒေတာလမ်းကြောင်း` holds `<n>` values at most: a send waits while it
is full, so a fast sender keeps pace with a slow receiver. A receive from a channel that
no job sends to or closes would wait for ever; the program stops with the trap `deadlock`
in its place.

Next: [Sharing between jobs](sharing.md).
