---
title: Errors and results
description: A function that returns a good result or an error value, the forms that pass a failure on to the caller, the order of work, and the errors the compiler reports.
---

# Errors and results

A function that can fail returns either a good result (`Ok` value) or an error value
(error). At the call site, the caller can pass the error back to its own caller or keep
the whole result and inspect it. For examples, see [Errors and
results](../../learn/error-handling/README.md) in Learn.

The rules are on three pages. This page covers how a function fails, how a caller passes a
failure on and the order in which those calls run. [Reading a result](reading-results.md)
covers how a caller keeps a whole result and tests and reads it, and [Result
kinds](result-kinds.md) covers how to write the result kind (result type) of a variable, a
parameter or a field. A data cell that might contain a value, with no error at all, is an
[optional](optionals.md), which is a different data kind (type) with its own tests and
reads.

```zaka
စားခွဲ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ ခ သည် ၀ နှင့် ညီလျှင် "သုညဖြင့် စား၍ မရပါ" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
က / ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပြ ဟူသော လုပ်ဆောင်ချက် သည်
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

```text
… အောက်ပါတို့ကို လုပ်ဆောင်ကာ <result kind> တစ်ခု ကို ပြန်ပေးမည်။
```

To show that a function can fail, we write a [result kind](result-kinds.md) in the result
clause of its function signature, after `အောက်ပါတို့ကို လုပ်ဆောင်ကာ`, and in the result
kind we write the good side and the error side:

| Result clause | The function |
| --- | --- |
| `<kind> တစ်ခု ကို ပြန်ပေးမည်။` | cannot fail, and returns a value |
| `<good kind> သို့မဟုတ် အမှားအယွင်းအနေဖြင့် <error kind> ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။` | can fail; it returns a good result of `<good kind>` or an error value of `<error kind>` |
| `အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် <error kind> ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။` | can fail, and has no good value: the good side contains nothing |

A function can fail when its result is a result kind, and at no other time, so a result
kind followed with `ကို ပြန်ပေးမည်` is the one function signature of a function that can
fail. The word `ရလဒ်` does not appear in a function signature. The word after `တစ်` is a
measure word (classifier), and it must fit the data kind (the warning ZK0509); `ခု` fits a
result kind.

The error kind can be whatever data kind we need: an integer, text, a struct or a list. A
function has no variable for its result, so it returns the result with a
[return](#giving-back-a-good-value) on each path.

The [program start block](#the-program-start-block) cannot use these clauses, because it
cannot fail: its result is an integer, the exit code of the run, or nothing. A result kind
on it is ZK0441.

## Giving back a good value

```text
<value> ကို ပြန်ပေးမည်။
```

We use this statement to end the function at once and return the value. In a function that
can fail, a plain value is the good side of the result, so no word marks it as good. The
value must fit the good side of the result clause (ZK0401).

When the good side has no value, a bare `ပြန်ပေးမည်။` returns a good result that contains
nothing, and so does reaching the end of the block. A function with a good value cannot
end like that: when a path of its block can reach the end with no return, the compiler
reports ZK0339. A path ends with a return when it ends at one of these:

- a return statement;
- an `အကယ်၍` with a `သို့မဟုတ်ပါက`, or a switch with a default, when each of their arms
  ends with a return;
- a loop whose condition is `မှန်` and that no break leaves, since it never ends.

Other loops do not end a path, because their condition can be false at the start.

```zaka-error ZK0339
ရှာ ဟူသော လုပ်ဆောင်ချက် သည်
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

We use this statement to end the function at once and return the value as its error value.
The words `<value> ဟူသော အမှားအယွင်း` make an *error value expression*: a value of the
result kind that the place expects, whose error side contains the value. It is an
expression and not a statement, so we can also store it, as in
`ရ ကို "အနုတ်" ဟူသော အမှားအယွင်း ဟု ထားမည်။`. The value must fit the error kind of the
result kind (ZK0401).

The expression needs a place whose result kind has an error side with a value. Where a
plain data kind is expected, as in the result clause of a function that cannot fail, where
an optional is expected, or where the error side of the result is
`မအောင်မြင်သည့်သတင်း`, it is ZK0476.

