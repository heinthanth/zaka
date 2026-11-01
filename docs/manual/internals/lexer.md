---
title: The lexer
description: How the lexer turns normalized text into tokens, with its word builder, reserved-word splitting, particles, numbers, literals, spans and error handling.
---

# The lexer

The lexer reads the normalized text of one source file and makes a flat list of tokens,
with three side tables: comments, decoded string literals and diagnostics. It lives in
`compiler/zakac_lexer`. Two crates sit below it: `zakac_span` for positions, and
`zakac_burmese_segmenter` for the first cut of the text (see
[Finding words in Burmese](../concepts/segmenting.md)).

The main function is `zakac_lexer::lexer::lex(&SourceFile, &LexerOptions) -> LexOutput`.
Its twin `lex_at(text, base, options)` takes a bare `&str` at an absolute position and
is used in the tests. `LexerOptions` holds one setting, the look-ahead limit handed to the
segmenter (16 as a default).

## Before the lexer: text and positions

`zakac_span` owns the text. A `SourceMap` holds each loaded `SourceFile`, and a `Pos` is
an absolute offset into the whole map, counted in octets of UTF-8. A `Span` is a `lo`
(first octet) and a `hi` (one past the last). A Burmese consonant takes three octets, so
`အသက်`, four code points, spans 12 octets.

When a file is loaded, its text is normalized once:

- a leading BOM (U+FEFF) is dropped, and a carriage return becomes a line feed;
- the text is put in Unicode NFC;
- the signs of each vanna are put in one canonical order, which fixes asat (U+103A)
  before dot below (U+1037).

The normalizer records where it changed the length, so a span can be mapped back to the
original file for messages. Each table of reserved words is written in normalized
spelling; a test pushes each row through the normalizer and checks it comes back
unchanged.

## The four steps

```text
SourceFile (normalized text)
   │  segment_at          one pre-token per vanna, digit or other char
   ▼
Vec<PreLexicalToken>
   │  build_words         group pre-tokens into words
   ▼
Vec<Word>
   │  Driver::step        sort each word into tokens, one rule at a time
   ▼
Vec<Token>
   │  join_phrases        merge multi-word reserved phrases
   ▼
LexOutput { tokens (the last is Eof), comments, strings, diagnostics }
```

The sentence `အသက် ကို ပြသပါ။` ("print age") shows each step. The segmenter cuts it into
vannas (Burmese clusters of a consonant and its signs), spaces and the line feed; `zakac segment` prints the
list:

| Pre-token | `အ` | `သက်` | space | `ကို` | space | `ပြ` | `သ` | `ပါ` | `။` | line feed |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Octets | 0..3 | 3..12 | 12..13 | 13..22 | 22..23 | 23..29 | 29..32 | 32..38 | 38..41 | 41..42 |

The word builder joins pre-tokens that touch and share a script, drops the white space,
and keeps `။` alone, which gives four words: `အသက်`, `ကို`, `ပြသပါ`, `။`. The classifier
then looks each word up. `zakac lex` prints the result (the header lines with the file
name and the source text are left out here):

```text
tokens:
- kind: ident
  span:
    lo: 0
    hi: 12
  particle: none
  joined: false
- kind:
    kw: ko
  span:
    lo: 13
    hi: 22
  particle: none
  joined: false
- kind:
    kw: print
  span:
    lo: 23
    hi: 38
  particle: none
  joined: false
- kind:
    punct: full_stop
  span:
    lo: 38
    hi: 41
  particle: none
  joined: true
- kind: eof
  span:
    lo: 42
    hi: 42
  particle: none
  joined: false
comments: []
strings: []
diagnostics: []
```

`အသက်` is no reserved word, so it is an identifier. `ကို` and `ပြသပါ` are in the reserved
table. `။` is punctuation, and `joined: true` records that no space came before it. The
last token is `Eof`, with a blank span at the end of the file.

## Words

A `Word` is a longest run of pre-tokens with no white space inside, no self-delimiting
char inside, and one script throughout.

- **White space** is the seven chars in `ident::WHITESPACE`: tab, line feed, vertical
  tab, form feed, carriage return, space and the zero-width space U+200B (the Burmese
  input method of macOS sends it on Shift+Space). `char::is_whitespace` is not used,
  because it gets both U+200B and U+00A0 wrong for Zaka.
- **Self-delimiting chars** are each a word alone: `၊ ။ ၏` and each ASCII punctuation
  char except `_`. That includes the backtick (raw identifiers) and `.` (fractions), and
  also the chars the language does not use, such as `;` and `?`, so a loose one is
  reported on its own and does not spoil the word beside it.
