---
title: Data kinds, generics and imports
description: How the parser reads data kind phrases and measure words, literals with a data kind, generics, traits and written-as hints, and how the loader reads the core module and the imports of a program.
---

# Data kinds, generics and imports

This page covers the parts of the parser that read data kinds (types) and the parts that
work on the whole program: the phrase of a data kind and its measure word (classifier),
literals with a data kind, generics and traits, written-as hints, and the loader, which
reads the core module and the files a program imports into one tree. [The parser](parser.md)
describes the engine around these parts. The rules of the language are on
[Kinds and values](../language/kinds.md), [Generics and traits](../language/generics.md)
and [Modules](../language/modules.md).

A small example: the declaration

```text
ကိန်းရှင် စာရင်းက သည် အကွက် ၃ ခုရှိသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
```

has a data kind with a count prefix, a sign word, a bit count, a base and a list level,
then the measure word `တစ်ခု`. `zakac parse` prints its `Kind` as below (spans left out):

```text
kind:
  base: int
  bits: 8
  signed: false
  wraps:
  - list:
      fixed: 3
  classifier:
    name: ခု
```

## Data kind phrases

`kind.rs` reads a data kind in this order:

```text
[အကွက် <count> ခုရှိသော]* [(လက္ခဏာရှိ|လက္ခဏာမဲ့)] [N ဘစ်] <base> [စာရင်း | လိပ်စာ | ၏ လိပ်စာ | <hint level>]*
```

