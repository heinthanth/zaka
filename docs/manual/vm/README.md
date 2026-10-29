---
title: The VM
description: What the Zaka VM is made of: its instructions, its calls, its values and its checked storage.
---

# The VM

The Zaka VM, or VM, runs a program that `zakac` has turned into instructions. It is
small on purpose. A loop takes one instruction at a time. A stack of values holds the
working steps of each sentence. A stack of calls remembers where each function is up to.
And a storage of *blocks* holds each variable, so each variable has an address.

Each read and write through an address is checked. A mistake ends the run with a message
that names the instruction and the block, and does not damage the program.

## The parts

```text
 module (from the compiler)            VM (what runs)
 ┌────────────────────────────┐        ┌───────────────────────────────────────────┐
 │ platform bits              │        │ calls, innermost last                     │
 │ globals    g0 g1 ...       │        │  ┌─────────────────────────────────────┐  │
 │ texts      t0 t1 ...       │        │  │ function  next instruction  locals  │  │
 │ functions  f0 f1 ... main  │ ─────▶ │  └─────────────────────────────────────┘  │
 └────────────────────────────┘        │ operand stack     7   0x1040   မှန်       │
                                       │ output            text written so far     │
                                       │ storage                                   │
                                       │  ┌───────┬────────┬──────────────┬──────┐ │
                                       │  │ null  │ static │ stack region │ heap │ │
                                       │  └───────┴────────┴──────────────┴──────┘ │
                                       └───────────────────────────────────────────┘
```

| Part | What it holds |
| --- | --- |
| module | the functions as lists of [instructions](instructions.md), the globals, a pool of print texts, and the platform bits |
| call | one function in progress: the index of its next instruction and the address of each of its locals |
| operand stack | the values a sentence is working on; an instruction pops its inputs and pushes its result |
| storage | the blocks where globals, locals and the heap live; see [blocks](blocks.md) |
| output | the text that print instructions wrote and the command has not taken so far |

The last function of a module is `main`, which runs the sentences outside each function.
The VM starts there, and the run ends when `main` returns or a [trap](traps.md)
stops it.

## Values

A value on the operand stack carries its kind. An instruction reads the kind from its
inputs, so `add` works for each integer width and for decimal numbers alike.

| Value | In storage |
| --- | --- |
| integer, signed or unsigned | one octet for each 8 bits of its width; the plain `ကိန်းပြည့်` has the platform width |
| decimal number, 32 or 64 bits | 4 or 8 octets, in IEEE 754 form |
| character | 4 octets, one Unicode scalar value |
| boolean | 1 octet, `0` or `1` |
| address | one octet for each 8 bits of the platform width; `ဗလာ` is address 0 |

Octets are stored least significant first. A load of octets that hold no value of the
kind, such as a boolean octet of `7`, is a trap (`bad value`).

A list is not one value. It is a run of cells in a block, and the instructions that
touch it work through its address. See [lists](../language/lists.md).

## Each variable has a block

The operand stack is for temporaries and has no addresses. A variable is another matter:
each global and each local lives in storage. So `x ၏ လိပ်စာ` costs nothing more than an
instruction that pushes the address of the block of `x`, and a read through a stale
address finds the freed block of the variable it once was.

A call takes room for its locals at the top of the stack region, and `ret` gives it
back. [Checked and packed blocks](blocks.md) tells how.

## The platform

`--platform-bits` picks the width of `ကိန်းပြည့်`, `လက္ခဏာမဲ့ ကိန်းပြည့်` and each address:
8, 16, 32, 64 (the default) or 128. The storage holds `2^bits` addresses, to at most
`2^64`. On a 16-bit platform, the plain integer wraps at 16 bits:

```console
$ zakac run --platform-bits 16 program.zaka
```

## Pages of this chapter

- [Running a program](running.md): the command, printing, exit codes and limits.
- [Checked and packed blocks](blocks.md): the address space, the table of blocks, the
  life of a block, and the two forms of placing variables.
- [When a program stops](traps.md): each trap, what causes it and what it prints.
- [Instructions](instructions.md): the instruction set, with a program shown as code.

The compile-time side of the same checks is in
[pointer checks](../compiler/checks.md). [Checks in the VM and in native
code](../concepts/checked.md) tells which checks a native build keeps.
