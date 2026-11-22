---
title: The lexer
description: How the lexer turns the normalized text of a file into tokens, with the word builder, the reserved words and how a word splits into them, written and spoken spellings, particles, phrases, measure words, numbers, literals, spans and errors.
---

# The lexer

The lexer (tokenizer) is the first stage of the compiler. It reads the normalized text of
one source file and produces a flat list of tokens, plus three side tables: the comments,
the decoded text (string) literals and the diagnostics. The code is in
`compiler/zakac_lexer`, and two crates sit below it: `zakac_span` for positions, and
`zakac_burmese_segmenter` for the first cut of the text, which
[Finding words in Burmese](../concepts/segmenting.md) explains. The rules a program
follows are on [Source text and words](../language/words.md); this page shows how the
code applies them.

The lexer has two functions to call, both in `zakac_lexer::lexer`:

| Function | Use |
| --- | --- |
| `lex(&SourceFile, &LexerOptions) -> LexOutput` | the normal path, for one loaded file |
| `lex_at(text, base, &LexerOptions) -> LexOutput` | the same for a bare `&str` that starts at the absolute position `base`; the tests use it, and so does the parser for the text of a template slot |

`LexerOptions` has one setting, `max_lookahead_limit`, the look-ahead limit that the lexer
passes to the segmenter (16 when not set).

## Before the lexer: text and positions

`zakac_span` owns the text. A `SourceMap` has each loaded `SourceFile`, and a `Pos` is an
absolute offset into the whole map, counted in octets of UTF-8. A `Span` is a `lo` (the
first octet) and a `hi` (one past the last). A Burmese consonant needs three octets, so
`အသက်`, four code points, spans 12 octets.

When a file is loaded, its text is normalized once:

- a leading BOM (U+FEFF) is dropped, and a carriage return becomes a line feed;
- the text is put in Unicode NFC;
- the signs of each vanna are put in one canonical order, which puts asat (U+103A) before
  dot below (U+1037).

The normalizer records where it changed the length, so that a message can map a span
back to the original file. Each table of reserved words is written in normalized
spelling, and a test pushes each row through the normalizer and checks that it comes
back unchanged.

## The steps

```text
SourceFile (normalized text)
   │  segment_at            one pre-token for each vanna, digit or other character
   ▼
Vec<PreLexicalToken>
   │  build_words           group the pre-tokens into words
   ▼
Vec<Word>
   │  Driver::step          sort each word into tokens, one rule after the other
   ▼
Vec<Token>
   │  join_phrases          join the reserved phrases of several words
   │  join_named_phrases    join အစ စကားစု and အဆုံး စကားစု
   │  mark_classifiers      turn the measure words of declared classifiers into tokens
   │  report_mix            warn when a file mixes spoken and written spellings
   ▼
LexOutput { tokens (the last is Eof), comments, strings, diagnostics }
```

The sentence `အသက် ကို ၂၀ ဟု ထားမည်။` ("set age to 20") shows each step. The segmenter cuts
it into vannas (Burmese clusters of a consonant and its signs), digits, spaces and the
line feed, and `zakac segment` prints the list:

| Pre-token | Octets |
| --- | --- |
| `အ` | 0..3 |
| `သက်` | 3..12 |
| space | 12..13 |
| `ကို` | 13..22 |
| space | 22..23 |
| `၂` | 23..26 |
| `၀` | 26..29 |
| space | 29..30 |
| `ဟု` | 30..36 |
| space | 36..37 |
| `ထား` | 37..46 |
| `မည်` | 46..55 |
| `။` | 55..58 |
| line feed | 58..59 |

The word builder joins pre-tokens that touch and share a script, drops the white space,
and keeps `။` alone, which gives six words: `အသက်`, `ကို`, `၂၀`, `ဟု`, `ထားမည်` and `။`.
`Driver::step` then sorts each word. `zakac lex` prints the result (here without the
lines at the top with the file name and the source text):

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
    number:
      script: burmese
      has_fraction: false
      has_exponent: false
      has_letters: false
  span:
    lo: 23
    hi: 29
  particle: none
  joined: false
- kind:
    kw: hu
  span:
    lo: 30
    hi: 36
  particle: none
  joined: false
- kind:
    kw: assign
  span:
    lo: 37
    hi: 55
  particle: none
  joined: false
- kind:
    punct: full_stop
  span:
    lo: 55
    hi: 58
  particle: none
  joined: true
- kind: eof
  span:
    lo: 59
    hi: 59
  particle: none
  joined: false
