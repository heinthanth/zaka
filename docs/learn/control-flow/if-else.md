---
title: If and else
description: Run statements when a condition is true.
---

# If and else

An `if` statement runs its inner part when a condition is true, and skips it otherwise.
In Zaka: "if *condition*, then *do this*": `အကယ်၍ <condition> လျှင် <action>`.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၅ ဖြစ်ပါစေ။

// if age is less than 18, then ...
အကယ်၍ အသက် သည် ၁၈ အောက် ငယ်လျှင် "ကလေး" ဟု ပြသမည်။

// nothing happens when the condition is false
အကယ်၍ အသက် သည် ၆၀ ထက် ကြီးလျှင် "အငြိမ်းစား" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အောင် သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် မမှန် ဖြစ်ပါစေ။ // passed?
အကယ်၍ အောင် ဖြစ်လျှင် "အောင်ပါသည်" ဟု ပြသမည်။
သို့မဟုတ်ပါက "ကျပါသည်" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ကျပါသည်
```

## More than two choices

`ထိုသို့မဟုတ်ဘဲ` ("if not so, but ...") adds another condition. Zaka tests the conditions
from top to bottom and runs the **first** branch whose condition is true:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၃ ဖြစ်ပါစေ။
အကယ်၍ အမှတ် သည် ၈၀ အောက် မငယ်လျှင် "အဆင့် က" ဟု ပြသမည်။
ထိုသို့မဟုတ်ဘဲ အမှတ် သည် ၆၀ အောက် မငယ်လျှင် "အဆင့် ခ" ဟု ပြသမည်။
ထိုသို့မဟုတ်ဘဲ အမှတ် သည် ၄၀ အောက် မငယ်လျှင် "အဆင့် ဂ" ဟု ပြသမည်။
သို့မဟုတ်ပါက "အဆင့် ဃ" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အဆင့် ခ
```

73 is also at least 40, but Zaka stops at the first branch that matches.

## Branches with several statements

To run several statements, make the branch a block:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၃ ဖြစ်ပါစေ။
အကယ်၍ အမှတ် % ၂ သည် ၀ နှင့် ညီလျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"စုံကိန်း" ဟု ပြသမည်။
အမှတ် / ၂ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
သို့မဟုတ်ပါက အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မကိန်း" ဟု ပြသမည်။
(အမှတ် + ၁) / ၂ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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
အကယ်၍ မှန် ဖြစ်လျှင် အကယ်၍ မမှန် ဖြစ်လျှင် ၁ ကို ပြသမည်။
```

```text
error[ZK0215]: a single-statement block cannot be another conditional
```

Put the inner `if` in a block instead:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၉၅ ဖြစ်ပါစေ။
အကယ်၍ အမှတ် သည် ၄၀ အောက် မငယ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"အောင်" ဟု ပြသမည်။
အကယ်၍ အမှတ် သည် ၉၀ ထက် ကြီးလျှင် "ဂုဏ်ထူး" ဟု ပြသမည်။     // distinction
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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
