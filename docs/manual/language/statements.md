---
title: Statements
description: Each form of statement (sentence) in Zaka at a glance, the verb that ends a statement, printing, calls and returns, the top level of a file and the heap statements.
---

# Statements

A program is a list of statements, which run from the first to the last. Each statement
is a Burmese sentence that ends with `။`, and the verb at the end of the sentence decides
what the statement does. A statement does something and has no value, while an
[expression](expressions.md) has a value and sits inside a statement.

This page lists each form of statement, states the rules that all statements share, and
covers printing, calls, returns and the heap statements. Two more pages state the other
forms in full: [Declarations and assignment](declarations.md) and [Blocks, branches and
loops](control-flow.md). [Zaka sentences](../../learn/sentences/README.md) in the Learn
book introduces the sentence shape with gentle examples.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ် သည် ၇၃ ဖြစ်ပါစေ။                        // a declaration
အမှတ် ကို အမှတ် + ၂ ဟု ထားမည်။                          // an assignment
အကယ်၍ အမှတ် သည် ၆၀ အောက် မငယ်လျှင် "အောင်" ဟု ပြသမည်။    // an if, with a print
၁ မှ ၂ အထိ န အနေဖြင့် ရေတွက်၍ န ကို ပြသမည်။               // a counted loop
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အောင်
၁
၂
```

## The forms at a glance

In the patterns below, `<name>` is a name, `<kind>` a [data kind](kinds.md),
`<measure word>` its [measure word](kinds.md#measure-words), `<value>` and `<condition>`
expressions, and `<place>` something that we can assign to: a name, a field, a cell, or
in unsafe code the value at a pointer. `<block>` is a block or a single statement, and
square brackets mark a part that can be left out.

| Statement | Pattern | Rules |
| --- | --- | --- |
| declare a variable | `ကိန်းရှင် <name> သည် <kind> တစ်<measure word> [ဖြစ်၍ တန်ဖိုးသည် <value>] ဖြစ်ပါစေ။` or `ကိန်းရှင် <name> သည် <value> ဖြစ်ပါစေ။` | [Declarations](declarations.md#declarations) |
| declare a constant | `ကိန်းသေ <name> သည် <kind> တစ်<measure word> ဖြစ်၍ တန်ဖိုးသည် <value> ဖြစ်ပါစေ။` or `ကိန်းသေ <name> သည် <value> ဖြစ်ပါစေ။` | [Declarations](declarations.md#declarations) |
| assign | `<place> ကို <value> ဟု ထားမည်။` | [Assignment](declarations.md#assignment) |
| declare a shallow duplicate | `ကိန်းရှင် <name> ထဲသို့ <value> ကို ကူးထည့်မည်။` (or `ကိန်းသေ`) | [Copies](declarations.md#copies-into-a-variable) |
| declare a deep duplicate | `ကိန်းရှင် <name> ထဲသို့ <value> ကို အကုန်ကူးထည့်မည်။` (or `ကိန်းသေ`) | [Copies](declarations.md#copies-into-a-variable) |
| shallow duplicate into | `<place> ထဲသို့ <value> ကို ကူးထည့်မည်။` | [Copies](declarations.md#copies-into-a-variable) |
| deep duplicate into | `<place> ထဲသို့ <value> ကို အကုန်ကူးထည့်မည်။` | [Copies](declarations.md#copies-into-a-variable) |
| set a character | `<text> ၏ စာလုံး <n> ကို <character> ဟု ထားမည်။` or `<text> ၏ <n> ခုမြောက်စာလုံး ကို <character> ဟု ထားမည်။` | [Text](text.md#changing-text) |
| print | `<value> ကို ပြသမည်။` or `<value> ဟု ပြသမည်။` | [Printing](#printing) |
| block | `အောက်ပါတို့ကို လုပ်ဆောင်မည်။` statements `လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။` | [Blocks](control-flow.md#blocks) |
| unsafe block | `အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။` statements `လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။` | [Unsafe Zaka](unsafe.md) |
| if | `အကယ်၍ <condition>လျှင် <block>`, then `[ထိုသို့မဟုတ်ဘဲ <condition>လျှင် <block>]`, then `[သို့မဟုတ်ပါက <block>]` | [If](control-flow.md#if) |
| switch | `<value> သည် <case> ဖြစ်လျှင် <block>`, then `[<case> နှင့် <case> တို့ ဖြစ်လျှင် <block>]`, then `[တခြား <kind> တစ်<measure word> ဖြစ်လျှင် <block>]` | [Enumerations](enums.md#switch) |
| while | `<condition> နေသရွေ့ <block>` | [While loops](control-flow.md#while-loops) |
| counted loop | `<from> မှ <to> အထိ [<step> စီ] <name> အနေဖြင့် ရေတွက်၍ <block>` | [Counted loops](control-flow.md#counted-loops) |
| each cell | `<list> ၏ အကွက်တိုင်း ကို [<name> နှင့်] <name> အနေဖြင့် ယူ၍ <block>` | [Each-cell loops](control-flow.md#each-cell-loops) |
| each character | `<text> ၏ စာလုံးတိုင်း ကို [<name> နှင့်] <name> အနေဖြင့် ယူ၍ <block>` | [Each-cell loops](control-flow.md#each-cell-loops) |
| each row of a table | `<table> ၏ အညွှန်းတိုင်း ကို …` or `<table> ၏ အတန်းတိုင်း ကို [<name> နှင့်] <name> အနေဖြင့် ယူ၍ <block>` | [Tables](tables.md#counting-and-walking-the-rows) |
| generic loop | `ကိန်းရှင် <name> သည် <kind> တစ်<measure word> [ဖြစ်၍ တန်ဖိုးသည် <value>]` or `ကိန်းရှင် <name> သည် <value>`, then `ဖြစ်ကာ <condition> နေသရွေ့` and a block with `[ပြီးလျှင် <statement>]` last | [Generic loops](control-flow.md#generic-loops) |
| break | `ကွင်းဆက် ကို ရပ်မည်။` | [Break and continue](control-flow.md#break-and-continue) |
| continue | `နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်။` | [Break and continue](control-flow.md#break-and-continue) |
| call | `<name> ဟူသော လုပ်ဆောင်ချက်ကို [<arguments> တို့ဖြင့်] လုပ်ဆောင်မည်။` | [Calls and returns](#calls-and-returns) |
| return | `<value> ကို ပြန်ပေးမည်။` or `ပြန်ပေးမည်။` | [Calls and returns](#calls-and-returns) |
| function | `<name> ဟူသော လုပ်ဆောင်ချက် သည် …` | [Functions](functions.md) |
| program start block | `ပရိုဂရမ်သည် …`, at most one | [Functions](functions-start-block.md) |
| add a cell | `<place> ထဲသို့ <value> ကို ထည့်မည်။` | [Lists](lists.md#adding-and-removing-cells) |
| add to text | `<text> ထဲသို့ <value> ကို ထည့်မည်။` | [Text](text.md#changing-text) |
| remove a cell | `<place> ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။` | [Lists](lists.md#adding-and-removing-cells) |
| set a row | `<table> ထဲမှ အညွှန်း <index> ၏ တန်ဖိုး ကို <value> ဟု ထားမည်။` | [Tables](tables.md#setting-and-removing-rows) |
| remove a row | `<table> ထဲမှ အညွှန်း <index> ကို ဖျက်မည်။` | [Tables](tables.md#setting-and-removing-rows) |
| replace part of text | `<text> ၏ အကွက် <start> မှ <end> ထိကို <value> ဖြင့် အစားထိုးမည်။` or `<text> ၏ <start> လုံးမြောက်စာလုံးမှ <end> လုံးမြောက်စာလုံးထိကို <value> ဖြင့် အစားထိုးမည်။` | [Text](text.md#changing-text) |
| heap, in unsafe code | `မင်မိုရီတွင် <size> နေရာယူ၍ လိပ်စာကို <place> တွင် သိမ်းထားမည်။` and two more | [Heap statements](#heap-statements) |

Some words have other spellings that mean the same: `လျှင်` or `ဖြစ်လျှင်`; `နေသရွေ့` or
`ဖြစ်နေသရွေ့`; `ဖြစ်ကာ` or `ဖြစ်ပြီး`; `ရေတွက်၍`, `တိုး၍` or `လျှော့၍` (these three differ
in direction); and the four spellings of the block closer, `ပြီးပါပြီ`, `ပြီးပါပြီး`,
`ပြီးဆုံးပါပြီ` and `ပြီးဆုံးပါပြီး`.

## The verb ends the sentence

A statement either starts with a reserved word (`ကိန်းရှင်`, `အကယ်၍`, `ကွင်းဆက်`, …) or
starts with an expression. A statement that starts with an expression must end in one of
these verbs:

| Verb | Statement | Particles on the operands |
| --- | --- | --- |
| `ပြသမည်` | print | `ကို` or `ဟု` after the value |
| `ထားမည်` | assign | `ကို` after the place, `ဟု` after the value |
| `ပြန်ပေးမည်` | return a value | `ကို` after the value, and after an error value, `<e> ဟူသော အမှားအယွင်း` |
| `နေသရွေ့` | while loop | none: it follows the condition |
| `မှ … အထိ … အနေဖြင့်` | counted loop | `မှ` after the first bound, `အထိ` after the second |

Each statement verb ends in `မည်` ("will"), while the declaration verb `ဖြစ်ပါစေ` ("let it
be") and the block closers keep their own spellings.

| Mistake | Code |
| --- | --- |
| a sentence that ends in another word; the help line lists the verbs | ZK0203 |
| a verb of the grammar, such as `ထားမည်`, finds a missing or wrong particle on an operand | ZK0206 |
| a verb of the grammar finds too few operands | ZK0205 |
| operands left over before the verb | ZK0209 |
| a missing `။` | ZK0201 |
| a verb of the core module, such as `ပြသမည်` or `ထည့်မည်`, in a sentence that fits none of its function signatures; the message lists the sentences of the verb | ZK0332 |
| a verb of the core module in a place where a value goes | ZK0204 |

The verbs of the core module are calls of functions, which is the reason a mismatch with
them gets ZK0332 rather than one of the grammar errors.

```zaka-error ZK0203 ZK0206
၁ ကို ပြသမည်။
၅ + ၃။
၂ ကို ပြသမည်။
က ဟု ၄ ကို ထားမည်။
```

```text
error[ZK0203]: this statement does not end in a statement verb, found "။"
help: a statement ends in "ပြသမည်", "ထားမည်" or "ကို ပြန်ပေးမည်", or is a loop with "နေသရွေ့" or "မှ ... အထိ"
error[ZK0206]: "ထားမည်" needs "ကို" after operand 1, found ဟု
```

The example shows how the compiler recovers from an error: it skips to the next `။` and
goes on, so one run reports the mistakes of several statements. A missing `။` joins the
statement with the next one in the report. A program with an error of this kind is not
checked for names and data kinds, and it does not run.

## Printing

```text
<value> ကို ပြသမည်။
"<text>" ဟု ပြသမည်။
<sentence> ဟု ပြသမည်။
```

`ပြသမည်` prints its operand and then a line break. A number prints in Burmese digits, a
true/false value as `မှန်` or `မမှန်`, a character and text as themselves, a list as its
cells between `[` and `]`, and a struct as its name and its fields. Text inside a list or
a struct prints in quotes. A template slot in a string literal writes its value with the
same rules ([Text](text.md#the-text-of-a-value)).

We write `ကို` after a plain value, a text variable included, and `ဟု` after a string
literal, after a `+` join that contains one, or after a whole sentence, which is a clause
that ends in a verb with a polite ending. The other particle still runs but gets a
warning, ZK0505 or ZK0506, and [Natural wording](naturalness.md) has the rules. A logical
expression needs parentheses (ZK0204).

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၄၂ ကို ပြသမည်။
"မင်္ဂလာပါ" ဟု ပြသမည်။
၃ သည် ၂ ထက် ကြီးသည် ဟု ပြသမည်။
'က' ကို ပြသမည်။
၁၊ ၂ နှင့် ၃ တို့ပါသော စာရင်း ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၂
မင်္ဂလာပါ
မှန်
က
[၁၊ ၂၊ ၃]
```

