---
title: Traits
description: List the functions that a data kind must have in a trait, give a data kind the trait with a trait block, and let a generic require it.
---

# Traits

A trait (interface) is a list of functions that a data kind must have. A kind parameter
can require a trait, and it then accepts just the data kinds that have those functions.
So a generic can compare two values of a struct, for example, once the struct has a
function that compares them.

## You will learn

- How to declare a trait and give a data kind the trait with a trait block.
- How to require a trait in a generic function.
- How a trait can give a function a default.
- Which traits the core module has, and how a struct can use operators such as `+`.

## A trait, a trait block and a requirement

A generic that finds the smallest cell of a list must compare two cells. Numbers compare
with `<`, but a struct of the program compares in the form that we choose. Here the trait
`ငယ်စဉ်ကြီးလိုက်စီနိုင်သော` asks for a function `ငယ်သလား` that checks whether one value is
smaller than another:

```zaka
ငယ်စဉ်ကြီးလိုက်စီနိုင်သော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍
၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
ငယ်သလား ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော A တစ်ခု ကို လည်းကောင်း
ခ ဟူသော A တစ်ခု ကို လည်းကောင်း လက်ခံပြီး မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

လူ ဟူသော ဒေတာအစုအဝေးတွင်
နာမည် ဟူသော စာသား တစ်ခု နှင့်
အသက် ဟူသော ကိန်းပြည့် တစ်ခု
တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

လူ တွင် ငယ်စဉ်ကြီးလိုက်စီနိုင်သော ဂုဏ်သတ္တိရှိစေရန် အောက်ပါလုပ်ဆောင်ချက်များ ကို ထည့်သွင်းမည်။
ငယ်သလား ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော လူ တစ်ယောက် ကို လည်းကောင်း
ခ ဟူသော လူ တစ်ယောက် ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
(က ၏ အသက်) < (ခ ၏ အသက်) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ထည့်သွင်းမည့် လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

အငယ်ဆုံး ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
ကိန်းများ ဟူသော T စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
ဤနေရာတွင် သတိပြုရန်မှာ
T တွင် ငယ်စဉ်ကြီးလိုက်စီနိုင်သော ဂုဏ်သတ္တိ ရှိသည်။
သတိပြုစရာများ ပြီးဆုံးပါပြီး။
ကိန်းရှင် အငယ် သည် ကိန်းများ ၏ အကွက် ၀ ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက်တိုင်း ကို ဝ အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ ငယ်သလား(ဝ၊ အငယ်) ဖြစ်လျှင် အငယ် ကို ဝ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အငယ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် လူများ သည် [လူ(နာမည်: "အေးအေး"၊ အသက်: ၃၀)၊ လူ(နာမည်: "မောင်မောင်"၊ အသက်: ၂၄)] ဖြစ်ပါစေ။
အငယ်ဆုံး(လူများ) ၏ နာမည် ကို ပြသမည်။
```

```text
မောင်မောင်
```

The program has three parts, and each one has a form.

**The trait.** We declare a trait with kind parameters, as a generic has, and the first
one, `A`, is the data kind that has the trait. Inside, we list the function signatures,
each ending at its result, without a block:

```
<trait> ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
<function signatures>
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။
```

**The trait block.** To give a data kind the trait, we write the functions of the trait
for that data kind inside a trait block, much like `impl` in Rust. The compiler checks
that the block has each function of the trait, with `လူ` in place of `A`:

```
<kind> တွင် <trait> ဂုဏ်သတ္တိရှိစေရန် အောက်ပါလုပ်ဆောင်ချက်များ ကို ထည့်သွင်းမည်။
<functions>
ထည့်သွင်းမည့် လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။
```

**The requirement.** To require the trait in a generic, we write a notes block right after
its function signature, much like `where T: …` in Rust. Each call checks the requirement
for the data kind that it fills in for `T`, before the compiler builds the function:

```
ဤနေရာတွင် သတိပြုရန်မှာ
<T> တွင် <trait> ဂုဏ်သတ္တိ ရှိသည်။
သတိပြုစရာများ ပြီးဆုံးပါပြီး။
```

