---
title: Instructions
description: The instructions of the Zaka machine in groups, with a program shown beside its code.
---

# Instructions

The code generator turns a program into a list of instructions for each function, and
the machine runs them one at a time. This page lists the instruction set and shows a
small program next to its instructions.

The instruction set belongs to the compiler and the machine as a pair, and it can change
between versions. A listing made with one version of `zakac` is for that version. The
rules of the language do not depend on it: what a program means is set in
[the language](../language/README.md), and the instructions are one means to run it.

## Reading a listing

`zakac codegen --asm` writes the listing of a program instead of the artifact. Here is a
function that doubles its parameter, and a `main` that calls it:

```zaka
လုပ်ဆောင်ချက် နှစ်ဆ သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
ရလဒ် ကို န * ၂ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၁ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် နှစ်ဆ ကို က တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
```

```text
၄၂
```

```console
$ zakac codegen --asm program.zaka
; zakac asm listing v1, platform 64 bits
globals
  g0 က isize

fn f0 နှစ်ဆ (params 1) -> isize
locals
  l0 န isize
  l1 ရလဒ် isize
     0  local_addr l1        ; ရလဒ်
     1  push 0:isize
     2  store isize
     3  local_addr l1        ; ရလဒ်
     4  local_addr l0        ; န
     5  load isize
     6  push 2:isize
     7  mul
     8  store isize
     9  local_addr l1        ; ရလဒ်
    10  load isize
    11  ret

fn f1 main (params 0)
     0  global_addr g0       ; က
     1  push 21:isize
     2  store isize
     3  global_addr g0       ; က
     4  load isize
     5  call f0              ; နှစ်ဆ
     6  print
     7  ret
```

How to read it:

- `g0`, `l1`, `f0`, `t0` are the index of a global, a local, a function and a pooled
  text. After `;` comes the source name, for the reader. A jump shows a label, `L0`, in
  the listing; the artifact holds the index of the target instruction.
- A function lists its locals, parameters first (`l0` is the parameter `န`), then the
  locals of the source, then slots the generator adds, such as `%list`, `%to`, `%step`
  and `%dir`. The last function is `main`, which holds the sentences outside each
  function.
- A kind is written short: `isize` and `usize` for the platform-sized integers, `u8`,
  `i32` and so on, `f64`, `char`, `bool`, `*isize` for an address of an integer,
  `[3]isize` for a list of three integers. A value is written `21:isize`.
- A variable has no instruction of its own. Reading `က` is `global_addr g0` to push the
  address of its block, then `load isize` to read it. Writing is the address, the value,
  then `store`. This is what gives each variable an address.
- The local `ရလဒ်` is set to its default, `0`, at the start, and `ret` leaves its value
  on the operand stack for the caller.
- The call `call f0` takes its argument from the operand stack into the parameter, and
  the result comes back on the operand stack. `print` pops it.

The artifact of `zakac codegen` holds the same instructions as data: raw (`.zkbc`) or
as text (`.zkbct`, a header line and then YAML), chosen with the file name or with
`--output-format`. `zakac run` runs either form.

## The instructions

The tables use `→` for the effect on the operand stack: the inputs, with the top
rightmost, then the outputs. `k` is a kind, and `s` the size of one element in octets.

### Values and addresses

| Instruction | Effect | Meaning |
| --- | --- | --- |
| `push v` | → v | push a constant |
| `local_addr n` | → addr | the address of the block of local `n` of the running call |
| `global_addr n` | → addr | the address of the block of global `n` |
| `dup` | v → v v | push the top value again |
| `pop` | v → | drop the top value |

### Operators

| Instruction | Effect | Meaning |
| --- | --- | --- |
| `neg`, `bnot`, `not` | a → r | minus, bitwise flip, logical not |
| `add`, `sub`, `mul`, `div`, `rem` | a b → r | arithmetic; integers wrap to their width |
| `band`, `bor`, `bxor` | a b → r | bitwise and, or, exclusive or |
| `shl`, `shr` | a n → r | shifts |
| `eq`, `ne`, `lt`, `gt`, `le`, `ge` | a b → bool | comparisons |
| `convert k` | a → a as k | convert a value to kind `k` |

`div` and `rem` trap on a zero divisor; `shl` and `shr` trap when the distance is not
below the bit width. The right operand is on top of the stack, so `sub` computes the
value below the top minus the top. Decimal numbers follow IEEE 754 with ties to even.

### Storage

| Instruction | Effect | Meaning |
| --- | --- | --- |
| `load k` | addr → v | read a value of kind `k` |
| `store k` | addr v → | write a value of kind `k` |
| `take k` | addr → v | read the value, then leave the default of the kind there (`ဗလာ` for an owner): a move |
| `ptr_add s` | addr n → addr | move an address forward `n` elements of `s` octets |
| `ptr_sub s` | a b → n | the number of elements between two addresses, an `isize` |
| `alloc s` | n → addr | a new heap block of `n` elements, or `ဗလာ` when there is no room |
| `realloc s` | addr n → addr | resize a heap block; the block can move |
| `free` | addr → | give a heap block back; `ဗလာ` does nothing |

