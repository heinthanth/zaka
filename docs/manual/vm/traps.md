---
title: When a program stops
description: Each reason the VM stops a program, what causes it, and the message it prints.
---

# When a program stops

A *trap* is the VM stopping a run because the next step would go wrong. Nothing
after it runs, and all that was printed before it remains printed. A trap is not a compiler
diagnostic: it has no `ZK` code, because it belongs to one run of a program and not to
the program text.

The Run buttons, the Explorer and `zakac run` show a trap in a shorter or a longer
form:

```text
the program stopped: access through a null address at 0x0
```

```console
zaka_vm: the program stopped: access through a null address at 0x0 (in main, op 5)
```

The command line adds the function and the index of the instruction. When the trap names
an address, it also adds the block that holds the address or ends just before it:
`block at 0x101040, 16 octets, heap`, or the variable the block belongs to.

Numerous traps have a twin in the compiler. The [pointer checks](../compiler/checks.md)
report a mistake that can be proved from the text, as an error; the VM reports the same
mistake when it happens in a run, which covers the paths and the numbers the compiler
cannot see. The examples below ask the compiler for warnings with `--mem-check warn`
(the Explorer calls it **Warn**) so that the program runs and the VM's own check
shows. Without it, the compiler would refuse numerous programs of this page.

## All the traps

| Trap | Raised when | Message |
| --- | --- | --- |
| zero divisor | an integer division or remainder has the divisor zero | `integer division or remainder with zero as divisor` |
| shift too far | a shift distance is not below the bit width | `shift distance D is not below the bit width B` |
| step not positive | a counted loop step is zero or negative | `counted loop step is not positive` |
| index out of range | a cell number is outside its list | `cell N is outside the list, whose cells run from A to B` |
| null address | an access below the first block | `access through a null address at A` |
| out of bounds | an access lands in no block, or runs past the end of one | `access of N octets at A lands outside each live block` |
| step out of bounds | a pointer step leaves the block it started in | `a pointer step to A leaves the block it started in` |
| misaligned | an address is not a multiple of the alignment of its kind | `address A is not a multiple of N` |
| bad value | the octets read hold no value of the kind | `the octets at A do not form a K` |
| use after free | an access lands in a freed block | `access at A inside a freed block` |
| double free | a heap block is freed after it was freed | `the block at A was freed before` |
| bad free | a free of an address that starts no live heap block | `A is not the start of a live heap block` |
| mixed blocks | a subtraction of pointers into different blocks | `pointers into different blocks cannot be subtracted` |
| out of mem | a resize finds no heap room | `no heap room for N octets` |
| stack full | a call finds no room in the stack region | `no stack room for N more octets` |
| static full | the space has no room for a global | `no room in the address space for a global of N octets` |
| stack overflow | calls nest deeper than `--max-depth` | `more than N nested calls` |
| step limit | more instructions ran than `--max-steps` | `stopped after N ops` |
| malformed | the instructions break a rule the code generator keeps | `malformed ops: R` |
| write | the output cannot be written | `cannot write the output: E` |

Addresses are written `0x` and lowercase hex digits. Integer arithmetic has no overflow
trap: it wraps to the width of its kind (`255 + 1` is `0` for an 8-bit unsigned integer),
and a constant that does not fit is a compile error ([ZK0601](../reference/errors.md)).

## Arithmetic and loops

### Zero as divisor

A division or remainder with a zero the compiler can see is an error (ZK0602). A zero that
first shows up in the run is a trap:

```zaka-stop
ကိန်းရှင် ခ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
"အစ" ဟု ပြသပါ။
၁၀ / ခ ကို ပြသပါ။
```

```text
အစ
the program stopped: integer division or remainder with zero as divisor
```

### A shift too far

A shift moves the bits fewer places than the integer has. A distance held in a variable
is checked when the shift runs:

```zaka-stop
ကိန်းရှင် က သည် လက္ခဏာမဲ့ ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် လက္ခဏာမဲ့ ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၇၀ ဖြစ်ပါစေ။
"အစ" ဟု ပြသပါ။
က ကို ဘယ်သို့ ခ ဘစ် ရွေ့သောရလဒ် ကို ပြသပါ။
```

```text
အစ
the program stopped: shift distance 70 is not below the bit width 64
```

### A counted loop that cannot count

The step of a counted loop must be positive, since the direction comes from the two
ends. A step written as a number is checked in the compiler; a step held in a variable
is checked once, before the first round:

```zaka-stop
ကိန်းရှင် ခြေလှမ်း သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
"အစ" ဟု ပြသပါ။
၁ မှ ၅ အထိ ခြေလှမ်း စီ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသပါ။
```

