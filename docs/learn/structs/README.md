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
ကိန်းရှင် ကျော် သည် လူ ဖြစ်ပါစေ။
ကျော် ကို ပြသပါ။
```

```text
လူ(အသက်: ၀၊ အမြင့်: ၀.၀၊ လုပ်သား: မမှန်)
```

| Zaka | Reads as |
| --- | --- |
| `ဒေတာအမျိုးအစား လူ` | "the data kind `လူ` (person)" |
| `တွင် ... တို့ ပါဝင်သည်` | "in it, these are included" |
| `ကိန်းပြည့် အသက်` | "an integer field, `အသက်` (age)" |
| `ကိန်းရှင် ကျော် သည် လူ ဖြစ်ပါစေ` | "let variable `ကျော်` be a `လူ`" |

A declaration lists the fields as `<kind> <name>`, with `၊` between them. `အချက်အလက်အမျိုးအစား`
is a second spelling of `ဒေတာအမျိုးအစား`; the two mean the same.

A struct is declared at the top level of a file, never inside a block or a function. The
rest of the program can use it from the first line, even above the declaration. A struct
name is a kind from then on, and it works wherever a kind does.

A struct declared with no value starts with each field at its default: `၀` for a number,
`မမှန်` for a true/false value, `ဗလာ` for an address, as the output above shows.

## Reading and writing fields

`ကျော် ၏ အသက်` reads as "the age of `ကျော်`". It is a place: it can be read, assigned to,
and have its address taken.

```zaka
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ဒဿမကိန်း အမြင့်၊ မှန်မှားတန်ဖိုး လုပ်သား တို့ ပါဝင်သည်။
ကိန်းရှင် ကျော် သည် လူ ဖြစ်ပါစေ။
ကျော် ၏ အသက် ကို ၂၀ ဟု ထားပါ။
ကျော် ၏ အမြင့် ကို ၁.၇၅ ဟု ထားပါ။
ကျော် ၏ အသက် ကို ပြသပါ။
(ကျော် ၏ အသက်) + ၁ ကို ပြသပါ။
ကျော် ကို ပြသပါ။
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

The address of a field is a pointer to a value of the field's kind, so it can be lent
or written through:

```zaka
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ဒဿမကိန်း အမြင့် တို့ ပါဝင်သည်။
ကိန်းရှင် ကျော် သည် လူ ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ကျော် ၏ အသက် ၏ လိပ်စာ ဖြစ်ပါစေ။
ပ ၏ တန်ဖိုး ကို ၂၅ ဟု ထားပါ။
ကျော် ၏ အသက် ကို ပြသပါ။
```

```text
၂၅
```

## Writing a struct out

A *struct literal* gives the fields of a struct at once: each field name with its value
after it, `၊` between the pairs, then `ရှိသော`, the struct name, `တစ်` ("one") and the
count word of the struct, `ခု` for now.

