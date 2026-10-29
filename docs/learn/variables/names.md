---
title: Names
description: What a variable name can contain, and names in backticks.
---

# Names

The programmer chooses the name of each variable and constant. A good name tells what
the value means: `အသက်` (age) is better than `က`.

A name can use Burmese letters, other scripts, digits (not first), `_`, and even emoji.
Upper and lower case are different: `Age` and `age` are two names.

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။          // Burmese
ကိန်းရှင် age သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။            // English
ကိန်းရှင် အမှတ်_၁ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၇၅ ဖြစ်ပါစေ။        // with _ and a digit
ကိန်းရှင် 🍎 သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။               // emoji
အသက် + age + အမှတ်_၁ + 🍎 ကို ပြသပါ။
```

```text
၁၁၆
```

## Where a name ends

A name ends at a space, or at a sign such as `+` or `။`. A change of script does not end
it: `အသက်age` and `age၂` are names too, one word each.

```zaka
ကိန်းရှင် အသက်age သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။      // Burmese, then English
ကိန်းရှင် age၂ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။           // English, then a digit
အသက်age + age၂ ကို ပြသပါ။
```

```text
၂၀
```

That is the reason a name needs a space before a particle. `ageကို` is one long name, not
the name `age` and the particle `ကို`, so the verb finds no particle:

```zaka-error ZK0206
ကိန်းရှင် age သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။
ageကို ပြသပါ။
```

```text
error[ZK0206]: "ပြသပါ" needs "ကို|ဟု" after operand 1, found no particle
  = help: write "age ကို" with a space: a name joined to a particle is read as one word
```

Write `age ကို` and `အသက် ကို`, with a space (see [Spaces between
words](../sentences/spacing.md)).

## Names that look like Zaka words

A name cannot be made of Zaka's reserved words alone. Short words such as `ကြီး`
(greater), `ငယ်` (less) and `ညီ` (equal) are reserved verbs, so `ကြီး` alone cannot be a
name; `ကြီးသူ` can. `တန်ဖိုးကို` reads as the two reserved words `တန်ဖိုး` + `ကို`. The
[Reserved words](../../manual/reference/reserved-words.md) page lists them all. To use such a
name, or a name with spaces in it, wrap it in backticks:

```zaka
ကိန်းရှင် `ကို ပြသပါ` သည် ကိန်းပြည့် ၇ ဖြစ်ပါစေ။     // a name made of Zaka words
ကိန်းရှင် `the total` သည် ကိန်းပြည့် ၉၀ ဖြစ်ပါစေ။      // a name with a space
`ကို ပြသပါ` ကို ပြသပါ။
`the total` ကို ပြသပါ။
```

```text
၇
၉၀
```

Backtick names work, but are hard to read. Save them for when nothing else will do.

## Each name once

In one place, a name can be declared once and no more:

```zaka-error ZK0302
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
```

```text
error[ZK0302]: "အသက်" is declared before in this scope
```

The help line suggests declaring the name again "in an inner block". The next page,
[Blocks and scope](scope.md), explains what that means.

## Hands on

- Declare a variable named with a favourite emoji and print it.
- Test `ကိန်းရှင် ၂အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။`. What stops a name
  from starting with a digit?

Next: [Blocks and scope](scope.md).
