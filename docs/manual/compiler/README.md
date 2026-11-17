---
title: The compiler
description: An overview of zakac, the compiler that turns Zaka sentences into ops for the Zaka VM, or into machine code through Cranelift or C.
---

# The compiler

`zakac` reads a Zaka program, checks it, and turns it into the ops that the Zaka VM
runs, or into machine code: through Cranelift (`zakac build --target native`, or
`zakac run --jit`, which runs it at once) or through C (`zakac build --target c`). It is
built as a chain of small stages. Each stage takes what the one before it
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
IR builder ──► constant folding ──► checker ──► codegen ──► the VM
 the IR         ZK06                 ZK07        the ops      runs the ops
```

The segmenter and the lexer work as one step in practice, and so do the IR builder and
constant folding. Each stage can also be run alone, up to itself, with its own command,
and write an artifact to look at.

| Page | Covers |
| --- | --- |
| [Commands](commands.md) | each `zakac` command and option, with sessions to run |
| [Configuration](configuration.md) | config files: where the files are, which one wins, and how each option is named in them |
| [Stages](stages.md) | what each stage takes in and gives out, and one sentence followed through all of them |
| [Diagnostics](diagnostics.md) | how a message is set out, levels, codes, and what stops the chain |
| [Pointer checks](checks.md) | the settings that tune how pointer mistakes are reported |
| [Editors](editors.md) | the language server, and its setup in VS Code, Zed and other editors |
| [Native programs](native.md) | machine code through Cranelift or C: what a native program needs, checks, and where it differs from the VM |

## Running a program

One command takes a source file through all the stages and runs it:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ ကမ္ဘာကြီး" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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
- To set options once for a project or a user: [Configuration](configuration.md).
- To see errors and kinds while editing: [Editors](editors.md).
- To build a program of its own, or run with no VM: [Native programs](native.md).
- To see what happens to a sentence: [Stages](stages.md).
- To read how a stage is built: the chapter [Inside the compiler](../internals/README.md).
- To learn what the checks follow: [Compiler checks](checks.md) and [Unsafe
  Zaka](../language/unsafe.md).
