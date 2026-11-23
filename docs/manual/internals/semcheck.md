---
title: Names and data kinds
description: How the check of names and data kinds works after parsing, with an example, the tables it builds, its passes in order, the wording check that runs before it, and the codes it raises.
---

# Names and data kinds

After the parser comes the stage of names and data kinds (name resolution and type
checking), which decides whether the tree makes sense. It checks that each name is
declared where we use it, that `ကွင်းဆက်` sits in a loop and a return in a function, and
that each value has a data kind (type) that fits its place. The stage does not change the
meaning of the tree, and it produces a table with the data kind of each expression, plus
diagnostics. The code is in `compiler/zakac_semcheck`, and the wording check that runs
first is in `compiler/zakac_naturalness`. The rules that need a flow through the program
belong to the next stage, [the flow checker (borrow checker)](borrow-checker.md), and
[Stages](../compiler/stages.md) shows where this stage sits among the others.

Constant folding is not here: it works on the IR, and [The IR](codegen.md) covers it.

## An example

For this program:

```text
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
အသက် ကို အသက် နှင့် ၁ ၏ ပေါင်းလဒ် ဟု ထားမည်။
အကယ်၍ အသက် သည် ၁၈ ထက် ကြီးလျှင် အသက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

`zakac semcheck` writes the tree again, and then the kind table:

```text
kinds:
  0: isize
  1: isize
  2: isize
  3: isize
  4: isize
  5: isize
  6: isize
  7: bool
  8: isize
