---
title: Function parameters
description: How to write the parameters of a function, the markers a parameter can have, mutable and writable parameters, and what a call does with each argument.
---

# Function parameters

A parameter is a variable inside a function, and each call sets it from the matching
argument. This page shows how we write parameters in the function signature, the markers
that a parameter can have, mutable parameters (`inout`) and writable parameters (`out`),
and what a call does with an argument of each data kind (type). The Learn page
[Parameters](../../learn/functions/parameters.md) teaches the same ideas with examples.

```zaka
ပြ ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် သေး သည် ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇ ဖြစ်ပါစေ။
ပြ ဟူသော လုပ်ဆောင်ချက်ကို သေး ဖြင့် လုပ်ဆောင်မည်။ // 32 bits widens to the full width
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၇
```

## Writing a parameter

```text
param := <name> ဟူသော <kind> တစ်<word> ကို [ဒေတာ ပြောင်းလဲရန် | ဒေတာ ထည့်ပေးရန်] [အပိုင်အနေဖြင့်] [ဝိဘတ် "<preposition>" ကို သုံး၍]

<param> လည်းကောင်း <param> လည်းကောင်း … လက်ခံပြီး      (two or more)
<param> [လည်းကောင်း] လက်ခံပြီး                          (one)
```

In the function signature, we can use `<name> ဟူသော <kind> တစ်<word> ကို` to define a
parameter, which means "the `<kind>` called `<name>`". The data kind comes with its
[measure word](kinds.md#measure-words) (classifier), and after `ကို` we can add these
markers, in this order:

| Marker | Meaning |
| --- | --- |
| `ဒေတာ ပြောင်းလဲရန်` | the function reads and changes the caller's place, which the caller sets first ([Mutable parameters](#mutable-parameters)) |
| `ဒေတာ ထည့်ပေးရန်` | the function writes the caller's place, which the caller does not need to set ([Writable parameters](#writable-parameters)) |
| `အပိုင်အနေဖြင့်` | the function takes over the block that a pointer points to; for pointers alone ([Unsafe Zaka](unsafe.md)) |
| `ဝိဘတ် "<preposition>" ကို သုံး၍` | the word that follows the argument in a call ([Prepositions](functions-sentences.md#prepositions)) |

A parameter is a local variable that starts with the value of its argument, and it follows
the rules of other local variables. It can have each data kind that a variable can have: a
number, a true/false value, a character, text, a list, a struct, a shared object, a
function value or a pointer. The argument must fit the data kind of its parameter under the
rules of [assignment](conversions.md): a value that widens without loss converts on its
own, as the 32-bit integer in the example above does, and each other difference needs an
explicit conversion (ZK0401).

The rules of the parameter list:

| Form | Rule | Code |
| --- | --- | --- |
| two or more parameters | each one ends in `လည်းကောင်း`, before the next parameter and before `လက်ခံပြီး` | ZK0201 |
| one parameter | `လည်းကောင်း` can be written or left out | |
| `ဒေတာ ပြောင်းလဲရန်`, `ဒေတာ ထည့်ပေးရန်` | a parameter has one of them at most, and each is a phrase of two words, so `ပြောင်းလဲရန်` or `ထည့်ပေးရန်` alone is an error | ZK0201 |
| the markers | in the order of the table above | ZK0201 |
| the names | the parameter names of one function signature differ from each other | ZK0305 |
| `အပိုင်အနေဖြင့်` | the data kind of the parameter has a pointer in it | ZK0470 |
| a function kind with parameters | as the data kind of a parameter, it goes in parentheses: `ဖ ဟူသော (ကိန်းပြည့် တစ်ခု ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို` | ZK0239 |

The spaces inside a parameter are optional, so `ကိုလည်းကောင်း` is the same as
`ကို လည်းကောင်း`. The program start block and C functions write their parameters in the
same form, while a [function kind](function-values.md#function-kinds) writes its parameters
without names. A C function can also end its parameters with `အခြားတန်ဖိုးများနှင့်အတူ`
to take more arguments than it names (variadic); see
[Other values](functions-c.md#other-values).

## How values are passed

Each parameter is a new variable, and the call stores the argument in it, just as an
assignment from one variable to another stores the value. What that store does depends on
the data kind:

| Data kind of the parameter | What the call does | Can the function change the caller's value? |
| --- | --- | --- |
| number, true/false value, character, enumeration | copies the value | no |
| text, plain list, counted list, struct, result | copies the value; text and plain lists share their cells until the first write | no |
| shared object | shares the object | it can: a change to its fields is seen through each name |
| function value | shares the function | no; the captures of a closure are shared with each holder |
| pointer, in unsafe code | copies the address | through the address; with `အပိုင်အနေဖြင့်`, the caller's place has the null address after the call |
| a parameter marked `ဒေတာ ပြောင်းလဲရန်` | copies the value in, and copies it out at the return | it can: the value goes back to the caller's place |
| a parameter marked `ဒေတာ ထည့်ပေးရန်` | starts unset, and copies the value out at the return | it does: the function sets the caller's place on each path |

A duplicate (copy) is what the call means, and the compiler skips the work where nothing
can tell the difference. Text and a plain list keep their cells in a counted block, so a
duplicate shares the block and adds to its count. The first write to a shared block, to a
cell or to the whole value, copies the cells into a new block (the VM operation `own`), so
the caller's value never changes. When the caller reads the text or the plain list again
after the call, the value is lent to the parameter for the length of the call, with no
count at all, and a value that the caller does not read again moves in. See
[Values and objects](objects.md).

```zaka
အကောင့် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင်
အမည် ဟူသော စာသား တစ်ခု နှင့်
လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု
တို့ပါဝင်သည်။

// a change to the list remains in the function
နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် သည်
ကိန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းများ ၏ အကွက် ၀ ကို ၉၉ ဟု ထားမည်။
ကိန်းများ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// a change to the object is seen through each name
အပ်ငွေ ဟူသော လုပ်ဆောင်ချက် သည်
အ ဟူသော အကောင့် တစ်ခု ကို လည်းကောင်း
ပမာဏ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အ ၏ လက်ကျန် ကို (အ ၏ လက်ကျန်) + ပမာဏ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စ သည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
နှစ်ဆ ဟူသော လုပ်ဆောင်ချက်ကို စ ဖြင့် လုပ်ဆောင်မည်။
စ ကို ပြသမည်။
ကိန်းရှင် က သည် အကောင့်(အမည်: "မမ"၊ လက်ကျန်: ၁၀၀) ဖြစ်ပါစေ။
အပ်ငွေ ဟူသော လုပ်ဆောင်ချက်ကို က နှင့် ၅၀ တို့ဖြင့် လုပ်ဆောင်မည်။
က ၏ လက်ကျန် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၉၉၊ ၂၊ ၃]
[၁၊ ၂၊ ၃]
၁၅၀
```

When a function must change the caller's value, we give it a
[mutable parameter](#mutable-parameters) or a shared object. Pointers, the heap and the
owned marker `အပိုင်အနေဖြင့်` on parameters and results belong to unsafe code, and their
rules are in [Unsafe Zaka](unsafe.md).

## Mutable parameters

```text
<name> ဟူသော <kind> တစ်<word> ကို ဒေတာ ပြောင်းလဲရန်
```

A mutable parameter (`inout`) lets a function change a variable, a field or a cell of the
caller. To make one, we write `ဒေတာ ပြောင်းလဲရန်` ("to change the data") after the `ကို`
of the parameter. The function starts with the value in the caller's place, and what the
parameter contains when the function returns goes back to that variable, field or cell. So
the caller sets the place before the call, and a place that can be unset there is ZK0722.
A function that writes the place and never reads it uses a
[writable parameter](#writable-parameters) instead.

The call writes the argument with no mark, in the word form and the sign form alike, and
the language server shows such an argument with a hint.

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။
တိုး ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို န + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ၅ ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် လူ(အသက်: ၃၀) ဖြစ်ပါစေ။
တိုး ဟူသော လုပ်ဆောင်ချက်ကို က ဖြင့် လုပ်ဆောင်မည်။
တိုး ဟူသော လုပ်ဆောင်ချက်ကို ပ ၏ အသက် ဖြင့် လုပ်ဆောင်မည်။
က ကို ပြသမည်။
ပ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၆
လူ(အသက်: ၃၁)
```

With two mutable parameters a function can swap two variables, and a mutable list grows in
the caller:

```zaka
လဲလှယ် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လည်းကောင်း
ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ယာယီ သည် က ဖြစ်ပါစေ။
က ကို ခ ဟု ထားမည်။
ခ ကို ယာယီ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

နှစ်ဆတိုး ဟူသော လုပ်ဆောင်ချက် သည်
ကိန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ (ကိန်းများ ၏ အကွက်အရေအတွက်) == ၀ ဖြစ်လျှင် ပြန်ပေးမည်။
၀ မှ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၁ အထိ န အနေဖြင့် ရေတွက်၍
ကိန်းများ ၏ အကွက် န ကို (ကိန်းများ ၏ အကွက် န) * ၂ ဟု ထားမည်။
ကိန်းများ ထဲသို့ ၀ ကို ထည့်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပထမ သည် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် ဒုတိယ သည် ၂ ဖြစ်ပါစေ။
လဲလှယ် ဟူသော လုပ်ဆောင်ချက်ကို ပထမ နှင့် ဒုတိယ တို့ဖြင့် လုပ်ဆောင်မည်။
"{ပထမ} {ဒုတိယ}" ဟု ပြသမည်။
ကိန်းရှင် မူရင်း သည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
နှစ်ဆတိုး ဟူသော လုပ်ဆောင်ချက်ကို မူရင်း ဖြင့် လုပ်ဆောင်မည်။
မူရင်း ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂ ၁
[၂၊ ၄၊ ၆၊ ၀]
```

### How the value goes back

The parameter works on a value inside the function, which goes back to the caller's place
when the call returns, as Swift's `inout` does: copied in, copied out. So the function
never refers to the place itself, and a place that the function also reaches through another
route remains sound. In the next example, the function adds to the list whose cell it
changes, and the new value lands in the cell after the call.

```zaka
ကိန်းရှင် ဂရစ် သည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။

ချဲ့ ဟူသော လုပ်ဆောင်ချက် သည်
ပ ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဂရစ် ထဲသို့ ၄ ကို ထည့်မည်။
ပ ကို ၉၉ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ချဲ့ ဟူသော လုပ်ဆောင်ချက်ကို ဂရစ် ၏ အကွက် ၀ ဖြင့် လုပ်ဆောင်မည်။
ဂရစ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၉၉၊ ၂၊ ၃၊ ၄]
```

The compiler passes the address of the place straight in when nothing else can reach it,
which is the case for a local that no closure shares, so the common case costs nothing
more. A place under a global or a captured variable is copied into a hidden slot, and the
slot goes back to the place right after the call. The place is found again at that point,
so a cell lands where the list has it after the call.

### Rules of mutable parameters

| Rule | Code |
| --- | --- |
| the argument is a place: a variable, a field or a cell | ZK0410 |
| the argument is not a constant | ZK0306 |
| the place is set before the call, since the function can read it | ZK0722 |
| the place has the data kind of the parameter, with no widening | ZK0401 |
| the place appears in no other argument of the same call, as a whole or through a field or a cell, since the order in which the values go back would decide the result | ZK0433 |

- A closure inside the function that uses the parameter shares the function's value, and
  what the parameter contains when the function returns goes back to the caller.
- A call through a function value whose function kind has `ဒေတာ ပြောင်းလဲရန်` changes its
  argument as a call to a named function does.
- When a pointer parameter also has `အပိုင်အနေဖြင့်`, the function takes over the old block
  and the caller owns whatever is written back ([Unsafe Zaka](unsafe.md)).

For a single value, we can also return the new value and assign it,
`က ကို နောက်တစ်(က) ဟု ထားမည်။`, which does the same work. A mutable parameter suits a
function that changes several places at once.

```zaka-error ZK0410
တိုး ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို န + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
တိုး ဟူသော လုပ်ဆောင်ချက်ကို ၅ ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0410]: this cannot be assigned to; a target must be a variable or a dereference
```

```zaka-error ZK0433
ထား ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လည်းကောင်း မ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို မ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ၅ ဖြစ်ပါစေ။
ထား ဟူသော လုပ်ဆောင်ချက်ကို က နှင့် က + ၁ တို့ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0433]: "က" is changed through another argument of this call, so it cannot appear here
```

## Writable parameters

```text
<name> ဟူသော <kind> တစ်<word> ကို ဒေတာ ထည့်ပေးရန်
```

A writable parameter (`out`) gives the function a place of the caller to fill, as the
`out` parameters of C# do. To make one, we write `ဒေတာ ထည့်ပေးရန်` ("to put the data in")
after the `ကို` of the parameter. The caller does not need to set the place first: it can
declare the variable with no value, and the place counts as set after the call. With
writable parameters, a function can return more than one value, each in its own place:

```zaka
ခွဲ ဟူသော လုပ်ဆောင်ချက် သည်
တန် ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
လဒ် ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ထည့်ပေးရန် လည်းကောင်း
ကြွင်း ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ထည့်ပေးရန် လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လဒ် ကို တန် / ၃ ဟု ထားမည်။
ကြွင်း ကို တန် % ၃ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။ // no value: the call sets it
ကိန်းရှင် ကျ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
ခွဲ ဟူသော လုပ်ဆောင်ချက်ကို ၁၀၊ လ နှင့် ကျ တို့ဖြင့် လုပ်ဆောင်မည်။
"{လ} {ကျ}" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၃ ၁
```

Inside the function, the parameter starts unset, as a variable declared with no value
does. The function sets the whole of it on each path before it returns, a struct one field
at a time or whole, and it reads the parameter after setting it, never before:

| Rule | Code |
| --- | --- |
| the function does not read the parameter before it sets it | ZK0722 |
| no return, and not the end of the function, is reached on a path with the parameter unset | ZK0725 |
| the argument is a place, as for a mutable parameter | ZK0410 |
| the place has the data kind of the parameter | ZK0401 |
| the place appears in no other argument of the call | ZK0433 |
| a [function kind](function-values.md#function-kinds) can use `ဒေတာ ပြောင်းလဲရန်` but not `ဒေတာ ထည့်ပေးရန်` | ZK0201 |

The value goes back to the caller's place when the call returns. For a
[C function](functions-c.md#lists-of-numbers), the marker is the author's promise that C
writes the place: the compiler cannot see inside C, so it trusts the marker. It works on
the data kinds that `ဒေတာ ပြောင်းလဲရန်` works on, and each other data kind is ZK0466.

```zaka-error ZK0725
တစ်ဝက် ဟူသော လုပ်ဆောင်ချက် သည်
တန် ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လဒ် ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ထည့်ပေးရန် လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ တန် သည် ၀ ထက် ကြီးလျှင် လဒ် ကို တန် / ၂ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0725]: "လဒ်" is marked "ဒေတာ ထည့်ပေးရန်", so the function must set it on each path, but a path reaches this return without setting it
```
