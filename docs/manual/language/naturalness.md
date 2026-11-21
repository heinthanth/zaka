---
title: Natural wording
description: The rules of the naturalness check, which warns when a sentence is valid Zaka but does not sound like Burmese, with the wording it prefers for each case.
---

# Natural wording

Zaka accepts more spellings than a Burmese speaker would write. A verb at the end of a
clause can end in `သည်`, `ပါ`, `ပါသည်` or nothing at all, and we can mark a printed operand
with `ကို` or `ဟု`, and the parser accepts each of these. A separate pass, the *naturalness
check* (a lint of wording), runs right after the parser and points out the spellings that do
not sound like Burmese. Its findings are warnings with the codes ZK0501 to ZK0509. A
warning never stops a program and never changes what it does. The Learn book introduces
the idea in [Natural Zaka](../../learn/natural-zaka/README.md).

## What the check looks at

The check walks the tree of the program, not the text, so spacing does not change its
answers. It looks at five kinds of place:

| Place | What it examines |
| --- | --- |
| a clause-final word | a verb such as `ကြီး`, `ညီ` or `မငယ်`, the words `မှန်` and `မမှန်`, or `မဟုတ်`, together with its ending and what follows it |
| a printed operand | whether it is a sentence, a string or a plain value, and whether it is marked with `ကို` or `ဟု` |
| a clause that gives a value | the ending of its last word, when the clause is stored, returned, passed or used as a loop bound |
| a group in parentheses | whether the parentheses change the meaning |
| a measure word (classifier) | whether it fits the data kind before it |

What follows a word matters because the right ending depends on the next word: `သို့မဟုတ်`
("or"), `ဖြစ်၍` ("and"), an attached `လျှင်` or `နေသရွေ့`, or the end of the clause.

When two rules fit one place, the check reports the first one in the table below alone. So
`မဟုတ်သည်` before `သို့မဟုတ်` is reported once, as ZK0501.

## The rules

