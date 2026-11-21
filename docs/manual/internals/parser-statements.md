---
title: Statements and declarations
description: How the parser picks the form of a sentence and reads statements, loops, switches, blocks, functions, structs, enumerations, kind aliases, imports and the declarations the compiler marks.
---

# Statements and declarations

This page follows `parse_stmt` in `compiler/zakac_parser/src/stmt.rs`: how the parser
picks the form of each sentence (statement), and how it reads loops, switches, blocks,
function declarations, structs, enumerations, kind aliases (type aliases) and imports.
The input is the cursor at the first token of a sentence, and the output is one `Stmt`
node, or the one diagnostic of the sentence. [The parser](parser.md) describes the
engine underneath, and [Expressions and calls](parser-expressions.md) the values inside
these statements. The rules of the language are on [Statements](../language/statements.md)
and [Functions](../language/functions.md).

A small example from `tests/stmt.rs`: a function with three parameters and a result.

```text
နှစ်ဆတိုးပြီးပေါင်း ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ဂ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က + ခ + ဂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
```

`at_fn_decl` sees `<name> ဟူသော လုပ်ဆောင်ချက် သည်`, so `parse_fn_decl` reads the name,
`parse_params` reads the three parameters up to `လက်ခံပြီး`, `parse_fn_block` reads the
opener `အောက်ပါတို့ကို လုပ်ဆောင်ကာ` (`Kw::FnOpenRet`) and the result clause, then the
statements up to the block closer. The test checks the S-expression form of the node:

```text
(fn နှစ်ဆတိုးပြီးပေါင်း [isize က, isize ခ, isize ဂ] isize (block (return (+ (+ က ခ) ဂ))))
```

## How a sentence picks its form

