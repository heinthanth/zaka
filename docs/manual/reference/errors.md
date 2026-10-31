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
| `help:` | a suggestion for fixing it (some messages have none) |

Fix the first error first. A single mistake can confuse the compiler into reporting
more errors after it.

When a program stops *while running* (for example, on a division whose zero appears at
run time), the message starts with `the program stopped:` and has no code.

## Short names for kinds

Messages use short names for kinds:

| Short name | Zaka kind |
| --- | --- |
| `isize` | `ကိန်းပြည့်` |
| `i8`, `i16`, `i32`, `i64`, `i128` | `၈ ဘစ် ကိန်းပြည့်` ... `၁၂၈ ဘစ် ကိန်းပြည့်` |
| `usize` | `လက္ခဏာမဲ့ ကိန်းပြည့်` |
| `u8`, `u16`, `u32`, `u64`, `u128` | `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်` ... |
| `f16`, `f32`, `f64`, `f128` | `၁၆ ဘစ် ဒဿမကိန်း` ... ; `f64` is plain `ဒဿမကိန်း` |
| `char` | `စာလုံး` |
| `bool` | `မှန်မှားတန်ဖိုး` |
| `*isize`, `*u8`, `**isize`, ... | `ကိန်းပြည့် လိပ်စာ`, ... ; one more `*` for each `၏ လိပ်စာ` after it |
| `ptr` | `လိပ်စာ` (the opaque pointer) |
| `[3]isize`, `[2][3]isize` | `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း`, ... ; one `[n]` for each `စာရင်း`, outermost first; `[]` while the count is not known |
| `*[3]isize` | `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ`, the address of a whole list |
| `null` | the kind of `ဗလာ` |

## Codes

The first two digits tell which stage found the problem. [Diagnostics](../compiler/diagnostics.md)
explains how a message is set out and when a message stops the compiler.

### ZK01: reading the characters

