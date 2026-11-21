---
title: Threads
description: Run a function on a thread while the rest of the program goes on - starting a thread, the thread kind, waiting for its result, sleeping, the values a thread can use, channels, objects that threads share and their locks, atomic numbers, parallel loops and maps, threads and OS threads, light tasks, the turns on the VM, and what a trap in a thread does.
---

# Threads

A thread (`အပြိုင်လုပ်ငန်း`, a thread or task) runs a function alongside the rest of the
program. To start one, put `အပြိုင်` before the last word of a call. The calling code
can continue and wait for the result later. Threads use the same heap, and a value
passed to a thread is duplicated (copied) when one side writes to it. For examples, see
[Threads](../../learn/threads/README.md) in Learn.

How a thread of a Zaka program maps to an OS thread depends on how the program runs. A
native program runs each thread as a light task on a pool of OS threads, or on an OS
thread of its own, so the threads run at the same time ([Light tasks](#light-tasks)). The
VM runs all threads on one OS thread and takes turns between them in a fixed
order, so a program prints the same lines on each run, and `--shuffle-thread-order`
shuffles those turns ([Threads and OS threads](#threads-and-os-threads)).

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
(ရှေ့ ကို စောင့်သော ရလဒ်) + (နောက် ကို စောင့်သော ရလဒ်) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၁
```

## Starting a thread

```text
<function> ဟူသော လုပ်ဆောင်ချက်ကို <arguments> အပြိုင် လုပ်ဆောင်မည်။
<function> ဟူသော လုပ်ဆောင်ချက်ကို <arguments> အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း
```

To start a thread, we write a long call ([Functions](functions.md)) and put `အပြိုင်`
right before its last word. The first form is a statement that starts the thread and
keeps nothing to wait on. The second form is an expression whose value is the thread, so
we can keep it in a variable and wait for it later.

- We write the arguments as for a plain call, with the same prepositions. Neither
  `အပြိုင်` nor `လုပ်ငန်း` is a reserved word.
- `သီးသန့် အပြိုင်` in place of `အပြိုင်` ("set apart, in parallel") asks for an OS thread
  of its own for the thread in a native program, as [Threads and OS
  threads](#threads-and-os-threads) tells. `သီးသန့်` is not a reserved word either.
- The thread enters the function at once, with its own duplicate of each argument, and
  the program goes on while the thread runs in its own turns.
- A function that returns nothing can run as a thread too, and a wait for it returns no
  value.

## The thread kind

```text
<kind> ပြန်ပေးမည့် အပြိုင်လုပ်ငန်း
အပြိုင်လုပ်ငန်း
```

The value of a thread has a thread kind, which names the data kind (type) that its
function returns. We write `<kind> ပြန်ပေးမည့် အပြိုင်လုပ်ငန်း` for a thread whose
function returns `<kind>`, and `အပြိုင်လုပ်ငန်း` alone for a thread whose function returns
nothing.

A thread value refers to its thread, so a duplicate of the value refers to the same
thread. In messages, the short name of a thread kind is `thread(<kind>)`, such as
`thread(isize)`, and `thread()` for a thread that returns nothing.

## Waiting for a thread

```text
<thread> ကို စောင့်မည်။
<thread> ကို စောင့်သော ရလဒ်
```

We write `<thread> ကို စောင့်သော ရလဒ်` to wait for a thread and use its result as a
value, and `<thread> ကို စောင့်မည်။` to wait for it and drop the result.

- A wait for a thread that has ended returns at once.
- One wait takes the result, as `join` does in Rust: the result moves to the waiter, and
  a second wait for the same thread stops the program with the trap `thread waited`. To
  use the result in several places, we keep it in a variable, or pass it on through a
  [channel](#channels).
- `<thread> ကို စောင့်မည်။` takes the result too, and drops it. When no wait takes the
  result of a thread, the result is dropped as the program ends.
- `စောင့်` is a verb of [the core module](core.md), so a function of the same name in the
  program joins it as an overload ([Verbs on structs](core.md#verbs-on-structs)).

```zaka
အော် ဟူသော လုပ်ဆောင်ချက် သည် စာ ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"{စာ}!" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အော် ဟူသော လုပ်ဆောင်ချက်ကို "မင်္ဂလာ" ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ကိန်းရှင် ရ သည် က ကို စောင့်သော ရလဒ် ဖြစ်ပါစေ။
ရ ကို ပြသမည်။
ရ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာ!
မင်္ဂလာ!
```

A second wait for the same thread stops the program:

```zaka-stop
အော် ဟူသော လုပ်ဆောင်ချက် သည် စာ ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"{စာ}!" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အော် ဟူသော လုပ်ဆောင်ချက်ကို "မင်္ဂလာ" ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
က ကို စောင့်သော ရလဒ် ကို ပြသမည်။
က ကို စောင့်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာ!
the program stopped: the thread 1 this waits for gave its result to an earlier wait
```

## The end of the program

When the program start block ends, the program waits for each thread that is still
running, and then it ends.

```zaka
ပြ ဟူသော လုပ်ဆောင်ချက် သည် စာ ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စာ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပြ ဟူသော လုပ်ဆောင်ချက်ကို "နောက်ကွယ်" ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
"ရှေ့" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရှေ့
နောက်ကွယ်
```

## Sleeping

```text
<count> မီလီစက္ကန့် စောင့်မည်။
<count> စက္ကန့် စောင့်မည်။
```

A thread sleeps for a count of milliseconds with `<count> မီလီစက္ကန့် စောင့်မည်။`, or for a
count of seconds with `<count> စက္ကန့် စောင့်မည်။`, and the other threads go on while it
sleeps. The program start block can sleep too, as it is a thread of its own.

- The count has the data kind `လက္ခဏာမဲ့ ကိန်းပြည့်`, the unsigned integer of the platform.
  A count of seconds is turned into milliseconds before the sleep.
- A sleep is no wait for another thread, so a thread that waits for a sleeping one is no
  deadlock: the program goes on once the sleep ends.
- Both forms are sentence calls of the verb `စောင့်` of [the core module](core.md), which a
  wait for a thread uses too. The preposition `မီလီစက္ကန့်` or `စက္ကန့်` tells a sleep from a
  wait.

```zaka
အိပ် ဟူသော လုပ်ဆောင်ချက် သည် အမည် ဟူသော စာသား တစ်ခု ကို လည်းကောင်း ကြာ ဟူသော လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကြာ မီလီစက္ကန့် စောင့်မည်။
အမည် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အိပ် ဟူသော လုပ်ဆောင်ချက်ကို "နှေး" နှင့် ၃၀၀ တို့ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အိပ် ဟူသော လုပ်ဆောင်ချက်ကို "မြန်" နှင့် ၂၀ တို့ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
"စ" ဟု ပြသမည်။
က ကို စောင့်မည်။
ခ ကို စောင့်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
စ
မြန်
နှေး
```

The two sleeps run at the same time, so the program takes about 300 milliseconds, not the
sum of the two. On the VM, a sleeping thread gives its turn to the next thread; when no
thread can run and one sleeps, the VM waits for the clock to reach the earliest wake. A
native program sleeps on the OS thread of the thread. The explorer and the tests of these
pages run the VM with a clock of their own, which a sleep moves on at once, so the output
comes at once and in the same order.

## Values that go to a thread

Each argument of a thread, and its result, must have a data kind that threads can use,
which means a data kind with the built-in trait `အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော`
("usable in threads", [Built-in traits](traits.md#built-in-traits)). Numbers, text,
lists, plain structs, enumerations, results and tables of these have the trait, and so do
the objects of a struct with that mark ([Objects threads can
use](objects.md#objects-threads-can-use)). A plain shared object (reference type) does not
have it, because two threads could write to it at once, so passing one to a thread is
ZK0494.

```zaka-error ZK0494
အကောင့် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ကြည့် ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော အကောင့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ၏ လက်ကျန် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် အကောင့်(လက်ကျန်: ၅) ဖြစ်ပါစေ။
ကြည့် ဟူသော လုပ်ဆောင်ချက်ကို ခ ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## What a thread can reach

A thread runs at the same time as the rest of the program, so it must not reach a
variable that another thread can change. The compiler checks three rules:

| Rule | Error |
| --- | --- |
| A thread starts in a function that the program declares in Zaka, at the top of a file or inside a function with no captures. A function value, a closure or a C function cannot start a thread, but the function can call them. | ZK0497 |
| The function has no mutable parameter (`ဒေတာ ပြောင်းလဲရန်`) and no writable parameter (`ဒေတာ ထည့်ပေးရန်`), because these would reach a variable of the caller. The thread gets a duplicate of each argument, and returns what it makes through its result or a channel. | ZK0498 |
| A variable at the top of a file belongs to the main thread. The function that a thread runs, and each function it calls, through function values too, neither reads nor writes such a variable. Constants are free to read. | ZK0726 |

```zaka-error ZK0726
ကိန်းရှင် ရေ သည် ၀ ဖြစ်ပါစေ။

တိုး ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ရေ ကို ရေ + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
တိုး ဟူသော လုပ်ဆောင်ချက်ကို အပြိုင် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## Channels

```text
<kind> ပို့နိုင်သော ဒေတာလမ်းကြောင်း
<kind> <n> ခု ပို့နိုင်သော ဒေတာလမ်းကြောင်း
```

A channel (`ဒေတာလမ်းကြောင်း`, a channel or message queue) carries values from one thread
to another, in the order of the sends. We write `<kind> ပို့နိုင်သော ဒေတာလမ်းကြောင်း`
for a channel of `<kind>` values with no limit on how much can wait in it, and
`<kind> <n> ခု ပို့နိုင်သော ဒေတာလမ်းကြောင်း` for a channel where at most `<n>` values
wait. `<n>` is a whole number above zero, written in digits. The measure word (classifier)
of a channel is `ခု`.

- `ပို့ဆောင်နိုင်သော` means the same as `ပို့နိုင်သော`.
- `ကနဦးတန်ဖိုး` makes a new channel, with the room that the declared data kind names.
- A channel value refers to its channel, so a duplicate refers to the same channel, and
  two threads that each have it reach one queue.
- In messages, the short name of a channel kind is `pipe(<kind>)`, such as `pipe(text)`.
  The room is not part of the data kind.

We use these forms to send, receive and close:

| Form | What it does |
| --- | --- |
| `<pipe> ထဲသို့ <value> ကို ပို့မည်။` | sends the value, and waits while a channel with a room is full |
| `<pipe> မှ ရရှိသော ဒေတာ` | the next value, as an [optional](optionals.md): it waits while the channel is open and has no value, and it has no data once the channel is closed and drained |
| `အကယ်၍ <pipe> ထဲတွင် ဒေတာ ရှိလျှင်` | takes the next value as a receive does; the branch runs when a value came, and `<pipe> ထဲမှ ဒေတာ` inside it is that value |
| `<pipe> မှ ရရှိသော ဒေတာတိုင်းကို <name> အနေဖြင့် ယူ၍` | a loop over each value that comes, which ends once the channel is closed and drained |
| `<pipe> ကို ပိတ်မည်။` | closes the channel: the values in it still come, and after them each receive returns no data |

- `လက်ခံရရှိသော` means the same as `ရရှိသော`, and `ဒေတာတိုင်း ကို` the same as
  `ဒေတာတိုင်းကို`.
- The value we send must have a data kind that threads can use, as an argument of a
  thread must.
- `<pipe> ထဲမှ ဒေတာ` is not a second receive, so no other thread can take the value
  between the test and the read. The test must be the whole condition of `အကယ်၍`, its
  channel must be a name, and the read must be inside its branch, or the program has the
  error ZK0496.
- A receive, a loop or a close of a value that is not a channel is ZK0495.
- A send to a closed channel, or a second close, stops the program with the trap
  `pipe closed`.

```zaka
ထုတ် ဟူသော လုပ်ဆောင်ချက် သည် လမ်း ဟူသော စာသား ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ လမ်း ထဲသို့ "စာကြောင်း {န}" ကို ပို့မည်။
လမ်း ကို ပိတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လိုင်း သည် စာသား ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ထုတ် ဟူသော လုပ်ဆောင်ချက်ကို လိုင်း ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
လိုင်း မှ ရရှိသော ဒေတာတိုင်းကို စာ အနေဖြင့် ယူ၍ စာ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
စာကြောင်း ၁
စာကြောင်း ၂
စာကြောင်း ၃
```

A channel with a room holds the sender back. Here the room is 2, so the third send waits
until the program has received the first value. The program uses each of the three forms
of a receive, and the receiving side alone prints, so the output is the same whatever
the order of the turns:

```zaka
ထုတ် ဟူသော လုပ်ဆောင်ချက် သည် လမ်း ဟူသော ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ မှ ၄ အထိ န အနေဖြင့် ရေတွက်၍ လမ်း ထဲသို့ န ကို ပို့မည်။
လမ်း ကို ပိတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကန့် သည် ကိန်းပြည့် ၂ ခု ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ထုတ် ဟူသော လုပ်ဆောင်ချက်ကို ကန့် ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
ကန့် မှ ရရှိသော ဒေတာ ကို ပြသမည်။
အကယ်၍ ကန့် ထဲတွင် ဒေတာ ရှိလျှင် "ရ {ကန့် ထဲမှ ဒေတာ}" ဟု ပြသမည်။
ကန့် မှ ရရှိသော ဒေတာတိုင်းကို တန် အနေဖြင့် ယူ၍ "ကျန် {တန်}" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဒေတာ(၁)
ရ ၂
ကျန် ၃
ကျန် ၄
```

```zaka-error ZK0496
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လမ်း သည် ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
လမ်း ထဲမှ ဒေတာ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## Waiting on several channels

```text
အောက်ပါ ဒေတာလမ်းကြောင်းများထဲမှ ဦးစွာရောက်လာသော ဒေတာကို စောင့်မည်။
<pipe> ထဲတွင် ဒေတာ ရှိလျှင် <block>
<pipe> ထဲတွင် ဒေတာ ရှိလျှင် <block>
မည်သည့်လမ်းကြောင်းထံမှ ဒေတာမရောက်လာနိုင်တော့လျှင် <block>
<n> မီလီစက္ကန့် အတွင်း ဒေတာ မရောက်လာလျှင် <block>
မည်သည့်လမ်းကြောင်းတွင်မှ ဒေတာ မရှိလျှင် <block>
```

A wait on several channels (select) receives one value from a channel with data and
runs the matching arm. After the opening line, write one arm per channel. Each arm
uses the same channel test as `အကယ်၍`. After the channel arms, we can add a closed
arm for when all channels are closed and drained, then a time limit arm or a no-data
arm. The first statement outside these forms marks the end of the wait.

- The wait removes the value from the channel before it runs the matching arm.
  Within that arm, `<pipe> ထဲမှ ဒေတာ` reads the received value.
- If several channels hold data, a native program picks one of them at random. The
  VM takes them in turn: each pick goes to the next matching arm, so no arm starves
  and a run gives the same output each time. With
  `--shuffle-thread-order=<seed>`, the VM draws from the channels with data using
  the seed. With a fixed seed, the choices are repeatable.
- Once all channels in the wait are closed and drained, the wait runs the closed arm,
  `မည်သည့်လမ်းကြောင်းထံမှ ဒေတာမရောက်လာနိုင်တော့လျှင်`. We can also write
  `ဒေတာ မရောက်လာနိုင်တော့လျှင်` with a space. If there is no closed arm, the wait
  runs the no-data arm if present, or continues without running an arm. To leave a
  loop once all channels are closed and drained, use an arm that exits the loop or
  provide another exit.
- `<n> မီလီစက္ကန့် အတွင်း ဒေတာ မရောက်လာလျှင်` runs its block when no value came within `<n>`
  milliseconds, and `<n> စက္ကန့် အတွင်း ဒေတာ မရောက်လာလျှင်` within `<n>` seconds. `<n>` is a
  `usize`, as the count of a [sleep](#sleeping) is.
- `မည်သည့်လမ်းကြောင်းတွင်မှ ဒေတာ မရှိလျှင်` runs at once if no channel holds a
  value and the closed arm does not run. A tight loop around this form can keep a
  CPU core active.
- To receive more than one value, put the wait in a loop.
- In a native program, a light task that waits gives its OS thread to the other tasks, as
  a receive does.
- If present, the closed arm comes before the time limit or no-data arm. A wait needs
  at least one channel arm (ZK0264) and can have at most one of the time limit and
  no-data arms (ZK0265). The value tested in a channel arm must be a channel (ZK0495).
  Reading `<pipe> ထဲမှ ဒေတာ` from the arm of a different channel raises ZK0496.

Here a thread sends three values, waits, then sends on the channel `ရပ်`. The loop prints
each value, prints a line each time 40 milliseconds pass with no value, and ends once
`ရပ်` has a value:

```zaka
ထုတ် ဟူသော လုပ်ဆောင်ချက် သည် အလုပ် ဟူသော ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ကို လည်းကောင်း ရပ် ဟူသော ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ အလုပ် ထဲသို့ န ကို ပို့မည်။
၁၀၀ မီလီစက္ကန့် စောင့်မည်။
ရပ် ထဲသို့ ၀ ကို ပို့မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အလုပ် သည် ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် ရပ် သည် ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ထုတ် ဟူသော လုပ်ဆောင်ချက်ကို အလုပ် နှင့် ရပ် တို့ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
ကိန်းရှင် ဆက် သည် မှန် ဖြစ်ပါစေ။
ဆက် ဖြစ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါ ဒေတာလမ်းကြောင်းများထဲမှ ဦးစွာရောက်လာသော ဒေတာကို စောင့်မည်။
အလုပ် ထဲတွင် ဒေတာ ရှိလျှင် "အလုပ် {အလုပ် ထဲမှ ဒေတာ}" ဟု ပြသမည်။
ရပ် ထဲတွင် ဒေတာ ရှိလျှင် ဆက် ကို မမှန် ဟု ထားမည်။
၄၀ မီလီစက္ကန့် အတွင်း ဒေတာ မရောက်လာလျှင် "စောင့်နေ" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
"ပြီး" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အလုပ် ၁
အလုပ် ၂
အလုပ် ၃
စောင့်နေ
စောင့်နေ
ပြီး
```

With the closed arm, a loop ends once the sender has closed each channel and the loop has
taken each value:

```zaka
ထုတ် ဟူသော လုပ်ဆောင်ချက် သည် အလုပ် ဟူသော ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ကို လည်းကောင်း ရပ် ဟူသော ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ အလုပ် ထဲသို့ န ကို ပို့မည်။
အလုပ် ကို ပိတ်မည်။
ရပ် ကို ပိတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အလုပ် သည် ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် ရပ် သည် ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ထုတ် ဟူသော လုပ်ဆောင်ချက်ကို အလုပ် နှင့် ရပ် တို့ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
ကိန်းရှင် ဆက် သည် မှန် ဖြစ်ပါစေ။
ဆက် ဖြစ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါ ဒေတာလမ်းကြောင်းများထဲမှ ဦးစွာရောက်လာသော ဒေတာကို စောင့်မည်။
အလုပ် ထဲတွင် ဒေတာ ရှိလျှင် "အလုပ် {အလုပ် ထဲမှ ဒေတာ}" ဟု ပြသမည်။
ရပ် ထဲတွင် ဒေတာ ရှိလျှင် "ရပ်" ဟု ပြသမည်။
မည်သည့်လမ်းကြောင်းထံမှ ဒေတာမရောက်လာနိုင်တော့လျှင် ဆက် ကို မမှန် ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
"ပြီး" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အလုပ် ၁
အလုပ် ၂
အလုပ် ၃
ပြီး
```

With the arm of no data, the wait looks once and goes on. The first wait finds no
value in the channel, and the second finds the value that came between them:

```zaka
ကိန်းရှင် လမ်း သည် ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
အောက်ပါ ဒေတာလမ်းကြောင်းများထဲမှ ဦးစွာရောက်လာသော ဒေတာကို စောင့်မည်။
လမ်း ထဲတွင် ဒေတာ ရှိလျှင် "ရ {လမ်း ထဲမှ ဒေတာ}" ဟု ပြသမည်။
မည်သည့်လမ်းကြောင်းတွင်မှ ဒေတာ မရှိလျှင် "မရှိသေး" ဟု ပြသမည်။
လမ်း ထဲသို့ ၅ ကို ပို့မည်။
အောက်ပါ ဒေတာလမ်းကြောင်းများထဲမှ ဦးစွာရောက်လာသော ဒေတာကို စောင့်မည်။
လမ်း ထဲတွင် ဒေတာ ရှိလျှင် "ရ {လမ်း ထဲမှ ဒေတာ}" ဟု ပြသမည်။
မည်သည့်လမ်းကြောင်းတွင်မှ ဒေတာ မရှိလျှင် "မရှိသေး" ဟု ပြသမည်။
```

```text
မရှိသေး
ရ ၅
```

```zaka-error ZK0265
ကိန်းရှင် လမ်း သည် ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
အောက်ပါ ဒေတာလမ်းကြောင်းများထဲမှ ဦးစွာရောက်လာသော ဒေတာကို စောင့်မည်။
လမ်း ထဲတွင် ဒေတာ ရှိလျှင် "ရ" ဟု ပြသမည်။
၁၀ မီလီစက္ကန့် အတွင်း ဒေတာ မရောက်လာလျှင် "ကုန်" ဟု ပြသမည်။
မည်သည့်လမ်းကြောင်းတွင်မှ ဒေတာ မရှိလျှင် "မရှိ" ဟု ပြသမည်။
```

## Objects threads share

```text
<object> ကို (ခေတ္တ|ခဏ)ယူ၍ <statement>
<object> ကို (ခေတ္တ|ခဏ)ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
<block>
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

When a struct has the mark `အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော` ([Objects threads can
use](objects.md#objects-threads-can-use)), its objects go to a thread as the object itself,
not as a duplicate, so each thread that has the object reaches one set of fields. To read
or write a field of such an object, we take its lock with a lock block, using one of the
forms above.

- Each field of such an object is read and written inside a lock block of the object, or
  the program has the error ZK0499. A thread that takes a lock that another thread has
  waits until that block ends.
- `<object>` must be a name whose value is a shared object, or the program has the error
  ZK0499 too. We can write `ယူ၍` glued (`ခေတ္တယူ၍`) or apart, and neither `ခေတ္တ` nor
  `ခဏ` is a reserved word.
- A thread that has a lock can take it again, as a call inside the block does. Each exit
  from the block gives the lock back: its end, a `ပြန်ပေးမည်`, a break or a skip to the
  next round.
- A thread that stops at a trap while it has a lock leaves the lock broken, and the next
  lock block of that object stops its thread with the trap `lock broken`.
- Two threads that each wait for a lock that the other has stop with the trap
  `deadlock`.

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

```zaka-error ZK0499
အကောင့် ဟူသော အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဘဏ် သည် အကောင့်(လက်ကျန်: ၀) ဖြစ်ပါစေ။
ဘဏ် ၏ လက်ကျန် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## Atomic numbers

```text
အက်တမ်မစ် ဖြစ်သော <kind>
<atomic> ထဲသို့ <value> ကို ပေါင်းထည့်မည်။
<atomic> ထဲမှ <value> ကို နုတ်ယူမည်။
<atomic> ထဲရှိ <expected> ကို <new> ဖြင့် အစားထိုးသော ရလဒ်
```

An atomic number (`အက်တမ်မစ်`, a term from computer science) is an integer that several
threads can add to and subtract from at once, with no lock and without losing a change.
We write `အက်တမ်မစ် ဖြစ်သော <kind>` for its data kind, where `<kind>` is `ကိန်းပြည့်` or
`လက္ခဏာမဲ့ ကိန်းပြည့်` of the platform width; another data kind is ZK0401.

| Form | What it does |
| --- | --- |
| `<atomic> ထဲသို့ <value> ကို ပေါင်းထည့်မည်။` | adds the value in one step that no other thread can split |
| `<atomic> ထဲမှ <value> ကို နုတ်ယူမည်။` | subtracts the value in one step |
| `<atomic> ထဲရှိ <expected> ကို <new> ဖြင့် အစားထိုးသော ရလဒ်` | sets the number to `<new>` when it has the value `<expected>`, in one step, and returns a result |

- A read of an atomic number returns its integer kind, and a set uses that kind. A read
  or a set of an atomic field of an object is one step too: a read sees each change that
  another thread made before it, and a set reaches each other thread whole.
- A set whose value reads the same number, such as `x ကို x + ၁ ဟု ထားမည်`, is ZK0727,
  because another thread could change the number between the read and the write.
- An add and a subtraction wrap around past the range of the data kind, and do not stop
  the program.
- A field that has an atomic number, in an object that threads share, is read, set and
  added to outside a lock block.

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
ရေတွက် ၏ ကြိမ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၀၀
```

```zaka-error ZK0727
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကြိမ် သည် အက်တမ်မစ် ဖြစ်သော ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
ကြိမ် ကို ကြိမ် + ၁ ဟု ထားမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

### Compare and set

The form with `အစားထိုးသော ရလဒ်` returns a [result](error-handling.md) whose two sides
have the integer kind of the number, as `compare_exchange` does in Rust. When the number
took `<new>`, the result is a good result with the value that the number had before, which
is `<expected>`. Otherwise the result is an error value with the value that the number had
in its place, and the number is left as it was.

- We use `ထဲတွင် အောင်မြင်သည့်သတင်း ရှိ` to test whether the number took `<new>`, and
  `ထဲမှ ဒေတာ` and `ထဲမှ အမှားအယွင်း ဒေတာ` to read the two sides ([Reading a
  result](reading-results.md)).
- `<expected>` and `<new>` can read the same number without the error ZK0727, because no
  thread can come between the test and the set.
- `<atomic>` must be a variable or a field that has an atomic number; another place is
  ZK0332. As a statement, the result would go unread, which is ZK0443.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် x သည် အက်တမ်မစ် ဖြစ်သော ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
x ထဲရှိ ၅ ကို ၆ ဖြင့် အစားထိုးသော ရလဒ် ကို ပြသမည်။
x ထဲရှိ ၅ ကို ၇ ဖြင့် အစားထိုးသော ရလဒ် ကို ပြသမည်။
x ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရလဒ်ကောင်း(၅)
အမှားအယွင်း(၆)
၆
```

In the next example, three threads race to claim an object. The first thread that sets
the owner from 0 wins, and the other threads see the owner that it set:

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

## Parallel loops and maps

```text
<from> မှ <to> အထိ <name> အနေဖြင့် အပြိုင် ရေတွက်၍ <statement>
<list> ၏ အကွက်တိုင်း ကို <name> အနေဖြင့် အပြိုင် ယူ၍ <statement>
<list> ၏ အကွက်တိုင်း ကို <number> နှင့် <name> အနေဖြင့် အပြိုင် ယူ၍ <statement>
<from> မှ <to> အထိ <name> အနေဖြင့် အပြိုင်လုပ်ငန်း <n> ခုခွဲ၍ ရေတွက်၍ <statement>
```

To let the rounds of a loop run at the same time, we put `အပြိုင်` right before the last
word of the loop. Each of these loops has the block form too.

To choose the count of threads, we write `အပြိုင်လုပ်ငန်း <n> ခုခွဲ၍` ("split into `<n>`
threads") in place of `အပြိုင်`, before the verb of the loop (`ရေတွက်၍`, `တိုး၍`,
`လျှော့၍` or `ယူ၍`), or before `လုပ်ဆောင်သော ရလဒ်များ` in a parallel map.

- `<n>` is an integer (ZK0405), worked out as the loop starts. Zero or a negative count
  stops the program with the trap `threads not positive`.
- The rounds then run on `<n>` threads, but never on more threads than rounds. Each
  thread takes the next block of rounds that no thread has taken, until none are left, so
  a thread whose rounds end fast takes more of them. A count of 1 runs the rounds in
  order on the thread of the loop.
- With no count, a loop uses one thread for each core.
- A counted loop whose counter is wider than the platform word (an `i64` counter in a
  build with `--target-bits 32`) runs its rounds in order, because their count could pass
  the word.

### What a round can do

The rounds run at the same time on several threads, so the rounds follow the rules of a thread:

| Rule | Error |
| --- | --- |
| A round writes the variables that it declares, and one cell of a list from outside the loop: the cell of its own number, `<list> ၏ အကွက် <number>`. Each other write from a round is an error, and so is an argument that lets a call change a variable from outside. | ZK0728 |
| Each variable from outside that a round reads has a value that threads can use, so it is not a plain shared object or a closure from outside the round. | ZK0494 |
| Each function that a round reaches neither reads nor writes a variable at the top of a file. | ZK0726 |
| A round cannot stop the loop with `ကွင်းဆက် ကို ရပ်မည်။`, or return from the function around the loop. `နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်။` ends just the round. | ZK0729 |

An add to an [atomic number](#atomic-numbers) from outside the loop, with `ပေါင်းထည့်မည်` or
`နုတ်ယူမည်`, is no write in the sense of ZK0728: each change is one step that no other round
can split, so the rounds can count into one number. A read of an atomic number from outside
the loop is one step too, so a round sees each change that another round made before it.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပေါင်း သည် အက်တမ်မစ် ဖြစ်သော ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
၁ မှ ၁၀၀ အထိ i အနေဖြင့် အပြိုင် ရေတွက်၍ ပေါင်း ထဲသို့ i ကို ပေါင်းထည့်မည်။
ပေါင်း ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၅၀၅၀
```

### Where the rounds run

A native program runs the rounds on OS threads, one for each core of the host, when the
count of rounds is at least the count of cores. With fewer rounds, the rounds run in order
on the thread of the loop, and the VM also runs them in order.

- Each counted loop, with a step or without one, spreads its rounds, and so does each
  loop over the cells of a list, the characters of text or the rows of a ဇယား (table). A
  loop over the values of a channel runs its rounds in order, because the values come one
  at a time.
- A round over a row of a ဇယား that has been removed ends at once.
- In a native program, the lines that the rounds print come in no fixed order.
- A trap in a round acts as a trap in a thread does: the rounds of its thread end, the
  other threads finish, and the loop then stops with `thread trapped`. Under
  `--trap abort`, a native program stops at once ([A trap in a
  thread](#a-trap-in-a-thread)). Rounds that run in order stop at the trap.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ထွက် သည် အကွက် ၅ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
၀ မှ ၄ အထိ i အနေဖြင့် အပြိုင် ရေတွက်၍ ထွက် ၏ အကွက် i ကို i * i ဟု ထားမည်။
ထွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၀၊ ၁၊ ၄၊ ၉၊ ၁၆]
```

```zaka-error ZK0728
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စု သည် ၀ ဖြစ်ပါစေ။
၀ မှ ၄ အထိ i အနေဖြင့် အပြိုင် ရေတွက်၍ စု ကို စု + i ဟု ထားမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

### Parallel maps

```text
<list> ၏ အကွက်တိုင်း ကို <function> ဟူသော လုပ်ဆောင်ချက် ဖြင့် အပြိုင် လုပ်ဆောင်သော ရလဒ်များ
<list> ၏ အကွက်တိုင်း ကို <function> ဟူသော လုပ်ဆောင်ချက် ဖြင့် အပြိုင်လုပ်ငန်း <n> ခုခွဲ၍ လုပ်ဆောင်သော ရလဒ်များ
```

A parallel map is an expression that calls a function on each cell of a list and returns
a new plain list of the results, in the order of the cells.

- The function has one parameter that the cells fit, and it follows the rules of the
  function that a thread starts in ([What a thread can reach](#what-a-thread-can-reach)),
  so the cells and its result must have data kinds that threads can use.
- A native program runs the rounds as it runs those of a parallel loop, when the cells
  have the data kind of the parameter, and the VM runs them in order. A trap in a round
  acts as it does in a parallel loop.

```zaka
စကားလုံးရေ ဟူသော လုပ်ဆောင်ချက် သည် စာ ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
စာ ၏ စာလုံးအရေအတွက် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာများ သည် ["က"၊ "ခဂ"၊ "ဃငစ"] ဖြစ်ပါစေ။
ကိန်းရှင် ရေများ သည် စာများ ၏ အကွက်တိုင်း ကို စကားလုံးရေ ဟူသော လုပ်ဆောင်ချက် ဖြင့် အပြိုင် လုပ်ဆောင်သော ရလဒ်များ ဖြစ်ပါစေ။
ရေများ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၂၊ ၃]
```

## Threads and OS threads

Where a thread runs depends on how the program runs:

- **The VM** runs all threads on one OS thread and takes turns between them, as the next
  section describes.
- **A native program**, from the C target or the native target ([Native
  programs](../compiler/native.md)), runs a thread as a light task when it can, and on an
  OS thread of its own when it cannot ([Light tasks](#light-tasks)). The threads run at
  the same time on the cores of the computer, so the order of their output lines can
  change from run to run, but what a thread returns and what a channel carries cannot. A
  wait for a thread, a receive or a send that would wait while no other thread runs stops
  the program with the trap `deadlock`, as on the VM. `zakac build --trap abort` makes a
  trap in a thread stop the whole program.
- **The JIT** of `zakac run`, which runs a source file as native code when it can, runs
  threads as light tasks or on OS threads, and parallel loops and maps on OS threads, as
  a native program does, and
  `zakac run --trap abort` makes a trap in a thread stop the whole program there too.
- While a thread runs, a native program collects no rings (objects that reach each other in
  a loop): the collector changes the reference counts with plain operations, so it waits
  until each thread has ended. The buffer still fills, and the end of the program
  collects as it does in all programs.

## Light tasks

A light task is a thread on a small stack of its own, 1 MiB, which the OS fills as the
stack grows, and a pool of OS threads, one for each core, takes turns on the tasks. Ten
thousand light tasks fit in a program where ten thousand OS threads, at 64 MiB of stack
each, do not. For each thread that a call starts, the compiler picks a light task or an OS
thread, and the words of the program are the same for both. A thread gets an OS thread of
its own when its function, or a function it calls, through function values of its kind
too:

- calls a C function (`"libX" မှလာသော`), which can keep state for each OS thread or hold
  its OS thread in a call that blocks;
- calls itself again, through other functions as well, since the stack of a task is small;
- reads standard input, which can wait with no end;
- keeps more than 16 KiB of locals in one function, more than the guard of the stack of a
  task, which a frame must never step past.

A thread written `သီးသန့် အပြိုင်` in place of `အပြိုင်` gets an OS thread of its own as well.

| | Light task | OS thread |
| --- | --- | --- |
| Stack | 1 MiB, filled as it grows | 64 MiB |
| In one program | tens of thousands | a few thousand |
| While it waits for a thread, a channel, a lock, a sleep or a file | its OS thread runs the next task | its OS thread waits |
| A long loop with no wait | gives its OS thread to the next task after 10 milliseconds, while other tasks wait | the OS shares the cores |
| C functions, standard input, a function that calls itself again, more than 16 KiB of locals in a function | none (the compiler picks an OS thread) | allowed |

- Parallel loops and maps run their rounds on OS threads.
- Light tasks run on macOS, Linux and Windows, where each task is a fiber of the OS. In a
  C lib (`--emit lib`), each thread runs on an OS thread of its own, and a program for
  WASI starts no threads.
- A light task whose calls take more than its stack stops at the trap `stack full`. Under
  `--trap unwind`, the default, the thread ends alone, as at each other trap (`thread N
  stopped: the stack of a light task, N octets, ran full`), and a wait for it stops the
  waiter. Under `--trap abort`, and on Windows in each mode, the program stops, after
  the output it printed so far. An OS thread of its own, with `သီးသန့် အပြိုင်`, has room for
  such a thread.
- On the VM, the choice changes nothing: each thread takes turns there
  ([Turns on the VM](#turns-on-the-vm)).

## Turns on the VM

The VM runs each thread in turns of a fixed count of operations (100), in the order that
the threads started. Because this order never changes, a program prints the same lines on
each run. A thread gives its turn to the next thread when it waits for a thread that has
not ended, receives from an open channel that has no value, sends to a full channel, or
sleeps ([Sleeping](#sleeping)).
Each line that a print writes comes out whole, on the VM and on OS threads alike.

A native program runs its threads in a different order on each run, so a mistake that
depends on the order can hide on the VM. To bring it out, we can run
`zakac run --shuffle-thread-order`, which runs the program on the VM and shuffles the
turns: each turn runs 1 to 200 operations and goes to one of the threads that can run,
both drawn from a seed. The run writes the seed it drew to standard error, and
`--shuffle-thread-order=<seed>` runs that order again ([Commands](../compiler/commands.md#run)).
The flag cannot be used together with `--jit`.

In the explorer, the setting "Turns of threads" shuffles the turns as the flag does, from
a seed that each run draws, and the `THREAD` register of the debugger shows the thread
that runs, what each thread waits for, and the seed.

Each thread gets its own stack region, of the size that `--max-stack` sets
([Commands](../compiler/commands.md#run)).

## A trap in a thread

A trap ([When a program stops](../vm/traps.md)) cannot be caught, and the `--trap` option
chooses what a trap in a thread stops:

| `--trap` | A trap in a thread |
| --- | --- |
| `unwind` (the default) | ends that thread alone: the reason goes to standard error, and the rest of the program goes on |
| `abort` | stops the whole program, on the VM, in the JIT of `zakac run` and in a program that `zakac build` makes |

- Under `unwind`, a wait for a thread that stopped at a trap stops the waiter in turn,
  with the trap `thread trapped`.
- When a thread stopped at a trap that no wait saw, the program ends with exit code 1.
- A trap in the program start block stops the whole program in both modes.
- When each thread waits, for another thread or for a channel, and none can go on, the
  program stops with the trap `deadlock`, as with a receive from a channel that no thread
  sends to or closes.
- A thread that stops at a trap while it has a lock leaves the lock broken ([Objects
  threads share](#objects-threads-share)).

## Threads that C starts

C can start its own OS threads, and two of the rules above reach them:

- A native program can hand C a function, and C can call it on an OS thread that C
  started. The call runs when the function, and each function it calls, neither reads nor
  writes a variable at the top of a file (the rule of ZK0726), and when each variable it
  captures is one that no statement writes, with a data kind that threads can use. An
  object it captures must have the mark `အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော`, and
  the function reaches it in a lock block, as a thread does. A write to a field of the
  object does not count as a write to the variable. Each other call from such an OS
  thread ends the run with a message ([Closures and
  C](closures.md#closures-and-c)).
- Each OS thread of C takes locks under its own number, so it waits for a lock that the
  main thread or another OS thread of C has. A wait never stops as a deadlock while an OS
  thread of C can still change what it waits for: while such a call runs, or while C has a
  closure with captures that it can call from its own OS thread. Once the OS thread of C
  leaves its call, the wait checks again.
- C can call a [lib built from Zaka](functions-exports.md) from several OS
  threads. A function of the lib that reaches no variable at the top of a file, calls no C
  function and no function value, and starts no thread runs on those OS threads at once,
  with no lock.

## Errors

| Code | When |
| --- | --- |
| ZK0401 | an atomic number of a data kind other than `ကိန်းပြည့်` or `လက္ခဏာမဲ့ ကိန်းပြည့်` |
| ZK0405 | the count of threads of a parallel loop or map is not an integer |
| ZK0494 | a value that goes to a thread or comes back from one, or that a round reads, has a data kind that threads cannot use |
| ZK0495 | a receive, a loop over each value or a close of a value that is not a channel |
| ZK0496 | a test of a channel for data that is not the whole condition of `အကယ်၍`, or a read of its data outside that branch |
| ZK0497 | a thread starts in a function value, a closure or a C function |
| ZK0498 | a thread starts in a function with a mutable or a writable parameter |
| ZK0499 | a field of an object that threads share is used outside a lock block, or a lock block names a value that is not a shared object |
| ZK0726 | a function that a thread runs reaches a variable at the top of a file |
| ZK0727 | a set of an atomic number whose value reads the same number |
| ZK0728 | a round of a parallel loop writes a variable from outside the loop |
| ZK0729 | a round of a parallel loop breaks out of the loop or returns from the function |
