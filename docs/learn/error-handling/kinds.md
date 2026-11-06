---
title: Writing result kinds
description: Write the kind of a result that can fail, with a good side and an error side, and use it for variables, parameters, fields and function results.
---

# Writing result kinds

So far the kind of a result that can fail was never written whole: the header named the
error kind with a short form, and a variable took its kind from the call. A result
kind can be written too, wherever a kind goes: in a declaration, in a parameter, in the
field of a struct and in the result clause of a function.

```text
<kind> ရလဒ် သို့မဟုတ် <kind> အမှားအယွင်း
```

reads as "a result of this kind, or an error value of that kind". Each side can also hold no value at all:

| Written | The good side | The error side |
| --- | --- | --- |
| `ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း` | an integer | text |
| `အောင်မြင်သည့်သတင်း သို့မဟုတ် စာသား အမှားအယွင်း` | no value | text |
| `ကိန်းပြည့် ရလဒ် သို့မဟုတ် မအောင်မြင်သည့်သတင်း` | an integer | no value |
| `အောင်မြင်သည့်သတင်း သို့မဟုတ် မအောင်မြင်သည့်သတင်း` | no value | no value |

`အောင်မြင်သည့်သတင်း` is "news of success" and `မအောင်မြင်သည့်သတင်း` is "news of failure":
each tells which side was met, and carries nothing more.

## In a clause, a parameter and a variable

The result clause of a function is the written kind, then `ကို ပြန်ပေးမည်`. The function can
fail when its result kind is a result kind, and not otherwise:

```zaka
လုပ်ဆောင်ချက် စားခွဲ သည်
ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
အကယ်၍ ခ သည် ၀ နှင့် ညီလျှင် "သုညဖြင့် စား၍ မရပါ" ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
က / ခ ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပြ သည်
ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း R ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ R သည် ရလဒ်ကောင်း ဖြစ်လျှင် "ရလဒ် {R ကို ရလဒ်ကောင်း အဖြစ် ပြောင်းလဲသော ရလဒ်}" ဟု ပြသပါ။
သို့မဟုတ်ပါက "အမှား: {R ကို အမှားအယွင်း အဖြစ် ပြောင်းလဲသော ရလဒ်}" ဟု ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် စားခွဲ(၁၀၀၊ ၂၅) ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ပြ ကို ရ တို့ဖြင့် လုပ်ဆောင်ပါ။
ရ ကို စားခွဲ(၁၀၀၊ ၀) ဟု ထားပါ။
လုပ်ဆောင်ချက် ပြ ကို ရ တို့ဖြင့် လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရလဒ် ၄
အမှား: သုညဖြင့် စား၍ မရပါ
```

The result kind with `ကို ပြန်ပေးမည်` after it is the one header of a function that can
fail. A function that fails and has no good result writes the side with no value as its good
side: `အောင်မြင်သည့်သတင်း သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်`.

A variable of a result kind can have no starting value. It then holds a failure, and the
failure holds the default of the error side, so an unset result never claims a success that
did not happen:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် R သည် ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း တစ်ခု ဖြစ်ပါစေ။
R ကို ပြသပါ။
ကိန်းရှင် S သည် ကိန်းပြည့် ရလဒ် သို့မဟုတ် မအောင်မြင်သည့်သတင်း တစ်ခု ဖြစ်ပါစေ။
S ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အမှားအယွင်း("")
မအောင်မြင်သည့်သတင်း
```

The default of `စာသား` is the empty text, so `R` is a failure that holds `""`. An error
side of `မအောင်မြင်သည့်သတင်း` holds no value, so `S` is the bare failure. The cells of a list
with a count and a struct field left out of a struct literal start the same way, and so do
the cells of a heap block. Give a starting value, such as the call
`ကိန်းရှင် ရ သည် စားခွဲ(၁၀၀၊ ၂၅) ဖြစ်ပါစေ။`, when the variable has to hold a result at
once, and let the value tell the kind.

## A side with no value

When a failure has nothing to tell, write `မအောင်မြင်သည့်သတင်း` for the error side. The
function ends with a failure through its own sentence, which is a return as
`ပြန်ပေးပါ` is:

```text
လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။
```

reads as "mark the function as not succeeded". The good side with no value is the same
for success: `လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်။` ends the function with a good
result that holds nothing, as a bare `ပြန်ပေးပါ` does. Printing a whole result of
such a side shows the news, with no value in brackets:

```zaka
လုပ်ဆောင်ချက် ရှာ သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ အောက် ငယ်လျှင် လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။
န * ၂ ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ရေး သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "ရေး၍ မရ" ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
"ရေးပြီး {န}" ဟု ပြသပါ။
လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ရှာ(၄) ကို ပြသပါ။
ရှာ(၀ - ၁) ကို ပြသပါ။
ရေး(၁) ကို ပြသပါ။
ရေး(၀) ကို ပြသပါ။
ကိန်းရှင် R သည် ကိန်းပြည့် ရလဒ် သို့မဟုတ် မအောင်မြင်သည့်သတင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ရှာ(၀ - ၂) ဖြစ်ပါစေ။
အကယ်၍ R သည် အမှားအယွင်း ဖြစ်လျှင် "ရှာ၍ မတွေ့" ဟု ပြသပါ။
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

