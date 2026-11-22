---
title: Threads
description: Run a function while the rest of the program goes on: start a thread, wait for its result, start a thread that returns nothing, pass values to it, and let a thread sleep.
---

# Threads

A thread (`အပြိုင်လုပ်ငန်း`) lets a function run while the rest of the program
continues. Start a thread when two pieces of work can proceed on their own, then wait
for its result when the program needs it.

## You will learn

- How to start a function as a thread and wait for its result.
- How threads run on the VM and in a native program.
- How to start a thread that returns nothing.
- Which values we can pass to a thread, and what happens when a thread stops.
- How to let a thread sleep for a while.

## Starting a thread

Here the program starts two threads, and each one adds up a list:

```zaka
စုပေါင်း ဟူသော လုပ်ဆောင်ချက် သည်
ဂဏန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် စု သည် ၀ ဖြစ်ပါစေ။
ဂဏန်းများ ၏ အကွက်တိုင်း ကို ဂ အနေဖြင့် ယူ၍ စု ကို စု + ဂ ဟု ထားမည်။
စု ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// start two threads, then go on at once
ကိန်းရှင် ရှေ့ သည် စုပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို [၁၊ ၂၊ ၃] ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ကိန်းရှင် နောက် သည် စုပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို [၄၊ ၅၊ ၆] ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
"နှစ်ခု စတင်ပြီ" ဟု ပြသမည်။
// wait for both threads and add their results
(ရှေ့ ကို စောင့်သော ရလဒ်) + (နောက် ကို စောင့်သော ရလဒ်) ကို ပြသမည်။
```

```text
နှစ်ခု စတင်ပြီ
၂၁
```

To start a thread, we write a long call of the function and end it with
`အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း` in place of `လုပ်ဆောင်သော ရလဒ်`. To get the result, we
wait for the thread with `ကို စောင့်သော ရလဒ်`:

```
<function> ဟူသော လုပ်ဆောင်ချက်ကို <arguments> ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း
<thread> ကို စောင့်သော ရလဒ်
```

| Zaka | Reads as |
| --- | --- |
| `စုပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို` | "the function called `စုပေါင်း`" |
| `[၁၊ ၂၊ ၃] ဖြင့်` | "with `[1, 2, 3]`" |
| `အပြိုင်` | "side to side", "in parallel" |
| `လုပ်ဆောင်သော လုပ်ငန်း` | "the work that runs it": the thread |
| `ရှေ့ ကို စောင့်သော ရလဒ်` | "the result of waiting for `ရှေ့`" |

The variable `ရှေ့` contains the thread, not the sum. Its data kind (type) is
`ကိန်းပြည့် ပြန်ပေးမည့် အပြိုင်လုပ်ငန်း`, "a thread that returns an integer". The program
prints its first line while the threads still work, and each wait returns the sum that
its thread made: `၆` and `၁၅`.

## The VM and a native program

A thread runs side to side with the rest of the program, but what that means depends on
where the program runs:

- **The VM** (`zakac run`) runs all the threads on one OS thread and takes turns between
  them: it runs one thread for a short while, then the next one, in the same fixed
  order each time. So a program prints the same lines in the same order on each run.
- **A native program** (`zakac build --backend native` or `--backend c`, or
  `zakac run --jit`) runs each thread as a light task, a small stack that a few OS
  threads take turns on, or on a separate OS thread when the thread needs one. The
  threads then run at the same time on several cores of the computer, so the work can
  finish sooner, but the lines that two threads print can come in a different order from
  one run to the next.

A thread returns the same result on both, so a program should depend on results and
channels, not on the order of the lines that two threads print.

## A thread that returns nothing

When we do not need a result, we end the call with `အပြိုင် လုပ်ဆောင်မည်။` and keep no
name for the thread. The program waits for each such thread before it ends:

```
<function> ဟူသော လုပ်ဆောင်ချက်ကို <arguments> ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
```

```zaka
ပြ ဟူသော လုပ်ဆောင်ချက် သည် စာ ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စာ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပြ ဟူသော လုပ်ဆောင်ချက်ကို "နောက်ကွယ်မှ" ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
"ရှေ့မှ" ဟု ပြသမည်။
```

```text
ရှေ့မှ
နောက်ကွယ်မှ
```

On the VM, the program start block keeps its turn, so its line comes first. To wait for
a thread that we kept in a variable and drop its result, we write
`<thread> ကို စောင့်မည်။`.

## Waiting once

A wait moves the result of the thread to the waiter, so we can wait for each thread just
once. To use the result in several places, we keep it in a variable:

```zaka
အော် ဟူသော လုပ်ဆောင်ချက် သည်
စာ ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"{စာ}!" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် က သည် အော် ဟူသော လုပ်ဆောင်ချက်ကို "မင်္ဂလာ" ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ကိန်းရှင် ရ သည် က ကို စောင့်သော ရလဒ် ဖြစ်ပါစေ။
ရ ကို ပြသမည်။
ရ ကို ပြသမည်။
```

```text
မင်္ဂလာ!
မင်္ဂလာ!
```

A second wait for the same thread stops the program with the trap `thread waited`.

## What a thread can use

A thread gets a duplicate (copy) of each argument, so a write on one side never reaches
the other. Numbers, text, lists, plain structs and results can all go to a thread. Three
rules make sure that two threads never change one value at the same time:

- We cannot pass a plain shared object to a thread (ZK0494), because both threads could
  write to it. [Sharing between threads](sharing.md) shows the objects that we can pass.
