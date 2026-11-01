---
title: Natural wording
description: The rules of the naturalness check, which warns when a sentence is valid Zaka but reads unlike Burmese, with the wording it prefers for each.
---

# Natural wording

Zaka accepts more spellings than a Burmese speaker would write. A verb at the end of a
clause can hold `သည်`, `ပါ`, `ပါသည်` or no ending at all, and a printed operand can be
marked with `ကို` or `ဟု`. The parser takes each of them. A separate pass, the
*naturalness check*, runs right after the parser and points out the spellings that read
unlike Burmese. Its findings are warnings with codes ZK0501 to ZK0508. A warning never
stops a program and never changes what it does. The Learn book introduces the idea in
[Natural Zaka](../../learn/natural-zaka/README.md).

## What the check looks at

The check reads the tree of the program, not the text, so spacing does not change its
answers. It looks at four kinds of place:

| Place | What is examined |
| --- | --- |
| a clause-final word | a verb such as `ကြီး`, `ညီ`, `မငယ်`, the words `မှန်` and `မမှန်`, or `မဟုတ်`; its ending; and what follows it |
| a printed operand | whether it is a sentence, a string or a plain value, and whether it is marked with `ကို` or `ဟု` |
| a clause that gives a value | the ending of its last word, when the clause is stored, returned, passed or used as a loop bound |
| a group in parentheses | whether the parentheses change the meaning |

"What follows" matters because the ending depends on the next word: `သို့မဟုတ်` ("or"),
`ဖြစ်၍` ("and"), an attached `လျှင်` or `နေသရွေ့`, or the end of the clause.

When two rules fit one place, the first in the table below alone is reported. So
`မဟုတ်သည်` before `သို့မဟုတ်` is reported once, as ZK0501.

## The rules

| Code | Where | Prefers | Warns about |
| --- | --- | --- | --- |
| ZK0501 | a "not" verb before `သို့မဟုတ်` | `မငယ်ပါ သို့မဟုတ်` | `မငယ်သည် သို့မဟုတ်` |
| ZK0502 | a verb before an attached `လျှင်` or `နေသရွေ့` | `ကြီးလျှင်`, `ကြီးနေသရွေ့` | `ကြီးသည်လျှင်` |
| ZK0503 | `မဟုတ်` ("not") | `မဟုတ်ပါ` or `မဟုတ်` | `မဟုတ်သည်` |
| ZK0504 | a verb before `ဖြစ်၍` ("and") | `ကြီးသည် ဖြစ်၍` | `ကြီးပါ ဖြစ်၍` |
| ZK0505 | printing a sentence or a string | `"..." ဟု ပြသပါ` | `"..." ကို ပြသပါ` |
| ZK0506 | printing a plain value | `အသက် ကို ပြသပါ` | `အသက် ဟု ပြသပါ` |
| ZK0507 | parentheses that change nothing | `အသက် + ၁ ကို ပြသပါ` | `(အသက် + ၁) ကို ပြသပါ` |
| ZK0508 | a sentence used as a value | `... ကြီးသည် ဖြစ်ပါစေ` | `... ကြီးပါ ဖြစ်ပါစေ` |

The right-hand column is still valid Zaka. It just sounds less natural. The "not" verbs
are `မညီ`, `မကြီး`, `မငယ်`, `မမှန်` and `မဟုတ်`.

### ZK0501: "not" before "or"

Before `သို့မဟုတ်` a negative verb takes `ပါ`.

```zaka-warning ZK0501
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
(အသက် သည် ၁၈ အောက် မငယ်သည် သို့မဟုတ် အသက် သည် ၆၀ ထက် ကြီးသည်) ဟု ပြသပါ။
```

```text
warning[ZK0501]: "မငယ်သည်" before "သို့မဟုတ်" reads better as "မငယ်ပါ"
```

### ZK0502: a particle before an attached "လျှင်" or "နေသရွေ့"

The attached forms join straight onto the verb: `ကြီးလျှင်`, `ကြီးနေသရွေ့`. A particle
between them is unnatural. To keep the particle, use the `ဖြစ်` forms: `ကြီးသည် ဖြစ်လျှင်`,
`ကြီးသည် ဖြစ်နေသရွေ့`.

```zaka-warning ZK0502
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
အကယ်၍ အသက် သည် ၁၈ ထက် ကြီးသည်လျှင် "အရွယ်ရောက်" ဟု ပြသပါ။
```

```text
warning[ZK0502]: "ကြီးသည်" before "လျှင်" reads better without the particle
help: write "ကြီးလျှင်", or keep the particle and write "ဖြစ်လျှင်"
```

The same holds for a loop:

```zaka-warning ZK0502
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
က သည် ၀ ထက် ကြီးသည်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
က ကို ပြသပါ။
က ကို က - ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
```

```text
warning[ZK0502]: "ကြီးသည်" before "နေသရွေ့" reads better without the particle
```

### ZK0503: "မဟုတ်" with "သည်"

