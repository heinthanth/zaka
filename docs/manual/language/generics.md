---
title: Generics and traits
description: Functions, structs and aliases with kind parameters, the notes block, traits and trait blocks, and the errors of each.
---

# Generics and traits

A *generic* function, struct or kind alias takes *kind parameters*: names that stand
for kinds the code does not fix. Each use gives each kind parameter a kind, and the
compiler makes one *instance* of the generic for each set of kinds the program uses, as
if the code were written once for each. A *trait* names the functions a kind must have,
and a kind parameter can need traits. The Learn book shows them in
[Generics](../../learn/generics/README.md) and [Traits](../../learn/generics/traits.md).

```zaka
လုပ်ဆောင်ချက် ထပ်<T> သည်
က ဟူသော T တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ထပ်(၁) ကို ပြသမည်။
ထပ်("မင်္ဂလာပါ") ကို ပြသမည်။
ထပ်::<ဒဿမကိန်း>(၂.၅) ကို ပြသမည်။
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
| `စာသား ကို အစားထိုးအသုံးပြုသည့် အတွဲ` | "the pair that puts text in place" |
| `… ဂုဏ်သတ္တိ` | "the … trait" |
| `ဤနေရာတွင် သတိပြုရန်မှာ` | "here, to note:" |

## Kind parameters

A function, a struct and an alias name their kind parameters in one of two forms:

```text
<name><<param>၊ …>                                                (notation form)
<head> ယေဘုယျဒေတာအမျိုးအစား <param> ကိုအစားထိုးသုံးနိုင်၍ ၎င်း…     (word form)
```

- In the notation form the parameters stand in `<` and `>` after the name, with `၊`
  between them: `လုပ်ဆောင်ချက် အတွဲလုပ်<A၊ B> သည်`, `အတွဲ<က၊ ခ> ဟူသော ဒေတာအစုအဝေးတွင်`,
  `ဒေတာအမျိုးအစား အမည်ဇယား<T> ကို …`.
- In the word form each parameter is `ယေဘုယျဒေတာအမျိုးအစား <name>`, the items a
  [list in words](words.md#lists-in-words): `ကို` after one, `နှင့်` before the last and
  `တို့ကို` after two or more. Then `အစားထိုးသုံးနိုင်၍ ၎င်း` and the word the head
  goes on with: `သည်` for a function, `တွင်` for a struct.
- Inside the declaration a kind parameter is a kind name. In a slot it takes the measure
  word `ခု`, whatever kind it stands for: `T တစ်ခု`, `T စာရင်း တစ်ခု`.
- A kind parameter stands for a kind a program can write: a function kind can be one, a
  borrow or a raw address cannot (ZK0341), as no phrase names them.
- A parameter can need traits, after `:` in the notation form, two or more joined with
  `နှင့်`: `<T: နံပါတ်နှင့်တူသော>`, `<T: စီနိုင်သော နှင့် ပြနိုင်သော>`. The
  [notes block](#the-notes-block) gives them in either form.
- A generic is never checked as written: each instance is checked with its kinds in
  place. A fault inside a generic points at its code, with a label at the use that asked
  for the instance; a fault the instances share is reported once.

## Kinds at a use

```text
<name><<kind>၊ …>                                      (sign form)
<kind> [၊ <kind> …] [နှင့် <kind>] (ကို|တို့ကို) အစားထိုးအသုံးပြုသည့် <name>   (word form)
```

- A kind that names a generic struct or alias gives its kinds in the sign form, after
  the name with no space, or in the word form, before it:
  `အတွဲ<စာသား၊ ကိန်းပြည့်>`, `စာသား နှင့် ကိန်းပြည့် တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ`.
- Kinds nest: `အတွဲ<အတွဲ<စာသား၊ ကိန်းပြည့်>၊ ကိန်းပြည့်>`; inside kinds `>>` reads as two
  `>`. A kind in the word form with a list of its own, inside other kinds, stands in
  parentheses.
- Another count of kinds than the generic declares is ZK0340.

## Calls

```text
<f>::<<kind>၊ …>(<args>)                                                    (sign call)
<kinds> အစားထိုးအသုံးပြုသည့် <f> ဟူသော လုပ်ဆောင်ချက်ကို <args> ဖြင့် လုပ်ဆောင်…      (long call)
<kinds> အစားထိုးအသုံးပြု၍ <args> ကို သုံး၍ <f> မည်။                                (arguments first)
```

- A call with no kinds written works them out: first from the kinds of its arguments,
  matched against the parameter kinds through lists, addresses, optionals, results,
  ဇယား and generic structs; then from the kind its place expects, for a kind the
  result alone names.
- A kind the call cannot work out, or two arguments that give one kind parameter two
  kinds, is ZK0341; the call then writes its kinds.
- Kinds written for a function that is not generic are ZK0340.
- A function value of a generic names its kinds, `<f>::<<kind>၊ …>`, or takes them from
  the function kind its place expects.

```zaka
လုပ်ဆောင်ချက် ဗလာ<T> သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
ကနဦးတန်ဖိုး ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
// T comes from the kind the variable declares
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဗလာ() ဖြစ်ပါစေ။
က ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၀
```

```zaka-error ZK0341
လုပ်ဆောင်ချက် ဗလာ<T> သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
ကနဦးတန်ဖိုး ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဗလာ() ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## Generic structs

