---
title: Storage, explained
description: The reason safe Zaka counts references, copies values on write and collects rings, what that costs, and the reason an unsafe tier remains.
---

# Storage, explained

A program asks the heap for a block, uses it, and the block must go back once. Three
questions follow, and the answers decide most of the storage mistakes a language allows:

1. **Who** gives the block back?
2. **When** is it given back?
3. **Can a name still reach it** after that?

Safe Zaka answers all three for the programmer: *the VM counts the references to each
block and frees it when the last one goes.* Values are copied, shared objects are shared,
and no safe program holds an address. This page tells the reasons, walks through a
scenario, and compares the design with C and with garbage collection. The rules
themselves are in [Values and objects](../language/objects.md), and pointers live in
[Unsafe Zaka](../language/unsafe.md).

## The three old mistakes

When a program frees blocks itself, a block given back can still be reached through
other pointers copied from the first. Nothing in C ties the copies to the block, and three
mistakes follow:

| Mistake | What happens | Where it shows |
| --- | --- | --- |
| use after free | a pointer reads or writes a block that was given back | much later, as wrong data, or never |
| double free | a block is given back twice | in the allocator, as a crash or worse |
| leak | the last pointer to a block is lost before the block is given back | never at once; the program grows |

In safe Zaka none of the three can happen: no name holds an address, and the block goes
when nothing reaches it. In unsafe code the checks of the VM find the first two when a run
reaches them ([When a program stops](../vm/traps.md)), and the compiler warns about each
of the three it can prove ([Compiler checks](../compiler/checks.md)).

## Three answers

| | C | tracing garbage collection | Zaka |
| --- | --- | --- | --- |
| who gives a block back | the programmer, with a call | the runtime, after nothing can reach the block | the runtime, when the count reaches zero |
| when | at the call to `free` | at a time the runtime picks | at once, at a known point; a ring when the collector next runs |
| use after free, double free | possible, silent | impossible | impossible in safe code; stopped at run time in unsafe code |
| leak | silent | rare: a block still reachable is kept | none in safe code; `--leaks` lists the blocks of unsafe code |
| cost at run time | none | the collector's work, and blocks kept longer | a count change per shared reference, and the ring collector |
| cost to the writer | remembers each rule | little | little: values and objects, and the word `စိတ်မချရသော` on unsafe code |

Tracing collection frees a block at a time the runtime picks. C frees it at a time the
programmer picks, and trusts the programmer. Counting frees it the moment the last
reference goes, so a block that holds a file closes the file at a point the reader can
find in the text, and the runtime needs no pause to search the heap.

## Counting

Each heap block of a value, an object or a closure carries a count of the references to
it. A new reference adds one; a reference that goes removes one; at zero the block is
freed, and the references it held go with it.

```text
ကိန်းရှင် က သည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။      block A, count 1   (က)
ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။               block A, count 2   (က, ခ)
ခ ၏ အကွက် ၀ ကို ၉ ဟု ထားမည်။               block A, count 1   (က)
                                           block B, count 1   (ခ), a copied block
end of the block                           A and B, count 0: both freed
```

The compiler leaves out most count changes. A value used for the last time is moved
rather than shared, so its count does not change, and an argument that a call reads
reaches the function with no count change: the caller's reference outlives the call.

## Copied on write

A value such as a list is copied when it is assigned, but the copied cells cost nothing
until one side writes. Assignment shares the block and adds one to its count; a write
first checks the count, and when the block is shared, copies the cells to a block of its
own. That is the third line of the scenario above: `ခ` gets block B the moment it writes,
and `က` keeps block A unchanged.

So a value behaves as a number does, a change through one name never reaches another, and
a program that passes a long list to a function that reads it copies nothing.

## Rings

Counting alone has one gap: objects that reach each other in a ring keep each other's
counts above zero after the program can no longer reach them. A tree with parent links is
such a ring. Zaka closes the gap with a **ring collector**, which uses trial deletion, the
method of PHP and CPython:

```text
 a ──▶ b            the program drops its last name for a:
 ▲     │            a's count goes from 2 to 1, so a waits as a candidate
 └─────┘
                    the collector: remove the counts that a and b give each other;
                    both reach zero, so nothing outside holds them: free both
```

- A block waits as a candidate when a release leaves its count above zero and its kind can
  hold an object or a closure. Text and lists of numbers never wait, so most values cost
  nothing extra.
- The collector runs when 1,024 blocks wait, and once more at the end of the program.
- A block outside a ring is still freed at once. A block inside a ring is freed at the next
  run of the collector, so a file held through a ring alone closes later than one held
  through a plain name.

## The reason for an unsafe tier

Two jobs need addresses: talking to C, whose functions take and return pointers, and
code that manages its own blocks, such as an allocator that hands out pieces of one large
block. Safe Zaka has no address, so these live in **unsafe code**: a function or a block
that carries `စိတ်မချရသော`. Inside it a program can hold a pointer, read and write through it,
and allocate and free blocks with the heap statements.

Unsafe code gives up the promise of the safe tier, not the checks:

- The VM still checks each access through a pointer, and stops the program at a use after
  free, a double free or an access out of bounds. Unsafe never means undefined in the VM.
- The compiler follows the pointers of unsafe code and warns about each mistake it can
  prove: a leak, a use after free, a double free, a local's address that escapes.
  `--mem-check` turns the warnings into errors or off.
- A search for `စိတ်မချရ` finds each unchecked place, calls included, and `--no-unsafe`
  refuses unsafe code for a lesson or a project that keeps to the safe tier.

## What it costs

- **Count changes.** Sharing a reference writes its count, and the compiler removes most
  of these writes, but not all.
- **A write can cost a duplicate.** The first write to a shared value copies its cells. A
  loop that writes a list it shares with another name spends that once, at the first
  write.
- **Rings wait.** An object in a ring outlives its last name until the collector runs.
- **A range copies its cells.** A range of a list or of text gives a list or text of its
  own.

## What it promises

A program with no unsafe code never reaches a use after free, a double free or a bad free,
never leaks a block, and never holds an address into a frame that has returned. That holds
on the VM and in native code alike.

Nor does it hold NULL: an object and a function value hold a value at each point, and no
variable is read before the program sets it. It does not promise freedom from a cell number
outside its list when the number is not known before the run, or from running out of heap:
the VM stops the program at each of these. A cell of a counted list that the compiler
cannot prove set is a warning (ZK0723); such a cell holds zero octets, and the VM stops a
call or a field read through it. [Checks in the VM and in
native code](checked.md) sorts them all.

## Where to go next

- [Values and objects](../language/objects.md) states the rules of the safe tier.
- [Unsafe Zaka](../language/unsafe.md) states the rules of unsafe code.
- [The flow checker](../internals/borrow-checker.md) tells how the compiler follows
  pointers in unsafe code.
