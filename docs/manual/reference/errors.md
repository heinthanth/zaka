---
title: Error messages
description: How to read a compiler message, and what the common codes mean.
---

# Error messages

When something is wrong, `zakac` prints a message instead of running the program. Here
is one such message:

```console
$ zakac run program.zaka
error[ZK0401]: expected isize, found f64
 --> program.zaka:2:10
  |
2 | အသက် ကို ၂.၅ ဟု ထားမည်။
  |       ^^^
  = help: convert the value with "ကို <kind> တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်"
```

| Part | Means |
| --- | --- |
| `error` | the program will not run. A `warning` does not stop it. |
| `[ZK0401]` | the code; look it up in the tables below |
| `expected isize, found f64` | what went wrong |
| `program.zaka:2:10` | file, line 2, character 10 |
| `^^^` | points at the problem in the line |
| `help:` | a suggestion for fixing it (some messages have none) |

Fix the first error first. A single mistake can confuse the compiler into reporting
more errors after it.

When a program stops *while running* (for example, on a division whose zero appears at
run time), the message starts with `the program stopped:` and has no code.

## Short names for kinds

Messages use short names for kinds. A short name starts with the base kind, then adds one
mark for each word around it, in the order of the Burmese words: `*` for `လိပ်စာ`, `[n]`
for `စာရင်း`, and `@` after the name of a shared object kind.

| Short name | Zaka kind |
| --- | --- |
| `isize` | `ကိန်းပြည့်` |
| `i8`, `i16`, `i32`, `i64` | `၈ ဘစ် ကိန်းပြည့်` ... `၆၄ ဘစ် ကိန်းပြည့်` |
| `usize` | `လက္ခဏာမဲ့ ကိန်းပြည့်` |
| `u8`, `u16`, `u32`, `u64` | `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်` ... |
| `f32`, `f64` | `၃၂ ဘစ် ဒဿမကိန်း` ... ; `f64` is plain `ဒဿမကိန်း` |
| `char` | `စာလုံး` |
| `bool` | `မှန်မှားတန်ဖိုး` |
| `isize*`, `u8*`, `isize**`, ... | `ကိန်းပြည့် လိပ်စာ`, ... ; one more `*` for each `၏ လိပ်စာ` after it |
| `ptr` | `လိပ်စာ` (the opaque pointer) |
| `isize[3]`, `isize[3][2]` | `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း`, ... ; one `[n]` for each `စာရင်း`, in the order of the words: `isize[3][2]` is two lists of three |
| `isize[]` | `ကိန်းပြည့် စာရင်း`, a plain list: its count comes when the program runs |
| `isize[3]*` | `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ`, the address of a whole list |
| `text` | `စာသား` |
| `အကောင့်@`, `အကောင့်@[]` | a [shared object](../language/objects.md) of the kind `အကောင့်`, a list of them |
| `null` | the kind of the null address, `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` |
| `fn(isize, isize) -> bool`, `fn(text(ကို))`, `fn()` | a [function kind](../language/function-values.md): its parameters, `mut ` before one marked `ဒေတာ ပြောင်းလဲရန်` and its mark after it, then `->` and the result; `(fn(isize))[]` is a list of them, a function kind inside another name in parentheses |
| `isize result or text error` | `ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ်`, a [result kind](../language/error-handling.md) |
| `success or text error`, `isize result or failure` | `အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ်`, `ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ်`: `success` and `failure` stand for a side with no value |
| `optional isize` | `ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`, an [optional kind](../language/optionals.md) |

## Codes

The first two digits tell which stage found the problem. [Diagnostics](../compiler/diagnostics.md)
explains how a message is set out and when a message stops the compiler.

### ZK01: reading the characters

