---
title: Functions as values
description: Keep a function in a variable, write its function kind, pass it to another function, and pick one from a list.
---

# Functions as values

A function value (first-class function) is a function that we keep in a variable or pass
to another function, just as we do with a number. When we call the variable, the function
in it runs, so one piece of code can run different functions at different times.

## You will learn

- How to keep a function in a variable and call it through the variable.
- How to write the function kind (function type) of a function value.
- How to pass a function to another function.
- How to keep functions in a list, and how to state that a function can be missing.

## A function in a variable

```zaka
နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// the function, kept in a variable
ကိန်းရှင် ဖ သည် နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။
ဖ(၇) ကို ပြသမည်။
ဖ ဟူသော လုပ်ဆောင်ချက်ကို ၅ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ဖ ကို ပြသမည်။      // the value itself
```

```text
၄၉
၂၅
နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက်
```

To get a function as a value, we write its name and `ဟူသော လုပ်ဆောင်ချက်`, without a call:

```
<name> ဟူသော လုပ်ဆောင်ချက်
```

| Zaka | Reads as |
| --- | --- |
| `နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက်` | "the function called `နှစ်ထပ်`" |
| `ဖ(၇)` | "`ဖ` of 7": a call through the variable |

We can call the variable in each form that we use for a plain function: the short form
`ဖ(၇)`, the long form `ဖ ဟူသော လုပ်ဆောင်ချက်ကို ၅ ဖြင့် လုပ်ဆောင်သော ရလဒ်`, and the forms
that put the arguments first. When we print a function value, it prints as the phrase
that makes it.

## The function kind

A function value has a function kind (function type), which lists the data kinds (types)
of its parameters and its result. We write it like a function signature without names,
and we end it with `သော လုပ်ဆောင်ချက် အမျိုးအစား`:

```
<kind> တစ်ခု ကို လက်ခံပြီး <kind> တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား
```

| Function kind | Reads as |
| --- | --- |
| `ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား` | "a function kind that accepts an integer and returns an integer" |
| `စာသား တစ်ခု ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား` | "a function kind that accepts text" |
| `ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား` | "a function kind that returns an integer" |
| `လုပ်ဆောင်ချက် အမျိုးအစား` | "a function kind": no parameters and no result |

Each parameter is its data kind, its measure word (classifier) and `ကို`. When there are
two or more parameters, we end each one with `လည်းကောင်း`, the same as in a function
signature. A variable of a function kind is counted with `တစ်ခု`, like other data kinds:

```zaka
ပေါင်း ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က + ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် လုပ် သည် ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ပေါင်း ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။
လုပ်(၂၊ ၃) ကို ပြသမည်။
```

```text
၅
```

## Passing a function

A parameter can have a function kind, and the caller then passes a function to it. In a
function signature, we put such a function kind in parentheses, so that its parameters do
not mix with the parameters around it:

```
<name> ဟူသော (<function kind>) တစ်ခု ကို
```

```zaka
နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// a new list: each cell changed with the function the caller passes
ကွက်တိုင်း ဟူသော လုပ်ဆောင်ချက် သည်
စ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လည်းကောင်း
ပြောင်း ဟူသော (ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို လည်းကောင်း
လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် စာရင်း တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [] ဖြစ်ပါစေ။
စ ၏ အကွက်တိုင်း ကို တန် အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အဖြေ ထဲသို့ ပြောင်း(တန်) ကို ထည့်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ဂဏန်း သည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကွက်တိုင်း ဟူသော လုပ်ဆောင်ချက်ကို ဂဏန်း နှင့် နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ကွက်တိုင်း ဟူသော လုပ်ဆောင်ချက်ကို ဂဏန်း နှင့် နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက် တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
```

```text
[၂၊ ၄၊ ၆]
[၁၊ ၄၊ ၉]
```

`ကွက်တိုင်း` does not know what each cell turns into, because the function that we pass
decides that. We can use the same idea to sort a list in either order. Here the sort asks
the function `ရှေ့` ("comes first") whether one number comes before another:

