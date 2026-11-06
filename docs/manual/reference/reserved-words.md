---
title: Reserved words
description: Each word Zaka gives a meaning to, with an English reading.
---

# Reserved words

These words have a fixed meaning in Zaka, so a program cannot use them as names. A word
made of nothing but reserved words, such as `ထက်ကြီးသည်` or `တန်ဖိုးကို`, is split into
them. To use one of these as a name all the same, wrap it in backticks (`` `ကြီး` ``);
see [Names](../../learn/variables/names.md).

## Particles

| Word | Reads as | Used in |
| --- | --- | --- |
| `သည်` | (subject); also a polite ending | [Particles](../../learn/sentences/particles.md) |
| `ကို` | (object) | [Particles](../../learn/sentences/particles.md) |
| `နှင့်` | "and", "with" | [Arithmetic](../../learn/operators/arithmetic.md), [Comparison](../../learn/operators/comparison.md) |
| `၏` | "of" | [Arithmetic](../../learn/operators/arithmetic.md) |
| `ဖြင့်` | "with", "through" | [Arithmetic](../../learn/operators/arithmetic.md), [Integers](../../learn/values/integers.md) |
| `ထက်` | "than" | [Comparison](../../learn/operators/comparison.md) |
| `အောက်` | "below" | [Comparison](../../learn/operators/comparison.md) |
| `ဟု` | "so" (quoting) | [Printing](../../learn/hello/printing.md), [Changing a variable](../../learn/variables/assignment.md) |
| `အဖြစ်` | "as", "into" | [Converting between kinds](../../learn/operators/conversion.md) |
| `ပါ`, `ပါသည်` | polite endings | [Particles](../../learn/sentences/particles.md) |
| `တို့ကို`, `တို့ဖြင့်` | close a list of parameters / arguments | [Parameters](../../learn/functions/parameters.md) |
| `လက်ခံပြီး` | "accepting ... and" | [Parameters](../../learn/functions/parameters.md) |
| `ပြင်ပမှလာသော` (`ပြင်ပ`, `မှ`, `လာသော`) | "coming from outside": marks an external function | [Functions](../language/functions.md#external-functions) |
| `လုပ်ဆောင်ချက်တစ်ခု` | "a function", in the header of an external function | [Functions](../language/functions.md#external-functions) |
| `သုံး၍` | "using": after `ကို` or `တို့ကို`, closes the arguments of a call written before its function | [Parameters](../../learn/functions/parameters.md#arguments-first) |
| `ထဲသို့` | "into", after the list or the text in `l ထဲသို့ v ကို ထည့်ပါ` | [Adding and removing cells](../../learn/lists/growing.md) |
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
| `ညီ`, `မညီ` | "equal", "not equal" | `==`, `!=` | [Comparison](../../learn/operators/comparison.md) |
| `ကြီး`, `မကြီး` | "greater", "not greater" | `>`, `<=` | [Comparison](../../learn/operators/comparison.md) |
| `ငယ်`, `မငယ်` | "less", "not less" | `<`, `>=` | [Comparison](../../learn/operators/comparison.md) |
| `ပြောင်းလဲသော ရလဒ်` | "changed result" | | [Converting between kinds](../../learn/operators/conversion.md) |
| `လုပ်ဆောင်သော ရလဒ်` | "result of running" | | [Results](../../learn/functions/results.md) |
| `သည့် ရလဒ်` | "the result that ...": ends the short call `<arguments> ကို သုံး၍ <name> သည့် ရလဒ်` | | [Results](../../learn/functions/results.md#arguments-first-with-a-result) |
| `သည့် ရလဒ်ကောင်း` | "the good result that ...": the short call passes a failure on | `?` after a call | [Functions](../language/functions.md#arguments-first) |
| `လုပ်ဆောင်သော ရလဒ်ကောင်း` | "good result of running": the call passes a failure on | `?` after a call | [Errors and results](../../learn/error-handling/README.md) |
| `ဖြစ်၍` | "and"; also joins the kind and the value in a declaration | `&&` | [Logic](../../learn/operators/logic.md), [Variables](../../learn/variables/README.md) |
| `သို့မဟုတ်` | "or" | `\|\|` | [Logic](../../learn/operators/logic.md) |
| `မဟုတ်` | "not" | `!` | [Logic](../../learn/operators/logic.md) |
| `အနုတ်` | "minus" | `-` (in front) | [Arithmetic](../../learn/operators/arithmetic.md) |

## Verbs that make a statement

| Word | Reads as | Page |
| --- | --- | --- |
| `ပြသပါ` | "please show" | [Printing](../../learn/hello/printing.md) |
| `ထားပါ` | "please set" | [Changing a variable](../../learn/variables/assignment.md) |
| `ဖြစ်ပါစေ` | "let it be" | [Variables](../../learn/variables/README.md) |
| `လုပ်ဆောင်ပါ` | "please run" | [Functions](../../learn/functions/README.md) |
| `လုပ်ဆောင်မည်` | "will run": a second end of a call statement, and the end of the long arguments-first call | [Parameters](../../learn/functions/parameters.md#arguments-first) |
| `မည်` | "will": ends the short call statement `<name> မည်` | [Functions](../../learn/functions/README.md#calling) |
| `လက်ခံမည်` | "will accept": ends the parameters of an external function with no result | [Functions](../language/functions.md#external-functions) |
| `ရလဒ်အဖြစ် ပြန်ပေးပါ` | "give back as the result": in a function that cannot fail | [Results](../../learn/functions/results.md) |
| `ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ` | "give back as the good result": in a function that can fail | [Errors and results](../../learn/error-handling/README.md) |
| `ပြန်ပေးပါ` | "please return" | [Functions](../../learn/functions/README.md) |
| `ပြန်ပေးမည်` | "will return" | [Results](../../learn/functions/results.md) |
| `အမှားအယွင်းအဖြစ် ပြန်ပေးပါ` | "give back as an error value" (also `အမှားအယွင်းအနေဖြင့်`) | [Errors and results](../../learn/error-handling/README.md) |
| `လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်` | "mark the function as succeeded": ends it with a good result that holds no value | [Writing result kinds](../../learn/error-handling/kinds.md) |
| `လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်` | "mark the function as not succeeded": ends it with a failure that holds no error value | [Writing result kinds](../../learn/error-handling/kinds.md) |
| `၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးပါ` | "if it fails, give back the error value": ends a call statement that passes a failure on | [Errors and results](../../learn/error-handling/README.md) |
| `ရပ်ပါ` | "please stop" | [Break and continue](../../learn/control-flow/break-continue.md) |
| `ထည့်ပါ` | "please add": `l ထဲသို့ v ကို ထည့်ပါ` adds `v` as a new last cell of `l`, or, when `l` is text, adds the text or character `v` at its end; `l ၏ အကွက် i ၏ အရှေ့တွင် v ကို ထည့်ပါ` puts it in front of cell `i` | [Adding and removing cells](../../learn/lists/growing.md) |
| `ဖျက်ပါ` | "please remove": `l ၏ နောက်ဆုံးအကွက်ကို ဖျက်ပါ` removes the last cell of `l` | [Adding and removing cells](../../learn/lists/growing.md) |
| `အစားထိုးပါ` | "please replace": `t ၏ အကွက် a မှ b ထိကို v ဖြင့် အစားထိုးပါ` puts `v` in place of cells `a` to `b` of text `t` | [Text](../../learn/text/changing.md#replacing-part-of-text) |
| `ကျော်ပါ` | "please skip" | [Break and continue](../../learn/control-flow/break-continue.md) |

## Blocks and control flow

| Word | Reads as | Page |
| --- | --- | --- |
| `အောက်ပါတို့ကို လုပ်ဆောင်ပါ` | "please do the following" | [Blocks and scope](../../learn/variables/scope.md) |
| `အောက်ပါတို့ကို လုပ်ဆောင်မည်` | "will do the following" | [Functions](../../learn/functions/README.md) |
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
| `ကိန်းပြည့်` | "integer" | [Integers](../../learn/values/integers.md) |
| `ဒဿမကိန်း` | "decimal number" | [Decimal numbers](../../learn/values/floats.md) |
| `စာလုံး` | "character" | [Characters](../../learn/values/characters.md) |
| `မှန်မှားတန်ဖိုး`, `ဘူလီယန်` | "true/false value", "boolean" | [True and false](../../learn/values/booleans.md) |
| `စာသား` | "text" | [Text](../../learn/text/README.md) |
| `လက္ခဏာရှိ`, `လက္ခဏာမဲ့` | "signed", "unsigned" | [Integers](../../learn/values/integers.md) |
| `အခြေ` | "base" | [Integers](../../learn/values/integers.md) |
| `ကိန်းရှင်` | "variable" | [Variables](../../learn/variables/README.md) |
| `ကိန်းသေ` | "constant" | [Constants](../../learn/variables/constants.md) |
| `လုပ်ဆောင်ချက်` | "function" | [Functions](../../learn/functions/README.md) |
| `ပရိုဂရမ်` | "program": `ပရိုဂရမ်သည်` opens the start block, where the program starts | [Where a program starts](../../learn/programs/start.md) |
| `လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သော` | "that the function can change": before the kind of a parameter that is the caller's place | [Functions](../language/functions.md#edit-references) |
| `လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်` | "the function can change it": in parentheses after an argument for such a parameter | [Functions](../language/functions.md#edit-references) |
| `ရလဒ်` | "result" | [Results](../../learn/functions/results.md) |
| `ရလဒ်ကောင်း` | "good result": of a call that can fail; also the test `R သည် ရလဒ်ကောင်း` and the conversion target `R ကို ရလဒ်ကောင်း အဖြစ် ပြောင်းလဲသော ရလဒ်` | [Errors and results](../../learn/error-handling/README.md) |
| `မအောင်မြင်ပါက` | "if it fails": in the trailer of a call statement, `၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးပါ` | [Errors and results](../../learn/error-handling/README.md) |
| `အောင်မြင်သည့်သတင်း` | "news of success": the good side of a result kind that holds no value | [Writing result kinds](../../learn/error-handling/kinds.md) |
| `မအောင်မြင်သည့်သတင်း` | "news of failure": the error side of a result kind that holds no value | [Writing result kinds](../../learn/error-handling/kinds.md) |
| `အောင်မြင်သည်` | "it succeeded": in the phrase `လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်` | [Writing result kinds](../../learn/error-handling/kinds.md) |
| `မအောင်မြင်ပါ` | "it did not succeed": in the phrase `လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်` | [Writing result kinds](../../learn/error-handling/kinds.md) |
| `သတ်မှတ်မည်` | "will mark": ends the two phrases above | [Writing result kinds](../../learn/error-handling/kinds.md) |
| `အမှားအယွင်း` | "error value"; also the test `R သည် အမှားအယွင်း` and the conversion target `R ကို အမှားအယွင်း အဖြစ် ပြောင်းလဲသော ရလဒ်` | [Errors and results](../../learn/error-handling/README.md) |
| `မှန်`, `မမှန်` | "true", "false" | [True and false](../../learn/values/booleans.md) |
| `မှတ်ချက်` | "note" (comment) | [Comments](../../learn/hello/comments.md) |

## Pointers and the heap

| Word | Reads as | Page |
| --- | --- | --- |
| `လိပ်စာ` | "address": after `<kind>` it makes a pointer kind (`<kind> လိပ်စာ`, then `၏ လိပ်စာ` for each further level); on its own it is the opaque pointer kind; after a name (`x ၏ လိပ်စာ`, or `&x`) it takes the address | [Pointers and the heap](../../learn/pointers/README.md) |
| `တန်ဖိုး` | "value": dereference, as in `p ၏ တန်ဖိုး` (or `*p`); also `တန်ဖိုးသည်` in a declaration | [Pointers and the heap](../../learn/pointers/README.md), [Variables](../../learn/variables/README.md) |
| `ဗလာ` | "blank": the null pointer | [Pointers and the heap](../../learn/pointers/README.md#the-null-address) |
| `ငှားထားသော` | "lent": in front of a pointer kind, the pointer borrows what it points at | [Ownership and borrowing](../../learn/pointers/ownership.md) |
| `တန်ဖိုးအငှား` | "the value, on loan": `ပ ၏ တန်ဖိုးအငှား` lends the block `ပ` owns | [Ownership and borrowing](../../learn/pointers/ownership.md#lending-a-block) |
| `လိပ်စာအကြမ်း` | "raw address": a pointer level the compiler does not follow | [Ownership and borrowing](../../learn/pointers/ownership.md#raw-addresses) |
| `အရွယ်အစား` | "size": the size of a kind in octets, as in `<kind> ၏ အရွယ်အစား` | [The heap](../../learn/pointers/heap.md#how-much-to-ask-for) |
| `မင်မိုရီ`, `မှတ်ဉာဏ်` | "RAM" (two interchangeable spellings) | [The heap](../../learn/pointers/heap.md) |
| `ဘိုက်စာ` | "octets' worth", as in `n ဘိုက်စာ` | [The heap](../../learn/pointers/heap.md#how-much-to-ask-for) |
| `ခုစာ` | "items' worth", as in `<kind> n ခုစာ`; a struct that names a classifier takes `<classifier>စာ` | [The heap](../../learn/pointers/heap.md#how-much-to-ask-for) |
| `မင်မိုရီတွင် … နေရာယူ၍ လိပ်စာကို … တွင် သိမ်းထားမည်` | "in RAM, taking … of room, keep the address in …" (allocate) | [The heap](../../learn/pointers/heap.md) |
| `လိပ်စာ … ရှိ မင်မိုရီကို … ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်` | "make the RAM at … grow or shrink so that it is …" (resize) | [Growing and shrinking](../../learn/pointers/resizing.md) |
| `လိပ်စာ … ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်` | "give back the RAM at …" (free) | [The heap](../../learn/pointers/heap.md#giving-a-block-back) |
| `လိပ်စာ … ရှိ … မင်မိုရီကို လိပ်စာ … ဆီသို့ ကူးယူမည်` | "write the RAM at … to the address …, ranges apart" (a `memcpy`) | [Writing and filling cells](../../learn/pointers/resizing.md#writing-and-filling-cells) |
| `လိပ်စာ … ရှိ … မင်မိုရီကို လိပ်စာ … ဆီသို့ ပြောင်းထည့်မည်` | "move the RAM at … to the address …, ranges can overlap" (a `memmove`) | [Writing and filling cells](../../learn/pointers/resizing.md#writing-and-filling-cells) |
| `လိပ်စာ … ရှိ မင်မိုရီကို … ဖြင့် … ဖြည့်မည်` | "fill the RAM at … with … for …" (fill) | [Writing and filling cells](../../learn/pointers/resizing.md#writing-and-filling-cells) |

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
| `နောက်ဆုံးအကွက်` | "last cell", right after `၏` and before `ကို ဖျက်ပါ` alone | [Adding and removing cells](../../learn/lists/growing.md) |
| `နောက်ဆုံးစာလုံး` | "last character", right after `၏` and before `ကို ဖျက်ပါ` alone: `t ၏ နောက်ဆုံးစာလုံးကို ဖျက်ပါ` removes the last character of text | [Text](../../learn/text/changing.md#adding-in-the-middle-and-removing-the-end) |
| `အရှေ့တွင်` | "in front of", after `<cell> ၏` or `<character> ၏` in an add: `l ၏ အကွက် i ၏ အရှေ့တွင် v ကို ထည့်ပါ` | [Adding and removing cells](../../learn/lists/growing.md#adding-in-the-middle) |
| `ဆက်ထားသည့်` | "that joins", in `a၊ b တို့ကို ဆက်ထားသည့် စာသား`: new text that joins the pieces | [Text](../../learn/text/README.md#joining-text) |
| `စာလုံးအရေအတွက်` | "character count", in `t ၏ စာလုံးအရေအတွက်`; `t ၏ စာလုံး i` is character `i` of text, counted from 0 | [Text](../../learn/text/characters.md#characters) |
| `စာရင်းအလွတ်` | "blank list": no cells, or each cell of a counted list at its default | [Lists](../../learn/lists/README.md) |
| `တို့ပါသော` | "that hold these", before `စာရင်း` in a list literal | [Lists](../../learn/lists/README.md#list-kinds) |
| `တစ်စိတ်တစ်ပိုင်း` | "a part": after a `စာရင်း` with no count it makes a slice kind | [Slices](../../learn/lists/slices.md) |
| `မှစ၍` | "starting at", in `x ၏ အကွက် <start> မှစ၍ <length> ကွက်` | [Slices](../../learn/lists/slices.md#part-of-a-list) |
| `ထိ` | "up to", after the last cell or character of the range a replace changes: `<start> မှ <end> ထိကို` | [Text](../../learn/text/changing.md#replacing-part-of-text) |
| `ကွက်` | "cells", after the length of a range; a struct can also name it as its classifier | [Slices](../../learn/lists/slices.md#part-of-a-list) |

## Structs

| Word | Reads as | Page |
| --- | --- | --- |
| `အချက်အလက်အမျိုးအစား`, `ဒေတာအမျိုးအစား` | "data kind": starts a struct declaration (`ဒေတာ` and `အချက်အလက်` are one word in each reserved word and name) | [Structs](../../learn/structs/README.md) |
| `တို့ ပါဝင်သည်` | "these are included": closes the field list of a struct | [Structs](../../learn/structs/README.md) |
| `တို့ ပါဝင်ပြီး` | "these are included, and": closes the field list when a classifier follows | [Structs](../../learn/structs/README.md#count-words) |
| `မျိုးပြပစ္စည်းမှာ` | "the count word is": names the classifier of a struct | [Structs](../../learn/structs/README.md#count-words) |
| `ဖြစ်သည်` | "is": ends a struct declaration that names a classifier | [Structs](../../learn/structs/README.md#count-words) |

`တို့` on its own is reserved as well, so it cannot be used as a name. The words inside
the two closing phrases (`တို့`, `ပါဝင်သည်`, `ပါဝင်ပြီး`) are reserved on their own.

## Enumerations and switches

| Word | Reads as | Page |
| --- | --- | --- |
| `အမျိုးအစား` | "kind": starts an enumeration declaration | [Enumerations and switches](../../learn/enums/README.md) |
| `ဆိုသည်မှာ` | "means": between the name of an enumeration and its members | [Enumerations and switches](../../learn/enums/README.md) |
| `ထဲမှ` | "from among", in `<name>များထဲမှ <member>` | [Enumerations and switches](../../learn/enums/README.md#naming-a-member) |
| `တခြား` | "another": starts the default arm of a switch, `တခြား <kind> ဖြစ်လျှင်` | [Enumerations and switches](../../learn/enums/README.md#the-default-arm) |

An enumeration declaration closes with `တို့ ဖြစ်သည်`, words reserved for structs. `ထဲမှ`
can be written joined to the name before it, as in `အရောင်များထဲမှ`: the lexer splits a
name that ends in `ထဲမှ` into the name and `ထဲမှ`, so no name can end in `ထဲမှ`.

## Modules

| Word | Reads as | Page |
| --- | --- | --- |
| `အများသုံး` | "for common use": exports a top-level declaration | [Modules](../../learn/modules/README.md#sharing-names) |
| `ထည့်သွင်းပါ` | "please bring in": ends an import, after `"<file>" မှ <name> ကို` | [Modules](../../learn/modules/README.md#importing-names) |
