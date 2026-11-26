---
title: Struct mistakes
description: The common mistakes with structs, the message Zaka shows for each one, and the fix.
---

# Struct mistakes

The compiler catches these common mistakes with structs (records) before the program
runs. Each example shows the code, the message from the compiler, and how to fix the
error.

## You will learn

- where a struct can be declared, and which names it can share
- the mistakes in reading a field and in writing a struct literal
- what a struct cannot contain and cannot do
- what happens when we use the wrong measure word (classifier)

## A struct declared inside a block

```zaka-error ZK0228
အကယ်၍ မှန် ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
error[ZK0228]: structs can be declared at the top level and nowhere else
```

We can declare a struct at the top level of a file alone, and not inside a block, a loop,
a conditional or a function. The fix is to move the declaration out of the block, above
the program start block.

## One name for two structs

```zaka-error ZK0316
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။
လူ ဟူသော ဒေတာအစုအဝေးတွင် အရပ် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။
```

```text
error[ZK0316]: "လူ" names a struct, an enumeration or a kind alias declared before
```

Two structs cannot have the same name, and neither can a struct and an enumeration or a
kind alias. The fix is to give the second struct a different name, or to merge the fields
into one struct.

## One name for two fields

```zaka-error ZK0317
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် အသက် ဟူသော ဒဿမကိန်း တစ်ခု တို့ပါဝင်သည်။
```

```text
error[ZK0317]: "လူ" names the field "အသက်" twice
```

We reach a field through its name, so two fields of the same struct need two different
names.

## A name that is no struct

```zaka-error ZK0318
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ကိန်းရှင် က သည် လုပ် တစ်ခု ဖြစ်ပါစေ။
```

```text
error[ZK0318]: no struct or enumeration named "လုပ်" is declared
```

The data kind (type) `လုပ်` is not declared at all, because the struct is named `လူ`.
The fix is to check the spelling of the name against the declaration.

## A struct name that is also a value name

```zaka-error ZK0319
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ကိန်းရှင် လူ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
```

```text
error[ZK0319]: "လူ" names a struct or an enumeration, so it cannot name a value or a function too
```

We can use a struct name in places where a value is expected, as in `လူ တစ်ခု ၏
အရွယ်အစား`, so the same name cannot also name a variable, a constant, a parameter, a field
or a function. The fix is to pick a different name for the variable.

## A field the struct does not have

```zaka-error ZK0320
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်ပါစေ။
ကျော် ၏ အရပ် ကို ပြသမည်။
```

```text
error[ZK0320]: "လူ" has no field named "အရပ်"
```

`လူ` has the field `အသက်` alone. The fix is to use a field from the declaration, or to add
the field to the struct. A struct literal that names a missing field gives the same error.

## One field set twice

```zaka-error ZK0321
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အသက် အနေဖြင့် ၂၀ နှင့် အသက် အနေဖြင့် ၃၀ ရှိသော လူတစ်ခု ဖြစ်ပါစေ။
```

```text
error[ZK0321]: the "လူ" literal sets the field "အသက်" twice
```

A struct literal gives each field one value, so the fix is to remove one of the two
values.

## A field left out

```zaka-error ZK0482
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် အမြင့် ဟူသော ဒဿမကိန်း တစ်ခု တို့ပါဝင်သည်။

ကိန်းရှင် ကျော် သည် လူ(အသက်: ၂၀) ဖြစ်ပါစေ။
ကျော် ကို ပြသမည်။
```

```text
error[ZK0482]: "လူ" needs a value for "အမြင့်"
```

A struct literal needs a value for each field, and Zaka does not fill in a missing field.
The fix is to give the field a value, or `ကနဦးတန်ဖိုး` for its initial value (default
value): `လူ(အသက်: ၂၀၊ အမြင့်: ကနဦးတန်ဖိုး)`.

## A struct read before each field is set

