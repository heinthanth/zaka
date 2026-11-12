---
title: Optional values
description: A data cell that might contain a value, how to test it before reading it, and the reason Zaka has it in place of a NULL.
---

# Optional values

A search can find nothing, and a form can leave a field out. Such a value is there at
times and not at others. Zaka writes it as an *optional*: a data cell that might contain a
value, and its kind declares so. An optional is a kind of its own, with its own tests and
reads; it tells nothing about the reason the cell holds nothing.

A [result](../error-handling/kinds.md) is the other kind that a call can give back when it
finds no value, and the two differ in what each holds:

| | Optional | Result |
| --- | --- | --- |
| Written | `စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` | `ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ်` |
| Holds, when no value | `အလွတ်`, with no reason | an error value, which gives the reason |
| Test | `န ထဲတွင် ဒေတာ ရှိ` or `မရှိ` | `ရ ထဲတွင် ရလဒ်ကောင်း ရှိ` or `ရ ထဲတွင် အမှားအယွင်း ရှိ` |
| Read | `န ထဲမှ ဒေတာ` | `ရ ထဲမှ ဒေတာ`, or `ရ ထဲမှ အမှားအယွင်း ဒေတာ` for the error |
| Passed on with `?` | no, since nothing failed (ZK0444) | a failure goes on to the caller |

## A function that might find nothing

The kind of an optional is the kind of the value, then `ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`, which
reads as "a data cell that might contain" a value of that kind:
`စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` is a data cell that might contain text. `အလွတ်` is the cell with
nothing in it. A plain value is given back as it is:

```zaka
လုပ်ဆောင်ချက် အမည်ရှာ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၁ နှင့် ညီလျှင် "မောင်" ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၂ နှင့် ညီလျှင် "မ" ကို ပြန်ပေးမည်။
အလွတ်ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အမည်ရှာ(၁) ကို ပြသမည်။
အမည်ရှာ(၉) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဒေတာ("မောင်")
အလွတ်
```

An optional prints as `ဒေတာ(<value>)`, or as `အလွတ်` when it holds nothing. The header
ends with `တစ်ခု ကို ပြန်ပေးမည်`, as for each function with a result.

## Asking before reading

A program cannot use the text in an optional until it knows the text is there. Two tests ask,
and a read takes the value out:

| Written | Is |
| --- | --- |
| `န ထဲတွင် ဒေတာ ရှိ` | true when `န` holds data |
| `န ထဲတွင် ဒေတာ မရှိ` | true when `န` holds nothing |
| `န ထဲမှ ဒေတာ` | the value that `န` holds |

`ရှိ` can take `နေ` and `လျှင်` (`ရှိနေလျှင်`), so a test reads as a sentence. `အချက်အလက်`
can stand in place of `ဒေတာ`.

```zaka
လုပ်ဆောင်ချက် အမည်ရှာ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၁ နှင့် ညီလျှင် "မောင်" ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၂ နှင့် ညီလျှင် "မ" ကို ပြန်ပေးမည်။
အလွတ်ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရှာတွေ့ချက် သည် အမည်ရှာ(၁) ဖြစ်ပါစေ။
အကယ်၍ ရှာတွေ့ချက် ထဲတွင် ဒေတာ ရှိလျှင် "တွေ့ပြီ: {ရှာတွေ့ချက် ထဲမှ ဒေတာ}" ဟု ပြသမည်။
သို့မဟုတ်ပါက "မတွေ့ပါ" ဟု ပြသမည်။
ရှာတွေ့ချက် ကို အမည်ရှာ(၉) ဟု ထားမည်။
အကယ်၍ ရှာတွေ့ချက် ထဲတွင် ဒေတာ မရှိလျှင် "မတွေ့ပါ" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
တွေ့ပြီ: မောင်
မတွေ့ပါ
```

An optional variable gets its first value as each variable does: `အလွတ်`, a plain value,
or `ကနဦးတန်ဖိုး`, which for an optional is `အလွတ်`. A plain value stored into it with
`ထားမည်` goes in as it is, and `အလွတ်` empties it again. A loop can ask too:
`ရှာတွေ့ချက် ထဲတွင် ဒေတာ ရှိနေသရွေ့` goes round while the cell holds data.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
ခ ထဲတွင် ဒေတာ ရှိနေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ခ ကို ပြသမည်။
အကယ်၍ ခ ထဲမှ ဒေတာ သည် ၁ နှင့် ညီလျှင် ခ ကို အလွတ် ဟု ထားမည်။
သို့မဟုတ်ပါက ခ ကို ခ ထဲမှ ဒေတာ - ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ခ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဒေတာ(၃)
ဒေတာ(၂)
ဒေတာ(၁)
အလွတ်
```

## A read with no test

A program that reads without asking does not read garbage: the run stops, with the reason.

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
"မစမီ" ဟု ပြသမည်။
ခ ထဲမှ ဒေတာ ကို ပြသမည်။
"ဤနေရာ မရောက်ပါ" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မစမီ
the program stopped: the optional holds no data
```

When a test has told that the cell holds nothing, the compiler does not wait for the
run. Inside the arm of `ခ ထဲတွင် ဒေတာ မရှိလျှင်`, a read of `ခ` would stop the run each
time, so it is the error ZK0454:

```zaka-error ZK0454
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
အကယ်၍ ခ ထဲတွင် ဒေတာ မရှိလျှင် ခ ထဲမှ ဒေတာ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0454]: "ခ" holds no data here, so reading its data stops the run each time
```

## An optional and a result

An optional has no error that gives a reason for holding nothing. When the caller has to know the reason,
use a [result](../error-handling/kinds.md) instead: its kind names the error, and its tests are
`ထဲတွင် ရလဒ်ကောင်း ရှိ` and `ထဲတွင် အမှားအယွင်း ရှိ`. The two sets of words do not mix: a test
of the other kind is ZK0475. Nor does `?` pass an optional on, since nothing failed: it is
ZK0444.

## NULL is a mistake

In 1965 Tony Hoare added the null reference to the language ALGOL W, because it was cheap to
build. In a talk in 2009 he called it his ***billion-dollar mistake***. A NULL that each pointer
can hold means that each read through a pointer can crash, and the kind of the pointer does
not tell which reads. The check that was left out shows up far from where it was left out.

Zaka puts the question into the kind. A plain `စာသား` holds text on each read, so nothing is left
to forget. A `စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` declares that it can hold nothing, so the
program has to test it before it reads, and the check refuses a read that a test has proved
wrong. A read that slips through stops the run with a trap, in place of reading garbage.

Safe Zaka holds no NULL at all. A [shared object](../structs/objects.md) and a [function
value](../function-values/README.md) hold a value on each read; one that can be missing is
an optional, as text is here. NULL remains in one place alone: the address of a pointer in
[unsafe Zaka](../pointers/README.md), where `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` ("an address
that points at nothing") is the null address. The rules in full are in [NULL is a
mistake](../../manual/language/optionals.md#null-is-a-mistake).

## Hands on

- Write a function that gives back half of a number as an optional, and `အလွတ်` for an odd
  number. Print both cases with a test in `အကယ်၍`.
- Write a function that looks a number up in a list and gives back an optional: the index of
  the number, or `အလွတ်`. Print both cases with a test in `အကယ်၍`.
- Keep three optionals in a list of optionals, and print the data of each cell that holds
  some.
- Change the program of "A read with no test" so that it asks first, and check that the
  run no longer stops.

Next: [Lists](../lists/README.md).
