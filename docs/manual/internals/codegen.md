---
title: The IR
description: How the checked tree is lowered to the IR, the statements and expressions of the IR, and how constants are folded, followed through one program.
---

# The IR

After the names and data kinds (types) are checked, the compiler stops working on the
tree. It lowers the tree to an *IR* (intermediate representation), a small program of
slots, statements and expressions that each have their data kind, and then folds the
constants in it. [The flow checker](borrow-checker.md) reads the folded IR next, and each
backend turns it into code: [the VM target](vm-target.md) into operations
(instructions) for the VM, [the C target](c-backend.md) into C, and [the native
target](native-backend.md) into Cranelift IR. The crate `zakac_ir_builder` contains the
IR, the lowering and the folding.

```text
 Program + KindTable                       from names and data kinds
        │  lower                           zakac_ir_builder::lower
        ▼
   ir::Module ──────────▶ fold ──▶ ir::Module (folded)         ZK06
                                        │
                                        │   borrow_check(&mut Module)   ZK07
                                        │   (marks proven cell addresses `checked`)
                                        ▼
                     zakac_codegen_target_vm::emit, or the C or native target
```

Each crate has a `run_phases` that runs the ones before it, so `zakac build` is lex,
parse, wording, names and data kinds, lower, fold, flow check and emit. When an earlier stage
reported an error, the later ones are left out and the artifact contains no module, while
warnings do not stop the chain.

## An example

Take this program, the file `sq.zaka`:

```text
ကိန်းသေ အလျား သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၂ ဖြစ်ပါစေ။

နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ အလျား သည် ၁၀ ထက် ကြီးလျှင် နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက်ကို အလျား ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

`zakac irgen --fold none` prints the IR below, with the spans (the `lo` and `hi` pairs on
each node) left out. The global is a constant, and the function has one local, its
parameter `က`:

```text
module:
  globals:
  - name: အလျား
    kind: isize
    access: const
    addressed: false
    unset: null
  funcs:
  - name: နှစ်ထပ်
    params: 1
    result: null
    ret: isize
    locals:
    - name: က
      kind: isize
      access: var
      addressed: false
      unset: null
    block:
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
    shared_params: []
    owned:
    - false
    owned_ret: false
    c_name: null
```

The program start block (entry point) is function 1, and with `--fold none` it comes out
like this:

```text
  - name: ပရိုဂရမ်
    params: 0
    result: null
    ret: null
    locals: []
    block:
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

`main` sets the constant and calls the program start block:

```text
  main:
  - set:
      place: g0
      value:
        kind: isize
        node:
          const: 12:isize
  - call:
      func: 1
      name: ပရိုဂရမ်
      args: []
```

With the default `--fold all`, `read g0` becomes the constant `12`, so the condition
`12 > 10` is true, and the `if` is gone with its branch in its place:

```text
  - name: ပရိုဂရမ်
    params: 0
    result: null
    ret: null
    locals: []
    block:
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

[The VM target](vm-target.md#an-example) continues this example as operations.

## Modules, functions and slots

The IR is in `zakac_ir_builder/src/ir.rs`. It keeps what the later stages need and drops
what those stages do not: names become numbers, blocks are flattened, operators have one
spelling, and each conversion (cast) is written out.

```text
Module { globals: Vec<Slot>, funcs: Vec<Func>, main: Vec<Stmt>, imported_inits, host,
         foreign: Vec<ForeignFn>, unsafe_spans, c_structs, threads_structs, built_for, span }
Func   { name, params, result, ret, locals: Vec<Slot>, block: Vec<Stmt>, shared_params,
         owned, owned_ret, c_name, span }
