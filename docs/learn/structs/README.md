---
title: Structs
description: Group a few values of different kinds under one name, and reach each one through its field name.
---

# Structs

A *struct* groups a fixed set of *fields* under one name. Each field has a name and a
kind of its own, and the fields sit next to each other in RAM. A list holds a large number of values
of one kind; a struct holds a few values of different kinds.

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု၊ အမြင့် ဟူသော ဒဿမကိန်း တစ်ခု၊ လုပ်သား ဟူသော မှန်မှားတန်ဖိုး တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကျော် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
လူ(အသက်: ၀၊ အမြင့်: ၀.၀၊ လုပ်သား: မမှန်)
```

| Zaka | Reads as |
| --- | --- |
| `လူ ဟူသော ဒေတာအစုအဝေး` | "the data group called `လူ` (person)" |
| `တွင် ... တို့ပါဝင်သည်` | "in it, these are included" (one field: `ပါဝင်သည်`) |
| `အသက် ဟူသော ကိန်းပြည့် တစ်ခု` | "an integer called `အသက်` (age)" |
| `ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ` | "let variable `ကျော်` be one `လူ`, its value the initial value" |

A declaration lists the fields as `<name> ဟူသော <kind> တစ်<word>`, with `၊` between them:
each field counts its kind with `တစ်` and a measure word, `ခု` for most kinds (see
[Count words](#count-words)). `အချက်အလက်` and `ဒေတာ` are one word in each name and
reserved word, so `အချက်အလက်အစုအဝေး` and `ဒေတာအစုအဝေး` mean the same,
and so do `ဒေတာလူ` and `အချက်အလက်လူ` as names.

A struct is declared at the top level of a file, never inside a block or a function. The
rest of the program can use it from the first line, even above the declaration. A struct
name is a kind from then on, and it works wherever a kind does.

`ကနဦးတန်ဖိုး` gives a `လူ` with each field at its own [initial
value](../variables/README.md#the-initial-value): `၀` for a number, `မမှန်` for a
true/false value, as the output above shows.

## Reading and writing fields

`ကျော် ၏ အသက်` reads as "the age of `ကျော်`". It is a place: it can be read and assigned
to.

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု၊ အမြင့် ဟူသော ဒဿမကိန်း တစ်ခု၊ လုပ်သား ဟူသော မှန်မှားတန်ဖိုး တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်ပါစေ။
ကျော် ၏ အသက် ကို ၂၀ ဟု ထားမည်။
ကျော် ၏ အမြင့် ကို ၁.၇၅ ဟု ထားမည်။
ကျော် ၏ လုပ်သား ကို မမှန် ဟု ထားမည်။
ကျော် ၏ အသက် ကို ပြသမည်။
(ကျော် ၏ အသက်) + ၁ ကို ပြသမည်။
ကျော် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၀
၂၁
လူ(အသက်: ၂၀၊ အမြင့်: ၁.၇၅၊ လုပ်သား: မမှန်)
```

As with a list cell, a field inside an operator expression goes in parentheses:
`(ကျော် ၏ အသက်) + ၁`.

Printing a whole struct shows the struct name, then each field as `name: value` with `၊`
between them, in parentheses.

