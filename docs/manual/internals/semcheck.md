---
title: Names and kinds
description: What the name and kind check does after parsing, with the passes in order, the tables it builds, the wording check that runs before it, and the codes it raises.
---

# Names and kinds

After the parser comes a stage that decides whether the tree makes sense. It checks that
each name is declared where it is used, that `ကွင်းဆက်` sits in a loop and a return in a
function, and that each value has a data kind that fits its place. It does not change the
tree. Its product is a table of the kind of each expression, plus diagnostics. The code
is in `compiler/zakac_semcheck`, and the wording check it runs first is in
`compiler/zakac_naturalness`.

Constant folding is not here. It works on the IR and is told in [IR and code](codegen.md).

## The driver

Each stage crate has a `run_phases` function that runs the stages before it. For this
one, `zakac_semcheck::artifact::run_phases` does the whole front end:

```text
 lex ──▶ parse ──┬─ a lexer or parser error? ──▶ stop: no tree check
                 │
                 ├──▶ zakac_naturalness::check ──▶ warnings ZK05 ┐
                 └──▶ zakac_semcheck::semcheck ──▶ kinds, ZK03 ZK04, ZK0705 ZK0713..17 ┤
                                                                  all diagnostics ◀──┘
```

The two checks are skipped when the lexer or the parser reported an error, so one broken
sentence does not start a chain of follow-on errors. The result is `Checked { program,
kinds, diagnostics }`: the tree, the kind table, and one list of diagnostics, the
lexer's first, then the parser's, then the wording warnings, then the name and kind
errors.

`semcheck` takes the program as `&mut Program` and works out each list count written as
an expression (`consts.rs`). Where a kind is read (`written_kind`, and struct fields in
`visit_struct`), `work_out_counts` checks the count as an expression and works it out with
the folder's own math: `value.rs` and the operator enums of `const_op.rs` live in this
crate, and the IR builder takes them from here. `work_out` follows lowering: each operand
in its recorded kind, converted where lowering converts it, so literals, constants,
operators in both spellings, comparisons, conversions and `၏ အရွယ်အစား` give the value the
folder would; a constant list or struct is held as its items (`Folded`, kept in
`Checker::aggs` while a run lasts) when each item is known, as the folder's `is_known`
asks, so a cell, a field or the count of a plain list reads it (`cell_place` reports
ZK0603), and the count of a counted list comes from its kind, as lowering takes it. The
test `counts_agree_with_the_values_the_program_computes` (`zaka_vm/tests/values.rs`) gives
each form to a constant and to a count and checks the two agree, so the two walks keep in
step; and an overflow (ZK0601), a division with zero (ZK0602) or a shift too far
(ZK0407) is reported as the folder reports it. Each constant declaration keeps its value
in its `Binding` (`ConstVal`). A size that hangs on a count still open waits
(`Stop::Pending`, `size_waits`, `open_structs`), so no count is fixed from a stand-in. A
run records each count (`CountTable`) and the top-level constants (`KnownConsts`);
`write_counts` writes each worked-out count back as `Cells::Fixed`, and `semcheck` runs
again while a run writes a count or finds new constants, so a struct can name a constant
declared below it and one count can wait on another. The last run gives the output, and
a count still open is ZK0227 (a count that waits on itself among them). A program with no
count expression is checked once.

`SemcheckArtifact` is the reloadable form (text header `# zakac-semcheck-artifact v1`,
files `.zksemt` and `.zksem`). The command takes `--platform-bits` (8 to 128, default 64),
`--strict-particles` and `--no-raw`; these become `SemcheckOptions { platform_bits,
strict_particles, no_raw }`, and the later stages receive the same options.

## The passes, in order

### 1. Wording (`zakac_naturalness`)

A visitor over the tree. The parser accepts each particle on a verb, and either `ကို` or
`ဟု` after a printed operand; this pass names which wording is natural, and reports the
rest as warnings. A warning never fails a build, and `--strict-particles` raises them to
errors. The pass reads just the tree: the `particle` and `next` fields the parser kept on
verbs, boolean words and `မဟုတ်`, the `slot` of a print, and the `using` mark of a call
written arguments first (`ကို` or `တို့ကို` before `သုံး၍`).

`site.rs` turns a node into a `Site` (a clause-final word, a group in parentheses, a
value, a print operand, the particle before `သုံး၍`). `rules.rs` is a table `RULES` of `{ code, fires }`; the first row
that fires at a site gives the code, so order matters when two rules overlap.

