---
title: Variables
description: Give a value a name with ကိန်းရှင် ... ဖြစ်ပါစေ။
---

# Variables

A *variable* (mutable variable) stores a value under a name. To create a variable, we
write a *declaration* that gives it a name and a data kind (type). We can give it a
value in the declaration or assign one before we first read it.

## You will learn

- How to declare a variable with `ကိန်းရှင်` and give it a value.
- How to write a data kind with its measure word (classifier), as in `ကိန်းပြည့် တစ်ခု`.
- How to infer the data kind from the value, and how to start from the initial value
  `ကနဦးတန်ဖိုး`.
- That a variable must get a value before we read it.

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။
အသက် ကို ပြသမည်။
```

```text
၁၈
```

The first line declares a variable `အသက်` (age), an integer with the value `၁၈`, and the
second line prints it. To declare a variable, we write `ကိန်းရှင်`, the name, the data kind
and the value:

```
ကိန်းရှင် <name> သည် <kind> တစ်ခု ဖြစ်၍ တန်ဖိုးသည် <value> ဖြစ်ပါစေ။
```

Word for word, the line means "let the variable age be one integer, and its value be 18":

| Zaka | Reads as | Part |
| --- | --- | --- |
| `ကိန်းရှင်` | "variable" | starts the declaration |
| `အသက်` | "age" | the name we choose |
| `သည်` | (the subject) | |
| `ကိန်းပြည့်` | "integer" | the data kind of the value |
| `တစ်ခု` | "one" | counts the data kind, as Burmese counts a thing |
| `ဖြစ်၍` | "is, and" | joins the data kind and the value |
| `တန်ဖိုးသည်` | "its value" | |
| `၁၈` | 18 | the value |
| `ဖြစ်ပါစေ` | "let it be" | the verb of the declaration |
| `။` | full stop | |

A variable keeps its data kind for its whole life, so a `ကိန်းပြည့်` variable
contains an integer and nothing else.

## Using variables

Once we declare a variable, we can use its name in place of its value, even in the value
of another variable:

```zaka
// "Let the variable price be one decimal number, and its value be 2.5."
ကိန်းရှင် ဈေး သည် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂.၅ ဖြစ်ပါစေ။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။

// The value can be worked out from other variables.
ကိန်းရှင် နှစ်ဆ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အသက် * ၂ ဖြစ်ပါစေ။

ဈေး ကို ပြသမည်။
နှစ်ဆ ကို ပြသမည်။
```

```text
၂.၅
၃၆
```

## Measure words

```
<kind> တစ်<measure word>
```

Burmese pairs `တစ်` ("one") with a measure word (classifier), much as English uses
"one *sheet* of paper". Zaka uses the same pattern after a data kind, as in
`ကိန်းပြည့် တစ်ခု` ("one integer"). Most data kinds use `ခု`. The character data
kind `စာလုံး` uses `လုံး`.

```zaka
ကိန်းရှင် အက္ခရာ သည် စာလုံး တစ်လုံး ဖြစ်၍ တန်ဖိုးသည် 'က' ဖြစ်ပါစေ။
အက္ခရာ ကို ပြသမည်။
```

```text
က
```

When we leave out the measure word, the compiler stops, and it shows the words to add:

```zaka-error ZK0234
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
```

```text
error[ZK0234]: a kind here takes "တစ်" and a classifier
help: write "ကိန်းပြည့် တစ်ခု"
```

## Letting the value choose the data kind

```
ကိန်းရှင် <name> သည် <value> ဖြစ်ပါစေ။
```

When we write the value, we can leave the data kind out, and the variable gets the data
kind of its value. A whole number gives `ကိန်းပြည့်`, a number with a point gives
`ဒဿမကိန်း`, and text in quotes gives `စာသား`:

```zaka
ကိန်းရှင် အသက် သည် ၁၈ ဖြစ်ပါစေ။
ကိန်းရှင် ဈေး သည် ၂.၅ ဖြစ်ပါစေ။
ကိန်းရှင် အမည် သည် "မမ" ဖြစ်ပါစေ။
ကိန်းရှင် နှစ်ဆ သည် အသက် * ၂ ဖြစ်ပါစေ။
အမည် ကို ပြသမည်။
နှစ်ဆ ကို ပြသမည်။
```

```text
မမ
၃၆
```

We write the data kind when the value alone would choose another one, such as a small
integer kind:
`ကိန်းရှင် ဘိုက် သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀၀ ဖြစ်ပါစေ။`

## The initial value

```
ကိန်းရှင် <name> သည် <kind> တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
```

When we want a plain starting value, we can write `ကနဦးတန်ဖိုး` ("the initial value"), the
initial value (default value) of the data kind. The declaration must write the data kind,
because the data kind decides what the initial value is:

| Data kind | `ကနဦးတန်ဖိုး` is |
| --- | --- |
| integers and decimal numbers | `၀` (zero) |
| `မှန်မှားတန်ဖိုး` | `မမှန်` (false) |
| `စာလုံး` | the null character `'\0'` |
| `စာသား` | `""`, text with no characters |

```zaka
ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။ // count, 0
ကိန်းရှင် ပြီးပြီ သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။ // done, false
ကိန်းရှင် ပျမ်းမျှ သည် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။ // average, 0.0
အရေအတွက် ကို ပြသမည်။
ပြီးပြီ ကို ပြသမည်။
ပျမ်းမျှ ကို ပြသမည်။
```

```text
၀
မမှန်
၀.၀
```

Later pages give the initial value of lists, structs and the other data kinds.

## Set before read

```
ကိန်းရှင် <name> သည် <kind> တစ်ခု ဖြစ်ပါစေ။
```

We can also declare a variable with no value at all. The variable then has no value until
the program gives it one, and Zaka does not fill in a zero in secret, so a read before
then is a mistake:

```zaka-error ZK0722
ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
အရေအတွက် ကို ပြသမည်။
```

```text
error[ZK0722]: this read can happen before "အရေအတွက်" is set
```

When we set the variable first, with `ထားမည်` from [Changing a variable](assignment.md),
the read works:

```zaka
ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
အရေအတွက် ကို ၃ ဟု ထားမည်။
အရေအတွက် ကို ပြသမည်။
```

```text
၃
```

Once a program can choose between paths, with [If and else](../control-flow/if-else.md),
each path that leads to a read must set the variable before it.

## A value with its data kind

```
ကိန်းရှင် <name> သည် <value> ဟူသော <kind> တစ်ခု ဖြစ်ပါစေ။
```

We can write a number with its data kind after it, as `<value> ဟူသော <kind> တစ်ခု` ("the
value, which is a `<kind>`"), and a declaration with that value alone gets the same data
kind.
These two lines declare two variables of the same data kind with the same value:

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။
ကိန်းရှင် အရွယ် သည် ၁၈ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
အသက် ကို ပြသမည်။
အရွယ် ကို ပြသမည်။
```

