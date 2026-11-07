---
title: Kind aliases
description: A second name for a kind, where the name can stand, and the errors of each.
---

# Kind aliases

A *kind alias* gives a kind a second name. The alias and the kind it names are one kind:
a value of one is a value of the other, and a printed value and a message use the name
the kind has on its own. The Learn book shows it in [Structs](../../learn/structs/README.md#another-name-for-a-kind)
and in [Functions as values](../../learn/pointers/functions.md#naming-a-function-kind).

```zaka
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက် တို့ ပါဝင်သည်။
ဒေတာအမျိုးအစား လူ ကို ကျောင်းသား ဟုလည်း မှတ်ယူနိုင်ပါသည်။
ဒေတာအမျိုးအစား ကျောင်းသား စာရင်း ကို အတန်းသားများ ဟုလည်း မှတ်ယူနိုင်ပါသည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကျော် သည် ကျောင်းသား(အသက်: ၁၂) ဖြစ်ပါစေ။
ကိန်းရှင် မွန် သည် လူ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကျော် ဖြစ်ပါစေ။
ကိန်းရှင် အတန်း သည် အတန်းသားများ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [ကျော်၊ မွန်] ဖြစ်ပါစေ။
အတန်း ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[လူ(အသက်: ၁၂)၊ လူ(အသက်: ၁၂)]
```

| Zaka | Reads as |
| --- | --- |
| `ဒေတာအမျိုးအစား လူ ကို ကျောင်းသား` | "the data kind `လူ`, as `ကျောင်းသား`" |
| `ဟုလည်း မှတ်ယူနိုင်ပါသည်` | "can also be taken as" |

## Declaring an alias

```text
ဒေတာအမျိုးအစား <kind> ကို <alias> ဟုလည်း မှတ်ယူနိုင်ပါသည်။
<function kind> ကို <alias> ဟုလည်း မှတ်ယူနိုင်ပါသည်။
```

- `<kind>` is a whole kind phrase, its levels included: a built-in kind such as
  `၃၂ ဘစ် ကိန်းပြည့်`, a struct, an enumeration, a list, an address, a
  [result kind](error-handling.md#result-kinds), a [function kind](function-values.md#the-kind),
  or another alias.
- The opener is the one a struct and an enumeration take, `ဒေတာအမျိုးအစား` or
  `အချက်အလက်အမျိုးအစား`. A function kind can stand with no opener before it; another
  kind with none is ZK0201.
- `ဟုလည်း` can be written apart, as `ဟု လည်း`.
- A declaration stands at the top level of a file. Inside a block, a loop, a conditional
  or a function it is ZK0240. An alias can be used above its declaration, since the
  declarations are read first.
- A name names one kind in the whole program: an alias that takes the name of a struct,
  an enumeration or another alias is ZK0316. An alias whose name ends in `များ` claims
  its name without `များ` too, as an enumeration does.
- An alias can name another alias, declared above it or below. A chain of aliases that
  comes back to its start names no kind, and each alias in it is ZK0336.
- `အများသုံး` before the declaration exports the alias, and another file imports it with
  its name, as it imports a struct ([Modules](modules.md)).

## Where an alias stands

The compiler reads an alias as the kind it names, so the alias stands wherever that kind
does:

- In each place a kind goes: a declaration, a parameter, a field, a result, a list
  `<alias> စာရင်း`, an address `<alias> ၏ လိပ်စာ`, a size `<alias> ၏ အရွယ်အစား` and
  a conversion. The levels written after the alias go around the levels of the kind it
  names: with the alias above, `အတန်းသားများ ၏ လိပ်စာ` is `လူ စာရင်း ၏ လိပ်စာ`.
- For a struct, in a struct literal of each form, `ကျောင်းသား(အသက်: ၁၂)` and
  `အသက် ၁၂ ရှိသော ကျောင်းသားတစ်ခု`. The literal counts with the classifier of the struct.
- For an enumeration, before `ထဲမှ` in a member, `<alias>များထဲမှ <member>`, and in the
  default arm of a switch, `တခြား <alias> ဖြစ်လျှင်`.

A function kind with parameters stands in parentheses in a list of parameters or fields
(ZK0239). Its alias is one word, so it needs none:

```zaka
ဒေတာအမျိုးအစား အရောင် ဆိုသည်မှာ နီ၊ ဝါ၊ စိမ်း တို့ ဖြစ်သည်။
ဒေတာအမျိုးအစား အရောင် ကို မီးရောင် ဟုလည်း မှတ်ယူနိုင်ပါသည်။
ကိန်းပြည့် ကို လက်ခံပြီး ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား ကို ပြောင်းချက် ဟုလည်း မှတ်ယူနိုင်ပါသည်။

လုပ်ဆောင်ချက် နှစ်ဆ သည် ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က * ၂ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် နှစ်ခါ သည် ပြောင်းချက် ဖ၊ ကိန်းပြည့် က တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
ဖ(ဖ(က)) ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မီး သည် မီးရောင်များထဲမှ စိမ်း ဖြစ်ပါစေ။
မီး သည် နီ ဖြစ်လျှင် "ရပ်" ဟု ပြသပါ။
တခြား မီးရောင် ဖြစ်လျှင် "{မီး}" ဟု ပြသပါ။
လုပ်ဆောင်ချက် နှစ်ခါ ကို လုပ်ဆောင်ချက် နှစ်ဆ ၏ လိပ်စာ၊ ၅ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အရောင်များထဲမှ စိမ်း
၂၀
```

The printed member names `အရောင်`, the enumeration itself: an alias is a second name,
not a kind of its own.

## Errors

An alias inside a block:

```zaka-error ZK0240
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဒေတာအမျိုးအစား ကိန်းပြည့် ကို အမှတ် ဟုလည်း မှတ်ယူနိုင်ပါသည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0240]: kind aliases can be declared at the top level and nowhere else
```

Two aliases that name each other:

```zaka-error ZK0336
ဒေတာအမျိုးအစား ခ ကို က ဟုလည်း မှတ်ယူနိုင်ပါသည်။
ဒေတာအမျိုးအစား က ကို ခ ဟုလည်း မှတ်ယူနိုင်ပါသည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0336]: the kind alias "က" leads back to itself
error[ZK0336]: the kind alias "ခ" leads back to itself
```

| Code | Problem |
| --- | --- |
| ZK0201 | a kind other than a function kind with no `ဒေတာအမျိုးအစား` before it |
| ZK0240 | an alias declared inside a block |
| ZK0316 | an alias with the name of a struct, an enumeration or another alias |
| ZK0336 | a chain of aliases that comes back to its start |