| Zaka | Reads as |
| --- | --- |
| `ငယ်စဉ်ကြီးလိုက်စီနိုင်သော ဟူသည့် ဂုဏ်သတ္တိ` | "the trait called 'can be put in order from small to big'" |
| `၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်` | "it must have the functions below" |
| `လူ တွင် … ဂုဏ်သတ္တိရှိစေရန်` | "so that `လူ` has the trait …" |
| `အောက်ပါလုပ်ဆောင်ချက်များ ကို ထည့်သွင်းမည်` | "add the functions below" |
| `T တွင် … ဂုဏ်သတ္တိ ရှိသည်` | "`T` has the trait …" |

## A data kind without a trait block

A data kind has a trait once the program has the functions of the trait for it, with a
trait block or without one. For example, a plain function `ငယ်သလား` that accepts two
`ကိန်းပြည့်` values gives `ကိန်းပြည့်` the trait. The trait block is a check: it lists
which functions belong to the trait, and the compiler makes sure that each one is there.

When a data kind lacks a function of the trait, the call reports it and names the
function. Here `ဒဿမကိန်း` has no `ငယ်သလား`:

```zaka-error ZK0343
ငယ်စဉ်ကြီးလိုက်စီနိုင်သော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
ငယ်သလား ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော A တစ်ခု ကို လည်းကောင်း ခ ဟူသော A တစ်ခု ကို လည်းကောင်း
လက်ခံပြီး မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

ငယ်ရာ ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
က ဟူသော T တစ်ခု ကို လည်းကောင်း ခ ဟူသော T တစ်ခု ကို လည်းကောင်း
လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
ဤနေရာတွင် သတိပြုရန်မှာ
T တွင် ငယ်စဉ်ကြီးလိုက်စီနိုင်သော ဂုဏ်သတ္တိ ရှိသည်။
သတိပြုစရာများ ပြီးဆုံးပါပြီး။
အကယ်၍ ငယ်သလား(က၊ ခ) ဖြစ်လျှင် က ကို ပြန်ပေးမည်။
ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ငယ်ရာ(၁.၅၊ ၂.၅) ကို ပြသမည်။
```

## Defaults

A function of a trait can come with a block, which we call its default. A data kind takes
the defaults when its trait block ends with a line that asks for them:

```
<trait> ဂုဏ်သတ္တိ၏ အခြားလုပ်ဆောင်ချက်များကိုလည်း မူရင်းအတိုင်း ဆက်ခံမည်။
```

Each function that the trait block leaves out then runs its default, and a function that
the trait block writes replaces the default. Here `လူ` takes the default greeting, while
`ကြောင်` writes its own:

```zaka
နှုတ်ဆက်နိုင်သော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍
၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
နာမည်ယူ ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော A တစ်ခု ကို လက်ခံပြီး စာသား တစ်ခု ကို ပြန်ပေးမည်။
နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော A တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"မင်္ဂလာပါ " + နာမည်ယူ(က) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

လူ ဟူသော ဒေတာအစုအဝေးတွင် နာမည် ဟူသော စာသား တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။
ကြောင် ဟူသော ဒေတာအစုအဝေးတွင် နာမည် ဟူသော စာသား တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ကောင် ဖြစ်သည်။

လူ တွင် နှုတ်ဆက်နိုင်သော ဂုဏ်သတ္တိရှိစေရန် အောက်ပါလုပ်ဆောင်ချက်များ ကို ထည့်သွင်းမည်။
နာမည်ယူ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော လူ တစ်ယောက် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
က ၏ နာမည် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
နှုတ်ဆက်နိုင်သော ဂုဏ်သတ္တိ၏ အခြားလုပ်ဆောင်ချက်များကိုလည်း မူရင်းအတိုင်း ဆက်ခံမည်။
ထည့်သွင်းမည့် လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

ကြောင် တွင် နှုတ်ဆက်နိုင်သော ဂုဏ်သတ္တိရှိစေရန် အောက်ပါလုပ်ဆောင်ချက်များ ကို ထည့်သွင်းမည်။
နာမည်ယူ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကြောင် တစ်ကောင် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
က ၏ နာမည် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကြောင် တစ်ကောင် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"ညောင် " + နာမည်ယူ(က) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ထည့်သွင်းမည့် လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

နှုတ်ဆက်(လူ(နာမည်: "အေးအေး")) ကို ပြသမည်။
နှုတ်ဆက်(ကြောင်(နာမည်: "ဝါဝါ")) ကို ပြသမည်။
```

