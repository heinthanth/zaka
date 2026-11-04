---
title: Text
description: Text in double quotes is a value of the kind စာသား - store it, fill template slots in it with values, join it, compare it, read its cells and characters, change a character and add to it.
---

# Text

Text in double quotes, such as `"မင်္ဂလာပါ"`, is a *string literal*. Its kind is `စာသား`
("text"). A program can print text, store it in a variable, pass it to a function,
return it from one, and join two pieces of it.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ" ဟု ပြသပါ။
"ဆို \"ဟယ်လို\"" ဟု ပြသပါ။               // \" puts a double quote inside
"ပထမကြောင်း{နောက်တစ်ကြောင်း}ဒုတိယကြောင်း" ဟု ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မောင်မောင်" ဖြစ်ပါစေ။
အမည် ကို ပြသပါ။
အမည် ကို "အောင်အောင်" ဟု ထားပါ။
အမည် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မောင်မောင်" ဖြစ်ပါစေ။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ၂၅ ဖြစ်ပါစေ။
"{အမည်} သည် အသက် {အသက်} နှစ် ဖြစ်သည်" ဟု ပြသပါ။
"နောက်နှစ်တွင် {အသက် + ၁} နှစ်" ဟု ပြသပါ။
"{၁.၅} {မှန်} {'က'} {[၁၊ ၂၊ ၃]}" ဟု ပြသပါ။
"{{ မဟုတ် }}" ဟု ပြသပါ။                 // a doubled brace is a brace
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် ၉၅ ဖြစ်ပါစေ။
"ရလဒ်: {"အမှတ် {အမှတ်}" + " (အောင်)"}" ဟု ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရလဒ်: အမှတ် ၉၅ (အောင်)
```

A brace that is part of the text is doubled: `{{` and `}}`. The escape names, such as
`{နောက်တစ်ကြောင်း}`, keep their old meaning.

A mistake inside a template slot is reported at its place inside the quotes:

```zaka-error ZK0301
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"{မသိ}" ဟု ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မမ" ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "ဟေး {အမည်}" ဖြစ်ပါစေ။
ခ ကို ပြသပါ။
အမည် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မောင်မောင်" ဖြစ်ပါစေ။
ကိန်းရှင် နှုတ်ဆက် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မင်္ဂလာပါ " + အမည် ဖြစ်ပါစေ။
နှုတ်ဆက် ကို ပြသပါ။
နှုတ်ဆက် ကို နှုတ်ဆက် + "!" ဟု ထားပါ။
နှုတ်ဆက် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ မောင်မောင်
မင်္ဂလာပါ မောင်မောင်!
```

Several pieces join in one sentence too, with `တို့ကို ဆက်ထားသည့် စာသား` ("the text that
joins these"):

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မောင်မောင်" ဖြစ်ပါစေ။
"မင်္ဂလာပါ "၊ အမည်၊ "!" တို့ကို ဆက်ထားသည့် စာသား ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ မောင်မောင်!
```

Text joins to text alone. `+` with a number is an error:

```zaka-error ZK0403
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "အဆင့် " ဖြစ်ပါစေ။
စာ + ၁ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0403]: + needs a number, found text
```

To put a number into text, use a template slot: `"အဆင့် {၁}"`.

## Comparing text

`==` and `!=` tell whether two pieces of text hold the same characters:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည် သည် "မောင်မောင်" ဖြစ်ပါစေ။
အမည် == "မောင်မောင်" ကို ပြသပါ။
အမည် != "မမ" ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
မှန်
```

## Copies of text

A `စာသား` variable owns its text. Putting it into another variable *copies* the text: each
variable then holds its own, and a change to one leaves the other as it was.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပထမ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "Zaka" ဖြစ်ပါစေ။
ကိန်းရှင် ဒုတိယ သည် စာသား တစ်ခု ဖြစ်ပါစေ။
ဒုတိယ ကို ပထမ ဟု ထားပါ။
ဒုတိယ ကို ဒုတိယ + "!" ဟု ထားပါ။
ပထမ ကို ပြသပါ။
ဒုတိယ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
Zaka
Zaka!
```

Passing text to a function copies it too, so a function cannot change the caller's
variable, not even with a character write or an add. When the first variable is not read again, nothing is copied: the compiler
hands its text over to the second. Part II tells what copies cost, and how to read text
with none: see [What a duplicate costs](../forms/costs.md).

## Cells

Text is stored as UTF-8, a row of small numbers from 0 to 255. Each of them is a *cell*
of kind `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်`. A Latin letter takes one cell, and a Burmese letter
takes three.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "Zaka စံကား" ဖြစ်ပါစေ။
စာ ၏ အကွက်အရေအတွက် ကို ပြသပါ။          // the count of cells
စာ ၏ အကွက် ၀ ကို ပြသပါ။                // the first cell: Z
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၀
၉၀
```

The cells of text can be read but not written, since one cell alone can break a Burmese
letter in two. A whole character can be written: see [Changing a
character](#changing-a-character).

```zaka-error ZK0429
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "abc" ဖြစ်ပါစေ။
စာ ၏ အကွက် ၀ ကို ၁ ဟု ထားပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0429]: the cells of text cannot be written
```

## Characters

Text is stored as UTF-8: a Latin letter takes one u8, a Burmese letter three. Its cells
(`၏ အကွက်`) are those u8s; its characters are read with `၏ စာလုံး`, counted from 0, or
`၏ n ခုမြောက်စာလုံး`, counted from 1, and counted with `၏ စာလုံးအရေအတွက်`:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "ကမ္ဘာ" ဖြစ်ပါစေ။
စာ ၏ အကွက်အရေအတွက် ကို ပြသပါ။
စာ ၏ စာလုံးအရေအတွက် ကို ပြသပါ။
စာ ၏ စာလုံး ၀ ကို ပြသပါ။
စာ ၏ ၂ ခုမြောက်စာလုံး ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၅
၅
က
မ
```

