---
title: Structs
description: The struct declaration, its classifier, struct literals, field access, copies, how a struct is stored, and what a struct cannot do.
---

# Structs

A struct groups a fixed set of fields, each with its own name and kind, in one stretch of
RAM. It is a kind that a program declares for itself, and its name works wherever a kind
does. The Learn book starts in small steps with [Structs](../../learn/structs/README.md).

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် အမြင့် ဟူသော ဒဿမကိန်း တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကျော် ၏ အသက် ကို ၂၀ ဟု ထားမည်။
ကျော် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
လူ(အသက်: ၂၀၊ အမြင့်: ၀.၀)
```

## Declaring a struct

```text
<name> ဟူသော [အများသုံး] ဒေတာအစုအဝေးတွင် <field> ဟူသော <kind> တစ်<word>၊ … နှင့် <field> ဟူသော <kind> တစ်<word> တို့ပါဝင်သည်။
<name> ဟူသော [အများသုံး] ဒေတာအစုအဝေးတွင် <field> ဟူသော <kind> တစ်<word>၊ … နှင့် … တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ <word> ဖြစ်သည်။
<name> ဟူသော [အများသုံး] ဒေတာအစုအဝေးတွင် <field> ဟူသော <kind> တစ်<word> ပါဝင်သည်။
```

`မျှဝေနိုင်သော` before `ဒေတာအစုအဝေး`, as in `<name> ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် ...`, declares a shared object kind in place of a
struct: the same declaration, literal and fields, but assigning one shares it rather than
duplicating it. This page states the rules of both; [Values and objects](objects.md) states
what sharing changes.

- `အချက်အလက်အစုအဝေး` and `ဒေတာအစုအဝေး` are one word, as `အချက်အလက်` and `ဒေတာ` are
  in each name and reserved word ([Two spellings of one word](words.md#names)); a
  struct name written with either spelling names one struct.
- `တွင်` can stand joined to `ဒေတာအစုအဝေး` or apart from it; the pages write it joined.
- The export mark `အများသုံး` stands after `ဟူသော`, before `မျှဝေနိုင်သော` and the word
  `ဒေတာအစုအဝေး` ([Modules](modules.md)).
- An enumeration has the same shape, with `ဒေတာအမျိုးအစားကွဲ` in place of `ဒေတာအစုအဝေး`
  ([Enumerations](enums.md#declaring-an-enumeration)).
- A declaration stands at the top level of a file. Inside a block, a loop, a conditional
  or a function it is ZK0228. A struct can be used above its declaration, since the
  declarations are read first.
- A struct has at least one field. Each field is `<name> ဟူသော <kind> တစ်<word>`, its
  kind with the [measure word](#the-classifier) of that kind. The fields are a
  [list in words](words.md#lists-in-words), with `၊` between them and `နှင့်` before the
  last, and the list closes with `တို့ပါဝင်သည်` or `တို့ပါဝင်ပြီး`. A lone field closes with
  `ပါဝင်သည်` or `ပါဝင်ပြီး`, with no `တို့`; `တို့` after one field, or none after
  several, is ZK0201, and a list without its `နှင့်` is ZK0245.
- A field kind is each kind there is: a number, a true/false value, a character, text, a
  list, another struct, a shared object, a function value or a pointer. Safe code can hold
  a pointer field, while reading through it needs unsafe code ([Unsafe Zaka](unsafe.md)).
- A struct name is declared once in the whole program (ZK0316), and the field names of
  one struct differ from each other (ZK0317).
- A struct name cannot also name a variable, a constant, a parameter, a field or a
  function (ZK0319), because the compiler reads each struct name as a kind wherever a
  value can stand, as in `လူ တစ်ခု ၏ အရွယ်အစား`.
- A struct cannot hold itself, at once, through other structs or through a counted list
  of itself (ZK0427): it would have no end. A shared object kind can name itself in its
  fields, since such a field holds a reference, but the chain needs an end: one field of
  the loop is an optional or a plain list, as in `နောက် ဟူသော အဆစ် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု`.
  A loop with no such field is ZK0481 ([Kinds no program can build](objects.md#kinds-no-program-can-build)).

Where a kind must stand, as in a declaration, a parameter or a field, a name is read as a
struct name; a name that no declaration gives is ZK0318, as in
`ကိန်းရှင် က သည် လုပ် တစ်ခု ဖြစ်ပါစေ။`.

## The classifier

Burmese counts things with a word that fits them: people with `ယောက်`, most other things
with `ခု`. A struct names its own counting word, its *classifier*, at the end of its
declaration, with `တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ <word> ဖြစ်သည်`. A struct that names
none is counted with `ခု`, like each other kind.

The classifier is the word after `တစ်` in a [struct literal](#struct-literals),
`လူတစ်ယောက်`, and the measure word after each kind written in a declaration, a parameter,
a field, a result, a conversion, a size, a default arm or after a literal:
`ကိန်းရှင် ကျော် သည် လူ တစ်ယောက် ဖြစ်ပါစေ။` ([Kinds](kinds.md#measure-words)). In unsafe
code it is also the word in a heap size, `မင်မိုရီတွင် လူ ၃ ယောက်စာ နေရာယူ၍ ...`
([Unsafe Zaka](unsafe.md)):

| Kind | Counted with |
| --- | --- |
| a struct, an enumeration or an alias that names a classifier | that word |
| a struct or an enumeration that names none | `ခု` |
| an alias that names none | the word of its kind |
| `စာလုံး` | `လုံး` |
| each other kind, such as `ကိန်းပြည့်` or `လူ စာရင်း` | `ခု` |

A kind in one of those places with no measure word after it is ZK0234. A measure word
other than the kind's own, such as `ခု` for a struct that names `ယောက်`, or `ယောက်` for
`ကိန်းပြည့်`, is the warning ZK0509. The size does not change with the word,
and the program runs. The help line of the message gives the word to write. A classifier
word other than `ခု` is read in a heap size once a struct of the program names it:
`ကိန်းပြည့် ၃ ယောက်စာ` is the warning when some struct names `ယောက်`, and a shape
mistake (ZK0201) when none does.

List counts (`အကွက် ၃ ခုရှိသော`) and ordinals (`ခုမြောက်`) count cells, not structs, so
these use `ခု` for each kind.

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် အတန်း ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ယောက် ဖြစ်ပါစေ။
ကျော် ၏ အသက် ကို ၁၀ ဟု ထားမည်။
ကျော် ၏ အတန်း ကို ၅ ဟု ထားမည်။
ကျော် ကို ပြသမည်။
အသက် အနေဖြင့် ၁၂ နှင့် အတန်း အနေဖြင့် ၇ ရှိသော လူတစ်ယောက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
လူ(အသက်: ၁၀၊ အတန်း: ၅)
လူ(အသက်: ၁၂၊ အတန်း: ၇)
```

