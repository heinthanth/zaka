---
title: Pointers and the heap
description: Hold the address of a variable, then read and change the variable through it.
---

# Pointers and the heap

Each variable lives somewhere in the computer's RAM, at a numbered place called its
*address*. A *pointer* is a value that holds an address. A pointer lets a program read
or change a variable without using its name: from inside another function, or in RAM
that has no name at all.

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။                       // age = 18
ကိန်းရှင် ညွှန်း သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် အသက် ၏ လိပ်စာ ဖြစ်ပါစေ။    // a pointer that borrows age
ညွှန်း ၏ တန်ဖိုး ကို ပြသပါ။                                 // read through it
ညွှန်း ၏ တန်ဖိုး ကို ၂၀ ဟု ထားပါ။                              // write through it
အသက် ကို ပြသပါ။                                            // age has changed
```

```text
၁၈
၂၀
```

After the first line, the program never assigns to `အသက်` with its name. It changes the
variable through the pointer `ညွှန်း` ("the pointer").

The pointer *borrows* `အသက်`: the variable keeps its own storage, and the pointer just
looks at it. A kind that borrows starts with `ငှားထားသော` ("lent"). The page on [owners
and borrows](ownership.md) tells the rest; until then, each pointer at a variable is
written like this.

| Zaka | Reads as |
| --- | --- |
| `ကိန်းပြည့် လိပ်စာ` | "integer address": the kind of a pointer to an integer |
| `ငှားထားသော` | "lent": the pointer borrows what it points at |
| `အသက် ၏ လိပ်စာ` | "the address of age" |
| `ညွှန်း ၏ တန်ဖိုး` | "the value at the pointer" |
| `လိပ်စာ` | "address" |
| `တန်ဖိုး` | "value" |

## Pointer kinds

A pointer kind is the kind of the value it points to, with `လိပ်စာ` after it. A pointer
to a pointer adds `၏ လိပ်စာ` ("address of") for each further level:

| Kind | Holds the address of |
| --- | --- |
| `ငှားထားသော ကိန်းပြည့် လိပ်စာ` | an integer, which it borrows |
| `ငှားထားသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ` | an unsigned 8-bit integer, which it borrows |
| `ငှားထားသော ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ` | a pointer to an integer, which it borrows |
| `လိပ်စာ` | a value of whatever kind (an *opaque* address) |

Without `ငှားထားသော`, a pointer kind *owns* a block of the heap; see [The heap](heap.md).

The older spelling with `၏` at the first level, `ကိန်းပြည့် ၏ လိပ်စာ`, still works and
means the same as `ကိန်းပြည့် လိပ်စာ`.

A pointer holds just the addresses of its own kind. The address of a `ကိန်းပြည့်` does not fit
a pointer to an 8-bit integer:

```zaka-error ZK0401
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ငှားထားသော ၈ ဘစ် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။
```

```text
error[ZK0401]: expected &*i8, found &*isize
```

A bare `လိပ်စာ` holds the address of a value of whatever kind, unless that value holds an
address itself, but the program cannot read through it until it states what kind of value
is there. The page on [pointer arithmetic](arithmetic.md) shows how.

## The null address

A pointer that points nowhere holds `ဗလာ` ("blank"), the null address. A new pointer
variable starts out null. A pointer is not true or false, so compare it with `ဗလာ` to
test it:

```zaka
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။     // no address so far
ပ ကို ပြသပါ။
အကယ်၍ ပ သည် ဗလာ နှင့် ညီလျှင် "ဘာမှ မညွှန်ပါ" ဟု ပြသပါ။
```

```text
ဗလာ
ဘာမှ မညွှန်ပါ
```

Reading or writing through a null pointer stops the program. See [Pointer
mistakes](mistakes.md).

## Short forms

Programmers who know C can write `&` for "the address of" and `*` for "the value at":

```zaka
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် &က ဖြစ်ပါစေ။     // the same as က ၏ လိပ်စာ
*ပ ကို *ပ + ၁ ဟု ထားပါ။                              // the same as ပ ၏ တန်ဖိုး
က ကို ပြသပါ။
```

```text
၆
```

Both spellings mean the same thing, but the word forms and the short forms group in a
different order. The page on [pointer arithmetic](arithmetic.md) explains the
difference.

## What has an address

Variables, parameters, `ရလဒ်` and each value reached through a pointer have addresses.
A number such as `၅` does not, and neither does the result of a sum. A constant has an
address in RAM, but Zaka will not give it out, because writing through it would change
the constant. The same holds for the variable of a counted loop:

```zaka-error ZK0412
ကိန်းသေ ကြီးဆုံး သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၀၀ ဖြစ်ပါစေ။
ကြီးဆုံး ၏ လိပ်စာ ကို ပြသပါ။
```

```text
error[ZK0412]: "ကြီးဆုံး" cannot be changed, so its address cannot be taken
```

To see these addresses, run a program in the Explorer with debug mode on. Its heap
pane shows a block for each global and for each local of each call in progress, and
the Block column next to each variable gives the address of its block.

## Hands on

- Declare two integers and one pointer. Point at the first, change it through the
  pointer, then point at the second and change that one.
- Take the address of `၅`. Read the message.
- Declare a pointer to a pointer, `ငှားထားသော ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ`, that points
  at a `ကိန်းပြည့် လိပ်စာ` variable, and print `၏ တန်ဖိုး` through it. After [the
  heap](heap.md), give that variable a block and reach its integer with
  `၏ တန်ဖိုး ၏ တန်ဖိုး`.

Next: [Changing the caller's variables](out-parameters.md).
