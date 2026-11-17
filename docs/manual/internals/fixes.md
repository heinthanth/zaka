---
title: Fixes
description: How a fix gets into a message, how the compiler checks it before showing it, and where the code lives.
---

# Fixes

A *fix* is a change to the source that a message proposes: one or more edits, each the
text that takes the place of a region. The compiler never applies one. It shows each fix
under its message, and the editor offers each as a quick fix. The reader's view is in
[Diagnostics](../compiler/diagnostics.md#fixes).

## Where fixes come from

- **A stage**, when it builds the message and knows the change: the classifier of a kind
  (ZK0509, in `zakac_diagnostics`), and the word the parser expected (ZK0201, attached in
  the parser after the end of the last token read, joined to it first and then after a
  space). These use `Diagnostic::with_fix` and `Diagnostic::help_as_fix`; the second
  turns the help text into the message of the fix.
- **The rules of `zakac_diagnostics_fixes`**, which read the source text: the particle
  warnings (`particles.rs`, ZK0501 to ZK0508), which find the clause-final word at the
  end of the span, and the closest names of the file for an undeclared name (`names.rs`,
  ZK0301).

## The check

`zakac_diagnostics_fixes::attach_fixes` takes the root file, the messages and a function
that compiles a patched text. For each message it gathers the candidates (the fixes the
message holds, then the rules), at most five, patches the text with each, and compiles it
again. A candidate is kept when the patched program has fewer messages of that code and
no more errors on the line of the fix. Errors on other lines can show up, since a fixed
parse lets the later stages run. Of two fixes with one message, the first that holds is
kept.

The function that compiles comes from the caller, so the crate needs no stage of the
compiler. `zakac_borrow_checker::fixes::Recompile` holds the options of the first
compile (the target, the check options, the foldings, the options of the checker and the
loader of imported files) and runs the stages up to the checker. `checked_fixes` builds
one from a command line for `build`, `run` and `borrowck`; the language server builds one
with its open buffers.

A message that keeps a fix drops its help text, which the fix stands for. When no
candidate holds, a help text that became a fix comes back.

## Not done

- A run makes at most 20 recompiles, shared among all its messages.
- Fixes are checked in the root file alone; a fix in an imported file is left out.
- The closest names come from the words of the root file, not from the names in scope.
- The stage commands (`lex`, `parse`, `semcheck`, `irgen`) show the fixes of their stage
  unchecked.
- A recompile runs the comptime functions of the program again.
