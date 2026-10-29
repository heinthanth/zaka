---
title: Ownership, explained
description: The reason for one owner of each heap block, what that finds before a program runs, and what it costs, set against C and garbage collection.
---

# Ownership, explained

A program asks the heap for a block, uses it, and must give it back once. Three
questions follow, and the answers decide most of the storage mistakes a language allows:

1. **Who** gives the block back?
2. **When** is it given back?
3. **Can a pointer still use it** after that?

Zaka answers the first with a rule: *each block has one owner, and the owner alone gives
it back.* This page tells the reasons, walks through a scenario, and compares the rule
with C and with garbage collection. The rules themselves are in [Ownership and borrowing](../language/ownership.md), and [Learn](../../learn/pointers/ownership.md) teaches
them through examples.

## The three old mistakes

A block given back can be reached through other pointers that were copied from the owner.
Nothing in C ties the copies to the block, and three mistakes follow:

| Mistake | What happens | Where it shows |
| --- | --- | --- |
| use after free | a pointer reads or writes a block that was given back | much later, as wrong data, or never |
| double free | a block is given back twice | in the allocator, as a crash or worse |
| leak | the last pointer to a block is lost before the block is given back | never at once; the program grows |

The checks of the VM find the first two when a run reaches them
([When a program stops](../vm/traps.md)). A run reaches just the paths it takes. A
mistake on a path no one walked this time remains hidden until a run takes it. Ownership
moves the search to the compiler, which reads each path before the program runs.

## Three answers

| | C | garbage collection | Zaka |
| --- | --- | --- | --- |
| who gives a block back | the programmer, with a call | the runtime, after nothing can reach the block | the one owner, with a call, or it hands the block on |
| when | at the call to `free` | at a time the runtime picks | at the owner's call: the time is in the text |
| use after free | possible, silent | impossible | an error before the run (ZK0704), for owners and borrows |
| double free | possible | impossible | an error before the run (ZK0702, ZK0701) |
| leak | silent | rare: a block still reachable is kept | a warning or error for a local (ZK0708); `--leaks` lists the rest |
| cost at run time | none | the collector's work, and blocks kept longer | one store of `ဗလာ` when an owner moves |
| runtime support | none | a collector | none; the VM checks as a second line |
| cost to the writer | remembers each rule | little | writes `ငှားထားသော`, learns moves, cannot keep a borrow in the heap |

Garbage collection takes the whole question off the programmer, at the price of a runtime and of not
knowing when a block goes. C leaves it all to the programmer. Zaka sits between: the
programmer still decides when a block goes, as in C, and the compiler checks that the
decisions agree with each other, as a collector would never need to.

## One owner, several borrows

The rule has three parts.

- A pointer of the plain kind, `ကိန်းပြည့် လိပ်စာ`, is an **owner**. Setting one owner from
  another *moves* the block, and the old one holds `ဗလာ` from then on.
- A pointer of the kind `ငှားထားသော ကိန်းပြည့် လိပ်စာ` is a **borrow**. It reads and writes
  the block, and never gives it back. Several borrows of one block can live at once.
- A pointer of the kind `ကိန်းပြည့် လိပ်စာအကြမ်း` is **raw**: the compiler does not follow
  it, and the VM still checks each use.

The compiler follows each owner and each borrow through each path of the program. It
refuses to let the owner give the block back, move it, or resize it while a borrow of it
is still used later.

## A scenario in steps

A program keeps three scores in a heap block. A function adds them up. Another function
gives the block back. The right program is:

```zaka
// sum: add up the boxes of a borrowed block
လုပ်ဆောင်ချက် ပေါင်း သည်
ငှားထားသော ကိန်းပြည့် လိပ်စာ က နှင့် ကိန်းပြည့် အရေအတွက် တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
ရလဒ် ကို ၀ ဟု ထားပါ။
ကိန်းရှင် ညွှန်း သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ကာ ညွှန်း သည် က + အရေအတွက် အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ရလဒ် ကို ရလဒ် + *ညွှန်း ဟု ထားပါ။
ပြီးလျှင် ညွှန်း ကို ညွှန်း + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

// clear: take the block and give it back
လုပ်ဆောင်ချက် ရှင်း သည်
ကိန်းပြည့် လိပ်စာ က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၃ ခုစာ နေရာယူ၍ လိပ်စာကို ကိန်းများ တွင် သိမ်းထားမည်။
၀ မှ ၂ အထိ နံပါတ် အနေဖြင့် ရေတွက်၍ ကိန်းများ + နံပါတ် ၏ တန်ဖိုး ကို နံပါတ် + ၁ ဟု ထားပါ။
လုပ်ဆောင်ချက် ပေါင်း ကို ကိန်းများ ၏ တန်ဖိုးအငှား၊ ၃ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
လုပ်ဆောင်ချက် ရှင်း ကို ကိန်းများ တို့ဖြင့် လုပ်ဆောင်ပါ။        // the block goes to ရှင်း
```

```text
၆
```

Who holds the block at each step? The names in the diagrams are `ကိန်းများ` in `main`,
and `က` inside each function:

