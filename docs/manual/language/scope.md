---
title: Names and scope
description: How names are declared and found - blocks, shadowing, constants, loop counters, functions - and the errors ZK0301 to ZK0308.
---

# Names and scope

A *name* stands for a variable, a constant, a parameter or a loop counter. The *scope*
of a name is the part of the program where the name can be used. This page states what
opens a scope, which names a place can see, what *shadowing* is and which names are
fixed. [Blocks and scope](../../learn/variables/scope.md) in the Learn book shows the
same ideas with gentler examples; [Source text and words](words.md) tells how a name
is written.

## Declaring a name

A name exists from the end of its declaration to the end of the scope that holds it. A
declaration checks its value **before** it adds the name, so the value cannot use the
name that is being declared:

```zaka-error ZK0301
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် က + ၁ ဖြစ်ပါစေ။
```

```text
error[ZK0301]: "က" is not declared
```

The same holds for a name that is used before its declaration, or after the scope that
held it has ended: both are ZK0301. In one scope, a name is declared once (ZK0302). The
second declaration clashes even when it has another kind.

```zaka-error ZK0302
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် က သည် ဒဿမကိန်း ဖြစ်၍ တန်ဖိုးသည် ၂.၅ ဖြစ်ပါစေ။
```

```text
error[ZK0302]: "က" is declared before in this scope
help: pick another name, or declare it in an inner block to shadow the outer one
```

## What opens a scope

The *top level* of the program is the outermost scope. A new scope, nested inside the
one around it, opens at each of these places:

| Place | The scope holds |
| --- | --- |
| a block, `အောက်ပါတို့ကို လုပ်ဆောင်ပါ။ ... လုပ်ဆောင်စရာများ ပြီးပါပြီး။` | the statements of the block |
| each branch of an `if`, and the block of a `while` | the statements of the block, or the one statement |
| a counted loop | the counter and the statements of the loop's block |
| a generic loop | the loop variable, the statements of the block and the update |
| a function | the parameters, `ရလဒ်` (when there is a result) and the statements of the block |

A name declared in a scope is visible in that scope after its declaration and in each
scope nested inside it, and in no other place. A lookup starts in the innermost scope
and moves outward, and the first match wins.

```text
top level               declares က              sees က
  block A               declares ခ              sees က, ခ
    block B             declares ဂ              sees က, ခ, ဂ
  back in block A                               sees က, ခ
back at the top level                           sees က
```

A branch or a loop with a single statement for its block is a scope too, so a declaration
there ends with the statement.

```zaka
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် က + ၁၀ ဖြစ်ပါစေ။
အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် ဂ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ခ + ၁၀ ဖြစ်ပါစေ။
က + ခ + ဂ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
ခ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
က ကို ပြသပါ။
```

```text
၃၃
၁၁
၁
```

```zaka-error ZK0301
အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် ယာယီ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
ယာယီ ကို ပြသပါ။
```

## Shadowing

An inner scope can declare a name that an outer scope has. Inside the inner scope, the
name means the new variable; the outer one is *shadowed*, hidden but not changed, and it
returns when the inner scope ends. The value of the inner declaration is checked before
the new name exists, so it can still use the outer one. The inner variable can have
another kind.

```zaka
ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် အရေအတွက် + ၁၀၀ ဖြစ်ပါစေ။
အရေအတွက် ကို ပြသပါ။
အရေအတွက် ကို အရေအတွက် + ၁ ဟု ထားပါ။
အရေအတွက် ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
အရေအတွက် ကို ပြသပါ။
```

```text
၁၀၁
၁၀၂
၁
```

Names that share a scope do not shadow each other; the two clash. Some places look as if
the pair held two scopes, but hold one:

- A counted loop's counter and the statements of its block share a scope, so the block
  cannot declare the counter's name again. An inner block can.
- A function's parameters and the statements of its block share a scope: a local named as
  a parameter is ZK0302.
- Two parameters of one name are ZK0305, and two functions of one name are ZK0304.

```zaka-error ZK0302
၁ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် န သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
```

```zaka
၁ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် န သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် န * ၁၀ ဖြစ်ပါစေ။
န ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
န ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
```

```text
၁၀
၁
၂၀
၂
၃၀
၃
```

## Constants and counters

A name has an *access* besides its kind. A variable, a parameter and a generic-loop
variable can be read and assigned to. A constant (`ကိန်းသေ`) can be read alone: an
assignment is ZK0306, and a constant must be given its value where it is declared
(ZK0308). The counter of a counted loop can be read alone as well (ZK0307). Neither can
have its address taken (ZK0412), since a pointer could write through it.

```zaka-error ZK0306
ကိန်းသေ တစ်ပတ် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၇ ဖြစ်ပါစေ။
တစ်ပတ် ကို ၈ ဟု ထားပါ။
```

```text
error[ZK0306]: "တစ်ပတ်" is a constant and cannot be assigned
```

A constant can be shadowed like a variable, and the shadow can be a variable.

## Names in functions

A function's block is a scope nested in the top level, so it sees the top-level names
declared **above** the function in the file, and shares them with the rest of the
program: assigning to such a variable changes it for the whole program.

```zaka
ကိန်းရှင် စုစုပေါင်း သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ပေါင်းထည့် သည် ကိန်းပြည့် က ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စုစုပေါင်း ကို စုစုပေါင်း + က ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်ချက် ပေါင်းထည့် ကို ၅ တို့ဖြင့် လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် ပေါင်းထည့် ကို ၇ တို့ဖြင့် လုပ်ဆောင်ပါ။
စုစုပေါင်း ကို ပြသပါ။
```

```text
၁၂
```

A name declared below the function is not visible in it (ZK0301). The functions
themselves are not subject to this order: a call can come before the declaration, and a
function's own name is visible in all places. Variables and functions have separate sets of
names, so a variable and a function can share a name without a clash. A call of a name
that no function has is ZK0303. A function is declared at the top level and nowhere else
(ZK0214), so a function never sees the locals of another function.

## The errors

| Code | Raised when |
| --- | --- |
| ZK0301 | a name is used where it is not declared, or after its scope ended |
| ZK0302 | a scope declares a name twice |
| ZK0303 | a call names a function that does not exist |
| ZK0304 | two functions have one name |
| ZK0305 | two parameters have one name |
| ZK0306 | a constant is assigned to |
| ZK0307 | a counted-loop counter is assigned to |
| ZK0308 | a constant is declared with no value |

The next pages build on these rules: [Functions](functions.md) for parameters and
results, and [Pointers](pointers.md) for names whose address is taken.