Slot   { name, kind: DataKind, access: Var | Const | LoopVar, span, addressed, unset }
Place  = Global(n) | Local(n)                   written g0, l0
Expr   { kind: DataKind, node: ExprNode, span }
```

Each declaration has its own slot, so a shadowing name never shares one. The parameters
of a function are its first locals, then come the other locals. An external function has
a result variable `ရလဒ်` after its parameters (a slot the generator adds, which no
program can name), where the host or C puts what it returns. `unset` keeps the span of a
declaration that wrote no value, which the flow checker reads. `addressed` marks a slot
whose address the program takes, so it needs an address that does not move.

## Statements

A statement is one of these forms:

| Form | Meaning |
| --- | --- |
| `Set { place, value }` | a declaration with its first value, or an assignment |
| `Print { value }` | a value (`PrintValue::Expr`) or static text (`PrintValue::Text`), then a line break |
| `If { cond, then, els }` | an `else if` chain nests in `els` |
| `Loop { init, cond, update, block }` | a while loop has a blank `init` and `update` |
| `Counted { var, from, to, step, mode, block }` | `var` runs from `from` to `to`, both included |
| `Break`, `Continue`, `Return { value }` | |
| `Call { func, name, args }` | a call whose result is dropped |
| `Foreign { func, args }` | a call of a C function with no result |
| `CallValue { callee, args }` | a call through the function value `callee`, its result dropped; written `(call_value f 1)` |
| `Store { addr, value }` | a write at an address |
| `Alloc`, `Resize`, `Free` | a heap sentence, with a `Dest`: a slot, or a cell in storage that contains an address |
| `CStore { base, offset, value }` | a write of a field of a struct that C keeps |
| `Leave { value, grant, span }` | give C leave to call the function value (`grant`), or take it back; the lowering spills the value to a hidden slot unless it is a place; written `(leave f)` and `(revoke f)` |
| `Blit { src, dst, count, unit, overlap }` | write `count` cells of `unit` from the address `src` to the address `dst`; `overlap` allows the ranges to share cells; written `(blit ...)`, and `(shift ...)` when `overlap` is set |
| `Fill { dst, count, value }` | set `count` cells at the address `dst` to `value`; written `(fill ...)` |
| `Wait { thread, kind, span }` | a wait for a thread whose result is dropped |
| `Send { pipe, value, span }`, `Close { pipe, span }` | send a value into a channel, or close it |
| `Select { pipes, slots, ms, idle, chosen }` | a wait on several channels: the value goes to the optional of its channel in `slots`, and `chosen` gets the place of that channel, the count of channels for the time limit or no data, or that count plus one once each is closed and drained; an `if` chain on `chosen` follows, where the closed arm tests the count plus one and, with no closed arm, the arm of no data takes both; written `(select ...)` |
| `Lock`, `Unlock` | take and release the lock of a shared object, around a lock block |
| `AtomicAdd { addr, value, sub }`, `AtomicSet` | add to (or subtract from) an atomic number, or set an atomic field, in one step |
| `Rings` | work of the ring collector: run it, pause it, resume it or set its threshold |
| `Halt { cause, value }` | stop the run (see below) |
| `Drop { dest, kind }` | let go of each counted block inside the value at `dest` (text, a plain list, a box, a closure record), of a data kind that contains one, and leave it blank: the last holder frees the block; written `(drop l0)` |
| `AddCell { dest, kind, value }` | add `value` as a new last cell of the plain list at `dest`; written `(add_cell l0 7)` |
| `RemoveCell { dest, kind }` | remove the last cell of the plain list at `dest`, or the last character of the text there; written `(remove_cell l0)` |
| `SetChar { dest, kind, index, first, value }` | write the character `value` in place of character `index` of the text at `dest`, counted from `first` (0 or 1); written `(set_char g0 2 'x')` |
| `AddText { dest, kind, value }` | add the text or character `value` at the end of the text at `dest`; written `(add_text g0 "c")` |
| `Replace { dest, kind, start, end, chars, value }` | put `value` in place of the octet cells (from 0) or, with `chars`, the characters (from 1) from `start` to `end` of the text at `dest`, both included; written `(replace l0 0 4 "x")` or `(replace_chars …)` |
| `Insert { dest, kind, index, first, value }` | put `value` in front of cell `index` of the plain list at `dest`, or in front of character `index` of the text there, counted from `first`; written `(insert l0 1 20)` |
| `SetRow`, `RemoveRow` | set or remove a row of a ဇယား (see ဇယား values, below) |

`Halt` stops the run for a read of the wrong part of a whole result or of an optional
with no data, and for a few other stops that the lowering writes out. Its `cause` is a
`HaltCause`, and `value` is the error value to show, when there is one:

| Cause | When |
| --- | --- |
| `not_good` | the good result of a result that contains an error was read; written `(not_good v)`, or `(not_good)` for a failure with no value, where the trap shows the bare message |
| `not_fault` | the error value of a good result was read; written `(not_fault)` |
| `no_data` (`HaltCause::NoData`) | the data of an optional that contains none was read; written `(no_data)` |
| `ClosedHandle`, `NullFromC` | a closed handle, or NULL from C for a plain handle result (see [C functions](vm-internals.md#c-functions)) |
| `NoRow` | a write inside a row of a ဇယား that has no row for the index |
| `StepNotPositive`, `ThreadsNotPositive` | a step of a parallel loop, or its count of threads, that is zero or less |

## Expressions

An expression has its data kind on each node, so no later stage has to work it out
again. `ExprNode` has these forms:

| Group | Forms |
| --- | --- |
| values and operators | `Const(Value)`, `Read(Place)`, `Prefix`, `Infix`, `Compare`, `Logic`, `Convert { implicit }` |
| calls | `Call`; `CallValue { callee, args }` (a call through a function value: the value first, then the arguments); `Host { func, args }`; `Foreign { func, args }` (the one expression of a C function with a result) |
| function values | `FnAddr { func, name }` (the function value of `func`, written `(fn_addr name)`); `Closure { func, name, record }` |
| threads | `Spawn { func, name, args }` (a call that starts a [thread](../language/threads.md), whose value is the `usize` that names it); `Wait(thread)` (a wait for a thread that returns its result); `MakePipe(room)` (a new channel, the initial value of a channel kind); `Receive(pipe)` (the next value of a channel, as an optional) |
| atomic numbers | `AtomicLoad`, `AtomicCas` |
| pointers | `AddrOf`, `Load`, `CField`, `Lend`, `Take`, `Dup`, `Own`, `PtrAdd`, `PtrSub`, `SizeOf` |
| lists | `List`, `Repeat { cell, count }`, `BlankList`, `CellAddr`, `FieldAddr`, `MakeSlice` |
| text | `Text(String)`, `Format(Vec<FmtPiece>)`, `Chars { text, index, first }` |
| other | `RingStat`, `RowFind` |

Some of these need a word more:

- `Closure { func, name, record }` is the closure of a nested function with captures,
  made where the function is declared: `func` is its thunk, and `record` the address of
  the record in a hidden slot, which moves to a counted heap block.
- `Host { func, args }` is the one statement of an external function: its result variable
  receives what the host function `func` returns for the address of each parameter,
  written `(host read_file (addr l0))`. The IR builder makes it in `host_statement`, and
  codegen emits the operation `host` into the result slot.
- `Lend` is an address that the IR makes itself, under the borrow mark that the flow
  checker reads. `Take` is a value moved out, with the place left at its default. `Dup`
  is a duplicate (copy), and `deep` marks a deep one. `Own` is a plain list made to own
  its block before a write, written `(own ...)`, and `pin` pins the block too.
- `Repeat { cell, count }` is a counted list whose cells each contain one value, written
  `(repeat 100 ...)`.
- `MakeSlice` is the inner view that a range reads its cells through.
- A `CellAddr` names the count of its list, or none for a cell of a plain list or a
  view, whose count is read when the program runs.
- `Text(String)` is static text, written `"..."` in the s-expression form, and
  `Format(Vec<FmtPiece>)` is text built when the program runs, written `(format "a" g0)`,
  whose pieces are `FmtPiece::Text(String)` or `FmtPiece::Value(Expr)`.
- `Chars { text, index, first }` is the character count of text (`(chars g0)`, no index)
  or the character at a number (`(char@ g0 i)` from 0, `(char1@ g0 i)` from 1).

A `Value` is an integer (data kind and bits), a float (width and bit pattern), a char, a
true/false value or an address.

## Lowering

`lower(program, kinds, structs, options)` walks the tree once with a `Lowerer`. It reads
the data kind of each expression from the `KindTable`, and it panics on a missing one,
which a clean check rules out. The rules follow, grouped under topics.

### Names, operators and conversions

- Each declaration becomes a new slot, global at the top level and local in a function.
  Scopes live in a stack of maps while lowering runs, and the IR has no scopes.
- A verb clause and a notation expression become the same `Infix`, `Compare` or
  `Prefix`.
- Where the check let a value widen into its place, lowering adds a `Convert` marked
  `implicit`. A written conversion is `implicit: false`.
- A negated integer literal becomes a negative constant, so `-၁၂၈` is a constant of `i8`.
- `ကိန်းသေ` becomes a slot with `access: const`, and the folder uses the flag.

### Calls and function values

A call runs a declared function of its name, as the check decides, or else the function
value of the name or of its place (`call_target`), so it becomes a `Call` or a
`CallValue`. The arguments of a call through a value follow `fn_shape`: each parameter in
its own data kind, or the address of the place for one marked `ဒေတာ ပြောင်းလဲရန်`, the
same as a declared function places its parameters. `NAME ဟူသော လုပ်ဆောင်ချက်` becomes
`FnAddr`, which the folder turns into the constant `f + 1`, so a constant list or an
imported global can contain it. A template slot keeps such a constant as a value, which
prints as the name of its function.

### Nested functions and closures

A function declared inside a function (`lower/nested.rs`) is lifted into its own function
of the module, numbered after the top-level ones and named `<name>#<index>` (with the file
suffix). Each local it names from around it (`FnDecl::captures`, which the check wrote)
lives in a *box*: a counted plain list of one cell (`DataKind::box_of`), made where the
local is declared. Each read and write of the local, in the function around it and inside
the nested one, goes through the address of cell 0 (`edit_addr`). A box is never lent and
never static. The lifted function receives each box after its declared parameters, with
a retain (`Sig::captures`), and a closure among the captures is kept as a value under the
name `<name>%`.

