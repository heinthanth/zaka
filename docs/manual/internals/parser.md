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
အသက် ကို   အသက် နှင့်  ၁ ၏   ပေါင်းလဒ်  ဟု   ထားပါ။
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
logical expression gets back inside a clause.

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

The table is the one place the parser learns what a verb takes. `Slot` has twelve forms:
the eight particles `နှင့် ၏ ကို ဖြင့် သည် ထက် အောက် ဟု`, `ခုမြောက်`, the shift pair
(`Bit(Dir)`, which fits either direction), `KoOrHu` (`ကို` or `ဟု`, for a print) and
`KindAhpjit` (a kind then `အဖြစ်`). Three examples of rows (the real ones share constants such as `PAIR`):

```rust
row(Kw::Sum,   VerbOp::Add,   &[Slot::Hnint, Slot::Ei],            false, false),
row(Kw::Greater, VerbOp::Gt,  &[Slot::Thi, Slot::Htet],            false, true),
row(Kw::Assign, VerbOp::Assign, &[Slot::Ko, Slot::Hu],             true,  false),
```

The last two flags are `is_stmt` (the verb closes a statement: `ပြသပါ`, `ထားပါ`, and
`ရလဒ်အဖြစ် ပြန်ပေးပါ`) and `is_cmp` (a comparison, which cannot be the operand of
another). There are 25 rows now. A new verb is a new row, and nothing in `clause.rs`
changes.

## One sentence, followed

Take `အသက် ကို အသက် နှင့် ၁ ၏ ပေါင်းလဒ် ဟု ထားပါ။` ("set age to the sum of age and 1").
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
ထားပါ    row Assign, slots [K, U]: take two,
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
  kind prefix and the digit `script`. A float literal holds an `f64`.
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
| `ကိန်းရှင်`, `ကိန်းသေ` | `parse_decl_or_loop` | shared start `NAME သည် kind init?`; then `ဖြစ်ပါစေ` is a declaration and `ဖြစ်ပြီး` a generic loop |
| `လုပ်ဆောင်ချက်` | `parse_fn_or_call` | `NAME သည်` two tokens ahead is a declaration; else a call, which heads a statement or an expression |
| a block opener | `parse_block` | opener, `။`, statements until the closer, closer, `။` |
| `ကွင်းဆက်`, `နောက်တစ်ကြိမ်ဆီသို့` | break, continue | |
| `ပြန်ပေးပါ` | a bare return | |
| `မင်မိုရီတွင်` | `parse_alloc` | the allocation sentence |
| `လိပ်စာ` | `parse_heap` | `လိပ်စာ PLACE ရှိ မင်မိုရီကို`, then `ပြန်လည်စွန့်လွှတ်မည်` for a release or a size for a resize |
| a string, then `မှ` | `parse_import` | `STR မှ NAME (၊ NAME)* (တို့ကို\|ကို) ထည့်သွင်းပါ။`; after another statement or inside a block it is parsed and reported (ZK0230) |
| `အများသုံး` | `parse_exported` | parses the next statement and sets `exported` on a function, struct or top-level declaration; on another statement it is ZK0229 |
| an `else`, an update, a closer with no opener | an error | ZK0211, ZK0212, ZK0226 |
| all other tokens | `parse_expr_headed` | a clause; see below |

`parse_expr_headed` runs the clause engine in statement mode. If a statement verb closed
the clause, the statement is built. Otherwise the clause is a value, and the next token
decides: `နေသရွေ့` makes a while loop; `မှ` makes a counted loop (`from မှ to အထိ (step စီ)? NAME အနေဖြင့်
mode block`); all else is ZK0203, which lists the endings a statement can have.

Blocks and branches share `parse_branch`: with no opener, it reads one sentence into a
`BlockStmt { single: true }`. Inside an arm of a conditional that sentence cannot itself
start with `အကယ်၍` (ZK0215). The parser counts the blocks around a statement in `depth`,
which is how a function declared inside a block is reported (ZK0214) and still parsed.

