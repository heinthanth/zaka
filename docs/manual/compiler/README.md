---
title: The compiler
description: An overview of zakac, the compiler that turns Zaka sentences into ops for the Zaka VM.
---

# The compiler

`zakac` reads a Zaka program, checks it, and turns it into the ops that the Zaka VM
runs. It is built as a chain of small stages. Each stage takes what the one before it
made, adds its own checks, and passes the result on; a stage that finds an error ends the
chain, and the program does not run.

```text
source text
    │
    ▼
segmenter ──► lexer ──► parser ──► naturalness ──► names and kinds
 pieces       tokens    tree       warnings ZK05    ZK03, ZK04
              ZK01      ZK02
                                                        │
    ┌───────────────────────────────────────────────────┘
    ▼
IR builder ──► constant folding ──► borrow check ──► codegen ──► the VM
 the IR         ZK06                 ZK07             the ops      runs the ops
```

The segmenter and the lexer work as one step in practice, and so do the IR builder and
constant folding. Each stage can also be run alone, up to itself, with its own command,
and write an artifact to look at.

| Page | Covers |
| --- | --- |
| [Commands](commands.md) | each `zakac` command and option, with sessions to run |
| [Stages](stages.md) | what each stage takes in and gives out, and one sentence followed through all of them |
| [Diagnostics](diagnostics.md) | how a message is set out, levels, codes, and what stops the chain |
| [Pointer checks](checks.md) | the settings that tune how pointer mistakes are reported |

## Running a program

One command takes a source file through all the stages and runs it:

```zaka
"မင်္ဂလာပါ ကမ္ဘာကြီး" ဟု ပြသပါ။
```

```text
မင်္ဂလာပါ ကမ္ဘာကြီး
```

```console
$ zakac run hello.zaka
မင်္ဂလာပါ ကမ္ဘာကြီး
```

When the compiler finds an error it prints a message with a code (`ZK0401`) and the
exit code is 1; the [error messages](../reference/errors.md) page lists each code.

## Where to read next

- To use the compiler: [Commands](commands.md), then [Diagnostics](diagnostics.md).
- To see what happens to a sentence: [Stages](stages.md).
- To read how a stage is built: the chapter [Inside the compiler](../internals/README.md).
- To learn what the checks follow: [Ownership and borrowing](../language/ownership.md).
