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
Module { globals: Vec<Slot>, funcs: Vec<Func>, main: Vec<Stmt>, imported_inits, host, span }
Func   { name, params, result, ret, locals: Vec<Slot>, block: Vec<Stmt>, span }
Slot   { name, kind: DataKind, access: Var | Const | LoopVar, addressed, span }
Place  = Global(n) | Local(n)                   written g0, l0
Expr   { kind: DataKind, node: ExprNode, span }
```

Each declaration has a slot of its own, so a shadowing name never shares one. The
parameters of a function are its first locals, then the result variable `ရလဒ်`, then the
other locals. `addressed` marks a slot whose address the program takes, so it needs an address
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
| `Halt { cause, value }` | stop the run for a read of the wrong part of a whole result; `cause` is `not_good` (the good result of a result that holds an error was read) or `not_fault` (the error value of a good result was read); `value` is the error value to show, and `not_fault` has none, nor has a failure that holds no value (the trap then shows the bare message); written `(not_good v)` and `(not_fault)`, and `(not_good)` for a failure with no value |
| `Call { func, name, args }` | a call whose result is dropped |
| `Store { addr, value }` | write at an address |
| `Alloc`, `Resize`, `Free` | a heap sentence, with a `Dest`: a slot, or a cell in storage that holds an address |
| `Blit { src, dst, count, unit, overlap }` | write `count` cells of `unit` from the address `src` to the address `dst`; `overlap` allows the ranges to share cells; written `(blit ...)`, and `(shift ...)` when `overlap` is set |
| `Fill { dst, count, value }` | set `count` cells at the address `dst` to `value`; written `(fill ...)` |
| `Drop { dest, kind }` | free each text on the heap inside the value at `dest`, of a kind that holds text, and leave it blank; written `(drop l0)` |
| `AddCell { dest, kind, value }` | add `value` as a new last cell of the plain list at `dest`; written `(add_cell l0 7)` |
| `RemoveCell { dest, kind }` | remove the last cell of the plain list at `dest`, or the last character of the text there; written `(remove_cell l0)` |
| `SetChar { dest, kind, index, first, value }` | write the character `value` in place of character `index` of the text at `dest`, counted from `first` (0 or 1); written `(set_char g0 2 'x')` |
| `AddText { dest, kind, value }` | add the text, view of text or character `value` at the end of the text at `dest`; written `(add_text g0 "c")` |
| `Replace { dest, kind, start, end, chars, value }` | put `value` in place of the u8 cells (from 0) or, with `chars`, the characters (from 1) from `start` to `end` of the text at `dest`, both included; written `(replace l0 0 4 "x")` or `(replace_chars …)` |
| `Insert { dest, kind, index, first, value }` | put `value` in front of cell `index` of the plain list at `dest`, or in front of character `index` of the text there, counted from `first`; written `(insert l0 1 20)` |

An expression has its kind on each node, so no later stage has to work it out again.
`ExprNode` has: `Const(Value)`, `Read(Place)`, `Prefix`, `Infix`, `Compare`, `Logic`,
`Convert { implicit }`, `Call`, the pointer forms `AddrOf`, `Load`, `Lend`, `Take`, `Dup`, `Own` (a plain list made to
own its block before a write, written `(own ...)`),
`PtrAdd`, `PtrSub`, `SizeOf`, the list forms `List`, `BlankList`, `CellAddr`, `FieldAddr`,
and `MakeSlice`, and the text forms `Text(String)` (static text, written `"..."` in the
s-expression form) and `Format(Vec<FmtPiece>)` (text built when the program runs, written
`(format "a" g0)`), whose pieces are `FmtPiece::Text(String)` or `FmtPiece::Value(Expr)`,
and `Chars { text, index, first }`: the character count of text (`(chars g0)`, no index)
or the character at a number (`(char@ g0 i)` from 0, `(char1@ g0 i)` from 1).
A `CellAddr` names the count of its list, or none for a cell of a slice,
whose count is read when the program runs. A `Value`
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
- A function that can fail gives back a compiler-built struct `%fallible{tag: u8, ok: T, err: E}`,
  with no `ok` field when the good side holds no value and no `err` field when the error side holds none. Tag 1 is a good result and tag 0 a failure, so the blank (all-zero) struct is a failure. The statements `လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်` and `လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်` are returns of a struct with the tag alone. Each
  return builds the struct in a hidden slot and returns it, after the drops of the places
  that hold text. A call that passes a failure on goes into a hidden slot, and statements
  that run before the statement holding it test its tag: on a failure the function drops
  its text and returns the error. Each operand left of the call is worked out first into
  a hidden slot (a duplicate for text and lists; a plain read of an owner is left in place), so
  the written order holds. The right side of `&&` and `||`, the condition of a while loop and that of an
  `else if` arm get their statements inside their own branch, so each runs just when
  reached. The start block cannot fail, so `main` calls it as a plain function and sets the
  exit code from its integer result.
  A call that is no pass-on keeps the whole result in a hidden slot of the same struct,
  and the tests `is_good` and `is_fault` read its tag. A conversion `unwrap` or
  `unwrap_fault` is lowered like a pass-on call: its statements go before the statement that
  holds it, test the tag, and `Halt` with `not_good` (showing the error value) or
  `not_fault` on the wrong part; the value is then the `ok` or the `err` field.
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
  the folder, the borrow checker and codegen see the value of the member. An enumeration
  inside a struct or a list kind remains, and the VM looks through it where it walks fields
  and cells. The value of a print or a template slot keeps the enumeration kind
  (`shown`), and codegen reads it as the value kind (`stored`) for `print_enum`.
- The default of a kind with an enumeration inside is built from the default of each field
  and cell (`defaults_of_parts`), so each enumeration starts at its first member; other
  kinds start at zero octets, through `BlankList`.
- `ကိန်းသေ` becomes a slot with `access: const`; the folder uses the flag.
- A pointer read through a place becomes `Load`; an owner moved out becomes `Take`; a
  lend becomes `Lend`. A parameter the function can change (`Param::edit`) gets a borrow slot;
  each read of its name becomes a `Load` through it and each write a store through it,
  and the marked argument becomes the address of the caller's place. A place of a kind that holds text, read into another place,
  becomes `Dup`: codegen copies the value into a temp slot and runs `dup_seqs` on it, so
  each text and plain list on the heap in the duplicate gets a block of its own. A `Dup`
  the borrow checker marks `lent` (`dup_lent` in the IR printout) runs `lend_seqs`
  instead, which gives each text and plain list a room of 0. A parameter in
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
  used as a value of its own, such as a printed one or one sliced, still goes to a slot of
  its own, which the blit fills, since a slice can write the cells it borrows.
- A write into a plain list: the address of each written place, a cell, a field of a cell
  or the whole list, wraps each plain list on its path in an `Own` node, the outer one
  first, so a list lent with a room of 0 copies its cells into a block of its own before
  the write. Codegen emits the address of the list, `dup`, and `own k`, and leaves the
  address for the write. A slice of a plain list (a range or a lend) and the address of a
  cell get the same `Own` on their source first, since either can write the cells.
- A pass at the end of lowering (`zakac_ir_builder/src/lends.rs`) takes the `Own` off the
  source of each slice that reads alone. It follows each slice through the slots it is
  set into and the parameters it is passed to, until nothing changes; a slice that is
  written, whose address is taken, or that a function gives back counts as written, and
  so does each slice parameter and the result slot of a function that gives back a slice.
  The `Own` of a slice that reads alone goes when the list is a slot no pointer reaches and each
  other `Own` on it sits in a `Store` address or an add or a remove, which end the loans
  into it. So a static list lent to a reader takes no heap block, and no slice outlives a
  move of the cells it reads.
- An add becomes `AddCell { dest, kind, value }` and a remove `RemoveCell { dest, kind }`
  (S-expressions `add_cell` and `remove_cell`). The list goes through `dest`, so each plain
  list around it on the path gets its `Own` step as for a write; the value of an add goes
  through `owned`, as an assigned value does: text and lists are duplicated, an owner
  moves. Codegen works the value out first into the scratch slot `%cell`, then pushes the
  address of the list and the address of the slot and emits `add_cell k`; the op moves
  the value out and leaves the default of the cell kind in the slot. A remove pushes the
  address of the list and emits `remove_cell k`.
- A write of one character of text (in `assign`) becomes `SetChar`, and an add to text (in
  `add_cell`) becomes `AddText`. The text goes through `dest` as a list does, so a plain
  list around it gets its `Own` step. Codegen pushes the address of the text, the number
  and the character, and emits `set_char`, or `set_char from 1` for `၏ n ခုမြောက်စာလုံး`.
  For an add it pushes the address of the text and the value through `read_operand`, emits
  `add_text k` with `k` the kind of the value, and drops a fresh text the value made, such
  as a format, right after. The value is read where it is, never duplicated, so `t ထဲသို့ t
  ကို ထည့်ပါ။` reads `t` itself; the op reads the added u8s before it writes.
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
- A range `x ၏ အကွက် s မှစ၍ n ကွက်` becomes `MakeSlice { source, start, len, from }`,
  where `from` tells whether the source is a list of a known count, a slice, or a heap
  block. The lend of a list is a `MakeSlice` of each cell, marked `checked`. The count of
  a slice is a `Load` of its second word, through a `FieldAddr`.
- A string literal with no template slots becomes `Text`; one with template slots, a verbal join, and a `+` of two texts,
  become a `Format`.
- A place whose kind can own a heap block (`holds_text`: text, a plain list, or a list or
  struct with one inside) gets a `Drop` at each exit of its scope: the end of its
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
  each `None` when the start block does not use it. Nothing else in the IR, the borrow
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
cell of one), and the borrow checker treats such a read as a duplicate, so a read
after one is no ZK0701.

Take this program (the file `sq.zaka`):

```text
ကိန်းသေ အလျား သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၂ ဖြစ်ပါစေ။

