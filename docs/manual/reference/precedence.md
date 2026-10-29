---
title: Operator precedence
description: Which operators bind tighter, in one table.
---

# Operator precedence

When an expression has several operators, the ones that bind tighter are worked out
first. Parentheses come before all else. From **tightest** to **loosest**:

| Order | Operators | Groups | Notes |
| --- | --- | --- | --- |
| 1 | literals, names, `( ... )`, function calls | | the building blocks |
| 2 | `-` `အနုတ်` `~` `!` in front of a value | right to left | `-က * ခ` is `(-က) * ခ` |
| 3 | `*` `/` `%` | left to right | |
| 4 | `+` `-` | left to right | |
| 5 | `<<` `>>` | left to right | |
| 6 | `&` | left to right | |
| 7 | `^` | left to right | |
| 8 | `\|` | left to right | |
| 9 | `<` `>` `<=` `>=` | not chainable | `က < ခ < ဂ` is an error |
| 10 | `==` `!=` | not chainable | |
| 11 | word phrases: `... ၏ ပေါင်းလဒ်`, `... ထက် ကြီး`, `... အဖြစ် ပြောင်းလဲသော ရလဒ်`, ... | with particles | each part is a whole operator expression |
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

- A word phrase inside an operator expression needs parentheses:
  `(က နှင့် ခ ၏ ပေါင်းလဒ်) * ဂ`.
- A logical expression (`ဖြစ်၍`, `သို့မဟုတ်`, `&&`, `||`, `မဟုတ်`) used as a value to print
  needs parentheses: `(က ဖြစ်၍ ခ) ကို ပြသပါ။`

See [Grouping and order](../../learn/operators/grouping.md) for worked examples.
