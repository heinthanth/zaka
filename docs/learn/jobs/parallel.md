---
title: Parallel work
description: A parallel map that applies a function to each cell of a list on several threads, and a parallel loop whose rounds each write their own cell.
---

# Parallel work

When each cell of a list needs the same work, the cells can be split among threads, as
several cooks each peel their own share of the potatoes.

## A parallel map

`<list> ၏ အကွက်တိုင်း ကို <fn> ဟူသော လုပ်ဆောင်ချက် ဖြင့် အပြိုင် လုပ်ဆောင်သော ရလဒ်များ` gives a
new plain list of what the function gives back for each cell, in the order of the cells. A
native program spreads the cells over one thread for each core:

```zaka
နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက် သည် န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
န * န ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဂဏန်းများ သည် [၁၊ ၂၊ ၃၊ ၄၊ ၅] ဖြစ်ပါစေ။
ဂဏန်းများ ၏ အကွက်တိုင်း ကို နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက် ဖြင့် အပြိုင် လုပ်ဆောင်သော ရလဒ်များ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၄၊ ၉၊ ၁၆၊ ၂၅]
```

The function follows the rules of a job: one the program declares, that reads no
variable at the top of a file.

## A parallel loop

`အပြိုင်` before `ရေတွက်၍` or `ယူ၍` marks a loop whose rounds can run at once. Each round
writes the variables it declares and the cell of its own number alone; each other write
from a round, such as a running sum, is ZK0728:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ထွက် သည် အကွက် ၅ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
၀ မှ ၄ အထိ i အနေဖြင့် အပြိုင် ရေတွက်၍ ထွက် ၏ အကွက် i ကို i * ၁၀ ဟု ထားမည်။
ထွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၀၊ ၁၀၊ ၂၀၊ ၃၀၊ ၄၀]
```

A native program runs the rounds on one thread for each core, once the rounds are at
least as numerous as the cores; fewer rounds, and the rounds on the VM, run in order. To pick the count of threads, write `အပြိုင်လုပ်ငန်း <n> ခုခွဲ၍` in place of
`အပြိုင်`: `၀ မှ ၉၉ အထိ i အနေဖြင့် အပြိုင်လုပ်ငန်း ၄ ခုခွဲ၍ ရေတွက်၍ ...` splits the rounds among
four threads. The rounds follow
the rules of a job:

- A round reads values from outside that jobs can use: numbers, text and lists, but no
  plain shared object (ZK0494).
- A round cannot stop the loop or return from its function (ZK0729), since the other
  rounds are running; `နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်။` ends its own round.
- The lines rounds print come in no fixed order on threads.
- A trap in a round ends the rounds of its thread; the loop then stops once the others
  finish, as a wait for a job that trapped does.

```zaka-error ZK0729
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ထွက် သည် အကွက် ၅ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
၀ မှ ၄ အထိ i အနေဖြင့် အပြိုင် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ထွက် ၏ အကွက် i ကို i ဟု ထားမည်။
ကွင်းဆက် ကို ရပ်မည်။          // the other rounds are running: ZK0729
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```
