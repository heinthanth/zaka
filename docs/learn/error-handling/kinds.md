---
title: Writing result kinds
description: Write the data kind of a result that can fail, with a good side and an error side, and use it for variables, parameters, fields and function results.
---

# Writing result kinds

So far, the compiler worked out the data kind (type) of a whole result from the function
call. We can also write a result kind (result type) in a variable declaration, a
parameter, a struct field or the result clause of a function.

## You will learn

- how to write a result kind for a variable or a parameter
- how to write a side that has no value, with `အောင်မြင်သည့်သတင်း` and `မအောင်မြင်သည့်သတင်း`
- how to put results in struct fields and lists
- the mistakes around a side with no value

```zaka
စားခွဲ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ ခ သည် ၀ နှင့် ညီလျှင် "သုညဖြင့် စား၍ မရပါ" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
က / ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပြ ဟူသော လုပ်ဆောင်ချက် သည်
R ဟူသော ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ R ထဲတွင် ရလဒ်ကောင်း ရှိလျှင် "ရလဒ် {R ထဲမှ ဒေတာ}" ဟု ပြသမည်။
သို့မဟုတ်ပါက "အမှား: {R ထဲမှ အမှားအယွင်း ဒေတာ}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ရ သည် ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် စားခွဲ(၁၀၀၊ ၂၅) ဖြစ်ပါစေ။
ပြ ဟူသော လုပ်ဆောင်ချက်ကို ရ ဖြင့် လုပ်ဆောင်မည်။
ရ ကို စားခွဲ(၁၀၀၊ ၀) ဟု ထားမည်။
ပြ ဟူသော လုပ်ဆောင်ချက်ကို ရ ဖြင့် လုပ်ဆောင်မည်။
```

```text
ရလဒ် ၄
အမှား: သုညဖြင့် စား၍ မရပါ
```

Here the variable `ရ` and the parameter `R` both have a written result kind, which is the
same one that `စားခွဲ` uses in its result clause:

```text
<good kind> သို့မဟုတ် အမှားအယွင်းအနေဖြင့် <error kind> ရှိနိုင်သော ဒေတာဆဲလ်
```

| Zaka | Reads as |
| --- | --- |
| `<good kind>` | the data kind of the good result (`Ok` value) |
| `သို့မဟုတ်` | "or" |
| `အမှားအယွင်းအနေဖြင့် <error kind>` | "as an error value, a `<error kind>`" |
| `ရှိနိုင်သော ဒေတာဆဲလ်` | "a data cell that can contain" |

In a declaration, a parameter or a result clause, we write `တစ်ခု` after the result kind,
as we do after other data kinds. A function can fail when its result clause has a result
kind, and it cannot fail otherwise.

## No initial value

