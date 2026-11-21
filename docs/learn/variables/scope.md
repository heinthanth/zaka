---
title: Blocks and scope
description: Group sentences into a block; names declared inside it can be used inside it alone.
---

# Blocks and scope

A *block* groups several sentences (statements) together, and a name that we declare
inside a block can be used inside that block alone. The part of a program where a name can
be used is the *scope* of the name.

## You will learn

- How to open a block and close it with the block closer.
- What the scope of a name is.
- How a name inside a block can shadow a name outside it.
- The four spellings of the block closer.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"အစ" ဟု ပြသမည်။
အောက်ပါတို့ကို လုပ်ဆောင်မည်။               // do the following:
ကိန်းရှင် လူဦးရေ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
လူဦးရေ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။              // the tasks are done.
"အဆုံး" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အစ
၃
အဆုံး
```

To make a block, we write `အောက်ပါတို့ကို လုပ်ဆောင်မည်။` ("do the following"), then the
sentences, and then the block closer (end of a block) `လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။`
("the tasks are done"):

```
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
<sentences>
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

| Zaka | Reads as | What it does |
| --- | --- | --- |
| `အောက်ပါတို့ကို လုပ်ဆောင်မည်။` | "please do the following" | opens a block |
| `လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။` | "the tasks are done" | closes the block |

A block on its own does little, and its main use is to group the sentences of an
[if](../control-flow/if-else.md), a [loop](../control-flow/while.md) or a
[function](../functions/README.md). It has one effect that is worth learning first,
though, which is scope.

## Scope

A name that we declare inside a block can be used inside that block, and nowhere else.
After the block closer, the name is gone:

```zaka-error ZK0301
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ယာယီ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။ // short-lived
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ယာယီ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0301]: "ယာယီ" is not declared
```

The other direction works: we can use the names from outside a block inside it, and we
can put a block inside another block:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ယာယီ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အရေအတွက် * ၁၀ ဖြစ်ပါစေ။ // uses the outer name
အောက်ပါတို့ကို လုပ်ဆောင်မည်။                            // a block inside a block
ယာယီ ကို ယာယီ + ၁ ဟု ထားမည်။
ယာယီ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၁
```

## Shadowing

A block can declare a name that also exists outside it. The inner name then *shadows*
the outer one, so inside the block the name means the new variable, and when the block
ends, the outer variable is back, with its value unchanged:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀၀ ဖြစ်ပါစေ။ // a new, inner အရေအတွက်
အရေအတွက် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အရေအတွက် ကို ပြသမည်။                          // the outer one again
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၀၀
၁
```

## Four closing forms

The block closer has four spellings, which all mean the same thing, so we can pick the one
we like best:

- `လုပ်ဆောင်စရာများ ပြီးပါပြီ။`
- `လုပ်ဆောင်စရာများ ပြီးပါပြီး။`
- `လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီ။`
- `လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။`

These pages use `ပြီးဆုံးပါပြီး`. The program start block ends with its own block closer,
`ပရိုဂရမ် ပြီးဆုံးပါပြီး။`, and we can spell it in the same four forms.

## Details

- **A block closer with no block.** A block closer with no open block before it is the
  error ZK0226. See [Blocks](../../manual/language/control-flow.md#blocks).
- **Other places that open a scope.** Each branch of an if, each loop and each function
  opens a scope too, even when its block is a single sentence. See [What opens a
  scope](../../manual/language/scope.md#what-opens-a-scope).
- **Shadowing a constant.** A block can shadow a constant with a variable of the same
  name. See [Shadowing](../../manual/language/scope.md#shadowing).

## Recap

- We open a block with `အောက်ပါတို့ကို လုပ်ဆောင်မည်။` and close it with
  `လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။`.
- A name declared inside a block can be used inside it alone, while names from outside
  work inside too.
- A name inside a block can shadow an outer name, and the outer one comes back after the
  block.
- The block closer has four spellings with one meaning.

## Hands on

- Declare a variable inside a block, then print it after the block, and read the
  error.
- Nest three blocks, each shadowing the same name with a different value, and print it
  in each.

Next: [Operators](../operators/README.md).
