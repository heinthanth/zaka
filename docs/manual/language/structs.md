---
title: Structs
description: The struct declaration, its measure word, struct literals, fields, comparing, printing, duplicates, and how a struct is stored.
---

# Structs

A struct (record) puts a fixed set of fields (members) together under one name, and each
field has its own name and data kind (type). Once we declare a struct, its name is a data
kind just like `ကိန်းပြည့်`, so we can use it wherever a data kind is expected, and its
fields are stored next to each other in RAM (memory). This page states the full rules,
while the Learn book teaches structs in small steps in [Structs](../../learn/structs/README.md).

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
<name> ဟူသော [အများသုံး] ဒေတာအစုအဝေးတွင် <field> ဟူသော <kind> တစ်<word> ပါဝင်သည်။
<name> ဟူသော [အများသုံး] ဒေတာအစုအဝေးတွင် <field> ဟူသော <kind> တစ်<word>၊ … နှင့် <field> ဟူသော <kind> တစ်<word> တို့ပါဝင်သည်။
<name> ဟူသော [အများသုံး] ဒေတာအစုအဝေးတွင် <field> ဟူသော <kind> တစ်<word>၊ … နှင့် … တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ <word> ဖြစ်သည်။
```

To declare a struct, we write its name, `ဟူသော`, the word `ဒေတာအစုအဝေးတွင်` and then its
fields. We write each field as `<field> ဟူသော <kind> တစ်<word>`, where `<word>` is the
[measure word](#measure-words) of the field's data kind, most often `ခု`. The fields form
a [list in words](words.md#lists-in-words), so we put `၊` between them and `နှင့်` before
the last one, and we end the list with `တို့ပါဝင်သည်`. A struct with a single field ends
with `ပါဝင်သည်`, without `တို့`. To give the struct a measure word of its choice, we end
with `တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ <word> ဖြစ်သည်` instead, or with
`ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ <word> ဖြစ်သည်` after a single field.

A struct needs at least one field, and a field can have each data kind there is: a
number, a true/false value, a character, text, a list, another struct, a shared object, a
function value or an address (pointer). Safe code can keep an address in a field, while
reading through it needs unsafe code ([Unsafe Zaka](unsafe.md)).

We declare a struct at the top level of a file. The rest of the program can use it even
above the declaration, because the compiler reads all declarations first. Where a data
kind is expected, as in a declaration, a parameter or a field, the compiler reads a name
as a struct name, and a struct name cannot also be the name of a value, because a struct
name can appear where a value can, as in `လူ တစ်ခု ၏ အရွယ်အစား`.

| Mistake | Code |
| --- | --- |
| `တို့` after a single field, or no `တို့` after several | ZK0201 |
| a list of fields with no `နှင့်` before the last one | ZK0245 |
| a declaration inside a block, a loop, a conditional or a function | ZK0228 |
| a struct name declared twice in the whole program | ZK0316 |
| two fields of one struct with the same name | ZK0317 |
| a name in the place of a data kind that no declaration gives, as in `ကိန်းရှင် က သည် လုပ် တစ်ခု ဖြစ်ပါစေ။` | ZK0318 |
| a struct name that also names a variable, a constant, a parameter, a field or a function | ZK0319 |
| a struct that contains itself | ZK0427 |

A struct cannot contain itself: not as a field, not through other structs and not through
a counted list of itself (ZK0427), because its size would have no end. A shared object
can name its own data kind in its fields, since such a field keeps a reference, but the
chain still needs an end, so one field on the loop must be an optional or a plain list,
as in `နောက် ဟူသော အဆစ် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု`. A loop with no such field
is ZK0481 ([Data kinds no program can build](objects.md#data-kinds-no-program-can-build)).

### Spellings and related forms

- Zaka treats `ဒေတာ` and `အချက်အလက်` as one word in names and reserved words
  ([Names](words.md#names)), so `အချက်အလက်အစုအဝေး` works the same as `ဒေတာအစုအဝေး`,
  and a struct name written with either spelling names one struct.
- We can join `တွင်` to `ဒေတာအစုအဝေး` or write it apart; these pages join them.
- To export a struct from its module, we put `အများသုံး` after `ဟူသော`, before
  `မျှဝေနိုင်သော` and `ဒေတာအစုအဝေး` ([Modules](modules.md)).
- When we add `မျှဝေနိုင်သော` before `ဒေတာအစုအဝေး`, as in
  `<name> ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် ...`, we declare a shared object
  (reference type) instead of a struct. The declaration, the literal and the fields work
  the same, but assigning a shared object shares it instead of duplicating it. This page
  states the rules for both, and [Values and objects](objects.md) explains what sharing
  changes.
- An enumeration uses the same shape, with `ဒေတာအမျိုးအစားကွဲ` in place of
  `ဒေတာအစုအဝေး` ([Enumerations](enums.md#declaring-an-enumeration)).

## Marks after the head

```text
<name> ဟူသော [အများသုံး] [မျှဝေနိုင်သော] ဒေတာအစုအဝေးသည် [<mark> ဒေတာအစုအဝေး ဖြစ်၍] [<kind parameters>] ၎င်းတွင် <fields> တို့ပါဝင်သည်။
```

The first part of a struct declaration, up to the fields, can also have a mark that
changes what the struct is, and the kind parameters of a [generic
struct](generics.md#generic-structs). To use either, we end that part with
`ဒေတာအစုအဝေးသည်` in place of `ဒေတာအစုအဝေးတွင်`, write the mark followed with
`ဒေတာအစုအဝေး ဖြစ်၍`, then the kind parameters, and then `၎င်းတွင်` and the fields, much as a
function signature has its marks after `သည်`.

| Mark | What it makes | See |
| --- | --- | --- |
| `"<C name>" အမည်ဖြင့် ပြင်ပမှ ယူသုံးနိုင်သော` | a struct that a lib gives to C under the C name | [Functions C can call](functions-exports.md) |
| `ပြင်ပမှ လာသော` | a data kind from outside the program: a handle to a struct that C keeps, with the fields C gives; without fields, the declaration ends with `ဖြစ်သည်။` | [C functions](functions-c.md#data-kinds-from-outside-the-program) |
| `ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော` | a struct that the compiler provides, allowed in the core module alone (ZK0349) | [The core module](core.md) |

```text
point ဟူသော ဒေတာအစုအဝေးသည် "point" အမည်ဖြင့် ပြင်ပမှ ယူသုံးနိုင်သော ဒေတာအစုအဝေး ဖြစ်၍ ၎င်းတွင် x ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု နှင့် y ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။
အတွဲ ဟူသော ဒေတာအစုအဝေးသည် ယေဘုယျဒေတာအမျိုးအစား K နှင့် ယေဘုယျဒေတာအမျိုးအစား V တို့ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင် ရှေ့ ဟူသော K တစ်ခု နှင့် နောက် ဟူသော V တစ်ခု တို့ပါဝင်သည်။
db ဟူသော ဒေတာအစုအဝေးသည် ပြင်ပမှ လာသော ဒေတာအစုအဝေး ဖြစ်သည်။
```

We can write `ဖြစ်ပြီး` or `ဖြစ်ကာ` in place of `ဖြစ်၍` ([Second
spellings](words.md#second-spellings)). A struct with no mark and no kind parameters goes
straight on with `ဒေတာအစုအဝေးတွင်`.

Some marks cannot be combined (ZK0263). A C name or `ပြင်ပမှ လာသော` cannot go with kind
parameters, because C has no generic structs and a data kind from outside is a single
struct of C. `မျှဝေနိုင်သော` cannot go with either of these marks, because C passes a
struct whole and each handle is a shared value.

## Measure words

```text
<name> ဟူသော ဒေတာအစုအဝေးတွင် <fields> တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ <word> ဖြစ်သည်။
<kind> တစ်<word>
```

Burmese counts things with a measure word (classifier) that fits the thing being counted,
so people are counted with `ယောက်` and most other things with `ခု`, and Zaka does the
same. A struct is counted with `ခု` unless we give it a different word at the end of its
declaration, with `တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ <word> ဖြစ်သည်`.

We write the measure word after `တစ်` each time we write out a data kind: in a
declaration, a parameter, a field, a result, a conversion, a size, a default arm, and in a
[struct literal](#struct-literals) such as `လူတစ်ယောက်`
([Measure words](kinds.md#measure-words)). With the struct below, a variable is
`ကိန်းရှင် ကျော် သည် လူ တစ်ယောက် ဖြစ်ပါစေ။`. In unsafe code, the word also goes in a heap
size, as in `မင်မိုရီတွင် လူ ၃ ယောက်စာ နေရာယူ၍ ...` ([Unsafe Zaka](unsafe.md)).

| Data kind | Counted with |
| --- | --- |
| a struct, an enumeration or an alias that names a measure word | that word |
| a struct or an enumeration that names none | `ခု` |
| an alias that names none | the word of the data kind behind it |
| `စာလုံး` | `လုံး` |
| all other data kinds, such as `ကိန်းပြည့်` or `လူ စာရင်း` | `ခု` |

When a data kind in one of those places has no measure word after it, the compiler
reports ZK0234. A measure word that does not belong to the data kind, such as `ခု` for a
struct that names `ယောက်`, or `ယောက်` for `ကိန်းပြည့်`, is the warning ZK0509: the sizedoes not change, the program still runs, and the help line of the message gives the right
word. In a heap size, the compiler reads a word other than `ခု` as a measure word once a
struct of the program names it, so `ကိန်းပြည့် ၃ ယောက်စာ` is the warning when some struct
names `ယောက်`, and a shape mistake (ZK0201) when none does.

List counts (`အကွက် ၃ ခုရှိသော`) and ordinals (`ခုမြောက်`) count cells, not structs, so
these use `ခု` for all data kinds.

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

## Using a struct as a data kind

A struct name works wherever a data kind does:

| Place | Written |
| --- | --- |
| a variable | `ကိန်းရှင် ကျော် သည် လူ တစ်ယောက် ဖြစ်ပါစေ။` |
| a parameter | `သူ ဟူသော လူ တစ်ယောက် ကို လက်ခံပြီး` |
| the cells of a list | `အကွက် ၃ ခုရှိသော လူ စာရင်း တစ်ခု` |
| a field of another struct | `ပိုင်ရှင် ဟူသော လူ တစ်ယောက်` |
| a size | `လူ တစ်ယောက် ၏ အရွယ်အစား` |
| an address (pointer) data kind | `လူ လိပ်စာ တစ်ခု` |
| a heap size, in unsafe code | `မင်မိုရီတွင် လူ ၃ ယောက်စာ နေရာယူ၍ ...` |

A variable of a struct can get a struct literal, another struct of the same data kind, or
`ကနဦးတန်ဖိုး`, the [initial value](kinds.md#initial-values) (default value), which sets
each field to the initial value of the field's data kind, as in
`ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။`. When we declare a
struct variable with no value, each field starts unset, and the program must set each one
before it reads the struct ([Set before read](declarations.md#set-before-read)):

- The struct counts as set once each field is set on each path.
- The compiler follows a struct inside a struct one field at a time too: writing
  `က ၏ ဆုံး ၏ x` sets that one field, and `က ၏ ဆုံး` counts as set once each of its
  fields is set.
- Reading a field that can be unset, or the whole struct while one of its fields can be
  unset, is ZK0722.

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

We can use a struct literal to write out a whole struct as one value. In the word form, we
give each field as `<field> အနေဖြင့် <value>`, join the pairs as a [list in
words](words.md#lists-in-words), with `၊` between them and `နှင့်` before the last, and
end with `ရှိသော <name>တစ်<word>`. So `အသက် အနေဖြင့် ၂၀ နှင့် အမြင့် အနေဖြင့် ၁.၇၅ ရှိသော လူတစ်ယောက်`
means "a person who has age 20 and height 1.75". The short form,
`လူ(အသက်: ၂၀၊ အမြင့်: ၁.၇၅)`, is the form that a struct prints in, and it needs no measure
word.

| Rule | Code |
| --- | --- |
| each field of the struct gets a value, so `လူ(အသက်: ၁)` is wrong for a struct that also has `အမြင့်` | ZK0482 |
| each field name is a field of the struct | ZK0320 |
| each field is set once | ZK0321 |
| each value converts to the data kind of its field, as the value of an assignment does | ZK0401 |
| `<name>` is a declared struct | ZK0318 |
| `<word>` is the measure word of the struct, `ခု` when the declaration names none | ZK0509 (warning) |

- The pairs can come in a different order from the declaration.
- To give a field its initial value, we write `ကနဦးတန်ဖိုး` as its value:
  `လူ(အသက်: ၁၊ အမြင့်: ကနဦးတန်ဖိုး)`.
- In the word form, each field needs `အနေဖြင့်` before its value, and a field next to
  its value without it does not parse. We can write the name and `တစ်` as one word or
  with a space between them.
- In the short form, we put `၊` between the pairs and `:` after each field name, and the
  struct name and `(` go together. After a name that no struct declares, `()` is a
  [call](functions-calls.md#the-sign-form) with no arguments.
- Each value is a notation expression, such as `x + ၁`, `(က ၏ အသက်)`, `[၁၊ ၂]` or
  `လူ(အသက်: ၁)`. A clause, or a literal in the `ရှိသော` or `တို့ပါသော စာရင်း` form, goes
  in parentheses.
- We can use a struct literal wherever a value of the struct is expected: as the value of
  a declaration, the value of an assignment, an argument, or the value of `ပြသမည်`.

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

The compiler tells a struct literal apart from other values at its start, where a field
name has a value right after it, before the next `ရှိသော`. Nowhere else in a value does a
name sit next to another value without an operator or a particle between them. The short
form starts with a struct name and `(`, or with a name, `(`, a field name and `:`.

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

```text
<value> ၏ <field>
```

To reach a field, we write the value, `၏` and the field name. A field is a place, so we
can read it, print it, assign to it and pass it to a mutable parameter (`inout`),
`ဒေတာ ပြောင်းလဲရန်` ([Mutable parameters](functions-parameters.md#mutable-parameters)), and in unsafe code we can also take its
address.

| Written | Meaning |
| --- | --- |
| `ကျော် ၏ အသက်` | the field of a struct variable |
| `ကျော် ၏ အသက် ကို ၂၀ ဟု ထားမည်။` | write to the field |
| `က ၏ ခ ၏ ဂ` | a field of a field, with one `၏` for each step |
| `xs ၏ အကွက် ၀ ၏ x` | a field of a list cell |
| `p ၏ တန်ဖိုး ၏ အသက်` | in unsafe code, the field of the struct that a pointer points to |
| `ကျော် ၏ အသက် ၏ လိပ်စာ` | in unsafe code, the address of the field, a pointer to the data kind of the field |

- The compiler looks up a field in the struct through its name, and a name that the
  struct does not have is ZK0320.
- The value before `၏` must be a struct or a shared object (ZK0426). To reach a field
  through a pointer, we first read the pointer with `တန်ဖိုး`, so `p ၏ အသက်` is ZK0426.
- When a field is part of an expression with an operator, we put it in parentheses, just
  as we do with a list cell: `(ကျော် ၏ အသက်) + ၁`. Without the parentheses, the sentence
  is ZK0209 or ZK0206.

```zaka
အမှတ် ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု နှင့် y ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။
မျဉ်း ဟူသော ဒေတာအစုအဝေးတွင် စ ဟူသော အမှတ် တစ်ခု နှင့် ဆုံး ဟူသော အမှတ် တစ်ခု တို့ပါဝင်သည်။

ကိုး ဟူသော လုပ်ဆောင်ချက် သည် န ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
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

```text
<struct> == <struct>
<struct> != <struct>
```

We can use `==` and `!=` on two structs of the same data kind, and two structs are equal
when each field equals the same field of the other. The compiler compares each field
according to its data kind:

| Field | Compared |
| --- | --- |
| a number or a character | as a value |
| text or a list | one cell at a time |
| a struct | one field at a time |
| a shared object | in depth, as `==` compares objects ([Values and objects](objects.md)) |
| a function value | as the same function |
| a pointer | through the address in it |

The padding between fields is not compared. A decimal field follows the rules of decimals,
so `-0.0` equals `0.0` and a `NaN` field makes the two structs unequal. A list of structs
compares each struct in turn.

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

Two structs of different data kinds cannot be compared, even when their fields are the
same (ZK0404), and the ordering operators such as `<` do not work on structs (ZK0403).

## Printing

When we print a struct, the output shows its name, then `(`, then each field as
`<name>: <value>` with `၊` between them, and then `)`, as in `လူ(အသက်: ၂၀၊ အမြင့်: ၁.၇၅)`.
A struct or a list in a field prints in the same form, nested inside.

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

## Duplicating a struct

A struct is a value, like an integer or a list ([Values and objects](objects.md)). When we
assign one struct to another of the same data kind, each field is duplicated (copied), so
the two structs are separate afterwards and a later write to one never reaches the other.
A text or list field shares its cells with the old struct until one side writes to it. A
field that keeps a shared object, a function value or a pointer duplicates the reference,
so both structs reach the same object.

When we pass a struct as an argument, the parameter gets a duplicate in the same manner,
so a change to the parameter is lost when the function ends. To let a function change the
caller's struct, we mark the parameter with `ဒေတာ ပြောင်းလဲရန်`, which makes it a mutable
parameter (`inout`) ([Mutable parameters](functions-parameters.md#mutable-parameters)), and the call passes the struct with no
extra mark. A struct that a function returns follows the same rules, as [Lists and structs
as results](functions-results.md#lists-and-structs-as-results) explains.

```zaka
အမှတ် ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု နှင့် y ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

ရွှေ့ ဟူသော လုပ်ဆောင်ချက် သည် မ ဟူသော အမှတ် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
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

The fields of a struct are stored in one stretch of RAM, in the order of the declaration,
with the same rules as a C struct:

- Each field starts at a multiple of its alignment, with padding before it when needed.
- The size of the struct is rounded up to a multiple of the largest alignment among its
  fields, with padding after the last field.
- The alignment of a struct is the largest alignment among its fields. A field that is a
  struct or a list follows the alignment of what is inside it.

We can use `အရွယ်အစား` to get the size in octets (bytes), and a counted list of structs
is its count times that size.

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

In unsafe code, the address of a struct is a pointer to it, so adding 1 to the pointer
steps over one whole struct, and a heap block of `n` structs is `n` times the size. See
[Unsafe Zaka](unsafe.md).

## What a struct cannot do

| Not allowed | Instead |
| --- | --- |
| `<`, `>` and the other ordering operators on structs (ZK0403) | compare the fields |
| `==` on two structs of different data kinds (ZK0404) | compare the fields |
| a struct with no fields | declare the field that the struct needs |
| a struct declared inside a block (ZK0228) | declare it at the top level |
| a struct that contains itself (ZK0427) | declare it `မျှဝေနိုင်သော`, so that the field keeps a reference, and make the field optional |
| a struct literal that leaves out a field (ZK0482) | give each field a value, or `ကနဦးတန်ဖိုး` for its initial value |
| reading a struct before each field is set (ZK0722) | set each field first, or declare the struct with a value |

## Errors at a glance

| Code | Problem | See |
| --- | --- | --- |
| ZK0228 | a struct declared inside a block | [Struct mistakes](../../learn/structs/mistakes.md#a-struct-declared-inside-a-block) |
| ZK0316 | a struct name declared twice | [Struct mistakes](../../learn/structs/mistakes.md#one-name-for-two-structs) |
| ZK0317 | a field name used twice in a struct | [Struct mistakes](../../learn/structs/mistakes.md#one-name-for-two-fields) |
| ZK0318 | a name in the place of a data kind that no struct declaration gives | [Struct mistakes](../../learn/structs/mistakes.md#a-name-that-is-no-struct) |
| ZK0319 | a struct name used for a value or a function | [Struct mistakes](../../learn/structs/mistakes.md#a-struct-name-that-is-also-a-value-name) |
| ZK0320 | a field that the struct does not have | [Struct mistakes](../../learn/structs/mistakes.md#a-field-the-struct-does-not-have) |
| ZK0321 | a field set twice in a struct literal | [Struct mistakes](../../learn/structs/mistakes.md#one-field-set-twice) |
| ZK0426 | a field of a value that is not a struct | [Struct mistakes](../../learn/structs/mistakes.md#a-field-of-a-value-that-is-no-struct) |
| ZK0427 | a struct that contains itself | [Struct mistakes](../../learn/structs/mistakes.md#a-struct-that-contains-itself) |
| ZK0482 | a struct literal that leaves out a field | [Struct mistakes](../../learn/structs/mistakes.md#a-field-left-out) |
| ZK0722 | a struct read before each field is set | [Struct mistakes](../../learn/structs/mistakes.md#a-struct-read-before-each-field-is-set) |
| ZK0509 | a data kind, a struct literal or a heap size counted with a word that is not the measure word of its data kind | [Struct mistakes](../../learn/structs/mistakes.md#the-wrong-measure-word) |
| ZK0263 | a struct with marks that cannot be combined | [Marks after the head](#marks-after-the-head) |