```text
၁၈
၁၈
```

The first form keeps the data kind and the value apart, which is clearer when the value
is long, so these pages use it.

## The value must fit the data kind

The value must suit the data kind of the variable, so a decimal number cannot go into an
integer variable:

```zaka-error ZK0401
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈.၅ ဖြစ်ပါစေ။
```

```text
error[ZK0401]: expected isize, found f64
```

`isize` is the compiler's short name for plain `ကိန်းပြည့်`, and `f64` is the short name
for `ဒဿမကိန်း`. The [Error messages](../../manual/reference/errors.md) page lists all
these short names.

## Using a name before declaring it

A variable exists from its declaration onward, and not before, so a name that was never
declared is an error:

```zaka-error ZK0301
အသက် ကို ပြသမည်။
```

```text
error[ZK0301]: "အသက်" is not declared
```

## Details

- **The wrong measure word.** A measure word that does not fit the data kind, such as
  `စာလုံး တစ်ခု`, is the warning ZK0509, and the program still runs. Structs can choose
  their own measure word. See [Measure words](../../manual/language/kinds.md#measure-words).
- **No data kind to find.** `ကနဦးတန်ဖိုး` needs a data kind from its place, so
  `ကိန်းရှင် က သည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။` and `ကနဦးတန်ဖိုး ကို ပြသမည်။` are the error
  ZK0478. See [Initial values](../../manual/language/kinds.md#initial-values).
- **Other spellings of `ဖြစ်၍`.** `ဖြစ်ပြီး` and `ဖြစ်ကာ` mean the same as `ဖြစ်၍`. See
  [Second spellings](../../manual/language/words.md#second-spellings).
- **A value cannot use its own name.** The compiler checks the value before it adds the
  name, so `ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က + ၁ ဖြစ်ပါစေ။` is ZK0301. See
  [Declaring a name](../../manual/language/scope.md#declaring-a-name).
- **Set before read in full.** The rules for each path of a program are in [Set before
  read](../../manual/language/declarations.md#set-before-read).

## Recap

- We write `ကိန်းရှင် <name> သည် <kind> တစ်ခု ဖြစ်၍ တန်ဖိုးသည် <value> ဖြစ်ပါစေ။` to declare
  a variable.
- We count a data kind with `တစ်` and its measure word: `ကိန်းပြည့် တစ်ခု`,
  `စာလုံး တစ်လုံး`.
- We can leave the data kind out when the value shows it, and we can use `ကနဦးတန်ဖိုး`
  for the initial value of a data kind.
- A variable must get a value before the program reads it.

## Hands on

- Declare a variable for a height in centimetres and print it.
- Declare a `ဒဿမကိန်း` variable with no value, print it, and read the mistake. Then give it
  `ကနဦးတန်ဖိုး`.
- Declare a `မှန်မှားတန်ဖိုး` variable with the value `၅ သည် ၃ ထက် ကြီးသည်` and print it.
- Declare a `စာလုံး` variable with `တစ်ခု` in place of `တစ်လုံး`, and read the warning.

Next: [Constants](constants.md).
