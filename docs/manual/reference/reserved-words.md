---
title: Reserved words
description: Every word Zaka gives a meaning to, with an English reading.
---

# Reserved words

These words have a fixed meaning in Zaka, so they cannot be used as names. A word made
only of reserved words, such as `ထက်ကြီးသည်` or `တန်ဖိုးကို`, is split into them. Wrap
a name in backticks (`` `ကြီး` ``) if you really need one of these
([Names](../variables/names.md)).

## Particles

| Word | Reads as | Used in |
| --- | --- | --- |
| `သည်` | (subject); also a polite ending | [Particles](../sentences/particles.md) |
| `ကို` | (object) | [Particles](../sentences/particles.md) |
| `နှင့်` | "and", "with" | [Arithmetic](../operators/arithmetic.md), [Comparison](../operators/comparison.md) |
| `၏` | "of" | [Arithmetic](../operators/arithmetic.md) |
| `ဖြင့်` | "by" | [Arithmetic](../operators/arithmetic.md), [Writing integers](../values/literals.md) |
| `ထက်` | "than" | [Comparison](../operators/comparison.md) |
| `အောက်` | "below" | [Comparison](../operators/comparison.md) |
| `ဟု` | "so" (quoting) | [Printing](../hello/printing.md), [Changing a variable](../variables/assignment.md) |
| `အဖြစ်` | "as", "into" | [Converting between kinds](../operators/conversion.md) |
| `ပါ`, `ပါသည်` | polite endings | [Particles](../sentences/particles.md) |
| `တို့ကို`, `တို့ဖြင့်` | close a list of parameters / arguments | [Parameters](../functions/parameters.md) |
| `လက်ခံပြီး` | "accepting ... and" | [Parameters](../functions/parameters.md) |
| `မှ`, `အထိ`, `စီ`, `အနေဖြင့်` | "from", "up to", "each", "as" | [Counted loops](../control-flow/counted-loops.md) |
| `ဖြစ်ပြီး` | "being ... and" | [Generic loops](../control-flow/generic-loops.md) |
| `ပြီးလျှင်` | "when done" | [Generic loops](../control-flow/generic-loops.md) |
| `ဘယ်သို့`, `ညာသို့` | "to the left", "to the right" | [Bitwise operators](../operators/bitwise.md) |
| `ဘစ်` | "bit" | [Integers](../values/integers.md), [Bitwise operators](../operators/bitwise.md) |

## Verbs that make a value

| Word | Reads as | Symbol | Page |
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
| `ဖြစ်၍` | "and" | `&&` | [Logic](../operators/logic.md) |
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
| `ကိန်းပြည့်` | "integer" | [Integers](../values/integers.md) |
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
| `တန်ဖိုး` | "value" (kept for later versions) | |
