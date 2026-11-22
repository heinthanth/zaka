---
title: Text
description: The text data kind, string literals and their template slots, the text of a value, joining, comparing and printing text, cells and characters, ranges, changing text, text from u8s, and the errors.
---

# Text

Text (string) in Zaka is a value of the data kind (type) `စာသား`: UTF-8 text that is
duplicated (copied) on assignment, as a number is. A string literal is text, and a pair of
braces in it is a *template slot* with an expression inside. This page covers how we
write, read and change text; [How text is stored](text-storage.md) covers its heap blocks,
duplicates, drops and static text. The Learn book teaches the same rules in small steps
in [Text](../../learn/text/README.md).

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

## The data kind

| Burmese | In messages | What it is |
| --- | --- | --- |
| `စာသား` | `text` | UTF-8 text |

- We can use `စာသား` as the data kind of a variable, a constant, a parameter, a result,
  a struct field and the cells of a list.
- A `စာသား` declared with no value has no text until the program sets it ([Set before
  read](declarations.md#set-before-read)). Its [initial value](kinds.md#initial-values)
  (default value), `ကနဦးတန်ဖိုး`, is blank text, with no octets.
- Text does not widen to a list of u8s, `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း` (`u8[]`),
  because we can write the cells of a list, and that would break the UTF-8 of the text.
  A plain list of u8s converts to text, with a check of its UTF-8; see [Text from
  u8s](#text-from-u8s).
- `စာသား` is three words: the address of its octets (bytes), their count, and its room,
  the size of the heap block that contains them. On a 64-bit platform the data kind needs
  24 octets. [How text is stored](text-storage.md#the-three-words) states the rest.

## String literals

```text
"<text>"
```

A string literal is text in double quotes, and its data kind is `စာသား` whether or not it
has template slots. We can store it, pass it, return it, join it and print it. The
escapes of [Source text and words](words.md#characters-and-strings) work in it, and it
must close on the line where it opens (ZK0104).

```zaka
ချီးမွမ်း ဟူသော လုပ်ဆောင်ချက် သည်
အမည် ဟူသော စာသား တစ်ခု ကို လည်းကောင်း
အမှတ်ပေါင်း ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"{အမည်} သည် {အမှတ်ပေါင်း} မှတ် ရသည်" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည်
ချီးမွမ်း ဟူသော လုပ်ဆောင်ချက်ကို "မြမြ" နှင့် ၉၅ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
စာ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မြမြ သည် ၉၅ မှတ် ရသည်
```

## Template literals

```text
"… {<expression>} …"
```

A `{...}` in a string literal that is not an escape name is a template slot (string
interpolation). We can put one expression of each data kind in it: a name, a sum, a
field or a call.

| Case | What happens | Code |
| --- | --- | --- |
| a template slot | ends at the `}` that matches its `{` on the same line; string and character literals inside it are skipped whole, so a slot can contain another template literal, to each depth: `"a {"b {x}"} c"` | |
| a template slot that does not close on its line | an error | ZK0106 |
| `{{` and `}}` | write a brace | |
| a lone `}`, or a `{` with no `}` after it on the line | an error | ZK0106 |
| `{နောက်တစ်ကြောင်း}` and `{ယူနီကုဒ် HHHH}` | remain escapes | |
| `{...}` in a character literal, as in `'{က}'` | not a template slot: an error | ZK0106 |

The compiler checks the expression in a template slot as it checks each other expression,
and reports a mistake in it at its place inside the quotes: `"{မသိ}"` is ZK0301, a name
that is not declared, and `"{}"` is ZK0202, an operand missing.

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

A template slot writes its value in the same form that
[`ပြသမည်`](statements.md#printing) prints it:

| Value | Written as |
| --- | --- |
| an integer | Burmese digits: `၂၅`, `-၅` |
| a decimal number | the shortest text that reads back as the same number: `၁.၅`, `inf` |
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
လူ ဟူသော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု နှင့် အသက် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

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
<text>၊ <text> နှင့် <text> တို့ကို ဆက်ထားသည့် စာသား
```

We use `a + b` to join two pieces of text into new text. To join two or more pieces in
order, we can also use the word form: `a၊ b နှင့် c တို့ကို ဆက်ထားသည့် စာသား` is the same
text as `a + b + c`, and it folds in the same manner. Other operators do not work on text,
and a join accepts text alone: `t + ၁`, or a number among the pieces of the word form, is
ZK0403.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မမ" ဖြစ်ပါစေ။
"ဟေး "၊ အမည် နှင့် "!" တို့ကို ဆက်ထားသည့် စာသား ကို ပြသမည်။
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

```text
<text> == <text>
<text> != <text>
<text> သည် <text> နှင့် ညီသည်
<text> သည် <text> နှင့် မညီ
```

We can compare two pieces of text with `==` and `!=`, or with the word forms `ညီ` and
`မညီ`. Two texts are equal when the two have the same octets. The comparison is precise,
with no normal form, so two spellings of one word that differ in their code points are
not equal. `<` and the other orderings do not work on text (ZK0403).

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

```text
"<text>" ဟု ပြသမည်။
<value> ကို ပြသမည်။
```

`ပြသမည်` prints text as it is, with no quotes. We quote a string literal, or a `+` join
with a literal in it, with `ဟု`; a text variable, a range or a call is a plain value, so
we use `ကို` with it. The other choice still runs, with the warning ZK0505 or ZK0506
([Natural wording](naturalness.md)).

## Cells

```text
<text> ၏ အကွက် <number>
<text> ၏ <number> ခုမြောက်အကွက်
<text> ၏ အကွက်အရေအတွက်
```

A cell of text is one octet of its UTF-8. A Latin letter needs one cell, and a Burmese
letter three.

| Written | Means |
| --- | --- |
| `t ၏ အကွက်အရေအတွက်` | the count of octets, a `လက္ခဏာမဲ့ ကိန်းပြည့်` |
| `t ၏ အကွက် n` | cell `n`, counted from 0, a `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်` |
| `t ၏ n ခုမြောက်အကွက်` | cell `n`, counted from 1 |
| `t ၏ အကွက် s မှစ၍ n ကွက်` | new text of `n` cells from cell `s`; see [Ranges](#ranges) |

- A cell number or a range outside the text stops the program when it runs.
- Text contains whole characters, as in Rust, so a range must start and end at the edge
  of a character. When the text, the start and the length are all known, as for a text
  constant, a range that cuts a character is ZK0607; otherwise it stops the program when
  it runs.
- We cannot write the cells of text: the assignment is ZK0429, since one octet alone can
  break the UTF-8 of the text. We can write a whole character instead: see [Changing
  text](#changing-text).

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

The characters of text are its Unicode scalar values, the values of the character
(`char`) data kind `စာလုံး`. `၏ စာလုံးအရေအတွက်` is the count of characters, a
`လက္ခဏာမဲ့ ကိန်းပြည့်`. `၏ စာလုံး n` is the character at number `n` counted from 0, and
`၏ n ခုမြောက်စာလုံး` the character at number `n` counted from 1, each a `စာလုံး`, in the
same manner as `၏ အကွက် n` and `၏ n ခုမြောက်အကွက်` for cells. A number outside the text
stops the program when it runs.

A character needs from one to four octets, so each of these forms walks the text from its
start. A counted loop that reads each character through its number therefore takes time
in the square of the count, while an [each-character loop](control-flow.md#each-cell-loops),
`စာ ၏ စာလုံးတိုင်း ကို စ အနေဖြင့် ယူ၍ ...`, walks the octets once.

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

A character is a Unicode code point, not a letter as it is written on the page. `ကျွန်`
is five characters, `က`, `ျ`, `ွ`, `န` and `်`, so a write at number 0 replaces the `က`
alone and keeps the marks on it.

## Ranges

```text
<text> ၏ အကွက် <start> မှစ၍ <length> ကွက်
```

A range (substring) of text is new text with `<length>` octets from octet `<start>`,
duplicated. It is separate from the text it came from, as a [range of a
list](lists.md#ranges) is, so a later edit of either one leaves the other unchanged.

- The cells count octets from 0. The start and the length must be integers (ZK0423).
- The source can be each text value: a variable, a constant, a field, a cell, a
  parameter, a literal, a call result or another range.
- A range must start and end at the edge of a character, because text contains whole
  characters. When the text, the start and the length are all known, as for a text
  constant, a range that cuts a character is ZK0607; otherwise the program stops when it
  runs. A range past the end stops the program too.

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

We can change text in place with four statements: an assignment writes one character, an
add puts new text at the end or in front of a character, a remove takes the last
character off, and a replace puts a value in place of a range. In each of them, the text
must be a place we can set, as for each assignment: a variable, a field, a cell of a
list, or `ပ ၏ တန်ဖိုး` through a pointer. A constant is ZK0306.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "abc" ဖြစ်ပါစေ။
စာ ၏ စာလုံး ၀ ကို 'A' ဟု ထားမည်။
စာ ၏ ၂ ခုမြောက်စာလုံး ကို 'က' ဟု ထားမည်။ // three octets in place of one
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
စ ၏ နောက်ဆုံးစာလုံးကို ဖျက်မည်။ // the last character is the mark ်
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

### Writing a character

To write one character, we use `၏ စာလုံး n` (counted from 0) or `၏ n ခုမြောက်စာလုံး`
(counted from 1) as the place. The number must be an integer (ZK0423), and the value a
`စာလုံး` (ZK0401). A number outside the text stops the program when it runs, with the
same trap as a read of a character outside the text. The write walks the text from its
start, as a read of a character does. The new character can need more or fewer octets
than the old one, so the octets after it shift, and the count of octets changes with
them.

### Adding

An add accepts text or a `စာလုံး` and puts its octets at the end. Each other value is
ZK0401 ("expected text"). The value is read before the text changes, so
`t ထဲသို့ t ကို ထည့်မည်။` doubles `t`.

To add in front of a character, we write `t ၏ စာလုံး n ၏ အရှေ့တွင် v ကို ထည့်မည်။`. It
accepts the same values and puts their octets in front of character `n`, counted from 0,
or from 1 for `n ခုမြောက်စာလုံး`. The number can also equal the count of characters,
which adds at the end; a larger number stops the program. In front of a cell of text,
`t ၏ အကွက် n ၏ အရှေ့တွင်`, the add is ZK0437, because a cell is one octet.

### Removing the last character

`t ၏ နောက်ဆုံးစာလုံးကို ဖျက်မည်။` takes the last character off. On text with no
characters it stops the program: "the text holds no characters, so it has no last
character to remove". `နောက်ဆုံးအကွက်` names the last cell of a list and is ZK0437 on
text, while `နောက်ဆုံးစာလုံး` on a list is ZK0403.

### Replacing a range

A replace puts a value in place of a range of the text, both ends included. There are
two forms:

| Form | The ends count | When an end is wrong |
| --- | --- | --- |
| `အကွက် <start> မှ <end> ထိကို` | octet cells from 0, as a range of cells does | an end inside a character stops the program, as such a range does ([Ranges](#ranges)) |
| `<start> လုံးမြောက်စာလုံးမှ <end> လုံးမြောက်စာလုံးထိကို` | characters from 1; `ခုမြောက်` is a second spelling of `လုံးမြောက်` | |

- The value accepts the same values as an add, and can be longer or shorter than the
  range, or blank.
- The value is read before the text changes, so it can be a range of the same text.
- The ends must be integers (ZK0423). An end outside the text stops the program, and so
  does an end before the start: "the range from 2 to 1 ends before it starts".
- A replace on a list is ZK0403, and a range in this form outside a replace
  statement is ZK0206.

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

### Edits and duplicates

An edit never writes a literal, and it never reaches a duplicate: text that shares its
octets with other text, or with the caller through a parameter, first gets its own heap
block. So a function that edits its parameter leaves the text of the caller as it was.
[Edits and the heap block](text-storage.md#edits-and-the-heap-block) states how the block
grows.

```zaka
အော် ဟူသော လုပ်ဆောင်ချက် သည် စ ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ထဲသို့ "!" ကို ထည့်မည်။
စ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည် သည် "မမ" ဖြစ်ပါစေ။
အော် ဟူသော လုပ်ဆောင်ချက်ကို အမည် ဖြင့် လုပ်ဆောင်မည်။
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

## Text from u8s

```text
<list> ကို စာသား တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်
```

A plain list of u8s, `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း`, converts to text. The VM checks
that the octets are UTF-8 (the operation `check_utf8`) and stops the program when the
octets are not. The new text gets a duplicate of the octets, so a later write into the list does
not reach it. A counted list of u8s, or a list of another cell kind, does not convert
(ZK0408), and neither does text into a list.

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

## Errors at a glance

| Code | Raised when |
| --- | --- |
| ZK0104 | a string literal that does not close on its line |
| ZK0106 | a lone brace, an unclosed `{`, or `{...}` in a character literal |
| ZK0202 | a template slot with no expression, `"{}"` |
| ZK0206 | a range form of a replace outside a replace statement |
| ZK0301 | a template slot names something that is not declared |
| ZK0306 | a change to the text of a constant |
| ZK0401 | text and a list of u8s in place of each other, a character write with no character, or an add of a value that is not text or a character |
| ZK0403 | `+` of text and a value that is not text, an operator other than `+`, `==` and `!=` on text, `နောက်ဆုံးစာလုံး` on a list, or a replace on a list |
| ZK0408 | a conversion to text of a counted list of u8s or of another list, or of text to a list |
| ZK0423 | a character number, a start, a length or an end that is not an integer |
| ZK0429 | a write to a cell of text |
| ZK0437 | `နောက်ဆုံးအကွက်` on text, or an add in front of a cell of text |
| ZK0439 | `၏ အရှေ့တွင်` after a value that is not a character of text or a cell of a list |
| ZK0505 | a warning: text printed with `ကို` instead of `ဟု` |
| ZK0506 | a warning: a plain value printed with `ဟု` instead of `ကို` |
| ZK0606 | a constant with text in it, whose value needs the program to run ([Folding](text-storage.md#folding)) |
| ZK0607 | a range of known text, start and length that cuts a character |
