---
title: Reserved words
description: Each word Zaka gives a meaning to, with an English reading.
---

# Reserved words

These words have a fixed meaning in Zaka, so a program cannot use them as names. A word
made of nothing but reserved words, such as `ထက်ကြီးသည်` or `တန်ဖိုးကို`, is split into
them. To use one of these as a name all the same, wrap it in backticks (`` `ကြီး` ``);
see [Names](../variables/names.md).

## Particles

| Word | Reads as | Used in |
| --- | --- | --- |
| `သည်` | (subject); also a polite ending | [Particles](../sentences/particles.md) |
| `ကို` | (object) | [Particles](../sentences/particles.md) |
| `နှင့်` | "and", "with" | [Arithmetic](../operators/arithmetic.md), [Comparison](../operators/comparison.md) |
| `၏` | "of" | [Arithmetic](../operators/arithmetic.md) |
| `ဖြင့်` | "with", "through" | [Arithmetic](../operators/arithmetic.md), [Writing integers](../values/literals.md) |
| `ထက်` | "than" | [Comparison](../operators/comparison.md) |
| `အောက်` | "below" | [Comparison](../operators/comparison.md) |
| `ဟု` | "so" (quoting) | [Printing](../hello/printing.md), [Changing a variable](../variables/assignment.md) |
| `အဖြစ်` | "as", "into" | [Converting between kinds](../operators/conversion.md) |
| `ပါ`, `ပါသည်` | polite endings | [Particles](../sentences/particles.md) |
| `တို့ကို`, `တို့ဖြင့်` | close a list of parameters / arguments | [Parameters](../functions/parameters.md) |
| `လက်ခံပြီး` | "accepting ... and" | [Parameters](../functions/parameters.md) |
| `မှ`, `အထိ`, `စီ`, `အနေဖြင့်` | "from", "up to", "each", "as" | [Counted loops](../control-flow/counted-loops.md) |
| `ဖြစ်ကာ`, `ဖြစ်ပြီး` | "being ... and" | [Generic loops](../control-flow/generic-loops.md) |
| `ပြီးလျှင်` | "when done" | [Generic loops](../control-flow/generic-loops.md) |
| `ဘယ်သို့`, `ညာသို့` | "to the left", "to the right" | [Bitwise operators](../operators/bitwise.md) |
| `ဘစ်` | "bit" | [Integers](../values/integers.md), [Bitwise operators](../operators/bitwise.md) |

## Verbs that make a value

| Word | Reads as | Operator | Page |
| --- | --- | --- | --- |
| `ပေါင်းလဒ်` | "sum" | `+` | [Arithmetic](../operators/arithmetic.md) |
| `နုတ်လဒ်` | "difference" | `-` | [Arithmetic](../operators/arithmetic.md) |
| `မြှောက်လဒ်` | "product" | `*` | [Arithmetic](../operators/arithmetic.md) |
| `စားသော စားလဒ်` | "quotient" | `/` | [Arithmetic](../operators/arithmetic.md) |
| `စားသော စားကြွင်း` | "remainder" | `%` | [Arithmetic](../operators/arithmetic.md) |
| `ဘစ်အလိုက် ဆုံလဒ်` | "bitwise meeting" | `&` | [Bitwise operators](../operators/bitwise.md) |
| `ဘစ်အလိုက် ပေါင်းစုလဒ်` | "bitwise union" | `\|` | [Bitwise operators](../operators/bitwise.md) |
| `ဘစ်အလိုက် ခြားနားလဒ်` | "bitwise difference" | `^` | [Bitwise operators](../operators/bitwise.md) |
| `ဘစ်အလိုက် ပြောင်းပြန်ပြောင်းလဒ်` | "bitwise flip" | `~` | [Bitwise operators](../operators/bitwise.md) |
| `ရွေ့သောရလဒ်` | "moved result" | `<<`, `>>` | [Bitwise operators](../operators/bitwise.md) |
| `ညီ`, `မညီ` | "equal", "not equal" | `==`, `!=` | [Comparison](../operators/comparison.md) |
| `ကြီး`, `မကြီး` | "greater", "not greater" | `>`, `<=` | [Comparison](../operators/comparison.md) |
| `ငယ်`, `မငယ်` | "less", "not less" | `<`, `>=` | [Comparison](../operators/comparison.md) |
| `ပြောင်းလဲသော ရလဒ်` | "changed result" | | [Converting between kinds](../operators/conversion.md) |
| `လုပ်ဆောင်သော ရလဒ်` | "result of running" | | [Results](../functions/results.md) |
| `ဖြစ်၍` | "and"; also joins the kind and the value in a declaration | `&&` | [Logic](../operators/logic.md), [Variables](../variables/README.md) |
| `သို့မဟုတ်` | "or" | `\|\|` | [Logic](../operators/logic.md) |
| `မဟုတ်` | "not" | `!` | [Logic](../operators/logic.md) |
| `အနုတ်` | "minus" | `-` (in front) | [Arithmetic](../operators/arithmetic.md) |

