---
title: Compile time
description: What the compiler works out before the program runs - the compile-time conditional that it settles for the target, with the five facts of the target; comptime functions, the places where their calls run, what such a run can do, and their errors.
---

# Compile time

The compiler works out two things before the program runs: the arm of a *compile-time
conditional* that it keeps for the target, and the value of each call of a *comptime
function* (compile-time function, `constexpr` in C++) that appears where a value is needed
at compile time. The Learn book teaches comptime functions in [Compile
time](../../learn/functions/compile-time.md).

A compile-time conditional exists because a C lib is not named and laid out the same on
each OS. The UCRT of Windows, for example, calls `strdup` `_strdup` and has no
`snprintf`. The program names the lib to read and the function to call, and a
compile-time conditional lets it choose them for each OS. The compiler keeps the arm whose
condition is true for the target it builds for, and drops the other arms before it checks
names and data kinds (types).

## The conditional

```text
အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် <condition> (လျှင်|ဖြစ်လျှင်) <branch>
{ ထိုသို့မဟုတ်ဘဲ <condition> (လျှင်|ဖြစ်လျှင်) <branch> }
[ သို့မဟုတ်ပါက <branch> ]
```

We write `ကွန်ပိုင်းလုပ်ချိန်တွင်` ("at compile time") right after `အကယ်၍` to make the
whole chain one that the compiler settles. The other arms are written as in a plain
[conditional](statements.md).

- The compiler keeps the statements of the first arm whose condition is true, or of the
  else branch, and puts them in place of the conditional. The arm opens no scope, so a
  name that it declares is visible after the conditional. When no arm is true and there
  is no else branch, nothing is left.
- A conditional can appear at the top level, around declarations, C functions, structs,
  traits and imports, or among the statements of a block. At the top level, its arms
  contain top-level statements.
- The arms that the target drops must parse, but their names and data kinds are never
  checked, so an arm for Windows can name a lib or a C function that this host lacks.

## The facts of the target

A condition reads the five facts of the target, which [the core module](core.md) declares
as constants:

| Constant | Values |
| --- | --- |
| `အလုပ်လုပ်မည့်ကွန်ပျူတာစနစ်` | the OS: `"windows"`, `"macos"`, `"linux"` or `"wasi"` |
| `အလုပ်လုပ်မည့်စီပီယူဗိသုကာပုံစံ` | the CPU: `"x86_64"`, `"aarch64"`, `"x86"`, `"arm"`, `"riscv64"` or `"wasm32"` |
| `အလုပ်လုပ်မည့်အပလီကေးရှင်းဘိုင်နရီကြားခံစနစ်` | the ABI that the machine code and the libs share: `"gnu"`, `"msvc"`, `"apple"`, `"wasip1"` or `"wasip2"`. When `--target-abi` gives none, it is the common ABI of the OS: `"apple"` on macOS, `"gnu"` on Linux, `"wasip1"` on WASI, and on Windows the ABI that `zakac` itself was built for. |
| `အလုပ်လုပ်မည့်ပလက်ဖောင်းဘစ်` | the bit count of `--target-bits`: 8, 16, 32 or 64. When it is not given, 32 for `"wasm32"` and 64 for the other CPUs. |
| `ကွန်ပိုင်လာအထွက်ကုဒ်အမျိုးအစား` | the code that the compiler writes: `"vm"`, `"c"` or `"native"` |

