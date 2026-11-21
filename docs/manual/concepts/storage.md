---
title: Storage, explained
description: The reason safe Zaka counts references, duplicates values on write and collects rings, what that costs, and the reason Zaka still has unsafe code.
---

# Storage, explained

A program asks the heap for a block (allocation) of storage (memory), uses it, and must
free it once. Three questions follow from that, and their answers decide most of
the storage mistakes that a language allows:

1. **Who** frees the block?
2. **When** is it freed?
3. **Can a name still reach it** after that?

Safe Zaka answers all three for us: *the VM (virtual machine) counts the references to
each block and frees the block when the last reference goes.* Values are duplicated,
shared objects are shared, and no safe program has an address (pointer). This page
explains the reasons, works through an example, and compares the design with C and with
garbage collection. The rules are in [Values and objects](../language/objects.md), and
pointers are in [Unsafe Zaka](../language/unsafe.md).

## The three classic mistakes

When a program frees its blocks itself, other pointers duplicated from the first one can
still reach a block after it is freed. Nothing in C ties these duplicates to the block, so
three mistakes follow:

| Mistake | What happens | Where it shows |
| --- | --- | --- |
| use after free | a pointer reads or writes a block that was freed | much later, as wrong data, or never |
| double free | a block is freed twice | in the allocator, as a crash or worse |
| leak | the last pointer to a block is lost before the block is freed | never at once, but the program grows |

None of the three can happen in safe Zaka, because no name has an address and a block is
freed when nothing reaches it. In unsafe code, the VM finds the first two when a run
reaches them ([When a program stops](../vm/traps.md)), and the compiler warns about each of
the three when it can prove it ([Compiler checks](../compiler/checks.md)).

## Three answers

| | C | tracing garbage collection | Zaka |
| --- | --- | --- | --- |
| who frees a block | the programmer, with a call | the runtime, after nothing can reach the block | the runtime, when the count reaches zero |
| when | at the call to `free` | at a time the runtime picks | at once, at a known point; a ring when the collector next runs |
| use after free, double free | possible, silent | impossible | impossible in safe code; stopped on the VM in unsafe code |
| leak | silent | rare: a block that is still reachable is kept | none in safe code; `--leaks` lists the blocks of unsafe code |
| cost at run time | none | the work of the collector, and blocks that are kept longer | a count change for each shared reference, and the ring collector |
| cost to the writer | remembering each rule | little | little: values and objects, and the word `စိတ်မချရသော` on unsafe code |

Tracing collection frees a block at a time that the runtime picks. C frees it at a time
that the programmer picks, and trusts the programmer to get it right. Counting frees it
the moment the last reference goes, so a block that has a file open closes the file at a
point that we can find in the source, and the runtime never needs to pause to search the
heap.

## Counting

Each heap block of a value, an object or a closure has a count of the references to it. A
new reference adds one, a reference that ends subtracts one, and at zero the block is
freed, along with the references stored inside it.

```text
ကိန်းရှင် က သည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။      block A, count 1   (က)
ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။               block A, count 2   (က, ခ)
ခ ၏ အကွက် ၀ ကို ၉ ဟု ထားမည်။               block A, count 1   (က)
                                           block B, count 1   (ခ), a duplicated block
end of the block                           A and B, count 0: both freed
```

The compiler leaves out most count changes. A value that is used for the last time is
moved instead of shared, so its count does not change, and when a call just reads an
argument, the argument reaches the function with no count change, because the caller's
reference outlives the call.

## Duplicated on write

A value such as a list is duplicated (copied) when we assign it, but the duplicate costs
nothing until one side writes. Assignment shares the block and adds one to its count, and
a write first checks the count: when the block is shared, the write duplicates the cells
into a new block that belongs to the writer alone. That is the third line of the example
above, where `ခ` gets block B the moment it writes, and `က` keeps block A unchanged.

So a value behaves like a number, a change through one name never reaches another name,
and a program that passes a long list to a function that just reads it duplicates nothing.

## Rings

Counting alone has one gap: objects that reach each other in a ring keep each other's
counts above zero after the program can no longer reach them, and a tree with links to
parents is such a ring. Zaka closes the gap with a **ring collector**, which uses trial
deletion, the method of PHP and CPython:

```text
 a ──▶ b            the program drops its last name for a:
 ▲     │            the count of a goes from 2 to 1, so a waits as a candidate
 └─────┘
                    the collector: remove the counts that a and b give each other;
                    both reach zero, so nothing outside reaches them: free both
```

- A block waits as a candidate when a release leaves its count above zero and the block
  can be part of a ring, because its fields lead back to a block of its data kind (type),
  or contain a closure. Text and lists of numbers never wait, so most values cost nothing
  extra.
- The collector runs when the waiting blocks add up to 16,384 octets (bytes), and once more
  at the end of the program. A program can run the collector, pause it, resume it and set
  that threshold ([Steering the collector](../language/objects.md#when-a-block-is-freed)).
- A block outside a ring is still freed at once, while a block inside a ring is freed at
  the next run of the collector. So a file that the program reaches through a ring alone
  closes later than one it reaches through a plain name.

## The reason for unsafe code

Two kinds of work need addresses: talking to C, whose functions accept and return
pointers, and code that manages its own blocks, such as an allocator that hands out pieces
of one large block. Safe Zaka has no addresses, so this work goes in **unsafe code**: a
function or a block marked with `စိတ်မချရသော`. Inside it, a program can store a pointer,
read and write through it, and ask for and free blocks with the heap statements.

Unsafe code gives up the promise of safe code, but it does not give up the checks:

- On the VM, each access through a pointer is still checked, and the VM stops the program
  at a use after free, a double free or an access out of bounds, so unsafe code never has
  undefined behaviour on the VM. Native code does not make these checks (see [Checks in
  the VM and in native code](checked.md)).
- The compiler follows the pointers of unsafe code and warns about each mistake it can
  prove: a leak, a use after free, a double free, or the address of a local that outlives
  its call. `--lint-error mem` turns the warnings into errors, and `--lint-allow mem` turns
  them off.
- A search for `စိတ်မချရ` finds each unchecked place, calls included, and `--no-unsafe`
  refuses unsafe code for a lesson or a project that keeps to safe code.

## What it costs

- **Count changes.** Sharing a reference writes to its count. The compiler removes most of
  these writes, but not all of them.
- **A write can cost a duplicate.** The first write to a shared value duplicates its
  cells, so a loop that writes to a list it shares with another name spends that cost once,
  at the first write.
- **Rings wait.** An object in a ring outlives its last name until the collector runs.
- **A range duplicates its cells.** A range of a list or of text is a new list or a new
  text value, not a view into the old one.

## What it promises

A program with no unsafe code never meets a use after free, a double free or a bad free,
never leaks a block, and never keeps an address into the frame of a call that has
returned. This is true on the VM and in native code alike.

Such a program has no null either: an object and a function value have a value at each point,
and no variable is read before the program sets it. Safe code does not promise that a
cell number is inside its list when the number is not known before the run, or that the
heap never runs out, and the program stops at each of these. A cell of a counted list that
the compiler cannot prove set is a warning (ZK0723): such a cell contains zero octets, and
the program stops at a call or a field read through it. [Checks in the VM and in native
code](checked.md) sorts all of these.

## Where to go next

- [Values and objects](../language/objects.md) states the rules of safe code.
- [Unsafe Zaka](../language/unsafe.md) states the rules of unsafe code, with [Pointers and
  addresses](../language/pointers.md) and [The heap and owned
  pointers](../language/heap.md).
- [The flow checker](../internals/borrow-checker.md) describes how the compiler follows
  pointers in unsafe code.