Where a function with captures is declared, `make_closure` fills a record in a hidden
slot, a struct of the data kind `DataKind::closure_record` (`%closure{code, info, c0,
...}`): the code and info words for the backend, then each box or captured closure.
`Closure { func, name, record }` moves the record to a counted heap block and gives the
function value, which a hidden local `<name>%` keeps and drops as its block ends. `func` is
the thunk `<name>#record<index>`, which receives the declared parameters and the record
address and calls the lifted function with each field.

A call of the function through its name, and its value, read that local
(`through_closure`, `closure_value`), and a call of itself inside itself passes its own
boxes on. A duplicate of a function value is a retain of the record, and a function value
argument is a `Dup { lent: true }` (`lend_fn`). A function value that a call makes fresh
waits in a hidden slot that its scope drops (`hold_fresh_fn`).

Each round of a loop gives a captured local declared in its block its own box. A captured
counter of a counted loop, or the variable of an each loop, is copied into a new box at
the start of each round (`rebox`, `counted_block`), so each closure made in a round keeps
the value of its round. The variable of a generic loop is one box for the whole loop.

### Duplicates and shared objects

- `ကူးထည့်မည်` and `X ၏ မိတ္တူ` on a shared object (reference type) build a new object from
  cell 0 of the box (`object_fields`), and on each other value each one is a plain
  duplicate.
