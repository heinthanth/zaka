---
title: Changing text
description: Add to text at its end or in the middle, remove its last character, replace part of it, and see when text frees its storage.
---

# Changing text

A text (string) variable can grow, shrink and change in place, so we do not need to build
new text for each change. Each change below keeps the text whole UTF-8, so it never breaks a
character in two.

## You will learn

- how to add text or a character at the end of text
- how to add in front of a character, and how to remove the last character
- how to replace a range of characters or cells
- when the storage of text is freed

## Adding to text

```text
<text> ထဲသို့ <value> ကို ထည့်မည်။
```

To add text or a character at the end of a text variable, we write the variable,
`ထဲသို့` ("into"), the value, and `ကို ထည့်မည်` ("will put"):

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

| Zaka | Reads as |
| --- | --- |
| `နှုတ်ဆက် ထဲသို့` | "into `နှုတ်ဆက်`" |
| `" မောင်မောင်" ကို` | "` မောင်မောင်`", the value to add |
| `ထည့်မည်` | "will put" |

An add changes the text in place, while `နှုတ်ဆက် ကို နှုတ်ဆက် + "!" ဟု ထားမည်။` builds new
text and puts it in place of the old. Adds are a good fit for building text one piece at a
time, such as in a [counted loop](../control-flow/counted-loops.md):

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

The value must be text or a character. To add a number, we put it in a template slot:
`လှေကား ထဲသို့ "{န}" ကို ထည့်မည်။`. A number with no quotes around it is a mistake,
because a number is not text:

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

```text
<text> ၏ စာလုံး <number> ၏ အရှေ့တွင် <value> ကို ထည့်မည်။
<text> ၏ နောက်ဆုံးစာလုံးကို ဖျက်မည်။
```

To add in the middle, we name a character and write `၏ အရှေ့တွင်` ("in front of") after it,
so the new text goes in front of that character. To remove the last character, we use
`နောက်ဆုံးစာလုံး` ("the last character") with `ဖျက်မည်` ("will remove"):

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
| `စာ ၏ စာလုံး ၁ ၏ အရှေ့တွင် "b" ကို ထည့်မည်` | "will put "b" in front of character 1 of the text" |
| `စာ ၏ နောက်ဆုံးစာလုံးကို ဖျက်မည်` | "will remove the last character of the text" |

As with a read, `၏ စာလုံး` counts from 0 and `၏ n ခုမြောက်စာလုံး` counts from 1. A
character is a Unicode code point, so a remove from `ကျွန်` removes the mark `်` alone.

## Replacing part of text

```text
<text> ၏ <start> လုံးမြောက်စာလုံးမှ <end> လုံးမြောက်စာလုံးထိကို <value> ဖြင့် အစားထိုးမည်။
<text> ၏ အကွက် <start> မှ <end> ထိကို <value> ဖြင့် အစားထိုးမည်။
```

To put a value in place of a range of the text, we name the range and write
`ဖြင့် အစားထိုးမည်` ("will replace with"). The range runs from its first character to its
last, and both ends are part of it:

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
| `စာ ၏ ၁ လုံးမြောက်စာလုံးမှ ၃ လုံးမြောက်စာလုံးထိကို "green" ဖြင့် အစားထိုးမည်` | "will replace the 1st to the 3rd character of the text with "green"" |
| `စာ ၏ အကွက် ၀ မှ ၄ ထိကို "" ဖြင့် အစားထိုးမည်` | "will replace cells 0 to 4 of the text with nothing" |

The first form counts characters from 1, as `ခုမြောက်စာလုံး` does. The second form counts
cells from 0, as `အကွက်` does, and it stops the program when an end falls inside a
character. The new value can be longer or shorter than the range, a character, or blank
text.

## No cleanup

A program never frees text itself. Text that is built when the program runs needs storage,
and a `စာသား` variable frees that storage when its block ends, or when a new value replaces
the old one. A loop frees the text of each round before the next round starts:

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

Here each round builds a new `လိုင်း`, and the program frees it at the end of the round.

## Details

- The value of an add is read before the text changes, so `t ထဲသို့ t ကို ထည့်မည်။`
  doubles `t` ([Changing text](../../manual/language/text.md#changing-text)).
- In the form with `၏ အရှေ့တွင်`, the number can also be the count of characters, which adds
  at the end, while a larger number stops the program.
- Removing the last character of text with no characters stops the program.
- `နောက်ဆုံးအကွက်` ("the last cell") removes from a list, and on text it is the mistake
  ZK0437 ([List mistakes](../lists/mistakes.md#removing-from-text)).
- In a replace, `ခုမြောက်` is a second spelling of `လုံးမြောက်`. An end outside the text, or an
  end before the start, stops the program.
- A function that adds to its text parameter changes its own duplicate, and the caller's
  text remains as it was ([Duplicating text](README.md#duplicating-text)).
- When text grows past its storage, it moves to a block twice the size, so a run of adds
  moves the text a few times, not once for each add. The rules on when text is freed are
  in [Drops](../../manual/language/text-storage.md#drops).

## Recap

- We use `<text> ထဲသို့ <value> ကို ထည့်မည်။` to add text or a character at the end.
- We use `၏ အရှေ့တွင်` to add in front of a character, and `၏ နောက်ဆုံးစာလုံးကို ဖျက်မည်။` to
  remove the last character.
- We use `… မှ … ထိကို <value> ဖြင့် အစားထိုးမည်။` to replace a range of characters or cells.
- A text variable frees its storage at the end of its block, or when it gets a new value.

## Hands on

- Build the text `"၁၂၃၄၅"` with a counted loop that adds `"{န}"` in each round.
- Remove the last character of `"ကျွန်"` twice, and print the text after each remove.
- Replace the second word of `"red apple pie"` with `"green"`.

Next: [Errors and results](../error-handling/README.md).
