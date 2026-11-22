---
title: Reading a result
description: How a caller keeps the whole result of a function that can fail, tests which part it contains, reads the good result or the error value, and what the compiler checks.
---

# Reading a result

When a call to a function that can fail does not pass the failure on, the caller gets the
*whole result*: a data cell that contains either a good result (`Ok` value) or an error
value (error). This page covers how to keep a whole result, test which part it contains
and read that part. [Errors and results](error-handling.md) covers how a function fails and
how a failure is passed on, and the Learn book teaches this page in [Reading a whole
result](../../learn/error-handling/results.md).

```zaka
ပိတ် ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
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

## The whole result

```text
ကိန်းရှင် R သည် <name> ဟူသော လုပ်ဆောင်ချက်ကို <arguments> တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် R သည် <name>(<arguments>) ဖြစ်ပါစေ။
R ကို <name>(<arguments>) ဟု ထားမည်။
```

A call to a function that can fail, written with no `ရလဒ်ကောင်း` and no `?`, passes
nothing on: its value is the whole result. The call is an expression like other calls, and
a call *statement* is the one place that needs the pass-on, because there the result
would be dropped (ZK0443).

We can write the data kind (type) of a whole result as a [result kind](result-kinds.md), or let a
declaration take it from its starting value. `ထားမည်` can store another result with the
same result kind: the same good side and the same error side. Messages write the result
kind as `isize result or text error`, `success or text error` or `isize result or failure`.
A whole result is no number, so using it as one is ZK0403.

`ပြသမည်` and a `{...}` slot write a whole result as the part it contains:
`ရလဒ်ကောင်း(<value>)` or `အမှားအယွင်း(<value>)`. A side with no value is written as its
news, `အောင်မြင်သည့်သတင်း` or `မအောင်မြင်သည့်သတင်း`, with no brackets.

## Tests

```text
<R> ထဲတွင် ရလဒ်ကောင်း ရှိ
<R> ထဲတွင် အမှားအယွင်း ရှိ
```

We use the first test to ask whether `R` contains a good result, and the second to ask
whether it contains an error value. We can write these words in other forms:

| Form | Meaning |
| --- | --- |
| `အောင်မြင်သည့်သတင်း` in place of `ရလဒ်ကောင်း` | the same test |
| `မအောင်မြင်သည့်သတင်း` in place of `အမှားအယွင်း` | the same test |
| `ရှိနေ` in place of `ရှိ` | the same test, as in the tests of an optional |
| `ရှိလျှင်`, `ရှိနေလျှင်`, `ရှိနေသရွေ့` | the test, with `လျှင်` or `သရွေ့` joined to it |
| `ရှိသည်` in place of `ရှိ` | the same test, which takes `မဟုတ်` after it |
| `ရှိသည် မဟုတ်လျှင်` | the opposite of the test |

A result has no `မရှိ`: to ask the opposite, we test the other part, or put `မဟုတ်` after
the test.

Each test is a true/false value (boolean), so we can use it wherever one fits: in `အကယ်၍`
and `ထိုသို့မဟုတ်ဘဲ`, in the condition of a loop, beside `ဖြစ်၍` and `သို့မဟုတ်` (`&&`
and `||`), and as the operand of a print or a declaration, as in
`ကိန်းရှင် ခ သည် R ထဲတွင် အမှားအယွင်း ရှိ ဖြစ်ပါစေ။`. The operand of a test can be a
variable, or a call of a function that can fail.

A test word of the other data kind is ZK0475: `ဒေတာ` on a result, or `ရလဒ်ကောင်း` or
`အမှားအယွင်း` on an optional. A test, or a read, of a value that is neither a result nor an
optional is ZK0448.

```zaka-error ZK0475
ရှာ ဟူသော လုပ်ဆောင်ချက် သည်
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

## Reads

```text
<R> ထဲမှ ဒေတာ
<R> ထဲမှ အမှားအယွင်း ဒေတာ
```

