---
title: Text
description: The text kind, string literals and their template slots, joining, cells and characters, changing a character, adding to text and removing its last character, then folding, ranges, text from u8s, copies, the drops the compiler adds, static text, and the errors.
---

# Text

Text in Zaka is a value of the kind `စာသား`: UTF-8 text that is copied as a number is. A
string literal is text, and a pair of braces in it is a *template slot* that holds an
expression. [Text](../../learn/text/README.md) in the Learn book shows the same rules in
small steps.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မောင်မောင်" ဖြစ်ပါစေ။
ကိန်းရှင် အသက် သည် ၂၅ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
"{အမည်} သည် အသက် {အသက်} နှစ် ဖြစ်သည်" ဟု ပြသမည်။
ကိန်းရှင် နှုတ်ဆက် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မင်္ဂလာပါ " + အမည် ဖြစ်ပါစေ။
နှုတ်ဆက် ကို ပြသမည်။
နှုတ်ဆက် ကို နှုတ်ဆက် + "!" ဟု ထားမည်။
နှုတ်ဆက် ကို ပြသမည်။
ကိန်းရှင် v သည် အမည် ၏ အကွက် ၀ မှစ၍ ၁၅ ကွက် ဖြစ်ပါစေ။
v ကို ပြသမည်။
v ၏ အကွက်အရေအတွက် ကို ပြသမည်။
"{{ မဟုတ် }}" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မောင်မောင် သည် အသက် ၂၅ နှစ် ဖြစ်သည်
မင်္ဂလာပါ မောင်မောင်
မင်္ဂလာပါ မောင်မောင်!
မောင်
၁၅
{ မဟုတ် }
```

## The kind

| Burmese | In messages | What it is |
| --- | --- | --- |
| `စာသား` | `text` | UTF-8 text |

- `စာသား` is three words: the address of its u8s, their count, and its *room*, the size of
  the heap block that holds them. A room of 0 means that the u8s belong to no block of
  this text: the u8s sit in static storage, or a parameter reads the caller's u8s. On a 64-bit
  platform the kind takes 24 octets.
- Text does not widen to a list of u8s, `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း` (`u8[]`): the
  cells of a list can be written, which would break the UTF-8 of the text. A plain list
  of u8s converts to text, with a check of its UTF-8; see [Text from u8s](#text-from-u8s).
- A `စာသား` declared with no value holds no text until the program sets it
  ([Set before read](statements.md#set-before-read)); its
  [initial value](kinds.md#initial-values), `ကနဦးတန်ဖိုး`, is blank text, with no u8s.
- `စာသား` can be the kind of a variable, a constant, a parameter, a result, a struct field
  and the cells of a list.

## String literals

A string literal is text in double quotes, and its kind is `စာသား`, whether or not it has
template slots. It can be stored, passed, returned, joined and printed. The escapes of
[Source text and words](words.md#characters-and-strings) work in it, and it must close on
the line where it opens (ZK0104).

```zaka
လုပ်ဆောင်ချက် ချီးမွမ်း သည်
အမည် ဟူသော စာသား တစ်ခု ကို လည်းကောင်း အမှတ်ပေါင်း ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"{အမည်} သည် {အမှတ်ပေါင်း} မှတ် ရသည်" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ချီးမွမ်း ဟူသော လုပ်ဆောင်ချက်ကို "မြမြ"၊ ၉၅ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
စာ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မြမြ သည် ၉၅ မှတ် ရသည်
```

## Template literals

A `{...}` in a string literal that is not an escape name is a template slot. It holds one
expression of each kind: a name, a sum, a field, a call.

- A template slot ends at the `}` that matches its `{` on the same line. String and character
  literals inside it are skipped whole, so a template slot can hold a template literal
  of its own, to each depth: `"a {"b {x}"} c"`. A template slot that does not close on its line
  is ZK0106.
- `{{` and `}}` write a brace. A lone `}`, and a `{` with no `}` after it on the line, are
  ZK0106.
- `{နောက်တစ်ကြောင်း}` and `{ယူနီကုဒ် HHHH}` remain escapes.
- The expression of a template slot is checked as each other expression is, and a mistake in it
  is reported at its place inside the quotes: `"{မသိ}"` is ZK0301, an undeclared name,
  and `"{}"` is ZK0202, an operand missing.
- In a character literal, `{...}` is not a template slot: `'{က}'` is ZK0106.

```zaka-error ZK0301
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"{မသိ}" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0301]: "မသိ" is not declared
```

```zaka-error ZK0202
"{}" ဟု ပြသမည်။
```

## The text of a value

A template slot writes its value as [`ပြသမည်`](statements.md#printing) prints it:

| Value | Written as |
| --- | --- |
| an integer | Burmese digits: `၂၅`, `-၅` |
| a decimal number | the shortest text that reads back: `၁.၅`, `inf` |
| a true/false value | `မှန်` or `မမှန်` |
| a character | itself |
| the null address | `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` |
| a list | its cells: `[၁၊ ၂၊ ၃]` |
| a struct | its name and fields: `အမှတ်(x: ၃၊ y: ၄)` |
| a shared object | as a struct; an object that is still being written, in a ring, as `…` |
| a function value | `နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက်` |
| text | itself, with no quotes |

Text inside a list or a struct is written in quotes, so that its edges show.

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု၊ အသက် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် လူ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် လူ(အမည်: "အောင်"၊ အသက်: ၃၀) ဖြစ်ပါစေ။
ကိန်းရှင် l သည် စာသား စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ["a ၂"၊ "b"] ဖြစ်ပါစေ။
"{ပ} {l}" ဟု ပြသမည်။
"{၁.၅} {မှန်} {'က'} {ပ ၏ အမည်}" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
လူ(အမည်: "အောင်"၊ အသက်: ၃၀) ["a ၂"၊ "b"]
၁.၅ မှန် က အောင်
```

