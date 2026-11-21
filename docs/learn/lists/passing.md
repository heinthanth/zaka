---
title: Passing and comparing lists
description: A list is a value, so each assignment and each argument makes a duplicate, and a function can return a list as its result.
---

# Passing and comparing lists

A list (dynamic array) is a value, the same as an integer, so when we assign a list to
another variable or pass it to a function, we get a duplicate (copy), and the two lists
are separate from then on. This page shows lists as parameters and results, and how to
compare two lists.

## You will learn

- what happens when we assign a list to another variable
- how to pass a list to a function, and how to let the function change it
- how to return a list from a function
- how to compare two lists

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မူရင်း သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂ နှင့် ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
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

After `ပွား ကို မူရင်း ဟု ထားမည်။`, the list `ပွား` (the duplicate) has the same cells as
`မူရင်း` (the original), and when we change a cell of `ပွား`, `မူရင်း` does not change.
The duplicate does not cost much, because the two lists share their cells until one of
them writes, and that write copies the cells first.

## Lists as parameters

```text
<name> ဟူသော <kind> စာရင်း တစ်ခု ကို လက်ခံပြီး
<name> ဟူသော <kind> စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
```

A list parameter gets a duplicate of the list we pass in, just as an integer parameter
does, so a change inside the function remains inside it. A parameter of a plain list
accepts a list with a different number of cells on each call:

```zaka
// the parameter is a duplicate of the list passed in
သုညထား ဟူသော လုပ်ဆောင်ချက် သည်
ကိန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းများ ၏ အကွက် ၀ ကို ၀ ဟု ထားမည်။
ကိန်းများ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မူရင်း သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂ နှင့် ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
သုညထား ဟူသော လုပ်ဆောင်ချက်ကို မူရင်း ဖြင့် လုပ်ဆောင်မည်။
မူရင်း ကို ပြသမည်။     // unchanged
သုညထား ဟူသော လုပ်ဆောင်ချက်ကို [၄၊ ၅၊ ၆၊ ၇] ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၀၊ ၂၊ ၃]
[၁၊ ၂၊ ၃]
[၀၊ ၅၊ ၆၊ ၇]
```

To let a function change the caller's list, we write `ဒေတာ ပြောင်းလဲရန်` in the parameter,
which makes it a mutable parameter (`inout`), as for [the caller's
variable](../functions/parameters.md#changing-the-callers-variable). Inside the function,
we reach the cells with `အကွက်` as before:

```zaka
// the function can change the caller's list
နှစ်ဆတိုး ဟူသော လုပ်ဆောင်ချက် သည်
ကိန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ (ကိန်းများ ၏ အကွက်အရေအတွက်) == ၀ ဖြစ်လျှင် ပြန်ပေးမည်။
၀ မှ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၁ အထိ န အနေဖြင့် ရေတွက်၍ ကိန်းများ ၏ အကွက် န ကို (ကိန်းများ ၏ အကွက် န) * ၂ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မူရင်း သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂ နှင့် ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
နှစ်ဆတိုး ဟူသော လုပ်ဆောင်ချက်ကို မူရင်း ဖြင့် လုပ်ဆောင်မည်။
မူရင်း ကို ပြသမည်။     // the function changed it
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၂၊ ၄၊ ၆]
```

The function checks first whether the list has no cells, because the last cell number,
`အကွက်အရေအတွက် - ၁`, cannot go below 0 ([A list with no cells and the last
number](mistakes.md#a-list-with-no-cells-and-the-last-number)).

## Lists as results

```text
အောက်ပါတို့ကို လုပ်ဆောင်ကာ <kind> စာရင်း တစ်ခု ကို ပြန်ပေးမည်။
```

A function can also return a list. In this example, the function builds the list in the
variable `အဖြေ` (answer), which starts with no cells, `[]`, and the loop adds one cell for
each square:

```zaka
// a function that builds a list and returns it
နှစ်ထပ်များ ဟူသော လုပ်ဆောင်ချက် သည်
အရေ ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် စာရင်း တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [] ဖြစ်ပါစေ။
၀ မှ အရေ - ၁ အထိ န အနေဖြင့် ရေတွက်၍ အဖြေ ထဲသို့ န * န ကို ထည့်မည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် နှစ်ထပ်များ ဟူသော လုပ်ဆောင်ချက်ကို ၄ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းများ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၀၊ ၁၊ ၄၊ ၉]
```

The caller gets a list that is separate from all others, the same as with an assignment.

## List literals as arguments

A list literal in words is a single argument, but it joins its cells with `နှင့်` and
`တို့`, the same words that join the arguments of a call. So when another argument comes
before it, we put the list literal in parentheses:

```zaka
// a list literal after another argument goes in parentheses
ပေါင်း ဟူသော လုပ်ဆောင်ချက် သည်
အစ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ကိန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
အဖြေ ကို အစ ဟု ထားမည်။
ကိန်းများ ၏ အကွက်တိုင်း ကို ကိန်း အနေဖြင့် ယူ၍ အဖြေ ကို အဖြေ + ကိန်း ဟု ထားမည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို ၁၀၀ နှင့် (၁၊ ၂ နှင့် ၃ တို့ပါသော စာရင်း) တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၀၆
```

A bracket literal such as `[၁၊ ၂၊ ၃]` needs no parentheses.

## Comparing lists

```text
<list> == <list>
<list> != <list>
<list> သည် <list> နှင့် ညီသည်
```

We can use `==` and `!=`, or the verbs `ညီ` ("equal") and `မညီ` ("not equal"), to compare
two lists of the same data kind (type), which checks them one cell at a time:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂ နှင့် ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
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

Two lists with different cell counts are never equal.

## Details

- Lists of different data kinds, such as `isize[]` and `f64[]`, cannot be compared at all
  ([Passing and comparing](../../manual/language/lists.md#passing-and-comparing)).
- Lists have no order, so `<` and `>` do not work on them.
- For programmers who know other languages: a Zaka list behaves like an array in PHP, and
  not like a list in Java or JavaScript, where an assignment shares the one list. When we
  want that sharing in Zaka, we use a [shared object](../structs/objects.md) (reference
  type).
- Part III passes the address of a list instead of a duplicate ([Lists and structs through
  addresses](../pointers/structs.md)), and Part II shows how to get a run of cells from a list
  ([Ranges](../forms/ranges.md)).
- The full rules for results are in [Lists and structs as
  results](../../manual/language/functions-results.md#lists-and-structs-as-results).

## Recap

- Assigning a list or passing it to a function makes a duplicate, and the duplicate copies
  the cells at its first write.
- A parameter with `ဒေတာ ပြောင်းလဲရန်` lets the function change the caller's list.
- A function can return a list with `ပြန်ပေးမည်`, and the caller gets a separate list.
- A list literal in words goes in parentheses after another argument.
- `==` and `!=` compare two lists one cell at a time.

## Hands on

- Write a function that can change a list of integers, and adds 1 to each cell.
- Duplicate a list, change the duplicate, and compare the two.
- Write a function that returns the cells of a list in reverse order.

Next: [Lists of lists](grids.md).
