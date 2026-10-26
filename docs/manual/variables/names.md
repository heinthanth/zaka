---
title: Names
description: What a variable name may contain, and names in backticks.
---

# Names

You choose the name of every variable and constant. A good name says what the value
means: `အသက်` (age) is better than `က`.

A name may use Burmese letters, other scripts, digits (not first), `_`, and even emoji.
Upper and lower case are different: `Age` and `age` are two names.

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ၁၈ ဖြစ်ပါစေ။          // Burmese
ကိန်းရှင် age သည် ကိန်းပြည့် ၂၀ ဖြစ်ပါစေ။            // English
ကိန်းရှင် အမှတ်_၁ သည် ကိန်းပြည့် ၇၅ ဖြစ်ပါစေ။        // with _ and a digit
ကိန်းရှင် 🍎 သည် ကိန်းပြည့် ၃ ဖြစ်ပါစေ။               // emoji
အသက် + age + အမှတ်_၁ + 🍎 ကို ပြသပါ။
```

```text
၁၁၆
```

## Where a name ends

A name ends at a space, at a symbol such as `+` or `။`, or where the script changes
(Burmese to English, or back). That is why a name needs a space before a particle:
`အသက် ကို`, not `အသက်ကို` (see [Spaces between words](../sentences/spacing.md)).

## Names that look like Zaka words

A name cannot be made only of Zaka's reserved words. Short words such as `ကြီး`
(greater), `ငယ်` (less) and `ညီ` (equal) are reserved verbs, so `ကြီး` alone cannot be a
name; `ကြီးသူ` can. `တန်ဖိုးကို` reads as the two reserved words `တန်ဖိုး` + `ကို`. The
[Reserved words](../reference/reserved-words.md) page lists them all. To use such a name, or a name with spaces in it, wrap it
in backticks:

```zaka
ကိန်းရှင် `ကို ပြသပါ` သည် ကိန်းပြည့် ၇ ဖြစ်ပါစေ။     // a name made of Zaka words
ကိန်းရှင် `my score` သည် ကိန်းပြည့် ၉၀ ဖြစ်ပါစေ။      // a name with a space
`ကို ပြသပါ` ကို ပြသပါ။
`my score` ကို ပြသပါ။
```

```text
၇
၉၀
```

Backtick names work, but they are hard to read. Use them only when you must.

## Each name once

In one place, a name can be declared only once:

```zaka-error ZK0302
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ၁၈ ဖြစ်ပါစေ။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ၂၀ ဖြစ်ပါစေ။
```

```text
error[ZK0302]: "အသက်" is declared before in this scope
```

The help line suggests declaring the name again "in an inner block". The next page,
[Blocks and scope](scope.md), explains what that means.

## Try it

- Declare a variable named with your favourite emoji and print it.
- Try `ကိန်းရှင် ၂အသက် သည် ကိန်းပြည့် ၁ ဖြစ်ပါစေ။`. Why does a name not start with a digit?

Next: [Blocks and scope](scope.md).
