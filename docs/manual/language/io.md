---
title: Input and output
description: The stdlib module of input and output, its streams and its error enumeration, reading a line, reading and writing a file, and writing to a stream.
---

# Input and output

A program reads and writes through the functions of the stdlib module
`"စံကား/အဝင်အထွက်"`. Each one is an [external function](functions.md#external-functions):
the VM does its work, and each one can fail with a member of the enumeration
`အချက်အလက်အဝင်အထွက်အမှားအယွင်း`. A program imports the names it needs:

```text
"စံကား/အဝင်အထွက်" မှ ရေး၊ ဖိုင်ကိုဖတ်၊ အချက်အလက်အဝင်အထွက်အမှားအယွင်း တို့ကို ထည့်သွင်းမည်။
```

## What the module holds

| Name | What it is |
| --- | --- |
| `အချက်အလက်အဝင်အထွက်အမှားအယွင်း` | the enumeration of failures: `မတွေ့ခြင်း` (no such file), `ခွင့်ပြုချက်မရှိခြင်း` (not allowed), `အဝင်အချက်အလက်အဆုံးကိုရောက်ခြင်း` (the end of the input), `စာသားဖွဲ့စည်းပုံမမှန်ခြင်း` (not UTF-8 text), `တခြားအကြောင်းအရင်း` (each other failure) |
| `အချက်အလက်အဝင်စီးကြောင်း` | a stream to read from |
| `အချက်အလက်အထွက်စီးကြောင်း` | a stream to write to |
| `စံပြုအဝင်` | standard input, a stream to read from |
| `စံပြုအထွက်`, `စံပြုအမှား` | standard output and standard error, streams to write to |
| `<stream> မှ စာကြောင်းတစ်ကြောင်းဖတ်` | one line of the stream, without its line break: text |
| `<path> ၌ရှိသော ဖိုင်ကိုဖတ်` | the whole file as text |
| `<text> ကို <path> သို့ ရေး` | write the text as the whole of the file |
| `<text> ကို <stream> သို့ ရေး` | write the text to the stream, with no line break |

The calls are [marked calls](functions.md#marked-parameters), and `ရေး` is
[two functions of one name](functions.md#one-name-several-functions): the kind of the
argument at `သို့` chooses a file path or a stream. Each other call form works too.

## Calling a function that can fail

The result of each function can fail, so a call passes the failure on, reads the whole
result, or both (see [Errors and results](error-handling.md)). The start block cannot pass
a failure on, so a program does its input and output in a function that can fail, and the
start block reads its whole result:

```zaka
"စံကား/အဝင်အထွက်" မှ ရေး၊ ဖိုင်ကိုဖတ်၊ အချက်အလက်အဝင်အထွက်အမှားအယွင်း တို့ကို ထည့်သွင်းမည်။

လုပ်ဆောင်ချက် မှတ်စု သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား သို့မဟုတ် အမှားအယွင်းအနေဖြင့် အချက်အလက်အဝင်အထွက်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
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

`ရေး` to `စံပြုအထွက်` writes with no line break, so a prompt and the answer can share a
line; what it writes keeps its order with what `ပြသမည်` prints. A write to `စံပြုအမှား`
goes to standard error, apart from the output.

```zaka
"စံကား/အဝင်အထွက်" မှ ရေး၊ စံပြုအထွက်၊ အချက်အလက်အဝင်အထွက်အမှားအယွင်း တို့ကို ထည့်သွင်းမည်။

လုပ်ဆောင်ချက် ရေတွက် သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် အချက်အလက်အဝင်အထွက်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
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

`စံပြုအဝင် မှ စာကြောင်းတစ်ကြောင်းဖတ်` gives the next line of standard input, without its
line break, and at the end of the input it fails with
`အဝင်အချက်အလက်အဆုံးကိုရောက်ခြင်း`. A loop that reads each line tests the whole result;
the `stdin` fence under a program of these pages holds what it reads:

```zaka
"စံကား/အဝင်အထွက်" မှ စာကြောင်းတစ်ကြောင်းဖတ်၊ စံပြုအဝင် တို့ကို ထည့်သွင်းမည်။

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

`zakac run` gives a program the terminal: standard input is the input of the process,
and a path names a file on disk, relative to the folder the command runs in. The Learn
and Docs pages give a world in storage: the input their `stdin` fence holds, and files
that last as long as the run. The explorer asks for each line on its output panel, as a
terminal does (see [Running a program](../vm/running.md#input-files-and-the-standard-streams)). See [Running a program](../vm/running.md#input-files-and-the-standard-streams).

## The rules

- An import path that starts with `စံကား/` names a module of the stdlib, which the
  compiler carries; it is never read from disk. Another name after `စံကား/` is ZK0238.
- A program read from storage (the explorer, a Learn page, standard input) can import
  from the stdlib, and no other file (ZK0232).
- The VM fixes the set of functions: a program cannot declare others
  ([External functions](functions.md#external-functions), ZK0334).
