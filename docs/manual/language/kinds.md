---
title: Data kinds and values
description: The data kinds (types) of Zaka - integers, decimal numbers, characters, true/false values, text, lists, shared objects and pointers - with their ranges, sizes, measure words and initial values.
---

# Data kinds and values

Each value in a Zaka program has a data kind (type), which sets how much room the value
uses, which values it can have and which operations work on it. The compiler works out
the data kind of each expression before the program runs, and a variable keeps the data
kind it was declared with for its whole life. We write a data kind in Burmese, while
compiler messages use a short English name for it, which the tables below list.

[Scalar values](../../learn/values/README.md) in the Learn book introduces the scalar
kinds with gentle examples. This page states the full rules.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။
ကိန်းရှင် အမြင့် သည် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁.၇၅ ဖြစ်ပါစေ။
ကိန်းရှင် အက္ခရာ သည် စာလုံး တစ်လုံး ဖြစ်၍ တန်ဖိုးသည် 'က' ဖြစ်ပါစေ။
ကိန်းရှင် အောင် သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် မှန် ဖြစ်ပါစေ။
ကိန်းရှင် အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မမ" ဖြစ်ပါစေ။
"{အသက်} {အမြင့်} {အက္ခရာ} {အောင်} {အမည်}" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၈ ၁.၇၅ က မှန် မမ
```

| Data kind | Standard term | Short name | Section |
| --- | --- | --- | --- |
| `ကိန်းပြည့်` | integer | `isize`, `i8` … `u64` | [Integer kinds](#integer-kinds) |
| `ဒဿမကိန်း` | float | `f64`, `f32` | [Decimal number kinds](#decimal-number-kinds) |
| `စာလုံး` | character, `char` | `char` | [Characters](#characters) |
| `မှန်မှားတန်ဖိုး` | boolean | `bool` | [True/false values](#truefalse-values) |
| `စာသား` | string | `text` | [Text](#text) |

## Integer kinds

```text
[လက္ခဏာရှိ | လက္ခဏာမဲ့] [<N> ဘစ်] ကိန်းပြည့်
```

To write an integer kind, we put up to three parts in this order: a sign word, a size
and the word `ကိန်းပြည့်`. The first two can be left out.

| Part | Words | When it is left out | Error |
| --- | --- | --- | --- |
| sign | `လက္ခဏာရှိ` (signed) or `လက္ခဏာမဲ့` (unsigned) | the integer is signed | ZK0217 before a data kind other than `ကိန်းပြည့်` |
| size | `၈ ဘစ်`, `၁၆ ဘစ်`, `၃၂ ဘစ်` or `၆၄ ဘစ်`, in Burmese or Western digits | the integer has the platform size | ZK0216 for another bit count |

Signed integers use two's complement.

| Data kind | Short name | Smallest | Largest |
| --- | --- | --- | --- |
| `၈ ဘစ် ကိန်းပြည့်` | `i8` | -128 | 127 |
| `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်` | `u8` | 0 | 255 |
| `၁၆ ဘစ် ကိန်းပြည့်` | `i16` | -32,768 | 32,767 |
| `လက္ခဏာမဲ့ ၁၆ ဘစ် ကိန်းပြည့်` | `u16` | 0 | 65,535 |
| `၃၂ ဘစ် ကိန်းပြည့်` | `i32` | -2,147,483,648 | 2,147,483,647 |
| `လက္ခဏာမဲ့ ၃၂ ဘစ် ကိန်းပြည့်` | `u32` | 0 | 4,294,967,295 |
| `၆၄ ဘစ် ကိန်းပြည့်` | `i64` | -2^63 | 2^63 - 1 |
| `လက္ခဏာမဲ့ ၆၄ ဘစ် ကိန်းပြည့်` | `u64` | 0 | 2^64 - 1 |
| `ကိန်းပြည့်`, `လက္ခဏာရှိ ကိန်းပြည့်` | `isize` | as the platform size | |
| `လက္ခဏာမဲ့ ကိန်းပြည့်` | `usize` | as the platform size | |

### The platform size

`isize` and `usize` get their width from the platform, which is 64 bits unless we pick
8, 16, 32 or 64 with `zakac run --target-bits N` (or the same flag on `semcheck`). Plain
`ကိန်းပြည့်` is the right choice when no size matters, and it is the data kind that an
integer literal gets when its place asks for nothing else.

```sh
zakac run --target-bits 32 program.zaka
```

`isize` and `i64` are two different data kinds even when the platform has 64 bits, and so
are `usize` and `u64`, so neither goes into the other without a
[conversion](conversions.md):

```zaka-error ZK0404
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ၆၄ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
က + ခ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0404]: cannot combine i64 and isize without a conversion
```

## Decimal number kinds

```text
[<N> ဘစ်] ဒဿမကိန်း
```

A decimal number kind (float) is `ဒဿမကိန်း` with an optional size of 32 or 64 bits, and
without a size it has 64 bits. Both sizes are the base-2 formats of IEEE 754, and Zaka
has no unsigned decimal kind.

| Data kind | Short name | Sign / exponent / fraction bits | Correct digits | Largest finite |
| --- | --- | --- | --- | --- |
| `၃၂ ဘစ် ဒဿမကိန်း` | `f32` | 1 / 8 / 23 | about 7 | about 3.4 × 10^38 |
| `ဒဿမကိန်း`, `၆၄ ဘစ် ဒဿမကိန်း` | `f64` | 1 / 11 / 52 | about 15 | about 1.8 × 10^308 |

A decimal number can also be `inf`, `-inf` or `NaN`. These come from arithmetic, such as
a division with a zero divisor, and none of them stops the program. A literal that is too
big for its decimal kind becomes `inf`, with no error. A decimal number prints with a
decimal point, and with an exponent when it is far from 1.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် သုည သည် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
၁ / သုည ကို ပြသမည်။
-၁ / သုည ကို ပြသမည်။
သုည / သုည ကို ပြသမည်။
သုည / သုည == သုည / သုည ကို ပြသမည်။
1e30 ကို ပြသမည်။
1e39 ဟူသော ၃၂ ဘစ် ဒဿမကိန်း တစ်ခု ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
inf
-inf
NaN
မမှန်
၁e၃၀
inf
```

