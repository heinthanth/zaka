---
title: Trait values
description: Keep values of different data kinds in one list through their trait, call the function of each one's own data kind, and see what a change through a trait value reaches.
---

# Trait values

A generic makes one instance for each data kind, so a list of `T` still holds values of
one data kind. A trait value (interface value) lifts that limit: its data kind is a trait
with `အရာ` ("thing") after it, and it holds a value of each data kind that has the trait. A
call on a trait value runs the function of the data kind it holds, picked while the
program runs, the same as a call through an interface in Go.

## You will learn

- How to write the data kind of a trait value and put values of different data kinds in it.
- How a call on a trait value picks the function of the held data kind.
- What a function that changes its first parameter does through a trait value.
- Which traits can be a data kind.

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

// one list, two data kinds: each cell is a thing that can greet
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

`ကျောင်းသား` (student) and `ဆရာ` (teacher) each have the trait `နှုတ်ဆက်နိုင်သော` through their
own `နှုတ်ဆက်` function, so both fit in a list of `နှုတ်ဆက်နိုင်သော အရာ`. In the loop, the call
`လ ၏ နှုတ်ဆက်()` runs the `နှုတ်ဆက်` of the student for the first cell and the one of the
teacher for the second. We can use this pattern to write the data kind of a trait value:

```text
<name> ဟူသော <trait> အရာ တစ်ခု
```

| Zaka | Reads as |
| --- | --- |
| `နှုတ်ဆက်နိုင်သော` | "that can greet", the trait |
| `အရာ` | "thing" |
| `နှုတ်ဆက်နိုင်သော အရာ တစ်ခု` | "one thing that can greet" |

## Values go in, calls go through

A value goes into a trait value wherever its data kind has the trait: in a declaration, an
assignment, an argument, a return, or a cell of a list. So a function that takes a trait
value works for each data kind with the trait, and so does a function that walks a list of
them:

```zaka
ပုံသဏ္ဌာန်ရှိသော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍
၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
ဧရိယာ ဟူသော လုပ်ဆောင်ချက် သည် ပ ဟူသော A တစ်ခု ကို လက်ခံပြီး ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

စတုရန်း ဟူသော ဒေတာအစုအဝေးတွင် ဘေး ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။
ထောင့်မှန်စတုဂံ ဟူသော ဒေတာအစုအဝေးတွင်
အလျား ဟူသော ကိန်းပြည့် တစ်ခု နှင့်
အနံ ဟူသော ကိန်းပြည့် တစ်ခု
တို့ပါဝင်သည်။

ဧရိယာ ဟူသော လုပ်ဆောင်ချက် သည်
ပ ဟူသော စတုရန်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
(ပ ၏ ဘေး) * (ပ ၏ ဘေး) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ဧရိယာ ဟူသော လုပ်ဆောင်ချက် သည်
ပ ဟူသော ထောင့်မှန်စတုဂံ တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
(ပ ၏ အလျား) * (ပ ၏ အနံ) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// the sum of the areas, for shapes of each data kind
စုစုပေါင်း ဟူသော လုပ်ဆောင်ချက် သည်
ပုံများ ဟူသော ပုံသဏ္ဌာန်ရှိသော အရာ စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် ပေါင်း သည် ၀ ဖြစ်ပါစေ။
ပုံများ ၏ အကွက်တိုင်း ကို ပ အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပေါင်း ကို ပေါင်း + (ပ ၏ ဧရိယာ()) ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပေါင်း ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

စုစုပေါင်း([စတုရန်း(ဘေး: ၃)၊ ထောင့်မှန်စတုဂံ(အလျား: ၂၊ အနံ: ၅)]) ကို ပြသမည်။
```

```text
၁၉
```

A value of a data kind without the trait does not go in: the compiler stops with ZK0343, the
same error as for a generic. A trait value also has its own trait, so it can go to a
generic that requires that trait.

## Changes go to the duplicate