We use the first form to read the good result, as `unwrap` does in Rust, and the second to
read the error value, as `unwrap_err` does. We can write `အချက်အလက်` in place of `ဒေတာ`,
as in the tests. To read the good result with a fallback in place of the error value, we
can use [the fallback of an optional](optionals.md#a-fallback),
`<R> ထဲမှ ဒေတာ၊ မရှိလျှင် <value>`.

A read of a part that is not there stops the program with a trap of the VM, just like a
conversion that cannot be made:

| Read | The result contains | Trap | The program stops with |
| --- | --- | --- | --- |
| the good result | an error value | not good | `the result holds an error, not a good result: V` |
| the good result | a failure with no value | not good | `the result holds a failure, not a good result` |
| the error value | a good result | not fault | `the result is a good result, not an error` |

[When a program stops](../vm/traps.md#a-whole-result-read-as-the-wrong-part) lists these
traps. A read of the data of an [optional](optionals.md#reading) that is blank stops the
program with a different trap, `no data`.

```zaka
စစ် ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
"စစ် {န}" ဟု ပြသမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "သုည" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ရေး ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
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

The second of the two `အကယ်၍` near the end never calls `စစ်(၇)`, because its left side is
false, so the program does not reach the right side of `ဖြစ်၍`. The function `ရေး` has no
good value, so its whole result contains an error value or nothing:
`ဂ ထဲတွင် ရလဒ်ကောင်း ရှိ` is true when it did not fail, and there is no good value to read
from it.

These two programs read the part that is not there:

```zaka-stop
စစ် ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
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
စစ် ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
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

A read runs where it is written, just like a call that passes a failure on: the operands on
its left run first, and the right side of `&&` and `||` runs when the program reaches it.
So a read of the wrong part stops the program at that place, and nothing to its right
runs.

## No initial value

A result contains its good result or its failure, and the program decides which. So a
result kind has no [initial value](kinds.md#initial-values) (default value): a good result
that no call returned would claim a success that did not happen, and a failure that no
call returned would claim a failure that did not happen either. `ကနဦးတန်ဖိုး` for a result
kind, or for a struct or a counted list that contains one, is ZK0479. A variable of a
result kind declared with no value is unset until the program sets it ([Set before
read](declarations.md#set-before-read)), and a struct literal must give each result field a
value (ZK0482).

```zaka-error ZK0479
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် R သည် ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0479]: "isize result or text error" has no initial value: a result must hold its good value or its failure
```

In [unsafe code](unsafe.md), a new heap block of results starts with octets of zero, as
each heap block does. Read as a result, those octets are a failure that contains the zero
value of the error side, or the bare failure when the error side is
`မအောင်မြင်သည့်သတင်း`, so even there no cell claims a success:

```zaka
ဖတ်ချက် ဟူသော ဒေတာအစုအဝေးတွင်
တွေ့ချက် ဟူသော ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု နှင့်
အကြိမ် ဟူသော ကိန်းပြည့် တစ်ခု
တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် ဖတ်ချက်(တွေ့ချက်: ၇၊ အကြိမ်: ၁) ဖြစ်ပါစေ။
ဖ ကို ပြသမည်။
// a heap block needs unsafe code
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ဘူး သည်
ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည်
ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် ၁ ခုစာ နေရာယူ၍
လိပ်စာကို ဘူး တွင် သိမ်းထားမည်။
ဘူး ၏ တန်ဖိုး ကို ပြသမည်။
လိပ်စာ ဘူး ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဖတ်ချက်(တွေ့ချက်: ရလဒ်ကောင်း(၇)၊ အကြိမ်: ၁)
အမှားအယွင်း("")
```

With the error side `မအောင်မြင်သည့်သတင်း`, a heap cell of zeros prints as
`မအောင်မြင်သည့်သတင်း`. A plain list of results starts with no cells, `[]`, and
`ထည့်မည်` adds each cell. When a variable must contain the result of a call, we write the
call as its starting value, as in `ကိန်းရှင် R သည် ရှာ(၄) ဖြစ်ပါစေ။`. An
[optional](optionals.md) is different: its initial value is the blank value `အလွတ်`,
because an optional claims no success.

## A part the test ruled out

A test proves which part a result variable contains, for as long as nothing changes the
variable. A read of the part that the test ruled out would stop the program each time it
ran, so the compiler reports it as ZK0454 and does not wait for the run. The message tells
which part the variable contains: `"R" holds a good result here, so reading its error value
stops the run each time` or `"R" holds a failure here, so reading its good result stops the
run each time`.

```zaka-error ZK0454
ရှာ ဟူသော လုပ်ဆောင်ချက် သည်
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

These forms prove a part:

| Form | What it proves |
| --- | --- |
| the arm of `အကယ်၍ R ထဲတွင် ရလဒ်ကောင်း ရှိလျှင်` | `R` contains a good result inside the arm |
| the arm of `အကယ်၍ R ထဲတွင် အမှားအယွင်း ရှိလျှင်` | `R` contains a failure inside the arm |
| the arms after an arm: each `ထိုသို့မဟုတ်ဘဲ` and the `သို့မဟုတ်ပါက` | the opposite of the tests before them |
| `ဖြစ်၍` (`&&`) in the condition of an arm | both sides |
| `သို့မဟုတ်` (`||`) in the condition of an arm | both sides false, in the arms after it |
| `မဟုတ်` | the opposite of what its clause proves |
| a guard: an `အကယ်၍` arm that is the whole statement, with no arm after it, whose block ends with `ပြန်ပေးမည်`, `ရပ်မည်` or `ကျော်မည်` | the opposite of its test, after the whole statement |
| the condition of a while loop | its test, inside the loop |

So after `အကယ်၍ R ထဲတွင် အမှားအယွင်း ရှိလျှင် ပြန်ပေးမည်`, `R` contains a good result.

These end a proof:

- an assignment to `R`;
- `R` passed to a parameter marked `ဒေတာ ပြောင်းလဲရန်`;
- the address of `R`;
- a new declaration of the same name;
- a loop that can change `R` in its statements, nested blocks included: a proof made
  outside the loop does not count inside the whole loop, while a test made inside the loop
  still proves its own arm.

When the compiler cannot prove the part, it reports nothing, and the program checks the
read when it reaches it: the read stops the program with the trap `not good` or `not
fault`, as [Reads](#reads) shows.

## Errors

| Code | Mistake |
| --- | --- |
| ZK0448 | a test or a read of a value that is neither a result nor an optional |
| ZK0449 | a read of the good result of a function with no good value |
| ZK0454 | a read of the part that a test ruled out |
| ZK0475 | a test word of the other data kind: `ဒေတာ` on a result, or `ရလဒ်ကောင်း` or `အမှားအယွင်း` on an optional |
| ZK0479 | `ကနဦးတန်ဖိုး` for a result kind, or for a struct or a counted list that contains one |

ZK0454, ZK0475 and ZK0479 are in the sections above, each with an example.

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
ရေး ဟူသော လုပ်ဆောင်ချက် သည်
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