| Code | Where | Prefers | Warns about |
| --- | --- | --- | --- |
| ZK0501 | a "not" verb before `သို့မဟုတ်` | `မငယ်ပါ သို့မဟုတ်` | `မငယ်သည် သို့မဟုတ်` |
| ZK0502 | a verb before an attached `လျှင်` or `နေသရွေ့` | `ကြီးလျှင်`, `ကြီးနေသရွေ့` | `ကြီးသည်လျှင်` |
| ZK0503 | `မဟုတ်` ("not") | `မဟုတ်ပါ` or `မဟုတ်` | `မဟုတ်သည်` |
| ZK0504 | a verb before `ဖြစ်၍` ("and") | `ကြီးသည် ဖြစ်၍` | `ကြီးပါ ဖြစ်၍` |
| ZK0505 | printing a sentence or a string | `"..." ဟု ပြသမည်` | `"..." ကို ပြသမည်` |
| ZK0506 | printing a plain value | `အသက် ကို ပြသမည်` | `အသက် ဟု ပြသမည်` |
| ZK0507 | parentheses that change nothing | `အသက် + ၁ ကို ပြသမည်` | `(အသက် + ၁) ကို ပြသမည်` |
| ZK0508 | a sentence used as a value | `... ကြီးသည် ဖြစ်ပါစေ` | `... ကြီးပါ ဖြစ်ပါစေ` |
| ZK0509 | the measure word (classifier) after a data kind | `ကိန်းပြည့် တစ်ခု`, `လူ တစ်ယောက်` ([Measure words](kinds.md#measure-words)) | `ကိန်းပြည့် တစ်ယောက်` |

The right-hand column is still valid Zaka, but it sounds less natural. The "not" verbs are
`မညီ`, `မကြီး`, `မငယ်`, `မမှန်` and `မဟုတ်`.

### ZK0501: "not" before "or"

Before `သို့မဟုတ်`, a negative verb ends in `ပါ`.

```zaka-warning ZK0501
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
(အသက် သည် ၁၈ အောက် မငယ်သည် သို့မဟုတ် အသက် သည် ၆၀ ထက် ကြီးသည်) ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0501]: "မငယ်သည်" before "သို့မဟုတ်" reads better as "မငယ်ပါ"
```

### ZK0502: a particle before an attached "လျှင်" or "နေသရွေ့"

The attached forms join straight onto the verb, as in `ကြီးလျှင်` and `ကြီးနေသရွေ့`, and a
particle between them does not sound natural. To keep the particle, we use the `ဖြစ်` forms
instead: `ကြီးသည် ဖြစ်လျှင်` and `ကြီးသည် ဖြစ်နေသရွေ့`.

```zaka-warning ZK0502
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
အကယ်၍ အသက် သည် ၁၈ ထက် ကြီးသည်လျှင် "အရွယ်ရောက်" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0502]: "ကြီးသည်" before "လျှင်" reads better without the particle
help: write "ကြီးလျှင်", or keep the particle and write "ဖြစ်လျှင်"
```

The rule is the same for a loop:

```zaka-warning ZK0502
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
က သည် ၀ ထက် ကြီးသည်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသမည်။
က ကို က - ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0502]: "ကြီးသည်" before "နေသရွေ့" reads better without the particle
```

### ZK0503: "မဟုတ်" with "သည်"

The negation that comes after a clause ends in `ပါ` or in nothing at all. `မဟုတ်သည်` and
`မဟုတ်ပါသည်` are grammatical but not idiomatic.

```zaka-warning ZK0503
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
(အသက် သည် ၁၈ နှင့် ညီသည် မဟုတ်သည်) ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0503]: "မဟုတ်သည်" reads better as "မဟုတ်ပါ" or "မဟုတ်"
```

### ZK0504: "ပါ" before "ဖြစ်၍"

Before the conjunction `ဖြစ်၍` ("and"), a verb ends in `သည်`. The `ပါ` forms belong before
`သို့မဟုတ်` and at the end of a quoted sentence.

```zaka-warning ZK0504
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
(အသက် သည် ၁၈ ထက် ကြီးပါ ဖြစ်၍ အသက် သည် ၆၀ အောက် ငယ်သည်) ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0504]: "ကြီးပါ" before "ဖြစ်၍" reads better as "ကြီးသည်"
```

### ZK0505 and ZK0506: "ဟု" and "ကို" in a print

We use `ကို` to mark a *thing*, such as a name, a number or the result of a sum, and `ဟု`
to quote a *sentence* or a string: a comparison that ends in `သည်` or `ပါ`, a string
literal (with template slots or without), or a `+` join that contains a string literal. A
bare clause with no ending is a thing and keeps `ကို`, and so is a text variable, as in
`အမည် ကို ပြသမည်`.

```zaka-warning ZK0505
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
"နှစ်ဆယ်" ကို ပြသမည်။
အသက် သည် ၁၈ ထက် ကြီးသည် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0505]: a sentence or a string is quoted with "ဟု", not marked with "ကို"
warning[ZK0505]: a sentence or a string is quoted with "ဟု", not marked with "ကို"
```

```zaka-warning ZK0506
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
အသက် + ၁ ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0506]: a plain value is marked with "ကို", not quoted with "ဟု"
```

### ZK0507: parentheses that change nothing

A group in parentheses is redundant when it is the whole operand of its place: a printed
value, an assigned value, a returned value, an initial value, a loop bound, a call
argument or a condition. It is also redundant inside another group. The parser reads such
an operand as far as it goes, so the parentheses add nothing.

The check leaves parentheses alone where the parentheses are needed: around a logical expression (one
with `ဖြစ်၍`, `သို့မဟုတ်` or `မဟုတ်`) in an operand place, and inside an operator chain or
a clause with more operands. Parentheses are also needed around a verb that joins its
operands with `နှင့်`, such as `(က နှင့် ခ ၏ ပေါင်းလဒ်)`, because as an item of a list in
words, its `နှင့်` would join the next item without them ([Lists in
words](words.md#lists-in-words)).

```zaka-warning ZK0507
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
(အသက် + ၁) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0507]: these parentheses are not needed
```

### ZK0508: "ပါ" on a clause used as a value

A clause that gives a value, such as an initial value, a returned value, a call argument or
a loop bound, names a fact rather than stating a sentence, so its last word ends in `သည်`.
The `ပါ` forms belong to quoted sentences (`... ဟု ပြသမည်`, `... ဟု ထားမည်`) and to the place
before `သို့မဟုတ်`.

```zaka-warning ZK0508
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
ကိန်းရှင် ကြီးသူ သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အသက် သည် ၁၈ ထက် ကြီးပါ ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0508]: "ကြီးပါ" as a value reads better as "ကြီးသည်"
```

## All of the rules, followed

When we write the same sentences in the wording that the check prefers, the sentences print the
same and get no warning:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
(အသက် သည် ၁၈ အောက် မငယ်ပါ သို့မဟုတ် အသက် သည် ၆၀ ထက် ကြီးသည်) ဟု ပြသမည်။
(အသက် သည် ၁၈ ထက် ကြီးသည် ဖြစ်၍ အသက် သည် ၆၀ အောက် ငယ်သည်) ဟု ပြသမည်။
(အသက် သည် ၁၈ နှင့် ညီသည် မဟုတ်ပါ) ဟု ပြသမည်။
အကယ်၍ အသက် သည် ၁၈ ထက် ကြီးလျှင် "အရွယ်ရောက်" ဟု ပြသမည်။
အသက် + ၁ ကို ပြသမည်။
ကိန်းရှင် ကြီးသူ သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အသက် သည် ၁၈ ထက် ကြီးသည် ဖြစ်ပါစေ။
ကြီးသူ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
မှန်
မှန်
အရွယ်ရောက်
၂၁
မှန်
```

## Strict mode

```sh
zakac run --lint-error naturalness program.zaka
```

We can use `--lint-error naturalness` to turn each of these warnings into an error, so
that a program with one does not run. This helps once the rules feel familiar and we want
each new program to sound like Burmese.

`--lint-allow`, `--lint-warn` and `--lint-error` set one rule or the group `naturalness`,
and a rule that we name wins over its group, so `--lint-error naturalness --lint-warn
ZK0505` keeps ZK0505 a warning. [Spoken Burmese](spoken.md) describes the two codes of the
spoken spellings, ZK0510 and ZK0511.

The flag changes how the findings are reported and nothing else. For how the compiler
collects and shows diagnostics, see [Diagnostics](../compiler/diagnostics.md).
