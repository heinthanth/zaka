---
title: Blocks and scope
description: Group statements into a block; names declared inside stay inside.
---

# Blocks and scope

A *block* groups several statements together. It opens with "do the following" and
closes with "the tasks are done":

```zaka
"အစ" ဟု ပြသပါ။
အောက်ပါတို့ကို လုပ်ဆောင်ပါ။               // do the following:
ကိန်းရှင် လူဦးရေ သည် ကိန်းပြည့် ၃ ဖြစ်ပါစေ။
လူဦးရေ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။              // the tasks are done.
"အဆုံး" ဟု ပြသပါ။
```

```text
အစ
၃
အဆုံး
```

| Zaka | Reads as | Job |
| --- | --- | --- |
| `အောက်ပါတို့ကို လုပ်ဆောင်ပါ။` | "please do the following" | opens a block |
| `လုပ်ဆောင်စရာများ ပြီးပါပြီး။` | "the tasks are done" | closes the block |

On its own, a block is not very useful. Its real job is to hold the many statements of
an [if](../control-flow/if-else.md), a [loop](../control-flow/while.md) or a
[function](../functions/README.md). But it has one effect worth learning first: **scope**.

## Scope

The *scope* of a name is the part of the program where the name can be used. A name
declared inside a block can be used only inside that block. After the closer, it is
gone:

```zaka-error ZK0301
အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် ယာယီ သည် ကိန်းပြည့် ၅ ဖြစ်ပါစေ။     // temporary
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
ယာယီ ကို ပြသပါ။
```

```text
error[ZK0301]: "ယာယီ" is not declared
```

Names from outside **are** visible inside, and blocks can sit inside other blocks:

```zaka
ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် ၁ ဖြစ်ပါစေ။
အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် ယာယီ သည် ကိန်းပြည့် အရေအတွက် * ၁၀ ဖြစ်ပါစေ။   // uses the outer name
အောက်ပါတို့ကို လုပ်ဆောင်ပါ။                            // a block inside a block
ယာယီ ကို ယာယီ + ၁ ဟု ထားပါ။
ယာယီ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
```

```text
၁၁
```

## Shadowing

An inner block may declare a name that already exists outside. The inner one *shadows*
the outer one: inside the block, the name means the new variable. When the block ends,
the outer variable is back, unchanged.

```zaka
ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် ၁ ဖြစ်ပါစေ။
အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် ၁၀၀ ဖြစ်ပါစေ။   // a new, inner အရေအတွက်
အရေအတွက် ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
အရေအတွက် ကို ပြသပါ။                          // the outer one again
```

```text
၁၀၀
၁
```

## Four ways to close

The closer has four spellings that mean the same thing. Use whichever reads best to you:

- `လုပ်ဆောင်စရာများ ပြီးပါပြီ။`
- `လုပ်ဆောင်စရာများ ပြီးပါပြီး။`
- `လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီ။`
- `လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။`

This manual uses `ပြီးပါပြီး`.

## Try it

- Declare a variable inside a block and try to print it after the block. Read the error.
- Nest three blocks, each shadowing the same name with a different value, and print it
  in each.

Next: [Operators](../operators/README.md).
