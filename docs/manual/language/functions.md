---
title: Functions
description: What a function is, how to declare one and where it can go, with a map of the pages that state the rest of the rules on functions.
---

# Functions

A function is a named group of statements that the rest of the program can call. In Zaka
we write a function as a few Burmese sentences: the function signature gives its name, its
parameters and its result, the statements follow, and a block closer ends it. This page
shows how to declare a function and where we can declare one, and the pages in the table
below state the rest of the rules. For a gentle start, see
[Functions](../../learn/functions/README.md) in the Learn book.

```zaka
// square: one parameter, one result
နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက်ကို ၇ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၉
```

## The pages on functions

| Page | What it covers |
| --- | --- |
| [Function parameters](functions-parameters.md) | how to write parameters, their markers, mutable parameters (`inout`), writable parameters (`out`), and what a call does with each argument |
| [Results and returns](functions-results.md) | the result kind, return statements, the rule that each path returns, and lists and structs as results |
| [Function calls](functions-calls.md) | the statement form, the expression form, the sign form `f(x)`, calls with the arguments first, calls on a value with `၏`, and recursion |
| [Calls written as sentences](functions-sentences.md) | prepositions (argument labels), starting and ending phrases, and what this flexible grammar costs |
| [One name, several functions](functions-overloads.md) | overloads: functions that share a name, and how a call picks one |
| [External and C functions](functions-c.md) | functions that the VM serves, C functions from a lib, callbacks, variadic C functions, and handles to the structs of a C lib |
| [Functions C can call](functions-exports.md) | functions that `zakac build --emit lib` exports to C |
| [The program start block](functions-start-block.md) | the entry point (`main`): its arguments, its environment and its exit code |

