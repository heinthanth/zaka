---
title: Text
description: Text in double quotes is a value of the kind စာသား - store it, fill template slots in it with values, join it, compare it and duplicate it.
---

# Text

Text in double quotes, such as `"မင်္ဂလာပါ"`, is a *string literal*. Its kind is `စာသား`
("text"). A program can print text, store it in a variable, pass it to a function,
return it from one, and join two pieces of it.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ" ဟု ပြသမည်။
"ဆို \"ဟယ်လို\"" ဟု ပြသမည်။               // \" puts a double quote inside
"ပထမကြောင်း{နောက်တစ်ကြောင်း}ဒုတိယကြောင်း" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ
ဆို "ဟယ်လို"
ပထမကြောင်း
ဒုတိယကြောင်း
```

Strings use the same [escape sequences](../values/characters.md#escape-sequences) as characters.
Print a string literal with `ဟု`, which quotes it, rather than `ကို`.

## Storing text

A variable of the kind `စာသား` holds text. The variable is a plain value, so a print marks
it with `ကို`, as it would a number.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မောင်မောင်" ဖြစ်ပါစေ။
အမည် ကို ပြသမည်။
အမည် ကို "အောင်အောင်" ဟု ထားမည်။
အမည် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မောင်မောင်
အောင်အောင်
```

`ကနဦးတန်ဖိုး` for `စာသား` is `""`, text with no characters, which prints as a blank line.

## Template literals

A pair of braces inside a string is a *template slot*. It holds an expression, and the program
writes the value of that expression in its place, the same as `ပြသမည်` would print it.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မောင်မောင်" ဖြစ်ပါစေ။
ကိန်းရှင် အသက် သည် ၂၅ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
"{အမည်} သည် အသက် {အသက်} နှစ် ဖြစ်သည်" ဟု ပြသမည်။
"နောက်နှစ်တွင် {အသက် + ၁} နှစ်" ဟု ပြသမည်။
"{၁.၅} {မှန်} {'က'} {[၁၊ ၂၊ ၃]}" ဟု ပြသမည်။
"{{ မဟုတ် }}" ဟု ပြသမည်။                 // a doubled brace is a brace
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မောင်မောင် သည် အသက် ၂၅ နှစ် ဖြစ်သည်
နောက်နှစ်တွင် ၂၆ နှစ်
၁.၅ မှန် က [၁၊ ၂၊ ၃]
{ မဟုတ် }
```

A template slot ends at the `}` that matches its `{`, so a template slot can hold text in quotes with
template slots of its own:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ် သည် ၉၅ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
"ရလဒ်: {"အမှတ် {အမှတ်}" + " (အောင်)"}" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရလဒ်: အမှတ် ၉၅ (အောင်)
```

A brace that is part of the text is doubled: `{{` and `}}`. The escape names, such as
`{နောက်တစ်ကြောင်း}`, keep their old meaning.

A mistake inside a template slot is reported at its place inside the quotes:

```zaka-error ZK0301
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"{မသိ}" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0301]: "မသိ" is not declared
```

When each template slot holds a value the compiler knows, such as a literal or a
[constant](../variables/constants.md), the compiler writes the whole text itself, and the
program does no work for it when it runs.

A text constant is such text too. Its value must be known when the program is compiled,
and each read of it is a duplicate, so it is never left blank:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မမ" ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "ဟေး {အမည်}" ဖြစ်ပါစေ။
ခ ကို ပြသမည်။
အမည် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဟေး မမ
မမ
```

A text constant built from a variable is an error (ZK0606): declare such text with
`ကိန်းရှင်`.

## Joining text

`+` joins two pieces of text into new text.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မောင်မောင်" ဖြစ်ပါစေ။
ကိန်းရှင် နှုတ်ဆက် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မင်္ဂလာပါ " + အမည် ဖြစ်ပါစေ။
နှုတ်ဆက် ကို ပြသမည်။
နှုတ်ဆက် ကို နှုတ်ဆက် + "!" ဟု ထားမည်။
နှုတ်ဆက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ မောင်မောင်
မင်္ဂလာပါ မောင်မောင်!
```

Several pieces join in one sentence too: `၊` between them, `နှင့်` before the last, and
`တို့ကို ဆက်ထားသည့် စာသား` ("the text that joins these") after it:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မောင်မောင်" ဖြစ်ပါစေ။
"မင်္ဂလာပါ "၊ အမည် နှင့် "!" တို့ကို ဆက်ထားသည့် စာသား ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ မောင်မောင်!
```

Text joins to text alone. `+` with a number is an error:

```zaka-error ZK0403
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "အဆင့် " ဖြစ်ပါစေ။
စာ + ၁ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0403]: + needs a number, found text
```

To put a number into text, use a template slot: `"အဆင့် {၁}"`.

## Comparing text

`==` and `!=` tell whether two pieces of text hold the same characters:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည် သည် "မောင်မောင်" ဖြစ်ပါစေ။
အမည် == "မောင်မောင်" ကို ပြသမည်။
အမည် != "မမ" ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
မှန်
```

## Copies of text

A `စာသား` variable owns its text. Putting it into another variable *copies* the text: each
variable then holds its own, and a change to one leaves the other as it was.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပထမ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "Zaka" ဖြစ်ပါစေ။
ကိန်းရှင် ဒုတိယ သည် စာသား တစ်ခု ဖြစ်ပါစေ။
ဒုတိယ ကို ပထမ ဟု ထားမည်။
ဒုတိယ ကို ဒုတိယ + "!" ဟု ထားမည်။
ပထမ ကို ပြသမည်။
ဒုတိယ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
Zaka
Zaka!
```

Passing text to a function copies it too, so a function cannot change the caller's
variable, not even with a character write or an add. When the first variable is not read again, nothing is copied: the compiler
hands its text over to the second. Part II tells what copies cost: see
[What a duplicate costs](../forms/costs.md).

The stdlib holds no helper to split text, so a program writes that for now: see
[What the stdlib holds](../../manual/language/io.md#what-the-stdlib-holds).

## Hands on

- Print a short poem, one line per `ပြသမည်`.
- Print the same poem with a single `ပြသမည်`, using `{နောက်တစ်ကြောင်း}`.
- Store a name and an age, and print a sentence about them with two template slots.
- Put a name into a second variable, change the second, and print both.

Next: [Cells and characters](characters.md).