| Code | Problem | See |
| --- | --- | --- |
| ZK0101 | a character Zaka does not use, such as `;` or `?` | |
| ZK0102 | a malformed number, such as `၂.၅e3` or `၁၈ဘစ်` | [Integers](../../learn/values/integers.md) |
| ZK0103 | Burmese and Western digits in one number | [Integers](../../learn/values/integers.md) |
| ZK0104 | text with no closing `"` | [Text](../../learn/text/README.md) |
| ZK0105 | a character literal with no closing `'` on its line | [Characters](../../learn/values/characters.md) |
| ZK0106 | unknown `\` escape, a lone brace, or an unknown `{...}` in a character | [Characters](../../learn/values/characters.md) |
| ZK0107 | a character literal with more than one character | [Characters](../../learn/values/characters.md) |
| ZK0108 | a name in braces with no closing brace on its line | [Names](../../learn/variables/names.md) |
| ZK0109 | a name in braces with nothing between them | [Names](../../learn/variables/names.md) |
| ZK0110 | a statement verb that ends in `ပါ`; statements end in `မည်` | [Sentences](../../learn/sentences/README.md) |

### ZK02: reading the sentences

| Code | Problem | See |
| --- | --- | --- |
| ZK0201 | something unexpected, often a missing `။` | [Hello, world](../../learn/hello/README.md) |
| ZK0202 | a value is missing where one was expected | |
| ZK0203 | the statement does not end in a verb such as `ပြသမည်` | [Grouping and order](../../learn/operators/grouping.md) |
| ZK0204 | a logical expression printed without parentheses | [Logic](../../learn/operators/logic.md) |
| ZK0205 | a verb is missing one of its parts | [Particles](../../learn/sentences/particles.md) |
| ZK0206 | wrong or missing particle, often a name joined to `ကို` | [Spaces between words](../../learn/sentences/spacing.md) |
| ZK0207 | two particles in a row after one operand | [Particles](../../learn/sentences/particles.md) |
| ZK0208 | operands left waiting for a verb at the end of a clause | [Particles](../../learn/sentences/particles.md) |
| ZK0209 | operands left over before a statement verb | [Particles](../../learn/sentences/particles.md) |
| ZK0210 | a comparison inside a comparison | [Comparison](../../learn/operators/comparison.md) |
| ZK0211 | `သို့မဟုတ်ပါက` without an `အကယ်၍` before it | [If and else](../../learn/control-flow/if-else.md) |
| ZK0212 | the `ပြီးလျှင်` update outside a generic loop | [Generic loops](../../learn/control-flow/generic-loops.md#the-update-goes-last) |
| ZK0213 | the `ပြီးလျှင်` update is not last | [Generic loops](../../learn/control-flow/generic-loops.md) |
| ZK0214 | an external function, a C function or the start block declared inside a block or function | [Functions](../language/functions.md), [Function values](../language/function-values.md) |
| ZK0215 | an `if` as the one-statement inner block of an `if` | [If and else](../../learn/control-flow/if-else.md) |
| ZK0216 | a bit count that the kind does not have, such as `၁၀ ဘစ် ကိန်းပြည့်` | [Integers](../../learn/values/integers.md#building-an-integer-kind) |
| ZK0217 | a sign word before a kind that is not an integer | [Integers](../../learn/values/integers.md#building-an-integer-kind) |
| ZK0218 | a number base that is not a whole number from 2 to 36 | [Integers](../../learn/values/integers.md#bases) |
| ZK0219 | a digit too big for the number base | [Integers](../../learn/values/integers.md) |
| ZK0220 | a number too big for its kind | [Integers](../../learn/values/integers.md) |
| ZK0221 | a fraction or exponent in a literal that is given an integer kind | [Integers](../../learn/values/integers.md#the-order-of-the-parts) |
| ZK0222 | a kind after a literal that does not fit it: other than a plain number kind after a number, other than a list kind after a list | [Integers](../../learn/values/integers.md#the-order-of-the-parts) |
| ZK0225 | a generic loop that starts with `ကိန်းသေ` | [Generic loops](../../learn/control-flow/generic-loops.md#the-variable-belongs-to-the-loop) |
| ZK0226 | a block closer with no block to close | [Blocks and scope](../../learn/variables/scope.md) |
| ZK0227 | a list count that is not a whole number above zero, or names no constant that works out | [Static data and counted lists](../../learn/forms/counted-lists.md#counted-lists) |
| ZK0228 | a struct declared inside a block | [Struct mistakes](../../learn/structs/mistakes.md#a-struct-declared-inside-a-block) |
| ZK0229 | `အများသုံး` before something other than a top-level declaration | [Module mistakes](../../learn/modules/mistakes.md#the-export-mark-in-the-wrong-place) |
| ZK0230 | an import after a declaration or statement, or inside a block | [Module mistakes](../../learn/modules/mistakes.md#an-import-below-a-statement) |
| ZK0231 | files that import each other | [Module mistakes](../../learn/modules/mistakes.md#files-that-import-each-other) |
| ZK0232 | an import in a program that was not read from a file | [Module mistakes](../../learn/modules/mistakes.md#imports-outside-a-file) |
| ZK0234 | a kind with no `တစ်<classifier>` after it, in a declaration, a parameter, a field, a result, a function kind, a conversion, a size, a default arm or after a literal | [Measure words](../language/kinds.md#measure-words) |
| ZK0235 | an enumeration declared inside a block | [Enumeration mistakes](../../learn/enums/mistakes.md#an-enumeration-declared-inside-a-block) |
| ZK0236 | a switch arm after the default arm | [Enumeration mistakes](../../learn/enums/mistakes.md#an-arm-after-the-default-arm) |
| ZK0237 | a header that gives some parameters a preposition and not the others | [Functions](../language/functions.md) |
| ZK0238 | an import from `စံကား/` that names no module of the stdlib | [Input and output](../language/io.md) |
| ZK0239 | a function kind with parameters inside a list of parameters or fields, not in parentheses | [Function values](../language/function-values.md) |
| ZK0240 | a kind alias declared inside a block | [Kind aliases](../language/aliases.md#errors) |
| ZK0241 | a parameter preposition string, after `ဝိဘတ်`, that holds a word a preposition cannot hold, or a first preposition that starts with `ကို သုံး၍` | [Functions](../language/functions.md) |
| ZK0243 | a parameter of a function kind marked `ဒေတာ ထည့်ပေးရန်` | [Function values](../language/function-values.md) |
| ZK0244 | nesting of parentheses, operands, calls, lists or blocks more than 128 levels deep | [The nesting limit](../internals/parser.md#the-nesting-limit) |
| ZK0245 | a list in words of two or more items without `နှင့်` before the last item, or with a `၊` after it | [Lists in words](../language/words.md#lists-in-words) |
| ZK0246 | the word that closes a list in words does not fit its count: `တို့` after one item, or none after two or more | [Lists in words](../language/words.md#lists-in-words) |
| ZK0247 | a trait or a trait block inside a block | [Generics and traits](../language/generics.md) |
| ZK0248 | a note in a notes block on a name that is no kind parameter of its declaration | [Generics and traits](../language/generics.md) |
| ZK0249 | a preposition that starts with `၏` on a parameter other than the last | [Functions](../language/functions.md#prepositions) |
| ZK0250 | warning: a declared name that is a word of the preposition of a function the file sees | [Functions](../language/functions.md#what-the-flexible-grammar-costs) |
| ZK0251 | warning: a declared name that reads as a function the file sees with a glued call end | [Functions](../language/functions.md#what-the-flexible-grammar-costs) |

### ZK03: names

| Code | Problem | See |
| --- | --- | --- |
| ZK0301 | a name that was never declared, or is out of scope | [Blocks and scope](../../learn/variables/scope.md) |
| ZK0302 | the same name declared twice in one block | [Names](../../learn/variables/names.md) |
| ZK0303 | calling a function that does not exist | [Functions](../../learn/functions/README.md) |
| ZK0304 | two functions with one name and the same parameter kinds and prepositions, or a function of a name the file imports | [Functions](../language/functions.md) |
| ZK0305 | two parameters of one function with one name | [Parameters](../../learn/functions/parameters.md) |
| ZK0306 | changing a constant | [Constants](../../learn/variables/constants.md) |
| ZK0307 | changing a counted loop's counter | [Counted loops](../../learn/control-flow/counted-loops.md) |
| ZK0308 | a constant without a value | [Constants](../../learn/variables/constants.md) |
| ZK0309 | a break outside a loop | [Break and continue](../../learn/control-flow/break-continue.md) |
| ZK0310 | a continue outside a loop | [Break and continue](../../learn/control-flow/break-continue.md) |
| ZK0311 | `ပြန်ပေးမည်` outside a function | [Functions](../../learn/functions/README.md) |
| ZK0312 | returning a value from a function without a result | [Results](../../learn/functions/results.md) |
| ZK0313 | returning without a value from a function with a result | [Results](../../learn/functions/results.md) |
| ZK0314 | the wrong number of arguments | [Parameters](../../learn/functions/parameters.md) |
| ZK0315 | using a function without a result as a value | [Results](../../learn/functions/results.md) |
| ZK0316 | a struct, enumeration or kind alias name declared twice | [Struct mistakes](../../learn/structs/mistakes.md#one-name-for-two-structs) |
| ZK0317 | a field name used twice in one struct | [Struct mistakes](../../learn/structs/mistakes.md#one-name-for-two-fields) |
| ZK0318 | a name in a kind that no struct or enumeration declaration gives | [Struct mistakes](../../learn/structs/mistakes.md#a-name-that-is-no-struct) |
| ZK0319 | a struct or enumeration name used for a variable, parameter, field or function | [Struct mistakes](../../learn/structs/mistakes.md#a-struct-name-that-is-also-a-value-name) |
| ZK0320 | a field the struct does not have | [Struct mistakes](../../learn/structs/mistakes.md#a-field-the-struct-does-not-have) |
| ZK0321 | a field set twice in a struct literal | [Struct mistakes](../../learn/structs/mistakes.md#one-field-set-twice) |
| ZK0322 | an import of a name the other file does not export | [Module mistakes](../../learn/modules/mistakes.md#a-name-the-file-does-not-export) |
| ZK0323 | a statement in an imported file | [Module mistakes](../../learn/modules/mistakes.md#a-statement-in-an-imported-file) |
| ZK0324 | an exported declaration whose kind names a private struct | [Module mistakes](../../learn/modules/mistakes.md#a-private-struct-in-an-export) |
| ZK0325 | a second start block | [Functions](../language/functions.md) |
| ZK0326 | a statement that runs, at the top level of a file | [Functions](../language/functions.md) |
| ZK0327 | a program with no start block | [Functions](../language/functions.md) |
| ZK0328 | a member name used twice in one enumeration | [Enumeration mistakes](../../learn/enums/mistakes.md#one-member-name-twice) |
| ZK0329 | a member the enumeration does not have | [Enumeration mistakes](../../learn/enums/mistakes.md#a-member-the-enumeration-does-not-have) |
| ZK0330 | a name before `ထဲမှ` that is no enumeration with `များ` | [Enumeration mistakes](../../learn/enums/mistakes.md#a-member-written-without-the-plural-mark) |
| ZK0331 | an argument of a preposition call with a preposition other than its parameter's | [Functions](../language/functions.md) |
| ZK0332 | a call that no function of its name fits | [Functions](../language/functions.md) |
| ZK0333 | a call that more than one function of its name fits | [Functions](../language/functions.md) |
| ZK0334 | an external function that is none of the functions the VM serves | [Functions](../language/functions.md) |
| ZK0335 | a call whose name finds both a declared function and a value of a function kind | [Function values](../language/function-values.md) |
| ZK0336 | a chain of kind aliases that comes back to its start | [Kind aliases](../language/aliases.md#errors) |
| ZK0337 | a C function whose name is no C name | [Functions](../language/functions.md) |
| ZK0338 | a C function declared twice with other parameter or result kinds | [Functions](../language/functions.md) |
| ZK0339 | a function with a result whose block can reach its end with no return | [Results](../../learn/functions/results.md) |
| ZK0340 | a call or a kind that gives a generic another number of kinds than it declares, or kinds for a function that is not generic | [Generics and traits](../language/generics.md) |
| ZK0341 | a call of a generic function, or a value of a generic struct, whose kinds its values do not show | [Generics and traits](../language/generics.md) |
| ZK0342 | instances of a generic function or struct that each ask for a deeper one, past the limit | [Generics and traits](../language/generics.md) |
| ZK0343 | a kind given to a generic that lacks a trait its kind parameter needs | [Generics and traits](../language/generics.md) |
| ZK0344 | a constraint or a trait block that names no trait | [Generics and traits](../language/generics.md) |
| ZK0345 | a trait block that lacks a function of its trait | [Generics and traits](../language/generics.md) |
| ZK0346 | a function of a trait block that is not one of its trait | [Generics and traits](../language/generics.md) |
| ZK0347 | two functions that give a kind one function of a trait | [Generics and traits](../language/generics.md) |
| ZK0348 | a trait block outside the file of its kind and of its trait | [Generics and traits](../language/generics.md) |
| ZK0349 | the compiler mark `ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော` outside the modules of the stdlib | [The prelude](../language/prelude.md) |
| ZK0350 | warning: a function with the name, the kinds and the prepositions of a function the prelude gives, which then takes over each sentence of it | [The prelude](../language/prelude.md#verbs-on-structs) |

### ZK04: kinds

| Code | Problem | See |
| --- | --- | --- |
| ZK0401 | a value of the wrong kind for a variable or parameter | [Converting between kinds](../../learn/operators/conversion.md) |
| ZK0402 | a condition that is not true/false | [If and else](../../learn/control-flow/if-else.md) |
| ZK0403 | an operator used on the wrong kind, such as `မှန် + ၁` | [Arithmetic](../../learn/operators/arithmetic.md) |
| ZK0404 | two kinds that cannot be combined without a conversion | [Arithmetic](../../learn/operators/arithmetic.md#mixing-kinds) |
| ZK0405 | a counted-loop bound or step that is not an integer | [Counted loops](../../learn/control-flow/counted-loops.md#rules) |
| ZK0406 | a number too big for the variable's kind | [Integers](../../learn/values/integers.md) |
| ZK0407 | a shift wider than the kind's bit count | [Bitwise operators](../../learn/operators/bitwise.md) |
| ZK0408 | a conversion that is not allowed | [Converting between kinds](../../learn/operators/conversion.md) |
| ZK0409 | a counted-loop step that is zero or less | [Counted loops](../../learn/control-flow/counted-loops.md#rules) |
| ZK0410 | an invalid assignment target, or an argument for a `ဒေတာ ပြောင်းလဲရန်` parameter that is no place: not a variable, a field, a cell or a read through a pointer | [Changing a variable](../../learn/variables/assignment.md) |
| ZK0411 | no address: taking the address of something that is not a variable, a field, a cell or a read through a pointer | [Unsafe Zaka](../language/unsafe.md) |
| ZK0412 | taking the address of a fixed name: a constant or a counted-loop variable | [Unsafe Zaka](../language/unsafe.md) |
| ZK0413 | not a pointer: reading through a value that has no pointer kind | [Unsafe Zaka](../language/unsafe.md) |
| ZK0414 | an opaque pointer (`လိပ်စာ`) read through or offset before a conversion | [Unsafe Zaka](../language/unsafe.md) |
| ZK0415 | an operator not for pointers, such as `*` or `/` on an address | [Unsafe Zaka](../language/unsafe.md) |
| ZK0416 | pointer kinds differ: comparing or subtracting pointers to different kinds | [Unsafe Zaka](../language/unsafe.md) |
| ZK0417 | a heap statement, such as a resize, a duplicate or a fill, that needs a pointer but was given another kind | [Unsafe Zaka](../language/unsafe.md) |
| ZK0419 | a list literal with more or fewer cells than its kind | [Static data and counted lists](../../learn/forms/counted-lists.md#a-literal-with-the-wrong-count) |
| ZK0420 | `စာရင်းအလွတ်` where no list kind is known | [List mistakes](../../learn/lists/mistakes.md#a-blank-list-with-no-kind) |
| ZK0422 | cells of a value that is no list | [List mistakes](../../learn/lists/mistakes.md#cells-of-a-value-that-is-no-list) |
| ZK0423 | a cell number that is not an integer | [Lists](../../learn/lists/README.md#cell-numbers) |
| ZK0425 | a list kind too large for the VM | [Static data and counted lists](../../learn/forms/counted-lists.md#the-size-of-a-counted-list) |
| ZK0426 | a field of a value that is no struct | [Struct mistakes](../../learn/structs/mistakes.md#a-field-of-a-value-that-is-no-struct) |
| ZK0427 | a struct that holds itself | [Struct mistakes](../../learn/structs/mistakes.md#a-struct-that-holds-itself) |
| ZK0429 | a write to a cell (a u8) of text | [Text](../language/text.md) |
| ZK0430 | a declaration with no kind whose value is the null address | [Statements](../language/statements.md) |
| ZK0433 | one place given to two `ဒေတာ ပြောင်းလဲရန်` parameters of one call, or a place a call changes in another argument of the same call | [Functions](../language/functions.md) |
| ZK0434 | an each loop over a value it cannot walk: no list for an each-cell loop, or no text for an each-character loop, such as a ဇယား | [Statements](../language/statements.md#loops) |
| ZK0436 | `[]` or `စာရင်းအလွတ်` given to a counted list | [Static data and counted lists](../../learn/forms/counted-lists.md#a-list-with-no-cells-in-a-counted-list) |
| ZK0437 | an add or a remove on a counted list, or `နောက်ဆုံးအကွက်` on text | [Static data and counted lists](../../learn/forms/counted-lists.md#adding-to-a-counted-list) |
| ZK0439 | `၏ အရှေ့တွင်` after a value that is no cell of a list and no character of text | [Lists](../language/lists.md) |
| ZK0440 | a start block parameter that is no `စာသား စာရင်း`, a parameter to change, or a third one | [Functions](../language/functions.md) |
| ZK0441 | a start block result that is no integer, such as a result kind | [Functions](../language/functions.md) |
| ZK0442 | a raw duplicate or a fill of cells that hold counted blocks: text, plain lists, objects or function values | [Writing and filling cells](../language/unsafe.md#writing-and-filling-cells) |
| ZK0443 | a call statement to a function that can fail, with no pass-on | [Errors and results](../language/error-handling.md) |
| ZK0444 | a pass-on for a function that cannot fail | [Errors and results](../language/error-handling.md) |
| ZK0445 | a pass-on in a function that cannot fail, in the start block, or in top-level statements | [Errors and results](../language/error-handling.md) |
| ZK0446 | a pass-on whose error kind differs from the error kind of the function around the call | [Errors and results](../language/error-handling.md) |
| ZK0447 | an error value given back from a function that cannot fail | [Errors and results](../language/error-handling.md) |
| ZK0448 | a test or a read of a value that is no result and no optional | [Errors and results](../language/error-handling.md) |
| ZK0449 | a read of the good result, `ထဲမှ ဒေတာ` or `လုပ်ဆောင်သော ရလဒ်ကောင်း`, of a result with no good value | [Errors and results](../language/error-handling.md) |
| ZK0450 | a read of the error value of a result whose error side holds no value | [Errors and results](../language/error-handling.md) |
| ZK0451 | a failure with no value in a function whose error side names a kind | [Errors and results](../language/error-handling.md) |
| ZK0454 | a read of the part of a result, or the data of an optional, that a test has ruled out | [Errors and results](../language/error-handling.md) |
| ZK0456 | a member value that does not work out when the program compiles | [Enumeration mistakes](../../learn/enums/mistakes.md#a-member-value-that-is-not-a-constant) |
| ZK0457 | two members of one enumeration with one value | [Enumeration mistakes](../../learn/enums/mistakes.md#two-members-with-one-value) |
| ZK0458 | a switch over an enumeration that names neither each member nor a default arm | [Enumeration mistakes](../../learn/enums/mistakes.md#a-switch-that-misses-a-member) |
| ZK0459 | a default arm whose kind is not the kind of the subject | [Enumeration mistakes](../../learn/enums/mistakes.md#a-default-arm-of-the-wrong-kind) |
| ZK0460 | two switch cases with one value | [Enumeration mistakes](../../learn/enums/mistakes.md#one-case-twice) |
| ZK0461 | a member with no value whose value cannot count on: its kind is no integer, or the value passes the largest of the kind | [Enumeration mistakes](../../learn/enums/mistakes.md#a-member-with-no-value-that-cannot-count-on) |
| ZK0462 | a call through a name or a place whose kind is no function kind; a leave for C of a value that is no function value | [Function values](../language/function-values.md) |
| ZK0463 | the value of a name of several functions, where no single one fits the kind the value goes to | [Function values](../language/function-values.md) |
| ZK0464 | the value of a function that comes from outside the program | [Function values](../language/function-values.md) |
| ZK0465 | a parameter or the result of a C function with a kind C cannot take: a bare text result, a function kind with a struct, text or `ဒေတာ ပြောင်းလဲရန်` in it, a list of other than numbers | [Functions](../language/functions.md) |
| ZK0466 | `ဒေတာ ပြောင်းလဲရန်` or `ဒေတာ ထည့်ပေးရန်` on a parameter of a C function that is no list of numbers and no struct that crosses | [Functions](../language/functions.md) |
| ZK0467 | `အပိုင်အနေဖြင့်` on a C function's result that is no text | [Functions](../language/functions.md) |
| ZK0468 | reading or writing through a pointer, pointer arithmetic, an address, a heap statement, a pointer conversion or a call of an unsafe function, outside unsafe code | [Unsafe Zaka](../language/unsafe.md) |
| ZK0469 | an unsafe function (`စိတ်မချရသော`) named as a function value | [Unsafe Zaka](../language/unsafe.md) |
| ZK0470 | `အပိုင်အနေဖြင့်` on a parameter or result that is no pointer (a handle parameter of a C function is allowed) | [Unsafe Zaka](../language/unsafe.md) |
| ZK0471 | `စိတ်မချရသော`, on a function or a block, under `--no-unsafe` | [Unsafe Zaka](../language/unsafe.md) |
| ZK0472 | a C function whose kinds hold a pointer, declared without `စိတ်မချရသော` | [Functions](../language/functions.md), [Unsafe Zaka](../language/unsafe.md) |
| ZK0473 | a value of a kind from outside the program, `ပြင်ပမှ လာသော ဒေတာအစုအဝေး`, that the program makes, duplicates or prints | [Functions](../language/functions.md#c-functions) |
| ZK0474 | a write to a field of a kind from outside the program outside unsafe code, or to a text or a handle field: the struct belongs to C | [Functions](../language/functions.md#c-functions) |
| ZK0475 | a test word that does not fit the kind: `ဒေတာ` on a result, or `ရလဒ်ကောင်း` or `အမှားအယွင်း` on an optional | [Optionals](../language/optionals.md) |
| ZK0476 | `<value> ဟူသော အမှားအယွင်း` where no result kind with an error side is wanted | [Error handling](../language/error-handling.md) |
| ZK0477 | `အလွတ်` where no optional kind is wanted | [Optionals](../language/optionals.md) |
| ZK0478 | `ကနဦးတန်ဖိုး` where no kind can be worked out | [Initial values](../language/kinds.md#initial-values) |
| ZK0479 | `ကနဦးတန်ဖိုး` for a kind that has no initial value: a function kind, a result, a handle, an enumeration with no marked member, or a kind that holds one | [Initial values](../language/kinds.md#initial-values), [Enumeration mistakes](../../learn/enums/mistakes.md#no-member-marked-as-the-initial-one) |
| ZK0480 | two members of one enumeration marked `ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်` | [The initial member](../language/enums.md#the-initial-member), [Enumeration mistakes](../../learn/enums/mistakes.md#two-members-marked-as-the-initial-one) |
| ZK0481 | a kind no program can build: its fields lead back to it with no optional or plain list on the loop | [Kinds no program can build](../language/objects.md#kinds-no-program-can-build) |
| ZK0482 | a struct literal that leaves out a field | [Struct literals](../language/structs.md#struct-literals), [Struct mistakes](../../learn/structs/mistakes.md#a-field-left-out) |
| ZK0483 | a global of a module declared with no value | [Globals of a module](../language/modules.md#globals-of-a-module), [Module mistakes](../../learn/modules/mistakes.md#a-variable-of-a-module-with-no-value) |
| ZK0484 | a row read, `အညွှန်းအရေအတွက်`, `အညွှန်းတိုင်း` or `အတန်းတိုင်း` on a value that is no ဇယား | [Tables](../language/tables.md#reading-a-row) |
| ZK0485 | a ဇယား kind whose index kind holds a part that cannot be hashed and compared as a whole value: an object, a function value, a pointer, an optional, a result or a ဇယား | [Index kinds](../language/tables.md#index-kinds) |
| ZK0486 | `ဇယားအလွတ်` or `[:]` with no ဇယား kind from its place | [Tables](../language/tables.md#literals) |
| ZK0487 | a warning: a ဇယား kind whose index kind holds a decimal number, which rounding can split into two rows | [Index kinds](../language/tables.md#index-kinds) |
| ZK0488 | `ကို ဖျက်မည်` after a value that is no row of a ဇယား | [Tables](../language/tables.md#setting-and-removing-rows) |

### ZK05: natural Zaka (warnings)

These are warnings: the program runs. With `--strict-particles` each one is an error.
See [Natural Zaka](../../learn/natural-zaka/README.md) and [Natural wording](../language/naturalness.md).

| Code | Problem | See |
| --- | --- | --- |
| ZK0501 | a "not" verb before `သို့မဟုတ်` ends in `သည်` instead of `ပါ` | [Natural Zaka](../../learn/natural-zaka/README.md#the-rules) |
| ZK0502 | a particle between a verb and an attached `လျှင်` or `နေသရွေ့` | [Natural Zaka](../../learn/natural-zaka/README.md#the-rules) |
| ZK0503 | `မဟုတ်` with `သည်` | [Natural Zaka](../../learn/natural-zaka/README.md#the-rules) |
| ZK0504 | `ပါ` on a verb before `ဖြစ်၍` | [Natural Zaka](../../learn/natural-zaka/README.md#the-rules) |
| ZK0505 | a sentence or text printed with `ကို` instead of `ဟု` | [Natural Zaka](../../learn/natural-zaka/README.md#the-rules) |
| ZK0506 | a plain value printed with `ဟု` instead of `ကို` | [Natural Zaka](../../learn/natural-zaka/README.md#the-rules) |
| ZK0507 | parentheses that change nothing | [Natural Zaka](../../learn/natural-zaka/README.md#the-rules) |
| ZK0508 | a sentence used as a value that ends in `ပါ` | [Natural Zaka](../../learn/natural-zaka/README.md#the-rules) |
| ZK0509 | a warning: a kind, a heap size or a struct literal counted with a word that is not the classifier of its kind | [Measure words](../language/kinds.md#measure-words), [Struct mistakes](../../learn/structs/mistakes.md#the-wrong-classifier) |

### ZK06: working out constants

| Code | Problem | See |
| --- | --- | --- |
| ZK0601 | a sum of constants that does not fit its kind | [Arithmetic](../../learn/operators/arithmetic.md) |
| ZK0602 | a constant zero as the divisor | [Arithmetic](../../learn/operators/arithmetic.md#dividing-integers) |
| ZK0603 | a constant cell number outside its list | [Static data and counted lists](../../learn/forms/counted-lists.md#a-cell-number-past-a-counted-list) |
| ZK0604 | a variable of an imported file whose starting value is not known at compile time | [Module mistakes](../../learn/modules/mistakes.md#a-starting-value-that-needs-a-call) |
| ZK0605 | a constant range past the end of its list | [Ranges](../language/lists.md#ranges) |
| ZK0606 | a constant that holds text or a plain list, whose value needs the program to run | [Static data and counted lists](../../learn/forms/counted-lists.md#constants-of-text-and-lists) |
| ZK0607 | a range of known text that cuts a character | [Text](../language/text.md) |

### ZK07: flow checks and the lint

The [checks](../compiler/checks.md) that follow pointers, the null address, cell numbers,
integer ranges and unset values through the program. Under `--mem-check warn`, the default, ZK0702 to ZK0704,
ZK0706 to ZK0708, ZK0712 and ZK0721 are warnings and the program runs; under
`--mem-check error` the compiler reports them as errors, and under `--mem-check off` the compiler leaves them
out. ZK0709 to ZK0711 follow the same mode inside unsafe code, and in safe code ZK0710, which a
list gives, keeps its own level. ZK0701, ZK0722 and ZK0724 are errors in each mode, and
ZK0723 is a warning in each mode: no `--mem-check` level changes the check of set before
read. `--allow`, `--warn` and `--error` set the level of one group or code of these over
`--mem-check`, ZK0719 and ZK0723 among them, but never that of ZK0701, ZK0722 or ZK0724
(see [Commands](../compiler/commands.md#options-of-the-compile-stages)). See [Unsafe
Zaka](../language/unsafe.md).

| Code | Problem | See |
| --- | --- | --- |
| ZK0701 | a read of a pointer after it went to a parameter marked `အပိုင်အနေဖြင့်`, which left it the null address | [Unsafe Zaka](../language/unsafe.md) |
| ZK0702 | giving back or resizing through a pointer that holds the address of a variable, never a heap block | [Unsafe Zaka](../language/unsafe.md) |
| ZK0703 | the address of a variable of a function stored in a heap block, behind a pointer, or in a top-level variable | [Compiler checks](../compiler/checks.md) |
| ZK0704 | giving back, resizing or editing a block while a pointer into it is still in use | [Unsafe Zaka](../language/unsafe.md) |
| ZK0706 | a function that gives back the address of its own local or parameter | [Unsafe Zaka](../language/unsafe.md) |
| ZK0707 | a call that can give back a block through one argument while another argument reaches it | [Unsafe Zaka](../language/unsafe.md) |
| ZK0708 | a block made in unsafe code that is not given back, handed over, returned as an owned result, or stored | [Unsafe Zaka](../language/unsafe.md) |
| ZK0709 | reading or writing through a pointer that is the null address on each path | [The null address](../language/unsafe.md#the-null-address) |
| ZK0710 | a box or cell number past the end of its block | [Compiler checks](../compiler/checks.md) |
| ZK0711 | a warning, with `--mem-strict` alone: a read or a write through a pointer that can be the null address | [Compiler checks](../compiler/checks.md) |
| ZK0712 | subtracting pointers into different blocks | [Unsafe Zaka](../language/unsafe.md) |
| ZK0719 | an operation that overflows for each value of its operands; a warning, not given with `--overflow wrap` | [Expressions](../language/expressions.md) |
| ZK0721 | the two ranges of `ကူးယူမည်` overlap, both in one known block at known offsets | [Writing and filling cells](../language/unsafe.md#writing-and-filling-cells) |
| ZK0722 | a read of a variable, a field or a cell that a path reaches with no value given | [Set before read](../language/statements.md#set-before-read), [Variables](../../learn/variables/README.md#set-before-read) |
| ZK0723 | a warning: a read of a counted list whose cells the check cannot prove set | [Set before read](../language/statements.md#set-before-read), [Static data and counted lists](../../learn/forms/counted-lists.md#filling-the-cells-one-at-a-time) |
| ZK0724 | a call of a function that reads a global that can be unset at the call | [Globals and calls](../language/statements.md#globals-and-calls) |
| ZK0725 | a return that can leave a parameter marked `ဒေတာ ထည့်ပေးရန်` unset | [Filled parameters](../language/functions.md#filled-parameters) |
