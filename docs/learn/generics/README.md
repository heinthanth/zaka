---
title: Generics
description: Write a function or a struct once for each kind, with kind parameters, and let the compiler make one for each kind the program uses.
---

# Generics

A function that gives back the first cell of a list does the same work for a list of
numbers and for a list of text. Written with a *kind parameter*, it is written once:

```zaka
လုပ်ဆောင်ချက် ပထမကွက် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
ကိန်းများ ဟူသော T စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းများ ၏ အကွက် ၀ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပထမကွက်([၇၊ ၈၊ ၉]) ကို ပြသမည်။
ပထမကွက်(["နီ"၊ "ဝါ"]) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၇
နီ
```

`ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်` after `သည်` reads "the general
data kind T can be put in place, and it ...": it tells that `T` stands for a kind.
Inside the function `T` is a kind like the others, counted with `ခု`. Each call gives
`T` the kind of its argument: the first call makes a function for `ကိန်းပြည့်`, the
second one for `စာသား`. The compiler checks each of them as if it were written out.

Two parameters are a list in words:
`ယေဘုယျဒေတာအမျိုးအစား A နှင့် ယေဘုယျဒေတာအမျိုးအစား B တို့ကိုအစားထိုးသုံးနိုင်၍`.

## The sign form

A shorter form puts the kind parameters in `<` and `>` after the name:

```zaka
လုပ်ဆောင်ချက် ထပ်<T> သည်
က ဟူသော T တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ထပ်('က') ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
က
```

The two forms make the same function. The examples on these pages write the word form.

## Writing the kinds