```zaka
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ဒဿမကိန်း အမြင့်၊ မှန်မှားတန်ဖိုး လုပ်သား တို့ ပါဝင်သည်။
ကိန်းရှင် ကျော် သည် လူ ဖြစ်၍ တန်ဖိုးသည် အသက် ၂၀၊ အမြင့် ၁.၇၅ ရှိသော လူတစ်ခု ဖြစ်ပါစေ။
ကျော် ကို ပြသပါ။
ကျော် ကို လုပ်သား မှန်၊ အသက် ၃၀ ရှိသော လူ တစ်ခု ဟု ထားပါ။
ကျော် ကို ပြသပါ။
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
written with a space, as `လူ တစ်ခု`. [Pointers and the heap](#pointers-and-the-heap)
shows how a struct names a count word of its own.

A struct literal stands wherever a struct value can: the value of a declaration, the
right side of `ထားပါ`, an argument, or `ပြသပါ`.

## Duplicating a struct

Setting one struct to another duplicates each field, and the two stand apart afterwards.
The same holds for a struct given as a function argument.

```zaka
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ဒဿမကိန်း အမြင့် တို့ ပါဝင်သည်။
ကိန်းရှင် ကျော် သည် လူ ဖြစ်ပါစေ။
ကျော် ၏ အသက် ကို ၂၀ ဟု ထားပါ။
ကိန်းရှင် မွန် သည် လူ ဖြစ်၍ တန်ဖိုးသည် ကျော် ဖြစ်ပါစေ။
မွန် ၏ အသက် ကို ၃၀ ဟု ထားပါ။
ကျော် ကို ပြသပါ။
မွန် ကို ပြသပါ။
မွန် ကို ကျော် ဟု ထားပါ။
မွန် ကို ပြသပါ။
```

```text
လူ(အသက်: ၂၀၊ အမြင့်: ၀.၀)
လူ(အသက်: ၃၀၊ အမြင့်: ၀.၀)
လူ(အသက်: ၂၀၊ အမြင့်: ၀.၀)
```

A struct that holds an owner address moves instead of being duplicated, as an owner
does: see [A linked list](#a-linked-list), and [Ownership and
borrowing](../pointers/ownership.md#one-owner-each).

## Structs in structs and lists

A field can hold another struct, and a list cell can hold a struct. A field of a field is
reached with a second `၏`, and a field of a cell with the cell first:

```zaka
ဒေတာအမျိုးအစား အမှတ် တွင် ကိန်းပြည့် x၊ ကိန်းပြည့် y တို့ ပါဝင်သည်။
ဒေတာအမျိုးအစား မျဉ်း တွင် အမှတ် စ၊ အမှတ် ဆုံး တို့ ပါဝင်သည်။
ကိန်းရှင် က သည် မျဉ်း ဖြစ်ပါစေ။
က ၏ ဆုံး ၏ x ကို ၃ ဟု ထားပါ။
က ကို ပြသပါ။
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော အမှတ် စာရင်း ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက် ၁ ၏ y ကို ၇ ဟု ထားပါ။
အမှတ်များ ကို ပြသပါ။
အမှတ်များ ၏ အကွက် ၁ ကို ပြသပါ။
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
ကိန်းရှင် က သည် မျဉ်း ဖြစ်၍ တန်ဖိုးသည် စ (x ၁၊ y ၂ ရှိသော အမှတ်တစ်ခု)၊ ဆုံး (x ၄ ရှိသော အမှတ်တစ်ခု) ရှိသော မျဉ်းတစ်ခု ဖြစ်ပါစေ။
က ကို ပြသပါ။
```

```text
မျဉ်း(စ: အမှတ်(x: ၁၊ y: ၂)၊ ဆုံး: အမှတ်(x: ၄၊ y: ၀))
```

A field can hold a list too, and a struct cannot hold itself, because it would need to be
larger than itself. A struct holds the address of its own kind instead; see [A linked
list](#a-linked-list).

## How big a struct is

Fields are stored in the order written. Each one starts at a multiple of its own
alignment, and the whole struct is rounded up to the largest alignment among its fields,
as in C. `အရွယ်အစား` gives the size, as for each kind, and the order of the fields
can change it:

```zaka
ဒေတာအမျိုးအစား ယ တွင် မှန်မှားတန်ဖိုး အ၊ ကိန်းပြည့် ဘ၊ မှန်မှားတန်ဖိုး စ တို့ ပါဝင်သည်။
ဒေတာအမျိုးအစား ရ တွင် ကိန်းပြည့် ဘ၊ မှန်မှားတန်ဖိုး အ၊ မှန်မှားတန်ဖိုး စ တို့ ပါဝင်သည်။
ဒေတာအမျိုးအစား လ တွင် ၃၂ ဘစ် ကိန်းပြည့် ဘ၊ မှန်မှားတန်ဖိုး အ တို့ ပါဝင်သည်။
ယ ၏ အရွယ်အစား ကို ပြသပါ။
ရ ၏ အရွယ်အစား ကို ပြသပါ။
လ ၏ အရွယ်အစား ကို ပြသပါ။
```

```text
၂၄
၁၆
၈
```

In the first struct, a one-octet true/false value comes first, then 7 octets of padding, so
that the integer starts at a multiple of 8, and another 7 octets follow the last field. In
the second, the two true/false values share the padding that is left.

## Pointers and the heap

A struct name goes in front of `လိပ်စာ` to make a pointer kind, and in front of a count
to ask the heap for room. Through a pointer, a field is reached with `တန်ဖိုး` first:
`p ၏ တန်ဖိုး ၏ အသက်`.

A struct counts with a *classifier*, a word that goes with what is counted, as in "three
people". Without more, a struct is counted with `ခု`. A declaration can name another
classifier at its end, with `တို့ ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်`
("these are included, and the count word is `ယောက်`"). The heap size then uses it:

```zaka
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ကိန်းပြည့် အတန်း တို့ ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။
ကိန်းရှင် p သည် လူ လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် လူ ၃ ယောက်စာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။
p ၏ တန်ဖိုး ၏ အသက် ကို ၁၀ ဟု ထားပါ။
(p + ၁) ၏ တန်ဖိုး ၏ အသက် ကို ၁၁ ဟု ထားပါ။
p ၏ တန်ဖိုး ကို ပြသပါ။
(p + ၁) ၏ တန်ဖိုး ကို ပြသပါ။
လူ ၏ အရွယ်အစား ကို ပြသပါ။
လိပ်စာ p ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
လူ(အသက်: ၁၀၊ အတန်း: ၀)
လူ(အသက်: ၁၁၊ အတန်း: ၀)
၁၆
```

The same word follows `တစ်` in a struct literal: `အသက် ၁၀ ရှိသော လူတစ်ယောက်`.

Adding 1 to the address of a struct moves one whole struct, here 16 octets. A struct with
no classifier line is counted with `ခု`, like each other kind. Another word is not a
mistake that stops the program, but Zaka warns about it; see [Struct
mistakes](mistakes.md#the-wrong-classifier). A list count (`အကွက် ၃ ခုရှိသော`) and an
ordinal (`၁ ခုမြောက်အကွက်`) count cells, not structs, so these use `ခု` each time.

## Passing to a function

A struct parameter is a new variable that holds a duplicate, so the function changes its
own struct and the caller's is left as it was. To let a function change the caller's
struct, pass a borrowed address: the parameter kind is `ငှားထားသော လူ လိပ်စာ`, and the
call gives `ကျော် ၏ လိပ်စာ`.

```zaka
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ကိန်းပြည့် အတန်း တို့ ပါဝင်သည်။
လုပ်ဆောင်ချက် ပွားကြည့် သည်
လူ သူ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
သူ ၏ အသက် ကို ၉၉ ဟု ထားပါ။
သူ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်ချက် အသက်ကြီး သည်
ငှားထားသော လူ လိပ်စာ သူ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
သူ ၏ တန်ဖိုး ၏ အသက် ကို (သူ ၏ တန်ဖိုး ၏ အသက်) + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
ကိန်းရှင် ကျော် သည် လူ ဖြစ်ပါစေ။
ကျော် ၏ အသက် ကို ၂၀ ဟု ထားပါ။
လုပ်ဆောင်ချက် ပွားကြည့် ကို ကျော် တို့ဖြင့် လုပ်ဆောင်ပါ။
ကျော် ၏ အသက် ကို ပြသပါ။
လုပ်ဆောင်ချက် အသက်ကြီး ကို ကျော် ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်ပါ။
ကျော် ၏ အသက် ကို ပြသပါ။
```

```text
လူ(အသက်: ၉၉၊ အတန်း: ၀)
၂၀
၂၁
```

## A function that builds a struct

A function can give back a struct, such as one it writes out as a literal:

```zaka
// a function that builds a struct and gives it back
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ကိန်းပြည့် အတန်း တို့ ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။
လုပ်ဆောင်ချက် ကျောင်းသား သည်
ကိန်းပြည့် နှစ် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ လူ ရလဒ် ကို ပြန်ပေးမည်။
အသက် နှစ်၊ အတန်း နှစ် - ၅ ရှိသော လူတစ်ယောက် ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် ကျော် သည် လူ ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် ကျောင်းသား ကို ၁၂ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကျော် ကို ပြသပါ။
```

```text
လူ(အသက်: ၁၂၊ အတန်း: ၇)
```

The caller gets the struct as with an assignment: duplicated, or moved when it holds an
owner, as the next sections show.

## A linked list

A struct can hold the address of its own kind. Such an address is an owner, so each node
owns the next one, and a chain of nodes forms a linked list. The `နောက်` ("next") field
ends in `ဗလာ` at the last node.

```zaka
ဒေတာအမျိုးအစား အဆစ် တွင် ကိန်းပြည့် တန်ဖိုးရင်း၊ အဆစ် လိပ်စာ နောက် တို့ ပါဝင်သည်။

