---
title: Ownership and borrowing
description: The complete rules the compiler enforces for owners, borrows and raw addresses, with one example of each mistake it reports.
---

# Ownership and borrowing

Each block of the heap has to be given back once, and no pointer can use it after that.
Zaka keeps track of this through the kind of each pointer: one pointer *owns* a block, other
pointers *borrow* it, and a few pointers are *raw* and left to the VM. The borrow
checker reads the whole program before it runs and reports each place where the rules
break. This page states the rules in full. The reasons for them are in [Ownership, explained](../concepts/ownership.md), and a slow tour with small programs is in
[Ownership and borrowing](../../learn/pointers/ownership.md) in the Learn book. How the checker
works inside is in [The borrow checker](../internals/borrow-checker.md).

## The three kinds of pointer

| Kind | Written | Made with | Gives the block back? | A plain assignment |
| --- | --- | --- | --- | --- |
| owner | `ကိန်းပြည့် လိပ်စာ` | an allocation, a move, a call whose result is an owner | it alone, once | **moves** the block |
| borrow | `ငှားထားသော ကိန်းပြည့် လိပ်စာ` | `ပ ၏ တန်ဖိုးအငှား`, `x ၏ လိပ်စာ`, `ပ + ၁`, a call whose result is a borrow | no (ZK0702) | duplicates the address |
| raw | `ကိန်းပြည့် လိပ်စာအကြမ်း` | a conversion | it can, and nothing is checked | duplicates the address |

Numbers, characters, true/false values, borrows and raw addresses are duplicated when
these are assigned, passed or returned. Owners move. So do lists whose cells hold owners.

`ငှားထားသော` stands once, at the front of the kind, and marks the outer level alone. The
kind `ငှားထားသော ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ` is a borrow of a variable that owns a block:
through it a function can look at the owner, lend its block, or even take the block out
and give it back. The inner levels of a kind are never borrows, since a borrow never sits
in a heap block or behind another pointer. A `ငှားထားသော` in another place, or in front
of a raw address, is ZK0713:

```zaka-error ZK0713
ကိန်းရှင် ပ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာအကြမ်း ဖြစ်ပါစေ။
```

```text
error[ZK0713]: a pointer kind takes one ငှားထားသော, at its front; a borrow never sits in a heap block or a slot, and a raw address cannot be borrowed
```

## One picture of a block's life

A block is asked for, passed around, lent and given back. This diagram follows one block
through five steps. `a` and `b` are owners, and `r` is a borrow.

```text
              1. ask          2. lend         3. last use     4. move         5. give back
                              r = a lent      of r            b = a           through b

  a           owns ===========================================| null .........................
  b           null ...........................................| owns =========| null .........
  r                           borrows ========| done .........................................
  the block   live ===========================================================| freed ........
```

The rules that this page states can all be read off the picture:

