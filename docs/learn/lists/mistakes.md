---
title: List mistakes
description: The mistakes lists make possible, and how Zaka catches each one.
---

# List mistakes

## A cell number outside the list

A list of 3 cells has cells 0, 1 and 2. A cell number past them is a mistake, and the
VM stops the program with the range the list allows:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် န သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက် န ကို ပြသမည်။            // cells 0, 1 and 2 hold values; 3 does not exist
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: cell 3 is outside the list, whose cells run from 0 to 2
```

The count of a list comes when the program runs, so the VM is the one to check it, even
for a number written in place. Counting from 1 starts at 1, not 0:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
အမှတ်များ ၏ ၀ ခုမြောက်အကွက် ကို ပြသမည်။     // counting from 1 starts at 1
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: cell 0 is outside the list, whose cells run from 1 to 3
```

## A list with no cells and the last number

A counted loop from `၀` to `အကွက်အရေအတွက် - ၁` runs the cells of a list. A list with no
cells has a count of 0, and `၀ - ၁` cannot be a count, so the VM stops the program at the
loop:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် စာရင်းအလွတ် ဖြစ်ပါစေ။
၀ မှ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၁ အထိ န အနေဖြင့် ရေတွက်၍ ကိန်းများ ၏ အကွက် န ကို (ကိန်းများ ၏ အကွက် န) * ၂ ဟု ထားမည်။
ကိန်းများ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: the result of - does not fit usize
```

A loop that just reads the cells needs no count: an each-cell loop does nothing for a
list with no cells. A loop that changes the cells, or needs their numbers, checks for a list
with no cells first and leaves the function:

```zaka
လုပ်ဆောင်ချက် နှစ်ဆတိုး သည်
ကိန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ (ကိန်းများ ၏ အကွက်အရေအတွက်) == ၀ ဖြစ်လျှင် ပြန်ပေးမည်။
၀ မှ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၁ အထိ န အနေဖြင့် ရေတွက်၍ ကိန်းများ ၏ အကွက် န ကို (ကိန်းများ ၏ အကွက် န) * ၂ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် စာရင်းအလွတ် ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် နှစ်ဆတိုး ကို ကိန်းများ တို့ဖြင့် လုပ်ဆောင်မည်။
ကိန်းများ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[]
```

## A cell number that takes a sum

The cell number takes the whole operator expression after `အကွက်`, so this reads cell
`၁ + ၂`, which is outside the list:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀၊ ၂၀၊ ၃၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက် ၁ + ၂ ကို ပြသမည်။         // cell 3, not cell 1 plus 2
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: cell 3 is outside the list, whose cells run from 0 to 2
```

Write `(ကိန်းများ ၏ အကွက် ၁) + ၂` to add 2 to cell 1.

## A blank list with no kind

`စာရင်းအလွတ်` ("blank list") takes its kind from where it stands. Standing alone, it has none:

```zaka-error ZK0420
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စာရင်းအလွတ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0420]: a blank list needs a list kind from its place
```

## Cells of a value that is no list

```zaka-error ZK0422
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀ ဖြစ်ပါစေ။
အမှတ် ၏ အကွက် ၀ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0422]: isize is not a list, so it has no cells
```

## Removing from text

Text has no last cell to remove, since a cell of text is one u8 of a character. Its end is
the last character, `နောက်ဆုံးစာလုံး`:

```zaka-error ZK0437
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "abc" ဖြစ်ပါစေ။
စာ ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0437]: text cannot grow or shrink here: "ထည့်မည်" takes a plain list or text, "နောက်ဆုံးအကွက်ကို ဖျက်မည်" a plain list, and "နောက်ဆုံးစာလုံးကို ဖျက်မည်" text
```

The fix is `စာ ၏ နောက်ဆုံးစာလုံးကို ဖျက်မည်။` (see [Adding in the middle and removing the
end](../text/changing.md#adding-in-the-middle-and-removing-the-end)).

The counted lists of Part II neither grow nor shrink: see [Static data and counted
lists](../forms/counted-lists.md#adding-to-a-counted-list).

## Removing from a list with no cells

A list with no cells has no last cell to remove. The count of a plain list comes when the
program runs, so the VM is the one to stop it:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁] ဖြစ်ပါစေ။
ကိန်းများ ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။
ကိန်းများ ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။      // no cell is left to remove
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: the list holds no cells, so it has no last cell to remove
```

Check `ကိန်းများ ၏ အကွက်အရေအတွက်` first when the count is not known.

## The cell count, written as two words

`အကွက်အရေအတွက်` is one word. Written with a space, `အကွက် အရေအတွက်` reads as the cell
numbered with a variable named `အရေအတွက်`.

## Hands on

- Read cell 5 of a list of 5 cells through a variable, run it, and read the message.
- Read the same cell counting from 1, as `၅ ခုမြောက်အကွက်`, and compare.
- Remove the last cell of a list of two cells three times, and see which remove stops the
  program.

Next: [Structs](../structs/README.md).
