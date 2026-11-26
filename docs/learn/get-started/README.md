---
title: Get started
description: Run Zaka in the browser, install the compiler, set up an editor, and run a first program.
---

# Get started

Zaka runs in two places: in the browser, through the [Zaka Explorer](/explorer/), and on
a computer, through the compiler, `zakac`. The explorer needs no setup, so it is the
fastest start with the language, and the pages of this chapter set up a computer for
the rest of the work.

## Run it in the browser

Open the [Zaka Explorer](/explorer/), paste this program into the editor, and press
**Run**:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ ကမ္ဘာကြီး" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ ကမ္ဘာကြီး
```

The explorer also shows each stage (compiler pass) of the program, from the words the
lexer finds to the operations the VM runs, and each example in these pages runs there
as it is.

## Set up a computer

Three short pages cover the setup, in order:

1. [Installation](installation.md): install `zakac` and `zakafmt` with one command, or
   download them, or build them from source.
2. [Editor setup](editors.md): the extension for VS Code and Cursor, Zed and other
   editors, the fonts and input methods for Burmese text, and
   [Ghostty Myanmar](editors.md#terminal), a terminal that draws Burmese text as written.
3. [Quick start](quick-start.md): write a program in a file, run it, format it, and build
   it as an executable.

Programmers who know another language can then read the tour, [Zaka for
programmers](../tour.md), and others can go on to [Hello, world](../hello/README.md),
the first lesson.

Next: [Installation](installation.md).