- At each moment, at most one owner holds the block (rules of [One owner](#one-owner)).
- A borrow lives inside the span in which its block is live and its owner has not moved
  it, resized it or given it back ([Loans](#loans-and-their-limits)).
- The owner alone gives the block back, once, and after the last use of each borrow.

## One owner

Setting an owner from another owner *moves* the block. The new variable owns it, and the
old one holds `ဗလာ` from then on.

```zaka
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ။
ကိန်းရှင် မ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ဖြစ်ပါစေ။          // the block moves to မ
အကယ်၍ ပ သည် ဗလာ နှင့် ညီလျှင် "ပ တွင် ဘာမှ မကျန်ပါ" ဟု ပြသပါ။     // ပ holds ဗလာ now
မ ၏ တန်ဖိုး ကို ပြသပါ။
လိပ်စာ မ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။                            // မ gives it back
```

```text
ပ တွင် ဘာမှ မကျန်ပါ
၇
```

A move happens when an owner is given to a place that wants an owner: an assignment, a
declaration with an initial value, a store through a pointer, an argument for an owner
parameter, a returned owner, or a list cell. The move is real at run time: the generated
code stores `ဗလာ` into the old variable, so a program behaves the same whichever mode the
checker ran in.

These are *not* moves: printing an owner, adding to it (`ပ + ၁` is a borrow), lending it,
and comparing it with `ဗလာ`. The comparison is the one read of a moved owner that remains
allowed, since it is how a program asks whether the block has gone.

### Use after a move: ZK0701

Reading an owner that has been moved is an error:

```zaka-error ZK0701
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ။
ကိန်းရှင် မ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ဖြစ်ပါစေ။
ပ ၏ တန်ဖိုး ကို ပြသပါ။
လိပ်စာ မ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
error[ZK0701]: "ပ" was moved to another owner and now holds ဗလာ
```

The compiler reports ZK0701 when the owner has been moved on **each** path that reaches the
read. When it was moved on a single branch, the compiler is quiet, and the VM
stops the program if the read finds `ဗလာ`. A move inside a loop reaches the next
round through the back edge, so a read at the start of the loop is reported.

A list whose cells hold owners moves as a whole when it is assigned or passed.

## Lending a block

To let another pointer use a block while the owner keeps it, *lend* the block with
`ပ ၏ တန်ဖိုးအငှား` ("the value of `ပ`, on loan"):

```zaka
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် ကြည့် သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
ကြည့် ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ။
ပ ၏ တန်ဖိုး ကို ပြသပါ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
၇
```

| Written | Gives |
| --- | --- |
| `ပ ၏ တန်ဖိုး` | the value in the block: a read |
| `ပ ၏ တန်ဖိုးအငှား` | a borrow of the block `ပ` owns; `ပ` keeps it |
| `x ၏ လိပ်စာ` | a borrow of the variable `x` |
| `ပ + ၁`, `ပ - ၁` | a borrow of a place inside the block |

A borrow can read and write as much as the program needs. Several borrows of one block
can be alive at once, and all of them can write: Zaka checks that a block is not used
after it is gone, and does not forbid two names for one value. What a borrow cannot do is
listed in the next sections.

The compiler wants to know which pointer is meant, because an owner written where a
borrow is wanted would move the block. So that place is refused, and the message names
the fix:

```zaka-error ZK0715
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် ကြည့် သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ဖြစ်ပါစေ။
ကြည့် ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ။
ပ ၏ တန်ဖိုး ကို ပြသပါ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
error[ZK0715]: "ပ" owns its block and would move here; write "ပ ၏ တန်ဖိုးအငှား" to lend it
```

The reverse is refused as well. A borrow never becomes an owner, because the
owner of what it points at is somewhere else:

```zaka-error ZK0714
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။
```

```text
error[ZK0714]: a borrowed address cannot be given where an owner is wanted
```

## What a borrow cannot do

### Give a block back: ZK0702

A borrow is a loan, so it has no right to give back or resize the block, whether it points
at a variable, at a block, or into the middle of one:

```zaka-error ZK0702
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
error[ZK0702]: a borrowed address cannot be freed or resized; free its owner instead
```

Asking for a new block into a borrow is ZK0702 too. This is the check that stops a double
free or a "bad free" (giving back a variable's storage, or the middle of a block) before
the program runs.

### Live in a place that outlasts its lender: ZK0703

A borrow lives in a local variable, a parameter, or a top-level variable set from top-level
code. It cannot be stored in a heap block, written through a pointer, or put in a
top-level variable from inside a function, because what it points at could end first:

```zaka-error ZK0703
ကိန်းရှင် ဝှက်ထား သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။          // top level

လုပ်ဆောင်ချက် ဝှက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် x သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
ဝှက်ထား ကို x ၏ လိပ်စာ ဟု ထားပါ။                                        // x ends with the call
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

လုပ်ဆောင်ချက် ဝှက် ကို လုပ်ဆောင်ပါ။
ဝှက်ထား ၏ တန်ဖိုး ကို ပြသပါ။
```

```text
error[ZK0703]: a borrowed address cannot be stored in a heap block, through a pointer, or in a top-level variable from a function
```

Setting a borrow to `ဗလာ` is never ZK0703, since it stores no address.

### Be pointed at in turn: ZK0705

The address of a borrow cannot be taken. With no address of a borrow in the program, each borrow
is reached through its own name, which several of the later rules need.

```zaka-error ZK0705
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
ကိန်းရှင် ကြည့် သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် နောက် သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ကြည့် ၏ လိပ်စာ ဖြစ်ပါစေ။
```

```text
error[ZK0705]: the address of a borrowed address cannot be taken
```

## Loans and their limits

Each borrow of a block is a *loan* from its owner. The loan is **in use** from the moment
the borrow is made to its last read, on each path of the program. While a loan is in
use, the owner cannot give the block back, resize it (the block can move) or move it to
another owner. If the program does, the compiler reports ZK0704 at that statement and
points at the later read.

```zaka-error ZK0704
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ။
ကိန်းရှင် မိတ္တူ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
မိတ္တူ ၏ တန်ဖိုး ကို ပြသပါ။
```

```text
error[ZK0704]: the block of "ပ" is freed here, but "မိတ္တူ" still borrows it and is used later
```

Swap the last two lines, so that the read comes before the give-back, and the program is
correct. A resize and a move are the same mistake:

```zaka-error ZK0704
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် ကြည့် သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ကိန်းပြည့် ၁၀၀ ခုစာ ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။
ကြည့် ၏ တန်ဖိုး ကို ပြသပါ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
error[ZK0704]: the block of "ပ" is resized here, but "ကြည့်" still borrows it and is used later
```

```zaka-error ZK0704
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် ကြည့် သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
ကိန်းရှင် မ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ဖြစ်ပါစေ။
ကြည့် ၏ တန်ဖိုး ကို ပြသပါ။
လိပ်စာ မ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
error[ZK0704]: the block of "ပ" is moved here, but "ကြည့်" still borrows it and is used later
```

A borrow that is set again before its next read is not in use, so lending the block again
after a resize is the fix.

"Used later" is worked out from the paths of the program, not from the order of the
lines. A read that runs once a month, behind a condition, still counts. The compiler also
reads a statement as a whole, without an order inside it: a borrow that is used in a part
of a statement is checked against each call in that statement. A borrow read *before* a
freeing call in the same statement is reported too, though it is safe; split the
statement in two.

A loan is tied to a place inside the owner, not to the name alone. A borrow of the
*variable* `ပ ၏ လိပ်စာ` remains good when the block of `ပ` is given back, since the variable
is still there; a borrow of the *block* does not. Giving back an outer block reaches each
borrow of the blocks stored inside it.

## Escaping addresses

The variables of a function end with the call. Their storage is given back when it returns,
so a borrow of one of them cannot leave the function:

```zaka-error ZK0706
လုပ်ဆောင်ချက် မှား သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ငှားထားသော ကိန်းပြည့် လိပ်စာ ရလဒ် ကို ပြန်ပေးမည်။
ကိန်းရှင် x သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
x ၏ လိပ်စာ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
```

```text
error[ZK0706]: returns a borrow of the local "x", which ends when the function returns
```

The compiler follows the origin of each borrow, so a borrow made from a local through
another local is caught as well. A borrow of a block the caller lent, or of a top-level
variable, is a fine result. The other routes through which an address could escape, a heap
block, a pointer and a top-level variable, are closed through ZK0703 above. A local address
that gets out in a manner the compiler cannot follow, through a raw address, is found
when it is used: the VM stops the program, since the stack of the call
that ended no longer holds that variable.

## Ownership through function calls

The header of a function tells what the function does with each pointer passed in, and
the call has to agree with it.

| In the header | The function promises | The call |
| --- | --- | --- |
| parameter `ငှားထားသော ကိန်းပြည့် လိပ်စာ ပ` | "I look at the block and can write to it. The caller keeps it." | `ပ ၏ တန်ဖိုးအငှား` |
| parameter `ကိန်းပြည့် လိပ်စာ ပ` | "Hand the block to me. I give it back or pass it on." | `ပ`; the caller's variable holds `ဗလာ` afterwards |
| parameter `ငှားထားသော ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ ပ` | "Lend me the variable. I can change what hangs on it, even give the block back." | `ပ ၏ လိပ်စာ` |
| result `ကိန်းပြည့် လိပ်စာ ရလဒ်` | "Here is a new block. The caller owns it." | the caller gives it back |
| result `ငှားထားသော ကိန်းပြည့် လိပ်စာ ရလဒ်` | "Look there: into a block the caller lent, or at a top-level variable." | the borrow is rooted in the borrow arguments |

### Taking

A call that hands a block over moves it. In C this would be a use after free hidden inside
another function; here the header hands it over:

```zaka-error ZK0701
လုပ်ဆောင်ချက် ရှင်းပစ် သည်
ကိန်းပြည့် လိပ်စာ က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လိပ်စာ က ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
လုပ်ဆောင်ချက် ရှင်းပစ် ကို ပ တို့ဖြင့် လုပ်ဆောင်ပါ။          // hands the block over
ပ ၏ တန်ဖိုး ကို ပြသပါ။
```

```text
error[ZK0701]: "ပ" was moved to another owner and now holds ဗလာ
```

### Lending

`ပေါင်း` adds up a block that it borrows, and `ရှင်း` takes a block and gives it back. The
first call lends the block, so `ကိန်းများ` keeps it; the second hands it over:

```zaka
// sum: add up the boxes of a borrowed block
လုပ်ဆောင်ချက် ပေါင်း သည်
ငှားထားသော ကိန်းပြည့် လိပ်စာ က နှင့် ကိန်းပြည့် အရေအတွက် တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
ရလဒ် ကို ၀ ဟု ထားပါ။
ကိန်းရှင် ညွှန်း သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ကာ ညွှန်း သည် က + အရေအတွက် အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ရလဒ် ကို ရလဒ် + *ညွှန်း ဟု ထားပါ။
ပြီးလျှင် ညွှန်း ကို ညွှန်း + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

// clear: take the block and give it back
လုပ်ဆောင်ချက် ရှင်း သည်
ကိန်းပြည့် လိပ်စာ က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လိပ်စာ က ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၃ ခုစာ နေရာယူ၍ လိပ်စာကို ကိန်းများ တွင် သိမ်းထားမည်။
၀ မှ ၂ အထိ နံပါတ် အနေဖြင့် ရေတွက်၍ ကိန်းများ + နံပါတ် ၏ တန်ဖိုး ကို နံပါတ် + ၁ ဟု ထားပါ။
လုပ်ဆောင်ချက် ပေါင်း ကို ကိန်းများ ၏ တန်ဖိုးအငှား၊ ၃ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
လုပ်ဆောင်ချက် ရှင်း ကို ကိန်းများ တို့ဖြင့် လုပ်ဆောင်ပါ။        // the block goes to ရှင်း
```

```text
၆
```

Inside a function the checker cannot see the callers, so it takes the parameters to be
distinct. It then records what the function can free, resize or move through each
parameter and each top-level variable, and checks each call against that record as if
the inner statements stood at the call. So a call that frees a top-level variable's block while a
borrow of it is in use is ZK0704 at the call, and recursion and mutual recursion are
covered.

### Returning a new block

A function whose result is an owner hands a new block to the caller, who owns it from then
on:

```zaka
// number: a new block that holds 5
လုပ်ဆောင်ချက် ဂဏန်း သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် လိပ်စာ ရလဒ် ကို ပြန်ပေးမည်။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ရလဒ် တွင် သိမ်းထားမည်။
ရလဒ် ၏ တန်ဖိုး ကို ၅ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် ဂဏန်း ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ပ ၏ တန်ဖိုး ကို ပြသပါ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
၅
```

### Returning a borrow

A function can return a borrow too, as long as it points into a block that the caller
lent through a borrow parameter, or at a top-level variable, so that the caller knows what
the result borrows. Here `အကြီး` borrows from `ကိန်းများ`, just as the argument did, so
`ကိန်းများ` gives the block back after the last use of `အကြီး`:

```zaka
// biggest: the address of the biggest number in the block
လုပ်ဆောင်ချက် အကြီးဆုံး သည်
ငှားထားသော ကိန်းပြည့် လိပ်စာ က နှင့် ကိန်းပြည့် အရေအတွက် တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ငှားထားသော ကိန်းပြည့် လိပ်စာ ရလဒ် ကို ပြန်ပေးမည်။
ရလဒ် ကို က ဟု ထားပါ။
ကိန်းရှင် ညွှန်း သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် က + ၁ ဖြစ်ကာ ညွှန်း သည် က + အရေအတွက် အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
အကယ်၍ *ညွှန်း သည် *ရလဒ် ထက် ကြီးလျှင် ရလဒ် ကို ညွှန်း ဟု ထားပါ။
ပြီးလျှင် ညွှန်း ကို ညွှန်း + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ကိန်းများ တွင် သိမ်းထားမည်။
ကိန်းများ ၏ တန်ဖိုး ကို ၃ ဟု ထားပါ။
ကိန်းများ + ၁ ၏ တန်ဖိုး ကို ၉ ဟု ထားပါ။
ကိန်းများ + ၂ ၏ တန်ဖိုး ကို ၄ ဟု ထားပါ။
ကိန်းများ + ၃ ၏ တန်ဖိုး ကို ၁ ဟု ထားပါ။
ကိန်းရှင် အကြီး သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် အကြီးဆုံး ကို ကိန်းများ ၏ တန်ဖိုးအငှား၊ ၄ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
အကြီး ၏ တန်ဖိုး ကို ပြသပါ။
အကြီး - ကိန်းများ ကို ပြသပါ။                  // its box number
လိပ်စာ ကိန်းများ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
၉
၁
```

A function that resizes a block and hands back a borrow of the *new* block is fine. If it
can, on some path, hand back a borrow of the *old* block instead, the caller's borrow is
reported when it is used after the resize (ZK0704).

### Two names for one block: ZK0707

A function that frees through one argument has to be the one route into that block during
the call. If another argument reaches the same block, that argument would be left pointing
at freed space. The compiler checks this at the call:

```zaka-error ZK0707
လုပ်ဆောင်ချက် ရှင်း သည်
ငှားထားသော ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ က နှင့် ငှားထားသော ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖတ် သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ခ ၏ တန်ဖိုး ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
လိပ်စာ က ၏ တန်ဖိုး ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ဖတ် ၏ တန်ဖိုး ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
လုပ်ဆောင်ချက် ရှင်း ကို ပ ၏ လိပ်စာ၊ ပ ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်ပါ။
```

```text
error[ZK0707]: "ရှင်း" can free through argument 1, but argument 2 reaches the same block
```

ZK0707 is about *freeing* through an alias alone. A function that just reads and writes
through two borrows can be given the same borrow twice, as `လဲလှယ်` of [Functions](functions.md)
can be given the same variable twice.

## Leaks: ZK0708

An owner that holds a block when it is overwritten, or when its function ends, loses the
block: no pointer is left that could give it back. The compiler reports both places:

```zaka-error ZK0708
လုပ်ဆောင်ချက် စမ်း သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
မင်မိုရီတွင် ကိန်းပြည့် ၈ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။  // the first block is lost
ပ ၏ တန်ဖိုး ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။                                     // the second is lost too

လုပ်ဆောင်ချက် စမ်း ကို လုပ်ဆောင်ပါ။
```

```text
error[ZK0708]: the block that "ပ" owns is never freed or moved on this path
error[ZK0708]: the block that "ပ" owns is never freed or moved on this path
```

- **Overwrite.** Checked in functions and at the top level. It fires when the owner
  holds a block for sure, or holds a fresh one; a state the compiler cannot tell is
  left quiet.
- **End of a function.** Checked for the locals and the parameters of a function. A
  top-level owner that still holds a block when the program ends is not reported; run
  the program with `zakac run --leaks` to list each block that is left.
- **Not a leak.** Returning soon on the `ဗလာ` branch of a fresh allocation, since there
  was no block, and an owner whose block went into a call that can have freed it.

A lost block does no harm at once, so with `--mem-check warn` the program still runs.

## Null and bounds

The compiler follows what each owner can hold. After an `အကယ်၍ ပ သည် ဗလာ နှင့် ညီလျှင်`,
the branch that runs on a match knows `ပ` is `ဗလာ`, and the other branch knows it holds a
block.

- **ZK0709.** Reading or writing through a pointer that is `ဗလာ` on each path to the
  read. A new pointer variable, a moved owner and a freed owner all hold `ဗလာ`.
- **ZK0711**, with `--mem-strict` alone. A pointer that *can* be `ဗလာ` there, most often
  a new block never compared with `ဗလာ`, since the heap can run out.
- **ZK0710.** A box or cell number that is a constant (or that the compiler works out from
  loops with counted bounds), and lies past the end of its block. The variable of a counted
  loop has a known range.
- **ZK0712.** Subtracting two pointers that point into different blocks.

```zaka-error ZK0709
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။     // never given an address
"အစ" ဟု ပြသပါ။
ပ ၏ တန်ဖိုး ကို ပြသပါ။
```

```text
error[ZK0709]: "ပ" is ဗလာ here, so it cannot be dereferenced
```

```zaka-warning ZK0711 mem-strict=on
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၁ ဟု ထားပါ။             // the heap can run out, and then ပ is ဗလာ
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
warning[ZK0711]: "ပ" can be ဗလာ here; compare it with ဗလာ first
```

```zaka-error ZK0710
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ + ၃ ၏ တန်ဖိုး ကို ပြသပါ။     // the last box: fine
ပ + ၄ ၏ တန်ဖိုး ကို ပြသပါ။     // one past the end
```

```text
error[ZK0710]: offset 4 is past the end of a block of 4 elements
```

When a size or an index comes from input, the compiler proves nothing and reports nothing;
the VM's own check stops the program at the bad access. A proof is also given up
when the number could be changed in a manner the compiler cannot follow: a counter whose
address is taken somewhere, or a program that uses a raw kind.

## Raw addresses

Some programs give out heap room in a fashion of their own. An *arena* takes one large
block and hands out pieces of it, and the pieces are never given back one at a time: the
whole arena goes back at once. No owner or borrow can describe such a piece, so the
pieces are raw addresses, written with `လိပ်စာအကြမ်း`.

```zaka
// one block, handed out piece after piece
ကိန်းရှင် အခင်း သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။          // owns the arena
ကိန်းရှင် သုံးပြီး သည် လက္ခဏာမဲ့ ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။     // octets handed out so far
မင်မိုရီတွင် ၆၄ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို အခင်း တွင် သိမ်းထားမည်။

// take: the next piece of the arena, as a raw address
လုပ်ဆောင်ချက် ယူ သည်
လက္ခဏာမဲ့ ကိန်းပြည့် အရွယ် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် လိပ်စာအကြမ်း ရလဒ် ကို ပြန်ပေးမည်။
ရလဒ် ကို အခင်း + သုံးပြီး ကို ကိန်းပြည့် လိပ်စာအကြမ်း အဖြစ် ပြောင်းလဲသော ရလဒ် ဟု ထားပါ။
သုံးပြီး ကို သုံးပြီး + အရွယ် ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် က သည် ကိန်းပြည့် လိပ်စာအကြမ်း ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် ယူ ကို ၁၆ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် လိပ်စာအကြမ်း ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် ယူ ကို ၁၆ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
က ၏ တန်ဖိုး ကို ၄၂ ဟု ထားပါ။
ခ + ၁ ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ။
က ၏ တန်ဖိုး ကို ပြသပါ။
ခ + ၁ ၏ တန်ဖိုး ကို ပြသပါ။
ခ - က ကို ပြသပါ။                                  // ခ starts two integers after က
လိပ်စာ အခင်း ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။      // one free for the whole arena
```

```text
၄၂
၇
၂
```

The arena `အခင်း` is a plain owner, and the compiler checks it like each other owner. The
conversion inside `ယူ` is where the program takes over from the compiler. What remains true
of a raw address:

- The VM checks each read and write through it: a piece used past the end of the
  arena, or after the arena is given back, stops the program.
- A raw address is duplicated on assignment, never moved, and can be given back with the
  heap statement, unchecked.
- A raw address cannot be lent with `တန်ဖိုးအငှား`; convert it to a pointer kind first
  (ZK0716).
- To get a checked pointer from an integer, convert to a raw kind first and then to the
  pointer kind, so that the step remains visible. A conversion between checked pointer
  kinds is refused when it would read stored addresses as other data; it too goes through
  a raw kind.
- Each raw kind is spelled with `အကြမ်း`, so a search for that word finds each place the
  compiler does not check.

```zaka-error ZK0716
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာအကြမ်း ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် ကြည့် သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
```

```text
error[ZK0716]: a raw address cannot be lent with တန်ဖိုးအငှား; convert it with a cast first
```

A lesson or a project can forbid raw addresses altogether with `--no-raw`:

```zaka-error ZK0717 no-raw=on
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာအကြမ်း ဖြစ်ပါစေ။
```

```text
error[ZK0717]: raw addresses are not allowed here (--no-raw)
```

## The options

The rules of this page are errors to start with. The first line of the table is the start:

| Option | In the Explorer's settings | Effect |
| --- | --- | --- |
| `--mem-check error` | **Ownership check**: **Error** | each mistake the compiler proves is an error, and nothing runs |
| `--mem-check warn` | **Ownership check**: **Warn** | the same mistakes are warnings; the program runs, and the VM stops it at a bad access |
| `--mem-check off` | **Ownership check**: **Off** | the compiler reports none of them |
| `--mem-strict` | **Strict ownership check** | also warns about a pointer that can be `ဗလာ` (ZK0711) |
| `--no-raw` | **Forbid raw addresses** | refuses each raw kind (ZK0717) |

`warn` and `off` change the codes ZK0701 to ZK0712 alone. The rules about kinds, ZK0705
and ZK0713 to ZK0717, are errors in each mode. A move leaves `ဗလာ` behind in each mode
too, so a program does the same thing whichever mode compiles it.

### When the VM is the one that stops it

The compiler proves just what the text of the program shows. When it is told to let a
mistake through, or when the mistake depends on input, the VM checks each access
while the program runs and stops at the first bad one. With `--mem-check warn`, each of
these programs compiles with a warning and stops at run time:

```zaka-stop mem-check=warn
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ။
ကိန်းရှင် မိတ္တူ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
မိတ္တူ ၏ တန်ဖိုး ကို ပြသပါ။
```

```text
the program stopped: access at 0x101040 inside a freed block
```

That is a use after free (ZK0704). A double free, from a borrow that gives the block back
a second time (ZK0702):

```zaka-stop mem-check=warn
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် မိတ္တူ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လိပ်စာ မိတ္တူ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
the program stopped: the block at 0x101040 was freed before
```

And a bad free, which gives back something that is no block of the heap (ZK0702):

```zaka-stop mem-check=warn
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
the program stopped: 0x1000 is not the start of a live heap block
```

The stops are listed in [When a program stops](../vm/traps.md), and the checks in [Pointer
checks](../compiler/checks.md).

## Errors at a glance

| Code | Mistake |
| --- | --- |
| ZK0701 | an owner used after its block moved to another owner |
| ZK0702 | giving back or resizing a block through a borrow |
| ZK0703 | a borrow kept in a heap block, behind a pointer, or in a top-level variable from a function |
| ZK0704 | giving back, resizing or moving a block while a borrow of it is still in use |
| ZK0705 | taking the address of a borrow |
| ZK0706 | returning a borrow of a local variable |
| ZK0707 | a call that can give back a block through one argument while another argument reaches the same block |
| ZK0708 | a block lost: its owner is overwritten, or its function ends |
| ZK0709 | a read or write through a pointer that is `ဗလာ` on each path |
| ZK0710 | a box or cell number past the end of a block |
| ZK0711 | a pointer that can be `ဗလာ` (a warning, with `--mem-strict`) |
| ZK0712 | pointers into different blocks subtracted |
| ZK0713 | `ငှားထားသော` in the wrong place of a kind |
| ZK0714 | a borrow given where an owner is wanted |
| ZK0715 | an owner given where a borrow is wanted |
| ZK0716 | a raw address lent with `တန်ဖိုးအငှား` |
| ZK0717 | a raw kind under `--no-raw` |
