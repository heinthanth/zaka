---
title: Natural Zaka
description: Choose the sentence endings and particles a Burmese reader expects.
---

# Natural Zaka

Zaka accepts more spellings than a Burmese speaker would write, so a verb can end with
`သည်`, `ပါ`, `ပါသည်` or no ending at all, and the program runs the same in each case. A
Burmese speaker, though, picks the ending from the word that comes next, and some choices
sound wrong. The *naturalness check* (a lint) points these out as warnings, which never
stop the program and suggest a spelling that sounds more natural.

## You will learn

- What a naturalness warning looks like and how to follow its `help` line.
- Which ending goes before "or", "and" and an attached `လျှင်`.
- When to print with `ဟု` and when with `ကို`.
- How to turn the warnings into errors.

## A first warning

In this program, `ကြီးသည်လျှင်` ("if it is greater") has the ending `သည်` before
`လျှင်`, which a Burmese speaker would not write:

```zaka-warning ZK0502
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
အကယ်၍ အသက် သည် ၁၈ ထက် ကြီးသည်လျှင် "အရွယ်ရောက်" ဟု ပြသမည်။
```

```text
warning[ZK0502]: "ကြီးသည်" before "လျှင်" reads better without the particle
help: write "ကြီးလျှင်", or keep the particle and write "ဖြစ်လျှင်"
```

The warning comes with a code and a `help` line. The `help` line suggests two fixes: we
can drop the particle and write `ကြီးလျှင်`, or keep it and add `ဖြစ်` before `လျှင်`:

```
<verb>လျှင်
<verb>သည် ဖြစ်လျှင်
```

| Zaka | Reads as |
| --- | --- |
| `ကြီးလျှင်` | "if greater" |
| `ကြီးသည် ဖြစ်လျှင်` | "if it is the case that it is greater" |

When we follow the `help` line, the warning disappears:

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
အကယ်၍ အသက် သည် ၁၈ ထက် ကြီးလျှင် "အရွယ်ရောက်" ဟု ပြသမည်။
အကယ်၍ အသက် သည် ၁၈ ထက် ကြီးသည် ဖြစ်လျှင် "အရွယ်ရောက်" ဟု ပြသမည်။
```

```text
အရွယ်ရောက်
အရွယ်ရောက်
```

## The rules

Each rule has its own code. The "Write" column is the natural spelling, and the "Not"
column is still valid Zaka that runs the same, but sounds less natural:

| Code | When | Write | Not |
| --- | --- | --- | --- |
| ZK0501 | a "not" verb before `သို့မဟုတ်` (or) | `မငယ်ပါ သို့မဟုတ်` | `မငယ်သည် သို့မဟုတ်` |
| ZK0502 | before an attached `လျှင်` or `နေသရွေ့` | `ကြီးလျှင်`, `ကြီးနေသရွေ့` | `ကြီးသည်လျှင်` |
| ZK0503 | on `မဟုတ်` (not) | `မဟုတ်ပါ` or `မဟုတ်` | `မဟုတ်သည်` |
| ZK0504 | before `ဖြစ်၍` (and) | `ကြီးသည် ဖြစ်၍` | `ကြီးပါ ဖြစ်၍` |
| ZK0505 | printing text or a whole sentence | `"..." ဟု ပြသမည်` | `"..." ကို ပြသမည်` |
| ZK0506 | printing a plain value | `အသက် ကို ပြသမည်` | `အသက် ဟု ပြသမည်` |
| ZK0507 | parentheses that change nothing | `က + ၁ ကို ပြသမည်` | `(က + ၁) ကို ပြသမည်` |
| ZK0508 | a sentence used as a value (stored, returned, passed) | `... ကြီးသည် ဖြစ်ပါစေ` | `... ကြီးပါ ဖြစ်ပါစေ` |
| ZK0509 | the measure word (classifier) after a data kind | `လူ တစ်ယောက်` | `လူ တစ်ခု`, for a struct counted with `ယောက်` |

## Before "or"

```
<not verb>ပါ သို့မဟုတ် <sentence>
```

The "not" verbs are `မညီ`, `မကြီး`, `မငယ်`, `မမှန်` and `မဟုတ်`, and before `သို့မဟုတ်`
("or") we end them with `ပါ`:

```zaka-warning ZK0501
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
(အသက် သည် ၁၈ အောက် မငယ်သည် သို့မဟုတ် အသက် သည် ၆၀ ထက် ကြီးသည်) ဟု ပြသမည်။
```

```text
warning[ZK0501]: "မငယ်သည်" before "သို့မဟုတ်" reads better as "မငယ်ပါ"
```

## Before "and", and as a value

```
<verb>သည် ဖြစ်၍ <sentence>
... တန်ဖိုးသည် <sentence ending in သည်> ဖြစ်ပါစေ။
```

When we join two sentences with `ဖြစ်၍` ("and"), or use a sentence as a value, we end the
verb with `သည်`, not `ပါ`. This program gets both wrong, so it gets two warnings:

```zaka-warning ZK0504 ZK0508
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
(အသက် သည် ၁၈ ထက် ကြီးပါ ဖြစ်၍ အသက် သည် ၆၀ အောက် ငယ်သည်) ဟု ပြသမည်။
ကိန်းရှင် ကြီးသူ သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အသက် သည် ၁၈ ထက် ကြီးပါ ဖြစ်ပါစေ။
```

```text
warning[ZK0504]: "ကြီးပါ" before "ဖြစ်၍" reads better as "ကြီးသည်"
warning[ZK0508]: "ကြီးပါ" as a value reads better as "ကြီးသည်"
```

Here are the programs of this section and the one before it, in their natural form:

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
(အသက် သည် ၁၈ အောက် မငယ်ပါ သို့မဟုတ် အသက် သည် ၆၀ ထက် ကြီးသည်) ဟု ပြသမည်။
(အသက် သည် ၁၈ ထက် ကြီးသည် ဖြစ်၍ အသက် သည် ၆၀ အောက် ငယ်သည်) ဟု ပြသမည်။
ကိန်းရှင် ကြီးသူ သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အသက် သည် ၁၈ ထက် ကြီးသည် ဖြစ်ပါစေ။
ကြီးသူ ကို ပြသမည်။
```

