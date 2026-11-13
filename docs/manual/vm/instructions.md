---
title: Instructions
description: The instructions of the Zaka VM in groups, with a program shown beside its code.
---

# Instructions

The code generator turns a program into a list of instructions for each function, and
the VM runs them one at a time. This page lists the instruction set and shows a
small program next to its instructions.

The instruction set belongs to the compiler and the VM as a pair, and it can change
between versions. A listing made with one version of `zakac` is for that version. The
rules of the language do not depend on it: what a program means is set in
[the language](../language/README.md), and the instructions are one means to run it.

## Reading a listing

`zakac build --emit asm` writes the listing of a program in place of the ops. Here is a
function that doubles its parameter, and a start block that calls it:

```zaka
လုပ်ဆောင်ချက် နှစ်ဆ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
န * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၁ ဖြစ်ပါစေ။
နှစ်ဆ ဟူသော လုပ်ဆောင်ချက်ကို က ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၂
```

```console
$ zakac build --emit asm program.zaka
; zakac asm listing v1, platform 64 bits

fn f0 နှစ်ဆ (params 1) -> isize
locals
  l0 န isize
  l1 ရလဒ် isize
     0  local_addr l1        ; ရလဒ်
     1  push 0:isize
     2  store isize
     3  local_addr l0        ; န
     4  load isize
     5  push 2:isize
     6  mul
     7  ret

fn f1 ပရိုဂရမ် (params 0)
locals
  l0 က isize
     0  local_addr l0        ; က
     1  push 21:isize
     2  store isize
     3  local_addr l0        ; က
     4  load isize
     5  call f0              ; နှစ်ဆ
     6  print
     7  ret

fn f2 main (params 0)
     0  call f1              ; ပရိုဂရမ်
     1  ret
```

How to read it:

- `g0`, `l1`, `f0`, `t0`, `fmt0` are the index of a global, a local, a function, a static
  text and a format. After `;` comes the source name, for the reader. A jump shows a label, `L0`, in
  the listing; the artifact holds the index of the target instruction.
- A function lists its locals, parameters first (`l0` is the parameter `န`), then the
  locals of the source, then slots the generator adds, such as `%list`, `%slice`, `%to`,
  `%step`, `%dir`, `%text`, `%cell` and `%new`. The start block is a function named `ပရိုဂရမ်`. The last function is `main`: it
  sets the starting values of the top-level variables, calls the start block, and gives
  back their text at the end.
- A kind is written short: `isize` and `usize` for the platform-sized integers, `u8`,
  `i32` and so on, `f64`, `char`, `bool`, `isize*` for a pointer to an integer,
  `isize[3]` for a list of three integers, `isize[]` for a plain list, `text`, and
  `အဆစ်@` for a shared object of the struct `အဆစ်`. A value is written `21:isize`.
- A variable has no instruction of its own. Reading `က` is `local_addr l0` to push the
  address of its block (`global_addr` for a top-level variable), then `load isize` to read it. Writing is the address, the value,
  then `store`. This is what gives each variable an address.
- The generator adds a local named `ရလဒ်` to a function with a result, which no program
  can name. It is set to its default, `0`, at the start, and a return leaves its value on the
  operand stack for the caller: `ret` ends the function with that value on top.
- The call `call f0` takes its argument from the operand stack into the parameter, and
  the result comes back on the operand stack. `print` pops it.

A program with text has two more tables before its functions. `texts` holds each static
text once: string literals, formats whose template slots folded, and printed text. `formats`
holds the pieces of each text built when the program runs: a static text `t1`, or the
kind of a value that the program supplies (`isize`, `text`, `isize[3]`, ...).

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "မြ" ဖြစ်ပါစေ။
ကိန်းရှင် အသက် သည် ၂၅ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
"{အမည်}: {အသက်}" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မြ: ၂၅
```

```console
$ zakac build --emit asm program.zaka
; zakac asm listing v1, platform 64 bits
texts
  t0 "မြ"
  t1 ": "
