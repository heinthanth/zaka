---
title: Asserts
description: The assert statement, which stops the program when a condition does not hold, with its message, the debug assert, the flag that drops debug asserts, and the text of the stop.
---

# Asserts

An assert (assertion) states a condition that must hold at one point of a program. When
the condition is true, the program goes on as if the assert were not there; when it is
false, the program stops (trap) with a message that names the assert and, for a
comparison, the values on both sides. This page states the full rules, and the Learn book
teaches asserts in [Asserts](../../learn/error-handling/asserts.md).

```zaka
ပေါင်း ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က + ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၃ ဖြစ်ပါစေ။
အမှတ် သည် ၄၀ အောက် မငယ်သည် ဖြစ်ရမည်။
ပေါင်း(၂၊ ၃) == ၅ ဖြစ်ရမည်၊ မဟုတ်ပါက "ပေါင်း မှားနေသည်" ဟု ရပ်မည်။
ပရိုဂရမ်ရေးသားချိန်တွင် အမှတ် <= ၁၀၀ ဖြစ်ရမည်။
အမှတ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၇၃
```

The three asserts hold, so the program prints `၇၃`. The first one is a comparison in
words, the second compares the result of a call and carries a message, and the third is a
debug assert, which a build can drop.

## The forms

```text
<condition> ဖြစ်ရမည်။
<condition> ဖြစ်ရမည်၊ မဟုတ်ပါက <text> ဟု ရပ်မည်။
ပရိုဂရမ်ရေးသားချိန်တွင် <condition> ဖြစ်ရမည်။
ပရိုဂရမ်ရေးသားချိန်တွင် <condition> ဖြစ်ရမည်၊ မဟုတ်ပါက <text> ဟု ရပ်မည်။
```

| Zaka | Reads as | Standard term |
| --- | --- | --- |
| `<condition> ဖြစ်ရမည်` | "`<condition>` must be so" | `assert!(condition)`, `assert condition` |
| `မဟုတ်ပါက <text> ဟု ရပ်မည်` | "if not, will stop with the words `<text>`" | the message of `assert!(condition, "...")` |
| `ပရိုဂရမ်ရေးသားချိန်တွင်` | "at the time of writing the program" | `debug_assert!` |

