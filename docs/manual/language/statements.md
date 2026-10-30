---
title: Statements
description: Each form of statement in Zaka - declaration, assignment, printing, branches, loops, blocks, calls, returns and heap statements - with its grammar and its errors.
---

# Statements

A program is a list of *statements*, run from the first to the last. Each statement is
a Burmese sentence that ends with `။`, and the **verb** at the end of the sentence
decides what the statement does. A statement does something and has no value; an
[expression](expressions.md) has a value and sits inside a statement.

## The forms at a glance

In the shapes below, `NAME` is a name, `KIND` a [kind](kinds.md), `VALUE` and `COND`
expressions, `PLACE` something that can be assigned to (a name, or a cell or a pointer's
value), and `BODY` a block or one statement. Square brackets mark what can be left out.

| Statement | Shape |
| --- | --- |
| declare | `ကိန်းရှင် NAME သည် KIND [ဖြစ်၍ တန်ဖိုးသည်] [VALUE] ဖြစ်ပါစေ။` |
| constant | `ကိန်းသေ NAME သည် KIND [ဖြစ်၍ တန်ဖိုးသည်] VALUE ဖြစ်ပါစေ။` |
| assign | `PLACE ကို VALUE ဟု ထားပါ။` |
| print | `VALUE ကို ပြသပါ။` or `VALUE ဟု ပြသပါ။` |
| block | `အောက်ပါတို့ကို လုပ်ဆောင်ပါ။` statements `လုပ်ဆောင်စရာများ ပြီးပါပြီး။` |
| if | `အကယ်၍ COND လျှင် BODY` then `[ထိုသို့မဟုတ်ဘဲ COND လျှင် BODY]` then `[သို့မဟုတ်ပါက BODY]` |
| while | `COND နေသရွေ့ BODY` |
| counted loop | `FROM မှ TO အထိ [STEP စီ] NAME အနေဖြင့် ရေတွက်၍ BODY` |
| generic loop | `ကိန်းရှင် NAME သည် KIND [VALUE] ဖြစ်ကာ COND နေသရွေ့` block with `[ပြီးလျှင် STATEMENT]` last |
| break | `ကွင်းဆက် ကို ရပ်ပါ။` |
| continue | `နောက်တစ်ကြိမ်ဆီသို့ ကျော်ပါ။` |
| call | `လုပ်ဆောင်ချက် NAME ကို [ARGS တို့ဖြင့်] လုပ်ဆောင်ပါ။` |
| return | `VALUE ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။` or `ပြန်ပေးပါ။` |
| function | `လုပ်ဆောင်ချက် NAME သည် ...` (see [Functions](functions.md)) |
| heap | `မင်မိုရီတွင် SIZE နေရာယူ၍ လိပ်စာကို PLACE တွင် သိမ်းထားမည်။` and two more (below) |

Some words have alternatives: `လျှင်` or `ဖြစ်လျှင်`; `နေသရွေ့` or `ဖြစ်နေသရွေ့`;
`ဖြစ်ကာ` or `ဖြစ်ပြီး`; `ရေတွက်၍`, `တိုး၍` or `လျှော့၍`; the four spellings of the
block closer (`ပြီးပါပြီ`, `ပြီးပါပြီး`, `ပြီးဆုံးပါပြီ`, `ပြီးဆုံးပါပြီး`).

## The verb ends the sentence

A statement either starts with a reserved word (`ကိန်းရှင်`, `အကယ်၍`, `ကွင်းဆက်`, ...)
or starts with an expression. A statement that starts with an expression must end in
one of these verbs:

| Verb | Statement | Particles on the operands |
| --- | --- | --- |
| `ပြသပါ` | print | the value takes `ကို` or `ဟု` |
| `ထားပါ` | assign | the place takes `ကို`, the value takes `ဟု` |
| `ရလဒ်အဖြစ် ပြန်ပေးပါ` | return a value | the value takes `ကို` |
| `နေသရွေ့` | while loop | none: it follows the condition |
| `မှ ... အထိ ... အနေဖြင့်` | counted loop | `မှ` after the first bound, `အထိ` after the second |

A sentence that ends in another word is ZK0203, with a help line that lists the verbs.
A verb that finds a missing or wrong particle on an operand is ZK0206, and one that finds
too few operands is ZK0205. Operands left over before the verb are ZK0209. A missing
`။` is ZK0201.

```zaka-error ZK0203 ZK0206
၁ ကို ပြသပါ။
၅ + ၃။
၂ ကို ပြသပါ။
၄ ပြသပါ။
```

```text
error[ZK0203]: this statement does not end in a statement verb, found "။"
help: a statement ends in "ပြသပါ", "ထားပါ" or "ရလဒ်အဖြစ် ပြန်ပေးပါ", or is a loop with "နေသရွေ့" or "မှ ... အထိ"
error[ZK0206]: "ပြသပါ" needs "ကို|ဟု" after operand 1, found no particle
```

