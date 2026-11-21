---
title: The VM
description: What the Zaka VM is made of: its operations, its calls, its values and its checked storage.
---

# The VM

The Zaka VM (virtual machine) runs the operations (instructions) generated for it. It
uses a stack for values and a call stack to track active function calls and where to
return. The VM divides its storage (memory) into *blocks* (allocations). Each variable
has an address (pointer) in that storage.

The VM checks each read and write through an address. A mistake stops the program (trap)
with a message that names the operation and the block, and nothing in the program gets
damaged. [Running a program](running.md#when-a-program-runs-on-the-vm) tells when a run
uses the VM and when it uses native code.

## The parts

```text
 module (from the compiler)            VM (what runs)
 ┌────────────────────────────┐        ┌───────────────────────────────────────────┐
 │ platform bits              │        │ calls, innermost last                     │
 │ globals    g0 g1 ...       │        │  ┌─────────────────────────────────────┐  │
 │ texts      t0 t1 ...       │        │  │ function  next operation   locals   │  │
 │ functions  f0 f1 ... main  │ ─────▶ │  └─────────────────────────────────────┘  │
 └────────────────────────────┘        │ operand stack     7   0x1040   မှန်       │
                                       │ output            text written so far     │
                                       │ storage                                   │
                                       │  ┌───────┬────────┬──────────────┬──────┐ │
                                       │  │ null  │ static │ stack region │ heap │ │
                                       │  └───────┴────────┴──────────────┴──────┘ │
                                       └───────────────────────────────────────────┘
```

| Part | What it contains |
| --- | --- |
| module | the functions as lists of [operations](instructions.md), the globals, the static texts, the formats of text built while the program runs, and the platform bits |
| call | one function in progress: the index of its next operation and the address of each of its locals |
| operand stack | the values a sentence is working on; an operation pops its inputs and pushes its result |
| storage | the blocks where globals, static texts, locals and the heap live; see [Addresses and blocks](blocks.md) |
| output | the text that print operations wrote and the command has not written out so far |
| threads | each [thread](../language/threads.md) the program started, with its own calls, operand stack and stack region; the VM runs them in turns on one OS thread |

The last function of a module is `main`: it sets the starting values of the top-level
variables and calls the program start block (entry point), the function named
`ပရိုဂရမ်`. The VM starts at `main`, and the run ends when `main` returns or the program
[stops](traps.md).

## Values

A value on the operand stack carries its data kind (type). An operation reads the data
kind from its inputs, so `add` works for each integer width and for decimal numbers alike.

| Value | In storage |
| --- | --- |
| integer, signed or unsigned | one octet (byte) for each 8 bits of its width; the plain `ကိန်းပြည့်` has the platform width |
| decimal number, 32 or 64 bits | 4 or 8 octets, in IEEE 754 form |
| character | 4 octets, one Unicode scalar value |
| true/false value | 1 octet, `0` or `1` |
| address | one octet for each 8 bits of the platform width; the null address, `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`, is address 0 |

Octets are stored least significant first. A load of octets that are no value of the data
kind, such as a true/false octet of `7`, stops the program with the `bad value` trap.

A list is not one value. It is a run of cells in a block, and the operations that work on
it go through its address. See [Lists](../language/lists.md).

## Each variable has a block

The operand stack is for the values of a sentence while it runs, and has no addresses. A variable is different,
because each global and each local lives in storage. So `x ၏ လိပ်စာ` costs nothing more
than an operation that pushes the address of the block of `x`, and a read through a stale
address finds the freed block of the variable it pointed at.

A call puts its locals at the top of the stack region, and `ret` frees that room again.
[Addresses and blocks](blocks.md) covers the details.

## The platform

We can use `--target-bits` to pick the width of `ကိန်းပြည့်`, `လက္ခဏာမဲ့ ကိန်းပြည့်` and
each address: 8, 16, 32 or 64 (the default). The storage has `2^bits` addresses, at most
`2^64`. On a 16-bit platform the plain integer has 16 bits, so a result past 32767 stops
the program:

```console
$ zakac run --target-bits 16 program.zaka
```

A width that is not the address width of the host sends a plain `zakac run` to the VM,
since native code uses the addresses of the host.

## C functions

A program can declare [C functions](../language/functions-c.md#c-functions). The VM calls
one through the operation `foreign`, which opens the libs of the program at the first call
(`libloading`) and makes the call with libffi. That needs the platform of the program to
match the host (`--target-bits 64` on a 64-bit host).

- An address that lies in a block reaches C as the native address of that octet, so C
  reads and writes the block in place.
- A text (string) parameter reaches C as a duplicate (copy) of its u8s with a zero octet
  at the end, freed when the call returns.
- A C function with a variable count of arguments (`printf`) gets the extra values as C
  promotes them.
- After each call, the VM flushes the output that C buffers for its streams
  (`fflush(NULL)`), so what C prints comes out in order with what the program prints.

The VM of the Explorer is built with no `ffi`, so it stops the program at the call; see
[Stops in unsafe code](traps-unsafe.md#c-functions).

## Pages of this chapter

- [Running a program](running.md): when a run uses the VM, the command, printing, exit
  codes and limits.
- [Addresses and blocks](blocks.md): the address space, the table of blocks, the life of
  a block, and the two forms of placing variables.
- [When a program stops](traps.md): each trap, what causes it and what it prints.
- [Stops in unsafe code](traps-unsafe.md): the traps at an address and in calls of C.
- [Instructions](instructions.md): the operations, with a program shown as a listing.

The compile-time side of the same checks is in [Compiler checks](../compiler/checks.md),
and [Checks in the VM and in native code](../concepts/checked.md) tells which checks a
native build keeps. [Inside the VM](../internals/vm-internals.md) describes the Rust code
of the VM itself.
