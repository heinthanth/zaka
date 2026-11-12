---
title: Unsafe Zaka
description: Marked code, the pointer kind, addresses, pointer arithmetic, the heap statements, owned markers, the conversions to and from pointers, and the checks the compiler runs on marked code.
---

# Unsafe Zaka

Safe Zaka has no pointer to read through and no block to give back: each value is
copied, each shared object is counted, and the VM frees a block when its last use goes
(see [Values and objects](objects.md)). Unsafe Zaka adds what C has: a pointer kind,
reads and writes through an address, pointer arithmetic, and heap blocks that the program
asks for and gives back itself. Each of these needs *marked code*, a function or a block
that states, in words, that the compiler does not vouch for it.

"Unsafe" never means that a mistake goes unseen. The VM still checks each read, write and
give-back through a pointer, and stops the program at the first bad one. What changes is
the compiler: in marked code its findings about pointers are warnings, and it leaves the
proof to the program. This page states the whole rules. The Learn book builds them up in
small steps, starting from [the unsafe chapter](../../learn/pointers/README.md).

## Marked code

There are two marks. `စိတ်မချရသော` ("not to be trusted") before `လုပ်ဆောင်ချက်` marks a
whole function:

```text
စိတ်မချရသော လုပ်ဆောင်ချက် <name> သည် ... အောက်ပါတို့ကို လုပ်ဆောင်မည်။
```

A function declared inside a marked function is marked too. The block phrase
`အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။` ("do the following, though it cannot be
trusted") opens a marked block, a statement that holds others, as each block does; the
block ends with `လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။`. The mark covers the statements of
the block and nothing past its closer.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ညွှန်း သည် အသက် ၏ လိပ်စာ ဖြစ်ပါစေ။
ညွှန်း ၏ တန်ဖိုး ကို ၂၀ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အသက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၀
```

The start block itself cannot hold the mark: `ပရိုဂရမ်သည်` takes the plain
`အောက်ပါတို့ကို လုပ်ဆောင်မည်။` and nothing else (ZK0201). A marked block goes inside it,
as above.

```zaka-error ZK0201
ပရိုဂရမ်သည် အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
"က" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0201]: expected "အောက်ပါ တို့ကို လုပ်ဆောင်မည်" or "အောက်ပါ တို့ကို လုပ်ဆောင်ကာ", found "အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်"
```

The block of a conditional, a switch arm or a loop can take the marked opener in place of
the plain one: its statements are then marked code, and nothing else of the statement is.
The condition and the loop head remain safe code.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ၅ ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
အကယ်၍ က > ၀ ဖြစ်လျှင် အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ပ ကို က ၏ လိပ်စာ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
၁ မှ ၂ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ပ ၏ တန်ဖိုး ကို (ပ ၏ တန်ဖိုး) + န ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
က ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၈
```

Both marks share the stem `စိတ်မချရ`, so a search for that word finds each place the
compiler does not vouch for, the calls of marked functions included.

## What needs marked code

A pointer value can pass through safe code: a variable or a field can hold it, `=` and
arguments pass it on, it prints, and it can be compared with the null address,
`မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`. That is how a safe
wrapper keeps a handle that a C lib gave it. The work that reaches the storage behind a
pointer needs marked code, and outside it each is ZK0468:

| Work | Written |
| --- | --- |
| a read or a write through a pointer | `ပ ၏ တန်ဖိုး`, `*ပ` |
| pointer arithmetic | `ပ + ၁`, `ပ - မ` |
| taking an address | `က ၏ လိပ်စာ`, `&က` |
| the heap statements | asking for, resizing, giving back, writing and filling a block |
| a conversion to or from a pointer | `ပ ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်` |
| a call of a marked function, or of a C function marked `စိတ်မချရသော` | `လုပ်ဆောင်ချက် ဂဏန်း ကို လုပ်ဆောင်မည်။` |

