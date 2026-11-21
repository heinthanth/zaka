---
title: Optional values
description: The optional kind, a data cell that might contain a value: how to declare it, test it, read it and return one, and the reason Zaka has it in place of a NULL.
---

# Optional values

Some values are there at times and missing at others: the age of a person who left it out,
or the first match of a search that found nothing. Zaka gives no data kind (type) a NULL
to mark such a value as missing. Instead, we use an *optional kind* (option type, nullable
type): a data cell that might contain a value, or might be blank (none, null). The optional
kind shows that the value can be missing, so the program must test the cell before it reads the
value in it. This page states the full rules, and the Learn book teaches optionals in
[Optional values](../../learn/optionals/README.md).

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု နှင့် အသက် ဟူသော ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု တို့ပါဝင်သည်။

ရှာ ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၁ နှင့် ညီလျှင် "တစ်" ကို ပြန်ပေးမည်။
အလွတ်ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပြ ဟူသော လုပ်ဆောင်ချက် သည်
စ ဟူသော စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ စ ထဲတွင် ဒေတာ ရှိလျှင် "တွေ့: {စ ထဲမှ ဒေတာ}" ဟု ပြသမည်။
သို့မဟုတ်ပါက "မတွေ့" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပြ ဟူသော လုပ်ဆောင်ချက်ကို ရှာ(၁) ဖြင့် လုပ်ဆောင်မည်။
ပြ ဟူသော လုပ်ဆောင်ချက်ကို ရှာ(၂) ဖြင့် လုပ်ဆောင်မည်။
ပြ ဟူသော လုပ်ဆောင်ချက်ကို "ဟယ်လို" ဖြင့် လုပ်ဆောင်မည်။
ကိန်းရှင် လ သည် လူ(အမည်: "မောင်"၊ အသက်: အလွတ်) ဖြစ်ပါစေ။
လ ကို ပြသမည်။
လ ၏ အသက် ကို ၃၀ ဟု ထားမည်။
လ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
တွေ့: တစ်
မတွေ့
တွေ့: ဟယ်လို
လူ(အမည်: "မောင်"၊ အသက်: အလွတ်)
လူ(အမည်: "မောင်"၊ အသက်: ဒေတာ(၃၀))
```

## The optional kind

```text
<kind> ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်
```

The words mean "a data cell that might contain" a value of `<kind>`. We can write an
optional kind (option type) wherever a data kind goes: in a declaration, a parameter, a
field of a struct and the result of a function. In a declaration and a result, `တစ်ခု`
follows it, as it follows other data kinds, and the measure word must fit the data kind
(the warning ZK0509).

| Written | Meaning |
| --- | --- |
| `ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` | an integer, or nothing |
| `စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` | text, or nothing |
| `ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် စာရင်း` | a plain list of optionals |
| `အကွက် ၂ ခုရှိသော ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် စာရင်း` | a list of 2 optionals |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` | a list of 3 integers, or nothing |
| `ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် လိပ်စာ` | a pointer to an optional, for [unsafe code](unsafe.md) |

