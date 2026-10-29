---
title: Pointers
description: The kinds of pointer, taking an address and reading through it, the null address, pointer arithmetic, sizes, and the statements that ask for, resize and give back heap blocks.
---

# Pointers

A pointer is a value that holds an address in the RAM of the machine. Reading through a
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
| `ကိန်းပြည့် လိပ်စာ` | an *owner*: it holds a heap block and alone gives it back | `*isize` |
| `ငှားထားသော ကိန်းပြည့် လိပ်စာ` | a *borrow*: it points at a variable, or into a block that another pointer owns | `&*isize` |
| `ကိန်းပြည့် လိပ်စာအကြမ်း` | a *raw* address: the compiler does not follow it, and the machine checks each use | `~isize` |
| `ငှားထားသော ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ` | a borrow of a variable that owns a block | `&**isize` |
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
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ငှားထားသော ၈ ဘစ် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။
```

```text
error[ZK0401]: expected &*i8, found &*isize
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
၅ ၏ လိပ်စာ ကို ပြသပါ။
```

```text
error[ZK0411]: the address of a literal cannot be taken; variables and dereferences alone have an address
```

```zaka-error ZK0412
ကိန်းသေ ကြီးဆုံး သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၀၀ ဖြစ်ပါစေ။
ကြီးဆုံး ၏ လိပ်စာ ကို ပြသပါ။
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
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။
ကိန်းရှင် ညွှန်း သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် အသက် ၏ လိပ်စာ ဖြစ်ပါစေ။
ညွှန်း ၏ တန်ဖိုး ကို ပြသပါ။
ညွှန်း ၏ တန်ဖိုး ကို ၂၀ ဟု ထားပါ။
အသက် ကို ပြသပါ။
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
ကိန်းရှင် ပ သည် လိပ်စာ ဖြစ်ပါစေ။
ပ ၏ တန်ဖိုး ကို ပြသပါ။
```

```text
error[ZK0414]: an opaque "လိပ်စာ" cannot be dereferenced or offset
```

## The null address

`ဗလာ` ("blank") is the null address: a pointer that points nowhere. It is the default
value of each pointer kind, so a new pointer variable starts as `ဗလာ`. Zaka does not
treat a pointer as true or false, so a test compares with `ဗလာ`:

```zaka
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
ပ ကို ပြသပါ။
အကယ်၍ ပ သည် ဗလာ နှင့် ညီလျှင် "ဘာမှ မညွှန်ပါ" ဟု ပြသပါ။
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
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် မ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ခ ၏ လိပ်စာ ဖြစ်ပါစေ။
ပ - မ ကို ပြသပါ။
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
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၃ ခုစာ နေရာယူ၍ လိပ်စာကို ကိန်းများ တွင် သိမ်းထားမည်။
၀ မှ ၂ အထိ နံပါတ် အနေဖြင့် ရေတွက်၍ ကိန်းများ + နံပါတ် ၏ တန်ဖိုး ကို နံပါတ် * နံပါတ် ဟု ထားပါ။

ကိန်းရှင် ညွှန်း သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ကိန်းများ ၏ တန်ဖိုးအငှား ဖြစ်ကာ ညွှန်း သည် ကိန်းများ + ၃ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ညွှန်း ၏ တန်ဖိုး ကို ပြသပါ။
ပြီးလျှင် ညွှန်း ကို ညွှန်း + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

(ကိန်းများ + ၃) - ကိန်းများ ကို ပြသပါ။       // the distance in boxes
ကိန်းများ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
၀
၁
၄
၃
```

The machine allows the address `ပ + n` likewise: a subtraction of two pointers
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
ကိန်းပြည့် ၏ အရွယ်အစား ကို ပြသပါ။
၃၂ ဘစ် ကိန်းပြည့် ၏ အရွယ်အစား ကို ပြသပါ။
မှန်မှားတန်ဖိုး ၏ အရွယ်အစား ကို ပြသပါ။
ကိန်းပြည့် လိပ်စာ ၏ အရွယ်အစား ကို ပြသပါ။
```

