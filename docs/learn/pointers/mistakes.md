---
title: Pointer mistakes
description: The mistakes pointers make possible, and how Zaka catches each one.
---

# Pointer mistakes

Pointers are powerful because each one reaches straight into RAM, and that same power
makes them simple to misuse. In a lot of other languages, a RAM mistake corrupts the
program without warning. Zaka catches these mistakes twice. The compiler finds each one
it can prove from the text of the program, and does not run the program. The Zaka
VM checks each read and write through a pointer while the program runs, stops the
program at the first mistake, and tells what went wrong.

Each section below shows both. With `--mem-check warn`, the compiler reports the
mistake as a warning and the program runs, so the VM's own check shows.

## Reading through a null pointer

```zaka-error ZK0709
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။     // never given an address
"အစ" ဟု ပြသပါ။
ပ ၏ တန်ဖိုး ကို ပြသပါ။
```

```text
error[ZK0709]: "ပ" is ဗလာ here, so it cannot be dereferenced
```

With `--mem-check warn`, or **Warn** in the Explorer, the program runs and the VM
stops it:

```zaka-stop mem-check=warn
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။     // never given an address
"အစ" ဟု ပြသပါ။
ပ ၏ တန်ဖိုး ကို ပြသပါ။
```

```text
အစ
the program stopped: access through a null address at 0x0
```

Give the pointer an address first, or test it against `ဗလာ`.

## Reading past the end

A block of 4 integers has boxes 0 to 3. Box 4 is outside it:

```zaka-error ZK0710
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ + ၃ ၏ တန်ဖိုး ကို ပြသပါ။     // the last box: fine
ပ + ၄ ၏ တန်ဖိုး ကို ပြသပါ။     // one past the end
```

```text
error[ZK0710]: offset 4 is past the end of a block of 4 elements
```

With `--mem-check warn`, or **Warn** in the Explorer, the program runs and the VM
stops it:

```zaka-stop mem-check=warn
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ + ၃ ၏ တန်ဖိုး ကို ပြသပါ။     // the last box: fine
ပ + ၄ ၏ တန်ဖိုး ကို ပြသပါ။     // one past the end
```

```text
၀
the program stopped: access of 8 octets at 0x101040 lands outside each live block
```

A new block starts out filled with zeros, so box 3 prints `၀`. Count from 0 and stop
one before the size. The compiler sees this mistake because the size and the box number
are both written in the program. When one of them comes from input, the VM's check
is the one that catches it.

## Reading past a variable

Each variable has a block of its own, with a gap after it, just like the heap blocks
above. A borrow of a variable points at a block of one box, so a step past it lands in
the gap:

```zaka-error ZK0710
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။     // points at က
ပ + ၁ ၏ တန်ဖိုး ကို ပြသပါ။                                      // one past က
```

```text
error[ZK0710]: offset 1 is past the end of a block of 1 elements
```

With `--mem-check warn`, or **Warn** in the Explorer, the program runs and the VM
stops it:

