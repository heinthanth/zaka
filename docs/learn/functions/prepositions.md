---
title: Prepositions
description: Give each parameter a preposition so that a call is a Burmese sentence, add phrases around the arguments, and share one name between several functions.
---

# Prepositions

In a Burmese sentence, a small word after each noun shows its role, such as `ကို` after the
object or `သို့` ("to") after a place. Zaka lets us give each parameter such a word, its
*preposition* (argument label), so that a call to the function is a plain Burmese sentence.
This page uses functions with a result, which [Results](results.md) teaches.

## You will learn

- how to give a parameter a preposition, and how a call then looks
- how to use a preposition of several words, and phrases around the arguments
- how a call can glue onto the function name, and what that costs
- how several functions can share one name

```zaka
// "Function write, accepting a text (preposition ကို) and a place (preposition သို့)"
ရေး ဟူသော လုပ်ဆောင်ချက် သည်
စာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လည်းကောင်း နေရာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "သို့" ကို သုံး၍ လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"{နေရာ}: {စာ}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// "(I) will write 'hello' to the notebook"
"မင်္ဂလာပါ" ကို "မှတ်စု" သို့ ရေး မည်။
```

```text
မှတ်စု: မင်္ဂလာပါ
```

In the function signature, we can use `ဝိဘတ် "<preposition>" ကို သုံး၍` after the `ကို`
of a parameter to give it a preposition. A call then writes each argument followed with its
preposition, in the order of the parameters, and ends with the function name and `မည်`,
with no `သုံး၍`:

```text
<parameter> ဟူသော <kind> တစ်ခု ကို ဝိဘတ် "<preposition>" ကို သုံး၍

<argument> <preposition> <argument> <preposition> <name> မည်။
```

| Zaka | Reads as |
| --- | --- |
| `စာ ဟူသော စာသား တစ်ခု ကို` | "a text called `စာ`" |
| `ဝိဘတ် "ကို" ကို သုံး၍` | "using the preposition `ကို`" |
| `လည်းကောင်း` | "and also", after each parameter |
| `"မင်္ဂလာပါ" ကို "မှတ်စု" သို့ ရေး မည်` | "will write 'hello' to the notebook" |

A parameter can also have no preposition, and then its argument comes right before the
next one, or `၊` separates the two.

## Prepositions of several words

```text
<parameter> ဟူသော <kind> တစ်ခု ကို ဝိဘတ် "<word> <word>" ကို သုံး၍
```

A preposition can be several words, because Burmese often puts a phrase after a noun:
`"ရန်ကုန်" မှ စ၍` means "starting from Yangon". We write the words inside the quotes of
`ဝိဘတ်`, with the same spaces as in the sentence:

```zaka
// "Function show trip: a start (preposition မှ စ၍) and an end (preposition အထိ)"
ခရီးစဉ်ပြ ဟူသော လုပ်ဆောင်ချက် သည်
စ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "မှ စ၍" ကို သုံး၍ လည်းကောင်း ဆုံး ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "အထိ" ကို သုံး၍ လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"{စ} → {ဆုံး}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// "(I) will show the trip starting from Yangon up to Mandalay"
"ရန်ကုန်" မှ စ၍ "မန္တလေး" အထိ ခရီးစဉ်ပြမည်။
```

```text
ရန်ကုန် → မန္တလေး
```

When the words after an argument can make up a long preposition or a short one, the call
uses the longest one that the function has, so `မှ စ၍` is one preposition here and not
`မှ` followed with a name. The call also glues `မည်` onto the name: because the file has
a function `ခရီးစဉ်ပြ`, the word `ခရီးစဉ်ပြမည်` means the same as `ခရီးစဉ်ပြ မည်`.

## A last preposition with ၏

```text
<argument> ကို <argument> ၏ <word> <name> သည့် ရလဒ်
```

