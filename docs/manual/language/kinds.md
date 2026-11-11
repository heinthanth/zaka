---
title: Kinds and values
description: The kinds of value in Zaka - integers, decimal numbers, characters, true/false values, text, lists, shared objects and pointers - with ranges, sizes and initial values.
---

# Kinds and values

Each value in a Zaka program has a *kind*. The kind tells how much room the value takes,
which values are possible and which operations are allowed. The compiler works out the
kind of each expression before the program runs, and a variable keeps the kind it was
declared with for its whole life. A kind is written in Burmese; compiler messages use a
short English name for it, listed below.

[Scalar values](../../learn/values/README.md) in the Learn book introduces the
scalar kinds with gentle examples. This page states the full rules.

## Integer kinds

An integer kind is built from three parts in a fixed order:

```text
(လက္ခဏာရှိ | လက္ခဏာမဲ့)  (N ဘစ်)  ကိန်းပြည့်
 sign, optional           size, optional     the kind word
```

- The sign is `လက္ခဏာရှိ` (signed) or `လက္ခဏာမဲ့` (unsigned). Without a sign word the
  integer is signed. Signed integers use two's complement.
- The size is 8, 16, 32, 64 or 128 bits, written with Burmese or Western digits. Another
  count is ZK0216. Without a size the integer has the *platform size*.
- A sign word can stand before `ကိန်းပြည့်` and no other kind (ZK0217).

| Kind | Short name | Smallest | Largest |
| --- | --- | --- | --- |
| `၈ ဘစ် ကိန်းပြည့်` | `i8` | -128 | 127 |
| `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်` | `u8` | 0 | 255 |
| `၁၆ ဘစ် ကိန်းပြည့်` | `i16` | -32,768 | 32,767 |
| `လက္ခဏာမဲ့ ၁၆ ဘစ် ကိန်းပြည့်` | `u16` | 0 | 65,535 |
| `၃၂ ဘစ် ကိန်းပြည့်` | `i32` | -2,147,483,648 | 2,147,483,647 |
| `လက္ခဏာမဲ့ ၃၂ ဘစ် ကိန်းပြည့်` | `u32` | 0 | 4,294,967,295 |
| `၆၄ ဘစ် ကိန်းပြည့်` | `i64` | -2^63 | 2^63 - 1 |
| `လက္ခဏာမဲ့ ၆၄ ဘစ် ကိန်းပြည့်` | `u64` | 0 | 2^64 - 1 |
| `၁၂၈ ဘစ် ကိန်းပြည့်` | `i128` | -2^127 | 2^127 - 1 |
| `လက္ခဏာမဲ့ ၁၂၈ ဘစ် ကိန်းပြည့်` | `u128` | 0 | 2^128 - 1 |
| `ကိန်းပြည့်`, `လက္ခဏာရှိ ကိန်းပြည့်` | `isize` | as the platform size | |
| `လက္ခဏာမဲ့ ကိန်းပြည့်` | `usize` | as the platform size | |

### The platform size

`isize` and `usize` take their width from the platform: 64 bits unless set otherwise.
`zakac run --platform-bits N` (and `semcheck`) picks 8, 16, 32, 64 or 128 instead. Plain
`ကိန်းပြည့်` is the right choice when no size matters, and it is the kind that an
integer literal gets when its place asks for nothing else.

`isize` and `i64` are two kinds even when the platform has 64 bits, and so are `usize`
and `u64`. Neither goes into the other without a [conversion](conversions.md):

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

```sh
zakac run --platform-bits 32 program.zaka
```

## Decimal number kinds

A decimal number kind is `ဒဿမကိန်း` with an optional size: 16, 32, 64 or 128 bits.
Without a size it has 64 bits. These are the IEEE 754 formats of base 2. There is no
unsigned decimal kind.

| Kind | Short name | Sign / exponent / fraction bits | Correct digits | Largest finite |
| --- | --- | --- | --- | --- |
| `၁၆ ဘစ် ဒဿမကိန်း` | `f16` | 1 / 5 / 10 | about 3 | 65,504 |
| `၃၂ ဘစ် ဒဿမကိန်း` | `f32` | 1 / 8 / 23 | about 7 | about 3.4 × 10^38 |
| `ဒဿမကိန်း`, `၆၄ ဘစ် ဒဿမကိန်း` | `f64` | 1 / 11 / 52 | about 15 | about 1.8 × 10^308 |
| `၁၂၈ ဘစ် ဒဿမကိန်း` | `f128` | 1 / 15 / 112 | about 33 | about 1.2 × 10^4932 |

