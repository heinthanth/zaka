---
title: When a program stops
description: Each reason the VM stops a program (trap), the message it prints, its common cause and how to fix it.
---

# When a program stops

When the next step of a run would go wrong, the VM (virtual machine) stops the program
(trap). Nothing after that step runs, and the output printed before it is kept.
A stop is not a compiler diagnostic, so it has no `ZK` code: it belongs to one run of a
program, not to the program text. This page lists each reason in one table, with
the message and the common cause, and then explains each group with an example and the
fix. The stops of unsafe code, at an address (pointer) or in a call of C, are explained
on [Stops in unsafe code](traps-unsafe.md).

A lot of these stops have a twin in the compiler. The [compiler
checks](../compiler/checks.md) report a mistake that can be proved from the text, while the
VM reports the same mistake when it happens in a run, which covers the paths and the
numbers the compiler cannot see.

## Reading the message

The Run buttons and the Explorer show the short form, and `zakac run` shows the long one:

```text
the program stopped: access through a null address at 0x0
```

```console
zaka_vm: the program stopped: access through a null address at 0x0 (in ပရိုဂရမ်, at nul2.zaka:4:1)
```

| Part of the long form | Meaning |
| --- | --- |
| `zaka_vm:` | the VM wrote it; native code writes the name of the program here |
| `the program stopped: ...` | the reason, as in the table below |
| `(in ပရိုဂရမ်, at nul2.zaka:4:1)` | the function and the place of the operation in the source, `file:line:column`; a build made with `--release` has no lines, so this reads `op N`, the index of the operation |
| `; block at 0x101040, 16 octets, heap` | when the reason names an address, the block (allocation) that contains it or ends just before it, with its origin: `heap`, `static text t0`, or the variable the block belongs to |

