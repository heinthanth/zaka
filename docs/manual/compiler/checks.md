---
title: Compiler checks
description: What the flow checks find in each program, the check that each value is set before it is read, how the pointer lint of unsafe code is reported, and the settings that tune them, with one program shown under each setting.
---

# Compiler checks

Mistakes with storage are caught twice: the compiler proves what it can from the text of
the program, and the VM checks each read and write while the program runs. The compiler's
part is the checker that runs after constant folding (see [Stages](stages.md)). It holds
two sets of findings:

- **The flow checks**, on each program: a read of a variable, a field or a cell before
  the program gives it a value (ZK0722, and the warning ZK0723 where the check cannot
  tell); a call that reads a global no path to it has set (ZK0724); a cell or an offset
  past the end; and an operation that overflows for each value it can see. These are
  errors, save ZK0723 and the overflow, which are warnings.
- **The pointer lint**, on unsafe code: the mistakes that pointers alone can make, such
  as a use after a give-back, a leak, or the address of a local that escapes. These follow
  `--mem-check`, and are warnings unless the setting asks for errors.

[Unsafe Zaka](../language/unsafe.md) gives the rules the lint follows, and the list of
its findings.

| Setting | Command line | In the Explorer | Default |
| --- | --- | --- | --- |
| how the lint is reported | `--mem-check error`, `warn` or `off` | **Ownership check** | `warn` |
| report what cannot be proved | `--mem-strict` | **Strict ownership check** | off |
| the level of one group or code | `--allow`, `--warn`, `--error` | | none |
| refuse unsafe code | `--no-unsafe` | **Forbid unsafe code** | off |
| what overflow does | `--overflow stop` or `wrap` | **Stop on overflow** | `stop` |

`--mem-check`, `--mem-strict`, `--overflow`, `--allow`, `--warn` and `--error` work on
`borrowck`, `build` and `run`, and `--no-unsafe` on `irgen` too. See
[Commands](commands.md). A [config file](configuration.md) can set each of them.

## What each setting changes

The VM's own checks never change with these settings. A read through a null address, past
the end of a block, or into a block that was given back stops the program under each of
them. The settings change what the *compiler* does with the same finding.

| Setting | Effect |
| --- | --- |
| `--mem-check warn` | each finding of the lint is a warning: the program runs, and the VM stops it if it reaches the bad access |
| `--mem-check error` | each finding of the lint is an error: nothing runs |
| `--mem-check off` | the lint reports nothing: the VM is the one check left |
| `--mem-strict` | adds the warnings for what the compiler cannot prove safe (ZK0711) |
| `--allow`, `--warn`, `--error` | set the level of one group of the lint, or of one code, over `--mem-check`: `--error leaks` makes each leak an error while the rest remain warnings |
| `--no-unsafe` | each use of `စိတ်မချရသော` is an error (ZK0471) |

The lint holds ZK0702 to ZK0708, ZK0712 and ZK0721 wherever the compiler finds them, and
ZK0709, ZK0710 and ZK0711 when it finds them inside unsafe code. Outside unsafe code, the last three
come from the flow checks and keep their own level: ZK0709 and ZK0710 are errors, ZK0711 a
warning, in each mode. ZK0701, a read after an owned hand-over, is an error in each mode
too, since the place it reads holds the null pointer for certain. ZK0722, ZK0723 and ZK0724
never move with the mode, in unsafe code or out of it: the kinds of Zaka promise that no
value is read before it is set.

## One program, three modes

A block of four integers has cells 0 to 3, and the program reads cell 4:

```zaka-warning ZK0710
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ + ၃ ၏ တန်ဖိုး ကို ပြသမည်။
ပ + ၄ ၏ တန်ဖိုး ကို ပြသမည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

With the default, `--mem-check warn`, the finding is a warning, the program starts, and
the VM stops it at the bad read. Cell 3 is printed first, since it was fine. A new block
holds zeros:

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ + ၃ ၏ တန်ဖိုး ကို ပြသမည်။
ပ + ၄ ၏ တန်ဖိုး ကို ပြသမည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၀
the program stopped: access of 8 octets at 0x101020 lands outside each live block
```

```console
$ zakac run past.zaka
warning[ZK0710]: offset 4 is past the end of a block of 4 elements
 --> past.zaka:6:1
  |
6 | ပ + ၄ ၏ တန်ဖိုး ကို ပြသမည်။
  | ^^^^^
၀
zaka_vm: the program stopped: access of 8 octets at 0x101020 lands outside each live block (in ပရိုဂရမ်, at past.zaka:6:1); block at 0x101000, 32 octets, heap
```

With `--mem-check error` the compiler refuses the program and nothing runs:

```zaka-error ZK0710 mem-check=error
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ + ၃ ၏ တန်ဖိုး ကို ပြသမည်။
ပ + ၄ ၏ တန်ဖိုး ကို ပြသမည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```console
$ zakac run --mem-check error past.zaka
error[ZK0710]: offset 4 is past the end of a block of 4 elements
 --> past.zaka:6:1
  |
6 | ပ + ၄ ၏ တန်ဖိုး ကို ပြသမည်။
  | ^^^^^
