---
title: The borrow checker
description: How the ownership check follows owners, borrows and moves through the IR, the facts it keeps, how it finds a use after free, and the options that change what it reports.
---

# The borrow checker

The borrow checker finds pointer mistakes before the program runs: a block read after it
was given back, a block given back twice, a block that is never given back, a pointer
used while it is `ဗလာ`. The rules, as a user meets them, are in
[Ownership and borrowing](../language/ownership.md) and, with the reasons, in
[Ownership, explained](../concepts/ownership.md). This page is about the code in
`compiler/zakac_borrow_checker`.

The check is the first line of defence. The VM is the second: it checks each read and
write while the program runs (see [When a program stops](../vm/traps.md)), so a mistake
the check cannot prove still stops the program at the bad access. A test keeps the two
in step: each sample that passes the check, with no raw address in it, must run with no
use after free, double free or bad free.

## Where it runs

```text
 tree ──▶ names and kinds ──▶ lower ──▶ fold ──▶ BORROW CHECK ──▶ codegen
                                                    │
                       Module (folded IR) ──────────┤  borrow_check(&mut Module, options, platform_bits)
                                                    ▼
                          BorrowCheckOutput { diagnostics, digests }
                          and `checked: true` set on the cell addresses it proved safe
```

The IR is the right input because it has made explicit, before this stage, what the checker needs.
Names are slots (`g0` for a global, `l1` for a local), blocks are flattened, widening is a
`Convert`, a move out of a place is a `Take`, a borrow is a `Lend`, a heap sentence is
an `Alloc`, `Resize` or `Free` with a destination, and the end of a text place is a
`Drop`. For this program:

```text
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ။
ကိန်းရှင် မိတ္တူ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ပ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
မိတ္တူ ၏ တန်ဖိုး ကို ပြသပါ။
```

`zakac irgen` shows six statements (spans and kinds left out): `set g0 = const 0x0` (a new
owner starts at `ဗလာ`), `alloc g0 count 1 unit isize`, `store (read g0) 7`,
`set g1 = lend (read g0)`, `free g0`, `print load (read g1)`. The checker works on just
these.

## What it keeps: the State

Each statement list (a function, the start block, or `main`, which holds the starting values of the top-level variables and the call of the start block) is a *routine*. The start block is checked as a function; the rules `main` alone has, such as keeping a borrow in a top-level variable, now reach the starting values alone. At each point of a
routine the checker keeps a `State`: one fact for each global and each local, a
`SlotState`.

| Fact | Holds |
| --- | --- |
| `Int(Interval)` | an integer slot, as a range `lo..=hi` of the values it can have |
| `Ptr(PtrInfo)` | an owner or a borrow; a slice is a borrow with no offset or extent known; a text slot is followed as an owner is |
| `List(PtrInfo)` | a list of pointers, one fact for all its cells |
| `None` | a slot the check does not follow |

A `PtrInfo` is `{ state, roots, offset, len, cells }`. `state` is an `OwnerState`, `roots`
names where a borrow points, `offset` is an interval of element offsets from the start of
the block, and `len` is the element count of the block when known. A plain list slot holds
`HELD` after an add, a remove or a character edit, since each leaves it with a block of
its own. For a plain list of owners, `cells` is the join of what its cells hold: an add
joins the state of the owner added, a literal gives the join of its cells, and a write
through a cell, a move out of one, or a call that can reach the list makes it
`UNKNOWN`, since the check does not follow cell numbers.

### Owner states

`OwnerState` is a set of facts in one octet, joined with a bit or:

| Fact | Meaning |
| --- | --- |
| `NULL` | holds `ဗလာ` |
| `HELD` | holds a block |
| `FRESH` | holds a new allocation never compared with `ဗလာ`: a block or `ဗလာ`, as the heap can run short |
| `MOVED` | was moved out; holds `ဗလာ`, and a read is a mistake |
| `GIVEN` | a mark, not a fact: a call could have freed, resized or moved the block, so the slot is not reported as a leak |

```text
                  alloc, resize                test: not ဗလာ
        NULL ───────────────────▶ FRESH ───────────────────▶ HELD
         ▲  ▲                       │                        │   │
         │  └──── test: is ဗလာ ─────┘                        │   │ move out
         │                                                   │   ▼
         └─────────────────────── free ──────────────────────┘ MOVED
```

