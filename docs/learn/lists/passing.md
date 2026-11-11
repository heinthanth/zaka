---
title: Passing and comparing lists
description: Lists are values, so each assignment and argument gives a duplicate, and a function gives a list back as its result.
---

# Passing and comparing lists

A list is a value, like an integer, as a list is in PHP and unlike a list in Java or
JavaScript. Setting one list to another gives a duplicate, and the two lists stand apart
afterwards. The duplicate is cheap: the two share their cells until one of them writes, and
that write copies the cells first.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မူရင်း သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် ပွား သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
ပွား ကို မူရင်း ဟု ထားမည်။              // shares the cells, for now
ပွား ၏ အကွက် ၀ ကို ၁၀၀ ဟု ထားမည်။        // this write copies them first
မူရင်း ကို ပြသမည်။
ပွား ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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
ကိန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းများ ၏ အကွက် ၀ ကို ၀ ဟု ထားမည်။
ကိန်းများ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မူရင်း သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် သုညထား ကို မူရင်း တို့ဖြင့် လုပ်ဆောင်မည်။
မူရင်း ကို ပြသမည်။     // unchanged
လုပ်ဆောင်ချက် သုညထား ကို [၄၊ ၅၊ ၆၊ ၇] တို့ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၀၊ ၂၊ ၃]
[၁၊ ၂၊ ၃]
[၀၊ ၅၊ ၆၊ ၇]
```

To let a function change the caller's list, write the parameter with `ဒေတာ ပြောင်းလဲရန်`, as
for [the caller's variable](../functions/parameters.md#changing-the-callers-variable). The cells are
reached through it with `အကွက်` as before:

```zaka
// the function can change the caller's list
လုပ်ဆောင်ချက် နှစ်ဆတိုး သည်
ကိန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၀ မှ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၁ အထိ န အနေဖြင့် ရေတွက်၍ ကိန်းများ ၏ အကွက် န ကို (ကိန်းများ ၏ အကွက် န) * ၂ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မူရင်း သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် နှစ်ဆတိုး ကို မူရင်း တို့ဖြင့် လုပ်ဆောင်မည်။
မူရင်း ကို ပြသမည်။     // the function changed it
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၂၊ ၄၊ ၆]
```

Part III passes the address of the list instead: see [Lists and structs through
addresses](../pointers/structs.md).

## Lists as results

A function can also give back a list: it builds one in a variable and returns it. Here
`အဖြေ` starts with no cells, `[]`, and the loop adds one for each square:

```zaka
// a function that builds a list and gives it back
လုပ်ဆောင်ချက် နှစ်ထပ်များ သည်
အရေ ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် စာရင်း တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [] ဖြစ်ပါစေ။
၀ မှ အရေ - ၁ အထိ န အနေဖြင့် ရေတွက်၍ အဖြေ ထဲသို့ န * န ကို ထည့်မည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် နှစ်ထပ်များ ကို ၄ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းများ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၀၊ ၁၊ ၄၊ ၉]
```

The caller gets a list of its own, as with an assignment. See [Lists and structs as
results](../../manual/language/functions.md#lists-and-structs-as-results) for the full rules.

## List literals as arguments

A list literal takes each value before its `တို့ပါသော စာရင်း`, back to the last `၊` that
starts the argument list. Put it in parentheses when another argument comes before it:

```zaka
// a list literal after another argument goes in parentheses
လုပ်ဆောင်ချက် ပေါင်း သည်
အစ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ကိန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
အဖြေ ကို အစ ဟု ထားမည်။
၀ မှ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၁ အထိ န အနေဖြင့် ရေတွက်၍ အဖြေ ကို အဖြေ + (ကိန်းများ ၏ အကွက် န) ဟု ထားမည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ပေါင်း ကို ၁၀၀၊ (၁၊ ၂၊ ၃ တို့ပါသော စာရင်း) တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၀၆
```

## Comparing lists

`==` and `!=`, and the verbs `ညီ` and `မညီ`, compare two lists of the same kind, one cell
at a time:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။
က သည် ခ နှင့် ညီသည် ဟု ပြသမည်။
ခ ၏ အကွက် ၂ ကို ၄ ဟု ထားမည်။
က == ခ ကို ပြသမည်။
က != စာရင်းအလွတ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
မမှန်
မှန်
```

Two lists compare at each count: lists of different counts are not equal. Lists of
different kinds do not compare, such as `isize[]` and `f64[]`. Lists have no order, so
`<` and `>` do not take them.

## Hands on

- Write a function that can change a list of integers, and adds 1 to each cell.
- Duplicate a list, change the duplicate, and compare the two.

Part II takes a run of cells from a list: see [Ranges](../forms/ranges.md).

Next: [Lists of lists](grids.md).
