---
title: Errors and results
description: A function that gives back a good result or an error value, the forms a caller passes a failure on in, how a caller reads a whole result, and the errors the compiler reports.
---

# Errors and results

A function can fail. Zaka writes a failure as a value: the function gives back either its
good result or an *error value*, and the call has to tell what happens to a failure: pass it
on, or keep the whole result and look at it. There are no exceptions and no jump to a distant
handler. This page states the rules in full. The
Learn book starts in small steps with [Errors and results](../../learn/error-handling/README.md).

```zaka
လုပ်ဆောင်ချက် စားခွဲ သည်
ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
အကယ်၍ ခ သည် ၀ နှင့် ညီလျှင် "သုညဖြင့် စား၍ မရပါ" ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
က / ခ ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပြ သည်
ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
စားခွဲ(က၊ ခ)? ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် ပြ(၁၀၊ ၂) ဖြစ်ပါစေ။
အကယ်၍ ရ သည် အမှားအယွင်း ဖြစ်လျှင် ရ ကို အမှားအယွင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၅
```

## The result clause

A function is marked as able to fail in its result clause, after `အောက်ပါတို့ကို လုပ်ဆောင်ကာ`:

| Clause | The function |
| --- | --- |
| `<kind> ရလဒ် ကို ပြန်ပေးမည်။` | cannot fail; gives back a result |
| `<result kind> ကို ပြန်ပေးမည်။` | can fail; the [result kind](#result-kinds) tells the good side and the error side |
| `အောင်မြင်သည့်သတင်း သို့မဟုတ် <error kind> အမှားအယွင်း ကို ပြန်ပေးမည်။` | can fail, and has no good value (the good side holds nothing) |

A function can fail when its result kind is a result kind, and not otherwise, so the written result kind with `ကို ပြန်ပေးမည်` after it is the one header of a function that can fail.

The error kind can be whatever kind is needed: an integer, text, a struct, a list. A function that can
fail has the variable `ရလဒ်`, and gives the good result back with `<value> ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။`.
A function that cannot fail keeps `ရလဒ်အဖြစ်`, and a function with the word of the other
sort is ZK0455 (an example is on [Reading a whole result](#reading-a-whole-result)). A function whose
good side holds no value has no `ရလဒ်`, as a function with no result has none. When
it reaches its end, it gives back a good result: the value of `ရလဒ်`, or nothing.

The [start block](functions.md#the-start-block) cannot use these clauses: it cannot fail, and
its result is an integer, the exit code of the run, or nothing. A result kind on it is ZK0441 (see [The start block](#the-start-block)).

## Giving back an error value

```text
<value> ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
```

ends the function at once and gives back the value as the error value. `အမှားအယွင်းအနေဖြင့်`
is a second spelling of `အမှားအယွင်းအဖြစ်`, and `ပြန်ပေးမည်` can stand in for `ပြန်ပေးပါ`. The
value must fit the error kind of the clause (ZK0401). The statement needs a function
that can fail: in another function it is ZK0447, and in a function whose error side holds
no value it is ZK0452.

## Passing a failure on

When a caller meets a function that can fail, it has to tell what happens to a failure.
One answer is to *pass the failure on*; the other, to keep the whole result, is in
[Reading a whole result](#reading-a-whole-result). When the callee fails, the function around the call stops at once and gives back the
same error value; when the callee gives back a good result, the call goes on with it.
This is the `?` of Rust. There are three forms:

| Form | Written | Is |
| --- | --- | --- |
| word form | `လုပ်ဆောင်ချက် <name> ကို <arguments> တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်ကောင်း` | a value: the good result |
| sign form | `<name>(<arguments>)?` | a value: the good result; the `?` touches the `)` |
| call statement | `လုပ်ဆောင်ချက် <name> ကို <arguments> တို့ဖြင့် လုပ်ဆောင်ပါ၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးပါ။` | a statement; a good result is dropped |

`ရလဒ်ကောင်း` means "the good result". In the call statement, `ပြန်ပေးမည်` can stand in for
`ပြန်ပေးပါ`. The word form and the sign form are the same call, as the two spellings of a
plain call are; the same checks and the same codes hold.

```zaka
လုပ်ဆောင်ချက် စစ် သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ အောက် ငယ်လျှင် "အနုတ်" ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
န * ၂ ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ရေး သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် စစ် ကို န တို့ဖြင့် လုပ်ဆောင်ပါ၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးပါ။
"ရေး {န}" ဟု ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် စုံ သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
(လုပ်ဆောင်ချက် စစ် ကို န တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်ကောင်း) + စစ်(န + ၁)? ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် အားလုံး သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် ရေး ကို ၄ တို့ဖြင့် လုပ်ဆောင်ပါ၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးပါ။
စုံ(၃)? ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် အားလုံး() ဖြစ်ပါစေ။
အကယ်၍ ရ သည် အမှားအယွင်း ဖြစ်လျှင် ရ ကို အမှားအယွင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရေး ၄
၁၄
```

A pass-on call is a value like each other value, so it can stand wherever an expression can: in a
sum, as an argument, in a condition, in a declaration, behind a return.

The function around the call must itself be able to fail (ZK0445), and with the same error
kind (ZK0446): the value goes out as it is, and Zaka does not convert one error kind into
another. The [start block](#the-start-block) cannot fail, so a pass-on there is ZK0445. A call statement to a function that can fail with no pass-on is ZK0443, so a
failure is never dropped without a word. The reverse, a pass-on for a function that cannot
fail, is ZK0444. A function whose good side holds no value has no good result to read: `ရလဒ်ကောင်း`
or `?` on it, used as a value, is ZK0315, as for a value of each function with no result.

## Reading a whole result

A call to a function that can fail, written with no `ရလဒ်ကောင်း` and no `?`, passes nothing
on: it gives back the *whole result*, a good result or an error value. The call is an
expression like each other call; just the call *statement* needs the pass-on, because
there the result would be dropped (ZK0443).

```text
ကိန်းရှင် R သည် လုပ်ဆောင်ချက် <name> ကို <arguments> တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် R သည် <name>(<arguments>) ဖြစ်ပါစေ။
R ကို <name>(<arguments>) ဟု ထားပါ။
```

The kind of a whole result can be [written](#result-kinds), or taken from the value in
a declaration with a starting value; `ထားပါ` stores another result of the same kinds (the
same good side and the same error side). In a message the kind reads like
`isize result or text error`, `success or text error` and `isize result or failure`. A whole
result is no number: using it as one is ZK0403.

`ပြသပါ` and a `{...}` slot write a whole result as the part it holds:
`ရလဒ်ကောင်း(<value>)` or `အမှားအယွင်း(<value>)`. A side with no value is written as its
news, `အောင်မြင်သည့်သတင်း` or `မအောင်မြင်သည့်သတင်း`, with no brackets.

```zaka
လုပ်ဆောင်ချက် ပိတ် သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "မပိတ်နိုင်ပါ" ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
န ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပိတ်(၀) ကို ပြသပါ။
ကိန်းရှင် ရ သည် ပိတ်(၇) ဖြစ်ပါစေ။
"ရ = {ရ}" ဟု ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အမှားအယွင်း("မပိတ်နိုင်ပါ")
ရ = ရလဒ်ကောင်း(၇)
```

### Tests

| Written | Is |
| --- | --- |
| `<R> သည် ရလဒ်ကောင်း` | true when `R` holds a good result |
| `<R> သည် အမှားအယွင်း` | true when `R` holds an error value |

Each test is a true or false value, so it can stand wherever one can: in `အကယ်၍` and
`ထိုသို့မဟုတ်ဘဲ`, in the condition of a loop, and beside `ဖြစ်၍` and `သို့မဟုတ်` (`&&` and
`||`), and as the operand of a print: `R သည် အမှားအယွင်း ကို ပြသပါ။` writes `မှန်` or
`မမှန်`.

### Conversions

| Written | Is |
| --- | --- |
| `<R> ကို ရလဒ်ကောင်း အဖြစ် ပြောင်းလဲသော ရလဒ်` | the good result, as `unwrap` of Rust gives it |
| `<R> ကို အမှားအယွင်း အဖြစ် ပြောင်းလဲသော ရလဒ်` | the error value, as `unwrap_err` gives it |

These are the [conversion sentence](conversions.md) with `ရလဒ်ကောင်း` or `အမှားအယွင်း` in the
place of a kind. A conversion of the part that is not there stops the run with a VM
trap, as a conversion that cannot be made does:

| Read | Held | Trap | The run stops with |
| --- | --- | --- | --- |
| the good result | an error value | `not_good` | `the result holds an error, not a good result: V` |
| the error value | a good result | `not_fault` | `the result is a good result, not an error` |

```zaka
လုပ်ဆောင်ချက် စစ် သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
"စစ် {န}" ဟု ပြသပါ။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "သုည" ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
န ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ရေး သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "ရေး၍ မရ" ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် စစ်(၅) ဖြစ်ပါစေ။
အကယ်၍ ရ သည် ရလဒ်ကောင်း ဖြစ်လျှင် ရ ကို ရလဒ်ကောင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
ရ ကို စစ်(၀) ဟု ထားပါ။
အကယ်၍ ရ သည် အမှားအယွင်း ဖြစ်လျှင် ရ ကို အမှားအယွင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
အကယ်၍ စစ်(၃) သည် ရလဒ်ကောင်း && စစ်(၀) သည် အမှားအယွင်း ဖြစ်လျှင် "နှစ်ခုလုံး" ဟု ပြသပါ။
အကယ်၍ စစ်(၀) သည် ရလဒ်ကောင်း && စစ်(၇) သည် ရလဒ်ကောင်း ဖြစ်လျှင် "မရောက်ပါ" ဟု ပြသပါ။
ကိန်းရှင် ဂ သည် ရေး(၀) ဖြစ်ပါစေ။
အကယ်၍ ဂ သည် အမှားအယွင်း ဖြစ်လျှင် ဂ ကို အမှားအယွင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
ဂ ကို ရေး(၁) ဟု ထားပါ။
အကယ်၍ ဂ သည် ရလဒ်ကောင်း ဖြစ်လျှင် "ရေးပြီး" ဟု ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
စစ် ၅
၅
စစ် ၀
သုည
စစ် ၃
စစ် ၀
နှစ်ခုလုံး
စစ် ၀
ရေး၍ မရ
ရေးပြီး
```

The second `အကယ်၍` of the two near the end never calls `စစ်(၇)`: its left side is false, so
the right side of `&&` is not reached. The function `ရေး` has no result, so its whole
result holds an error value or nothing: `ဂ သည် ရလဒ်ကောင်း` is true when it did not fail, and
there is no good value to read from it.

```zaka-stop
လုပ်ဆောင်ချက် စစ် သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "သုည" ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
န ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် စစ်(၀) ဖြစ်ပါစေ။
"မစမီ" ဟု ပြသပါ။
ရ ကို ရလဒ်ကောင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
"ဤနေရာ မရောက်ပါ" ဟု ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မစမီ
the program stopped: the result holds an error, not a good result: သုည
```

```zaka-stop
လုပ်ဆောင်ချက် စစ် သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "သုည" ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
န ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် စစ်(၄) ဖြစ်ပါစေ။
ရ ကို အမှားအယွင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
"ဤနေရာ မရောက်ပါ" ဟု ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: the result is a good result, not an error
```

### Order of work

A conversion runs where it is written, as a pass-on call does: the operands on its left run
first, and the right side of `&&` and `||` runs just when it is reached. So a read of the
wrong part stops the run at that place, and nothing to its right runs.

### A starting value

A variable with no starting value gets the default of its kind. The default of a result
kind is a failure that holds the default of the error side, or the bare failure when the
error side is `မအောင်မြင်သည့်သတင်း`. A good result with a default value would claim a
success that did not happen, so an unset result never holds one:

```zaka
ဒေတာအမျိုးအစား ဖတ်ချက် တွင် ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း တွေ့ချက်၊ ကိန်းပြည့် အကြိမ် တို့ ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် R သည် ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း တစ်ခု ဖြစ်ပါစေ။
R ကို ပြသပါ။
ကိန်းရှင် ဖ သည် ဖတ်ချက်(အကြိမ်: ၁) ဖြစ်ပါစေ။
ဖ ကို ပြသပါ။
ကိန်းရှင် တစ်ခုတည်း သည် ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ၁ ခုစာ နေရာယူ၍ လိပ်စာကို တစ်ခုတည်း တွင် သိမ်းထားမည်။
တစ်ခုတည်း ၏ တန်ဖိုး ကို ပြသပါ။
လိပ်စာ တစ်ခုတည်း ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အမှားအယွင်း("")
ဖတ်ချက်(တွေ့ချက်: အမှားအယွင်း("")၊ အကြိမ်: ၁)
အမှားအယွင်း("")
```

The rule holds in each place a result starts with no value:

- a variable declared with no starting value;
- the cells of a list with a count, and a field of a struct that is a result kind and is
  left out of a struct literal;
- the cells of a fresh heap block of results.

A plain list of results starts with no cells, and `ထည့်ပါ` adds each cell. With the error
side `မအောင်မြင်သည့်သတင်း` the unset result prints as `မအောင်မြင်သည့်သတင်း`. Write a call as
the starting value, as `ကိန်းရှင် R သည် ရှာ(၄) ဖြစ်ပါစေ။`, when the variable has to hold
the result of that call.

### A part the test ruled out

A test proves which part a result variable holds, for as long as nothing changes the
variable. A conversion to the part the test ruled out would stop the run each time it ran,
so the compiler reports it as ZK0454 and does not wait for the run. The message tells
which part is held: `"R" holds a good result here, so reading its error value always stops
the run` or `"R" holds a failure here, so reading its good result stops the run each time`.

```zaka-error ZK0454
လုပ်ဆောင်ချက် ရှာ သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
န * ၂ ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် R သည် ရှာ(၄) ဖြစ်ပါစေ။
အကယ်၍ R သည် ရလဒ်ကောင်း ဖြစ်လျှင် R ကို အမှားအယွင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0454]: "R" holds a good result here, so reading its error value stops the run each time
```

What proves a part:

- Inside the arm of `အကယ်၍ R သည် ရလဒ်ကောင်း ဖြစ်လျှင်`, `R` holds a good result; inside the
  arm of `အကယ်၍ R သည် အမှားအယွင်း ဖြစ်လျှင်`, it holds a failure.
- The arms after an arm (each `ထိုသို့မဟုတ်ဘဲ` and the `သို့မဟုတ်ပါက`) get the opposite
  of the tests before them.
- In the condition of an arm, `ဖြစ်၍` (`&&`) proves both sides, `သို့မဟုတ်` (`||`) proves
  both sides false in the arms after it, and `မဟုတ်` flips what its clause proves.
- An `အကယ်၍` arm that is the whole statement, with no arm after it, and whose block ends
  with `ပြန်ပေးပါ`, `ရပ်ပါ` or `ကျော်ပါ`, proves the opposite after the whole statement: a
  guard. After `အကယ်၍ R သည် အမှားအယွင်း ဖြစ်လျှင် ပြန်ပေးပါ`, `R` holds a good result.
- The condition of a while loop proves its test inside the loop.

What ends a proof:

- an assignment to `R`;
- `R` given to a call with the edit mark `(လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်)`;
- the address of `R`;
- a new declaration of the same name;
- a loop that can change `R` in its statements, nested blocks included: for a proof made outside the
  loop, the whole loop is without it. A test made inside the loop still proves its own arm.

When the part cannot be proven, nothing is reported, and the read is checked when the run
reaches it: it stops with `not_good` or `not_fault`, as above.

### A good result of a function that can fail

A function that can fail gives its good result back with `<value> ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ`,
and a function that cannot fail uses `ရလဒ်အဖြစ်`. The word of the other sort of function
is ZK0455, in each direction:

```zaka-error ZK0455
လုပ်ဆောင်ချက် ရှာ သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
န * ၂ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0455]: "ရှာ" can fail, so its good result goes back with "ရလဒ်ကောင်းအဖြစ်"
```

```zaka-error ZK0455
လုပ်ဆောင်ချက် နှစ်ဆ သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
န * ၂ ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0455]: "နှစ်ဆ" cannot fail, so its result goes back with "ရလဒ်အဖြစ်"
```

### Mistakes

| Code | Mistake |
| --- | --- |
| ZK0448 | a test or a conversion of a value that is no result of a function that can fail |
| ZK0449 | a conversion to `ရလဒ်ကောင်း` of the result of a function with no good result |
| ZK0454 | a conversion to the part that a test ruled out |

The last two are on the parts above, with an example each.

```zaka-error ZK0448
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၅ ကို ရလဒ်ကောင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0448]: the value is "isize", not the result of a function that can fail
```

```zaka-error ZK0449
လုပ်ဆောင်ချက် ရေး သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် ရေး() ဖြစ်ပါစေ။
ရ ကို ရလဒ်ကောင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0449]: the function gives back no good value, so the result has none to read
```

## Result kinds

A result kind can be written wherever a kind goes: in a declaration, a parameter, a field of a
struct and the result clause of a function. It names the good side and the error side:

```text
<kind> ရလဒ် သို့မဟုတ် <kind> အမှားအယွင်း
```

The good side is a kind, then `ရလဒ်`, or `အောင်မြင်သည့်သတင်း` when it holds no value.
The error side is a kind, then `အမှားအယွင်း`, or `မအောင်မြင်သည့်သတင်း` when it
holds no value:

| Written | Good side | Error side |
| --- | --- | --- |
| `<T> ရလဒ် သို့မဟုတ် <E> အမှားအယွင်း` | a value of `T` | a value of `E` |
| `အောင်မြင်သည့်သတင်း သို့မဟုတ် <E> အမှားအယွင်း` | no value | a value of `E` |
| `<T> ရလဒ် သို့မဟုတ် မအောင်မြင်သည့်သတင်း` | a value of `T` | no value |
| `အောင်မြင်သည့်သတင်း သို့မဟုတ် မအောင်မြင်သည့်သတင်း` | no value | no value |

The first three are `Result<T, E>`, `Result<(), E>` and `Result<T, ()>`, as Rust writes
them. A function can fail when its result kind is a result kind, and just then.
In a message the kinds read like `isize result or text error`, `success or text error` and
`isize result or failure`.

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

လုပ်ဆောင်ချက် ပြ သည်
ကိန်းပြည့် ရလဒ် သို့မဟုတ် မအောင်မြင်သည့်သတင်း R ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
R ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ဆင့် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ကို ပြန်ပေးမည်။
ရှာ(၀ - ၁)? ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် ဖတ်ချက် တစ်ခု ဖြစ်ပါစေ။
ဖ ၏ တွေ့ချက် ကို ဖွင့်(၃) ဟု ထားပါ။
ဖ ၏ တွေ့ချက် ကို ပြသပါ။
ဖွင့်(၀) ကို ပြသပါ။
ကိန်းရှင် R သည် ကိန်းပြည့် ရလဒ် သို့မဟုတ် မအောင်မြင်သည့်သတင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ရှာ(၄) ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ပြ ကို R တို့ဖြင့် လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် ပြ ကို ရှာ(၀ - ၂) တို့ဖြင့် လုပ်ဆောင်ပါ။
ရေး(၁) ကို ပြသပါ။
ရေး(၀) ကို ပြသပါ။
ဆင့်() ကို ပြသပါ။
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
မအောင်မြင်သည့်သတင်း
```

### Lists and pointers of results

A result kind can take list, slice and pointer levels after it; the levels wrap the *whole*
result, not the error side:

| Written | Is |
| --- | --- |
| `ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း စာရင်း` | a plain list of results |
| `အကွက် ၂ ခုရှိသော ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း စာရင်း` | a list of 2 results |
| `ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း စာရင်း တစ်စိတ်တစ်ပိုင်း` | a slice of results |
| `ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း လိပ်စာ` | a pointer to a result |
| `ငှားထားသော ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း လိပ်စာ` | a borrowed pointer to a result |

A count prefix (`အကွက် ၂ ခုရှိသော`) and the mark `ငှားထားသော` written before a result kind
belong to its levels. The kinds serve a variable, a parameter, a field of a struct and the
cells of a heap block taken with `မင်မိုရီတွင်`. A cell of a list with a count starts as a
failure that holds the default of the error side, so a cell never set claims no success; a cell added with
`ထည့်ပါ` holds the result given. A list prints its cells in brackets, each as a whole
result, and a cell is read with `အကွက်တိုင်း` or a slice as for each list, then tested and converted as in [Reading a whole result](#reading-a-whole-result).

```zaka
ဒေတာအမျိုးအစား ဖိုင်အမှားအယွင်း တွင် စာသား စာ တို့ ပါဝင်သည်။
ဒေတာအမျိုးအစား ဖတ်ချက် တွင် ကိန်းပြည့် ရလဒ် သို့မဟုတ် ဖိုင်အမှားအယွင်း စာရင်း တွေ့ချက်များ၊ ကိန်းပြည့် အကြိမ် တို့ ပါဝင်သည်။

လုပ်ဆောင်ချက် ရှာ သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် ဖိုင်အမှားအယွင်း ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ အောက် ငယ်လျှင် ဖိုင်အမှားအယွင်း(စာ: "အနုတ်") ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
န * ၂ ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပြ သည်
ကိန်းပြည့် ရလဒ် သို့မဟုတ် ဖိုင်အမှားအယွင်း စာရင်း တစ်စိတ်တစ်ပိုင်း အပိုင်း ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အပိုင်း ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ဖတ် သည်
ငှားထားသော ကိန်းပြည့် ရလဒ် သို့မဟုတ် ဖိုင်အမှားအယွင်း လိပ်စာ ပ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပ ၏ တန်ဖိုး ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရများ သည် ကိန်းပြည့် ရလဒ် သို့မဟုတ် ဖိုင်အမှားအယွင်း စာရင်း တစ်ခု ဖြစ်ပါစေ။
ရများ ထဲသို့ ရှာ(၃) ကို ထည့်ပါ။
ရများ ထဲသို့ ရှာ(၀ - ၁) ကို ထည့်ပါ။
ရများ ထဲသို့ ရှာ(၄) ကို ထည့်ပါ။
ရများ ကို ပြသပါ။
ရများ ၏ အကွက်တိုင်း ကို ရ အနေဖြင့် ယူ၍ ရ ကို ပြသပါ။
လုပ်ဆောင်ချက် ပြ ကို ရများ ၏ တန်ဖိုးအငှား တို့ဖြင့် လုပ်ဆောင်ပါ။
ကိန်းရှင် အလယ် သည် ကိန်းပြည့် ရလဒ် သို့မဟုတ် ဖိုင်အမှားအယွင်း စာရင်း တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ရများ ၏ အကွက် ၁ မှစ၍ ၂ ကွက် ဖြစ်ပါစေ။
အလယ် ကို ပြသပါ။

ကိန်းရှင် နှစ်ကွက် သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် ရလဒ် သို့မဟုတ် ဖိုင်အမှားအယွင်း စာရင်း တစ်ခု ဖြစ်ပါစေ။
နှစ်ကွက် ကို ပြသပါ။

ကိန်းရှင် တစ်ခုတည်း သည် ကိန်းပြည့် ရလဒ် သို့မဟုတ် ဖိုင်အမှားအယွင်း လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ရလဒ် သို့မဟုတ် ဖိုင်အမှားအယွင်း ၁ ခုစာ နေရာယူ၍ လိပ်စာကို တစ်ခုတည်း တွင် သိမ်းထားမည်။
တစ်ခုတည်း ၏ တန်ဖိုး ကို ရှာ(၀ - ၅) ဟု ထားပါ။
လုပ်ဆောင်ချက် ဖတ် ကို တစ်ခုတည်း ၏ တန်ဖိုးအငှား တို့ဖြင့် လုပ်ဆောင်ပါ။
လိပ်စာ တစ်ခုတည်း ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။

ကိန်းရှင် ဖ သည် ဖတ်ချက် တစ်ခု ဖြစ်ပါစေ။
ဖ ၏ တွေ့ချက်များ ထဲသို့ ရှာ(၉) ကို ထည့်ပါ။
ဖ ၏ တွေ့ချက်များ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[ရလဒ်ကောင်း(၆)၊ အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: "အနုတ်"))၊ ရလဒ်ကောင်း(၈)]
ရလဒ်ကောင်း(၆)
အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: "အနုတ်"))
ရလဒ်ကောင်း(၈)
[ရလဒ်ကောင်း(၆)၊ အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: "အနုတ်"))၊ ရလဒ်ကောင်း(၈)]
[အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: "အနုတ်"))၊ ရလဒ်ကောင်း(၈)]
[အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: ""))၊ အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: ""))]
အမှားအယွင်း(ဖိုင်အမှားအယွင်း(စာ: "အနုတ်"))
[ရလဒ်ကောင်း(၁၈)]
```

With the short form, the levels after a struct name that ends in `အမှားအယွင်း` belong to the
whole result: `ကိန်းပြည့် ရလဒ် သို့မဟုတ် ဖိုင်အမှားအယွင်း စာရင်း` is a list of results.
A list as the error kind needs the full form, `ဖိုင်အမှားအယွင်း စာရင်း အမှားအယွင်း`, so a
single `ဖိုင်အမှားအယွင်း` does not fit it:

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

### The short form

A struct whose name ends in `ရလဒ်` can leave out the repeated `ရလဒ်` before `သို့မဟုတ်`, and a
struct whose name ends in `အမှားအယွင်း` can leave out the repeated `အမှားအယွင်း` after it. So
`ဖိုင်ရလဒ် သို့မဟုတ် ဖိုင်အမှားအယွင်း` is `ဖိုင်ရလဒ် ရလဒ် သို့မဟုတ် ဖိုင်အမှားအယွင်း အမှားအယွင်း`;
the full form still works, and so does a mix of the two. The short form is read just next to
`သို့မဟုတ်` in a kind: a lone `ဖိုင်ရလဒ်` is the plain struct. It applies in the result clause
as well: `ဖိုင်ရလဒ် သို့မဟုတ် ဖိုင်အမှားအယွင်း ကို ပြန်ပေးမည်`. The rule will hold for
enumerations too, when Zaka has them.

### Marking the end of a function

| Statement | Ends the function with | For a result kind with |
| --- | --- | --- |
| `လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်။` | a good result that holds no value, as a bare `ပြန်ပေးပါ။` does | `အောင်မြင်သည့်သတင်း` |
| `လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။` | a failure that holds no error value | `မအောင်မြင်သည့်သတင်း` |

Both end the function at once, as `ပြန်ပေးပါ` does. The second needs an error side with
no value: in a function whose error side names a kind it is ZK0451, and
`<value> ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ` in a function whose error side holds no value is ZK0452.

### Passing on and reading

`?`, `ရလဒ်ကောင်း` and the statement trailer work with each result kind. For ZK0446, an
error side with no value is an error kind of its own, so `မအောင်မြင်သည့်သတင်း` and `စာသား` differ. The
tests `R သည် ရလဒ်ကောင်း` and `R သည် အမှားအယွင်း` work with each kind. A read of the error value
of a result whose error side holds no value is ZK0450.

A side with no value prints as `အောင်မြင်သည့်သတင်း` or `မအောင်မြင်သည့်သတင်း`. When the good
result is read from a result that holds a failure with no value, the VM traps with
`the result holds a failure, not a good result`; with a value the message is
`the result holds an error, not a good result: V`
(see [When a program stops](../vm/traps.md#a-whole-result-read-as-the-wrong-part)).

### Mistakes

| Code | Mistake |
| --- | --- |
| ZK0450 | a read of the error value of a result whose error side holds no value |
| ZK0451 | a failure with no value in a function whose error side names a kind |
| ZK0452 | an error value given back from a function whose error side holds no value |

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

## Placement and order of work

The call and its test run at the place where the expression stands, so the order of work
is kept as written:

- the operands to the left of the call run before it;
- the right side of `ဖြစ်၍` and of `သို့မဟုတ်` (`&&` and `||`) runs just when the left side
  lets it, so a call there passes a failure on just when it is reached;
- the condition of a while loop runs again in each round, and a failure in it leaves the
  function from that round;
- the condition of an `ထိုသို့မဟုတ်ဘဲ` arm runs just when the arms before it did not hold.

```zaka
လုပ်ဆောင်ချက် စစ် သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
"စစ် {န}" ဟု ပြသပါ။
အကယ်၍ န သည် ၃ နှင့် ညီလျှင် "သုံး" ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
န ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် အလုပ် သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
အကယ်၍ န == ၀ && စစ်(န + ၁)? == ၁ ဖြစ်လျှင် "ညီ" ဟု ပြသပါ။
ကိန်းရှင် ရ သည် ၀ ဖြစ်ပါစေ။
စစ်(ရ)? သည် ၂ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ရ ကို ရ + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
စစ်(၃)? ကို ပြသပါ။
"မရောက်ပါ" ဟု ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် အလုပ်(၅) ဖြစ်ပါစေ။
အကယ်၍ ရ သည် အမှားအယွင်း ဖြစ်လျှင် ရ ကို အမှားအယွင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
စစ် ၀
စစ် ၁
စစ် ၂
စစ် ၃
သုံး
```

The left side of the `ဖြစ်၍` is false for 5, so `စစ်(၆)` never runs. The loop tests
`စစ်(ရ)` for 0, 1 and 2 and leaves it at 2. Then `စစ်(၃)` fails, and the call of `အလုပ်`
fails with `"သုံး"`.

### How the order is kept

A call that passes a failure on is lowered to statements that run before the statement
that holds it. Each operand on the left of such a call in the same expression is worked
out first, into a hidden place, so it keeps the written order whatever its kind. Text and
lists held there are copies, so a call that changes a variable on its left does not change
the value read before it. A plain read of an owner is left where it is: it moves the owner,
and the call cannot change that variable.

## The start block

The [start block](functions.md#the-start-block) is like `main` of other languages when it
returns an integer: its result is the exit code of the run, or nothing, and it cannot fail.

| Written on the start block | Is |
| --- | --- |
| a result kind | ZK0441: the result must be an integer |
| `?`, `ရလဒ်ကောင်း` or the call statement trailer | ZK0445: a pass-on where the code cannot fail |

Inside it, a failure is handled with what a function that cannot fail uses: the tests
`R သည် ရလဒ်ကောင်း` and `R သည် အမှားအယွင်း`, and the conversions
`R ကို ရလဒ်ကောင်း အဖြစ် ပြောင်းလဲသော ရလဒ်` and `R ကို အမှားအယွင်း အဖြစ် ပြောင်းလဲသော ရလဒ်`
of [Reading a whole result](#reading-a-whole-result). To pass a failure on, move the call into a
function that can fail, and keep its whole result in the start block. A start block that
gives back an integer sets the exit code as before. The top-level statements of a file
belong to no function either, so a pass-on there is ZK0445 too.

```zaka-error ZK0441
လုပ်ဆောင်ချက် စစ် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
၁ ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
စစ်()? ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0441]: the result of the start block is "isize result or text error", but it must be an integer
```

## Errors

| Code | Mistake |
| --- | --- |
| ZK0443 | a call statement to a function that can fail, with no pass-on |
| ZK0444 | a pass-on for a function that cannot fail |
| ZK0445 | a pass-on in a function that cannot fail, in the start block, or in top-level statements |
| ZK0446 | a pass-on whose error kind differs from that of the function around the call |
| ZK0447 | an error value given back from a function that cannot fail |
| ZK0448 | a test or a conversion of a value that is no result of a function that can fail |
| ZK0449 | a conversion to `ရလဒ်ကောင်း` of the result of a function with no good result |
| ZK0450 | a read of the error value of a result whose error side holds no value |
| ZK0451 | a failure with no value in a function whose error side names a kind |
| ZK0452 | an error value given back from a function whose error side holds no value |
| ZK0454 | a conversion to the part that a test ruled out |
| ZK0455 | a good result given back with the word of the other sort of function |

The codes ZK0450 to ZK0452 are on the [result kinds](#result-kinds) part, and ZK0454 and ZK0455
are on [Reading a whole result](#reading-a-whole-result), with an example each.

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

```zaka-error ZK0444
လုပ်ဆောင်ချက် တစ် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
၁ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် သုံး သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
တစ်()? ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0444]: the call passes a failure on, but "တစ်" cannot fail
```

```zaka-error ZK0445
လုပ်ဆောင်ချက် စစ် သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "သုည" ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
န ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် သုံး သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
စစ်(၁)? ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0445]: the call passes a failure on, but the code around it cannot fail
```

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

```zaka-error ZK0447
လုပ်ဆောင်ချက် စစ် သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
"သုည" ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0447]: "စစ်" cannot fail, so it has no error value to give back
```

## Open

- A block that takes the failure and goes on with its own code has no form of its own:
  [reading a whole result](#reading-a-whole-result) covers it, with a test in `အကယ်၍`
  and a conversion for each part.
- Zaka does not convert one error kind into another, so a pass-on needs the same error kind
  in both functions.

How a failing function is lowered is in [IR and code](../internals/codegen.md#the-ir).
