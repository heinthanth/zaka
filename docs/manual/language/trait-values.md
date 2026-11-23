---
title: Trait values
description: The data kind of a trait, <trait> အရာ, which holds a value of each data kind with the trait, the calls that run the function of the held data kind, changes through a trait value, which traits can be a data kind, and the errors.
---

# Trait values

A trait value (interface value) holds a value of each data kind (type) that has a
[trait](traits.md), and a call on it runs the function of the data kind that it holds. The
data kind of a trait value is the trait with `အရာ` ("thing") after it, as in
`နှုတ်ဆက်နိုင်သော အရာ`, "a thing that can greet". So one list can hold a `ကျောင်းသား` and a
`ဆရာ` together, and the program picks the function of each one while it runs, as a call
through an interface does in Go. The Learn book teaches trait values in
[Trait values](../../learn/generics/trait-values.md).

```zaka
နှုတ်ဆက်နိုင်သော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍
၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော A တစ်ခု ကို လက်ခံပြီး စာသား တစ်ခု ကို ပြန်ပေးမည်။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

ကျောင်းသား ဟူသော ဒေတာအစုအဝေးတွင် နာမည် ဟူသော စာသား တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။
ဆရာ ဟူသော ဒေတာအစုအဝေးတွင် ဘာသာ ဟူသော စာသား တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကျောင်းသား တစ်ယောက် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"ကျောင်းသား {က ၏ နာမည်}" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ဆရာ တစ်ယောက် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"{က ၏ ဘာသာ} ဆရာ" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် လူများ သည် နှုတ်ဆက်နိုင်သော အရာ စာရင်း တစ်ခု ဖြစ်၍
တန်ဖိုးသည် [ကျောင်းသား(နာမည်: "မောင်မောင်")၊ ဆရာ(ဘာသာ: "သင်္ချာ")] ဖြစ်ပါစေ။
လူများ ၏ အကွက်တိုင်း ကို လ အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လ ၏ နှုတ်ဆက်() ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
ကျောင်းသား မောင်မောင်
သင်္ချာ ဆရာ
```

| Zaka | Reads as |
| --- | --- |
| `<trait> အရာ` | "a thing that has `<trait>`" |
| `အရာ` | "thing", the head noun of a trait value |

## The data kind

`<trait> အရာ` stands wherever a data kind stands: a variable, a parameter, a field, a cell
of a list, an optional, a result, an alias, and a data kind given to a generic. It counts
with `ခု`, as in `နှုတ်ဆက်နိုင်သော အရာ တစ်ခု`. A trait with more kind parameters writes its
other kinds as a constraint writes them, so `စာသား အဖြစ် ပြောင်းနိုင်သော အရာ` holds a value of
each data kind that converts to text. A noun name for a trait value, such as `လူ` for
`နှုတ်ဆက်နိုင်သော အရာ`, comes from a [kind alias](aliases.md), which can name its own measure
word:

```zaka-file
// people.zaka
ဒေတာအမျိုးအစား လူ ကို နှုတ်ဆက်နိုင်သော အရာ ဟုလည်း မှတ်ယူနိုင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။
```

`အရာ` is not a reserved word. It reads as the head noun right after a trait, where a data
kind stands, and remains free to use as a name elsewhere.

## Putting a value in