structs: {}
diagnostics: []
```

The number on the left of each line is the `NodeId` that the parser gave an expression. Id 0 is the literal `၂၀`.
Ids 1 to 4 belong to the second sentence: the target `အသက်` is 1, the operands `အသက်` and
`၁` are 2 and 3, and the sum around them is 4. Ids 5 and 6 are the operands of the
condition, id 7 is the comparison, which is a `bool`, and id 8 is the printed name. The
other entries are `isize` because the literals had no data kind after them and took the
data kind of their place.

## The data it builds

### `DataKind`

`DataKind` is `Copy` and has twelve forms:

| Form | What it has |
| --- | --- |
| `Int(IntKind)` | a sign and a `Width`, which is `Platform` or `Bits(n)` |
| `Float(bits)` | 32 or 64 |
| `Char`, `Bool` | |
| `Ptr(PtrKind)` | levels of address over a base: the one pointer kind of unsafe code |
| `Null` | the data kind of the null pointer `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`, which converts to each pointer kind and to nothing else |
| `List(&'static ListKind)` | a counted list: a count of cells of one data kind, inline; list kinds are interned so the form remains `Copy` |
| `Struct(&'static StructKind)` | a struct with its named fields, interned as list kinds are |
| `Seq(&'static SeqKind)` | a plain list or text, three words (address, count, room); see below |
| `Enum(&'static EnumKind)` | an enumeration; see below |
| `Fn(&'static FnKind)` | a function value: one word, the function it calls (see [Data kinds and conversion](#data-kinds-and-conversion)) |
| `Task(&'static TaskKind)` | a thread, a channel or an atomic number: one word, the number that names it to the scheduler (see [Calls a node stands for](#calls-a-node-stands-for)) |

`SeqKind { cell, text, boxed }` is the shape of a plain list. The flag `text` marks UTF-8
text, `DataKind::TEXT`, whose cells are `u8`, and `boxed` marks a box (below). Plain
lists are interned through `seq_of`. `is_text` reads the flag, while `as_seq`, `is_seq`
and `cell_kind` see plain lists alone, so the rules of text remain apart from the rules
of lists. `holds_seq` is true for a plain list and for each counted list or struct with
one inside. `holds_text` is true for text, for plain lists and for each list or struct
with one inside, so the compiler counts, drops and duplicates each of these the same.

`EnumKind` has the name of the enumeration, its value kind and its members, each with its
`MemberValue` (a scalar `Value`, text, or the items of a list or a struct; `enums.rs`). An
enumeration is interned on the signature `name<member=value|...;kind>`, which names it in
artifacts, with each value hex-coded. Storage has the value of the member, so the size,
the alignment and the `holds_*` tests are those of the value kind (`repr` takes it off).
An enumeration widens to nothing, converts to its value kind and back from a scalar one,
and is copied.

An enumeration whose members hold data is no `EnumKind`: `variants_decl` (`enum_check.rs`)
turns its declaration into a struct declaration that `collect_structs` makes with the other
structs, so a member that holds the enumeration again through a pointer or a shared struct
resolves as a struct field does. The first field is the tag, `u8` (`u16` past 256 members),
whose name `%v|<member>|!<member>|…` lists the members and marks the initial one; each member
that holds data has a field `%<position>`. No written name starts with `%`. `Variants`
(`variants.rs`, `DataKind::as_variants`) reads the members, the tag and the data of a member
off the struct. The buildable check takes one member that can be built, not each field.
`variant_check.rs` checks a member with its data, the test of a member (`IsVariant`), the
read of its data (`VariantData`), and the name a test gives the data: the condition of an
arm marks the test that can name data (`bind_site`), and `with_bind` declares the name in a scope
around the block. `variants_proven` keeps the member a test proved for a variable, as
`proven` does for results, so a read of another member is ZK0358. `foreign::crosses` and
`exchanged` refuse such a value at the edge to C.

A written data kind becomes a `DataKind` with the defaults filled in: no sign word means
signed, and no bit count means the platform size for an integer and 64 for a float. So the
plain `ကိန်းပြည့်` is `isize` and the plain `ဒဿမကိန်း` is `f64`. `isize` and `i64` are
different data kinds, even on a 64-bit platform, and neither widens to the other.

A *box* is a plain list of one cell with `boxed` set (`DataKind::box_of`). Its duplicates
share the cell, and nothing lends or duplicates it. A shared object (reference type) is
the box of its struct, so it needs no `DataKind` form: the struct table maps the name of a
struct declared `မျှဝေနိုင်သော` to its box, and `as_object` finds the struct inside. An
object is one word, never NULL, and its short name is the struct name with `@` after it. A
local that a closure captures lives in a box as well.

The checker keeps two flavours of pointer that no source can spell, in
`PtrKind::borrowed` and `PtrKind::raw`. The IR builder marks with them the addresses it
makes itself, such as the address of an argument for a mutable parameter (`inout`,
`ဒေတာ ပြောင်းလဲရန်`), so the flow checker does not take them for blocks to free. Each
pointer kind that a program writes is a plain one.

### Scopes and functions

`Scopes` is a stack of maps from a name to a `Binding { kind, access, span, from, exported,
value }`, where `Access` is `Var`, `Const` or `LoopVar`. A lookup runs from the innermost
scope out. A `kind` of `None` means that an error left the data kind unknown, which
silences follow-on messages.

`FnSigs` maps a function name to its overloads, each an `FnSig { params, ret, unchecked,
.. }`, and `unchecked` is true for a function declared `စိတ်မချရသော`. `Scopes` also keeps
the functions that each scope declares, beside its names (`declare_fn`, `lookup_fn`,
`mark_closure`), and a stack of function frames (`enter_fn`, `leave_fn`). When a lookup
finds a local of an enclosing function, or `lookup_fn` finds a closure declared in one, it
records a `Capture { name, decl }` on each open function between the two (`note_capture`),
and `leave_fn` returns what the function it closes used from around it. `local` finds a
local with no note of a capture. `FnSig::closure` marks a nested function with captures.

### The kind table

`KindTable` is `BTreeMap<NodeId, DataKind>`, the product of the stage. The id is the
`NodeId` that the parser gave each expression, as [the example](#an-example) shows.

## The driver

Each stage crate has a `run_phases` function that runs the stages before it. For this
stage, `zakac_semcheck::artifact::run_phases` runs the whole front end:

```text
 lex ──▶ parse ──┬─ a lexer or parser error? ──▶ stop: no check of the tree
                 │
                 ├──▶ zakac_naturalness::check ──▶ warnings ZK05 ────────┐
                 └──▶ zakac_semcheck::semcheck ──▶ data kinds, ZK03 ZK04 ┤
                                                    all diagnostics ◀────┘
```

We skip both checks when the lexer or the parser reported an error, so that one broken
sentence does not start a chain of follow-on errors. The result is `Checked { program,
kinds, diagnostics }`: the tree, the kind table and one list of diagnostics, with the
lexer's first, then the parser's, then the wording warnings, then the errors of names and
data kinds.

### Counts written as expressions

`semcheck` takes the program as `&mut Program`, because it works out each list count
written as an expression (`consts.rs`). Where a data kind is read (`written_kind`, and the
struct fields in `visit_struct`), `work_out_counts` checks the count as an expression and
works it out with the folder's own math: `value.rs` and the operator enums of
`const_op.rs` live in this crate, and the IR builder uses them from here.

`work_out` follows lowering. It works out each operand in its recorded data kind,
converted where lowering converts it, so literals, constants, operators in both
spellings, comparisons, conversions, `၏ အရွယ်အစား` and the size operands
(`<kind> <count> <word>စာ`, `<count> ဘိုက်စာ`) give the value that the folder would give.
A constant list or struct is kept as its items (`Folded`, kept in `Checker::aggs` while a
run lasts) when each item is known, as the folder's `is_known` asks, so a cell, a field or
the count of a plain list can read it (`cell_place` reports ZK0603). The count of a
counted list comes from its data kind, as lowering reads it. The test
`counts_agree_with_the_values_the_program_computes` (`zaka_vm/tests/values.rs`) gives
each form to a constant and to a count and checks that the two agree, so the two walks
keep in step. An overflow (ZK0601), a division with zero (ZK0602) or a shift too far
(ZK0407) is reported as the folder reports it. Each constant declaration keeps its value
in its `Binding` (`ConstVal`).

A size that depends on a count still open waits (`Stop::Pending`, `size_waits`,
`open_structs`), so no count is fixed from a stand-in. A run records each count
(`CountTable`) and the top-level constants (`KnownConsts`), and `write_counts` writes each
worked-out count back as `Cells::Fixed`. `semcheck` runs again while a run writes a count
or finds new constants, so a struct can name a constant declared below it and one count
can wait on another. The last run produces the output, and a count still open is ZK0227
(a count that waits on itself among them). A program with no count expression is checked
once.

### Artifact and options

`SemcheckArtifact` is the form we can load again (text first line
`# zakac-semcheck-artifact v1`, files `.zksemt` and `.zksem`). The command has these flags:

| Flag | Effect |
| --- | --- |
| `--target-bits` | the width of the platform-sized integer kinds: 8, 16, 32 or 64; 64 when not given, or 32 for a WebAssembly CPU |
| `--target-os`, `--target-arch`, `--target-abi` | the target the facts of compile-time conditionals describe |
| `--lint-allow`, `--lint-warn`, `--lint-error` | the lint levels |
| `--comptime-allow-*` | the rights of the comptime runs (`set_grants`) |

These become `SemcheckOptions { platform_bits, lint, no_unsafe, as_module }`, and the
later stages receive the same options. `no_unsafe` comes from `--no-unsafe` of the later
commands (`irgen` and on), and the `semcheck` command leaves it off. `SemcheckOptions::levels`
returns the lint levels, which `settled` uses while the program is read and `check_loaded`
after the check, so an allowed code stops no pass. The option `as_module` has no flag: the
language server sets it for an open module, and the check then asks for no program start
block (ZK0327).

### References for the editors

The check also fills `SemcheckOutput::refs`, a `RefTable` for the editors. For each name it
resolves, the table pairs the span of the name where it is used with the span of the
declared name in the file that declares it, and a declared name is paired with itself.

| Names | Where the check records them |
| --- | --- |
| variables, parameters, constants | where a lookup finds them (`infer_name`, `check_writable`, `call_target`) |
| functions | where a call settles on one overload, from the `decl` span that each `FnSig` keeps |
| imported names | from the span in their own file, kept in `Binding::from` |
| structs, fields, enumerations, members | from `KindDecls` (`refs.rs`), gathered from the declarations of all files; field and member uses where the check knows the struct or enumeration, and names in written data kinds through a walk of the tree |
| a top-level constant that a list count names above its declaration | from `KindDecls` too |
| kind aliases | from the loader (below) |

A run writes each count it works out back as digits, so `semcheck` keeps what each run
records, not just the pairs of the last run. Kind aliases are gone before the check: the
loader records each alias name beside its declaration while it expands them
(`Loaded::alias_refs`, from `expand_aliases`), and `check_loaded` joins those pairs to the
table. At a place where the check also resolved the struct or the enumeration that an
alias names, the alias wins.

## The passes, in order

### 1. Wording (`zakac_naturalness`)

This pass is a visitor over the tree. The parser accepts each particle on a verb, and
either `ကို` or `ဟု` after a printed operand, so this pass decides which wording is natural
and reports the rest as warnings. A warning never fails a build, and
`--lint-error naturalness` raises them to errors. The pass reads the tree alone: the
`particle` and `next` fields that the parser kept on verbs, true/false words and `မဟုတ်`,
and the `slot` of a print.

`site.rs` turns a node into a `Site`: a clause-final word, a group in parentheses, a value,
a print operand, or the particle before `သုံး၍`. `rules.rs` has a table `RULES` of
`{ code, fires }`, and the first row that fires at a site decides the code, so the order
matters when two rules overlap.

| Code | Fires when | Wants |
| --- | --- | --- |
| ZK0501 | a negative word, then `သို့မဟုတ်`, with `သည်` | `ပါ` |
| ZK0502 | a particle before `လျှင်` or `နေသရွေ့` | none |
| ZK0503 | `မဟုတ်` with `သည်` or `ပါသည်` | `ပါ` or none |
| ZK0504 | `ပါ` or `ပါသည်` before `ဖြစ်၍` | `သည်` |
| ZK0505 | a sentence or a string printed with `ကို` | `ဟု` |
| ZK0506 | a plain value printed with `ဟု` | `ကို` |
| ZK0507 | parentheses that add nothing | none |
| ZK0508 | a clause that has a value ends in `ပါ` or `ပါသည်` | `သည်` |
| ZK0512 | a verb or a truth value before `မဟုတ်`, with no particle or `ပါ` | `သည်` |

A run of `"မင်္ဂလာပါ" ကို ပြသမည်။` produces `warning[ZK0505]` with the help `write "... ဟု
ပြသမည်။"`. The natural wording is a matter of the language, and
[Natural wording](../language/naturalness.md) covers it.

### 2. Function signatures (`collect_fns`)

A first walk over the top-level statements records the signature of each function: the
data kinds of its parameters, of its result and of its error value, and its export mark.
Because of this walk, a call can come before the declaration. The table has one map for
each file of the program: the first declaration of a name in a file wins, and a second one
in the same file is ZK0304. Two files can each declare a function of one name.

A signature keeps the preposition of each parameter (`FnSig::prepositions`, `None` with no
prepositions). After the count check (ZK0314), `check_prepositions` compares the
prepositions of a preposition call with it, one argument after another, and reports
another word as ZK0331. A preposition call of a signature with no prepositions is ZK0332.
A call through a value uses the prepositions of the value's function kind, because the
parser's name table knows no scopes and so cannot widen what a function kind allows.

**Overloads.** A file can declare several functions of one name: `FnSigs` maps a name to
its overloads in the order of their declarations, and the same parameter kinds,
prepositions and result kind twice are ZK0304 (`FnSig::same_params`). Each signature has its inner name (`FnSig::inner`, from
`overload_name`): the written name for the first, `<name>#<n>` for the n-th. `check_call`
runs `choose`. With more than one function of the name, `fits_call` keeps the ones whose
count, prepositions (when the call writes prepositions) and parameter kinds fit, with a
trial of `expect_kind` on each argument that puts the diagnostics back. When several
fit, `choose` keeps the ones whose result kind is `Checker::call_expect`, the kind the place
of the call expects, which `infer` sets for each call expression. One fit is
recorded in `Checker::chosen` under the span of the name at the call, none is ZK0332 and
more than one is ZK0333. After the last run, `write_overloads` (`overloads.rs`) writes the
inner names into the program: each later declaration of a name, each call in `chosen`, and
each import of an overloaded name. The IR builder, the flow checker and codegen then see
one function for each name. An import brings each exported overload (`import_fn`), and a
name that a file both declares and imports is ZK0304.

**External functions.** An external function (`FnDecl::external`) must be one of the
functions the host serves. `check_external` asks `host::find`, a thin adapter over the
table `HOST_FNS` of `zaka_host`, for the row with its name, parameter kinds and result
kind (a plain value or a result with an error kind), and reports ZK0334, with `host::listing` as
help, when no row matches. The table matches data kinds through their names, so the
stream structs and error enumerations are the ones the stdlib declares. The IR builder
asks `host::find` again for the host function of its one statement, with the written name
of an overload (`#<n>` cut off). A function signature that ends in
`အခြားတန်ဖိုးများနှင့်အတူ` on a host function is ZK0334 too, since the host table has no
function with other values.

**C functions.** A C function (`FnDecl::lib` is set) goes to `check_c_fn` instead, which
asks `foreign.rs`:

| What | Rule | Code |
| --- | --- | --- |
| the C name | `is_c_name`: ASCII letters, digits and `_`, no digit first | ZK0337 |
| a parameter | `crosses_param`: a data kind that `crosses` accepts, text, a callback (`is_callback`), a list of numbers, or a handle | ZK0465 |
| the result | `crosses_back`: a data kind that `crosses` accepts, or an optional of text (`is_c_text_result`) or of a handle (`is_c_handle_result`) | ZK0465 |
| `ဒေတာ ပြောင်းလဲရန်` on a parameter | `edit_crosses`: a list of numbers or a struct that crosses; `number_cells` finds the integer of up to 64 bits, `f32` or `f64` cells of a plain or counted list | ZK0466 |
| the declaration | must be `စိတ်မချရသော`, since C runs outside the checks | ZK0472 |
| a second declaration of the C name | the same parameter and result kinds, in each file and among overloads (`Checker::c_names`) | ZK0338 |
| its address | refused, as for each external function | ZK0464 |

The data kinds that `crosses` accepts are the integers of 8 to 64 bits, `f32`, `f64`,
`bool`, pointers, and structs and counted lists of these. An enumeration is left out,
since Zaka keeps it as an `isize` and a C enum is an `int`. A result and an optional are no
C struct, so `crosses` refuses them as cells. Since C runs outside the checks, a call to a
C function needs unsafe code, as calls to each unsafe function do.

A callback (`is_callback`) is a function kind whose result is a scalar or an address that
`crosses` accepts, and whose parameters `callback_param` accepts, with no
`ဒေတာ ပြောင်းလဲရန်`: a scalar, an address, a plain struct, text, an optional of text, a
handle, or an optional of a handle. A bare text result is ZK0465, and so is the old form
`စာသား သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ်`; the help of both names the optional.

A call that passes a handle parameter of the function around it (`FnCtx::handles`), as it
is or read out of its optional, to a C parameter marked `အပိုင်အနေဖြင့်` (`FnSig::closes`)
needs unsafe code (ZK0468, `is_handle_param`), because the caller, or the C code that
called back, still has the handle.

A function signature that ends in `အခြားတန်ဖိုးများနှင့်အတူ` sets `FnSig::variadic`. A call
then needs at least the named arguments (ZK0314 for fewer), and `check_rest_arg` infers
each other value with no expected data kind and asks `crosses_param`, so a struct can be
an other value too (else ZK0465).

**Enumerations.** Before the function signatures, `collect_enums` (`enum_check.rs`) makes
the data kind of each enumeration, on its name and on its stem. It reports a member name
used twice (ZK0328) and a name that a struct or another enumeration has (ZK0316). A member
value can name a constant, so the walk works the values out (`check_enum`, once the walk
has the top-level names of the file):

- The first value decides the value kind, and an enumeration with no values is `isize`.
- Each other value converts to it (`expect_kind`) and folds (`constant_of`: `coerced` for a
  scalar, `folded` for text, a list or a struct, with `Folded::Text` for constant text).
- A member with no value counts on from the one before it (`count_on`, ZK0461).
- A value that does not work out is ZK0456, and one that two members share is ZK0457.

The values go into `found_enums`. Until a run has them, an enumeration with a written
value is treated as `isize` members numbered from 0, and `semcheck` runs the check again
while the values change, as it does for list counts. `collect_structs` reads a field of an
enumeration kind through that table, and the enumerations then join the struct table, so
each place that reads a named data kind finds both.

A member marked `ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်` is the
[initial member](../language/enums.md#the-initial-member): `EnumKind::initial` names it,
and the signature of the data kind writes `!` in front of it. A second mark is ZK0480
(`two_initial_members`). With no mark, the enumeration has no initial value, and the
first member is no default.

**Structs.** `collect_structs` (`struct_check.rs`) then breaks the loops of plain structs
(`break_loops`, ZK0427 for a plain struct that contains itself) and reports the data kinds
that no program can build (`report_unbuildable` in `buildable.rs`, ZK0481). With no NULL,
a field of a struct kind needs a value of that kind before the struct can have one, unless
the field is an optional or a plain list, which can be blank. A least fixpoint marks the
structs that can be built: first those whose required fields contain no struct, then each
whose required fields name marked structs. A struct left unmarked waits on itself. The
check reports one ZK0481 for each loop that passes through a shared struct, naming the
shortest loop, and nothing for a struct that contains a struct of a broken loop, since
fixing the loop fixes it.

The same first step finds the file of each struct and enumeration (`struct_homes`) and
checks each exported declaration with `private_in_exports`: a data kind of an exported
function, variable, constant or struct that names a struct its file keeps private is
ZK0324.

### 3. The walk (`Checker`)

`semcheck(program, options)` pushes the global scope and visits the tree with a `Checker`,
which implements `Visitor`. It has one method for each statement form, and it calls
`infer(expr, expect)` for each expression. The table lists each form with its main checks,
and the sections after it give the details.

| Statement | Checks |
| --- | --- |
| declaration | a constant has a value; the value converts to the data kind, or decides it when none is written; one declaration of a name in a scope (below) |
| assignment | the target is a place, declared, not a constant or a counted-loop variable; the value converts to the data kind of the place |
| block, `if` arm | a block is a new scope; a condition is `bool` |
| while, generic loop | a `bool` condition; the loop depth grows with one |
| each loop | a list or a pointer to a list (`each_list`) for an each-cell loop, text for an each-character loop, else ZK0434; the loop variable gets the cell kind or `char` |
| counted loop | bounds and step are integers (ZK0405); the loop variable gets the common data kind of the bounds; a literal step of zero or less is ZK0409 |
| `ကွင်းဆက်`, `နောက်တစ်ကြိမ်ဆီသို့` | the loop depth is above zero in the current function |
| function | distinct parameters, one scope for the parameters and the block, a return on each path (ZK0339) |
| return | in a function; a value just when the function has a result, and the value converts to the result kind |
| results and optionals | the table under "Results, errors and optionals" below |
| leave for C | `visit_leave`: the value is of a function kind (ZK0462) |
| call | the function exists; one argument for each parameter; each converts; the mutable parameters follow the rule below |
| heap sentences | the target is a place of a pointer kind (ZK0417); the size is `usize` |
| blit, fill | `visit_blit`, `visit_fill`: below |
| add, remove, replace, writes to text | the table under "Lists and text in the walk" below |
| switch, member | "Switches and members" below |

Each subtree is inferred once, so a diagnostic is never repeated. When an operand has an
error, its operation returns `None` and reports nothing more, and an operation with two
bad operands reports the first. The walk goes on past an error, so one run can report
several. For this program:

```text
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀၀ ဖြစ်ပါစေ။
အမည် ကို ပြသမည်။
အကယ်၍ ၁ ဖြစ်လျှင် "a" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

the check reports three errors:

```text
error[ZK0406]: the literal 200 does not fit i8
 --> bad2.zaka:2:57
error[ZK0301]: "အမည်" is not declared
 --> bad2.zaka:3:1
error[ZK0402]: a condition must be a boolean, found isize
 --> bad2.zaka:4:7
```

At the end, the diagnostics are sorted in source order.

### Declarations and functions

**Declarations.** The null pointer `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` names no data kind, so as
the value of a declaration with no written data kind it is ZK0430, and `ကနဦးတန်ဖိုး` there
is ZK0478. The check declares the name after it checks the value. A declaration with no
value is allowed here, because [the flow checker](borrow-checker.md#set-before-read)
proves that each path sets it before a read. In a module (a file that another file
imports, or a program built with `as_module`), a global declared with no value is ZK0483
(`module_global_needs_value`, from `units.rs`), since a module has no program start block
to set it.

**Functions.** The parameters are distinct, and one scope has both the parameters and the
block. The variable `ရလဒ်` is gone: a use of it is ZK0301, with the help line
`codes.ZK0301.help_result`. The loop depth restarts at zero. `visit_fn` splits the written
result kind into the good side and the fault side for a result kind with an error side
alone: an optional is a plain result, and the program start block never fails. When the
function has a result, the block must return on each path, else ZK0339
(`returns_on_each_path` in `proof.rs`). A path returns when the last statement is a
return, a block that returns, an `if` with an else whose arms each return, or a switch
with a default whose arms each return; a loop never counts. The program start block and
an external function are exempt.

**Calls and mutable parameters.** For a mutable parameter (`inout`, marked
`ဒေတာ ပြောင်းလဲရန်`), the argument is a writable place (`place_kind`: ZK0410, ZK0306) of the
parameter kind, with no widening (ZK0401), and its variable appears in no other argument
(`check_edits_apart`, ZK0433).

**Blit and fill.** `visit_blit` and `visit_fill` check that each address is a value of a
pointer kind, not a place (ZK0417). The count is `usize`, and the fill value is of the
unit kind, or a `u8` for a size in octets (ZK0401). `owned_cells` refuses cells of the unit
kind that contain a counted block, or, for a size in octets, such cells of the data kind
that each address points at (ZK0442).

### Results, errors and optionals

| Form | Rule | Code |
| --- | --- | --- |
| a plain return in a function that can fail | `visit_return` checks it against the whole result kind (`whole.or(ret)`), so a good value, `<e> ဟူသော အမှားအယွင်း` or a whole result fits | |
| a plain return in a function with an optional | a plain value or `အလွတ်` fits | |
| `လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်` (`ReturnStmt::fault`) | needs a function that can fail | ZK0447 |
| `<e> ဟူသော အမှားအယွင်း` | the expression `make_fault`: `infer_make_fault` uses the result kind of the place (the expected data kind) and checks `<e>` against its error side | ZK0476 where no result kind with an error side is wanted |
| `T ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` | becomes `DataKind::optional(some)`, the struct that `DataKind::fallible` makes, with a tag and a field `some` for the data; `is_optional` tells it from a result | |
| `အလွတ်` | `infer_no_data`: gets the optional kind that the place expects | ZK0477 where no optional is expected |
| a plain value in a place of an optional or a result kind | goes in as the good side | |
| `ထဲတွင် ဒေတာ ရှိ`, `ထဲတွင် ဒေတာ မရှိ`, `ထဲမှ ဒေတာ` | `infer_result_part`: the test word must fit the data kind (`ဒေတာ` on a result, or `ရလဒ်ကောင်း` or `အမှားအယွင်း` on an optional, do not) | ZK0475 |
| a call statement to a function with an error kind | must pass the failure on | ZK0443 |
| a pass-on | needs a callee that can fail | ZK0444 |
| a pass-on in a function that cannot fail, in the program start block, or in top-level statements | refused | ZK0445 |
| a pass-on with another error kind | converted through a plain `ပြောင်းလဲ` of the two kinds, or the instance of a generic one (`error_conversion`, `error_instance`, `OpCalls::fault_vias`, then `CallExpr::fault_via` and `CallStmt::fault_via`, which `checked_call` lowers), else refused | ZK0446 |
| the good result of a callee with none | refused | ZK0315 |
| a whole result as an operand | refused | ZK0403 |

An expression call with no pass-on is a whole result, of the result class `%fallible`,
built with `DataKind::fallible(ok, err)`. Each side is optional, and a side with no value
has no field; a message writes `isize result or text error`, `success or text error` and
`isize result or failure`. A written result kind becomes the same class, in a declaration,
a parameter, a struct field (the sugar is resolved in `structs.rs`, where the field kinds
are read) and a result clause. A whole result fits a place of the same data kinds alone.
An optional is no failure, so `check_pass` and `visit_fn` skip it, and a call that returns
an optional needs no pass-on.

### Lists and text in the walk

`growing_cells` (from `visit_add` and `visit_remove`) checks an add and a remove: the list
is a writable place (`place_kind`: ZK0410, ZK0306) of a plain list kind, else ZK0437. That
covers a counted list, a pointer to a list, and `နောက်ဆုံးအကွက်` on text.

| Form | Rule | Code |
| --- | --- | --- |
| add in front of a cell (`insert`) | checks the list of the cell and its number (`cell_number`) | ZK0423 |
| add in front of a character | checks the text (`is_text_place`), then goes on as an add to text | ZK0403 |
| `၏ အရှေ့တွင်` after another value | refused | ZK0439 |
| remove of the last character (`shrinking_text`) | needs text | ZK0403 |
| the value of an add | converts to the cell kind | ZK0401 |
| add to text (`text_add`) | the value is text or a `char` | ZK0401 ("expected text") |
| assignment to a cell of text | refused | ZK0429 |
| assignment to a character of text (`char_write`, when `char_target` finds `၏ စာလုံး n` or `၏ n ခုမြောက်စာလုံး` as the target) | the text is a writable place (`place_kind`), the number an integer, the value a `char`; a target that is no text is ZK0403 | ZK0423, ZK0401 |
| replace (`visit_replace`) | the text is a place (`place_kind`) of text (`is_text_place`), both ends are integers (`cell_number`), and the rest goes as an add to text (`text_add`) | ZK0403, ZK0423, ZK0401 |

### Switches and members

`visit_switch` infers the subject once, and each case of each arm goes through
`check_case` (`switch_check.rs`). Over an enumeration, a bare name that is a member
(`member_case`) means the member before a variable, and a name that is neither is ZK0329.
The check infers each other case with the subject kind as its expected data kind and
checks it as `==` checks its sides (`pair_result`). A case is known as a member position
over an enumeration and as a folded value otherwise (`CaseHit`), and two cases known as
one are ZK0460. `check_default` checks the data kind of the default arm (ZK0459), or, with
no default arm, that the cases over an enumeration name each member (ZK0458). With no
subject kind, the check skips the cases.

`infer_member` checks a member, `<stem>များ ထဲမှ <member>`: the name before `ထဲမှ` must be
the stem of an enumeration with `များ` (ZK0330, with help when the enumeration is written
without its `များ`), and a member that it lacks is ZK0329.

### Modules

The walk goes over the statements of each file in turn, in the order the loader put them,
and `visit_top` moves to the file of each top-level statement first. Moving to a new file
keeps the top-level scope of the file that ended in `saved`, opens a blank scope, and sets
`fns` to the functions of the new file. A statement of an imported file that is not an
import, a function, a struct or a declaration is ZK0323, and so is a program start block
there.

`start_rules` looks at the file that is run before the walk: a second program start block
is ZK0325, each top-level statement that runs is ZK0326, and a file with no program start
block is ZK0327, reported at its start. `collect_fns` leaves the program start block out
of the function map, so no call finds it. `visit_fn` checks it as a function, then checks
its function signature with `check_start_header`: at most two parameters, each a plain
`စာသား စာရင်း` that the function cannot change (ZK0440), and no result or an integer one;
a result kind is ZK0441.

An import runs at its place, at the top of the file. For each name, it looks in the file
it reads for an exported function (from that file's function map), an exported variable
or constant (from its saved scope, where the declaration recorded the mark), and an
exported struct. Each one found joins the current file: the function goes into `fns` and
the variable into the top-level scope, without the mark, so it is not exported again. A
name found nowhere is ZK0322, with help when the file declares it without the mark. A
name that the file declares as well is the usual ZK0302 or ZK0304, since the import
declares it.

Struct names form one table for the whole program, so two files with a struct of one name
are ZK0316. The parser keeps a struct name of another file from starting a data kind
unless the file imports it. When a name is unknown (ZK0301, ZK0303), `with_hidden` looks
for it in the other files and adds help that names the file that declares it.

## Data kinds and conversion

An implicit conversion (`widens_to`) never loses data:

| From | To | When |
| --- | --- | --- |
| integer | integer | the target is wider and no sign is lost (an unsigned kind can go to a wider signed one) |
| integer | float | the integer width is at most the significand of the float: 24 and 53 bits for 32 and 64 |
| float | float | the target is wider |
| the null pointer | pointer | in each case |
| function kind | function kind | the same parameter kinds, `ဒေတာ ပြောင်းလဲရန်` markers and result, and the same prepositions when the target has prepositions |
| pointer | pointer | the same shape, or one is the opaque `လိပ်စာ` |
| counted list | plain list of the same cell kind | when the counted list has cells; lowering moves the cells to a heap block (the VM operation `ToHeap`) |
| plain list | counted list | never: the count of a counted list is part of its data kind (ZK0401) |
| `char`, `bool`, list, text, object | other data kinds | never |

The conversion verb (`cast_allowed`) allows more: number to number, `char` to integer and
back, `bool` or an enumeration to integer, a plain list of u8s to text (the VM checks that
the u8s are UTF-8, `check_utf8`), an object to a pointer and back, address to address, and
address to integer and back. Each conversion that makes or reads an address needs unsafe
code (ZK0468). An object turned into a pointer keeps a count on the object until the
pointer turns back into an object.

### Function values and outside kinds

`DataKind::Fn(&'static FnKind)` is a function kind (`fn_kind.rs`): the data kind, the
`ဒေတာ ပြောင်းလဲရန်` marker and the preposition of each parameter, and the result. Each
function kind is made once (`FnKind::make`), as a list kind is. `FnKind::fits` accepts a
function in a place of a function kind with the same parameter kinds,
`ဒေတာ ပြောင်းလဲရန်` markers and result, and with the same prepositions when the place names
prepositions: a function kind with no prepositions accepts a function with prepositions or
without. No value widens to a function kind except a function: a function value is never
NULL, and a missing one is an optional function value.

The short name is `fn(<params>) -> <ret>`, each parameter `[mut ]<kind>[(<preposition>)]`.
Inside another name, it is written in parentheses (`spell_inner`), so `split_marks` and the
struct signatures (`split_top`, `bracket_step`, which skips the `>` of `->`) can read it
back. A struct field of a function kind writes its parts with each struct as its bare
name (`field_text`, `collect_kind`, `expand`, `kind_of`), as each other field does.

`fn_value.rs` has the calls. `call_target` finds what a call runs: the declared functions
of its name; else a variable, a parameter or a constant of a function kind, whose data
kind provides one signature (`value_target`); for a place, the data kind of the place. A
name that finds both a function and a value of a function kind is ZK0335, and a value of
another data kind is ZK0462. `check_pass` uses the result kind from the signature, so a
call through a value passes a failure on as a direct call does.

`infer_fn_addr` decides the function kind of `NAME ဟူသော လုပ်ဆောင်ချက်` (`sig_kind`). With
overloads, it keeps the one whose function kind fits the place (ZK0463 when none or several
do) and records it in `chosen`, so `write_overloads` gives the value its inner name. An
external function has no value (ZK0464), and an unsafe function (`စိတ်မချရသော`) cannot be
one either (ZK0469). `==` and `!=` accept two function values of one function kind
(`fn_pair`), and so does the test of one object, `X နှင့် Y သည် တစ်ခုတည်း` (`same_pair`),
which accepts two objects or two function values.

**Nested functions and closures.** A function with statements can be declared inside the
statements of a function or a block. `visit_fn` saves and restores `func`, `loop_depth`,
`proven` and `touched` around a nested function, and `declare_nested` declares it in the
scope around it (ZK0304 for a second one of its name). `fns_named` finds nested functions
before top-level ones, so a nested function hides a top-level one of its name, and a
nested function has no overloads.

A nested function writes no capture list: the check notes each local of an open function,
and each closure, that its statements name as a capture (`note_capture`). `captures.rs`
settles which nested functions are closures in a fixpoint (a function that names a closure
is one too), and `write_captures` writes the captures of each into `FnDecl::captures`,
each a `CaptureItem` with the mode `Lend`. The IR builder keeps each captured local in a
box, which the closure shares with the block around it.

A closure passed to a C function is no error: the call lends it, and a leave statement
(`visit_leave`) lets C keep it. `visit_leave` infers the value and reports ZK0462
(`codes.ZK0462.leave`) when its data kind is no function kind, and the check of a call to
a C function accepts a function value of each form. A call from C to a closure, with no
call running and no leave, stops the run with the trap `closure_escape`.

**Data kinds from outside the program.** A struct declared `ပြင်ပမှ လာသော ဒေတာအစုအဝေး`
(`StructDecl.external`) is a handle. In `struct_check.rs`, its struct gets one hidden
field (`extern_field`): `%c` (`EXTERN_FIELD`, of `DataKind::handle_pointer()`) and then
a 16-hex-digit FNV-1a hash of its C fields, each as `name:kind;`. So two programs that use
one name for two C structs make two data kinds, where otherwise the C fields of the first
would stick to the name for the rest of the process. The struct is boxed as a shared one
is, so duplicates share the box. `is_external` finds such a struct, and `as_external`
(`kind.rs`) finds it behind the box.

`set_c_fields` then gives it the fields that the declaration names (`c_fields`). A number,
a boolean, a counted list of numbers, text, and a handle of another data kind from outside
can be read (`foreign::crosses`, `number_cells`), and another data kind is ZK0465.
`c_field(name, platform)` returns the data kind and the offset of a field, each at the
next multiple of its alignment, with a text field as a pointer.

A C function accepts a handle as a parameter and returns one with no `စိတ်မချရသော`:
`crosses_param` and `crosses_back` accept it. `is_c_handle_result` accepts the result
`<kind> ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` for one, an optional of a handle, where NULL is the
optional with no data. A plain handle result promises that C returns no NULL, and a NULL
there stops the run, as the [code generator](vm-internals.md#c-functions) arranges. `check_rest_arg`
refuses a handle as an other value of a variadic function (ZK0465), and `check_owned`
allows `အပိုင်အနေဖြင့်` on a handle parameter, which is how a C function closes it.

`infer_field` (`list.rs`) gives `h ၏ field` the data kind of the C field. For text, and for
a field of a data kind from outside, that is the optional of the data kind
(`စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`), and a NULL pointer becomes the optional with no
data.

| Use of a handle | Rule | Code |
| --- | --- | --- |
| print | `visit_print` refuses it (`HandleUse::Print`) | ZK0473 |
| duplicate | `infer` refuses it (`HandleUse::Copy`) | ZK0473 |
| `X()` | the literal check refuses it (`HandleUse::Make`) | ZK0473 |
| a write to a field that the C struct has | `place_kind` refuses it (`c_field_write`) | ZK0474 |

### Initial values

`ကနဦးတန်ဖိုး`, the [initial value](../language/kinds.md#initial-values), is a `NullLit` with
`initial` set. `infer_initial` gives it the data kind that its place expects: a
declaration with a written data kind, an assignment, a field, a cell, a return or an
argument. With no expected data kind it is ZK0478.

`DataKind::initial_gap` then tells whether the data kind has one: a data kind has an
initial value when each of its parts has one. The walk stops at an optional, whose initial
value is blank, at a plain list, which starts with no cells, and at a shared struct met
again. A function value, a handle, a result and an enumeration with no marked member have
none, and neither does a data kind that contains one of them (`InitialGap::Part` names the
field or the cells). `ကနဦးတန်ဖိုး` for such a data kind is ZK0479, and its message names the
reason.

A struct literal names each field of its struct: a field left out is ZK0482 (the
constructor `field_left_out`, from `list.rs`), and the message lists the missing fields.
`ကနဦးတန်ဖိုး` as the value of a field gives that field its initial value.

### Literals that get the data kind of their place

An unprefixed literal is *flexible*: `infer` passes down an `expect` data kind from the
declaration, the assignment target, the parameter, the result kind or the other operand,
and the literal uses it.

- An integer literal gets the expected numeric kind, or `isize`. The parser has not
  checked its range, so a value that does not fit is ZK0406 here, and the check still
  records the expected data kind so that nothing else complains. A `-` before the literal
  checks the negative range, so `-၁၂၈` fits `i8`.
- A float literal gets the expected float kind, or `f64`.
- In an operation with two operands where one is flexible, the check infers the fixed side
  first, and its data kind becomes the expectation of the other side. With `က: u8`, the
  `၁` of `က + ၁` is a `u8`. If both sides are flexible and one is a float literal, both
  expect `f64`.
- Bitwise operators and shifts pass an expectation on just when it is an integer kind, so
  a float place does not turn `၁` in `၁ << ၂` into a float.

### ဇယား kinds

A ဇယား (map) is the struct that `DataKind::table` makes:
`%map{%rows:%row{%hash:u64,%live:bool,%index:K,%value:V}[],%slots:u32[],%live:usize}`. As
for the struct of a result, its signature lists its fields in full and never uses a name
in another signature, so two ဇယား kinds in one signature never clash, and a struct that
contains a ဇယား of itself can be read back. No field that a program writes starts with
`%`. `is_table`, `table_row` and `table_parts` read a ဇယား kind, and the name in messages
is `map<K, V>`.

A read of a row is an optional of the value kind, but the base of a place that a statement
writes through has the value kind itself. `place_kind` marks each `row_value` at the base
of its target first (`mark_row_write`), a ဇယား in a row of another included, and the IR
builder reads the data kind it recorded to tell the two apart. Each written data kind is
checked for an index (key) kind that cannot be hashed (ZK0485, `index_gap`), and the check
warns on one with a decimal number (ZK0487).

## Unsafe code, pointers and lists

`Checker::unchecked` is true while the walk is inside unsafe code. `visit_fn` sets it for
a function declared `စိတ်မချရသော`, and a nested function inherits it from the function
around it. `visit_block` adds a block that opens with `UnsafeBlockOpen`
(`BlockStmt.unchecked`) and restores the old value after it. `need_unchecked` reports
ZK0468 outside unsafe code for each operation that needs it:

- `၏ တန်ဖိုး` on a pointer, and pointer arithmetic;
- `x ၏ လိပ်စာ`;
- the heap statements (`HEAP_WORK`);
- a conversion that makes or reads an address;
- a call of an unsafe function, or of a C function declared unsafe.

A pointer value itself can pass through safe code: kept, copied and compared with the null
pointer. Under `--no-unsafe`, each use of `စိတ်မချရသော`, on a function or a block, is
ZK0471. `အပိုင်အနေဖြင့်` on a parameter or a result that is no pointer is ZK0470
(`owned_not_ptr`), though a handle parameter of a C function passes. On the result of a C
function, the marker keeps its older meaning, text that C hands over, and each other
result there is ZK0467. `place_of` finds the place that a target or an address operand
names, for `၏ လိပ်စာ`, `၏ တန်ဖိုး` and list cells.

These codes are errors at each level of the `mem` lint, because the walk decides them
here, from data kinds and the words alone. The rules that need a flow through the program
belong to [the flow checker](borrow-checker.md). `unsafe_code.rs` (`in_unsafe_code`) is a
tool for programs written before unsafe code was marked in words: it puts the pointer work
of such a program into unsafe code, as the tests and the migration of the samples need.

`list.rs` checks lists: the cell kind, the written count, a cell number that is an
integer, and a short-lived list that is not a place. A list literal with no counted list
expected is a plain list. `[]` or `စာရင်းအလွတ်` checked against a counted list is ZK0436 in
each place, the value of a declaration included: a counted list gets each cell from a
literal, from `ကနဦးတန်ဖိုး`, or from writes that the flow checker follows. A constant keeps
the data kind that its declaration gives. `==` and `!=` accept two lists of one data kind,
or two structs or objects of one data kind (`infer_list_pair` in `ops.rs`), and compare
them deep. A counted and a plain list, or two structs of different data kinds, is ZK0404,
and an ordering operator on a struct is ZK0403.

The count, a cell and a range also work on text: a cell of text is a `u8`, and a range is a
new value (`infer_range`), text from text, else a plain list of the cells, whose cells are
duplicates. Since a range duplicates its cells, each value can be its source
(`range_cells`). The assignment check in `semcheck.rs` reports a write to a cell of text as
ZK0429. `infer_chars` gives the character count of text a `usize`, and a character of it
(`CharAt`, `CharNth`) a `char`; an operand that is no text is ZK0403 (`Want::Text`).
`infer_join` checks that each piece of a verbal join is text, and gives the join `text`.

A string literal (`infer_str`) is `text`, whatever the data kinds of its template slots.
The check infers each template slot with no expected data kind, and an error in a template
slot leaves the literal unknown. In `ops.rs`, `+` with text on each side is a join, of
data kind `text`, while each other operator on text, and `+` of text with a number, is
ZK0403.

## Generics

A generic function or struct is a *template*: `collect_fns` and `collect_structs` leave it
out, and the walk skips it (`generic::is_template`). The driver runs the check again while
a run asks for instances (`zakac_semcheck/src/generic.rs`, `generic_struct.rs`,
`traits.rs`).

Generic functions of one name are overloads (`Templates`). The first keeps the written name
as its inner name, each later one gets `<name>#2`, `<name>#3`, … (`overload_name`), and two
with one function signature are ZK0304 (`same_head`). A call picks one
(`choose_template`): the templates whose parameter count and prepositions fit and whose
data kinds the call shows (`fitting_kinds`), narrowed through the parameters of fixed data
kinds when more than one fits (`template_fits`). None is ZK0332, and more than one is
ZK0333. A plain function of the name that fits comes first (`plain_or_generic`), and a
function value picks through its count of data kinds or the function kind that its place
expects (`choose_value_template`).

1. A call of a template works out its data kinds: written, or matched from the argument
   kinds (`infer_kinds`, through `Binder`), then from the data kind that its place expects.
   It checks the constraints of each kind parameter (`meets_bounds`, `has_trait`), names
   the instance (`instance_name`: `f<isize၊ text>`, `f<isize>#2` for a later overload),
   records it in `wanted`, and records its choice in `Chosen` under the top-level function
   it belongs to and the span of the name, since each instance keeps the spans of its
   template. The run checks the call against the instance once the instance exists.
2. Between two runs, `name_instances` (`generic_struct.rs`) gives each data kind that names
   a generic struct with known data kinds the name of its instance, inner data kinds
   first, and records the instance. A struct value with no data kinds asks the checker for
   its instance (`generic_struct_lit`), from the data kind that its place expects or from
   its field values.
3. `add_instances` appends each new instance: a duplicate of the template with kind phrases
   in place of its kind parameters (`kind_ast`, `Kind::substitute`), with fresh node
   numbers. The instance of a function goes right after its template, in its file
   (`place_instance`, which moves the start of each later file), and the instance of a
   struct at the end. While an instance is checked (`open_instance`), it also sees the
   functions of the files of its data kinds and its traits, and a call from it into
   another file gets the name `<name>@<file>` (`name_foreign`), which the IR builder knows
   in each file. A report from an instance gets a label at the first use that asked for it
   (`uses`, `label_instance`).
4. Once a run asks for nothing new, the last run writes the choices back
   (`write_overloads`), and `leftovers` reports each data kind that kept its kind
   parameters. At most 64 runs add instances (ZK0342).

The traits are collected per file (`traits::traits`). A trait block is checked against
its trait (`check_trait_block`): each function signature with the data kind in place, each
function of the block one of the trait, and the file rule. A block that inherits the
defaults (`inheriting`) leaves out each function with a default: `gives` counts the
default for its data kind (`takes_default`) when the data kind provides none, and the call
reaches the default through the generic overloads. A block that inherits no default is
ZK0351. The built-in traits are predicates on the data kind (`traits::builtin`).

## Operators on structs

An operator on two values of one struct that a file declares calls the function that its
trait names, when the file of the use or of the struct has one with those parameter kinds
(`op_calls.rs`). `infer_infix` and `infer_verb` leave the operator in `pending_op`,
`infer_pair` picks it up before the operands, and `struct_operator` records the call. `~`,
the `-` of one operand and the shifts, whose operands are not two values of one struct,
record their calls through `struct_call` (`infer_prefix`, `infer_neg`, `infer_shift`):
one parameter for the first two, and the struct and the unsigned distance for a shift.
`ပြသမည်` and a template slot record a call of `စာသားအဖြစ်ပြောင်း`.

Once the check settles, `write_op_calls` puts each call into the tree, with `!` around a
negated one, fresh node numbers and their data kinds. A name that a file declares, or a
variable, hides a generic of the core module (`templates_named`), a template of a file
hides one of the core module with the same head (`same_head`, `Templates::add_fn`) and
stands beside one with another head, and a trait of a file hides one of the core module. The compiler mark outside the core module is
ZK0349.

## Calls a node stands for

- **Trait values.** `trait_fit` (in `trait_value.rs`), called from `expect_fit`, lets a value
  into a trait value kind when its data kind has the trait (`has_trait`, ZK0343 when not).
  Outside the trial checks of `fits_call` (`Checker::trials`), it records in
  `OpCalls::to_traits` the trait value kind and the inner name of the function that gives
  the data kind each function of the trait (`impl_of`: a plain function of the files of
  `places`, else the instance of the default for the data kind), and ZK0362 for a plain
  value whose trait has a function that changes its first parameter. `write_op_calls` puts
  the call `%trait-value` with those names around the value. A call whose first argument
  is a trait value and whose name is a function of its trait takes the signature
  `dispatch_sig` builds from the trait function, and its name becomes `%trait-call:<n>` in
  `chosen`. `check_trait_values`, called from `written_kind`, checks the trait of each
  written trait value kind: ZK0344, ZK0340 and ZK0360.
- **Parts of a struct.** A base is a field named `%%<struct>` (`BASE_FIELD`), which the
  parser adds. `embed.rs` searches the parts, the nearest first (`search_parts`), and
  writes what each read stands for with nodes numbered past the program's
  (`Checker::next_node`, `fresh_node`): a field of a part, `x ၏ f` for `x ၏ %%B ၏ f`
  (`promoted_field`, into `OpCalls::fields`); a function of a part, `g(x)` for
  `g(x ၏ %%B)` (`promoted_call`, `promoted_method`, into `OpCalls::methods`, tried when
  `has_parts` and no function takes `x`); and the part itself, `x ကို B အဖြစ်
  ပြောင်းလဲသော ရလဒ်` (`part_read`, from `infer_cast` and the read of a member). A struct
  has a trait through a part (`part_gives` in `gives`), and the function of a trait value
  names the path to the part after `PART_PATH`. Two parts at one depth are ZK0364, a base
  that is no struct ZK0363 (`check_parts`), and a literal sets a part under the name of its
  struct (`StructKind::literal_field`).
- **A call through a field.** The parser gives each call on a value, `x ၏ f(…)`, a node
  number for the read of the field `f` (`CallExpr::field_id`). `field_call` (in
  `fn_value.rs`), called first in `infer_call`, makes the call one through that field when
  no function, generic, variable or trait function `f` takes `x` and the field holds a
  function value: the place is the read of the field, the arguments those after `x`, and
  `write_methods` puts the call in place of the old one.
- **`x ၏ f`.** A field read that finds no field, with no brace on the name, checks the call
  `f(x)` instead (`infer_method`). Each records its call in `OpCalls::methods`, and
  `write_methods` puts it in place of the node before the overloads get their inner names.
- **Conversions.** A conversion with no built-in pair, or one that passes a failure on,
  looks for a function of the conversion traits (`conversion_fn` in `op_calls.rs`): a
  `ပြောင်းလဲ` that takes the source data kind and returns the target, or a
  `စမ်းသပ်ပြောင်းလဲ` that returns a result whose good side is the target, in the file of the
  use and the files of the two data kinds. A pass-on takes the second first, a plain
  conversion the first. `infer_conversion` records the call under its inner name, with the
  `@<stem>` of another file, in `OpCalls::methods`. With no such function, a conversion to
  `စာသား` of a value that prints goes into `OpCalls::texts`, and `write_methods` puts the
  template text `"{x}"` in its place (`to_template`), so a struct with
  `စာသားအဖြစ်ပြောင်း` gives that text. `ပြောင်းလဲနိုင်သော` holds for a pair of the table
  through `builtin_pair` in `traits.rs`. With no plain function, `generic_conversion`
  looks at the generic functions of the two names: the kind parameter that the one
  parameter is written with stands for the source data kind and each other one for the
  target (`conversion_kinds`), the constraints must hold in a trial that drops its
  reports (`generic_result`, which also gives the result kind with the kinds in place),
  and the call is checked with those kinds written (`Via::Generic`). The same generic
  functions give a built-in kind `ပြောင်းလဲနိုင်သော` or `စမ်းသပ်ပြောင်းလဲနိုင်သော` in a
  constraint (`builtin_pair`, `generic_converts`). A call that
  writes kinds goes to the generics even when a plain function of the name takes its
  arguments (`plain_or_generic`). The core's `စမ်းသပ်ပြောင်းလဲ` over two integer kinds
  makes `ပြောင်းလဲသော ရလဒ်ကောင်း` between integers fail when the value does not fit.
- **Imports.** A name imported from several files is one overload set (`import_fn`). The
  call of one that another file declares gets the name of that file, `<name>@<stem>`
  (`name_foreign`). A generic function imported beside plain functions of its name gets a
  call that no plain one fits (`plain_or_generic`).

### Threads

The checks of threads (`threads.rs`, with the data kinds in `task.rs`):

| Form | Rule | Code |
| --- | --- | --- |
| a call with `parallel` set | returns `DataKind::Task`, the thread kind of what its function returns (`start_thread`); each argument and the result must have the trait of the data kinds that threads can use (`goes_to_thread`) | ZK0494 |
| the function a thread starts in | `check_thread_callee` finds it from the call's choice: no function value, closure or C function | ZK0497 |
| a mutable or writable parameter (`inout`, `out`) of that function | refused | ZK0498 |
| `စောင့်` of the core module on one thread (`wait_call`) | returns the data kind of the thread's result | |
| `ပို့` or `ပိတ်` whose first argument is a channel | a send or a close (`pipe_call`) | |
| a receive | returns the optional of what the channel carries (`infer_receive`) | |
| `<pipe> ထဲတွင် ဒေတာ ရှိ` | good as the whole condition of an arm alone (`check_arm_cond` sets `pipe_test`); `pipe_part` serves both this and the next row | ZK0496 |
| `<pipe> ထဲမှ ဒေတာ` | good inside that arm alone (`pipe_reads`) | ZK0496 |
| an arm of a wait on several channels | `check_select_arm`: the test of a channel as for `အကယ်၍`, its reads inside its block (`pipe_reads`); the time limit is a `usize`; `check_select_spin` warns on a while loop that is just a wait with the arm of no data and no sleep | ZK0495, ZK0496, ZK0418 |
| a lock block | `check_lock` checks that its name has an object, and keeps the name in `held_locks` while it checks the block | |
| a field of a marked object | `check_locked_field`, from `infer_field`: the field needs a lock around it on its name | ZK0499 |
| an atomic number | used as its integer (`read_kind` in `infer_name` and `infer_field`), needs no lock, and accepts `ပေါင်းထည့်` and `နုတ်ယူ` (`atomic_call`) | |
| a set of an atomic number that reads the number it sets | `check_atomic_set` | ZK0727 |
| an atomic number of a narrow integer kind | `check_atomic_kind` | ZK0401 |
| a parallel map (`infer_par_map`) | needs a function of one parameter that the cells fit, and returns a plain list of its result | |
| the count of threads of a parallel loop or map | `check_thread_count`: an integer, as a bound of a counted loop is | ZK0405 |
| a write in a parallel loop to a variable from outside it | `check_round` walks the block and reports each one except the cell of the round's own number | ZK0728 |

`task.rs` makes the thread, channel and atomic kinds once for each carried data kind, with
the signatures `%thread(<kind>)`, `%pipe(<kind>)` and `%atomic(<kind>)`, and their phrases
come from the written-as hints of the core module. Inside the round of a parallel loop,
`RoundCtx` (`in_round`) keeps the loop. `round_read` reports a variable from outside of a
data kind that threads cannot use, and `round_call` a closure from outside (both ZK0494).
`round_edit_arg` reports an argument that lets a call change a variable from outside
(ZK0728), and a break of the loop itself or a return is ZK0729. A function declared in the
round clears the context.

## Comptime calls

A call of a comptime function (`FnDecl.comptime`, `FnSig.comptime`) in a spot that needs
its value at compile time runs on the VM while the check runs (`src/comptime_calls.rs`).

- `check_call` notes the comptime function that each call picks (`Checker::comptime_sigs`,
  on the span of the call). The evaluator of constants (`folded`, `work_out` in
  `consts.rs`) meets the call in a constant, a count, the starting value of a variable of
  an imported file (`visit_decl`), a member value or a case. It folds each argument to a
  `ConstTree`, records a `Site` and returns `Stop::Pending`, as an open count does. An
  argument that does not fold is ZK0608.
- After each run of the check, `SiteState::run` splits the sites. `Reach` has the uses
  between the top-level statements (from the `RefTable`). `Reach::kept` drops each
  statement that contains an open site, each global that the functions do not reach (its
  starting value would run), the program start block, and each statement that reaches one
  of these, and keeps the rest, so an operator on a struct or a trait default finds its
  function with no name recorded. A site runs once its function is kept. The sites of one
  file go to the installed `ComptimeRunner` as one `SmallProgram`: the files up to the file
  of the function, as the file that is run, the statements kept, and a global `%ctN` for
  each site, set to the call with its arguments as values.
- `write_values` puts an `Expr::Comptime` in place of each call that ran, keeping its node
  number: the value as a `ConstTree`, and the result kind written back from the data kind
  the call produced (`written_kind`), so an instance of a generic writes its own. The
  check then runs again, and later stages treat the node as a literal (`tree_expr` in the
  IR builder).
- The runner returns a `Ran` for each site: its value or its ZK0611, and a ZK0612 note of
  what it printed. The rights of comptime runs live beside the runner (`set_grants`, from
  the `--comptime-allow-*` flags). A call whose arguments still wait after the last run is
  ZK0609 (`blocked`).
- The last run reports each site that stopped (ZK0611, from the runner), each one still
  waiting (ZK0609) and each one with no runner installed (ZK0610). ZK0489, from
  `collect_fns`, checks the result kind of each comptime function.
- `open_chains::settled` reads a program in passes until no compile-time conditional is
  open. Each pass puts each open condition in a constant `%condN` of a duplicate of the
  program, checks the duplicate, settles the condition with
  `zakac_parser::comptime::truth`, and reads the program again with the truths. A
  condition whose faults are unknown names waits while another chain is open, and a
  function that its own chain declares is ZK0609. `load_settled`, `single` and
  `single_for` wrap it for the commands and the `run_phases` of each stage.

## Codes

| Range | What it covers |
| --- | --- |
| ZK03 | names: undeclared or doubled names, constants and loop variables assigned, `ကွင်းဆက်` outside a loop, return outside a function, argument counts |
| ZK04 | data kinds: mismatches, conditions that are not `bool`, operands of the wrong data kind, literals that do not fit, shifts that go too far, bad conversions, bad use of pointers and lists |
| ZK0339 | a function with a result whose block can reach its end with no return (`returns_on_each_path`) |
| ZK0443 to ZK0449 | failing functions: a call statement that drops a failure (ZK0443), a pass-on with no failure to pass (ZK0444), a pass-on in a function that cannot fail (ZK0445), another error kind (ZK0446, where an error side with no value is a data kind too), an error value from a function that cannot fail (ZK0447), a test or a read of a value that is no result (ZK0448), a good result of a function that has none (ZK0449) |
| ZK0450 and ZK0451 | the sides with no value: a read of the error value of a result whose error side has none (ZK0450), `လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်` in a function whose error side names a data kind (ZK0451); an error value where the error side has none is ZK0476 |
| ZK0454 | a read of a part ruled out (below) |
| ZK0468 to ZK0472 | unsafe code: an unsafe operation or call outside it, an unsafe function as a value, an owned marker on no pointer, unsafe code under `--no-unsafe`, a C function that needs `စိတ်မချရသော` |
| ZK0473 and ZK0474 | data kinds from outside the program: a handle made, duplicated or printed; a write to a field that C keeps |
| ZK0475 to ZK0477 | optionals and error values: a test word that does not fit the data kind (ZK0475), `<e> ဟူသော အမှားအယွင်း` where no result kind with an error side is wanted (ZK0476), `အလွတ်` where no optional is wanted (ZK0477) |
| ZK05 | wording warnings |

For ZK0454, `visit_if`, `visit_while` and the other loop visitors keep the set of result
variables proven to have a good result, a failure, data or no data, built from the tests
with `facts_when` in `proof.rs`. A read of the part ruled out is ZK0454. A guard arm proves
the opposite after the statement. An assignment, an argument for a mutable parameter, the
address of the variable or a new declaration of its name ends the proof, and a loop that
can change the variable ends it for the whole loop. ZK0455, the return word of the other
sort of function, is gone with the old return words.

Each code has a constructor in `zakac_diagnostics` (`resolve.rs`, `kinds.rs`,
`naturalness.rs`, `ownership.rs`), and [Error messages](../reference/errors.md) lists them
all.

## Tests and extending

`tests/common/mod.rs` has these helpers:

| Helper | What it does |
| --- | --- |
| `check`, `check_with` | puts the text in a program start block, then lexes, parses and checks it |
| `check_as_is`, `codes_as_is` | the same with no program start block added |
| `codes` | the diagnostic codes of a check |
| `ok` | asserts a check with no diagnostics |
| `kind_at(checked, text)` | the data kind recorded for the first expression whose source text is `text` |

Each test file of `tests/` covers one area: `kind.rs` (each widening row, the conversion
table), `resolve.rs` (each ZK03 code, scopes, calls before declarations), `kinds.rs` (each
ZK04 code, flexible literals), `lists.rs`, `text.rs`, `structs.rs`, `objects.rs`,
`pointers.rs`, `unsafe.rs`, `external.rs`, `callbacks.rs`, `nested.rs`, `edits.rs`,
`prepositions.rs`, `overloads.rs`, `generics.rs`, `optionals.rs`, `faults.rs`,
`initial.rs`, `counts.rs`, `start.rs`, `exports.rs`, `threads.rs`, `artifact.rs`,
`cli.rs`, and `samples.rs` (golden `.zksemt` files). `modules.rs` checks the rules across
files, with programs kept in a `MemFileLoader`.

- **A statement form**: override its `visit_*` method in `semcheck.rs`, and infer each
  expression once, with `expect_kind` or `expect_bool` when the place fixes a data kind.
- **An operator**: give it an `OpClass` in `ops::infix_class` or `ops::verb_class`.
- **A wording rule**: add a row in `RULES`, a `ZK05` code and a constructor. If the rule
  needs context that the tree lacks, add it in the parser (a `NextCtx` form or a field).
- **A message**: add a row in `define_codes!`, the texts in `locales/codes.toml`, and a
  constructor.

The checked tree and its kind table go on to [The IR](codegen.md).