| Code | Fires when | Wants |
| --- | --- | --- |
| ZK0501 | a negative word, then `သို့မဟုတ်`, with `သည်` | `ပါ` |
| ZK0502 | a particle before `လျှင်` or `နေသရွေ့` | none |
| ZK0503 | `မဟုတ်` with `သည်` or `ပါသည်` | `ပါ` or none |
| ZK0504 | `ပါ` or `ပါသည်` before `ဖြစ်၍` | `သည်` |
| ZK0505 | a sentence or a string printed with `ကို` | `ဟု` |
| ZK0506 | a plain value printed with `ဟု` | `ကို` |
| ZK0507 | parentheses that add nothing | none |
| ZK0508 | a clause that gives a value ends in `ပါ` or `ပါသည်` | `သည်` |
| ZK0510 | `တို့ကို` before `သုံး၍` after one argument, or `ကို` after several | `ကို` for one, `တို့ကို` for several |

A run of `"မင်္ဂလာပါ" ကို ပြသပါ။` gives `warning[ZK0505]` with the help `write "... ဟု
ပြသပါ။"`. The natural wording is a matter of the language, and is told in
[Natural wording](../language/naturalness.md).

### 2. Function signatures (`collect_fns`)

A first walk over the top-level statements records the signature of each function (the
kinds of its parameters, of its result and of its error value, and its export mark), so a call can come before
the declaration. The table holds one map for each file of the program: the first
declaration of a name in a file wins, and a second in the same file is ZK0304. Two files
can each hold a function of one name. A signature keeps the mark of each parameter
(`FnSig::marks`, `None` with no marks); `check_marks` compares the marks of a marked call
with it, one argument after another, after the count check (ZK0314), and reports another word
as ZK0331.

A file can declare several functions of one name: `FnSigs` maps a name to its overloads in
the order of their declarations, and the same parameter kinds and marks twice are ZK0304.
Each signature carries its inner name (`FnSig::inner`, from `overload_name`): the written
name for the first, `<name>#<n>` for the n-th. `check_call` runs `choose`: with more than
one function of the name, `fits_call` keeps the ones whose count, marks (when the call
writes marks) and parameter kinds fit, with a trial of `expect_kind` on each argument that puts
the diagnostics back; one fit is recorded in `Checker::chosen` under the span of the name
at the call, none is ZK0332, more is ZK0333. After the last run, `write_overloads`
(`overloads.rs`) writes the inner names into the program: each later declaration of a name,
each call in `chosen`, and the inner names into each import of an overloaded name. The IR
builder, the ownership check and codegen then see one function for each name. An import
brings each exported overload (`import_fn`); a name a file declares and imports is ZK0304.

An external function (`FnDecl::external`) must be one of the functions the host serves:
`check_external` asks `host::find`, a thin adapter over the table `HOST_FNS` of
`zaka_host`, for the row with its name, parameter kinds and result kind (a result that can
fail with `IO_ERROR`), and reports ZK0334 with `host::listing` as help when none matches.
The table matches kinds through their names, so the stream structs and the error
enumeration are the ones the stdlib declares. The IR builder asks `host::find` again for
the host function of its one statement, with the written name of an overload (`#<n>` cut
off).

A C function (`FnDecl::lib` is set) goes to `check_c_fn` instead, which asks `foreign.rs`:
`is_c_name` (ASCII letters, digits and `_`, no digit first: ZK0337), `crosses` for each
parameter and `crosses_back` for the result (ZK0465; the kinds C takes are integers of 8 to
64 bits, `f32`, `f64`, `bool`, borrowed and raw addresses, and structs and counted lists of
these; the owner `လိပ်စာ` is refused, since the wrapper would free it, and an enumeration
is left out, since Zaka holds it as an `isize` and a C enum is an `int`), and the edit mark
on a parameter (ZK0466). `Checker::c_names` maps each C name to its parameter and result
kinds, so a second declaration of the name, in another file or among overloads, with
other kinds is ZK0338. Its address is refused as for each external function (ZK0464).

A parameter of a C function can be text too (`crosses_param`: `crosses` or a textual
kind); a result cannot be (`crosses_back`, ZK0465). A header that ends in `နှင့်
အခြားတန်ဖိုးများ` sets `FnSig::variadic`: a call needs at least the named arguments (ZK0314
for fewer), and `check_rest_arg` infers each other value with no expected kind and asks
`crosses_rest` (a parameter kind but a struct, else ZK0465). `check_external` gives a host
function that takes other values ZK0334, since the host table has none.

