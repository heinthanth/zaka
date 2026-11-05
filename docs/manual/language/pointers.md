---
title: Pointers
description: The kinds of pointer, taking an address and reading through it, the null address, pointer arithmetic, sizes, and the statements that ask for, resize and give back heap blocks.
---

# Pointers

A pointer is a value that holds an address in the RAM of the VM. Reading through a
pointer gives the value stored at that address, and writing through it changes that value.
This page states the whole rules for pointers and for the heap statements. Who is allowed
to give a block back, and when a pointer can be used, is the subject of [Ownership and
borrowing](ownership.md). The Learn book builds these ideas up in small steps, starting from
[pointers and the heap](../../learn/pointers/README.md).

## Two places for values

A variable lives on the stack of its function and is gone when the function returns. A
block of the heap lives until the program gives it back. A pointer can point at either:

```text
  stack of a call                       heap

  age    +------+
         |  18  | <---------+
         +------+           |
  ptr    +------+           |
         |  *---|-----------+
         +------+

  list   +------+                       +------+------+------+
         |  *---|---------------------> |  10  |  20  |  30  |
         +------+                       +------+------+------+
                                        ^      ^      ^      ^
                                        list   list+1 list+2 list+3
                                                             (one past the end)
```

`ptr` borrows the variable `age`. `list` owns the heap block: it is the one name that gives
the block back. `list + 1` is the address of the next integer, and `list + 3` is the
address just past the last one.

## The kinds of pointer

A pointer kind is the kind of the value it points at, with `လိပ်စာ` ("address") after it. A
pointer to a pointer adds `၏ လိပ်စာ` for each further level.

| Kind | What it is | Short name in messages |
| --- | --- | --- |
| `ကိန်းပြည့် လိပ်စာ` | an *owner*: it holds a heap block and alone gives it back | `isize*` |
| `ငှားထားသော ကိန်းပြည့် လိပ်စာ` | a *borrow*: it points at a variable, or into a block that another pointer owns | `isize&` |
| `ကိန်းပြည့် လိပ်စာအကြမ်း` | a *raw* address: the compiler does not follow it, and the VM checks each use | `isize^` |
| `ငှားထားသော ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ` | a borrow of a variable that owns a block | `isize*&` |
| `လိပ်စာ` | an *opaque* address: it points at a value of no known kind | `ptr` |

