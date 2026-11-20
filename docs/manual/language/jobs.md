---
title: Jobs
description: Work that runs side to side with the rest of the program: starting a job, the job kind, waiting for its result, the values that can go to a job, what a job can reach, objects jobs share and their locks, atomic numbers, parallel loops and maps, channels between jobs, how the VM takes turns, the end of the program, and what a trap in a job does.
---

# Jobs

A job (`အပြိုင်လုပ်ငန်း`) runs a function side to side with the rest of the program. The
program starts it with a call that ends in `အပြိုင်`, goes on with its own work, and waits
for the result when it needs it. Each job reaches the same heap: a value that goes to a job
is not copied until one side writes it. The Learn book starts in small steps with
[Jobs](../../learn/jobs/README.md).

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

## Starting a job

A long call ([Functions](functions.md)) runs as a job when `အပြိုင်` stands before its end:

| Form | What it does |
| --- | --- |
| `<fn> ဟူသော လုပ်ဆောင်ချက်ကို <args> အပြိုင် လုပ်ဆောင်မည်။` | starts the job; no name holds it |
| `<fn> ဟူသော လုပ်ဆောင်ချက်ကို <args> အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း` | starts the job and gives it, to hold and wait for |

- The arguments are written as for a plain call, with the same prepositions; `အပြိုင်` and
  `လုပ်ငန်း` are no reserved words.
- The job enters the function at once, with its own copies of the arguments. Then the
  program goes on; the job runs in its own turns.
- A function that gives nothing can run as a job too; a wait for it gives no value.

## The job kind

The kind of a job names the kind its function gives back:

| Kind | Holds |
| --- | --- |
| `<kind> ပြန်ပေးမည့် အပြိုင်လုပ်ငန်း` | a job whose function gives `<kind>` |
| `အပြိုင်လုပ်ငန်း` | a job whose function gives nothing |

A job value names its job; a duplicate names the same job. Its short name in messages is
`job(<kind>)`, such as `job(isize)`, and `job()` for a job that gives nothing.

## Waiting for a job

| Form | What it does |
| --- | --- |
| `<job> ကို စောင့်မည်။` | waits for the end of the job; a result is dropped |
| `<job> ကို စောင့်သော ရလဒ်` | waits, then gives the result |

- A wait for a job that has ended gives at once. A job can be waited for more than once,
  and each wait gives the same result: a list or text result is shared, as a duplicate is.
- `စောင့်` is a verb of [the core](core.md); a function of its own name in the program
  stands beside it.

```zaka
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
မင်္ဂလာ!
```

## Values that go to a job

