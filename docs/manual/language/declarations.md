---
title: Declarations and assignment
description: Declaring variables and constants, the rule that each value is set before it is read, globals, assignment and the statements that duplicate a value.
---

# Declarations and assignment

A declaration creates a variable (mutable variable, `let mut`) or a constant (`const`).
This page explains when a declaration needs a value, when a read is allowed, and how
assignment and the two forms of a duplicate (copy) work. For examples, see
[Variables](../../learn/variables/README.md) in Learn.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။
ကိန်းရှင် အရွယ် သည် ၂၀ ဖြစ်ပါစေ။
ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းသေ အများဆုံး သည် ၁၀၀ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် အက္ခရာ သည် စာလုံး တစ်လုံး ဖြစ်၍ တန်ဖိုးသည် 'က' ဖြစ်ပါစေ။
ကိန်းရှင် အမည် သည် "မမ" ဖြစ်ပါစေ။
အသက် ကို အသက် + ၁ ဟု ထားမည်။
အသက် ကို ပြသမည်။
အရွယ် ကို ပြသမည်။
အရေအတွက် ကို ပြသမည်။
အများဆုံး ကို ပြသမည်။
အက္ခရာ ကို ပြသမည်။
အမည် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၉
၂၀
၀
၁၀၀
က
မမ
```

## Declarations

```text
ကိန်းရှင် <name> သည် <kind> တစ်<measure word> ဖြစ်ပါစေ။
ကိန်းရှင် <name> သည် <kind> တစ်<measure word> ဖြစ်၍ တန်ဖိုးသည် <value> ဖြစ်ပါစေ။
ကိန်းရှင် <name> သည် <value> ဖြစ်ပါစေ။
ကိန်းသေ <name> သည် <kind> တစ်<measure word> ဖြစ်၍ တန်ဖိုးသည် <value> ဖြစ်ပါစေ။
ကိန်းသေ <name> သည် <value> ဖြစ်ပါစေ။
```

We use `ကိန်းရှင်` to declare a variable and `ကိန်းသေ` to declare a constant. A
declaration can write the data kind (type) with its measure word, the value, or both:

| Form | Example |
| --- | --- |
| the data kind, with no value | `ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။` |
| the data kind, then the value | `ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။` |
| the data kind at its initial value | `ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။` |
| the value alone | `ကိန်းရှင် အသက် သည် ၁၈ ဖြစ်ပါစေ။` |

| Words | Reads as |
| --- | --- |
| `ကိန်းရှင်` / `ကိန်းသေ` | "variable" / "constant" |
| `<name> သည်` | "`<name>` is" |
| `<kind> တစ်ခု` | "one `<kind>`"; `ခု` is the measure word (classifier) |
| `ဖြစ်၍ တန်ဖိုးသည်` | "is, and its value is" |
| `ဖြစ်ပါစေ` | "let it be" |

### The data kind and its measure word

We count a data kind as Burmese counts a thing: the data kind, then `တစ်` ("one")
and its measure word. Most data kinds use `ခု` and `စာလုံး` uses `လုံး`, while a struct, an
enumeration or a kind alias uses the measure word that its declaration gives, or `ခု` when
it gives none ([Measure words](kinds.md#measure-words)). A struct name and its measure
word can be written as one word: `လူတစ်ယောက်`.

- A data kind with no measure word after it is ZK0234.
- A measure word that is not the data kind's own is the warning ZK0509, and the program
  still compiles.
- To set the variable to the [initial value](kinds.md#initial-values) of its data kind, we
  write `ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး`.
- Parameters, struct fields and the result in a function signature count their data kind
  alike: `<name> ဟူသော <kind> တစ်ခု ကို လက်ခံပြီး`, `<name> ဟူသော <kind> တစ်ခု` and
  `<kind> တစ်ခု ကို ပြန်ပေးမည်` ([Functions](functions-results.md#declaring-a-result)).

### A declaration with no data kind

When we leave the data kind out, the declaration gets the data kind of its value:
`ကိန်းပြည့်` for an integer literal, `ဒဿမကိန်း` for a decimal one, `စာသား` for text in
quotes, and for a list literal a list of the cells' data kind and count. A literal with a
data kind after it, such as `၂၀၀ ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု`, gives that data
kind.

Some values fit several data kinds and name none, so a declaration with one of them needs
the data kind written out:

| Value | Error without a written data kind |
| --- | --- |
| the null address `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`, which fits each pointer kind | ZK0430 |
| the blank `အလွတ်` | ZK0477 |
| `ကနဦးတန်ဖိုး` | ZK0478 |

### Rules of the value

A variable declared without a value has none: no default fills it, and each path must set
it before it is read ([Set before read](#set-before-read)). A constant needs a value
(ZK0308) and can never be assigned to (ZK0306). The compiler checks the value before it
adds the name, so the value cannot use the name that the statement declares (ZK0301), and
the value must fit the data kind (ZK0401).

```zaka-error ZK0306
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ အများဆုံး သည် ၁၀၀ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
အများဆုံး ကို ၂၀၀ ဟု ထားမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```zaka-error ZK0234
ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် ဖြစ်ပါစေ။
```