```text
 1. main asks for the block        ကိန်းများ ══owns══▶ [ 1 | 2 | 3 ]

 2. ပေါင်း borrows it              ကိန်းများ ══owns══▶ [ 1 | 2 | 3 ] ◀──borrows── က of ပေါင်း
    (main keeps it)                the call ends; the borrow ends with it

 3. main hands it to ရှင်း         ကိန်းများ  ဗလာ
                                   က of ရှင်း ══owns══▶ [ 1 | 2 | 3 ]

 4. ရှင်း gives it back             က of ရှင်း  (gone)       [ freed ]
```

The header of each function tells which of the two it does. The call of `ပေါင်း` writes
`ကိန်းများ ၏ တန်ဖိုးအငှား`, which lends the block: `ကိန်းများ` keeps it. The call of
`ရှင်း` writes `ကိန်းများ` alone, which hands the block over.

### The mistake

Now add one line at the end, which reads the block after it was handed over. In C, this
is a use after free hidden inside another function, and nothing at the call site shows
it. In Zaka it is an error:

```zaka-error ZK0701
လုပ်ဆောင်ချက် ရှင်း သည်
ကိန်းပြည့် လိပ်စာ က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၃ ခုစာ နေရာယူ၍ လိပ်စာကို ကိန်းများ တွင် သိမ်းထားမည်။
လုပ်ဆောင်ချက် ရှင်း ကို ကိန်းများ တို့ဖြင့် လုပ်ဆောင်ပါ။
ကိန်းများ ၏ တန်ဖိုး ကို ပြသပါ။                       // the block is gone
```

```text
error[ZK0701]: "ကိန်းများ" was moved to another owner and now holds ဗလာ
```

The message points at the line that reads, and the header of `ရှင်း` is where the fault
comes from. Had the owner been checked while the program ran alone, the mistake would show
just when this line was reached, and the VM would stop it with `access through a
null address`.

### A borrow that outlives its block

The block can also go while a borrow is waiting. A resize makes a new block each time, copies
the contents, and gives the old block back, so a borrow made before the resize points at a
block that is gone:

```text
 before the resize                   after the resize

 ပ ══owns══▶ [A: 2 ints]             ပ ══owns══▶ [B: 200 ints]
 ကြည့် ──borrows──▶ [A]               ကြည့် ──borrows──▶ [A: freed]   stale
```

```zaka-error ZK0704
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် ကြည့် သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
ပ လိပ်စာရှိကို မင်မိုရီ ကိန်းပြည့် ၂၀၀ ခုစာ ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။
ကြည့် ၏ တန်ဖိုး ကို ပြသပါ။
ပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
error[ZK0704]: the block of "ပ" is resized here, but "ကြည့်" still borrows it and is used later
```

This one is simple to write and hard to see in C: the code works until the allocator picks a
new place for the block. The compiler reports it each time, however the allocator
chooses. With the check turned down to a warning, the program runs, and the VM's
quarantine of freed blocks stops it:

```zaka-stop mem-check=warn
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် ကြည့် သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
ပ လိပ်စာရှိကို မင်မိုရီ ကိန်းပြည့် ၂၀၀ ခုစာ ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။
ကြည့် ၏ တန်ဖိုး ကို ပြသပါ။
ပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
the program stopped: access at 0x101040 inside a freed block
```

## What it costs

- **Kinds to write.** A function that just looks at a block marks it with `ငှားထားသော`.
  A call that lends writes `တန်ဖိုးအငှား`. The marks are the price of a check that needs no
  other annotation.
- **Borrows are second-class.** A borrow lives in a local variable or a parameter. It
  cannot be kept in a heap block or behind a pointer, and a function cannot return a
  borrow of its own variable (ZK0703, ZK0706). That is what lets the compiler infer how
  long a borrow lasts, with no lifetime marks to write.
- **More shapes are refused.** The address of a borrow cannot be taken (ZK0705), and a
  call that could give a block back through one argument while another argument reaches
  the same block is refused (ZK0707).
- **Moves are visible.** Setting one owner from another empties the first. Reading it
  afterwards is an error, and comparing it with `ဗလာ` is the one read still allowed.
- **All borrows can write.** Unlike Rust, Zaka does not forbid two borrows that write
  to one block. Giving back, moving and resizing alone conflict with a live borrow. A
  program has no threads, so no two things run at once.
- **Some code needs to step outside.** A hand-made allocator hands out pieces of one
  block. No owner or borrow describes a piece, so the pieces are raw addresses. The
  compiler does not follow them and the VM still checks them; `--no-raw` refuses
  them for a lesson that must not step around the check.

## What it promises

A program with no raw kinds, compiled with `--mem-check error` (the default), never
reaches a use after free, a double free or a bad free, and the address of a local never
outlives its call. That holds in the VM and would hold in native code alike, since
the compiler proved it from the text.

It does not promise freedom from a null access on a path where a pointer can be `ဗလာ`,
an access past the end of a block when the offset is not known before the run, octets
that hold no value, running out of heap, or a leak through the heap. Those remain checks of
the VM. [Checks in the VM and in native code](checked.md) sorts all of them.

## Where to go next

- [Ownership and borrowing](../language/ownership.md) states the rules in full.
- [The borrow checker](../internals/borrow-checker.md) tells how the compiler follows an
  owner.
- [Pointer mistakes](../../learn/pointers/mistakes.md) shows each mistake with the
  message of the compiler and the message of the VM.
