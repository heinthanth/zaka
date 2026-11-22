---
title: Pointer mistakes
description: The mistakes that pointers make possible, the warnings the compiler gives for them, and how the VM stops each one.
---

# Pointer mistakes

A pointer reaches straight into RAM (memory), which makes it powerful and also simple to
misuse. In a lot of other languages, a mistake with RAM breaks the program without a
warning, but Zaka catches these mistakes twice: in unsafe code, the compiler warns about
each one that it can prove from the text of the program, and the VM checks each read and
write through a pointer while the program runs, stops the program at the first mistake,
and reports what went wrong.

## You will learn

- what happens when a program reads through a null address (null pointer), or past the end
  of a block (allocation) or a variable
- what happens when a block is used or given back after it has gone
- how a program loses a block, and how to find lost blocks
- which error codes the compiler gives for pointer mistakes

The checks of the VM work when the program runs on the VM, so we run the programs of this
page in the Explorer or with `zakac run --vm`. Plain `zakac run` runs a program as native
code when it can, and there a pointer mistake does what it would do in C.

Each program below runs with the default level, `--lint-warn mem`, so the compiler prints
its warning, the program runs, and the VM stops it. With `--lint-error mem`, or **Error**
for **Pointer check** in the Explorer's settings, the same warnings are errors and nothing
runs:

```zaka-error ZK0709 lint-error=mem
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။ // the null address
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
error[ZK0709]: "ပ" points at nothing here, so there is nothing to read or write through it
```

## Reading through a null pointer

```zaka-stop
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။ // the null address
"အစ" ဟု ပြသမည်။
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
အစ
the program stopped: access through a null address at 0x0
```

The compiler warns with ZK0709, and the VM stops the read, because `ပ` has the null
address, so there is no value to read. To fix it, we give the pointer an address first,
or compare it with `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` before we read through it.

## Reading past the end

```zaka-stop
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ + ၃ ၏ တန်ဖိုး ကို ပြသမည်။ // the last cell: fine
ပ + ၄ ၏ တန်ဖိုး ကို ပြသမည်။ // one past the end
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
၀
the program stopped: access of 8 octets at 0x101020 lands outside each live block
```

A block of 4 integers has cells 0 to 3, so cell 4 is outside it. The compiler warns with
ZK0710, and the VM stops the second read. The first read prints `၀`, because on the VM a
new block starts out filled with zeros. To fix it, we count from 0 and stop one before the
size.

The compiler sees this mistake because the size and the cell number are both written in
the program. When one of them comes from input, the check of the VM is the one that
catches it.

## Reading past a variable

```zaka-stop
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။ // points at က
ပ + ၁ ၏ တန်ဖိုး ကို ပြသမည်။ // one past က
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
the program stopped: access of 8 octets at 0x1008 lands outside each live block
```

On the VM, each variable has a separate block with a gap after it, the same as the heap
blocks above. The address of a variable points at a block of one cell, so a step past it
lands in the gap and the VM stops the read. To fix it, we keep a pointer to a variable on
that one variable.

### Checked and packed variables

The VM can set out variables in two forms:

| Mode | Where the variables go | A step past a variable |
| --- | --- | --- |
| checked, the default | each in a separate block, with a gap after it | stops the program |
| packed | the locals of a call next to each other in one block, as C does, and the globals in one more | reads or changes the next variable |

To pick packed, we run the program with `zakac run --frames packed`, or pick **Packed, as
C** for **Variables** in the Explorer's settings. The program above then prints `၂`, the
value of `ခ`, and goes on as if nothing were wrong.

Packed is how a C compiler sets out the variables of a function, and there a step past a
variable changes its neighbour with no warning, so the program goes wrong far from the line
at fault. Heap blocks keep their checks in both modes, and so does a step past the whole
block of a call. We keep the checked mode while learning, and switch to packed to see what
the checks catch.

## Using a block after giving it back

```zaka-stop
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားမည်။
ကိန်းရှင် မ သည် ပ ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
မ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
the program stopped: access at 0x101000 inside a freed block
```

`မ` has a duplicate (copy) of the address in `ပ`. Giving the block back through `ပ` sets `ပ`
to the null address, but `မ` still has the old address, so the read through `မ` reaches a
freed block. The compiler warns with ZK0704, at the give-back and at the later use. To fix
it, we swap the last two lines of the unsafe block, so that the read comes before the
give-back, and the program prints `၇`. Resizing a block while a duplicate of its address
is still in use is the same mistake, because the block can move.

## Giving a block back twice

```zaka-stop
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် မ သည် ပ ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လိပ်စာ မ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
the program stopped: the block at 0x101000 was freed before
```

