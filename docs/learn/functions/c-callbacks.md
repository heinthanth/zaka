---
title: Callbacks and handles
description: Pass a function of the program to C so that C can call it later, and work with a struct that a C lib keeps.
---

# Callbacks and handles

We can pass a Zaka function to C so that C can call it later. This is a *callback*. A C
lib can also give us a pointer to a struct it manages, called a *handle* (opaque
pointer). This page builds on [Calling C](c.md) with an example of each.

## You will learn

- how to pass a function of the program to C as a callback
- how to give C leave to call a closure later, and take the leave back
- how to declare a struct that a C lib keeps, read its fields through a handle, and close it

## A function C keeps

```text
<name> ဟူသော လုပ်ဆောင်ချက်ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်။
<name> ဟူသော လုပ်ဆောင်ချက်ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြန်လည်ရုပ်သိမ်းမည်။
```

A function of the program goes to a C function as a function value, and this works for a
[closure](../function-values/closures.md), a function that uses variables around it, too.
Some C functions keep the function and call it later: `signal` keeps a handler, and `raise`
makes the signal come. When C calls a closure after the call that passed it has ended, the
closure needs *leave* from the program. We give that leave with the first line of the
pattern, which means "allow C to call it", and we take it back with the second, which means
"take back the leave for C":

```zaka
signal ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
sig ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
handler ဟူသော (၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို လည်းကောင်း
လက်ခံမည်။
raise ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
sig ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖမ်းကြိမ် သည် ၀ ဖြစ်ပါစေ။
ဖမ်း ဟူသော လုပ်ဆောင်ချက် သည်
s ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဖမ်းကြိမ် ကို ဖမ်းကြိမ် + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ဖမ်း ဟူသော လုပ်ဆောင်ချက်ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
signal ဟူသော လုပ်ဆောင်ချက်ကို ၁၅ နှင့် ဖမ်း ဟူသော လုပ်ဆောင်ချက် တို့ဖြင့် လုပ်ဆောင်မည်။
raise ဟူသော လုပ်ဆောင်ချက်ကို ၁၅ ဖြင့် လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ဖမ်းကြိမ် ကို ပြသမည်။
ဖမ်း ဟူသော လုပ်ဆောင်ချက်ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြန်လည်ရုပ်သိမ်းမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁
```

| Zaka | Reads as |
| --- | --- |
| `ပြင်ပမှ` | "from outside" |
| `ခေါ်ဆိုခွင့်` | "leave to call" |
| `ပြုမည်` | "will grant" |
| `ပြန်လည်ရုပ်သိမ်းမည်` | "will take back" |

The parameter `handler` has a [function kind](../function-values/README.md) (function
type), which we write in parentheses inside a parameter. A function that uses no variables
around it needs no leave.

## What a callback can accept

C can call back with more than numbers. A parameter of the callback can be a plain struct,
which C passes as a whole value, text, which Zaka duplicates from the `char *` that C
passes, or a handle, which the next section explains. When C can pass NULL, we write the
parameter as an optional, such as `စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`, and test it before
we read it, because a NULL for plain text stops the program. A handle that C passed
belongs to C, so the callback cannot close it outside of unsafe code.

## A struct C keeps

```text
<name> ဟူသော ဒေတာအစုအဝေးသည် ပြင်ပမှ လာသော ဒေတာအစုအဝေး ဖြစ်၍ ၎င်းတွင် <fields> တို့ပါဝင်သည်။
```