Each of these checks its address as described in [blocks](blocks.md), and can trap.
`ptr_add` also traps on a step off `ဗလာ` and on a step out of the block it starts in.

### Lists

A list lives in a block, and its instructions work on addresses.

| Instruction | Effect | Meaning |
| --- | --- | --- |
| `index 3`, `index 3 from 1` | n → i | check a cell number against a list of 3 cells, whose first cell is numbered 0 (or 1, after `from`); push it counted from 0 |
| `blit k` | to from → | duplicate a value of kind `k`, such as a whole list, from one address to another |
| `zero k` | addr → | set the octets of a value of kind `k` to zero |
| `list_eq k` | a b → bool | whether each cell of one list equals the cell of the other |

### Control

| Instruction | Effect | Meaning |
| --- | --- | --- |
| `jump t` | | go to instruction `t` of the function |
| `jump_if_false t` | bool → | go to `t` when the boolean is false |
| `jump_if_true t` | bool → | go to `t` when the boolean is true |
| `call f` | args → result | call function `f`; the last argument is on top |
| `ret` | | leave the function; a result remains on the stack |
| `trap step_not_positive` | | stop the program: a counted loop step is zero or negative |

`call` takes room for the locals of the callee in the stack region and moves the
arguments into its parameters; `ret` gives the room back. A call nested deeper than
`--max-depth` traps.

### Output

| Instruction | Effect | Meaning |
| --- | --- | --- |
| `print` | v → | write the value and a line break |
| `print_text t` | | write the pooled text `t` and a line break |
| `print_list k` | addr → | write the cells of a list of kind `k` and a line break |

How each value reads is in [Running a program](running.md#printing).

## How sentences become instructions

| Sentence | Instructions |
| --- | --- |
| read a variable | `x; load k` |
| `x ကို v ဟု ထားပါ` | `x; v; store k` |
| address of a variable | `x` |
| `a နှင့် b ၏ ပေါင်းလဒ်` | `a; b; add` |
| `v ကို ပြသပါ` | `v; print` |
| call as a statement | `args; call f`, and `pop` when the callee gives a result no one uses |
| `p ၏ တန်ဖိုး` | `p; load k` |
| `p + i` | `p; i; ptr_add s` |
| allocate | `x; count; alloc s; store *k` |
| resize | `x; x; load *k; count; realloc s; store *k` |
| free | `x; x; load *k; free; push ဗလာ; store *k` |
| move an owner | `to; from; take *k; store *k` |
| `a ဖြစ်၍ b` (and) | `a; jump_if_false F; b; jump E; F: push မမှန်; E:` |
| `a သို့မဟုတ် b` (or) | `a; jump_if_true T; b; jump E; T: push မှန်; E:` |
| if and else | `cond; jump_if_false Else; then; jump End; Else: else; End:` |
| a loop | `Head: cond; jump_if_false Exit; block; jump Head; Exit:` |

Here `x` stands for `local_addr n` or `global_addr n`.

A counted loop evaluates its two ends and its step once, in that order, into slots when
those are not constants. It never moves the counter past the last value, so a counter
cannot wrap. A step that is not a constant is checked once, before the first round, with
`trap step_not_positive`.

### A heap block in instructions

A block from the heap, written to, read and given back (see [the
heap](../../learn/pointers/heap.md)), and the instructions of `main`:

```zaka
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ။
ပ + ၁ ၏ တန်ဖိုး ကို ၉ ဟု ထားပါ။
ပ + ၁ ၏ တန်ဖိုး ကို ပြသပါ။
ပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
၉
```

```console
fn f0 main (params 0)
     0  global_addr g0       ; ပ
     1  push 0x0:ptr
     2  store *isize         ; ပ starts as ဗလာ
     3  global_addr g0       ; ပ
     4  push 2:usize
     5  alloc 8              ; 2 elements of 8 octets
     6  store *isize         ; keep the address in ပ
     7  global_addr g0       ; ပ
     8  load *isize
     9  push 7:isize
    10  store isize          ; ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ
    11  global_addr g0       ; ပ
    12  load *isize
    13  push 1:isize
    14  ptr_add 8            ; ပ + ၁
    15  push 9:isize
    16  store isize
    17  global_addr g0       ; ပ
    18  load *isize
    19  push 1:isize
    20  ptr_add 8
    21  load isize
    22  print
    23  global_addr g0       ; ပ
    24  global_addr g0       ; ပ
    25  load *isize
    26  free
    27  push 0x0:ptr
    28  store *isize         ; the free statement clears ပ
    29  ret
```

The comments after `;` in this listing were added to explain it; `zakac` prints the
variable names alone. Two instructions name a kind that the others leave out: `load` and
`store` name the kind read or written, and `alloc` names the octet size of one
element, so the machine can check the access and the cap.

## Where to look next

- [When a program stops](traps.md) tells what each instruction does when its check fails.
- [IR and code](../internals/codegen.md) tells how the code generator makes these
  instructions.