Function values, closures and function kinds are on [Function values](function-values.md),
generic functions on [Generics](generics.md), comptime functions on
[Compile time](compile-time.md#comptime-functions), and unsafe functions on
[Unsafe Zaka](unsafe.md).

## Declaring a function

```text
<name> ဟူသော လုပ်ဆောင်ချက် သည်
<phrases> ကို သုံး၍                              (left out when there are none)
<parameters> လက်ခံပြီး                          (left out when there are none)
အောက်ပါတို့ကို လုပ်ဆောင်မည်။                       (no result)
  or
အောက်ပါတို့ကို လုပ်ဆောင်ကာ <kind> တစ်ခု ကို ပြန်ပေးမည်။   (with a result)
<statements>
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

In the function signature, we can use `<name> ဟူသော <kind> တစ်ခု ကို` to define a parameter
and `<kind> တစ်ခု ကို ပြန်ပေးမည်` to define a return value. A function with no parameters
leaves out the parameter list and `လက်ခံပြီး`, and a function with no result ends its
signature with `အောက်ပါတို့ကို လုပ်ဆောင်မည်။`. Each part of the function signature follows
these rules:

| Part | Rule |
| --- | --- |
| name | a name like other names; it comes first, then `ဟူသော`, as in the declaration of a [struct](structs.md) |
| words before `လုပ်ဆောင်ချက်` | we can write `အများသုံးနိုင်သော` to export the function ([Modules](modules.md)), `စိတ်မချရသော` to make it unsafe ([Unsafe Zaka](unsafe.md)) and `ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော` to make it a comptime function ([Compile time](compile-time.md#comptime-functions)), between `ဟူသော` and `လုပ်ဆောင်ချက်`, in either order |
| phrases | a starting phrase, an ending phrase or both: words that a call written as a sentence puts around its arguments ([Starting and ending phrases](functions-sentences.md#starting-and-ending-phrases)) |
| parameters | each one written `<name> ဟူသော <kind> တစ်<word> ကို`, then its markers ([Function parameters](functions-parameters.md)) |
| `လည်းကောင်း` | after each parameter when there are two or more; after a lone parameter we can write it or leave it out |
| result | a single data kind (type), then `တစ်` with the measure word (classifier) of that data kind, then `ကို ပြန်ပေးမည်`; the measure word is `ခု` for most data kinds, `လုံး` for `စာလုံး`, and the measure word of a struct, an enumeration or a kind alias for those ([Results and returns](functions-results.md)) |
| block closer | `လုပ်ဆောင်စရာများ` followed with `ပြီးပါပြီ`, `ပြီးပါပြီး`, `ပြီးဆုံးပါပြီ` or `ပြီးဆုံးပါပြီး` |

The line breaks in a function signature are plain white space, so we can also write the
whole signature on one line. Declaring a function runs nothing, and the statements inside
run each time the function is called.

### Where a function can be declared

- We can declare a function with statements at the top level of a file, or inside the
  statements of another function, the program start block, a block, a loop or a
  conditional. A function declared inside another one is a *nested function*: we can use
  it from its declaration to the end of its block, and it shares the local variables
  around it through their names. See
  [Nested functions](closures.md#nested-functions).
- External functions, C functions and the program start block are declared at the top
  level and nowhere else (ZK0214).
- At the top level, a call can come before the declaration of its function, so the order
  of declarations does not matter there and two functions can call each other.
- A file can declare several functions with one name when their parameters or their
  results differ ([One name, several functions](functions-overloads.md)). A second function
  with the same name, parameter kinds, prepositions and result is ZK0304. Nested functions of one scope have no
  overloads, so a second one with the same name is ZK0304 too.
- A call to a name that no function has is ZK0303.
- When we write `ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော` between `ဟူသော` and `လုပ်ဆောင်ချက်`
  on a top-level function with statements, the function becomes a *comptime function*
  (compile-time function). A call of it in a constant, a list count or another place that
  needs a value before the program runs is run while the compiler runs. See
  [Compile time](compile-time.md#comptime-functions).

```zaka-error ZK0214
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
abs ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
n ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0214]: external functions, C functions and the start block can be declared at the top level and nowhere else
```

## Errors at a glance

| Code | Mistake | Page |
| --- | --- | --- |
| ZK0201 | a parameter without `လည်းကောင်း`, markers out of order, half a marker, or a C name missing before `အမည်ဖြင့် ပြင်ပမှ ခေါ်ဆိုနိုင်သော` | [Function parameters](functions-parameters.md#writing-a-parameter) |
| ZK0204 | a statement end, `မည်` or `လုပ်ဆောင်မည်`, where a value is needed | [Function calls](functions-calls.md#arguments-first) |
| ZK0214 | an external function, a C function or the program start block declared inside a block or a function | [Declaring a function](#declaring-a-function) |
| ZK0241 | a preposition string with a word that a preposition cannot contain | [Prepositions](functions-sentences.md#prepositions) |
| ZK0249 | `၏` at the start of a preposition other than the last, or before an ending phrase | [Prepositions](functions-sentences.md#prepositions) |
| ZK0250 | a name spelled as a word of a preposition or a phrase (a warning) | [What the flexible grammar costs](functions-sentences.md#what-the-flexible-grammar-costs) |
| ZK0251 | a name spelled as a function name glued to a call end (a warning) | [What the flexible grammar costs](functions-sentences.md#what-the-flexible-grammar-costs) |
| ZK0258 | a phrase string with a word that a phrase cannot contain, or with `၏` | [Starting and ending phrases](functions-sentences.md#starting-and-ending-phrases) |
| ZK0260 | words before a function that cut into its arguments in two readings | [Prepositions](functions-sentences.md#prepositions) |
| ZK0303 | a call to a function that is not declared | [Declaring a function](#declaring-a-function) |
| ZK0304 | two functions with one name and the same parameter kinds, prepositions, phrases and result | [One name, several functions](functions-overloads.md) |
| ZK0305 | one parameter name listed twice | [Function parameters](functions-parameters.md#writing-a-parameter) |
| ZK0306 | a constant as the argument of a mutable parameter | [Function parameters](functions-parameters.md#mutable-parameters) |
| ZK0311 | a return statement outside a function | [Results and returns](functions-results.md#return-statements) |
| ZK0312 | a value returned from a function without a result | [Results and returns](functions-results.md#return-statements) |
| ZK0313 | no value returned from a function with a result | [Results and returns](functions-results.md#return-statements) |
| ZK0314 | a wrong number of arguments | [Function calls](functions-calls.md#arguments) |
| ZK0315 | the result of a function that has none, used as a value | [Function calls](functions-calls.md#arguments) |
| ZK0325 | a second program start block | [The program start block](functions-start-block.md#where-the-program-start-block-goes) |
| ZK0326 | a statement that runs, at the top level of a file | [The program start block](functions-start-block.md#where-the-program-start-block-goes) |
| ZK0327 | a program with no program start block | [The program start block](functions-start-block.md#where-the-program-start-block-goes) |
| ZK0331 | an argument with a preposition that its parameter does not have | [Prepositions](functions-sentences.md#prepositions) |
| ZK0332 | a call that no function of the name fits | [One name, several functions](functions-overloads.md#how-a-call-picks-a-function) |
| ZK0333 | a call that two or more functions of the name fit | [One name, several functions](functions-overloads.md#how-a-call-picks-a-function) |
| ZK0334 | an external function that the VM does not serve | [External and C functions](functions-c.md#external-functions) |
| ZK0337 | a C function or a C name that is no C name | [External and C functions](functions-c.md#c-functions) |
| ZK0338 | two declarations of one C function with other data kinds | [External and C functions](functions-c.md#c-functions) |
| ZK0339 | a function with a result whose block can reach its end with no return | [Results and returns](functions-results.md#each-path-returns) |
| ZK0352 | a call written as a sentence whose phrases are not those of the function signature | [Starting and ending phrases](functions-sentences.md#starting-and-ending-phrases) |
| ZK0401 | an argument or a returned value that does not fit its data kind | [Function calls](functions-calls.md#arguments) |
| ZK0410 | an argument of a mutable or writable parameter that is no place | [Function parameters](functions-parameters.md#mutable-parameters) |
| ZK0433 | a place that a call changes, in another argument of the same call | [Function parameters](functions-parameters.md#mutable-parameters) |
| ZK0440 | a program start block parameter that is no `စာသား စာရင်း`, or a third one | [The program start block](functions-start-block.md#where-the-program-start-block-goes) |
| ZK0441 | a program start block result that is no integer | [The program start block](functions-start-block.md#where-the-program-start-block-goes) |
| ZK0443 to ZK0449 | the mistakes of functions that can fail | [Errors and results](error-handling.md#errors) |
| ZK0464 | a C function used as a function value | [External and C functions](functions-c.md#c-functions) |
| ZK0465 | a parameter or a result of a C function with a data kind that C cannot use | [External and C functions](functions-c.md#c-functions) |
| ZK0466 | `ဒေတာ ပြောင်းလဲရန်` or `ဒေတာ ထည့်ပေးရန်` on a parameter of a C function that is no list of numbers and no struct that crosses | [External and C functions](functions-c.md#lists-of-numbers) |
| ZK0467 | `အပိုင်အနေဖြင့်` on a C result that is no optional of text | [External and C functions](functions-c.md#text-as-a-result) |
| ZK0468 | a call of an unsafe function, a C one included, outside unsafe code | [External and C functions](functions-c.md#unsafe-c-functions) |
| ZK0470 | `အပိုင်အနေဖြင့်` on a parameter or a result that has no pointer in it | [Function parameters](functions-parameters.md#writing-a-parameter) |
| ZK0472 | a C function not declared `စိတ်မချရသော` | [External and C functions](functions-c.md#unsafe-c-functions) |
| ZK0473 | a value of a data kind from outside the program that the program makes, duplicates or prints | [External and C functions](functions-c.md#data-kinds-from-outside-the-program) |
| ZK0474 | a write to a field of a data kind from outside the program, outside unsafe code or to a text or handle field | [External and C functions](functions-c.md#data-kinds-from-outside-the-program) |
| ZK0479 | `ကနဦးတန်ဖိုး` for a handle | [External and C functions](functions-c.md#data-kinds-from-outside-the-program) |
| ZK0490 | a function C can call that is generic, nested, has no statements, or uses `အပိုင်အနေဖြင့်` | [Functions C can call](functions-exports.md#where-and-how-it-is-declared) |
| ZK0491 | the C name of a function C can call that is `main`, starts with `zk_` or `ZK_`, or is taken | [Functions C can call](functions-exports.md#where-and-how-it-is-declared) |
| ZK0492 | a struct without a C name, passed to or from a function C can call | [Functions C can call](functions-exports.md#structs-and-their-c-names) |
| ZK0722 | an argument of a mutable parameter that can be unset, or a writable parameter read before the function sets it | [Function parameters](functions-parameters.md#mutable-parameters) |
| ZK0725 | a return that a path reaches with a writable parameter unset | [Function parameters](functions-parameters.md#writable-parameters) |

The text of each message is in [Error messages](../reference/errors.md).
