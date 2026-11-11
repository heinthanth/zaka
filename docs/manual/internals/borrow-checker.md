---
title: The flow checker
description: How the checker follows pointers, integers and unset values through the IR, the facts it keeps, the check that each value is set before it is read, the lint it runs on marked code, and the options that change what it reports.
---

# The flow checker

The checker finds mistakes before the program runs. On each part of a program it follows
values that are not set, cell numbers and integer ranges: a read of a variable, a field or
a cell before the program gives it a value, a cell number past the end of a list, an
operation that overflows on each run. On
marked code it also runs a *lint* on pointers: a block read after it was given back, a
block given back twice, a block that is never given back, a pointer read after an owned
hand-over. The rules, as a user meets them, are in [Unsafe Zaka](../language/unsafe.md)
and [Compiler checks](../compiler/checks.md), and the reasons in [Storage,
explained](../concepts/storage.md). This page is about the code in
`compiler/zakac_borrow_checker`; the crate keeps its old name.

The check is the first line of defence. The VM is the second: it checks each read and
write while the program runs (see [When a program stops](../vm/traps.md)), so a mistake
the check cannot prove still stops the program at the bad access. A test keeps the two
in step: each sample that passes the check, with no raw address in it, must run with no
use after free, double free or bad free.

## Where it runs

```text
 tree ──▶ names and kinds ──▶ lower ──▶ fold ──▶ FLOW CHECK ──▶ codegen
                                                    │
                       Module (folded IR) ──────────┤  borrow_check(&mut Module, options, platform_bits)
                                                    ▼
                          BorrowCheckOutput { diagnostics, digests }
                          and `checked: true` set on the cell addresses it proved safe
```

The IR is the right input because it has made explicit, before this stage, what the
checker needs. Names are slots (`g0` for a global, `l1` for a local), blocks are
flattened, widening is a `Convert`, a hand-over out of a place is a `Take`, a heap
sentence is an `Alloc`, `Resize` or `Free` with a destination, and the end of a counted
place is a `Drop`. `Module.marked` holds the source regions of marked code: the bodies of
marked functions and of marked blocks. For this program:

```text
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားမည်။
ကိန်းရှင် q သည် ပ ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
q ၏ တန်ဖိုး ကို ပြသမည်။
```

`zakac irgen` shows six statements (spans and kinds left out): `set l0 = const 0x0` (the
declaration gives no value, so the slot `l0` notes the span of that `set` in `unset`, below),
`alloc l0 count 1 unit isize`, `store (read l0) 7`,
`set l1 = read l0`, `free l0`, `print load (read l1)`. The checker works on these alone.

## What it keeps: the State

Each statement list (a function, the start block, or `main`, which holds the starting
values of the top-level variables and the call of the start block) is a *routine*. The
start block is checked as a function. Its flow does not start from globals it knows
nothing of, as other functions do: `main` calls it once, as its last statement, and
nothing else can call it, so `run_start_flow` runs it again from the facts `main` has at
that call (`Flow::seed`), with the hidden host globals not known, since the VM fills them
first. Its diagnostics and cell proofs come from that run, and its digest from the run
with nothing known. At each point of a routine the checker keeps a `State`: one fact for
each global and each local, a `SlotState`.

| Fact | Holds |
| --- | --- |
| `Int(Interval)` | an integer slot, as a range `lo..=hi` of the values it can have |
| `Ptr(PtrInfo)` | a pointer, a shared object or a function value; a text slot is followed as a pointer to its block is |
| `List(PtrInfo)` | a list of pointers, one fact for all its cells |
| `None` | a slot the check does not follow |

A `PtrInfo` is `{ state, roots, offset, len, cells }`. `state` is an `OwnerState`, `roots`
names where a pointer can point when the block is not its own, `offset` is an interval of
element offsets from the start of the block, and `len` is the element count of the block
when known. A plain list slot holds `HELD` after an add, a remove or a character edit,
since each leaves it with a block of its own. For a plain list of pointers, `cells` is the
join of what its cells hold: an add joins the state of the pointer added, a literal gives
the join of its cells, and a write through a cell or a call that can reach the list makes
it `UNKNOWN`, since the check does not follow cell numbers.

