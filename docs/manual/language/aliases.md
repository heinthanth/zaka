---
title: Kind aliases
description: A second name for a kind, where the name can stand, and the errors of each.
---

# Kind aliases

A *kind alias* gives a kind a second name. The alias and the kind it names are one kind:
a value of one is a value of the other, and a printed value and a message use the name
the kind has on its own. The Learn book shows it in [Structs](../../learn/structs/README.md#another-name-for-a-kind)
and in [Function values](../../learn/function-values/README.md).

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

- The new name comes first, then `ကို` and the kind it names.
- `<kind>` is a whole kind phrase, its levels included, with no measure word after it: a built-in kind such as
  `၃၂ ဘစ် ကိန်းပြည့်`, a struct, an enumeration, a list, an address, a
  [result kind](error-handling.md#result-kinds), a [function kind](function-values.md#the-kind),
  or another alias.
- The opener is `ဒေတာအမျိုးအစား` or `အချက်အလက်အမျိုးအစား`, one word in two spellings,
  for each kind a function kind among them.
- An alias counts with the classifier its declaration names with
  `မှတ်ယူနိုင်ပြီး မျိုးပြပစ္စည်းမှာ <word> ဖြစ်သည်`; with none, it counts with the word of
  the kind it names ([Measure words](kinds.md#measure-words)).
- `ဟုလည်း` can be written apart, as `ဟု လည်း`.
- A declaration stands at the top level of a file. Inside a block, a loop, a conditional
  or a function it is ZK0240. An alias can be used above its declaration, since the
  declarations are read first.
- A name names one kind in the whole program: an alias that takes the name of a struct,
  an enumeration or another alias is ZK0316. An alias whose name ends in `များ` claims
  its name without `များ` too, as an enumeration does.
- An alias can name another alias, declared above it or below. A chain of aliases that
  comes back to its start names no kind, and each alias in it is ZK0336.
- `အများသုံး` at the start of the declaration exports the alias, and another file imports it with
  its name, as it imports a struct ([Modules](modules.md)).

## Where an alias stands

The compiler reads an alias as the kind it names, so the alias stands wherever that kind
does:

- In each place a kind goes: a declaration, a parameter, a field, a result, a list
  `<alias> စာရင်း`, a pointer `<alias> ၏ လိပ်စာ`, a size `<alias> တစ်<word> ၏ အရွယ်အစား`
  and a conversion, each with its measure word. The levels written after the alias go around the levels of the kind it
  names: with the alias above, `အတန်းသားများ ၏ လိပ်စာ` is `လူ စာရင်း ၏ လိပ်စာ`.
- For a struct, in a struct literal of each form, `ကျောင်းသား(အသက်: ၁၂)` and
  `အသက် ၁၂ ရှိသော ကျောင်းသားတစ်ခု`. The literal counts with the classifier of the struct.
- For an enumeration, before `ထဲမှ` in a member, `<alias>များထဲမှ <member>`, and in the
  default arm of a switch, `တခြား <alias> တစ်<word> ဖြစ်လျှင်`.

A function kind with parameters stands in parentheses in a list of parameters or fields
(ZK0239). Its alias is one word, so it needs none:

```zaka
အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ၊ စိမ်း တို့ပါဝင်သည်။
ဒေတာအမျိုးအစား မီးရောင် ကို အရောင် ဟုလည်း မှတ်ယူနိုင်ပါသည်။
ဒေတာအမျိုးအစား ပြောင်းချက် ကို ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား ဟုလည်း မှတ်ယူနိုင်ပါသည်။

လုပ်ဆောင်ချက် နှစ်ဆ သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် နှစ်ခါ သည် ဖ ဟူသော ပြောင်းချက် တစ်ခု ကို လည်းကောင်း က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ဖ(ဖ(က)) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မီး သည် မီးရောင်များထဲမှ စိမ်း ဖြစ်ပါစေ။
မီး သည် နီ ဖြစ်လျှင် "ရပ်" ဟု ပြသမည်။
တခြား မီးရောင် တစ်ခု ဖြစ်လျှင် "{မီး}" ဟု ပြသမည်။
နှစ်ခါ ဟူသော လုပ်ဆောင်ချက်ကို နှစ်ဆ ဟူသော လုပ်ဆောင်ချက်၊ ၅ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
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
| ZK0336 | a chain of aliases that comes back to its start |
