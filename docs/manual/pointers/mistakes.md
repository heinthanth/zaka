---
title: Pointer mistakes
description: The mistakes pointers make possible, and how Zaka catches each one.
---

# Pointer mistakes

Pointers are powerful because each one reaches straight into RAM, and that same power
makes them simple to misuse. In a lot of other languages, a RAM mistake corrupts the
program without warning. The Zaka machine checks each read and write through a pointer,
stops the program at the first mistake, and tells what went wrong.

## Reading through a null pointer

```zaka-stop
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

```zaka-stop
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ + ၃ ၏ တန်ဖိုး ကို ပြသပါ။     // the last box: fine
ပ + ၄ ၏ တန်ဖိုး ကို ပြသပါ။     // one past the end
```

```text
၀
the program stopped: access of 8 octets at 0x101040 lands outside each live block
```

A new block starts out filled with zeros, so box 3 prints `၀`. Count from 0 and stop
one before the size.

## Reading past a variable

Each variable has a block of its own, with a gap after it, just like the heap blocks
above. A step past a variable lands in the gap and stops the program:

```zaka-stop
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။     // points at က
ပ + ၁ ၏ တန်ဖိုး ကို ပြသပါ။                                      // one past က
```

```text
the program stopped: access of 8 octets at 0x1008 lands outside each live block
```

### Checked and packed variables

The Zaka machine can set out variables in two forms:

| Mode | Where the variables go | A step past a variable |
| --- | --- | --- |
| checked, the default | each in a block of its own, with a gap after it | stops the program |
| packed | the locals of a call next to each other in one block, as C does, and the globals in one more | reads or changes the next variable |

Run a program with `zakac run --frames packed`, or pick **Packed, as C** for
**Variables** in the Explorer's settings. The program above then prints `၂`, the value
of `ခ`, and goes on as if nothing were wrong.

Packed is how C sets out variables, and how native Zaka programs will. There, a step
past a variable changes its neighbour with no warning, and the program goes wrong far
from the line at fault. Heap blocks keep their checks in both modes, and so does a step
past the whole block of a call. Keep the checked mode while learning; switch to packed
to see what the checks catch.

## Using a block after giving it back

Giving a block back clears the named pointer, but not other pointers that hold the same
address:

```zaka-stop
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် မိတ္တူ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ဖြစ်ပါစေ။      // a duplicate of the address
ပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
မိတ္တူ ၏ တန်ဖိုး ကို ပြသပါ။
```

```text
the program stopped: access at 0x101040 inside a freed block
```

The same happens when a function returns the address of its own variable and the caller
reads through it. Zaka also [warns about that](out-parameters.md#how-long-an-address-lasts)
before the program runs.

## Giving a block back twice

```zaka-stop
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် မိတ္တူ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ဖြစ်ပါစေ။
ပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
မိတ္တူ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
the program stopped: the block at 0x101040 was freed before
```

Giving back `ပ` a second time would be harmless, because the first time set it to
null. The mistake is giving back the same block through another pointer.

## Giving back a block that was not asked for

Blocks from `နေရာယူ၍` can be given back; a variable's own storage cannot:

```zaka-stop
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။
ပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
the program stopped: 0x1000 is not the start of a live heap block
```

## Mistakes the compiler catches

Some mistakes are visible before the program runs, and the compiler reports them:

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
| ZK0418 | (warning) returning the address of a function's own variable |

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
- Run the "past a variable" program with `--frames packed`. Then write `၉` through
  `ပ + ၁` and print `ခ`. Which variable changed?
- Write a function that asks for a block and returns its address. Is that a mistake?
  (No: a heap block lasts until it is given back. It is the function's own variables that
  end with it.)

Next: [Lists](../lists/README.md).
