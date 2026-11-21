---
title: Parallel work
description: Split the same work on each cell of a list among several threads, with a parallel map that returns a new list and a parallel loop whose rounds each write their own cell.
---

# Parallel work

When each cell of a list needs the same work, we can split the cells among several
threads, much as several cooks can each peel a share of the potatoes. Zaka has two forms
for this: a parallel map, which returns a new list, and a parallel loop (parallel `for`).

## You will learn

- How to run a function on each cell of a list on several threads.
- How to mark a loop so that its rounds can run at the same time.
- What a round of a parallel loop can and cannot write.
- How to choose the count of threads.

## A parallel map

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

A parallel map calls a function on each cell (element) of a list and returns a new list of
the results, in the order of the cells. In a native program, the calls run on several
threads, one for each core of the computer, while the VM runs them in order:

```
<list> ၏ အကွက်တိုင်း ကို <function> ဟူသော လုပ်ဆောင်ချက် ဖြင့် အပြိုင် လုပ်ဆောင်သော ရလဒ်များ
```

| Zaka | Reads as |
| --- | --- |
| `ဂဏန်းများ ၏ အကွက်တိုင်း ကို` | "each cell of `ဂဏန်းများ`" |
| `နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက် ဖြင့်` | "with the function called `နှစ်ထပ်`" |
| `အပြိုင် လုပ်ဆောင်သော ရလဒ်များ` | "the results of running it in parallel" |

The function follows the same rules as the function of a [thread](README.md): the program
declares it, it has one parameter that fits the cells, and it does not touch a variable at
the top of a file.

## A parallel loop

To let the rounds of a loop run at the same time, we write `အပြိုင်` before the last
word of the loop, `ရေတွက်၍` or `ယူ၍`:

```
<from> မှ <to> အထိ <name> အနေဖြင့် အပြိုင် ရေတွက်၍ <sentence>
<list> ၏ အကွက်တိုင်း ကို <name> အနေဖြင့် အပြိုင် ယူ၍ <sentence>
```

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

A native program runs the rounds on one thread for each core, once the count of rounds is
at least the count of cores. With fewer rounds, and on the VM, the rounds run in order.
The block form works too, with `အောက်ပါတို့ကို လုပ်ဆောင်မည်။` after `ရေတွက်၍` and a block
closer at the end.

## What a round can write

The rounds of a parallel loop run at the same time, so two rounds must never write the
same place. For this reason, each round can write just the variables that it declares and
the cell with the number of the round, `<list> ၏ အကွက် <number>`. Another write, such as
adding to a running sum, is the error ZK0728:

```zaka-error ZK0728
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စု သည် ၀ ဖြစ်ပါစေ။
၀ မှ ၄ အထိ i အနေဖြင့် အပြိုင် ရေတွက်၍ စု ကို စု + i ဟု ထားမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0728]: a round of a parallel loop writes "စု", a variable from outside the loop, and each round writes the cell of its own number alone
```

To keep a running sum, we use an [atomic number](sharing.md#atomic-numbers) in its place.
Each round adds to it with `ပေါင်းထည့်မည်`, in one step that no other round can split:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စု သည် အက်တမ်မစ် ဖြစ်သော ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
၁ မှ ၁၀၀ အထိ i အနေဖြင့် အပြိုင် ရေတွက်၍ စု ထဲသို့ i ကို ပေါင်းထည့်မည်။
စု ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၅၀၅၀
```

A round also cannot stop the whole loop with `ကွင်းဆက် ကို ရပ်မည်။` or return from its
function, because the other rounds are running at the same time (ZK0729). To end just its
own round, a round can use `နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်။`:

```zaka-error ZK0729
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ထွက် သည် အကွက် ၅ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
၀ မှ ၄ အထိ i အနေဖြင့် အပြိုင် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ထွက် ၏ အကွက် i ကို i ဟု ထားမည်။
ကွင်းဆက် ကို ရပ်မည်။          // the other rounds are running: ZK0729
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## Choosing the count of threads

To choose the count of threads that the work uses, we write `အပြိုင်လုပ်ငန်း <count> ခုခွဲ၍` ("split
into `<count>` threads") in place of `အပြိုင်`:

```
<from> မှ <to> အထိ <name> အနေဖြင့် အပြိုင်လုပ်ငန်း <count> ခုခွဲ၍ ရေတွက်၍ <sentence>
<list> ၏ အကွက်တိုင်း ကို <function> ဟူသော လုပ်ဆောင်ချက် ဖြင့် အပြိုင်လုပ်ငန်း <count> ခုခွဲ၍ လုပ်ဆောင်သော ရလဒ်များ
```

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ထွက် သည် အကွက် ၈ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
၀ မှ ၇ အထိ i အနေဖြင့် အပြိုင်လုပ်ငန်း ၂ ခုခွဲ၍ ရေတွက်၍ ထွက် ၏ အကွက် i ကို i * i ဟု ထားမည်။
ထွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၀၊ ၁၊ ၄၊ ၉၊ ၁၆၊ ၂၅၊ ၃၆၊ ၄၉]
```

Here a native program splits the eight rounds between two threads. Each thread takes the
next part of the rounds that no other thread took, until none are left, so a thread whose
rounds end fast takes more of them. A count of 1 runs the rounds in order.

## Details

- **What a round can read.** A round can read numbers, text and lists from outside the
  loop, but not a plain shared object (ZK0494), and each function it calls cannot touch a
  variable at the top of a file (ZK0726) ([Threads](../../manual/language/threads.md)).
- **Printed lines.** In a native program, the lines that the rounds print come in no fixed
  order.
- **A trap in a round.** A trap in a round ends the rounds of its thread. The loop stops
  once the other threads finish, just as a wait for a thread that stopped at a trap does.
- **The count.** The count is an integer (ZK0405), worked out as the loop starts. A count
  of zero or below stops the program with the trap `threads not positive`, and a loop never
  uses more threads than it has rounds.
- **Other loops.** A loop that counts down or uses a step can also be parallel, and so can a
  loop over the characters of text or the rows of a table (map). A loop over the values of
  a [channel](channels.md) runs its rounds in order, because the values come one at a time.

## Recap

- A parallel map, `<list> ၏ အကွက်တိုင်း ကို <function> … ဖြင့် အပြိုင် လုပ်ဆောင်သော ရလဒ်များ`,
  returns a new list of the results in the order of the cells.
- `အပြိုင်` before `ရေတွက်၍` or `ယူ၍` lets the rounds of a loop run at the same time.
- Each round can write the variables it declares and the cell with the number of the
  round, and it can add to an atomic number from outside the loop.
- `အပြိုင်လုပ်ငန်း <count> ခုခွဲ၍` chooses the count of threads.
- The VM runs the rounds in order, while a native program spreads them over the cores.

## Hands on

- Write a parallel map that turns a list of texts into a list of their lengths.
- Fill a list of 100 cells with a parallel loop, so that cell `i` has `i * i`.
- Add a running sum to that loop and read the error. Then work out the sum after the loop
  instead.

Next: [Input and output](../io/README.md).
