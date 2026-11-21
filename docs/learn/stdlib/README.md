---
title: The stdlib
description: Use the modules the compiler carries for text, paths, lists, files, math and time.
---

# The stdlib

The stdlib (standard library) comes with the compiler and includes functions for text,
paths, lists, files, folders, math and time. We can import and call these functions in the same
form as functions from our own modules. See [Modules](../modules/README.md) for how
imports work.

## You will learn

- How to import a function from the stdlib and call it in a sentence.
- How to find, split, join and change text, and read text as an integer.
- How to build a path and take it apart.
- How to sort and reverse a list, and how to make folders and list them.

## Text

This program imports four functions of the text module, `"စံကား/စာသား"`, and calls each
one:

```zaka
"စံကား/စာသား" မှ ပါမပါစစ်၊ ခွဲ၊ ဆက် နှင့် စာလုံးကြီးပြောင်း တို့ကို ထည့်သွင်းမည်။

"ကမ္ဘာ" ကို "မင်္ဂလာပါ ကမ္ဘာ" ထဲတွင် ပါမပါစစ် သည့် ရလဒ် ကို ပြသမည်။
ကိန်းရှင် သီးများ သည် "ပန်း၊ငှက်ပျော၊သရက်" ကို "၊" ဖြင့် ခွဲ သည့် ရလဒ် ဖြစ်ပါစေ။
သီးများ ကို ပြသမည်။
သီးများ ကို " / " ဖြင့် ဆက် သည့် ရလဒ် ကို ပြသမည်။
"zaka" ကို စာလုံးကြီးပြောင်း သည့် ရလဒ် ကို ပြသမည်။
```

```text
မှန်
["ပန်း"၊ "ငှက်ပျော"၊ "သရက်"]
ပန်း / ငှက်ပျော / သရက်
ZAKA
```

Each call is a sentence: we write the arguments, each followed with its preposition, then
the name of the function and `သည့် ရလဒ်` to use its result. To split text, for example, we
write:

```
<text> ကို <separator> ဖြင့် ခွဲ သည့် ရလဒ်
```

| Zaka | Reads as | What it does |
| --- | --- | --- |
| `"ပန်း၊ငှက်ပျော၊သရက်" ကို` | "the text" (object) | the text to split |
| `"၊" ဖြင့်` | "with `၊`" | the separator |
| `ခွဲ` | "split" | the function |
| `သည့် ရလဒ်` | "its result" | the value of the call: a list of text |

The other calls follow the same pattern: `ပါမပါစစ်` ("is it there or not?") tells whether
the first text is inside the second, `ဆက်` ("join") joins a list of text with a separator
between each two, and `စာလုံးကြီးပြောင်း` ("to capitals") turns each Latin letter into a
capital.

## Text as an integer

When we import `ကိန်းပြည့်အဖြစ်ပြောင်း` from the text module, the conversion
`ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်` also works on text:

```
<text> ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်
```

Text might not have a number in it, so the conversion returns a result (result type) that
has either the number or an error value, as [Errors and
results](../error-handling/README.md) shows:

```zaka
"စံကား/စာသား" မှ ကိန်းပြည့်အဖြစ်ပြောင်း ကို ထည့်သွင်းမည်။

"၄၂" ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
"လေးဆယ်" ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
```

```text
ရလဒ်ကောင်း(၄၂)
အမှားအယွင်း(ကိန်းဂဏန်းဖတ်ခြင်းအမှားအယွင်းများထဲမှ ကိန်းဂဏန်းမဟုတ်သောစာလုံးပါခြင်း)
```

`"လေးဆယ်"` is the number 40 written as a word, not in digits, so the second conversion
fails with "a character that is not a digit".

## Paths

```
<path> ကို <path> နှင့် ဖိုင်လမ်းကြောင်းဆက် သည့် ရလဒ်
<path> ၏ ဖိုင်အမည်
```

