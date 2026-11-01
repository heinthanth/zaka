---
title: Kinds and values
description: The kinds of value in Zaka - integers, decimal numbers, characters, true/false values, text, addresses and lists - with ranges, sizes and default values.
---

# Kinds and values

Each value in a Zaka program has a *kind*. The kind tells how much room the value takes,
which values are possible and which operations are allowed. The compiler works out the
kind of each expression before the program runs, and a variable keeps the kind it was
declared with for its whole life. A kind is written in Burmese; compiler messages use a
short English name for it, listed below.

[Values and kinds](../../learn/values/README.md) in the Learn book introduces the
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
`zakac run --platform-bits N` (and `semantic-check`) picks 8, 16, 32, 64 or 128 instead. Plain
`ကိန်းပြည့်` is the right choice when no size matters, and it is the kind that an
integer literal gets when its place asks for nothing else.

`isize` and `i64` are two kinds even when the platform has 64 bits, and so are `usize`
and `u64`. Neither goes into the other without a [conversion](conversions.md):

```zaka-error ZK0404
ကိန်းရှင် က သည် ၆၄ ဘစ် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
က + ခ ကို ပြသပါ။
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
ကိန်းရှင် သုည သည် ဒဿမကိန်း ဖြစ်ပါစေ။
၁ / သုည ကို ပြသပါ။
-၁ / သုည ကို ပြသပါ။
သုည / သုည ကို ပြသပါ။
သုည / သုည == သုည / သုည ကို ပြသပါ။
1e30 ကို ပြသပါ။
၁၆ ဘစ် ဒဿမကိန်း ၇၀၀၀၀ ကို ပြသပါ။
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

`စာသား` (`text`) owns UTF-8 text: the address of its u8s, their count and the size of
its heap block, three words in all. `စာသား တစ်စိတ်တစ်ပိုင်း` (`&text`) is a view of text,
two words. String literals are of the first kind. [Text](text.md) states their rules.

## Addresses and lists

Two more families of kind are built from the scalar kinds. Their rules have pages of
their own.

| Burmese | Short name | Page |
| --- | --- | --- |
| `ကိန်းပြည့် လိပ်စာ` | `*isize` | [Pointers](pointers.md) |
| `ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ` | `**isize` | [Pointers](pointers.md) |
| `ငှားထားသော ကိန်းပြည့် လိပ်စာ` | `&*isize` | [Ownership and borrowing](ownership.md) |
| `ကိန်းပြည့် လိပ်စာအကြမ်း` | `~isize` | [Ownership and borrowing](ownership.md) |
| `လိပ်စာ` | `ptr` | [Pointers](pointers.md) |
| `ဗလာ` (the null address) | `null` | [Pointers](pointers.md) |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း` | `[3]isize` | [Lists](lists.md) |
| `အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း` | `[2][3]isize` | [Lists](lists.md) |
| `ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း` | `&[]isize` | [Slices](slices.md) |
| `စာသား တစ်စိတ်တစ်ပိုင်း` | `&text` | [Text](text.md) |

A short name reads left to right as the kind is built: `*` for each level of owner
address, `&` in front for a borrow, `~` for a raw level, `[n]` for each list level.
An address takes the platform size.

## The size of a kind

`<kind> ၏ အရွယ်အစား` ("the size of the kind") is the number of octets one value takes:

```zaka
၈ ဘစ် ကိန်းပြည့် ၏ အရွယ်အစား ကို ပြသပါ။
၁၆ ဘစ် ဒဿမကိန်း ၏ အရွယ်အစား ကို ပြသပါ။
စာလုံး ၏ အရွယ်အစား ကို ပြသပါ။
မှန်မှားတန်ဖိုး ၏ အရွယ်အစား ကို ပြသပါ။
ကိန်းပြည့် ၏ အရွယ်အစား ကို ပြသပါ။
ကိန်းပြည့် လိပ်စာ ၏ အရွယ်အစား ကို ပြသပါ။
အကွက် ၃ ခုရှိသော ၃၂ ဘစ် ကိန်းပြည့် စာရင်း ၏ အရွယ်အစား ကို ပြသပါ။
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

## Default values

A variable declared with no value starts at the default of its kind. A constant has no
default: it must be given a value (ZK0308).

| Kind | Default |
| --- | --- |
| integers | `၀` |
| decimal numbers | `၀.၀` |
| `စာလုံး` | the null character, `'\0'` |
| `မှန်မှားတန်ဖိုး` | `မမှန်` |
| `စာသား` | blank text, with no u8s |
| an address | `ဗလာ` |
| a list | each cell at the default of its kind |

```zaka
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ၃၂ ဘစ် ဒဿမကိန်း ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် မှန်မှားတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် ဃ သည် စာလုံး ဖြစ်ပါစေ။
ကိန်းရှင် င သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် စ သည် အကွက် ၃ ခုရှိသော မှန်မှားတန်ဖိုး စာရင်း ဖြစ်ပါစေ။
က ကို ပြသပါ။
ခ ကို ပြသပါ။
ဂ ကို ပြသပါ။
ဃ ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
င ကို ပြသပါ။
စ ကို ပြသပါ။
```

```text
၀
၀.၀
မမှန်
၀
ဗလာ
[မမှန်၊ မမှန်၊ မမှန်]
```

## The edges of a kind

A value keeps inside the range of its kind. A literal that falls outside it is an error
at compile time, and arithmetic that goes past an edge while the program runs wraps
around ([Expressions](expressions.md) gives the rules).

```zaka
ကိန်းရှင် အနည်း သည် ၈ ဘစ် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် -၁၂၈ ဖြစ်ပါစေ။
ကိန်းရှင် အများ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၅၅ ဖြစ်ပါစေ။
အနည်း ကို ပြသပါ။
အများ ကို ပြသပါ။
```

```text
-၁၂၈
၂၅၅
```

```zaka-error ZK0406
ကိန်းရှင် အများ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၅၆ ဖြစ်ပါစေ။
```

```text
error[ZK0406]: the literal 256 does not fit u8
```

```zaka-error ZK0216
ကိန်းရှင် က သည် ၇ ဘစ် ကိန်းပြည့် ဖြစ်ပါစေ။
```

```text
error[ZK0216]: ၇ is not a valid bit count here; use 8, 16, 32, 64, 128
```

## Kinds do not mix on their own

A true/false value is not a number, a character is not an integer and a decimal number
is not an integer. A value goes where a different kind is wanted just through a
[conversion](conversions.md), or through the widening that conversions describes.
