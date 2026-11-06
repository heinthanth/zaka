---
title: Passing a failure on
description: How a caller hands the error value of a failing function up to its own caller, in three forms, and where the error value ends.
---

# Passing a failure on

A caller cannot just use the call as a number: the call might have failed, and there would
be no number. Zaka makes the caller answer, and the first answer is to *pass the failure
on*. If the callee fails, the function around the call stops at once and gives back the same
error value. If the callee gives back a good result, the call goes on with it. The second
answer, to keep the whole result and look at it, is in [Reading a whole result](results.md).

The sign form adds a `?` right after the `)`:

```zaka
လုပ်ဆောင်ချက် စားခွဲ သည်
ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
အကယ်၍ ခ သည် ၀ နှင့် ညီလျှင် "သုညဖြင့် စား၍ မရပါ" ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
က / ခ ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// This function can fail too, with the same kind of error value.
လုပ်ဆောင်ချက် ထပ်စား သည်
ကိန်းပြည့် က၊ ကိန်းပြည့် ခ၊ ကိန်းပြည့် ဂ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
"စတင်" ဟု ပြသပါ။
စားခွဲ(စားခွဲ(က၊ ခ)?၊ ဂ)? ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပြ သည်
ကိန်းပြည့် က၊ ကိန်းပြည့် ခ၊ ကိန်းပြည့် ဂ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
ထပ်စား(က၊ ခ၊ ဂ)? ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် ပြ(၁၀၀၊ ၅၊ ၂) ဖြစ်ပါစေ။
အကယ်၍ ရ သည် အမှားအယွင်း ဖြစ်လျှင် "မအောင်မြင်" ဟု ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
စတင်
၁၀
```

Each call in `ထပ်စား` and `ပြ` that can fail passes the failure on: none of them drops it.
The start block cannot pass a failure on, since it cannot fail itself; it reads the
whole result of `ပြ` with a test, as [the end of this page](#the-start-block-cannot-pass-a-failure-on) shows.

A pass-on has three forms:

| Form | Written |
| --- | --- |
| sign form | `စားခွဲ(က၊ ခ)?` |
| word form | `လုပ်ဆောင်ချက် စားခွဲ ကို က၊ ခ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်ကောင်း` |
| call statement | `လုပ်ဆောင်ချက် ပြ ကို ၁၀၀၊ ၅၊ ၂ တို့ဖြင့် လုပ်ဆောင်ပါ၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးပါ။` |

`ရလဒ်ကောင်း` means "the good result". The first two give a value and the third is a
statement, for a function with no result or one whose result is dropped. A call that
passes a failure on can stand wherever a value can: in a sum, in a condition, as an
argument.

## The order of work

A call that passes a failure on runs at the place where it is written. The calls to its
left run first, and the first failure ends the function, so what stands to its right
never runs:

```zaka
လုပ်ဆောင်ချက် စစ် သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
"စစ် {န}" ဟု ပြသပါ။
အကယ်၍ န သည် ၃ နှင့် ညီလျှင် "သုံး" ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
န ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပေါင်း သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
စစ်(၂)? + စစ်(၃)? + စစ်(၄)? ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် ပေါင်း() ဖြစ်ပါစေ။
အကယ်၍ ရ သည် အမှားအယွင်း ဖြစ်လျှင် "မအောင်မြင်: {ရ ကို အမှားအယွင်း အဖြစ် ပြောင်းလဲသော ရလဒ်}" ဟု ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
စစ် ၂
စစ် ၃
မအောင်မြင်: သုံး
```

`စစ်(၄)` is never called: `စစ်(၃)` failed first, so `ပေါင်း` stopped and gave back the
error value. The start block holds that error value in `ရ` and tests it, which is the next
part.

## The start block cannot pass a failure on

Passing a failure on moves it one function up each time, and the start block is the last
one: no caller is left to take what it gives back, since its result is the exit code of
the run. So the start block cannot fail, and a pass-on in it is a mistake:

```zaka-error ZK0445
လုပ်ဆောင်ချက် စားခွဲ သည်
ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
အကယ်၍ ခ သည် ၀ နှင့် ညီလျှင် "သုညဖြင့် စား၍ မရပါ" ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
က / ခ ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စားခွဲ(၁၀၀၊ ၀)? ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0445]: the call passes a failure on, but the code around it cannot fail
```

Writing a result kind on the start block is the mistake ZK0441: its result is an integer
(the exit code) or nothing. There are two exits. Move the call into a function that can
fail and read its result in the start block, as the first examples of this page do, or keep
the whole result of the call and test it:

```zaka
လုပ်ဆောင်ချက် စားခွဲ သည်
ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
အကယ်၍ ခ သည် ၀ နှင့် ညီလျှင် "သုညဖြင့် စား၍ မရပါ" ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
က / ခ ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် စားခွဲ(၁၀၀၊ ၂၅) ဖြစ်ပါစေ။
အကယ်၍ ရ သည် ရလဒ်ကောင်း ဖြစ်လျှင် ရ ကို ရလဒ်ကောင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
ရ ကို စားခွဲ(၁၀၀၊ ၀) ဟု ထားပါ။
အကယ်၍ ရ သည် အမှားအယွင်း ဖြစ်လျှင် ရ ကို အမှားအယွင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄
သုညဖြင့် စား၍ မရပါ
```

The start block decides for itself what a failure means. A program that wants a failed
exit code gives an integer back from the start block.

## Mistakes

A call statement that can fail has to pass the failure on, or its result would be dropped
with no word. Leaving the end of the statement out is an error, so a failure is never lost:

```zaka-error ZK0443
လုပ်ဆောင်ချက် စစ် သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "သုည" ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
န ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် စစ် ကို ၁ တို့ဖြင့် လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0443]: "စစ်" can fail, but the call does not pass the failure on
```

The function around a pass-on must be able to fail, and with the same kind of error value.
Here the callee fails with text, and the caller is written to fail with an integer:

```zaka-error ZK0446
လုပ်ဆောင်ချက် စစ် သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "သုည" ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
န ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် သုံး သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် ကိန်းပြည့် အမှားအယွင်း ကို ပြန်ပေးမည်။
စစ်(၁)? ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0446]: the called function fails with "text", but the function around the call fails with "isize"
```

The other mistakes are in [Errors and results](../../manual/language/error-handling.md#errors), with the
full rules. A function that takes the failure and goes on with its own code does so with the tests of [Reading a whole result](results.md):
keep the whole result, test it, and read the part that is there.

Next: [Reading a whole result](results.md).