When more than one call is in progress, the lines after the reason list the calls, most
recent first; [How a run ends](running.md#how-a-run-ends) shows the list and the rules
that keep it short.

A [thread](../language/threads.md) that stops under the default `--trap unwind` writes
its own line, and the rest of the program goes on:

```console
$ zakac run program.zaka
အစ
program.zaka: thread 1 stopped: integer division or remainder with zero as divisor (in စား, at program.zaka:3:1)
program.zaka: the program stopped: the thread 1 this waits for stopped at a trap (in ပရိုဂရမ်, at program.zaka:9:1)
```

The line reads `<program>: thread <n> stopped: <reason>` from native code, and
`zaka_vm: thread <n> stopped: <reason>` from the VM.

## All the traps

Addresses in messages are written as `0x` and lowercase hex digits. Each trap name links
to the section that explains it.

| Trap | Message | Common cause |
| --- | --- | --- |
| [zero divisor](#arithmetic-and-loops) | `integer division or remainder with zero as divisor` | a division or remainder where the divisor is a variable that is zero |
| [shift too far](#arithmetic-and-loops) | `shift distance D is not below the bit width B` | a shift distance from a variable that is at least the bit width |
| [overflow](#arithmetic-and-loops) | `the result of OP does not fit K` | an integer result outside its data kind, with the default `--overflow stop` |
| [step not positive](#arithmetic-and-loops) | `counted loop step is not positive` | a counted loop whose step variable is zero or negative |
| [index out of range](#lists-and-text) | `cell N is outside the list, whose cells run from A to B` | a cell number (index) past the end of a list, or below its first cell |
| [no cell to remove](#lists-and-text) | `the list holds no cells, so it has no last cell to remove` | a remove of the last cell from a list with no cells |
| [no character to remove](#lists-and-text) | `the text holds no characters, so it has no last character to remove` | a remove of the last character from text with no characters |
| [no place to insert](#lists-and-text) | `no value can go in front of N: the places run from A to B, and B is the end` | an add in front of a cell or character past the count |
| [end before start](#lists-and-text) | `the range from A to B ends before it starts` | a replace whose last cell or character comes before its first |
| [slice out of range](#lists-and-text) | `N cells from cell S pass the end of C cells` | a [range](../language/lists.md#ranges) that runs past the end of its list or heap block |
| [character out of range](#lists-and-text) | `character N is outside the text, whose characters run from L to H` | a character number past the end of the text |
| [text cut](#lists-and-text) | `a range of text at A starts or ends inside a character` | a range of text whose edge falls inside the u8s of one character |
| [not UTF-8](#lists-and-text) | `the u8s at A are not UTF-8 text` | a list of u8s [turned into text](../language/text.md#text-from-u8s) that is not UTF-8 |
| [null address](traps-unsafe.md#addresses) | `access through a null address at A` | a read or write through the null address, or a small offset from it |
| [out of bounds](traps-unsafe.md#addresses) | `access of N octets at A lands outside each live block` | a read or write past the end of a block, or at an address made from a number |
| [step out of bounds](traps-unsafe.md#addresses) | `a pointer step to A leaves the block it started in` | address arithmetic that leaves its block |
| [misaligned](traps-unsafe.md#addresses) | `address A is not a multiple of N` | an address cast to a data kind of larger alignment |
| [bad value](traps-unsafe.md#addresses) | `the octets at A do not form a K` | octets read as a true/false value or a character, which the octets do not form |
| [use after free](traps-unsafe.md#addresses) | `access at A inside a freed block` | an address used after its heap block was freed, or after its function returned |
| [double free](traps-unsafe.md#addresses) | `the block at A was freed before` | a second free of the same heap block |
| [bad free](traps-unsafe.md#addresses) | `A is not the start of a live heap block` | a free of the address of a variable, of the middle of a block, or of a wild address |
| [mixed blocks](traps-unsafe.md#addresses) | `pointers into different blocks cannot be subtracted` | a subtraction of addresses from two blocks |
| [overlap](traps-unsafe.md#addresses) | `the cells at A overlap their source; "ပြောင်းထည့်မည်" allows that` | a `ကူးယူမည်` whose source and destination share octets |
| [frozen write](traps-unsafe.md#addresses) | `write at A inside static data, which cannot change` | a write through an address into a static text or static list |
| [null call](traps-unsafe.md#addresses) | `the function value holds no function to call` | a call through a [function value](../language/function-values.md) read from zeroed octets |
| [null object](traps-unsafe.md#addresses) | `the object variable holds no object, so it has no field to read or write` | a field read through an [object](../language/objects.md) variable read from zeroed octets |
| [out of mem](#room-and-limits) | `no heap room for N octets` | a resize, an added cell, text built at run time, or the arguments of the [program start block](../language/functions-start-block.md) with no heap room left |
| [stack full](#room-and-limits) | `no stack room for N more octets` | locals too large for the stack region, such as a counted list of 200000 cells |
| [stack overflow](#room-and-limits) | `more than N nested calls` | a recursion with no end, or one deeper than `--max-depth` |
| [step limit](#room-and-limits) | `stopped after N ops` | a run longer than `--max-steps`, such as a loop that never ends |
| [static full](#room-and-limits) | `no room in the address space for a global of N octets` | more globals than fit with a small `--target-bits` |
| [not good](#a-whole-result-read-as-the-wrong-part) | `the result holds an error, not a good result: V`, or `the result holds a failure, not a good result` | the good result read from a [whole result](../language/reading-results.md#reads) that holds an error value or a failure |
| [not fault](#a-whole-result-read-as-the-wrong-part) | `the result is a good result, not an error` | the error value read from a whole result that holds a good result |
| [no data](#an-optional-read-with-no-data) | `the optional holds no data` | the data read from an [optional](../language/optionals.md) that is blank |
| [no row](#a-write-through-a-missing-row) | `no row of the ဇယား has this index` | a write inside the value of a [row of a table](../language/tables.md#writing-inside-a-row) that does not exist |
| [wrong member](#a-read-of-the-data-of-another-member) | `the value is another member than the one read` | the data of a [member](../language/enums.md#reading-the-data) read from a value of an enumeration that is another member |
| [thread trapped](#threads) | `the thread N this waits for stopped at a trap` | a wait for a thread that stopped, under `--trap unwind` |
| [thread waited](#threads) | `the thread N this waits for gave its result to an earlier wait` | a second wait for the same thread |
| [lock broken](#threads) | `the lock of this object is broken: the thread that held it stopped at a trap` | a lock block on an object whose holder stopped inside its lock |
| [threads not positive](#threads) | `the count of threads of a parallel loop or map is not positive` | a parallel loop or map split into zero or fewer threads |
| [pipe closed](#threads) | `the channel N is closed: no value can go to it, and it cannot close again` | a send to a closed [channel](../language/threads.md#channels), or a second close |
| [deadlock](#threads) | `each thread waits; none can go on` | a receive from a channel that no thread sends to or closes, or threads that wait for each other |
| [null from C](traps-unsafe.md#c-functions) | `C gave NULL where its declaration promised a value; declare the result or the parameter as an optional, "T ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်", when C can give NULL` | C returned NULL for a plain handle, or passed NULL for a plain text or handle parameter |
| [C function not run](traps-unsafe.md#c-functions) | `the C function N runs in "zakac run" and in native builds, not in this VM` | a C call in the Explorer, whose VM has no `ffi` |
| [C link](traps-unsafe.md#c-functions) | `the C functions of the program cannot be reached: R` | a lib that does not open, a C name that no lib has, or a `--target-bits` that is not the host's |
| [closed handle](traps-unsafe.md#c-functions) | `a C function that took the handle over closed it, so it cannot be used again` | a handle used after a C function took it over |
| [closure escape](traps-unsafe.md#c-functions) | `C called a closure after the call that gave it ended, and C has no leave to call it; give one with "ခေါ်ဆိုခွင့် ပြုမည်"` | C kept a closure and called it later, with no leave |
| [permission denied](#rights-damaged-programs-and-output) | `calling the C function N needs --allow-ffi`, `the environment of the start block needs --allow-env` | a C call with no ffi right, or a program start block that reads the environment with no env right |
| [malformed](#rights-damaged-programs-and-output) | `malformed ops: R` | a damaged artifact, or a value passed to C that does not suit its parameter |
| [write](#rights-damaged-programs-and-output) | `cannot write the output: E` | standard output closed, such as a pipe whose reader has ended |

## Finding the cause

The message names what went wrong and where. A good order of work:

1. Read the reason and the block. `use after free` and `double free` name a block that a
   free statement freed, and `out of bounds` names the block the access ran past.
2. Look at the operation (instruction) at the place the message names. `zakac build
   --emit asm` lists the operations of each function with their index and the name of
   the variable each one reads; see [Instructions](instructions.md).
3. Check the diagnostics of the compiler. The warnings of the checker on unsafe code point
   at a lot of the stops on this page, and `--lint-error mem` turns them into errors. The
   codes are in [Error messages](../reference/errors.md).

## Arithmetic and loops

### Zero divisor

A division or remainder with a zero divisor that the compiler can see is an error (ZK0602), while a
zero that first shows up in the run stops the program. To fix it, we test the divisor
before we divide:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
"အစ" ဟု ပြသမည်။
၁၀ / ခ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အစ
the program stopped: integer division or remainder with zero as divisor
```

### Shift too far

A shift moves the bits fewer places than the integer has. The VM checks a distance from a
variable when the shift runs:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀ ဖြစ်ပါစေ။
"အစ" ဟု ပြသမည်။
က ကို ဘယ်သို့ ခ ဘစ် ရွေ့သောရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အစ
the program stopped: shift distance 70 is not below the bit width 64
```

### Overflow

Integer arithmetic whose result does not fit its data kind (type) stops the program. With
`--overflow wrap`, the result wraps to the width of its data kind instead (`255 + 1` is
`0` for an 8-bit unsigned integer), and a constant that does not fit is a compile error in
both modes ([ZK0601](../reference/errors.md)). Here a counter of 8 bits passes 255:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၅၀ ဖြစ်ပါစေ။
၁ မှ ၁၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ က ကို က + ၁ ဟု ထားမည်။
က ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: the result of + does not fit u8
```

To fix it, we pick a wider data kind, or compile with `--overflow wrap` when wrapping is
what we want (the program above then prints `၄`).

### Step not positive

The step of a counted loop must be positive, since the direction comes from the two ends.
The compiler checks a step written as a number, and the VM checks a step from a variable
once, before the first round:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခြေလှမ်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
"အစ" ဟု ပြသမည်။
၁ မှ ၅ အထိ ခြေလှမ်း စီ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အစ
the program stopped: counted loop step is not positive
```

## Lists and text

### Index out of range

A list (dynamic array) of 3 cells has cells 0, 1 and 2. The VM checks each cell number
(index) of a plain list, and each cell number of a counted list that the compiler cannot
work out, when the program reads the cell. The message gives the range the list allows:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၀၊ ၈၅ နှင့် ၉၀ တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် န သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
အမှတ်များ ၏ အကွက် န ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: cell 3 is outside the list, whose cells run from 0 to 2
```

To fix it, we compare the number with `အကွက်အရေအတွက်` before the read, or loop over the
cells with `အကွက်တိုင်း`.

### No cell to remove

`ဖျက်မည်` removes the last cell of a plain list. The count of such a list is known just
when the program runs, so the VM checks it at each remove, and a list with no cells stops
the program:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁] ဖြစ်ပါစေ။
ကိန်းများ ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။
ကိန်းများ ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: the list holds no cells, so it has no last cell to remove
```

`no character to remove` is the same stop for `နောက်ဆုံးစာလုံးကို ဖျက်မည်` on text (string)
with no characters. Both are fixed with a test of the count before the remove.

### Character out of range

A character number of text runs from 0 to the count minus 1, and the VM checks a number
from a variable when the program reads or writes the character:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာ သည် "ကခဂ" ဖြစ်ပါစေ။
ကိန်းရှင် န သည် ၅ ဖြစ်ပါစေ။
စာ ၏ စာလုံး န ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: character 5 is outside the text, whose characters run from 0 to 2
```

### Ranges, inserts and replaces

A [range](../language/lists.md#ranges) of `n` cells from cell `s` must end inside its
list, or the program stops with `slice out of range`:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် အရေ သည် ၅ ဖြစ်ပါစေ။
ကိန်းများ ၏ အကွက် ၁ မှစ၍ အရေ ကွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: 5 cells from cell 1 pass the end of 3 cells
```

The other edits of lists and text check their numbers in the same manner:

| Trap | When |
| --- | --- |
| `no place to insert` | an add in front of cell or character `N` where `N` is past the count; the count itself is the last place, which adds at the end |
| `end before start` | a replace from `A` to `B` where `B` comes before `A` |
| `text cut` | a range of text, counted in u8s, whose first or last u8 falls inside a character |
| `not UTF-8` | a list of u8s [turned into text](../language/text.md#text-from-u8s) whose u8s are not UTF-8; the message gives the address of the first u8 that is wrong |

## Room and limits

### Out of mem

An allocate statement that finds no room leaves the null address, but a resize cannot do
that, since it would lose the old block, so it stops the program. The size below is far
over the heap cap:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် အရေအတွက် သည် လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀၀၀၀၀၀၀၀၀ ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ကိန်းပြည့် အရေအတွက် ခုစာ ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: no heap room for 8000000000 octets
```

An added cell, and text that a program builds as it runs, from a template literal or a
join, need a heap block too, and stop the program in the same manner when the heap has no
room for them. `--max-heap` raises the cap ([Limits](running.md#limits)).

### Stack full

The locals of a call live in the stack region, which has 1 MiB at most with the default
limit. A list of 200000 integers needs 1600000 octets:

```zaka-stop
ကြီးမား ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် အကွက် ၂၀၀၀၀၀ ရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"အစ" ဟု ပြသမည်။
ကြီးမား ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အစ
the program stopped: no stack room for 1600000 more octets
```

To fix it, we use a plain list, whose cells live on the heap, or raise `--max-stack`.

### Stack overflow

A function that calls itself with no end fills the stack region, or reaches the depth
limit first. The limit is 4096 calls waiting to return, and `--max-depth` changes it:

```zaka-stop
အဆုံးမဲ့ ဟူသော လုပ်ဆောင်ချက် သည် န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အဆုံးမဲ့ ဟူသော လုပ်ဆောင်ချက်ကို န + ၁ ဖြင့် လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အဆုံးမဲ့ ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
the program stopped: more than 4096 nested calls
```

### Step limit

`--max-steps` caps the operations a run can take, which stops a loop that never ends:

```console
$ zakac run --max-steps 50 samples/mistakes/endless_loop.zaka
zaka_vm: the program stopped: stopped after 50 ops (in ပရိုဂရမ်, at samples/mistakes/endless_loop.zaka:4:41)
```

`--max-steps` is a flag of the VM alone, so it sends the run to the VM. The Run buttons of
these pages stop a program after 20 million operations in the same manner.

### Static full

On a platform with few addresses, the globals and the static texts can fill the space
before `main` starts. The message reads `global` for a static text too:

```console
$ zakac run --target-bits 8 nine-globals.zaka
zaka_vm: the program stopped: no room in the address space for a global of 1 octets (in main, at nine-globals.zaka:1:1)
```

## A whole result read as the wrong part

A call to a [function that can fail](../language/reading-results.md), kept as a whole
result, contains a good result or an error value. A read of the part that the result does
not contain, `x ထဲမှ ဒေတာ` or `x ထဲမှ အမှားအယွင်း ဒေတာ`, stops the run:

| Read | Result contains | Trap | Message |
| --- | --- | --- | --- |
| `x ထဲမှ ဒေတာ` | an error value | `not good` | `the result holds an error, not a good result: V`, with the error value shown as `ပြသမည်` shows it |
| `x ထဲမှ ဒေတာ` | a failure with no value | `not good` | `the result holds a failure, not a good result` |
| `x ထဲမှ အမှားအယွင်း ဒေတာ` | a good result | `not fault` | `the result is a good result, not an error` |

```zaka-stop
စစ် ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် "မပိတ်နိုင်ပါ" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် စစ်(၀) ဖြစ်ပါစေ။
ရ ထဲမှ အမှားအယွင်း ဒေတာ ကို ပြသမည်။
ရ ထဲမှ ဒေတာ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မပိတ်နိုင်ပါ
the program stopped: the result holds an error, not a good result: မပိတ်နိုင်ပါ
```

To fix it, we test which part is there before the read, with
`x ထဲတွင် ရလဒ်ကောင်း ရှိလျှင်` or `x ထဲတွင် အမှားအယွင်း ရှိလျှင်`. Both stops come from the
operation `halt`, with the cause `not_good` or `not_fault`.

## An optional read with no data

An [optional](../language/optionals.md) contains data or is blank. Reading the data of a
blank optional, `x ထဲမှ ဒေတာ`, stops the run with the trap `no data`, which has no value to
show. To fix it, we test with `x ထဲတွင် ဒေတာ ရှိလျှင်` before the read. The stop comes
from the operation `halt`, with the cause `no_data`:

```zaka-stop
ရှာ ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် အလွတ် ကို ပြန်ပေးမည်။
န ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် ရှာ(၀) ဖြစ်ပါစေ။
"အစ" ဟု ပြသမည်။
ရ ထဲမှ ဒေတာ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အစ
the program stopped: the optional holds no data
```

## A read of the data of another member

The data of a member of an enumeration, `<value> ကို <member> အဖြစ် ပြောင်းလဲသော ရလဒ်`,
is there when the value is that member. When the value is another member, the run stops
with the trap `wrong member`. To fix it, we test the member first, or give the data a name
in the test, `<value> သည် <name> ဟူသည့် <member> အဖြစ်သို့ ပြောင်းလဲနိုင်လျှင်`, which reads
it once the test holds. The stop comes from the operation `halt`, with the cause
`wrong_variant`:

```zaka-stop
ပုံ ဟူသော ဒေတာအမျိုးအစားကွဲတွင်
ဒေတာအနေဖြင့် ကိန်းပြည့် တစ်ခု ရှိသည့် စက်ဝိုင်း နှင့်
အမှတ်
တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ပုံများထဲမှ အမှတ် ဖြစ်ပါစေ။
"အစ" ဟု ပြသမည်။
ပ ကို စက်ဝိုင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အစ
the program stopped: the value is another member than the one read
```

## A write through a missing row

A read of a row of a table (map), `ဇယား`, returns an optional and never stops the run. A
statement that writes inside the value of a row needs the row: a write to a field or a
cell of it, an add to it or a remove from it. When no row has the index (key), the run
stops with the trap `no row`, while a write of the whole value adds the row instead. To
fix it, we write the whole value first, or test for the row. The stop comes from the
operation `halt`, with the cause `no_row`:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အုပ် သည် စာသား ဖြင့် ညွှန်းသော ကိန်းပြည့် စာရင်း ဇယား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [:] ဖြစ်ပါစေ။
အုပ် ထဲမှ အညွှန်း "က" ၏ တန်ဖိုး ကို [၁] ဟု ထားမည်။
အုပ် ထဲမှ အညွှန်း "က" ၏ တန်ဖိုး ထဲသို့ ၂ ကို ထည့်မည်။
အုပ် ကို ပြသမည်။
အုပ် ထဲမှ အညွှန်း "ခ" ၏ တန်ဖိုး ထဲသို့ ၃ ကို ထည့်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
["က": [၁၊ ၂]]
the program stopped: no row of the ဇယား has this index
```

## Threads

A program with [threads](../language/threads.md) can stop for reasons that a program with
one thread never meets.

| Trap | What happened | Fix |
| --- | --- | --- |
| `thread trapped` | a wait found that its thread stopped, under the default `--trap unwind` ([A trap in a thread](../language/threads.md#a-trap-in-a-thread)); the thread wrote its own `thread N stopped` line first | fix the stop in the thread, whose line names it |
| `thread waited` | a second wait for a thread whose result an earlier wait took ([Waiting for a thread](../language/threads.md#waiting-for-a-thread)) | keep the result of the first wait in a variable |
| `lock broken` | a lock block takes the lock of an object whose holder stopped while it held the lock ([Objects threads share](../language/threads.md#objects-threads-share)) | fix the stop in the holder |
| `threads not positive` | a parallel loop or map split into a count of threads that is zero or negative ([Parallel loops and maps](../language/threads.md#parallel-loops-and-maps)) | check the count before the loop |
| `pipe closed` | a send to a closed channel, or a second close | close a channel once, after the last send |
| `deadlock` | each thread waits, for another thread or for a channel, and none can go on | close the channel, or send to it, from some thread |

With `--trap abort`, a stop in a thread stops the whole program at once, with the reason of
that thread. A stop in the program start block stops the whole program in both modes. On
the VM, the turns of the threads come in the same order on each run, and
`--shuffle-thread-order` shuffles them, to find a mistake that hangs on the order
([Turns on the VM](../language/threads.md#turns-on-the-vm)).

A send to a closed channel:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လမ်း သည် ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
လမ်း ကို ပိတ်မည်။
"အစ" ဟု ပြသမည်။
လမ်း ထဲသို့ ၁ ကို ပို့မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အစ
the program stopped: the channel 1 is closed: no value can go to it, and it cannot close again
```

A receive from a channel that no thread sends to or closes would wait for ever, so the VM
stops the program:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လမ်း သည် ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
"အစ" ဟု ပြသမည်။
ကိန်းရှင် ရ သည် လမ်း မှ ရရှိသော ဒေတာ ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အစ
the program stopped: each thread waits; none can go on
```

A parallel loop split into zero threads:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အရေ သည် ၀ ဖြစ်ပါစေ။
"အစ" ဟု ပြသမည်။
၁ မှ ၄ အထိ ကိန်း အနေဖြင့် အပြိုင်လုပ်ငန်း အရေ ခုခွဲ၍ ရေတွက်၍ ကိန်း ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အစ
the program stopped: the count of threads of a parallel loop or map is not positive
```

## Rights, damaged programs and output

**Permission denied.** A C call with no ffi right, and a program start block that reads
the environment with no env right, have no error result to fail with, so these stop the
program. The message names the flag that grants the right, as
`calling the C function N needs --allow-ffi` or
`the environment of the start block needs --allow-env`. See
[Permissions](running.md#permissions).

**Malformed.** A module read from a damaged artifact can contain operations that break the
rules of the code generator, such as a slot that does not exist. The VM [checks an
artifact](running.md#checking-an-artifact) before it runs and rejects most such faults
there. A fault the check cannot see, such as a value passed to C that does not suit its
parameter, stops the program with `malformed ops: ...`, and the VM does not crash.

**Write.** The VM stops the program when it cannot write the output, such as when the
reader of a pipe has closed it.
