---
title: Input and output
description: Write to the screen and to files, read files and lines, with the stdlib module စံကား/အဝင်အထွက်.
---

# Input and output

The stdlib (standard library) module `"စံကား/အဝင်အထွက်"` handles input and output
(I/O), such as reading a line or writing a file. The module comes with the compiler,
so we can import its functions without adding a file to the program folder.

## You will learn

- How to import names from the stdlib and write and read a whole file.
- How to handle a call that fails, such as a read of a file that is not there.
- How to write to the screen and read the lines the user enters.
- How to read until the end of the input.

## Writing and reading a file

This program writes a word into a file, then reads the file back:

```zaka
"စံကား/အဝင်အထွက်" မှ ရေး၊ ဖိုင်ကိုဖတ် နှင့် အချက်အလက်အဝင်အထွက်အမှားအယွင်း တို့ကို ထည့်သွင်းမည်။

မှတ်စုရေးပြီးဖတ် ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား သို့မဟုတ် အမှားအယွင်းအနေဖြင့် အချက်အလက်အဝင်အထွက်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
"မင်္ဂလာပါ" ကို "မှတ်စု.txt" သို့ ရေး မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။
"မှတ်စု.txt" ၌ရှိသော ဖိုင်ကိုဖတ် သည့် ရလဒ်ကောင်း ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

မှတ်စုရေးပြီးဖတ် သည့် ရလဒ် ကို ပြသမည်။
```

```text
ရလဒ်ကောင်း("မင်္ဂလာပါ")
```

The first line imports three names from the module, in the same form as an import from
another file of our own ([Modules](../modules/README.md)):

```
"စံကား/<module>" မှ <name>၊ <name> နှင့် <name> တို့ကို ထည့်သွင်းမည်။
```

To write a whole file, we use `ရေး`, and to read a whole file back as text (string), we
use `ဖိုင်ကိုဖတ်`:

```
<text> ကို <path> သို့ ရေး မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။
<path> ၌ရှိသော ဖိုင်ကိုဖတ် သည့် ရလဒ်ကောင်း
```

| Zaka | Reads as | What it does |
| --- | --- | --- |
| `"မင်္ဂလာပါ" ကို` | "hello" (object) | the text to write |
| `"မှတ်စု.txt" သို့` | "to notes.txt" | the path of the file |
| `ရေး မည်` | "will write" | writes the text as the whole file |
| `မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်` | "if it fails, return the error" | passes a failure on |
| `"မှတ်စု.txt" ၌ရှိသော ဖိုင်ကိုဖတ်` | "read the file located at notes.txt" | reads the whole file |
| `သည့် ရလဒ်ကောင်း` | "its good result" | the text, or the failure passed on |

A file might be missing or locked, so each of these calls can fail, and its result is a
result kind (result type) whose error value is a member of the enumeration (enum)
`အချက်အလက်အဝင်အထွက်အမှားအယွင်း` ("data input-output error"). For that reason, we put
the work in a function that can fail, `မှတ်စုရေးပြီးဖတ်`, and each call inside it passes a failure
on to the caller ([Passing a failure on](../error-handling/passing.md)). The program
start block (entry point, `main`) cannot pass a failure on, so it prints the whole
result instead.

## When a call fails

When a call fails, its error value tells us the reason. A file that is not there gives
`မတွေ့ခြင်း` ("not found"):

```zaka
"စံကား/အဝင်အထွက်" မှ ဖိုင်ကိုဖတ် ကို ထည့်သွင်းမည်။

"မရှိ.txt" ၌ရှိသော ဖိုင်ကိုဖတ် သည့် ရလဒ် ကို ပြသမည်။
```

```text
အမှားအယွင်း(အချက်အလက်အဝင်အထွက်အမှားအယွင်းများထဲမှ မတွေ့ခြင်း)
```

Here we write `သည့် ရလဒ်` in place of `သည့် ရလဒ်ကောင်း`, which gives the whole result,
good or failed, and the program can print it. [Reading a whole
result](../error-handling/results.md) shows how to test it.

## Permission to read and write

`zakac run` does not let a program touch the files or the environment until we grant it
the right, as Deno does. We grant each right with a flag:

```sh
zakac run --repl --allow-read --allow-write notes.zaka
```

