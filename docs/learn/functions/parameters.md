---
title: Parameters
description: Pass values into a function, and let a function change or set the caller's variables.
---

# Parameters

A function becomes far more useful when it gets values to work with. The values that a
function accepts are its *parameters*, and the values that a call passes in are the
*arguments*. A parameter can also be a mutable parameter (`inout`) or a writable parameter
(`out`), which lets the function change or set a variable of the caller.

## You will learn

- how to give a function one or more parameters, and how to pass arguments in a call
- how to write the arguments before the function name
- how to let a function change a variable of the caller, or set one for it
- what happens when a call does not match the function

```zaka
// "Function count-up, accepting an integer amount, will do the following"
ရေတွက်ပြ ဟူသော လုပ်ဆောင်ချက် သည်
အရေအတွက် ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ မှ အရေအတွက် အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// "Run the function count-up with 3"
ရေတွက်ပြ ဟူသော လုပ်ဆောင်ချက်ကို ၃ ဖြင့် လုပ်ဆောင်မည်။
"---" ဟု ပြသမည်။
ရေတွက်ပြ ဟူသော လုပ်ဆောင်ချက်ကို ၂ ဖြင့် လုပ်ဆောင်မည်။
```

```text
၁
၂
၃
---
၁
၂
```

Inside the function, `အရေအတွက်` is a variable that has the value the call passed in, so it
is 3 in the first call and 2 in the second. In the function signature, we can use
`<name> ဟူသော <kind> တစ်ခု ကို` to define a parameter, and in the call we write the argument
before `ဖြင့်`:

```text
<name> ဟူသော လုပ်ဆောင်ချက် သည်
<parameter> ဟူသော <kind> တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။

<name> ဟူသော လုပ်ဆောင်ချက်ကို <argument> ဖြင့် လုပ်ဆောင်မည်။
```

| Zaka | Reads as |
| --- | --- |
| `အရေအတွက် ဟူသော ကိန်းပြည့် တစ်ခု` | "an integer called `အရေအတွက်`" |
| `ကို` | (the object particle) |
| `လက်ခံပြီး` | "having accepted" |
| `၃ ဖြင့်` | "with 3" |

The word after `တစ်` is the measure word (classifier) of the data kind (type), which is `ခု`
for most data kinds.

## Several parameters

```text
<parameter> ဟူသော <kind> တစ်ခု ကို လည်းကောင်း <parameter> ဟူသော <kind> တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
<name> ဟူသော လုပ်ဆောင်ချက်ကို <argument> နှင့် <argument> တို့ဖြင့် လုပ်ဆောင်မည်။
```

