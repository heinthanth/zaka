---
title: Addresses and blocks
description: How the VM sets out its storage in blocks, how it checks each access, and the two forms of placing variables.
---

# Addresses and blocks

VM (virtual machine) storage (memory) is one range of addresses (pointers). A *block*
(allocation) is a run of octets (bytes) with a start, a length and a state. Globals,
locals and heap allocations have blocks. Before a read or write through an address, the
VM looks up the block in its table. This page covers how addresses are arranged, the
life of a block and the two *frame modes*. For examples of failed checks, see [Pointer
mistakes](../../learn/pointers/mistakes.md) in Learn.

## The address space

The space has `2^bits` addresses for a platform of `bits` bits (at most `2^64`), laid out
upward from address 0:

```text
 0x0000 ┌──────────────────────────────┐
        │ reserved: never handed out   │  4096 addresses (16 on an 8-bit platform)
 0x1000 ├──────────────────────────────┤
        │ static blocks: the globals,  │  one block each, for the whole run
        │   then the static texts and  │
        │   the static lists           │
        ├──────────────────────────────┤
        │ stack region                 │  the locals of the calls in progress,
        │   (up to --max-stack octets) │  last in, first out
        ├──────────────────────────────┤
        │ heap                         │  blocks from the allocate statement,
        │                              │  upward; capped with --max-heap
        │            ...               │
 end    └──────────────────────────────┘
```

| Part | What it contains |
| --- | --- |
| reserved | no block at all: an address below 4096 (below 1/16 of the space on a small platform, 16 addresses on an 8-bit one) |
| static | the globals, the static texts and the static lists, in that order, for the whole run |
| stack region | the locals of the calls in progress; its size is `--max-stack` or half of the room left, whichever is smaller |
| heap | the blocks of the allocate statement, each one above all the blocks made so far |

The null address, 0, and each small offset from it fall in the reserved part, so a read
through the null address cannot find a block through luck.

