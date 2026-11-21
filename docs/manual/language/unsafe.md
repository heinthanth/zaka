---
title: Unsafe Zaka
description: Unsafe functions and unsafe blocks, the work that needs them, safe wrappers, C functions, the checks the compiler runs on unsafe code and the checks the VM still makes.
---

# Unsafe Zaka

Safe Zaka has no pointer to read through and no block (allocation) to free: each value is
duplicated, each shared object is counted, and the VM frees a block when its last use
ends (see [Values and objects](objects.md)). Unsafe Zaka adds what C has: addresses
(pointers), reads and writes through an address, pointer arithmetic, and blocks of the
heap that the program asks for and frees itself. Each of these needs *unsafe code*, which
is a function or a block that states, in words, that the compiler cannot vouch for it.

"Unsafe" does not mean that a mistake goes unseen. On the VM (virtual machine), each read,
write and free through a pointer is still checked, and the program stops at the first bad
one. What changes is the compiler: in unsafe code, its findings about pointers are
warnings, and it leaves the proof to the program. Native code is different, because it
runs a pointer mistake as C would (see [Native code](#native-code)).

The rules of unsafe Zaka fill three pages. This page covers unsafe code itself, C
functions and the checks, [Pointers and addresses](pointers.md) covers the pointer kind
(pointer type) and the work with addresses, and [The heap and owned pointers](heap.md)
covers blocks of the heap and owned pointers (owning pointers). The Learn book builds them
up in small steps, starting from [Unsafe Zaka](../../learn/pointers/README.md).

## Unsafe code

```text
<name> ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် … အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။ … လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

Unsafe code comes in two forms. To make a whole function unsafe, we put `စိတ်မချရသော`
("not to be trusted") between `ဟူသော` and `လုပ်ဆောင်ချက်`, and a function that we declare
inside an unsafe function is unsafe too. To make a few statements unsafe, we put them in
an *unsafe block*, which opens with `အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။` ("do the
following, though it cannot be trusted") and ends with the block closer
`လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။`. An unsafe block is a statement like each other block,
and the statements between its opener and its block closer are unsafe code, while nothing
after the block closer is.

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

Both forms share the stem `စိတ်မချရ`, so a search for that word finds each place that the
compiler cannot vouch for, the calls of unsafe functions included.

### Blocks of conditions and loops

The block of a condition, a switch arm or a loop can use the unsafe opener in place of the
plain one. Its statements are then unsafe code, while the condition and the loop head
remain safe code.

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

### The program start block

The program start block (entry point) cannot be unsafe. After `ပရိုဂရမ်သည်` we must write
the plain `အောက်ပါတို့ကို လုပ်ဆောင်မည်။` (ZK0201), and `စိတ်မချရသော` before `ပရိုဂရမ်သည်` is
ZK0257. To use unsafe code there, we put an unsafe block inside the program start block,
as in the examples above.

```zaka-error ZK0257
စိတ်မချရသော ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"က" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0257]: "စိတ်မချရသော" marks a function, and the start block cannot be unsafe
```

```zaka-error ZK0201
ပရိုဂရမ်သည် အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
"က" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0201]: expected "အောက်ပါ တို့ကို လုပ်ဆောင်မည်" or "အောက်ပါ တို့ကို လုပ်ဆောင်ကာ", found "အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်"
```

## What needs unsafe code

A pointer value can pass through safe code. Safe code can store it in a variable or a
field, pass it on with `=` and with arguments, print it, and compare it with the null
address (null pointer) `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`, which is how a safe wrapper keeps a
handle that a C lib returned. The work that reaches the storage behind a pointer needs
unsafe code, and outside unsafe code each of these is ZK0468:

| Work | Written | Rules |
| --- | --- | --- |
| a read or a write through a pointer | `ပ ၏ တန်ဖိုး`, `*ပ` | [Reading and writing through a pointer](pointers.md#reading-and-writing-through-a-pointer) |
| pointer arithmetic | `ပ + ၁`, `ပ - မ` | [Pointer arithmetic](pointers.md#pointer-arithmetic) |
| taking an address | `က ၏ လိပ်စာ`, `&က` | [Taking an address](pointers.md#taking-an-address) |
| the heap statements | asking for, resizing, freeing, writing and filling a block | [Heap statements](heap.md#heap-statements) |
| a conversion to or from a pointer | `ပ ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်` | [Conversions](pointers.md#conversions) |
| a call of an unsafe function, or of a C function declared `စိတ်မချရသော` | `ဂဏန်း ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။` | [Safe wrappers](#safe-wrappers), [C functions](#c-functions) |

```zaka-error ZK0468
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
အကယ်၍ ပ သည် မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ နှင့် ညီလျှင် "မညွှန်း" ဟု ပြသမည်။
ပ ၏ တန်ဖိုး ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0468]: "၏ တန်ဖိုး" needs unsafe code
```

The comparison on line 3 is fine in safe code, while the read on line 4 is not.

## Safe wrappers

Zaka follows the rule of Rust for calls: each call of an unsafe function needs unsafe
code, so unsafe code spreads to the caller until some function takes the risk on itself.
A *safe wrapper* does that. It is a function that is not unsafe, but contains an unsafe
block, and its callers need no unsafe code at all.

```zaka
ဂဏန်း ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို အဖြေ တွင် သိမ်းထားမည်။
အဖြေ ၏ တန်ဖိုး ကို ၅ ဟု ထားမည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ရှင်း ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည်
ပ ဟူသော ကိန်းပြည့် လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// a safe wrapper: the function is not unsafe, an unsafe block is inside
ငါး ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ဂဏန်း ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
အဖြေ ကို ပ ၏ တန်ဖိုး ဟု ထားမည်။
ရှင်း ဟူသော လုပ်ဆောင်ချက်ကို ပ ဖြင့် လုပ်ဆောင်မည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ငါး ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၅
```

Here `ဂဏန်း` returns an owned pointer and `ရှင်း` takes one over, as [Owned
pointers](heap.md#owned-pointers) describes. The same call of `ဂဏန်း` from the program
start block, outside an unsafe block, is ZK0468:

```zaka-error ZK0468
ဂဏန်း ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို အဖြေ တွင် သိမ်းထားမည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် q သည် ဂဏန်း ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0468]: a call of "ဂဏန်း", an unsafe function, needs unsafe code
```

A call through a [function value](function-values.md) is a plain call that safe code can
make, so an unsafe function cannot become a function value (ZK0469). To pass it around,
we write a safe function that calls it inside an unsafe block, and we use that function as
the value.

```zaka-error ZK0469
ဂဏန်း ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည်
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
error[ZK0469]: "ဂဏန်း" is an unsafe function, declared with "စိတ်မချရသော", so it cannot be a function value
```

## C functions

```text
<name> ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် "<lib>" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ … ပြန်ပေးမည်။
```

We declare a C function with `"<lib>" မှလာသော လုပ်ဆောင်ချက်` (see [C
functions](functions-c.md#c-functions)), and the declaration is the point where we decide to
trust C. C runs outside the checks of Zaka, so each C declaration must include the word
`စိတ်မချရသော`, whatever its data kinds (ZK0472), and each call of the function then needs
unsafe code.

```zaka-error ZK0472
frexp ဟူသော လုပ်ဆောင်ချက် သည် "libm" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
x ဟူသော ဒဿမကိန်း တစ်ခု ကို လည်းကောင်း e ဟူသော ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း လက်ခံပြီး ဒဿမကိန်း တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"က" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0472]: "frexp" is a C function, so its declaration must be unsafe, with "စိတ်မချရသော"
```

On the VM, when we pass a C function a pointer into a freed block, the VM stops the run
with the use-after-free trap before the call. A null pointer reaches C as `NULL`, and the
contract of the C function decides whether `NULL` is allowed: `frexp` with a `NULL`
address crashes inside libm, as it does in C.

A closure can reach C as well. The closure is lent to the call, and a leave lets C keep it
(see [Closures and C](closures.md#closures-and-c)). When C calls a
closure while no call that was given it runs and C has no leave, the program stops with
the trap `closure escape`.

## The checks on unsafe code

The compiler follows each pointer of unsafe code along each path, and reports the
mistakes it can prove. These findings make up the *pointer lint*:

| Code | Finding |
| --- | --- |
| ZK0702 | freeing or resizing through a pointer that can contain the address of a variable, or of a cell or field stored inside a variable, and never a heap block |
| ZK0703 | an address stored where it can outlive the place it points at |
| ZK0704 | a block freed or resized while another pointer still reads it, so that the read uses a freed block; a block freed twice |
| ZK0706 | a function that returns the address of one of its locals or parameters |
| ZK0707 | an argument for an owned parameter that another argument of the call also reaches |
| ZK0708 | a block that this function asked for and never frees, hands to an owned parameter, returns as an owned result, or stores in a heap block, a global or a field |
| ZK0709 | a read or a write through a pointer that is the null address on each path |
| ZK0710 | an offset past the end of a block |
| ZK0711 | a read or a write through a pointer that can be the null address (with `--lint-warn doubts`) |
| ZK0712 | two pointers into different blocks subtracted |
| ZK0721 | the two ranges of `ကူးယူမည်` overlap for certain |

The lint group `mem` gets its level from these options:

| Setting | The lint |
| --- | --- |
| `--lint-warn mem` (the default) | warnings: the program runs, and on the VM the program stops if it reaches the bad access |
| `--lint-error mem` | errors: nothing runs |
| `--lint-allow mem` | nothing: the checks of the VM are the last ones left |

ZK0711 is a warning under both `--lint-error mem` and `--lint-warn mem`. ZK0702 to ZK0708,
ZK0712 and ZK0721 come from pointers alone, so the setting moves them wherever the
compiler finds them. ZK0709, ZK0710 and ZK0711 can also come from safe code, through a
function value or a list. Outside unsafe code these three keep their normal level
whatever the setting, and the setting moves them inside unsafe code alone. We can also
use `--lint-allow`, `--lint-warn` and `--lint-error` to set the level of one group or code
of the lint, over the level of `mem` (see
[Commands](../compiler/commands.md#options-of-the-compile-stages)). ZK0701 is an error in
each mode.

With the default setting, a program that the lint finds fault with still runs, so the VM
can show where it goes wrong:

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

The output above comes from the VM, as with `zakac run --vm`. A plain `zakac run` of the
same program runs it as native code, which reads the freed block and prints whatever it
finds there. Under `--lint-error mem`, the same program does not compile:

```zaka-error ZK0704 lint-error=mem
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
block that the program never freed (`--leaks` runs the program on the VM):

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

