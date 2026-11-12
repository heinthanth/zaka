---
title: Owned pointers
description: Declare a function စိတ်မချရသော, wrap it in a safe function, hand a block over with အပိုင်အနေဖြင့်, and set how the compiler reports pointer mistakes.
---

# Owned pointers

A block from the heap should be given back once, and no pointer should use it after that.
When pointers pass between functions, the headers tell which function gives each block
back. This page shows those headers, the safe functions that wrap unsafe ones, and the
settings of the compiler's pointer checks.

## Unsafe functions

A function whose header starts with `စိတ်မချရသော` is unsafe: each line of it is unsafe code,
and so is each function declared inside it. A call of an unsafe function is unsafe too, so
it needs unsafe code, as in Rust:

```zaka-error ZK0468
// number: a new block that holds 5
စိတ်မချရသော လုပ်ဆောင်ချက် ဂဏန်း သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို အဖြေ တွင် သိမ်းထားမည်။
အဖြေ ၏ တန်ဖိုး ကို ၅ ဟု ထားမည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ဂဏန်း ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။   // outside unsafe code
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0468]: a call of "ဂဏန်း", an unsafe function, needs unsafe code
```

An unsafe function cannot be a [function value](../function-values/README.md) either
(ZK0469), since a call through the value would not show that it is unsafe.

## A safe wrapper

A safe function with an unsafe block inside is a *safe wrapper*: its callers need
no unsafe code. The function promises that it uses its pointers right, and the VM still checks
each access:

```zaka
// number: a new block that holds 5, which the caller owns
စိတ်မချရသော လုပ်ဆောင်ချက် ဂဏန်း သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို အဖြေ တွင် သိမ်းထားမည်။
အဖြေ ၏ တန်ဖိုး ကို ၅ ဟု ထားမည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// clear: take the block over and give it back
စိတ်မချရသော လုပ်ဆောင်ချက် ရှင်း သည်
ပ ဟူသော ကိန်းပြည့် လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// five: a safe wrapper, a plain function with an unsafe block inside
လုပ်ဆောင်ချက် ငါး သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ဂဏန်း ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
အဖြေ ကို ပ ၏ တန်ဖိုး ဟု ထားမည်။
ရှင်း ဟူသော လုပ်ဆောင်ချက်ကို ပ တို့ဖြင့် လုပ်ဆောင်မည်။      // ပ points at nothing from here on
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ငါး ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။     // safe code
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၅
```

## Owned results and parameters

`အပိုင်အနေဖြင့်` ("as its own") tells who owns a block, the one who must give it back:

| In the header | It promises |
| --- | --- |
| result `ကိန်းပြည့် လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်` | "Here is a block. The caller owns it now." |
| parameter `ပ ဟူသော ကိန်းပြည့် လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့်` | "Hand the block to me. I give it back, or pass it on." |
| parameter `ပ ဟူသော ကိန်းပြည့် လိပ်စာ တစ်ခု ကို` | "I look at the block, and can write it. The caller keeps it." |

The word stands after the `ကို` of the parameter, as the marks of
[marked arguments](../functions/parameters.md#marked-arguments) do, and it goes on pointer
kinds alone (ZK0470). After a call that hands a block over, the caller's variable holds the
null address, and a read of it is an error, in each mode:

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
ရှင်း ဟူသော လုပ်ဆောင်ချက်ကို ပ တို့ဖြင့် လုပ်ဆောင်မည်။       // hands the block over
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0701]: "ပ" handed its block to a parameter marked "အပိုင်အနေဖြင့်" and now points at nothing
```

Comparing the variable with `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` is still allowed: it is how a program asks whether the
block has gone. A function that gives back a block it made, with no `အပိုင်အနေဖြင့်` on its
result, gets a warning that names the word:

```zaka-warning ZK0708
စိတ်မချရသော လုပ်ဆောင်ချက် ဂဏန်း သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် လိပ်စာ တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို အဖြေ တွင် သိမ်းထားမည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0708]: the block that "အဖြေ" holds is never given back or handed over on this path
  = help: the caller takes this block: mark the result "<kind> တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်"
```

## How the checks report

In unsafe code, the compiler reports the pointer mistakes it can prove as warnings, and
the program runs; the VM then stops it at the bad access. In safe code, such as a read of a
variable before the program sets it, each finding remains an error. The option `--mem-check` sets the
level for unsafe code:

| Option | In the Explorer's settings | Effect in unsafe code |
| --- | --- | --- |
| `--mem-check warn` | **Ownership check**: **Warn** | the default: each mistake is a warning, and the program runs |
| `--mem-check error` | **Ownership check**: **Error** | each mistake is an error, and nothing runs |
| `--mem-check off` | **Ownership check**: **Off** | the compiler reports none of those mistakes |
| `--mem-strict` | **Strict ownership check** | also warns about a pointer that can hold the null address |

A read after a hand-over (ZK0701) is an error in each mode. The strict check warns where a
pointer can hold the null address, such as a new block never compared with it, since the
heap can run out:

```zaka-warning ZK0711 mem-strict=on
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၁ ဟု ထားမည်။             // the heap can run out, and then ပ is null
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0711]: "ပ" can point at nothing here; compare it with "မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ" first
```

## Keeping to safe Zaka

A lesson or a project can refuse unsafe code with `zakac run --no-unsafe`, or **Forbid
unsafe code** in the Explorer. Each use of it is then an error:

```zaka-error ZK0471 no-unsafe=on
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ကိန်းရှင် ညွှန်း သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အသက် ၏ လိပ်စာ ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0471]: this program keeps to safe Zaka, so it cannot hold unsafe code ("စိတ်မချရသော")
```

## Hands on

- Write a safe wrapper `ပေါင်းလဒ်` that asks for a block of three integers, fills it, adds
  them up, gives the block back and returns the sum.
- Take `အပိုင်အနေဖြင့်` off the parameter of `ရှင်း`, then read `ပ` after the call. What does
  the compiler report now, and where does the VM stop?
- Mark the start block itself with `စိတ်မချရသော`, and read the message.

Next: [C functions and addresses](c.md).
