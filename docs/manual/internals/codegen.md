---
title: IR and code
description: How the checked tree is lowered to the IR, how constants are folded, how the IR becomes ops, and how the VM runs them, followed through one program.
---

# IR and code

After the names and kinds are checked, the compiler stops working on the tree. It lowers
the tree to an *IR*, a small program of slots, statements and kinded expressions; folds the
constants in it; lets [the borrow checker](borrow-checker.md) read it; and then turns it
into *ops*, the instructions of a stack machine. The VM, `zaka_vm`, runs the ops.
Two crates hold this: `zakac_ir_builder` (the IR, lowering and folding) and
`zakac_codegen` (the ops and the listing).

```text
 Program + KindTable                       from the name and kind check
        │  lower                           zakac_ir_builder::lower
        ▼
   ir::Module ──────────▶ fold ──▶ ir::Module (folded)         ZK06
                                        │
                                        │   borrow_check(&mut Module)   ZK07
                                        │   (marks proven cell addresses `checked`)
                                        ▼
                                  emit   zakac_codegen::emit
                                        ▼
                                  bc::Module ──▶ listing (asm text)
                                        │
                                        ▼  zaka_vm::machine::Machine
                                  the output of the program
```

Each crate has a `run_phases` that runs the ones before it, so `zakac codegen` is lex,
parse, wording, names and kinds, lower, fold, borrow check, emit. If an earlier stage
reported an error, the later ones are left out and the artifact holds no module; warnings
do not stop the chain.

## The IR

The IR is in `zakac_ir_builder/src/ir.rs`. It keeps what the later stages need and drops
what is not needed: names become numbers, blocks are flattened, operators have one
spelling, and each conversion is written out.

```text
Module { globals: Vec<Slot>, funcs: Vec<Func>, main: Vec<Stmt>, imported_inits, span }
Func   { name, params, result, ret, locals: Vec<Slot>, block: Vec<Stmt>, span }
Slot   { name, kind: DataKind, access: Var | Const | LoopVar, addressed, span }
Place  = Global(n) | Local(n)                   written g0, l0
Expr   { kind: DataKind, node: ExprNode, span }
```

Each declaration has a slot of its own, so a shadowing name never shares one. The
parameters of a function are its first locals, then the result variable `ရလဒ်`, then the
other locals. `addressed` marks a slot whose address the program takes, so it needs an address
that remains fixed.

A statement is one of thirteen forms:

| Form | Meaning |
| --- | --- |
| `Set { place, value }` | a declaration with its first value, or an assignment |
| `Print { value }` | a value or a decoded string, then a line break |
| `If { cond, then, els }` | an `else if` chain nests in `els` |
| `Loop { init, cond, update, block }` | a while loop has a blank `init` and `update` |
| `Counted { var, from, to, step, mode, block }` | `var` runs from `from` to `to`, both included |
| `Break`, `Continue`, `Return { value }` | |
| `Call { func, name, args }` | a call whose result is dropped |
| `Store { addr, value }` | write at an address |
| `Alloc`, `Resize`, `Free` | a heap sentence, with a `Dest`: a slot, or a cell in storage that holds an address |

An expression has its kind on each node, so no later stage has to work it out again.
`ExprNode` has: `Const(Value)`, `Read(Place)`, `Prefix`, `Infix`, `Compare`, `Logic`,
`Convert { implicit }`, `Call`, the pointer forms `AddrOf`, `Load`, `Lend`, `Take`,
`PtrAdd`, `PtrSub`, `SizeOf`, and the list forms `List`, `BlankList`, `CellAddr`. A `Value`
is an integer (kind and bits), a float (width and bit pattern), a char, a boolean or an
address.

### Lowering

`lower(program, kinds, options)` walks the tree once with a `Lowerer`. It reads the kind
of each expression from the `KindTable` (it panics on a missing one, which a clean check
rules out). Its rules:

- Each declaration becomes a new slot, global at top level and local in a function. Scopes
  are kept in a stack of maps just while lowering; the IR has no scopes.
- A verb clause and a notation expression become the same `Infix`, `Compare` or `Prefix`.
- Where the check let a value widen into its place, lowering adds a `Convert` marked
  `implicit`. A written conversion is `implicit: false`.
- A negated integer literal becomes a negative constant, so `-၁၂၈` is a constant of `i8`.
- A function with a result starts with `set l1 = default`, so the result variable has a
  value if the block never sets it.
- `ကိန်းသေ` becomes a slot with `access: const`; the folder uses the flag.
- A pointer read through a place becomes `Load`; an owner moved out becomes `Take`; a
  lend becomes `Lend`.
