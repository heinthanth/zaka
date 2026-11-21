---
title: One name, several functions
description: Overloaded functions, which share a name and differ in their parameter kinds, prepositions or phrases, and how a call picks one.
---

# One name, several functions

A file can declare several functions with one name when their parameter kinds or their
prepositions differ, which other languages call function overloading. A call then runs the
one function that its arguments fit. When we want one function for each data kind (type),
written once, we use a [generic](generics.md) instead. For the declaration of a single
function, see [Functions](functions.md#declaring-a-function).

```zaka
ပြ ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ကိန်း {န}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပြ ဟူသော လုပ်ဆောင်ချက် သည်
စ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"စာ {စ}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပြ ဟူသော လုပ်ဆောင်ချက် သည်
စ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ထဲသို့" ကို သုံး၍ လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"{စ} ထဲသို့" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၅ ကို ပြ မည်။
"ဟယ်လို" ကို ပြ မည်။
"အိမ်" ထဲသို့ ပြ မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ကိန်း ၅
စာ ဟယ်လို
အိမ် ထဲသို့
```

## How a call picks a function

A call runs the function of the name that its arguments fit: there is an argument for each
parameter, the arguments have the prepositions of the parameters when the call writes
prepositions, and each argument fits the data kind of its parameter, under the rules that a
call to a single function follows. The prepositions, and the
[phrases](functions-sentences.md#starting-and-ending-phrases) of a call written as a
sentence, can tell two functions apart on their own, so a preposition call also chooses
from the words it writes.

| Case | Code |
| --- | --- |
| no function of the name fits the call; the help lists the parameters of each one | ZK0332 |
| two or more functions fit the call | ZK0333 |
| two functions with one name and the same parameter kinds and prepositions in one file | ZK0304 |

A number with no data kind fits each integer kind, and a call with no prepositions fits
each function that differs from the others in its prepositions alone. To pick one, we write
a data kind on the number, or the prepositions of the function we mean.

```zaka-error ZK0333
ပြ ဟူသော လုပ်ဆောင်ချက် သည်
စ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပြ ဟူသော လုပ်ဆောင်ချက် သည်
စ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ထဲသို့" ကို သုံး၍ လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပြ ဟူသော လုပ်ဆောင်ချက်ကို "ဟယ်လို" ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0333]: more than one function "ပြ" takes these arguments
```

```zaka-error ZK0332
ပြ ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပြ ဟူသော လုပ်ဆောင်ချက် သည်
စ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
မှန် ကို ပြ မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0332]: no function "ပြ" takes these arguments
```

## Overloads and imports

An import of a name brings each exported function of that name. Imports of one name from
several files join into one set, so `ပါမပါစစ်` of text and `ပါမပါစစ်` of lists can both be
imported. Two functions with the same parameter kinds, prepositions and phrases, from one
file or from two, are ZK0304, and a file cannot declare a function with a name that it
imports (ZK0304).

Inside the compiler, the second function of a name is `<name>#2` and the third is
`<name>#3`, and the IR, the flow checker and the VM listing show these names.
