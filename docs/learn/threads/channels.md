---
title: Channels
description: Pass values from one thread to another with a channel: make one, send values, receive them in a loop or one at a time, close it, and limit how much it can hold.
---

# Channels

A channel (`ဒေတာလမ်းကြောင်း`, a message queue) carries values from one thread to another,
much as a moving belt carries dishes from the kitchen to the counter. One thread puts
values on it, and another thread takes them off in the same order.

## You will learn

- How to make a channel, send values on it and close it.
- How to receive each value in a loop.
- How to receive one value at a time.
- How a channel with a limit holds a fast sender back.
- How to wait on several channels at once.

## Sending and receiving

```zaka
ထုတ် ဟူသော လုပ်ဆောင်ချက် သည် လမ်း ဟူသော စာသား ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လမ်း ထဲသို့ "ထမင်း" ကို ပို့မည်။
လမ်း ထဲသို့ "ဟင်း" ကို ပို့မည်။
လမ်း ကို ပိတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ခါးပတ် သည် စာသား ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ထုတ် ဟူသော လုပ်ဆောင်ချက်ကို ခါးပတ် ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
ခါးပတ် မှ ရရှိသော ဒေတာတိုင်းကို ပန်းကန် အနေဖြင့် ယူ၍ "ရ: {ပန်းကန်}" ဟု ပြသမည်။
```

```text
ရ: ထမင်း
ရ: ဟင်း
```

The data kind (type) `စာသား ပို့နိုင်သော ဒေတာလမ်းကြောင်း` is "a channel that can send
text", and we use `ကနဦးတန်ဖိုး` to make a new channel of it, with no values in it. The thread sends two
values and closes the channel, while the program start block receives each value until the
channel is closed. These are the forms:

```
<kind> ပို့နိုင်သော ဒေတာလမ်းကြောင်း
<channel> ထဲသို့ <value> ကို ပို့မည်။
<channel> ကို ပိတ်မည်။
<channel> မှ ရရှိသော ဒေတာတိုင်းကို <name> အနေဖြင့် ယူ၍ <sentence>
```

| Zaka | Reads as |
| --- | --- |
| `စာသား ပို့နိုင်သော ဒေတာလမ်းကြောင်း` | "a data path that can send text" |
| `လမ်း ထဲသို့ "ထမင်း" ကို ပို့မည်` | "send `"ထမင်း"` into `လမ်း`" |
| `လမ်း ကို ပိတ်မည်` | "close `လမ်း`" |
| `ခါးပတ် မှ ရရှိသော ဒေတာတိုင်းကို` | "each piece of data received from `ခါးပတ်`" |
| `ပန်းကန် အနေဖြင့် ယူ၍` | "taking it as `ပန်းကန်`" |

The loop waits while the channel is open and has no value, and it ends once the channel
is closed and each value in it has come out. When we close a channel, the values that are
still in it come out first.

A duplicate (copy) of a channel value names the same channel, so the thread and the
statements above both reach one queue. The values that we send must be values
that a thread can use, the same as the arguments of a thread.

## One value at a time

To receive just the next value, we use `<channel> မှ ရရှိသော ဒေတာ`, which returns an
[optional value](../optionals/README.md) (option type). It waits while the channel is open
and has no value, and it returns `အလွတ်` once the channel is closed and has no values
left. We can also test for a value with `အကယ်၍`, and then use `<channel> ထဲမှ ဒေတာ`
inside the branch to read the value that came:

```
<channel> မှ ရရှိသော ဒေတာ
အကယ်၍ <channel> ထဲတွင် ဒေတာ ရှိလျှင် … <channel> ထဲမှ ဒေတာ …
```

