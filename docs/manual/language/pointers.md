---
title: Pointers and addresses
description: The pointer kind, the null address, taking an address, reading and writing through a pointer, pointer arithmetic, the size of a data kind and the conversions to and from pointers.
---

# Pointers and addresses

An address (pointer) is a value that points at a place in the RAM (memory) of the VM
(virtual machine), and we can use it to read or write the value at that place. This page
covers the pointer kind (pointer type), the null address (null pointer), taking an
address, reading and writing through a pointer, pointer arithmetic, the size of a data
kind (type), and conversions between pointers and other data kinds. Each of these needs
unsafe code, which [Unsafe Zaka](unsafe.md) describes, and the heap and owned pointers
are in [The heap and owned pointers](heap.md). The Learn book teaches pointers in
[Unsafe Zaka](../../learn/pointers/README.md) and [Pointer
arithmetic](../../learn/pointers/arithmetic.md).

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ညွှန်း သည် အသက် ၏ လိပ်စာ ဖြစ်ပါစေ။
ညွှန်း ၏ တန်ဖိုး ကို ပြသမည်။
ညွှန်း ၏ တန်ဖိုး ကို ၂၀ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အသက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၈
၂၀
```

## The pointer kind

```text
<kind> လိပ်စာ
<kind> လိပ်စာ ၏ လိပ်စာ
လိပ်စာ
```

To write a pointer kind, we put `လိပ်စာ` ("address") after the data kind that the
pointer points at, and for a pointer to a pointer we add `၏ လိပ်စာ` for each further
level. Each data kind has one pointer kind, so the pointer kind does not show who
frees the block (allocation) it points into. The data kinds of numbers, characters and
true/false values are in [Data kinds and values](kinds.md), and each of them can come before
`လိပ်စာ`, as can a struct or a list.

| Pointer kind | What it points at | Short name in messages |
| --- | --- | --- |
| `ကိန်းပြည့် လိပ်စာ` | an integer | `isize*` |
| `ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ` | a pointer to an integer | `isize**` |
| `လိပ်စာ` | a value of no known data kind: an opaque pointer, like `void *` in C | `ptr` |

At the first level we can also write the older form `ကိန်းပြည့် ၏ လိပ်စာ`, which means the
same as `ကိန်းပြည့် လိပ်စာ`.

Pointers of different pointer kinds do not mix, so a pointer to an integer does not fit a
variable that points at a smaller integer (ZK0401):

```zaka-error ZK0401
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ၈ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0401]: expected i8*, found isize*
```

Two changes of pointer kind need no conversion: the null address fits each pointer kind,
and a pointer to a named data kind turns into the opaque `လိပ်စာ` and back. Each other
change needs a [conversion](#conversions).

## The null address

```text
မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ
```

We write `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` ("the address that points at nothing") for the
null address (null pointer, `NULL`). The pointer is the one data kind in Zaka that can be
null: a shared object and a function value cannot, and for a value that might be missing
we use an [optional](optionals.md#null-is-a-mistake).

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ပ ကို ပြသမည်။
အကယ်၍ ပ သည် မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ နှင့် ညီလျှင် "ဘာမှ မညွှန်ပါ" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ
ဘာမှ မညွှန်ပါ
```

