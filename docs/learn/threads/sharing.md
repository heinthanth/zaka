---
title: Sharing between threads
description: Let several threads change one value: an object that threads share and the lock block that guards it, atomic numbers that threads add to without a lock, and a compare-and-set.
---

# Sharing between threads

A thread gets a duplicate (copy) of each value that we pass to it, so two threads never
write to the same value. When threads must change one thing together, Zaka has two tools
for it: a shared object (reference type) that a lock guards (a mutex), and an atomic
number (atomic integer) that threads can change without a lock.

## You will learn

- How to declare a shared object that threads can use.
- How to read and write its fields inside a lock block.
- How threads add to an atomic number without a lock.
- How to set an atomic number in one step with a compare-and-set.

## An object that threads share

Here two threads each add 100 to the balance of one account:

```zaka
အကောင့် ဟူသော အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ထည့်ငွေ ဟူသော လုပ်ဆောင်ချက် သည် ဘဏ် ဟူသော အကောင့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ မှ ၁၀၀ အထိ န အနေဖြင့် ရေတွက်၍ ဘဏ် ကို ခဏယူ၍ ဘဏ် ၏ လက်ကျန် ကို (ဘဏ် ၏ လက်ကျန်) + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ဘဏ် သည် အကောင့်(လက်ကျန်: ၀) ဖြစ်ပါစေ။
ကိန်းရှင် က သည် ထည့်ငွေ ဟူသော လုပ်ဆောင်ချက်ကို ဘဏ် ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ထည့်ငွေ ဟူသော လုပ်ဆောင်ချက်ကို ဘဏ် ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
က ကို စောင့်မည်။
ခ ကို စောင့်မည်။
ဘဏ် ကို ခေတ္တယူ၍ ဘဏ် ၏ လက်ကျန် ကို ပြသမည်။
```

```text
၂၀၀
```

To let threads share the objects of a struct, we add `အပြိုင်လုပ်ငန်းများတွင်
အသုံးပြုနိုင်သော` before `မျှဝေနိုင်သော` in its declaration. Such an object goes to a
thread as the object itself, not as a duplicate, so both threads above change the same
balance. To read or write a field, we take the lock of the object in a lock block:

```
<name> ဟူသော အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် <fields> ပါဝင်သည်။
<object> ကို ခေတ္တယူ၍ <sentence>
```

| Zaka | Reads as |
| --- | --- |
| `အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော` | "that threads can use" |
| `မျှဝေနိုင်သော ဒေတာအစုအဝေး` | "a struct that can be shared": a shared object |
| `ဘဏ် ကို ခေတ္တယူ၍` | "taking `ဘဏ်` for a while" |

Just one thread at a time can be inside a lock block of an object, much as just one cook
at a time can use the knife. A thread that reaches the lock block while another thread is
inside it waits until that block ends. So the read and the write of `လက်ကျန်` in each
round happen together, and no change gets lost. `ခဏယူ၍` means the same as `ခေတ္တယူ၍`.

A lock block can also contain several sentences:

