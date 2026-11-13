---
title: Reserved words
description: Each word Zaka gives a meaning to, with an English reading.
---

# Reserved words

These words have a fixed meaning in Zaka, so a program cannot use them as names. A word
made of nothing but reserved words, such as `ထက်ကြီးသည်` or `တန်ဖိုးကို`, is split into
them. To use one of these as a name all the same, wrap it in backticks (`` `ကြီး` ``);
see [Names](../../learn/variables/names.md).

The names of the built-in kinds, `ကိန်းပြည့်`, `ဒဿမကိန်း`, `စာလုံး`, `စာသား`,
`မှန်မှားတန်ဖိုး` and `ဘူလီယန်`, are not reserved words: the compiler knows them as names,
and a file can declare a kind of its own under one of them. A word glued from them and reserved words, such as
`လုံးမြောက်စာလုံးမှ`, still splits.

## Particles

| Word | Reads as | Used in |
| --- | --- | --- |
| `သည်` | (subject); also a polite ending | [Particles](../../learn/sentences/particles.md) |
| `ကို` | (object) | [Particles](../../learn/sentences/particles.md) |
| `နှင့်` | "and", "with"; before the last item of a list in words (`က၊ ခ နှင့် ဂ`) | [Arithmetic](../../learn/operators/arithmetic.md), [Comparison](../../learn/operators/comparison.md), [Lists in words](../language/words.md#lists-in-words) |
| `၏` | "of" | [Arithmetic](../../learn/operators/arithmetic.md) |
| `ဖြင့်` | "with", "through" | [Arithmetic](../../learn/operators/arithmetic.md), [Integers](../../learn/values/integers.md) |
| `ထက်` | "than" | [Comparison](../../learn/operators/comparison.md) |
| `အောက်` | "below" | [Comparison](../../learn/operators/comparison.md) |
| `ဟု` | "so" (quoting) | [Printing](../../learn/hello/printing.md), [Changing a variable](../../learn/variables/assignment.md) |
| `အဖြစ်` | "as", "into" | [Converting between kinds](../../learn/operators/conversion.md) |
| `ပါ`, `ပါသည်` | polite endings | [Particles](../../learn/sentences/particles.md) |
| `တို့ကို`, `တို့ဖြင့်` | close a list of arguments | [Parameters](../../learn/functions/parameters.md) |
| `လက်ခံပြီး` | "accepting ... and" | [Parameters](../../learn/functions/parameters.md) |
| `ပြင်ပမှလာသော` (`ပြင်ပ`, `မှ`, `လာသော`) | "coming from outside": marks an external function; with a text of a lib in place of `ပြင်ပ` (`"libm" မှလာသော`), a C function; before `ဒေတာအစုအဝေး` in `<name> ဟူသော ပြင်ပမှ လာသော ဒေတာအစုအဝေး`, a kind from outside the program, whose declaration ends in `ဖြစ်သည်` or in `တွင် <fields> တို့ပါဝင်သည်` | [Functions](../language/functions.md), [C functions](../language/functions.md) |
| `အခြားတန်ဖိုးများနှင့်အတူ` (`အခြားတန်ဖိုးများ`, `နှင့်အတူ`) | "together with other values": `... လည်းကောင်း အခြားတန်ဖိုးများနှင့်အတူ လက်ခံပြီး` ends the parameters of a C function that takes more arguments than it names, as a C `...` does | [C functions](../language/functions.md) |
| `အပိုင်အနေဖြင့်` | "as its own": after `ကို` in a parameter, after `ဒေတာ ပြောင်းလဲရန်` when both stand, a pointer parameter the function takes over; in `<kind> တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်`, a pointer result the caller takes over, or the string a C function hands over, which Zaka frees with libc's `free` | [Unsafe Zaka](../language/unsafe.md), [Functions](../language/functions.md) |
| `လုပ်ဆောင်ချက်တစ်ခု` | "a function", in the header of an external function | [Functions](../language/functions.md) |
| `သုံး၍` | "using": after `ကို` or `တို့ကို`, closes the arguments of a call written before its function; in `ဝိဘတ် "<mark>" ကို သုံး၍`, closes the mark of a parameter | [Parameters](../../learn/functions/parameters.md#arguments-first) |
| `ထဲသို့` | "into", after the list or the text in `l ထဲသို့ v ကို ထည့်မည်` | [Adding and removing cells](../../learn/lists/growing.md) |
| `မှ`, `အထိ`, `စီ`, `အနေဖြင့်` | "from", "up to", "each", "as" | [Counted loops](../../learn/control-flow/counted-loops.md) |
| `ဖြစ်ကာ`, `ဖြစ်ပြီး` | "being ... and" | [Generic loops](../../learn/control-flow/generic-loops.md) |
| `ပြီးလျှင်` | "when done" | [Generic loops](../../learn/control-flow/generic-loops.md) |
| `ဘယ်သို့`, `ညာသို့` | "to the left", "to the right" | [Bitwise operators](../../learn/operators/bitwise.md) |
| `ဘစ်` | "bit" | [Integers](../../learn/values/integers.md), [Bitwise operators](../../learn/operators/bitwise.md) |

## Verbs that make a value

| Word | Reads as | Operator | Page |
| --- | --- | --- | --- |
| `ပေါင်းလဒ်` | "sum" | `+` | [Arithmetic](../../learn/operators/arithmetic.md) |
| `နုတ်လဒ်` | "difference" | `-` | [Arithmetic](../../learn/operators/arithmetic.md) |
| `မြှောက်လဒ်` | "product" | `*` | [Arithmetic](../../learn/operators/arithmetic.md) |
| `စားသော စားလဒ်` | "quotient" | `/` | [Arithmetic](../../learn/operators/arithmetic.md) |
| `စားသော စားကြွင်း` | "remainder" | `%` | [Arithmetic](../../learn/operators/arithmetic.md) |
| `ဘစ်အလိုက် ဆုံလဒ်` | "bitwise meeting" | `&` | [Bitwise operators](../../learn/operators/bitwise.md) |
| `ဘစ်အလိုက် ပေါင်းစုလဒ်` | "bitwise union" | `\|` | [Bitwise operators](../../learn/operators/bitwise.md) |
| `ဘစ်အလိုက် ခြားနားလဒ်` | "bitwise difference" | `^` | [Bitwise operators](../../learn/operators/bitwise.md) |
| `ဘစ်အလိုက် ပြောင်းပြန်ပြောင်းလဒ်` | "bitwise flip" | `~` | [Bitwise operators](../../learn/operators/bitwise.md) |
| `ရွေ့သောရလဒ်` | "moved result" | `<<`, `>>` | [Bitwise operators](../../learn/operators/bitwise.md) |
| `ညီ`, `မညီ` | "equal", "not equal": compares contents, deep into lists, structs and shared objects | `==`, `!=` | [Comparison](../../learn/operators/comparison.md) |
| `တစ်ခုတည်း` | "one and the same": `X နှင့် Y သည် တစ်ခုတည်း ဖြစ်လျှင်` asks whether two shared objects, or two function values, are one | | [Values and objects](../language/objects.md) |
| `ကြီး`, `မကြီး` | "greater", "not greater" | `>`, `<=` | [Comparison](../../learn/operators/comparison.md) |
| `ငယ်`, `မငယ်` | "less", "not less" | `<`, `>=` | [Comparison](../../learn/operators/comparison.md) |
| `ပြောင်းလဲသော ရလဒ်` | "changed result" | | [Converting between kinds](../../learn/operators/conversion.md) |
| `လုပ်ဆောင်သော ရလဒ်` | "result of running" | | [Results](../../learn/functions/results.md) |
| `သည့် ရလဒ်` | "the result that ...": ends the short call `<arguments> ကို သုံး၍ <name> သည့် ရလဒ်` | | [Results](../../learn/functions/results.md#arguments-first-with-a-result) |
| `သည့် ရလဒ်ကောင်း` | "the good result that ...": the short call passes a failure on | `?` after a call | [Functions](../language/functions.md) |
| `လုပ်ဆောင်သော ရလဒ်ကောင်း` | "good result of running": the call passes a failure on | `?` after a call | [Errors and results](../../learn/error-handling/README.md) |
| `ဖြစ်၍` | "and"; also joins the kind and the value in a declaration | `&&` | [Logic](../../learn/operators/logic.md), [Variables](../../learn/variables/README.md) |
| `သို့မဟုတ်` | "or" | `\|\|` | [Logic](../../learn/operators/logic.md) |
| `မဟုတ်` | "not" | `!` | [Logic](../../learn/operators/logic.md) |
| `အနုတ်` | "minus" | `-` (in front) | [Arithmetic](../../learn/operators/arithmetic.md) |

`မိတ္တူ` ("duplicate") is no reserved word, so it can be a name, but right after `၏` it
makes a value: `X ၏ မိတ္တူ` is a shallow duplicate of `X`, and `X တစ်<classifier>လုံး ၏ မိတ္တူ`
("a duplicate of the whole of `X`", with `၏` joined or not) a deep one. See [Values and
objects](../language/objects.md).

## Verbs that make a statement

| Word | Reads as | Page |
| --- | --- | --- |
| `ပြသမည်` | "will show" | [Printing](../../learn/hello/printing.md) |
| `ထားမည်` | "will set" | [Changing a variable](../../learn/variables/assignment.md) |
| `ဖြစ်ပါစေ` | "let it be" | [Variables](../../learn/variables/README.md) |
| `လုပ်ဆောင်မည်` | "will run": ends a call statement, and the long arguments-first call | [Functions](../language/functions.md) |
| `မည်` | "will": ends the short call statement `<name> မည်` | [Functions](../language/functions.md) |
| `လက်ခံမည်` | "will accept": ends the parameters of an external function with no result | [Functions](../language/functions.md) |
| `ပြန်ပေးမည်` | "will give back": ends a return, `<value> ကို ပြန်ပေးမည်။`, and the result clause of a header, `<kind> တစ်ခု ကို ပြန်ပေးမည်` | [Results](../../learn/functions/results.md) |
| `ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်` | "give back the error value called ...": `<e> ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။` in a function that can fail | [Errors and results](../language/error-handling.md) |
| `လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်` | "mark the function as succeeded": ends it with a good result that holds no value | [Writing result kinds](../../learn/error-handling/kinds.md) |
| `လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်` | "mark the function as not succeeded": ends it with a failure that holds no error value | [Writing result kinds](../../learn/error-handling/kinds.md) |
| `၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်` | "if it fails, give back the error value": ends a call statement that passes a failure on | [Errors and results](../../learn/error-handling/README.md) |
| `ရပ်မည်` | "will stop" | [Break and continue](../../learn/control-flow/break-continue.md) |
| `ထည့်မည်` | "will add": `l ထဲသို့ v ကို ထည့်မည်` adds `v` as a new last cell of `l`, or, when `l` is text, adds the text or character `v` at its end; `l ၏ အကွက် i ၏ အရှေ့တွင် v ကို ထည့်မည်` puts it in front of cell `i` | [Adding and removing cells](../../learn/lists/growing.md) |
| `ဖျက်မည်` | "will remove": `l ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်` removes the last cell of `l` | [Adding and removing cells](../../learn/lists/growing.md) |
| `အစားထိုးမည်` | "will replace": `t ၏ အကွက် a မှ b ထိကို v ဖြင့် အစားထိုးမည်` puts `v` in place of cells `a` to `b` of text `t` | [Text](../../learn/text/changing.md#replacing-part-of-text) |
| `ကျော်မည်` | "will skip" | [Break and continue](../../learn/control-flow/break-continue.md) |
| `ခေါ်ဆိုခွင့် ပြုမည်` (`ခေါ်ဆိုခွင့်`, `ပြုမည်`) | "will grant leave to call": `f ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်` gives code outside the program, such as a C function, leave to call the function value `f` after the call that gave it. The fused spelling `ခေါ်ဆိုခွင့်ပြုမည်` is the same | [Function values](../language/function-values.md#nested-functions-and-closures), [Calling C](../../learn/functions/c.md) |
| `ခေါ်ဆိုခွင့် ပြန်လည်ရုပ်သိမ်းမည်` (`ခေါ်ဆိုခွင့်`, `ပြန်လည်ရုပ်သိမ်းမည်`) | "will take back leave to call": `f ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြန်လည်ရုပ်သိမ်းမည်` ends that leave | [Function values](../language/function-values.md#nested-functions-and-closures), [Calling C](../../learn/functions/c.md) |
| `ထဲသို့ … ကို ကူးထည့်မည်` | "will duplicate … into": puts a duplicate into a variable, or declares one after `ကိန်းရှင်`; on a shared object, a new object with the same fields (a shallow duplicate) | [Values and objects](../language/objects.md) |
| `ထဲသို့ … ကို အကုန်ကူးထည့်မည်` | "will duplicate all of … into": a deep duplicate, which duplicates each shared object it reaches once | [Values and objects](../language/objects.md) |
| `အကုန်` | "all": the word `အကုန်ကူးထည့်မည်` starts with, written apart from `ကူးထည့်မည်`; reserved, so it cannot name a variable | [Values and objects](../language/objects.md) |
| `ထဲသို့ … ကို ပြောင်းထည့်မည်` | "will move … into": puts the value into a variable and leaves its old place blank, or declares one after `ကိန်းရှင်` | [Statements](../language/statements.md) |

Each statement verb ends in `မည်`. The older spellings that end in `ပါ` (`ပြသပါ`, `ထားပါ`,
`ပြန်ပေးပါ`, `လုပ်ဆောင်ပါ`, `ရပ်ပါ`, `ကျော်ပါ`, `ထည့်သွင်းပါ`, `ထည့်ပါ`, `ဖျက်ပါ`,
`အစားထိုးပါ`) are no words of the language: the lexer reports ZK0110, with the `မည်` spelling
in its help line, and reads on as if the `မည်` spelling stood there.

## Blocks and control flow

| Word | Reads as | Page |
| --- | --- | --- |
| `အောက်ပါတို့ကို လုပ်ဆောင်မည်` | "will do the following": opens a block, and the bodies of functions, loops and `if` | [Blocks and scope](../../learn/variables/scope.md) |
| `အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်` | "will do the following, though it cannot be trusted": opens an unsafe block | [Unsafe Zaka](../language/unsafe.md) |
| `အောက်ပါတို့ကို လုပ်ဆောင်ကာ` | "doing the following, ..." | [Results](../../learn/functions/results.md) |
| `လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး` | "the tasks are done" (four spellings) | [Blocks and scope](../../learn/variables/scope.md) |
| `ပရိုဂရမ် ပြီးဆုံးပါပြီး` | "the program is done": closes the start block (four spellings) | [Hello, world](../../learn/hello/README.md#where-the-program-starts) |
| `အကယ်၍` | "if" | [If and else](../../learn/control-flow/if-else.md) |
| `လျှင်`, `ဖြစ်လျှင်` | "then" | [If and else](../../learn/control-flow/if-else.md) |
| `ထိုသို့မဟုတ်ဘဲ` | "if not so, but" (else if) | [If and else](../../learn/control-flow/if-else.md) |
| `သို့မဟုတ်ပါက` | "otherwise" (else) | [If and else](../../learn/control-flow/if-else.md) |
| `နေသရွေ့`, `ဖြစ်နေသရွေ့` | "as long as" | [While loops](../../learn/control-flow/while.md) |
| `ရေတွက်၍`, `တိုး၍`, `လျှော့၍` | "counting", "increasing", "decreasing" | [Counted loops](../../learn/control-flow/counted-loops.md) |
| `အကွက်တိုင်း`, `စာလုံးတိုင်း`, `ယူ၍` | "each cell", "each character", "taking" | [Lists](../../learn/lists/README.md#walking-a-list) |
| `ကွင်းဆက်` | "loop" | [Break and continue](../../learn/control-flow/break-continue.md) |
| `နောက်တစ်ကြိမ်ဆီသို့` | "to the next round" | [Break and continue](../../learn/control-flow/break-continue.md) |

## Kinds, declarations and values

| Word | Reads as | Page |
| --- | --- | --- |
| `လက္ခဏာရှိ`, `လက္ခဏာမဲ့` | "signed", "unsigned" | [Integers](../../learn/values/integers.md) |
| `အခြေ` | "base" | [Integers](../../learn/values/integers.md) |
| `ကိန်းရှင်` | "variable" | [Variables](../../learn/variables/README.md) |
| `ကိန်းသေ` | "constant" | [Constants](../../learn/variables/constants.md) |
| `လုပ်ဆောင်ချက်` | "function" | [Functions](../../learn/functions/README.md) |
| `ပရိုဂရမ်` | "program": `ပရိုဂရမ်သည်` opens the start block, where the program starts | [Where a program starts](../../learn/programs/start.md) |
| `ဒေတာ ပြောင်းလဲရန်` | "to change the data": right after `ကို` in a parameter, a parameter the function can read and write; the caller sets the place first, and the changes go back to it when the call returns. A function kind takes it too | [Changed parameters](../language/functions.md#changed-parameters) |
| `ဒေတာ ထည့်ပေးရန်` | "to put the data in": right after `ကို` in a parameter, a parameter the function writes and never reads first; the caller need not set the place, and it counts as set after the call. A function kind does not take it | [Filled parameters](../language/functions.md#filled-parameters) |
| `ပြောင်းလဲရန်` | "in order to change": the second word of the phrase `ဒေတာ ပြောင်းလဲရန်`; alone it marks nothing | [Changed parameters](../language/functions.md#changed-parameters) |
| `ထည့်ပေးရန်` | "in order to put in": the second word of the phrase `ဒေတာ ထည့်ပေးရန်`; alone it marks nothing | [Filled parameters](../language/functions.md#filled-parameters) |
| `ဝိဘတ်` | "particle": `ဝိဘတ် "<mark>" ကို သုံး၍`, last among the markers of a parameter, gives the parameter its mark | [Functions](../language/functions.md) |
| `ဟူသော` | "called": after a name or a literal, the word after it decides. `<name> ဟူသော လုပ်ဆောင်ချက်`, "the function called `<name>`", is the function as a value, and with `ကို` and no verb after it, the head of a long call; `<e> ဟူသော အမှားအယွင်း` is an error value; `<name> ဟူသော <kind> တစ်<word>` names a parameter or a field; `<name> ဟူသော ... ဒေတာအစုအဝေး` and `... ဒေတာအမျိုးအစားကွဲ` open a struct and an enumeration; `<literal> ဟူသော <kind> တစ်<word>` gives a number or a list literal its kind | [Function values](../language/function-values.md), [Measure words](../language/kinds.md#measure-words) |
| `မည့်` | "named": the same as `ဟူသော` before `လုပ်ဆောင်ချက်`, as in `<name> မည့် လုပ်ဆောင်ချက်ကို ... လုပ်ဆောင်မည်။` | [Functions](../language/functions.md#calls) |
| `လည်းကောင်း` | "and also": ends each parameter, after `ကို` and its markers, when there are two or more | [Functions](../language/functions.md) |
| `လုပ်ဆောင်ချက် အမျိုးအစား` | "function kind": closes a function kind | [Function values](../language/function-values.md) |
| `လက်ခံသော` | "that accepts": ends the parameters of a function kind with no result | [Function values](../language/function-values.md) |
| `ပြန်ပေးသော` | "that gives back": ends the result of a function kind | [Function values](../language/function-values.md) |
| `ရလဒ်` | "result": the last word of the call, conversion, shift and arguments-first forms, such as `လုပ်ဆောင်သော ရလဒ်`. It is no variable, no part of a result kind and no part of a return | [Results](../../learn/functions/results.md) |
| `ရလဒ်ကောင်း` | "good result": of a call that can fail, `လုပ်ဆောင်သော ရလဒ်ကောင်း`; also the test `x ထဲတွင် ရလဒ်ကောင်း ရှိလျှင်` | [Errors and results](../language/error-handling.md) |
| `ရှိကောင်းရှိနိုင်သော` | "that might contain": `<kind> ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` is an optional kind | [Optionals](../language/optionals.md) |
| `ရှိနိုင်သော` | "that can hold": ends a result kind, `<kind> သို့မဟုတ် အမှားအယွင်းအနေဖြင့် <kind> ရှိနိုင်သော ဒေတာဆဲလ်` | [Errors and results](../language/error-handling.md) |
| `အချက်အလက်ဆဲလ်`, `ဒေတာဆဲလ်` | "data cell": the cell that an optional kind and a result kind hold (the spellings are one word) | [Optionals](../language/optionals.md) |
| `အလွတ်` | "blank": the optional that holds no data | [Optionals](../language/optionals.md) |
| `ကနဦးတန်ဖိုး` | "the initial value": the written starting value of the kind its place takes, as in `... ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။`; the first word of `ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်` | [Initial values](../language/kinds.md#initial-values), [The initial member](../language/enums.md#the-initial-member) |
| `အချက်အလက်`, `ဒေတာ` | "data": in the tests `x ထဲတွင် ဒေတာ ရှိလျှင်` and the read `x ထဲမှ ဒေတာ`; a reserved word after `ထဲမှ`, `ထဲတွင်` and `အမှားအယွင်း`, and before `ပြောင်းလဲရန်` and `ထည့်ပေးရန်`, alone; elsewhere it is a name, so a variable can be called `ဒေတာ` | [Optionals](../language/optionals.md) |
| `ထဲတွင်` | "inside": `x ထဲတွင် ဒေတာ ရှိလျှင်` tests what an optional or a result holds | [Optionals](../language/optionals.md) |
| `ရှိနေ` | "is there": the test and the loop, `x ထဲတွင် ဒေတာ ရှိနေလျှင်`, `x ထဲတွင် ဒေတာ ရှိနေသရွေ့` | [Optionals](../language/optionals.md) |
| `မရှိ`, `မရှိနေ` | "is not there": `x ထဲတွင် ဒေတာ မရှိလျှင်` tests that an optional holds no data | [Optionals](../language/optionals.md) |
| `မအောင်မြင်ပါက` | "if it fails": in the trailer of a call statement, `၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်` | [Errors and results](../../learn/error-handling/README.md) |
| `အောင်မြင်သည့်သတင်း` | "news of success": the good side of a result kind that holds no value | [Writing result kinds](../../learn/error-handling/kinds.md) |
| `မအောင်မြင်သည့်သတင်း` | "news of failure": the error side of a result kind that holds no value | [Writing result kinds](../../learn/error-handling/kinds.md) |
| `အောင်မြင်သည်` | "it succeeded": in the phrase `လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်` | [Writing result kinds](../../learn/error-handling/kinds.md) |
| `မအောင်မြင်ပါ` | "it did not succeed": in the phrase `လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်` | [Writing result kinds](../../learn/error-handling/kinds.md) |
| `သတ်မှတ်မည်` | "will mark": ends the two phrases above | [Writing result kinds](../../learn/error-handling/kinds.md) |
| `အမှားအယွင်း` | "error value": in `<e> ဟူသော အမှားအယွင်း`; also the test `x ထဲတွင် အမှားအယွင်း ရှိလျှင်`, the read `x ထဲမှ အမှားအယွင်း ဒေတာ`, and the error side of a result kind | [Errors and results](../language/error-handling.md) |
| `အမှားအယွင်းအနေဖြင့်` | "as an error value": in a result kind, `<kind> သို့မဟုတ် အမှားအယွင်းအနေဖြင့် <kind> ရှိနိုင်သော ဒေတာဆဲလ်` (also `အမှားအယွင်း အနေဖြင့်`) | [Errors and results](../language/error-handling.md) |
| `မှန်`, `မမှန်` | "true", "false" | [True and false](../../learn/values/booleans.md) |
| `မှတ်ချက်` | "note" (comment) | [Comments](../../learn/hello/comments.md) |

## Unsafe code, pointers and the heap

The words of this section belong to unsafe code; see [Unsafe Zaka](../language/unsafe.md).

| Word | Reads as | Page |
| --- | --- | --- |
| `စိတ်မချရသော` | "that cannot be trusted": in front of `လုပ်ဆောင်ချက်`, makes a function or a C function declaration unsafe | [Unsafe Zaka](../language/unsafe.md) |
| `စိတ်မချရသော်လည်း` | "though it cannot be trusted": in `အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်`, opens an unsafe block | [Unsafe Zaka](../language/unsafe.md) |
| `လိပ်စာ` | "address": after `<kind>` it makes the pointer kind (`<kind> လိပ်စာ`, then `၏ လိပ်စာ` for each further level); on its own it is the opaque pointer kind; after a name (`x ၏ လိပ်စာ`, or `&x`) it takes the address | [Unsafe Zaka](../language/unsafe.md) |
| `တန်ဖိုး` | "value": reads or writes through a pointer, as in `p ၏ တန်ဖိုး` (or `*p`); also `တန်ဖိုးသည်` in a declaration | [Unsafe Zaka](../language/unsafe.md), [Variables](../../learn/variables/README.md) |
| `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` | "the address that points at nothing": the null address of a pointer, the one NULL of Zaka; safe code can hold it and compare with it | [The null address](../language/unsafe.md#the-null-address) |
| `မည်သည်ကိုမှ` | "nothing", the first word of the null address | [The null address](../language/unsafe.md#the-null-address) |
| `မညွှန်းသော` | "that points at no", the second word of the null address | [The null address](../language/unsafe.md#the-null-address) |
| `အရွယ်အစား` | "size": the size of a kind in octets, as in `<kind> တစ်<word> ၏ အရွယ်အစား` | [Unsafe Zaka](../language/unsafe.md) |
| `မင်မိုရီ`, `မှတ်ဉာဏ်` | "RAM" (two interchangeable spellings) | [Unsafe Zaka](../language/unsafe.md) |
| `ဘိုက်စာ` | "octets' worth", as in `n ဘိုက်စာ` | [Unsafe Zaka](../language/unsafe.md) |
| `ခုစာ`, `လုံးစာ` | "items' worth", as in `ကိန်းပြည့် n ခုစာ` and `စာလုံး n လုံးစာ`; a kind that names a classifier takes `<classifier>စာ` | [Unsafe Zaka](../language/unsafe.md) |
| `မင်မိုရီတွင် … နေရာယူ၍ လိပ်စာကို … တွင် သိမ်းထားမည်` | "in RAM, taking … of room, keep the address in …" (allocate) | [Unsafe Zaka](../language/unsafe.md) |
| `လိပ်စာ … ရှိ မင်မိုရီကို … ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်` | "make the RAM at … grow or shrink so that it is …" (resize) | [Unsafe Zaka](../language/unsafe.md) |
| `လိပ်စာ … ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်` | "give back the RAM at …" (free) | [Unsafe Zaka](../language/unsafe.md) |
| `လိပ်စာ … ရှိ … မင်မိုရီကို လိပ်စာ … ဆီသို့ ကူးယူမည်` | "write the RAM at … to the address …, ranges apart" (a `memcpy`) | [Writing and filling cells](../language/unsafe.md#writing-and-filling-cells) |
| `လိပ်စာ … ရှိ … မင်မိုရီကို လိပ်စာ … ဆီသို့ ပြောင်းထည့်မည်` | "move the RAM at … to the address …, ranges can overlap" (a `memmove`) | [Writing and filling cells](../language/unsafe.md#writing-and-filling-cells) |
| `လိပ်စာ … ရှိ မင်မိုရီကို … ဖြင့် … ဖြည့်မည်` | "fill the RAM at … with … for …" (fill) | [Writing and filling cells](../language/unsafe.md#writing-and-filling-cells) |

The words inside the statements (`နေရာယူ၍`, `တွင်`, `သိမ်းထားမည်`, `ရှိ`,
`ဖြစ်အောင်`, `နေရာအချဲ့အချုံ့လုပ်မည်`, `ပြန်လည်စွန့်လွှတ်မည်`, `ဆီသို့`, `ကူးယူမည်`,
`ပြောင်းထည့်မည်`, `ဖြည့်မည်`) are reserved on their own
as well.

## Lists

| Word | Reads as | Page |
| --- | --- | --- |
| `စာရင်း` | "list": after `<kind>` it makes a list kind (`<kind> စာရင်း`, or `<kind> ၏ စာရင်း`), plain with no count; `တို့ပါသော စာရင်း` closes a list literal | [Lists](../../learn/lists/README.md) |
| `အကွက်` | "cell": `x ၏ အကွက် i` is cell `i`, counted from 0; `အကွက် n ခုရှိသော` gives the count of a counted list kind | [Lists](../../learn/lists/README.md) |
| `ခုမြောက်`, `လုံးမြောက်` | "-th": `x ၏ n ခုမြောက်အကွက်` is cell `n`, counted from 1, and `t ၏ n ခုမြောက်စာလုံး` or `t ၏ n လုံးမြောက်စာလုံး` character `n` of text | [Lists](../../learn/lists/README.md#cell-numbers) |
| `ခုရှိသော`, `ရှိသော` | "that has", in `အကွက် n ခုရှိသော <kind> စာရင်း` | [Static data and counted lists](../../learn/forms/counted-lists.md#counted-lists) |
| `အကွက်အရေအတွက်` | "cell count", in `x ၏ အကွက်အရေအတွက်` | [Lists](../../learn/lists/README.md#cell-numbers) |
| `နောက်ဆုံးအကွက်` | "last cell", right after `၏` and before `ကို ဖျက်မည်` alone | [Adding and removing cells](../../learn/lists/growing.md) |
| `နောက်ဆုံးစာလုံး` | "last character", right after `၏` and before `ကို ဖျက်မည်` alone: `t ၏ နောက်ဆုံးစာလုံးကို ဖျက်မည်` removes the last character of text | [Text](../../learn/text/changing.md#adding-in-the-middle-and-removing-the-end) |
| `အရှေ့တွင်` | "in front of", after `<cell> ၏` or `<character> ၏` in an add: `l ၏ အကွက် i ၏ အရှေ့တွင် v ကို ထည့်မည်` | [Adding and removing cells](../../learn/lists/growing.md#adding-in-the-middle) |
| `ဆက်ထားသည့်` | "that joins", in `a နှင့် b တို့ကို ဆက်ထားသည့် စာသား`: new text that joins the pieces | [Text](../../learn/text/README.md#joining-text) |
| `စာလုံးအရေအတွက်` | "character count", in `t ၏ စာလုံးအရေအတွက်`; `t ၏ စာလုံး i` is character `i` of text, counted from 0 | [Text](../../learn/text/characters.md#characters) |
| `စာရင်းအလွတ်` | "blank list": a plain list with no cells; a counted list cannot take it (ZK0436) | [Lists](../../learn/lists/README.md) |
| `တို့ပါသော` | "that hold these", before `စာရင်း` in a list literal | [Lists](../../learn/lists/README.md#list-kinds) |
| `ပါသော` | "that holds", before `စာရင်း` or `ဇယား` in a literal of one cell or row | [Lists in words](../language/words.md#lists-in-words) |
| `မှစ၍` | "starting at", in the range `x ၏ အကွက် <start> မှစ၍ <length> ကွက်`, a new list or text that holds duplicates of those cells | [Ranges](../language/lists.md#ranges) |
| `ထိ` | "up to", after the last cell or character of the range a replace changes: `<start> မှ <end> ထိကို` | [Text](../../learn/text/changing.md#replacing-part-of-text) |
| `ကွက်` | "cells", after the length of a range; a struct can also name it as its classifier | [Ranges](../language/lists.md#ranges) |

## Tables

| Word | Reads as | Page |
| --- | --- | --- |
| `ဇယား` | "table": ends a ဇယား kind, `<index kind> ဖြင့် ညွှန်းသော <value kind> ဇယား`; `တို့ပါသော ဇယား` closes a ဇယား literal | [Tables](../language/tables.md) |
| `ဇယားအလွတ်` | "blank table": a ဇယား with no rows, as `[:]` is | [Tables](../language/tables.md#literals) |
| `ညွှန်းသော` | "indexed", after `<index kind> ဖြင့်` in a ဇယား kind; `မညွှန်းသော` of the null address is a word of its own | [Tables](../language/tables.md) |
| `အညွှန်း` | "index": `<table> ထဲမှ အညွှန်း <index>` is the row of the index, and `၏ တန်ဖိုး` after it its value | [Tables](../language/tables.md#reading-a-row) |
| `အညွှန်းအရေအတွက်` | "index count", in `<table> ၏ အညွှန်းအရေအတွက်`: the number of rows | [Tables](../language/tables.md#counting-and-walking-the-rows) |
| `အညွှန်းတိုင်း`, `အတန်းတိုင်း` | "each index", "each row": the loops over the rows of a ဇယား | [Tables](../language/tables.md#counting-and-walking-the-rows) |

`ထဲမှ` and `အနေဖြင့်` take part in a ဇယား too, in `<table> ထဲမှ အညွှန်း <index>` and in the row
`<index> အနေဖြင့် <value>` of a literal. `မရှိလျှင်` (`မရှိ`, `လျှင်`) after `ထဲမှ ဒေတာ၊` gives the
fallback of an optional ([Optional values](../language/optionals.md#reading)).

## Structs

| Word | Reads as | Page |
| --- | --- | --- |
| `အချက်အလက်အစုအဝေး`, `ဒေတာအစုအဝေး` | "data group": the word of a struct declaration, `<name> ဟူသော ဒေတာအစုအဝေးတွင် ...` (`ဒေတာ` and `အချက်အလက်` are one word in each reserved word and name; `တွင်` can stand joined or apart) | [Structs](../../learn/structs/README.md) |
| `တို့ပါဝင်သည်` (`တို့`, `ပါဝင်သည်`) | "these are included": closes the field list of a struct, and the member list of an enumeration | [Structs](../../learn/structs/README.md) |
| `တို့ပါဝင်ပြီး` (`တို့`, `ပါဝင်ပြီး`) | "these are included, and": closes the list when a classifier follows | [Structs](../../learn/structs/README.md#count-words) |
| `မျှဝေနိုင်သော` | "that can be shared": after `ဟူသော`, in front of `ဒေတာအစုအဝေး`, declares a shared object kind instead of a struct | [Values and objects](../language/objects.md) |
| `မျိုးပြပစ္စည်းမှာ` | "the count word is": names the classifier of a struct, an enumeration or an alias | [Structs](../../learn/structs/README.md#count-words) |
| `ဖြစ်သည်` | "is": ends a declaration that names a classifier, and a kind from outside with no fields | [Structs](../../learn/structs/README.md#count-words) |

`တို့` on its own is reserved as well, so it cannot be used as a name. The words inside
the two closing phrases (`တို့`, `ပါဝင်သည်`, `ပါဝင်ပြီး`) are reserved on their own, and the
phrases can be written joined or with spaces.

## Enumerations and switches

| Word | Reads as | Page |
| --- | --- | --- |
| `အချက်အလက်အမျိုးအစားကွဲ`, `ဒေတာအမျိုးအစားကွဲ` | "data kind of variants": the word of an enumeration declaration, `<name> ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ...` | [Enumerations and switches](../../learn/enums/README.md) |
| `ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်` (`ကနဦးတန်ဖိုး`, `အနေဖြင့်`, `သုံးမည့်`) | "to be used as the initial value": in front of a member, the initial member of an enumeration | [The initial member](../language/enums.md#the-initial-member) |
| `တန်ဖိုးအနေဖြင့်` (`တန်ဖိုး`, `အနေဖြင့်`) | "as its value": opens the value of a member, `တန်ဖိုးအနေဖြင့် <value> ရှိသည့် <member>` | [Enumerations](../language/enums.md#declaring-an-enumeration) |
| `ရှိသည့်` | "that has": closes the value of a member | [Enumerations](../language/enums.md#declaring-an-enumeration) |
| `သုံးမည့်` | "to be used": the last word of `ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်` | [The initial member](../language/enums.md#the-initial-member) |
| `ထဲမှ` | "from among", in `<name>များထဲမှ <member>` | [Enumerations and switches](../../learn/enums/README.md#naming-a-member) |
| `တခြား` | "another": starts the default arm of a switch, `တခြား <kind> တစ်<word> ဖြစ်လျှင်` | [Enumerations and switches](../../learn/enums/README.md#the-default-arm) |

An enumeration declaration has the shape of a struct declaration, with
`ဒေတာအမျိုးအစားကွဲ` in place of `ဒေတာအစုအဝေး`, and closes with `တို့ပါဝင်သည်`, or
`ပါဝင်သည်` after a lone member. `အမျိုးအစား` alone is reserved as a piece of
`လုပ်ဆောင်ချက် အမျိုးအစား`, the closer of a function kind.

`ထဲမှ`
can be written joined to the name before it, as in `အရောင်များထဲမှ`: the lexer splits a
name that ends in `ထဲမှ` into the name and `ထဲမှ`, so no name can end in `ထဲမှ`.

## Kind aliases

| Word | Reads as | Page |
| --- | --- | --- |
| `အချက်အလက်အမျိုးအစား`, `ဒေတာအမျိုးအစား` | "data kind": opens a kind alias | [Kind aliases](../language/aliases.md) |
| `ဟုလည်း မှတ်ယူနိုင်ပါသည်` | "can also be taken as": closes a kind alias, `ဒေတာအမျိုးအစား <alias> ကို <kind> ဟုလည်း မှတ်ယူနိုင်ပါသည်။` | [Kind aliases](../language/aliases.md) |
| `ဟုလည်း မှတ်ယူနိုင်ပြီး` | "can also be taken as, and": closes a kind alias that names its classifier, `... ဟုလည်း မှတ်ယူနိုင်ပြီး မျိုးပြပစ္စည်းမှာ <word> ဖြစ်သည်။` | [Kind aliases](../language/aliases.md) |

The pieces of the closers, `လည်း` ("also"), `မှတ်ယူနိုင်ပါသည်` ("can be taken as") and
`မှတ်ယူနိုင်ပြီး` ("can be taken as, and"), are reserved on their own too. `ဟုလည်း` can be written as one word or as two.

## Generics and traits

| Word | Reads as | Page |
| --- | --- | --- |
| `ယေဘုယျဒေတာအမျိုးအစား`, `ယေဘုယျအချက်အလက်အမျိုးအစား` | "general data kind": names a kind parameter in the word form, `ယေဘုယျဒေတာအမျိုးအစား T` | [Generics](../../learn/generics/README.md) |
| `အစားထိုးသုံးနိုင်၍` | "can be put in place, and": after the kind parameters of the word form, `… ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်` | [Generics and traits](../language/generics.md#kind-parameters) |
| `၎င်း` | "it": the declaration again, after its kind parameters, `၎င်းသည်` or `၎င်းတွင်` | [Generics and traits](../language/generics.md#kind-parameters) |
| `အစားထိုးအသုံးပြုသည့်` | "that puts in place": after the kinds of a generic in the word form, before its name | [Generics and traits](../language/generics.md#kinds-at-a-use) |
| `အစားထိုးအသုံးပြု၍` | "putting in place": after the kinds of a call written arguments first | [Generics and traits](../language/generics.md#calls) |
| `::` | the kinds of a generic in the sign form, `f::<ကိန်းပြည့်>(၁)` | [Generics and traits](../language/generics.md#calls) |
| `ဟူသည့်` | "called": after the name of a trait in its declaration, `<name> ဟူသည့် ဂုဏ်သတ္တိ` | [Generics and traits](../language/generics.md#traits) |
| `ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော` | "included inside the compiler": marks a trait of the prelude, where `ပြင်ပမှလာသော` marks an external one | [The prelude](../language/prelude.md#the-compiler-mark) |
| `ဂုဏ်သတ္တိ` | "trait": after the name of a trait, in its declaration, its closer, a constraint and a trait block | [Traits](../../learn/generics/traits.md) |
| `ပါဝင်ရမည်` | "must hold": closes the head of a trait, `… အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။` | [Generics and traits](../language/generics.md#traits) |
| `လုပ်ဆောင်ချက်များ` | "functions": in the heads and closers of traits and trait blocks | [Generics and traits](../language/generics.md#traits) |
| `ရှိစေရန်` | "so that it has": in the head of a trait block, `<kind> တွင် <trait> ဂုဏ်သတ္တိရှိစေရန်` | [Generics and traits](../language/generics.md#trait-blocks) |
| `ထည့်သွင်းမည့်` | "to be added": opens the closer of a trait block | [Generics and traits](../language/generics.md#trait-blocks) |
| `ဤနေရာတွင်`, `သတိပြုရန်မှာ` | "here", "to note": open a notes block | [Generics and traits](../language/generics.md#the-notes-block) |
| `သတိပြုစရာများ` | "the notes": opens the closer of a notes block | [Generics and traits](../language/generics.md#the-notes-block) |
| `ရှိသည်` | "has": closes a constraint in a notes block, `A တွင် … ဂုဏ်သတ္တိ ရှိသည်။` | [Generics and traits](../language/generics.md#the-notes-block) |

The names of the traits the [prelude](../language/prelude.md) declares are not reserved
words: a name a program declares hides the prelude's.

## Modules

| Word | Reads as | Page |
| --- | --- | --- |
| `အများသုံး` | "for common use": exports a top-level declaration | [Modules](../../learn/modules/README.md#sharing-names) |
| `ထည့်သွင်းမည်` | "will bring in": ends an import, after `"<file>" မှ <name> ကို` | [Modules](../../learn/modules/README.md#importing-names) |
