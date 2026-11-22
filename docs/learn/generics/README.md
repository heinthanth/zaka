---
title: Generics
description: Write a function or a struct once for several data kinds with a kind parameter, and let the compiler build a version for each data kind that the program uses.
---

# Generics

A generic is a function or a struct that we write once for several data kinds (types).
In it, a kind parameter (type parameter) such as `T` stands for a data kind, and each use
of the generic fills in the data kind that it needs. So a function that returns the first
cell of a list can work for a list of numbers and for a list of text alike.

## You will learn

- How to write a generic function with a kind parameter.
- How to name the data kinds at a call, when the arguments do not show them.
- How to write a generic struct and a generic kind alias.
- How to give a long data kind a short name inside a function.

## A generic function

```zaka
ပထမကွက် ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
ကိန်းများ ဟူသော T စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းများ ၏ အကွက် ၀ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပထမကွက်([၇၊ ၈၊ ၉]) ကို ပြသမည်။
ပထမကွက်(["နီ"၊ "ဝါ"]) ကို ပြသမည်။
```

```text
၇
နီ
```

To make a function generic, we add `ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်`
after `သည်` in its function signature. Inside the function, `T` is a data kind like the
others, so we can write `T စာရင်း` and `T တစ်ခု`:

```
<name> ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား <T> ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
<parameters> လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ <kind> တစ်ခု ကို ပြန်ပေးမည်။
```

| Zaka | Reads as |
| --- | --- |
| `ယေဘုယျဒေတာအမျိုးအစား T` | "the general data kind `T`" |
| `ကိုအစားထိုးသုံးနိုင်၍` | "can be put in place, and" |
| `၎င်းသည်` | "it", the function, goes on with its parameters |

Each call fills in `T` with the data kind of its argument. The first call above passes a
list of integers, so `T` is `ကိန်းပြည့်`, and the second call passes a list of text, so
`T` is `စာသား`. The compiler builds an instance (monomorphized instance) of the function
for each data kind that the calls fill in, and it checks each instance as if we had
written it out.

To use two kind parameters, we join them with `နှင့်` and end with `တို့ကို`:
`ယေဘုယျဒေတာအမျိုးအစား A နှင့် ယေဘုယျဒေတာအမျိုးအစား B တို့ကိုအစားထိုးသုံးနိုင်၍`.

## Naming the data kinds at a call

Most of the time, the arguments of a call show the data kinds. When the arguments do not,
we can name the data kinds before the function with `<kind> ကို အစားထိုးအသုံးပြုသည့်`
("that puts `<kind>` in place"):

```
<kind> ကို အစားထိုးအသုံးပြုသည့် <name> ဟူသော လုပ်ဆောင်ချက်ကို <arguments> ဖြင့် လုပ်ဆောင်သော ရလဒ်
```

```zaka
ထပ် ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
က ဟူသော T တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ဒဿမကိန်း ကို အစားထိုးအသုံးပြုသည့် ထပ် ဟူသော လုပ်ဆောင်ချက်ကို ၂ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
```

```text
၂.၀
```

Here `T` is `ဒဿမကိန်း`, so the integer `၂` becomes the decimal number `၂.၀`. We need to
name the data kinds when a kind parameter appears in the result alone. Then the compiler
uses the data kind that the place of the call expects, and when there is no such place,
it stops with the error ZK0341:

```zaka-error ZK0341
ဗလာ ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
ကနဦးတန်ဖိုး ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ဗလာ() ကို ပြသမည်။
```

To fix it, we name the data kind at the call,
`ကိန်းပြည့် ကို အစားထိုးအသုံးပြုသည့် ဗလာ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ်`, or we
put the call in a place with a declared data kind:
`ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဗလာ() ဖြစ်ပါစေ။`

## Two generic functions of one name

Generic functions can share a name, just as plain functions can, and each call runs the
function that its arguments and prepositions fit. Here the stdlib function `ဖယ်` counts
cells from 0, and the program adds a second `ဖယ်` that counts from 1 and uses the
preposition `ကွက်မြောက်ကို`:

```zaka
"စံကား/စာရင်း" မှ ဖယ် ကို ထည့်သွင်းမည်။

ဖယ် ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
ပစ္စည်းများ ဟူသော T စာရင်း တစ်ခု ကို ဝိဘတ် "မှ" ကို သုံး၍ လည်းကောင်း
နေရာ ဟူသော လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို ဝိဘတ် "ကွက်မြောက်ကို" ကို သုံး၍ လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ T စာရင်း တစ်ခု ကို ပြန်ပေးမည်။
ပစ္စည်းများ မှ နေရာ - ၁ အကွက်ကို ဖယ် သည့် ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် l သည် [၁၀၊ ၂၀၊ ၃၀] ဖြစ်ပါစေ။
l မှ ၁ အကွက်ကို ဖယ် သည့် ရလဒ် ကို ပြသမည်။
l မှ ၁ ကွက်မြောက်ကို ဖယ် သည့် ရလဒ် ကို ပြသမည်။
```

```text
[၁၀၊ ၃၀]
[၂၀၊ ၃၀]
```

## Generic structs

A struct can have kind parameters too. We then end the first part of its declaration with
`ဒေတာအစုအဝေးသည်` and the kind parameters, and we use `၎င်းတွင်` before the fields:

```
<name> ဟူသော ဒေတာအစုအဝေးသည် ယေဘုယျဒေတာအမျိုးအစား <A> နှင့် ယေဘုယျဒေတာအမျိုးအစား <B> တို့ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင်
<fields> တို့ပါဝင်သည်။
```

To name the struct with its data kinds, we write
`<kind> နှင့် <kind> တို့ကို အစားထိုးအသုံးပြုသည့် <name>`. A value of the struct gets its
data kinds from its fields, or from the data kind that its place declares:

```zaka
အတွဲ ဟူသော ဒေတာအစုအဝေးသည် ယေဘုယျဒေတာအမျိုးအစား က နှင့် ယေဘုယျဒေတာအမျိုးအစား ခ တို့ကိုအစားထိုးသုံးနိုင်၍
၎င်းတွင်
ပထမ ဟူသော က တစ်ခု နှင့်
ဒုတိယ ဟူသော ခ တစ်ခု
တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ တွဲ ဖြစ်သည်။

လှည့် ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား A နှင့် ယေဘုယျဒေတာအမျိုးအစား B တို့ကိုအစားထိုးသုံးနိုင်၍
၎င်းသည်
တွဲ ဟူသော A နှင့် B တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ တစ်တွဲ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ B နှင့် A တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ တစ်တွဲ ကို ပြန်ပေးမည်။
ပထမ အနေဖြင့် (တွဲ ၏ ဒုတိယ) နှင့် ဒုတိယ အနေဖြင့် (တွဲ ၏ ပထမ) ရှိသော အတွဲ တစ်တွဲ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် လူ သည် ပထမ အနေဖြင့် "မောင်မောင်" နှင့် ဒုတိယ အနေဖြင့် ၂၄ ရှိသော အတွဲ တစ်တွဲ ဖြစ်ပါစေ။
လှည့်(လူ) ၏ ပထမ ကို ပြသမည်။
ကိန်းရှင် အမှတ် သည် ဒဿမကိန်း နှင့် ဒဿမကိန်း တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ တစ်တွဲ ဖြစ်၍ တန်ဖိုးသည် ပထမ အနေဖြင့် ၁ နှင့် ဒုတိယ အနေဖြင့် ၂ ရှိသော အတွဲ တစ်တွဲ ဖြစ်ပါစေ။
အမှတ် ၏ ဒုတိယ ကို ပြသမည်။
```

```text
၂၄
၂.၀
```

| Zaka | Reads as |
| --- | --- |
| `A နှင့် B တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ` | "the pair that puts `A` and `B` in place" |
| `တစ်တွဲ` | "one pair"; `တွဲ` is the measure word (classifier) of `အတွဲ` |

`လှည့်` works out its data kinds through the struct: its argument is a pair of `စာသား`
and `ကိန်းပြည့်`, so `A` is `စာသား` and `B` is `ကိန်းပြည့်`, and it returns the pair with
the two fields swapped. The variable `အမှတ်` names its data kinds, so its fields `၁` and `၂`
become decimal numbers.

## Written-as hints

