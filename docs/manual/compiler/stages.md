---
title: Stages
description: What each stage of the compiler reads and writes, which codes it owns, and one sentence followed through all of them.
---

# Stages

The compiler processes a program in stages (compiler passes). The segmenter splits the
source into pieces, the lexer turns those pieces into tokens, and the parser uses the
tokens to build a tree. Later stages check the program, lower it to an intermediate
representation (IR), and generate operations (instructions) for the VM (virtual machine)
or code for another target. See [Commands](commands.md) to run the compiler up to a
specific stage.

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
names, kinds  resolves names, works out kinds,     ZK03  ├─────  zksem
              checks unsafe code                   ZK04  ┘
    │
    ▼
IR builder    lowers the tree to the IR                          zkir
folding       works out the constants              ZK06
    │
    ▼
checker       flow checks and the pointer lint     ZK07          zkbck
    │
    ▼
codegen       writes the code of the target (`build`)            zkc
    │
    ▼
the VM        runs the operations                  (traps)
```

The two digits after `ZK` show which stage found a problem. The page of [error
messages](../reference/errors.md) lists each code, and [Diagnostics](diagnostics.md)
shows how a message is laid out.

## When a stage is left out

A stage runs when the stages before it found no *error*, and a warning never stops the
chain:

| A stage reports an error | Left out |
| --- | --- |
| the lexer or the parser | naturalness, names, data kinds, and all the stages after |
| names and data kinds (or naturalness, whose warnings are errors under `--lint-error naturalness`) | the IR builder and all after |
| constant folding | the checker and all after |
| the checker | the code generator and the run |

This keeps one broken sentence from causing a chain of errors in the stages that depend
on it. Inside a stage, the first problem does not end the stage: the lexer reports each
bad character, and the parser goes on from the next sentence.

## One sentence through each stage

This sentence adds two numbers and prints the sum:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ + ၂ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၃
```

Each listing below is a short form of the output of the command named above it. We leave
out the `span` of each item (the range of source octets it came from). The real artifacts
have the spans, which let a later stage point back at the source.

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
burmese_vanna      မည်
burmese_vanna      ။
non_burmese_char   "\n"
```

`ပြ`, `သ` and `မည်` are three pieces here, and it is the next stage that knows these make
one word. The segmenter reports no problems.

### Lexer

`zakac lex` reads the pieces and makes *tokens*: numbers, names, reserved words, signs,
and the end of the input. It joins the pieces of a reserved word or phrase, so
`ပြ သ မည်` becomes one `print` token. When a sentence-final particle (`သည်`, `ပါ`,
`ပါသည်`) follows a verb, the lexer keeps it on the verb, as the `particle` of that token.

```text
number  (burmese digits)    ၁
punct   plus                +
number  (burmese digits)    ၂
kw      ko                  ကို
kw      print               ပြသမည်
punct   full_stop           ။
eof
```

Each token also records whether it is `joined` to the one before, with no space between.
The lexer owns **ZK01**: a character that Zaka does not use, a malformed number, text
(string) with no closing quote, an unknown escape, or a name in braces that does not
close. See [The lexer](../internals/lexer.md).

### Parser

`zakac parse` reads the tokens as sentences and builds the *tree*. A number is a leaf, `+`
makes a node with a left side and a right side, and `ကို` with the verb makes a `print`
statement whose slot is `ko`:

```text
print (slot ko)
└─ add
   ├─ int 1
   └─ int 2
