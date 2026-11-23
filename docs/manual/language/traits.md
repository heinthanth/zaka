---
title: Traits
description: Declaring a trait, when a data kind has one, default functions, traits with more kind parameters, trait blocks, the built-in traits, and their errors.
---

# Traits

A trait (interface) lists the functions that a data kind (type) must have, and a kind
parameter of a [generic](generics.md) can require traits, so that the generic can call
those functions on the data kinds that it gets. A data kind has a trait when the program
has a function for each function in the trait, and a trait block lets us give a data kind
those functions together, with a check that none is missing. The Learn book teaches traits
in small steps in [Traits](../../learn/generics/traits.md).

```zaka
ငယ်စဉ်ကြီးလိုက်စီနိုင်သော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍
၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
ငယ်သလား ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော A တစ်ခု ကို လည်းကောင်း
ခ ဟူသော A တစ်ခု ကို လည်းကောင်း လက်ခံပြီး မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

ငယ်သလား ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
က < ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ငယ်ရာ ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
က ဟူသော T တစ်ခု ကို လည်းကောင်း
ခ ဟူသော T တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
ဤနေရာတွင် သတိပြုရန်မှာ
T တွင် ငယ်စဉ်ကြီးလိုက်စီနိုင်သော ဂုဏ်သတ္တိ ရှိသည်။
သတိပြုစရာများ ပြီးဆုံးပါပြီး။
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

| Zaka | Reads as |
| --- | --- |
| `… ဟူသည့် ဂုဏ်သတ္တိ` | "the trait called …" |
| `ဂုဏ်သတ္တိ` | "attribute", the word for a trait |
| `T တွင် … ဂုဏ်သတ္တိ ရှိသည်` | "T has the … trait" |

## Declaring a trait

```text
<name> ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား <A> [… ] (ကို|တို့ကို)အစားထိုးသုံးနိုင်၍ [ရေးသားရာတွင် <hint> ဟု ရေးသားနိုင်ပြီး] ၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
<name><<A>၊ …> ဟူသည့် ဂုဏ်သတ္တိ တွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
<name><<A>၊ …> ဟူသည့် ဂုဏ်သတ္တိ သည် ရေးသားရာတွင် <hint> ဟု ရေးသားနိုင်ပြီး ၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
[notes block]
<function signatures>
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။
```

To declare a trait, we write its name, `ဟူသည့် ဂုဏ်သတ္တိ သည်`, its kind parameters as for a
[generic function](generics.md#kind-parameters), and then
`၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။`. Below that line we list the functions of
the trait, and we end the trait with `ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။`.

- A trait has one kind parameter or more. The first one is the data kind that has the
  trait, and the others are data kinds that the trait names beside it, which a use of the
  trait gives ([Traits with more kind parameters](#traits-with-more-kind-parameters)).
- Each function of the trait is a function signature with no block:
  `<name> ဟူသော လုပ်ဆောင်ချက် သည် <params> လက်ခံပြီး <kind> ကို ပြန်ပေးမည်။`, or
  `… လက်ခံမည်။` for a function with no result, or the result clause alone for a function
  with no parameters. A function written in full, with a block, is a [default
  function](#default-functions).
- A trait can have a [notes block](generics.md#the-notes-block) after its first line,
  which gives its first kind parameter the traits that it requires, so a data kind has the
  trait when it also has those. A trait cannot declare local kinds.
- We declare a trait at the top level of a file (ZK0247 inside a block), and to let other
  files import it, we put `အများသုံးနိုင်သော` before it.

## When a data kind has a trait

A data kind has a trait when the program has a function for each function signature of
the trait, with the same name, and with the data kind in place of the first kind parameter
and the given data kinds in place of the others in its parameter and result data kinds. The
compiler looks at the functions of the file of the use, of the file of the data kind and
of the file of the trait. No block is needed, because a plain function gives a data kind a
trait, as `ငယ်သလား` gives `ကိန်းပြည့်` the trait in the first example. A trait whose functions
take its data kind first can also be a data kind, `<trait> အရာ`, which holds a value of each
data kind with the trait ([Trait values](trait-values.md)).

The compiler checks each constraint of a generic where we use the generic, before it
builds the instance:

| Mistake | Code |
| --- | --- |
| a data kind that does not have a trait that a kind parameter requires | ZK0343 |
| a name that is not a trait | ZK0344 |
| two functions that each fit one function signature of a trait | ZK0347 |

Text does not have the trait of the first example, because no function `ငယ်သလား` takes two
text values:

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

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ငယ်ရာ("က"၊ "ခ") ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## Default functions

When we write a function of a trait in full, with `အောက်ပါတို့ကို လုပ်ဆောင်ကာ` and a block,
the function has a *default*. A data kind whose [trait block](#trait-blocks) takes the
defaults runs the default when the data kind does not give its own function with that name.

- The default is a generic function of the file, with the kind parameters of the trait,
  and its first kind parameter requires the trait.
- `အများသုံးနိုင်သော` on the trait exports the defaults too.
- A default counts for a data kind through a trait block alone. A data kind with no block
  has the trait when it gives each function itself, defaults included.

The [Trait blocks](#trait-blocks) example below has a default, `နှုတ်ဆက်`.

## Traits with more kind parameters

The kind parameters after the first one are data kinds that the trait names beside the
data kind that has it. A use gives them in the sign form, `ပြောင်းနိုင်သော<စာသား>`, in the
word form, `စာသား ကို အစားထိုးအသုံးပြုသည့် ပြောင်းနိုင်သော`, or with the words of the trait's
[written-as hint](generics.md#written-as-hints), which leaves out the first kind parameter.
Here, the hint `B အဖြစ် ပြောင်းနိုင်သော` lets us write `T တွင် စာသား အဖြစ် ပြောင်းနိုင်သော
ဂုဏ်သတ္တိ ရှိသည်`:

```zaka
ပြောင်းနိုင်သော ဟူသည့် ဂုဏ်သတ္တိ သည်
ယေဘုယျဒေတာအမျိုးအစား A နှင့် ယေဘုယျဒေတာအမျိုးအစား B တို့ကိုအစားထိုးသုံးနိုင်၍
ရေးသားရာတွင် B အဖြစ် ပြောင်းနိုင်သောဟု ရေးသားနိုင်ပြီး ၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
ပြောင်း ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော A တစ်ခု ကို လက်ခံပြီး B တစ်ခု ကို ပြန်ပေးမည်။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