- `အကုန်ကူးထည့်မည်` and `X တစ်<classifier>လုံး ၏ မိတ္တူ` lower to `Dup { deep: true }`, which
  duplicates each object it reaches once and shares function values. A deep `Dup` is
  never turned into a move and never lent.
- The test of one object (`same`) compares two function values as words, and two objects
  through the first words of their boxes.
- A field of a shared object reads cell 0 of its box first, and a write to it never
  duplicates first, since each name of the object sees the one box.

### Returns, results and optionals

A function with a result returns each value at its return. A return that must drop text
first puts its value in its own hidden slot, drops, then returns the slot. The block of a
function with a result returns on each path, as the check of ZK0339 makes sure, so no path
falls off its end. A block that does not end in a return gets an unreached return of the
zero value of the data kind, since the targets want one there.

A function that can fail returns a compiler-built struct `%fallible{tag: u8, ok: T, err:
E}`, with no `ok` field when the good side has no value and no `err` field when the error
side has none. Tag 1 is a good result and tag 0 a failure, so the blank (all-zero) struct
is a failure. The statements `လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်` and `လုပ်ဆောင်ချက်
မအောင်မြင်ပါဟု သတ်မှတ်မည်` are returns of a struct with the tag alone. Each return builds
the struct in a hidden slot and returns it, after the drops of the places that contain
text.

A call that passes a failure on (error propagation) goes into a hidden slot, and
statements that run before the statement around it test its tag: on a failure, the
function drops its text and returns the error. Each operand to the left of the call is
worked out first into a hidden slot (a duplicate for text and lists, while a plain read of
a pointer is left in place), so the written order holds. The right side of `&&` and `||`,
the condition of a while loop and that of an `else if` arm get their statements inside
their own branch, so each runs just when reached. The program start block cannot fail, so
`main` calls it as a plain function and sets the exit code from its integer result.

A call that does not pass the failure on keeps the whole result in a hidden slot of the
same struct, and the tests `is_good` and `is_fault` read its tag. A read `unwrap` or
`unwrap_fault` (`result_part`) is lowered like a pass-on call: its statements go before
the statement around it, test the tag, and `Halt` with `not_good` (showing the error
value) or `not_fault` on the wrong part. The value is then the `ok` or the `err` field.

An optional is the same struct with the tag and one field, `some`, for the data
(`Fault::slot` reaches `some` where the code asks for `ok`). Tag 1 has data and tag 0, the
blank struct, has none, so `အလွတ်` is the all-zero value. `has_data` and `lacks_data` read
the tag. `ထဲမှ ဒေတာ` is a `result_part` whose wrong part is an optional with no data, and
it halts with `HaltCause::NoData`, with no value to show.

Where a place wants an optional or a result and the value is a plain one,
`coerce_lowered` wraps it as the good side (tag 1 and the value), and `make_fault` builds
the result that contains `<e> ဟူသော အမှားအယွင်း` (tag 0 and the error value, a duplicate
of text read from a place). A return whose value has the data kind of the whole result,
such as `အလွတ်`, an error value or a result kept in a variable, leaves as it is
(`leave_with`, after the drops of the places that contain text), and each other return
returns the good value, which `coerce_lowered` wraps.