An *each-character loop* takes the characters in turn, "taking each character of the
text as *c*, do ...":

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ကမ္ဘာ" ၏ စာလုံးတိုင်း ကို စ အနေဖြင့် ယူ၍ စ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
က
မ
္
ဘ
ာ
```

A Burmese letter and each mark on it are characters of their own.

## Changing a character

A character of text is a place, as a cell of a list is: the assignment writes a new
character there. `၏ စာလုံး` counts from 0, and `၏ n ခုမြောက်စာလုံး` from 1.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "cat" ဖြစ်ပါစေ။
စာ ၏ စာလုံး ၀ ကို 'b' ဟု ထားပါ။
စာ ကို ပြသပါ။
စာ ၏ ၃ ခုမြောက်စာလုံး ကို 'd' ဟု ထားပါ။
စာ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
bat
bad
```

The new value must be a character, in single quotes. Text in double quotes is an error,
even with one character in it:

```zaka-error ZK0401
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "cat" ဖြစ်ပါစေ။
စာ ၏ စာလုံး ၀ ကို "b" ဟု ထားပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0401]: expected char, found text
```

A character is one Unicode code point, not a letter as it is written. `ကျွန်` reads as one
sound, but it is five characters: `က`, `ျ`, `ွ`, `န` and `်`. A write replaces one of
them and keeps the rest:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "ကျွန်" ဖြစ်ပါစေ။
စာ ၏ စာလုံးအရေအတွက် ကို ပြသပါ။
စာ ၏ စာလုံး ၀ ကို 'ခ' ဟု ထားပါ။
စာ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၅
ချွန်
```

A number past the last character stops the program, as a read there does:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "ab" ဖြစ်ပါစေ။
စာ ၏ ၃ ခုမြောက်စာလုံး ကို 'x' ဟု ထားပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: character 3 is outside the text, whose characters run from 1 to 2
```

## Adding to text

`ထဲသို့ ... ကို ထည့်ပါ` ("put ... into") adds text or a character at the end of text:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် နှုတ်ဆက် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မင်္ဂလာပါ" ဖြစ်ပါစေ။
နှုတ်ဆက် ထဲသို့ " မောင်မောင်" ကို ထည့်ပါ။
နှုတ်ဆက် ထဲသို့ '!' ကို ထည့်ပါ။
နှုတ်ဆက် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ မောင်မောင်!
```

An add changes the text in place, where `နှုတ်ဆက် ကို နှုတ်ဆက် + "!" ဟု ထားပါ။` builds new
text and puts it in place of the old. Adds build text one piece at a time, such as in a
[counted loop](../control-flow/counted-loops.md):

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လှေကား သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "" ဖြစ်ပါစေ။
၁ မှ ၄ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
လှေကား ထဲသို့ '*' ကို ထည့်ပါ။
လှေကား ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
*
**
***
****
```

A template literal adds a number: `လှေကား ထဲသို့ "{န}" ကို ထည့်ပါ။`. A number with no
quotes around it is an error, since a number is no text:

```zaka-error ZK0401
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "အဆင့် " ဖြစ်ပါစေ။
စာ ထဲသို့ ၁ ကို ထည့်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0401]: expected text, found isize
```

## Adding in the middle and removing the end

`၏ အရှေ့တွင်` ("in front of") after a character puts the added text in front of it, and
`နောက်ဆုံးစာလုံး` ("the last character") names the end that a remove takes off:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "acd" ဖြစ်ပါစေ။
စာ ၏ စာလုံး ၁ ၏ အရှေ့တွင် "b" ကို ထည့်ပါ။        // in front of c
စာ ၏ ၁ ခုမြောက်စာလုံး ၏ အရှေ့တွင် '>' ကို ထည့်ပါ။   // at the start
စာ ကို ပြသပါ။
စာ ၏ နောက်ဆုံးစာလုံးကို ဖျက်ပါ။
စာ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
>abcd
>abc
```

| Zaka | Reads as |
| --- | --- |
| `စာ ၏ စာလုံး ၁ ၏ အရှေ့တွင် "b" ကို ထည့်ပါ` | "please add "b" in front of character 1 of the text" |
| `စာ ၏ နောက်ဆုံးစာလုံးကို ဖျက်ပါ` | "please remove the last character of the text" |

The number can also be the character count, which adds at the end, as `ထဲသို့` does. A
character is a Unicode code point, so a remove from `ကျွန်` takes off the mark `်` alone.
`နောက်ဆုံးအကွက်` ("the last cell") belongs to lists: see [List
mistakes](../lists/mistakes.md#removing-from-text).

## No cleanup

Text built when the program runs needs storage of its own, but a program never frees
text. A `စာသား` variable frees its text when its block ends, and when
a new value replaces the old one. A loop frees the text of each round before the next
round starts.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် i သည် ကိန်းပြည့် ၁ ဖြစ်ပါစေ။
i သည် ၄ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် လိုင်း သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "အဆင့် {i}: {i * i}" ဖြစ်ပါစေ။
လိုင်း ကို ပြသပါ။
i ကို i + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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
- Put a name into a second variable, change the second, and print both.
- Change the first character of `"မောင်"` to `'ခ'`, and count its characters first: which
  marks does the write keep?
- Build the text `"၁၂၃၄၅"` with a counted loop that adds `"{န}"` in each round.

Next: [Variables](../variables/README.md).
