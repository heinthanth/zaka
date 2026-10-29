---
title: Zaka through Examples
description: Learn the Zaka programming language one small, runnable example at a time.
---

# Zaka through Examples

Zaka (စံကား) is a programming language written in Burmese. A Zaka program is a list of
Burmese sentences such as `"မင်္ဂလာပါ" ဟု ပြသပါ။` ("show: hello"). The computer carries
them out one after another.

These pages teach the language through examples. No other programming language is
needed beforehand. Each page has:

- a short program, with comments that explain each line,
- the exact output the program prints,
- a table that glosses the Burmese words used,
- a **Hands on** section that suggests a small change to make to the program.

Read the pages in order. Each page builds on nothing but what the pages before it have
shown.

## How to run the examples

The fastest route is the online **[Zaka Explorer](https://zaka-lang.org/explorer/)**.
Paste a program into the editor and press **Run**.

To run programs on a local computer, build the compiler `zakac` from the Zaka source
folder. This needs [Rust](https://www.rust-lang.org/tools/install) installed first.

```sh
cargo build --release -p zakac
```

Save a program in a file whose name ends in `.zaka` (or `.zk`), then run it:

```sh
./target/release/zakac run hello.zaka
```

## A note on the examples

- Example programs use Burmese names such as `အသက်` (age). The comment next to a name
  gives its meaning in English.
- Zaka prints numbers with Burmese digits (`၁၂၃`), even when the source writes them
  with Western digits (`123`).
- When an example is meant to fail, the page states this and shows the message the
  compiler prints.

Start with [Hello, world](hello/README.md), or browse the [table of
contents](CONTENTS.md).

These pages teach through examples and leave some details out. The
[Docs](../manual/README.md) give the full rules of the language, explain how the
compiler and the virtual machine work, and list the reserved words, the operator
precedence and each error code.
