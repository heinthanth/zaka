---
title: Natural Zaka
description: Choose the sentence endings and particles a Burmese reader expects.
---

# Natural Zaka

Zaka accepts each of the endings `သည်`, `ပါ`, `ပါသည်`, or none at all, on a verb. The
program runs in all these cases. But a Burmese speaker picks the ending from what comes
next, and some choices sound wrong. Zaka's *naturalness check* points these out as
**warnings**. A warning never stops the program. It suggests a more natural spelling.

```zaka-warning ZK0502
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
အကယ်၍ အသက် သည် ၁၈ ထက် ကြီးသည်လျှင် "အရွယ်ရောက်" ဟု ပြသပါ။
```

```text
warning[ZK0502]: "ကြီးသည်" before "လျှင်" reads better without the particle
help: write "ကြီးလျှင်", or keep the particle and write "ဖြစ်လျှင်"
```

Follow the `help` line and the warning disappears:

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
အကယ်၍ အသက် သည် ၁၈ ထက် ကြီးလျှင် "အရွယ်ရောက်" ဟု ပြသပါ။
အကယ်၍ အသက် သည် ၁၈ ထက် ကြီးသည် ဖြစ်လျှင် "အရွယ်ရောက်" ဟု ပြသပါ။
```

```text
အရွယ်ရောက်
အရွယ်ရောက်
```

## The rules

| Code | When | Write | Not |
| --- | --- | --- | --- |
| ZK0501 | a "not" verb before `သို့မဟုတ်` (or) | `မငယ်ပါ သို့မဟုတ်` | `မငယ်သည် သို့မဟုတ်` |
| ZK0502 | before attached `လျှင်` or `နေသရွေ့` | `ကြီးလျှင်`, `ကြီးနေသရွေ့` | `ကြီးသည်လျှင်` |
| ZK0503 | on `မဟုတ်` (not) | `မဟုတ်ပါ` or `မဟုတ်` | `မဟုတ်သည်` |
| ZK0504 | before `ဖြစ်၍` (and) | `ကြီးသည် ဖြစ်၍` | `ကြီးပါ ဖြစ်၍` |
| ZK0505 | printing text or a whole sentence | `"..." ဟု ပြသပါ` | `"..." ကို ပြသပါ` |
| ZK0506 | printing a plain value | `အသက် ကို ပြသပါ` | `အသက် ဟု ပြသပါ` |
| ZK0507 | parentheses that change nothing | `က + ၁ ကို ပြသပါ` | `(က + ၁) ကို ပြသပါ` |
| ZK0508 | a sentence used as a value (stored, returned, passed) | `... ကြီးသည် ဖြစ်ပါစေ` | `... ကြီးပါ ဖြစ်ပါစေ` |

The "Not" column is still valid Zaka. It just sounds less natural.

## More examples

"Not" verbs (`မညီ`, `မကြီး`, `မငယ်`, `မမှန်`, `မဟုတ်`) take `ပါ` before "or":

```zaka-warning ZK0501
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
(အသက် သည် ၁၈ အောက် မငယ်သည် သို့မဟုတ် အသက် သည် ၆၀ ထက် ကြီးသည်) ဟု ပြသပါ။
```

```text
warning[ZK0501]: "မငယ်သည်" before "သို့မဟုတ်" reads better as "မငယ်ပါ"
```

A sentence joined with "and", or one used as a value, takes `သည်`:

```zaka-warning ZK0504 ZK0508
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
(အသက် သည် ၁၈ ထက် ကြီးပါ ဖြစ်၍ အသက် သည် ၆၀ အောက် ငယ်သည်) ဟု ပြသပါ။
ကိန်းရှင် ကြီးသူ သည် မှန်မှားတန်ဖိုး ဖြစ်၍ တန်ဖိုးသည် အသက် သည် ၁၈ ထက် ကြီးပါ ဖြစ်ပါစေ။
```

```text
warning[ZK0504]: "ကြီးပါ" before "ဖြစ်၍" reads better as "ကြီးသည်"
warning[ZK0508]: "ကြီးပါ" as a value reads better as "ကြီးသည်"
```

The same program, in its natural form:

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
(အသက် သည် ၁၈ အောက် မငယ်ပါ သို့မဟုတ် အသက် သည် ၆၀ ထက် ကြီးသည်) ဟု ပြသပါ။
(အသက် သည် ၁၈ ထက် ကြီးသည် ဖြစ်၍ အသက် သည် ၆၀ အောက် ငယ်သည်) ဟု ပြသပါ။
ကိန်းရှင် ကြီးသူ သည် မှန်မှားတန်ဖိုး ဖြစ်၍ တန်ဖိုးသည် အသက် သည် ၁၈ ထက် ကြီးသည် ဖြစ်ပါစေ။
ကြီးသူ ကို ပြသပါ။
```

```text
မှန်
မှန်
မှန်
```

## Strict mode

To treat these warnings as errors, so that a program with one or more of them does not
run, pass `--strict-particles`:

```sh
zakac run --strict-particles program.zaka
```

This is useful once the rules feel familiar and each new program should sound natural.

## Hands on

- Take a program from an earlier chapter and change the endings until it warns. Then
  follow the `help` lines back.

Next: [Example programs](../programs/README.md).