A condition can use the facts, text and digits, `==` and `!=` (and `<`, `<=`, `>`, `>=`
on the bit count), `ဖြစ်၍` (`&&`), `သို့မဟုတ်` (`||`), `မဟုတ်`, parentheses, and calls of
[comptime functions](#conditions-that-call-comptime-functions). Each condition of a chain
must be one that the compiler can settle, so each other part of a condition is ZK0255. A
value that a fact never has is ZK0255 too, because `"window"` would make a branch that
never runs.

The facts are constants all through the program as well, so `အလုပ်လုပ်မည့်ကွန်ပျူတာစနစ်
ကို ပြသမည်။` prints the OS, and a plain conditional can test a fact when the program
runs.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
// an arm that is true on each target, and one that is true on none
အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် အလုပ်လုပ်မည့်ပလက်ဖောင်းဘစ် >= ၈ ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည် သည် "built" ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် အလုပ်လုပ်မည့်ပလက်ဖောင်းဘစ် > ၆၄ ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
နာမည်မရှိ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အမည် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
built
```

```zaka-error ZK0255
အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် အလုပ်လုပ်မည့်ကွန်ပျူတာစနစ် == "window" ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ ပြတင်း သည် ၁ ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## C functions on each OS

The arm of each OS declares the C function that its lib has. A small function of the
program can give the two the same name, so that the calls are the same on each OS:

```zaka-file
// dup.zaka
အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် အလုပ်လုပ်မည့်ကွန်ပျူတာစနစ် == "windows" ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
_strdup ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။
strdup ဟူသော လုပ်ဆောင်ချက် သည်
s ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
_strdup ဟူသော လုပ်ဆောင်ချက်ကို s ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
သို့မဟုတ်ပါက အောက်ပါတို့ကို လုပ်ဆောင်မည်။
strdup ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "libc" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
s ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

A C function that accepts extra values cannot be wrapped like this, because a function of
the program cannot accept extra values. Instead, the call itself goes in a conditional
with an arm for each OS, as `snprintf` does on [C functions](functions-c.md#other-values).
The VM opens `ucrtbase.dll` for `"libc"` on Windows, where `snprintf` is missing, so the
program reads `_snprintf` from `"msvcrt"` there when `ကွန်ပိုင်လာအထွက်ကုဒ်အမျိုးအစား`
is `"vm"`. The C target links its own `snprintf`.

## The target of a build

The target is the host that the compiler runs on: its OS, its CPU, the bit count of
`--target-bits`, and the code of the command, which is `"c"` for
`zakac build --backend c`, `"native"` for `zakac build --backend native` and
`zakac run --jit`, and `"vm"` for the rest. We can use `--target-os` and `--target-arch`
to name another target, and `--target-os wasi` uses the CPU `wasm32` unless
`--target-arch` names one ([Commands](../compiler/commands.md#options-of-the-compile-stages)).

- A program that has a compile-time conditional, or names a fact, is settled for its
  target, and its operations for the VM record that target. The VM refuses to run them on
  another OS or CPU, with a message that names both.
- The C target builds an executable for the OS and the CPU of the host alone, with either
  ABI on Windows ([Native programs](../compiler/native.md#windows)). For another target, it
  writes the C files (`--emit c`) so that we can build them there.
- The native target also links an executable for the OS and the CPU of the host alone. For
  another target, it writes the object file (`--emit obj`) or the Cranelift IR
  (`--emit clif`).
- The explorer settles a program for Linux on x86-64.

## Comptime functions

```text
<name> ဟူသော [အများသုံး] ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော [စိတ်မချရသော] လုပ်ဆောင်ချက် သည် ...
```

To declare a comptime function, we write `ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော`
("that can run at compile time") between `ဟူသော` and `လုပ်ဆောင်ချက်`, together with the
other marks of the function. The rest of the function signature and the block are the
same as for each other [function](functions.md).

- The mark goes on a function declared at the top level of a file, with a block. The
  program start block, a C function, a function of the host, a function that the compiler
  provides and a function declared inside another function cannot have the mark: ZK0256.
  The compiler takes the mark before `ပရိုဂရမ်သည်` as the start of a sentence, so it
  reports that case as ZK0256 too.
- The result must be a value that a constant can have: a number, a character, true or
  false, a member of an enumeration, text, a plain or counted list, or a struct of these.
  A function with no result, or with a pointer, a table, a function value, an optional
  kind or a result kind as its result, is ZK0489.
- A comptime function is still a function, so a call of it in another place runs with the
  program.

```zaka
နှစ်ထပ် ဟူသော ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် သည်
n ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
n * n ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းသေ အကွက်ရေ သည် နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက်ကို ၂ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဂ သည် အကွက် အကွက်ရေ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ဂ ကို ပြသမည်။
နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက်ကို ၉ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၀၊ ၀၊ ၀၊ ၀]
၈၁
```

```zaka-error ZK0489
ဇ ဟူသော ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ဖြင့် ညွှန်းသော ကိန်းပြည့် ဇယား တစ်ခု ကို ပြန်ပေးမည်။
ဇယားအလွတ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## Where a call runs at compile time

