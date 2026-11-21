---
title: Structs
description: Put a few values of different data kinds together under one name, and use each value through its field name.
---

# Structs

A *struct* (record) puts a few related values together under one name. Unlike a list,
whose values all share one data kind (type), a struct keeps each value in a named *field*
(member), and each field can have a different data kind.

## You will learn

- how to declare a struct and create a variable of it
- how to read and change a field, and how to write a whole struct in one go
- what happens when we assign, compare or pass a struct to a function
- how to give a struct its own measure word

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် အမြင့် ဟူသော ဒဿမကိန်း တစ်ခု တို့ပါဝင်သည်။

ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကျော် ကို ပြသမည်။
```

```text
လူ(အသက်: ၀၊ အမြင့်: ၀.၀)
```

The first line declares a struct named `လူ` (person) with two fields, an integer `အသက်`
(age) and a decimal number `အမြင့်` (height). We can use this pattern to declare a struct:

```text
<name> ဟူသော ဒေတာအစုအဝေးတွင် <field> ဟူသော <kind> တစ်ခု နှင့် <field> ဟူသော <kind> တစ်ခု တို့ပါဝင်သည်။
```

| Zaka | Reads as |
| --- | --- |
| `<name> ဟူသော` | "called `<name>`" |
| `ဒေတာအစုအဝေး` | "data group" (the word for a struct) |
| `တွင်` | "in it" |
| `<field> ဟူသော <kind>` | "a `<kind>` called `<field>`" |
| `တစ်ခု` | "one"; `ခု` is the measure word (classifier) for most data kinds |
| `နှင့်` | "and" |
| `တို့ပါဝင်သည်` | "these are included" |

When a struct has three or more fields, we put `၊` between them and `နှင့်` before the
last one.

After the declaration, `လူ` is a data kind just like `ကိန်းပြည့်`, so we can use it
wherever a data kind is expected. In the example, we write `ကျော် သည် လူ တစ်ခု` to create
a variable of that data kind, and we use `ကနဦးတန်ဖိုး` to set each field to its [initial
value](../variables/README.md#the-initial-value) (default value), which is `၀` for an
integer and `၀.၀` for a decimal number. When we print a whole struct, the output shows
the struct name followed with each field as `name: value` inside parentheses, with `၊`
between the fields.

## Reading and writing fields

```text
<variable> ၏ <field>
```

To reach a field, we write the variable, `၏` and the field name, so `ကျော် ၏ အသက်` means
"the age of `ကျော်`". We can read a field like a variable, and we can assign to it with
`ထားမည်`.

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် အမြင့် ဟူသော ဒဿမကိန်း တစ်ခု တို့ပါဝင်သည်။

ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်ပါစေ။
ကျော် ၏ အသက် ကို ၂၀ ဟု ထားမည်။
ကျော် ၏ အမြင့် ကို ၁.၇၅ ဟု ထားမည်။
ကျော် ၏ အသက် ကို ပြသမည်။
(ကျော် ၏ အသက်) + ၁ ကို ပြသမည်။
ကျော် ကို ပြသမည်။
```

```text
၂၀
၂၁
လူ(အသက်: ၂၀၊ အမြင့်: ၁.၇၅)
```

When a field is part of an expression with an operator, we put it in parentheses, just as
we do with a list cell: `(ကျော် ၏ အသက်) + ၁`. Here `ကျော်` is declared without a value,
so we must set each field before we read the whole struct.

## Writing a struct out

We can use a *struct literal* to write out a whole struct in one value, with a value for each field:

