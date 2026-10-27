---
title: Memory mistakes
description: The mistakes pointers make possible, and how Zaka catches each one.
---

# Memory mistakes

Pointers are powerful because they reach memory directly, and that also makes them easy
to misuse. In many languages a memory mistake quietly corrupts the program. The Zaka
machine checks every read and write through a pointer and stops the program at the
first mistake, saying what went wrong.

## Reading through an empty pointer

```zaka-stop
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။     // never given an address
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
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ + ၃ ၏ တန်ဖိုး ကို ပြသပါ။     // the last box: fine
ပ + ၄ ၏ တန်ဖိုး ကို ပြသပါ။     // one past the end
```

```text
၀
the program stopped: access of 8 octets at 0x1020 lands outside each live block
```

A new block is filled with zeros, which is why box 3 prints `၀`. Count from 0 and stop
one before the size.

## Using memory after giving it back

Giving a block back empties the pointer you name, but not other copies of the address:

```zaka-stop
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် မိတ္တူ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ဖြစ်ပါစေ။      // a second copy of the address
ပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
မိတ္တူ ၏ တန်ဖိုး ကို ပြသပါ။
```

```text
the program stopped: access at 0x1000 inside a freed block
```

The same happens when a function returns the address of its own variable and the caller
reads through it. Zaka also [warns about that](out-parameters.md#how-long-an-address-lasts)
before the program runs.

## Giving memory back twice

```zaka-stop
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် မိတ္တူ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ဖြစ်ပါစေ။
ပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
မိတ္တူ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
the program stopped: the block at 0x1000 was freed before
```

Giving back `ပ` a second time would be harmless, because the first time emptied it. The
mistake is giving back the same block through another pointer.

## Giving back memory that was not asked for

Only blocks from `နေရာယူ၍` can be given back. A variable's own memory cannot:

```zaka-stop
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။
ပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
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
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
က ၏ တန်ဖိုး ကို ပြသပါ။
```

```text
error[ZK0413]: isize is not a pointer and cannot be dereferenced
```

## Try it

- Break each program on this page in a new way, and read the message.
- Fix the "past the end" program so it prints all four boxes and nothing more.
- Write a function that asks for a block and returns its address. Is that a mistake?
  (No: heap memory lasts until it is given back. Only a function's own variables end
  with it.)

Next: [Natural Zaka](../natural-zaka/README.md).