- **Script**: Burmese vannas and Burmese digits make one class, all the rest another.
  `abcက` is two words, both marked `joined`.

Each word keeps its slice of pre-tokens (`word.pre`) because the splitter below needs the
vanna borders.

## Sorting a word

`Driver::step` tries these rules in order and takes the first that applies. A rule
returns `Some(next_word_index)` when the word is its own, or `None`. In the source, `step`
is a column of `ret_if!` lines, one per rule.

| # | Rule | Result |
| --- | --- | --- |
| 1 | `//`, `///` (joined slashes) or the word `မှတ်ချက်` | a comment to the end of the line; no token |
| 2 | an opening `"`, `'` or backtick | a string, char or raw identifier token |
| 3 | a delimiter char | punctuation; `<= >= == != << >> && \|\|` join when the second char is attached |
| 4 | a run of words in two scripts that is one name, such as `ဖြစ်ရပ်_၁` or `abcက` | one identifier |
| 5 | an exact reserved word, or a word that splits into reserved words | one or more `Kw` tokens |
| 6 | the first char is a digit | a number |
| 7 | a name of valid shape | an identifier |
| 8 | none of these | ZK0101, and the word is skipped |

The order matters. A comment marker is tested first so that a `//` is not read as two
slashes. A reserved word is tested before the identifier rule, so `ကို` is never a name.
A run that starts with a digit is a number and never a name, so rule 4 steps aside for
rule 6 on `၁၂၃က`.

### What can be a name

`ident::is_ident` decides. The first char is Unicode `XID_Start`, `_`, a non-ASCII emoji
or a Burmese char that is not a digit; the rest are `XID_Continue`, `_`, emoji (with ZWJ,
variation selectors and skin tones) or Burmese chars. The whole Burmese block counts,
since common signs such as `၌ ၍ ၎` are not `XID_Continue`.

### Reserved words and splitting

`reserved::RESERVED` lists each reserved spelling once, mapped to a `Kw` variant; a word
with two spellings has two rows (the four closers of a block all map to `Kw::Done`).
`lookup` is a hash map over the table.

Burmese is written without spaces, so a word made just of reserved words is lexed as
those words: `တန်ဖိုးကို` is `တန်ဖိုး` then `ကို` (`value` then `ko` in the dump). The
function `split_reserved` does this with a small table over vanna borders:

```text
best[0] = the blank split
for end in 1..=n:
    best[end] = the shortest of  best[start] + piece(start..end)
                for each start < end where pre[start..end] is a reserved word
answer = best[n]; refused with fewer than two pieces, or with particles alone
```

Three consequences follow:

- Borders are vannas, never code points, so a piece cannot cut a vanna in two.
- The split with the fewest pieces wins. `အောက်ပါတို့ကို` is two pieces, `Following`
  and `ListKo`, not three.
- A split that is made of particles alone is refused, so `ကိုကို` remains a name. And a
  word with one non-reserved vanna cannot split at all: `အသက်ကို` is one identifier. This
  is the rule behind the need for a space between a name and a particle, and the parser
  adds a hint for it (see [The parser](parser.md)).

A few reserved words are whole rows on purpose, so that none is ever split: `တန်ဖိုးအငှား`
(`Lend`), `လိပ်စာအကြမ်း` (`RawAddress`), `ငှားထားသော` (`Borrowed`). The word `မှတ်ချက်` is
both the comment marker (rule 1 sees it first) and `Kw::Note`, so `မှတ်ချက်ကို` still
splits.

### Token kinds

| `TokenKind` | Meaning |
| --- | --- |
| `Ident` | a name; the text is read through the span |
| `RawIdent` | a name in backticks; the span covers the backticks |
| `Number(NumberShape)` | a number; the shape records script, fraction, exponent, letters |
| `Char(char)` | a char literal, decoded |
| `Str(u32)` | a string literal; an index into `LexOutput::strings` |
| `Kw(Kw)` | a reserved word, or a joined phrase |
| `Punct(Punct)` | `၊ ။ ၏ ( ) + - * / % & \| ^ ~ ! < > <= >= == != << >> && \|\|` |
| `Eof` | the end of the file |

A `Token` is `{ kind, span, particle, joined }` and is `Copy`. It has no text field; the
parser reads a lexeme through the span when it needs one.

`Kw` is one enum for all reserved words. Its groups: particles (`Thi`, `Ko`, `Hnint`,
`Hpjint`, `Htet`, `Auk`, `Hu`, and more; `Kw::is_particle`), verbs and verb-nouns
(`Sum`, `Equal`, `Not`, `And`, ...), statement words (`Print`, `Assign`, `If`, `While`),
kind names (`IntKind`), pieces that exist so the phrase joiner can see them (`*Head`,
`*Tail`), and joined phrases (`Divide`, `BitAnd`, `Shift`, `Convert`, `CallResult`,
`BlockOpen`, `FnOpen`, `BlockClose`, ...) that the joiner alone makes.

