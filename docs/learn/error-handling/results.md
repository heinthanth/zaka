---
title: Reading a whole result
description: Keep the good result or the error value of a call, test which one it holds, and read each part.
---

# Reading a whole result

A call that can fail, written with no `ရလဒ်ကောင်း` and no `?`, passes no failure on.
It gives back the *whole result*: the good result or the error value, whichever the call
met. The first store of such a value is a declaration, and the kind here is taken from the
value, as the kind of a variable is when it is written with no kind (the next page writes
the kind out):

```zaka
လုပ်ဆောင်ချက် စားခွဲ သည်
ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
အကယ်၍ ခ သည် ၀ နှင့် ညီလျှင် "သုညဖြင့် စား၍ မရပါ" ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
က / ခ ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် စားခွဲ(၁၀၀၊ ၂၅) ဖြစ်ပါစေ။
အကယ်၍ ရ သည် ရလဒ်ကောင်း ဖြစ်လျှင် "ရလဒ် {ရ ကို ရလဒ်ကောင်း အဖြစ် ပြောင်းလဲသော ရလဒ်}" ဟု ပြသပါ။
သို့မဟုတ်ပါက "မအောင်မြင်" ဟု ပြသပါ။
ရ ကို စားခွဲ(၁၀၀၊ ၀) ဟု ထားပါ။
အကယ်၍ ရ သည် ရလဒ်ကောင်း ဖြစ်လျှင် "ရလဒ်" ဟု ပြသပါ။
သို့မဟုတ်ပါက "အမှား: {ရ ကို အမှားအယွင်း အဖြစ် ပြောင်းလဲသော ရလဒ်}" ဟု ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရလဒ် ၄
အမှား: သုညဖြင့် စား၍ မရပါ
```

The second line of the start block stores another result of the same kinds with `ထားပါ`.
Two tests and two reads are new:

| Written | Is |
| --- | --- |
| `ရ သည် ရလဒ်ကောင်း` | true when `ရ` holds a good result |
| `ရ သည် အမှားအယွင်း` | true when `ရ` holds an error value |
| `ရ ကို ရလဒ်ကောင်း အဖြစ် ပြောင်းလဲသော ရလဒ်` | the good result |
| `ရ ကို အမှားအယွင်း အဖြစ် ပြောင်းလဲသော ရလဒ်` | the error value |

The tests are true or false like `ညီ`, so each fits in `အကယ်၍`, in a loop condition, and
beside `&&` and `||`. A read uses the conversion sentence of
[Conversions](../../manual/language/conversions.md), with `ရလဒ်ကောင်း` or `အမှားအယွင်း` in the
place of a kind.

A read of the part the result does not hold stops the run, as a conversion that cannot be
made does. Test first, as the program above does, or be sure of the part:

```zaka-stop
လုပ်ဆောင်ချက် စားခွဲ သည်
ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
အကယ်၍ ခ သည် ၀ နှင့် ညီလျှင် "သုညဖြင့် စား၍ မရပါ" ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
က / ခ ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် စားခွဲ(၁၀၀၊ ၀) ဖြစ်ပါစေ။
"မစမီ" ဟု ပြသပါ။
ရ ကို ရလဒ်ကောင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
"ဤနေရာ မရောက်ပါ" ဟု ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မစမီ
the program stopped: the result holds an error, not a good result: သုညဖြင့် စား၍ မရပါ
```

When a test tells the part, the compiler does not wait for the run. Inside the
arm of `ရ သည် ရလဒ်ကောင်း`, `ရ` holds a good result, so a read of its error value there is
ZK0454. The arms after it get the opposite, and an arm that ends with `ပြန်ပေးပါ`,
`ရပ်ပါ` or `ကျော်ပါ` proves the opposite for what follows the whole statement. A change to
`ရ`, such as `ထားပါ`, ends the proof:

```zaka-error ZK0454
လုပ်ဆောင်ချက် စားခွဲ သည်
ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
အကယ်၍ ခ သည် ၀ နှင့် ညီလျှင် "သုညဖြင့် စား၍ မရပါ" ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
က / ခ ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် စားခွဲ(၁၀၀၊ ၂၅) ဖြစ်ပါစေ။
အကယ်၍ ရ သည် ရလဒ်ကောင်း ဖြစ်လျှင် ရ ကို အမှားအယွင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

Where the compiler cannot tell, the read is checked in the run, as above. The reverse, a read
of the error value of a good result, stops the run with "the
result is a good result, not an error". A function with no result has no good result to
read, so a read of `ရလဒ်ကောင်း` from its call is a compile error. The full rules are in
[Errors and results](../../manual/language/error-handling.md#reading-a-whole-result).

## Hands on

- Write a function that gives back twice a number as a good result, and fails with text
  for a number below zero. Pass its failure on from a second function, and test the result of
  that function in the start block.
- Make the error value a struct with a code and a message, and print it from the trap.
- Store the result of a call in a variable, and print the good result or the error value
  with a test in `အကယ်၍` and `သို့မဟုတ်ပါက`.

Next: [Writing result kinds](kinds.md).