The path module, `"စံကား/လမ်းကြောင်း"`, builds the path of a file and takes it apart.
`ဖိုင်လမ်းကြောင်းဆက်` joins two paths with a `/` between them, and when a function needs
just one argument, we can also call it with `၏`, as if we were reading a field of the
path:

```zaka
"စံကား/လမ်းကြောင်း" မှ ဖိုင်အမည်၊ ဖိုင်အမည်နောက်တွဲ နှင့် ဖိုင်လမ်းကြောင်းဆက် တို့ကို ထည့်သွင်းမည်။

ကိန်းရှင် လမ်း သည် "photos" ကို "ပန်း.jpg" နှင့် ဖိုင်လမ်းကြောင်းဆက် သည့် ရလဒ် ဖြစ်ပါစေ။
လမ်း ကို ပြသမည်။
လမ်း ၏ ဖိုင်အမည် ကို ပြသမည်။
လမ်း ၏ ဖိုင်အမည်နောက်တွဲ ကို ပြသမည်။
```

```text
photos/ပန်း.jpg
ပန်း.jpg
ဒေတာ("jpg")
```

`ဖိုင်အမည်` ("file name") is the last part of the path, and `ဖိုင်အမည်နောက်တွဲ` ("file name
ending") is the extension without its dot. A file name might have no extension, so the
extension comes as an optional value (option type), and the output shows it inside
`ဒေတာ(...)` ([Optional values](../optionals/README.md)).

## Lists

```
<list> ကို ငယ်စဉ်ကြီးလိုက်စီ သည့် ရလဒ်
```

The list module, `"စံကား/စာရင်း"`, sorts a list, removes a cell, finds a value and
reverses the order of the cells. Each function returns a new list and leaves the list we
pass in unchanged, as the last line of the output shows:

```zaka
"စံကား/စာရင်း" မှ ငယ်စဉ်ကြီးလိုက်စီ၊ ဖယ် နှင့် ပြောင်းပြန်စီ တို့ကို ထည့်သွင်းမည်။

ကိန်းရှင် အမှတ်များ သည် [၇၀၊ ၉၅၊ ၆၁၊ ၈၈] ဖြစ်ပါစေ။
အမှတ်များ ကို ငယ်စဉ်ကြီးလိုက်စီ သည့် ရလဒ် ကို ပြသမည်။
အမှတ်များ မှ ၀ အကွက်ကို ဖယ် သည့် ရလဒ် ကို ပြသမည်။
အမှတ်များ ကို ပြောင်းပြန်စီ သည့် ရလဒ် ကို ပြသမည်။
အမှတ်များ ကို ပြသမည်။
```

```text
[၆၁၊ ၇၀၊ ၈၈၊ ၉၅]
[၉၅၊ ၆၁၊ ၈၈]
[၈၈၊ ၆၁၊ ၉၅၊ ၇၀]
[၇၀၊ ၉၅၊ ၆၁၊ ၈၈]
```

| Zaka | Reads as |
| --- | --- |
| `ငယ်စဉ်ကြီးလိုက်စီ` | "sort from small to large" |
| `<list> မှ ၀ အကွက်ကို ဖယ်` | "remove cell 0 from the list" |
| `ပြောင်းပြန်စီ` | "sort in reverse", from the last cell to the first |

## Files and folders

```
<path> နေရာတွင် ဖိုင်တွဲတစ်ခုပြုလုပ် သည့် ရလဒ်
<text> ကို <path> ၏ နောက်ဆုံးမှဆက်ရေး သည့် ရလဒ်
<path> ဖိုင်တွဲထဲမှ အမည်များကို စု သည့် ရလဒ်
```

The module of [Input and output](../io/README.md), `"စံကား/အဝင်အထွက်"`, can also make a
folder, add text at the end of a file and list the names inside a folder. Each of these
calls can fail, so this program prints the whole result of each one:

```zaka
"စံကား/အဝင်အထွက်" မှ ဖိုင်တွဲတစ်ခုပြုလုပ်၊ နောက်ဆုံးမှဆက်ရေး နှင့် စု တို့ကို ထည့်သွင်းမည်။

"notes" နေရာတွင် ဖိုင်တွဲတစ်ခုပြုလုပ် သည့် ရလဒ် ကို ပြသမည်။
"ဒီနေ့" ကို "notes/day.txt" ၏ နောက်ဆုံးမှဆက်ရေး သည့် ရလဒ် ကို ပြသမည်။
"notes" ဖိုင်တွဲထဲမှ အမည်များကို စု သည့် ရလဒ် ကို ပြသမည်။
```

```text
အောင်မြင်သည့်သတင်း
အောင်မြင်သည့်သတင်း
ရလဒ်ကောင်း(["day.txt"])
```

| Zaka | Reads as |
| --- | --- |
| `"notes" နေရာတွင် ဖိုင်တွဲတစ်ခုပြုလုပ်` | "make a folder at notes" |
| `"notes/day.txt" ၏ နောက်ဆုံးမှဆက်ရေး` | "write on at the end of notes/day.txt" |
| `"notes" ဖိုင်တွဲထဲမှ အမည်များကို စု` | "collect the names inside the folder at notes" |

`နောက်ဆုံးမှဆက်ရေး` makes the file when it is not there, which is the reason the folder
lists `day.txt` afterwards.

## Details

- **Math and time.** Import `နှစ်ထပ်ကိန်းရင်း` from `"စံကား/သင်္ချာ"` to find a square
  root. Import `ကြာချိန်နာရီကိုဖတ်` from `"စံကား/အချိန်"` and subtract two readings to
  measure elapsed milliseconds. See [Math](../../manual/language/stdlib.md#math) and
  [Time](../../manual/language/stdlib.md#time) for runnable examples.
- **More functions.** Each module has more functions than this page shows: for example,
  `ရှာ` finds where a piece of text starts, `စာကြောင်းများအဖြစ်ခွဲ` splits text into lines,
  and `စီ` sorts a list with a function that compares two cells. [The
  stdlib](../../manual/language/stdlib.md) lists each function with its rules and its cost.
- **The sign form.** Each stdlib function also works in the sign form, such as
  `ခွဲ("က၊ခ"၊ "၊")` ([The sign form](../functions/results.md#the-sign-form)).
- **Places in text count u8s.** A function that returns a place in text, such as `ရှာ`,
  counts the u8s (bytes) of the text, so a Burmese letter takes three ([Cells](../text/characters.md#cells)).
- **Paths as a data kind.** A path that the module returns is a `ဖိုင်လမ်းကြောင်း`, which
  prints as its text, and each function accepts text as well
  ([Paths](../../manual/language/stdlib.md#paths)).
- **Generic list functions.** The list functions work on a list of each data kind, because
  each one is generic ([Generics](../generics/README.md)).
- **The rest of the file calls.** The I/O module can also remove, move and test a path:
  see [Files and folders](../../manual/language/io.md#files-and-folders).

## Recap

- We import stdlib functions with `"စံကား/<module>" မှ <names> တို့ကို ထည့်သွင်းမည်။`.
- We call each function as a sentence, with the arguments and their prepositions first and
  `သည့် ရလဒ်` after the name.
- `"စံကား/စာသား"` works on text, `"စံကား/လမ်းကြောင်း"` on paths, `"စံကား/စာရင်း"` on lists,
  and `"စံကား/အဝင်အထွက်"` on files and folders.
- The text and list functions return new values and leave their arguments unchanged.

## Hands on

- Split a sentence on `" "` into words, print the number of words, then join the words
  again with `"-"`.
- Sort a list of five numbers, then reverse the sorted list so that the largest comes first.
- Join three folder names into one path with `ဖိုင်လမ်းကြောင်းဆက်`, and print its last part.
- Convert `"၁၂၃"` and `"abc"` to integers and print both results.

Next: [Natural Zaka](../natural-zaka/README.md).
