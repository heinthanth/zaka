---
title: Parameters
description: Pass values into a function.
---

# Parameters

A function becomes far more useful when it receives values to work with. The values a
function accepts are its *parameters*. The values passed in a call are the *arguments*.

```zaka
// "Function count-up, accepting an integer amount, will do the following"
လုပ်ဆောင်ချက် ရေတွက်ပြ သည်
အရေအတွက် ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ မှ အရေအတွက် အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// "Run the function count-up with 3"
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ရေတွက်ပြ ဟူသော လုပ်ဆောင်ချက်ကို ၃ ဖြင့် လုပ်ဆောင်မည်။
"---" ဟု ပြသမည်။
ရေတွက်ပြ ဟူသော လုပ်ဆောင်ချက်ကို ၂ ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁
၂
၃
---
၁
၂
```

Inside the function, `အရေအတွက်` is a variable that holds whatever value the call passed
in: 3 the first time, 2 the second.

| Zaka | Reads as |
| --- | --- |
| `အရေအတွက် ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး` | "accepting an integer called amount, and ..." |
| `လက်ခံပြီး` | "having accepted" |
| `၃ ဖြင့်` | "with 3" |

## Several parameters

Write each parameter as `<name> ဟူသော <kind> တစ်ခု ကို`, the kind counted with its measure
word, and end each one with `လည်းကောင်း` ("and also") when there are two or more. Pass the
arguments in the same order:

```zaka
// rectangle: print the area of a length × width rectangle
လုပ်ဆောင်ချက် ဧရိယာပြ သည်
အလျား ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း အနံ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အလျား * အနံ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဧရိယာပြ ဟူသော လုပ်ဆောင်ချက်ကို ၃ နှင့် ၄ တို့ဖြင့် လုပ်ဆောင်မည်။
ဧရိယာပြ ဟူသော လုပ်ဆောင်ချက်ကို ၁၀ နှင့် ၂ + ၃ တို့ဖြင့် လုပ်ဆောင်မည်။     // arguments can be sums
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၂
၅၀
```

| Number of parameters | Each ends in | Example |
| --- | --- | --- |
| one | `ကို` | `က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး` |
| two or more | `ကို လည်းကောင်း` | `က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး` |

