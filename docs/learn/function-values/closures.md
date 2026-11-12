---
title: Closures
description: A function inside another uses the variables around it and shares them; it can leave its block, and each loop round gives it a fresh variable.
---

# Closures

A function declared inside another function, or inside the start block, uses the
variables above it with no list and no mark, as in JavaScript or Python. Such a function
is a *closure*: it shares those variables with the block around it, so the block sees each
write the closure makes, and the closure sees each write of the block.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် တိုး သည် ၁၀ ဖြစ်ပါစေ။
// a closure: it reads တိုး, a variable around it
လုပ်ဆောင်ချက် ထည့် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
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

## A closure that keeps a count

A closure can leave the block that made it, as a result or in a variable, and it keeps
the variables it uses alive. Here a function gives back a counter:

```zaka
လုပ်ဆောင်ချက် ရေတွက်စက် သည် စ ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ (လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် ကြိမ် သည် စ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် တိုး သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကြိမ် ကို ကြိမ် + ၁ ဟု ထားမည်။
ကြိမ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
တိုး ဟူသော လုပ်ဆောင်ချက် ကို ပြန်ပေးမည်။        // the closure leaves: fine
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရေတွက် သည် ရေတွက်စက် ဟူသော လုပ်ဆောင်ချက်ကို ၁၀ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် အခြား သည် ရေတွက် ဖြစ်ပါစေ။                 // the same counter
ရေတွက် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
အခြား ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
ကိန်းရှင် သစ် သည် ရေတွက်စက် ဟူသော လုပ်ဆောင်ချက်ကို ၀ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
သစ် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။                   // a new counter of its own
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၁
၁၂
၁
```

Each call of `ရေတွက်စက်` makes a new `ကြိမ်`, so `သစ်` counts on its own. A duplicate of a function
value is the same closure, as a duplicate of a [shared object](../structs/objects.md) is the
same object: `အခြား` goes on from where `ရေတွက်` stopped.

## Closures made in a loop

Each round of a loop gets a fresh loop variable, as `let` gives in JavaScript, so closures
made in a loop each see their own round:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပြများ သည် (လုပ်ဆောင်ချက် အမျိုးအစား) စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [] ဖြစ်ပါစေ။
၁ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ပြ သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
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

A variable declared inside the loop is fresh in each round too.

## Nothing to free

A closure and the variables it keeps are freed once nothing reaches them. A closure can
even sit in a variable it uses, and the ring collector of [shared
objects](../structs/objects.md#objects-that-point-at-each-other) frees the pair.

## Hands on

- Write `ပေါင်းစက်`, which takes a number `န` and gives back a function that adds `န` to its
  parameter. Make one for 2 and one for 10, and call each.
- Change the counter so that its function also gives back the count as its result.
- Make three closures in a loop over the cells of a text list, each printing its own cell.

Next: [Modules](../modules/README.md).