A plain value goes into a trait value as a duplicate (copy), the same as when it goes into
a variable. So a function of the trait that changes its first parameter, marked with
`ဒေတာ ပြောင်းလဲရန်`, changes the duplicate in the trait value, and the value we put in keeps
its old state. A reader can miss this, so the compiler warns where such a value goes in
(ZK0362). Here we allow the lint group `duplicates` to show the result:

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
က ၏ အကြိမ် ကို ပြသမည်။ // the counter we put in is still at 0
```

```text
၁
၂
၀
```

When `ကောင်တာ` is a [shared object](../structs/objects.md), declared with
`မျှဝေနိုင်သော ဒေတာအစုအဝေး`, the trait value holds the same object as `က`, so each change shows
through `က` as well, and the compiler gives no warning.

## Traits that can be a data kind

A call on a trait value picks its function through the one value it holds, so each
function of the trait must take the trait's data kind as its first parameter, and nowhere
else except as its whole result. A function such as `ငယ်သလား`, which compares two values of
the trait's data kind, breaks the rule, and the compiler stops with ZK0360 when we use its
trait as a data kind:

```zaka-error ZK0360
ငယ်စဉ်ကြီးလိုက်စီနိုင်သော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍
၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
ငယ်သလား ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော A တစ်ခု ကို လည်းကောင်း
ခ ဟူသော A တစ်ခု ကို လည်းကောင်း လက်ခံပြီး မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

ကိန်းရှင် တ သည် ငယ်စဉ်ကြီးလိုက်စီနိုင်သော အရာ စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် စာရင်းအလွတ် ဖြစ်ပါစေ။
```

## Details

- **Results of the trait's data kind.** A function of the trait that returns the trait's
  data kind gives back a new trait value through the call
  ([Trait values](../../manual/language/trait-values.md#results-of-the-traits-data-kind)).
- **Comparing and printing.** `==` compares the held values, `တစ်ခုတည်း` asks whether two
  trait values hold one shared object, and a print shows the held value
  ([Trait values](../../manual/language/trait-values.md#comparing-and-printing)).
- **Threads.** `နှုတ်ဆက်နိုင်သော နှင့် အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော အရာ` holds values
  that threads can use, so it goes to a thread, and a call of its trait runs as one
  ([Trait values](../../manual/language/trait-values.md#trait-values-and-threads)).
- **Ordering.** `<` on trait values calls a `ငယ်မငယ်စစ်` declared for the two operand kinds,
  which can also be a trait value and a number
  ([Trait values](../../manual/language/trait-values.md#ordering)).
- **Not now.** A trait value has no initial value (ZK0361); an optional trait value starts
  blank. A list of trait values prints and compares through the held values.
- **The held data kind.** `လ သည် ကျောင်းသား အဖြစ်သို့ ပြောင်းလဲနိုင်သည်` tests which data kind
  a trait value holds, and `လ ကို ကျောင်းသား အဖြစ် ပြောင်းလဲသော ရလဒ်` reads it, with the
  words of members that hold data
  ([Trait values](../../manual/language/trait-values.md#testing-the-held-data-kind)).
- **Two traits.** `နှုတ်ဆက်နိုင်သော နှင့် အမည်ရှိသော အရာ` holds a value of each data kind with
  both traits, and runs the functions of both
  ([Trait values](../../manual/language/trait-values.md#two-traits-at-once)).
- **Names.** A [kind alias](../../manual/language/aliases.md) gives a trait value a noun
  name, such as `လူ`, with a measure word of its own.
- **Cost.** Putting a value into a trait value makes one counted block, and a duplicate of
  the trait value shares it.

## Recap

- `<trait> အရာ` is the data kind of a trait value, which holds a value of each data kind
  with the trait.
- A call on a trait value runs the function of the data kind it holds.
- A plain value goes in as a duplicate, so a change through the trait value reaches the
  duplicate; a shared object goes in as itself.
- Each function of the trait takes the trait's data kind first and nowhere else, except as
  its whole result.

## Hands on

- Add a third shape, `တြိဂံ`, with its own `ဧရိယာ`, and put one in the list of the second
  example.
- Give `ပုံသဏ္ဌာန်ရှိသော` a function `အမည်` that returns text, and print the name and the area
  of each shape.
- Make `ကောင်တာ` a shared object in the last example and read what `က ၏ အကြိမ်` prints.

Next: [Modules](../modules/README.md).