ပြောင်း ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော မှန်မှားတန်ဖိုး တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ က ဖြစ်လျှင် "ဟုတ်" ကို ပြန်ပေးမည်။
"မဟုတ်" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ဖြေ ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
က ဟူသော T တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
ဤနေရာတွင် သတိပြုရန်မှာ
T တွင် စာသား အဖြစ် ပြောင်းနိုင်သော ဂုဏ်သတ္တိ ရှိသည်။
သတိပြုစရာများ ပြီးဆုံးပါပြီး။
ပြောင်း(က) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဖြေ(မှန်) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဟုတ်
```

## Trait blocks

```text
<kind> တွင် <trait> ဂုဏ်သတ္တိရှိစေရန် အောက်ပါလုပ်ဆောင်ချက်များ ကို ထည့်သွင်းမည်။
<functions>
[<trait> ဂုဏ်သတ္တိ၏ အခြားလုပ်ဆောင်ချက်များကိုလည်း မူရင်းအတိုင်း ဆက်ခံမည်။]
ထည့်သွင်းမည့် လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။
```

We can use a trait block (`impl` block) to give a data kind a trait and have the compiler
check that it does. Each function signature of the trait must be in the block, with the
data kind of the block in place of the first kind parameter, and each function of the
block must be one of the trait. Apart from that check, the functions of a block are plain
functions of the file: a call reaches them as it reaches the others, and the block adds
nothing that a plain function does not.

To take the defaults, we end the block with
`<trait> ဂုဏ်သတ္တိ၏ အခြားလုပ်ဆောင်ချက်များကိုလည်း မူရင်းအတိုင်း ဆက်ခံမည်။`. The data kind then
gets the default of each function of the trait that the block leaves out, and a block with
this line alone takes all the defaults. A function of the data kind in the block replaces
the default. Without the line, a function left out is ZK0345, whether it has a default or
not.

| Mistake | Code |
| --- | --- |
| a block that leaves out a function of its trait, with no line that takes the defaults | ZK0345 |
| a function in the block that is not one of the trait | ZK0346 |
| a block outside the file of its data kind and the file of its trait | ZK0348 |
| a block inside another block | ZK0247 |
| a block for a trait that the program does not declare | ZK0344 |
| a line that takes the defaults of a different trait from the block | ZK0254 |
| a line that takes no default, because the block gives each function that has one (warning) | ZK0351 |

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

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
နှုတ်ဆက်(လူ(နာမည်: "အေးအေး")) ကို ပြသမည်။
နှုတ်ဆက်(ကြောင်(နာမည်: "ဝါဝါ")) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ အေးအေး
ညောင် ဝါဝါ
```

`လူ` takes the default `နှုတ်ဆက်`, while `ကြောင်` gives its own.

## Built-in traits

