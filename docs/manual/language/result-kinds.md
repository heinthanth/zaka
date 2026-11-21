---
title: Result kinds
description: How to write the result kind of a function, a variable, a parameter or a field, the sides with no value, lists and pointers of results, and the statements that end a function with a bare success or failure.
---

# Result kinds

A result kind (result type) is the data kind (type) of a data cell that contains either a
good result (`Ok` value) or an error value (error). We write one in the result clause of a
function that can fail, and wherever else a data kind goes. This page covers the forms of
a result kind. [Errors and results](error-handling.md) covers how a function fails and
passes a failure on, [Reading a result](reading-results.md) covers tests and reads, and the
Learn book teaches the forms in [Writing result kinds](../../learn/error-handling/kinds.md).

## Writing a result kind

```text
<good kind> သို့မဟုတ် အမှားအယွင်းအနေဖြင့် <error kind> ရှိနိုင်သော ဒေတာဆဲလ်
```

We can write a result kind wherever a data kind goes: in a declaration, a parameter, a
field of a struct and the result clause of a function. The good side comes first, then
`သို့မဟုတ်` ("or"), then the error side, and `ရှိနိုင်သော ဒေတာဆဲလ်` ("a data cell that can
contain") ends the result kind, because the cell can contain either side. When a side has
no value, we write a word of news in its place:

- For a good side with no value, we write `အောင်မြင်သည့်သတင်း` ("news of success") in place
  of `<good kind>`.
- For an error side with no value, we write `မအောင်မြင်သည့်သတင်း` ("news of failure") in
  place of both `အမှားအယွင်းအနေဖြင့်` and `<error kind>`.

| Written | Good side | Error side | In Rust |
| --- | --- | --- | --- |
| `<T> သို့မဟုတ် အမှားအယွင်းအနေဖြင့် <E> ရှိနိုင်သော ဒေတာဆဲလ်` | a value of `T` | a value of `E` | `Result<T, E>` |
| `အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် <E> ရှိနိုင်သော ဒေတာဆဲလ်` | no value | a value of `E` | `Result<(), E>` |
| `<T> သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ်` | a value of `T` | no value | `Result<T, ()>` |
| `အောင်မြင်သည့်သတင်း သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ်` | no value | no value | `Result<(), ()>` |

A function can fail when its result is a result kind, and at no other time. We write the
result kind of a variable, a parameter or a field in the same form, and `တစ်ခု` follows it
in a declaration, as it follows other data kinds. Messages write the result kinds as
`isize result or text error`, `success or text error` and `isize result or failure`.

A literal gets the data kind of the good side: the value `၁` of a variable of
`၈ ဘစ် ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ်` is an
`၈ ဘစ် ကိန်းပြည့်`, and a literal that does not fit the good kind is ZK0406.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖတ်ရလဒ် သည် ၈ ဘစ် ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
ဖတ်ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရလဒ်ကောင်း(၁)
```

This program uses structs for both sides, a result kind as a field of a struct and as a
parameter, and the sides with no value:

```zaka
ဖိုင်ရလဒ် ဟူသော ဒေတာအစုအဝေးတွင် အရွယ် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။
ဖိုင်အမှားအယွင်း ဟူသော ဒေတာအစုအဝေးတွင် စာ ဟူသော စာသား တစ်ခု ပါဝင်သည်။
ဖတ်ချက် ဟူသော ဒေတာအစုအဝေးတွင် တွေ့ချက် ဟူသော ဖိုင်ရလဒ် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု နှင့် အကြိမ် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

ဖွင့် ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ဖိုင်ရလဒ် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် ဖိုင်အမှားအယွင်း(စာ: "မရှိ") ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
ဖိုင်ရလဒ်(အရွယ်: န * ၁၀၀) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ရှာ ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ အောက် ငယ်လျှင် လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။
န * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ရေး ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "ရေး၍ မရ" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
"ရေးပြီး {န}" ဟု ပြသမည်။
လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပြ ဟူသော လုပ်ဆောင်ချက် သည်
R ဟူသော ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
R ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ဆင့် ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
ရှာ(၀ - ၁)? ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် ဖတ်ချက် တစ်ခု ဖြစ်ပါစေ။
ဖ ၏ တွေ့ချက် ကို ဖွင့်(၃) ဟု ထားမည်။
ဖ ၏ တွေ့ချက် ကို ပြသမည်။
ဖွင့်(၀) ကို ပြသမည်။
ကိန်းရှင် R သည် ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ရှာ(၄) ဖြစ်ပါစေ။
ပြ ဟူသော လုပ်ဆောင်ချက်ကို R ဖြင့် လုပ်ဆောင်မည်။
ပြ ဟူသော လုပ်ဆောင်ချက်ကို ရှာ(၀ - ၂) ဖြင့် လုပ်ဆောင်မည်။
ရေး(၁) ကို ပြသမည်။
ရေး(၀) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရလဒ်ကောင်း(ဖိုင်ရလဒ်(အရွယ်: ၃၀၀))
အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: "မရှိ"))
ရလဒ်ကောင်း(၈)
မအောင်မြင်သည့်သတင်း
ရေးပြီး ၁
အောင်မြင်သည့်သတင်း
အမှားအယွင်း("ရေး၍ မရ")
```

## Lists and pointers of results

```text
<result kind> စာရင်း
အကွက် <n> ခုရှိသော <result kind> စာရင်း
<result kind> လိပ်စာ
```

We can add list and pointer levels after `ဒေတာဆဲလ်`, and these levels wrap the *whole*
result, not the error side:

| Written | Meaning |
| --- | --- |
| `ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် စာရင်း` | a plain list of results |
| `အကွက် ၂ ခုရှိသော ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် စာရင်း` | a list of 2 results |
| `ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် လိပ်စာ` | a pointer to a result, for [unsafe code](unsafe.md) |

A count prefix (`အကွက် ၂ ခုရှိသော`) that we write before a result kind belongs to its
levels. We can use these data kinds for a variable, a parameter, a field of a struct and
the cells of a heap block taken with `မင်မိုရီတွင်`.

A result has no initial value, so a list of results with a count needs a literal that
gives each cell, and a cell added with `ထည့်မည်` contains the result we give it. A list
prints its cells in brackets, each as a whole result. We read a cell with `အကွက်တိုင်း` or
a range, as in other lists, then test it and read it as [Reading a
result](reading-results.md) states.

```zaka
ဖိုင်အမှားအယွင်း ဟူသော ဒေတာအစုအဝေးတွင် စာ ဟူသော စာသား တစ်ခု ပါဝင်သည်။
ဖတ်ချက် ဟူသော ဒေတာအစုအဝေးတွင် တွေ့ချက်များ ဟူသော ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် စာရင်း တစ်ခု နှင့် အကြိမ် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