```zaka-error ZK0468
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
အကယ်၍ ပ သည် မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ နှင့် ညီလျှင် "မညွှန်း" ဟု ပြသမည်။
ပ ၏ တန်ဖိုး ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0468]: "၏ တန်ဖိုး" needs marked code
```

The comparison on line 3 is fine in safe code; the read on line 4 is not.

The call rule is the one Rust has: a marked function needs marked code at each call, so
the mark spreads to the caller until a function takes the risk on itself. A *safe
wrapper* does that: a function with no mark, whose statements hold a marked block, and whose
callers need no mark at all.

```zaka
စိတ်မချရသော လုပ်ဆောင်ချက် ဂဏန်း သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို အဖြေ တွင် သိမ်းထားမည်။
အဖြေ ၏ တန်ဖိုး ကို ၅ ဟု ထားမည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

စိတ်မချရသော လုပ်ဆောင်ချက် ရှင်း သည်
ပ ဟူသော ကိန်းပြည့် လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// a safe wrapper: no mark on the function, a marked block inside
လုပ်ဆောင်ချက် ငါး သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် လုပ်ဆောင်ချက် ဂဏန်း ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
အဖြေ ကို ပ ၏ တန်ဖိုး ဟု ထားမည်။
လုပ်ဆောင်ချက် ရှင်း ကို ပ တို့ဖြင့် လုပ်ဆောင်မည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ငါး ကို လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၅
```

The same call of `ဂဏန်း` from the start block, outside a marked block, is ZK0468:

```zaka-error ZK0468
စိတ်မချရသော လုပ်ဆောင်ချက် ဂဏန်း သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို အဖြေ တွင် သိမ်းထားမည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် q သည် လုပ်ဆောင်ချက် ဂဏန်း ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0468]: a call of "ဂဏန်း", a function marked "စိတ်မချရသော", needs marked code
```

A call through a [function value](function-values.md) is a plain call, which safe
code can make, so a marked function cannot become a value (ZK0469). Take a function with
no mark that calls it inside a marked block, and make that the value.

```zaka-error ZK0469
စိတ်မချရသော လုပ်ဆောင်ချက် ဂဏန်း သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို အဖြေ တွင် သိမ်းထားမည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် ဂဏန်း ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0469]: "ဂဏန်း" is marked "စိတ်မချရသော", so it cannot be a function value
```

## The pointer kind

A pointer holds an address in the RAM of the VM. Its kind is the kind of the value it
points at, with `လိပ်စာ` ("address") after it, and a pointer to a pointer adds
`၏ လိပ်စာ` for each further level. There is one pointer kind for each kind it points at:
no flavour tells who gives the block back.

| Kind | What it is | Short name in messages |
| --- | --- | --- |
| `ကိန်းပြည့် လိပ်စာ` | the address of an integer | `isize*` |
| `ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ` | the address of a pointer to an integer | `isize**` |
| `လိပ်စာ` | an *opaque* address: it points at a value of no known kind | `ptr` |

The kinds of numbers, characters and true/false values are in [Kinds and
values](kinds.md); each can stand in front of `လိပ်စာ`, and so can a struct or a list.
The first level also takes the older spelling `ကိန်းပြည့် ၏ လိပ်စာ`, which means the same
as `ကိန်းပြည့် လိပ်စာ`.

Pointers of different kinds do not mix. A pointer to an integer does not fit a variable
that points at a smaller integer:

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

