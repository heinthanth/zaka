---
title: Text
description: Owned text and views of text - the two text kinds, string literals and their template slots, folding, joining, cells, moves, the drops the compiler adds, static text, and the errors ZK0403, ZK0429 and ZK0701.
---

# Text

Zaka has two kinds of text. `စာသား` owns its text; `စာသား တစ်စိတ်တစ်ပိုင်း` views text that
another place owns. A string literal is a value of the first kind, and a pair of braces
in it is a *template slot* that holds an expression. [Text](../../learn/values/strings.md) in the
Learn book shows the same rules in small steps.

```zaka
ကိန်းရှင် အမည် သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "မောင်မောင်" ဖြစ်ပါစေ။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ၂၅ ဖြစ်ပါစေ။
"{အမည်} သည် အသက် {အသက်} နှစ် ဖြစ်သည်" ဟု ပြသပါ။
ကိန်းရှင် နှုတ်ဆက် သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "မင်္ဂလာပါ " + အမည် ဖြစ်ပါစေ။
နှုတ်ဆက် ကို ပြသပါ။
နှုတ်ဆက် ကို နှုတ်ဆက် + "!" ဟု ထားပါ။
နှုတ်ဆက် ကို ပြသပါ။
ကိန်းရှင် v သည် စာသား တစ်စိတ်တစ်ပိုင်း ဖြစ်၍ တန်ဖိုးသည် အမည် ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
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
| `စာသား` | `text` | owned UTF-8 text |
| `စာသား တစ်စိတ်တစ်ပိုင်း` | `&text` | a view of text: a borrowed run of u8 cells that hold UTF-8 |

- `စာသား` is three words: the address of its u8s, their count, and its *room*, the size of
  the heap block that holds them. A room of 0 means that the u8s sit in static storage
  and belong to no block. On a 64-bit platform the kind takes 24 octets.
- `စာသား တစ်စိတ်တစ်ပိုင်း` is a [slice](slices.md): two words, the address and the count.
  `တစ်စိတ်တစ်ပိုင်း` straight after `စာသား` makes the view, with no `စာရင်း` between them.
- A view of text widens to `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း` (`&[]u8`) on
  its own. The reverse does not hold: a slice of u8s is not known to hold UTF-8, so
  giving one where a view of text is wanted is ZK0401.
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

ကိန်းရှင် စာ သည် စာသား ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် ချီးမွမ်း ကို "မြမြ"၊ ၉၅ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
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
ကိန်းရှင် ပ သည် လူ ဖြစ်၍ တန်ဖိုးသည် လူ(အမည်: "အောင်"၊ အသက်: ၃၀) ဖြစ်ပါစေ။
ကိန်းရှင် l သည် စာသား စာရင်း ဖြစ်၍ တန်ဖိုးသည် ["a ၂"၊ "b"] ဖြစ်ပါစေ။
"{ပ} {l}" ဟု ပြသပါ။
"{၁.၅} {မှန်} {'က'} {ပ ၏ အမည်}" ဟု ပြသပါ။
```

```text
လူ(အမည်: "အောင်"၊ အသက်: ၃၀) ["a ၂"၊ "b"]
၁.၅ မှန် က အောင်
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
ကိန်းသေ အမည် သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "မမ" ဖြစ်ပါစေ။
ကိန်းသေ နှုတ်ဆက် သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "ဟေး {အမည်}" + "!" ဖြစ်ပါစေ။
နှုတ်ဆက် ကို ပြသပါ။
```

```text
ဟေး မမ!
```

```zaka-error ZK0606
ကိန်းရှင် n သည် ကိန်းပြည့် ၃ ဖြစ်ပါစေ။
ကိန်းသေ အဆင့် သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "အဆင့် {n}" ဖြစ်ပါစေ။
```

```text
error[ZK0606]: a text constant must be known at compile time
```

## Joining

```text
<text> + <text>
<text> (၊ <text>)+ တို့ကို ဆက်ထားသည့် စာသား
```

`a + b` joins two pieces of text, owned or views in each mix, into new owned text. The
verbal form joins two or more pieces in order: `a၊ b၊ c တို့ကို ဆက်ထားသည့် စာသား` is the same
text as `a + b + c`, and folds the same way. Other operators do not take text, and a join
takes nothing else with it: `t + ၁`, `t == "a"` and a number among the pieces of the
verbal form are ZK0403. Text cannot be compared so far; compare the cells one at a time.

```zaka
ကိန်းရှင် အမည် သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "မမ" ဖြစ်ပါစေ။
"ဟေး "၊ အမည်၊ "!" တို့ကို ဆက်ထားသည့် စာသား ကို ပြသပါ။
```

```text
ဟေး မမ!
```

```zaka-error ZK0403
ကိန်းရှင် t သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "abc" ဖြစ်ပါစေ။
t + ၁ ကို ပြသပါ။
```

```text
error[ZK0403]: + needs a number, found text
```

## Printing

`ပြသပါ` prints text as it is, with no quotes. A string literal, or a `+` join that holds
one, is quoted with `ဟု`; a text variable, a view or a call is a plain value and takes
`ကို`. The other choice still runs, with the warning ZK0505 or ZK0506 ([Natural
wording](naturalness.md)).

## Views

```text
<text> ၏ တန်ဖိုးအငှား
<text> ၏ အကွက် <start> မှစ၍ <length> ကွက်
```

The lend `၏ တန်ဖိုးအငှား` views each cell of a text place; the range views `length` cells
from cell `start`. Both take owned text or a view. The lend needs a place, a constant
included: the lend of a literal or of a call result is ZK0411.

