---
title: Fixes
description: How a fix gets into a message, how the compiler checks it before showing it, and where the code lives.
---

# Fixes

A *fix* is a change to the source that a message proposes (a quick fix): one or more
edits, each with the text that goes in place of a region. The compiler never applies a
fix. It shows each fix under its message, and the editor offers each one as a quick fix.
[Diagnostics](../compiler/diagnostics.md#fixes) shows what this looks like from the
command line.

The work takes in the diagnostics of a compile, the root file and a function that
compiles a patched text, and it returns the same diagnostics with each fix that passed
the check attached. The code is in `compiler/zakac_diagnostics_fixes`.

## An example

A misspelled name gives ZK0301, and the rules of `names.rs` propose the closest name of
the file:

```text
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ၁၈ ဖြစ်ပါစေ။
အသတ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0301]: "အသတ်" is not declared
 --> fix.zaka:3:1
  |
3 | အသတ် ကို ပြသမည်။
  | ^^^
  = help: replace this name with "အသက်"
  |
3 | အသက် ကို ပြသမည်။
  | ~~~
```

Before the message shows the fix, the compiler patched line 3 with `အသက်`, compiled
the text again and saw that ZK0301 was gone and no new error came up on that line.

## Where fixes come from

A fix comes from one of two places:

- **A stage**, when it builds the message and knows the change. The measure word
  (classifier) of a data kind (type), ZK0509, is built in `zakac_diagnostics`, and the
  word the parser expected, ZK0201, is attached in the parser after the end of the last
  token it read: first joined to that token, then after a space. A stage uses
  `Diagnostic::with_fix` to attach a fix, or `Diagnostic::help_as_fix` to turn the help
  text into the message of the fix.
- **The rules of `zakac_diagnostics_fixes`**, which read the source text. `particles.rs`
  handles the particle warnings (ZK0501 to ZK0508, and ZK0512) and finds the clause-final word at
  the end of the span, and `names.rs` proposes the closest names of the file for an
  undeclared name (ZK0301).

## The check

`zakac_diagnostics_fixes::attach_fixes` takes the root file, the messages and the function
that compiles. For each message it gathers the candidates: first the fixes the message
has, then those of the rules, at most five (`MAX_CANDIDATES`). It patches the text with
each candidate and compiles it again. A candidate passes when the patched program has
fewer messages of that code and no more errors on the line of the fix. Errors on other
lines are allowed, because a parse that the fix repairs lets the later stages run and
report more. When two fixes of one message pass, the first one wins.

The caller supplies the function that compiles, so the crate depends on no stage of the
compiler. `zakac_borrow_checker::fixes::Recompile` keeps the options of the first compile
(the root file, the loader of imported files, the target, the check options, the
foldings and the options of the flow checker) and runs the stages up to the flow checker.
`checked_fixes` builds one from a command line for `build`, `run` and `borrowck`, and the
language server builds one with its open buffers.

A message that keeps a fix drops its help text, because the fix now gives the same advice.
When no candidate passes, a help text that `help_as_fix` turned into a fix comes back.

## Limits

- A run makes at most 20 recompiles (`MAX_RECOMPILES`), shared among all its messages.
- Fixes are checked in the root file alone, so a fix in an imported file is left out.
- The closest names come from the words of the root file, not from the names in scope.
- The stage commands (`lex`, `parse`, `semcheck`, `irgen`) show the fixes of their stage
  without the check.
- A recompile runs the comptime functions of the program again.
