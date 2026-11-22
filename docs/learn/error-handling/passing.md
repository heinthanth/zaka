---
title: Passing a failure on
description: How a caller hands the error value of a failing function up to its own caller, in three forms, and where the error value ends.
---

# Passing a failure on

We can *pass a failure on* (error propagation, `?` in Rust) when we call a function that
can fail. On success, the call gives us the good result (`Ok` value) and the enclosing
function continues. On failure, the enclosing function returns the same error value
(error) at once.

## You will learn

- how to pass a failure on with `?`
- the two word forms that do the same
- in which order the calls of a sentence run
- the reason the program start block cannot pass a failure on

```zaka
စားခွဲ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ ခ သည် ၀ နှင့် ညီလျှင် "သုညဖြင့် စား၍ မရပါ" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
က / ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// divide twice: each call that fails passes its failure on
ထပ်စား ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
ဂ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
စားခွဲ(စားခွဲ(က၊ ခ)?၊ ဂ)? ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ထပ်စား(၁၀၀၊ ၅၊ ၂) ကို ပြသမည်။
ထပ်စား(၁၀၀၊ ၀၊ ၂) ကို ပြသမည်။
```

```text
ရလဒ်ကောင်း(၁၀)
အမှားအယွင်း("သုညဖြင့် စား၍ မရပါ")
```

To pass a failure on, we write `?` straight after the `)` of the call:

```text
<name>(<arguments>)?
```

| Zaka | Reads as | What it does |
| --- | --- | --- |
| `စားခွဲ(က၊ ခ)` | "divide `က` with `ခ`" | the call that can fail |
| `?` | "or pass the failure on" | on a good result, the call becomes that result; on an error value, `ထပ်စား` returns it at once |

In the first call, both divisions succeed, so `ထပ်စား` returns `၁၀` as its good result. In
the second call, the inner division fails, so `ထပ်စား` stops and returns the same error
value, and the outer division never runs. The program prints each whole result
so that we can see both outcomes.

A call with `?` is a value like other values, so we can use it wherever a value goes: in a
sum, in a condition, or as an argument, as the inner call above shows.

## Three forms

```text
<name>(<arguments>)?
<name> ဟူသော လုပ်ဆောင်ချက်ကို <arguments> တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်ကောင်း
<name> ဟူသော လုပ်ဆောင်ချက်ကို <arguments> တို့ဖြင့် လုပ်ဆောင်မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။
```

Besides the sign form with `?`, we can pass a failure on with two word forms. The *word
form* is the long call with `ရလဒ်ကောင်း` ("the good result") at the end, and it means the same
as the sign form. The *call statement* is a whole sentence, and we use it for a function
that has no good result, or one whose good result we do not need:

```zaka
စားခွဲ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ ခ သည် ၀ နှင့် ညီလျှင် "သုညဖြင့် စား၍ မရပါ" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
က / ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// the word form: the same as စားခွဲ(က၊ ခ)?
ပြ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
စားခွဲ ဟူသော လုပ်ဆောင်ချက်ကို က နှင့် ခ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်ကောင်း ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// the call statement: ပြ has no good result to use
အားလုံး ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
ပြ ဟူသော လုပ်ဆောင်ချက်ကို ၁၀၀ နှင့် ၅ တို့ဖြင့် လုပ်ဆောင်မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။
ပြ ဟူသော လုပ်ဆောင်ချက်ကို ၁၀၀ နှင့် ၀ တို့ဖြင့် လုပ်ဆောင်မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။
"ဤနေရာ မရောက်ပါ" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

အားလုံး() ကို ပြသမည်။
```

```text
၂၀
အမှားအယွင်း("သုညဖြင့် စား၍ မရပါ")
```

| Zaka | Reads as |
| --- | --- |
| `… လုပ်ဆောင်သော ရလဒ်ကောင်း` | "the good result of doing …" |
| `… လုပ်ဆောင်မည်၊` | "will do …, and" |
| `မအောင်မြင်ပါက` | "if it does not succeed" |
| `အမှားအယွင်းကို ပြန်ပေးမည်` | "will return the error value" |

`ပြ` and `အားလုံး` return `အောင်မြင်သည့်သတင်း` ("news of success") as their good result,
which means these functions have no good value to return; [Writing result kinds](kinds.md) explains it.
The second call of `ပြ` fails, so `အားလုံး` stops there and the last line never runs.

## The order of work

A call that passes a failure on runs at the place where we wrote it. The calls to its left
run first, and the first failure ends the function, so the calls to its right never run:

```zaka
စစ် ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
"စစ် {န}" ဟု ပြသမည်။
အကယ်၍ န သည် ၃ နှင့် ညီလျှင် "သုံး" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပေါင်း ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
စစ်(၂)? + စစ်(၃)? + စစ်(၄)? ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ရ သည် ပေါင်း() ဖြစ်ပါစေ။
အကယ်၍ ရ ထဲတွင် အမှားအယွင်း ရှိလျှင် "မအောင်မြင်: {ရ ထဲမှ အမှားအယွင်း ဒေတာ}" ဟု ပြသမည်။
```

