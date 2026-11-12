---
title: Zaka Docs
description: The full rules of the Zaka language, how the compiler and the VM work, and the ideas behind them.
---

# Zaka Docs

These pages are the manual of Zaka (စံကား). Where [Learn](../learn/README.md) teaches the
language one small example at a time, the manual states the rules in full: what each
sentence means, what the compiler checks, what the VM does while a program
runs, and the reasons behind each choice.

## How the manual is laid out

| Chapter | For | What it holds |
| --- | --- | --- |
| [The language](language/README.md) | programmers | the exact rules of words, kinds, sentences, functions, lists, text, structs and modules, then shared objects and unsafe Zaka |
| [The compiler](compiler/README.md) | programmers | the `zakac` commands, the stages a program passes through, and its messages |
| [The VM](vm/README.md) | programmers | how the Zaka VM runs a program, holds its data, and stops on a mistake |
| [Concepts](concepts/README.md) | the curious | the ideas behind the design, explained with scenarios |
| [Inside the compiler](internals/README.md) | contributors | how the code of the compiler is built, crate after crate |

The language chapter climbs from values to storage. Its pages state the rules of safe
Zaka: [words](language/words.md), [kinds](language/kinds.md),
[conversions](language/conversions.md), [expressions](language/expressions.md),
[statements](language/statements.md), [names and scope](language/scope.md),
[functions](language/functions.md), [function values](language/function-values.md),
[lists](language/lists.md), [text](language/text.md), [structs](language/structs.md),
[values and objects](language/objects.md), [modules](language/modules.md) and [natural
wording](language/naturalness.md). The last page, [Unsafe Zaka](language/unsafe.md),
states the rules of unsafe code: pointers, the heap and the checks on them.

The reference pages at the end list the [reserved words](reference/reserved-words.md),
the [operator precedence](reference/precedence.md), the [grammar](reference/grammar.md) and
each [error code](reference/errors.md).

## From a sentence to a running program

Each chapter looks at one part of this road:

```text
 program.zaka                    The language: what the sentences mean
      │
      ▼
 ┌─────────┐   words   ┌────────┐   tree   ┌──────────────────────────┐
 │  lexer  │ ────────▶ │ parser │ ───────▶ │ natural wording,         │
 └─────────┘           └────────┘          │ names and kinds          │
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

A mistake the compiler finds stops the program before it runs, with a
message and a code such as `ZK0701`. A mistake that slips past the checks stops the
program while it runs, with a message that starts `the program stopped:`.

## Reading the examples

The examples in the manual run, just as the ones in Learn do: press **Run** to see the
output, or change the program and run it again. Each example is checked against the
compiler, so the output shown is the output the program prints now.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ" ဟု ပြသမည်။          // show: hello
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ
```

An example meant to fail states it, and names the code of the message it gives. A
program that prints Western digits in its source still prints Burmese digits (`၁၂၃`).

Start with [the language](language/README.md), or open the
[table of contents](CONTENTS.md).