A new owner declared with no value starts at `NULL`; an allocation or a resize makes it
`FRESH`; a free sets the slot back to `NULL`; a `Take` (a move out) leaves `MOVED`. A
read of a moved owner sees `NULL` (`as_read`), and the one read that remains allowed is the
test against `ဗလာ`, the means to ask "was it given off?". Where two paths meet, the
sets are joined, so after an `if` that moves the owner on one branch alone the state is
`HELD | MOVED`.

The reports read the joined state with care. ZK0701 (a read of a moved owner) fires when
the state is `MOVED` and nothing else, that is, on each path to the read. ZK0709 (a
dereference of `ဗလာ`) fires when the state is `NULL` and nothing else. A mistake that
happens on some paths alone is left to the VM. This is the rule that keeps the check
free of false alarms on a correct program: the checker reports "is" and not "can be".
The strict mode (below) adds the "can be".

A test against `ဗလာ` narrows the state on each edge of the branch (`narrow`): after
`အကယ်၍ ပ သည် ဗလာ နှင့် ညီလျှင်`, the "then" edge keeps the facts that allow `ဗလာ` and the
other edge keeps the facts that allow a block; a `FRESH` owner becomes `NULL` on the first
edge and `HELD` on the second.

## Roots and paths

A borrow does not name the owner it came from. The checker gives each borrow a set of
*roots*: places it can point into. A `Root` is a slot and a *path* of steps, `Block` (the
block an owner holds) or `Cell(n)` (one cell of a list, `None` when the number is not
known):

| Expression | Root |
| --- | --- |
| `x ၏ လိပ်စာ` | `x`, the slot itself |
| `ပ ၏ တန်ဖိုးအငှား`, `ပ + ၁` | `ပ.block` |
| `ခ ၏ တန်ဖိုး ၏ တန်ဖိုးအငှား` | one `block` deeper than what `ခ` points at |
| `l ၏ အကွက် ၁ ၏ လိပ်စာ` | `l.cell[1]` |
| a cell with an unknown number | `l.cell[*]`, the whole list |

A free or a resize of `ပ` gives back the path `ပ.block`. It *reaches* a loan when its path
is a prefix of the root of the loan (an unknown cell matches each cell). So a borrow of the
slot `ပ` survives a free of the block of `ပ`, while a borrow of the block does not.
Paths are cut at four steps (`MAX_PATH`); a cut path is a shorter prefix, which reaches
more, so the cut errs on the safe side.

## One routine: the flow

`run_flow` takes a routine through five steps (`flow.rs`).

```text
 build_cfg      statements ──▶ graph of nodes:  Start, Leaf, Branch, LoopHead,
                                                CountedInit, CountedHead, End, Exit
    │
 live_out       backward fixpoint: which borrow slots can still be read after each node,
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

**Liveness** (`liveness.rs`) is the backward pass. A borrow slot is live after a node when
some path from there reads it before writing it again. Because it looks at all paths, a
read that runs once a month is found as well as one that runs each time.

**The solve** is the forward pass: `transfer` turns the state before a node into one state
for each successor edge. Integers are intervals, and a loop head *widens*: if a bound
moved since the last visit it jumps to the end of the range, so the pass ends. The facts
are recorded but nothing is reported so far, since a fact seen in the first round of a loop
can still change.

**The report pass** runs `transfer` again over the final facts with `reporting` on. Now
each check in `checks/` can raise a diagnostic; `report` keeps one per code, span and
message.

A free, a resize or a move out is a *loss* of a block (`Loss`, with the paths that lose
it). When a node ends, `check_losses` takes each loss and asks whether a *live* borrow
slot has a root that the lost path reaches. If so: ZK0704, "the block of `ပ` is freed
here, but `မိတ္တူ` still borrows it and is used later", with a second label on the
witness read from the liveness pass. Here is the program from above, as the compiler
reports it:

```zaka-error ZK0704
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ။
ကိန်းရှင် မိတ္တူ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ပ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
မိတ္တူ ၏ တန်ဖိုး ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0704]: the block of "ပ" is freed here, but "မိတ္တူ" still borrows it and is used later
 --> borrow.zaka:5:1
  |
