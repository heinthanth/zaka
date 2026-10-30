---
title: Ownership and borrowing
description: Each heap block has one owner; lend it with တန်ဖိုးအငှား, and the compiler finds the pointer mistakes before the program runs.
---

# Ownership and borrowing

Each block from the heap should be given back once, and no pointer should use it after
that. Zaka keeps track of who is in charge of each block: one pointer *owns* it, and the
others *borrow* it. With that, the compiler finds the [pointer mistakes](mistakes.md)
before the program runs, and points at the line at fault.

## Three kinds of pointer

| Kind | What it is |
| --- | --- |
| `ကိန်းပြည့် လိပ်စာ` | an *owner*: it holds a block of the heap, and it alone gives the block back |
| `ငှားထားသော ကိန်းပြည့် လိပ်စာ` | a *borrow*: it points at a variable, or at a block that another pointer owns |
| `ကိန်းပြည့် လိပ်စာအကြမ်း` | a *raw* address: the compiler does not follow it, but the VM still checks each use |

`ငှားထားသော` ("lent") stands once, at the front of the kind, and marks the outer level
alone. `ငှားထားသော ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ` borrows a variable that owns a block:
through it, a function can look at the owner and lend its block, or take the block out of
the variable and even give it back. The inner levels of a pointer kind are never borrows,
because a borrow never sits in a heap block or behind another pointer. `ငှားထားသော` in
another place, or in front of a raw address, is an error (ZK0713).

## One owner each

Setting one owner from another *moves* the block: the new variable owns it, and the old
one holds `ဗလာ` from then on.

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

Reading through the old owner after the move is a mistake, and the compiler reports it:

```zaka-error ZK0701
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ။
ကိန်းရှင် မ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ဖြစ်ပါစေ။          // the block moves to မ
ပ ၏ တန်ဖိုး ကို ပြသပါ။                                                 // ပ holds ဗလာ now
မ ၏ တန်ဖိုး ကို ပြသပါ။
လိပ်စာ မ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
error[ZK0701]: "ပ" was moved to another owner and now holds ဗလာ
```

Comparing the old owner with `ဗလာ` is the one read still allowed after a move: it is
how a program asks whether the block has gone. Numbers, characters, `မှန်` and `မမှန်`,
borrows and raw addresses are duplicated, not moved. Owners, and lists that hold owners,
move.

## Lending a block

To let another pointer use a block while the owner keeps it, *lend* the block with
`တန်ဖိုးအငှား` ("the value, on loan"):

```zaka
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် ကြည့် သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။   // a borrow
ကြည့် ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ။                                          // write through it
ပ ၏ တန်ဖိုး ကို ပြသပါ။                                                   // ပ still owns the block
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။                             // after the last use of ကြည့်
```

```text
၇
```

| Zaka | Gives |
| --- | --- |
| `ပ ၏ တန်ဖိုး` | the value in the block: a read |
| `ပ ၏ တန်ဖိုးအငှား` | a borrow of the block `ပ` owns; `ပ` keeps it |
| `x ၏ လိပ်စာ` | a borrow of the variable `x` |
| `ပ + ၁`, `ပ - ၁` | a borrow of a place inside the block |

A borrow reads and writes through the block as often as the program needs, and several
borrows of one block can live at once. A borrow never gives the block back: that is the
owner's task, after the last use of each borrow.

The compiler wants to know which one is meant. An owner written where a borrow is wanted
would move the block, so it is refused, and the message names the fix:

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

The reverse is refused too. A borrow never becomes an owner, because what it points at
has an owner of its own:

```zaka-error ZK0714
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။
```

```text
error[ZK0714]: a borrowed address cannot be given where an owner is wanted
```

## What a function header promises

A function's header tells what it does with each pointer passed in:

| In the header | It promises |
| --- | --- |
| parameter `ငှားထားသော ကိန်းပြည့် လိပ်စာ ပ` | "I look at the block, and can write to it. The caller keeps it." |
| parameter `ကိန်းပြည့် လိပ်စာ ပ` | "Hand the block to me. I give it back, or pass it on." |
| result `ကိန်းပြည့် လိပ်စာ ရလဒ်` | "Here is a new block. The caller owns it now." |
| result `ငှားထားသော ကိန်းပြည့် လိပ်စာ ရလဒ်` | "Look there: inside a block the caller lent me, or at a top-level variable." |

`ပေါင်း` adds up a block it borrows, and `ရှင်း` takes a block and gives it back:

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

The first call lends the block with `ကိန်းများ ၏ တန်ဖိုးအငှား`, so `ကိန်းများ` keeps it.
The second hands the block over: after it, `ကိန်းများ` holds `ဗလာ`, and `ရှင်း` has given
the block back. For these two forms, the call itself tells whether the variable keeps its
block.

