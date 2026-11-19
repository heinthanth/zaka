---
title: Jobs
description: Work that runs side to side with the rest of the program: starting a job, the job kind, waiting for its result, the values that can go to a job, how the VM takes turns, the end of the program, and what a trap in a job does.
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

## Turns on the VM

The VM runs each job on the one thread of the VM, in turns of a fixed count of ops, in
the order the jobs started. The order of the turns never changes, so a program prints the
same lines each run. A wait for a job that has not ended gives the turn to the next job.
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
- When each job waits for another and none can go on, the program stops with the trap
  `deadlock`.

## Where jobs run

Jobs run on the VM, which `zakac run` picks for a program with a job. The C and native
targets refuse a program with a job for now.
