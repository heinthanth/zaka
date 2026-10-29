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
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

လုပ်ဆောင်ချက် နှစ်ထပ် ကို ၇ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
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
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
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
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
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
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
ကိန်းရှင် သေး သည် ၃၂ ဘစ် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၇ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ပြ ကို သေး တို့ဖြင့် လုပ်ဆောင်ပါ။       // 32 bits widens to the full width
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
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်ချက် မေ့ ကို လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
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
| a result that is a list | ZK0424 |
| a value that does not fit the result kind | ZK0401 |

```zaka-error ZK0312
လုပ်ဆောင်ချက် ပြ သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
```

```text
error[ZK0312]: function "ပြ" returns nothing, but a value is returned
```

```zaka-error ZK0313
လုပ်ဆောင်ချက် တစ် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
```

```text
error[ZK0313]: function "တစ်" returns isize, but no value is returned
```

A list cannot be a result. The fix is a parameter that holds the address of the list
to fill; see [Lists](lists.md#passing-and-comparing).

```zaka-error ZK0424
လုပ်ဆောင်ချက် ဖန်တီး သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
```

```text
error[ZK0424]: a function cannot give back a list as its result
```

## Calls

A call has two forms. Both name the function after `လုပ်ဆောင်ချက်`, and both close the
argument list with `တို့ဖြင့်` ("with these"), one argument or several. A function with no
parameters has no argument list.

| Form | Written | Is |
| --- | --- | --- |
| statement | `လုပ်ဆောင်ချက် <name> ကို <arguments> တို့ဖြင့် လုပ်ဆောင်ပါ။` | a statement; a result, if there is one, is dropped |
| expression | `လုပ်ဆောင်ချက် <name> ကို <arguments> တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်` | a value |

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
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်ချက် အစီအစဉ် ကို ၁ + ၁၊ ၂ * ၅ တို့ဖြင့် လုပ်ဆောင်ပါ။
```

```text
၂
၁၀
```

A statement that begins with `လုပ်ဆောင်ချက်` is read as a call statement. A sum or other
expression that begins with a call is written with that call in parentheses; see
[Expressions](expressions.md).

```zaka-error ZK0314
လုပ်ဆောင်ချက် ပြ သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်ချက် ပြ ကို ၁၊ ၂ တို့ဖြင့် လုပ်ဆောင်ပါ။
```

```text
error[ZK0314]: function "ပြ" takes 1 arguments, 2 given
```

```zaka-error ZK0401
လုပ်ဆောင်ချက် ပြ သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်ချက် ပြ ကို မှန် တို့ဖြင့် လုပ်ဆောင်ပါ။
```

```text
error[ZK0401]: expected isize, found bool
```

## How values are passed

Each parameter is a new variable, and the call stores the argument in it. What that store
does depends on the kind of the parameter, as it does for a variable set from
another:

| Kind of the parameter | What the call does | Can the function change the caller's value? |
| --- | --- | --- |
| number, true/false value, character | duplicates the value | no |
| list | duplicates each cell | no; pass the address of the list to change it |
| `ငှားထားသော … လိပ်စာ` (a borrow) | duplicates the address; the caller keeps the owner | through the address |
| `… လိပ်စာ` (an owner) | **moves** the block to the function; the caller's variable holds `ဗလာ` afterwards | the function owns the block now |
| `… လိပ်စာအကြမ်း` (raw) | duplicates the address | through the address, and nothing is tracked |

### Values and lists are duplicated

A change to a parameter remains in the function:

```zaka
လုပ်ဆောင်ချက် တိုးပြ သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို က + ၁ ဟု ထားပါ။
က ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် ခ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် တိုးပြ ကို ခ တို့ဖြင့် လုပ်ဆောင်ပါ။
ခ ကို ပြသပါ။
```

```text
၆
၅
```

A list parameter is the same: the call duplicates the list one cell after another, and a large list
costs a duplicate on each call. A function that must change the caller's list, or that
should be spared the cost of a duplicate, takes the address of the list. The parameter kind is then
the list kind with `လိပ်စာ` after it, and `ငှားထားသော` in front:

```zaka
// the parameter holds the address of the caller's list
လုပ်ဆောင်ချက် နှစ်ဆတိုး သည်
ငှားထားသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ ကိန်းများ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၀ မှ ၂ အထိ န အနေဖြင့် ရေတွက်၍ ကိန်းများ ၏ အကွက် န ကို (ကိန်းများ ၏ အကွက် န) * ၂ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် မူရင်း သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် နှစ်ဆတိုး ကို မူရင်း ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်ပါ။
မူရင်း ကို ပြသပါ။
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
ကိန်းရှင် ယာယီ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် က ၏ တန်ဖိုး ဖြစ်ပါစေ။
က ၏ တန်ဖိုး ကို ခ ၏ တန်ဖိုး ဟု ထားပါ။
ခ ၏ တန်ဖိုး ကို ယာယီ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် ပထမ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် ဒုတိယ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် လဲလှယ် ကို ပထမ ၏ လိပ်စာ၊ ဒုတိယ ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်ပါ။
ပထမ ကို ပြသပါ။
ဒုတိယ ကို ပြသပါ။
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
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

လုပ်ဆောင်ချက် မကိန်းလား သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး ရလဒ် ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် မမှန် ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်ချက် စုံသလား ကို န - ၁ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

လုပ်ဆောင်ချက် စုံသလား ကို ၁၀ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
လုပ်ဆောင်ချက် စုံသလား ကို ၇ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
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
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

လုပ်ဆောင်ချက် အဆုံးမဲ့ ကို ၁ တို့ဖြင့် လုပ်ဆောင်ပါ။
```

```text
the program stopped: more than 4096 nested calls
```

See [When a program stops](../vm/traps.md) for the other stops.

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
| ZK0314 | a wrong number of arguments |
| ZK0315 | the result of a function that has none, used as a value |
| ZK0401 | an argument or a returned value that does not fit its kind |
| ZK0424 | a list as a result |

The text of each message is in [Error messages](../reference/errors.md).