A call given the address of the variable, `ပ ၏ လိပ်စာ`, is different. Through a
`ငှားထားသော ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ` parameter, the function can change `ပ` itself, as
on [changing the caller's variables](out-parameters.md). It can even take the block out of
`ပ` and give it back:

```zaka
// drop: take the block out of the caller's variable and give it back
လုပ်ဆောင်ချက် ဖျက် သည်
ငှားထားသော ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ နေရာ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ယူထား သည် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် နေရာ ၏ တန်ဖိုး ဖြစ်ပါစေ။     // the block moves out of ပ
လိပ်စာ ယူထား ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
လုပ်ဆောင်ချက် ဖျက် ကို ပ ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်ပါ။
အကယ်၍ ပ သည် ဗလာ နှင့် ညီလျှင် "ပ တွင် ဘာမှ မကျန်ပါ" ဟု ပြသပါ။
```

```text
ပ တွင် ဘာမှ မကျန်ပါ
```

After such a call, compare `ပ` with `ဗလာ` before reading through it.

## A new block from a function

A function whose result is an owner hands a new block to its caller. The caller owns it
from then on, and gives it back:

```zaka
// number: a new block that holds 5
လုပ်ဆောင်ချက် ဂဏန်း သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် လိပ်စာ ရလဒ် ကို ပြန်ပေးမည်။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ရလဒ် တွင် သိမ်းထားမည်။
ရလဒ် ၏ တန်ဖိုး ကို ၅ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် ဂဏန်း ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။   // ပ owns it now
ပ ၏ တန်ဖိုး ကို ပြသပါ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
၅
```

The address of one of the function's own variables cannot leave like this: see [how long
an address lasts](out-parameters.md#how-long-an-address-lasts).

## A borrow from a function

A function can return a borrow as well. The borrow must point into a block that the
caller lent through a borrow parameter, or at a top-level variable, so the caller knows
what the result borrows:

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

`အကြီး` borrows from `ကိန်းများ`, just as the argument did, so `ကိန်းများ` gives the
block back after the last use of `အကြီး`. In C, a function that moves the caller's
pointer takes a pointer to that pointer. In Zaka it returns the new borrow instead.

## Borrows live in variables

A borrow lives in a local variable, a parameter, or a top-level variable that the
top-level code sets. It cannot be kept in a heap block, written through a pointer, or put
in a top-level variable from inside a function, since what it points at could end first:

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

After the call, `ဝှက်ထား` would point at a variable that is gone. Taking the address of a
borrow is refused as well (ZK0705), so each borrow is reached through its own name.

## Raw addresses

Some programs hand out heap room in a fashion of their own. An *arena* takes one large
block and hands out pieces of it. The pieces are never given back one at a time: the
whole arena goes back at once. No owner or borrow can describe a piece, so the pieces are
raw addresses, written with `လိပ်စာအကြမ်း`:

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

The arena itself, `အခင်း`, is a plain owner, and the compiler checks it like each other
owner. The conversion inside `ယူ` is where the program takes over from the compiler. The
VM still checks each read and write through `က` and `ခ`: a piece used past the end
of the arena, or after the arena is given back, stops the program. Each raw kind is
spelled with `အကြမ်း`, so a search for that word finds each place the compiler does not
check.

A lesson can forbid raw addresses with `zakac run --no-raw`:

```zaka-error ZK0717 no-raw=on
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာအကြမ်း ဖြစ်ပါစေ။
```

```text
error[ZK0717]: raw addresses are not allowed here (--no-raw)
```

## Turning the checks down

| Option | In the Explorer's settings | Effect |
| --- | --- | --- |
| `--mem-check error` | **Ownership check**: **Error** | the default: each pointer mistake the compiler proves is an error, and nothing runs |
| `--mem-check warn` | **Ownership check**: **Warn** | the same mistakes are warnings; the program runs, and the VM stops it at a bad access |
| `--mem-check off` | **Ownership check**: **Off** | the compiler reports none of those mistakes |
| `--mem-strict` | **Strict ownership check** | also warns about a pointer that can be `ဗလာ` |
| `--no-raw` | **Forbid raw addresses** | refuses each raw kind (ZK0717) |

`warn` and `off` change the codes from ZK0701 to ZK0712 alone. The kind rules of this
page, ZK0705 and ZK0713 to ZK0717, are errors in each mode. A move leaves `ဗလာ` behind in
each mode too, so a program does the same thing whichever mode compiles it.

The strict check warns where a pointer can be `ဗလာ`, such as a new block never compared
with `ဗလာ`, since the heap can run out:

```zaka-warning ZK0711 mem-strict=on
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၁ ဟု ထားပါ။             // the heap can run out, and then ပ is ဗလာ
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
warning[ZK0711]: "ပ" can be ဗလာ here; compare it with ဗလာ first
```

The page on [pointer mistakes](mistakes.md) shows each mistake twice: as the compiler
reports it, and as the VM stops it under `warn`.

## Hands on

- Call `လဲလှယ်` from [changing the caller's variables](out-parameters.md) with one block
  lent twice: `ပ ၏ တန်ဖိုးအငှား၊ ပ ၏ တန်ဖိုးအငှား`. Does it compile, and what does it do?
- Write a function that takes an owner and a count, doubles each box of the block, and
  returns the block as its result. Which variable gives the block back?
- Change `ယူ` so that each piece starts at a multiple of 8 octets.

Next: [Pointer mistakes](mistakes.md).