The test `R သည် အမှားအယွင်း` is true for a failure with no value too. What a side with no
value cannot do is be read: there is no value to read. `ရ ကို အမှားအယွင်း အဖြစ်
ပြောင်းလဲသော ရလဒ်` on such a side is an error, shown in [Mistakes](#mistakes).

Passing on works as before. A failure with no value counts as an error side of its own:
a function that fails with no value passes its failure on to a caller that fails with
no value, and a caller that fails with text is the mistake ZK0446. The start block cannot
fail, so it keeps the whole result. Reading the good result of one that holds a failure
stops the run:

```zaka-stop
လုပ်ဆောင်ချက် ရှာ သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ အောက် ငယ်လျှင် လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။
န * ၂ ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် နှစ်ဆ သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ကို ပြန်ပေးမည်။
"စစ် {န}" ဟု ပြသပါ။
ရှာ(န)? + ရှာ(န)? ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
နှစ်ဆ(၅) ကို ပြသပါ။
ကိန်းရှင် ရ သည် နှစ်ဆ(၀ - ၁) ဖြစ်ပါစေ။
ရ ကို ပြသပါ။
"မစမီ" ဟု ပြသပါ။
ရ ကို ရလဒ်ကောင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
"ဤနေရာ မရောက်ပါ" ဟု ပြသပါ။
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

## Struct fields and the short form

A field of a struct can hold a result kind. Here the names end in `ရလဒ်` and
`အမှားအယွင်း`, and the kind of the field is written in the short form:

```zaka
ဒေတာအမျိုးအစား ဖိုင်ရလဒ် တွင် ကိန်းပြည့် အရွယ် တို့ ပါဝင်သည်။
ဒေတာအမျိုးအစား ဖိုင်အမှားအယွင်း တွင် စာသား စာ တို့ ပါဝင်သည်။
ဒေတာအမျိုးအစား ဖတ်ချက် တွင် ဖိုင်ရလဒ် သို့မဟုတ် ဖိုင်အမှားအယွင်း တွေ့ချက်၊ ကိန်းပြည့် အကြိမ် တို့ ပါဝင်သည်။

လုပ်ဆောင်ချက် ဖွင့် သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ဖိုင်ရလဒ် သို့မဟုတ် ဖိုင်အမှားအယွင်း ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် ဖိုင်အမှားအယွင်း(စာ: "မရှိ") ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
ဖိုင်ရလဒ်(အရွယ်: န * ၁၀၀) ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် ဖတ်ချက် တစ်ခု ဖြစ်ပါစေ။
ဖ ၏ တွေ့ချက် ကို ဖွင့်(၃) ဟု ထားပါ။
ဖ ၏ တွေ့ချက် ကို ပြသပါ။
ဖ ၏ တွေ့ချက် ကို ဖွင့်(၀) ဟု ထားပါ။
ဖ ၏ တွေ့ချက် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရလဒ်ကောင်း(ဖိုင်ရလဒ်(အရွယ်: ၃၀၀))
အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: "မရှိ"))
```

A struct whose name ends in `ရလဒ်` can leave out the repeated `ရလဒ်` before `သို့မဟုတ်`, and
a struct whose name ends in `အမှားအယွင်း` can leave out the repeated `အမှားအယွင်း` after it. So

```text
ဖိုင်ရလဒ် သို့မဟုတ် ဖိုင်အမှားအယွင်း
```

is the short form of `ဖိုင်ရလဒ် ရလဒ် သို့မဟုတ် ဖိုင်အမှားအယွင်း အမှားအယွင်း`, and the full form
still works. The short form is read just next to `သို့မဟုတ်` in a kind; a lone `ဖိုင်ရလဒ်`
is the plain struct. It works in the header too:
`ဖိုင်ရလဒ် သို့မဟုတ် ဖိုင်အမှားအယွင်း ကို ပြန်ပေးမည်`.

## Lists of results

A list can be made of results. The word `စာရင်း` goes after the whole result kind, and it
wraps the whole result: each cell of the list holds a result.

| Written | Is |
| --- | --- |
| `ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း စာရင်း` | a list of results |
| `အကွက် ၂ ခုရှိသော ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း စာရင်း` | a list of 2 results |

A count such as `အကွက် ၂ ခုရှိသော` stands before the result kind, and belongs to the list
after it. A result kind with no such word after it is one result, as before. The same
kinds serve a variable, a parameter and a field of a struct:

```zaka
ဒေတာအမျိုးအစား ဖိုင်အမှားအယွင်း တွင် စာသား စာ တို့ ပါဝင်သည်။
ဒေတာအမျိုးအစား ဖတ်ချက် တွင် ကိန်းပြည့် ရလဒ် သို့မဟုတ် ဖိုင်အမှားအယွင်း စာရင်း တွေ့ချက်များ၊ ကိန်းပြည့် အကြိမ် တို့ ပါဝင်သည်။

