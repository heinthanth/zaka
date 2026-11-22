---
title: Zaka for programmers
description: The whole language on one page, for developers who know Python, Rust or C.
---

# Zaka for programmers

Zaka uses Burmese word order. A sentence (statement) puts its verb last and ends with
`။`, the Burmese full stop. Particles (postpositions) follow parts of a sentence and
mark their roles, such as subject or object. Most operators have both a Burmese word
form and an equivalent sign form, such as `+`.

This page covers the language one feature at a time, for developers who know another
language. Zaka has its own names for some ideas, so the usual name follows in parentheses
the first time, as in data kind (type), and the [list of terms](../manual/reference/terms.md)
has all of them. To run an example, paste it into the [Explorer](/explorer/), or save it as
`x.zaka` and run `zakac run --repl x.zaka`. To test statements one at a time, run `zakac
repl`.

## Sentence shape

```text
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
<value> ကို ပြသမည်။
"<text>" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```zaka
// The program start block: where the program begins
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ ကမ္ဘာကြီး" ဟု ပြသမည်။ // quoted text takes ဟု
၆ * ၇ ကို ပြသမည်။ // a value takes ကို
၁၀ သည် ၃ ထက် ကြီးသည် ဟု ပြသမည်။ // "10 is greater than 3", a sentence
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ ကမ္ဘာကြီး
၄၂
မှန်
```

| Zaka | In other languages |
| --- | --- |
| `ပရိုဂရမ်သည် … ပရိုဂရမ် ပြီးဆုံးပါပြီး။` | `fn main() { … }`, `int main(void) { … }` |
| `၆ * ၇ ကို ပြသမည်။` | `print(6 * 7)` |
| `။` | `;` |
| `၁၀ သည် ၃ ထက် ကြီးသည်` | `10 > 3` |

Each program has one program start block (entry point, `main`), which begins with
`ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။` and ends with `ပရိုဂရမ် ပြီးဆုံးပါပြီး။`, and the
statements between these two lines run in order. A statement ends at `။`, not at the end of
a line, so we can split a long statement over several lines. We use `သည်` after the subject
and `ကို` after the object. To print, we write `ကို` after a plain value and `ဟု` after
quoted text or a full sentence such as `၁၀ သည် ၃ ထက် ကြီးသည်`. Numbers print in Burmese
digits.

The examples below leave the program start block out: in [REPL
mode](../manual/language/functions-start-block.md#repl-mode), which `zakac run --repl`,
`zakac repl` and the Explorer use, the statements at the top level form the block.

Learn: [Hello, world](hello/README.md), [Zaka sentences](sentences/README.md).
Docs: [Statements](../manual/language/statements.md), [Source text and words](../manual/language/words.md).

## Variables and data kinds

```text
ကိန်းရှင် <name> သည် <kind> တစ်ခု ဖြစ်၍ တန်ဖိုးသည် <value> ဖြစ်ပါစေ။
ကိန်းရှင် <name> သည် <value> ဖြစ်ပါစေ။
ကိန်းသေ <name> သည် <value> ဖြစ်ပါစေ။
<name> ကို <value> ဟု ထားမည်။
```

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။ // the data kind written out
ကိန်းရှင် အမည် သည် "မမ" ဖြစ်ပါစေ။ // the data kind comes from the value
ကိန်းသေ တစ်ပတ် သည် ၇ ဖြစ်ပါစေ။ // a constant
ကိန်းရှင် ပျမ်းမျှ သည် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။ // 0.0
အသက် ကို အသက် + ၁ ဟု ထားမည်။ // assignment
"{အမည်} {အသက်} {တစ်ပတ်} {ပျမ်းမျှ}" ဟု ပြသမည်။
```

```text
မမ ၁၉ ၇ ၀.၀
```

| Zaka | In other languages |
| --- | --- |
| `ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။` | `let mut age: isize = 18;` |
| `ကိန်းသေ တစ်ပတ် သည် ၇ ဖြစ်ပါစေ။` | `const WEEK: isize = 7;` |
| `အသက် ကို အသက် + ၁ ဟု ထားမည်။` | `age = age + 1` |
| `ကိန်းပြည့်`, `ဒဿမကိန်း`, `မှန်မှားတန်ဖိုး`, `စာလုံး`, `စာသား` | `isize`, `f64`, `bool`, `char`, `String` |
| `ကနဦးတန်ဖိုး` | `Default::default()`: `0`, `0.0`, `false`, `""` |

We declare a variable (`let mut`) with `ကိန်းရှင်` and a constant (`const`) with `ကိန်းသေ`,
and to change a variable later we write `<name> ကို <value> ဟု ထားမည်။`. The data kinds
(types) in the table are the integer, the decimal number (float), the true/false value
(boolean), the character and the text (string). When a value is given, the data kind can be
left out, because the compiler works it out from the value.

