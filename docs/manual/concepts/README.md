---
title: Concepts
description: The ideas behind Zaka, explained with scenarios: sentences, Burmese text, ownership and the checks.
---

# Concepts

The other chapters of the manual state rules. These pages tell the reasons. Each one
takes a single idea of the design, states what problem it answers, walks through a
scenario, and names the cost.

| Page | The question it answers |
| --- | --- |
| [The case for sentences](sentences.md) | What is the reason a program reads as Burmese sentences, with the verb last? |
| [Finding words in Burmese](segmenting.md) | Burmese has no spaces between words. How does the compiler find where a word ends? |
| [Ownership, explained](ownership.md) | What is the reason each heap block has one owner, and what does that gain? |
| [Checks in the VM and in native code](checked.md) | Which mistakes are found before the run, which during it, and which once a native build exists? |

```text
 a sentence             a word               a block                a check
 ──────────             ──────               ───────                ───────
 operands, then         where does it        who gives it back,     found before the run,
 the verb               end?                 and when?              during it, or not at all
     │                      │                    │                      │
 sentences.md          segmenting.md         ownership.md           checked.md
```

The pages can be read in a free order. The last two go together: ownership is what lets the
compiler find a mistake before the run, and the checks page tells what is left for the
VM.

For the rules themselves, see [the language](../language/README.md); for the VM
that runs a program, see [the VM](../vm/README.md).
