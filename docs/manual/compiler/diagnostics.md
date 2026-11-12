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
$ zakac run --mem-check error freed.zaka
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

## Levels

| Level | Shown as | Effect |
| --- | --- | --- |
| error | `error[ZKnnnn]` | the compiler goes on to the end of its stage, then stops; the program does not run |
| warning | `warning[ZKnnnn]` | the program runs; the message points at something to look at |
| note | `note[ZKnnnn]` | a hint that is not about a fault; no code uses it now |

A code has one level of its own: ZK0101 is an error at all times and ZK0502 a warning at all times.
Two settings move the level:

- `--strict-particles` makes each ZK05 warning an error.
- `--mem-check` sets the level of the pointer lint of unsafe code (ZK0702 to ZK0712 and
  ZK0721): `warn`, the default, makes each finding a warning, `error` an error, and `off`
  drops them. `--allow`, `--warn` and `--error` set one group or code of it on its own,
  as `--error leaks`. See [Compiler checks](checks.md).

## Codes

A code is `ZK` and four digits. The first two digits name the stage that found the
problem, as the table tells; the last two number the problems of that stage.

| Range | Stage | Examples |
| --- | --- | --- |
| ZK01 | lexer | a character Zaka does not use; a malformed number |
| ZK02 | parser | a missing `။`; a particle in the wrong place |
| ZK03 | names | a name never declared; a name declared twice |
| ZK04 | kinds | a value of the wrong kind; pointer work outside unsafe code |
| ZK05 | naturalness | `ကြီးသည်လျှင်` for `ကြီးလျှင်` (warnings) |
| ZK06 | constant folding | a constant sum that does not fit; a divisor of zero |
| ZK07 | the checker | a read through the null pointer; a value read before it is set; a block used after it was given back |

A code keeps its meaning once it is given out. The numbers within a range are not all
used (there is no ZK0224 or ZK0418, for one). [Stages](stages.md) tells which stage owns
which range, and the page of error messages gives each code a row.

Each code also has a short *title* and a longer *explanation* in the message catalogue,
for tools that want to show more than the message.

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
`help` (or `null`), and the `labels`:

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
$ zakac run --mem-check warn nul2.zaka
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
