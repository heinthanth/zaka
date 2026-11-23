---
title: Generics
description: Functions, structs, enumerations and aliases with kind parameters, data kinds at a use and in calls, the notes block, written-as hints, and how the compiler builds instances.
---

# Generics

A generic function, struct, enumeration or kind alias has kind parameters (type parameters): names that
take the place of data kinds (types) that the code leaves open. Each use gives each kind
parameter a data kind, and the compiler builds one instance (monomorphized instance) of
the generic for each set of data kinds that the program uses, as if we had written the
code once for each set. A kind parameter can require traits (interfaces), which
[Traits](traits.md) describes. The Learn book teaches generics in small steps in
[Generics](../../learn/generics/README.md).

```zaka
ထပ် ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
က ဟူသော T တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ထပ်(၁) ကို ပြသမည်။
ထပ်("မင်္ဂလာပါ") ကို ပြသမည်။
ဒဿမကိန်း ကို အစားထိုးအသုံးပြုသည့် ထပ် ဟူသော လုပ်ဆောင်ချက်ကို ၂.၅ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁
မင်္ဂလာပါ
၂.၅
```

| Zaka | Reads as |
| --- | --- |
| `ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍` | "the general data kind T can be put in place, and" |
| `ဒဿမကိန်း ကို အစားထိုးအသုံးပြုသည့် ထပ်` | "the ထပ် that puts the decimal number in place" |
| `ဤနေရာတွင် သတိပြုရန်မှာ` | "here, to note:" |

## Kind parameters

```text
<name> ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား <param> ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည် …
<name> ဟူသော ဒေတာအစုအဝေးသည် ယေဘုယျဒေတာအမျိုးအစား <param> ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင် …
ဒေတာအမျိုးအစား <alias> သည် ယေဘုယျဒေတာအမျိုးအစား <param> ကိုအစားထိုးသုံးနိုင်၍ ၎င်းကို …
<name><<param>၊ …> ဟူသော …                                              (sign form)
```

