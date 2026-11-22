---
title: List mistakes
description: The common mistakes with lists, the message Zaka shows for each one, and the fix.
---

# List mistakes

Some mistakes with lists (dynamic arrays) are caught at compile time. A list can change
length when the program runs, so other errors, such as reading past the end of a list,
can also surface at run time.

## You will learn

- what happens when a cell number (index) is outside the list
- how to loop over a list that might have no cells
- which errors the compiler shows for lists, and how to fix each one

## A cell number outside the list

```zaka-stop
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည်
၇၀၊ ၈၅ နှင့် ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် န သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက် န ကို ပြသမည်။ // the cells are 0, 1 and 2, so 3 does not exist
```

```text
the program stopped: cell 3 is outside the list, whose cells run from 0 to 2
```

A list of 3 cells has the cells 0, 1 and 2, so the VM stops the program at cell 3 and
shows the range of cell numbers that the list has. The count of a list is known when the
program runs, so the VM checks the cell number then, even when we write the number
straight into the program. The same happens when we count from 1 and start at 0:

```zaka-stop
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည်
၇၀၊ ၈၅ နှင့် ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
အမှတ်များ ၏ ၀ ခုမြောက်အကွက် ကို ပြသမည်။ // counting from 1 starts at 1
```

```text
the program stopped: cell 0 is outside the list, whose cells run from 1 to 3
```

The fix is a cell number inside the range, and when the number comes from a variable, we
compare it with `အကွက်အရေအတွက်` before the read.

## A list with no cells and the last number

```zaka-stop
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် စာရင်းအလွတ် ဖြစ်ပါစေ။
၀ မှ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၁ အထိ န အနေဖြင့် ရေတွက်၍
ကိန်းများ ၏ အကွက် န ကို (ကိန်းများ ၏ အကွက် န) * ၂ ဟု ထားမည်။
ကိန်းများ ကို ပြသမည်။
```

```text
the program stopped: the result of - does not fit usize
```

A counted loop from `၀` to `အကွက်အရေအတွက် - ၁` visits each cell number. The cell count is
an unsigned integer (`usize`), and for a list with no cells, `၀ - ၁` is below 0, so the VM
stops the program at the loop.

A loop that just reads the cells can use an each-cell loop instead, which does nothing for
a list with no cells. A loop that changes the cells, or needs their numbers, checks for a
list with no cells first and leaves the function:

```zaka
နှစ်ဆတိုး ဟူသော လုပ်ဆောင်ချက် သည်
ကိန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ (ကိန်းများ ၏ အကွက်အရေအတွက်) == ၀ ဖြစ်လျှင် ပြန်ပေးမည်။
၀ မှ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၁ အထိ န အနေဖြင့် ရေတွက်၍
ကိန်းများ ၏ အကွက် န ကို (ကိန်းများ ၏ အကွက် န) * ၂ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် စာရင်းအလွတ် ဖြစ်ပါစေ။
နှစ်ဆတိုး ဟူသော လုပ်ဆောင်ချက်ကို ကိန်းများ ဖြင့် လုပ်ဆောင်မည်။
ကိန်းများ ကို ပြသမည်။
```

```text
[]
```

## A cell number with a sum

```zaka-stop
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည်
၁၀၊ ၂၀ နှင့် ၃၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက် ၁ + ၂ ကို ပြသမည်။ // cell 3, not cell 1 plus 2
```

```text
the program stopped: cell 3 is outside the list, whose cells run from 0 to 2
```

The cell number includes the whole operator expression after `အကွက်`, so this line reads
cell `၁ + ၂`, which is outside the list. To add 2 to the value of cell 1, we write
`(ကိန်းများ ၏ အကွက် ၁) + ၂`.

## The cell count, written as two words

```zaka-error ZK0301
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက် အရေအတွက် ကို ပြသမည်။
```

```text
error[ZK0301]: "အရေအတွက်" is not declared
```

`အကွက်အရေအတွက်` is one word. With a space, `အကွက် အရေအတွက်` means the cell whose number
is in a variable named `အရေအတွက်`, and no such variable exists here. The fix is to write
`အကွက်အရေအတွက်` without the space.

## A blank list with no data kind

```zaka-error ZK0420
စာရင်းအလွတ် ကို ပြသမည်။
```

```text
error[ZK0420]: a blank list needs a list kind from its place
```

`စာရင်းအလွတ်` ("blank list") gets its data kind (type) from the place where we write it,
such as the variable it sets. Alone after `ပြသမည်`, nothing gives it a data kind. The fix
is to put it in a variable of a list kind first, and print the variable.

## Cells of a value that is no list

```zaka-error ZK0422
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀ ဖြစ်ပါစေ။
အမှတ် ၏ အကွက် ၀ ကို ပြသမည်။
```

```text
error[ZK0422]: isize is not a list, so it has no cells
```

`အမှတ်` is an integer, and an integer has no cells. The fix is to declare it as a list,
such as `ကိန်းပြည့် စာရင်း`, or to read it without `၏ အကွက်`.

## Removing from text

```zaka-error ZK0437
ကိန်းရှင် စာ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "abc" ဖြစ်ပါစေ။
စာ ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။
```

```text
error[ZK0437]: text cannot grow or shrink here: "ထည့်မည်" takes a plain list or text, "နောက်ဆုံးအကွက်ကို ဖျက်မည်" a plain list, and "နောက်ဆုံးစာလုံးကို ဖျက်မည်" text
```

A cell of text is one `u8` of a character, so text has no last cell to remove. The end of
text is its last character, so the fix is `စာ ၏ နောက်ဆုံးစာလုံးကို ဖျက်မည်။` (see [Adding
in the middle and removing the
end](../text/changing.md#adding-in-the-middle-and-removing-the-end)). The counted lists of
Part II give the same error, because these never grow or shrink ([Adding to a counted
list](../forms/counted-lists.md#adding-to-a-counted-list)).

## Removing from a list with no cells

```zaka-stop
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁] ဖြစ်ပါစေ။
ကိန်းများ ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။
ကိန်းများ ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။ // no cell is left to remove
```

```text
the program stopped: the list holds no cells, so it has no last cell to remove
```

After the first remove, the list has no cells, so the second remove has nothing to take
off. The count of a plain list is known when the program runs, so the VM is the one that
stops it. When we do not know the count, we check `ကိန်းများ ၏ အကွက်အရေအတွက်` first.

## Recap

- A cell number outside the list, or a remove from a list with no cells, stops the program
  while it runs.
- For a list that might have no cells, a counted loop up to `အကွက်အရေအတွက် - ၁` needs a
  check first, while an each-cell loop needs none.
- A cell in a sum goes in parentheses, and `အကွက်အရေအတွက်` is one word.
- Text loses its last character with `နောက်ဆုံးစာလုံး`, not `နောက်ဆုံးအကွက်`.

## Hands on

- Read cell 5 of a list of 5 cells through a variable, run it, and read the message.
- Read the same cell counting from 1, as `၅ ခုမြောက်အကွက်`, and compare.
- Remove the last cell of a list of two cells three times, and see which remove stops the
  program.

Next: [Tables](../tables/README.md).
