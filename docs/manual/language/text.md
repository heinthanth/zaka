---
title: Text
description: Owned text and views of text - the text kinds, string literals and their template slots, joining, cells and characters, changing a character and adding to text, then folding, the three forms of owned text, views, copies, the drops the compiler adds, static text, and the errors ZK0403, ZK0429 and ZK0438.
---

# Text

Zaka has two kinds of text. `စာသား` owns its text; `စာသား တစ်စိတ်တစ်ပိုင်း` views text that
another place owns. A string literal is a value of the first kind, and a pair of braces
in it is a *template slot* that holds an expression. [Text](../../learn/values/strings.md) in the
Learn book shows the same rules in small steps.

```zaka
ကိန်းရှင် အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မောင်မောင်" ဖြစ်ပါစေ။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ၂၅ ဖြစ်ပါစေ။
"{အမည်} သည် အသက် {အသက်} နှစ် ဖြစ်သည်" ဟု ပြသပါ။
ကိန်းရှင် နှုတ်ဆက် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မင်္ဂလာပါ " + အမည် ဖြစ်ပါစေ။
နှုတ်ဆက် ကို ပြသပါ။
နှုတ်ဆက် ကို နှုတ်ဆက် + "!" ဟု ထားပါ။
နှုတ်ဆက် ကို ပြသပါ။
ကိန်းရှင် v သည် စာသား တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အမည် ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
v ကို ပြသပါ။
v ၏ အကွက်အရေအတွက် ကို ပြသပါ။
"{{ မဟုတ် }}" ဟု ပြသပါ။
```

```text
မောင်မောင် သည် အသက် ၂၅ နှစ် ဖြစ်သည်
မင်္ဂလာပါ မောင်မောင်
မင်္ဂလာပါ မောင်မောင်!
မောင်မောင်
၃၀
{ မဟုတ် }
```

## The kinds

