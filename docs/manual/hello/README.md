---
title: Hello, world
description: A first Zaka program, and how to run it.
---

# Hello, world

A *program* is a list of instructions for the computer. Tradition holds that the first
program in a new language prints a greeting. Here it is in Zaka:

```zaka
// The rest of a line after "//" is a comment for people. The computer skips it.

// Print the text inside the quotes.
"မင်္ဂလာပါ ကမ္ဘာကြီး" ဟု ပြသပါ။
```

```text
မင်္ဂလာပါ ကမ္ဘာကြီး
```

The line reads as a Burmese sentence: *"Hello, world" — thus, please show.* Each part
has a job:

| Zaka | Reads as | Job |
| --- | --- | --- |
| `"မင်္ဂလာပါ ကမ္ဘာကြီး"` | "Hello, world" | the text to print, in double quotes |
| `ဟု` | "so" (quoting) | marks the words before it as a quote |
| `ပြသပါ` | "please show" | the command: print a line |
| `။` | full stop | ends the sentence |

Each Zaka statement ends with `။`, the Burmese full stop. A line break does **not** end
a statement; `။` alone does.

## Running it

1. Open the [Zaka Explorer](https://zaka-lang.org/explorer/), paste the program, and
   press **Run**. Or:
2. Save the program as `hello.zaka` and run it with the compiler:

   ```sh
   zakac run hello.zaka
   ```

## More than one line

A program runs from top to bottom, one statement after another. A program can also
print numbers, and it can do sums before it prints them.

```zaka
"မင်္ဂလာပါ ကမ္ဘာကြီး" ဟု ပြသပါ။
"စံကား ဘာသာစကားမှ ကြိုဆိုပါသည်" ဟု ပြသပါ။   // "Welcome to the Zaka language"
၂၀၂၆ ကို ပြသပါ။                            // a number: show 2026
၆ * ၇ ကို ပြသပါ။                            // a sum: show 6 times 7
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
"မင်္ဂလာပါ" ဟု ပြသပါ
```

```text
error[ZK0201]: expected "။", found end of input
```

Each message has a code such as `ZK0201`. The [error messages](../reference/errors.md)
page lists the common ones.

## Hands on

- Change the greeting so that it greets a friend instead of the world.
- Add a line that prints `၁၀၀ + ၂၃`.
- Put two statements on the same line. Does it still run?

Next: [Comments](comments.md).
