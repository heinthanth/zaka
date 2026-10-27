---
title: Error messages
description: How to read a compiler message, and what the common codes mean.
---

# Error messages

When something is wrong, `zakac` prints a message instead of running the program. Here
is a typical one:

```console
$ zakac run program.zaka
error[ZK0401]: expected isize, found f64
 --> program.zaka:2:10
  |
2 | အသက် ကို ၂.၅ ဟု ထားပါ။
  |       ^^^
  = help: convert the value with "ကို <kind> အဖြစ် ပြောင်းလဲသော ရလဒ်"
```

| Part | Means |
| --- | --- |
| `error` | the program will not run. A `warning` does not stop it. |
| `[ZK0401]` | the code; look it up in the tables below |
| `expected isize, found f64` | what went wrong |
| `program.zaka:2:10` | file, line 2, character 10 |
| `^^^` | points at the problem in the line |
| `help:` | a suggestion for fixing it (not every message has one) |

Fix the first error first. A single mistake can confuse the compiler into reporting
more errors after it.

When a program stops *while running* (for example, dividing by a zero that only appears
at run time), the message starts with `the program stopped:` and has no code.

## Short names for kinds

Messages use short names for kinds:

| Short name | Zaka kind |
| --- | --- |
| `isize` | `ကိန်းပြည့်` |
| `i8`, `i16`, `i32`, `i64`, `i128` | `၈ ဘစ် ကိန်းပြည့်` ... `၁၂၈ ဘစ် ကိန်းပြည့်` |
| `usize` | `လက္ခဏာမဲ့ ကိန်းပြည့်` |
| `u8`, `u16`, `u32`, `u64`, `u128` | `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်` ... |
| `f16`, `f32`, `f64`, `f128` | `၁၆ ဘစ် ဒဿမကိန်း` ... ; `f64` is plain `ဒဿမကိန်း` |
| `char` | `စာလုံး` |
| `bool` | `မှန်မှားတန်ဖိုး` |
| `*isize`, `*u8`, `**isize`, ... | `ကိန်းပြည့် လိပ်စာ`, ... ; one more `*` for each `၏ လိပ်စာ` after it |
| `ptr` | `လိပ်စာ` (the opaque pointer) |
| `null` | the kind of `ဗလာ` |

## Codes

The first two digits tell you which stage found the problem.

### ZK01: reading the characters

