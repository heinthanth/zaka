---
title: Function calls
description: The forms of a function call, with the function first or the arguments first, how arguments are checked, calls on a value with ၏, and recursion.
---

# Function calls

A call runs a function with its arguments. Zaka has several forms of a call: the word
forms, which name the function first and then give the arguments, the sign form `f(x)`
that other languages use, and the forms that give the arguments first and name the
function last, in the order of a Burmese sentence. This page covers each form, how the
arguments are checked, calls that can fail, calls on a value with `၏`, and recursion. A
call can also put a preposition (argument label) after each argument; those calls are on
[Calls written as sentences](functions-sentences.md). For a gentle start, see
[Functions](../../learn/functions/README.md) in the Learn book.

## The word forms

```text
<name> ဟူသော လုပ်ဆောင်ချက်ကို <arguments> တို့ဖြင့် လုပ်ဆောင်မည်။
<name> ဟူသော လုပ်ဆောင်ချက်ကို <arguments> တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်
```

To call a function, we write its name, then `ဟူသော လုပ်ဆောင်ချက်ကို` ("the function
called …"), then the arguments. We end the arguments with `ဖြင့်` ("with") after one
argument, or with `တို့ဖြင့်` ("with these") after two or more, which we join as a
[list in words](words.md#lists-in-words) with `နှင့်` before the last one. A function with
no parameters has no argument list.

| Form | Written | What it is |
| --- | --- | --- |
| statement | `<name> ဟူသော လုပ်ဆောင်ချက်ကို <arguments> တို့ဖြင့် လုပ်ဆောင်မည်။` (or `လုပ်ဆောင်မည်`) | a statement; a result, if the function has one, is dropped |
| expression | `<name> ဟူသော လုပ်ဆောင်ချက်ကို <arguments> တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်` | a value |
| sign form | `<name>(<arguments>)` | a value, the same call as the expression form ([The sign form](#the-sign-form)) |

We can write `မည့်` in place of `ဟူသော`, so `<name> မည့် လုပ်ဆောင်ချက်ကို …` is the same
call.

```zaka
နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
နှစ်ဆ ဟူသော လုပ်ဆောင်ချက်ကို ၄ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
နှစ်ဆ မည့် လုပ်ဆောင်ချက်ကို ၅ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၈
၁၀
```

The expression form is an operand like a number: we can print it, store it, pass it as an
argument, use it in a sum, or return it from another function. A statement that begins
with `လုပ်ဆောင်ချက်` is read as a call statement, and a sum or other expression that begins
with a call puts that call in parentheses; see [Expressions](expressions.md).

`<name> ဟူသော လုပ်ဆောင်ချက်` alone is the function as a value
([Function values](function-values.md)), so the word after `ကို` decides between the two.
A verb there makes the function value the object of that verb, so
`နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက်ကို ပြန်ပေးမည်။` returns the function. An argument or a call
word (`လုပ်ဆောင်မည်`, `လုပ်ဆောင်သော ရလဒ်`) there makes a call.

The name can also be a variable, a parameter or a constant that has a function value, and
the word forms can call a field or a cell that has one, in parentheses; see
[Function values](function-values.md#calling-a-function-value).

## The sign form

```text
<name>(<argument>၊ <argument>)
```

The sign form is the same call as the expression form, just as `+` is the
same sum as its word form: it has the same order of work, the same checks and the same
error codes. Its rules:

- The `(` touches the name. With a space, `f (a)`, the name and the group in parentheses
  remain apart.
- `၊` goes between the arguments, and `f()` calls a function with no parameters.
- A struct name followed with `(` is a [struct literal](structs.md#struct-literals), and so
  is another name followed with `(<field>:`, which the check then reports for the name.
- A sign-form call is a value, never a statement, so a call whose result is dropped uses
  the statement form.

```zaka
နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
နှစ်ထပ်(၇) ကို ပြသမည်။
နှစ်ထပ်(၃) + နှစ်ထပ်(၄) ကို ပြသမည်။
နှစ်ထပ်(နှစ်ထပ်(၂)) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၉
၂၅
၁၆
```

## Arguments

An argument is a full expression, and `၊` ends it, so an argument needs no parentheses:

```zaka
အစီအစဉ် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသမည်။
ခ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အစီအစဉ် ဟူသော လုပ်ဆောင်ချက်ကို ၁ + ၁ နှင့် ၂ * ၅ တို့ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂
၁၀
```

The compiler checks each call against the function signature:

| Mistake | Code |
| --- | --- |
| the number of arguments differs from the number of parameters | ZK0314 |
| an argument does not fit the data kind (type) of its parameter | ZK0401 |
| the expression form of a function that has no result, used as a value | ZK0315 |

```zaka-error ZK0314
ပြ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပြ ဟူသော လုပ်ဆောင်ချက်ကို ၁ နှင့် ၂ တို့ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0314]: function "ပြ" takes 1 arguments, 2 given
```

```zaka-error ZK0401
ပြ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပြ ဟူသော လုပ်ဆောင်ချက်ကို မှန် ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0401]: expected isize, found bool
```

What a call does with each argument, and how a function changes the caller's variable, is
on [Function parameters](functions-parameters.md#how-values-are-passed).

## Calls that can fail

A function whose result is a result kind (result type) can fail, and a call to it passes
the failure on in one of these forms:

- `… လုပ်ဆောင်သော ရလဒ်ကောင်း` in place of `… လုပ်ဆောင်သော ရလဒ်`;
- a `?` after the `)` of a sign-form call;
- `၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်` at the end of a call statement.

With none of these, the expression form returns the whole result, which a test and a
conversion can read. A call statement with no pass-on is ZK0443. See
[Errors and results](error-handling.md).

## Arguments first

A call can also give its arguments first and name its function last, which is the order of
an object and its verb in a Burmese sentence. We end the arguments with `သုံး၍`
("using"), with `ကို` before it after one argument and `တို့ကို` after several, which
use `နှင့်` before the last one. The function then comes in a long form or a short form,
and each one ends as a statement or as a value:

| Form | Statement | Value |
| --- | --- | --- |
| long | `<arguments> ကို သုံး၍ <name> ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။` | `<arguments> ကို သုံး၍ <name> ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ်` |
| short | `<arguments> ကို သုံး၍ <name> မည်။` | `<arguments> ကို သုံး၍ <name> သည့် ရလဒ်` |

For a function with no parameters, we leave out the arguments and `ကို သုံး၍`, which
gives `<name> မည်။`, `<name> သည့် ရလဒ်`, and the long forms of
[The word forms](#the-word-forms). Each form is the same call as the forms above, with the
same order of work, the same checks and the same error codes. A failure passes on as it
does there: `ရလဒ်ကောင်း` in place of `ရလဒ်` after the call, or
`၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်` (or `ပြန်ပေးမည်`) after `မည်` or
`လုပ်ဆောင်မည်`.

```zaka
ပေါင်း ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က + ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပြ ဟူသော လုပ်ဆောင်ချက် သည် န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ခုနစ် ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
၇ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၃ နှင့် ၄ တို့ကို သုံး၍ ပေါင်း သည့် ရလဒ် ကို ပြသမည်။
၅ ကို သုံး၍ ပြ မည်။
၆ ကို သုံး၍ ပြ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
ခုနစ် သည့် ရလဒ် နှင့် ၁ တို့ကို သုံး၍ ပေါင်း သည့် ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၇
၅
၆
၈
```

The rules of these forms:

- An argument is a notation expression: a name, a literal, a sum, a sign-form call, or a
  short call with no arguments. A sentence that gives a value, another arguments-first
  call included, goes in parentheses.
- We can also write the value end as `မည့် ရလဒ်` or `သော ရလဒ်`, the same as
  `သည့် ရလဒ်`, each with its `ရလဒ်ကောင်း` form.
- `မည်` and `သည့်` are separate words, with a space between each one and the name, unless
  the file sees a function of that name. So `ပြမည်` is one name, but a file that sees a
  function `ပြ` reads it as `ပြ မည်`, and `ပေါင်းသည့်ရလဒ်` as `ပေါင်း သည့် ရလဒ်` (see
  [What the flexible grammar costs](functions-sentences.md#what-the-flexible-grammar-costs)).

| Form | Rule | Code |
| --- | --- | --- |
| the particle before `သုံး၍` | `ကို` after one argument, `တို့ကို` after two or more | ZK0246 |
| two or more arguments | `နှင့်` before the last one ([Lists in words](words.md#lists-in-words)) | ZK0245 |
| `လုပ်ဆောင်မည်` right after `သုံး၍` | an error: the long form ends in `<name> ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်` | ZK0201 |
| a statement end, `မည်` or `လုပ်ဆောင်မည်`, where a value is needed | an error | ZK0204 |

```zaka-error ZK0204
ပြ ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ၅ ကို သုံး၍ ပြ မည် ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0204]: statement verb "ပြ မည်" cannot stand inside an expression
```

A call can also put a preposition after each argument, and starting and ending phrases
around the arguments, which makes the whole call a Burmese sentence; see
[Calls written as sentences](functions-sentences.md).

## Calls on a value

```text
<value> ၏ <function>
<value> ၏ <function>(<arguments>)
```

`x ၏ f` is the call `f(x)` when the file sees a function `f` that accepts the data kind of
`x`, and `x ၏ f(a)` is the call `f(x, a)`: the value before `၏` comes first, as the
subject of a Burmese sentence does. The function is chosen as for a sign-form call with
`x` first, and prepositions do not count. When there is no such function, `x ၏ f` reads
the field `f` of `x`. Reads chain from the left, so `x ၏ a ၏ f` is `f(x ၏ a)`.

- A function wins: when a function `f` accepts `x`, `x ၏ f` calls it, even when the struct
  has a field `f`.
- `x ၏ {f}`, with the name in braces, reads the field alone and never calls. Braces are
  the last resort: we write them where a function `f` that accepts `x` hides the field, as
  below, and nowhere else.
- A reading that the language gives after `၏`, such as `အကွက် n` or `မိတ္တူ`, wins over a
  function of that name.
- A field that holds a function value: when no function, generic, variable or trait
  function `f` accepts `x`, and the field `f` of `x` holds a
  [function value](function-values.md), `x ၏ f(a)` calls that function value with `a`. It
  is the short form of `(x ၏ f) ဟူသော လုပ်ဆောင်ချက်ကို a ဖြင့် လုပ်ဆောင်သော ရလဒ်`, and it needs
  the parentheses, so `x ၏ f` with none still reads the field.

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု ပါဝင်သည်။

နှုတ်ဆက်စာ ဟူသော လုပ်ဆောင်ချက် သည်
လ ဟူသော လူ တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"မင်္ဂလာပါ { လ ၏ {အမည်} }" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

အမည် ဟူသော လုပ်ဆောင်ချက် သည်
လ ဟူသော လူ တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"the function" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် သည်
စ ဟူသော စာသား တစ်ခု ကို လည်းကောင်း
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"{စ}{စ}{န}" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လ သည် လူ(အမည်: "မောင်") ဖြစ်ပါစေ။
လ ၏ နှုတ်ဆက်စာ ကို ပြသမည်။ // the call နှုတ်ဆက်စာ(လ)
လ ၏ အမည် ကို ပြသမည်။ // the function wins: အမည်(လ)
လ ၏ {အမည်} ကို ပြသမည်။ // the field
"ab" ၏ နှစ်ဆ(၃) ကို ပြသမည်။ // နှစ်ဆ("ab", ၃)
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ မောင်
the function
မောင်
abab၃
```

## Recursion

A function can call itself, straight or through other functions. Nothing marks a function
as recursive, and the call is a plain call. Each call has its own parameters and its own
local variables.

```zaka
// even and odd, each defined with the other
စုံသလား ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် မှန် ကို ပြန်ပေးမည်။
မကိန်းလား ဟူသော လုပ်ဆောင်ချက်ကို န - ၁ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

မကိန်းလား ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် မမှန် ကို ပြန်ပေးမည်။
စုံသလား ဟူသော လုပ်ဆောင်ချက်ကို န - ၁ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စုံသလား ဟူသော လုပ်ဆောင်ချက်ကို ၁၀ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
စုံသလား ဟူသော လုပ်ဆောင်ချက်ကို ၇ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
မမှန်
```

The VM allows 4096 calls that wait to return at one time, and the next call stops the
program. We can use `zakac run --max-depth <calls>` to set another limit. A native program,
built with `zakac build --backend c` or `--backend native` or run with `zakac run --jit`,
counts calls the same under the default checks and accepts `--max-depth` too
([Native programs](../compiler/native.md#checks)). A recursion without a base case meets
this limit:

```zaka-stop
အဆုံးမဲ့ ဟူသော လုပ်ဆောင်ချက် သည် န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အဆုံးမဲ့ ဟူသော လုပ်ဆောင်ချက်ကို န + ၁ ဖြင့် လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အဆုံးမဲ့ ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: more than 4096 nested calls
```

See [When a program stops](../vm/traps.md) for the other stops.
