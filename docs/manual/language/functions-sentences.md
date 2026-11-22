---
title: Calls written as sentences
description: Prepositions after the arguments of a call, starting and ending phrases around them, and the cost of this flexible grammar.
---

# Calls written as sentences

A function can give each parameter a preposition (argument label), the word that follows
the argument in a call, and it can give starting and ending phrases that a call puts around
its arguments. With them, a call becomes a Burmese sentence, such as
`"မင်္ဂလာပါ" ကို "စာအုပ်" သို့ ရေး မည်။`. This page covers prepositions, phrases, and the
cost of the flexible grammar that makes such calls possible. The Learn page
[Prepositions](../../learn/functions/prepositions.md) teaches the same ideas with examples.

## Prepositions

```text
<name> ဟူသော လုပ်ဆောင်ချက် သည် <param> ဟူသော <kind> တစ်<word> ကို ဝိဘတ် "<preposition>" ကို သုံး၍ လည်းကောင်း ... လက်ခံပြီး ...
<argument> <preposition> <argument> <preposition> <name> မည်။
```

To give a parameter a preposition, we write `ဝိဘတ် "<preposition>" ကို သုံး၍` ("using the
particle …") after its `ကို`, as in
`လမ်းကြောင်း ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "၌ရှိသော" ကို သုံး၍`. A call then puts each
argument with its preposition after it, and the function last.

The call ends as the [arguments-first](functions-calls.md#arguments-first) forms do:
`<name> မည်` or `<name> ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်` for a statement, and
`<name> သည့် ရလဒ်` or `<name> ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ်` for a value, with
`ရလဒ်ကောင်း` and the pass-on ending as there. A function with prepositions can still be
called in the other forms.

```zaka
ရေး ဟူသော လုပ်ဆောင်ချက် သည်
စာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လည်းကောင်း နေရာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "သို့" ကို သုံး၍ လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"{နေရာ}: {စာ}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ဖတ် ဟူသော လုပ်ဆောင်ချက် သည်
လမ်း ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "၌ရှိသော" ကို သုံး၍ လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"{လမ်း} ၏ စာ" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ" ကို "စာအုပ်" သို့ ရေး မည်။
"ဇာတ်လမ်း" ၌ရှိသော ဖတ် သည့် ရလဒ် ကို ပြသမည်။
ကိန်းရှင် စာ သည် "ကဗျာ" ၌ရှိသော ဖတ် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
စာ ကို ပြသမည်။
ရေး ဟူသော လုပ်ဆောင်ချက်ကို "ဟယ်လို" နှင့် "မှတ်စု" တို့ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
စာအုပ်: မင်္ဂလာပါ
ဇာတ်လမ်း ၏ စာ
ကဗျာ ၏ စာ
မှတ်စု: ဟယ်လို
```

When a parameter has no preposition, its argument has nothing after it. Here the first
parameter has a preposition and the second has none:

```zaka
ကူး ဟူသော လုပ်ဆောင်ချက် သည်
ရင်း ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "မှ" ကို သုံး၍ လည်းကောင်း ပစ် ဟူသော စာသား တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"{ရင်း} → {ပစ်}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"a.txt" မှ "b.txt" ကူး မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
a.txt → b.txt
```

### What a preposition can be

We write a preposition in the string as one word or as several: names, particles such as
`ကို`, `မှ`, `ဖြင့်`, `နှင့်`, `ထဲသို့` and `တွင်`, and other reserved words, as in
`လမ်းကြောင်းရှိ အရာကို` or `ကြိမ်မြောက်အကွက်ကို`. The compiler splits the string into words as the
lexer splits a sentence, so `ကိုသုံး၍` and `ကို သုံး၍` are one preposition, and so are
`၌ရှိသော` and `၌ ရှိသော`.

| Form | Rule | Code |
| --- | --- | --- |
| a call end (`မည်`, `သည့် ရလဒ်`, …), `ဟူသော`, a literal word, `အနုတ်`, a sign, `၊` or `။` in the string | not allowed, because the call could not end or an argument could not start | ZK0241 |
| a closing word at the end: `သည်`, `ပါ`, `ဖြစ်ကာ`, `ပြီးလျှင်` or `လက်ခံပြီး` | not allowed; such a word before another word of the preposition is fine, as `ပါ` in `ပါရှိသော` | ZK0241 |
| `၏` at the start of the last preposition, right before the function | allowed: `"စာ" ကို p ၏ နောက်ဆုံးမှဆက်ရေး သည့် ရလဒ်` | |
| `၏` at the start of another preposition, or of a last one before an [ending phrase](#starting-and-ending-phrases) | not allowed, because elsewhere `x ၏ f` reads a field | ZK0249 |
| `၏` after the first word of a preposition | not allowed | ZK0241 |
| a first preposition that starts with `ကို သုံး၍` or `တို့ကို သုံး၍` | not allowed, because these start the arguments-first call | ZK0241 |

### How a call is read

- A call reads the longest preposition of the function that the words after an argument
  spell, so the call reads `လမ်းကြောင်းရှိ အရာကို` as one preposition.
- Each parameter has one preposition or none. A call gives the arguments in the order of
  the parameters, each with the preposition of its parameter, and nothing after an argument
  whose parameter has none. A different preposition is ZK0331.
- Two arguments with no preposition between them can touch, `"a" "b"`, or we can put `၊`
  between them, `"a"၊ "b"`: each argument ends where the next operand starts.
- An argument is a whole clause, so `က နှင့် ခ ၏ ပေါင်းလဒ် ကို ပြ မည်` passes the sum. The
  compiler cuts the words before the function at its prepositions, and keeps each cut whose
  pieces each read as one clause. A word inside an argument can also be a preposition of
  the call, as the `နှင့်` of a sum: in `က နှင့် ခ ၏ ပေါင်းလဒ် နှင့် ဂ ကို တွဲ မည်`, a cut at
  the first `နှင့်` leaves `ခ ၏ ပေါင်းလဒ် နှင့် ဂ`, which is not a clause, so the call passes
  the sum and `ဂ`.
- When the words cut into the arguments in two different cuts that each read as clauses,
  the call is ZK0260, and parentheses around an argument keep its words together.
- A preposition call ends at a call of a function that declares prepositions or phrases,
  when its arguments and prepositions fit the function, and nowhere else. So a sentence
  with no such call keeps its meaning: `က ကို ခုနစ် သည့် ရလဒ် ဟု ထားမည်။` sets `က` when
  `ခုနစ်` declares no prepositions.

```zaka-error ZK0331
ရေး ဟူသော လုပ်ဆောင်ချက် သည်
စာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လည်းကောင်း နေရာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "သို့" ကို သုံး၍ လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စာ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ" ကို "စာအုပ်" ထဲသို့ ရေး မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0331]: argument 2 of "ရေး" has the preposition "ထဲသို့", not "သို့"
```

## Starting and ending phrases

```text
<name> ဟူသော လုပ်ဆောင်ချက် သည် အစ စကားစု "<words>" ကို သုံး၍ ...
<name> ဟူသော လုပ်ဆောင်ချက် သည် အဆုံး စကားစု "<words>" ကို သုံး၍ ...
<name> ဟူသော လုပ်ဆောင်ချက် သည် အစ စကားစု "<words>" နှင့် အဆုံး စကားစု "<words>" တို့ကို သုံး၍ ...

<start> <argument> <preposition> ... <argument> <preposition> <end> <name> မည်။
```

A function signature can also give words that belong to no parameter: a *starting phrase*,
which a call writes before its first argument, and an *ending phrase*, which a call writes
after its last argument, right before the function name. With them a function works like a
custom command. We write the phrases right before the parameters: right after `သည်` (or the
`၎င်းသည်` of a [generic](generics.md)), and after `ဖြစ်၍` in the function signature of an
[external](functions-c.md#external-functions) or a [C](functions-c.md#c-functions)
function. The phrases form a [list in words](words.md#lists-in-words) that `သုံး၍` ends.

A function with no parameters can have phrases alone, so a call of it is the phrases, the
name and the end: `ဖန်သားပြင်ကို အားလုံး ရှင်း မည်။`.

```zaka
ဖတ် ဟူသော လုပ်ဆောင်ချက် သည်
အစ စကားစု "ဖိုင်" နှင့် အဆုံး စကားစု "စာကို" တို့ကို သုံး၍
လမ်း ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ထဲမှ" ကို သုံး၍ လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"{လမ်း} ထဲမှ စာ" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ရှင်း ဟူသော လုပ်ဆောင်ချက် သည်
အစ စကားစု "ဖန်သားပြင်ကို" နှင့် အဆုံး စကားစု "အားလုံး" တို့ကို သုံး၍
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ရှင်းပြီ" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပြ ဟူသော လုပ်ဆောင်ချက် သည် အဆုံး စကားစု "အမြန်" ကို သုံး၍
စ ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"{စ}!" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဖိုင် "a.txt" ထဲမှ စာကို ဖတ် သည့် ရလဒ် ကို ပြသမည်။
ဖန်သားပြင်ကို အားလုံး ရှင်း မည်။
"ဟယ်လို" အမြန် ပြ မည်။
ဖတ်("b.txt") ကို ပြသမည်။
ရှင်း မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
a.txt ထဲမှ စာ
ရှင်းပြီ
ဟယ်လို!
b.txt ထဲမှ စာ
ရှင်းပြီ
```

The rules of phrases:

| Form | Rule | Code |
| --- | --- | --- |
| `အစ စကားစု`, `အဆုံး စကားစု` | reserved words, glued (`အစစကားစု`) or apart; `အစ` and `အဆုံး` alone remain names | |
| the particle before `သုံး၍` | `ကို` after one phrase, `တို့ကို` after two, with `နှင့်` between them | ZK0246 |
| the order | the starting phrase comes first | ZK0201 |
| the words of a phrase | as in a [preposition](#prepositions): names and particles, one space apart, with no call end, no closing word at the end, and no `၏` | ZK0258 |
| a call written as a sentence | it gives the phrases of the function signature in their places; a call that leaves a phrase out or writes a different one is an error, and the help shows the call as the signature spells it | ZK0352 |
| a last preposition that starts with `၏` | it cannot come before an ending phrase, because `x ၏ <word>` reads a field | ZK0249 |

- The parameters need no preposition when the function has phrases:
  `"ဟယ်လို" အမြန် ပြ မည်`. Arguments with no preposition between them touch or have `၊`
  between them, as in each [preposition call](#prepositions).
- The other call forms, `ဖတ်("b.txt")`, `ဖတ် ဟူသော လုပ်ဆောင်ချက်ကို "b.txt" ဖြင့် လုပ်ဆောင်မည်`
  and the short `ရှင်း မည်`, leave the phrases out.
- The phrases can tell two functions with one name apart, as prepositions can
  ([One name, several functions](functions-overloads.md)).
- A [function kind](function-values.md#function-kinds) can have phrases too, and a call through a
  value of it writes them, as a call of a named function does.

```zaka-error ZK0352
ဖတ် ဟူသော လုပ်ဆောင်ချက် သည်
အစ စကားစု "ဖိုင်" နှင့် အဆုံး စကားစု "စာကို" တို့ကို သုံး၍
လမ်း ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ထဲမှ" ကို သုံး၍ လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
လမ်း ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"a.txt" ထဲမှ စာကို ဖတ် သည့် ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0352]: the phrases of this call do not match the head of "ဖတ်"
```

Two parameters with no preposition, after a starting phrase:

```zaka
ပေါင်း ဟူသော လုပ်ဆောင်ချက် သည် အစ စကားစု "ကိန်း" ကို သုံး၍
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က + ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ၆ ဖြစ်ပါစေ။
ကိန်း က ၇ ပေါင်း သည့် ရလဒ် ကို ပြသမည်။
ကိန်း ၄၊ က + ၁ ပေါင်း သည့် ရလဒ် ကို ပြသမည်။
ကိန်း က နှင့် ၁ ၏ ပေါင်းလဒ် ၇ ပေါင်း သည့် ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၃
၁၁
၁၄
```

When a sign can join two bare arguments, the words cut into two readings, `က` and
`- ၁ - ၂` or `က - ၁` and `- ၂`, which is ZK0260:

```zaka-error ZK0260
ပေါင်း ဟူသော လုပ်ဆောင်ချက် သည် အစ စကားစု "ကိန်း" ကို သုံး၍
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က + ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ၆ ဖြစ်ပါစေ။
ကိန်း က - ၁ - ၂ ပေါင်း သည့် ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0260]: the words before "ပေါင်း" read as its arguments in more than one cut
```

```zaka-error ZK0258
ယူ ဟူသော လုပ်ဆောင်ချက် သည် အဆုံး စကားစု "x ၏" ကို သုံး၍
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0258]: "x ၏" cannot be a phrase
```

## What the flexible grammar costs

Prepositions of several words, call ends glued to a name, and calls after `၏` let a call
work as a Burmese sentence. Each one has a cost, listed here, and braces take a name back
from each one.

- **Braces.** `{x}` is the name `x`, whatever it spells: never a reserved word, a
  preposition, a call, or a name with a call end glued to it. `x ၏ {f}` reads the field `f`
  alone. A name in braces can contain spaces and signs too ([Names](words.md#names)).
- **The words of a preposition are read as the preposition.** A word of a preposition or of
  a [phrase](#starting-and-ending-phrases) that a file declares or imports is read as that
  preposition wherever a preposition can go. Declaring a variable, constant, parameter or
  function with such a name is the warning ZK0250, so we write `{name}` where the name is
  meant. Fields are not checked, since a field never sits where a preposition can go.
- **A function name glued to a call end is a call.** When the file sees a function `f`, the
  words `fမည်`, `fမည့်ရလဒ်`, `fသည့်ရလဒ်`, `fသောရလဒ်` (with their `ရလဒ်ကောင်း` forms), `fသည်`
  and `fပါ` are read as `f` and that end. Declaring a name spelled in this form is the
  warning ZK0251, and the name, without braces, is read as the call wherever it is not
  being declared. The split looks at the functions that the file sees, not at the end of
  each word, so `အမည်` ("name") remains a name unless the file sees a function `အ`. An
  import that brings a new function can change how a name without braces is read, and the
  warning at the declaration shows it.
- **A function hides a field.** `x ၏ f` calls a function `f` that accepts `x` before it
  reads a field `f`, and `x ၏ {f}` reads the field. A reading that the language gives after
  `၏` wins over both ([Calls on a value](functions-calls.md#calls-on-a-value)).

Because of these costs, some function signatures are not allowed:

| Function signature | Reason | Code |
| --- | --- | --- |
| a preposition with a call end, `ဟူသော`, a literal word, `အနုတ်`, a sign, `၊` or `။`, or one that ends in a closing word | the call could not end, or an argument could not start | ZK0241 |
| `၏` in a preposition other than at the start of the last one, or before an ending phrase | elsewhere `x ၏ f` reads a field | ZK0249, ZK0241 |
| `၏`, a call end, or a closing word at its end, in a phrase | the call could not end, or would read a field | ZK0258 |
| a first preposition that starts with `ကို သုံး၍` or `တို့ကို သုံး၍` | it starts the arguments-first call | ZK0241 |
| two functions with one name and the same parameter kinds, prepositions, phrases and result, in one file or across imports | no call could tell them apart | ZK0304 |

A reserved word declared in braces can name a function: `{စီ} ဟူသော လုပ်ဆောင်ချက် သည် …`
is called as `l ကို စီ မည်။`, with the word bare, right before a call end. Elsewhere the
word keeps its meaning in the grammar.

```zaka-warning ZK0251
ပေါင်း ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို ဝိဘတ် "နှင့်" ကို သုံး၍ လည်းကောင်း
ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က + ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၂ နှင့် ၃ ၏ ပေါင်းလဒ် ကို ပြသမည်။
ကိန်းရှင် ပေါင်းမည် သည် ၅ ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0251]: "ပေါင်းမည်" can read as "ပေါင်း" and "မည်"
```
