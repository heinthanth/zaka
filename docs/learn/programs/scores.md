---
title: Scores
description: A list of scores walked one cell at a time, a second list built from it, and a name walked one character at a time.
---

# Scores

A class has five students and a score for each. The program finds the passing scores,
the count, the total, the highest score and the scores above the average, then spells
out the name of the top student one character at a time.

```zaka
ကိန်းသေ အောင်မှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၄၀ ဖြစ်ပါစေ။

/// The total of the scores.
လုပ်ဆောင်ချက် စုစုပေါင်း သည်
ကိန်းပြည့် စာရင်း အမှတ်များ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
အမှတ်များ ၏ အကွက်တိုင်း ကို အမှတ် အနေဖြင့် ယူ၍ ရလဒ် ကို ရလဒ် + အမှတ် ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

/// The cell number of the highest score, counting from 0.
လုပ်ဆောင်ချက် ထိပ်ဆုံး သည်
ကိန်းပြည့် စာရင်း အမှတ်များ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
ကိန်းရှင် နေရာ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက်တိုင်း ကို အမှတ် အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
အကယ်၍ အမှတ် သည် အမှတ်များ ၏ အကွက် ရလဒ် ထက် ကြီးလျှင် ရလဒ် ကို နေရာ ဟု ထားပါ။
နေရာ ကို နေရာ + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

/// A new list of the scores at or above the pass mark.
လုပ်ဆောင်ချက် အောင်သောအမှတ်များ သည်
ကိန်းပြည့် စာရင်း အမှတ်များ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် စာရင်း ရလဒ် ကို ပြန်ပေးမည်။
အမှတ်များ ၏ အကွက်တိုင်း ကို အမှတ် အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
အကယ်၍ အမှတ် သည် အောင်မှတ် အောက် မငယ်လျှင် ရလဒ် ထဲသို့ အမှတ် ကို ထည့်ပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

/// Print each score above the average.
လုပ်ဆောင်ချက် ပျမ်းမျှအထက် သည်
ကိန်းပြည့် စာရင်း အမှတ်များ နှင့် ကိန်းပြည့် ပျမ်းမျှ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အမှတ်များ ၏ အကွက်တိုင်း ကို အမှတ် အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
အကယ်၍ အမှတ် သည် ပျမ်းမျှ ထက် ကြီးလျှင် အမှတ် ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

/// Print a name one character at a time.
လုပ်ဆောင်ချက် စာလုံးချင်း သည်
စာသား အမည် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အမည် ၏ စာလုံးတိုင်း ကို စ အနေဖြင့် ယူ၍ စ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၇၂၊ ၃၅၊ ၉၁၊ ၅၈၊ ၂၄] ဖြစ်ပါစေ။

// The program starts here
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည်များ သည် စာသား စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ["အောင်အောင်"၊ "သီတာ"၊ "မြမြ"၊ "ဇော်ဇော်"၊ "နန္ဒာ"] ဖြစ်ပါစေ။

ကိန်းရှင် အောင်သူ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အောင်သောအမှတ်များ(အမှတ်များ) ဖြစ်ပါစေ။
အောင်သူ ကို ပြသပါ။
အောင်သူ ၏ နောက်ဆုံးအကွက်ကို ဖျက်ပါ။
အောင်သူ ကို ပြသပါ။

ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (အမှတ်များ ၏ အကွက်အရေအတွက်) ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် ပေါင်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် စုစုပေါင်း(အမှတ်များ) ဖြစ်ပါစေ။
ကိန်းရှင် ထိပ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ထိပ်ဆုံး(အမှတ်များ) ဖြစ်ပါစေ။
"အရေအတွက် {အရေအတွက်}" ဟု ပြသပါ။
"စုစုပေါင်း {ပေါင်း}" ဟု ပြသပါ။
"အများဆုံး {အမှတ်များ ၏ အကွက် ထိပ်}" ဟု ပြသပါ။
"ပျမ်းမျှ {ပေါင်း / အရေအတွက်} ထက် ကြီးသော အမှတ်များ" ဟု ပြသပါ။
လုပ်ဆောင်ချက် ပျမ်းမျှအထက် ကို အမှတ်များ၊ ပေါင်း / အရေအတွက် တို့ဖြင့် လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် စာလုံးချင်း ကို အမည်များ ၏ အကွက် ထိပ် တို့ဖြင့် လုပ်ဆောင်ပါ။
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

- **Two lists, one beside the other,** hold the data: `အမည်များ` (the names) and `အမှတ်များ` (the
  scores). Cell 2 of each belongs to the same student.
- **Each function walks its list once** with the
  [each-cell loop](../lists/README.md#walking-a-list), so no cell number can run past the
  end. `ထိပ်ဆုံး` keeps its own count in `နေရာ` to know which cell holds the highest score.
- **`အောင်သောအမှတ်များ` builds a new list.** `ရလဒ်` starts as a list with no cells, and
  [each add](../lists/growing.md#building-a-list) puts a passing score at its end. The
  start block then removes the last cell, and prints the list before and after.
- **The results are values of their own.** The list that `အောင်သောအမှတ်များ` gives back
  belongs to `အောင်သူ`; removing a cell from it leaves `အမှတ်များ` as it was.
- **The average is a whole number.** `ပေါင်း / အရေအတွက်` divides two integers, so 280 / 5
  gives 56. The count of cells is a `usize`; the start block
  [converts](../operators/conversion.md) it to `ကိန်းပြည့်` first.
- **The name walk** uses the [each-character loop](../text/characters.md#characters). A
  Burmese letter and the mark on it are two characters, so `မြမြ` prints four lines.
- **The calls** use the [sign form](../functions/results.md#the-sign-form), `f(x)`, where
  the result goes into a variable.

## Hands on

- Change the pass mark `အောင်မှတ်` to 60. Which scores remain?
- Write a function that counts the scores below the pass mark, and print the count.
- Print the name of the student with the lowest score.

The same program can read its lists through views, which state in each signature that the
function reads alone, and build its passing list on the heap: see [the end of What a
duplicate costs](../forms/costs.md#the-scores-program-with-no-duplicate).