```text
စစ် ၂
စစ် ၃
မအောင်မြင်: သုံး
```

`စစ်(၄)` is never called, because `စစ်(၃)` failed first, so `ပေါင်း` stopped and returned
the error value. The program keeps that error value in `ရ` and tests it, which
[Reading a whole result](results.md) explains next.

## The program start block cannot pass a failure on

Each time we pass a failure on, it moves one function up, and the program start block (entry
point, `main`) is the last one. No caller is left above it, because its result is the exit
code of the run, so the program start block cannot fail, and a pass-on in it is a mistake:

```zaka-error ZK0445
စားခွဲ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ ခ သည် ၀ နှင့် ညီလျှင် "သုညဖြင့် စား၍ မရပါ" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
က / ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

စားခွဲ(၁၀၀၊ ၀)? ကို ပြသမည်။
```

```text
error[ZK0445]: the call passes a failure on, but the code around it cannot fail
```

There are two fixes. We can move the call into a function that can fail and print its whole
result in the program start block, as the first examples of this page do, or we can keep
the whole result of the call in a variable and test it:

```zaka
စားခွဲ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ ခ သည် ၀ နှင့် ညီလျှင် "သုညဖြင့် စား၍ မရပါ" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
က / ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ရ သည် စားခွဲ(၁၀၀၊ ၂၅) ဖြစ်ပါစေ။
အကယ်၍ ရ ထဲတွင် ရလဒ်ကောင်း ရှိလျှင် ရ ထဲမှ ဒေတာ ကို ပြသမည်။
ရ ကို စားခွဲ(၁၀၀၊ ၀) ဟု ထားမည်။
အကယ်၍ ရ ထဲတွင် အမှားအယွင်း ရှိလျှင် ရ ထဲမှ အမှားအယွင်း ဒေတာ ကို ပြသမည်။
```

```text
၄
သုညဖြင့် စား၍ မရပါ
```

So the program start block decides what a failure means for the whole program. When a
program needs a failed exit code, the program start block returns an integer.

## Mistakes

A call statement to a function that can fail must pass the failure on, or its result would
be dropped without a word. So leaving out the end of the statement is a mistake, and a
failure is never lost:

```zaka-error ZK0443
စစ် ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "သုည" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

စစ် ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်မည်။
```

```text
error[ZK0443]: "စစ်" can fail, but the call does not pass the failure on
```

The function around a pass-on must be able to fail, and with the same data kind (type) of
error value, unless a `ပြောင်းလဲ` converts the one error value into the other (see
Details). Here the callee fails with text, while the caller fails with an integer, and no
function converts text to an integer:

```zaka-error ZK0446
စစ် ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "သုည" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

သုံး ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ကိန်းပြည့် ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
စစ်(၁)? ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
error[ZK0446]: the called function fails with "text", but the function around the call fails with "isize"
```

When a function needs to handle the failure and go on with its own work, it keeps the
whole result and tests it, as [Reading a whole result](results.md) shows.

## Details

- The `?` must touch the `)`, with no space between them.
- **Converting the error.** A function of our own, `ပြောင်းလဲ`, that takes the error value
  of the callee and returns one of the caller's error kind lets the pass-on convert it
  ([Converting the error kind](../../manual/language/error-handling.md#converting-the-error-kind)).
- Writing a result kind on the program start block is the mistake ZK0441, because its result
  is an integer (the exit code) or nothing. The sentences at the top level of a file are not
  inside a function either, so a pass-on there is ZK0445 too.
- A pass-on for a function that cannot fail is ZK0444, and so is `?` on a call that returns
  an [optional](../optionals/README.md), because a blank optional is not a failure.
- When the good result of a function is `အောင်မြင်သည့်သတင်း`, the call has no value, so using
  its `?` or `ရလဒ်ကောင်း` as a value is ZK0315.
- The full rules and the other mistakes are in [Passing a failure
  on](../../manual/language/error-handling.md#passing-a-failure-on) and
  [Errors](../../manual/language/error-handling.md#errors).

## Recap

- We write `?` after a call, or the word form with `ရလဒ်ကောင်း`, to pass a failure on and use
  the good result.
- We write the call statement with `မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်` when we need no
  good result.
- The calls run from left to right, and the first failure ends the function.
- The program start block cannot pass a failure on, so it keeps the whole result and tests
  it.

## Hands on

- Write a function that adds the results of two calls to `စားခွဲ` with `?`, and print its
  whole result for a call that succeeds and one that fails.
- Change the `?` of one call to the word form with `ရလဒ်ကောင်း`, and check that the output
  does not change.
- Remove the end of a call statement, from `မအောင်မြင်ပါက` onward, and read the error.

Next: [Reading a whole result](results.md).
