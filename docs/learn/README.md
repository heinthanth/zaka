---
title: Zaka through Examples
description: Learn the Zaka programming language one small, runnable example at a time.
---

# Zaka through Examples

Zaka (စံကား) is a programming language based on a subset of Burmese. We write a Zaka
program as a list of Burmese sentences (statements), such as `"မင်္ဂလာပါ" ဟု ပြသမည်။`
("show: hello"), and we put them inside a program start block (entry point, `main`) so
that the computer runs them one after another.

Programmers who know another language can start with the tour, [Zaka for
programmers](tour.md), and the [Terms](../manual/reference/terms.md) page lists each Zaka
term next to the standard one.

These pages teach the language through examples, and no earlier programming knowledge is
needed. Each page has these parts:

- **You will learn**: a short list of what the page teaches.
- A small program, with comments that explain each line, and the exact output it prints.
- A pattern line that shows the shape of a sentence, and a table that translates the
  Burmese words.
- **Details**: the finer rules, kept apart from the main text.
- **Recap**: the main points, in brief.
- **Hands on**: small changes to make to the program.

When a page uses a Zaka term for the first time, the standard term follows in
parentheses, as in "data kind (type)".

Read the pages in order, because each page builds on the pages before it. When a page
mentions a later page, it also tells which page to finish before coming back. The pages
come in three parts:

1. **Values**, from [Hello, world](hello/README.md) to the [Example
   programs](programs/README.md): numbers, text, lists, structs, shared objects,
   functions, closures and loops. Each assignment and each call either makes a duplicate
   (copy) of a value or shares an object, and we never need to write code that frees
   storage.
2. **Copies and storage**, from [Copies and storage](forms/README.md): ranges, where the
   cells of a value live, the counted list, and what a duplicate costs.
3. **Unsafe Zaka**, from [Unsafe Zaka](pointers/README.md): unsafe code, pointers, blocks
   of the heap, and C functions that need addresses. This part is for advanced work.

A program from an earlier part still means the same thing in the later parts. The first
part is enough to write whole programs, while the other two parts give more control over
cost and storage.

## How to run the examples

The fastest option is the online **[Zaka Explorer](/explorer/)**.
Paste a program into the editor and press **Run**.

To run programs on a computer, we build the compiler, `zakac`, from the Zaka source
folder, which needs [Rust](https://www.rust-lang.org/tools/install) installed first.

```sh
cargo build --release -p zakac
```

Save a program in a file whose name ends in `.zaka` (or `.zk`), then run it:

```sh
./target/release/zakac run hello.zaka
```

## A note on the examples

- Example programs use Burmese names such as `အသက်` (age). The comment next to a name
  tells what it means in English.
- Zaka prints numbers with Burmese digits (`၁၂၃`), even when the program uses
  Western digits (`123`).
- When an example is meant to fail, the page points this out and shows the message the
  compiler prints.

Start with [Hello, world](hello/README.md), or browse the [table of
contents](CONTENTS.md).

These pages teach through examples, so some details are left out. The
[Docs](../manual/README.md) contain the full rules of the language, explain how the
compiler and the VM (virtual machine) work, and list the reserved words, the operator
precedence and each error code.