Some libs keep a struct of their own and give the program a pointer to it, such as a
counter or a database connection. To use one, we declare a struct that comes from outside
the program, and a value of that struct is a handle. In the declaration, we write
`ပြင်ပမှ လာသော ဒေတာအစုအဝေး` ("a struct that comes from outside"), then `၎င်းတွင်` ("in
it") and the C fields that the program wants to read, in order from the start of the C
struct:

```zaka-file
// counter.zaka
ရေတွက်စက် ဟူသော ဒေတာအစုအဝေးသည် ပြင်ပမှ လာသော ဒေတာအစုအဝေး ဖြစ်၍ ၎င်းတွင်
count ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု၊ step ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု နှင့် label ဟူသော စာသား တစ်ခု တို့ပါဝင်သည်။

counter_open ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libcounter" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
step ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ရေတွက်စက် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
counter_next ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libcounter" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
c ဟူသော ရေတွက်စက် တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
counter_close ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libcounter" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
c ဟူသော ရေတွက်စက် တစ်ခု ကို အပိုင်အနေဖြင့် လက်ခံမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် စက် သည် counter_open ဟူသော လုပ်ဆောင်ချက်ကို ၅ ဖြင့် လုပ်ဆောင်သော ရလဒ် ထဲမှ ဒေတာ ဖြစ်ပါစေ။
counter_next ဟူသော လုပ်ဆောင်ချက်ကို စက် ဖြင့် လုပ်ဆောင်မည်။
"{စက် ၏ count} {စက် ၏ step}" ဟု ပြသမည်။
စက် ၏ label ကို ပြသမည်။
counter_close ဟူသော လုပ်ဆောင်ချက်ကို စက် ဖြင့် လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

The program opens a counter, steps it once, reads two fields and its label, and closes it:

- `counter_open` returns a handle, or `အလွတ်` when C returns NULL, so its result is an
  optional of the handle. A plain `ရေတွက်စက်` result would promise that C never returns
  NULL, and a NULL would then stop the program. The program never has the address inside
  the handle.
- `စက် ၏ count` reads the field from C's storage each time, so it shows the value that C
  has now.
- `စက် ၏ label` is an optional of text, because a `char *` can be NULL.
- Safe code cannot write a field, so `စက် ၏ count ကို ၀ ဟု ထားမည်` is an error there
  (ZK0474), and a C function changes the struct instead.
- A program cannot make a handle, duplicate the struct behind it, or print it (ZK0473).

| Zaka | Reads as |
| --- | --- |
| `ပြင်ပမှ လာသော ဒေတာအစုအဝေး` | "a struct that comes from outside" |
| `၎င်းတွင်` | "in it" |
| `အပိုင်အနေဖြင့်` | "as its own" |

## Closing a handle

Closing the handle is a task for the program. `counter_close` marks its parameter with
`အပိုင်အနေဖြင့်` ("as its own"), which means that the C function takes the handle over.
After the call, each variable that shares the handle counts as closed, and a call or a
field read through one of them stops the program with the [closed handle
trap](../../manual/vm/traps-unsafe.md#c-functions). So we close a handle once and leave it alone
after that.

## Details

- A closure that C calls after the call that passed it has ended, with no leave, stops the
  program with the closure escape trap ([When a program stops](../../manual/vm/traps.md)).
  [C functions and addresses](../pointers/c.md#a-closure-for-a-call) shows a closure that
  C calls while the call that passed it is still running, which needs no leave.
- The full rules for closures that C calls are in [Closures and
  C](../../manual/language/closures.md#closures-and-c).
- Each data kind that a callback can accept, and each rule for a struct from outside the
  program, is in [C functions](../../manual/language/functions-c.md#callbacks).

## Recap

- A function of the program goes to C as a function value, and C can call it back.
- A closure that C calls later needs leave: `ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်` grants it and
  `ပြင်ပမှ ခေါ်ဆိုခွင့် ပြန်လည်ရုပ်သိမ်းမည်` takes it back.
- `ပြင်ပမှ လာသော ဒေတာအစုအဝေး` declares a struct that a C lib keeps, and a value of it is a
  handle, whose fields safe code can read but not write.
- A C function whose parameter has `အပိုင်အနေဖြင့်` closes the handle, and a closed handle
  stops the program when it is used.

## Hands on

- Change the signal example so that `raise` runs twice, and print the count.
- Leave out the line that grants leave, and see what happens.
- Write a safe wrapper that opens a counter, steps it three times and closes it.

Next: [A lib for C](c-lib.md).
