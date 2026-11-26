---
title: Hello, world
description: A first Zaka program, and how to run it.
---

# Hello, world

A *program* is a list of instructions for the computer. A first program in a new
language often prints a greeting, so we start with one here and then see how to run it.

## You will learn

- How to write a sentence (statement) that prints a line of text (string).
- Where a program starts and where it ends: the program start block (entry point, `main`).
- How to run a program in the Explorer or with the compiler, `zakac`.
- What happens when a sentence has no full stop.

## The first program

Here is the greeting in Zaka:

```zaka
// The rest of a line after "//" is a comment for people. The computer skips it.

// The program starts here.
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
// Print the text inside the quotes.
"မင်္ဂလာပါ ကမ္ဘာကြီး" ဟု ပြသမည်။
// The program ends here.
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ ကမ္ဘာကြီး
```

The greeting line is a Burmese sentence (statement). `ဟု` marks the quoted text and
`ပြသမည်` means "will show". To print a line of text (string), put it in double quotes
and follow it with `ဟု ပြသမည်။`.

```
"<text>" ဟု ပြသမည်။
```

To print a different greeting, change the text between the quotes.

| Zaka | Reads as | What it does |
| --- | --- | --- |
| `"မင်္ဂလာပါ ကမ္ဘာကြီး"` | "Hello, world" | the text to print, in double quotes |
| `ဟု` | "so" (quoting) | marks the preceding text as a quote |
| `ပြသမည်` | "will show" | the command: print a line |
| `။` | full stop | ends the sentence |

We end each Zaka sentence with `။`, the Burmese full stop. A new line does **not** end a
sentence, so we need the `။` even at the end of a line.

## Where the program starts

Each program needs a program start block (entry point, `main`), and we write it with
two fixed lines: `ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။` to start the program and
`ပရိုဂရမ် ပြီးဆုံးပါပြီး။` to end it. The sentences we want to run go between them:

```
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
<sentences>
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

| Zaka | Reads as |
| --- | --- |
| `ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။` | "the program will do the following" |
| `ပရိုဂရမ် ပြီးဆုံးပါပြီး။` | "the program is done" |

The program starts at `ပရိုဂရမ်သည်` and runs the sentences inside the block in order.
The last line is the block closer (end of a block). A program has one program start
block. Later examples often leave it out and let [REPL
mode](../../manual/language/functions-start-block.md#repl-mode) create it. See
[Where a program starts](../programs/start.md) for more on the block.

## Running it

We can run a program in two places:

1. Open the [Zaka Explorer](/explorer/), paste the program, and press **Run**.
2. Save the program as `hello.zaka` and run it with the compiler:

   ```sh
   zakac run hello.zaka
   ```

Both print `မင်္ဂလာပါ ကမ္ဘာကြီး`. To install `zakac` on a computer, see
[Installation](../get-started/installation.md).

## More than one line

A program start block runs from top to bottom, one sentence at a time. Besides text, we
can print numbers, and we can work out a sum such as `၆ * ၇` before we print it.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ ကမ္ဘာကြီး" ဟု ပြသမည်။
"စံကား ဘာသာစကားမှ ကြိုဆိုပါသည်" ဟု ပြသမည်။ // "Welcome to the Zaka language"
၂၀၂၆ ကို ပြသမည်။ // a number: show 2026
၆ * ၇ ကို ပြသမည်။ // a sum: show 6 times 7
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ ကမ္ဘာကြီး
စံကား ဘာသာစကားမှ ကြိုဆိုပါသည်
၂၀၂၆
၄၂
```

We use `ဟု` after text in quotes, and `ကို` ("show *this*") after a number:

```
<number> ကို ပြသမည်။
```

The page on [printing](printing.md) explains when to use each one.

## Forgetting the full stop

When we leave out the `။`, the compiler expects it before the next line of code:

```zaka-error ZK0201
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ" ဟု ပြသမည်
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0201]: expected "။", found "ပရိုဂရမ် ပြီးဆုံးပါပြီး"
help: insert "။"
```

The fix inserts the missing `။` after `ပြသမည်`. With the full stop in place, the program
runs.

## Details

- **Other spellings of the block closer.** We can also write the last line in other
  forms, such as `ပရိုဂရမ် ပြီးပါပြီ။`. See [Four closing forms](../variables/scope.md#four-closing-forms).
- **Error codes.** Each error message comes with a code such as `ZK0201`, and the [error
  messages](../../manual/reference/errors.md) page lists the common ones.
- **Fix hints.** When the compiler knows how to fix an error, it shows the line with the
  fix in place and puts `~` under the new text. In this example, the `။` at the end of
  the last line is missing:

  ```console
  $ zakac run hello.zaka
  error[ZK0201]: expected "။", found end of input
   --> hello.zaka:4:1
    |
  4 |
    | ^
    = help: insert "။"
    |
  3 | ပရိုဂရမ် ပြီးဆုံးပါပြီး။
    |                 ~
  ```

  The compiler does not change the file, so we still make the edit ourselves.
- **Native code and the VM.** `zakac run` runs a source file as native code when it can,
  and on the VM (virtual machine) otherwise. We can use `zakac run --vm` to run it on the
  VM, which checks more while the program runs, and `zakac build --backend native` or
  `--backend c` to build a standalone program. Both backends need a C compiler on the
  PATH to link the program. See [Native programs](../../manual/compiler/native.md).

## Recap

- We end each sentence with `။`, because a new line does not end it.
- We use `"<text>" ဟု ပြသမည်။` to print a line of text and `<number> ကို ပြသမည်။` to
  print a number.
- Each program has one program start block, from `ပရိုဂရမ်သည်` to
  `ပရိုဂရမ် ပြီးဆုံးပါပြီး။`, and its sentences run from top to bottom.
- We can run a program in the [Zaka Explorer](/explorer/) or with `zakac run`.
- When a `။` is missing, the compiler stops and prints an error with its code.

## Hands on

- Change the greeting so that it greets a friend instead of the world.
- Add a line that prints `၁၀၀ + ၂၃`.
- Put two sentences on the same line. Does the program still run?
- Move a sentence above `ပရိုဂရမ်သည်`. What does the compiler report?

Next: [Comments](comments.md).