A struct with kind parameters is a pattern for structs. A value of one with no kinds
written takes them from the kind its place expects, or else from its field values;
`<struct>::<<kind>၊ …>(…)` and the word form `… ရှိသော <kinds> အစားထိုးအသုံးပြုသည့်
<struct> တစ်<word>` write them.

```zaka
အတွဲ<က၊ ခ> ဟူသော ဒေတာအစုအဝေးတွင်
ပထမ ဟူသော က တစ်ခု နှင့် ဒုတိယ ဟူသော ခ တစ်ခု
တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ တွဲ ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အတွဲ(ပထမ: "မောင်မောင်"၊ ဒုတိယ: ၂၄) ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အတွဲ<ဒဿမကိန်း၊ ဒဿမကိန်း> တစ်တွဲ ဖြစ်၍ တန်ဖိုးသည် အတွဲ(ပထမ: ၁၊ ဒုတိယ: ၂) ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် ပထမ အနေဖြင့် 'က' နှင့် ဒုတိယ အနေဖြင့် ၃ ရှိသော စာလုံး နှင့် ကိန်းပြည့် တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ တစ်တွဲ ဖြစ်ပါစေ။
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

- The instance of a struct for its kinds is a struct of its own, named with them, such
  as `အတွဲ<text၊ isize>`; messages and printed kinds use that name.
- A struct whose instances each hold a deeper one, such as a field
  `နက်<T စာရင်း> လိပ်စာ` in `နက်<T>`, asks for instances with no end; the compiler stops
  at a limit with ZK0342.
- The constraints of a generic struct are checked for each instance, at its first use.

## Generic aliases

An alias with kind parameters takes the kinds of each use:

```text
ဒေတာအမျိုးအစား <alias><<param>၊ …> ကို <kind> ဟုလည်း မှတ်ယူနိုင်ပါသည်။
```

```zaka
အတွဲ<က၊ ခ> ဟူသော ဒေတာအစုအဝေးတွင်
ပထမ ဟူသော က တစ်ခု နှင့် ဒုတိယ ဟူသော ခ တစ်ခု
တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ တွဲ ဖြစ်သည်။
ဒေတာအမျိုးအစား အမည်တွဲ<T> ကို စာသား နှင့် T တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ ဟုလည်း မှတ်ယူနိုင်ပါသည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အရောင် သည် အမည်တွဲ<စာလုံး> တစ်တွဲ ဖြစ်၍ တန်ဖိုးသည် အတွဲ(ပထမ: "နီ"၊ ဒုတိယ: 'န') ဖြစ်ပါစေ။
အရောင် ၏ ဒုတိယ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
န
```

The loader puts the kind in place of each use, with the kinds of the use in place of
the parameters, as for a plain [alias](aliases.md). Another count of kinds is ZK0340.

## The notes block

A generic function and a trait can follow their head with a notes block:

```text
ဤနေရာတွင် သတိပြုရန်မှာ
<name> သည် <kind> တစ်<word> ဖြစ်သည်။
<param> တွင် <trait> ဂုဏ်သတ္တိ ရှိသည်။
<param> တွင် <trait> ဂုဏ်သတ္တိ နှင့် <trait> ဂုဏ်သတ္တိ တို့ ရှိသည်။
သတိပြုစရာများ ပြီးဆုံးပါပြီး။
```

- `<name> သည် <kind> … ဖြစ်သည်` names a *local kind*: a kind name for the whole
  function, its head included, that stands for the kind after it. A local kind can name
  the kind parameters and the local kinds above it; it is no parameter, and a call never
  gives it. A trait names no local kinds.
- `<param> တွင် … ရှိသည်` gives a kind parameter the traits it needs, as `:` does in the
  notation form; two or more traits are a list in words, with `တို့` before `ရှိသည်`.
  A note on a name that is no kind parameter of the declaration is ZK0248.
- A local kind counts with `ခု`, as a kind parameter does.

```zaka
အတွဲ<က၊ ခ> ဟူသော ဒေတာအစုအဝေးတွင်
ပထမ ဟူသော က တစ်ခု နှင့် ဒုတိယ ဟူသော ခ တစ်ခု
တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ တွဲ ဖြစ်သည်။

