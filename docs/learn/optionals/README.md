---
title: Optional values
description: A data cell that might contain a value, how to test it before reading it, and the reason Zaka has it in place of a NULL.
---

# Optional values

Some values are there at times and missing at others: a search can find nothing, and a form
can leave a field out. In Zaka, we keep such a value in an *optional kind* (option type,
nullable type), which is a data cell that might contain a value, or might be blank (none,
null).

## You will learn

- how to write an optional kind, and return a value or a blank from a function
- how to test an optional before we read it
- what happens when we read a blank optional
- when to use an optional and when to use a result, and the reason Zaka has no NULL

```zaka
အမည်ရှာ ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၁ နှင့် ညီလျှင် "မောင်" ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၂ နှင့် ညီလျှင် "မ" ကို ပြန်ပေးမည်။
အလွတ်ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

အမည်ရှာ(၁) ကို ပြသမည်။
အမည်ရှာ(၉) ကို ပြသမည်။
```

```text
ဒေတာ("မောင်")
အလွတ်
```

The function `အမည်ရှာ` finds a name for `၁` and `၂`, and nothing for other numbers. We write
its result as an optional kind:

```text
<kind> ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်
```

| Zaka | Reads as |
| --- | --- |
| `စာသား` | the data kind of the value, text here |
| `ရှိကောင်းရှိနိုင်သော` | "that might contain" |
| `ဒေတာဆဲလ်` | "data cell" |

To return a value, we return it as it is, such as `"မောင်" ကို ပြန်ပေးမည်။`, and to return
nothing, we write `အလွတ်ကို ပြန်ပေးမည်။`, where `အလွတ်` means "blank". An optional prints as
`ဒေတာ(<value>)`, or as `အလွတ်` when it is blank.

## Testing before reading

```text
<optional> ထဲတွင် ဒေတာ ရှိလျှင်
<optional> ထဲတွင် ဒေတာ မရှိလျှင်
<optional> ထဲမှ ဒေတာ
```