Before it, `collect_enums` (`enum_check.rs`) makes the kind of each enumeration, on its
name and on its stem, and reports a member name used twice (ZK0328) and a name a struct
or another enumeration has (ZK0316). A member value can name a constant, so the walk works
the values out (`check_enum`, once the walk has the top-level names of the file): the
first value gives the value kind (`isize` with none), each other value converts to it
(`expect_kind`) and folds (`constant_of`: `coerced` for a scalar, `folded` for text, a list or
a struct, with `Folded::Text` for constant text), and a member with none counts on (`count_on`, ZK0461). A value
that does not work out is ZK0456, and one two members share ZK0457. The values go into
`found_enums`; until a run has them, an enumeration with a value written stands as `isize`
members numbered from 0, and `semcheck` runs the check again while the values change,
as it does for list counts. `collect_structs` reads a field of an enumeration kind
through that table, and the enumerations then join the struct table, so each place that
reads a named kind finds both.

The same first step finds the file of each struct and enumeration (`struct_homes`) and checks each
exported declaration with `private_in_exports`: a kind of an exported function,
variable, constant or struct that names a struct its file keeps private is ZK0324.

### 3. The walk (`Checker`)

`semcheck(program, options)` pushes the global scope and visits the tree with a `Checker`
that implements `Visitor`. It has one method for each statement form and calls
`infer(expr, expect)` for each expression.

| Statement | Checks |
| --- | --- |
| declaration | a constant has a value; the value converts to the kind, or gives it when none is written (`ဗလာ`, which names no kind, is ZK0430); one declaration of a name in a scope. The name is declared after the value is checked |
| assignment | the target is a place, declared, not a constant or a counted-loop variable; the value converts to the kind of the place |
| block, if arm | a block is a scope of its own; a condition is `bool` |
| while, generic loop | a `bool` condition; the loop depth grows with one |
| each loop | a list, a slice or a pointer to a list (`each_list`) whose cells hold no owner pointer for an each-cell loop, text or a view of text for an each-character loop, else ZK0434; the loop variable takes the cell kind or `char` |
| counted loop | bounds and step are integers; the loop variable takes the common kind of the bounds; a literal step of zero or less is ZK0409 |
| `ကွင်းဆက်`, `နောက်တစ်ကြိမ်ဆီသို့` | the loop depth is above zero in the current function |
| function | parameters are distinct; one scope holds the parameters, `ရလဒ်` (just when the function has a result) and the block; the loop depth restarts at zero |
| return | in a function; a value just when the function has a result, and it converts to the result kind |
| return of an error value | `ReturnStmt::fault`: the function can fail (else ZK0447) and the value converts to its error kind (ZK0401) |
| call that can fail | a call statement to a function with an error kind passes the failure on (else ZK0443); an expression call with no pass-on is a whole result, of the result class `%fallible`, built with `DataKind::fallible(ok, err)`, each side optional (a side with no value has no field; a message writes `isize result or text error`, `success or text error` and `isize result or failure`); a written result kind becomes the same class, in a declaration, a parameter, a struct field (the sugar is resolved in `structs.rs`, where the field kinds are read) and a result clause; it is no operand (ZK0403) and fits just a place of the same kinds; a pass-on needs a callee that can fail (ZK0444) and a function around it that can (ZK0445, also in the start block, which cannot fail, and in top-level statements) with the same error kind (ZK0446); the good result of a callee with none is ZK0315 |
| call | the function exists; one argument for each parameter; each converts. For a parameter marked `လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သော`, the argument takes the mark (ZK0431; a mark elsewhere is ZK0432), is a writable place (`place_kind`: ZK0410, ZK0306) of the parameter kind, with no widening (ZK0401), and its variable appears in no other argument (`check_edits_apart`, ZK0433) |
| heap sentences | the target is a place of a pointer kind (ZK0417); the size is `usize` |
| blit, fill | `visit_blit`, `visit_fill`: each address is a value of a pointer kind, not a place (ZK0417); the count is `usize` and the fill value is of the unit kind, or a `u8` for a size in octets (ZK0401); `owned_cells` refuses cells of the unit kind, or for octets of the kind each address points at, that hold an owner, text, a plain list or a borrow (ZK0442) |
| assignment to a cell of text | ZK0429, for owned text and a view alike |
| assignment to a character of text | `char_write`, when `char_target` finds `၏ စာလုံး n` or `၏ n ခုမြောက်စာလုံး` as the target: the text is a writable place (`place_kind`), the number an integer (ZK0423) and the value a `char` (ZK0401); `changeable_text` reports a view of text (ZK0438), and a target that is no text is ZK0403 |
| add, remove | `growing_cells` (from `visit_add` and `visit_remove`): the list is a writable place (`place_kind`: ZK0410, ZK0306) of a plain list kind, else ZK0437 (a counted list, a slice, or a pointer to a list, and `နောက်ဆုံးအကွက်` on text); an add in front of a cell (`insert`) checks the list of the cell so and its number (`cell_number`, ZK0423), an add in front of a character checks the text (`is_text_place`, ZK0403; ZK0438) and goes on as an add to text, and `၏ အရှေ့တွင်` after each other value is ZK0439; a remove of the last character (`shrinking_text`) takes text that can change (ZK0403, ZK0438); the value of an add converts to the cell kind (ZK0401). An add to text goes to `text_add` instead: `changeable_text` reports a view (ZK0438), and the value is text, a view of text or a `char`, else ZK0401 ("expected text") |
| switch | `visit_switch`: the subject is inferred once; each case of each arm goes through `check_case` (`switch_check.rs`): over an enumeration, a bare name that is a member (`member_case`) takes the member before a variable, and a name that is neither is ZK0329; each other case is inferred with the subject kind as its expected kind and checked as `==` checks its sides (`pair_result`). A case is known as a member position over an enumeration and as a folded value otherwise (`CaseHit`); two cases known as one are ZK0460. `check_default` checks the kind of the default arm (ZK0459), or with none, that the cases over an enumeration name each member (ZK0458). With no subject kind, the cases are not checked |
| member | `infer_member`: the name before `ထဲမှ` is the stem of an enumeration and `များ` (ZK0330, with help when an enumeration is written without its `များ`); a member it lacks is ZK0329 |
| replace | `visit_replace`: the text is a place (`place_kind`) of text (`is_text_place`, ZK0403), both ends are integers (`cell_number`, ZK0423), and the rest goes as an add to text (`text_add`: ZK0438 for a view, ZK0401 for a value that is no text, view or `char`) |