Giving the block back through `မ` as well gives it back a second time. The compiler warns
with ZK0704, and the VM stops the second give-back. Giving back `ပ` a second time would do
no harm, because the first time set it to null. To fix it, we keep each block in one
variable that gives it back, and hand it to another function with
[`အပိုင်အနေဖြင့်`](ownership.md#owned-results-and-parameters).

## Giving back a block that was not asked for

```zaka-stop
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
the program stopped: 0x1000 is not the start of a live heap block
```

We can give back the blocks that come from `နေရာယူ၍`, but not the storage of a variable.
The compiler warns with ZK0702, and the VM stops the give-back. An address inside a heap
block, such as `ပ + ၂`, is not the start of a block either, so giving it back stops the
program too. To fix it, we give back the address that `နေရာယူ၍` put in the pointer, and
nothing else.

## Returning the address of a local

```zaka-stop
မှား ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် လိပ်စာ တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် x သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
x ၏ လိပ်စာ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် မှား ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
the program stopped: access at 0x1020 inside a freed block
```

The variables of a function end with the call, so when the function returns, their RAM is
given back. A function that returns the address of one of them leaves its caller a pointer
into storage that is gone, so the compiler warns with ZK0706 and the VM stops the read.
Pointing *into* a function at the caller's variables is fine, because the caller's
variables outlive the call. To fix it, room that must outlive the function that makes it
comes from the [heap](heap.md), and the function hands it to the caller as an [owned
result](ownership.md#owned-results-and-parameters).

## Losing a block

```zaka
စမ်း ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
မင်မိုရီတွင် ကိန်းပြည့် ၈ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။ // the first block is lost
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။ // the second is lost too

အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
စမ်း ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
၀
```

When the last pointer to a block is overwritten, or ends with its function, the block can
never be given back. The compiler warns with ZK0708, and the program runs to its end,
because a lost block does no harm at once. Called in a loop, though, such a function eats
the heap a little at a time. To fix it, we give each block back before its last pointer
goes.

The compiler checks each overwrite of a pointer that has a block it made, and the end of
each function. To find the blocks that remain when the program ends, we run it with
`zakac run --leaks`, which lists each forgotten block. `samples/mistakes/leak.zaka` asks
for a block three times and frees the last one alone:

```console
$ zakac run --leaks samples/mistakes/leak.zaka
warning[ZK0708]: the block that "ပ" holds is never given back or handed over on this path
 --> samples/mistakes/leak.zaka:7:1
  |
7 | မင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
၁၀
၂၀
၃၀
leak: block at 0x101000, 16 octets
leak: block at 0x101020, 16 octets
```

Text, lists, shared objects and closures never show up in this list, because the compiler
frees them.

## Mistakes the compiler catches

The ZK04 codes are about data kinds and unsafe code, and each one is an error in each mode. The
ZK07 codes come from the pointer check: in unsafe code, each one follows the level of the
`mem` lint, and in safe code, each one is an error.

| Code | Mistake |
| --- | --- |
| ZK0410 | assigning to something that is not a variable or a value through a pointer |
| ZK0411 | taking the address of something that has none, such as `၅` |
| ZK0412 | taking the address of a constant or a counted-loop variable |
| ZK0413 | reading through a value that is not a pointer |
| ZK0414 | reading through, or adding to, an opaque `လိပ်စာ` |
| ZK0415 | using an operator such as `*` or `/` on a pointer |
| ZK0416 | comparing or subtracting pointers of different data kinds |
| ZK0417 | a heap statement given something other than a pointer |
| ZK0468 | pointer work, or a call of an unsafe function, outside unsafe code |
| ZK0469 | an unsafe function as a function value |
| ZK0470 | `အပိုင်အနေဖြင့်` on a data kind that is not a pointer |
| ZK0471 | unsafe code under `--no-unsafe` |
| ZK0472 | a C function not declared unsafe |
| ZK0701 | reading a variable after it handed its block to an `အပိုင်အနေဖြင့်` parameter: an error in each mode |
| ZK0702 | giving back or resizing the address of a variable |
| ZK0703 | keeping the address of a variable in a heap block, behind a pointer, or in a top-level variable from a function |
| ZK0704 | using, giving back or resizing a block after it was given back, resized or moved |
| ZK0706 | returning the address of a function's own variable |
| ZK0707 | a call that hands a block over through one argument while another argument reaches the same block |
| ZK0708 | losing a block: its pointer is overwritten, or its function ends, while it has the block |
| ZK0709 | reading or writing through a pointer that has the null address on each path to the read |
| ZK0710 | a cell number past the end of its block |
| ZK0711 | a warning, with `--lint-warn doubts` alone: using a pointer that can have the null address |
| ZK0712 | subtracting pointers into different blocks |
| ZK0721 | a `ကူးယူမည်` whose source and destination share cells |

For example, `၏ တန်ဖိုး` on an integer is ZK0413, because an integer is not an address:

```zaka-error ZK0413
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
က ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
error[ZK0413]: isize is not a pointer and cannot be dereferenced
```

The [error messages](../../manual/reference/errors.md) page lists each code, and [What the
VM still stops](../../manual/language/unsafe.md#what-the-vm-still-stops) lists each stop
of the VM.

## Recap

- The compiler warns about each pointer mistake that it can prove, and the VM stops the
  program at the first bad access, so we run these programs in the Explorer or with
  `zakac run --vm`.
- Reading through a null address, past the end of a block, or past a variable stops the
  program.
- A duplicate of an address does not change when the block is given back or moved, so a
  use of it later stops the program.
- A block that loses its last pointer can never be given back, and `zakac run --leaks`
  lists such blocks.

## Hands on

- Bring a new fault into each program on this page, and read the message.
- Fix the "past the end" program so that it prints all four cells and nothing more.
- Run the "past a variable" program with `--frames packed`. Then write `၉` through
  `ပ + ၁` and print `ခ`. Which variable changed?
- Step through each program on this page in the Explorer to the line where the VM stops it.
- Write an unsafe function that asks for a block and returns its address. What must its
  function signature have? (`အပိုင်အနေဖြင့်`, since the caller owns the block from then
  on.)

This is the last page of the lessons, and the [Docs](../../manual/README.md) give the full
rules of the language, the compiler and the VM.

Next: [Zaka for programmers](../tour.md).