Two changes of kind need no words: the null address turns into each pointer kind, and a pointer of
a named kind turns into the opaque `လိပ်စာ` and back. Each other change is a
[conversion](#conversions).

## The null address

`မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` ("the address that points at nothing") is the null
address: a pointer that points nowhere. The pointer is the one kind of Zaka that can hold
NULL; a shared object and a function value cannot, and a value that can be missing is an
[optional](optionals.md#null-is-a-mistake). The phrase is the
[initial value](kinds.md#initial-values) of each pointer kind, so
`ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။` sets `ပ` to it. A pointer variable declared with no value holds
no address at all, not even this one: it is unset until the program sets it ([Set before
read](statements.md#set-before-read)). Zaka does not treat a pointer as true or false, so a
test compares with the phrase, which safe code can do. A null pointer prints as the phrase
too, so the output reads as the source does:

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

- The phrase is three reserved words that the lexer joins into one value:
  `မည်သည်ကိုမှ` ("nothing"), `မညွှန်းသော` ("that points at no"), `လိပ်စာ`.
- It names no pointer kind of its own, so a declaration with it and no written kind is
  ZK0430, and it fits no kind but a pointer (ZK0401).

A pointer that holds an address prints as `0x` and the address in hexadecimal. Reading or
writing through the null address stops the program. When the compiler can prove that a
pointer is null on each path to a read, it reports ZK0709, and under `--mem-strict` it
warns (ZK0711) about a pointer that *can* be null, such as a new block that was never
compared with the phrase. In marked code both follow `--mem-check`; see [The checks on marked
code](#the-checks-on-marked-code).

## Taking an address

`<place> ၏ လိပ်စာ` is the address of a place, and the short form is `&<place>`. Its kind
is the kind of the place with `လိပ်စာ` after it. A place that has an address is:

- a variable or a parameter,
- a value reached through a pointer,
- a field of a struct, and
- a cell of a list or of text.

A number such as `၅`, and the result of a sum, are values with no place, so there is no
address to take (ZK0411). A constant does have a place, but Zaka does not give its
address out, since a write through it would change a constant. The variable of a counted
loop is the same (ZK0412).

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
the address of a top-level variable is good for the whole run. A function that returns
the address of its own local or parameter gets the warning ZK0706, whatever its pointer kind, and the VM stops a read through
that address once the function has returned.

### Pinning

A list or text value shares its cells with its copies until one side writes (see [Values
and objects](objects.md)). An address of its cells would break that: a write through the
address would reach each duplicate. So taking the address of a cell first gives the list cells
of its own, and then *pins* them: a later duplicate of a pinned list gets cells of its own at
once, and never shares them.

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

A pin lasts as long as the cells do, and a resize keeps it. An add to the list can still
move its cells to a new block; the VM stops a read through an address of the old cells.

## Reading and writing through a pointer

`<pointer> ၏ တန်ဖိုး` is the value at the address, and the short form is `*<pointer>`.
The result is a place: it can be read, printed, or assigned to.

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

The two spellings group in a different order, and the difference matters when the pointer
is part of a sum:

| Written | Reads as |
| --- | --- |
| `ပ + ၁ ၏ တန်ဖိုး` | the value at `ပ + ၁`: the verb takes the whole expression before it |
| `*ပ + ၁` | `(*ပ) + ၁`: the star takes the name right after it |
| `(ပ ၏ တန်ဖိုး) + ၁` | the same as `*ပ + ၁` |

Reading through something that is not a pointer is ZK0413. The opaque `လိပ်စာ` cannot be
read through, and cannot be offset, until it is converted to a pointer of a named kind
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

A pointer moves in steps of whole values. `ပ + ၁` is the next value, not the next octet,
and the pointer kind tells the size in octets. These operations alone exist on pointers:

| Form | Kinds | Gives |
| --- | --- | --- |
| `ပ + န`, `န + ပ`, `ပ နှင့် န ၏ ပေါင်းလဒ်` | a pointer and an integer | a pointer of the same kind, `န` values on |
| `ပ - န`, `ပ နှင့် န ၏ နုတ်လဒ်` | a pointer and an integer | a pointer of the same kind, `န` values back |
| `ပ - မ`, `ပ နှင့် မ ၏ နုတ်လဒ်` | two pointers of one kind | the distance in values, a `ကိန်းပြည့်` |
| `ပ သည် မ နှင့် ညီ`, `မညီ` | two pointers of one kind, or a pointer and the null address | whether both hold the same address |
| `ပ သည် မ အောက် ငယ်` and the other orders | two pointers of one kind | which address is lower |

Each other operator on a pointer, such as `*` or `/`, is ZK0415. Pointers of different
kinds cannot be subtracted or compared (ZK0416). Two pointers into different blocks have
no distance: the compiler warns (ZK0712) when it knows the blocks differ, and the VM stops
the subtraction.

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

A block of `n` values holds the values `ပ + ၀` to `ပ + (n - ၁)`. The address `ပ + n`, just
past the last value, is allowed as well: a program can form it, compare against it and
subtract it. It can never be read or written, since no value is stored there. Each other
address outside the block is out of bounds: a constant offset past `n` is ZK0710 when the
pointer is formed, and a read or write at an address outside the block stops the program.

That rule is what a walk through a block needs. A loop stops when the walker meets the
end:

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

The VM allows the address `ပ + n` likewise: a subtraction of two pointers counts the end
of the block as inside it. A read at that address stops the program with an access
outside each live block.

## Size of a kind

`<kind> တစ်<word> ၏ အရွယ်အစား` ("the size of one of the kind") is the number of octets one
value of the kind takes; the kind takes its [measure word](kinds.md#measure-words). It is a constant, a `လက္ခဏာမဲ့ ကိန်းပြည့်`, and an operand like a number; it
reads no storage, so safe code can use it.

| Kind | Octets |
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

The numbers are those of the target the program is built for. Each value sits at an
address that is a multiple of its size, up to 16; the VM stops a program that reads or
writes at another address.

## Heap statements

Three statements work with blocks of the heap, and [three more](#writing-and-filling-cells)
write the cells of a block. A block from these statements has no count: the VM never
frees it on its own, and the program gives it back. In each statement, `မင်မိုရီ` can be
replaced with `မှတ်ဉာဏ်`. Resizing and giving back start alike, with
`လိပ်စာ <place> ရှိ မင်မိုရီကို` ("the RAM at the address in the place"); the words after it
tell the two apart.

### Sizes

A request names its size in one of two forms:

| Form | Asks for |
| --- | --- |
| `<kind> <count> <word>စာ` | room for `count` values of the kind, `<word>` its [measure word](kinds.md#measure-words): `ကိန်းပြည့် ၃ ခုစာ`, `စာလုံး ၅ လုံးစာ`, `လူ ၂ ယောက်စာ` |
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
that is not null and holds nothing. When the heap has no room left, the place gets the
null address; a careful program compares with `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` before it
uses the block. The VM has
256 MiB of heap to start with, and `zakac run --max-heap <octets>` sets another limit.

### Resizing a block

```text
လိပ်စာ <place> ရှိ မင်မိုရီကို <size> ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။
```

The statement grows or shrinks the block whose address is in the place, and stores the
address of the result back in the place. The block can move. The values that fit the new
size are kept, and a larger block holds new room after them. A place that holds the null address
makes the statement ask for a new block. When the heap has no room for the new size, the
program stops with "no heap room" rather than setting the place to the null address, which
would lose the block still held.

### Giving a block back

```text
လိပ်စာ <place> ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

The statement gives the block back and sets the place to the null address. Giving back a
place that holds the null address does nothing. The place alone is cleared; another pointer to the block would
point at a block that is gone. The VM stops a program that uses such a pointer, and the
compiler warns before the program runs when it can see the mistake (ZK0704).

This program asks for a block of two integers, grows it to three, gives it back twice
(the second time is harmless) and prints the place:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၁၀ ဟု ထားမည်။
ပ + ၁ ၏ တန်ဖိုး ကို ၂၀ ဟု ထားမည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ကိန်းပြည့် ၃ ခုစာ ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။
ပ + ၂ ၏ တန်ဖိုး ကို ၃၀ ဟု ထားမည်။
၀ မှ ၂ အထိ န အနေဖြင့် ရေတွက်၍ ပ + န ၏ တန်ဖိုး ကို ပြသမည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ပ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၀
၂၀
၃၀
မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ
```

A heap statement whose place is not a pointer is ZK0417:

```zaka-error ZK0417
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ၈ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို က တွင် သိမ်းထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0417]: the heap statement needs a pointer, found isize
```

## Writing and filling cells

Three statements write a whole run of cells at once. Each starts as resizing does, with
`လိပ်စာ <ptr> ရှိ`, and each has a size, in a form of [the heap statements](#heap-statements), of the cells it handles:

| Statement | Shape |
| --- | --- |
| write the cells, no overlap | `လိပ်စာ SRC ရှိ SIZE မင်မိုရီကို လိပ်စာ DST ဆီသို့ ကူးယူမည်။` |
| write the cells, overlap allowed | `လိပ်စာ SRC ရှိ SIZE မင်မိုရီကို လိပ်စာ DST ဆီသို့ ပြောင်းထည့်မည်။` |
| fill | `လိပ်စာ DST ရှိ မင်မိုရီကို VALUE ဖြင့် SIZE ဖြည့်မည်။` |

`SIZE` is `<kind> N <word>စာ`, a count of cells of the kind with its measure word, or `N ဘိုက်စာ`, a count of octets.
The first two write the cells at `SRC` to `DST`, and the third sets each cell at `DST` to
`VALUE`. `ကူးယူမည်` is the `memcpy` of C: its two ranges must not overlap.
`ပြောင်းထည့်မည်` is the `memmove` of C: it reads the source in full before it writes, so
the ranges can overlap, as when a run of cells moves one place on.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို က တွင် သိမ်းထားမည်။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ခ တွင် သိမ်းထားမည်။

လိပ်စာ က ရှိ မင်မိုရီကို ၇ ဖြင့် ကိန်းပြည့် ၄ ခုစာ ဖြည့်မည်။       // fill
က ၏ တန်ဖိုး ကို ၁ ဟု ထားမည်။
က + ၁ ၏ တန်ဖိုး ကို ၂ ဟု ထားမည်။
က + ၂ ၏ တန်ဖိုး ကို ၃ ဟု ထားမည်။
လိပ်စာ က ရှိ ကိန်းပြည့် ၄ ခုစာ မင်မိုရီကို လိပ်စာ ခ ဆီသို့ ကူးယူမည်။  // two blocks
လိပ်စာ က ရှိ ကိန်းပြည့် ၃ ခုစာ မင်မိုရီကို လိပ်စာ က + ၁ ဆီသို့ ပြောင်းထည့်မည်။  // one block
၀ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ "{က + န ၏ တန်ဖိုး} {ခ + န ၏ တန်ဖိုး}" ဟု ပြသမည်။
လိပ်စာ က ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လိပ်စာ ခ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁ ၁
၁ ၂
၂ ၃
၃ ၇
```

The rules:

- `SRC` and `DST` are values of each pointer kind, or the opaque `လိပ်စာ`. Neither need
  be a variable (`က + ၁` serves), since nothing is written back to them. The kinds of
  `SRC`, `DST` and the cells need not match. A value that is no pointer is ZK0417.
- The count is a `လက္ခဏာမဲ့ ကိန်းပြည့်`, as for the other heap statements (ZK0401 if it
  is not).
- The value of a fill is of the kind of the cells when the size is `<kind> N <word>စာ`, and a
  `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်` when the size is `N ဘိုက်စာ`; with another kind it is ZK0401.
  A struct value fills as well: `အမှတ်(ဒေါင်: ၁၊ အလျား: ၂) ဖြင့် အမှတ် ၃ ခုစာ`.
- A `ဖြင့်` right after a lone value ends the value, while one inside a phrase such as a
  division belongs to it: in `က ကို ၂ ဖြင့် စားသော စားလဒ် ဖြင့် ၁ ဘိုက်စာ` the value is the
  quotient.
- A count of 0 does nothing, and neither the compiler nor the VM checks the addresses.
  Else the VM checks that the source range lies in one live block and the destination
  range in one live block that can be written, and each failure stops the program as an
  access does: out of bounds, use after free, a null address, a write into static text. A
  count whose product with the size of a cell does not fit is out of bounds as well.
- `ကူးယူမည်` whose ranges overlap gets the warning ZK0721 when both addresses point into
  one block at offsets the compiler knows and the count is known; else the VM stops the
  program with the trap `overlap` (see [all the traps](../vm/traps.md)). Use
  `ပြောင်းထည့်မည်` when the ranges can overlap.
- The cells must hold no counted block (ZK0442): text, a plain list, a shared object, a
  function value, or a struct that holds one of these. Each such cell holds a count on its
  block, and a duplicate made octet for octet would give the block a second holder that
  the count does not know of. The cells are of the kind of the size, or, for a size in
  octets, of the kind each address points at. Assign such cells one at a time. Cells
that hold pointers are plain addresses, as in C, and a block write or a fill takes them.

```zaka-error ZK0442
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် စာသား လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် စာသား လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် စာသား ၂ ခုစာ နေရာယူ၍ လိပ်စာကို က တွင် သိမ်းထားမည်။
မင်မိုရီတွင် စာသား ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ခ တွင် သိမ်းထားမည်။
လိပ်စာ က ရှိ စာသား ၂ ခုစာ မင်မိုရီကို လိပ်စာ ခ ဆီသို့ ကူးယူမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0442]: the cells are "text", which hold counted blocks: a raw duplicate or fill would not keep their counts
```

The compiler sees the destination as a write through the address, as a store through a
pointer is: a destination that is the null address for sure, or whose first or last cell lies past
the end of its block, gets a finding, and what the compiler knew about the slots the
destination names is forgotten. It sees the source as a read through the address, with
the same checks of its first and last cell. The last cell is known when the count is known
on each path.

## Owned markers

A pointer tells nothing about who gives its block back. The marker `အပိုင်အနေဖြင့်` ("as
its own") on a parameter or a result states it, and the compiler checks it.

**An owned parameter** takes the block over. The parameter carries the marker after its
`ကို` (see [Functions](functions.md#parameters)):

```text
<name> ဟူသော <kind> လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့်
```

After the call, the caller's variable holds the null address: the call stores it there, and the
function is now the one to give the block back, to hand it on, or to store it. An
argument for an owned parameter must be a place, so that it can be cleared. A read of the
cleared place after a hand-over on each path is ZK0701, an error in each mode:

```zaka-error ZK0701
စိတ်မချရသော လုပ်ဆောင်ချက် ရှင်း သည်
ပ ဟူသော ကိန်းပြည့် လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
လုပ်ဆောင်ချက် ရှင်း ကို ပ တို့ဖြင့် လုပ်ဆောင်မည်။
ပ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0701]: "ပ" handed its block to a parameter marked "အပိုင်အနေဖြင့်" and now points at nothing
```

A hand-over on one branch alone leaves the place null on that branch; the compiler is
silent, and the VM stops a read through it. An argument that another argument of the
same call also reaches gets the warning ZK0707, since the function can give the block back
while it still reads it through the other one. With `ဒေတာ ပြောင်းလဲရန်` beside it, the
function takes the old block, and the caller owns whatever the function writes back.

**An owned result** gives the caller a block:

```text
<kind> လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။
```

The caller is then the one to give it back, hand it on or store it. A function that
returns a block it asked for, with no marker on its result, gets the leak warning ZK0708
with a help line that asks for the marker.

The marker hands over a block, so it belongs on pointer kinds alone. On another kind it is
ZK0470: other values are copied or shared, and hand nothing over. The one other place is a
parameter of a C function whose kind comes from outside the program, which the C function
closes ([Functions](functions.md#c-functions)).

```zaka-error ZK0470
လုပ်ဆောင်ချက် ယူ သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို အပိုင်အနေဖြင့် လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ယူ ကို ၁ တို့ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0470]: "အပိုင်အနေဖြင့်" hands over a block, and "isize" holds no pointer to one
```

## Conversions

The conversion verb `ကို <kind> တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်` (see [Conversions](conversions.md))
turns a pointer into another kind, and other kinds into pointers. Each such conversion
needs marked code:

| From | To | What happens |
| --- | --- | --- |
| a pointer | a pointer of another kind, or `လိပ်စာ` | the same address, read as the new kind |
| a pointer | an integer | the address as a number |
| an integer | a pointer | the number as an address; the VM checks each use |
| a shared object | a pointer | the address of the object, and the pointer holds a count of it |
| a pointer | a shared object | the object again, and the count the pointer held comes back with it |

The last two take an object across a border that knows addresses alone, such as the
user data of a C callback. The object lives on while the pointer holds its count; a
pointer that never turns back into an object keeps it alive to the end, and `--leaks`
lists it.

```zaka
အကောင့် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု၊ လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

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

`ခ` is the object `က` names, so the write through `ခ` shows through `က`.

## C functions

A C function is declared with `"<lib>" မှလာသော လုပ်ဆောင်ချက်` (see [C
functions](functions.md#c-functions)). The declaration is the point of trust. A
declaration whose parameters or result hold a pointer must hold the mark `စိတ်မချရသော`
(ZK0472), and each call of it then needs marked code. Other C functions, on numbers, text,
lists of numbers, plain structs and handles, with a result that can be an optional of text or
of a handle (C gives NULL for it), are called from safe code.

```zaka-error ZK0472
လုပ်ဆောင်ချက် frexp သည် "libm" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
x ဟူသော ဒဿမကိန်း တစ်ခု ကို လည်းကောင်း e ဟူသော ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း လက်ခံပြီး ဒဿမကိန်း တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"က" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0472]: "frexp" passes a pointer, so its declaration needs the mark "စိတ်မချရသော"
```

In the VM, a pointer that lies in a freed block, passed to a C function, stops the run with
the use-after-free trap before the call. A null pointer goes to C as NULL, and the contract of
the C function decides whether NULL is allowed: `frexp` with a NULL address crashes inside
libm, as it does in C.

A closure can reach C: it is lent to the call, and a leave lets C keep it (see
[Closures and C](function-values.md#nested-functions-and-closures)). A call from C to a closure with no
call running and no leave stops the program (the trap `closure escape`).

## The checks on marked code

The compiler follows each pointer of marked code through each path, and reports what it
can prove wrong. These findings are the *pointer lint*:

| Code | Finding |
| --- | --- |
| ZK0702 | giving back or resizing through a pointer that can hold the address of a variable, or of a cell or field it holds in place, alone, never a heap block |
| ZK0703 | an address stored where it can outlast the place it points at |
| ZK0704 | a block given back or resized while another pointer still reads it, after which that read uses a freed block; a block given back twice |
| ZK0706 | a function that gives back the address of its own local or parameter |
| ZK0707 | an argument for an owned parameter that another argument of the call also reaches |
| ZK0708 | a block this function asked for that is never given back, handed to an owned parameter, returned as an owned result, or stored in a heap block, a global or a field |
| ZK0709 | a read or a write through a pointer that is the null address on each path |
| ZK0710 | an offset past the end of a block |
| ZK0711 | a read or a write through a pointer that can be the null address (with `--mem-strict`) |
| ZK0712 | two pointers into different blocks subtracted |
| ZK0721 | the two ranges of `ကူးယူမည်` overlap for certain |

`--mem-check` sets the level of the lint:

| Setting | The lint |
| --- | --- |
| `--mem-check warn` (the default) | warnings: the program runs, and the VM stops it if it reaches the bad access |
| `--mem-check error` | errors: nothing runs |
| `--mem-check off` | nothing: the VM is the one check left |

ZK0711 is a warning in the modes `error` and `warn`. ZK0702 to ZK0708, ZK0712 and ZK0721
come from pointers alone, so the setting moves them wherever the compiler finds them. ZK0709, ZK0710
and ZK0711 can also come from safe code, from a function value or a list: outside marked code these keep
their own level whatever the setting, and the setting moves them inside marked code alone. `--allow`, `--warn` and `--error` set
one group or code of the lint on its own, over `--mem-check`; see
[Commands](../compiler/commands.md#options-of-the-compile-stages).
ZK0701 is an error in each mode.

The default runs a program the lint finds fault with, so the VM shows where it goes
wrong:

```zaka-warning ZK0704
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် မ သည် ပ ဖြစ်ပါစေ။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားမည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
မ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0704]: the block of "ပ" is freed here, but "မ" still points into it and is used later
the program stopped: access at 0x101000 inside a freed block
```

Under `--mem-check error` the same program does not compile:

```zaka-error ZK0704 mem-check=error
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် မ သည် ပ ဖြစ်ပါစေ။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားမည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
မ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

A leak is a warning too. The program runs to its end, and `zakac run --leaks` lists the
block that was never given back:

```zaka-warning ZK0708
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားမည်။
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```console
$ zakac run --leaks leak.zaka
warning[ZK0708]: the block that "ပ" holds is never given back or handed over on this path
၇
leak: block at 0x101000, 8 octets
```

The lint cannot find each mistake: an offset that comes from input, or a pointer that a
C function gave, is out of its reach. [Compiler checks](../compiler/checks.md) shows one
program under each setting.

## What the VM still stops

In marked code as in safe code, the VM checks each read, write and give-back through a
pointer, and stops the program at the first bad one, with a message that names the block:

- a read or a write through the null address;
- a read or a write outside each live block, past the end of a block among them;
- a read or a write in a block that was given back, or in the frame of a call that has
  returned;
- giving back or resizing an address that is not the start of a live heap block, or a
  block given back twice;
- two pointers into different blocks subtracted, and the overlapping ranges of
  `ကူးယူမည်`;
- a read or a write at an address that is not a multiple of the size.

[When a program stops](../vm/traps.md) lists each message. A native build behaves as C
does, with the checks that `--checks` turns on (see [Native programs](../compiler/native.md)).

## Keeping to safe Zaka

`--no-unsafe` refuses each mark, for a lesson or a project that keeps to safe Zaka. Each
marked function and each marked block is then ZK0471, an error in each mode:

```zaka-error ZK0471 no-unsafe=on
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
"က" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0471]: this program keeps to safe Zaka, so it cannot mark code "စိတ်မချရသော"
```

A program that compiles under `--no-unsafe` never reaches a pointer mistake: the VM
frees each block, and no read goes through an address the program made.

## Errors at a glance

| Code | Mistake |
| --- | --- |
| ZK0201 | a mark on the start block |
| ZK0401 | a pointer of one kind where another kind is wanted |
| ZK0410 | an assignment to something that is not a variable or a value through a pointer |
| ZK0411 | the address of a value that has no place, such as `၅` |
| ZK0412 | the address of a constant or of a counted-loop variable |
| ZK0413 | a read through a value that is not a pointer |
| ZK0414 | a read through, or an offset of, the opaque `လိပ်စာ` |
| ZK0415 | an operator such as `*` or `/` on a pointer |
| ZK0416 | pointers of different kinds compared or subtracted |
| ZK0417 | a heap statement given something other than a pointer |
| ZK0442 | cells that hold a counted block, in a block write or a fill |
| ZK0468 | pointer work, or a call of a marked function, outside marked code |
| ZK0469 | a marked function taken as a function value |
| ZK0470 | `အပိုင်အနေဖြင့်` on a kind that is no pointer |
| ZK0471 | marked code under `--no-unsafe` |
| ZK0472 | a C declaration with a pointer, with no mark |
| ZK0701 | a read of a place after it handed its block to an owned parameter |
| ZK0702 to ZK0712, ZK0721 | the pointer lint, [above](#the-checks-on-marked-code) |

The text of each message is in [Error messages](../reference/errors.md).
