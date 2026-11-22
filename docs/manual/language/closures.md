---
title: Closures
description: Nested functions, their captured variables, closures as function values, closures made in a loop, closures given to C, and how a closure runs.
---

# Closures

We can declare a function inside the statements of another function, and such a *nested
function* (local function) can use the local variables of the code around it. The
variables it uses are its *captures* (captured variables), and the function value of a
nested function with captures is a *closure*, which keeps its captures alive
after the block that declares it ends. This page states the rules in full, and the Learn
book teaches closures in [Closures](../../learn/function-values/closures.md). Function
kinds and the function values of top-level functions are on [Function
values](function-values.md).

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စုစုပေါင်း သည် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် မှတ်တမ်း သည် "" ဖြစ်ပါစေ။
// uses စုစုပေါင်း and မှတ်တမ်း of the program start block
ထည့် ဟူသော လုပ်ဆောင်ချက် သည် န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စုစုပေါင်း ကို စုစုပေါင်း + န ဟု ထားမည်။
မှတ်တမ်း ထဲသို့ "+{န}" ကို ထည့်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ထည့် ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်မည်။
စုစုပေါင်း ကို ၁၀ ဟု ထားမည်။
ထည့် ဟူသော လုပ်ဆောင်ချက်ကို ၂ ဖြင့် လုပ်ဆောင်မည်။
"{စုစုပေါင်း} {မှတ်တမ်း}" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၂ +၁+၂
```

## Nested functions

We can declare a function with statements inside the statements of a function, the program
start block, a block, a loop or a conditional. Such a nested function is visible from its
declaration to the end of the block that contains it, and inside itself, so it can call
itself.

| Case | Rule |
| --- | --- |
| two nested functions of one name in one scope | the error ZK0304 |
| two nested functions that share a name, as overloads | not allowed: nested functions have no overloads |
| a nested function with the name of a top-level function | the nested function hides the top-level one |
| an external function, a C function or the program start block inside a function | the error ZK0214: these remain at the top level |

## Captures

A nested function uses the local variables and the parameters of the functions around it
through their names, as a function in JavaScript or Python does, and each one it uses is a
capture. We do not list the captures: the compiler finds them in the statements. A capture
is the outer variable itself, shared between the function and the block around it, so a
write on either side shows on the other, as the first example shows.

- Globals and top-level functions are not captures, because each function reaches them.
- A function nested in a nested function reaches the locals of each function around it.
  The function between the two captures those locals too, because it makes the inner
  function.
- A nested function that calls another nested function with captures, or names it with
  `ဟူသော`, captures that closure just as it captures a variable.
- A nested function with no captures is a plain function value.
- When a capture is a mutable parameter (`ဒေတာ ပြောင်းလဲရန်`) of the function around it,
  the nested function shares that parameter, and the value of the parameter when that
  function returns goes back to its caller, as in [Mutable
  parameters](functions-parameters.md#mutable-parameters).

## Closures as values

```text
<name> ဟူသော လုပ်ဆောင်ချက်
```

When a nested function has captures, `<name> ဟူသော လုပ်ဆောင်ချက်` is a closure: a value of
the same function kind (function type) as a plain function, so each function kind accepts
closures and no new form is needed. The captures never show in the function kind. A call
of the function through its name goes through the one closure that is made where the
function is declared. A closure prints as `<name> ဟူသော လုပ်ဆောင်ချက်`, as other function
values do.

A closure keeps its captures alive, so we can use it wherever other values go: we can
return it, store it in a field, a cell or a global, and call it after the block that
declares it ends. Each holder shares the one closure and its captures:

```zaka
ရေတွက်စက် ဟူသော လုပ်ဆောင်ချက် သည်
စ ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ (လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် ကြိမ် သည် စ ဖြစ်ပါစေ။
တိုး ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကြိမ် ကို ကြိမ် + ၁ ဟု ထားမည်။
ကြိမ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
// ကြိမ် outlives the call, inside the closure
တိုး ဟူသော လုပ်ဆောင်ချက် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

နှစ်ကြိမ် ဟူသော လုပ်ဆောင်ချက် သည်
ဖ ဟူသော (လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဖ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
ဖ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရေတွက် သည် ရေတွက်စက် ဟူသော လုပ်ဆောင်ချက်ကို ၁၀ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် အခြား သည် ရေတွက် ဖြစ်ပါစေ။
ရေတွက် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
// the same counter, through another name and a parameter
နှစ်ကြိမ် ဟူသော လုပ်ဆောင်ချက်ကို အခြား ဖြင့် လုပ်ဆောင်မည်။
ကိန်းရှင် နောက် သည် ရေတွက်စက် ဟူသော လုပ်ဆောင်ချက်ကို ၁၀ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
// a second call makes a second counter
နောက် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
ရေတွက် == အခြား ကို ပြသမည်။
ရေတွက် == နောက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၁
၁၂
၁၃
၁၁
မှန်
မမှန်
```

This program keeps a closure in a field and in a global, and nests a closure two levels
deep:

```zaka
ခလုတ် ဟူသော ဒေတာအစုအဝေးတွင်
အမည် ဟူသော စာသား တစ်ခု နှင့်
နှိပ် ဟူသော (လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု
တို့ပါဝင်သည်။

ဘာမျှမလုပ် ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// a global that keeps a function value, with a function from the start
ကိန်းရှင် သိမ်း သည် (လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ဖြစ်၍ တန်ဖိုးသည်
ဘာမျှမလုပ် ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။

ခလုတ်သစ် ဟူသော လုပ်ဆောင်ချက် သည်
အမည် ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ခလုတ် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် ကြိမ် သည် ၀ ဖြစ်ပါစေ။
နှိပ် ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
// nested in နှိပ်: it reaches ကြိမ် and အမည် through နှိပ်
ပြ ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"{အမည်}: {ကြိမ်}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ကြိမ် ကို ကြိမ် + ၁ ဟု ထားမည်။
ပြ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
သိမ်း ကို နှိပ် ဟူသော လုပ်ဆောင်ချက် ဟု ထားမည်။
ခလုတ်(အမည်: အမည်၊ နှိပ်: နှိပ် ဟူသော လုပ်ဆောင်ချက်) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ခလုတ်သစ် ဟူသော လုပ်ဆောင်ချက်ကို "အိုကေ" ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
(ခ ၏ နှိပ်) ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
သိမ်း ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
(ခ ၏ နှိပ်) ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
ခ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အိုကေ: ၁
အိုကေ: ၂
အိုကေ: ၃
ခလုတ်(အမည်: "အိုကေ"၊ နှိပ်: နှိပ် ဟူသော လုပ်ဆောင်ချက်)
```

`==` and the test `X နှင့် Y သည် တစ်ခုတည်း ဖြစ်လျှင်` both ask whether two function values
contain one function: the same function of the file, or the same closure. The two calls of
`ရေတွက်စက်` above make two closures, so the two values are not equal, though their code is
the same.

## A fresh variable each round

A local declared inside the block of a loop is a new variable in each round, as a `let` in
JavaScript is, so closures made in a loop each keep their own round. The counter of a
counted loop and the cell variable of an each-loop are also new in each round. The variable
of a generic loop is one variable for the whole loop, because its update passes it from
round to round, and so is each variable declared before the loop:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပြများ သည် (လုပ်ဆောင်ချက် အမျိုးအစား) စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [] ဖြစ်ပါစေ။
// a counted loop: န is new in each round
၁ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပြ ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ရေတွက် {န}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပြများ ထဲသို့ ပြ ဟူသော လုပ်ဆောင်ချက် ကို ထည့်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
// a generic loop: မ is one variable for each round
ကိန်းရှင် မ သည် ၁ ဖြစ်ကာ မ <= ၂ နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဆ သည် မ * ၁၀ ဖြစ်ပါစေ။
ပြ၂ ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မ {မ}၊ ဆ {ဆ}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပြများ ထဲသို့ ပြ၂ ဟူသော လုပ်ဆောင်ချက် ကို ထည့်မည်။
ပြီးလျှင် မ ကို မ + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပြများ ၏ အကွက်တိုင်း ကို ဖ အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဖ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ရေတွက် ၁
ရေတွက် ၂
ရေတွက် ၃
မ ၃၊ ဆ ၁၀
မ ၃၊ ဆ ၂၀
```

## Closures and C

A closure, which is a nested function with captures or a variable that contains one, goes
to a parameter of a function kind of a [C function](functions-c.md#c-functions) just as a
function of the program does. C gets a pointer that it can call, and a call from C
runs the closure.

### Lent for the call

A closure given to a C call is lent to that call: C can call it while the call runs, and
the call needs nothing more.

```text
the C call runs  ->  C calls the closure  ->  fine
the C call ends  ->  C calls the closure  ->  the trap `closure escape`
```

### Leave to call later

```text
<function value> ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်။
<function value> ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြန်လည်ရုပ်သိမ်းမည်။
```

A C function that keeps the function, as `signal` keeps a handler, calls it after the call
that gave it has ended. For that, the closure needs *leave* (permission to be called
later). We use the first statement to give C leave to call the closure after the call that
gave it, and the leave keeps a count of the record of the closure until it ends. We use the
second statement to take the leave back. A leave that is still open when the program ends
ends with it, because C can call nothing more, so `--leaks` lists nothing for it.

In the program below, `signal` keeps a handler and `raise` calls it. (The sample
`samples/ffi/closures.zaka` adds a `qsort` call, which needs no leave.)

```zaka
signal ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
sig ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
handler ဟူသော (၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို
လည်းကောင်း လက်ခံမည်။
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

The rules of the two statements:

- Before `ကို`, we can write a function value in each form: the name form
  `X ဟူသော လုပ်ဆောင်ချက်`, a variable, a field or a cell. A value of another data kind (type) is
  ZK0462.
- `ပြင်ပမှ` refers to code outside the program, as in `ပြင်ပမှလာသော`. The words
  `ခေါ်ဆိုခွင့်`, `ပြုမည်` and `ပြန်လည်ရုပ်သိမ်းမည်` are reserved, and the joined
  spelling `ခေါ်ဆိုခွင့်ပြုမည်` is the same as the spaced one. Each statement also works as
  the inline block of an `if`.
- A second leave of one closure does nothing, and so does a revoke of a closure that has no
  leave. A function with no captures needs no leave, so the statement does nothing for it.
- Two values with the same function and the same captures are one closure, so one leave
  covers both. The closures of different rounds of a loop are different closures, so each
  round gives its own leave.

```zaka-error ZK0462
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဂ သည် ၅ ဖြစ်ပါစေ။
ဂ ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0462]: this value is isize, not a function value, so C cannot get leave to call it
```

### A late call

When C calls a closure while no C call is running and the closure has no leave, the
program stops with the trap `closure escape` (see [When a program
stops](../vm/traps.md#all-the-traps)), and the message tells how to give
leave. The native code of a closure remains in place after its last use, so a late call
traps and never runs freed code.

### Calls from other threads

C can also call a function from another OS thread, when each of these is true:

- the function, and each function it calls, uses no mutable global;
- no statement writes the variables that it captures;
- each captured value is one that threads can use: no function value, pointer or handle,
  and no object other than one of a struct marked
  `အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော`, whose fields each thread reaches in a lock
  block of the object.

A write to a field of an object does not count as a write to the variable that refers to
it. The counts of the heap turn atomic once such a function goes to C, and a trap on that
OS thread stops the program. A thread that C starts runs after the call that started it
returns, so a closure that it calls needs leave. A call of each other function from another
OS thread ends the run of the program with a message, and so does each call from another OS
thread in the VM, which runs on one OS thread.

In a C lib ([Functions C can call](functions-exports.md)), C can call from each
OS thread. The lock of the lib lets one thread in at a time, a call that waits in C lets the
others in, and a trap on a thread with no call of the lib in progress stops the lib and
returns zeros to C. In a lib, a function that calls a function value takes that lock,
because the value can come from C.

### Where it runs

`zakac run` makes one native closure for each closure given to C, which it finds again from
the octets of its record, and a native program does the same through the runtime that it
links. The C target writes an extra file, `zaka_callback.c`, with a bridge for each such
function kind, when a program makes a closure and gives function values to C or contains a
leave statement, and it links `-lffi`. The native target writes the bridges as machine
code. See [Native builds](../compiler/native.md).

A C declaration accepts function values of numbers, true/false values and addresses alone
(ZK0465), so each closure that reaches C has a native closure. A leave of a function value
whose function kind accepts or returns a struct, text or a function value, which no C
function can get, is a construct that a native build cannot write.

## How a closure runs

A captured local that no statement writes after its declaration, in the function around it
or in a nested one, is duplicated: the closure keeps its own count of the value, made where
the closure is made, and the local remains in its frame. The duplicate behaves the same as
the variable, because neither side can change it. Each other captured local lives in a
counted *box*, a block of one cell on the heap, and each read and write of it, inside the
nested function and in the block around it, goes through the box. A local that no closure
captures remains in its frame. A local of a loop block gets a new box in each round, and a
captured counter of a counted loop, or the cell variable of an each-loop, is duplicated into
a new box in each round.

The function of a nested function accepts its declared parameters, then its own closure,
then each capture, passed with a count: a duplicate, a box, or the closure of another nested
function. Where the function is declared, the program fills a counted record of a code
word, an info word and one field for each capture, and a hidden variable of the block keeps
it. A call through a closure runs a *thunk*, which accepts the record last and calls the
function with its fields. A call of the function inside itself passes its own closure and
captures on. See [call_value](../vm/instructions.md) and [Function values in
C](../compiler/native.md#function-values-and-closures).

## Errors

| Code | Problem |
| --- | --- |
| ZK0214 | an external function, a C function or a program start block declared inside a function |
| ZK0304 | two nested functions of one name in one scope |
| ZK0462 | a leave for C of a value that is no function value |
| ZK0465 | a C declaration whose function kind uses data kinds other than numbers, true/false values and addresses |

The errors of function kinds and calls through function values are on [Function
values](function-values.md#errors-at-a-glance).