## Verbs that make a statement

| Word | Reads as | Page |
| --- | --- | --- |
| `ပြသပါ` | "please show" | [Printing](../hello/printing.md) |
| `ထားပါ` | "please set" | [Changing a variable](../variables/assignment.md) |
| `ဖြစ်ပါစေ` | "let it be" | [Variables](../variables/README.md) |
| `လုပ်ဆောင်ပါ` | "please run" | [Functions](../functions/README.md) |
| `ရလဒ်အဖြစ် ပြန်ပေးပါ` | "give back as the result" | [Results](../functions/results.md) |
| `ပြန်ပေးပါ` | "please return" | [Functions](../functions/README.md) |
| `ပြန်ပေးမည်` | "will return" | [Results](../functions/results.md) |
| `ရပ်ပါ` | "please stop" | [Break and continue](../control-flow/break-continue.md) |
| `ကျော်ပါ` | "please skip" | [Break and continue](../control-flow/break-continue.md) |

## Blocks and control flow

| Word | Reads as | Page |
| --- | --- | --- |
| `အောက်ပါတို့ကို လုပ်ဆောင်ပါ` | "please do the following" | [Blocks and scope](../variables/scope.md) |
| `အောက်ပါတို့ကို လုပ်ဆောင်မည်` | "will do the following" | [Functions](../functions/README.md) |
| `အောက်ပါတို့ကို လုပ်ဆောင်ကာ` | "doing the following, ..." | [Results](../functions/results.md) |
| `လုပ်ဆောင်စရာများ ပြီးပါပြီး` | "the tasks are done" (four spellings) | [Blocks and scope](../variables/scope.md) |
| `အကယ်၍` | "if" | [If and else](../control-flow/if-else.md) |
| `လျှင်`, `ဖြစ်လျှင်` | "then" | [If and else](../control-flow/if-else.md) |
| `ထိုသို့မဟုတ်ဘဲ` | "if not so, but" (else if) | [If and else](../control-flow/if-else.md) |
| `သို့မဟုတ်ပါက` | "otherwise" (else) | [If and else](../control-flow/if-else.md) |
| `နေသရွေ့`, `ဖြစ်နေသရွေ့` | "as long as" | [While loops](../control-flow/while.md) |
| `ရေတွက်၍`, `တိုး၍`, `လျှော့၍` | "counting", "increasing", "decreasing" | [Counted loops](../control-flow/counted-loops.md) |
| `ကွင်းဆက်` | "loop" | [Break and continue](../control-flow/break-continue.md) |
| `နောက်တစ်ကြိမ်ဆီသို့` | "to the next round" | [Break and continue](../control-flow/break-continue.md) |

## Kinds, declarations and values

| Word | Reads as | Page |
| --- | --- | --- |
| `ကိန်းပြည့်` | "integer" | [Integers](../values/integers.md) |
| `ဒဿမကိန်း` | "decimal number" | [Decimal numbers](../values/floats.md) |
| `စာလုံး` | "character" | [Characters](../values/characters.md) |
| `မှန်မှားတန်ဖိုး`, `ဘူလီယန်` | "true/false value", "boolean" | [True and false](../values/booleans.md) |
| `လက္ခဏာရှိ`, `လက္ခဏာမဲ့` | "signed", "unsigned" | [Integers](../values/integers.md) |
| `အခြေ` | "base" | [Writing integers](../values/literals.md) |
| `ကိန်းရှင်` | "variable" | [Variables](../variables/README.md) |
| `ကိန်းသေ` | "constant" | [Constants](../variables/constants.md) |
| `လုပ်ဆောင်ချက်` | "function" | [Functions](../functions/README.md) |
| `ရလဒ်` | "result" | [Results](../functions/results.md) |
| `မှန်`, `မမှန်` | "true", "false" | [True and false](../values/booleans.md) |
| `မှတ်ချက်` | "note" (comment) | [Comments](../hello/comments.md) |