```text
မင်္ဂလာပါ အေးအေး
ညောင် ဝါဝါ
```

| Zaka | Reads as |
| --- | --- |
| `အခြားလုပ်ဆောင်ချက်များကိုလည်း` | "the other functions as well" |
| `မူရင်းအတိုင်း ဆက်ခံမည်` | "inherit as in the original" |

## Built-in traits

The core module (prelude), which each file can use without an import, has a trait for
each operator. A data kind has the trait of an operator when we can use that operator on
it:

| Trait | Data kinds |
| --- | --- |
| `ပေါင်းနိုင်သော` | numbers and text, which `+` works on |
| `နုတ်နိုင်သော`, `မြှောက်နိုင်သော`, `စားနိုင်သော` | numbers, which `-`, `*` and `/` work on |
| `နံပါတ်နှင့်တူသော` | the data kinds with the four traits above |
| `အကြွင်းရှာနိုင်သော` | numbers, which `%` works on |
| `ဘစ်ဆုံနိုင်သော`, `ဘစ်ပေါင်းစုနိုင်သော`, `ဘစ်ခြားနားနိုင်သော`, `ဘစ်ပြောင်းပြန်ပြောင်းလဲနိုင်သော` | integers, which `&`, `\|`, `^` and `~` work on |
| `ဘစ်ဘယ်ရွေ့နိုင်သော`, `ဘစ်ညာရွေ့နိုင်သော` | integers, which `<<` and `>>` work on |
| `အနုတ်ပြောင်းနိုင်သော` | signed numbers, which the `-` before one operand works on |
| `နှိုင်းယှဉ်နိုင်သော` | numbers and characters, which `<`, `>`, `<=` and `>=` work on |
| `တူမတူစစ်ဆေးနိုင်သော` | all data kinds, because `==` compares two values of one data kind |
| `စာသားအဖြစ်ပြောင်းလဲပြသနိုင်သော` | all data kinds that `ပြသမည်` prints |
| `ပြောင်းလဲနိုင်သော`, `စမ်းသပ်ပြောင်းလဲနိုင်သော` | the data kinds that convert to another, without a failure and with one ([Converting between data kinds](../operators/conversion.md)) |

So a generic can require a built-in trait in the same form as a trait of the program:

```zaka
စုစုပေါင်း ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
ကိန်းများ ဟူသော T စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
ဤနေရာတွင် သတိပြုရန်မှာ
T တွင် နံပါတ်နှင့်တူသော ဂုဏ်သတ္တိ ရှိသည်။
သတိပြုစရာများ ပြီးဆုံးပါပြီး။
ကိန်းရှင် ပေါင်း သည် T တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက်တိုင်း ကို ဝ အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပေါင်း ကို ပေါင်း + ဝ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပေါင်း ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

စုစုပေါင်း([၁၊ ၂၊ ၃]) ကို ပြသမည်။
စုစုပေါင်း([၁.၅၊ ၂.၅]) ကို ပြသမည်။
```

```text
၆
၄.၀
```

Text has `+` but no `-`, `*` or `/`, so it does not have `နံပါတ်နှင့်တူသော`, and a call
such as `စုစုပေါင်း(["က"])` stops with the error ZK0343 before the compiler builds the
function for text.

## Operators on a struct

A struct can use `+` once the program has the function that `+` calls, `ပေါင်း`, for two
values of the struct. In the same form, `<` calls `ငယ်မငယ်စစ်`, `==` calls
`ညီမညီစစ်`, and printing calls `စာသားအဖြစ်ပြောင်း`. The struct then has the trait of the
operator too, so a generic that requires the trait accepts the struct:

```zaka
ဗက်တာ ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု နှင့် y ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

ပေါင်း ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ဗက်တာ တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ဗက်တာ တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ဗက်တာ တစ်ခု ကို ပြန်ပေးမည်။
ဗက်တာ(x: (က ၏ x) + (ခ ၏ x)၊ y: (က ၏ y) + (ခ ၏ y)) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

စာသားအဖြစ်ပြောင်း ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ဗက်တာ တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"<{က ၏ x}၊ {က ၏ y}>" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

စုစုပေါင်း ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
ပထမ ဟူသော T တစ်ခု ကို လည်းကောင်း
ကျန် ဟူသော T စာရင်း တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
ဤနေရာတွင် သတိပြုရန်မှာ
T တွင် ပေါင်းနိုင်သော ဂုဏ်သတ္တိ ရှိသည်။
သတိပြုစရာများ ပြီးဆုံးပါပြီး။
ကိန်းရှင် ပေါင်းပြီး သည် ပထမ ဖြစ်ပါစေ။
ကျန် ၏ အကွက်တိုင်း ကို ဝ အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပေါင်းပြီး ကို ပေါင်းပြီး + ဝ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပေါင်းပြီး ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် က သည် ဗက်တာ(x: ၁၊ y: ၂) ဖြစ်ပါစေ။
က + က ကို ပြသမည်။
စုစုပေါင်း(က၊ [က၊ က]) ကို ပြသမည်။
စုစုပေါင်း("က"၊ ["ခ"]) ကို ပြသမည်။
```

```text
<၂၊ ၄>
<၃၊ ၆>
ကခ
```

The same generic `စုစုပေါင်း` adds vectors with our `ပေါင်း` and joins text with the `+`
of the core module.

## Traits that require traits

A trait can require other traits, in a notes block right after its first line. A data
kind then has the trait once it has those traits as well:

```text
စီနိုင်သောနံပါတ် ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
ဤနေရာတွင် သတိပြုရန်မှာ
A တွင် နံပါတ်နှင့်တူသော ဂုဏ်သတ္တိ နှင့် ငယ်စဉ်ကြီးလိုက်စီနိုင်သော ဂုဏ်သတ္တိ တို့ ရှိသည်။
သတိပြုစရာများ ပြီးဆုံးပါပြီး။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။
```

To require two traits at once, we join them with `နှင့်` and end with `တို့ ရှိသည်`, the
same as in the notes block of a generic function.

## Details

- **Defaults need the line.** A trait block without the `ဆက်ခံမည်` line must write each
  function of the trait, the defaults too (ZK0345), and a data kind without a trait block
  gets no defaults at all. A trait block that has the line alone takes each default.
- **The sign form.** In the sign form, `<T: နံပါတ်နှင့်တူသော>` after the name of a generic
  means the same as `T တွင် နံပါတ်နှင့်တူသော ဂုဏ်သတ္တိ ရှိသည်။` in its notes block
  ([Generics and traits](../../manual/language/generics.md)).
- **Operators.** The full list of the functions that operators call is on [The core
  module](../../manual/language/core.md).
- **Threads.** The built-in trait `အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော` marks the data
  kinds that a [thread](../threads/README.md) can use.

## Recap

- A trait lists the functions that a data kind must have, and its first kind parameter is
  the data kind that has the trait.
- A trait block writes those functions for one data kind, and the compiler checks that
  each one is there.
- A notes block after a function signature, `T တွင် <trait> ဂုဏ်သတ္တိ ရှိသည်။`, requires
  the trait for `T`.
- A trait can give its functions defaults, which a trait block takes with the
  `ဆက်ခံမည်` line.
- The core module has a trait for each operator, and a struct gets one when the program has
  the function that the operator calls.

## Hands on

- Declare a trait `အရွယ်ရှိသော` with a function `အရွယ်` that returns an integer, give it to
  a struct, and write a generic that returns the biggest value of a list.
- Give `ဗက်တာ` the function `ငယ်မငယ်စစ်`, then compare two vectors with `<`.
- Remove one function from the trait block of `လူ` in the first example and read the error.

Next: [Modules](../modules/README.md).