When a function has two or more parameters, we end each one with `လည်းကောင်း` ("and
also"). In the call, we pass the arguments in the same order, with `နှင့်` ("and") before
the last one and `တို့ဖြင့်` ("with these") after it:

```zaka
// print the area of a length × width rectangle
ဧရိယာပြ ဟူသော လုပ်ဆောင်ချက် သည်
အလျား ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း အနံ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အလျား * အနံ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ဧရိယာပြ ဟူသော လုပ်ဆောင်ချက်ကို ၃ နှင့် ၄ တို့ဖြင့် လုပ်ဆောင်မည်။
ဧရိယာပြ ဟူသော လုပ်ဆောင်ချက်ကို ၁၀ နှင့် ၂ + ၃ တို့ဖြင့် လုပ်ဆောင်မည်။     // an argument can be a sum
```

```text
၁၂
၅၀
```

| Count | Each parameter ends in | The arguments end in | Example call |
| --- | --- | --- | --- |
| one | `ကို` | `ဖြင့်` | `… ၃ ဖြင့် လုပ်ဆောင်မည်။` |
| two | `ကို လည်းကောင်း` | `နှင့် … တို့ဖြင့်` | `… ၃ နှင့် ၄ တို့ဖြင့် လုပ်ဆောင်မည်။` |
| three or more | `ကို လည်းကောင်း` | `၊ … နှင့် … တို့ဖြင့်` | `… ၁၊ ၂ နှင့် ၃ တို့ဖြင့် လုပ်ဆောင်မည်။` |

With three or more arguments, we put `၊` between the first ones and `နှင့်` before the last,
just as in a list of words in a Burmese sentence.

## Arguments first

```text
<argument> ကို သုံး၍ <name> မည်။
<argument> နှင့် <argument> တို့ကို သုံး၍ <name> မည်။
```

A call can also give the arguments first and the function name last, which is the order of
a Burmese sentence, where the object comes before its verb. We write `သုံး၍` ("using")
after the arguments and `မည်` ("will") after the name:

```zaka
ဧရိယာပြ ဟူသော လုပ်ဆောင်ချက် သည်
အလျား ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း အနံ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အလျား * အနံ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// "Using 3 and 4, (the function) area-show will (run)"
၃ နှင့် ၄ တို့ကို သုံး၍ ဧရိယာပြ မည်။
၆ နှင့် ၇ တို့ကို သုံး၍ ဧရိယာပြ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
```

```text
၁၂
၄၂
```

| Zaka | Reads as |
| --- | --- |
| `၃ နှင့် ၄ တို့ကို သုံး၍` | "using 3 and 4": `တို့ကို` after several arguments, `ကို` after one |
| `ဧရိယာပြ မည်` | "area-show will (run)" |
| `ဧရိယာပြ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်` | "will run the function area-show", the long form |

An argument in this form is a name, a literal, a sum or a call in the sign form, which
[Results](results.md#the-sign-form) teaches. When an argument is a whole sentence that
gives a value, we put it in parentheses. [Results](results.md) also shows the form of this
call that returns a value.

## Parameters are local copies

A parameter is a new variable inside the function, and it gets a duplicate (copy) of the
argument. So when the function changes the parameter, the variable that the caller passed
in does not change:

```zaka
တိုးပြ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို က + ၁ ဟု ထားမည်။
က ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
တိုးပြ ဟူသော လုပ်ဆောင်ချက်ကို ခ ဖြင့် လုပ်ဆောင်မည်။     // prints 6
ခ ကို ပြသမည်။                                    // still 5
```

```text
၆
၅
```

## Changing the caller's variable

```text
<parameter> ဟူသော <kind> တစ်ခု ကို ဒေတာ ပြောင်းလဲရန်
```

One method to change the caller's variable is to return the new value and set the variable
from it, which [Results](results.md) shows. When a function must change the variable
itself, we write `ဒေတာ ပြောင်းလဲရန်` ("to change the data") after the `ကို` of the
parameter, which makes it a *mutable parameter* (`inout`), and the call passes the variable
as before:

```zaka
တိုး ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို က + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ခ သည် ၅ ဖြစ်ပါစေ။
တိုး ဟူသော လုပ်ဆောင်ချက်ကို ခ ဖြင့် လုပ်ဆောင်မည်။
ခ ကို ပြသမည်။
```

```text
၆
```

| Zaka | Reads as |
| --- | --- |
| `ဒေတာ` | "data" |
| `ပြောင်းလဲရန်` | "to change" |

The function works on its own value, and that value goes back to the caller's variable when
the call returns, as with `inout` in Swift. The argument must be something we can assign
to, which is a variable, a field or a cell, and because the function can read it, the
caller gives it a value before the call.

## Filling the caller's variable

```text
<parameter> ဟူသော <kind> တစ်ခု ကို ဒေတာ ထည့်ပေးရန်
```

When a function sets a variable of the caller and never reads it first, we write
`ဒေတာ ထည့်ပေးရန်` ("to put the data in") after the `ကို` of the parameter, which makes it
a *writable parameter* (`out`). The caller can then declare the variable with no value,
and after the call the variable has the value that the function put in it. This lets one
call return two values:

```zaka
စားခွဲ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း စား ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ထည့်ပေးရန် လည်းကောင်း ကြွင်း ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ထည့်ပေးရန် လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စား ကို က / ခ ဟု ထားမည်။
ကြွင်း ကို က % ခ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ဆ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ကျန် သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
စားခွဲ ဟူသော လုပ်ဆောင်ချက်ကို ၁၇၊ ၅၊ ဆ နှင့် ကျန် တို့ဖြင့် လုပ်ဆောင်မည်။
"{ဆ} {ကျန်}" ဟု ပြသမည်။
```

```text
၃ ၂
```

| Zaka | Reads as |
| --- | --- |
| `ဒေတာ` | "data" |
| `ထည့်ပေးရန်` | "to put in" |

Inside the function, a writable parameter starts with no value, so the function must set
it before it reads it, and it must set it on each path before it returns. When a path can
leave it unset, the compiler stops with an error:

```zaka-error ZK0725
အပေါင်းဖြစ်လျှင်ထား ဟူသော လုပ်ဆောင်ချက် သည်
တန် ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လဒ် ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ထည့်ပေးရန် လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ တန် သည် ၀ ထက် ကြီးလျှင် လဒ် ကို တန် ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
error[ZK0725]: "လဒ်" is marked "ဒေတာ ထည့်ပေးရန်", so the function must set it on each path, but a path reaches this return without setting it
```

When the number is 0 or less, the function reaches its end without setting `လဒ်`, so the
fix is to set `လဒ်` in that case too.

## Matching the call to the function

A call must pass one argument for each parameter of the function, and each argument must
fit the data kind of its parameter:

```zaka-error ZK0314
ပြ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပြ ဟူသော လုပ်ဆောင်ချက်ကို ၁ နှင့် ၂ တို့ဖြင့် လုပ်ဆောင်မည်။
```

```text
error[ZK0314]: function "ပြ" takes 1 arguments, 2 given
```

An argument of the wrong data kind, such as `မှန်` for an integer parameter, is the error
ZK0401.

## Details

- After a lone parameter, `လည်းကောင်း` is optional. With two or more parameters, a
  parameter that does not end in `လည်းကောင်း` is the error ZK0201, and two parameters of
  one function with the same name are ZK0305
  ([Parameters](../../manual/language/functions-parameters.md#writing-a-parameter)).
- An argument is a whole expression, and `၊` ends it, so an argument needs no parentheses
  in the `ဖြင့်` form ([Calls](../../manual/language/functions-calls.md#arguments)).
- A value that widens without loss, such as a 32-bit integer for a `ကိန်းပြည့်` parameter,
  converts on its own; other differences need a conversion
  ([Converting between data kinds](../operators/conversion.md)).
- The argument of a mutable or writable parameter must be a variable, a field or a cell
  (ZK0410), not a constant (ZK0306), with the same data kind as the parameter and no
  widening (ZK0401), and it cannot appear in a second argument of the same call (ZK0433)
  ([Mutable parameters](../../manual/language/functions-parameters.md#mutable-parameters)).
- The argument of a mutable parameter must have a value before the call (ZK0722), and a
  writable parameter that the function reads before it sets it is ZK0722 too ([Writable
  parameters](../../manual/language/functions-parameters.md#writable-parameters)).
- A struct or a list can be a mutable parameter as well, as [Structs](../structs/README.md#passing-to-a-function)
  and [Passing and comparing lists](../lists/passing.md#lists-as-parameters) show.
- Parameters can also have prepositions, which let a call read as a Burmese sentence:
  [Prepositions](prepositions.md) teaches them after [Results](results.md).

## Recap

- In the function signature, we use `<name> ဟူသော <kind> တစ်ခု ကို` for each parameter,
  with `လည်းကောင်း` after each one when there are two or more, and then `လက်ခံပြီး`.
- A call passes the arguments in the same order, closed with `ဖြင့်` or `တို့ဖြင့်`, or
  before the name with `ကို သုံး၍` or `တို့ကို သုံး၍`.
- A parameter is a duplicate of its argument, so a change to it remains inside the function.
- `ဒေတာ ပြောင်းလဲရန်` makes a mutable parameter (`inout`), which changes the caller's
  variable, and `ဒေတာ ထည့်ပေးရန်` makes a writable parameter (`out`), which sets it.

## Hands on

- Write a function that accepts a number and prints its times table from 1 to 10.
- Give `ရေတွက်ပြ` a guard, `အကယ်၍ အရေအတွက် သည် ၁ အောက် ငယ်လျှင် ပြန်ပေးမည်။`, then call it
  with `၀`.
- Write a function with two mutable parameters that swaps the values of two variables.
- Call `စားခွဲ` with a constant in place of `ဆ`, and read the error.

Next: [Results](results.md).