## Pointers and the heap

| Word | Reads as | Page |
| --- | --- | --- |
| `လိပ်စာ` | "address": after `<kind>` it makes a pointer kind (`<kind> လိပ်စာ`, then `၏ လိပ်စာ` for each further level); on its own it is the opaque pointer kind; after a name (`x ၏ လိပ်စာ`, or `&x`) it takes the address | [Pointers and the heap](../pointers/README.md) |
| `တန်ဖိုး` | "value": dereference, as in `p ၏ တန်ဖိုး` (or `*p`); also `တန်ဖိုးသည်` in a declaration | [Pointers and the heap](../pointers/README.md), [Variables](../variables/README.md) |
| `ဗလာ` | "blank": the null pointer | [Pointers and the heap](../pointers/README.md#the-null-address) |
| `အရွယ်အစား` | "size": the size of a kind in octets, as in `<kind> ၏ အရွယ်အစား` | [The heap](../pointers/heap.md#how-much-to-ask-for) |
| `မင်မိုရီ`, `မှတ်ဉာဏ်` | "RAM" (two interchangeable spellings) | [The heap](../pointers/heap.md) |
| `ဘိုက်စာ` | "octets' worth", as in `n ဘိုက်စာ` | [The heap](../pointers/heap.md#how-much-to-ask-for) |
| `ခုစာ` | "items' worth", as in `<kind> n ခုစာ` | [The heap](../pointers/heap.md#how-much-to-ask-for) |
| `မင်မိုရီ … နေရာယူ၍ လိပ်စာကို … တွင် သိမ်းထားမည်` | "taking … of heap room, keep the address in …" (allocate) | [The heap](../pointers/heap.md) |
| `… လိပ်စာရှိကို မင်မိုရီ … ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်` | "grow or shrink the space at … so that it is …" (resize) | [Growing and shrinking](../pointers/resizing.md) |
| `… လိပ်စာရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်` | "release the block at …" (free) | [The heap](../pointers/heap.md#giving-a-block-back) |

The words inside the three statements (`နေရာယူ၍`, `တွင်`, `သိမ်းထားမည်`, `လိပ်စာရှိ`,
`ဖြစ်အောင်`, `နေရာအချဲ့အချုံ့လုပ်မည်`, `ပြန်လည်စွန့်လွှတ်မည်`) are reserved on their own
as well.

## Written and spoken endings

Each reserved word that ends in the written `…မည်` also accepts the spoken `…မယ်`, and
the two spellings mean the same. This covers the three heap statements above and the
older `လုပ်ဆောင်မည်` (in `အောက်ပါတို့ကို လုပ်ဆောင်မည်`) and `ပြန်ပေးမည်`:
`အောက်ပါတို့ကို လုပ်ဆောင်မယ်` and `ပြန်ပေးမယ်` work too.

## Lists

| Word | Reads as | Page |
| --- | --- | --- |
| `စာရင်း` | "list": after `<kind>` it makes a list kind (`<kind> စာရင်း`, or `<kind> ၏ စာရင်း`); `တို့ပါသော စာရင်း` closes a list literal | [Lists](../lists/README.md) |
| `အကွက်` | "cell": `x ၏ အကွက် i` is cell `i`, counted from 0; `အကွက် n ခုရှိသော` gives the count of a list kind | [Lists](../lists/README.md) |
| `ခုမြောက်` | "-th": `x ၏ n ခုမြောက်အကွက်` is cell `n`, counted from 1 | [Lists](../lists/README.md#cell-numbers) |
| `ခုရှိသော`, `ရှိသော` | "that has", in `အကွက် n ခုရှိသော <kind> စာရင်း` | [Lists](../lists/README.md#list-kinds) |
| `အကွက်အရေအတွက်` | "cell count", in `x ၏ အကွက်အရေအတွက်` | [Lists](../lists/README.md#cell-numbers) |
| `စာရင်းအလွတ်` | "blank list": each cell at its default | [Lists](../lists/README.md) |
| `တို့ပါသော` | "that hold these", before `စာရင်း` in a list literal | [Lists](../lists/README.md#list-kinds) |
