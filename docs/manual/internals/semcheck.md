---
title: Names and kinds
description: What the name and kind check does after parsing, with the passes in order, the tables it builds, the wording check that runs before it, and the codes it raises.
---

# Names and kinds

After the parser comes a stage that decides whether the tree makes sense. It checks that
each name is declared where it is used, that `ကွင်းဆက်` sits in a loop and a return in a
function, and that each value has a data kind that fits its place. It does not change the
tree. Its product is a table of the kind of each expression, plus diagnostics. The code
is in `compiler/zakac_semcheck`, and the wording check it runs first is in
`compiler/zakac_naturalness`.

Constant folding is not here. It works on the IR and is told in [IR and code](codegen.md).

## The driver

Each stage crate has a `run_phases` function that runs the stages before it. For this
one, `zakac_semcheck::artifact::run_phases` does the whole front end:

```text
 lex ──▶ parse ──┬─ a lexer or parser error? ──▶ stop: no tree check
                 │
                 ├──▶ zakac_naturalness::check ──▶ warnings ZK05 ┐
                 └──▶ zakac_semcheck::semcheck ──▶ kinds, ZK03 ZK04, ZK0705 ZK0713..17 ┤
                                                                  all diagnostics ◀──┘
```

The two checks are skipped when the lexer or the parser reported an error, so one broken
sentence does not start a chain of follow-on errors. The result is `Checked { program,
kinds, diagnostics }`: the tree, the kind table, and one list of diagnostics, the
lexer's first, then the parser's, then the wording warnings, then the name and kind
errors.

`SemcheckArtifact` is the reloadable form (text header `# zakac-semcheck-artifact v1`,
files `.zksemt` and `.zksem`). The command takes `--platform-bits` (8 to 128, default 64),
`--strict-particles` and `--no-raw`; these become `SemcheckOptions { platform_bits,
strict_particles, no_raw }`, and the later stages receive the same options.

## The passes, in order

### 1. Wording (`zakac_naturalness`)

A visitor over the tree. The parser accepts each particle on a verb, and either `ကို` or
`ဟု` after a printed operand; this pass names which wording is natural, and reports the
rest as warnings. A warning never fails a build, and `--strict-particles` raises them to
errors. The pass reads just the tree: the `particle` and `next` fields the parser kept on
verbs, boolean words and `မဟုတ်`, and the `slot` of a print.

`site.rs` turns a node into a `Site` (a clause-final word, a group in parentheses, a
value, a print operand). `rules.rs` is a table `RULES` of `{ code, fires }`; the first row
that fires at a site gives the code, so order matters when two rules overlap.

| Code | Fires when | Wants |
| --- | --- | --- |
| ZK0501 | a negative word, then `သို့မဟုတ်`, with `သည်` | `ပါ` |
| ZK0502 | a particle before `လျှင်` or `နေသရွေ့` | none |
| ZK0503 | `မဟုတ်` with `သည်` or `ပါသည်` | `ပါ` or none |
| ZK0504 | `ပါ` or `ပါသည်` before `ဖြစ်၍` | `သည်` |
| ZK0505 | a sentence or a string printed with `ကို` | `ဟု` |
| ZK0506 | a plain value printed with `ဟု` | `ကို` |
| ZK0507 | parentheses that add nothing | none |
| ZK0508 | a clause that gives a value ends in `ပါ` or `ပါသည်` | `သည်` |

A run of `"မင်္ဂလာပါ" ကို ပြသပါ။` gives `warning[ZK0505]` with the help `write "... ဟု
ပြသပါ။"`. The natural wording is a matter of the language, and is told in
[Natural wording](../language/naturalness.md).

### 2. Function signatures (`collect_fns`)

A first walk over the top-level statements records the signature of each function (the
kinds of its parameters and of its result), so a call can come before the declaration. The
first declaration of a name wins; a second is ZK0304.

### 3. The walk (`Checker`)

`semcheck(program, options)` pushes the global scope and visits the tree with a `Checker`
that implements `Visitor`. It has one method for each statement form and calls
`infer(expr, expect)` for each expression.

