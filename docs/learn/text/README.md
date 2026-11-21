---
title: Text
description: Text in double quotes is a value of the data kind စာသား. Store it, put values inside it with template slots, join it, compare it and duplicate it.
---

# Text

Text (string) is a row of characters in double quotes, such as `"မင်္ဂလာပါ"`, and its
data kind (type) is `စာသား` ("text"). We can print text, store it in a variable, pass it to
a function, return it from one, and join two pieces of it into one.

## You will learn

- how to write text and store it in a variable
- how to put values inside text with template slots
- how to join two pieces of text and compare them
- what happens when we assign text to another variable

```zaka
"မင်္ဂလာပါ" ဟု ပြသမည်။
"ဆို \"ဟယ်လို\"" ဟု ပြသမည်။               // \" puts a double quote inside
"ပထမကြောင်း{နောက်တစ်ကြောင်း}ဒုတိယကြောင်း" ဟု ပြသမည်။
```

```text
မင်္ဂလာပါ
ဆို "ဟယ်လို"
ပထမကြောင်း
ဒုတိယကြောင်း
```

Text that we write in double quotes is a *string literal*:

```text
"<characters>"
```

| Zaka | Reads as | What it does |
| --- | --- | --- |
| `"…"` | the text itself | a string literal, of the data kind `စာသား` |
| `\"` | a double quote | puts a `"` inside the text |
| `{နောက်တစ်ကြောင်း}` | "next line" | starts a new line inside the text |

