---
title: Checked and packed blocks
description: How the machine sets out its storage in blocks, how it checks each access, and the two forms of placing variables.
---

# Checked and packed blocks

The storage of the machine is a flat run of addresses, and each part of it belongs to a
*block*: a run of octets with a start, a length and a state. A global, a local, and
each piece of heap is a block. The machine keeps a table of the blocks, and each read or
write through an address looks the address up in that table before it touches an octet.
This page tells how the addresses are laid out, how a block lives and dies, and how the
two *frame modes* differ.

## The address space

The space holds `2^bits` addresses for a platform of `bits` bits (at most `2^64`). It is
laid out upward from address 0:

```text
 0x0000 ┌──────────────────────────────┐
        │ reserved: never handed out   │  4096 addresses (16 on an 8-bit platform)
 0x1000 ├──────────────────────────────┤
        │ static blocks: the globals   │  one block each, for the whole run
        ├──────────────────────────────┤
        │ stack region                 │  the locals of the calls in progress,
        │   (up to --max-stack octets) │  last in, first out
        ├──────────────────────────────┤
        │ heap                         │  blocks from the allocate statement,
        │                              │  upward; capped with --max-heap
        │            ...               │
 end    └──────────────────────────────┘
```

- **Reserved.** An address below 4096 is never part of a block. `ဗလာ`, and each small
  offset from it, falls here, so a read through a null pointer cannot find a block
  through luck. On a platform of 8 bits the reserved part is 1/16 of the space, 16 addresses.
- **Static.** Before `main` starts, each global gets a block of zeros. Static blocks last until
  the end.
- **Stack region.** It opens right after the statics. Its size is `--max-stack` or half
  of the room left, whichever is smaller; the heap starts at its end.
- **Heap.** New blocks go above all the blocks made so far.

With two integer globals and the default limits, the stack region runs from `0x1040` to
`0x101040`, and the first heap block starts at `0x101040`. A pointer prints as its
address, so the arrangement can be seen:

```zaka
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် ဒုတိယ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ကို ပြသပါ။
ပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
မင်မိုရီ ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ဒုတိယ တွင် သိမ်းထားမည်။
ဒုတိယ ကို ပြသပါ။
ဒုတိယ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
0x101040
0x101060
```

