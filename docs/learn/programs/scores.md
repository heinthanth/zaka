---
title: Scores
description: A list of scores walked one cell at a time, a second list built from it, and a name walked one character at a time.
---

# Scores

A class has five students, and each one has a score. This program finds the passing
scores, the count, the total, the highest score and the scores above the average, then
spells out the name of the top student one character at a time.

```zaka
ကိန်းသေ အောင်မှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၄၀ ဖြစ်ပါစေ။

/// The total of the scores.
စုစုပေါင်း ဟူသော လုပ်ဆောင်ချက် သည်
အမှတ်များ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ၀ ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက်တိုင်း ကို အမှတ် အနေဖြင့် ယူ၍ အဖြေ ကို အဖြေ + အမှတ် ဟု ထားမည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

/// The cell number of the highest score, counting from 0.
အမြင့်ဆုံးအမှတ်နေရာ ဟူသော လုပ်ဆောင်ချက် သည်
အမှတ်များ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် နေရာ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက်တိုင်း ကို အမှတ် အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ အမှတ် သည် အမှတ်များ ၏ အကွက် အဖြေ ထက် ကြီးလျှင် အဖြေ ကို နေရာ ဟု ထားမည်။
နေရာ ကို နေရာ + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

/// A new list of the scores at or above the pass mark.
အောင်သောအမှတ်များ ဟူသော လုပ်ဆောင်ချက် သည်
အမှတ်များ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် စာရင်း တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [] ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက်တိုင်း ကို အမှတ် အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ အမှတ် သည် အောင်မှတ် အောက် မငယ်လျှင် အဖြေ ထဲသို့ အမှတ် ကို ထည့်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

/// Print each score above the average.
ပျမ်းမျှအထက် ဟူသော လုပ်ဆောင်ချက် သည်
အမှတ်များ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လည်းကောင်း
ပျမ်းမျှ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အမှတ်များ ၏ အကွက်တိုင်း ကို အမှတ် အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ အမှတ် သည် ပျမ်းမျှ ထက် ကြီးလျှင် အမှတ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

/// Print a name one character at a time.
စာလုံးချင်း ဟူသော လုပ်ဆောင်ချက် သည်
အမည် ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အမည် ၏ စာလုံးတိုင်း ကို စ အနေဖြင့် ယူ၍ စ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// The program starts here
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၇၂၊ ၃၅၊ ၉၁၊ ၅၈၊ ၂၄] ဖြစ်ပါစေ။
ကိန်းရှင် အမည်များ သည် စာသား စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ["အောင်အောင်"၊ "သီတာ"၊ "မြမြ"၊ "ဇော်ဇော်"၊ "နန္ဒာ"] ဖြစ်ပါစေ။

ကိန်းရှင် အောင်မှတ်မီအမှတ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အမှတ်များ ၏ အောင်သောအမှတ်များ ဖြစ်ပါစေ။
အောင်မှတ်မီအမှတ်များ ကို ပြသမည်။
အောင်မှတ်မီအမှတ်များ ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။
အောင်မှတ်မီအမှတ်များ ကို ပြသမည်။

ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (အမှတ်များ ၏ အကွက်အရေအတွက်) ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် ပေါင်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အမှတ်များ ၏ စုစုပေါင်း ဖြစ်ပါစေ။
ကိန်းရှင် ထိပ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အမှတ်များ ၏ အမြင့်ဆုံးအမှတ်နေရာ ဖြစ်ပါစေ။
"အရေအတွက် {အရေအတွက်}" ဟု ပြသမည်။
"စုစုပေါင်း {ပေါင်း}" ဟု ပြသမည်။
"အများဆုံး {အမှတ်များ ၏ အကွက် ထိပ်}" ဟု ပြသမည်။
"ပျမ်းမျှ {ပေါင်း / အရေအတွက်} ထက် ကြီးသော အမှတ်များ" ဟု ပြသမည်။
ပျမ်းမျှအထက် ဟူသော လုပ်ဆောင်ချက်ကို အမှတ်များ နှင့် ပေါင်း / အရေအတွက် တို့ဖြင့် လုပ်ဆောင်မည်။
စာလုံးချင်း ဟူသော လုပ်ဆောင်ချက်ကို အမည်များ ၏ အကွက် ထိပ် ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၇၂၊ ၉၁၊ ၅၈]
[၇၂၊ ၉၁]
အရေအတွက် ၅
စုစုပေါင်း ၂၈၀
အများဆုံး ၉၁
ပျမ်းမျှ ၅၆ ထက် ကြီးသော အမှတ်များ
၇၂
၉၁
၅၈
မ
ြ
မ
ြ
```

## How it works

- **Two lists, next to each other,** have the data: `အမည်များ` (the names) and `အမှတ်များ` (the
  scores). Cell 2 of each list belongs to the same student.
- **Each function walks its list once** with the [loop over each
  cell](../lists/README.md#walking-a-list), so no cell number can run past the end.
  `အမြင့်ဆုံးအမှတ်နေရာ` keeps its own count in `နေရာ` to know which cell has the highest score.
- **`အောင်သောအမှတ်များ` builds a new list.** `အဖြေ` starts as `[]`, a list with no cells,
  and [each add](../lists/growing.md#building-a-list) puts a passing score at its end. The
  program start block then removes the last cell, and prints the list before and after.
- **A returned list is a separate value.** The list that `အောင်သောအမှတ်များ` returns
  belongs to `အောင်မှတ်မီအမှတ်များ`, so removing a cell from it leaves `အမှတ်များ` as it was.
- **The average is a whole number.** `ပေါင်း / အရေအတွက်` divides two integers, so 280 / 5
  gives 56. The count of cells is a `usize`, an integer with no sign, so the program
  start block [converts](../operators/conversion.md) it to `ကိန်းပြည့်` first.
- **The name walk** uses the [loop over each character](../text/characters.md#characters).
  A Burmese letter and the mark on it are two characters, so `မြမြ` prints four lines.
- **The calls on the list** use the [`၏` form](../../manual/language/functions-calls.md#calls-on-a-value).
  For example, `အမှတ်များ ၏ စုစုပေါင်း` reads as “the total of the scores”.
- **No list is copied.** Each call passes a list to a function that never writes it, so the
  function reads the caller's cells in place ([What a duplicate
  costs](../forms/costs.md#where-nothing-is-copied)).

## Hands on

- Change the pass mark `အောင်မှတ်` to 60. Which scores remain?
- Write a function that counts the scores below the pass mark, and print the count.
- Print the name of the student with the lowest score.

Next: [Where a program starts](start.md).
