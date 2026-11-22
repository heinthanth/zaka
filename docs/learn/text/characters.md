---
title: Cells and characters
description: Read the u8 cells and the characters of text, take its characters in turn with a loop, and change one character.
---

# Cells and characters

Text is stored as a row of small numbers, its *cells*, and the cells make up its
*characters* (Unicode scalar values). We can read both, take the characters one at a time
with a loop, and write a whole character in place of another.

## You will learn

- how text is stored, and how to read its cells
- how to count and read its characters
- how to take each character in turn with a loop
- how to change one character

## Cells

```text
<text> ၏ အကွက်အရေအတွက်
<text> ၏ အကွက် <number>
```

Text is stored as UTF-8, a row of numbers from 0 to 255. Each of them is a cell (byte) of
the data kind `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်` (`u8`). A Latin letter needs one cell, and a
Burmese letter needs three.

```zaka
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "Zaka စံကား" ဖြစ်ပါစေ။
စာ ၏ အကွက်အရေအတွက် ကို ပြသမည်။ // the count of cells
စာ ၏ အကွက် ၀ ကို ပြသမည်။ // the first cell: Z
```

```text
၂၀
၉၀
```

| Zaka | Reads as |
| --- | --- |
| `စာ ၏ အကွက်အရေအတွက်` | "the count of cells of `စာ`" |
| `စာ ၏ အကွက် ၀` | "cell 0 of `စာ`", counted from 0 |

The text has 20 cells: four for `Zaka`, one for the space, and three for each of the five
characters of `စံကား`. Cell 0 is `၉၀`, the code of `Z`.

