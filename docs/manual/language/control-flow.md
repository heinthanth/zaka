---
title: Blocks, branches and loops
description: Blocks, the if statement and each loop of Zaka - while, counted, each-cell and generic loops - with break and continue.
---

# Blocks, branches and loops

Control flow determines which statements run and in what order. This page covers blocks,
`if` branches, the four loop forms, and the statements that exit a loop or skip a round.
For a guided example, see [Control flow](../../learn/control-flow/README.md) in Learn.
For the rules of `switch`, see [Enumerations](enums.md#switch).

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

## Blocks

```text
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
<statements>
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

To group statements, we put them between `အောက်ပါတို့ကို လုပ်ဆောင်မည်။` ("do the
following") and the block closer (closing brace) `လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။`. Names
declared inside a block end at its block closer ([Names and scope](scope.md)), and a block
closer with no block to close is ZK0226. The block closer has four spellings that mean the
same: `ပြီးပါပြီ`, `ပြီးပါပြီး`, `ပြီးဆုံးပါပြီ` and `ပြီးဆုံးပါပြီး`.

To write an unsafe block, we open it with
`အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။` ("do the following, although it is
unsafe") and close it as a block. Inside it, pointer work and calls of unsafe functions
are allowed ([Unsafe Zaka](unsafe.md)). The program start block cannot be unsafe itself
(ZK0201), so we put an unsafe block inside it.

### A single statement in place of a block

Wherever a pattern on these pages has `<block>`, we can write a block or one single
statement. A single statement is a short form of a block of one statement:
`အကယ်၍ … လျှင် ၁ ကို ပြသမည်။`. A single-statement `<block>` cannot be an if itself
(ZK0215), because it would be unclear which if an `otherwise` belongs to, so we put the
inner if in a block.

```zaka-error ZK0215
အကယ်၍ မှန် ဖြစ်လျှင် အကယ်၍ မမှန် ဖြစ်လျှင် ၁ ကို ပြသမည်။
```

## If

```text
အကယ်၍ <condition>လျှင် <block>
ထိုသို့မဟုတ်ဘဲ <condition>လျှင် <block>
သို့မဟုတ်ပါက <block>
```

| Words | Reads as | Standard term |
| --- | --- | --- |
| `အကယ်၍ … လျှင်` | "if … then" | `if` |
| `ထိုသို့မဟုတ်ဘဲ … လျှင်` | "if not that, then if …" | `else if`, `elif` |
| `သို့မဟုတ်ပါက` | "otherwise" | `else` |

We use `အကယ်၍` to test a condition, which must be a true/false value (ZK0402). The first
branch whose condition holds runs, and the rest are skipped. We can add several
`ထိုသို့မဟုတ်ဘဲ` branches, and a last `သို့မဟုတ်ပါက` branch, with no condition, runs when
no other branch did.

We write `လျှင်` attached to the last word of the condition, as in `ငယ်လျှင်`. After a
name or a whole sentence, we write `ဖြစ်လျှင်` as a separate word instead. A
`ထိုသို့မဟုတ်ဘဲ` or `သို့မဟုတ်ပါက` with no if before it is ZK0211.

```zaka-error ZK0402
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ ၁ ဖြစ်လျှင် ၁ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## Loops

| Loop | Pattern | Standard term |
| --- | --- | --- |
| [while](#while-loops) | `<condition> နေသရွေ့ <block>` | `while` |
| [counted](#counted-loops) | `<from> မှ <to> အထိ [<step> စီ] <name> အနေဖြင့် ရေတွက်၍ <block>` | `for i in a..=b` |
| [each cell](#each-cell-loops) | `<list> ၏ အကွက်တိုင်း ကို <name> အနေဖြင့် ယူ၍ <block>` | `for x in list` |
| [generic](#generic-loops) | `ကိန်းရှင် <name> သည် <value> ဖြစ်ကာ <condition> နေသရွေ့ <block>` | C `for (init; cond; update)` |

Each loop tests or counts before its first round, so a loop can run zero times.
[Break and continue](#break-and-continue) work in each of them.

## While loops

```text
<condition> နေသရွေ့ <block>
```

A while loop works out the condition before each round and runs the `<block>` while the
condition is true. `ဖြစ်နေသရွေ့` is another spelling of the same verb, as in
`မှန် ဖြစ်နေသရွေ့`. The condition must be a true/false value (ZK0402).

## Counted loops

```text
<from> မှ <to> အထိ [<step> စီ] <name> အနေဖြင့် ရေတွက်၍ <block>
```

A counted loop declares a counter, `<name>`, that lives in the loop and runs the
`<block>` once for each value from `<from>` to `<to>`, both included.

- The bounds and the step are worked out once, before the first round, and must be
  integers (ZK0405).
- The counter has the common data kind of the two bounds, and the step must widen to it.
  A literal bound gets the data kind of the other bound.
- The step must be positive: a literal step of zero or less is ZK0409, and a zero step
  while the program runs stops the program.
- The `<block>` can read the counter but cannot assign to it (ZK0307).
- The counter and the names declared in the `<block>` share one scope, so the `<block>`
  cannot declare a name equal to the counter's (ZK0302).

The verb at the end sets the direction:

| Verb | Direction |
| --- | --- |
| `ရေတွက်၍` | toward `<to>`: up when `<from>` is not above `<to>`, down otherwise |
| `တိုး၍` | up in each case, with zero rounds when `<from>` is past `<to>` |
| `လျှော့၍` | down in each case, with zero rounds when `<from>` is below `<to>` |

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

The first loop shows that the bound is worked out once, because changing `ဆုံး` inside the
loop does not make the loop longer.

```zaka-error ZK0307
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ န ကို ၀ ဟု ထားမည်။
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

## Each-cell loops

```text
<list> ၏ အကွက်တိုင်း ကို [<number> နှင့်] <name> အနေဖြင့် ယူ၍ <block>
<text> ၏ စာလုံးတိုင်း ကို [<number> နှင့်] <name> အနေဖြင့် ယူ၍ <block>
```

To run a `<block>` once for each cell (element) of a list, first cell first, we write
`<list> ၏ အကွက်တိုင်း ကို <name> အနေဖြင့် ယူ၍ <block>`. To run it once for each character
of text, we write `စာလုံးတိုင်း` in place of `အကွက်တိုင်း`, and the loop walks the u8s of
the text once.

- The value before `၏` is worked out once, before the first round, and the loop walks a
  duplicate of it that shares the cells. So the `<block>` can change the list or the
  text, while the rounds still see the cells that it had at the start.
- `<name>` is a variable of the cell's data kind, or a `စာလုံး`, that lives for one round.
  Each round sets a fresh variable to the next cell or character, so assigning to it
  leaves the list as it was, and a nested function that uses it sees the value of that
  round ([Function values](function-values.md)).
- In unsafe code, a loop over a pointer to a list walks the list that the pointer points
  at, the same as a cell read does, and a list of pointers is walked like each other list.
- An each-cell loop over text, or an each-character loop over a value that is not text,
  is ZK0434.

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

```zaka-error ZK0434
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"abc" ၏ အကွက်တိုင်း ကို a အနေဖြင့် ယူ၍ a ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

### Two names, and the rows of a table

The word before `ကို` picks the part that each round gets. With two names,
`<number> နှင့် <name>`, the round gets the position of the item first and then the item:

| Part | Walks | One name gets | Two names get |
| --- | --- | --- | --- |
| `အကွက်တိုင်း` | a list | the cell | the cell number (index) from 0, a `usize`, then the cell |
| `စာလုံးတိုင်း` | text | the character | the character number from 0, a `usize`, then the character |
| `အညွှန်းတိုင်း` | a table (map) | the index (key) | the index, then the value |
| `အတန်းတိုင်း` | a table | the value | the index, then the value |

We can write `ကို` attached to the part word or after a space. A part that does not fit the
value is ZK0434 for a list or text and ZK0484 for a table, and the loop over the rows of a
table is on [Tables](tables.md#counting-and-walking-the-rows).

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
["နီ"၊ "ပြာ"] ၏ အကွက်တိုင်း ကို န နှင့် အရောင် အနေဖြင့် ယူ၍ "{န}: {အရောင်}" ဟု ပြသမည်။
"ကခ" ၏ စာလုံးတိုင်း ကို န နှင့် စ အနေဖြင့် ယူ၍ "{န}: {စ}" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၀: နီ
၁: ပြာ
၀: က
၁: ခ
```

## Generic loops

```text
ကိန်းရှင် <name> သည် <kind> တစ်<measure word> [ဖြစ်၍ တန်ဖိုးသည် <value>] ဖြစ်ကာ <condition> နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
<statements>
ပြီးလျှင် <update>။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

A generic loop is the C `for` loop in Zaka. It starts with a declaration, written in
either form of [Declarations](declarations.md#declarations), and the declaration runs
once. Then, while the condition holds, the block runs and then the update. We write the
update as the last statement of the block, after `ပြီးလျှင်` ("when done"), and it also
runs after `နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်`.

| Mistake | Code |
| --- | --- |
| an update that is not the last statement of the block | ZK0213 |
| a `ပြီးလျှင်` outside a generic loop | ZK0212 |
| a loop variable declared with `ကိန်းသေ` in place of `ကိန်းရှင်` | ZK0225 |

A generic loop needs a block, not a single statement, and its variable is gone after the
block closer. `ဖြစ်ကာ` can also be written `ဖြစ်ပြီး`.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ကာ ကိန်း သည် ၁၀ အောက် ငယ်နေသရွေ့
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
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

The second loop in the example is a while loop with `မှန်` as its condition, which runs
until a break leaves it.

## Break and continue

```text
ကွင်းဆက် ကို ရပ်မည်။
နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်။
```

To leave the innermost loop, we write `ကွင်းဆက် ကို ရပ်မည်။` ("will stop the loop", `break`).
To skip the rest of the round of the innermost loop and go on to the next round, we write
`နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်။` ("will skip to the next time", `continue`), and in a
generic loop the update runs first. Outside a loop, these are ZK0309 and ZK0310.

```zaka-error ZK0309
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကွင်းဆက် ကို ရပ်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```
