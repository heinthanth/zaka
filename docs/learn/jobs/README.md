---
title: Jobs
description: Work that runs side to side with the rest of the program: starting a job, waiting for its result, a job that gives nothing, and the values a job can take.
---

# Jobs

A job (`အပြိုင်လုပ်ငန်း`) runs a function while the rest of the program goes on, as a cook
puts rice on the fire and cuts the onions while it boils. A native program runs each job on
a thread of its own; the VM takes turns between them. [Jobs](../../manual/language/jobs.md)
in the Docs states the rules in full.

## Starting a job and waiting for it

A long call that ends in `အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း` starts the function as a job and
gives the job. `<job> ကို စောင့်သော ရလဒ်` waits for the job to end and gives what its
function gave back:

```zaka
စုပေါင်း ဟူသော လုပ်ဆောင်ချက် သည် ဂဏန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် စု သည် ၀ ဖြစ်ပါစေ။
ဂဏန်းများ ၏ အကွက်တိုင်း ကို ဂ အနေဖြင့် ယူ၍ စု ကို စု + ဂ ဟု ထားမည်။
စု ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရှေ့ သည် စုပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို [၁၊ ၂၊ ၃] ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ကိန်းရှင် နောက် သည် စုပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို [၄၊ ၅၊ ၆] ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
"နှစ်ခု စတင်ပြီ" ဟု ပြသမည်။
(ရှေ့ ကို စောင့်သော ရလဒ်) + (နောက် ကို စောင့်သော ရလဒ်) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
နှစ်ခု စတင်ပြီ
၂၁
```

The kind of `ရှေ့` is `ကိန်းပြည့် ပြန်ပေးမည့် အပြိုင်လုပ်ငန်း`: a job whose function gives an
integer. One wait takes the result: keep it in a variable to use it again, since a second
wait for the same job stops the program.

## A job that gives nothing

A call that ends in `အပြိုင် လုပ်ဆောင်မည်။` starts a job and keeps no name for it. The
program waits for each such job before it ends:

```zaka
ပြ ဟူသော လုပ်ဆောင်ချက် သည် စာ ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စာ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပြ ဟူသော လုပ်ဆောင်ချက်ကို "နောက်ကွယ်မှ" ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
"ရှေ့မှ" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရှေ့မှ
နောက်ကွယ်မှ
```

`<job> ကို စောင့်မည်။` waits for a job and drops what it gives back.

## What a job can take

A job gets its own duplicate of each argument: numbers, text, lists, plain structs and
results go as each is, and a write on one side never reaches the other. Three rules
guard a job, so the program never changes one value from two threads:

- A plain shared object cannot go to a job, ZK0494; [Sharing between
  jobs](sharing.md) shows the objects that can.
- The function a job starts in is one the program declares, and changes no parameter:
  ZK0497, ZK0498.
- A job reads and writes no variable at the top of a file, ZK0726: it belongs to the main
  thread. Pass the value as an argument instead.

```zaka-error ZK0726
ကိန်းရှင် ကြိမ် သည် ၀ ဖြစ်ပါစေ။

တိုး ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကြိမ် ကို ကြိမ် + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
တိုး ဟူသော လုပ်ဆောင်ချက်ကို အပြိုင် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## When a job stops

A [trap](../../manual/vm/traps.md) in a job ends that job alone: the reason goes to
standard error, and the program goes on. A wait for that job stops the waiter in turn,
and a program whose job stopped with no wait to see it ends with exit code 1.
`--trap abort` makes a trap in a job stop the whole program.

Next: [Channels](channels.md) take values from one job to another.