The example shows how the compiler recovers: after an error in a statement it skips to
the next `။` and reads on, so one run reports the mistakes of several statements. A
missing `။` joins the statement with the next one in the report. A program with a
reading error is not checked for names and kinds, and does not run.

## Declarations

`ကိန်းရှင်` declares a variable, `ကိန်းသေ` a constant. The kind comes first. The value
can follow the kind at once, or after `ဖြစ်၍ တန်ဖိုးသည်` ("is, and its value is"),
which keeps the kind and the value apart. A variable without a value starts at the
[default](kinds.md#default-values) of its kind; a constant needs a value (ZK0308) and
can never be assigned to (ZK0306). The value is checked first, so it cannot use the
name that the statement declares (ZK0301), and it must fit the kind (ZK0401).

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။
ကိန်းရှင် အရွယ် သည် ကိန်းပြည့် ၂၀ ဖြစ်ပါစေ။
ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် ဖြစ်ပါစေ။
ကိန်းသေ အများဆုံး သည် ကိန်းပြည့် ၁၀၀ ဖြစ်ပါစေ။
အသက် ကို အသက် + ၁ ဟု ထားပါ။
အသက် ကို ပြသပါ။
အရွယ် ကို ပြသပါ။
အရေအတွက် ကို ပြသပါ။
အများဆုံး ကို ပြသပါ။
```

```text
၁၉
၂၀
၀
၁၀၀
```

```zaka-error ZK0306
ကိန်းသေ အများဆုံး သည် ကိန်းပြည့် ၁၀၀ ဖြစ်ပါစေ။
အများဆုံး ကို ၂၀၀ ဟု ထားပါ။
```

## Assignment

`PLACE ကို VALUE ဟု ထားပါ။` puts the value into the place. The value is worked out first
with the current contents, then stored, so `အသက် ကို အသက် + ၁ ဟု ထားပါ။` adds one. The
place is a variable, a cell of a list (`x ၏ အကွက် i`) or the value at a pointer
(`p ၏ တန်ဖိုး`). Another expression is ZK0410. A constant (ZK0306) and a counted-loop counter
(ZK0307) cannot be assigned to. The value must widen to the kind of the place
([Conversions](conversions.md)).

## Printing

`ပြသပါ` prints its operand and then a line break. A number prints in Burmese digits, a
true/false value as `မှန်` or `မမှန်`, a character as itself, and a list as its cells
between `[` and `]`. A string literal can be printed and nothing else (ZK0223).

The operand takes `ကို` when it is a plain value, and `ဟု` when it is a string or a whole
sentence (a clause that ends in a verb with a polite ending). The other choice still runs
and gets a warning, ZK0505 or ZK0506; [Natural wording](naturalness.md) has the rules.
A logical expression needs parentheses (ZK0204).

```zaka
၄၂ ကို ပြသပါ။
"မင်္ဂလာပါ" ဟု ပြသပါ။
၃ သည် ၂ ထက် ကြီးသည် ဟု ပြသပါ။
'က' ကို ပြသပါ။
၁၊ ၂၊ ၃ တို့ပါသော စာရင်း ကို ပြသပါ။
```

```text
၄၂
မင်္ဂလာပါ
မှန်
က
[၁၊ ၂၊ ၃]
```

## Blocks

A block is a run of statements between `အောက်ပါတို့ကို လုပ်ဆောင်ပါ။` and
`လုပ်ဆောင်စရာများ ပြီးပါပြီး။`. Names declared inside it end at the closer
([Names and scope](scope.md)). A closer with no opener is ZK0226.

Wherever the shape of a statement has `BODY`, a block or one single statement can stand.
A single statement is a short form of a block of one: `အကယ်၍ ... လျှင် ၁ ကို ပြသပါ။`. A
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
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၇၃ ဖြစ်ပါစေ။
အကယ်၍ အမှတ် သည် ၈၀ အောက် မငယ်လျှင် "က" ဟု ပြသပါ။
ထိုသို့မဟုတ်ဘဲ အမှတ် သည် ၆၀ အောက် မငယ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
"ခ" ဟု ပြသပါ။
"ကောင်းပါသည်" ဟု ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
သို့မဟုတ်ပါက "ဂ" ဟု ပြသပါ။
```

```text
ခ
ကောင်းပါသည်
```

```zaka-error ZK0215
အကယ်၍ မှန် ဖြစ်လျှင် အကယ်၍ မမှန် ဖြစ်လျှင် ၁ ကို ပြသပါ။
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

**Generic loop.** `ကိန်းရှင် NAME သည် KIND [VALUE] ဖြစ်ကာ COND နေသရွေ့` opens a block.
The declaration runs once; then, while the condition holds, the block runs and then the
update. The update is the last statement of the block, opened with `ပြီးလျှင်`
("when done"); it also runs after `နောက်တစ်ကြိမ်ဆီသို့ ကျော်ပါ`. An update that is not
last is ZK0213, and a `ပြီးလျှင်` outside a generic loop is ZK0212. The loop takes a
block alone, and its variable is declared with `ကိန်းရှင်`, never `ကိန်းသေ` (ZK0225). The
variable is gone after the closer.

```zaka
ကိန်းရှင် ဆုံး သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
၁ မှ ဆုံး အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ဆုံး ကို ၁၀ ဟု ထားပါ။
န ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
"---" ဟု ပြသပါ။
၁၀ မှ ၀ အထိ ၅ စီ န အနေဖြင့် လျှော့၍ န ကို ပြသပါ။
"---" ဟု ပြသပါ။
၅ မှ ၁ အထိ န အနေဖြင့် တိုး၍ န ကို ပြသပါ။
"ပြီး" ဟု ပြသပါ။
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
ကိန်းရှင် ကိန်း သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ကာ ကိန်း သည် ၁၀ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
အကယ်၍ ကိန်း % ၂ သည် ၁ နှင့် ညီလျှင် နောက်တစ်ကြိမ်ဆီသို့ ကျော်ပါ။
အကယ်၍ ကိန်း သည် ၆ ထက် ကြီးလျှင် ကွင်းဆက် ကို ရပ်ပါ။
ကိန်း ကို ပြသပါ။
ပြီးလျှင် ကိန်း ကို ကိန်း + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
ကိန်းရှင် ဂဏန်း သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
မှန် ဖြစ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ဂဏန်း ကို ဂဏန်း * ၂ ဟု ထားပါ။
အကယ်၍ ဂဏန်း သည် ၁၀၀ ထက် ကြီးလျှင် ကွင်းဆက် ကို ရပ်ပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
ဂဏန်း ကို ပြသပါ။
```

```text
၀
၂
၄
၆
၁၂၈
```

## Break and continue

`ကွင်းဆက် ကို ရပ်ပါ။` leaves the innermost loop. `နောက်တစ်ကြိမ်ဆီသို့ ကျော်ပါ။` skips
the rest of the round of the innermost loop and goes to the next round; in a generic
loop the update runs first. Outside a loop these are ZK0309 and ZK0310.

## Calls and returns

`လုပ်ဆောင်ချက် f ကို ၁၊ ၂ တို့ဖြင့် လုပ်ဆောင်ပါ။` runs a function and drops its result.
`VALUE ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။` ends a function with a value, and `ပြန်ပေးပါ။` ends it
with none. A return outside a function is ZK0311. [Functions](functions.md) has the rest.

## Heap statements

Three statements ask for room on the heap, change it and give it back. Each works on a
pointer, which names the block. [Pointers](pointers.md) and [Ownership and borrowing](ownership.md)
cover them; the shapes are:

| Statement | Shape |
| --- | --- |
| allocate | `မင်မိုရီတွင် KIND N ခုစာ နေရာယူ၍ လိပ်စာကို PLACE တွင် သိမ်းထားမည်။` |
| resize | `လိပ်စာ PLACE ရှိ မင်မိုရီကို KIND N ခုစာ ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။` |
| give back | `လိပ်စာ PLACE ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။` |

The size can also be `N ဘိုက်စာ`, a count of octets. `မင်မိုရီ` and `မှတ်ဉာဏ်` mean the
same.

```zaka
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၄၂ ဟု ထားပါ။
ပ ၏ တန်ဖိုး ကို ပြသပါ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ပ ကို ပြသပါ။
```

```text
၄၂
ဗလာ
```

## Errors of statements

| Code | Raised when |
| --- | --- |
| ZK0201 | a token is not the one expected, often a missing `။` |
| ZK0203 | a sentence ends in no statement verb |
| ZK0205, ZK0206, ZK0209 | a verb has too few operands, a wrong particle, or a surplus |
| ZK0211, ZK0212, ZK0213, ZK0215, ZK0226 | a branch, update, single-statement block or closer in the wrong place |
| ZK0302, ZK0306, ZK0307, ZK0308 | a name declared twice, a constant or counter assigned to, a constant with no value |
| ZK0309, ZK0310, ZK0311 | break, continue or return outside its home |
| ZK0402, ZK0405, ZK0409 | a condition that is no true/false value, a bound that is no integer, a step of zero or below |

```zaka-error ZK0307
၁ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ န ကို ၀ ဟု ထားပါ။
```

```zaka-error ZK0402
အကယ်၍ ၁ ဖြစ်လျှင် ၁ ကို ပြသပါ။
```

```zaka-error ZK0309
ကွင်းဆက် ကို ရပ်ပါ။
```

```zaka-stop
ကိန်းရှင် ခြေလှမ်း သည် ကိန်းပြည့် ဖြစ်ပါစေ။
"အစ" ဟု ပြသပါ။
၁ မှ ၃ အထိ ခြေလှမ်း စီ န အနေဖြင့် ရေတွက်၍ န ကို ပြသပါ။
```

```text
အစ
the program stopped: counted loop step is not positive
```