```zaka-warning ZK0509
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကျော် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0509]: လူ is counted with "ယောက်", not "ခု"
help: write "တစ်ယောက်"
```

## Where a struct kind goes

A struct name stands wherever a kind does:

| Place | Written |
| --- | --- |
| a variable | `ကိန်းရှင် ကျော် သည် လူ တစ်ယောက် ဖြစ်ပါစေ။` |
| a parameter | `သူ ဟူသော လူ တစ်ယောက် ကို လက်ခံပြီး` |
| the cells of a list | `အကွက် ၃ ခုရှိသော လူ စာရင်း တစ်ခု` |
| a field of another struct | `ပိုင်ရှင် ဟူသော လူ တစ်ယောက်` |
| a size | `လူ တစ်ယောက် ၏ အရွယ်အစား` |
| a pointer kind | `လူ လိပ်စာ တစ်ခု` |
| a heap size, in unsafe code | `မင်မိုရီတွင် လူ ၃ ယောက်စာ နေရာယူ၍ ...` |

A variable of a struct kind takes a struct literal, another struct of the same kind, or
`ကနဦးတန်ဖိုး`, the [initial value](kinds.md#initial-values), which sets each field to the
initial value of its own kind:
`ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။`. A declaration with no value leaves each field
unset, and the program sets each one before it reads the struct
([Set before read](statements.md#set-before-read)):

- The struct counts as set once each field is set on each path.
- A struct inside a struct is followed one field at a time too: writing `က ၏ ဆုံး ၏ x`
  sets that one field, and `က ၏ ဆုံး` counts as set once each of its fields is.
- Reading a field that can be unset, or the whole struct while one field can be, is
  ZK0722.

```zaka
အမှတ် ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု နှင့် y ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မ သည် အမှတ် တစ်ခု ဖြစ်ပါစေ။
မ ၏ x ကို ၃ ဟု ထားမည်။
မ ၏ y ကို ၄ ဟု ထားမည်။      // each field is set now
မ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အမှတ်(x: ၃၊ y: ၄)
```

```zaka-error ZK0722
အမှတ် ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု နှင့် y ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မ သည် အမှတ် တစ်ခု ဖြစ်ပါစေ။
မ ၏ x ကို ၃ ဟု ထားမည်။
မ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0722]: "မ" can be unset here: a path reaches this read with no value given to it
```

## Struct literals

```text
<field> အနေဖြင့် <value> နှင့် <field> အနေဖြင့် <value> ရှိသော <name>တစ်<word>
<field> အနေဖြင့် <value> နှင့် <field> အနေဖြင့် <value> ရှိသော <name> တစ်<word>
<name>(<field>: <value>၊ <field>: <value>)
```

A struct literal is a value of the struct `<name>`, with the fields it names set to the
values after them, as in `အသက် အနေဖြင့် ၂၀ နှင့် အမြင့် အနေဖြင့် ၁.၇၅ ရှိသော လူတစ်ယောက်`, "a person who has
age 20 and height 1.75". The last line is the notation form, the one a struct prints in:
`လူ(အသက်: ၂၀၊ အမြင့်: ၁.၇၅)` means the same, and writes no classifier.

- The pairs stand in each order. In the word form each field name takes `အနေဖြင့်` before
  its value, and the pairs are a [list in words](words.md#lists-in-words), with `၊` between
  them and `နှင့်` before the last; a bare field and value does not parse. In the notation
  form `၊` stands between the pairs. After a name that no struct declares, `()` is a
  [call](functions.md#calls) with no arguments.
- Each field of the struct has a pair: a field left out is ZK0482, as in `လူ(အသက်: ၁)`
  for a struct that also has `အမြင့်`. `ကနဦးတန်ဖိုး` as the value gives a field the
  initial value of its kind: `လူ(အသက်: ၁၊ အမြင့်: ကနဦးတန်ဖိုး)`.
- Each field name is one the struct has (ZK0320), and each is set once (ZK0321).
- Each value converts to the kind of its field, as the value of an assignment does
  (ZK0401).
- A value is a notation expression, such as `x + ၁`, `(က ၏ အသက်)`, `[၁၊ ၂]` or
  `လူ(အသက်: ၁)`. A clause, or a literal in the `ရှိသော` or `တို့ပါသော စာရင်း` form, goes
  in parentheses.
- `<name>` is a declared struct (ZK0318). `<word>` is its classifier, `ခု` when the
  declaration names none; another word is the warning ZK0509. The name and `တစ်` can be
  written as one word or with a space between them. In the notation form, the name and
  `(` stand together, and a `:` follows each field name.
- A struct literal stands wherever a value of the struct can: the value of a
  declaration, the value of an assignment, an argument, or the value of `ပြသမည်`.

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် အမြင့် ဟူသော ဒဿမကိန်း တစ်ခု တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ယောက် ဖြစ်၍ တန်ဖိုးသည် အမြင့် အနေဖြင့် ၁.၇၅ နှင့် အသက် အနေဖြင့် ၂၀ ရှိသော လူတစ်ယောက် ဖြစ်ပါစေ။
ကျော် ကို ပြသမည်။
အသက် အနေဖြင့် ၃၀ နှင့် အမြင့် အနေဖြင့် ကနဦးတန်ဖိုး ရှိသော လူ တစ်ယောက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
လူ(အသက်: ၂၀၊ အမြင့်: ၁.၇၅)
လူ(အသက်: ၃၀၊ အမြင့်: ၀.၀)
```

The compiler tells a struct literal from other values at its start: a field name with a
value right after it, before the next `ရှိသော`. Nowhere else in a value does a name stand
next to another value without an operator or a particle between them. The notation form
starts with a struct name and `(`, or with a name, `(`, a field name and `:`.

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် အမြင့် ဟူသော ဒဿမကိန်း တစ်ခု တို့ပါဝင်သည်။
အသင်း ဟူသော ဒေတာအစုအဝေးတွင် ခေါင်းဆောင် ဟူသော လူ တစ်ခု နှင့် အဖွဲ့ဝင် ဟူသော အကွက် ၂ ခုရှိသော လူ စာရင်း တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အသင်း(ခေါင်းဆောင်: လူ(အသက်: ၄၀၊ အမြင့်: ၁.၇)၊ အဖွဲ့ဝင်: [လူ(အသက်: ၂၀၊ အမြင့်: ကနဦးတန်ဖိုး)၊ ကနဦးတန်ဖိုး]) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အသင်း(ခေါင်းဆောင်: လူ(အသက်: ၄၀၊ အမြင့်: ၁.၇)၊ အဖွဲ့ဝင်: [လူ(အသက်: ၂၀၊ အမြင့်: ၀.၀)၊ လူ(အသက်: ၀၊ အမြင့်: ၀.၀)])
```

## Fields

`<value> ၏ <field>` is a field. It is a place: it can be read, printed, assigned to, and
passed to a `ဒေတာ ပြောင်းလဲရန်` parameter ([Functions](functions.md)). In unsafe code its
address can be taken.

| Written | Means |
| --- | --- |
| `ကျော် ၏ အသက်` | the field of a struct variable |
| `ကျော် ၏ အသက် ကို ၂၀ ဟု ထားမည်။` | write the field |
| `က ၏ ခ ၏ ဂ` | a field of a field, one `၏` for each step |
| `xs ၏ အကွက် ၀ ၏ x` | a field of a list cell |
| `p ၏ တန်ဖိုး ၏ အသက်` | in unsafe code, the field of the struct a pointer points at |
| `ကျော် ၏ အသက် ၏ လိပ်စာ` | in unsafe code, the address of the field, a pointer to the kind of the field |

- A field is looked up in the struct on its name. A name the struct does not have is
  ZK0320.
- The value in front of `၏` has to be a struct or a shared object (ZK0426). For a pointer, dereference it
  first with `တန်ဖိုး`: `p ၏ အသက်` is ZK0426.
- A field inside an operator expression goes in parentheses, as a list cell does:
  `(ကျော် ၏ အသက်) + ၁`. Without parentheses the sentence is ZK0209 or ZK0206.

```zaka
အမှတ် ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု နှင့် y ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။
မျဉ်း ဟူသော ဒေတာအစုအဝေးတွင် စ ဟူသော အမှတ် တစ်ခု နှင့် ဆုံး ဟူသော အမှတ် တစ်ခု တို့ပါဝင်သည်။

လုပ်ဆောင်ချက် ကိုး သည် န ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို ၉ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် မျဉ်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
က ၏ ဆုံး ၏ x ကို ၃ ဟု ထားမည်။
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော အမှတ် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက် ၁ ၏ y ကို ၇ ဟု ထားမည်။
ကိုး ဟူသော လုပ်ဆောင်ချက်ကို က ၏ စ ၏ y ဖြင့် လုပ်ဆောင်မည်။
က ကို ပြသမည်။
အမှတ်များ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မျဉ်း(စ: အမှတ်(x: ၀၊ y: ၉)၊ ဆုံး: အမှတ်(x: ၃၊ y: ၀))
[အမှတ်(x: ၀၊ y: ၀)၊ အမှတ်(x: ၀၊ y: ၇)၊ အမှတ်(x: ၀၊ y: ၀)]
```

## Comparing

`==` and `!=` take two structs of one kind. Two structs are equal when each field equals
the same field of the other: numbers and characters as values, text and lists one cell at
a time, nested structs one field at a time, shared objects in depth as `==` compares
them ([Values and objects](objects.md)), function values as the same function, and
pointers as the address each holds. The padding
between fields takes no part. A decimal field follows the rules of decimals, so `-0.0`
equals `0.0` and a `NaN` field makes the two unequal. A list of structs compares each
struct in turn.

```zaka
အမှတ် ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု နှင့် y ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။
လူ ဟူသော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု နှင့် နေရာ ဟူသော အမှတ် တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် လူ(အမည်: "မောင်"၊ နေရာ: အမှတ်(x: ၁၊ y: ၀)) ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။
က == ခ ကို ပြသမည်။
ခ ၏ နေရာ ၏ y ကို ၂ ဟု ထားမည်။
က != ခ ကို ပြသမည်။
[က၊ ခ] == [က၊ ခ] ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
မှန်
မှန်
```

Two structs of different kinds do not compare, even with the same fields (ZK0404), and the
ordering operators take no structs (ZK0403).

## Printing

A struct prints as its name, then `(`, each field as `<name>: <value>` with `၊` between
them, and `)`, as in `လူ(အသက်: ၂၀၊ အမြင့်: ၁.၇၅)`. A struct in a field and a list in a
field print alike, nested.

```zaka
ရမှတ်များ ဟူသော ဒေတာအစုအဝေးတွင် အကြိမ် ဟူသော လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု နှင့် ရမှတ် ဟူသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပွဲ သည် ရမှတ်များ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ပွဲ ၏ ရမှတ် ကို ၅၊ ၇ နှင့် ၉ တို့ပါသော စာရင်း ဟု ထားမည်။
ပွဲ ၏ အကြိမ် ကို ပွဲ ၏ ရမှတ် ၏ အကွက်အရေအတွက် ဟု ထားမည်။
ပွဲ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရမှတ်များ(အကြိမ်: ၃၊ ရမှတ်: [၅၊ ၇၊ ၉])
```

## What a struct cannot do

| Not allowed | Instead |
| --- | --- |
| `<`, `>` and the like on structs (ZK0403) | compare the fields |
| `==` on two structs of different kinds (ZK0404) | compare the fields |
| a struct with no fields | declare the field the struct needs |
| a struct declared inside a block (ZK0228) | declare it at the top level |
| a struct that holds itself (ZK0427) | declare the kind `မျှဝေနိုင်သော`, so the field holds a reference, and make the field optional |
| a struct literal that leaves out a field (ZK0482) | give each field a value, `ကနဦးတန်ဖိုး` for its initial value |
| reading a struct before each field is set (ZK0722) | set each field first, or declare the struct with a value |

## Copies

A struct is a value, like an integer or a list ([Values and objects](objects.md)).
Setting one struct to another of the same kind copies each field, and the two stand apart
afterwards: a later write to one never reaches the other. A text or list field shares its
cells with the old struct until one side writes. A field that holds a shared object, a
function value or a pointer copies the reference, so both structs reach the one object.

A struct passed as an argument is copied into the parameter alike, so a change to a
struct parameter is lost when the function ends. To let a function change the caller's
struct, mark the parameter `ဒေတာ ပြောင်းလဲရန်` ([Functions](functions.md)); the call names the
struct with no mark. A struct given back from a function follows the same rules; see
[Lists and structs as results](functions.md#lists-and-structs-as-results).

```zaka
အမှတ် ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု နှင့် y ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

လုပ်ဆောင်ချက် ရွှေ့ သည် မ ဟူသော အမှတ် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
မ ၏ x ကို (မ ၏ x) + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အမှတ်(x: ၁၊ y: ၂) ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။
ခ ၏ y ကို ၉ ဟု ထားမည်။
ရွှေ့ ဟူသော လုပ်ဆောင်ချက်ကို က ဖြင့် လုပ်ဆောင်မည်။
က ကို ပြသမည်။
ခ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အမှတ်(x: ၂၊ y: ၂)
အမှတ်(x: ၁၊ y: ၉)
```

## How a struct is stored

The fields stand in one stretch of RAM in the order declared, with the same rules
as a C struct:

- Each field starts at a multiple of its own alignment, with padding before it when needed.
- The size of the struct is rounded up to a multiple of the largest alignment among its
  fields, with padding after the last field.
- The alignment of a struct is the largest alignment among its fields. A field that is a
  struct or a list follows the alignment of what it holds.

`အရွယ်အစား` gives the size, in octets, and a list of structs is its count times that size.

```text
  ယ: { မှန်မှားတန်ဖိုး အ, ကိန်းပြည့် ဘ, မှန်မှားတန်ဖိုး စ }, 24 octets

  octet    0   1         8                16  17        24
           +---+---------+----------------+---+---------+
           | အ | padding |       ဘ        | စ | padding |
           +---+---------+----------------+---+---------+
```

```zaka
ယ ဟူသော ဒေတာအစုအဝေးတွင် အ ဟူသော မှန်မှားတန်ဖိုး တစ်ခု၊ ဘ ဟူသော ကိန်းပြည့် တစ်ခု နှင့် စ ဟူသော မှန်မှားတန်ဖိုး တစ်ခု တို့ပါဝင်သည်။
ရ ဟူသော ဒေတာအစုအဝေးတွင် ဘ ဟူသော ကိန်းပြည့် တစ်ခု၊ အ ဟူသော မှန်မှားတန်ဖိုး တစ်ခု နှင့် စ ဟူသော မှန်မှားတန်ဖိုး တစ်ခု တို့ပါဝင်သည်။
လ ဟူသော ဒေတာအစုအဝေးတွင် ဘ ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု နှင့် အ ဟူသော မှန်မှားတန်ဖိုး တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ယ တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
ရ တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
လ တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
အကွက် ၃ ခုရှိသော ယ စာရင်း တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၄
၁၆
၈
၇၂
```

In unsafe code, the address of a struct is a pointer to it, so adding 1 to it steps over
one whole struct, and a heap block of `n` structs holds `n` times the size. See [Unsafe
Zaka](unsafe.md).

## Errors at a glance

| Code | Problem | See |
| --- | --- | --- |
| ZK0228 | a struct declared inside a block | [Struct mistakes](../../learn/structs/mistakes.md#a-struct-declared-inside-a-block) |
| ZK0316 | a struct name declared twice | [Struct mistakes](../../learn/structs/mistakes.md#one-name-for-two-structs) |
| ZK0317 | a field name used twice in a struct | [Struct mistakes](../../learn/structs/mistakes.md#one-name-for-two-fields) |
| ZK0318 | a name in a kind that no struct declaration gives | [Struct mistakes](../../learn/structs/mistakes.md#a-name-that-is-no-struct) |
| ZK0319 | a struct name used for a value or a function | [Struct mistakes](../../learn/structs/mistakes.md#a-struct-name-that-is-also-a-value-name) |
| ZK0320 | a field the struct does not have | [Struct mistakes](../../learn/structs/mistakes.md#a-field-the-struct-does-not-have) |
| ZK0321 | a field set twice in a struct literal | [Struct mistakes](../../learn/structs/mistakes.md#one-field-set-twice) |
| ZK0426 | a field of a value that is no struct | [Struct mistakes](../../learn/structs/mistakes.md#a-field-of-a-value-that-is-no-struct) |
| ZK0427 | a struct that holds itself | [Struct mistakes](../../learn/structs/mistakes.md#a-struct-that-holds-itself) |
| ZK0482 | a struct literal that leaves out a field | [Struct mistakes](../../learn/structs/mistakes.md#a-field-left-out) |
| ZK0722 | a struct read before each field is set | [Struct mistakes](../../learn/structs/mistakes.md#a-struct-read-before-each-field-is-set) |
| ZK0509 | a kind, a struct literal or a heap size counted with a word that is not the classifier of the kind | [Struct mistakes](../../learn/structs/mistakes.md#the-wrong-classifier) |