A view widens to a slice of u8s, `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း`, on its
own. The way back is a conversion, which checks the u8s: u8s that are not UTF-8 stop the
program when it runs.

```zaka
ကိန်းရှင် b သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် [၁၀၄၊ ၁၀၅] ဖြစ်ပါစေ။
(b ၏ တန်ဖိုးအငှား) ကို စာသား တစ်စိတ်တစ်ပိုင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
```

```text
hi
```

A view is a borrow, and the rules of [slices](slices.md#ownership) hold for it:

- a struct field cannot hold a view (ZK0718);
- a function cannot return a view of its own local text (ZK0706);
- a view used after its text is dropped, through a new value or the end of its block, is
  ZK0704.

```zaka-error ZK0706
လုပ်ဆောင်ချက် ယူ သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်စိတ်တစ်ပိုင်း ရလဒ် ကို ပြန်ပေးမည်။
ကိန်းရှင် t သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "abc" ဖြစ်ပါစေ။
t ၏ တန်ဖိုးအငှား ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
```

```zaka-error ZK0704
ကိန်းရှင် v သည် စာသား တစ်စိတ်တစ်ပိုင်း ဖြစ်ပါစေ။
အကယ်၍ မှန် ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် t သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "ab" + "{၁ + ၁}" ဖြစ်ပါစေ။
v ကို t ၏ တန်ဖိုးအငှား ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
v ကို ပြသပါ။
```

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
  ZK0429. Static text is shared, so a write to it would change each literal of the same
  text.

```zaka
ကိန်းရှင် စာ သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "Zaka စံကား" ဖြစ်ပါစေ။
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
ကိန်းရှင် t သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "abc" ဖြစ်ပါစေ။
t ၏ အကွက် ၀ ကို ၁ ဟု ထားပါ။
```

```text
error[ZK0429]: the cells of text cannot be written
```

```zaka-error ZK0607
ကိန်းသေ ကမ္ဘာ သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "ကမ္ဘာ" ဖြစ်ပါစေ။
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
u8s: a loop that reads each character in turn takes time in the square of the count. A
number outside the text stops the program when it runs.

```zaka
ကိန်းရှင် စာ သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "ကမ္ဘာ ok" ဖြစ်ပါစေ။
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

## Moves

`စာသား` is a move kind, as an [owner address](ownership.md) is. Setting one text place
from another (`ခ ကို က ဟု ထားပါ။`), declaring a variable from one, and passing one to a
`စာသား` parameter each move the text: the old place holds nothing after that, and reading
it is ZK0701. A function can return `စာသား`, and its caller owns the result. To keep
the text, lend a view instead.

A text constant is never moved: its text is static, so each read of it is a free
duplicate that shares the static block, and dropping a duplicate frees nothing. The
constant can be read again after that.

```zaka
ကိန်းသေ အမည် သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "မမ" ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် စာသား ဖြစ်၍ တန်ဖိုးသည် အမည် ဖြစ်ပါစေ။
ခ ကို ပြသပါ။
အမည် ကို ပြသပါ။
```

```text
မမ
မမ
```

```zaka-error ZK0701
ကိန်းရှင် က သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "abc" ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် စာသား ဖြစ်ပါစေ။
ခ ကို က ဟု ထားပါ။
က ကို ပြသပါ။
```

```text
error[ZK0701]: "က" was moved to another owner and now holds ဗလာ
```

## Drops

Text is *managed*: the compiler frees it, and a program has no statement for it. A place
whose kind holds text (a `စာသား` variable or parameter, a struct with a `စာသား` field, a
list of `စာသား`) drops its text:

- at the end of its block;
- at the end of each round of a loop;
- before `ကွင်းဆက် ကို ရပ်ပါ`, before `နောက်တစ်ကြိမ်ဆီသို့ ကျော်ပါ` and before a return;
- before a new value replaces the old one;
- for a top-level variable, at the end of the program.

A short-lived text, such as a join or a format that is printed or used in a template slot, is
dropped right after its use. A drop frees text that sits on the heap. It leaves alone
static text (room 0) and text that was moved out. The [ownership check](ownership.md)
reports no leak for these kinds, and `zakac run --leaks` lists none of their blocks.

```zaka
ကိန်းရှင် i သည် ကိန်းပြည့် ၁ ဖြစ်ပါစေ။
i သည် ၁၀ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် လိုင်း သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "အဆင့် {i}: {i * i}" ဖြစ်ပါစေ။
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

A write inside a frozen block stops the program. The compiler forbids it (ZK0429), so a
raw address that steps around the check is the one road to it:

```zaka-stop
ကိန်းရှင် t သည် စာသား ဖြစ်၍ တန်ဖိုးသည် "abc" ဖြစ်ပါစေ။
ကိန်းရှင် p သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာအကြမ်း ၏ လိပ်စာအကြမ်း ဖြစ်၍ တန်ဖိုးသည် t ၏ လိပ်စာ ကို လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာအကြမ်း ၏ လိပ်စာအကြမ်း အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် q သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာအကြမ်း ဖြစ်၍ တန်ဖိုးသည် p ၏ တန်ဖိုး ဖြစ်ပါစေ။
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
| ZK0401 | owned text where a view is wanted, or a slice of u8s where a view of text is wanted |
| ZK0403 | `+` of text and a non-text value, or another operator on text |
| ZK0411 | the lend of a literal or a call result |
| ZK0429 | a write to a cell of text |
| ZK0701 | a read of a text place after its text moved |
| ZK0704 | a view used after its text is dropped |
| ZK0706 | a function returns a view of its own local text |
| ZK0718 | a struct field holds a view |