```zaka
ကိန်းရှင် လမ်း သည် ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
လမ်း ထဲသို့ ၁၀ ကို ပို့မည်။
လမ်း ထဲသို့ ၂၀ ကို ပို့မည်။
လမ်း ကို ပိတ်မည်။
လမ်း မှ ရရှိသော ဒေတာ ကို ပြသမည်။
အကယ်၍ လမ်း ထဲတွင် ဒေတာ ရှိလျှင် (လမ်း ထဲမှ ဒေတာ) * ၂ ကို ပြသမည်။
လမ်း မှ ရရှိသော ဒေတာ ကို ပြသမည်။
```

```text
ဒေတာ(၁၀)
၄၀
အလွတ်
```

The first receive returns `ဒေတာ(၁၀)`, an optional value with `၁၀` in it. The test in
`အကယ်၍` takes the next value, `၂၀`, and the branch prints it twice over. The last receive
finds the channel closed and drained, so it returns `အလွတ်`.

## A channel with a limit

A channel that we declare with a count, such as `ကိန်းပြည့် ၂ ခု ပို့နိုင်သော
ဒေတာလမ်းကြောင်း`, has room for that count of values at most. A send waits while the channel
is full, so a fast sender keeps pace with a slow receiver:

```
<kind> <count> ခု ပို့နိုင်သော ဒေတာလမ်းကြောင်း
```

```zaka
ထုတ် ဟူသော လုပ်ဆောင်ချက် သည် လမ်း ဟူသော ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ မှ ၄ အထိ န အနေဖြင့် ရေတွက်၍ လမ်း ထဲသို့ န ကို ပို့မည်။
လမ်း ကို ပိတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ကန့် သည် ကိန်းပြည့် ၂ ခု ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ထုတ် ဟူသော လုပ်ဆောင်ချက်ကို ကန့် ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
ကန့် မှ ရရှိသော ဒေတာတိုင်းကို တန် အနေဖြင့် ယူ၍ "ရ {တန်}" ဟု ပြသမည်။
```

```text
ရ ၁
ရ ၂
ရ ၃
ရ ၄
```

The thread wants to send four values, but the channel has room for two, so the thread
can never get more than two values ahead of the receiver: once the channel is full, the
next send waits until the program start block receives a value. A parameter can use the
channel kind without the count, as `ထုတ်` does, because the limit is not part of the data
kind.

## Waiting on several channels

A thread that reacts to two channels, such as one of work and one that asks it to stop, cannot
wait on both with a receive. A wait on several channels (select) takes the next value of
the first channel that has one, and runs the arm of that channel:

```
အောက်ပါ ဒေတာလမ်းကြောင်းများထဲမှ ဦးစွာရောက်လာသော ဒေတာကို စောင့်မည်။
<channel> ထဲတွင် ဒေတာ ရှိလျှင် <block>
<n> မီလီစက္ကန့် အတွင်း ဒေတာ မရောက်လာလျှင် <block>
```

```zaka
ထုတ် ဟူသော လုပ်ဆောင်ချက် သည် ရပ် ဟူသော ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၅၀ မီလီစက္ကန့် စောင့်မည်။
ရပ် ထဲသို့ ၀ ကို ပို့မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် အလုပ် သည် ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် ရပ် သည် ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ထုတ် ဟူသော လုပ်ဆောင်ချက်ကို ရပ် ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
အလုပ် ထဲသို့ ၇ ကို ပို့မည်။
အောက်ပါ ဒေတာလမ်းကြောင်းများထဲမှ ဦးစွာရောက်လာသော ဒေတာကို စောင့်မည်။
အလုပ် ထဲတွင် ဒေတာ ရှိလျှင် "အလုပ် {အလုပ် ထဲမှ ဒေတာ}" ဟု ပြသမည်။
ရပ် ထဲတွင် ဒေတာ ရှိလျှင် "ရပ်" ဟု ပြသမည်။
အောက်ပါ ဒေတာလမ်းကြောင်းများထဲမှ ဦးစွာရောက်လာသော ဒေတာကို စောင့်မည်။
အလုပ် ထဲတွင် ဒေတာ ရှိလျှင် "အလုပ် {အလုပ် ထဲမှ ဒေတာ}" ဟု ပြသမည်။
ရပ် ထဲတွင် ဒေတာ ရှိလျှင် "ရပ်" ဟု ပြသမည်။
၁၀ မီလီစက္ကန့် အတွင်း ဒေတာ မရောက်လာလျှင် "စောင့်နေ" ဟု ပြသမည်။
အောက်ပါ ဒေတာလမ်းကြောင်းများထဲမှ ဦးစွာရောက်လာသော ဒေတာကို စောင့်မည်။
အလုပ် ထဲတွင် ဒေတာ ရှိလျှင် "အလုပ် {အလုပ် ထဲမှ ဒေတာ}" ဟု ပြသမည်။
ရပ် ထဲတွင် ဒေတာ ရှိလျှင် "ရပ်" ဟု ပြသမည်။
```

