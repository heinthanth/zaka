---
title: The compiler
description: An overview of zakac, the compiler that turns Zaka sentences into operations for the Zaka VM, or into machine code through Cranelift or C.
---

# The compiler

`zakac` reads a Zaka program, checks it, and turns it into the operations (instructions)
that the Zaka VM (virtual machine) runs, or into machine code: through Cranelift
(`zakac build --backend native`, or `zakac run`, which runs it at once) or through C
(`zakac build --backend c`). It is built as a chain of small stages (compiler passes).
Each stage reads what the stage before it made, adds its checks and passes the result on,
and a stage that finds an error ends the chain, so the program does not run. [Hello,
world](../../learn/hello/README.md#running-it) shows the first run.

```text
source text
    │
    ▼
segmenter ──► lexer ──► parser ──► naturalness ──► names and data kinds
 pieces       tokens    tree       warnings ZK05    ZK03, ZK04
              ZK01      ZK02
                                                        │
    ┌───────────────────────────────────────────────────┘
    ▼
IR builder ──► constant folding ──► checker ──► codegen ──► the VM or native code
 the IR         ZK06                 ZK07        operations   runs the program
```

In practice the segmenter and the lexer work as one step, and so do the IR builder and
constant folding. We can also run the chain up to each stage with its own command, and
write an artifact to look at.

| Page | Covers |
| --- | --- |
| [Commands](commands.md) | each `zakac` command and flag, with sessions to run |
| [Building programs](build.md) | the targets of `zakac build`: the VM, C, Cranelift and C libs, with their flags |
| [Configuration](configuration.md) | config files: where the files are, which one wins, and how each flag is named in them |
| [Stages](stages.md) | what each stage reads and writes, and one sentence followed through all of them |
| [Diagnostics](diagnostics.md) | how a message is set out, levels, codes, and what stops the chain |
| [Compiler checks](checks.md) | the flow checks, the pointer lint and the settings that tune them |
| [Native programs](native.md) | machine code through Cranelift or C: what a native program needs, what it checks, and where it differs from the VM |
| [Editors](editors.md) | the language server, and its setup in VS Code, Zed and other editors |

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

`zakac run` runs a source file as native code through Cranelift when it can, and on the
VM otherwise, and `zakac run --vm` uses the VM in each case ([The VM or
Cranelift](commands.md#the-vm-or-cranelift)). When the compiler finds an error, it prints
a message with a code (`ZK0401`) and the exit code is 1. The page of [error
messages](../reference/errors.md) lists each code.

## Where to read next

- To use the compiler: [Commands](commands.md), then [Diagnostics](diagnostics.md).
- To build an executable or a C lib (library): [Building programs](build.md).
- To set flags once for a project or a user: [Configuration](configuration.md).
- To see errors and data kinds (types) while editing: [Editors](editors.md).
- To build a program that runs without `zakac`, or run with no VM: [Native
  programs](native.md).
- To see what happens to a sentence: [Stages](stages.md).
- To read how a stage is built: the chapter [Inside the compiler](../internals/README.md).
- To learn what the checks follow: [Compiler checks](checks.md) and [Unsafe
  Zaka](../language/unsafe.md).
