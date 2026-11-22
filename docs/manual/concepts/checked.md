---
title: Checks in the VM and in native code
description: Which mistakes the compiler finds, which the VM stops, and which checks a native build keeps.
---

# Checks in the VM and in native code

A storage mistake can be found in three places: in the compiler, before the program runs,
in the VM (virtual machine), while it runs, or nowhere, when the program runs as native
code with no checks, as a C program does. This page sorts each mistake into one of those
places and explains the reason for the split.

Native code comes from several commands. `zakac build --backend c` writes the program as C
and compiles it, `zakac build --backend native` compiles it to machine code with Cranelift,
and `zakac run --jit` runs that Cranelift code at once. A plain `zakac run`, with neither
`--vm` nor `--jit`, also runs a source file as native code when it can, and on the VM
otherwise, so to watch the checks of the VM we run `zakac run --vm`. The `--checks` option
of a build decides how much of the checking of the VM a native program keeps, from none up to the
null checks and, on the C target, the sanitizers of the C compiler, and [Native
programs](../compiler/native.md) has the details.

## Three places to look

```text
   the source of the program
          │
          │  the compiler follows each path, runs nothing
          ▼
   ┌──────────────┐   a mistake it can prove is a warning in unsafe
   │ compile time │   code, or stops the build (ZK0701 to ZK0712)
   └──────┬───────┘
          │ the program that remains
          ▼
   ┌──────────────┐   checks each read, write and free on the path
   │    the VM    │   that this run takes; stops with a trap
   └──────┬───────┘
          │ the same program as native code  (--backend c, --backend native, run, run --jit)
          ▼
   ┌──────────────┐   no checks of storage: what the compiler
   │ native code  │   proved is still true; the rest behaves as in C
   └──────────────┘
```

The compiler sees each path, but it knows just what the source shows. The VM sees the
real numbers of one run, but just the paths that this run takes. Native code sees
neither, so it keeps what the compiler proved and loses the rest.

## Which mistake is found where

