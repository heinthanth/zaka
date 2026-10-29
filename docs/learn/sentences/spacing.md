---
title: Spaces between words
description: Where Zaka needs a space, and where it does not.
---

# Spaces between words

Written Burmese has no spaces between words, and readers know where each word ends. The
compiler has no word list, so it cannot know. In Zaka, **a space separates two words**.

The rule that matters most: put a space between a **made-up name** and the particle
after it.

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။   // make a number named အသက် (age)
အသက် ကို ပြသပါ။                          // space between အသက် and ကို
```

```text
၂၀
```

Without the space, `အသက်ကို` is read as one long name. No such name exists, so the
compiler complains, and it suggests the fix:

```zaka-error ZK0206
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
အသက်ကို ပြသပါ။
```

```text
error[ZK0206]: "ပြသပါ" needs "ကို|ဟု" after operand 1, found no particle
help: write "အသက် ကို" with a space: a name joined to a particle is read as one word
```

## Where spaces are optional

- **Between Zaka's own words.** The compiler splits a run made of reserved words alone.
  `ထက်ကြီးသည်` is `ထက်` + `ကြီး` + `သည်`.
- **Around signs.** `။`, `၊`, `၏`, `(`, `)`, quotes, and maths signs such as `+` and
  `<` are words on their own in all cases. `၁+၂`, `ပြသပါ။` and `၏ပေါင်းလဒ်` are fine.
- **How much space.** One space, several spaces, a tab, or a line break all count the
  same.

```zaka
၅ သည် ၃ ထက်ကြီးသည် ဟု ပြသပါ။      // reserved words joined
၁+၂ ကို ပြသပါ။                     // no spaces around +
၁ နှင့် ၂ ၏ပေါင်းလဒ် ကို ပြသပါ။      // ၏ needs no space
```

```text
မှန်
၃
၃
```

With the macOS Burmese input source, `Shift` + `Space` enters an invisible "zero width
space". Zaka counts it as a space too.

## Hands on

- Remove the space between `အသက်` and `ကို` in the first example and read the message.
- Write `၃ နှင့် ၄ ၏ ပေါင်းလဒ်` with as few spaces as possible while it still runs.

Next: [Values and kinds](../values/README.md).
