---
title: Input and output
description: Write to the screen and to files, read files and lines, with the stdlib module စံကား/အဝင်အထွက်.
---

# Input and output

A program talks to the world outside it through the stdlib module
`"စံကား/အဝင်အထွက်"` ("input and output"). The compiler carries the module, so a program
imports from it as from a file of its own, and no file needs to sit next to it:

```text
"စံကား/အဝင်အထွက်" မှ ရေး နှင့် ဖိုင်ကိုဖတ် တို့ကို ထည့်သွင်းမည်။
```

## Writing and reading a file

`<text> ကို <path> သို့ ရေး မည်` ("will write the text to the path") writes a whole file,
and `<path> ၌ရှိသော ဖိုင်ကိုဖတ် သည့် ရလဒ်` ("the result of reading the file at the path")
reads it back. Each can fail, as for a file that is not there, so the work goes in a
function that can fail, and each call passes a failure on:

```zaka
"စံကား/အဝင်အထွက်" မှ ရေး၊ ဖိုင်ကိုဖတ် နှင့် အချက်အလက်အဝင်အထွက်အမှားအယွင်း တို့ကို ထည့်သွင်းမည်။

လုပ်ဆောင်ချက် မှတ်စု သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား သို့မဟုတ် အမှားအယွင်းအနေဖြင့် အချက်အလက်အဝင်အထွက်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
"မင်္ဂလာပါ" ကို "မှတ်စု.txt" သို့ ရေး မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။
"မှတ်စု.txt" ၌ရှိသော ဖိုင်ကိုဖတ် သည့် ရလဒ်ကောင်း ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
မှတ်စု သည့် ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရလဒ်ကောင်း("မင်္ဂလာပါ")
```

| Zaka | Reads as |
| --- | --- |
| `"မင်္ဂလာပါ" ကို "မှတ်စု.txt" သို့ ရေး မည်` | "will write 'hello' to notes.txt" |
| `"မှတ်စု.txt" ၌ရှိသော ဖိုင်ကိုဖတ်` | "read the file located at notes.txt" |
| `အချက်အလက်အဝင်အထွက်အမှားအယွင်း` | "data input-output error": the kind of each failure |

## When it fails

A file that is not there gives a failure, and its error value names the class of the
failure, here `မတွေ့ခြင်း` ("not found"):

```zaka
"စံကား/အဝင်အထွက်" မှ ဖိုင်ကိုဖတ် ကို ထည့်သွင်းမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မရှိ.txt" ၌ရှိသော ဖိုင်ကိုဖတ် သည့် ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အမှားအယွင်း(အချက်အလက်အဝင်အထွက်အမှားအယွင်းများထဲမှ မတွေ့ခြင်း)
```

## Writing to the screen

`စံပြုအထွက်` ("standard output") is the screen. `ရေး` to it writes with no line break, so
several writes can share a line:

```zaka
"စံကား/အဝင်အထွက်" မှ ရေး၊ စံပြုအထွက် နှင့် အချက်အလက်အဝင်အထွက်အမှားအယွင်း တို့ကို ထည့်သွင်းမည်။

လုပ်ဆောင်ချက် ပြ သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် အချက်အလက်အဝင်အထွက်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
"မင်္ဂလာ" ကို စံပြုအထွက် သို့ ရေး မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။
"ပါ" ကို စံပြုအထွက် သို့ ရေး မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။
"!" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပြ သည့် ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ!
အောင်မြင်သည့်သတင်း
```

## Reading what the user enters

`စံပြုအဝင် မှ စာကြောင်းတစ်ကြောင်းဖတ်` ("read one line from standard input") gives the
next line the user enters. On a page, the `Standard input` box under the program holds
what the program reads; in the explorer, the output panel asks for each line as a
terminal does; with `zakac run`, it is what the terminal gives:

```zaka
"စံကား/အဝင်အထွက်" မှ ရေး၊ စာကြောင်းတစ်ကြောင်းဖတ်၊ စံပြုအဝင်၊ စံပြုအထွက် နှင့် အချက်အလက်အဝင်အထွက်အမှားအယွင်း တို့ကို ထည့်သွင်းမည်။

လုပ်ဆောင်ချက် နှုတ်ဆက် သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် အချက်အလက်အဝင်အထွက်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
"နာမည်: " ကို စံပြုအထွက် သို့ ရေး မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။
ကိန်းရှင် နာမည် သည် စံပြုအဝင် မှ စာကြောင်းတစ်ကြောင်းဖတ် သည့် ရလဒ်ကောင်း ဖြစ်ပါစေ။
"မင်္ဂလာပါ {နာမည်}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် နှုတ်ဆက် သည့် ရလဒ် ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```stdin
ဟိန်း
```

```text
နာမည်: မင်္ဂလာပါ ဟိန်း
```

The line comes without its line break, so a blank line is the text with no characters, and it is
a good result like each other line. At the end of the input there is no next line, and the
call fails with `အဝင်အချက်အလက်အဆုံးကိုရောက်ခြင်း`; each later read fails alike.

The [Input and output](../../manual/language/io.md) page of the manual lists each name of
the module.

## Reading to the end of the input

A program that is not told the number of lines reads until the end. The loop runs while
the result is a good result, and the test after it tells the end of the input from each
other failure. This program counts three lines, one of them blank:

```zaka
"စံကား/အဝင်အထွက်" မှ စာကြောင်းတစ်ကြောင်းဖတ်၊ စံပြုအဝင် နှင့် အချက်အလက်အဝင်အထွက်အမှားအယွင်း တို့ကို ထည့်သွင်းမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် လိုင်း သည် စံပြုအဝင် မှ စာကြောင်းတစ်ကြောင်းဖတ် သည့် ရလဒ် ဖြစ်ပါစေ။
လိုင်း ထဲတွင် ရလဒ်ကောင်း ရှိနေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အရေအတွက် ကို အရေအတွက် + ၁ ဟု ထားမည်။
လိုင်း ကို စံပြုအဝင် မှ စာကြောင်းတစ်ကြောင်းဖတ် သည့် ရလဒ် ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အကယ်၍ လိုင်း ထဲမှ အမှားအယွင်း ဒေတာ == အချက်အလက်အဝင်အထွက်အမှားအယွင်းများထဲမှ အဝင်အချက်အလက်အဆုံးကိုရောက်ခြင်း ဖြစ်လျှင် "{အရေအတွက်} လိုင်း" ဟု ပြသမည်။
သို့မဟုတ်ပါက "ဖတ်၍မရ" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```stdin
က

ခ
```

```text
၃ လိုင်း
```

The second line is blank, and the loop counts it. When the loop stops, `လိုင်း` holds the
failure; if it is the end of the input, the count is complete, and each other failure
prints `ဖတ်၍မရ`.
