---
title: Comments
description: Notes for people that the computer skips.
---

# Comments

A *comment* is a note for people who read the program. The computer skips it. Zaka has
three ways to start a comment. Each runs to the end of its line.

```zaka
// A comment can start with two slashes.
/// Or with three slashes, often used to describe what comes next.
မှတ်ချက် Or with the Burmese word for "note".

ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် ၃ ဖြစ်ပါစေ။ // after a statement
အရေအတွက် ကို ပြသပါ။ မှတ်ချက် also after a statement

// A comment can switch a line off without deleting it:
// အရေအတွက် ကို ၀ ဟု ထားပါ။
အရေအတွက် * ၂ ကို ပြသပါ။
```

```text
၃
၆
```

The program makes a number called `အရေအတွက်` (count), prints it, and prints it doubled.
The [Variables](../variables/README.md) chapter explains how that works. For now, look at
the comments: none of them change what the program prints.

| Zaka | Reads as | Job |
| --- | --- | --- |
| `//` | — | comment to the end of the line |
| `///` | — | same; by habit, describes the code below it |
| `မှတ်ချက်` | "note" | same, in Burmese |

Zaka has no comment that spans several lines. Start each line with its own marker.

## `မှတ်ချက်` must stand alone

`မှတ်ချက်` starts a comment only when it is a word on its own, with a space after it.
Joined to the next word, it is read as program text:

```zaka-error ZK0202
မှတ်ချက်ကို မမေ့ပါနှင့်
```

```text
error[ZK0202]: expected an operand, found "မှတ်ချက်"
```

## Try it

- Add a comment above each line of the [Hello, world](README.md) program that says what
  the line does.
- Switch off the last line of the example above with `//`. What prints now?

Next: [Printing](printing.md).
