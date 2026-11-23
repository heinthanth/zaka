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

## What a trait value cannot do

A trait value is used through the functions of its trait. These uses are ZK0361 for now:

- a comparison with `==`, `!=` or `တစ်ခုတည်း`;
- a print, and a template slot in text;
- the initial value, `ကနဦးတန်ဖိုး`;
- a call through it as a [function value](function-values.md);
- a call of its trait that runs as a [thread](threads.md), since the call passes its
  arguments in the frame of the caller; a function of our own that makes the call can run
  as the thread instead.

A trait value cannot go to C either: a C function or an export that takes or gives one is
ZK0465, as for each data kind that C cannot hold. Getting the held data kind back, a test
of it, and two traits at once are left for a later step of the language.

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
| ZK0361 | a trait value compared, printed, given an initial value or called as a function value, or a call of its trait run as a thread |
| ZK0362 | a plain value goes into a trait value whose trait has a function that changes its first parameter (warning, lint group `duplicates`) |
