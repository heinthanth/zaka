---
title: Closures
description: Declare a function inside another function, use the variables around it, return it from its block, and make a fresh one in each round of a loop.
---

# Closures

A closure is a function that we declare inside another function, or inside the program
start block, and that uses the variables around it. It shares those variables with the
block around it, so each side sees the writes of the other, much as a nested function in
JavaScript or Python does.

## You will learn

- How a function inside a block uses the variables around it.
- How a closure can leave its block and keep its variables alive.
- How closures made in a loop each see their own round.

## A function inside a block

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် တိုး သည် ၁၀ ဖြစ်ပါစေ။
// a closure: it reads တိုး, a variable around it
ထည့် ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က + တိုး ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ထည့်(၁) ကို ပြသမည်။
တိုး ကို ၅၀ ဟု ထားမည်။
ထည့်(၁) ကို ပြသမည်။                // the closure sees the new value
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၁
၅၁
```

We declare a closure in the same form as other functions, just inside a block, and it can
use each variable that the block declares above it. We do not list those variables or
mark them:

```
<name> ဟူသော လုပ်ဆောင်ချက် သည် … အောက်ပါတို့ကို လုပ်ဆောင်ကာ … ကို ပြန်ပေးမည်။
<sentences that use the variables around it>
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

The closure does not get a duplicate (copy) of `တိုး`. It uses the variable itself, so
after the program start block sets `တိုး` to `၅၀`, the second call sees the new value.

## A closure that keeps a count

A closure can leave the block that made it, as a result or in a variable, and the
variables that it uses remain alive with it. Here a function returns a counter:

```zaka
ရေတွက်စက် ဟူသော လုပ်ဆောင်ချက် သည် စ ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ (လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် ကြိမ် သည် စ ဖြစ်ပါစေ။
တိုး ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကြိမ် ကို ကြိမ် + ၁ ဟု ထားမည်။
ကြိမ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
တိုး ဟူသော လုပ်ဆောင်ချက် ကို ပြန်ပေးမည်။        // the closure leaves: fine
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ရေတွက် သည် ရေတွက်စက် ဟူသော လုပ်ဆောင်ချက်ကို ၁၀ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် အခြား သည် ရေတွက် ဖြစ်ပါစေ။                 // the same counter
ရေတွက် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
အခြား ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
ကိန်းရှင် သစ် သည် ရေတွက်စက် ဟူသော လုပ်ဆောင်ချက်ကို ၀ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
သစ် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။                   // a new, separate counter
```

```text
၁၁
၁၂
၁
```

Each call of `ရေတွက်စက်` makes a new `ကြိမ်`, so `သစ်` counts apart from `ရေတွက်`. A
duplicate of a function value is the same closure, just as a duplicate of a [shared
object](../structs/objects.md) (reference type) is the same object, so `အခြား` goes on
from where `ရေတွက်` stopped.

## Closures made in a loop

Each round of a loop gets a fresh loop variable, as `let` does in JavaScript, so the
closures that we make in a loop each see their own round:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပြများ သည် (လုပ်ဆောင်ချက် အမျိုးအစား) စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [] ဖြစ်ပါစေ။
၁ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပြ ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပြများ ထဲသို့ ပြ ဟူသော လုပ်ဆောင်ချက် ကို ထည့်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပြများ ၏ အကွက်တိုင်း ကို ဖ အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဖ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁
၂
၃
```

A variable that we declare inside the loop is fresh in each round too.

## Details

- **Nothing to free.** A closure and the variables that it keeps are freed once nothing
  reaches them. A closure can even sit in a variable that it uses, and the ring collector
  of [shared objects](../structs/objects.md#objects-that-point-at-each-other) frees the
  pair.
- **Threads.** A [thread](../threads/README.md) cannot start in a closure (ZK0497), but the
  function that it starts in can call one ([Closures](../../manual/language/closures.md)).
- **C.** A closure can also be passed to a C function, which calls it while the call runs
  ([C functions and addresses](../pointers/c.md)).

## Recap

- A function that we declare inside a block is a closure, and it can use the variables
  that the block declares above it.
- A closure shares those variables with its block, so each side sees the writes of the
  other.
- A closure can leave its block, and it keeps its variables alive.
- Each round of a loop gets fresh variables, so each closure made in a round sees its own
  round.

## Hands on

- Write `ပေါင်းစက်`, which accepts a number `န` and returns a function that adds `န` to its
  parameter. Make one for 2 and one for 10, and call each.
- Change the counter so that its function also returns the count as its result.
- Make three closures in a loop over the cells of a list of texts, and let each one print
  its own cell.

Next: [Generics](../generics/README.md).