`ကျော်` is declared with no value here. A struct counts as set once each of its fields
is, so the three writes come before the first read. With one write left out, the read of
`ကျော်` is a mistake, ZK0722: see [Struct
mistakes](mistakes.md#a-struct-read-before-each-field-is-set).

## Writing a struct out

A *struct literal* gives the fields of a struct at once: each field name with its value
after it, `၊` between the pairs, then `ရှိသော`, the struct name, `တစ်` ("one") and the
count word of the struct, `ခု` for now.

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု၊ အမြင့် ဟူသော ဒဿမကိန်း တစ်ခု၊ လုပ်သား ဟူသော မှန်မှားတန်ဖိုး တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အသက် ၂၀၊ အမြင့် ၁.၇၅၊ လုပ်သား မမှန် ရှိသော လူတစ်ခု ဖြစ်ပါစေ။
ကျော် ကို ပြသမည်။
ကျော် ကို လုပ်သား မှန်၊ အသက် ၃၀၊ အမြင့် ကနဦးတန်ဖိုး ရှိသော လူ တစ်ခု ဟု ထားမည်။
ကျော် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
လူ(အသက်: ၂၀၊ အမြင့်: ၁.၇၅၊ လုပ်သား: မမှန်)
လူ(အသက်: ၃၀၊ အမြင့်: ၀.၀၊ လုပ်သား: မှန်)
```

| Zaka | Reads as |
| --- | --- |
| `အသက် ၂၀၊ အမြင့် ၁.၇၅၊ လုပ်သား မမှန်` | "age 20, height 1.75, worker false" |
| `ရှိသော` | "that has" |
| `လူတစ်ခု` | "one person" |

The fields can come in another order than in the declaration, and each field gets a
value: a field left out is a mistake, ZK0482 ([Struct
mistakes](mistakes.md#a-field-left-out)). `ကနဦးတန်ဖိုး` as the value gives a field its
initial value, as `အမြင့်` shows in the second line of the output. `လူတစ်ခု` can also be
written with a space, as `လူ တစ်ခု`. [Count words](#count-words) shows how a struct names
a count word of its own.

A struct literal stands wherever a struct value can: the value of a declaration, the
right side of `ထားမည်`, an argument, or `ပြသမည်`.

A struct literal can also take the form a struct prints in: the struct name, then each
field as `<field>: <value>` in parentheses, with `၊` between the pairs. This form needs no
count word, and it nests inside a bracket list as it stands. It names each field too. A
cell of a `လူ စာရင်း` can be `ကနဦးတန်ဖိုး` as well: the list kind tells which initial value
it is.

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု၊ အမြင့် ဟူသော ဒဿမကိန်း တစ်ခု၊ လုပ်သား ဟူသော မှန်မှားတန်ဖိုး တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် လူ(အသက်: ၂၀၊ အမြင့်: ၁.၇၅၊ လုပ်သား: မမှန်) ဖြစ်ပါစေ။
ကျော် ကို ပြသမည်။
ကိန်းရှင် လူများ သည် လူ စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [လူ(အသက်: ၁၊ အမြင့်: ကနဦးတန်ဖိုး၊ လုပ်သား: မှန်)၊ ကနဦးတန်ဖိုး] ဖြစ်ပါစေ။
လူများ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
လူ(အသက်: ၂၀၊ အမြင့်: ၁.၇၅၊ လုပ်သား: မမှန်)
[လူ(အသက်: ၁၊ အမြင့်: ၀.၀၊ လုပ်သား: မှန်)၊ လူ(အသက်: ၀၊ အမြင့်: ၀.၀၊ လုပ်သား: မမှန်)]
```

## Duplicating a struct

Setting one struct to another duplicates each field, and the two stand apart afterwards.
The same holds for a struct given as a function argument.

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု၊ အမြင့် ဟူသော ဒဿမကိန်း တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ(အသက်: ၂၀၊ အမြင့်: ကနဦးတန်ဖိုး) ဖြစ်ပါစေ။
ကိန်းရှင် မွန် သည် လူ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကျော် ဖြစ်ပါစေ။
မွန် ၏ အသက် ကို ၃၀ ဟု ထားမည်။
ကျော် ကို ပြသမည်။
မွန် ကို ပြသမည်။
မွန် ကို ကျော် ဟု ထားမည်။
မွန် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
လူ(အသက်: ၂၀၊ အမြင့်: ၀.၀)
လူ(အသက်: ၃၀၊ အမြင့်: ၀.၀)
လူ(အသက်: ၂၀၊ အမြင့်: ၀.၀)
```

## Comparing structs

`==` and `!=` compare two structs of one kind, one field at a time:

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု၊ အသက် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ(အမည်: "ကျော်"၊ အသက်: ၂၀) ဖြစ်ပါစေ။
ကိန်းရှင် မွန် သည် ကျော် ဖြစ်ပါစေ။
ကျော် == မွန် ကို ပြသမည်။
မွန် ၏ အသက် ကို ၃၀ ဟု ထားမည်။
ကျော် == မွန် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
မမှန်
```

## Structs in structs and lists

A field can hold another struct, and a list cell can hold a struct. A field of a field is
reached with a second `၏`, and a field of a cell with the cell first:

```zaka
အမှတ် ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု၊ y ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။
မျဉ်း ဟူသော ဒေတာအစုအဝေးတွင် စ ဟူသော အမှတ် တစ်ခု၊ ဆုံး ဟူသော အမှတ် တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် မျဉ်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
က ၏ ဆုံး ၏ x ကို ၃ ဟု ထားမည်။
က ကို ပြသမည်။
ကိန်းရှင် အမှတ်များ သည် အမှတ် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [အမှတ်(x: ၀၊ y: ၀)၊ အမှတ်(x: ၀၊ y: ၀)၊ အမှတ်(x: ၀၊ y: ၀)] ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက် ၁ ၏ y ကို ၇ ဟု ထားမည်။
အမှတ်များ ကို ပြသမည်။
အမှတ်များ ၏ အကွက် ၁ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မျဉ်း(စ: အမှတ်(x: ၀၊ y: ၀)၊ ဆုံး: အမှတ်(x: ၃၊ y: ၀))
[အမှတ်(x: ၀၊ y: ၀)၊ အမှတ်(x: ၀၊ y: ၇)၊ အမှတ်(x: ၀၊ y: ၀)]
အမှတ်(x: ၀၊ y: ၇)
```

In a struct literal, the value of a field that is itself a literal goes in parentheses, so
that its `၊` and `ရှိသော` stand apart from the outer ones:

```zaka
အမှတ် ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု၊ y ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။
မျဉ်း ဟူသော ဒေတာအစုအဝေးတွင် စ ဟူသော အမှတ် တစ်ခု၊ ဆုံး ဟူသော အမှတ် တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် မျဉ်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် စ (x ၁၊ y ၂ ရှိသော အမှတ်တစ်ခု)၊ ဆုံး (x ၄၊ y ၀ ရှိသော အမှတ်တစ်ခု) ရှိသော မျဉ်းတစ်ခု ဖြစ်ပါစေ။
က ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မျဉ်း(စ: အမှတ်(x: ၁၊ y: ၂)၊ ဆုံး: အမှတ်(x: ၄၊ y: ၀))
```

A field can hold a list too. A struct cannot hold itself, because it would need to be
larger than itself; Part III shows how a struct reaches another of its own kind, in [A
linked list](../pointers/structs.md#a-linked-list).

## How big a struct is

Fields are stored in the order written. Each one starts at a multiple of its own
alignment, and the whole struct is rounded up to the largest alignment among its fields,
as in C. `အရွယ်အစား` gives the size, as for each kind, and the order of the fields
can change it:

```zaka
ယ ဟူသော ဒေတာအစုအဝေးတွင် အ ဟူသော မှန်မှားတန်ဖိုး တစ်ခု၊ ဘ ဟူသော ကိန်းပြည့် တစ်ခု၊ စ ဟူသော မှန်မှားတန်ဖိုး တစ်ခု တို့ပါဝင်သည်။
ရ ဟူသော ဒေတာအစုအဝေးတွင် ဘ ဟူသော ကိန်းပြည့် တစ်ခု၊ အ ဟူသော မှန်မှားတန်ဖိုး တစ်ခု၊ စ ဟူသော မှန်မှားတန်ဖိုး တစ်ခု တို့ပါဝင်သည်။
လ ဟူသော ဒေတာအစုအဝေးတွင် ဘ ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု၊ အ ဟူသော မှန်မှားတန်ဖိုး တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ယ တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
ရ တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
လ တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၄
၁၆
၈
```

In the first struct, a one-octet true/false value comes first, then 7 octets of padding, so
that the integer starts at a multiple of 8, and another 7 octets follow the last field. In
the second, the two true/false values share the padding that is left.

## Count words

A struct counts with a *classifier*, a word that goes with what is counted, as in "three
people". Without more, a struct is counted with `ခု`. A declaration can name another
classifier at its end, with `တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်`
("these are included, and the count word is `ယောက်`"). The word then follows `တစ်` each
time the struct is written as a kind, in a declaration, a parameter or a field, and in a
struct literal:

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု၊ အတန်း ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ယောက် ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် မွန် သည် လူ တစ်ယောက် ဖြစ်၍ တန်ဖိုးသည် အသက် ၁၀၊ အတန်း ၀ ရှိသော လူတစ်ယောက် ဖြစ်ပါစေ။
ကျော် ကို ပြသမည်။
မွန် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
လူ(အသက်: ၀၊ အတန်း: ၀)
လူ(အသက်: ၁၀၊ အတန်း: ၀)
```

A struct with no classifier line is counted with `ခု`, like each other kind. Another word
is not a mistake that stops the program, but Zaka warns about it; see [Struct
mistakes](mistakes.md#the-wrong-classifier). A list ordinal (`၁ ခုမြောက်အကွက်`) counts
cells, not structs, so it uses `ခု` each time.

## Passing to a function

A struct parameter is a new variable that holds a duplicate, so the function changes its
own struct and the caller's is left as it was. To let a function change the caller's
struct, mark the parameter and the argument as [the caller's
variable](../functions/parameters.md#changing-the-callers-variable) is marked:

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု၊ အတန်း ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။
လုပ်ဆောင်ချက် ပွားကြည့် သည်
သူ ဟူသော လူ တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
သူ ၏ အသက် ကို ၉၉ ဟု ထားမည်။
သူ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် အသက်ကြီး သည်
သူ ဟူသော လူ တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
သူ ၏ အသက် ကို (သူ ၏ အသက်) + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ(အသက်: ၂၀၊ အတန်း: ၀) ဖြစ်ပါစေ။
ပွားကြည့် ဟူသော လုပ်ဆောင်ချက်ကို ကျော် တို့ဖြင့် လုပ်ဆောင်မည်။
ကျော် ၏ အသက် ကို ပြသမည်။
အသက်ကြီး ဟူသော လုပ်ဆောင်ချက်ကို ကျော် တို့ဖြင့် လုပ်ဆောင်မည်။
ကျော် ၏ အသက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
လူ(အသက်: ၉၉၊ အတန်း: ၀)
၂၀
၂၁
```

Part III passes the address of the struct instead: see [Lists and structs through
addresses](../pointers/structs.md).

## A function that builds a struct

A function can give back a struct, such as one it writes out as a literal:

```zaka
// a function that builds a struct and gives it back
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု၊ အတန်း ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။
လုပ်ဆောင်ချက် ကျောင်းသား သည်
နှစ် ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ လူ တစ်ယောက် ကို ပြန်ပေးမည်။
အသက် နှစ်၊ အတန်း နှစ် - ၅ ရှိသော လူတစ်ယောက် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ယောက် ဖြစ်၍ တန်ဖိုးသည် ကျောင်းသား ဟူသော လုပ်ဆောင်ချက်ကို ၁၂ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကျော် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
လူ(အသက်: ၁၂၊ အတန်း: ၇)
```

The caller gets the struct as with an assignment: a duplicate of its own.

## Another name for a kind

A *kind alias* gives a kind a second name, with `ဟုလည်း မှတ်ယူနိုင်ပါသည်` ("can also be
taken as"). The alias is no new kind: a value of the alias is a value of the struct, and
it prints with the struct name.

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု၊ အတန်း ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။
ဒေတာအမျိုးအစား ကျောင်းသား ကို လူ ဟုလည်း မှတ်ယူနိုင်ပါသည်။
ဒေတာအမျိုးအစား အတန်းသားများ ကို ကျောင်းသား စာရင်း ဟုလည်း မှတ်ယူနိုင်ပါသည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် ကျောင်းသား တစ်ယောက် ဖြစ်၍ တန်ဖိုးသည် အသက် ၁၂၊ အတန်း ၇ ရှိသော ကျောင်းသားတစ်ယောက် ဖြစ်ပါစေ။
ကိန်းရှင် အခန်း သည် အတန်းသားများ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [ကျော်၊ လူ(အသက်: ၁၃၊ အတန်း: ကနဦးတန်ဖိုး)] ဖြစ်ပါစေ။
အခန်း ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[လူ(အသက်: ၁၂၊ အတန်း: ၇)၊ လူ(အသက်: ၁၃၊ အတန်း: ၀)]
```

| Zaka | Reads as |
| --- | --- |
| `ဒေတာအမျိုးအစား ကျောင်းသား ကို လူ` | "the data kind `ကျောင်းသား`, as `လူ`" |
| `ဟုလည်း မှတ်ယူနိုင်ပါသည်` | "can also be taken as" |

The alias stands wherever the struct name does: in a declaration, a parameter, a struct
literal and a list of the struct, as `အတန်းသားများ` shows. The new name comes first, and the kind
after `ကို` can be each kind, not a struct alone: `ဒေတာအမျိုးအစား အမှတ် ကို ၃၂ ဘစ် ကိန်းပြည့်
ဟုလည်း မှတ်ယူနိုင်ပါသည်။` names a kind of number. The alias counts with the word of its kind,
here `ယောက်`, as `ကျောင်းသား တစ်ယောက်` shows. The rules in full are in
[Kind aliases](../../manual/language/aliases.md).

## What a struct cannot do

- `<`, `>` and the like do not take structs: compare the fields. See
  [Ordering structs](mistakes.md#ordering-structs).
- A struct cannot hold itself: see [Struct mistakes](mistakes.md#a-struct-that-holds-itself).

## Hands on

- Declare a struct with three fields of three kinds, and print it before and after setting
  each field.
- Declare a struct of two structs, and set a field of the inner one.
- Write the same struct out with a struct literal, leaving one field out, and read the
  mistake. Then give that field `ကနဦးတန်ဖိုး`.
- Print the size of a struct, then swap two of its fields in the declaration and print the
  size again.
- Write a function that can change a struct, and sets one of its fields.

Next: [Struct mistakes](mistakes.md).