```text
error[ZK0234]: a kind here takes "တစ်" and a classifier
```

```zaka-error ZK0430
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0430]: the value gives no kind for "ပ"
```

## Marks

A declaration can take marks, each a word or a phrase that ends in `သော`, and these stand
in two places of its head. Before the noun of the declaration stand the marks that change
how others can use it: `အများသုံးနိုင်သော` (exported), `စိတ်မချရသော` (unsafe),
`ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော` (comptime), `မျှဝေနိုင်သော` (shared) and the mark of
threads. After `သည်` stand the marks that tell what the declaration is: where it comes from
or where it goes, and for a struct the structs it is based on. Two marks in one place stand
with `၊` between them, as the items of a list do, and a missing `၊` is ZK0266:

```zaka-file
// marks.zaka
အကောင့် ဟူသော အများသုံးနိုင်သော၊ မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။
```

| Place | Marks | Pages |
| --- | --- | --- |
| before `လုပ်ဆောင်ချက်` | `အများသုံးနိုင်သော`, `စိတ်မချရသော`, `ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော` | [Modules](modules.md), [Unsafe code](unsafe.md), [Compile time](compile-time.md) |
| before `ဒေတာအစုအဝေး` | `အများသုံးနိုင်သော`, the mark of threads, `မျှဝေနိုင်သော` | [Values and objects](objects.md), [Threads](threads.md) |
| before `ကိန်းရှင်`, `ကိန်းသေ`, `ဒေတာအမျိုးအစား`, `ဒေတာအမျိုးအစားကွဲ`, `ဂုဏ်သတ္တိ` | `အများသုံးနိုင်သော` | [Modules](modules.md) |
| after `သည်`, in a function | `"<lib>" မှလာသော`, `ပြင်ပမှလာသော`, `"<C name>" အမည်ဖြင့် ပြင်ပမှ ခေါ်ဆိုနိုင်သော` | [C functions](functions-c.md) |
| after `သည်`, in a struct | `"<C name>" အမည်ဖြင့် ပြင်ပမှ ယူသုံးနိုင်သော`, `ပြင်ပမှ လာသော`, `<struct> ကို အခြေခံသော` | [Structs](structs.md), [Structs based on structs](based-structs.md) |

## Set before read