```zaka-stop mem-check=warn
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။     // points at က
ပ + ၁ ၏ တန်ဖိုး ကို ပြသပါ။                                      // one past က
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

Run a program with `zakac run --mem-check warn --frames packed`, or pick **Warn** for
**Ownership check** and **Packed, as C** for **Variables** in the Explorer's settings.
The program above then prints `၂`, the value of `ခ`, and goes on as if nothing were
wrong.

Packed is how C sets out variables, and how native Zaka programs will. There, a step
past a variable changes its neighbour with no warning, and the program goes wrong far
from the line at fault. Heap blocks keep their checks in both modes, and so does a step
past the whole block of a call. Keep the checked mode while learning; switch to packed
to see what the checks catch.

## Using a block after giving it back

`မိတ္တူ` borrows the block of `ပ`, and `ပ` gives the block back while `မိတ္တူ` still has
a read to do:

```zaka-error ZK0704
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ။
ကိန်းရှင် မိတ္တူ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
မိတ္တူ ၏ တန်ဖိုး ကို ပြသပါ။
```

```text
error[ZK0704]: the block of "ပ" is freed here, but "မိတ္တူ" still borrows it and is used later
```

The message points at the give-back, and at the later read of `မိတ္တူ` as well. With
`--mem-check warn`, or **Warn** in the Explorer, the program runs and the VM stops
it:

```zaka-stop mem-check=warn
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ။
ကိန်းရှင် မိတ္တူ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
မိတ္တူ ၏ တန်ဖိုး ကို ပြသပါ။
```

```text
the program stopped: access at 0x101040 inside a freed block
```

Swap the last two lines, so the read comes before the give-back, and the program prints
`၇`. Resizing the block while a borrow of it is still in use is the same mistake, since
the block can move.

Declared as a plain `ကိန်းပြည့် လိပ်စာ`, `မိတ္တူ` would take the block from `ပ` instead of
borrowing it. That program is correct: `ပ` holds `ဗလာ` after the move, so giving it back
does nothing, and `မိတ္တူ` owns the block and gives it back itself.

A function that returns a borrow of its own variable would leave its caller with the same
fault. The compiler refuses that function: see [how long an address
lasts](out-parameters.md#how-long-an-address-lasts).

## Giving a block back twice

```zaka-error ZK0702
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် မိတ္တူ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လိပ်စာ မိတ္တူ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
error[ZK0704]: the block of "ပ" is freed here, but "မိတ္တူ" still borrows it and is used later
error[ZK0702]: a borrowed address cannot be freed or resized; free its owner instead
```

The last line gives the block back through a borrow, which is never allowed: the owner
alone gives a block back (ZK0702). The line before it is wrong as well, because the last
line still uses `မိတ္တူ` (ZK0704). With `--mem-check warn`, or **Warn** in the Explorer,
the program runs and the VM stops it:

```zaka-stop mem-check=warn
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် မိတ္တူ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လိပ်စာ မိတ္တူ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
the program stopped: the block at 0x101040 was freed before
```

Giving back `ပ` a second time would be harmless, because the first time set it to
null.

## Giving back a block that was not asked for

Blocks from `နေရာယူ၍` can be given back; a variable's own storage cannot. A pointer at a
variable is a borrow, and a borrow never gives a block back:

```zaka-error ZK0702
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
error[ZK0702]: a borrowed address cannot be freed or resized; free its owner instead
```

With `--mem-check warn`, or **Warn** in the Explorer, the program runs and the VM
stops it:

```zaka-stop mem-check=warn
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
the program stopped: 0x1000 is not the start of a live heap block
```

An address inside a heap block, such as `ပ + ၂`, is a borrow too, so giving it back is
the same error.

## After giving a block to a function

A function whose parameter is a plain `ကိန်းပြည့် လိပ်စာ` takes the block. After the
call, the caller's variable holds `ဗလာ`:

```zaka-error ZK0701
လုပ်ဆောင်ချက် ရှင်းပစ် သည်
ကိန်းပြည့် လိပ်စာ က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လိပ်စာ က ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
လုပ်ဆောင်ချက် ရှင်းပစ် ကို ပ တို့ဖြင့် လုပ်ဆောင်ပါ။          // hands the block over
ပ ၏ တန်ဖိုး ကို ပြသပါ။
```

```text
error[ZK0701]: "ပ" was moved to another owner and now holds ဗလာ
```

With `--mem-check warn`, or **Warn** in the Explorer, the program runs and the VM
stops it at the read of `ဗလာ`:

```zaka-stop mem-check=warn
လုပ်ဆောင်ချက် ရှင်းပစ် သည်
ကိန်းပြည့် လိပ်စာ က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လိပ်စာ က ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
လုပ်ဆောင်ချက် ရှင်းပစ် ကို ပ တို့ဖြင့် လုပ်ဆောင်ပါ။          // hands the block over
ပ ၏ တန်ဖိုး ကို ပြသပါ။
```

```text
the program stopped: access through a null address at 0x0
```

In C, this is a use after free hidden inside another function. In Zaka the header tells:
`ရှင်းပစ်` takes the block. To keep the block, give the parameter a `ငှားထားသော` kind and
lend the block with `ပ ၏ တန်ဖိုးအငှား`. Then `ရှင်းပစ်` cannot give it back.

## Losing a block

A block whose last owner is overwritten, or ends with its function, can never be given
back:

```zaka-error ZK0708
လုပ်ဆောင်ချက် စမ်း သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
မင်မိုရီတွင် ကိန်းပြည့် ၈ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။  // the first block is lost
ပ ၏ တန်ဖိုး ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။                                     // the second is lost too

