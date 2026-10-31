---
title: Structs
description: The struct declaration, its classifier, struct literals, field access, duplicating and moving, how a struct is stored, and what a struct cannot do.
---

# Structs

A struct groups a fixed set of fields, each with its own name and kind, in one stretch of
RAM. It is a kind that a program declares for itself, and its name works wherever a kind
does. The Learn book starts in small steps with [Structs](../../learn/structs/README.md).

```zaka
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ဒဿမကိန်း အမြင့် တို့ ပါဝင်သည်။
ကိန်းရှင် ကျော် သည် လူ ဖြစ်ပါစေ။
ကျော် ၏ အသက် ကို ၂၀ ဟု ထားပါ။
ကျော် ကို ပြသပါ။
```

```text
လူ(အသက်: ၂၀၊ အမြင့်: ၀.၀)
```

## Declaring a struct

```text
ဒေတာအမျိုးအစား <name> တွင် <kind> <field>၊ <kind> <field> တို့ ပါဝင်သည်။
ဒေတာအမျိုးအစား <name> တွင် <kind> <field>၊ <kind> <field> တို့ ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ <word> ဖြစ်သည်။
```

- `အချက်အလက်အမျိုးအစား` is a second spelling of `ဒေတာအမျိုးအစား`.
- A declaration stands at the top level of a file. Inside a block, a loop, a conditional
  or a function it is ZK0228. A struct can be used above its declaration, since the
  declarations are read first.
- A struct has at least one field. Each field is `<kind> <name>`, with `၊` between
  them, and the list closes with `တို့ ပါဝင်သည်` or `တို့ ပါဝင်ပြီး`.
- A field kind is each kind there is: a number, a true/false value, a character, a pointer, a list,
  or another struct.
- A struct name is declared once in the whole program (ZK0316), and the field names of
  one struct differ from each other (ZK0317).
- A struct name cannot also name a variable, a constant, a parameter, a field or a
  function (ZK0319), because the compiler reads each struct name as a kind wherever a
  value can stand, as in `လူ ၏ အရွယ်အစား`.
- A field cannot hold its own struct, at once or through other structs (ZK0427): it would
  have to be larger than itself. Hold an address of it, as in `အဆစ် လိပ်စာ နောက်`.
- A field cannot hold a borrow, as in `ငှားထားသော ကိန်းပြည့် လိပ်စာ ရှင်` (ZK0718). Use an
  owner (`<kind> လိပ်စာ`) or a raw address (`<kind> လိပ်စာအကြမ်း`). A borrow lasts as
  long as the owner it borrows from, which the checker follows through variables and
  parameters and not through a struct that can sit in the heap.

Where a kind must stand, as in a declaration, a parameter or a field, a name is read as a
struct name; a name that no declaration gives is ZK0318, as in
`ကိန်းရှင် က သည် လုပ် ဖြစ်ပါစေ။`.

## The classifier

Burmese counts things with a word that fits them: people with `ယောက်`, most other things
with `ခု`. A struct names its own counting word, its *classifier*, at the end of its
declaration, with `မျိုးပြပစ္စည်းမှာ <word> ဖြစ်သည်`. A struct that names none is counted
with `ခု`, like each other kind.