| Burmese | In messages | What it is |
| --- | --- | --- |
| `စာသား` | `text` | owned UTF-8 text, the plain form |
| `စာသားသေ` | `textf` | owned UTF-8 text whose u8s never change (see [Forms](#forms)) |
| `စာသားရှင်` | `text~` | owned UTF-8 text in a heap block of its own from the start (see [Forms](#forms)) |
| `စာသား တစ်စိတ်တစ်ပိုင်း` | `text[..]` | a view of text: a borrowed run of u8 cells that hold UTF-8 |

- `စာသား` is three words: the address of its u8s, their count, and its *room*, the size of
  the heap block that holds them. A room of 0 means that the u8s sit in static storage
  and belong to no block. On a 64-bit platform the kind takes 24 octets.
- `စာသား တစ်စိတ်တစ်ပိုင်း` is a [slice](slices.md): two words, the address and the count.
  `တစ်စိတ်တစ်ပိုင်း` straight after `စာသား` makes the view, with no `စာရင်း` between them.
- A view of text does not widen to `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း`
  (`u8[][..]`): the cells of such a slice can be written, which would break the UTF-8 of the
  text. The reverse does not hold either: a slice of u8s is not known to hold UTF-8.
  Giving one where the other is wanted is ZK0401.
- Owned text does not turn into a view on its own either: lend it (see
  [Views](#views)). Giving a `စာသား` where a view is wanted is ZK0401.
- A `စာသား` declared with no value holds blank text, with no u8s. A view declared with no value views
  no cells.
- `စာသား` can be the kind of a variable, a constant, a parameter, a result, a struct field
  and the cells of a list. A view lives in variables and parameters alone, as each slice
  does.

## String literals

A string literal is text in double quotes, and its kind is `စာသား`, whether or not it has
template slots. It can be stored, passed, returned, joined and printed. The escapes of
[Source text and words](words.md#characters-and-strings) work in it, and it must close on
the line where it opens (ZK0104).

```zaka
လုပ်ဆောင်ချက် ချီးမွမ်း သည်
စာသား အမည်၊ ကိန်းပြည့် အမှတ်ပေါင်း တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား ရလဒ် ကို ပြန်ပေးမည်။
"{အမည်} သည် {အမှတ်ပေါင်း} မှတ် ရသည်" ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။

ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် ချီးမွမ်း ကို "မြမြ"၊ ၉၅ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
စာ ကို ပြသပါ။
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
"{မသိ}" ဟု ပြသပါ။
```

```text
error[ZK0301]: "မသိ" is not declared
```

```zaka-error ZK0202
"{}" ဟု ပြသပါ။
```

## The text of a value

A template slot writes its value as [`ပြသပါ`](statements.md#printing) prints it:

| Value | Written as |
| --- | --- |
| an integer | Burmese digits: `၂၅`, `-၅` |
| a decimal number | the shortest text that reads back: `၁.၅`, `inf` |
| a true/false value | `မှန်` or `မမှန်` |
| a character | itself |
| the null address | `ဗလာ` |
| a list or a slice | its cells: `[၁၊ ၂၊ ၃]` |
| a struct | its name and fields: `အမှတ်(x: ၃၊ y: ၄)` |
| text | itself, with no quotes |

Text inside a list or a struct is written in quotes, so that its edges show.

```zaka
ဒေတာအမျိုးအစား လူ တွင် စာသား အမည်၊ ကိန်းပြည့် အသက် တို့ ပါဝင်သည်။
ကိန်းရှင် ပ သည် လူ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် လူ(အမည်: "အောင်"၊ အသက်: ၃၀) ဖြစ်ပါစေ။
ကိန်းရှင် l သည် စာသား စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ["a ၂"၊ "b"] ဖြစ်ပါစေ။
"{ပ} {l}" ဟု ပြသပါ။
"{၁.၅} {မှန်} {'က'} {ပ ၏ အမည်}" ဟု ပြသပါ။
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

`a + b` joins two pieces of text, owned or views in each mix, into new owned text. The
verbal form joins two or more pieces in order: `a၊ b၊ c တို့ကို ဆက်ထားသည့် စာသား` is the same
text as `a + b + c`, and folds the same way. Other operators do not take text, and a join
takes nothing else with it: `t + ၁` and a number among the pieces of the verbal form are
ZK0403.

```zaka
ကိန်းရှင် အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မမ" ဖြစ်ပါစေ။
"ဟေး "၊ အမည်၊ "!" တို့ကို ဆက်ထားသည့် စာသား ကို ပြသပါ။
```

```text
ဟေး မမ!
```

```zaka-error ZK0403
ကိန်းရှင် t သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "abc" ဖြစ်ပါစေ။
t + ၁ ကို ပြသပါ။
```

```text
error[ZK0403]: + needs a number, found text
```

## Comparing

`a == b` and `a != b`, and the word forms `a သည် b နှင့် ညီသည်` and `မညီ`, compare two pieces
of text, owned or views in each mix. Two texts are equal when the two hold the same u8s: the
compare is exact, with no normal form, so two spellings of one word that differ in their
code points are not equal. `<` and the other orderings do not take text (ZK0403).

```zaka
ကိန်းရှင် i သည် ၇ ဖြစ်ပါစေ။
ကိန်းရှင် က သည် "{i}" ဖြစ်ပါစေ။
က == "၇" ကို ပြသပါ။
က != "၈" ကို ပြသပါ။
```

```text
မှန်
မှန်
```

## Printing

`ပြသပါ` prints text as it is, with no quotes. A string literal, or a `+` join that holds
one, is quoted with `ဟု`; a text variable, a view or a call is a plain value and takes
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
| `t ၏ အကွက် s မှစ၍ n ကွက်` | a view of `n` cells from cell `s` |
| `t ၏ တန်ဖိုးအငှား` | a view of each cell |

- These work on owned text and on views alike.
- A cell number or a range outside the text stops the program when it runs.
- Text holds whole characters, as in Rust: a range must start and end at the edge of a
  character. When the text, the start and the length are each known, as for a text
  constant, a range that cuts a character is ZK0607; otherwise it stops the program when
  it runs.
- The cells of text cannot be written, through owned text or a view: the assignment is
  ZK0429, since one u8 alone can break the UTF-8 of the text. A whole character can be
  written: see [Changing text](#changing-text).

```zaka
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "Zaka စံကား" ဖြစ်ပါစေ။
စာ ၏ အကွက်အရေအတွက် ကို ပြသပါ။
စာ ၏ အကွက် ၀ ကို ပြသပါ။
စာ ၏ ၂ ခုမြောက်အကွက် ကို ပြသပါ။
စာ ၏ အကွက် ၅ မှစ၍ ၁၅ ကွက် ကို ပြသပါ။
```

```text
၂၀
၉၀
၉၇
စံကား
```

```zaka-error ZK0429
ကိန်းရှင် t သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "abc" ဖြစ်ပါစေ။
t ၏ အကွက် ၀ ကို ၁ ဟု ထားပါ။
```

```text
error[ZK0429]: the cells of text cannot be written
```

```zaka-error ZK0607
ကိန်းသေ ကမ္ဘာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "ကမ္ဘာ" ဖြစ်ပါစေ။
ကမ္ဘာ ၏ အကွက် ၀ မှစ၍ ၂ ကွက် ကို ပြသပါ။
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
`စာလုံး`, as `၏ အကွက် n` and `၏ n ခုမြောက်အကွက်` are for cells. Each takes owned text or a
view, and each walks the text from its start, since a character takes from one to four
u8s: a counted loop that reads each character through its number takes time in the
square of the count. An [each-character loop](statements.md#loops), `စာ ၏ စာလုံးတိုင်း ကို
စ အနေဖြင့် ယူ၍ ...`, walks the u8s once. A number outside the text stops the program
when it runs.

```zaka
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "ကမ္ဘာ ok" ဖြစ်ပါစေ။
စာ ၏ အကွက်အရေအတွက် ကို ပြသပါ။
စာ ၏ စာလုံးအရေအတွက် ကို ပြသပါ။
စာ ၏ စာလုံး ၁ ကို ပြသပါ။
စာ ၏ ၂ ခုမြောက်စာလုံး ကို ပြသပါ။
စာ ၏ စာလုံး ၆ ကို ပြသပါ။
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
<text> ၏ စာလုံး <number> ကို <character> ဟု ထားပါ။
<text> ၏ <number> ခုမြောက်စာလုံး ကို <character> ဟု ထားပါ။
<text> ထဲသို့ <value> ကို ထည့်ပါ။
```

Text changes in place through two statements: the assignment writes one character, and
the add statement adds to its end.

- **A character write** takes `၏ စာလုံး n` (counted from 0) or `၏ n ခုမြောက်စာလုံး`
  (counted from 1) as its place. The text is a place that can be set, as for each
  assignment: a variable, a field, a cell of a list, or `ပ ၏ တန်ဖိုး` through a pointer;
  a constant is ZK0306. The number is an integer (ZK0423), and the value a `စာလုံး`
  (ZK0401). A number outside the text stops the program when it runs, with the trap that
  a read of a character outside the text gives.
- The write walks the text from its start, as a read of a character does. The new
  character can take more or fewer u8s than the old one: the u8s after it shift, and the
  count of u8s changes with them.
- **An add** takes text, a view of text, or a `စာလုံး`, and adds its u8s at the end. Each
  other value is ZK0401 ("expected text"). The value is read before the text changes, so
  `t ထဲသို့ t ကို ထည့်ပါ။` doubles `t`.
- Text takes no remove: `t ၏ နောက်ဆုံးအကွက်ကို ဖျက်ပါ။` is ZK0437, and so is a remove from
  each other form of text.
- The cells (u8s) of text still cannot be written (ZK0429, see [Cells](#cells)); a whole
  character can.
- Plain text (`စာသား`) and heap text (`စာသားရှင်`) change. Fixed text (`စာသားသေ`) and a
  view of text do not: a character write or an add on either is ZK0438 (see
  [Forms](#forms) and [Views](#views)). Each edit ends the views of the text (ZK0704).

A character is a Unicode code point, not a letter as it is written. `ကျွန်` is five characters,
`က`, `ျ`, `ွ`, `န`, `်`, so a write at number 0 replaces the `က` alone and keeps the marks
on it.

```zaka
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "abc" ဖြစ်ပါစေ။
စာ ၏ စာလုံး ၀ ကို 'A' ဟု ထားပါ။
စာ ၏ ၂ ခုမြောက်စာလုံး ကို 'က' ဟု ထားပါ။      // three u8s in place of one
စာ ထဲသို့ "!" ကို ထည့်ပါ။
စာ ထဲသို့ '?' ကို ထည့်ပါ။
စာ ကို ပြသပါ။
စာ ၏ အကွက်အရေအတွက် ကို ပြသပါ။
စာ ထဲသို့ စာ ကို ထည့်ပါ။
စာ ကို ပြသပါ။
ကိန်းရှင် စ သည် "ကျွန်" ဖြစ်ပါစေ။
စ ၏ စာလုံး ၀ ကို 'ခ' ဟု ထားပါ။
စ ကို ပြသပါ။
```

```text
Aကc!?
၇
Aကc!?Aကc!?
ချွန်
```

```zaka-stop
ကိန်းရှင် t သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "ab" ဖြစ်ပါစေ။
t ၏ ၃ ခုမြောက်စာလုံး ကို 'x' ဟု ထားပါ။
```

```text
the program stopped: character 3 is outside the text, whose characters run from 1 to 2
```

```zaka-error ZK0437
ကိန်းရှင် t သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "ab" ဖြစ်ပါစေ။
t ၏ နောက်ဆုံးအကွက်ကို ဖျက်ပါ။
```

```text
error[ZK0437]: text cannot grow or shrink here: "ထည့်ပါ" takes a plain or a heap list or text, and "ဖျက်ပါ" a plain or a heap list
```

When it runs, text with a room of 0 (static text, or text lent to a parameter) first
takes a heap block of its own with the same u8s. A literal is never written, a duplicate of it
shares nothing once it changes, and a function that edits its parameter leaves the
caller's text as it was (see [Copies](#copies)). The new u8s go into the block when it has
room; else the text moves to a block twice its size, 16 u8s at the least. A run of adds
therefore copies the text a few times, not once for each add.

```zaka
လုပ်ဆောင်ချက် အော် သည် စာသား စ ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ထဲသို့ "!" ကို ထည့်ပါ။
စ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။

ကိန်းရှင် အမည် သည် "မမ" ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် အော် ကို အမည် တို့ဖြင့် လုပ်ဆောင်ပါ။
အမည် ကို ပြသပါ။
ကိန်းရှင် l သည် ["ab"၊ "cd"] ဖြစ်ပါစေ။
l ၏ အကွက် ၁ ၏ စာလုံး ၀ ကို 'X' ဟု ထားပါ။
l ၏ အကွက် ၀ ထဲသို့ "!" ကို ထည့်ပါ။
l ကို ပြသပါ။
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
ကိန်းသေ အကျယ် သည် ကိန်းပြည့် ၈ ဖြစ်ပါစေ။
"အကျယ် {အကျယ်}၊ ဧရိယာ {အကျယ် * အကျယ်}" ဟု ပြသပါ။       // static text
ကိန်းရှင် i သည် ကိန်းပြည့် ၃ ဖြစ်ပါစေ။
"အဆင့် {i}" ဟု ပြသပါ။                                // built when it runs
```

```text
အကျယ် ၈၊ ဧရိယာ ၆၄
အဆင့် ၃
```

A `ကိန်းသေ` of kind `စာသား` (or a list or struct that holds text) is static text in
each build: its value must fold at compile time, whatever `--fold` holds. Its value can
use literals, other constants, template slots of these and `+`. A value that needs the program
to run, such as a variable in a template slot, a call, a cell or a field, is ZK0606, reported at
the first part that does not fold. `--fold` changes how the IR shows the reads of the
constant, never whether it is valid.

```zaka
ကိန်းသေ အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မမ" ဖြစ်ပါစေ။
ကိန်းသေ နှုတ်ဆက် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "ဟေး {အမည်}" + "!" ဖြစ်ပါစေ။
နှုတ်ဆက် ကို ပြသပါ။
```

```text
ဟေး မမ!
```

```zaka-error ZK0606
ကိန်းရှင် n သည် ကိန်းပြည့် ၃ ဖြစ်ပါစေ။
ကိန်းသေ အဆင့် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "အဆင့် {n}" ဖြစ်ပါစေ။
```

```text
error[ZK0606]: a text constant must be known at compile time
```

## Forms

Owned text comes in three forms. Each is the same three words, and each converts to the
others on its own where a value goes into a place: a declaration, a set, an argument, a
result, a struct field or a list cell.

| Form | Burmese | Its u8s |
| --- | --- | --- |
| plain | `စာသား` | in static storage for a literal or a folded value (room 0), else in a heap block it owns |
| fixed | `စာသားသေ` | in static storage in each case (room 0): each value is known at compile time (ZK0608), and its u8s never change (ZK0438) |
| heap | `စာသားရှင်` | in a heap block it owns, from the start |

- **Heap text** owns its block. Where a value of another form becomes heap text, u8s in
  static storage are duplicated into a new heap block, so a literal given to heap text
  allocates. Blank heap text holds no u8s and no block. A parameter of heap text takes a
  duplicate of its argument; the caller's text is never lent to it.
- **Fixed text** is static text, as `&'static str` is in Rust. Its value is known at
  compile time, whatever `--fold` holds: a literal, a constant, a template literal whose
  slots [fold](#folding), a join of these, or fixed text read from another place. A value
  that needs the program to run, such as plain text in a variable, a template slot with a
  variable in it, or a call that returns plain text, is ZK0608, at the first part that
  does not fold. Fixed text owns no block, so nothing frees it.
- Fixed text has no length in its kind: a set gives a variable of it other static text
  of each length, and a parameter of it takes static text of each length.
- Fixed text never changes its u8s: a [character write or an add](#changing-text) on
  it is ZK0438. Plain and heap text take both edits.
- A constant holds plain or fixed text, never heap text (ZK0435): a constant sits in
  static storage.
- A lend of text of each form gives a view, `text[..]`. Joins, formats and template literals
  make plain text, which converts to the other forms.

```zaka
ကိန်းရှင် က သည် စာသားသေ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မင်္ဂလာပါ" ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် စာသားရှင် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ခ + " မောင်မောင်" ဖြစ်ပါစေ။
"{က} | {ခ} | {ဂ}" ဟု ပြသပါ။
```

```text
မင်္ဂလာပါ | မင်္ဂလာပါ | မင်္ဂလာပါ မောင်မောင်
```

```zaka-error ZK0435
ကိန်းသေ က သည် စာသားရှင် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "abc" ဖြစ်ပါစေ။
```

```zaka
လုပ်ဆောင်ချက် ပြ သည် စာသားသေ စ ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။

ကိန်းသေ ဗားရှင်း သည် ကိန်းပြည့် ၃ ဖြစ်ပါစေ။
ကိန်းရှင် f သည် စာသားသေ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "Zaka {ဗားရှင်း}" ဖြစ်ပါစေ။   // folds: static text
လုပ်ဆောင်ချက် ပြ ကို f တို့ဖြင့် လုပ်ဆောင်ပါ။
f ကို "ab" ဟု ထားပါ။                                                       // other static text, another length
လုပ်ဆောင်ချက် ပြ ကို f တို့ဖြင့် လုပ်ဆောင်ပါ။
```

```text
Zaka ၃
ab
```

```zaka-error ZK0608
ကိန်းရှင် i သည် ကိန်းပြည့် ၃ ဖြစ်ပါစေ။
ကိန်းရှင် f သည် စာသားသေ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "အဆင့် {i}" ဖြစ်ပါစေ။
```

```text
error[ZK0608]: fixed text ("စာသားသေ") must be known at compile time
```

```zaka-error ZK0438
ကိန်းရှင် f သည် စာသားသေ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "ab" ဖြစ်ပါစေ။
f ထဲသို့ "c" ကို ထည့်ပါ။
```

```text
error[ZK0438]: textf cannot change: a character write or an add takes text (စာသား) or heap text (စာသားရှင်)
```

## Views

```text
<text> ၏ တန်ဖိုးအငှား
<text> ၏ အကွက် <start> မှစ၍ <length> ကွက်
```

The lend `၏ တန်ဖိုးအငှား` views each cell of a text place; the range views `length` cells
from cell `start`. Both take owned text or a view. The lend needs a place, a constant
included: the lend of a literal or of a call result is ZK0411.

A view never widens to a slice of u8s, `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း`: a
write through the slice would break the text. A conversion turns u8s into text, and it
checks them: u8s that are not UTF-8 stop the program when it runs.

```zaka
ကိန်းရှင် b သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၀၄၊ ၁၀၅] ဖြစ်ပါစေ။
(b ၏ တန်ဖိုးအငှား) ကို စာသား တစ်စိတ်တစ်ပိုင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
```

```text
hi
```

A view is a borrow, and the rules of [slices](slices.md#ownership) hold for it:

- a struct field cannot hold a view (ZK0718);
- a function cannot return a view of its own local text (ZK0706);
- a view used after its text is dropped, through a new value or the end of its block, is
  ZK0704;
- a view used after its text is edited, through a [character write or an
  add](#changing-text), is ZK0704 too, since the edit can move the u8s to a new block. A
  view read as the added value of the edit itself is fine: `t ထဲသို့ v ကို ထည့်ပါ။` reads `v`
  first.

A view reads text that another place owns, so it cannot change: a character write or an
add on a view is ZK0438. Make text of its own from the view first: a template literal
`"{v}"` or a join builds new `စာသား` from it.

```zaka-error ZK0438
ကိန်းရှင် t သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "ab" ဖြစ်ပါစေ။
ကိန်းရှင် v သည် စာသား တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် t ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
v ၏ စာလုံး ၀ ကို 'x' ဟု ထားပါ။
```

```zaka-error ZK0704
ကိန်းရှင် t သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "ab" ဖြစ်ပါစေ။
ကိန်းရှင် v သည် စာသား တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် t ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
t ထဲသို့ "z" ကို ထည့်ပါ။
v ကို ပြသပါ။
```

```text
error[ZK0704]: the block of "t" is edited, so its cells can move, here, but "v" still borrows it and is used later
```

```zaka-error ZK0706
လုပ်ဆောင်ချက် ယူ သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်စိတ်တစ်ပိုင်း ရလဒ် ကို ပြန်ပေးမည်။
ကိန်းရှင် t သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "abc" ဖြစ်ပါစေ။
t ၏ တန်ဖိုးအငှား ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
```

```zaka-error ZK0704
ကိန်းရှင် v သည် စာသား တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်ပါစေ။
အကယ်၍ မှန် ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် t သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "ab" + "{၁ + ၁}" ဖြစ်ပါစေ။
v ကို t ၏ တန်ဖိုးအငှား ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
v ကို ပြသပါ။
```

## Copies

`စာသား` is copied, as a number is. Setting one text place from another
(`ခ ကို က ဟု ထားပါ။`), declaring a variable from one, passing one to a `စာသား` parameter,
returning one, and putting one into a list or a struct field each duplicate the text: the new
place gets a heap block of its own with the same u8s, and the old place keeps its text.
A list or a struct with text in it is copied alike, each text in it with a block
of its own. A list or a struct that holds an [owner address](ownership.md) still moves,
since a duplicate would free one block twice.

Static text is shared: nothing writes it, so a duplicate of a literal or a constant takes no
block, and dropping the duplicate frees nothing. The first [edit](#changing-text) of such a
duplicate gives it a heap block of its own, and the static text remains as it was.

```zaka
ကိန်းသေ အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မမ" ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အမည် ဖြစ်ပါစေ။
ခ ကို ပြသပါ။
အမည် ကို ပြသပါ။
```

```text
မမ
မမ
```

A duplicate of heap text takes a new block, and each duplicate is dropped on its own. A
[view](#views) reads text with no duplicate.

When the place read is not read again, the compiler moves the text instead, and nothing
is copied: the new place takes the block, and the old one is left blank. The move is
never seen, since nothing reads the old place. It is not made for a parameter, whose text
can belong to the caller; for a global read inside a function; for a global that a call of
the same statement sets; for a place that a view or an address reads; or for a place the
same statement reads twice.

Text given to a function that is still read after the call is lent to the parameter, not
copied: the parameter reads the caller's u8s for the length of the call, and nothing is
freed when the function ends. The function sees a value of its own all the same. Setting
the parameter gives it new text, an edit of it (a character write or an add) first copies
the caller's u8s into a block of its own, and returning it, storing it or setting another place
from it makes a duplicate there. The text is copied at the call instead when the function,
or a call inside the same arguments, sets the global given, or when a view or an address
of the place is taken. `--warn-duplicates` names each statement where a duplicate remains
(ZK0720).

```zaka
ကိန်းရှင် က သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "abc" ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် စာသား တစ်ခု ဖြစ်ပါစေ။
ခ ကို က ဟု ထားပါ။
ခ ကို ခ + "!" ဟု ထားပါ။
က ကို ပြသပါ။
ခ ကို ပြသပါ။
```

```text
abc
abc!
```

## Drops

Text is *managed*: the compiler frees it, and a program has no statement for it. A place
whose kind holds plain or heap text (a `စာသား` variable or parameter, a struct with a
`စာသား` field, a list of `စာသား`) drops its text. Fixed text is static, so a place of it
has nothing to drop. The drops come:

- at the end of its block;
- at the end of each round of a loop;
- before `ကွင်းဆက် ကို ရပ်ပါ`, before `နောက်တစ်ကြိမ်ဆီသို့ ကျော်ပါ` and before a return;
- before a new value replaces the old one;
- for a top-level variable, at the end of the program.

A short-lived text, such as a join or a format that is printed or used in a template slot, is
dropped right after its use. A drop frees text that sits on the heap. It leaves alone
static text (room 0) and text that was moved out. The [ownership check](ownership.md)
reports no leak for the text, and `zakac run --leaks` lists none of its blocks. An owner
pointer beside the text in a struct or a list is still the program's to free, and the check
reports it as for each other struct (ZK0708).

```zaka
ကိန်းရှင် i သည် ကိန်းပြည့် ၁ ဖြစ်ပါစေ။
i သည် ၁၀ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် လိုင်း သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "အဆင့် {i}: {i * i}" ဖြစ်ပါစေ။
အကယ်၍ i သည် ၄ နှင့် ညီလျှင် ကွင်းဆက် ကို ရပ်ပါ။
လိုင်း ကို ပြသပါ။
i ကို i + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
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
block is frozen. Owned text made from a literal points into that block with a room of 0;
nothing is duplicated.

A write inside a frozen block stops the program. The compiler forbids a cell write
(ZK0429), and a character write or an add first moves text with a room of 0 to a block
of its own, so a raw address that steps around the check is the one road to it:

```zaka-stop
ကိန်းရှင် t သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "abc" ဖြစ်ပါစေ။
ကိန်းရှင် p သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာအကြမ်း ၏ လိပ်စာအကြမ်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် t ၏ လိပ်စာ ကို လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာအကြမ်း ၏ လိပ်စာအကြမ်း အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် q သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာအကြမ်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် p ၏ တန်ဖိုး ဖြစ်ပါစေ။
q ၏ တန်ဖိုး ကို ၁၂၀ ဟု ထားပါ။
```

```text
the program stopped: write at 0x1070 inside static text, which cannot change
```

[Instructions](../vm/instructions.md) lists the ops that build and drop text, and [When
a program stops](../vm/traps.md) the trap.

## The errors

| Code | Raised when |
| --- | --- |
| ZK0106 | a lone brace, an unclosed `{`, or `{...}` in a character literal |
| ZK0301 | a template slot names something not declared |
| ZK0401 | owned text where a view is wanted, a view of text and a slice of u8s in place of each other, a character write with no character, or an add of a value that is no text and no character |
| ZK0403 | `+` of text and a non-text value, or an operator other than `+`, `==` and `!=` on text |
| ZK0411 | the lend of a literal or a call result |
| ZK0423 | a character write whose number is no integer |
| ZK0429 | a write to a cell of text |
| ZK0435 | a constant that holds heap text or a heap list |
| ZK0437 | a remove from text |
| ZK0438 | a character write or an add on fixed text or a view of text |
| ZK0608 | fixed text given a value that needs the program to run |
| ZK0704 | a view used after its text is dropped or edited |
| ZK0706 | a function returns a view of its own local text |
| ZK0718 | a struct field holds a view |