လုပ်ဆောင်ချက် ရှာ သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် ဖိုင်အမှားအယွင်း ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ အောက် ငယ်လျှင် ဖိုင်အမှားအယွင်း(စာ: "အနုတ်") ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
န * ၂ ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရများ သည် ကိန်းပြည့် ရလဒ် သို့မဟုတ် ဖိုင်အမှားအယွင်း စာရင်း တစ်ခု ဖြစ်ပါစေ။
ရများ ထဲသို့ ရှာ(၃) ကို ထည့်ပါ။
ရများ ထဲသို့ ရှာ(၀ - ၁) ကို ထည့်ပါ။
ရများ ထဲသို့ ရှာ(၄) ကို ထည့်ပါ။
ရများ ကို ပြသပါ။
ရများ ၏ အကွက်တိုင်း ကို ရ အနေဖြင့် ယူ၍ အကယ်၍ ရ သည် အမှားအယွင်း ဖြစ်လျှင် ရ ကို ပြသပါ။

ကိန်းရှင် နှစ်ကွက် သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် ရလဒ် သို့မဟုတ် ဖိုင်အမှားအယွင်း စာရင်း တစ်ခု ဖြစ်ပါစေ။
နှစ်ကွက် ကို ပြသပါ။

ကိန်းရှင် ဖ သည် ဖတ်ချက် တစ်ခု ဖြစ်ပါစေ။
ဖ ၏ တွေ့ချက်များ ထဲသို့ ရှာ(၉) ကို ထည့်ပါ။
ဖ ၏ တွေ့ချက်များ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[ရလဒ်ကောင်း(၆)၊ အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: "အနုတ်"))၊ ရလဒ်ကောင်း(၈)]
အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: "အနုတ်"))
[အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: ""))၊ အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: ""))]
[ရလဒ်ကောင်း(၁၈)]
```

A list prints its cells in brackets, each cell as a whole result. The cells of a list with
a count start as failures that hold the default of the error side. A failure holds no claim
of success, so a cell that was never set is not read as a good value. The cells added with `ထည့်ပါ` hold what was given. Each cell is read with
`အကွက်တိုင်း` as for each list, and the test `ရ သည် အမှားအယွင်း` of
[Reading a whole result](results.md) tells which part a cell holds. Slices and pointers of
results work the same, and are in
[Errors and results](../../manual/language/error-handling.md#result-kinds).

With the short form, the words after a struct name that ends in `အမှားအယွင်း` belong to the
whole result, so `ကိန်းပြည့် ရလဒ် သို့မဟုတ် ဖိုင်အမှားအယွင်း စာရင်း` is a list of results,
as the program above uses it. A list as the error kind needs the full form,
`ကိန်းပြည့် ရလဒ် သို့မဟုတ် ဖိုင်အမှားအယွင်း စာရင်း အမှားအယွင်း`, so the error value of a
function with that clause is a list, and a single `ဖိုင်အမှားအယွင်း` does not fit it:

```zaka-error ZK0401
ဒေတာအမျိုးအစား ဖိုင်အမှားအယွင်း တွင် စာသား စာ တို့ ပါဝင်သည်။

လုပ်ဆောင်ချက် ရှာ သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် ဖိုင်အမှားအယွင်း စာရင်း အမှားအယွင်း ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ အောက် ငယ်လျှင် ဖိုင်အမှားအယွင်း(စာ: "အနုတ်") ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
န * ၂ ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ရှာ(၃) ကို ပြသပါ။
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
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ အောက် ငယ်လျှင် လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။
န * ၂ ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် ရှာ(၁) ဖြစ်ပါစေ။
ရ ကို အမှားအယွင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0450]: the error side of the result is "မအောင်မြင်သည့်သတင်း": it holds no value to read
```

A failure in a function whose error side names a kind has to hold a value of that kind
(ZK0451):

```zaka-error ZK0451
လုပ်ဆောင်ချက် ရေး သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0451]: the function fails with an error value of the kind "text", so a failure must give one back
```

And a failure in a function whose error side holds no value cannot hold one (ZK0452):

```zaka-error ZK0452
လုပ်ဆောင်ချက် ရေး သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ကို ပြန်ပေးမည်။
"ရေး၍ မရ" ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0452]: a failure of "ရေး" holds no error value
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

Next: [Lists](../lists/README.md).