```zaka
ငယ်စဉ် ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
က < ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကြီးစဉ် ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
က > ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

စီပေး ဟူသော လုပ်ဆောင်ချက် သည်
ကိန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လည်းကောင်း ရှေ့ ဟူသော (ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ (ကိန်းများ ၏ အကွက်အရေအတွက်) == ၀ ဖြစ်လျှင် ပြန်ပေးမည်။
၁ မှ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၁ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မ သည် န ဖြစ်ပါစေ။
// move the cell back while it comes before the cell in front of it
မ > ၀ && ရှေ့(ကိန်းများ ၏ အကွက် မ၊ ကိန်းများ ၏ အကွက် (မ - ၁)) ဖြစ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ယာယီ သည် ကိန်းများ ၏ အကွက် မ ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက် မ ကို ကိန်းများ ၏ အကွက် (မ - ၁) ဟု ထားမည်။
ကိန်းများ ၏ အကွက် (မ - ၁) ကို ယာယီ ဟု ထားမည်။
မ ကို မ - ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ကိန်းများ သည် [၅၊ ၂၊ ၉၊ ၁၊ ၇] ဖြစ်ပါစေ။
စီပေး ဟူသော လုပ်ဆောင်ချက်ကို ကိန်းများ နှင့် ငယ်စဉ် ဟူသော လုပ်ဆောင်ချက် တို့ဖြင့် လုပ်ဆောင်မည်။
ကိန်းများ ကို ပြသမည်။
စီပေး ဟူသော လုပ်ဆောင်ချက်ကို ကိန်းများ နှင့် ကြီးစဉ် ဟူသော လုပ်ဆောင်ချက် တို့ဖြင့် လုပ်ဆောင်မည်။
ကိန်းများ ကို ပြသမည်။
```

```text
[၁၊ ၂၊ ၅၊ ၇၊ ၉]
[၉၊ ၇၊ ၅၊ ၂၊ ၁]
```

## Naming a function kind