The [core module](core.md) declares the built-in traits with the compiler mark (ZK0349
outside the core module). The built-in data kinds have them through a rule of the
compiler, and a struct has them through its functions, which its operators then call.

| Trait | The data kinds that have it |
| --- | --- |
| `ပေါင်းနိုင်သော` | the numbers and text, which `+` works on, and each data kind with `ပေါင်း` |
| `နုတ်နိုင်သော`, `မြှောက်နိုင်သော`, `စားနိုင်သော` | the numbers, and each data kind with `နုတ်`, `မြှောက်` or `စား` |
| `နံပါတ်နှင့်တူသော` | the data kinds with the four traits above |
| `အကြွင်းရှာနိုင်သော` | the numbers, which `%` works on, and each data kind with `အကြွင်းရှာ` |
| `ဘစ်ဆုံနိုင်သော`, `ဘစ်ပေါင်းစုနိုင်သော`, `ဘစ်ခြားနားနိုင်သော`, `ဘစ်ပြောင်းပြန်ပြောင်းလဲနိုင်သော` | the integers, which `&`, `\|`, `^` and `~` work on, and each data kind with `ဘစ်ဆုံ`, `ဘစ်ပေါင်းစု`, `ဘစ်ခြားနား` or `ဘစ်ပြောင်းပြန်` |
| `ဘစ်ဘယ်ရွေ့နိုင်သော`, `ဘစ်ညာရွေ့နိုင်သော` | the integers, which `<<` and `>>` work on, and each data kind with `ဘစ်ဘယ်ရွေ့` or `ဘစ်ညာရွေ့` |
| `အနုတ်ပြောင်းနိုင်သော` | the signed numbers, which the `-` with one operand works on, and each data kind with `အနုတ်ပြောင်း` |
| `နှိုင်းယှဉ်နိုင်သော` | the numbers and the characters, and each data kind with `ငယ်မငယ်စစ်` and `ကြီးမကြီးစစ်` |
| `တူမတူစစ်ဆေးနိုင်သော` | all data kinds, because `==` and `!=` work on two values of one data kind |
| `စာသားအဖြစ်ပြောင်းလဲပြသနိုင်သော` | all data kinds except a handle of a data kind from outside, because `ပြသမည်` prints them |
| `ပြောင်းလဲနိုင်သော` (`B အဖြစ် ပြောင်းလဲနိုင်သော`) | the pairs that the [conversion](conversions.md#conversions-through-traits) converts, because `အဖြစ် ပြောင်းလဲသော ရလဒ်` works on them, and each data kind with a `ပြောင်းလဲ` that returns `B` |
| `စမ်းသပ်ပြောင်းလဲနိုင်သော` (`B အဖြစ် E အမှားဖြင့် စမ်းသပ်ပြောင်းလဲနိုင်သော`) | each data kind with a `စမ်းသပ်ပြောင်းလဲ` that returns `B`, or an error of data kind `E` |
| `အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော` | the data kinds that threads can use: numbers, characters, true/false values, text, and lists, structs, optionals, results and tables of such data kinds, and the [objects threads can use](objects.md#objects-threads-can-use); not a plain shared object, a function value, a pointer or a handle of a data kind from outside |

An operator calls the function of its name when at least one operand is a struct or a
[trait value](trait-values.md#ordering), and the function takes the left operand first and
the right operand second. The two can be of different data kinds: a `ငယ်မငယ်စစ်` that
takes a struct and a `ကိန်းပြည့်` makes `x < ၃၀` work, and `၃၀ < x` needs one that takes
the number first. A function that takes the two data kinds as written wins over one whose
parameters the operands widen to. `>=`, `<=` and `!=` call `ငယ်မငယ်စစ်`, `ကြီးမကြီးစစ်` and
`ညီမညီစစ်` and turn the answer around.

A trait that the program declares with one of these names hides the built-in one. The
last trait has two words, which the core module does not declare, so these words are
still free to use as names.

## Errors

| Code | When |
| --- | --- |
| ZK0247 | a trait or a trait block inside a block |
| ZK0254 | a line that takes the defaults of a different trait from its block |
| ZK0343 | a data kind that does not have a trait that a kind parameter requires |
| ZK0344 | a name that is not a trait |
| ZK0345 | a trait block that leaves out a function of its trait |
| ZK0346 | a function of a trait block that is not one of its trait |
| ZK0347 | two functions that give a data kind one function of a trait |
| ZK0348 | a trait block outside the file of its data kind and the file of its trait |
| ZK0349 | the compiler mark outside the [core module](core.md) |
| ZK0351 | a trait block that takes the defaults of its trait when none is left to take (warning) |

The errors of kind parameters, calls and hints are on [Generics](generics.md#errors).
