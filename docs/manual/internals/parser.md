---
title: The parser
description: How the parser builds the tree from tokens, with its three expression tiers, the verb table, statements, the shape of the tree and how it skips a bad sentence.
---

# The parser

The parser takes the tokens of one file and builds a tree of statements, plus a list of
diagnostics. A sentence that fails is left out of the tree and reported, and the rest of
the file still parses. Two crates are involved: `zakac_ast` holds the tree and the table of
verbs, and `zakac_parser` holds the code that reads tokens.

The functions are in `compiler/zakac_parser/src/parser.rs`:

| Function | Use |
| --- | --- |
| `parse(&SourceFile, &LexOutput) -> ParseOutput` | the normal path |
| `parse_at(text, base, &LexOutput) -> ParseOutput` | the same for a bare `&str` at a position |
| `parse_lone_expr(text, base, &LexOutput) -> Result<Expr, Diagnostic>` | one expression that must use all the tokens; used in tests |
| `parse_with(&SourceFile, &LexOutput, first_id, &structs) -> ParseOutput` | one file of a program with imports: node numbers start at `first_id`, and the struct names the file imports start a kind |

`ParseOutput` is `{ program, diagnostics, next_id }`; `next_id` is the number the next
expression node would take. Spans are absolute, as in the lexer.