`parse_program` calls `parse_stmt_recover` until `Eof`, and `parse_stmt` calls
`parse_stmt_node`, then `heap_node`, which turns a call of a heap function of the core
module into its heap statement (see
[Heap sentences](parser-expressions.md#heap-sentences)). `parse_stmt_node` first runs a
few tests that look ahead past the first token, then matches the reserved word at the
start, and sends each other sentence to `parse_expr_headed`.

The tests that run first, in this order:

| Test | Form | Goes to |
| --- | --- | --- |
| `at_kind_decl` | `<name> ဟူသော [marks] ဒေတာအစုအဝေး\|ဒေတာအမျိုးအစားကွဲ` | `parse_kind_decl`: a struct or an enumeration |
| `at_fn_decl` | `<name> [<…>] ဟူသော [marks] လုပ်ဆောင်ချက် သည်` | `parse_fn_decl`: a function |
| `at_builtin_kind` | a data kind name, `စာရင်း` or `ဇယား`, then `သည်` and the compiler mark or kind parameters | `parse_builtin_kind` |
| `at_trait_decl` | a name, its kind parameters, then `ဟူသည့်` | `parse_trait_decl` |
| `at_trait_block` | `ရှိစေရန်` before the full stop | `parse_trait_block` |
| `at_auto_rings` | `အလိုအလျောက် မင်မိုရီ` | `parse_auto_rings` |
| `at_lock` | `<obj> ကို (ခေတ္တ\|ခဏ)ယူ၍` | `parse_lock` (in `threads.rs`) |
| `at_select` | `အောက်ပါ ဒေတာလမ်းကြောင်းများထဲမှ ဦးစွာရောက်လာသော ဒေတာကို စောင့်မည်။` | `parse_select` (in `select.rs`): the arms of the channels, the closed arm (`closed_arm`, its words joined, so glued or apart), then the arm of a time limit (`timeout_unit`) or of no data (`at_idle_arm`); ZK0264 and ZK0265 |

Then the reserved word at the start:

| First word | Goes to | Notes |
| --- | --- | --- |
| `အကယ်၍` | `parse_if` | with its `ထိုသို့မဟုတ်ဘဲ` and `သို့မဟုတ်ပါက` arms |
| `ထိုသို့မဟုတ်ဘဲ`, `သို့မဟုတ်ပါက` | an error | an else with no `if`, ZK0211 |
| `ကိန်းရှင်`, `ကိန်းသေ` | `parse_decl_or_loop` | a declaration or a generic loop |
| `ပရိုဂရမ်` | `parse_fn_decl` | the program start block (entry point) |
| `ဒေတာအမျိုးအစား` (`Kw::DataKindOpen`) | `parse_alias_decl` | a kind alias |
| `အများသုံး` | `parse_exported` | a variable, constant or kind alias that a module exports |
| `စိတ်မချရသော`, `ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော`, then `ပရိုဂရမ်` | `parse_marked_start` | the marks of a function on the program start block: ZK0257, ZK0256 (and ZK0229 for `အများသုံး`) |
| a block opener, plain or unsafe | `parse_block` | opener, `။`, statements up to the block closer, the block closer, `။` |
| `ကွင်းဆက်` | `parse_break` | `ကွင်းဆက် ကို ရပ်မည်။` |
| `မင်မိုရီရှိ` (`Kw::Mem`, `at_rings_collect`) | `parse_rings_collect` | a run of the ring collector |
| `နောက်တစ်ကြိမ်ဆီသို့` | `parse_continue` | `နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်။` |
| `ပြန်ပေးမည်`, `လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်` | `parse_bare_return` | a return with no value |
| `လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်` | `parse_mark_failed` | a `ReturnStmt` with `fault` set and no value |
| `ပြီးလျှင်` (`Kw::WhenDone`) | an error | an update outside a generic loop, ZK0212 |
| a block closer, or the block closer of the program start block | an error | no opener, ZK0226 |
| each other token | `parse_expr_headed` | see the next section |

Three marks (`FN_MARKS`) can come before `ပရိုဂရမ်` and still parse:
`at_marked_start` sees them, `parse_marked_start` reads the program start block and
reports each mark, since none of them can mark the start of a program.

## Sentences that start with a value

`parse_expr_headed` handles each sentence whose first word is no statement word. It looks
for three forms of import first:

| Form | Function | Node |
| --- | --- | --- |
| `"<file>" မှ <names> (ကို\|တို့ကို) ထည့်သွင်းမည်။` | `parse_import` | an `ImportStmt` of the names |
| `"<file>" ကို ထည့်သွင်းမည်။` or `"<file>" ကို ပြန်လည် မျှဝေမည်။` | `parse_whole_import` | a whole import (`ImportStmt.whole`) or a whole share (`ImportStmt.share`) |
| `<names> (ကို\|တို့ကို) အများသုံးနိုင်ရန် ပြန်လည် မျှဝေမည်။` | `parse_share` (`at_share`) | a `ShareStmt` of imported names |

A string with `မှ` after it opens an import unless `preposition_end` finds a preposition
call there, since the string can also be the first argument of such a call. Each import
and share must come first in its file: after another statement or inside a block it is
parsed and reported (`check_import_place`, ZK0230).

For each other sentence, `parse_expr_headed` runs the clause engine in statement mode, and
`finish_expr_headed` looks at what it returned:

| Clause result | Next token | Statement |
| --- | --- | --- |
| a statement | | the statement a statement verb closed (an assignment, a return, a call statement, a leave) |
| a switch head | | `parse_switch` |
| a value | `နေသရွေ့` | a while loop, `parse_while_tail` |
| a value | `မှ ရရှိသော ဒေတာတိုင်း ကို` | a loop over a channel, `parse_receive_each` |
| a value | `မှ` | a counted loop, `parse_counted_tail` |
| a value | `၏ <part>တိုင်း` | an each loop, `parse_each_tail` |
| a value | each other token | ZK0203, which lists the endings a statement can have |

The value is first joined with the logical words after it (`parse_logical_rest`), so the
condition of a while loop can be a logical expression. The forms of each loop:

```text
<cond> နေသရွေ့ <block>
<from> မှ <to> အထိ [<step> စီ] <name> အနေဖြင့် [အပြိုင်] (ရေတွက်၍|တိုး၍|လျှော့၍) <block>
<value> ၏ (အကွက်တိုင်း|စာလုံးတိုင်း|အညွှန်းတိုင်း|အတန်းတိုင်း) ကို [<name> နှင့်] <name> အနေဖြင့် ယူ၍ <block>
<channel> မှ ရရှိသော ဒေတာတိုင်း ကို <name> အနေဖြင့် ယူ၍ <block>
```

A counted loop reads its mode with `count_mode` (`Kw::Counting`, `Kw::Increasing` or
`Kw::Decreasing`, as `CountMode`); `parse_step_and_var` tells the step from the name
through the word after the expression. The clause engine stops before the `၏` of an each
loop, and `each_part` maps the word before `ကို` to its `EachPart`: `Cell`, `Char`,
`Index` or `Row`. A loop over a channel is an each loop of `EachPart::Received`. Before the
mode of a counted loop or the `ယူ၍` of an each loop, `အပြိုင်` marks a parallel loop
(`eat_parallel`), and `အပြိုင်လုပ်ငန်း <n> ခုခွဲ၍` in its place gives the loop its count of
threads (`parse_threads`); see [Threads](parser-expressions.md#threads-channels-and-locks).

When `at_leave` sees `ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်` or `... ပြန်လည်ရုပ်သိမ်းမည်` after an
operand, `finish_leave` (`clause.rs`) builds `Stmt::Leave(LeaveStmt { value, grant,
span })`. The lexer joins `ခေါ်ဆိုခွင့်` with `ပြုမည်` (`Kw::WillGrant`) or
`ပြန်လည်ရုပ်သိမ်းမည်` (`Kw::WillRevoke`) into `Kw::AllowCall` or `Kw::RevokeCall`, the fused
spelling too, and `at_leave` looks past the `ကို` for `ပြင်ပ` (`Kw::Outside`) and `မှ`, so
the value is read alone. Because the clause engine builds it, the leave statement also
works as the inline block of an `if`.

A call statement ends in `finish_call_stmt`, which reads the pass-on
`၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်` when it is written (the words after the `၊`
are the phrase `PassFault`, and these set `CallStmt::pass`), then the full stop. Each form
of a call statement ends here: `လုပ်ဆောင်မည်` after a long call, and `မည်` or
`လုပ်ဆောင်မည်` after the other forms. The forms of a call are on
[Expressions and calls](parser-expressions.md#calls).

## Declarations and generic loops

```text
(ကိန်းရှင်|ကိန်းသေ) <name> သည် <kind> တစ်<measure word> [ဖြစ်၍ တန်ဖိုးသည် <value>] ဖြစ်ပါစေ။
(ကိန်းရှင်|ကိန်းသေ) <name> သည် <value> ဖြစ်ပါစေ။
ကိန်းရှင် <name> သည် <head> ဖြစ်ပြီး <cond> နေသရွေ့ <block>
(ကိန်းရှင်|ကိန်းသေ) <name> ထဲသို့ <value> ကို [အကုန်]ကူးထည့်မည်။
```

A declaration and a generic loop share the start `NAME သည် <head>`, so
`parse_decl_or_loop` reads the start once and lets the word after the head decide:
`ဖြစ်ပါစေ` (`Kw::Let`) makes a `DeclStmt`, and `ဖြစ်ပြီး` (`Kw::And`) a generic loop
(`parse_generic_tail`), which a constant cannot start (ZK0225). The head is read with
`parse_decl_head` (`decl_head.rs`): a data kind and its measure word (classifier)
(`counted_head`, through `parse_slot_kind`), with `ဖြစ်၍ တန်ဖိုးသည် <value>` for a value
(`parse_value_clause`), or a value alone (`value_head`), whose data kind the check takes
from it. A data kind with no measure word after it is ZK0234 (`needs_word`). A data kind with
its word and then `၏ အရွယ်အစား` (`size_at`) is a value, a size, and so is a data kind with
a `<classifier>စာ` ahead in the statement (`items_ahead`).

The generic loop can end its block with a `ပြီးလျှင်` update (`parse_loop_stmts`), and an
update that the block closer does not follow is ZK0213 (`check_update_last`).

When the declared name has `ထဲသို့` after it, `parse_decl_or_loop` goes back to the name
and `parse_decl_into` reads the sentence as the assignment of a duplicate, then makes a
`DeclStmt` whose start is the value and whose data kind is not written. A constant with
the compiler mark after `သည်` is a constant of the core module (`parse_builtin_const`, see
[Declarations the compiler marks](#declarations-the-compiler-marks)).

A declaration records the prepositions of a call through its name: a written function
kind with prepositions gives them (`note_preposition_value`), and with no written kind, a
start of `<fn> ဟူသော လုပ်ဆောင်ချက်` gives the prepositions of that function
(`note_preposition_addr`).

## Conditionals, blocks and branches

```text
အကယ်၍ [ကွန်ပိုင်းလုပ်ချိန်တွင်] <cond> (လျှင်|ဖြစ်လျှင်) <block>
ထိုသို့မဟုတ်ဘဲ <cond> (လျှင်|ဖြစ်လျှင်) <block>
သို့မဟုတ်ပါက <block>
```

`parse_if` reads the arms with `parse_if_chain` and `parse_if_arm`. With
`ကွန်ပိုင်းလုပ်ချိန်တွင်` after `အကယ်၍`, the conditional is a compile-time conditional
(`IfStmt.comptime`), which the loader settles (see
[Compile-time conditionals](parser-kinds.md#reading-the-imports)).

Blocks and branches share `parse_branch`. With an opener it calls `parse_block`; with no
opener it reads one sentence into a `BlockStmt { single: true }`. Inside an arm of a
conditional, that one sentence cannot itself start with `အကယ်၍` (ZK0215). `parse_block`
reads the opener (`BlockOpen`, or `UnsafeBlockOpen`, which sets `BlockStmt.unchecked`),
the statements up to the block closer (`parse_stmts_to_close`, `at_close`) and the block
closer (`parse_block_close`).

The parser counts the blocks around a statement in `depth`, which is how it reports a
program start block, an external function or a C function declared inside a block
(ZK0214) and still parses it. A function with statements inside a block parses as at the
top level, a `Stmt::Fn` among the statements of the block, and the check of names and
data kinds declares it in its scope.

## Switches

```text
<subject> သည် <case> ဖြစ်လျှင် <block>
<case> [၊ <case> နှင့် <case> တို့] ဖြစ်လျှင် <block>
တခြား <kind> တစ်<measure word> ဖြစ်လျှင် <block>
```

A switch has no verb row. When the clause engine in statement mode meets `ဖြစ်လျှင်`
with two operands on its stack, the first with `သည်` and the second with no particle,
`switch_head` returns the pair as `ClauseOut::Switch`, and `parse_switch` reads the first
arm from there. Each sentence after it that reaches `ဖြစ်လျှင်` before its full stop, with
no `သည်` before it and no statement word at its start, is one more arm (`at_switch_arm`,
`reaches_then_is`); an arm of two or more cases reads them as a list in words
(`parse_switch_arm`). `တခြား <kind> တစ်WORD ဖြစ်လျှင်` is the default arm
(`parse_switch_default`, which reads the data kind with `parse_slot_kind`), and an arm
after it is parsed and reported (ZK0236). The block of an arm (`parse_switch_branch`)
cannot be one sentence that is a conditional or another switch (ZK0215), so each arm
after it belongs to one clear switch. The S-expression is
`(switch subject case {..} case {..} else kind {..})`.

An arm can test a member, `[<name> ဟူသည့်] <member> အဖြစ်သို့ ပြောင်းလဲနိုင်လျှင်`
(`at_variant_arm`, `variant_arm_rest`); its one case is an `Expr::Variant` with the name.
A statement whose clause ends in the test of a member, `<subject> သည် … ပြောင်းလဲနိုင်`, with
`လျှင်` after it, starts a switch whose first arm is that test (`parse_variant_switch`).
`parse_enum_member` reads `ဒေတာအနေဖြင့် <kind> တစ်<word> ရှိသည့်` into `EnumMember::data`.

## Functions

```text
<name> [<kind parameters>] ဟူသော [<marks>] လုပ်ဆောင်ချက် သည်
    [<phrases> သုံး၍] [<parameters> လက်ခံပြီး]
    (အောက်ပါတို့ကို လုပ်ဆောင်မည်။ | အောက်ပါတို့ကို လုပ်ဆောင်ကာ <kind> တစ်<measure word> ကို ပြန်ပေးမည်။)
    <statements>
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
```

A function declaration starts with its name. `fn_head_at` (in `parser.rs`) tells
`at_fn_decl` whether `<name> [<…>] ဟူသော [<marks>] လုပ်ဆောင်ချက် သည်` starts here, and
`parse_fn_decl` reads it in this order:

1. the name, and the kind parameters in the notation form (`parse_notation_params`);
2. `ဟူသော`, then the marks of `FN_MARKS` (`အများသုံး`, `စိတ်မချရသော`,
   `ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော`), then `လုပ်ဆောင်ချက်` and `သည်`;
3. the C name of a function that code outside the program can call (`parse_c_name`, see
   below);
4. the kind parameters in the word form and the `သည်` after them (`parse_word_params`);
5. the compiler mark, or the phrases of the function signature (`parse_head_phrases`);
6. the rest, in `parse_fn_kinds`: an external or C function (`parse_external`), a
   function the compiler gives (`parse_builtin_fn`), or a function with statements
   (`parse_fn_rest`, then `parse_fn_block`).

While the parts of a generic function are read, its kind parameters are data kind names
(`add_kind_names`, `drop_kind_names`). `mark_fn` then sets the flag of each mark:
`FnDecl.exported` for `အများသုံး`, which a function inside a block cannot have (ZK0229);
`FnDecl.unchecked` for `စိတ်မချရသော`; and `FnDecl.comptime` for the comptime mark, which
needs a function with statements at the top level (ZK0256).

The program start block, `ပရိုဂရမ်သည် ...`, is a function signature with no name:
`parse_fn_decl` gives it the name `ပရိုဂရမ်` and `start: true`, and its statements end at
`ပရိုဂရမ် ပြီးဆုံးပါပြီး` (`ProgramClose`, a joined phrase) where other blocks end at
`BlockClose`.

`parse_fn_block` reads the opener: `အောက်ပါတို့ကို လုပ်ဆောင်မည်` (`BlockOpen`) for a
function with no result, or `အောက်ပါတို့ကို လုပ်ဆောင်ကာ` (`FnOpenRet`) and the result
clause. A notes block after the opener (`ဤနေရာတွင် သတိပြုရန်မှာ`) gives the kind
parameters their traits and names local data kinds (see
[Generics and traits](parser-kinds.md#generics-and-traits)). A program start block
declared inside a block is ZK0214.

### Phrases of the function signature

```text
<name> ဟူသော လုပ်ဆောင်ချက် သည် အစ စကားစု "<words>" [နှင့် အဆုံး စကားစု "<words>" တို့] ကို သုံး၍ ...
```

Right after `သည်`, `parse_head_phrases` reads the phrases of the function signature when these are written: `အစ စကားစု "<words>"` (`Kw::StartPhrase`), `အဆုံး စကားစု "<words>"`
(`Kw::EndPhrase`), or both joined with `နှင့်` (`parse_phrase_list`), then `ကို` or
`တို့ကို` (`expect_list_word`) and `သုံး၍`. The lexer joins the spaced spelling of each
phrase word after the phrases of `PHRASES` (`join_named_phrases`), so `အစ` and `အဆုံး`
alone remain names. A string that is no phrase, or that has `၏` in it, is ZK0258
(`parse_phrase`), and `check_phrases` reports a last `၏` preposition before an ending
phrase (ZK0249). The S-expression shows the phrases after the name:
`(fn ဖတ် {ဖိုင် … စာကို} [text လမ်း(ထဲမှ)] - (block))`. Before the parse, `fn_header`
records the phrases, the prepositions and the parameter count of each function signature
as `CallWords`, so a call can come before the declaration.

### Parameters and their markers

```text
<name> ဟူသော <kind> တစ်<measure word> ကို [ဒေတာ ပြောင်းလဲရန် | ဒေတာ ထည့်ပေးရန်] [အပိုင်အနေဖြင့်] [ဝိဘတ် "<preposition>" ကို သုံး၍] [လည်းကောင်း]
```

When `<name> ဟူသော` follows `သည်` (`at_param`), `parse_params` reads the parameters
(`parse_param_items`), each with `parse_param`: the name and the data kind
(`parse_name_and_kind`, `parse_slot_kind`), `ကို`, then the markers that
`parse_item_markers` reads in this order:

| Marker | Token | Field |
| --- | --- | --- |
| `ဒေတာ ပြောင်းလဲရန်` | `Kw::DataToChange` | `Param::edit`: a mutable parameter (`inout`) |
| `ဒေတာ ထည့်ပေးရန်` | `Kw::DataToFill` | `Param::fill`, which sets `Param::edit` too: a writable parameter (`out`) |
| `အပိုင်အနေဖြင့်` | `Kw::AsOwned` | `Param::owned` |
| `ဝိဘတ် "<preposition>" ကို သုံး၍` | `Kw::PrepositionWord`, the string, `Kw::Ko`, `Kw::Using` | `Param::preposition` |

With several parameters, each one ends in `လည်းကောင်း` (`err_as_well_as` otherwise), and
the clause ends in `လက်ခံပြီး`, or in `လက်ခံမည်` for an external function. Either each
parameter has a preposition or none has. The string of a preposition must lex as words a
preposition can have (`parse_words_string`, `preposition_words`, ZK0241), and
`check_preposition_places` checks where the prepositions are: a preposition that starts
with `၏` can mark the last parameter alone (ZK0249), and the first preposition cannot be
`ကို သုံး၍`, which starts the arguments-first call (ZK0241). The S-expression writes
`own` and `edit` before the data kind and the preposition after the name:
`own edit int* ပ(ကို)`.

`parse_item_markers` takes a flag, `fill_ok`: a function signature passes `true`, and a
function kind (`fn_kind.rs`) passes `false`, so `SigParam` has no `fill`, and
`ဒေတာ ထည့်ပေးရန်` in a function kind stops the item with ZK0201. A lone `ပြောင်းလဲရန်` or
`ထည့်ပေးရန်` (`Kw::ToChange`, `Kw::ToFill`) is no marker and stops the item too.

### The result clause

`parse_ret_clause` reads the result clause after `အောက်ပါတို့ကို လုပ်ဆောင်ကာ` into
`FnDecl::ret`: the written result kind and its measure word (`parse_slot_kind`, the word
kept in `Kind::classifier`), then `ကို`, an optional `အပိုင်အနေဖြင့်` (`FnDecl.owned_ret`),
and `ပြန်ပေးမည်`. The clause is a `RetClause { kind, owned }`. A clause with no measure
word is ZK0234. `FnDecl` has no field for the error kind: a function can fail when its
result kind is a result kind (see
[Result and optional kinds](parser-kinds.md#result-and-optional-kinds)), and not
otherwise. A function kind reads its result clause as a function signature does (`fn_kind.rs`).

### External and C functions

```text
<name> ဟူသော လုပ်ဆောင်ချက် သည် ပြင်ပမှလာသော လုပ်ဆောင်ချက်(တစ်ခု)? (ဖြစ်သည်။ | ဖြစ်၍ <parameters> (လက်ခံမည်။ | လက်ခံပြီး <result>) | ဖြစ်၍ <result>)
<name> ဟူသော လုပ်ဆောင်ချက် သည် "<lib>" မှလာသော လုပ်ဆောင်ချက် ...
```

`parse_fn_kinds` sends a function signature to `parse_external` when `ပြင်ပမှလာသော`
(`Kw::External`, which the lexer joins from `ပြင်ပ မှ လာသော`) or `"<lib>" မှ` (`at_lib`)
comes after `သည်`. `parse_origin` reads where the function comes from: the host, or a
shared lib, whose text it keeps in `FnDecl::lib` as a `StrArg` (`None` for the host form).
The lexer has no joined word for the lib form: the text is a `Str` token, then `မှ`
(`Kw::From`) and `လာသော` (`Kw::Coming`). `parse_one_function` reads
`လုပ်ဆောင်ချက်တစ်ခု`, or `လုပ်ဆောင်ချက်` with an optional `တစ်ခု`. Then comes `ဖြစ်သည်`
for a function with no parameters and no result, or `ဖြစ်၍` and the parameters (when
`at_param` sees `NAME ဟူသော`) ending in `လက်ခံပြီး` or `လက်ခံမည်`, and the result clause.
`FnDecl::external` is set, and the block has no statements. The S-expression of a C
function is `(external "libm" name [params] ret)`.

`parse_params(ends, rest)` takes `rest` for an external function: after parameters that
each end in `လည်းကောင်း`, `အခြားတန်ဖိုးများနှင့်အတူ` (`Kw::WithOtherValues`, which the lexer
joins from `Kw::OtherValues` and `နှင့်အတူ`, `Kw::Along`, written apart) before the end word
sets `FnDecl::variadic`, and the S-expression writes `...` among the parameters. A C
function can use it, and a host function that has it is ZK0334 in the check.

A function that code outside the program can call has its C name after `သည်`:

```text
<name> ဟူသော လုပ်ဆောင်ချက် သည် "<C name>" အမည်ဖြင့် ပြင်ပမှ ခေါ်ဆိုနိုင်သော လုပ်ဆောင်ချက်ဖြစ်၍ ...
```

`parse_c_name` reads it into `FnDecl::c_name`; the phrase with no C name before it is an
error.

## Structs and enumerations

```text
<name> ဟူသော [အများသုံး] [<threads mark>] [မျှဝေနိုင်သော] ဒေတာအစုအဝေး တွင် <fields> တို့ပါဝင်သည်။
<name> ဟူသော ... ဒေတာအစုအဝေး သည် [<mark> ဒေတာအစုအဝေး ဖြစ်၍] [<kind parameters>] ၎င်းတွင် <fields> တို့ပါဝင်သည်။
<name> ဟူသော [အများသုံး] ဒေတာအမျိုးအစားကွဲ တွင် <members> တို့ပါဝင်သည်။
```

`at_kind_decl` checks for `<name> ဟူသော`, then the marks (`past_marks` in `threads.rs`),
then `ဒေတာအစုအဝေး` (`Kw::StructWord`) or `ဒေတာအမျိုးအစားကွဲ` (`Kw::EnumWord`). The
check runs before each other form. `parse_kind_decl` reads the marks in order:
`အများသုံး` sets `exported`, the phrase `အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော`
(`eat_threads_phrase`) sets `StructDecl.threads`, and `မျှဝေနိုင်သော` (`Kw::Shareable`)
sets `StructDecl.shared` and requires a struct. A shared mark or the threads mark on a
struct with a C name or from outside, and the threads mark on an enumeration, is ZK0263
(`mark_clash`).

`parse_struct_decl` reads a struct after `ဒေတာအစုအဝေး`. With `တွင်` it reads the fields at
once. With `သည်` it first reads a mark (`parse_struct_mark`), each one closed with
`ဒေတာအစုအဝေး` and `ဖြစ်၍`:

| Mark | Meaning |
| --- | --- |
| `"<C name>" အမည်ဖြင့် ပြင်ပမှ ယူသုံးနိုင်သော` (`parse_struct_c_name`, `Kw::UsableOutside`) | gives the struct to C under that name (`StructDecl.c_name`) |
| `ပြင်ပမှ လာသော` (`Kw::External`) | a data kind from outside the program (`external_kind`) |
| `ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော` (`Kw::InCompiler`, `Kw::IncludedWith`) | a struct the compiler gives, in the core module |

Then come the kind parameters in words and `၎င်း`. A mark that does not go with kind
parameters is ZK0263 (`check_mark_params`), since C cannot declare a generic struct and a
data kind from outside is one struct of C. The fields follow as a list in words
(`parse_listed_fields`, `parse_fields`), each `NAME ဟူသော KIND တစ်WORD`
(`parse_field`), then `တို့ပါဝင်သည်` (`FieldsClose`), `ပါဝင်သည်` after one field, or the
`ပြီး` forms (`FieldsCloseAnd`) and a measure word clause (`parse_decl_end`,
`parse_classifier`): `မျိုးပြပစ္စည်းမှာ <classifier> ဖြစ်သည်`, where the measure word is a
name or the reserved `ကွက်`. A struct can end the measure word clause with its written-as
hint, `ဖြစ်ကာ ရေးသားရာတွင် <words> ဟု ရေးသားနိုင်သည်`.

A data kind from outside (`external_kind`) can end at once with `ဖြစ်သည်` (`Kw::Is`), a
declaration with no fields, or go on with `ဖြစ်၍ ၎င်းတွင်` to the fields of the C struct
and `တို့ပါဝင်သည်`. Its node is a `StructDecl` with `shared` and `external` set, and its
S-expression head is `external_struct`.

`parse_enum_decl` reads an enumeration after `ဒေတာအမျိုးအစားကွဲ` and `တွင်`: members with `၊`
between them, each one

```text
[ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်] [တန်ဖိုးအနေဖြင့် <value> ရှိသည့်] <name>
```

(`parse_enum_member`). The lexer joins the first mark into `Kw::InitialMark` and the
opener of the value into `Kw::ValueAs`, and `HavingRel` closes the value.
`EnumMember::initial` marks the one
[initial member](../language/enums.md#the-initial-member). Then come `တို့ပါဝင်သည်`, or
`ပါဝင်သည်` after a lone member, or the `ပြီး` forms and a measure word clause
(`EnumDecl::classifier`); `member_closers` lists these words.

A struct or an enumeration declared inside a block is parsed and reported (ZK0228,
ZK0235).

## Kind aliases

```text
ဒေတာအမျိုးအစား <alias> ကို <kind> ဟုလည်း မှတ်ယူနိုင်ပါသည်။
ဒေတာအမျိုးအစား <alias> ကို <kind> ဟုလည်း မှတ်ယူနိုင်ပြီး မျိုးပြပစ္စည်းမှာ <word> ဖြစ်သည်။
```

`parse_alias_decl` reads the alias name, its kind parameters (`<…>` in the sign form, or
`သည်` and the word form, `parse_word_params`), `ကို`, the data kind (`parse_kind`, with no
measure word), then `ဟုလည်း မှတ်ယူနိုင်ပါသည်` (`Kw::AliasClose`), or
`ဟုလည်း မှတ်ယူနိုင်ပြီး` (`Kw::AliasCloseAnd`) and a measure word clause
(`AliasDecl::classifier`). Inside a block it is parsed and reported (ZK0240). The
S-expression is `(alias NAME KIND)`. The loader puts the data kind in place of each use
(see [Kind aliases in the loader](parser-kinds.md#reading-the-imports)).

## Marks in front of declarations

The marks of a declaration sit in three places, depending on what is declared:

| Declaration | Where the marks go | Read with |
| --- | --- | --- |
| a variable, a constant, a kind alias | `အများသုံး` before the statement | `parse_exported`, which parses the next statement and sets `exported` |
| a function | `အများသုံး`, `စိတ်မချရသော`, `ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော` between `ဟူသော` and `လုပ်ဆောင်ချက်` | `parse_fn_decl` and `mark_fn` |
| a struct, an enumeration | `အများသုံး`, the threads phrase, `မျှဝေနိုင်သော` after `ဟူသော` | `parse_kind_decl` |

`parse_exported` reports the mark before another statement, or before a declaration
inside a block (ZK0229). An unsafe block opens with the joined token `UnsafeBlockOpen`,
which `parse_block` accepts in the place of `BlockOpen`, and which sets
`BlockStmt.unchecked`.

## Statements of the ring collector

`rings.rs` reads the statements that drive the ring collector. Each one starts with the
reserved word `မင်မိုရီ` (`Kw::Mem`) or `အလိုအလျောက် မင်မိုရီ`; the other words are read
from their text (`eat_words`, `eat_word`), glued or apart, so none of them is reserved and
each remains free to name a variable.

| Sentence | Act |
| --- | --- |
| `မင်မိုရီရှိ မလိုအပ်သော အရာများကို ရှင်းလင်းမည်။` | `RingsAct::Collect`, a run now (`parse_rings_collect`) |
| `အလိုအလျောက် မင်မိုရီ ရှင်းလင်းမှုကို ခေတ္တရပ်မည်။` | `RingsAct::Pause` |
| `အလိုအလျောက် မင်မိုရီ ရှင်းလင်းမှုကို ပြန်လည်စတင်မည်။` | `RingsAct::Resume` |
| `အလိုအလျောက် မင်မိုရီ ရှင်းလင်းမှုကို <octets> ဘိုက် နှင့်အထက်မှ လုပ်ဆောင်မည်။` | `RingsAct::Threshold`, with `RingsStmt::octets` |

The node is a `RingsStmt { act, octets, span }`. `at_rings_collect` checks for
`မင်မိုရီရှိ`, since `မင်မိုရီတွင်` opens a heap statement instead. The expression
`အလိုအလျောက် မင်မိုရီ ရှင်းလင်းသည့် အခြေအနေ` (`at_ring_stats`, `parse_ring_stats`) returns
the stats of the collector as the struct of the core module
`အလိုအလျောက်မင်မိုရီရှင်းလင်းမှုအခြေအနေပြကိန်းဂဏန်းများ`.

## Declarations the compiler marks

The core module declares some data kinds, traits, functions and constants that the
compiler itself provides. Each one has the compiler mark
`ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော` (`eat_builtin_mark`):

- a data kind, `Stmt::BuiltinKind` (`at_builtin_kind`, `parse_builtin_kind`): a name,
  `စာရင်း` or `ဇယား`, `သည်`, the mark, `[အများသုံး]` and `ဒေတာအမျိုးအစား`, then `ဖြစ်သည်`,
  or `ဖြစ်၍`, the kind parameters in the word form and `၎င်းကို` before the written-as
  hint; kind parameters before the mark are ZK0201;
- a trait, after `ဟူသည့်`, with `[အများသုံး]` after the mark;
- a function, `FnDecl.builtin` (`parse_builtin_fn`), with no block: the mark comes right
  after `သည်`, then `လုပ်ဆောင်ချက်ဖြစ်၍`, the kind parameters in the word form with
  `၎င်းသည်` (`parse_word_params_then_thi`), the phrases and the parameters
  (`parse_builtin_rest`); after the kind parameters the mark no longer counts, and the
  function signature goes on as one with statements;
- a constant, `DeclStmt.builtin` (`parse_builtin_const`):
  `ကိန်းသေ <name> သည် ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော <kind> တစ်<word> ဖြစ်သည်။`, with no
  value; the loader writes the value of the target in (`fill_facts`).

Next: [Expressions and calls](parser-expressions.md)