Each subtree is inferred once, so a diagnostic is never repeated. When an operand has an
error, its operation returns `None` and reports nothing more; an operation with two bad
operands reports the first. The walk goes on past an error, so one run can report several:

```text
error[ZK0406]: the literal 200 does not fit i8
 --> bad2.zaka:1:54
error[ZK0301]: "အမည်" is not declared
 --> bad2.zaka:2:1
error[ZK0401]: expected bool, found isize
 --> bad2.zaka:3:52
```

At the end the diagnostics are sorted in source order.

### Modules

The walk goes over the statements of each file in turn, in the order the loader put
them, and `visit_top` moves to the file of each top-level statement first. Moving to a
new file keeps the top-level scope of the file that ended in `saved`, opens a blank one,
and sets `fns` to the functions of the new file. A statement of an imported file that is
not an import, a function, a struct or a declaration is ZK0323, and so is a start block
there.

`start_rules` looks at the file that is run before the walk: a second start block is
ZK0325, each top-level statement that runs is ZK0326, and a file with no start block is
ZK0327, reported at its start.
`collect_fns` leaves the start block out of the function map, so no call finds it, and
`visit_fn` checks it as a function, then checks its header with `check_start_header`: up
to two parameters, each a plain `စာသား စာရင်း` the function cannot change (ZK0440), and
no result or an integer one: a result kind is ZK0441.