Before `main` starts, each global gets a block of zeros, then each static text of the
module gets a block that contains its UTF-8, and then each static list gets a block with
its cells ([Static data](../language/counted-lists.md#static-data)). Static blocks last until the end of
the run. A static text or list block is *frozen*: a read works, and a write stops the
program with `write at ADDR inside static data, which cannot change`. Each literal of the
same text points into the one block, so a write would change them all. The compiler
forbids such a write ([Text](../language/text.md)), and an edit of text or of a plain
list, or taking the address of one of its cells, first copies the static cells into a heap
block. Just an address made from an integer in unsafe code can reach a frozen block.

The stack region opens right after the statics, and the heap starts at its end. With two
integer globals, no text and the default limits, the stack region runs from `0x1040` to
`0x101040`, and the first heap block starts at `0x101040`. An address prints as a number,
so we can see where each block starts:

```zaka
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ဒုတိယ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ကို ပြသမည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ဒုတိယ တွင် သိမ်းထားမည်။
ဒုတိယ ကို ပြသမည်။
လိပ်စာ ဒုတိယ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
0x101040
0x101060
```

The second block does not reuse the address of the first, although the first was freed.
[The life of a heap block](#the-life-of-a-heap-block) explains the reason. These
addresses come from the VM: native code prints the addresses of the host instead.

## What a block contains

| Field | Meaning |
| --- | --- |
| start | the address of the first octet; a multiple of 16 |
| length | the octet count; kept after the block is freed |
| contents | the octets; dropped when the block is freed |
| origin | `heap`, `stack` (a local or parameter of a call), or `static` (a global or a static text) |
| live | whether the block is in use; a freed block remains in the table as a record |
| frozen | whether a write inside the block stops the program; set for static text |
| owner | the call depth that made a stack block; 0 for heap and static blocks |
| count | the number of values that share a counted block (text, a plain list, a shared object, a box, the record of a closure); the last one to let go frees it, and a block from a heap statement has no count |
| cell kind | the data kind (type) of each cell of a counted block, so the ring collector can walk the blocks its cells point at |
| pinned | whether a program took the address of one of its cells; a duplicate (copy) of a pinned block copies its cells and never shares them |

The VM keeps the table in order of start. To check an access of `n` octets at `a`, it
takes the last block that starts at or below `a` and checks that `a + n` is still inside
it.

### Red zones

A gap of at least 16 addresses, which belong to no block, follows each block. A read or
write just past the end of a block lands in the gap, never in the next block, so a
mistake of one cell too far is caught instead of reading a neighbour:

```text
  block A                gap                      block B
 ┌──────────────────┐ ░░░░░░░░░░░░░░░░░░░░░░ ┌──────────────────┐
 │ 0x1000 .. 0x1007 │ ░ 0x1008 .. 0x101f   ░ │ 0x1020 .. 0x1027 │
 └──────────────────┘ ░░░░░░░░░░░░░░░░░░░░░░ └──────────────────┘
          a read at 0x1008 lands in the gap: out of bounds
```

### The order of the checks

One function checks each load and store, in this order, and the first check that fails
stops the program:

| Check | Trap |
| --- | --- |
| 1. the address is below the reserved part | `null address` |
| 2. the address is not a multiple of the alignment of the data kind | `misaligned` |
| 3. no block contains the address, or the access runs past the end of the block it starts in | `out of bounds` |
| 4. the block is freed | `use after free` |
| 5. the octets read are no value of the data kind | `bad value` |

[Stops in unsafe code](traps-unsafe.md#addresses) shows each message with an example.

### Forming an address is free

Moving an address does not read an octet, so the VM checks just that each step remains
inside the block the address started in, one past its end included. A step that leaves
the block stops the program at once, since no offset leads from one block into another. A
subtraction of two addresses needs both in one live block, again counting the end.

## The life of a heap block

```text
            allocate                    free
 (no block) ───────▶  live  ─────────────────────▶  freed
                       │                             │
                       │ resize: a new block is      │ the record remains in the table,
                       │ made, and this one is       │ and its start joins the queue
                       │ freed                       │ of freed blocks: the quarantine
                       ▼                             ▼
              reads and writes work         reads, writes and frees stop the program
```

1. **Allocate.** A new block gets the lowest fresh address above each block made so far,
   filled with zeros, and counts against the heap cap. When the cap or the space has no
   room, the statement leaves the null address.
2. **Free.** The block becomes *freed*: its contents are dropped, but its start and length
   remain in the table, and its start joins a queue of freed blocks, oldest first. This is
   the *quarantine*. An access that lands in a freed block is a use after free, and
   freeing it again is a double free.
3. **Reuse.** The VM takes fresh addresses first, and takes the oldest freed block that is
   long enough just when the space is full. On a 64-bit platform that does not happen in
   practice, so a stale address keeps pointing at the record of a freed block, and the VM
   catches the mistake however late the address is used. On a small platform the
   quarantine is shorter, and a use after free can go unseen once the block is reused.
4. **Resize.** A resize makes a new block each time, copies as much of the old contents as
   fits, sets the rest to zero, and frees the old block. The new block can sit at another
   address, so an address taken before the resize points into a freed block afterwards.

The cap counts live heap octets. A free lowers the count, and the VM remembers the peak,
which the Explorer shows.

## Blocks of the stack region

A call puts one block for each local, parameters first, at the top of the stack region.
Each is aligned to 16 and has its red zone after it. When the call returns, the VM marks
its blocks freed and moves the top of the stack back to where it was before the call:

```text
 stack region, growing upward

 start calls f, f calls g             g returns                  start calls h

 ┌──────────────┐ top              ┌──────────────┐          ┌──────────────┐
 │ g: s, t      │                  │ (g: freed)   │          │ h: p, q      │ takes
 ├──────────────┤                  ├──────────────┤ top      ├──────────────┤ the room
 │ f: n, r      │                  │ f: n, r      │          │ f: n, r      │ of g
 └──────────────┘                  └──────────────┘          └──────────────┘
```

A freed stack block has no quarantine. It remains in the table until a later call takes its
room, and the table then forgets it. Until then, a read through the address of a local of
a function that has returned stops the program as a use after free, and the message names
the variable and its function. After another call has taken the room, the read finds the
new local, as it would in a native program. This is the reason the compiler warns about a
function that returns the address of one of its locals (ZK0706): see [Unsafe
Zaka](../language/unsafe.md).

A call whose locals do not fit in what is left of the region stops the program with
`stack full`.

## Checked and packed variables

The VM can place the variables of a run in two forms, and we pick one with `--frames`:

| | `checked`, the default | `packed` |
| --- | --- | --- |
| globals | one static block each | one static block for all, at C struct offsets |
| locals of a call | one stack block each | one stack block for all, at C struct offsets |
| a step past a variable | lands in its red zone: the program stops | reads or changes the next variable |
| a step past the whole frame, or past a heap block | the program stops | the program stops |
| a trap names | `x of f`, `global x` | `frame of f`, `globals` |

```text
 checked: each variable is a block

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

`packed` puts each variable at the next multiple of its alignment, in the order the
variables are declared, with parameters first. It is the shape a C compiler gives to the
locals of a function, and the shape a native Zaka program gives them (see [Checks in the
VM and in native code](../concepts/checked.md)). A program with no mistakes prints the
same in both modes.

The addresses of two globals show the difference. Here `program.zaka` declares two
integers `က` and `ခ` one after the other at its top level, and an unsafe block in its
program start block prints `က ၏ လိပ်စာ` and `ခ ၏ လိပ်စာ`:

```console
$ zakac run --vm program.zaka
0x1000
0x1020
$ zakac run --frames packed program.zaka
0x1000
0x1008
```

The checked run leaves 24 addresses between the end of `က` and the start of `ခ`, and the
packed run leaves none. `--frames` is a flag of the VM alone, so it sends the run to the
VM without `--vm`.

The difference matters when a program makes a mistake. Here an unsafe block takes the
address of `က`, writes `၉` one cell past it, and the program then prints `ခ`:

```console
$ zakac run --vm program.zaka
warning[ZK0710]: offset 1 is past the end of a block of 1 elements
zaka_vm: the program stopped: access of 8 octets at 0x1008 lands outside each live block (in ပရိုဂရမ်, at program.zaka:6:1); block at 0x1000, 8 octets, global က
$ zakac run --frames packed program.zaka
warning[ZK0710]: offset 1 is past the end of a block of 1 elements
၉
```

The checked mode stops at the write. The packed mode writes `၉` over `ခ`, so the program
prints it and goes wrong far from the line at fault, as native code would. We keep the
checked mode to find such mistakes, and use the packed mode to see what the checks
protect us from.

## Looking at the blocks

The [Explorer](/explorer/) draws the table of blocks while a program steps, with the
start, length, origin and state of each block. `zakac run --leaks` lists the live heap
blocks at the end of a run, once the ring collector has run, and each block that is still
live then was never freed ([Leaks](running.md#how-a-run-ends)).