```

The parser checks the grammar of a sentence, such as which particle goes with which verb,
but it knows nothing at this point about what a name means. It owns **ZK02**: a missing
`။`, a missing operand, a particle in the wrong place, a statement with no verb, or a
number too big for its data kind. See [The parser](../internals/parser.md).

When the file imports other files, this stage reads them too: it lexes and parses each
imported file before the file that imports it, and puts the statements of all the files
into one tree, imported files first. The later stages see that one tree. The errors of
the import sentence and the export mark, ZK0229 to ZK0232, belong to this stage; see
[Modules](../language/modules.md).

### Natural wording

When the lexer and the parser found no error, the naturalness check reads the tree for
wording that is valid but unlike Burmese, such as `ကြီးသည်လျှင်` where `ကြီးလျှင်` sounds
better. It reports **ZK05** warnings, which are errors under `--lint-error naturalness`.
The sentence above has none. See [Natural wording](../language/naturalness.md).

### Names and data kinds

`zakac semcheck` resolves each name to its declaration, works out the *data kind* (type)
of each expression, and checks that the data kinds fit. It owns **ZK03** (names and
scopes) and **ZK04** (data kinds), and among them the rules of unsafe code:

| Code | Rule |
| --- | --- |
| ZK0468 | pointer work outside unsafe code |
| ZK0469 | an unsafe function used as a value |
| ZK0470 | an owned marker on something that is not a pointer |
| ZK0471 | unsafe code under `--no-unsafe` |
| ZK0472 | a C declaration that is not declared unsafe |

The artifact keeps the tree and adds a table from each expression to its data kind:

```text
0 (the 1)    isize
1 (the 2)    isize
2 (the add)  isize
```

An integer written with no data kind is an `ကိန်းပြည့်` (`isize` in messages). This stage
also works out each list count written as an expression, such as
`အကွက် N * ၂ ခုရှိသော`, with the same math the folding step uses, and writes it back as a
number, so each later stage sees the count. See [Names and data
kinds](../internals/semcheck.md).

### IR builder and folding

`zakac irgen` lowers the tree to the IR, a smaller form of the program with the data kind
written on each expression. The next step, *folding*, works out what can be worked out
before the program runs:

```text
print
└─ const 3 : isize
```

The add is gone. Without folding (`--fold none`), the IR keeps `add` with `const 1` and
`const 2` under it. Folding owns **ZK06**: a sum of constants that does not fit its data
kind, a division with the constant zero as divisor, or a constant cell number (index)
outside its list.

### The checker

`zakac borrowck` follows the values of the program through the IR and reports **ZK07**.
Its flow checks run on each program and find a read of a value before it is set, a read
past the end of a block, or an operation that overflows for each value. Its pointer lint
runs on unsafe code and finds a block used after it was given back, a block never given
back, the address of a local that escapes, or a read of a place after an owned hand-over.
The sentence above has no pointer, so there is nothing to find.

The artifact adds a *digest* of each function: what it frees, resizes or hands over
through its parameters, and which parameters its result can point into. A call site uses
the digest instead of looking inside the function again. The checker also marks the cell
reads it has proved to be in range, so the code generator can leave out the run-time
check for those. See [Compiler checks](checks.md) and [The flow
checker](../internals/borrow-checker.md).

### Code generation

`zakac build` turns the IR into the VM's operations, with a table that maps each
operation back to a range of the source. With `--backend c` or `--backend native` it turns
the same IR into a native program instead ([Building programs](build.md)):

```text
fn ပရိုဂရမ်
   0  push 3:isize
   1  print
   2  ret

fn main
   0  call f0
   1  ret
```

[Instructions](../vm/instructions.md) lists the operations, and [The VM
target](../internals/vm-target.md) shows how the compiler makes them.

### The run

`zakac run` runs the program. On the VM, `main` calls the program start block, which
pushes `3`, prints it as `၃` and returns. With neither `--vm` nor `--jit`, a source file
runs as native code through Cranelift when it can, and on the VM otherwise ([The VM or
Cranelift](commands.md#the-vm-or-cranelift)). A run reports no code: when a program does
something it must not, the VM or the native code stops it and reports it ([When a program
stops](../vm/traps.md)).

## Artifacts

Each stage command can write what its stage made. The artifact is a YAML file, and its
first line names the stage that made it:

```console
$ zakac parse sum.zaka | head -3
# zakac-parse-artifact v1
name:
  Real:
```

Each artifact has the name of the source file, the *source text itself* (so that a span
can be shown without the file), and what the stage made. Each one after the segmenter's
has the diagnostics found so far as well, which is how a saved `.zkc` file can still print
a warning with the line it points at. The raw form of an artifact has the same data in a
compact encoding: it starts with `ZKAR`, the version and the name of the stage. The
reader checks the name first, so a raw file of one stage is never read as the artifact of
another.

| Stage | Command | What it has besides the source and the diagnostics |
| --- | --- | --- |
| segmenter | `segment` | the pieces (no diagnostics) |
| lexer | `lex` | the tokens, the text of each string, the comments |
| parser | `parse` | the tree |
| names and data kinds | `semcheck` | the tree and the data kind of each expression |
| IR builder, folding | `irgen` | the IR after folding |
| checker | `borrowck` | the IR and the digest of each function |
| codegen | `build` | the module of operations, the one `run` reads |

[Commands](commands.md) lists the flags and the file extensions.