A value of a data kind goes into a place of a trait value kind when its data kind has the
trait: through its own functions, a trait block, or the defaults that a trait block takes
([When a data kind has a trait](traits.md#when-a-data-kind-has-a-trait)). This works in each
place where a value goes to a data kind: a declaration, an assignment, an argument, a
return, a cell of a list literal, a field of a struct literal, and an add to a list. A
value whose data kind lacks the trait is ZK0343. A trait value of one trait never converts
to a trait value of another trait.

The trait value holds the value as a variable holds it: a plain value (a number, text, a
list or a struct) goes in as a duplicate (copy), and a
[shared object](objects.md) goes in as a reference to the same object.

## Calls

`x ၏ f(a)`, `f(x၊ a)` and `x ၏ f` call the function `f` of the trait on a trait value `x`,
and run the `f` of the data kind that `x` holds, with the held value as the first argument.
The other arguments follow the parameters of the trait function, with the trait value kind
in place of the trait's data kind. A plain function `f` whose first parameter is the trait
value kind itself wins over the call through the trait value.

A trait value has its own trait, so it goes to a generic whose kind parameter requires that
trait, and the calls inside the instance run through it:

```zaka
ဖော်ပြ ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
က ဟူသော T တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
ဤနေရာတွင် သတိပြုရန်မှာ
T တွင် နှုတ်ဆက်နိုင်သော ဂုဏ်သတ္တိ ရှိသည်။
သတိပြုစရာများ ပြီးဆုံးပါပြီး။
"[{နှုတ်ဆက်(က)}]" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

နှုတ်ဆက်နိုင်သော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍
၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော A တစ်ခု ကို လက်ခံပြီး စာသား တစ်ခု ကို ပြန်ပေးမည်။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"နံပါတ် {က}" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် တ သည် နှုတ်ဆက်နိုင်သော အရာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇ ဖြစ်ပါစေ။
ဖော်ပြ(တ) ကို ပြသမည်။
```

```text
[နံပါတ် ၇]
```

## Results of the trait's data kind

A trait function can give back the trait's own data kind as its whole result, as in
`နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် သည် ပ ဟူသော A တစ်ခု ကို လက်ခံပြီး A တစ်ခု ကို ပြန်ပေးမည်။`. A call
of it through a trait value gives back a new trait value, which holds the value that the
function of the held data kind returned.

## Changes through a trait value

A trait function can take its first parameter with `ဒေတာ ပြောင်းလဲရန်`, so the function of the
held data kind changes the held value. A call of it through a trait value changes the
value that the trait value holds, and nothing else: the program puts a new trait value with
the changed value in the place of the old one. Each other trait value, and the value that
went in, keeps the value it had. A shared object goes in as a reference, so a change
through the trait value reaches the object, and each name of the object sees it.

Since a change to a plain value reaches the duplicate in the trait value alone, the lint
warns where a plain value goes into a trait value whose trait has such a function
(ZK0362). The warning belongs to the lint group `duplicates`: `--lint-allow duplicates` turns
it off, and `--lint-error duplicates` makes it an error
([Commands](../compiler/commands.md)).

```zaka-warning ZK0362
တိုးနိုင်သော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍
၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
တိုး ဟူသော လုပ်ဆောင်ချက် သည်
ပ ဟူသော A တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

ကောင်တာ ဟူသော ဒေတာအစုအဝေးတွင် အကြိမ် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

တိုး ဟူသော လုပ်ဆောင်ချက် သည်
ပ ဟူသော ကောင်တာ တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ပ ၏ အကြိမ် ကို (ပ ၏ အကြိမ်) + ၁ ဟု ထားမည်။
ပ ၏ အကြိမ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် က သည် ကောင်တာ(အကြိမ်: ၀) ဖြစ်ပါစေ။
ကိန်းရှင် တ သည် တိုးနိုင်သော အရာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။
တ ၏ တိုး() ကို ပြသမည်။
```

With the group allowed, the same program runs, and `က` keeps `၀`:

```zaka lint-allow=duplicates
တိုးနိုင်သော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍
၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
တိုး ဟူသော လုပ်ဆောင်ချက် သည်
ပ ဟူသော A တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

ကောင်တာ ဟူသော ဒေတာအစုအဝေးတွင် အကြိမ် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

တိုး ဟူသော လုပ်ဆောင်ချက် သည်
ပ ဟူသော ကောင်တာ တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ပ ၏ အကြိမ် ကို (ပ ၏ အကြိမ်) + ၁ ဟု ထားမည်။
ပ ၏ အကြိမ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် က သည် ကောင်တာ(အကြိမ်: ၀) ဖြစ်ပါစေ။
ကိန်းရှင် တ သည် တိုးနိုင်သော အရာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။
တ ၏ တိုး() ကို ပြသမည်။
တ ၏ တိုး() ကို ပြသမည်။
က ၏ အကြိမ် ကို ပြသမည်။
```

```text
၁
၂
၀
```

## Which traits can be a data kind

A trait can be a data kind when the call can pick its function through the first argument.
So each function of the trait takes the trait's data kind as its first parameter, takes it
in no other parameter, gives it back as its whole result or not at all, and has no kind
parameters of its own. A trait that breaks one of these rules is ZK0360 where it is used as
a data kind, and so is a trait that the compiler gives, such as the operator traits of the
[core module](core.md), which have no functions to run:

```zaka-error ZK0360
ငယ်စဉ်ကြီးလိုက်စီနိုင်သော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍
၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
ငယ်သလား ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော A တစ်ခု ကို လည်းကောင်း
ခ ဟူသော A တစ်ခု ကို လည်းကောင်း လက်ခံပြီး မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

ကိန်းရှင် တ သည် ငယ်စဉ်ကြီးလိုက်စီနိုင်သော အရာ စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် စာရင်းအလွတ် ဖြစ်ပါစေ။
```

`ငယ်သလား` takes two values of the trait's data kind, and a trait value holds one, so the
call has no single data kind to pick its function from.

## Two traits at once

A trait value can hold a value of each data kind that has two traits or more, written with
`နှင့်` between the traits: `နှုတ်ဆက်နိုင်သော နှင့် အမည်ရှိသော အရာ`. A value goes in when its
data kind has each of the traits, and a call on the trait value runs the function of the
trait that has its name. Each trait must be able to be a data kind on its own (ZK0360). Such a trait value has
each of its traits, so it goes to a generic that requires one of them; a trait value of
other traits, or of the traits in another order, is another data kind.

```zaka
နှုတ်ဆက်နိုင်သော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍
၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော A တစ်ခု ကို လက်ခံပြီး စာသား တစ်ခု ကို ပြန်ပေးမည်။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

အမည်ရှိသော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍
၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
အမည်ယူ ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော A တစ်ခု ကို လက်ခံပြီး စာသား တစ်ခု ကို ပြန်ပေးမည်။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

ကျောင်းသား ဟူသော ဒေတာအစုအဝေးတွင် နာမည် ဟူသော စာသား တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။
ဆရာ ဟူသော ဒေတာအစုအဝေးတွင် ဘာသာ ဟူသော စာသား တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကျောင်းသား တစ်ယောက် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"ကျောင်းသား {က ၏ နာမည်}" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ဆရာ တစ်ယောက် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"{က ၏ ဘာသာ} ဆရာ" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

အမည်ယူ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကျောင်းသား တစ်ယောက် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
က ၏ နာမည် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

အမည်ယူ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ဆရာ တစ်ယောက် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"ဆရာ" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် လူများ သည် နှုတ်ဆက်နိုင်သော နှင့် အမည်ရှိသော အရာ စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည်
[ကျောင်းသား(နာမည်: "မောင်မောင်")၊ ဆရာ(ဘာသာ: "သင်္ချာ")] ဖြစ်ပါစေ။
လူများ ၏ အကွက်တိုင်း ကို လ အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"{လ ၏ အမည်ယူ()}: {လ ၏ နှုတ်ဆက်()}" ဟု ပြသမည်။
အကယ်၍ လ သည် ဆ ဟူသည့် ဆရာ အဖြစ်သို့ ပြောင်းလဲနိုင်လျှင် ဆ ၏ ဘာသာ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
မောင်မောင်: ကျောင်းသား မောင်မောင်
ဆရာ: သင်္ချာ ဆရာ
သင်္ချာ
```

## Testing the held data kind

A trait value can be asked which data kind it holds, with the words of an
[enumeration whose members hold data](enums.md#members-that-hold-data), where the name of a
data kind with the trait stands for the member:

- `<x> သည် <kind> အဖြစ်သို့ ပြောင်းလဲနိုင်သည်` tests whether `x` holds a value of `<kind>`;
- `<x> ကို <kind> အဖြစ် ပြောင်းလဲသော ရလဒ်` reads the held value as `<kind>`, and the run
  stops with `wrong member` when `x` holds another data kind;
- `<x> သည် <name> ဟူသည့် <kind> အဖြစ်သို့ ပြောင်းလဲနိုင်လျှင်`, as the condition of `အကယ်၍`
  or as the first arm of a switch, names the held value in its block;
- a switch over a trait value takes the name of a data kind as a case, as in
  `<kind> ဖြစ်လျှင်`, and a default arm.

The data kind is a struct, an enumeration, an alias or a data kind name of the compiler. A
name that names no data kind is ZK0318, and a data kind without the trait is ZK0343, since
no trait value of the trait can hold one.

```zaka
နှုတ်ဆက်နိုင်သော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍
၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော A တစ်ခု ကို လက်ခံပြီး စာသား တစ်ခု ကို ပြန်ပေးမည်။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

ကျောင်းသား ဟူသော ဒေတာအစုအဝေးတွင် နာမည် ဟူသော စာသား တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။
ဆရာ ဟူသော ဒေတာအစုအဝေးတွင် ဘာသာ ဟူသော စာသား တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကျောင်းသား တစ်ယောက် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"ကျောင်းသား {က ၏ နာမည်}" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ဆရာ တစ်ယောက် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"{က ၏ ဘာသာ} ဆရာ" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ဖော်ပြ ဟူသော လုပ်ဆောင်ချက် သည်
လ ဟူသော နှုတ်ဆက်နိုင်သော အရာ တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
လ သည် ကျ ဟူသည့် ကျောင်းသား အဖြစ်သို့ ပြောင်းလဲနိုင်လျှင် "ကျောင်းသား {ကျ ၏ နာမည်}" ကို ပြန်ပေးမည်။
ဆရာ ဖြစ်လျှင် "ဆရာ" ကို ပြန်ပေးမည်။
တခြား နှုတ်ဆက်နိုင်သော အရာ တစ်ခု ဖြစ်လျှင် "တခြား" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် လူများ သည် နှုတ်ဆက်နိုင်သော အရာ စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည်
[ကျောင်းသား(နာမည်: "မောင်မောင်")၊ ဆရာ(ဘာသာ: "သင်္ချာ")] ဖြစ်ပါစေ။
လူများ ၏ အကွက်တိုင်း ကို လ အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လ သည် ဆရာ အဖြစ်သို့ ပြောင်းလဲနိုင်သည် ဟု ပြသမည်။
ဖော်ပြ(လ) ကို ပြသမည်။
အကယ်၍ လ သည် ဆ ဟူသည့် ဆရာ အဖြစ်သို့ ပြောင်းလဲနိုင်လျှင် ဆ ၏ ဘာသာ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ကိန်းရှင် ပ သည် လူများ ၏ အကွက် ၀ ဖြစ်ပါစေ။
(ပ ကို ကျောင်းသား အဖြစ် ပြောင်းလဲသော ရလဒ်) ၏ နာမည် ကို ပြသမည်။
```

```text
မမှန်
ကျောင်းသား မောင်မောင်
မှန်
ဆရာ
သင်္ချာ
မောင်မောင်
```

## Comparing and printing

Two trait values of one trait value kind compare through the values the two hold:

- `==` is true when both hold a value of one data kind and the two values are equal, as
  `==` compares them; values of two data kinds are never equal, and `!=` is the opposite
  of `==`;
- `X နှင့် Y သည် တစ်ခုတည်း` is true when both hold one and the same
  [shared object](objects.md) or function value. A plain value goes in as a
  duplicate, so two trait values that hold plain values are never the same.

A print, or a template slot in text, shows the held value as a print of it would: through
the `စာသားအဖြစ်ပြောင်း` of its struct when the struct has one
([Conversions](conversions.md)).

A trait value inside a list, a struct, an object, a table or an optional compares and prints
through its held value too, as each cell and field does: `[က၊ ဂ]` prints as
`[လူ(နာမည်: "မောင်")၊ ကြောင်(နာမည်: "မောင်")]`, and two lists of trait values are equal when
each pair of held values is. Inside another value, a held struct prints in the plain form,
as a struct in a list does, without its `စာသားအဖြစ်ပြောင်း`. An object that reaches itself
through a trait value prints and compares as each ring does ([Values and
objects](objects.md)).

```zaka
အမည်ရှိသော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍
၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
အမည်ယူ ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော A တစ်ခု ကို လက်ခံပြီး စာသား တစ်ခု ကို ပြန်ပေးမည်။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

လူ ဟူသော ဒေတာအစုအဝေးတွင် နာမည် ဟူသော စာသား တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။
ကြောင် ဟူသော ဒေတာအစုအဝေးတွင် နာမည် ဟူသော စာသား တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ကောင် ဖြစ်သည်။

အမည်ယူ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော လူ တစ်ယောက် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
က ၏ နာမည် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

အမည်ယူ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကြောင် တစ်ကောင် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
က ၏ နာမည် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

စာသားအဖြစ်ပြောင်း ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကြောင် တစ်ကောင် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"ကြောင် {က ၏ နာမည်}" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အမည်ရှိသော အရာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် လူ(နာမည်: "မောင်") ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အမည်ရှိသော အရာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် လူ(နာမည်: "မောင်") ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် အမည်ရှိသော အရာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကြောင်(နာမည်: "မောင်") ဖြစ်ပါစေ။
က == ခ ကို ပြသမည်။
က == ဂ ကို ပြသမည်။
က နှင့် ခ သည် တစ်ခုတည်း ကို ပြသမည်။
က ကို ပြသမည်။
"[{ဂ}]" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
မမှန်
မမှန်
လူ(နာမည်: "မောင်")
[ကြောင် မောင်]
```

A trait value holds a value from the start, so it has no initial value. A place that is
blank at first is an [optional](optionals.md) trait value,
`<trait> အရာ ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`, which starts as `အလွတ်` and takes a value of
each data kind with the trait.

## Ordering

A trait value has no order of its own, since two of them can hold values of two data kinds.
As for a struct, `<` and `>` call a `ငယ်မငယ်စစ်` or a `ကြီးမကြီးစစ်` that the program
declares for the two operand kinds ([Built-in traits](traits.md#built-in-traits)), in the
file of the trait or of the use; with neither, the ordering is ZK0361. The function decides
how two data kinds compare, often through a function of the trait, and the other operand
can be of another data kind, such as a number. With both functions for two trait values,
the trait value kind has `နှိုင်းယှဉ်နိုင်သော`, so a list of trait values sorts.

```zaka
လူနှင့်တူသော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍
၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
အသက်ယူ ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော A တစ်ခု ကို လက်ခံပြီး ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

ကျောင်းသား ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

အသက်ယူ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကျောင်းသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က ၏ အသက် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ငယ်မငယ်စစ် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော လူနှင့်တူသော အရာ တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
(က ၏ အသက်ယူ()) < ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် လူနှင့်တူသော အရာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကျောင်းသား(အသက်: ၁၉) ဖြစ်ပါစေ။
က < ၃၀ ကို ပြသမည်။
က >= ၃၀ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
မမှန်
```

## Trait values and threads

A trait value can hold a plain shared object, which two threads could write at once, so a
plain trait value does not go to a [thread](threads.md) (ZK0494). Joined with the built-in
trait of threads, as in `နှုတ်ဆက်နိုင်သော နှင့် အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော အရာ`,
a trait value takes values of data kinds that threads can use alone (ZK0343 for another
data kind), and it goes to a thread as each such value does. A call of its trait then runs
as a thread too, unless the trait function changes or fills a parameter (ZK0498). The
trait of threads gives no function, so alone it is no trait value kind (ZK0360).

```zaka
နှုတ်ဆက်နိုင်သော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍
၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော A တစ်ခု ကို လက်ခံပြီး စာသား တစ်ခု ကို ပြန်ပေးမည်။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

လူ ဟူသော ဒေတာအစုအဝေးတွင် နာမည် ဟူသော စာသား တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော လူ တစ်ယောက် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"မင်္ဂလာပါ {က ၏ နာမည်}" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် တ သည် နှုတ်ဆက်နိုင်သော နှင့် အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော အရာ တစ်ခု ဖြစ်၍
တန်ဖိုးသည် လူ(နာမည်: "မောင်") ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည်
နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက်ကို တ ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
စောင့်(ခ) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ မောင်
```

## What a trait value cannot do

A trait value is used through the functions of its trait, and through the comparisons and
the print above. These uses are ZK0361:

- an ordering, `<`, `>`, `<=` or `>=`, with no `ငယ်မငယ်စစ်` or `ကြီးမကြီးစစ်` for its operand
  kinds;
- the initial value, `ကနဦးတန်ဖိုး`;
- a call through it as a [function value](function-values.md).

A trait value cannot go to C either: a C function or an export that takes or gives one is
ZK0465, as for each data kind that C cannot hold.

```zaka-error ZK0361
နှုတ်ဆက်နိုင်သော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍
၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော A တစ်ခု ကို လက်ခံပြီး စာသား တစ်ခု ကို ပြန်ပေးမည်။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

ကိန်းရှင် တ သည် နှုတ်ဆက်နိုင်သော အရာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
```

## How a trait value runs

A trait value is a closure: a counted heap block that holds the value, and the code of a
dispatcher, which the compiler makes once for each pair of trait value kind and held data
kind. A call passes the number of the trait function and its other arguments to the
dispatcher, which calls the function of the held data kind. A duplicate of a trait value
shares the block, so it costs a count, and the ring collector and the drops of the VM, C and
native code handle it as each other closure. Putting a value in makes one block, and a call
that changes the held value makes a new block for the changed value. The details are on
[The IR](../internals/codegen.md#trait-values).

## Errors

| Code | When |
| --- | --- |
| ZK0343 | a value goes into a trait value whose trait its data kind lacks |
| ZK0344 | a trait value of a name that is not a trait |
| ZK0360 | a trait that cannot be a data kind used as one |
| ZK0361 | a trait value ordered with no function for it, given an initial value, or called as a function value |
| ZK0362 | a plain value goes into a trait value whose trait has a function that changes its first parameter (warning, lint group `duplicates`) |