Arguments close with `ဖြင့်` ("with") after one argument, and with `တို့ဖြင့်` ("with
these") after two or more, which take `နှင့်` before the last: `၃ နှင့် ၄ တို့ဖြင့်`.

## Arguments first

A call can also give the arguments first, then name the function, the order a Burmese
sentence puts an object before its verb. `သုံး၍` ("using") closes the arguments, and
`မည်` ("will") ends the call:

```zaka
လုပ်ဆောင်ချက် ဧရိယာပြ သည်
အလျား ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း အနံ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အလျား * အနံ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
// "Using 3 and 4, (the function) area-show will (run)"
၃ နှင့် ၄ တို့ကို သုံး၍ ဧရိယာပြ မည်။
၆ နှင့် ၇ တို့ကို သုံး၍ ဧရိယာပြ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၂
၄၂
```

| Zaka | Reads as |
| --- | --- |
| `၃ နှင့် ၄ တို့ကို သုံး၍` | "using 3 and 4": `တို့ကို` after several arguments, `ကို` after one |
| `ဧရိယာပြ မည်` | "area-show will (run)" |
| `ဧရိယာပြ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်` | "will run the function area-show": the long form |

An argument here is a name, a literal, a sum or a sign-form call. A sentence that gives a
value goes in parentheses. The [Results](results.md) page shows the form of this call
that gives back a value.

## Prepositions

A parameter can also name the word that follows its argument in a call, its *preposition*, with
`ဝိဘတ် "<preposition>" ကို သုံး၍` ("using the particle …") after its `ကို`, and a call reads as a Burmese sentence, with no `သုံး၍`:

```zaka
// "Function write, accepting a text (preposition ကို) and a place (preposition သို့)"
လုပ်ဆောင်ချက် ရေး သည်
စာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လည်းကောင်း နေရာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "သို့" ကို သုံး၍ လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"{နေရာ}: {စာ}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
// "(I) will write 'hello' to the notebook"
"မင်္ဂလာပါ" ကို "မှတ်စု" သို့ ရေး မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှတ်စု: မင်္ဂလာပါ
```

| Zaka | Reads as |
| --- | --- |
| `စာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လည်းကောင်း` | "a text called `စာ`, written with `ကို` in a call" |
| `နေရာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "သို့" ကို သုံး၍ လည်းကောင်း လက်ခံပြီး` | "a text called `နေရာ`, written with `သို့`; having accepted" |
| `"မင်္ဂလာပါ" ကို "မှတ်စု" သို့ ရေး မည်` | "will write 'hello' to the notebook" |

Each argument takes the preposition of its parameter, in the order of the parameters. Give each
parameter a preposition or none of them. The [Functions](../../manual/language/functions.md#prepositions)
page of the manual has the full rules.

## Prepositions of several words

A preposition can be several words, as Burmese often puts a phrase after a noun:
`"ရန်ကုန်" မှ စ၍` is "starting from Yangon". Write the words in the string of `ဝိဘတ်`,
with the spaces of the sentence:

```zaka
// "Function show trip: a start (preposition မှ စ၍) and an end (preposition အထိ)"
လုပ်ဆောင်ချက် ခရီးစဉ်ပြ သည်
စ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "မှ စ၍" ကို သုံး၍ လည်းကောင်း ဆုံး ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "အထိ" ကို သုံး၍ လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"{စ} → {ဆုံး}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
// "(I) will show the trip starting from Yangon up to Mandalay"
"ရန်ကုန်" မှ စ၍ "မန္တလေး" အထိ ခရီးစဉ်ပြမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရန်ကုန် → မန္တလေး
```

The call also glues `မည်` to the name: `ခရီးစဉ်ပြမည်` reads as `ခရီးစဉ်ပြ မည်`, since the
file has a function `ခရီးစဉ်ပြ`. The endings that give a value glue alike, as
`တွဲသည့်ရလဒ်` below.

The last preposition can start with `၏`, so a call reads "at the end of the text":

```zaka
// "Function join: a piece (preposition ကို) at the end of a text (preposition ၏ နောက်ဆုံးတွင်)"
လုပ်ဆောင်ချက် တွဲ သည်
ပိုင်း ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လည်းကောင်း စာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "၏ နောက်ဆုံးတွင်" ကို သုံး၍ လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"{စာ}{ပိုင်း}" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
// "Show the result of joining 'pa' at the end of 'mingala'"
"ပါ" ကို "မင်္ဂလာ" ၏ နောက်ဆုံးတွင် တွဲသည့်ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ
```

`၏` starts the last preposition alone; before another parameter it is ZK0249, and
elsewhere `x ၏ y` reads a field, as before. A call reads the longest preposition of the
function that the words after an argument spell, so `မှ စ၍` is one preposition, not `မှ`
and then a name.

## Names that read as calls

Gluing has a cost. In a file with a function `နာ` ("hurt"), the word `နာမည်` ("name") can
read as `နာ` and `မည်`. Declaring a variable of that name is the warning ZK0251:

```zaka-warning ZK0251
// "Function hurt": with it in the file, နာမည် ("name") can read as နာ + မည်
လုပ်ဆောင်ချက် နာ သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"အား" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် နာမည် သည် "မောင်" ဖြစ်ပါစေ။
{နာမည်} ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

A name in braces is a name and nothing else. Declare it as `{နာမည်}`, write `{နာမည်}`
where the name is meant, and the bare word stands for the call:

```zaka
လုပ်ဆောင်ချက် နာ သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"အား" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် {နာမည်} သည် "မောင်" ဖြစ်ပါစေ။
{နာမည်} ကို ပြသမည်။
နာမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မောင်
အား
```

A file that has no function `နာ` reads `နာမည်` as a name, with no warning. The
[Functions](../../manual/language/functions.md#what-the-flexible-grammar-costs) page of the
manual lists each cost of these forms.

## One name, several functions

Two functions can share a name when their parameter kinds or their prepositions differ. A call
runs the one its arguments fit:

```zaka
လုပ်ဆောင်ချက် ပြ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ကိန်း {န}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပြ သည်
စ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"စာ {စ}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၅ ကို ပြ မည်။            // the integer one
"ဟယ်လို" ကို ပြ မည်။      // the text one
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ကိန်း ၅
စာ ဟယ်လို
```

When no function of the name fits the arguments, or two fit, the compiler reports it
(ZK0332, ZK0333); the [Functions](../../manual/language/functions.md#one-name-several-functions)
page of the manual has the rules.

## Parameters are local copies

A parameter is a new variable inside the function. Changing it does not change the
variable that was passed in:

```zaka
လုပ်ဆောင်ချက် တိုးပြ သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို က + ၁ ဟု ထားမည်။
က ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
တိုးပြ ဟူသော လုပ်ဆောင်ချက်ကို ခ ဖြင့် လုပ်ဆောင်မည်။     // prints 6
ခ ကို ပြသမည်။                                    // still 5
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၆
၅
```

## Changing the caller's variable

To change the variable a caller holds, return the new value and set the variable from it:

```zaka
လုပ်ဆောင်ချက် နောက်တစ် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က + ၁ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ၅ ဖြစ်ပါစေ။
ခ ကို နောက်တစ်(ခ) ဟု ထားမည်။
ခ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၆
```

When a function must change the variable itself, write `ဒေတာ ပြောင်းလဲရန်` ("to change the data") after the
`ကို` of the parameter, and pass the variable as usual:

```zaka
လုပ်ဆောင်ချက် တိုး သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို က + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ၅ ဖြစ်ပါစေ။
တိုး ဟူသော လုပ်ဆောင်ချက်ကို ခ ဖြင့် လုပ်ဆောင်မည်။
ခ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၆
```

The function works on its own value, which goes back to the variable when the call
returns. The argument must be a variable, a field or a cell, and it cannot appear twice in
one call. The function can read it, so the caller gives it a value first. See
[Changed parameters](../../manual/language/functions.md#changed-parameters) in the Docs.

## Filling the caller's variable

A function that writes a variable and never reads it first marks the parameter
`ဒေတာ ထည့်ပေးရန်` ("to put the data in"). The caller can declare the variable with no
value; after the call it holds what the function put in it. So one call gives back
two values:

```zaka
လုပ်ဆောင်ချက် စားခွဲ သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း စား ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ထည့်ပေးရန် လည်းကောင်း ကြွင်း ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ထည့်ပေးရန် လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စား ကို က / ခ ဟု ထားမည်။
ကြွင်း ကို က % ခ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဆ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ကျန် သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
စားခွဲ ဟူသော လုပ်ဆောင်ချက်ကို ၁၇၊ ၅၊ ဆ နှင့် ကျန် တို့ဖြင့် လုပ်ဆောင်မည်။
"{ဆ} {ကျန်}" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၃ ၂
```

A list can come in as well. The next example uses lists, which [Lists](../lists/README.md)
teaches later; come back to it after that chapter.

```zaka
လုပ်ဆောင်ချက် အစွန်းများ သည်
ကိန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လည်းကောင်း အငယ် ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ထည့်ပေးရန် လည်းကောင်း အကြီး ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ထည့်ပေးရန် လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အငယ် ကို ကိန်းများ ၏ အကွက် ၀ ဟု ထားမည်။
အကြီး ကို ကိန်းများ ၏ အကွက် ၀ ဟု ထားမည်။
ကိန်းများ ၏ အကွက်တိုင်း ကို န အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ န သည် အငယ် အောက် ငယ်လျှင် အငယ် ကို န ဟု ထားမည်။
အကယ်၍ န သည် အကြီး ထက် ကြီးလျှင် အကြီး ကို န ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် နိမ့် သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် မြင့် သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
အစွန်းများ ဟူသော လုပ်ဆောင်ချက်ကို [၄၊ ၉၊ ၂၊ ၇]၊ နိမ့် နှင့် မြင့် တို့ဖြင့် လုပ်ဆောင်မည်။
"{နိမ့်} {မြင့်}" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂ ၉
```

Inside the function the parameter starts with no value, so the function must set it
before it reads it, and on each path before it returns. A path that leaves it unset is
refused:

```zaka-error ZK0725
လုပ်ဆောင်ချက် အပေါင်းဖြစ်လျှင်ထား သည်
တန် ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လဒ် ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ထည့်ပေးရန် လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ တန် သည် ၀ ထက် ကြီးလျှင် လဒ် ကို တန် ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0725]: "လဒ်" is marked "ဒေတာ ထည့်ပေးရန်", so the function must set it on each path, but a path reaches this return without setting it
```

See [Filled parameters](../../manual/language/functions.md#filled-parameters) in the Docs.

## Matching the call to the function

The call must pass the same number of arguments as the function has parameters, and
each must fit its parameter's kind:

```zaka-error ZK0314
လုပ်ဆောင်ချက် ပြ သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပြ ဟူသော လုပ်ဆောင်ချက်ကို ၁ နှင့် ၂ တို့ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0314]: function "ပြ" takes 1 arguments, 2 given
```

## Hands on

- Write a function that takes a number and prints its times table from 1 to 10.
- Give `ရေတွက်ပြ` a guard: `အကယ်၍ အရေအတွက် သည် ၁ အောက် ငယ်လျှင် ပြန်ပေးမည်။`, then call it
  with `၀`.

Next: [Results](results.md).