| Statement | Checks |
| --- | --- |
| declaration | a constant has a value; the value converts to the kind; one declaration of a name in a scope. The name is declared after the value is checked |
| assignment | the target is a place, declared, not a constant or a counted-loop variable; the value converts to the kind of the place |
| block, if arm | a block is a scope of its own; a condition is `bool` |
| while, generic loop | a `bool` condition; the loop depth grows with one |
| counted loop | bounds and step are integers; the loop variable takes the common kind of the bounds; a literal step of zero or less is ZK0409 |
| `ကွင်းဆက်`, `နောက်တစ်ကြိမ်ဆီသို့` | the loop depth is above zero in the current function |
| function | parameters are distinct; one scope holds the parameters, `ရလဒ်` (just when the function has a result) and the block; the loop depth restarts at zero |
| return | in a function; a value just when the function has a result, and it converts to the result kind |
| call | the function exists; one argument for each parameter; each converts |
| heap sentences | the target is a place of a pointer kind (ZK0417); the size is `usize` |

Each subtree is inferred once, so a diagnostic is never repeated. When an operand has an
error, its operation returns `None` and reports nothing more; an operation with two bad
operands reports the first. The walk goes on past an error, so one run can report several:

```text
error[ZK0406]: the literal 200 does not fit i8
 --> bad2.zaka:1:54
error[ZK0301]: "အမည်" is not declared
 --> bad2.zaka:2:1
error[ZK0401]: expected bool, found isize
 --> bad2.zaka:3:52
```

At the end the diagnostics are sorted in source order.

## The data it builds

**`DataKind`** is `Copy` and has seven forms:

| Form | Holds |
| --- | --- |
| `Int(IntKind)` | a sign and a `Width`, which is `Platform` or `Bits(n)` |
| `Float(bits)` | 16, 32, 64 or 128 |
| `Char`, `Bool` | |
| `Ptr(PtrKind)` | levels of address over a base, with the outer level an owner, a borrow or raw |
| `Null` | the kind of `ဗလာ`, which converts to each pointer kind |
| `List(&'static ListKind)` | a count of cells of one kind; list kinds are interned so the form remains `Copy` |

A written kind becomes a `DataKind` with defaults filled in: no sign word is signed, no
bit count is platform size for an integer and 64 for a float. So the plain
`ကိန်းပြည့်` is `isize` and the plain `ဒဿမကိန်း` is `f64`. `isize` and `i64` are different
kinds, even on a 64-bit platform, and neither widens to the other.

**`Scopes`** is a stack of maps from a name to a `Binding { kind, access, span }`, where
`Access` is `Var`, `Const` or `LoopVar`. A lookup runs from the innermost scope out. A
`kind` of `None` means an error left it unknown, which silences follow-on messages.

**`FnSigs`** maps a function name to `FnSig { params, ret }`.

**`KindTable`** is `BTreeMap<NodeId, DataKind>`, the product of the stage. The id is the
`NodeId` that the parser gave each expression. For this program:

```text
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
အသက် ကို အသက် နှင့် ၁ ၏ ပေါင်းလဒ် ဟု ထားပါ။
အကယ်၍ အသက် သည် ၁၈ ထက် ကြီးလျှင် အသက် ကို ပြသပါ။
```

`zakac semantic-check` writes the tree again and then:

```text
kinds:
  0: isize
  1: isize
  2: isize
  3: isize
  4: isize
  5: isize
  6: isize
  7: bool
  8: isize
diagnostics: []
```

Id 0 is the literal `၂၀`; ids 1 to 4 are the target, the two operands and the sum of the
second sentence; ids 5 and 6 are the operands of the condition and id 7 the comparison
(`bool`); id 8 is the printed name. Each kind is `isize` because the literals had no
kind prefix and took the kind of their place.

## Kinds and conversion

An implicit conversion (`widens_to`) never loses data:

| From | To | When |
| --- | --- | --- |
| integer | integer | the target is wider and no sign is lost (an unsigned kind can go to a wider signed one) |
| integer | float | the integer width is at most the significand of the float: 11, 24, 53, 113 bits for 16, 32, 64, 128 |
| float | float | the target is wider |
| `ဗလာ` | pointer | in each case |
| pointer | pointer | the same flavour (owner, borrow or raw) and the same shape, or one is the opaque `လိပ်စာ`; a block of addresses read as other data needs a raw step |
| char, bool, list | other kinds | never |