## Particles

A comparison verb, `မဟုတ်` and the two boolean words can end a sentence with `သည်`, `ပါ` or
`ပါသည်`. The lexer does not emit that particle as a token. It folds it into the token
before, in the `particle` field. `Particle::with` is the whole rule:

| Held | Next word | Result |
| --- | --- | --- |
| `None` | `သည်` | `Thi` |
| `None` | `ပါ` | `Pa` |
| `Pa` | `သည်` | `PaThi` |
| other pairs | | no combination; the word is a token of its own |

`Driver::absorb` also needs the previous token to be a `Kw` that `absorbs_particle()` (the
comparison verbs, `Not`, `True`, `False`) and the particle to be written attached. For
`က သည် ခ နှင့် ညီပါသည်` the dump of the last token is:

```text
- kind:
    kw: equal
  span:
    lo: 34
    hi: 55
  particle: pa_thi
  joined: false
```

The span grew to cover `ညီပါသည်`, and no token was pushed for `ပါ` or `သည်`. With a space,
`ညီ သည်` keeps two tokens, because there `သည်` marks a new subject; the parser relies on
that to reject a chained comparison. Verb-nouns such as `ပေါင်းလဒ်` never absorb either,
since in `ပေါင်းလဒ်သည်` the `သည်` marks the sum as the subject. The wording rules that read
the stored particle are in [Names and kinds](semcheck.md).

## Phrases

Some reserved phrases span several words, with or without spaces between them. After the
classifier is done, `join_phrases` walks the token list and replaces each run that spells
a phrase with one `Kw` token. `PHRASES` lists them, the longest first, each with a
comment showing the sentence shape:

```text
အောက်ပါတို့ကို လုပ်ဆောင်ပါ။                          → BlockOpen
လုပ်ဆောင်စရာများ ပြီး(ဆုံး)?ပါပြီ(း)?။                  → BlockClose
... ကို ... ဖြင့် စားသော စားလဒ် | စားကြွင်း              → Divide | Remainder
... ကို ... အဖြစ် ပြောင်းလဲသော ရလဒ်                    → Convert
လုပ်ဆောင်ချက် ... ကို ... တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်    → CallResult
```

The joined token spans from the first piece to the last, and takes `joined` from the
first and `particle` from the last. Matching ignores white space and line breaks, so
`အောက်ပါတို့ကို လုပ်ဆောင်ပါ` is one `block_open` token of 76 octets, and the same words
split over two lines are the same token.

Where the language writes two forms (`ရလဒ်အဖြစ်` attached, `ရလဒ် အဖြစ်` apart) the
reserved table has the pieces and the classifier splits the attached form, so one phrase
row serves both.

## Numbers and scripts

Zaka reads digits in two scripts, Burmese `၀၁၂၃၄၅၆၇၈၉` and Western `0123456789`. The
lexer checks the shape of a number and leaves its value to the parser, which knows the
base from an `အခြေ ... ဖြင့်` prefix. `lex_number` works on words, because a literal can
be several: `၁၂.၃၄` reaches it as the words `၁၂`, `.`, `၃၄`.

1. The first word must be digits of one script (Western words can also hold letters, for
   bases above 10). Otherwise: ZK0102, and the word is skipped.
2. A joined `.` and then a joined digit word make a fraction.
3. A text that ends in `e` or `E`, then a joined sign and digits, make an exponent.
4. A joined digit word of the other script is taken into the same token and reported once
   as ZK0103, so a bad literal gives one token and one message.

The shape, `NumberShape { script, has_fraction, has_exponent, has_letters }`, shows in the
dump of `၁၂.၅ 3e5`: the first is `script: burmese, has_fraction: true`, the second
`script: western, has_exponent: true`. A number glued to letters of another script, such
as `၁၈ဘစ်`, is ZK0102 for the whole run. One case is left to the parser: `1e5` is a
valid base-16 literal as well as an exponent, so for a base above 10 the parser reads the
span again as digits.

## Strings, chars and raw identifiers

A quoted literal can hold white space, which the word builder dropped, so the scanners in
`text.rs` read the source text from the opening quote. Each returns `Scan { value, end,
diags }`; the driver then skips each word that starts before `end`. A string goes to
`LexOutput::strings` (the token holds the index) as a list of parts, plus the joined
`value`, where a template slot stands as `{text}` and a bad escape as written:

| Part | Holds |
| --- | --- |
| `Text(String)` | literal text, with `{{` and `}}` folded |
| `Escape(char)` | a decoded escape |
| `TemplateSlot { text, span }` | the text between the braces of a template slot, and its region in the file |
| `Bad(String)` | an escape that does not decode, as written; its ZK0106 is in `diags` |

| Written | Decoded |
| --- | --- |
| `\n \r \t \0 \\ \' \"` | the usual chars |
| `\u` and 4 or 8 hex digits | that code point |
| `{{` and `}}` | a literal `{` and `}` |
| `{နောက်တစ်ကြောင်း}` | a line feed (`NAMED_ESCAPES`) |
| `{ယူနီကုဒ် HHHH}` | that code point |
| other `{...}` in a string | a `TemplateSlot`; the lexer reports nothing, and the parser reads it |
| other `{...}` in a char, a lone `{` or `}` | ZK0106; the text is kept as `Bad` |

A template slot ends at the `}` that matches its `{` on the same line (`open_brace`, through
`template_slot_end`): `quoted_end` skips a string or character literal inside the template slot whole,
with its escapes, `{{`, `}}` and its own template slots, and a nested `{` is matched in turn. So `"a {"b {x}"} c"` is one literal whose template slot holds the literal `"b {x}"`; the
parser lexes that template slot again and reads the nested literal with its own template slot.

A char literal must decode to one scalar value alone (ZK0107), so a Burmese vanna
with signs does not fit. A raw identifier is the text up to the next backtick on the same
line, so a name can hold spaces (`` `ကိန်း ရှင်` `` lexes as one `raw_ident` token); two
backticks in a row are ZK0109.

## Comments

`//` and `///` run to the end of the line, and so does the word `မှတ်ချက်`. A comment never
becomes a token; it goes to `LexOutput::comments` with a kind (`slash`, `doc_slash`,
`note`) and the span from the marker to the line end.

## Spans and `joined`

Spans in `LexOutput` are absolute positions in the `SourceMap`. The artifact the CLI
writes stores them relative to the start of the file, with the normalized source text
embedded, so a dump can be read again without the original file. The `joined` flag tells
a later stage whether white space came before a token; the number rules use it, and so
does the particle rule above.

## Errors and going on

Each lexer message is recoverable. The offending word is skipped, or a token is made from
what was decoded, and lexing goes on, so one bad line does not hide the next.

| Code | Raised when |
| --- | --- |
| ZK0101 | a word is no token kind: a lone `=` or `,`, a loose `;` or `?` |
| ZK0102 | a word starts with a digit and is not a number |
| ZK0103 | Burmese and Western digits in one number |
| ZK0104 | a string is not closed before the line end |
| ZK0105 | a char literal is not closed before the line end |
| ZK0106 | an unknown escape or brace form |
| ZK0107 | a char literal that holds zero or several chars |
| ZK0108 | a raw identifier is not closed |
| ZK0109 | a raw identifier is blank |

For `ကိန်းရှင် က သည် ကိန်းပြည့် ၁၈ဘစ် ဖြစ်ပါစေ။`, `zakac lex` prints the message and still
writes the tokens of the line:

```text
error[ZK0102]: invalid number literal "၁၈ဘစ်"
 --> bad.zaka:1:28
  |
1 | ကိန်းရှင် က သည် ကိန်းပြည့် ၁၈ဘစ် ဖြစ်ပါစေ။
  |                   ^^^^
```

The parser sees a token list without the bad word and goes on from there. The exit code of
the command is a failure when an error was reported, but the artifact is written all the
same, so later stages can be tried on a partial file. The texts of the codes live in
`zakac_diagnostics` (see [Diagnostics](../compiler/diagnostics.md)); the lexer just calls
the constructors in `zakac_diagnostics::lexer`.

## Extending the lexer

- **A reserved word**: add a `Kw` variant in `token.rs`, a row in `RESERVED` (and in
  `Kw::is_particle` for a particle). The tests in `tests/reserved.rs` catch a spelling in
  the wrong sign order and a word that another word can swallow, through the round trips
  over all pairs and a sample of triples of reserved words.
- **A phrase**: give each piece a `Kw`, add the joined variant, and add a row to `PHRASES`
  (a longer row goes before a shorter one that shares its start). Test both spellings,
  attached and apart, in `tests/phrases.rs`.
- **A message**: a row in `define_codes!`, three texts in `locales/codes.toml`, a
  constructor in `zakac_diagnostics/src/lexer.rs`.
- **A named escape**: append to `NAMED_ESCAPES` in `text.rs`.
- The golden files: `tests/inputs/*.zaka` against `tests/expected/*.zktokt`, rewritten
  with `ZAKA_BLESS=1`.

The next stage is [the parser](parser.md).