```text
<field> အနေဖြင့် <value> နှင့် <field> အနေဖြင့် <value> ရှိသော <name>တစ်ခု
```

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် အမြင့် ဟူသော ဒဿမကိန်း တစ်ခု တို့ပါဝင်သည်။

ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အသက် အနေဖြင့် ၂၀ နှင့် အမြင့် အနေဖြင့် ၁.၇၅ ရှိသော လူတစ်ခု ဖြစ်ပါစေ။
ကျော် ကို ပြသမည်။
ကျော် ကို အမြင့် အနေဖြင့် ၁.၆ နှင့် အသက် အနေဖြင့် ကနဦးတန်ဖိုး ရှိသော လူ တစ်ခု ဟု ထားမည်။
ကျော် ကို ပြသမည်။
```

```text
လူ(အသက်: ၂၀၊ အမြင့်: ၁.၇၅)
လူ(အသက်: ၀၊ အမြင့်: ၁.၆)
```

| Zaka | Reads as |
| --- | --- |
| `အသက် အနေဖြင့် ၂၀` | "age as 20" |
| `နှင့်` | "and", before the last pair; `၊` between the others |
| `ရှိသော` | "that has" |
| `လူတစ်ခု` | "one person"; it can also be written with a space: `လူ တစ်ခု` |

The pairs do not need to follow the order of the declaration, but each field must get a
value. To give a field its initial value, we can use `ကနဦးတန်ဖိုး` as its value, as `အသက်`
shows in the second line of the output.

## The short form

We can also write a struct literal in the same form that a struct is printed in:

```text
<name>(<field>: <value>၊ <field>: <value>)
```

This form needs no measure word, so we can put it straight inside a bracket list. In a
`လူ စာရင်း`, a cell can also be `ကနဦးတန်ဖိုး`, because the data kind of the list shows
which initial value is meant.

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် အမြင့် ဟူသော ဒဿမကိန်း တစ်ခု တို့ပါဝင်သည်။

ကိန်းရှင် ကျော် သည် လူ(အသက်: ၂၀၊ အမြင့်: ၁.၇၅) ဖြစ်ပါစေ။
ကျော် ကို ပြသမည်။
ကိန်းရှင် လူများ သည် လူ စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [လူ(အသက်: ၁၊ အမြင့်: ကနဦးတန်ဖိုး)၊ ကနဦးတန်ဖိုး] ဖြစ်ပါစေ။
လူများ ကို ပြသမည်။
```

```text
လူ(အသက်: ၂၀၊ အမြင့်: ၁.၇၅)
[လူ(အသက်: ၁၊ အမြင့်: ၀.၀)၊ လူ(အသက်: ၀၊ အမြင့်: ၀.၀)]
```

We can use both forms wherever a struct value is expected: as the value of a
declaration, on the right side of `ထားမည်`, as an argument, or with `ပြသမည်`.

## Duplicating a struct

When we assign one struct to another, each field is duplicated (copied), so the two
structs are separate afterwards and a change to one does not affect the other.

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် အမြင့် ဟူသော ဒဿမကိန်း တစ်ခု တို့ပါဝင်သည်။

ကိန်းရှင် ကျော် သည် လူ(အသက်: ၂၀၊ အမြင့်: ကနဦးတန်ဖိုး) ဖြစ်ပါစေ။
ကိန်းရှင် မွန် သည် လူ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကျော် ဖြစ်ပါစေ။
မွန် ၏ အသက် ကို ၃၀ ဟု ထားမည်။
ကျော် ကို ပြသမည်။
မွန် ကို ပြသမည်။
မွန် ကို ကျော် ဟု ထားမည်။
မွန် ကို ပြသမည်။
```

```text
လူ(အသက်: ၂၀၊ အမြင့်: ၀.၀)
လူ(အသက်: ၃၀၊ အမြင့်: ၀.၀)
လူ(အသက်: ၂၀၊ အမြင့်: ၀.၀)
```

## Comparing structs

```text
<struct> == <struct>
<struct> != <struct>
```

We can use `==` and `!=` to compare two structs of the same data kind, which checks them one field at a time:

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု နှင့် အသက် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

ကိန်းရှင် ကျော် သည် လူ(အမည်: "ကျော်"၊ အသက်: ၂၀) ဖြစ်ပါစေ။
ကိန်းရှင် မွန် သည် ကျော် ဖြစ်ပါစေ။
ကျော် == မွန် ကို ပြသမည်။
မွန် ၏ အသက် ကို ၃၀ ဟု ထားမည်။
ကျော် == မွန် ကို ပြသမည်။
```

```text
မှန်
မမှန်
```

## Structs in structs and lists

A field can contain another struct, and a list cell can contain a struct. To reach a field
inside, we add one more `၏` for each step:

```text
<variable> ၏ <field> ၏ <field>
<list> ၏ အကွက် <number> ၏ <field>
```

