---
title: Diagnostics
description: How a compiler message is laid out, what its levels and codes mean, and when a message stops the compiler.
---

# Diagnostics

A *diagnostic* is what the compiler tells about a place in the program: an error, a
warning, or a hint. Each one has a code, a message, a place in the source, and often a
suggestion. This page explains how one is laid out and how the codes are shared out among
the stages. The list of codes is on the page of [error messages](../reference/errors.md).

## How a message is set out

```console
$ zakac run --lint-error mem freed.zaka
error[ZK0704]: the block of "ပ" is freed here, but "တွဲ" still points into it and is used later
 --> freed.zaka:6:1
  |
6 | လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
7 | တွဲ ၏ တန်ဖိုး ကို ပြသမည်။
  | - "တွဲ" is used here
```

| Part | Means |
| --- | --- |
| `error` | the *level* |
| `[ZK0704]` | the *code*, the same through each release |
| the text after `:` | the *message*: what is wrong, with the names from the program |
| `--> freed.zaka:6:1` | the file, then the line and the column of the first marked character |
| the line with its number | the source line, taken from the program |
| `^^^^` | the *main span*: the part of the line that the message is about |
| `----` and its text | a *label*: a second place that helps to see the problem |
| `= help: ...` | a *suggestion* that fixes the problem, when there is one |
| a line with `~~~` | a *fix*: the source line as the suggestion leaves it |

