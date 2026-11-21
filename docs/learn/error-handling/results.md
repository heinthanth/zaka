---
title: Reading a whole result
description: Keep the good result or the error value of a call, test which one it contains, and read each part.
---

# Reading a whole result

When we call a function that can fail without `?` or `ရလဒ်ကောင်း`, the call passes nothing
on and returns the *whole result*, which is either the good result (`Ok` value) or the error
value (error). We can keep the whole result in a variable, test which part it contains, and
then read that part.

## You will learn

- how to keep a whole result in a variable
- how to test whether it contains a good result or an error value
- how to read each part
- what happens when we read the part that is not there

```zaka
စားခွဲ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ ခ သည် ၀ နှင့် ညီလျှင် "သုညဖြင့် စား၍ မရပါ" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
က / ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ရ သည် စားခွဲ(၁၀၀၊ ၂၅) ဖြစ်ပါစေ။
အကယ်၍ ရ ထဲတွင် ရလဒ်ကောင်း ရှိလျှင် "ရလဒ် {ရ ထဲမှ ဒေတာ}" ဟု ပြသမည်။
သို့မဟုတ်ပါက "မအောင်မြင်" ဟု ပြသမည်။
ရ ကို စားခွဲ(၁၀၀၊ ၀) ဟု ထားမည်။
အကယ်၍ ရ ထဲတွင် ရလဒ်ကောင်း ရှိလျှင် "ရလဒ်" ဟု ပြသမည်။
သို့မဟုတ်ပါက "အမှား: {ရ ထဲမှ အမှားအယွင်း ဒေတာ}" ဟု ပြသမည်။
```

```text
ရလဒ် ၄
အမှား: သုညဖြင့် စား၍ မရပါ
```

The first line after the function keeps the whole result in `ရ`, and `ရ` gets its
data kind (type) from the call, as a variable declared without a data kind does. Later,
`ထားမည်` puts another result of the same data kind into `ရ`. To test and read a whole
result, we use these forms:

```text
<result> ထဲတွင် ရလဒ်ကောင်း ရှိလျှင်
<result> ထဲတွင် အမှားအယွင်း ရှိလျှင်
<result> ထဲမှ ဒေတာ
<result> ထဲမှ အမှားအယွင်း ဒေတာ
```

| Zaka | Reads as | What it does |
| --- | --- | --- |
| `ရ ထဲတွင် ရလဒ်ကောင်း ရှိလျှင်` | "if there is a good result in `ရ`" | true when `ရ` contains a good result |
| `ရ ထဲတွင် အမှားအယွင်း ရှိလျှင်` | "if there is an error value in `ရ`" | true when `ရ` contains an error value |
| `ရ ထဲမှ ဒေတာ` | "the data from `ရ`" | the good result |
| `ရ ထဲမှ အမှားအယွင်း ဒေတာ` | "the error data from `ရ`" | the error value |

The tests are true/false values (booleans), so we can use them with `အကယ်၍`, in a loop
condition such as `ရ ထဲတွင် ရလဒ်ကောင်း ရှိနေသရွေ့`, and with `&&` and `||`. We can also write
`ရှိနေလျှင်` in place of `ရှိလျှင်`, with the same meaning.

## Reading the wrong part

When we read the part that the result does not contain, the program stops, as it does for a
conversion that cannot be made. So we test the result first, as the program above does, or
we make sure which part it contains:

```zaka-stop
စားခွဲ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ ခ သည် ၀ နှင့် ညီလျှင် "သုညဖြင့် စား၍ မရပါ" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
က / ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ရ သည် စားခွဲ(၁၀၀၊ ၀) ဖြစ်ပါစေ။
"မစမီ" ဟု ပြသမည်။
ရ ထဲမှ ဒေတာ ကို ပြသမည်။
"ဤနေရာ မရောက်ပါ" ဟု ပြသမည်။
```

```text
မစမီ
the program stopped: the result holds an error, not a good result: သုညဖြင့် စား၍ မရပါ
```

The reverse, a read of the error value of a good result, stops the program too, with the
message "the result is a good result, not an error".

## A read that a test rules out

When a test has told the compiler which part a result contains, the compiler does not wait
for the run. Inside the arm of `ရ ထဲတွင် ရလဒ်ကောင်း ရှိလျှင်`, `ရ` contains a good result,
so a read of its error value there is the mistake ZK0454:

```zaka-error ZK0454
စားခွဲ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ ခ သည် ၀ နှင့် ညီလျှင် "သုညဖြင့် စား၍ မရပါ" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
က / ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ရ သည် စားခွဲ(၁၀၀၊ ၂၅) ဖြစ်ပါစေ။
အကယ်၍ ရ ထဲတွင် ရလဒ်ကောင်း ရှိလျှင် ရ ထဲမှ အမှားအယွင်း ဒေတာ ကို ပြသမည်။
```

The arms after it (`သို့မဟုတ်ပါက`) know the opposite. When an arm ends with `ပြန်ပေးမည်`,
`ရပ်မည်` or `ကျော်မည်`, the sentences after the whole `အကယ်၍` know the opposite too. A change
to `ရ`, such as `ထားမည်`, ends what the compiler knows. Where the compiler cannot tell, the
program checks the read when it runs, as in the previous section.

## Details

- We can give a fallback for the error case: `ရ ထဲမှ ဒေတာ၊ မရှိလျှင် ၀` is the good result,
  or `၀` when `ရ` contains an error value ([A fallback](../../manual/language/optionals.md#a-fallback)).
- A whole result is not a number, so using it as one, such as `ရ + ၁`, is ZK0403.
- When the good result of a function is `အောင်မြင်သည့်သတင်း`, the function has no good value,
  so `ရ ထဲမှ ဒေတာ` on its result is ZK0449 ([Writing result kinds](kinds.md)).
- A `ဒေတာ ပြောင်းလဲရန်` (mutable parameter) argument also ends what the compiler knows about a
  result, the same as `ထားမည်` does.
- The full rules are in [Reading a
  result](../../manual/language/reading-results.md).

## Recap

- A call without `?` or `ရလဒ်ကောင်း` returns the whole result, which we can keep in a
  variable.
- We test it with `ထဲတွင် ရလဒ်ကောင်း ရှိလျှင်` or `ထဲတွင် အမှားအယွင်း ရှိလျှင်`.
- We read the good result with `ထဲမှ ဒေတာ` and the error value with `ထဲမှ အမှားအယွင်း ဒေတာ`.
- Reading the part that is not there stops the program, or is a compile error when a test
  rules it out.

## Hands on

- Write a function that returns twice a number as a good result, and fails with text for a
  number below zero. Pass its failure on from a second function, and test the result of
  that function in the program start block.
- Make the error value a struct with a code and a message, and print both fields when the
  test finds an error value.
- Keep the result of a call in a variable, and print the good result or the error value
  with a test in `အကယ်၍` and `သို့မဟုတ်ပါက`.

Next: [Writing result kinds](kinds.md).