လုပ်ဆောင်ချက် နှစ်ထပ် သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က * က ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ အလျား သည် ၁၀ ထက် ကြီးလျှင် လုပ်ဆောင်ချက် နှစ်ထပ် ကို အလျား တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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

`zakac_codegen::emit(module, platform_bits, options)` turns the folded IR into a
`bc::Module`:

```text
Module { platform_bits, globals: Vec<Slot>, texts: Vec<String>, formats: Vec<Vec<FmtArg>>, funcs: Vec<Func>, main: u32, overflow, host }
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
and the explorer point at the line being run. The generator can add locals of its own,
named with a `%`: `%to`, `%step` and `%dir` for a counted loop, `%list`, `%slice`,
`%ret` and `%at` for the lists, structs and slices a statement builds, `%text` for
text with no slot to go to, such as a printed format, and `%cell` for the value of an add. `EmitOptions`
holds the two choices of the generator: `results` (`--results`) and `scratch`
(`--scratch`), both below.

`overflow` is the `--overflow` mode the borrow-check options hold: with `stop`, the default, the VM
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
| control | `jump t`, `jump_if_false t`, `jump_if_true t`, `call f`, `ret`, `result_addr`, `trap code` |
| output | `print`, `print_text t`, `print_list k`, `print_enum k` (a member, found through its value and written as its name) |
| failure | `halt cause [k]` |
| pointers | `ptr_add stride`, `ptr_sub stride`, `alloc stride`, `realloc stride`, `free`, `mem_blit stride`, `mem_move stride`, `mem_fill k` |
| lists | `index len from`, `blit k`, `zero k`, `list_eq k`, `seq_eq k`, `static_seq s`, `seq_addr s`, `to_heap k`, `add_cell k`, `remove_cell k` |
| slices | `make_slice stride source`, `slice_cell stride from` |
| text | `text t`, `format fmt`, `drop k`, `dup_seqs k`, `lend_seqs k`, `own k`, `text_eq`, `char_count`, `char_at`, `set_char from`, `add_text k`, `text_edges`, `check_utf8` |

A range of text that is not the whole text gets `text_edges` after its `make_slice`, which
stops the program when an edge falls inside a character; when the fold knows the text, the
start and the length, it reports such a range as ZK0607 first. A conversion from a slice of
u8s to a view of text emits `check_utf8`, and a widening of a view to a slice of u8s emits
nothing, since both are the same two words. `char_count` and `char_at` take their text
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
| each loop | lowered to a `Loop`: `init` sets `%each` to a duplicate of the value (a lent one when the loop leaves the value as it is, see the borrow checker internals; through the pointer for a pointer to a list) and `%new` to 0, `cond` tests it against the count, each round sets the variable to the cell at that number or to the character that starts at that u8 (`char_at_octet`), and `update` adds 1, or the u8 count of the character |
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

A list, a struct or a slice set into a slot is built in the slot itself when nothing it
reads could change while it is written: a slice, whose `make_slice` reads its source
before it writes, and a literal whose cells are constants, other slots, and operators
on these. Each other value, such as `l = [l[1], l[0]]`, is built in a `%list` or
`%slice` local first and then duplicated in with `blit`, so it reads the slot whole. A
literal or a slice with no slot to go to, such as one printed or passed, is built in
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

In `f0`, ops 0 to 2 set the result variable to zero, ops 3 to 7 read `က` twice and
mul them, and `ret` leaves the product on the stack, which is how a result travels. In
`main`, the global gets its value (ops 0 to 2) and `call f1` runs the start block. There
the argument `12` is pushed (op 0; the folder turned `read g0` into a constant), `call f0`
takes the argument from the stack into the local `က` and leaves the result, and `print`
writes it. The program prints `၁၄၄`.

A counted loop shows the labels. For `၁ မှ ၃ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသပါ။`:

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

- **Start.** `Machine::new` gives each global a static block, then each static text a
  frozen static block that holds its UTF-8, opens the stack region and enters `main`. A
  write inside a frozen block is the trap `FrozenWrite`. Globals start at the default of their kind, and the `Set` statements that
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
[Addresses and blocks](../vm/blocks.md) and [When a program stops](../vm/traps.md)
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