`parse_kind` reads the count prefixes (`parse_counts`), the base part of the data kind
(`parse_core`: a bare `လိပ်စာ`, or a base word with its sign and bit count,
`parse_bits`, `parse_base_kind`), then the levels (`parse_levels`, `take_levels`). It
checks the bit counts against the base (`check_bits`, ZK0216): 8, 16, 32 or 64 for
integers, 32 or 64 for floats. In `kind.rs`, the base word `စာသား` gives the base `text`.
`builtin_kind` maps each data kind name the compiler gives to its `BaseKind`, and
`builtin_at` lets a declared name of the same spelling hide it (see
[Built-in data kind names](parser-expressions.md#operands)).

The levels after a base are `Wrap::List` (`စာရင်း`, with a count before it or none) and
`Wrap::Ptr` (`လိပ်စာ`, the pointer kind); there is no other level in the tree. While the
parser scans, `Suffix` has two more forms: `Suffix::Hint`, a written-as hint that follows
a data kind as a level, and `Suffix::Room`, the room of a bounded channel. Each
`စာရင်း` takes the innermost count prefix that is left (`parse_levels`).

The lookahead functions measure a data kind without consuming it: `kind_len` and
`kind_len_at` count its tokens, levels included; `plain_kind_len` counts one with no result
sides; `starts_kind` tells whether one starts at the cursor (a number starts one when
`ဘစ်` and an integer or float word follow it, as in `၈ ဘစ် ကိန်းပြည့်`); `at_size_of` checks
for `<kind> တစ်<measure word> ၏ အရွယ်အစား`. `kind_len_at` and `parse_kind_with` look for
the function forms first, then a result kind, then a plain data kind.

### List counts

A list count in digits is `Cells::Fixed`, and each other count is `Cells::Expr`, a full
expression (`parse_count`, through `parse_expr`), which the check works out. The lookahead
(`count_prefix_len`) runs from `အကွက်` to the `ခုရှိသော` or `ရှိသော` that closes it, counts
the counts nested inside it (as in a size), and stops at the full stop. While a count is
parsed, `in_count` is set and no struct literal starts, since the `ရှိသော` of the count
would otherwise end one (`N - ၁ ရှိသော`).

## Measure words

```text
<kind> တစ်<measure word>
<struct>တစ်<measure word>
```

Each place a data kind goes reads it with `parse_slot_kind` (`decl_head.rs`): a
declaration, a generic loop, a parameter, a field, a result, a parameter or the result of
a function kind, a conversion, a size, a default arm and the data kind of a literal. It
reads the data kind, then the measure word `တစ်WORD` (`parse_count_word`,
`is_count_word_at`), kept in `Kind::classifier`, or a struct name joined to its measure
word, `လူတစ်ယောက်` (`at_joined_count`, `joined_count_at`, `name_len` in `struct_lit.rs`).
A data kind with no word after it is ZK0234 (`needs_word`), with the word the data kind
needs in its help. `slot_kind_len_at` measures the same with a lookahead, and `slot_then`
asks whether a word follows a data kind with or without its measure word, which a
conversion uses for `အဖြစ်`. `named_kind_len` measures a data kind that starts with a name
no struct declaration gives, such as `လုပ် လိပ်စာ`: with a measure word after it, it is a
data kind all the same, and the check reports the name. `value_goes_on` tells a data kind
from a value that starts the same, such as a size whose word is missing or a struct
literal in the sign form.

A data kind inside a data kind needs no word, and neither does the data kind of an alias.
A size operand keeps its measure word in `Kind::classifier` of its unit too. The loader
writes `Kind::alias` when it puts the data kind of an alias in place of a use that adds no
levels, so the wording check counts it with the word of the alias.

## Result and optional kinds

```text
<kind> ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်
<kind> သို့မဟုတ် အမှားအယွင်းအနေဖြင့် <kind> ရှိနိုင်သော ဒေတာဆဲလ်
```

A result kind is `BaseKind::Result`: `Kind::parts` has `ResultParts { ok, err, optional
}`, and a side with no value (`အောင်မြင်သည့်သတင်း`, `မအောင်မြင်သည့်သတင်း`) is `None`. An
optional kind, `T ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`, is the same base with `optional` set and
no error side. `parse_cell_kind` reads both: the good side (`အောင်မြင်သည့်သတင်း`, a function
kind in parentheses or a plain data kind), then `ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`, or
`သို့မဟုတ်` and the error side (`မအောင်မြင်သည့်သတင်း`, or `အမှားအယွင်းအနေဖြင့်` and a plain
data kind) and `ရှိနိုင်သော ဒေတာဆဲလ်`. `cell_kind_len` measures the same phrase without
consuming it, so `kind_len_at` and `parse_kind_with` can tell a result kind from a plain
data kind with a lookahead (`result_kind` builds the node).

`kind.rs` reads a result kind wherever a data kind goes, so a declaration, a parameter and
a struct field accept it as well as the result clause. The S-expression writes it
`(ok ! err)`, with `-` for a side with no value. The list and pointer levels after a
result kind, and the count before it, belong to the whole result, so
`ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် စာရင်း` is a list of
results. A result kind ends in `ဒေတာဆဲလ်`, and no other phrase does.

## Function kinds

```text
<params> လက်ခံပြီး <kind> တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား
<params> လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား
<kind> တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား
လုပ်ဆောင်ချက် အမျိုးအစား
```

A function kind is `BaseKind::Fn`: `Kind::sig` has `FnSig { params, ret }`, each
parameter a `SigParam` with its owned and edit flags, its data kind with its measure word,
and the preposition when it is written. Each parameter is
`<kind> တစ်<measure word> ကို [markers] [လည်းကောင်း]`, as in a function signature with no
names (`sig_params_len`, `sig_item_len`, `markers_len`, `parse_sig_params`,
`parse_sig_item`). `fn_kind.rs` reads it (`parse_fn_head`, `parse_fn_parts`), and a
function kind can also open with phrases (`phrased_head_len`, `phrases_len`).

A data kind phrase can be a bare function kind at its first token, in the forms that
`FnForm` allows (`fn_form`):

| `FnForm` | Where | Allowed |
| --- | --- | --- |
| `Full` | no list is open (a declaration, a result, a conversion) | each form, a parameter list among them |
| `NoParams` | a list of parameters or fields is open (`Parser::in_list`, which `parse_params`, `parse_listed_fields` and the parameters of a function kind set) | the forms with no parameters |
| `NoFn` | the result of a function kind | `လုပ်ဆောင်ချက် အမျိုးအစား` alone |

A function kind with parameters inside a list goes in parentheses (`paren_fn_len`,
`parse_wrapped_fn`). `listed_head_len` and `ret_tail_len` work out the length of a function
kind from its closing words, `လက်ခံသော`, `ပြန်ပေးသော` and `လုပ်ဆောင်ချက် အမျိုးအစား`, so the
search stops at the first word that does not fit. A parameter whose data kind is a bare
function kind with parameters is read all the same and reported (ZK0239,
`at_bare_listed_kind`, `parse_bare_listed_kind`). The S-expression writes a function kind as
`fn(<params>) -> <ret>`. Function values and the long call are on
[Expressions and calls](parser-expressions.md#calls).

## Literals with a data kind

```text
[အခြေ N ဖြင့်] <digits> [ဟူသော <kind> တစ်<measure word>]
```

`literal.rs` reads a number literal (`parse_literal`): the base prefix (`parse_base`),
the digits (`literal_digits`), and the data kind after it (`at_literal_kind`,
`parse_literal_kind`). It converts one digit after the other in the given base
(`int_value`, `digit_value`), checks the digits against the base (ZK0219) and the range of
the data kind (`fits`, ZK0220); a data kind that is not a plain number kind is ZK0222. A
literal with no data kind is limited to 64 bits here (a larger one is reported as not
fitting); the check gives it the data kind of its place and checks the range there. A
float literal becomes an `f64` (`float_lit`, `based_float` for another base).

A list literal, `[...]` or `... တို့ပါသော စာရင်း`, accepts `ဟူသော <kind> တစ်WORD` after it as
well (`parse_list_kind`, a list kind or ZK0222), kept in `ListLit::kind`. One case is left
from the lexer: `1e5` is a valid base-16 literal as well as an exponent, so for a base
above 10 the parser reads the span again as digits.

## Table kinds

```text
<index kind> ဖြင့် ညွှန်းသော <value kind> ဇယား
```

A ဇယား (table, map) kind is the written-as hint of the `ဇယား` of the core module (see
[Written-as hints](#written-as-hints)): after the index kind, the rest of the hint is a
level, as `စာရင်း` is, and the parser builds the table kind from its two slots, kept in
`Kind::table`. The value kind can be each data kind, a ဇယား included, and the levels after
`ဇယား` wrap the ဇယား. An optional or a result index parses, so the check can refuse it.
Rows and literals of a ဇယား are on
[Expressions and calls](parser-expressions.md#table-rows-and-literals).

## Generics and traits

Kind parameters follow a name in the notation form, `<T၊ U: <trait> နှင့် <trait>>`
(`parse_notation_params`, `parse_kind_param`), or come after `သည်` in the word form, each
`ယေဘုယျဒေတာအမျိုးအစား <name>` as an item of a list in words, then `ကို` or `တို့ကို`,
`အစားထိုးသုံးနိုင်၍` and `၎င်း` (`parse_word_params`). While a declaration is parsed, its
kind parameters are data kind names (`add_kind_names`). Kind arguments go on `Kind.args`
and on calls, function values and struct values: after `::` or the name in the sign form
(`parse_sign_kind_args`, `at_turbofish`, `parse_turbofish`), before it in the word form
(`parse_word_kind_args`, `word_call_kind_args`, `give_kind_args`, `with_kind_args`), all in
`zakac_parser/src/generics.rs`. Inside kind arguments a `>>` token counts as two closing
marks (`closers`, `angle_len`, `expect_angle_close`).

A trait (`at_trait_decl`: a name, then `ဟူသည့်`) and a trait block (`at_trait_block`:
`ရှိစေရန်` in the statement) parse in `zakac_parser/src/traits.rs`:

```text
<name> ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
    <function signatures>
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

<kind> တွင် <trait> ဂုဏ်သတ္တိရှိစေရန် အောက်ပါလုပ်ဆောင်ချက်များ ကို ထည့်သွင်းမည်။
    <functions>
ထည့်သွင်းမည့် လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။
```

`parse_trait_decl` reads the name, the kind parameters in either form, the written-as hint
of the trait when it has one (`parse_trait_hint`), a notes block of the traits it needs,
the function signatures (`parse_trait_rest`, `parse_trait_fn_head`) and the closing line.
A function of a trait with a block (`parse_trait_fn_head`, then `parse_fn_block`) is a
default: `hoist_defaults` moves its block out after the trait as a generic function of the
trait's kind parameters, the first of these needing the trait, and the trait keeps the
signature with `has_default`.

`parse_trait_block` reads the data kind, the trait (`parse_trait_ref`), each function
(`parse_block_fn`) and the closing line. The functions of a trait block come out of it
into the statement list of the file (`parse_program`, `push_top`), each one a plain
function, and the block keeps their names, and `inherits` for its last line that accepts
the defaults:

```text
<trait> ဂုဏ်သတ္တိ၏ အခြားလုပ်ဆောင်ချက်များကိုလည်း မူရင်းအတိုင်း ဆက်ခံမည်။
```

(`at_inherit`, `parse_inherit`; another trait there is ZK0254).

A notes block after the function signature of a function or a trait,
`ဤနေရာတွင် သတိပြုရန်မှာ … သတိပြုစရာများ ပြီးဆုံးပါပြီး။` (`parse_notes`, `parse_note`),
gives the kind parameters their traits (`A တွင် <trait> ဂုဏ်သတ္တိ ရှိသည်။`, `give_bounds`;
a note on another name is ZK0248) and names local data kinds
(`C သည် <kind> ဖြစ်သည်။`). The local data kinds are data kind names for the rest of the
function, then `expand_local_kinds` puts their data kinds in their place, marked in
`Kind.alias` so that these count with `ခု`.

A trait with `အရာ` after it, where a data kind stands, is a trait value kind
(`BaseKind::TraitValue`): `trait_value_len` in `kind.rs` finds a trait as a hint
(`hint_trait_at`) or a name with its kinds in signs, then the word `အရာ`, which is no
reserved word, and `parse_core` reads the trait through `parse_trait_ref`. A hint of a
trait can start with a slot whose data kind phrase leads back to the same place, so
`scanning_trait_value` keeps the scan from starting again inside itself.

## Written-as hints

A written-as hint is the words that a generic data kind or trait is written with:

```text
ရေးသားရာတွင် K ဖြင့် ညွှန်းသော V ဇယားဟု ရေးသားနိုင်သည်
```

Each word of a hint that is a kind parameter of its declaration is a slot, where a use
writes a data kind; the other words are fixed, and the name of the data kind or trait ends
the hint. `zakac_parser/src/hint.rs` reads a hint twice:

1. Before a file is parsed, `declared_hints` reads each hint of the file from its tokens
   (`hint_words`, `read`), so a use can come before its declaration. `core_hints` reads
   those of the core module once, and the loader passes on the hints of the exported
   structs and traits a file imports (`exported_hints`, `imported_hints`). Each hint is a
   list of parts, a slot (the index of the kind argument it gives) or a fixed word.
2. The declaration then parses its own hint (`parse_hint_decl`), keeps the words on the
   declaration's `hint`, and reports a rule it breaks (ZK0252, `hint_fault`).

At a use, a hint that starts with a slot (`starts_with_slot`) is a level after a data kind
(`Suffix::Hint`, from `hint_level_at`), so it nests with the pointer and list levels; one
that starts with a word is a base (`hint_kind_at`); a trait hint goes where a trait is
named (`hint_trait_at`, in `parse_trait_ref`). The longest hint that fits wins
(`longest_hint`), and a slot reads the longest data kind (`kind_len_at`, then
`parse_kind_with`). A kind parameter can appear twice in a hint: at a use, its second place
must have the data kind of the first (`fill_slot`, through `Kind::same_as`; ZK0253).
`builtin_generic` turns `စာရင်း` and `ဇယား` with their kind arguments, from a hint, the long
word form or the sign form, into the list wrap and the table kind, unless the file
declares a data kind of that name. The list remains a separate level (`Suffix::List`),
since count prefixes go with it.

## The core module

The loader reads the core module, `"စံကား/အခြေခံအုတ်မြစ်"` (`stdlib/core/mod.zaka` in
`STD_MODULES`), as a file of the program (`read_core`), ahead of the first file that names
one of its traits or enumerations and does not declare that name itself, or that writes
`ပြောင်းလဲသော ရလဒ်ကောင်း` (`uses_core`). A part opened alone
in an editor counts as the core module (`std_module_at`), and the whole core module is
then not read beside it.

`mod.zaka` shares its parts, `kinds.zaka`, `traits.zaka`, `verbs.zaka`, `heap.zaka` and
`convert.zaka`, whole. The parse of each file knows the structs and enumerations of the
core module as data kinds with no import (`core_kinds`). A share of a part in `core/mod.zaka` reads it from `CORE_PARTS`
(`import_core_part`), each one a unit named `စံကား/အခြေခံအုတ်မြစ်/<part>`; an import from
another file cannot reach a part. The check counts each of these units as the core module
(`in_core`). The readers that take the declarations of the whole core module from its
words before a parse (`core_functions`, `core_builtins`, `core_hints`, `core_names`,
`core_builtin_fns`) read `CORE_SOURCE`, the parts joined into one text. A file that names
a fact of the target gets an import of it from the part that declares it
(`import_facts`, `named_facts`).

## Reading the imports

`load(root, &mut SourceMap, &LexerOptions) -> Result<Loaded>` reads a file and the files it
imports, and returns one tree for all of them:

```text
Loaded  { program, imports: Vec<EmbeddedFile>, diagnostics, alias_refs, open_conds }
Program { stmts, units: Vec<Unit>, target, span }
Unit    { name, first_stmt }
```

For each file, from the file that is run (`visit`):

1. Lex it, and parse it a first time to find its import sentences. The imports do not
   depend on struct names, so this parse finds all of them, and its tree is dropped.
2. For each import, join the path to the folder of the file and drop the `.` and `..`
   parts (`normalize`), so two spellings of one file meet. A file read before returns its
   unit again; a file on the chain of files being read is a circle (ZK0231); each other
   file is read through the source map and visited in turn (`import`), so its own imports
   come first. A file that cannot be read ends the load with a host error, which has no
   code.
3. Parse the file a second time with `parse_with`. Its node numbers go on from the file
   before, so each expression of the program has a distinct number, and the struct names
   it imports from exported structs (`imported_structs`) can start a data kind. Before that
   parse, `bring` works out the names each import brings, each with the unit that declares
   it (`origin`): the listed names of a named import; for a whole import, each name the
   file exports, less a name the importing file declares (`top_names`), one a named import
   brings, and one another whole import brings from a different unit, which is a clash
   (`add`). A clash leaves the name out of both, and `report_clashes` gives ZK0262 at its
   first use in the file. A whole share brings nothing.
4. Settle each compile-time conditional for the target (`comptime::settle`, run on the
   first parse too, so the imports of the arm it keeps are read). Give each import its
   names and set `unit` to the unit that declares them (`split_imports`, `split_import`: an
   import whose names come from several units through shares becomes one import of each).
   Give a file that names a fact of the target an import of it from the core module
   (`import_facts`), append a `Unit` and the statements of the file to the program, and
   note the structs, enumerations and kind aliases the file exports
   (`exported_structs`, `exported_prepositions`, `exported_hints`). `Loader::exports` notes
   each name the file exports with the unit that declares it: its own exported
   declarations, then each name it shares (`shared`, which gives ZK0261 for a name it does
   not import, and `pass_on`, which copies the struct names, the functions with their
   prepositions and the hints of the name from the unit it comes from).

For a file read from disk, `lib_paths_from` makes the lib of each C function that is a path
absolute (`is_lib_path`, in `zakac_ast`: it has `/` or `\`, ends in `.so`, `.dylib` or
`.dll`, or has `.so.` in it): a relative one starts at the folder of the file that declares
the function, as an import does. A lib name is left as it is, and so is a program read from
storage.

So the units come in post-order: each imported file ahead of the files that import it, and
the file that is run last. `Program::unit_of(i)` returns the unit of statement `i`, and
`Program::is_root(unit)` tells whether a unit is the file that is run. A program of one
file has no units, and both functions treat each statement as part of the file that is run.

The imported files go into each later artifact under `imports`, in reading order. That
order matches the order of the source map after the file that is run, so a decoded
artifact registers the files again in that order, and its spans, relative to the start of
the first file, remain valid. The parser owns the codes of this step: ZK0229 to ZK0232.

### The stdlib and programs with no file

An import whose path starts with `စံကား/` (`STD_PREFIX`, `is_std`) reads a module of the
stdlib (standard library) from `STD_MODULES`, which the build script writes from the
`[package.modules]` table of `stdlib/zakac.toml`, each file through `include_str!`:
`import_std` registers it in the source map as a file named after the module
(`std_file`), and reads it as each imported file, once. A name the stdlib does not have is
ZK0270, as a module a package lacks; the stdlib has no path to fall back on.
`std_replicas` is the folder where a language server writes copies of the stdlib, so an
editor can open a declaration of the stdlib.

### Packages

`Loader::import_path` reads each other import. When the first part of the path, up to the
first `/` (`dep_and_module`), is a key of the `[dependencies]` of the nearest `zakac.toml`
of the importing file, the import reads a module of that dep: `module_path` looks the rest
up in the `[package.modules]` of the dep (`Manifest::module_file`), else takes
`<rest>.zaka` under its root, and reports ZK0270 when neither file is there. A git dep
(`Source::Git`) reads its root from the lock file of the package of the file that is run
(`Loader::lock`, `checkout_dir` in the cache of `cache_root`), and a rev that the lock does
not pin, or a checkout that is not there, is ZK0271. A path that names no dep goes beside
the importing file, as before, except that a missing file whose path does not end in
`.zaka` is ZK0269 (`beside_or_unknown`). `package.rs` holds the manifest, the lock and the
folder of a checkout; `zakac fetch` (`compiler/zakac/src/fetch.rs`) is the one place that
runs `git` and writes the lock.

The loader reads manifests and lock files through the file loader of the source map, as
it reads sources, and keeps each manifest once (`Loader::manifests`). It keys each file on
its absolute path (`known_as`) in `done` and on the chain, so a file that one import reaches
through a dep and another beside the importer is read once. A unit of a file under the
root of a dep has `Unit::package` set, which `zakac test` reads to leave the tests of deps
out, and messages name such a file `<dep>/<path>`. The parser owns these codes too:
ZK0269 to ZK0271.

A root whose name is not a real file, such as standard input or the text of the explorer,
is parsed once with `Loaded::single`, and `refuse_imports` reports each import in it
(ZK0232). When the program imports from the stdlib, `with_std` runs the loader with `disk`
off, in a source map that starts with the file (`SourceMap::add_file`), so the modules
come after it; each other import is ZK0232. `Loaded::single` is also what the `run_phases`
function of each later stage uses, so a test that compiles a bare string needs no file at
all. The stage commands and `zakac run` call `load_settled` (the passes of
`zakac_semcheck::open_chains` over `load_resolved`), then `check_loaded` or `run_loaded`.

### Compile-time conditionals

A compile-time conditional (`IfStmt.comptime`, from `ကွန်ပိုင်းလုပ်ချိန်တွင်` after
`အကယ်၍`) parses as a conditional. At the top level its arms raise `Parser::top`, so their
statements count as top-level ones (`nested()`), with the functions of a trait block and
the defaults of a trait moved out (`push_top`). `settle` (`comptime.rs`) keeps the
statements of the first arm whose condition holds, spliced in place, through each block
(`flatten`, `choose`), and reports ZK0255 for a condition outside the facts. `load_for`
and `Loaded::single_for` accept the target; `load` and `Loaded::single` use the host's.
`fill_facts` writes the value of the target into each marked constant of the core module
(`DeclStmt.builtin`), and `Program.target` records the target when a file has a
conditional or names a fact.

A condition that has a call waits for the check. `settle` accepts a `Resolved` table, the
truth of each such condition on its `CondPlace` (the file name and the offset of the
condition in its file, the same in each read, `place`). A condition found there is settled
with its truth (`called`), and one missing leaves its chain in place, unsettled, with its
arms parsed and their imports not read (`open_cond`). `Loaded.open_conds` lists each open
condition with its place. `load_resolved` and `Loaded::single_resolved` accept the table,
and `load_for` and `Loaded::single_for` pass a blank one. The passes that fill the table
live in the check (`zakac_semcheck::open_chains`). `pub fn truth` settles a condition whose
calls the check has replaced with values (`Expr::Comptime`).

### Kind aliases in the loader

Once the files are read, `expand_aliases` (`src/alias.rs`) puts the data kind of each alias
in place of each use of its name: in a data kind, where the levels written after the alias
go around the levels of its data kind (`spliced`), as the name of a struct literal, and
before `ထဲမှ` in a member (`swap_qualifier`). Alias names belong to the whole program, as
struct names do, so one table serves each file (`collect`). A name that a struct, an
enumeration or an earlier alias has is ZK0316, a chain of aliases that comes back to its
start is ZK0336 for each alias in it, and a generic alias used with another count of kinds
than it declares is ZK0340 (`with_args`). `note` records each alias name beside its
declaration in `Loaded::alias_refs`, for the editors. Each exit of the loader
(`Loaded::single`, `with_std`, `load`) runs it, so the stages after the parser see no
alias; the parse artifact of one file keeps the `alias` statements as written.

Next: [Names and data kinds](semcheck.md)
