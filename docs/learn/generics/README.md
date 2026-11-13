---
title: Generics
description: Write a function or a struct once for each kind, with kind parameters, and let the compiler make one for each kind the program uses.
---

# Generics

A function that gives back the first cell of a list does the same work for a list of
numbers and for a list of text. Written with a *kind parameter*, it is written once:

```zaka
လုပ်ဆောင်ချက် ပထမကွက်<T> သည်
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

`<T>` after the name tells that `T` stands for a kind. Inside the function `T` is a kind
like the others, counted with `ခု`. Each call gives `T` the kind of its argument: the
first call makes a function for `ကိန်းပြည့်`, the second one for `စာသား`. The compiler
checks each of them as if it were written out.

## The word form

The kind parameters can be written in words after `သည်` too:

```zaka
လုပ်ဆောင်ချက် ထပ် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
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

`ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်` reads "the general data kind T
can be put in place, and it ...". Two parameters are a list in words:
`ယေဘုယျဒေတာအမျိုးအစား A နှင့် ယေဘုယျဒေတာအမျိုးအစား B တို့ကိုအစားထိုးသုံးနိုင်၍`.

## Writing the kinds

A call can name its kinds: after `::` in the sign form, or before the function in the
word forms, with `အစားထိုးအသုံးပြုသည့်` ("that puts ... in place"):

```zaka
လုပ်ဆောင်ချက် ထပ်<T> သည်
က ဟူသော T တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ထပ်::<ဒဿမကိန်း>(၂) ကို ပြသမည်။
ဒဿမကိန်း ကို အစားထိုးအသုံးပြုသည့် ထပ် ဟူသော လုပ်ဆောင်ချက်ကို ၃ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
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
လုပ်ဆောင်ချက် ဗလာ<T> သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
ကနဦးတန်ဖိုး ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဗလာ() ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

`ဗလာ::<ကိန်းပြည့်>()` fixes it, and so does a declared kind:
`ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဗလာ() ဖြစ်ပါစေ။`

## Generic structs

A struct can take kind parameters too. Its values take the kinds from their fields, or
from the kind their place declares:

```zaka
အတွဲ<က၊ ခ> ဟူသော ဒေတာအစုအဝေးတွင်
ပထမ ဟူသော က တစ်ခု နှင့် ဒုတိယ ဟူသော ခ တစ်ခု
တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ တွဲ ဖြစ်သည်။

လုပ်ဆောင်ချက် လှည့်<A၊ B> သည်
တွဲ ဟူသော အတွဲ<A၊ B> တစ်တွဲ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အတွဲ<B၊ A> တစ်တွဲ ကို ပြန်ပေးမည်။
အတွဲ(ပထမ: (တွဲ ၏ ဒုတိယ)၊ ဒုတိယ: (တွဲ ၏ ပထမ)) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လူ သည် အတွဲ(ပထမ: "မောင်မောင်"၊ ဒုတိယ: ၂၄) ဖြစ်ပါစေ။
လှည့်(လူ) ၏ ပထမ ကို ပြသမည်။
ကိန်းရှင် အမှတ် သည် အတွဲ<ဒဿမကိန်း၊ ဒဿမကိန်း> တစ်တွဲ ဖြစ်၍ တန်ဖိုးသည် အတွဲ(ပထမ: ၁၊ ဒုတိယ: ၂) ဖြစ်ပါစေ။
အမှတ် ၏ ဒုတိယ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၄
၂.၀
```

`အတွဲ<A၊ B>` names the struct with its kinds; in words it reads
`A နှင့် B တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ`. `လှည့်` works its kinds out through the
struct: its argument is an `အတွဲ<text၊ isize>`, so `A` is `စာသား` and `B` is
`ကိန်းပြည့်`.

## Local kinds

Long kinds in a header read better with a short name. A notes block after the header,
`ဤနေရာတွင် သတိပြုရန်မှာ` ("here, to note:"), names them:

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

`C` and `D` are no parameters: the two are names for the pair kinds, and a call gives
them nothing. Like a kind parameter, a local kind counts with `ခု`.

## Generic aliases

An alias can take kind parameters as well:
`ဒေတာအမျိုးအစား အမည်တွဲ<T> ကို စာသား နှင့် T တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ ဟုလည်း မှတ်ယူနိုင်ပါသည်။`
makes `အမည်တွဲ<ကိန်းပြည့်>` a name for `အတွဲ<စာသား၊ ကိန်းပြည့်>`.

Kinds can also need traits: functions a kind must have before a generic takes it.
[Generics and traits](../../manual/language/generics.md) holds the full rules.

Next: [Traits](traits.md).