```

With `--mem-check off` the compiler is silent, and the VM stops the program at the same
read:

```zaka-stop mem-check=off
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ + ၃ ၏ တန်ဖိုး ကို ပြသမည်။
ပ + ၄ ၏ တန်ဖိုး ကို ပြသမည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၀
the program stopped: access of 8 octets at 0x101020 lands outside each live block
```

The runs that start end at the same read. What differs is when the reader hears of it:
before a thing ran, as a warning followed with the stop, or just in the stop.

| Mode | Message from the compiler | Program runs | Exit code |
| --- | --- | --- | --- |
| `warn` | `warning[ZK0710]` | up to the bad read | 1 |
| `error` | `error[ZK0710]` | no | 1 |
| `off` | none | up to the bad read | 1 |

The default lets a program with a pointer mistake run to the mistake, so the VM shows
where it goes wrong, which is the point of the lessons on [unsafe
Zaka](../../learn/pointers/README.md). Pick `error` to keep such a program from running at
all. The compiler cannot find each mistake: an offset that comes from input is out of its
reach, and the VM's check alone can catch it.

## Safe code keeps its errors

Outside unsafe code, a finding of the flow checks does not move with `--mem-check`. A
function value is never null, so a variable of a function kind that a path reaches with no
function given is a read of an unset value, ZK0722, even under `--mem-check off`:

```zaka-error ZK0722 mem-check=off
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် (လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ဖြစ်ပါစေ။
ဖ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```console
$ zakac run --mem-check off call.zaka
error[ZK0722]: "ဖ" can be unset here: a path reaches this read with no value given to it
```

The rules of that check, for variables, fields, cells and globals, are in
[Statements](../language/statements.md#set-before-read). Safe code cannot hold a mistake of
the lint at all, since it never reads through a pointer: each block it makes is counted,
and the VM frees it.

## Strict: what cannot be proved

The compiler is quiet about a pointer when it cannot tell whether the pointer holds the
null pointer, `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`. A new block can fail to come, since the heap
can run out, so a pointer that has just been filled can point at nothing. `--mem-strict`
asks for a warning at each read or write through such a pointer that was never compared
with the null pointer:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၁ ဟု ထားမည်။
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁
```

Without `--mem-strict` the program above compiles with no message. With it, the compiler
adds a warning (ZK0711); the program still runs:

```zaka-warning ZK0711 mem-strict=on
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၁ ဟု ထားမည်။
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```console
$ zakac run --mem-strict nul.zaka
warning[ZK0711]: "ပ" can point at nothing here; compare it with "မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ" first
 --> nul.zaka:5:1
  |
5 | ပ ၏ တန်ဖိုး ကို ၁ ဟု ထားမည်။
  | ^
```

The warning goes when the program compares the pointer with the null pointer first. ZK0711 is a
warning in the modes `error` and `warn`, and `--mem-check off` drops it with the rest of
the lint. Strict adds warnings alone; it never turns a program that compiles into one
that does not.

## Safe Zaka alone

For a lesson or a project that must keep to safe Zaka, `--no-unsafe` refuses each use of
`စိတ်မချရသော`, on a function or on a block:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
"ပြီး" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ပြီး
```

```zaka-error ZK0471 no-unsafe=on
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
"ပြီး" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```console
$ zakac run --no-unsafe unsafe.zaka
error[ZK0471]: this program keeps to safe Zaka, so it cannot hold unsafe code ("စိတ်မချရသော")
 --> unsafe.zaka:2:1
```

The refusal is an error in each `--mem-check` mode, because it is a rule of the language,
read with names and kinds. Each use is spelled with `စိတ်မချရ`, so a search for that
word finds each place the compiler does not vouch for.

## Overflow

`--overflow stop`, the default, stops the program at an integer operation whose result
lies outside its kind, and the flow checks warn (ZK0719) about an operation that
overflows for each value of its operands the checks can see. `--overflow wrap` wraps the result
around and drops the warning. See
[Arithmetic](../language/expressions.md#arithmetic).

## Variables: checked and packed

One more setting belongs near these, though it is a setting of the VM and not of the
compiler: `--frames checked|packed` on `run`. In `checked` form (the default) each
variable sits in a block of its own with a gap after it, so a read past a variable stops
the program. In `packed` form the locals of a call sit next to each other, as in C, and a
read past one reaches its neighbour. Blocks from the heap keep their checks in both forms.
See [Addresses and blocks](../vm/blocks.md).

## Where each setting is read

| Setting | Read in | Result |
| --- | --- | --- |
| `--no-unsafe` | names and kinds | ZK0471, an error |
| `--mem-check`, `--mem-strict` | the checker | the level of the lint, and ZK0711; never ZK0722 to ZK0724 |
| `--allow`, `--warn`, `--error` | the checker | the level of the codes of the lint, ZK0719 and ZK0723; never ZK0701, ZK0722 or ZK0724 |
| `--overflow` | the checker and the code generators | what an overflow does, and ZK0719 |
| `--frames` | the VM | where the variables sit |

The checker runs when the stages before it found no error, so a program with a kind error
shows that error first and the findings of the checker after it is fixed. See
[Stages](stages.md).
