---
title: If and else
description: Run statements when a condition is true.
---

# If and else

An `if` statement runs its inner part when a condition is true, and skips it otherwise.
In Zaka: "if *condition*, then *do this*": `အကယ်၍ <condition> လျှင် <action>`.

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၅ ဖြစ်ပါစေ။

// if age is less than 18, then ...
အကယ်၍ အသက် သည် ၁၈ အောက် ငယ်လျှင် "ကလေး" ဟု ပြသပါ။

// nothing happens when the condition is false
အကယ်၍ အသက် သည် ၆၀ ထက် ကြီးလျှင် "အငြိမ်းစား" ဟု ပြသပါ။
```

```text
ကလေး
```

| Zaka | Reads as |
| --- | --- |
| `အကယ်၍` | "if" |
| `လျှင်` | "then"; joined to the condition's last verb (`ငယ်လျှင်`) |
| `ဖြစ်လျှင်` | "if so, then"; a separate word, used after a variable or a full sentence |
| `ထိုသို့မဟုတ်ဘဲ` | "if not so, but"; else if |
| `သို့မဟုတ်ပါက` | "otherwise"; else |

The condition must be a [boolean](../values/booleans.md): a comparison, a logical
expression, or a boolean variable. With a boolean variable, use `ဖြစ်လျှင်`:
`အကယ်၍ အောင် ဖြစ်လျှင် ...`.

## Otherwise

`သို့မဟုတ်ပါက` ("otherwise") names what to run when the condition is false:

```zaka
ကိန်းရှင် အောင် သည် မှန်မှားတန်ဖိုး ဖြစ်၍ တန်ဖိုးသည် မမှန် ဖြစ်ပါစေ။     // passed?
အကယ်၍ အောင် ဖြစ်လျှင် "အောင်ပါသည်" ဟု ပြသပါ။
သို့မဟုတ်ပါက "ကျပါသည်" ဟု ပြသပါ။
```

```text
ကျပါသည်
```

## More than two choices

`ထိုသို့မဟုတ်ဘဲ` ("if not so, but ...") adds another condition. Zaka tests the conditions
from top to bottom and runs the **first** branch whose condition is true:

```zaka
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၇၃ ဖြစ်ပါစေ။
အကယ်၍ အမှတ် သည် ၈၀ အောက် မငယ်လျှင် "အဆင့် က" ဟု ပြသပါ။
ထိုသို့မဟုတ်ဘဲ အမှတ် သည် ၆၀ အောက် မငယ်လျှင် "အဆင့် ခ" ဟု ပြသပါ။
ထိုသို့မဟုတ်ဘဲ အမှတ် သည် ၄၀ အောက် မငယ်လျှင် "အဆင့် ဂ" ဟု ပြသပါ။
သို့မဟုတ်ပါက "အဆင့် ဃ" ဟု ပြသပါ။
```

```text
အဆင့် ခ
```

73 is also at least 40, but Zaka stops at the first branch that matches.

## Branches with several statements

To run several statements, make the branch a block:

```zaka
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၇၃ ဖြစ်ပါစေ။
အကယ်၍ အမှတ် % ၂ သည် ၀ နှင့် ညီလျှင် အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
"စုံကိန်း" ဟု ပြသပါ။
အမှတ် / ၂ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
သို့မဟုတ်ပါက အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
"မကိန်း" ဟု ပြသပါ။
(အမှတ် + ၁) / ၂ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
```

```text
မကိန်း
၃၇
```

`စုံကိန်း` is "even number" and `မကိန်း` is "odd number".

## An if inside an if

A one-statement branch cannot be another `if`. That would make it unclear which `if` an
`otherwise` belongs to:

```zaka-error ZK0215
အကယ်၍ မှန် ဖြစ်လျှင် အကယ်၍ မမှန် ဖြစ်လျှင် ၁ ကို ပြသပါ။
```

```text
error[ZK0215]: a single-statement block cannot be another conditional
```

Put the inner `if` in a block instead:

```zaka
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၉၅ ဖြစ်ပါစေ။
အကယ်၍ အမှတ် သည် ၄၀ အောက် မငယ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
"အောင်" ဟု ပြသပါ။
အကယ်၍ အမှတ် သည် ၉၀ ထက် ကြီးလျှင် "ဂုဏ်ထူး" ဟု ပြသပါ။     // distinction
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
```

```text
အောင်
ဂုဏ်ထူး
```

## Hands on

- Print whether a number is positive, negative, or zero.
- Print `"FizzBuzz"` if 15 divides a number, `"Fizz"` if 3 divides it, `"Buzz"` if 5
  does, and the number itself otherwise. Check 9, 10, 15 and 7. Does the order of the
  tests matter?

Next: [While loops](while.md).