A call can name its kinds before the function, with `အစားထိုးအသုံးပြုသည့်` ("that puts
... in place"), or after `::` in the sign form:

```zaka
လုပ်ဆောင်ချက် ထပ် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
က ဟူသော T တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဒဿမကိန်း ကို အစားထိုးအသုံးပြုသည့် ထပ် ဟူသော လုပ်ဆောင်ချက်ကို ၂ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ထပ်::<ဒဿမကိန်း>(၃) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂.၀
၃.၀
```

The kinds are needed when the arguments do not show them. A kind that the result alone
names comes from the kind the place of the call expects, and with no such place the
compiler asks for the kinds:

```zaka-error ZK0341
လုပ်ဆောင်ချက် ဗလာ သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
ကနဦးတန်ဖိုး ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဗလာ() ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

Kinds written before the name fix it,
`ကိန်းပြည့် ကို အစားထိုးအသုံးပြုသည့် ဗလာ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ်`, and so does a
declared kind:
`ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဗလာ() ဖြစ်ပါစေ။`

## Generic structs

A struct can take kind parameters too. Its head then ends in `ဒေတာအစုအဝေးသည်`, and
`၎င်းတွင်` opens the fields. Its values take the kinds from their fields, or from the
kind their place declares:

```zaka
အတွဲ ဟူသော ဒေတာအစုအဝေးသည် ယေဘုယျဒေတာအမျိုးအစား က နှင့် ယေဘုယျဒေတာအမျိုးအစား ခ တို့ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင်
ပထမ ဟူသော က တစ်ခု နှင့် ဒုတိယ ဟူသော ခ တစ်ခု
တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ တွဲ ဖြစ်သည်။

လုပ်ဆောင်ချက် လှည့် သည် ယေဘုယျဒေတာအမျိုးအစား A နှင့် ယေဘုယျဒေတာအမျိုးအစား B တို့ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
တွဲ ဟူသော A နှင့် B တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ တစ်တွဲ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ B နှင့် A တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ တစ်တွဲ ကို ပြန်ပေးမည်။
ပထမ အနေဖြင့် (တွဲ ၏ ဒုတိယ) နှင့် ဒုတိယ အနေဖြင့် (တွဲ ၏ ပထမ) ရှိသော အတွဲ တစ်တွဲ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လူ သည် ပထမ အနေဖြင့် "မောင်မောင်" နှင့် ဒုတိယ အနေဖြင့် ၂၄ ရှိသော အတွဲ တစ်တွဲ ဖြစ်ပါစေ။
လှည့်(လူ) ၏ ပထမ ကို ပြသမည်။
ကိန်းရှင် အမှတ် သည် ဒဿမကိန်း နှင့် ဒဿမကိန်း တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ တစ်တွဲ ဖြစ်၍ တန်ဖိုးသည် ပထမ အနေဖြင့် ၁ နှင့် ဒုတိယ အနေဖြင့် ၂ ရှိသော အတွဲ တစ်တွဲ ဖြစ်ပါစေ။
အမှတ် ၏ ဒုတိယ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၄
၂.၀
```

`A နှင့် B တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ` ("the pair that puts A and B in place")
names the struct with its kinds; in signs it reads `အတွဲ<A၊ B>`. `လှည့်` works its
kinds out through the struct: its argument is an `အတွဲ<text၊ isize>`, so `A` is `စာသား` and `B` is
`ကိန်းပြည့်`.

`အတွဲ<က၊ ခ> ဟူသော ဒေတာအစုအဝေးတွင် …` declares the same struct in the sign form.

## Local kinds

Long kinds in a header read better with a short name. A notes block after the header,
`ဤနေရာတွင် သတိပြုရန်မှာ` ("here, to note:"), names them:

```zaka
အတွဲ ဟူသော ဒေတာအစုအဝေးသည် ယေဘုယျဒေတာအမျိုးအစား က နှင့် ယေဘုယျဒေတာအမျိုးအစား ခ တို့ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင်
ပထမ ဟူသော က တစ်ခု နှင့် ဒုတိယ ဟူသော ခ တစ်ခု
တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ တွဲ ဖြစ်သည်။

လုပ်ဆောင်ချက် လှည့် သည် ယေဘုယျဒေတာအမျိုးအစား A နှင့် ယေဘုယျဒေတာအမျိုးအစား B တို့ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
တွဲ ဟူသော C တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ D တစ်ခု ကို ပြန်ပေးမည်။
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

`C` and `D` are no parameters: the two are names for the pair kinds, and a call gives
them nothing. Like a kind parameter, a local kind counts with `ခု`.

## Generic aliases

An alias can take kind parameters as well. Its parameters follow `သည်` as in a function, and
`၎င်းကို` ("it") goes on to the kind the alias names:

```zaka
အတွဲ ဟူသော ဒေတာအစုအဝေးသည် ယေဘုယျဒေတာအမျိုးအစား က နှင့် ယေဘုယျဒေတာအမျိုးအစား ခ တို့ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင်
ပထမ ဟူသော က တစ်ခု နှင့် ဒုတိယ ဟူသော ခ တစ်ခု
တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ တွဲ ဖြစ်သည်။
ဒေတာအမျိုးအစား အမည်တွဲ သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍
၎င်းကို စာသား နှင့် T တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ ဟုလည်း မှတ်ယူနိုင်ပါသည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရမှတ် သည် ကိန်းပြည့် ကို အစားထိုးအသုံးပြုသည့် အမည်တွဲ တစ်တွဲ ဖြစ်၍ တန်ဖိုးသည် ပထမ အနေဖြင့် "မြမြ" နှင့် ဒုတိယ အနေဖြင့် ၉၀ ရှိသော အတွဲ တစ်တွဲ ဖြစ်ပါစေ။
ရမှတ် ၏ ဒုတိယ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၉၀
```

`ကိန်းပြည့် ကို အစားထိုးအသုံးပြုသည့် အမည်တွဲ` is a name for
`စာသား နှင့် ကိန်းပြည့် တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ`. In the sign form the alias reads
`ဒေတာအမျိုးအစား အမည်တွဲ<T> ကို …`.

Kinds can also need traits: functions a kind must have before a generic takes it.
[Generics and traits](../../manual/language/generics.md) holds the full rules.

Next: [Traits](traits.md).
