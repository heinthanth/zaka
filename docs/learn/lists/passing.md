---
title: Passing and comparing lists
description: Lists are values, so each assignment and argument duplicates the cells; lend a list through its address.
---

# Passing and comparing lists

A list is a value, like an integer. Setting one list to another duplicates each cell,
and the two lists stand apart afterwards:

```zaka
ကိန်းရှင် မူရင်း သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် ပွား သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
ပွား ကို မူရင်း ဟု ထားပါ။              // each cell is duplicated
ပွား ၏ အကွက် ၀ ကို ၁၀၀ ဟု ထားပါ။
မူရင်း ကို ပြသပါ။
ပွား ကို ပြသပါ။
```

```text
[၁၊ ၂၊ ၃]
[၁၀၀၊ ၂၊ ၃]
```

## Lists as parameters

A list parameter gets a duplicate of the list passed in, just as an integer parameter
does. A change inside the function remains inside. A plain list parameter takes lists of
each count:

```zaka
// the parameter is a duplicate of the list passed in
လုပ်ဆောင်ချက် သုညထား သည်
ကိန်းပြည့် စာရင်း ကိန်းများ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းများ ၏ အကွက် ၀ ကို ၀ ဟု ထားပါ။
ကိန်းများ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် မူရင်း သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် သုညထား ကို မူရင်း တို့ဖြင့် လုပ်ဆောင်ပါ။
မူရင်း ကို ပြသပါ။     // unchanged
လုပ်ဆောင်ချက် သုညထား ကို [၄၊ ၅၊ ၆၊ ၇] တို့ဖြင့် လုပ်ဆောင်ပါ။
```

```text
[၀၊ ၂၊ ၃]
[၁၊ ၂၊ ၃]
[၀၊ ၅၊ ၆၊ ၇]
```

The duplicate costs little: the parameter reads the caller's cells until its first
write, and that write gives it cells of its own.

To let a function change the caller's list, pass its address, as with [the caller's
variables](../pointers/out-parameters.md). The parameter kind is the list kind with
`လိပ်စာ` after it and `ငှားထားသော` in front, since the function borrows the caller's
list. The cells are reached through it with `အကွက်` as before:

```zaka
// the parameter is the address of the caller's list
လုပ်ဆောင်ချက် နှစ်ဆတိုး သည်
ငှားထားသော ကိန်းပြည့် စာရင်း လိပ်စာ ကိန်းများ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၀ မှ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၁ အထိ န အနေဖြင့် ရေတွက်၍ ကိန်းများ ၏ အကွက် န ကို (ကိန်းများ ၏ အကွက် န) * ၂ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် မူရင်း သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် နှစ်ဆတိုး ကို မူရင်း ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်ပါ။
မူရင်း ကို ပြသပါ။     // changed through the address
```

```text
[၂၊ ၄၊ ၆]
```

A fixed list is passed in the same manner, through `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း`
or the address of one; its count is part of the kind, so the function takes lists of
that count alone.

## Lists as results

A function can also give back a list, plain or fixed, and the function fills `ရလဒ်` or
returns a list. Here the result is a fixed list of four cells, which start at `၀`:

```zaka
// a function that builds a list and gives it back
လုပ်ဆောင်ချက် နှစ်ထပ်များ သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အကွက် ၄ ခုရှိသော ကိန်းပြည့် စာရင်း ရလဒ် ကို ပြန်ပေးမည်။
၀ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ ရလဒ် ၏ အကွက် န ကို န * န ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် နှစ်ထပ်များ ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းများ ကို ပြသပါ။
```

```text
[၀၊ ၁၊ ၄၊ ၉]
```

The caller gets a list of its own, as with an assignment: here a plain list, which takes
the cells of the fixed one. See [Lists and structs as
results](../../manual/language/functions.md#lists-and-structs-as-results) for the full rules.

## List literals as arguments

A list literal takes each value before its `တို့ပါသော စာရင်း`, back to the last `၊` that
starts the argument list. Put it in parentheses when another argument comes before it:

```zaka
// a list literal after another argument goes in parentheses
လုပ်ဆောင်ချက် ပေါင်း သည်
ကိန်းပြည့် အစ နှင့် ကိန်းပြည့် စာရင်း ကိန်းများ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
ရလဒ် ကို အစ ဟု ထားပါ။
၀ မှ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၁ အထိ န အနေဖြင့် ရေတွက်၍ ရလဒ် ကို ရလဒ် + (ကိန်းများ ၏ အကွက် န) ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

လုပ်ဆောင်ချက် ပေါင်း ကို ၁၀၀၊ (၁၊ ၂၊ ၃ တို့ပါသော စာရင်း) တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
```

```text
၁၀၆
```

## Comparing lists

`==` and `!=`, and the verbs `ညီ` and `မညီ`, compare two lists of the same kind, one cell
at a time:

```zaka
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။
က သည် ခ နှင့် ညီသည် ဟု ပြသပါ။
ခ ၏ အကွက် ၂ ကို ၄ ဟု ထားပါ။
က == ခ ကို ပြသပါ။
က != စာရင်းအလွတ် ကို ပြသပါ။
```

```text
မှန်
မမှန်
မှန်
```

Two plain lists compare at each count: lists of different counts are not equal. A plain
and a heap list of one cell kind compare too. Lists of different kinds do not compare,
such as `isize[3]` and `isize[2]`, or a fixed and a plain list. Lists have no order, so
`<` and `>` do not take them.

## Hands on

- Write a function that takes the address of a list of integers and adds 1 to each cell.
- Duplicate a list, change the duplicate, and compare the two.

To take a part of a list, or a fixed list with no duplicate, take a [slice](slices.md).

Next: [Adding and removing cells](growing.md).