- A program of several files gives one module. As in the check, each file has its own
  top-level scope and its own function map, and an import brings the places and
  functions it names from the file it reads. Functions are numbered across all the
  files in the order of the tree, and a function of an imported file is named
  `<name>@<file>`, so two private functions of one name differ in the listing.
- The declarations of the imported files come first in the tree, so the `set`s of their
  variables are the first statements of `main`; `imported_inits` counts them.

### Folding

`fold(module, options, platform_bits)` has three foldings, chosen with `--fold` (`all`,
`none`, or a list of `exprs`, `consts` and `branches`):

| Folding | Effect |
| --- | --- |
| `exprs` | an operation on constants is replaced with its value |
| `consts` | a read of a `ကိန်းသေ` set to a constant is replaced with the value |
| `branches` | an `if` on a constant keeps the taken branch; a loop whose condition is the constant false is dropped |

The errors of constant operations are reported whatever `--fold` holds: an overflow
(ZK0601), a zero divisor (ZK0602) and a cell number outside its list (ZK0603), found
with the same `value` functions that the VM uses at run time, so the two cannot
differ. Folding does not remove the `set` of a constant; it just rewrites its reads.

The first `imported_inits` statements of `main` get one more check: a duplicate of them is
folded with each folding on, and each one whose value is then not known (not a constant,
a size, a blank list or a list of known values) is ZK0604. The check uses its own
duplicate, so `--fold` changes the IR but not which programs compile.

Take this program (the file `sq.zaka`):

```text
ကိန်းသေ အလျား သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၂ ဖြစ်ပါစေ။

လုပ်ဆောင်ချက် နှစ်ထပ် သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က * က ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

အကယ်၍ အလျား သည် ၁၀ ထက် ကြီးလျှင် လုပ်ဆောင်ချက် နှစ်ထပ် ကို အလျား တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
```

`zakac irgen --fold none` prints the IR below (spans left out: `lo` and `hi`
pairs on each node). The global is a constant, the function has two locals (`က`, then
`ရလဒ်`) and sets the result variable first:

```text
module:
  globals:
  - name: အလျား
    kind: isize
    access: const
    addressed: false
  funcs:
  - name: နှစ်ထပ်
    params: 1
    result: 1
    ret: isize
    locals:
    - name: က
      kind: isize
      access: var
      addressed: false
    - name: ရလဒ်
      kind: isize
      access: var
      addressed: false
    block:
    - set:
        place: l1
        value:
          kind: isize
          node:
            const: 0:isize
    - return:
        value:
          kind: isize
          node:
            infix:
              op: mul
              lhs:
                kind: isize
                node:
                  read: l0
              rhs:
                kind: isize
                node:
                  read: l0
```

The top-level part, `main`, comes out like this with `--fold none`:

```text
  main:
  - set:
      place: g0
      value:
        kind: isize
        node:
          const: 12:isize
  - if:
      cond:
        kind: bool
        node:
          compare:
            op: gt
            lhs:
              kind: isize
              node:
                read: g0
            rhs:
              kind: isize
              node:
                const: 10:isize
      then:
      - print:
          value:
            expr:
              kind: isize
              node:
                call:
                  func: 0
                  name: နှစ်ထပ်
                  args:
                  - kind: isize
                    node:
                      read: g0
      els:
      []
```

and like this with the default `--fold all`: `read g0` is the constant `12`, so the
condition `12 > 10` is true, so the `if` is gone and its branch stands in its place.

```text
  main:
  - set:
      place: g0
      value:
        kind: isize
        node:
          const: 12:isize
  - print:
      value:
        expr:
          kind: isize
          node:
            call:
              func: 0
              name: နှစ်ထပ်
              args:
              - kind: isize
                node:
                  const: 12:isize
```

## The ops

`zakac_codegen::emit(module, platform_bits, results)` turns the folded IR into a
`bc::Module`:

```text
Module { platform_bits, globals: Vec<Slot>, texts: Vec<String>, funcs: Vec<Func>, main: u32 }
Func   { name, params, ret, locals: Vec<Slot>, code: Vec<Op>, spans: Vec<Span> }
```

`funcs` holds the IR functions at their IR numbers, then one more function, `main`, for the
top-level statements; `main` is its number. `texts` is the pool of print strings. `spans`
runs parallel to `code` and tells the source region of each op, which is how the VM
and the explorer point at the line being run. The generator can add locals of its own,
named with a `%`, such as `%to`, `%step` and `%dir` for a counted loop.

The VM is a stack machine. Ops that work on values take the kind from the values on
the stack, so most ops name no kind. The ones that need a kind name it: `convert`, `load`,
`store`, `take`, and the list ops. The groups:

| Group | Ops |
| --- | --- |
| values | `push v`, `pop`, `dup` |
| storage | `local_addr n`, `global_addr n`, `load k`, `store k`, `take k` |
| arithmetic | `prefix op`, `infix op`, `compare op`, `convert k` (the ops print as `add`, `lt`, ...) |
| control | `jump t`, `jump_if_false t`, `jump_if_true t`, `call f`, `ret`, `result_addr`, `trap code` |
| output | `print`, `print_text t`, `print_list k` |
| pointers | `ptr_add stride`, `ptr_sub stride`, `alloc stride`, `realloc stride`, `free` |
| lists | `index len from`, `blit k`, `zero k`, `list_eq k` |

Each variable lives in storage. A read is `local_addr n` or `global_addr n`, then `load`;
a write is the address, the value, then `store`. That is the reason each access of a
variable can be checked: it goes through an address of a block, as a heap pointer does.

`CodeBuilder` (`code.rs`) collects the ops of a function. A jump names a `Label`, and the
labels are bound to op numbers when their places are reached; at the end the jumps are
patched with the numbers (`Unbound` is an error for a label that was named and never
bound).

### Statement and expression rules

| IR | Ops |
| --- | --- |
| `and` | `<l>; jump_if_false F; <r>; jump E; F: push false; E:` |
| `or` | `<l>; jump_if_true T; <r>; jump E; T: push true; E:` |
| `if` | `<cond>; jump_if_false Else; <then>; jump End; Else: <els>; End:`; the first `jump` is left out when `els` is blank or `then` ends in a jump |
| while, generic loop | `<init>; Head: <cond>; jump_if_false Exit; <block>; Next: <update>; jump Head; Exit:`; `continue` goes to `Next`, `break` to `Exit` |
| call statement | `<args>; call f`, then `pop` if the callee has a result |
| function | its block, then `ret` unless the block ends in one |

A list or struct result travels through a slot of the caller. Each call of such a
function gets a `%ret` local: the caller pushes `local_addr` of it before the arguments,
and the call leaves that address on the stack as its value. Inside the function,
`result_addr` pushes the same address. `ret` takes the address the function gives back
and, unless it is the slot itself, duplicates the value into the slot before the frame
goes. A list or struct local given back is not moved out first (no `take`), since its
frame ends with the return.

`results` (`ResultMode`, the `--results` option) picks what lives in the slot:

| Mode | Ops |
| --- | --- |
| `in-place` (default) | the local that each `return` gives back, often the result variable, is read and written through `result_addr`; a returned literal is built at `result_addr`, and a returned call passes `result_addr` on as its slot, so `ret` duplicates nothing |
| `duplicate` | the function keeps the value in its own frame, and `ret` duplicates it into the slot |

A counted loop evaluates `from`, `to` and `step` once, in that order. A constant bound
or step is pushed again where needed; a value that is not constant goes into a `%to` or
`%step` local. A step that is not a constant is checked once, and the program stops with
`step_not_positive` if it is zero or less. The variable is never moved past `to`, so it
cannot wrap. After each round:

```text
counting up:    if (to as U) - (var as U) < (step as U) then leave, else var += step
counting down:  if (var as U) - (to as U) < (step as U) then leave, else var -= step
```

`U` is the unsigned kind of the same width as the variable. The difference is never
negative, so it fits `U` even for the range of `i8`. With both bounds constant the
direction is chosen while compiling. Otherwise a `%dir` local holds it, and the two
directions share one run of the block.

### One program, as ops

`zakac codegen --asm` writes the listing. For `sq.zaka` above:

```text
; zakac asm listing v1, platform 64 bits
globals
  g0 အလျား isize

fn f0 နှစ်ထပ် (params 1) -> isize
locals
  l0 က isize
  l1 ရလဒ် isize
     0  local_addr l1        ; ရလဒ်
     1  push 0:isize
     2  store isize
     3  local_addr l0        ; က
     4  load isize
     5  local_addr l0        ; က
     6  load isize
     7  mul
     8  ret

fn f1 main (params 0)
     0  global_addr g0       ; အလျား
     1  push 12:isize
     2  store isize
     3  push 12:isize
     4  call f0              ; နှစ်ထပ်
     5  print
     6  ret
```

In `f0`, ops 0 to 2 set the result variable to zero, ops 3 to 7 read `က` twice and
mul them, and `ret` leaves the product on the stack, which is how a result travels. In
`main`, the global gets its value (ops 0 to 2), the argument `12` is pushed (op 3; the
folder turned `read g0` into a constant), `call f0` takes the argument from the stack
into the local `က` and leaves the result, and `print` writes it. The program prints `၁၄၄`.

A counted loop shows the labels. For `၁ မှ ၃ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသပါ။`:

