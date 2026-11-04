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

- A function is declared at the top level of the program: not inside a block, a loop, a
  conditional or another function (ZK0214).
- A call can come before the declaration it names. The order of declarations does not
  matter, and two functions can call each other.
- Each name is declared once. Zaka has no overloading: a second function with a name in
  use is ZK0304.
- A call to a name that no function has is ZK0303.

```zaka-error ZK0214
လုပ်ဆောင်ချက် ပြင်ပ သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် အတွင်း သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
error[ZK0214]: functions can be declared at the top level and nowhere else
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
number, `မမှန်` for a true/false value, `ဗလာ` for an address), and the call gives back
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

A call has three forms. The two word forms name the function after `လုပ်ဆောင်ချက်`, and both
close the argument list with `တို့ဖြင့်` ("with these"), one argument or several. A
function with no parameters has no argument list. The sign form writes the arguments in
parentheses after the name.

| Form | Written | Is |
| --- | --- | --- |
| statement | `လုပ်ဆောင်ချက် <name> ကို <arguments> တို့ဖြင့် လုပ်ဆောင်ပါ။` | a statement; a result, if there is one, is dropped |
| expression | `လုပ်ဆောင်ချက် <name> ကို <arguments> တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်` | a value |
| sign form | `<name>(<arguments>)` | a value, the same call as the expression form |

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
| result | none, or an integer kind: the exit code of the run |

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
  third parameter are ZK0440. A result that is no integer is ZK0441.
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
| ZK0214 | a function declared inside a block or another function |
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

The text of each message is in [Error messages](../reference/errors.md).
