---
title: Compile time
description: Functions that the compiler can run while it compiles, where such a call runs, what the run can do, and conditions that call them.
---

# Compile time

Some values never change from one run to the next, such as the size of a table of squares,
a greeting built from a name, or the cells of a lookup list. A *comptime function*
(compile-time function, `constexpr` in C++) works out such a value while the compiler runs,
so the program gets the value as if we had written it out in full.

## You will learn

- how to mark a function so that the compiler can run it
- where a call runs at compile time, and where it runs with the program
- what a run at compile time can do, and what happens when it stops
- how a compile-time condition can call a comptime function

```zaka
နှစ်ဆ ဟူသော ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် သည်
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

To make a comptime function, we write `ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော` between
`ဟူသော` and `လုပ်ဆောင်ချက်`, and the rest of the function is the same as before:

```text
<name> ဟူသော ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် သည်
```

| Zaka | Reads as |
| --- | --- |
| `ကွန်ပိုင်းလုပ်ချိန်တွင်` | "at compile time" |
| `လုပ်ဆောင်နိုင်သော` | "that can run" |

The constant `အရွယ်` gets the value 6 before the program runs, so it can set the number of
cells of a counted list (fixed-size array). The last call has no such need, so it runs with
the program, the same as a call to a plain function.

## Where a call runs at compile time

A call to a comptime function runs while the compiler runs when it is in one of these
places:

- the value of a constant, `ကိန်းသေ`
- the cell count of a counted list
- the first value of a variable or a constant in an imported file
- the value of a member of an enumeration, or a case of a switch
- the condition of a compile-time conditional, which the [last section](#conditions-that-call-comptime-functions) shows

In those places, each argument of the call must be known before the program runs, so it can
be a literal, a constant, an operator on these, or another comptime call.

## Text, lists and structs

A comptime function can return text, a list or a struct, and it can build the value with
loops, conditionals and calls, including recursive calls:

```zaka
ဆ ဟူသော ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် သည်
n ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ n <= ၁ ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
n * ဆ ဟူသော လုပ်ဆောင်ချက်ကို n - ၁ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ဇယားလုပ် ဟူသော ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် သည်
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

The result must be a value that a constant can have: a number, a character, true or false,
a member of an enumeration, text, or a list or a struct of these.

## What a run can do

A run at compile time can call each function of the program, whether it is a comptime
function or not. It has no right to read files, read the environment or call C unless the
build grants one, as `zakac run --comptime-allow-read f.zaka` grants reads of files, and an
effect with no right stops the build. What the run prints shows up as a note at the call.
A run that stops, as a division with a zero divisor does, stops the build too:

```zaka-error ZK0611
ခွဲ ဟူသော ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် သည်
n ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
၁၀ / n ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းသေ အပိုင်း သည် ခွဲ ဟူသော လုပ်ဆောင်ချက်ကို ၀ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အပိုင်း ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

The error names the call and the reason, and its help lists the calls that led there.

## Conditions that call comptime functions

```text
အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် <condition> ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
<declarations>
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
သို့မဟုတ်ပါက အောက်ပါတို့ကို လုပ်ဆောင်မည်။
<declarations>
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

A *compile-time conditional* is an `if` that the compiler settles before the program runs:
we write `ကွန်ပိုင်းလုပ်ချိန်တွင်` ("at compile time") after `အကယ်၍`, and the compiler keeps
the declarations of the arm that the condition picks and drops the other arm. The condition
can call a comptime function, and the compiler runs that call first:

```zaka
ကျယ် ဟူသော ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် သည်
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

A compile-time conditional can also test facts of the computer that the program is built
for, such as its OS, and [Calling C](c.md#each-os-its-own-lib) uses that to declare
different C functions on each OS.

## Details

- We can put the mark on a function with statements at the top level of a file. On the
  program start block, a C function or a nested function, it is the error ZK0256
  ([Comptime functions](../../manual/language/compile-time.md#comptime-functions)).
- A comptime function with no result, or whose result is a pointer, a table, a function
  value, an optional or a result kind, is ZK0489.
- A variable as an argument, in a place that needs the value at compile time, is ZK0608
  ([Where a call runs at compile
  time](../../manual/language/compile-time.md#where-a-call-runs-at-compile-time)), and a
  call that needs a value that waits on the call itself is ZK0609.
- The flags `--comptime-allow-read`, `--comptime-allow-write`, `--comptime-allow-env`,
  `--comptime-allow-ffi` and `--comptime-allow-all` grant the rights of a run. Reading
  input stops the run, and a run has a cap on its operations, its heap and its depth of calls
  ([What a run can do](../../manual/language/compile-time.md#what-a-run-can-do)).
- In the language server and the explorer, a run can read files under the folder of the
  program at most, and it cannot write files or call C.

## Recap

- `ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော` before `လုပ်ဆောင်ချက်` makes a comptime function.
- A call to it runs at compile time in a constant, a list count, an enumeration value, a
  switch case or a compile-time condition, and with the program in all other places.
- Its arguments there must be known before the program runs, and its result must be a
  value that a constant can have.
- A run that stops, or that tries an effect with no right, stops the build.

## Hands on

- Write a comptime function that returns the sum of 1 to n, and use it to set the cell
  count of a counted list.
- Give `ခွဲ` a 2 in place of the 0 above, and print the constant.
- Pass a variable to `နှစ်ဆ` in the value of a constant, and read the error.

Next: [Text](../text/README.md).