A generic struct can also end its declaration with a written-as hint,
`ရေးသားရာတွင် <words> ဟု ရေးသားနိုင်သည်` ("when written, it can be written as …"). We put
the hint after `ဖြစ်ကာ` at the end of the declaration, and the kind parameters in the hint
mark the places where a use writes a data kind:

```
… တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ <word> ဖြစ်ကာ ရေးသားရာတွင် <words> ဟု ရေးသားနိုင်သည်။
```

```zaka
အတွဲ ဟူသော ဒေတာအစုအဝေးသည် ယေဘုယျဒေတာအမျိုးအစား က နှင့် ယေဘုယျဒေတာအမျိုးအစား ခ တို့ကိုအစားထိုးသုံးနိုင်၍
၎င်းတွင်
ပထမ ဟူသော က တစ်ခု နှင့်
ဒုတိယ ဟူသော ခ တစ်ခု
တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ တွဲ ဖြစ်ကာ ရေးသားရာတွင် က နှင့် ခ အတွဲဟု ရေးသားနိုင်သည်။

ကိန်းရှင် လူ သည် စာသား နှင့် ကိန်းပြည့် အတွဲ တစ်တွဲ ဖြစ်၍ တန်ဖိုးသည် ပထမ အနေဖြင့် "မောင်မောင်" နှင့် ဒုတိယ အနေဖြင့် ၂၄ ရှိသော အတွဲ တစ်တွဲ ဖြစ်ပါစေ။
လူ ၏ ပထမ ကို ပြသမည်။
```

```text
မောင်မောင်
```

`စာသား နှင့် ကိန်းပြည့် အတွဲ` is the same data kind as
`စာသား နှင့် ကိန်းပြည့် တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ`, because the hint just adds a
second spelling. The list and the table (map) have hints in the core module, which is the
reason we can write `ကိန်းပြည့် စာရင်း` and `စာသား ဖြင့် ညွှန်းသော ကိန်းပြည့် ဇယား`.

## Local names for data kinds

When a function signature has long data kinds, we can give them short names in a notes
block, `ဤနေရာတွင် သတိပြုရန်မှာ` ("here, to note:"), right after the function signature:

```
ဤနေရာတွင် သတိပြုရန်မှာ
<name> သည် <kind> တစ်ခု ဖြစ်သည်။
သတိပြုစရာများ ပြီးဆုံးပါပြီး။
```

```zaka
အတွဲ ဟူသော ဒေတာအစုအဝေးသည် ယေဘုယျဒေတာအမျိုးအစား က နှင့် ယေဘုယျဒေတာအမျိုးအစား ခ တို့ကိုအစားထိုးသုံးနိုင်၍
၎င်းတွင်
ပထမ ဟူသော က တစ်ခု နှင့်
ဒုတိယ ဟူသော ခ တစ်ခု
တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ တွဲ ဖြစ်သည်။

လှည့် ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား A နှင့် ယေဘုယျဒေတာအမျိုးအစား B တို့ကိုအစားထိုးသုံးနိုင်၍
၎င်းသည်
တွဲ ဟူသော C တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ D တစ်ခု ကို ပြန်ပေးမည်။
ဤနေရာတွင် သတိပြုရန်မှာ
C သည် A နှင့် B တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ တစ်တွဲ ဖြစ်သည်။
D သည် B နှင့် A တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ တစ်တွဲ ဖြစ်သည်။
သတိပြုစရာများ ပြီးဆုံးပါပြီး။
ပထမ အနေဖြင့် (တွဲ ၏ ဒုတိယ) နှင့် ဒုတိယ အနေဖြင့် (တွဲ ၏ ပထမ) ရှိသော အတွဲ တစ်တွဲ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ရမှတ် သည် ပထမ အနေဖြင့် "မြမြ" နှင့် ဒုတိယ အနေဖြင့် ၉၀ ရှိသော အတွဲ တစ်တွဲ ဖြစ်ပါစေ။
လှည့်(ရမှတ်) ၏ ပထမ ကို ပြသမည်။
```

```text
၉၀
```

`C` and `D` are not parameters but short names for the two pair kinds, and a call
does not fill them in. Like a kind parameter, a local name is counted with `ခု`, so we
write `C တစ်ခု`.

