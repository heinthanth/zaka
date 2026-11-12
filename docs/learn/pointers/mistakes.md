---
title: Pointer mistakes
description: The mistakes pointers make possible, the warnings the compiler gives for them, and how the VM stops each one.
---

# Pointer mistakes

Pointers are powerful because each one reaches straight into RAM, and that same power
makes them simple to misuse. In a lot of other languages, a RAM mistake corrupts the
program without warning. Zaka catches these mistakes twice. In unsafe code, the compiler
warns about each one it can prove from the text of the program. The Zaka VM checks each
read and write through a pointer while the program runs, stops the program at the first
mistake, and tells what went wrong.

Each program below runs with the default `--mem-check warn`: the compiler prints its
warning, the program runs, and the VM stops it. With `--mem-check error`, or **Error** in
the Explorer, the same warnings are errors and nothing runs:

```zaka-error ZK0709 mem-check=error
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။ // the null address
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0709]: "ပ" points at nothing here, so there is nothing to read or write through it
```

## Reading through a null pointer

The compiler warns with ZK0709, and the VM stops the read:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။ // the null address
"အစ" ဟု ပြသမည်။
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အစ
the program stopped: access through a null address at 0x0
```

Give the pointer an address first, or test it against `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`.

## Reading past the end

A block of 4 integers has boxes 0 to 3. Box 4 is outside it. The compiler warns with
ZK0710:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ + ၃ ၏ တန်ဖိုး ကို ပြသမည်။     // the last box: fine
ပ + ၄ ၏ တန်ဖိုး ကို ပြသမည်။     // one past the end
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၀
the program stopped: access of 8 octets at 0x101020 lands outside each live block
```

A new block starts out filled with zeros, so box 3 prints `၀`. Count from 0 and stop
one before the size. The compiler sees this mistake because the size and the box number
are both written in the program. When one of them comes from input, the VM's check
is the one that catches it.

## Reading past a variable

Each variable has a block of its own, with a gap after it, just like the heap blocks
above. The address of a variable points at a block of one box, so a step past it lands in
the gap:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။ // points at က
ပ + ၁ ၏ တန်ဖိုး ကို ပြသမည်။                                      // one past က
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: access of 8 octets at 0x1008 lands outside each live block
```

### Checked and packed variables

The Zaka VM can set out variables in two forms:

| Mode | Where the variables go | A step past a variable |
| --- | --- | --- |
| checked, the default | each in a block of its own, with a gap after it | stops the program |
| packed | the locals of a call next to each other in one block, as C does, and the globals in one more | reads or changes the next variable |

Run a program with `zakac run --frames packed`, or pick **Packed, as C** for **Variables**
in the Explorer's settings. The program above then prints `၂`, the value of `ခ`, and goes
on as if nothing were wrong.

Packed is how C sets out variables, and how native Zaka programs will. There, a step
past a variable changes its neighbour with no warning, and the program goes wrong far
from the line at fault. Heap blocks keep their checks in both modes, and so does a step
past the whole block of a call. Keep the checked mode while learning; switch to packed
to see what the checks catch.

## Using a block after giving it back

`မ` holds a duplicate of the address in `ပ`. Giving the block back through `ပ` sets `ပ` to
the null address, but `မ` still holds the old address. The compiler warns with ZK0704, at the
give-back and at the later use:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားမည်။
ကိန်းရှင် မ သည် ပ ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
မ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: access at 0x101000 inside a freed block
```

Swap the last two lines of the unsafe block, so the read comes before the give-back, and
the program prints `၇`. Resizing the block while a duplicate of its address is still in use is
the same mistake, since the block can move.

## Giving a block back twice

Giving the block back through `မ` as well gives it back a second time. The compiler warns
with ZK0704:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် မ သည် ပ ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လိပ်စာ မ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: the block at 0x101000 was freed before
```

Giving back `ပ` a second time would be harmless, because the first time set it to
null. Keep each block in one variable that gives it back, and hand it to another function
with [`အပိုင်အနေဖြင့်`](ownership.md#owned-results-and-parameters).

## Giving back a block that was not asked for

Blocks from `နေရာယူ၍` can be given back; a variable's own storage cannot. The compiler
warns with ZK0702:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: 0x1000 is not the start of a live heap block
```

An address inside a heap block, such as `ပ + ၂`, is not the start of a block either, so
giving it back stops the program too.

## Giving back the address of a local

The variables of a function end with the call: when it returns, their RAM is given back.
A function that gives back the address of one of them leaves its caller a pointer into
storage that is gone. The compiler warns with ZK0706:

```zaka-stop
စိတ်မချရသော လုပ်ဆောင်ချက် မှား သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် လိပ်စာ တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် x သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
x ၏ လိပ်စာ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် မှား ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: access at 0x1020 inside a freed block
```

Pointing *into* a function at the caller's variables is fine: the caller's variables
outlive the call. Room that must outlive the function that makes it is a block from the
[heap](heap.md), handed to the caller as an [owned
result](ownership.md#owned-results-and-parameters).

## Losing a block

A block whose last pointer is overwritten, or ends with its function, can never be given
back. The compiler warns with ZK0708, and the program runs to its end, since a lost block
does no harm at once:

```zaka
စိတ်မချရသော လုပ်ဆောင်ချက် စမ်း သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
မင်မိုရီတွင် ကိန်းပြည့် ၈ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။  // the first block is lost
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။                                     // the second is lost too

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
စမ်း ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၀
```

Called in a loop, though, such a function eats the heap a little at a time. The compiler
checks each overwrite of a pointer that holds a block it made, and the end of each
function. For the blocks that remain when the program ends, run it with
`zakac run --leaks`, which lists each forgotten block. `samples/mistakes/leak.zaka` asks
for a block three times and gives back the last one alone:

```console
$ zakac run --leaks samples/mistakes/leak.zaka
warning[ZK0708]: the block that "ပ" holds is never given back or handed over on this path
 --> samples/mistakes/leak.zaka:6:1
  |
6 | မင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
၁၀
၂၀
၃၀
leak: block at 0x101000, 16 octets
leak: block at 0x101020, 16 octets
```

Text, lists, shared objects and closures never show here: the compiler frees them.

## Mistakes the compiler catches

The ZK04 codes are about kinds and unsafe code, and are errors in each mode. The ZK07 codes come
from the pointer check: in unsafe code each follows `--mem-check`, and in safe code each
is an error.

| Code | Mistake |
| --- | --- |
| ZK0410 | assigning to something that is not a variable or a value through a pointer |
| ZK0411 | taking the address of something that has none, such as `၅` |
| ZK0412 | taking the address of a constant or a counted-loop variable |
| ZK0413 | reading through a value that is not a pointer |
| ZK0414 | reading through, or adding to, an opaque `လိပ်စာ` |
| ZK0415 | using an operator such as `*` or `/` on a pointer |
| ZK0416 | comparing or subtracting pointers of different kinds |
| ZK0417 | a heap statement given something other than a pointer |
| ZK0468 | pointer work, or a call of an unsafe function, outside unsafe code |
| ZK0469 | an unsafe function as a function value |
| ZK0470 | `အပိုင်အနေဖြင့်` on a kind that is not a pointer |
| ZK0471 | unsafe code under `--no-unsafe` |
| ZK0472 | a C function with a pointer, not declared unsafe |
| ZK0701 | reading a variable after it handed its block to an `အပိုင်အနေဖြင့်` parameter: an error in each mode |
| ZK0702 | giving back or resizing the address of a variable |
| ZK0703 | keeping the address of a variable in a heap block, behind a pointer, or in a top-level variable from a function |
| ZK0704 | using, giving back or resizing a block after it was given back, resized or moved |
| ZK0706 | giving back the address of a function's own variable |
| ZK0707 | a call that hands a block over through one argument while another argument reaches the same block |
| ZK0708 | losing a block: its pointer is overwritten, or its function ends, while it holds the block |
| ZK0709 | reading or writing through a pointer that holds the null address on each path to the read |
| ZK0710 | a box or cell number past the end of its block |
| ZK0711 | a warning, with `--mem-strict` alone: using a pointer that can hold the null address |
| ZK0712 | subtracting pointers into different blocks |
| ZK0721 | a `ကူးယူမည်` whose source and destination share cells |

```zaka-error ZK0413
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
က ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0413]: isize is not a pointer and cannot be dereferenced
```

## Hands on

- Introduce a new fault into each program on this page, and read the message.
- Fix the "past the end" program so it prints all four boxes and nothing more.
- Run the "past a variable" program with `--frames packed`. Then write `၉` through
  `ပ + ၁` and print `ခ`. Which variable changed?
- Step through each program on this page in the Explorer to the line where the VM stops it.
- Write an unsafe function that asks for a block and returns its address. What must its
  result header hold? (`အပိုင်အနေဖြင့်`: the caller owns the block from then on.)

This is the last page of Learn. The [Docs](../../manual/README.md) give the full rules of
the language, the compiler and the VM.
