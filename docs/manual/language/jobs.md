---
title: Jobs
description: Work that runs side to side with the rest of the program: starting a job, the job kind, waiting for its result, the values that can go to a job, what a job can reach, channels between jobs, how the VM takes turns, the end of the program, and what a trap in a job does.
---

# Jobs

A job (`အပြိုင်လုပ်ငန်း`) runs a function side to side with the rest of the program. The
program starts it with a call that ends in `အပြိုင်`, goes on with its own work, and waits
for the result when it needs it. Each job reaches the same heap: a value that goes to a job
is not copied until one side writes it.

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

## Where jobs run

- **The VM** runs each job in turns on its one thread, as above.
- **A native program**, from the C target or the native target
  ([Native programs](../compiler/native.md)), runs each job on a thread of its own, with
  a stack of 64 MiB. The threads run at once, so the order of their lines can change from
  run to run; what a job gives back, and what a channel carries, cannot. A wait for a
  job, a receive or a send that would wait while no other thread runs stops the program
  with the trap `deadlock`, as on the VM. `zakac build --trap abort` makes a trap in a
  job stop the whole program.
- **The JIT** cannot run a job: `zakac run` gives a program with a job to the VM, and
  `zakac run --jit` reports it.
- While a job runs, a native program collects no rings: the collector changes counts
  with plain ops, so it waits until each job has ended. The buffer still fills, and the
  end of the program collects, as ever.
