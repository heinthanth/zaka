---
title: Reserved words
description: Each word Zaka gives a meaning to, with an English reading.
---

# Reserved words

Zaka gives each word on this page a fixed meaning, so we cannot use these words as names
(identifiers). When a word is made of reserved words alone, such as `ထက်ကြီးသည်` or
`တန်ဖိုးကို`, the compiler splits it into those words. To use a reserved word as a name all
the same, we wrap it in braces (`{ကြီး}`), as [Names](../../learn/variables/names.md)
shows.

The names of the built-in data kinds (types), `ကိန်းပြည့်`, `ဒဿမကိန်း`, `စာလုံး`, `စာသား`,
`မှန်မှားတန်ဖိုး`, `ဘူလီယန်`, `စာရင်း` and `ဇယား`, are not reserved words. The compiler
knows them as names, so a file can declare a data kind under one of these names. A word
glued from these names and reserved words, such as `လုံးမြောက်စာလုံးမှ`, still splits.

Each table below gives the word, an English gloss in quotes, and what we use the word for,
with a link to the page that teaches it or states its rules.

## Particles

| Word | Reads as | Used in |
| --- | --- | --- |
| `သည်` | after the subject of a sentence; also a polite ending | [Particles](../../learn/sentences/particles.md) |
| `ကို` | after the object of a sentence | [Particles](../../learn/sentences/particles.md) |
| `နှင့်` | "and", "with": to join two values, and before the last item of a list in words (`က၊ ခ နှင့် ဂ`) | [Arithmetic](../../learn/operators/arithmetic.md), [Comparison](../../learn/operators/comparison.md), [Lists in words](../language/words.md#lists-in-words) |
| `၏` | "of": before a field, a cell, or the result word of a word operator | [Arithmetic](../../learn/operators/arithmetic.md) |
| `ဖြင့်` | "with", "through": before the divisor in a division, and after the number base of a literal | [Arithmetic](../../learn/operators/arithmetic.md), [Integers](../../learn/values/integers.md) |
| `ထက်` | "than": in the comparisons "greater" and "not greater" | [Comparison](../../learn/operators/comparison.md) |
| `အောက်` | "below": in the comparisons "less" and "not less" | [Comparison](../../learn/operators/comparison.md) |
| `ဟု` | "so" (quoting): after text or a sentence that we print, and before `ထားမည်` in an assignment | [Printing](../../learn/hello/printing.md), [Changing a variable](../../learn/variables/assignment.md) |
| `အဖြစ်` | "as", "into": after the target data kind of a conversion | [Converting between data kinds](../../learn/operators/conversion.md) |
| `ပါ`, `ပါသည်` | polite endings of a verb | [Particles](../../learn/sentences/particles.md) |
| `တို့ကို`, `တို့ဖြင့်` | after the last argument when a call has two or more | [Parameters](../../learn/functions/parameters.md) |
| `လက်ခံပြီး` | "accepting ... and": after the parameters in a function signature | [Parameters](../../learn/functions/parameters.md) |
| `ပြင်ပမှလာသော` (`ပြင်ပ`, `မှ`, `လာသော`) | "coming from outside": to declare an external function; with the name of a lib in place of `ပြင်ပ` (`"libm" မှလာသော`), to declare a C function; and before `ဒေတာအစုအဝေး`, to declare a struct that comes from outside the program | [Functions](../language/functions.md), [Marks after the head](../language/structs.md#marks-after-the-head) |
| `ပြင်ပမှ ခေါ်ဆိုနိုင်သော` (`ပြင်ပ`, `မှ`, `ခေါ်ဆိုနိုင်သော`) | "that code outside can call": after `"<C name>" အမည်ဖြင့်` ("under the name"), to export a function to C under that C name, from the lib that we build | [Functions C can call](../language/functions-exports.md), [A lib for C](../../learn/functions/c-lib.md) |
| `ပြင်ပမှ ယူသုံးနိုင်သော` (`ပြင်ပ`, `မှ`, `ယူသုံးနိုင်သော`) | "that code outside can take up": after `"<C name>" အမည်ဖြင့်`, to export a struct to C under that C name, from the lib that we build | [Marks after the head](../language/structs.md#marks-after-the-head) |
| `အခြားတန်ဖိုးများနှင့်အတူ` (`အခြားတန်ဖိုးများ`, `နှင့်အတူ`) | "together with other values": after the last parameter of a C function that accepts more arguments than it names, like `...` in C: `... လည်းကောင်း အခြားတန်ဖိုးများနှင့်အတူ လက်ခံပြီး` | [C functions](../language/functions.md) |
| `အပိုင်အနေဖြင့်` | "as its own" (owned): on a pointer parameter, so that the function takes over the pointer; on a result, `<kind> တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်`, so that the caller takes over the pointer, or the string a C function returns, which Zaka frees with libc's `free` | [Unsafe Zaka](../language/unsafe.md), [Functions](../language/functions.md) |
| `လုပ်ဆောင်ချက်တစ်ခု` | "a function": in the function signature of an external function | [Functions](../language/functions.md) |
| `သုံး၍` | "using": after the arguments of a call that we write before its function, after the preposition of a parameter (`ဝိဘတ် "<preposition>" ကို သုံး၍`), and after the phrases in a function signature | [Parameters](../../learn/functions/parameters.md#arguments-first) |
| `ထဲသို့` | "into", after the list or the text in `l ထဲသို့ v ကို ထည့်မည်` | [Adding and removing cells](../../learn/lists/growing.md) |
| `မှ`, `အထိ`, `စီ`, `အနေဖြင့်` | "from", "up to", "each", "as": in the head of a counted loop | [Counted loops](../../learn/control-flow/counted-loops.md) |
| `ဖြစ်ကာ`, `ဖြစ်ပြီး` | second spellings of `ဖြစ်၍`, to join the setup of a generic loop to its condition | [Generic loops](../../learn/control-flow/generic-loops.md) |
| `ပြီးလျှင်` | "when done": before the update statement at the end of a generic loop | [Generic loops](../../learn/control-flow/generic-loops.md) |
| `ဘစ်` | "bit": in integer data kinds such as `၈ ဘစ် ကိန်းပြည့်`, and in shifts | [Integers](../../learn/values/integers.md), [Bitwise operators](../../learn/operators/bitwise.md) |

## Verbs that make a value

| Word | Reads as | Operator | Page |
| --- | --- | --- | --- |
| `ပေါင်းလဒ်`, `နုတ်လဒ်`, `မြှောက်လဒ်` | "the sum", "the difference", "the product": after `က နှင့် ခ ၏`; on structs, each calls the function of the struct, as the sign does | `+`, `-`, `*` | [Arithmetic](../../learn/operators/arithmetic.md) |
| `စားသော စားလဒ်`, `စားသော စားကြွင်း` | "the quotient", "the remainder": after `က ကို ခ ဖြင့်` | `/`, `%` | [Arithmetic](../../learn/operators/arithmetic.md) |
| `ဘစ်အလိုက် ဆုံလဒ်`, `ဘစ်အလိုက် ပေါင်းစုလဒ်`, `ဘစ်အလိုက် ခြားနားလဒ်`, `ဘစ်အလိုက် ပြောင်းပြန်ပြောင်းလဒ်` | the bitwise operators: after `က နှင့် ခ ၏`, or after `က ၏` to flip the bits | `&`, `\|`, `^`, `~` | [Bitwise operators](../../learn/operators/bitwise.md) |
| `ဘယ်သို့`, `ညာသို့`, `ရွေ့သော ရလဒ်` | "to the left", "to the right", "shifted result": to shift bits, as in `က ကို ဘယ်သို့ n ဘစ် ရွေ့သောရလဒ်` | `<<`, `>>` | [Bitwise operators](../../learn/operators/bitwise.md) |
| `ညီ`, `မညီ` | "equal", "not equal": to compare contents, deep into lists, structs and shared objects | `==`, `!=` | [Comparison](../../learn/operators/comparison.md) |
| `တစ်ခုတည်း` | "one and the same": to test whether two shared objects, or two function values, are the same one, as in `X နှင့် Y သည် တစ်ခုတည်း ဖြစ်လျှင်` | | [Values and objects](../language/objects.md) |
| `ကြီး`, `မကြီး` | "greater", "not greater" | `>`, `<=` | [Comparison](../../learn/operators/comparison.md) |
| `ငယ်`, `မငယ်` | "less", "not less" | `<`, `>=` | [Comparison](../../learn/operators/comparison.md) |
| `ပြောင်းလဲသော ရလဒ်` | "changed result": to convert a value to another data kind | | [Converting between data kinds](../../learn/operators/conversion.md) |
| `ပြောင်းလဲသော ရလဒ်ကောင်း` | "good changed result": to convert through a `စမ်းသပ်ပြောင်းလဲ`, which can fail, and pass the failure on | | [Conversions](../language/conversions.md#conversions-through-traits) |
| `လုပ်ဆောင်သော ရလဒ်` | "result of running": to use the result of a long call as a value | | [Results](../../learn/functions/results.md) |
| `သည့် ရလဒ်` | "the result that ...": at the end of the short call `<arguments> ကို သုံး၍ <name> သည့် ရလဒ်`, to use its result as a value | | [Results](../../learn/functions/results.md#arguments-first-with-a-result) |
| `သည့် ရလဒ်ကောင်း` | "the good result that ...": the same short call, which passes a failure on | `?` after a call | [Functions](../language/functions.md) |
| `မည့် ရလဒ်`, `သော ရလဒ်` (`မည့်`, `သော`) | second spellings of `သည့် ရလဒ်`, each with its `ရလဒ်ကောင်း` form; we can glue each of these ends to the name of a function that the file sees, as in `ဖတ်သည့်ရလဒ်` | | [Functions](../language/functions-calls.md#arguments-first) |
| `လုပ်ဆောင်သော ရလဒ်ကောင်း` | "good result of running": to use the result of a long call and pass a failure on | `?` after a call | [Errors and results](../../learn/error-handling/README.md) |
| `ဖြစ်၍` (`ဖြစ်ပြီး`, `ဖြစ်ကာ`) | "and": the logical and; also between the data kind and the value in a declaration | `&&` | [Logic](../../learn/operators/logic.md), [Variables](../../learn/variables/README.md) |
| `သို့မဟုတ်` | "or": the logical or | `\|\|` | [Logic](../../learn/operators/logic.md) |
| `မဟုတ်` | "not": the logical not, after the value | `!` | [Logic](../../learn/operators/logic.md) |
| `အနုတ်` | "minus": to negate a value | `-` (in front) | [Arithmetic](../../learn/operators/arithmetic.md) |

`မိတ္တူ` ("duplicate") is not a reserved word, so we can use it as a name, but right after
`၏` it makes a duplicate (copy): `X ၏ မိတ္တူ` is a shallow duplicate of `X`, and
`X တစ်<word>လုံး ၏ မိတ္တူ` ("a duplicate of the whole of `X`", with `၏` joined or apart)
is a deep one. See [Values and objects](../language/objects.md).

## Verbs that make a statement

| Word | Reads as | Page |
| --- | --- | --- |
| `ပြသမည်` | "will show": to print a value; a call of the core module's function `ပြသ` with `မည်`, so not a reserved word | [Printing](../../learn/hello/printing.md) |
| `ထားမည်` | "will set": to give a variable a new value | [Changing a variable](../../learn/variables/assignment.md) |
| `ဖြစ်ပါစေ` | "let it be": at the end of a declaration | [Variables](../../learn/variables/README.md) |
| `လုပ်ဆောင်မည်` | "will run": at the end of a call statement and of the long arguments-first call | [Functions](../language/functions.md) |
| `မည်` | "will": at the end of the short call statement `<name> မည်` | [Functions](../language/functions.md) |
| `လက်ခံမည်` | "will accept": after the parameters of an external function with no result | [Functions](../language/functions.md) |
| `ပြန်ပေးမည်` | "will return": to return a value, `<value> ကို ပြန်ပေးမည်။`, and to define the result in a function signature, `<kind> တစ်ခု ကို ပြန်ပေးမည်` | [Results](../../learn/functions/results.md) |
| `ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်` | "return the error value called ...": to fail with an error value in a function that can fail, `<e> ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။` | [Errors and results](../language/error-handling.md) |
| `လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်` | "mark the function as succeeded": to end the function with a good result that has no value | [Writing result kinds](../../learn/error-handling/kinds.md) |
| `လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်` | "mark the function as not succeeded": to end the function with a failure that has no error value | [Writing result kinds](../../learn/error-handling/kinds.md) |
| `၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်` | "if it fails, return the error value": at the end of a call statement, to pass a failure on | [Errors and results](../../learn/error-handling/README.md) |
| `ရပ်မည်` | "will stop": to leave a loop (`break`), and at the end of the message of an assert, `မဟုတ်ပါက <text> ဟု ရပ်မည်` | [Break and continue](../../learn/control-flow/break-continue.md), [Asserts](../language/asserts.md#the-message) |
| `ထည့်မည်` | "will add": to add a value at the end of a list or text, `l ထဲသို့ v ကို ထည့်မည်`, or in front of cell `i`, `l ၏ အကွက် i ၏ အရှေ့တွင် v ကို ထည့်မည်`; a call of the core module's function `ထည့်` with `မည်`, so not a reserved word | [Adding and removing cells](../../learn/lists/growing.md) |
| `ဖျက်မည်` | "will remove": to remove the last cell of a list, `l ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်`; a call of the core module's function `ဖျက်` with `မည်`, so not a reserved word | [Adding and removing cells](../../learn/lists/growing.md) |
| `အစားထိုးမည်` | "will replace": to put `v` in place of cells `a` to `b` of text `t`, `t ၏ အကွက် a မှ b ထိကို v ဖြင့် အစားထိုးမည်`; a call of the core module's function `အစားထိုး` with `မည်`, so not a reserved word | [Text](../../learn/text/changing.md#replacing-part-of-text) |
| `ကျော်မည်` | "will skip": to go on to the next round of a loop (`continue`) | [Break and continue](../../learn/control-flow/break-continue.md) |
| `ခေါ်ဆိုခွင့် ပြုမည်` (`ခေါ်ဆိုခွင့်`, `ပြုမည်`) | "will grant leave to call": to let code outside the program, such as a C function, call the function value `f` after the call that passed it, as in `f ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်`; also written glued, `ခေါ်ဆိုခွင့်ပြုမည်` | [Closures and C](../language/closures.md#closures-and-c), [Callbacks and handles](../../learn/functions/c-callbacks.md#a-function-c-keeps) |
| `ခေါ်ဆိုခွင့် ပြန်လည်ရုပ်သိမ်းမည်` (`ခေါ်ဆိုခွင့်`, `ပြန်လည်ရုပ်သိမ်းမည်`) | "will take back leave to call": to end that leave, as in `f ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြန်လည်ရုပ်သိမ်းမည်` | [Closures and C](../language/closures.md#closures-and-c), [Callbacks and handles](../../learn/functions/c-callbacks.md#a-function-c-keeps) |
| `ထဲသို့ … ကို ကူးထည့်မည်` | "will duplicate … into": to put a duplicate (copy) into a variable, or to declare one after `ကိန်းရှင်`; for a shared object, a new object with the same fields (a shallow duplicate); a verb of the core module, as `ထည့်` is | [Values and objects](../language/objects.md) |
| `ထဲသို့ … ကို အကုန်ကူးထည့်မည်` | "will duplicate all of … into": to make a deep duplicate, which duplicates each shared object it reaches once | [Values and objects](../language/objects.md) |
| `အကုန်` | "all": before the verb `ကူးထည့်`, to make the duplicate deep, `အကုန်ကူးထည့်မည်` or `အကုန် ကူးထည့် မည်`; reserved, so we cannot use it as a variable name | [Values and objects](../language/objects.md) |

Each statement verb ends in `မည်`.

## Blocks and control flow

| Word | Reads as | Page |
| --- | --- | --- |
| `အောက်ပါတို့ကို လုပ်ဆောင်မည်` | "will do the following": to open a block, or the statements of a function, a loop or an `if` | [Blocks and scope](../../learn/variables/scope.md) |
| `အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်` | "will do the following, though it cannot be trusted": to open an unsafe block | [Unsafe Zaka](../language/unsafe.md) |
| `အောက်ပါတို့ကို လုပ်ဆောင်ကာ` | "doing the following, ...": before the result in a function signature, to open the statements of a function that returns a value | [Results](../../learn/functions/results.md) |
| `လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး` | "the tasks are done": the block closer, in four spellings | [Blocks and scope](../../learn/variables/scope.md) |
| `ပရိုဂရမ် ပြီးဆုံးပါပြီး` | "the program is done": to close the program start block, in four spellings | [Hello, world](../../learn/hello/README.md#where-the-program-starts) |
| `အကယ်၍` | "if": to start an `if` statement | [If and else](../../learn/control-flow/if-else.md) |
| `ကွန်ပိုင်းလုပ်ချိန်တွင်` | "at compile time": after `အကယ်၍`, for a condition that the compiler settles for the target | [Compile time](../language/compile-time.md) |
| `ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော` | "that can run at compile time": between `ဟူသော` and `လုပ်ဆောင်ချက်`, to declare a comptime function | [Compile time](../language/compile-time.md#comptime-functions) |
| `လုပ်ဆောင်နိုင်သော` | "that can run": the second word of the comptime mark | [Compile time](../language/compile-time.md#comptime-functions) |
| `လျှင်`, `ဖြစ်လျှင်` | "then": after the condition of an `if` and the cases of a switch arm | [If and else](../../learn/control-flow/if-else.md) |
| `ထိုသို့မဟုတ်ဘဲ` | "if not so, but": to start an else-if branch | [If and else](../../learn/control-flow/if-else.md) |
| `သို့မဟုတ်ပါက` | "otherwise": to start the else branch | [If and else](../../learn/control-flow/if-else.md) |
| `ဖြစ်ရမည်` | "must be so": after the condition of an assert, `<condition> ဖြစ်ရမည်` | [Asserts](../../learn/error-handling/asserts.md) |
| `မဟုတ်ပါက` | "if not": after `ဖြစ်ရမည်၊`, to give the message of an assert, `မဟုတ်ပါက <text> ဟု ရပ်မည်` | [Asserts](../language/asserts.md#the-message) |
| `ပရိုဂရမ်ရေးသားချိန်တွင်` | "at the time of writing the program": in front of an assert, to make a debug assert, which `--debug-asserts off` drops; also written `ပရိုဂရမ်ရေးသားချိန်၌` | [Asserts](../language/asserts.md#debug-asserts) |
| `လုပ်ဆောင်စရာများပြီးသည့် အခါတွင်` | "at the time the tasks are done": in front of a statement or a block, a cleanup that runs when the block around it ends; its words are `လုပ်ဆောင်စရာများ`, `ပြီးသည့်` and `အခါတွင်` (also written `အခါ၌`) | [Cleanup](../language/cleanup.md) |
| `မအောင်မြင်၍` | "as it failed": in front of the cleanup mark, a cleanup that runs at a failure alone; also written `မအောင်မြင်ပြီး` and `မအောင်မြင်ကာ` | [Cleanup](../language/cleanup.md#cleanup-on-failure) |
| `နေသရွေ့`, `ဖြစ်နေသရွေ့` | "as long as": after the condition of a while loop | [While loops](../../learn/control-flow/while.md) |
| `ရေတွက်၍`, `တိုး၍`, `လျှော့၍` | "counting", "increasing", "decreasing": at the end of the head of a counted loop | [Counted loops](../../learn/control-flow/counted-loops.md) |
| `အကွက်တိုင်း`, `စာလုံးတိုင်း`, `ယူ၍` | "each cell", "each character", "taking": in the head of a loop over each cell of a list or each character of text | [Lists](../../learn/lists/README.md#walking-a-list) |
| `ကွင်းဆက်` | "loop": in `ကွင်းဆက် ကို ရပ်မည်` (`break`) | [Break and continue](../../learn/control-flow/break-continue.md) |
| `နောက်တစ်ကြိမ်ဆီသို့` | "to the next round": in `နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်` (`continue`) | [Break and continue](../../learn/control-flow/break-continue.md) |

## Data kinds, declarations and values

| Word | Reads as | Page |
| --- | --- | --- |
| `လက္ခဏာရှိ`, `လက္ခဏာမဲ့` | "signed", "unsigned": before an integer data kind | [Integers](../../learn/values/integers.md) |
| `အခြေ` | "base": to give the number base of a literal, `အခြေ ၁၆ ဖြင့် ...` | [Integers](../../learn/values/integers.md) |
| `ကိန်းရှင်` | "variable": to declare a variable | [Variables](../../learn/variables/README.md) |
| `ကိန်းသေ` | "constant": to declare a constant | [Constants](../../learn/variables/constants.md) |
| `လုပ်ဆောင်ချက်` | "function": in a function declaration and a function value | [Functions](../../learn/functions/README.md) |
| `စမ်းသပ်ချက်`, `စမ်းသပ်မှု` | "test": to declare a test block (unit test), `<name> ဟူသော စမ်းသပ်ချက် သည် …`, which `zakac test` runs and a build leaves out; the two spellings are one word | [Tests](../language/tests.md) |
| `ပရိုဂရမ်` | "program": in `ပရိုဂရမ်သည်`, to open the program start block (entry point), where the program starts | [Where a program starts](../../learn/programs/start.md) |
| `ဒေတာ ပြောင်းလဲရန်` | "to change the data": right after `ကို` in a parameter, to make it a mutable parameter (`inout`), which the function can read and write; the changes go back to the caller's place when the call returns | [Mutable parameters](../language/functions-parameters.md#mutable-parameters) |
| `ဒေတာ ထည့်ပေးရန်` | "to put the data in": right after `ကို` in a parameter, to make it a writable parameter (`out`), which the function writes before it reads; the caller's place counts as set after the call | [Writable parameters](../language/functions-parameters.md#writable-parameters) |
| `ပြောင်းလဲရန်` | "in order to change": the second word of `ဒေတာ ပြောင်းလဲရန်`, with no meaning alone | [Mutable parameters](../language/functions-parameters.md#mutable-parameters) |
| `ထည့်ပေးရန်` | "in order to put in": the second word of `ဒေတာ ထည့်ပေးရန်`, with no meaning alone | [Writable parameters](../language/functions-parameters.md#writable-parameters) |
| `ဝိဘတ်` | "preposition": to give a parameter its preposition, with `ဝိဘတ် "<preposition>" ကို သုံး၍` last among the markers of the parameter | [Functions](../language/functions.md) |
| `အစ စကားစု` | "starting phrase": after `သည်` in a function signature, `အစ စကားစု "<words>"` declares the words that come before the first argument when we write the call as a sentence; glued or apart, and `အစ` alone is a name | [Starting and ending phrases](../language/functions-sentences.md#starting-and-ending-phrases) |
| `သုံးသော`, `သုံးသည့်` | "that uses": after the phrases of a function kind with no parameters and no result, right before `လုပ်ဆောင်ချက် အမျိုးအစား` | [Function kinds](../language/function-values.md#function-kinds) |
| `အဆုံး စကားစု` | "ending phrase": after `သည်` in a function signature, `အဆုံး စကားစု "<words>"` declares the words that come right before the function name when we write the call as a sentence; glued or apart, and `အဆုံး` alone is a name | [Starting and ending phrases](../language/functions-sentences.md#starting-and-ending-phrases) |
| `ဟူသော` | "called": after a name, to declare a parameter or a field (`<name> ဟူသော <kind> တစ်<word>`), a function, a struct or an enumeration; to use a function as a value (`<name> ဟူသော လုပ်ဆောင်ချက်`), or to start a long call with `ကို` after it; to make an error value (`<e> ဟူသော အမှားအယွင်း`); and after a number or a list literal, to give it a data kind | [Function values](../language/function-values.md), [Measure words](../language/kinds.md#measure-words) |
| `မည့်` | "named": a second spelling of `ဟူသော` before `လုပ်ဆောင်ချက်`, as in `<name> မည့် လုပ်ဆောင်ချက်ကို ... လုပ်ဆောင်မည်။` | [Functions](../language/functions-calls.md#the-word-forms) |
| `လည်းကောင်း` | "and also": after each parameter (after `ကို` and its markers) when a function has two or more | [Functions](../language/functions.md) |
| `လုပ်ဆောင်ချက် အမျိုးအစား` | "function kind": the last words of a function kind | [Function values](../language/function-values.md) |
| `လက်ခံသော`, `လက်ခံသည့်` | "that accepts": after the parameters of a function kind with no result | [Function values](../language/function-values.md) |
| `ပြန်ပေးသော`, `ပြန်ပေးသည့်` | "that returns": after the result of a function kind | [Function values](../language/function-values.md) |
| `ရလဒ်` | "result": the last word of the calls, conversions and shifts that make a value, such as `လုပ်ဆောင်သော ရလဒ်`; it is not a variable, and not part of a result kind or a return | [Results](../../learn/functions/results.md) |
| `ရလဒ်ကောင်း` | "good result": to use the result of a call that can fail and pass a failure on, `လုပ်ဆောင်သော ရလဒ်ကောင်း`; also in the test `x ထဲတွင် ရလဒ်ကောင်း ရှိလျှင်` | [Errors and results](../language/error-handling.md) |
| `ရှိကောင်းရှိနိုင်သော` | "that might contain": in an optional kind (option type), `<kind> ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` | [Optionals](../language/optionals.md) |
| `ရှိနိုင်သော` | "that can contain": near the end of a result kind, `<kind> သို့မဟုတ် အမှားအယွင်းအနေဖြင့် <kind> ရှိနိုင်သော ဒေတာဆဲလ်` | [Errors and results](../language/error-handling.md) |
| `အချက်အလက်ဆဲလ်`, `ဒေတာဆဲလ်` | "data cell": the box of an optional kind and of a result kind (the two spellings are one word) | [Optionals](../language/optionals.md) |
| `အလွတ်` | "blank": an optional with no data (none, null) | [Optionals](../language/optionals.md) |
| `ကနဦးတန်ဖိုး` | "the initial value": the default value of the data kind of its place, as in `... ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။`; also the first word of `ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်` | [Initial values](../language/kinds.md#initial-values), [The initial member](../language/enums.md#the-initial-member) |
| `အချက်အလက်`, `ဒေတာ` | "data": in the test `x ထဲတွင် ဒေတာ ရှိလျှင်` and the read `x ထဲမှ ဒေတာ`; reserved after `ထဲမှ`, `ထဲတွင်` and `အမှားအယွင်း` and before `ပြောင်းလဲရန်` and `ထည့်ပေးရန်`, and a plain name elsewhere, so a variable can be called `ဒေတာ` | [Optionals](../language/optionals.md) |
| `ထဲတွင်` | "inside": to test what an optional or a result contains, as in `x ထဲတွင် ဒေတာ ရှိလျှင်` | [Optionals](../language/optionals.md) |
| `ရှိနေ` | "is there": in the test `x ထဲတွင် ဒေတာ ရှိနေလျှင်` and the loop `x ထဲတွင် ဒေတာ ရှိနေသရွေ့` | [Optionals](../language/optionals.md) |
| `မရှိ`, `မရှိနေ` | "is not there": to test that an optional has no data, as in `x ထဲတွင် ဒေတာ မရှိလျှင်` | [Optionals](../language/optionals.md) |
| `မအောင်မြင်ပါက` | "if it fails": at the end of a call statement that passes a failure on, `၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်` | [Errors and results](../../learn/error-handling/README.md) |
| `အောင်မြင်သည့်သတင်း` | "news of success": the good side of a result kind, when it has no value | [Writing result kinds](../../learn/error-handling/kinds.md) |
| `မအောင်မြင်သည့်သတင်း` | "news of failure": the error side of a result kind, when it has no value | [Writing result kinds](../../learn/error-handling/kinds.md) |
| `အောင်မြင်သည်` | "it succeeded": in the phrase `လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်` | [Writing result kinds](../../learn/error-handling/kinds.md) |
| `မအောင်မြင်ပါ` | "it did not succeed": in the phrase `လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်` | [Writing result kinds](../../learn/error-handling/kinds.md) |
| `သတ်မှတ်မည်` | "will mark": the last word of the two phrases above | [Writing result kinds](../../learn/error-handling/kinds.md) |
| `အမှားအယွင်း` | "error value": in `<e> ဟူသော အမှားအယွင်း`, in the test `x ထဲတွင် အမှားအယွင်း ရှိလျှင်`, in the read `x ထဲမှ အမှားအယွင်း ဒေတာ`, and on the error side of a result kind | [Errors and results](../language/error-handling.md) |
| `အမှားအယွင်းအနေဖြင့်` | "as an error value": before the error kind in a result kind, `<kind> သို့မဟုတ် အမှားအယွင်းအနေဖြင့် <kind> ရှိနိုင်သော ဒေတာဆဲလ်` (also `အမှားအယွင်း အနေဖြင့်`) | [Errors and results](../language/error-handling.md) |
| `မှန်`, `မမှန်` | "true", "false": the two true/false (boolean) values | [True and false](../../learn/values/booleans.md) |
| `မှတ်ချက်` | "note": to start a comment that runs to the end of the line | [Comments](../../learn/hello/comments.md) |

## Unsafe code, pointers and the heap

The words of this section belong to unsafe code; see [Unsafe Zaka](../language/unsafe.md).

| Word | Reads as | Page |
| --- | --- | --- |
| `စိတ်မချရသော` | "that cannot be trusted": between `ဟူသော` and `လုပ်ဆောင်ချက်`, to make a function or a C function declaration unsafe | [Unsafe Zaka](../language/unsafe.md) |
| `စိတ်မချရသော်လည်း` | "though it cannot be trusted": in `အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်`, to open an unsafe block | [Unsafe Zaka](../language/unsafe.md) |
| `လိပ်စာ` | "address": after `<kind>`, to make a pointer kind (`<kind> လိပ်စာ`, then `၏ လိပ်စာ` for each further level); alone, the opaque pointer kind; after a name (`x ၏ လိပ်စာ`, or `&x`), to get the address of the name | [Unsafe Zaka](../language/unsafe.md) |
| `တန်ဖိုး` | "value": to read or write through a pointer, as in `p ၏ တန်ဖိုး` (or `*p`); also in `တန်ဖိုးသည်` in a declaration | [Unsafe Zaka](../language/unsafe.md), [Variables](../../learn/variables/README.md) |
| `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` | "the address that points at nothing": the null address (null pointer), the one NULL in Zaka; safe code can store it and compare with it | [The null address](../language/pointers.md#the-null-address) |
| `မည်သည်ကိုမှ` | "nothing": the first word of the null address | [The null address](../language/pointers.md#the-null-address) |
| `မညွှန်းသော` | "that points at no": the second word of the null address | [The null address](../language/pointers.md#the-null-address) |
| `အရွယ်အစား` | "size": to get the size of a data kind in octets (bytes), as in `<kind> တစ်<word> ၏ အရွယ်အစား` | [Unsafe Zaka](../language/unsafe.md) |
| `မင်မိုရီ`, `မှတ်ဉာဏ်` | "RAM" (two interchangeable spellings) | [Unsafe Zaka](../language/unsafe.md) |
| `ဘိုက်စာ` | "octets' worth": a size in octets, as in `n ဘိုက်စာ` | [Unsafe Zaka](../language/unsafe.md) |
| `ခုစာ`, `လုံးစာ` | "items' worth": a size in items of a data kind, as in `ကိန်းပြည့် n ခုစာ` and `စာလုံး n လုံးစာ`; a data kind that declares a measure word uses `<word>စာ` | [Unsafe Zaka](../language/unsafe.md) |

The heap sentences are calls of functions of the core module, so their verbs (`သိမ်းထား`,
`ပြန်လည်စွန့်လွှတ်`, `နေရာအချဲ့အချုံ့လုပ်`, `ဖြည့်`, `ကူးယူ`, `ပြောင်းထည့်`) are names, not
reserved words ([Heap statements](../language/heap.md#heap-statements)). The words inside
these sentences (`နေရာယူ၍`, `တွင်`, `ရှိ`, `ဖြစ်အောင်`, `ဆီသို့`) are still reserved.

## Lists

| Word | Reads as | Page |
| --- | --- | --- |
| `စာရင်း` | "list": after `<kind>`, to make a list kind (`<kind> စာရင်း`, or `<kind> ၏ စာရင်း`), a plain list when it has no count, through its [written-as hint](../language/generics.md#written-as-hints); also at the end of a list literal, `တို့ပါသော စာရင်း`; a name of the core module, not a reserved word | [Lists](../../learn/lists/README.md) |
| `အကွက်` | "cell" (element): to reach cell `i` of a list, `x ၏ အကွက် i`, counted from 0; and to give the count of a counted list kind, `အကွက် n ခုရှိသော` | [Lists](../../learn/lists/README.md) |
| `ခုမြောက်`, `လုံးမြောက်` | "-th": to reach cell `n` counted from 1, `x ၏ n ခုမြောက်အကွက်`, or character `n` of text, `t ၏ n ခုမြောက်စာလုံး` or `t ၏ n လုံးမြောက်စာလုံး` | [Lists](../../learn/lists/README.md#cell-numbers) |
| `ခုရှိသော`, `ရှိသော` | "that has": in a counted list kind, `အကွက် n ခုရှိသော <kind> စာရင်း` | [Static data and counted lists](../../learn/forms/counted-lists.md#counted-lists) |
| `အကွက်အရေအတွက်` | "cell count": the number of cells, `x ၏ အကွက်အရေအတွက်`; a function of the core module that we use after `၏`, not a reserved word | [Lists](../../learn/lists/README.md#cell-numbers) |
| `နောက်ဆုံးအကွက်` | "last cell": to remove the last cell of a list, `l ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်`, and reserved in that spot alone | [Adding and removing cells](../../learn/lists/growing.md) |
| `နောက်ဆုံးစာလုံး` | "last character": to remove the last character of text, `t ၏ နောက်ဆုံးစာလုံးကို ဖျက်မည်`, and reserved in that spot alone | [Text](../../learn/text/changing.md#adding-in-the-middle-and-removing-the-end) |
| `အရှေ့တွင်` | "in front of": to add a value in front of a cell or a character, `l ၏ အကွက် i ၏ အရှေ့တွင် v ကို ထည့်မည်` | [Adding and removing cells](../../learn/lists/growing.md#adding-in-the-middle) |
| `ဆက်ထားသည့်` | "that joins": to make new text from the pieces, `a နှင့် b တို့ကို ဆက်ထားသည့် စာသား` | [Text](../../learn/text/README.md#joining-text) |
| `စာလုံးအရေအတွက်` | "character count": the number of characters in text, `t ၏ စာလုံးအရေအတွက်`; a function of the core module, not a reserved word. Character `i`, counted from 0, is `t ၏ စာလုံး i` | [Text](../../learn/text/characters.md#characters) |
| `စာရင်းအလွတ်` | "blank list": a plain list with no cells; a counted list cannot use it (ZK0436) | [Lists](../../learn/lists/README.md) |
| `တို့ပါသော` | "that contain these": before `စာရင်း` in a list literal | [Lists](../../learn/lists/README.md#list-kinds) |
| `ပါသော` | "that contains": before `စာရင်း` or `ဇယား` in a literal of one cell or row | [Lists in words](../language/words.md#lists-in-words) |
| `မှစ၍` | "starting at": in a range, `x ၏ အကွက် <start> မှစ၍ <length> ကွက်`, to make a new list or text with duplicates of those cells | [Ranges](../language/lists.md#ranges) |
| `ထိ` | "up to": after the last cell or character of the range that a replace changes, `<start> မှ <end> ထိကို` | [Text](../../learn/text/changing.md#replacing-part-of-text) |
| `ကွက်` | "cells": after the length of a range; a struct can also declare it as its measure word | [Ranges](../language/lists.md#ranges) |

## Tables

| Word | Reads as | Page |
| --- | --- | --- |
| `ဇယား` | "table" (map): at the end of a table kind, `<index kind> ဖြင့် ညွှန်းသော <value kind> ဇယား`, through its [written-as hint](../language/generics.md#written-as-hints), and of a table literal, `တို့ပါသော ဇယား`; a name of the core module, not a reserved word | [Tables](../language/tables.md) |
| `ဇယားအလွတ်` | "blank table": a table with no rows, the same as `[:]` | [Tables](../language/tables.md#literals) |
| `ညွှန်းသော` | "indexed": after `<index kind> ဖြင့်` in a table kind; `မညွှန်းသော` of the null address is a separate word | [Tables](../language/tables.md) |
| `အညွှန်း` | "index" (key): to read the row of an index, `<table> ထဲမှ အညွှန်း <index>`, and its value, with `၏ တန်ဖိုး` after it | [Tables](../language/tables.md#reading-a-row) |
| `အညွှန်းအရေအတွက်` | "index count": the number of rows, `<table> ၏ အညွှန်းအရေအတွက်`; a function of the core module, not a reserved word | [Tables](../language/tables.md#counting-and-walking-the-rows) |
| `အညွှန်းတိုင်း`, `အတန်းတိုင်း` | "each index", "each row": in the loops over the rows of a table | [Tables](../language/tables.md#counting-and-walking-the-rows) |

We use `ထဲမှ` and `အနေဖြင့်` with tables too, in `<table> ထဲမှ အညွှန်း <index>` and in the row
`<index> အနေဖြင့် <value>` of a literal. After `ထဲမှ ဒေတာ၊`, we use `မရှိလျှင်` (`မရှိ`, `လျှင်`)
to give the fallback value of an optional ([Optional values](../language/optionals.md#reading)).

## Structs

| Word | Reads as | Page |
| --- | --- | --- |
| `အချက်အလက်အစုအဝေး`, `ဒေတာအစုအဝေး` | "data group": to declare a struct, `<name> ဟူသော ဒေတာအစုအဝေးတွင် ...`; `ဒေတာ` and `အချက်အလက်` are one word in each reserved word and name, and `တွင်` can be joined or apart | [Structs](../../learn/structs/README.md) |
| `တို့ပါဝင်သည်` (`တို့`, `ပါဝင်သည်`) | "these are included": at the end of the field list of a struct and of the member list of an enumeration | [Structs](../../learn/structs/README.md) |
| `တို့ပါဝင်ပြီး` (`တို့`, `ပါဝင်ပြီး`) | "these are included, and": at the end of the list when a measure word follows | [Structs](../../learn/structs/README.md#measure-words) |
| `မျှဝေနိုင်သော` | "that can be shared": after `ဟူသော`, in front of `ဒေတာအစုအဝေး`, to declare a shared object kind (reference type) instead of a struct | [Values and objects](../language/objects.md) |
| `မျိုးပြပစ္စည်းမှာ` | "the measure word is": to declare the measure word (classifier) of a struct, an enumeration or a kind alias | [Structs](../../learn/structs/README.md#measure-words) |
| `ဖြစ်သည်` | "is": at the end of a declaration that declares a measure word, and of a struct from outside the program with no fields | [Structs](../../learn/structs/README.md#measure-words) |

`တို့` alone is reserved too, so we cannot use it as a name. The words inside the two
closing phrases (`တို့`, `ပါဝင်သည်`, `ပါဝင်ပြီး`) are each reserved, and we can write the
phrases joined or with spaces.

## Enumerations and switches

| Word | Reads as | Page |
| --- | --- | --- |
| `အချက်အလက်အမျိုးအစားကွဲ`, `ဒေတာအမျိုးအစားကွဲ` | "data kind of variants": to declare an enumeration (enum), `<name> ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ...` | [Enumerations and switches](../../learn/enums/README.md) |
| `ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်` (`ကနဦးတန်ဖိုး`, `အနေဖြင့်`, `သုံးမည့်`) | "to be used as the initial value": in front of a member, to make it the initial member of an enumeration | [The initial member](../language/enums.md#the-initial-member) |
| `တန်ဖိုးအနေဖြင့်` (`တန်ဖိုး`, `အနေဖြင့်`) | "as its value": to give a member its value, `တန်ဖိုးအနေဖြင့် <value> ရှိသည့် <member>` | [Enumerations](../language/enums.md#declaring-an-enumeration) |
| `ရှိသည့်` | "that has": after the value of a member, and after the data kind of a member that holds data | [Enumerations](../language/enums.md#declaring-an-enumeration) |
| `ပြောင်းလဲနိုင်` | "can become": the verb of the test of a member, `<x> သည် <member> အဖြစ်သို့ ပြောင်းလဲနိုင်သည်`, which takes the endings of a comparison verb | [Members that hold data](../language/enums.md#testing-a-member) |
| `အဖြစ်သို့` | "into": after the member a test names | [Members that hold data](../language/enums.md#testing-a-member) |
| `သုံးမည့်` | "to be used": the last word of `ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်` | [The initial member](../language/enums.md#the-initial-member) |
| `ထဲမှ` | "from among": to name a member, `<name>များထဲမှ <member>` | [Enumerations and switches](../../learn/enums/README.md#naming-a-member) |
| `တခြား` | "another": to start the default arm of a switch, `တခြား <kind> တစ်<word> ဖြစ်လျှင်` | [Enumerations and switches](../../learn/enums/README.md#the-default-arm) |

`ဒေတာအနေဖြင့်`, before the data kind of a member that holds data, is `ဒေတာ` and `အနေဖြင့်`,
and `ဒေတာ` remains a name there, as it is outside the reads of an optional. A name for the
data of a member is written with `ဟူသည့်` ("called").

We declare an enumeration in the same shape as a struct, with `ဒေတာအမျိုးအစားကွဲ` in place
of `ဒေတာအစုအဝေး`, and end the member list with `တို့ပါဝင်သည်`, or with `ပါဝင်သည်` after a
single member. `အမျိုးအစား` alone is reserved too, as part of `လုပ်ဆောင်ချက် အမျိုးအစား`,
the last words of a function kind.

We can write `ထဲမှ` joined to the name before it, as in `အရောင်များထဲမှ`, because the lexer
splits a name that ends in `ထဲမှ` into the name and `ထဲမှ`, so no name can end in `ထဲမှ`.

## Kind aliases

| Word | Reads as | Page |
| --- | --- | --- |
| `အချက်အလက်အမျိုးအစား`, `ဒေတာအမျိုးအစား` | "data kind": to start a kind alias (type alias) | [Kind aliases](../language/aliases.md) |
| `ဟုလည်း မှတ်ယူနိုင်ပါသည်` | "can also be taken as": at the end of a kind alias, `ဒေတာအမျိုးအစား <alias> ကို <kind> ဟုလည်း မှတ်ယူနိုင်ပါသည်။` | [Kind aliases](../language/aliases.md) |
| `ဟုလည်း မှတ်ယူနိုင်ပြီး` | "can also be taken as, and": in a kind alias that declares its measure word, `... ဟုလည်း မှတ်ယူနိုင်ပြီး မျိုးပြပစ္စည်းမှာ <word> ဖြစ်သည်။` | [Kind aliases](../language/aliases.md) |

The words inside these endings, `လည်း` ("also"), `မှတ်ယူနိုင်ပါသည်` ("can be taken as") and
`မှတ်ယူနိုင်ပြီး` ("can be taken as, and"), are each reserved too, and we can write
`ဟုလည်း` as one word or as two.

## Generics and traits

| Word | Reads as | Page |
| --- | --- | --- |
| `ယေဘုယျဒေတာအမျိုးအစား`, `ယေဘုယျအချက်အလက်အမျိုးအစား` | "general data kind": to declare a kind parameter (type parameter) in the word form, `ယေဘုယျဒေတာအမျိုးအစား T` | [Generics](../../learn/generics/README.md) |
| `အစားထိုးသုံးနိုင်၍` | "can be put in place, and": after the kind parameters in the word form, `… ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်` | [Generics](../language/generics.md#kind-parameters) |
| `၎င်း` | "it": to refer back to the declaration after its kind parameters, `၎င်းသည်`, `၎င်းတွင်` or `၎င်းကို` | [Generics](../language/generics.md#kind-parameters) |
| `အစားထိုးအသုံးပြုသည့်` | "that puts in place": after the data kinds that we give a generic in the word form, before its name | [Generics](../language/generics.md#data-kinds-at-a-use) |
| `အစားထိုးအသုံးပြု၍` | "putting in place": after the data kinds of a call that we write arguments first | [Generics](../language/generics.md#calls) |
| `::` | before the data kinds that we give a generic in the sign form, `f::<ကိန်းပြည့်>(၁)` | [Generics](../language/generics.md#calls) |
| `ဟူသည့်` | "called": after the name of a trait in its declaration, `<name> ဟူသည့် ဂုဏ်သတ္တိ` | [Traits](../language/traits.md) |
| `ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော` | "included inside the compiler": on a data kind, a trait or a function of the core module, where an external function uses `ပြင်ပမှလာသော` | [The core module](../language/core.md#the-compiler-mark) |
| `ရေးသားရာတွင်` | "when written": to start the written-as hint of a generic struct, data kind or trait, which ends with `ဟု` | [Generics](../language/generics.md#written-as-hints) |
| `ရေးသားနိုင်သည်`, `ရေးသားနိုင်ပြီး` | "can be written", "can be written, and": at the end of a written-as hint, in a struct or a data kind and in the first line of a trait | [Generics](../language/generics.md#written-as-hints) |
| `အခြားလုပ်ဆောင်ချက်များကိုလည်း`, `မူရင်းအတိုင်း`, `ဆက်ခံမည်` | "the other functions too", "as in the original", "will inherit": in the last line of a trait block, to use the defaults of the trait, `<trait> ဂုဏ်သတ္တိ၏ အခြားလုပ်ဆောင်ချက်များကိုလည်း မူရင်းအတိုင်း ဆက်ခံမည်။` | [Traits](../language/traits.md#trait-blocks) |
| `ဂုဏ်သတ္တိ` | "trait": after the name of a trait, in its declaration and its last line, in a constraint and in a trait block | [Traits](../../learn/generics/traits.md) |
| `ပါဝင်ရမည်` | "must contain": at the end of the first line of a trait, `… အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။` | [Traits](../language/traits.md) |
| `လုပ်ဆောင်ချက်များ` | "functions": in the first and last lines of traits and trait blocks | [Traits](../language/traits.md) |
| `ရှိစေရန်` | "so that it has": in the first line of a trait block, `<kind> တွင် <trait> ဂုဏ်သတ္တိရှိစေရန်` | [Traits](../language/traits.md#trait-blocks) |
| `ထည့်သွင်းမည့်` | "to be added": the first word of the last line of a trait block | [Traits](../language/traits.md#trait-blocks) |
| `ဤနေရာတွင်`, `သတိပြုရန်မှာ` | "here", "to note": to open a notes block | [Generics](../language/generics.md#the-notes-block) |
| `သတိပြုစရာများ` | "the notes": the first word of the last line of a notes block | [Generics](../language/generics.md#the-notes-block) |
| `ရှိသည်` | "has": at the end of a constraint in a notes block, `A တွင် … ဂုဏ်သတ္တိ ရှိသည်။` | [Generics](../language/generics.md#the-notes-block) |

The names of the traits that the [core module](../language/core.md) declares are not reserved
words, so a name that a program declares hides the trait of the core module.

## Modules

| Word | Reads as | Page |
| --- | --- | --- |
| `အများသုံးနိုင်သော` | "for common use": to export a top-level declaration | [Modules](../../learn/modules/README.md#sharing-names) |
| `ထည့်သွင်းမည်` | "will bring in": at the end of an import, after `"<file>" မှ <name> ကို` or `"<file>" ကို` | [Modules](../../learn/modules/README.md#importing-names) |
| `အများသုံးနိုင်ရန်` | "for common use": in a share, after the names that the file passes on | [Modules](../language/modules.md#sharing-imported-names) |
| `ပြန်လည်` | "again": the first word of `ပြန်လည် မျှဝေမည်` | [Modules](../language/modules.md#sharing-imported-names) |
| `မျှဝေမည်` | "will share": at the end of a share, after `ပြန်လည်` | [Modules](../language/modules.md#sharing-imported-names) |

## Second spellings

Some parts of reserved words have second spellings with the same meaning, such as the verb
endings `၍`, `ပြီး` and `ကာ`, so `လက်ခံပြီး`, `လက်ခံ၍` and `လက်ခံကာ` are the same word.
Each spelling is reserved. [Words](../language/words.md#second-spellings) lists the
parts.

## Spoken spellings

Each reserved word below also has a spoken spelling, which is reserved too. A program can
use the spoken spellings once it allows them (ZK0510), as explained in
[Spoken Burmese](../language/spoken.md).

| Spoken | Written |
| --- | --- |
| `တယ်` | `သည်` |
| `နဲ့` | `နှင့်` |
| `လို့` | `ဟု` |
| `ကနေ` | `မှ` |
| `ပြီးရင်` | `ပြီးလျှင်` |
| `ပြောင်းလဲတဲ့` | `ပြောင်းလဲသော` |
| `ဟူတဲ့` | `ဟူသော`, `ဟူသည့်` |
| `မယ့်` | `မည့်` |
| `တဲ့` | `သော`, `သည့်` |
| `လုပ်ဆောင်တဲ့` | `လုပ်ဆောင်သော` |
| `လုပ်ဆောင်မယ်` | `လုပ်ဆောင်မည်` |
| `မယ်` | `မည်` |
| `လာတဲ့` | `လာသော` |
| `နဲ့အတူ` | `နှင့်အတူ` |
| `အခြားတန်ဖိုးများနဲ့အတူ` | `အခြားတန်ဖိုးများနှင့်အတူ` |
| `လက်ခံမယ်` | `လက်ခံမည်` |
| `လက်ခံတဲ့` | `လက်ခံသော` |
| `ပြန်ပေးတဲ့` | `ပြန်ပေးသော` |
| `ထားမယ်` | `ထားမည်` |
| `စားတဲ့` | `စားသော` |
| `ရွေ့တဲ့` | `ရွေ့သော` |
| `ပြန်ပေးမယ်` | `ပြန်ပေးမည်` |
| `ရပ်မယ်` | `ရပ်မည်` |
| `ကျော်မယ်` | `ကျော်မည်` |
| `မျှဝေမယ်` | `မျှဝေမည်` |
| `ပြန်လည်မျှဝေမယ်` | `ပြန်လည်မျှဝေမည်` |
| `ထည့်သွင်းမယ်` | `ထည့်သွင်းမည်` |
| `ကွန်ပိုင်းလုပ်ချိန်မှာ` | `ကွန်ပိုင်းလုပ်ချိန်တွင်` |
| `လုပ်ဆောင်နိုင်တဲ့` | `လုပ်ဆောင်နိုင်သော` |
| `ရင်` | `လျှင်` |
| `ဖြစ်ရင်` | `ဖြစ်လျှင်` |
| `ဖြစ်ရမယ်` | `ဖြစ်ရမည်` |
| `ပရိုဂရမ်ရေးသားချိန်မှာ` | `ပရိုဂရမ်ရေးသားချိန်တွင်` |
| `ပြီးတဲ့` | `ပြီးသည့်` |
| `အခါမှာ` | `အခါတွင်` |
| `အောင်မြင်တဲ့သတင်း` | `အောင်မြင်သည့်သတင်း` |
| `မအောင်မြင်တဲ့သတင်း` | `မအောင်မြင်သည့်သတင်း` |
| `ရှိကောင်းရှိနိုင်တဲ့` | `ရှိကောင်းရှိနိုင်သော` |
| `ရှိနိုင်တဲ့` | `ရှိနိုင်သော` |
| `ထဲမှာ` | `ထဲတွင်` |
| `အောင်မြင်တယ်` | `အောင်မြင်သည်` |
| `သတ်မှတ်မယ်` | `သတ်မှတ်မည်` |
| `မျှဝေနိုင်တဲ့` | `မျှဝေနိုင်သော` |
| `စိတ်မချရတဲ့` | `စိတ်မချရသော` |
| `မညွှန်းတဲ့` | `မညွှန်းသော` |
| `မှာ` | `တွင်` |
| `ပြုမယ်` | `ပြုမည်` |
| `ပြန်လည်ရုပ်သိမ်းမယ်` | `ပြန်လည်ရုပ်သိမ်းမည်` |
| `ခေါ်ဆိုခွင့်ပြုမယ်` | `ခေါ်ဆိုခွင့်ပြုမည်` |
| `ခေါ်ဆိုခွင့်ပြန်လည်ရုပ်သိမ်းမယ်` | `ခေါ်ဆိုခွင့်ပြန်လည်ရုပ်သိမ်းမည်` |
| `အရှေ့မှာ` | `အရှေ့တွင်` |
| `ဆက်ထားတဲ့` | `ဆက်ထားသည့်` |
| `ခုရှိတဲ့` | `ခုရှိသော` |
| `ရှိတဲ့` | `ရှိသော`, `ရှိသည့်` |
| `တို့ပါတဲ့` | `တို့ပါသော` |
| `ပါတဲ့` | `ပါသော` |
| `ကနေစ၍` | `မှစ၍` |
| `ကနေစပြီး`, `ကနေစကာ` | `မှစပြီး`, `မှစကာ` |
| `ညွှန်းတဲ့` | `ညွှန်းသော` |
| `အစားထိုးအသုံးပြုတဲ့` | `အစားထိုးအသုံးပြုသည့်` |
| `ပါဝင်ရမယ်` | `ပါဝင်ရမည်` |
| `ထည့်သွင်းမယ့်` | `ထည့်သွင်းမည့်` |
| `ဒီနေရာမှာ` | `ဤနေရာတွင်` |
| `ရှိတယ်` | `ရှိသည်` |
| `ကွန်ပိုင်လာထဲမှာ` | `ကွန်ပိုင်လာထဲတွင်` |
| `တစ်ပါတည်းပါဝင်တဲ့` | `တစ်ပါတည်းပါဝင်သော` |
| `သုံးမယ့်` | `သုံးမည့်` |
| `ပါဝင်တယ်` | `ပါဝင်သည်` |
| `ရေးသားရာမှာ` | `ရေးသားရာတွင်` |
| `ရေးသားနိုင်တယ်` | `ရေးသားနိုင်သည်` |
| `ဆက်ခံမယ်` | `ဆက်ခံမည်` |
| `ဖြစ်တယ်` | `ဖြစ်သည်` |
| `ထဲကနေ` | `ထဲမှ` |
| `မှတ်ယူနိုင်ပါတယ်` | `မှတ်ယူနိုင်ပါသည်` |
| `ရဲ့` | `၏` |

## Words a file reserves for itself

Two more rules reserve words in a file, from the functions that the file sees, and [What the flexible grammar costs](../language/functions-sentences.md#what-the-flexible-grammar-costs)
states them in full.

- Each word of a preposition of a function that the file declares or imports is read as
  that preposition wherever a preposition can go, and declaring a name with such a word
  gives the warning ZK0250. The stdlib modules add these words: `ဖိုင်ကို`, `ဖိုင်တွဲအတွင်းရှိ`, `အမည်များကို` and
  `နေရာတွင်` in a file that imports from `"စံကား/အဝင်အထွက်"`; the other modules use
  reserved words alone.
- A function name with a call end glued to it (`fမည်`, `fသည့်ရလဒ်`, `fမည့်ရလဒ်`,
  `fသောရလဒ်`, the `ရလဒ်ကောင်း` forms, `fသည်`, `fပါ`, and the spoken `fမယ်`,
  `fတယ်`, `fမယ့်ရလဒ်`, `fတဲ့ရလဒ်`) is read as a call of the function `f` that the
  file sees, and declaring such a name gives the warning ZK0251. Each file sees the verbs
  of the core module, so `ပြသမည်` and `ထည့်မည်` cannot be names.

A name in braces, `{name}`, is read as neither.
