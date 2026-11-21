---
title: Ranges
description: Take a run of cells from a list or from text as a separate list or text.
---

# Ranges

A *range* (slice) is a run of cells that we take from a list or from text, counted from
0. A range of a list is a new list, and a range of text is new text, so we can store,
print and pass a range like each other list or text.

## You will learn

- How to take a range of a list.
- How a write to a range leaves the list unchanged.
- How to take a range of text, and where it must start and end.
- What happens when a range runs past the end.

## A range of a list

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် l သည် [၁၀၊ ၂၀၊ ၃၀၊ ၄၀၊ ၅၀] ဖြစ်ပါစေ။
l ၏ အကွက် ၁ မှစ၍ ၃ ကွက် ကို ပြသမည်။
l ၏ အကွက် ၀ မှစ၍ ၂ ကွက် ကို ပြသမည်။
l ၏ အကွက် ၅ မှစ၍ ၀ ကွက် ကို ပြသမည်။    // no cells
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၂၀၊ ၃၀၊ ၄၀]
[၁၀၊ ၂၀]
[]
```

To take a range, we write the list, `၏ အကွက်`, the first cell and `မှစ၍`, then the
number of cells and `ကွက်`:

```
<list> ၏ အကွက် <start> မှစ၍ <count> ကွက်
```

| Zaka | Reads as |
| --- | --- |
| `l ၏` | "of `l`" |
| `အကွက် ၁ မှစ၍` | "starting at cell 1" |
| `၃ ကွက်` | "three cells" |

The first line of the output has three cells from cell 1, which are cells 1, 2 and 3. A
count of `၀` gives a list with no cells, as the last line shows.

## A range is a duplicate

A range has duplicates (copies) of the cells, so it is separate from the list it came
from. A write to the range leaves the list as it was, and the list can grow while we use
the range:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် အလယ် သည် က ၏ အကွက် ၁ မှစ၍ ၂ ကွက် ဖြစ်ပါစေ။
က ထဲသို့ ၄ ကို ထည့်မည်။               // fine: the range is a separate list
အလယ် ၏ အကွက် ၀ ကို ၀ ဟု ထားမည်။
အလယ် ကို ပြသမည်။
က ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၀၊ ၃]
[၁၊ ၂၊ ၃၊ ၄]
```

We can use a range wherever a list of its data kind fits: in a variable, as an argument
or as a result. A function with a `ကိန်းပြည့် စာရင်း` parameter accepts a range the same as
a whole list.

## A range of text

```
<text> ၏ အကွက် <start> မှစ၍ <count> ကွက်
```

The cells of text are its u8s (bytes), so a Latin letter takes one cell and a Burmese
letter takes three, as [Cells](../text/characters.md#cells) shows. In `"Zaka စံကား"`, the
first four cells are `Zaka`, and the Burmese word starts at cell 5, after the space:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် "Zaka စံကား" ဖြစ်ပါစေ။
စာ ၏ အကွက် ၀ မှစ၍ ၄ ကွက် ကို ပြသမည်။
စာ ၏ အကွက် ၅ မှစ၍ ၁၅ ကွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
Zaka
စံကား
```

Text must contain whole characters at each point, so a range of text must start and end
between two characters. `ကမ္ဘာ` takes 15 cells, three for each of its five characters, so
six cells from cell 0 are its first two characters, while two cells would cut the first
character in half, and the program stops:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် "ကမ္ဘာ" ဖြစ်ပါစေ။
စာ ၏ အကွက် ၀ မှစ၍ ၆ ကွက် ကို ပြသမည်။
စာ ၏ အကွက် ၀ မှစ၍ ၂ ကွက် ကို ပြသမည်။      // ends inside the first character
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ကမ
the program stopped: a range of text at 0x1002 starts or ends inside a character
```

## Past the end

When a range runs past the end of its list, the program stops:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် l သည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
l ၏ အကွက် ၂ မှစ၍ ၂ ကွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: 2 cells from cell 2 pass the end of 3 cells
```

A [counted list](counted-lists.md) has a count that the compiler knows, so when the start
and the count of the range are fixed numbers, the compiler reports a range past the end
as an error before the program runs:

```zaka-error ZK0605
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် l သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
l ၏ အကွက် ၂ မှစ၍ ၂ ကွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0605]: 2 cells from cell 2 pass the end of a list of 3 cells
```

## Details

- **A range of a value.** We can take a range of a variable, a field, a cell or a
  parameter, and also of a literal, of a call result or of another range ([Ranges of
  lists](../../manual/language/lists.md#ranges)).
- **No parentheses needed.** The start and the count each run up to the word after them,
  so `x ၏ အကွက် i + ၁ မှစ၍ န - ၁ ကွက်` works as written. Both must be integers (ZK0423).
- **The cost.** A range duplicates its cells when it is made, so it costs time and
  storage in its length.
- **Text known in advance.** When the text, the start and the count are all known to the
  compiler, as for a text constant, a range that cuts a character is the error ZK0607
  ([Ranges of text](../../manual/language/text.md#ranges)).

## Recap

- `<list> ၏ အကွက် <start> မှစ၍ <count> ကွက်` takes a range, counted from cell 0.
- A range is a separate list or text, so a write to it does not change the source.
- A range of text counts u8s, and must start and end between two characters.
- A range past the end stops the program, or, on a counted list with fixed numbers, is an
  error before it runs.

## Hands on

- Write a function that receives a list and prints its largest cell, and call it with a
  whole list and with the last two cells of it.
- Print the first two characters of a Burmese word with a range, then take one cell less.
- Set each cell of a range to 0, then print the range and the list it came from.

Next: [Static data and counted lists](counted-lists.md).
