---
title: Functions
description: How to declare a function, what its parameters and result mean, how a call passes values, and the errors a header or a call can cause.
---

# Functions

A function is a named group of statements. This page states the whole rules: the shape of
a header, the kinds of parameters and results, the two forms of a call, how values
travel into a call, and the errors the compiler reports. For a gentle start, see
[Functions](../../learn/functions/README.md) in the Learn book.

```zaka
// square: one parameter, one result
လုပ်ဆောင်ချက် နှစ်ထပ် သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က * က ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် နှစ်ထပ် ကို ၇ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၉
```

## Declaring a function

```text
လုပ်ဆောင်ချက် <name> သည်
<parameters> လက်ခံပြီး                          (left out when there are none)
အောက်ပါတို့ကို လုပ်ဆောင်မည်။                       (no result)
  or
အောက်ပါတို့ကို လုပ်ဆောင်ကာ <kind> ရလဒ် ကို ပြန်ပေးမည်။   (with a result)
<statements>
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

| Part | Rule |
| --- | --- |
| name | a name like each other name |
| parameters | `<kind> <name>`, joined with `၊`; the last two can be joined with `နှင့်` |
| closer of the list | `ကို` after one parameter, `တို့ကို` after two or more |
| result | a single kind, written before the word `ရလဒ်` |
| end | `ပြီးပါပြီ`, `ပြီးပါပြီး`, `ပြီးဆုံးပါပြီ` or `ပြီးဆုံးပါပြီး`, all with `လုပ်ဆောင်စရာများ` before them |

The line breaks of the header are plain white space. The whole header can stand on one line.

Declaring a function runs nothing. The statements inside run each time the function is
called.

### Where a function stands

- A function with statements is declared at the top level of the program, or inside the
  statements of a function, the start block, a block, a loop or a conditional. A function
  declared inside another one is a *nested function*: it sees from its declaration to the
  end of its block, and it lists the locals around it that it uses, as copies, moves or
  borrows. See
  [Nested functions](function-values.md#nested-functions-and-closures).
- External functions, C functions and the start block are declared at the top level and
  nowhere else (ZK0214).
- A call can come before the declaration it names at the top level. The order of
  declarations does not matter there, and two functions can call each other.
- A file can declare several functions of one name when their parameters differ (see
  [One name, several functions](#one-name-several-functions)); a second function with
  the same name, parameter kinds and marks is ZK0304. Nested functions of one scope have
  no overloads: a second one of a name is ZK0304.
- A call to a name that no function has is ZK0303.

```zaka-error ZK0214
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် abs သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
၃၂ ဘစ် ကိန်းပြည့် n ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0214]: external functions, C functions and the start block can be declared at the top level and nowhere else
```

## Parameters

A parameter is a variable that lives inside the function. A call fills it with the value
of the matching argument, so it starts with that value and nothing else. Parameters follow
the rules of other local variables, with one more rule: the names of one header are
distinct (ZK0305).

A parameter can have each kind a variable can have: a number, a true/false value, a
character, a list, a pointer. What the call does with the argument depends on that kind;
see [How values are passed](#how-values-are-passed) below.

The argument must fit the kind of its parameter, under the rules of
[assignment](conversions.md): a value that widens without loss converts on its own, and
each other difference needs an explicit conversion.

```zaka
လုပ်ဆောင်ချက် ပြ သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် သေး သည် ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ပြ ကို သေး တို့ဖြင့် လုပ်ဆောင်ပါ။       // 32 bits widens to the full width
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၇
```

## Results and return

A function without a result returns nothing. A function with a result has a variable named
`ရလဒ်` ("result") of the result kind. It starts at the default of the kind (`၀` for a
number, `မမှန်` for a true/false value, `ဗလာ` for an address, a failure for a result
kind), and the call gives back
what it holds when the function ends.

A function ends in one of two manners, and both give the same result:

- **The end is reached.** The call gives back the current value of `ရလဒ်`.
- **A return statement runs.** `<value> ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။` gives back the value
  at once, and `ပြန်ပေးပါ။` alone ends a function that has no result.

There is no "missing return" error. A function with a result that reaches its end without
setting `ရလဒ်` gives back the default of its kind:

```zaka
လုပ်ဆောင်ချက် မေ့ သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် မေ့ ကို လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၀
```

A result clause can also be a whole result kind, `<kind> ရလဒ် သို့မဟုတ် <kind> အမှားအယွင်း ကို ပြန်ပေးမည်`,
or `အောင်မြင်သည့်သတင်း သို့မဟုတ် <kind> အမှားအယွင်း ကို ပြန်ပေးမည်` for a good side with no value. Such a function can fail:
`<value> ကို အမှားအယွင်းအဖြစ် ပြန်ပေးပါ။` gives back an error value instead of a result. The
rules are in [Errors and results](error-handling.md).

What the compiler does check:

| Mistake | Code |
| --- | --- |
| a value returned from a function without a result | ZK0312 |
| `ပြန်ပေးပါ။` with no value in a function with a result | ZK0313 |
| a return statement outside each function | ZK0311 |
| a result that is a list of borrows | ZK0424 |
| a value that does not fit the result kind | ZK0401 |

```zaka-error ZK0312
လုပ်ဆောင်ချက် ပြ သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0312]: function "ပြ" returns nothing, but a value is returned
```

```zaka-error ZK0313
လုပ်ဆောင်ချက် တစ် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0313]: function "တစ်" returns isize, but no value is returned
```

## Calls

A call has three forms here, and [Arguments first](#arguments-first) gives two more. The two
word forms name the function after `လုပ်ဆောင်ချက်`, and both close the argument list with
`တို့ဖြင့်` ("with these"), one argument or several. A function with no parameters has no
argument list. The sign form writes the arguments in parentheses after the name.

| Form | Written | Is |
| --- | --- | --- |
| statement | `လုပ်ဆောင်ချက် <name> ကို <arguments> တို့ဖြင့် လုပ်ဆောင်ပါ။` (or `လုပ်ဆောင်မည်`) | a statement; a result, if there is one, is dropped |
| expression | `လုပ်ဆောင်ချက် <name> ကို <arguments> တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်` | a value |
| sign form | `<name>(<arguments>)` | a value, the same call as the expression form |

The name can also be a variable, a parameter or a constant that holds a function value,
and the word forms can name a field or a cell that holds one; see
[Function values](function-values.md#calling-a-function-value).

A function whose result clause names a result kind can fail. A call to it
passes the failure on, with `ရလဒ်ကောင်း` in place of `ရလဒ်`, a `?` after the `)`, or
`၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးပါ` at the end of a call statement; or, with none
of these, the expression form gives back the whole result, which a test and a conversion
read. A call statement with no pass-on is ZK0443. See [Errors and results](error-handling.md).

The expression form is an operand like a number: print it, store it, pass it as an
argument, use it in a sum, or give it back from another function. Using the expression
form of a function that has no result is ZK0315. The number of arguments must match the
number of parameters (ZK0314), and each argument must fit its parameter (ZK0401).

An argument is a full expression, and `၊` ends it, so an argument needs no parentheses:

```zaka
လုပ်ဆောင်ချက် အစီအစဉ် သည်
ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသပါ။
ခ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် အစီအစဉ် ကို ၁ + ၁၊ ၂ * ၅ တို့ဖြင့် လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂
၁၀
```

The sign form is the same call as the expression form, as `+` is the same sum as the
word form: the same order of work, the same checks and the same codes. Its rules:

- The `(` touches the name. With a space, `f (a)`, the name and the group remain apart.
- `၊` comes between the arguments, and `f()` calls a function with no parameters.
- A struct name and `(` is a [struct literal](structs.md#struct-literals); another name and
  `(<field>:` is one too, and the check reports the name.
- A sign-form call is a value, never a statement: a call whose result is dropped takes
  the statement form.

```zaka
လုပ်ဆောင်ချက် နှစ်ထပ် သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က * က ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
နှစ်ထပ်(၇) ကို ပြသပါ။
နှစ်ထပ်(၃) + နှစ်ထပ်(၄) ကို ပြသပါ။
နှစ်ထပ်(နှစ်ထပ်(၂)) ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၉
၂၅
၁၆
```

A statement that begins with `လုပ်ဆောင်ချက်` is read as a call statement. A sum or other
expression that begins with a call is written with that call in parentheses; see
[Expressions](expressions.md).

```zaka-error ZK0314
လုပ်ဆောင်ချက် ပြ သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ပြ ကို ၁၊ ၂ တို့ဖြင့် လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0314]: function "ပြ" takes 1 arguments, 2 given
```

```zaka-error ZK0401
လုပ်ဆောင်ချက် ပြ သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ပြ ကို မှန် တို့ဖြင့် လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0401]: expected isize, found bool
```

## Arguments first

A call can also give its arguments first and name its function last, the order a Burmese
sentence puts an object and its verb in. `သုံး၍` ("using") closes the arguments, with
`ကို` after one and `တို့ကို` after several. The function then comes in a long form or a
short form, and each ends as a statement or as a value:

| Form | Statement | Value |
| --- | --- | --- |
| long | `<arguments> ကို သုံး၍ လုပ်ဆောင်ချက် <name> ကို လုပ်ဆောင်မည်။` | `<arguments> ကို သုံး၍ လုပ်ဆောင်ချက် <name> ကို လုပ်ဆောင်သော ရလဒ်` |
| short | `<arguments> ကို သုံး၍ <name> မည်။` | `<arguments> ကို သုံး၍ <name> သည့် ရလဒ်` |

For a function with no parameters, leave out the arguments and `ကို သုံး၍`: `<name> မည်။`,
`<name> သည့် ရလဒ်`, and the long forms of [Calls](#calls). Each form is the same call as
the forms above, with the same order of work, the same checks and the same codes. A
failure passes on as it does there: `ရလဒ်ကောင်း` in place of `ရလဒ်`, or
`၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးပါ` (or `ပြန်ပေးမည်`) after `မည်` or `လုပ်ဆောင်မည်`.

```zaka
လုပ်ဆောင်ချက် ပေါင်း သည်
ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က + ခ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပြ သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ခုနစ် သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
၇ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၃၊ ၄ တို့ကို သုံး၍ ပေါင်း သည့် ရလဒ် ကို ပြသပါ။
၅ ကို သုံး၍ ပြ မည်။
၆ ကို သုံး၍ လုပ်ဆောင်ချက် ပြ ကို လုပ်ဆောင်မည်။
ခုနစ် သည့် ရလဒ်၊ ၁ တို့ကို သုံး၍ ပေါင်း သည့် ရလဒ် ကို ပြသပါ။
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
- `မည်` and `သည့်` are words of their own, with a space between each and the name:
  `ပြမည်` is one name, not a call of `ပြ`.
- `ကို` closes one argument and `တို့ကို` two or more; the other particle is the warning
  ZK0510 ([Natural wording](naturalness.md#the-rules)).
- The long form ends in `လုပ်ဆောင်မည်`; `လုပ်ဆောင်ပါ` after `သုံး၍` is ZK0201.
- A statement end, `မည်` or `လုပ်ဆောင်မည်`, where a value is needed is ZK0201.

```zaka-error ZK0201
လုပ်ဆောင်ချက် ပြ သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ၅ ကို သုံး၍ ပြ မည် ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0201]: expected "သည့် ရလဒ်" or "သည့် ရလဒ်ကောင်း", found "မည်"
```

## Marked parameters

A parameter can name the word that follows its argument in a call, its *mark*, in
parentheses right after the parameter name: `စာသား လမ်းကြောင်း(၌ရှိသော)`. A call then
reads as a Burmese sentence, each argument with its mark after it and the function last:

```text
လုပ်ဆောင်ချက် <name> သည် <kind> <param>(<mark>)၊ <kind> <param>(<mark>) လက်ခံပြီး ...
<argument> <mark> <argument> <mark> <name> မည်။
```

With marks, the parameters end in `လက်ခံပြီး` straight after the last one; `တို့ကို` is left
out, since the marks tell how each argument joins the sentence. The call ends as the
[arguments-first](#arguments-first) forms do: `<name> မည်` or
`လုပ်ဆောင်ချက် <name> ကို လုပ်ဆောင်မည်` for a statement, `<name> သည့် ရလဒ်` or
`လုပ်ဆောင်ချက် <name> ကို လုပ်ဆောင်သော ရလဒ်` for a value, with `ရလဒ်ကောင်း` and the pass-on
tail as there. A function with marks takes the other call forms too.

```zaka
လုပ်ဆောင်ချက် ရေး သည်
စာသား စာ(ကို)၊ စာသား နေရာ(သို့) လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"{နေရာ}: {စာ}" ဟု ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ဖတ် သည်
စာသား လမ်း(၌ရှိသော) လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား ရလဒ် ကို ပြန်ပေးမည်။
"{လမ်း} ၏ စာ" ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ" ကို "စာအုပ်" သို့ ရေး မည်။
"ဇာတ်လမ်း" ၌ရှိသော ဖတ် သည့် ရလဒ် ကို ပြသပါ။
ကိန်းရှင် စာ သည် "ကဗျာ" ၌ရှိသော လုပ်ဆောင်ချက် ဖတ် ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
စာ ကို ပြသပါ။
လုပ်ဆောင်ချက် ရေး ကို "ဟယ်လို"၊ "မှတ်စု" တို့ဖြင့် လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
စာအုပ်: မင်္ဂလာပါ
ဇာတ်လမ်း ၏ စာ
ကဗျာ ၏ စာ
မှတ်စု: ဟယ်လို
```

The rules of marks:

- A mark is one word: a name, or a particle such as `ကို`, `မှ`, `ဖြင့်`, `နှင့်`, `ထဲသို့`,
  `တွင်`. `သည်`, `ပါ`, `ဖြစ်ကာ`, `ပြီးလျှင်` and `လက်ခံပြီး` cannot be marks, and neither can
  `၏`. A mark of two words needs them glued: `၌ရှိသော`, not `၌ ရှိသော`.
- A header marks each parameter or none of them; marks on some alone are ZK0237.
- A call gives the arguments in the order of the parameters, each with the mark of its
  parameter. Another mark is ZK0331.
- An argument is a notation expression, as in the arguments-first call; a sentence that
  gives a value, another marked call included, goes in parentheses.
- A marked call ends at a call of a function that declares marks, when the number of
  marks fits the function, and nowhere else. So a sentence that holds no such call keeps its
  meaning: `က ကို ခုနစ် သည့် ရလဒ် ဟု ထားပါ။` sets `က` when `ခုနစ်` declares no marks.

```zaka-error ZK0331
လုပ်ဆောင်ချက် ရေး သည်
စာသား စာ(ကို)၊ စာသား နေရာ(သို့) လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စာ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ" ကို "စာအုပ်" ထဲသို့ ရေး မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0331]: argument 2 of "ရေး" is marked "ထဲသို့", not "သို့"
```

```zaka-error ZK0237
လုပ်ဆောင်ချက် ရေး သည်
စာသား စာ(ကို)၊ စာသား နေရာ လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စာ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0237]: some parameters have a mark and some do not
```

## One name, several functions

A file can declare several functions of one name when their parameter kinds or their
marks differ. A call runs the one its arguments fit: an argument for each parameter, the
marks of the parameters when the call writes marks, and each argument fit for the kind of
its parameter, under the rules a call to a single function follows. The marks can tell two
functions apart on their own, so a marked call chooses with the words written too.

```zaka
လုပ်ဆောင်ချက် ပြ သည်
ကိန်းပြည့် န(ကို) လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ကိန်း {န}" ဟု ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပြ သည်
စာသား စ(ကို) လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"စာ {စ}" ဟု ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပြ သည်
စာသား စ(ထဲသို့) လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"{စ} ထဲသို့" ဟု ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၅ ကို ပြ မည်။
"ဟယ်လို" ကို ပြ မည်။
"အိမ်" ထဲသို့ ပြ မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ကိန်း ၅
စာ ဟယ်လို
အိမ် ထဲသို့
```

The rules of functions that share a name:

- Two functions of one name with the same parameter kinds and the same marks in one file
  are ZK0304.
- A call that no function of the name fits is ZK0332; the help lists the parameters of
  each.
- A call that two or more fit is ZK0333. A number with no kind fits each integer kind,
  and a call with no marks fits functions that differ in their marks alone: write a kind
  on the number, or the marks of the function meant.
- An import of the name brings each exported function of it. A file cannot declare a
  function of a name it imports (ZK0304).
- Inside the compiler, the second function of a name is `<name>#2`, the third
  `<name>#3`; the IR, the ownership check and the VM listing show these names.

```zaka-error ZK0333
လုပ်ဆောင်ချက် ပြ သည်
စာသား စ(ကို) လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပြ သည်
စာသား စ(ထဲသို့) လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ပြ ကို "ဟယ်လို" တို့ဖြင့် လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0333]: more than one function "ပြ" takes these arguments
```

```zaka-error ZK0332
လုပ်ဆောင်ချက် ပြ သည်
ကိန်းပြည့် န(ကို) လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပြ သည်
စာသား စ(ကို) လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
မှန် ကို ပြ မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0332]: no function "ပြ" takes these arguments
```

## External functions

A function can come from outside the program, `ပြင်ပမှလာသော`: its header names its
parameters and its result, and no statements follow it, since the VM does its work while
the program runs. The header is one sentence:

| Parameters | Result | Written |
| --- | --- | --- |
| none | none | `လုပ်ဆောင်ချက် <name> သည် ပြင်ပမှလာသော လုပ်ဆောင်ချက်ဖြစ်သည်။` |
| some | none | `... လုပ်ဆောင်ချက်ဖြစ်၍ <parameters> လက်ခံမည်။` |
| none | a result | `... လုပ်ဆောင်ချက်ဖြစ်၍ <kind> ရလဒ် ကို ပြန်ပေးမည်။` |
| some | a result | `... လုပ်ဆောင်ချက်ဖြစ်၍ <parameters> လက်ခံပြီး <kind> ရလဒ် ကို ပြန်ပေးမည်။` |

`လုပ်ဆောင်ချက်` can take `တစ်ခု` after it (`လုပ်ဆောင်ချက်တစ်ခုဖြစ်၍`), and `ပြင်ပမှလာသော`
can be written apart (`ပြင်ပ မှ လာသော`). The parameters are those of a regular header,
marks included, and so is the result kind. A call of an external function is a call
like each other, in each call form.

The VM serves a fixed set of functions, so an external function must be one of them: the
same name, the same parameter kinds, and a result that can fail with the error
enumeration of input and output. Another one is ZK0334, whose help lists the set. The
stdlib declares each of them, so a program imports them from there rather than
writing the headers.

```zaka-error ZK0334
လုပ်ဆောင်ချက် ဟိုဟို သည် ပြင်ပမှလာသော လုပ်ဆောင်ချက်ဖြစ်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0334]: the VM serves no function "ဟိုဟို" with these parameter and result kinds
```

## C functions

A text with the name of a shared lib in place of `ပြင်ပ` makes a C function:
`"libm" မှလာသော`. The function is the C function of its name in that lib, so its name is a
C name: ASCII letters, digits and `_`, not starting with a digit (ZK0337). Each C function
names its lib, libc too (`"libc"`). The header has the forms of the table above, and a
call is a call like each other.

```zaka
လုပ်ဆောင်ချက် abs သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
၃၂ ဘစ် ကိန်းပြည့် n ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် abs ကို -၄၂ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၂
```

C takes each parameter and gives its result as a whole value, laid out as in C, so the
kinds that cross are those C has:

| Kind | In C |
| --- | --- |
| integers of 8 to 64 bits, `ကိန်းပြည့်` | `int8_t` to `int64_t`, `uint8_t` to `uint64_t` |
| `၃၂ ဘစ် ဒဿမကိန်း`, `ဒဿမကိန်း` | `float`, `double` |
| `မှန်မှားတန်ဖိုး` | `_Bool` |
| a borrowed address, a raw address | a pointer |
| a struct whose fields are these kinds, structs or counted lists of them | a struct, passed whole |
| `စာသား`, `စာသား တစ်စိတ်တစ်ပိုင်း`, as a parameter | `const char *` |
| `စာသား ရလဒ် သို့မဟုတ် မအောင်မြင်သည့်သတင်း`, as a result | `const char *`, NULL for the failure |
| a function kind whose parameters and result cross, as a parameter | a pointer to a function |

C reads a string up to a zero octet, which Zaka text does not hold, so a text parameter
reaches C as the address of a duplicate of its u8s with a zero octet after them. The duplicate
lives until the call ends: C reads it but cannot change the text, and a zero octet
inside the text ends the string there for C.

```zaka
လုပ်ဆောင်ချက် strlen သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
စာသား s ကို လက်ခံပြီး လက္ခဏာမဲ့ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် strlen ကို "မောင်" တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၅
```

Each other kind is ZK0465: an owner, which the call would free as it ends; plain lists
and slices, which C has no form for; function values that do not cross (see
[Callbacks](#c-functions)); text as a bare result, since C can give back NULL for it (see
[Text as a result](#c-functions)); and a result that can fail. A result
cannot be a borrow, since C has no place to lend from: it gives back a raw address. A
parameter with the edit mark is ZK0466; to let C write a value back, pass a borrowed
address, as `frexp` takes its `int *`:

```zaka
လုပ်ဆောင်ချက် frexp သည် "libm" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
ဒဿမကိန်း x၊ ငှားထားသော ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ e တို့ကို လက်ခံပြီး ဒဿမကိန်း ရလဒ် ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ထပ်ကိန်း သည် ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် frexp ကို ၈.၀၊ ထပ်ကိန်း ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ထပ်ကိန်း ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၀.၅
၄
```

```zaka-error ZK0465
လုပ်ဆောင်ချက် getenv သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
စာသား n ကို လက်ခံပြီး စာသား ရလဒ် ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0465]: a C function cannot take or give back a value of the kind "text"
```

Each declaration of one C name, in each file and among overloads, gives it the same
parameter and result kinds (ZK0338). Like each function that comes from outside, a C
function has no address (ZK0464): a function of the program that calls it stands in for
it.

### Text as a result

A C function that gives back a string writes its result as `စာသား ရလဒ် သို့မဟုတ်
မအောင်မြင်သည့်သတင်း` ([a result kind](error-handling.md#result-kinds) with no error
value). NULL is the failure; each other address is a good result. Zaka copies the string
up to its zero octet into new text of its own, so the result is a plain text that the
program owns. Octets that are no UTF-8 stop the program with the
[not UTF-8 trap](../vm/traps.md#all-the-traps).

Who frees the string of C depends on the result clause:

| Result clause | C's string | For |
| --- | --- | --- |
| `... ကို ပြန်ပေးမည်` | C keeps it; Zaka never frees it | `getenv`, `strerror`, `strstr`, static buffers |
| `... ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်` | C hands it over; Zaka frees it with libc's `free` once its text holds the octets | `strdup` and each function that allocates the string it gives |

`အပိုင်အနေဖြင့်` stands on a C function whose result is text or a failure; on each other
function it is ZK0467. Without it, a string that C allocates is never freed.

Zaka takes the text before it frees the duplicates of the text arguments, so a result
that points into an argument, as `strstr` gives, is copied while it is still there.

```zaka
လုပ်ဆောင်ချက် strstr သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
စာသား s၊ စာသား part တို့ကို လက်ခံပြီး စာသား ရလဒ် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် strstr ကို "မင်္ဂလာပါ ကမ္ဘာ"၊ "ကမ္ဘာ" တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
လုပ်ဆောင်ချက် strstr ကို "abc"၊ "x" တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရလဒ်ကောင်း("ကမ္ဘာ")
မအောင်မြင်သည့်သတင်း
```

`strdup` hands its string over, so its result clause holds `အပိုင်အနေဖြင့်`:

```zaka
လုပ်ဆောင်ချက် strdup သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
စာသား s ကို လက်ခံပြီး စာသား ရလဒ် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် strdup ကို "စံကား" တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရလဒ်ကောင်း("စံကား")
```

A program can also take the string as a raw address and free it itself, as Rust and Go
programs do: read its length with `strlen`, bring its octets into a list of its own with
`memcpy`, free it with `free`, and read the list as text. The VM cannot read through an
address C gives, so `memcpy`, a C function, copies the octets.

```zaka
လုပ်ဆောင်ချက် strdup သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
စာသား s ကို လက်ခံပြီး လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာအကြမ်း ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် strlen သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာအကြမ်း s ကို လက်ခံပြီး လက္ခဏာမဲ့ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် memcpy သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
ငှားထားသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ dst၊ လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာအကြမ်း src၊ လက္ခဏာမဲ့ ကိန်းပြည့် n တို့ကို
လက်ခံပြီး လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာအကြမ်း ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် free သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာအကြမ်း p ကို လက်ခံမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် လုပ်ဆောင်ချက် strdup ကို "zaka" တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် ရေ သည် လုပ်ဆောင်ချက် strlen ကို ပ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် ဘောင် သည် အကွက် ၆၄ ခုရှိသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ပြန် သည် လုပ်ဆောင်ချက် memcpy ကို ဘောင် ၏ အကွက် ၀ ၏ လိပ်စာ၊ ပ၊ ရေ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် free ကို ပ တို့ဖြင့် လုပ်ဆောင်ပါ။
(ဘောင် ၏ အကွက် ၀ မှစ၍ ရေ ကွက်) ကို စာသား တစ်စိတ်တစ်ပိုင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
zaka
```

The list here holds 64 octets; a longer string needs a larger list, or a block from the
heap.

### Callbacks

A parameter of a function kind crosses to C as a pointer to a function, when each
parameter and the result of the kind is an integer up to 64 bits, a `၃၂ ဘစ် ဒဿမကိန်း`, a
`ဒဿမကိန်း`, a `မှန်မှားတန်ဖိုး`, or a borrowed or raw address. The kind has no edit mark,
no struct passed whole and no text, and its result is no borrow. Each other function kind
is ZK0465. The argument is the address of a function of the program, or `ဗလာ`, which C
gets as NULL.

```zaka
လုပ်ဆောင်ချက် qsort သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
ငှားထားသော ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ base၊ လက္ခဏာမဲ့ ကိန်းပြည့် count၊ လက္ခဏာမဲ့ ကိန်းပြည့် size၊
(ငှားထားသော ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ၊ ငှားထားသော ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ တို့ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) compare
တို့ကို လက်ခံမည်။

လုပ်ဆောင်ချက် ငယ်စဉ် သည်
ငှားထားသော ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ က၊ ငှားထားသော ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ၃၂ ဘစ် ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
(က ၏ တန်ဖိုး) - (ခ ၏ တန်ဖိုး) ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် အကွက် ၅ ခုရှိသော ၃၂ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၅၊ ၂၊ ၉၊ ၁၊ ၇] ဖြစ်ပါစေ။
ကိန်းရှင် အရွယ် သည် ၃၂ ဘစ် ကိန်းပြည့် ၏ အရွယ်အစား ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် qsort ကို ကိန်းများ ၏ အကွက် ၀ ၏ လိပ်စာ၊ ၅၊ အရွယ်၊ လုပ်ဆောင်ချက် ငယ်စဉ် ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်ပါ။
ကိန်းများ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၂၊ ၅၊ ၇၊ ၉]
```

Each build gives C a pointer it can call:

- `zakac run` makes one native closure for each function a call gives C, on the first
  such call, and keeps it for the run. The callback runs the function in the VM until it
  returns.
- A native build passes the code address of the function.

A trap inside a callback waits until C returns, then stops the program; each later
callback of that C call gives 0. Text that a callback prints in the VM comes out after
the output of the C call itself, where a native build keeps the order. A callback must
not resize or free a block that C holds an address into, since the native address goes
stale.

A [closure](function-values.md#nested-functions-and-closures), the address of a nested
function with captures, cannot go to C, since C calls a bare code address: ZK0723 when the
address is written in the call, and the [closure escape trap](../vm/traps.md#all-the-traps)
when the compiler cannot tell.

### Other values

A C function whose parameters end in `...`, as `printf` does, takes more arguments than
it names. Its header ends the parameters with `နှင့် အခြားတန်ဖိုးများ` ("and other
values"), after the last parameter it names:

```text
... စာသား fmt နှင့် အခြားတန်ဖိုးများ ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
```

A call gives an argument for each parameter, then each other value it wants (too few is
ZK0314). An other value has the kind it has on its own: a number literal is a
`ကိန်းပြည့်` or a `ဒဿမကိန်း`. C reads it as C promotes it: an integer narrower than 32
bits and a boolean as an `int`, a `၃၂ ဘစ် ဒဿမကိန်း` as a `double`. Text goes as a duplicate
that ends in a zero octet, and a struct goes whole, so C reads it as it reads a struct parameter. The format must
match the kinds, as in C: `%ld` for a `ကိန်းပြည့်`, which has 64 bits, and `%d` for a
`၃၂ ဘစ် ကိန်းပြည့်` or a boolean. Functions the host serves take no other values (ZK0334).

```zaka
လုပ်ဆောင်ချက် snprintf သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
ငှားထားသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ buf၊ လက္ခဏာမဲ့ ကိန်းပြည့် size၊ စာသား fmt နှင့် အခြားတန်ဖိုးများ ကို
လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဘောင် သည် အကွက် ၃၂ ခုရှိသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ရေ သည် ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် snprintf ကို ဘောင် ၏ အကွက် ၀ ၏ လိပ်စာ၊ ၃၂၊ "%s=%ld"၊ "x"၊ ၄၂ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
(ဘောင် ၏ အကွက် ၀ မှစ၍ ရေ ကွက်) ကို စာသား တစ်စိတ်တစ်ပိုင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
x=42
```

### The lib

The text before `မှလာသော` is a lib name or a path to the file of a lib. A name, such as
`"libsqlite3"`, is looked up where the host keeps its libs. A path holds a `/` or a `\`,
or ends in `.so`, `.dylib` or `.dll`, such as `"./libpair.so"` or `"/opt/x/libfoo.dylib"`.
A relative path starts at the folder of the file that declares the function, as an import
does, so the program finds its lib wherever it runs from.

### How each build calls C

- `zakac run` opens each lib as the program first calls a C function: `libX` is the file
  `libX.dylib`, `libX.so` or `X.dll`, a path opens as it is, and a C name the lib does not
  hold is looked up in the process, which holds libc. An address that lies in a block of
  the VM reaches C as the native address of that octet, so C reads and writes the block in
  place. After each call the VM writes out what C holds back for its streams, so the text
  a C function prints comes in order with the text the program prints. A lib that does
  not open, or a C name no lib holds, stops the program with `ForeignLink`.
- The VM in the browser opens no lib: a call of a C function stops the program with
  `ForeignCall`.
- `zakac build --target c` declares each C function in a C file of its own and links
  `-lX` for each lib `libX` other than libc and libm, and a lib given as a path as it is,
  with its folder as a run path. Give the folder of a named lib with
  `--ld-flag -L<folder>`. On macOS a lib given as a path must name itself with an
  absolute path or `@rpath/...` (its install name), since a bare file name there is
  looked up in the folder the program starts from.

C does what C does: a C function that writes past a block, or keeps an address after
the call, harms the run as it would harm a C program. A native address that C gives
back, and that lies in no block, passes back to C as it is, but the VM cannot read
through it. C can call back into the program through a function value of a kind that
crosses (see [Callbacks](#c-functions)).

## Recursion

A function can call itself, at once or through other functions. Nothing marks a function
as recursive; the call is a plain call. Each call has its own parameters and its own
`ရလဒ်`.

```zaka
// even and odd, each defined with the other
လုပ်ဆောင်ချက် စုံသလား သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး ရလဒ် ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် မှန် ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်ချက် မကိန်းလား ကို န - ၁ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် မကိန်းလား သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး ရလဒ် ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် မမှန် ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်ချက် စုံသလား ကို န - ၁ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် စုံသလား ကို ၁၀ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
လုပ်ဆောင်ချက် စုံသလား ကို ၇ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
မမှန်
```

The VM allows 4096 calls that wait to return at one time; the next call stops the
program. `zakac run --max-depth <calls>` sets another limit. A recursion without a base
case meets this limit:

```zaka-stop
လုပ်ဆောင်ချက် အဆုံးမဲ့ သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် အဆုံးမဲ့ ကို န + ၁ တို့ဖြင့် လုပ်ဆောင်ပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် အဆုံးမဲ့ ကို ၁ တို့ဖြင့် လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: more than 4096 nested calls
```

See [When a program stops](../vm/traps.md) for the other stops.

## Lists and structs as results

A function can give back a list or a struct. The result follows the rules of an
assignment: the caller gets the whole value, and a list or struct that holds an owner
moves to the caller, which then frees the blocks it owns. See [Duplicating and
moving](structs.md#duplicating-and-moving).

```zaka
ဒေတာအမျိုးအစား နေရာ တွင် ကိန်းပြည့် x၊ ကိန်းပြည့် y တို့ ပါဝင်သည်။
လုပ်ဆောင်ချက် နေရာသစ် သည်
ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ နေရာ ရလဒ် ကို ပြန်ပေးမည်။
x က၊ y ခ ရှိသော နေရာတစ်ခု ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် အစဉ် သည်
ကိန်းပြည့် စ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း ရလဒ် ကို ပြန်ပေးမည်။
၀ မှ ၂ အထိ န အနေဖြင့် ရေတွက်၍ ရလဒ် ၏ အကွက် န ကို စ + န ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် နေရာသစ် ကို ၃၊ ၄ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
လုပ်ဆောင်ချက် အစဉ် ကို ၅ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
နေရာ(x: ၃၊ y: ၄)
[၅၊ ၆၊ ၇]
```

- A plain list result takes its count from the value given back. A counted list
  result has a written count.
- A list whose cells are borrows cannot be a result (ZK0424). A borrow lives in a local or
  a parameter, so it cannot leave the function inside a list.
- The caller keeps a slot for the result of each call. With the default,
  `--results in-place`, the function builds a returned literal, or the local that each
  return gives back, straight in that slot. With `--results duplicate`, the function keeps
  the value in its own frame and the return duplicates it into the slot. The program prints the same in
  both modes; see [Commands](../compiler/commands.md#options-of-the-compile-stages).

```zaka-error ZK0424
လုပ်ဆောင်ချက် ငှား သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ငှားထားသော အကွက် ၂ ခုရှိသော ကိန်းပြည့် လိပ်စာ စာရင်း ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0424]: a function cannot give back a list whose cells are borrows
```

## Edit references

A parameter written `လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သော <kind> <name>` ("that the function can
change") is the caller's place itself, not a duplicate: a change to it changes the caller's
variable. The call marks the argument too, with `(လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်)`
("the function can change it") after it, so a reader of the call sees which variable
changes. The mark works in the word form and the sign form of a call alike.

```zaka
ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက် တို့ ပါဝင်သည်။
လုပ်ဆောင်ချက် တိုး သည်
လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သော ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို န + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ၅ ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် လူ(အသက်: ၃၀) ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် တိုး ကို က (လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်) တို့ဖြင့် လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် တိုး ကို ပ ၏ အသက် (လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်) တို့ဖြင့် လုပ်ဆောင်ပါ။
က ကို ပြသပါ။
ပ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၆
လူ(အသက်: ၃၁)
```

The rules:

- The reference lasts for the call alone. The name stands for the value, so the function
  cannot keep the reference: setting another place from it, returning it or storing it
  makes a duplicate, as for each other value.
- The argument is a place: a variable, a field or a cell (ZK0410 for another expression).
  A constant cannot be changed (ZK0306).
- The place has the kind of the parameter, with no widening (ZK0401).
- The place appears in no other argument of the same call, as a whole or through a field
  or a cell (ZK0433): the function would read it through one parameter while it changes
  it through another.
- Both ends take the mark: an argument for such a parameter with no mark is ZK0431, and
  a mark on an argument for another parameter is ZK0432.

Returning the new value, `က ကို နောက်တစ်(က) ဟု ထားပါ။`, does the same work for one
value. An edit reference suits a function that changes several places at once, or a
large struct in place.

```zaka-error ZK0431
လုပ်ဆောင်ချက် တိုး သည်
လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သော ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို န + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ၅ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် တိုး ကို က တို့ဖြင့် လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0431]: the function changes this argument; mark it with "(လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်)"
```

```zaka-error ZK0433
လုပ်ဆောင်ချက် ထား သည်
လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သော ကိန်းပြည့် န၊ ကိန်းပြည့် မ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို မ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ၅ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ထား ကို က (လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်)၊ က + ၁ တို့ဖြင့် လုပ်ဆောင်ပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0433]: "က" is changed through another argument of this call, so it cannot appear here
```

## How values are passed

Each parameter is a new variable, and the call stores the argument in it. What that store
does depends on the kind of the parameter, as it does for a variable set from
another:

| Kind of the parameter | What the call does | Can the function change the caller's value? |
| --- | --- | --- |
| number, true/false value, character | duplicates the value | no |
| plain list | lends the caller's cells until the first write, which copies them | no; pass the address to change it |
| counted list, struct | duplicates each cell and field | no; pass the address to change it |
| text, or a list or struct with text in it | duplicates it | no |
| `ငှားထားသော … လိပ်စာ` (a borrow) | duplicates the address; the caller keeps the owner | through the address |
| `… လိပ်စာ` (an owner) | **moves** the block to the function; the caller's variable holds `ဗလာ` afterwards | the function owns the block now |
| `… လိပ်စာအကြမ်း` (raw) | duplicates the address | through the address, and nothing is tracked |

A duplicate is what the call means; the compiler skips the work where nothing can tell.
A plain list parameter is lent the caller's cells, with a room of 0, for the length of the
call. Its first write, to a cell or to the whole list, first copies the cells into a block
of its own (the VM op `own`), so the caller's list never changes. A counted list or struct parameter that the function never
writes, nor views nor takes the address of, is read in the caller's storage through its
address. Text the caller reads again after
the call is lent to the parameter for the length of the call, and text the caller reads no
more moves in. The caller's value is duplicated at the call after all when the function,
or a call inside the same arguments, sets the global given, or when the address of the
value or a view of it is taken somewhere. See [Text](text.md#copies).

### Values and lists are duplicated

A change to a parameter remains in the function:

```zaka
လုပ်ဆောင်ချက် တိုးပြ သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို က + ၁ ဟု ထားပါ။
က ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် တိုးပြ ကို ခ တို့ဖြင့် လုပ်ဆောင်ပါ။
ခ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၆
၅
```

A list parameter is the same: a change to it remains in the function. A function that must
change the caller's list takes the address of the list. The parameter kind is then the list
kind with `လိပ်စာ` after it, and `ငှားထားသော` in front:

```zaka
// the parameter holds the address of the caller's list
လုပ်ဆောင်ချက် နှစ်ဆတိုး သည်
ငှားထားသော ကိန်းပြည့် စာရင်း လိပ်စာ ကိန်းများ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၀ မှ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၁ အထိ န အနေဖြင့် ရေတွက်၍ ကိန်းများ ၏ အကွက် န ကို (ကိန်းများ ၏ အကွက် န) * ၂ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မူရင်း သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် နှစ်ဆတိုး ကို မူရင်း ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်ပါ။
မူရင်း ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၂၊ ၄၊ ၆]
```

### Addresses give the function a route in

A parameter that borrows a variable lets the function read and write that variable. The
caller passes `x ၏ လိပ်စာ`, and the function writes through the borrow. This is also the
route to get more than one result out of a call:

```zaka
// swap: exchange the values of two integers
လုပ်ဆောင်ချက် လဲလှယ် သည်
ငှားထားသော ကိန်းပြည့် လိပ်စာ က နှင့် ငှားထားသော ကိန်းပြည့် လိပ်စာ ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ယာယီ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ တန်ဖိုး ဖြစ်ပါစေ။
က ၏ တန်ဖိုး ကို ခ ၏ တန်ဖိုး ဟု ထားပါ။
ခ ၏ တန်ဖိုး ကို ယာယီ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပထမ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် ဒုတိယ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် လဲလှယ် ကို ပထမ ၏ လိပ်စာ၊ ဒုတိယ ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်ပါ။
ပထမ ကို ပြသပါ။
ဒုတိယ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂
၁
```

The variables of a function end with the call, so a function never gives back a borrow of
one of its own variables: that is ZK0706, in [Ownership and borrowing](ownership.md).

### Blocks are handed over, lent or created

For a pointer to a block of the heap, the header of the function tells what happens to the
block, and the call must agree:

| In the header | The call |
| --- | --- |
| parameter `ငှားထားသော ကိန်းပြည့် လိပ်စာ ပ` | lends the block: write `ပ ၏ တန်ဖိုးအငှား`; the caller keeps it |
| parameter `ကိန်းပြည့် လိပ်စာ ပ` | hands the block over: the caller's variable holds `ဗလာ` afterwards |
| result `ကိန်းပြည့် လိပ်စာ ရလဒ်` | the function made a new block, and the caller owns it |
| result `ငှားထားသော ကိန်းပြည့် လိပ်စာ ရလဒ်` | a borrow into a block that the caller lent, or at a top-level variable |

The full rules, with one error for each manner of getting them wrong, are in [Ownership and
borrowing](ownership.md).

## The start block

Each program starts in its start block, a block of its own after the word `ပရိုဂရမ်`
("program"). Its header has the parts of a function header, with no name. The statements
that run go there or into functions: the top level of a file holds declarations alone.

```text
ပရိုဂရမ်သည်
<parameters> လက်ခံပြီး                                    (left out when there are none)
အောက်ပါတို့ကို လုပ်ဆောင်မည်။                                (exit code 0)
  or
အောက်ပါတို့ကို လုပ်ဆောင်ကာ <integer kind> ရလဒ် ကို ပြန်ပေးမည်။   (with an exit code)
<statements>
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

| Part | Rule |
| --- | --- |
| end | `ပရိုဂရမ်` with one of the four spellings of the done word: `ပြီးဆုံးပါပြီး`, `ပြီးဆုံးပါပြီ`, `ပြီးပါပြီး` or `ပြီးပါပြီ`; the closer of other blocks, `လုပ်ဆောင်စရာများ ...`, does not close it (ZK0201) |
| parameters | none, one or two, each a `စာသား စာရင်း`, with names the program chooses |
| the first | the name of the program, then the arguments of the run, one text for each |
| the second | the environment variables, one `NAME=VALUE` text for each |
| result | none, or an integer kind: the exit code of the run; the clauses of a [failing function](error-handling.md) can be used too |

```zaka
ကိန်းရှင် နှုတ်ခွန်း သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မင်္ဂလာပါ" ဖြစ်ပါစေ။

ပရိုဂရမ်သည်
စာသား စာရင်း ညွှန်ကြားချက်များ နှင့် စာသား စာရင်း ပတ်ဝန်းကျင် တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
နှုတ်ခွန်း ကို ပြသပါ။
ညွှန်ကြားချက်များ ၏ အကွက်အရေအတွက် ကို ပြသပါ။
၀ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ
၁
```

The first cell is the name of the program, as in C, Rust and Python, so the count is at
least 1. On the command line it is the input as written, or `-` for standard input; the
arguments come after `--`, and the environment is the one `zakac` runs in. The Run
buttons of these pages give the name `main.zaka` and the Explorer `explorer.zaka`, with
no arguments and a blank environment:

```console
$ zakac run greet.zaka -- ကက ခခ
မင်္ဂလာပါ
၃
$ echo $?
0
```

### How a program runs

1. The top-level variables and constants get their starting values, in order, those of
   imported files first.
2. The start block runs, with the arguments and the environment.
3. The text that the top-level variables hold is given back, as at the end of each
   program.

The result of the start block is the exit code. The host keeps its lowest 8 bits, so
`-၁` ends the run with 255. A start block with no result, or one that never sets
`ရလဒ်`, ends the run with 0. A run that stops on a trap ends with 1 whatever the
start block holds.

Inside, the start block follows the rules of a function: it has its own variables and
`ရလဒ်`, a return statement ends it, and it owns what it allocates, so a block it loses is
[ZK0708](ownership.md). It can change its two lists as it can each list parameter.

### Where a start block stands

- A program has one start block and no more, in the file given to `zakac run`: a second one
  is ZK0325, and a file with none is ZK0327. An imported file holds none (ZK0323).
- The top level of a file holds declarations alone: functions, structs, variables,
  constants and imports. A statement that runs there is ZK0326; it goes inside the start
  block or a function.
- A parameter that is no `စာသား စာရင်း`, a parameter the function can change, and a
  third parameter are ZK0440. A result that is no integer is ZK0441, a result kind among
  them: the start block cannot fail, so it holds no pass-on (ZK0445); see
  [Errors and results](error-handling.md#the-start-block).
- No call names the start block, it is not declared inside a block (ZK0214), and
  `အများသုံး` does not export it (ZK0229).

```zaka-error ZK0326
"ဟိုင်း" ဟု ပြသပါ။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0326]: a statement that runs goes inside the start block, not at the top level
```

```zaka-error ZK0325
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0325]: the program has a start block before this one
```

```zaka-error ZK0327
ကိန်းရှင် အမည် သည် "ဇကား" ဖြစ်ပါစေ။
```

```text
error[ZK0327]: the program has no start block, "ပရိုဂရမ်သည်"
```

```zaka-error ZK0440
ပရိုဂရမ်သည်
ကိန်းပြည့် အရေအတွက် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0440]: a parameter of the start block is a "စာသား စာရင်း", and there are two of them at most
```

```zaka-error ZK0441
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား ရလဒ် ကို ပြန်ပေးမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0441]: the result of the start block is "text", but it must be an integer
```

## Errors at a glance

| Code | Mistake |
| --- | --- |
| ZK0214 | an external function, a C function or the start block declared inside a block or a function |
| ZK0303 | a call to a function that is not declared |
| ZK0304 | two functions with one name |
| ZK0305 | one parameter name listed twice |
| ZK0311 | a return statement outside a function |
| ZK0312 | a value returned from a function without a result |
| ZK0313 | no value returned from a function with a result |
| ZK0325 | a second start block |
| ZK0326 | a statement that runs, at the top level of a file |
| ZK0327 | a program with no start block |
| ZK0314 | a wrong number of arguments |
| ZK0315 | the result of a function that has none, used as a value |
| ZK0401 | an argument or a returned value that does not fit its kind |
| ZK0424 | a list of borrows as a result |
| ZK0431 | an argument for a parameter the function can change, with no mark |
| ZK0432 | the mark on an argument for a parameter the function cannot change |
| ZK0433 | a place a call changes, in another argument of the same call |
| ZK0440 | a start block parameter that is no `စာသား စာရင်း`, or a third one |
| ZK0441 | a start block result that is no integer |
| ZK0337 | a C function whose name is no C name |
| ZK0338 | two declarations of one C function with other kinds |
| ZK0465 | a parameter or a result of a C function with a kind C cannot take |
| ZK0466 | the edit mark on a parameter of a C function |
| ZK0467 | `အပိုင်အနေဖြင့်` on a result that is no text from a C function |
| ZK0443 to ZK0449 | the failing-function mistakes of [Errors and results](error-handling.md#errors) |

The text of each message is in [Error messages](../reference/errors.md).