| Code | Problem | See |
| --- | --- | --- |
| ZK0101 | a character Zaka does not use, such as `;` or `?` | |
| ZK0102 | a malformed number, such as `၂.၅e3` or `၁၈ဘစ်` | [Writing integers](../../learn/values/literals.md) |
| ZK0103 | Burmese and Western digits in one number | [Writing integers](../../learn/values/literals.md) |
| ZK0104 | text with no closing `"` | [Text in quotes](../../learn/values/strings.md) |
| ZK0105 | a character literal with no closing `'` on its line | [Characters](../../learn/values/characters.md) |
| ZK0106 | unknown `{...}` or `\` escape | [Characters](../../learn/values/characters.md) |
| ZK0107 | a character literal with more than one character | [Characters](../../learn/values/characters.md) |
| ZK0108 | a name in backticks with no closing backtick on its line | [Names](../../learn/variables/names.md) |
| ZK0109 | a name in backticks with nothing between them | [Names](../../learn/variables/names.md) |

### ZK02: reading the sentences

| Code | Problem | See |
| --- | --- | --- |
| ZK0201 | something unexpected, often a missing `။` | [Hello, world](../../learn/hello/README.md) |
| ZK0202 | a value is missing where one was expected | |
| ZK0203 | the statement does not end in a verb such as `ပြသပါ` | [Grouping and order](../../learn/operators/grouping.md) |
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
| ZK0214 | a function declared inside a block or function | [Functions](../../learn/functions/README.md) |
| ZK0215 | an `if` as the one-statement inner block of an `if` | [If and else](../../learn/control-flow/if-else.md) |
| ZK0216 | a bit count that the kind does not have, such as `၁၀ ဘစ် ကိန်းပြည့်` | [Integers](../../learn/values/integers.md#building-an-integer-kind) |
| ZK0217 | a sign word before a kind that is not an integer | [Integers](../../learn/values/integers.md#building-an-integer-kind) |
| ZK0218 | a number base that is not a whole number from 2 to 36 | [Writing integers](../../learn/values/literals.md#bases) |
| ZK0219 | a digit too big for the number base | [Writing integers](../../learn/values/literals.md) |
| ZK0220 | a number too big for its kind | [Integers](../../learn/values/integers.md) |
| ZK0221 | a fraction or exponent in a literal that is given an integer kind | [Writing integers](../../learn/values/literals.md#the-order-of-the-parts) |
| ZK0222 | a character or true/false kind written before a number | [Writing integers](../../learn/values/literals.md#the-order-of-the-parts) |
| ZK0223 | text in quotes used somewhere other than printing | [Text in quotes](../../learn/values/strings.md) |
| ZK0225 | a generic loop that starts with `ကိန်းသေ` | [Generic loops](../../learn/control-flow/generic-loops.md#the-variable-belongs-to-the-loop) |
| ZK0226 | a block closer with no block to close | [Blocks and scope](../../learn/variables/scope.md) |
| ZK0227 | a list count that is not a whole number above zero | [Lists](../../learn/lists/README.md#list-kinds) |
| ZK0228 | a struct declared inside a block | [Struct mistakes](../../learn/structs/mistakes.md#a-struct-declared-inside-a-block) |

### ZK03: names

| Code | Problem | See |
| --- | --- | --- |
| ZK0301 | a name that was never declared, or is out of scope | [Blocks and scope](../../learn/variables/scope.md) |
| ZK0302 | the same name declared twice in one block | [Names](../../learn/variables/names.md) |
| ZK0303 | calling a function that does not exist | [Functions](../../learn/functions/README.md) |
| ZK0304 | two functions with one name | [Functions](../../learn/functions/README.md) |
| ZK0305 | two parameters of one function with one name | [Parameters](../../learn/functions/parameters.md) |
| ZK0306 | changing a constant | [Constants](../../learn/variables/constants.md) |
| ZK0307 | changing a counted loop's counter | [Counted loops](../../learn/control-flow/counted-loops.md) |
| ZK0308 | a constant without a value | [Constants](../../learn/variables/constants.md) |
| ZK0309 | a break outside a loop | [Break and continue](../../learn/control-flow/break-continue.md) |
| ZK0310 | a continue outside a loop | [Break and continue](../../learn/control-flow/break-continue.md) |
| ZK0311 | `ပြန်ပေးပါ` outside a function | [Functions](../../learn/functions/README.md) |
| ZK0312 | returning a value from a function without a result | [Results](../../learn/functions/results.md) |
| ZK0313 | returning without a value from a function with a result | [Results](../../learn/functions/results.md) |
| ZK0314 | the wrong number of arguments | [Parameters](../../learn/functions/parameters.md) |
| ZK0315 | using a function without a result as a value | [Results](../../learn/functions/results.md) |
| ZK0316 | a struct name declared twice | [Struct mistakes](../../learn/structs/mistakes.md#one-name-for-two-structs) |
| ZK0317 | a field name used twice in one struct | [Struct mistakes](../../learn/structs/mistakes.md#one-name-for-two-fields) |
| ZK0318 | a name in a kind that no struct declaration gives | [Struct mistakes](../../learn/structs/mistakes.md#a-name-that-is-no-struct) |
| ZK0319 | a struct name used for a variable, parameter, field or function | [Struct mistakes](../../learn/structs/mistakes.md#a-struct-name-that-is-also-a-value-name) |
| ZK0320 | a field the struct does not have | [Struct mistakes](../../learn/structs/mistakes.md#a-field-the-struct-does-not-have) |
| ZK0321 | a field set twice in a struct literal | [Struct mistakes](../../learn/structs/mistakes.md#one-field-set-twice) |

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
| ZK0410 | an invalid assignment target: not a variable or a dereference | [Changing a variable](../../learn/variables/assignment.md) |
| ZK0411 | no address: taking the address of something that is not a variable or a dereference | [Pointers and the heap](../../learn/pointers/README.md#what-has-an-address) |
| ZK0412 | taking the address of a fixed name: a constant or a counted-loop variable | [Pointers and the heap](../../learn/pointers/README.md#what-has-an-address) |
| ZK0413 | not a pointer: dereferencing a value that has no pointer kind | [Pointer mistakes](../../learn/pointers/mistakes.md) |
| ZK0414 | an opaque pointer (`လိပ်စာ`) dereferenced or offset before a conversion | [Pointer arithmetic](../../learn/pointers/arithmetic.md#reading-the-same-octets-as-another-kind) |
| ZK0415 | an operator not for pointers, such as `*` or `/` on an address | [Pointer arithmetic](../../learn/pointers/arithmetic.md#walking-with-a-pointer) |
| ZK0416 | pointer kinds differ: comparing or subtracting pointers to different kinds | [Pointer arithmetic](../../learn/pointers/arithmetic.md#walking-with-a-pointer) |
| ZK0417 | a heap statement that needs a pointer but was given another kind | [The heap](../../learn/pointers/heap.md) |
| ZK0419 | a list literal with more or fewer cells than its kind | [List mistakes](../../learn/lists/mistakes.md#a-literal-with-the-wrong-count) |
| ZK0420 | `စာရင်းအလွတ်` where no list kind is known | [List mistakes](../../learn/lists/mistakes.md#a-blank-list-with-no-kind) |
| ZK0421 | a list kind with no count where nothing gives one | [List mistakes](../../learn/lists/mistakes.md#a-list-kind-with-no-count) |
| ZK0422 | cells of a value that is no list | [List mistakes](../../learn/lists/mistakes.md#cells-of-a-value-that-is-no-list) |
| ZK0423 | a cell number that is not an integer | [Lists](../../learn/lists/README.md#cell-numbers) |
| ZK0424 | a list as a function result | [Passing and comparing lists](../../learn/lists/passing.md#lists-as-parameters) |
| ZK0425 | a list kind too large for the VM | [Lists of lists](../../learn/lists/grids.md#the-size-of-a-list) |
| ZK0426 | a field of a value that is no struct | [Struct mistakes](../../learn/structs/mistakes.md#a-field-of-a-value-that-is-no-struct) |
| ZK0427 | a struct that holds itself | [Struct mistakes](../../learn/structs/mistakes.md#a-struct-that-holds-itself) |
| ZK0428 | a struct as a function result | [Struct mistakes](../../learn/structs/mistakes.md#a-struct-as-a-function-result) |

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
| ZK0509 | a warning: a heap size counted with a word that is not the classifier of its kind | [Struct mistakes](../../learn/structs/mistakes.md#the-wrong-classifier) |

### ZK06: working out constants

| Code | Problem | See |
| --- | --- | --- |
| ZK0601 | a sum of constants that does not fit its kind | [Arithmetic](../../learn/operators/arithmetic.md) |
| ZK0602 | a constant zero as the divisor | [Arithmetic](../../learn/operators/arithmetic.md#dividing-integers) |
| ZK0603 | a constant cell number outside its list | [List mistakes](../../learn/lists/mistakes.md#a-cell-number-outside-the-list) |

### ZK07: ownership and borrowing

The check of [ownership and borrowing](../../learn/pointers/ownership.md). With `--mem-check warn`,
ZK0701 to ZK0704 and ZK0706 to ZK0712 are warnings and the program runs; with
`--mem-check off`, the compiler leaves them out. The others are errors in each mode. See
[Pointer checks](../compiler/checks.md).

| Code | Problem | See |
| --- | --- | --- |
| ZK0701 | using an owner after its block moved to another owner | [Ownership and borrowing](../../learn/pointers/ownership.md#one-owner-each) |
| ZK0702 | giving back or resizing a block through a borrow | [Pointer mistakes](../../learn/pointers/mistakes.md#giving-back-a-block-that-was-not-asked-for) |
| ZK0703 | a borrow kept in a heap block, behind a pointer, or in a top-level variable from a function | [Ownership and borrowing](../../learn/pointers/ownership.md#borrows-live-in-variables) |
| ZK0704 | giving back, resizing or moving a block while a borrow of it is still in use | [Pointer mistakes](../../learn/pointers/mistakes.md#using-a-block-after-giving-it-back) |
| ZK0705 | the address of a borrow | [Ownership and borrowing](../../learn/pointers/ownership.md#borrows-live-in-variables) |
| ZK0706 | a function that returns a borrow of its own variable | [Changing the caller's variables](../../learn/pointers/out-parameters.md#how-long-an-address-lasts) |
| ZK0707 | a call that can give back a block through one argument while another argument reaches it | [Pointer mistakes](../../learn/pointers/mistakes.md#mistakes-the-compiler-catches) |
| ZK0708 | a block lost before it is given back | [Pointer mistakes](../../learn/pointers/mistakes.md#losing-a-block) |
| ZK0709 | reading or writing through a pointer that is `ဗလာ` | [Pointer mistakes](../../learn/pointers/mistakes.md#reading-through-a-null-pointer) |
| ZK0710 | a box or cell number past the end of its block | [Pointer mistakes](../../learn/pointers/mistakes.md#reading-past-the-end) |
| ZK0711 | a warning, with `--mem-strict` alone: a pointer that can be `ဗလာ` | [Ownership and borrowing](../../learn/pointers/ownership.md#turning-the-checks-down) |
| ZK0712 | subtracting pointers into different blocks | [Pointer mistakes](../../learn/pointers/mistakes.md#mistakes-the-compiler-catches) |
| ZK0713 | `ငှားထားသော` in the wrong place in a pointer kind | [Ownership and borrowing](../../learn/pointers/ownership.md#three-kinds-of-pointer) |
| ZK0714 | a borrow given where an owner is wanted | [Ownership and borrowing](../../learn/pointers/ownership.md#lending-a-block) |
| ZK0715 | an owner given where a borrow is wanted | [Ownership and borrowing](../../learn/pointers/ownership.md#lending-a-block) |
| ZK0716 | a raw address lent with `တန်ဖိုးအငှား` | [Ownership and borrowing](../../learn/pointers/ownership.md#raw-addresses) |
| ZK0717 | a raw address kind under `--no-raw` | [Ownership and borrowing](../../learn/pointers/ownership.md#raw-addresses) |
| ZK0718 | a field that holds a borrow | [Struct mistakes](../../learn/structs/mistakes.md#a-borrow-in-a-field) |
