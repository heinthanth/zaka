---
title: Input and output
description: The stdlib module of input and output - its streams and its error enumeration, reading a line, reading and writing a file, files and folders, writing to a stream, and the rights a run needs.
---

# Input and output

A program reads and writes through the functions of the stdlib module
`"စံကား/အဝင်အထွက်"` (input and output, I/O). Each one is an [external
function](functions-c.md#external-functions), which means that the VM does its work, and
each one can fail with a member of the enumeration `အချက်အလက်အဝင်အထွက်အမှားအယွင်း`. The
Learn book teaches the module in [Input and output](../../learn/io/README.md). A program
imports the names that it needs:

```text
"စံကား/အဝင်အထွက်" မှ ရေး၊ ဖိုင်ကိုဖတ် နှင့် အချက်အလက်အဝင်အထွက်အမှားအယွင်း တို့ကို ထည့်သွင်းမည်။
```

## What the module contains

| Name | What it is |
| --- | --- |
| `အချက်အလက်အဝင်အထွက်အမှားအယွင်း` | the enumeration of failures: `မတွေ့ခြင်း` (no such file), `ခွင့်ပြုချက်မရှိခြင်း` (not allowed), `အဝင်အချက်အလက်အဆုံးကိုရောက်ခြင်း` (the end of the input), `စာသားဖွဲ့စည်းပုံမမှန်ခြင်း` (not UTF-8 text), `တခြားအကြောင်းအရင်း` (each other failure) |
| `အချက်အလက်အဝင်စီးကြောင်း` | a stream to read from |
| `အချက်အလက်အထွက်စီးကြောင်း` | a stream to write to |
| `စံပြုအဝင်` | standard input, a stream to read from |
| `စံပြုအထွက်`, `စံပြုအမှား` | standard output and standard error, streams to write to |
| `<stream> မှ စာကြောင်းတစ်ကြောင်းဖတ်` | one line of the stream, without its line break, as text |
| `<path> ၌ရှိသော ဖိုင်ကိုဖတ်` | the whole file as text |
| `<text> ကို <path> သို့ ရေး` | writes the text as the whole content of the file |
| `<text> ကို <stream> သို့ ရေး` | writes the text to the stream, with no line break |
| `<name> ဟူသည့် ပတ်ဝန်းကျင်ကိန်းရှင် ကို ဖတ်` | the variable of the environment with that name, as text, or `မတွေ့ခြင်း` when it is not set |

We call these functions as [preposition calls](functions-sentences.md#prepositions), and
the other call forms work too. `ရေး` is [two functions with one
name](functions-overloads.md): the data kind (type) of the argument at `သို့` chooses
between a file path and a stream.

## Calling a function that can fail

The result of each function can be a failure, so a call passes the failure on, reads the
whole result, or both ([Errors and results](error-handling.md)). The program start block
cannot pass a failure on, so a program does its input and output in a function that can
fail, and the program start block reads the whole result:

```zaka
"စံကား/အဝင်အထွက်" မှ ရေး၊ ဖိုင်ကိုဖတ် နှင့် အချက်အလက်အဝင်အထွက်အမှားအယွင်း တို့ကို ထည့်သွင်းမည်။

မှတ်စု ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား သို့မဟုတ် အမှားအယွင်းအနေဖြင့် အချက်အလက်အဝင်အထွက်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
"မင်္ဂလာပါ" ကို "မှတ်စု.txt" သို့ ရေး မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။
"မှတ်စု.txt" ၌ရှိသော ဖိုင်ကိုဖတ် သည့် ရလဒ်ကောင်း ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
မှတ်စု သည့် ရလဒ် ကို ပြသမည်။
"မရှိ.txt" ၌ရှိသော ဖိုင်ကိုဖတ် သည့် ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရလဒ်ကောင်း("မင်္ဂလာပါ")
အမှားအယွင်း(အချက်အလက်အဝင်အထွက်အမှားအယွင်းများထဲမှ မတွေ့ခြင်း)
```

## Writing to a stream

```text
<text> ကို <stream> သို့ ရေး
```

When we use `ရေး` with `စံပြုအထွက်`, it writes with no line break, so a prompt and the
answer can share a line, and what it writes keeps its order with what `ပြသမည်` prints. A
write to `စံပြုအမှား` goes to standard error, apart from the output.

```zaka
"စံကား/အဝင်အထွက်" မှ ရေး၊ စံပြုအထွက် နှင့် အချက်အလက်အဝင်အထွက်အမှားအယွင်း တို့ကို ထည့်သွင်းမည်။

ရေတွက် ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် အချက်အလက်အဝင်အထွက်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
၁ မှ ၃ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"{ကိန်း} " ကို စံပြုအထွက် သို့ ရေး မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
"ပြီး" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ရေတွက် သည့် ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁ ၂ ၃ ပြီး
အောင်မြင်သည့်သတင်း
```

## Reading a line

```text
စံပြုအဝင် မှ စာကြောင်းတစ်ကြောင်းဖတ်
```

We use `စံပြုအဝင် မှ စာကြောင်းတစ်ကြောင်းဖတ်` to get the next line of standard input,
without its line break. At the end of the input, it fails with
`အဝင်အချက်အလက်အဆုံးကိုရောက်ခြင်း`, so a loop that reads each line tests the whole
result. On these pages, the `stdin` fence under a program shows what the program reads:

```zaka
"စံကား/အဝင်အထွက်" မှ စာကြောင်းတစ်ကြောင်းဖတ် နှင့် စံပြုအဝင် တို့ကို ထည့်သွင်းမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လိုင်း သည် စံပြုအဝင် မှ စာကြောင်းတစ်ကြောင်းဖတ် သည့် ရလဒ် ဖြစ်ပါစေ။
လိုင်း ထဲတွင် ရလဒ်ကောင်း ရှိနေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လိုင်း ထဲမှ ဒေတာ ကို ပြသမည်။
လိုင်း ကို စံပြုအဝင် မှ စာကြောင်းတစ်ကြောင်းဖတ် သည့် ရလဒ် ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```stdin
က
ခ
```

```text
က
ခ
```

## Where the input and the files come from

`zakac run` gives a program the terminal: standard input is the input of the process, and
a path names a file on disk, relative to the folder where the command runs. The Learn and
Docs pages give a program a world in storage instead: the input in their `stdin` fence,
and files that last as long as the run. The explorer asks for each line on its output
panel, as a terminal does ([Running a
program](../vm/running.md#input-files-and-the-standard-streams)).

`zakac run` grants a program no right to the files or the environment unless its flags do:
`--allow-read`, `--allow-write` and `--allow-env`, each with an optional list of paths or
names ([Permissions](../vm/running.md#permissions)). A call without the right asks on a
terminal, and then fails with `ခွင့်ပြုချက်မရှိခြင်း` when the right is not given, as it
does for a file that the OS refuses. Reading input and writing the standard streams need
no right. The pages and the explorer run in storage and grant each right.

## Files and folders

| Call | What it does |
| --- | --- |
| `<text> ကို <path> ၏ နောက်ဆုံးမှဆက်ရေး` | adds the text at the end of the file, and makes the file when it is not there |
| `<path> ရှိ ဖိုင်ကို ဖျက်` | removes the file, or the folder when it has nothing in it |
| `<path> နေရာတွင် ဖိုင်တွဲတစ်ခုပြုလုပ်` | makes the folder, and each folder above it that is not there |
| `<path> ရှိ ဖိုင်တွဲအတွင်းရှိ အမည်များကို စု` | the names in the folder, sorted on their u8s, as a `စာသား စာရင်း` |
| `<path> ကို အမျိုးအစားခွဲ` | what is at the path, as a member of `ဖိုင်စနစ်ပစ္စည်း`: `ဖိုင်` (a file), `ဖိုင်တွဲ` (a folder) or `တခြားပစ္စည်း` (another thing); a path with nothing at it fails with `မတွေ့ခြင်း` |
| `<old> ကို <new> သို့ ရွှေ့` | moves the file or the folder, and replaces what is at the new path |
| `<path> တွင် တစ်ခုခုရှိမရှိစစ်` | whether a file, a folder or another thing is at the path |

- Each of these except `တစ်ခုခုရှိမရှိစစ်` can fail with a member of
  `အချက်အလက်အဝင်အထွက်အမှားအယွင်း`. A folder with something in it cannot be removed
  (`တခြားအကြောင်းအရင်း`), and a name that is not UTF-8 text fails the listing
  (`စာသားဖွဲ့စည်းပုံမမှန်ခြင်း`).
- Each of these functions, and `ဖိုင်ကိုဖတ်` and `ရေး` on a file, accepts a path as text
  or as a `ဖိုင်လမ်းကြောင်း` of [the path module](stdlib.md#paths), which this module
  imports.
- The prepositions `ရှိ ဖိုင်ကို` and `ရှိ ဖိုင်တွဲအတွင်းရှိ အမည်များကို` contain several
  words. In a file that imports these functions, `ဖိုင်ကို`, `ဖိုင်တွဲအတွင်းရှိ`,
  `အမည်များကို` and `နေရာတွင်` are taken as words of their prepositions ([What the
  flexible grammar costs](functions-sentences.md#what-the-flexible-grammar-costs)).
- In a world in storage (a Learn page, the explorer), the folders are those that the
  program made and each folder above a file. On disk, each call is the call of the OS.

```zaka
"စံကား/အဝင်အထွက်" မှ ရေး၊ ဖိုင်ကိုဖတ်၊ နောက်ဆုံးမှဆက်ရေး၊ ဖျက်၊ ဖိုင်တွဲတစ်ခုပြုလုပ်၊ စု၊ အမျိုးအစားခွဲ၊ ရွှေ့ နှင့် တစ်ခုခုရှိမရှိစစ် တို့ကို ထည့်သွင်းမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"work/notes" နေရာတွင် ဖိုင်တွဲတစ်ခုပြုလုပ် သည့် ရလဒ် ကို ပြသမည်။
"က" ကို "work/a.txt" သို့ ရေး သည့် ရလဒ် ကို ပြသမည်။
"ခ" ကို "work/a.txt" ၏ နောက်ဆုံးမှဆက်ရေး သည့် ရလဒ် ကို ပြသမည်။
"work/a.txt" ၌ရှိသော ဖိုင်ကိုဖတ် သည့် ရလဒ် ကို ပြသမည်။
"work" ရှိ ဖိုင်တွဲအတွင်းရှိ အမည်များကို စု သည့် ရလဒ် ကို ပြသမည်။
"work/notes" ကို အမျိုးအစားခွဲ သည့် ရလဒ် ကို ပြသမည်။
"work/a.txt" ကို "work/notes/b.txt" သို့ ရွှေ့ သည့် ရလဒ် ကို ပြသမည်။
"work/a.txt" တွင် တစ်ခုခုရှိမရှိစစ် သည့် ရလဒ် ကို ပြသမည်။
"work/notes" ရှိ ဖိုင်ကို ဖျက် သည့် ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အောင်မြင်သည့်သတင်း
အောင်မြင်သည့်သတင်း
အောင်မြင်သည့်သတင်း
ရလဒ်ကောင်း("ကခ")
ရလဒ်ကောင်း(["a.txt"၊ "notes"])
ရလဒ်ကောင်း(ဖိုင်စနစ်ပစ္စည်းများထဲမှ ဖိုင်တွဲ)
အောင်မြင်သည့်သတင်း
မမှန်
အမှားအယွင်း(အချက်အလက်အဝင်အထွက်အမှားအယွင်းများထဲမှ တခြားအကြောင်းအရင်း)
```

## What the stdlib contains

Next to this module, the stdlib (standard library) has `"စံကား/စာသား"` (finding,
splitting and joining text, and text as an integer), `"စံကား/လမ်းကြောင်း"` (paths) and
`"စံကား/စာရင်း"` (sorting, removing, finding and reversing the cells of a list), which
[The stdlib](stdlib.md) describes. A table has its own reads and walks in the language
([Tables](tables.md)).

## The rules

- An import path that starts with `စံကား/` names a module of the stdlib, which the
  compiler carries, so it is never read from disk. Another name after `စံကား/` is ZK0238.
- A program read from storage (the explorer, a Learn page, standard input) can import from
  the stdlib, but from no other file (ZK0232).
- The VM fixes the set of external functions, so a program cannot declare others
  ([External functions](functions-c.md#external-functions), ZK0334).
- A program that imports from this module reads the path module too, because this module
  imports it.