The second block is not at the address of the first, although the first was given back.
The reason is in [the life of a heap block](#the-life-of-a-heap-block).

## What a block holds

| Field | Meaning |
| --- | --- |
| start | the address of the first octet; a multiple of 16 |
| length | the octet count; kept after the block is freed |
| contents | the octets; dropped when the block is freed |
| origin | `heap`, `stack` (a local or parameter of a call), or `static` (a global) |
| live | whether the block is in use; a freed block stands in the table as a record |
| owner | the call depth that made a stack block; 0 for heap and static blocks |

The table is kept in order of start. To check an access of `n` octets at `a`, the machine takes
the last block that starts at or below `a` and asks whether `a + n` remains inside it.

**Red zones.** A gap of at least 16 addresses, which belong to no block, follows each
block. A read or write just past the end of a block lands in the gap, never in the
next block, so a mistake of one box too far is caught instead of reading a neighbour:

```text
  block A                gap                      block B
 ┌──────────────────┐ ░░░░░░░░░░░░░░░░░░░░░░ ┌──────────────────┐
 │ 0x1000 .. 0x1007 │ ░ 0x1008 .. 0x101f   ░ │ 0x1020 .. 0x1027 │
 └──────────────────┘ ░░░░░░░░░░░░░░░░░░░░░░ └──────────────────┘
          a read at 0x1008 lands in the gap: out of bounds
```

**The order of the checks.** One function checks each load and store, in this order, and
the first failure is the trap:

1. the address is below the reserved part: `null address`
2. the address is not a multiple of the alignment of the kind: `misaligned`
3. no block holds the address, or the access runs past the end of the block it starts
   in: `out of bounds`
4. the block is freed: `use after free`
5. the octets read hold no value of the kind: `bad value`

[When a program stops](traps.md) gives each message with an example.

**Forming an address is free.** Moving a pointer does not read an octet, so the machine
checks just that a step remains inside the block the pointer started in, one past its end
included. A step that leaves the block traps at once, since no offset leads from one
block into another. A subtraction of two pointers needs both in one live block, again
counting the end.

## The life of a heap block

```text
            allocate                    free
 (no block) ───────▶  live  ─────────────────────▶  freed
                       │                             │
                       │ resize: a new block is      │ the record remains in the table,
                       │ made, and this one is       │ and its start joins the queue
                       │ freed                       │ of freed blocks: the quarantine
                       ▼                             ▼
              reads and writes work         reads, writes and frees trap
```

1. **Allocate.** A new block gets the lowest fresh address above each block made so far,
   filled with zeros, and counts against the heap cap. If the cap or the space has no
   room, the statement leaves `ဗလာ`.
2. **Free.** The block becomes *freed*: its contents are dropped, but its start and
   length remain in the table, and its start joins a queue of freed blocks, oldest first.
   This is the *quarantine*. An access that lands in a freed block is a use after free,
   and freeing it again is a double free.
3. **Reuse.** The machine takes fresh addresses first. When the space is full, and not before, it
   takes the oldest freed block that is long enough. On a 64-bit platform that does not
   happen in practice, so a stale pointer keeps pointing at a record of a freed block,
   and the mistake is caught however late the pointer is used. On a small platform the
   quarantine is shorter, and a use after free can go unseen once the block is reused.
4. **Resize.** A resize makes a new block each time, copies as much of the old contents as
   fits, zeroes the rest, and frees the old block. The new block can sit at another
   address, so a borrow made before the resize points into a freed block afterwards.

The cap counts live heap octets. A free lowers the count, and the machine remembers the
peak, which the Explorer shows.

## Blocks of the stack region

A call takes one block for each local, parameters first, at the top of the stack region.
Each is aligned to 16 and has its red zone after it. When the call returns, its blocks are
marked freed and the top of the stack moves back to where it was before the call:

```text
 stack region, growing upward

 main calls f, f calls g              g returns                  main calls h

 ┌──────────────┐ top              ┌──────────────┐          ┌──────────────┐
 │ g: s, t      │                  │ (g: freed)   │          │ h: p, q      │ takes
 ├──────────────┤                  ├──────────────┤ top      ├──────────────┤ the room
 │ f: n, r      │                  │ f: n, r      │          │ f: n, r      │ of g
 └──────────────┘                  └──────────────┘          └──────────────┘
```

A freed stack block has no quarantine. It remains in the table until a later call takes its
room, and the table then forgets it. Until then, a read through the address of a local of
a function that has returned traps as a use after free, and the message names the
variable and its function. After another call has taken the room, the read finds the new
local, as it would on a real machine. This is the reason the compiler refuses a function that
returns a borrow of its own variable: see [pointer checks](../compiler/checks.md).

A call whose locals do not fit in what is left of the region traps with `stack full`.

## Checked and packed variables

The machine can set out the variables of a run in two forms, picked with `--frames`:

| | checked, the default | packed |
| --- | --- | --- |
| globals | one static block each | one static block for all, at C struct offsets |
| locals of a call | one stack block each | one stack block for all, at C struct offsets |
| a step past a variable | lands in its red zone: the program stops | reads or changes the next variable |
| a step past the whole frame, or past a heap block | stops | stops |
| a trap names | `x of f`, `global x` | `frame of f`, `globals` |

```text
 checked: each variable is a block of its own

   0x1000        0x1020        0x1040
  ┌──────┐ gap  ┌──────┐ gap  ┌──────┐
  │  a   │░░░░░░│  b   │░░░░░░│  c   │
  └──────┘      └──────┘      └──────┘

 packed: one block, the variables next to each other, as a C struct

   0x1000
  ┌──────┬──────┬──────┐
  │  a   │  b   │  c   │
  └──────┴──────┴──────┘
```

Packed puts each variable at the next multiple of its alignment, in the order the
variables are declared, with parameters first. It is the shape a C compiler gives to the
locals of a function, and the shape a native Zaka program would give them (see [Checks in
the VM and in native code](../concepts/checked.md)). A program with no mistakes prints
the same in both modes.

The addresses of two globals show the difference:

```console
$ zakac run program.zaka
0x1000
0x1020
$ zakac run --frames packed program.zaka
0x1000
0x1008
```

Here `program.zaka` prints `က ၏ လိပ်စာ` and `ခ ၏ လိပ်စာ`, two integers declared one
after the other. The checked run leaves 24 addresses between them, and the packed run
leaves none.

The difference shows when a mistake is made. A program borrows `က`, writes `၉` one box
past it, and prints `ခ`:

```console
$ zakac run --mem-check warn program.zaka
warning[ZK0710]: offset 1 is past the end of a block of 1 elements
zaka_vm: the program stopped: access of 8 octets at 0x1008 lands outside each live block (in main, op 14); block at 0x1000, 8 octets, global က
$ zakac run --mem-check warn --frames packed program.zaka
warning[ZK0710]: offset 1 is past the end of a block of 1 elements
၉
```

Checked stops at the write. Packed writes `၉` over `ခ`, so the program prints it, and the program goes wrong far
from the line at fault. Keep the checked mode to find such mistakes; use packed to see
what the checks protect from. [Pointer mistakes](../../learn/pointers/mistakes.md#reading-past-a-variable)
walks through the program in Learn.

## Looking at the blocks

The [Explorer](/explorer/) draws the table of blocks while a
program steps, with each block's start, length, origin and state. `zakac run --leaks`
lists the live heap blocks at the end of a run: a block that is still live then was never
given back.