ကိန်းရှင် ထိပ် သည် အဆစ် လိပ်စာ ဖြစ်ပါစေ။
၁ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် အသစ် သည် အဆစ် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် အဆစ် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို အသစ် တွင် သိမ်းထားမည်။
အသစ် ၏ တန်ဖိုး ၏ တန်ဖိုးရင်း ကို န * န ဟု ထားပါ။
အသစ် ၏ တန်ဖိုး ၏ နောက် ကို ထိပ် ဟု ထားပါ။    // the old head moves into the new node
ထိပ် ကို အသစ် ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

// walk the list with a borrowed address
ကိန်းရှင် လက်ရှိ သည် ငှားထားသော အဆစ် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ထိပ် ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
လက်ရှိ သည် ဗလာ နှင့် မညီနေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
လက်ရှိ ၏ တန်ဖိုး ၏ တန်ဖိုးရင်း ကို ပြသပါ။
လက်ရှိ ကို လက်ရှိ ၏ တန်ဖိုး ၏ နောက် ၏ တန်ဖိုးအငှား ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

// give back each node
ထိပ် သည် ဗလာ နှင့် မညီနေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် ကျန် သည် အဆစ် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ထိပ် ၏ တန်ဖိုး ၏ နောက် ဖြစ်ပါစေ။
လိပ်စာ ထိပ် ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ထိပ် ကို ကျန် ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
```

```text
၉
၄
၁
```

Setting a struct that holds an owner moves it, as setting an owner does: the new struct
takes the address, and the old one is left with `ဗလာ` in that field and cannot be used
again. See [Ownership and borrowing](../pointers/ownership.md) for the rules of owners and
borrows.

## What a struct cannot do

- `==` and `!=` do not take structs: compare the fields.
- A field cannot hold a borrow (`ငှားထားသော ... လိပ်စာ`), just an owner or a raw address.

## Hands on

- Declare a struct with three fields of three kinds, and print it before and after setting
  each field.
- Declare a struct of two structs, and set a field of the inner one.
- Write the same struct out with a struct literal, leaving one field out, and print it.
- Print the size of a struct, then swap two of its fields in the declaration and print the
  size again.
- Add a function to the linked list that sums the `တန်ဖိုးရင်း` of each node.

Next: [Struct mistakes](mistakes.md).
