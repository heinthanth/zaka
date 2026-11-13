---
title: Optional values
description: The optional kind, a data cell that might contain a value: how it is declared, tested and read, how a function gives one back, and the reason Zaka has it in place of a NULL.
---

# Optional values

Some values are there at times and not at others: the age of a person who left it out, the
first match of a search that found none. Zaka does not give such a value a NULL that each pointer can
hold. It gives the *optional kind*: a data cell that might contain a value, which the kind
declares. The program must test the cell before it reads the value in
it. This page states the rules in full. The Learn book teaches the kind in
[Optional values](../../learn/optionals/README.md).

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု နှင့် အသက် ဟူသော ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု တို့ပါဝင်သည်။

လုပ်ဆောင်ချက် ရှာ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၁ နှင့် ညီလျှင် "တစ်" ကို ပြန်ပေးမည်။
အလွတ်ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပြ သည်
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

reads as "a data cell that might contain" a value of the kind. It can be written wherever a kind
goes: in a declaration, a parameter, a field of a struct and the result clause of a
function. In a declaration and a result clause `တစ်ခု` follows it, as it follows each
kind (the measure word has to fit the kind, ZK0509).

| Written | Is |
| --- | --- |
| `ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` | an integer, or nothing |
| `စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` | text, or nothing |
| `ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် စာရင်း` | a plain list of optionals |
| `အကွက် ၂ ခုရှိသော ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် စာရင်း` | a list of 2 optionals |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` | a list of 3 integers, or nothing |
| `ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် လိပ်စာ` | a pointer to an optional, for [unsafe code](unsafe.md) |

List and pointer levels come after `ဒေတာဆဲလ်`, and wrap the whole optional, as the levels of a
[result kind](error-handling.md#result-kinds) wrap the whole result. Counts written before the
kind go to its lists innermost first, as in nested lists: first to a list of the data, then
to the levels after `ဒေတာဆဲလ်`. A list literal fills the data of an optional.
In a message the kind reads like
`optional isize`.

Two words make a value of the kind:

- `အလွတ်` ("blank data") is the value that holds nothing. It stands wherever an optional
  is wanted: the value of a declaration, a store, an argument, a return, a field of a struct literal,
  a cell to add to a list. Where no optional is wanted, it is ZK0477.
- A plain value of the kind inside goes in as it is. A `ကိန်းပြည့်` fits an optional of
  `ကိန်းပြည့်`, with no word around it: the cell holds it from then on. The same holds for
  an argument, a store with `ထားမည်`, a field and a return.

A literal takes the kind inside the optional, so `၁` is an `၈ ဘစ် ကိန်းပြည့်` where an optional of
that kind is wanted; a literal outside that kind is ZK0406, as it is for a plain variable.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ၈ ဘစ် ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
အသက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဒေတာ(၁)
```

