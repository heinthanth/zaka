---
title: Stages
description: What each stage of the compiler takes in and gives out, which codes it owns, and one sentence followed through all of them.
---

# Stages

The compiler is a chain of stages. Each takes what the one before it made and gives
out something nearer to a program the VM can run: pieces, tokens, a tree, the IR,
and at last ops.

## The chain

```text
source text
    │
    ▼
segmenter     cuts the text into pieces                          zkseg
    │
    ▼
lexer         joins pieces into tokens             ZK01          zktok
    │
    ▼
parser        builds the tree of sentences         ZK02          zkast
    │  (the next two stages run when there is no error so far)
    ▼
naturalness   warns about unnatural wording        ZK05  ┐
names, kinds  resolves names, works out kinds      ZK03  ├─────  zksem
              ZK04, and ZK0705, ZK0713 to ZK0717            ┘
    │
    ▼
IR builder    lowers the tree to the IR                          zkir
folding       works out the constants              ZK06
    │
    ▼
borrow check  follows owners and borrows           ZK07          zkbck
    │
    ▼
codegen       writes the ops of the VM                           zkbc
    │
    ▼
the VM        runs the ops                         (traps)
```

The two digits after `ZK` tell which stage found a problem. The page of [error
messages](../reference/errors.md) lists each code, and [Diagnostics](diagnostics.md)
tells how a message is laid out.

## When a stage is left out

A stage runs when the stages before it found no *error*. A warning never stops the chain.

| A stage reports an error | Left out |
| --- | --- |
| the lexer or the parser | naturalness, names, kinds, and all the stages after |
| names and kinds (and a warning under `--strict-particles`) | the IR builder and all after |
| constant folding | the borrow check and all after |
| the borrow check | the code generator and the VM |

This keeps one broken sentence from making a chain of errors in the stages that depend
on it. Inside a stage, the first problem does not end the stage: the lexer reports each
bad character, and the parser goes on from the next sentence.

## One sentence through each stage

This sentence adds two numbers and prints the sum:

```zaka
၁ + ၂ ကို ပြသပါ။
```

```text
၃
```

Each listing below is shortened from the output of the command named above it. The
`span` of each item (the range of source octets it came from) is left out. The spans are
there in the real artifacts, and these let a later stage point back at the source.

### Segmenter

`zakac segment` cuts the text at the edges of Burmese letters with their marks (`ကိန်း`,
`ရှင်`), digits and other characters. It does not know what the pieces mean.

```text
burmese_digit      ၁
non_burmese_char   " "
non_burmese_char   +
non_burmese_char   " "
burmese_digit      ၂
non_burmese_char   " "
burmese_vanna      ကို
non_burmese_char   " "
burmese_vanna      ပြ
burmese_vanna      သ
burmese_vanna      ပါ
burmese_vanna      ။
non_burmese_char   "\n"
```

`ပြ`, `သ` and `ပါ` are three pieces here; the next stage is the one that knows these make
one word. The segmenter reports no problems.

### Lexer

`zakac lex` takes the pieces and makes *tokens*: numbers, names, reserved words, signs,
and the end of the input. It joins the pieces of a reserved word or phrase, so
`ပြ သ ပါ` becomes one `print` token. A sentence-final particle (`သည်`, `ပါ`,
`ပါသည်`) is kept on the verb it follows, as the `particle` of that token.

```text
number  (burmese digits)    ၁
punct   plus                +
number  (burmese digits)    ၂
kw      ko                  ကို
kw      print               ပြသပါ
punct   full_stop           ။
eof
```

Each token also tells whether it is `joined` to the one before, with no space between.
The lexer owns **ZK01**: a character Zaka does not use, a malformed number, text with no
closing quote, an unknown escape, a name in backticks that does not close. See [The
lexer](../internals/lexer.md).

### Parser

`zakac parse` reads the tokens as sentences, and builds the *tree*. A number is a leaf,
`+` makes a node with a left side and a right side, and the `ကို` and the verb make a
`print` statement whose slot is `ko`:

```text
print (slot ko)
└─ add
   ├─ int 1
   └─ int 2
```