The kinds of the checker keep two flavours of pointer that no source spells: the IR marks
the addresses it makes itself (the address of a `ဒေတာ ပြောင်းလဲရန်` argument, the boxes of a
closure record) as borrowed, so the check never takes them for blocks to give back. Each
pointer the program declares is a plain pointer.

### Pointer states

`OwnerState` is a set of facts in one octet, joined with a bit or:

| Fact | Meaning |
| --- | --- |
| `NULL` | holds the null pointer, `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` |
| `HELD` | holds a block |
| `FRESH` | holds a new allocation never compared with the null pointer: a block or null, as the heap can run short |
| `MOVED` | handed its block to an owned parameter; holds the null pointer, and a read is a mistake |
| `GIVEN` | a mark, not a fact: a call could have freed, resized or taken the block, so the slot is not reported as a leak |

```text
                  alloc, resize                test: not null
        NULL ───────────────────▶ FRESH ───────────────────▶ HELD
         ▲  ▲                       │                        │   │
         │  └──── test: is null ────┘                        │   │ owned hand-over
         │                                                   │   ▼
         └─────────────────────── free ──────────────────────┘ MOVED
```

A pointer declared with `ကနဦးတန်ဖိုး` or the null pointer starts at `NULL`, and so does one
declared with no value, which [the init check](#set-before-read) also reports at a read
before a write; an allocation or a resize makes it
`FRESH`; a free sets the slot back to `NULL`. An argument for a parameter marked
`အပိုင်အနေဖြင့်` is lowered as a `Take` (`hand_over` in the IR builder), which leaves
the caller's place `MOVED`; a `Take` of text, a list or a function value leaves the place
blank and not known, since a blank one reads as each value does. A read of a moved
pointer sees `NULL` (`as_read`), and the one read that remains allowed is the test
against the null pointer. Where two paths meet, the sets are joined, so after an `if` that hands the
block over on one branch alone the state is `HELD | MOVED`.

The reports read the joined state with care. ZK0701 (a read after an owned hand-over)
fires when the state is `MOVED` and nothing else, that is, on each path to the read.
ZK0709 (a read through the null pointer) fires when the state is `NULL` and nothing else,
and on a pointer slot alone (`check_deref` in `checks/nullness.rs`): a shared object and a
function value are never null. A mistake
that happens on some paths alone is left to the VM. This is the rule that keeps the check
free of false alarms on a correct program: the checker reports "is" and not "can be".
The strict mode (below) adds the "can be".

A test against the null pointer narrows the state on each edge of the branch (`narrow`): after
`အကယ်၍ ပ သည် မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ နှင့် ညီလျှင်`, the "then" edge keeps the facts that allow null and the
other edge keeps the facts that allow a block; a `FRESH` pointer becomes `NULL` on the
first edge and `HELD` on the second.

## Roots and paths

A pointer into a block it did not make does not name the block. The checker gives each
such pointer a set of *roots*: places it can point into. A `Root` is a slot and a *path*
of steps, `Block` (the block a pointer holds) or `Cell(n)` (one cell of a list, `None`
when the number is not known):

| Expression | Root |
| --- | --- |
| `x ၏ လိပ်စာ` | `x`, the slot itself |
| `ပ + ၁`, a copied `ပ` | `ပ.block` |
| `ခ ၏ တန်ဖိုး + ၁` | one `block` deeper than what `ခ` points at |
| `l ၏ အကွက် ၁ ၏ လိပ်စာ` | `l.cell[1]` |
| a cell with an unknown number | `l.cell[*]`, the whole list |

Arithmetic, a conversion and a duplicate of a pointer *derive* the new pointer from the old
one (`derived`, `escape_dup`): the new one roots at the block, and the block remains the
old one's to give back, so a duplicate is not a second block to leak. A pointer that holds the
address of a known place keeps its roots (`holds_address`, `Root::reaches`).

A free or a resize of `ပ` gives back the path `ပ.block`. It *reaches* a root when its path
is a prefix of the root (an unknown cell matches each cell). So a pointer to the slot `ပ`
survives a free of the block of `ပ`, while a pointer into the block does not. Paths are
cut at four steps (`MAX_PATH`); a cut path is a shorter prefix, which reaches more, so the
cut errs on the safe side.

## One routine: the flow

`run_flow` takes a routine through five steps (`flow.rs`).

```text
 build_cfg      statements ──▶ graph of nodes:  Start, Leaf, Branch, LoopHead,
                                                CountedInit, CountedHead, End, Exit
    │
 live_out       backward fixpoint: which pointer slots can still be read after each node,
    │           with the span of one such read as a witness
    │
 solve          forward fixpoint of State per node (a worklist in reverse postorder;
    │           join at merges, widen at loop heads)
    │
 report_pass    run the transfer of each node once more with the final facts,
    │           now with `reporting = true`, so the checks fire
    │
 check_loop_moves, finish   second look at loops; produce the digest and the proofs
```

**The graph** (`cfg.rs`) has one node for a statement with no statement nested in it, one
for the test of an `if` or a loop (two out-edges: holds, does not hold), a pair for a
counted loop, and an `End` and an `Exit` node. `break`, `continue` and `return` link to
the right place.

**Liveness** (`liveness.rs`) is the backward pass. A pointer slot is live after a node
when some path from there reads it before writing it again. Because it looks at all
paths, a read that runs once a month is found as well as one that runs each time.

**The solve** is the forward pass: `transfer` turns the state before a node into one state
for each successor edge. Integers are intervals, and a loop head *widens*: if a bound
moved since the last visit it jumps to the end of the range, so the pass ends. The facts
are recorded but nothing is reported so far, since a fact seen in the first round of a loop
can still change.

**The report pass** runs `transfer` again over the final facts with `reporting` on. Now
each check in `checks/` can raise a diagnostic; `report` keeps one per code, span and
message.

A free, a resize or a hand-over is a *loss* of a block (`Loss`, with the paths that lose
it). When a node ends, `check_losses` takes each loss and asks whether a *live* pointer
slot has a root that the lost path reaches. If so: ZK0704, with a second label on the
witness read from the liveness pass. Here is the program from above in marked code, as the
compiler reports it:

```zaka-warning ZK0704
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားမည်။
ကိန်းရှင် q သည် ပ ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
q ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0704]: the block of "ပ" is freed here, but "q" still points into it and is used later
 --> uaf.zaka:7:1
  |
7 | လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
8 | q ၏ တန်ဖိုး ကို ပြသမည်။
  | - "q" is used here
```

The finding is a warning, as `--mem-check warn` is the default, so the program runs, and
the VM stops it at the read of `q`. Swap the last two statements and the program is
correct: `q` is no longer live at the free.

A `Blit` or a `Fill` is checked as a write and a read through an address. `Fill` and the
destination of a `Blit` go to `write_cells`, which treats the address as a `Store` does: a
destination that is the null pointer for sure, or whose first cell lies past the end of its block, is
a finding, and the facts of the slots the address names are forgotten. The source of a
`Blit` goes to `read_cells`, a read through the address. Both also check the last cell
(`check_last_cell`): the octets of the run, from `octets`, at the end the count reaches on
some path, rounded up to whole cells of the pointee. A run of no octets checks neither
address. For a `ကူးယူမည်`, `check_overlap` reports ZK0721 when both addresses name one
block (`blocks_of`, as for ZK0712), both offsets and the count are single values, and the
octet ranges share an octet; else the VM's trap `overlap` remains.

A write into a plain list is a loss too, of the pointers into its cells. Lowering wraps
each plain list on the path of a written place in an `Own` node, since the write can first
move its cells to a block of its own; for a `Store`, `owned_lists` finds the place of each
such list and `check_heap` runs with `Invalidation::Edited`. A pointer into the cells that
is still live is ZK0704, with "is edited, so its cells can move," in place of "is freed".
An `AddCell`, an `Insert`, a `Replace` or a `RemoveCell` is a loss as well: an add can move
the cells to a larger block, and a remove drops the last cell. `SetChar` and `AddText`
edit text in the same manner. The address of the cells of a list or of text pins its
block (`Own { pin: true }`), so a duplicate of the list made later clones the cells and a
write through the pointer never reaches it; the edits above can still move the pinned
cells, and the check sees them as losses.

Before the flow, `last_use.rs` turns each `Dup` of a whole place that the program reads no
more into a `Take`, so the check follows the code that runs and the VM retains and
releases less. A `Dup` remains for a parameter; for a global read in a function; for a
global that a call of its statement sets; for a place that a range or an address reads in
some routine; and for a place its statement reads twice. A deep `Dup`, from a deep
duplicate, is never turned into a `Take`.

After the moves, `lend_arguments` marks each `Dup` left in a call argument `lent`, so the
callee reads the caller's u8s, or the cells of a plain list, with a room of 0 and no
retain, unless the call, or a call inside its arguments, sets the global read
(`global_writes`, at each depth), or the place is read through a pointer, or a range or an
address reads it. A box, which holds a captured local, is never lent. The same pass lends
the `Dup` a loop starts from, such as the source of an each loop (`lend_loop_dup`), when
the loop reads the duplicate through its cells alone and leaves the source as it is. Last,
`share_params` fills `Func::shared_params` with each list or struct parameter that holds
no text, plain list or pointer and that the function never sets, stores into, drops or
takes the address of; codegen reads such a parameter in the caller's storage.

## The checks

Each file of `checks/` owns some codes. All of them work on a `Flow` and the `State`.

| File | Raises | Finds |
| --- | --- | --- |
| `moves.rs` | ZK0701 | a read of a pointer that is `MOVED` on each path, after an owned hand-over, also in a later round of a loop (`check_loop_moves`) |
| `static_rules.rs` | ZK0702, ZK0703 | a free or a resize through the address of a variable, or of a cell or field it holds in place (each root with no block step); the address of a variable of a function (a root at a local slot with no block step) stored in a heap block, through a pointer, or in a global from a function |
| `loans.rs` | ZK0704 | a block lost while a live pointer can point into it |
| `escape.rs` | ZK0706 | a function that returns the address of its own local or parameter |
| `calls.rs` | ZK0704, ZK0707 | the effect of a call (below); one block freed through one argument and reachable through another |
| `leaks.rs` | ZK0708 | a block this function made (`FRESH`) that is overwritten, or still held when the function ends, and was not freed, handed to an owned parameter, returned as an owned result, or stored into a heap block, a global or a field; an unmarked return of such a block gets the help to mark the result `အပိုင်အနေဖြင့်` |
| `nullness.rs` | ZK0709, ZK0711 | a read through a pointer that is `NULL` (or `FRESH`, in strict mode); a shared object and a function value are never null, so it skips them |
| `bounds.rs` | ZK0710, ZK0712 | an offset or cell number proven outside its block; a pointer difference between different blocks |
| `lists.rs` | | the facts of a list of pointers, one fact for all its cells |

ZK0722 to ZK0724 come from a check outside `checks/`, with facts of its own: see [Set
before read](#set-before-read).

An owned hand-over into a call is a `Take` of the argument's place, so the check follows
it as it follows each other statement:

```zaka-error ZK0701
စိတ်မချရသော လုပ်ဆောင်ချက် ရှင်းပစ် သည်
က ဟူသော ကိန်းပြည့် လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လိပ်စာ က ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
လုပ်ဆောင်ချက် ရှင်းပစ် ကို ပ တို့ဖြင့် လုပ်ဆောင်မည်။
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0701]: "ပ" handed its block to a parameter marked "အပိုင်အနေဖြင့်" and now points at nothing
```

The parameter of `ရှင်းပစ်` is marked, so the call takes the block and `ပ` becomes
`MOVED`; ZK0701 is no lint and remains an error in each mode. The checker sees this without
looking inside `ရှင်းပစ်`. `Func.owned` (from `Frame.owned_ret` in the IR builder) records
an owned result, which the caller takes as a block of its own, as it takes a fresh
allocation.

### Intervals and bounds

Integer slots hold an `Interval`, and arithmetic on intervals (`add`, `sub`, `mul`, with an
overflow giving the whole range) gives the offset of a pointer and the number of a list
cell. A counted loop gives its variable the range of the bounds. `check_offset` and
`check_cell` report ZK0710 when the interval of an offset lies outside the block
altogether. When a cell address is proven in range, its span goes to `CellProofs`; at the
end, `mark_checked` sets `checked: true` on those `CellAddr` nodes, and code generation
leaves the run-time `index` op out for them. This is done when the check found no error and
the program uses no raw address, since a raw address can change a fact the proofs rest on.
The proof marks `CellAddr` alone, so a loop over the cells of a plain list parameter keeps
its run-time check (`SliceCell`, the internal op of such a cell).

With `--overflow stop`, the default (`BorrowCheckOptions.overflow`), `check_overflow` works
out the interval of each integer `+`, `-`, `*` and negation from the intervals of its
operands, before it is cut to the kind. When that interval is bounded and lies outside the
range of the kind, each value the operands can hold overflows, and the check reports
ZK0719, a warning: the VM stops there each time the program reaches it.

## Calls: digests

Inside a function the checker cannot see its callers. It assumes the parameters are
distinct, runs the flow, and writes a *digest* (`FnDigest`), a short record of what the
function can do to the storage it reaches:

| Field | Plain words |
| --- | --- |
| `frees` | the parameter and global paths it can free, resize or take over, through hooks too |
| `unfilled` | paths it freed and did not fill again, so the pointer there can be null after |
| `result_from`, `result_roots` | which parameters or globals a pointer result can point into |
| `renewed` | result paths that point into a block the function made or resized itself |
| `reads` | the paths it reads or writes through, also in its callees |
| `result_roots_ok` | whether the result points into nothing that dies with the call |

At a call, `check_call` translates the digest through the real arguments (a path from
parameter 1 becomes a path from the slot that was passed) and treats it as the loss of
those blocks, then checks the losses against the live pointers as for each node. A call
that frees a global under a live pointer is ZK0704 at the call; two arguments that reach
one block the function frees through the first are ZK0707.

Digests are computed in groups of functions that call each other (`callgraph.rs`), callees
first; a group that calls itself runs again until none of its digests grows (a fixpoint),
so recursion and mutual recursion are covered. The globals add a loop around this: a
global pointer whose starting value comes from a call takes its roots from a digest, and a
function that reads the global starts from those roots. `roots_and_digests` repeats until
the roots stop growing. Then `main` runs last. `zakac borrowck` prints the digests; for
`ရှင်းပစ်` above:

```text
digests:
- name: ရှင်းပစ်
  params: 1
  frees:
  - place: l0
    path:
    - Block
  unfilled:
  - place: l0
    path:
    - Block
  result_roots_ok: true
```

(the blank fields are left out). The order of the statements inside the callee is not
looked at, so a function that reads a global and then frees it counts as a function that
frees and reads; this costs a few false alarms and closes a group of holes in return.

A call that passes a failure on is lowered to a plain return that leaves the function (see
[IR and code](codegen.md)), so the checker needs no rule of its own: a block alive at the
pass-on site is lost when the function leaves, which is ZK0708 at the call. The IR
statement `Halt` (the causes `not_good`, `not_fault` and `no_data`) ends a path in the graph
(`cfg.rs`), with no leak check and no exit check there: the run stops, so nothing is left
to free.

A call through a function value (`Stmt::CallValue`, `ExprNode::CallValue`) names no
function, but a function value comes from naming a function alone. `values.rs` gives the
functions the module names as values (`taken_fns`) and, of those, the ones whose
parameters and result fit the kind of the value (`value_callees`, `Func::fits_kind`).
`Flow::call` takes a list of functions and runs `check_call`, `note_unfilled` and
`lift_reads` for each, and joins what each gives back; `call_roots` joins their result
roots. A function named as a value gets no shared parameters (`share_params`): a call
through a value passes each argument in its own slot, whichever function runs.

A closure is a function value with a record: the walk reads the record of
`ExprNode::Closure` and gives the value the fact `HELD`, and `addressed_fn` (`values.rs`)
counts its thunk as a taken function, so a call through a value reaches the thunk. The
record is a counted block of boxes, one for each captured local, and each box holds its
local for the routine and for the closure alike. `closures.rs` finds the closures of each
routine before its flow: the function each thunk calls and the addresses its record
passes (`lent_fields`). Which closures a slot holds is a fact of the flow,
`State::closures`; `Flow::call` treats an argument that can hold a closure as a call that
can run its function (`closure_args`, `run_or_not`, `run_held`), and `check_closure_args`
reports ZK0707 for another argument that reaches a block the closure can free.
`callgraph.rs` puts the thunk of each closure a function makes before the function, so the
digests are there.

A slot of a function kind gets a pointer fact (`SlotState::of_kind`, `fn_const`), `HELD`
for a function value, folded or not, so the walk knows which functions and closures it can
hold. No null check looks at it: a function value is never null, and no call through one is
reported, and the null check skips a shared object slot too. A function value or an object read
before it is set is the [init check](#set-before-read)'s to report; a cell of a counted list
of functions or objects that the check cannot prove set is the warning ZK0723, and the VM's
traps `null call` and `null object` remain as the backstop for that cell.

An external function has one statement, `ExprNode::Host`: the flow reads each argument and
gives a value with no roots, a fresh one, and the digest of the function follows from that
statement as for each other function. A C function has `ExprNode::Foreign` (or
`Stmt::Foreign`) in its place, read as the host form is: each argument is a read, and the
result is a fresh value. A list of numbers passed to C goes as the address of its first
cell, which the IR builds before the call (`c_list_arg`); a list built for a call that
takes other values waits in a hidden slot, which the scope end drops.

## Set before read

`init.rs` holds a check of its own, apart from the flow above: no variable, field or cell
is read before the program gives it a value. A declaration can leave the value out, and no
kind fills one in. The rules, as a user meets them, are in
[Statements](../language/statements.md#set-before-read).

The IR marks the slots to follow. A declaration with no value still lowers to a `Set` that
gives the slot its zero octets, so the VM and C see a slot like each other; the slot notes
the span of that statement in `Slot.unset`. The check follows these slots alone
(`followed`), so the slots the compiler makes for itself need no care, and the `Set` whose
span is the declaring one leaves its slot unset where each other `Set` sets it.

The fact of a followed slot is `Unset { parts, doubt }`: the parts that can still be unset,
and whether a cell was written at a position the check does not know. A slot is one part,
a plain struct has one part for each field, and a counted list one part for each cell, up
to `CELL_CAP` (256) cells; a longer list is one part. A struct inside a struct is followed one
leaf field at a time: a plain struct has one part for each leaf (`leaves`), and a field, at
each depth, is the range of leaves it holds (`field_leaves`, `Part::Field(first, count)`),
which `part_of` follows through the `load` of each outer field. A field of a slot that is no plain
struct, such as the count of a plain list, is the whole slot (`plain_part`). A variable a
closure shares lives in a box; its slot is followed too, and a write through the box is a
write of its one cell, so a closure that captures it before it is set reads it unset. The
facts are solved over the graph of each routine (`cfg.rs`) as the flow's are: a join keeps
each part that one path leaves unset. A second pass reports:

| Read | Report |
| --- | --- |
| the whole slot, or a field, a cell or the address of it, with a part that a path leaves unset | ZK0722 |
| the same, when a cell was written at a position the check does not know | ZK0723, a warning |
| a cell at a position the check does not know, while some cell is unset | ZK0723 when cells were set, ZK0722 when none was |

A write sets a part: a `Set` sets the whole slot, a `Store` to a field or to a cell at a
constant position sets that part, and a `Store` to a cell at another position sets the
`doubt`. An allocation into a slot, a field or a cell sets that part as a write does
(`alloc_into`). A `Drop` the compiler adds, as before a text value moves into a field,
reads nothing: an unset place holds zero octets, which drop as nothing.

A counted loop sets its cells at once, on the edge out of it (`fill`): fixed bounds, no
`break`, and a statement at the top level of its block that writes the cell of the counter
(`written_cell`) set each cell from the start to the end. Before the solve, `earlier_reads`
finds each read of `a[i-k]`, for a constant `k` of at least 1, inside such a loop: the read
is set when cells `s-k` to `s-1` are (`earlier_set`), and ZK0722 when one of them can be
unset (`earlier_read`). Each other statement that changes a slot in
place, such as an add to a list, reads the slot first (`dest_slot`).

Globals are checked at the calls of the top level and of the start block, the two routines
that set them. `liveness.rs` gives the globals each function reads, itself or through its
calls (`global_reads_of`), and a call through a function value reads those of each
function the program names as a value (`taken_fns`). A call whose callee reads a global
that a path to it leaves unset is ZK0724 (`call`). The top level runs first and keeps its
facts at the call of the start block (`check_main`), and the start block starts from them.
Each other function takes the globals as set, since each call of it is checked, and a write
of a global inside a function does not count as setting it for its callers.

A parameter marked `ဒေတာ ထည့်ပေးရန်` is followed too. The IR builder notes its span in
`Slot.unset` on the parameter slot, so `routine_slots` takes it among the followed slots
and keeps it apart as a filled place (`fills`): the routine starts with it unset, so a read
before a write is ZK0722, and at each return and at the end of the routine `check_fills`
reports each filled place with a part a path leaves unset, ZK0725 (`fill_unset_at_return`).
At a call, `fills_param` tells a filled parameter from the `unset` of the callee's slot; the
argument reads nothing but the cell number of its address, and after the call
`write_fills` sets the place it names, for a call statement and, through `fill_args`, for
each call inside an expression. An argument of a `ဒေတာ ပြောင်းလဲရန်` parameter is read as
each other one is, so a place that can be unset there is ZK0722. The function that wraps a
C function holds one statement, the call into C (`calls_c`); the check cannot see what C
writes, so it trusts the mark and follows none of the filled parameters there.

The diagnostics of this check join the others after `downgrade`, so `--mem-check` never
changes them: the kinds promise there is no NULL and no hidden default, and that promise
holds at each level. A pointer read before it is set also holds the zero octets of its
declaration, the null address, so the flow can report ZK0709 or ZK0711 at the same read;
`borrow_check` drops those, and the report of the unset read stands for both.

## Options

`BorrowCheckOptions { check: MemCheck, strict: bool, overflow }`, chosen with
`--mem-check`, `--mem-strict` and `--overflow` on `borrowck`, `codegen` and `run`.

| Option | Effect |
| --- | --- |
| `MemCheck::Warn` (default) | each lint finding is made a warning (`downgrade`); code generation goes on and the VM stops the program at the bad access |
| `MemCheck::Error` | each lint finding is an error and stops code generation |
| `MemCheck::Off` | the lint findings are dropped; the flow still runs |
| `strict` | also reports what cannot be proven safe: ZK0711, a read through a new allocation never compared with the null pointer |

`downgrade` sorts each finding into the lint or not:

| Set | Codes | Under `--mem-check` |
| --- | --- | --- |
| `LINT` | ZK0702, ZK0703, ZK0704, ZK0706, ZK0707, ZK0708, ZK0712, ZK0721 | follow the mode wherever the check finds them; pointers, which marked code alone works with, give them |
| `MARKED_LINT` | ZK0709, ZK0710, ZK0711 | follow the mode inside a region of `Module.marked`, and remain errors in safe code, where a cell number past the end of a list gives ZK0710 |
| the rest | ZK0701, ZK0719 | as their codes define them in each mode |
| the init check | ZK0722, ZK0723, ZK0724 | added after `downgrade`, so no mode changes them: ZK0722 and ZK0724 are errors and ZK0723 a warning in each mode |

```zaka-error ZK0704 mem-check=error
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားမည်။
ကိန်းရှင် q သည် ပ ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
q ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

The rules of marked code itself (ZK0468 to ZK0472: an unsafe operation outside marked
code, a marked function as a value, an owned marker on a non-pointer, `--no-unsafe`, a C
declaration that needs the mark) are decided in [the kind check](semcheck.md) and are
errors in each mode. The choice to mark proofs is made before `downgrade`, on the raw
findings, so a program with a finding keeps all its run-time checks, whatever the mode. A
hand-over leaves the null pointer behind in each mode, so a program does the same thing whichever
mode compiled it.

## Tests and extending

`tests/` has a file for each group (`moves.rs`, `loans.rs`, `calls.rs`, `leaks.rs`,
`nullness.rs`, `bounds.rs`, `lists.rs`, `escape.rs`, `free_rules.rs`, `modes.rs`), golden
`.zkbckt` files for clean inputs, `still_works.rs` for programs that look unsafe and are
correct, and `oracle.rs`, which runs each program that passes the check in the VM and asks
for no use after free, double free or bad free. Some of these tests still state the rules
of the earlier model and wait to be rewritten around blocks.

- **A new mistake**: a check in `checks/`, called from the right place in `Flow::stmt` or
  `node_outs`; a `ZK07` code, texts and a constructor in `zakac_diagnostics`; its place in
  `LINT` or `MARKED_LINT` when it is a lint; a test that fires it and a test that a correct
  close program remains clean.
- **A new fact**: a field of `PtrInfo` or a form of `SlotState`, joined in `join` and
  `widen`, or the pass will not end.
- **A new instruction in the IR**: `walk.rs` (`each_expr`, `each_stmt`) must visit it, or
  the liveness pass and the call graph will miss it.

The next stage, which also reads this module, is [IR and code](codegen.md).
