---
title: Zaka Docs
description: The full rules of the Zaka language, how the compiler and the VM work, and the ideas behind them.
---

# Zaka Docs

These pages are the manual of Zaka (စံကား). [Learn](../learn/README.md) teaches the
language one small example at a time, while the manual states the rules in full: what
each sentence (statement) means, what the compiler checks, what the VM does while a
program runs, and the reasons behind each choice.

## How the manual is laid out

| Chapter | For | What it covers |
| --- | --- | --- |
| [The language](language/README.md) | developers | the full rules of words, data kinds (types), sentences, functions, lists, text, structs and modules, then shared objects and unsafe Zaka |
| [The compiler](compiler/README.md) | developers | the `zakac` commands, the stages that a program passes through, and the compiler messages |
| [The VM](vm/README.md) | developers | how the Zaka VM runs a program, stores its data, and stops on a mistake |
| [Concepts](concepts/README.md) | the curious | the ideas behind the design, explained with scenarios |
| [Inside the compiler](internals/README.md) | contributors | how the code of the compiler is built, crate after crate |

The language chapter goes from values to storage, and [its first
page](language/README.md) has a map of all its pages. Most of them state the rules of safe
Zaka, from [source text and words](language/words.md), [data kinds and
values](language/kinds.md), [expressions](language/expressions.md) and
[statements](language/statements.md) up to [functions](language/functions.md),
[lists](language/lists.md), [text](language/text.md), [structs](language/structs.md),
[values and objects](language/objects.md) and [modules](language/modules.md). The last
pages, from [Unsafe Zaka](language/unsafe.md) on, state the rules of unsafe code:
pointers, the heap and the checks on them.

The reference pages at the end list the [terms](reference/terms.md) with their standard
names, the [reserved words](reference/reserved-words.md), the [operator
precedence](reference/precedence.md), the [grammar](reference/grammar.md) and each [error
code](reference/errors.md).

## From a sentence to a running program

Each chapter covers one part of this road:

```text
 program.zaka                    The language: what the sentences mean
      │
      ▼
 ┌─────────┐   words   ┌────────┐   tree   ┌──────────────────────────┐
 │  lexer  │ ────────▶ │ parser │ ───────▶ │ natural wording,         │
 └─────────┘           └────────┘          │ names and data kinds     │
                                           └────────────┬─────────────┘
      The compiler: what it checks                      │ tree
                                                        ▼
 ┌────────────────┐   IR   ┌──────────────────┐  ┌──────────────┐
 │  flow checker  │ ◀───── │ fold constants   │◀─│ lower to IR  │
 └───────┬────────┘        └──────────────────┘  └──────────────┘
         │ IR
         ▼
 ┌────────────────┐   instructions   ┌──────────────┐
 │ code generator │ ───────────────▶ │   Zaka VM    │
 └────────────────┘                  └──────────────┘
                                       The VM: how it runs
```

A mistake that the compiler finds stops the program before it runs, with a message and a
code such as `ZK0701`. A mistake that gets past the checks stops the program while it
runs, with a message that starts with `the program stopped:`.

## Reading the examples

The examples in the manual run, just as the ones in Learn do: press **Run** to see the
output, or change the program and run it again. The tests check each example against the
compiler, so the output on the page is the output that the program prints now.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ" ဟု ပြသမည်။          // show: hello
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ
```

When an example is meant to fail, the page tells us so and names the code of the
message it gives.
Numbers print in Burmese digits (`၁၂၃`), even when the program writes them in Western
digits.

Start with [the language](language/README.md), or open the
[table of contents](CONTENTS.md).
