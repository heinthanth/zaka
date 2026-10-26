---
title: Zaka by Example
description: Learn the Zaka programming language one small, runnable example at a time.
---

# Zaka by Example

Zaka (စံကား) is a programming language written in Burmese. A Zaka program is a list of
Burmese sentences such as `"မင်္ဂလာပါ" ဟု ပြသပါ။` ("say: hello"). The computer carries
them out one after another.

This manual teaches the language through examples, in the style of *Rust by Example*.
You do not need to know any other programming language first. Every page has:

- a short program, with comments that explain each line,
- the exact output the program prints,
- a table that glosses the Burmese words used,
- a **Try it** section with a small change for you to make.

Read the pages in order. Each page uses only what the pages before it have shown.

## How to run the examples

The quickest way is the online **[Zaka Explorer](https://zaka-lang.org/explorer/)**.
Paste a program into the editor and press **Run**.

To run programs on your own computer, build the compiler `zakac` from the Zaka source
folder. You need [Rust](https://www.rust-lang.org/tools/install) installed.

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
- Zaka prints numbers with Burmese digits (`၁၂၃`), even when you write them with
  Western digits (`123`).
- When an example is meant to fail, the page says so and shows the message the
  compiler prints.

Start with [Hello, world](hello/README.md), or browse the [table of
contents](SUMMARY.md).
