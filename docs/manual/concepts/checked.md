---
title: Checks in the VM and in native code
description: Which mistakes the compiler finds, which the VM stops, and which checks a native build keeps.
---

# Checks in the VM and in native code

A mistake with storage can be found in three places: in the compiler, before the program
runs; in the VM, while it runs; or in neither, when the program runs as native code
with no checks, as a C program does. This page sorts each mistake into those places and
tells the reason for the split.

Native code comes from the C target: `zakac build --target c` writes a program as C and
compiles it. Its `--checks` option picks how much of the VM's watch a native program keeps,
from none to the sanitizers of the C compiler; [Native programs](../compiler/native.md) has
the details. LLVM comes later.

## Three places to look

```text
   the text of the program
          │
          │  the compiler reads each path, runs nothing
          ▼
   ┌─────────────┐   a mistake it can prove is a warning in unsafe
   │ compile time │   code, or stops the build (ZK0701 to ZK0712)
   └──────┬──────┘
          │ the program that remains
          ▼
   ┌─────────────┐   checks each read, write and free on the path
   │   the VM    │   that this run takes; stops with a trap
   └──────┬──────┘
          │ the same program, built as native code  (--target c)
          ▼
   ┌─────────────┐   no checks of its own: what the compiler
   │ native code  │   proved still holds; the rest behaves as in C
   └─────────────┘
```

The compiler sees each path, but just what the text shows. The VM sees the numbers
of one run, but just the paths that run takes. Native code sees neither: it keeps what
the compiler proved, and loses the rest.

## Which mistake is found where

