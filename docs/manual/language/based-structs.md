---
title: Structs based on structs
description: A struct based on other structs, <struct> ကို အခြေခံသော, holds each as a part and reaches the fields and the functions of its parts as its own; reading a part, names in two parts, traits through parts, and the errors.
---

# Structs based on structs

A struct can be based on other structs, with `<struct> ကို အခြေခံသော` after `သည်` in its head.
It then holds each of them as a part, and a value of it reaches the fields and the functions
of its parts as its own, the same as an embedded struct (struct embedding) in Go. This
gives the reuse of a parent class without making a child a kind of its parent: a function
that takes a `လူ` still takes a `လူ` alone. To take each data kind that can greet, a function
takes a [trait value](trait-values.md), and a struct gets the traits of its parts. The Learn
book teaches the form in [Based on another struct](../../learn/structs/based.md).

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် နာမည် ဟူသော စာသား တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည်
လ ဟူသော လူ တစ်ယောက် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"မင်္ဂလာပါ {လ ၏ နာမည်}" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကျောင်းသား ဟူသော ဒေတာအစုအဝေး သည်
လူ ကို အခြေခံသော ဒေတာအစုအဝေး ဖြစ်၍ ၎င်းတွင်
ကျောင်း ဟူသော စာသား တစ်ခု
ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

ကိန်းရှင် က သည် ကျောင်းသား(လူ: လူ(နာမည်: "မောင်")၊ ကျောင်း: "YTU") ဖြစ်ပါစေ။
က ၏ နာမည် ကို ပြသမည်။
က ၏ နှုတ်ဆက်() ကို ပြသမည်။
က ၏ နာမည် ကို "မမ" ဟု ထားမည်။
က ၏ နှုတ်ဆက်() ကို ပြသမည်။
ကိန်းရှင် ပ သည် က ကို လူ အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
ပ ၏ နာမည် ကို ပြသမည်။
က ၏ ကျောင်း ကို ပြသမည်။
```

```text
မောင်
မင်္ဂလာပါ မောင်
မင်္ဂလာပါ မမ
မမ
YTU
```

| Zaka | Reads as |
| --- | --- |
| `လူ ကို အခြေခံသော ဒေတာအစုအဝေး` | "a struct based on `လူ`" |
| `လူ နှင့် ဝန်ထမ်း တို့ကို အခြေခံသော` | "based on `လူ` and `ဝန်ထမ်း`" |

## The head and the literal

The mark `<struct> ကို အခြေခံသော` stands after `သည်`, among the other marks of the head
([Marks](declarations.md#marks)), with `နှင့်` between two structs:
`… သည် လူ နှင့် ဝန်ထမ်း တို့ကို အခြေခံသော ဒေတာအစုအဝေး ဖြစ်၍ ၎င်းတွင် …`. A base is a struct or
a [shared object](objects.md) (ZK0363): a plain struct is held as a value, a duplicate of
the one a literal gives, and a shared object as a reference, so two values can share one
part. A kind from outside, `ပြင်ပမှ လာသော`, is based on no struct (ZK0263), since its fields
are those of a C struct; a struct a lib gives to C can be, and C sees each part as a field
of the struct of the part.

A literal sets each part under the name of its struct, `ကျောင်းသား(လူ: လူ(…)၊ ကျောင်း: …)`,
and a part with no initial value must be set, as each other field. A print shows the part
in the same form, under the name of its struct.

## Fields and functions of parts

`x ၏ f` reads the field `f` of `x`, and when the struct of `x` has no field `f`, the field
`f` of the nearest part that has one: the parts of `x` first, then their parts. A write
reaches the same field. A call on a value, `x ၏ g(a)`, and a call `g(x၊ a)` run the
function `g` of the struct of `x`, and when no function `g` takes `x`, the function `g` of
the nearest part, with that part as the first argument. The long call statement,
`g ဟူသော လုပ်ဆောင်ချက်ကို x ဖြင့် လုပ်ဆောင်မည်။`, follows the same rule, and a mutable
parameter of `g` changes the part in place. So a struct overrides a function of its part
with its own function of the name, which wins.

When two parts at one depth have the name, the compiler cannot pick one, and the read is
ZK0364:

```zaka-error ZK0364
လူ ဟူသော ဒေတာအစုအဝေးတွင် နာမည် ဟူသော စာသား တစ်ခု ပါဝင်သည်။
ဝန်ထမ်း ဟူသော ဒေတာအစုအဝေးတွင် နာမည် ဟူသော စာသား တစ်ခု ပါဝင်သည်။
ဆရာ ဟူသော ဒေတာအစုအဝေး သည် လူ နှင့် ဝန်ထမ်း တို့ကို အခြေခံသော ဒေတာအစုအဝေး ဖြစ်၍
၎င်းတွင် ဘာသာ ဟူသော စာသား တစ်ခု ပါဝင်သည်။

ကိန်းရှင် ဆ သည် ဆရာ(လူ: လူ(နာမည်: "က")၊ ဝန်ထမ်း: ဝန်ထမ်း(နာမည်: "ခ")၊ ဘာသာ: "x") ဖြစ်ပါစေ။
ဆ ၏ နာမည် ကို ပြသမည်။
```

## Reading a part

`x ကို <struct> အဖြစ် ပြောင်းလဲသော ရလဒ်` reads the part of `x` of that struct, the nearest
first, as the read of members that hold data does. A function that takes the struct of the
part takes it: `ပြ(က ကို လူ အဖြစ် ပြောင်းလဲသော ရလဒ်)`. With two parts of one struct at one
depth, the read is ZK0364.

## Traits through parts

A struct has a trait when its own functions give it each function of the trait, or the
functions of a part do. So a `ကျောင်းသား` based on a `လူ` with `နှုတ်ဆက်` goes to a generic
that requires `နှုတ်ဆက်နိုင်သော`, and into a `နှုတ်ဆက်နိုင်သော အရာ`, where a call runs the
function of the part. A function of the trait that gives back the trait's data kind is no
function a part gives, since the part's function gives back the part.

A [default function](traits.md#default-functions) is different. When the trait block of a
part takes the default, the struct takes it too, and the default runs on the whole struct: each function of the trait that it calls is the one of the struct, its own
when it has one and the one of its part otherwise. With a `နာမည်ယူ` of its own, a `ဆရာ`
based on a `လူ` that takes the default `နှုတ်ဆက်` greets with its own name.

A function of a part sees the part alone: when a function of `လူ` calls another function
on its `လူ`, it runs the one of `လူ`, even when `ကျောင်းသား` has its own. Code that must reach
the function of the outer struct takes a trait value.

## Errors

| Code | When |
| --- | --- |
| ZK0263 | a kind from outside, `ပြင်ပမှ လာသော`, based on a struct |
| ZK0266 | two marks of the head with no `၊` between them |
| ZK0363 | a base that is no struct and no shared object |
| ZK0364 | a name in two parts at one depth |