## Joining

```text
<text> + <text>
<text> (၊ <text>)+ တို့ကို ဆက်ထားသည့် စာသား
```

`a + b` joins two pieces of text into new text. The
verbal form joins two or more pieces in order: `a၊ b၊ c တို့ကို ဆက်ထားသည့် စာသား` is the same
text as `a + b + c`, and folds alike. Other operators do not take text, and a join
takes nothing else with it: `t + ၁` and a number among the pieces of the verbal form are
ZK0403.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မမ" ဖြစ်ပါစေ။
"ဟေး "၊ အမည်၊ "!" တို့ကို ဆက်ထားသည့် စာသား ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဟေး မမ!
```

```zaka-error ZK0403
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် t သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "abc" ဖြစ်ပါစေ။
t + ၁ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0403]: + needs a number, found text
```

## Comparing

`a == b` and `a != b`, and the word forms `a သည် b နှင့် ညီသည်` and `မညီ`, compare two pieces
of text. Two texts are equal when the two hold the same u8s: the
compare is exact, with no normal form, so two spellings of one word that differ in their
code points are not equal. `<` and the other orderings do not take text (ZK0403).

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် i သည် ၇ ဖြစ်ပါစေ။
ကိန်းရှင် က သည် "{i}" ဖြစ်ပါစေ။
က == "၇" ကို ပြသမည်။
က != "၈" ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
မှန်
```

## Printing

`ပြသမည်` prints text as it is, with no quotes. A string literal, or a `+` join that holds
one, is quoted with `ဟု`; a text variable, a range or a call is a plain value and takes
`ကို`. The other choice still runs, with the warning ZK0505 or ZK0506 ([Natural
wording](naturalness.md)).

## Cells

A cell of text is one u8 of its UTF-8. A Latin letter takes one cell, a Burmese letter
three.