No variable, field or cell is read before the program gives it a value. A declaration can
leave the value out when the value comes later or differs from path to path, and the
compiler then checks that each path from the declaration to a read sets the variable
first. A read that a path reaches with the variable unset is ZK0722.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ် သည် ၇၅ ဖြစ်ပါစေ။
ကိန်းရှင် အဆင့် သည် စာသား တစ်ခု ဖြစ်ပါစေ။
အကယ်၍ အမှတ် >= ၅၀ ဖြစ်လျှင် အဆင့် ကို "အောင်" ဟု ထားမည်။
သို့မဟုတ်ပါက အဆင့် ကို "ကျ" ဟု ထားမည်။ // both arms set it
အဆင့် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အောင်
```

```zaka-error ZK0722
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ် သည် ၇၅ ဖြစ်ပါစေ။
ကိန်းရှင် အဆင့် သည် စာသား တစ်ခု ဖြစ်ပါစေ။
အကယ်၍ အမှတ် >= ၅၀ ဖြစ်လျှင် အဆင့် ကို "အောင်" ဟု ထားမည်။
အဆင့် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0722]: "အဆင့်" can be unset here: a path reaches this read with no value given to it
```

### What sets a variable and what reads it

Two things set a variable:

- an assignment;
- a call that passes it to a writable parameter (`out`), one marked `ဒေတာ ထည့်ပေးရန်`
  ([Function parameters](functions-parameters.md#writable-parameters)).

Each other use reads it: printing it, an operand, an argument, a call through it, a field
or a cell of it, an add to a list, and in unsafe code its address. An argument of a
mutable parameter (`inout`), one marked `ဒေတာ ပြောင်းလဲရန်`, reads it too, because the
function can read it before it writes.

Where two paths join, the variable is set when it is set on both of them. The block of a
loop can run zero times, so a write inside a `နေသရွေ့` loop sets nothing for the code after
the loop.

The rule applies to each data kind, and no data kind has a hidden default: a number, text
and a plain list need a value as much as a shared object or a function value does. When
we want the [initial value](kinds.md#initial-values) of a data kind and have no other
value, we write `ကနဦးတန်ဖိုး`.

### Structs and counted lists

- **Structs.** A struct counts as set once each of its fields is set on each path. A
  struct inside a struct is followed one field at a time too: writing `က ၏ ခ ၏ ဂ` sets that
  one field, and `က ၏ ခ` counts as set once each of its fields is
  ([Structs](structs.md#using-a-struct-as-a-data-kind)).
- **Counted lists.** A counted list (fixed-size array) counts as set once each cell is.
  The cells can be written at constant positions, or with a counted loop of fixed bounds,
  with no break, whose block writes the cell of its counter on each round, which sets each
  cell from the start of the loop to its end. The check follows the cells of a list of up
  to 256 cells, and a longer list needs a whole value or a loop over each cell.
- **Earlier cells.** Inside such a loop, a read of the cell a constant number of steps
  before the counter, as in `ဖ ၏ အကွက် (န - ၁)`, counts as set when the cells before the
  start of the loop are set, because each round reads a cell that was set before the loop
  or in an earlier round. When one of those cells can be unset, the read is ZK0722.
- **Cells the check cannot follow.** A read of a counted list whose cells the check can
  neither prove set nor unset, as after a write at a computed position, gets the warning
  ZK0723, and the program runs. Such a cell contains zero octets until the program writes
  it ([Lists](lists.md#list-kinds)).

The check runs at each level of the `mem` lint, and the level never changes it, because
these are errors of safe code, not findings about unsafe code.

```zaka-warning ZK0723
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် တ သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် န သည် ၂ ဖြစ်ပါစေ။
တ ၏ အကွက် ၀ ကို ၅ ဟု ထားမည်။
တ ၏ အကွက် ၁ ကို ၅ ဟု ထားမည်။
တ ၏ အကွက် န ကို ၆ ဟု ထားမည်။
တ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0723]: "တ" can have a cell with no value here; the check cannot tell which cells a write or a read reaches
```

## Globals and calls

A variable declared at the top level of a file is a global, and each function after it
can reach it. The program start block sets a global or reads it the same as a local. A
function that reads a global, in its own block or through the functions it calls, needs
that global set on each path to each call of it, and a call made while the global can be
unset is ZK0724.

- A call through a function value counts the reads of each function that the program uses
  as a value.
- A write of a global inside a function does not count as setting it for the code after
  the call, so we give the global its value at its declaration or in the program start
  block.
- A global of a module needs its value at its declaration (ZK0483), because a module has
  no program start block to set it ([Modules](modules.md#globals-of-a-module)).

```zaka-error ZK0724
ကိန်းရှင် စုစုပေါင်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
ပြ ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စုစုပေါင်း ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပြ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0724]: the call of "ပြ" reads "စုစုပေါင်း", which can be unset here
```

## Assignment

```text
<place> ကို <value> ဟု ထားမည်။
```

To put a value into a place, we write `<place> ကို <value> ဟု ထားမည်။`. The value is worked
out first, with the current contents, and then stored, so
`အသက် ကို အသက် + ၁ ဟု ထားမည်။` adds one. A field and a cell follow the same order: the
value runs first, and then the place gets it. A value that reads an optional or a result,
passes a failure on with `?`, or calls a function that adds to the same list is done before
the place is found, so the store lands in the list as it is after the call.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁] ဖြစ်ပါစေ။
ချဲ့ ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
စာ ထဲသို့ ၅၀ ကို ထည့်မည်။ // the list gains a cell
၇ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
စာ ၏ အကွက် ၀ ကို ချဲ့ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ဟု ထားမည်။
စာ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၇၊ ၅၀]
```

