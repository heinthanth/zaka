---
title: Kind aliases
description: A second name for a data kind, where we can use the new name, and the errors of each form.
---

# Kind aliases

A kind alias (type alias) gives an existing data kind (type) another name. Both names
refer to the same data kind, and printed values and diagnostics use its original name.
For examples, see [Structs](../../learn/structs/README.md#another-name-for-a-kind) and
[Function values](../../learn/function-values/README.md) in Learn.

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။
ဒေတာအမျိုးအစား ကျောင်းသား ကို လူ ဟုလည်း မှတ်ယူနိုင်ပါသည်။
ဒေတာအမျိုးအစား အတန်းသားများ ကို ကျောင်းသား စာရင်း ဟုလည်း မှတ်ယူနိုင်ပါသည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် ကျောင်းသား(အသက်: ၁၂) ဖြစ်ပါစေ။
ကိန်းရှင် မွန် သည် လူ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကျော် ဖြစ်ပါစေ။
ကိန်းရှင် အတန်း သည် အတန်းသားများ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [ကျော်၊ မွန်] ဖြစ်ပါစေ။
အတန်း ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[လူ(အသက်: ၁၂)၊ လူ(အသက်: ၁၂)]
```

| Zaka | Reads as |
| --- | --- |
| `ဒေတာအမျိုးအစား ကျောင်းသား ကို လူ` | "the data kind `ကျောင်းသား`, as `လူ`" |
| `ဟုလည်း မှတ်ယူနိုင်ပါသည်` | "can also be taken as" |

## Declaring an alias

```text
[အများသုံး] ဒေတာအမျိုးအစား <alias> ကို <kind> ဟုလည်း မှတ်ယူနိုင်ပါသည်။
[အများသုံး] ဒေတာအမျိုးအစား <alias> ကို <kind> ဟုလည်း မှတ်ယူနိုင်ပြီး မျိုးပြပစ္စည်းမှာ <word> ဖြစ်သည်။
```

To declare an alias, we write `ဒေတာအမျိုးအစား`, the new name, `ကို`, the data kind that it
names, and `ဟုလည်း မှတ်ယူနိုင်ပါသည်`. The data kind is a whole phrase, with all its levels
and no measure word after it: a built-in data kind such as `၃၂ ဘစ် ကိန်းပြည့်`, a struct,
an enumeration, a list, an address, a [result kind](result-kinds.md), a [function
kind](function-values.md#function-kinds), or another alias.

- `အချက်အလက်အမျိုးအစား` is the same word as `ဒေတာအမျိုးအစား`, in a second spelling, and
  both work for all data kinds, function kinds included.
- We can write `ဟုလည်း` apart, as `ဟု လည်း`.
- An alias is counted with the measure word (classifier) that its declaration gives with
  `မှတ်ယူနိုင်ပြီး မျိုးပြပစ္စည်းမှာ <word> ဖြစ်သည်`. Without one, it is counted with the
  word of the data kind that it names ([Measure words](kinds.md#measure-words)).
- We declare an alias at the top level of a file, and the rest of the program can use it
  even above the declaration, because the compiler reads all declarations first.
- An alias can name another alias, declared above it or below it.
- To export an alias, we put `အများသုံး` at the start of the declaration, and another file
  imports it with its name, as it imports a struct ([Modules](modules.md)).

| Mistake | Code |
| --- | --- |
| an alias declared inside a block, a loop, a conditional or a function | ZK0240 |
| an alias with the name of a struct, an enumeration or another alias, because a name names one data kind in the whole program | ZK0316 |
| a chain of aliases that leads back to its start, so it names no data kind; each alias in the chain gets the error | ZK0336 |

An alias whose name ends in `များ` also claims its name without `များ`, as an enumeration
does ([The name and its stem](enums.md#the-name-and-its-stem)).

## Where we can use an alias

The compiler reads an alias as the data kind that it names, so we can use the alias
wherever we can use that data kind:

| Place | Written |
| --- | --- |
| a declaration, a parameter, a field, a result or a conversion | the alias with its measure word, `<alias> တစ်<word>` |
| a list | `<alias> စာရင်း` |
| a pointer | `<alias> ၏ လိပ်စာ` |
| a size | `<alias> တစ်<word> ၏ အရွယ်အစား` |
| a struct literal, for an alias of a struct | `ကျောင်းသား(အသက်: ၁၂)` or `အသက် အနေဖြင့် ၁၂ ရှိသော ကျောင်းသားတစ်ခု` |
| a member, for an alias of an enumeration | `<alias>များထဲမှ <member>` |
| the default arm of a switch, for an alias of an enumeration | `တခြား <alias> တစ်<word> ဖြစ်လျှင်` |

The levels that we write after an alias wrap the levels of the data kind that it names,
so with the aliases above, `အတန်းသားများ ၏ လိပ်စာ` means `လူ စာရင်း ၏ လိပ်စာ`. A struct
literal written with an alias is counted with the measure word of the struct.

In a list of parameters or fields, a function kind with parameters must go in parentheses
(ZK0239). Its alias is one word, so it needs no parentheses:

```zaka
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ နှင့် စိမ်း တို့ပါဝင်သည်။
ဒေတာအမျိုးအစား မီးရောင် ကို အရောင် ဟုလည်း မှတ်ယူနိုင်ပါသည်။
ဒေတာအမျိုးအစား ပြောင်းချက် ကို ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား ဟုလည်း မှတ်ယူနိုင်ပါသည်။

နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

နှစ်ခါ ဟူသော လုပ်ဆောင်ချက် သည်
ဖ ဟူသော ပြောင်းချက် တစ်ခု ကို လည်းကောင်း
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ဖ(ဖ(က)) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မီး သည် မီးရောင်များထဲမှ စိမ်း ဖြစ်ပါစေ။
မီး သည် နီ ဖြစ်လျှင် "ရပ်" ဟု ပြသမည်။
တခြား မီးရောင် တစ်ခု ဖြစ်လျှင် "{မီး}" ဟု ပြသမည်။
နှစ်ခါ ဟူသော လုပ်ဆောင်ချက်ကို နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် နှင့် ၅ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို
ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အရောင်များထဲမှ စိမ်း
၂၀
```

The printed member names `အရောင်`, the enumeration itself, because an alias is a second
name and not a separate data kind. An alias can also have kind parameters, which
[Generic aliases](generics.md#generic-aliases) explains.

## Errors

An alias inside a block:

```zaka-error ZK0240
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဒေတာအမျိုးအစား အမှတ် ကို ကိန်းပြည့် ဟုလည်း မှတ်ယူနိုင်ပါသည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0240]: kind aliases can be declared at the top level and nowhere else
```

Two aliases that name each other:

```zaka-error ZK0336
ဒေတာအမျိုးအစား က ကို ခ ဟုလည်း မှတ်ယူနိုင်ပါသည်။
ဒေတာအမျိုးအစား ခ ကို က ဟုလည်း မှတ်ယူနိုင်ပါသည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0336]: the kind alias "က" leads back to itself
error[ZK0336]: the kind alias "ခ" leads back to itself
```

| Code | Problem |
| --- | --- |
| ZK0240 | an alias declared inside a block |
| ZK0316 | an alias with the name of a struct, an enumeration or another alias |
| ZK0336 | a chain of aliases that leads back to its start |