Without the flag, the program asks on the terminal, and a call that does not get the
right fails with `ခွင့်ပြုချက်မရှိခြင်း` ("no permission"). The Run buttons of these pages
keep the files in storage and grant each right.

To read a variable of the environment, we use `ပတ်ဝန်းကျင်ကိန်းရှင် ကို ဖတ်` and the flag
`--allow-env`:

```zaka
"စံကား/အဝင်အထွက်" မှ ဖတ် ကို ထည့်သွင်းမည်။

"HOME" ဟူသည့် ပတ်ဝန်းကျင်ကိန်းရှင် ကို ဖတ်သည့်ရလဒ် ကို ပြသမည်။
```

```text
အမှားအယွင်း(အချက်အလက်အဝင်အထွက်အမှားအယွင်းများထဲမှ မတွေ့ခြင်း)
```

The pages run with no environment, so `HOME` is not set here, and the call fails with
`မတွေ့ခြင်း`.

## Writing to the screen

```
<text> ကို စံပြုအထွက် သို့ ရေး မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။
```

The screen is `စံပြုအထွက်` ("standard output"), and we write to it with the same `ရေး`
that writes a file. Unlike `ပြသမည်`, `ရေး` does not end the line, so several writes can
share one line:

```zaka
"စံကား/အဝင်အထွက်" မှ ရေး၊ စံပြုအထွက် နှင့် အချက်အလက်အဝင်အထွက်အမှားအယွင်း တို့ကို ထည့်သွင်းမည်။

ပြ ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် အချက်အလက်အဝင်အထွက်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
"မင်္ဂလာ" ကို စံပြုအထွက် သို့ ရေး မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။
"ပါ" ကို စံပြုအထွက် သို့ ရေး မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။
"!" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပြ သည့် ရလဒ် ကို ပြသမည်။
```

```text
မင်္ဂလာပါ!
အောင်မြင်သည့်သတင်း
```

