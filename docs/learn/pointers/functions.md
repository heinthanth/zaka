---
title: Functions as values
description: Keep a function in a variable, pass it to another function, and pick one from a table.
---

# Functions as values

A function can be handed around like a number. As `x ၏ လိပ်စာ` is the address of a
variable, `လုပ်ဆောင်ချက် <name> ၏ လိပ်စာ` is the address of a function: a value to keep in a
variable, and a call through the variable runs the function. No block of storage stands
behind it, so there is nothing to free.

```zaka
လုပ်ဆောင်ချက် နှစ်ထပ် သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က * က ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
// the address of the function, kept in a variable
ကိန်းရှင် ဖ သည် လုပ်ဆောင်ချက် နှစ်ထပ် ၏ လိပ်စာ ဖြစ်ပါစေ။
ဖ(၇) ကို ပြသပါ။
လုပ်ဆောင်ချက် ဖ ကို ၅ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ဖ ကို ပြသပါ။      // the value itself
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၉
၂၅
လုပ်ဆောင်ချက် နှစ်ထပ် ၏ လိပ်စာ
```

Each call form works through the variable: the name after `လုပ်ဆောင်ချက်`, the name before
`(`, and the arguments-first forms. A function value prints as the phrase that makes it.

## The kind of a function

A function value has a *function kind*. It reads like the header of the function, with
`သော` ("that") and `လုပ်ဆောင်ချက် အမျိုးအစား` ("function kind") at the end:

| Kind | Reads as |
| --- | --- |
| `ကိန်းပြည့် ကို လက်ခံပြီး ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား` | "a function kind that accepts an integer and gives back an integer result" |
| `စာသား ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား` | "a function kind that accepts text" |
| `ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား` | "a function kind that gives back an integer result" |
| `လုပ်ဆောင်ချက် အမျိုးအစား` | "a function kind": no parameters, no result |

The parameters can have names, `ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို`, or not,
`ကိန်းပြည့်၊ ကိန်းပြည့် တို့ကို`; a name just explains the parameter. A declaration
counts the kind with `တစ်ခု`, as other kinds:

```zaka
လုပ်ဆောင်ချက် ပေါင်း သည်
ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က + ခ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လုပ် သည် ကိန်းပြည့်၊ ကိန်းပြည့် တို့ကို လက်ခံပြီး ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် ပေါင်း ၏ လိပ်စာ ဖြစ်ပါစေ။
လုပ်(၂၊ ၃) ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၅
```

## Passing a function

A parameter of a function kind takes a function from the caller. In a list of
parameters, the kind of such a parameter stands in parentheses, so its own parameters
do not mix with those of the list:

```zaka
လုပ်ဆောင်ချက် နှစ်ဆ သည် ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က * ၂ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် နှစ်ထပ် သည် ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က * က ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// a new list: each cell changed with the function the caller gives
လုပ်ဆောင်ချက် ကွက်တိုင်း သည်
ကိန်းပြည့် စာရင်း စ၊
(ကိန်းပြည့် ကို လက်ခံပြီး ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) ပြောင်း
တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် စာရင်း ရလဒ် ကို ပြန်ပေးမည်။
စ ၏ အကွက်တိုင်း ကို တန် အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ရလဒ် ထဲသို့ ပြောင်း(တန်) ကို ထည့်ပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဂဏန်း သည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ကွက်တိုင်း ကို ဂဏန်း၊ လုပ်ဆောင်ချက် နှစ်ဆ ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
လုပ်ဆောင်ချက် ကွက်တိုင်း ကို ဂဏန်း၊ လုပ်ဆောင်ချက် နှစ်ထပ် ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၂၊ ၄၊ ၆]
[၁၊ ၄၊ ၉]
```

