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
ကိန်းပြည့် အရေအတွက် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ မှ အရေအတွက် အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// "Run the function count-up with 3"
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ရေတွက်ပြ ကို ၃ တို့ဖြင့် လုပ်ဆောင်ပါ။
"---" ဟု ပြသပါ။
လုပ်ဆောင်ချက် ရေတွက်ပြ ကို ၂ တို့ဖြင့် လုပ်ဆောင်ပါ။
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
| `ကိန်းပြည့် အရေအတွက် ကို လက်ခံပြီး` | "accepting an integer amount, and ..." |
| `လက်ခံပြီး` | "having accepted" |
| `၃ တို့ဖြင့်` | "with 3" |

## Several parameters

List parameters with `၊`, the Burmese comma. Each has its kind before its name. Close a
list of two or more with `တို့ကို`, and pass the arguments in the same order:

```zaka
// rectangle: print the area of a length × width rectangle
လုပ်ဆောင်ချက် ဧရိယာပြ သည်
ကိန်းပြည့် အလျား၊ ကိန်းပြည့် အနံ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အလျား * အနံ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ဧရိယာပြ ကို ၃၊ ၄ တို့ဖြင့် လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် ဧရိယာပြ ကို ၁၀၊ ၂ + ၃ တို့ဖြင့် လုပ်ဆောင်ပါ။     // arguments can be sums
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၂
၅၀
```

As in Burmese, `နှင့်` ("and") can replace the last `၊`: `ကိန်းပြည့် က၊ ကိန်းပြည့် ခ နှင့်
ကိန်းပြည့် ဂ တို့ကို လက်ခံပြီး`.

| Number of parameters | Closer | Example |
| --- | --- | --- |
| one | `ကို` | `ကိန်းပြည့် က ကို လက်ခံပြီး` |
| two or more | `တို့ကို` | `ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး` |

Arguments close with `တို့ဖြင့်` ("with these") in all cases, even when there is just
one.

## Arguments first

A call can also give the arguments first, then name the function, the order a Burmese
sentence puts an object before its verb. `သုံး၍` ("using") closes the arguments, and
`မည်` ("will") ends the call:

```zaka
လုပ်ဆောင်ချက် ဧရိယာပြ သည်
ကိန်းပြည့် အလျား၊ ကိန်းပြည့် အနံ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အလျား * အနံ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
// "Using 3 and 4, (the function) area-show will (run)"
၃၊ ၄ တို့ကို သုံး၍ ဧရိယာပြ မည်။
၆၊ ၇ တို့ကို သုံး၍ လုပ်ဆောင်ချက် ဧရိယာပြ ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၂
၄၂
```

| Zaka | Reads as |
| --- | --- |
| `၃၊ ၄ တို့ကို သုံး၍` | "using 3 and 4": `တို့ကို` after several arguments, `ကို` after one |
| `ဧရိယာပြ မည်` | "area-show will (run)" |
| `လုပ်ဆောင်ချက် ဧရိယာပြ ကို လုပ်ဆောင်မည်` | "will run the function area-show": the long form |

An argument here is a name, a literal, a sum or a sign-form call. A sentence that gives a
value goes in parentheses. The [Results](results.md) page shows the form of this call
that gives back a value.

## Parameters are local copies

A parameter is a new variable inside the function. Changing it does not change the
variable that was passed in:

```zaka
လုပ်ဆောင်ချက် တိုးပြ သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို က + ၁ ဟု ထားပါ။
က ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် တိုးပြ ကို ခ တို့ဖြင့် လုပ်ဆောင်ပါ။     // prints 6
ခ ကို ပြသပါ။                                    // still 5
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
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က + ၁ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ၅ ဖြစ်ပါစေ။
ခ ကို နောက်တစ်(ခ) ဟု ထားပါ။
ခ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၆
```

When a function must change the variable itself, mark the parameter with
`လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သော` ("that the function can change") and the argument with
`(လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်)` ("the function can change it"):

```zaka
လုပ်ဆောင်ချက် တိုး သည်
လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သော ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို က + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ၅ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် တိုး ကို ခ (လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်) တို့ဖြင့် လုပ်ဆောင်ပါ။
ခ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၆
```

Both ends take the mark, so the call itself shows which variable changes. The argument
must be a variable, a field or a cell, and it cannot appear twice in one call. See
[Edit references](../../manual/language/functions.md#edit-references) in the Docs.

## Matching the call to the function

The call must pass the same number of arguments as the function has parameters, and
each must fit its parameter's kind:

```zaka-error ZK0314
လုပ်ဆောင်ချက် ပြ သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ပြ ကို ၁၊ ၂ တို့ဖြင့် လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0314]: function "ပြ" takes 1 arguments, 2 given
```

## Hands on

- Write a function that takes a number and prints its times table from 1 to 10.
- Give `ရေတွက်ပြ` a guard: `အကယ်၍ အရေအတွက် သည် ၁ အောက် ငယ်လျှင် ပြန်ပေးပါ။`, then call it
  with `၀`.

Next: [Results](results.md).
