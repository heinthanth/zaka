---
title: The path module
description: The path kind and the functions that build a path and take it apart.
---

# The path module

The path module, `"စံကား/လမ်းကြောင်း"`, holds the path kind and the functions that build a
path and take it apart. Its functions are written in Zaka.

`ဖိုင်လမ်းကြောင်း` is the data kind (type) of a path to a file or a folder. It is a [kind
alias](../language/aliases.md) of the struct `ဖိုင်လမ်းကြောင်းအချက်အလက်`, whose one field `စာသားပုံစံ`
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
([Calls on a value](../language/functions-calls.md#calls-on-a-value)).

```zaka
"စံကား/လမ်းကြောင်း" မှ ဖိုင်လမ်းကြောင်း၊ ဖိုင်လမ်းကြောင်းဆက်၊ ဖိုင်အမည်၊ ဖိုင်အမည်နောက်တွဲ၊
ဖိုင်တွဲလမ်းကြောင်း နှင့် ရှင်းလင်းသေသပ်အောင်ပြုလုပ် တို့ကို ထည့်သွင်းမည်။

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