### Switches and enumerations

A switch becomes the `If` chain of a conditional (`switch_stmt`, which shares `if_chain`
with `if_stmt`). Each arm tests `Compare Eq` of the subject and its case (`case_cond`),
and the default arm is the last `else`. A subject that is a read of a place or a constant
is read again in each arm, while each other subject goes once into a hidden slot, which
joins the drops of its scope when it contains text. An arm with several cases tests them
with `Or`. A member, written out or as a bare case over an enumeration, becomes its value
(`member_expr`): a `Const`, static `Text`, or a `List` of the items of a list or a struct,
so the IR has no separate node for either.

The IR keeps a value of an enumeration as its value data kind: `kind_of`, each slot, each
signature and each wanted data kind of `coerce_lowered` take the enumeration off
(`repr`), so the folder, the flow checker and codegen see the value of the member. An
enumeration inside a struct or a list kind remains, and the VM looks through it where it
walks fields and cells. The value of a print or a template slot keeps the enumeration
kind (`shown`), and codegen reads it as the value kind (`stored`) for `print_enum`.

An enumeration whose members hold data is a struct, so the IR keeps no enumeration kind for
it (`variant.rs`). A member becomes a `List` of the tag, its data, and the zero `blank` of
each other field (`variant::value`). The test of a member is `Compare Eq` of the tag, a
`Load` at the `FieldAddr` of field 0, and the position of the member (`variant::is`); a
switch over such a value tests the tag in each arm (`variant_arm_cond`). The read of the
data holds a value that is no read of a place in a hidden slot (`held_subject`), stops the
run with `Halt` and the cause `wrong_variant` when the tag is another member, and loads the
field of the member. A name for the data (`bound_block`) is a local of the scope of the
block, set from a duplicate of the data when it holds text. The print of such a struct
writes `<stem>များထဲမှ <member>(<data>)`: the VM reads the members off the struct
(`variant_tasks`), and the C and native targets give its descriptor the member table, one
row for each member with its text and the number of the field of its data, which the
runtime prints (`push_variant`).

### Initial values and declarations with no value

