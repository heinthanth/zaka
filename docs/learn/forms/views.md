---
title: Borrowing text
description: A view reads text where it is, with no duplicate - lend whole text to a function, or take a run of its cells.
---

# Borrowing text

Part I gave each variable and each parameter text of its own: a duplicate, or the text itself
at its last use. A *view* reads text where it is instead, and copies nothing. Its kind is
`စာသား တစ်စိတ်တစ်ပိုင်း` ("a part of text"). A view is a [slice](../lists/slices.md) of
the cells of the text, and it follows the same rules.

## Lending text

`<text> ၏ တန်ဖိုးအငှား` ("the value, on loan") lends the whole text as a view. A
function whose parameter is a view reads the caller's text, with no duplicate:

```zaka
လုပ်ဆောင်ချက် နှုတ်ဆက် သည်
စာသား တစ်စိတ်တစ်ပိုင်း အမည် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ {အမည်}" ဟု ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် နှစ်ခါ သည်
စာသား တစ်စိတ်တစ်ပိုင်း စာ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား ရလဒ် ကို ပြန်ပေးမည်။
စာ + " " + စာ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မောင်မောင်" ဖြစ်ပါစေ။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် နှုတ်ဆက် ကို အမည် ၏ တန်ဖိုးအငှား တို့ဖြင့် လုပ်ဆောင်ပါ။
ကိန်းရှင် ထပ် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် နှစ်ခါ ကို အမည် ၏ တန်ဖိုးအငှား တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ထပ် ကို ပြသပါ။
အမည် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ မောင်မောင်
မောင်မောင် မောင်မောင်
မောင်မောင်
```

| Zaka | Reads as |
| --- | --- |
| `စာသား တစ်စိတ်တစ်ပိုင်း အမည်` | "a part of text, `အမည်`": a view parameter |
| `အမည် ၏ တန်ဖိုးအငှား` | "the value of the name, on loan" |

`အမည်` still holds its text at the end: the functions borrowed it and gave it back. A
view prints, joins and fills a template slot as text does.

## Part of text

`<text> ၏ အကွက် <start> မှစ၍ <length> ကွက်` is a view of `length` cells, from cell
`start` on, counted from 0. A Latin letter takes one cell and a Burmese letter three, as
[Cells](../values/strings.md#cells) shows:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "Zaka စံကား" ဖြစ်ပါစေ။
စာ ၏ အကွက် ၀ မှစ၍ ၄ ကွက် ကို ပြသပါ။      // a view of four cells
စာ ၏ အကွက် ၅ မှစ၍ ၁၅ ကွက် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
Zaka
စံကား
```

| Zaka | Reads as |
| --- | --- |
| `အကွက် ၀ မှစ၍` | "starting at cell 0" |
| `၄ ကွက်` | "four cells" |

A part of text must start and end between two characters, so that text holds whole
characters at each step. `ကမ္ဘာ` takes 15 cells, three for each of its five characters,
so six cells from cell 0 are its first two characters, and two cells cut one in half:

```zaka-stop
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "ကမ္ဘာ" ဖြစ်ပါစေ။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စာ ၏ အကွက် ၀ မှစ၍ ၆ ကွက် ကို ပြသပါ။
စာ ၏ အကွက် ၀ မှစ၍ ၂ ကွက် ကို ပြသပါ။      // ends inside the first character
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ကမ
the program stopped: a range of text at 0x1032 starts or ends inside a character
```

A part of text goes wherever a view does, so a function with a view parameter takes it as
it takes the whole text:

```zaka
လုပ်ဆောင်ချက် နှုတ်ဆက် သည်
စာသား တစ်စိတ်တစ်ပိုင်း အမည် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ {အမည်}" ဟု ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "Zaka စံကား" ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် နှုတ်ဆက် ကို စာ ၏ အကွက် ၅ မှစ၍ ၁၅ ကွက် တို့ဖြင့် လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ စံကား
```

## View mistakes

Text does not turn into a view on its own, a literal included. Hold it in a variable and
lend the variable, so the reader sees the loan; the help line of the error tells how:

```zaka-error ZK0401
လုပ်ဆောင်ချက် နှုတ်ဆက် သည်
စာသား တစ်စိတ်တစ်ပိုင်း အမည် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ {အမည်}" ဟု ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် နှုတ်ဆက် ကို "မမ" တို့ဖြင့် လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0401]: expected text[..], found text
  = help: hold the value in a variable and lend its cells with "<variable> ၏ တန်ဖိုးအငှား"
```

A view borrows the text, so the text must stand still while the view is in use. Setting
the text to new text frees its old cells, and a view read after that is an error:

```zaka-error ZK0704
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "abc" ဖြစ်ပါစေ။
ကိန်းရှင် v သည် စာသား တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် စာ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
စာ ကို "xyz" ဟု ထားပါ။
v ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0704]: the block of "စာ" is freed here, but "v" still borrows it and is used later
```

A [character write](../values/strings.md#changing-a-character) or an
[add](../values/strings.md#adding-to-text) changes the text in place, and an add can move
its cells to a larger block. Each ends the views of the text in the same manner:

```zaka-error ZK0704
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "abc" ဖြစ်ပါစေ။
ကိန်းရှင် v သည် စာသား တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် စာ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
စာ ထဲသို့ "d" ကို ထည့်ပါ။
v ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0704]: the block of "စာ" is edited, so its cells can move, here, but "v" still borrows it and is used later
```

Use the view before the text changes, or take it again after. A view read as the value of
the add itself is fine, since the add reads it first: `စာ ထဲသို့ v ကို ထည့်ပါ။` doubles `စာ`.

A view reads text that another place owns, so the view cannot change that text: a
character write or an add on a view is an error. Build text of its own from the view
first, such as `"{v}"`.

```zaka-error ZK0438
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "abc" ဖြစ်ပါစေ။
ကိန်းရှင် v သည် စာသား တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် စာ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
v ၏ စာလုံး ၀ ကို 'x' ဟု ထားပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0438]: text[..] cannot change: a character write or an add takes text (စာသား)
```

[Views](../../manual/language/text.md#views) in the Docs gives each rule.

## Hands on

- Write a function with a view parameter that prints its text twice, and call it with a
  whole text and with a part of it.
- Print the first two characters of a Burmese word with a part of text, then take one cell
  less.
- Take a view of a variable, set the variable, then print the view, and read the message.

Next: [Slices](../lists/slices.md).