The preposition of the last parameter can start with `၏` ("of"), so that the call means
"at the end of the text" or "in the middle of the list". This function returns a value, so
the call ends with `သည့် ရလဒ်` ("the result that …"), which [Results](results.md#arguments-first-with-a-result)
explains, and it can glue onto the name just as `မည်` does:

```zaka
// "Function join: a piece (preposition ကို) at the end of a text (preposition ၏ နောက်ဆုံးတွင်)"
တွဲ ဟူသော လုပ်ဆောင်ချက် သည်
ပိုင်း ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လည်းကောင်း စာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "၏ နောက်ဆုံးတွင်" ကို သုံး၍ လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"{စာ}{ပိုင်း}" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// "Show the result of joining 'pa' at the end of 'mingala'"
"ပါ" ကို "မင်္ဂလာ" ၏ နောက်ဆုံးတွင် တွဲသည့်ရလဒ် ကို ပြသမည်။
```

```text
မင်္ဂလာပါ
```

Just the last preposition can start with `၏`. In all other places, `x ၏ y` still reads the
field `y` of `x`.

## Phrases: a function as a command

```text
<name> ဟူသော လုပ်ဆောင်ချက် သည်
အစ စကားစု "<words>" နှင့် အဆုံး စကားစု "<words>" တို့ကို သုံး၍
```

Prepositions come after the arguments. A function signature can also add words before the
first argument, a *starting phrase*, and words right before the function name, an *ending
phrase*. We write them right after `သည်`, joined with `နှင့်` and followed with
`တို့ကို သုံး၍`, or a single phrase followed with `ကို သုံး၍`. A function with no
parameters can have phrases too, so that a call is a short command:

```zaka
// "Function read: starting phrase ဖိုင် ('file'), the path (preposition ထဲမှ), ending phrase စာကို ('the text')"
ဖတ် ဟူသော လုပ်ဆောင်ချက် သည်
အစ စကားစု "ဖိုင်" နှင့် အဆုံး စကားစု "စာကို" တို့ကို သုံး၍
လမ်း ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ထဲမှ" ကို သုံး၍ လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"{လမ်း} ထဲမှ စာ" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// "Function clear: no parameters, the words ဖန်သားပြင်ကို ('the screen') before it"
ရှင်း ဟူသော လုပ်ဆောင်ချက် သည် အစ စကားစု "ဖန်သားပြင်ကို" ကို သုံး၍
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ရှင်းပြီ" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// "Show the result of reading the text from the file 'notes.txt'"
ဖိုင် "notes.txt" ထဲမှ စာကို ဖတ် သည့် ရလဒ် ကို ပြသမည်။
// "(I) will clear the screen"
ဖန်သားပြင်ကို ရှင်း မည်။
```

```text
notes.txt ထဲမှ စာ
ရှင်းပြီ
```

| Zaka | Reads as |
| --- | --- |
| `အစ စကားစု "ဖိုင်"` | "the starting phrase 'file'" |
| `အဆုံး စကားစု "စာကို"` | "the ending phrase 'the text'" |
| `ဖိုင် "notes.txt" ထဲမှ စာကို ဖတ်` | "read the text from the file 'notes.txt'" |

A call written as a sentence must write each phrase in its place. The other call forms,
such as `ဖတ်("notes.txt")` and `ရှင်း မည်`, leave the phrases out.

## Names that read as calls

Gluing has a cost. In a file with a function `နာ` ("hurt"), the word `နာမည်` ("name") can
also mean `နာ` followed with `မည်`, so a variable of that name gets the warning ZK0251:

```zaka-warning ZK0251
// "Function hurt": with it in the file, နာမည် ("name") can read as နာ + မည်
နာ ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"အား" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် နာမည် သည် "မောင်" ဖြစ်ပါစေ။
{နာမည်} ကို ပြသမည်။
```

To keep both meanings apart, we can write a name in braces, `{<name>}`, which is a name and
never a call. So we declare the variable as `{နာမည်}` and write `{နာမည်}` each time we mean
the variable, while the bare word `နာမည်` is the call:

```zaka
နာ ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"အား" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် {နာမည်} သည် "မောင်" ဖြစ်ပါစေ။
{နာမည်} ကို ပြသမည်။
နာမည်။
```

```text
မောင်
အား
```

In a file with no function `နာ`, `နာမည်` is a plain name, with no warning.

## One name, several functions

Two functions can share a name when the data kinds or the prepositions of their parameters
differ, which other languages call *overloading*. Each call then runs the function that its
arguments fit:

```zaka
ပြ ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ကိန်း {န}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပြ ဟူသော လုပ်ဆောင်ချက် သည်
စ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် "ကို" ကို သုံး၍ လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"စာ {စ}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

၅ ကို ပြ မည်။            // the integer one
"ဟယ်လို" ကို ပြ မည်။      // the text one
```

```text
ကိန်း ၅
စာ ဟယ်လို
```

## Details

- A preposition that starts with `၏` on a parameter other than the last, or before an
  ending phrase, is the error ZK0249
  ([Prepositions](../../manual/language/functions-sentences.md#prepositions)).
- A call written as a sentence that leaves out a phrase, or writes a different one, is
  ZK0352. A function with phrases can still have parameters with no preposition:
  `ကိန်း ၄၊ ၅ ပေါင်း` passes two bare arguments ([Starting and ending
  phrases](../../manual/language/functions-sentences.md#starting-and-ending-phrases)).
- `သည့် ရလဒ်` and `မည်` glue onto a name when the file has a function of that name. The
  full list of what this flexible grammar costs is in [What the flexible grammar
  costs](../../manual/language/functions-sentences.md#what-the-flexible-grammar-costs).
- When no function of the name fits the arguments, the compiler reports ZK0332, and when
  two fit, ZK0333. Two functions with the same name, data kinds and prepositions are
  ZK0304 ([One name, several
  functions](../../manual/language/functions-overloads.md)).

## Recap

- We use `ဝိဘတ် "<preposition>" ကို သုံး၍` after the `ကို` of a parameter to give it a
  preposition, and a call then writes each argument with its preposition, before the
  function name and `မည်`.
- A preposition can be several words, and the last one can start with `၏`.
- `အစ စကားစု` and `အဆုံး စကားစု` add words before the arguments and before the name.
- A name in braces, such as `{နာမည်}`, is a name and never a call.
- Several functions can share a name when their parameters differ.

## Hands on

- Write a function `ပို့` with the prepositions `ကို` and `ထံသို့` ("to a person"), and call
  it as a sentence.
- Give `ရှင်း` an ending phrase as well, and call it with and without the phrase.
- Add a third `ပြ` that accepts a decimal number, and call all three.

Next: [Recursion](recursion.md).