| Written | Means |
| --- | --- |
| `t ၏ အကွက်အရေအတွက်` | the count of u8s, a `လက္ခဏာမဲ့ ကိန်းပြည့်` |
| `t ၏ အကွက် n` | cell `n`, counted from 0, a `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်` |
| `t ၏ n ခုမြောက်အကွက်` | cell `n`, counted from 1 |
| `t ၏ အကွက် s မှစ၍ n ကွက်` | new text of `n` cells from cell `s`; see [Ranges](#ranges) |

- A cell number or a range outside the text stops the program when it runs.
- Text holds whole characters, as in Rust: a range must start and end at the edge of a
  character. When the text, the start and the length are each known, as for a text
  constant, a range that cuts a character is ZK0607; otherwise it stops the program when
  it runs.
- The cells of text cannot be written: the assignment is
  ZK0429, since one u8 alone can break the UTF-8 of the text. A whole character can be
  written: see [Changing text](#changing-text).

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "Zaka စံကား" ဖြစ်ပါစေ။
စာ ၏ အကွက်အရေအတွက် ကို ပြသမည်။
စာ ၏ အကွက် ၀ ကို ပြသမည်။
စာ ၏ ၂ ခုမြောက်အကွက် ကို ပြသမည်။
စာ ၏ အကွက် ၅ မှစ၍ ၁၅ ကွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၀
၉၀
၉၇
စံကား
```

```zaka-error ZK0429
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် t သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "abc" ဖြစ်ပါစေ။
t ၏ အကွက် ၀ ကို ၁ ဟု ထားမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0429]: the cells of text cannot be written
```

```zaka-error ZK0607
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ ကမ္ဘာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "ကမ္ဘာ" ဖြစ်ပါစေ။
ကမ္ဘာ ၏ အကွက် ၀ မှစ၍ ၂ ကွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0607]: 2 u8s from u8 0 start or end inside a character of the text
```

## Characters

```text
<text> ၏ စာလုံးအရေအတွက်
<text> ၏ စာလုံး <number>
<text> ၏ <number> ခုမြောက်စာလုံး
```

The characters of text are its Unicode scalar values, the values of `စာလုံး`. The count
`၏ စာလုံးအရေအတွက်` is a `လက္ခဏာမဲ့ ကိန်းပြည့်`; `၏ စာလုံး n` is the character at number `n`,
counted from 0, and `၏ n ခုမြောက်စာလုံး` the character at number `n` counted from 1, each a
`စာလုံး`, as `၏ အကွက် n` and `၏ n ခုမြောက်အကွက်` are for cells. Each takes text, and each walks the text from its start, since a character takes from one to four
u8s: a counted loop that reads each character through its number takes time in the
square of the count. An [each-character loop](statements.md#loops), `စာ ၏ စာလုံးတိုင်း ကို
စ အနေဖြင့် ယူ၍ ...`, walks the u8s once. A number outside the text stops the program
when it runs.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "ကမ္ဘာ ok" ဖြစ်ပါစေ။
စာ ၏ အကွက်အရေအတွက် ကို ပြသမည်။
စာ ၏ စာလုံးအရေအတွက် ကို ပြသမည်။
စာ ၏ စာလုံး ၁ ကို ပြသမည်။
စာ ၏ ၂ ခုမြောက်စာလုံး ကို ပြသမည်။
စာ ၏ စာလုံး ၆ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၈
၈
မ
မ
o
```

## Changing text

```text
<text> ၏ စာလုံး <number> ကို <character> ဟု ထားမည်။
<text> ၏ <number> ခုမြောက်စာလုံး ကို <character> ဟု ထားမည်။
<text> ထဲသို့ <value> ကို ထည့်မည်။
<text> ၏ စာလုံး <number> ၏ အရှေ့တွင် <value> ကို ထည့်မည်။
<text> ၏ <number> ခုမြောက်စာလုံး ၏ အရှေ့တွင် <value> ကို ထည့်မည်။
<text> ၏ နောက်ဆုံးစာလုံးကို ဖျက်မည်။
<text> ၏ အကွက် <start> မှ <end> ထိကို <value> ဖြင့် အစားထိုးမည်။
<text> ၏ <start> လုံးမြောက်စာလုံးမှ <end> လုံးမြောက်စာလုံးထိကို <value> ဖြင့် အစားထိုးမည်။
```

Text changes in place through four statements: the assignment writes one character, the
add statement adds at its end or in front of a character, the remove statement takes its
last character off, and the replace statement puts a value in place of a range.

