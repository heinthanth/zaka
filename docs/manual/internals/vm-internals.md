---
title: Inside the VM
description: How zaka_vm runs the operations of a program, its threads and channels, the ring collector, its traps, the host functions, and how a call of a C function passes through each stage.
---

# Inside the VM

The VM (virtual machine), in the crate `zaka_vm`, runs the `bc::Module` that [the VM
target](vm-target.md) writes, one operation (instruction) at a time. This page follows
the Rust code of the VM: the run loop, calls, storage (memory), threads, the ring
collector and traps, then the host functions it serves and the path of a call of a C
function through each stage. The pages [Running a program](../vm/running.md),
[Instructions](../vm/instructions.md), [Addresses and blocks](../vm/blocks.md) and [When
a program stops](../vm/traps.md) describe the VM from the side of its user.

## The run loop

`zaka_vm::machine::Machine` owns a `bc::Module` and runs it one operation at a time.
`step()` fetches the operation at the program counter of the innermost call, executes it
with `exec`, and returns a `State`:

| State | Meaning |
| --- | --- |
| `Running` | the program can take another step |
| `Finished` | the program ended |
| `Trapped` | the program stopped at a trap |
| `Waiting` | a read of standard input found no line in a world that keeps its input open |

`run` and `run_with` loop `step`, and the explorer calls `step`, `step_over` and `step_out`
to show a program running.

