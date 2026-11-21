---
title: The parser
description: How the parser turns the tokens of a file into the tree, with its three tiers of expressions, the verb table, a sentence followed through the code, and how it skips a bad sentence.
---

# The parser

The parser takes the tokens of one file (the `LexOutput` of [the lexer](lexer.md)) and
builds the tree of its statements (the AST), plus a list of
diagnostics. When a sentence (statement) fails, the parser reports it, leaves it out of
the tree and goes on, so the rest of the file still parses. Two crates share the work:
`zakac_ast` has the tree and the table of verbs, and `zakac_parser` has the code that
reads tokens.

The parser is too large for one page, so it has four:

- this page: the approach, the three tiers of an expression, one sentence followed
  through the code, skipping a bad sentence, the limits and the tests;
- [Statements and declarations](parser-statements.md): how `parse_stmt` picks the form
  of a sentence, and how functions, structs, imports and loops are read;
- [Expressions and calls](parser-expressions.md): operands, the reads after `၏`, the
  forms of a call, the verbs of the core module, failures, text, lists and threads;
- [Data kinds, generics and imports](parser-kinds.md): data kind (type) phrases and
  measure words, literals with a data kind, generics, written-as hints, the core module
  and the loader that reads the imports.

The functions we call from outside are in `compiler/zakac_parser/src/parser.rs`:

| Function | Use |
| --- | --- |
| `parse(&SourceFile, &LexOutput) -> ParseOutput` | the normal path |
| `parse_at(text, base, &LexOutput) -> ParseOutput` | the same for a bare `&str` at a position |
| `parse_lone_expr(text, base, &LexOutput) -> Result<Expr, Diagnostic>` | one expression that must use all the tokens; for tests |
| `parse_with(&SourceFile, &LexOutput, first_id, &structs) -> ParseOutput` | one file of a program with imports: node numbers start at `first_id`, and the struct names the file imports can start a data kind |