`ပြ` has no value to return, so its result kind uses `အောင်မြင်သည့်သတင်း` ("news of
success") as the good side, and that is what the program prints on the last
line ([A side with no value](../error-handling/kinds.md#a-side-with-no-value)).

## Reading what the user enters

```
စံပြုအဝင် မှ စာကြောင်းတစ်ကြောင်းဖတ် သည့် ရလဒ်ကောင်း
```

To read the next line that the user enters, we read one line from `စံပြုအဝင်` ("standard
input"). With `zakac run`, the line comes from the terminal. In the Explorer, the output
panel asks for each line as a terminal does, and on these pages, the `Standard input` box
under the program has the lines that the program reads:

```zaka
"စံကား/အဝင်အထွက်" မှ ရေး၊ စာကြောင်းတစ်ကြောင်းဖတ်၊ စံပြုအဝင်၊ စံပြုအထွက် နှင့် အချက်အလက်အဝင်အထွက်အမှားအယွင်း တို့ကို ထည့်သွင်းမည်။

နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် အချက်အလက်အဝင်အထွက်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
"နာမည်: " ကို စံပြုအထွက် သို့ ရေး မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။
ကိန်းရှင် နာမည် သည် စံပြုအဝင် မှ စာကြောင်းတစ်ကြောင်းဖတ် သည့် ရလဒ်ကောင်း ဖြစ်ပါစေ။
"မင်္ဂလာပါ {နာမည်}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ရ သည် နှုတ်ဆက် သည့် ရလဒ် ဖြစ်ပါစေ။
```

```stdin
ဟိန်း
```

```text
နာမည်: မင်္ဂလာပါ ဟိန်း
```

The line comes without its line break. On a terminal, the user presses Enter after the
name, so the greeting starts on a new line, while on this page the input is not echoed,
and the greeting follows the prompt on the same line.

## Reading to the end of the input

When a program does not know the number of lines to come, it reads until the end of
the input. At the end, there is no next line, so the read fails with
`အဝင်အချက်အလက်အဆုံးကိုရောက်ခြင်း` ("reached the end of the input"). We can loop while
the result is a good result, and after the loop, test which failure stopped it:

```
<variable> ထဲတွင် ရလဒ်ကောင်း ရှိနေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
```

This program counts three lines, and one of them is blank:

```zaka
"စံကား/အဝင်အထွက်" မှ စာကြောင်းတစ်ကြောင်းဖတ်၊ စံပြုအဝင် နှင့် အချက်အလက်အဝင်အထွက်အမှားအယွင်း တို့ကို ထည့်သွင်းမည်။

ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် လိုင်း သည် စံပြုအဝင် မှ စာကြောင်းတစ်ကြောင်းဖတ် သည့် ရလဒ် ဖြစ်ပါစေ။
လိုင်း ထဲတွင် ရလဒ်ကောင်း ရှိနေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အရေအတွက် ကို အရေအတွက် + ၁ ဟု ထားမည်။
လိုင်း ကို စံပြုအဝင် မှ စာကြောင်းတစ်ကြောင်းဖတ် သည့် ရလဒ် ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အကယ်၍ လိုင်း ထဲမှ အမှားအယွင်း ဒေတာ == အချက်အလက်အဝင်အထွက်အမှားအယွင်းများထဲမှ အဝင်အချက်အလက်အဆုံးကိုရောက်ခြင်း ဖြစ်လျှင် "{အရေအတွက်} လိုင်း" ဟု ပြသမည်။
သို့မဟုတ်ပါက "ဖတ်၍မရ" ဟု ပြသမည်။
```

```stdin
က

ခ
```

```text
၃ လိုင်း
```

A blank line is a good result too: text with no characters. So the loop counts the
second line, and it stops after the third. When the loop ends, `လိုင်း` has the failure,
and when that failure is the end of the input, the count is complete. Each other failure
prints `ဖတ်၍မရ` ("cannot read").

## Details

- **What the module has.** Besides `ရေး`, `ဖိုင်ကိုဖတ်` and `စာကြောင်းတစ်ကြောင်းဖတ်`, the
  module has standard error (`စံပြုအမှား`), and calls to add to a file, remove it, move
  it, make a folder and list a folder. [Input and
  output](../../manual/language/io.md#what-the-module-contains) lists each name, and [The
  stdlib](../stdlib/README.md#files-and-folders) shows the folder calls.
- **The members of the error enumeration.** Besides `မတွေ့ခြင်း`,
  `ခွင့်ပြုချက်မရှိခြင်း` and `အဝင်အချက်အလက်အဆုံးကိုရောက်ခြင်း`, a call can fail with
  `စာသားဖွဲ့စည်းပုံမမှန်ခြင်း` (the file is not UTF-8 text) or `တခြားအကြောင်းအရင်း` (some other
  reason).
- **One name, two functions.** `ရေး` to a path writes a file, and `ရေး` to a stream such as
  `စံပြုအထွက်` writes to the stream. The data kind of the argument after `သို့` picks
  which one runs ([One name, several
  functions](../functions/prepositions.md#one-name-several-functions)).
- **After the end of the input.** Once a read fails with
  `အဝင်အချက်အလက်အဆုံးကိုရောက်ခြင်း`, each later read fails in the same form.
- **Where paths start.** With `zakac run`, a path such as `"မှတ်စု.txt"` is relative to the
  folder that the command runs in, so the file appears there.
- **Rights for one path.** Each flag can name a list of paths or variables, as in
  `--allow-read=data`, to grant the right for these alone ([Permissions](../../manual/vm/running.md#permissions)).

## Recap

- We import from the stdlib with `"စံကား/<module>" မှ <names> တို့ကို ထည့်သွင်းမည်။`, the
  same as from a file of our own.
- `ရေး` writes text to a file or to `စံပြုအထွက်`, and `ဖိုင်ကိုဖတ်` reads a whole file.
- Each call can fail, so we put the calls in a function that can fail and pass each
  failure on, or we print or test the whole result.
- `စံပြုအဝင် မှ စာကြောင်းတစ်ကြောင်းဖတ်` reads one line, and fails with
  `အဝင်အချက်အလက်အဆုံးကိုရောက်ခြင်း` at the end of the input.
- `zakac run` needs `--allow-read`, `--allow-write` or `--allow-env` before a program can
  touch files or the environment.

## Hands on

- Write two lines into a file with one `ရေး`, with `{နောက်တစ်ကြောင်း}` (a line break)
  between them, then read the file back and print it.
- Read a file that is not there and print the whole result. Then run the program with
  `zakac run` and no flags, and see what the terminal asks.
- Change the greeting program so that it asks for a name and a town, and prints both.
- Change the counting program so that it also prints the longest line.

Next: [The stdlib](../stdlib/README.md).
