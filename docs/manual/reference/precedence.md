---
title: Operator precedence
description: Which operators bind tighter, in one table.
---

# Operator precedence

When an expression has several operators, Zaka works out the operators that bind tighter
first, and parentheses come before all of them. The table lists the levels (precedence)
from the **tightest** to the **loosest**, with the direction in which operators of one
level group. [Grouping and order](../../learn/operators/grouping.md)
teaches the same rules through examples.

| Level | Operators | Grouping | Notes |
| --- | --- | --- | --- |
| 1 | literals, names, `( ... )`, function calls | | the values that operators work on |
| 2 | `-` `အနုတ်` `~` `!` in front of a value; `*` and `&` in front of a value in [unsafe code](../language/pointers.md#reading-and-writing-through-a-pointer) | right to left | `-က * ခ` is `(-က) * ခ`; `*ပ + ၁` is `(*ပ) + ၁` |
| 3 | `*` `/` `%` | left to right | |
| 4 | `+` `-` | left to right | |
| 5 | `<<` `>>` | left to right | |
| 6 | `&` | left to right | |
| 7 | `^` | left to right | |
| 8 | `\|` | left to right | |
| 9 | `<` `>` `<=` `>=` | no chains | `က < ခ < ဂ` is an error |
| 10 | `==` `!=` | no chains | |
| 11 | word operators: `... ၏ ပေါင်းလဒ်`, `... ထက် ကြီး`, `... အဖြစ် ပြောင်းလဲသော ရလဒ်`, ... | through their particles | each operand is a whole sign expression |
| 12 | `မဟုတ်` after a value | | `က == ခ မဟုတ်ပါ` is `!(က == ခ)` |
| 13 | `ဖြစ်၍` `&&` | left to right | |
| 14 | `သို့မဟုတ်` `\|\|` | left to right | |

## Examples

| Written | Means |
| --- | --- |
| `၂ + ၃ * ၄` | `၂ + (၃ * ၄)` = 14 |
| `၁၀ - ၃ - ၂` | `(၁၀ - ၃) - ၂` = 5 |
| `က + ခ နှင့် ဂ ၏ မြှောက်လဒ်` | `(က + ခ) * ဂ` |
| `!က == ခ` | `(!က) == ခ` |
| `က == ခ မဟုတ်ပါ` | `!(က == ခ)` |
| `က သို့မဟုတ် ခ ဖြစ်၍ ဂ` | `က သို့မဟုတ် (ခ ဖြစ်၍ ဂ)` |

## Moving between levels

Two cases need parentheses because the parts come from different levels:

- To use a word operator inside a sign expression, we put the word operator in
  parentheses: `(က နှင့် ခ ၏ ပေါင်းလဒ်) * ဂ`.
- To print a logical expression (`ဖြစ်၍`, `သို့မဟုတ်`, `&&`, `||`, `မဟုတ်`), or to use it
  where a value goes, such as the value of a declaration, we put it in parentheses:
  `(က ဖြစ်၍ ခ) ကို ပြသမည်။`