## Calls and returns

```text
<name> ဟူသော လုပ်ဆောင်ချက်ကို <arguments> တို့ဖြင့် လုပ်ဆောင်မည်။
<value> ကို ပြန်ပေးမည်။
ပြန်ပေးမည်။
```

To run a function and drop its result, we write the call as a statement, as in
`f ဟူသော လုပ်ဆောင်ချက်ကို ၁ နှင့် ၂ တို့ဖြင့် လုပ်ဆောင်မည်။`. Inside a function, we write
`<value> ကို ပြန်ပေးမည်။` to end the function with a value and `ပြန်ပေးမည်။` to end it
with none. A return outside a function is ZK0311, and [Results and returns](functions-results.md) has the
rest of the rules.

Statements run inside the [program start block](functions-start-block.md) and inside
functions. The top level of a file contains declarations alone, and a statement that runs
there is ZK0326: the declarations set their starting values first, and then the program
start block runs.

## Heap statements

Three statements ask for room on the heap, resize it and free it, and three more
write the cells of a block. Each one works on a pointer, which names the block, and each
one needs unsafe code, so outside it each is ZK0468. [Unsafe Zaka](unsafe.md) states their
rules, and these are the patterns of the first three:

| Statement | Pattern |
| --- | --- |
| allocate | `မင်မိုရီတွင် <kind> <n> <measure word>စာ နေရာယူ၍ လိပ်စာကို <place> တွင် သိမ်းထားမည်။` |
| resize | `လိပ်စာ <place> ရှိ မင်မိုရီကို <kind> <n> <measure word>စာ ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။` |
| free | `လိပ်စာ <place> ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။` |