For the example of [The VM target](vm-target.md#an-example), the run enters `main` (`f2`),
stores 12 in the static block of `g0`, and calls `f1`. That call pushes a `Frame`, `call
f0` pushes another with one block for the local `က`, and `ret` of `f0` gives the block back
and leaves 144 on the stack for `print`.

## Start and host

`Machine::new` gives each global a static block, then gives each static text a frozen
static block with its UTF-8, opens the stack region and enters `main`. A write inside a
frozen block is the trap `FrozenWrite`. Globals start at zero octets (bytes), and the `Set`
statements that the IR keeps run in source order.

`set_host` puts the arguments and the environment in the globals that `host.args` and
`host.env` name, before the first operation: each text in its own heap block, and each
list in one more, so the drops at the end of `main` give them back. `exit_code` reads the
global `host.exit` once the run has finished and keeps its lowest 8 bits. A run with no
host leaves both lists blank.

A REPL session (`zakac repl`) keeps its machine from one chunk to the next.
`Machine::go_on` takes the module of the next chunk on the state of a run that finished:
the heap, the world, the rights, the threads and the closures remain. Each global of the
new module takes over the block of an old global, or gets a new block (`take_global`). The
static texts and lists of the new module get blocks of their own, and the old ones remain
for the values that point at them. The REPL clones the machine before each chunk and goes
back to the clone when the chunk traps.

## Calls and frames

`call f` takes one block of the stack region for each local of `f`
(`FrameMode::Checked`), or one block for all of them at C offsets (`Packed`), moves the
arguments from the operand stack into the first locals, and pushes a `Frame`. `ret` pops
the frame and gives the blocks back. For a list or struct result, it first duplicates
(copies) the value into the caller's slot, the value under the arguments. A depth past
`max_depth` is the trap `StackOverflow`.

## Function values and closures

`fn_addr f` pushes `f + 1` as an address value, so 0 contains no function. `call_value n`
takes it from under the `n` arguments, stops the program with `NullCall` for 0 (a
backstop for a cell that the init check could not prove set), and otherwise enters
function `f` the same as `call f` does.

A closure is the address of its record on the heap, which lies above the function numbers
(`Mem::closure_record`, `Mem::thunk_word`), with the lowest bit set while a call borrows
it. `closure f k` pops the address of a record of the data kind `k`, moves it to a new
counted heap block (`move_record`), writes `f + 1` into its first word, keeps `k` for the
thunk in `Machine::records`, and pushes the address. `call_value n` on a closure reads the
thunk from the first word and passes the address of the record as the last argument.

`blocks_in` returns each function value inside a value too, so `dup_value` retains the
record (`dup_fn`), `drop_value` releases it and the last holder frees it with what its
fields contain (`drop_fn`), and `lend_value` sets the lowest bit (`lend_fn`), so a drop of
a lent closure frees nothing. A closure given to a C function is lent to that call
(`ffi.rs`, see [C functions](#c-functions)), and a call from C to a closure that has no
running C call and no leave stops the program with `ClosureEscape`. A call statement that
drops a function value it receives stores it in a scratch slot and drops that.

A print of a closure names the function its thunk calls. `Mem` keeps the name of each
function, so `print_enum` and the prints of lists, structs and template slots write a
function value as `<name> ဟူသော လုပ်ဆောင်ချက်` (`Mem::fn_text`, which cuts the overload
number and the file off the inner name).

## Storage and values

`Mem` is the address space: blocks of three origins (`Heap`, `Stack`, `Static`), each with
a red zone after it, so an access just past a block never lands in the next one. `load`,
`store` and `take` check that the address is inside a live block, and trap if not: a null
address, a freed block, or past the end.

The operand stack contains `Value`s, so arithmetic wraps to the width of the data kind
(type), a zero divisor is the trap `ZeroDivisor`, and a shift past the width is
`ShiftTooFar`. The same `value` functions did the folding.

### Counted blocks

Each heap block of text, a plain list, a box or a closure record has a count (`Block.rc`,
with `Mem::retain` and `Mem::release`), the data kind of its cells (`Block.kind`) and a
`pinned` bit. Static and frozen blocks have no count. `own_value` and `remove_cell` give a
block that another holder shares its own duplicate first (`unshare`), and each
duplicate of a pinned block clones it. A drop queues the blocks it lets go (`freeing`), so
a chain of 20,000 objects frees in a loop and not through deep recursion.

`deep_seqs` (`machine/deep.rs`) makes a deep duplicate (deep copy) with a table from old
objects to new ones, so each object is copied once and a ring keeps its shape. `==` on
objects is deep and safe on rings (a stack of pairs), and a print shows an object that is
still being printed as `…`. The rows of a ဇယား (map) live in `machine/table.rs`.

## The ring collector

Counts alone never free objects that reach each other in a ring. A release that leaves a
count above 0 buffers the block when a block of its data kind can be part of a ring
(`can_ring`, worked out once for each data kind): a walk over the data kinds its cells
contain, through boxes and plain lists, meets the data kind again, or meets a function
value, whose closure can contain each thing. Text, lists of numbers and an object whose
fields cannot lead back to its data kind never wait.

The collector (`machine/collector.rs`, `collect_rings`) runs trial deletion, as PHP and
CPython do, when the buffer reaches the threshold (16,384 octets, or what a `rings
threshold` operation set) and when the program ends, so `--leaks` remains clean for safe
code. A pause keeps the buffer filling and the end of the program collecting. The deep
duplicate keeps the wider test, `holds_shared`: it walks into each object a value
contains.

## Threads

The VM runs the [threads](../language/threads.md) (`အပြိုင်လုပ်ငန်း`) of a program as
green threads on one OS thread, while the C and native targets run each thread as a light
task on a pool of OS threads, or on an OS thread of its own ([The C
target](c-backend.md#threads-and-channels)).

### Turns

`machine/threads.rs` keeps the threads. The running thread keeps its operand stack, calls
and count of locals in the fields of the VM, and each other thread keeps them in its row
of `Threads`, with its stack region (`Mem::take_region`, swapped in through
`Mem::swap_stack`). `spawn f` takes a region of `max_stack` octets past the heap, puts the
slot of a list or struct result at its bottom and a duplicate of each list, struct or text
argument that the thread keeps (`keep_arg`), and enters `f`, and then the spawner goes
on.

`take_turns` runs after each operation. A thread whose calls ended keeps its result
(`end_thread`), and a thread that ended, parked, or ran its 100 operations (`TURN`) passes
the turn to the next thread that can run, in order (`next_turn`). With none that can run,
the result is the trap `Deadlock`. With `RunOptions::shuffle` (`--shuffle-thread-order`),
a `fastrand::Rng` of the seed draws the length of each turn (1 to 200 operations) and its
thread among those that can run (`Threads::turn`, `next_to_go`). In native code, each
thread keeps the line it prints while threads run (`out::LINE`) and passes it on whole at
its end, so two threads never mix inside a line.

`wait` on a thread still running pushes the number back, steps the program counter back
one operation and parks, so the operation runs again on a later turn. The last `ret` of
`main` parks too while a thread runs (`park_at_end`). Under `TrapMode::Unwind`, a
trap in a thread ends that thread alone (`thread_trapped`), and a later wait for it is the
trap `ThreadTrapped`. `exit_code` is 1 when a trap ended a thread that no wait saw.

### Channels

`machine/pipes.rs` keeps each channel in `Threads::pipes`: a queue of heap blocks, its
room, the data kind of its values and whether it is closed. `pipe_send` moves the value
into a new heap block and zeroes its slot, and `pipe_receive` writes the optional (`tag` 1
and the value in `some`, or zeros once closed and drained) and frees the block. A full
channel, or an open one that has no values, parks the thread with `Waits::Send` or
`Waits::Receive`, which `can_go` tests. The end of the program drops the values left in
each queue (`drop_pipe_values`).

A wait on several channels (`Op::Select`) reads its operands without popping them, so it can
park and run again: the channels go to `Thread::selects`, the end of its time limit to
`Thread::until` at the first park, and the thread parks with `Waits::Select`. `can_go` lets
it run once a channel holds a value, each is closed, or the clock passed the end, and
`first_wake` counts the end as a wake. The pick among channels that hold a value is the
first, or one drawn from the seed of `--shuffle-thread-order` (`Threads::draw`).

### Locks

The lowering turns a lock block into `Stmt::Lock`, the block, and `Stmt::Unlock`.
`Lowerer::locks` keeps each open lock with the drop scope it belongs to, and `drops_from`
releases the locks of each scope after its drops, so a return, a break and a next round
release them too. The VM keeps `Threads::locks`, found through the block of the object,
which each thread that holds the lock shares. A lock that another thread holds parks with
`Waits::Lock`, and a trap of the holder marks it broken.

### Atomic numbers

An atomic number is a data kind of `task.rs` in `zakac_semcheck` (`%atomic(isize)`), kept
as its platform-sized integer, and `read_kind` gives the integer that a read and a slot
load use. `Stmt::AtomicAdd { addr, value, sub }` (from `ပေါင်းထည့်` and `နုတ်ယူ`,
`atomic_stmt`) is the operation `atomic_add`: one operation on the VM, and `zk_atomic_add`
(`AtomicIsize`) in native code.

A read of an atomic field of an object is `ExprNode::AtomicLoad`, and a set of one is
`Stmt::AtomicSet` (`holds_atomic`). These are plain `load` and `store` operations on the
VM, which runs one thread at a time, and `zk_atomic_load` and `zk_atomic_set` (`SeqCst`)
in native code, so a C compiler never keeps the number in a register across a loop that
waits for it, and folding never treats it as known. A local atomic number belongs to its
thread and is read and set as a plain slot.

A compare-and-set (a call of the core module's `အစားထိုး` with `ထဲရှိ`, `ကို` and
`ဖြင့်`, which `cas_call` checks) lowers to `ExprNode::AtomicCas`, which returns what the
number contained (`cas_node`): the operation `atomic_cas` on the VM, and `zk_atomic_cas`
(`compare_exchange`) in native code. The result is then good, with that value, when it
equals the expected one, and an error with that value otherwise.

### Parallel maps

The lowering (`par_map`) fills a hidden plain list in a loop of calls (`each_loop`) and
marks the loop with `Par::Map` (`Stmt::Loop::par`). Cells that widen to the parameter go
first to a hidden plain list of its data kind (`widened`), so the rounds receive each cell
as it is, and cells of text that differ from the parameter keep the loop in order. The VM
runs the loop, while the C and native targets call `zk_par_map` in its place with a step
of the function (`zk_map_step_<f>`), which lends the cell and writes the result. The
runtime runs the rounds the same as those of a parallel loop, then adds each result in the
order of the cells.

### Parallel loops

A counted loop whose counter is wider than the platform word (`counter_wider`) lowers as a
plain counted loop (`wide_counted`), after the check of its count of threads, since the
count of its rounds could pass the word.

`lower/rounds.rs` lifts the round of a counted loop, or of a loop over the cells of a
list, the characters of text or the rows of a ဇယား, into its own function,
`<fn>#round<n>`. Its parameters are:

- the number of the round (`usize`);
- for a counted loop, the first value, the direction and the step (a step of zero or less
  is `HaltCause::StepNotPositive` before the loop);
- for a list, its address; for text, the addresses of the text and of a list of where each
  character starts (a first pass, `char_starts`); for a ဇယား, its address (round `r` is
  the room of row `r`, and a removed row ends its round);
- then each variable from outside the round, through its address (a box with a count for
  one that a closure shares). The frame of the round marks each atomic number among them
  (`Frame::atomics`), so a read of it is an `AtomicLoad` and an add an `AtomicAdd`.

Its statements run inside a loop of one round, so a continue ends the round. A plain list
that a round writes gets its own block before the loop (`own_before`). The loop calls the
function once for each round and carries `Par::Rounds`. The VM runs it in order, and
native code calls `zk_par_for(count, step, data, guard, leave, abort)` with a step
(`zk_round_step_<f>`) that reads the arguments from a data block.

The runtime runs fewer rounds than cores in order, and otherwise starts one thread for
each core, or for each thread of the count of threads the loop was given
(`ParRounds::threads`, after `HaltCause::ThreadsNotPositive` for zero or less). The
threads take blocks of rounds from one counter until none is left (`Rounds`), so a fast
thread takes more. The runtime waits for each thread and joins its OS thread, then stops
with `thread trapped` when one trapped. The JIT runs both forms, through the guard the
runtime carries.

## Limits and traps

`RunOptions` has `max_depth`, `max_steps`, `max_heap`, `max_stack`, the frame mode, and
`leaks`. A trap is not a `ZK` diagnostic, since it belongs to the run of a program and not
to the compiler.

A `Trap` has the `kind`, the `func` and `pc` of the operation that stopped, the `block`
text of an address trap, and `frames: Vec<TrapFrame>`, the calls in progress with the
innermost first. A `TrapFrame { func, pc, span }` is a function name, the index of its
operation (for a caller, the operation of its call, which is the return address less
one) and the source region of that operation, `None` when the module has none.
`Machine::trap_frames` builds the list from the frame stack and leaves out the function
`main` that the compiler adds, unless the stop is inside it.

`Trap::report` writes the first line with the line of the first frame
(`file:line:column`, the column from 1, through the source map), or `(in <func>, op <N>)`
when the region is a placeholder. Then, when more than one frame exists, it writes the
line `calls in progress, the last first:` and one line for each call. `backtrace` folds a
run of frames of one operation into the line and `... the call above, N times more`, and
keeps `SHOWN_CALLS` (16) lines before `... N calls more`. The short form that the Run
buttons show, `the program stopped: <reason>`, has no place and no calls.

## Host functions

The functions the VM serves to a program live in `zaka_host`, in three parts:

| Part | Where | What it contains |
| --- | --- | --- |
| signature | the stdlib (`ပြင်ပမှလာသော` declarations) and `HOST_FNS` | the name, parameter data kinds, result and optional error kind. The check matches each external declaration to a row (`zakac_semcheck::host::find`) |
| values | `zaka_vm/src/machine/host.rs` | the VM reads an argument and writes a result from its data kind alone, so a new function over the data kinds there needs no VM code |
| effects | the run function of a row, against `Effects` | plain Rust: `Effects` reaches the `World` (`read_line`, `read_file`, `write_file`) and the standard streams of the VM (`write`) |

The operation `host h`, the one operation of an external function, reads each argument
through the address of its parameter as a `zaka_host::HostValue` (from its data kind:
text, a number, a stream struct or an enumeration member), runs `h`, and
writes the result in the result slot from the data kind of the result (for a result that
can fail: tag 1 and the good side, or tag 0 and the error side).

The VM keeps its world (`Box<dyn World>`, a `MemWorld` unless `set_world` or `run_in`
passes another) with the rest of its state, so a clone for the explorer carries the world
along. Writes to the standard streams go to the output of the VM, so those writes keep
their order with what the program prints. A `MemWorld` with `open` set (the explorer's)
answers a read with no line with `WouldBlock`: `Effects` marks the call blocked, and the
operation puts its operands back and steps back to itself uncounted, and the state
becomes `Waiting` until `give_input` or `close_input`.

`HOST_FNS` is one table, made with the macro `host_fns!`, which also makes the enum
`HostFn`:

```text
ReadFile = "ဖိုင်ကိုဖတ်"(TEXT) -> Some(TEXT), Some(IO_ERROR), Some(Right::Read), HostAction::Effect(read_file);
```

An effect row runs a Rust function `fn(&Args, &mut Effects) -> HostValue`. `Args::text(n)`
and `Args::stream(n)` read the arguments, and `text_or_fault` and `nothing_or_fault` turn an
`io::Result` into a result, with a failure going through one mapping of `io::ErrorKind`
to the members of the error enumeration. Math rows share their float calculations
with the native runtime.

To add a host function:

1. Add a row to `host_fns!` and the function that runs it.
2. When it needs an effect outside the VM that `World` lacks, add a `World` method and
   write it for `MemWorld` (the browser and the tests) and `TermWorld` (the terminal). A
   new data kind of argument or result needs a case in `machine/host.rs`.
3. Declare it in the stdlib with its prepositions and connect it to the native and C
   targets.

A `MemWorld` has the input lines given before the run and a map of files, and a
`TermWorld` reads the standard input of the process and the files on disk. `zakac run`
runs in a `TermWorld`, and each other run (the tests, the books, the explorer) in a
`MemWorld`.

## C functions

A C function (`"libm" မှလာသော`) remains a function of the program whose one statement calls
C. Unlike a host function, it passes the value of each parameter and not its address. A C
function that accepts other values (`အခြားတန်ဖိုးများနှင့်အတူ`, C's `...`) has no such
function: each call names the C function itself, with all its arguments. This section
follows a C call through each stage.

### The IR

In `lower.rs`, `foreign_statement` and `foreign_index` make the statement
`ExprNode::Foreign { func, args }` (`Stmt::Foreign` with no result). `Module::foreign` has
one `ForeignFn { lib, name, params, ret, variadic }` for each C name, so two declarations
of a name share a row.

- A C function that accepts other values has `ForeignFn::variadic`, and `collect_sigs`
  gives its signature the `ForeignFn`, so `Target::Foreign` makes the call site
  `ExprNode::Foreign` (or `Stmt::Foreign`) with each argument, and no wrapper function
  runs. `rest_arg` lowers an other value the same as C promotes it (`promoted`: an integer
  under 32 bits and a true/false value to `i32`, `f32` to `f64`). A text that is not a
  plain read is built into a hidden slot (`spill`) that the end of the scope drops.
- A list of numbers goes as the address of its first cell (`c_list_arg`,
  `c_param_kind`), so `ForeignFn.params` has a pointer for it. For a parameter marked
  `ဒေတာ ပြောင်းလဲရန်`, a plain list owns its cells first, so C writes them in place and no
  duplicate sees the writes.

### Handles

A [data kind from outside the
program](../language/functions-c.md#data-kinds-from-outside-the-program) crosses as its
address: `c_param_kind` turns it into `DataKind::handle_pointer()`. The lowering
(`lower.rs`: `foreign_statement`, `handle_ptr`, `handle_field`, `handle_guard`,
`handle_close`, `handle_of`, `handle_or_none`, `c_field_read`) adds no new statement:

- The wrapper function guards each handle parameter with `Stmt::Halt { cause:
  ClosedHandle }` (the box contains 0) and passes the address read from the box
  (`handle_ptr`).
- It wraps a plain result with `make_box` after a test of the address (`handle_or_stop`):
  NULL halts with `HaltCause::NullFromC`, which the VM and the C target report as the trap
  `null from C`, since the declaration promised a handle.
- It builds the optional of the result (`handle_or_none`: no data for NULL, the box as
  data otherwise), and `is_c_handle_result` tests the data kind.
- After the call, it empties the box of a parameter marked `အပိုင်အနေဖြင့်` with a
  `Store` of 0 (`handle_close`), so each duplicate of the handle counts as closed.
- A field read is the node `ExprNode::CField { base, offset }`: the address of the C
  struct, which the guard checked, and the offset of the field. Text read from a `char *`
  waits in a hidden slot that the scope drops.
- A write of a field of a handle in unsafe code is `Stmt::CStore { base, offset, value }`
  after the guard of the handle (`c_field_store` in `lower.rs`). The VM pops the value and
  the address and writes the octets in C's storage (`c_store` in `machine/ffi.rs`), and the
  C target writes them with `memcpy` (`c_store` in its `stmts.rs`).

### The flow checker

In `walk.rs` and `flow.rs`, each argument is a read and the result is a fresh value, the
same as for `Host`.

### The operations

`emit.rs` writes the operation `foreign c<n>` (`Op::Foreign`), which pops each parameter
and each other value, the last on top (a struct or a text is the address of its value). A
struct result has the address of its slot under them and leaves it, another result is
pushed, and no result pushes nothing. One `bc::ForeignCall { func, extra }` serves each
shape of call (the C function and the data kinds of its other values), so the same shape
shares an index, and `bc::Module::foreign_calls` carries them. A statement call
(`foreign_stmt`) of a C function with a result pops it.

A field read of a struct that C keeps is `c_load k at o` (`c_field_at` in `emit.rs`,
`c_load` in `machine/ffi.rs`). It pops the address of the C struct and pushes a number or a
true/false value read at the offset. For a counted list or text, it pops the address of a
block under it, writes the cells (or the string copied up to its zero octet, the optional
with no data for NULL) there, and pushes that address. A handle that is closed stops the
run first (`halt closed_handle`, the trap `closed handle`).

### The VM

The call is `foreign c<n>`, made with `Module::foreign_calls`, a table of `ForeignCall {
func, extra }` over `Module::foreign`. The code lives in `zaka_vm/src/machine/ffi.rs`,
behind the feature `ffi` of `zaka_vm`:

- The first call links. Each lib opens (`libX` is `libX.dylib`, `libX.so` or `X.dll`, on
  Linux `libX.so.6` too, and libc and libm on Windows are `ucrtbase.dll`), and each C
  name is looked up in its lib, or else in the process, which contains libc. A lib given
  as a path opens as it is (`open`).
- libffi makes the call (`Cif::new_variadic` for a C function that accepts other values,
  with the count of its named parameters).
- A text parameter or other value goes as the address of a duplicate of its octets with a
  zero octet at the end (`arg_cell`, `text_octets`), kept alive until the call returns.
- After each call, `flush_c` calls libc's `fflush(NULL)`, found in the process at link
  time, so what C printed comes in order with the program's output.
- The run refuses when `--target-bits` is not the host's, and the VM on a big-endian host
  is not supported for C.

The result data kind `စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`, an optional that
`is_c_text_result` (`zakac_semcheck/src/foreign.rs`) tests, is read as a C string
(`text_from_c` in `machine/ffi.rs`): NULL writes the optional with no data, and otherwise
the zero-ended octets are copied into new text of the program (`bad_utf8` for octets that
are not UTF-8). The text is read before the duplicates of the text arguments are freed.

An address that lies in a live block becomes the native address of that octet
(`Mem::native_at`), for a parameter and for an address field of a struct, so C reads and
writes the block in place. An address that C returns that lies in a block, or just past
its end, maps back (`Mem::vm_addr`), and another passes as it is and cannot be read
through in the VM. The octets of each block have one spare octet of room past their end
(`zeros`), so the address just past a block lies in that block's own allocation and is
never the start of a struct that C allocates: such a struct, behind a handle, remains a
native address.

A build with no feature `ffi` (the explorer) has `without_ffi::call_c`, which gives the
trap `ForeignCall`. A lib that does not open, a missing C name and a platform size that is
not the host's give `ForeignLink`. The zakac and book-test builds turn `ffi` on.

### Callbacks

A function value of a function kind that `is_callback` accepts goes to C as a function
pointer (`Callbacks` in `machine/ffi.rs`): one libffi closure for each function that a
call gives C, made on the first such call and kept for the run.

`callback_arg` turns each argument into a value of the VM. A scalar is read as it is, a
plain struct is copied from the octets C passed, text comes from the `char *` (`text_arg`,
where NULL is `NullFromC`), an optional of text through `text_from_c`, and a handle
becomes a new box around the address (`handle_arg`, `optional_handle_arg`). Each of these
lives in a heap block, and the function gets a lent duplicate of it, the same as a call of
the program lends, and `drop_temps` drops both once the function returns. The callback
runs the function in the VM until it returns. A trap in it waits until C returns, then
stops the program, and each later callback of that call returns 0.

A closure goes through `closure_callback`: one libffi closure for each record, found again
from the octets of the record, and its code remains in place for the run, so a late call
finds live code. `callback` counts the C calls that got the closure
(`CallbackData::calls`, ended in `end_lends` once the call returns), and the operations
`c_leave` and `c_revoke` (`Machine::c_leave`, `c_revoke`) add or drop a count of the
record and set `CallbackData::kept`. C can call a closure while `CallbackData::live` holds
(a running call or a leave). A call from C at another time stops the program with
`ClosureEscape`, and a call on another OS thread ends the run.

### The C target

`zakac_codegen_target_c/src/ffi.rs` writes `zaka_ffi.c` and one wrapper for each call
shape (`FfiCall { func, extra }`, `Shared::ffi_calls`, `ffi_call`); see [The C
target](c-backend.md#c-functions).

## Tests

`zaka_vm/tests` has `machine.rs` (hand-built modules, malformed operations included, which
must trap and not panic), `traps.rs`, `heap.rs`, `rings.rs`, `threads.rs`, `callbacks.rs`,
`host.rs`, `stepping.rs`, and `samples.rs`, which runs each program in `samples/` outside
`diagnostics` and `mistakes` and compares its output with
`tests/expected/<group>/<name>.out`.