The null address is the [initial value](kinds.md#initial-values) (default value) of each
pointer kind, so `ကနဦးတန်ဖိုး` sets `ပ` to it in the example. A pointer variable that we
declare with no value has no address at all, not even the null one, and it must be set
before the program reads it ([Set before read](declarations.md#set-before-read)). Zaka does
not treat a pointer as true or false, so to test a pointer we compare it with the null
address, which safe code can do. A null pointer prints as the same phrase, and a pointer
with an address prints as `0x` followed with the address in hexadecimal.

The phrase is three reserved words that the lexer joins into one value: `မည်သည်ကိုမှ`
("nothing"), `မညွှန်းသော` ("that points at no") and `လိပ်စာ`. It has no pointer kind alone,
 so a declaration with this value and no data kind written is ZK0430, and it fits
no data kind except a pointer (ZK0401).

Reading or writing through the null address is a mistake in each case. When the compiler can
prove that a pointer is null on each path to a read, it reports ZK0709, and under
`--lint-warn doubts` it warns (ZK0711) about a pointer that *can* be null, such as a new
block that the program never compared with the null address. In unsafe code both
findings follow the level of the `mem` lint (see [The checks on unsafe
code](unsafe.md#the-checks-on-unsafe-code)). On the VM the read stops the program, while
in native code it crashes the program, as it does in C.

## Taking an address

```text
<place> ၏ လိပ်စာ
&<place>
```

We use `<place> ၏ လိပ်စာ`, or the short form `&<place>`, to take the address of a place.
The result has the data kind of the place with `လိပ်စာ` after it. These places have an
address:

- a variable or a parameter,
- a value reached through a pointer,
- a field of a struct,
- a cell of a list or of text.

A number such as `၅`, or the result of a sum, is a value with no place, so it has no
address to take (ZK0411). A constant does have a place, but Zaka does not let us take its
address, because a write through that address would change the constant, and the
variable of a counted loop is refused for the same reason (ZK0412).

```zaka-error ZK0411
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
၅ ၏ လိပ်စာ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0411]: the address of a literal cannot be taken; variables and dereferences alone have an address
```

```zaka-error ZK0412
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းသေ ကြီးဆုံး သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀၀ ဖြစ်ပါစေ။
ကြီးဆုံး ၏ လိပ်စာ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0412]: "ကြီးဆုံး" cannot be changed, so its address cannot be taken
```

The address of a local variable or a parameter is good until its function returns, and
the address of a top-level variable is good for the whole run. When a function returns
the address of one of its locals or parameters, the compiler warns with ZK0706, whatever
the pointer kind, and on the VM a read through that address after the function has
returned stops the program.

### Pinning

A list or text value shares its cells with its duplicates (copies) until one side writes
(see [Values and objects](objects.md)). An address of a cell would break that, because a
write through the address would reach each duplicate. So when we take the address of a
cell, the list first gets cells that no other value shares, and then the VM *pins* these
cells: a later duplicate of a pinned list gets new cells at once and never shares them.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် က ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။
ပ ၏ တန်ဖိုး ကို ၉ ဟု ထားမည်။
က ကို ပြသမည်။
ခ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၉၊ ၂၊ ၃]
[၁၊ ၂၊ ၃]
```

A pin lasts as long as the cells do, and a resize keeps it. Adding a cell to the list can
still move its cells to a new block, and on the VM a read through an address of the old
cells stops the program.

## Reading and writing through a pointer

```text
<pointer> ၏ တန်ဖိုး
*<pointer>
```

We use `<pointer> ၏ တန်ဖိုး`, or the short form `*<pointer>`, to reach the value at the
address. The result is a place, so we can read it, print it or assign to it, as the first
example on this page does.

The two forms group in a different order, and the difference matters when the pointer is
part of a sum:

| Written | Meaning |
| --- | --- |
| `ပ + ၁ ၏ တန်ဖိုး` | the value at `ပ + ၁`, because `၏ တန်ဖိုး` applies to the whole expression before it |
| `*ပ + ၁` | `(*ပ) + ၁`, because `*` applies to the name right after it |
| `(ပ ၏ တန်ဖိုး) + ၁` | the same as `*ပ + ၁` |

Reading through a value that is not a pointer is ZK0413. We cannot read through the
opaque `လိပ်စာ` or offset it until we convert it to a pointer of a named data kind
(ZK0414):

```zaka-error ZK0414
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0414]: an opaque "လိပ်စာ" cannot be dereferenced or offset
```

## Pointer arithmetic

A pointer moves in steps of whole values, so `ပ + ၁` is the next value, not the next octet
(byte), and the pointer kind decides the size of each step in octets. Pointers support
these operations and no others:

| Form | Operands | Result |
| --- | --- | --- |
| `ပ + န`, `န + ပ`, `ပ နှင့် န ၏ ပေါင်းလဒ်` | a pointer and an integer | a pointer of the same pointer kind, `န` values further on |
| `ပ - န`, `ပ နှင့် န ၏ နုတ်လဒ်` | a pointer and an integer | a pointer of the same pointer kind, `န` values back |
| `ပ - မ`, `ပ နှင့် မ ၏ နုတ်လဒ်` | two pointers of one pointer kind | the distance in values, as a `ကိန်းပြည့်` |
| `ပ သည် မ နှင့် ညီ`, `မညီ` | two pointers of one pointer kind, or a pointer and the null address | whether both point at the same address |
| `ပ သည် မ အောက် ငယ်` and the other orders | two pointers of one pointer kind | which address is lower |

Each other operator on a pointer, such as `*` or `/`, is ZK0415, and we cannot subtract or
compare pointers of different pointer kinds (ZK0416). Two pointers into different blocks have no
distance: the compiler warns (ZK0712) when it knows that the blocks differ, and the VM
stops the subtraction.

```zaka-warning ZK0712
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် က ၏ လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် မ သည် ခ ၏ လိပ်စာ ဖြစ်ပါစေ။
ပ - မ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0712]: these pointers point into different blocks and cannot be subtracted
the program stopped: pointers into different blocks cannot be subtracted
```

### The one-past-the-end rule

A block of `n` values has its values at `ပ + ၀` to `ပ + (n - ၁)`. We can also form the
address `ပ + n`, just past the last value, and compare or subtract it, but we can never
read or write there, because no value is stored at that address. Each other address
outside the block is out of bounds:

- a constant offset past `n` is ZK0710 at the point where the program forms the pointer,
- on the VM, a step that lands further than `ပ + n`, or before the start of the block,
  stops the program at once, even before a read,
- on the VM, a read or a write at `ပ + n` stops the program as an access outside each live
  block, while a subtraction counts `ပ + n` as inside the block.

The rule is what a walk through a block needs, because the walker can reach the end and
the loop then stops:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၃ ခုစာ နေရာယူ၍ လိပ်စာကို ကိန်းများ တွင် သိမ်းထားမည်။
၀ မှ ၂ အထိ နံပါတ် အနေဖြင့် ရေတွက်၍ ကိန်းများ + နံပါတ် ၏ တန်ဖိုး ကို နံပါတ် * နံပါတ် ဟု ထားမည်။

ကိန်းရှင် ညွှန်း သည် ကိန်းများ ဖြစ်ကာ ညွှန်း သည် ကိန်းများ + ၃ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ညွှန်း ၏ တန်ဖိုး ကို ပြသမည်။
ပြီးလျှင် ညွှန်း ကို ညွှန်း + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

(ကိန်းများ + ၃) - ကိန်းများ ကို ပြသမည်။       // the distance in values
လိပ်စာ ကိန်းများ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၀
၁
၄
၃
```

## The size of a data kind

```text
<kind> တစ်<word> ၏ အရွယ်အစား
```

We use `<kind> တစ်<word> ၏ အရွယ်အစား` ("the size of one `<kind>`") to get the number of
octets that one value of a data kind uses, where `<word>` is the [measure
word](kinds.md#measure-words) (classifier) of the data kind. The result is a constant of
the data kind `လက္ခဏာမဲ့ ကိန်းပြည့်`, and we can use it as an operand like a number.
Because it reads no storage, safe code can use it too.

| Data kind | Octets |
| --- | --- |
| `N ဘစ် ကိန်းပြည့်`, `N ဘစ် ဒဿမကိန်း` | `N / 8` |
| `ကိန်းပြည့်`, `လက္ခဏာမဲ့ ကိန်းပြည့်` | 8 |
| `စာလုံး` | 4 |
| `မှန်မှားတန်ဖိုး` | 1 |
| each pointer kind and `လိပ်စာ` | 8 |
| a counted list | its cell count times the size of one cell, with no gaps |

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းပြည့် တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
မှန်မှားတန်ဖိုး တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
ကိန်းပြည့် လိပ်စာ တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၈
၄
၁
၈
```

The sizes are those of the target that the program is built for. Each value sits at an
address that is a multiple of its size, up to 16, and on the VM a read or a write at
another address stops the program. The heap statements use the same sizes in their
size forms ([Heap statements](heap.md#heap-statements)).

## Conversions

```text
<value> ကို <kind> တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်
```

We use the conversion form `ကို <kind> တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်` (see
[Conversions](conversions.md)) to turn a pointer into another data kind, or another data
kind into a pointer. Each of these conversions needs unsafe code (ZK0468):

| From | To | Result |
| --- | --- | --- |
| a pointer | a pointer of another pointer kind, or `လိပ်စာ` | the same address, used as the new pointer kind |
| a pointer | an integer | the address as a number |
| an integer | a pointer | the number as an address, and the VM checks each use of it |
| a shared object | a pointer | the address of the object, and the pointer keeps a count on the object |
| a pointer | a shared object | the object again, and the count that the pointer kept comes back with it |

We use the last two to pass an object through code that knows addresses alone, such as
the user data of a C callback. The object lives on while the pointer keeps its count,
so a pointer that never turns back into an object keeps the object alive to the end of
the run, and `zakac run --leaks` lists it.

```zaka
အကောင့် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု နှင့် လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အကောင့်(အမည်: "မမ"၊ လက်ကျန်: ၁၀၀) ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ပ ကို က ကို လိပ်စာ တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ဟု ထားမည်။
ကိန်းရှင် ခ သည် ပ ကို အကောင့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
ခ ၏ လက်ကျန် ကို ၅၀ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
က ၏ လက်ကျန် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၅၀
```

`ခ` is the same object as `က`, so the change that we make through `ခ` shows through `က`.

## Errors

| Code | Mistake |
| --- | --- |
| ZK0401 | a pointer of one pointer kind where another pointer kind is expected, or the null address where no pointer is expected |
| ZK0410 | an assignment to something that is not a variable or a value through a pointer |
| ZK0411 | the address of a value that has no place, such as `၅` |
| ZK0412 | the address of a constant or of a counted-loop variable |
| ZK0413 | a read through a value that is not a pointer |
| ZK0414 | a read through, or an offset of, the opaque `လိပ်စာ` |
| ZK0415 | an operator such as `*` or `/` on a pointer |
| ZK0416 | pointers of different pointer kinds compared or subtracted |
| ZK0430 | a declaration with the null address and no data kind |
| ZK0468 | pointer work outside unsafe code |
| ZK0706, ZK0709 to ZK0712 | findings of the pointer lint, in [The checks on unsafe code](unsafe.md#the-checks-on-unsafe-code) |

The text of each message is in [Error messages](../reference/errors.md).