```text
အလုပ် ၇
စောင့်နေ
ရပ်
```

The first wait finds the value in `အလုပ်` at once. The second has a time limit, and no
value comes within 10 milliseconds, so its last arm runs. The third waits until the thread
sends on `ရပ်`. Each arm reads its value with `<channel> ထဲမှ ဒေတာ`, as the test of
`အကယ်၍` does. To keep watching the channels, we put the wait in a loop.

## Details

- **Other spellings.** `ပို့ဆောင်နိုင်သော` means the same as `ပို့နိုင်သော`, and
  `လက်ခံရရှိသော` means the same as `ရရှိသော` ([Channels](../../manual/language/threads.md)).
- **Closing twice.** A send to a closed channel, or a second close, stops the program with
  the trap `pipe closed` ([When a program stops](../../manual/vm/traps.md)).
- **The test and the read.** `<channel> ထဲမှ ဒေတာ` is not a second receive, so no other
  thread can take the value between the test and the read. It works just inside the branch
  of the test, and the test must be the whole condition of `အကယ်၍`, with the channel as a
  name; another use is the error ZK0496. A receive, a loop or a close of a value that is
  not a channel is the error ZK0495.
- **Waiting on several channels.** When values wait in several channels, the wait picks
  one at random; on the VM it picks the first, so a run gives the same output each time. A
  channel that is closed and drained holds no value, and once each one is, the wait runs no
  arm. `မည်သည့်လမ်းကြောင်းတွင်မှ ဒေတာ မရှိလျှင်` in place of a time limit runs at once when no
  channel holds a value ([Waiting on several channels](../../manual/language/threads.md#waiting-on-several-channels)).
- **Deadlock.** A receive from a channel that no thread sends to or closes would wait
  forever, so the program stops with the trap `deadlock` instead
  ([When a program stops](../../manual/vm/traps.md)).

## Recap

- We declare a channel with `<kind> ပို့နိုင်သော ဒေတာလမ်းကြောင်း` and make one with
  `ကနဦးတန်ဖိုး`.
- We send with `<channel> ထဲသို့ <value> ကို ပို့မည်။` and close with
  `<channel> ကို ပိတ်မည်။`.
- `<channel> မှ ရရှိသော ဒေတာတိုင်းကို <name> အနေဖြင့် ယူ၍` receives each value until the
  channel is closed and drained.
- `<channel> မှ ရရှိသော ဒေတာ` receives one value as an optional value.
- A channel with a count holds that count of values at most, and a send waits while it is full.
- `အောက်ပါ ဒေတာလမ်းကြောင်းများထဲမှ ဦးစွာရောက်လာသော ဒေတာကို စောင့်မည်။` waits on several channels and runs the arm of the one whose value came first, or
  the arm of a time limit.

## Hands on

- Start two threads that each send three numbers on one channel, and print each number
  that comes. Who closes the channel?
- Change the limit of the last example to 1 and predict the output before running it.
- Receive from a channel that no thread sends to, and read the message.
- Write a loop that waits on a channel of work and a channel that asks it to stop, and ends once
  the stop comes.

Next: [Sharing between threads](sharing.md).