We cannot use the text in an optional until we know that the text is there. So we test the
optional with `ထဲတွင် ဒေတာ ရှိလျှင်` ("if there is data inside") or `ထဲတွင် ဒေတာ မရှိလျှင်`
("if there is no data inside"), and then we read the value with `ထဲမှ ဒေတာ` ("the data
from inside"):

```zaka
အမည်ရှာ ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၁ နှင့် ညီလျှင် "မောင်" ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၂ နှင့် ညီလျှင် "မ" ကို ပြန်ပေးမည်။
အလွတ်ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ရှာတွေ့ချက် သည် အမည်ရှာ(၁) ဖြစ်ပါစေ။
အကယ်၍ ရှာတွေ့ချက် ထဲတွင် ဒေတာ ရှိလျှင် "တွေ့ပြီ: {ရှာတွေ့ချက် ထဲမှ ဒေတာ}" ဟု ပြသမည်။
သို့မဟုတ်ပါက "မတွေ့ပါ" ဟု ပြသမည်။
ရှာတွေ့ချက် ကို အမည်ရှာ(၉) ဟု ထားမည်။
အကယ်၍ ရှာတွေ့ချက် ထဲတွင် ဒေတာ မရှိလျှင် "မတွေ့ပါ" ဟု ပြသမည်။
```

```text
တွေ့ပြီ: မောင်
မတွေ့ပါ
```

| Zaka | What it does |
| --- | --- |
| `ရှာတွေ့ချက် ထဲတွင် ဒေတာ ရှိလျှင်` | true when `ရှာတွေ့ချက်` contains data |
| `ရှာတွေ့ချက် ထဲတွင် ဒေတာ မရှိလျှင်` | true when `ရှာတွေ့ချက်` is blank |
| `ရှာတွေ့ချက် ထဲမှ ဒေတာ` | the value inside `ရှာတွေ့ချက်` |

We can also write `ရှိနေလျှင်` in place of `ရှိလျှင်`, with the same meaning, and
`အချက်အလက်` in place of `ဒေတာ`.

## Changing an optional

```text
ကိန်းရှင် <name> သည် <kind> ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် <value> ဖြစ်ပါစေ။
```

An optional variable gets its first value as other variables do. The value can be a plain
value, `အလွတ်`, or `ကနဦးတန်ဖိုး`, which for an optional is `အလွတ်`. Later, `ထားမည်` puts a
plain value into the cell as it is, and `အလွတ်` makes it blank again. A loop can test the
optional too: `ခ ထဲတွင် ဒေတာ ရှိနေသရွေ့` repeats while the cell contains data.

```zaka
ကိန်းရှင် ခ သည် ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
ခ ထဲတွင် ဒေတာ ရှိနေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ခ ကို ပြသမည်။
အကယ်၍ ခ ထဲမှ ဒေတာ သည် ၁ နှင့် ညီလျှင် ခ ကို အလွတ် ဟု ထားမည်။
သို့မဟုတ်ပါက ခ ကို ခ ထဲမှ ဒေတာ - ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ခ ကို ပြသမည်။
```

```text
ဒေတာ(၃)
ဒေတာ(၂)
ဒေတာ(၁)
အလွတ်
```

## A read with no test

When we read a blank optional without a test, the program does not read garbage. It stops
and tells the reason:

```zaka-stop
ကိန်းရှင် ခ သည် ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
"မစမီ" ဟု ပြသမည်။
ခ ထဲမှ ဒေတာ ကို ပြသမည်။
"ဤနေရာ မရောက်ပါ" ဟု ပြသမည်။
```

```text
မစမီ
the program stopped: the optional holds no data
```

When a test has told the compiler that the cell is blank, the compiler does not wait for
the run. Inside the arm of `ခ ထဲတွင် ဒေတာ မရှိလျှင်`, a read of `ခ` would stop the program
each time, so it is the mistake ZK0454:

```zaka-error ZK0454
ကိန်းရှင် ခ သည် ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
အကယ်၍ ခ ထဲတွင် ဒေတာ မရှိလျှင် ခ ထဲမှ ဒေတာ ကို ပြသမည်။
```

```text
error[ZK0454]: "ခ" holds no data here, so reading its data stops the run each time
```

## An optional and a result

A blank optional gives no reason for being blank. When the caller needs to know the reason,
we use a [result](../error-handling/kinds.md) instead, because its error value (error) can
tell the reason. The two differ like this:

| | Optional | Result |
| --- | --- | --- |
| Written | `စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` | `ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ်` |
| When there is no value | `အလွတ်`, with no reason | an error value, which gives the reason |
| Test | `န ထဲတွင် ဒေတာ ရှိ` or `မရှိ` | `ရ ထဲတွင် ရလဒ်ကောင်း ရှိ` or `ရ ထဲတွင် အမှားအယွင်း ရှိ` |
| Read | `န ထဲမှ ဒေတာ` | `ရ ထဲမှ ဒေတာ`, or `ရ ထဲမှ အမှားအယွင်း ဒေတာ` for the error value |
| Pass on with `?` | no, because nothing failed (ZK0444) | the failure goes on to the caller |

The test words do not mix, so a test word of a result on an optional, or the reverse, is the mistake ZK0475. And `?`
does not pass a blank optional on, because a blank is not a failure, so `?` on a call that
returns an optional is ZK0444.

## NULL is a mistake

In 1965, Tony Hoare added the null reference to the language ALGOL W, because it was cheap
to build, and in a talk in 2009 he called it his ***billion-dollar mistake***. When each
pointer can be NULL, each read through a pointer can crash, and the data kind (type) of the
pointer does not show which reads are safe. A missing check then shows up far from the
place where it was left out.

Zaka puts the question into the data kind. A plain `စာသား` contains text on each read, so
there is nothing to forget. A `စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` shows that it can be blank,
so the program must test it before it reads it, and the compiler refuses a read that a test
has proved wrong. A read that gets past these checks stops the program, rather than reading
garbage.

Safe Zaka has no NULL at all. A [shared object](../structs/objects.md) (reference type) and a
[function value](../function-values/README.md) contain a value on each read; when one of them can
be missing, we make it an optional, as we did with text here. NULL remains in one place
alone: the address of a pointer in [unsafe Zaka](../pointers/README.md), where
`မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` ("an address that points at nothing") is the null address
(null pointer). The full rules are in [NULL is a
mistake](../../manual/language/optionals.md#null-is-a-mistake).

## Details

- We can give a fallback for a blank optional: `ခ ထဲမှ ဒေတာ၊ မရှိလျှင် ၀` is the data in
  `ခ`, or `၀` when `ခ` is blank ([A fallback](../../manual/language/optionals.md#a-fallback)).
- A variable declared with no value is not blank: it is unset, as other variables are, and
  we must give it a value before we read it ([The optional
  kind](../../manual/language/optionals.md#the-optional-kind)).
- `အလွတ်` where no optional is wanted, such as for a plain `ကိန်းပြည့်`, is the mistake ZK0477
  ([Errors](../../manual/language/optionals.md#errors)).
- A struct field, a parameter and the cells of a list can be optionals too, and a list of
  optionals is written with `စာရင်း` after `ဒေတာဆဲလ်` ([The optional
  kind](../../manual/language/optionals.md#the-optional-kind)).
- A function with an optional result must end each path with a return, or the compiler
  reports ZK0339 ([Giving one back](../../manual/language/optionals.md#giving-one-back)).

## Recap

- We write `<kind> ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` for a value that might be missing, and
  `အလွတ်` for the blank value.
- We test an optional with `ထဲတွင် ဒေတာ ရှိလျှင်` or `မရှိလျှင်`, and read it with `ထဲမှ ဒေတာ`.
- Reading a blank optional stops the program, or is a compile error when a test rules it out.
- We use an optional when "nothing" needs no reason, and a result when the caller needs the
  reason.

## Hands on

- Write a function that returns half of a number as an optional, and `အလွတ်` for an odd
  number. Print both cases with a test in `အကယ်၍`.
- Write a function that looks a number up in a list and returns an optional: the cell number
  of the number, or `အလွတ်`. Print both cases with a test in `အကယ်၍`.
- Keep three optionals in a list of optionals, and print the data of each cell that has
  some.
- Change the program of "A read with no test" so that it tests first, and check that the
  program no longer stops.

Next: [Lists](../lists/README.md).