- **A character write** takes `၏ စာလုံး n` (counted from 0) or `၏ n ခုမြောက်စာလုံး`
  (counted from 1) as its place. The text is a place that can be set, as for each
  assignment: a variable, a field, a cell of a list, or `ပ ၏ တန်ဖိုး` through a pointer;
  a constant is ZK0306. The number is an integer (ZK0423), and the value a `စာလုံး`
  (ZK0401). A number outside the text stops the program when it runs, with the trap that
  a read of a character outside the text gives.
- The write walks the text from its start, as a read of a character does. The new
  character can take more or fewer u8s than the old one: the u8s after it shift, and the
  count of u8s changes with them.
- **An add** takes text or a `စာလုံး`, and adds its u8s at the end. Each
  other value is ZK0401 ("expected text"). The value is read before the text changes, so
  `t ထဲသို့ t ကို ထည့်မည်။` doubles `t`.
- **An add in front of a character**, `t ၏ စာလုံး n ၏ အရှေ့တွင် v ကို ထည့်မည်။`, takes the
  same values and puts their u8s in front of character `n` (from 0, or from 1 for
  `n ခုမြောက်စာလုံး`). The number can also be the character count, which adds at the end;
  a larger one stops the program.
- **A remove**, `t ၏ နောက်ဆုံးစာလုံးကို ဖျက်မည်။`, takes the last character off; text with no
  characters stops the program: "the text holds no characters, so it has no last character
  to remove". `နောက်ဆုံးအကွက်` names the last cell of a list, and on text it is ZK0437;
  `နောက်ဆုံးစာလုံး` on a list is ZK0403.
