---
title: Structs
description: Group a few values of different kinds under one name, and reach each one through its field name.
---

# Structs

A *struct* groups a fixed set of *fields* under one name. Each field has a name and a
kind of its own, and the fields sit next to each other in RAM. A list holds a large number of values
of one kind; a struct holds a few values of different kinds.

```zaka
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ဒဿမကိန်း အမြင့်၊ မှန်မှားတန်ဖိုး လုပ်သား တို့ ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်ပါစေ။
ကျော် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
လူ(အသက်: ၀၊ အမြင့်: ၀.၀၊ လုပ်သား: မမှန်)
```

| Zaka | Reads as |
| --- | --- |
| `ဒေတာအမျိုးအစား လူ` | "the data kind `လူ` (person)" |
| `တွင် ... တို့ ပါဝင်သည်` | "in it, these are included" |
| `ကိန်းပြည့် အသက်` | "an integer field, `အသက်` (age)" |
| `ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်ပါစေ` | "let variable `ကျော်` be a `လူ`" |

A declaration lists the fields as `<kind> <name>`, with `၊` between them. `အချက်အလက်` and
`ဒေတာ` are one word in each name and reserved word, so `အချက်အလက်အမျိုးအစား` and `ဒေတာအမျိုးအစား` mean the same,
and so do `ဒေတာလူ` and `အချက်အလက်လူ` as names.

A struct is declared at the top level of a file, never inside a block or a function. The
rest of the program can use it from the first line, even above the declaration. A struct
name is a kind from then on, and it works wherever a kind does.

A struct declared with no value starts with each field at its default: `၀` for a number,
`မမှန်` for a true/false value, as the output above shows.

## Reading and writing fields

`ကျော် ၏ အသက်` reads as "the age of `ကျော်`". It is a place: it can be read and assigned
to.