```text
မှန်
မှန်
မှန်
```

## Strict mode

To treat these warnings as errors, so that a program with one of them does not run, we
pass `--lint-error naturalness` to the compiler:

```sh
zakac run --repl --lint-error naturalness program.zaka
```

This helps once the rules feel familiar and we want each new program to sound natural.

## Details

- **One warning for each place.** When two rules fit the same word, the compiler reports
  the first rule in the table alone, so `မဟုတ်သည်` before `သို့မဟုတ်` gets ZK0501 and not
  ZK0503 ([Natural wording](../../manual/language/naturalness.md#what-the-check-looks-at)).
- **Spacing does not matter.** The check looks at the words of the program, not at the
  spaces between them.
- **One rule at a time.** `--lint-allow`, `--lint-warn` and `--lint-error` also accept a
  single code, and a code wins over its group: `--lint-error naturalness --lint-warn
  ZK0505` keeps ZK0505 a warning ([Strict
  mode](../../manual/language/naturalness.md#strict-mode)).
- **Spoken Burmese.** Spoken forms of the words, and a mix of spoken and written forms in
  one file, have their own codes, ZK0510 and ZK0511 ([Spoken
  Burmese](../../manual/language/spoken.md)).

## Recap

- The naturalness check gives warnings, not errors, and the program still runs.
- Before `သို့မဟုတ်` ("or"), a "not" verb ends with `ပါ`.
- Before `ဖြစ်၍` ("and") and in a sentence used as a value, a verb ends with `သည်`.
- Before an attached `လျှင်`, a verb has no ending, or we write `သည် ဖြစ်လျှင်`.
- `--lint-error naturalness` turns the warnings into errors.

## Hands on

- Take a program from an earlier chapter and change the endings until it warns, then
  follow the `help` lines back.
- Print a number with `ဟု` and text with `ကို`, and read both warnings.
- Run one of the warning programs on this page with `--lint-error naturalness`, and see
  how the output changes.

Next: [Example programs](../programs/README.md).