comments: []
strings: []
diagnostics: []
```

`အသက်` is no reserved word, so it becomes an identifier. `ကို`, `ဟု` and `ထားမည်` are in
the reserved table, and `၂၀` is a number whose shape the token records; its value is left
to the parser. `။` is punctuation, and `joined: true` records that no space came before
it. The last token is `Eof`, with a blank span at the end of the file.

Not each verb is a reserved word. The verbs of the core module, such as `ပြသ` in
`အသက် ကို ပြသမည်။`, are functions, so `ပြသမည်` lexes as one identifier, and the parser
splits the call end `မည်` off it (see [Expressions and calls](parser-expressions.md)).

## Words

A `Word` is the longest run of pre-tokens with no white space inside, no self-delimiting
character inside, and one script from start to end.

- **White space** is the seven characters in `ident::WHITESPACE`: tab, line feed,
  vertical tab, form feed, carriage return, space and the zero-width space U+200B (the
  Burmese input method of macOS sends it on Shift+Space). We do not use
  `char::is_whitespace`, because it gets both U+200B and U+00A0 wrong for Zaka.
- **Self-delimiting characters** are each a word alone: `၊ ။ ၏` and each ASCII
  punctuation character except `_`. That includes the backtick (raw identifiers) and `.`
  (fractions), and also characters the language does not use, such as `;` and `$`, so a
  loose one is reported alone and does not spoil the word beside it.
- **Script**: Burmese vannas and Burmese digits make one class, and all the rest another.
  So `abcက` is two words, both marked `joined`.

Each word keeps its slice of pre-tokens (`word.pre`), because the splitter below needs
the borders of the vannas.

## Sorting a word

`Driver::step` tries these rules in order and keeps the first that applies. Each rule
returns `Some(next)`, the index of the next word to lex, when the word is its own, and
`None` when it is not. In the source, `step` is a column of `ret_if!` lines, one for each
rule.

| # | Rule (function) | Result |
| --- | --- | --- |
| 1 | `//`, `///` (joined slashes) or the word `မှတ်ချက်` (`lex_comment`) | a comment to the end of the line; no token |
| 2 | an opening `"`, `'` or backtick (`lex_quoted`) | a text, character or raw identifier token |
| 3 | a delimiter character (`lex_punct`) | punctuation; `<= >= == != << >> && \|\| ::` join when the second character touches the first |
| 4 | a run of words in two scripts that is one name, such as `ဖြစ်ရပ်_၁` or `abcက` (`lex_mixed_ident`) | one identifier |
| 5 | a reserved word, `၏` in a spelling of its own, or a word that splits into reserved words (`lex_reserved`) | one or more `Kw` tokens |
| 6 | the first character is a digit (`lex_digits`) | a number |
| 7 | a name of valid shape (`lex_ident`) | an identifier |
| 8 | none of these (`invalid`) | ZK0101, and the word is skipped |

The order matters. We test for a comment marker first so that `//` is not lexed as two
slashes. A reserved word comes before the identifier rule, so `ကို` is never a name. A
run that starts with a digit is a number and never a name, so rule 4 steps aside for
rule 6 on `၁၂၃က`.

### What can be a name

`ident::is_ident` decides. The first character is Unicode `XID_Start`, `_`, a non-ASCII
emoji or a Burmese character that is not a digit; the rest are `XID_Continue`, `_`, emoji
(with ZWJ, variation selectors and skin tones) or Burmese characters. The whole Burmese
block counts, since common signs such as `၌ ၍ ၎` are not `XID_Continue`.

## Reserved words and splitting

`reserved::RESERVED` lists each reserved word once, mapped to a `Kw` variant. A word with
two spellings either has two rows (`ခုမြောက်` and `လုံးမြောက်` both map to `Kw::Ordinal`) or
gets its second spelling from the alias table below. `lookup` is a hash map over all the
written spellings, and it reads the word through `spelling::canonical`, which writes each
`ဒေတာ` as `အချက်အလက်`: the table has the `အချက်အလက်` form alone, and
`ဒေတာအမျိုးအစား` finds `Kw::DataKindOpen` through it. When the written spellings do not
know the word, `lookup` tries the spoken spellings, so a spoken word lexes as its written
`Kw`.

The same function makes names one: `Name::new` in `zakac_ast` keeps the canonical text,
so the checker and each later stage see one name for both spellings. The parser matches
struct names against source text that is not canonical (a struct name glued to its
measure word), so its set of struct names has each name in both spellings
(`spelling::spellings`).

The names of the built-in data kinds (types), `reserved::KIND_NAMES` (`ကိန်းပြည့်`,
`ဒဿမကိန်း`, `စာလုံး`, `စာသား`, `မှန်မှားတန်ဖိုး`, `ဘူလီယန်`, `စာရင်း` and `ဇယား`), are
not in the table. The core module declares them, so a whole word of one of them is an
identifier, which the parser then reads as the data kind (see
[Data kinds, generics and imports](parser-kinds.md)), and a file can declare a data kind
of that name.

### Splitting a word

Burmese is written without spaces, so a word made of reserved words alone is lexed as
those words: `တန်ဖိုးကို` is `တန်ဖိုး` then `ကို` (`value` then `ko` in the dump). The
function `split_reserved` does this with a small table over the vanna borders, and it
counts a data kind name as a piece too, which it returns as an `Ident`:
`လုံးမြောက်စာလုံးမှ` is `Ordinal`, the name `စာလုံး`, then `From`.

```text
best[0] = the blank split
for end in 1..=n:
    best[end] = the shortest of  best[start] + piece(start..end)
                for each start < end where pre[start..end] is a reserved word
                                         or a data kind name
                (a piece is never longer than the longest reserved word)
answer = best[n]; refused with fewer than two pieces, or with particles alone
```

Three rules follow from it:

