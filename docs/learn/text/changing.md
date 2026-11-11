---
title: Changing text
description: Add to text at its end or in the middle, remove its last character, replace part of it, and see how text frees its storage.
---

# Changing text

A `စာသား` variable can grow, shrink and change in place. Each change below keeps the text
whole UTF-8.

## Adding to text

`ထဲသို့ ... ကို ထည့်မည်` ("put ... into") adds text or a character at the end of text:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် နှုတ်ဆက် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မင်္ဂလာပါ" ဖြစ်ပါစေ။
နှုတ်ဆက် ထဲသို့ " မောင်မောင်" ကို ထည့်မည်။
နှုတ်ဆက် ထဲသို့ '!' ကို ထည့်မည်။
နှုတ်ဆက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ မောင်မောင်!
```

An add changes the text in place, where `နှုတ်ဆက် ကို နှုတ်ဆက် + "!" ဟု ထားမည်။` builds new
text and puts it in place of the old. Adds build text one piece at a time, such as in a
[counted loop](../control-flow/counted-loops.md):

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လှေကား သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "" ဖြစ်ပါစေ။
၁ မှ ၄ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လှေကား ထဲသို့ '*' ကို ထည့်မည်။
လှေကား ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
*
**
***
****
```

A template literal adds a number: `လှေကား ထဲသို့ "{န}" ကို ထည့်မည်။`. A number with no
quotes around it is an error, since a number is no text:

```zaka-error ZK0401
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "အဆင့် " ဖြစ်ပါစေ။
စာ ထဲသို့ ၁ ကို ထည့်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0401]: expected text, found isize
```

## Adding in the middle and removing the end

`၏ အရှေ့တွင်` ("in front of") after a character puts the added text in front of it, and
`နောက်ဆုံးစာလုံး` ("the last character") names the end that a remove takes off:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "acd" ဖြစ်ပါစေ။
စာ ၏ စာလုံး ၁ ၏ အရှေ့တွင် "b" ကို ထည့်မည်။        // in front of c
စာ ၏ ၁ ခုမြောက်စာလုံး ၏ အရှေ့တွင် '>' ကို ထည့်မည်။   // at the start
စာ ကို ပြသမည်။
စာ ၏ နောက်ဆုံးစာလုံးကို ဖျက်မည်။
စာ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
>abcd
>abc
```

| Zaka | Reads as |
| --- | --- |
| `စာ ၏ စာလုံး ၁ ၏ အရှေ့တွင် "b" ကို ထည့်မည်` | "please add "b" in front of character 1 of the text" |
| `စာ ၏ နောက်ဆုံးစာလုံးကို ဖျက်မည်` | "please remove the last character of the text" |

The number can also be the character count, which adds at the end, as `ထဲသို့` does. A
character is a Unicode code point, so a remove from `ကျွန်` takes off the mark `်` alone.
`နောက်ဆုံးအကွက်` ("the last cell") belongs to lists: see [List
mistakes](../lists/mistakes.md#removing-from-text).

## Replacing part of text

`ဖြင့် အစားထိုးမည်` ("please replace with") puts a value in place of a range of the text.
The range runs from its first character to its last, both included:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် "red apple" ဖြစ်ပါစေ။
စာ ၏ ၁ လုံးမြောက်စာလုံးမှ ၃ လုံးမြောက်စာလုံးထိကို "green" ဖြင့် အစားထိုးမည်။
စာ ကို ပြသမည်။
စာ ၏ အကွက် ၀ မှ ၄ ထိကို "" ဖြင့် အစားထိုးမည်။      // cells 0 to 4: "green"
စာ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
green apple
 apple
```

| Zaka | Reads as |
| --- | --- |
| `စာ ၏ ၁ လုံးမြောက်စာလုံးမှ ၃ လုံးမြောက်စာလုံးထိကို "green" ဖြင့် အစားထိုးမည်` | "please replace the 1st to the 3rd character of the text with "green"" |
| `စာ ၏ အကွက် ၀ မှ ၄ ထိကို "" ဖြင့် အစားထိုးမည်` | "please replace cells 0 to 4 of the text with nothing" |

The character form counts from 1, as `ခုမြောက်စာလုံး` does; the cell form counts the u8s
of the text from 0, as `အကွက်` does, and stops the program at an end that falls inside a
character. The new value can be longer or shorter than the range, a character, or blank.

## No cleanup

Text built when the program runs needs storage of its own, but a program never frees
text. A `စာသား` variable frees its text when its block ends, and when
a new value replaces the old one. A loop frees the text of each round before the next
round starts.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် i သည် ၁ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
i သည် ၄ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လိုင်း သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "အဆင့် {i}: {i * i}" ဖြစ်ပါစေ။
လိုင်း ကို ပြသမည်။
i ကို i + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အဆင့် ၁: ၁
အဆင့် ၂: ၄
အဆင့် ၃: ၉
```

## Hands on

- Build the text `"၁၂၃၄၅"` with a counted loop that adds `"{န}"` in each round.

Next: [Errors and results](../error-handling/README.md).