## Generic kind aliases

A [kind alias](../structs/README.md#another-name-for-a-kind) (type alias) can have kind
parameters as well. We write them after `သည်`, as in a function, and then use `၎င်းကို`
("it") before the data kind that the alias names:

```
ဒေတာအမျိုးအစား <name> သည် ယေဘုယျဒေတာအမျိုးအစား <T> ကိုအစားထိုးသုံးနိုင်၍
၎င်းကို <kind> ဟုလည်း မှတ်ယူနိုင်ပါသည်။
```

```zaka
အတွဲ ဟူသော ဒေတာအစုအဝေးသည် ယေဘုယျဒေတာအမျိုးအစား က နှင့် ယေဘုယျဒေတာအမျိုးအစား ခ တို့ကိုအစားထိုးသုံးနိုင်၍
၎င်းတွင်
ပထမ ဟူသော က တစ်ခု နှင့်
ဒုတိယ ဟူသော ခ တစ်ခု
တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ တွဲ ဖြစ်သည်။
ဒေတာအမျိုးအစား အမည်တွဲ သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍
၎င်းကို စာသား နှင့် T တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ ဟုလည်း မှတ်ယူနိုင်ပါသည်။

ကိန်းရှင် ရမှတ် သည် ကိန်းပြည့် ကို အစားထိုးအသုံးပြုသည့် အမည်တွဲ တစ်တွဲ ဖြစ်၍ တန်ဖိုးသည် ပထမ အနေဖြင့် "မြမြ" နှင့် ဒုတိယ အနေဖြင့် ၉၀ ရှိသော အတွဲ တစ်တွဲ ဖြစ်ပါစေ။
ရမှတ် ၏ ဒုတိယ ကို ပြသမည်။
```

```text
၉၀
```

Here `ကိန်းပြည့် ကို အစားထိုးအသုံးပြုသည့် အမည်တွဲ` is a short name for
`စာသား နှင့် ကိန်းပြည့် တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ`.

## Details

- **The sign form.** We can also write the kind parameters in `<` and `>` after the name,
  as in `ထပ်<T> ဟူသော လုပ်ဆောင်ချက် သည် …`, `အတွဲ<က၊ ခ> ဟူသော ဒေတာအစုအဝေးတွင် …` or
  `ဒေတာအမျိုးအစား အမည်တွဲ<T> ကို …`, and name the data kinds at a call after `::`, as in
  `ထပ်::<ဒဿမကိန်း>(၃)`. Both forms make the same generic, and these pages use the word
  form ([Generics and traits](../../manual/language/generics.md)).
- **Two functions that fit.** When two generic functions of one name fit a call alike, the
  call is the error ZK0333, and we then write its prepositions to pick one.
- **Measure words.** A kind parameter is counted with `ခု`, as in `T တစ်ခု`.
- **A hint that repeats.** A written-as hint can name a kind parameter twice, as
  `ရေးသားရာတွင် က ကို က ဖြင့် တွဲသော ညီအတွဲဟု ရေးသားနိုင်သည်` does, and a use then writes
  one data kind in both places: `ကိန်းပြည့် ကို ကိန်းပြည့် ဖြင့် တွဲသော ညီအတွဲ`
  ([Written-as hints](../../manual/language/generics.md#written-as-hints)).
- **Traits.** A kind parameter can also require functions of the data kinds that fill it
  in, which the next page shows ([Traits](traits.md)).

## Recap

- We add `ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်` to a function signature to
  make the function generic, and `T` is then a data kind inside it.
- Each call fills in the kind parameters from its arguments, or from
  `<kind> ကို အစားထိုးအသုံးပြုသည့်` before the function.
- A struct and a kind alias can have kind parameters too.
- A written-as hint gives a generic struct a shorter spelling, and a notes block gives long
  data kinds short local names.

## Hands on

- Write a generic function `နောက်ဆုံးကွက်` that returns the last cell of a list, and call
  it with a list of numbers and a list of text.
- Write a generic function that accepts two values of one data kind and returns a list of
  the two.
- Declare a generic struct `သေတ္တာ` with one field of data kind `T`, and print a `သေတ္တာ`
  of text and a `သေတ္တာ` of integers.

Next: [Traits](traits.md).