A decimal number can also be `inf`, `-inf` or `NaN`. These come from arithmetic, such as
a division with a zero divisor, and do not stop the program. A literal that is too big for its float
kind becomes `inf`, with no error. A decimal number prints with a decimal point, and with
an exponent when its size is far from 1.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် သုည သည် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
၁ / သုည ကို ပြသမည်။
-၁ / သုည ကို ပြသမည်။
သုည / သုည ကို ပြသမည်။
သုည / သုည == သုည / သုည ကို ပြသမည်။
1e30 ကို ပြသမည်။
၇၀၀၀၀ ဟူသော ၁၆ ဘစ် ဒဿမကိန်း တစ်ခု ကို ပြသမည်။
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

`စာလုံး` holds one Unicode scalar value: 32 bits, written `'က'`. Characters can be
compared with `==`, `!=` and the ordering operators, and convert to integers and
back. Arithmetic does not take them.

## True/false values

`မှန်မှားတန်ဖိုး` (or the loanword `ဘူလီယန်`) holds `မှန်` or `မမှန်`. It takes one octet,
which holds 1 or 0. A condition must be of this kind (ZK0402): an integer is not true or
false. It prints as `မှန်` or `မမှန်`.

## Text

`စာသား` (`text`) holds UTF-8 text: the address of its u8s, their count and the size of
its heap block, three words in all. Text is a value: assigning it shares the u8s until one
side writes ([Values and objects](objects.md)). String literals point at static text until
their first edit. [Text](text.md) states the rules.

## The edges of a kind

