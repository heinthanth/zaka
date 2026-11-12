---
title: IR and code
description: How the checked tree is lowered to the IR, how constants are folded, how the IR becomes ops, and how the VM runs them, followed through one program.
---

# IR and code

After the names and kinds are checked, the compiler stops working on the tree. It lowers
the tree to an *IR*, a small program of slots, statements and kinded expressions; folds the
constants in it; lets [the flow checker](borrow-checker.md) read it; and then turns it
into *ops*, the instructions of a stack machine. The VM, `zaka_vm`, runs the ops.
Two crates hold this: `zakac_ir_builder` (the IR, lowering and folding) and
`zakac_codegen_target_vm` (the ops and the listing).

```text
 Program + KindTable                       from the name and kind check
        │  lower                           zakac_ir_builder::lower
        ▼
   ir::Module ──────────▶ fold ──▶ ir::Module (folded)         ZK06
                                        │
                                        │   borrow_check(&mut Module)   ZK07
                                        │   (marks proven cell addresses `checked`)
                                        ▼
                                  emit   zakac_codegen_target_vm::emit
                                        ▼
                                  bc::Module ──▶ listing (asm text)
                                        │
                                        ▼  zaka_vm::machine::Machine
                                  the output of the program
```

Each crate has a `run_phases` that runs the ones before it, so `zakac build` is lex,
parse, wording, names and kinds, lower, fold, flow check, emit. If an earlier stage
reported an error, the later ones are left out and the artifact holds no module; warnings
do not stop the chain.

## The IR

The IR is in `zakac_ir_builder/src/ir.rs`. It keeps what the later stages need and drops
what is not needed: names become numbers, blocks are flattened, operators have one
spelling, and each conversion is written out.

```text
Module { globals: Vec<Slot>, funcs: Vec<Func>, main: Vec<Stmt>, imported_inits, host, foreign: Vec<ForeignFn>, span }
Func   { name, params, result, ret, locals: Vec<Slot>, block: Vec<Stmt>, span }
Slot   { name, kind: DataKind, access: Var | Const | LoopVar, addressed, span }
Place  = Global(n) | Local(n)                   written g0, l0
Expr   { kind: DataKind, node: ExprNode, span }
```

Each declaration has a slot of its own, so a shadowing name never shares one. The
parameters of a function are its first locals, then the other locals; an external
function has a result variable `ရလဒ်` after its parameters (a slot the generator adds,
which no program can name), where the host or C puts what it gives. `unset` holds the span
of a declaration that wrote no value, which the flow checker reads. `addressed` marks a slot whose address the program takes, so it needs an address
that remains fixed.

A statement is one of nineteen forms:

| Form | Meaning |
| --- | --- |
| `Set { place, value }` | a declaration with its first value, or an assignment |
| `Print { value }` | a value (`PrintValue::Expr`) or static text (`PrintValue::Text`), then a line break |
| `If { cond, then, els }` | an `else if` chain nests in `els` |
| `Loop { init, cond, update, block }` | a while loop has a blank `init` and `update` |
| `Counted { var, from, to, step, mode, block }` | `var` runs from `from` to `to`, both included |
| `Break`, `Continue`, `Return { value }` | |
| `Leave { value, grant, span }` | give C leave to call the function value (`grant`), or take it back; the lowering spills the value to a hidden slot unless it is a place; written `(leave f)` and `(revoke f)` |
| `Halt { cause, value }` | stop the run for a read of the wrong part of a whole result or of an optional with no data; `cause` is `not_good` (the good result of a result that holds an error was read), `not_fault` (the error value of a good result was read) or `no_data` (`HaltCause::NoData`: the data of an optional that holds none was read); `value` is the error value to show, and `not_fault` and `no_data` have none, nor has a failure that holds no value (the trap then shows the bare message); written `(not_good v)`, `(not_fault)`, `(no_data)`, and `(not_good)` for a failure with no value |
| `Call { func, name, args }` | a call whose result is dropped |
| `CallValue { callee, args }` | a call through the function value `callee`, its result dropped; written `(call_value f 1)` |
| `Store { addr, value }` | write at an address |
| `Alloc`, `Resize`, `Free` | a heap sentence, with a `Dest`: a slot, or a cell in storage that holds an address |
| `Blit { src, dst, count, unit, overlap }` | write `count` cells of `unit` from the address `src` to the address `dst`; `overlap` allows the ranges to share cells; written `(blit ...)`, and `(shift ...)` when `overlap` is set |
| `Fill { dst, count, value }` | set `count` cells at the address `dst` to `value`; written `(fill ...)` |
| `Drop { dest, kind }` | let go of each counted block inside the value at `dest` (text, a plain list, a box, a closure record), of a kind that holds one, and leave it blank: the last holder frees the block; written `(drop l0)` |
| `AddCell { dest, kind, value }` | add `value` as a new last cell of the plain list at `dest`; written `(add_cell l0 7)` |
| `RemoveCell { dest, kind }` | remove the last cell of the plain list at `dest`, or the last character of the text there; written `(remove_cell l0)` |
| `SetChar { dest, kind, index, first, value }` | write the character `value` in place of character `index` of the text at `dest`, counted from `first` (0 or 1); written `(set_char g0 2 'x')` |
| `AddText { dest, kind, value }` | add the text or character `value` at the end of the text at `dest`; written `(add_text g0 "c")` |
| `Replace { dest, kind, start, end, chars, value }` | put `value` in place of the u8 cells (from 0) or, with `chars`, the characters (from 1) from `start` to `end` of the text at `dest`, both included; written `(replace l0 0 4 "x")` or `(replace_chars …)` |
| `Insert { dest, kind, index, first, value }` | put `value` in front of cell `index` of the plain list at `dest`, or in front of character `index` of the text there, counted from `first`; written `(insert l0 1 20)` |

An expression has its kind on each node, so no later stage has to work it out again.
`ExprNode` has: `Const(Value)`, `Read(Place)`, `Prefix`, `Infix`, `Compare`, `Logic`,
`Convert { implicit }`, `Call`, `CallValue { callee, args }` (a call through a function
value: the value first, then the arguments), `FnAddr { func, name }` (the function value of
`func`, written `(fn_addr name)`), `Closure { func, name, record }` (the closure of a nested function with captures, made where it is declared: `func` is its thunk, and `record` the address of the record in a hidden slot, which moves to a counted heap block), `Host { func, args }` (the one statement of an external
function: its result variable takes what the host function `func` gives for the
address of each parameter, written `(host read_file (addr l0))`; the IR builder makes it in
`host_statement`, and codegen emits the op `host` into the result slot), `Foreign { func, args }` (the one expression of a C function with a result; with no result, the statement `Stmt::Foreign { func, args, span }`), the pointer forms `AddrOf`, `Load`, `Lend` (an address the IR makes itself, under the
borrow mark the flow checker reads), `Take` (a value moved out, the place left at its
default), `Dup` (a duplicate; `deep` for a deep one), `Own` (a plain list made to own its
block before a write, written `(own ...)`; `pin` pins the block too),
`PtrAdd`, `PtrSub`, `SizeOf`, the list forms `List`, `BlankList`, `CellAddr`, `FieldAddr`,
and `MakeSlice` (the inner view a range reads its cells through), and the text forms `Text(String)` (static text, written `"..."` in the
s-expression form) and `Format(Vec<FmtPiece>)` (text built when the program runs, written
`(format "a" g0)`), whose pieces are `FmtPiece::Text(String)` or `FmtPiece::Value(Expr)`,
and `Chars { text, index, first }`: the character count of text (`(chars g0)`, no index)
or the character at a number (`(char@ g0 i)` from 0, `(char1@ g0 i)` from 1).
A `CellAddr` names the count of its list, or none for a cell of a plain list or a
view, whose count is read when the program runs. A `Value`
is an integer (kind and bits), a float (width and bit pattern), a char, a boolean or an
address.