```zaka
အမှတ် ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု နှင့် y ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။
မျဉ်း ဟူသော ဒေတာအစုအဝေးတွင် စ ဟူသော အမှတ် တစ်ခု နှင့် ဆုံး ဟူသော အမှတ် တစ်ခု တို့ပါဝင်သည်။

ကိန်းရှင် က သည် မျဉ်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
က ၏ ဆုံး ၏ x ကို ၃ ဟု ထားမည်။
က ကို ပြသမည်။
ကိန်းရှင် အမှတ်များ သည် အမှတ် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [အမှတ်(x: ၀၊ y: ၀)၊ အမှတ်(x: ၀၊ y: ၀)၊ အမှတ်(x: ၀၊ y: ၀)] ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက် ၁ ၏ y ကို ၇ ဟု ထားမည်။
အမှတ်များ ကို ပြသမည်။
အမှတ်များ ၏ အကွက် ၁ ကို ပြသမည်။
```

```text
မျဉ်း(စ: အမှတ်(x: ၀၊ y: ၀)၊ ဆုံး: အမှတ်(x: ၃၊ y: ၀))
[အမှတ်(x: ၀၊ y: ၀)၊ အမှတ်(x: ၀၊ y: ၇)၊ အမှတ်(x: ၀၊ y: ၀)]
အမှတ်(x: ၀၊ y: ၇)
```

A field can also contain a list.

## How big a struct is

```text
<kind> တစ်ခု ၏ အရွယ်အစား
```

The fields of a struct are stored next to each other in RAM (memory), in the order of the
declaration. Each field starts at a multiple of its alignment, and the size of the whole
struct is rounded up to the largest alignment among its fields, as in C. We can use
`အရွယ်အစား` to get the size in octets (bytes), and as the example shows, changing the
order of the fields can change the size:

```zaka
ယ ဟူသော ဒေတာအစုအဝေးတွင် အ ဟူသော မှန်မှားတန်ဖိုး တစ်ခု၊ ဘ ဟူသော ကိန်းပြည့် တစ်ခု နှင့် စ ဟူသော မှန်မှားတန်ဖိုး တစ်ခု တို့ပါဝင်သည်။
ရ ဟူသော ဒေတာအစုအဝေးတွင် ဘ ဟူသော ကိန်းပြည့် တစ်ခု၊ အ ဟူသော မှန်မှားတန်ဖိုး တစ်ခု နှင့် စ ဟူသော မှန်မှားတန်ဖိုး တစ်ခု တို့ပါဝင်သည်။
လ ဟူသော ဒေတာအစုအဝေးတွင် ဘ ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု နှင့် အ ဟူသော မှန်မှားတန်ဖိုး တစ်ခု တို့ပါဝင်သည်။

ယ တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
ရ တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
လ တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
```

```text
၂၄
၁၆
၈
```

In the first struct, the one-octet true/false value comes first and is followed with 7
octets of padding, so that the integer starts at a multiple of 8, and another 7 octets of
padding come after the last field. In the second struct, the two true/false values sit
together after the integer, so the padding at the end is shared between them.

## Measure words

```text
<name> ဟူသော ဒေတာအစုအဝေးတွင် <fields> တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ <word> ဖြစ်သည်။
```

Burmese counts things with a measure word (classifier) that fits the thing being counted,
so people are counted with `ယောက်` and most other things with `ခု`, and Zaka does the
same. A struct is counted with `ခု` unless we give a different word at the end of its
declaration, and we then write that word after `တစ်` each time we use the struct as a data
kind or write a struct literal:

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် အတန်း ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