## Characters

`စာလုံး` (character, `char`) contains one Unicode scalar value in 32 bits, and we write a
character in single quotes, as in `'က'`. We can compare characters with `==`, `!=` and
the ordering operators, and convert them to integers and back, but arithmetic does not
work on them. A character is counted with `လုံး`: `စာလုံး တစ်လုံး`.

## True/false values

`မှန်မှားတန်ဖိုး` (boolean), or the loanword `ဘူလီယန်`, has one of two values, `မှန်`
(true) or `မမှန်` (false), and it prints as that word. It uses one octet (byte), which
contains 1 or 0. A condition must be of this data kind (ZK0402), because an integer is
not true or false.

## Text

`စာသား` (string, short name `text`) contains UTF-8 text in three words: the address of its
u8s, their count and the size of its heap block. Text is a value, so assigning it shares
the u8s until one side writes to them ([Values and objects](objects.md)), and a string
literal points at static text until its first edit. [Text](text.md) states the rules.

## The edges of a data kind

A value remains inside the range of its data kind. A literal outside that range is an error
at compile time, and arithmetic that goes past an edge while the program runs stops the
program, or wraps around under `--overflow wrap` ([Expressions](expressions.md) has the
rules).

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အနည်း သည် ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် -၁၂၈ ဖြစ်ပါစေ။
ကိန်းရှင် အများ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၅၅ ဖြစ်ပါစေ။
အနည်း ကို ပြသမည်။
အများ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
-၁၂၈
၂၅၅
```

```zaka-error ZK0406
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အများ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၅၆ ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0406]: the literal 256 does not fit u8
```

```zaka-error ZK0216
ကိန်းရှင် က သည် ၇ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
```

```text
error[ZK0216]: ၇ is not a valid bit count here; use 8, 16, 32, 64
```

## Data kinds do not mix on their own

A true/false value is not a number, a character is not an integer, and a decimal number
is not an integer. To use a value where a different data kind is expected, we write a
[conversion](conversions.md), unless the change is one of the widenings that
[Conversions](conversions.md#widening-the-change-that-needs-no-words) lists.

## Measure words

```text
<kind> တစ်<measure word>
```

Burmese counts a thing with `တစ်` ("one") and a measure word (classifier) that fits the
thing, and Zaka counts a data kind the same: in each place where we write a data kind, we
put `တစ်` and its measure word after it.

| Place | Written |
| --- | --- |
| a declaration or a generic loop | `ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။` |
| a parameter | `l ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လက်ခံပြီး` |
| a field | `အမည် ဟူသော စာသား တစ်ခု` |
| a result | `စာလုံး တစ်လုံး ကို ပြန်ပေးမည်` |
| a parameter or result of a function kind | `(ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု` |
| a conversion | `x ကို ၈ ဘစ် ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်` |
| a size | `ကိန်းပြည့် တစ်ခု ၏ အရွယ်အစား` |
| the default arm of a switch | `တခြား အရောင် တစ်ခု ဖြစ်လျှင်` |
| a literal with a data kind | `၁၂၇ ဟူသော ၈ ဘစ် ကိန်းပြည့် တစ်ခု` |

Each data kind has one measure word:

| Data kind | Measure word |
| --- | --- |
| `စာလုံး` | `လုံး` |
| a struct or an enumeration | the word that its declaration gives, or `ခု` when it gives none |
| a kind alias | the word that its declaration gives, or else the word of the data kind behind it |
| each other data kind: the numbers, true/false values, text, lists, pointers, optionals, results and function kinds | `ခု` |

- The word goes after the outermost data kind alone: `ကိန်းပြည့် စာရင်း တစ်ခု`,
  `စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု`, `ကိန်းပြည့် လိပ်စာ တစ်ခု`. A list cell, a
  side of an optional or a result, and the base of a pointer get none, while each
  parameter and the result of a function kind get their own.
- In the declaration of a kind alias, the data kind behind the alias gets no measure word
  ([Kind aliases](aliases.md)).
- A heap size uses the measure word inside a *size word*, such as `ခုစာ` in
  `ကိန်းပြည့် ၃ ခုစာ` ([The size of a data kind](#the-size-of-a-data-kind)).
- A data kind in one of these places with no measure word after it is ZK0234. A measure
  word other than the data kind's own is the warning ZK0509, and since the size and the
  value do not change, the program still runs.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အက္ခရာ သည် စာလုံး တစ်လုံး ဖြစ်၍ တန်ဖိုးသည် 'က' ဖြစ်ပါစေ။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။
အက္ခရာ ကို ပြသမည်။
ကိန်းများ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
က
[၁၊ ၂]
```

```zaka-error ZK0234
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0234]: a kind here takes "တစ်" and a classifier
help: write "ကိန်းပြည့် တစ်ခု"
```

## Initial values

```text
ကနဦးတန်ဖိုး
```

No data kind has a hidden default. A variable declared without a value has none until the
program sets it, and the compiler refuses a read before that ([Set before
read](declarations.md#set-before-read)). When we want a plain starting value, we write
`ကနဦးတန်ဖိုး` ("the initial value", the default value), and the data kind decides what that
value is.

`ကနဦးတန်ဖိုး` gets its data kind from the place where we write it, just as `အလွတ်` does:

| Place | Written |
| --- | --- |
| a declaration | `ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။` |
| an assignment | `က ကို ကနဦးတန်ဖိုး ဟု ထားမည်။` |
| a field of a struct literal | `လူ(အမည်: "မောင်"၊ အသက်: ကနဦးတန်ဖိုး)` |
| a cell of a list literal | `[လူ(အမည်: "မောင်"၊ အသက်: ၁)၊ ကနဦးတန်ဖိုး]` |
| a return | `ကနဦးတန်ဖိုး ကို ပြန်ပေးမည်။` |
| an argument | `ပြ ဟူသော လုပ်ဆောင်ချက်ကို ကနဦးတန်ဖိုး ဖြင့် လုပ်ဆောင်မည်။` |

When nothing gives it a data kind, as in `ကနဦးတန်ဖိုး ကို ပြသမည်။` or
`ကိန်းရှင် က သည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။`, the compiler reports ZK0478.

| Data kind | `ကနဦးတန်ဖိုး` |
| --- | --- |
| integers | `၀` |
| decimal numbers | `၀.၀` |
| `စာလုံး` | the null character, `'\0'` |
| `မှန်မှားတန်ဖိုး` | `မမှန်` |
| `စာသား` | blank text, with no u8s |
| a plain list | no cells |
| an optional | `အလွတ်` |
| a pointer | the null address, `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` |
| a counted list | each cell at its own initial value |
| a struct | each field at its own initial value |
| a shared object | a new object, each field at its own initial value |
| an enumeration | the member marked `ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်` ([Enumerations](enums.md#the-initial-member)); none when no member is marked |
| a function kind, a result, a handle of C | none |

A data kind that contains a part with no initial value, such as a struct with a function
field, has none either, and `ကနဦးတန်ဖိုး` for such a data kind is ZK0479, so we write the
value in full. A shared object kind that refers to itself through an optional field still
has an initial value, because the walk stops at the blank optional. A constant needs a
value (ZK0308), and `ကနဦးတန်ဖိုး` counts as one:
`ကိန်းသေ က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။`.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ၃၂ ဘစ် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် ဃ သည် စာလုံး တစ်လုံး ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် င သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် စ သည် အကွက် ၃ ခုရှိသော မှန်မှားတန်ဖိုး စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
က ကို ပြသမည်။
ခ ကို ပြသမည်။
ဂ ကို ပြသမည်။
ဃ ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
င ကို ပြသမည်။
စ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၀
၀.၀
မမှန်
၀
မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ
[မမှန်၊ မမှန်၊ မမှန်]
```

```zaka-error ZK0478
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကနဦးတန်ဖိုး ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0478]: "ကနဦးတန်ဖိုး" stands where no kind can be worked out
```

```zaka-error ZK0479
ခလုတ် ဟူသော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု နှင့် နှိပ် ဟူသော (ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ခလုတ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0479]: "ခလုတ်" has no initial value: "နှိပ်" inside it has none
```

Having no initial value is not the same as being impossible to build. A struct with a
function field can be built, with a function in the field, but it has no initial value.
A data kind that no program can build is refused at its declaration ([Data kinds no program
can build](objects.md#data-kinds-no-program-can-build)).

## The size of a data kind

```text
<kind> တစ်<measure word> ၏ အရွယ်အစား
<kind> <count> <measure word>စာ
<count> ဘိုက်စာ
```

To get the number of octets (bytes) that one value of a data kind uses, as C does with
`sizeof(T)`, we write `<kind> တစ်<measure word> ၏ အရွယ်အစား` ("the size of one"), with the
[measure word](#measure-words) of the data kind. The pointer and the counted list in the
example are data kinds from [Lists, objects and pointers](#lists-objects-and-pointers),
below.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၈ ဘစ် ကိန်းပြည့် တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
၃၂ ဘစ် ဒဿမကိန်း တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
စာလုံး တစ်လုံး ၏ အရွယ်အစား ကို ပြသမည်။
မှန်မှားတန်ဖိုး တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
ကိန်းပြည့် တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
ကိန်းပြည့် လိပ်စာ တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
အကွက် ၃ ခုရှိသော ၃၂ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁
၄
၄
၁
၈
၈
၁၂
```

### Room for several values

To get the size of several values, as C writes `n * sizeof(T)`, we write the data kind,
the count and a *size word*, which is the measure word of the data kind with `စာ` ("room
for") after it, as in `ကိန်းပြည့် ၃ ခုစာ` or `စာလုံး n လုံးစာ`. To count plain octets
instead, we write `<count> ဘိုက်စာ`. Both forms are values of the data kind
`လက္ခဏာမဲ့ ကိန်းပြည့်` (`usize`) that we can use wherever a value goes, and the [heap
statements](heap.md#heap-statements) use them for their sizes.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် n သည် လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၄ ဖြစ်ပါစေ။
ကိန်းပြည့် ၃ ခုစာ ကို ပြသမည်။
စာလုံး n လုံးစာ ကို ပြသမည်။
၃၂ ဘစ် ကိန်းပြည့် n + ၁ ခုစာ ကို ပြသမည်။
n + ၁ ဘိုက်စာ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၄
၁၆
၂၀
၅
```

- The count is a notation expression of the data kind `လက္ခဏာမဲ့ ကိန်းပြည့်`, and a
  count of another data kind is ZK0401.
- The size word uses the measure word of the data kind: `ခုစာ` for `ကိန်းပြည့်` and
  `လုံးစာ` for `စာလုံး`. Another word is the warning ZK0509.
- `ဘိုက်စာ` closes the whole notation expression before it, so `n + ၁ ဘိုက်စာ` is
  `n + ၁` octets.
- When the count is a constant, the size is a constant too:
  `ကိန်းသေ ကျ သည် ကိန်းပြည့် ၃ ခုစာ ဖြစ်ပါစေ။`

```zaka-error ZK0401
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် n သည် ၄ ဖြစ်ပါစေ။
ကိန်းပြည့် n ခုစာ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0401]: expected usize, found isize
```

## Lists, objects and pointers

Other groups of data kind are built from the scalar kinds, and each group has a page
that states its rules.

| Burmese | Short name | Page |
| --- | --- | --- |
| `ကိန်းပြည့် စာရင်း` (plain list) | `isize[]` | [Counted lists and storage](counted-lists.md#forms) |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း` (counted list) | `isize[3]` | [Counted lists and storage](counted-lists.md#forms) |
| `အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း` | `isize[3][2]` | [Lists](lists.md) |
| a struct `လူ` | `လူ` | [Structs](structs.md) |
| a shared object kind `ဘူး` | `ဘူး@` | [Values and objects](objects.md) |
| a function kind | `fn(isize) -> isize` | [Function values](function-values.md) |
| `ကိန်းပြည့် လိပ်စာ` | `isize*` | [Unsafe Zaka](unsafe.md) |
| `ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ` | `isize**` | [Unsafe Zaka](unsafe.md) |
| `လိပ်စာ` | `ptr` | [Unsafe Zaka](unsafe.md) |
| `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` (the null address) | `null` | [Unsafe Zaka](pointers.md#the-null-address) |

We read a short name from left to right in the order of the Burmese words: the base data
kind first, then `*` for each level of pointer, `[n]` for each level of counted list and
`[]` for a plain one. So `isize[3][2]` is two lists of three, `isize[3][]` is a plain list
of lists of three, and `isize*[2]` is a list of two pointers. A shared object kind is its
name with `@` after it, because a value of that data kind is a reference to the object. A
pointer and an object reference both use the platform size.

Safe code can store a pointer, pass it on and compare it with the null address
`မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`, while reading or writing through a pointer, and making
one, need unsafe code ([Unsafe Zaka](unsafe.md)). The pointer is the one data kind that
can be null: a shared object and a function value contain an object or a function at each
point, and a value that can be missing is an [optional](optionals.md).
