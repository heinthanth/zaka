---
title: The flow checker
description: How the flow checker follows pointers, integers and unset values through the IR, the facts it keeps, the check that each value is set before it is read, the lint it runs on unsafe code, and the options that change what it reports.
---

# The flow checker

The flow checker (borrow checker) finds mistakes before the program runs. On each part of
a program, it follows unset values, cell numbers (indexes) and integer ranges, so it can
report a read of a variable, a field or a cell before the program gives it a value, a
cell number past the end of a list, and an operation that overflows each time it runs. On
unsafe code, it also runs a *lint* on pointers: a block read after it was freed, a block
freed twice, a block that is never freed, and a pointer read after an owned hand-over.
[Unsafe Zaka](../language/unsafe.md) and [Compiler checks](../compiler/checks.md) state
the rules from the side of a program, and [Storage, explained](../concepts/storage.md)
gives the reasons. This page is about the code in `compiler/zakac_borrow_checker`, and
the crate keeps its old name.

The flow checker is the first line of defence. The VM is the second: it checks each read
and write while the program runs (see [When a program stops](../vm/traps.md)), so a
mistake the flow checker cannot prove still stops the program at the bad access. A test
keeps the two in step: each sample that passes the flow checker, with no raw address in
it, must run on the VM with no use after free, double free or bad free.

## Where it runs

```text
 tree ──▶ names and data kinds ──▶ lower ──▶ fold ──▶ FLOW CHECK ──▶ codegen
                                                    │
                       Module (folded IR) ──────────┤  borrow_check(&mut Module, &options, platform_bits)
                                                    ▼
                          BorrowCheckOutput { diagnostics, digests }
                          and `checked: true` set on the cell addresses it proved safe
```

The flow checker takes in the folded IR and returns its diagnostics (`ZK07` codes) and a
digest of each function. It also changes the module in place: it marks the cell
addresses it proved in range, turns some duplicates into moves and lends others, and
fills the list of parameters a function can read in the caller's storage. Code
generation reads all of these.

The IR is the right input because it has made explicit, before this stage, what the flow
checker needs. Names are slots (`g0` for a global, `l1` for a local), blocks are
flattened, widening is a `Convert`, a hand-over out of a place is a `Take`, a heap
sentence is an `Alloc`, `Resize` or `Free` with a destination, and the end of a counted
place is a `Drop`. `Module.unsafe_spans` lists the source regions of unsafe code: the
bodies of unsafe functions and of unsafe blocks.

## An example

Take these lines inside an unsafe block:

```text
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားမည်။
ကိန်းရှင် q သည် ပ ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
q ၏ တန်ဖိုး ကို ပြသမည်။
```

`zakac irgen` shows six statements for them, written here in short form with the spans
and data kinds (types) left out:

```text
set l0 = const 0x0
alloc l0 count 1 unit isize
store (read l0) 7
set l1 = read l0
free l0
print load (read l1)
```