`ParseOutput` is `{ program, diagnostics, next_id }`, where `next_id` is the number the
next expression node would get. Spans are absolute positions, as in the lexer. A program
of several files goes through `load` in `src/load.rs`, which calls these functions once or
twice for each file (see [Reading the imports](parser-kinds.md#reading-the-imports)).

## The approach

The parser is hand-written, as a set of methods on one `Parser` struct. Each method reads
tokens from a `Cursor` and returns `PResult<T>`, a `Result` whose error is the one
diagnostic of the sentence being read. The cursor has `peek_n`, `bump` and `back(n)`, and
`Cursor::limited` can end the input before its last token for a trial read, but it has no mark and reset:
where two forms share a start, we read the start once and let the token after it decide.
The few places that attempt a reading and go back (a preposition call, a phrase call, a size
operand, a call whose arguments are clauses) put back the cursor, the diagnostics and the
node number themselves.

Zaka sentences put the verb last, and each operand has a particle after it that shows its
role:

```text
အသက် ကို   အသက် နှင့်  ၁ ၏ ပေါင်းလဒ်  ဟု   ထားမည်။
 operand ကို  operand နှင့်  operand ၏   verb    ဟု  statement verb
```

A parser for a C-like language reads an operator and then its right side. Here the verb
comes last, so the operands arrive first and have to wait, which is the reason for the
middle tier below: an operand stack that is reduced when a verb arrives.

An expression has three tiers, one module each, from the loosest to the tightest:

| Tier | Module | What it joins | Method |
| --- | --- | --- | --- |
| logical | `logical.rs` | clauses, with `ဖြစ်၍`, `သို့မဟုတ်`, `&&`, `\|\|` and postfix `မဟုတ်` | Pratt loop over clauses |
| clause | `clause.rs` | operands, each with a particle, closed with a verb | operand stack, run from the verb table |
| notation | `notation.rs` | atoms, with the C operators and the prefix `- အနုတ် ~ ! * &` | Pratt loop over atoms |

```text
 parse_expr         clause ဖြစ်၍ clause သို့မဟုတ် clause မဟုတ်      logical.rs
    │
 parse_clause       operand:slot operand:slot ... verb              clause.rs
    │
 parse_notation     atom + atom * atom ...                          notation.rs
    │
 parse_atom         name  number  "(" parse_expr ")"  call  ...     notation.rs
```

`parse_expr` is the top: it parses one clause, then loops on logical words.
`parse_clause` parses one notation expression, then loops on particles and verbs.
`parse_notation` parses one atom, then loops on operators. A parenthesized expression is
an atom, which is how a logical expression gets back inside a clause.

### Logical tier

The binding powers are 25 for postfix `မဟုတ်` (`NOT_BP`), 20 and 21 for `ဖြစ်၍` and `&&`,
and 10 and 11 for `သို့မဟုတ်` and `||`, all left to right (`logic_of`). Each operand of a
logical word is a whole clause, so `က သည် ခ နှင့် ညီသည် ဖြစ်၍ ဂ သည် ဃ ထက် ကြီးသည်` needs no
parentheses. Before each step the loop calls `mark_next`, which records on the left
operand the token that follows it (`NextCtx`: `Or`, `Then`, `ThenIs`, `While`, `WhileIs`,
`And`, `Close` or `Other`), and the wording check uses that later.

### Notation tier

The notation tier is a standard Pratt loop with the powers of C (`binding_power`):

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
| prefix `-` `အနုတ်` `~` `!` `*` `&` | 120 (`PREFIX_BP`) |

The loop stops at a token that is not an infix operator: a particle, a verb, a logical
word, `လျှင်`, `။`, `၊` or `)`. So an operand ends at the right place without help, and in
`က + ၁ သည် ခ ...` the operand is `က + ၁`. A comparison cannot have an unparenthesized
comparison on either side (ZK0210), and the help suggests `ဖြစ်၍`.

### Clause tier

The engine is `parse_clause_from(first, mode)`. Its stack has `Item { operand, slot,
whole }` entries and starts as `[first]`; `mode` is `Mode::Stmt` at the start of a
statement, where a statement verb can close the clause, and `Mode::Expr` elsewhere. The
engine reads the next token and does one of four things:

1. **A slot particle.** The top item must still have no slot (else ZK0207). We record the
   slot on it, consume the particle and read the next operand (`push_next_operand`). After
   `ကို`, when a data kind starts and `အဖြစ်` follows it, the data kind is pushed as the
   target of a conversion. When a verb comes next, nothing is pushed, because the particle
   belonged to the operand before it.
2. **A shift word** (`ဘယ်သို့`, `ညာသို့`): `push_shift` reads the distance, expects
   `ဘစ်` and pushes the distance with its slot.
3. **A verb.** `reduce` looks up the row of the verb in `VERBS`, takes one item for each
   slot of the row, checks the slot of each item against the row (`check_slots`), and
   reduces them to one expression, pushed back with no slot. For a statement verb the
   stack must then be blank, and `build_stmt` reads `။` and returns a statement.
4. **Each other token** ends the clause (`finish_clause`): one item with no slot must be
   left, and it is the result (else ZK0208).

The engine returns a `ClauseOut`: `Expr` for a value, `Stmt` when a statement verb closed
it, and `Switch` for the head of a switch (see
[Switches](parser-statements.md#switches)).

The verb table is the one place where the parser learns what a verb needs. `Slot` has
sixteen forms: the eight particles `နှင့် ၏ ကို ဖြင့် သည် ထက် အောက် ဟု`, then
`Ordinal` (`ခုမြောက်`), `Into` (`ထဲသို့`, the list of an add), `Before` (`၏ အရှေ့တွင်`,
the cell or character an add puts its value in front of, in the role of `Into`), `Upto`
(`ထိကို`, the range a replace changes), `Within` (`ထဲတွင်`, the optional or result a test
looks in), the shift pair `Bit(Dir)` (either direction fits), `KoOrHu` (`ကို` or `ဟု`,
for a print) and `KindAhpjit` (a data kind, then `အဖြစ်`). Some rows of
`zakac_ast/src/verb.rs`, which share constants such as `PAIR`:

```rust
row(Kw::Sum, VerbOp::Add, PAIR, false, false),
row(Kw::Greater, VerbOp::Gt, GREATER, false, true),
row(Kw::Assign, VerbOp::Assign, &[Slot::Ko, Slot::Hu], true, false),
```

The last two flags are `is_stmt` (the verb closes a statement: `ထားမည်`, and
`ကို ပြန်ပေးမည်`, the row of `WillReturn` with `[Ko]`) and `is_cmp` (a comparison, which
cannot be the operand of another comparison). The tests of an optional or a result are
rows too: `HasGood`, `HasFault`, `HasData` and `LacksData` take the slot `Within`. The
table has 31 rows. A new verb is a new row, and nothing in `clause.rs` changes. The verbs
of the core module, such as `ပြသမည်` and `ထည့်မည်`, have no rows: these are
calls of functions of the core module (see
[The verbs of the core module](parser-expressions.md#the-verbs-of-the-core-module)).

## One sentence, followed

Take `အသက် ကို အသက် နှင့် ၁ ၏ ပေါင်းလဒ် ဟု ထားမည်။` ("set age to the sum of age and 1").
The first token is a name, so `parse_stmt` sends it to `parse_expr_headed`, which runs
the clause engine in statement mode. Letters stand for the slots: `K` is `ကို`, `H` is
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

`zakac parse` prints the tree. Here a second sentence, an `if`, follows the first; we left
out the spans and the first lines of the artifact, and the rest is as printed:

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
          pass: false
          classifier: null
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
            pass: false
            classifier: null
        block:
          stmts:
          - print:
              value:
                name:
                  id: 7
                  name: အသက်
              slot: ko
          single: true
          unchecked: false
      else_block: null
      comptime: false
  units:
  - name: x.zaka
    first_stmt: 0
  target: null
diagnostics: []
```

What we can read from it:

- A statement is a variant of `Stmt` (`assign`, `if`, `print`, ...), and an expression is
  a variant of `Expr` (`name`, `int`, `verb`, ...). Each variant wraps a struct
  (`AssignStmt`, `VerbExpr`), so a pass can work on a `&VerbExpr` alone, and a form can
  gain a field without a change to the enum.
- The last field of each struct is a `span`, which reaches the closing `။` for a
  statement. Each expression struct also has an `id`, a `NodeId`. Ids count up in the
  order the nodes are built, so an inner node has a lower id than the node around it, and later stages index their side tables (the data kind of each expression) with these
  ids.
- `verb.op` is a `VerbOp`, the operation (`add`, `gt`, ...), and the operands are `args`
  in source order. The tree does not record which spelling was written, but it keeps what
  the wording check needs: the `particle` the lexer folded into the verb, and `next`, the
  context that came after it (`then` here, because `လျှင်` followed the verb).
- `single: true` marks a block of one sentence written without the block phrases, and
  `unchecked` marks an unsafe block.
- The integer literal has its converted `value` (a `u128`), its `base`, the data kind
  written after it (`ဟူသော <kind> တစ်ခု`) when there is one, and the digit `script`. A
  float literal has an `f64`.
- A name in a binding place (a declaration, a parameter, a call target) is an `Ident`, and
  a name used as a value is a `NameExpr`. Names are `Name` values (`name.rs`): the
  text is kept once for the whole run, in its canonical spelling, so a name is `Copy` and
  two equal names share one address.
- `units` lists the files of the program, and `target` records the target when a file has
  a compile-time conditional or names a fact of the target.

The one-line form of the tests, `ToSexpr`, prints the same tree as
`(program (set အသက် (add အသက် 1)) (if (gt အသက် 18) (block (print အသက်))))`. Operator and
verb names come from `strum`, and spans, ids and particles are left out. Most parser
tests assert against this form.

### Visitors

`zakac_ast::visit` has `Visitor`, which reads the tree and changes nothing, and
`VisitorMut`. `Visitor` has a `visit_*` method for each node struct, and each one
defaults to a `walk_*` function that visits the children in source order, with `enter`
and `leave` hooks around each node. A pass overrides just the methods it needs. The
wording check and the check of names and data kinds are visitors, and `VisitorMut` moves
spans when an artifact is saved or loaded.

## Skipping a bad sentence

`parse_program` calls `parse_stmt_recover` until `Eof`, and `parse_stmt_recover` wraps
`parse_stmt`. On an `Err` it does three things (`recover.rs`):

1. It adds hints. `hint_split` checks the two tokens before the error: when one is a name
   whose last vannas spell a particle (`split_particle`), it adds the hint "write
   `အသက် ကို` with a space", the common trap of a name written joined to its particle. This
   hint replaces the help of a statement that ends in no verb, since the joined particle
   hides the verb. `hint_group` adds a hint when an operator follows a verb that reads a
   value, as in `xs ၏ အကွက်အရေအတွက် == ၀`: the clause before the operator needs
   parentheses.
2. It pushes the diagnostic.
3. `skip_stmt` consumes tokens up to just past a `။` at block depth 0, or stops at a
   block closer or at `Eof`, which are kept. Block openers and block closers inside the
   skipped text are counted (`track_depth`, `open_blocks_since`), so a broken statement
   that opened a block skips the whole block, and when the failed statement started with
   `အကယ်၍`, the else arms after it are skipped too, so that no ZK0211 is raised for them.
   When the failed parse used no token, one token is skipped, so the loop makes progress at each
   turn.

Blocks and function blocks read their statements through the same function, so an error
inside a block remains inside it and the block closer still matches. A real run, with an
error on each of the first two lines:

```text
error[ZK0203]: this statement does not end in a statement verb, found "ပြသ"
 --> recover.zaka:1:9
  |
1 | အသက်ကို ပြသမည်။
  |      ^^^
  = help: write "အသက် ကို" with a space: a name joined to a particle is read as one word

error[ZK0206]: "ပေါင်းလဒ်" needs "နှင့်" after operand 1, found ကို
 --> recover.zaka:2:1
  |
2 | ၁ ကို ၂ ကို ပေါင်းလဒ် ဟု ထားမည်။
  | ^
```

The third line of that file, `၂၀ ကို ပြသမည်။`, is fine and ends up in the tree as its one
statement. The known weak spot: skipping goes to the next `။`, so a sentence with a
missing `။` takes the next sentence with it.

## Diagnostics

The parser owns the `ZK02` codes. The main ones: `ZK0201` (an expected token is
missing), `ZK0202` (no operand), `ZK0203` (no statement verb), `ZK0205` (too few operands
for a verb), `ZK0206` (an operand has the wrong particle), `ZK0207` (two particles in a
row), `ZK0208` (operands left unclosed), `ZK0209` (operands left over after a statement
verb), `ZK0210` (a chained comparison) and `ZK0244` (nesting past
[the limit](#the-nesting-limit)). The full list is on
[Error messages](../reference/errors.md). Each code has a constructor in
`zakac_diagnostics::parser` that works from rendered text, so that crate never needs the
parser.

## The nesting limit

The parser counts how deep notation expressions (parentheses, operands, calls, lists) and
blocks nest. Past 128 levels (`MAX_NEST`) it reports `ZK0244` and skips the sentence,
where it would otherwise recurse until the stack runs out. A template slot (`"{...}"`) is
not counted. Each `zakac` command also runs on a thread with a 256 MB stack
(`COMMAND_STACK` in `zakac_cli_common`), so a deep but legal program, such as a sum of
five thousand terms (`၁ + ၁ + ...`), still compiles.

## Artifacts, tests and extending

`ParseArtifact` has the source, the imported files, the tree and the diagnostics (the
lexer's first, then the parser's), with spans relative to the start of the file that is
run. Its text form starts with `# zakac-parse-artifact v1`. The golden files are
`tests/inputs/*.zaka` against `tests/expected/*.zkastt`, which we rewrite with
`ZAKA_BLESS=1`. `tests/clause.rs` has a trace for each shape of clause, and
`tests/stmt.rs` a case for each statement form and error.

To extend the parser:

- **A verb or comparison**: the lexer first (a `Kw`, a `RESERVED` row, perhaps a
  `PHRASES` row), then a `VerbOp` and a `row(...)` in `VERBS`, then a trace test in
  `tests/clause.rs`.
- **A statement verb**: the same, with `is_stmt: true`, plus a `Stmt` variant and its
  struct, arms in `visit.rs` and `s_expr.rs`, and the build in `clause.rs`.
- **A statement that starts with a reserved word**: an arm in the `match` of
  `parse_stmt_node` (or an `at_*` test before it) and a `parse_*` method that ends with
  `expect_stop`. When it shares a start with another form, read the start once; do not
  add mark and reset to the cursor.
- **An operator**: a `Punct` in the lexer, an `InfixOp` with the same `strum` string, and
  a row in `binding_power`.

The next stage, [names and data kinds](semcheck.md), reads the tree this stage builds.

Next: [Statements and declarations](parser-statements.md)