A program of several files goes through `load` in `src/load.rs`, which calls the
functions above once or twice for each file; see [Reading the imports](#reading-the-imports).

## The approach

The parser is hand-written, as a set of methods on one `Parser` struct. Each method
reads tokens from a `Cursor` and returns `PResult<T>`, a `Result` whose error is the one
diagnostic for the sentence being read. The cursor has `peek_n`, `bump` and `back(n)` but
no mark and reset: the grammar needs no guessing, so no method ever reads a token twice.
Where two forms share a start, the start is read once and the token after it decides.

Zaka sentences are verb-final, and each operand carries a particle that names its role:

```text
အသက် ကို   အသက် နှင့်  ၁ ကို ပေါင်းသည့်ရလဒ်  ဟု   ထားမည်။
 operand ကို  operand နှင့်  operand ၏   verb    ဟု  statement verb
```

A parser for C-like text reads an operator, then its right side. Here the verb comes
last, so the operands are met first and must wait. That is the reason for the middle tier
below: an operand stack that is reduced when a verb arrives.

An expression has three tiers, one module each, from the loosest to the tightest:

| Tier | Module | What it joins | Method |
| --- | --- | --- | --- |
| logical | `logical.rs` | clauses, with `ဖြစ်၍`, `သို့မဟုတ်`, `&&`, `\|\|` and postfix `မဟုတ်` | Pratt loop over clauses |
| clause | `clause.rs` | operands, each with a particle, closed with a verb | operand stack, run from the verb table |
| notation | `notation.rs` | atoms, with the C operators and prefix `- အနုတ် ~ !` | Pratt loop over atoms |

`parse_expr` is the top: it parses one clause, then loops on logical words. `parse_clause`
parses one notation expression, then loops on particles and verbs. `parse_notation` parses
one atom, then loops on operators. A parenthesized expression is an atom, which is how a
logical expression gets back inside a clause. A name with `(` right after it, with no space
(the token's `joined` flag), is the sign form of a call, `f(a၊ b)`, and builds the same
`CallExpr` as the word form; a struct name, or `<name>(<field>:`, is a struct literal
instead. A call writes each argument as a plain value; which ones the function changes is known
from its parameters.

```text
 parse_expr         clause ဖြစ်၍ clause သို့မဟုတ် clause မဟုတ်      logical.rs
    │
 parse_clause       operand:slot operand:slot ... verb              clause.rs
    │
 parse_notation     atom + atom * atom ...                          notation.rs
    │
 parse_atom         name  number  "(" parse_expr ")"  call  ...     notation.rs
```

### Logical tier

The binding powers are `မဟုတ်` (postfix) 25, `ဖြစ်၍` and `&&` 20 and 21, `သို့မဟုတ်` and
`||` 10 and 11, all left to right. Each operand of a logical word is a whole clause, so
`က သည် ခ နှင့် ညီသည် ဖြစ်၍ ဂ သည် ဃ ထက် ကြီးသည်` needs no parentheses. Before each step
the loop calls `mark_next`, which records on the left operand what token follows it
(`NextCtx`: `Or`, `Then`, `While`, and so on); the wording check uses that later.

### Notation tier

A standard Pratt loop with the powers of C:

| Operators | Power (left, right) |
| --- | --- |
| `==` `!=` | 30, 31 |
| `<` `>` `<=` `>=` | 40, 41 |
| `\|` | 50, 51 |
| `^` | 60, 61 |
| `&` | 70, 71 |
| `<<` `>>` | 80, 81 |
| `+` `-` | 90, 91 |
| `*` `/` `%` | 100, 101 |
| prefix `-` `အနုတ်` `~` `!` | 120 |

The loop stops at a token that is not an infix operator: a particle, a verb, a logical
word, `လျှင်`, `။`, `၊`, `)`. So an operand ends on its own, and `က + ၁ သည် ခ ...` reads
`က + ၁` as one operand. A comparison cannot take an unparenthesized comparison on either
side (ZK0210); the help suggests `ဖြစ်၍`.

### Clause tier

The engine is `parse_clause_from(first, mode)`, with a stack of `Item { operand, slot }`
that starts as `[first]`. It reads the next token and does one of four things.

1. **A slot particle.** The top item must still have no slot (else ZK0207). Record the slot
   on it, consume the particle, and read the next operand. After `ကို`, if a kind phrase
   starts and `အဖြစ်` follows it, the kind is pushed as the target of a conversion. If
   a verb comes next, nothing is pushed: the particle belonged to the operand before it.
2. **A shift word** (`ဘယ်သို့`, `ညာသို့`): read the distance, expect `ဘစ်`, push the
   distance with its slot.
3. **A verb.** Look up its row in `VERBS`, take one item for each slot of the row, check
   each item's slot against the row, and reduce them to one expression pushed back with no
   slot. For a statement verb the stack must then be blank, and the engine reads `။` and
   returns a statement.
4. **Each other token** ends the clause: one item alone, with no slot, is the result
   (else ZK0208).

The table is the one place the parser learns what a verb takes. `Slot` has sixteen forms:
the eight particles `နှင့် ၏ ကို ဖြင့် သည် ထက် အောက် ဟု`, `ခုမြောက်`, `ထဲသို့` (`Into`, the list of an add), `၏ အရှေ့တွင်` (`Before`, the cell or the character an add puts its value in front of; it fills the role of `Into`), `ထိကို` (`Upto`), `ထဲတွင်` (`Within`, the optional or the result a test looks in), the shift pair
(`Bit(Dir)`, which fits either direction), `KoOrHu` (`ကို` or `ဟု`, for a print) and
`KindAhpjit` (a kind then `အဖြစ်`). Three examples of rows (the real ones share constants such as `PAIR`):

```rust
row(Kw::Sum,   VerbOp::Add,   &[Slot::Hnint, Slot::Ei],            false, false),
row(Kw::Greater, VerbOp::Gt,  &[Slot::Thi, Slot::Htet],            false, true),
row(Kw::Assign, VerbOp::Assign, &[Slot::Ko, Slot::Hu],             true,  false),
```

The last two flags are `is_stmt` (the verb closes a statement: `ပြသမည်`, `ထားမည်`,
`ကို ပြန်ပေးမည်` (the row of `WillReturn`, with `[Ko]`), and `ထည့်မည်`, whose row takes `[Into, Ko]` and builds an `AddStmt`)
and `is_cmp` (a comparison, which cannot be the operand of another). The tests of an optional
or a result are rows too: `HasGood`, `HasFault`, `HasData` and `LacksData` take the slot
`Within` (`ထဲတွင်`). There are 34 rows
now. A new verb is a new row, and nothing in `clause.rs`
changes.

## One sentence, followed

Take `အသက် ကို အသက် နှင့် ၁ ကို ပေါင်းသည့်ရလဒ် ဟု ထားမည်။` ("set age to the sum of age and 1").
The first token is a name, so `parse_stmt` sends it to `parse_expr_headed`, which runs the
clause engine in statement mode. Letters stand for the slots: `K` is `ကို`, `H` is
`နှင့်`, `E` is `၏`, `U` is `ဟု`.

```text
parse_notation → အသက်                         stack: [အသက်]
ကို     slot K on the top, read an operand     stack: [အသက်:K  အသက်]
နှင့်    slot H, read an operand                stack: [အသက်:K  အသက်:H  ၁]
၏       slot E; a verb comes next, so          stack: [အသက်:K  အသက်:H  ၁:E]
        nothing is pushed
ပေါင်းလဒ်  row Sum, slots [H, E]: take two,
        both match, reduce to (add အသက် ၁)     stack: [အသက်:K  r1]
ဟု      slot U; a verb comes next              stack: [အသက်:K  r1:U]
ထားမည်    row Assign, slots [K, U]: take two,
        the stack is now blank, read ။,
        build the assignment, return it
```

`zakac parse` prints the tree. A second sentence, an `if`, comes after it; spans are
left out of this dump, which is otherwise as printed:

```text
program:
  stmts:
  - assign:
      target:
        name:
          id: 0
          name: အသက်
      value:
        verb:
          id: 3
          op: add
          args:
          - name:
              id: 1
              name: အသက်
          - int:
              id: 2
              value: 1
              base: 10
              kind: null
              script: burmese
          kind_arg: null
          dir: null
          particle: none
          next: other
          is_cmp: false
  - if:
      arms:
      - cond:
          verb:
            id: 6
            op: gt
            args:
            - name:
                id: 4
                name: အသက်
            - int:
                id: 5
                value: 18
                base: 10
                kind: null
                script: burmese
            kind_arg: null
            dir: null
            particle: none
            next: then
            is_cmp: true
        block:
          stmts:
          - print:
              value:
                expr:
                  name:
                    id: 7
                    name: အသက်
              slot: ko
          single: true
      else_block: null
diagnostics: []
```

Things to read from it:

- A statement is a variant of `Stmt` (`assign`, `if`, `print`, ...), and an expression a
  variant of `Expr` (`name`, `int`, `verb`, ...). Each variant wraps a struct of its own
  (`AssignStmt`, `VerbExpr`), so a pass can take `&VerbExpr` alone, and a form can grow a
  field without touching the enum.
- The last field of each struct is a `span`, which reaches the closing `။` for a
  statement. Each expression struct also has an `id`, a `NodeId`. Ids count up in the
  order nodes are built, so an inner node has a lower id than the node around it; later
  stages index their side tables (the kind of each expression) on ids.
- `verb.op` is a `VerbOp`, the operation (`add`, `gt`, ...); the operands are `args` in
  source order. The tree does not remember which spelling was written, but it keeps what
  the wording check needs: the `particle` the lexer folded into the verb and `next`, the
  context that came after it (`then` here: `လျှင်` came after the verb).
- `single: true` marks a block that is one sentence written without the block phrases.
- The integer literal holds its converted `value` (a `u128`), its `base`, an optional
  kind written after it (`ဟူသော <kind> တစ်ခု`) and the digit `script`. A float literal holds an `f64`.
- A name in a binding place (declaration, parameter, call target) is an `Ident`; a name
  used as a value is a `NameExpr`. Names are plain `String`s; there is no interner.

The one-line form used in the tests, `ToSexpr`, prints the same tree as
`(program (set အသက် (add အသက် 1)) (if (gt အသက် 18) (block (print အသက်))))`: operator
and verb names come from `strum`, and spans, ids and particles are left out. Most parser
tests assert against this form.

### Visitors

`zakac_ast::visit` has `Visitor` (it reads and changes nothing) and `VisitorMut`. `Visitor` has a `visit_*`
method per node struct, each defaulting to a `walk_*` function that visits the children in
source order, plus `enter` and `leave` hooks around each node. A pass overrides just what
it needs. The wording check and the name and kind check are visitors; `VisitorMut` moves
spans when an artifact is saved or loaded.

## Statements

`parse_program` loops `parse_stmt_recover` until `Eof`. `parse_stmt` looks at the first
token:

| First token | Goes to | Notes |
| --- | --- | --- |
| `အကယ်၍` | `parse_if` | arms of `ထိုသို့မဟုတ်ဘဲ` and `သို့မဟုတ်ပါက` are read here |
| `ကိန်းရှင်`, `ကိန်းသေ` | `parse_decl_or_loop` | shared start `NAME သည်`, then the head (`parse_decl_head` in `decl_head.rs`): `kind တစ်WORD` (`parse_slot_kind`), with `ဖြစ်၍ တန်ဖိုးသည် VALUE` for a value, or a value alone; a kind with no count word after it is ZK0234 (`needs_word`), and a kind with its word and then `၏ အရွယ်အစား` (`size_at`) is a value, a size; then `ဖြစ်ပါစေ` is a declaration and `ဖြစ်ပြီး` a generic loop |
| `လုပ်ဆောင်ချက်` | `parse_fn_decl` | a function declaration, `လုပ်ဆောင်ချက် NAME သည်`; a call never starts with this word |
| a name or a group, then `ဟူသော လုပ်ဆောင်ချက်ကို` | `parse_first` (`clause.rs`) | the long call at the start of a statement (`at_long_call`): `parse_call_head`, then `finish_using` ends it as a statement after `လုပ်ဆောင်မည်` or as a value the clause goes on from |
| `ပရိုဂရမ်` | `parse_fn_decl` | the start block: the header of a function with no name, which gets the name `ပရိုဂရမ်` and `start: true`; its statements end at `ပရိုဂရမ် ပြီးဆုံးပါပြီး` (`ProgramClose`, a joined phrase of `ပရိုဂရမ်` and the done word) where other blocks end at `BlockClose` |
| a block opener | `parse_block` | opener, `။`, statements until the closer, closer, `။` |
| `ကွင်းဆက်`, `နောက်တစ်ကြိမ်ဆီသို့` | break, continue | |
| `ပြန်ပေးမည်` | a bare return | |
| an operand, then `ကို`, `ပြင်ပ`, `မှ` and `ခေါ်ဆိုခွင့်` | `at_leave`, `finish_leave` (`clause.rs`) | the leave statement `f ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်` or `... ပြန်လည်ရုပ်သိမ်းမည်`: the lexer joins `ခေါ်ဆိုခွင့်` with `ပြုမည်` (`Kw::WillGrant`) or `ပြန်လည်ရုပ်သိမ်းမည်` (`Kw::WillRevoke`) into `Kw::AllowCall` or `Kw::RevokeCall`, the fused spelling too; `at_leave` looks past the `ကို` for `ပြင်ပ` (`Kw::Outside`) and `မှ`, so the value is read alone. The clause engine builds `Stmt::Leave(LeaveStmt { value, grant, span })`, so the statement also works as the inline block of an `if` |
| a call statement, then `၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်` | `finish_call_stmt` | sets `CallStmt::pass`; the words after the `၊` are the phrase `PassFault`. Each call statement form ends here: `လုပ်ဆောင်မည်` after `parse_first`, `မည်` and `လုပ်ဆောင်မည်` after the forms of `using.rs` |
| an operand, then `ကို သုံး၍` or `တို့ကို သုံး၍` | `parse_using` (`using.rs`) | the arguments-first call: `parse_head` in `clause.rs` reads the first operand of a clause and hands it over when `at_using_rest` sees `(၊ ARG)* (တို့)?ကို သုံး၍` ahead. The arguments after the first are notation expressions; then `NAME ဟူသော လုပ်ဆောင်ချက်ကို` (long) or `NAME` (short). The end decides the node: `လုပ်ဆောင်မည်` or `မည်` gives a `CallStmt`, and stands at the start of a statement alone; `လုပ်ဆောင်သော ရလဒ်(ကောင်း)` or `သည့် ရလဒ်(ကောင်း)` gives a `CallExpr`, which the clause goes on from |
| a name, then `မည်` or `သည့် ရလဒ်` | `parse_short_after` (`using.rs`) | the short call with no arguments, read where `parse_head` reads a first operand and where `parse_using` reads an argument |
| `ပြင်ပမှလာသော` after `လုပ်ဆောင်ချက် NAME သည်` | `parse_external` (`stmt.rs`) | the header of an external function: `လုပ်ဆောင်ချက်(တစ်ခု)?` (`parse_one_function`: the word `လုပ်ဆောင်ချက်တစ်ခု`, or `လုပ်ဆောင်ချက်` with an optional `တစ်ခု` name), then `ဖြစ်သည်` (nothing more) or `ဖြစ်၍`; parameters come when `at_param` sees `NAME ဟူသော` (`parse_params`, ending in `လက်ခံပြီး` or `လက်ခံမည်`), and the result clause after `လက်ခံပြီး` or alone. `FnDecl::external` is set and the block holds no statements. The lexer joins `ပြင်ပ မှ လာသော` into `External` |
| a text then `မှလာသော` after `လုပ်ဆောင်ချက် NAME သည်` | `parse_external`, through `at_lib` and `parse_origin` (`stmt.rs`) | the header of a C function: the same header as an external function, with the text of a lib where `ပြင်ပ` stood. The lexer has no joined word for it: the text is a `Str` token, then `မှ` (`Kw::From`) and `လာသော` (`Kw::Coming`). `parse_origin` gives `FnDecl::lib`, a `StrArg` of the text (`None` for the host form); the s-expression form is `(external "libm" name [params] ret)`. `parse_params(ends, rest)` takes `rest` for an external header: `အခြားတန်ဖိုးများနှင့်အတူ` (`Kw::WithOtherValues`, the lexer joins `Kw::OtherValues` and `နှင့်အတူ`, `Kw::Along`, written apart) after the last `လည်းကောင်း`, before the end word, sets `FnDecl::variadic` (the s-expression writes `...` among the parameters), and each parameter before it ends in `လည်းကောင်း`. A C function can use it; a host function that has it is ZK0334 |
| an operand, then a preposition | `attempt_preposition_call` (`using.rs`) | the preposition call. `Parser::functions` holds the functions that declare prepositions, with the prepositions of each overload: `Parser::new` finds them in the headers of the file (`fn_header`), and the loader passes the ones a file imports to `parse_with`. `preposition_end` scans for the end of a call of such a function, outside parentheses, past prepositions and notation words; a value end with a preposition after it can be an argument, so the scan goes on past it and keeps it when no later end comes. `parse_preposition_call` then reads argument, preposition, argument, preposition, up to that end, and `attempt_preposition_call` keeps the call when its prepositions count as the parameters of a function of that name; otherwise it puts the cursor, the diagnostics and the node number back, and the sentence reads as before (`က ကို ခ ၌ရှိသော ဖတ် သည့် ရလဒ် ဟု ထားမည်။` remains a set). The `"<file>" မှ` start of an import is read as an import unless `preposition_end` finds a preposition call |
| `မင်မိုရီတွင်` | `parse_alloc` | the allocation sentence |
| `လိပ်စာ` | `parse_heap` | `လိပ်စာ PLACE ရှိ`, then a size before `မင်မိုရီကို` for a blit (`parse_blit_end`: `ကူးယူမည်` or `ပြောင်းထည့်မည်`); else after `မင်မိုရီကို`, `ပြန်လည်စွန့်လွှတ်မည်` for a release, a fill (`parse_fill_end`) when `ဖြည့်မည်` comes before the `။` (`ends_in_fill`), or a size for a resize. A size with a kind is `KIND N WORDစာ` (`parse_size_arg`): the text of the count word before its `စာ` becomes the measure word of the unit kind, `Kind::classifier`. The value of a fill is read with `in_fill` set, so a `ဖြင့်` right after a lone value ends it |
| a string, then `မှ` | `parse_import` | `STR မှ NAME ကို`, or the names as a list in words then `တို့ကို`, then `ထည့်သွင်းမည်။`; after another statement or inside a block it is parsed and reported (ZK0230) |
| a name, then `ဟူသော`, the marks `အများသုံး`, `မျှဝေနိုင်သော` or `ပြင်ပမှ လာသော`, then `ဒေတာအစုအဝေး` or `ဒေတာအမျိုးအစားကွဲ` | `parse_kind_decl` | checked first (`at_kind_decl`); the marks set `exported`, `shared` and `external`. `ဒေတာအစုအဝေး` (`Kw::StructWord`) then `တွင်` reads a struct (`parse_struct_decl`): fields with `၊` between them, each `NAME ဟူသော KIND တစ်WORD` (`parse_field`), then `တို့ပါဝင်သည်` (`FieldsClose`) or `တို့ပါဝင်ပြီး` (`FieldsCloseAnd`) and the classifier clause (`parse_decl_end`). With `ပြင်ပမှ လာသော` (`Kw::External`) it reads a kind from outside the program (`parse_external_kind`): `ဖြစ်သည်` (`Kw::Is`) ends a declaration with no field, and `တွင်` goes on to the fields and `တို့ပါဝင်သည်`; the node is a `StructDecl` with `shared` and `external` set (s-expression head `external_struct`). `ဒေတာအမျိုးအစားကွဲ` (`Kw::EnumWord`) then `တွင်` reads an enumeration (`parse_enum_decl`): members with `၊` between them, each `[ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်] [တန်ဖိုးအနေဖြင့် VALUE ရှိသည့်] NAME` (`parse_enum_member`; the lexer joins the mark into `Kw::InitialMark` and the opener of the value into `Kw::ValueAs`, `HavingRel` closes it; `EnumMember::initial` marks the one [initial member](../language/enums.md#the-initial-member)), then `တို့ပါဝင်သည်`, or `ပါဝင်သည်` after a lone member, or the `...ပြီး` closers and a classifier clause (`EnumDecl::classifier`). A declaration inside a block is parsed and reported (ZK0228, ZK0235) |
| `အချက်အလက်အမျိုးအစား` | `parse_alias_decl` | the alias name, its kind parameters (`<…>`, or `သည်` and the word form, `parse_word_params`), `ကို`, the kind (`parse_kind`, with no measure word), then `ဟုလည်း မှတ်ယူနိုင်ပါသည်` (`Kw::AliasClose`), or `ဟုလည်း မှတ်ယူနိုင်ပြီး` (`Kw::AliasCloseAnd`) and a classifier clause (`AliasDecl::classifier`); inside a block it is parsed and reported (ZK0240). The S-expression is `(alias NAME KIND)` |
| `အများသုံး` | `parse_exported` | parses the next statement and sets `exported` on a function, kind alias or top-level declaration; on another statement or the start block it is ZK0229. A struct and an enumeration hold the mark after `ဟူသော` instead |
| an `else`, an update, a closer with no opener | an error | ZK0211, ZK0212, ZK0226 |
| all other tokens | `parse_expr_headed` | a clause; see below |

`parse_expr_headed` runs the clause engine in statement mode. If a statement verb closed
the clause, the statement is built. Otherwise the clause is a value, and the next token
decides: `နေသရွေ့` makes a while loop; `မှ` makes a counted loop (`from မှ to အထိ (step စီ)? NAME အနေဖြင့်
mode block`); `၏` before `အကွက်တိုင်း` or `စာလုံးတိုင်း` makes an each loop (`value ၏
(အကွက်တိုင်း|စာလုံးတိုင်း) ကို NAME အနေဖြင့် ယူ၍ block`, an `EachStmt`), and the clause engine
stops before that `၏`; all else is ZK0203, which lists the endings a statement can have.

A switch has no verb row either. When the clause engine in statement mode meets
`ဖြစ်လျှင်` with two operands on its stack, the first marked with `သည်` and the second with
no particle, `switch_head` gives the pair back as `ClauseOut::Switch`, and `parse_switch`
reads the first arm from there. Each sentence after it that reaches `ဖြစ်လျှင်` before its
full stop, with no `သည်` before it and no statement word at its start, is one more arm
(`at_switch_arm`); `တခြား KIND တစ်WORD ဖြစ်လျှင်` reads the default arm (`parse_slot_kind`), and an arm after it is
parsed and reported (ZK0236). A one-statement arm that is itself a switch is ZK0215. The
S-expression is `(switch subject case {..} case {..} else kind {..})`.

An operand that is a name with `ထဲမှ` after it is a member, `Expr::Member` (`parse_member`).
The parser's set of struct names holds the enumeration names and their stems as well, and
the alias names, which `Parser::new` finds in each declaration (`declares_kind`: `NAME ဟူသော ... ဒေတာအစုအဝေး|ဒေတာအမျိုးအစားကွဲ`, and `ဒေတာအမျိုးအစား NAME ကို`), so each of them
can start a kind; `is_struct_at` refuses a name with `ထဲမှ` after it, which
names a member instead.

The names of the built-in kinds reach the parser as identifiers. `builtin_at` reads one
as its base kind (`ကိန်းပြည့်` as `BaseKind::Int`, and so on) when the set of struct
names does not hold it, so a struct, an enumeration, an alias or a kind parameter of that
name hides the built-in kind. `starts_kind`, `plain_kind_len`, `at_sized_kind` and
`parse_core` all ask `builtin_at`, so the AST holds the same kind as before, and the
stages after the parser see no change. In a value's place, a built-in kind name starts
a size (`ကိန်းပြည့် တစ်ခု ၏ အရွယ်အစား`), or is ZK0202.

A remove has no verb row, since its list comes before `၏`. When the clause engine meets
`နောက်ဆုံးအကွက်` (`Kw::LastCell`) after an operand that holds `၏`, `finish_remove` takes
that operand as the list, expects `ကို`, `ဖျက်မည်` and `။`, and builds a `RemoveStmt`; in an
expression it is ZK0204, a statement verb inside an expression. `attach_slot` reads
no operand after the `၏` when `နောက်ဆုံးအကွက်` follows it. `နောက်ဆုံးစာလုံး`
(`Kw::LastChar`) takes the same path and sets `RemoveStmt::chars`.

When `အရှေ့တွင်` (`Kw::Before`) follows an operand that holds `၏`, `attach_before` turns
that `၏` into the slot `Before` and reads the value after it; the `ထည့်မည်` row then builds
an `AddStmt` with `before` set, whose `list` is the cell or the character operand. The
S-expressions are `(add l 3)`, `(insert (cell l 1) 3)`, `(remove l)` and
`(remove_char t)`.

A replace starts as a cell (`push_index`) or an n-th character (`push_char_nth`). When
`မှ` follows and `at_range_end` finds a `ထိ` (`Kw::Upto`) before the full stop or an
`အထိ`, `push_range` reads the end (and, for characters, `လုံးမြောက်စာလုံး`), then `ထိ`
and `ကို`, and pushes the range as one operand with the slot `Upto`; with no `ထိ`, `မှ`
keeps making a counted loop. The `အစားထိုးမည်` row takes `[Upto, Hpjint]`, so the value
is read as each operand is, and a range before another verb is ZK0206.
`လုံးမြောက်` is a second spelling of `ခုမြောက်` (`Kw::Ordinal`). The S-expressions are
`(replace t 0 4 v)` and `(replace_chars t 1 3 v)`.

Blocks and branches share `parse_branch`: with no opener, it reads one sentence into a
`BlockStmt { single: true }`. Inside an arm of a conditional that sentence cannot itself
start with `အကယ်၍` (ZK0215). The parser counts the blocks around a statement in `depth`,
which is how a start block, an external function or a C function declared inside a block
is reported (`parse_fn_decl`, `parse_external`: ZK0214) and still parsed. A function with
statements inside a block parses as at the top level, a `Stmt::Fn` among the statements of
the block; semcheck declares it in its scope.

When `NAME ဟူသော` starts the header after `သည်` (`at_param`), `parse_params` reads the
parameters (`parse_param_items`), each `NAME ဟူသော KIND တစ်WORD ကို` (`parse_name_and_kind`,
`parse_slot_kind`), then the markers `parse_item_markers` reads in order, `ဒေတာ ပြောင်းလဲရန်`
(`Kw::DataToChange`, `Param::edit`) or `ဒေတာ ထည့်ပေးရန်` (`Kw::DataToFill`, `Param::fill`,
which sets `Param::edit` too), `အပိုင်အနေဖြင့်` (`Kw::AsOwned`, `Param::owned`), and
`ဝိဘတ် "<preposition>" ကို သုံး၍` (`Kw::PrepositionWord`, the string, `Kw::Ko`, `Kw::Using`; `Param::preposition`),
and `လည်းကောင်း`, which each parameter of several takes (`err_as_well_as` otherwise); then
`လက်ခံပြီး`, or `လက်ခံမည်` for an external function. The preposition string must lex as words a preposition can hold
(`preposition_words`, ZK0241). The s-expression writes `own` and `edit` before the kind and the
preposition after the name: `own edit int* ပ(ကို)`. `parse_item_markers` takes a flag, `fill_ok`:
a header passes `true`, and a function kind (`fn_kind.rs`) passes `false`, so `SigParam` has
no `fill` and `ဒေတာ ထည့်ပေးရန်` in a function kind stops the item, ZK0201. A lone
`ပြောင်းလဲရန်` or `ထည့်ပေးရန်` (`Kw::ToChange`, `Kw::ToFill`) is no marker and stops the item
likewise.

`ကူးထည့်မည်` (`Kw::WillDupIn`) and `အကုန်ကူးထည့်မည်` (`Kw::WillDupAllIn`) are statement
verbs of the valence table with the slots `ထဲသို့` and `ကို`, as `ထည့်မည်` is
(`VerbOp::DupInto`, `VerbOp::DeepDupInto`): `x ထဲသို့ v ကို ကူးထည့်မည်` is an `AssignStmt`
whose value is the duplicate expression of `v`, shallow or deep, as below. After `ကိန်းရှင်` or
`ကိန်းသေ`, a name with `ထဲသို့` after it sends `parse_decl_or_loop` back to the name, reads
the sentence as that assignment, and makes a `DeclStmt` with the value as its start and no
written kind (`parse_decl_into`). `ပြောင်းထည့်မည်` (`Kw::WillShift`) is left to the heap
statement, where it is the `memmove` form of `ကူးယူမည်`.

The duplicate word `မိတ္တူ` is no reserved word: when the clause engine meets the name `မိတ္တူ`
after an operand that holds `၏` (`at_dup_word`), `push_dup` turns the operand into a duplicate
of it (`VerbOp::DupOf`). A word `တစ်<classifier>လုံး` before the `၏`, with the `၏` joined to
it or not (`whole_word`, `mark_whole`), marks the operand as the whole of a value, and the
duplicate is deep (`VerbOp::DeepDupOf`). The classifier itself is not kept in the tree. The
test of one object `X နှင့် Y သည် တစ်ခုတည်း ဖြစ်…` is the comparison row of `Kw::SameOne`
(`VerbOp::Same`), with the slots `နှင့်` and `သည်`.

Three words stand in front of a declaration, each read with a function of its own that
parses the next statement and sets a flag: `အများသုံး` (`parse_exported`, the field
`exported`), `မျှဝေနိုင်သော` (`parse_shared`, `StructDecl.shared`, on a struct alone) and
`စိတ်မချရသော` (`parse_unchecked`, `FnDecl.unchecked`, on a function that is not the start
block). An unsafe block opens with the joined token `UnsafeBlockOpen`, which `parse_block`
takes as it takes `BlockOpen`, and sets `BlockStmt.unchecked`.

The pass-on forms of a call set `CallExpr::pass`: the word form ends in `ရလဒ်ကောင်း`
(`CallGoodResult`), and the sign form takes a `?` that touches the `)`. The statement
`<value> ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်` is a plain `ReturnStmt` whose value is an error value: `ဟူသော အမှားအယွင်း`
is the phrase `CalledFault`, a postfix of the operand before it that `push_postfix` turns into
the verb `make_fault` (`VerbOp::MakeFault`), so an error value is an expression that a
return, an assignment or an argument can hold; the plain return is the `WillReturn` row.
A call with neither form is a plain call, and its value is a whole result. The tests of a
result and of an optional are the verbs `is_good`, `is_fault`, `has_data` and `lacks_data`, with the
slot `Within`: `<x> ထဲတွင် ရလဒ်ကောင်း ရှိ`, `<x> ထဲတွင် အမှားအယွင်း ရှိ`, `<x> ထဲတွင် ဒေတာ ရှိ` and
`<x> ထဲတွင် ဒေတာ မရှိ`. The reads `ထဲမှ ဒေတာ` and `ထဲမှ အမှားအယွင်း ဒေတာ` are postfix too:
`data_read_at` in `notation.rs` tells which of `VerbOp::Unwrap` and `VerbOp::UnwrapFault` stands
at a `ထဲမှ`, `data_reads` wraps a value in each read that follows it (a read binds tighter than an
operator), and the clause reader does the same for an operand with no slot through
`push_postfix`.
`အလွတ်` is an atom, a `NullLit` with `blank` set (`parse_null`), the optional that holds no data.
`ကနဦးတန်ဖိုး` is the same node with `initial` set, the initial value of the kind its place
takes, and the null pointer `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`, one token `Kw::Null`, is the node
with neither set. The `initial` field of a `NullLit` and of an `EnumMember` is left out of
an artifact when false.
A result kind is `BaseKind::Result`: `Kind::parts` holds `ResultParts { ok, err, optional }`, and a side
with no value (`အောင်မြင်သည့်သတင်း`, `မအောင်မြင်သည့်သတင်း`) is `None`. An optional kind,
`T ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`, is the same base with `optional` set and no error side.
`parse_cell_kind` reads both: the good side (`အောင်မြင်သည့်သတင်း`, a function kind in parentheses or a
plain kind), then `ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`, or `သို့မဟုတ်` and the error side
(`မအောင်မြင်သည့်သတင်း`, or `အမှားအယွင်းအနေဖြင့်` and a plain kind) and `ရှိနိုင်သော ဒေတာဆဲလ်`.
`cell_kind_len` measures the same phrase without consuming it, so `kind_len_at` and
`parse_kind_with` can tell a result kind from a plain kind with a look ahead. `kind.rs` reads it
wherever a kind goes, so a declaration, a parameter and a struct field take it as well as
the result clause; it is written `(ok ! err)` in an S-expression, with `-` for a side with no
value. The list and pointer levels after a result kind, and the count before it, belong to the
whole result, so `ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် စာရင်း` is a list
of results. A result kind ends in `ဒေတာဆဲလ်`, and no other phrase is one.
`parse_ret_clause` reads the result clause of a function into `FnDecl::ret`: the written result kind and its
measure word (`parse_slot_kind`, the word kept in `Kind::classifier`; the clause is a
`RetClause { kind, owned }`), then `ကို`, an optional `အပိုင်အနေဖြင့်` (`FnDecl.owned_ret`), and
`ပြန်ပေးမည်`. A clause with no measure word is ZK0234. A function kind reads its result clause as a header does (`fn_kind.rs`);
the result kind
is the one header of a function that can fail. `FnDecl` has no field for the error kind: a function can fail when
its result kind is a result kind, and not otherwise. The mark statements `လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်` and
`လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်` are return statements: the first is a bare return,
and the second a `ReturnStmt` with `fault` set and no value. The S-expressions are `(call-pass f ..)`,
`(make_fault v)` and `(return-fault)`.

A function kind is `BaseKind::Fn`: `Kind::sig` holds `FnSig { params, ret }`, each
parameter a `SigParam` with its owned and edit flags, its kind with its measure word, and
the preposition when written; each reads `KIND တစ်WORD ကို`, the markers and `လည်းကောင်း`, as in a
header with no name (`sig_params_len`, `parse_sig_params`). `fn_kind.rs` reads it. A kind phrase can be a bare function kind at its first
token, in the forms `FnForm` allows: `Full` where no list is open, `NoParams` while a list
of parameters or fields is open (`Parser::in_list`, which `parse_params`,
`parse_listed_fields` and the parameters of a function kind itself set), and `NoFn`, the result of a
function kind, where `လုပ်ဆောင်ချက် အမျိုးအစား` alone can stand. A function kind with
parameters inside a list stands in parentheses (`paren_fn_len`). `kind_len_at` and
`parse_kind_with` look for the function forms first, then a result kind, then a plain kind;
`listed_head_len` and `ret_tail_len` work out the length of a function kind from its
closing words, `လက်ခံသော`, `ပြန်ပေးသော` and `လုပ်ဆောင်ချက် အမျိုးအစား`, so the search stops at
the first word that does not fit. A parameter whose kind is a bare function kind with
parameters is read all the same and reported (ZK0239, `parse_bare_listed_kind`). It is
written `fn(<params>) -> <ret>` in an S-expression.

`NAME ဟူသော လုပ်ဆောင်ချက်` is `Expr::FnAddr` (`at_fn_value`, `parse_fn_value`): the
phrase `ဟူသော လုပ်ဆောင်ချက်` after a name is one token, `Kw::FnValue`, so the word right
after the name tells a value from a call. Before `အမျိုးအစား` the lexer keeps `ဟူသော`
apart (`Kw::Called`, then `Kw::FnKind`), so `NAME ဟူသော လုပ်ဆောင်ချက် အမျိုးအစား တစ်ခု` is a
parameter or a field of the bare function kind. The same token, then `ကို`, opens a long
call: `at_long_call` looks one token past the `ကို`, and a verb there (`as_verb`, or
`ပြင်ပ` of a leave) keeps the function value as the object of that verb; each other token
makes a call. A call expression is read as an atom (`parse_atom`), and at the start of a
statement through `parse_first`. The long call forms read their function with
`parse_long_callee`: a name, or a place in parentheses that starts with a name, then
`Kw::FnValue` and `ကို`; for a group, `group_end` scans to its `)` first. The place goes into
`CallExpr::place` or `CallStmt::place`, and `callee` keeps its first name. A parameter or a variable declared with a function kind that has prepositions
joins `Parser::functions` under its name (`note_preposition_value`), so a preposition call through it
reads. A variable declared with no kind that starts as `<fn> ဟူသော လုပ်ဆောင်ချက်` joins it with
the prepositions of that function (`note_preposition_addr`); a written kind wins over the
starting value, so a kind with no prepositions adds nothing; `fn_header` counts the prepositions of a header outside parentheses alone, so the
prepositions of a function kind in it do not count.

Kinds and literals have their own modules. `kind.rs` reads `(လက္ခဏာရှိ|လက္ခဏာမဲ့)? (N ဘစ်)?
base`, and checks the bit counts (8, 16, 32, 64 for integers; 32 and 64 for floats).
`literal.rs` reads `(အခြေ N ဖြင့်)? digits (ဟူသော kind တစ်WORD)?` (`at_literal_kind`,
`parse_literal_kind`): it converts one digit after the other in the given base, checks the
range of the kind (ZK0220) and the digits against the base (ZK0219); a kind that is not a
plain number kind is ZK0222. A list literal, `[...]` or `... တို့ပါသော စာရင်း`, takes
`ဟူသော kind တစ်WORD` after it as well (`parse_list_kind`, a list kind or ZK0222), kept in
`ListLit::kind`. A literal with no kind is limited to 64 bits here (a larger one is reported as not fitting); the kind check gives it the
kind of its place and checks the range there. In `kind.rs`, the base word `စာသား` gives
the base `text`. The levels after a base are `Wrap::List` (`စာရင်း`, with a count before it or
none) and `Wrap::Ptr` (`လိပ်စာ`, the pointer kind); there is no other level.

Each place a kind stands in, a declaration, a generic loop, a parameter, a field, a
result, a parameter or the result of a function kind, a conversion, a size, a default arm
and the kind of a literal, reads it with `parse_slot_kind` (`decl_head.rs`): the kind, then
the count word `တစ်WORD`, kept in `Kind::classifier`, or a struct name joined to its count
word, `လူတစ်ယောက်`. A kind with no word after it is ZK0234 (`needs_word`), with the word the
kind takes in its help. `slot_kind_len_at` measures the same with a look ahead, and
`slot_then` asks whether a word follows a kind with or without its count word, which a
conversion uses for `အဖြစ်`. A kind inside a kind takes no word, and neither does the kind
of an alias. The size of a heap statement keeps its count word in `Kind::classifier` of
its unit too. The loader writes `Kind::alias` when it puts the kind of an alias in place
of a use that adds no levels, so the naturalness check counts it with the word of the
alias.

A string literal is an expression, read in `str_lit.rs` wherever an operand starts, a
print included: a print of a literal goes through the clause engine as each other print
does, and `PrintStmt.value` is an `Expr`. `parse_str` turns the parts of the lexer into
an `Expr::Str(StrExpr)` whose pieces are `StrPiece::Text(String)` or
`StrPiece::TemplateSlot(Expr)`; text next to text is joined into one piece. `parse_template_slot` lexes the
text of each template slot with `lex_at` at its own offset, so spans point inside the literal, and
parses it as one expression with an inner `Parser` that continues the node numbers of the
file. A template slot with nothing in it is ZK0202, and text left after the expression is the
"expected `}`" error. The lexer ends a template slot at its matching `}`, so the template slot can hold a
template literal of its own; `parse_template_slot` reads that literal as each other one,
and the nesting needs nothing more.

The characters of text are read in the clause engine. After an operand that holds `၏`,
the name `စာလုံး` (matched on its text, `name_at`) and a number build `VerbOp::CharAt` (`push_char_at`, counted from
0); after `<text> ၏ <number> ခုမြောက်`, the word `စာလုံး` builds `VerbOp::CharNth`
(`push_char_nth`, counted from 1). Neither has a verb word, so both are built in place as a
`VerbExpr` (`char_verb`). The count `စာလုံးအရေအတွက်` is a verb of the valence table, as
`အကွက်အရေအတွက်` is.

The verbal join `a နှင့် b တို့ကို ဆက်ထားသည့် စာသား` is read in `parse_operand`, as a list literal
is: after the first piece, a `၊` or `နှင့်` with `တို့ကို ဆက်ထားသည့်` ahead (`at_join_rest`)
starts it, and `parse_join` reads the pieces as a list in words, then `တို့ကို`,
`ဆက်ထားသည့်` and `စာသား`, into a `VerbExpr` with `VerbOp::Join`.

### Lists in words

Each list written in words, the cells of a list or a ဇယား, the pieces of a join, the
pairs of a struct literal, the arguments of a long call or of an arguments-first call,
imported names, fields, members and switch cases, goes through one helper in
`word_list.rs`: `parse_word_list` (or `parse_word_list_from`, after a first item read
before) reads the items with `၊` between them and `နှင့်` before the last, and ZK0245
reports a list of two or more without the `နှင့်` or with a `၊` after it.
`expect_list_word` then takes the closing word of the count, such as `ကို` after one item
and `တို့ကို` after two or more; the word of the other count is ZK0246.

While an item is read, `Parser::in_word_list` is set, and the clause loop ends the item at a
`နှင့်` outside parentheses (`at_item_end`), and at `ဖြင့်` after a lone operand right before
the end of a long call, which closes its one argument. A group, the arguments of a sign-form
call and a callee place in parentheses clear the flag (`outside_word_list`), so an item that
joins its own operands with `နှင့်`, as `(က နှင့် ခ ကို ပေါင်းသည့်ရလဒ်)` does, stands in parentheses.
A switch arm of two or more cases starts at its `နှင့်` too, with the then-word ahead.

The scan that tells a list from a clause, `word_list_ahead`, walks the tokens outside
parentheses and brackets to the closing word, and stops at a particle, `၏` or the full
stop, which no item holds; so `က နှင့် ခ ကို ပေါင်းသည့်ရလဒ်` is never taken for the start of a list.

### ဇယား kinds, rows and the fallback

A ဇယား kind, `<index kind> ဖြင့် ညွှန်းသော <value kind> ဇယား`, is the first form `kind_len_at` and
`parse_kind_with` test. The index kind is a kind with no ဇယား of its own at its start, so the
search ends; an optional or a result index parses, so the check can refuse it. The value
kind is each kind, a ဇယား included, and the levels after `ဇယား` wrap the ဇယား.

A row is a postfix of an operand, as `ထဲမှ ဒေတာ` is: `ထဲမှ အညွှန်း <notation>` builds the
verb `row`, and `၏ တန်ဖိုး` right after it makes it `row_value`. The index is a notation
expression, so it ends before the `၏`. `၊ မရှိလျှင် <prefix operand>` after `ထဲမှ ဒေတာ` builds the
verb `or_else` over the optional itself; the fallback is one operand, so a read binds tighter
than an operator. The postfixes run in `data_reads`, after each atom, and again in the clause
loop for an operand a clause built.

`[k: v၊ …]` and `[:]` are bracket literals, told from a list through the `:` after the first
cell; `k အနေဖြင့် v နှင့် … တို့ပါသော ဇယား` is a sentence literal, told from a counted loop through the
`တို့ပါသော ဇယား` ahead. `<row> ကို ဖျက်မည်` reduces through a verb row of its own into a remove
statement marked `row`. An each loop reads its part, `အကွက်တိုင်း`, `စာလုံးတိုင်း`, `အညွှန်းတိုင်း`
or `အတန်းတိုင်း`, and a first name before `နှင့်` when there are two.

## Generics and traits

Kind parameters stand after a name in the notation form (`parse_notation_params`) or
after `သည်` in the word form (`parse_word_params`); while a declaration is parsed the
parameters are kind names (`add_kind_names`). Kind arguments go on `Kind.args` and on calls,
function values and struct values (`kind_args`): after `::` or the name in the sign
form, before it in the word form (`zakac_parser/src/generics.rs`). Inside kind
arguments a `>>` token reads as two closers (`half_close`).

A trait (`at_trait_decl`: a name, then `ဂုဏ်သတ္တိ`) and a trait block (`at_trait_block`:
`ရှိစေရန်` in the statement) parse in `zakac_parser/src/traits.rs`. The functions of a
trait block come out of it into the statement list of the file (`parse_program`), each
a plain function; the block keeps their names. A notes block after the head of a
function or a trait gives the kind parameters their traits (`give_bounds`), and its
local kinds become kind names for the rest of the function, then their kinds stand in
their place (`expand_local_kinds`), marked in `Kind.alias` to count with `ခု`.

## The prelude

The loader reads the prelude, `"စံကား/အခြေခံအုတ်မြစ်"` (`stdlib/core_kinds.zaka`,
`core_traits.zaka` and `core_verbs.zaka`, joined into one source in `STD_MODULES`), as a
file of the program once a file names one of its traits that the file does not declare
itself (`uses_prelude`), ahead of that file. A part opened alone in an editor reads as the
prelude (`std_module_at`), and the whole prelude is then not read beside it.

The compiler mark `ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော` heads three declarations:

- a kind, `Stmt::BuiltinKind` (`parse_builtin_kind`): a name, `စာရင်း` or `ဇယား`, `သည်`, the
  kind parameters in the word form, then the mark, `[အများသုံး]` and `ဒေတာအမျိုးအစား ဖြစ်သည်`;
- a trait, after `ဟူသည့်`, with `[အများသုံး]` after the mark;
- a function, `FnDecl.builtin` (`parse_builtin_fn`), with no block.

The names of the marked functions (`prelude_builtins`) take no prepositions in the table of
preposition calls. The clause engine reads them as verbs (`verb_at`): `<name> မည်` of `ပြသ`,
`ထည့်`, `ဖျက်` and `အစားထိုး` (a glued `ပြသမည်` splits so), and the three counts right after
`၏`; each stands for its old verb row (`BUILTIN_VERBS`, `BUILTIN_COUNTS` in the lexer's
`role.rs`, which paints them too). Semcheck calls a function of the file of the name in
place of the sentence when one takes the operands (`own_verb`, `own_count`).

## Prepositions, glued ends and word operators

- A preposition string is lexed: its tokens are the preposition, so prepositions hold several words
  (`mark_words` in `using.rs`). `Parser::functions` maps each function the file declares
  or imports to the preposition list of each of its functions; the parser derives the words of
  the prepositions and the first words from it. A preposition call starts at a first word, or at a `၏`
  with a function that declares prepositions and a call end after it (`preposition_end`), and `parse_preposition` reads
  the longest preposition of the callee that the next words spell.
- `glued.rs` splits a name token that is a function the file sees with a call end glued
  to it (`ပေါင်းသည့်ရလဒ်`) before the parse, never a declared name, and warns of names that
  can clash (ZK0250, ZK0251). A name in braces never splits.
- Each file sees the functions of the prelude (`with_prelude`); the operator functions come
  without prepositions, since the clause engine reads them: `operator_at` takes
  `<operator> <value end>` as a verb whose slots are the prepositions of the trait function
  (`OPERATOR_VERBS` in `zakac_ast::verb`).
- `x ၏ f(a)` joins the sign-form call to the operand before `၏` (`method_of`); `x ၏ f` remains
  a field read, which the check turns into a call when no field fits.

## Reading the imports

An import whose path starts with `စံကား/` (`STD_PREFIX`) reads a module of the stdlib
from `STD_MODULES`, which `include_str!` fills from `stdlib/`: `import_std` registers it in
the source map as a file named after the module, and reads it as each imported file, once.
`Loaded::single`, for a program read from storage, keeps its old path unless the program
imports from the stdlib; then `with_std` runs the loader with `disk` off, in a source map
of its own that starts with the file (`SourceMap::add_file`), so the modules sit after it;
each other import is ZK0232, and a name the stdlib lacks is ZK0238.


`load(root, &mut SourceMap, &LexerOptions) -> Result<Loaded>` reads a file and the files it
imports, and gives one tree for all of them:

```text
Loaded  { program, imports: Vec<EmbeddedFile>, diagnostics }
Program { stmts, units: Vec<Unit>, span }
Unit    { name, first_stmt }
```

For each file, from the file that is run:

1. Lex it, and parse it a first time to find its import sentences. The imports do not
   depend on struct names, so this parse finds them all; its tree is dropped.
2. For each import, join the path to the folder of the file and drop the `.` and `..`
   parts, so two spellings of one file meet. A file read before gives its unit again; a
   file on the chain of files being read is a circle (ZK0231); each other file is read
   through the source map and visited in turn, so its own imports come first. A file
   that cannot be read ends the load with a host error, which has no code.
3. Parse the file a second time with `parse_with`: its node numbers go on from the file
   before, so each expression of the program has a number of its own, and the struct
   names it imports from exported structs start a kind.
4. Set `unit` on each import to the unit it reads, append a `Unit` and the statements
   of the file to the program, and note the structs, enumerations and kind aliases the
   file exports.

   For a file read from disk, `lib_paths_from` makes the lib of each C function that is a
   path (`is_lib_path`, in `zakac_ast`: it holds `/` or `\`, ends in `.so`, `.dylib` or
   `.dll`, or holds `.so.`) absolute: a relative one starts at the folder of the file that
   declares the function, as an import does. A lib name is left as it is, and so is a
   program read from storage.

Once the files are read, `expand_aliases` (`src/alias.rs`) puts the kind of each alias in
place of each use of its name: in a kind, where the levels written after the alias go
around the levels of its kind, as the name of a struct literal, and before `ထဲမှ` in a
member. Alias names belong to the whole program, as struct names do, so one table serves
each file; a name a struct, an enumeration or an earlier alias has is ZK0316, and a chain
of aliases that comes back to its start is ZK0336 for each alias in it. Each exit of the
loader (`Loaded::single`, `with_std`, `load`) runs it, so the stages after the parser see
no alias; the parse artifact of one file keeps the `alias` statements as written.

So the units come in post-order: each imported file ahead of the files that import it,
and the file that is run last. `Program::unit_of(i)` gives the unit of statement `i`
and `Program::is_root(unit)` tells the file that is run. A program of one file has no
units, and both functions treat each statement as part of the file that is run.

A root whose name is not a real file, such as standard input or the text of the
explorer, is parsed once with `Loaded::single`, and `refuse_imports` reports each import
in it (ZK0232). `Loaded::single` is also what the `run_phases` function of each later
stage uses, so a test that compiles a bare string needs no file at all; the stage
commands and `zakac run` call `load` and then `check_loaded` or `run_loaded`.

A list count in digits is `Cells::Fixed`; each other count is `Cells::Expr`, a full
expression (`parse_count`, through `parse_expr`), which the check works out. The lookahead
(`count_prefix_len`) runs from `အကွက်` to the `ခုရှိသော` or `ရှိသော` that closes it, counting
the counts nested inside it (as in a size), and stops at the full stop. While a count is
parsed, `in_count` holds and no struct literal starts, since a count's own `ရှိသော` would
read as the end of one (`N - ၁ ရှိသော`).

The imported files go into each later artifact under `imports`, in reading order. That
order matches the order the source map holds them in after the file that is
run, so a decoded artifact registers the files again in that order and its spans,
relative to the start of the first file, remain valid. The parser owns the codes of this
step: ZK0229 to ZK0232.

## Skipping a bad sentence

`parse_stmt_recover` wraps `parse_stmt`. On an `Err` it does three things.

1. `hint_split`: if one of the two tokens before the error is a name whose last vannas
   spell a particle, add the hint "write `အသက် ကို` with a space". This is the common trap
   of a name written joined to a particle.
2. Push the diagnostic.
3. `skip_stmt`: consume tokens up to just past a `။` at block depth 0, or stop at a
   block closer or `Eof`. Block openers and closers inside the skipped text are counted,
   so a broken statement that opened a block skips the whole block. If the failed
   statement started with `အကယ်၍`, the `else` arms after it are skipped too, so that ZK0211
   is not raised for them. If the failed parse took no token, one is skipped, so the loop
   makes progress at each turn.

Blocks and function bodies read their statements through the same function, so an error
inside a block remains inside it and the closer still matches. A real run, with an error on
each of two lines:

```text
error[ZK0206]: "ပြသမည်" needs "ကို|ဟု" after operand 1, found no particle
 --> recover.zaka:1:1
  |
1 | အသက်ကို ပြသမည်။
  | ^^^^
  = help: write "အသက် ကို" with a space: a name joined to a particle is read as one word

error[ZK0206]: "ပေါင်းလဒ်" needs "နှင့်" after operand 1, found ကို
 --> recover.zaka:2:1
  |
2 | ၁ ကို ၂ ကို ပေါင်းလဒ် ဟု ထားမည်။
  | ^
```

The third line of that file, `၂၀ ကို ပြသမည်။`, is fine and ends up in the tree, as the one
statement. The known weak spot: skipping goes to the next `။`, so a sentence with a
missing `။` takes the next one with it.

## Diagnostics

The parser owns the `ZK02` codes. The main ones: `ZK0201` (a token that was expected
is missing), `ZK0202` (no operand), `ZK0203` (no statement verb), `ZK0205` (too few
operands for a verb), `ZK0206` (an operand has the wrong particle), `ZK0207` (two
particles in a row), `ZK0208` (operands left unclosed), `ZK0209` (operands left over after
a statement verb), `ZK0210` (a chained comparison), `ZK0244` (nesting past the [limit](#the-nesting-limit)). The full list is in
[Error messages](../reference/errors.md). Each has a constructor in
`zakac_diagnostics::parser` from rendered text, so that crate never needs the parser.

## The nesting limit

The parser counts how deep notation expressions (parentheses, operands, calls, lists) and
blocks nest. Past 128 levels it reports `ZK0244` and skips the sentence, in place of
recursing until the stack is gone. A template slot (`"{...}"`) is not counted. Each `zakac`
command also runs on a thread with a 256 MB stack, so a deep but legal program, such as a
sum of five thousand terms (`၁ + ၁ + ...`), still compiles.

## Artifacts, tests and extending

`ParseArtifact` holds the source, the imported files, the tree and the diagnostics (the
lexer's first, then the parser's), with spans relative to the start of the file that is run; its text form starts with
`# zakac-parse-artifact v1`. The golden files are `tests/inputs/*.zaka` against
`tests/expected/*.zkastt`, rewritten with `ZAKA_BLESS=1`. `tests/clause.rs` holds a trace
for each shape of clause, `tests/stmt.rs` a case for each statement form and error.

- **A verb or comparison**: the lexer first (a `Kw`, a `RESERVED` row, perhaps a `PHRASES`
  row), then a `VerbOp` and a `row(...)` in `VERBS`, then a trace test in
  `tests/clause.rs`.
- **A statement verb**: the same, with `is_stmt: true`, plus a `Stmt` variant and struct,
  arms in `visit.rs` and `s_expr.rs`, and the build in `clause.rs`.
- **A statement that starts with a reserved word**: a row in the `match` of `parse_stmt`
  and a `parse_*` method that ends with `expect_stop`. If it shares a start with another
  form, read the start once; do not add mark and reset to the cursor.
- **An operator**: a `Punct` in the lexer and an `InfixOp` with the same `strum` string,
  and a row in `binding_power`.

The next stage is [names and kinds](semcheck.md), which reads the tree this page builds.
