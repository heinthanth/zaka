---
title: The stdlib
description: The stdlib modules for text, paths, lists, I/O, math and time.
---

# The stdlib

The stdlib (standard library) is a set of modules that the compiler carries, and a
program imports the names it needs from each one, as it does from its own files
([Modules](modules.md)). The Learn book teaches it in [The
stdlib](../../learn/stdlib/README.md). These modules ship with [the core module](core.md).

| Module | File | What it has |
| --- | --- | --- |
| `"စံကား/စာသား"` | `stdlib/text.zaka` | finding, splitting, joining and changing text, and text as an integer |
| `"စံကား/လမ်းကြောင်း"` | `stdlib/path.zaka` | the path kind `ဖိုင်လမ်းကြောင်း` and the parts of a path |
| `"စံကား/စာရင်း"` | `stdlib/list.zaka` | sorting, removing, finding and reversing the cells of a list |
| `"စံကား/အဝင်အထွက်"` | `stdlib/io.zaka` | streams, files and folders: [Input and output](io.md) |
| `"စံကား/သင်္ချာ"` | `stdlib/math.zaka` | [math](#math) constants and functions on 64-bit floats |
| `"စံကား/အချိန်"` | `stdlib/time.zaka` | [clocks and UTC formatting](#time) |

Each function of the first three modules is written in Zaka. Its source is in the stdlib
folder of the compiler, and an editor opens it through go to definition. Text and lists
are values, so a function returns new text or a new list and leaves its argument as it
was. The examples use sentence forms, including calls with
[argument particles](functions-sentences.md#prepositions) and
[conversions through traits](conversions.md#conversions-through-traits).
The [sign form](functions-calls.md#the-sign-form) also works.

## Text

| Call | What it returns |
| --- | --- |
| `<piece> ကို <text> ထဲတွင် ရှာ` | the first u8 number where the piece appears, or blank data, as a `လက္ခဏာမဲ့ ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` |
| `<piece> ကို <text> ထဲတွင် ပါမပါစစ်` | whether the piece appears in the text |
| `<text> ကို <piece> ဖြင့် စမစစစ်` | whether the text starts with the piece |
| `<text> ကို <piece> ဖြင့် ဆုံးမဆုံးစစ်` | whether the text ends with the piece |
| `<text> ကို <separator> ဖြင့် ခွဲ` | the pieces between the separators, as a `စာသား စာရင်း`; a blank separator returns the text alone |
| `<text> ကို စာကြောင်းများအဖြစ်ခွဲ` | the lines, each without its `\n` or `\r\n`; a line break at the end starts no blank line |
| `<text> ကို စကားလုံးများအဖြစ်ခွဲ` | the words: the runs of characters between spaces, tabs and line breaks |
| `<list> ကို <separator> ဖြင့် ဆက်` | the texts of a list joined, with the separator between each two |
| `<text> ကို ဘေးနှစ်ဖက်တွင်နေရာလွတ်ဖယ်` | the text without the white space at its two ends |
| `<text> တွင် <old> ကို <new> ဖြင့် အကုန်အစားထိုး` | the text with each old piece replaced, from the start on |
| `<text> ကို <count> ကြိမ် ထပ်ခါထပ်ခါဆက်` | the text repeated exactly that many times, or empty text for a count of zero or less |
| `<text> ကို စာလုံးကြီးပြောင်း`, `<text> ကို စာလုံးသေးပြောင်း` | the text with each Latin letter in upper or in lower case |
| `<text> ကို <kind> တစ်<measure word> အဖြစ် ပြောင်းလဲသော ရလဒ်` | the text read as an integer, a decimal number or a character, or a failure ([Conversions in the stdlib](conversions.md#conversions-in-the-stdlib)). Import `စမ်းသပ်ပြောင်းလဲ` to use these conversions |

A u8 number counts the u8s of the UTF-8 form of the text, as a [range of
text](text.md#ranges) does, so `ကမ္ဘာ` starts at 22 in `မင်္ဂလာ ကမ္ဘာ`, because each
Burmese letter uses three u8s. A search walks the text once for each place it tries, so
finding a piece of `m` u8s in text of `n` u8s takes up to `n × m` steps.

```zaka
"စံကား/စာသား" မှ ရှာ၊ ခွဲ၊ ဆက်၊ ဘေးနှစ်ဖက်တွင်နေရာလွတ်ဖယ် နှင့် အကုန်အစားထိုး တို့ကို ထည့်သွင်းမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ကမ္ဘာ" ကို "မင်္ဂလာ ကမ္ဘာ" ထဲတွင် ရှာ သည့် ရလဒ် ကို ပြသမည်။
ကိန်းရှင် အပိုင်းများ သည် "a,b,,c" ကို "," ဖြင့် ခွဲ သည့် ရလဒ် ဖြစ်ပါစေ။
အပိုင်းများ ကို ပြသမည်။
အပိုင်းများ ကို " | " ဖြင့် ဆက် သည့် ရလဒ် ကို ပြသမည်။
ကိန်းရှင် ဖယ်ပြီး သည် "  hi  " ကို ဘေးနှစ်ဖက်တွင်နေရာလွတ်ဖယ် သည့် ရလဒ် ဖြစ်ပါစေ။
"[{ဖယ်ပြီး}]" ဟု ပြသမည်။
"a.b.c" တွင် "." ကို "::" ဖြင့် အကုန်အစားထိုး သည့် ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဒေတာ(၂၂)
["a"၊ "b"၊ ""၊ "c"]
a | b |  | c
[hi]
a::b::c
```

## Paths

`ဖိုင်လမ်းကြောင်း` is the data kind (type) of a path to a file or a folder. It is a [kind
alias](aliases.md) of the struct `ဖိုင်လမ်းကြောင်းအချက်အလက်`, whose one field `စာသားပုံစံ`
has the text of the path. Text converts to a path, and a path prints as its text. Each
function below accepts a path or text, as two functions with one name, and a path that it
returns is a `ဖိုင်လမ်းကြောင်း`. Both `/` and `\` separate the parts of a path, and a path
that the module makes uses `/`.

| Call | What it returns |
| --- | --- |
| `<a> ကို <b> နှင့် ဖိုင်လမ်းကြောင်းဆက်` | `b` after `a`, with one separator between them, or an absolute `b` alone |
| `<path> ၏ ဖိုင်အမည်` | the last part: `ဖိုင်.txt` of `a/ဖိုင်.txt` |
| `<path> ၏ ဖိုင်အမည်နောက်တွဲ` | the extension without its `.`, or blank data; a name that starts with `.` has none |
| `<path> ၏ ဖိုင်အမည်အရှေ့ပိုင်း` | the file name without its extension |
| `<path> ၏ ဖိုင်တွဲလမ်းကြောင်း` | the folder that contains the path: `a` of `a/b`, `/` of `/b`, blank for a path of one part |
| `<path> ကို ဖိုင်လမ်းကြောင်းအပြည့်အစုံဟုတ်မဟုတ်စစ်` | whether the path starts with `/` or `\`, or with a drive such as `C:` |
| `<path> ကို ရှင်းလင်းသေသပ်အောင်ပြုလုပ်` | the path with no `.` part, each `..` removing the part before it, and no separator twice in a row; `.` when nothing is left |

The names of the path parts read as properties. Write `p ၏ ဖိုင်အမည်` to get the
file name or `p ၏ ဖိုင်တွဲလမ်းကြောင်း` to get the folder that contains it
([Calls on a value](functions-calls.md#calls-on-a-value)).

```zaka
"စံကား/လမ်းကြောင်း" မှ ဖိုင်လမ်းကြောင်း၊ ဖိုင်လမ်းကြောင်းဆက်၊ ဖိုင်အမည်၊ ဖိုင်အမည်နောက်တွဲ၊ ဖိုင်တွဲလမ်းကြောင်း နှင့် ရှင်းလင်းသေသပ်အောင်ပြုလုပ် တို့ကို ထည့်သွင်းမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လ သည် "docs/ဖိုင်.txt" ကို ဖိုင်လမ်းကြောင်း တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
လ ကို ပြသမည်။
လ ၏ ဖိုင်အမည် ကို ပြသမည်။
လ ၏ ဖိုင်အမည်နောက်တွဲ ကို ပြသမည်။
လ ၏ ဖိုင်တွဲလမ်းကြောင်း ကို ပြသမည်။
"a/b" ကို "c.md" နှင့် ဖိုင်လမ်းကြောင်းဆက် သည့် ရလဒ် ကို ပြသမည်။
"a/./b//../c" ကို ရှင်းလင်းသေသပ်အောင်ပြုလုပ် သည့် ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
docs/ဖိုင်.txt
ဖိုင်.txt
ဒေတာ("txt")
docs
a/b/c.md
a/c
```

## Lists

Each function of the list module is [generic](generics.md), so it accepts a list of each
data kind.

| Call | What it returns |
| --- | --- |
| `<list> ကို ငယ်စဉ်ကြီးလိုက်စီ` | the cells from the smallest to the largest; the cell kind must work with `<` (`နှိုင်းယှဉ်နိုင်သော`) |
| `<list> ကို (<function>) ကိုသုံး၍ စီ` | the cells in the order given by the comparison function, which returns true when its first argument comes before its second |
| `<list> မှ <number> အကွက်ကို ဖယ်` | the list without the cell at a number counted from 0; a number outside the list stops the program |
| `<list> ထဲတွင် <value> ၏ အကွက်နေရာကိုရှာ` | the first cell number, from 0, whose cell has the value, or blank data |
| `<value> ကို <list> ထဲတွင် ပါမပါစစ်` | whether a cell has the value |
| `<list> ကို ပြောင်းပြန်စီ` | the cells from the last to the first |

- Both sorts are a merge sort, which takes `n × log n` steps for `n` cells, and cells where
  neither comes before the other keep their order. Text does not work with `<`, so we sort a
  list of text through `စီ` with a function that compares two texts.
- `စီ` is a reserved word of the counted loop, so the module declares the function as
  `{စီ}`, an import names it `{စီ}`, and a call writes it bare, right before its call end.
- The function before `ကိုသုံး၍` goes in parentheses, because without them
  `f ဟူသော လုပ်ဆောင်ချက် ကို` would start a long call.
- `ပါမပါစစ်` of lists and `ပါမပါစစ်` of text can both be imported into one file, and each
  call uses the one that its arguments fit.
- Each function duplicates (copies) the list once.

```zaka
"စံကား/စာရင်း" မှ ငယ်စဉ်ကြီးလိုက်စီ၊ {စီ}၊ ဖယ်၊ အကွက်နေရာကိုရှာ နှင့် ပြောင်းပြန်စီ တို့ကို ထည့်သွင်းမည်။

ပိုကြီး ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
က သည် ခ ထက် ကြီးသည် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် l သည် [၅၊ ၃၊ ၈၊ ၁] ဖြစ်ပါစေ။
l ကို ငယ်စဉ်ကြီးလိုက်စီ သည့် ရလဒ် ကို ပြသမည်။
l ကို (ပိုကြီး ဟူသော လုပ်ဆောင်ချက်) ကိုသုံး၍ စီ သည့် ရလဒ် ကို ပြသမည်။
l မှ ၁ အကွက်ကို ဖယ် သည့် ရလဒ် ကို ပြသမည်။
l ထဲတွင် ၈ ၏ အကွက်နေရာကိုရှာ သည့် ရလဒ် ကို ပြသမည်။
l ကို ပြောင်းပြန်စီ သည့် ရလဒ် ကို ပြသမည်။
l ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၃၊ ၅၊ ၈]
[၈၊ ၅၊ ၃၊ ၁]
[၅၊ ၈၊ ၁]
ဒေတာ(၂)
[၁၊ ၈၊ ၃၊ ၅]
[၅၊ ၃၊ ၈၊ ၁]
```

## Math

Import the constants `ပိုင်` (π) and `အွိုင်လာကိန်းသေ` (e) from `"စံကား/သင်္ချာ"`.
Each function takes and gives a `ဒဿမကိန်း`, a 64-bit floating-point number.

| Call | What it gives |
| --- | --- |
| `ပကတိတန်ဖိုး(n)` | absolute value |
| `နှစ်ထပ်ကိန်းရင်း(n)` | square root |
| `a ကို b ထပ်တင်သည့်ရလဒ်`, `ထပ်တင်(a၊ b)` | `a` raised to the power `b` |
| `အခြေ၁၀ရှိသောလော့ဂရစ်သမ်(n)` | natural and base-ten logarithms |
| `ဆိုင်း(n)`, `ကိုဆိုင်း(n)`, `တန်းဂျင့်(n)` | sine, cosine and tangent of an angle in radians |
| `အောက်ဘက်အနီးဆုံးကိန်းပြည့်လျှော့ယူ(n)`, `အပေါ်ဘက်အနီးဆုံးကိန်းပြည့်တိုးယူ(n)` | round down or up to an integer value |
| `အနီးဆုံးကိန်းပြည့်ယူ(n)` | round to the nearest integer value, with ties going to the larger magnitude |
| `ဒဿမဖြုတ်(n)` | drop the fractional part, rounding towards zero |

The functions follow floating-point arithmetic. The square root of a negative number
gives `NaN`, and the logarithm of zero gives `-inf`. Transcendental functions
can differ in their last digits across platforms.

```zaka
"စံကား/သင်္ချာ" မှ နှစ်ထပ်ကိန်းရင်း၊ ထပ်တင် နှင့် အနီးဆုံးကိန်းပြည့်ယူ တို့ကို ထည့်သွင်းမည်။
၈၁.၀ ၏ နှစ်ထပ်ကိန်းရင်း ကို ပြသမည်။
၁၀ ကို ၂ ထပ်တင်သည့်ရလဒ် ကို ပြသမည်။
အနီးဆုံးကိန်းပြည့်ယူ(-၂.၅) ကို ပြသမည်။
```

```text
၉.၀
၁၀၀.၀
-၃.၀
```

## Time

Import these functions from `"စံကား/အချိန်"`. A clock reading is a signed 64-bit integer
in milliseconds.

| Call | What it gives |
| --- | --- |
| `ကြာချိန်နာရီကိုဖတ်()` | a reading from a monotonic clock with an unspecified origin. Subtract two readings to measure elapsed time |
| `ယူနစ်အချိန်ကိုဖတ်()` | the current wall-clock time in milliseconds since 1970-01-01T00:00:00Z |
| `အိုင်အက်စ်အိုစာသားအဖြစ်ပြောင်း(ms)` | UTC text such as `1970-01-01T00:00:00.001Z`, or an `အချိန်အမှားအယွင်း` for a date outside 0000 through 9999 |

UTC formatting uses Western digits and three fractional digits. Negative stamps refer
to instants before 1970. The explorer uses a virtual clock that starts at the Unix epoch
and advances during waits. Comptime calls read zero from both clocks.

```zaka
"စံကား/အချိန်" မှ အိုင်အက်စ်အိုစာသားအဖြစ်ပြောင်း ကို ထည့်သွင်းမည်။
အိုင်အက်စ်အိုစာသားအဖြစ်ပြောင်း(၀) ကို ပြသမည်။
အိုင်အက်စ်အိုစာသားအဖြစ်ပြောင်း(-၁) ကို ပြသမည်။
```

```text
ရလဒ်ကောင်း("1970-01-01T00:00:00.000Z")
ရလဒ်ကောင်း("1969-12-31T23:59:59.999Z")
```

## Open

- Removing a cell counted from 1, `<list> မှ <number> ကြိမ်မြောက်အကွက်ကို ဖယ်`, waits
  for generic functions that share a name, because a generic function at present has one
  declaration for each name.