| Code | Problem | See |
| --- | --- | --- |
| ZK0101 | a character Zaka does not use, such as `;` or `?` | |
| ZK0102 | a malformed number, such as `၂.၅e3` or `၁၈ဘစ်` | [Writing integers](../values/literals.md) |
| ZK0103 | Burmese and Western digits in one number | [Writing integers](../values/literals.md) |
| ZK0104 | text with no closing `"` | [Text in quotes](../values/strings.md) |
| ZK0106 | unknown `{...}` or `\` escape | [Characters](../values/characters.md) |
| ZK0107 | a character literal with more than one character | [Characters](../values/characters.md) |

### ZK02: reading the sentences

| Code | Problem | See |
| --- | --- | --- |
| ZK0201 | something unexpected, often a missing `။` | [Hello, world](../hello/README.md) |
| ZK0202 | a value is missing where one was expected | |
| ZK0203 | the statement does not end in a verb such as `ပြသပါ` | [Grouping and order](../operators/grouping.md) |
| ZK0204 | a logical expression printed without parentheses | [Logic](../operators/logic.md) |
| ZK0205 | a verb is missing one of its parts | [Particles](../sentences/particles.md) |
| ZK0206 | wrong or missing particle, often a name joined to `ကို` | [Spaces between words](../sentences/spacing.md) |
| ZK0210 | a comparison inside a comparison | [Comparison](../operators/comparison.md) |
| ZK0211 | `သို့မဟုတ်ပါက` without an `အကယ်၍` before it | [If and else](../control-flow/if-else.md) |
| ZK0213 | the `ပြီးလျှင်` update is not last | [Generic loops](../control-flow/generic-loops.md) |
| ZK0214 | a function declared inside a block or function | [Functions](../functions/README.md) |
| ZK0215 | an `if` as the one-statement body of an `if` | [If and else](../control-flow/if-else.md) |
| ZK0219 | a digit too big for the number base | [Writing integers](../values/literals.md) |
| ZK0220 | a number too big for its kind | [Integers](../values/integers.md) |
| ZK0223 | text in quotes used somewhere other than printing | [Text in quotes](../values/strings.md) |
| ZK0226 | a block closer with no block to close | [Blocks and scope](../variables/scope.md) |

### ZK03: names

| Code | Problem | See |
| --- | --- | --- |
| ZK0301 | a name that was never declared, or is out of scope | [Blocks and scope](../variables/scope.md) |
| ZK0302 | the same name declared twice in one block | [Names](../variables/names.md) |
| ZK0303 | calling a function that does not exist | [Functions](../functions/README.md) |
| ZK0306 | changing a constant | [Constants](../variables/constants.md) |
| ZK0307 | changing a counted loop's counter | [Counted loops](../control-flow/counted-loops.md) |
| ZK0308 | a constant without a value | [Constants](../variables/constants.md) |
| ZK0309, ZK0310 | break or continue outside a loop | [Break and continue](../control-flow/break-continue.md) |
| ZK0311 | `ပြန်ပေးပါ` outside a function | [Functions](../functions/README.md) |
| ZK0312 | returning a value from a function without a result | [Results](../functions/results.md) |
| ZK0313 | returning without a value from a function with a result | [Results](../functions/results.md) |
| ZK0314 | the wrong number of arguments | [Parameters](../functions/parameters.md) |
| ZK0315 | using a function without a result as a value | [Results](../functions/results.md) |

### ZK04: kinds

| Code | Problem | See |
| --- | --- | --- |
| ZK0401 | a value of the wrong kind for a variable or parameter | [Converting between kinds](../operators/conversion.md) |
| ZK0402 | a condition that is not true/false | [If and else](../control-flow/if-else.md) |
| ZK0403 | an operator used on the wrong kind, such as `မှန် + ၁` | [Arithmetic](../operators/arithmetic.md) |
| ZK0404 | two kinds that cannot be combined without a conversion | [Arithmetic](../operators/arithmetic.md#mixing-kinds) |
| ZK0406 | a number too big for the variable's kind | [Integers](../values/integers.md) |
| ZK0407 | a shift by too many bits | [Bitwise operators](../operators/bitwise.md) |
| ZK0408 | a conversion that is not allowed | [Converting between kinds](../operators/conversion.md) |
| ZK0410 | an invalid assignment target: not a variable or a dereference | [Changing a variable](../variables/assignment.md) |
| ZK0411 | no address: taking the address of something that is not a variable or a dereference | [Pointers and memory](../pointers/README.md#what-has-an-address) |
| ZK0412 | taking the address of a fixed name: a constant or a counted-loop variable | [Pointers and memory](../pointers/README.md#what-has-an-address) |
| ZK0413 | not a pointer: dereferencing a value that has no pointer kind | [Memory mistakes](../pointers/mistakes.md) |
| ZK0414 | an opaque pointer (`လိပ်စာ`) dereferenced or offset before a conversion | [Pointer arithmetic](../pointers/arithmetic.md#looking-at-the-same-bytes-another-way) |
| ZK0415 | an operator not for pointers, such as `*` or `/` on an address | [Pointer arithmetic](../pointers/arithmetic.md#walking-with-a-pointer) |
| ZK0416 | pointer kinds differ: comparing or subtracting pointers to different kinds | [Pointer arithmetic](../pointers/arithmetic.md#walking-with-a-pointer) |
| ZK0417 | a heap statement that needs a pointer but was given another kind | [Heap memory](../pointers/heap.md) |
| ZK0418 | (warning) the address of a local returned from its function | [Changing the caller's variables](../pointers/out-parameters.md#how-long-an-address-lasts) |

### ZK05: natural Zaka (warnings)

ZK0501 to ZK0508 are warnings about particles and parentheses. See [Natural
Zaka](../natural-zaka/README.md).

### ZK06: working out constants

| Code | Problem | See |
| --- | --- | --- |
| ZK0601 | a sum of constants that does not fit its kind | [Arithmetic](../operators/arithmetic.md) |
| ZK0602 | dividing by a constant zero | [Arithmetic](../operators/arithmetic.md#dividing-integers) |