Each argument of a job, and its result, must be of a kind jobs on other threads can use:
the kinds with the built-in trait `အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော`
([Built-in traits](generics.md#built-in-traits)). Numbers, text, lists, plain structs,
enumerations, results and tables of these have it, and so do the objects of a struct with
the mark ([Objects jobs can use](objects.md#objects-jobs-can-use)). A plain shared object
lacks it: two jobs would write it at once.

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

## What a job can reach

A job runs side to side with the rest of the program, so it reaches no variable another
thread can change:

- A job starts in a function the program declares in Zaka: at the top of a file, or inside
  a function with no captures. A function value, a closure and a C function are ZK0497;
  the function can call them.
- The function changes no parameter: one marked `ဒေတာ ပြောင်းလဲရန်` or `ဒေတာ ထည့်ပေးရန်`
  would reach a variable of the caller, ZK0498. The job gets a duplicate of each argument
  and gives back what it makes through its result or a channel.
- A variable at the top of a file belongs to the main thread. The function a job runs,
  and each function it calls, through function values too, reads and writes none of
  them, ZK0726; constants are free to read.

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

## Objects jobs share

An object whose struct carries the mark `အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော`
([Objects jobs can use](objects.md#objects-jobs-can-use)) goes to a job as the object
itself: each job that holds it reaches one set of fields. A lock block holds its lock:

```text
<obj> ကို (ခေတ္တ|ခဏ)ယူ၍ <stmt>
<obj> ကို (ခေတ္တ|ခဏ)ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
<block>
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

- Each field of such an object is read and written inside a lock block of it, ZK0499. A
  job that takes a lock another job holds waits until that block ends.
- `<obj>` is a name that holds a shared object; another value is ZK0499 too. `ယူ၍` reads
  glued (`ခေတ္တယူ၍`) or apart, and neither `ခေတ္တ` nor `ခဏ` is a reserved word.
- A job that holds a lock can take it again, as a call inside the block does. Each exit
  from the block gives the lock back: its end, a `ပြန်ပေးမည်`, a break or a next round.
- A job that stops at a trap while it holds a lock leaves the lock broken: the next lock
  of the object stops its job with the trap `lock broken`.
- Two jobs that each wait for a lock the other holds stop with the trap `deadlock`.

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

An atomic number (`အက်တမ်မစ်`, a term of computer science) is an integer that jobs add to
and subtract from at once with no lock and lose no change:

| Form | What it does |
| --- | --- |
| `အက်တမ်မစ် ဖြစ်သော <kind>` | the kind of an atomic number; `<kind>` is `ကိန်းပြည့်` or `လက္ခဏာမဲ့ ကိန်းပြည့်`, of the platform width, ZK0401 for another |
| `<atomic> ထဲသို့ <value> ကို ပေါင်းထည့်မည်။` | adds the value in one step no other job can split |
| `<atomic> ထဲမှ <value> ကို နုတ်ယူမည်။` | subtracts the value in one step |

- A read of an atomic number is a plain read, of its integer kind, and a set is a plain
  set. A set whose value reads the same number, such as `x ကို x + ၁ ဟု ထားမည်`, is
  ZK0727: a job could change the number between the read and the write.
- An add and a subtraction wrap past the range of the kind, with no stop.
- A field of an object jobs share that holds an atomic number is read, set and added to
  outside a lock block.

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

## Channels

A channel (`ဒေတာလမ်းကြောင်း`) carries values from one job to another, in the order of the
sends. Its kind names the kind of its values, and its measure word is `ခု`:

| Kind | Holds |
| --- | --- |
| `<kind> ပို့နိုင်သော ဒေတာလမ်းကြောင်း` | a channel of `<kind>` values, with no limit on the values that wait in it |
| `<kind> <n> ခု ပို့နိုင်သော ဒေတာလမ်းကြောင်း` | a channel where `<n>` values wait at most; `<n>` is a whole number above zero, in digits |

`ပို့ဆောင်နိုင်သော` reads the same as `ပို့နိုင်သော`. `ကနဦးတန်ဖိုး` gives a new channel, with
the room the kind phrase of the declaration names. A channel value names its channel: a
duplicate names the same one, so two jobs that hold it reach one queue. Its short name in
messages is `pipe(<kind>)`, such as `pipe(text)`; the room is no part of the kind.

| Form | What it does |
| --- | --- |
| `<pipe> ထဲသို့ <value> ကို ပို့မည်။` | sends the value; it waits while a channel with a room is full |
| `<pipe> မှ ရရှိသော ဒေတာ` | the next value, as an optional ([Optional values](optionals.md)): it waits while the channel is open and holds none, and holds no data once the channel is closed and drained |
| `အကယ်၍ <pipe> ထဲတွင် ဒေတာ ရှိလျှင်` | takes the next value as the receive does; the branch runs when a value came, and `<pipe> ထဲမှ ဒေတာ` inside it names that value |
| `<pipe> မှ ရရှိသော ဒေတာတိုင်းကို <name> အနေဖြင့် ယူ၍` | a loop over each value that comes, which ends once the channel is closed and drained |
| `<pipe> ကို ပိတ်မည်။` | closes the channel: the values in it still come, then each receive gives no data |

- `လက်ခံရရှိသော` reads the same as `ရရှိသော`, and `ဒေတာတိုင်း ကို` the same as `ဒေတာတိုင်းကို`.
- The value a send takes must be of a kind jobs can use, as for an argument of a job.
- `<pipe> ထဲမှ ဒေတာ` is no second receive: no other job can take the value between the test
  and the read. The test stands alone as the condition of `အကယ်၍`, its channel is a name,
  and the read stands inside its branch, ZK0496. A receive, a loop or a close of a value
  that is no channel is ZK0495.
- A send to a closed channel, and a second close, stop the program with the trap
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
for the first receive:

```zaka
ထုတ် ဟူသော လုပ်ဆောင်ချက် သည် လမ်း ဟူသော ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ မှ ၄ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လမ်း ထဲသို့ န ကို ပို့မည်။
"ပို့ {န}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
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
ပို့ ၁
ပို့ ၂
ဒေတာ(၁)
ရ ၂
ပို့ ၃
ပို့ ၄
ကျန် ၃
ကျန် ၄
```

```zaka-error ZK0496
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လမ်း သည် ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
လမ်း ထဲမှ ဒေတာ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## Parallel loops and maps

A loop marked `အပြိုင်` before its last word lets its rounds run at once:

```text
<from> မှ <to> အထိ <name> အနေဖြင့် အပြိုင် ရေတွက်၍ <stmt>
<list> ၏ အကွက်တိုင်း ကို <name> အနေဖြင့် အပြိုင် ယူ၍ <stmt>
<list> ၏ အကွက်တိုင်း ကို <number> နှင့် <name> အနေဖြင့် အပြိုင် ယူ၍ <stmt>
<from> မှ <to> အထိ <name> အနေဖြင့် အပြိုင်လုပ်ငန်း <n> ခုခွဲ၍ ရေတွက်၍ <stmt>
```

`အပြိုင်လုပ်ငန်း <n> ခုခွဲ၍` ("split into `<n>` jobs") stands in place of `အပြိုင်` before
the verb of each parallel loop (`ရေတွက်၍`, `တိုး၍`, `လျှော့၍`, `ယူ၍`), and before
`လုပ်ဆောင်သော ရလဒ်များ` in a parallel map. `<n>` is an integer (ZK0405), worked out as the loop
starts, and zero or below stops the program with the trap `threads not positive`. The rounds
then run on `<n>` threads, never more threads than rounds, each thread a chunk of
the rounds one after another; 1 runs them in order on the thread of the loop. With no count,
a loop takes one thread for each core.

Each has the block form too. The rounds follow the rules of a job, since the rounds run at
once on several threads:

- A round writes the variables it declares, and one cell of a list from outside: the cell
  of its own number, `<list> ၏ အကွက် <number>`. Each other write from a round is ZK0728,
  an argument that lets a call change a variable from outside too.
- Each variable from outside that a round reads holds a value jobs can use (ZK0494): no
  plain shared object, and no closure from outside the round. Each function a round
  reaches reads and writes no variable at the top of a file (ZK0726).
- A round cannot stop the loop with `ကွင်းဆက် ကို ရပ်မည်။`, nor return from the function
  around it (ZK0729); `နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်။` ends the round alone.

A native program runs the rounds on threads, one for each core of the host, when the rounds
are at least as numerous as the cores; fewer rounds run in order on the thread of the loop, and so
do the rounds on the VM. Each counted loop, with a step or not, and each loop over the
cells of a list, the characters of text or the rows of a ဇယား spreads its rounds; a loop
over the values of a channel runs them in order, since its values come one at a time. A
round over a removed row of a ဇယား ends at once. The lines
the rounds print come in no fixed order in a native program. A trap in a round acts as in a
job: the rounds of its thread end, the other threads finish, and the loop then stops with
`job trapped`; under `--trap abort` the program stops at once. Rounds that run in order
stop at the trap.

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

A parallel map is an expression: a new plain list of what a function gives back for each
cell, in the order of the cells.

```text
<list> ၏ အကွက်တိုင်း ကို <fn> ဟူသော လုပ်ဆောင်ချက် ဖြင့် အပြိုင် လုပ်ဆောင်သော ရလဒ်များ
<list> ၏ အကွက်တိုင်း ကို <fn> ဟူသော လုပ်ဆောင်ချက် ဖြင့် အပြိုင်လုပ်ငန်း <n> ခုခွဲ၍ လုပ်ဆောင်သော ရလဒ်များ
```

- The function takes one parameter that the cells fit, and follows the rules of the
  function a job starts in ([What a job can reach](#what-a-job-can-reach)): the cells and
  its result are of kinds jobs can use.
- A native program runs the rounds as those of a parallel loop, when the cells are of the
  kind of the parameter; the VM runs them in order. A trap in a round acts as in a parallel
  loop.

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

## Turns on the VM

The VM runs each job on the one thread of the VM, in turns of a fixed count of ops, in
the order the jobs started. The order of the turns never changes, so a program prints the
same lines each run. A wait for a job that has not ended, a receive from an open channel
that holds none and a send to a full one give the turn to the next job.
Each line a print writes comes out whole.

Each job gets a stack region of its own, of the size `--max-stack` gives
([Commands](../compiler/commands.md#run)).

## The end of the program

When the start block ends, the program waits for each job still running, then ends.

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

## A trap in a job

A trap ([When a program stops](../vm/traps.md)) cannot be caught. What it stops, `--trap`
chooses:

| `--trap` | A trap in a job |
| --- | --- |
| `unwind` (the default) | ends that job alone; the reason goes to standard error, and the rest of the program goes on |
| `abort` | stops the whole program |

- Under `unwind`, a wait for a job that stopped at a trap stops the waiter in turn, with
  the trap `job trapped`.
- A program whose job stopped at a trap that no wait saw ends with exit code 1.
- A trap in the start block stops the whole program under either choice.
- When each job waits, for another job or for a channel, and none can go on, the program
  stops with the trap `deadlock`: as a receive from a channel no job sends to or closes.
- A job that stops at a trap while it holds a lock leaves the lock broken ([Objects jobs
  share](#objects-jobs-share)).

## Where jobs run

- **The VM** runs each job in turns on its one thread, as above.
- **A native program**, from the C target or the native target
  ([Native programs](../compiler/native.md)), runs each job on a thread of its own, with
  a stack of 64 MiB. The threads run at once, so the order of their lines can change from
  run to run; what a job gives back, and what a channel carries, cannot. A wait for a
  job, a receive or a send that would wait while no other thread runs stops the program
  with the trap `deadlock`, as on the VM. `zakac build --trap abort` makes a trap in a
  job stop the whole program.
- **The JIT** (`zakac run`) runs jobs, parallel loops and maps on threads of its own, as a
  native program does.
- While a job runs, a native program collects no rings: the collector changes counts
  with plain ops, so it waits until each job has ended. The buffer still fills, and the
  end of the program collects, as ever.

## Threads of C

C starts threads of its own too, and two of the rules above reach them:

- A native program hands C a function, and C calls it on a thread it started: the call
  runs when the function, and each function it calls, reads and writes no variable at the
  top of a file, the rule of ZK0726, and when each variable it captures is one no
  statement writes, of a kind jobs can use other than an object. Each other call from
  such a thread ends the run with a message
  ([Closures C calls](function-values.md#nested-functions-and-closures)).
- C calls a [lib built from Zaka](functions.md#functions-c-can-call) from several
  threads: a function of the lib that reaches no such variable, calls no C and no function
  value, and starts no job runs on the threads at once, with no lock.