```text
fn f0 main (params 0)
     0  global_addr g0       ; ကိန်း
     1  push 1:isize
     2  store isize
     3  global_addr g0       ; ကိန်း
     4  load isize
     5  push 3:isize
     6  gt
     7  jump_if_true L1
L0:
     8  global_addr g0       ; ကိန်း
     9  load isize
    10  print
    11  push 3:usize
    12  global_addr g0       ; ကိန်း
    13  load isize
    14  convert usize
    15  sub
    16  push 1:usize
    17  lt
    18  jump_if_true L1
    19  global_addr g0       ; ကိန်း
    20  global_addr g0       ; ကိန်း
    21  load isize
    22  push 1:isize
    23  add
    24  store isize
    25  jump L0
L1:
    26  ret
```

Ops 3 to 7 are the one test before the first round (a start past the end runs zero
rounds). Ops 8 to 10 are the block. Ops 11 to 18 are the test of the formula above, with
`U` = `usize`. Ops 19 to 25 add the step and go round. The loop variable is `ကိန်း`, a
global here because the loop is at top level.

## The artifacts

`zakac irgen` writes the folded IR (`.zkirt` text, `.zkir` raw), `zakac borrow-check`
the IR and the digests, and `zakac codegen` the ops (`.zkbct`, `.zkbc`) or, with `--asm`,
the listing (`.zkasm`). Text artifacts start with a header line such as
`# zakac-codegen-artifact v1` and are the form of the golden files; the raw form is the
`ZKAR` envelope with Postcard inside. `zakac run` takes Zaka source, which it compiles,
or a codegen artifact in either form, which it knows from the first octets.

## The VM

`zaka_vm::machine::Machine` owns a `bc::Module` and runs it one op at a time: `step()`
fetches the op at the program counter of the innermost call, executes it with `exec`, and
returns a `State`: `Running`, `Finished` or `Trapped`. `run` and `run_with` loop `step`;
the explorer calls `step`, `step_over` and `step_out` to show a program running.

- **Start.** `Machine::new` gives each global a static block, opens the stack region and
  enters `main`. Globals start at the default of their kind, and the `Set` statements that
  the IR keeps run in source order.
- **A call.** `call f` takes one block of the stack region for each local of `f`
  (`FrameMode::Checked`), or one block for all of them at C offsets (`Packed`), moves the
  arguments from the operand stack into the first locals, and pushes a `Frame`. `ret`
  pops the frame and gives the blocks back; for a list or struct result it first
  duplicates the value into the caller's slot, the value under the arguments. A depth past `max_depth` is the trap
  `StackOverflow`.
- **Storage.** `Mem` is the address space: blocks of three origins (`Heap`, `Stack`,
  `Static`), each with a red zone after it, so an access just past a block never lands in
  the next one. `load`, `store` and `take` check that the address is inside a live
  block, and trap if not: a null address, a freed block, past the end.
- **Values.** The operand stack holds `Value`s, so arithmetic wraps to the width of the
  kind, a zero divisor is the trap `ZeroDivisor`, and a shift past the width is
  `ShiftTooFar`. The same `value` functions did the folding.
- **Limits.** `RunOptions` has `max_depth`, `max_steps`, `max_heap`, `max_stack`, the
  frame mode, and `leaks`. A trap is not a `ZK` diagnostic, since it belongs to the run of
  a program and not to the compiler.

The pages [Running a program](../vm/running.md), [Instructions](../vm/instructions.md),
[Checked and packed blocks](../vm/blocks.md) and [When a program stops](../vm/traps.md)
tell the VM from the side of its user.

## Tests and extending

- `zakac_ir_builder/tests`: `lower.rs`, `fold.rs`, `value.rs` (operations on values),
  `lists.rs`, `moves.rs`, and golden `.zkirt` files in `expected/`.
- `zakac_codegen/tests`: `code.rs` (label patching), `emit.rs` (the ops of each
  construct), `listing.rs`, `artifact.rs`, and `samples.rs`, which holds golden
  `.zkbct` and `.zkasm` files and checks that each op of each sample is well formed
  (indices in range, each function ending in `ret`).
- `zaka_vm/tests`: `machine.rs` (hand-built modules, malformed ops included, which must
  trap and not panic), `traps.rs`, `heap.rs`, `stepping.rs`, and `samples.rs`, which runs
  each program in `samples/` outside `diagnostics` and `mistakes` and compares its output
  with `tests/expected/<group>/<name>.out`.

To add an IR form: a variant in `ir.rs`, a rule in `lower.rs`, a case in `fold.rs` and in
`walk.rs` of the borrow checker, a rule in `emit.rs`, and an op if the VM needs one
(`bc.rs`, `listing.rs`, `machine.rs`). To add an op: `bc::Op`, its operand text in
`Op::operand`, the arm of `Machine::exec`, and a test of the op on a hand-built module.