Safe code makes none of the mistakes below: it never reads or writes through a pointer,
and the VM frees each block it makes, rings of objects included. It holds no NULL either:
a shared object and a function value hold a value at each point, and the compiler proves that each
variable, field and cell is set before a read, as an error in each mode (ZK0722; see
[Statements](../language/statements.md#set-before-read)). Each row is a mistake of
[unsafe code](../language/unsafe.md), where the compiler's findings are warnings, and
errors under `--mem-check error`.

| Mistake | Before the run | In the VM | In native code (`--checks default`) |
| --- | --- | --- | --- |
| use after free | ZK0704 | `use after free` | as in C |
| double free | ZK0704 | `double free` | as in C |
| free of something that is not a heap block | ZK0702 | `bad free` | as in C |
| the address of a local outliving its call | ZK0706, ZK0703 | `use after free`, naming the variable | as in C |
| a read after an owned hand-over, on each path | error ZK0701 | `null address` | kept: an error in each mode |
| a leak of a block this function asked for | ZK0708 | `--leaks` lists it | as in C |
| a null access on each path | ZK0709 | `null address` | a crash, as in C; `null address` with `--checks mem` |
| a null access on some path | ZK0711, with `--mem-strict` | `null address` | a crash; `null address` with `--checks mem` |
| a box number past the end, known before the run | ZK0710 | `out of bounds` | not checked; the sanitizer with `--checks mem` |
| a box number past the end, known just in the run | nothing | `out of bounds` | not checked; the sanitizer with `--checks mem` |
| pointers into two blocks subtracted | ZK0712, when provable | `mixed blocks` | not checked |
| a misaligned address, or octets that hold no value | nothing | `misaligned`, `bad value` | not checked |
| the heap running out | nothing | allocate leaves the null pointer; resize: `out of mem` | allocate leaves the null pointer; a resize stops the program |

"As in C" means the native program does what a C program with the same mistake does,
unless `--mem-check error` kept the program from compiling. "Kept" means the compiler
refuses the mistake in each mode, so the program that remains has none of it, however it
is run. The other rows depend on a check that the VM alone makes.

## A mistake that the VM alone sees

The compiler gives this unsafe function a block and a count. It cannot know that the
block holds 4 boxes while the loop goes to box 4:

```zaka-stop
// fill: write the box number into each box, from box 0 up to box အရေအတွက်
စိတ်မချရသော လုပ်ဆောင်ချက် ဖြည့် သည်
က ဟူသော ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း အရေအတွက် ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၀ မှ အရေအတွက် အထိ နံပါတ် အနေဖြင့် ရေတွက်၍ က + နံပါတ် ၏ တန်ဖိုး ကို နံပါတ် ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
"အစ" ဟု ပြသမည်။
ဖြည့် ဟူသော လုပ်ဆောင်ချက်ကို ပ နှင့် ၄ တို့ဖြင့် လုပ်ဆောင်မည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အစ
the program stopped: access of 8 octets at 0x101040 lands outside each live block
```

A counted loop includes its last number, so the loop writes boxes 0 to 4: five boxes into
a block of four. The VM stops at the fifth write, which lands in the gap after the
block. The program is correct in each respect the compiler can follow, since the block is
asked for, used and given back in order.

In C the same loop writes 8 octets past the block. Nothing stops it, and it changes what
comes next in the heap: the header of the next block, or the next variable. The program
goes wrong somewhere else, much later.

The VM can show this difference now. `--frames packed` sets the variables of a run
next to each other, as a C compiler does, so that a write past one variable changes the next
([Addresses and blocks](../vm/blocks.md#checked-and-packed-variables)). It is a view
of what the same program does without the red zones. Heap blocks remain checked in both
modes, so this program stops in each.

## What the compiler cannot check

The compiler could refuse the program above if it knew the size of the block. It does
not, for two reasons:

- A size or an offset can come from input or from a loop. To prove it safe, the compiler
  would have to prove a fact about numbers, and it would often have to give up. When it
  cannot know, it reports nothing and leaves the check to the VM, instead of guessing.
  A check that guessed wrong in the other direction would refuse good programs.
- A pointer can come from C, from an integer, or from an object turned into an address.
  Unsafe code is the exit for work that shares storage in its own fashion, such as an
  arena, and the compiler does not attempt to follow each such pointer. It warns about what it
  can see, and `--no-unsafe` forbids unsafe code when a lesson must not step around the
  checks.

What is proved is a smaller set, and the proof is sound for that set. A program that
compiles under `--no-unsafe` never reaches a use after free, a double free, a bad free or
a leak, since it has no pointer to misuse and the VM frees each block it makes.
[Storage, explained](storage.md) tells how.

## Native code

The VM remains as it is, and the C target adds a second means to run the same program:

- **The VM remains the reference.** It runs programs for the Explorer and for the
  tests, and it is the place where a mistake is shown rather than suffered.
- **The C target reads what the VM's code generator reads.** Both start from the same
  intermediate form, so the arrangements agree: the size of each kind, the offset of each
  field, and the arithmetic on pointers are the same on each backend. A program that runs
  clean in the VM prints the same output as native code.
- **The checks of the language remain at the default level.** A zero divisor, an overflow under
  `--overflow stop`, a cell outside its list, a range of text that cuts a character, a
  call through a function value that holds no function, and C that gives NULL for a handle
  result written with no optional stop a native program with the VM's message.
- **The checks of storage remain in the compiler alone.** A native build behaves like C
  there: a pointer is a plain address, and an owned hand-over costs one store of the null pointer.
  Safe code counts its blocks, and frees them, as it does in the VM.
  All that the compiler proved still holds. A mistake that the VM alone would have caught
  does what it does in C, unless `--checks mem` asks for the null checks and the sanitizers
  of the C compiler.

For this reason the split matters. Safe code needs no check of storage at run time: the
counts free each block, and no pointer reaches it. Unsafe code is the small part of a
program where the VM's checks are gone in a native build, and the compiler's warnings
point at it.

## Turning the checks up and down

| Option | What changes |
| --- | --- |
| `--mem-check warn` | a proved mistake in unsafe code is a warning; the program runs, and the VM stops it; the default |
| `--mem-check error` | the compiler refuses programs with a proved mistake in unsafe code |
| `--mem-check off` | the compiler reports none of them |
| `--mem-strict` | also warn about a pointer that can point at nothing, which the compiler cannot prove safe |
| `--no-unsafe` | refuse unsafe code |
| `--frames packed` | the VM places variables next to each other, as C does |
| `--checks none`, `default`, `mem` | for `zakac build --target c`: no check past what C has, the checks of the language, or those and the mem checks |

The Explorer has the same settings. Use `warn` to watch the VM catch what the compiler
saw, and `packed` to see what a mistake does when nothing watches. The compile-time side
is in [Compiler checks](../compiler/checks.md), and the VM's side in [When a program
stops](../vm/traps.md).