5 | လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
6 | မိတ္တူ ၏ တန်ဖိုး ကို ပြသပါ။
  | --- "မိတ္တူ" is used here
```

Swap the last two lines and the program is correct: `မိတ္တူ` is no longer live at the
free.

A write into a plain list is a loss too. Lowering wraps each plain list on the path of a
written place in an `Own` node, since the write can first move its cells to a block of its
own; for a `Store`, `owned_lists` finds the place of each such list and
`check_heap` runs with `Invalidation::Edited`. A slice of the list still live is ZK0704,
with "is edited, so its cells can move," in place of "is freed". The `Own` that lowering
puts on the source of a slice or of the address of a cell is no `Store`, so it ends no
loan. The pass that takes the `Own` off a slice that reads alone keeps it on each slice of
a list that has such an `Own` left, so a slice without one never sees the cells move.

An `AddCell`, an `Insert`, a `Replace` or a `RemoveCell` is a loss as well: an add
can move the cells to a larger block, and a remove drops the last cell. An `Insert` into a
list moves its value in as an add does, and one into text reads its value as an add to
text does; a `Replace` reads its two ends and its value as well. `edit_list` runs
`check_heap` with `Invalidation::Edited` on the list itself and on each plain list
`owned_lists` finds around it, since those first take a block of their own; then the
value of an add moves into the list (`moved`), as an assigned value does. The liveness
pass counts both statements as reads of the list slot.

A `SetChar` and an `AddText` edit text in the same manner: the new u8s can move the text
to a larger block, and text with a room of 0 first takes a block of its own. Each reads
its operands first (the number and the character, or the added value), then runs
`edit_list` on the text, so each view of the text still live is ZK0704 and so is each loan
into a plain list that holds the text. A view read as the added value of the same
statement is used before the edit, and is fine. The liveness pass counts both statements
as reads of the text slot.

```zaka-error ZK0704
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် s သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
က ၏ အကွက် ၀ ကို ၉ ဟု ထားပါ။
s ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0704]: the block of "က" is edited, so its cells can move, here, but "s" still borrows it and is used later
```

Text works through the same paths, and so do plain lists, whose kinds
`holds_text` covers. A `text` slot owns the block of its u8s, so a read of a
text place roots at that block, as a read of an owner does, and a lend or a range of the
text is a borrow of it. A `Drop` of a text place gives the block back as a `Free` does,
and then leaves the slot blank, save a struct or a list with an owner in it: the drop frees
its text alone, so the slot keeps the facts of its owners for the leak checks. A view of text that is still live at the drop is ZK0704,
and a function that returns a view of its own local text is ZK0706. Text is no move kind:
a read of a text place into another place is a `Dup`, whose value holds blocks of its own,
so it points into nothing the place read points into.

A text constant holds static text, so a read of it is a duplicate, never a move:
`RootCtx::is_const` keeps `moved` and the moved arguments of a call from marking a
constant moved, and a read of the constant after a duplicate is no ZK0701.

Before the flow, `last_use.rs` turns each `Dup` of a whole place that the program reads
no more into a `Take`, so the check follows the code that runs. Liveness
(`live_out_of` with `holds_text`) gives the places live after each node; a call reads
each global of a text kind its function reads, at each depth. A `Dup` remains for a
parameter; for a global read in a function; for a global that a call of its statement
sets, since the check names a global's block after the global and would see the moved
text freed; for a place that a lend, a range or an address reads in some routine; and for
a place its statement reads twice.

After the moves, `lend_arguments` marks each `Dup` left in a call argument `lent`, so the
callee reads the caller's u8s, or the cells of a plain list, with a room of 0, unless the call, or a call inside its
arguments, sets the global read (`global_writes`, at each depth), or the place is read
through a pointer, or a lend, a range or an address reads it. The same pass lends the
`Dup` a loop starts from, such as the source of an each loop (`lend_loop_dup`), when
the loop reads the duplicate through its cells alone and leaves the source as it is: no
statement of the loop writes it, each read of it is the base of a cell, a field or a
character or the value of another `Dup`, no call in the loop sets it, and no lend, range
or address reads it. A lent `Dup` is fresh for the check, as a `Dup` is. With `--warn-duplicates`, each `Dup` left that is not lent is
ZK0720.

Last, `share_params` fills `Func::shared_params` with each list or struct parameter that
holds no text, plain list, owner or borrow and that the function never sets, stores into, moves out
of, drops, views or takes the address of; codegen reads such a parameter in the caller's
storage. A call gets a plain `Dup` of an argument for such a parameter when the call, or
a call inside its arguments, writes the global read, or a lend, a range or an address
reads the place.

## The checks

Each file of `checks/` owns some codes. All of them work on a `Flow` and the `State`.

| File | Raises | Finds |
| --- | --- | --- |
| `moves.rs` | ZK0701 | a read of an owner that is `MOVED` on each path, also in a later round of a loop (`check_loop_moves`) |
| `static_rules.rs` | ZK0702, ZK0703 | a borrow given back or resized; a borrow stored in a block, through a pointer, or in a global from a function |
| `loans.rs` | ZK0704 | a block lost while a live borrow can point into it |
| `escape.rs` | ZK0706 | a function that returns a borrow of its own local or parameter |
| `calls.rs` | ZK0704, ZK0707 | the effect of a call (below); one block freed through one argument and reachable through another |
| `leaks.rs` | ZK0708 | an owner that holds a block when it is overwritten, or when its function ends; text is never lost, since the compiler drops it, but an owner beside it in a struct or a list is checked as in each other struct; the cells of a plain list of owners at a remove, an overwrite and the end of a function (`PtrInfo::cells`) |
| `nullness.rs` | ZK0709, ZK0711 | a dereference of an owner that is `NULL` (or `FRESH`, in strict mode) |
| `bounds.rs` | ZK0710, ZK0712 | an offset or cell number proven outside its block; a pointer difference between different blocks |
| `lists.rs` | | the facts of a list of pointers, one fact for all its cells |

The move from the user's point of view is shown in [Pointer checks](../compiler/checks.md).
A move into a call is the same check as a move into a variable:

```zaka-error ZK0701
လုပ်ဆောင်ချက် ရှင်းပစ် သည်
ကိန်းပြည့် လိပ်စာ က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လိပ်စာ က ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
လုပ်ဆောင်ချက် ရှင်းပစ် ကို ပ တို့ဖြင့် လုပ်ဆောင်ပါ။
ပ ၏ တန်ဖိုး ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0701]: "ပ" was moved to another owner and now holds ဗလာ
```

The parameter of `ရှင်းပစ်` is a plain owner, so the call takes the block and `ပ` becomes
`MOVED`. The checker sees this without looking inside `ရှင်းပစ်`.

### Intervals and bounds

Integer slots hold an `Interval`, and arithmetic on intervals (`add`, `sub`, `mul`, with an
overflow giving the whole range) gives the offset of a pointer and the number of a list
cell. A counted loop gives its variable the range of the bounds. `check_offset` and
`check_cell` report ZK0710 when the interval of an offset lies outside the block altogether.
When a cell address is proven in range, its span goes to `CellProofs`; at the end,
`mark_checked` sets `checked: true` on those `CellAddr` nodes, and code generation leaves
the run-time `index` op out for them.

A slice made with a length of one known value keeps it as the length of its `PtrInfo`, at
offset 0, so its cells are checked as the cells of a list are, ZK0710 included. For a
slice whose count is known at run time alone, `State` keeps `below`: pairs of an integer
local and a slice local, added on the edge where `n < s ၏ အကွက်အရေအတွက်` holds, kept at a
merge when both sides hold them, and dropped on each write to either slot. A cell
`s ၏ အကွက် n` is proven when `n` is at 0 or above and the pair holds; code generation
then reads the first word of the slice and steps to the cell, with no `slice_cell`. This
is done just when the check found no error and
the program uses no raw address, since a raw address can change a fact the proofs rest on.

With `--overflow stop`, the default (`BorrowCheckOptions.overflow`), `check_overflow` works out the
interval of each integer `+`, `-`, `*` and negation from the intervals of its operands,
before it is cut to the kind. When that interval is bounded and lies outside the range of
the kind, each value the operands can hold overflows, and the check reports ZK0719, a
warning: the VM stops there each time the program reaches it.

## Calls: digests

Inside a function the checker cannot see its callers. It assumes the parameters are
distinct, runs the flow, and writes a *digest* (`FnDigest`), a short record of what the function
can do to the storage it reaches:

| Field | Plain words |
| --- | --- |
| `frees` | the parameter and global paths it can free, resize or move out, through hooks too |
| `unfilled` | paths it freed and did not fill again, so the owner there can be `ဗလာ` after |
| `result_from`, `result_roots` | which parameters or globals a borrow result can point into |
| `renewed` | result paths that point into a block the function made or resized itself |
| `reads` | the paths it reads or writes through, also in its callees |
| `result_roots_ok` | whether the result points into nothing that dies with the call |

At a call, `check_call` translates the digest through the real arguments (a path from
parameter 1 becomes a path from the slot that was passed) and treats it as the loss of
those blocks, then checks the losses against the loans as for each node. A call that frees
a global under a live borrow is ZK0704 at the call; two arguments that reach one block the
function frees through the first are ZK0707.

Digests are computed in groups of functions that call each other (`callgraph.rs`), callees
first; a group that calls itself runs again until none of its digests grows (a fixpoint),
so recursion and mutual recursion are covered. The global borrows add a loop around this:
a global borrow whose starting value comes from a call takes its roots from a digest, and a function
that reads the global starts from those roots. `roots_and_digests` repeats until the roots
stop growing. Then `main` runs last. `zakac borrow-check` prints the digests; for
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

## Options

`BorrowCheckOptions { check: MemCheck, strict: bool }`, chosen with `--mem-check` and
`--mem-strict` on `borrow-check`, `codegen` and `run`.

| Option | Effect |
| --- | --- |
| `MemCheck::Error` (default) | each finding is an error and stops code generation |
| `MemCheck::Warn` | each finding is made a warning (`downgrade`); code generation goes on and the VM stops the program at the bad access |
| `MemCheck::Off` | the findings are dropped; the flow still runs |
| `strict` | also reports what cannot be proven safe: ZK0711, a dereference of a new allocation never compared with `ဗလာ`, a leak of a counted list of owners, and a leak of a plain list of owners whose cells just can hold a block |
| `--no-raw` | not an option of this crate: `SemcheckOptions::no_raw` refuses each raw kind (ZK0717) before the IR exists |

```zaka-warning ZK0704 mem-check=warn
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ။
ကိန်းရှင် မိတ္တူ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ပ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
မိတ္တူ ၏ တန်ဖိုး ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

