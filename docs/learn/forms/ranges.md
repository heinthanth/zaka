---
title: Ranges
description: Take a run of cells from a list or from text as a list or text of its own.
---

# Ranges

`x ၏ အကွက် <start> မှစ၍ <length> ကွက်` ("`length` cells of `x`, starting at cell
`start`") is a *range*: a run of cells, counted from 0. A range of a list is a list of its
own, and a range of text is text of its own:

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

| Zaka | Reads as |
| --- | --- |
| `အကွက် ၁ မှစ၍` | "starting at cell 1" |
| `၃ ကွက်` | "three cells" |

## A range is a duplicate

A range holds copies of the cells, so a write to it leaves the list as it was, and the
list can grow while the range is in use:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် အလယ် သည် က ၏ အကွက် ၁ မှစ၍ ၂ ကွက် ဖြစ်ပါစေ။
က ထဲသို့ ၄ ကို ထည့်မည်။               // fine: the range is a list of its own
အလယ် ၏ အကွက် ၀ ကို ၀ ဟု ထားမည်။
အလယ် ကို ပြသမည်။
က ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၀၊ ၃]
[၁၊ ၂၊ ၃၊ ၄]
```

A range goes wherever a list of its kind goes: into a variable, a function, a result. A
function that takes `ကိန်းပြည့် စာရင်း` takes a range as it takes a whole list.

## Part of text

The cells of text are its u8s: a Latin letter takes one cell and a Burmese letter three,
as [Cells](../text/characters.md#cells) shows.

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

A range of text must start and end between two characters, so that text holds whole
characters at each step. `ကမ္ဘာ` takes 15 cells, three for each of its five characters,
so six cells from cell 0 are its first two characters, and two cells cut one in half:

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

A range past the end of its list stops the program:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် l သည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
l ၏ အကွက် ၂ မှစ၍ ၂ ကွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: 2 cells from cell 2 pass the end of 3 cells
```

On a [counted list](counted-lists.md), whose count the compiler knows, a range of
constant numbers past the end is an error before the program runs:

```zaka-error ZK0605
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် l သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
l ၏ အကွက် ၂ မှစ၍ ၂ ကွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## Hands on

- Write a function that takes a list and prints its largest cell, and call it with a
  whole list and with the last two cells of it.
- Print the first two characters of a Burmese word with a range, then take one cell less.
- Set each cell of a range to 0, then print the range and the list it came from.

Next: [Static data and counted lists](counted-lists.md).
