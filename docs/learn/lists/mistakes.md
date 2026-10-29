---
title: List mistakes
description: The mistakes lists make possible, and how Zaka catches each one.
---

# List mistakes

## A cell number outside the list

A list of 3 cells has cells 0, 1 and 2. A cell number past them is a mistake. Here the
number sits in a variable, and the compiler still works out that it is 3:

```zaka-error ZK0710
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် န သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက် န ကို ပြသပါ။            // cells 0, 1 and 2 hold values; 3 does not exist
```

```text
error[ZK0710]: offset 3 is past the end of a block of 3 elements
```

With `--mem-check warn`, or **Warn** in the Explorer, the program runs and the machine
stops it, with the range the list allows:

```zaka-stop mem-check=warn
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် န သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက် န ကို ပြသပါ။            // cells 0, 1 and 2 hold values; 3 does not exist
```

```text
the program stopped: cell 3 is outside the list, whose cells run from 0 to 2
```

When the number comes from input, the machine's check is the one that catches it. When
the number is written in place, as a constant, Zaka reports it with another code, in each
mode. Counting from 1 starts at 1, not 0:

```zaka-error ZK0603
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
အမှတ်များ ၏ ၀ ခုမြောက်အကွက် ကို ပြသပါ။     // counting from 1 starts at 1
```

```text
error[ZK0603]: cell 0 is outside the list, whose cells run from 1 to 3
```

## A cell number that takes a sum

The cell number takes the whole operator expression after `အကွက်`, so this reads cell
`၁ + ၂`, which is outside the list:

```zaka-error ZK0603
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၁၀၊ ၂၀၊ ၃၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက် ၁ + ၂ ကို ပြသပါ။         // cell 3, not cell 1 plus 2
```

```text
error[ZK0603]: cell 3 is outside the list, whose cells run from 0 to 2
```

Write `(ကိန်းများ ၏ အကွက် ၁) + ၂` to add 2 to cell 1.

## A literal with the wrong count

```zaka-error ZK0419
ကိန်းရှင် အမှတ်များ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
```

```text
error[ZK0419]: the kind holds 3 cells, but the list has 2
```

## A blank list with no kind

`စာရင်းအလွတ်` takes its kind from where it stands. Standing alone, it has none:

```zaka-error ZK0420
စာရင်းအလွတ် ကို ပြသပါ။
```

```text
error[ZK0420]: a blank list needs a list kind from its place
```

## A list kind with no count

A kind with no count needs a value that gives one:

```zaka-error ZK0421
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း ဖြစ်ပါစေ။
```

```text
error[ZK0421]: this list kind has no cell count, and nothing here gives one
```

## Cells of a value that is no list

```zaka-error ZK0422
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၇၀ ဖြစ်ပါစေ။
အမှတ် ၏ အကွက် ၀ ကို ပြသပါ။
```

```text
error[ZK0422]: isize is not a list, so it has no cells
```

## The cell count, written as two words

`အကွက်အရေအတွက်` is one word. Written with a space, `အကွက် အရေအတွက်` reads as the cell
numbered with a variable named `အရေအတွက်`.

## Hands on

- Read cell 5 of a list of 5 cells through a variable, and read the message.
- Read the same cell with a constant number, and compare the two messages.

Next: [Natural Zaka](../natural-zaka/README.md).
