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
`Convert`, a move out of a place is a `Take`, a borrow is a `Lend`, and a heap sentence is
an `Alloc`, `Resize` or `Free` with a destination. For this program:

```text
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ။
ကိန်းရှင် မိတ္တူ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
မိတ္တူ ၏ တန်ဖိုး ကို ပြသပါ။
```

`zakac irgen` shows six statements (spans and kinds left out): `set g0 = const 0x0` (a new
owner starts at `ဗလာ`), `alloc g0 count 1 unit isize`, `store (read g0) 7`,
`set g1 = lend (read g0)`, `free g0`, `print load (read g1)`. The checker works on just
these.

## What it keeps: the State

Each statement list (a function, or the main program) is a *routine*. At each point of a
routine the checker keeps a `State`: one fact for each global and each local, a
`SlotState`.

| Fact | Holds |
| --- | --- |
| `Int(Interval)` | an integer slot, as a range `lo..=hi` of the values it can have |
| `Ptr(PtrInfo)` | an owner or a borrow |
| `List(PtrInfo)` | a list of pointers, one fact for all its cells |
| `None` | a slot the check does not follow |

A `PtrInfo` is `{ state, roots, offset, len }`. `state` is an `OwnerState`, `roots` names
where a borrow points, `offset` is an interval of element offsets from the start of the
block, and `len` is the element count of the block when known.

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
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ။
ကိန်းရှင် မိတ္တူ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
မိတ္တူ ၏ တန်ဖိုး ကို ပြသပါ။
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

## The checks

Each file of `checks/` owns some codes. All of them work on a `Flow` and the `State`.

| File | Raises | Finds |
| --- | --- | --- |
| `moves.rs` | ZK0701 | a read of an owner that is `MOVED` on each path, also in a later round of a loop (`check_loop_moves`) |
| `static_rules.rs` | ZK0702, ZK0703 | a borrow given back or resized; a borrow stored in a block, through a pointer, or in a global from a function |
| `loans.rs` | ZK0704 | a block lost while a live borrow can point into it |
| `escape.rs` | ZK0706 | a function that returns a borrow of its own local or parameter |
| `calls.rs` | ZK0704, ZK0707 | the effect of a call (below); one block freed through one argument and reachable through another |
| `leaks.rs` | ZK0708 | an owner that holds a block when it is overwritten, or when its function ends |
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
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
လုပ်ဆောင်ချက် ရှင်းပစ် ကို ပ တို့ဖြင့် လုပ်ဆောင်ပါ။
ပ ၏ တန်ဖိုး ကို ပြသပါ။
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
the run-time `index` op out for them. This is done just when the check found no error and
the program uses no raw address, since a raw address can change a fact the proofs rest on.

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
a global borrow set at top level from a call takes its roots from a digest, and a function
that reads the global starts from those roots. `roots_and_digests` repeats until the roots
stop growing. Then the main program runs last. `zakac borrow-check` prints the digests; for
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
| `strict` | also reports what cannot be proven safe: ZK0711, a dereference of a new allocation never compared with `ဗလာ`, and a leak of a list of owners |
| `--no-raw` | not an option of this crate: `SemcheckOptions::no_raw` refuses each raw kind (ZK0717) before the IR exists |

```zaka-warning ZK0704 mem-check=warn
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ။
ကိန်းရှင် မိတ္တူ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
မိတ္တူ ၏ တန်ဖိုး ကို ပြသပါ။
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