formats
  fmt0 text t1 isize

fn f0 ပရိုဂရမ် (params 0)
locals
  l0 အမည် text
  l1 အသက် isize
  l2 %text text
     0  local_addr l0        ; အမည်
     1  text t0
     2  pop
     3  local_addr l1        ; အသက်
     4  push 25:isize
     5  store isize
     6  local_addr l2        ; %text
     7  local_addr l0        ; အမည်
     8  local_addr l1        ; အသက်
     9  load isize
    10  format fmt0
    11  print_list text
    12  local_addr l2        ; %text
    13  drop text
    14  local_addr l0        ; အမည်
    15  drop text
    16  ret

fn f1 main (params 0)
     0  call f0              ; ပရိုဂရမ်
     1  ret
```

A list literal whose cells are each known when the program is compiled adds one more
table, `seqs`: each static list, with its kind and its cells. A text cell names a static
text (`t0`), and a plain list cell an earlier static list (`s0`). The VM puts each in a
frozen static block before `main` starts. `static_seq` points a plain list at one, and a
counted list or a struct copies its cells from one with `seq_addr` and `blit`:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် l သည် [၁၊ ၂] ဖြစ်ပါစေ။
l ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၂]
```

```console
$ zakac build --emit asm program.zaka
; zakac asm listing v1, platform 64 bits
seqs
  s0 isize[] [1:isize, 2:isize]

fn f0 ပရိုဂရမ် (params 0)
locals
  l0 l isize[]
     0  local_addr l0        ; l
     1  static_seq s0
     2  pop
     3  local_addr l0        ; l
     4  print_list isize[]
     5  local_addr l0        ; l
     6  drop isize[]
     7  ret

fn f1 main (params 0)
     0  call f0              ; ပရိုဂရမ်
     1  ret
```

`text t0` points `အမည်` at the static text, with no heap block. The template has a
variable in it, so `format fmt0` builds it in the scratch slot `%text` from the address
of the text `အမည်` and the value of `အသက်`. The print drops the slot right after, and the
end of the program drops `အမည်`, which holds static text and frees nothing.

The ops of `zakac build` hold the same instructions as data: raw (`.zkc`) or as
text (`.zkct`, a header line and then YAML), chosen with `--emit` or the file name in
`-o`. `zakac run` runs either form.

## The instructions

The tables use `→` for the effect on the operand stack: the inputs, with the top
rightmost, then the outputs. `k` is a kind, and `s` the size of one element in octets.

### Values and addresses

| Instruction | Effect | Meaning |
| --- | --- | --- |
| `push v` | → v | push a constant |
| `local_addr n` | → addr | the address of the block of local `n` of the running call |
| `local_ref n` | → addr | the address local `n` holds: the caller's list or struct, for a parameter the function reads in place |
| `global_addr n` | → addr | the address of the block of global `n` |
| `result_addr` | → addr | the address of the caller's slot for the list or struct result of the running call |
| `dup` | v → v v | push the top value again |
| `pop` | v → | drop the top value |

### Operators

| Instruction | Effect | Meaning |
| --- | --- | --- |
| `neg`, `bnot`, `not` | a → r | minus, bitwise flip, logical not |
| `add`, `sub`, `mul`, `div`, `rem` | a b → r | arithmetic; integers trap on a result outside their kind, or wrap to their width when the module asks for it (`--overflow wrap`) |
| `wrap op` | a b → r | `op` on integers, wrapped to their width in each mode; a counted loop uses it for its own steps |
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
| `take k` | addr → v | read the value, then leave zero octets of the kind there (the null pointer for a pointer): a move at a last use, or the hand-over of an `အပိုင်အနေဖြင့်` argument |
| `ptr_add s` | addr n → addr | move an address forward `n` elements of `s` octets |
| `ptr_sub s` | a b → n | the number of elements between two addresses, an `isize` |
| `alloc s` | n → addr | a new heap block of `n` elements, or the null pointer when there is no room |
| `realloc s` | addr n → addr | resize a heap block; the block can move |
| `free` | addr → | give a heap block back; the null pointer does nothing |
| `mem_blit s` | src count dst → | write `count` cells of `s` octets (1 for octets) from `src` to `dst`; the ranges must not overlap, or the program stops |
| `mem_move s` | src count dst → | the same, with the ranges free to overlap: the source is read in full before the write |
| `mem_fill k` | dst value count → | set `count` cells at `dst` to `value`, a value of kind `k` |

