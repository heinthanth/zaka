---
title: The stdlib
description: The modules of the stdlib written in Zaka - text, paths and lists - with the call of each function, its result and its cost.
---

# The stdlib

The stdlib (standard library) is a set of modules that the compiler carries, and a
program imports the names it needs from each one, as it does from its own files
([Modules](modules.md)). The Learn book teaches it in [The
stdlib](../../learn/stdlib/README.md). Next to [the core module](core.md), there are four
modules:

| Module | File | What it has |
| --- | --- | --- |
| `"စံကား/စာသား"` | `stdlib/text.zaka` | finding, splitting, joining and changing text, and text as an integer |
| `"စံကား/လမ်းကြောင်း"` | `stdlib/path.zaka` | the path kind `ဖိုင်လမ်းကြောင်း` and the parts of a path |
| `"စံကား/စာရင်း"` | `stdlib/list.zaka` | sorting, removing, finding and reversing the cells of a list |
| `"စံကား/အဝင်အထွက်"` | `stdlib/io.zaka` | streams, files and folders: [Input and output](io.md) |

Each function of the first three modules is written in Zaka. Its source is in the stdlib
folder of the compiler, and an editor opens it through go to definition. Text and lists
are values, so a function returns new text or a new list and leaves its argument as it
was. We can write each call below as a sentence, through the
[prepositions](functions-sentences.md#prepositions) of its parameters, and the sign form
`f(a, b)` calls each function too.

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
| `<text> ကို <count> အထိ ထပ်ခါထပ်ခါဆက်` | the text written a number of times in a row |
| `<text> ကို စာလုံးကြီးပြောင်း`, `<text> ကို စာလုံးသေးပြောင်း` | the text with each Latin letter in upper or in lower case |
| `ကိန်းပြည့်အဖြစ်ပြောင်း(<text>)` | the text read as an integer, or a failure; the conversion `<text> ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်` calls it ([Conversions](conversions.md#conversions-through-functions)) |

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
| `<path> ပါရှိသော ဖိုင်တွဲလမ်းကြောင်း` | the folder that contains the path: `a` of `a/b`, `/` of `/b`, blank for a path of one part |
| `<path> ကို ဖိုင်လမ်းကြောင်းအပြည့်အစုံဟုတ်မဟုတ်စစ်` | whether the path starts with `/` or `\`, or with a drive such as `C:` |
| `<path> ကို ရှင်းလင်းသေသပ်အောင်ပြုလုပ်` | the path with no `.` part, each `..` removing the part before it, and no separator twice in a row; `.` when nothing is left |

We can call the functions whose last preposition starts with `၏` after `၏` with no call
end, as in `p ၏ ဖိုင်အမည်` ([Calls on a value](functions-calls.md#calls-on-a-value)), or
as a preposition call with one, as in `p ၏ ဖိုင်အမည် သည့် ရလဒ်`.

```zaka
"စံကား/လမ်းကြောင်း" မှ ဖိုင်လမ်းကြောင်း၊ ဖိုင်လမ်းကြောင်းအဖြစ်ပြောင်း၊ ဖိုင်လမ်းကြောင်းဆက်၊ ဖိုင်အမည်၊ ဖိုင်အမည်နောက်တွဲ၊ ဖိုင်တွဲလမ်းကြောင်း နှင့် ရှင်းလင်းသေသပ်အောင်ပြုလုပ် တို့ကို ထည့်သွင်းမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လ သည် "docs/ဖိုင်.txt" ကို ဖိုင်လမ်းကြောင်း တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
လ ကို ပြသမည်။
လ ၏ ဖိုင်အမည် ကို ပြသမည်။
လ ၏ ဖိုင်အမည်နောက်တွဲ ကို ပြသမည်။
လ ပါရှိသော ဖိုင်တွဲလမ်းကြောင်း သည့် ရလဒ် ကို ပြသမည်။
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
| `<list> ကို (<function>) ကိုသုံး၍ စီ` | the cells in the order that a function gives: `f(a, b)` is true when `a` comes before `b` |
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

ကြီးသလား ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
က > ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် l သည် [၅၊ ၃၊ ၈၊ ၁] ဖြစ်ပါစေ။
l ကို ငယ်စဉ်ကြီးလိုက်စီ သည့် ရလဒ် ကို ပြသမည်။
l ကို (ကြီးသလား ဟူသော လုပ်ဆောင်ချက်) ကိုသုံး၍ စီ သည့် ရလဒ် ကို ပြသမည်။
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

## Open

- Removing a cell counted from 1, `<list> မှ <number> ကြိမ်မြောက်အကွက်ကို ဖယ်`, waits
  for generic functions that share a name, because a generic function at present has one
  declaration for each name.