For the size, we write the data kind, the count and the size word of the data kind, which
is its measure word with `စာ` after it, as in `ကိန်းပြည့် ၃ ခုစာ` and `စာလုံး ၅ လုံးစာ`
([The size of a data kind](kinds.md#the-size-of-a-data-kind)). The size can also be
`<n> ဘိုက်စာ`, a count of octets (bytes). `မင်မိုရီ` and `မှတ်ဉာဏ်` mean the same.

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
error[ZK0468]: a heap statement needs unsafe code
```

## Errors of statements

This table collects the errors of statements from this page and from the two pages after
it.

| Code | Raised when | Rules |
| --- | --- | --- |
| ZK0201 | a token is not the one expected, often a missing `။` | [The verb ends the sentence](#the-verb-ends-the-sentence) |
| ZK0203 | a sentence ends in no statement verb | [The verb ends the sentence](#the-verb-ends-the-sentence) |
| ZK0205, ZK0206, ZK0209 | a verb has too few operands, a wrong particle, or operands left over | [The verb ends the sentence](#the-verb-ends-the-sentence) |
| ZK0211, ZK0212, ZK0213, ZK0215, ZK0226 | a branch, an update, a single-statement block or a block closer in the wrong place | [Blocks, branches and loops](control-flow.md) |
| ZK0302, ZK0306, ZK0307, ZK0308 | a name declared twice, a constant or a counter assigned to, a constant with no value | [Declarations and assignment](declarations.md) |
| ZK0309, ZK0310, ZK0311 | a break, a continue or a return outside the loop or function it belongs to | [Break and continue](control-flow.md#break-and-continue), [Calls and returns](#calls-and-returns) |
| ZK0326 | a statement that runs at the top level of a file | [Calls and returns](#calls-and-returns) |
| ZK0402, ZK0405, ZK0409 | a condition that is no true/false value, a bound that is no integer, a step of zero or below | [Blocks, branches and loops](control-flow.md) |
| ZK0434 | an each-cell loop over a value it cannot walk | [Each-cell loops](control-flow.md#each-cell-loops) |
| ZK0468 | a heap statement outside unsafe code | [Heap statements](#heap-statements) |
| ZK0722 | a read of a variable, a field or a cell that a path reaches with no value given | [Set before read](declarations.md#set-before-read) |
| ZK0723 | a warning: a read of a counted list whose cells the check cannot prove set | [Set before read](declarations.md#set-before-read) |
| ZK0724 | a call of a function that reads a global that can be unset at the call | [Globals and calls](declarations.md#globals-and-calls) |
| ZK0437 | an add or a remove on a list that cannot grow or shrink, or `နောက်ဆုံးအကွက်` on text | [Lists](lists.md#adding-and-removing-cells) |
| ZK0439 | `၏ အရှေ့တွင်` after a value that is no cell and no character | [Lists](lists.md) |
