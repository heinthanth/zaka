---
title: Example programs
description: Complete programs that combine the parts from the earlier chapters.
---

# Example programs

The earlier chapters covered the parts of the language one at a time, and this chapter
puts them together in complete programs. We can paste each program into the
[Explorer](/explorer/) and run it, and after each one, a short walk through the code
explains the parts that are new.

## You will learn

- How functions, loops, lists and text work together in a whole program.
- How to arrange a program: small functions first, then the program start block.
- How a program reads its command line and reports an exit code.

## The programs

| Program | Uses |
| --- | --- |
| [FizzBuzz](fizzbuzz.md) | functions with results, true/false variables, if / else-if chains, counted loops |
| [Greatest common divisor](gcd.md) | while loops, functions that call functions, parameters that change |
| [Fibonacci numbers](fibonacci.md) | generic loops, returning a variable, converting to decimal numbers |
| [The Collatz sequence](collatz.md) | a helper function that two functions share, returning before the end |
| [Scores](scores.md) | loops over each cell and each character, building a list, removing a cell, template literals |
| [Where a program starts](start.md) | the program start block (entry point, `main`), the arguments of the command line, an exit code |

These programs, and more, are also in the `samples/programs` folder of the Zaka source
tree.

## How the programs are laid out

The first five programs share the same shape:

1. Small functions at the top, each with a `///` comment that explains what it does.
2. The program start block at the bottom, from `ပရိုဂရမ်သည်` to
   `ပရိုဂရမ် ပြီးဆုံးပါပြီး။`, with the sentences that run when the program starts.

We can call a function above its declaration, so the program start block could also come
first. Either order works, as long as we pick one and keep to it. The program start block
can also receive the arguments of the command line, which [Where a program
starts](start.md) shows.

Next: [FizzBuzz](fizzbuzz.md).
