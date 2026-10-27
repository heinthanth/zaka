---
title: Example programs
description: Complete programs that combine all the parts from the earlier chapters.
---

# Example programs

All parts of the language are now covered. The programs in this chapter put the parts
together. Each one is a complete program: paste it into the
[Explorer](https://zaka-lang.org/explorer/) and run it. After each program, a short
walk-through explains how it is built.

| Program | Uses |
| --- | --- |
| [FizzBuzz](fizzbuzz.md) | functions with results, boolean variables, if / else-if chains, counted loops |
| [Greatest common divisor](gcd.md) | while loops, functions calling functions, changing parameters |
| [Fibonacci numbers](fibonacci.md) | generic loops, the implicit `ရလဒ်`, converting to decimals |
| [The Collatz sequence](collatz.md) | a shared helper function, returning before the end |

These programs, and more, are also in the `samples/programs` folder of the Zaka source
tree.

## Common shape

All four programs follow the same structure:

1. Small functions at the top, each with a `///` comment telling what it does.
2. The *main program* at the bottom: the top-level statements that run when the program
   starts.

Because a function can be called before its declaration, the main program could also
come first. Choose one structure and keep to it.

Next: [FizzBuzz](fizzbuzz.md).