A function kind is long to write in each function signature, so we can give it a short
name with a [kind alias](../structs/README.md#another-name-for-a-kind) (type alias). The
alias is one word, so we can use it in a list of parameters without parentheses:

```
ဒေတာအမျိုးအစား <name> ကို <function kind> ဟုလည်း မှတ်ယူနိုင်ပါသည်။
```

```zaka
ဒေတာအမျိုးအစား တွက်ချက် ကို ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား ဟုလည်း မှတ်ယူနိုင်ပါသည်။

ပေါင်း ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က + ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

မြှောက် ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// join the cells of a list, starting from a value, with the function the caller passes
စုစည်း ဟူသော လုပ်ဆောင်ချက် သည် စ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လည်းကောင်း အစ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ဖ ဟူသော တွက်ချက် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
အဖြေ ကို အစ ဟု ထားမည်။
စ ၏ အကွက်တိုင်း ကို တန် အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အဖြေ ကို ဖ(အဖြေ၊ တန်) ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ဂဏန်း သည် [၁၊ ၂၊ ၃၊ ၄] ဖြစ်ပါစေ။
စုစည်း ဟူသော လုပ်ဆောင်ချက်ကို ဂဏန်း၊ ၀ နှင့် ပေါင်း ဟူသော လုပ်ဆောင်ချက် တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
စုစည်း ဟူသော လုပ်ဆောင်ချက်ကို ဂဏန်း၊ ၁ နှင့် မြှောက် ဟူသော လုပ်ဆောင်ချက် တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
```

```text
၁၀
၂၄
```

## A list of functions

A list can contain function values, and a call can name a cell of the list in place of a
function name, so the program picks the function while it runs. We put the cell in
parentheses before `ဟူသော လုပ်ဆောင်ချက်ကို`:

```
(<list> ၏ အကွက် <number>) ဟူသော လုပ်ဆောင်ချက်ကို <arguments> ဖြင့် လုပ်ဆောင်သော ရလဒ်
```

Here the cells follow the members (variants) of an enumeration (enum):

```zaka
လုပ်ငန်း ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ပေါင်း၊ နုတ် နှင့် မြှောက် တို့ပါဝင်သည်။

ပေါင်းပေး ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က + ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

နုတ်ပေး ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က - ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

မြှောက်ပေး ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းသေ ဂရစ် သည် [ပေါင်းပေး ဟူသော လုပ်ဆောင်ချက်၊ နုတ်ပေး ဟူသော လုပ်ဆောင်ချက်၊ မြှောက်ပေး ဟူသော လုပ်ဆောင်ချက်] ဖြစ်ပါစေ။
ကိန်းရှင် ခိုင်း သည် [လုပ်ငန်းများထဲမှ မြှောက်၊ လုပ်ငန်းများထဲမှ ပေါင်း၊ လုပ်ငန်းများထဲမှ နုတ်] ဖြစ်ပါစေ။
ခိုင်း ၏ အကွက်တိုင်း ကို ခု အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ် သည် ခု ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
"{ခု}: {(ဂရစ် ၏ အကွက် အမှတ်) ဟူသော လုပ်ဆောင်ချက်ကို ၆ နှင့် ၃ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
လုပ်ငန်းများထဲမှ မြှောက်: ၁၈
လုပ်ငန်းများထဲမှ ပေါင်း: ၉
လုပ်ငန်းများထဲမှ နုတ်: ၃
```

Each member of the enumeration converts to its number, `၀`, `၁` or `၂`, and that number
picks the function from `ဂရစ်`. A struct field can also contain a function value, and we
call it in the same form, with the field in parentheses:
`(ခလုတ် ၏ လုပ်) ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်မည်။`

## A function that can be missing

A function value is never without a function, because Zaka has no NULL function. So a
variable of a function kind that we declare without a value must get a function before
its first call, the same as each other variable:

```zaka-error ZK0722
ကိန်းရှင် ဖ သည် ကိန်းပြည့် တစ်ခု ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား တစ်ခု ဖြစ်ပါစေ။
ဖ ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်မည်။
```

```text
error[ZK0722]: "ဖ" can be unset here: a path reaches this read with no value given to it
```

When a function can be missing, we state it in its data kind with an
[optional value](../optionals/README.md) (option type), and the program checks for a
function before the call. We put the function kind in parentheses before
`ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`:

```
(<function kind>) ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်
```

```zaka
ပြ ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ဖ သည် (ကိန်းပြည့် တစ်ခု ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား) ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အလွတ် ဖြစ်ပါစေ။
အကယ်၍ ဖ ထဲတွင် ဒေတာ မရှိလျှင် "မရှိ" ဟု ပြသမည်။
ဖ ကို ပြ ဟူသော လုပ်ဆောင်ချက် ဟု ထားမည်။
အကယ်၍ ဖ ထဲတွင် ဒေတာ ရှိလျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ဖ ထဲမှ ဒေတာ ဖြစ်ပါစေ။
ခ ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
မရှိ
၁
```

## Details

- **No initial value.** A function kind has no [initial
  value](../variables/README.md#the-initial-value) (default value), because no function
  would be a fair choice, so `ကနဦးတန်ဖိုး` for a function kind is the error ZK0479.
- **Duplicates.** A duplicate (copy) of a function value is the same function. We can use
  `ဖ နှင့် ဂ သည် တစ်ခုတည်း ဖြစ်လျှင်` to check whether two values are one function, as for
  [shared objects](../structs/objects.md#equal-or-the-same).
- **Prepositions and mutable parameters.** A function kind can also name the prepositions
  of its parameters, and a mutable parameter (`inout`) with `ဒေတာ ပြောင်းလဲရန်`
  ([Function values](../../manual/language/function-values.md)).
- **Unsafe functions.** An unsafe function cannot be a function value
  ([Owned pointers](../pointers/ownership.md)).

## Recap

- `<name> ဟူသော လုပ်ဆောင်ချက်` is a function value, and we can keep it in a variable and
  call it through the variable.
- A function kind lists the data kinds of the parameters and the result, and it ends with
  `သော လုပ်ဆောင်ချက် အမျိုးအစား`.
- A parameter of a function kind lets the caller pass a function, and we write that
  function kind in parentheses.
- A kind alias gives a long function kind a short name.
- A function value is never without a function, so a function that can be missing is an
  optional value.

## Hands on

1. Write `နှစ်ကြိမ်`, which accepts a function of the function kind
   `ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား`
   and a number, and runs the function twice on the number. With `နှစ်ဆ ဟူသော လုပ်ဆောင်ချက်`
   and `၃`, it returns `၁၂`.
2. Add `ပိုင်း` ("divide") to the list of functions, and a member for it.
3. Sort a list of numbers so that the even numbers come first.

Next: [Closures](closures.md).