Kinds and literals have their own modules. `kind.rs` reads `(လက္ခဏာရှိ|လက္ခဏာမဲ့)? (N ဘစ်)?
base`, and checks the bit counts (8, 16, 32, 64, 128 for integers; 16 and up for floats).
`literal.rs` reads `(အခြေ N ဖြင့်)? kind? digits`: it converts one digit after the other in the given
base, checks the range of the kind (ZK0220) and the digits against the base (ZK0219).
A literal with no kind prefix is limited to 128 bits here; the kind check gives it the
kind of its place and checks the range there. In `kind.rs`, the base word `စာသား` gives
the base `text`, and a bare `တစ်စိတ်တစ်ပိုင်း` straight after it, with no `စာရင်း`, is the
view of text (`Wrap::View`).

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
the kind word `စာလုံး` and a number build `VerbOp::CharAt` (`push_char_at`, counted from
0); after `<text> ၏ <number> ခုမြောက်`, the word `စာလုံး` builds `VerbOp::CharNth`
(`push_char_nth`, counted from 1). Neither has a verb word, so both are built in place as a
`VerbExpr` (`char_verb`). The count `စာလုံးအရေအတွက်` is a verb of the valence table, as
`အကွက်အရေအတွက်` is.

The verbal join `a၊ b တို့ကို ဆက်ထားသည့် စာသား` is read in `parse_operand`, as a list literal
is: after the first piece, a `၊` with `တို့ကို ဆက်ထားသည့်` ahead (`at_join_rest`) starts it,
and `parse_join` reads the pieces, then `တို့ကို`, `ဆက်ထားသည့်` and `စာသား`, into a `VerbExpr`
with `VerbOp::Join`.

## Reading the imports

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
   of the file to the program, and note the structs the file exports.

So the units come in post-order: each imported file ahead of the files that import it,
and the file that is run last. `Program::unit_of(i)` gives the unit of statement `i`
and `Program::is_root(unit)` tells the file that is run. A program of one file has no
units, and both functions treat each statement as part of the file that is run.

A root whose name is not a real file, such as standard input or the text of the
explorer, is parsed once with `Loaded::single`, and `refuse_imports` reports each import
in it (ZK0232). `Loaded::single` is also what the `run_phases` function of each later
stage uses, so a test that compiles a bare string needs no file at all; the stage
commands and `zakac run` call `load` and then `check_loaded` or `run_loaded`.

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
error[ZK0206]: "ပြသပါ" needs "ကို|ဟု" after operand 1, found no particle
 --> recover.zaka:1:1
  |
1 | အသက်ကို ပြသပါ။
  | ^^^^
  = help: write "အသက် ကို" with a space: a name joined to a particle is read as one word

error[ZK0206]: "ပေါင်းလဒ်" needs "နှင့်" after operand 1, found ကို
 --> recover.zaka:2:1
  |
2 | ၁ ကို ၂ ကို ပေါင်းလဒ် ဟု ထားပါ။
  | ^
```

The third line of that file, `၂၀ ကို ပြသပါ။`, is fine and ends up in the tree, as the one
statement. The known weak spot: skipping goes to the next `။`, so a sentence with a
missing `။` takes the next one with it.

## Diagnostics

The parser owns the `ZK02` codes. The main ones: `ZK0201` (a token that was expected
is missing), `ZK0202` (no operand), `ZK0203` (no statement verb), `ZK0205` (too few
operands for a verb), `ZK0206` (an operand has the wrong particle), `ZK0207` (two
particles in a row), `ZK0208` (operands left unclosed), `ZK0209` (operands left over after
a statement verb), `ZK0210` (a chained comparison). The full list is in
[Error messages](../reference/errors.md). Each has a constructor in
`zakac_diagnostics::parser` from rendered text, so that crate never needs the parser.

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