The [initial value](kinds.md#initial-values) of an optional, `ကနဦးတန်ဖိုး`, is `အလွတ်`, so
`ကနဦးတန်ဖိုး` fills a counted list of optionals with blank cells. A declaration with no value
holds no `အလွတ်`: it is unset, as a variable of each other kind is, and the program gives
it a value before it reads it ([Set before read](statements.md#set-before-read)).

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

An optional prints as `ဒေတာ(<value>)`, or as `အလွတ်` when it holds nothing, whether it
stands in `ပြသမည်`, in a `{...}` slot, in a list or in a struct.

## Testing

A test tells whether the optional holds data. It is a true or false value, so it stands
wherever one can: in `အကယ်၍` and `ထိုသို့မဟုတ်ဘဲ`, in the condition of a loop, beside `ဖြစ်၍`
and `သို့မဟုတ်` (`&&` and `||`), and in a declaration.

| Written | Is true when |
| --- | --- |
| `<x> ထဲတွင် ဒေတာ ရှိ` | `x` holds data |
| `<x> ထဲတွင် ဒေတာ မရှိ` | `x` holds nothing |

`အချက်အလက်` is another spelling of `ဒေတာ`, and can stand in its place. `ရှိ` and
`မရှိ` can take `နေ` (`ရှိနေ`, `မရှိနေ`) to read as "is there" and "is not there", and `လျှင်`
or `သရွေ့` can follow either fused, which ends an `အကယ်၍` condition or a loop condition:
`ရှိလျှင်`, `မရှိနေလျှင်`, `ရှိနေသရွေ့`, `မရှိနေသရွေ့`.

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

The tests of an optional and the tests of a [result](error-handling.md#reading-a-whole-result) are two sets,
as the two kinds are two. A test word of the other kind is ZK0475: `ရလဒ်ကောင်း` or `အမှားအယွင်း`
on an optional, and `ဒေတာ` on a result.

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

is the value the optional holds, of the kind inside. `အချက်အလက်` can stand in place of `ဒေတာ`.
The read is a value like each other value, so it can be an operand, an argument or a
condition. The program has to know the cell holds data before it reads: a read of an optional
that holds nothing stops the run with the trap `the optional holds no data`, in the VM and in a native
program alike, so the program never reads a value that is not there.

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

The read of a [result](error-handling.md#reading-a-whole-result) has the same spelling, and it reads the good
value. An optional has no error side, so `<x> ထဲမှ အမှားအယွင်း ဒေတာ` on one is ZK0475.

### A fallback

```text
<x> ထဲမှ ဒေတာ၊ မရှိလျှင် <value>
```

is the data the optional holds, or the value when it holds none ("if none, the value"). The
value runs when the optional holds no data, and at no other time, and it converts to the kind
of the data. It is one operand, a prefix operator and its operand included, as a read binds
tighter than an operator: `x ထဲမှ ဒေတာ၊ မရှိလျှင် ၀ + ၁` adds 1 to the data or to 0, and a
clause goes in parentheses. On a result, the fallback stands in for the error.

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

A ဇယား gives an optional for each row it reads, so the fallback fills the gap of a missing row
([Tables](tables.md#reading-a-row)).

### A read a test ruled out

A test proves what an optional holds, for as long as nothing changes the variable. A read of
the data that a test proved absent would stop the run each time it ran, so the compiler
reports it as ZK0454 and does not wait for the run. The proofs and what ends them are those
of a [result](error-handling.md#reading-a-whole-result): the arms of `အကယ်၍` and of the
arms after it, `ဖြစ်၍`, `သို့မဟုတ်`, `မဟုတ်`, a guard that ends with `ပြန်ပေးမည်`, `ရပ်မည်` or
`ကျော်မည်`, a loop condition, and an assignment, a `ဒေတာ ပြောင်းလဲရန်` parameter or an address that
ends a proof.

```zaka-error ZK0454
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အလွတ် ဖြစ်ပါစေ။
အကယ်၍ ခ ထဲတွင် ဒေတာ မရှိလျှင် ခ ထဲမှ ဒေတာ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0454]: "ခ" holds no data here, so reading its data stops the run each time
```

When the compiler cannot tell, it reports nothing, and the run checks the read when it
reaches it, as above.

## Giving one back

A function whose result clause is an optional kind gives back a plain value, or the blank
value:

```text
<value> ကို ပြန်ပေးမည်။
အလွတ်ကို ပြန်ပေးမည်။
```

The first gives the cell that holds the value. The second gives the cell that holds
nothing. As for each function with a result, each path of the block ends with a return, or
the compiler reports ZK0339 (see [Giving back a good value](error-handling.md#giving-back-a-good-value)).
The result kind with an optional in it is no failure: a call of the function needs no
pass-on, and `?` on it is ZK0444.

```zaka-error ZK0444
လုပ်ဆောင်ချက် ရှာ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
န * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် သုံး သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
ရှာ(၁)? ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0444]: the call passes a failure on, but "ရှာ" cannot fail
```

A function that has to give the reason for a miss gives back a [result](error-handling.md)
with an error side instead.

## NULL is a mistake

Tony Hoare put the null reference into ALGOL W in 1965, because it was cheap to build. In his
talk of 2009 he called it his ***billion-dollar mistake***: the errors, crashes and open doors
for attackers that it caused, over decades, cost that much or more.

The mistake is in what a NULL does to a kind. When each pointer can hold NULL, the kind
`char *` does not tell whether the pointer points at text or at nothing. Each read through a
pointer can crash, and nothing in the kind tells which reads. The programmer has to
remember, at each place, whether a check came before, and a check left out shows up as a
crash far from its cause, or as a read of garbage.

The optional kind moves the question into the kind:

- A plain `စာသား` holds text on each read. No test is needed, and none can be left out.
- A `စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` declares in its kind that it can hold nothing, so the
  program has to ask first, with `ထဲတွင် ဒေတာ ရှိ`.
- The check refuses the read of data that a test proved absent (ZK0454).
- A read that nothing proves safe and nothing proves wrong is still checked at run time: a
  read of a blank cell stops the run with the trap `the optional holds no data`, in place of
  reading garbage.

So safe Zaka holds no NULL. Each kind but the pointer holds a value on each read:

- A shared object and a function value hold an object or a function at each point. A variable of
  either kind with no value is unset, and the check refuses a read of it (ZK0722,
  [Set before read](statements.md#set-before-read)).
- An object or a function that can be missing is an optional of it, as
  `ဘူး ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`, tested and read like each other optional.
- A shared kind that names itself in a field ends its chain with an optional field; a kind
  whose fields lead back to it with no optional on the loop can never be built (ZK0481,
  [Kinds no program can build](objects.md#kinds-no-program-can-build)).

The one NULL left is the null address of a pointer, `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`
("the address that points at nothing"), and it lives where a program works at a low level:
reading through a pointer needs [unsafe code](unsafe.md#the-null-address), and the check of
unsafe code looks for the null address there.

The optional is also what Zaka gives for a NULL that C hands back. A C function whose text or
handle result can be NULL declares an optional, and NULL reads as the optional with no data.
A plain, non-optional handle result promises that C gives no NULL: when C breaks the promise,
the run stops, as [Functions](functions.md#c-functions) states.

## Errors

| Code | Mistake |
| --- | --- |
| ZK0444 | a pass-on, `?`, for a call that gives back an optional: an optional is no failure |
| ZK0454 | a read of data that a test proved absent |
| ZK0475 | a test word of the other kind: `ရလဒ်ကောင်း` or `အမှားအယွင်း` on an optional, or `ဒေတာ` on a result |
| ZK0477 | `အလွတ်` where no optional kind is wanted |
| ZK0339 | a function with an optional result whose block can reach its end with no return |

```zaka-error ZK0477
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အလွတ် ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0477]: "အလွတ်" goes where an optional kind is wanted
```
