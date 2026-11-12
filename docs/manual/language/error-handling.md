---
title: Errors and results
description: A function that gives back a good value or an error value, the forms a caller passes a failure on in, how a caller reads a whole result, the result kinds, and the errors the compiler reports.
---

# Errors and results

A function can fail. Zaka writes a failure as a value: the function gives back either its
good value or an *error value*, and the call has to tell what happens to a failure: pass it
on, or keep the whole result and look at it. There are no exceptions and no jump to a distant
handler. This page states the rules in full. The
Learn book starts in small steps with [Errors and results](../../learn/error-handling/README.md).
A data cell that might contain a value, with no error at all, is an
[optional](optionals.md), a kind of its own with its own tests and reads.

```zaka
လုပ်ဆောင်ချက် စားခွဲ သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ ခ သည် ၀ နှင့် ညီလျှင် "သုညဖြင့် စား၍ မရပါ" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
က / ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပြ သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
စားခွဲ(က၊ ခ)? ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် ပြ(၁၀၊ ၂) ဖြစ်ပါစေ။
အကယ်၍ ရ ထဲတွင် ရလဒ်ကောင်း ရှိလျှင် ရ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၅
အောင်မြင်သည့်သတင်း
```

## The result clause

A function is marked as able to fail in its result clause, after `အောက်ပါတို့ကို လုပ်ဆောင်ကာ`:

| Clause | The function |
| --- | --- |
| `<kind> တစ်ခု ကို ပြန်ပေးမည်။` | cannot fail; gives back a value |
| `<result kind> တစ်ခု ကို ပြန်ပေးမည်။` | can fail; the [result kind](#result-kinds) tells the good side and the error side |
| `အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် <error kind> ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။` | can fail, and has no good value (the good side holds nothing) |

The word after `တစ်` is a measure word, and it has to fit the kind (ZK0509, a warning).
`ခု` fits a result kind. A function can fail when its result kind is a result kind, and not
otherwise, so the written result kind with `ကို ပြန်ပေးမည်` after it is the one header of a
function that can fail. No word `ရလဒ်` stands in a header.

The error kind can be whatever kind is needed: an integer, text, a struct, a list. A function
has no variable that holds its result: it gives the result back with a
[return](#giving-back-a-good-value) on each path.

The [start block](functions.md#the-start-block) cannot use these clauses: it cannot fail, and
its result is an integer, the exit code of the run, or nothing. A result kind on it is ZK0441 (see [The start block](#the-start-block)).

## Giving back a good value

```text
<value> ကို ပြန်ပေးမည်။
```

ends the function at once and gives the value back. In a function that can fail, a plain
value is the good side of the result: no word marks it. The value must fit the good side of
the clause (ZK0401).

A function whose good side holds no value gives back a good result that holds nothing with a
bare `ပြန်ပေးမည်။`, or when its block reaches its end. A function with a good value has no
such end: when a path of its block can reach the end with no return, the compiler reports
ZK0339. A path ends at a return, at an `အကယ်၍` with an `သို့မဟုတ်ပါက`, or at a switch with a
default, whose arms each end in one, or at a loop whose condition is `မှန်` and that no
break leaves, since it never ends. Each other loop is no such end, as its condition can be
false at the start.

```zaka-error ZK0339
လုပ်ဆောင်ချက် ရှာ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "သုည" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0339]: function "ရှာ" gives back a result, but its block can reach its end without giving one back
```

## Giving back an error value

```text
<value> ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
```

ends the function at once and gives back the value as the error value. The words
`<value> ဟူသော အမှားအယွင်း` make an *error value expression*: a value of the result kind that
the place wants, whose error side holds the value. It is no statement of its own, so it can be
given back or stored (`ရ ကို "အနုတ်" ဟူသော အမှားအယွင်း ဟု ထားမည်။`). The value must fit the
error kind of the result kind (ZK0401). The expression needs a place with a result kind that
has an error side that holds a value: beside a plain kind, as in the result clause of a
function that cannot fail, beside an optional, or beside a result whose error side is
`မအောင်မြင်သည့်သတင်း`, it is ZK0476.

```zaka-error ZK0476
လုပ်ဆောင်ချက် နှစ်ဆ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
"သုည" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0476]: an error value goes where a result kind with an error side is wanted
```

## Passing a failure on

When a caller meets a function that can fail, it has to tell what happens to a failure.
One answer is to *pass the failure on*; the other, to keep the whole result, is in
[Reading a whole result](#reading-a-whole-result). When the callee fails, the function around the call stops at once and gives back the
same error value; when the callee gives back a good value, the call goes on with it.
This is the `?` of Rust. There are three forms:

| Form | Written | Is |
| --- | --- | --- |
| word form | `လုပ်ဆောင်ချက် <name> ကို <arguments> တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်ကောင်း` | a value: the good value |
| sign form | `<name>(<arguments>)?` | a value: the good value; the `?` touches the `)` |
| call statement | `လုပ်ဆောင်ချက် <name> ကို <arguments> တို့ဖြင့် လုပ်ဆောင်မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။` | a statement; a good value is dropped |

`ရလဒ်ကောင်း` means "the good result". The word form and the sign form are the same call, as
the two spellings of a plain call are; the same checks and the same codes hold.

```zaka
လုပ်ဆောင်ချက် စစ် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ အောက် ငယ်လျှင် "အနုတ်" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ရေး သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် စစ် ကို န တို့ဖြင့် လုပ်ဆောင်မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။
"ရေး {န}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် စုံ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
(လုပ်ဆောင်ချက် စစ် ကို န တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်ကောင်း) + စစ်(န + ၁)? ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် အားလုံး သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် ရေး ကို ၄ တို့ဖြင့် လုပ်ဆောင်မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။
စုံ(၃)? ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် အားလုံး() ဖြစ်ပါစေ။
အကယ်၍ ရ ထဲတွင် အမှားအယွင်း ရှိလျှင် ရ ထဲမှ အမှားအယွင်း ဒေတာ ကို ပြသမည်။
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
fail, is ZK0444. An [optional](optionals.md) is no failure: `?` on a call that gives back one is
ZK0444 as well. A function whose good side holds no value has no good value to read: `ရလဒ်ကောင်း`
or `?` on it, used as a value, is ZK0315, as for a value of each function with no result.

## Reading a whole result

A call to a function that can fail, written with no `ရလဒ်ကောင်း` and no `?`, passes nothing
on: it gives back the *whole result*, a good value or an error value. The call is an
expression like each other call; just the call *statement* needs the pass-on, because
there the result would be dropped (ZK0443).

```text
ကိန်းရှင် R သည် လုပ်ဆောင်ချက် <name> ကို <arguments> တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် R သည် <name>(<arguments>) ဖြစ်ပါစေ။
R ကို <name>(<arguments>) ဟု ထားမည်။
```

The kind of a whole result can be [written](#result-kinds), or taken from the value in
a declaration with a starting value; `ထားမည်` stores another result of the same kinds (the
same good side and the same error side). In a message the kind reads like
`isize result or text error`, `success or text error` and `isize result or failure`. A whole
result is no number: using it as one is ZK0403.

`ပြသမည်` and a `{...}` slot write a whole result as the part it holds:
`ရလဒ်ကောင်း(<value>)` or `အမှားအယွင်း(<value>)`. A side with no value is written as its
news, `အောင်မြင်သည့်သတင်း` or `မအောင်မြင်သည့်သတင်း`, with no brackets.

```zaka
လုပ်ဆောင်ချက် ပိတ် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "မပိတ်နိုင်ပါ" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပိတ်(၀) ကို ပြသမည်။
ကိန်းရှင် ရ သည် ပိတ်(၇) ဖြစ်ပါစေ။
"ရ = {ရ}" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အမှားအယွင်း("မပိတ်နိုင်ပါ")
ရ = ရလဒ်ကောင်း(၇)
```

### Tests

| Written | Is |
| --- | --- |
| `<R> ထဲတွင် ရလဒ်ကောင်း ရှိ` | true when `R` holds a good value |
| `<R> ထဲတွင် အမှားအယွင်း ရှိ` | true when `R` holds an error value |

`အောင်မြင်သည့်သတင်း` stands in place of `ရလဒ်ကောင်း`, and `မအောင်မြင်သည့်သတင်း` in place of
`အမှားအယွင်း`, with the same meaning. `ရှိ` can take `နေ` (`ရှိနေ`) as it can in the optional's
tests, and `လျှင်` or `သရွေ့` can follow it fused: `ရှိလျှင်`, `ရှိနေလျှင်`, `ရှိနေသရွေ့`. A
result has no `မရှိ`: to ask the opposite, test the other part, or put `မဟုတ်` after the
test (`ရှိ မဟုတ်လျှင်`).

Each test is a true or false value, so it can stand wherever one can: in `အကယ်၍` and
`ထိုသို့မဟုတ်ဘဲ`, in the condition of a loop, beside `ဖြစ်၍` and `သို့မဟုတ်` (`&&` and `||`), and
as the operand of a print or a declaration: `ကိန်းရှင် ခ သည် R ထဲတွင် အမှားအယွင်း ရှိ ဖြစ်ပါစေ။`
makes a true or false value. The operand of a test can be a variable, or a call of a function
that can fail.

A test word of the other kind is ZK0475: `ဒေတာ` on a result, or `ရလဒ်ကောင်း` or `အမှားအယွင်း` on
an optional. A test on a value that is neither is ZK0448.

```zaka-error ZK0475
လုပ်ဆောင်ချက် ရှာ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
န * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် R သည် ရှာ(၅) ဖြစ်ပါစေ။
အကယ်၍ R ထဲတွင် ဒေတာ ရှိလျှင် "ဒေတာ" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0475]: "isize result or text error" is no optional: test a result with "ထဲတွင် ရလဒ်ကောင်း ရှိ" or "ထဲတွင် အမှားအယွင်း ရှိ"
```

### Reads

| Written | Is |
| --- | --- |
| `<R> ထဲမှ ဒေတာ` | the good value, as `unwrap` of Rust gives it |
| `<R> ထဲမှ အမှားအယွင်း ဒေတာ` | the error value, as `unwrap_err` gives it |

`အချက်အလက်` can stand in place of `ဒေတာ`, as it does in the tests. A read of the part that is
not there stops the run with a VM trap, as a conversion that cannot be made does:

| Read | Held | Trap | The run stops with |
| --- | --- | --- | --- |
| the good value | an error value | `not_good` | `the result holds an error, not a good result: V` |
| the error value | a good value | `not_fault` | `the result is a good result, not an error` |

The good value of a result that holds a failure with no value stops the run with `the result holds a failure, not a good result`.
The read of the data of an [optional](optionals.md) that holds nothing stops it with its own trap.

```zaka
လုပ်ဆောင်ချက် စစ် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
"စစ် {န}" ဟု ပြသမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "သုည" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ရေး သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "ရေး၍ မရ" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် စစ်(၅) ဖြစ်ပါစေ။
အကယ်၍ ရ ထဲတွင် ရလဒ်ကောင်း ရှိလျှင် ရ ထဲမှ ဒေတာ ကို ပြသမည်။
ရ ကို စစ်(၀) ဟု ထားမည်။
အကယ်၍ ရ ထဲတွင် အမှားအယွင်း ရှိလျှင် ရ ထဲမှ အမှားအယွင်း ဒေတာ ကို ပြသမည်။
အကယ်၍ စစ်(၃) ထဲတွင် ရလဒ်ကောင်း ရှိ ဖြစ်၍ စစ်(၀) ထဲတွင် အမှားအယွင်း ရှိလျှင် "နှစ်ခုလုံး" ဟု ပြသမည်။
အကယ်၍ စစ်(၀) ထဲတွင် ရလဒ်ကောင်း ရှိ ဖြစ်၍ စစ်(၇) ထဲတွင် ရလဒ်ကောင်း ရှိလျှင် "မရောက်ပါ" ဟု ပြသမည်။
ကိန်းရှင် ဂ သည် ရေး(၀) ဖြစ်ပါစေ။
အကယ်၍ ဂ ထဲတွင် အမှားအယွင်း ရှိလျှင် ဂ ထဲမှ အမှားအယွင်း ဒေတာ ကို ပြသမည်။
ဂ ကို ရေး(၁) ဟု ထားမည်။
အကယ်၍ ဂ ထဲတွင် ရလဒ်ကောင်း ရှိလျှင် "ရေးပြီး" ဟု ပြသမည်။
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
the right side of `ဖြစ်၍` is not reached. The function `ရေး` has no good value, so its whole
result holds an error value or nothing: `ဂ ထဲတွင် ရလဒ်ကောင်း ရှိ` is true when it did not fail, and
there is no good value to read from it.

```zaka-stop
လုပ်ဆောင်ချက် စစ် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "သုည" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် စစ်(၀) ဖြစ်ပါစေ။
"မစမီ" ဟု ပြသမည်။
ရ ထဲမှ ဒေတာ ကို ပြသမည်။
"ဤနေရာ မရောက်ပါ" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မစမီ
the program stopped: the result holds an error, not a good result: သုည
```

```zaka-stop
လုပ်ဆောင်ချက် စစ် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "သုည" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် စစ်(၄) ဖြစ်ပါစေ။
ရ ထဲမှ အမှားအယွင်း ဒေတာ ကို ပြသမည်။
"ဤနေရာ မရောက်ပါ" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: the result is a good result, not an error
```

### Order of work

A read runs where it is written, as a pass-on call does: the operands on its left run
first, and the right side of `&&` and `||` runs just when it is reached. So a read of the
wrong part stops the run at that place, and nothing to its right runs.

### No initial value

A result holds its good value or its failure, and the program tells which: a result kind
has no [initial value](kinds.md#initial-values), since a good value no call gave would
claim a success that did not happen, and a failure no call gave would claim one that did
not happen either. `ကနဦးတန်ဖိုး` for a result kind, or for a struct or a counted list that
holds one, is ZK0479. A variable of a result kind declared with no value is unset until the
program sets it ([Set before read](statements.md#set-before-read)), and a struct literal
names each result field (ZK0482).

```zaka-error ZK0479
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် R သည် ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0479]: "isize result or text error" has no initial value: a result must hold its good value or its failure
```

A fresh heap block of results, in [marked code](unsafe.md), starts at zero octets, as each
heap block does. Read as a result, those octets are a failure that holds the zero value of
the error side, or the bare failure when the error side is `မအောင်မြင်သည့်သတင်း`, so even
there no cell claims a success:

```zaka
ဖတ်ချက် ဟူသော ဒေတာအစုအဝေးတွင် တွေ့ချက် ဟူသော ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု၊ အကြိမ် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် ဖတ်ချက်(တွေ့ချက်: ၇၊ အကြိမ်: ၁) ဖြစ်ပါစေ။
ဖ ကို ပြသမည်။
// a heap block needs marked code
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ဘူး သည် ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ဘူး တွင် သိမ်းထားမည်။
ဘူး ၏ တန်ဖိုး ကို ပြသမည်။
လိပ်စာ ဘူး ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဖတ်ချက်(တွေ့ချက်: ရလဒ်ကောင်း(၇)၊ အကြိမ်: ၁)
အမှားအယွင်း("")
```

A plain list of results starts with no cells, `[]`, and `ထည့်မည်` adds each cell. With the
error side `မအောင်မြင်သည့်သတင်း` a zeroed heap cell prints as `မအောင်မြင်သည့်သတင်း`. Write
a call as the starting value, as `ကိန်းရှင် R သည် ရှာ(၄) ဖြစ်ပါစေ။`, when the variable has to
hold the result of that call. (An [optional](optionals.md) differs: its initial value is the
blank value `အလွတ်`, since an optional claims no success.)

### A part the test ruled out

A test proves which part a result variable holds, for as long as nothing changes the
variable. A read of the part the test ruled out would stop the run each time it ran,
so the compiler reports it as ZK0454 and does not wait for the run. The message tells
which part is held: `"R" holds a good result here, so reading its error value stops the run each time`
or `"R" holds a failure here, so reading its good result stops the run each time`.

```zaka-error ZK0454
လုပ်ဆောင်ချက် ရှာ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
န * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် R သည် ရှာ(၄) ဖြစ်ပါစေ။
အကယ်၍ R ထဲတွင် ရလဒ်ကောင်း ရှိလျှင် R ထဲမှ အမှားအယွင်း ဒေတာ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0454]: "R" holds a good result here, so reading its error value stops the run each time
```

What proves a part:

- Inside the arm of `အကယ်၍ R ထဲတွင် ရလဒ်ကောင်း ရှိလျှင်`, `R` holds a good value; inside the
  arm of `အကယ်၍ R ထဲတွင် အမှားအယွင်း ရှိလျှင်`, it holds a failure.
- The arms after an arm (each `ထိုသို့မဟုတ်ဘဲ` and the `သို့မဟုတ်ပါက`) get the opposite
  of the tests before them.
- In the condition of an arm, `ဖြစ်၍` (`&&`) proves both sides, `သို့မဟုတ်` (`||`) proves
  both sides false in the arms after it, and `မဟုတ်` flips what its clause proves.
- An `အကယ်၍` arm that is the whole statement, with no arm after it, and whose block ends
  with `ပြန်ပေးမည်`, `ရပ်မည်` or `ကျော်မည်`, proves the opposite after the whole statement: a
  guard. After `အကယ်၍ R ထဲတွင် အမှားအယွင်း ရှိလျှင် ပြန်ပေးမည်`, `R` holds a good value.
- The condition of a while loop proves its test inside the loop.

What ends a proof:

- an assignment to `R`;
- `R` given to a parameter marked `ဒေတာ ပြောင်းလဲရန်`;
- the address of `R`;
- a new declaration of the same name;
- a loop that can change `R` in its statements, nested blocks included: for a proof made outside the
  loop, the whole loop is without it. A test made inside the loop still proves its own arm.

When the part cannot be proven, nothing is reported, and the read is checked when the run
reaches it: it stops with `not_good` or `not_fault`, as above.

### Mistakes

| Code | Mistake |
| --- | --- |
| ZK0448 | a test of a value that is neither a result nor an optional |
| ZK0449 | a read of the good value of the result of a function with no good value |
| ZK0454 | a read of the part that a test ruled out |
| ZK0475 | a test word of the other kind: `ဒေတာ` on a result, or `ရလဒ်ကောင်း` or `အမှားအယွင်း` on an optional |

ZK0454 and ZK0475 are on the parts above, with an example each.

```zaka-error ZK0448
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ၅ ဖြစ်ပါစေ။
အကယ်၍ ပ ထဲတွင် အမှားအယွင်း ရှိလျှင် "မဟုတ်" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0448]: the value is "isize", not the result of a function that can fail
```

```zaka-error ZK0449
လုပ်ဆောင်ချက် ရေး သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် ရေး() ဖြစ်ပါစေ။
ရ ထဲမှ ဒေတာ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0449]: the function gives back no good value, so the result has none to read
```

## Result kinds

A result kind can be written wherever a kind goes: in a declaration, a parameter, a field of a
struct and the result clause of a function. It names the good side and the error side:

```text
<kind> သို့မဟုတ် အမှားအယွင်းအနေဖြင့် <kind> ရှိနိုင်သော ဒေတာဆဲလ်
```

The good side is a kind, or `အောင်မြင်သည့်သတင်း` when it holds no value. The error side is
`အမှားအယွင်းအနေဖြင့်` and a kind, or `မအောင်မြင်သည့်သတင်း` in the place of both words when it
holds no value. `ရှိနိုင်သော ဒေတာဆဲလ်` ends the kind: the cell can hold either side.

| Written | Good side | Error side |
| --- | --- | --- |
| `<T> သို့မဟုတ် အမှားအယွင်းအနေဖြင့် <E> ရှိနိုင်သော ဒေတာဆဲလ်` | a value of `T` | a value of `E` |
| `အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် <E> ရှိနိုင်သော ဒေတာဆဲလ်` | no value | a value of `E` |
| `<T> သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ်` | a value of `T` | no value |
| `အောင်မြင်သည့်သတင်း သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ်` | no value | no value |

The first three are `Result<T, E>`, `Result<(), E>` and `Result<T, ()>`, as Rust writes
them. A function can fail when its result kind is a result kind, and just then.
In a message the kinds read like `isize result or text error`, `success or text error` and
`isize result or failure`. The kind of a variable, a parameter or a field is written in the same
form, and `တစ်ခု` follows it in a declaration, as it follows each kind.

A literal takes the good kind of a result kind: the value `၁` of a variable of
`၈ ဘစ် ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ်` is an
`၈ ဘစ် ကိန်းပြည့်`, and a literal that does not fit it is ZK0406.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖတ်ရလဒ် သည် ၈ ဘစ် ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
ဖတ်ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရလဒ်ကောင်း(၁)
```

```zaka
ဖိုင်ရလဒ် ဟူသော ဒေတာအစုအဝေးတွင် အရွယ် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။
ဖိုင်အမှားအယွင်း ဟူသော ဒေတာအစုအဝေးတွင် စာ ဟူသော စာသား တစ်ခု ပါဝင်သည်။
ဖတ်ချက် ဟူသော ဒေတာအစုအဝေးတွင် တွေ့ချက် ဟူသော ဖိုင်ရလဒ် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု၊ အကြိမ် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

လုပ်ဆောင်ချက် ဖွင့် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ဖိုင်ရလဒ် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် ဖိုင်အမှားအယွင်း(စာ: "မရှိ") ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
ဖိုင်ရလဒ်(အရွယ်: န * ၁၀၀) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

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

လုပ်ဆောင်ချက် ပြ သည်
R ဟူသော ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
R ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ဆင့် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
ရှာ(၀ - ၁)? ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် ဖတ်ချက် တစ်ခု ဖြစ်ပါစေ။
ဖ ၏ တွေ့ချက် ကို ဖွင့်(၃) ဟု ထားမည်။
ဖ ၏ တွေ့ချက် ကို ပြသမည်။
ဖွင့်(၀) ကို ပြသမည်။
ကိန်းရှင် R သည် ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ရှာ(၄) ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ပြ ကို R တို့ဖြင့် လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ပြ ကို ရှာ(၀ - ၂) တို့ဖြင့် လုပ်ဆောင်မည်။
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

### Lists and pointers of results

A result kind can take list and pointer levels after `ဒေတာဆဲလ်`; the levels wrap the *whole*
result, not the error side:

| Written | Is |
| --- | --- |
| `ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် စာရင်း` | a plain list of results |
| `အကွက် ၂ ခုရှိသော ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် စာရင်း` | a list of 2 results |
| `ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် လိပ်စာ` | a pointer to a result, for [marked code](unsafe.md) |

A count prefix (`အကွက် ၂ ခုရှိသော`) written before a result kind belongs to its levels.
The kinds serve a variable, a parameter, a field of a struct and the cells of a heap block
taken with `မင်မိုရီတွင်`. A list with a count of results takes a literal that names each
cell, since a result has no initial value; a cell added with `ထည့်မည်` holds the result
given. A list prints its cells in brackets, each as a whole
result, and a cell is read with `အကွက်တိုင်း` or a range as for each list, then tested and read as in [Reading a whole result](#reading-a-whole-result).

```zaka
ဖိုင်အမှားအယွင်း ဟူသော ဒေတာအစုအဝေးတွင် စာ ဟူသော စာသား တစ်ခု ပါဝင်သည်။
ဖတ်ချက် ဟူသော ဒေတာအစုအဝေးတွင် တွေ့ချက်များ ဟူသော ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် စာရင်း တစ်ခု၊ အကြိမ် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

လုပ်ဆောင်ချက် ရှာ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ အောက် ငယ်လျှင် ဖိုင်အမှားအယွင်း(စာ: "အနုတ်") ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပြ သည်
အပိုင်း ဟူသော ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အပိုင်း ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

စိတ်မချရသော လုပ်ဆောင်ချက် ဖတ် သည်
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
လုပ်ဆောင်ချက် ပြ ကို ရများ တို့ဖြင့် လုပ်ဆောင်မည်။
ကိန်းရှင် အလယ် သည် ရများ ၏ အကွက် ၁ မှစ၍ ၂ ကွက် ဖြစ်ပါစေ။
အလယ် ကို ပြသမည်။

ကိန်းရှင် နှစ်ကွက် သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [ရှာ(၁)၊ ရှာ(၀ - ၁)] ဖြစ်ပါစေ။
နှစ်ကွက် ကို ပြသမည်။

အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ဘူး သည် ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ဘူး တွင် သိမ်းထားမည်။
ဘူး ၏ တန်ဖိုး ကို ရှာ(၀ - ၅) ဟု ထားမည်။
လုပ်ဆောင်ချက် ဖတ် ကို ဘူး တို့ဖြင့် လုပ်ဆောင်မည်။
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

The levels come after `ဒေတာဆဲလ်`, so it is plain which part the levels wrap. A list as the
error kind goes before it: `အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း စာရင်း ရှိနိုင်သော ဒေတာဆဲလ်`, so a
single `ဖိုင်အမှားအယွင်း` does not fit it:

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

### Marking the end of a function

| Statement | Ends the function with | For a result kind with |
| --- | --- | --- |
| `လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်။` | a good result that holds no value, as a bare `ပြန်ပေးမည်။` does | `အောင်မြင်သည့်သတင်း` |
| `လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။` | a failure that holds no error value | `မအောင်မြင်သည့်သတင်း` |

Both end the function at once, as `ပြန်ပေးမည်` does. The second needs an error side with
no value: in a function whose error side names a kind it is ZK0451. When the error side holds
no value, there is no place for an error value, so `<value> ဟူသော အမှားအယွင်း` is ZK0476.

### Passing on and reading

`?`, `ရလဒ်ကောင်း` and the statement trailer work with each result kind. For ZK0446, an
error side with no value is an error kind of its own, so `မအောင်မြင်သည့်သတင်း` and `စာသား` differ. The
tests and reads of [Reading a whole result](#reading-a-whole-result) work with each kind. A read of the error value
of a result whose error side holds no value is ZK0450.

A side with no value prints as `အောင်မြင်သည့်သတင်း` or `မအောင်မြင်သည့်သတင်း`. When the good
value is read from a result that holds a failure with no value, the VM traps with
`the result holds a failure, not a good result`; with a value the message is
`the result holds an error, not a good result: V`
(see [When a program stops](../vm/traps.md#a-whole-result-read-as-the-wrong-part)).

### Mistakes

| Code | Mistake |
| --- | --- |
| ZK0450 | a read of the error value of a result whose error side holds no value |
| ZK0451 | a failure with no value in a function whose error side names a kind |

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
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
"စစ် {န}" ဟု ပြသမည်။
အကယ်၍ န သည် ၃ နှင့် ညီလျှင် "သုံး" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် အလုပ် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န == ၀ && စစ်(န + ၁)? == ၁ ဖြစ်လျှင် "ညီ" ဟု ပြသမည်။
ကိန်းရှင် ရ သည် ၀ ဖြစ်ပါစေ။
စစ်(ရ)? သည် ၂ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ရ ကို ရ + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
စစ်(၃)? ကို ပြသမည်။
"မရောက်ပါ" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် အလုပ်(၅) ဖြစ်ပါစေ။
အကယ်၍ ရ ထဲတွင် အမှားအယွင်း ရှိလျှင် ရ ထဲမှ အမှားအယွင်း ဒေတာ ကို ပြသမည်။
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
the value read before it.

## The start block

The [start block](functions.md#the-start-block) is like `main` of other languages when it
returns an integer: its result is the exit code of the run, or nothing, and it cannot fail.

| Written on the start block | Is |
| --- | --- |
| a result kind | ZK0441: the result must be an integer |
| `?`, `ရလဒ်ကောင်း` or the call statement trailer | ZK0445: a pass-on where the code cannot fail |

Inside it, a failure is handled with what a function that cannot fail uses: the tests
`R ထဲတွင် ရလဒ်ကောင်း ရှိ` and `R ထဲတွင် အမှားအယွင်း ရှိ`, and the reads
`R ထဲမှ ဒေတာ` and `R ထဲမှ အမှားအယွင်း ဒေတာ`
of [Reading a whole result](#reading-a-whole-result). To pass a failure on, move the call into a
function that can fail, and keep its whole result in the start block. A start block that
gives back an integer sets the exit code as before. The top-level statements of a file
belong to no function either, so a pass-on there is ZK0445 too.

```zaka-error ZK0441
လုပ်ဆောင်ချက် စစ် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
၁ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
စစ်()? ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0441]: the result of the start block is "isize result or text error", but it must be an integer
```

## Errors

| Code | Mistake |
| --- | --- |
| ZK0339 | a function with a result whose block can reach its end with no return |
| ZK0443 | a call statement to a function that can fail, with no pass-on |
| ZK0444 | a pass-on for a function that cannot fail, or for one that gives back an optional |
| ZK0445 | a pass-on in a function that cannot fail, in the start block, or in top-level statements |
| ZK0446 | a pass-on whose error kind differs from that of the function around the call |
| ZK0448 | a test of a value that is neither a result nor an optional |
| ZK0449 | a read of the good value of the result of a function with no good value |
| ZK0450 | a read of the error value of a result whose error side holds no value |
| ZK0451 | a failure with no value in a function whose error side names a kind |
| ZK0454 | a read of the part that a test ruled out |
| ZK0475 | a test word of the other kind: `ဒေတာ` on a result, or `ရလဒ်ကောင်း` or `အမှားအယွင်း` on an optional |
| ZK0476 | `<value> ဟူသော အမှားအယွင်း` where no result kind with an error side is wanted |

The codes ZK0450 and ZK0451 are on the [result kinds](#result-kinds) part, ZK0454 and ZK0475
are on [Reading a whole result](#reading-a-whole-result), and ZK0339 and ZK0476 are on the parts
above, with an example each. ZK0477, `အလွတ်` where no optional is wanted, is on the page of
[optionals](optionals.md).

```zaka-error ZK0443
လုပ်ဆောင်ချက် စစ် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "သုည" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် စစ် ကို ၁ တို့ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0443]: "စစ်" can fail, but the call does not pass the failure on
```

```zaka-error ZK0444
လုပ်ဆောင်ချက် တစ် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
၁ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် သုံး သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
တစ်()? ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0444]: the call passes a failure on, but "တစ်" cannot fail
```

```zaka-error ZK0445
လုပ်ဆောင်ချက် စစ် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "သုည" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် သုံး သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
စစ်(၁)? ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0445]: the call passes a failure on, but the code around it cannot fail
```

```zaka-error ZK0446
လုပ်ဆောင်ချက် စစ် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "သုည" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် သုံး သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ကိန်းပြည့် ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
စစ်(၁)? ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0446]: the called function fails with "text", but the function around the call fails with "isize"
```

## Open

- A block that takes the failure and goes on with its own code has no form of its own:
  [reading a whole result](#reading-a-whole-result) covers it, with a test in `အကယ်၍`
  and a read for each part.
- Zaka does not convert one error kind into another, so a pass-on needs the same error kind
  in both functions.

How a failing function is lowered is in [IR and code](../internals/codegen.md#the-ir).
