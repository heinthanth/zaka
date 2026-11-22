---
title: Error messages
description: How to read a compiler message, and what the common codes mean.
---

# Error messages

When the compiler finds a mistake, `zakac` prints a message (a diagnostic) instead of
running the program, like this one:

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
| `error` | the program does not run; a `warning` does not stop it |
| `[ZK0401]` | the error code, which the tables below list |
| `expected isize, found f64` | what went wrong |
| `program.zaka:2:10` | file, line 2, character 10 |
| `^^^` | the part of the line with the problem |
| `help:` | a hint on how to fix it; not all messages have one |

Fix the first error first, because one mistake can lead the compiler to report more
errors after it.

When a program stops while it runs (a trap), for example on a division whose divisor
turns out to be zero at run time, the message starts with `the program stopped:` and has no code.

## Short names for data kinds

Messages use short English names for data kinds (types), close to the names in Rust. A
short name starts with the base data kind and adds one mark for each word around it, in
the order of the Burmese words: `*` for `လိပ်စာ`, `[n]` for `စာရင်း`, and `@` after the
name of a shared object kind.

| Short name | Zaka data kind |
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
| `isize[]` | `ကိန်းပြည့် စာရင်း`, a plain list, whose count is known when the program runs |
| `isize[3]*` | `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ`, the address of a whole list |
| `text` | `စာသား` |
| `အကောင့်@`, `အကောင့်@[]` | a [shared object](../language/objects.md) of the kind `အကောင့်`, a list of them |
| `null` | the kind of the null address, `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` |
| `fn(isize, isize) -> bool`, `fn(text(ကို))`, `fn()` | a [function kind](../language/function-values.md): the parameters, with `mut ` before a mutable parameter (`ဒေတာ ပြောင်းလဲရန်`), `out ` before a writable parameter (`ဒေတာ ထည့်ပေးရန်`) and the preposition in parentheses after a parameter, then `->` and the result. Inside another name, a function kind goes in parentheses: `(fn(isize))[]` is a list of them |
| `isize result or text error` | `ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ်`, a [result kind](../language/result-kinds.md) |
| `success or text error`, `isize result or failure` | `အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ်`, `ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ်`: `success` and `failure` mean a side with no value |
| `optional isize` | `ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`, an [optional kind](../language/optionals.md) |
| `map<text, isize>` | `စာသား ဖြင့် ညွှန်းသော ကိန်းပြည့် ဇယား`, a [table](../language/tables.md) (map): the index kind, then the value kind |
| `thread(isize)`, `thread()` | `ကိန်းပြည့် ပြန်ပေးမည့် အပြိုင်လုပ်ငန်း`, `အပြိုင်လုပ်ငန်း`: a [thread](../language/threads.md#the-thread-kind) |
| `pipe(text)` | `စာသား ပို့နိုင်သော ဒေတာလမ်းကြောင်း`, a [channel](../language/threads.md#channels) |
| `atomic(isize)` | `အက်တမ်မစ် ဖြစ်သော ကိန်းပြည့်`, an [atomic number](../language/threads.md#atomic-numbers) |

## Codes

The first two digits of a code show the stage of the compiler that found the problem. [Diagnostics](../compiler/diagnostics.md)
explains how a message is set out and when a message stops the compiler.

### ZK01: reading the characters

| Code | Problem | See |
| --- | --- | --- |
| ZK0101 | a character Zaka does not use, such as `;` or `?` | |
| ZK0102 | a malformed number, such as `၂.၅e3` or `၁၈ဘစ်` | [Integers](../../learn/values/integers.md) |
| ZK0103 | Burmese and Western digits in one number | [Integers](../../learn/values/integers.md) |
| ZK0104 | text with no closing `"` | [Text](../../learn/text/README.md) |
| ZK0105 | a character literal with no closing `'` on its line | [Characters](../../learn/values/characters.md) |
| ZK0106 | an unknown `\` escape, a lone brace, or an unknown `{...}` in a character | [Characters](../../learn/values/characters.md) |
| ZK0107 | a character literal with more than one character | [Characters](../../learn/values/characters.md) |
| ZK0108 | a name in braces with no closing brace on its line | [Names](../../learn/variables/names.md) |
| ZK0109 | a name in braces with nothing between them | [Names](../../learn/variables/names.md) |

### ZK02: reading the sentences

| Code | Problem | See |
| --- | --- | --- |
| ZK0201 | a word the parser did not expect at that point, often because a `။` is missing | [Hello, world](../../learn/hello/README.md) |
| ZK0202 | a value is missing where one was expected | |
| ZK0203 | the statement does not end in a verb such as `ပြသမည်` | [Grouping and order](../../learn/operators/grouping.md) |
| ZK0204 | a statement verb, or a call that ends in `မည်`, where a value should go, for example a logical expression printed without parentheses | [Logic](../../learn/operators/logic.md) |
| ZK0205 | a verb is missing one of its parts | [Particles](../../learn/sentences/particles.md) |
| ZK0206 | a wrong or missing particle, often because a name is joined to `ကို` | [Spaces between words](../../learn/sentences/spacing.md) |
| ZK0207 | two particles in a row after one operand | [Particles](../../learn/sentences/particles.md) |
| ZK0208 | operands left waiting for a verb at the end of a clause | [Particles](../../learn/sentences/particles.md) |
| ZK0209 | operands left over before a statement verb | [Particles](../../learn/sentences/particles.md) |
| ZK0210 | a comparison inside a comparison | [Comparison](../../learn/operators/comparison.md) |
| ZK0211 | `သို့မဟုတ်ပါက` without an `အကယ်၍` before it | [If and else](../../learn/control-flow/if-else.md) |
| ZK0212 | the `ပြီးလျှင်` update outside a generic loop | [Generic loops](../../learn/control-flow/generic-loops.md#the-update-goes-last) |
| ZK0213 | the `ပြီးလျှင်` update is not last | [Generic loops](../../learn/control-flow/generic-loops.md) |
| ZK0214 | an external function, a C function or the program start block declared inside a block or a function | [Functions](../language/functions.md), [Closures](../language/closures.md#nested-functions) |
| ZK0215 | an `if` as the single statement of an `if` branch, with no block around it | [If and else](../../learn/control-flow/if-else.md) |
| ZK0216 | a bit count that the kind does not have, such as `၁၀ ဘစ် ကိန်းပြည့်` | [Integers](../../learn/values/integers.md#building-an-integer-kind) |
| ZK0217 | a sign word before a kind that is not an integer | [Integers](../../learn/values/integers.md#building-an-integer-kind) |
| ZK0218 | a number base that is not a whole number from 2 to 36 | [Integers](../../learn/values/integers.md#bases) |
| ZK0219 | a digit too big for the number base | [Integers](../../learn/values/integers.md) |
| ZK0220 | a number too big for its kind | [Integers](../../learn/values/integers.md) |
| ZK0221 | a fraction or exponent in a literal that is given an integer kind | [Integers](../../learn/values/integers.md#the-order-of-the-parts) |
| ZK0222 | a data kind after a literal that does not fit it: a number needs a plain number kind, and a list needs a list kind | [Integers](../../learn/values/integers.md#the-order-of-the-parts) |
| ZK0225 | a generic loop that starts with `ကိန်းသေ` | [Generic loops](../../learn/control-flow/generic-loops.md#the-variable-belongs-to-the-loop) |
| ZK0226 | a block closer with no block to close | [Blocks and scope](../../learn/variables/scope.md) |
| ZK0227 | a list count that is not a whole number above zero, or names a constant whose value the compiler cannot work out | [Static data and counted lists](../../learn/forms/counted-lists.md#counted-lists) |
| ZK0228 | a struct declared inside a block | [Struct mistakes](../../learn/structs/mistakes.md#a-struct-declared-inside-a-block) |
| ZK0229 | `အများသုံး` before something other than a top-level declaration | [Module mistakes](../../learn/modules/mistakes.md#the-export-mark-in-the-wrong-place) |
| ZK0230 | an import after a declaration or statement, or inside a block | [Module mistakes](../../learn/modules/mistakes.md#an-import-below-a-statement) |
| ZK0231 | files that import each other | [Module mistakes](../../learn/modules/mistakes.md#files-that-import-each-other) |
| ZK0232 | an import in a program that was not read from a file | [Module mistakes](../../learn/modules/mistakes.md#imports-outside-a-file) |
| ZK0234 | a data kind with no measure word (`တစ်<word>`) after it, in a declaration, a parameter, a field, a result, a function kind, a conversion, a size, a default arm or after a literal | [Measure words](../language/kinds.md#measure-words) |
| ZK0235 | an enumeration declared inside a block | [Enumeration mistakes](../../learn/enums/mistakes.md#an-enumeration-declared-inside-a-block) |
| ZK0236 | a switch arm after the default arm | [Enumeration mistakes](../../learn/enums/mistakes.md#an-arm-after-the-default-arm) |
| ZK0238 | an import from `စံကား/` that names no module of the stdlib | [Input and output](../language/io.md) |
| ZK0239 | a function kind with parameters inside a list of parameters or fields, not in parentheses | [Function values](../language/function-values.md) |
| ZK0240 | a kind alias declared inside a block | [Kind aliases](../language/aliases.md#errors) |
| ZK0241 | a preposition string after `ဝိဘတ်` with a word that a preposition cannot contain, or a first preposition that starts with `ကို သုံး၍` | [Functions](../language/functions.md) |
| ZK0244 | nesting of parentheses, operands, calls, lists or blocks more than 128 levels deep | [The nesting limit](../internals/parser.md#the-nesting-limit) |
| ZK0245 | a list in words of two or more items without `နှင့်` before the last item, or with a `၊` after it | [Lists in words](../language/words.md#lists-in-words) |
| ZK0246 | the word that closes a list in words does not fit its count: `တို့` after one item, or none after two or more | [Lists in words](../language/words.md#lists-in-words) |
| ZK0247 | a trait or a trait block inside a block | [Traits](../language/traits.md) |
| ZK0248 | a note in a notes block about a name that is not a kind parameter of the declaration | [Generics](../language/generics.md) |
| ZK0249 | a preposition that starts with `၏` on a parameter other than the last, or before an ending phrase | [Functions](../language/functions-sentences.md#prepositions) |
| ZK0250 | warning: a declared name that is a word of a preposition or a phrase of a function the file sees | [Functions](../language/functions-sentences.md#what-the-flexible-grammar-costs) |
| ZK0251 | warning: a declared name that is a function the file sees with a call end glued to it | [Functions](../language/functions-sentences.md#what-the-flexible-grammar-costs) |
| ZK0252 | a written-as hint that breaks a rule of its form | [Generics](../language/generics.md#written-as-hints) |
| ZK0253 | a use of a written-as hint that gives two data kinds for a kind parameter that the hint names twice | [Generics](../language/generics.md#written-as-hints) |
| ZK0254 | the line that uses the defaults of a trait names a trait other than the one of its trait block | [Traits](../language/traits.md#trait-blocks) |
| ZK0255 | a compile-time condition with something the compiler cannot settle, or that compares a fact of the target with a value the fact can never have | [Compile time](../language/compile-time.md) |
| ZK0256 | the comptime mark `ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော` on something other than a function with a block at the top level | [Compile time](../language/compile-time.md#comptime-functions) |
| ZK0257 | the unsafe mark `စိတ်မချရသော` on the program start block | [Unsafe Zaka](../language/unsafe.md) |
| ZK0258 | a phrase string after `အစ စကားစု` or `အဆုံး စကားစု` with a word that a phrase cannot contain, or with `၏` | [Functions](../language/functions-sentences.md#starting-and-ending-phrases) |
| ZK0260 | a call written as a sentence whose words the parser can cut into arguments in more than one valid cut | [Functions](../language/functions-sentences.md#prepositions) |
| ZK0261 | a share, `... ကို အများသုံးနိုင်ရန် ပြန်လည် မျှဝေမည်။`, that names a name the file does not import | [Modules](../language/modules.md#sharing-imported-names) |
| ZK0262 | a use of a name that two whole imports bring from two files | [Modules](../language/modules.md#whole-imports) |
| ZK0263 | a struct whose marks do not go together: a C name or `ပြင်ပမှ လာသော` with kind parameters, or `မျှဝေနိုင်သော` or the thread mark with either; or the thread mark on an enumeration | [Structs](../language/structs.md#marks-after-the-head) |
| ZK0264 | a wait on several channels whose opener has no arm of a channel after it | [Threads](../language/threads.md#waiting-on-several-channels) |
| ZK0265 | a wait on several channels with both a time limit and an arm of no data | [Threads](../language/threads.md#waiting-on-several-channels) |

### ZK03: names

| Code | Problem | See |
| --- | --- | --- |
| ZK0301 | a name that was never declared, or is out of scope | [Blocks and scope](../../learn/variables/scope.md) |
| ZK0302 | the same name declared twice in one block | [Names](../../learn/variables/names.md) |
| ZK0303 | calling a function that does not exist | [Functions](../../learn/functions/README.md) |
| ZK0304 | two functions with the same name, parameter kinds, prepositions and result, or a function with a name that the file imports | [Functions](../language/functions.md) |
| ZK0305 | two parameters of one function with one name | [Parameters](../../learn/functions/parameters.md) |
| ZK0306 | changing a constant | [Constants](../../learn/variables/constants.md) |
| ZK0307 | changing a counted loop's counter | [Counted loops](../../learn/control-flow/counted-loops.md) |
| ZK0308 | a constant without a value | [Constants](../../learn/variables/constants.md) |
| ZK0309 | a break outside a loop | [Break and continue](../../learn/control-flow/break-continue.md) |
| ZK0310 | a continue outside a loop | [Break and continue](../../learn/control-flow/break-continue.md) |
| ZK0311 | `ပြန်ပေးမည်` outside a function | [Functions](../../learn/functions/README.md) |
| ZK0312 | returning a value from a function without a result | [Results](../../learn/functions/results.md) |
| ZK0313 | a return with no value, or `လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်`, in a function whose result has a good value | [Results](../../learn/functions/results.md) |
| ZK0314 | the wrong number of arguments | [Parameters](../../learn/functions/parameters.md) |
| ZK0315 | a function with no result used as a value, or `လုပ်ဆောင်သော ရလဒ်ကောင်း` on a function whose good side has no value | [Results](../../learn/functions/results.md) |
| ZK0316 | a struct, enumeration or kind alias name declared twice | [Struct mistakes](../../learn/structs/mistakes.md#one-name-for-two-structs) |
| ZK0317 | a field name used twice in one struct | [Struct mistakes](../../learn/structs/mistakes.md#one-name-for-two-fields) |
| ZK0318 | a name in a data kind that is not a declared struct or enumeration | [Struct mistakes](../../learn/structs/mistakes.md#a-name-that-is-no-struct) |
| ZK0319 | a struct or enumeration name used for a variable, parameter, field or function | [Struct mistakes](../../learn/structs/mistakes.md#a-struct-name-that-is-also-a-value-name) |
| ZK0320 | a field the struct does not have | [Struct mistakes](../../learn/structs/mistakes.md#a-field-the-struct-does-not-have) |
| ZK0321 | a field set twice in a struct literal | [Struct mistakes](../../learn/structs/mistakes.md#one-field-set-twice) |
| ZK0322 | an import of a name the other file does not export | [Module mistakes](../../learn/modules/mistakes.md#a-name-the-file-does-not-export) |
| ZK0323 | a statement in an imported file | [Module mistakes](../../learn/modules/mistakes.md#a-statement-in-an-imported-file) |
| ZK0324 | an exported declaration whose data kind uses a private struct | [Module mistakes](../../learn/modules/mistakes.md#a-private-struct-in-an-export) |
| ZK0325 | a second program start block | [Functions](../language/functions.md) |
| ZK0326 | a statement that runs, at the top level of a file | [Functions](../language/functions.md) |
| ZK0327 | a program with no program start block | [Functions](../language/functions.md) |
| ZK0328 | a member name used twice in one enumeration | [Enumeration mistakes](../../learn/enums/mistakes.md#one-member-name-twice) |
| ZK0329 | a member the enumeration does not have | [Enumeration mistakes](../../learn/enums/mistakes.md#a-member-the-enumeration-does-not-have) |
| ZK0330 | a name before `ထဲမှ` that is not an enumeration name with `များ` | [Enumeration mistakes](../../learn/enums/mistakes.md#a-member-written-without-the-plural-mark) |
| ZK0331 | an argument of a call with prepositions that has a preposition other than its parameter's | [Functions](../language/functions.md) |
| ZK0332 | a call that fits no function of that name | [Functions](../language/functions.md) |
| ZK0333 | a call that fits more than one function of that name | [Functions](../language/functions.md) |
| ZK0334 | an external function that is not one of the host functions of the VM | [Functions](../language/functions.md) |
| ZK0335 | a call whose name matches both a declared function and a function value | [Function values](../language/function-values.md) |
| ZK0336 | a chain of kind aliases that loops back to its start | [Kind aliases](../language/aliases.md#errors) |
| ZK0337 | the name of a C function, or the C name of a function or struct exported to C, that is not a valid C name | [Functions](../language/functions.md) |
| ZK0338 | a C function declared twice with different parameter or result kinds | [Functions](../language/functions.md) |
| ZK0339 | a function with a result that can reach its end without a return | [Results](../../learn/functions/results.md) |
| ZK0340 | a call or a data kind that gives a generic a different number of data kinds than it declares, or gives data kinds to a function that is not generic | [Generics](../language/generics.md) |
| ZK0341 | a call of a generic function, or a value of a generic struct, whose data kinds the compiler cannot work out from its values | [Generics](../language/generics.md) |
| ZK0342 | instances of a generic function or struct that each need a deeper instance, past the limit | [Generics](../language/generics.md) |
| ZK0343 | a data kind given to a generic that lacks a trait which its kind parameter needs | [Traits](../language/traits.md#when-a-data-kind-has-a-trait) |
| ZK0344 | a constraint or a trait block with a name that is not a trait | [Traits](../language/traits.md#when-a-data-kind-has-a-trait) |
| ZK0345 | a trait block that lacks a function of its trait | [Traits](../language/traits.md#trait-blocks) |
| ZK0346 | a function of a trait block that is not a function of its trait | [Traits](../language/traits.md#trait-blocks) |
| ZK0347 | two functions that both give a data kind the same function of a trait | [Traits](../language/traits.md#when-a-data-kind-has-a-trait) |
| ZK0348 | a trait block in a file that declares neither its data kind nor its trait | [Traits](../language/traits.md#trait-blocks) |
| ZK0349 | the compiler mark `ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော` outside the modules of the stdlib | [The core module](../language/core.md) |
| ZK0350 | warning: a function with the same name, data kinds and prepositions as a function of the core module, which then replaces that function in each call | [The core module](../language/core.md#verbs-on-structs) |
| ZK0351 | warning: a trait block that asks for the defaults of its trait, though no function it leaves out has a default | [Traits](../language/traits.md#trait-blocks) |
| ZK0352 | a call written as a sentence that leaves out a phrase of its function signature, or uses a different phrase | [Functions](../language/functions-sentences.md#starting-and-ending-phrases) |
| ZK0353 | an enumeration gives some members constant values and lets others hold data | [Enumerations](../language/enums.md#members-that-hold-data) |
| ZK0354 | a member that holds data is written with no data | [Enumerations](../language/enums.md#members-that-hold-data) |
| ZK0355 | data is given to, read from or named for a member that holds none | [Enumerations](../language/enums.md#members-that-hold-data) |
| ZK0356 | a test or a read of a member looks at a value that is no enumeration | [Enumerations](../language/enums.md#members-that-hold-data) |
| ZK0357 | a name for the data of a member, `<name> ဟူသည့်`, outside the condition of `အကယ်၍` or a switch arm | [Enumerations](../language/enums.md#members-that-hold-data) |
| ZK0358 | a read of the data of a member that a test of the variable ruled out | [Enumerations](../language/enums.md#members-that-hold-data) |
| ZK0359 | a case of a switch over an enumeration whose members hold data gives data | [Enumerations](../language/enums.md#switch) |
| ZK0360 | a trait used as a data kind (`<trait> အရာ`) has a function that does not take the trait's data kind first, or takes it elsewhere | [Trait values](../language/trait-values.md#which-traits-can-be-a-data-kind) |
| ZK0361 | a trait value is compared, printed, given an initial value or called as a function value | [Trait values](../language/trait-values.md#what-a-trait-value-cannot-do) |
| ZK0362 | a warning of the lint group `duplicates`: a plain value goes into a trait value whose trait has a function that changes its first parameter | [Trait values](../language/trait-values.md#changes-through-a-trait-value) |

### ZK04: data kinds

| Code | Problem | See |
| --- | --- | --- |
| ZK0401 | a value of the wrong data kind for a variable or a parameter | [Converting between data kinds](../../learn/operators/conversion.md) |
| ZK0402 | a condition that is not true/false | [If and else](../../learn/control-flow/if-else.md) |
| ZK0403 | an operator used on the wrong data kind, such as `မှန် + ၁` | [Arithmetic](../../learn/operators/arithmetic.md) |
| ZK0404 | two data kinds that we cannot combine without a conversion | [Arithmetic](../../learn/operators/arithmetic.md#mixing-data-kinds) |
| ZK0405 | a counted-loop bound or step, or the count of threads of a parallel loop or map, that is not an integer | [Counted loops](../../learn/control-flow/counted-loops.md#details) |
| ZK0406 | a number too big for the data kind of the variable | [Integers](../../learn/values/integers.md) |
| ZK0407 | a shift wider than the bit count of the data kind | [Bitwise operators](../../learn/operators/bitwise.md) |
| ZK0408 | a conversion that is not allowed | [Converting between data kinds](../../learn/operators/conversion.md) |
| ZK0409 | a counted-loop step that is zero or less | [Counted loops](../../learn/control-flow/counted-loops.md#details) |
| ZK0410 | an invalid target of an assignment, or an argument for a mutable parameter (`ဒေတာ ပြောင်းလဲရန်`) that is not a place: a variable, a field, a cell or a read through a pointer | [Changing a variable](../../learn/variables/assignment.md) |
| ZK0411 | no address: taking the address of something that is not a variable, a field, a cell or a read through a pointer | [Unsafe Zaka](../language/unsafe.md) |
| ZK0412 | taking the address of a fixed name: a constant or a counted-loop variable | [Unsafe Zaka](../language/unsafe.md) |
| ZK0413 | not a pointer: reading through a value that has no pointer kind | [Unsafe Zaka](../language/unsafe.md) |
| ZK0414 | an opaque pointer (`လိပ်စာ`) read through or offset before a conversion | [Unsafe Zaka](../language/unsafe.md) |
| ZK0415 | an operator not for pointers, such as `*` or `/` on an address | [Unsafe Zaka](../language/unsafe.md) |
| ZK0416 | pointer kinds differ: comparing or subtracting pointers to different data kinds | [Unsafe Zaka](../language/unsafe.md) |
| ZK0417 | a heap statement, such as a resize, a duplicate or a fill, that needs a pointer but was given another data kind | [Unsafe Zaka](../language/unsafe.md) |
| ZK0418 | a while loop whose block is one wait on several channels with the arm of no data and no sleep, which keeps a core at work (a warning) | [Threads](../language/threads.md#waiting-on-several-channels) |
| ZK0419 | a list literal with more or fewer cells than its data kind | [Static data and counted lists](../../learn/forms/counted-lists.md#a-literal-with-the-wrong-count) |
| ZK0420 | `စာရင်းအလွတ်` where no list kind is known | [List mistakes](../../learn/lists/mistakes.md#a-blank-list-with-no-data-kind) |
| ZK0422 | cells of a value that is not a list | [List mistakes](../../learn/lists/mistakes.md#cells-of-a-value-that-is-no-list) |
| ZK0423 | a cell number that is not an integer | [Lists](../../learn/lists/README.md#cell-numbers) |
| ZK0425 | a list kind too large for the VM | [Static data and counted lists](../../learn/forms/counted-lists.md#the-size-of-a-counted-list) |
| ZK0426 | a field of a value that is not a struct | [Struct mistakes](../../learn/structs/mistakes.md#a-field-of-a-value-that-is-no-struct) |
| ZK0427 | a struct that contains itself | [Struct mistakes](../../learn/structs/mistakes.md#a-struct-that-contains-itself) |
| ZK0429 | a write to a cell (a u8) of text | [Text](../language/text.md) |
| ZK0430 | a declaration with no data kind whose value is the null address | [Statements](../language/statements.md) |
| ZK0433 | one place passed to two mutable parameters of one call, or a place that the call changes, used again in another argument of the same call | [Functions](../language/functions.md) |
| ZK0434 | a loop over each cell or each character of a value that it cannot walk, such as a table: an each-cell loop needs a list, and an each-character loop needs text | [Loops](../language/control-flow.md#loops) |
| ZK0436 | `[]` or `စာရင်းအလွတ်` given to a counted list | [Static data and counted lists](../../learn/forms/counted-lists.md#a-list-with-no-cells-in-a-counted-list) |
| ZK0437 | an add or a remove on a counted list, or `နောက်ဆုံးအကွက်` on text | [Static data and counted lists](../../learn/forms/counted-lists.md#adding-to-a-counted-list) |
| ZK0439 | `၏ အရှေ့တွင်` after a value that is neither a cell of a list nor a character of text | [Lists](../language/lists.md) |
| ZK0440 | a parameter of the program start block that is not a `စာသား စာရင်း`, is a mutable parameter, or is a third one | [Functions](../language/functions.md) |
| ZK0441 | a result of the program start block that is not an integer, such as a result kind | [Functions](../language/functions.md) |
| ZK0442 | a raw duplicate or a fill of cells that contain counted blocks: text, plain lists, objects or function values | [Writing and filling cells](../language/heap.md#writing-and-filling-cells) |
| ZK0443 | a call statement of a function that can fail, with no pass-on | [Errors and results](../language/error-handling.md) |
| ZK0444 | a pass-on for a function that cannot fail | [Errors and results](../language/error-handling.md) |
| ZK0445 | a pass-on in a function that cannot fail, in the program start block, or in top-level statements | [Errors and results](../language/error-handling.md) |
| ZK0446 | a pass-on whose error kind differs from the error kind of the function around the call, with no `ပြောင်းလဲ` between the two | [Errors and results](../language/error-handling.md) |
| ZK0447 | an error value returned from a function that cannot fail, or `လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်` in such a function | [Result kinds](../language/result-kinds.md) |
| ZK0448 | a test or a read (`ထဲတွင် … ရှိ`, `ထဲမှ ဒေတာ`, `ထဲမှ အမှားအယွင်း ဒေတာ`) of a value that is neither a result nor an optional | [Reading a result](../language/reading-results.md) |
| ZK0449 | a read of the good value, `ထဲမှ ဒေတာ`, of a result whose good side has no value | [Reading a result](../language/reading-results.md) |
| ZK0450 | a read of the error value of a result whose error side has no value | [Result kinds](../language/result-kinds.md) |
| ZK0451 | a failure with no value in a function whose error side names a kind | [Result kinds](../language/result-kinds.md) |
| ZK0454 | a read of the part of a result, or the data of an optional, that a test has ruled out | [Reading a result](../language/reading-results.md) |
| ZK0456 | a member value that the compiler cannot work out | [Enumeration mistakes](../../learn/enums/mistakes.md#a-member-value-that-is-not-a-constant) |
| ZK0457 | two members of one enumeration with one value | [Enumeration mistakes](../../learn/enums/mistakes.md#two-members-with-one-value) |
| ZK0458 | a switch over an enumeration that misses a member and has no default arm | [Enumeration mistakes](../../learn/enums/mistakes.md#a-switch-that-misses-a-member) |
| ZK0459 | a default arm whose data kind is not the data kind of the subject | [Enumeration mistakes](../../learn/enums/mistakes.md#a-default-arm-of-the-wrong-data-kind) |
| ZK0460 | two switch cases with one value | [Enumeration mistakes](../../learn/enums/mistakes.md#one-case-twice) |
| ZK0461 | a member with no value that cannot count on from the member before it: its data kind is not an integer, or the value passes the largest value of the data kind | [Enumeration mistakes](../../learn/enums/mistakes.md#a-member-with-no-value-that-cannot-count-on) |
| ZK0462 | a call through a name or a place whose data kind is not a function kind, or a leave to call for C on a value that is not a function value | [Function values](../language/function-values.md) |
| ZK0463 | a function value from a name with several functions, where no single one fits the data kind that the value goes to | [Function values](../language/function-values.md) |
| ZK0464 | a function value of a function that comes from outside the program | [Function values](../language/function-values.md) |
| ZK0465 | a parameter or the result of a C function with a data kind that C cannot use: a bare text result, a function kind with a struct, text or `ဒေတာ ပြောင်းလဲရန်` in it, or a list of something other than numbers | [Functions](../language/functions.md) |
| ZK0466 | `ဒေတာ ပြောင်းလဲရန်` or `ဒေတာ ထည့်ပေးရန်` on a parameter of a C function that is neither a list of numbers nor a struct that can cross to C | [Functions](../language/functions.md) |
| ZK0467 | `အပိုင်အနေဖြင့်` on the result of a C function that is not text | [Functions](../language/functions.md) |
| ZK0468 | reading or writing through a pointer, pointer arithmetic, an address, a heap statement, a pointer conversion or a call of an unsafe function, outside unsafe code | [Unsafe Zaka](../language/unsafe.md) |
| ZK0469 | an unsafe function (`စိတ်မချရသော`) named as a function value | [Unsafe Zaka](../language/unsafe.md) |
| ZK0470 | `အပိုင်အနေဖြင့်` on a parameter or result that is not a pointer (a handle parameter of a C function is allowed) | [Unsafe Zaka](../language/unsafe.md) |
| ZK0471 | `စိတ်မချရသော`, on a function or a block, under `--no-unsafe` | [Unsafe Zaka](../language/unsafe.md) |
| ZK0472 | a C function declared without `စိတ်မချရသော` | [Functions](../language/functions.md), [Unsafe Zaka](../language/unsafe.md) |
| ZK0473 | a value of a struct from outside the program (`ပြင်ပမှ လာသော ဒေတာအစုအဝေး`) that the program makes, duplicates or prints | [Functions](../language/functions-c.md#data-kinds-from-outside-the-program) |
| ZK0474 | a write to a field of a struct from outside the program, outside unsafe code, or a write to its text or handle field: the struct belongs to C | [Functions](../language/functions-c.md#data-kinds-from-outside-the-program) |
| ZK0475 | a test word that does not fit the data kind: `ဒေတာ` on a result, or `ရလဒ်ကောင်း` or `အမှားအယွင်း` on an optional | [Optionals](../language/optionals.md) |
| ZK0476 | `<value> ဟူသော အမှားအယွင်း` where no result kind with an error side is expected | [Error handling](../language/error-handling.md) |
| ZK0477 | `အလွတ်` where no optional kind is expected | [Optionals](../language/optionals.md) |
| ZK0478 | `ကနဦးတန်ဖိုး` where the compiler cannot work out the data kind | [Initial values](../language/kinds.md#initial-values) |
| ZK0479 | `ကနဦးတန်ဖိုး` for a data kind that has no initial value: a function kind, a result, a handle, an enumeration with no marked member, or a data kind that contains one | [Initial values](../language/kinds.md#initial-values), [Enumeration mistakes](../../learn/enums/mistakes.md#no-member-marked-as-the-initial-one) |
| ZK0480 | two members of one enumeration marked `ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်` | [The initial member](../language/enums.md#the-initial-member), [Enumeration mistakes](../../learn/enums/mistakes.md#two-members-marked-as-the-initial-one) |
| ZK0481 | a data kind that no program can build, because its fields lead back to it with no optional or plain list on the loop | [Data kinds no program can build](../language/objects.md#data-kinds-no-program-can-build) |
| ZK0482 | a struct literal that leaves out a field | [Struct literals](../language/structs.md#struct-literals), [Struct mistakes](../../learn/structs/mistakes.md#a-field-left-out) |
| ZK0483 | a global of a module declared with no value | [Globals of a module](../language/modules.md#globals-of-a-module), [Module mistakes](../../learn/modules/mistakes.md#a-variable-of-a-module-with-no-value) |
| ZK0484 | a row read, `အညွှန်းအရေအတွက်`, `အညွှန်းတိုင်း` or `အတန်းတိုင်း` on a value that is not a table | [Tables](../language/tables.md#reading-a-row) |
| ZK0485 | a table kind whose index kind contains a part that cannot be hashed and compared as a whole value: an object, a function value, a pointer, an optional, a result or a table | [Index kinds](../language/tables.md#index-kinds) |
| ZK0486 | `ဇယားအလွတ်` or `[:]` with no table kind from its place | [Tables](../language/tables.md#literals) |
| ZK0487 | a warning: a table kind whose index kind contains a decimal number, which rounding can split into two rows | [Index kinds](../language/tables.md#index-kinds) |
| ZK0489 | a comptime function whose result is not a value that a constant can have | [Compile time](../language/compile-time.md#comptime-functions) |
| ZK0490 | a function exported to C that is generic, is declared inside a function, has no statements, or takes over or returns an owned block | [Functions C can call](../language/functions-exports.md) |
| ZK0491 | the C name of a function or struct exported to C is `main`, starts with `zk_` or `ZK_`, or names a C function or another such function or struct | [Functions C can call](../language/functions-exports.md) |
| ZK0492 | a function exported to C passes C a struct that has no C name | [Functions C can call](../language/functions-exports.md) |
| ZK0493 | warning: the thread mark `အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော` on a struct that is not a shared object, where it changes nothing | [Objects threads can use](../language/objects.md#objects-threads-can-use) |
| ZK0494 | a value that goes to a thread, or comes back from one, of a data kind that other threads cannot use, such as a plain shared object | [Values that go to a thread](../language/threads.md#values-that-go-to-a-thread) |
| ZK0495 | a receive, a loop over each value, or a close on a value that is not a channel | [Channels](../language/threads.md#channels) |
| ZK0496 | a test of a channel for data that is not the whole condition of `အကယ်၍`, or a read of its data outside the branch of that test | [Channels](../language/threads.md#channels) |
| ZK0497 | a thread that starts in a function value, a closure or a C function, not in a function that the program declares in Zaka | [What a thread can reach](../language/threads.md#what-a-thread-can-reach) |
| ZK0498 | a thread that starts in a function with a mutable or writable parameter | [What a thread can reach](../language/threads.md#what-a-thread-can-reach) |
| ZK0499 | a field of an object that threads share, read or written outside a lock block on the object, or a lock block on a value that is not a shared object | [Objects threads share](../language/threads.md#objects-threads-share) |

### ZK05: natural Zaka (warnings)

These are warnings, so the program still runs; with `--lint-error naturalness`, each one
becomes an error.
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
| ZK0509 | a warning: a data kind, a heap size or a struct literal counted with a word that is not the measure word of its data kind | [Measure words](../language/kinds.md#measure-words), [Struct mistakes](../../learn/structs/mistakes.md#the-wrong-measure-word) |
| ZK0510 | a word in its spoken spelling, such as `ထားမယ်` for `ထားမည်`, in a program that does not allow them | [Spoken Burmese](../language/spoken.md) |
| ZK0511 | a warning: a file that allows the spoken spellings writes some words spoken and others written | [Spoken Burmese](../language/spoken.md#mixing-the-two) |
| ZK0512 | a verb before `မဟုတ်` without `သည်` | [Natural Zaka](../../learn/natural-zaka/README.md#the-rules) |

### ZK06: working out constants

| Code | Problem | See |
| --- | --- | --- |
| ZK0601 | a sum of constants that does not fit its data kind | [Arithmetic](../../learn/operators/arithmetic.md) |
| ZK0602 | a constant zero as the divisor | [Arithmetic](../../learn/operators/arithmetic.md#dividing-integers) |
| ZK0603 | a constant cell number outside its list | [Static data and counted lists](../../learn/forms/counted-lists.md#a-cell-number-past-a-counted-list) |
| ZK0604 | a variable of an imported file whose starting value is not known at compile time, such as a call of a function that is not comptime | [Module mistakes](../../learn/modules/mistakes.md#a-starting-value-that-needs-a-call) |
| ZK0605 | a constant range past the end of its list | [Ranges](../language/lists.md#ranges) |
| ZK0606 | a constant of text or a plain list, whose value needs the program to run | [Static data and counted lists](../../learn/forms/counted-lists.md#constants-of-text-and-lists) |
| ZK0607 | a range of known text that cuts a character | [Text](../language/text.md) |
| ZK0608 | an argument of a comptime call in a compile-time spot that is not known at compile time | [Compile time](../language/compile-time.md#where-a-call-runs-at-compile-time) |
| ZK0609 | a comptime call that needs a value waiting on the call itself, or a condition that calls a function its own chain declares | [Compile time](../language/compile-time.md#conditions-that-call-comptime-functions) |
| ZK0610 | a comptime call in a tool with no VM to run it | [Compile time](../language/compile-time.md#what-a-run-can-do) |
| ZK0611 | a comptime call whose run stopped: a trap, a cap, an effect with no right, or code that does not compile | [Compile time](../language/compile-time.md#what-a-run-can-do) |
| ZK0612 | a note: what a comptime call printed while the compiler ran it | [Compile time](../language/compile-time.md#what-a-run-can-do) |

### ZK07: flow checks and the lint

These codes come from the [flow checks](../compiler/checks.md), which follow pointers, the
null address, cell numbers, integer ranges and unset values through the program. The level
of each code depends on the `mem` lint:

- Under `--lint-warn mem`, the default, ZK0702 to ZK0704, ZK0706 to ZK0708, ZK0712 and
  ZK0721 are warnings and the program runs. Under `--lint-error mem` the compiler reports
  them as errors, and under `--lint-allow mem` it leaves them out.
- ZK0709 to ZK0711 follow the same level inside unsafe code. In safe code, ZK0710, which a
  list gives, keeps a separate level.
- ZK0701, ZK0722 and ZK0724 are errors at each level, and ZK0723 is a warning at each
  level, because no level of the `mem` lint changes the check of set before read.
- `--lint-allow`, `--lint-warn` and `--lint-error` can also set the level of one group or
  code of these, ZK0719 and ZK0723 among them, but never the level of ZK0701, ZK0722 or
  ZK0724 (see [Commands](../compiler/commands.md#options-of-the-compile-stages)).

See [Unsafe Zaka](../language/unsafe.md).

| Code | Problem | See |
| --- | --- | --- |
| ZK0701 | a read of a pointer after it went to an owned parameter (`အပိုင်အနေဖြင့်`), which left it the null address | [Unsafe Zaka](../language/unsafe.md) |
| ZK0702 | freeing or resizing through a pointer that has the address of a variable, not of a heap block | [Unsafe Zaka](../language/unsafe.md) |
| ZK0703 | the address of a local variable stored in a heap block, behind a pointer, or in a top-level variable | [Compiler checks](../compiler/checks.md) |
| ZK0704 | freeing, resizing or editing a block while a pointer into it is still in use | [Unsafe Zaka](../language/unsafe.md) |
| ZK0706 | a function that returns the address of one of its locals or parameters | [Unsafe Zaka](../language/unsafe.md) |
| ZK0707 | a call that can free a block through one argument while another argument reaches it | [Unsafe Zaka](../language/unsafe.md) |
| ZK0708 | a block made in unsafe code that is not freed, handed over, returned as an owned result, or stored | [Unsafe Zaka](../language/unsafe.md) |
| ZK0709 | reading or writing through a pointer that is the null address on each path | [The null address](../language/pointers.md#the-null-address) |
| ZK0710 | a box or cell number past the end of its block | [Compiler checks](../compiler/checks.md) |
| ZK0711 | a warning, with `--lint-warn doubts` alone: a read or a write through a pointer that can be the null address | [Compiler checks](../compiler/checks.md) |
| ZK0712 | subtracting pointers into different blocks | [Unsafe Zaka](../language/unsafe.md) |
| ZK0719 | an operation that overflows for each value of its operands; a warning, not given with `--overflow wrap` | [Expressions](../language/expressions.md) |
| ZK0721 | the two ranges of `ကူးယူမည်` overlap, both in one known block at known offsets | [Writing and filling cells](../language/heap.md#writing-and-filling-cells) |
| ZK0722 | a read of a variable, a field or a cell on a path where it has no value | [Set before read](../language/declarations.md#set-before-read), [Variables](../../learn/variables/README.md#set-before-read) |
| ZK0723 | a warning: a read of a counted list whose cells the check cannot prove set | [Set before read](../language/declarations.md#set-before-read), [Static data and counted lists](../../learn/forms/counted-lists.md#filling-the-cells-one-at-a-time) |
| ZK0724 | a call of a function that reads a global that can be unset at the call | [Globals and calls](../language/declarations.md#globals-and-calls) |
| ZK0725 | a return that can leave a writable parameter (`ဒေတာ ထည့်ပေးရန်`) unset | [Writable parameters](../language/functions-parameters.md#writable-parameters) |
| ZK0726 | a function that a thread or a round of a parallel loop runs reaches a variable at the top of a file, itself or through a function it calls | [What a thread can reach](../language/threads.md#what-a-thread-can-reach) |
| ZK0727 | a set of an atomic number whose value reads the same number | [Atomic numbers](../language/threads.md#atomic-numbers) |
| ZK0728 | a round of a parallel loop writes a variable from outside the loop, other than the cell with the number of the round, or lets a call change one | [Parallel loops and maps](../language/threads.md#parallel-loops-and-maps) |
| ZK0729 | a round of a parallel loop stops the loop with a break, or returns from the function around it | [Parallel loops and maps](../language/threads.md#parallel-loops-and-maps) |