Safe code makes none of the mistakes below, because it never reads or writes through a
pointer, and the program frees each block it makes, rings of objects included. Safe code
has no null either: a shared object and a function value have a value at each point, and the
compiler proves that each variable, field and cell is set before a read, as an error in
each mode (ZK0722, see [Set before read](../language/declarations.md#set-before-read)). So each
row below is a mistake of [unsafe code](../language/unsafe.md), where the findings of the
compiler are warnings, or errors under `--lint-error mem`.

| Mistake | Before the run | On the VM | In native code (`--checks default`) |
| --- | --- | --- | --- |
| use after free | ZK0704 | `use after free` | as in C |
| double free | ZK0704 | `double free` | as in C |
| free of something that is not a heap block | ZK0702 | `bad free` | as in C |
| the address of a local outliving its call | ZK0706, ZK0703 | `use after free`, naming the variable | as in C |
| a read after an owned hand-over, on each path | error ZK0701 | `null address` | kept: an error in each mode |
| a leak of a block this function asked for | ZK0708 | `--leaks` lists it | as in C |
| a null access on each path | ZK0709 | `null address` | a crash, as in C; `null address` with `--checks mem` |
| a null access on some path | ZK0711, with `--lint-warn doubts` | `null address` | a crash; `null address` with `--checks mem` |
| a cell past the end, known before the run | ZK0710 | `out of bounds` | not checked; the sanitizer of the C target with `--checks mem` |
| a cell past the end, known in the run alone | nothing | `out of bounds` | not checked; the sanitizer of the C target with `--checks mem` |
| a pointer step further than one past the end | ZK0710, for a constant offset | `step out of bounds` | not checked |
| pointers into two blocks subtracted | ZK0712, when provable | `mixed blocks` | not checked |
| a misaligned address, or octets that form no value | nothing | `misaligned`, `bad value` | not checked |
| the heap running out | nothing | asking for a block leaves the null address; a resize stops with `out of mem` | asking for a block leaves the null address; a resize stops the program |

"As in C" means that the native program does what a C program with the same mistake does,
unless `--lint-error mem` kept the program from compiling. "Kept" means that the compiler
refuses the mistake in each mode, so the program that remains does not have it, however we
run it. The other rows depend on a check that the VM alone makes.

## A mistake that the VM alone sees

In this program, the compiler sees an unsafe function that gets a block and a count. It
cannot know that the block has room for 4 cells while the loop goes up to cell 4:

```zaka-stop
// fill: write the cell number into each cell, from cell 0 up to cell အရေအတွက်
ဖြည့် ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း
အရေအတွက် ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
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

A counted loop includes its last number, so the loop writes cells 0 to 4, which is five
cells into a block of four. On the VM (`zakac run --vm`), the program stops at the fifth
write, which lands in the gap after the block. Each step that the compiler can follow is
correct, because the program asks for the block, uses it and frees it in order.

In native code, which is what a plain `zakac run` makes of this program, the same loop
writes 8 octets (bytes) past the block. Nothing stops it, and the program prints `အစ`
and ends as if nothing went wrong, while the write has changed whatever came next in the
heap: the bookkeeping of the next block, or another value. In a larger program, the
damage shows up somewhere else, much later.

The VM can also show this difference. `--frames packed` places the variables of a call
next to each other, as a C compiler does, so a write past one variable changes the next
one ([Addresses and blocks](../vm/blocks.md#checked-and-packed-variables)). It shows what
the same program does without the guard gaps between variables. Heap blocks remain checked
in both modes, so this program stops in both.

## What the compiler cannot check

The compiler could refuse the program above if it knew the size of the block. It does not,
for two reasons:

- A size or an offset can come from input or from a loop. To prove such an access safe,
  the compiler would have to prove a fact about numbers, and it would often have to give
  up. When it cannot know, it reports nothing and leaves the check to the VM, because a
  check that guessed would refuse good programs.
- A pointer can come from C, from an integer, or from an object turned into an address.
  Unsafe code is the exit for work that shares storage in its own fashion, such as an
  arena, and the compiler does not attempt to follow each such pointer. It warns about
  what it can see, and `--no-unsafe` forbids unsafe code when a lesson must not step
  around the checks.

So the compiler proves a smaller set of facts, and its proof is sound for that set. A
program that compiles under `--no-unsafe` never meets a use after free, a double free, a
bad free or a leak, because it has no pointer to misuse and the program frees each block
it makes. [Storage, explained](storage.md) explains how.

## Native code

The VM remains as it is, and two targets add native code for the same program: the C
target, and the Cranelift target, which builds an executable or runs the code at once.

- **The VM is the reference.** It runs programs for the Explorer and for the tests, and it
  is the place where a mistake is shown instead of suffered.
- **The native targets start from what the code generator of the VM reads.** Each starts
  from the same intermediate form, so the arrangements agree: the size of each data kind
  (type), the offset of each field and the arithmetic on pointers are the same on each
  backend. A program that runs clean on the VM prints the same output in native code.
- **The checks of the language remain at the default level.** A zero divisor, an overflow
  under `--overflow stop`, a cell number outside its list, a range of text that cuts a
  character, a call through a function value that has no function, and a C function that
  returns `NULL` for a handle result written with no optional all stop a native program
  with the message that the VM gives.
- **The checks of storage remain in the compiler alone.** A native build behaves like C
  there: a pointer is a plain address, and an owned hand-over costs one store of the null
  address. Safe code counts its blocks and frees them, as it does on the VM. All that the
  compiler proved is still true, but a mistake that the VM alone would have caught does what
  it does in C, unless `--checks mem` asks for the null checks and, on the C target, for
  the sanitizers of the C compiler.

That is the reason the split matters. Safe code needs no check of storage at run time,
because the counts free each block and no pointer reaches it. Unsafe code is the small
part of a program that loses the checks of the VM in native code, and the warnings of the
compiler point at it.

## Turning the checks up and down

| Option | What changes |
| --- | --- |
| `--lint-warn mem` | a proved mistake in unsafe code is a warning, and the program runs; the default |
| `--lint-error mem` | the compiler refuses programs with a proved mistake in unsafe code |
| `--lint-allow mem` | the compiler reports none of these mistakes |
| `--lint-warn doubts` | also warn about a pointer that can point at nothing, which the compiler cannot prove safe |
| `--no-unsafe` | refuse unsafe code |
| `zakac run --vm` | run on the VM, which stops each storage mistake it meets |
| `--frames packed` | the VM places the variables of a call next to each other, as C does |
| `--checks none`, `default`, `mem` | for `zakac build --backend c` and `--backend native`: no checks past what C has, the checks of the language, or those and the storage checks; `zakac run` keeps `default` when it runs native code |

The Explorer runs programs on the VM, and it has the same settings. We can use `warn` to
watch the VM catch what the compiler saw, and `packed` to see what a mistake does when
nothing watches. The compile-time side is in [Compiler checks](../compiler/checks.md), and
the side of the VM is in [When a program stops](../vm/traps.md).