ကိန်းရှင် ကျော် သည် လူ တစ်ယောက် ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် မွန် သည် လူ တစ်ယောက် ဖြစ်၍ တန်ဖိုးသည် အသက် အနေဖြင့် ၁၀ နှင့် အတန်း အနေဖြင့် ၀ ရှိသော လူတစ်ယောက် ဖြစ်ပါစေ။
ကျော် ကို ပြသမည်။
မွန် ကို ပြသမည်။
```

```text
လူ(အသက်: ၀၊ အတန်း: ၀)
လူ(အသက်: ၁၀၊ အတန်း: ၀)
```

| Zaka | Reads as |
| --- | --- |
| `တို့ပါဝင်ပြီး` | "these are included, and" |
| `မျိုးပြပစ္စည်းမှာ ယောက်` | "the measure word is `ယောက်`" |
| `ဖြစ်သည်` | "is" |

## Passing to a function

When we pass a struct to a function, the parameter gets a duplicate of the argument, so
the function changes that duplicate while the caller's struct is left unchanged. To let a
function change the caller's struct, we mark the parameter with `ဒေတာ ပြောင်းလဲရန်`,
which makes it a mutable parameter (`inout`), just as for [the caller's
variable](../functions/parameters.md#changing-the-callers-variable):

```text
<name> ဟူသော <struct> တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
```

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် အတန်း ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။
ပွားကြည့် ဟူသော လုပ်ဆောင်ချက် သည်
သူ ဟူသော လူ တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
သူ ၏ အသက် ကို ၉၉ ဟု ထားမည်။
သူ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အသက်ကြီး ဟူသော လုပ်ဆောင်ချက် သည်
သူ ဟူသော လူ တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
သူ ၏ အသက် ကို (သူ ၏ အသက်) + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ကျော် သည် လူ(အသက်: ၂၀၊ အတန်း: ၀) ဖြစ်ပါစေ။
ပွားကြည့် ဟူသော လုပ်ဆောင်ချက်ကို ကျော် ဖြင့် လုပ်ဆောင်မည်။
ကျော် ၏ အသက် ကို ပြသမည်။
အသက်ကြီး ဟူသော လုပ်ဆောင်ချက်ကို ကျော် ဖြင့် လုပ်ဆောင်မည်။
ကျော် ၏ အသက် ကို ပြသမည်။
```

```text
လူ(အသက်: ၉၉၊ အတန်း: ၀)
၂၀
၂၁
```

Part III shows another method, where we pass the address of the struct instead: see
[Lists and structs through addresses](../pointers/structs.md).

## A function that builds a struct

A function can return a struct, for example one that it writes as a struct literal, and
the caller then gets a duplicate, the same as with `ထားမည်`.

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် အတန်း ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။
ကျောင်းသား ဟူသော လုပ်ဆောင်ချက် သည်
နှစ် ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ လူ တစ်ယောက် ကို ပြန်ပေးမည်။
အသက် အနေဖြင့် နှစ် နှင့် အတန်း အနေဖြင့် နှစ် - ၅ ရှိသော လူတစ်ယောက် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ကျော် သည် လူ တစ်ယောက် ဖြစ်၍ တန်ဖိုးသည် ကျောင်းသား ဟူသော လုပ်ဆောင်ချက်ကို ၁၂ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကျော် ကို ပြသမည်။
```

```text
လူ(အသက်: ၁၂၊ အတန်း: ၇)
```

## Another name for a kind

```text
ဒေတာအမျိုးအစား <new name> ကို <kind> ဟုလည်း မှတ်ယူနိုင်ပါသည်။
```

We can use a *kind alias* (type alias) to give a data kind a second name. The alias is not
a new data kind, so a value of the alias is a value of the struct, and it prints with the
struct name.

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် အတန်း ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။
ဒေတာအမျိုးအစား ကျောင်းသား ကို လူ ဟုလည်း မှတ်ယူနိုင်ပါသည်။
ဒေတာအမျိုးအစား အတန်းသားများ ကို ကျောင်းသား စာရင်း ဟုလည်း မှတ်ယူနိုင်ပါသည်။

ကိန်းရှင် ကျော် သည် ကျောင်းသား တစ်ယောက် ဖြစ်၍ တန်ဖိုးသည် အသက် အနေဖြင့် ၁၂ နှင့် အတန်း အနေဖြင့် ၇ ရှိသော ကျောင်းသားတစ်ယောက် ဖြစ်ပါစေ။
ကိန်းရှင် အခန်း သည် အတန်းသားများ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [ကျော်၊ လူ(အသက်: ၁၃၊ အတန်း: ကနဦးတန်ဖိုး)] ဖြစ်ပါစေ။
အခန်း ကို ပြသမည်။
```

```text
[လူ(အသက်: ၁၂၊ အတန်း: ၇)၊ လူ(အသက်: ၁၃၊ အတန်း: ၀)]
```