- Borders are vannas, never code points, so a piece cannot cut a vanna in two.
- The split with the fewest pieces wins. `အောက်ပါတို့ကို` is two pieces, `Following` and
  `ListKo`, not three. Among splits of the same count, one that is not all particles
  wins, so `အောက်ပါ` + `သော` beats `အောက်` + `ပါသော`.
- A split made of particles alone is refused, so `ကိုကို` remains a name. And a word
  whose pieces are not all reserved words or data kind names cannot split at all:
  `အသက်ကို` is one identifier. This is the reason a space is needed between a name and a
  particle, and the parser adds a hint for it (see
  [Skipping a bad sentence](parser.md#skipping-a-bad-sentence)).

A name joined to `ထဲမှ`, as in `အရောင်များထဲမှ`, has a part that is not reserved, so
`split_reserved` cannot split it. `lex_ident` does: a name word that ends in `ထဲမှ`, with
a name before it, becomes an `Ident` and `Kw::Among` (`lex_among`). The `ထဲမှ` must start
a vanna, so a stacked consonant before it keeps the word whole. A name in Latin letters,
or in two scripts (`Deliveryများထဲမှ`), is a run of words, and `lex_among` uses the text
of the whole run, so it splits just as a Burmese name does.

## Written and spoken spellings

`aliases.rs` has `ALIASES`, the table of the parts of reserved words that mean the same
thing. Each row is one part, written as plain Zaka writes it (`base("၍")`), with the
other written forms of the part (`.written(&[…])`) and its spoken forms
(`.spoken(&[…])`). A reserved word that contains the part, or one of these forms, reads
the same with each other form:

| Row | Effect |
| --- | --- |
| `base("၍").written(&["ပြီး", "ကာ"]).at_end()` | `ဖြစ်ပြီး` and `ဖြစ်ကာ` lex as `ဖြစ်၍` (`Kw::And`) |
| `base("ပြီးဆုံးပါပြီး").written(&[…])` | the four spellings of the block closer word all map to `Kw::Done` |
| `base("မည်").spoken(&["မယ်"])` | `ထားမယ်` lexes as `ထားမည်`, and the lint reports it (ZK0510) |
| `base("၏").spoken(&["ရဲ့"]).alone()` | `ရဲ့` is the `၏` token |
| `base("မင်မိုရီ").written(&["မှတ်ဉာဏ်"])` | both words for storage lex as one |

A row can limit where it applies: `.at_end()` (the part counts at the end of a word
alone, as the ending of a verb does), `.swaps_within(&[…])` (the written forms swap in
these reserved words alone), `.except(&[…])` (the row does not hold in these words,
where the part is no particle) and `.alone()` (the part is a word of its own, such as
`၏`). A part matches whole vannas, so the `မှ` of `မှတ်` is no part `မှ`. Where parts of
several rows start at one place, the longest part wins.

`aliases::WRITTEN` lists each written spelling: each word of `RESERVED`, then the
spellings that the written rows make from it. A word of `RESERVED` wins over a spelling
that the aliases make. `spoken.rs` builds `SPOKEN` from the spoken rows, applied to each
written spelling. Two written words in `သော` and `သည့်` give one spoken spelling in `တဲ့`,
which lexes as the word in `သော`, and `Spoken::also` names the other word. The parser
accepts it for that word where its grammar asks for it (`Parser::expect_reading`), and
ZK0510 gives a fix for each. `split_reserved` splits with the spoken spellings too, so
`ညီတယ်` is `ညီ` with the particle `သည်` folded in.

After the lexer pushes a reserved word or `၏`, it notes its form (`note_form`): a spoken
spelling is ZK0510, with a fix to the written spelling, and a written word that has a
spoken spelling is counted. A `သည်` that is a token alone marks a topic and is not
counted. At the end, `report_mix` gives a file with both forms ZK0511, at the first word
of the form it uses less. The lint levels decide whether either message shows. The tests
write each spelling the table makes to `tests/expected/aliases.tsv` and
`tests/expected/spoken.tsv` for review, and fail when a spelling lexes as two reserved
words.

## The reserved words of each feature

The table below groups the reserved words that the features of the language add. Each
is a plain row of `RESERVED`, so each splits off a word it is written against.

| Feature | Words (`Kw`) |
| --- | --- |
| failing functions | `ရလဒ်ကောင်း` (`GoodResult`), `မအောင်မြင်ပါက` (`OnFail`, for the pass-on after a call statement), `အမှားအယွင်း` (`Fault`) |
| optionals and result kinds | `ရှိကောင်းရှိနိုင်သော` (`Perhaps`), `ရှိနိုင်သော` (`CanHold`), `အချက်အလက်ဆဲလ်` (`DataCell`), `အလွတ်` (`NoData`), `အချက်အလက်` (`Data`), `ထဲတွင်` (`Within`), `ရှိနေ` (`Located`), `မရှိ` and `မရှိနေ` (`NotLocated`) |
| the sides with no value | `အောင်မြင်သည့်သတင်း` (`SuccessNews`), `မအောင်မြင်သည့်သတင်း` (`FailureNews`) |
| the mark statements | `အောင်မြင်သည်` (`Succeeded`), `မအောင်မြင်ပါ` (`NotSucceeded`), `သတ်မှတ်မည်` (`WillMark`) |
| structs and enumerations | `ဒေတာအစုအဝေး` (`StructWord`), `ဒေတာအမျိုးအစားကွဲ` (`EnumWord`), `ရှိသည့်` (`HavingRel`, the end of a member value), `သုံးမည့်` (`ToUse`, the last piece of the initial mark) |
| switches | `ထဲမှ` (`Among`), `တခြား` (`Other`) |
| members that hold data | `ပြောင်းလဲနိုင်` (`CanBecome`, the verb of the test of a member, which absorbs a particle as a comparison verb does), `အဖြစ်သို့` (`AsInto`) |
| data kinds | `အမျိုးအစား` (`KindWord`, a piece of the function kind phrase), `ဒေတာအမျိုးအစား` (`DataKindOpen`, opens a kind alias), `လက္ခဏာရှိ` (`Signed`), `ဘစ်` (`Bit`), `လိပ်စာ` (`Address`) |
| measure words (classifiers) | `ခုစာ` and `လုံးစာ` (`Items`, the measure word of a heap size), `ကွက်` (`CellWord`), `မျိုးပြပစ္စည်းမှာ` (`ClassifierIs`) |
| function signatures | `ပြောင်းလဲရန်` (`ToChange`) and `ထည့်ပေးရန်` (`ToFill`), the second words of the parameter marks; `ဝိဘတ်` (`PrepositionWord`); `ဟူသော` (`Called`) after a name or a literal, where the next word decides what it starts (a data kind, the function value, the error value, or the words of a struct or an enumeration); `မည့်` (`Named`); `လက်ခံသော` (`AcceptingRel`) and `ပြန်ပေးသော` (`ReturningRel`) |
| statements | the statement verbs end in `မည်`: `လုပ်ဆောင်မည်` is `WillDo`, `ထားမည်` is `Assign`, `ပြန်ပေးမည်` is `WillReturn`, and so on |
| lists and text | `အကွက်` (`Cell`), `နောက်ဆုံးအကွက်` (`LastCell`), `နောက်ဆုံးစာလုံး` (`LastChar`), `အရှေ့တွင်` (`Before`), `ထိ` (`Upto`), `ခုမြောက်` and `လုံးမြောက်` (`Ordinal`) |
| tables (maps) | `ဇယားအလွတ်` (`BlankTable`), `ညွှန်းသော` (`Indexed`), `အညွှန်း` (`IndexWord`), `အညွှန်းတိုင်း` (`EachIndex`), `အတန်းတိုင်း` (`EachRow`) |
| generics and traits | `ယေဘုယျဒေတာအမျိုးအစား` (`GenericKind`), `အစားထိုးသုံးနိုင်၍` (`Substitutable`), `၎င်း` (`Itself`), `အစားထိုးအသုံးပြုသည့်` (`SubstitutedRel`), `အစားထိုးအသုံးပြု၍` (`Substituting`), `ဂုဏ်သတ္တိ` (`Trait`), `ရှိစေရန်` (`ToHave`), `ပါဝင်ရမည်` (`MustContain`), `လုပ်ဆောင်ချက်များ` (`Functions`), `ဤနေရာတွင်` (`Here`), `သတိပြုရန်မှာ` (`NoteThat`), `သတိပြုစရာများ` (`Notes`), `ရှိသည်` (`Has`) |
| written-as hints | `ရေးသားရာတွင်` (`WritingOpen`), `ရေးသားနိုင်သည်` (`CanWrite`), `ရေးသားနိုင်ပြီး` (`CanWriteAnd`) |
| trait defaults | `အခြားလုပ်ဆောင်ချက်များကိုလည်း` (`OtherFnsToo`), `မူရင်းအတိုင်း` (`AsOriginal`), `ဆက်ခံမည်` (`Inherit`) |
| comments | `မှတ်ချက်` (`Note`) |

Some notes on these words:

- `ဒေတာ` reaches the table in its `အချက်အလက်` form, and so does `ဒေတာဆဲလ်`. A lone
  `ဒေတာ` splits joined words as a reserved word, and `free_data_words` in `phrases.rs`
  turns it back into a name unless it follows `ထဲမှ`, `ထဲတွင်` or `အမှားအယွင်း`, or comes
  before `ပြောင်းလဲရန်` or `ထည့်ပေးရန်`. So a program can still name a variable `ဒေတာ`.
- `ဒေတာအစုအဝေး` and `ဒေတာအမျိုးအစားကွဲ` are each one whole word in the table, so the
  longest match wins over `ဒေတာအမျိုးအစား` and `အမျိုးအစား` inside them. A joined `တွင်`
  splits off, as in `ဒေတာအမျိုးအစားကွဲတွင်` (`EnumWord`, `At`).
- `မညွှန်းသော` of the null address is one word in the table: the whole word is looked up
  before a split, so it never splits into `မ` and `ညွှန်းသော`. A joined `ဖြင့်ညွှန်းသော`
  splits into `ဖြင့်` and `ညွှန်းသော`, and a name that ends in a reserved word, such as
  `ဖိုင်အညွှန်း`, remains a name, since `ဖိုင်` is no reserved word.
- The fallback of an optional, `၊ မရှိလျှင်`, needs no new word: `မရှိ` and `လျှင်` are
  in the table for the tests of an optional.
- `ဇယား` itself is a data kind name, not a reserved word.
- A word made of reserved words alone splits into them, except the names of the core in
  `WHOLE_NAMES` (`reserved.rs`): `ပြောင်းလဲနိုင်သော`, the conversion trait, is
  `ပြောင်းလဲနိုင်` and `သော`, and remains one name, glued to a following word as in
  `ပြောင်းလဲနိုင်သောဟု` too.
- `မှတ်ချက်` is both the comment marker (rule 1 sees it first) and `Kw::Note`, so
  `မှတ်ချက်ကို` still splits.
- `မိတ္တူ` is no reserved word: the parser recognizes it after `၏` alone.
- `?` is the sign `Punct::Question`. The parser treats a `?` as a pass-on just when it
  touches the `)` of a call (`joined`).
- `::` is one token, `Punct::PathSep`; a `:` alone remains the field mark of a struct
  value.

Some `Kw` variants are not in `RESERVED` at all: `Kw::Print`, `Kw::Add`, `Kw::Remove`,
`Kw::Replace` and `Kw::CellCount`, among others, name what a function of the core module
means. `role::BUILTIN_VERBS` maps the verbs `ပြသ`, `ထည့်`, `ကူးထည့်`, `ဖျက်` and `အစားထိုး`
to them, and `role::BUILTIN_COUNTS` the counts `အကွက်အရေအတွက်`, `စာလုံးအရေအတွက်` and
`အညွှန်းအရေအတွက်`. These words lex as identifiers.

## Tokens

| `TokenKind` | Meaning |
| --- | --- |
| `Ident` | a name; the text comes from the span |
| `RawIdent` | a name in backticks; the span covers the backticks |
| `Number(NumberShape)` | a number; the shape records the script, a fraction, an exponent and letters |
| `Char(char)` | a character literal, decoded |
| `Str(u32)` | a text literal; an index into `LexOutput::strings` |
| `Kw(Kw)` | a reserved word, or a joined phrase |
| `Punct(Punct)` | `၊ ။ ၏ ( ) [ ] : :: + - * / % & \| ^ ~ ! < > <= >= == != << >> && \|\| ?` |
| `Eof` | the end of the file |

A `Token` is `{ kind, span, particle, joined }` and is `Copy`. It has no text field; the
parser reads a lexeme through the span when it needs one.

`Kw` is one enum for all reserved words. Its groups are:

- particles (`Thi`, `Ko`, `Hnint`, `Hpjint`, `Htet`, `Auk`, `Hu`, `Into`, and more;
  `Kw::is_particle`);
- verbs and verb-nouns (`Sum`, `Equal`, `Not`, `And`, …);
- statement words (`Assign`, `If`, `While`, `WillReturn`, …);
- list words (`Cell`, `LastCell`, `LastChar`, `Before`, `Upto`, `Ordinal`);
- data kind words (`Signed`, `Bit`, `Address`);
- pieces that exist so that the phrase joiner can see them (`*Head`, `*Tail`);
- joined phrases (`Divide`, `BitAnd`, `Shift`, `Convert`, `CallResult`, `BlockOpen`,
  `UnsafeBlockOpen`, `FnValue`, `BlockClose`, …), which the joiner alone makes.

### Roles and colors

`role::token_role` gives each token kind a `TokenRole`: a control word, a verb, an
operator, a particle, a data kind noun, a truth value, a name, a raw name, a number, a
character, a text, a delimiter, a comment or plain text. The site and the explorer color
a token after its role, and the language server sends the role to the editor; a new
reserved word gets its role in `role::kw_role`. `role::role_with_text` reads the text as
well, so an identifier that is a built-in data kind name gets the role of a data kind
noun, a verb of the core module (`BUILTIN_VERBS`) the role of a verb, and a count of the
core module (`BUILTIN_COUNTS`) the role of an operator.

## Particles

A comparison verb, `မဟုတ်` and the two true/false words can end a sentence with `သည်`,
`ပါ` or `ပါသည်`. The lexer does not emit that particle as a token. It folds it into the
token before, in the `particle` field. `Particle::with` is the whole rule:

| The token has | Next word | Result |
| --- | --- | --- |
| `None` | `သည်` | `Thi` |
| `None` | `ပါ` | `Pa` |
| `Pa` | `သည်` | `PaThi` |
| other pairs | | no fold; the word becomes a token alone |

`Driver::absorb` also needs the previous token to be a `Kw` that `absorbs_particle()` (the
comparison verbs, `Not`, `True`, `False`), and the particle must be written attached. For
`က သည် ခ နှင့် ညီပါသည်`, the dump of the last token is:

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
`ညီ သည်` remains two tokens, because there `သည်` marks a new subject, and the parser
depends on that to reject a chained comparison. Verb-nouns such as `ပေါင်းလဒ်` never
absorb either, since in `ပေါင်းလဒ်သည်` the `သည်` marks the sum as the subject. The wording
rules that read the stored particle are in [Names and data kinds](semcheck.md).

## Phrases

Some reserved phrases span several words, with or without spaces between them. After the
words are sorted, `join_phrases` walks the token list and replaces each run that spells a
phrase with one `Kw` token. `PHRASES` lists them, the longest first, each with a comment
that shows the shape of the sentence:

| Phrase | Joined token |
| --- | --- |
| `အောက်ပါတို့ကို လုပ်ဆောင်မည်။` | `BlockOpen` |
| `အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။` | `UnsafeBlockOpen` |
| `အောက်ပါတို့ကို လုပ်ဆောင်ကာ` | `FnOpenRet` |
| `… လုပ်ဆောင်မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။` | `PassFault` (the words after the `၊`) |
| `လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်` | `MarkSucceeded` |
| `လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်` | `MarkFailed` |
| `<name> ဟူသော <kind> တစ်ခု ကို ဒေတာ ပြောင်းလဲရန်` | `DataToChange` (the last two words) |
| `<name> ဟူသော <kind> တစ်ခု ကို ဒေတာ ထည့်ပေးရန်` | `DataToFill` (the last two words) |
| `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` | `Null` |
| `… လည်းကောင်း အခြားတန်ဖိုးများနှင့်အတူ လက်ခံပြီး` | `WithOtherValues` |
| `… ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်` / `ပြန်လည်ရုပ်သိမ်းမည်` | `AllowCall` / `RevokeCall` |
| `… ကို ပြန်လည် မျှဝေမည်` | `Reshare` |
| `လုပ်ဆောင်စရာများ ပြီး(ဆုံး)?ပါပြီ(း)?။` | `BlockClose` |
| `ပရိုဂရမ် ပြီး(ဆုံး)?ပါပြီ(း)?။` | `ProgramClose` |
| `… ကို … ဖြင့် စားသော စားလဒ်` / `စားကြွင်း` | `Divide` / `Remainder` |
| `… နှင့် … ၏ ဘစ်အလိုက် ဆုံလဒ်` / `ပေါင်းစုလဒ်` / `ခြားနားလဒ်`; `… ၏ ဘစ်အလိုက် ပြောင်းပြန်ပြောင်းလဒ်` | `BitAnd` / `BitOr` / `BitXor` / `BitNot` |
| `… ကို ဘယ်သို့\|ညာသို့ … ဘစ် ရွေ့သော ရလဒ်` | `Shift` |
| `… ကို … အဖြစ် ပြောင်းလဲသော ရလဒ်` / `ရလဒ်ကောင်း` | `Convert` / `ConvertGood` |
| `… ဟူသော လုပ်ဆောင်ချက်ကို … တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်` / `ရလဒ်ကောင်း` | `CallResult` / `CallGoodResult` |
| `… ကို သုံး၍ <fn> သည့် ရလဒ်`, `<fn> မည့် ရလဒ်`, `<fn> သော ရလဒ်` | `UsedResult` |
| the same three with `ရလဒ်ကောင်း` | `UsedGoodResult` |
| `ပြင်ပမှလာသော` | `External` |
| `ပြင်ပမှ ခေါ်ဆိုနိုင်သော` | `CallableOutside` |
| `ပြင်ပမှ ယူသုံးနိုင်သော` | `UsableOutside` |
| `ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော` | `Comptime` |
| `<parameters> လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား` | `FnKind` (the last two words) |
| `<name> ဟူသော လုပ်ဆောင်ချက်`, `<name> မည့် လုပ်ဆောင်ချက်` | `FnValue` |
| `<value> ဟူသော အမှားအယွင်း` | `CalledFault` |
| `<x> ထဲတွင် ဒေတာ ရှိ(နေ)` / `မရှိ(နေ)` | `HasData` / `LacksData` |
| `<x> ထဲတွင် ရလဒ်ကောင်း \| အောင်မြင်သည့်သတင်း ရှိ(နေ)` | `HasGood` |
| `<x> ထဲတွင် အမှားအယွင်း \| မအောင်မြင်သည့်သတင်း ရှိ(နေ)` | `HasFault` |
| `<x> ထဲမှ အမှားအယွင်း ဒေတာ` | `FaultData` |
| `T သို့မဟုတ် အမှားအယွင်းအနေဖြင့် E ရှိနိုင်သော ဒေတာဆဲလ်` | `AsFault` (the piece `အမှားအယွင်းအနေဖြင့်`) |
| `… ဟုလည်း မှတ်ယူနိုင်ပါသည်` / `မှတ်ယူနိုင်ပြီး` | `AliasClose` / `AliasCloseAnd` |
| `… တို့ပါဝင်သည်` / `တို့ပါဝင်ပြီး` | `FieldsClose` / `FieldsCloseAnd` |
| `ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့် <member>` | `InitialMark` |
| `တန်ဖိုးအနေဖြင့် <value> ရှိသည့် <member>` | `ValueAs` (the piece `တန်ဖိုးအနေဖြင့်`) |

The joined token spans from the first piece to the last, and it is `joined` when the first
piece is and keeps the `particle` of the last. Matching ignores white space and line
breaks, so `အောက်ပါတို့ကို လုပ်ဆောင်မည်` is one `block_open` token of 76 octets, and the
same words split over two lines make the same token.

The rules around the joiner:

- `FnKind` closes a [function kind](../language/function-values.md), and `FnValue` after a
  name makes the function a value, or with `ကို` after it opens a long call. `မည့်`
  (`Named`) is a second spelling of `ဟူသော` (`Called`) in this phrase and in the ends of
  `UsedResult`.
- `phrase_at` keeps `ဟူသော` as `Called` in two runs. Before `လုပ်ဆောင်ချက် အမျိုးအစား`
  the next two words join into `FnKind` instead, the data kind of a parameter or a
  field, as in `ဖ ဟူသော လုပ်ဆောင်ချက် အမျိုးအစား တစ်ခု`. Before `လုပ်ဆောင်ချက် သည်`
  nothing joins, since that run starts the function signature of a declaration.
- `Null` is the null pointer, "an address that points at nothing": the reserved words
  `မည်သည်ကိုမှ` (`Nothing`) and `မညွှန်းသော` (`PointsAtNo`) and the data kind word
  `လိပ်စာ` (`Address`) join into it, and the first two never appear alone in a program.
  `ကနဦးတန်ဖိုး` (`Initial`), the [initial value](../language/kinds.md#initial-values), is
  one word and needs no joining. The editors paint all three as literal values, like
  `မှန်` and `အလွတ်`.
- A block opener and the opener of a function are one token, `BlockOpen`; the parser
  tells them apart from what comes before it.
- The two parameter marks are phrases: `ဒေတာ` (`Data`) then `ပြောင်းလဲရန်` joins into
  `DataToChange`, and `ဒေတာ` then `ထည့်ပေးရန်` into `DataToFill`. The second word alone
  remains a reserved word, so a lone `ပြောင်းလဲရန်` after `ကို` reaches the parser as
  `ToChange`, which no rule accepts (ZK0201).
- Where the language has two forms of one phrase (`ဟူသော အမှားအယွင်း` apart and
  `ဟူသောအမှားအယွင်း` attached), the reserved table has the pieces and the splitter
  splits the attached form, so one phrase row serves both.
- The tests of an optional or a result are phrases of the two words after `ထဲတွင်`:
  `ထဲတွင်` is a word alone, and a phrase such as `HasData` has the word the test asks
  about and the `ရှိ`, `ရှိနေ`, `မရှိ` or `မရှိနေ` that ends it.

After `join_phrases`, `join_named_phrases` joins the two words of `အစ စကားစု` into
`Kw::StartPhrase` and of `အဆုံး စကားစု` into `Kw::EndPhrase` (`NAMED_PHRASES`). `အစ`
and `အဆုံး` are no reserved words, so these remain names in each other place.

## Declared measure words

A struct, an enumeration or a kind alias can declare its measure word after `မျိုးပြပစ္စည်းမှာ`
(`Kw::ClassifierIs`). The lexer has no list of the measure words, so the measure word
`ယောက်စာ` of a heap size lexes as a name. `mark_classifiers` in `classifiers.rs` fixes
that after the phrases are joined: it collects the name after each `ClassifierIs` (or the
reserved `ကွက်`, `Kw::CellWord`), adds `COUNT_ENDING` (`စာ`), and turns each identifier of
that spelling into `Kw::Items`, with its span kept, so the parser reads the measure word
from the text before `စာ`. Without a declaration in the file, `ယောက်စာ` remains a name.
`ခုစာ` and `လုံးစာ` are reserved and need no declaration.

## Numbers and scripts

Zaka reads digits in two scripts, Burmese `၀၁၂၃၄၅၆၇၈၉` and Western `0123456789`. The
lexer checks the shape of a number and leaves its value to the parser, which knows the
base from an `အခြေ … ဖြင့်` prefix. `lex_number` works on words, because a literal can be
several of them: `၁၂.၃၄` reaches it as the words `၁၂`, `.` and `၃၄`.

1. The first word must be digits of one script (Western words can also contain letters,
   for bases above 10). Otherwise it is ZK0102, and the word is skipped.
2. A joined `.` and then a joined digit word make a fraction.
3. A text that ends in `e` or `E`, then a joined sign and digits, make an exponent.
4. A joined digit word of the other script goes into the same token and is reported once
   as ZK0103, so a bad literal gives one token and one message.

The shape, `NumberShape { script, has_fraction, has_exponent, has_letters }`, shows in the
dump of `၁၂.၅ 3e5`: the first is `script: burmese, has_fraction: true`, the second
`script: western, has_exponent: true`. A number glued to letters of another script, such
as `၁၈ဘစ်`, is ZK0102 for the whole run. One case is left to the parser: `1e5` is a valid
base-16 literal as well as an exponent, so for a base above 10 the parser reads the span
again as digits.

## Text, characters and raw identifiers

A quoted literal can contain white space, which the word builder dropped, so the scanners
in `text.rs` read the source text from the opening quote. Each returns
`Scan { value, end, diags }`, and the driver then skips each word that starts before
`end`. A text literal goes to `LexOutput::strings` (the token has its index) as a list of
parts, plus the joined `value`, where a template slot appears as `{text}` and a bad escape
as written:

| Part | Contains |
| --- | --- |
| `Text(String)` | literal text, with `{{` and `}}` folded |
| `Escape(char)` | a decoded escape |
| `TemplateSlot { text, span }` | the text between the braces of a template slot, and its place in the file |
| `Bad(String)` | an escape that does not decode, as written; its ZK0106 is in `diags` |

| Written | Decoded |
| --- | --- |
| `\n \r \t \0 \\ \' \"` | the usual characters |
| `\u` and 4 or 8 hex digits | that code point |
| `{{` and `}}` | a literal `{` and `}` |
| `{နောက်တစ်ကြောင်း}` | a line feed (`NAMED_ESCAPES`) |
| `{ယူနီကုဒ် HHHH}` | that code point |
| other `{…}` in a text literal | a `TemplateSlot`; the lexer reports nothing, and the parser reads it |
| other `{…}` in a character literal, a lone `{` or `}` | ZK0106; the text is kept as `Bad` |

A template slot ends at the `}` that matches its `{` on the same line (`open_brace`,
through `template_slot_end`). `quoted_end` skips a text or character literal inside the
template slot whole, with its escapes, `{{`, `}}` and its own template slots, and a
nested `{` is matched in turn. So `"a {"b {x}"} c"` is one literal whose template slot
contains the literal `"b {x}"`; the parser lexes that template slot again and reads the
nested literal with its own template slot.

A character literal must decode to a single scalar value (ZK0107), so a Burmese vanna
with signs does not fit. A raw identifier is the text up to the next backtick on the same
line, so a name can contain spaces (`` `ကိန်း ရှင်` `` lexes as one `raw_ident` token);
two backticks in a row are ZK0109.

## Comments

`//` and `///` run to the end of the line, and so does the word `မှတ်ချက်`. A comment
never becomes a token; it goes to `LexOutput::comments` with a `CommentKind` (`slash`,
`doc_slash`, `note`) and the span from the marker to the end of the line.

## Spans and `joined`

Spans in `LexOutput` are absolute positions in the `SourceMap`. The artifact that the
command writes stores them relative to the start of the file, with the normalized source
text inside, so we can read a dump again without the original file. The `joined` flag
tells a later stage whether white space came before a token; the number rules use it, and
so does the particle rule above.

## Errors and going on

Each lexer message is recoverable. The lexer skips the offending word, or makes a token
from what it decoded, and goes on, so one bad line does not hide the next.

| Code | Raised when |
| --- | --- |
| ZK0101 | a word is no token kind: a lone `=` or `,`, a loose `;` or `$` |
| ZK0102 | a word starts with a digit and is not a number |
| ZK0103 | Burmese and Western digits in one number |
| ZK0104 | a text literal is not closed before the end of the line |
| ZK0105 | a character literal is not closed before the end of the line |
| ZK0106 | an unknown escape or brace form |
| ZK0107 | a character literal with zero or several characters |
| ZK0108 | a raw identifier is not closed |
| ZK0109 | a raw identifier is blank |
| ZK0510 | a spoken spelling of a reserved word (a lint; see above) |
| ZK0511 | spoken and written spellings in one file (a lint warning) |

For `ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ဘစ် ဖြစ်ပါစေ။`, `zakac lex` prints
the message and still writes the tokens of the line:

```text
error[ZK0102]: invalid number literal "၁၈ဘစ်"
 --> bad.zaka:1:51
  |
1 | ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ဘစ် ဖြစ်ပါစေ။
  |                                   ^^^^
```

That is the plain form, as a pipe gets it. On a color terminal the word is colored in the
line, and the caret row follows how the terminal draws Burmese text; where the row is left
out, the word is underlined in its place. See
[How a message is set out](../compiler/diagnostics.md#how-a-message-is-set-out).

The parser sees a token list without the bad word and goes on from there. The exit code of
the command is a failure when an error was reported, but the artifact is written all the
same, so we can run later stages on a partial file. The texts of the codes are in
`zakac_diagnostics` (see [Diagnostics](../compiler/diagnostics.md)); the lexer just calls
the constructors in `zakac_diagnostics::lexer`.

## Extending the lexer

| To add | Change |
| --- | --- |
| a reserved word | a `Kw` variant in `token.rs` and a row in `RESERVED` (and in `Kw::is_particle` for a particle). The tests in `tests/reserved.rs` catch a spelling in the wrong sign order and a word that another word can swallow, through round trips over all pairs and a sample of triples of reserved words |
| another spelling of a part | a row, or a form in a row, of `ALIASES`; review the new lines of `tests/expected/aliases.tsv` and `tests/expected/spoken.tsv` |
| a phrase | a `Kw` for each piece, the joined variant, and a row in `PHRASES` (a longer row goes before a shorter one that shares its start); test both spellings, attached and apart, in `tests/phrases.rs` |
| a message | a row in `define_codes!`, the texts in `locales/codes.toml`, a constructor in `zakac_diagnostics/src/lexer.rs` |
| a named escape | a row in `NAMED_ESCAPES` in `text.rs` |

The golden files are `tests/inputs/*.zaka` against `tests/expected/*.zktokt`; rewrite
them with `ZAKA_BLESS=1`.

Next: [The parser](parser.md)