The lint cannot find each mistake, because an offset that comes from input, or a pointer
that a C function returned, is out of its reach. [Compiler checks](../compiler/checks.md)
shows one program under each setting.

## What the VM still stops

On the VM, unsafe code is checked as safe code is: the VM checks each read, write and free
through a pointer, and stops the program at the first bad one, with a message that names
the block. It stops these mistakes:

- a read or a write through the null address,
- a read or a write outside each live block, the address just past the end of a block
  included,
- a pointer step that lands further than one past the end of its block, or before its
  start,
- a read or a write in a block that the program freed, or in the frame of a call that has
  returned,
- freeing or resizing an address that is not the start of a live heap block, or freeing a
  block twice,
- a subtraction of two pointers into different blocks, and the overlapping ranges of
  `ကူးယူမည်`,
- a read or a write at an address that is not a multiple of the size of its data kind.

[When a program stops](../vm/traps.md) lists each message.

## Native code

`zakac run` with neither `--vm` nor `--jit` runs a source file as native code when it can,
and on the VM otherwise, and `zakac build --backend c` or `--backend native` builds a native
program. Native code keeps the checks of the language, such as a cell number outside its
list, but a pointer is a plain address there, so the mistakes in the list above behave as
these mistakes do in C: a read through the null address crashes the program, and a read past the
end of a block or in a freed block prints whatever the octets contain. A build can ask for
more checks with `--checks` (see [Native programs](../compiler/native.md)).