We can read the cells of text, but we cannot write them, because a single cell can break a
Burmese letter in two. We can write a whole character instead, as [Changing a
character](#changing-a-character) shows.

```zaka-error ZK0429
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "abc" ဖြစ်ပါစေ။
စာ ၏ အကွက် ၀ ကို ၁ ဟု ထားမည်။
```

```text
error[ZK0429]: the cells of text cannot be written
```

## Characters

```text
<text> ၏ စာလုံးအရေအတွက်
<text> ၏ စာလုံး <number>
<text> ၏ <number> ခုမြောက်စာလုံး
```

To work with the characters of text rather than its cells, we use `စာလုံး` ("character")
in place of `အကွက်`. We write `၏ စာလုံးအရေအတွက်` to count the characters, `၏ စာလုံး <number>`
to read one counted from 0, and `၏ <number> ခုမြောက်စာလုံး` to read one counted from 1:

```zaka
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "ကမ္ဘာ" ဖြစ်ပါစေ။
စာ ၏ အကွက်အရေအတွက် ကို ပြသမည်။
စာ ၏ စာလုံးအရေအတွက် ကို ပြသမည်။
စာ ၏ စာလုံး ၀ ကို ပြသမည်။
စာ ၏ ၂ ခုမြောက်စာလုံး ကို ပြသမည်။
```

```text
၁၅
၅
က
မ
```

| Zaka | Reads as |
| --- | --- |
| `စာ ၏ စာလုံးအရေအတွက်` | "the count of characters of `စာ`" |
| `စာ ၏ စာလုံး ၀` | "character 0 of `စာ`" |
| `စာ ၏ ၂ ခုမြောက်စာလုံး` | "the 2nd character of `စာ`" |

The word `ကမ္ဘာ` has 5 characters in 15 cells, three cells for each character.

## Each character in turn

```text
<text> ၏ စာလုံးတိုင်း ကို <name> အနေဖြင့် ယူ၍ <sentence>
```

To visit the characters one after another, we use an *each-character loop* (for-each loop),
which means "taking each character of the text as `<name>`, do `<sentence>`". A list has the
same loop for its cells, as [Walking a list](../lists/README.md#walking-a-list) shows later.

```zaka
"ကမ္ဘာ" ၏ စာလုံးတိုင်း ကို စ အနေဖြင့် ယူ၍ စ ကို ပြသမည်။
```

```text
က
မ
္
ဘ
ာ
```

| Zaka | Reads as |
| --- | --- |
| `စာလုံးတိုင်း ကို` | "each character" |
| `စ အနေဖြင့် ယူ၍` | "taking it as `စ`" |

As the output shows, a Burmese letter and each mark on it are separate characters.

## Changing a character

```text
<text> ၏ စာလုံး <number> ကို <character> ဟု ထားမည်။
<text> ၏ <number> ခုမြောက်စာလုံး ကို <character> ဟု ထားမည်။
```

A character of a text variable is a place that we can assign to, as we assign to a variable.
We use `ထားမည်` to write a new character there, with `၏ စာလုံး` counting from 0 and
`၏ n ခုမြောက်စာလုံး` counting from 1:

```zaka
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "cat" ဖြစ်ပါစေ။
စာ ၏ စာလုံး ၀ ကို 'b' ဟု ထားမည်။
စာ ကို ပြသမည်။
စာ ၏ ၃ ခုမြောက်စာလုံး ကို 'd' ဟု ထားမည်။
စာ ကို ပြသမည်။
```

```text
bat
bad
```

The new value must be a character in single quotes. Text in double quotes is a mistake,
even when it contains a single character:

```zaka-error ZK0401
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "cat" ဖြစ်ပါစေ။
စာ ၏ စာလုံး ၀ ကို "b" ဟု ထားမည်။
```

```text
error[ZK0401]: expected char, found text
```

A character is one Unicode code point, not a letter as we see it on the screen. In speech,
`ကျွန်` is one sound, but it has five characters: `က`, `ျ`, `ွ`, `န` and `်`. So a write
replaces one of them and keeps the rest:

```zaka
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "ကျွန်" ဖြစ်ပါစေ။
စာ ၏ စာလုံးအရေအတွက် ကို ပြသမည်။
စာ ၏ စာလုံး ၀ ကို 'ခ' ဟု ထားမည်။
စာ ကို ပြသမည်။
```

```text
၅
ချွန်
```

A number past the last character stops the program, for a write as for a read:

```zaka-stop
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "ab" ဖြစ်ပါစေ။
စာ ၏ ၃ ခုမြောက်စာလုံး ကို 'x' ဟု ထားမည်။
```

```text
the program stopped: character 3 is outside the text, whose characters run from 1 to 2
```

## Details

- We can also read a cell counted from 1, with `၏ n ခုမြောက်အကွက်`, and a cell number
  outside the text stops the program ([Cells](../../manual/language/text.md#cells)).
- `၏ အကွက် <start> မှစ၍ <count> ကွက်` makes new text from a range of cells, which must start
  and end at the edge of a character ([Ranges](../../manual/language/text.md#ranges)).
- A read of a character through its number walks the text from its start, because a
  character can need from one to four cells. So a counted loop that reads each character
  through its number is slow on long text, while the each-character loop walks the text
  once ([Characters](../../manual/language/text.md#characters)).
- The new character can need more or fewer cells than the old one: the cells after it move,
  and the count of cells changes with them. A write to a text constant is ZK0306
  ([Changing text](../../manual/language/text.md#changing-text)).

## Recap

- Text is stored as UTF-8 cells, and we read them with `၏ အကွက်`, but we cannot write them.
- We count characters with `၏ စာလုံးအရေအတွက်`, and read one with `၏ စာလုံး <number>`
  (from 0) or `၏ <number> ခုမြောက်စာလုံး` (from 1).
- `၏ စာလုံးတိုင်း ကို <name> အနေဖြင့် ယူ၍` visits each character in turn.
- We use `ထားမည်` to write a whole character in place of another, and a Burmese mark is a
  character too.

## Hands on

- Count the cells of a Burmese word, and divide the count with three.
- Change the first character of `"မောင်"` to `'ခ'`, and count its characters first: which
  marks does the write keep?
- Print each character of a Burmese word, with a number before each one.

Next: [Changing text](changing.md).