A string literal uses the same [escape sequences](../values/characters.md#escape-sequences)
as a character does. When we print a string literal, we write `ဟု` after it, which quotes
it, rather than `ကို`.

## Storing text

```text
ကိန်းရှင် <name> သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "<characters>" ဖြစ်ပါစေ။
```

We declare a text variable as we declare a number variable, with `စာသား` as the data kind.
The variable is a plain value, so we print it with `ကို`, the same as a number.

```zaka
ကိန်းရှင် အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မောင်မောင်" ဖြစ်ပါစေ။
အမည် ကို ပြသမည်။
အမည် ကို "အောင်အောင်" ဟု ထားမည်။
အမည် ကို ပြသမည်။
```

```text
မောင်မောင်
အောင်အောင်
```

The initial value (`ကနဦးတန်ဖိုး`) of `စာသား` is `""`, text with no characters, which
prints as a blank line.

## Template literals

```text
"… {<expression>} …"
```

To put a value inside text, we write an expression in braces inside the quotes. The pair of
braces is a *template slot*, and text with slots in it is a *template literal* (string
interpolation). When the program runs, it writes the value of the expression in place of
the slot, the same as `ပြသမည်` would print it:

```zaka
ကိန်းရှင် အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မောင်မောင်" ဖြစ်ပါစေ။
ကိန်းရှင် အသက် သည် ၂၅ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
"{အမည်} သည် အသက် {အသက်} နှစ် ဖြစ်သည်" ဟု ပြသမည်။
"နောက်နှစ်တွင် {အသက် + ၁} နှစ်" ဟု ပြသမည်။
"{၁.၅} {မှန်} {'က'} {[၁၊ ၂၊ ၃]}" ဟု ပြသမည်။
"{{ မဟုတ် }}" ဟု ပြသမည်။                 // a doubled brace is a brace
```

```text
မောင်မောင် သည် အသက် ၂၅ နှစ် ဖြစ်သည်
နောက်နှစ်တွင် ၂၆ နှစ်
၁.၅ မှန် က [၁၊ ၂၊ ၃]
{ မဟုတ် }
```

To write a brace as part of the text, we double it: `{{` and `}}`. The escape names, such as
`{နောက်တစ်ကြောင်း}`, keep their meaning and are not slots. A slot ends at the `}` that
matches its `{`, so we can put a string literal with slots inside a slot:

```zaka
ကိန်းရှင် အမှတ် သည် ၉၅ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
"ရလဒ်: {"အမှတ် {အမှတ်}" + " (အောင်)"}" ဟု ပြသမည်။
```

```text
ရလဒ်: အမှတ် ၉၅ (အောင်)
```

When the expression in a slot has a mistake, the compiler points at its place inside the
quotes:

```zaka-error ZK0301
"{မသိ}" ဟု ပြသမည်။
```

```text
error[ZK0301]: "မသိ" is not declared
```

A text constant can use slots too, as long as each slot contains a value that is known when
the program is compiled, such as a literal or another constant. Each read of the constant
is a fresh duplicate of its text, so the constant keeps its value:

```zaka
ကိန်းသေ အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မမ" ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "ဟေး {အမည်}" ဖြစ်ပါစေ။
ခ ကို ပြသမည်။
အမည် ကို ပြသမည်။
```

```text
ဟေး မမ
မမ
```

## Joining text

```text
<text> + <text>
<text>၊ <text> နှင့် <text> တို့ကို ဆက်ထားသည့် စာသား
```

We use `+` to join two pieces of text into new text:

```zaka
ကိန်းရှင် အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မောင်မောင်" ဖြစ်ပါစေ။
ကိန်းရှင် နှုတ်ဆက် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မင်္ဂလာပါ " + အမည် ဖြစ်ပါစေ။
နှုတ်ဆက် ကို ပြသမည်။
နှုတ်ဆက် ကို နှုတ်ဆက် + "!" ဟု ထားမည်။
နှုတ်ဆက် ကို ပြသမည်။
```

```text
မင်္ဂလာပါ မောင်မောင်
မင်္ဂလာပါ မောင်မောင်!
```

To join several pieces in one sentence, we can also use the word form: `၊` between the
pieces, `နှင့်` before the last one, and `တို့ကို ဆက်ထားသည့် စာသား` after it.

```zaka
ကိန်းရှင် အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မောင်မောင်" ဖြစ်ပါစေ။
"မင်္ဂလာပါ "၊ အမည် နှင့် "!" တို့ကို ဆက်ထားသည့် စာသား ကို ပြသမည်။
```

```text
မင်္ဂလာပါ မောင်မောင်!
```

| Zaka | Reads as |
| --- | --- |
| `တို့ကို` | "these" |
| `ဆက်ထားသည့်` | "that joins" |
| `စာသား` | "text" |

We can join text to text alone, so `+` with a number is a mistake:

```zaka-error ZK0403
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "အဆင့် " ဖြစ်ပါစေ။
စာ + ၁ ကို ပြသမည်။
```

```text
error[ZK0403]: + needs a number, found text
```

To put a number into text, we use a template slot instead: `"အဆင့် {၁}"`.

## Comparing text

```text
<text> == <text>
<text> != <text>
```

We use `==` and `!=` to check whether two pieces of text have the same characters:

```zaka
ကိန်းရှင် အမည် သည် "မောင်မောင်" ဖြစ်ပါစေ။
အမည် == "မောင်မောင်" ကို ပြသမည်။
အမည် != "မမ" ကို ပြသမည်။
```

```text
မှန်
မှန်
```

## Duplicating text

A `စာသား` variable owns its text. When we assign it to another variable, the text is
duplicated (copied), so each variable has its own text, and a change to one leaves the other
as it was:

```zaka
ကိန်းရှင် ပထမ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "Zaka" ဖြစ်ပါစေ။
ကိန်းရှင် ဒုတိယ သည် စာသား တစ်ခု ဖြစ်ပါစေ။
ဒုတိယ ကို ပထမ ဟု ထားမည်။
ဒုတိယ ကို ဒုတိယ + "!" ဟု ထားမည်။
ပထမ ကို ပြသမည်။
ဒုတိယ ကို ပြသမည်။
```

```text
Zaka
Zaka!
```

When we pass text to a function, the parameter gets a duplicate too, so the function cannot
change the caller's variable, even when it writes a character or adds to the text. When the
first variable is not read again, nothing is duplicated, because the compiler hands its
text over to the second. Part II explains the cost in [What a duplicate
costs](../forms/costs.md).

## Details

- A string literal must close on the line where it opens (ZK0104), and so must a template
  slot (ZK0106). A slot with nothing in it, `"{}"`, is ZK0202 ([Template
  literals](../../manual/language/text.md#template-literals)).
- When each slot contains a literal or a constant, the compiler builds the whole text, so
  the program does no work for it when it runs ([Folding](../../manual/language/text-storage.md#folding)).
- A text constant whose slot needs the program to run, such as a slot with a variable,
  is ZK0606, so we declare such text with `ကိန်းရှင်` ([Constants of text and
  lists](../forms/counted-lists.md#constants-of-text-and-lists)).
- `==` compares the text as it is stored, so two spellings of one word whose code points
  differ are not equal. `<` and the other orderings do not work on text (ZK0403)
  ([Comparing](../../manual/language/text.md#comparing)).
- A string literal, or a join with `+` that contains one, is printed with `ဟု`, while a
  variable or a call is printed with `ကို`. The other choice still runs, with the warning
  ZK0505 or ZK0506 ([Printing](../../manual/language/text.md#printing)).
- The text module of the stdlib, `"áá¶áá¬á¸/áá¬áá¬á¸"`, has functions to split text, to join it
  and to search it ([The stdlib](../stdlib/README.md#text)).

## Recap

- We write text in double quotes, and its data kind is `စာသား`.
- We write `{<expression>}` inside the quotes to put a value into the text, and `{{` or `}}`
  for a brace.
- We use `+`, or `… နှင့် … တို့ကို ဆက်ထားသည့် စာသား`, to join pieces of text, and `==` or
  `!=` to compare them.
- Assigning text or passing it to a function makes a duplicate, so the original does not
  change.

## Hands on

- Print a short poem, one line for each `ပြသမည်`.
- Print the same poem with a single `ပြသမည်`, using `{နောက်တစ်ကြောင်း}`.
- Store a name and an age, and print a sentence about them with two template slots.
- Put a name into a second variable, change the second, and print both.

Next: [Cells and characters](characters.md).
