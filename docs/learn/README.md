---
title: Learn Zaka
description: Set up Zaka, then learn the language one small, runnable example at a time.
---

# Learn Zaka

Zaka (စံကား) is a programming language based on a subset of Burmese. A Zaka sentence
(statement) such as `"မင်္ဂလာပါ" ဟု ပြသမည်။` prints "hello". We put the sentences we want
to run in a program start block (entry point, `main`). The first chapter shows the block
in full. Most later examples leave it out, and [REPL
mode](../manual/language/functions-start-block.md#repl-mode) puts their top-level
statements in a program start block for us.

Programmers who know another language can start with the tour, [Zaka for
programmers](tour.md), and the [Terms](../manual/reference/terms.md) page lists each Zaka
term next to the standard one.

The lessons assume no programming experience. Each page follows the same format.

- **You will learn** lists the skills or ideas the page covers.
- A small program, with comments that explain each line, and the exact output it prints.
- A pattern line that shows the shape of a sentence, and a table that translates the
  Burmese words.
- **Details** explains special cases and links to the full rules.
- **Recap** sums up the main points.
- **Hands on** suggests small changes to make to the program.

When a page uses a Zaka term for the first time, the standard term follows in
parentheses, as in "data kind (type)".

Read the pages in order. Each lesson builds on earlier ones, and links to later pages
point out when a topic needs more background. The lessons come in three parts.

1. **Values**, from [Hello, world](hello/README.md) to the [Example
   programs](programs/README.md): numbers, text, lists, structs, shared objects,
   functions, closures and loops. Each assignment and each call either makes a duplicate
   (copy) of a value or shares an object, and we never need to write code that frees
   storage.
2. **Copies and storage**, from [Copies and storage](forms/README.md): ranges, where the
   cells of a value live, the counted list, and what a duplicate costs.
3. **Unsafe Zaka**, from [Unsafe Zaka](pointers/README.md): unsafe code, pointers, blocks
   of the heap, and C functions that need addresses. This part is for advanced work.

Part I covers what we need to write complete programs. Parts II and III give us more
control over how much work a program does and where it stores its data.

## How to run the examples

The fastest option is the online **[Zaka Explorer](/explorer/)**. Paste a program into the
editor and press **Run**.

To run programs on a computer, [Get started](get-started/README.md) installs the compiler,
`zakac`, and sets up an editor. Save a program in a file whose name ends in `.zaka` (or
`.zk`), and run it with `--repl` to support examples that leave out the program start
block:

```sh
zakac run --repl hello.zaka
```

## A note on the examples

- Example programs use Burmese names such as `အသက်` (age). The comment next to a name
  tells what it means in English.
- Zaka prints numbers with Burmese digits (`၁၂၃`), even when the program uses
  Western digits (`123`).
- When an example is meant to fail, the page points this out and shows the message the
  compiler prints.

Start with [Get started](get-started/README.md), or go straight to [Hello,
world](hello/README.md), or browse the [table of contents](CONTENTS.md).

These pages teach through examples, so some details are left out. The
[Docs](../manual/README.md) contain the full rules of the language, explain how the
compiler and the VM (virtual machine) work, and list the reserved words, the operator
precedence and each error code.
