---
title: Compile time
description: Functions the compiler runs while it compiles - their mark, the spots where a call runs while the compiler runs, what such a run can do, and conditions that call them.
---

# Compile time

Some values never change from run to run: the size of a table of squares, a greeting
built from a name, the cells of a lookup list. A *comptime function* works such a value
out while the compiler runs, so the program gets it as if it were written out in full.

## A comptime function

Put `ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော` ("that can run at compile time") in
front of `လုပ်ဆောင်ချက်`. The rest of the function is as before:

```zaka
ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် နှစ်ဆ သည်
n ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
n * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းသေ အရွယ် သည် နှစ်ဆ ဟူသော လုပ်ဆောင်ချက်ကို ၃ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဂ သည် အကွက် အရွယ် ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ဂ ကို ပြသမည်။
နှစ်ဆ ဟူသော လုပ်ဆောင်ချက်ကို ၁၀ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၀၊ ၀၊ ၀၊ ၀၊ ၀၊ ၀]
၂၀
```

The constant `အရွယ်` gets 6 before the program runs, so it can count the cells of a
list. The last call has no such need: it runs with the program, as each call does.

## Where a call runs at compile time

A call of a comptime function runs while the compiler runs when it stands in:

- the value of a constant, `ကိန်းသေ`;
- the count of a counted list;
- the starting value of a variable or a constant of an imported file;
- the value of a member of an enumeration, or a case of a switch;
- the condition of a [compile-time conditional](c.md#each-os-its-own-lib).

Each argument of such a call must be known before the program runs: literals,
constants, operators on them, and other comptime calls.

## Text, lists and structs

A comptime function can give text, a list or a struct, and build it with loops,
conditionals and calls, recursion among them:

```zaka
ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် ဆ သည်
n ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ n <= ၁ ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
n * ဆ ဟူသော လုပ်ဆောင်ချက်ကို n - ၁ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် ဇယားလုပ် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် စာရင်း တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် ဇ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် စာရင်းအလွတ် ဖြစ်ပါစေ။
၁ မှ ၅ အထိ i အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဇ ထဲသို့ ဆ ဟူသော လုပ်ဆောင်ချက်ကို i ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ထည့်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ဇ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းသေ ဆများ သည် ဇယားလုပ် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းသေ ခေါင်းစဉ် သည် "{ဆ ဟူသော လုပ်ဆောင်ချက်ကို ၅ ဖြင့် လုပ်ဆောင်သော ရလဒ်} ခု" ဖြစ်ပါစေ။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဆများ ကို ပြသမည်။
ခေါင်းစဉ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၂၊ ၆၊ ၂၄၊ ၁၂၀]
၁၂၀ ခု
```

The result must be a value a constant can hold: a number, a character, true or false, a
member of an enumeration, text, or a list or a struct of these.

## What a run can do

A run at compile time can call each function of the program, comptime or not. It holds
no right to the files, the environment or C unless the build grants one, as
`zakac run --comptime-allow-read f.zaka` grants reads of files: an effect with no right
stops the build. What it prints shows as a note at the call. A run that stops, as a
division with a zero divisor does, stops the build with it:

```zaka-error ZK0611
ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် ခွဲ သည်
n ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
၁၀ / n ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းသေ အပိုင်း သည် ခွဲ ဟူသော လုပ်ဆောင်ချက်ကို ၀ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အပိုင်း ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

The error names the call and the reason, and the help lists the calls that led there.

## Conditions that call comptime functions

A compile-time conditional can call a comptime function in its condition. The compiler
runs the call, then keeps the arm the value picks:

```zaka
ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် ကျယ် သည်
n ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
n > ၁၀ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် ကျယ် ဟူသော လုပ်ဆောင်ချက်ကို ၂၀ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ အမည် သည် "ကျယ်" ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
သို့မဟုတ်ပါက အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ အမည် သည် "ကျဉ်း" ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အမည် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ကျယ်
```

## Hands on

- Write a comptime function that gives the sum of 1 to n, and use it to count the cells
  of a list.
- Give `ခွဲ` a 2 in place of the 0 above, and print the constant.

Next: [Text](../text/README.md).