```zaka-error ZK0722
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် အမြင့် ဟူသော ဒဿမကိန်း တစ်ခု တို့ပါဝင်သည်။

ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်ပါစေ။
ကျော် ၏ အသက် ကို ၂၀ ဟု ထားမည်။
ကျော် ကို ပြသမည်။
```

```text
error[ZK0722]: this read can happen before "ကျော်" is set
```

A struct declared without a value counts as set once all of its fields are set. Here
`အမြင့်` has no value when the program prints the whole struct. The fix is to set
`ကျော် ၏ အမြင့်` too before the read, or to declare the struct with a value such as
`ကနဦးတန်ဖိုး`. The full rules are in [Set before
read](../../manual/language/declarations.md#set-before-read).

## A field of a value that is no struct

```zaka-error ZK0426
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
က ၏ အသက် ကို ပြသမည်။
```

```text
error[ZK0426]: "အသက်" is read as a field, but the value before "၏" is isize, not a struct
```

`က` is an integer, and an integer has no fields. The fix is to declare `က` as a struct
that has the field `အသက်`.

## A struct that contains itself

```zaka-error ZK0427
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် သူ ဟူသော လူ တစ်ခု တို့ပါဝင်သည်။
```

```text
error[ZK0427]: "လူ" holds itself, so it would have no end
```

A field is stored inside its struct, so a struct that contains itself would need to be
larger than itself. To link one struct to another of the same data kind, Part III uses
an address, in [A linked list](../pointers/structs.md#a-linked-list), and a [shared
object](objects.md#an-object-that-can-be-missing) can do it with an optional field.

## Ordering structs

```zaka-error ZK0403
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ကိန်းရှင် က သည် လူ(အသက်: ၁၀) ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် လူ(အသက်: ၁၂) ဖြစ်ပါစေ။
က < ခ ကို ပြသမည်။
```

```text
error[ZK0403]: < needs a number or a character, found လူ
```

We can compare two structs with `==` and `!=`, but `<`, `>` and the other order operators
do not work on structs, because Zaka cannot know which field should decide the order. The
fix is to compare that field: `က ၏ အသက် < ခ ၏ အသက်`.

## The wrong measure word

```zaka-warning ZK0509
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်ပါစေ။
```

```text
warning[ZK0509]: လူ is counted with "ယောက်", not "ခု"
```

Burmese counts things with a measure word (classifier). A struct uses the measure word in
its declaration, or `ခု` when the declaration gives none, while `စာလုံး` uses `လုံး` and
each other data kind uses `ခု`. The wrong word gives a warning, and the program still runs
because the meaning is the same. The fix is to write `လူ တစ်ယောက်`.

We get the same warning when we use `ယောက်` with a data kind such as `ကိန်းပြည့်`:

```zaka-warning ZK0509
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ယောက် ဖြစ်ပါစေ။
```

```text
warning[ZK0509]: isize is counted with "ခု", not "ယောက်"
```

A struct literal uses the same measure word after `တစ်`, so `လူတစ်ခု` here gives the
warning too:

```zaka-warning ZK0509
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

ကိန်းရှင် ကျော် သည် လူ တစ်ယောက် ဖြစ်၍ တန်ဖိုးသည် အသက် အနေဖြင့် ၂၀ ရှိသော လူတစ်ခု ဖြစ်ပါစေ။
```

```text
warning[ZK0509]: လူ is counted with "ယောက်", not "ခု"
```

## Recap

- A struct is declared at the top level, with a name that no other struct, enumeration,
  kind alias or value uses.
- A struct literal gives each field one value, and `ကနဦးတန်ဖိုး` fills a field with its
  initial value.
- Structs compare with `==` and `!=` alone.
- A struct cannot contain itself.
- The wrong measure word gives a warning, not an error.

## Hands on

- Declare a struct with a field `အသက်`, then read a field `အရပ်` and read the message.
- Give a struct the measure word `ယောက်`, declare a variable of it with `တစ်ခု`, and
  compare the warning with the ones above.

Next: [Shared objects](objects.md).