The postfix negation takes `ပါ` or nothing at all. `မဟုတ်သည်` and `မဟုတ်ပါသည်` are
grammatical but not idiomatic.

```zaka-warning ZK0503
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
(အသက် သည် ၁၈ နှင့် ညီသည် မဟုတ်သည်) ဟု ပြသပါ။
```

```text
warning[ZK0503]: "မဟုတ်သည်" reads better as "မဟုတ်ပါ" or "မဟုတ်"
```

### ZK0504: "ပါ" before "ဖြစ်၍"

Before the conjunction `ဖြစ်၍` ("and") a verb takes `သည်`. The `ပါ` forms belong before
`သို့မဟုတ်` and at the end of a quoted sentence.

```zaka-warning ZK0504
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
(အသက် သည် ၁၈ ထက် ကြီးပါ ဖြစ်၍ အသက် သည် ၆၀ အောက် ငယ်သည်) ဟု ပြသပါ။
```

```text
warning[ZK0504]: "ကြီးပါ" before "ဖြစ်၍" reads better as "ကြီးသည်"
```

### ZK0505 and ZK0506: "ဟု" and "ကို" in a print

`ကို` marks a *thing*: a name, a number, the result of a sum. `ဟု` quotes a *sentence* or
a string: a comparison that ends in `သည်` or `ပါ`, a string literal (with template slots or not), or
a `+` join that holds a string literal. A bare clause with no ending is a thing and keeps
`ကို`, and so is a text variable: `အမည် ကို ပြသပါ`.

```zaka-warning ZK0505
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
"နှစ်ဆယ်" ကို ပြသပါ။
အသက် သည် ၁၈ ထက် ကြီးသည် ကို ပြသပါ။
```

```text
warning[ZK0505]: a sentence or a string is quoted with "ဟု", not marked with "ကို"
warning[ZK0505]: a sentence or a string is quoted with "ဟု", not marked with "ကို"
```

```zaka-warning ZK0506
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
အသက် + ၁ ဟု ပြသပါ။
```

```text
warning[ZK0506]: a plain value is marked with "ကို", not quoted with "ဟု"
```

### ZK0507: parentheses that change nothing

A group in parentheses is redundant when it is the whole operand of its place: a printed
value, an assigned value, a returned value, an initial value, a loop bound, a call
argument or a condition. It is also redundant inside another group. Such an operand is
read as far as it goes, so the parentheses add nothing.

Parentheses are needed, and left alone, around a logical expression (one with `ဖြစ်၍`,
`သို့မဟုတ်` or `မဟုတ်`) in an operand place, and inside an operator chain or a clause with
more operands.

```zaka-warning ZK0507
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
(အသက် + ၁) ကို ပြသပါ။
```

```text
warning[ZK0507]: these parentheses are not needed
```

### ZK0508: "ပါ" on a clause used as a value

A clause that gives a value, as an initial value, a returned value, a call argument or a
loop bound, names a fact and does not state a sentence, so its last word takes `သည်`. The
`ပါ` forms belong to quoted sentences (`... ဟု ပြသပါ`, `... ဟု ထားပါ`) and to the place
before `သို့မဟုတ်`.

```zaka-warning ZK0508
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
ကိန်းရှင် ကြီးသူ သည် မှန်မှားတန်ဖိုး ဖြစ်၍ တန်ဖိုးသည် အသက် သည် ၁၈ ထက် ကြီးပါ ဖြစ်ပါစေ။
```

```text
warning[ZK0508]: "ကြီးပါ" as a value reads better as "ကြီးသည်"
```

## All of the rules, followed

The same sentences in the wording the check prefers print the same and warn about
nothing:

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
(အသက် သည် ၁၈ အောက် မငယ်ပါ သို့မဟုတ် အသက် သည် ၆၀ ထက် ကြီးသည်) ဟု ပြသပါ။
(အသက် သည် ၁၈ ထက် ကြီးသည် ဖြစ်၍ အသက် သည် ၆၀ အောက် ငယ်သည်) ဟု ပြသပါ။
(အသက် သည် ၁၈ နှင့် ညီသည် မဟုတ်ပါ) ဟု ပြသပါ။
အကယ်၍ အသက် သည် ၁၈ ထက် ကြီးလျှင် "အရွယ်ရောက်" ဟု ပြသပါ။
အသက် + ၁ ကို ပြသပါ။
ကိန်းရှင် ကြီးသူ သည် မှန်မှားတန်ဖိုး ဖြစ်၍ တန်ဖိုးသည် အသက် သည် ၁၈ ထက် ကြီးသည် ဖြစ်ပါစေ။
ကြီးသူ ကို ပြသပါ။
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

`--strict-particles` turns each of these warnings into an error, so a program that has
one does not run. This is useful once the rules feel familiar and each new program should
read as Burmese.

```sh
zakac run --strict-particles program.zaka
```

The flag changes how the findings are reported and nothing more. For how the
compiler collects and shows diagnostics, see [Diagnostics](../compiler/diagnostics.md).