| Place | Example | Rules |
| --- | --- | --- |
| a variable | `က` | |
| a field | `ကျော် ၏ အသက်` | [Structs](structs.md) |
| a cell of a list | `x ၏ အကွက် i` | [Lists](lists.md) |
| a character of text | `t ၏ စာလုံး n`, `t ၏ n ခုမြောက်စာလုံး` | [Text](text.md#changing-text) |
| the value at a pointer, in unsafe code | `p ၏ တန်ဖိုး` | [Unsafe Zaka](unsafe.md) |

Another expression in the place is ZK0410. A constant (ZK0306) and the counter of a
counted loop (ZK0307) cannot be assigned to. The value must widen to the data kind of the
place ([Conversions](conversions.md)).

## Copies into a variable

```text
<place> ထဲသို့ <value> ကို ကူးထည့်မည်။
<place> ထဲသို့ <value> ကို အကုန်ကူးထည့်မည်။
ကိန်းရှင် <name> ထဲသို့ <value> ကို ကူးထည့်မည်။
```

Two verbs put a duplicate (copy) of a value into a place, and the two differ in how deep the
duplicate goes:

| Verb | Reads as | On a value (text, a list, a struct) | On a shared object (reference type) |
| --- | --- | --- | --- |
| `ကူးထည့်မည်` | "will duplicate into" | the same as `ဟု ထားမည်`, since a value is duplicated in each case | a new object with the same fields, a shallow duplicate whose fields still reach the objects that the old fields reached |
| `အကုန်ကူးထည့်မည်` | "will duplicate all into" | a deep duplicate, as in the next cell | a deep duplicate: each shared object that the value reaches is duplicated too, once each, so the duplicate keeps the shape of the original |

`အကုန် ကူးထည့်မည်` with a space means the same as `အကုန်ကူးထည့်မည်`. After `ကိန်းရှင်` or
`ကိန်းသေ`, the same sentences declare the name, with the data kind of the value. The
expressions `<value> ၏ မိတ္တူ` and `<value> တစ်<measure word>လုံး ၏ မိတ္တူ` make the same
two duplicates wherever a value goes, and [Values and objects](objects.md) states the
rules of both depths. `ထဲသို့ <value> ကို ထည့်မည်` without `ကူး` still adds a cell or text,
and the verb tells the two apart.

```zaka
ဘူး ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင်
အမည် ဟူသော စာသား တစ်ခု နှင့်
တန် ဟူသော ကိန်းပြည့် တစ်ခု
တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် "စံ" ဖြစ်ပါစေ။
ကိန်းရှင် ခ ထဲသို့ က ကို ကူးထည့်မည်။
ခ ထဲသို့ "ကား" ကို ထည့်မည်။
"{က} {ခ}" ဟု ပြသမည်။
ကိန်းရှင် ဘ သည် ဘူး(အမည်: "ဘူး"၊ တန်: ၁) ဖြစ်ပါစေ။
ကိန်းရှင် တူ သည် ဘ ဖြစ်ပါစေ။
ကိန်းရှင် သစ် ထဲသို့ ဘ ကို ကူးထည့်မည်။
သစ် ၏ တန် ကို ၂ ဟု ထားမည်။
တူ ၏ တန် ကို ၃ ဟု ထားမည်။
ဘ ကို ပြသမည်။
သစ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
စံ စံကား
ဘူး(အမည်: "ဘူး"၊ တန်: ၃)
ဘူး(အမည်: "ဘူး"၊ တန်: ၂)
```

`ပြောင်းထည့်မည်` is no longer a statement on its own, and in unsafe code it names the
block duplicate that allows overlap ([Unsafe Zaka](unsafe.md)).