### Lowering

`lower(program, kinds, structs, options)` walks the tree once with a `Lowerer`. It reads the kind
of each expression from the `KindTable` (it panics on a missing one, which a clean check
rules out). Its rules:

- Each declaration becomes a new slot, global at top level and local in a function. Scopes
  are kept in a stack of maps just while lowering; the IR has no scopes.
- A verb clause and a notation expression become the same `Infix`, `Compare` or `Prefix`.
- Where the check let a value widen into its place, lowering adds a `Convert` marked
  `implicit`. A written conversion is `implicit: false`.
- A negated integer literal becomes a negative constant, so `-၁၂၈` is a constant of `i8`.
- A call runs a declared function of its name, as the check decides, or else the function
  value of the name or of its place (`call_target`): a `Call` or a `CallValue`. The
  arguments of a call through a value follow `fn_shape`: each parameter in its own kind,
  or the address of the place for one marked `ဒေတာ ပြောင်းလဲရန်`, as a declared function places
  its parameters. `NAME ဟူသော လုပ်ဆောင်ချက်` becomes `FnAddr`, which the folder turns into the
  constant `f + 1`, so a constant list or an imported global can hold it; a template slot
  keeps such a constant as a value, which prints as the name of its function.
- A function declared inside a function (`lower/nested.rs`) is lifted into a function of the
  module of its own, numbered after the top-level ones and named `<name>#<index>` (and the
  file suffix). Each local it names from around it (`FnDecl::captures`, which the check
  wrote) lives in a *box*: a counted plain list of one cell (`DataKind::box_of`), made where
  the local is declared, and each read and write of the local, in the function around and
  inside the nested one, goes through the address of cell 0 (`edit_addr`). A box is never
  lent nor static. The lifted function takes each box after its declared parameters, with a
  retain (`Sig::captures`); a closure among the captures is kept as a value under the name
  `<name>%`. Where a function with captures is declared, `make_closure` fills a record in a
  hidden slot, a struct of kind `DataKind::closure_record` (`%closure{code, info, c0, ...}`):
  the code and info words for the backend, then each box or captured closure. `Closure {
  func, name, record }` moves the record to a counted heap block and gives the function
  value, which a hidden local `<name>%` holds and drops as its block ends; `func` is the
  thunk `<name>#record<index>`, which takes the declared parameters and the record address
  and calls the lifted function with each field. A call of the function through its name,
  and its value, read that local (`through_closure`, `closure_value`); a call of itself
  inside itself passes its own boxes on. A duplicate of a function value is a retain of
  the record, and a function value argument is a `Dup { lent: true }` (`lend_fn`); one that
  a call makes fresh waits in a hidden slot its scope drops (`hold_fresh_fn`).
- Each round of a loop gives a captured local declared in its block a box of its own. A
  captured counter of a counted loop, or the variable of an each loop, is copied into a new
  box at the start of each round (`rebox`, `counted_block`), so each closure made in a
  round keeps the value of its round. The variable of a generic loop is one box for the
  whole loop.
- `ကူးထည့်မည်` and `X ၏ မိတ္တူ` on a shared object build an object of its own from cell 0 of
  the box (`object_fields`); on each other value each is a plain duplicate.
  `အကုန်ကူးထည့်မည်` and `X တစ်<classifier>လုံး ၏ မိတ္တူ` lower to `Dup { deep: true }`, which
  copies each object it reaches once and shares function values; a deep `Dup` is never
  turned into a move nor lent. The test of one object (`same`) compares two function
  values as words, and two objects through the first words of their boxes.
- A field of a shared object reads cell 0 of its box first; a write to it never duplicates
  first, since each name of the object sees the one box.
- A function with a result gives each value back at its return; a return that must drop
  text first puts its value in a hidden slot of its own, drops, then returns the slot. The
  block of a function with a result gives back on each path, as the check of ZK0339 sees
  to, so no path falls off its end; a block that does not end in a return gets an
  unreached return of the zero value of the kind, since the targets want one there.
- A function that can fail gives back a compiler-built struct `%fallible{tag: u8, ok: T, err: E}`,
  with no `ok` field when the good side holds no value and no `err` field when the error side holds none. Tag 1 is a good result and tag 0 a failure, so the blank (all-zero) struct is a failure. The statements `လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်` and `လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်` are returns of a struct with the tag alone. Each
  return builds the struct in a hidden slot and returns it, after the drops of the places
  that hold text. A call that passes a failure on goes into a hidden slot, and statements
  that run before the statement holding it test its tag: on a failure the function drops
  its text and returns the error. Each operand left of the call is worked out first into
  a hidden slot (a duplicate for text and lists; a plain read of a pointer is left in place), so
  the written order holds. The right side of `&&` and `||`, the condition of a while loop and that of an
  `else if` arm get their statements inside their own branch, so each runs just when
  reached. The start block cannot fail, so `main` calls it as a plain function and sets the
  exit code from its integer result.
  A call that is no pass-on keeps the whole result in a hidden slot of the same struct,
  and the tests `is_good` and `is_fault` read its tag. A read `unwrap` or
  `unwrap_fault` (`result_part`) is lowered like a pass-on call: its statements go before the statement that
  holds it, test the tag, and `Halt` with `not_good` (showing the error value) or
  `not_fault` on the wrong part; the value is then the `ok` or the `err` field.
- An optional is the same struct with the tag and one field, `some`, for the data
  (`Fault::slot` reaches `some` where the code asks for `ok`). Tag 1 holds data and tag 0, the
  blank struct, holds none, so `အလွတ်` is the all-zero value. `has_data` and `lacks_data` read
  the tag; `ထဲမှ ဒေတာ` is a `result_part` whose wrong part is an optional with no data, and it
  halts with `HaltCause::NoData`, with no value to show. Where a place wants an optional or a
  result and the value is a plain one, `coerce_lowered` wraps it as the good side (tag 1 and
  the value); `make_fault` builds the result that holds `<e> ဟူသော အမှားအယွင်း` (tag 0 and
  the error value, a duplicate of text read from a place). A return whose value has the
  kind of the whole result, such as `အလွတ်`, an error value or a result held in a
  variable, leaves as it is (`leave_with`, after the drops of the places that hold text);
  each other return gives back the good value, which `coerce_lowered` wraps.