To see the checks of the VM, we run the program with `zakac run --vm`. The Explorer and
the tests of these pages also run programs on the VM, and so do the VM flags `--leaks`,
`--frames`, `--max-steps` and `--max-stack`. [Checks in the VM and in native
code](../concepts/checked.md) explains the reason for this split.

## Keeping to safe Zaka

With `--no-unsafe`, the compiler refuses unsafe code, for a lesson or a project that keeps
to safe Zaka. Each unsafe function and each unsafe block is then ZK0471, which is an error
in each mode:

```zaka-error ZK0471 no-unsafe=on
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
"က" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0471]: this program keeps to safe Zaka, so it cannot hold unsafe code ("စိတ်မချရသော")
```

A program that compiles under `--no-unsafe` never meets a pointer mistake, on the VM or in
native code, because the program frees no block itself and no read goes through an
address that the program made.

## Errors at a glance

| Code | Mistake | Page |
| --- | --- | --- |
| ZK0201 | an unsafe block as the program start block | this page |
| ZK0257 | `စိတ်မချရသော` before the program start block | this page |
| ZK0315 | a heap statement used as a value | [The heap](heap.md#heap-statements) |
| ZK0401 | a pointer of one pointer kind where another data kind is expected | [Pointers](pointers.md#the-pointer-kind) |
| ZK0410 | an assignment to something that is not a variable or a value through a pointer | [Pointers](pointers.md#reading-and-writing-through-a-pointer) |
| ZK0411 | the address of a value that has no place, such as `၅` | [Pointers](pointers.md#taking-an-address) |
| ZK0412 | the address of a constant or of a counted-loop variable | [Pointers](pointers.md#taking-an-address) |
| ZK0413 | a read through a value that is not a pointer | [Pointers](pointers.md#reading-and-writing-through-a-pointer) |
| ZK0414 | a read through, or an offset of, the opaque `လိပ်စာ` | [Pointers](pointers.md#reading-and-writing-through-a-pointer) |
| ZK0415 | an operator such as `*` or `/` on a pointer | [Pointers](pointers.md#pointer-arithmetic) |
| ZK0416 | pointers of different pointer kinds compared or subtracted | [Pointers](pointers.md#pointer-arithmetic) |
| ZK0417 | a heap statement given something other than a pointer | [The heap](heap.md#heap-statements) |
| ZK0430 | a declaration with the null address and no data kind | [Pointers](pointers.md#the-null-address) |
| ZK0442 | cells that contain a counted block, in a block write or a fill | [The heap](heap.md#writing-and-filling-cells) |
| ZK0468 | pointer work, or a call of an unsafe function, outside unsafe code | this page |
| ZK0469 | an unsafe function used as a function value | this page |
| ZK0470 | `အပိုင်အနေဖြင့်` on a data kind that is not a pointer | [The heap](heap.md#owned-pointers) |
| ZK0471 | unsafe code under `--no-unsafe` | this page |
| ZK0472 | a C declaration that is not declared unsafe | this page |
| ZK0701 | a read of a place after it handed its block to an owned parameter | [The heap](heap.md#owned-pointers) |
| ZK0702 to ZK0712, ZK0721 | the pointer lint | [above](#the-checks-on-unsafe-code) |

The text of each message is in [Error messages](../reference/errors.md).