ရှာ ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ အောက် ငယ်လျှင် ဖိုင်အမှားအယွင်း(စာ: "အနုတ်") ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပြ ဟူသော လုပ်ဆောင်ချက် သည်
အပိုင်း ဟူသော ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အပိုင်း ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ဖတ် ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည်
ပ ဟူသော ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် လိပ်စာ တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရများ သည် ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [] ဖြစ်ပါစေ။
ရများ ထဲသို့ ရှာ(၃) ကို ထည့်မည်။
ရများ ထဲသို့ ရှာ(၀ - ၁) ကို ထည့်မည်။
ရများ ထဲသို့ ရှာ(၄) ကို ထည့်မည်။
ရများ ကို ပြသမည်။
ရများ ၏ အကွက်တိုင်း ကို ရ အနေဖြင့် ယူ၍ ရ ကို ပြသမည်။
ပြ ဟူသော လုပ်ဆောင်ချက်ကို ရများ ဖြင့် လုပ်ဆောင်မည်။
ကိန်းရှင် အလယ် သည် ရများ ၏ အကွက် ၁ မှစ၍ ၂ ကွက် ဖြစ်ပါစေ။
အလယ် ကို ပြသမည်။

ကိန်းရှင် နှစ်ကွက် သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [ရှာ(၁)၊ ရှာ(၀ - ၁)] ဖြစ်ပါစေ။
နှစ်ကွက် ကို ပြသမည်။

အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ဘူး သည် ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ဘူး တွင် သိမ်းထားမည်။
ဘူး ၏ တန်ဖိုး ကို ရှာ(၀ - ၅) ဟု ထားမည်။
ဖတ် ဟူသော လုပ်ဆောင်ချက်ကို ဘူး ဖြင့် လုပ်ဆောင်မည်။
လိပ်စာ ဘူး ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ဖ သည် ဖတ်ချက်(တွေ့ချက်များ: []၊ အကြိမ်: ၀) ဖြစ်ပါစေ။
ဖ ၏ တွေ့ချက်များ ထဲသို့ ရှာ(၉) ကို ထည့်မည်။
ဖ ၏ တွေ့ချက်များ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[ရလဒ်ကောင်း(၆)၊ အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: "အနုတ်"))၊ ရလဒ်ကောင်း(၈)]
ရလဒ်ကောင်း(၆)
အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: "အနုတ်"))
ရလဒ်ကောင်း(၈)
[ရလဒ်ကောင်း(၆)၊ အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: "အနုတ်"))၊ ရလဒ်ကောင်း(၈)]
[အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: "အနုတ်"))၊ ရလဒ်ကောင်း(၈)]
[ရလဒ်ကောင်း(၂)၊ အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: "အနုတ်"))]
အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: "အနုတ်"))
[ရလဒ်ကောင်း(၁၈)]
```

Because the levels come after `ဒေတာဆဲလ်`, it is clear which part the levels wrap. To make a list
the error kind, we put the list before `ရှိနိုင်သော ဒေတာဆဲလ်`, as in
`အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း စာရင်း ရှိနိုင်သော ဒေတာဆဲလ်`, and then a single
`ဖိုင်အမှားအယွင်း` does not fit it:

```zaka-error ZK0401
ဖိုင်အမှားအယွင်း ဟူသော ဒေတာအစုအဝေးတွင် စာ ဟူသော စာသား တစ်ခု ပါဝင်သည်။

ရှာ ဟူသော လုပ်ဆောင်ချက် သည်
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

## Ending a function with no value

```text
လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်။
လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။
```

When a side of the result kind has no value, we can use these statements to end the
function with that side. Both end the function at once, as `ပြန်ပေးမည်` does.

| Statement | Ends the function with | For a result kind with |
| --- | --- | --- |
| `လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်။` | a good result that contains no value, the same as a bare `ပြန်ပေးမည်။` | `အောင်မြင်သည့်သတင်း` |
| `လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။` | a failure that contains no error value | `မအောင်မြင်သည့်သတင်း` |

Each statement fits one side of the result kind, and the compiler checks it:

| Case | Code |
| --- | --- |
| `လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။` in a function whose error side has a data kind | ZK0451 |
| `လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။` in a function that cannot fail | ZK0447 |
| `လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်။` in a function with a good value, as with a bare `ပြန်ပေးမည်။` | ZK0313 |
| `<value> ဟူသော အမှားအယွင်း` where the error side has no value, because there is no place for an error value | ZK0476 |

## Passing on and reading

`?`, `ရလဒ်ကောင်း` and the pass-on call statement work with each result kind. For ZK0446,
an error side with no value counts as an error kind in itself, so `မအောင်မြင်သည့်သတင်း`
and `စာသား` are different error kinds. The tests and reads of [Reading a
result](reading-results.md) also work with each result kind, and a read of the error value
of a result whose error side has no value is ZK0450.

A side with no value prints as `အောင်မြင်သည့်သတင်း` or `မအောင်မြင်သည့်သတင်း`. When the
program reads the good result from a result that contains a failure with no value, the VM
stops it with `the result holds a failure, not a good result`, and when the failure has a
value, the message is `the result holds an error, not a good result: V` (see [When a
program stops](../vm/traps.md#a-whole-result-read-as-the-wrong-part)).

## Errors

| Code | Mistake |
| --- | --- |
| ZK0447 | a failure with no value in a function that cannot fail |
| ZK0450 | a read of the error value of a result whose error side has no value |
| ZK0451 | a failure with no value in a function whose error side has a data kind |

```zaka-error ZK0450
ရှာ ဟူသော လုပ်ဆောင်ချက် သည်
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

```zaka-error ZK0451
ရေး ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0451]: the function fails with an error value of the kind "text", so a failure must give one back
```
