---
title: Pointer checks
description: The settings that decide how the compiler reports pointer mistakes, with one program shown under each of them.
---

# Pointer checks

Pointer mistakes are caught twice: the compiler proves what it can from the text of the
program, and the machine checks each read and write while the program runs. Three
settings tune the first of these. This page lists them and shows the same program under
each setting. [Ownership and borrowing](../language/ownership.md) gives the rules the check
follows, and [Ownership and borrowing](../../learn/pointers/ownership.md#turning-the-checks-down)
in Learn shows the settings in the Explorer.

| Setting | Command line | In the Explorer | Default |
| --- | --- | --- | --- |
| how findings are reported | `--mem-check error`, `warn` or `off` | **Ownership check** | `error` |
| report what cannot be proved | `--mem-strict` | **Strict ownership check** | off |
| refuse raw addresses | `--no-raw` | **Forbid raw addresses** | off |

Each one works on `borrow-check`, `codegen` and `run`, and `--no-raw` on `irgen` too.
See [Commands](commands.md).

## What each setting changes

The machine's own checks never change with these settings. A read through a null
address, past the end of a block, or into a block that was given back stops the program
under each of them. The settings change what the *compiler* does with the same finding.

| Setting | Effect |
| --- | --- |
| `--mem-check error` | each finding of the check is an error: nothing runs |
| `--mem-check warn` | each finding is a warning: the program runs, and the machine stops it if it reaches the bad access |
| `--mem-check off` | the check reports nothing: the machine is the one check left |
| `--mem-strict` | adds the warnings for what the check cannot prove safe (ZK0711) |
| `--no-raw` | each raw address kind (`လိပ်စာအကြမ်း`) is an error (ZK0717) |

`warn` and `off` move the findings of the borrow checker alone: ZK0701 to ZK0704 and
ZK0706 to ZK0712. The rules of kinds, ZK0705 and ZK0713 to ZK0717, are errors in each
mode, since a program that breaks one of them has no meaning to check. A move leaves
`ဗလာ` in the old owner in each mode as well, so a program does the same thing whichever
mode compiled it.

## One program, three modes

A block of four integers has cells 0 to 3, and the program reads cell 4:

```zaka-error ZK0710
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ + ၃ ၏ တန်ဖိုး ကို ပြသပါ။
ပ + ၄ ၏ တန်ဖိုး ကို ပြသပါ။
ပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

With the default, `--mem-check error`, the compiler refuses the program and nothing runs:

```console
$ zakac run past.zaka
error[ZK0710]: offset 4 is past the end of a block of 4 elements
 --> past.zaka:4:1
  |
4 | ပ + ၄ ၏ တန်ဖိုး ကို ပြသပါ။
  | ^^^^^
```

With `--mem-check warn` the same finding is a warning, the program starts, and the machine
stops it at the bad read. Cell 3 is printed first, since it was fine. A new block holds
zeros:

```zaka-stop mem-check=warn
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ + ၃ ၏ တန်ဖိုး ကို ပြသပါ။
ပ + ၄ ၏ တန်ဖိုး ကို ပြသပါ။
ပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
၀
the program stopped: access of 8 octets at 0x101040 lands outside each live block
```

```console
$ zakac run --mem-check warn past.zaka
warning[ZK0710]: offset 4 is past the end of a block of 4 elements
 --> past.zaka:4:1
  |
4 | ပ + ၄ ၏ တန်ဖိုး ကို ပြသပါ။
  | ^^^^^
၀
zaka_vm: the program stopped: access of 8 octets at 0x101040 lands outside each live block (in main, op 17); block at 0x101020, 32 octets, heap
```

With `--mem-check off` the compiler is silent, and the machine stops the program at
the same read:

```zaka-stop mem-check=off
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ + ၃ ၏ တန်ဖိုး ကို ပြသပါ။
ပ + ၄ ၏ တန်ဖိုး ကို ပြသပါ။
ပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
၀
the program stopped: access of 8 octets at 0x101040 lands outside each live block
```

The three runs end at the same read. What differs is when the reader hears of it: before
a thing ran, as a warning followed with the stop, or just in the stop.

| Mode | Message from the compiler | Program runs | Exit code |
| --- | --- | --- | --- |
| `error` | `error[ZK0710]` | no | 1 |
| `warn` | `warning[ZK0710]` | up to the bad read | 1 |
| `off` | none | up to the bad read | 1 |

Pick `error` to find mistakes soon, as the default does. Pick `warn` or `off` to see
what the machine does with a mistake, which is the point of the lessons on [pointer
mistakes](../../learn/pointers/mistakes.md). The compiler cannot find all mistakes: a cell number that comes from input is out of its reach, and the machine's
check alone can catch it.

## Strict: what cannot be proved

The compiler is quiet about a pointer when it cannot tell whether the pointer holds
`ဗလာ`. A new block can fail to come, since the heap can run out, so a pointer that has
just been filled can hold `ဗလာ`. `--mem-strict` asks for a warning at each read or write
through such a pointer that was never compared with `ဗလာ`:

```zaka
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၁ ဟု ထားပါ။
ပ ၏ တန်ဖိုး ကို ပြသပါ။
ပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
၁
```

Without `--mem-strict` the program above compiles with no message. With it, the compiler
adds a warning (ZK0711); the program still runs:

```zaka-warning ZK0711 mem-strict=on
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၁ ဟု ထားပါ။
ပ ၏ တန်ဖိုး ကို ပြသပါ။
ပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```console
$ zakac run --mem-strict nul.zaka
warning[ZK0711]: "ပ" can be ဗလာ here; compare it with ဗလာ first
 --> nul.zaka:3:1
  |
3 | ပ ၏ တန်ဖိုး ကို ၁ ဟု ထားပါ။
  | ^
```

The warning goes when the program compares the pointer with `ဗလာ` first. ZK0711 is a
warning in the modes `error` and `warn`, and `--mem-check off` drops it with the rest.
Strict adds warnings alone; it never turns a program that compiles into one that does not.

## No raw addresses

A raw address (`လိပ်စာအကြမ်း`) is a pointer that the compiler does not follow: it has
no owner to check. The machine still checks each use. For a lesson that must remain inside
the ownership check, `--no-raw` refuses each raw kind:

```zaka
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာအကြမ်း ဖြစ်ပါစေ။
"ပြီး" ဟု ပြသပါ။
```

```text
ပြီး
```

```zaka-error ZK0717 no-raw=on
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာအကြမ်း ဖြစ်ပါစေ။
"ပြီး" ဟု ပြသပါ။
```

```console
$ zakac run --no-raw raw.zaka
error[ZK0717]: raw addresses are not allowed here (--no-raw)
 --> raw.zaka:1:17
  |
1 | ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာအကြမ်း ဖြစ်ပါစေ။
  |            ^^^^^^^^^^^^^^^^
```

The refusal is an error in each `--mem-check` mode, because it is a rule of kinds.
Each raw kind is spelled with `အကြမ်း`, so a search for that word finds each place that
the compiler does not follow.

## Variables: checked and packed

One more setting belongs near these, though it is a setting of the machine and not of
the compiler: `--frames checked|packed` on `run`. In `checked` form (the default) each
variable sits in a block of its own with a gap after it, so a read past a variable
stops the program. In `packed` form the locals of a call sit next to each other, as in
C, and a read past one reaches its neighbour. Blocks from the heap keep their checks in
both forms. See [Checked and packed blocks](../vm/blocks.md).

## Where each setting is read

| Setting | Read in | Result |
| --- | --- | --- |
| `--no-raw` | names and kinds | ZK0717, an error |
| `--mem-check`, `--mem-strict` | the borrow checker | the level of ZK0701 to ZK0704 and ZK0706 to ZK0712, and ZK0711 |
| `--frames` | the machine | where the variables sit |

The borrow checker runs just when the stages before it found no error, so under
`--mem-check error` a program with a kind error shows that error first and the pointer
findings after it is fixed. See [Stages](stages.md).
