---
title: Text
description: Text in double quotes is a value of the kind စာသား - store it, fill template slots in it with values, join it, lend it to a function and read its cells.
---

# Text

Text in double quotes, such as `"မင်္ဂလာပါ"`, is a *string literal*. Its kind is `စာသား`
("text"). A program can print text, store it in a variable, pass it to a function,
return it from one, and join two pieces of it.

```zaka
"မင်္ဂလာပါ" ဟု ပြသပါ။
"ဆို \"ဟယ်လို\"" ဟု ပြသပါ။               // \" puts a double quote inside
"ပထမကြောင်း{နောက်တစ်ကြောင်း}ဒုတိယကြောင်း" ဟု ပြသပါ။
```

```text
မင်္ဂလာပါ
ဆို "ဟယ်လို"
ပထမကြောင်း
ဒုတိယကြောင်း
```

Strings use the same [escape sequences](characters.md#escape-sequences) as characters.
Print a string literal with `ဟု`, which quotes it, rather than `ကို`.

## Storing text

A variable of the kind `စာသား` holds text. The variable is a plain value, so a print marks
it with `ကို`, as it would a number.

```zaka
ကိန်းရှင် အမည် သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "မောင်မောင်" ဖြစ်ပါစေ။
အမည် ကို ပြသပါ။
အမည် ကို "အောင်အောင်" ဟု ထားပါ။
အမည် ကို ပြသပါ။
```

```text
မောင်မောင်
အောင်အောင်
```

A `စာသား` declared with no value holds no text at all, and prints as a blank line.

## Template literals

A pair of braces inside a string is a *template slot*. It holds an expression, and the program
writes the value of that expression in its place, the same as `ပြသပါ` would print it.

```zaka
ကိန်းရှင် အမည် သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "မောင်မောင်" ဖြစ်ပါစေ။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ၂၅ ဖြစ်ပါစေ။
"{အမည်} သည် အသက် {အသက်} နှစ် ဖြစ်သည်" ဟု ပြသပါ။
"နောက်နှစ်တွင် {အသက် + ၁} နှစ်" ဟု ပြသပါ။
"{၁.၅} {မှန်} {'က'} {[၁၊ ၂၊ ၃]}" ဟု ပြသပါ။
"{{ မဟုတ် }}" ဟု ပြသပါ။                 // a doubled brace is a brace
```

```text
မောင်မောင် သည် အသက် ၂၅ နှစ် ဖြစ်သည်
နောက်နှစ်တွင် ၂၆ နှစ်
၁.၅ မှန် က [၁၊ ၂၊ ၃]
{ မဟုတ် }
```

A template slot ends at the `}` that matches its `{`, so a template slot can hold text in quotes with
template slots of its own:

```zaka
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် ၉၅ ဖြစ်ပါစေ။
"ရလဒ်: {"အမှတ် {အမှတ်}" + " (အောင်)"}" ဟု ပြသပါ။
```

```text
ရလဒ်: အမှတ် ၉၅ (အောင်)
```

A brace that is part of the text is doubled: `{{` and `}}`. The escape names, such as
`{နောက်တစ်ကြောင်း}`, keep their old meaning.

A mistake inside a template slot is reported at its place inside the quotes:

```zaka-error ZK0301
"{မသိ}" ဟု ပြသပါ။
```

```text
error[ZK0301]: "မသိ" is not declared
```

When each template slot holds a value the compiler knows, such as a literal or a
[constant](../variables/constants.md), the compiler writes the whole text itself, and the
program does no work for it when it runs.

A text constant is such text too. Its value must be known when the program is compiled,
and each read of it is a duplicate, so it is never left blank:

```zaka
ကိန်းသေ အမည် သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "မမ" ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "ဟေး {အမည်}" ဖြစ်ပါစေ။
ခ ကို ပြသပါ။
အမည် ကို ပြသပါ။
```

```text
ဟေး မမ
မမ
```

A text constant built from a variable is an error (ZK0606): declare such text with
`ကိန်းရှင်`.

## Joining text

`+` joins two pieces of text into new text.

```zaka
ကိန်းရှင် အမည် သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "မောင်မောင်" ဖြစ်ပါစေ။
ကိန်းရှင် နှုတ်ဆက် သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "မင်္ဂလာပါ " + အမည် ဖြစ်ပါစေ။
နှုတ်ဆက် ကို ပြသပါ။
နှုတ်ဆက် ကို နှုတ်ဆက် + "!" ဟု ထားပါ။
နှုတ်ဆက် ကို ပြသပါ။
```

```text
မင်္ဂလာပါ မောင်မောင်
မင်္ဂလာပါ မောင်မောင်!
```

Several pieces join in one sentence too, with `တို့ကို ဆက်ထားသည့် စာသား` ("the text that
joins these"):

```zaka
ကိန်းရှင် အမည် သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "မောင်မောင်" ဖြစ်ပါစေ။
"မင်္ဂလာပါ "၊ အမည်၊ "!" တို့ကို ဆက်ထားသည့် စာသား ကို ပြသပါ။
```

```text
မင်္ဂလာပါ မောင်မောင်!
```

Text joins to text alone. `+` with a number is an error, and two pieces of text cannot be
compared with `==` so far:

```zaka-error ZK0403
ကိန်းရှင် စာ သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "အဆင့် " ဖြစ်ပါစေ။
စာ + ၁ ကို ပြသပါ။
```

```text
error[ZK0403]: + needs a number, found text
```

To put a number into text, use a template slot: `"အဆင့် {၁}"`.

## Moving text

A `စာသား` variable owns its text. Putting it into another variable *moves* the text, the
same as an [owner address](../pointers/ownership.md): the old variable gives it up and
cannot be read after that.

```zaka-error ZK0701
ကိန်းရှင် ပထမ သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "Zaka" ဖြစ်ပါစေ။
ကိန်းရှင် ဒုတိယ သည် စာသား ဖြစ်ပါစေ။
ဒုတိယ ကို ပထမ ဟု ထားပါ။
ပထမ ကို ပြသပါ။
```

```text
error[ZK0701]: "ပထမ" was moved to another owner and now holds ဗလာ
```

Passing a `စာသား` to a function moves it too.

## Lending text

To let a function read text without taking it, lend it. The kind
`စာသား တစ်စိတ်တစ်ပိုင်း` ("a part of text") is a *view* of text, and
`<text> ၏ တန်ဖိုးအငှား` ("the value, on loan") makes one. A view is a
[slice](../lists/slices.md), and it follows the same rules.

```zaka
လုပ်ဆောင်ချက် နှုတ်ဆက် သည်
စာသား တစ်စိတ်တစ်ပိုင်း အမည် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ {အမည်}" ဟု ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။

လုပ်ဆောင်ချက် နှစ်ခါ သည်
စာသား တစ်စိတ်တစ်ပိုင်း စာ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား ရလဒ် ကို ပြန်ပေးမည်။
စာ + " " + စာ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။

ကိန်းရှင် အမည် သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "မောင်မောင်" ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် နှုတ်ဆက် ကို အမည် ၏ တန်ဖိုးအငှား တို့ဖြင့် လုပ်ဆောင်ပါ။
ကိန်းရှင် ထပ် သည် စာသား ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် နှစ်ခါ ကို အမည် ၏ တန်ဖိုးအငှား တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ထပ် ကို ပြသပါ။
အမည် ကို ပြသပါ။
```

```text
မင်္ဂလာပါ မောင်မောင်
မောင်မောင် မောင်မောင်
မောင်မောင်
```

`အမည်` still holds its text at the end: the functions borrowed it and gave it back.

## Cells

Text is stored as UTF-8, a row of small numbers from 0 to 255. Each of them is a *cell*
of kind `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်`. A Latin letter takes one cell, and a Burmese letter
takes three.

```zaka
ကိန်းရှင် စာ သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "Zaka စံကား" ဖြစ်ပါစေ။
စာ ၏ အကွက်အရေအတွက် ကို ပြသပါ။          // the count of cells
စာ ၏ အကွက် ၀ ကို ပြသပါ။                // the first cell: Z
စာ ၏ အကွက် ၀ မှစ၍ ၄ ကွက် ကို ပြသပါ။      // a view of four cells
စာ ၏ အကွက် ၅ မှစ၍ ၁၅ ကွက် ကို ပြသပါ။
```

```text
၂၀
၉၀
Zaka
စံကား
```

A range of cells is a view. Nothing checks that it starts and ends at the edge of a
letter, so count the cells with care. The cells of text can be read but not written:

```zaka-error ZK0429
ကိန်းရှင် စာ သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "abc" ဖြစ်ပါစေ။
စာ ၏ အကွက် ၀ ကို ၁ ဟု ထားပါ။
```

```text
error[ZK0429]: the cells of text cannot be written
```

## Characters

Text is stored as UTF-8: a Latin letter takes one u8, a Burmese letter three. Its cells
(`၏ အကွက်`) are those u8s; its characters are read with `၏ စာလုံး`, counted from 0, or
`၏ n ခုမြောက်စာလုံး`, counted from 1, and counted with `၏ စာလုံးအရေအတွက်`:

```zaka
ကိန်းရှင် စာ သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "ကမ္ဘာ" ဖြစ်ပါစေ။
စာ ၏ အကွက်အရေအတွက် ကို ပြသပါ။
စာ ၏ စာလုံးအရေအတွက် ကို ပြသပါ။
စာ ၏ စာလုံး ၀ ကို ပြသပါ။
စာ ၏ ၂ ခုမြောက်စာလုံး ကို ပြသပါ။
```

```text
၁၅
၅
က
မ
```

A part of text, `စာ ၏ အကွက် ၀ မှစ၍ ၆ ကွက်`, must start and end between two characters,
so that text holds whole characters at each step.

## No cleanup

Text built when the program runs needs room on the [heap](../pointers/heap.md), but a
program never frees text. A `စာသား` variable frees its text when its block ends, and when
a new value replaces the old one. A loop frees the text of each round before the next
round starts.

```zaka
ကိန်းရှင် i သည် ကိန်းပြည့် ၁ ဖြစ်ပါစေ။
i သည် ၄ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် လိုင်း သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "အဆင့် {i}: {i * i}" ဖြစ်ပါစေ။
လိုင်း ကို ပြသပါ။
i ကို i + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
```

```text
အဆင့် ၁: ၁
အဆင့် ၂: ၄
အဆင့် ၃: ၉
```

## Hands on

- Print a short poem, one line per `ပြသပါ`.
- Print the same poem with a single `ပြသပါ`, using `{နောက်တစ်ကြောင်း}`.
- Store a name and an age, and print a sentence about them with two template slots.
- Count the cells of a Burmese word. Divide the count with three.

Next: [Variables](../variables/README.md).
