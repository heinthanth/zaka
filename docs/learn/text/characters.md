---
title: Cells and characters
description: Read the u8 cells and the characters of text, take its characters in turn with a loop, and change one character.
---

# Cells and characters

Text is a row of cells, and a row of characters on top of them. This page reads both, and
writes a whole character.

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

A character of text is a place, as a variable is: the assignment writes a new
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

## Hands on

- Count the cells of a Burmese word. Divide the count with three.
- Change the first character of `"မောင်"` to `'ခ'`, and count its characters first: which
  marks does the write keep?

Next: [Changing text](changing.md).