The declaration gives no value, so the slot `l0` notes the span of the first `set` in
`unset` (see [Set before read](#set-before-read)). The flow checker works on these six
statements alone. At the `free`, the block of `ပ` is lost while `q` still points into it
and is read on the next line, so the flow checker reports ZK0704:

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

The finding is a warning, because `--lint-warn mem` is the default, so the program
still runs. On the VM (`zakac run --vm`), the run stops at the read of `q` with
`access ... inside a freed block`; native code, which `zakac run` picks when it can, does
not check the access. If we swap the last two statements, the program is correct, because
`q` is no longer live at the free. The rest of this page explains how the flow checker
gets from the six statements to that warning.

## What it keeps: the State

Each statement list is a *routine*: a function, the program start block (entry point), or
`main`, which has the starting values of the top-level variables and the call of the
program start block. At each point of a routine, the flow checker keeps a `State`, with
one fact, a `SlotState`, for each global and each local:

| Fact | What it describes |
| --- | --- |
| `Int(Interval)` | an integer slot, as a range `lo..=hi` of the values it can have |
| `Ptr(PtrInfo)` | a pointer, a shared object (reference type) or a function value; a text slot is followed the same as a pointer to its block |
| `List(PtrInfo)` | a list of pointers, with one fact for all its cells |
| `None` | a slot the flow checker does not follow |

The local slots of a `State` are an `imbl::Vector`, because the flow checker keeps a state
at each node of the routine. A node shares each slot it leaves alone with the node
before it, and the join and the widen of two states change just the slots that differ,
so the storage no longer grows with the square of the locals.

The program start block is checked as a function, with one difference. Other functions
start from globals the flow checker knows nothing about, while `main` calls the program
start block once, as its last statement, and nothing else can call it. So
`run_start_flow` runs it again from the facts `main` has at that call (`Flow::seed`), with
the hidden host globals not known, since the VM fills them first. Its diagnostics and
cell proofs come from that run, and its digest comes from the run with nothing known.

### Pointer facts

A `PtrInfo` is `{ state, roots, offset, len, cells }`:

- `state` is an `OwnerState` (below).
- `roots` lists where a pointer can point when the block is not its own (see [Roots and
  paths](#roots-and-paths)).
- `offset` is an interval of element offsets from the start of the block.
- `len` is the element count of the block, when known.
- `cells`, for a plain list of pointers, is the join of what its cells contain. An add
  joins the state of the pointer added, and a literal gives the join of its cells. A
  write through a cell, or a call that can reach the list, makes it `UNKNOWN`, since the
  flow checker does not follow cell numbers.

A plain list slot is `HELD` after an add, a remove or a character edit, since each of
these leaves it with its own block.

The data kinds of the flow checker have two flavours of pointer that no source spells.
The IR marks the addresses it makes itself (the address of a `ဒေတာ ပြောင်းလဲရန်`
argument, the boxes of a closure record) as borrowed, so the flow checker never takes
them for blocks to free. Each pointer the program declares is a plain pointer.

### Pointer states

`OwnerState` is a set of facts in one octet (byte), joined with a bitwise or:

| Fact | Meaning |
| --- | --- |
| `NULL` | the slot contains the null address, `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` |
| `HELD` | the slot points to a block |
| `FRESH` | the slot has a new allocation never compared with the null address: a block or null, as the heap can run short |
| `MOVED` | the slot handed its block to an owned parameter; it contains the null address, and a read is a mistake |
| `GIVEN` | a mark, not a fact: a call could have freed, resized or taken the block, so the slot is not reported as a leak |

```text
                  alloc, resize                test: not null
        NULL ───────────────────▶ FRESH ───────────────────▶ HELD
         ▲  ▲                       │                        │   │
         │  └──── test: is null ────┘                        │   │ owned hand-over
         │                                                   │   ▼
         └─────────────────────── free ──────────────────────┘ MOVED
```

A pointer declared with `ကနဦးတန်ဖိုး` or with the null address starts at `NULL`, and so
does one declared with no value, which [the init check](#set-before-read) also reports
when it is read before a write. An allocation or a resize makes it `FRESH`, and a free
sets the slot back to `NULL`.

An argument for a parameter marked `အပိုင်အနေဖြင့်` is lowered as a `Take` (`hand_over`
in the IR builder), which leaves the caller's place `MOVED`. A `Take` of text, a list or a
function value leaves the place blank and not known, since a blank one reads the same as
each value does. A read of a moved pointer sees `NULL` (`as_read`), and the one read
that is still allowed is the test against the null address. Where two paths meet, the sets
are joined, so after an `if` that hands the block over on one branch alone, the state is
`HELD | MOVED`.

The reports read the joined state with care:

- ZK0701 (a read after an owned hand-over) fires when the state is `MOVED` and nothing
  else, which means on each path to the read.
- ZK0709 (a read through the null address) fires when the state is `NULL` and nothing
  else, and on a pointer slot alone (`check_deref` in `checks/nullness.rs`), because a
  shared object and a function value are never null.

A mistake that happens on some paths alone is left to the VM. This rule keeps the flow
checker free of false alarms on a correct program: it reports "is" and not "can be". The
strict mode ([Options](#options)) adds the "can be".

A test against the null address narrows the state on each edge of the branch (`narrow`).
After `အကယ်၍ ပ သည် မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ နှင့် ညီလျှင်`, the "then" edge keeps the
facts that allow null and the other edge keeps the facts that allow a block, so a `FRESH`
pointer becomes `NULL` on the first edge and `HELD` on the second.

## Roots and paths

A pointer into a block it did not make does not name the block. Instead, the flow checker
gives each such pointer a set of *roots*: the places it can point into. A `Root` is a
slot and a *path* of steps, where each step is `Block` (the block a pointer points to)
or `Cell(n)` (one cell of a list, `None` when the number is not known):

| Expression | Root |
| --- | --- |
| `x ၏ လိပ်စာ` | `x`, the slot itself |
| `ပ + ၁`, or a duplicate of `ပ` | `ပ.block` |
| `ခ ၏ တန်ဖိုး + ၁` | one `block` deeper than what `ခ` points at |
| `l ၏ အကွက် ၁ ၏ လိပ်စာ` | `l.cell[1]` |
| a cell with an unknown number | `l.cell[*]`, the whole list |

Arithmetic, a conversion (cast) and a duplicate (copy) of a pointer *derive* the new
pointer from the old one (`derived`, `escape_dup`). The new one roots at the block, and
the block is still the old one's to free, so a duplicate is not a second block to leak.
A pointer that contains the address of a known place keeps its roots (`holds_address`,
`Root::reaches`).

A free or a resize of `ပ` gives up the path `ပ.block`. That path *reaches* a root when
it is a prefix of the root, and an unknown cell matches each cell. So a pointer to the
slot `ပ` survives a free of the block of `ပ`, while a pointer into the block does not.
Paths are cut at four steps (`MAX_PATH`). A cut path is a shorter prefix, which reaches
more, so the cut errs on the safe side.

## One routine: the flow

`run_flow` takes a routine through five steps (`flow.rs`):

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

**The graph** (`cfg.rs`) has one node for each statement with no statement nested in it,
one for the test of an `if` or a loop (with two out-edges: the test is true, or it is
false), a pair for a counted loop, and an `End` and an `Exit` node. `break`, `continue`
and `return` link to the right place.

**Liveness** (`liveness.rs`) is the backward pass. A pointer slot is live after a node
when some path from there reads it before it is written again. Because the pass looks at
all paths, it finds a read that runs once a month as well as one that runs each time.

**The solve** is the forward pass. `transfer` turns the state before a node into one
state for each successor edge. Integers are intervals, and a loop head *widens*: when a
bound moved since the last visit, it jumps to the end of the range, so the pass ends. The
facts are recorded but nothing is reported so far, because a fact seen in the first round
of a loop can still change.

**The report pass** runs `transfer` again over the final facts with `reporting` on. Now
each check in `checks/` can raise a diagnostic, and `report` keeps one for each code,
span and message.

### Losses

A free, a resize or a hand-over is a *loss* of a block (`Loss`, with the paths that lose
it). When a node ends, `check_losses` takes each loss and asks whether a *live* pointer
slot has a root that the lost path reaches. If one does, the result is ZK0704, with a
second label on the witness read from the liveness pass. That is how the
[example](#an-example) gets its warning: `q` roots at `ပ.block`, the free loses
`ပ.block`, and `q` is live because line 8 reads it.

A `Blit` or a `Fill` is checked as a write and a read through an address. A `Fill` and the
destination of a `Blit` go to `write_cells`, which treats the address the same as a
`Store` does: a destination that is the null address on each path, or whose first cell lies
past the end of its block, is a finding, and the flow checker forgets the facts of the
slots the address names. The source of a `Blit` goes to `read_cells`, a read through the
address. Both also check the last cell (`check_last_cell`): the octets of the run, from
`octets`, at the end the count reaches on some path, rounded up to whole cells of the
pointee. A run of no octets checks neither address. For a `ကူးယူမည်`, `check_overlap`
reports ZK0721 when both addresses name one block (`blocks_of`, as for ZK0712), both
offsets and the count are single values, and the octet ranges share an octet; otherwise
the VM's trap `overlap` remains.

A write into a plain list is a loss too, of the pointers into its cells. Lowering wraps
each plain list on the path of a written place in an `Own` node, since the write can first
move its cells to its own block. For a `Store`, `owned_lists` finds the place of
each such list, and `check_heap` runs with `Invalidation::Edited`. A pointer into the
cells that is still live is ZK0704, with "is edited, so its cells can move," in place of
"is freed". An `AddCell`, an `Insert`, a `Replace` or a `RemoveCell` is a loss as well,
because an add can move the cells to a larger block and a remove drops the last cell.
`SetChar` and `AddText` edit text in the same manner. The address of the cells of a list
or of text pins its block (`Own { pin: true }`), so a duplicate of the list made later
clones the cells and a write through the pointer never reaches it. The edits above can
still move the pinned cells, and the flow checker sees them as losses.

### Moves and loans before the flow

Before the flow, `last_use.rs` turns each `Dup` of a whole place that the program does not
read again into a `Take`, so the flow checker follows the code that runs, and the VM
retains and releases less. A `Dup` remains for:

- a parameter;
- a global read in a function;
- a global that a call of its statement sets;
- a place that a range or an address reads in some routine;
- a place its statement reads twice.

A deep `Dup`, from a deep duplicate, is never turned into a `Take`.

After the moves, `lend_arguments` marks each `Dup` left in a call argument as `lent`, so
the callee reads the caller's octets, or the cells of a plain list, with a room of 0 and
no retain. It does not lend when the call, or a call inside its arguments, sets the
global it reads (`global_writes`, at each depth), when the place is read through a
pointer, or when a range or an address reads it.

A call to C counts as a call that sets each global (`calls_c`), because C can call back
each function of the program, and in a C lib another OS thread runs calls while the call
waits in C, much as Rust keeps a loan from living across a call that can change what
it borrows. So a function that calls C, at each depth, receives each global as a
duplicate, an each loop whose block reaches C walks a duplicate of a global, and the flow
forgets the facts of each global after a call to C (`forget_globals`). A box, which
contains a captured local, is never lent.

The same pass lends the `Dup` a loop starts from, such as the source of an each loop
(`lend_loop_dup`), when the loop reads the duplicate through its cells alone and leaves
the source as it is. Last, `share_params` fills `Func::shared_params` with each list or
struct parameter that contains no text, plain list or pointer and that the function never
sets, stores into, drops or takes the address of. Code generation reads such a parameter
in the caller's storage.

## The checks

Each file of `checks/` owns some codes, and all of them work on a `Flow` and the `State`:

| File | Raises | Finds |
| --- | --- | --- |
| `moves.rs` | ZK0701 | a read of a pointer that is `MOVED` on each path, after an owned hand-over, also in a later round of a loop (`check_loop_moves`) |
| `static_rules.rs` | ZK0702, ZK0703 | a free or a resize through the address of a variable, or of a cell or field it contains in place (each root with no block step); the address of a variable of a function (a root at a local slot with no block step) stored in a heap block, through a pointer, or in a global from a function |
| `loans.rs` | ZK0704 | a block lost while a live pointer can point into it |
| `escape.rs` | ZK0706 | a function that returns the address of its own local or parameter |
| `calls.rs` | ZK0704, ZK0707 | the effect of a call ([Calls: digests](#calls-digests)); one block freed through one argument and reachable through another |
| `leaks.rs` | ZK0708 | a block this function made (`FRESH`) that is overwritten, or still held when the function ends, and was not freed, handed to an owned parameter, returned as an owned result, or stored into a heap block, a global or a field; an unmarked return of such a block gets the help to mark the result `အပိုင်အနေဖြင့်` |
| `nullness.rs` | ZK0709, ZK0711 | a read through a pointer that is `NULL` (or `FRESH`, in strict mode); a shared object and a function value are never null, so it skips them |
| `bounds.rs` | ZK0710, ZK0712 | an offset or cell number proven outside its block; a pointer difference between different blocks |
| `lists.rs` | | the facts of a list of pointers, one fact for all its cells |

Two checks live outside `checks/` and keep their own facts:

- `threads.rs` raises ZK0726, the globals rule of threads. A function that a thread or a
  parallel map runs, itself or through each function it calls, must read and write no
  mutable global. The check finds the functions a spawn starts in, walks their callees
  (`callees`, through function values to each function whose address the module takes),
  and reports the first read, address or write of a global that is not a constant in
  each one. It runs once the lint levels are set, so ZK0726 is an error at each level.
- `init.rs` raises ZK0722 to ZK0724 (see [Set before read](#set-before-read)).

The drop that the compiler writes at the end of a scope (`Stmt::Drop`) does not run the
bad-free rule of `static_rules.rs`. It frees no block a variable contains, since a range
of a counted list on the stack owns none, so `week ၏ အကွက် ၂ မှစ၍ ၂ ကွက်` no longer warns
ZK0702 at the first line of the program start block. A free or a resize that the program
writes keeps the rule.

An owned hand-over into a call is a `Take` of the argument's place, so the flow checker
follows it the same as each other statement:

```zaka-error ZK0701
ရှင်းပစ် ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လိပ်စာ က ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ရှင်းပစ် ဟူသော လုပ်ဆောင်ချက်ကို ပ ဖြင့် လုပ်ဆောင်မည်။
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0701]: "ပ" handed its block to a parameter marked "အပိုင်အနေဖြင့်" and now points at nothing
```

The parameter of `ရှင်းပစ်` is marked, so the call takes the block and `ပ` becomes
`MOVED`. ZK0701 is not a lint, so it is an error in each mode, and the flow checker sees
it without looking inside `ရှင်းပစ်`. `Func.owned` (from `Frame.owned_ret` in the IR
builder) records an owned result, which the caller receives as its own block, the same
as a fresh allocation.

### Intervals and bounds

Integer slots have an `Interval`, and arithmetic on intervals (`add`, `sub`, `mul`, with
an overflow giving the whole range) gives the offset of a pointer and the number of a
list cell. A counted loop gives its variable the range of its bounds. `check_offset` and
`check_cell` report ZK0710 when the interval of an offset lies outside the block altogether.

When a cell address is proven in range, its span goes to `CellProofs`, and at the end
`mark_checked` sets `checked: true` on those `CellAddr` nodes, so code generation leaves
out the run-time `index` operation for them. This happens when the flow checker found no
error and the program uses no raw address, since a raw address can change a fact the
proofs rest on. The proof marks `CellAddr` alone, so a loop over the cells of a plain
list parameter keeps its run-time check (`SliceCell`, the internal operation of such a
cell).

With `--overflow stop`, the default (`BorrowCheckOptions.overflow`), `check_overflow` works
out the interval of each integer `+`, `-`, `*` and negation from the intervals of its
operands, before it is cut to the data kind. When that interval is bounded and lies
outside the range of the data kind, each value the operands can have overflows, and the
flow checker reports ZK0719, a warning: the VM stops there each time the program reaches
it.

## Calls: digests

Inside a function, the flow checker cannot see the callers. It assumes the parameters are
distinct, runs the flow, and writes a *digest* (`FnDigest`), a short record of what the
function can do to the storage it reaches:

| Field | What it records |
| --- | --- |
| `frees` | the parameter and global paths it can free, resize or take over, through hooks too |
| `unfilled` | paths it freed and did not fill again, so the pointer there can be null after the call |
| `result_from`, `result_roots` | which parameters or globals a pointer result can point into |
| `renewed` | result paths that point into a block the function made or resized itself |
| `reads` | the paths it reads or writes through, also in its callees |
| `result_roots_ok` | whether the result points into nothing that dies with the call |

At a call, `check_call` translates the digest through the real arguments (a path from
parameter 1 becomes a path from the slot that was passed) and treats it as the loss of
those blocks, then checks the losses against the live pointers, the same as for each
node. A call that frees a global under a live pointer is ZK0704 at the call, and two
arguments that reach one block the function frees through the first are ZK0707.

Digests are computed in groups of functions that call each other (`callgraph.rs`),
callees first. A group that calls itself runs again until none of its digests grows (a
fixpoint), so recursion and mutual recursion are covered. The globals add a loop around
this: a global pointer whose starting value comes from a call receives its roots from a
digest, and a function that reads the global starts from those roots. `roots_and_digests`
repeats until the roots stop growing, and then `main` runs last.

`zakac borrowck` prints the digests. For `ရှင်းပစ်` above:

```text
digests:
- name: ရှင်းပစ်
  params: 1
  frees:
  - place: l0
    path:
    - Block
  result_from: []
  result_roots: []
  renewed: []
  unfilled:
  - place: l0
    path:
    - Block
  reads: []
  result_roots_ok: true
```

The order of the statements inside the callee is not looked at, so a function that reads
a global and then frees it counts as a function that frees and reads. This costs a few
false alarms and closes a group of holes in return.

### Failures and stops

A call that passes a failure on (error propagation) is lowered to a plain return that
leaves the function (see [The IR](codegen.md#lowering)), so the flow checker needs no
separate rule for it: a block alive at the pass-on site is lost when the function leaves,
which is ZK0708 at the call. The IR statement `Halt` ends a path in the graph (`cfg.rs`),
with no leak check and no exit check there, because the run stops, so nothing is left to
free.

### Function values and closures

A call through a function value (`Stmt::CallValue`, `ExprNode::CallValue`) names no
function, but a function value can come from naming a function alone. `values.rs` gives
the functions the module names as values (`taken_fns`) and, of those, the ones whose
parameters and result fit the function kind of the value (`value_callees`,
`Func::fits_kind`). `Flow::call` takes a list of functions, runs `check_call`,
`note_unfilled` and `lift_reads` for each, and joins what each returns, and `call_roots`
joins their result roots. A function named as a value gets no shared parameters
(`share_params`), because a call through a value passes each argument in its own slot,
whichever function runs.

A closure is a function value with a record. The walk reads the record of
`ExprNode::Closure` and gives the value the fact `HELD`, and `addressed_fn` (`values.rs`)
counts its thunk as a taken function, so a call through a value reaches the thunk. The
record is a counted block of boxes, one for each captured local, and each box contains
its local for the routine and for the closure alike. `closures.rs` finds the closures of
each routine before its flow: the function each thunk calls and the addresses its record
passes (`lent_fields`). Which closures a slot can contain is a fact of the flow,
`State::closures`. `Flow::call` treats an argument that can contain a closure as a call
that can run its function (`closure_args`, `run_or_not`, `run_held`), and
`check_closure_args` reports ZK0707 for another argument that reaches a block the closure
can free. `callgraph.rs` puts the thunk of each closure a function makes before the
function, so the digests exist when the flow needs them.

A slot of a function kind gets a pointer fact (`SlotState::of_kind`, `fn_const`), `HELD`
for a function value, folded or not, so the walk knows which functions and closures it can
contain. No null check looks at it: a function value is never null, so no call through
one is reported, and the null check skips a shared object slot too. A function value or
an object read before it is set is for the [init check](#set-before-read) to report. A
cell of a counted list of functions or objects that the flow checker cannot prove set is
the warning ZK0723, and the VM's traps `null call` and `null object` remain as the
backstop for that cell.

### Host and C functions

An external function has one statement, `ExprNode::Host`. The flow reads each argument
and gives a value with no roots, a fresh one, and the digest of the function follows from
that statement, the same as for each other function. A C function has
`ExprNode::Foreign` (or `Stmt::Foreign`) in its place, which the flow reads the same as
the host form: each argument is a read, and the result is a fresh value. A list of
numbers passed to C goes as the address of its first cell, which the IR builds before the
call (`c_list_arg`), and a list built for a call that accepts other values waits in a
hidden slot, which the end of the scope drops.

## Set before read

`init.rs` contains a separate check, apart from the flow above: no variable, field or cell
is read before the program gives it a value. A declaration can leave the value out, and
no data kind fills one in. [Set before read](../language/declarations.md#set-before-read) states
the rules from the side of a program.

The IR marks the slots to follow. A declaration with no value still lowers to a `Set`
that gives the slot its zero octets, so the VM and C see the same slot, and the
slot notes the span of that statement in `Slot.unset`. The check follows these slots
alone (`followed`), so the slots the compiler makes for itself need no care, and the
`Set` whose span is the declaring one leaves its slot unset, where each other `Set` sets
it.

### Parts

The fact of a followed slot is `Unset { parts, doubt }`: the parts that can still be
unset, and whether a cell was written at a position the check does not know. A slot is
one part, a plain struct has one part for each field, and a counted list has one part for
each cell, up to `CELL_CAP` (256) cells, while a longer list is one part.

A struct inside a struct is followed one leaf field at a time. A plain struct has one part
for each leaf (`leaves`), and a field, at each depth, is the range of leaves it contains
(`field_leaves`, `Part::Field(first, count)`), which `part_of` follows through the `load`
of each outer field. A field of a slot that is not a plain struct, such as the count of a
plain list, is the whole slot (`plain_part`). A variable that a closure shares lives in a
box. Its slot is followed too, and a write through the box is a write of its one cell, so
a closure that captures it before it is set reads it unset.

The facts are solved over the graph of each routine (`cfg.rs`), the same as the flow's
facts: a join keeps each part that one path leaves unset. A second pass reports:

| Read | Report |
| --- | --- |
| the whole slot, or a field, a cell or the address of it, with a part that a path leaves unset | ZK0722 |
| the same, when a cell was written at a position the check does not know | ZK0723, a warning |
| a cell at a position the check does not know, while some cell is unset | ZK0723 when cells were set, ZK0722 when none was |

### Writes

A write sets a part. A `Set` sets the whole slot, a `Store` to a field or to a cell at a
constant position sets that part, and a `Store` to a cell at another position sets the
`doubt`. An allocation into a slot, a field or a cell sets that part, the same as a write
does (`alloc_into`). A `Drop` that the compiler adds, as before a text value moves into a
field, reads nothing, because an unset place contains zero octets, which drop as nothing.

A counted loop sets its cells at once, on the edge out of it (`fill`), when it has fixed
bounds, no `break`, and a statement at the top level of its block that writes the cell of
the counter (`written_cell`): it then sets each cell from the start to the end. Before
the solve, `earlier_reads` finds each read of `a[i-k]`, for a constant `k` of at least 1,
inside such a loop. The read is set when cells `s-k` to `s-1` are set (`earlier_set`),
and it is ZK0722 when one of them can be unset (`earlier_read`). Each other statement that
changes a slot in place, such as an add to a list, reads the slot first (`dest_slot`).

### Globals

Globals are checked at the calls of the top level and of the program start block, the two
routines that set them. `liveness.rs` gives the globals each function reads, itself or
through its calls (`global_reads_of`), and a call through a function value reads those of
each function the program names as a value (`taken_fns`). A call whose callee reads a
global that a path to it leaves unset is ZK0724 (`call`). The top level runs first and
keeps its facts at the call of the program start block (`check_main`), and the program
start block starts from them. Each other function treats the globals as set, since each
call of it is checked, and a write of a global inside a function does not count as
setting it for its callers.

### Writable parameters

A writable parameter (`out`), marked `ဒေတာ ထည့်ပေးရန်`, is followed too. The IR builder
notes its span in `Slot.unset` on the parameter slot, so `routine_slots` takes it among
the followed slots and keeps it apart as a filled place (`fills`). The routine starts with
it unset, so a read before a write is ZK0722. At each return and at the end of the
routine, `check_fills` reports ZK0725 (`fill_unset_at_return`) for each filled place with a
part a path leaves unset.

At a call, `fills_param` tells a writable parameter apart from the `unset` of the
callee's slot. The argument reads nothing but the cell number of its address, and after
the call `write_fills` sets the place it names, for a call statement and, through
`fill_args`, for each call inside an expression. An argument of a mutable parameter
(`inout`, marked `ဒေတာ ပြောင်းလဲရန်`) is read the same as each other argument, so a place
that can be unset there is ZK0722. The function that wraps a C function has one
statement, the call into C (`calls_c`). The check cannot see what C writes, so it trusts
the mark and follows none of the writable parameters there.

### How the reports join the others

The diagnostics of this check join the others before `downgrade`, but no level of the
`mem` lint changes them, because none of them is in the lint: the data kinds promise
there is no NULL and no hidden default, and that promise holds at each level. A pointer
read before it is set also contains the zero octets of its declaration, the null address,
so the flow can report ZK0709 or ZK0711 at the same read. `borrow_check` drops those, and
the report of the unset read covers both.

### ဇယား statements

`set_row` and `remove_row` count as edits of a list on the ဇယား (map) the statement
changes, the same as an add and an insert do. The rows can move, so each loan into them
ends, and the ဇယား counts as written for the checks of last uses, of liveness and of set
before read. The address of a row is never lent, because a write inside a row reaches it
through the rows, the same as a cell write does.

## Options

```text
BorrowCheckOptions { check: MemCheck, strict: bool, overflow: OverflowMode, lint: LintLevels }
```

We choose these options with `--overflow` and with `--lint-allow`, `--lint-warn` and
`--lint-error` on `borrowck`, `build` and `run`. `MemCheck` is another name for
`LintLevel` (`Error`, `Warn`, `Off`). `LintLevels` (in `zakac_diagnostics::lint`) has an
`Option<LintLevel>` for each code of `LINT_CODES`: the lint, ZK0719, ZK0723 and the
naturalness codes. `lint_levels` (in `zakac_cli_common::lints`) fills it from the three
flags, the groups (`LintGroup`) first and the codes after them, each in the order of the
command line, and a value from a config file comes before each one on the command line.
`check` keeps its default, `Warn`, as the level of the lint that no name sets, and a name
for the group `mem` wins over it. `strict` is on when a name sets the group `doubts` to
`warn` or `error`.

| Option | Effect |
| --- | --- |
| `MemCheck::Warn` (default) | each lint finding becomes a warning (`downgrade`); code generation goes on, and the VM stops the program at the bad access |
| `MemCheck::Error` | each lint finding is an error and stops code generation |
| `MemCheck::Off` | the lint findings are dropped; the flow still runs |
| `strict` | also reports what cannot be proven safe: ZK0711, a read through a new allocation never compared with the null address |

`downgrade` sorts each finding into the lint or not:

| Set | Codes | Under the level of `mem` |
| --- | --- | --- |
| `LINT` | ZK0702, ZK0703, ZK0704, ZK0706, ZK0707, ZK0708, ZK0712, ZK0721 | follow the mode wherever the flow checker finds them; these come from pointers, which unsafe code alone works with |
| `UNSAFE_LINT` | ZK0709, ZK0710, ZK0711 | follow the mode inside a region of `Module.unsafe_spans`, and remain errors in safe code, where a cell number past the end of a list gives ZK0710 |
| the rest | ZK0701, ZK0719 | as their codes define them, in each mode |
| the init check | ZK0722, ZK0723, ZK0724 | outside the lint, so no mode changes them: ZK0722 and ZK0724 are errors and ZK0723 is a warning in each mode |

A level set through a name wins over the mode. It reaches an `UNSAFE_LINT` code inside
unsafe code alone, and ZK0719 and ZK0723 wherever the flow checker finds them, while
`NotALint` refuses each other code at the command line. With `lint-error=mem`, the
example from the top of the page no longer compiles:

```zaka-error ZK0704 lint-error=mem
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

The rules of unsafe code itself (ZK0468 to ZK0472: an unsafe operation outside unsafe
code, an unsafe function as a value, an owned marker on something that is not a pointer,
`--no-unsafe`, and a C declaration that needs `စိတ်မချရသော`) are decided in [Names and data
kinds](semcheck.md) and are errors in each mode. The choice to mark proofs is made before
`downgrade`, on the raw findings, so a program with a finding keeps all its run-time
checks, whatever the mode. A hand-over leaves the null address behind in each mode, so a
program does the same thing whichever mode compiled it.

## Tests and extending

`tests/` has a file for each group (`moves.rs`, `loans.rs`, `calls.rs`, `leaks.rs`,
`nullness.rs`, `bounds.rs`, `lists.rs`, `escape.rs`, `free_rules.rs`, `closures.rs`,
`last_use.rs`, `unset.rs`, `modes.rs`, `fixes.rs`), golden `.zkbckt` files for clean
inputs, and `still_works.rs` for programs that look unsafe and are correct. `oracle.rs`
runs each program that passes the flow checker on the VM and expects no use after free,
double free or bad free. Some of these tests still state the rules of the earlier model
and wait to be rewritten around blocks.

- **A new mistake**: a check in `checks/`, called from the right place in `Flow::stmt` or
  `node_outs`; a `ZK07` code, its texts and a constructor in `zakac_diagnostics`; its
  place in `LINT` or `UNSAFE_LINT` when it is a lint; a test that fires it, and a test
  that a correct program close to it remains clean.
- **A new fact**: a field of `PtrInfo` or a form of `SlotState`, joined in `join` and
  `widen`, or the pass will not end.
- **A new statement or expression in the IR**: `walk.rs` (`each_expr`, `each_stmt`) must
  visit it, or the liveness pass and the call graph will miss it.

The next stage, which also reads this module, is [The VM target](vm-target.md), and the
IR itself is described in [The IR](codegen.md).
