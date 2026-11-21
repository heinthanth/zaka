---
title: Spoken Burmese
description: The spoken spellings of the reserved words, such as ထားမယ် for ထားမည်, how a program turns them on, and the lint that finds a file that mixes the two.
---

# Spoken Burmese

Zaka uses written Burmese: `ထားမည်`, `ဖြစ်သည်`, `ထဲတွင်`. Each of these words also has a
spoken spelling, the one that people use in speech: `ထားမယ်`, `ဖြစ်တယ်`, `ထဲမှာ`. The
compiler accepts both, and a spoken spelling means the same as its written word in each
place, so a program in spoken Burmese compiles to the same code. The Learn book shows the
spoken forms in [Natural Zaka](../../learn/natural-zaka/README.md).

The written spellings are the default. A spoken spelling is the error ZK0510 until the
program allows the spoken spellings, and the message comes with a fix that writes the word
in its written spelling:

```zaka-error ZK0510
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
    "မင်္ဂလာပါ" ကို ပြသမယ်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## Turning them on

ZK0510 is a lint code, and the one code of the lint group `spoken`. To write a program in
spoken Burmese, we allow the group:

```console
$ zakac run --lint-allow spoken program.zaka
```

or, for each command of a project, in its `zakac.toml`:

```toml
[lint]
allow = ["spoken"]
```

With the spoken spellings allowed, a program can use the words of speech:

```zaka lint-allow=spoken
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မယ်။
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၃ ဖြစ်ပါစေ။
အကယ်၍ အမှတ် သည် ၄၀ အောက် မငယ်ရင် "အောင်တယ်" လို့ ပြသမယ်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အောင်တယ်
```

The editor reads the same `zakac.toml`, so it shows no ZK0510 in that project. The
[compiler explorer](/explorer/) keeps the default, so there each spoken spelling is an
error.

## The spoken parts

Each spoken spelling comes from a written one, one *vanna* at a time (a vanna is a
consonant with its marks, the unit of Burmese writing):

| Written | Spoken | Meaning |
| --- | --- | --- |
| `မည်` | `မယ်` | "will", the end of a statement |
| `မည့်` | `မယ့်` | "that will", before a name |
| `သည်` | `တယ်` | the end of a sentence, and the particle on a verb |
| `သည့်`, `သော` | `တဲ့` | "that", before a noun |
| `တွင်` | `မှာ` | "at", "in" |
| `လျှင်` | `ရင်` | "if" |
| `ဟု` | `လို့` | the quote particle |
| `မှ` | `ကနေ` | "from" |
| `နှင့်` | `နဲ့` | "and", "with" |
| `ဤ` | `ဒီ` | "this" |
| `၏` | `ရဲ့` | "of" |

A reserved word that contains one of these parts as a whole vanna has a spoken spelling
too: `ထားမယ်`, `ပြန်ပေးမယ်`, `ဖြစ်တယ်`, `ထဲမှာ`, `ဖြစ်ရင်`, `ဒီနေရာမှာ`,
`မှတ်ယူနိုင်ပါတယ်`. The match is a whole vanna, so the `မှ` inside `မှတ်` remains as it is.
The [reserved words](../reference/reserved-words.md#spoken-spellings) page lists each
spelling.

One word has no spoken spelling: `မည်သည်ကိုမှ` ("nothing"), whose `မည်` and `သည်` ask
"which" and are not endings.

Where two written words give one spoken spelling, the spoken word can mean both, and its
place in the sentence tells which one:

| Spoken | Written | Where |
| --- | --- | --- |
| `တဲ့` | `သော`, `သည့်` | before a noun, where the two mean the same |
| `ဟူတဲ့` | `ဟူသော`, `ဟူသည့်` | `ဟူသည့်` after the name of a trait, `ဟူသော` elsewhere |
| `ရှိတဲ့` | `ရှိသော`, `ရှိသည့်` | `ရှိသည့်` after the value of an enumeration member, `ရှိသော` elsewhere |

The message ZK0510 for such a word names both written words, and its fix writes the one
that fits the place.

A spoken word becomes a reserved word, as its written word is, so `မှာ`, `ရင်` or `နဲ့`
cannot be the name of a variable.

## "နှင့်" in place of "ဖြင့်"

Spoken Burmese uses `နဲ့` for "with" in both of its senses: "together with" (`နှင့်`) and
"through" (`ဖြင့်`). So the parser accepts `နှင့်` and `နဲ့` where it expects `ဖြင့်` too:

- in the places whose grammar asks for `ဖြင့်`: the base of a literal, and the argument of
  a call in notation, right before the word that ends the call. A fill keeps `ဖြင့်`,
  because there `နှင့်` can start the clause of the value;
- in a call, for a preposition `ဖြင့်` of the function, when the function itself has no
  `နှင့်` preposition and the `နှင့်` does not join two items of a list.

When the function has both prepositions, `နှင့်` remains `နှင့်`, and the argument for
`ဖြင့်` needs `ဖြင့်`.

## Mixing the two

A file that allows the spoken spellings can still write some words spoken and others
written. The program means the same, but it is easier to read in one form. The compiler
warns once for such a file, with ZK0511, at the first word of the form that the file uses
less, with a fix that writes it in the other form. We can turn the warning off with
`--lint-allow ZK0511`. The warning does not come while the spoken spellings are not
allowed, because each spoken word then gets its own error.

## Levels

| Name | Codes | Default |
| --- | --- | --- |
| `spoken` | ZK0510 | an error |
| `ZK0511` (in `naturalness`) | ZK0511 | a warning, once the spoken spellings are allowed |
| `naturalness` | ZK0501 to ZK0511 | as each code |

`--lint-allow`, `--lint-warn` and `--lint-error` accept a group or a code, and a code wins
over its group ([Diagnostics](../compiler/diagnostics.md#levels)).