The condition is written the same as the condition of an [if](control-flow.md#if): a
true/false value (boolean), a comparison in signs or in words, or a logical expression.
As `ဖြစ်လျှင်` follows the condition of an `if`, `ဖြစ်ရမည်` follows the condition of an
assert as a separate word. A comparison in words keeps its verb, with or without `သည်`, so
`အမှတ် သည် ၄၀ အောက် မငယ်သည် ဖြစ်ရမည်` and `အမှတ် သည် ၄၀ အောက် မငယ် ဖြစ်ရမည်` are the same
assert. Zaka has no form that glues `ရမည်` to the verb, so `မငယ်ရမည်` is the parse error
ZK0208.

An assert is a statement of its own. It can stand wherever a statement can, the one
statement of an `if` branch included, and it gives no value.

## The message

```text
<condition> ဖြစ်ရမည်၊ မဟုတ်ပါက <text> ဟု ရပ်မည်။
```

After `၊ မဟုတ်ပါက`, an assert can give the text that its stop shows. The message is one
value of text (string), such as a string literal, a template string or a text variable,
and a value of another data kind (type) is ZK0401. The message is built when the assert
fails and at no other time, so a template slot or a call in it costs nothing while the
condition holds:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကြိမ် သည် ၃ ဖြစ်ပါစေ။
ကြိမ် > ၀ ဖြစ်ရမည်၊ မဟုတ်ပါက "ကြိမ် {ကြိမ်} သည် မှားနေသည်" ဟု ရပ်မည်။
ကြိမ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၃
```

## When an assert fails

When the condition is false, the program stops with the trap `assert`. The stop line
starts with `the program stopped: an assert did not hold`, and what follows depends on
the assert:

| The assert has | The stop reads |
| --- | --- |
| a message and a comparison | `an assert did not hold: <message> (<left> <sign> <right>)` |
| a comparison and no message | `an assert did not hold: <left> <sign> <right>` |
| a message and no comparison | `an assert did not hold: <message>` |
| neither | `an assert did not hold` |

A comparison here is a condition whose last operation is `==`, `!=`, `<`, `>`, `<=` or
`>=`, in signs or in words. The stop writes the sign of the operation even for a
comparison in words, so `က သည် ၃ နှင့် မညီသည် ဖြစ်ရမည်` shows `၃ != ၃`, and it shows each
value as `ပြသမည်` prints it: numbers in Burmese digits, text with no quotes, and a member
of an enumeration as its name. A true/false value or a logical expression, such as
`က > ၀ && က < ၁၀`, is no comparison, and its stop shows the message alone.

```zaka-stop
ပေါင်း ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
"ပေါင်းသည်" ဟု ပြသမည်။
က + ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပေါင်း(၂၊ ၄) == ၇ ဖြစ်ရမည်၊ မဟုတ်ပါက "ပေါင်း မှားနေသည်" ဟု ရပ်မည်။
"ဤနေရာ မရောက်ပါ" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ပေါင်းသည်
the program stopped: an assert did not hold: ပေါင်း မှားနေသည် (၆ == ၇)
```

Each side of the comparison runs once. The assert keeps the value of each side in a
hidden slot, tests the slots, and shows the same slots when it fails, so the call
`ပေါင်း(၂၊ ၄)` above prints `ပေါင်းသည်` once and not twice.

`zakac run` writes the long form, with the function and the place of the assert, as for
each other [stop](../vm/traps.md#reading-the-message):

```console
$ zakac run --vm sum.zaka
ပေါင်းသည်
zaka_vm: the program stopped: an assert did not hold: ပေါင်း မှားနေသည် (၆ == ၇) (in ပရိုဂရမ်, at sum.zaka:10:1)
```

The exit code is 1, as for each other stop.

## Debug asserts

```text
ပရိုဂရမ်ရေးသားချိန်တွင် <condition> ဖြစ်ရမည်။
```

A plain assert is checked in each build, `--release` included, as Rust checks
`assert!`. A debug assert, an assert that starts with `ပရိုဂရမ်ရေးသားချိန်တွင်`, is
checked as well unless the build drops it with `--debug-asserts off`:

| Flag | Value | Default | What it does |
| --- | --- | --- | --- |
| `--debug-asserts` | `on` or `off` | `on` | `off` drops each debug assert, its condition and its message from the program |

The flag belongs to `irgen` and the stages after it, so `zakac run` and `zakac build` take
it on each target ([Commands](../compiler/commands.md#options-of-the-compile-stages)).
A config file sets it as `debug-asserts = "off"` ([Configuration](../compiler/configuration.md)).
`--release` does not change it: a release build keeps its debug asserts until the flag
drops them.

With a debug assert that does not hold in `check.zaka`:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ်ရေးသားချိန်တွင် ၁ > ၂ ဖြစ်ရမည်။
"ပြီး" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: an assert did not hold: ၁ > ၂
```

the flag decides whether the run stops:

```console
$ zakac run check.zaka
check.zaka: the program stopped: an assert did not hold: ၁ > ၂ (in ပရိုဂရမ်, at check.zaka:2:1)
$ zakac run --debug-asserts off check.zaka
ပြီး
```

The check of names and data kinds reads each assert whatever the flag holds. A dropped
debug assert must still have a true/false condition and a text message, and its names
must still be declared, so a program that builds with `--debug-asserts off` builds with
`on` too.

## Asserts and the checker

An assert gives no fact to the [flow checker](../compiler/checks.md). After
`န < ၃ ဖြစ်ရမည်။`, the checker does not treat `န` as below 3, so the assert does not prove
a cell number (index) in range or a divisor nonzero for the statements after it. The run
checks each such operation as before.

## Other spellings

| Word | Other spelling | Kind |
| --- | --- | --- |
| `ဖြစ်ရမည်` | `ဖြစ်ရမယ်` | spoken, ZK0510 unless the program allows spoken spellings |
| `ပရိုဂရမ်ရေးသားချိန်တွင်` | `ပရိုဂရမ်ရေးသားချိန်မှာ` | spoken, ZK0510 as above |
| `ပရိုဂရမ်ရေးသားချိန်တွင်` | `ပရိုဂရမ်ရေးသားချိန်၌` | written, with no warning |

The [Spoken Burmese](spoken.md) page explains how a program allows the spoken spellings.

```zaka lint-allow=spoken
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မယ်။
ကိန်းရှင် က သည် ၃ ဖြစ်ပါစေ။
ပရိုဂရမ်ရေးသားချိန်မှာ က > ၀ ဖြစ်ရမယ်။
က ကို ပြသမယ်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၃
```

## Mistakes

| Mistake | Code |
| --- | --- |
| a condition that is no true/false value | ZK0402 |
| a message that is not text | ZK0401 |
| `ရမည်` glued to the verb of a comparison in words, such as `ငယ်ရမည်` | ZK0208 |
| a message with no `ရပ်မည်` after its `ဟု` | ZK0201 |
| a spoken spelling in a program that does not allow them | ZK0510 |

```zaka-error ZK0402
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ၃ ဖြစ်ပါစေ။
က ဖြစ်ရမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0402]: a condition must be a boolean, found isize
```

## How an assert is built

The lowering turns an assert into an `if` on the negated condition around a `halt`
operation with the cause `assert`, whose value is the text to show; an assert with neither
a message nor a comparison has no value. A comparison first keeps each side in a hidden
slot, and the statements that build the message run inside the `if`, so the message is
built when the assert fails. On the VM the operation is `halt assert`
([Instructions](../vm/instructions.md#the-operations)); the C and native targets call the runtime
function `zk_assert_failed`, or `zk_assert_failed_bare` with no text, which stop with the
trap `ZK_TRAP_ASSERT` or `ZK_TRAP_ASSERT_BARE`. [The IR](../internals/codegen.md) and
[The parser](../internals/parser-statements.md#asserts) give the details.
