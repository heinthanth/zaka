---
title: Results and returns
description: How a function declares its result, how a return statement ends it, the rule that each path returns, and lists and structs as results.
---

# Results and returns

A function can return a result (return value) to its caller, and a return statement ends
the function. This page covers the result in the function signature, the return
statement, the rule that each path of a function with a result returns, results that can
fail, and lists and structs as results. The Learn page
[Results](../../learn/functions/results.md) teaches the same ideas with examples.

```zaka
// square: one parameter, one result
နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက်ကို ၇ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၉
```

## Declaring a result

```text
<name> ဟူသော လုပ်ဆောင်ချက် သည်
<parameters> လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ <kind> တစ်<word> ကို ပြန်ပေးမည်။
```

To give a function a result, we end its function signature with
`အောက်ပါတို့ကို လုပ်ဆောင်ကာ <kind> တစ်<word> ကို ပြန်ပေးမည်။`, where `<word>` is the
measure word (classifier) of the data kind (type). The same form works for a result kind
and for an optional kind (`... ရှိနိုင်သော ဒေတာဆဲလ်`, `... ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`).
A function without a result ends its signature with `အောက်ပါတို့ကို လုပ်ဆောင်မည်။` and
returns nothing.

| Form | Rule | Code |
| --- | --- | --- |
| no measure word after the data kind | an error | ZK0234 |
| a measure word that does not fit the data kind | a warning, whose help line names the right measure word | ZK0509 |

## Return statements

```text
<value> ကို ပြန်ပေးမည်။
ပြန်ပေးမည်။
```

In a function with a result, we write `<value> ကို ပြန်ပေးမည်။` to return the value at once,
and the call then has that value. In a function without a result, we write `ပြန်ပေးမည်။`
alone to end it before its last statement. The function has no variable named `ရလဒ်`: a name that is never
declared is ZK0301, and for `ရလဒ်` the help line of that error shows the return
`<value> ကို ပြန်ပေးမည်။`.

The compiler checks these mistakes:

| Mistake | Code |
| --- | --- |
| a value returned from a function without a result | ZK0312 |
| `ပြန်ပေးမည်။` with no value in a function with a result | ZK0313 |
| a return statement outside a function | ZK0311 |
| a function with a result whose block can reach its end with no return | ZK0339 |
| a value that does not fit the result kind | ZK0401 |

```zaka-error ZK0312
ပြ ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0312]: function "ပြ" returns nothing, but a value is returned
```

```zaka-error ZK0313
တစ် ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0313]: function "တစ်" returns isize, but no value is returned
```

## Each path returns

A function with a result must return on each path. Its block must end in one of these:

- a return statement;
- a block that returns;
- an `if` with a `မဟုတ်ပါက` arm where each arm returns;
- a `switch` with a default where each arm returns;
- a loop whose test is `မှန်` and that no break leaves, which never ends.

Other loops do not count, because their test can be false at the start. A block that can
reach its end with no return is ZK0339:

```zaka-error ZK0339
မေ့ ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0339]: function "မေ့" gives back a result, but its block can reach its end without giving one back
```

The program start block is different: it can end without a return, and its exit code is
then 0 ([The program start block](functions-start-block.md)).

## Results that can fail

```text
အောက်ပါတို့ကို လုပ်ဆောင်ကာ <kind> သို့မဟုတ် အမှားအယွင်းအနေဖြင့် <kind> ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် <kind> ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
```

The result of a function can also be a whole result kind (result type), and the second
form is for a good result with no value. Such a function can fail: we write
`<value> ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။` to return an error value instead of a result.
A call to such a function passes the failure on or reads the whole result
([Calls that can fail](functions-calls.md#calls-that-can-fail)). The full rules are in
[Errors and results](error-handling.md).

## Lists and structs as results

A function can return a list or a struct. The result follows the rules of an assignment,
so the caller gets the whole value as its own duplicate (copy). See
[Values and objects](objects.md).

```zaka
နေရာ ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု နှင့် y ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။
နေရာသစ် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ နေရာ တစ်ခု ကို ပြန်ပေးမည်။
x အနေဖြင့် က နှင့် y အနေဖြင့် ခ ရှိသော နေရာတစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

အစဉ် ဟူသော လုပ်ဆောင်ချက် သည်
စ ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
၀ မှ ၂ အထိ န အနေဖြင့် ရေတွက်၍ အဖြေ ၏ အကွက် န ကို စ + န ဟု ထားမည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
နေရာသစ် ဟူသော လုပ်ဆောင်ချက်ကို ၃ နှင့် ၄ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
အစဉ် ဟူသော လုပ်ဆောင်ချက်ကို ၅ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
နေရာ(x: ၃၊ y: ၄)
[၅၊ ၆၊ ၇]
```

- A plain list result gets its count from the value returned, while a counted list result
  has a written count.
- A function can return a [range](lists.md) of a local list, because a range is a new
  list.
- The caller keeps a slot for the result of each call. With the default,
  `--results in-place`, the function builds a returned literal, or the local that each
  return returns, straight in that slot. With `--results duplicate`, the function keeps
  the value in its own frame and the return duplicates it into the slot. The program
  prints the same in both modes; see
  [Commands](../compiler/commands.md#options-of-the-compile-stages).
- A pointer result can have `အပိုင်အနေဖြင့်`, as in
  `<kind> တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်`, and the caller then owns the block. See
  [Unsafe Zaka](unsafe.md).
