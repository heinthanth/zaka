---
title: The VM target
description: How zakac_codegen_target_vm turns the folded IR into operations for the VM, the rules for each statement, the listing, and the artifacts of each stage.
---

# The VM target

`zakac_codegen_target_vm` turns the folded and checked IR into *operations*
(instructions) for the VM, a stack machine in the crate `zaka_vm`. It takes in the
`ir::Module` that [the flow checker](borrow-checker.md) leaves and produces a
`bc::Module`, which `zakac build` writes as an artifact (`.zkc`, or `.zkct` as text) or as
a listing (`--emit asm`). [The IR](codegen.md) describes the input, and [Inside the
VM](vm-internals.md) describes how the VM runs the output. [Instructions](../vm/instructions.md)
lists each operation from the side of its user.

```text
   ir::Module (folded, checked)
        │  emit                            zakac_codegen_target_vm::emit
        ▼
   bc::Module ──▶ listing (asm text), .zkct, .zkc
        │
        ▼  zaka_vm::machine::Machine
   the output of the program
```

## An example

`zakac build --emit asm` writes the listing. For `sq.zaka`, the program of the [IR
example](codegen.md#an-example), it is:

```text
; zakac asm listing v1, platform 64 bits
globals
  g0 အလျား isize

fn f0 နှစ်ထပ် (params 1) -> isize
locals
  l0 က isize
     0  local_addr l0        ; က
     1  load isize
     2  local_addr l0        ; က
     3  load isize
     4  mul
     5  ret

fn f1 ပရိုဂရမ် (params 0)
     0  push 12:isize
     1  call f0              ; နှစ်ထပ်
     2  print
     3  ret

fn f2 main (params 0)
     0  global_addr g0       ; အလျား
     1  push 12:isize
     2  store isize
     3  call f1              ; ပရိုဂရမ်
     4  ret
```

In `f0`, operations 0 to 4 read `က` twice and `mul` makes the product, and `ret` leaves
the product on the stack, which is how a result travels. In `main`, the global gets its
value (operations 0 to 2) and `call f1` runs the program start block (entry point).
There, the argument `12` is pushed (operation 0, because the folder turned `read g0` into
a constant), `call f0` moves the argument from the stack into the local `က` and leaves
the result, and `print` writes it. The program prints `၁၄၄`.

## The module of operations

`zakac_codegen_target_vm::emit(module, platform_bits, options)` turns the folded IR into a
`bc::Module`:

```text
Module { platform_bits, built_for, globals: Vec<Slot>, texts: Vec<String>, seqs: Vec<StaticSeq>,
         formats: Vec<Vec<FmtArg>>, funcs: Vec<Func>, main: u32, overflow, host,
         foreign: Vec<ForeignFn>, foreign_calls: Vec<ForeignCall> }
Func   { name, params, ret, locals: Vec<Slot>, code: Vec<Op>, spans: Vec<Span> }
```

- `funcs` has the IR functions at their IR numbers, the program start block among them,
  then one more function, `main`, which sets the top-level variables and calls the
  program start block. `main` is its number.
- `texts` is the table of static texts, each distinct text once: string literals, folded
  formats and printed text. `print_text` and `text` name them through their index.
- `seqs` is the table of static lists that `static_seq` and `seq_addr` name.
- `formats` lists the pieces of each format, `FmtArg::Text(t)` for a static text or
  `FmtArg::Value(kind)` for a value from the stack. `format` names one through its index,
  and the listing shows them in a `formats` section (`fmt0 t0 isize t1`).
- `spans` runs parallel to `code` and gives the source region of each operation, which is
  how the VM and the explorer point at the line being run, and how a trap names its line
  (see [Inside the VM](vm-internals.md#limits-and-traps)). `Module::strip_debug` replaces
  each region with `Span::PLACEHOLDER`, the span of no source, which is what a release
  build keeps (`--release`).
- `overflow` is the `--overflow` mode that the options of the flow checker have. With
  `stop`, the default, the VM works out `prefix` and `infix` with checked arithmetic and
  traps on a result outside its data kind (type). `wrap op` wraps in each mode, and the
  generator emits it for the steps and the distance of a counted loop, which depend on
  two's complement arithmetic.

The generator can add its own locals, named with a `%`: `%to`, `%step` and `%dir` for a
counted loop, `%list`, `%slice`, `%ret` and `%at` for the lists, structs and ranges a
statement builds, `%text` for text with no slot to go to, such as a printed format, and
`%cell` for the value of an add. `EmitOptions` has the two choices of the generator:
`results` (`--results`) and `scratch` (`--scratch`), both [below](#results-and-scratch-locals).

## Operations

The VM is a stack machine. Operations that work on values get the data kind from the
values on the stack, so most operations name no data kind. The ones that need one name
it: `convert`, `load`, `store`, `take`, and the list operations. Each operation is a
variant of `bc::Op`, and its mnemonic is the variant name in snake case, with an
operator printed as its short name (`add`, `lt`, ...).

| Group | Operations |
| --- | --- |
| values | `push v`, `pop`, `dup` |
| storage | `local_addr n`, `local_ref n`, `global_addr n`, `load k`, `store k`, `take k` |
| arithmetic | `prefix op`, `infix op`, `compare op`, `convert k`, `wrap op` |
| control | `jump t`, `jump_if_false t`, `jump_if_true t`, `call f`, `fn_addr f`, `closure f k`, `call_value n`, `ret`, `result_addr`, `trap code` |
| host and C | `host h`, `foreign c<n>`, `c_leave k`, `c_revoke`, `c_load k at o`, `c_store k at o` |
| output | `print`, `print_text t`, `print_list k`, `print_enum k` (a member, found through its value and written as its name; a function value, written as the function it contains) |
| failure | `halt cause [k]`, with `not_good`, `not_fault`, `no_data`, `closed_handle` and the other causes of `HaltCause` |
| pointers | `ptr_add stride`, `ptr_sub stride`, `alloc stride`, `realloc stride`, `free`, `mem_blit stride`, `mem_move stride`, `mem_fill k`, `pin`, `obj_to_ptr`, `ptr_to_obj` |
| lists | `index len [from f]`, `blit k`, `zero k`, `list_eq k`, `seq_eq k`, `static_seq s`, `seq_addr s`, `to_heap k`, `add_cell k`, `remove_cell k`, `insert_cell k`, `object_cell stride` |
| views | `make_slice stride source`, `slice_cell stride [from f]`: the inner views that a range and a cell of a plain list read through |
| text | `text t`, `format fmt`, `drop k`, `dup_seqs k`, `deep_seqs k`, `lend_seqs k`, `own k`, `text_eq`, `char_count`, `char_at`, `char_at_octet`, `set_char [from 1]`, `add_text k`, `insert_text k`, `replace_text k [chars]`, `text_edges`, `check_utf8` |
| ဇယား (map) | `row_find k`, `set_row k`, `remove_row k` |
| threads | `spawn f`, `wait [k]`, `pipe_new [room]`, `pipe_send k`, `pipe_receive k`, `pipe_close`, `select n [timed|idle]`, `sleep`, `lock`, `unlock`, `atomic_add [sub] k`, `atomic_cas k` |
| ring collector | `rings act`, `ring_stat n` |

A few of these need a word more:

- A range of text that is not the whole text gets `text_edges` after its `make_slice`,
  which stops the program when an edge falls inside a character. When the fold knows the
  text, the start and the length, it reports such a range as ZK0607 first.
- A conversion from a plain list of octets (bytes) to text emits `check_utf8`, which stops
  the program when the octets are not UTF-8.
- `pin` marks a block so a later duplicate (copy) clones it.
- `obj_to_ptr` turns an object into a pointer to its fields and keeps a count on it, and
  `ptr_to_obj` turns the pointer back and takes that count over.
- `dup_seqs`, `drop` and `own` retain, release and unshare counted blocks. The ring
  collector runs inside the VM, and `rings` and `ring_stat` serve its statements (see
  [Inside the VM](vm-internals.md#the-ring-collector)).
- `char_count` and `char_at` get their text through `read_operand`, so a fresh text is
  dropped after the read.

Each variable lives in storage (memory). A read is `local_addr n` or `global_addr n`, then
`load`, and a write is the address, the value, then `store`. That is the reason each
access of a variable can be checked: it goes through an address of a block, the same as
a heap pointer does.

`CodeBuilder` (`code.rs`) collects the operations of a function. A jump names a `Label`,
and the labels are bound to operation numbers when their places are reached. At the end,
the jumps are patched with the numbers, and `Unbound` is an error for a label that was
named and never bound.

## How statements become operations

| IR | Operations |
| --- | --- |
| `and` | `<l>; jump_if_false F; <r>; jump E; F: push false; E:` |
| `or` | `<l>; jump_if_true T; <r>; jump E; T: push true; E:` |
| `if` | `<cond>; jump_if_false Else; <then>; jump End; Else: <els>; End:`; the first `jump` is left out when `els` is blank or `then` ends in a jump |
| each loop | lowered to a `Loop`: `init` sets `%each` to a duplicate of the value (a lent one when the loop leaves the value as it is, see [the flow checker](borrow-checker.md#one-routine-the-flow); through the pointer for a pointer to a list) and `%new` to 0, `cond` tests it against the count, each round sets the variable to the cell at that number or to the character that starts at that octet (`char_at_octet`), and `update` adds 1, or the octet count of the character |
| while, generic loop | `<init>; Head: <cond>; jump_if_false Exit; <block>; Next: <update>; jump Head; Exit:`; `continue` goes to `Next`, `break` to `Exit` |
| call statement | `<args>; call f`, then `pop` when the callee has a result |
| declaration of a nested function with captures | `<record>; closure f k` |
| leave for C | `<value>; c_leave k` or `<value>; c_revoke` |
| field of a struct C keeps | `<address of the struct>; c_load k at o`, with a block address pushed first for a list or text |
| call through a function value | `<value>; <args>; call_value n`, the value read first; the result kind and the `%ret` slot come from the function kind (`fn_shape`) |
| function | its block, then `ret` unless the block ends in one |

## Results and scratch locals

A list or struct result travels through a slot of the caller. Each call of such a
function gets a `%ret` local: the caller pushes `local_addr` of it before the arguments,
and the call leaves that address on the stack as its value. Inside the function,
`result_addr` pushes the same address. `ret` takes the address the function returns and,
unless it is the slot itself, duplicates the value into the slot before the frame goes.
A list or struct local that is returned is not moved out first (no `take`), since its
frame ends with the return.

`results` (`ResultMode`, the `--results` option) picks what lives in the slot:

| Mode | Operations |
| --- | --- |
| `in-place` (default) | the local that each `return` returns is read and written through `result_addr`; a returned literal is built at `result_addr`, and a returned call passes `result_addr` on as its slot, so `ret` duplicates nothing |
| `duplicate` | the function keeps the value in its own frame, and `ret` duplicates it into the slot |

A list, a struct or a range set into a slot is built in the slot itself when nothing it
reads could change while it is written: a range, whose `make_slice` reads its source
before it writes, and a literal whose cells are constants, other slots, and operators on
these. Each other value, such as `l = [l[1], l[0]]`, is built in a `%list` or `%slice`
local first and then duplicated in with `blit`, so it reads the slot whole. A literal or
a range with no slot to go to, such as one printed or passed, is built in such a local
too.

`scratch` (the `--scratch` option) picks how those locals are shared. With `--scratch
apart`, each such value gets its own local. With `--scratch shared`, a value of one
statement is dead once the statement ends, so a later statement takes a free local of the
same name and data kind, while the values of one statement still keep their own slots.
The locals of a counted loop live across its block and are never shared.

## Counted loops

A counted loop evaluates `from`, `to` and `step` once, in that order. A constant bound or
step is pushed again where needed, and a value that is not constant goes into a `%to` or
`%step` local. A step that is not a constant is checked once, and the program stops with
`step_not_positive` when it is zero or less. The variable is never moved past `to`, so it
cannot wrap. After each round:

```text
counting up:    if (to as U) - (var as U) < (step as U) then leave, else var += step
counting down:  if (var as U) - (to as U) < (step as U) then leave, else var -= step
```

`U` is the unsigned data kind of the same width as the variable. The difference is never
negative, so it fits `U` even for the range of `i8`. With both bounds constant, the
direction is chosen at compile time. Otherwise a `%dir` local keeps it, and the two
directions share one run of the block.

The listing of a counted loop shows the labels. For `၁ မှ ၃ အထိ ကိန်း အနေဖြင့် ရေတွက်၍
ကိန်း ကို ပြသမည်။`:

```text
fn f0 ပရိုဂရမ် (params 0)
locals
  l0 ကိန်း isize
     0  local_addr l0        ; ကိန်း
     1  push 1:isize
     2  store isize
     3  local_addr l0        ; ကိန်း
     4  load isize
     5  push 3:isize
     6  gt
     7  jump_if_true L1
L0:
     8  local_addr l0        ; ကိန်း
     9  load isize
    10  print
    11  push 3:usize
    12  local_addr l0        ; ကိန်း
    13  load isize
    14  convert usize
    15  wrap sub
    16  push 1:usize
    17  lt
    18  jump_if_true L1
    19  local_addr l0        ; ကိန်း
    20  local_addr l0        ; ကိန်း
    21  load isize
    22  push 1:isize
    23  wrap add
    24  store isize
    25  jump L0
L1:
    26  ret
```

Operations 3 to 7 are the one test before the first round, so a start past the end runs
zero rounds. Operations 8 to 10 are the block. Operations 11 to 18 are the test of the
formula above, with `U` = `usize`. Operations 19 to 25 add the step and go round. The
loop variable is `ကိန်း`, a local of the program start block.

## Artifacts

`zakac irgen` writes the folded IR (`.zkirt` text, `.zkir` raw), `zakac borrowck` writes
the IR and the digests, and `zakac build` writes the operations (`.zkct`, `.zkc`) or, with
`--emit asm`, the listing. Text artifacts start with a first line such as `#
zakac-codegen-artifact v1` and are the form of the golden files, while the raw form is the
`ZKAR` envelope with Postcard inside. `zakac run` accepts Zaka source, which it compiles,
or the operations of `zakac build` in either form, which it tells apart from the first
octets. A source file runs as native code when it can, and on the VM otherwise or with
`--vm`, while the operations of an artifact run on the VM.

An artifact keeps its spans relative to the start of its own source, so the spans do not
depend on where the source map placed the file. A decoded artifact registers its embedded
source and its imported files again, then makes its spans absolute against that
registered file (`absolute_module`, `absolute_diagnostics`). `relative` and `absolute` in
`zakac_ir_builder` leave a placeholder span as it is, since it names no source and a shift
would turn it into a region.

`CodegenArtifact::strip_debug` is the release form of an artifact: it calls
`Module::strip_debug`, then clears the embedded source, the imported files and the
diagnostics, whose regions point into that source. `zakac build --release` calls it
before the artifact is written, and `zakac run --release` calls `Module::strip_debug` on
the module it compiles. An artifact built with no `--release` keeps its regions, so
`zakac run x.zkct` still names lines. `--backend c` refuses the flag.

## Tests and extending

`zakac_codegen_target_vm/tests` has `code.rs` (label patching), `emit.rs` (the operations
of each construct), `listing.rs`, `artifact.rs`, `cli.rs`, and `samples.rs`, which keeps
golden `.zkct` and `.zkasm` files and checks that each operation of each sample is well
formed (indices in range, each function ending in `ret`).

- **A new IR form**: after the steps in [The IR](codegen.md#tests-and-extending), a rule
  in `emit.rs`, and an operation when the VM needs one (`bc.rs`, `listing.rs`,
  `machine.rs`).
- **A new operation**: a variant of `bc::Op`, its operand text in `Op::operand`, the arm
  of `Machine::exec`, and a test of the operation on a hand-built module.