An import runs at its place, at the top of the file. For each name it looks in the file
it reads for an exported function (from that file's function map), an exported variable
or constant (from its saved scope, where the declaration recorded the mark), and an
exported struct. Each one found joins the current file: the function goes into `fns` and
the variable into the top-level scope, without the mark, so it is not exported again.
A name found nowhere is ZK0322, with help when the file declares it without the mark. A
name the file declares as well is the usual ZK0302 or ZK0304, since the import declares it.

Struct names form one table for the whole program, so two files with a struct of one
name are ZK0316. The parser keeps a struct name of another file from starting a kind
unless the file imports it. When a name is unknown (ZK0301, ZK0303), `with_hidden`
looks for it in the other files and adds help that names the file that declares it.

## The data it builds

**`DataKind`** is `Copy` and has twelve forms:

| Form | Holds |
| --- | --- |
| `Int(IntKind)` | a sign and a `Width`, which is `Platform` or `Bits(n)` |
| `Float(bits)` | 16, 32, 64 or 128 |
| `Char`, `Bool` | |
| `Ptr(PtrKind)` | levels of address over a base, with the outer level an owner, a borrow or raw |
| `Null` | the kind of `ဗလာ`, which converts to each pointer kind |
| `List(&'static ListKind)` | a counted list: a count of cells of one kind, inline; list kinds are interned so the form remains `Copy` |
| `Seq(&'static SeqKind)` | a plain list or text, three words (address, count, room); `SeqKind { cell, text }`: `text` marks owned UTF-8 text, `DataKind::TEXT`, whose cells are `u8`, as `SliceKind::text` marks the view. Plain lists are interned through `seq_of`. `is_text` reads the flag, and `as_seq`, `is_seq` and `cell_kind` see plain lists alone, so each rule of text remains apart from the rules of lists; `holds_seq` holds for a plain list and for each counted list or struct with one inside; `holds_text` holds for text and plain lists and for each list or struct with one inside, so each is dropped, duplicated and lent alike; copied, not moved (`is_move_kind` is `holds_owner_ptr`) |
| `Struct(&'static StructKind)` | a struct with its named fields, interned as list kinds are |
| `Slice(&'static SliceKind)` | a borrowed run of cells of one kind, with its count known at run time; `is_borrow` holds for it. `SliceKind { cell, text }`: `text` marks the view of text, `text[..]`, whose cells are `u8` |
| `Enum(&'static EnumKind)` | an enumeration: its name, its value kind and its members, each with its `MemberValue` (a scalar `Value`, text, or the items of a list or a struct; `enums.rs`), interned on the signature `name<member=value\|...;kind>`, which names it in artifacts (each value hex-coded). Storage holds the value of the member, so the size, the alignment and the `holds_*` tests are those of the value kind (`repr` takes it off); it widens to nothing, converts to its value kind and on from a scalar one, and is copied |

A written kind becomes a `DataKind` with defaults filled in: no sign word is signed, no
bit count is platform size for an integer and 64 for a float. So the plain
`ကိန်းပြည့်` is `isize` and the plain `ဒဿမကိန်း` is `f64`. `isize` and `i64` are different
kinds, even on a 64-bit platform, and neither widens to the other.

**`Scopes`** is a stack of maps from a name to a `Binding { kind, access, span }`, where
`Access` is `Var`, `Const` or `LoopVar`. A lookup runs from the innermost scope out. A
`kind` of `None` means an error left it unknown, which silences follow-on messages.

**`FnSigs`** maps a function name to `FnSig { params, ret }`.

**`KindTable`** is `BTreeMap<NodeId, DataKind>`, the product of the stage. The id is the
`NodeId` that the parser gave each expression. For this program:

```text
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
အသက် ကို အသက် နှင့် ၁ ၏ ပေါင်းလဒ် ဟု ထားပါ။
အကယ်၍ အသက် သည် ၁၈ ထက် ကြီးလျှင် အသက် ကို ပြသပါ။
```

`zakac semcheck` writes the tree again and then:

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
diagnostics: []
```

Id 0 is the literal `၂၀`; ids 1 to 4 are the target, the two operands and the sum of the
second sentence; ids 5 and 6 are the operands of the condition and id 7 the comparison
(`bool`); id 8 is the printed name. Each kind is `isize` because the literals had no
kind prefix and took the kind of their place.

## Kinds and conversion

An implicit conversion (`widens_to`) never loses data:

| From | To | When |
| --- | --- | --- |
| integer | integer | the target is wider and no sign is lost (an unsigned kind can go to a wider signed one) |
| integer | float | the integer width is at most the significand of the float: 11, 24, 53, 113 bits for 16, 32, 64, 128 |
| float | float | the target is wider |
| `ဗလာ` | pointer, function kind | in each case |
| function kind | function kind | the same parameter kinds, edit marks and result, and the same marks when the target has marks |
| pointer | pointer | the same flavour (owner, borrow or raw) and the same shape, or one is the opaque `လိပ်စာ`; a block of addresses read as other data needs a raw step |
| counted list | plain list of the same cell kind | when the counted list has cells; lowering moves the cells to a heap block (`to_heap`) |
| plain list | counted list | never: the count of a counted list is part of its kind (ZK0401) |
| `text[..]` | `u8[][..]` | never: a write through the slice would break the text; nor the reverse, since a slice of u8s is not known to hold UTF-8 |
| char, bool, list, text | other kinds | never; owned text does not become a view without a lend |

The conversion verb (`cast_allowed`) is wider: number to number, char to integer and back,
bool to integer, address to address, address to integer, and integer to a raw address.
An integer never becomes an owner or a borrow straight off; the raw address between is
the step a reader can search for.

### Function values

`DataKind::Fn(&'static FnKind)` is a function kind (`fn_kind.rs`): the kind, the edit mark
and the mark of each parameter, and the result. Each kind is made once (`FnKind::make`),
as a list kind is. `FnKind::fits` takes a function into a place of a kind with the same
parameter kinds, edit marks and result, and with the same marks when the place names
marks: a kind with none takes a function with marks or without. `ဗလာ` widens to a function
kind, and the default of one is the zero word. The short name is
`fn(<params>) -> <ret>`, each parameter `[mut ]<kind>[(<mark>)]`; inside another name it
stands in parentheses (`spell_inner`), so `split_marks` and the struct signatures
(`split_top`, `bracket_step`, which skips the `>` of `->`) read it back. A struct field of a
function kind writes its parts with each struct as its bare name (`field_text`,
`collect_kind`, `expand`, `kind_of`), as each other field does.

`fn_value.rs` holds the calls. `call_target` finds what a call runs: the declared
functions of its name; else a variable, a parameter or a constant of a function kind, whose
kind gives one signature (`value_target`); for a place, the kind of the place. A name that
finds both a function and a value of a function kind is ZK0335, and a value of another kind
ZK0462. `check_pass` takes the result kind from the signature, so a call through a value
passes a failure on as a direct call does. `infer_fn_addr` gives the function kind of
`လုပ်ဆောင်ချက် NAME ၏ လိပ်စာ` (`sig_kind`); with overloads it keeps the one whose kind fits the
place (ZK0463 when none or several do) and records it in `chosen`, so `write_overloads`
gives the address its inner name. An external function has no address (ZK0464). `==` and
`!=` take two function values one kind takes, or one with `ဗလာ` (`fn_pair`).

### Literals that take the kind of their place

An unprefixed literal is *flexible*: `infer` passes down an `expect` kind from the
declaration, the assignment target, the parameter, the result kind or the other operand,
and the literal takes it.

- An integer literal takes the expected numeric kind, or `isize`. The parser has not
  checked its range, so a value that does not fit is ZK0406 here; the expected kind is
  still recorded so nothing else complains. A `-` before the literal checks the negative
  range, so `-၁၂၈` fits `i8`.
- A float literal takes the expected float kind, or `f64`.
- In an operation with two operands where one is flexible, the fixed side is inferred
  first and its kind becomes the expectation of the other. With `က: u8`, the `၁` of
  `က + ၁` is a `u8`. If both sides are flexible and one is a float literal, both expect
  `f64`.
- Bitwise operators and shifts pass an expectation on just when it is an integer kind, so
  a float place does not turn `၁` in `၁ << ၂` into a float.

## Pointers and lists

The same walk checks the owner, borrow and raw kinds (see
[Ownership and borrowing](../language/ownership.md)). A kind keeps its flavour inside `PtrKind`, and
`expect_fit` names a mismatch of flavour on its own: a borrow given for an owner (ZK0714)
or an owner for a borrow (ZK0715). `place_of` finds the place that a target or an address
operand names, for `၏ လိပ်စာ`, `၏ တန်ဖိုး` and list cells. Taking the address of a borrow
is ZK0705, a marker `ငှားထားသော` in a wrong place is ZK0713, a lend of a raw address is
ZK0716, and `--no-raw` makes each kind with a raw level ZK0717.

These codes are errors in each mode of the ownership check: these are decided here, from
kinds alone, and `--mem-check` never sees them. The rules that need a flow through the
program belong to [the borrow checker](borrow-checker.md).

Lists are checked in `list.rs`: the cell kind, the written count, a cell number that is an
integer, and a short-lived list that is not a place. A list literal with no counted list
expected is a plain list. `[]` checked against a counted list is
allowed in the value of a declaration alone, where each cell starts at its default; in
each other place it is ZK0436. A constant keeps the kind its declaration gives. `==` and
`!=` take two lists of one kind, or two structs of one kind (`infer_list_pair` in
`ops.rs`); a counted and a plain list, or two structs of different kinds, is ZK0404, and
an ordering operator on a struct is ZK0403.
The count, a cell, a range and the lend also take text, owned or a view: a cell of text
is a `u8`, and a range or a lend of text is a `text[..]`. The assignment check in
`semcheck.rs` reports a write to a cell of text as ZK0429. A range or a lend of text needs
a place but not a variable (`viewable`): a view never writes, so a text constant can be
viewed, where a list constant cannot (ZK0412). `infer_chars` gives the character count of
text a `usize` and a character of it (`CharAt`, `CharNth`) a `char`; an operand that is no
text is ZK0403 (`Want::Text`). `cast_allowed` lets a slice of `u8` convert to `text[..]`, the
one conversion between slices; the VM checks that its u8s are UTF-8. `infer_join` checks
that each piece of a verbal join is text and gives the join `text`.

A string literal (`infer_str`) is `text`, whatever the kinds of its template slots; each template slot is
inferred with no expected kind, and an error in a template slot leaves the literal unknown. In
`ops.rs`, `+` with a textual operand on each side (`is_textual`: `text` or `text[..]`) is a
join, of kind `text`; each other operator on text, and `+` of text with a number, is
ZK0403.

## Codes

| Range | Stage |
| --- | --- |
| ZK03 | names: undeclared or doubled names, constants and loop variables assigned, `ကွင်းဆက်` outside a loop, return outside a function, argument counts |
| ZK04 | kinds: mismatches, conditions that are not `bool`, operands of the wrong kind, literals that do not fit, shifts that go too far, bad conversions, bad pointer and list use |
| ZK0443 to ZK0449 | failing functions: a call statement that drops a failure, a pass-on with no failure to pass, a pass-on in a function that cannot fail, another error kind (ZK0446, where an error side with no value is a kind of its own), an error value from a function that cannot fail, a test or a conversion of a value that is no result (ZK0448), a good result of a function that has none (ZK0449) |
| ZK0450 to ZK0452 | the sides with no value: a read of the error value of a result whose error side holds none (ZK0450), `လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်` in a function whose error side names a kind (ZK0451), an error value given back where the error side holds none (ZK0452) |
| ZK0454 and ZK0455 | `visit_if`, `visit_while` and the other loop visitors keep the set of result variables proven to hold a good result or a failure, built from the tests with `facts_when` in `proof.rs`, and a conversion to the part ruled out is ZK0454 (a guard arm proves the opposite after the statement, and an assignment, an edit-marked argument, the address of the variable or a new declaration of its name ends the proof; a loop that can change the variable ends it for the whole loop); `visit_return` reports the return word of the other sort of function, `ရလဒ်အဖြစ်` in a function that can fail or `ရလဒ်ကောင်းအဖြစ်` in one that cannot (ZK0455) |
| ZK05 | wording warnings |
| ZK0705, ZK0713 to ZK0717 | owner and borrow kind rules |

Each code has a constructor in `zakac_diagnostics` (`resolve.rs`, `kinds.rs`,
`naturalness.rs`, `ownership.rs`). See [Error messages](../reference/errors.md) for the list.

`tests/modules.rs` checks the rules across files, with programs held in a
`MemFileLoader`.

## Tests and extending

`tests/common/mod.rs` has `check`, `codes`, `ok` and `kind_at(checked, text)`, which gives
the kind recorded for the first expression whose source text is `text`. The files are
`kind.rs` (each widening row, the conversion table), `resolve.rs` (each ZK03 code,
scopes, calls before declarations), `kinds.rs` (each ZK04 code, flexible literals),
`lists.rs`, `pointers.rs`, `ownership.rs`, and `samples.rs` (golden `.zksemt` files).

- **A statement form**: override its `visit_*` method in `semcheck.rs`; infer each
  expression once, with `expect_kind` or `expect_bool` when the place fixes a kind.
- **An operator**: give it an `OpClass` in `ops::infix_class` or `ops::verb_class`.
- **A wording rule**: a row in `RULES`, a `ZK05` code, a constructor; if the rule needs
  context the tree lacks, add it in the parser (a `NextCtx` form or a field).
- **A message**: a row in `define_codes!`, texts in `locales/codes.toml`, a constructor.

The checked tree and its kind table go on to [IR and code](codegen.md).