Each of these checks its address as described in [blocks](blocks.md), and can trap. The
compiler emits `ptr_add`, `ptr_sub`, the heap ops and the `load` and `store` through a
pointer for unsafe code alone ([Unsafe Zaka](../language/unsafe.md)); the VM checks
them all the same.
`ptr_add` also traps on a step off the null pointer and on a step out of the block it starts in.
`mem_blit`, `mem_move` and `mem_fill` check that the source range lies in one live block and
the destination range in one live block that can be written; a count of 0 checks nothing,
and `mem_blit` traps with [overlap](traps.md#all-the-traps) when its two ranges share an octet.

### Lists

A list lives in a block, and its instructions work on addresses. A counted list is its
cells, inline; a plain list is three words, as text is (the address of its first
cell, its count, and the room of its heap block), and its cells sit in a static block or
a heap block. A heap block of text or of a plain list counts the values that share it.

| Instruction | Effect | Meaning |
| --- | --- | --- |
| `index 3`, `index 3 from 1` | n → i | check a cell number against a list of 3 cells, whose first cell is numbered 0 (or 1, after `from`); push it counted from 0 |
| `blit k` | to from → | duplicate a value of kind `k`, such as a whole list, from one address to another |
| `zero k` | addr → | set the octets of a value of kind `k` to zero |
| `list_eq k` | a b → bool | whether each cell of one counted list, or each field of one struct, equals the cell or field of the other |
| `seq_eq k` | addr addr → b | whether two plain lists with cells of kind `k` hold the same count of cells, each equal; a shared object is a one-cell list, so this is also the deep `==` of objects. A pair of blocks still under compare further up counts as equal, so a ring ends, and one block on both sides is equal at once |
| `static_seq s` | slot → slot | point the plain list slot at the static list `s`, with a room of 0 |
| `seq_addr s` | → addr | push the address of the cells of the static list `s`, the source of a `blit` into a counted list or a struct |
| `to_heap k` | slot addr → | move the cells of the counted list of kind `k` at `addr` to a new heap block, and write the plain list of them in the slot |
| `add_cell k` | list value → | give the plain list of kind `k` at `list` a block of its own when its room is 0, move it to a block twice the size (four cells at the least) when the block is full, then move the cell at `value` into a new last cell and leave zero octets of the cell kind at `value` |
| `remove_cell k` | list → | stop the program when the plain list of kind `k` at `list` holds no cells; else drop its last cell when the list owns its block (a room above 0), and take one from its count. For `text`, take the u8s of the last character off the count, and stop the program when the text holds no characters |
| `insert_cell k` | list i value → | stop the program when number `i` is past the count of the plain list of kind `k` at `list` (counted from 0, or from 1 when listed `insert_cell k from 1`); else make room for one more cell as `add_cell` does, move the cells from `i` on one cell on, and move the cell at `value` into cell `i` |
| `row_find k` | table index → n | push the number of the row of the ဇယား of kind `k` at `table` that holds the index at `index`, an `isize`, or -1 when no row holds it. The index is hashed, then compared as a whole value, with each NaN equal to each other one and -0.0 equal to 0.0 |
| `set_row k` | table index value → | give the ဇယား of kind `k` at `table` blocks of its own, then move the value at `value` into the row that holds the index at `index`, dropping its old value and the index given; with no such row, rebuild the slots when a third of them would no longer be free, and move both into a new last row. Each address given is left at zero octets |
| `remove_row k` | table index → | when a row of the ဇယား of kind `k` at `table` holds the index at `index`, give the ဇယား blocks of its own, drop the index and the value of the row, leave the row as zero octets and mark its slot removed; a ဇယား left with no live row gives back its blocks |
| `make_slice 8 of 3` | slot source start len → slot | write in the slot a plain list that reads `len` cells from cell `start` of the source, the first step of a [range](../language/lists.md#ranges); the source is a counted list of 3 cells (`of 3`), a plain list (`of slice`) or a heap block (`of block`), and a range past its end stops the program; `checked` skips the test. The generator then sets the room word of the slot to 0 and runs `own`, so the range gets its cells copied into a block of its own |
| `slice_cell 8` | list n → addr | check a cell number against the count of a plain list; push the address of the cell. A cell the compiler proves in range is `load` of the first word, then `ptr_add`, with no check |
| `object_cell 8` | box n → addr | push the address of the fields of an object, cell 0 of its box; stop the program with the `null object` trap when the box holds no object, a backstop: a shared object is never null, and the check of unset values guards each read it can prove |

### Text

| Instruction | Effect | Meaning |
| --- | --- | --- |
| `text t` | slot → slot | write the static text `t` in the text slot, with a room of 0 |
| `format fmt` | slot v… → slot | build the text of format `fmt` in a new heap block and write it in the slot, with a room equal to its count |
| `drop k` | addr → | for each text, plain list, object and closure inside the value of kind `k` whose room is above 0, take one off the count of its block; a block whose count reaches 0 is freed, its cells dropped in turn, and one still held is noted for the ring collector. Leave each blank |
| `dup_seqs k` | addr → | for each text, plain list, object and closure inside the value of kind `k`, add one to the count of its block, so the copied value shares it; a block of room 0 (lent), or a pinned block, is copied into a new heap block instead, its cells copied in turn. Static text is shared with no count |
| `deep_seqs k` | addr → | give the value of kind `k` a copied set of each shared object it reaches, at each depth, each object copied once (a table maps old objects to new ones), so a ring keeps its shape; text, plain lists and function values are shared as `dup_seqs` shares them |
| `lend_seqs k` | addr → | give each text and each plain list inside the value of kind `k` a room of 0, and set the lowest bit of each closure, so a parameter reads the caller's cells and runs the caller's closure with no count, and its drop frees nothing |
| `own k` | addr → | give each text and each plain list inside the value of kind `k` a block that it holds alone: one with cells and a room of 0, or whose block holds a count above 1, gets a new heap block with the same cells, the cells copied in turn. The op runs before each write, so a write never reaches a value that another place shares. The block of an object is never copied so |
| `pin` | addr → | mark the block of the text or plain list at `addr` as pinned, after `own`: unsafe code took the address of one of its cells. A later `dup_seqs` copies a pinned block, so a write through the pointer never reaches a copied value; a resize keeps the mark |
| `obj_to_ptr` | addr → ptr | push the address of the block of the object at `addr`, as a pointer that holds one more count of it: the conversion of an object to a pointer |
| `ptr_to_obj` | slot ptr → slot | write the object whose block the pointer names into the slot, which takes over the count the pointer held: the conversion of a pointer back to an object |
| `text_eq` | addr addr → b | whether two texts hold the same u8s |
| `char_count` | addr → n | the number of characters of the text at `addr`, a `usize` |
| `char_at` | addr i → c | the character at number `i` of the text at `addr`, counted from 0, or from 1 when listed `char_at from 1`; a number outside the text traps |
| `char_at_octet` | addr i → c | the character that starts at u8 number `i` of the text at `addr`, counted from 0; a number outside the text or inside a character traps |
| `set_char` | addr i c → | stop the program when number `i` is outside the text at `addr` (counted from 0, or from 1 when listed `set_char from 1`); else give the text a block of its own when its room is 0, and write the character `c` in place of character `i`. A character of another size shifts the u8s after it |
| `insert_text k` | addr i v → | stop the program when number `i` is past the character count of the text at `addr` (from 0, or from 1 when listed `insert_text k from 1`); else give the text a block of its own when its room is 0, and put the u8s of `v` (a `char`, or the address of `text`, as for `add_text`) in front of character `i` |
| `replace_text k` | addr a b v → | stop the program when cell `a` or `b` lies outside the text at `addr` (u8 cells from 0, or characters from 1 when listed `replace_text k chars`), when `b` comes before `a`, or when a cell edge falls inside a character; else give the text a block of its own when its room is 0, and put the u8s of `v` (as for `add_text`) in place of the range from `a` to `b`, both included |
| `add_text k` | addr v → | add the value `v` of kind `k`, a `char`, or the address of `text`, at the end of the text at `addr`. The added u8s are read first, so the text can be added to itself. The text takes a block of its own when its room is 0 |
| `text_edges` | addr → addr | trap when the range of text at `addr`, before its `own`, starts or ends inside a character; the range remains |
| `check_utf8` | addr → addr | trap when the u8s of the plain list at `addr` are not UTF-8; the list remains. It runs in the conversion of a list of u8s to text |

`format` pops one value for each value piece of the format, the last piece on top, and
then the address of the slot. A list, a struct or text comes as its address, each other
kind as itself. Each piece is written as a print writes it. When the heap has no room for
the text, `format` traps (`OutOfMem`). `drop` works on each kind that holds text or a plain list: `text`, a plain list,
a struct with such a field, a list of such cells, an object and a function value. A drop
of text whose room is 0, static or moved out, frees nothing.

No op runs the ring collector. A `drop` that leaves a block of an object, a closure or a
list that can reach one with a count above 0 notes the block as a candidate. When 1,024
candidates wait, and again at the end of the program, the VM runs trial deletion over them
and frees each ring that nothing outside it holds. A text or a list of numbers is never a
candidate. `zakac run --leaks` therefore lists none of these blocks.

`set_char` and `add_text` write the new u8s into the block of the text when it has room,
and else move the text to a block twice its size, 16 u8s at the least (or the new count,
when that is more). Text with a room of 0, static or lent to a parameter, takes a block
of its own first (as `own text` gives one), so an edit never reaches static text or the
caller's text.

The VM puts each static text in a frozen block of its own before `main` starts, so a
write into one traps ([When a program stops](traps.md)).

### Control

| Instruction | Effect | Meaning |
| --- | --- | --- |
| `jump t` | | go to instruction `t` of the function |
| `jump_if_false t` | bool → | go to `t` when the boolean is false |
| `jump_if_true t` | bool → | go to `t` when the boolean is true |
| `call f` | args → result | call function `f`; the last argument is on top |
| `fn_addr f` | → fn | push the [function value](../language/function-values.md) that calls function `f`: one word, `f + 1`, so 0 holds no function. The word of a plain function lies below the first record of a closure, so the two never meet |
| `closure f k` | addr → fn | make a [closure](../language/function-values.md): pop the address of a record of kind `k`, move the record to a new counted heap block and leave zeros behind, write `f + 1` into its first word, where `f` is the thunk that calls the nested function with the address of each capture, and push the address of the block as the function value |
| `call_value n` | fn args → result | take the function value from under the `n` arguments, then call the function it holds as `call` does; a value of 0 stops the program with the `null call` trap, a backstop, since a function value is never null. A closure is the address of its record, with the lowest bit set while it is lent to a call: the call reads the thunk from the first word and passes the address of the record as one more argument, the last |
| `host h` | slot addrs → slot | serve host function `h` (`read_line`, `read_file`, `write_file`, `write_stream`), the one op of an external function: pop the address of each parameter, with the address of the result slot under them, write the result there and leave its address. The VM reads each value through its address and frees none. `zakac run` serves it from the terminal and the files on disk; the books and the explorer serve it from storage (see [Host functions](../internals/codegen.md#host-functions)) |
| `foreign c` | values → result | make C call `c` of the table of C calls of the module (`foreign_calls`), the op of a [C function](../language/functions.md#c-functions) (`--emit asm` writes `foreign c0`, `foreign c1`, ...). A call row names the C function and the kind of each of its other values, so a C function that takes other values has a row for each shape of call. Pop the value of each parameter and then of each other value, the last on top, where a struct or a text is the address of its value (a text goes to C as a copied set of its u8s that ends in a zero octet, freed when the call ends). The generator passes a list of numbers as the address of its first cell, a plain pointer, so C reads or writes the cells in place. A struct result goes to the slot whose address lies under the values, and the address remains; another result is pushed; a function with no result pushes nothing. An address that lies in a live block reaches C as the native address of that octet. The first call opens each lib; a VM built with no `ffi`, such as the one in the explorer, stops at the call with the trap `C function not run` (see [When a program stops](traps.md#c-functions)) |
| `c_leave k` | fn → | give C leave to call the [closure](../language/function-values.md#nested-functions-and-closures) of the function value on top, a function of kind `k`, after the call that gave it: make the native closure of its record, once, and hold a count of the record until the leave ends. A leave of a value that holds a leave, and of a function with no captures does nothing |
| `c_store k at o` | addr value → | write a field of kind `k` at offset `o` of the struct C keeps, from unsafe code: a number or a boolean as it is, a counted list from the block whose address is the value. A guard has checked that the handle is open |
| `c_load k at o` | addr → value | read a field of kind `k` at offset `o` of the struct C keeps at the address on top, the address a [handle](../language/functions.md#c-functions) holds, in C's storage. A number or a boolean is pushed. For a counted list of numbers, the op pops the address of a block under it, writes the cells there, and pushes that address; for a text field, a `char *`, it does the same with the string copied up to its zero octet, as an optional of text, with no data for NULL. A guard of the wrapper has checked that the handle is open (`halt closed_handle`) |
| `c_revoke` | fn → | take back the leave of the function value on top and the count it held; a revoke of a closure with no leave does nothing. |
| `ret` | | leave the function; a result remains on the stack |
| `trap step_not_positive` | | stop the program: a counted loop step is zero or negative |
| `halt cause [k]` | v → | stop the program with the trap that `cause` names: `not_good` (a good result was read from a result that holds an error) pops the value `v` of kind `k` and shows it as a print would, and for a list, a struct or text the operand is its address; when the failure holds no value, `not_good` has no `k` and no operand, and the trap shows the bare message `the result holds a failure, not a good result`; `not_fault` (an error value was read from a good result) has no `k` and no operand; `no_data` (the data was read from an optional that holds none) has no `k` and no operand; `no_row` (a statement wrote inside the value of a row of a ဇယား that no row holds) has no `k` and no operand; `closed_handle` (a handle that a C function took over was used) has no `k` and no operand; `null_from_c` (C gave NULL for a handle result written with no optional) has no `k` and no operand, and stops with the trap `null from C` |

`call` takes room for the locals of the callee in the stack region and moves the
arguments into its parameters; `ret` gives the room back. A call nested deeper than
`--max-depth` traps. `halt` ends the run for a read of the wrong part of a [whole
result](../language/error-handling.md#reading-a-whole-result), `not_good` and `not_fault` (see
[When a program stops](traps.md#a-whole-result-read-as-the-wrong-part)), and for a read of the data of an
[optional](../language/optionals.md) that holds none, `no_data` (see
[an optional read with no data](traps.md#an-optional-read-with-no-data)).

### Output

| Instruction | Effect | Meaning |
| --- | --- | --- |
| `print` | v → | write the value and a line break |
| `print_text t` | | write the static text `t` and a line break |
| `print_list k` | addr → | write the cells of a list, the fields of a struct or an object, or the text, of kind `k`, and a line break; an object that is still being written further up, in a ring, is written as `…` |
| `print_enum k` | v → | write the member of enumeration `k` whose value `v` is (the address of the value for a list, a struct or text) as `<stem>များထဲမှ <member>`, and a line break; a value no member has as its value. For a function kind `k`, write the function value as `<name> ဟူသော လုပ်ဆောင်ချက်`, or the phrase of the null pointer for a word of 0 (a closure names the function its thunk calls) |

How each value reads is in [Running a program](running.md#printing).

## How sentences become instructions

| Sentence | Instructions |
| --- | --- |
| read a variable | `x; load k` |
| `x ကို v ဟု ထားမည်` | `x; v; store k` |
| address of a variable | `x` |
| `a နှင့် b ကို ပေါင်းသည့်ရလဒ်` | `a; b; add` |
| `v ကို ပြသမည်` | `v; print` |
| call as a statement | `args; call f`, and `pop` when the callee gives a result no one uses |
| call through a function value | `fn; args; call_value n` |
| declaration of a nested function with captures | `record; closure f k`, kept in a hidden local |
| `a နှင့် b သည် တစ်ခုတည်း ဖြစ်…` (objects or function values) | the block address of each, `load ptr`, then `eq` |
| `x ၏ မိတ္တူ` (an object) | the fields of `x` (`object_cell`) into a slot, `dup_seqs` on them, then `to_heap` into a new object |
| `x တစ်ခုလုံး ၏ မိတ္တူ` | `blit k` into a slot, then `deep_seqs k` |
| `x ၏ အကွက် s မှစ၍ n ကွက်` | `slot; x; s; n; make_slice …`, the room word set to 0, then `own k` |
| `l ၏ အကွက် i ၏ လိပ်စာ` (unsafe code) | `l; dup; own k; dup; pin; i; slice_cell s` |
| an object converted to a pointer, and back | `x; obj_to_ptr`, and `slot; p; ptr_to_obj` |
| `p ၏ တန်ဖိုး` | `p; load k` |
| `p + i` | `p; i; ptr_add s` |
| allocate | `x; count; alloc s; store k*` |
| resize | `x; x; load k*; count; realloc s; store k*` |
| free | `x; x; load k*; free; push 0x0:ptr; store k*` |
| `လိပ်စာ s ရှိ n ခုစာ မင်မိုရီကို လိပ်စာ d ဆီသို့ ကူးယူမည်` | `s; n; d; mem_blit size`, and `mem_move size` for `ပြောင်းထည့်မည်` |
| `လိပ်စာ d ရှိ မင်မိုရီကို v ဖြင့် n ခုစာ ဖြည့်မည်` | `d; v; n; mem_fill k` |
| an `အပိုင်အနေဖြင့်` argument | `x; take k*`, which leaves the null pointer in the caller's place |
| a string literal in a text slot | `x; text t; pop` |
| a template literal, or a `+` join | `x; v…; format fmt; pop` |
| `t ကို v ဟု ထားမည်` for text | `v` into the hidden slot `%new`; `t; drop text`; then the text of `%new` moves into `t` (`blit text`, and `zero text` on `%new`) |
| `t ၏ စာလုံး i ကို c ဟု ထားမည်` | `t; i; c; set_char`, or `set_char from 1` for `၏ i ခုမြောက်စာလုံး` |
| `t ထဲသို့ v ကို ထည့်မည်` for text | `t; v; add_text k`; text that `v` builds when it runs, such as a format, is dropped right after |
| `l ၏ အကွက် i ၏ အရှေ့တွင် v ကို ထည့်မည်` | `v` into the scratch slot `%cell`; `l; i; %cell; insert_cell k` |
| `t ၏ စာလုံး i ၏ အရှေ့တွင် v ကို ထည့်မည်` | `t; i; v; insert_text k`, dropped as for `add_text` |
| `t ၏ နောက်ဆုံးစာလုံးကို ဖျက်မည်` | `t; remove_cell text` |
| `z ထဲမှ အညွှန်း i ၏ တန်ဖိုး ကို v ဟု ထားမည်` | `i` and `v` into scratch slots `%cell`; `z; %cell; %cell; set_row k` |
| `z ထဲမှ အညွှန်း i ကို ဖျက်မည်` | `z; i; remove_row k`, a scalar index in a scratch slot first |
| `z ထဲမှ အညွှန်း i ၏ တန်ဖိုး`, read | `z; i; row_find k` into a hidden slot; a test of it builds the optional from the row, or the optional with no data |
| a write inside the value of a row | `row_find k` as for a read; `halt no_row` when it gives -1; then the address of the value through the rows, `own` first, as for a cell |
| `t ၏ အကွက် a မှ b ထိကို v ဖြင့် အစားထိုးမည်` | `t; a; b; v; replace_text k`, dropped as for `add_text` |
| the end of a text variable | `x; drop text` |
| `a ဖြစ်၍ b` (and) | `a; jump_if_false F; b; jump E; F: push မမှန်; E:` |
| `a သို့မဟုတ် b` (or) | `a; jump_if_true T; b; jump E; T: push မှန်; E:` |
| if and else | `cond; jump_if_false Else; then; jump End; Else: else; End:` |
| a loop | `Head: cond; jump_if_false Exit; block; jump Head; Exit:` |

Here `x` stands for `local_addr n` or `global_addr n`.

A counted loop evaluates its two ends and its step once, in that order, into slots when
those are not constants. It never moves the counter past the last value, so a counter
cannot wrap. It works out the distance left with `wrap sub`, which gives the true
distance for ends of each sign, and steps with `wrap add` or `wrap sub`, so
`--overflow stop` does not change it. A step that is not a constant is checked once, before the first round, with
`trap step_not_positive`.

### A heap block in instructions

A block from the heap, written to, read and given back in an unsafe block (see [Unsafe
Zaka](../language/unsafe.md)), and the instructions of its start block. An unsafe block
adds no instruction of its own:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားမည်။
ပ + ၁ ၏ တန်ဖိုး ကို ၉ ဟု ထားမည်။
ပ + ၁ ၏ တန်ဖိုး ကို ပြသမည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၉
```

```console
fn f0 ပရိုဂရမ် (params 0)
locals
  l0 ပ isize*
     0  local_addr l0        ; ပ
     1  push 0x0:ptr
     2  store isize*         ; ပ holds zero octets until the next write
     3  local_addr l0        ; ပ
     4  push 2:usize
     5  alloc 8              ; 2 elements of 8 octets
     6  store isize*         ; keep the address in ပ
     7  local_addr l0        ; ပ
     8  load isize*
     9  push 7:isize
    10  store isize          ; ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားမည်
    11  local_addr l0        ; ပ
    12  load isize*
    13  push 1:isize
    14  ptr_add 8            ; ပ + ၁
    15  push 9:isize
    16  store isize
    17  local_addr l0        ; ပ
    18  load isize*
    19  push 1:isize
    20  ptr_add 8
    21  load isize
    22  print
    23  local_addr l0        ; ပ
    24  local_addr l0        ; ပ
    25  load isize*
    26  free
    27  push 0x0:ptr
    28  store isize*         ; the free statement clears ပ
    29  ret
```

The comments after `;` in this listing were added to explain it; `zakac` prints the
variable names alone. Two instructions name a kind that the others leave out: `load` and
`store` name the kind read or written, and `alloc` names the octet size of one
element, so the VM can check the access and the cap.

## Where to look next

- [When a program stops](traps.md) tells what each instruction does when its check fails.
- [IR and code](../internals/codegen.md) tells how the code generator makes these
  instructions.