`ကနဦးတန်ဖိုး` lowers through `initial_value` to the [initial
value](../language/kinds.md#initial-values) of its data kind: the blank optional, the
marked member of an enumeration, a new box for a shared object, a `List` of the initial
value of each field for a struct, and a `Repeat` of the initial value of its cell for a
counted list, so a long list costs one cell in the IR. Each backend spells a `Repeat` out
as a `List` before it writes code (`spans::spell_repeats`), so the backends see the cells.
A data kind whose initial value is zero octets (`zero_initial`: numbers, characters,
true/false values, pointers, text, plain lists, and structs and counted lists of these
alone) is one constant, a `BlankList` for an aggregate.

A declaration with no value still lowers to a `Set` of a filler (`default_constant`): zero
octets, the first member for an enumeration, or, for a data kind with an enumeration
inside, a value built one field at a time (`defaults_of_parts`, with a `Repeat` of one
cell for a counted list). The slot notes the span of that `Set` in `Slot.unset`, and [the
flow checker](borrow-checker.md#set-before-read) makes sure no path reads the filler, so
the program never sees it as a value.

### Pointers and parameters

- A pointer read through a place becomes `Load`.
- An argument for a parameter marked `အပိုင်အနေဖြင့်` is taken (`hand_over`): a `Take` of
  the place, so the caller's place contains the null address after the call. An owned
  result is noted on `Func.owned` (`Frame.owned_ret`).
- A mutable parameter (`inout`), marked `ဒေတာ ပြောင်းလဲရန်` (`Param::edit`), gets a slot
  that contains an address, and each read of its name becomes a `Load` through it and
  each write a store through it. Its argument (`edit_arg`) is the address of the caller's
  place, unless the root of the place is a global or a box (`reached_elsewhere`). Then
  `stage_in` copies the value into a hidden slot, whose address goes instead, and queues
  the write back in `Lowerer.post`, which runs right after the call with the place lowered
  again, so the cell is found after the call. A boxed mutable parameter is noted in
  `Frame.write_backs`, and each exit stores the box's value back through the caller's
  address.
- A writable parameter (`out`), marked `ဒေတာ ထည့်ပေးရန်` (`Param::fill`), is lowered the
  same as a mutable parameter, with two changes: its slot notes the span of the parameter
  in `Slot.unset`, so the set-before-read check follows it from the start of the
  function, and `stage_in` starts the hidden slot blank (the default constant of the data
  kind), since the callee reads nothing from it. `FnSig.fills` keeps the flag of each
  parameter for the calls.
- A place of a data kind that contains a counted block, read into another place, becomes
  `Dup`. Codegen copies the value into a scratch slot and runs `dup_seqs` on it, which
  retains each counted block in it, so the two share each block until one of them writes.
  A `Dup` that the flow checker marks `lent` (`dup_lent` in the IR printout) runs
  `lend_seqs` instead, which gives each text and plain list a room of 0 and retains
  nothing.
- A parameter in `Func::shared_params` gets a borrow slot that contains the caller's
  address, and each read of it goes through `local_ref`.

### List literals and static data

A plain list literal is built as a counted list of its cells, wrapped in a `Convert`. A
counted list that becomes a plain list, a literal or a counted place, gets a `Convert`
too. When the cells of the literal are each known (`fold::is_known`), codegen adds it to
the module's table of static lists, `seqs`, and emits `static_seq s<n>`: the slot points
at the static list with a room of 0, and no heap block is made. A set writes the three
words straight into the variable (`static_seq`, then `pop`), while a literal used as a
value on its own takes a `%own` slot first. Each other `Convert` gets a `%own` slot and
`to_heap k`, which moves the cells to a new heap block and writes the plain list of them
in the slot. A constant keeps the data kind its declaration gives.

A counted list or struct literal of known cells, set into a slot or duplicated into one,
gets a static list too: codegen emits `seq_addr s<n>` and `blit k`, in place of a store for
each cell or field. The VM puts each field of a struct at its offset. Such a literal used
as a value on its own, such as a printed one or one a range reads, still goes to its own
slot, which the blit fills.

### Writes and edits of lists and text

- A write into a plain list: the address of each written place (a cell, a field of a
  cell or the whole list) wraps each plain list on its path in an `Own` node, the outer
  one first, so a list lent with a room of 0, or one whose block another duplicate shares,
  gets its own block before the write. Codegen emits the address of the list, `dup`, and
  `own k`, and leaves the address for the write. In unsafe code, the address of a cell of
  a plain list or of text gets `Own { pin: true }` on its source: codegen adds `pin`, and
  the block is never shared again, so a write through the pointer reaches no duplicate.
- An add becomes `AddCell { dest, kind, value }` and a remove `RemoveCell { dest, kind }`
  (the s-expressions `add_cell` and `remove_cell`). The list goes through `dest`, so each
  plain list around it on the path gets its `Own` step, the same as for a write. The value
  of an add goes through `owned`, the same as an assigned value: text and lists are
  duplicated, and other values are read. Codegen works the value out first into the
  scratch slot `%cell`, then pushes the address of the list and the address of the slot
  and emits `add_cell k`. The operation moves the value out and leaves zero octets of the
  cell kind in the slot. A remove pushes the address of the list and emits `remove_cell
  k`.
- A write of one character of text (in `assign`) becomes `SetChar`, and an add to text (in
  `add_cell`) becomes `AddText`. The text goes through `dest` the same as a list does, so
  a plain list around it gets its `Own` step. Codegen pushes the address of the text, the
  number and the character, and emits `set_char`, or `set_char from 1` for `၏ n
  ခုမြောက်စာလုံး`. For an add, it pushes the address of the text and the value through
  `read_operand`, emits `add_text k` with `k` the data kind of the value, and drops a fresh
  text that the value made, such as a format, right after. The value is read where it is
  and never duplicated, so `t ထဲသို့ t ကို ထည့်မည်။` reads `t` itself, and the operation
  reads the added octets before it writes.
- A replace (`replace_text`) becomes `Replace`, with its value read where it is. Codegen
  pushes the text, both ends and the value and emits `replace_text k` (`replace_text k
  chars` for characters), dropping a fresh value the same as for `add_text`.
- An add in front of a cell or a character (`insert`, from `add_cell`) becomes `Insert`.
  A list cell goes through `owned` the same as for an add, and added text is read where it
  is. For a list, codegen works the value out into `%cell`, then pushes the list, the
  number and the slot and emits `insert_cell k`. For text, it pushes the text, the number
  and the value and emits `insert_text k`, dropping a fresh value the same as for
  `add_text`. A remove of the last character becomes `RemoveCell` with the text kind.
- `==` and `!=` on two plain lists emit `seq_eq k`, which compares the counts and then
  each cell, and `!=` adds a `not`. Two counted lists keep `list_eq k`, and so do two
  structs, which the VM compares one field at a time and never as raw octets.

### Ranges, text and drops

- A range `x ၏ အကွက် s မှစ၍ n ကွက်` (`range`) is a new list or text: the `Own` of a view, a
  `MakeSlice { source, start, len, from }` given the list kind, whose room word is set to
  0, so `own` copies the cells into its own block. `from` tells whether the source is a
  list of a known count, a plain list, or a heap block. A range is a fresh value
  (`fresh_text`), so a template slot or an operand drops it after the read. The count of a
  plain list is a `Load` of its second word, through a `FieldAddr`.
- A string literal with no template slots becomes `Text`, while one with template slots,
  a verbal join, and a `+` of two texts become a `Format`.
- A place whose data kind can contain a counted block (`holds_text`: text, a plain list,
  an object, a function value, or a list or struct with one inside) gets a `Drop` at each
  exit of its scope: the end of its block, the end of a loop round, before a `break`, a
  `continue` or a return, and, for a global, at the end of `main`. A short-lived text that
  is printed or used in a template slot is dropped right after its use.
- An assignment to such a place goes through a hidden slot `%new`: `(set l1 <value>)`,
  then `(drop l0)`, then `(set l0 (take (addr l1)))`, so the value is worked out from the
  old text before the old text is freed.

### Several files and the program start block

- A program of several files gives one module. As in the check, each file has its own
  top-level scope and its own function map, and an import brings the places and functions
  it names from the file it reads. Functions are numbered across all the files in the
  order of the tree, and a function of an imported file is named `<name>@<file>`, so two
  private functions of one name differ in the listing.
- The declarations of the imported files come first in the tree, so the `set`s of their
  variables are the first statements of `main`, and `imported_inits` counts them.
- The program start block lowers to a function, in its place among the others. `main`
  then ends with one call of it, before the drops of the globals. Its arguments are
  duplicates of the hidden globals `%args` and `%env`, and its result goes to the hidden
  global `%exit`. `host` (`Host { args, env, exit }`) records the numbers of these
  globals, each `None` when the program start block does not use it. Nothing else in the
  IR, the flow checker or the operations knows about the program start block.
- A REPL session lowers each chunk with `LowerOptions.repl_from`, the place where the
  chunk starts. Each variable of the program start block's own scope lives in a global
  (`Frame::session`), with no drop, and `main` ends with no drops of the globals, so the
  VM keeps the values for the next chunk. The statements before `repl_from` are still
  lowered, so each slot and function keeps its number, but their code is thrown away,
  since that code ran in earlier chunks. A box of such a variable is in `global_boxes`.

Threads, locks, atomic numbers and parallel loops lower to the nodes in the tables above,
and [Inside the VM](vm-internals.md#threads) follows each of them from the lowering to the
VM and the native targets.

## Folding

`fold(module, options, platform_bits)` has three foldings, which we choose with `--fold`
(`all`, `none`, or a list of `exprs`, `consts` and `branches`):

| Folding | Effect |
| --- | --- |
| `exprs` | an operation on constants is replaced with its value; a `Format` whose values are each constant becomes `Text`, and a print of such text becomes a print of static text |
| `consts` | a read of a `ကိန်းသေ` set to a constant is replaced with the value, and a read of a text constant with its static `Text` |
| `branches` | an `if` on a constant keeps the taken branch; a loop whose condition is the constant false is dropped |

The errors of constant operations are reported whatever the value of `--fold`: an overflow
(ZK0601), a zero divisor (ZK0602), a cell number outside its list (ZK0603), and a range
that passes the end of a list of a known count (ZK0605). The fold finds them with the
same `value` functions that the VM uses at run time, so the two cannot differ. Folding
does not remove the `set` of a constant; it rewrites its reads.

### Constant checks

The first `imported_inits` statements of `main` get one more check. A duplicate of them is
folded with each folding on, and each one whose value is then not known (not a constant,
a size, a blank list or a list of known values) is ZK0604. The check uses its own
duplicate, so `--fold` changes the IR but not which programs compile.

A constant that contains text or a plain list (a `ကိန်းသေ` whose data kind contains
either) gets the same kind of check. Both checks live in a separate compile-time step
(`comptime.rs`) that runs before the fold. It folds a duplicate of the module with each
folding on, keeps the value of each such constant that is then known, and reports ZK0606
for one that is not, at the first part that did not fold. The fold proper puts each kept
value in the constant's `set` whatever the value of `--fold`, so the constant has static data with
room 0. The options of the build then change how the IR shows a value, never whether a
program compiles.

With constant propagation on, the fold also keeps the known value of each constant list
or struct, and a `load` of a cell at a constant number, of a struct field, or of the count
of a plain list (`field_addr` 1) becomes its value. A constant number outside a known
plain list is ZK0603. Lowering never takes a constant (`owned` sees a read of the
constant, or of a field or cell of one), and the flow checker treats such a read as a
duplicate.

A range of text whose edge falls inside a character is ZK0607 when the fold knows the
text, the start and the length; otherwise the run-time check `text_edges` stops the
program (see [The VM target](vm-target.md#operations)).

## ဇယား values

Three IR nodes work on a ဇယား (map): the expression `RowFind`, which returns the number of
the row of an index (key) or -1, and the statements `SetRow` and `RemoveRow`. The rest
lowers to the nodes above (`lower/table.rs`):

- A read finds the row into a hidden slot, then a test builds the optional from the row,
  read through the rows the same as a cell, or the optional with no data.
- A write inside a row finds the row, halts with `no_row` when it returns -1, and writes at
  the address of the value through the rows, with `own` on the path the same as for a cell
  write.
- A set of the whole value is `SetRow`, and a literal sets each row into a hidden ဇယား in
  turn.
- The count reads the field of live rows. A loop walks a duplicate over its rows and skips
  each removed row with `continue`.
- The fallback of an optional keeps the optional in a hidden slot, and an `if` moves its
  data, or puts the fallback, into a second slot, so the fallback runs in its arm alone.

## Trait values

A trait value (`<trait> အရာ`) is a closure, so the backends see no new node
(`lower/trait_values.rs`). Its data kind is a function kind whose `of_trait` names the trait;
the dispatcher of each pair of trait value kind and held data kind takes the number of a
trait function, the address of an argument pack and its record.

- **Putting a value in.** The check puts the call `%trait-value` around the value, with the
  address of each function that gives its data kind a function of the trait, in the order
  of the trait (`FnAddr` arguments). `to_trait_node` lowers it as a closure: a record
  `%closure{code, info, c0}` holds the value, and the code is the dispatcher, made once
  for the pair and kept in `Lowerer::dispatchers`.
- **A call.** The check names a call through a trait value `%trait-call:<n>`. `dispatch`
  fills a hidden `%args{a0, …}` slot of the caller with the arguments after the first
  (the address of the place for a parameter marked `ဒေတာ ပြောင်းလဲရန်` or `ဒေတာ ထည့်ပေးရန်`)
  and a blank result, calls the trait value with the number and the raw address of the
  slot, and reads the result from its last field into a slot that its scope drops. The
  scope drops the pack too, so the dispatcher passes a new count of each value on.
- **The dispatcher.** An `if` chain on the number picks the function of the held data kind;
  the held value is field `c0` of the record. A result of the trait's own data kind is
  wrapped in a new record of the same dispatcher.
- **Selectors past the trait functions.** After the arm of each trait function, the
  dispatcher answers six more numbers, each with a pack of its own: the number of the held
  data kind (`NUMBER`, for tests and switches), a new count of the held value (`READ`), the
  address of the record (`RECORD`), `==` and `တစ်ခုတည်း` (`EQUAL`, `SAME`), and the text
  of the held value (`TEXT`). `trait_compare` asks the left trait value for its number and
  its record, then the right one to compare: its arm reads the other held value from that
  record when the numbers match, which makes the shapes match too. `TEXT` calls the
  `စာသားအဖြစ်ပြောင်း` of the held data kind, which the check passes after the trait
  functions, or formats the value as a print does.
- **A part.** The name of the function of a held kind that gets a trait through a part
  carries the fields of the path after `PART_PATH`; the arm passes the address of that
  part (`base_part_addr`), through the box of a shared object.
- **A change.** For a function that changes its first parameter, the pack starts with a
  field for the new trait value. The dispatcher duplicates the held value into a local,
  passes the address of the local, and writes a new trait value of that local into the
  field; the caller takes it out and puts it in the place of the trait value, dropping the
  old one. Each other holder of the old record keeps its value.

## Artifacts

`zakac irgen` writes the folded IR (`.zkirt` text, `.zkir` raw). [The VM
target](vm-target.md#artifacts) covers the format of artifacts and their spans, which are
the same for each stage.

## Tests and extending

`zakac_ir_builder/tests` has `lower.rs`, `fold.rs`, `value.rs` (operations on values),
`lists.rs`, `moves.rs`, `text.rs`, `modules.rs`, `artifact.rs`, `cli.rs` and `samples.rs`,
with golden `.zkirt` files in `expected/`.

To add an IR form: a variant in `ir.rs`, a rule in `lower.rs`, a case in `fold.rs` and in
`walk.rs` of the flow checker, then a rule in each backend ([The VM
target](vm-target.md#tests-and-extending) lists the steps for the VM).