| Zaka | Reads as |
| --- | --- |
| `ဒေတာအမျိုးအစား ကျောင်းသား ကို လူ` | "the data kind `ကျောင်းသား` (student), as `လူ`" |
| `ဟုလည်း မှတ်ယူနိုင်ပါသည်` | "can also be taken as" |

We can use the alias wherever we can use the struct name: in a declaration, a parameter,
a struct literal or the data kind of a list, as `အတန်းသားများ` shows. The alias also uses
the measure word of the data kind behind it, which here is `ယောက်`.

## Details

These rules were left out of the sections above to keep them short, and each one links to
the page that has the full rule.

- Zaka treats `ဒေတာ` and `အချက်အလက်` as the same word in names and reserved words, so
  `အချက်အလက်အစုအဝေး` means the same as `ဒေတာအစုအဝေး`, and `ဒေတာလူ` and `အချက်အလက်လူ`
  name the same struct ([Names](../../manual/language/words.md#names)).
- When a struct has a single field, we end the declaration with `ပါဝင်သည်`, without
  `တို့`.
- A struct must be declared at the top level of a file, not inside a block or a function
  ([Struct mistakes](mistakes.md#a-struct-declared-inside-a-block)), and the rest of the
  program can use it even above the declaration ([Declaring a
  struct](../../manual/language/structs.md#declaring-a-struct)).
- A struct declared without a value counts as set once all of its fields are set, and
  reading the whole struct before that is the mistake ZK0722 ([Struct
  mistakes](mistakes.md#a-struct-read-before-each-field-is-set)).
- Leaving out a field in a struct literal is the mistake ZK0482, so we give each field a
  value, or `ကနဦးတန်ဖိုး` for its initial value ([Struct
  mistakes](mistakes.md#a-field-left-out)).
- When a field value inside a struct literal is itself a struct literal in the `ရှိသော`
  form, we put it in parentheses, which keeps its `နှင့်` and `ရှိသော` apart from the
  outer ones: `စ အနေဖြင့် (x အနေဖြင့် ၁ နှင့် y အနေဖြင့် ၂ ရှိသော အမှတ်တစ်ခု) နှင့် ...`
  ([Struct literals](../../manual/language/structs.md#struct-literals)).
- `<`, `>` and similar operators do not work on structs, so we compare the fields instead
  ([Ordering structs](mistakes.md#ordering-structs)).
- A struct cannot contain itself, because it would need to be larger than itself ([A
  struct that contains itself](mistakes.md#a-struct-that-contains-itself)), but Part III shows
  how a struct can point to another struct of the same data kind, in [A linked
  list](../pointers/structs.md#a-linked-list).
- The wrong measure word does not stop the program, but Zaka gives a warning ([The wrong
  measure word](mistakes.md#the-wrong-measure-word)). A list count or ordinal
  (`၁ ခုမြောက်အကွက်`) counts cells rather than structs, so it uses `ခု` for all data kinds
  ([Measure words](../../manual/language/structs.md#measure-words)).
- The full rules on how fields are laid out are in [How a struct is
  stored](../../manual/language/structs.md#how-a-struct-is-stored), and the rules for
  aliases are in [Kind aliases](../../manual/language/aliases.md).

## Recap

- We write `<name> ဟူသော ဒေတာအစုအဝေးတွင် … တို့ပါဝင်သည်။` to declare a struct, and its
  name is a data kind from then on.
- We write `<variable> ၏ <field>` to read or change one field.
- We can write out a whole struct as a struct literal, either with `ရှိသော` or in the
  short form `<name>(<field>: <value>)`.
- Assigning a struct or passing it to a function makes a duplicate, while a mutable
  parameter (`ဒေတာ ပြောင်းလဲရန်`) lets a function change the caller's struct.
- A struct is counted with `ခု`, or with the measure word that we give in its declaration.

## Hands on

- Declare a struct with three fields of three different data kinds, then print it before
  and after setting each field.
- Declare a struct that contains two structs, and set a field of the inner one.
- Write the struct out as a struct literal with one field missing and read the error, then
  give that field `ကနဦးတန်ဖိုး`.
- Print the size of a struct, then swap two of its fields in the declaration and print the
  size again.
- Write a function with a mutable parameter that changes one field of the caller's struct.

Next: [Struct mistakes](mistakes.md).