လုပ်ဆောင်ချက် လှည့်<A၊ B> သည်
တွဲ ဟူသော C တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ D တစ်ခု ကို ပြန်ပေးမည်။
ဤနေရာတွင် သတိပြုရန်မှာ
C သည် A နှင့် B တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ တစ်တွဲ ဖြစ်သည်။
D သည် B နှင့် A တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ တစ်တွဲ ဖြစ်သည်။
သတိပြုစရာများ ပြီးဆုံးပါပြီး။
ပထမ အနေဖြင့် (တွဲ ၏ ဒုတိယ) နှင့် ဒုတိယ အနေဖြင့် (တွဲ ၏ ပထမ) ရှိသော အတွဲ တစ်တွဲ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လှည့်(အတွဲ(ပထမ: "မြမြ"၊ ဒုတိယ: ၉၀)) ၏ ပထမ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၉၀
```

## Traits

```text
<name> ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား <A> [… ] (ကို|တို့ကို)အစားထိုးသုံးနိုင်၍ ၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
<name><<A>၊ …> ဟူသည့် ဂုဏ်သတ္တိ တွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
[notes block]
<function heads>
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။
```

- A trait has one kind parameter or more. The first is the kind that has the trait; the
  others are kinds the trait names beside it, given where the trait is used:
  `ပြောင်းနိုင်သော<စာသား>` or `စာသား ကို သုံး၍ ပြောင်းနိုင်သော`.
- Each function head is a header with no block: `လုပ်ဆောင်ချက် <name> သည် <params>
  လက်ခံပြီး <kind> ကို ပြန်ပေးမည်။`, `… လက်ခံမည်။` for no result, or the result clause alone.
- The notes block of a trait gives its first kind parameter the traits it needs: a kind
  has the trait when it has those too.
- A kind *has* a trait when, for each function head, a function of the program has its
  name, and its parameter and result kinds with the kind in place of the first kind
  parameter and the given kinds in place of the others. The functions looked at are
  those of the file of the use, of the file of the kind and of the file of the trait. No
  block is needed: a plain function gives a kind a trait.
- Each constraint of a generic is checked where it is used, before the compiler makes
  the instance: a kind that lacks a trait is ZK0343, and a name that is no trait ZK0344.
  Two functions that each fit one function head are ZK0347.
- A trait stands at the top level of a file (ZK0247 inside a block), and `အများသုံး`
  before it lets other files import it.

```zaka
ငယ်စဉ်ကြီးလိုက်စီနိုင်သော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
လုပ်ဆောင်ချက် ငယ်သလား သည်
က ဟူသော A တစ်ခု ကို လည်းကောင်း ခ ဟူသော A တစ်ခု ကို လည်းကောင်း
လက်ခံပြီး မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ငယ်သလား သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
က < ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ငယ်ရာ<T: ငယ်စဉ်ကြီးလိုက်စီနိုင်သော> သည်
က ဟူသော T တစ်ခု ကို လည်းကောင်း ခ ဟူသော T တစ်ခု ကို လည်းကောင်း
လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ ငယ်သလား(က၊ ခ) ဖြစ်လျှင် က ကို ပြန်ပေးမည်။
ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ငယ်ရာ(၇၊ ၃) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၃
```

```zaka-error ZK0343
ငယ်စဉ်ကြီးလိုက်စီနိုင်သော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
လုပ်ဆောင်ချက် ငယ်သလား သည်
က ဟူသော A တစ်ခု ကို လည်းကောင်း ခ ဟူသော A တစ်ခု ကို လည်းကောင်း
လက်ခံပြီး မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ငယ်ရာ<T: ငယ်စဉ်ကြီးလိုက်စီနိုင်သော> သည်
က ဟူသော T တစ်ခု ကို လည်းကောင်း ခ ဟူသော T တစ်ခု ကို လည်းကောင်း
လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ ငယ်သလား(က၊ ခ) ဖြစ်လျှင် က ကို ပြန်ပေးမည်။
ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ငယ်ရာ("က"၊ "ခ") ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## Trait blocks

