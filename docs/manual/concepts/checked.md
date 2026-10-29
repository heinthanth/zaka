---
title: Checks in the VM and in native code
description: Which mistakes the compiler finds, which the VM stops, and which checks a planned native build would keep.
---

# Checks in the VM and in native code

A mistake with storage can be found in three places: in the compiler, before the program
runs; in the VM, while it runs; or in neither, when the program runs as native code
with no checks, as a C program does. This page sorts each mistake into those places and
tells the reason for the split.

One part of this is not built. Now Zaka has one means to run a program: the VM.
Native backends, first a translation to C and then LLVM, are planned. Each statement below
about native code is the plan and the reasoning behind it, not a feature to use.

## Three places to look

```text
   the text of the program
          │
          │  the compiler reads each path, runs nothing
          ▼
   ┌─────────────┐   a mistake it can prove stops the build
   │ compile time │   (ZK0701 to ZK0712)
   └──────┬──────┘
          │ the program that remains
          ▼
   ┌─────────────┐   checks each read, write and free on the path
   │   the VM    │   that this run takes; stops with a trap
   └──────┬──────┘
          │ the same program, built as native code  (planned)
          ▼
   ┌─────────────┐   no checks of its own: what the compiler
   │ native code  │   proved still holds; the rest behaves as in C
   └─────────────┘
```

The compiler sees each path, but just what the text shows. The VM sees the numbers
of one run, but just the paths that run takes. Native code sees neither: it keeps what
the compiler proved, and loses the rest.

## Which mistake is found where

| Mistake | Before the run | In the VM | In native code (planned) |
| --- | --- | --- | --- |
| use after free, through an owner or a borrow | error ZK0704 | `use after free` | kept: proved before the run |
| double free | error ZK0701, ZK0702 | `double free` | kept |
| free of something that is not a heap block | error ZK0702, ZK0714 | `bad free` | kept |
| the address of a local outliving its call | error ZK0706, ZK0703 | `use after free`, naming the variable | kept |
| a leak of a local's block | error ZK0708 | `--leaks` lists it | kept |
| a null access on each path | error ZK0709 | `null address` | a crash, as in C |
| a null access on some path | warning ZK0711, with `--mem-strict` | `null address` | a crash |
| a box number past the end, known before the run | error ZK0710 | `out of bounds` | not checked |
| a box number past the end, known just in the run | nothing | `out of bounds` | not checked |
| pointers into two blocks subtracted | error ZK0712, when provable | `mixed blocks` | not checked |
| a use after free through a raw address | nothing | `use after free` | not checked |
| a misaligned address, or octets that hold no value | nothing | `misaligned`, `bad value` | not checked |
| the heap running out | nothing | allocate leaves `ဗလာ`; resize: `out of mem` | allocate leaves `ဗလာ`; a resize stops the program |

"Kept" means the compiler found the mistake, so the program that remains has none of it,
however it is run. The other rows depend on a check that the VM alone makes.

## A mistake that the VM alone sees

The compiler gives this function a borrowed block and a count. It cannot know that the
block holds 4 boxes while the loop goes to box 4:

```zaka-stop
// fill: write the box number into each box, from box 0 up to box အရေအတွက်
လုပ်ဆောင်ချက် ဖြည့် သည်
ငှားထားသော ကိန်းပြည့် လိပ်စာ က နှင့် ကိန်းပြည့် အရေအတွက် တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၀ မှ အရေအတွက် အထိ နံပါတ် အနေဖြင့် ရေတွက်၍ က + နံပါတ် ၏ တန်ဖိုး ကို နံပါတ် ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
"အစ" ဟု ပြသပါ။
လုပ်ဆောင်ချက် ဖြည့် ကို ပ ၏ တန်ဖိုးအငှား၊ ၄ တို့ဖြင့် လုပ်ဆောင်ပါ။
ပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
အစ
the program stopped: access of 8 octets at 0x101040 lands outside each live block
```

A counted loop includes its last number, so the loop writes boxes 0 to 4: five boxes into
a block of four. The VM stops at the fifth write, which lands in the gap after the
block. The program is correct in each respect the compiler can follow, since the owner, the
borrow and the free are all in order.

In C the same loop writes 8 octets past the block. Nothing stops it, and it changes what
comes next in the heap: the header of the next block, or the next variable. The program
goes wrong somewhere else, much later.

The VM can show this difference now. `--frames packed` sets the variables of a run
next to each other, as a C compiler does, so that a write past one variable changes the next
([Checked and packed blocks](../vm/blocks.md#checked-and-packed-variables)). It is a view
of what the same program does without the red zones. Heap blocks remain checked in both
modes, so this program stops in each.

## What the compiler cannot check

The compiler could refuse the program above if it knew the size of the block. It does
not, for two reasons:

- A size or an offset can come from input or from a loop. To prove it safe, the compiler
  would have to prove a fact about numbers, and it would often have to give up. When it
  cannot know, it reports nothing and leaves the check to the VM, instead of guessing.
  A check that guessed wrong in the other direction would refuse good programs.
- A raw address is the exit for code that shares storage in its own fashion, such as an
  arena. The compiler does not follow raw addresses, on purpose, and `--no-raw` forbids
  them when a lesson must not step around the check.

What the compiler does prove is a smaller set, and the proof is sound for that set. A
program with no raw kinds, compiled at `--mem-check error`, never reaches a use after
free, a double free or a bad free, and the address of a local never outlives its call.
[Ownership, explained](ownership.md) tells how.

## Native code, as planned

The plan keeps the VM as it is and adds other means to run the same program:

- **The VM remains the reference.** It runs programs for the Explorer and for the
  tests, and it is the place where a mistake is shown rather than suffered.
- **Native backends come after.** The first is a translation to C, which each C compiler
  can build, and the next is LLVM. Both read the same intermediate form the VM's
  instructions come from, so the arrangements agree: the size of each kind, the offset of each
  field, and the arithmetic on pointers are the same on each backend.
- **A native build has no run-time checks of its own.** It behaves like C. A pointer
  becomes a plain `T *`, a read becomes a plain load, and an owner that moves costs one
  store of `ဗလာ`. All that the compiler proved still holds. A mistake that the
  VM alone would have caught does what it does in C.
- **A debug flag could add cheap checks later**, such as a null check, or the sanitizer
  flags of the C compiler. That is an idea, not a promise.

For this reason the order matters. Ownership came before the native backends: it moves
the checks that matter most to the compiler, where such checks cost nothing at run time, before
the VM's checks are gone.

## Turning the checks up and down

| Option | What changes |
| --- | --- |
| `--mem-check error` | the compiler refuses programs with a proved mistake; the default |
| `--mem-check warn` | the same mistakes are warnings; the program runs, and the VM stops it |
| `--mem-check off` | the compiler reports none of them |
| `--mem-strict` | also warn about a pointer that can be `ဗလာ`, which the compiler cannot prove safe |
| `--no-raw` | refuse each raw address kind |
| `--frames packed` | the VM places variables next to each other, as C does |

The Explorer has the same settings. Use `warn` to watch the VM catch what the
compiler would have caught, and `packed` to see what a mistake does when nothing watches.
The compile-time side is in [pointer checks](../compiler/checks.md), and the VM's
side in [When a program stops](../vm/traps.md).