The parser checks the grammar of a sentence, such as which particle goes with which verb,
and it knows nothing so far about what a name means. It owns **ZK02**: a missing `။`, a
missing operand, a particle in the wrong place, a statement with no verb, a number too
big for its kind. See [The parser](../internals/parser.md).

When the file imports other files, this stage reads them too: each imported file is
lexed and parsed before the file that imports it, and the statements of all the files go
into one tree, imported files first. The later stages see that one tree. The errors of
the import sentence and the export mark, ZK0229 to ZK0232, belong to this stage; see
[Modules](../language/modules.md).

### Natural wording

When the lexer and the parser found no error, the naturalness check reads the tree for
wording that is valid but unlike Burmese: `ကြီးသည်လျှင်` where `ကြီးလျှင်` reads better.
It reports **ZK05** warnings. With `--strict-particles` these are errors. The sentence
above has none. See [Natural wording](../language/naturalness.md).

### Names and kinds

`zakac semantic-check` resolves each name to its declaration, works out the *kind* of each
expression, and checks that the kinds fit. It owns **ZK03** (names and scopes) and
**ZK04** (kinds), and also the kind rules of owners and borrows: ZK0705 and ZK0713 to
ZK0717. The artifact keeps the tree and adds a table from each expression to its kind:

```text
0 (the 1)    isize
1 (the 2)    isize
2 (the add)  isize
```

An integer written with no kind is an `ကိန်းပြည့်` (`isize` in messages). See [Names and
kinds](../internals/semcheck.md).

### IR builder and folding

`zakac irgen` lowers the tree to the IR: a smaller form of the program with the kind
written on each expression. The next step, *folding*, works out what can be worked out
before the program runs:

```text
print
└─ const 3 : isize
```

The add is gone. Without folding (`--fold none`), the IR keeps `add` with `const 1`
and `const 2` under it. Folding owns **ZK06**: a sum of constants that does not fit its
kind, a division with the constant zero as divisor, a constant cell number outside its list.

### Borrow check

`zakac borrow-check` follows each owner and each borrow of the program through the IR,
and reports **ZK07**: a block used after it moved, a borrow that outlives its block, a
block never given back, a read past the end of a block. The sentence above holds no
pointer, so there is nothing to find. The artifact adds a *digest* of each function: what
it frees, resizes or moves out through its parameters, and which parameters its result
can point into. A call site uses the digest instead of looking inside the function again.

The check also marks the cell reads it has proved to be in range, so the code generator
can leave out the run-time check for those. See [Pointer checks](checks.md) and [The
borrow checker](../internals/borrow-checker.md).

### Code generation

`zakac codegen` turns the IR into the ops of the stack machine, with a table that maps
each op back to a range of the source:

```text
fn main
   0  push 3:isize
   1  print
   2  ret
```

The ops are listed in [Instructions](../vm/instructions.md); see [IR and
code](../internals/codegen.md) for how these are made.

### The VM

`zakac run` hands the ops to the VM. It pushes `3`, prints it as `၃`, and returns.
The VM reports no code: when a program does something it must not, the VM
stops it and tells so ([When a program stops](../vm/traps.md)).

## Artifacts

Each stage command can write what its stage made. The artifact is a YAML file, and a
header line tells which stage made it:

```console
$ zakac parse sum.zaka | head -3
# zakac-parse-artifact v1
name:
  Real:
```

Each artifact holds the name of the source file, the *source text itself* (so that
spans can be shown without the file), and what the stage made. Each one after the
segmenter's holds the diagnostics found so far as well. That is how a saved `.zkbc` file
can still print a warning with the line it points at. The raw form of an artifact holds
the same data in a compact encoding, behind the header `ZKAR`, the version, and the name
of the stage; the name is checked first, so a raw file of one stage is not read as the
artifact of another.

| Stage | Command | Holds besides the source and the diagnostics |
| --- | --- | --- |
| segmenter | `segment` | the pieces (no diagnostics) |
| lexer | `lex` | the tokens, the text of each string, the comments |
| parser | `parse` | the tree |
| names and kinds | `semantic-check` | the tree and the kind of each expression |
| IR builder, folding | `irgen` | the IR after folding |
| borrow check | `borrow-check` | the IR and the digest of each function |
| codegen | `codegen` | the module of ops, the one `run` reads |

[Commands](commands.md) lists the options and the file extensions.