The word `ငှားထားသော` ("lent") stands once, at the front of the kind, and marks the outer
level. The kinds of Zaka for numbers, true/false values and characters are in [Kinds and
values](kinds.md); each of them can stand in front of `လိပ်စာ`, and a list kind can as
well (see [Lists](lists.md#lists-and-addresses)).

The first level also takes the older spelling `ကိန်းပြည့် ၏ လိပ်စာ`, which means the same
as `ကိန်းပြည့် လိပ်စာ`.

Pointers of different kinds do not mix. A pointer to an integer does not fit a variable
that points at a smaller integer:

```zaka-error ZK0401
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ငှားထားသော ၈ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0401]: expected i8&, found isize&
```

Two implicit conversions exist: `ဗလာ` converts to each pointer kind, and a pointer of a named kind
converts to the opaque `လိပ်စာ` and back. Each other change of pointer kind is written
with `ကို <kind> အဖြစ် ပြောင်းလဲသော ရလဒ်`; see [Conversions](conversions.md). A conversion
that would read stored addresses as other data is refused, and the message names the raw
step to take instead. The whole set of rules for owners, borrows and raw addresses is in
[Ownership and borrowing](ownership.md).

## Taking an address

`<place> ၏ လိပ်စာ` is the address of a place. The short form is `&<place>`. The result is
a borrow, and its kind is the kind of the place with `ငှားထားသော` in front and `လိပ်စာ`
after it.

A place that has an address is:

- a variable or a parameter,
- the variable `ရလဒ်`,
- a value reached through a pointer, and
- a cell of a list (see [Lists](lists.md#lists-and-addresses)).

A number such as `၅`, and the result of a sum, are values with no place, so there is no
address to take (ZK0411). A constant does have a place, but Zaka does not give its
address out, since a write through it would change a constant. The variable of a counted
loop is the same (ZK0412).

```zaka-error ZK0411
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၅ ၏ လိပ်စာ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0411]: the address of a literal cannot be taken; variables and dereferences alone have an address
```

```zaka-error ZK0412
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ ကြီးဆုံး သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀၀ ဖြစ်ပါစေ။
ကြီးဆုံး ၏ လိပ်စာ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0412]: "ကြီးဆုံး" cannot be changed, so its address cannot be taken
```

The address of a local variable or a parameter is good until its function returns. The
address of a top-level variable is good for the whole run. A function cannot return a
borrow of its own variable (ZK0706), and a borrow cannot be stored where it would outlast
what it points at (ZK0703). Both are in [Ownership and
borrowing](ownership.md#escaping-addresses).

## Reading and writing through a pointer

`<pointer> ၏ တန်ဖိုး` is the value at the address, and the short form is `*<pointer>`. The
result is a place: it can be read, printed, or assigned to.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။
ကိန်းရှင် ညွှန်း သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အသက် ၏ လိပ်စာ ဖြစ်ပါစေ။
ညွှန်း ၏ တန်ဖိုး ကို ပြသပါ။
ညွှန်း ၏ တန်ဖိုး ကို ၂၀ ဟု ထားပါ။
အသက် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၈
၂၀
```

The two spellings group in a different order, and the difference matters when the pointer is part
of a sum:

| Written | Reads as |
| --- | --- |
| `ပ + ၁ ၏ တန်ဖိုး` | the value at `ပ + ၁`: the verb takes the whole expression before it |
| `*ပ + ၁` | `(*ပ) + ၁`: the star takes the name right after it |
| `(ပ ၏ တန်ဖိုး) + ၁` | the same as `*ပ + ၁` |

Reading through something that is not a pointer is ZK0413. The opaque `လိပ်စာ` cannot be
read through, and cannot be offset, until it is converted to a pointer of a named kind (ZK0414):

```zaka-error ZK0414
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
ပ ၏ တန်ဖိုး ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0414]: an opaque "လိပ်စာ" cannot be dereferenced or offset
```

## The null address

`ဗလာ` ("blank") is the null address: a pointer that points nowhere. It is the default
value of each pointer kind, so a new pointer variable starts as `ဗလာ`. Zaka does not
treat a pointer as true or false, so a test compares with `ဗလာ`:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
ပ ကို ပြသပါ။
အကယ်၍ ပ သည် ဗလာ နှင့် ညီလျှင် "ဘာမှ မညွှန်ပါ" ဟု ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဗလာ
ဘာမှ မညွှန်ပါ
```

A pointer that holds an address prints as `0x` and the address in hexadecimal. Reading or
writing through `ဗလာ` stops the program. When the compiler can prove that a pointer is
`ဗလာ` on each path to a read, it reports ZK0709 instead of waiting for the stop; and under
`--mem-strict` it warns (ZK0711) about a pointer that *can* be `ဗလာ`, such as a new block
that was never compared with `ဗလာ`. See [Pointer checks](../compiler/checks.md).

## Pointer arithmetic

A pointer moves in steps of whole values. `ပ + ၁` is the next value, not the next octet,
and the pointer kind tells the size in octets. These operations alone exist on
pointers:

| Form | Kinds | Gives |
| --- | --- | --- |
| `ပ + န`, `န + ပ`, `ပ နှင့် န ၏ ပေါင်းလဒ်` | a pointer and an integer | a pointer of the same kind, `န` values on |
| `ပ - န`, `ပ နှင့် န ၏ နုတ်လဒ်` | a pointer and an integer | a pointer of the same kind, `န` values back |
| `ပ - မ`, `ပ နှင့် မ ၏ နုတ်လဒ်` | two pointers of one kind | the distance in values, a `ကိန်းပြည့်` |
| `ပ သည် မ နှင့် ညီ`, `မညီ` | two pointers of one kind, or a pointer and `ဗလာ` | whether both hold the same address |
| `ပ သည် မ အောက် ငယ်` and the other orders | two pointers of one kind | which address is lower |

Each other operator on a pointer, such as `*` or `/`, is ZK0415. Pointers of different
kinds cannot be subtracted or compared (ZK0416), and two pointers into different blocks
have no distance (ZK0712):

```zaka-error ZK0712
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် မ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ခ ၏ လိပ်စာ ဖြစ်ပါစေ။
ပ - မ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0712]: these pointers point into different blocks and cannot be subtracted
```

### The one-past-the-end rule

A block of `n` values holds the values `ပ + ၀` to `ပ + (n - ၁)`. The address `ပ + n`, just
past the last value, is allowed as well: a program can form it, compare against it and
subtract it. It can never be read or written, since no value is stored there. Each other
address outside the block is out of bounds: a constant offset past `n` is ZK0710 when the
pointer is formed, and a read or write at an address outside the block stops the
program.

That rule is what a walk through a block needs. A loop stops when the walker meets the
end:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၃ ခုစာ နေရာယူ၍ လိပ်စာကို ကိန်းများ တွင် သိမ်းထားမည်။
၀ မှ ၂ အထိ နံပါတ် အနေဖြင့် ရေတွက်၍ ကိန်းများ + နံပါတ် ၏ တန်ဖိုး ကို နံပါတ် * နံပါတ် ဟု ထားပါ။

ကိန်းရှင် ညွှန်း သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကိန်းများ ၏ တန်ဖိုးအငှား ဖြစ်ကာ ညွှန်း သည် ကိန်းများ + ၃ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ညွှန်း ၏ တန်ဖိုး ကို ပြသပါ။
ပြီးလျှင် ညွှန်း ကို ညွှန်း + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

(ကိန်းများ + ၃) - ကိန်းများ ကို ပြသပါ။       // the distance in boxes
လိပ်စာ ကိန်းများ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၀
၁
၄
၃
```

The VM allows the address `ပ + n` likewise: a subtraction of two pointers
counts the end of the block as inside it. A read at that address stops the program with
an access outside each live block.

`ကိန်းများ ၏ တန်ဖိုးအငှား` lends the block so that `ညွှန်း` can walk it while `ကိန်းများ`
remains the owner. An address made with `+` or `-` is a borrow of itself and needs no such
word.

## Size of a kind

`<kind> ၏ အရွယ်အစား` ("the size of the kind") is the number of octets one value of the
kind takes. It is a constant, a `လက္ခဏာမဲ့ ကိန်းပြည့်`, and an operand like a number.

| Kind | Octets |
| --- | --- |
| `N ဘစ် ကိန်းပြည့်`, `N ဘစ် ဒဿမကိန်း` | `N / 8` |
| `ကိန်းပြည့်`, `လက္ခဏာမဲ့ ကိန်းပြည့်` | 8 |
| `စာလုံး` | 4 |
| `မှန်မှားတန်ဖိုး` | 1 |
| each pointer kind and `လိပ်စာ` | 8 |
| a list | its cell count times the size of one cell, with no gaps |

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းပြည့် ၏ အရွယ်အစား ကို ပြသပါ။
၃၂ ဘစ် ကိန်းပြည့် ၏ အရွယ်အစား ကို ပြသပါ။
မှန်မှားတန်ဖိုး ၏ အရွယ်အစား ကို ပြသပါ။
ကိန်းပြည့် လိပ်စာ ၏ အရွယ်အစား ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၈
၄
၁
၈
```

The numbers are those of the target the program is built for. Each value sits at an
address that is a multiple of its size, up to 16; the VM stops a program that
reads or writes at another address.

## Heap statements

Three statements work with blocks of the heap, and [three more](#writing-and-filling-cells)
write the cells of a block. In all of them, `မင်မိုရီ` can be replaced with `မှတ်ဉာဏ်`. Resizing
and giving back start alike, with `လိပ်စာ <place> ရှိ မင်မိုရီကို` ("the RAM at the
address in the place"); the words after it tell the two apart.

### Sizes

A request names its size in one of three forms:

| Form | Asks for |
| --- | --- |
| `<kind> <count> ခုစာ` | room for `count` values of the kind |
| `<struct> <count> <classifier>စာ` | room for `count` values of a struct that names a [classifier](structs.md#the-classifier) |
| `<count> ဘိုက်စာ` | `count` octets |

The count is a `လက္ခဏာမဲ့ ကိန်းပြည့်`: a literal fits as it stands, and a variable used as a
count has that kind. Each form with a kind multiplies the count with the size of the kind,
and a product that does not fit counts as running out of room.

### Asking for a block

```text
မင်မိုရီတွင် <size> နေရာယူ၍ လိပ်စာကို <place> တွင် သိမ်းထားမည်။
```

The statement reserves a new block and stores its address in the place, which must have a
pointer kind. The VM fills a new block with zeros, but the language promises nothing about
what a new block holds, so a program writes before it reads. A size of 0 gives a block
that is not `ဗလာ` and holds nothing. When the heap has no room left, the place gets
`ဗလာ`; a careful program compares with `ဗလာ` before it uses the block. The VM has
256 MiB of heap to start with, and `zakac run --max-heap <octets>` sets another limit.

### Resizing a block

```text
လိပ်စာ <place> ရှိ မင်မိုရီကို <size> ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။
```

The statement grows or shrinks the block whose address is in the place, and stores the
address of the result back in the place. The block can move. The values that fit the new
size are kept, and a larger block holds new room after them. A place that holds `ဗလာ`
makes the statement ask for a new block. When the heap has no room for the new size, the
program stops with "no heap room" rather than setting the place to `ဗလာ`, which would
lose the block still held.

### Giving a block back

```text
လိပ်စာ <place> ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

The statement gives the block back and sets the place to `ဗလာ`. Giving back a place that
holds `ဗလာ` does nothing. The place alone is cleared; another pointer to the block would
point at a block that is gone. The VM stops a program that uses such a pointer, and
the compiler finds the mistake before the program runs when it can. The rules are in
[Ownership and borrowing](ownership.md).

This program asks for a block of two integers, grows it to three, gives it back twice
(the second time is harmless) and prints the place:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၁၀ ဟု ထားပါ။
ပ + ၁ ၏ တန်ဖိုး ကို ၂၀ ဟု ထားပါ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ကိန်းပြည့် ၃ ခုစာ ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။
ပ + ၂ ၏ တန်ဖိုး ကို ၃၀ ဟု ထားပါ။
၀ မှ ၂ အထိ န အနေဖြင့် ရေတွက်၍ ပ + န ၏ တန်ဖိုး ကို ပြသပါ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ပ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၀
၂၀
၃၀
ဗလာ
```

A heap statement whose place is not a pointer is ZK0417:

```zaka-error ZK0417
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ၈ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို က တွင် သိမ်းထားမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0417]: the heap statement needs a pointer, found isize
```

## Writing and filling cells

Three statements write a whole run of cells at once. Each starts as resizing does, with
`လိပ်စာ <ptr> ရှိ`, and each has a size, as in Sizes above, of the cells it handles:

| Statement | Shape |
| --- | --- |
| write the cells, no overlap | `လိပ်စာ SRC ရှိ SIZE မင်မိုရီကို လိပ်စာ DST ဆီသို့ ကူးယူမည်။` |
| write the cells, overlap allowed | `လိပ်စာ SRC ရှိ SIZE မင်မိုရီကို လိပ်စာ DST ဆီသို့ ပြောင်းထည့်မည်။` |
| fill | `လိပ်စာ DST ရှိ မင်မိုရီကို VALUE ဖြင့် SIZE ဖြည့်မည်။` |

`SIZE` is `<kind> N ခုစာ`, a count of cells of the kind, or `N ဘိုက်စာ`, a count of octets.
The first two write the cells at `SRC` to `DST`, and the third sets each cell at `DST` to
`VALUE`. `ကူးယူမည်` is the `memcpy` of C: its two ranges must not overlap.
`ပြောင်းထည့်မည်` is the `memmove` of C: it reads the source in full before it writes, so
the ranges can overlap, as when a run of cells moves one place on.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို က တွင် သိမ်းထားမည်။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ခ တွင် သိမ်းထားမည်။

လိပ်စာ က ရှိ မင်မိုရီကို ၇ ဖြင့် ကိန်းပြည့် ၄ ခုစာ ဖြည့်မည်။       // fill
က ၏ တန်ဖိုး ကို ၁ ဟု ထားပါ။
က + ၁ ၏ တန်ဖိုး ကို ၂ ဟု ထားပါ။
က + ၂ ၏ တန်ဖိုး ကို ၃ ဟု ထားပါ။
လိပ်စာ က ရှိ ကိန်းပြည့် ၄ ခုစာ မင်မိုရီကို လိပ်စာ ခ ဆီသို့ ကူးယူမည်။  // two blocks
လိပ်စာ က ရှိ ကိန်းပြည့် ၃ ခုစာ မင်မိုရီကို လိပ်စာ က + ၁ ဆီသို့ ပြောင်းထည့်မည်။  // one block
၀ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ "{က + န ၏ တန်ဖိုး} {ခ + န ၏ တန်ဖိုး}" ဟု ပြသပါ။
လိပ်စာ က ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လိပ်စာ ခ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁ ၁
၁ ၂
၂ ၃
၃ ၇
```

The rules:

- `SRC` and `DST` are values of each pointer kind: an owner, a borrow, a raw address or the
  opaque `လိပ်စာ`. Each need not be a variable (`က + ၁` serves), since nothing is written back
  to them. The kinds of `SRC`, `DST` and the cells need not match. A value that is no pointer
  is ZK0417.
- The count is a `လက္ခဏာမဲ့ ကိန်းပြည့်`, as for the other heap statements (ZK0401 if it
  is not).
- The value of a fill is of the kind of the cells when the size is `<kind> N ခုစာ`, and a
  `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်` when the size is `N ဘိုက်စာ`; with another kind it is ZK0401.
  A struct value fills as well: `အမှတ်(ဒေါင်: ၁၊ အလျား: ၂) ဖြင့် အမှတ် ၃ ခုစာ`.
- The verb at the end, `ဖြည့်မည်`, makes the statement a fill, so a value that starts with
  a kind word, such as `၈ ဘစ် ကိန်းပြည့် ၅`, reads as a value and needs no parentheses. A
  `ဖြင့်` right after a lone value ends the value, while
  one inside a phrase such as a division belongs to it: in
  `က ကို ၂ ဖြင့် စားသော စားလဒ် ဖြင့် ၁ ဘိုက်စာ` the value is the quotient.
- A count of 0 does nothing, and neither the borrow check nor the VM checks the
  addresses. Else the VM checks that the source
  range lies in one live block and the destination range in one live block that can be
  written, and each failure stops the program as an access does: out of bounds, use after
  free, a null address, a write into static text. A count whose product with the size of
  a cell does not fit is out of bounds as well.
- `ကူးယူမည်` whose ranges overlap is ZK0721 when both addresses point into one block at
  offsets the borrow check knows and the count is known; else the VM stops the program
  with the trap `overlap` (see [all the traps](../vm/traps.md#all-the-traps)). Use
  `ပြောင်းထည့်မည်` when the ranges can overlap.
- The cells must own no block and borrow none (ZK0442). A cell of an owner pointer, of
  text or of a plain list holds a block, and a raw duplicate would give one block two
  owners, which would both give it back. A cell of a borrow or a slice points into a place
  the borrow check follows, and a raw duplicate would put the borrow where the check
  cannot see it. The cells are of the kind of the size, or, for a size in
  octets, of the kind each address points at. Assign such cells one at a time, or keep the
  addresses in raw pointers.

```zaka-error ZK0442
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် စာသား လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် စာသား လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် စာသား ၂ ခုစာ နေရာယူ၍ လိပ်စာကို က တွင် သိမ်းထားမည်။
မင်မိုရီတွင် စာသား ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ခ တွင် သိမ်းထားမည်။
လိပ်စာ က ရှိ စာသား ၂ ခုစာ မင်မိုရီကို လိပ်စာ ခ ဆီသို့ ကူးယူမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0442]: the cells are "text", which own a block or borrow one: a raw duplicate or fill would give the block a second owner, or keep a borrow the check cannot follow
```

The borrow check sees the destination as a write through the address, as a store through a
pointer is: a loan into a plain list that the statement edits ends, a destination that
is `ဗလာ` for sure, or whose first or last cell lies past the end of its block, is an
error, and what the check knew about the slots the destination names is forgotten. It
sees the source as a read through the address, with the same checks of its first and
last cell. The last cell is known when the count is known on each path.

## Errors at a glance

| Code | Mistake |
| --- | --- |
| ZK0410 | an assignment to something that is not a variable or a value through a pointer |
| ZK0411 | the address of a value that has no place, such as `၅` |
| ZK0412 | the address of a constant or of a counted-loop variable |
| ZK0413 | a read through a value that is not a pointer |
| ZK0414 | a read through, or an offset of, the opaque `လိပ်စာ` |
| ZK0415 | an operator such as `*` or `/` on a pointer |
| ZK0416 | pointers of different kinds compared or subtracted |
| ZK0417 | a heap statement given something other than a pointer |
| ZK0442 | cells that own a block or borrow one, in a duplicate or a fill |
| ZK0721 | the two ranges of `ကူးယူမည်` overlap for certain |
| ZK0709 | a read through a pointer that is `ဗလာ` on each path |
| ZK0710 | an offset past the end of a block |
| ZK0711 | a pointer that can be `ဗလာ` (a warning, with `--mem-strict`) |
| ZK0712 | two pointers into different blocks subtracted |

The text of each message is in [Error messages](../reference/errors.md).