- **A replace** puts a value in place of a range of the text, both ends included. With
  `အကွက် <start> မှ <end> ထိကို`, the ends count u8 cells from 0, as a range of cells
  does; an end that falls inside a character stops the program, as such a range does (see
  [Ranges](#ranges)). With `<start> လုံးမြောက်စာလုံးမှ <end> လုံးမြောက်စာလုံးထိကို`, the
  ends count characters from 1 (`ခုမြောက်` is a second spelling of `လုံးမြောက်`). The value
  takes the values of an add, and can be longer or shorter than the range, or blank. The
  value is read before the text changes, so it can be a range of the same text. The ends
  are integers (ZK0423); an end outside the text stops the program, and so does an end
  before the start: "the range from 2 to 1 ends before it starts". A range of a list is
  ZK0403, and a range on its own, outside a replace statement, is ZK0206.
- The cells (u8s) of text still cannot be written one at a time (ZK0429, see
  [Cells](#cells)); a whole character can, and a replace can write a range of cells.

A character is a Unicode code point, not a letter as it is written. `ကျွန်` is five characters,
`က`, `ျ`, `ွ`, `န`, `်`, so a write at number 0 replaces the `က` alone and keeps the marks
on it.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "abc" ဖြစ်ပါစေ။
စာ ၏ စာလုံး ၀ ကို 'A' ဟု ထားမည်။
စာ ၏ ၂ ခုမြောက်စာလုံး ကို 'က' ဟု ထားမည်။      // three u8s in place of one
စာ ထဲသို့ "!" ကို ထည့်မည်။
စာ ထဲသို့ '?' ကို ထည့်မည်။
စာ ကို ပြသမည်။
စာ ၏ အကွက်အရေအတွက် ကို ပြသမည်။
စာ ထဲသို့ စာ ကို ထည့်မည်။
စာ ကို ပြသမည်။
ကိန်းရှင် စ သည် "ကျွန်" ဖြစ်ပါစေ။
စ ၏ စာလုံး ၀ ကို 'ခ' ဟု ထားမည်။
စ ကို ပြသမည်။
စ ၏ စာလုံး ၀ ၏ အရှေ့တွင် "မ" ကို ထည့်မည်။
စ ၏ နောက်ဆုံးစာလုံးကို ဖျက်မည်။      // the last character is the mark ်
စ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
Aကc!?
၇
Aကc!?Aကc!?
ချွန်
မချွန
```

A replace of cells and a replace of characters:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် "hello world" ဖြစ်ပါစေ။
စာ ၏ အကွက် ၀ မှ ၄ ထိကို "bye" ဖြင့် အစားထိုးမည်။
စာ ကို ပြသမည်။
ကိန်းရှင် အမည် သည် "ကခဂ" ဖြစ်ပါစေ။
အမည် ၏ ၂ လုံးမြောက်စာလုံးမှ ၃ လုံးမြောက်စာလုံးထိကို "ငင" ဖြင့် အစားထိုးမည်။
အမည် ကို ပြသမည်။
အမည် ၏ ၁ လုံးမြောက်စာလုံးမှ ၁ လုံးမြောက်စာလုံးထိကို "" ဖြင့် အစားထိုးမည်။
အမည် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
bye world
ကငင
ငင
```

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် t သည် "abc" ဖြစ်ပါစေ။
t ၏ အကွက် ၂ မှ ၁ ထိကို "x" ဖြင့် အစားထိုးမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: the range from 2 to 1 ends before it starts
```

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် t သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "ab" ဖြစ်ပါစေ။
t ၏ ၃ ခုမြောက်စာလုံး ကို 'x' ဟု ထားမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: character 3 is outside the text, whose characters run from 1 to 2
```

```zaka-error ZK0437
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် t သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "ab" ဖြစ်ပါစေ။
t ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0437]: text cannot grow or shrink here: "ထည့်မည်" takes a plain list or text, "နောက်ဆုံးအကွက်ကို ဖျက်မည်" a plain list, and "နောက်ဆုံးစာလုံးကို ဖျက်မည်" text
```

When it runs, text with a room of 0 (static text, or a parameter that reads the caller's
u8s), and text whose block another text shares, first takes a heap block of its own with
the same u8s. A literal is never written, copied text shares nothing once it changes,
and a function that edits its parameter leaves the caller's text as it was (see
[Copies](#copies)). The new u8s go into the block when it has
room; else the text moves to a block twice its size, 16 u8s at the least. A run of adds
therefore copies the text a few times, not once for each add.

```zaka
လုပ်ဆောင်ချက် အော် သည် စ ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ထဲသို့ "!" ကို ထည့်မည်။
စ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည် သည် "မမ" ဖြစ်ပါစေ။
အော် ဟူသော လုပ်ဆောင်ချက်ကို အမည် တို့ဖြင့် လုပ်ဆောင်မည်။
အမည် ကို ပြသမည်။
ကိန်းရှင် l သည် ["ab"၊ "cd"] ဖြစ်ပါစေ။
l ၏ အကွက် ၁ ၏ စာလုံး ၀ ကို 'X' ဟု ထားမည်။
l ၏ အကွက် ၀ ထဲသို့ "!" ကို ထည့်မည်။
l ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မမ!
မမ
["ab!"၊ "Xd"]
```

## Folding

With expression folding on (the default; `--fold` holds `exprs`), a literal whose template slots
each fold to a constant becomes static text when the program is compiled, and it takes
no heap. A template slot folds when it holds a literal, a constant, or constant arithmetic on
them. A `+` of two pieces of constant text folds in the same manner.

A template slot with another value, such as a variable or a list, makes the literal a *format*:
the program builds its text on the heap when it runs.
Under `--fold none` each template literal is built when the program runs. The output
is the same in both cases.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ အကျယ် သည် ၈ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
"အကျယ် {အကျယ်}၊ ဧရိယာ {အကျယ် * အကျယ်}" ဟု ပြသမည်။       // static text
ကိန်းရှင် i သည် ၃ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
"အဆင့် {i}" ဟု ပြသမည်။                                // built when it runs
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အကျယ် ၈၊ ဧရိယာ ၆၄
အဆင့် ၃
```

A `ကိန်းသေ` of kind `စာသား` (or a list or struct that holds text) is static text in
each build, as a constant plain list is static data ([Lists](lists.md#static-data)): its
value must fold at compile time, whatever `--fold` holds. Its value can
use literals, other constants, template slots of these and `+`. A value that needs the program
to run, such as a variable in a template slot, a call, a cell or a field, is ZK0606, reported at
the first part that does not fold. `--fold` changes how the IR shows the reads of the
constant, never whether it is valid.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မမ" ဖြစ်ပါစေ။
ကိန်းသေ နှုတ်ဆက် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "ဟေး {အမည်}" + "!" ဖြစ်ပါစေ။
နှုတ်ဆက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဟေး မမ!
```

```zaka-error ZK0606
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် n သည် ၃ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
ကိန်းသေ အဆင့် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "အဆင့် {n}" ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0606]: a constant that holds text or a plain list must be known at compile time
```

## Ranges

```text
<text> ၏ အကွက် <start> မှစ၍ <length> ကွက်
```

A range of text is new text that holds `length` u8s from u8 `start`, copied. It stands
apart from the text it came from, as a [range of a list](lists.md#ranges) does, and a
later edit of either one leaves the other as it was.

- The cells count u8s from 0. The start and the length are integers (ZK0423).
- The source is a text place: a variable, a constant, a field, a cell or a parameter. A
  literal, a call result or another range is ZK0411.
- A range must start and end at the edge of a character, as text holds whole characters.
  When the text, the start and the length are each known, as for a text constant, a range
  that cuts a character is ZK0607; otherwise the program stops when it runs. A range past
  the end stops the program too.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် "Zaka စံကား" ဖြစ်ပါစေ။
ကိန်းရှင် ပိုင်း သည် စာ ၏ အကွက် ၅ မှစ၍ ၁၅ ကွက် ဖြစ်ပါစေ။
ပိုင်း ထဲသို့ "!" ကို ထည့်မည်။
စာ ထဲသို့ "?" ကို ထည့်မည်။
ပိုင်း ကို ပြသမည်။
စာ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
စံကား!
Zaka စံကား?
```

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကမ္ဘာ သည် "ကမ္ဘာ" ဖြစ်ပါစေ။
ကမ္ဘာ ၏ အကွက် ၀ မှစ၍ ၂ ကွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: a range of text at 0x1002 starts or ends inside a character
```

## Text from u8s

```text
<list> ကို စာသား တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်
```

A plain list of u8s, `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း`, converts to text. The VM checks
that the u8s are UTF-8 (the op `check_utf8`) and stops the program when the u8s are not; the
new text holds a copied set of the u8s, so a later write into the list does not reach it.
A counted list of u8s, or a list of another cell kind, does not convert (ZK0408), and
neither does text into a list.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် b သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၀၄၊ ၁၀၅] ဖြစ်ပါစေ။
b ကို စာသား တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
ကိန်းရှင် ဆ သည် b ကို စာသား တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
ဆ ထဲသို့ "!" ကို ထည့်မည်။
ဆ ကို ပြသမည်။
b ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
hi
hi!
[၁၀၄၊ ၁၀၅]
```

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် b သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၀၄၊ ၂၅၅] ဖြစ်ပါစေ။
"အစ" ဟု ပြသမည်။
b ကို စာသား တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အစ
the program stopped: the u8s at 0x1021 are not UTF-8 text
```

## Copies

`စာသား` is copied, as a number is. Setting one text place from another
(`ခ ကို က ဟု ထားမည်။`), declaring a variable from one, passing one to a `စာသား` parameter,
returning one, and putting one into a list or a struct field each give the new place text
of its own, and the old place keeps its text. A list or a struct with text in it is
copied alike.

The copied text costs nothing at first: the two places share one heap block, and the
block counts the texts that hold it. The first [edit](#changing-text) of either one gives
it a block of its own with the same u8s, and the other keeps the old block. Static text is
shared in the same manner, with no count: nothing writes it, so copied static text takes
no block, and its first edit gives it a heap block of its own.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မမ" ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အမည် ဖြစ်ပါစေ။
ခ ကို ပြသမည်။
အမည် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မမ
မမ
```

When the place read is not read again, the compiler moves the text instead, and the
count does not change: the new place takes the block, and the old one is left blank. The
move is never seen, since nothing reads the old place. It is not made for a parameter,
whose text can belong to the caller; for a global read inside a function; for a global
that a call of the same statement sets; for a place whose address unsafe code takes; or
for a place the same statement reads twice.

Text given to a function that is still read after the call is lent to the parameter with
a room of 0: the parameter reads the caller's u8s for the length of the call, and the
count does not change. The function sees a value of its own all the same. Setting the
parameter gives it new text, an edit of it first copies the caller's u8s into a block of
its own, and returning it, storing it or setting another place from it makes a copied text
there. The text is shared through the count at the call instead when the function, or a
call inside the same arguments, sets the global given, or when unsafe code takes the
address of the place.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "abc" ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် စာသား တစ်ခု ဖြစ်ပါစေ။
ခ ကို က ဟု ထားမည်။
ခ ကို ခ + "!" ဟု ထားမည်။
က ကို ပြသမည်။
ခ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
abc
abc!
```

## Drops

Text is *managed*: the compiler frees it, and a program has no statement for it. A place
whose kind holds text (a `စာသား` variable or parameter, a struct with a `စာသား` field, a
list of `စာသား`) drops its text. The drops come:

- at the end of its block;
- at the end of each round of a loop;
- before `ကွင်းဆက် ကို ရပ်မည်`, before `နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်` and before a return;
- before a new value replaces the old one;
- for a top-level variable, at the end of the program.

A short-lived text, such as a join, a range or a format that is printed or used in a
template slot, is dropped right after its use. A drop takes one off the count of the heap
block, and frees the block when the count reaches 0. It leaves alone static text (room 0)
and text that was moved out. `zakac run --leaks` lists none of these blocks. A pointer
beside the text in a struct or a list is not dropped: the block behind it, if it came from
a heap statement, is for unsafe code to give back ([Unsafe Zaka](unsafe.md)).

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် i သည် ၁ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
i သည် ၁၀ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လိုင်း သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "အဆင့် {i}: {i * i}" ဖြစ်ပါစေ။
အကယ်၍ i သည် ၄ နှင့် ညီလျှင် ကွင်းဆက် ကို ရပ်မည်။
လိုင်း ကို ပြသမည်။
i ကို i + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အဆင့် ၁: ၁
အဆင့် ၂: ၄
အဆင့် ၃: ၉
```

## Static text

The compiled module has one table of static texts, and each distinct text sits in it
once: string literals, folded formats and the texts that print instructions write all
share it. Before `main` starts, the VM puts each in a static block of its own, and the
block is frozen. Text made from a literal points into that block with a room of 0;
nothing is copied.

A write inside a frozen block stops the program. The compiler forbids a cell write
(ZK0429), and a character write or an add first moves text with a room of 0 to a block
of its own, so a pointer that unsafe code takes around the check is the one road to it:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် t သည် "abc" ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် p သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် t ၏ လိပ်စာ ကို လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် q သည် p ၏ တန်ဖိုး ဖြစ်ပါစေ။
q ၏ တန်ဖိုး ကို ၁၂၀ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: write at 0x1000 inside static data, which cannot change
```

[Instructions](../vm/instructions.md) lists the ops that build and drop text, and [When
a program stops](../vm/traps.md) the trap.

## The errors

| Code | Raised when |
| --- | --- |
| ZK0106 | a lone brace, an unclosed `{`, or `{...}` in a character literal |
| ZK0301 | a template slot names something not declared |
| ZK0401 | text and a list of u8s in place of each other, a character write with no character, or an add of a value that is no text and no character |
| ZK0403 | `+` of text and a non-text value, or an operator other than `+`, `==` and `!=` on text |
| ZK0408 | a conversion to text of a counted list of u8s or of another list, or of text to a list |
| ZK0411 | a range of a literal, a call result or another range |
| ZK0423 | a character write whose number is no integer |
| ZK0429 | a write to a cell of text |
| ZK0437 | `နောက်ဆုံးအကွက်` on text |
| ZK0439 | `၏ အရှေ့တွင်` after a value that is no character of text, and no cell of a list |
| ZK0606 | a constant that holds text, whose value needs the program to run |
| ZK0607 | a range of known text, start and length that cuts a character |
