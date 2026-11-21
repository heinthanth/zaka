---
title: Names and scope
description: How names are declared and found - blocks, shadowing, constants, loop counters, functions - and the errors ZK0301 to ZK0308.
---

# Names and scope

A name refers to a variable, a constant, a parameter or a loop counter, and the scope of
a name is the part of the program where we can use it. This page states what opens a
scope, which names a place can see, what shadowing is, and which names cannot be assigned
to. [Blocks and scope](../../learn/variables/scope.md) in the Learn book shows the same
ideas with gentle examples, and [Source text and words](words.md#names) states how we
write a name.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က + ၁၀ ဖြစ်ပါစေ။
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဂ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ခ + ၁၀ ဖြစ်ပါစေ။
က + ခ + ဂ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ခ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
က ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၃၃
၁၁
၁
```

## Declaring a name

A name exists from the end of its declaration to the end of the scope that contains it.
A declaration checks its value before it adds the name, so the value cannot use the name
that is being declared:

```zaka-error ZK0301
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က + ၁ ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0301]: "က" is not declared
```

A name used before its declaration, or after the end of the scope that contained it, is
ZK0301 as well. In one scope, we can declare a name once (ZK0302), and a second
declaration clashes even when it has a different data kind (type).

```zaka-error ZK0302
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် က သည် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂.၅ ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0302]: "က" is declared before in this scope
help: pick another name, or declare it in an inner block to shadow the outer one
```

## What opens a scope

The top level of the program is the outermost scope. A new scope, nested inside the one
around it, opens at each of these places:

| Place | The scope contains |
| --- | --- |
| a block, `အောက်ပါတို့ကို လုပ်ဆောင်မည်။ … လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။` | the statements of the block |
| each branch of an if, and the block of a while loop | the statements of the block, or the single statement |
| a counted loop | the counter and the statements of the loop's block |
| a generic loop | the loop variable, the statements of the block and the update |
| a function | the parameters and the statements of the block |
| the [program start block](functions-start-block.md) | the same as a function |

A name declared in a scope is visible in that scope after its declaration and in each
scope nested inside it, and nowhere else. A lookup starts in the innermost scope and moves
outward, and the first match wins. In the example at the top of the page:

```text
top level               declares က              sees က
  block A               declares ခ              sees က, ခ
    block B             declares ဂ              sees က, ခ, ဂ
  back in block A                               sees က, ခ
back at the top level                           sees က
```

A branch or a loop whose block is a single statement is a scope too, so a declaration
there ends with that statement. A name declared inside a block cannot be used after the
block closer:

```zaka-error ZK0301
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ယာယီ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ယာယီ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## Shadowing

An inner scope can declare a name that an outer scope has. Inside the inner scope, the
name means the new variable, while the outer one is shadowed: hidden but not changed, and
it comes back when the inner scope ends. The value of the inner declaration is checked
before the new name exists, so it can still use the outer one, and the inner variable can
have a different data kind.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အရေအတွက် + ၁၀၀ ဖြစ်ပါစေ။
အရေအတွက် ကို ပြသမည်။
အရေအတွက် ကို အရေအတွက် + ၁ ဟု ထားမည်။
အရေအတွက် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အရေအတွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၀၁
၁၀၂
၁
```

Names that share a scope do not shadow each other, so the two clash. In some places the
names seem to be in two scopes but share one:

| Names | Shared scope | Error |
| --- | --- | --- |
| a counted loop's counter and the names in its block | one scope, so the block cannot declare the counter's name again, though an inner block can | ZK0302 |
| a function's parameters and the names in its block | one scope, so a local with a parameter's name clashes | ZK0302 |
| two parameters of one function with one name | | ZK0305 |
| two functions with one name | | ZK0304 |

```zaka-error ZK0302
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် န သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် န သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် န * ၁၀ ဖြစ်ပါစေ။
န ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
န ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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

Besides its data kind, each name has an access, which decides whether we can assign to it.
We can read each name:

| Name | Assigning to it | Error |
| --- | --- | --- |
| a variable, a parameter, a generic-loop variable | allowed | |
| a constant (`ကိန်းသေ`) | not allowed | ZK0306 |
| the counter of a counted loop | not allowed | ZK0307 |

A constant must get its value where it is declared (ZK0308). A constant and a counter
cannot go to a mutable parameter (`inout`), one marked `ဒေတာ ပြောင်းလဲရန်`, because it
writes the argument back (ZK0306, ZK0307; [Mutable parameters](functions-parameters.md#mutable-parameters)), and in unsafe code
we cannot take their address (ZK0412), because a pointer could write through it.

```zaka-error ZK0306
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ တစ်ပတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇ ဖြစ်ပါစေ။
တစ်ပတ် ကို ၈ ဟု ထားမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0306]: "တစ်ပတ်" is a constant and cannot be assigned
```

We can shadow a constant like a variable, and the shadow can be a variable.

## Names in functions

A function's block is a scope nested in the top level, so it sees the top-level names
declared above the function in the file and shares them with the rest of the program:
assigning to such a variable changes it for the whole program.

```zaka
ကိန်းရှင် စုစုပေါင်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
ပေါင်းထည့် ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စုစုပေါင်း ကို စုစုပေါင်း + က ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပေါင်းထည့် ဟူသော လုပ်ဆောင်ချက်ကို ၅ ဖြင့် လုပ်ဆောင်မည်။
ပေါင်းထည့် ဟူသော လုပ်ဆောင်ချက်ကို ၇ ဖြင့် လုပ်ဆောင်မည်။
စုစုပေါင်း ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၂
```

- A name declared below the function is not visible in it (ZK0301).
- Functions do not follow this order: a call can come before the declaration of the
  function, and a function's name is visible in all places.
- Variables and functions have separate sets of names, so a variable and a function can
  share a name without a clash. A call of a name that no function has is ZK0303.
- A top-level function never sees the locals of another function. A function nested in
  another one is the exception: it reads and writes the locals around it, with no list of
  them, and shares each one with the block around it, so a write on either side is seen
  on the other ([Function values](function-values.md)).

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
results, and [Unsafe Zaka](unsafe.md) for names whose address is taken.