```text
၈
၄
၁
၈
```

The numbers are those of the target the program is built for. Each value sits at an
address that is a multiple of its size, up to 16; the machine stops a program that
reads or writes at another address.

## Heap statements

Three statements work with blocks of the heap. In all three, `မင်မိုရီ` can be replaced
with `မှတ်ဉာဏ်`, and each ending `…မည်` can be written `…မယ်`.

### Sizes

A request names its size in one of two forms:

| Form | Asks for |
| --- | --- |
| `<kind> <count> ခုစာ` | room for `count` values of the kind |
| `<count> ဘိုက်စာ` | `count` octets |

The count is a `လက္ခဏာမဲ့ ကိန်းပြည့်`: a literal fits as it stands, and a variable used as a
count has that kind. The first form multiplies the count with the size of the kind, and a
product that does not fit counts as running out of room.

### Asking for a block

```text
မင်မိုရီ <size> နေရာယူ၍ လိပ်စာကို <place> တွင် သိမ်းထားမည်။
```

The statement reserves a new block and stores its address in the place, which must have a
pointer kind. The VM fills a new block with zeros, but the language promises nothing about
what a new block holds, so a program writes before it reads. A size of 0 gives a block
that is not `ဗလာ` and holds nothing. When the heap has no room left, the place gets
`ဗလာ`; a careful program compares with `ဗလာ` before it uses the block. The machine has
256 MiB of heap to start with, and `zakac run --max-heap <octets>` sets another limit.

### Resizing a block

```text
<place> လိပ်စာရှိကို မင်မိုရီ <size> ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။
```

The statement grows or shrinks the block whose address is in the place, and stores the
address of the result back in the place. The block can move. The values that fit the new
size are kept, and a larger block holds new room after them. A place that holds `ဗလာ`
makes the statement ask for a new block. When the heap has no room for the new size, the
program stops with "no heap room" rather than setting the place to `ဗလာ`, which would
lose the block still held.

### Giving a block back

```text
<place> လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

The statement gives the block back and sets the place to `ဗလာ`. Giving back a place that
holds `ဗလာ` does nothing. The place alone is cleared; another pointer to the block would
point at a block that is gone. The machine stops a program that uses such a pointer, and
the compiler finds the mistake before the program runs when it can. The rules are in
[Ownership and borrowing](ownership.md).

This program asks for a block of two integers, grows it to three, gives it back twice
(the second time is harmless) and prints the place:

```zaka
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၁၀ ဟု ထားပါ။
ပ + ၁ ၏ တန်ဖိုး ကို ၂၀ ဟု ထားပါ။
ပ လိပ်စာရှိကို မင်မိုရီ ကိန်းပြည့် ၃ ခုစာ ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။
ပ + ၂ ၏ တန်ဖိုး ကို ၃၀ ဟု ထားပါ။
၀ မှ ၂ အထိ န အနေဖြင့် ရေတွက်၍ ပ + န ၏ တန်ဖိုး ကို ပြသပါ။
ပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ပ ကို ပြသပါ။
```

```text
၁၀
၂၀
၃၀
ဗလာ
```

A heap statement whose place is not a pointer is ZK0417:

```zaka-error ZK0417
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်ပါစေ။
မင်မိုရီ ၈ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို က တွင် သိမ်းထားမည်။
```

```text
error[ZK0417]: the heap statement needs a pointer, found isize
```

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
| ZK0709 | a read through a pointer that is `ဗလာ` on each path |
| ZK0710 | an offset past the end of a block |
| ZK0711 | a pointer that can be `ဗလာ` (a warning, with `--mem-strict`) |
| ZK0712 | two pointers into different blocks subtracted |

The text of each message is in [Error messages](../reference/errors.md).