A result has no [initial value](../variables/README.md#the-initial-value), because Zaka
never picks a success or a failure for the program. So a result variable gets its value
from a call or a return, such as `ကိန်းရှင် ရ သည် စားခွဲ(၁၀၀၊ ၂၅) ဖြစ်ပါစေ။`, before its
first read, and `ကနဦးတန်ဖိုး` for a result kind is a mistake:

```zaka-error ZK0479
ကိန်းရှင် R သည် ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
R ကို ပြသမည်။
```

```text
error[ZK0479]: "isize result or text error" has no initial value: a result must hold its good value or its failure
```

A struct or a list that contains a result has no initial value either, so we write out a
value for each of its fields and cells.

## A side with no value

```text
<good kind> သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ်
အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် <error kind> ရှိနိုင်သော ဒေတာဆဲလ်
```

Sometimes a side has nothing to tell. When a failure has no value, we write
`မအောင်မြင်သည့်သတင်း` ("news of failure") as the whole error side, without
`အမှားအယွင်းအနေဖြင့်`. When a success has no value, we write `အောင်မြင်သည့်သတင်း` ("news of
success") as the good side. Each tells which side the call met, and nothing more:

| Zaka | The good side | The error side |
| --- | --- | --- |
| `ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ်` | an integer | text |
| `အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ်` | no value | text |
| `ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ်` | an integer | no value |
| `အောင်မြင်သည့်သတင်း သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ်` | no value | no value |

To end a function with a side that has no value, we use one of these sentences, which
return at once, as `ပြန်ပေးမည်` does:

```text
လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။
လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်။
```

| Zaka | Reads as |
| --- | --- |
| `လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်` | "will mark the function as not succeeded" |
| `လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်` | "will mark the function as succeeded" |

When we print a whole result with such a side, the output shows the news word alone, with
no value in brackets:

```zaka
ရှာ ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ အောက် ငယ်လျှင် လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။
န * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ရေး ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "ရေး၍ မရ" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
"ရေးပြီး {န}" ဟု ပြသမည်။
လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ရှာ(၄) ကို ပြသမည်။
ရှာ(၀ - ၁) ကို ပြသမည်။
ရေး(၁) ကို ပြသမည်။
ရေး(၀) ကို ပြသမည်။
ကိန်းရှင် R သည် ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ရှာ(၀ - ၂) ဖြစ်ပါစေ။
အကယ်၍ R ထဲတွင် အမှားအယွင်း ရှိလျှင် "ရှာ၍ မတွေ့" ဟု ပြသမည်။
```

```text
ရလဒ်ကောင်း(၈)
မအောင်မြင်သည့်သတင်း
ရေးပြီး ၁
အောင်မြင်သည့်သတင်း
အမှားအယွင်း("ရေး၍ မရ")
ရှာ၍ မတွေ့
```

The test `R ထဲတွင် အမှားအယွင်း ရှိလျှင်` is true for a failure with no value too. What we
cannot do is read a side with no value, because there is nothing to read, as
[Mistakes](#mistakes) shows.

We pass a failure on as before. A failure with no value is a separate error kind, so a
function that fails with no value can pass its failure on to a caller that also fails with
no value, and a caller that fails with text is the mistake ZK0446. Reading the good result
of a result that contains a failure stops the program:

```zaka-stop
ရှာ ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ အောက် ငယ်လျှင် လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။
န * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
"စစ် {န}" ဟု ပြသမည်။
ရှာ(န)? + ရှာ(န)? ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

နှစ်ဆ(၅) ကို ပြသမည်။
ကိန်းရှင် ရ သည် နှစ်ဆ(၀ - ၁) ဖြစ်ပါစေ။
ရ ကို ပြသမည်။
"မစမီ" ဟု ပြသမည်။
ရ ထဲမှ ဒေတာ ကို ပြသမည်။
"ဤနေရာ မရောက်ပါ" ဟု ပြသမည်။
```

```text
စစ် ၅
ရလဒ်ကောင်း(၂၀)
စစ် -၁
မအောင်မြင်သည့်သတင်း
မစမီ
the program stopped: the result holds a failure, not a good result
```

When the failure has an error value, the message ends with that value, as on [Reading a
whole result](results.md#reading-the-wrong-part).

## Struct fields

A field of a struct can have a result kind. Here the good side and the error side are both
structs, and the field contains one of them:

```zaka
ဖိုင်ရလဒ် ဟူသော ဒေတာအစုအဝေးတွင် အရွယ် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။
ဖိုင်အမှားအယွင်း ဟူသော ဒေတာအစုအဝေးတွင် စာ ဟူသော စာသား တစ်ခု ပါဝင်သည်။
ဖတ်ချက် ဟူသော ဒေတာအစုအဝေးတွင်
တွေ့ချက် ဟူသော ဖိုင်ရလဒ်
သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု နှင့်
အကြိမ် ဟူသော ကိန်းပြည့် တစ်ခု
တို့ပါဝင်သည်။

ဖွင့် ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
ဖိုင်ရလဒ် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် ဖိုင်အမှားအယွင်း(စာ: "မရှိ") ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
ဖိုင်ရလဒ်(အရွယ်: န * ၁၀၀) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ဖ သည် ဖတ်ချက်(တွေ့ချက်: ဖွင့်(၃)၊ အကြိမ်: ၁) ဖြစ်ပါစေ။
ဖ ၏ တွေ့ချက် ကို ပြသမည်။
ဖ ၏ တွေ့ချက် ကို ဖွင့်(၀) ဟု ထားမည်။
ဖ ၏ တွေ့ချက် ကို ပြသမည်။
```

```text
ရလဒ်ကောင်း(ဖိုင်ရလဒ်(အရွယ်: ၃၀၀))
အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: "မရှိ"))
```

We write a struct on either side the same as other data kinds: `ဖိုင်ရလဒ်` is the good side
and `အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း` is the error side. The function `ဖွင့်` returns a
struct as its good result with a plain return, and a struct as its error value with
`ဟူသော အမှားအယွင်း`.

## Lists of results

```text
<good kind> သို့မဟုတ် အမှားအယွင်းအနေဖြင့် <error kind> ရှိနိုင်သော ဒေတာဆဲလ် စာရင်း
```

To make a list of results, we write `စာရင်း` after the whole result kind, so each cell of the
list contains a whole result. For a counted list, we put the count, such as
`အကွက် ၂ ခုရှိသော`, before the result kind:

| Zaka | Means |
| --- | --- |
| `ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် စာရင်း` | a list of results |
| `အကွက် ၂ ခုရှိသော ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် စာရင်း` | a list of 2 results |

We can use these list kinds for a variable, a parameter or a field of a struct:

```zaka
ဖိုင်အမှားအယွင်း ဟူသော ဒေတာအစုအဝေးတွင် စာ ဟူသော စာသား တစ်ခု ပါဝင်သည်။
ဖတ်ချက် ဟူသော ဒေတာအစုအဝေးတွင်
တွေ့ချက်များ ဟူသော ကိန်းပြည့်
သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် စာရင်း တစ်ခု နှင့်
အကြိမ် ဟူသော ကိန်းပြည့် တစ်ခု
တို့ပါဝင်သည်။

ရှာ ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ အောက် ငယ်လျှင် ဖိုင်အမှားအယွင်း(စာ: "အနုတ်") ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

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
```

```text
[ရလဒ်ကောင်း(၆)၊ အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: "အနုတ်"))၊ ရလဒ်ကောင်း(၈)]
အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: "အနုတ်"))
[ရလဒ်ကောင်း(၂)၊ အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: "အနုတ်"))]
[ရလဒ်ကောင်း(၁၈)]
```

A list prints its cells in brackets, each cell as a whole result. Because a result has no
initial value, we write out each cell of a counted list, as `နှစ်ကွက်` does, and the cells
that we add with `ထည့်မည်` contain the results we added. We visit the cells with
`အကွက်တိုင်း`, as for other lists, and use the test `ရ ထဲတွင် အမှားအယွင်း ရှိလျှင်` from
[Reading a whole result](results.md) to find which part each cell contains.

The place of `စာရင်း` matters. After `ရှိနိုင်သော ဒေတာဆဲလ်`, it makes a list of whole results,
as above. Before `ရှိနိုင်သော ဒေတာဆဲလ်`, as in `အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း စာရင်း`,
it makes the error value a list, so a single `ဖိုင်အမှားအယွင်း` does not fit it:

```zaka-error ZK0401
ဖိုင်အမှားအယွင်း ဟူသော ဒေတာအစုအဝေးတွင် စာ ဟူသော စာသား တစ်ခု ပါဝင်သည်။

ရှာ ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း စာရင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ အောက် ငယ်လျှင် ဖိုင်အမှားအယွင်း(စာ: "အနုတ်") ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ရှာ(၃) ကို ပြသမည်။
```

```text
error[ZK0401]: expected ဖိုင်အမှားအယွင်း[], found ဖိုင်အမှားအယွင်း
```

## Mistakes

A side with no value has nothing to read, so reading the error value of a failure with no
value is the mistake ZK0450:

```zaka-error ZK0450
ရှာ ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ အောက် ငယ်လျှင် လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။
န * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ရ သည် ရှာ(၁) ဖြစ်ပါစေ။
ရ ထဲမှ အမှားအယွင်း ဒေတာ ကို ပြသမည်။
```

```text
error[ZK0450]: the error side of the result is "မအောင်မြင်သည့်သတင်း": it holds no value to read
```

A function must return what its result kind names. When the error side names a data kind,
each failure must return an error value of that data kind, so a failure with no value is
the mistake ZK0451:

```zaka-error ZK0451
ရေး ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
error[ZK0451]: the function fails with an error value of the kind "text", so a failure must give one back
```

The reverse is a mistake too: when the error side has no value, an error value has no place
to go (ZK0476):

```zaka-error ZK0476
ရေး ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
"ရေး၍ မရ" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
error[ZK0476]: an error value goes where a result kind with an error side is wanted
```

## Details

- `လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်။` returns a good result with no value, the same as
  a bare `ပြန်ပေးမည်။`.
- When the good side is `အောင်မြင်သည့်သတင်း`, `ရ ထဲမှ ဒေတာ` has nothing to read and is ZK0449.
- In a message, a result kind is written in English, such as `isize result or text error`,
  `success or text error` and `isize result or failure`.
- For a pointer to a result, we write `လိပ်စာ` after the whole result kind, as we write
  `စာရင်း` for a list, and Part III covers pointers. The full rules and the other mistakes are in [Result
  kinds](../../manual/language/result-kinds.md).

## Recap

- We write a result kind as `<good kind> သို့မဟုတ် အမှားအယွင်းအနေဖြင့် <error kind> ရှိနိုင်သော ဒေတာဆဲလ်`,
  and we can use it for a variable, a parameter, a field or a result clause.
- A result has no initial value, so each result variable, field and cell gets a value from
  a call or a return.
- `မအောင်မြင်သည့်သတင်း` is an error side with no value, and `အောင်မြင်သည့်သတင်း` is a good side
  with no value.
- We write `စာရင်း` after the whole result kind to make a list of results.

## Hands on

- Write a function that looks a number up in a list and fails with no value when the number
  is not there. Print the whole result for a number that is there and one that is not.
- Use a whole result as a parameter, as `ပြ` does above, and print the good result or the
  error value with a test in `အကယ်၍`.
- Make a struct with a field of a result kind, and fill the field with the call of a
  function that can fail.
- Keep the results of three calls in a list of results, and print each cell with a test in
  `အကယ်၍`.

Next: [Optional values](../optionals/README.md).