After the data kind we write `တစ်ခု` ("one"), which is a measure word (classifier): Burmese
counts things with these words, so the compiler reports ZK0234 when it is missing. We can
use `ကနဦးတန်ဖိုး` to set a variable to the initial value (default value) of its data kind,
such as `0` or `""`. A variable keeps its data kind for its whole life, and it must be set
before it is read: when some path can reach a read before the variable is set, the compiler
reports ZK0722, where C would read garbage.

Learn: [Variables](variables/README.md), [Scalar values](values/README.md).
Docs: [Data kinds and values](../manual/language/kinds.md), [Declarations and assignment](../manual/language/declarations.md#declarations).

## Operators

```text
<a> + <b>                    <a> နှင့် <b> ၏ ပေါင်းလဒ်
<a> > <b>                    <a> သည် <b> ထက် ကြီးသည်
<a> == <b>                   <a> သည် <b> နှင့် ညီသည်
<value> ကို <kind> တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်
```

```zaka
ကိန်းရှင် က သည် ၁၇ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ၅ ဖြစ်ပါစေ။
က + ခ ကို ပြသမည်။ // sign form
က နှင့် ခ ၏ ပေါင်းလဒ် ကို ပြသမည်။ // word form: the sum of k and kh
က / ခ ကို ပြသမည်။ // integer division
က သည် ခ ထက် ကြီးသည် ဟု ပြသမည်။ // k > kh
(က > ခ ဖြစ်၍ ခ သည် ၀ နှင့် ညီသည် မဟုတ်) ကို ပြသမည်။ // and, not
က ကို ဒဿမကိန်း တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။ // a conversion
```

```text
၂၂
၂၂
၃
မှန်
မှန်
၁၇.၀
```

| Zaka | In other languages |
| --- | --- |
| `က နှင့် ခ ၏ ပေါင်းလဒ်` | `k + kh` |
| `က သည် ခ ထက် ကြီးသည်` | `k > kh` |
| `ဖြစ်၍`, `သို့မဟုတ်`, `မဟုတ်` | `and`, `or`, `not` |
| `က ကို ဒဿမကိန်း တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်` | `k as f64`, `float(k)`, `(double)k` |

The signs `&&`, `||` and `!` work too, and we can mix sign forms and word forms in one
program. Integer `/` drops the remainder, as it does in C and Rust. Variables of different
data kinds do not mix, so adding an integer variable to a decimal one is ZK0404. To combine
them, we first convert one side with `<value> ကို <kind> တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်`,
which Zaka calls a conversion (cast).

Learn: [Operators](operators/README.md), [Converting between data kinds](operators/conversion.md).
Docs: [Expressions](../manual/language/expressions.md), [Conversions](../manual/language/conversions.md).

## Conditions and loops

```text
အကယ်၍ <a> သည် <b> ထက် ကြီးလျှင် <block>
ထိုသို့မဟုတ်ဘဲ <condition>လျှင် <block>
သို့မဟုတ်ပါက <block>
<from> မှ <to> အထိ <name> အနေဖြင့် ရေတွက်၍ <block>
<a> သည် <b> အောက် ငယ်နေသရွေ့ <block>

<block> := <one statement>။
         | အောက်ပါတို့ကို လုပ်ဆောင်မည်။ … လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```zaka
ကိန်းရှင် အမှတ် သည် ၇၃ ဖြစ်ပါစေ။
အကယ်၍ အမှတ် သည် ၈၀ အောက် မငယ်လျှင် "က" ဟု ပြသမည်။ // if score >= 80
ထိုသို့မဟုတ်ဘဲ အမှတ် သည် ၆၀ အောက် မငယ်လျှင် "ခ" ဟု ပြသမည်။ // else if
သို့မဟုတ်ပါက "ဂ" ဟု ပြသမည်။ // else
၁ မှ ၃ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသမည်။ // 1, 2, 3
ကိန်းရှင် န သည် ၁ ဖြစ်ပါစေ။
န သည် ၁၀၀ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။ // while n < 100
န ကို န * ၃ ဟု ထားမည်။
အကယ်၍ န သည် ၅၀ ထက် ကြီးလျှင် ကွင်းဆက် ကို ရပ်မည်။ // break
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
န ကို ပြသမည်။
```

```text
ခ
၁
၂
၃
၈၁
```

| Zaka | In other languages |
| --- | --- |
| `အကယ်၍ … လျှင်`, `ထိုသို့မဟုတ်ဘဲ … လျှင်`, `သို့မဟုတ်ပါက` | `if`, `elif`, `else` |
| `၁ မှ ၃ အထိ ကိန်း အနေဖြင့် ရေတွက်၍` | `for i in 1..=3` |
| `န သည် ၁၀၀ အောက် ငယ်နေသရွေ့` | `while n < 100` |
| `ကွင်းဆက် ကို ရပ်မည်။`, `နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်။` | `break`, `continue` |
| `အောက်ပါတို့ကို လုပ်ဆောင်မည်။ … လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။` | `{ … }` |

A counted loop such as `၁ မှ ၃ အထိ` includes both ends, and when the start is larger than
the end, as in `၅ မှ ၁ အထိ`, it counts down. When a branch or a loop has a single
statement, we can write it on the same line. For more than one statement, we open a block
with `အောက်ပါတို့ကို လုပ်ဆောင်မည်။` and end it with the block closer (closing brace)
`လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။`.

Learn: [Control flow](control-flow/README.md).
Docs: [Loops](../manual/language/control-flow.md#loops).

## Functions

```text
<name> ဟူသော လုပ်ဆောင်ချက် သည်
<param> ဟူသော <kind> တစ်ခု ကို လည်းကောင်း <param> ဟူသော <kind> တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ <kind> တစ်ခု ကို ပြန်ပေးမည်။
<value> ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```zaka
// area: two integer parameters, an integer result
ဧရိယာ ဟူသော လုပ်ဆောင်ချက် သည်
အလျား ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း အနံ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
အလျား * အနံ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// write: each parameter has its own preposition
ရေး ဟူသော လုပ်ဆောင်ချက် သည်
စာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လည်းကောင်း နေရာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "သို့" ကို သုံး၍ လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"{နေရာ}: {စာ}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ဧရိယာ ဟူသော လုပ်ဆောင်ချက်ကို ၃ နှင့် ၄ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။ // the long form
ဧရိယာ(၅၊ ၆) ကို ပြသမည်။ // the short form
"မင်္ဂလာပါ" ကို "မှတ်စု" သို့ ရေး မည်။ // a sentence call
```

```text
၁၂
၃၀
မှတ်စု: မင်္ဂလာပါ
```

| Zaka | In other languages |
| --- | --- |
| the function signature of `ဧရိယာ` | `fn area(length: isize, width: isize) -> isize` |
| `အလျား * အနံ ကို ပြန်ပေးမည်။` | `return length * width` |
| `ဧရိယာ ဟူသော လုပ်ဆောင်ချက်ကို ၃ နှင့် ၄ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်` | `area(3, 4)` |
| `ဧရိယာ(၅၊ ၆)` | `area(5, 6)` |
| `… ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး` | `&mut`, `inout` |

In the function signature, we can use `<name> ဟူသော <kind>` to define a parameter and
`ပြန်ပေးမည်` to define a return value, and inside the function we write
`<value> ကို ပြန်ပေးမည်။` to return a value. A call can use the long form
`… ဟူသော လုပ်ဆောင်ချက်ကို … တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်` or the short form `ဧရိယာ(၅၊ ၆)`,
where `၊`, the Burmese comma, separates the arguments. When we give a parameter a
preposition (argument label) such as `သို့`, the call can be written as a sentence, as the
last line of the example shows.

Arguments are passed as duplicates (copies), so a function cannot change the caller's
variable unless we mark the parameter with `ဒေတာ ပြောင်းလဲရန်` to make it a mutable
parameter (`inout`). A function can be called before it is declared.

Learn: [Functions](functions/README.md), [Parameters](functions/parameters.md), [Results](functions/results.md).
Docs: [Functions](../manual/language/functions.md).

## Text

```text
"… {<expression>} …"
<text> + <text>
<text> ၏ စာလုံးအရေအတွက်
```

```zaka
ကိန်းရှင် အမည် သည် "မောင်မောင်" ဖြစ်ပါစေ။
ကိန်းရှင် အသက် သည် ၂၅ ဖြစ်ပါစေ။
"{အမည်} သည် နောက်နှစ်တွင် {အသက် + ၁} နှစ်" ဟု ပြသမည်။ // an expression in braces
ကိန်းရှင် နှုတ်ဆက် သည် "မင်္ဂလာပါ " + အမည် ဖြစ်ပါစေ။ // join with +
နှုတ်ဆက် ကို နှုတ်ဆက် + "!" ဟု ထားမည်။
နှုတ်ဆက် ကို ပြသမည်။
အမည် ၏ စာလုံးအရေအတွက် ကို ပြသမည်။ // characters
အမည် ၏ အကွက်အရေအတွက် ကို ပြသမည်။ // cells
```

```text
မောင်မောင် သည် နောက်နှစ်တွင် ၂၆ နှစ်
မင်္ဂလာပါ မောင်မောင်!
၁၀
၃၀
```

| Zaka | In other languages |
| --- | --- |
| `"{အမည်} … {အသက် + ၁}"` | `f"{name} … {age + 1}"` |
| `"မင်္ဂလာပါ " + အမည်` | `"hello " + name` |
| `အမည် ၏ စာလုံးအရေအတွက်` | `len(name)`, `name.chars().count()` |
| `အမည် ၏ အကွက်အရေအတွက်` | `name.len()`, `strlen(name)` |

Text (string) is stored as UTF-8, and a Burmese letter uses three cells of one octet (byte)
each, so `၏ အကွက်အရေအတွက်` (the cell count) and `၏ စာလုံးအရေအတွက်` (the character count)
return different numbers. To insert a value into text, we put an expression in braces, and
to write a literal brace we use `{{`.

Learn: [Text](text/README.md), [Cells and characters](text/characters.md).
Docs: [Text](../manual/language/text.md).

## Lists and tables

```text
ကိန်းရှင် <name> သည် <kind> စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [<a>၊ <b>] ဖြစ်ပါစေ။
<list> ၏ အကွက် <i>
<list> ထဲသို့ <value> ကို ထည့်မည်။
<list> ၏ အကွက်တိုင်း ကို <name> အနေဖြင့် ယူ၍ <block>
<table> ထဲမှ အညွှန်း <index> ၏ တန်ဖိုး
```

```zaka
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၀၊ ၂၀] ဖြစ်ပါစေ။
ကိန်းများ ထဲသို့ ၃၀ ကို ထည့်မည်။ // append
ကိန်းများ ၏ အကွက် ၀ ကို ပြသမည်။ // cell 0
ကိန်းများ ၏ အကွက်အရေအတွက် ကို ပြသမည်။ // length
ကိန်းများ ၏ အကွက်တိုင်း ကို က အနေဖြင့် ယူ၍ က * ၂ ကို ပြသမည်။ // for each
ကိန်းရှင် ဖုန်း သည် ["မမ": ၉၅၁၁၁] ဖြစ်ပါစေ။ // a table from text to integers
ဖုန်း ထဲမှ အညွှန်း "ကို" ၏ တန်ဖိုး ကို ၉၅၂၂၂ ဟု ထားမည်။
ဖုန်း ကို ပြသမည်။
ဖုန်း ထဲမှ အညွှန်း "ဇော်" ၏ တန်ဖိုး ကို ပြသမည်။ // no such row: a blank
```

```text
၁၀
၃
၂၀
၄၀
၆၀
["မမ": ၉၅၁၁၁၊ "ကို": ၉၅၂၂၂]
အလွတ်
```

| Zaka | In other languages |
| --- | --- |
| `ကိန်းပြည့် စာရင်း` | `Vec<isize>`, `list` |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း` | `[isize; 3]`, `int[3]` |
| `ကိန်းများ ထဲသို့ ၃၀ ကို ထည့်မည်။` | `nums.append(30)` |
| `ကိန်းများ ၏ အကွက်တိုင်း ကို က အနေဖြင့် ယူ၍` | `for k in nums:` |
| `ဖုန်း ထဲမှ အညွှန်း "ဇော်" ၏ တန်ဖိုး` | `phone.get("ဇော်")` |

We add a cell to the end of a list (dynamic array) with `ထဲသို့ … ကို ထည့်မည်`, so a list
can grow, while a counted list (fixed-size array) such as `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း`
keeps a fixed number of cells. To read a cell (element), we use its cell number (index) with
`၏ အကွက် i`, which counts from 0, or with `၏ i ခုမြောက်အကွက်`, which counts from 1.

A table (map) finds each value through an index (key), and we write
`ထဲမှ အညွှန်း <index> ၏ တန်ဖိုး` both to read a value and to set one. A lookup returns an
optional kind (option type), which is `ဒေတာ(…)` when the index exists and `အလွတ်` when it
does not. Lists and tables are values, so assigning one to a new variable makes a duplicate,
just as with structs.

Learn: [Lists](lists/README.md), [Tables](tables/README.md), [Static data and counted lists](forms/counted-lists.md).
Docs: [Lists](../manual/language/lists.md), [Tables](../manual/language/tables.md).

## Structs and shared objects

```text
<name> ဟူသော ဒေတာအစုအဝေးတွင် <field> ဟူသော <kind> တစ်ခု နှင့် <field> ဟူသော <kind> တစ်ခု တို့ပါဝင်သည်။
<name> ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် <field> ဟူသော <kind> တစ်ခု ပါဝင်သည်။
<name>(<field>: <value>၊ <field>: <value>)
<value> ၏ <field>
```

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု နှင့် အသက် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။
အကောင့် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ကိန်းရှင် ကျော် သည် လူ(အမည်: "ကျော်"၊ အသက်: ၂၀) ဖြစ်ပါစေ။
ကိန်းရှင် မွန် သည် ကျော် ဖြစ်ပါစေ။ // a duplicate
မွန် ၏ အသက် ကို ၃၀ ဟု ထားမည်။
ကျော် ကို ပြသမည်။ // stands as it was
ကိန်းရှင် က သည် အကောင့်(လက်ကျန်: ၁၀၀) ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။ // one account, two names
ခ ၏ လက်ကျန် ကို ၅၀ ဟု ထားမည်။
က ကို ပြသမည်။ // changed through ခ
```

```text
လူ(အမည်: "ကျော်"၊ အသက်: ၂၀)
အကောင့်(လက်ကျန်: ၅၀)
```

| Zaka | In other languages |
| --- | --- |
| `လူ ဟူသော ဒေတာအစုအဝေးတွင် …` | `struct Person { name: String, age: isize }` |
| `အကောင့် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် …` | `class Account:`, `Rc<RefCell<Account>>` |
| `လူ(အမည်: "ကျော်"၊ အသက်: ၂၀)` | `Person { name: …, age: 20 }` |
| `ကျော် ၏ အသက်` | `kyaw.age` |
| `ခ ထဲသို့ က ကို ကူးထည့်မည်။`, `ခ ထဲသို့ က ကို အကုန်ကူးထည့်မည်။` | `copy.copy(k)`, `copy.deepcopy(k)` |

We declare a struct with its fields after `ဒေတာအစုအဝေးတွင်`, build one with
`လူ(အမည်: …၊ အသက်: …)`, and read a field with `၏`, as in `ကျော် ၏ အသက်`. A struct is a
value, as in C and Rust, so assignment and passing make a duplicate, and changing `မွန်`
leaves `ကျော်` as it was. When we add `မျှဝေနိုင်သော` to the declaration, we get a shared
object (reference type), which works like an object in Python or Java: both names point to
the same object. To duplicate a shared object, we use `ကူးထည့်မည်`, and for a deep copy we
use `အကုန်ကူးထည့်မည်`.

Learn: [Structs](structs/README.md), [Shared objects](structs/objects.md).
Docs: [Structs](../manual/language/structs.md), [Values and objects](../manual/language/objects.md).

## Enumerations and switches

```text
<name> ဟူသော ဒေတာအမျိုးအစားကွဲတွင် <member>၊ <member> နှင့် <member> တို့ပါဝင်သည်။
<name>များထဲမှ <member>
<subject> သည် <case> ဖြစ်လျှင် <block>
<case> ဖြစ်လျှင် <block>
တခြား <kind> တစ်ခု ဖြစ်လျှင် <block>
```

```zaka
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ နှင့် စိမ်း တို့ပါဝင်သည်။

ကိန်းရှင် မီး သည် အရောင်များထဲမှ ဝါ ဖြစ်ပါစေ။
မီး ကို ပြသမည်။
မီး သည် နီ ဖြစ်လျှင် "ရပ်" ဟု ပြသမည်။ // a switch: one arm per member
ဝါ ဖြစ်လျှင် "ဖြည်းဖြည်း" ဟု ပြသမည်။
စိမ်း ဖြစ်လျှင် "သွား" ဟု ပြသမည်။
မီး ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။ // the member's number
```

```text
အရောင်များထဲမှ ဝါ
ဖြည်းဖြည်း
၁
```

| Zaka | In other languages |
| --- | --- |
| `အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ နှင့် စိမ်း တို့ပါဝင်သည်။` | `enum Color { Red, Yellow, Green }` |
| `အရောင်များထဲမှ ဝါ` | `Color::Yellow` |
| `မီး သည် နီ ဖြစ်လျှင် …` | `match light { … }`, `switch (light)` |
| `တခြား အရောင် တစ်ခု ဖြစ်လျှင်` | `_ =>`, `default:` |

An enumeration (enum) is a set of named constants, as in C, and we refer to a member
(variant) with `<name>များထဲမှ <member>`, as in `အရောင်များထဲမှ ဝါ`. Each member has a
number or a text value, which a conversion returns. In a switch (`match`), each arm starts
with `<case> ဖြစ်လျှင်`, and the switch must list each member or end with a default arm
`တခြား <kind> တစ်ခု ဖြစ်လျှင်` (ZK0458). Arms need no `break`, and the switch ends at the
first sentence that is not an arm.

A member can hold data (payload) of its own data kind instead, as a Rust `enum` with data
does. A test names the member and can give the data a name for the block it guards:

```zaka
ပုံ ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ဒေတာအနေဖြင့် ကိန်းပြည့် တစ်ခု ရှိသည့် စက်ဝိုင်း နှင့် အမှတ် တို့ပါဝင်သည်။

ကိန်းရှင် ပ သည် ပုံများထဲမှ စက်ဝိုင်း(၅) ဖြစ်ပါစေ။
ပ ကို ပြသမည်။
ပ သည် ရ ဟူသည့် စက်ဝိုင်း အဖြစ်သို့ ပြောင်းလဲနိုင်လျှင် ရ * ၂ ကို ပြသမည်။ // match p { Circle(r) => … }
အမှတ် ဖြစ်လျှင် "အမှတ်" ဟု ပြသမည်။
```

```text
ပုံများထဲမှ စက်ဝိုင်း(၅)
၁၀
```

Learn: [Enumerations and switches](enums/README.md), [Members that hold data](enums/data.md).
Docs: [Enumerations](../manual/language/enums.md).

## Results and optionals

```text
<kind> သို့မဟုတ် အမှားအယွင်းအနေဖြင့် <kind> ရှိနိုင်သော ဒေတာဆဲလ်
<kind> ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်
"<message>" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
အကယ်၍ <result> ထဲတွင် အမှားအယွင်း ရှိလျှင် <block>
<optional> ထဲမှ ဒေတာ၊ မရှိလျှင် <fallback>
```

```zaka
// divide: a good integer, or an error value of text
စားခွဲ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ ခ သည် ၀ နှင့် ညီလျှင် "သုညဖြင့် စား၍ မရပါ" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
က / ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

စားခွဲ(၁၀၊ ၂) ကို ပြသမည်။
ကိန်းရှင် ရ သည် စားခွဲ(၁၀၊ ၀) ဖြစ်ပါစေ။
အကယ်၍ ရ ထဲတွင် အမှားအယွင်း ရှိလျှင် "မအောင်မြင်: {ရ ထဲမှ အမှားအယွင်း ဒေတာ}" ဟု ပြသမည်။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အလွတ် ဖြစ်ပါစေ။
ခ ကို ပြသမည်။
ခ ထဲမှ ဒေတာ၊ မရှိလျှင် ၀ ကို ပြသမည်။ // a fallback
ခ ကို ၇ ဟု ထားမည်။
အကယ်၍ ခ ထဲတွင် ဒေတာ ရှိလျှင် ခ ထဲမှ ဒေတာ ကို ပြသမည်။
```

```text
ရလဒ်ကောင်း(၅)
မအောင်မြင်: သုညဖြင့် စား၍ မရပါ
အလွတ်
၀
၇
```

| Zaka | In other languages |
| --- | --- |
| `ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ်` | `Result<isize, String>` |
| `ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` | `Option<isize>`, `Optional[int]` |
| `အလွတ်` | `None` |
| `စားခွဲ(က၊ ခ)?` | `divide(k, kh)?` |
| `ခ ထဲမှ ဒေတာ၊ မရှိလျှင် ၀` | `kh.unwrap_or(0)` |

A result kind (result type) contains either a good result (`Ok` value) or an error value
(error), and an optional kind (option type) contains either data or the blank (`None`,
null), which we write as `အလွတ်`. Zaka has no exceptions, so a function that can fail shows
this in its result kind, and we return a failure with `"…" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။`.
To pass a failure on (error propagation) to the caller, we put `?` after the call, as in
Rust.

To check a result, we use `ထဲတွင် အမှားအယွင်း ရှိလျှင်`, and to read an optional, we test it
with `ထဲတွင် ဒေတာ ရှိလျှင်` or give a fallback with `ထဲမှ ဒေတာ၊ မရှိလျှင် <value>`. Nothing
but a pointer can be null, and reading `ထဲမှ ဒေတာ` from a blank optional stops the program.

Learn: [Errors and results](error-handling/README.md), [Optional values](optionals/README.md).
Docs: [Errors and results](../manual/language/error-handling.md), [Optional values](../manual/language/optionals.md).

## Generics and traits

```text
<name> ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည် …
<trait> ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
<kind> တွင် <trait> ဂုဏ်သတ္တိရှိစေရန် အောက်ပါလုပ်ဆောင်ချက်များ ကို ထည့်သွင်းမည်။
ဤနေရာတွင် သတိပြုရန်မှာ T တွင် <trait> ဂုဏ်သတ္တိ ရှိသည်။ သတိပြုစရာများ ပြီးဆုံးပါပြီး။
```

```zaka
// a trait: each data kind that has it returns a name
နာမည်ရှိသော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
နာမည်ယူ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော A တစ်ခု ကို လက်ခံပြီး စာသား တစ်ခု ကို ပြန်ပေးမည်။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

ကြောင် ဟူသော ဒေတာအစုအဝေးတွင် နာမည် ဟူသော စာသား တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ကောင် ဖြစ်သည်။

// the struct ကြောင် takes the trait
ကြောင် တွင် နာမည်ရှိသော ဂုဏ်သတ္တိရှိစေရန် အောက်ပါလုပ်ဆောင်ချက်များ ကို ထည့်သွင်းမည်။
နာမည်ယူ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကြောင် တစ်ကောင် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
က ၏ နာမည် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ထည့်သွင်းမည့် လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

// a generic function over T, for each T with the trait
နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
က ဟူသော T တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
ဤနေရာတွင် သတိပြုရန်မှာ
T တွင် နာမည်ရှိသော ဂုဏ်သတ္တိ ရှိသည်။
သတိပြုစရာများ ပြီးဆုံးပါပြီး။
"မင်္ဂလာပါ " + နာမည်ယူ(က) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

နှုတ်ဆက်(ကြောင်(နာမည်: "ဖြူဖြူ")) ကို ပြသမည်။
```

```text
မင်္ဂလာပါ ဖြူဖြူ
```

| Zaka | In other languages |
| --- | --- |
| `နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ …` | `fn greet<T>(k: T) -> String` |
| `နာမည်ရှိသော ဟူသည့် ဂုဏ်သတ္တိ …` | `trait Named { fn name(k: &Self) -> String; }` |
| `ကြောင် တွင် နာမည်ရှိသော ဂုဏ်သတ္တိရှိစေရန် …` | `impl Named for Cat { … }` |
| `T တွင် နာမည်ရှိသော ဂုဏ်သတ္တိ ရှိသည်` | `where T: Named` |

To make a generic, we add `ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍` to the function
signature, which adds a kind parameter (type parameter) `T`. A trait (interface) lists the
functions a data kind must have, and we add those functions to a data kind with
`<kind> တွင် <trait> ဂုဏ်သတ္တိရှိစေရန် …`. To require a trait, we write
`T တွင် <trait> ဂုဏ်သတ္တိ ရှိသည်` inside the `ဤနေရာတွင် သတိပြုရန်မှာ` block, much like
`where T: Named` in Rust.

The compiler builds one instance (monomorphized instance) of a generic for each set of data
kinds the program uses, and it checks each instance with the data kinds it was built for.
Because the struct `ကြောင်` sets its measure word to `ကောင်`, a parameter of that data kind
is written `ကြောင် တစ်ကောင်`. We can also use the sign form `နှုတ်ဆက်<T>`.

Learn: [Generics](generics/README.md), [Traits](generics/traits.md).
Docs: [Generics](../manual/language/generics.md), [Traits](../manual/language/traits.md).

## Modules

```text
<name> ဟူသော အများသုံး လုပ်ဆောင်ချက် သည် …
"<file>" မှ <name> နှင့် <name> တို့ကို ထည့်သွင်းမည်။
"<file>" ကို ထည့်သွင်းမည်။
"စံကား/<module>" မှ <name> ကို ထည့်သွင်းမည်။
```

```zaka-file
// math.zaka
နှစ်ထပ် ဟူသော အများသုံး လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```zaka-file
// main.zaka
"math.zaka" မှ နှစ်ထပ် ကို ထည့်သွင်းမည်။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
နှစ်ထပ်(၇) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

`zakac run main.zaka` prints `၄၉`. To use a module of the stdlib (standard library), we
write the same import with a path that starts with `စံကား/`:

```zaka
"စံကား/စာသား" မှ ခွဲ နှင့် ဆက် တို့ကို ထည့်သွင်းမည်။

ကိန်းရှင် သီးများ သည် "ပန်း၊ငှက်ပျော၊သရက်" ကို "၊" ဖြင့် ခွဲ သည့် ရလဒ် ဖြစ်ပါစေ။
သီးများ ကို ပြသမည်။
သီးများ ကို " / " ဖြင့် ဆက် သည့် ရလဒ် ကို ပြသမည်။
```

```text
["ပန်း"၊ "ငှက်ပျော"၊ "သရက်"]
ပန်း / ငှက်ပျော / သရက်
```

| Zaka | In other languages |
| --- | --- |
| `အများသုံး` | `pub` |
| `"math.zaka" မှ နှစ်ထပ် ကို ထည့်သွင်းမည်။` | `from math import square`, `use math::square;` |
| `"math.zaka" ကို ထည့်သွင်းမည်။` | `from math import *` |
| `"စံကား/စာသား" မှ ခွဲ ကို ထည့်သွင်းမည်။` | `use std::…`, `#include <string.h>` |

A module is one file, and its top-level names are private unless we mark them with
`အများသုံး`. In an import, we write the path in quotes, relative to the folder of the
importing file, and each file is read once, even when several files import it. Two files that
import each other are an error (ZK0231). `ပြသမည်` and the other built-ins come from
the core module (prelude), so these need no import.

Learn: [Modules](modules/README.md), [The stdlib](stdlib/README.md).
Docs: [Modules](../manual/language/modules.md), [The stdlib](../manual/language/stdlib.md).

## Threads

```text
<function> ဟူသော လုပ်ဆောင်ချက်ကို <arguments> ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း
<function> ဟူသော လုပ်ဆောင်ချက်ကို <arguments> ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
<thread> ကို စောင့်သော ရလဒ်
<channel> ထဲသို့ <value> ကို ပို့မည်။
```

```zaka
စုပေါင်း ဟူသော လုပ်ဆောင်ချက် သည် ဂဏန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် စု သည် ၀ ဖြစ်ပါစေ။
ဂဏန်းများ ၏ အကွက်တိုင်း ကို ဂ အနေဖြင့် ယူ၍ စု ကို စု + ဂ ဟု ထားမည်။
စု ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// start two threads that run စုပေါင်း side to side with the program
ကိန်းရှင် ရှေ့ သည် စုပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို [၁၊ ၂၊ ၃] ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ကိန်းရှင် နောက် သည် စုပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို [၄၊ ၅၊ ၆] ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
// wait for each and add the results
(ရှေ့ ကို စောင့်သော ရလဒ်) + (နောက် ကို စောင့်သော ရလဒ်) ကို ပြသမည်။
```

```text
၂၁
```

| Zaka | In other languages |
| --- | --- |
| `စုပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို … ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း` | `thread::spawn(…)`, `pthread_create` |
| `ရှေ့ ကို စောင့်သော ရလဒ်` | `handle.join()` |
| `စာသား ပို့နိုင်သော ဒေတာလမ်းကြောင်း` | `mpsc::channel::<String>()`, `queue.Queue()` |
| `လမ်း ထဲသို့ "ထမင်း" ကို ပို့မည်။` | `tx.send(…)` |
| `၅၀၀ မီလီစက္ကန့် စောင့်မည်။` | `thread::sleep(Duration::from_millis(500))`, `time.sleep(0.5)` |

To start a thread, we add `အပြိုင်` to a call, and the long form
`… အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း` returns a value that we can wait on with
`ကို စောင့်သော ရလဒ်`. A thread gets a duplicate of each argument, and it returns its work
through its result or through a channel. A thread cannot read or write a variable at the top
of a file (ZK0726), and we cannot pass a plain shared object to a thread (ZK0494), because
two threads could write to it at once.

The VM runs the threads one after another in short turns, in a fixed order, so the output is
the same on each run, while a native program runs them at the same time as light tasks
on a pool of OS threads, as goroutines run in Go, so tens of thousands of threads
fit in one program.

Learn: [Threads](threads/README.md), [Channels](threads/channels.md).
Docs: [Threads](../manual/language/threads.md).

## Unsafe code and C

```text
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။ … လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
<kind> လိပ်စာ
<variable> ၏ လိပ်စာ
<address> ၏ တန်ဖိုး
<name> ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "<lib>" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ … ပြန်ပေးမည်။
```

```zaka
// abs from the C lib: each C function is unsafe
abs ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
n ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။

ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ညွှန်း သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အသက် ၏ လိပ်စာ ဖြစ်ပါစေ။ // a pointer to အသက်
ညွှန်း ၏ တန်ဖိုး ကို ၂၀ ဟု ထားမည်။ // write through it
abs ဟူသော လုပ်ဆောင်ချက်ကို -၄၂ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အသက် ကို ပြသမည်။
```

```text
၄၂
၂၀
```

| Zaka | In other languages |
| --- | --- |
| `အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။` | `unsafe { … }` |
| `ကိန်းပြည့် လိပ်စာ` | `*mut isize` |
| `အသက် ၏ လိပ်စာ`, `&အသက်` | `&age` |
| `ညွှန်း ၏ တန်ဖိုး`, `*ညွှန်း` | `*p` |
| `abs ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော …` | `extern "C" { fn abs(n: i32) -> i32; }` |

To take an address (pointer) or call C, we need unsafe code (ZK0468), so we put these
statements in a block that starts with `အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။`. We
declare a C function with `စိတ်မချရသော` and `"libc" မှလာသော`, and we read or write through
an address with `၏ တန်ဖိုး`. Safe code can still store an address, pass it around and
compare it with the null address (null pointer, `NULL`), `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`. To
run a program that calls C, use `zakac run --allow-ffi`.

Learn: [Unsafe Zaka](pointers/README.md), [Calling C](functions/c.md).
Docs: [Unsafe Zaka](../manual/language/unsafe.md), [Functions](../manual/language/functions-c.md#c-functions).

## Where next

- The [Learn book](README.md) teaches each feature in small steps, with exercises.
- The [Docs](../manual/README.md) have the full rules for [the language](../manual/language/README.md),
  [the compiler commands](../manual/compiler/commands.md) and
  [the error messages](../manual/reference/errors.md).
- The [list of terms](../manual/reference/terms.md) matches each Zaka term with its standard name.
