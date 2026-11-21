---
title: Instructions
description: The operations of the Zaka VM in groups, with a program shown beside its listing.
---

# Instructions

The code generator turns each function of a program into a list of operations
(instructions), and the VM (virtual machine) runs them one at a time. This page shows how
to read a listing of a program and then lists each operation in groups.

The operations belong to the compiler and the VM as a pair, and the set can change between
versions, so a listing made with one version of `zakac` is for that version. The rules of
the language do not depend on them: [the language](../language/README.md) sets what a
program means, and the operations are one means to run it.

## Reading a listing

`zakac build --emit asm` writes the listing of a program in place of the artifact. Here is
a function that doubles its parameter, and a program start block (entry point) that calls
it:

```zaka
နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် သည်
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
     0  local_addr l0        ; န
     1  load isize
     2  push 2:isize
     3  mul
     4  ret

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

| In the listing | Meaning |
| --- | --- |
| `g0`, `l1`, `f0`, `t0`, `fmt0`, `s0` | the index of a global, a local, a function, a static text, a format and a static list; the source name after `;` is a note for us |
| `L0` | the label of a jump target; the artifact has the index of the target operation instead |
| `locals` | the locals of the function, parameters first (`l0` is the parameter `န`), then the locals of the source, then slots the generator adds, such as `%list`, `%slice`, `%to`, `%step`, `%dir`, `%text`, `%cell` and `%new` |
| `isize`, `usize`, `u8`, `i32`, `f64`, `char`, `bool` | data kinds (types) in short form; `isize` and `usize` are the integers of the platform width |
| `isize*`, `isize[3]`, `isize[]`, `text`, `အဆစ်@` | an address (pointer) of an integer, a counted list of three integers, a plain list, text (string), and a shared object of the struct `အဆစ်` |
| `21:isize` | a value with its data kind |

The program start block is a function named `ပရိုဂရမ်`. The last function is `main`: it
sets the starting values of the top-level variables, calls the program start block, and
frees their text at the end.

No operation belongs to a variable alone. To read `က`, the code runs `local_addr l0` to push
the address of its block (allocation), or `global_addr` for a top-level variable, and then
`load isize` to read it. To write it, the code pushes the address and the value and then
runs `store`. This is what gives each variable an address. A call such as `call f0` moves
its arguments from the operand stack into the parameters of the callee, and `ret` leaves
the result on the operand stack for the caller, where `print` pops it here.

### Texts and formats

A program with text has two more tables before its functions. `texts` has each static
text once: string literals, formats whose template slots folded, and printed text.
`formats` has the pieces of each text built while the program runs: a static text `t1`,
or the data kind of a value that the program supplies (`isize`, `text`, `isize[3]`, ...).

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

`text t0` points `အမည်` at the static text, with no heap block. The template has a variable
in it, so `format fmt0` builds it in the scratch slot `%text` from the address of the text
`အမည်` and the value of `အသက်`. The print drops the slot right after, and the end of the
program drops `အမည်`, which points at static text and frees nothing.

### Static lists

A list literal whose cells are all known when the program is compiled adds one more table,
`seqs`, with each static list, its data kind and its cells. A text cell names a static text
(`t0`), and a plain list cell names an earlier static list (`s0`). The VM puts each one in a
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

### The artifact

The artifact of `zakac build` has the same operations as data, raw (`.zkc`) or as text
(`.zkct`, a first line and then YAML). We pick the form with `--emit zkc` or `--emit zkct`,
or with the file name in `-o`, and `zakac run` runs either form on the VM.

## The operations

The tables use `→` for the effect on the operand stack: the inputs, with the top
rightmost, then the outputs. `k` is a data kind, and `s` the size of one element in octets
(bytes).

### Values and addresses

| Operation | Effect | Meaning |
| --- | --- | --- |
| `push v` | → v | push a constant |
| `local_addr n` | → addr | the address of the block of local `n` of the running call |
| `local_ref n` | → addr | the address that local `n` contains: the list or struct of the caller, for a parameter the function reads in place |
| `global_addr n` | → addr | the address of the block of global `n` |
| `result_addr` | → addr | the address of the slot of the caller for the list or struct result of the running call |
| `dup` | v → v v | push the top value again |
| `pop` | v → | drop the top value |

### Operators

| Operation | Effect | Meaning |
| --- | --- | --- |
| `neg`, `bnot`, `not` | a → r | minus, bitwise flip, logical not |
| `add`, `sub`, `mul`, `div`, `rem` | a b → r | arithmetic; an integer result outside its data kind stops the program, or wraps to the width when the module asks for that (`--overflow wrap`) |
| `wrap op` | a b → r | `op` on integers, wrapped to their width in both modes; a counted loop uses it for its own steps |
| `band`, `bor`, `bxor` | a b → r | bitwise and, or, exclusive or |
| `shl`, `shr` | a n → r | shifts |
| `eq`, `ne`, `lt`, `gt`, `le`, `ge` | a b → bool | comparisons |
| `convert k` | a → a as k | convert a value to data kind `k` |

`div` and `rem` stop the program on a zero divisor, and `shl` and `shr` stop it when the
distance is not below the bit width. The right operand is on top of the stack, so `sub`
computes the value below the top minus the top. Decimal numbers follow IEEE 754 with ties
to even.

### Storage

| Operation | Effect | Meaning |
| --- | --- | --- |
| `load k` | addr → v | read a value of data kind `k` |
| `store k` | addr v → | write a value of data kind `k` |
| `take k` | addr → v | read the value, then leave zero octets of the data kind there (the null address for an address): a move at a last use, or the hand-over of an `အပိုင်အနေဖြင့်` argument |
| `ptr_add s` | addr n → addr | move an address forward `n` elements of `s` octets |
| `ptr_sub s` | a b → n | the number of elements between two addresses, an `isize` |
| `alloc s` | n → addr | a new heap block of `n` elements, or the null address when there is no room |
| `realloc s` | addr n → addr | resize a heap block; the block can move |
| `free` | addr → | free a heap block; the null address does nothing |
| `mem_blit s` | src count dst → | write `count` cells of `s` octets (1 for octets) from `src` to `dst`; the ranges must not overlap, or the program stops |
| `mem_move s` | src count dst → | the same, with the ranges free to overlap: the source is read in full before the write |
| `mem_fill k` | dst value count → | set `count` cells at `dst` to `value`, a value of data kind `k` |

Each of these checks its address as [Addresses and blocks](blocks.md) describes, and can
stop the program. The compiler emits `ptr_add`, `ptr_sub`, the heap operations, and the
`load` and `store` through an address, for unsafe code alone ([Unsafe
Zaka](../language/unsafe.md)), and the VM checks them all the same. `ptr_add` also stops
the program on a step from the null address and on a step out of the block it starts in.
`mem_blit`, `mem_move` and `mem_fill` check that the source range lies in one live block
and the destination range in one live block that can be written. A count of 0 checks
nothing, and `mem_blit` stops the program with [overlap](traps.md#all-the-traps) when its
two ranges share an octet.

### Lists

A list lives in a block, and its operations work on addresses. A counted list is its cells,
inline. A plain list is three words, as text is: the address of its first cell, its count,
and the room of its heap block, and its cells sit in a static block or a heap block. A heap
block of text or of a plain list counts the values that share it.

| Operation | Effect | Meaning |
| --- | --- | --- |
| `index 3`, `index 3 from 1` | n → i | check a cell number against a list of 3 cells, whose first cell is numbered 0 (or 1, after `from`); push it counted from 0 |
| `blit k` | to from → | duplicate (copy) a value of data kind `k`, such as a whole list, from one address to another |
| `zero k` | addr → | set the octets of a value of data kind `k` to zero |
| `list_eq k` | a b → bool | whether each cell of one counted list, or each field of one struct, equals the cell or field of the other |
| `seq_eq k` | addr addr → b | whether two plain lists with cells of data kind `k` have the same count of cells, each equal; a shared object is a list of one cell, so this is also the deep `==` of objects. A pair of blocks that a compare further up is still comparing counts as equal, so a ring ends, and one block on both sides is equal at once |
| `static_seq s` | slot → slot | point the plain list slot at the static list `s`, with a room of 0 |
| `seq_addr s` | → addr | push the address of the cells of the static list `s`, the source of a `blit` into a counted list or a struct |
| `to_heap k` | slot addr → | move the cells of the counted list of data kind `k` at `addr` to a new heap block, and write the plain list of them in the slot |
| `add_cell k` | list value → | give the plain list of data kind `k` at `list` a block when its room is 0, move it to a block twice the size (four cells at the least) when the block is full, then move the cell at `value` into a new last cell and leave zero octets of the cell kind at `value` |
| `remove_cell k` | list → | stop the program when the plain list at `list` has no cells; else drop its last cell when the list owns its block (a room above 0), and take one from its count. For `text`, take the u8s of the last character off the count, and stop the program when the text has no characters |
| `insert_cell k` | list i value → | stop the program when number `i` is past the count of the plain list at `list` (counted from 0, or from 1 when listed `insert_cell k from 1`); else make room for one more cell as `add_cell` does, move the cells from `i` on one cell along, and move the cell at `value` into cell `i` |
| `row_find k` | table index → n | push the number of the row of the table (map) of data kind `k` at `table` that has the index (key) at `index`, an `isize`, or -1 when no row has it. The index is hashed, then compared as a whole value, with each NaN equal to each other one and -0.0 equal to 0.0 |
| `set_row k` | table index value → | give the table at `table` blocks of their own, then move the value at `value` into the row that has the index at `index`, dropping its old value and the index given; with no such row, rebuild the slots when a third of them would no longer be free, and move both into a new last row. Each address given is left at zero octets |
| `remove_row k` | table index → | when a row of the table at `table` has the index at `index`, give the table blocks of their own, drop the index and the value of the row, leave the row as zero octets and mark its slot removed; a table left with no live row frees its blocks |
| `make_slice 8 of 3` | slot source start len → slot | write in the slot a plain list that reads `len` cells from cell `start` of the source, the first step of a [range](../language/lists.md#ranges); the source is a counted list of 3 cells (`of 3`), a plain list (`of slice`) or a heap block (`of block`), and a range past its end stops the program; `checked` skips the test. The generator then sets the room word of the slot to 0 and runs `own`, so the range gets a copied set of its cells in a new block |
| `slice_cell 8` | list n → addr | check a cell number against the count of a plain list, and push the address of the cell. A cell that the compiler proves in range is a `load` of the first word, then `ptr_add`, with no check |
| `object_cell 8` | box n → addr | push the address of the fields of an object, cell 0 of its box; stop the program with the `null object` trap when the box has no object, a backstop, since a shared object is never null and the check of unset values guards each read it can prove |

### Text

| Operation | Effect | Meaning |
| --- | --- | --- |
| `text t` | slot → slot | write the static text `t` in the text slot, with a room of 0 |
| `format fmt` | slot v… → slot | build the text of format `fmt` in a new heap block and write it in the slot, with a room equal to its count |
| `drop k` | addr → | for each text, plain list, object and closure inside the value of data kind `k` whose room is above 0, take one off the count of its block; a block whose count reaches 0 is freed, its cells dropped in turn, and one still in use is noted for the ring collector. Leave each blank |
| `dup_seqs k` | addr → | for each text, plain list, object and closure inside the value of data kind `k`, add one to the count of its block, so the copied value shares it; a block of room 0 (lent), or a pinned block, is copied into a new heap block instead, its cells copied in turn. Static text is shared with no count |
| `deep_seqs k` | addr → | give the value of data kind `k` a copied set of each shared object it reaches, at each depth, each object copied once (a table maps old objects to new ones), so a ring keeps its shape; text, plain lists and function values are shared as `dup_seqs` shares them |
| `lend_seqs k` | addr → | give each text and each plain list inside the value of data kind `k` a room of 0, and set the lowest bit of each closure, so a parameter reads the cells of the caller and runs the closure of the caller with no count, and its drop frees nothing |
| `own k` | addr → | give each text and each plain list inside the value of data kind `k` a block that no other value shares: one with cells and a room of 0, or whose block has a count above 1, gets a new heap block with the same cells, the cells copied in turn. The operation runs before each write, so a write never reaches a value that another place shares. The block of an object is never copied in this manner |
| `pin` | addr → | mark the block of the text or plain list at `addr` as pinned, after `own`, because unsafe code took the address of one of its cells. A later `dup_seqs` copies a pinned block, so a write through the address never reaches a copied value; a resize keeps the mark |
| `obj_to_ptr` | addr → ptr | push the address of the block of the object at `addr`, as an address that counts as one more user of the block: the conversion of an object to an address |
| `ptr_to_obj` | slot ptr → slot | write the object whose block the address names into the slot, which takes over the count of the address: the conversion of an address back to an object |
| `text_eq` | addr addr → b | whether two texts have the same u8s |
| `char_count` | addr → n | the number of characters of the text at `addr`, a `usize` |
| `char_at` | addr i → c | the character at number `i` of the text at `addr`, counted from 0, or from 1 when listed `char_at from 1`; a number outside the text stops the program |
| `char_at_octet` | addr i → c | the character that starts at u8 number `i` of the text at `addr`, counted from 0; a number outside the text or inside a character stops the program |
| `set_char` | addr i c → | stop the program when number `i` is outside the text at `addr` (counted from 0, or from 1 when listed `set_char from 1`); else give the text a block when its room is 0, and write the character `c` in place of character `i`. A character of another size shifts the u8s after it |
| `insert_text k` | addr i v → | stop the program when number `i` is past the character count of the text at `addr` (from 0, or from 1 when listed `insert_text k from 1`); else give the text a block when its room is 0, and put the u8s of `v` (a `char`, or the address of `text`, as for `add_text`) in front of character `i` |
| `replace_text k` | addr a b v → | stop the program when cell `a` or `b` lies outside the text at `addr` (u8 cells from 0, or characters from 1 when listed `replace_text k chars`), when `b` comes before `a`, or when a cell edge falls inside a character; else give the text a block when its room is 0, and put the u8s of `v` (as for `add_text`) in place of the range from `a` to `b`, both included |
| `add_text k` | addr v → | add the value `v` of data kind `k`, a `char` or the address of `text`, at the end of the text at `addr`. The added u8s are read first, so the text can be added to itself. The text gets a block when its room is 0 |
| `text_edges` | addr → addr | stop the program when the range of text at `addr`, before its `own`, starts or ends inside a character; the range remains on the stack |
| `check_utf8` | addr → addr | stop the program when the u8s of the plain list at `addr` are not UTF-8; the list remains on the stack. It runs in the conversion of a list of u8s to text |

`format` pops one value for each value piece of the format, the last piece on top, and
then the address of the slot. A list, a struct or text comes as its address, and each other
data kind as itself. Each piece is written as a print writes it. When the heap has no room
for the text, `format` stops the program with `out of mem`.

`drop` works on each data kind that contains text or a plain list: `text`, a plain list, a
struct with such a field, a list of such cells, an object and a function value. A drop of
text whose room is 0, static or moved out, frees nothing. A `drop` that leaves a block with
a count above 0 notes the block as a candidate of the ring collector when it can be part of
a ring, that is, when the blocks its cells point at lead back to a block of its data kind,
or contain a closure. When the candidates reach 16,384 octets (or the threshold that a
`rings threshold` operation set), and again at the end of the program, the VM runs trial
deletion over them and frees each ring that nothing outside it points at. A text or a list
of numbers is never a candidate. The `rings` operation steers the collector, and
`zakac run --leaks` lists none of these blocks.

`set_char` and `add_text` write the new u8s into the block of the text when it has room,
and else move the text to a block twice its size, 16 u8s at the least (or the new count,
when that is more). Text with a room of 0, static or lent to a parameter, first gets a
block (as `own text` gives one), so an edit never reaches static text or the text of the
caller. The VM puts each static text in a frozen block before `main` starts, so a write into
one stops the program ([Stops in unsafe code](traps-unsafe.md#addresses)).

### Control

| Operation | Effect | Meaning |
| --- | --- | --- |
| `jump t` | | go to operation `t` of the function |
| `jump_if_false t` | bool → | go to `t` when the true/false value is false |
| `jump_if_true t` | bool → | go to `t` when the true/false value is true |
| `call f` | args → result | call function `f`; the last argument is on top |
| `fn_addr f` | → fn | push the [function value](../language/function-values.md) that calls function `f`: one word, `f + 1`, so 0 is no function. The word of a plain function lies below the first record of a closure, so the two never meet |
| `closure f k` | addr → fn | make a [closure](../language/function-values.md): pop the address of a record of data kind `k`, move the record to a new counted heap block and leave zeros behind, write `f + 1` into its first word, where `f` is the thunk that calls the nested function with the address of each capture, and push the address of the block as the function value |
| `call_value n` | fn args → result | take the function value from under the `n` arguments, then call the function it names as `call` does; a value of 0 stops the program with the `null call` trap, a backstop, since a function value is never null. A closure is the address of its record, with the lowest bit set while it is lent to a call: the call reads the thunk from the first word and passes the address of the record as one more argument, the last |
| `ret` | | leave the function; a result remains on the stack |
| `trap step_not_positive` | | stop the program: the step of a counted loop is zero or negative |
| `halt cause [k]` | [v] → | stop the program with the trap that `cause` names; see the table below |
| `rings act` | [n] → | steer the ring collector: `collect` runs it now; `pause` stops the runs that a full list of candidates starts, while the end of the program still collects; `resume` starts them again and collects at once when the list has reached the threshold; `threshold` pops a `usize` `n` and makes `n` octets of candidates start a run, collecting at once when the list has that much and the runs are not paused |
| `ring_stat n` | → value | push one stat of the ring collector, for its field `n` in the struct of the core module: 0 the collections that ran, 1 the octets the collections freed, 2 the octets of the candidates, 3 the threshold, each an `isize`; 4 whether the runs are paused, a true/false value |

`call` puts the locals of the callee in the stack region and moves the arguments into its
parameters, and `ret` frees that room again. A call nested deeper than `--max-depth` stops
the program.

`halt` ends the run for a failure that the program did not handle. The cause names the
trap, and a cause with a data kind first pops a value to show:

| Cause | When | Operand | Trap |
| --- | --- | --- | --- |
| `not_good` | the good result was read from a result that has an error value | the error value, of data kind `k` (its address for a list, a struct or text), shown as a print shows it | [not good](traps.md#a-whole-result-read-as-the-wrong-part) |
| `not_good` with no `k` | the good result was read from a failure with no value | none | not good, with the bare message `the result holds a failure, not a good result` |
| `not_fault` | the error value was read from a good result | none | [not fault](traps.md#a-whole-result-read-as-the-wrong-part) |
| `no_data` | the data was read from a blank [optional](../language/optionals.md) | none | [no data](traps.md#an-optional-read-with-no-data) |
| `no_row` | a statement wrote inside the value of a row of a table that no row has | none | no row |
| `closed_handle` | a handle that a C function took over was used | none | closed handle |
| `null_from_c` | C returned NULL for a handle result written with no optional | none | null from C |
| `step_not_positive` | the step of a parallel counted loop is zero or negative | none | step not positive |
| `threads_not_positive` | a parallel loop or map was split into zero or fewer threads | none | threads not positive |

A [whole result](../language/reading-results.md#reads) is explained in the language pages.

### Threads

| Operation | Effect | Meaning |
| --- | --- | --- |
| `spawn f` | args → thread | start a [thread](../language/threads.md) that calls function `f`: the arguments leave the stack, the thread enters `f` at once with a duplicate of each list, struct and text argument, in a new stack region, and the `usize` that names the thread is pushed. The thread runs in its own turns |
| `wait [k]` | [slot] thread → [result] | pop the `usize` that names a thread and wait for its end: a thread that has not ended passes the turn to the next thread, and the operation runs again later. A result of data kind `k` is pushed, or for a list, a struct or text it goes to the slot under the thread, whose address remains, sharing what it points at; with no `k`, nothing remains. A thread that stopped stops the waiter with the trap `thread trapped` |
| `pipe_new [n]` | → pipe | push the `usize` that names a new [channel](../language/threads.md#channels), where `n` values wait at most, or with no limit |
| `pipe_send k` | pipe addr → | pop the address of a value of data kind `k` and a channel, and move the value into a new heap block at the back of the queue, leaving the initial value of `k` in its place; a full channel passes the turn to the next thread and the operation runs again later, and a closed one stops the program with the trap `pipe closed` |
| `pipe_receive k` | slot pipe → slot | pop a channel and write its next value as an optional of `k` to the slot under it, moving the value out of the front block and freeing the block; an open channel with no value passes the turn to the next thread and the operation runs again later, and a closed and drained one writes no data |
| `pipe_close` | pipe → | close a channel; a second close stops the program with the trap `pipe closed` |
| `lock` | obj → | pop the variable of a shared object and take the lock of its block: a lock that another thread has passes the turn to the next thread, and the operation runs again later; a thread that has the lock takes it once more; a broken lock stops the program with the trap `lock broken` |
| `unlock` | obj → | pop the variable of a shared object and release the lock of its block once |
| `atomic_add [sub] k` | addr value → | add the value to the atomic number of integer data kind `k` at the address, or subtract it with `sub`, in one operation; the result wraps |
| `atomic_cas k` | addr expected new → old | set the atomic number of integer data kind `k` at the address to `new` when it is `expected`, in one operation, and push the value it had before |

### Host functions and C

| Operation | Effect | Meaning |
| --- | --- | --- |
| `host h` | slot addrs → slot | serve host function `h`, the one operation of an [external function](../language/functions-c.md#external-functions): pop the address of each parameter, with the address of the result slot under them, write the result there and leave its address. The VM reads each value through its address and frees none |
| `foreign c` | values → result | make C call `c` of the table of C calls of the module (`foreign_calls`), the operation of a [C function](../language/functions-c.md#c-functions); see below |
| `c_leave k` | fn → | give C leave to call the [closure](../language/closures.md#closures-and-c) of the function value on top, a function of data kind `k`, after the call that gave it: make the native closure of its record, once, and keep a count of the record until the leave ends. A leave of a value that has a leave, and of a function with no captures, does nothing |
| `c_revoke` | fn → | take back the leave of the function value on top and the count it kept; a revoke of a closure with no leave does nothing |
| `c_store k at o` | addr value → | write a field of data kind `k` at offset `o` of the struct C keeps, from unsafe code: a number or a true/false value as it is, a counted list from the block whose address is the value. A guard has checked that the handle is open |
| `c_load k at o` | addr → value | read a field of data kind `k` at offset `o` of the struct that C keeps at the address on top, the address a [handle](../language/functions-c.md#data-kinds-from-outside-the-program) points at, in the storage of C. A number or a true/false value is pushed. For a counted list of numbers, the operation pops the address of a block under it, writes the cells there, and pushes that address; for a text field, a `char *`, it does the same with the string copied up to its zero octet, as an optional of text, with no data for NULL. A guard of the wrapper has checked that the handle is open (`halt closed_handle`) |

The host functions are `read_line`, `read_file`, `write_file`, `write_stream`,
`append_file`, `remove`, `make_folder`, `list_folder`, `path_kind`, `rename` and
`read_env`. `zakac run` serves them from the terminal and the files on disk, and the books
and the Explorer serve them from storage (see [Host
functions](../internals/vm-internals.md#host-functions)).

For `foreign`, each row of the table of C calls names the C function and the data kind of
each of its extra values, so a C function with a variable count of arguments has a row
for each shape of call (`--emit asm` writes `foreign c0`, `foreign c1`, ...):

- The operation pops the value of each parameter and then of each extra value, the last
  on top. A struct or text comes as the address of its value, and text goes to C as a
  copied set of its u8s that ends in a zero octet, freed when the call ends.
- The generator passes a list of numbers as the address of its first cell, a plain
  address, so C reads or writes the cells in place. An address that lies in a live block
  reaches C as the native address of that octet.
- A struct result goes to the slot whose address lies under the values, and the address
  remains; another result is pushed, and a function with no result pushes nothing.
- The first call opens each lib. A VM built with no `ffi`, such as the one in the Explorer,
  stops the program at the call with the trap `C function not run` (see [Stops in unsafe
  code](traps-unsafe.md#c-functions)).

### Output

| Operation | Effect | Meaning |
| --- | --- | --- |
| `print` | v → | write the value and a line break |
| `print_text t` | | write the static text `t` and a line break |
| `print_list k` | addr → | write the cells of a list, the fields of a struct or an object, or the text, of data kind `k`, and a line break; an object that is still being written further up, in a ring, is written as `…` |
| `print_enum k` | v → | write the member of enumeration `k` whose value is `v` (the address of the value for a list, a struct or text) as `<stem>များထဲမှ <member>`, and a line break. For a function kind `k`, write the function value as `<name> ဟူသော လုပ်ဆောင်ချက်`, or the phrase of the null address for a word of 0 (a closure names the function its thunk calls) |

[Running a program](running.md#printing) shows how each value prints.

## How sentences become operations

| Sentence | Operations |
| --- | --- |
| read a variable | `x; load k` |
| `x ကို v ဟု ထားမည်` | `x; v; store k` |
| address of a variable | `x` |
| `a နှင့် b ၏ ပေါင်းလဒ်` | `a; b; add` |
| `v ကို ပြသမည်` | `v; print` |
| call as a statement | `args; call f`, and `pop` when the callee returns a result that nothing uses |
| call through a function value | `fn; args; call_value n` |
| declaration of a nested function with captures | `record; closure f k`, kept in a hidden local |
| `a နှင့် b သည် တစ်ခုတည်း ဖြစ်…` (objects or function values) | the block address of each, `load ptr`, then `eq` |
| `x ၏ မိတ္တူ` (an object) | the fields of `x` (`object_cell`) into a slot, `dup_seqs` on them, then `to_heap` into a new object |
| `x တစ်ခုလုံး ၏ မိတ္တူ` | `blit k` into a slot, then `deep_seqs k` |
| `x ၏ အကွက် s မှစ၍ n ကွက်` | `slot; x; s; n; make_slice …`, the room word set to 0, then `own k` |
| `l ၏ အကွက် i ၏ လိပ်စာ` (unsafe code) | `l; dup; own k; dup; pin; i; slice_cell s` |
| an object converted to an address, and back | `x; obj_to_ptr`, and `slot; p; ptr_to_obj` |
| `p ၏ တန်ဖိုး` | `p; load k` |
| `p + i` | `p; i; ptr_add s` |
| allocate | `x; count; alloc s; store k*` |
| resize | `x; x; load k*; count; realloc s; store k*` |
| free | `x; x; load k*; free; push 0x0:ptr; store k*` |
| `လိပ်စာ s ရှိ n ခုစာ မင်မိုရီကို လိပ်စာ d ဆီသို့ ကူးယူမည်` | `s; n; d; mem_blit size`, and `mem_move size` for `ပြောင်းထည့်မည်` |
| `လိပ်စာ d ရှိ မင်မိုရီကို v ဖြင့် n ခုစာ ဖြည့်မည်` | `d; v; n; mem_fill k` |
| an `အပိုင်အနေဖြင့်` argument | `x; take k*`, which leaves the null address in the place of the caller |
| a string literal in a text slot | `x; text t; pop` |
| a template literal, or a `+` join | `x; v…; format fmt; pop` |
| `t ကို v ဟု ထားမည်` for text | `v` into the hidden slot `%new`; `t; drop text`; then the text of `%new` moves into `t` (`blit text`, and `zero text` on `%new`) |
| `t ၏ စာလုံး i ကို c ဟု ထားမည်` | `t; i; c; set_char`, or `set_char from 1` for `၏ i ခုမြောက်စာလုံး` |
| `t ထဲသို့ v ကို ထည့်မည်` for text | `t; v; add_text k`; text that `v` builds as it runs, such as a format, is dropped right after |
| `l ၏ အကွက် i ၏ အရှေ့တွင် v ကို ထည့်မည်` | `v` into the scratch slot `%cell`; `l; i; %cell; insert_cell k` |
| `t ၏ စာလုံး i ၏ အရှေ့တွင် v ကို ထည့်မည်` | `t; i; v; insert_text k`, dropped as for `add_text` |
| `t ၏ နောက်ဆုံးစာလုံးကို ဖျက်မည်` | `t; remove_cell text` |
| `z ထဲမှ အညွှန်း i ၏ တန်ဖိုး ကို v ဟု ထားမည်` | `i` and `v` into scratch slots `%cell`; `z; %cell; %cell; set_row k` |
| `z ထဲမှ အညွှန်း i ကို ဖျက်မည်` | `z; i; remove_row k`, a scalar index in a scratch slot first |
| `z ထဲမှ အညွှန်း i ၏ တန်ဖိုး`, read | `z; i; row_find k` into a hidden slot; a test of it builds the optional from the row, or the blank optional |
| a write inside the value of a row | `row_find k` as for a read; `halt no_row` when it returns -1; then the address of the value through the rows, `own` first, as for a cell |
| `t ၏ အကွက် a မှ b ထိကို v ဖြင့် အစားထိုးမည်` | `t; a; b; v; replace_text k`, dropped as for `add_text` |
| the end of a text variable | `x; drop text` |
| `မင်မိုရီရှိ မလိုအပ်သော အရာများကို ရှင်းလင်းမည်` | `rings collect`, and `rings pause` or `rings resume` for `အလိုအလျောက် မင်မိုရီ ရှင်းလင်းမှုကို ခေတ္တရပ်မည်` or `ပြန်လည်စတင်မည်` |
| `အလိုအလျောက် မင်မိုရီ ရှင်းလင်းမှုကို n ဘိုက် နှင့်အထက်မှ လုပ်ဆောင်မည်` | `n; rings threshold` |
| `a ဖြစ်၍ b` (and) | `a; jump_if_false F; b; jump E; F: push မမှန်; E:` |
| `a သို့မဟုတ် b` (or) | `a; jump_if_true T; b; jump E; T: push မှန်; E:` |
| if and else | `cond; jump_if_false Else; then; jump End; Else: else; End:` |
| a loop | `Head: cond; jump_if_false Exit; block; jump Head; Exit:` |

Here `x` means `local_addr n` or `global_addr n`.

A counted loop works out its two ends and its step once, in that order, into slots when
those are not constants. It never moves the counter past the last value, so a counter
cannot wrap. It works out the distance left with `wrap sub`, which gives the true distance
for ends of each sign, and steps with `wrap add` or `wrap sub`, so `--overflow stop` does
not change it. A step that is not a constant is checked once, before the first round, with
`trap step_not_positive`.

### A heap block in operations

Here is a block from the heap, written to, read and freed in an unsafe block (see [Unsafe
Zaka](../language/unsafe.md)), with the operations of its program start block. An unsafe
block adds no operation:

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
     2  store isize*         ; ပ has zero octets until the next write
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

The comments after `;` in this listing were added to explain it, and `zakac` prints the
variable names alone. Two operations name a data kind that the others leave out: `load`
and `store` name the data kind read or written, and `alloc` names the octet size of one
element, so the VM can check the access and the cap.

## Where to look next

- [When a program stops](traps.md) and [Stops in unsafe code](traps-unsafe.md) tell what
  each operation does when its check fails.
- [The VM target](../internals/vm-target.md) tells how the code generator makes these
  operations.