```zaka-error ZK0476
နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် သည်
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

A result kind whose error side has no value fails with
`လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။` instead ([Ending a function with no
value](result-kinds.md#ending-a-function-with-no-value)).

## Passing a failure on

When a function calls a function that can fail, the call must show what happens to a
failure. One choice is to *pass the failure on* (error propagation), which is the `?` of
Rust: when the called function fails, the function around the call stops at once and
returns the same error value, and when the called function returns a good result, the call
goes on with that value. The other choice is to keep the whole result, which [Reading a
result](reading-results.md) covers.

We can pass a failure on in three forms:

```text
<name> ဟူသော လုပ်ဆောင်ချက်ကို <arguments> တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်ကောင်း
<name>(<arguments>)?
<name> ဟူသော လုပ်ဆောင်ချက်ကို <arguments> တို့ဖြင့် လုပ်ဆောင်မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။
```

| Form | Written | Result |
| --- | --- | --- |
| word form | `… လုပ်ဆောင်သော ရလဒ်ကောင်း` | a value: the good result |
| sign form | `<name>(<arguments>)?`, with the `?` right after the `)` | a value: the good result |
| call statement | `… လုပ်ဆောင်မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။` | a statement; it drops a good value |

`ရလဒ်ကောင်း` means "the good result". The word form and the sign form are the same call
written in two forms, just like the two spellings of a plain call, so the same checks and
the same codes hold for both.

```zaka
စစ် ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ အောက် ငယ်လျှင် "အနုတ်" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ရေး ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
စစ် ဟူသော လုပ်ဆောင်ချက်ကို န ဖြင့် လုပ်ဆောင်မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။
"ရေး {န}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

စုံ ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
(စစ် ဟူသော လုပ်ဆောင်ချက်ကို န ဖြင့် လုပ်ဆောင်သော ရလဒ်ကောင်း) + စစ်(န + ၁)? ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

အားလုံး ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
ရေး ဟူသော လုပ်ဆောင်ချက်ကို ၄ ဖြင့် လုပ်ဆောင်မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။
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

A pass-on call in the word form or the sign form is a value like other values, so we can
use it wherever an expression fits: in a sum, as an argument, in a condition, in a
declaration or after a return.

### Where a pass-on fits

A pass-on needs a function around it that can fail with the same error kind, because the
error value goes out as it is and Zaka does not convert one error kind into another.

