---
title: Hello, world
description: A first Zaka program, and how to run it.
---

# Hello, world

A *program* is a list of instructions for the computer. Tradition holds that the first
program in a new language prints a greeting. Here it is in Zaka:

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

The line in the middle reads as a Burmese sentence: *"Hello, world" — thus, please
show.* Each part has a job:

| Zaka | Reads as | Job |
| --- | --- | --- |
| `"မင်္ဂလာပါ ကမ္ဘာကြီး"` | "Hello, world" | the text to print, in double quotes |
| `ဟု` | "so" (quoting) | marks the words before it as a quote |
| `ပြသမည်` | "please show" | the command: print a line |
| `။` | full stop | ends the sentence |

Each Zaka statement ends with `။`, the Burmese full stop. A line break does **not** end
a statement; `။` alone does.

## Where the program starts

The first and the last line hold the program together. The program starts at
`ပရိုဂရမ်သည်`, runs the statements after it one after another, and ends at the closer.

| Zaka | Reads as |
| --- | --- |
| `ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။` | "the program will do the following" |
| `ပရိုဂရမ် ပြီးဆုံးပါပြီး။` | "the program is done" |

This is the *start block*. Each program has one and no more, and each statement that
runs goes inside it. The examples on these pages all start like this; [Where a program
starts](../programs/start.md) shows what more a start block can do.

The closer has other accepted spellings, such as `ပရိုဂရမ် ပြီးပါပြီ။`: see [Four closing
forms](../variables/scope.md#four-closing-forms).

## Running it

1. Open the [Zaka Explorer](/explorer/), paste the program, and
   press **Run**. Or:
2. Save the program as `hello.zaka` and run it with the compiler:

   ```sh
   zakac run hello.zaka
   ```
3. Or build it into a program of its own, through C. This needs a C compiler on the
   PATH, such as GCC, Clang or `zig cc`:

   ```sh
   zakac build --target c hello.zaka
   ./hello
   ```

   [Native programs](../../manual/compiler/native.md) tells what such a program checks.

## More than one line

A start block runs from top to bottom, one statement after another. A program can also
print numbers, and it can do sums before it prints them.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ ကမ္ဘာကြီး" ဟု ပြသမည်။
"စံကား ဘာသာစကားမှ ကြိုဆိုပါသည်" ဟု ပြသမည်။   // "Welcome to the Zaka language"
၂၀၂၆ ကို ပြသမည်။                            // a number: show 2026
၆ * ၇ ကို ပြသမည်။                            // a sum: show 6 times 7
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မင်္ဂလာပါ ကမ္ဘာကြီး
စံကား ဘာသာစကားမှ ကြိုဆိုပါသည်
၂၀၂၆
၄၂
```

Text in quotes takes `ဟု`; a number takes `ကို` ("show *this*"). The page on
[printing](printing.md) explains the difference.

## Forgetting the full stop

If a statement has no `။`, the program does not run. The compiler stops and reports what
it expected:

```zaka-error ZK0201
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"မင်္ဂလာပါ" ဟု ပြသမည်
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0201]: expected "။", found "ပရိုဂရမ် ပြီးဆုံးပါပြီး"
```

Each message has a code such as `ZK0201`. The [error messages](../../manual/reference/errors.md)
page lists the common ones.

## Hands on

- Change the greeting so that it greets a friend instead of the world.
- Add a line that prints `၁၀၀ + ၂၃`.
- Put two statements on the same line. Does it still run?
- Move a statement above `ပရိုဂရမ်သည်`. What does the compiler report?

Next: [Comments](comments.md).