လုပ်ဆောင်ချက် စမ်း ကို လုပ်ဆောင်ပါ။
```

```text
error[ZK0708]: the block that "ပ" owns is never freed or moved on this path
error[ZK0708]: the block that "ပ" owns is never freed or moved on this path
```

The first message is for the second `နေရာယူ၍`, which overwrites the owner of the first
block. The second is for the end of `စမ်း`, where `ပ` still holds the second block. With
`--mem-check warn`, or **Warn** in the Explorer, the program runs to its end, since a
lost block does no harm at once:

```zaka mem-check=warn
လုပ်ဆောင်ချက် စမ်း သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
မင်မိုရီတွင် ကိန်းပြည့် ၈ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။  // the first block is lost
ပ ၏ တန်ဖိုး ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။                                     // the second is lost too

လုပ်ဆောင်ချက် စမ်း ကို လုပ်ဆောင်ပါ။
```

```text
၀
```

Called in a loop, though, such a function eats the heap a little at a time. The compiler
checks each overwrite of an owner and the end of each function. A block that a top-level
variable still holds when the program ends is for `zakac run --leaks`, which lists each
forgotten block. `samples/mistakes/leak.zaka` asks for a block three times and gives back
the last one alone:

```console
$ zakac run --mem-check warn --leaks samples/mistakes/leak.zaka
warning[ZK0708]: the block that "ပ" owns is never freed or moved on this path
 --> samples/mistakes/leak.zaka:4:1
  |
4 | မင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
၁၀
၂၀
၃၀
leak: block at 0x101040, 16 octets
leak: block at 0x101060, 16 octets
```

## Mistakes the compiler catches

Some mistakes are visible before the program runs, and the compiler reports them. The
ZK04 codes are about kinds; the ZK07 codes come from the check of [ownership and
borrowing](ownership.md):

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
| ZK0701 | using an owner after its block moved to another owner |
| ZK0702 | giving back or resizing a block through a borrow |
| ZK0703 | keeping a borrow in a heap block, behind a pointer, or in a top-level variable from a function |
| ZK0704 | giving back, resizing or moving a block while a borrow of it is still in use |
| ZK0705 | taking the address of a borrow |
| ZK0706 | returning a borrow of a function's own variable |
| ZK0707 | a call that can give back a block through one argument while another argument reaches the same block |
| ZK0708 | losing a block: its owner is overwritten, or its function ends, while it holds the block |
| ZK0709 | reading or writing through a pointer that is `ဗလာ` on each path to the read |
| ZK0710 | a box or cell number past the end of its block |
| ZK0711 | a warning, with `--mem-strict` alone: using a pointer that can be `ဗလာ` |
| ZK0712 | subtracting pointers into different blocks |
| ZK0713 | `ငှားထားသော` in the wrong place in a pointer kind |
| ZK0714 | a borrow given where an owner is wanted |
| ZK0715 | an owner given where a borrow is wanted |
| ZK0716 | lending a raw address with `တန်ဖိုးအငှား` |
| ZK0717 | a raw address kind under `--no-raw` |

```zaka-error ZK0413
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
က ၏ တန်ဖိုး ကို ပြသပါ။
```

```text
error[ZK0413]: isize is not a pointer and cannot be dereferenced
```

## Hands on

- Introduce a new fault into each program on this page, and read the message.
- Fix the "past the end" program so it prints all four boxes and nothing more.
- Run the "past a variable" program with `--mem-check warn --frames packed`. Then write
  `၉` through `ပ + ၁` and print `ခ`. Which variable changed?
- Set **Ownership check** to **Warn** in the Explorer's settings, and step through each
  program on this page to the line where the VM stops it.
- Write a function that asks for a block and returns its address. Is that a mistake?
  (No: the caller owns the block from then on.)

Next: [Lists](../lists/README.md).
