---
title: Owned pointers
description: Declare a function စိတ်မချရသော, wrap it in a safe function, hand a block over with အပိုင်အနေဖြင့်, and set how the compiler reports pointer mistakes.
---

# Owned pointers

A block from the heap should be given back once, and no pointer should use it after that.
When pointers pass between functions, we mark each owned pointer (owning pointer) in the
function signature, so that each block has one function that is in charge of giving it
back.

## You will learn

- how to declare an unsafe function with `စိတ်မချရသော`
- how a safe function can wrap unsafe code, so that its callers need none
- how to hand a block from one function to another with `အပိုင်အနေဖြင့်`
- how to set what the compiler does with the pointer mistakes it finds, and how to refuse
  unsafe code altogether

```zaka
// number: a new block that contains 5, which the caller owns
ဂဏန်း ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို အဖြေ တွင် သိမ်းထားမည်။
အဖြေ ၏ တန်ဖိုး ကို ၅ ဟု ထားမည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ဂဏန်း ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။ // the caller gives the block back
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
၅
```

`ဂဏန်း` asks the heap for a block and returns it to the caller, and its function signature
tells two things: the function is unsafe, and the caller owns the block it returns. We
write `စိတ်မချရသော` after the name to declare an unsafe function, and `အပိုင်အနေဖြင့်`
before `ပြန်ပေးမည်` to return an owned pointer:

```text
<name> ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ <kind> လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။
```

| Zaka | Reads as |
| --- | --- |
| `ဂဏန်း ဟူသော` | "called `ဂဏန်း`" (number) |
| `စိတ်မချရသော လုပ်ဆောင်ချက်` | "a function that is not to be trusted" |
| `ကိန်းပြည့် လိပ်စာ တစ်ခု ကို` | "an integer address" |
| `အပိုင်အနေဖြင့်` | "as its own" |
| `ပြန်ပေးမည်` | "will return" |

## Unsafe functions

Each line of an unsafe function is unsafe code, and so is each function declared inside
it. A call of an unsafe function is unsafe too, so the call needs unsafe code, as in Rust:

```zaka-error ZK0468
// number: a new block that contains 5
ဂဏန်း ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို အဖြေ တွင် သိမ်းထားမည်။
အဖြေ ၏ တန်ဖိုး ကို ၅ ဟု ထားမည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ပ သည် ဂဏန်း ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။   // outside unsafe code
```

```text
error[ZK0468]: a call of "ဂဏန်း", an unsafe function, needs unsafe code
```

## A safe wrapper

A *safe wrapper* is a plain function with an unsafe block inside, and its callers need no
unsafe code. The function promises that it uses its pointers right, while the VM still
checks each access. Here `ငါး` (five) uses `ဂဏန်း` to get a block, reads it, and hands the
block to `ရှင်း` (clear), which gives it back:

```zaka
// number: a new block that contains 5, which the caller owns
ဂဏန်း ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို အဖြေ တွင် သိမ်းထားမည်။
အဖြေ ၏ တန်ဖိုး ကို ၅ ဟု ထားမည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// clear: take the block over and give it back
ရှင်း ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည်
ပ ဟူသော ကိန်းပြည့် လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// five: a safe wrapper, a plain function with an unsafe block inside
ငါး ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ဂဏန်း ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
အဖြေ ကို ပ ၏ တန်ဖိုး ဟု ထားမည်။
ရှင်း ဟူသော လုပ်ဆောင်ချက်ကို ပ ဖြင့် လုပ်ဆောင်မည်။ // ပ points at nothing from here on
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ငါး ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။ // safe code
```

```text
၅
```

## Owned results and parameters

```text
<kind> လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်
<name> ဟူသော <kind> လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် လက်ခံပြီး
```

We write `အပိုင်အနေဖြင့်` ("as its own") to tell who owns a block, which is the function
that must give it back:

| In the function signature | It promises |
| --- | --- |
| result `ကိန်းပြည့် လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်` | "Here is a block. The caller owns it now." |
| parameter `ပ ဟူသော ကိန်းပြည့် လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့်` | "Hand the block to me. I give it back, or pass it on." |
| parameter `ပ ဟူသော ကိန်းပြည့် လိပ်စာ တစ်ခု ကို` | "I look at the block, and can write it. The caller keeps it." |

For a parameter, the word goes after `ကို`, where a
[preposition](../functions/prepositions.md) can also go. After a call that
hands a block over, the caller's variable has the null address, and a read of it is an
error in each mode of the checks:

```zaka-error ZK0701
ရှင်း ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည်
ပ ဟူသော ကိန်းပြည့် လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ရှင်း ဟူသော လုပ်ဆောင်ချက်ကို ပ ဖြင့် လုပ်ဆောင်မည်။       // hands the block over
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
error[ZK0701]: "ပ" handed its block to a parameter marked "အပိုင်အနေဖြင့်" and now points at nothing
```

We can still compare the variable with `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`, which is how a
program asks whether the block has gone. A function that returns a block it made, with no
`အပိုင်အနေဖြင့်` on its result, gets a warning that names the word:

```zaka-warning ZK0708
ဂဏန်း ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် လိပ်စာ တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို အဖြေ တွင် သိမ်းထားမည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0708]: the block that "အဖြေ" holds is never given back or handed over on this path
  = help: the caller takes this block: mark the result "<kind> တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်"
```

## How the checks report

In unsafe code, the compiler reports the pointer mistakes it can prove as warnings and lets
the program run, and the VM then stops the program at the bad access. In safe code, such
as a read of a variable before the program sets it, each finding remains an error. To set
the level for unsafe code, we use the lint group `mem`:

| Option | In the Explorer's settings | Effect in unsafe code |
| --- | --- | --- |
| `--lint-warn mem` | **Pointer check**: **Warn** | the default: each mistake is a warning, and the program runs |
| `--lint-error mem` | **Pointer check**: **Error** | each mistake is an error, and nothing runs |
| `--lint-allow mem` | **Pointer check**: **Off** | the compiler reports none of these mistakes |
| `--lint-warn doubts` | **Strict pointer check** | also warns about a pointer that can have the null address |

A read after a hand-over (ZK0701) is an error in each mode. The strict check warns where a
pointer can have the null address, such as a new block that the program never compares
with it, because the heap can run out:

```zaka-warning ZK0711 lint-warn=doubts
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၁ ဟု ထားမည်။ // the heap can run out, and then ပ is null
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0711]: "ပ" can point at nothing here; compare it with "မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ" first
```

## Keeping to safe Zaka

A lesson or a project can refuse unsafe code with `zakac run --no-unsafe`, or with
**Forbid unsafe code** in the Explorer's settings, and each use of unsafe code is then an
error:

```zaka-error ZK0471 no-unsafe=on
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ညွှန်း သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အသက် ၏ လိပ်စာ ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
error[ZK0471]: this program keeps to safe Zaka, so it cannot hold unsafe code ("စိတ်မချရသော")
```

## Details

- An unsafe function cannot be a [function value](../function-values/README.md) (ZK0469),
  because a call through the value would not show that it is unsafe ([What needs unsafe
  code](../../manual/language/unsafe.md#what-needs-unsafe-code)).
- `အပိုင်အနေဖြင့်` goes on pointer kinds alone, and on another data kind it is the error
  ZK0470 ([Owned pointers](../../manual/language/heap.md#owned-pointers)).
- A call that hands a block over through one argument while another argument reaches the
  same block gets the warning ZK0707, because the function can give the block back while
  the other argument still uses it ([Owned
  pointers](../../manual/language/heap.md#owned-pointers)).
- The program start block cannot be marked with `စိတ်မချရသော`, so we put an unsafe block
  inside it instead.
- Each lint group and level, and how the three modes treat one program, are in [Compiler
  checks](../../manual/compiler/checks.md).

## Recap

- We write `စိတ်မချရသော` in a function signature to declare an unsafe function, and a call
  of it needs unsafe code.
- A safe wrapper keeps its unsafe code inside an unsafe block, so its callers remain in safe
  code.
- `အပိုင်အနေဖြင့်` on a result hands the block to the caller, and on a parameter it hands
  the block to the function, after which the caller's variable has the null address.
- The lint group `mem` sets whether pointer mistakes in unsafe code are warnings, errors or
  nothing, and `--no-unsafe` refuses unsafe code altogether.

## Hands on

- Write a safe wrapper `ပေါင်းလဒ်` that asks for a block of three integers, fills it, adds
  them up, gives the block back and returns the sum.
- Take `အပိုင်အနေဖြင့်` off the parameter of `ရှင်း`, then read `ပ` after the call. What does
  the compiler report now, and where does the VM stop?
- Mark the program start block itself with `စိတ်မချရသော`, and read the message.

Next: [C functions and addresses](c.md).
