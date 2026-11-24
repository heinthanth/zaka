---
title: Asserts
description: Stop the program at once when a condition that must hold does not, with a message that tells what went wrong.
---

# Asserts

A result is for a failure that the program expects, such as a divisor of zero that a
caller can handle. Some mistakes should never happen at all, such as a score above 100,
and for those we can write an assert (assertion): a sentence that states what must be true
at that point of the program. When the condition is true, the program goes on, and when it
is false, the program stops (trap) and tells us which assert did not hold.

## You will learn

- how to write an assert with `ဖြစ်ရမည်`
- what the program prints when an assert fails
- how to give an assert a message
- how a debug assert differs from a plain one

```zaka
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၃ ဖြစ်ပါစေ။
အမှတ် <= ၁၀၀ ဖြစ်ရမည်။ // the score must be 100 or less
"အမှတ် မှန်ကန်သည်" ဟု ပြသမည်။
```

```text
အမှတ် မှန်ကန်သည်
```

The score is 73, so the condition `အမှတ် <= ၁၀၀` holds, and the program goes on to the
print. We write an assert with this pattern:

```text
<condition> ဖြစ်ရမည်။
```

| Zaka | Reads as |
| --- | --- |
| `<condition>` | a true/false value (boolean), written the same as the condition of an `if` |
| `ဖြစ်ရမည်` | "must be so" |

An assert is a statement of its own, and it gives no value. As `ဖြစ်လျှင်` follows a
condition in an [if](../control-flow/if-else.md#a-truefalse-value-as-the-condition),
`ဖြစ်ရမည်` follows the condition of an assert as a separate word.

## When an assert fails

When the condition is false, the program stops at the assert, and nothing after it runs.
For a comparison, the stop shows the values on both sides, so we can see at once what was
wrong:

```zaka-stop
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၂၀ ဖြစ်ပါစေ။
"စစ်ဆေးမည်" ဟု ပြသမည်။
အမှတ် <= ၁၀၀ ဖြစ်ရမည်။
"ဤနေရာ မရောက်ပါ" ဟု ပြသမည်။
```

```text
စစ်ဆေးမည်
the program stopped: an assert did not hold: ၁၂၀ <= ၁၀၀
```

The stop writes the value of `အမှတ်`, then the sign, then `၁၀၀`. A condition that is no
comparison, such as a true/false variable, has no sides to show, and its stop reads
`an assert did not hold` alone.

## A comparison in words

```text
<a> သည် <b> အောက် မငယ်သည် ဖြစ်ရမည်။
```

A comparison in words works too, and it keeps its verb, here `မငယ်သည်` ("is not less"),
before `ဖြစ်ရမည်`. The stop still writes the comparison with its sign, so this assert shows
`>=` when it fails:

```zaka-stop
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃၅ ဖြစ်ပါစေ။
အမှတ် သည် ၄၀ အောက် မငယ်သည် ဖြစ်ရမည်။
```

```text
the program stopped: an assert did not hold: ၃၅ >= ၄၀
```

We cannot glue `ရမည်` to the verb: `မငယ်ရမည်` is a mistake, and `ဖြစ်ရမည်` stands apart.

## A message

```text
<condition> ဖြစ်ရမည်၊ မဟုတ်ပါက <text> ဟု ရပ်မည်။
```

| Zaka | Reads as |
| --- | --- |
| `၊ မဟုတ်ပါက` | ", if not," |
| `<text> ဟု ရပ်မည်` | "will stop with the words `<text>`" |

A message tells the reader of the stop what the assert was for. It is text (string): a
string, a template string or a text variable. The program builds it when the assert fails,
so a template slot in it costs nothing while the condition holds. The stop shows the
message first, then the comparison in parentheses:

```zaka-stop
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၂၀ ဖြစ်ပါစေ။
ကိန်းရှင် အမည် သည် "မောင်မောင်" ဖြစ်ပါစေ။
အမှတ် <= ၁၀၀ ဖြစ်ရမည်၊ မဟုတ်ပါက "{အမည်} ၏ အမှတ် မှားနေသည်" ဟု ရပ်မည်။
```

```text
the program stopped: an assert did not hold: မောင်မောင် ၏ အမှတ် မှားနေသည် (၁၂၀ <= ၁၀၀)
```

## Debug asserts

```text
ပရိုဂရမ်ရေးသားချိန်တွင် <condition> ဖြစ်ရမည်။
```

| Zaka | Reads as |
| --- | --- |
| `ပရိုဂရမ်ရေးသားချိန်တွင်` | "at the time of writing the program" |

A plain assert is checked in each build of the program. An assert that starts with
`ပရိုဂရမ်ရေးသားချိန်တွင်` is a debug assert (`debug_assert!`): it is checked as well, but a
build can drop it with `--debug-asserts off`. We use it for a check that costs time and
that we want while we write and test the program:

```zaka
ကိန်းရှင် အမှတ်များ သည် [၇၃၊ ၈၅၊ ၆၀] ဖြစ်ပါစေ။
ပရိုဂရမ်ရေးသားချိန်တွင် (အမှတ်များ ၏ အကွက်အရေအတွက်) == ၃ ဖြစ်ရမည်။
အမှတ်များ ကို ပြသမည်။
```

```text
[၇၃၊ ၈၅၊ ၆၀]
```

```console
$ zakac run --debug-asserts off program.zaka
```

With `--debug-asserts off`, the debug asserts are gone from the program, but the compiler
still checks that each one is written right: its condition must be a true/false value and
its message must be text.

## Details

- A condition that is no true/false value is the mistake ZK0402, as in an `if`, and a
  message that is not text is ZK0401.
- Each side of a comparison runs once, so a call in the condition does not run again to
  show its value ([When an assert fails](../../manual/language/asserts.md#when-an-assert-fails)).
- An assert gives the compiler checks no fact to build on: after `န < ၃ ဖြစ်ရမည်။`, a cell
  number `န` is still checked as before ([Asserts and the
  checker](../../manual/language/asserts.md#asserts-and-the-checker)).
- `ဖြစ်ရမယ်` and `ပရိုဂရမ်ရေးသားချိန်မှာ` are the spoken spellings, and
  `ပရိုဂရမ်ရေးသားချိန်၌` is another written one ([Other
  spellings](../../manual/language/asserts.md#other-spellings)).
- The full rules, the flag and the text of each stop are on
  [Asserts](../../manual/language/asserts.md), and the stop itself is on [When a program
  stops](../../manual/vm/traps.md#an-assert-that-does-not-hold).

## Recap

- `<condition> ဖြစ်ရမည်။` stops the program when the condition is false.
- For a comparison, the stop shows both sides, such as `၁၂၀ <= ၁၀၀`.
- `၊ မဟုတ်ပါက <text> ဟု ရပ်မည်` adds a message, which the program builds when the assert
  fails.
- `ပရိုဂရမ်ရေးသားချိန်တွင်` in front makes a debug assert, which `--debug-asserts off`
  drops.
- A result is for a failure that a caller handles, and an assert is for a mistake that
  should never happen.

## Hands on

- Write an assert that the count of a list is above zero, then run it with a list that
  holds no cells and read the stop.
- Give the assert a message with a template slot that writes the count.
- Turn the assert into a debug assert, and run the program with `--debug-asserts off`.

Next: [Optional values](../optionals/README.md).