The conversion verb (`cast_allowed`) is wider: number to number, char to integer and back,
bool to integer, address to address, address to integer, and integer to a raw address.
An integer never becomes an owner or a borrow straight off; the raw address between is
the step a reader can search for.

### Literals that take the kind of their place

An unprefixed literal is *flexible*: `infer` passes down an `expect` kind from the
declaration, the assignment target, the parameter, the result kind or the other operand,
and the literal takes it.

- An integer literal takes the expected numeric kind, or `isize`. The parser has not
  checked its range, so a value that does not fit is ZK0406 here; the expected kind is
  still recorded so nothing else complains. A `-` before the literal checks the negative
  range, so `-၁၂၈` fits `i8`.
- A float literal takes the expected float kind, or `f64`.
- In an operation with two operands where one is flexible, the fixed side is inferred
  first and its kind becomes the expectation of the other. With `က: u8`, the `၁` of
  `က + ၁` is a `u8`. If both sides are flexible and one is a float literal, both expect
  `f64`.
- Bitwise operators and shifts pass an expectation on just when it is an integer kind, so
  a float place does not turn `၁` in `၁ << ၂` into a float.

## Pointers and lists

The same walk checks the owner, borrow and raw kinds (see
[Ownership and borrowing](../language/ownership.md)). A kind keeps its flavour inside `PtrKind`, and
`expect_fit` names a mismatch of flavour on its own: a borrow given for an owner (ZK0714)
or an owner for a borrow (ZK0715). `place_of` finds the place that a target or an address
operand names, for `၏ လိပ်စာ`, `၏ တန်ဖိုး` and list cells. Taking the address of a borrow
is ZK0705, a marker `ငှားထားသော` in a wrong place is ZK0713, a lend of a raw address is
ZK0716, and `--no-raw` makes each kind with a raw level ZK0717.

These codes are errors in each mode of the ownership check: these are decided here, from
kinds alone, and `--mem-check` never sees them. The rules that need a flow through the
program belong to [the borrow checker](borrow-checker.md).

Lists are checked in `list.rs`: the cell kind, the count (written or taken from the
initializer), a cell number that is an integer, and a short-lived list that is not a place.

## Codes

| Range | Stage |
| --- | --- |
| ZK03 | names: undeclared or doubled names, constants and loop variables assigned, `ကွင်းဆက်` outside a loop, return outside a function, argument counts |
| ZK04 | kinds: mismatches, conditions that are not `bool`, operands of the wrong kind, literals that do not fit, shifts that go too far, bad conversions, bad pointer and list use |
| ZK05 | wording warnings |
| ZK0705, ZK0713 to ZK0717 | owner and borrow kind rules |

Each code has a constructor in `zakac_diagnostics` (`resolve.rs`, `kinds.rs`,
`naturalness.rs`, `ownership.rs`). See [Error messages](../reference/errors.md) for the list.

## Tests and extending

`tests/common/mod.rs` has `check`, `codes`, `ok` and `kind_at(checked, text)`, which gives
the kind recorded for the first expression whose source text is `text`. The files are
`kind.rs` (each widening row, the conversion table), `resolve.rs` (each ZK03 code,
scopes, calls before declarations), `kinds.rs` (each ZK04 code, flexible literals),
`lists.rs`, `pointers.rs`, `ownership.rs`, and `samples.rs` (golden `.zksemt` files).

- **A statement form**: override its `visit_*` method in `semcheck.rs`; infer each
  expression once, with `expect_kind` or `expect_bool` when the place fixes a kind.
- **An operator**: give it an `OpClass` in `ops::infix_class` or `ops::verb_class`.
- **A wording rule**: a row in `RULES`, a `ZK05` code, a constructor; if the rule needs
  context the tree lacks, add it in the parser (a `NextCtx` form or a field).
- **A message**: a row in `define_codes!`, texts in `locales/codes.toml`, a constructor.

The checked tree and its kind table go on to [IR and code](codegen.md).