A call of a comptime function runs while the compiler runs when it appears in a place
that needs its value before the program runs:

- the value of a constant (`ကိန်းသေ`), alone or inside an expression;
- the count of a counted list;
- the starting value of a variable or a constant of an imported file;
- the value of a member of an enumeration, and a case of a switch;
- the condition of a compile-time conditional.

Each argument must be known at compile time: literals, constants, operators on them and
other comptime calls. A variable as an argument is ZK0608. The compiler puts the value in
place of the call, so the later stages, the VM and the C target see a literal.

```zaka-error ZK0608
နှစ်ထပ် ဟူသော ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် သည်
n ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
n * n ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် v သည် ၃ ဖြစ်ပါစေ။
ကိန်းသေ က သည် နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက်ကို v ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
က ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

The compiler runs a call once each value that its function reads is known, so when the
function reads a constant whose value is another comptime call, that call runs first. A
call whose function reads a value that waits on the call itself has no order that works,
which is ZK0609.

## What a run can do

The compiler runs a call on the [VM](../vm/README.md), in a small program made of the
function, the declarations of the program that do not wait on another comptime call, and
the arguments.

- The function can call each function of the program, comptime or not, and use pointers,
  the heap, tables, closures, operators on structs and generics.
- The run has the rights of the `--comptime-allow-*` flags of the command, and none when
  no flag is given ([Commands](../compiler/commands.md)): `--comptime-allow-read[=<paths>]`,
  `--comptime-allow-write[=<paths>]`, `--comptime-allow-env[=<names>]`,
  `--comptime-allow-ffi` and `--comptime-allow-all`. These are granted as the [rights of
  a run](../vm/running.md#permissions) are. A path in the program starts from the folder
  of the program. Nothing is asked, so an effect without the right stops the run.
- Reading input stops the run.
- The language server and the explorer run a call with reads under the folder of the
  program at most, and with no write or C call, whatever flags are given.
- A run has a cap on its operations (50 000 000, or 1 000 000 in the language server and
  the explorer), on its heap and on the depth of its calls.
- What the run prints becomes a note at the call, ZK0612.

A run that stops, through a trap, a cap or an effect without the right, stops the build
with ZK0611, which names the call and the reason, and its help lists the calls that led
there. When the small program does not compile, the compiler reports its first error as
ZK0611 too.

```zaka-error ZK0611
ခွဲ ဟူသော ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် သည်
n ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
၁၀ / n ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းသေ အပိုင်း သည် ခွဲ ဟူသော လုပ်ဆောင်ချက်ကို ၀ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အပိုင်း ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## Conditions that call comptime functions

A condition can call a comptime function, alone or next to the facts, as in
`<call> ဖြစ်၍ အလုပ်လုပ်မည့်ပလက်ဖောင်းဘစ် == ၆၄`. The compiler then reads the program in
passes:

1. A chain whose condition calls a function remains open: its arms are parsed, but their
   imports are not read.
2. The compiler checks a duplicate of the program, with each open condition as a constant
   in place of its chain, and this check runs the calls.
3. The compiler reads the program again, and each chain keeps the arm that the values
   pick. An arm that is kept can contain its own chain, which the next pass settles.

A condition that calls a function declared in an arm of another open chain waits for the
pass that keeps that arm. A function declared in an arm of the same chain whose condition
calls it is ZK0609.

```zaka-error ZK0609
အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် ကိုယ်တိုင် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိုယ်တိုင် ဟူသော ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
မှန် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## Errors

| Code | When |
| --- | --- |
| ZK0255 | a compile-time condition that contains something the compiler cannot settle, or compares a fact with a value that the fact never has |
| ZK0256 | the comptime mark on something other than a top-level function with a block |
| ZK0489 | a comptime function whose result is not a value that a constant can have |
| ZK0608 | an argument of a comptime call, in a place that needs it at compile time, that is not known then |
| ZK0609 | a comptime call that needs a value that waits on the call itself |
| ZK0610 | a comptime call in a tool that has no VM to run it |
| ZK0611 | a comptime call whose run stopped: a trap, a cap, an effect without the right, or code that does not compile |
| ZK0612 | a note: what a comptime call printed |

```zaka-error ZK0256
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အတွင်း ဟူသော ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
၁ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```
