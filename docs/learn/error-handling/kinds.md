---
title: Writing result kinds
description: Write the kind of a result that can fail, with a good side and an error side, and use it for variables, parameters, fields and function results.
---

# Writing result kinds

So far a variable that held the result of a call took its kind from the call. A result
kind can be written out too, wherever a kind goes: in a declaration, in a parameter, in the
field of a struct and in the result clause of a function. A function that can fail has
named its whole result kind in its header from the start.

```text
<kind> သို့မဟုတ် အမှားအယွင်းအနေဖြင့် <kind> ရှိနိုင်သော ဒေတာဆဲလ်
```

reads as "a cell that can hold a value of the first kind, or an error value of the second kind". Each side can also hold no value at all:

| Written | The good side | The error side |
| --- | --- | --- |
| `ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ်` | an integer | text |
| `အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ်` | no value | text |
| `ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ်` | an integer | no value |
| `အောင်မြင်သည့်သတင်း သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ်` | no value | no value |

`အောင်မြင်သည့်သတင်း` is "news of success" and `မအောင်မြင်သည့်သတင်း` is "news of failure":
each tells which side was met, and carries nothing more. A side with no error value drops
`အမှားအယွင်းအနေဖြင့်` as well, so the error side reads `မအောင်မြင်သည့်သတင်း` alone.

## In a clause, a parameter and a variable

The result clause of a function is the written kind, then `တစ်ခု ကို ပြန်ပေးမည်`. The function can
fail when its result kind is a result kind, and not otherwise:

```zaka
လုပ်ဆောင်ချက် စားခွဲ သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ ခ သည် ၀ နှင့် ညီလျှင် "သုညဖြင့် စား၍ မရပါ" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
က / ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပြ သည်
R ဟူသော ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ R ထဲတွင် ရလဒ်ကောင်း ရှိလျှင် "ရလဒ် {R ထဲမှ ဒေတာ}" ဟု ပြသမည်။
သို့မဟုတ်ပါက "အမှား: {R ထဲမှ အမှားအယွင်း ဒေတာ}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် စားခွဲ(၁၀၀၊ ၂၅) ဖြစ်ပါစေ။
ပြ ဟူသော လုပ်ဆောင်ချက်ကို ရ ဖြင့် လုပ်ဆောင်မည်။
ရ ကို စားခွဲ(၁၀၀၊ ၀) ဟု ထားမည်။
ပြ ဟူသော လုပ်ဆောင်ချက်ကို ရ ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရလဒ် ၄
အမှား: သုညဖြင့် စား၍ မရပါ
```

The result kind with `တစ်ခု ကို ပြန်ပေးမည်` after it is the header of a function that can
fail. A function that fails and has no good result writes the side with no value as its good
side: `အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်`.

