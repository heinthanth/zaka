---
title: Parameters
description: Pass values into a function.
---

# Parameters

A function becomes far more useful when you can give it values to work with. The values
a function accepts are its *parameters*. The values you pass when calling it are the
*arguments*.

```zaka
// "Function count-up, accepting an integer amount, will do the following"
လုပ်ဆောင်ချက် ရေတွက်ပြ သည်
ကိန်းပြည့် အရေအတွက် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ မှ အရေအတွက် အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

// "Run the function count-up with 3"
လုပ်ဆောင်ချက် ရေတွက်ပြ ကို ၃ တို့ဖြင့် လုပ်ဆောင်ပါ။
"---" ဟု ပြသပါ။
လုပ်ဆောင်ချက် ရေတွက်ပြ ကို ၂ တို့ဖြင့် လုပ်ဆောင်ပါ။
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
| `ကိန်းပြည့် အရေအတွက် ကို လက်ခံပြီး` | "accepting an integer amount, and ..." |
| `လက်ခံပြီး` | "having accepted" |
| `၃ တို့ဖြင့်` | "with 3" |

## Several parameters

List parameters with `၊`, the Burmese comma. Each has its kind before its name. Close a
list of two or more with `တို့ကို`, and pass the arguments in the same order:

```zaka
// rectangle: print the area of a length × width rectangle
လုပ်ဆောင်ချက် ဧရိယာပြ သည်
ကိန်းပြည့် အလျား၊ ကိန်းပြည့် အနံ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အလျား * အနံ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

လုပ်ဆောင်ချက် ဧရိယာပြ ကို ၃၊ ၄ တို့ဖြင့် လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် ဧရိယာပြ ကို ၁၀၊ ၂ + ၃ တို့ဖြင့် လုပ်ဆောင်ပါ။     // arguments can be sums
```

```text
၁၂
၅၀
```

As in Burmese, `နှင့်` ("and") may replace the last `၊`: `ကိန်းပြည့် က၊ ကိန်းပြည့် ခ နှင့်
ကိန်းပြည့် ဂ တို့ကို လက်ခံပြီး`.

| Number of parameters | Closer | Example |
| --- | --- | --- |
| one | `ကို` | `ကိန်းပြည့် က ကို လက်ခံပြီး` |
| two or more | `တို့ကို` | `ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး` |

Arguments always close with `တို့ဖြင့်` ("with these"), even when there is only one.

## Parameters are local copies

A parameter is a new variable inside the function. Changing it does not change the
variable that was passed in:

```zaka
လုပ်ဆောင်ချက် တိုးပြ သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို က + ၁ ဟု ထားပါ။
က ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် ခ သည် ကိန်းပြည့် ၅ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် တိုးပြ ကို ခ တို့ဖြင့် လုပ်ဆောင်ပါ။     // prints 6
ခ ကို ပြသပါ။                                    // still 5
```

```text
၆
၅
```

## Matching the call to the function

The call must pass exactly as many arguments as the function has parameters, and each
must fit its parameter's kind:

```zaka-error ZK0314
လုပ်ဆောင်ချက် ပြ သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်ချက် ပြ ကို ၁၊ ၂ တို့ဖြင့် လုပ်ဆောင်ပါ။
```

```text
error[ZK0314]: function "ပြ" takes 1 arguments, 2 given
```

## Try it

- Write a function that takes a number and prints its times table from 1 to 10.
- Give `ရေတွက်ပြ` a guard: `အကယ်၍ အရေအတွက် သည် ၁ အောက် ငယ်လျှင် ပြန်ပေးပါ။`, then call it
  with `၀`.

Next: [Results](results.md).