```
<object> ကို ခေတ္တယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
<sentences>
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

When we read a field of such an object outside a lock block, the compiler stops with the
error ZK0499:

```zaka-error ZK0499
အကောင့် ဟူသော အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ကိန်းရှင် ဘဏ် သည် အကောင့်(လက်ကျန်: ၀) ဖြစ်ပါစေ။
ဘဏ် ၏ လက်ကျန် ကို ပြသမည်။
```

## Atomic numbers

When threads share just one number, a lock is more than the work needs. An atomic number
(atomic integer), `အက်တမ်မစ် ဖြစ်သော ကိန်းပြည့်`, is an integer that threads can add to
and subtract from without a lock, because each change happens in one step that no other
thread can split:

```
<field> ဟူသော အက်တမ်မစ် ဖြစ်သော ကိန်းပြည့် တစ်ခု
<atomic> ထဲသို့ <value> ကို ပေါင်းထည့်မည်။
<atomic> ထဲမှ <value> ကို နုတ်ယူမည်။
```

```zaka
ကောင်တာ ဟူသော အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် ကြိမ် ဟူသော အက်တမ်မစ် ဖြစ်သော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ရေ ဟူသော လုပ်ဆောင်ချက် သည် ရေတွက် ဟူသော ကောင်တာ တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ မှ ၁၀၀ အထိ န အနေဖြင့် ရေတွက်၍ ရေတွက် ၏ ကြိမ် ထဲသို့ ၁ ကို ပေါင်းထည့်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ရေတွက် သည် ကောင်တာ(ကြိမ်: ၀) ဖြစ်ပါစေ။
ကိန်းရှင် က သည် ရေ ဟူသော လုပ်ဆောင်ချက်ကို ရေတွက် ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ရေ ဟူသော လုပ်ဆောင်ချက်ကို ရေတွက် ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
က ကို စောင့်မည်။
ခ ကို စောင့်မည်။
ရေတွက် ၏ ကြိမ် ထဲမှ ၁ ကို နုတ်ယူမည်။
ရေတွက် ၏ ကြိမ် ကို ပြသမည်။
```

```text
၁၉၉
```

| Zaka | Reads as |
| --- | --- |
| `အက်တမ်မစ် ဖြစ်သော ကိန်းပြည့်` | "an integer that is atomic" |
| `ကြိမ် ထဲသို့ ၁ ကို ပေါင်းထည့်မည်` | "add 1 into `ကြိမ်`" |
| `ကြိမ် ထဲမှ ၁ ကို နုတ်ယူမည်` | "take 1 out of `ကြိမ်`" |

We can read and set an atomic field outside a lock block, as the last lines show. But we
cannot set it from its own value, as in `x ကို x + ၁ ဟု ထားမည်`, because that is a read
followed with a write, and another thread could change the number between the two. The
compiler stops with the error ZK0727 there, so we use `ပေါင်းထည့်မည်` instead.

## Compare and set

To set an atomic number just when it still has the value we expect, in one step, we use a
compare-and-set:

```
<atomic> ထဲရှိ <expected> ကို <new> ဖြင့် အစားထိုးသော ရလဒ်
```

It returns a [result](../error-handling/README.md) (result type). When the number had
`<expected>`, it now has `<new>`, and the result is a good result with the old value.
When another thread changed the number first, the number does not change, and the result
is an error value with the value that the number has. Here three threads each want to
claim one box, and just one of them wins:

```zaka
ကောင်တာ ဟူသော အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် ပိုင်ရှင် ဟူသော အက်တမ်မစ် ဖြစ်သော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ယူ ဟူသော လုပ်ဆောင်ချက် သည် ဘူး ဟူသော ကောင်တာ တစ်ခု ကို လည်းကောင်း န ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် ရ သည် ဘူး ၏ ပိုင်ရှင် ထဲရှိ ၀ ကို န ဖြင့် အစားထိုးသော ရလဒ် ဖြစ်ပါစေ။
အကယ်၍ ရ ထဲတွင် အောင်မြင်သည့်သတင်း ရှိလျှင် ၁ ကို ပြန်ပေးမည်။
၀ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ဘူး သည် ကောင်တာ(ပိုင်ရှင်: ၀) ဖြစ်ပါစေ။
ကိန်းရှင် က သည် ယူ ဟူသော လုပ်ဆောင်ချက်ကို ဘူး နှင့် ၁ တို့ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ယူ ဟူသော လုပ်ဆောင်ချက်ကို ဘူး နှင့် ၂ တို့ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် ယူ ဟူသော လုပ်ဆောင်ချက်ကို ဘူး နှင့် ၃ တို့ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
(က ကို စောင့်သော ရလဒ်) + (ခ ကို စောင့်သော ရလဒ်) + (ဂ ကို စောင့်သော ရလဒ်) ကို ပြသမည်။
```

```text
၁
```

Each thread wants to change the owner from 0 to its own number. The first thread to get
there wins and returns 1, while the other two see that the owner is no longer 0 and
return 0, so the sum is 1 whichever thread wins.

## Details

- **Broken locks.** When a thread stops at a trap while it is inside a lock block, the lock
  is left broken, and the next lock of that object stops its thread with the trap
  `lock broken` ([Threads](../../manual/language/threads.md)).
- **Deadlock.** Two threads that each wait for a lock that the other one has stop with the
  trap `deadlock`.
- **Taking a lock again.** A thread that is inside a lock block can take the same lock
  again, for example in a function that it calls from the block. Each exit from the block
  returns the lock: its end, `ပြန်ပေးမည်`, a break or a next round.
- **Atomic data kinds.** An atomic number is a `ကိန်းပြည့်` or a `လက္ခဏာမဲ့ ကိန်းပြည့်` of
  the platform width; another data kind is the error ZK0401. An add or a subtraction wraps
  around past the range of the data kind, without a stop.
- **The result of a compare-and-set.** We can use `ထဲမှ ဒေတာ` and `ထဲမှ အမှားအယွင်း ဒေတာ`
  to read the two sides ([Reading a whole result](../error-handling/results.md)). As a
  sentence on its own, its result would go unread, which is the error ZK0443.

## Recap

- We add `အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော` to a shared struct to let threads share
  its objects.
- We read and write the fields of such an object inside `<object> ကို ခေတ္တယူ၍ …`, and just
  one thread at a time can be inside it.
- An atomic number changes in one step with `ပေါင်းထည့်မည်` and `နုတ်ယူမည်`, without a lock.
- A compare-and-set sets an atomic number just when it has the expected value, and returns
  a result.

## Hands on

- Change the first example so that four threads each add 50, and check the balance.
- Remove `ဘဏ် ကို ခဏယူ၍` from `ထည့်ငွေ` and read the error.
- Write a function that subtracts from an atomic number, and run it in two threads at once.

Next: [Parallel work](parallel.md).