A value keeps inside the range of its kind. A literal that falls outside it is an error
at compile time, and arithmetic that goes past an edge while the program runs stops the
program, or wraps around with `--overflow wrap` ([Expressions](expressions.md) gives the
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
error[ZK0216]: ၇ is not a valid bit count here; use 8, 16, 32, 64, 128
```

## Kinds do not mix on their own

A true/false value is not a number, a character is not an integer and a decimal number
is not an integer. A value goes where a different kind is wanted just through a
[conversion](conversions.md), or through the widening that conversions describes.

## Measure words

Burmese counts a thing with `တစ်` ("one") and a classifier that fits it, and a kind written
in a Zaka program is counted alike: `<kind> တစ်<classifier>`. The word goes after
the kind in each place a kind stands:

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
| a literal with a kind | `၁၂၇ ဟူသော ၈ ဘစ် ကိန်းပြည့် တစ်ခု` |

- The word goes after the outermost kind alone: `ကိန်းပြည့် စာရင်း တစ်ခု`,
  `စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု`, `ကိန်းပြည့် လိပ်စာ တစ်ခု`. A list cell, a side of
  an optional or a result, and the base of a pointer take none. The parameters and the
  result of a function kind take their own.
- `စာလုံး` counts with `လုံး`. A struct, an enumeration and an alias count with the word
  their declaration names, and with `ခု` when it names none; an alias that names none
  counts with the word of its kind. Each other kind counts with `ခု`: the numbers,
  true/false, text, lists, pointers, optionals, results and function kinds.
- The kind of an alias, in its own declaration, takes no word
  ([Kind aliases](aliases.md)).
- A heap size counts its cells with the same word and `စာ`: `စာလုံး ၅ လုံးစာ`,
  `လူ ၃ ယောက်စာ` ([Unsafe Zaka](unsafe.md)).
- A kind in one of these places with no word after it is ZK0234. A word other than the
  kind's own is the warning ZK0509; the size and the value do not change, so the program
  still runs.

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

No kind has a hidden default. A variable declared with no value holds none until the
program sets it, and the check refuses a read before that
([Set before read](statements.md#set-before-read)). Where a program wants a plain starting
value, it writes `ကနဦးတန်ဖိုး` ("the initial value"), and the kind decides what that is.

`ကနဦးတန်ဖိုး` takes its kind from where it stands, as `အလွတ်` does:

| Place | Written |
| --- | --- |
| a declaration | `ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။` |
| an assignment | `က ကို ကနဦးတန်ဖိုး ဟု ထားမည်။` |
| a field of a struct literal | `လူ(အမည်: "မောင်"၊ အသက်: ကနဦးတန်ဖိုး)` |
| a cell of a list literal | `[လူ(အမည်: "မောင်"၊ အသက်: ၁)၊ ကနဦးတန်ဖိုး]` |
| a return | `ကနဦးတန်ဖိုး ကို ပြန်ပေးမည်။` |
| an argument | `လုပ်ဆောင်ချက် ပြ ကို ကနဦးတန်ဖိုး တို့ဖြင့် လုပ်ဆောင်မည်။` |

Where nothing gives a kind, as in `ကနဦးတန်ဖိုး ကို ပြသမည်။` or
`ကိန်းရှင် က သည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။`, it is ZK0478.

| Kind | `ကနဦးတန်ဖိုး` |
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

A kind that holds a part with none, such as a struct with a function field, has none
too. `ကနဦးတန်ဖိုး` for a kind with none is ZK0479: write the value in full. A shared kind
that names itself through an optional field still has an initial value, since the walk
stops at the blank optional. A constant needs a value (ZK0308), and `ကနဦးတန်ဖိုး` is one:
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
ခလုတ် ဟူသော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု၊ နှိပ် ဟူသော (ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ခလုတ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0479]: "ခလုတ်" has no initial value: "နှိပ်" inside it has none
```

Whether a kind can be built at all is a question apart: a struct with a function field can
be built, with a function in the field, but has no initial value. A kind no program can
build is refused at its declaration ([Kinds no program can
build](objects.md#kinds-no-program-can-build)).

## The size of a kind

`<kind> တစ်<word> ၏ အရွယ်အစား` ("the size of one of the kind") is the number of octets
one value takes; the kind takes its [measure word](#measure-words). The
pointer and the counted list in the example are kinds of [Lists, objects and
pointers](#lists-objects-and-pointers), below:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၈ ဘစ် ကိန်းပြည့် တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
၁၆ ဘစ် ဒဿမကိန်း တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
စာလုံး တစ်လုံး ၏ အရွယ်အစား ကို ပြသမည်။
မှန်မှားတန်ဖိုး တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
ကိန်းပြည့် တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
ကိန်းပြည့် လိပ်စာ တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
အကွက် ၃ ခုရှိသော ၃၂ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁
၂
၄
၁
၈
၈
၁၂
```

## Lists, objects and pointers

More families of kind are built from the scalar kinds. Their rules have pages of their
own.

| Burmese | Short name | Page |
| --- | --- | --- |
| `ကိန်းပြည့် စာရင်း` (plain) | `isize[]` | [Lists](lists.md#forms) |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း` (counted) | `isize[3]` | [Lists](lists.md#forms) |
| `အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း` | `isize[3][2]` | [Lists](lists.md) |
| a struct `လူ` | `လူ` | [Structs](structs.md) |
| a shared object kind `ဘူး` | `ဘူး@` | [Values and objects](objects.md) |
| a function kind | `fn(isize) -> isize` | [Function values](function-values.md) |
| `ကိန်းပြည့် လိပ်စာ` | `isize*` | [Unsafe Zaka](unsafe.md) |
| `ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ` | `isize**` | [Unsafe Zaka](unsafe.md) |
| `လိပ်စာ` | `ptr` | [Unsafe Zaka](unsafe.md) |
| `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` (the null address) | `null` | [Unsafe Zaka](unsafe.md#the-null-address) |

A short name reads left to right in the order of the Burmese words: the base kind first,
then `*` for each level of pointer, `[n]` for each counted list level and `[]` for a
plain one. `isize[3][2]` is two lists of three, `isize[3][]` a plain list of lists of
three, and `isize*[2]` a list of two pointers. A shared object kind is its name with `@`
after it, since a value of the kind is a reference to the object. A pointer and an object
reference take the platform size.

Safe code can hold a pointer, pass it on and compare it with the null address
`မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`; reading or writing through it, and making one, need marked
code ([Unsafe Zaka](unsafe.md)). The pointer is the one kind that can hold NULL: a shared
object and a function value hold an object or a function at each point, and a value that can be
missing is an [optional](optionals.md).