Just the codes decided in this crate (ZK0701 to ZK0704, ZK0706 to ZK0712) change with the
mode. The kind rules ZK0705 and ZK0713 to ZK0717 are decided in
[the kind check](semcheck.md) and are errors in each mode. The choice to mark proofs is made
before `downgrade`, on the raw findings, so a program with a finding keeps all its
run-time checks, whatever the mode. A move leaves `ဗလာ` behind in each mode, so a program does the same
thing whichever mode compiled it.

## Tests and extending

`tests/` has a file for each group (`moves.rs`, `loans.rs`, `calls.rs`, `leaks.rs`,
`nullness.rs`, `bounds.rs`, `lists.rs`, `escape.rs`, `free_rules.rs`, `modes.rs`), golden
`.zkbckt` files for clean inputs, `still_works.rs` for programs that look unsafe and are
correct (two pointers writing into one block, an out-parameter, a walking borrow), and
`oracle.rs`, which runs each program that passes the check in the VM and asks for no
use after free, double free or bad free.

- **A new mistake**: a check in `checks/`, called from the right place in `Flow::stmt` or
  `node_outs`; a `ZK07` code, texts and a constructor in `zakac_diagnostics`; a test that
  fires it and a test that a correct close program remains clean.
- **A new fact**: a field of `PtrInfo` or a form of `SlotState`, joined in `join` and
  `widen`, or the pass will not end.
- **A new instruction in the IR**: `walk.rs` (`each_expr`, `each_stmt`) must visit it, or
  the liveness pass and the call graph will miss it.

The next stage, which also reads this module, is [IR and code](codegen.md).
