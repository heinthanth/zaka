---
title: Example programs
description: Complete programs that combine all the parts from the earlier chapters.
---

# Example programs

All parts of the language are now covered. The programs in this chapter put the parts
together. Each one is a complete program: paste it into the
[Explorer](/explorer/) and run it. After each program, a short
walk-through explains how it is built.

| Program | Uses |
| --- | --- |
| [FizzBuzz](fizzbuzz.md) | functions with results, boolean variables, if / else-if chains, counted loops |
| [Greatest common divisor](gcd.md) | while loops, functions calling functions, changing parameters |
| [Fibonacci numbers](fibonacci.md) | generic loops, a variable that is given back, converting to decimals |
| [The Collatz sequence](collatz.md) | a shared helper function, returning before the end |
| [Scores](scores.md) | each-cell and each-character loops, building a list with adds, removing a cell, template literals |
| [Where a program starts](start.md) | a start block, the arguments of the command line, an exit code |

These programs, and more, are also in the `samples/programs` folder of the Zaka source
tree.

## Common shape

All five programs follow the same structure:

1. Small functions at the top, each with a `///` comment telling what it does.
2. The start block at the bottom, `ပရိုဂရမ်သည်`, with the statements that run when the
   program starts.

Because a function can be called before its declaration, the start block could also
come first. Choose one structure and keep to it. The start block can also take the
arguments of the command line: see [Where a program starts](start.md).

Next: [FizzBuzz](fizzbuzz.md).
