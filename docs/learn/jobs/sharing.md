---
title: Sharing between jobs
description: An object two jobs share and change, the lock block that guards it, and atomic numbers that jobs add to with no lock.
---

# Sharing between jobs

Values go to a job as duplicates, so two jobs never write one value. When jobs must
change one thing together, Zaka gives two tools: an object a lock guards, and an
atomic number.

## An object jobs share

The mark `အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော` on a shared struct makes objects that jobs
can share. Each field of such an object is read and written inside a lock block,
`<obj> ကို ခေတ္တယူ၍ ...`, which one job at a time holds, as one cook at a time holds the
knife:

```zaka
အကောင့် ဟူသော အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ထည့်ငွေ ဟူသော လုပ်ဆောင်ချက် သည် ဘဏ် ဟူသော အကောင့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ မှ ၁၀၀ အထိ န အနေဖြင့် ရေတွက်၍ ဘဏ် ကို ခဏယူ၍ ဘဏ် ၏ လက်ကျန် ကို (ဘဏ် ၏ လက်ကျန်) + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဘဏ် သည် အကောင့်(လက်ကျန်: ၀) ဖြစ်ပါစေ။
ကိန်းရှင် က သည် ထည့်ငွေ ဟူသော လုပ်ဆောင်ချက်ကို ဘဏ် ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ထည့်ငွေ ဟူသော လုပ်ဆောင်ချက်ကို ဘဏ် ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
က ကို စောင့်မည်။
ခ ကို စောင့်မည်။
ဘဏ် ကို ခေတ္တယူ၍ ဘဏ် ၏ လက်ကျန် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၀၀
```

A field read outside a lock block is ZK0499. A job that stops at a trap while it holds a
lock leaves it broken, and the next lock of that object stops in turn.

## Atomic numbers

For one number, a lock is more than the work needs. `အက်တမ်မစ် ဖြစ်သော ကိန်းပြည့်` is an
integer that jobs add to with `ပေါင်းထည့်` and subtract from with `နုတ်ယူ`, each in one step no
other job can split, with no lock:

```zaka
ကောင်တာ ဟူသော အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် ကြိမ် ဟူသော အက်တမ်မစ် ဖြစ်သော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ရေ ဟူသော လုပ်ဆောင်ချက် သည် ရေတွက် ဟူသော ကောင်တာ တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ မှ ၁၀၀ အထိ န အနေဖြင့် ရေတွက်၍ ရေတွက် ၏ ကြိမ် ထဲသို့ ၁ ကို ပေါင်းထည့်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရေတွက် သည် ကောင်တာ(ကြိမ်: ၀) ဖြစ်ပါစေ။
ကိန်းရှင် က သည် ရေ ဟူသော လုပ်ဆောင်ချက်ကို ရေတွက် ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ရေ ဟူသော လုပ်ဆောင်ချက်ကို ရေတွက် ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
က ကို စောင့်မည်။
ခ ကို စောင့်မည်။
ရေတွက် ၏ ကြိမ် ထဲမှ ၁ ကို နုတ်ယူမည်။
ရေတွက် ၏ ကြိမ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၉၉
```

A set that reads the number it sets, such as `x ကို x + ၁ ဟု ထားမည်`, splits into a read and a
write, and another job can come between them: it is ZK0727.

To set a number when it holds what a job expects, in one step, ask for
`<atomic> ထဲရှိ <expected> ကို <new> ဖြင့် အစားထိုးသော ရလဒ်`. It gives a result: good, holding the
expected value, when the number took the new one; an error, holding what the number held,
when another job got there first. Here three jobs each want one object, and one wins:

```zaka
ကောင်တာ ဟူသော အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် ပိုင်ရှင် ဟူသော အက်တမ်မစ် ဖြစ်သော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ယူ ဟူသော လုပ်ဆောင်ချက် သည် ဘူး ဟူသော ကောင်တာ တစ်ခု ကို လည်းကောင်း န ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် ရ သည် ဘူး ၏ ပိုင်ရှင် ထဲရှိ ၀ ကို န ဖြင့် အစားထိုးသော ရလဒ် ဖြစ်ပါစေ။
အကယ်၍ ရ ထဲတွင် အောင်မြင်သည့်သတင်း ရှိလျှင် ၁ ကို ပြန်ပေးမည်။
၀ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဘူး သည် ကောင်တာ(ပိုင်ရှင်: ၀) ဖြစ်ပါစေ။
ကိန်းရှင် က သည် ယူ ဟူသော လုပ်ဆောင်ချက်ကို ဘူး နှင့် ၁ တို့ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ယူ ဟူသော လုပ်ဆောင်ချက်ကို ဘူး နှင့် ၂ တို့ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် ယူ ဟူသော လုပ်ဆောင်ချက်ကို ဘူး နှင့် ၃ တို့ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
(က ကို စောင့်သော ရလဒ်) + (ခ ကို စောင့်သော ရလဒ်) + (ဂ ကို စောင့်သော ရလဒ်) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁
```

Next: [Parallel work](parallel.md).