```text
<kind> တွင် <trait> ဂုဏ်သတ္တိရှိစေရန် အောက်ပါလုပ်ဆောင်ချက်များ ကို ထည့်သွင်းမည်။
<functions>
ထည့်သွင်းမည့် လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။
```

- A trait block gives a kind a trait, and checks that it does: each function head of the
  trait must be in the block, with the kind of the block in place of the first kind
  parameter (ZK0345 for one left out), and each function of the block must be one of the
  trait (ZK0346).
- The functions of a block are plain functions of the file: a call reaches them as it
  reaches the others, and the block adds nothing a plain function does not.
- A block stands in the file that declares its kind or its trait (ZK0348), and at the
  top level (ZK0247). A trait that the program does not declare is ZK0344.

## Built-in traits

The [prelude](prelude.md) declares the built-in traits, with the compiler mark: the
built-in kinds have them through the compiler's rule, and a struct through its
functions, which its operators then call.

| Trait | The kinds that have it |
| --- | --- |
| `ပေါင်းနိုင်သော` | the numbers and text, which `+` takes, and each kind with `ပေါင်း` |
| `နုတ်နိုင်သော`, `မြှောက်နိုင်သော`, `စားနိုင်သော` | the numbers, and each kind with `နုတ်`, `မြှောက်` or `စား` |
| `နံပါတ်နှင့်တူသော` | the kinds with the four traits above |
| `အကြွင်းရှာနိုင်သော` | the numbers, which `%` takes, and each kind with `အကြွင်းရှာ` |
| `ဘစ်ဆုံနိုင်သော`, `ဘစ်ပေါင်းစုနိုင်သော`, `ဘစ်ခြားနားနိုင်သော`, `ဘစ်ပြောင်းပြန်ပြောင်းလဲနိုင်သော` | the integers, which `&`, `\|`, `^` and `~` take, and each kind with `ဘစ်ဆုံ`, `ဘစ်ပေါင်းစု`, `ဘစ်ခြားနား` or `ဘစ်ပြောင်းပြန်` |
| `ဘစ်ဘယ်ရွေ့နိုင်သော`, `ဘစ်ညာရွေ့နိုင်သော` | the integers, which `<<` and `>>` take, and each kind with `ဘစ်ဘယ်ရွေ့` or `ဘစ်ညာရွေ့` |
| `အနုတ်ပြောင်းနိုင်သော` | the signed numbers, which the `-` of one operand takes, and each kind with `အနုတ်ပြောင်း` |
| `နှိုင်းယှဉ်နိုင်သော` | the numbers and the characters, and each kind with `ငယ်မငယ်စစ်` and `ကြီးမကြီးစစ်` |
| `တူမတူစစ်ဆေးနိုင်သော` | each kind, as `==` and `!=` take two values of one kind |
| `စာသားအဖြစ်ပြောင်းလဲပြသနိုင်သော` | each kind but a handle of a kind from outside, as `ပြသမည်` prints |

A trait the program declares with one of these names hides the built-in one.

## Generics in modules

`အများသုံး` before a generic function, a generic struct or a trait lets other files
import it with its name. An instance of a function stands in the file of its generic, so
it sees each name of that file, private functions, variables and constants too. It also
sees the functions of the file of each of its kinds and of the file of each trait it
needs, so a generic in a module calls the trait functions of the kinds it is given.

## How the compiler builds generics

The check runs again while the program asks for new instances. Each run adds the
instances the run before asked for, with the kinds in place of the kind parameters, and
checks them as plain functions and structs; the stages after the check see no generic.
An instance has a name no program can write, the generic's name and its kinds:
`ထပ်<isize>`, `အတွဲ<text၊ isize>`. A chain of instances deeper than 64 stops with ZK0342.
[Names and kinds](../internals/semcheck.md#generics) follows the build one step at a time.

## Errors

| Code | When |
| --- | --- |
| ZK0247 | a trait or a trait block inside a block |
| ZK0248 | a note on a name that is no kind parameter |
| ZK0340 | another count of kinds than the generic declares, or kinds for a function that is not generic |
| ZK0341 | kinds a call or a struct value cannot work out |
| ZK0342 | instances with no end |
| ZK0343 | a kind that lacks a trait a kind parameter needs |
| ZK0344 | a name that is no trait |
| ZK0345 | a trait block that lacks a function of its trait |
| ZK0346 | a function of a trait block that is not one of its trait |
| ZK0347 | two functions that give a kind one function of a trait |
| ZK0348 | a trait block outside the file of its kind and of its trait |
| ZK0349 | the compiler mark outside the [prelude](prelude.md) |