```zaka
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ဒဿမကိန်း အမြင့်၊ မှန်မှားတန်ဖိုး လုပ်သား တို့ ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်ပါစေ။
ကျော် ၏ အသက် ကို ၂၀ ဟု ထားပါ။
ကျော် ၏ အမြင့် ကို ၁.၇၅ ဟု ထားပါ။
ကျော် ၏ အသက် ကို ပြသပါ။
(ကျော် ၏ အသက်) + ၁ ကို ပြသပါ။
ကျော် ကို ပြသပါ။
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

## Writing a struct out

A *struct literal* gives the fields of a struct at once: each field name with its value
after it, `၊` between the pairs, then `ရှိသော`, the struct name, `တစ်` ("one") and the
count word of the struct, `ခု` for now.

```zaka
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ဒဿမကိန်း အမြင့်၊ မှန်မှားတန်ဖိုး လုပ်သား တို့ ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အသက် ၂၀၊ အမြင့် ၁.၇၅ ရှိသော လူတစ်ခု ဖြစ်ပါစေ။
ကျော် ကို ပြသပါ။
ကျော် ကို လုပ်သား မှန်၊ အသက် ၃၀ ရှိသော လူ တစ်ခု ဟု ထားပါ။
ကျော် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
လူ(အသက်: ၂၀၊ အမြင့်: ၁.၇၅၊ လုပ်သား: မမှန်)
လူ(အသက်: ၃၀၊ အမြင့်: ၀.၀၊ လုပ်သား: မှန်)
```

| Zaka | Reads as |
| --- | --- |
| `အသက် ၂၀၊ အမြင့် ၁.၇၅` | "age 20, height 1.75" |
| `ရှိသော` | "that has" |
| `လူတစ်ခု` | "one person" |

The fields can come in another order than in the declaration, and a field left out holds
its default, as `လုပ်သား` does in the first line of the output. `လူတစ်ခု` can also be
written with a space, as `လူ တစ်ခု`. [Count words](#count-words) shows how a struct names
a count word of its own.

A struct literal stands wherever a struct value can: the value of a declaration, the
right side of `ထားပါ`, an argument, or `ပြသပါ`.

A struct literal can also take the form a struct prints in: the struct name, then each
field as `<field>: <value>` in parentheses, with `၊` between the pairs. This form needs no
count word, and it nests inside a bracket list as it stands. With no pairs, `လူ()` is a
struct with each field at its default.

```zaka
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ဒဿမကိန်း အမြင့်၊ မှန်မှားတန်ဖိုး လုပ်သား တို့ ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် လူ(အသက်: ၂၀၊ အမြင့်: ၁.၇၅) ဖြစ်ပါစေ။
ကျော် ကို ပြသပါ။
[လူ(အသက်: ၁)၊ လူ(လုပ်သား: မှန်)] ကို ပြသပါ။
လူ() ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
လူ(အသက်: ၂၀၊ အမြင့်: ၁.၇၅၊ လုပ်သား: မမှန်)
[လူ(အသက်: ၁၊ အမြင့်: ၀.၀၊ လုပ်သား: မမှန်)၊ လူ(အသက်: ၀၊ အမြင့်: ၀.၀၊ လုပ်သား: မှန်)]
လူ(အသက်: ၀၊ အမြင့်: ၀.၀၊ လုပ်သား: မမှန်)
```

## Duplicating a struct

Setting one struct to another duplicates each field, and the two stand apart afterwards.
The same holds for a struct given as a function argument.

```zaka
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ဒဿမကိန်း အမြင့် တို့ ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်ပါစေ။
ကျော် ၏ အသက် ကို ၂၀ ဟု ထားပါ။
ကိန်းရှင် မွန် သည် လူ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကျော် ဖြစ်ပါစေ။
မွန် ၏ အသက် ကို ၃၀ ဟု ထားပါ။
ကျော် ကို ပြသပါ။
မွန် ကို ပြသပါ။
မွန် ကို ကျော် ဟု ထားပါ။
မွန် ကို ပြသပါ။
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
ဒေတာအမျိုးအစား လူ တွင် စာသား အမည်၊ ကိန်းပြည့် အသက် တို့ ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ(အမည်: "ကျော်"၊ အသက်: ၂၀) ဖြစ်ပါစေ။
ကိန်းရှင် မွန် သည် ကျော် ဖြစ်ပါစေ။
ကျော် == မွန် ကို ပြသပါ။
မွန် ၏ အသက် ကို ၃၀ ဟု ထားပါ။
ကျော် == မွန် ကို ပြသပါ။
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
ဒေတာအမျိုးအစား အမှတ် တွင် ကိန်းပြည့် x၊ ကိန်းပြည့် y တို့ ပါဝင်သည်။
ဒေတာအမျိုးအစား မျဉ်း တွင် အမှတ် စ၊ အမှတ် ဆုံး တို့ ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် မျဉ်း တစ်ခု ဖြစ်ပါစေ။
က ၏ ဆုံး ၏ x ကို ၃ ဟု ထားပါ။
က ကို ပြသပါ။
ကိန်းရှင် အမှတ်များ သည် အမှတ် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [အမှတ်()၊ အမှတ်()၊ အမှတ်()] ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက် ၁ ၏ y ကို ၇ ဟု ထားပါ။
အမှတ်များ ကို ပြသပါ။
အမှတ်များ ၏ အကွက် ၁ ကို ပြသပါ။
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
ဒေတာအမျိုးအစား အမှတ် တွင် ကိန်းပြည့် x၊ ကိန်းပြည့် y တို့ ပါဝင်သည်။
ဒေတာအမျိုးအစား မျဉ်း တွင် အမှတ် စ၊ အမှတ် ဆုံး တို့ ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် မျဉ်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် စ (x ၁၊ y ၂ ရှိသော အမှတ်တစ်ခု)၊ ဆုံး (x ၄ ရှိသော အမှတ်တစ်ခု) ရှိသော မျဉ်းတစ်ခု ဖြစ်ပါစေ။
က ကို ပြသပါ။
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
ဒေတာအမျိုးအစား ယ တွင် မှန်မှားတန်ဖိုး အ၊ ကိန်းပြည့် ဘ၊ မှန်မှားတန်ဖိုး စ တို့ ပါဝင်သည်။
ဒေတာအမျိုးအစား ရ တွင် ကိန်းပြည့် ဘ၊ မှန်မှားတန်ဖိုး အ၊ မှန်မှားတန်ဖိုး စ တို့ ပါဝင်သည်။
ဒေတာအမျိုးအစား လ တွင် ၃၂ ဘစ် ကိန်းပြည့် ဘ၊ မှန်မှားတန်ဖိုး အ တို့ ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ယ ၏ အရွယ်အစား ကို ပြသပါ။
ရ ၏ အရွယ်အစား ကို ပြသပါ။
လ ၏ အရွယ်အစား ကို ပြသပါ။
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
classifier at its end, with `တို့ ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်`
("these are included, and the count word is `ယောက်`"). The word then follows `တစ်`, in a
declaration and in a struct literal:

```zaka
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ကိန်းပြည့် အတန်း တို့ ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ယောက် ဖြစ်ပါစေ။
ကိန်းရှင် မွန် သည် လူ တစ်ယောက် ဖြစ်၍ တန်ဖိုးသည် အသက် ၁၀ ရှိသော လူတစ်ယောက် ဖြစ်ပါစေ။
ကျော် ကို ပြသပါ။
မွန် ကို ပြသပါ။
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
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ကိန်းပြည့် အတန်း တို့ ပါဝင်သည်။
လုပ်ဆောင်ချက် ပွားကြည့် သည်
လူ သူ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
သူ ၏ အသက် ကို ၉၉ ဟု ထားပါ။
သူ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် အသက်ကြီး သည်
လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သော လူ သူ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
သူ ၏ အသက် ကို (သူ ၏ အသက်) + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်ပါစေ။
ကျော် ၏ အသက် ကို ၂၀ ဟု ထားပါ။
လုပ်ဆောင်ချက် ပွားကြည့် ကို ကျော် တို့ဖြင့် လုပ်ဆောင်ပါ။
ကျော် ၏ အသက် ကို ပြသပါ။
လုပ်ဆောင်ချက် အသက်ကြီး ကို ကျော် (လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်) တို့ဖြင့် လုပ်ဆောင်ပါ။
ကျော် ၏ အသက် ကို ပြသပါ။
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
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ကိန်းပြည့် အတန်း တို့ ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။
လုပ်ဆောင်ချက် ကျောင်းသား သည်
ကိန်းပြည့် နှစ် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ လူ ရလဒ် ကို ပြန်ပေးမည်။
အသက် နှစ်၊ အတန်း နှစ် - ၅ ရှိသော လူတစ်ယောက် ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ယောက် ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် ကျောင်းသား ကို ၁၂ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကျော် ကို ပြသပါ။
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
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ကိန်းပြည့် အတန်း တို့ ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။
ဒေတာအမျိုးအစား လူ ကို ကျောင်းသား ဟုလည်း မှတ်ယူနိုင်ပါသည်။
ဒေတာအမျိုးအစား ကျောင်းသား စာရင်း ကို အတန်းသားများ ဟုလည်း မှတ်ယူနိုင်ပါသည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် ကျောင်းသား တစ်ယောက် ဖြစ်၍ တန်ဖိုးသည် အသက် ၁၂၊ အတန်း ၇ ရှိသော ကျောင်းသားတစ်ယောက် ဖြစ်ပါစေ။
ကိန်းရှင် အခန်း သည် အတန်းသားများ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [ကျော်၊ လူ(အသက်: ၁၃)] ဖြစ်ပါစေ။
အခန်း ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[လူ(အသက်: ၁၂၊ အတန်း: ၇)၊ လူ(အသက်: ၁၃၊ အတန်း: ၀)]
```

| Zaka | Reads as |
| --- | --- |
| `ဒေတာအမျိုးအစား လူ ကို ကျောင်းသား` | "the data kind `လူ`, as `ကျောင်းသား`" |
| `ဟုလည်း မှတ်ယူနိုင်ပါသည်` | "can also be taken as" |

The alias stands wherever the struct name does: in a declaration, a parameter, a struct
literal and a list of the struct, as `အတန်းသားများ` shows. The kind after the opener can
be each kind, not a struct alone: `ဒေတာအမျိုးအစား ၃၂ ဘစ် ကိန်းပြည့် ကို အမှတ် ဟုလည်း
မှတ်ယူနိုင်ပါသည်။` names a kind of number. The rules in full are in
[Kind aliases](../../manual/language/aliases.md).

## What a struct cannot do

- `<`, `>` and the like do not take structs: compare the fields. See
  [Ordering structs](mistakes.md#ordering-structs).
- A struct cannot hold itself: see [Struct mistakes](mistakes.md#a-struct-that-holds-itself).

## Hands on

- Declare a struct with three fields of three kinds, and print it before and after setting
  each field.
- Declare a struct of two structs, and set a field of the inner one.
- Write the same struct out with a struct literal, leaving one field out, and print it.
- Print the size of a struct, then swap two of its fields in the declaration and print the
  size again.
- Write a function that can change a struct, and sets one of its fields.

Next: [Struct mistakes](mistakes.md).