To give a function, a struct or an alias kind parameters, we write
`ယေဘုယျဒေတာအမျိုးအစား <name>` for each one after the name and `သည်`, and then
`အစားထိုးသုံးနိုင်၍ ၎င်း` and the word that continues the declaration: `သည်` for a
function, `တွင်` for a struct and `ကို` for an alias. The parameters form a [list in
words](words.md#lists-in-words), so we write `ကို` after a single parameter, and with two
or more we put `နှင့်` before the last one and `တို့ကို` after it.

We can also use the sign form, with the parameters inside `<` and `>` right after the name
and `၊` between them: `အတွဲလုပ်<A၊ B> ဟူသော လုပ်ဆောင်ချက် သည်`,
`အတွဲ<က၊ ခ> ဟူသော ဒေတာအစုအဝေးတွင်`, `ဒေတာအမျိုးအစား အမည်ဇယား<T> ကို …`. Both forms
declare the same generic, and these pages use the word form.

- Inside the declaration, a kind parameter works as a data kind name. It is counted with
  `ခု`, whatever data kind it takes the place of: `T တစ်ခု`, `T စာရင်း တစ်ခု`.
- A kind parameter can take each data kind that a program can write, function kinds
  included, but not a borrow or a raw address, because no phrase names them (ZK0341).
- A kind parameter can require traits. In the sign form, we write them after `:`, joined
  with `နှင့်` when there are two or more: `<T: နံပါတ်နှင့်တူသော>`,
  `<T: စီနိုင်သော နှင့် ပြနိုင်သော>`. The [notes block](#the-notes-block) gives them in
  either form.
- The compiler never checks a generic as written. It checks each instance with its data
  kinds in place, and an error inside a generic points at the generic's code, with a label
  at the use that asked for the instance. An error that several instances share is
  reported once.

A struct in the word form ends its first part with `ဒေတာအစုအဝေးသည်` in place of
`ဒေတာအစုအဝေးတွင်`, and `၎င်းတွင်` then starts its fields, as for the other [marks after
the head](structs.md#marks-after-the-head). This declares the same struct as
`အတွဲ<က၊ ခ> ဟူသော ဒေတာအစုအဝေးတွင် …`:

```zaka
အတွဲ ဟူသော ဒေတာအစုအဝေးသည်
ယေဘုယျဒေတာအမျိုးအစား က နှင့် ယေဘုယျဒေတာအမျိုးအစား ခ တို့ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင်
ပထမ ဟူသော က တစ်ခု နှင့်
ဒုတိယ ဟူသော ခ တစ်ခု
တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ တွဲ ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အတွဲ(ပထမ: "မောင်မောင်"၊ ဒုတိယ: ၂၄) ဖြစ်ပါစေ။
က ၏ ဒုတိယ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၄
```

## Data kinds at a use

```text
<kind> [၊ <kind> …] [နှင့် <kind>] (ကို|တို့ကို) အစားထိုးအသုံးပြုသည့် <name>   (word form)
<name><<kind>၊ …>                                                    (sign form)
<the words of its hint>                                              (written-as form)
```

To give the data kinds of a generic struct or alias where we use it, we write them in the
word form, before the name, or in the sign form, right after the name with no space:
`စာသား နှင့် ကိန်းပြည့် တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ`, `အတွဲ<စာသား၊ ကိန်းပြည့်>`. A
third form uses the words of the generic's [written-as hint](#written-as-hints).

- Data kinds can nest: `အတွဲ<အတွဲ<စာသား၊ ကိန်းပြည့်>၊ ကိန်းပြည့်>`, and inside data kinds,
  `>>` counts as two `>`. A data kind in the word form that contains a list goes in
  parentheses when it is inside other data kinds.
- A different count of data kinds from the declaration of the generic is ZK0340.

## Calls

```text
<kinds> အစားထိုးအသုံးပြုသည့် <f> ဟူသော လုပ်ဆောင်ချက်ကို <args> ဖြင့် လုပ်ဆောင်…      (long call)
<kinds> အစားထိုးအသုံးပြု၍ <args> ကို သုံး၍ <f> မည်။                                (arguments first)
<f>::<<kind>၊ …>(<args>)                                                    (sign call)
```

When a call writes no data kinds, the compiler works them out: first from the data kinds
of the arguments, matched against the parameter data kinds through lists, addresses,
optionals, results, tables (`ဇယား`) and generic structs, and then from the data kind that
the place of the call expects, for a kind parameter that the result alone names.

- When the call cannot work out a data kind, or two arguments give one kind parameter two
  different data kinds, the compiler reports ZK0341, and the call must then write its data
  kinds.
- Data kinds written for a function that is not generic are ZK0340.
- A function value of a generic writes its data kinds, `<f>::<<kind>၊ …>`, or gets them
  from the function kind that its place expects.

Generic functions with the same name are overloads, as plain functions are. A call runs
the one whose parameter count and prepositions fit the call and whose data kinds the call
can work out, and when the data kinds fit more than one, the parameters of fixed data
kinds tell them apart. A plain function of the same name that fits comes first. A function
value picks its generic function through the count of data kinds that it writes, or
through the function kind that its place expects.

| Case | Code |
| --- | --- |
| no generic function of the name fits the call | ZK0332 |
| more than one generic function fits | ZK0333 |
| two generic functions with the same signature: the same parameters, prepositions and result, whatever their kind parameters are called | ZK0304 |

```zaka
ဗလာ ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
ကနဦးတန်ဖိုး ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
// T comes from the data kind of the variable
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဗလာ() ဖြစ်ပါစေ။
က ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၀
```

Without a data kind to take `T` from, the call is ZK0341:

```zaka-error ZK0341
ဗလာ ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
ကနဦးတန်ဖိုး ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဗလာ() ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

Two generic functions named `ရွေး`, one with a single parameter and one with two:

```zaka
ရွေး ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
က ဟူသော T တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ရွေး ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
က ဟူသော T တစ်ခု ကို လည်းကောင်း
ခ ဟူသော T တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
// one value, then two
ရွေး(၁) ကို ပြသမည်။
ရွေး("က"၊ "ခ") ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁
ခ
```

## Generic structs

A struct with kind parameters is a pattern for structs. When a value of one writes no data
kinds, it gets them from the data kind that its place expects, or else from its field
values. To write them, we use the word form
`… ရှိသော <kinds> အစားထိုးအသုံးပြုသည့် <struct> တစ်<word>`, or the sign form
`<struct>::<<kind>၊ …>(…)`.

```zaka
အတွဲ ဟူသော ဒေတာအစုအဝေးသည်
ယေဘုယျဒေတာအမျိုးအစား က နှင့် ယေဘုယျဒေတာအမျိုးအစား ခ တို့ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင်
ပထမ ဟူသော က တစ်ခု နှင့်
ဒုတိယ ဟူသော ခ တစ်ခု
တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ တွဲ ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ပထမ အနေဖြင့် "မောင်မောင်" နှင့် ဒုတိယ အနေဖြင့် ၂၄ ရှိသော အတွဲ တစ်တွဲ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ဒဿမကိန်း နှင့် ဒဿမကိန်း တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ တစ်တွဲ ဖြစ်၍ တန်ဖိုးသည်
ပထမ အနေဖြင့် ၁ နှင့် ဒုတိယ အနေဖြင့် ၂ ရှိသော အတွဲ တစ်တွဲ ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည်
ပထမ အနေဖြင့် 'က' နှင့် ဒုတိယ အနေဖြင့် ၃ ရှိသော စာလုံး နှင့် ကိန်းပြည့် တို့ကို အစားထိုးအသုံးပြုသည့်
အတွဲ တစ်တွဲ ဖြစ်ပါစေ။
က ၏ ပထမ ကို ပြသမည်။
ခ ၏ ဒုတိယ ကို ပြသမည်။
ဂ ၏ ပထမ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မောင်မောင်
၂.၀
က
```

- The instance of a struct for a set of data kinds is a separate struct, named with those
  data kinds, such as `အတွဲ<text၊ isize>`, and messages and printed data kinds use that
  name.
- A struct whose instances each contain a deeper one, such as a field
  `နက်<T စာရင်း> လိပ်စာ` in `နက်<T>`, asks for instances with no end, and the compiler
  stops at a limit with ZK0342.
- The compiler checks the constraints of a generic struct for each instance, at its first
  use.

## Generic enumerations

An enumeration whose members hold data can have kind parameters too, in the forms of a
struct, with `ဒေတာအမျိုးအစားကွဲ` in place of `ဒေတာအစုအဝေး`. A member value works out its
data kinds from its data or from the place it goes into, and otherwise writes them before
the member in words or after the enumeration name in signs:
`စာသား ကို အစားထိုးအသုံးပြုသည့် ဖြစ်နိုင်များထဲမှ မတွေ့`, `ဖြစ်နိုင်<စာသား>များထဲမှ မတွေ့`.
[Generic enumerations](enums.md#generic-enumerations) gives the rules.

## Generic aliases

```text
ဒေတာအမျိုးအစား <alias> သည် ယေဘုယျဒေတာအမျိုးအစား <param> … ၎င်းကို <kind> ဟုလည်း မှတ်ယူနိုင်ပါသည်။   (word form)
ဒေတာအမျိုးအစား <alias><<param>၊ …> ကို <kind> ဟုလည်း မှတ်ယူနိုင်ပါသည်။                       (sign form)
```

An alias with kind parameters gets the data kinds of each use. In the word form, the kind
parameters follow `သည်`, as in a function, and `၎င်းကို` ("it") continues with the data kind
that the alias names.

```zaka
အတွဲ ဟူသော ဒေတာအစုအဝေးသည်
ယေဘုယျဒေတာအမျိုးအစား က နှင့် ယေဘုယျဒေတာအမျိုးအစား ခ တို့ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင်
ပထမ ဟူသော က တစ်ခု နှင့်
ဒုတိယ ဟူသော ခ တစ်ခု
တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ တွဲ ဖြစ်သည်။
ဒေတာအမျိုးအစား အမည်တွဲ သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍
၎င်းကို စာသား နှင့် T တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ ဟုလည်း မှတ်ယူနိုင်ပါသည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အရောင် သည် စာလုံး ကို အစားထိုးအသုံးပြုသည့် အမည်တွဲ တစ်တွဲ ဖြစ်၍ တန်ဖိုးသည်
ပထမ အနေဖြင့် "နီ" နှင့် ဒုတိယ အနေဖြင့် 'န' ရှိသော အတွဲ တစ်တွဲ ဖြစ်ပါစေ။
အရောင် ၏ ဒုတိယ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
န
```

The loader puts the data kind in place of each use, with the data kinds of the use in
place of the parameters, as for a plain [alias](aliases.md). A different count of data
kinds is ZK0340.

## The notes block

```text
ဤနေရာတွင် သတိပြုရန်မှာ
<name> သည် <kind> တစ်<word> ဖြစ်သည်။
<param> တွင် <trait> ဂုဏ်သတ္တိ ရှိသည်။
<param> တွင် <trait> ဂုဏ်သတ္တိ နှင့် <trait> ဂုဏ်သတ္တိ တို့ ရှိသည်။
သတိပြုစရာများ ပြီးဆုံးပါပြီး။
```

After the function signature, a generic function can have a notes block, and a
[trait](traits.md#declaring-a-trait) can have one after its first line.

- We can write `<name> သည် <kind> … ဖြစ်သည်` to declare a *local kind*: a data kind name
  for the whole function, its signature included, that means the data kind after it. A
  local kind can use the kind parameters and the local kinds above it. It is not a
  parameter, so a call never gives it, and it is counted with `ခု`, as a kind parameter
  is. A trait cannot declare local kinds.
- We can write `<param> တွင် … ရှိသည်` to give a kind parameter the traits that it requires,
  as `:` does in the sign form. Two or more traits form a list in words, with `တို့` before
  `ရှိသည်`.
- A note on a name that is not a kind parameter of the declaration is ZK0248.

```zaka
အတွဲ ဟူသော ဒေတာအစုအဝေးသည်
ယေဘုယျဒေတာအမျိုးအစား က နှင့် ယေဘုယျဒေတာအမျိုးအစား ခ တို့ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင်
ပထမ ဟူသော က တစ်ခု နှင့်
ဒုတိယ ဟူသော ခ တစ်ခု
တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ တွဲ ဖြစ်သည်။

လှည့် ဟူသော လုပ်ဆောင်ချက် သည်
ယေဘုယျဒေတာအမျိုးအစား A နှင့် ယေဘုယျဒေတာအမျိုးအစား B တို့ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
တွဲ ဟူသော C တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ D တစ်ခု ကို ပြန်ပေးမည်။
ဤနေရာတွင် သတိပြုရန်မှာ
C သည် A နှင့် B တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ တစ်တွဲ ဖြစ်သည်။
D သည် B နှင့် A တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ တစ်တွဲ ဖြစ်သည်။
သတိပြုစရာများ ပြီးဆုံးပါပြီး။
ပထမ အနေဖြင့် (တွဲ ၏ ဒုတိယ) နှင့် ဒုတိယ အနေဖြင့် (တွဲ ၏ ပထမ) ရှိသော အတွဲ တစ်တွဲ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရမှတ် သည် ပထမ အနေဖြင့် "မြမြ" နှင့် ဒုတိယ အနေဖြင့် ၉၀ ရှိသော အတွဲ တစ်တွဲ ဖြစ်ပါစေ။
လှည့်(ရမှတ်) ၏ ပထမ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၉၀
```

## Written-as hints

```text
… မျိုးပြပစ္စည်းမှာ <word> ဖြစ်ကာ ရေးသားရာတွင် <hint> ဟု ရေးသားနိုင်သည်။                  (struct)
… ဒေတာအမျိုးအစား ဖြစ်ပြီး ရေးသားရာတွင် <hint> ဟု ရေးသားနိုင်သည်။                         (data kind of the core module)
… (ကို|တို့ကို)အစားထိုးသုံးနိုင်၍ ရေးသားရာတွင် <hint> ဟု ရေးသားနိုင်ပြီး ၎င်းတွင် …            (trait)
<name><<A>၊ …> ဟူသည့် ဂုဏ်သတ္တိ သည် ရေးသားရာတွင် <hint> ဟု ရေးသားနိုင်ပြီး ၎င်းတွင် …      (trait, sign form)
```

A generic struct or trait can give the words that a use writes, with a *written-as hint* at the end
of its declaration. The hint adds a spelling, and the sign form and the long word form
still name the same data kind.

- Each word of the hint that is a kind parameter of the declaration is a *slot*, where a
  use writes a data kind. The other words are fixed words that a use writes unchanged.
- Each kind parameter appears in the hint at least once, in the order that the hint
  chooses, and a use gives the data kinds back to the parameters in their declared order.
- A parameter that appears twice gets one data kind, which a use writes in each place.
  So `ရေးသားရာတွင် က ကို က ဖြင့် တွဲသော ညီအတွဲဟု ရေးသားနိုင်သည်` gives
  `ကိန်းပြည့် ကို ကိန်းပြည့် ဖြင့် တွဲသော ညီအတွဲ`, and a different data kind in the second
  place is ZK0253.
- The hint ends with the name of the struct or trait, which appears nowhere else in it,
  and we can glue `ဟု` to that name: `အတွဲဟု`.
- A fixed word must come between two slots, because a data kind in a slot can itself be a
  written-as data kind, so two slots next to each other would be ambiguous.
- A struct gives its hint at the end of its measure-word clause, after `ဖြစ်ကာ`, so a
  struct with a hint must name its measure word.
- A trait leaves its first kind parameter out of the hint, because that is the data kind
  that has the trait, and the sentence around a use names it:
  `T တွင် စာသား အဖြစ် ပြောင်းနိုင်သော ဂုဏ်သတ္တိ ရှိသည်။` ([Traits with more kind
  parameters](traits.md#traits-with-more-kind-parameters)).
- A use reads the longest hint that fits there, and the data kind in a slot is the longest
  data kind that fits it.
- An imported struct or trait brings its hint with it.
- A hint that breaks one of these rules is ZK0252.

```zaka
အတွဲ ဟူသော ဒေတာအစုအဝေးသည်
ယေဘုယျဒေတာအမျိုးအစား က နှင့် ယေဘုယျဒေတာအမျိုးအစား ခ တို့ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင်
ပထမ ဟူသော က တစ်ခု နှင့်
ဒုတိယ ဟူသော ခ တစ်ခု
တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ တွဲ ဖြစ်ကာ ရေးသားရာတွင် က နှင့် ခ အတွဲဟု ရေးသားနိုင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် စာသား နှင့် ကိန်းပြည့် အတွဲ တစ်တွဲ ဖြစ်၍ တန်ဖိုးသည်
ပထမ အနေဖြင့် "မောင်မောင်" နှင့် ဒုတိယ အနေဖြင့် ၂၄ ရှိသော အတွဲ တစ်တွဲ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် စာသား ဖြင့် ညွှန်းသော စာသား နှင့် ကိန်းပြည့် အတွဲ ဇယား တစ်ခု ဖြစ်၍ တန်ဖိုးသည်
ဇယားအလွတ် ဖြစ်ပါစေ။
ခ ထဲမှ အညွှန်း "မ" ၏ တန်ဖိုး ကို က ဟု ထားမည်။
ခ ကို ပြသမည်။
ကိန်းရှင် ဂ သည် စာသား နှင့် ကိန်းပြည့် တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ တစ်တွဲ ဖြစ်၍ တန်ဖိုးသည်
က ဖြစ်ပါစေ။
ဂ ၏ ဒုတိယ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
["မ": အတွဲ<text၊ isize>(ပထမ: "မောင်မောင်"၊ ဒုတိယ: ၂၄)]
၂၄
```

The [list](lists.md) and the [table](tables.md) get their hints from the [core
module](core.md): `A စာရင်း` and `K ဖြင့် ညွှန်းသော V ဇယား`. `စာရင်း` and `ဇယား` are names,
not reserved words, so the long form and the sign form name them too:
`ကိန်းပြည့် ကို အစားထိုးအသုံးပြုသည့် စာရင်း`, `ဇယား<စာသား၊ ကိန်းပြည့်>`. A file that declares
a data kind with either name hides the one of the core module.

## Generics in modules

To let other files import a generic function, a generic struct or a trait, we mark it with
`အများသုံးနိုင်သော`, and other files import it with its name. An instance of a function belongs to
the file of its generic, so it sees each name of that file, private functions, variables
and constants included. It also sees the functions of the file of each of its data kinds
and of the file of each trait that it requires, so a generic in a module can call the trait
functions of the data kinds that it gets.

## How the compiler builds generics

The check runs again each time the program asks for new instances. Each run adds the
instances that the run before asked for, with the data kinds in place of the kind
parameters, and checks them as plain functions and structs, so the stages after the check
see no generics. An instance has a name that no program can write, made of the generic's
name and its data kinds: `ထပ်<isize>`, `အတွဲ<text၊ isize>`. The instance of a later generic
function with the same name keeps its number at the end, as in `ရွေး<isize>#2`, just as a
later plain function of a name is `<name>#2`, and a stop of the program prints the name
without the number. A chain of instances deeper than 64 stops with ZK0342. [Names and data
kinds](../internals/semcheck.md#generics) follows the build one step at a time.

## Errors

| Code | When |
| --- | --- |
| ZK0248 | a note on a name that is not a kind parameter |
| ZK0252 | a written-as hint that breaks a rule of its form |
| ZK0253 | two different data kinds in the places of one kind parameter that a hint names twice |
| ZK0304 | two generic functions with the same signature |
| ZK0332 | no generic function of the name fits the call |
| ZK0333 | more than one generic function fits the call |
| ZK0340 | a different count of data kinds from the declaration of the generic, or data kinds for a function that is not generic |
| ZK0341 | data kinds that a call or a struct value cannot work out |
| ZK0342 | instances with no end |

The errors of traits and trait blocks are on [Traits](traits.md#errors).