A list or pointer level that we write after `ဒေတာဆဲလ်` wraps the whole optional, just as
the levels of a [result kind](result-kinds.md#lists-and-pointers-of-results) wrap the whole
result. A count written before the data kind goes to its lists from the innermost out, as
in nested lists: first to a list inside the data, then to the levels after `ဒေတာဆဲလ်`. A
list literal fills the data of an optional. Messages write the optional kind as `optional isize`.

### Values of an optional kind

We can put two sorts of value into an optional:

- `အလွတ်` ("blank data") is the blank value (none), which contains nothing. We can use it
  wherever an optional is wanted: as the value of a declaration, in an assignment, as an
  argument, in a return, as a field of a struct literal, or as a cell to add to a list.
  `အလွတ်` where no optional is wanted is the error ZK0477.
- A plain value of the data kind inside goes in as it is, with no word around it. A
  `ကိန်းပြည့်` fits an optional of `ကိန်းပြည့်`, and from then on the cell contains it.
  This works the same for an argument, an assignment with `ထားမည်`, a field and a return.

A literal gets the data kind inside the optional, so `၁` is an `၈ ဘစ် ကိန်းပြည့်` where
an optional of that data kind is wanted, and a literal that does not fit that data kind is
ZK0406, as it is for a plain variable.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ၈ ဘစ် ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
အသက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဒေတာ(၁)
```

### The initial value and printing

The [initial value](kinds.md#initial-values) (default value) of an optional is `အလွတ်`, so
`ကနဦးတန်ဖိုး` fills a counted list of optionals with blank cells. A declaration with no
value does not contain `အလွတ်`, though: the variable is unset, as a variable of each other
data kind is, and the program must give it a value before it reads it ([Set before
read](declarations.md#set-before-read)).

An optional prints as `ဒေတာ(<value>)`, or as `အလွတ်` when it is blank. This is the same in
`ပြသမည်`, in a `{...}` slot, in a list and in a struct.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အလွတ် ဖြစ်ပါစေ။
ခ ကို ပြသမည်။
ခ ကို ၇ ဟု ထားမည်။
ခ ကို ပြသမည်။
"{ခ}" ဟု ပြသမည်။
ကိန်းရှင် ဂဏန်းများ သည် ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [] ဖြစ်ပါစေ။
ဂဏန်းများ ထဲသို့ ၅ ကို ထည့်မည်။
ဂဏန်းများ ထဲသို့ အလွတ် ကို ထည့်မည်။
ဂဏန်းများ ကို ပြသမည်။
ကိန်းရှင် နှစ်ကွက် သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
နှစ်ကွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အလွတ်
ဒေတာ(၇)
ဒေတာ(၇)
[ဒေတာ(၅)၊ အလွတ်]
[အလွတ်၊ အလွတ်]
```

## Testing

```text
<x> ထဲတွင် ဒေတာ ရှိ
<x> ထဲတွင် ဒေတာ မရှိ
```

We use the first test to ask whether the optional `x` contains data, and the second to ask
whether it is blank. A test is a true/false value (boolean), so we can use it wherever one
fits: in `အကယ်၍` and `ထိုသို့မဟုတ်ဘဲ`, in the condition of a loop, beside `ဖြစ်၍` and
`သို့မဟုတ်` (`&&` and `||`), and in a declaration.

We can also write these test words in other forms:

| Form | Meaning |
| --- | --- |
| `အချက်အလက်` in place of `ဒေတာ` | the same word in another spelling |
| `ရှိနေ`, `မရှိနေ` | "is there", "is not there"; the same tests |
| `ရှိလျှင်`, `မရှိနေလျှင်` | the test, with `လျှင်` joined to it to end an `အကယ်၍` condition |
| `ရှိနေသရွေ့`, `မရှိနေသရွေ့` | the test, with `သရွေ့` joined to it to end a loop condition |

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
ခ ထဲတွင် ဒေတာ ရှိနေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ခ ကို ပြသမည်။
အကယ်၍ ခ ထဲမှ ဒေတာ သည် ၁ နှင့် ညီလျှင် ခ ကို အလွတ် ဟု ထားမည်။
သို့မဟုတ်ပါက ခ ကို ခ ထဲမှ ဒေတာ - ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ခ ကို ပြသမည်။
အကယ်၍ ခ ထဲတွင် အချက်အလက် မရှိလျှင် "ကုန်ပြီ" ဟု ပြသမည်။
ကိန်းရှင် ပြီး သည် ခ ထဲတွင် ဒေတာ မရှိ ဖြစ်ပါစေ။
ပြီး ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဒေတာ(၃)
ဒေတာ(၂)
ဒေတာ(၁)
အလွတ်
ကုန်ပြီ
မှန်
```

An optional and a [result](reading-results.md#tests) are two different data kinds, so
each has its own set of tests. A test word of the other set is ZK0475: `ရလဒ်ကောင်း` or
`အမှားအယွင်း` on an optional, or `ဒေတာ` on a result.

```zaka-error ZK0475
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
အကယ်၍ ခ ထဲတွင် ရလဒ်ကောင်း ရှိလျှင် "ကောင်း" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0475]: "optional isize" is an optional: test it with "ထဲတွင် ဒေတာ ရှိ" or "ထဲတွင် ဒေတာ မရှိ"
```

## Reading

```text
<x> ထဲမှ ဒေတာ
```

We write `x ထဲမှ ဒေတာ` to get the value that the optional contains, as a value of the
data kind inside, and we can write `အချက်အလက်` in place of `ဒေတာ`. The read is a value
like other values, so we can use it as an operand, an argument or a condition. The program
must know that the cell has data before it reads it: a read of a blank optional stops the
program with the trap `the optional holds no data`, in the VM and in a native program
alike, so the program never reads a value that is not there.

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အလွတ် ဖြစ်ပါစေ။
"မစမီ" ဟု ပြသမည်။
ခ ထဲမှ ဒေတာ ကို ပြသမည်။
"ဤနေရာ မရောက်ပါ" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မစမီ
the program stopped: the optional holds no data
```

