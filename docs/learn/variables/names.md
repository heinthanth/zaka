---
title: Names
description: What a variable name can contain, and names in braces.
---

# Names

We choose the name (identifier) of each variable and constant, and a good name shows
what the value means: `အသက်` (age) is better than `က`. A name can use Burmese letters,
letters of other scripts, digits after the first character, `_`, and even emoji.

## You will learn

- Which characters a name can use.
- Where a name ends, and the reason a name needs a space before its particle.
- How to use a reserved word, or a space, inside a name, with braces.
- That we can declare a name once in each place.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။      // Burmese
ကိန်းရှင် age သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။        // English
ကိန်းရှင် အမှတ်_၁ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၅ ဖြစ်ပါစေ။    // with _ and a digit
ကိန်းရှင် 🍎 သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။           // emoji
အသက် + age + အမှတ်_၁ + 🍎 ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၁၆
```

Each of the four names works. The table shows which names work and which do not:

| Name | Allowed? | Because |
| --- | --- | --- |
| `အသက်` | allowed | Burmese letters |
| `age`, `Age` | allowed | two different names: upper and lower case differ |
| `အမှတ်_၁` | allowed | a digit and `_` after the first character |
| `🍎` | allowed | an emoji |
| `၂အသက်` | not allowed | a name cannot start with a digit |
| `ကြီး` | not allowed | a reserved word of Zaka ("is greater") |

## Where a name ends

A name ends at a space, or at a sign such as `+` or `။`. A change of script does not end
it, so `အသက်age` and `age၂` are names too, one word each:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက်age သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။  // Burmese, then English
ကိန်းရှင် age၂ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။       // English, then a digit
အသက်age + age၂ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၀
```

This is the reason a name needs a space before its particle. `ageကို` is one long name,
not the name `age` and the particle `ကို`, so the sentence has no particle before its
verb:

```zaka-error ZK0203 repl=on
ကိန်းရှင် age သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။
ageကို ပြသမည်။
```

```text
error[ZK0203]: this statement does not end in a statement verb, found "ပြသ"
  = help: write "age ကို" with a space: a name joined to a particle is read as one word
```

We write `age ကို` and `အသက် ကို`, with a space (see [Spaces between
words](../sentences/spacing.md)).

## Names in braces

```
{<name>}
```

A name cannot be made of Zaka's reserved words alone. Short words such as `ကြီး`
(greater), `ငယ်` (less) and `ညီ` (equal) are reserved verbs, so `ကြီး` alone cannot be a
name, while `ကြီးသူ` can. The [Reserved words](../../manual/reference/reserved-words.md)
page lists them all.

To use a reserved word as a name, or a name with spaces in it, we write the name in
braces:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် {ကို ပြသမည်} သည် ၇ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။     // a name made of Zaka words
ကိန်းရှင် {the total} သည် ၉၀ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။      // a name with a space
{ကို ပြသမည်} ကို ပြသမည်။
{the total} ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၇
၉၀
```

Names in braces work, but these are hard to read, so we save these for when nothing else
will do.

## Each name once

In one place, we can declare a name just once:

```zaka-error ZK0302
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0302]: "အသက်" is declared before in this scope
  = help: pick another name, or declare it in an inner block to shadow the outer one
```

The help line suggests that we declare the name again "in an inner block". The next page,
[Blocks and scope](scope.md), explains what that means.

## Details

- **Two spellings of "data".** `ဒေတာ` and `အချက်အလက်` ("data") are one word inside a
  name, so `ဒေတာအရေအတွက်` and `အချက်အလက်အရေအတွက်` are the same name, and declaring
  both in one place is the same mistake, ZK0302. See [Names](../../manual/language/words.md#names).
- **Names of data kinds.** The names of the built-in data kinds, such as `ကိန်းပြည့်` and
  `စာသား`, are not reserved, so a program can declare a struct with one of these names,
  and the name then means the struct in that file.
- **The rules of braces.** A name in braces cannot contain a `}` or a line break, `{}`
  with nothing inside is ZK0109, and a `{` with no `}` on its line is ZK0108. A plain
  reserved word as a name is ZK0201. See [Names](../../manual/language/words.md#names).

## Recap

- A name can use letters of each script, `_`, emoji, and digits after the first
  character, and upper and lower case differ.
- A name runs to the next space or sign, so we put a space between a name and its
  particle.
- We write a name in braces, such as `{the total}`, to use reserved words or spaces in
  it.
- We can declare a name once in each place.

## Hands on

- Declare a variable with a favourite emoji as its name, and print it.
- Test `ကိန်းရှင် ၂အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။`. What stops a name
  from starting with a digit?
- Declare a variable with the name `ကြီး`, read the message, then put the name in braces.

Next: [Blocks and scope](scope.md).
