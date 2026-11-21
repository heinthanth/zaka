---
title: Concepts
description: The ideas behind Zaka, explained through examples: sentences, Burmese text, storage and the checks.
---

# Concepts

The other chapters of the Docs state the rules, and these pages explain the reasons behind
them. Each page picks one idea of the design, describes the problem it solves, works
through an example, and names what it costs.

| Page | The question it answers |
| --- | --- |
| [The case for sentences](sentences.md) | Zaka uses Burmese sentences, with the verb at the end. What does that give us? |
| [Finding words in Burmese](segmenting.md) | Burmese has no spaces between words. How does the compiler find where a word ends? |
| [Storage, explained](storage.md) | How does Zaka count and free values and objects, what does the ring collector do, and what is left to unsafe code? |
| [Checks in the VM and in native code](checked.md) | Which mistakes are found before the run, which during it, and which are lost in a native build? |

```text
 a sentence             a word               a block                a check
 ──────────             ──────               ───────                ───────
 operands, then         where does it        who frees it,          found before the run,
 the verb               end?                 and when?              during it, or not at all
     │                      │                    │                      │
 sentences.md          segmenting.md         storage.md             checked.md
```

We can read the pages in a free order, but the last two go together: counted storage is
what keeps safe code free of storage mistakes, and the checks page explains what the
compiler finds in unsafe code before the run and what it leaves to the VM.

For the rules themselves, see [the language](../language/README.md), and for the VM
(virtual machine) that runs a program, see [the VM](../vm/README.md).