```text
အစ
the program stopped: counted loop step is not positive
```

### A cell outside the list

A list of 3 cells has cells 0, 1 and 2. A cell number the compiler cannot work out
is checked when the cell is read. The message gives the range the list allows:

```zaka-stop mem-check=warn
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅၊ ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် န သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက် န ကို ပြသပါ။
```

```text
the program stopped: cell 3 is outside the list, whose cells run from 0 to 2
```

## Addresses

### A null address

An access below the first block is a null access. That covers `ဗလာ` itself and each
small address near it. A step off `ဗလာ` is the same trap, since no offset turns null
into the address of a block:

```zaka-stop mem-check=warn
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။     // never given an address
"အစ" ဟု ပြသပါ။
ပ ၏ တန်ဖိုး ကို ပြသပါ။
```

```text
အစ
the program stopped: access through a null address at 0x0
```

### Outside each block

An access must lie inside one live block. An address in the gap after a block, or in no
block at all, such as a number turned into a raw address, is out of bounds:

```zaka-stop mem-check=warn
ကိန်းရှင် ရ သည် ကိန်းပြည့် လိပ်စာအကြမ်း ဖြစ်၍ တန်ဖိုးသည် ၁၂၃၄၅၆ ကို ကိန်းပြည့် လိပ်စာအကြမ်း အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
ရ ၏ တန်ဖိုး ကို ပြသပါ။
```

```text
the program stopped: access of 8 octets at 0x1e240 lands outside each live block
```

An access that starts inside a block and runs past its end is the same trap. Reading
box 4 of a block of 4 integers starts at the end of the block, in the gap:

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

### A step out of the block

Forming an address reads nothing, so it is allowed to reach one past the end of a block.
A step that lands farther, or before the start, traps at once. The step reads nothing,
so its message names no octet count:

```zaka-stop mem-check=warn
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် ဝေး သည် ကိန်းပြည့် လိပ်စာအကြမ်း ဖြစ်ပါစေ။
ဝေး ကို ပ ကို ကိန်းပြည့် လိပ်စာအကြမ်း အဖြစ် ပြောင်းလဲသော ရလဒ် ဟု ထားပါ။
ကိန်းရှင် နေရာ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၀၀၀ ဖြစ်ပါစေ။
ဝေး ကို ဝေး + နေရာ ဟု ထားပါ။
```

```text
the program stopped: a pointer step to 0x102fa0 leaves the block it started in
```

### A misaligned address

An integer of 8 octets must sit at a multiple of 8, a character at a multiple of 4. A
raw address made from a block of octets plus one is not:

```zaka-stop mem-check=warn
ကိန်းရှင် ဘိုက် သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ၁၆ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို ဘိုက် တွင် သိမ်းထားမည်။
ကိန်းရှင် ရ သည် ကိန်းပြည့် လိပ်စာအကြမ်း ဖြစ်ပါစေ။
ရ ကို ဘိုက် + ၁ ကို ကိန်းပြည့် လိပ်စာအကြမ်း အဖြစ် ပြောင်းလဲသော ရလဒ် ဟု ထားပါ။
ရ ၏ တန်ဖိုး ကို ပြသပါ။
```

```text
the program stopped: address 0x101041 is not a multiple of 8
```

### Octets that are no value

Not each pattern of octets is a value. A boolean octet holds `0` or `1`, and a character
holds a Unicode scalar value. Reading other octets as one of these kinds traps:

```zaka-stop mem-check=warn
ကိန်းရှင် ဘိုက် သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ၁၆ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို ဘိုက် တွင် သိမ်းထားမည်။
ဘိုက် ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ။
ကိန်းရှင် အမှန် သည် မှန်မှားတန်ဖိုး လိပ်စာအကြမ်း ဖြစ်ပါစေ။
အမှန် ကို ဘိုက် ကို မှန်မှားတန်ဖိုး လိပ်စာအကြမ်း အဖြစ် ပြောင်းလဲသော ရလဒ် ဟု ထားပါ။
အမှန် ၏ တန်ဖိုး ကို ပြသပါ။
```

```text
the program stopped: the octets at 0x101040 do not form a bool
```

### Using a block after it was freed

A freed block remains in the table, so an access through a stale address finds it. The
heap block of a pointer that was given back, and the block of a local whose call has
returned, both trap in this manner:

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

A function that hands back the address of its own variable leaves the caller with the
same fault. The compiler refuses such a function (ZK0706); with `--mem-check warn` the
VM shows the fault, and its long form names the variable:

```zaka-stop mem-check=warn
လုပ်ဆောင်ချက် ဒေသ သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ငှားထားသော ကိန်းပြည့် လိပ်စာ ရလဒ် ကို ပြန်ပေးမည်။
ကိန်းရှင် ပမာဏ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
ရလဒ် ကို ပမာဏ ၏ လိပ်စာ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် ပ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် ဒေသ ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ပ ၏ တန်ဖိုး ကို ပြသပါ။
```

```text
the program stopped: access at 0x1040 inside a freed block
```

```console
$ zakac run --mem-check warn program.zaka
zaka_vm: the program stopped: access at 0x1040 inside a freed block (in main, op 5); block at 0x1040, 8 octets, ပမာဏ of ဒေသ
```

### Giving a block back twice

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

Freeing `ဗလာ` is not a trap: it does nothing. That is the reason freeing the same owner twice
is harmless, since the first free left `ဗလာ` in it.

### Giving back what is not a heap block

The start of a live heap block is the one address that can be given back. The block of a variable, an
address inside a block, and a wild address are not:

```zaka-stop mem-check=warn
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
the program stopped: 0x1000 is not the start of a live heap block
```

### Pointers into two blocks

The distance between two pointers means something just inside one block. The VM
checks that a live block reaches both addresses, its end included:

```zaka-stop mem-check=warn
ကိန်းရှင် က သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို က တွင် သိမ်းထားမည်။
မင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ခ တွင် သိမ်းထားမည်။
က နှင့် ခ ၏ နုတ်လဒ် ကို ပြသပါ။
```

```text
the program stopped: pointers into different blocks cannot be subtracted
```

## Room

### A resize with no room

An allocate statement that finds no room leaves `ဗလာ`. A resize cannot do that, since it
would lose the old block, so it traps. The size below is far over the heap cap:

```zaka-stop
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် အရေအတွက် သည် လက္ခဏာမဲ့ ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၀၀၀၀၀၀၀၀၀ ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ကိန်းပြည့် အရေအတွက် ခုစာ ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။
```

```text
the program stopped: no heap room for 8000000000 octets
```

### Locals too big for the stack

The locals of a call live in the stack region, 1 MiB at most, with the default limit. A list of
200000 integers is 1600000 octets:

```zaka-stop
လုပ်ဆောင်ချက် ကြီးမား သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် အကွက် ၂၀၀၀၀၀ ရှိသော ကိန်းပြည့် စာရင်း ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

"အစ" ဟု ပြသပါ။
လုပ်ဆောင်ချက် ကြီးမား ကို လုပ်ဆောင်ပါ။
```

```text
အစ
the program stopped: no stack room for 1600000 more octets
```

### Calls that never return

A function that calls itself with no end fills the stack region, or reaches the depth
limit first. The limit is 4096 calls waiting to return:

```zaka-stop
လုပ်ဆောင်ချက် အဆုံးမဲ့ သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် အဆုံးမဲ့ ကို န + ၁ တို့ဖြင့် လုပ်ဆောင်ပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

လုပ်ဆောင်ချက် အဆုံးမဲ့ ကို ၁ တို့ဖြင့် လုပ်ဆောင်ပါ။
```

```text
the program stopped: more than 4096 nested calls
```

### Traps that need the command line

Three traps come from a limit that the command line alone sets, or from a situation that
a program cannot make on its own.

**Step limit.** `--max-steps` bounds the instructions a run takes. It stops
a loop that never ends:

```console
$ zakac run --max-steps 50 samples/mistakes/endless_loop.zaka
zaka_vm: the program stopped: stopped after 50 ops (in main, op 13)
```

**Static full.** On a platform with few addresses, the globals can fill the space before
`main` starts:

```console
$ zakac run --platform-bits 8 nine-globals.zaka
zaka_vm: the program stopped: no room in the address space for a global of 1 octets (in main, op 0)
```

**Malformed** and **write**. A module read from a damaged artifact can hold instructions
that break the rules of the code generator, such as a slot that does not exist. The
VM traps with `malformed ops: ...` and does not crash. A `write` trap happens when
the output cannot be written, such as a pipe that has been closed.

## What to do with a trap

The message tells what went wrong and where. A good order of work:

1. Read the reason and the block. `use after free` and `double free` name a block that
   some owner gave back; `out of bounds` names the block the access ran past.
2. Look at the instruction index with `zakac codegen --asm`, which lists the operations of
   each function with their index and the name of the variable each touches. See
   [Instructions](instructions.md).
3. Check the compile-time diagnostics. `--mem-check error`, the default, would have
   refused numerous programs of this page. The codes are in [Error messages](../reference/errors.md).
