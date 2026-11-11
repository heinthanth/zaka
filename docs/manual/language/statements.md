---
title: Statements
description: Each form of statement in Zaka - declaration, assignment, copies, printing, branches, loops, blocks, calls, returns and heap statements - with its grammar and its errors.
---

# Statements

A program is a list of *statements*, run from the first to the last. Each statement is
a Burmese sentence that ends with `။`, and the **verb** at the end of the sentence
decides what the statement does. A statement does something and has no value; an
[expression](expressions.md) has a value and sits inside a statement.

## The forms at a glance

In the shapes below, `NAME` is a name, `KIND` a [kind](kinds.md), `VALUE` and `COND`
expressions, `PLACE` something that can be assigned to (a name, a field or a cell, or in
marked code a pointer's value), `WORD` the classifier of the kind, and `BODY` a block or one statement. Square brackets mark what can be left out.

| Statement | Shape |
| --- | --- |
| declare | `ကိန်းရှင် NAME သည် KIND တစ်WORD [ဖြစ်၍ တန်ဖိုးသည် VALUE] ဖြစ်ပါစေ။` or `ကိန်းရှင် NAME သည် VALUE ဖြစ်ပါစေ။` |
| constant | `ကိန်းသေ NAME သည် KIND တစ်WORD ဖြစ်၍ တန်ဖိုးသည် VALUE ဖြစ်ပါစေ။` or `ကိန်းသေ NAME သည် VALUE ဖြစ်ပါစေ။` |
| assign | `PLACE ကို VALUE ဟု ထားမည်။` |
| declare a shallow duplicate | `ကိန်းရှင် NAME ထဲသို့ VALUE ကို ကူးထည့်မည်။` (or `ကိန်းသေ`) |
| declare a deep duplicate | `ကိန်းရှင် NAME ထဲသို့ VALUE ကို အကုန်ကူးထည့်မည်။` (or `ကိန်းသေ`) |
| shallow duplicate into | `PLACE ထဲသို့ VALUE ကို ကူးထည့်မည်။` |
| deep duplicate into | `PLACE ထဲသို့ VALUE ကို အကုန်ကူးထည့်မည်။` |
| set a character | `TEXT ၏ စာလုံး N ကို CHAR ဟု ထားမည်။` or `TEXT ၏ N ခုမြောက်စာလုံး ကို CHAR ဟု ထားမည်။` |
| print | `VALUE ကို ပြသမည်။` or `VALUE ဟု ပြသမည်။` |
| block | `အောက်ပါတို့ကို လုပ်ဆောင်မည်။` statements `လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။` |
| marked block | `အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။` statements `လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။` (see [Unsafe Zaka](unsafe.md)) |
| if | `အကယ်၍ COND လျှင် BODY` then `[ထိုသို့မဟုတ်ဘဲ COND လျှင် BODY]` then `[သို့မဟုတ်ပါက BODY]` |
| switch | `VALUE သည် CASE ဖြစ်လျှင် BODY` then `[CASE၊ CASE တို့ ဖြစ်လျှင် BODY]` then `[တခြား KIND တစ်WORD ဖြစ်လျှင် BODY]` (see [Enumerations](enums.md#switch)) |
| while | `COND နေသရွေ့ BODY` |
| counted loop | `FROM မှ TO အထိ [STEP စီ] NAME အနေဖြင့် ရေတွက်၍ BODY` |
| each cell | `LIST ၏ အကွက်တိုင်း ကို NAME အနေဖြင့် ယူ၍ BODY` |
| each character | `TEXT ၏ စာလုံးတိုင်း ကို NAME အနေဖြင့် ယူ၍ BODY` |
| generic loop | `ကိန်းရှင် NAME သည် KIND တစ်WORD [ဖြစ်၍ တန်ဖိုးသည် VALUE]`, or `ကိန်းရှင် NAME သည် VALUE`, then `ဖြစ်ကာ COND နေသရွေ့` and a block with `[ပြီးလျှင် STATEMENT]` last |
| break | `ကွင်းဆက် ကို ရပ်မည်။` |
| continue | `နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်။` |
| call | `လုပ်ဆောင်ချက် NAME ကို [ARGS တို့ဖြင့်] လုပ်ဆောင်မည်။` |
| return | `VALUE ကို ပြန်ပေးမည်။` or `ပြန်ပေးမည်။` |
| function | `လုပ်ဆောင်ချက် NAME သည် ...` (see [Functions](functions.md)) |
| start block | `ပရိုဂရမ်သည် ...`, at most one (see [Functions](functions.md#the-start-block)) |
| add a cell | `PLACE ထဲသို့ VALUE ကို ထည့်မည်။` (see [Lists](lists.md#adding-and-removing-cells)) |
| add to text | `TEXT ထဲသို့ VALUE ကို ထည့်မည်။` (see [Text](text.md#changing-text)) |
| remove a cell | `PLACE ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။` |
| replace part of text | `TEXT ၏ အကွက် START မှ END ထိကို VALUE ဖြင့် အစားထိုးမည်။`, or `TEXT ၏ START လုံးမြောက်စာလုံးမှ END လုံးမြောက်စာလုံးထိကို VALUE ဖြင့် အစားထိုးမည်။` (see [Text](text.md#changing-text)) |
| heap, in marked code | `မင်မိုရီတွင် SIZE နေရာယူ၍ လိပ်စာကို PLACE တွင် သိမ်းထားမည်။` and two more (below) |

Some words have alternatives: `လျှင်` or `ဖြစ်လျှင်`; `နေသရွေ့` or `ဖြစ်နေသရွေ့`;
`ဖြစ်ကာ` or `ဖြစ်ပြီး`; `ရေတွက်၍`, `တိုး၍` or `လျှော့၍`; the four spellings of the
block closer (`ပြီးပါပြီ`, `ပြီးပါပြီး`, `ပြီးဆုံးပါပြီ`, `ပြီးဆုံးပါပြီး`).

## The verb ends the sentence

A statement either starts with a reserved word (`ကိန်းရှင်`, `အကယ်၍`, `ကွင်းဆက်`, ...)
or starts with an expression. A statement that starts with an expression must end in
one of these verbs:

| Verb | Statement | Particles on the operands |
| --- | --- | --- |
| `ပြသမည်` | print | the value takes `ကို` or `ဟု` |
| `ထားမည်` | assign | the place takes `ကို`, the value takes `ဟု` |
| `ပြန်ပေးမည်` | return a value | the value takes `ကို`; so does an error value, `<e> ဟူသော အမှားအယွင်း` |
| `နေသရွေ့` | while loop | none: it follows the condition |
| `မှ ... အထိ ... အနေဖြင့်` | counted loop | `မှ` after the first bound, `အထိ` after the second |

Each statement verb ends in `မည်` ("will"). The older spellings that end in `ပါ`, such
as `ပြသပါ` and `ထားပါ`, are ZK0110, whose help line gives the `မည်` form; the compiler
reads the verb as meant and goes on. The declaration `ဖြစ်ပါစေ` ("let it be") and the
closers keep their own spellings.

```zaka-error ZK0110
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0110]: "ပြသပါ" ends in ပါ; a statement ends in မည်
help: write "ပြသမည်"
```

A sentence that ends in another word is ZK0203, with a help line that lists the verbs.
A verb that finds a missing or wrong particle on an operand is ZK0206, and one that finds
too few operands is ZK0205. Operands left over before the verb are ZK0209. A missing
`။` is ZK0201.

```zaka-error ZK0203 ZK0206
၁ ကို ပြသမည်။
၅ + ၃။
၂ ကို ပြသမည်။
၄ ပြသမည်။
```

```text
error[ZK0203]: this statement does not end in a statement verb, found "။"
help: a statement ends in "ပြသမည်", "ထားမည်" or "ကို ပြန်ပေးမည်", or is a loop with "နေသရွေ့" or "မှ ... အထိ"
error[ZK0206]: "ပြသမည်" needs "ကို|ဟု" after operand 1, found no particle
```

The example shows how the compiler recovers: after an error in a statement it skips to
the next `။` and reads on, so one run reports the mistakes of several statements. A
missing `။` joins the statement with the next one in the report. A program with a
reading error is not checked for names and kinds, and does not run.

## Declarations

`ကိန်းရှင်` declares a variable, `ကိန်းသေ` a constant. A declaration writes its kind and
counts it, or writes a value alone:

| Form | Example |
| --- | --- |
| the kind, with no value | `ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။` |
| the kind, then the value | `ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။` |
| the kind at its initial value | `ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။` |
| the value alone | `ကိန်းရှင် အသက် သည် ၁၈ ဖြစ်ပါစေ။` |

- The kind is counted as Burmese counts a thing: the kind, then `တစ်` ("one") and the
  classifier of the kind. Most kinds take `ခု`, `စာလုံး` takes `လုံး`, and a struct, an
  enumeration or an alias takes the classifier its declaration names, `ခု` when it names
  none ([Measure words](kinds.md#measure-words)).
  A struct name and its count word can form one word: `လူတစ်ယောက်`.
- A kind with no count word after it is ZK0234. A count word with another classifier is
  the warning ZK0509; the program still compiles.
- `ဖြစ်၍ တန်ဖိုးသည်` ("is, and its value is") brings the value after the counted kind.
- `ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး` sets the variable to the
  [initial value](kinds.md#initial-values) of its kind.
- With no kind written, the declaration takes the kind of its value: `ကိန်းပြည့်` for an
  integer literal, `ဒဿမကိန်း` for a decimal one, `စာသား` for text in quotes, and a list of
  the cells' kind and count for a list literal. A literal with a kind after it, such as
  `၂၀၀ ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု`, gives that kind. The null address
  `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` fits each pointer kind and names none, so it needs the
  written kind (ZK0430); so do `အလွတ်` (ZK0477) and `ကနဦးတန်ဖိုး` (ZK0478).
- Parameters, struct fields and the result clause of a function header count their kind
  alike: `<name> ဟူသော <kind> တစ်ခု ကို လက်ခံပြီး`, `<name> ဟူသော <kind> တစ်ခု` and
  `<kind> တစ်ခု ကို ပြန်ပေးမည်` ([Functions](functions.md#results-and-return)).

A variable without a value holds none: no default fills it, and each path sets it before
it is read ([Set before read](#set-before-read)). A constant needs a value (ZK0308) and can
never be assigned to (ZK0306). The value is
checked first, so it cannot use the name that the statement declares (ZK0301), and it
must fit the kind (ZK0401).

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

## Set before read

No variable, field or cell is read before the program gives it a value. A declaration can
leave the value out, when the value comes later or differs from path to path; the compiler
then checks that each path from the declaration to a read sets the variable first. A read
that a path reaches with the variable unset is ZK0722.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ် သည် ၇၅ ဖြစ်ပါစေ။
ကိန်းရှင် အဆင့် သည် စာသား တစ်ခု ဖြစ်ပါစေ။
အကယ်၍ အမှတ် >= ၅၀ ဖြစ်လျှင် အဆင့် ကို "အောင်" ဟု ထားမည်။
သို့မဟုတ်ပါက အဆင့် ကို "ကျ" ဟု ထားမည်။      // both arms set it
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

The rules:

- A declaration with no value leaves the variable unset; an assignment sets it, and so
  does a call that takes the variable through a parameter marked `ဒေတာ ထည့်ပေးရန်`
  ([Filled parameters](functions.md#filled-parameters)). An argument of a parameter marked
  `ဒေတာ ပြောင်းလဲရန်` is a read, since the function can read it. Where two
  paths join, the variable is set when it is set on both. The block of a loop can run no
  times, so a write inside a `နေသရွေ့` loop sets nothing for the code after the loop.
- A read is each use of the value: printing it, an operand, an argument, a call through
  it, a field or a cell of it, an add to a list, and, in marked code, its address.
- The rule holds for each kind, and no kind has a hidden default: a number, text and a
  plain list as much as a shared object or a function value.
- **Structs.** A struct counts as set once each field is set on each path. A struct inside
  a struct is followed one field at a time too: writing `က ၏ ခ ၏ ဂ` sets that one field,
  and `က ၏ ခ` counts as set once each of its fields is
  ([Structs](structs.md#where-a-struct-kind-goes)).
- **Counted lists.** A counted list counts as set once each cell is: cells written at
  constant positions, or a counted loop of fixed bounds, with no break, whose block writes
  the cell of its counter on each round, which sets each cell from its start to its end.
  The check follows the cells of a list of up to 256; a longer list needs a whole value or
  a loop over each cell.
- **Earlier cells.** Inside such a loop, a read of the cell a constant number of steps
  before the counter, as in `ဖ ၏ အကွက် (န - ၁)`, is set when the cells before the start of
  the loop are: each round reads a cell set before the loop or in an earlier round. When
  one of those cells can be unset, the read is ZK0722.
- A read of a counted list whose cells the check can neither prove set nor unset, as after
  a write at a computed position, is the warning ZK0723,
  and the program runs. Such a cell holds zero octets until the program writes it
  ([Lists](lists.md#list-kinds)).
- The check runs at each `--mem-check` level, and the level never changes it: these are
  errors of safe code, not findings about marked code.
- `ကနဦးတန်ဖိုး` is the value to write when the program wants a kind's
  [initial value](kinds.md#initial-values) and has no other.

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

A variable of the top level is a global: each function after it reaches it. The start
block sets a global or reads it as it does a local. A function that reads a global, itself
or through the functions it calls, needs that global set on each path to each call of it:
a call made while the global can be unset is ZK0724.

- A call through a function value counts the reads of each function the program uses as
  a value.
- A write of a global inside a function does not count as setting it for the code after
  the call: give the global its value at its declaration, or in the start block.
- A global of a module needs its value at its declaration (ZK0483), since a module has no
  start block to set it ([Modules](modules.md#globals-of-a-module)).

```zaka-error ZK0724
ကိန်းရှင် စုစုပေါင်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ပြ သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စုစုပေါင်း ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ပြ ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0724]: the call of "ပြ" reads "စုစုပေါင်း", which can be unset here
```

## Assignment

`PLACE ကို VALUE ဟု ထားမည်။` puts the value into the place. The value is worked out first
with the current contents, then stored, so `အသက် ကို အသက် + ၁ ဟု ထားမည်။` adds one. The
place is a variable, a field, a cell of a list (`x ၏ အကွက် i`), in marked code the value
at a pointer (`p ၏ တန်ဖိုး`), or a character of text (`t ၏ စာလုံး n`, `t ၏ n ခုမြောက်စာလုံး`; see
[Text](text.md#changing-text)). Another expression is ZK0410. A constant (ZK0306) and a counted-loop counter
(ZK0307) cannot be assigned to. The value must widen to the kind of the place
([Conversions](conversions.md)).

## Copies into a variable

Two verbs put a duplicate of a value into a variable, and tell how deep it goes:

- `PLACE ထဲသို့ VALUE ကို ကူးထည့်မည်။` ("will duplicate into") puts a duplicate of the value
  there.
  On a value, such as text, a list or a struct, this is the same as `ဟု ထားမည်`, since a
  value is copied in each case. On a shared object it makes a **new object** with the
  same fields: a shallow duplicate, whose fields still reach the objects the old fields
  reached.
- `PLACE ထဲသို့ VALUE ကို အကုန်ကူးထည့်မည်။` ("will duplicate all into") makes a deep duplicate: each
  shared object the value reaches is copied too, once each, so the duplicate keeps the shape of
  the original. `အကုန် ကူးထည့်မည်` with a space reads the same.

After `ကိန်းရှင်` or `ကိန်းသေ`, the same sentences declare the name, with the kind of the
value. The expression forms `VALUE ၏ မိတ္တူ` and `VALUE တစ်<classifier>လုံး ၏ မိတ္တူ` make
the same duplicates wherever a value goes; [Values and objects](objects.md) states the rules
of both depths. `ထဲသို့ VALUE ကို ထည့်မည်` remains the add of a cell or of text; the verb
tells them apart.

```zaka
ဘူး ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု၊ တန် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

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

`ပြောင်းထည့်မည်` is no longer a statement of its own; in marked code it names the block
duplicate that allows overlap ([Unsafe Zaka](unsafe.md)).

## Printing

`ပြသမည်` prints its operand and then a line break. A number prints in Burmese digits, a
true/false value as `မှန်` or `မမှန်`, a character and text as themselves, a list as its
cells between `[` and `]`, and a struct as its name and fields. Text inside a list or a
struct prints in quotes. A template slot in a string literal writes its value with the same rules
([Text](text.md#the-text-of-a-value)).

The operand takes `ကို` when it is a plain value (a text variable included), and `ဟု` when
it is a string literal, a `+` join that holds one, or a whole sentence (a clause that ends in a verb with a polite ending). The other choice still runs
and gets a warning, ZK0505 or ZK0506; [Natural wording](naturalness.md) has the rules.
A logical expression needs parentheses (ZK0204).

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၄၂ ကို ပြသမည်။
"မင်္ဂလာပါ" ဟု ပြသမည်။
၃ သည် ၂ ထက် ကြီးသည် ဟု ပြသမည်။
'က' ကို ပြသမည်။
၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၂
မင်္ဂလာပါ
မှန်
က
[၁၊ ၂၊ ၃]
```

## Blocks

A block is a run of statements between `အောက်ပါတို့ကို လုပ်ဆောင်မည်။` and
`လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။`. Names declared inside it end at the closer
([Names and scope](scope.md)). A closer with no opener is ZK0226.

A **marked block** opens with `အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။` ("do the
following, although it is unsafe") and closes as a block does. Inside it, pointer work
and calls of marked functions are allowed ([Unsafe Zaka](unsafe.md)). A start block
cannot be marked itself (ZK0201); a marked block goes inside it.

Wherever the shape of a statement has `BODY`, a block or one single statement can stand.
A single statement is a short form of a block of one: `အကယ်၍ ... လျှင် ၁ ကို ပြသမည်။`. A
one-statement `BODY` cannot itself be an `if` (ZK0215), since it would be unclear which
`if` an `otherwise` belongs to. Put the inner `if` in a block.

## If

`အကယ်၍` tests the condition, which must be a true/false value (ZK0402). The first
branch whose condition holds runs, and the rest are skipped. `သို့မဟုတ်ပါက` ("otherwise")
takes no condition and runs when no other branch did. `လျှင်` is written attached to
the last word of the condition (`ငယ်လျှင်`); after a name or a whole sentence, write
`ဖြစ်လျှင်` as a word of its own. An `ထိုသို့မဟုတ်ဘဲ` or `သို့မဟုတ်ပါက` with no `if` before it is
ZK0211.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၃ ဖြစ်ပါစေ။
အကယ်၍ အမှတ် သည် ၈၀ အောက် မငယ်လျှင် "က" ဟု ပြသမည်။
ထိုသို့မဟုတ်ဘဲ အမှတ် သည် ၆၀ အောက် မငယ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ခ" ဟု ပြသမည်။
"ကောင်းပါသည်" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
သို့မဟုတ်ပါက "ဂ" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ခ
ကောင်းပါသည်
```

```zaka-error ZK0215
အကယ်၍ မှန် ဖြစ်လျှင် အကယ်၍ မမှန် ဖြစ်လျှင် ၁ ကို ပြသမည်။
```

## Loops

All loops test or count before the first round, so a loop can run zero times.

**While.** `COND နေသရွေ့ BODY` works out the condition before each round and runs the
`BODY` while it is true.

**Counted loop.** `FROM မှ TO အထိ [STEP စီ] NAME အနေဖြင့် ရေတွက်၍ BODY` declares a counter
`NAME` that lives in the loop. The bounds and the step are worked out once, before the
first round, and must be integers (ZK0405). Both bounds are included. The counter has the
common kind of the two bounds, and the step must widen to it; a literal bound takes the
kind of the other bound. The step must be positive, a literal zero or less being ZK0409
and a zero at run time stopping the program. The three verbs differ in the direction:

| Verb | Direction |
| --- | --- |
| `ရေတွက်၍` | toward `TO`: up when `FROM <= TO`, down otherwise |
| `တိုး၍` | up in each case; zero rounds when `FROM` is past `TO` |
| `လျှော့၍` | down in each case; zero rounds when `FROM` is below `TO` |

The inner part can read the counter but not assign to it (ZK0307). The counter and the
names declared in the inner part share one scope, so the inner part cannot declare a name
equal to the counter's (ZK0302).

**Each cell and each character.** `LIST ၏ အကွက်တိုင်း ကို NAME အနေဖြင့် ယူ၍ BODY` runs
the `BODY` once for each cell of a list, the first cell first; in marked code, for a
pointer to a list it walks the list the pointer points at, as a cell read does.
`TEXT ၏ စာလုံးတိုင်း ကို NAME အနေဖြင့် ယူ၍ BODY` runs it once for each character of text,
and walks the u8s of the text once. The value in front of `၏` is worked out once, before
the first round, and the loop walks its own duplicate of it, which shares the cells: the
`BODY` can change the list or the text, and the rounds still see the cells it held at the
start. `NAME` is a variable of the cell kind, or a `စာလုံး`, that lives in one round: each
round sets a fresh variable to the next cell or character, so assigning to it leaves the
list as it was, and a nested function that uses it sees the value of its own round
([Function values](function-values.md)). A list of pointers is walked
like each other list. An each-cell loop over text, or an each-character loop over a value that
is not text, is ZK0434.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၇၀၊ ၈၅၊ ၉၀] ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက်တိုင်း ကို အမှတ် အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အမှတ် ကို အမှတ် + ၅ ဟု ထားမည်။
အမှတ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အမှတ်များ ကို ပြသမည်။
"a၁" ၏ စာလုံးတိုင်း ကို စ အနေဖြင့် ယူ၍ စ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၇၅
၉၀
၉၅
[၇၀၊ ၈၅၊ ၉၀]
a
၁
```

**Generic loop.** `ကိန်းရှင် NAME သည် KIND [VALUE] ဖြစ်ကာ COND နေသရွေ့` opens a block.
The declaration runs once; then, while the condition holds, the block runs and then the
update. The update is the last statement of the block, opened with `ပြီးလျှင်`
("when done"); it also runs after `နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်`. An update that is not
last is ZK0213, and a `ပြီးလျှင်` outside a generic loop is ZK0212. The loop takes a
block alone, and its variable is declared with `ကိန်းရှင်`, never `ကိန်းသေ` (ZK0225). The
variable is gone after the closer.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဆုံး သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
၁ မှ ဆုံး အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဆုံး ကို ၁၀ ဟု ထားမည်။
န ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
"---" ဟု ပြသမည်။
၁၀ မှ ၀ အထိ ၅ စီ န အနေဖြင့် လျှော့၍ န ကို ပြသမည်။
"---" ဟု ပြသမည်။
၅ မှ ၁ အထိ န အနေဖြင့် တိုး၍ န ကို ပြသမည်။
"ပြီး" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁
၂
၃
---
၁၀
၅
၀
---
ပြီး
```

The first loop shows that the bound is worked out once: changing `ဆုံး` in the inner part does
not lengthen the loop.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ကာ ကိန်း သည် ၁၀ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ ကိန်း % ၂ သည် ၁ နှင့် ညီလျှင် နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်။
အကယ်၍ ကိန်း သည် ၆ ထက် ကြီးလျှင် ကွင်းဆက် ကို ရပ်မည်။
ကိန်း ကို ပြသမည်။
ပြီးလျှင် ကိန်း ကို ကိန်း + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ကိန်းရှင် ဂဏန်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
မှန် ဖြစ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဂဏန်း ကို ဂဏန်း * ၂ ဟု ထားမည်။
အကယ်၍ ဂဏန်း သည် ၁၀၀ ထက် ကြီးလျှင် ကွင်းဆက် ကို ရပ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ဂဏန်း ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၀
၂
၄
၆
၁၂၈
```

## Break and continue

`ကွင်းဆက် ကို ရပ်မည်။` leaves the innermost loop. `နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်။` skips
the rest of the round of the innermost loop and goes to the next round; in a generic
loop the update runs first. Outside a loop these are ZK0309 and ZK0310.

## Calls and returns

`လုပ်ဆောင်ချက် f ကို ၁၊ ၂ တို့ဖြင့် လုပ်ဆောင်မည်။` runs a function and drops its result.
`VALUE ကို ပြန်ပေးမည်။` ends a function with a value, and `ပြန်ပေးမည်။` ends it
with none. A return outside a function is ZK0311. [Functions](functions.md) has the rest.

Statements run inside the [start block](functions.md#the-start-block) of the program and
inside functions. The top level of a file holds declarations alone, and a statement that
runs there is ZK0326: the declarations set their starting values, and then the start
block runs.

## Heap statements

Three statements ask for room on the heap, change it and give it back, and three more
write the cells of a block. Each works on a pointer, which names the block, and each needs
marked code: outside it each is ZK0468. [Unsafe Zaka](unsafe.md) states their rules; the
shapes are:

| Statement | Shape |
| --- | --- |
| allocate | `မင်မိုရီတွင် KIND N WORDစာ နေရာယူ၍ လိပ်စာကို PLACE တွင် သိမ်းထားမည်။` |
| resize | `လိပ်စာ PLACE ရှိ မင်မိုရီကို KIND N WORDစာ ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။` |
| give back | `လိပ်စာ PLACE ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။` |

`WORD` is the measure word of the kind, as in `ကိန်းပြည့် ၃ ခုစာ` and `စာလုံး ၅ လုံးစာ`. The
size can also be `N ဘိုက်စာ`, a count of octets. `မင်မိုရီ` and `မှတ်ဉာဏ်` mean the
same.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၄၂ ဟု ထားမည်။
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ပ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၂
မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ
```

```zaka-error ZK0468
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0468]: a heap statement needs marked code
```

## Errors of statements

| Code | Raised when |
| --- | --- |
| ZK0110 | a statement verb that ends in `ပါ` in place of `မည်` |
| ZK0201 | a token is not the one expected, often a missing `။` |
| ZK0203 | a sentence ends in no statement verb |
| ZK0205, ZK0206, ZK0209 | a verb has too few operands, a wrong particle, or a surplus |
| ZK0211, ZK0212, ZK0213, ZK0215, ZK0226 | a branch, update, single-statement block or closer in the wrong place |
| ZK0302, ZK0306, ZK0307, ZK0308 | a name declared twice, a constant or counter assigned to, a constant with no value |
| ZK0309, ZK0310, ZK0311 | break, continue or return outside its home |
| ZK0402, ZK0405, ZK0409 | a condition that is no true/false value, a bound that is no integer, a step of zero or below |
| ZK0434 | an each loop over a value it cannot walk |
| ZK0468 | a heap statement outside marked code |
| ZK0722 | a read of a variable, a field or a cell that a path reaches with no value given |
| ZK0723 | a warning: a read of a counted list whose cells the check cannot prove set |
| ZK0724 | a call of a function that reads a global that can be unset at the call |
| ZK0437 | an add or a remove on a list that cannot grow or shrink, or `နောက်ဆုံးအကွက်` on text |
| ZK0439 | `၏ အရှေ့တွင်` after a value that is no cell and no character |

```zaka-error ZK0307
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ န ကို ၀ ဟု ထားမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```zaka-error ZK0402
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ ၁ ဖြစ်လျှင် ၁ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```zaka-error ZK0309
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကွင်းဆက် ကို ရပ်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```zaka-error ZK0434
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"abc" ၏ အကွက်တိုင်း ကို a အနေဖြင့် ယူ၍ a ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခြေလှမ်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
"အစ" ဟု ပြသမည်။
၁ မှ ၃ အထိ ခြေလှမ်း စီ န အနေဖြင့် ရေတွက်၍ န ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အစ
the program stopped: counted loop step is not positive
```
