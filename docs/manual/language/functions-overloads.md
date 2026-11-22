---
title: One name, several functions
description: Overloaded functions, which share a name and differ in their parameter kinds, prepositions, phrases or result, and how a call picks one.
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
| two functions with one name, the same parameter kinds and prepositions, and the same result data kind in one file | ZK0304 |

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

## Overloads that differ in the result

Two functions of one name can also take the same parameters and give different data kinds.
A call then runs the one whose result data kind is the data kind that the place of the call
expects: the data kind of a declaration, of a parameter, or of the value that a function
returns. A call whose place expects no data kind, such as a call that `ပြသမည်` prints, fits
both, and is ZK0333.

```zaka
ယူ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော မှန်မှားတန်ဖိုး တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ က ဖြစ်လျှင် "ဟုတ်" ကို ပြန်ပေးမည်။
"မဟုတ်" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ယူ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော မှန်မှားတန်ဖိုး တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ က ဖြစ်လျှင် ၁ ကို ပြန်ပေးမည်။
၀ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ယူ(မှန်) ဖြစ်ပါစေ။
ကိန်းရှင် န သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ယူ(မှန်) ဖြစ်ပါစေ။
စ ကို ပြသမည်။
န + ၁ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဟုတ်
၂
```

```zaka-error ZK0333
ယူ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော မှန်မှားတန်ဖိုး တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ က ဖြစ်လျှင် "ဟုတ်" ကို ပြန်ပေးမည်။
"မဟုတ်" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ယူ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော မှန်မှားတန်ဖိုး တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ က ဖြစ်လျှင် ၁ ကို ပြန်ပေးမည်။
၀ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ယူ(မှန်) ဖြစ်ပါစေ။
ကိန်းရှင် န သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ယူ(မှန်) ဖြစ်ပါစေ။
ယူ(မမှန်) ကို ပြသမည်။
စ ကို ပြသမည်။
န + ၁ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0333]: more than one function "ယူ" takes these arguments
```

## Overloads and imports

An import of a name brings each exported function of that name. Imports of one name from
several files join into one set, so `ပါမပါစစ်` of text and `ပါမပါစစ်` of lists can both be
imported. Two functions with the same parameter kinds, prepositions, phrases and result data kind,
from one file or from two, are ZK0304, and a file cannot declare a function with a name that it
imports (ZK0304).

Inside the compiler, the second function of a name is `<name>#2` and the third is
`<name>#3`, and the IR, the flow checker and the VM listing show these names.