| Case | Code |
| --- | --- |
| a pass-on in a function that cannot fail, in the [program start block](#the-program-start-block), or in the top-level statements of a file | ZK0445 |
| a pass-on whose error kind differs from the error kind of the function around it | ZK0446 |
| a call statement to a function that can fail, with no pass-on, so a failure is never dropped without a word | ZK0443 |
| a pass-on for a function that cannot fail, or for one that returns an [optional](optionals.md), which is no failure | ZK0444 |
| `ရလဒ်ကောင်း` or `?` used as a value, on a function whose good side has no value, as for each function with no result | ZK0315 |

The pass-on also works on a call through a [function value](function-values.md) whose
function kind can fail, and with each result kind, as [Passing on and
reading](result-kinds.md#passing-on-and-reading) states.

## Order of work

A call that passes a failure on, and the test that goes with it, run at the place where the
expression is, so the program keeps the order in which the code is written:

- the operands to the left of the call run before it;
- the right side of `ဖြစ်၍` and `သို့မဟုတ်` (`&&` and `||`) runs just when the left side
  lets it, so a call there passes a failure on just when the program reaches it;
- the condition of a while loop runs again in each round, and a failure in it leaves the
  function from that round;
- the condition of an `ထိုသို့မဟုတ်ဘဲ` arm runs just when the arms before it did not
  match.

```zaka
စစ် ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
"စစ် {န}" ဟု ပြသမည်။
အကယ်၍ န သည် ၃ နှင့် ညီလျှင် "သုံး" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

အလုပ် ဟူသော လုပ်ဆောင်ချက် သည်
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

The left side of the `&&` is false for 5, so `စစ်(၆)` never runs. The loop tests
`စစ်(ရ)` for 0, 1 and 2 and leaves at 2. Then `စစ်(၃)` fails, so the call of `အလုပ်`
fails with `"သုံး"`.

### How the order is kept

The compiler lowers a call that passes a failure on to statements that run before the
statement that contains it. It first works out each operand to the left of that call in
the same expression and puts it in a hidden place, so the operand keeps the written order
whatever its data kind. Text and lists in such a place are duplicates (copies), so a call
that changes a variable on its left does not change the value that was read before it.

## The program start block

The [program start block](functions-start-block.md) is like `main` in other languages
when it returns an integer: its result is the exit code of the run, or nothing, and it
cannot fail.

| Written in the program start block | Code |
| --- | --- |
| a result kind as its result | ZK0441: the result must be an integer |
| `?`, `ရလဒ်ကောင်း` or the pass-on call statement | ZK0445: a pass-on where the code cannot fail |

Inside the program start block, we handle a failure just as we do in a function
that cannot fail: we keep the whole result, test it with `R ထဲတွင် ရလဒ်ကောင်း ရှိ` or
`R ထဲတွင် အမှားအယွင်း ရှိ`, and read it with `R ထဲမှ ဒေတာ` or
`R ထဲမှ အမှားအယွင်း ဒေတာ` ([Reading a result](reading-results.md)). To pass a failure on,
we move the call into a function that can fail and keep the whole result of that function
in the program start block. A program start block that returns an integer sets the exit
code as before. The top-level statements of a file are not part of a function either, so a
pass-on there is ZK0445 too.

```zaka-error ZK0441
စစ် ဟူသော လုပ်ဆောင်ချက် သည်
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

These are the codes of errors and results on all three pages:

| Code | Mistake | Shown on |
| --- | --- | --- |
| ZK0339 | a function with a result whose block can reach its end with no return | [Giving back a good value](#giving-back-a-good-value) |
| ZK0441 | a result kind on the program start block | [The program start block](#the-program-start-block) |
| ZK0443 | a call statement to a function that can fail, with no pass-on | below |
| ZK0444 | a pass-on for a function that cannot fail, or for one that returns an optional | below |
| ZK0445 | a pass-on in a function that cannot fail, in the program start block, or in top-level statements | below |
| ZK0446 | a pass-on whose error kind differs from the error kind of the function around the call | below |
| ZK0447 | a failure with no value in a function that cannot fail | [Result kinds](result-kinds.md#ending-a-function-with-no-value) |
| ZK0448 | a test or a read of a value that is neither a result nor an optional | [Reading a result](reading-results.md#errors) |
| ZK0449 | a read of the good result of a function with no good value | [Reading a result](reading-results.md#errors) |
| ZK0450 | a read of the error value of a result whose error side has no value | [Result kinds](result-kinds.md#errors) |
| ZK0451 | a failure with no value in a function whose error side has a data kind | [Result kinds](result-kinds.md#errors) |
| ZK0454 | a read of the part that a test ruled out | [Reading a result](reading-results.md#a-part-the-test-ruled-out) |
| ZK0475 | a test word of the other data kind: `ဒေတာ` on a result, or `ရလဒ်ကောင်း` or `အမှားအယွင်း` on an optional | [Reading a result](reading-results.md#tests) |
| ZK0476 | `<value> ဟူသော အမှားအယွင်း` where no result kind with an error side is wanted | [Giving back an error value](#giving-back-an-error-value) |
| ZK0479 | `ကနဦးတန်ဖိုး` for a result kind, or for a data kind that contains one | [Reading a result](reading-results.md#no-initial-value) |

ZK0477, `အလွတ်` where no optional is wanted, is on the page of
[optionals](optionals.md#errors).

```zaka-error ZK0443
စစ် ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "သုည" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စစ် ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0443]: "စစ်" can fail, but the call does not pass the failure on
```

```zaka-error ZK0444
တစ် ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
၁ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

သုံး ဟူသော လုပ်ဆောင်ချက် သည်
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
စစ် ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "သုည" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

သုံး ဟူသော လုပ်ဆောင်ချက် သည်
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

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0446]: the called function fails with "text", but the function around the call fails with "isize"
```

## Open

- Zaka has no block that catches a failure and goes on with its own code. [Reading a
  result](reading-results.md) covers that need, with a test in `အကယ်၍` and a read for
  each part.
- Zaka does not convert one error kind into another, so a pass-on needs the same error
  kind in both functions.

How the compiler lowers a function that can fail is in [The
IR](../internals/codegen.md#lowering).