- A thread starts in a function that the program declares, not in a function value or a
  closure (ZK0497), and that function has no mutable parameter (`inout`) or writable
  parameter (`out`) (ZK0498).
- A thread cannot read or write a variable at the top of a file (ZK0726), because that
  variable belongs to the main thread. We pass the value as an argument instead.

```zaka-error ZK0726
ကိန်းရှင် ကြိမ် သည် ၀ ဖြစ်ပါစေ။

တိုး ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကြိမ် ကို ကြိမ် + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

တိုး ဟူသော လုပ်ဆောင်ချက်ကို အပြိုင် လုပ်ဆောင်မည်။
```

A thread returns what it makes through its result, or through a
[channel](channels.md).

## When a thread stops

When a thread stops at a [trap](../../manual/vm/traps.md) (runtime error), just that
thread ends: the reason goes to standard error, and the rest of the program goes on. A
wait for that thread then stops the waiter too:

```zaka-stop
စား ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
၁၀ / န ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် က သည် စား ဟူသော လုပ်ဆောင်ချက်ကို ၀ ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
"ဆက်လုပ်" ဟု ပြသမည်။
က ကို စောင့်သော ရလဒ် ကို ပြသမည်။
```

```text
ဆက်လုပ်
the program stopped: the thread 1 this waits for stopped at a trap
```

The thread divides with zero and stops, while the program start block goes on and prints
its line. Then the wait for `က` stops the program, and the program ends with exit code 1.
A program also ends with exit code 1 when one of its threads stopped at a trap and no wait
saw it.

## Sleeping

A thread can sleep for a while, and the other threads go on while it sleeps. We write the
count of milliseconds or seconds before `မီလီစက္ကန့်` or `စက္ကန့်`, then `စောင့်မည်။`:

```
<count> မီလီစက္ကန့် စောင့်မည်။
<count> စက္ကန့် စောင့်မည်။
```

```zaka
အိပ် ဟူသော လုပ်ဆောင်ချက် သည်
အမည် ဟူသော စာသား တစ်ခု ကို လည်းကောင်း
ကြာ ဟူသော လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကြာ မီလီစက္ကန့် စောင့်မည်။
အမည် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် က သည် အိပ် ဟူသော လုပ်ဆောင်ချက်ကို "နှေး" နှင့် ၃၀၀ တို့ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အိပ် ဟူသော လုပ်ဆောင်ချက်ကို "မြန်" နှင့် ၂၀ တို့ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
"စ" ဟု ပြသမည်။
က ကို စောင့်မည်။
ခ ကို စောင့်မည်။
```

```text
စ
မြန်
နှေး
```

Word for word, `ကြာ မီလီစက္ကန့် စောင့်မည်` is "for `ကြာ` milliseconds, will wait". The
thread with the shorter sleep prints first, and the two sleeps run at the same time, so
the program ends after about 300 milliseconds and not after the sum of the two. The
program start block can sleep as well. On this page, the examples run with a clock of
their own, so a sleep ends at once and the lines still come in the same order.

## Details

- **The data kind of a thread.** A thread that returns nothing has the data kind
  `အပြိုင်လုပ်ငန်း`. A duplicate of a thread value names the same thread
  ([Threads](../../manual/language/threads.md)).
- **The words.** `အပြိုင်` and `လုပ်ငန်း` are not reserved words, and `စောင့်` is a verb
  of the core module, so a program can declare a function of the same name
  ([Threads](../../manual/language/threads.md)).
- **Stopping the whole program.** We can build a native program with `--trap abort`, and a
  trap in one thread then stops the whole program
  ([Commands](../../manual/compiler/commands.md)).
- **Light tasks.** Tens of thousands of threads fit in a native program, as light tasks.
  The compiler gives a thread an OS thread of its own when it calls C, reads standard
  input, calls a function again or keeps large locals, and we can ask for one with `သီးသန့် အပြိုင်` in place
  of `အပြိုင်` ([Light tasks](../../manual/language/threads.md#light-tasks)).
- **Shuffled turns.** `zakac run --shuffle-thread-order` makes the VM take turns in a
  shuffled order, which helps to find a mistake that hangs on the order of the threads
  ([Commands](../../manual/compiler/commands.md#run)).
- **Deadlock.** When each thread waits and none can go on, the program stops with the trap
  `deadlock` ([When a program stops](../../manual/vm/traps.md)). A sleeping thread is not
  waiting for another thread, so a program with a sleep is no deadlock
  ([Sleeping](../../manual/language/threads.md#sleeping)).

## Recap

- We end a long call with `အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း` to start a thread, and we use
  `<thread> ကို စောင့်သော ရလဒ်` to wait for its result.
- The VM takes turns between threads on one OS thread, while a native program runs each
  thread as a light task on a few OS threads, or on a separate OS thread when it needs one.
- `… အပြိုင် လုပ်ဆောင်မည်။` starts a thread that we do not wait for, and the program waits
  for it before it ends.
- A thread gets a duplicate of each argument, and it cannot touch a variable at the top of
  a file or a plain shared object.
- A trap in a thread ends that thread, and a wait for it stops the waiter.
- `<count> မီလီစက္ကန့် စောင့်မည်။` and `<count> စက္ကန့် စောင့်မည်။` let a thread sleep while
  the other threads go on.

## Hands on

- Start three threads that each add up a different list, and print the sum of the three
  results.
- Wait for the same thread twice and read the message.
- Build the first example with `zakac build --backend native` and run it a few times. Does
  the output change?
- Start three threads that sleep for 3, 1 and 2 seconds and then print their names. In
  which order do the names come, and how long does the program take?

Next: [Channels](channels.md).