We use the same words to read the good result (`Ok` value) of a
[result](reading-results.md#reads). An optional has no error side, so
`<x> ထဲမှ အမှားအယွင်း ဒေတာ` on an optional is ZK0475.

## A fallback

```text
<x> ထဲမှ ဒေတာ၊ မရှိလျှင် <value>
```

We can use this form to read an optional with a fallback (`unwrap_or`): it is the data in
`x`, or `<value>` when `x` is blank ("if none, the value"). The program works out `<value>`
when `x` is blank and at no other time, and the value converts to the data kind of the
data. On a [result](reading-results.md), the fallback replaces the error value.

The fallback is one operand, with a prefix operator and its operand included, because a
read binds tighter than an operator. So `x ထဲမှ ဒေတာ၊ မရှိလျှင် ၀ + ၁` adds 1 to the data,
or to 0, and a clause with an operator goes in parentheses.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အလွတ် ဖြစ်ပါစေ။
က ထဲမှ ဒေတာ၊ မရှိလျှင် ၀ ကို ပြသမည်။
ခ ထဲမှ ဒေတာ၊ မရှိလျှင် ၀ ကို ပြသမည်။
ခ ထဲမှ ဒေတာ၊ မရှိလျှင် -၁ + ၁၀ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၇
၀
၉
```

A read of a row of a table (map) returns an optional, so the fallback is also how we fill
the gap of a missing row ([Tables](tables.md#reading-a-row)).

## A read a test ruled out

A test proves what an optional contains, for as long as nothing changes the variable. A
read of data that a test proved absent would stop the program each time it ran, so the
compiler reports it as ZK0454 and does not wait for the run.

```zaka-error ZK0454
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အလွတ် ဖြစ်ပါစေ။
အကယ်၍ ခ ထဲတွင် ဒေတာ မရှိလျှင် ခ ထဲမှ ဒေတာ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0454]: "ခ" holds no data here, so reading its data stops the run each time
```

The proofs, and what ends them, are the same as for a
[result](reading-results.md#a-part-the-test-ruled-out): the arm of an `အကယ်၍` and the arms
after it, `ဖြစ်၍`, `သို့မဟုတ်` and `မဟုတ်` in a condition, a guard that ends with
`ပြန်ပေးမည်`, `ရပ်မည်` or `ကျော်မည်`, and a loop condition all prove something, while an
assignment, a `ဒေတာ ပြောင်းလဲရန်` parameter or an address of the variable ends a proof.
When the compiler cannot tell what the optional contains, it reports nothing, and the
program checks the read when it reaches it, as [Reading](#reading) shows.

## Giving one back

```text
<value> ကို ပြန်ပေးမည်။
အလွတ်ကို ပြန်ပေးမည်။
```

When the result of a function is an optional kind, we use the first form to return a cell
that contains the value, and the second to return a blank cell. As in other functions with
a result, each path of the block must end with a return, or the compiler reports ZK0339
([Giving back a good value](error-handling.md#giving-back-a-good-value)).

An optional result is no failure, so a call of the function needs no pass-on, and `?` on it
is ZK0444. When a function must give the reason for a miss, it returns a
[result](error-handling.md) with an error side instead.

```zaka-error ZK0444
ရှာ ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
န * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

သုံး ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
ရှာ(၁)? ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0444]: the call passes a failure on, but "ရှာ" cannot fail
```

## NULL is a mistake

Tony Hoare added the null reference to ALGOL W in 1965, because it was cheap to build. In
a talk in 2009 he called it his ***billion-dollar mistake***: over decades, the errors,
crashes and openings for attackers that it caused cost that much or more.

The mistake is in what a NULL does to a data kind. When each pointer can be NULL, the C
type `char *` does not tell whether the pointer points at text or at nothing. Each read
through a pointer can crash, and nothing in the data kind shows which reads can. The
programmer must remember at each place whether a check came first, and a check that was
left out shows up as a crash far from its cause, or as a read of garbage.

The optional kind moves the question into the data kind:

- A plain `စာသား` contains text on each read. No test is needed, and none can be left out.
- A `စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` shows in its data kind that it can be blank, so
  the program must ask first, with `ထဲတွင် ဒေတာ ရှိ`.
- The compiler refuses a read of data that a test proved absent (ZK0454).
- A read that nothing proves safe and nothing proves wrong is still checked when the
  program runs: a read of a blank cell stops the program with the trap
  `the optional holds no data`, instead of reading garbage.

So safe Zaka has no NULL, and each data kind but the pointer has a value on each read:

- A shared object (reference type) and a function value contain an object or a function at
  each point. A variable of either data kind with no value is unset, and the compiler
  refuses a read of it (ZK0722, [Set before read](declarations.md#set-before-read)).
- An object or a function that can be missing is an optional of it, such as
  `ဘူး ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`, which we test and read like other optionals.
- A shared data kind that names itself in a field ends its chain with an optional field. A
  data kind whose fields lead back to it with no optional on the loop can never be built
  (ZK0481, [Data kinds no program can build](objects.md#data-kinds-no-program-can-build)).

The one NULL left is the null address (null pointer) of a pointer,
`မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` ("the address that points at nothing"), and it lives
where a program works at a low level: a read through a pointer needs [unsafe
code](pointers.md#the-null-address), and the check of unsafe code looks for the null address
there.

The optional is also how Zaka receives a NULL from C. When a C function can return NULL
for a text or handle result, we declare that result as an optional, and NULL arrives as a
blank optional. A plain handle result that is not optional promises that C returns no
NULL, and when C breaks the promise the program stops, as
[Functions](functions-c.md#data-kinds-from-outside-the-program) states.

## Errors

| Code | Mistake |
| --- | --- |
| ZK0339 | a function with an optional result whose block can reach its end with no return |
| ZK0444 | a pass-on, `?`, for a call that returns an optional: an optional is no failure |
| ZK0454 | a read of data that a test proved absent |
| ZK0475 | a test word of the other set: `ရလဒ်ကောင်း` or `အမှားအယွင်း` on an optional, or `ဒေတာ` on a result |
| ZK0477 | `အလွတ်` where no optional kind is wanted |

```zaka-error ZK0477
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အလွတ် ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0477]: "အလွတ်" goes where an optional kind is wanted
```