The classifier is the word in a heap size, `မင်မိုရီတွင် လူ ၃ ယောက်စာ နေရာယူ၍ ...`, and
the word after `တစ်` in a [struct literal](#struct-literals), `လူတစ်ယောက်`:

| Kind | Counted with |
| --- | --- |
| a struct that names a classifier | that word |
| a struct that names none | `ခု` |
| each other kind, such as `ကိန်းပြည့်` | `ခု` |

A heap size or a struct literal that uses another word, such as `ခု` for a struct that names `ယောက်`, or
`ယောက်` for `ကိန်းပြည့်`, is the warning ZK0509. The size does not change with the word,
and the program runs. The help line of the message gives the word to write. A classifier
word other than `ခု` is read in a heap size once a struct of the program names it:
`ကိန်းပြည့် ၃ ယောက်စာ` is the warning when some struct names `ယောက်`, and a shape
mistake (ZK0201) when none does.

List counts (`အကွက် ၃ ခုရှိသော`) and ordinals (`ခုမြောက်`) count cells, not structs, so
these use `ခု` for each kind.

```zaka
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ကိန်းပြည့် အတန်း တို့ ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။
ကိန်းရှင် p သည် လူ လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် လူ ၃ ယောက်စာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။
p ၏ တန်ဖိုး ၏ အသက် ကို ၁၀ ဟု ထားပါ။
p ၏ တန်ဖိုး ကို ပြသပါ။
လိပ်စာ p ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
လူ(အသက်: ၁၀၊ အတန်း: ၀)
```

```zaka-warning ZK0509
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက် တို့ ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။
ကိန်းရှင် p သည် လူ လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် လူ ၃ ခုစာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။
လိပ်စာ p ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
warning[ZK0509]: လူ is counted with "ယောက်", not "ခု"
```

## Where a struct kind goes

A struct name stands wherever a kind does:

| Place | Written |
| --- | --- |
| a variable | `ကိန်းရှင် ကျော် သည် လူ ဖြစ်ပါစေ။` |
| a parameter | `လူ သူ ကို လက်ခံပြီး` |
| a pointer kind | `လူ လိပ်စာ` |
| a borrow | `ငှားထားသော လူ လိပ်စာ` |
| the cells of a list | `အကွက် ၃ ခုရှိသော လူ စာရင်း` |
| a field of another struct | `လူ ပိုင်ရှင်` |
| a heap size | `မင်မိုရီတွင် လူ ၃ ယောက်စာ နေရာယူ၍ ...` |
| a size | `လူ ၏ အရွယ်အစား` |

A variable of a struct kind declared with no value starts with each field at its default:
`၀` for a number, `မမှန်` for a true/false value, `ဗလာ` for an address, and the default of
its kind for a list or another struct. A declaration can also take another struct of the
same kind, or a struct literal.

## Struct literals

```text
<field> <value>၊ <field> <value> ရှိသော <name>တစ်<word>
<field> <value>၊ <field> <value> ရှိသော <name> တစ်<word>
<name>(<field>: <value>၊ <field>: <value>)
```

A struct literal is a value of the struct `<name>`, with the fields it names set to the
values after them, as in `အသက် ၂၀၊ အမြင့် ၁.၇၅ ရှိသော လူတစ်ယောက်`, "a person who has
age 20 and height 1.75". The last line is the notation form, the one a struct prints in:
`လူ(အသက်: ၂၀၊ အမြင့်: ၁.၇၅)` means the same, and writes no classifier.

- The pairs stand in each order, with `၊` between them. The `ရှိသော` form writes at
  least one pair; the notation form can write none, as in `လူ()`, a struct with each
  field at its default.
- A field left out holds its default, as in a declaration with no value.
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
  declaration, the value of an assignment, an argument, or the value of `ပြသပါ`.
- An owner given to a field moves into the struct, and the variable it came from holds
  `ဗလာ` afterwards, as in an assignment.

```zaka
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ဒဿမကိန်း အမြင့် တို့ ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။
ကိန်းရှင် ကျော် သည် လူ ဖြစ်၍ တန်ဖိုးသည် အမြင့် ၁.၇၅၊ အသက် ၂၀ ရှိသော လူတစ်ယောက် ဖြစ်ပါစေ။
ကျော် ကို ပြသပါ။
အသက် ၃၀ ရှိသော လူ တစ်ယောက် ကို ပြသပါ။
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
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ဒဿမကိန်း အမြင့် တို့ ပါဝင်သည်။
ဒေတာအမျိုးအစား အသင်း တွင် လူ ခေါင်းဆောင်၊ အကွက် ၂ ခုရှိသော လူ စာရင်း အဖွဲ့ဝင် တို့ ပါဝင်သည်။
အသင်း(ခေါင်းဆောင်: လူ(အသက်: ၄၀)၊ အဖွဲ့ဝင်: [လူ(အသက်: ၂၀)၊ လူ(အမြင့်: ၁.၈)]) ကို ပြသပါ။
```

```text
အသင်း(ခေါင်းဆောင်: လူ(အသက်: ၄၀၊ အမြင့်: ၀.၀)၊ အဖွဲ့ဝင်: [လူ(အသက်: ၂၀၊ အမြင့်: ၀.၀)၊ လူ(အသက်: ၀၊ အမြင့်: ၁.၈)])
```

## Fields

`<value> ၏ <field>` is a field. It is a place: it can be read, printed, assigned to, and
have its address taken.

| Written | Means |
| --- | --- |
| `ကျော် ၏ အသက်` | the field of a struct variable |
| `ကျော် ၏ အသက် ကို ၂၀ ဟု ထားပါ။` | write the field |
| `p ၏ တန်ဖိုး ၏ အသက်` | the field of the struct a pointer points at |
| `က ၏ ခ ၏ ဂ` | a field of a field, one `၏` for each step |
| `xs ၏ အကွက် ၀ ၏ x` | a field of a list cell |
| `ကျော် ၏ အသက် ၏ လိပ်စာ` | the address of the field, a pointer to the kind of the field |

- A field is looked up in the struct on its name. A name the struct does not have is
  ZK0320.
- The value in front of `၏` has to be a struct (ZK0426). For a pointer, dereference it
  first with `တန်ဖိုး`: `p ၏ အသက်` is ZK0426.
- A field inside an operator expression goes in parentheses, as a list cell does:
  `(ကျော် ၏ အသက်) + ၁`. Without parentheses the sentence is ZK0209 or ZK0206.

```zaka
ဒေတာအမျိုးအစား အမှတ် တွင် ကိန်းပြည့် x၊ ကိန်းပြည့် y တို့ ပါဝင်သည်။
ဒေတာအမျိုးအစား မျဉ်း တွင် အမှတ် စ၊ အမှတ် ဆုံး တို့ ပါဝင်သည်။
ကိန်းရှင် က သည် မျဉ်း ဖြစ်ပါစေ။
က ၏ ဆုံး ၏ x ကို ၃ ဟု ထားပါ။
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော အမှတ် စာရင်း ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက် ၁ ၏ y ကို ၇ ဟု ထားပါ။
ကိန်းရှင် ပ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် က ၏ စ ၏ y ၏ လိပ်စာ ဖြစ်ပါစေ။
ပ ၏ တန်ဖိုး ကို ၉ ဟု ထားပါ။
က ကို ပြသပါ။
အမှတ်များ ကို ပြသပါ။
```

```text
မျဉ်း(စ: အမှတ်(x: ၀၊ y: ၉)၊ ဆုံး: အမှတ်(x: ၃၊ y: ၀))
[အမှတ်(x: ၀၊ y: ၀)၊ အမှတ်(x: ၀၊ y: ၇)၊ အမှတ်(x: ၀၊ y: ၀)]
```

## Printing

A struct prints as its name, then `(`, each field as `<name>: <value>` with `၊` between
them, and `)`, as in `လူ(အသက်: ၂၀၊ အမြင့်: ၁.၇၅)`. A struct in a field and a list in a
field print alike, nested.

```zaka
ဒေတာအမျိုးအစား ရမှတ်များ တွင် လက္ခဏာမဲ့ ကိန်းပြည့် အကြိမ်၊ အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း ရမှတ် တို့ ပါဝင်သည်။
ကိန်းရှင် ပွဲ သည် ရမှတ်များ ဖြစ်ပါစေ။
ပွဲ ၏ ရမှတ် ကို ၅၊ ၇၊ ၉ တို့ပါသော စာရင်း ဟု ထားပါ။
ပွဲ ၏ အကြိမ် ကို ပွဲ ၏ ရမှတ် ၏ အကွက်အရေအတွက် ဟု ထားပါ။
ပွဲ ကို ပြသပါ။
```

```text
ရမှတ်များ(အကြိမ်: ၃၊ ရမှတ်: [၅၊ ၇၊ ၉])
```

## Duplicating and moving

A struct is a value, like an integer or a list. Setting one struct to another of the same
kind duplicates each field, and the two stand apart afterwards. A struct passed as an
argument is duplicated into the parameter alike, so a change to a struct
parameter is lost when the function ends. To let a function change the caller's struct, pass a
borrowed address: the parameter kind is `ငှားထားသော <struct> လိပ်စာ`, and the call gives
`<variable> ၏ လိပ်စာ`.

A struct with a field that is an owner pointer (`<kind> လိပ်စာ`), at once or inside a field
that is a struct or a list, moves instead of being duplicated, as an owner does. The new
struct takes the address, the old one is left with `ဗလာ` in that field, and a later use of
the old struct is ZK0701. See [Ownership and borrowing](ownership.md#one-owner). A struct
given back from a function follows the same rules; see [Lists and structs as
results](functions.md#lists-and-structs-as-results).

```zaka
ဒေတာအမျိုးအစား အဆစ် တွင် ကိန်းပြည့် တန်ဖိုးရင်း၊ အဆစ် လိပ်စာ နောက် တို့ ပါဝင်သည်။
ကိန်းရှင် က သည် အဆစ် ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အဆစ် ဖြစ်ပါစေ။
မင်မိုရီတွင် အဆစ် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို က ၏ နောက် တွင် သိမ်းထားမည်။
က ၏ တန်ဖိုးရင်း ကို ၇ ဟု ထားပါ။
ခ ကို က ဟု ထားပါ။            // the address moves into ခ
ခ ၏ တန်ဖိုးရင်း ကို ပြသပါ။
ခ ၏ နောက် ၏ တန်ဖိုး ကို ပြသပါ။
လိပ်စာ ခ ၏ နောက် ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
၇
အဆစ်(တန်ဖိုးရင်း: ၀၊ နောက်: ဗလာ)
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
ဒေတာအမျိုးအစား ယ တွင် မှန်မှားတန်ဖိုး အ၊ ကိန်းပြည့် ဘ၊ မှန်မှားတန်ဖိုး စ တို့ ပါဝင်သည်။
ဒေတာအမျိုးအစား ရ တွင် ကိန်းပြည့် ဘ၊ မှန်မှားတန်ဖိုး အ၊ မှန်မှားတန်ဖိုး စ တို့ ပါဝင်သည်။
ဒေတာအမျိုးအစား လ တွင် ၃၂ ဘစ် ကိန်းပြည့် ဘ၊ မှန်မှားတန်ဖိုး အ တို့ ပါဝင်သည်။
ယ ၏ အရွယ်အစား ကို ပြသပါ။
ရ ၏ အရွယ်အစား ကို ပြသပါ။
လ ၏ အရွယ်အစား ကို ပြသပါ။
အကွက် ၃ ခုရှိသော ယ စာရင်း ၏ အရွယ်အစား ကို ပြသပါ။
```

```text
၂၄
၁၆
၈
၇၂
```

The address of a struct is a pointer to it, so adding 1 to it moves one whole struct, and
a heap block of `n` structs holds `n` times the size. See [Pointers](pointers.md#pointer-arithmetic).

## What a struct cannot do

| Not allowed | Instead |
| --- | --- |
| `==`, `!=`, `<` and the like on structs (ZK0403) | compare the fields |
| a field that is a borrow (ZK0718) | hold an owner or a raw address |
| a struct that holds itself, except through an address (ZK0427) | hold `<name> လိပ်စာ` |
| a struct with no fields | declare the field the struct needs |
| a struct declared inside a block (ZK0228) | declare it at the top level |

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
| ZK0509 | a heap size or a struct literal counted with a word that is not the classifier of the kind | [Struct mistakes](../../learn/structs/mistakes.md#the-wrong-classifier) |
| ZK0718 | a field that holds a borrow | [Struct mistakes](../../learn/structs/mistakes.md#a-borrow-in-a-field) |