`ကွက်တိုင်း` does not know what each cell turns into: the function it is given decides.
The same idea sorts a list in each order. The sort asks the function `ရှေ့` ("comes
first") whether one number comes before another:

```zaka
လုပ်ဆောင်ချက် ငယ်စဉ် သည် ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး ရလဒ် ကို ပြန်ပေးမည်။
က < ခ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ကြီးစဉ် သည် ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး ရလဒ် ကို ပြန်ပေးမည်။
က > ခ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် စီပေး သည်
လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သော ကိန်းပြည့် စာရင်း ကိန်းများ၊
(ကိန်းပြည့်၊ ကိန်းပြည့် တို့ကို လက်ခံပြီး မှန်မှားတန်ဖိုး ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) ရှေ့
တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ မှ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၁ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် မ သည် န ဖြစ်ပါစေ။
// move the cell back while it comes before the cell in front of it
မ > ၀ && ရှေ့(ကိန်းများ ၏ အကွက် မ၊ ကိန်းများ ၏ အကွက် (မ - ၁)) ဖြစ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် ယာယီ သည် ကိန်းများ ၏ အကွက် မ ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက် မ ကို ကိန်းများ ၏ အကွက် (မ - ၁) ဟု ထားပါ။
ကိန်းများ ၏ အကွက် (မ - ၁) ကို ယာယီ ဟု ထားပါ။
မ ကို မ - ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် [၅၊ ၂၊ ၉၊ ၁၊ ၇] ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် စီပေး ကို ကိန်းများ (လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်)၊ လုပ်ဆောင်ချက် ငယ်စဉ် ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်ပါ။
ကိန်းများ ကို ပြသပါ။
လုပ်ဆောင်ချက် စီပေး ကို ကိန်းများ (လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်)၊ လုပ်ဆောင်ချက် ကြီးစဉ် ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်ပါ။
ကိန်းများ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၂၊ ၅၊ ၇၊ ၉]
[၉၊ ၇၊ ၅၊ ၂၊ ၁]
```

## A function that uses the variables around it

A function declared inside another function, or inside the start block, can use the
variables above it. It lists each one it uses in the list of its parameters, and tells how
it takes it. With such a list, each item ends in `ကိုလည်းကောင်း` ("and also"), the
parameters too:

| Item | Reads as | The function gets |
| --- | --- | --- |
| `တိုး ကိုလည်းကောင်း` | "and တိုး" | a duplicate of its own |
| `တိုး ကို အပိုင်အနေဖြင့်လည်းကောင်း` | "and တိုး, as its own" | the value, moved; `တိုး` is left blank |
| `တိုး ကို ငှား၍လည်းကောင်း` | "and တိုး, borrowed" | `တိုး` itself |

The listed variables are its *captures*, and its address, a function value that carries
them, is a *closure*. A borrowed variable is shared, so each call sees its current value:

```zaka
လုပ်ဆောင်ချက် ကွက်တိုင်း သည်
ကိန်းပြည့် စာရင်း စ၊
(ကိန်းပြည့် ကို လက်ခံပြီး ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) ပြောင်း
တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် စာရင်း ရလဒ် ကို ပြန်ပေးမည်။
စ ၏ အကွက်တိုင်း ကို တန် အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ရလဒ် ထဲသို့ ပြောင်း(တန်) ကို ထည့်ပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် တိုး သည် ၁၀ ဖြစ်ပါစေ။
// a function that borrows တိုး, a variable around it
လုပ်ဆောင်ချက် ထည့် သည် ကိန်းပြည့် က ကိုလည်းကောင်း၊ တိုး ကို ငှား၍လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က + တိုး ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် ကွက်တိုင်း ကို [၁၊ ၂၊ ၃]၊ လုပ်ဆောင်ချက် ထည့် ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
တိုး ကို ၅၀ ဟု ထားပါ။
လုပ်ဆောင်ချက် ကွက်တိုင်း ကို [၁၊ ၂၊ ၃]၊ လုပ်ဆောင်ချက် ထည့် ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၁၊ ၁၂၊ ၁၃]
[၅၁၊ ၅၂၊ ၅၃]
```

A variable the list leaves out is an error, ZK0339, so each function shows what it takes
from around it.

## A function that keeps values

A duplicate or a move belongs to the closure, and what the function writes to it remains for
the next call. Such a closure owns what it keeps, so it can leave the block that made it: here
a function gives back a counter.

```zaka
လုပ်ဆောင်ချက် ရေတွက်စက် သည် ကိန်းပြည့် စ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ (လုပ်ဆောင်ချက် အမျိုးအစား) ရလဒ် ကို ပြန်ပေးမည်။
ကိန်းရှင် ကြိမ် သည် စ ဖြစ်ပါစေ။
// a duplicate of ကြိမ်, which this counter keeps
လုပ်ဆောင်ချက် တိုး သည် ကြိမ် ကိုလည်းကောင်း လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကြိမ် ကို ကြိမ် + ၁ ဟု ထားပါ။
ကြိမ် ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် တိုး ၏ လိပ်စာ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရေ သည် လုပ်ဆောင်ချက် ရေတွက်စက် ကို ၁၀ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ရေ ကို လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် ရေ ကို လုပ်ဆောင်ပါ။
// a duplicate of the closure counts on its own
ကိန်းရှင် နောက်တစ်ခု ထဲသို့ ရေ ကို ကူးထည့်မည်။
လုပ်ဆောင်ချက် နောက်တစ်ခု ကို လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် ရေ ကို လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၁
၁၂
၁၃
၁၃
```

A closure handed to a function is lent for the call: the function runs that same closure,
so its count goes on.

A closure that borrows uses the variables of the block that declares it, so it cannot leave
that block. Giving its address back as a result would leave the function with variables
that are gone, and the compiler refuses it with ZK0722:

```zaka-error ZK0722
လုပ်ဆောင်ချက် ဖန်တီး သည် ကိန်းပြည့် တိုး ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ (ကိန်းပြည့် ကို လက်ခံပြီး ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် ထည့် သည် ကိန်းပြည့် က ကိုလည်းကောင်း၊ တိုး ကို ငှား၍လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က + တိုး ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် ထည့် ၏ လိပ်စာ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0722]: "ထည့်" borrows the locals around it, so its address cannot go where it outlives them
```

List `တိုး ကိုလည်းကောင်း` in its place, and the closure keeps a duplicate and can go. The full
rules are in
[Nested functions and closures](../../manual/language/function-values.md#nested-functions-and-closures).

## Naming a function kind

A function kind is long to write in each header. A [kind
alias](../../manual/language/aliases.md) names it once, and the name then stands where
the kind does. A function kind needs no `ဒေတာအမျိုးအစား` before it, and its alias is one
word, so a list of parameters takes it with no parentheses:

```zaka
ကိန်းပြည့်၊ ကိန်းပြည့် တို့ကို လက်ခံပြီး ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား ကို တွက်ချက် ဟုလည်း မှတ်ယူနိုင်ပါသည်။

လုပ်ဆောင်ချက် ပေါင်း သည် ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က + ခ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် မြှောက် သည် ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က * ခ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// join the cells of a list, starting from a value, with the function the caller gives
လုပ်ဆောင်ချက် စုစည်း သည် ကိန်းပြည့် စာရင်း စ၊ ကိန်းပြည့် အစ၊ တွက်ချက် ဖ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
ရလဒ် ကို အစ ဟု ထားပါ။
စ ၏ အကွက်တိုင်း ကို တန် အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ရလဒ် ကို ဖ(ရလဒ်၊ တန်) ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဂဏန်း သည် [၁၊ ၂၊ ၃၊ ၄] ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် စုစည်း ကို ဂဏန်း၊ ၀၊ လုပ်ဆောင်ချက် ပေါင်း ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
လုပ်ဆောင်ချက် စုစည်း ကို ဂဏန်း၊ ၁၊ လုပ်ဆောင်ချက် မြှောက် ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၀
၂၄
```

## A table of functions

A list can hold function values. A call names the cell where it stands, after
`လုပ်ဆောင်ချက်`, so the table picks the function at run time. Here the cells follow the
members of an enumeration:

```zaka
ဒေတာအမျိုးအစား လုပ်ငန်း ဆိုသည်မှာ ပေါင်း၊ နုတ်၊ မြှောက် တို့ ဖြစ်သည်။

လုပ်ဆောင်ချက် ပေါင်းပေး သည် ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က + ခ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် နုတ်ပေး သည် ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က - ခ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် မြှောက်ပေး သည် ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က * ခ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ ဇယား သည် [လုပ်ဆောင်ချက် ပေါင်းပေး ၏ လိပ်စာ၊ လုပ်ဆောင်ချက် နုတ်ပေး ၏ လိပ်စာ၊ လုပ်ဆောင်ချက် မြှောက်ပေး ၏ လိပ်စာ] ဖြစ်ပါစေ။
ကိန်းရှင် ခိုင်း သည် [လုပ်ငန်းများထဲမှ မြှောက်၊ လုပ်ငန်းများထဲမှ ပေါင်း၊ လုပ်ငန်းများထဲမှ နုတ်] ဖြစ်ပါစေ။
ခိုင်း ၏ အကွက်တိုင်း ကို ခု အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် အမှတ် သည် ခု ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
"{ခု}: {လုပ်ဆောင်ချက် ဇယား ၏ အကွက် အမှတ် ကို ၆၊ ၃ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်}" ဟု ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
လုပ်ငန်းများထဲမှ မြှောက်: ၁၈
လုပ်ငန်းများထဲမှ ပေါင်း: ၉
လုပ်ငန်းများထဲမှ နုတ်: ၃
```

A struct field can hold a function value too, and a call names it the same:
`လုပ်ဆောင်ချက် ခလုတ် ၏ လုပ် ကို ၁ တို့ဖြင့် လုပ်ဆောင်ပါ။`

## No function: ဗလာ

A variable of a function kind with no value holds `ဗလာ`: no function. The compiler
catches a call through it:

```zaka-error ZK0709
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် ကိန်းပြည့် ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား တစ်ခု ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ဖ ကို ၁ တို့ဖြင့် လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0709]: "ဖ" is ဗလာ here, so it holds no function to call
```

Compare a function value with `ဗလာ` before the call when it can hold none. The full rules,
with marks, functions of one name and the errors of each, are in
[Function values](../../manual/language/function-values.md).

## Hands on

1. Write `နှစ်ကြိမ်`, which takes a function of kind
   `ကိန်းပြည့် ကို လက်ခံပြီး ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား` and a number,
   and runs the function twice on the number: given the address of `နှစ်ဆ` and `၃`, it
   gives `၁၂`.
2. Add `ပိုင်း` ("divide") to the table of functions, and a member for it.

Next: [Ownership and borrowing](ownership.md).
