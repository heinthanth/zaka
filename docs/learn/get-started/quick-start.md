---
title: Quick start
description: Write a Zaka program in a file, run it, test sentences in the REPL, format the file, and build it as an executable.
---

# Quick start

With `zakac` installed and an editor set up, a first program takes a few minutes. This page
runs through the commands we use most: `run`, `repl`, `fmt` and `build`.

## Write a program

Make a folder for the program, and save these lines in it as `hello.zaka`:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ ကမ္ဘာကြီး" ဟု ပြသမည်။
၁ + ၂ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ ကမ္ဘာကြီး
၃
```

The first line starts the program start block (entry point, `main`), and the last line
ends it. Each sentence (statement) between them ends with `။`, the Burmese full stop.
[Hello, world](../hello/README.md) explains each word.

## Run it

In a terminal, in the folder of the program:

```console
$ zakac run hello.zaka
မင်္ဂလာပါ ကမ္ဘာကြီး
၃
```

When the Burmese letters show apart, the terminal cannot draw Burmese text, and
[Editor setup](editors.md#terminal) recommends one that can.

`zakac run` checks the program and runs it at once, as native code when it can and on the
VM otherwise. When the program has a mistake, `zakac` prints the error with its code, such
as `ZK0201`, and the line and column, and the
[Error codes](../../manual/reference/errors.md) page explains each code.

## Test sentences in the REPL

`zakac repl` runs one sentence at a time, with no file and no program start block, which
suits a quick test of a sentence:

```console
$ zakac repl
```

Enter a sentence such as `၂ * ၃ ကို ပြသမည်။` and press Enter to see its result. The
[REPL](../../manual/compiler/commands.md#repl) section lists its commands.

## Format it

`zakac fmt` rewrites a file with the standard spaces, line breaks and indent, and a
folder name formats each `.zaka` file under it:

```sh
zakac fmt hello.zaka
zakac fmt .
```

The editors run the same formatter, and `zakafmt` is the same command as a program of its
own.

## Build an executable

`zakac build --backend native` builds the program as an executable that runs with no
`zakac`, for the OS and the CPU of the computer:

```console
$ zakac build --backend native hello.zaka
$ ./hello
မင်္ဂလာပါ ကမ္ဘာကြီး
၃
```

The build needs a C compiler for the link, as [Installation](installation.md#what-else-to-install)
explains. On Windows, the executable is `hello.exe`. [Building
programs](../../manual/compiler/build.md) covers the other targets: the VM, C source and
C libs.

## Where next

- [Hello, world](../hello/README.md) starts the lessons, one idea at a time, for readers new
  to programming.
- [Zaka for programmers](../tour.md) covers the whole language on one page, for readers
  who know another language.
- [Tests](../../manual/language/tests.md) shows test blocks and `zakac test`, and
  [Packages](../modules/packages.md) shows a project with a `zakac.toml` and other
  packages.

Next: [Hello, world](../hello/README.md).
