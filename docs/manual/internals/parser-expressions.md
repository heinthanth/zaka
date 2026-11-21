---
title: Expressions and calls
description: How the parser reads operands, the reads after ၏, the forms of a call, the verbs and heap functions of the core module, failures and optionals, text, lists in words, ဇယား rows, prepositions and threads.
---

# Expressions and calls

This page covers the values inside a sentence (statement): the operands the clause engine
stacks, the reads that follow `၏`, the forms of a function call, and the forms that the
parser rebuilds into special nodes, such as the verbs of the core module and the heap
sentences. The input is the cursor at the start of an operand, and the output is an `Expr`
node, or a `Stmt` when a call closes the sentence. [The parser](parser.md) describes the
three tiers these forms plug into, and [Statements and declarations](parser-statements.md)
the sentences around them. The rules of the language are on
[Expressions](../language/expressions.md) and [Functions](../language/functions.md).

A call can be written in several forms, and each form builds the same `CallExpr`. The
tests of `tests/expr.rs` check the word form and the sign form against each other:

```text
ပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို ၁၊ (က နှင့် ခ ၏ ပေါင်းလဒ်) နှင့် ဂ + ၁ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်
ပေါင်း(၁၊ (က နှင့် ခ ၏ ပေါင်းလဒ်)၊ ဂ + ၁)
```

Both give `(call ပေါင်း 1 [(add က ခ)] (+ ဂ 1))`, where the brackets show the group in
parentheses.

## Operands

`parse_head` reads the first operand of a clause, and `parse_operand` each operand after a
particle. Each one first tries the forms that are calls: a call written as a sentence
with its arguments first (`<args> ကို သုံး၍ ...`), a call with no arguments
(`<fn> မည်`, `<fn> သည့် ရလဒ်`), a call that opens with a phrase, a call whose arguments are
clauses, and at the start of a statement the long call that ends the whole statement
(`parse_first`). Each other operand is a plain one (`parse_plain_operand`): a struct
literal, a notation expression, or a list literal, ဇယား literal or verbal join that
starts with one.

`parse_atom` in `notation.rs` reads the atoms of a notation expression:

| Atom | Function |
| --- | --- |
| a number, with an optional base and data kind | `parse_literal` ([Literals](parser-kinds.md#literals-with-a-data-kind)) |
| a character, `မှန်` or `မမှန်` | `parse_char`, `parse_bool` |
| a string literal | `parse_str` (`str_lit.rs`) |
| a name, the result variable `ရလဒ်` included | `parse_name` |
| `<enum>များထဲမှ <member>` | `parse_member` |
| `( expr )` | `parse_group` |
| `[a၊ b]`, `[]`, `[k: v၊ …]`, `[:]` | `parse_bracket_list`, `parse_bracket_table` |
| `စာရင်းအလွတ်` | `parse_blank_list` |
| `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`, `အလွတ်`, `ကနဦးတန်ဖိုး` | `parse_null` |
| `<kind> တစ်<measure word> ၏ အရွယ်အစား` | `parse_size_of` |
| `<kind> <count> <classifier>စာ` | `attempt_kind_size` |
| a function value, `<name> ဟူသော လုပ်ဆောင်ချက်` | `parse_fn_value` |
| a call in the long form or the sign form | `parse_call_expr`, `parse_sign_call` |
| `<struct>(<field>: <value>)` | `parse_call_struct_lit` (`struct_lit.rs`) |

A name with `(` right after it and no space between (the token's `joined` flag) is the
sign form of a call, `f(a၊ b)` (`at_sign_call`), and builds the same `CallExpr` as the
word form; a struct name, or `<name>(<field>:`, starts a struct literal instead
(`at_call_struct_lit`). A struct literal in words is a list of
`<field> အနေဖြင့် <value>` pairs, then `ရှိသော <struct>တစ်<classifier>` (`at_struct_lit`,
`parse_struct_lit`). A call writes each argument as a plain value; the parameters of the
function decide which ones it can change.

An operand that is a name with `ထဲမှ` after it is a member of an enumeration,
`Expr::Member` (`parse_member`). The set of struct names in the parser has the
enumeration names and their stems as well, and the alias names, which `Parser::new` finds
in each declaration (`declares_kind`: `NAME ဟူသော ... ဒေတာအစုအဝေး|ဒေတာအမျိုးအစားကွဲ`, and
`ဒေတာအမျိုးအစား NAME ကို`), so each of them can start a data kind. `is_struct_at` refuses a
name with `ထဲမှ` after it, which names a member instead. A member with `(<value>)` after it
keeps the value as its data (`MemberExpr::data`), and so does the words form
`ဒေတာအနေဖြင့် <value> ရှိသော <member>`, which `parse_struct_lit` reads as a struct literal of
one field up to `ရှိသော` and finishes in `variant_words`.

In a clause, a name with `အဖြစ်သို့` after it, or `<name> ဟူသည့် <name> အဖြစ်သို့`, is an
`Expr::Variant` (`at_variant_operand`); `ပြောင်းလဲနိုင်` (`CanBecome`) is the verb row of the
test, `IsVariant`, with the slots `သည်` and `အဖြစ်သို့`. After `ကို`, a name with `အဖြစ်` and
the conversion verb after it, and no measure word, is the member a read names
(`at_variant_target`), and the conversion becomes `VariantData`.

### Built-in data kind names and sizes

The names of the built-in data kinds reach the parser as identifiers. `builtin_at` reads
one as its base kind (`ကိန်းပြည့်` as `BaseKind::Int`, and so on) when the set of struct
names does not have it, so a struct, an enumeration, an alias or a kind parameter of that
name hides the built-in data kind. `starts_kind`, `plain_kind_len`, `at_sized_kind` and
`parse_core` all ask `builtin_at`, so the AST has the same data kind as before, and the
stages after the parser see no change. In a value's place, a built-in data kind name
starts a size (`ကိန်းပြည့် တစ်ခု ၏ အရွယ်အစား`), or is ZK0202.

A data kind at the start of an atom can also start a size operand,
`<kind> <count> <classifier>စာ` (`Expr::Size`): `attempt_kind_size` (`heap.rs`) reads the
data kind, the count as a notation expression and the `<classifier>စာ` word
(`parse_items_word`, which the lexer marks as `Kw::Items`), and puts back the cursor, the
diagnostics and the node number when one of them is missing. `parse_notation` closes its
expression with `ဘိုက်စာ` when that word follows (`octets_after`), so `n + ၁ ဘိုက်စာ` counts
`n + ၁` octets.

## Reads after ၏

`parse_notation` reads the `၏` reads of notation after its expression (`ei_readings` in
`clause.rs`), as operators that close the whole notation before them:

| Read | Built through |
| --- | --- |
| `၏ တန်ဖိုး`, `၏ လိပ်စာ` | the verb rows (`reduce`) |
| `၏ အကွက် <index>`, `၏ အကွက် <start> မှစ၍ <length> ကွက်` | `push_index`, a cell counted from 0 or a range |
| `၏ <field>` | `field_of` |
| `၏ <fn>(<args>)` | `method_of`: `x ၏ f(a)` is `f(x, a)` |

So `ပ + ၁ ၏ တန်ဖိုး` reads the value at `ပ + ၁`, and the nodes are the ones the clause
engine builds for the same words. A name after `၏` is a field unless a function or a
preposition has it, it spells a read (`စာလုံး`, `မိတ္တူ`) or `ခုမြောက်` follows it
(`field_name_at`), and a name in braces is a field in each case. A read with an operator after it,
and the range of a replace statement (`cell_starts_range`), are left to the clause. The
parts of a larger read get no read of their own (`parse_bare_notation`): the number and
the length of a cell, the number of a character, the end of a range and the index of a
row. The same goes for the name after `၏` that the clause engine takes as a field
(`bare_notation`, which `push_next_operand` sets). `x ၏ f` remains a field read, which the
check turns into a call when no field fits.

The characters of text are read in the clause engine. After an operand that has `၏`, the
name `စာလုံး` (matched on its text, `name_at`) and a number build `VerbOp::CharAt`
(`push_char_at`, counted from 0). After `<text> ၏ <number> ခုမြောက်`, the word `စာလုံး`
builds `VerbOp::CharNth` (`push_char_nth`, counted from 1), or with `မှ <end> ... ထိကို`
next, the range of characters from that number. Neither has a verb word, so both are
built in place as a `VerbExpr` (`char_verb`). `လုံးမြောက်` is a second spelling of
`ခုမြောက်` (`Kw::Ordinal`). `at_range_end` tells a range (`မှ` with a `ထိ` before the full
stop) from a counted loop, whose `<cell> မှ <end> အထိ` has no separate `ထိ`. The counts
`အကွက်အရေအတွက်`, `စာလုံးအရေအတွက်` and `အညွှန်းအရေအတွက်` are functions of the core module
that the clause engine reads right after `၏` as the rows `CellCount`, `CharCount` and
`IndexCount` (`verb_at`, through `BUILTIN_COUNTS` in the `role.rs` of the lexer).

The duplicate word `မိတ္တူ` is no reserved word. When the clause engine meets the name
`မိတ္တူ` after an operand that has `၏` (`at_dup_word`), `push_dup` turns the operand into a
duplicate (copy) of it (`VerbOp::DupOf`). A word `တစ်<classifier>လုံး` before the `၏`,
with the `၏` joined to it or not (`whole_at`, `whole_word`, `mark_whole`), marks the
operand as the whole of a value, and the duplicate is deep (`VerbOp::DeepDupOf`). The
measure word is kept on the node (`with_classifier`). The test of one object
`X နှင့် Y သည် တစ်ခုတည်း ဖြစ်…` is the comparison row of `Kw::SameOne` (`VerbOp::Same`),
with the slots `နှင့်` and `သည်`.

## Calls

| Form | Pattern | Read with | File |
| --- | --- | --- | --- |
| sign form | `f(a၊ b)`, `f()`, `f(a)?` | `parse_sign_call`, `parse_sign_tail` | `notation.rs` |
| long form | `f ဟူသော လုပ်ဆောင်ချက်ကို a နှင့် b တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်` | `parse_call_expr`, `parse_call_head` | `notation.rs` |
| long form, statement | `... လုပ်ဆောင်မည်။` | `parse_first`, `finish_using` | `clause.rs`, `using.rs` |
| arguments first | `a နှင့် b တို့ကို သုံး၍ f ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။`, or `... f မည်။` | `parse_using` | `using.rs` |
| no arguments | `f မည်`, `f သည့် ရလဒ်` | `parse_short_after`, `parse_bare_short` | `using.rs` |
| preposition call | `<arg> <preposition> ... f မည်` | `attempt_preposition_call` | `using.rs` |
| phrase call | `<starting phrase> <arg> ... f မည်` | `attempt_phrase_call` | `using.rs` |
| arguments that are clauses | `က နှင့် ခ ၏ ပေါင်းလဒ် ကို ပြ မည်` | `attempt_clause_call` | `clause_call.rs` |
| method form | `x ၏ f(a)` | `method_of` | `clause.rs` |

Each form ends as a statement or as a value. The long form names the function as
`<fn> ဟူသော လုပ်ဆောင်ချက်ကို`, then ends with `လုပ်ဆောင်မည်` (a statement) or
`လုပ်ဆောင်သော ရလဒ်` (a value). The short form names the function alone, then ends with
`မည်` (a statement) or `သည့် ရလဒ်` (a value). `finish_using` reads the end once the function
is read; a call that ends in `မည်` where a value goes is ZK0204.

### The long call and function values

`NAME ဟူသော လုပ်ဆောင်ချက်` is `Expr::FnAddr` (`at_fn_value`, `parse_fn_value`): the phrase
`ဟူသော လုပ်ဆောင်ချက်` after a name is one token, `Kw::FnValue`, so the word right after
the name tells a value from a call. Before `အမျိုးအစား` the lexer keeps `ဟူသော` apart
(`Kw::Called`, then `Kw::FnKind`), so `NAME ဟူသော လုပ်ဆောင်ချက် အမျိုးအစား တစ်ခု` is a
parameter or a field of the bare function kind. The same token, then `ကို`, opens a long
call: `at_long_call` looks one token past the `ကို`, and a verb there (`as_verb`, or the
`ပြင်ပ` of a leave) keeps the function value as the object of that verb, while each other
token makes a call. A call expression is read as an atom (`parse_atom`), and at the start
of a statement through `parse_first`.

The long forms read their function with `parse_long_callee`: a name, or a place in
parentheses that starts with a name (`parse_callee`), then `Kw::FnValue` and `ကို`. For a
group, `group_end` scans to its `)` first. The place goes into `CallExpr::place` or
`CallStmt::place` (`with_place`), and `callee` keeps its first name. `parse_call_args`
reads the arguments as a list in words, then `ဖြင့်` after one or `တို့ဖြင့်` after two or
more; an argument with a `နှင့်` inside it goes in parentheses.

A parameter or a variable declared with a function kind that has prepositions joins
`Parser::functions` under its name (`note_preposition_value`), so a preposition call
through it works. A variable declared with no data kind that starts as
`<fn> ဟူသော လုပ်ဆောင်ချက်` joins with the prepositions of that function
(`note_preposition_addr`). A written data kind wins over the starting value, so a kind
with no prepositions adds nothing. `fn_header` counts the prepositions of a function
signature outside parentheses alone, so the prepositions of a function kind in it do not
count.

### The arguments-first call and the call with no arguments

`parse_head` reads the first operand of a clause and hands it to `parse_using` when
`at_using_rest` sees `(၊ ARG)* (တို့)?ကို သုံး၍` ahead. The arguments after the first are
notation expressions, then comes `NAME ဟူသော လုပ်ဆောင်ချက်ကို` (long) or `NAME` (short).
The end decides the node: `လုပ်ဆောင်မည်` or `မည်` gives a `CallStmt`, which can be the
whole statement alone, and `လုပ်ဆောင်သော ရလဒ်(ကောင်း)` or `သည့် ရလဒ်(ကောင်း)` gives a
`CallExpr`, which the clause goes on from. `parse_short_after` reads a name with
`မည်` or `သည့် ရလဒ်` after it (`at_short_end`) as a call with no arguments, both where
`parse_head` reads a first operand and where `parse_using` reads an argument.

### Preposition calls

```text
<arg> <preposition> <arg> <preposition> ... <fn> (မည် | သည့် ရလဒ်)
```

`Parser::functions` has the functions that declare prepositions, with the prepositions of
each overload: `Parser::new` finds them in the function signatures of the file
(`fn_header`, `declared_functions`), and the loader passes the ones a file imports to
`parse_with`. `preposition_end` scans for the end of a call of such a function
(`call_end_at`), outside parentheses, past prepositions and notation words
(`stops_preposition_scan`); a value end with a preposition after it can be an argument, so
the scan goes on past it and keeps it when no later end comes. `parse_preposition_call`
then reads each argument, its preposition when it has one (`parse_arg_preposition`), and
a `၊` when written, up to that end. `attempt_preposition_call` keeps the call when its
prepositions are in the slots of the parameters of a function of that name
(`fits_words`); otherwise it puts back the cursor, the diagnostics and the node number,
and the sentence is read as before (`က ကို ခ ၌ရှိသော ဖတ် သည့် ရလဒ် ဟု ထားမည်။` remains a set).
The `"<file>" မှ` start of an import is an import unless `preposition_end` finds a
preposition call.

A preposition string is lexed, and its tokens are the preposition, so a preposition can
have several words (`preposition_words`). The parser derives the words of the
prepositions and the first words from `Parser::functions`. A preposition call starts at a first word, or at a `၏` with a function that declares prepositions and a call end after it,
and `parse_arg_preposition` reads the longest preposition of the callee that the next
words spell (`spells_at`, `take_words`). Each argument has one slot in `prepositions`,
`None` where it has no preposition; a slot is read where some overload declares a
preposition, and two bare arguments touch or have `၊` between them, so each argument ends
where the next operand starts. `fits_words` keeps an overload whose parameters have
prepositions in the same slots. Each word of a preposition is kept in its written spelling
(`written_form`), so `နဲ့` in a call is the preposition `နှင့်`, and the check lets `နှင့်`
fill a `ဖြင့်` of a function that declares no `နှင့်`. A `နှင့်` right before the word
that closes a long call is its `ဖြင့်` (`at_with_before_call`), and so is a `နှင့်` after the
digits of a base.

### Phrase calls

A phrase call is a call written as a sentence that opens with a phrase of the function
signature. `parse_head` tries it before the first operand, when the word is in
`Parser::start_words` or `Parser::first_words` (`attempt_phrase_call`).
`bare_phrase_call` reads the phrases, the name and the end of a function with no
parameters; otherwise `start_phrase_call` reads the longest starting phrase, the first
argument, and the rest through `parse_preposition_call` (`read_after_start`). A call that
does not read, or whose counts fit no function of its name (`fits_words`), puts back the
cursor, the diagnostics and the node number. In `parse_preposition_call`, the words
between the last argument (or its preposition) and the function name that spell an ending
phrase of the function are that phrase (`parse_end_phrase`), and the call keeps both in
`Phrases`.

### Calls whose arguments are clauses

When the short reading of a call written as a sentence does not fit, `parse_head` goes
back to the start of the clause and reads its arguments as whole clauses
(`clause_call_or`, then `attempt_clause_call` in `clause_call.rs`); at a word of a
starting phrase it does so before it reads the word as a name. It takes the functions
ahead that end such a call, the farthest first (`callees_ahead`), and for each function
signature lists the cuts of the words before it (`cuts_of`, `cut_args`, `arg_ends`): the
phrases where the signature has them, and each argument up to a spelling of its
preposition, or up to each token for a bare one. A cut is kept when each piece is one
clause and nothing more (`arg_reads`): the cursor sees an end of input at the end of the
piece (`Cursor::limited`), and the parse leaves the cursor, the diagnostics and the node
number as these were. One cut is read (`read_cut`); two are ZK0260, since the sentence then
has two meanings. The tests check, for example:

```text
က နှင့် ခ ၏ ပေါင်းလဒ် ကို ပြ မည်။               (call ပြ (add က ခ))
က နှင့် ခ ၏ ပေါင်းလဒ် နှင့် ဂ ကို တွဲ မည်။       (call တွဲ (add က ခ) ဂ)
```

## The verbs of the core module

The verbs `ပြသ`, `ထည့်`, `ကူးထည့်`, `ဖျက်` and `အစားထိုး` are functions of the core module
that the compiler gives, and a call of one is written as a sentence whose words the core
module declares. `VERB_FNS` in `verb_call.rs` lists them with `စောင့်`, `ပို့`, `ပိတ်`,
`ပေါင်းထည့်` and `နုတ်ယူ`, the verbs of threads, channels and atomic numbers. Their
prepositions are in the table of preposition calls (`with_core`), beside the functions of
the same names that a file declares, which add overloads.

A statement that ends in `<verb> မည်` is first read as that call (`verb_sentence`, from
`parse_head`), its arguments as whole clauses. `verb_node` (from `finish_using`) then turns
the call statement into the statement of its sentence, as the words of the call tell
(`form_of_words`):

| Words of the call | Node |
| --- | --- |
| `<value> ကို ပြသမည်`, `<value> ဟု ပြသမည်` | `Stmt::Print` with its `ကို` or `ဟု` |
| `<list> ထဲသို့ <value> ကို ထည့်မည်` | `Stmt::Add` |
| `<cell> ၏ အရှေ့တွင် <value> ကို ထည့်မည်` | `Stmt::Add` with `before` set |
| `<place> ထဲသို့ <value> ကို ကူးထည့်မည်`, with the ending phrase `အကုန်` for a deep one | the `AssignStmt` of a duplicate (`VerbOp::DupOf`, `VerbOp::DeepDupOf`) |
| `<list> ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်`, `... နောက်ဆုံးစာလုံးကို ဖျက်မည်` | `Stmt::Remove`, with `RemoveStmt::chars` for the character |
| `<table> ထဲမှ အညွှန်း <index> ကို ဖျက်မည်` | `Stmt::Remove` marked `row` (`row_read`) |
| `<text> ၏ အကွက် <start> မှ <end> ထိကို <value> ဖြင့် အစားထိုးမည်` | `Stmt::Replace` |
| `<text> ၏ <n> (လုံး\|ခု)မြောက်စာလုံးမှ <m> ...မြောက်စာလုံးထိကို <value> ဖြင့် အစားထိုးမည်` | `Stmt::Replace` counted in characters |

A call with other words, through a place, with kind arguments, or that passes a failure on
remains a call. The calls of `စောင့်`, `ပို့`, `ပိတ်`, `ပေါင်းထည့်` and `နုတ်ယူ` remain calls,
which the check and the lowering serve. The clause engine has no rows for these verbs: a
sentence of one that fits no signature is read as a call of it, which the check reports
(ZK0332, with the signatures of the verb as sentences from `core_builtin_fns`, or ZK0331).
A value form of one is ZK0315 (`finish_sentence_call`), since these functions return no
value.

`BUILTIN_VERBS` in the `role.rs` of the lexer maps each verb to its reserved word
(`ကူးထည့်` to `Kw::WillDupIn`) for the painting of the editors and for `verb_at`, which
also reads `<name> မည်` of a verb of the core module; a glued `ပြသမည်` splits into the name
and its end (`glued_parts`). `parse_decl_into` reads the duplicate form in a declaration
(see [Declarations](parser-statements.md#declarations-and-generic-loops)).

The other marked functions, the three counts, have no prepositions: the clause engine
reads them right after `၏` (`BUILTIN_COUNTS`). The check of names and data kinds calls a
function of the file of the same name in place of the sentence when one accepts the
operands (`own_verb`, `own_count`).

## Heap sentences

The heap sentences are calls of the functions of `stdlib/core/heap.zaka`, listed with
their counts of arguments in `HEAP_FNS` (`heap.rs`): `သိမ်းထား`, `ပြန်လည်စွန့်လွှတ်`,
`နေရာအချဲ့အချုံ့လုပ်`, `ဖြည့်`, `ကူးယူ` and `ပြောင်းထည့်`. `with_core` keeps their
prepositions, as it does for the verbs, and returns the ones the file sees, which no
function of the file hides (`Parser::core_heap`). `parse_stmt` turns a call statement of
one, with its count of arguments, into its node (`heap_node`):

| Function | Node |
| --- | --- |
| `သိမ်းထား` | `Stmt::Alloc` |
| `ပြန်လည်စွန့်လွှတ်` | `Stmt::Free` |
| `နေရာအချဲ့အချုံ့လုပ်` | `Stmt::Resize` |
| `ဖြည့်` | `Stmt::Fill` |
| `ကူးယူ` | `Stmt::Blit` |
| `ပြောင်းထည့်` | `Stmt::Blit` with `overlap` set, the `memmove` form of `ကူးယူ` |

A size argument (`Expr::Size`) gives its count and its data kind (`size_arg`); another
value counts octets. A call of one in a value's place is ZK0315 (`finish_sentence_call`).
The scan for the end of a sentence call goes past the words of a size, a data kind and a
read (`in_size`) and past a literal with its data kind, and `written_form` reads
`မှတ်ဉာဏ်` as `မင်မိုရီ`, so a preposition matches either spelling.

## Failures, optionals and results

The pass-on forms of a call set `CallExpr::pass`: the word form ends in `ရလဒ်ကောင်း`
(`CallGoodResult`), and the sign form has a `?` that touches the `)`. A call with neither
form is a plain call, and its value is a whole result.

The statement `<value> ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်` is a plain `ReturnStmt` whose value is
an error value: `ဟူသော အမှားအယွင်း` is the phrase `CalledFault`, a postfix of the operand
before it that `push_postfix` turns into the verb `make_fault` (`VerbOp::MakeFault`). So an
error value is an expression that a return, an assignment or an argument can use. The
plain return is the `WillReturn` row. The mark statements
`လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်` and `လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်` are
return statements: the first is a bare return, and the second a `ReturnStmt` with `fault`
set and no value. The S-expressions are `(call-pass f ..)`, `(make_fault v)` and
`(return-fault)`.

The tests of a result and of an optional are the verbs `is_good`, `is_fault`, `has_data`
and `lacks_data`, with the slot `Within`:

```text
<x> ထဲတွင် ရလဒ်ကောင်း ရှိ
<x> ထဲတွင် အမှားအယွင်း ရှိ
<x> ထဲတွင် ဒေတာ ရှိ
<x> ထဲတွင် ဒေတာ မရှိ
```

The reads `ထဲမှ ဒေတာ` and `ထဲမှ အမှားအယွင်း ဒေတာ` are postfixes: `data_read_at` in
`notation.rs` tells which of `VerbOp::Unwrap` and `VerbOp::UnwrapFault` begins at a
`ထဲမှ`, and `data_reads` wraps a value in each read that follows it, so a read binds
tighter than an operator. The clause engine does the same for an operand with no slot
through `push_postfix`, and `read_of_field` rebuilds `x ၏ f ထဲမှ ဒေတာ`, so the field comes
first and each read wraps it. `၊ မရှိလျှင် <prefix operand>` after `ထဲမှ ဒေတာ`
(`at_fallback`, `parse_fallback`, `push_fallback`) builds the verb `or_else` over the
optional; the fallback is one operand, so here too a read binds tighter than an operator.

`အလွတ်` is an atom, a `NullLit` with `blank` set (`parse_null`), the optional with no
data. `ကနဦးတန်ဖိုး` is the same node with `initial` set, the initial value of the data kind
its place needs, and the null pointer `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`, one token
`Kw::Null`, is the node with neither set. The `initial` field of a `NullLit` and of an
`EnumMember` is left out of an artifact when false.

## Text

A string literal is an expression, read in `str_lit.rs` wherever an operand starts, a print
included: a print of a literal goes through the clause engine as each other print does,
and `PrintStmt.value` is an `Expr`. `parse_str` turns the parts the lexer made into an
`Expr::Str(StrExpr)` whose pieces are `StrPiece::Text(String)` or
`StrPiece::TemplateSlot(Expr)`, and joins text next to text into one piece (`push_piece`).
`parse_template_slot` lexes the text of each template slot with `lex_at` at its own
offset, so spans point inside the literal, and parses it as one expression with an inner
`Parser` that continues the node numbers of the file. A template slot with nothing in it
is ZK0202, and text left after the expression is the "expected `}`" error. The lexer ends
a template slot at its matching `}`, so a template slot can contain a template literal,
and `parse_template_slot` reads that literal as each other one: the nesting needs nothing
more.

The verbal join `a နှင့် b တို့ကို ဆက်ထားသည့် စာသား` is read as a list literal is: after the
first piece, a `၊` or `နှင့်` with `တို့ကို ဆက်ထားသည့်` ahead (`at_join_rest`,
`at_join_close`) starts it, and `parse_join` reads the pieces as a list in words, then
`တို့ကို`, `ဆက်ထားသည့်` and `စာသား`, into a `VerbExpr` with `VerbOp::Join`.

## Lists in words

Each list written in words goes through one helper in `word_list.rs`: the cells of a list
or a ဇယား, the pieces of a join, the pairs of a struct literal, the arguments of a long
call or of an arguments-first call, imported names, fields, members and switch cases.

```text
<item>
<item> နှင့် <item>
<item>၊ <item> နှင့် <item>
```

`parse_word_list` (or `parse_word_list_from`, after a first item read before) reads the
items with `၊` between them and `နှင့်` before the last (`word_list_rest`), and ZK0245
reports a list of two or more without the `နှင့်`, or with a `၊` after it.
`expect_list_word` then reads the closing word of the count, such as `ကို` after one item
and `တို့ကို` after two or more, and the word of the other count is ZK0246.

While an item is read, `Parser::in_word_list` is set, and the clause loop ends the item at
a `နှင့်` outside parentheses (`at_item_end`), and at `ဖြင့်` after a lone operand right
before the end of a long call, which closes its one argument. A group, the arguments of a
sign-form call and a callee place in parentheses clear the flag (`outside_word_list`), so
an item that joins its own operands with `နှင့်`, as `(က နှင့် ခ ၏ ပေါင်းလဒ်)` does, goes in
parentheses. A switch arm of two or more cases starts at its `နှင့်` too, with the
then-word ahead.

The scan that tells a list from a clause, `word_list_ahead` (and `word_list_end`), walks the
tokens outside parentheses and brackets to the closing word, and stops at a particle, `၏`
or the full stop, which no item has (`stops_item_scan`). So `က နှင့် ခ ၏ ပေါင်းလဒ်` is never
taken for the start of a list.

A list literal in words is `<cells> (ပါသော|တို့ပါသော) စာရင်း` (`at_list_rest`,
`parse_list`), with `ဟူသော <kind> တစ်<measure word>` after it for its data kind
(`parse_list_kind`).

## Table rows and literals

A row is a postfix of an operand, as `ထဲမှ ဒေတာ` is: `ထဲမှ အညွှန်း <notation>` (`at_row_read`,
`parse_row`) builds the verb `row`, and `၏ တန်ဖိုး` right after it makes it `row_value`. The
index is a notation expression, so it ends before the `၏`. The postfixes run in
`data_reads` after each atom, and again in the clause loop for an operand that a clause
built.

`[k: v၊ …]` and `[:]` are bracket literals, told from a list through the `:` after the
first cell (`parse_bracket_table`, `table_rest`).
`k အနေဖြင့် v နှင့် … တို့ပါသော ဇယား` is a sentence literal (`at_table_rest`, `parse_table`,
`table_row`), told from a counted loop through the `တို့ပါသော ဇယား` ahead.
`<row> ကို ဖျက်မည်` is a call of the verb `ဖျက်` that `verb_node` turns into a remove
statement marked `row` (see [The verbs of the core module](#the-verbs-of-the-core-module)).
An each loop reads its part, `အကွက်တိုင်း`, `စာလုံးတိုင်း`, `အညွှန်းတိုင်း` or `အတန်းတိုင်း`, and
a first name before `နှင့်` when there are two. The data kind of a ဇယား is on
[Data kinds, generics and imports](parser-kinds.md#table-kinds).

## Prepositions and glued ends

`glued.rs` splits a name token that is a function the file sees with a call end glued to
it (`ပြသမည်`) before the parse (`split_glued`, `split_token`), never a declared name
(`declared_at`), and warns about names that can clash (`clash_warnings`: ZK0250 for a
word of a preposition, ZK0251 for a glued call). The split runs against the functions the
file sees, not a list of suffixes, so `အမည်` ("name") remains a name unless the file sees
a function `အ`. A name in braces never splits. The call ends include the spoken ones
(`ပြသမယ်`), and `glued_spoken` reports each one as ZK0510, since the lexer saw a name.
Before `ရလဒ်` and `ရလဒ်ကောင်း`, `glued.rs` splits the short ends too
(`ENDS_BEFORE_RESULT`), so `စောင့်သော ရလဒ်` becomes the call `စောင့်` and its end.

Each file sees the functions of the core module (`with_core`). The functions of the
operator traits (`OPERATOR_FNS`) come without prepositions, because the word operators are
reserved words that the clause engine reads through its verb rows: `ပေါင်းလဒ်` and the
others, `စားသော စားလဒ်` and the joined phrases of the lexer, and the shifts, whose
`ဘယ်သို့|ညာသို့ <distance> ဘစ်` fills `Slot::Bit` (`push_shift`).

## Threads, channels and locks

The forms of threads live in `threads.rs`:

| Form | Read with | Node |
| --- | --- | --- |
| a long call that ends in `အပြိုင် လုပ်ဆောင်မည်` or `အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း` | `parallel_end`, `finish_parallel` | the call with `parallel` set, a statement or the value of the thread |
| `အပြိုင်` after `အနေဖြင့်` in a loop | `eat_parallel` | `parallel` on `CountedStmt` and `EachStmt` |
| `အပြိုင်လုပ်ငန်း <n> ခုခွဲ၍` in place of `အပြိုင်` | `parse_threads` | `threads` on `CountedStmt` and `EachStmt`, or a third argument of `VerbOp::ParMap` |
| `<list> ၏ အကွက်တိုင်း ကို <fn> ဟူသော လုပ်ဆောင်ချက် ဖြင့် အပြိုင် လုပ်ဆောင်သော ရလဒ်များ` | `at_par_map`, `parse_par_map` | `VerbOp::ParMap`, with the list and the name of the function |
| `<x> မှ ရရှိသော ဒေတာ` | `at_receive`, `parse_receive` | `VerbOp::Receive` |
| `<x> မှ ရရှိသော ဒေတာတိုင်း ကို <name> အနေဖြင့် ယူ၍ <block>` | `at_receive_each`, `parse_receive_each` | an each loop of `EachPart::Received` |
| `<name> ကို (ခေတ္တ\|ခဏ)ယူ၍ <block>` | `at_lock`, `parse_lock` | `Stmt::Lock` |
| `အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော` | `eat_threads_phrase`, `threads_trait_name` | the mark of a struct that threads can use, or the name of the trait of the data kinds threads can use |

`parallel_end` finds the end of a parallel call, the argument loop stops before it
(`call_word_at`), and the call node gets `parallel`. `ခုခွဲ၍` is read glued or apart. The
lock block is found before the statement is read as a clause, and `data_reads` reads the
receive, and the parallel map, as postfixes. The receive loop head is checked before the
head of a counted loop. The words of channels are no reserved words.

The room of a bounded channel, `<n> <measure word>` before the hint of the channel kind,
is a level suffix (`Suffix::Room`, `room_hint_at`, `parse_room`), kept in `Kind::room`;
`HINT_TWINS` lets a hint word come in a second spelling, `ပို့ဆောင်နိုင်သော` for
`ပို့နိုင်သော`. `အက်တမ်မစ် ဖြစ်သော <kind>` is read at the start of a data kind as the named
data kind `အက်တမ်မစ်` with one argument (`at_atomic_words` in `kind.rs`). The language rules
are on [Threads](../language/threads.md).

Next: [Data kinds, generics and imports](parser-kinds.md)