- A switch becomes the `If` chain of a conditional (`switch_stmt`, which shares `if_chain`
  with `if_stmt`): each arm tests `Compare Eq` of the subject and its case (`case_cond`),
  and the default arm is the last `else`. A subject that is a read of a place or a
  constant is read again in each arm; each other subject goes once into a hidden slot,
  which joins the drops of its scope when it holds text. An arm with several cases tests
  them with `Or`. A member, written out or as a bare case over an enumeration, becomes its
  value (`member_expr`): a `Const`, static `Text`, or a `List` of the items of a list or a
  struct, so the IR has no node of its own for either.
- The IR holds a value of an enumeration as its value kind: `kind_of`, each slot, each
  signature and each wanted kind of `coerce_lowered` take the enumeration off (`repr`), so
  the folder, the flow checker and codegen see the value of the member. An enumeration
  inside a struct or a list kind remains, and the VM looks through it where it walks fields
  and cells. The value of a print or a template slot keeps the enumeration kind
  (`shown`), and codegen reads it as the value kind (`stored`) for `print_enum`.
- `ကနဦးတန်ဖိုး` lowers through `initial_value` to the
  [initial value](../language/kinds.md#initial-values) of its kind: the blank optional, the
  marked member of an enumeration, a new box for a shared object, and a `List` of the
  initial value of each field or cell for a struct or a counted list. A kind whose initial
  value is zero octets (`zero_initial`: numbers, characters, true/false, pointers, text,
  plain lists, and structs and counted lists of these alone) is one constant, a `BlankList`
  for an aggregate.
- A declaration with no value still lowers to a `Set` of a filler (`default_constant`):
  zero octets, the first member for an enumeration, built one field at a time for a kind with
  an enumeration inside (`defaults_of_parts`). The slot notes the span of that `Set` in
  `Slot.unset`, and [the flow checker](borrow-checker.md#set-before-read) makes sure no
  path reads the filler, so it is never a value the program sees.
- `ကိန်းသေ` becomes a slot with `access: const`; the folder uses the flag.
- A pointer read through a place becomes `Load`. An argument for a parameter marked
  `အပိုင်အနေဖြင့်` is taken (`hand_over`): a `Take` of the place, so the caller's place holds
  the null pointer after the call. An owned result is noted on `Func.owned` (`Frame.owned_ret`). A
  parameter marked `ဒေတာ ပြောင်းလဲရန်` (`Param::edit`) gets a slot that holds an address; each read
  of its name becomes a `Load` through it and each write a store through it. Its argument
  (`edit_arg`) is the address of the caller's place, unless the root of the place is a
  global or a box (`reached_elsewhere`): then `stage_in` copies the value into a hidden slot,
  whose address goes instead, and queues the write back in `Lowerer.post`, which runs right
  after the call, with the place lowered again, so the cell is found after the call. A boxed
  `ဒေတာ ပြောင်းလဲရန်` parameter is noted in `Frame.write_backs`, and each exit stores the box's value
  back through the caller's address. A parameter marked `ဒေတာ ထည့်ပေးရန်` (`Param::fill`) is
  lowered as such a parameter is, with two changes: its slot notes the span of the parameter in
  `Slot.unset`, so the set-before-read check follows it from the start of the function, and
  `stage_in` starts the hidden slot blank (the kind's default constant), since the callee
  reads nothing from it. `FnSig.fills` holds the flag of each parameter for the calls. A place of a kind that holds a counted block, read into
  another place, becomes `Dup`: codegen copies the value into a temp slot and runs
  `dup_seqs` on it, which retains each counted block in it, so the two share each block
  until one of them writes. A `Dup` the flow checker marks `lent` (`dup_lent` in the IR
  printout) runs `lend_seqs` instead, which gives each text and plain list a room of 0 and
  retains nothing. A parameter in
  `Func::shared_params` gets a borrow slot that holds the caller's address, and each
  read of it goes through `local_ref`.
- A plain list literal is built as a counted list of its cells, wrapped in a
  `Convert`. A counted list that becomes a plain list, a literal or a counted place,
  gets a `Convert` too. When the literal's cells are each known (`fold::is_known`), codegen
  adds it to the module's table of static lists, `seqs`, and emits `static_seq s<n>`: the
  slot points at the static list with a room of 0, and no heap block is made. A set writes
  the three words straight into the variable (`static_seq`, then `pop`); a literal used as
  a value of its own takes a `%own` slot first. Each other `Convert` gets a `%own` slot and `to_heap k`, which moves the cells to a
  new heap block and writes the plain list of them in the slot. A constant keeps the kind
  its declaration gives.
- A counted list or struct literal of known cells, set into a slot or duplicated into one,
  gets a static list too: codegen emits `seq_addr s<n>` and `blit k`, in place of a store
  for each cell or field. The VM puts each field of a struct at its offset. Such a literal
  used as a value of its own, such as a printed one or one a range reads, still goes to a
  slot of its own, which the blit fills.
- A write into a plain list: the address of each written place, a cell, a field of a cell
  or the whole list, wraps each plain list on its path in an `Own` node, the outer one
  first, so a list lent with a room of 0, or one whose block another duplicate shares, gets
  a block of its own before the write. Codegen emits the address of the list, `dup`, and
  `own k`, and leaves the address for the write. The address of a cell of a plain list or
  of text, in marked code, gets `Own { pin: true }` on its source: codegen adds `pin`, and
  the block is never shared again, so a write through the pointer reaches no duplicate.
- An add becomes `AddCell { dest, kind, value }` and a remove `RemoveCell { dest, kind }`
  (S-expressions `add_cell` and `remove_cell`). The list goes through `dest`, so each plain
  list around it on the path gets its `Own` step as for a write; the value of an add goes
  through `owned`, as an assigned value does: text and lists are duplicated, other values
  read. Codegen works the value out first into the scratch slot `%cell`, then pushes the
  address of the list and the address of the slot and emits `add_cell k`; the op moves
  the value out and leaves zero octets of the cell kind in the slot. A remove pushes the
  address of the list and emits `remove_cell k`.
- A write of one character of text (in `assign`) becomes `SetChar`, and an add to text (in
  `add_cell`) becomes `AddText`. The text goes through `dest` as a list does, so a plain
  list around it gets its `Own` step. Codegen pushes the address of the text, the number
  and the character, and emits `set_char`, or `set_char from 1` for `၏ n ခုမြောက်စာလုံး`.
  For an add it pushes the address of the text and the value through `read_operand`, emits
  `add_text k` with `k` the kind of the value, and drops a fresh text the value made, such
  as a format, right after. The value is read where it is, never duplicated, so `t ထဲသို့ t
  ကို ထည့်မည်။` reads `t` itself; the op reads the added u8s before it writes.
- A replace (`replace_text`) becomes `Replace`, its value read where it is; codegen
  pushes the text, both ends and the value and emits `replace_text k` (`replace_text k
  chars` for characters), dropping a fresh value as for `add_text`.
- An add in front of a cell or a character (`insert`, from `add_cell`) becomes `Insert`;
  a list cell goes through `owned` as for an add, and added text is read where it is. For
  a list, codegen works the value out into `%cell`, then pushes the list, the number and
  the slot and emits `insert_cell k`; for text it pushes the text, the number and the
  value and emits `insert_text k`, dropping a fresh value as for `add_text`. A remove of
  the last character becomes `RemoveCell` with the text kind.
- `==` and `!=` on two plain lists emit `seq_eq k`, which compares the counts and
  then each cell; `!=` adds a `not`. Two counted lists keep `list_eq k`, and so do two
  structs, which the VM compares one field at a time and never as raw u8s.
- A range `x ၏ အကွက် s မှစ၍ n ကွက်` (`range`) is a list or text of its own: the `Own` of a
  view, a `MakeSlice { source, start, len, from }` given the list kind, whose room word is
  set to 0, so `own` copies the cells into a block of its own. `from` tells whether the
  source is a list of a known count, a plain list, or a heap block. A range is a fresh
  value (`fresh_text`), so a template slot or an operand drops it after the read. The count
  of a plain list is a `Load` of its second word, through a `FieldAddr`.
- A string literal with no template slots becomes `Text`; one with template slots, a verbal join, and a `+` of two texts,
  become a `Format`.
- A place whose kind can hold a counted block (`holds_text`: text, a plain list, an
  object, a function value, or a list or struct with one inside) gets a `Drop` at each exit
  of its scope: the end of its
  block, the end of a loop round, before a `break`, a `continue` or a return, and, for a
  global, at the end of `main`. A short-lived text that is printed or used in a template slot is
  dropped right after its use.
- An assignment to such a place goes through a hidden slot `%new`: `(set l1 <value>)`,
  then `(drop l0)`, then `(set l0 (take (addr l1)))`, so the value is worked out from the
  old text before the old text is freed.
- A program of several files gives one module. As in the check, each file has its own
  top-level scope and its own function map, and an import brings the places and
  functions it names from the file it reads. Functions are numbered across all the
  files in the order of the tree, and a function of an imported file is named
  `<name>@<file>`, so two private functions of one name differ in the listing.
- The declarations of the imported files come first in the tree, so the `set`s of their
  variables are the first statements of `main`; `imported_inits` counts them.
- The start block lowers to a function, in its place among the others. `main` then ends
  with one call of it, before the drops of the globals: its arguments are duplicates of
  the hidden globals `%args` and `%env`, and its result goes to the hidden global
  `%exit`. `host` (`Host { args, env, exit }`) records the numbers of these globals,
  each `None` when the start block does not use it. Nothing else in the IR, the flow
  checker or the ops knows about the start block.

### Folding

`fold(module, options, platform_bits)` has three foldings, chosen with `--fold` (`all`,
`none`, or a list of `exprs`, `consts` and `branches`):

| Folding | Effect |
| --- | --- |
| `exprs` | an operation on constants is replaced with its value; a `Format` whose values are each constant becomes `Text`, and a print of such text becomes a print of static text |
| `consts` | a read of a `ကိန်းသေ` set to a constant is replaced with the value, and a read of a text constant with its static `Text` |
| `branches` | an `if` on a constant keeps the taken branch; a loop whose condition is the constant false is dropped |

The errors of constant operations are reported whatever `--fold` holds: an overflow
(ZK0601), a zero divisor (ZK0602) and a cell number outside its list (ZK0603), found
with the same `value` functions that the VM uses at run time, so the two cannot
differ. Folding does not remove the `set` of a constant; it just rewrites its reads.

The first `imported_inits` statements of `main` get one more check: a duplicate of them is
folded with each folding on, and each one whose value is then not known (not a constant,
a size, a blank list or a list of known values) is ZK0604. The check uses its own
duplicate, so `--fold` changes the IR but not which programs compile.

A constant that holds text or a plain list (a `ကိန်းသေ` whose kind holds either) gets the
same kind of check. Both checks live in a compile-time step of their own (`comptime.rs`)
that runs before the fold: it folds a duplicate of the module with each folding on, keeps
the value of each such constant that is then known, and reports ZK0606 for one that is not,
at the first part that did not fold. The fold proper puts each kept value in the
constant's `set` whatever `--fold` holds, so the constant holds static data with room 0.
The options of the build then change how the IR shows a value, never whether a program
compiles. With constant propagation on, the fold also keeps the known value of each
constant list or struct, and a `load` of a cell at a constant number, of a struct field,
or of the count of a plain list (`field_addr` 1) becomes its value; a constant number
outside a known plain list is ZK0603.
Lowering never takes a constant (`owned` sees a read of the constant, or of a field or
cell of one), and the flow checker treats such a read as a duplicate.

Take this program (the file `sq.zaka`):

```text
ကိန်းသေ အလျား သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၂ ဖြစ်ပါစေ။

လုပ်ဆောင်ချက် နှစ်ထပ် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ အလျား သည် ၁၀ ထက် ကြီးလျှင် လုပ်ဆောင်ချက် နှစ်ထပ် ကို အလျား တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

`zakac irgen --fold none` prints the IR below (spans left out: `lo` and `hi`
pairs on each node). The global is a constant, and the function has one local, its
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
```

`main` sets the constant and calls the start block, function 1:

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

The start block comes out like this with `--fold none`:

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

and like this with the default `--fold all`: `read g0` is the constant `12`, so the
condition `12 > 10` is true, so the `if` is gone and its branch stands in its place.

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

## The ops

`zakac_codegen_target_vm::emit(module, platform_bits, options)` turns the folded IR into a
`bc::Module`:

```text
Module { platform_bits, globals: Vec<Slot>, texts: Vec<String>, formats: Vec<Vec<FmtArg>>, funcs: Vec<Func>, main: u32, overflow, host, foreign: Vec<ForeignFn>, foreign_calls: Vec<ForeignCall> }
Func   { name, params, ret, locals: Vec<Slot>, code: Vec<Op>, spans: Vec<Span> }
```

`funcs` holds the IR functions at their IR numbers, the start block among them, then one
more function, `main`, which sets the top-level variables and calls the start block;
`main` is its number. `texts` is the table of static texts, each
distinct text once: string literals, folded formats and printed text; `print_text` and
`text` name them through their index. `formats` lists the pieces of each format, `FmtArg::Text(t)` for
a static text or `FmtArg::Value(kind)` for a value from the stack; `format` names one through
its index, and the listing shows them in a `formats` section (`fmt0 t0 isize t1`). `spans`
runs parallel to `code` and tells the source region of each op, which is how the VM
and the explorer point at the line being run, and how a trap names its line (see [The
VM](#the-vm)). `Module::strip_debug` replaces each region with `Span::PLACEHOLDER`, the
span of no source, which is what a release build keeps (`--release`). The generator can add locals of its own,
named with a `%`: `%to`, `%step` and `%dir` for a counted loop, `%list`, `%slice`,
`%ret` and `%at` for the lists, structs and ranges a statement builds, `%text` for
text with no slot to go to, such as a printed format, and `%cell` for the value of an add. `EmitOptions`
holds the two choices of the generator: `results` (`--results`) and `scratch`
(`--scratch`), both below.

`overflow` is the `--overflow` mode the flow check options hold: with `stop`, the default, the VM
works `prefix` and `infix` out with checked arithmetic and traps on a result outside its
kind. `wrap op` wraps in each mode; the generator emits it for the steps and the distance
of a counted loop, which count on two's complement arithmetic.

The VM is a stack machine. Ops that work on values take the kind from the values on
the stack, so most ops name no kind. The ones that need a kind name it: `convert`, `load`,
`store`, `take`, and the list ops. The groups:

| Group | Ops |
| --- | --- |
| values | `push v`, `pop`, `dup` |
| storage | `local_addr n`, `local_ref n`, `global_addr n`, `load k`, `store k`, `take k` |
| arithmetic | `prefix op`, `infix op`, `compare op`, `convert k` (the ops print as `add`, `lt`, ...), `wrap op` |
| control | `jump t`, `jump_if_false t`, `jump_if_true t`, `call f`, `fn_addr f`, `closure f k`, `call_value n`, `c_leave k`, `c_revoke`, `c_load k at o`, `c_store k at o`, `ret`, `result_addr`, `trap code` |
| output | `print`, `print_text t`, `print_list k`, `print_enum k` (a member, found through its value and written as its name; a function value, written as the function it holds) |
| failure | `halt cause [k]`, with `not_good`, `not_fault`, `no_data` or `closed_handle` |
| pointers | `ptr_add stride`, `ptr_sub stride`, `alloc stride`, `realloc stride`, `free`, `mem_blit stride`, `mem_move stride`, `mem_fill k`, `pin`, `obj_to_ptr`, `ptr_to_obj` |
| lists | `index len from`, `blit k`, `zero k`, `list_eq k`, `seq_eq k`, `static_seq s`, `seq_addr s`, `to_heap k`, `add_cell k`, `remove_cell k` |
| views | `make_slice stride source`, `slice_cell stride from`: the inner views a range and a cell of a plain list read through |
| text | `text t`, `format fmt`, `drop k`, `dup_seqs k`, `deep_seqs k`, `lend_seqs k`, `own k`, `text_eq`, `char_count`, `char_at`, `set_char from`, `add_text k`, `text_edges`, `check_utf8` |

A range of text that is not the whole text gets `text_edges` after its `make_slice`, which
stops the program when an edge falls inside a character; when the fold knows the text, the
start and the length, it reports such a range as ZK0607 first. A conversion from a plain
list of u8s to text emits `check_utf8`, which stops the program when the u8s are not
UTF-8. `pin` marks a block so a later duplicate clones it. `obj_to_ptr` turns an object
into a pointer to its fields and keeps a count on it; `ptr_to_obj` turns the pointer back
and takes that count over. `dup_seqs`, `drop` and `own` retain, release and unshare
counted blocks, and the ring collector runs inside the VM, with no op of its own (see
below). `char_count` and `char_at` take their text
through `read_operand`, so a fresh text is dropped after the read.

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
| each loop | lowered to a `Loop`: `init` sets `%each` to a duplicate of the value (a lent one when the loop leaves the value as it is, see the flow checker internals; through the pointer for a pointer to a list) and `%new` to 0, `cond` tests it against the count, each round sets the variable to the cell at that number or to the character that starts at that u8 (`char_at_octet`), and `update` adds 1, or the u8 count of the character |
| while, generic loop | `<init>; Head: <cond>; jump_if_false Exit; <block>; Next: <update>; jump Head; Exit:`; `continue` goes to `Next`, `break` to `Exit` |
| call statement | `<args>; call f`, then `pop` if the callee has a result |
| declaration of a nested function with captures | `<record>; closure f k` |
| leave for C | `<value>; c_leave k` or `<value>; c_revoke` |
| field of a struct C keeps | `<address of the struct>; c_load k at o`, with a block address pushed first for a list or text |
| call through a function value | `<value>; <args>; call_value n`, the value read first; the result kind and the `%ret` slot come from the function kind (`fn_shape`) |
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
| `in-place` (default) | the local that each `return` gives back is read and written through `result_addr`; a returned literal is built at `result_addr`, and a returned call passes `result_addr` on as its slot, so `ret` duplicates nothing |
| `duplicate` | the function keeps the value in its own frame, and `ret` duplicates it into the slot |

A list, a struct or a range set into a slot is built in the slot itself when nothing it
reads could change while it is written: a range, whose `make_slice` reads its source
before it writes, and a literal whose cells are constants, other slots, and operators
on these. Each other value, such as `l = [l[1], l[0]]`, is built in a `%list` or
`%slice` local first and then duplicated in with `blit`, so it reads the slot whole. A
literal or a range with no slot to go to, such as one printed or passed, is built in
such a local too. With `--scratch apart`, each such value gets a local of its own. With
`--scratch shared`, a value of one statement is dead once the statement ends, so a later
statement takes a free local of the same name and kind; the values of one statement
still keep slots of their own. The locals of a counted loop live across its block and
are never shared.

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

`zakac build --emit asm` writes the listing. For `sq.zaka` above:

```text
; zakac asm listing v1, platform 64 bits
globals
  g0 အလျား isize

fn f0 နှစ်ထပ် (params 1) -> isize
locals
  l0 က isize
     0  local_addr l0        ; က
     1  load isize
     2  local_addr l0        ; က
     3  load isize
     4  mul
     5  ret

fn f1 ပရိုဂရမ် (params 0)
     0  push 12:isize
     1  call f0              ; နှစ်ထပ်
     2  print
     3  ret

fn f2 main (params 0)
     0  global_addr g0       ; အလျား
     1  push 12:isize
     2  store isize
     3  call f1              ; ပရိုဂရမ်
     4  ret
```

In `f0`, ops 0 to 4 read `က` twice and mul them, and `ret` leaves the product on the stack, which is how a result travels. In
`main`, the global gets its value (ops 0 to 2) and `call f1` runs the start block. There
the argument `12` is pushed (op 0; the folder turned `read g0` into a constant), `call f0`
takes the argument from the stack into the local `က` and leaves the result, and `print`
writes it. The program prints `၁၄၄`.

A counted loop shows the labels. For `၁ မှ ၃ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသမည်။`:

```text
fn f0 ပရိုဂရမ် (params 0)
locals
  l0 ကိန်း isize
     0  local_addr l0        ; ကိန်း
     1  push 1:isize
     2  store isize
     3  local_addr l0        ; ကိန်း
     4  load isize
     5  push 3:isize
     6  gt
     7  jump_if_true L1
L0:
     8  local_addr l0        ; ကိန်း
     9  load isize
    10  print
    11  push 3:usize
    12  local_addr l0        ; ကိန်း
    13  load isize
    14  convert usize
    15  wrap sub
    16  push 1:usize
    17  lt
    18  jump_if_true L1
    19  local_addr l0        ; ကိန်း
    20  local_addr l0        ; ကိန်း
    21  load isize
    22  push 1:isize
    23  wrap add
    24  store isize
    25  jump L0
L1:
    26  ret
```

Ops 3 to 7 are the one test before the first round (a start past the end runs zero
rounds). Ops 8 to 10 are the block. Ops 11 to 18 are the test of the formula above, with
`U` = `usize`. Ops 19 to 25 add the step and go round. The loop variable is `ကိန်း`, a
local of the start block.

## The artifacts

`zakac irgen` writes the folded IR (`.zkirt` text, `.zkir` raw), `zakac borrowck`
the IR and the digests, and `zakac build` the ops (`.zkct`, `.zkc`) or, with
`--emit asm`, the listing. Text artifacts start with a header line such as
`# zakac-codegen-artifact v1` and are the form of the golden files; the raw form is the
`ZKAR` envelope with Postcard inside. `zakac run` takes Zaka source, which it compiles,
or the ops of `zakac build` in either form, which it knows from the first octets.

An artifact keeps its spans relative to the start of its own source, so the spans do not
depend on where the source map placed the file. A decoded artifact registers its embedded source
and its imported files again, then makes its spans absolute against that registered file
(`absolute_module`, `absolute_diagnostics`). `relative` and `absolute` in
`zakac_ir_builder` leave a placeholder span as it is, since it names no source and a
shift would turn it into a region. `CodegenArtifact::strip_debug` is the release form of
an artifact: it calls `Module::strip_debug`, then clears the embedded source, the
imported files and the diagnostics, whose regions point into that source. `zakac build
--release` calls it before the artifact is written, and `zakac run --release` calls
`Module::strip_debug` on the module it compiles. An artifact built with no `--release`
keeps its regions, so `zakac run x.zkct` still names lines; `--target c` refuses the
flag.

## The VM

`zaka_vm::machine::Machine` owns a `bc::Module` and runs it one op at a time: `step()`
fetches the op at the program counter of the innermost call, executes it with `exec`, and
returns a `State`: `Running`, `Finished`, `Trapped`, or `Waiting` while a read of
standard input finds no line in a world that keeps its input open. `run` and `run_with` loop `step`;
the explorer calls `step`, `step_over` and `step_out` to show a program running.

- **Start.** `Machine::new` gives each global a static block, then each static text a
  frozen static block that holds its UTF-8, opens the stack region and enters `main`. A
  write inside a frozen block is the trap `FrozenWrite`. Globals start at zero octets, and the `Set` statements that
  the IR keeps run in source order.
- **The host.** `set_host` puts the arguments and the environment in the globals that
  `host.args` and `host.env` name, before the first op: each text in a heap block of its
  own, and each list in one more, so the drops at the end of `main` give them back.
  `exit_code` reads the global `host.exit` once the run has finished, and keeps its
  lowest 8 bits. A run with no host leaves both lists blank.
- **A call.** `call f` takes one block of the stack region for each local of `f`
  (`FrameMode::Checked`), or one block for all of them at C offsets (`Packed`), moves the
  arguments from the operand stack into the first locals, and pushes a `Frame`. `ret`
  pops the frame and gives the blocks back; for a list or struct result it first
  duplicates the value into the caller's slot, the value under the arguments. A depth past `max_depth` is the trap
  `StackOverflow`.
- **A function value.** `fn_addr f` pushes `f + 1` as an address value, so 0 holds no
  function. `call_value n` takes it from under the `n` arguments, stops the program with
  `NullCall` for 0, a backstop for a cell the init check could not prove set, and else enters function `f` as `call f` does. A closure is the address of its
  record on the heap, which lies above the function numbers (`Mem::closure_record`,
  `Mem::thunk_word`), with the lowest bit set while a call borrows it: `closure f k` pops
  the address of a record of kind `k`, moves it to a new counted heap block
  (`move_record`), writes `f + 1` into its first word, keeps `k` for the thunk in
  `Machine::records`, and pushes the address; `call_value n` on a closure reads the thunk
  from the first word and passes the address of the record as the last argument.
  `blocks_in` gives each function value inside a value too, so `dup_value` retains the
  record (`dup_fn`), `drop_value` releases it and the last holder frees it with what its
  fields hold (`drop_fn`), and `lend_value` sets the lowest bit (`lend_fn`), so a drop of a
  lent closure frees nothing. A closure given to a C function is lent to that call
  (`ffi.rs`, see [C functions](#c-functions)); a call from C to a closure that has no
  running C call and no leave stops the program with `ClosureEscape`. A call statement that drops a function value it gets stores it in a
  scratch slot and drops that. A print of a closure names the function its thunk calls.
  `Mem` keeps the name of each function, so `print_enum` and the prints of lists, structs
  and template slots write a function value as `<name> ဟူသော လုပ်ဆောင်ချက်` (`Mem::fn_text`,
  which cuts the overload number and the file off the inner name).
- **Counted blocks.** Each heap block of text, a plain list, a box or a closure record
  holds a count (`Block.rc`, with `Mem::retain`, `Mem::release`), the kind of its cells
  (`Block.kind`) and a `pinned` bit. Static and frozen blocks have no count. `own_value`
  and `remove_cell` give a block another holder shares a duplicate of its own first
  (`unshare`); each duplicate of a pinned block clones it. A drop queues the blocks it lets
  go (`freeing`), so a chain of 20,000 objects frees in a loop and not through deep
  recursion. `deep_seqs` (`machine/deep.rs`) makes a deep duplicate with a table from old
  objects to new ones, so each object is copied once and a ring keeps its shape. `==` on
  objects is deep and safe on rings (a stack of pairs), and a print shows an object that is
  still being printed as `…`.
- **The ring collector.** Counts alone never free objects that reach each other in a ring.
  A release that leaves a count above 0 buffers the block when its kind can reach objects,
  closures or boxes (`can_ring`); text and lists of numbers never wait. The collector
  (`machine/collector.rs`, `collect_rings`) runs trial deletion, as PHP and CPython do,
  when 1,024 blocks wait and when the program ends, so `--leaks` remains clean for safe code.
- **Storage.** `Mem` is the address space: blocks of three origins (`Heap`, `Stack`,
  `Static`), each with a red zone after it, so an access just past a block never lands in
  the next one. `load`, `store` and `take` check that the address is inside a live
  block, and trap if not: a null address, a freed block, past the end.
- **Values.** The operand stack holds `Value`s, so arithmetic wraps to the width of the
  kind, a zero divisor is the trap `ZeroDivisor`, and a shift past the width is
  `ShiftTooFar`. The same `value` functions did the folding.
- **Host functions.** The op `host h`, the one op of an external function, reads each
  argument through the address of its parameter as a `zaka_host::HostValue` (from its
  kind: text, a stream struct, an enumeration member), runs `h` against the world of
  the machine, and writes the result in the result slot from the kind of the result (a
  result that can fail: tag 1 and the good side, or tag 0 and the error side). The
  machine keeps its world (`Box<dyn World>`, a `MemWorld` unless `set_world` or `run_in`
  gives another) with the rest of its state, so a clone for the explorer carries the
  world along. Writes to the standard streams go to the output of the machine, so those
  writes keep their order with what the program prints. A `MemWorld` with `open` set (the
  explorer's) answers a read with no line with `WouldBlock`: `Effects` marks the call
  blocked, and the op puts its operands back, steps back to itself uncounted, and the
  state becomes `Waiting` until `give_input` or `close_input`. See
  [Host functions](#host-functions).
- **C functions.** The op `foreign c<n>` (`Op::Foreign`) makes the C call `n` of
  `Module::foreign_calls`, a table of `ForeignCall { func, extra }` over
  `Module::foreign`, a table of `ForeignFn { lib, name, params, ret, variadic }`. The code lives in
  `machine/ffi.rs`, behind the feature `ffi` of `zaka_vm`. See [C functions](#c-functions).
- **Limits.** `RunOptions` has `max_depth`, `max_steps`, `max_heap`, `max_stack`, the
  frame mode, and `leaks`. A trap is not a `ZK` diagnostic, since it belongs to the run of
  a program and not to the compiler.
- **The trap.** A `Trap` holds the `kind`, the `func` and `pc` of the op that stopped,
  the `block` text of an address trap, and `frames: Vec<TrapFrame>`, the calls in
  progress with the innermost first. A `TrapFrame { func, pc, span }` is a function name,
  the index of its op (for a caller, the op of its call, which is the return address
  less one) and the source region of that op, `None` when the module has none.
  `Machine::trap_frames` builds the list from the frame stack and leaves out the
  function `main` that the compiler adds, unless the stop is inside it. `Trap::report`
  writes the head line with the line of the first frame (`file:line:column`, the column
  from 1, through the source map), or `(in <func>, op <N>)` when the region is a
  placeholder; then, when more than one frame exists, the line `calls in progress, the
  last first:` and one line for each call. `backtrace` folds a run of frames of one op
  into the line and `... the call above, N times more`, and keeps `SHOWN_CALLS` (16)
  lines before `... N calls more`. The short form that the Run buttons show,
  `the program stopped: <reason>`, has no place and no calls.

The pages [Running a program](../vm/running.md), [Instructions](../vm/instructions.md),
[Addresses and blocks](../vm/blocks.md) and [When a program stops](../vm/traps.md)
tell the VM from the side of its user.

## Host functions

The functions the VM serves to a program live in `zaka_host`, in three parts:

| Part | Where | What it holds |
| --- | --- | --- |
| signature | the stdlib (`ပြင်ပမှလာသော` declarations) and `HOST_FNS` | the name, the parameter kinds and the good result of each function; the check matches each external declaration to a row (`zakac_semcheck::host::find`) |
| values | `zaka_vm/src/machine/host.rs` | the VM reads an argument and writes a result from its kind alone, so a new function over the kinds there needs no VM code |
| effects | the run function of a row, against `Effects` | plain Rust: `Effects` reaches the `World` (`read_line`, `read_file`, `write_file`) and the standard streams of the VM (`write`) |

`HOST_FNS` is one table, made with the macro `host_fns!`, which also makes the enum
`HostFn`:

```text
ReadFile = "ဖိုင်ကိုဖတ်"(TEXT) -> Some(TEXT), read_file;
```

Each row runs a Rust function `fn(&Args, &mut Effects) -> HostValue`; `Args::text(n)` and
`Args::stream(n)` read the arguments, and `text_or_fault` and `nothing_or_fault` turn an
`io::Result` into a result, a failure going through one mapping of `io::ErrorKind` to
the members of the error enumeration.

To add a host function:

1. Add a row to `host_fns!` and the function that runs it.
2. When it needs an effect outside the VM that `World` lacks, add a `World` method and
   write it for `MemWorld` (the browser and the tests) and `TermWorld` (the terminal).
   A new kind of argument or result needs a case in `machine/host.rs`.
3. Declare it in the stdlib with its marks.

A `MemWorld` holds the input lines given before the run and a map of files; a
`TermWorld` reads the standard input of the process and the files on disk. `zakac run`
runs in a `TermWorld`, and each other run (the tests, the books, the explorer) in a
`MemWorld`.

## C functions

A C function (`"libm" မှလာသော`) remains a function of the program whose one statement calls
C. Unlike a host function, it passes the value of each parameter and not its address. A C
function that takes other values (`အခြားတန်ဖိုးများနှင့်အတူ`, C's `...`) has no such function:
each call names the C function itself, with all its arguments.

| Stage | Where | What it does |
| --- | --- | --- |
| IR | `lower.rs`: `foreign_statement`, `foreign_index` | the statement is `ExprNode::Foreign { func, args }` (`Stmt::Foreign` with no result); `Module::foreign` holds one `ForeignFn { lib, name, params, ret }` for each C name, so two declarations of a name share a row. A C function that takes other values has `ForeignFn::variadic`, and `collect_sigs` gives its signature the `ForeignFn`, so `Target::Foreign` makes the call site `ExprNode::Foreign` (or `Stmt::Foreign`) with each argument and no wrapper function runs. `rest_arg` lowers an other value as C promotes it (`promoted`: an integer under 32 bits and a boolean to `i32`, `f32` to `f64`); a text that is not a plain read is built into a hidden slot (`spill`) that the scope end drops. A list of numbers goes as the address of its first cell (`c_list_arg`, `c_param_kind`), so `ForeignFn.params` holds a pointer for it; for a parameter marked `ဒေတာ ပြောင်းလဲရန်`, a plain list owns its cells first, so C writes them in place and no duplicate sees the writes |
| flow checker | `walk.rs`, `flow.rs` | each argument is a read, and the result is a fresh value, as for `Host` |
| VM ops | `emit.rs`: `foreign` | the op `foreign c<n>` pops each parameter and each other value, the last on top (a struct or a text is the address of its value); a struct result has the address of its slot under them and leaves it; another result is pushed; no result pushes nothing. One `bc::ForeignCall { func, extra }` stands for each shape of call: the C function and the kinds of its other values, so the same shape shares an index; `bc::Module::foreign_calls` carries them. A statement call (`foreign_stmt`) of a C function with a result pops it |
| VM | `zaka_vm/src/machine/ffi.rs` | the first call links: each lib opens (`libX` is `libX.dylib`, `libX.so` or `X.dll`; on Linux `libX.so.6` too; libc on Windows is `ucrtbase.dll`) and each C name is looked up in its lib, else in the process, which holds libc. libffi makes the call (`Cif::new_variadic` for a C function that takes other values, with the count of its named parameters). A text parameter or other value goes as the address of a duplicate of its u8s with a zero octet at the end (`arg_cell`, `text_octets`), kept alive until the call returns. A lib given as a path opens as it is (`open`). After each call `flush_c` calls libc's `fflush(NULL)`, found in the process at link time, so what C printed comes in order with the program's output. The run refuses when `--platform-bits` is not the host's, and the VM on a big-endian host is not supported for C |
| text result | `zakac_semcheck/src/foreign.rs` `is_c_text_result`; VM `machine/ffi.rs` `text_from_c` | the result kind `စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`, an optional that `is_c_text_result` tests, is read as a C string: NULL writes the optional with no data, else the zero-ended octets are copied into new text of the program (`bad_utf8` for octets that are not UTF-8). The text is taken before the duplicates of the text arguments are freed |
| handles | `lower.rs`: `foreign_statement`, `handle_ptr`, `handle_field`, `handle_guard`, `handle_close`, `handle_of`, `handle_or_none`, `c_field_read` | a [kind from outside the program](../language/functions.md#c-functions) crosses as its address: `c_param_kind` turns it into `DataKind::handle_pointer()`. The wrapper function guards each handle parameter with `Stmt::Halt { cause: ClosedHandle }` (the box holds 0), passes the address read from the box (`handle_ptr`), wraps a plain result with `make_box` after a test of the address (`handle_or_stop`: NULL halts with `HaltCause::NullFromC`, which the VM and the C target report as the trap `null from C`, since the declaration promised a handle), builds the optional of the result (`handle_or_none`: no data for NULL, the box as data otherwise; `is_c_handle_result` tests the kind), and after the call empties the box of a parameter marked `အပိုင်အနေဖြင့်` with a `Store` of 0 (`handle_close`), so each duplicate of the handle reads as closed. A field read is the node `ExprNode::CField { base, offset }`: the address of the C struct, which the guard checked, and the offset of the field; text read from a `char *` waits in a hidden slot the scope drops. No new statement exists |
| C field write | `lower.rs`: `c_field_store`; `emit.rs`; `machine/ffi.rs`: `c_store`; C `stmts.rs`: `c_store` | a write of a field of a handle in marked code is `Stmt::CStore { base, offset, value }` after the guard of the handle; the VM pops the value and the address and writes the octets in C's storage, and the C target writes them with `memcpy` |
| C field read | `emit.rs`: `c_field_at`; `machine/ffi.rs`: `c_load` | the op `c_load k at o` pops the address of the C struct and pushes a number or a boolean read at the offset; for a counted list or text, it pops the address of a block under it, writes the cells (or the string copied up to its zero octet, the optional with no data for NULL) there, and pushes that address. A handle that is closed stops the run first (`halt closed_handle`, the trap `closed handle`) |
| callbacks | `machine/ffi.rs`: `Callbacks` | a function value of a kind `is_callback` takes goes to C as a function pointer: one libffi closure for each function a call gives C, made on the first such call and kept for the run. `callback_arg` turns each argument into a value of the VM: a scalar is read as it is; a
plain struct is copied from the octets C passed, text from the `char *` (`text_arg`; NULL is
`NullFromC`), an optional of text through `text_from_c`, a handle into a new box around the
address (`handle_arg`, `optional_handle_arg`). Each of these lives in a heap block, and the
function gets a lent duplicate of it, as a call of the program lends; `drop_temps` drops
both once the function returns. It runs the function in the VM until it returns; a trap in it waits until C returns, then stops the program (each later callback of that call gives 0). A closure goes through `closure_callback`: one libffi closure for each record, found again from the octets of the record, and its code remains in place for the run, so a late call finds live code. `callback` counts the C calls that got the closure (`CallbackData::calls`, ended in `end_lends` once the call returns), and the ops `c_leave` and `c_revoke` (`Machine::c_leave`, `c_revoke`) hold or drop a count of the record and set `CallbackData::kept`. C can call a closure while `CallbackData::live` holds (a running call or a leave); a call from C at another time stops the program with `ClosureEscape`, and a call on another thread ends the run |
| C target | `zakac_codegen_target_c/src/ffi.rs` | `zaka_ffi.c` and one wrapper for each call shape (`FfiCall { func, extra }`, `Shared::ffi_calls`, `ffi_call`); see [The C target](c-backend.md#c-functions) |

An address that lies in a live block becomes the native address of that octet
(`Mem::native_at`), for a parameter and for an address field of a struct, so C reads and
writes the block in place. An address C gives back that lies in a block maps back
(`Mem::vm_addr`); another passes as it is and cannot be read through in the VM. A build
with no feature `ffi` (the explorer) holds `without_ffi::call_c`, which gives the trap
`ForeignCall`; a lib that does not open, a missing C name and a platform size that is not
the host's give `ForeignLink`. The zakac and book-test builds turn `ffi` on.

## Tests and extending

- `zakac_ir_builder/tests`: `lower.rs`, `fold.rs`, `value.rs` (operations on values),
  `lists.rs`, `moves.rs`, `text.rs`, `modules.rs`, and golden `.zkirt` files in `expected/`.
- `zakac_codegen_target_vm/tests`: `code.rs` (label patching), `emit.rs` (the ops of each
  construct), `listing.rs`, `artifact.rs`, and `samples.rs`, which holds golden
  `.zkct` and `.zkasm` files and checks that each op of each sample is well formed
  (indices in range, each function ending in `ret`).
- `zaka_vm/tests`: `machine.rs` (hand-built modules, malformed ops included, which must
  trap and not panic), `traps.rs`, `heap.rs`, `stepping.rs`, and `samples.rs`, which runs
  each program in `samples/` outside `diagnostics` and `mistakes` and compares its output
  with `tests/expected/<group>/<name>.out`.

To add an IR form: a variant in `ir.rs`, a rule in `lower.rs`, a case in `fold.rs` and in
`walk.rs` of the flow checker, a rule in `emit.rs`, and an op if the VM needs one
(`bc.rs`, `listing.rs`, `machine.rs`). To add an op: `bc::Op`, its operand text in
`Op::operand`, the arm of `Machine::exec`, and a test of the op on a hand-built module.
