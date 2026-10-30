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
| `ဖြင့်` | "with", "through" | [Arithmetic](../../learn/operators/arithmetic.md), [Writing integers](../../learn/values/literals.md) |
| `ထက်` | "than" | [Comparison](../../learn/operators/comparison.md) |
| `အောက်` | "below" | [Comparison](../../learn/operators/comparison.md) |
| `ဟု` | "so" (quoting) | [Printing](../../learn/hello/printing.md), [Changing a variable](../../learn/variables/assignment.md) |
| `အဖြစ်` | "as", "into" | [Converting between kinds](../../learn/operators/conversion.md) |
| `ပါ`, `ပါသည်` | polite endings | [Particles](../../learn/sentences/particles.md) |
| `တို့ကို`, `တို့ဖြင့်` | close a list of parameters / arguments | [Parameters](../../learn/functions/parameters.md) |
| `လက်ခံပြီး` | "accepting ... and" | [Parameters](../../learn/functions/parameters.md) |
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
| `ရလဒ်အဖြစ် ပြန်ပေးပါ` | "give back as the result" | [Results](../../learn/functions/results.md) |
| `ပြန်ပေးပါ` | "please return" | [Functions](../../learn/functions/README.md) |
| `ပြန်ပေးမည်` | "will return" | [Results](../../learn/functions/results.md) |
| `ရပ်ပါ` | "please stop" | [Break and continue](../../learn/control-flow/break-continue.md) |
| `ကျော်ပါ` | "please skip" | [Break and continue](../../learn/control-flow/break-continue.md) |

## Blocks and control flow

| Word | Reads as | Page |
| --- | --- | --- |
| `အောက်ပါတို့ကို လုပ်ဆောင်ပါ` | "please do the following" | [Blocks and scope](../../learn/variables/scope.md) |
| `အောက်ပါတို့ကို လုပ်ဆောင်မည်` | "will do the following" | [Functions](../../learn/functions/README.md) |
| `အောက်ပါတို့ကို လုပ်ဆောင်ကာ` | "doing the following, ..." | [Results](../../learn/functions/results.md) |
| `လုပ်ဆောင်စရာများ ပြီးပါပြီး` | "the tasks are done" (four spellings) | [Blocks and scope](../../learn/variables/scope.md) |
| `အကယ်၍` | "if" | [If and else](../../learn/control-flow/if-else.md) |
| `လျှင်`, `ဖြစ်လျှင်` | "then" | [If and else](../../learn/control-flow/if-else.md) |
| `ထိုသို့မဟုတ်ဘဲ` | "if not so, but" (else if) | [If and else](../../learn/control-flow/if-else.md) |
| `သို့မဟုတ်ပါက` | "otherwise" (else) | [If and else](../../learn/control-flow/if-else.md) |
| `နေသရွေ့`, `ဖြစ်နေသရွေ့` | "as long as" | [While loops](../../learn/control-flow/while.md) |
| `ရေတွက်၍`, `တိုး၍`, `လျှော့၍` | "counting", "increasing", "decreasing" | [Counted loops](../../learn/control-flow/counted-loops.md) |
| `ကွင်းဆက်` | "loop" | [Break and continue](../../learn/control-flow/break-continue.md) |
| `နောက်တစ်ကြိမ်ဆီသို့` | "to the next round" | [Break and continue](../../learn/control-flow/break-continue.md) |

## Kinds, declarations and values

| Word | Reads as | Page |
| --- | --- | --- |
| `ကိန်းပြည့်` | "integer" | [Integers](../../learn/values/integers.md) |
| `ဒဿမကိန်း` | "decimal number" | [Decimal numbers](../../learn/values/floats.md) |
| `စာလုံး` | "character" | [Characters](../../learn/values/characters.md) |
| `မှန်မှားတန်ဖိုး`, `ဘူလီယန်` | "true/false value", "boolean" | [True and false](../../learn/values/booleans.md) |
| `လက္ခဏာရှိ`, `လက္ခဏာမဲ့` | "signed", "unsigned" | [Integers](../../learn/values/integers.md) |
| `အခြေ` | "base" | [Writing integers](../../learn/values/literals.md) |
| `ကိန်းရှင်` | "variable" | [Variables](../../learn/variables/README.md) |
| `ကိန်းသေ` | "constant" | [Constants](../../learn/variables/constants.md) |
| `လုပ်ဆောင်ချက်` | "function" | [Functions](../../learn/functions/README.md) |
| `ရလဒ်` | "result" | [Results](../../learn/functions/results.md) |
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

The words inside the three statements (`နေရာယူ၍`, `တွင်`, `သိမ်းထားမည်`, `ရှိ`,
`ဖြစ်အောင်`, `နေရာအချဲ့အချုံ့လုပ်မည်`, `ပြန်လည်စွန့်လွှတ်မည်`) are reserved on their own
as well.

## Lists

| Word | Reads as | Page |
| --- | --- | --- |
| `စာရင်း` | "list": after `<kind>` it makes a list kind (`<kind> စာရင်း`, or `<kind> ၏ စာရင်း`); `တို့ပါသော စာရင်း` closes a list literal | [Lists](../../learn/lists/README.md) |
| `အကွက်` | "cell": `x ၏ အကွက် i` is cell `i`, counted from 0; `အကွက် n ခုရှိသော` gives the count of a list kind | [Lists](../../learn/lists/README.md) |
| `ခုမြောက်` | "-th": `x ၏ n ခုမြောက်အကွက်` is cell `n`, counted from 1 | [Lists](../../learn/lists/README.md#cell-numbers) |
| `ခုရှိသော`, `ရှိသော` | "that has", in `အကွက် n ခုရှိသော <kind> စာရင်း` | [Lists](../../learn/lists/README.md#list-kinds) |
| `အကွက်အရေအတွက်` | "cell count", in `x ၏ အကွက်အရေအတွက်` | [Lists](../../learn/lists/README.md#cell-numbers) |
| `စာရင်းအလွတ်` | "blank list": each cell at its default | [Lists](../../learn/lists/README.md) |
| `တို့ပါသော` | "that hold these", before `စာရင်း` in a list literal | [Lists](../../learn/lists/README.md#list-kinds) |

## Structs

| Word | Reads as | Page |
| --- | --- | --- |
| `ဒေတာအမျိုးအစား`, `အချက်အလက်အမျိုးအစား` | "data kind": starts a struct declaration (two interchangeable spellings) | [Structs](../../learn/structs/README.md) |
| `တို့ ပါဝင်သည်` | "these are included": closes the field list of a struct | [Structs](../../learn/structs/README.md) |
| `တို့ ပါဝင်ပြီး` | "these are included, and": closes the field list when a classifier follows | [Structs](../../learn/structs/README.md#pointers-and-the-heap) |
| `မျိုးပြပစ္စည်းမှာ` | "the count word is": names the classifier of a struct | [Structs](../../learn/structs/README.md#pointers-and-the-heap) |
| `ဖြစ်သည်` | "is": ends a struct declaration that names a classifier | [Structs](../../learn/structs/README.md#pointers-and-the-heap) |

`တို့` on its own is reserved as well, so it cannot be used as a name. The words inside
the two closing phrases (`တို့`, `ပါဝင်သည်`, `ပါဝင်ပြီး`) are reserved on their own.
