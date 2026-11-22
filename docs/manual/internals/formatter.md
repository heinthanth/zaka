---
title: The formatter
description: How zakac fmt prints a file again, where the depth of a line comes from, how the rewrites work, and how the formatter checks its own output.
---

# The formatter

The formatter sets the shape of a source file: the space between words, the line breaks
and the indent. [Commands](../compiler/commands.md#fmt) lists its flags. The code is in
`compiler/zakac_formatter`, a library with the command `zakac fmt` and the program
`zakafmt` behind its `cli` feature; the language server and the explorer build the
library alone.

## Words, not a tree

The formatter does not print the syntax tree again, as gofmt and rustfmt do. The parser
rewrites what it reads (kind aliases, glued calls, phrases of a call, generics in words),
so a tree printed again would change what the author wrote. The formatter prints the
tokens and comments of the lexer instead, each one as it is written, and sets the space
between them. A token keeps its `joined` mark: no space goes in front of a token that had
none, since the end of a word changes how the lexer reads it (a glued call, `x ၏ f`, a
joined phrase).

`format` in `format.rs` runs these steps on a file:

1. Lex it, then parse it with `parse_in_place` of the parser's loader, which reads the
   files it imports and the core, as a build does, and gives back the statements of the
   file before the compile-time conditionals are settled, so each arm keeps its blocks.
   An error of the file that the lint cannot turn down leaves it as it is.
2. With `--import-order sorted`, move the lines of the imports, then format the moved
   text as a file of its own.
3. Plan the lines of each struct, enumeration and function signature.
4. Print each token and comment again under the rules, with the words the rewrites swap
   in.
5. Check the output.

## The depth of a line

A `Visitor` over the statements collects the span of each block written with its opener
and its closer; a block that stands as one statement adds no depth. A line is one level
deeper for each block whose opener sits on a line above it and whose closer sits on a line
below it, so the closer stands at the depth of its opener.

A line starts a statement when the token before it is `။`, and takes the indent of its
depth. A later line of a statement keeps the spaces its author wrote, or takes the count
`--continuation-indent` gives past the depth. A comment on a line of its own between two
statements takes the depth of the statement after it.

## The lines of a declaration

`breaks.rs` walks the statements for each struct, enumeration, function signature and
block written as one statement, and
notes where each of its lines would start: before each field or member (back over the
`ကနဦးတန်ဖိုး` mark of the initial member), after the last one, and for a signature before
each parameter and before the opener. A signature ends at the stop after its opener, so
the block is not part of it.

For each declaration, `plan` decides the gap before each of its pieces: a join everywhere
when its lines are `fit` and the declaration fits on one line, else a break at the starts
of its lines and a join elsewhere. A line still too wide gets more breaks, in this order:
after the opener of a function that gives a result (`အောက်ပါတို့ကို လုပ်ဆောင်ကာ`), before each
`သို့မဟုတ်`, and last at the last space that fits. The width is counted in characters, as an
editor counts its columns, and a line starts at the depth of its statement. The printer
takes these gaps in place of the line breaks as written; a declaration that holds a
comment keeps its gaps as written. A break never goes before a joined word, so the check
sees the same tokens.

A block written as one statement gets one decision: the gap before its statement. It
breaks when the line as written that ends before the statement, with the first line of the
statement joined to it, is too wide, and joins when it fits; the other lines remain as
written.

## The wrap of other lines

`wrap.rs` takes each line of the output, as the gaps so far make it, that lies outside the
declarations and is wider than the widest line, and breaks it where it reads best: the
end of a clause first, then after a `၊`, then a space after a reserved word or a closing
mark. Within the best kind of place that has one, the last place that keeps the line
within the width wins, and when none does, the first. A space break never leaves the last
word of a statement alone on its line. A line too wide inside a declaration is wrapped the
same way after the breaks of its lines. The wrap only adds breaks; a comment after code
takes no part in the width.

## The rewrites

The rewrites change words, never the space around them. Each one is a *swap*: a span of
the normalized text and the text that goes in its place.

- **Speech forms.** The lexer records each reserved word written in a spoken spelling,
  with its written spelling, and each written word that has a spoken one, with that
  spelling (`LexOutput::spoken_words` and `written_words`). The ends glued to calls, such
  as `မည်` in `ပြသမည်`, are lexed in the parser's pass over glued calls, which hands
  over the same two lists (`GluedEnds`). A spoken spelling that stands for two written
  words, as `ရှိတဲ့` does, becomes the word the parser read in that place: the parser
  records each such reading in `also_taken`.
- **Digits.** A number literal of digits alone, with or without a fraction, takes the
  digits of the other script.
- **Imports.** `sort_imports` in `rewrite.rs` finds the runs of imports at the head of the
  file, each run the imports on lines next to each other with the `///` lines above them,
  and sorts the lines of each run: the stdlib first, then the others, each group in the
  order of its path. A run where two statements share a line keeps its order.

## The check

`same_words` in `check.rs` lexes the output again and compares it with the file, one token
at a time: each token must have the text of the file's token with the swaps written in, the
same particle and the same `joined` mark, and the same kind. A rewrite can change two
kinds: a number in the other script, and a spoken spelling read as the second word it
stands for. The comments must match too. When the two differ, the formatter writes nothing
and reports a fault of its own, as gofmt does.

The parser reads the tokens alone, never the line breaks, so the same tokens make the
same program; the check needs no second parse.

## Where the formatter runs

- `zakac fmt` and `zakafmt` read the `[fmt]` section of the config files, as each command
  reads its own section.
- The language server answers `textDocument/formatting` with one edit for the whole file,
  with the settings of the `[fmt]` section of the `zakac.toml` nearest the file.
- The explorer formats the code of its editor with the default settings.
- `mise run format` formats the samples and the stdlib modules, and the tests check that
  the samples, the stdlib modules and the code of Learn and Docs are formatted.