A result has no [initial value](../variables/README.md#the-initial-value): Zaka never
picks a success or a failure for the program. A result variable gets its value from a
call or a return, such as `ကိန်းရှင် ရ သည် စားခွဲ(၁၀၀၊ ၂၅) ဖြစ်ပါစေ။`, before its first
read, and `ကနဦးတန်ဖိုး` for a result kind is a mistake:

```zaka-error ZK0479
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် R သည် ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
R ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0479]: "isize result or text error" has no initial value: a result must hold its good value or its failure
```

The same goes for a struct or a list that holds a result: it has no initial value either,
so each of its fields and cells gets a value written out.

## A side with no value

When a failure has nothing to tell, write `မအောင်မြင်သည့်သတင်း` for the error side. The
function ends with a failure through its own sentence, which is a return as
`ပြန်ပေးမည်` is:

```text
လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။
```

reads as "mark the function as not succeeded". The good side with no value is the same
for success: `လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်။` ends the function with a good
result that holds nothing, as a bare `ပြန်ပေးမည်` does. Printing a whole result of
such a side shows the news, with no value in brackets:

```zaka
လုပ်ဆောင်ချက် ရှာ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ အောက် ငယ်လျှင် လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။
န * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ရေး သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "ရေး၍ မရ" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
"ရေးပြီး {န}" ဟု ပြသမည်။
လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ရှာ(၄) ကို ပြသမည်။
ရှာ(၀ - ၁) ကို ပြသမည်။
ရေး(၁) ကို ပြသမည်။
ရေး(၀) ကို ပြသမည်။
ကိန်းရှင် R သည် ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ရှာ(၀ - ၂) ဖြစ်ပါစေ။
အကယ်၍ R ထဲတွင် အမှားအယွင်း ရှိလျှင် "ရှာ၍ မတွေ့" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရလဒ်ကောင်း(၈)
မအောင်မြင်သည့်သတင်း
ရေးပြီး ၁
အောင်မြင်သည့်သတင်း
အမှားအယွင်း("ရေး၍ မရ")
ရှာ၍ မတွေ့
```

The test `R ထဲတွင် အမှားအယွင်း ရှိလျှင်` is true for a failure with no value too. What a side with no
value cannot do is be read: there is no value to read. `ရ ထဲမှ အမှားအယွင်း ဒေတာ`
on such a side is an error, shown in [Mistakes](#mistakes).

Passing on works as before. A failure with no value counts as an error side of its own:
a function that fails with no value passes its failure on to a caller that fails with
no value, and a caller that fails with text is the mistake ZK0446. The start block cannot
fail, so it keeps the whole result. Reading the good result of one that holds a failure
stops the run:

```zaka-stop
လုပ်ဆောင်ချက် ရှာ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ အောက် ငယ်လျှင် လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။
န * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် နှစ်ဆ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
"စစ် {န}" ဟု ပြသမည်။
ရှာ(န)? + ရှာ(န)? ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
နှစ်ဆ(၅) ကို ပြသမည်။
ကိန်းရှင် ရ သည် နှစ်ဆ(၀ - ၁) ဖြစ်ပါစေ။
ရ ကို ပြသမည်။
"မစမီ" ဟု ပြသမည်။
ရ ထဲမှ ဒေတာ ကို ပြသမည်။
"ဤနေရာ မရောက်ပါ" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
စစ် ၅
ရလဒ်ကောင်း(၂၀)
စစ် -၁
မအောင်မြင်သည့်သတင်း
မစမီ
the program stopped: the result holds a failure, not a good result
```

The message is `the result holds a failure, not a good result`; with an error value to show,
the message ends with the value.

## Struct fields

A field of a struct can hold a result kind. The structs here are the good side and the
error side, and the field holds either of them:

```zaka
ဖိုင်ရလဒ် ဟူသော ဒေတာအစုအဝေးတွင် အရွယ် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။
ဖိုင်အမှားအယွင်း ဟူသော ဒေတာအစုအဝေးတွင် စာ ဟူသော စာသား တစ်ခု ပါဝင်သည်။
ဖတ်ချက် ဟူသော ဒေတာအစုအဝေးတွင် တွေ့ချက် ဟူသော ဖိုင်ရလဒ် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု နှင့် အကြိမ် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

လုပ်ဆောင်ချက် ဖွင့် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ဖိုင်ရလဒ် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် ဖိုင်အမှားအယွင်း(စာ: "မရှိ") ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
ဖိုင်ရလဒ်(အရွယ်: န * ၁၀၀) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် ဖတ်ချက်(တွေ့ချက်: ဖွင့်(၃)၊ အကြိမ်: ၁) ဖြစ်ပါစေ။
ဖ ၏ တွေ့ချက် ကို ပြသမည်။
ဖ ၏ တွေ့ချက် ကို ဖွင့်(၀) ဟု ထားမည်။
ဖ ၏ တွေ့ချက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရလဒ်ကောင်း(ဖိုင်ရလဒ်(အရွယ်: ၃၀၀))
အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: "မရှိ"))
```

A struct on either side is written like each other kind: `ဖိုင်ရလဒ်` is the good side and
`အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း` is the error side. The function above gives back
a struct value for the good side with a plain return, and a struct value for the error side
with `ဟူသော အမှားအယွင်း`.

## Lists of results

A list can be made of results. The word `စာရင်း` goes after the whole result kind, and it
wraps the whole result: each cell of the list holds a result.

| Written | Is |
| --- | --- |
| `ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် စာရင်း` | a list of results |
| `အကွက် ၂ ခုရှိသော ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် စာရင်း` | a list of 2 results |

A count such as `အကွက် ၂ ခုရှိသော` stands before the result kind, and belongs to the list
after it. A result kind with no such word after it is one result, as before. The same
kinds serve a variable, a parameter and a field of a struct:

```zaka
ဖိုင်အမှားအယွင်း ဟူသော ဒေတာအစုအဝေးတွင် စာ ဟူသော စာသား တစ်ခု ပါဝင်သည်။
ဖတ်ချက် ဟူသော ဒေတာအစုအဝေးတွင် တွေ့ချက်များ ဟူသော ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် စာရင်း တစ်ခု နှင့် အကြိမ် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

လုပ်ဆောင်ချက် ရှာ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ အောက် ငယ်လျှင် ဖိုင်အမှားအယွင်း(စာ: "အနုတ်") ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရများ သည် ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [] ဖြစ်ပါစေ။
ရများ ထဲသို့ ရှာ(၃) ကို ထည့်မည်။
ရများ ထဲသို့ ရှာ(၀ - ၁) ကို ထည့်မည်။
ရများ ထဲသို့ ရှာ(၄) ကို ထည့်မည်။
ရများ ကို ပြသမည်။
ရများ ၏ အကွက်တိုင်း ကို ရ အနေဖြင့် ယူ၍ အကယ်၍ ရ ထဲတွင် အမှားအယွင်း ရှိလျှင် ရ ကို ပြသမည်။

ကိန်းရှင် နှစ်ကွက် သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [ရှာ(၁)၊ ရှာ(၀ - ၂)] ဖြစ်ပါစေ။
နှစ်ကွက် ကို ပြသမည်။

ကိန်းရှင် ဖ သည် ဖတ်ချက်(တွေ့ချက်များ: []၊ အကြိမ်: ၀) ဖြစ်ပါစေ။
ဖ ၏ တွေ့ချက်များ ထဲသို့ ရှာ(၉) ကို ထည့်မည်။
ဖ ၏ တွေ့ချက်များ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[ရလဒ်ကောင်း(၆)၊ အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: "အနုတ်"))၊ ရလဒ်ကောင်း(၈)]
အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: "အနုတ်"))
[ရလဒ်ကောင်း(၂)၊ အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: "အနုတ်"))]
[ရလဒ်ကောင်း(၁၈)]
```

A list prints its cells in brackets, each cell as a whole result. A result has no initial
value, so a list with a count gets each cell written out, as `နှစ်ကွက်` does: a cell
never set would hold neither a success nor a failure. The cells added with `ထည့်မည်`
hold what was given. Each cell is read with
`အကွက်တိုင်း` as for each list, and the test `ရ ထဲတွင် အမှားအယွင်း ရှိလျှင်` of
[Reading a whole result](results.md) tells which part a cell holds. Pointers to
results work the same, and are in
[Errors and results](../../manual/language/error-handling.md#result-kinds).

The levels after `ရှိနိုင်သော ဒေတာဆဲလ်` belong to the whole result, so
`ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် စာရင်း` is
a list of results, as the program above uses it. A list as the error kind goes before
`ရှိနိုင်သော ဒေတာဆဲလ်`, as `အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း စာရင်း`, so the error value
of a function with that clause is a list, and a single `ဖိုင်အမှားအယွင်း` does not fit it:

```zaka-error ZK0401
ဖိုင်အမှားအယွင်း ဟူသော ဒေတာအစုအဝေးတွင် စာ ဟူသော စာသား တစ်ခု ပါဝင်သည်။

လုပ်ဆောင်ချက် ရှာ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း စာရင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ အောက် ငယ်လျှင် ဖိုင်အမှားအယွင်း(စာ: "အနုတ်") ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ရှာ(၃) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0401]: expected ဖိုင်အမှားအယွင်း[], found ဖိုင်အမှားအယွင်း
```

## Mistakes

A side with no value has nothing to read, and a function must give back what its kind names.
Reading the error value of a failure with no value is ZK0450:

```zaka-error ZK0450
လုပ်ဆောင်ချက် ရှာ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ အောက် ငယ်လျှင် လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။
န * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် ရှာ(၁) ဖြစ်ပါစေ။
ရ ထဲမှ အမှားအယွင်း ဒေတာ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0450]: the error side of the result is "မအောင်မြင်သည့်သတင်း": it holds no value to read
```

A failure in a function whose error side names a kind has to hold a value of that kind
(ZK0451):

```zaka-error ZK0451
လုပ်ဆောင်ချက် ရေး သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0451]: the function fails with an error value of the kind "text", so a failure must give one back
```

And an error value has no place in a function whose error side holds no value (ZK0476):

```zaka-error ZK0476
လုပ်ဆောင်ချက် ရေး သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
"ရေး၍ မရ" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0476]: an error value goes where a result kind with an error side is wanted
```

The full rules, with the other codes, are in
[Errors and results](../../manual/language/error-handling.md#result-kinds).

## Hands on

- Write a function that looks a number up in a list and fails with no value when it is
  not there; print the whole result for a number that is and one that is not.
- Take a whole result as a parameter, as `ပြ` does above, and print the good result or the
  error value with a test in `အကယ်၍`.
- Make a struct with a field of a result kind, and fill the field with the call of a
  function that can fail.
- Keep the results of three calls in a list of results, and print each cell with a test in
  `အကယ်၍`.

Next: [Optional values](../optionals/README.md).