The rows of `^^^^` and `----` above are what a pipe or a file gets. On a color terminal
the main span and each label span are marked inside the source line, in the color of the
level. Terminals do not agree on the cells a Burmese sign takes, so a row can drift from
the text it points at: the rows are drawn when the compiler knows how the terminal draws
Burmese text, and left out otherwise. Without the rows, each span is also underlined, and
a label shows its text alone on the line under its source line. `ZAKAC_CARETS`, or `carets` in a [config file](configuration.md), overrides the choice (see
[Environment](commands.md#environment)). The explorer draws the rows in each font: a row
repeats the text above it, hidden, so the browser sets the marks out under the span.

Columns count characters, not octets. A message with no place in the source (rare)
shows `<PLACEHOLDER>` in place of the file. When two or more messages are shown, a blank
line comes between them.

A message with a suggestion looks like this:

```console
$ zakac run bad2.zaka
error[ZK0401]: expected isize, found f64
 --> bad2.zaka:2:10
  |
2 | အသက် ကို ၂.၅ ဟု ထားမည်။
  |       ^^^
  = help: convert the value with "ကို <kind> တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်"
```

The names of kinds in messages are the short ones: `isize`, `f64`, `u8*`. The table is
in [Error messages](../reference/errors.md#short-names-for-kinds).

## Fixes

When the compiler knows the change that fixes a problem, the `= help:` line names it and
the source line follows as the change leaves it, with `~~~` under the new text (green on a
color terminal):

```console
$ zakac run count.zaka
warning[ZK0509]: လူ is counted with "ယောက်", not "ခု"
 --> count.zaka:4:27
  |
4 | ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်ပါစေ။
  |                   ^
  = help: write "တစ်ယောက်"
  |
4 | ကိန်းရှင် ကျော် သည် လူ တစ်ယောက် ဖြစ်ပါစေ။
  |                   ~~~~
```

The compiler never applies a fix: the change is the reader's to make, with a hand edit or
through the quick fix of an [editor](editors.md). A message can hold more than one fix,
the likeliest first.

`zakac build`, `zakac run`, `zakac borrowck` and the editor check each fix before showing
it: the file is compiled again with the fix made, and the fix is kept when the message is
gone and no new error stands on its line. A fix that fails the check is left out, and its
text comes back as a plain `= help:` line. The stage commands (`lex`, `parse`,
`semcheck`, `irgen`) show the fixes their stage knows, with no such check.

| Message | Fix |
| --- | --- |
| a wording warning, ZK0501 to ZK0508 | the idiomatic particle, or the parentheses removed |
| a classifier, ZK0509 | the classifier of the kind |
| a missing word, ZK0201 | the word the parser expected, when it names one word |
| an undeclared name, ZK0301 | the closest names of the file; `ဝ` and `၀`, `ဥ` and `ဉ` count as one letter |

## Levels

| Level | Shown as | Effect |
| --- | --- | --- |
| error | `error[ZKnnnn]` | the compiler goes on to the end of its stage, then stops; the program does not run |
| warning | `warning[ZKnnnn]` | the program runs; the message points at something to look at |
| note | `note[ZKnnnn]` | a hint that is not about a fault; no code uses it now |

A code has one level of its own: ZK0101 is an error at all times, and ZK0502 a warning
unless a setting moves it. These settings move the level:

- `--lint-allow`, `--lint-warn` and `--lint-error` set one lint group or code: `--lint-allow spoken` lets a
  program use the [spoken spellings](../language/spoken.md) (ZK0510, an error until
  allowed), `--lint-error naturalness` makes each [natural wording](../language/naturalness.md)
  warning an error, and `--lint-allow ZK0505` leaves one rule out. A config file sets them for a
  project, in its `[lint]` section: `allow = ["spoken"]`.
- The group `mem` is the pointer lint of unsafe code (ZK0702 to ZK0712 and ZK0721):
  warnings when no name sets it, errors under `--lint-error mem`, and none under
  `--lint-allow mem`. A group or code of it can take its own level, as
  `--lint-error leaks`. See [Compiler checks](checks.md).

## Codes

A code is `ZK` and four digits. The first two digits name the stage that found the
problem, as the table tells; the last two number the problems of that stage.

| Range | Stage | Examples |
| --- | --- | --- |
| ZK01 | lexer | a character Zaka does not use; a malformed number |
| ZK02 | parser | a missing `။`; a particle in the wrong place |
| ZK03 | names | a name never declared; a name declared twice |
| ZK04 | kinds | a value of the wrong kind; pointer work outside unsafe code |
| ZK05 | naturalness | `ကြီးသည်လျှင်` for `ကြီးလျှင်` (warnings); a spoken spelling such as `ထားမယ်` (ZK0510, an error until allowed) |
| ZK06 | constant folding | a constant sum that does not fit; a divisor of zero |
| ZK07 | the checker | a read through the null pointer; a value read before it is set; a block used after it was given back |

A code keeps its meaning once it is given out. The numbers within a range are not all
used (there is no ZK0224 or ZK0418, for one). [Stages](stages.md) tells which stage owns
which range, and the page of error messages gives each code a row.

Each code also has a short *title* and a longer *explanation* in the message catalogue,
for tools that want to show more than the message. Learner mode prints the explanation,
and the editor links each code to its page.

## Learner mode

Learner mode adds two lines to each message: `= explain:` with the explanation of its
code, and `= see:` with a link to the page of Learn or Docs that teaches it. The
explanation comes once a run, at the first message with its code; the link comes with
each message.

```console
$ ZAKAC_LEARN=1 zakac run program.zaka
error[ZK0301]: "အမည်" is not declared
 --> program.zaka:2:1
  |
2 | အမည် ကို ပြသမည်။
  | ^^^^
  = explain: A name is used before a declaration of it is in scope. ...
  = see: https://zaka-lang.org/learn/variables/scope/
```

Turn it on with `ZAKAC_LEARN=1`, or with `learn = true` in a
[config file](configuration.md#settings-that-variables-hold). The page of each code is
the one its row on the [page of error messages](../reference/errors.md) names; a code
whose row names none gets no `= see:` line. A build of `zakac` that was given no site
address writes the link as a path, such as `/learn/variables/scope/`.

## How a message stops the compiler

Each stage collects its messages in a list, and the chain looks at the list when the
stage ends:

1. A stage goes on after its first message where it can, so that one run shows a number of
   problems at once.
2. When the list holds an *error*, the stages after it do not run (see the table in
   [Stages](stages.md#when-a-stage-is-left-out)). A later stage would meet nothing but the
   fallout of the first mistake.
3. A *warning* is kept in the list and shown, and the chain goes on.

A program with an error in its kinds and one in its constants shows just the first:

```console
$ zakac run two2.zaka
error[ZK0401]: expected isize, found f64
 --> two2.zaka:3:10
  |
3 | အသက် ကို ၂.၅ ဟု ထားမည်။
  |       ^^^
  = help: convert the value with "ကို <kind> တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်"
```

The line `အသက် ကို ၁ / ၀ ဟု ထားမည်။` before it holds a ZK0602, but constants are worked out
after the kinds, so that error shows once the first is fixed. Fix the first error first.

With warnings alone, the program runs, and the messages come before its output:

```console
$ zakac run nat.zaka
warning[ZK0502]: "ကြီးသည်" before "လျှင်" reads better without the particle
 --> nat.zaka:2:7
  |
2 | အကယ်၍ အသက် သည် ၁၈ ထက် ကြီးသည်လျှင် "အရွယ်ရောက်" ဟု ပြသမည်။
  |      ^^^^^^^^^^^^^^^^^^
  = help: write "ကြီးလျှင်", or keep the particle and write "ဖြစ်လျှင်"
အရွယ်ရောက်
```

### Messages kept in artifacts

A stage command shows its messages on standard error and also stores them in its
artifact, as a list. Each holds the `code`, the `level`, the `message`, the `span`, the
`help` (or `null`), the `labels` and the `fixes`; a list that holds nothing is left out:

```text
diagnostics:
- code: ZK0401
  level: error
  message: expected isize, found f64
  span:
    lo: 187
    hi: 194
  help: convert the value with "ကို <kind> တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်"
```

`lo` and `hi` are octet offsets into the source text kept in the same artifact. Running a
saved `codegen` artifact shows its warnings again, in the same form.

### Stopping at run time

A message about the *running* program is another thing. It has no code, because it is
not about the text. The VM writes `the program stopped:` and what it met, and the
exit code is 1:

```console
$ zakac run --lint-warn mem nul2.zaka
warning[ZK0709]: "ပ" points at nothing here, so there is nothing to read or write through it
 --> nul2.zaka:4:1
  |
4 | ပ ၏ တန်ဖိုး ကို ပြသမည်။
  | ^
zaka_vm: the program stopped: access through a null address at 0x0 (in ပရိုဂရမ်, at nul2.zaka:4:1)
```

Here the compiler warned (ZK0709) and the VM stopped the program at the same place.
The stops are listed in [When a program stops](../vm/traps.md).

## The language of messages

The text of each message comes from a catalogue of strings, one for each language, held
in the crates that make the messages. The compiler picks the language when it starts,
from the first of `LC_ALL`, `LC_MESSAGES`, `LANG` and `LANGUAGE` that is set, and from
the locale of the computer if none is. A region or a script it does not have
falls back to the language alone (`en-US` to `en`), and a language it does not have
falls back to English.

English is the one language the catalogue holds now. The code of a message does not
change with the language, so a code found in a forum post or a log means the same thing
for each reader.
